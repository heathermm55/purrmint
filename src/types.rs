//! NIP-74 types and error definitions

use serde::{Deserialize, Serialize};

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
} 