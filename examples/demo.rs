use std::sync::Arc;
use std::collections::{HashMap, HashSet};

use cashu_mint_nip74::MintInfo;
use cdk::mint::{MintBuilder, MintMeltLimits};
use cdk::nuts::{CurrencyUnit, PaymentMethod};
use cdk::types::FeeReserve;
use cdk::nuts as cashu;
use cdk::types::QuoteTTL;
use cdk_fake_wallet::FakeWallet;
use cdk_sqlite::mint::memory;
use nostr::prelude::*;
use purrmint::{handler::DefaultMintHandler, DynSigner, MintService};
use rand::Rng;

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
    // Build an in-memory Cashu Mint instance.
    // ---------------------------------------------------------------------
    let db = Arc::new(memory::empty().await?);

    // Very simple fake LN backend – good enough for demonstration purposes.
    let fee_reserve = FeeReserve {
        min_fee_reserve: cdk::Amount::ZERO,
        percent_fee_reserve: 0.0,
    };
    let fake_ln = FakeWallet::new(fee_reserve, HashMap::new(), HashSet::new(), 0);
    let ln_backend: Arc<dyn cdk::cdk_payment::MintPayment<Err = cdk::cdk_payment::Error> + Send + Sync> =
        Arc::new(fake_ln);

    // Random 32-byte seed for mint keys.
    let seed: Vec<u8> = rand::thread_rng().gen::<[u8; 32]>().to_vec();

    // Compose the Mint.
    let builder = MintBuilder::new()
        .with_seed(seed)
        .with_localstore(db.clone())
        .with_keystore(db.clone());

    // Add the dummy LN backend – Sat / Bolt11 with generous limits.
    let builder = builder
        .add_ln_backend(
            CurrencyUnit::Sat,
            PaymentMethod::Bolt11,
            MintMeltLimits::new(1, 1_000_000),
            ln_backend,
        )
        .await?;

    // Clone the internal Cashu MintInfo so we can store it in DB later.
    let cdk_mint_info: cashu::MintInfo = builder.mint_info.clone();

    let mint = builder.build().await?;

    // Store MintInfo and a default QuoteTTL so Info requests succeed.
    mint.set_mint_info(cdk_mint_info).await?;
    mint.set_quote_ttl(QuoteTTL::new(600, 600)).await?;

    // ---------------------------------------------------------------------
    // NIP-74 MintInfo – advertise demo mint on Nostr.
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
    let relays = vec!["ws://127.0.0.1:7777".parse()?];

    // ---------------------------------------------------------------------
    // Spin up the service with the real Mint handler.
    // ---------------------------------------------------------------------
    let handler = Arc::new(DefaultMintHandler::new(mint));
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