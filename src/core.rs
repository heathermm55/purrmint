//! Core functionality for PurrMint
//! Internal module containing shared functions for Android integration

use std::sync::{Arc, Mutex};
use std::ffi::{CString, c_char};
use serde_json::json;
use tracing::{info, error};

use crate::config::AndroidConfig;
use crate::nostr::{NostrAccount, nsec_to_npub as nostr_nsec_to_npub};
use crate::mintd_service::MintdService;

/// Global state for the mint service
static mut MINT_SERVICE: Option<Arc<Mutex<Option<MintdService>>>> = None;
static mut NOSTR_ACCOUNT: Option<Arc<Mutex<Option<NostrAccount>>>> = None;

/// Initialize global state
fn init_globals() {
    unsafe {
        if MINT_SERVICE.is_none() {
            MINT_SERVICE = Some(Arc::new(Mutex::new(None)));
        }
        if NOSTR_ACCOUNT.is_none() {
            NOSTR_ACCOUNT = Some(Arc::new(Mutex::new(None)));
        }
    }
}

// =============================================================================
// Basic functionality
// =============================================================================

/// Initialize logging for Android
pub fn init_logging() {
    // Initialize Android logger for logcat output
    #[cfg(target_os = "android")]
    {
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(log::LevelFilter::Debug)
                .with_tag("PurrMint")
        );
    }
    
    // Also initialize tracing subscriber for non-Android platforms
    #[cfg(not(target_os = "android"))]
    {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "purrmint=debug,tracing=debug".into()),
            )
            .with_target(false)
            .with_thread_ids(false)
            .with_thread_names(false)
            .with_file(false)
            .with_line_number(false)
            .init();
    }
    
    info!("PurrMint logging initialized");
    info!("Log level set to debug");
    info!("Android logger configured for logcat output");
}

// =============================================================================
// Nostr account management (using nostr module)
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
    
    // Read file content
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read config file '{}': {}", file_path, e))?;
    
    // Validate JSON by parsing it
    let config = AndroidConfig::from_json(&content)
        .map_err(|e| format!("Invalid config file format: {}", e))?;
    
    // Return the validated JSON
    let json = config.to_json()
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    
    info!("Android config loaded successfully from: {}", file_path);
    Ok(json)
}

/// Save Android configuration to JSON file
pub fn save_android_config_to_file(file_path: &str, config_json: &str) -> Result<(), String> {
    info!("Saving Android config to file: {}", file_path);
    
    // Validate JSON by parsing it
    let config = AndroidConfig::from_json(config_json)
        .map_err(|e| format!("Invalid config JSON: {}", e))?;
    
    // Create parent directory if it doesn't exist
    if let Some(parent) = std::path::Path::new(file_path).parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return Err(format!("Failed to create config directory: {}", e));
        }
    }
    
    // Convert back to pretty JSON and write to file
    let pretty_json = config.to_json()
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    
    std::fs::write(file_path, pretty_json)
        .map_err(|e| format!("Failed to write config file '{}': {}", file_path, e))?;
    
    info!("Android config saved successfully to: {}", file_path);
    Ok(())
}

/// Generate default Android configuration JSON
pub fn generate_default_android_config() -> Result<String, String> {
    info!("Generating default Android config");
    
    let config = AndroidConfig::default();
    let json = config.to_json()
        .map_err(|e| format!("Failed to serialize default config: {}", e))?;
    
    info!("Default Android config generated successfully");
    Ok(json)
}

// =============================================================================
// Service management
// =============================================================================

/// Start Android service with configuration
pub fn start_android_service(config: &AndroidConfig, nsec: &str) -> Result<(), String> {
    if nsec.is_empty() {
        return Err("nsec is empty".to_string());
    }
    
    // Validate database path - don't give default, fail if invalid
    let config_path = std::path::Path::new(&config.database_path)
        .parent()
        .ok_or_else(|| "Invalid database path: no parent directory".to_string())?
        .to_path_buf();
    
    // Create config directory if it doesn't exist
    if let Err(e) = std::fs::create_dir_all(&config_path) {
        return Err(format!("Failed to create config directory: {:?}", e));
    }
    
    info!("Starting Android service with config: port={}, mode={}", config.port, config.mode);
    
    init_globals();
    
    // Create MintdService with nsec
    let mut mint_service = MintdService::new_with_nsec(config_path, nsec.to_string());
    
    // Start the service asynchronously
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("Failed to create Tokio runtime: {:?}", e))?;
    
    rt.block_on(async move {
        match mint_service.start().await {
            Ok(()) => {
                info!("MintdService started successfully");
                
                // Store the running service in global state
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
                error!("Failed to start MintdService: {:?}", e);
                Err(format!("Failed to start MintdService: {:?}", e))
            }
        }
    })
}

/// Stop mint service
pub fn stop_service() -> Result<(), String> {
    init_globals();
    
    unsafe {
        if let Some(service_guard) = MINT_SERVICE.as_ref() {
            if let Ok(mut guard) = service_guard.lock() {
                if let Some(mut service) = guard.take() {
                    let rt = tokio::runtime::Runtime::new()
                        .map_err(|e| format!("Failed to create Tokio runtime: {:?}", e))?;
                    
                    rt.block_on(async move {
                        match service.stop().await {
                            Ok(()) => {
                                info!("Service stopped successfully");
                                Ok(())
                            }
                            Err(e) => {
                                error!("Failed to stop service: {:?}", e);
                                Err(format!("Failed to stop service: {:?}", e))
                            }
                        }
                    })
                } else {
                    Err("No service to stop".to_string())
                }
            } else {
                Err("Failed to lock service guard".to_string())
            }
        } else {
            Err("Service not initialized".to_string())
        }
    }
}

/// Get service status
pub fn get_service_status() -> String {
    unsafe {
        if let Some(service_guard) = MINT_SERVICE.as_ref() {
            if let Ok(guard) = service_guard.lock() {
                if let Some(service) = guard.as_ref() {
                    let status = service.get_status();
                    let server_url = format!("http://{}:{}", 
                        service.config.info.listen_host, 
                        service.config.info.listen_port);
                    
                    return json!({
                        "running": service.is_running(),
                        "server_url": server_url,
                        "work_dir": service.work_dir.to_string_lossy(),
                        "status": status
                    }).to_string();
                }
            }
        }
    }
    
    json!({
        "running": false,
        "server_url": "",
        "work_dir": "",
        "status": null
    }).to_string()
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
    fn test_invalid_nsec() {
        let result = nsec_to_npub("invalid_nsec");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_empty_nsec() {
        let result = nsec_to_npub("");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_generate_default_android_config() {
        let result = generate_default_android_config();
        assert!(result.is_ok());
        
        let json = result.unwrap();
        assert!(json.contains("port"));
        assert!(json.contains("host"));
        assert!(json.contains("mint_name"));
        assert!(json.contains("PurrMint"));
    }
    
    #[test]
    fn test_save_and_load_android_config() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let config_file = temp_dir.path().join("test_config.json");
        let config_file_path = config_file.to_str().unwrap();
        
        // Generate default config
        let default_config_json = generate_default_android_config().unwrap();
        
        // Save config to file
        let save_result = save_android_config_to_file(config_file_path, &default_config_json);
        assert!(save_result.is_ok());
        
        // Load config from file
        let load_result = load_android_config_from_file(config_file_path);
        assert!(load_result.is_ok());
        
        let loaded_config_json = load_result.unwrap();
        
        // Parse both configs to compare
        let default_config = AndroidConfig::from_json(&default_config_json).unwrap();
        let loaded_config = AndroidConfig::from_json(&loaded_config_json).unwrap();
        
        assert_eq!(default_config.port, loaded_config.port);
        assert_eq!(default_config.host, loaded_config.host);
        assert_eq!(default_config.mint_name, loaded_config.mint_name);
        assert_eq!(default_config.mode, loaded_config.mode);
    }
    
    #[test]
    fn test_load_nonexistent_config_file() {
        let result = load_android_config_from_file("/nonexistent/path/config.json");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to read config file"));
    }
    
    #[test]
    fn test_save_config_with_invalid_json() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let config_file = temp_dir.path().join("invalid_config.json");
        let config_file_path = config_file.to_str().unwrap();
        
        let invalid_json = "{invalid json}";
        let result = save_android_config_to_file(config_file_path, invalid_json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid config JSON"));
    }
    
    #[test]
    fn test_save_config_creates_directory() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let nested_path = temp_dir.path().join("nested").join("directory").join("config.json");
        let config_file_path = nested_path.to_str().unwrap();
        
        // Generate default config
        let default_config_json = generate_default_android_config().unwrap();
        
        // Save config to nested path (should create directories)
        let save_result = save_android_config_to_file(config_file_path, &default_config_json);
        assert!(save_result.is_ok());
        
        // Verify file was created
        assert!(nested_path.exists());
        
        // Verify we can load it back
        let load_result = load_android_config_from_file(config_file_path);
        assert!(load_result.is_ok());
    }
} 