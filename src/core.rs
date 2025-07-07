//! Core functionality for PurrMint
//! Internal module containing shared functions for Android integration

use std::ffi::CString;
use std::os::raw::c_char;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;

use serde_json::json;
use tracing::info;

use crate::service::MintService;
use crate::config::{LightningConfig, ServiceMode};
use crate::nostr::{NostrAccount, create_nostr_account, free_nostr_account, nsec_to_npub as nostr_nsec_to_npub};

/// Service mode for Android
#[repr(C)]
#[derive(Debug)]
pub enum AndroidServiceMode {
    MintdOnly = 0,
    Nip74Only = 1,
    MintdAndNip74 = 2,
}



/// Global state for the mint service
static mut MINT_SERVICE: Option<Arc<Mutex<Option<bool>>>> = None;
static mut NOSTR_ACCOUNT: Option<Arc<Mutex<Option<NostrAccount>>>> = None;

/// Initialize global state
fn init_globals() {
    unsafe {
        if MINT_SERVICE.is_none() {
            MINT_SERVICE = Some(Arc::new(Mutex::new(Some(false))));
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
// Service management
// =============================================================================

/// Convert Android service mode to internal service mode
fn android_mode_to_service_mode(mode: AndroidServiceMode) -> ServiceMode {
    match mode {
        AndroidServiceMode::MintdOnly => ServiceMode::MintdOnly,
        AndroidServiceMode::Nip74Only => ServiceMode::Nip74Only,
        AndroidServiceMode::MintdAndNip74 => ServiceMode::MintdAndNip74,
    }
}

/// Start Android service
pub fn start_android_service(
    mode: AndroidServiceMode,
    config_dir: &str,
    mnemonic: &str,
    port: u16
) -> Result<(), String> {
    if config_dir.is_empty() || mnemonic.is_empty() {
        return Err("config_dir or mnemonic is empty".to_string());
    }
    
    let config_path = PathBuf::from(config_dir);
    
    // Create config directory if it doesn't exist
    if let Err(e) = std::fs::create_dir_all(&config_path) {
        return Err(format!("Failed to create config directory: {:?}", e));
    }
    
    // Generate Android configuration using the new config management
    let toml_content = crate::config::Settings::generate_android_config(&config_path, mnemonic, port)
        .map_err(|e| format!("Failed to generate Android config: {:?}", e))?;
    
    // Write config file
    let config_file = config_path.join("mintd.toml");
    std::fs::write(&config_file, toml_content)
        .map_err(|e| format!("Failed to write config file: {:?}", e))?;
    
    init_globals();
    
    // Mark service as started in global state
    unsafe {
        if let Some(service_guard) = MINT_SERVICE.as_ref() {
            if let Ok(mut guard) = service_guard.lock() {
                *guard = Some(true);
            }
        }
    }
    
    info!("Android service configuration completed successfully");
    Ok(())
}

/// Stop mint service
pub fn stop_service() -> Result<(), String> {
    init_globals();
    
    unsafe {
        if let Some(service_guard) = MINT_SERVICE.as_ref() {
            if let Ok(mut guard) = service_guard.lock() {
                if guard.is_some() {
                    *guard = Some(false);
                    info!("Service stopped successfully");
                    return Ok(());
                }
            }
        }
    }
    
    Err("No service to stop".to_string())
}

/// Get service status
pub fn get_service_status() -> String {
    init_globals();
    
    unsafe {
        if let Some(service_guard) = MINT_SERVICE.as_ref() {
            if let Ok(guard) = service_guard.lock() {
                if let Some(running) = *guard {
                    if running {
                        return json!({
                            "status": "running",
                            "message": "Service is running"
                        }).to_string();
                    }
                }
            }
        }
    }
    
    json!({
        "status": "stopped",
        "message": "Service is not running"
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
} 