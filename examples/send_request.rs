use std::time::Duration;

use cashu_mint_nip74::{new_request_id, OperationMethod, OperationRequest};
use nostr::prelude::*;
use nostr_sdk::{Client, Options, RelayPoolNotification};
use tokio::time::timeout;

/// Build and send an OperationRequest, then wait for the reply.
///
/// USAGE:
///     cargo run --example send_request -- <MINT_NPUB> [relay_url]
///
/// `MINT_NPUB`  – mint public key (npub...)
/// `relay_url`  – optional, default to wss://relay.0xchat.com
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    const DEFAULT_MINT_PUB: &str = "8157b28f002c90aee5693493c1720918c53d6d0eaa5e9e0c6c5427a137c1efd4"; // hex pk

    // args[0] = program name
    // args[1] = optional mint pubkey
    // args[2] = optional relay url
    let args: Vec<String> = std::env::args().collect();
    let mint_pubkey_str = args.get(1).map(String::as_str).unwrap_or(DEFAULT_MINT_PUB);
    let mint_pubkey = mint_pubkey_str.parse::<PublicKey>()?;
    let relay = args.get(2).cloned().unwrap_or_else(|| "ws://127.0.0.1:7777".to_string());

    // Generate temporary client keys.
    let keys = Keys::parse("5b710e6de48418b70182584fdf06c692bc422478be42729939203b4c2aa496c1")?;
    let client = Client::builder()
        .signer(keys.clone())   
        .opts(Options::default())
        .build();

    client.add_relay(relay).await?;
    client.connect().await;
    // Wait up to 5 seconds for the relay connection to establish.
    client.wait_for_connection(Duration::from_secs(5)).await;

    // Subscribe for OperationResult events addressed to us
    let sub_filter = Filter::new()
        .kind(Kind::from(27402u16))
        .pubkey(keys.public_key()); // 'p' tag contains our pubkey
    client.subscribe(sub_filter, None).await?;

    // Compose request.
    let request = OperationRequest {
        method: OperationMethod::Info,
        request_id: new_request_id(),
        data: serde_json::json!({}),
    };

    // Serialize and encrypt payload with NIP-44.
    let plaintext = serde_json::to_string(&request)?;
    let ciphertext = nostr::nips::nip44::encrypt(keys.secret_key(), &mint_pubkey, plaintext, Default::default())?;

    // Build event.
    let event = EventBuilder::new(Kind::from(27401u16), ciphertext)
        .tag(Tag::public_key(mint_pubkey))
        .sign_with_keys(&keys)?;

    let output = client.send_event(&event).await?;
    println!("27401 sent, success relays: {:?}, failed: {:?}", output.success, output.failed);

    // Wait up to 30 seconds for the result.
    let mut notifications = client.notifications();
    match timeout(Duration::from_secs(30), async move {
        while let Ok(notif) = notifications.recv().await {
            if let RelayPoolNotification::Event { event, .. } = notif {
                if event.kind == Kind::from(27402u16) {
                    // Decrypt and display.
                    if let Ok(decrypted) = nostr::nips::nip44::decrypt(
                        keys.secret_key(),
                        &mint_pubkey,
                        &event.content,
                    ) {
                        println!("Got OperationResult: {decrypted}");
                        return Some(());
                    }
                }
            }
        }
        None
    })
    .await
    {
        Ok(Some(_)) => println!("Done"),
        _ => println!("No reply within timeout"),
    }

    Ok(())
} 