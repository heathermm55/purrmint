use std::sync::Arc;

use async_trait::async_trait;
use cashu_mint_nip74::{MintInfo, OperationRequest, OperationResult, ResultStatus, Result as Nip74Result};
use nostr::prelude::*;
use purrmint::{MintService, RequestHandler, DynSigner};

/// Simple echo handler – returns the same `data` field back to the caller.
struct EchoHandler;

#[async_trait]
impl RequestHandler for EchoHandler {
    async fn handle(&self, req: OperationRequest) -> Nip74Result<OperationResult> {
        Ok(OperationResult {
            status: ResultStatus::Success,
            request_id: req.request_id,
            data: Some(req.data),
            error: None,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize stdout logger.
    tracing_subscriber::fmt::init();

    // ---------------------------------------------------------------------
    // Keys & signer – local random keys are good enough for the demo.
    // ---------------------------------------------------------------------
    let keys = Keys::parse("6ff05f667814b3695ab7d5627c57226ca0534f640ebc7b22cf4759964d6cda68")?;
    let signer: DynSigner = Arc::new(keys.clone());

    eprintln!("signer: {}", signer.get_public_key().await?);

    // ---------------------------------------------------------------------
    // MintInfo – describe our mint and where it lives.
    // ---------------------------------------------------------------------
    let mint_info = MintInfo {
        identifier: "demo-mint".into(),
        name: "PurrMint Demo".into(),
        description: "Demo Cashu mint using NIP-74".into(),
        icon: None,
        version: "0.1.0".into(),
        units: vec!["sat".into()],
        contacts: vec![format!("npub{}", keys.public_key())],
        nuts: vec!["03".into(), "04".into(), "05".into()],
        url: None,
        relays: vec!["ws://127.0.0.1:7777".into()],
        status: "running".into(),
    };

    // ---------------------------------------------------------------------
    // Relay list – the service will connect to those and stay online.
    // ---------------------------------------------------------------------
    let relays = vec![
        "ws://127.0.0.1:7777".parse()?
    ];

    // ---------------------------------------------------------------------
    // Spin up the service.
    // ---------------------------------------------------------------------
    let handler = Arc::new(EchoHandler);
    let mut service = MintService::new(signer, mint_info, relays, handler).await?;
    service.start().await?;

    // ---------------------------------------------------------------------
    // Keep the example alive until Ctrl-C.
    // ---------------------------------------------------------------------
    eprintln!("PurrMint demo running – press Ctrl-C to stop…");
    tokio::signal::ctrl_c().await?;

    service.stop().await?;
    eprintln!("Service stopped – goodbye!");
    Ok(())
} 