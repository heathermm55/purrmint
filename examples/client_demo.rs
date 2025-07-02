use std::time::Duration;

use purrmint::{new_request_id, OperationMethod, OperationRequest};
use nostr::prelude::*;
use nostr_sdk::{Client, Options, RelayPoolNotification};
use tokio::time::timeout;

/// Build and send an OperationRequest, then wait for the reply.
///
/// USAGE:
///     cargo run --example client_demo -- <MINT_NPUB> [relay_url] [operation]
///
/// `MINT_NPUB`  – mint public key (npub...)
/// `relay_url`  – optional, default to ws://127.0.0.1:7777
/// `operation`  – optional, one of: info, get_mint_quote, check_mint_quote, mint, get_melt_quote, check_melt_quote, melt
///                default: info
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    const DEFAULT_MINT_PUB: &str = "8157b28f002c90aee5693493c1720918c53d6d0eaa5e9e0c6c5427a137c1efd4"; // hex pk

    // args[0] = program name
    // args[1] = optional mint pubkey
    // args[2] = optional relay url
    // args[3] = optional operation type
    let args: Vec<String> = std::env::args().collect();
    
    // Show help if no arguments or help requested
    if args.len() == 1 || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        println!("PurrMint NIP-74 Client Example");
        println!();
        println!("USAGE:");
        println!("    cargo run --example client_demo -- <MINT_NPUB> [relay_url] [operation]");
        println!();
        println!("ARGUMENTS:");
        println!("    MINT_NPUB    mint public key (hex format)");
        println!("    relay_url    optional relay URL, default: ws://127.0.0.1:7777");
        println!("    operation    optional operation type, default: info");
        println!();
        println!("OPERATIONS:");
        println!("    info              get mint information");
        println!("    get_mint_quote    request mint quote for 1000 sats");
        println!("    check_mint_quote  check mint quote status (uses dummy UUID)");
        println!("    mint              mint tokens (uses dummy data)");
        println!("    get_melt_quote    request melt quote for Lightning invoice");
        println!("    check_melt_quote  check melt quote status (uses dummy UUID)");
        println!("    melt              melt tokens (uses dummy data)");
        println!();
        println!("EXAMPLES:");
        println!("    cargo run --example client_demo");
        println!("    cargo run --example client_demo 8157b28f002c90aee5693493c1720918c53d6d0eaa5e9e0c6c5427a137c1efd4");
        println!("    cargo run --example client_demo 8157b28f002c90aee5693493c1720918c53d6d0eaa5e9e0c6c5427a137c1efd4 ws://127.0.0.1:7777 get_mint_quote");
        return Ok(());
    }
    
    let mint_pubkey_str = args.get(1).map(String::as_str).unwrap_or(DEFAULT_MINT_PUB);
    let mint_pubkey = mint_pubkey_str.parse::<PublicKey>()?;
    let relay = args.get(2).cloned().unwrap_or_else(|| "ws://127.0.0.1:7777".to_string());
    let operation = args.get(3).map(String::as_str).unwrap_or("info");

    println!("Using Mint public key: {}", mint_pubkey);
    println!("Connecting to Relay: {}", relay);
    println!("Operation: {}", operation);

    // Generate temporary client keys.
    let keys = Keys::parse("5b710e6de48418b70182584fdf06c692bc422478be42729939203b4c2aa496c1")?;
    println!("Client public key: {}", keys.public_key());
    
    let client = Client::builder()
        .signer(keys.clone())   
        .opts(Options::default())
        .build();

    client.add_relay(relay).await?;
    client.connect().await;
    // Wait up to 5 seconds for the relay connection to establish.
    client.wait_for_connection(Duration::from_secs(5)).await;
    println!("Connected to relay");

    // Subscribe for OperationResult events addressed to us
    let sub_filter = Filter::new()
        .kind(Kind::from(27402u16))
        .pubkey(keys.public_key()); // 'p' tag contains our pubkey
    client.subscribe(sub_filter, None).await?;
    println!("Subscribed to 27402 events");

    // Compose request based on operation type
    let request_id = new_request_id();
    println!("Request ID: {}", request_id);
    
    let request = match operation {
        "info" => OperationRequest {
            method: OperationMethod::Info,
            request_id: request_id,
            data: Some(serde_json::json!({})),
        },
        "get_mint_quote" => OperationRequest {
            method: OperationMethod::GetMintQuote,
            request_id: request_id,
            data: Some(serde_json::json!({
                "amount": 1000,
                "unit": "sat"
            })),
        },
        "check_mint_quote" => OperationRequest {
            method: OperationMethod::CheckMintQuote,
            request_id: request_id,
            data: Some(serde_json::json!("00000000-0000-0000-0000-000000000000")), // dummy UUID
        },
        "mint" => OperationRequest {
            method: OperationMethod::Mint,
            request_id: request_id,
            data: Some(serde_json::json!({
                "quote": "00000000-0000-0000-0000-000000000000", // dummy UUID
                "outputs": []
            })),
        },
        "get_melt_quote" => OperationRequest {
            method: OperationMethod::GetMeltQuote,
            request_id: request_id,
            data: Some(serde_json::json!({
                "request": "lnbc100n1pnvpufspp5djn8hrq49r8cghwye9kqw752qjncwyfnrprhprpqk43mwcy4yfsqdq5g9kxy7fqd9h8vmmfvdjscqzzsxqyz5vqsp5uhpjt36rj75pl7jq2sshaukzfkt7uulj456s4mh7uy7l6vx7lvxs9qxpqysgqedwz08acmqwtk8g4vkwm2w78suwt2qyzz6jkkwcgrjm3r3hs6fskyhvud4fan3keru7emjm8ygqpcrwtlmhfjfmer3afs5hhwamgr4cqtactdq",
                "unit": "sat"
            })),
        },
        "check_melt_quote" => OperationRequest {
            method: OperationMethod::CheckMeltQuote,
            request_id: request_id,
            data: Some(serde_json::json!("00000000-0000-0000-0000-000000000000")), // dummy UUID
        },
        "melt" => OperationRequest {
            method: OperationMethod::Melt,
            request_id: request_id,
            data: Some(serde_json::json!({
                "quote": "00000000-0000-0000-0000-000000000000", // dummy UUID
                "inputs": []
            })),
        },
        _ => {
            eprintln!("Unknown operation: {}. Available operations: info, get_mint_quote, check_mint_quote, mint, get_melt_quote, check_melt_quote, melt", operation);
            eprintln!("Run with --help for usage information");
            std::process::exit(1);
        }
    };

    println!("Request content: {}", serde_json::to_string_pretty(&request)?);

    // Serialize and encrypt payload with NIP-44.
    let plaintext = serde_json::to_string(&request)?;
    // Use NIP-44 encryption
    let ciphertext = nostr::nips::nip44::encrypt(keys.secret_key(), &mint_pubkey, plaintext, Default::default())?;

    // Build event.
    let event = EventBuilder::new(Kind::from(27401u16), ciphertext)
        .tag(Tag::public_key(mint_pubkey))
        .sign_with_keys(&keys)?;

    println!("Event ID: {}", event.id);
    println!("Event tags: {:?}", event.tags);

    let output = client.send_event(&event).await?;
    println!("27401 sent, success relays: {:?}, failed: {:?}", output.success, output.failed);

    // Wait up to 30 seconds for the result.
    let mut notifications = client.notifications();
    match timeout(Duration::from_secs(30), async move {
        while let Ok(notif) = notifications.recv().await {
            if let RelayPoolNotification::Event { event, .. } = notif {
                println!("Received event: kind={}, id={}", event.kind, event.id);
                if event.kind == Kind::from(27402u16) {
                    println!("Received 27402 response event, tags: {:?}", event.tags);
                    // Decrypt with NIP-44
                    if let Ok(decrypted) = nostr::nips::nip44::decrypt(
                        keys.secret_key(),
                        &mint_pubkey,
                        &event.content,
                    ) {
                        println!("Decryption successful! OperationResult: {decrypted}");
                        return Some(());
                    } else {
                        println!("Decryption failed!");
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