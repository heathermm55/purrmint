//! JNI bindings for PurrMint library
//! Provides Java-compatible interface to the Rust FFI functions

use std::ffi::{CStr, CString};
use jni::JNIEnv;
use jni::objects::{JClass, JObject, JString, JValue};
use jni::sys::{jint, jstring};
use crate::ffi::NostrAccount;
use std::os::raw::c_char;
use std::ptr;

/// Service mode enum for JNI
#[repr(C)]
pub enum JniServiceMode {
    MintdOnly = 0,
    Nip74Only = 1,
    MintdAndNip74 = 2,
}

/// Convert JNI service mode to FFI service mode
fn jni_mode_to_ffi_mode(mode: JniServiceMode) -> FfiServiceMode {
    match mode {
        JniServiceMode::MintdOnly => FfiServiceMode::MintdOnly,
        JniServiceMode::Nip74Only => FfiServiceMode::Nip74Only,
        JniServiceMode::MintdAndNip74 => FfiServiceMode::MintdAndNip74,
    }
}

/// Convert Rust string to Java string
fn rust_string_to_java_string<'a>(env: &JNIEnv<'a>, rust_string: String) -> jstring {
    env.new_string(rust_string).unwrap().into_raw()
}

/// Convert Java string to Rust string
fn java_string_to_rust_string(env: &mut JNIEnv, java_string: JString) -> String {
    env.get_string(&java_string).unwrap().into()
}

/// Convert Rust NostrAccount to Java NostrAccount object
fn rust_account_to_java_account<'a>(env: &mut JNIEnv<'a>, account: *mut NostrAccount) -> JObject<'a> {
    if account.is_null() {
        return JObject::null();
    }
    unsafe {
        let acc = &*account;
        let pubkey = CStr::from_ptr(acc.pubkey).to_str().unwrap_or("");
        let secret_key = CStr::from_ptr(acc.secret_key).to_str().unwrap_or("");
        let class = env.find_class("com/example/purrmint/NostrAccount").unwrap();
        let obj = env.alloc_object(&class).unwrap();
        // pubkey
        let pubkey_field = env.get_field_id(&class, "pubkey", "Ljava/lang/String;").unwrap();
        let pubkey_string = env.new_string(pubkey).unwrap();
        env.set_field_unchecked(&obj, pubkey_field, JValue::Object(&JObject::from(pubkey_string))).unwrap();
        // secretKey
        let secret_field = env.get_field_id(&class, "secretKey", "Ljava/lang/String;").unwrap();
        let secret_string = env.new_string(secret_key).unwrap();
        env.set_field_unchecked(&obj, secret_field, JValue::Object(&JObject::from(secret_string))).unwrap();
        // isImported
        let imported_field = env.get_field_id(&class, "isImported", "Z").unwrap();
        env.set_field_unchecked(&obj, imported_field, JValue::Bool(if acc.is_imported { 1 } else { 0 })).unwrap();
        obj
    }
}

/// Test the JNI interface
#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_testFfi(
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
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_createAccount(
    _env: JNIEnv,
    _class: JClass,
) -> JObject {
    let account = crate::ffi::nostr_create_account();
    if account.is_null() {
        return ptr::null_mut();
    }
    
    // Create NostrAccount object
    let account_class = _env.find_class("com/example/purrmint/NostrAccount").unwrap();
    let constructor = _env.get_method_id(account_class, "<init>", "()V").unwrap();
    
    let account_obj = _env.new_object(account_class, constructor, &[]).unwrap();
    
    // Set fields
    unsafe {
        let account_ptr = account as *const NostrAccount;
        let account_ref = &*account_ptr;
        
        // Set pubkey
        let pubkey_str = CStr::from_ptr(account_ref.pubkey).to_str().unwrap_or("");
        let pubkey_field = _env.get_field_id(account_class, "pubkey", "Ljava/lang/String;").unwrap();
        let pubkey_java = _env.new_string(pubkey_str).unwrap();
        _env.set_field(account_obj, pubkey_field, "Ljava/lang/String;", pubkey_java.into()).unwrap();
        
        // Set secret_key
        let secret_str = CStr::from_ptr(account_ref.secret_key).to_str().unwrap_or("");
        let secret_field = _env.get_field_id(account_class, "secretKey", "Ljava/lang/String;").unwrap();
        let secret_java = _env.new_string(secret_str).unwrap();
        _env.set_field(account_obj, secret_field, "Ljava/lang/String;", secret_java.into()).unwrap();
        
        // Set is_imported
        let imported_field = _env.get_field_id(account_class, "isImported", "Z").unwrap();
        _env.set_field(account_obj, imported_field, "Z", account_ref.is_imported.into()).unwrap();
    }
    
    account_obj.into_raw()
}

/// Import an existing Nostr account
#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_importAccount(
    _env: JNIEnv,
    _class: JClass,
    secret_key: JString,
) -> JObject {
    let secret_key_str = java_string_to_rust_string(&mut _env, secret_key);
    let secret_key_cstr = CString::new(secret_key_str).unwrap();
    
    let account = crate::ffi::nostr_import_account(secret_key_cstr.as_ptr());
    if account.is_null() {
        return ptr::null_mut();
    }
    
    // Create NostrAccount object (same as createAccount)
    let account_class = _env.find_class("com/example/purrmint/NostrAccount").unwrap();
    let constructor = _env.get_method_id(account_class, "<init>", "()V").unwrap();
    
    let account_obj = _env.new_object(account_class, constructor, &[]).unwrap();
    
    // Set fields
    unsafe {
        let account_ptr = account as *const NostrAccount;
        let account_ref = &*account_ptr;
        
        // Set pubkey
        let pubkey_str = CStr::from_ptr(account_ref.pubkey).to_str().unwrap_or("");
        let pubkey_field = _env.get_field_id(account_class, "pubkey", "Ljava/lang/String;").unwrap();
        let pubkey_java = _env.new_string(pubkey_str).unwrap();
        _env.set_field(account_obj, pubkey_field, "Ljava/lang/String;", pubkey_java.into()).unwrap();
        
        // Set secret_key
        let secret_str = CStr::from_ptr(account_ref.secret_key).to_str().unwrap_or("");
        let secret_field = _env.get_field_id(account_class, "secretKey", "Ljava/lang/String;").unwrap();
        let secret_java = _env.new_string(secret_str).unwrap();
        _env.set_field(account_obj, secret_field, "Ljava/lang/String;", secret_java.into()).unwrap();
        
        // Set is_imported
        let imported_field = _env.get_field_id(account_class, "isImported", "Z").unwrap();
        _env.set_field(account_obj, imported_field, "Z", account_ref.is_imported.into()).unwrap();
    }
    
    account_obj.into_raw()
}

/// Configure the mint service
#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_configureMint(
    _env: JNIEnv,
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
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_startMintWithMode(
    _env: JNIEnv,
    _class: JClass,
    mode: jint,
    config_dir: JString,
    port: jint,
) -> jint {
    let config_dir_str = java_string_to_rust_string(&mut _env, config_dir);
    let config_dir_cstr = CString::new(config_dir_str).unwrap();
    
    let ffi_mode = jni_mode_to_ffi_mode(match mode {
        0 => JniServiceMode::MintdOnly,
        1 => JniServiceMode::Nip74Only,
        2 => JniServiceMode::MintdAndNip74,
        _ => JniServiceMode::MintdOnly, // Default to mintd only
    });
    
    let result = crate::ffi::mint_start_with_mode(ffi_mode, config_dir_cstr.as_ptr(), port as u16);
    result as jint
}

/// Start the mint service (legacy - uses mintd only mode)
#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_startMint(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    let result = crate::ffi::mint_start();
    result as jint
}

/// Stop the mint service
#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_stopMint(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    let result = crate::ffi::mint_stop();
    result as jint
}

/// Get mint information
#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_getMintInfo(
    _env: JNIEnv,
    _class: JClass,
) -> jstring {
    let info = crate::ffi::mint_get_info();
    if info.is_null() {
        return ptr::null_mut();
    }
    
    let info_str = unsafe { CStr::from_ptr(info) }.to_str().unwrap_or("{}");
    let java_string = _env.new_string(info_str).unwrap();
    let java_string_ptr = java_string.into_raw();
    
    unsafe {
        mint_free_string(info);
    }
    
    java_string_ptr
}

/// Get mint status
#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_getMintStatus(
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

/// Get current Nostr account information
#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_getCurrentAccount(
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
    
    unsafe {
        mint_free_string(account);
    }
    
    java_string_ptr
}

/// Get service access URLs
#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_getAccessUrls(
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
    
    unsafe {
        mint_free_string(urls);
    }
    
    java_string_ptr
}

/// Start mintd service (legacy function)
#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_startMintd(
    _env: JNIEnv,
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
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_stopMintd(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    let result = crate::ffi::mint_stop_mintd();
    result as jint
}

/// Check if mintd is running
#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_isMintdRunning(
    _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    let running = crate::ffi::mint_is_mintd_running();
    running as jboolean
} 