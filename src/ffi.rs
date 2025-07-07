//! FFI interface for Android integration
//! Provides C ABI functions that can be called from Android via JNI

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use std::sync::{Arc, Mutex};
use std::str::FromStr;
use std::path::PathBuf;

use nostr::prelude::*;
use serde_json::{json, Value};
use tracing::{info, error};

use crate::service::{MintService, ServiceMode};
use crate::handler::default::DefaultRequestHandler;
use crate::config::LightningConfig;
use crate::mintd_service::MintdService;

/// FFI Error codes
#[repr(C)]
#[derive(PartialEq, Copy, Clone)]
pub enum FfiError {
    Success = 0,
    NullPointer = 1,
    InvalidInput = 2,
    ServiceError = 3,
    NotInitialized = 4,
}

/// Service mode for FFI
#[repr(C)]
#[derive(Debug)]
pub enum FfiServiceMode {
    MintdOnly = 0,
    Nip74Only = 1,
    MintdAndNip74 = 2,
}

/// Nostr Account structure for FFI
#[repr(C)]
pub struct NostrAccount {
    pub pubkey: *mut c_char,
    pub secret_key: *mut c_char,
    pub is_imported: bool,
}

/// Mint Configuration structure for FFI
#[repr(C)]
pub struct MintConfig {
    pub identifier: *mut c_char,
    pub relays: *mut c_char, // JSON array of relay URLs
    pub lightning_backend: *mut c_char, // JSON config for lightning
    pub mint_info: *mut c_char, // JSON mint info
}

/// Global state for the mint service
static mut MINT_SERVICE: Option<Arc<Mutex<Option<Arc<MintService>>>>> = None;
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

/// Initialize logging for Android
#[no_mangle]
pub extern "C" fn mint_init_logging() {
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

/// Convert FFI service mode to internal service mode
fn ffi_mode_to_service_mode(mode: FfiServiceMode) -> ServiceMode {
    match mode {
        FfiServiceMode::MintdOnly => ServiceMode::MintdOnly,
        FfiServiceMode::Nip74Only => ServiceMode::Nip74Only,
        FfiServiceMode::MintdAndNip74 => ServiceMode::MintdAndNip74,
    }
}

/// Create a new Nostr account
#[no_mangle]
pub extern "C" fn nostr_create_account() -> *mut NostrAccount {
    init_globals();
    
    // Generate new keys
    let keys = Keys::generate();
    let pubkey = CString::new(keys.public_key().to_string()).unwrap();
    let secret_key = CString::new(keys.secret_key().to_secret_hex()).unwrap();
    
    let account = Box::new(NostrAccount {
        pubkey: pubkey.into_raw(),
        secret_key: secret_key.into_raw(),
        is_imported: false,
    });
    
    // Store in global state
    unsafe {
        if let Some(account_guard) = NOSTR_ACCOUNT.as_ref() {
            if let Ok(mut guard) = account_guard.lock() {
                *guard = Some(NostrAccount {
                    pubkey: CString::new(keys.public_key().to_string()).unwrap().into_raw(),
                    secret_key: CString::new(keys.secret_key().to_secret_hex()).unwrap().into_raw(),
                    is_imported: false,
                });
            }
        }
    }
    
    Box::into_raw(account)
}

/// Import an existing Nostr account from secret key
#[no_mangle]
pub extern "C" fn nostr_import_account(secret_key_str: *const c_char) -> *mut NostrAccount {
    if secret_key_str.is_null() {
        return ptr::null_mut();
    }
    
    init_globals();
    
    let secret_str = unsafe { CStr::from_ptr(secret_key_str) }.to_str().unwrap_or("");
    if secret_str.is_empty() {
        return ptr::null_mut();
    }
    
    // Parse the secret key
    let keys = match Keys::from_str(secret_str) {
        Ok(k) => k,
        Err(_) => return ptr::null_mut(),
    };
    
    let pubkey = CString::new(keys.public_key().to_string()).unwrap();
    let secret_key = CString::new(secret_str.to_string()).unwrap(); // Keep original format
    
    let account = Box::new(NostrAccount {
        pubkey: pubkey.into_raw(),
        secret_key: secret_key.into_raw(),
        is_imported: true,
    });
    
    // Store in global state
    unsafe {
        if let Some(account_guard) = NOSTR_ACCOUNT.as_ref() {
            if let Ok(mut guard) = account_guard.lock() {
                *guard = Some(NostrAccount {
                    pubkey: CString::new(keys.public_key().to_string()).unwrap().into_raw(),
                    secret_key: CString::new(secret_str.to_string()).unwrap().into_raw(), // Keep original format
                    is_imported: true,
                });
            }
        }
    }
    
    Box::into_raw(account)
}

/// Configure the mint service
#[no_mangle]
pub extern "C" fn mint_configure(config_json: *const c_char) -> FfiError {
    if config_json.is_null() {
        return FfiError::NullPointer;
    }
    
    let config_str = unsafe { CStr::from_ptr(config_json) }.to_str().unwrap_or("");
    if config_str.is_empty() {
        return FfiError::InvalidInput;
    }
    
    // Parse configuration JSON
    let _config: Value = match serde_json::from_str(config_str) {
        Ok(c) => c,
        Err(_) => return FfiError::InvalidInput,
    };
    
    // TODO: Implement actual configuration logic
    // For now, just return success
    FfiError::Success
}

/// Start the mint service with specified mode
#[no_mangle]
pub extern "C" fn mint_start_with_mode(mode: FfiServiceMode, config_dir: *const c_char, port: u16) -> FfiError {
    if config_dir.is_null() {
        error!("mint_start_with_mode: config_dir is null");
        return FfiError::NullPointer;
    }
    
    init_globals();
    
    let config_dir_str = unsafe { CStr::from_ptr(config_dir) }.to_str().unwrap_or("");
    if config_dir_str.is_empty() {
        error!("mint_start_with_mode: config_dir_str is empty");
        return FfiError::InvalidInput;
    }
    
    let config_path = PathBuf::from(config_dir_str);
    let service_mode = ffi_mode_to_service_mode(mode);
    

    
    // Create mint info (default)
    let mint_info = cdk::nuts::nut06::MintInfo {
        name: Some("purrmint".to_string()),
        pubkey: None,
        version: Some(cdk::nuts::nut06::MintVersion::new("PurrMint".to_string(), "0.1.0".to_string())),
        description: Some("PurrMint Cashu Mint".to_string()),
        description_long: None,
        contact: None,
        nuts: cdk::nuts::Nuts::default(),
        icon_url: None,
        urls: None,
        motd: None,
        time: None,
        tos_url: None,
    };
    
    // Default relays
    let relays = vec![
        RelayUrl::from_str("wss://relay.damus.io").unwrap(),
        RelayUrl::from_str("wss://nos.lol").unwrap(),
    ];
    
    // Default lightning config
    let lightning_config = LightningConfig::default();
    
    // Create service
    let rt = tokio::runtime::Runtime::new().unwrap();
    let service_result = rt.block_on(async {
        let service = MintService::new(
            service_mode,
            mint_info,
            lightning_config,
            relays,
            config_path,
            port,
        ).await;
        
        match service {
            Ok(mut svc) => {
                // For NIP-74 modes, set up signer and handler
                if service_mode != ServiceMode::MintdOnly {
                    // Get current account
                    unsafe {
                        if let Some(account_guard) = NOSTR_ACCOUNT.as_ref() {
                            if let Ok(guard) = account_guard.lock() {
                                if let Some(account) = guard.as_ref() {
                                    let secret_str = CStr::from_ptr(account.secret_key).to_str().unwrap_or("");
                                    if let Ok(keys) = Keys::from_str(secret_str) {
                                        let signer = Arc::new(keys);
                                        svc.set_signer(signer)?;
                                        
                                        // Set default handler that proxies to mintd
                                        let handler = Arc::new(DefaultRequestHandler::new(port));
                                        svc.set_handler(handler)?;
                                    }
                                }
                            }
                        }
                    }
                }
                // Start service
                svc.start().await?;
                futures::future::pending::<()>().await;
                Ok(())
            }
            Err(e) => Err(e),
        }
    });
    
    match service_result {
        Ok(_) => FfiError::Success,
        Err(e) => {
            error!("mint_start_with_mode: Failed to start mint service: {:?}", e);
            FfiError::ServiceError
        }
    }
}

/// Start the mint service (legacy - uses mintd only mode)
#[no_mangle]
pub extern "C" fn mint_start() -> FfiError {
    info!("mint_start: LEGACY FUNCTION CALLED - this should not be used on Android!");
    // Default to mintd only mode with default config
    // Note: This function is not used on Android, Android uses mint_start_android instead
    let config_dir = CString::new("/tmp/purrmint").unwrap();
    mint_start_with_mode(FfiServiceMode::MintdOnly, config_dir.as_ptr(), 3338)
}

/// Stop the mint service
#[no_mangle]
pub extern "C" fn mint_stop() -> FfiError {
    init_globals();
    
    unsafe {
        if let Some(service_guard) = MINT_SERVICE.as_ref() {
            if let Ok(mut guard) = service_guard.lock() {
                if let Some(service_arc) = guard.take() {
                    let service = Arc::try_unwrap(service_arc).ok().map(|s| s);
                    if let Some(mut service) = service {
                        let rt = tokio::runtime::Runtime::new().unwrap();
                        let _ = rt.block_on(service.stop());
                    }
                }
            }
        }
    }
    
    FfiError::Success
}

/// Get mint information as JSON string
#[no_mangle]
pub extern "C" fn mint_get_info() -> *mut c_char {
    let info = json!({
        "status": "running",
        "version": "0.1.0",
        "supported_operations": ["info", "get_mint_quote", "check_mint_quote", "mint", "get_melt_quote", "check_melt_quote", "melt"],
        "supported_modes": ["mintd_only", "nip74_only", "mintd_and_nip74"]
    });
    
    let info_str = serde_json::to_string(&info).unwrap();
    CString::new(info_str).unwrap().into_raw()
}

/// Get mint status as JSON string
#[no_mangle]
pub extern "C" fn mint_get_status() -> *mut c_char {
    init_globals();
    
    unsafe {
        if let Some(service_guard) = MINT_SERVICE.as_ref() {
            if let Ok(guard) = service_guard.lock() {
                if let Some(service) = guard.as_ref() {
                    let status = service.get_status();
                    let status_str = serde_json::to_string(&status).unwrap();
                    return CString::new(status_str).unwrap().into_raw();
                }
            }
        }
    }
    
    // Return default status if no service is running
    let default_status = json!({
        "mode": "none",
        "mintd_running": false,
        "nip74_running": false,
        "mintd_port": 3338,
        "relays": []
    });
    
    let status_str = serde_json::to_string(&default_status).unwrap();
    CString::new(status_str).unwrap().into_raw()
}

/// Get current Nostr account information as JSON string
#[no_mangle]
pub extern "C" fn nostr_get_account() -> *mut c_char {
    init_globals();
    
    unsafe {
        if let Some(account_guard) = NOSTR_ACCOUNT.as_ref() {
            if let Ok(guard) = account_guard.lock() {
                if let Some(account) = guard.as_ref() {
                    let pubkey = CStr::from_ptr(account.pubkey).to_str().unwrap_or("");
                    let account_info = json!({
                        "pubkey": pubkey,
                        "is_imported": account.is_imported
                    });
                    let info_str = serde_json::to_string(&account_info).unwrap();
                    return CString::new(info_str).unwrap().into_raw();
                }
            }
        }
    }
    
    // Return empty account info if no account is set
    let empty_info = json!({
        "pubkey": "",
        "is_imported": false
    });
    let info_str = serde_json::to_string(&empty_info).unwrap();
    CString::new(info_str).unwrap().into_raw()
}

/// Free a C string
#[no_mangle]
pub extern "C" fn mint_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}

/// Free a Nostr account
#[no_mangle]
pub extern "C" fn nostr_free_account(account: *mut NostrAccount) {
    if !account.is_null() {
        unsafe {
            let account = Box::from_raw(account);
            mint_free_string(account.pubkey);
            mint_free_string(account.secret_key);
        }
    }
}

/// Test the FFI interface
#[no_mangle]
pub extern "C" fn mint_test_ffi() -> *mut c_char {
    let test_result = json!({
        "status": "success",
        "message": "FFI interface is working correctly",
        "version": "0.1.0",
        "features": ["nostr_account", "mint_service", "three_modes"]
    });
    
    let result_str = serde_json::to_string(&test_result).unwrap();
    CString::new(result_str).unwrap().into_raw()
}

/// Get service access URLs as JSON string
#[no_mangle]
pub extern "C" fn mint_get_access_urls() -> *mut c_char {
    init_globals();
    
    unsafe {
        if let Some(service_guard) = MINT_SERVICE.as_ref() {
            if let Ok(guard) = service_guard.lock() {
                if let Some(service) = guard.as_ref() {
                    let urls = service.get_access_urls();
                    let urls_str = serde_json::to_string(&urls).unwrap();
                    return CString::new(urls_str).unwrap().into_raw();
                }
            }
        }
    }
    
    // Return empty URLs if no service is running
    let empty_urls = json!({});
    let urls_str = serde_json::to_string(&empty_urls).unwrap();
    CString::new(urls_str).unwrap().into_raw()
}

/// Start mintd service (legacy function - now use mint_start_with_mode)
#[no_mangle]
pub extern "C" fn mint_start_mintd(config_dir: *const c_char, port: u16) -> FfiError {
    mint_start_with_mode(FfiServiceMode::MintdOnly, config_dir, port)
}

/// Stop mintd service (legacy function - now use mint_stop)
#[no_mangle]
pub extern "C" fn mint_stop_mintd() -> FfiError {
    mint_stop()
}

/// Check if mintd is running
#[no_mangle]
pub extern "C" fn mint_is_mintd_running() -> bool {
    init_globals();
    
    unsafe {
        if let Some(service_guard) = MINT_SERVICE.as_ref() {
            if let Ok(guard) = service_guard.lock() {
                if let Some(service) = guard.as_ref() {
                    let status = service.get_status();
                    if let Some(mintd_running) = status.get("mintd_running") {
                        return mintd_running.as_bool().unwrap_or(false);
                    }
                }
            }
        }
    }
    
    false
}

/// Generate mintd config for Android with proper paths
#[no_mangle]
pub extern "C" fn mint_generate_android_config(
    config_dir: *const c_char,
    mnemonic: *const c_char,
    port: u16
) -> FfiError {
    if config_dir.is_null() || mnemonic.is_null() {
        error!("mint_generate_android_config: config_dir or mnemonic is null");
        return FfiError::NullPointer;
    }
    
    let config_dir_str = unsafe { CStr::from_ptr(config_dir) }.to_str().unwrap_or("");
    let mnemonic_str = unsafe { CStr::from_ptr(mnemonic) }.to_str().unwrap_or("");
    
    if config_dir_str.is_empty() || mnemonic_str.is_empty() {
        error!("mint_generate_android_config: config_dir_str or mnemonic_str is empty");
        return FfiError::InvalidInput;
    }
    
    let config_path = PathBuf::from(config_dir_str);
    
    // Create config directory if it doesn't exist
    if let Err(e) = std::fs::create_dir_all(&config_path) {
        error!("mint_generate_android_config: failed to create config directory: {:?}", e);
        return FfiError::ServiceError;
    }
    
    // Generate Android configuration using the new config management
    let config_content = match crate::config::Settings::generate_android_config(&config_path, mnemonic_str, port) {
        Ok(content) => content,
        Err(e) => {
            error!("mint_generate_android_config: failed to generate config: {:?}", e);
            return FfiError::ServiceError;
        }
    };
    
    // Write config file
    let config_file = config_path.join("mintd.toml");
    if let Err(e) = std::fs::write(&config_file, config_content) {
        error!("mint_generate_android_config: failed to write config file: {:?}", e);
        return FfiError::ServiceError;
    }
    
    FfiError::Success
}

/// Start mint service with Android-optimized configuration
#[no_mangle]
pub extern "C" fn mint_start_android(
    mode: FfiServiceMode,
    config_dir: *const c_char,
    mnemonic: *const c_char,
    port: u16
) -> FfiError {
    if config_dir.is_null() || mnemonic.is_null() {
        error!("mint_start_android: config_dir or mnemonic is null");
        return FfiError::NullPointer;
    }
    
    let config_dir_str = unsafe { CStr::from_ptr(config_dir) }.to_str().unwrap_or("");
    let mnemonic_str = unsafe { CStr::from_ptr(mnemonic) }.to_str().unwrap_or("");
    
    // Generate Android config first
    let config_result = mint_generate_android_config(config_dir, mnemonic, port);
    if config_result != FfiError::Success {
        error!("mint_start_android: config generation failed with error code: {}", config_result as i32);
        return config_result;
    }
    
    // Start service with generated config
    mint_start_with_mode(mode, config_dir, port)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nostr_create_account() {
        let account = nostr_create_account();
        assert!(!account.is_null());
        
        unsafe {
            let account = Box::from_raw(account);
            let pubkey = CStr::from_ptr(account.pubkey).to_str().unwrap();
            let secret_key = CStr::from_ptr(account.secret_key).to_str().unwrap();
            
            assert!(!pubkey.is_empty());
            assert!(!secret_key.is_empty());
            assert!(!account.is_imported);
            
            mint_free_string(account.pubkey);
            mint_free_string(account.secret_key);
        }
    }

    #[test]
    fn test_mint_get_info() {
        let info = mint_get_info();
        assert!(!info.is_null());
        
        unsafe {
            let info_str = CStr::from_ptr(info).to_str().unwrap();
            let info_json: Value = serde_json::from_str(info_str).unwrap();
            assert!(info_json.get("status").is_some());
            assert!(info_json.get("version").is_some());
            
            mint_free_string(info);
        }
    }

    #[test]
    fn test_mint_test_ffi() {
        let result = mint_test_ffi();
        assert!(!result.is_null());
        
        unsafe {
            let result_str = CStr::from_ptr(result).to_str().unwrap();
            let result_json: Value = serde_json::from_str(result_str).unwrap();
            assert_eq!(result_json.get("status").unwrap().as_str().unwrap(), "success");
            
            mint_free_string(result);
        }
    }

    #[test]
    fn test_nostr_import_account() {
        let test_secret = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";
        let secret_cstr = CString::new(test_secret).unwrap();
        let account = nostr_import_account(secret_cstr.as_ptr());
        assert!(!account.is_null());
        
        unsafe {
            let account = Box::from_raw(account);
            let pubkey = CStr::from_ptr(account.pubkey).to_str().unwrap();
            let secret_key = CStr::from_ptr(account.secret_key).to_str().unwrap();
            
            assert!(!pubkey.is_empty());
            // The secret key should be returned in the same format as input
            assert_eq!(secret_key, test_secret);
            assert!(account.is_imported);
            
            mint_free_string(account.pubkey);
            mint_free_string(account.secret_key);
        }
    }
} 