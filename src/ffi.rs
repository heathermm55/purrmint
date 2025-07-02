//! FFI interface for Android integration
//! Provides C ABI functions that can be called from Android via JNI

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use std::sync::{Arc, Mutex};
use std::str::FromStr;

use nostr::prelude::*;
use serde_json::{json, Value};

use crate::MintService;

/// FFI Error codes
#[repr(C)]
pub enum FfiError {
    Success = 0,
    NullPointer = 1,
    InvalidInput = 2,
    ServiceError = 3,
    NotInitialized = 4,
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
        if let Some(account_guard) = &NOSTR_ACCOUNT {
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
    let secret_key = CString::new(keys.secret_key().to_secret_hex()).unwrap();
    
    let account = Box::new(NostrAccount {
        pubkey: pubkey.into_raw(),
        secret_key: secret_key.into_raw(),
        is_imported: true,
    });
    
    // Store in global state
    unsafe {
        if let Some(account_guard) = &NOSTR_ACCOUNT {
            if let Ok(mut guard) = account_guard.lock() {
                *guard = Some(NostrAccount {
                    pubkey: CString::new(keys.public_key().to_string()).unwrap().into_raw(),
                    secret_key: CString::new(keys.secret_key().to_secret_hex()).unwrap().into_raw(),
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
    let config: Value = match serde_json::from_str(config_str) {
        Ok(c) => c,
        Err(_) => return FfiError::InvalidInput,
    };
    
    // TODO: Implement actual configuration logic
    // For now, just return success
    FfiError::Success
}

/// Start the mint service
#[no_mangle]
pub extern "C" fn mint_start() -> FfiError {
    init_globals();
    
    // Check if account is available
    unsafe {
        if let Some(account_guard) = &NOSTR_ACCOUNT {
            if let Ok(guard) = account_guard.lock() {
                if guard.is_none() {
                    return FfiError::NotInitialized;
                }
            }
        }
    }
    
    // TODO: Implement actual mint service startup
    // For now, just return success
    FfiError::Success
}

/// Stop the mint service
#[no_mangle]
pub extern "C" fn mint_stop() -> FfiError {
    init_globals();
    
    // TODO: Implement actual mint service shutdown
    // For now, just return success
    FfiError::Success
}

/// Get mint information as JSON string
#[no_mangle]
pub extern "C" fn mint_get_info() -> *mut c_char {
    let info = json!({
        "status": "running",
        "version": "0.1.0",
        "supported_operations": ["info", "get_mint_quote", "check_mint_quote", "mint", "get_melt_quote", "check_melt_quote", "melt"]
    });
    
    let info_str = serde_json::to_string(&info).unwrap();
    CString::new(info_str).unwrap().into_raw()
}

/// Get mint status as JSON string
#[no_mangle]
pub extern "C" fn mint_get_status() -> *mut c_char {
    let status = json!({
        "running": true,
        "uptime": 0,
        "total_requests": 0,
        "active_quotes": 0
    });
    
    let status_str = serde_json::to_string(&status).unwrap();
    CString::new(status_str).unwrap().into_raw()
}

/// Get current Nostr account information
#[no_mangle]
pub extern "C" fn nostr_get_account() -> *mut c_char {
    unsafe {
        if let Some(account_guard) = &NOSTR_ACCOUNT {
            if let Ok(guard) = account_guard.lock() {
                if let Some(account) = &*guard {
                    let account_info = json!({
                        "pubkey": CStr::from_ptr(account.pubkey).to_str().unwrap(),
                        "is_imported": account.is_imported
                    });
                    let info_str = serde_json::to_string(&account_info).unwrap();
                    return CString::new(info_str).unwrap().into_raw();
                }
            }
        }
    }
    
    // Return empty object if no account
    let empty = json!({});
    let empty_str = serde_json::to_string(&empty).unwrap();
    CString::new(empty_str).unwrap().into_raw()
}

/// Free a C string allocated by Rust
#[no_mangle]
pub extern "C" fn mint_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe { CString::from_raw(s) };
    }
}

/// Free a NostrAccount structure
#[no_mangle]
pub extern "C" fn nostr_free_account(account: *mut NostrAccount) {
    if !account.is_null() {
        unsafe {
            let acc = Box::from_raw(account);
            mint_free_string(acc.pubkey);
            mint_free_string(acc.secret_key);
        }
    }
}

/// Test function to verify FFI is working
#[no_mangle]
pub extern "C" fn mint_test_ffi() -> *mut c_char {
    let test_result = json!({
        "status": "success",
        "message": "FFI interface is working correctly",
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });
    
    let result_str = serde_json::to_string(&test_result).unwrap();
    CString::new(result_str).unwrap().into_raw()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nostr_create_account() {
        let account = nostr_create_account();
        assert!(!account.is_null());
        
        unsafe {
            let acc = &*account;
            let pubkey = CStr::from_ptr(acc.pubkey).to_str().unwrap();
            let secret = CStr::from_ptr(acc.secret_key).to_str().unwrap();
            
            assert!(!pubkey.is_empty());
            assert!(!secret.is_empty());
            assert!(!acc.is_imported);
            
            nostr_free_account(account);
        }
    }

    #[test]
    fn test_mint_get_info() {
        let info = mint_get_info();
        assert!(!info.is_null());
        
        let info_str = unsafe { CStr::from_ptr(info).to_str().unwrap() };
        let info_json: Value = serde_json::from_str(info_str).unwrap();
        
        assert_eq!(info_json["status"], "running");
        assert_eq!(info_json["version"], "0.1.0");
        
        mint_free_string(info);
    }

    #[test]
    fn test_mint_test_ffi() {
        let result = mint_test_ffi();
        assert!(!result.is_null());
        
        let result_str = unsafe { CStr::from_ptr(result).to_str().unwrap() };
        let result_json: Value = serde_json::from_str(result_str).unwrap();
        
        assert_eq!(result_json["status"], "success");
        
        mint_free_string(result);
    }

    #[test]
    fn test_nostr_import_account() {
        // Test with invalid input
        let account = nostr_import_account(ptr::null());
        assert!(account.is_null());
        
        // Test with empty string
        let empty = CString::new("").unwrap();
        let account = nostr_import_account(empty.as_ptr());
        assert!(account.is_null());
    }
} 