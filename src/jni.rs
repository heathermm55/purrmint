//! JNI bindings for PurrMint library
//! Provides Java-compatible interface to the Rust FFI functions

use std::ffi::{CStr, CString};
use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::{jint, jstring, jboolean};
use crate::ffi::{NostrAccount, FfiServiceMode, mint_free_string};
use std::ptr;
use serde_json;
use tracing::info;





/// Convert Java string to Rust string
fn java_string_to_rust_string(env: &mut JNIEnv, java_string: JString) -> String {
    env.get_string(&java_string).unwrap().into()
}



/// Initialize logging for Android
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_PurrmintNative_initLogging(
    _env: JNIEnv,
    _class: JClass,
) {
    crate::ffi::mint_init_logging();
}

/// Test the JNI interface
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_PurrmintNative_testFfi(
    _env: JNIEnv,
    _class: JClass,
) -> jstring {
    let result = crate::ffi::mint_test_ffi();
    if result.is_null() {
        return ptr::null_mut();
    }
    
    let result_str = unsafe { CStr::from_ptr(result) }.to_str().unwrap_or("{}");
    let java_string = _env.new_string(result_str).unwrap();
    let java_string_ptr = java_string.into_raw();
    
    unsafe {
        mint_free_string(result);
    }
    
    java_string_ptr
}

/// Create a new Nostr account
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_PurrmintNative_createAccount(
    _env: JNIEnv,
    _class: JClass,
) -> jstring {
    let account = crate::ffi::nostr_create_account();
    if account.is_null() {
        return ptr::null_mut();
    }
    
    // Convert account to JSON string
    unsafe {
        let account_ptr = account as *const NostrAccount;
        let account_ref = &*account_ptr;
        
        let pubkey_str = CStr::from_ptr(account_ref.pubkey).to_str().unwrap_or("");
        let secret_str = CStr::from_ptr(account_ref.secret_key).to_str().unwrap_or("");
        
        let account_json = serde_json::json!({
            "pubkey": pubkey_str,
            "secretKey": secret_str,
            "isImported": account_ref.is_imported
        });
        
        let json_str = serde_json::to_string(&account_json).unwrap();
        let java_string = _env.new_string(json_str).unwrap();
        let java_string_ptr = java_string.into_raw();
        
        // Free the account
        crate::ffi::nostr_free_account(account);
        
        java_string_ptr
    }
}

/// Import an existing Nostr account
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_PurrmintNative_importAccount(
    mut _env: JNIEnv,
    _class: JClass,
    secret_key: JString,
) -> jstring {
    let secret_key_str = java_string_to_rust_string(&mut _env, secret_key);
    let secret_key_cstr = CString::new(secret_key_str).unwrap();
    
    let account = crate::ffi::nostr_import_account(secret_key_cstr.as_ptr());
    if account.is_null() {
        return ptr::null_mut();
    }
    
    // Convert account to JSON string
    unsafe {
        let account_ptr = account as *const NostrAccount;
        let account_ref = &*account_ptr;
        
        let pubkey_str = CStr::from_ptr(account_ref.pubkey).to_str().unwrap_or("");
        let secret_str = CStr::from_ptr(account_ref.secret_key).to_str().unwrap_or("");
        
        let account_json = serde_json::json!({
            "pubkey": pubkey_str,
            "secretKey": secret_str,
            "isImported": account_ref.is_imported
        });
        
        let json_str = serde_json::to_string(&account_json).unwrap();
        let java_string = _env.new_string(json_str).unwrap();
        let java_string_ptr = java_string.into_raw();
        
        // Free the account
        crate::ffi::nostr_free_account(account);
        
        java_string_ptr
    }
}

/// Configure the mint service
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_PurrmintNative_configureMint(
    mut _env: JNIEnv,
    _class: JClass,
    config_json: JString,
) -> jint {
    let config_str = java_string_to_rust_string(&mut _env, config_json);
    let config_cstr = CString::new(config_str).unwrap();
    
    let result = crate::ffi::mint_configure(config_cstr.as_ptr());
    result as jint
}

/// Start the mint service with specified mode
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_PurrmintNative_startMintWithMode(
    mut _env: JNIEnv,
    _class: JClass,
    mode: jint,
    config_dir: JString,
    port: jint,
) -> jint {
    let config_dir_str = java_string_to_rust_string(&mut _env, config_dir);
    let config_dir_cstr = CString::new(config_dir_str).unwrap();
    
    let ffi_mode = match mode {
        0 => FfiServiceMode::MintdOnly,
        1 => FfiServiceMode::Nip74Only,
        2 => FfiServiceMode::MintdAndNip74,
        _ => FfiServiceMode::MintdOnly, // Default to mintd only
    };
    
    let result = crate::ffi::mint_start_with_mode(ffi_mode, config_dir_cstr.as_ptr(), port as u16);
    result as jint
}

/// Start the mint service (legacy - uses mintd only mode)
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_PurrmintNative_startMint(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    info!("JNI startMint (no params): LEGACY FUNCTION - this should not be used!");
    info!("JNI startMint (no params): Android should call startMint with parameters instead");
    let result = crate::ffi::mint_start();
    result as jint
}

/// Start the mint service with parameters (Android version)
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_PurrmintNative_startMint__Ljava_lang_String_2Ljava_lang_String_2I(
    mut _env: JNIEnv,
    _class: JClass,
    config_dir: JString,
    mnemonic: JString,
    port: jint,
) -> jint {
    let config_dir_str = java_string_to_rust_string(&mut _env, config_dir);
    let mnemonic_str = java_string_to_rust_string(&mut _env, mnemonic);
    

    
    let config_dir_cstr = CString::new(config_dir_str).unwrap();
    let mnemonic_cstr = CString::new(mnemonic_str).unwrap();
    
    let result = crate::ffi::mint_start_android(
        crate::ffi::FfiServiceMode::MintdOnly,
        config_dir_cstr.as_ptr(),
        mnemonic_cstr.as_ptr(),
        port as u16,
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
    
        mint_free_string(status);
    
    java_string_ptr
}

/// Get current Nostr account information
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_PurrmintNative_getCurrentAccount(
    _env: JNIEnv,
    _class: JClass,
) -> jstring {
    let account = crate::ffi::nostr_get_account();
    if account.is_null() {
        return ptr::null_mut();
    }
    
    let account_str = unsafe { CStr::from_ptr(account) }.to_str().unwrap_or("{}");
    let java_string = _env.new_string(account_str).unwrap();
    let java_string_ptr = java_string.into_raw();
    
        mint_free_string(account);
    
    java_string_ptr
}

/// Get service access URLs
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_PurrmintNative_getAccessUrls(
    _env: JNIEnv,
    _class: JClass,
) -> jstring {
    let urls = crate::ffi::mint_get_access_urls();
    if urls.is_null() {
        return ptr::null_mut();
    }
    
    let urls_str = unsafe { CStr::from_ptr(urls) }.to_str().unwrap_or("{}");
    let java_string = _env.new_string(urls_str).unwrap();
    let java_string_ptr = java_string.into_raw();
    
        mint_free_string(urls);
    
    java_string_ptr
}

/// Start mintd service (legacy function)
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_PurrmintNative_startMintd(
    mut _env: JNIEnv,
    _class: JClass,
    config_dir: JString,
    port: jint,
) -> jint {
    let config_dir_str = java_string_to_rust_string(&mut _env, config_dir);
    let config_dir_cstr = CString::new(config_dir_str).unwrap();
    
    let result = crate::ffi::mint_start_mintd(config_dir_cstr.as_ptr(), port as u16);
    result as jint
}

/// Stop mintd service (legacy function)
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_PurrmintNative_stopMintd(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    let result = crate::ffi::mint_stop_mintd();
    result as jint
}

/// Check if mintd is running
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_PurrmintNative_isMintdRunning(
    _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    let running = crate::ffi::mint_is_mintd_running();
    running as jboolean
}

/// Start mint service with Android-optimized configuration
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_PurrmintNative_startMintAndroid(
    mut _env: JNIEnv,
    _class: JClass,
    mode: jint,
    config_dir: JString,
    mnemonic: JString,
    port: jint,
) -> jint {
    let config_dir_str = java_string_to_rust_string(&mut _env, config_dir);
    let mnemonic_str = java_string_to_rust_string(&mut _env, mnemonic);
    
    let config_dir_cstr = CString::new(config_dir_str).unwrap();
    let mnemonic_cstr = CString::new(mnemonic_str).unwrap();
    
    let ffi_mode = match mode {
        0 => crate::ffi::FfiServiceMode::MintdOnly,
        1 => crate::ffi::FfiServiceMode::Nip74Only,
        2 => crate::ffi::FfiServiceMode::MintdAndNip74,
        _ => crate::ffi::FfiServiceMode::MintdOnly,
    };
    
    let result = crate::ffi::mint_start_android(
        ffi_mode,
        config_dir_cstr.as_ptr(),
        mnemonic_cstr.as_ptr(),
        port as u16,
    );
    
    result as jint
}

/// Generate configuration file (alias for generateAndroidConfig)
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_PurrmintNative_generateConfig(
    mut _env: JNIEnv,
    _class: JClass,
    config_dir: JString,
    mnemonic: JString,
    port: jint,
) -> jint {
    let config_dir_str = java_string_to_rust_string(&mut _env, config_dir);
    let mnemonic_str = java_string_to_rust_string(&mut _env, mnemonic);
    
    let config_dir_cstr = CString::new(config_dir_str).unwrap();
    let mnemonic_cstr = CString::new(mnemonic_str).unwrap();
    
    let result = crate::ffi::mint_generate_android_config(
        config_dir_cstr.as_ptr(),
        mnemonic_cstr.as_ptr(),
        port as u16,
    );
    
    result as jint
}

/// Generate Android mintd configuration
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_PurrmintNative_generateAndroidConfig(
    mut _env: JNIEnv,
    _class: JClass,
    config_dir: JString,
    mnemonic: JString,
    port: jint,
) -> jint {
    let config_dir_str = java_string_to_rust_string(&mut _env, config_dir);
    let mnemonic_str = java_string_to_rust_string(&mut _env, mnemonic);
    
    let config_dir_cstr = CString::new(config_dir_str).unwrap();
    let mnemonic_cstr = CString::new(mnemonic_str).unwrap();
    
    let result = crate::ffi::mint_generate_android_config(
        config_dir_cstr.as_ptr(),
        mnemonic_cstr.as_ptr(),
        port as u16,
    );
    
    result as jint
}

/// Get Android app data directory path
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_PurrmintNative_getAndroidDataDir(
    _env: JNIEnv,
    _class: JClass,
) -> jstring {
    // This should be called from Android context to get the actual data directory
    // For now, return a placeholder that Android should replace
    let placeholder = "/data/data/com.example.purrmint/files";
    _env.new_string(placeholder).unwrap().into_raw()
} 