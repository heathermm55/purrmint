//! Core functionality for PurrMint
//! Internal module containing shared functions for Android integration

use std::sync::{Arc, Mutex, OnceLock};
use std::ffi::{CString, c_char};
use serde_json::json;
use tracing::{info, error};

use crate::config::AndroidConfig;
use crate::nostr::{nsec_to_npub as nostr_nsec_to_npub};
use crate::mintd_service::MintdService;
use crate::tor_service::TorService;

/// Global state for the mint service
static mut MINT_SERVICE: Option<Arc<Mutex<Option<MintdService>>>> = None;

/// Global state for the Tor service
static mut TOR_SERVICE: Option<Arc<Mutex<Option<TorService>>>> = None;

/// Global runtime for service management
static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

/// Initialize global state and runtime
fn init_globals() {
    unsafe {
        if MINT_SERVICE.is_none() {
            MINT_SERVICE = Some(Arc::new(Mutex::new(None)));
        }
        if TOR_SERVICE.is_none() {
            TOR_SERVICE = Some(Arc::new(Mutex::new(None)));
        }
    }
    
    // Initialize runtime if not already done
    RUNTIME.get_or_init(|| {
        tokio::runtime::Runtime::new().expect("Failed to create global runtime")
    });
}

// =============================================================================
// Basic functionality
// =============================================================================

/// Initialize logging for Android
pub fn init_logging() {
    #[cfg(target_os = "android")]
    {
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(log::LevelFilter::Debug)
                .with_tag("PurrMint")
        );
    }
    
    #[cfg(not(target_os = "android"))]
    {
        tracing_subscriber::fmt()
            .with_env_filter("purrmint=debug,tracing=debug")
            .with_target(false)
            .init();
    }
    
    info!("PurrMint logging initialized");
    info!("Log level set to debug");
    info!("Android logger configured for logcat output");
}

// =============================================================================
// Nostr account management
// =============================================================================

/// Convert nsec to npub (wrapper for nostr module function)
pub fn nsec_to_npub(nsec: &str) -> Result<String, String> {
    nostr_nsec_to_npub(nsec).map_err(|e| e.to_string())
}

// =============================================================================
// Configuration management
// =============================================================================

/// Load Android configuration from JSON file
pub fn load_android_config_from_file(file_path: &str) -> Result<String, String> {
    info!("Loading Android config from file: {}", file_path);
    
    if !std::path::Path::new(file_path).exists() {
        error!("Config file does not exist: {}", file_path);
        return Err(format!("Config file does not exist: {}", file_path));
    }
    
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;
    
    // Validate by parsing
    let config = AndroidConfig::from_json(&content)
        .map_err(|e| format!("Invalid config file format: {}", e))?;
    
    let json = config.to_json()
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    
    info!("Android config loaded successfully");
    Ok(json)
}

/// Save Android configuration to JSON file
pub fn save_android_config_to_file(file_path: &str, config_json: &str) -> Result<(), String> {
    info!("Saving Android config to file: {}", file_path);
    
    // Validate JSON by parsing it
    let config = AndroidConfig::from_json(config_json)
        .map_err(|e| format!("Invalid config JSON: {}", e))?;
    
    // Create parent directory if needed
    if let Some(parent) = std::path::Path::new(file_path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }
    
    let json = config.to_json()
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    
    std::fs::write(file_path, &json)
        .map_err(|e| format!("Failed to write config file: {}", e))?;
    
    info!("Android config saved successfully");
    Ok(())
}

/// Generate default Android configuration JSON
pub fn generate_default_android_config() -> Result<String, String> {
    let config = AndroidConfig::default();
    config.to_json().map_err(|e| format!("Failed to serialize default config: {}", e))
}

// =============================================================================
// Service management
// =============================================================================

/// Start Android service with configuration
pub fn start_android_service(config: &AndroidConfig, nsec: &str) -> Result<(), String> {
    info!("Starting Android service...");
    
    if nsec.is_empty() {
        return Err("nsec is empty".to_string());
    }
    
    info!("Service configuration: port={}, host={}", config.port, config.host);
    
    let config_path = std::path::Path::new(&config.database_path)
        .parent()
        .ok_or("Invalid database path")?
        .to_path_buf();
    
    // Create directory if needed
    std::fs::create_dir_all(&config_path)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;
    
    // Check if service is already running
    init_globals();
    unsafe {
        if let Some(service_guard) = MINT_SERVICE.as_ref() {
            if let Ok(guard) = service_guard.lock() {
                if let Some(service) = guard.as_ref() {
                    if service.is_running() {
                        info!("Service is already running");
                        return Ok(());
                    }
                }
            }
        }
    }
    
    // Start Tor service if enabled
    if config.tor_enabled.unwrap_or(false) {
        info!("Starting Tor service...");
        let tor_config = config.to_tor_config();
        let mut tor_service = TorService::with_config(tor_config)
            .map_err(|e| format!("Failed to create Tor service: {}", e))?;
        
        let rt = RUNTIME.get().unwrap();
        rt.block_on(async {
            match tor_service.start().await {
                Ok(()) => {
                    info!("Tor service started successfully");
                    
                    // Store Tor service in global state
                    unsafe {
                        if let Some(tor_service_guard) = TOR_SERVICE.as_ref() {
                            if let Ok(mut guard) = tor_service_guard.lock() {
                                *guard = Some(tor_service);
                            }
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to start Tor service: {}", e);
                    Err(format!("Failed to start Tor service: {}", e))
                }
            }
        })?;
        
        // Create hidden service if enabled
        if config.tor_enable_hidden_services.unwrap_or(false) {
            info!("Creating Tor hidden service...");
            let rt = RUNTIME.get().unwrap();
            rt.block_on(async {
                unsafe {
                    if let Some(tor_service_guard) = TOR_SERVICE.as_ref() {
                        if let Ok(guard) = tor_service_guard.lock() {
                            if let Some(tor_service) = guard.as_ref() {
                                // Use nsec as nickname for the hidden service
                                let nickname = format!("mint_{}", &nsec[..8]);
                                match tor_service.create_hidden_service(&nickname).await {
                                    Ok(info) => {
                                        info!("Hidden service created: {}", info.onion_address);
                                        Ok(())
                                    }
                                    Err(e) => {
                                        error!("Failed to create hidden service: {}", e);
                                        Err(format!("Failed to create hidden service: {}", e))
                                    }
                                }
                            } else {
                                Err("Tor service not available".to_string())
                            }
                        } else {
                            Err("Failed to lock Tor service".to_string())
                        }
                    } else {
                        Err("Tor service not initialized".to_string())
                    }
                }
            })?;
        }
    }
    
    // Create and start mint service using global runtime
    let mut mint_service = MintdService::new_with_android_config(config_path, config, nsec.to_string());
    
    let rt = RUNTIME.get().unwrap();
    rt.block_on(async move {
        match mint_service.start().await {
            Ok(()) => {
                info!("MintdService started successfully");
                
                // Store service in global state
                unsafe {
                    if let Some(service_guard) = MINT_SERVICE.as_ref() {
                        if let Ok(mut guard) = service_guard.lock() {
                            *guard = Some(mint_service);
                        }
                    }
                }
                Ok(())
            }
            Err(e) => {
                error!("Failed to start MintdService: {}", e);
                Err(format!("Failed to start MintdService: {}", e))
            }
        }
    })
}

/// Stop mint service
pub fn stop_service() -> Result<(), String> {
    info!("Stopping mint service...");
    init_globals();
    
    unsafe {
        if let Some(service_guard) = MINT_SERVICE.as_ref() {
            if let Ok(mut guard) = service_guard.lock() {
                if let Some(mut service) = guard.take() {
                    let rt = RUNTIME.get().unwrap();
                    
                    return rt.block_on(async move {
                        service.stop().await
                            .map_err(|e| format!("Failed to stop service: {}", e))
                    });
                }
            }
        }
    }
    
    info!("No running service found to stop");
    Ok(())
}

/// Get service status
pub fn get_service_status() -> String {
    init_globals();
    
    unsafe {
        if let Some(service_guard) = MINT_SERVICE.as_ref() {
            if let Ok(guard) = service_guard.lock() {
                if let Some(service) = guard.as_ref() {
                    return service.get_status().to_string();
                }
            }
        }
    }
    
    json!({
        "running": false,
        "details": "Service not initialized"
    }).to_string()
}

/// Get onion address if available
pub fn get_onion_address() -> Option<String> {
    init_globals();
    
    unsafe {
        if let Some(tor_service_guard) = TOR_SERVICE.as_ref() {
            if let Ok(guard) = tor_service_guard.lock() {
                if let Some(tor_service) = guard.as_ref() {
                    // Get the first hidden service's onion address
                    let rt = RUNTIME.get().unwrap();
                    return rt.block_on(async {
                        let services = tor_service.list_hidden_services().await;
                        match services {
                            Ok(services) => {
                                if let Some(first_service) = services.first() {
                                    Some(first_service.onion_address.clone())
                                } else {
                                    None
                                }
                            }
                            Err(_) => None
                        }
                    });
                }
            }
        }
    }
    
    None
}

/// Free string memory
pub fn free_string(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    
    unsafe {
        drop(CString::from_raw(s));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nostr::{Keys, ToBech32};
    use tempfile::tempdir;

    #[test]
    fn test_nsec_to_npub() {
        let keys = Keys::generate();
        let nsec = keys.secret_key().to_secret_hex();
        let expected_npub = keys.public_key().to_bech32().unwrap();
        
        let result = nsec_to_npub(&nsec);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_npub);
    }
    
    #[test]
    fn test_generate_default_android_config() {
        let result = generate_default_android_config();
        assert!(result.is_ok());
        
        let json = result.unwrap();
        assert!(json.contains("port"));
        assert!(json.contains("PurrMint"));
    }
    
    #[test]
    fn test_config_roundtrip() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let config_file = temp_dir.path().join("test_config.json");
        let config_file_path = config_file.to_str().unwrap();
        
        let default_config_json = generate_default_android_config().unwrap();
        
        // Save and load
        let save_result = save_android_config_to_file(config_file_path, &default_config_json);
        assert!(save_result.is_ok());
        
        let load_result = load_android_config_from_file(config_file_path);
        assert!(load_result.is_ok());
    }
} 