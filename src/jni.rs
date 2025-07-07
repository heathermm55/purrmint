//! JNI bindings for PurrMint library
//! Provides Java-compatible interface to the Rust FFI functions

use std::ffi::{CStr, CString};
use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::{jint, jstring};
use crate::ffi::{NostrAccount, FfiServiceMode, mint_free_string};
use crate::config::{AndroidConfig, Settings};
use std::ptr;
use serde_json;
use tracing::info;
use nostr::{Keys, ToBech32};
use std::str::FromStr;

/// Convert Java string to Rust string
fn java_string_to_rust_string(env: &mut JNIEnv, java_string: JString) -> String {
    env.get_string(&java_string).unwrap().into()
}

// =============================================================================
// Basic functionality interfaces
// =============================================================================

/// Initialize logging for Android
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_PurrmintNative_initLogging(
    _env: JNIEnv,
    _class: JClass,
) {
    crate::ffi::mint_init_logging();
}

// =============================================================================
// Nostr account management interfaces
// =============================================================================

/// Create a new Nostr account and return nsec
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_PurrmintNative_createAccount(
    _env: JNIEnv,
    _class: JClass,
) -> jstring {
    let account = crate::ffi::nostr_create_account();
    if account.is_null() {
        return ptr::null_mut();
    }
    
    // Convert account to nsec format
    unsafe {
        let account_ptr = account as *const NostrAccount;
        let account_ref = &*account_ptr;
        
        let secret_str = CStr::from_ptr(account_ref.secret_key).to_str().unwrap_or("");
        
        // Parse the secret key to convert to bech32 format
        let nsec = match Keys::from_str(secret_str) {
            Ok(keys) => keys.secret_key().to_bech32().unwrap_or_else(|_| secret_str.to_string()),
            Err(_) => secret_str.to_string(),
        };
        
        let java_string = _env.new_string(nsec).unwrap();
        let java_string_ptr = java_string.into_raw();
        
        // Free the account
        crate::ffi::nostr_free_account(account);
        
        java_string_ptr
    }
}

/// Convert nsec to npub
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_PurrmintNative_nsecToNpub(
    mut _env: JNIEnv,
    _class: JClass,
    nsec: JString,
) -> jstring {
    let nsec_str = java_string_to_rust_string(&mut _env, nsec);
    
    match Keys::from_str(&nsec_str) {
        Ok(keys) => {
            match keys.public_key().to_bech32() {
                Ok(npub) => {
                    let java_string = _env.new_string(npub).unwrap();
                    java_string.into_raw()
                },
                Err(_) => ptr::null_mut(),
            }
        },
        Err(_) => ptr::null_mut(),
    }
}

// =============================================================================
// Configuration interfaces
// =============================================================================

/// Load configuration, return default Android config object
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_PurrmintNative_loadConfig(
    _env: JNIEnv,
    _class: JClass,
) -> jstring {
    // Use the new AndroidConfig from config.rs
    let default_config = AndroidConfig::default();
    
    match default_config.to_json() {
        Ok(json_str) => {
            let java_string = _env.new_string(json_str).unwrap();
            java_string.into_raw()
        },
        Err(_) => {
            // Fallback to empty JSON object
            let java_string = _env.new_string("{}").unwrap();
            java_string.into_raw()
        }
    }
}

/// Update configuration and return updated config object
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_PurrmintNative_updateConfig(
    mut _env: JNIEnv,
    _class: JClass,
    config_json: JString,
) -> jstring {
    let config_str = java_string_to_rust_string(&mut _env, config_json);
    
    // Start with default config
    let mut android_config = AndroidConfig::default();
    
    // Update with provided JSON
    if let Err(_) = android_config.update_from_json(&config_str) {
        // If update fails, try to parse as new config
        match AndroidConfig::from_json(&config_str) {
            Ok(new_config) => android_config = new_config,
            Err(_) => {
                // If all parsing fails, return original string
                let java_string = _env.new_string(config_str).unwrap();
                return java_string.into_raw();
            }
        }
    }
    
    // Return updated config as JSON
    match android_config.to_json() {
        Ok(json_str) => {
            let java_string = _env.new_string(json_str).unwrap();
            java_string.into_raw()
        },
        Err(_) => {
            // Fallback to original string
            let java_string = _env.new_string(config_str).unwrap();
            java_string.into_raw()
        }
    }
}

// =============================================================================
// Service start/stop interfaces
// =============================================================================

/// Start mint service with configuration and nsec
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_PurrmintNative_startMintWithConfig(
    mut _env: JNIEnv,
    _class: JClass,
    config_json: JString,
    nsec: JString,
) -> jint {
    let config_str = java_string_to_rust_string(&mut _env, config_json);
    let nsec_str = java_string_to_rust_string(&mut _env, nsec);
    
    // Parse Android configuration
    let android_config = match AndroidConfig::from_json(&config_str) {
        Ok(config) => config,
        Err(_) => {
            tracing::error!("Failed to parse Android config JSON");
            return 1; // Invalid configuration
        }
    };
    
    // Extract configuration parameters
    let port = android_config.port;
    let mode_str = &android_config.mode;
    let data_dir = std::path::Path::new(&android_config.database_path)
        .parent()
        .unwrap_or_else(|| std::path::Path::new("/data/data/com.purrmint.app/files"));
    
    // Convert nsec to mnemonic or derive mnemonic from nsec
    // For now, we'll use a default mnemonic but this should be derived from nsec
    let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    
    // Generate TOML configuration using the new config management
    match Settings::generate_android_config(data_dir, mnemonic, port) {
        Ok(toml_content) => {
            // Write TOML config file
            let config_file = data_dir.join("mintd.toml");
            if let Err(e) = std::fs::create_dir_all(data_dir) {
                tracing::error!("Failed to create config directory: {:?}", e);
                return 2;
            }
            
            if let Err(e) = std::fs::write(&config_file, toml_content) {
                tracing::error!("Failed to write config file: {:?}", e);
                return 3;
            }
        },
        Err(e) => {
            tracing::error!("Failed to generate Android config: {:?}", e);
            return 4;
        }
    }
    
    // Parse mode
    let ffi_mode = match mode_str.as_str() {
        "MintdOnly" => FfiServiceMode::MintdOnly,
        "Nip74Only" => FfiServiceMode::Nip74Only,
        "MintdAndNip74" => FfiServiceMode::MintdAndNip74,
        _ => FfiServiceMode::MintdOnly,
    };
    
    let config_dir_cstr = CString::new(data_dir.to_string_lossy().as_ref()).unwrap();
    let mnemonic_cstr = CString::new(mnemonic).unwrap();
    
    let result = crate::ffi::mint_start_android(
        ffi_mode,
        config_dir_cstr.as_ptr(),
        mnemonic_cstr.as_ptr(),
        port,
    );
    
    result as jint
}

/// Stop the mint service
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_PurrmintNative_stopMint(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    let result = crate::ffi::mint_stop();
    result as jint
}

// =============================================================================
// Status query interfaces
// =============================================================================

/// Get mint status
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_PurrmintNative_getMintStatus(
    _env: JNIEnv,
    _class: JClass,
) -> jstring {
    let status = crate::ffi::mint_get_status();
    if status.is_null() {
        return ptr::null_mut();
    }
    
    let status_str = unsafe { CStr::from_ptr(status) }.to_str().unwrap_or("{}");
    let java_string = _env.new_string(status_str).unwrap();
    let java_string_ptr = java_string.into_raw();
    
    unsafe {
        mint_free_string(status);
    }
    
    java_string_ptr
} 