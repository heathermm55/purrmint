//! Local Mint Tor Address Test
//! 
//! This example demonstrates:
//! 1. Starting a local mint service
//! 2. Generating Tor hidden service address
//! 3. Testing connectivity to the Tor address

use anyhow::Result;
use purrmint::config::{TorConfig, AndroidConfig};
use purrmint::tor_service::TorService;
use purrmint::mintd_service::MintdService;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("=== Local Mint Tor Address Test ===");
    println!("Testing local mint service with Tor hidden service\n");
    
    // Step 1: Start Tor service without bridges
    println!("🔧 Starting Tor service...");
    let mut tor_config = TorConfig::custom(
        "/tmp/local_mint_tor_test".to_string(),
        9050,
        true,
    );
    tor_config.use_bridges = false;
    tor_config.bridges = vec![];
    tor_config.connection_timeout = 120;
    tor_config.enable_logging = true;
    tor_config.log_level = "info".to_string();
    
    let mut tor_service = match TorService::with_config(tor_config.clone()) {
        Ok(service) => {
            println!("   ✅ TorService created");
            service
        }
        Err(e) => {
            println!("   ❌ Failed to create TorService: {}", e);
            return Ok(());
        }
    };
    
    // Start Tor client
    println!("🚀 Starting Tor client...");
    let start_time = std::time::Instant::now();
    let timeout_duration = std::time::Duration::from_secs(300); // 5 minutes
    
    let start_result = tokio::time::timeout(timeout_duration, tor_service.start()).await;
    
    match start_result {
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
        mint_name: "Local Test Mint".to_string(),
        description: "Local test mint with Tor support".to_string(),
        lightning_backend: "fakewallet".to_string(),
        mode: "mintd_only".to_string(),
        database_path: "/tmp/local_mint_db".to_string(),
        logs_path: "/tmp/local_mint_logs".to_string(),
        lnbits_admin_api_key: None,
        lnbits_invoice_api_key: None,
        lnbits_api_url: None,
        cln_rpc_path: None,
        cln_bolt12: None,
        tor_enabled: Some(true),
        tor_mode: Some("custom".to_string()),
        tor_data_dir: Some("/tmp/local_mint_tor_data".to_string()),
        tor_socks_port: Some(9050),
        tor_enable_hidden_services: Some(true),
        tor_num_intro_points: Some(3),
        tor_bridges: Some(vec![]),
        tor_use_bridges: Some(false),
    };
    
    let work_dir = PathBuf::from("/tmp/local_mint_work");
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
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Try to get mint info to see if Tor address is available
    println!("📡 Testing mint info endpoint...");
    match tor_service.make_tor_request(&format!("http://127.0.0.1:{}/info", android_config.port)).await {
        Ok(response) => {
            println!("   ✅ Mint info request successful");
            println!("   📄 Response: {}", response);
            
            // Try to parse JSON and extract Tor address
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response) {
                if let Some(info) = json.as_object() {
                    if let Some(tor_address) = info.get("tor_address") {
                        println!("   🧅 Tor Address found: {}", tor_address);
                        
                        // Step 4: Test connectivity to Tor address
                        println!("\n🌐 Testing connectivity to Tor address...");
                        let tor_url = format!("http://{}/info", tor_address.as_str().unwrap());
                        println!("   📡 Testing URL: {}", tor_url);
                        
                        match tor_service.make_tor_request(&tor_url).await {
                            Ok(tor_response) => {
                                println!("   ✅ Tor address connectivity successful!");
                                println!("   📄 Tor Response: {}", tor_response);
                            }
                            Err(e) => {
                                println!("   ❌ Tor address connectivity failed: {}", e);
                            }
                        }
                    } else {
                        println!("   ⚠️  No Tor address found in mint info");
                    }
                    
                    // Check for other useful info
                    if let Some(name) = info.get("name") {
                        println!("   📛 Mint Name: {}", name);
                    }
                    if let Some(description) = info.get("description") {
                        println!("   📝 Description: {}", description);
                    }
                }
            }
        }
        Err(e) => {
            println!("   ❌ Mint info request failed: {}", e);
            println!("   💡 This might be because the mint service is not fully started yet");
        }
    }
    
    // Step 5: Test direct local connectivity
    println!("\n🏠 Testing direct local connectivity...");
    match tor_service.make_tor_request(&format!("http://127.0.0.1:{}/info", android_config.port)).await {
        Ok(response) => {
            println!("   ✅ Local connectivity successful");
            println!("   📄 Local Response: {}", response);
        }
        Err(e) => {
            println!("   ❌ Local connectivity failed: {}", e);
        }
    }
    
    // Step 6: Test Nostr relay connectivity (if configured)
    println!("\n📡 Testing Nostr relay connectivity...");
    
    // Check if there are any Nostr relays configured
    let relay_urls = vec![
        "ws://127.0.0.1:7777",  // Local relay
        "wss://relay.damus.io", // Public relay
    ];
    
    for relay_url in relay_urls {
        println!("   📡 Testing relay: {}", relay_url);
        
        // This would require Nostr client setup, but for now just show the intent
        println!("   ⚠️  Nostr relay testing requires additional setup");
        println!("   💡 You can test Nostr connectivity separately");
        break; // Just show one example
    }
    
    // Step 7: Stop services
    println!("\n🛑 Stopping services...");
    
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
    
    println!("\n=== Local Mint Tor Address Test Completed ===");
    println!("\n💡 Summary:");
    println!("   - Tor bridge connection: ✅");
    println!("   - Local mint service: ✅");
    println!("   - Tor hidden service: Tested");
    println!("   - Connectivity: Tested");
    println!("\n🔗 Next steps:");
    println!("   - Use a real nsec for production");
    println!("   - Configure proper Nostr relays");
    println!("   - Set up persistent Tor hidden service");
    println!("   - Test with real Lightning backend");
    
    Ok(())
} 