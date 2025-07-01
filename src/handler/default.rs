use std::sync::Arc;

use async_trait::async_trait;
use cashu_mint_nip74::{OperationMethod, OperationRequest, OperationResult, ResultStatus};
use cdk::mint::Mint;
use serde_json::json;

use crate::{RequestHandler, Nip74Result};

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
                        error: Some(cashu_mint_nip74::ResultError {
                            code: "info_failed".into(),
                            message: e.to_string(),
                        }),
                    }),
                }
            }
            _ => Ok(OperationResult {
                status: ResultStatus::Error,
                request_id: req.request_id,
                data: None,
                error: Some(cashu_mint_nip74::ResultError {
                    code: "unsupported".into(),
                    message: format!("method {:?} not implemented", req.method),
                }),
            }),
        }
    }
} 