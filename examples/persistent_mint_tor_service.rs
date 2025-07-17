//! Persistent Mint Tor Service
//! 
//! This example runs a mint service with Tor hidden service continuously
//! so you can test the onion address accessibility

use anyhow::Result;
use purrmint::config::{AndroidConfig, TorConfig};
use purrmint::mintd_service::MintdService;
use purrmint::tor_service::TorService;
use std::path::PathBuf;
use std::time::Duration;
use tokio::signal;
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("=== Persistent Mint Tor Service ===");
    println!("Starting mint service with Tor hidden service...\n");
    
    // Step 1: Start Tor service
    println!("🔧 Starting Tor service...");
    let mut tor_config = TorConfig::custom(
        "/tmp/persistent_mint_tor".to_string(),
        9050,
        true,
    );
    tor_config.use_bridges = false; // Use direct connection for testing
    tor_config.connection_timeout = 120;
    tor_config.enable_logging = true;
    tor_config.log_level = "info".to_string();
    
    let mut tor_service = TorService::with_config(tor_config)?;
    println!("   ✅ TorService created");
    
    // Start Tor client
    println!("🚀 Starting Tor client...");
    let start_time = std::time::Instant::now();
    match tokio::time::timeout(Duration::from_secs(60), tor_service.start()).await {
        Ok(Ok(_)) => {
            let elapsed = start_time.elapsed();
            println!("   ✅ Tor client started in {:?}", elapsed);
        }
        Ok(Err(e)) => {
            let elapsed = start_time.elapsed();
            println!("   ❌ Failed to start Tor client after {:?}: {}", elapsed, e);
            return Ok(());
        }
        Err(_) => {
            let elapsed = start_time.elapsed();
            println!("   ⏰ Tor client startup timed out after {:?}", elapsed);
            return Ok(());
        }
    }
    
    // Test Tor connection
    println!("🌐 Testing Tor connection...");
    match tor_service.test_connection().await {
        Ok(success) => {
            if success {
                println!("   ✅ Tor connection successful");
            } else {
                println!("   ❌ Tor connection failed");
                return Ok(());
            }
        }
        Err(e) => {
            println!("   ❌ Tor connection error: {}", e);
            return Ok(());
        }
    }
    
    // Step 2: Create and start local mint service
    println!("\n🏦 Creating local mint service...");
    
    // Generate a test nsec (in production, use a real one)
    let test_nsec = "5b710e6de48418b70182584fdf06c692bc422478be42729939203b4c2aa496c1";
    
    // Create Android config for mint service
    let android_config = AndroidConfig {
        port: 3338,
        host: "127.0.0.1".to_string(),
        mint_name: "Persistent Tor Mint".to_string(),
        description: "Persistent mint service with Tor hidden service".to_string(),
        lightning_backend: "fakewallet".to_string(),
        mode: "mintd_only".to_string(),
        database_path: "/tmp/persistent_mint_db".to_string(),
        logs_path: "/tmp/persistent_mint_logs".to_string(),
        lnbits_admin_api_key: None,
        lnbits_invoice_api_key: None,
        lnbits_api_url: None,
        cln_rpc_path: None,
        cln_bolt12: None,
        tor_enabled: Some(true),
        tor_mode: Some("custom".to_string()),
        tor_data_dir: Some("/tmp/persistent_mint_tor_data".to_string()),
        tor_socks_port: Some(9050),
        tor_enable_hidden_services: Some(true),
        tor_num_intro_points: Some(3),
        tor_bridges: Some(vec![]),
        tor_use_bridges: Some(false),
    };
    
    let work_dir = PathBuf::from("/tmp/persistent_mint_work");
    let mut mint_service = MintdService::new_with_android_config(
        work_dir.clone(),
        &android_config,
        test_nsec.to_string(),
    );
    
    println!("   - Work directory: {:?}", work_dir);
    println!("   - Service port: {}", android_config.port);
    println!("   - Tor enabled: {}", android_config.tor_enabled.unwrap_or(false));
    println!();
    
    // Start mint service
    println!("🚀 Starting mint service...");
    match mint_service.start().await {
        Ok(_) => {
            println!("   ✅ Mint service started successfully");
        }
        Err(e) => {
            println!("   ❌ Failed to start mint service: {}", e);
            return Ok(());
        }
    }
    
    // Step 3: Generate Tor hidden service address
    println!("\n🧅 Generating Tor hidden service address...");
    
    // Wait a bit for the service to fully start
    tokio::time::sleep(Duration::from_secs(3)).await;
    
    // Create a Tor hidden service for the mint
    let mint_pubkey = "8157b28f002c90aee5693493c1720918c53d6d0eaa5e9e0c6c5427a137c1efd4"; // Test pubkey
    println!("   🔑 Using mint pubkey: {}", mint_pubkey);
    
    let hidden_service_info = match tor_service.create_hidden_service_for_mint(mint_pubkey).await {
        Ok(info) => {
            println!("   ✅ Tor hidden service created successfully!");
            println!("   🧅 Onion Address: {}", info.onion_address);
            println!("   📛 Service Nickname: {}", info.nickname);
            println!("   📊 Status: {:?}", info.status);
            info
        }
        Err(e) => {
            println!("   ❌ Failed to create Tor hidden service: {}", e);
            return Ok(());
        }
    };
    
    // Step 4: Test mint info endpoint
    println!("\n📡 Testing mint info endpoint...");
    match tor_service.make_tor_request(&format!("http://127.0.0.1:{}/v1/info", android_config.port)).await {
        Ok(response) => {
            println!("   ✅ Mint info request successful");
            println!("   📄 Response: {}", response);
            
            // Try to parse JSON and extract useful info
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response) {
                if let Some(info) = json.as_object() {
                    if let Some(name) = info.get("name") {
                        println!("   📛 Mint Name: {}", name);
                    }
                    if let Some(description) = info.get("description") {
                        println!("   📝 Description: {}", description);
                    }
                    if let Some(version) = info.get("version") {
                        println!("   🔢 Version: {}", version);
                    }
                }
            }
        }
        Err(e) => {
            println!("   ❌ Mint info request failed: {}", e);
        }
    }
    
    // Step 5: Display service information
    println!("🎉 SERVICE IS NOW RUNNING!");
    println!();
    println!("📋 Service Information:");
    println!("   🏠 Local Address: http://127.0.0.1:{}", android_config.port);
    println!("   🧅 Tor Onion Address: http://{}", hidden_service_info.onion_address);
    println!("   📛 Service Name: {}", android_config.mint_name);
    println!("   🔑 Mint Pubkey: {}", mint_pubkey);
    println!();
    println!("🔗 Test URLs:");
    println!("   Local Info: http://127.0.0.1:{}/v1/info", android_config.port);
    println!("   Tor Info: http://{}/v1/info", hidden_service_info.onion_address);
    println!("   Local Keys: http://127.0.0.1:{}/v1/keys", android_config.port);
    println!("   Tor Keys: http://{}/v1/keys", hidden_service_info.onion_address);
    println!();
    println!("💡 Instructions:");
    println!("   1. Test local access: curl http://127.0.0.1:{}/v1/info", android_config.port);
    println!("   2. Test Tor access: curl --socks5 127.0.0.1:9050 http://{}/v1/info", hidden_service_info.onion_address);
    println!("   3. Use Tor Browser to visit: http://{}", hidden_service_info.onion_address);
    println!();
    println!("⏹️  Press Ctrl+C to stop the service");
    println!();
    
    // Step 6: Keep the service running
    info!("Service is running. Press Ctrl+C to stop.");
    
    // Wait for shutdown signal
    match signal::ctrl_c().await {
        Ok(()) => {
            println!("\n🛑 Received shutdown signal...");
        }
        Err(err) => {
            error!("Unable to listen for shutdown signal: {}", err);
        }
    }
    
    // Step 7: Clean shutdown
    println!("🛑 Stopping services...");
    
    // Stop mint service
    match mint_service.stop().await {
        Ok(_) => println!("   ✅ Mint service stopped"),
        Err(e) => println!("   ❌ Failed to stop mint service: {}", e),
    }
    
    // Stop Tor service
    match tor_service.stop().await {
        Ok(_) => println!("   ✅ Tor service stopped"),
        Err(e) => println!("   ❌ Failed to stop Tor service: {}", e),
    }
    
    println!("\n👋 Service stopped. Goodbye!");
    
    Ok(())
} 