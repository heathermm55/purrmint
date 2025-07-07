//! JNI bindings for PurrMint library
//! Provides Java-compatible interface organized into three main sections:
//! 1. Nostr methods - Account creation and key conversion
//! 2. Config methods - Configuration management
//! 3. Service methods - Service lifecycle and status

use std::ffi::{CStr, CString};
use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::{jint, jstring};
use crate::core::{AndroidServiceMode, free_string};
use crate::config::{AndroidConfig, Settings};
use std::ptr;
use tracing::error;

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
    crate::core::init_logging();
}

// =============================================================================
// Nostr methods - Account creation and key conversion
// =============================================================================

/// Create a new Nostr account and return nsec
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_PurrmintNative_createAccount(
    _env: JNIEnv,
    _class: JClass,
) -> jstring {
    match crate::nostr::create_account() {
        Ok(nsec) => {
            match _env.new_string(nsec) {
                Ok(java_string) => java_string.into_raw(),
                Err(e) => {
                    error!("Failed to create Java string: {:?}", e);
                    ptr::null_mut()
                }
            }
        },
        Err(e) => {
            error!("Failed to create Nostr account: {:?}", e);
            ptr::null_mut()
        }
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
    
    match crate::nostr::nsec_to_npub(&nsec_str) {
        Ok(npub) => {
            match _env.new_string(npub) {
                Ok(java_string) => java_string.into_raw(),
                Err(e) => {
                    error!("Failed to create Java string: {:?}", e);
                    ptr::null_mut()
                }
            }
        },
        Err(e) => {
            error!("Failed to convert nsec to npub: {:?}", e);
            ptr::null_mut()
        }
    }
}

// =============================================================================
// Config methods - Configuration management
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
            match _env.new_string(json_str) {
                Ok(java_string) => java_string.into_raw(),
                Err(e) => {
                    error!("Failed to create Java string for config: {:?}", e);
                    // Fallback to empty JSON object
                    match _env.new_string("{}") {
                        Ok(fallback) => fallback.into_raw(),
                        Err(_) => ptr::null_mut(),
                    }
                }
            }
        },
        Err(e) => {
            error!("Failed to serialize default config: {:?}", e);
            // Fallback to empty JSON object
            match _env.new_string("{}") {
                Ok(fallback) => fallback.into_raw(),
                Err(_) => ptr::null_mut(),
            }
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
    if let Err(e) = android_config.update_from_json(&config_str) {
        error!("Failed to update config from JSON: {:?}", e);
        // If update fails, try to parse as new config
        match AndroidConfig::from_json(&config_str) {
            Ok(new_config) => android_config = new_config,
            Err(parse_error) => {
                error!("Failed to parse config JSON: {:?}", parse_error);
                // If all parsing fails, return original string
                match _env.new_string(config_str) {
                    Ok(java_string) => return java_string.into_raw(),
                    Err(_) => return ptr::null_mut(),
                }
            }
        }
    }
    
    // Return updated config as JSON
    match android_config.to_json() {
        Ok(json_str) => {
            match _env.new_string(json_str) {
                Ok(java_string) => java_string.into_raw(),
                Err(e) => {
                    error!("Failed to create Java string for updated config: {:?}", e);
                    // Fallback to original string
                    match _env.new_string(config_str) {
                        Ok(fallback) => fallback.into_raw(),
                        Err(_) => ptr::null_mut(),
                    }
                }
            }
        },
        Err(e) => {
            error!("Failed to serialize updated config: {:?}", e);
            // Fallback to original string
            match _env.new_string(config_str) {
                Ok(fallback) => fallback.into_raw(),
                Err(_) => ptr::null_mut(),
            }
        }
    }
}

// =============================================================================
// Service methods - Service lifecycle and status
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
    let _nsec_str = java_string_to_rust_string(&mut _env, nsec);
    
    // Parse Android configuration
    let android_config = match AndroidConfig::from_json(&config_str) {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to parse Android config JSON: {:?}", e);
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
                error!("Failed to create config directory: {:?}", e);
                return 2;
            }
            
            if let Err(e) = std::fs::write(&config_file, toml_content) {
                error!("Failed to write config file: {:?}", e);
                return 3;
            }
        },
        Err(e) => {
            error!("Failed to generate Android config: {:?}", e);
            return 4;
        }
    }
    
    // Parse mode
    let android_mode = match mode_str.as_str() {
        "MintdOnly" => AndroidServiceMode::MintdOnly,
        "Nip74Only" => AndroidServiceMode::Nip74Only,
        "MintdAndNip74" => AndroidServiceMode::MintdAndNip74,
        _ => AndroidServiceMode::MintdOnly,
    };
    
    let config_dir_str = data_dir.to_string_lossy();
    
    match crate::core::start_android_service(
        android_mode,
        &config_dir_str,
        mnemonic,
        port,
    ) {
        Ok(()) => 0,
        Err(e) => {
            error!("Failed to start Android service: {}", e);
            5
        }
    }
}

/// Stop the mint service
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_PurrmintNative_stopMint(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    match crate::core::stop_service() {
        Ok(()) => 0,
        Err(e) => {
            error!("Failed to stop service: {}", e);
            1
        }
    }
}

/// Get mint status
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_PurrmintNative_getMintStatus(
    _env: JNIEnv,
    _class: JClass,
) -> jstring {
    let status_str = crate::core::get_service_status();
    
    match _env.new_string(status_str) {
        Ok(java_string) => java_string.into_raw(),
        Err(e) => {
            error!("Failed to create Java string for status: {:?}", e);
            ptr::null_mut()
        }
    }
} 