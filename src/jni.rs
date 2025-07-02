//! JNI bindings for PurrMint library
//! Provides Java-compatible interface to the Rust FFI functions

use std::ffi::{CStr, CString};
use jni::JNIEnv;
use jni::objects::{JClass, JObject, JString, JValue};
use jni::sys::{jint, jstring};
use crate::ffi::NostrAccount;

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

#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_testFfi(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    let result = crate::ffi::mint_test_ffi();
    if result.is_null() {
        return std::ptr::null_mut();
    }
    let result_str = unsafe { CStr::from_ptr(result).to_str().unwrap_or("") };
    rust_string_to_java_string(&env, result_str.to_string())
}

#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_createAccount<'a>(
    mut env: JNIEnv<'a>,
    _class: JClass<'a>,
) -> JObject<'a> {
    let account = crate::ffi::nostr_create_account();
    rust_account_to_java_account(&mut env, account)
}

#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_getMintInfo(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    let info = crate::ffi::mint_get_info();
    if info.is_null() {
        return std::ptr::null_mut();
    }
    let info_str = unsafe { CStr::from_ptr(info).to_str().unwrap_or("") };
    rust_string_to_java_string(&env, info_str.to_string())
}

#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_getMintStatus(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    let status = crate::ffi::mint_get_status();
    if status.is_null() {
        return std::ptr::null_mut();
    }
    let status_str = unsafe { CStr::from_ptr(status).to_str().unwrap_or("") };
    rust_string_to_java_string(&env, status_str.to_string())
}

#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_getCurrentAccount(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    let account_info = crate::ffi::nostr_get_account();
    if account_info.is_null() {
        return std::ptr::null_mut();
    }
    let account_str = unsafe { CStr::from_ptr(account_info).to_str().unwrap_or("") };
    rust_string_to_java_string(&env, account_str.to_string())
}

#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_importAccount<'a>(
    mut env: JNIEnv<'a>,
    _class: JClass<'a>,
    secret_key: JString<'a>,
) -> JObject<'a> {
    let secret_key_str = java_string_to_rust_string(&mut env, secret_key);
    let secret_key_cstr = CString::new(secret_key_str).unwrap();
    let account = crate::ffi::nostr_import_account(secret_key_cstr.as_ptr());
    rust_account_to_java_account(&mut env, account)
}

#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_configureMint(
    mut env: JNIEnv,
    _class: JClass,
    config_json: JString,
) -> jint {
    let config_str = java_string_to_rust_string(&mut env, config_json);
    let config_cstr = CString::new(config_str).unwrap();
    let result = crate::ffi::mint_configure(config_cstr.as_ptr());
    result as jint
}

#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_startMint(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    let result = crate::ffi::mint_start();
    result as jint
}

#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_stopMint(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    let result = crate::ffi::mint_stop();
    result as jint
} 