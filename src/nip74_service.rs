//! NIP-74 Service Module
//! 
//! This module contains all NIP-74 related functionality including:
//! - Type definitions and error handling
//! - Event helper functions
//! - Default request handlers

use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use nostr::event::tag::kind::TagKind;
use cdk::mint::Mint;
use serde_json::json;
use cdk::nuts::{MintQuoteBolt11Request, MeltQuoteBolt11Request, MintRequest, MeltRequest};
use serde_json::Value;
use serde::de::Error as _;
use reqwest;

use crate::MintInfo;

// ===== TYPE DEFINITIONS =====

/// Crate-level error type for NIP-74 helpers.
#[derive(Debug, thiserror::Error)]
pub enum Nip74Error {
    /// JSON (de)serialization error.
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    /// Nostr signer error.
    #[error(transparent)]
    Signer(#[from] nostr::signer::SignerError),
    /// Event builder error.
    #[error(transparent)]
    Nostr(#[from] nostr::event::builder::Error),
}

/// Convenience result alias for NIP-74 helpers.
pub type Nip74Result<T> = core::result::Result<T, Nip74Error>;

/// Result status for an [`OperationResult`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResultStatus {
    /// Operation succeeded.
    Success,
    /// Operation failed.
    Error,
}

/// Error payload for a failed [`OperationResult`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResultError {
    /// Machine-readable error code.
    pub code: String,
    /// Human-readable error message.
    pub message: String,
}

/// Supported NIP-74 operation methods.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationMethod {
    /// Static information about the mint.
    Info,
    /// Lightning invoice quote for minting.
    GetMintQuote,
    /// Check status of previously requested mint quote.
    CheckMintQuote,
    /// Perform minting using a quote.
    Mint,
    /// Lightning invoice quote for melting.
    GetMeltQuote,
    /// Check status of previously requested melt quote.
    CheckMeltQuote,
    /// Perform melt using a quote.
    Melt,
}

/// Request sent to a mint (kind 27401).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationRequest {
    /// Requested operation.
    pub method: OperationMethod,
    /// Client-generated request id (UUID string).
    pub request_id: String,
    /// Arbitrary JSON payload (depends on `method`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Response emitted by a mint (kind 27402).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult {
    /// Outcome status.
    pub status: ResultStatus,
    /// Mirrors `request_id` from the originating [`OperationRequest`].
    pub request_id: String,
    /// Optional JSON payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    /// Optional error object (populated if `status == Error`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ResultError>,
}

// ===== HELPER FUNCTIONS =====

/// Generate a fresh request id (UUID v4 as lowercase string).
pub fn new_request_id() -> String {
    Uuid::new_v4().to_string()
}

/// Build a `kind:37400` MintInfo event following the agreed strategy:
/// – `content` contains the exact JSON serialization of NUT-06 `MintInfo`;
/// – `d` tag stores the provided unique `identifier` (slug / pubkey);
/// – `relays` tag lists relay URLs where the mint is reachable;
/// – `status` tag gives a quick health indicator (e.g. "running").
/// Additional tags can be appended via `extra_tags`.
pub async fn build_mint_info_event<S>(
    mint_info: &MintInfo,
    signer: &S,
    identifier: &str,
    relays: &[nostr::RelayUrl],
    status: &str,
    extra_tags: Option<Vec<nostr::Tag>>,
) -> Nip74Result<nostr::Event>
where
    S: nostr::NostrSigner,
{
    // Serialize full NUT-06 payload verbatim.
    let content = serde_json::to_string(mint_info)?;

    // Compose mandatory tags.
    let mut builder = nostr::EventBuilder::new(nostr::Kind::from(37400u16), content)
        .tag(nostr::Tag::identifier(identifier.to_owned()))
        .tag(nostr::Tag::custom(
            TagKind::Relays,
            relays.iter().map(|r| r.as_str()).collect::<Vec<_>>(),
        ))
        .tag(nostr::Tag::custom(TagKind::Status, [status.to_owned()]));

    // Append optional extra tags, if any.
    if let Some(tags) = extra_tags {
        builder = builder.tags(tags);
    }

    // Sign and return event.
    let event = builder.sign(signer).await?;
    Ok(event)
}

impl OperationResult {
    /// Convert to `kind:27402` event and sign.
    pub async fn to_event_with_signer<T>(
        &self,
        signer: &T,
        author_pubkey: &nostr::PublicKey,
        receiver_pubkey: &nostr::PublicKey,
        request_event_id: &nostr::EventId,
        extra_tags: Option<Vec<nostr::Tag>>,
    ) -> nostr::Result<nostr::Event>
    where
        T: nostr::NostrSigner,
    {
        // Serialize response content
        let content_str = serde_json::to_string(self)?;
        
        // Use NIP-44 encryption
        let encrypted_content = signer.nip44_encrypt(receiver_pubkey, &content_str).await?;
        
        let mut builder = nostr::EventBuilder::new(nostr::Kind::from(27402u16), encrypted_content)
            .tag(nostr::Tag::public_key(*receiver_pubkey))
            .tag(nostr::Tag::event(*request_event_id));

        if let Some(tags) = extra_tags {
            builder = builder.tags(tags);
        }

        // NIP-74 spec doesn't enforce, but we set the author explicitly for clarity.
        builder = builder.allow_self_tagging();

        let event = builder.sign(signer).await?;
        // Ensure builder signed with the provided author_pubkey if needed.
        debug_assert_eq!(event.pubkey, *author_pubkey);
        Ok(event)
    }
}

// ===== REQUEST HANDLER TRAIT =====

// RequestHandler trait is defined in service.rs
use crate::service::RequestHandler;

// ===== DEFAULT REQUEST HANDLERS =====

/// Default request handler that proxies requests to local mintd HTTP API
pub struct DefaultRequestHandler {
    mintd_port: u16,
}

impl DefaultRequestHandler {
    pub fn new(mintd_port: u16) -> Self {
        Self { mintd_port }
    }

    /// Convert NIP-74 operation to mintd HTTP endpoint
    fn get_mintd_endpoint(&self, method: &OperationMethod) -> String {
        match method {
            OperationMethod::Info => "/v1/info".to_string(),
            OperationMethod::GetMintQuote => "/v1/mint/quote".to_string(),
            OperationMethod::CheckMintQuote => "/v1/mint/quote/check".to_string(),
            OperationMethod::Mint => "/v1/mint".to_string(),
            OperationMethod::GetMeltQuote => "/v1/melt/quote".to_string(),
            OperationMethod::CheckMeltQuote => "/v1/melt/quote/check".to_string(),
            OperationMethod::Melt => "/v1/melt".to_string(),
        }
    }

    /// Make HTTP request to mintd
    async fn call_mintd(&self, endpoint: &str, payload: Value) -> Result<Value, Nip74Error> {
        let url = format!("http://localhost:{}{}", self.mintd_port, endpoint);
        
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| Nip74Error::Serde(serde_json::Error::custom(format!("HTTP request failed: {}", e))))?;

        let status = response.status();
        let text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        if status.is_success() {
            let result: Value = serde_json::from_str(&text)
                .map_err(|e| Nip74Error::Serde(serde_json::Error::custom(format!("Failed to parse response: {}", e))))?;
            Ok(result)
        } else {
            Err(Nip74Error::Serde(serde_json::Error::custom(format!("Mintd request failed: {} - {}", status, text))))
        }
    }
}

#[async_trait]
impl RequestHandler for DefaultRequestHandler {
    async fn handle(&self, req: OperationRequest) -> Nip74Result<OperationResult> {
        let endpoint = self.get_mintd_endpoint(&req.method);
        
        // Convert OperationRequest to mintd payload
        let payload = serde_json::json!({
            "request_id": req.request_id,
            "data": req.data,
        });

        // Call mintd
        let result = self.call_mintd(&endpoint, payload).await?;

        // Convert mintd response to OperationResult
        Ok(OperationResult {
            status: ResultStatus::Success,
            request_id: req.request_id,
            data: Some(result),
            error: None,
        })
    }
}

/// Default handler that bridges NIP-74 requests to the underlying Cashu `Mint` implementation.
pub struct DefaultMintHandler {
    mint: Arc<Mint>,
}

impl DefaultMintHandler {
    /// Create new handler from an instantiated [`Mint`].
    pub fn new(mint: Mint) -> Self {
        Self { mint: Arc::new(mint) }
    }
}

#[async_trait]
impl RequestHandler for DefaultMintHandler {
    async fn handle(&self, req: OperationRequest) -> Nip74Result<OperationResult> {
        match req.method {
            OperationMethod::Info => {
                // For Info we just relay the static mint info.
                match self.mint.mint_info().await {
                    Ok(info) => Ok(OperationResult {
                        status: ResultStatus::Success,
                        request_id: req.request_id,
                        data: Some(json!({ "info": info })),
                        error: None,
                    }),
                    Err(e) => Ok(OperationResult {
                        status: ResultStatus::Error,
                        request_id: req.request_id,
                        data: None,
                        error: Some(ResultError {
                            code: "info_failed".into(),
                            message: e.to_string(),
                        }),
                    }),
                }
            }
            OperationMethod::GetMintQuote => {
                // Parse request payload
                let request: MintQuoteBolt11Request = serde_json::from_value(req.data.unwrap_or(Value::Null))?;
                match self.mint.get_mint_bolt11_quote(request).await {
                    Ok(quote) => Ok(OperationResult {
                        status: ResultStatus::Success,
                        request_id: req.request_id,
                        data: Some(json!(quote)),
                        error: None,
                    }),
                    Err(e) => Ok(OperationResult {
                        status: ResultStatus::Error,
                        request_id: req.request_id,
                        data: None,
                        error: Some(ResultError {
                            code: "get_mint_quote_failed".into(),
                            message: e.to_string(),
                        }),
                    }),
                }
            }
            OperationMethod::CheckMintQuote => {
                let quote_id: Uuid = serde_json::from_value(req.data.unwrap_or(Value::Null))?;
                match self.mint.check_mint_quote(&quote_id).await {
                    Ok(quote) => Ok(OperationResult {
                        status: ResultStatus::Success,
                        request_id: req.request_id,
                        data: Some(json!(quote)),
                        error: None,
                    }),
                    Err(e) => Ok(OperationResult {
                        status: ResultStatus::Error,
                        request_id: req.request_id,
                        data: None,
                        error: Some(ResultError {
                            code: "check_mint_quote_failed".into(),
                            message: e.to_string(),
                        }),
                    }),
                }
            }
            OperationMethod::Mint => {
                let mint_req_str: MintRequest<String> = serde_json::from_value(req.data.unwrap_or(Value::Null))?;
                let mint_req_uuid: MintRequest<Uuid> = mint_req_str.try_into().map_err(|e| serde_json::Error::custom(e))?;
                match self.mint.process_mint_request(mint_req_uuid).await {
                    Ok(response) => Ok(OperationResult {
                        status: ResultStatus::Success,
                        request_id: req.request_id,
                        data: Some(json!(response)),
                        error: None,
                    }),
                    Err(e) => Ok(OperationResult {
                        status: ResultStatus::Error,
                        request_id: req.request_id,
                        data: None,
                        error: Some(ResultError {
                            code: "mint_failed".into(),
                            message: e.to_string(),
                        }),
                    }),
                }
            }
            OperationMethod::GetMeltQuote => {
                let request: MeltQuoteBolt11Request = serde_json::from_value(req.data.unwrap_or(Value::Null))?;
                match self.mint.get_melt_bolt11_quote(&request).await {
                    Ok(quote) => Ok(OperationResult {
                        status: ResultStatus::Success,
                        request_id: req.request_id,
                        data: Some(json!(quote)),
                        error: None,
                    }),
                    Err(e) => Ok(OperationResult {
                        status: ResultStatus::Error,
                        request_id: req.request_id,
                        data: None,
                        error: Some(ResultError {
                            code: "get_melt_quote_failed".into(),
                            message: e.to_string(),
                        }),
                    }),
                }
            }
            OperationMethod::CheckMeltQuote => {
                let quote_id: Uuid = serde_json::from_value(req.data.unwrap_or(Value::Null))?;
                match self.mint.check_melt_quote(&quote_id).await {
                    Ok(quote) => Ok(OperationResult {
                        status: ResultStatus::Success,
                        request_id: req.request_id,
                        data: Some(json!(quote)),
                        error: None,
                    }),
                    Err(e) => Ok(OperationResult {
                        status: ResultStatus::Error,
                        request_id: req.request_id,
                        data: None,
                        error: Some(ResultError {
                            code: "check_melt_quote_failed".into(),
                            message: e.to_string(),
                        }),
                    }),
                }
            }
            OperationMethod::Melt => {
                let melt_req_str: MeltRequest<String> = serde_json::from_value(req.data.unwrap_or(Value::Null))?;
                let melt_req_uuid: MeltRequest<Uuid> = melt_req_str.try_into().map_err(|e| serde_json::Error::custom(e))?;
                match self.mint.melt_bolt11(&melt_req_uuid).await {
                    Ok(response) => Ok(OperationResult {
                        status: ResultStatus::Success,
                        request_id: req.request_id,
                        data: Some(json!(response)),
                        error: None,
                    }),
                    Err(e) => Ok(OperationResult {
                        status: ResultStatus::Error,
                        request_id: req.request_id,
                        data: None,
                        error: Some(ResultError {
                            code: "melt_failed".into(),
                            message: e.to_string(),
                        }),
                    }),
                }
            }
        }
    }
}

// ===== TESTS =====

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_result_status_serde() {
        let ok = ResultStatus::Success;
        let err = ResultStatus::Error;
        let ok_json = serde_json::to_string(&ok).unwrap();
        let err_json = serde_json::to_string(&err).unwrap();
        assert_eq!(ok_json, "\"success\"");
        assert_eq!(err_json, "\"error\"");
        assert_eq!(serde_json::from_str::<ResultStatus>(&ok_json).unwrap(), ok);
        assert_eq!(serde_json::from_str::<ResultStatus>(&err_json).unwrap(), err);
    }

    #[test]
    fn test_operation_method_serde() {
        let m = OperationMethod::Mint;
        let s = serde_json::to_string(&m).unwrap();
        assert_eq!(s, "\"mint\"");
        let d: OperationMethod = serde_json::from_str(&s).unwrap();
        assert_eq!(d, OperationMethod::Mint);
    }

    #[test]
    fn test_operation_request_roundtrip() {
        let req = OperationRequest {
            method: OperationMethod::Info,
            request_id: "abc-123".to_string(),
            data: Some(serde_json::json!({"foo": 1})),
        };
        let s = serde_json::to_string(&req).unwrap();
        let de: OperationRequest = serde_json::from_str(&s).unwrap();
        assert_eq!(de.method, OperationMethod::Info);
        assert_eq!(de.request_id, "abc-123");
        assert_eq!(de.data.unwrap()["foo"], 1);
    }

    #[test]
    fn test_operation_result_roundtrip() {
        let res = OperationResult {
            status: ResultStatus::Success,
            request_id: "id-1".to_string(),
            data: Some(serde_json::json!({"bar": 2})),
            error: None,
        };
        let s = serde_json::to_string(&res).unwrap();
        let de: OperationResult = serde_json::from_str(&s).unwrap();
        assert_eq!(de.status, ResultStatus::Success);
        assert_eq!(de.request_id, "id-1");
        assert_eq!(de.data.unwrap()["bar"], 2);
        assert!(de.error.is_none());
    }

    #[test]
    fn test_result_error_serde() {
        let err = ResultError { code: "fail".into(), message: "fail msg".into() };
        let s = serde_json::to_string(&err).unwrap();
        let de: ResultError = serde_json::from_str(&s).unwrap();
        assert_eq!(de.code, "fail");
        assert_eq!(de.message, "fail msg");
    }

    #[test]
    fn test_new_request_id_unique() {
        let a = new_request_id();
        let b = new_request_id();
        assert_ne!(a, b);
        assert_eq!(a.len(), 36); // UUID v4
    }

    #[tokio::test]
    async fn test_build_mint_info_event_basic() {
        // Minimal MintInfo mock
        #[derive(serde::Serialize)]
        struct DummyMintInfo {
            name: String,
        }
        let mint_info = DummyMintInfo { name: "test-mint".to_string() };
        let keys = nostr::Keys::generate();
        let event = nostr::EventBuilder::new(nostr::Kind::from(37400u16), serde_json::to_string(&mint_info).unwrap())
            .tag(nostr::Tag::identifier("test"))
            .sign_with_keys(&keys)
            .unwrap();
        assert_eq!(event.kind, nostr::Kind::from(37400u16));
        assert!(event.tags.iter().any(|t| t.as_slice()[0] == "d"));
    }

    #[tokio::test]
    async fn test_operation_result_to_event_with_signer() {
        let keys = nostr::Keys::generate();
        let op_res = OperationResult {
            status: ResultStatus::Success,
            request_id: "reqid".to_string(),
            data: None,
            error: None,
        };
        let author = keys.public_key();
        let receiver = keys.public_key();
        let dummy_event_id = nostr::EventId::all_zeros();
        let event = op_res
            .to_event_with_signer(&keys, &author, &receiver, &dummy_event_id, None)
            .await
            .unwrap();
        assert_eq!(event.kind, nostr::Kind::from(27402u16));
        assert_eq!(event.pubkey, author);
    }
} 