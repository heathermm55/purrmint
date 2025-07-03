use std::sync::Arc;

use async_trait::async_trait;
use cdk::mint::Mint;
use serde_json::json;
use cdk::nuts::{MintQuoteBolt11Request, MeltQuoteBolt11Request, MintRequest, MeltRequest};
use serde_json::Value;
use uuid::Uuid;
use serde::de::Error as _;
use reqwest;

use crate::{RequestHandler, Nip74Result, Nip74Error};
use crate::{OperationMethod, OperationRequest, OperationResult, ResultStatus, ResultError};

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
        let url = format!("http://127.0.0.1:{}{}", self.mintd_port, endpoint);
        
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