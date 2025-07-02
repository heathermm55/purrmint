#include <jni.h>
#include <string>
#include "purrmint.h"

extern "C" {

// Test FFI interface
JNIEXPORT jstring JNICALL
Java_com_example_purrmint_PurrmintNative_testFfi(JNIEnv *env, jclass clazz) {
    char *result = mint_test_ffi();
    if (result) {
        jstring jresult = env->NewStringUTF(result);
        mint_free_string(result);
        return jresult;
    }
    return env->NewStringUTF("{\"error\": \"FFI test failed\"}");
}

// Create Nostr account
JNIEXPORT jobject JNICALL
Java_com_example_purrmint_PurrmintNative_createAccount(JNIEnv *env, jclass clazz) {
    NostrAccount *account = nostr_create_account();
    if (!account) {
        return nullptr;
    }
    
    // Create Java NostrAccount object
    jclass accountClass = env->FindClass("com/example/purrmint/NostrAccount");
    if (!accountClass) {
        nostr_free_account(account);
        return nullptr;
    }
    
    jmethodID constructor = env->GetMethodID(accountClass, "<init>", "()V");
    if (!constructor) {
        nostr_free_account(account);
        return nullptr;
    }
    
    jobject javaAccount = env->NewObject(accountClass, constructor);
    if (!javaAccount) {
        nostr_free_account(account);
        return nullptr;
    }
    
    // Set fields
    jfieldID pubkeyField = env->GetFieldID(accountClass, "pubkey", "Ljava/lang/String;");
    jfieldID secretKeyField = env->GetFieldID(accountClass, "secretKey", "Ljava/lang/String;");
    jfieldID isImportedField = env->GetFieldID(accountClass, "isImported", "Z");
    
    if (pubkeyField && secretKeyField && isImportedField) {
        jstring pubkey = env->NewStringUTF(account->pubkey);
        jstring secretKey = env->NewStringUTF(account->secret_key);
        
        env->SetObjectField(javaAccount, pubkeyField, pubkey);
        env->SetObjectField(javaAccount, secretKeyField, secretKey);
        env->SetBooleanField(javaAccount, isImportedField, account->is_imported);
    }
    
    nostr_free_account(account);
    return javaAccount;
}

// Get mint info
JNIEXPORT jstring JNICALL
Java_com_example_purrmint_PurrmintNative_getMintInfo(JNIEnv *env, jclass clazz) {
    char *info = mint_get_info();
    if (info) {
        jstring jinfo = env->NewStringUTF(info);
        mint_free_string(info);
        return jinfo;
    }
    return env->NewStringUTF("{\"error\": \"Failed to get mint info\"}");
}

// Get mint status
JNIEXPORT jstring JNICALL
Java_com_example_purrmint_PurrmintNative_getMintStatus(JNIEnv *env, jclass clazz) {
    char *status = mint_get_status();
    if (status) {
        jstring jstatus = env->NewStringUTF(status);
        mint_free_string(status);
        return jstatus;
    }
    return env->NewStringUTF("{\"error\": \"Failed to get mint status\"}");
}

// Configure mint
JNIEXPORT jint JNICALL
Java_com_example_purrmint_PurrmintNative_configureMint(JNIEnv *env, jclass clazz, jstring config_json) {
    if (!config_json) {
        return static_cast<jint>(FfiError::NullPointer);
    }
    
    const char *config_str = env->GetStringUTFChars(config_json, nullptr);
    if (!config_str) {
        return static_cast<jint>(FfiError::NullPointer);
    }
    
    FfiError result = mint_configure(config_str);
    env->ReleaseStringUTFChars(config_json, config_str);
    
    return static_cast<jint>(result);
}

// Start mint service
JNIEXPORT jint JNICALL
Java_com_example_purrmint_PurrmintNative_startMint(JNIEnv *env, jclass clazz) {
    FfiError result = mint_start();
    return static_cast<jint>(result);
}

// Stop mint service
JNIEXPORT jint JNICALL
Java_com_example_purrmint_PurrmintNative_stopMint(JNIEnv *env, jclass clazz) {
    FfiError result = mint_stop();
    return static_cast<jint>(result);
}

} // extern "C" 