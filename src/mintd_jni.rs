use std::sync::Arc;
use std::sync::Mutex;
use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jint, jlong, jstring};

use anyhow::Result;
use tracing::{info, error};
use tokio::runtime::Runtime;

use crate::mintd_service::MintdService;

// Global mintd service instance
static MINTD_SERVICE: Mutex<Option<Arc<Mutex<MintdService>>>> = Mutex::new(None);

// Global tokio runtime for async operations
static RUNTIME: Mutex<Option<Runtime>> = Mutex::new(None);

/// Initialize the mintd service
#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_initMintdService(
    mut _env: JNIEnv,
    _class: JClass,
    work_dir: JString,
) -> jint {
    info!("Initializing mintd service...");
    let result: Result<i32> = (|| {
        let work_dir_str: String = _env.get_string(&work_dir)?.into();
        let work_dir_path = std::path::PathBuf::from(work_dir_str);
        
        info!("Initializing mintd service with work_dir: {:?}", work_dir_path);
        
        // Initialize tokio runtime
        info!("Creating tokio runtime...");
        let runtime = Runtime::new()?;
        {
            let mut global_runtime = RUNTIME.lock().unwrap();
            *global_runtime = Some(runtime);
        }
        info!("Tokio runtime created");
        
        info!("Creating mintd service...");
        let service = MintdService::new(work_dir_path);
        let service_arc = Arc::new(Mutex::new(service));
        
        {
            let mut global_service = MINTD_SERVICE.lock().unwrap();
            *global_service = Some(service_arc);
        }
        
        info!("Mintd service initialized successfully");
        Ok(0)
    })();
    
    match result {
        Ok(_) => {
            info!("Init completed successfully");
            0
        }
        Err(e) => {
            error!("Failed to initialize mintd service: {}", e);
            -1
        }
    }
}

/// Start the mintd service
#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_startMintdService(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    info!("Starting mintd service...");
    let result: Result<i32> = (|| {
        let runtime_guard = RUNTIME.lock().unwrap();
        let runtime = runtime_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Runtime not initialized"))?;
        
        let service_guard = MINTD_SERVICE.lock().unwrap();
        let service = service_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Mintd service not initialized"))?;
        
        info!("Starting service in runtime...");
        runtime.block_on(async {
            let mut service = service.lock().unwrap();
            service.start().await?;
            Ok::<(), anyhow::Error>(())
        })?;
        
        info!("Mintd service started successfully");
        Ok(0)
    })();
    
    match result {
        Ok(_) => {
            info!("Start completed successfully");
            0
        }
        Err(e) => {
            error!("Failed to start mintd service: {}", e);
            -1
        }
    }
}

/// Stop the mintd service
#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_stopMintdService(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    info!("Stopping mintd service...");
    let result: Result<i32> = (|| {
        let runtime_guard = RUNTIME.lock().unwrap();
        let runtime = runtime_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Runtime not initialized"))?;
        
        let service_guard = MINTD_SERVICE.lock().unwrap();
        let service = service_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Mintd service not initialized"))?;
        
        info!("Stopping service in runtime...");
        runtime.block_on(async {
            let mut service = service.lock().unwrap();
            service.stop().await?;
            Ok::<(), anyhow::Error>(())
        })?;
        
        info!("Mintd service stopped successfully");
        Ok(0)
    })();
    
    match result {
        Ok(_) => {
            info!("Stop completed successfully");
            0
        }
        Err(e) => {
            error!("Failed to stop mintd service: {}", e);
            -1
        }
    }
}

/// Check if mintd service is running
#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_isMintdServiceRunning(
    _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    info!("Checking if mintd service is running...");
    let result: Result<bool> = (|| {
        let service_guard = MINTD_SERVICE.lock().unwrap();
        let service = service_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Mintd service not initialized"))?;
        
        let service = service.lock().unwrap();
        let running = service.is_running();
        Ok(running)
    })();
    
    match result {
        Ok(running) => {
            let result = if running { 1 } else { 0 };
            result
        }
        Err(e) => {
            error!("Failed to check mintd service status: {}", e);
            0
        }
    }
}

/// Get mintd service status as JSON string
#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_getMintdServiceStatus(
    mut _env: JNIEnv,
    _class: JClass,
) -> jstring {
    info!("Getting mintd service status...");
    let result: Result<jstring> = (|| {
        let service_guard = MINTD_SERVICE.lock().unwrap();
        let service = service_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Mintd service not initialized"))?;
        
        let service = service.lock().unwrap();
        let status = service.get_status();
        let status_json = serde_json::to_string(&status)?;
        
        let jni_string = _env.new_string(&status_json)?;
        Ok(jni_string.into_raw())
    })();
    
    match result {
        Ok(jni_string) => {
            jni_string
        }
        Err(e) => {
            error!("Failed to get mintd service status: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Get mintd service URL
#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_getMintdServiceUrl(
    mut _env: JNIEnv,
    _class: JClass,
) -> jstring {
    info!("Getting mintd service URL...");
    let result: Result<jstring> = (|| {
        let service_guard = MINTD_SERVICE.lock().unwrap();
        let service = service_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Mintd service not initialized"))?;
        
        let service = service.lock().unwrap();
        let url = service.get_server_url();
        
        let jni_string = _env.new_string(&url)?;
        Ok(jni_string.into_raw())
    })();
    
    match result {
        Ok(jni_string) => {
            jni_string
        }
        Err(e) => {
            error!("Failed to get mintd service URL: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Handle mint request
#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_handleMintRequest(
    mut _env: JNIEnv,
    _class: JClass,
    amount: jlong,
    unit: JString,
) -> jstring {
    info!("Handling mint request: amount={}, unit={}", amount, _env.get_string(&unit).unwrap().to_string_lossy());
    let result: Result<jstring> = (|| {
        let runtime_guard = RUNTIME.lock().unwrap();
        let runtime = runtime_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Runtime not initialized"))?;
        
        let service_guard = MINTD_SERVICE.lock().unwrap();
        let service = service_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Mintd service not initialized"))?;
        
        let unit_str: String = _env.get_string(&unit)?.into();
        
        info!("Processing mint request in runtime...");
        let response = runtime.block_on(async {
            let service = service.lock().unwrap();
            service.handle_mint_request(amount as u64, &unit_str).await
        })?;
        
        let response_json = serde_json::to_string(&response)?;
        info!("Mint request response: {}", response_json);
        
        let jni_string = _env.new_string(&response_json)?;
        Ok(jni_string.into_raw())
    })();
    
    match result {
        Ok(jni_string) => {
            jni_string
        }
        Err(e) => {
            error!("Failed to handle mint request: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Handle melt request
#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_handleMeltRequest(
    mut _env: JNIEnv,
    _class: JClass,
    quote_id: JString,
) -> jstring {
    let quote_id_binding = _env.get_string(&quote_id).unwrap();
    let quote_id_str = quote_id_binding.to_string_lossy();
    info!("Handling melt request: quote_id={}", quote_id_str);
    let result: Result<jstring> = (|| {
        let runtime_guard = RUNTIME.lock().unwrap();
        let runtime = runtime_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Runtime not initialized"))?;
        
        let service_guard = MINTD_SERVICE.lock().unwrap();
        let service = service_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Mintd service not initialized"))?;
        
        let quote_id_str: String = _env.get_string(&quote_id)?.into();
        
        info!("Processing melt request in runtime...");
        let response = runtime.block_on(async {
            let service = service.lock().unwrap();
            service.handle_melt_request(&quote_id_str).await
        })?;
        
        let response_json = serde_json::to_string(&response)?;
        info!("Melt request response: {}", response_json);
        
        let jni_string = _env.new_string(&response_json)?;
        Ok(jni_string.into_raw())
    })();
    
    match result {
        Ok(jni_string) => {
            jni_string
        }
        Err(e) => {
            error!("Failed to handle melt request: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Get mint info
#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_getMintInfo(
    mut _env: JNIEnv,
    _class: JClass,
) -> jstring {
    info!("Getting mint info...");
    let result: Result<jstring> = (|| {
        let runtime_guard = RUNTIME.lock().unwrap();
        let runtime = runtime_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Runtime not initialized"))?;
        
        let service_guard = MINTD_SERVICE.lock().unwrap();
        let service = service_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Mintd service not initialized"))?;
        
        info!("Getting mint info in runtime...");
        let mint_info = runtime.block_on(async {
            let service = service.lock().unwrap();
            service.mint_info().await
        })?;
        
        let mint_info_json = serde_json::to_string(&mint_info)?;
        info!("Mint info: {}", mint_info_json);
        
        let jni_string = _env.new_string(&mint_info_json)?;
        Ok(jni_string.into_raw())
    })();
    
    match result {
        Ok(jni_string) => {
            jni_string
        }
        Err(e) => {
            error!("Failed to get mint info: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Get keys
#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_getKeys(
    mut _env: JNIEnv,
    _class: JClass,
) -> jstring {
    info!("Getting keys...");
    let result: Result<jstring> = (|| {
        let runtime_guard = RUNTIME.lock().unwrap();
        let runtime = runtime_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Runtime not initialized"))?;
        
        let service_guard = MINTD_SERVICE.lock().unwrap();
        let service = service_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Mintd service not initialized"))?;
        
        info!("Getting keys in runtime...");
        let keys = runtime.block_on(async {
            let service = service.lock().unwrap();
            service.get_keys().await
        })?;
        
        let keys_json = serde_json::to_string(&keys)?;
        info!("Keys: {}", keys_json);
        
        let jni_string = _env.new_string(&keys_json)?;
        Ok(jni_string.into_raw())
    })();
    
    match result {
        Ok(jni_string) => {
            jni_string
        }
        Err(e) => {
            error!("Failed to get keys: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Get keysets
#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_getKeysets(
    mut _env: JNIEnv,
    _class: JClass,
) -> jstring {
    info!("Getting keysets...");
    let result: Result<jstring> = (|| {
        let runtime_guard = RUNTIME.lock().unwrap();
        let runtime = runtime_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Runtime not initialized"))?;
        
        let service_guard = MINTD_SERVICE.lock().unwrap();
        let service = service_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Mintd service not initialized"))?;
        
        info!("Getting keysets in runtime...");
        let keysets = runtime.block_on(async {
            let service = service.lock().unwrap();
            service.get_keysets().await
        })?;
        
        let keysets_json = serde_json::to_string(&keysets)?;
        info!("Keysets: {}", keysets_json);
        
        let jni_string = _env.new_string(&keysets_json)?;
        Ok(jni_string.into_raw())
    })();
    
    match result {
        Ok(jni_string) => {
            jni_string
        }
        Err(e) => {
            error!("Failed to get keysets: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Get keyset pubkeys
#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_getKeysetPubkeys(
    mut _env: JNIEnv,
    _class: JClass,
    keyset_id: JString,
) -> jstring {
    let keyset_id_binding = _env.get_string(&keyset_id).unwrap();
    let keyset_id_str = keyset_id_binding.to_string_lossy();
    info!("Getting keyset pubkeys: keyset_id={}", keyset_id_str);
    let result: Result<jstring> = (|| {
        let runtime_guard = RUNTIME.lock().unwrap();
        let runtime = runtime_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Runtime not initialized"))?;
        
        let service_guard = MINTD_SERVICE.lock().unwrap();
        let service = service_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Mintd service not initialized"))?;
        
        let keyset_id_str: String = _env.get_string(&keyset_id)?.into();
        
        info!("Getting keyset pubkeys in runtime...");
        let keys = runtime.block_on(async {
            let service = service.lock().unwrap();
            service.get_keyset_pubkeys(&keyset_id_str).await
        })?;
        
        let keys_json = serde_json::to_string(&keys)?;
        info!("Keyset pubkeys: {}", keys_json);
        
        let jni_string = _env.new_string(&keys_json)?;
        Ok(jni_string.into_raw())
    })();
    
    match result {
        Ok(jni_string) => {
            jni_string
        }
        Err(e) => {
            error!("Failed to get keyset pubkeys: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Get mint quote
#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_getMintQuote(
    mut _env: JNIEnv,
    _class: JClass,
    amount: jlong,
    unit: JString,
) -> jstring {
    let unit_binding = _env.get_string(&unit).unwrap();
    let unit_str = unit_binding.to_string_lossy();
    info!("Getting mint quote: amount={}, unit={}", amount, unit_str);
    let result: Result<jstring> = (|| {
        let runtime_guard = RUNTIME.lock().unwrap();
        let runtime = runtime_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Runtime not initialized"))?;
        
        let service_guard = MINTD_SERVICE.lock().unwrap();
        let service = service_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Mintd service not initialized"))?;
        
        let unit_str: String = _env.get_string(&unit)?.into();
        
        info!("Getting mint quote in runtime...");
        let quote = runtime.block_on(async {
            let service = service.lock().unwrap();
            service.get_mint_quote(amount as u64, &unit_str).await
        })?;
        
        let quote_json = serde_json::to_string(&quote)?;
        info!("Mint quote: {}", quote_json);
        
        let jni_string = _env.new_string(&quote_json)?;
        Ok(jni_string.into_raw())
    })();
    
    match result {
        Ok(jni_string) => {
            jni_string
        }
        Err(e) => {
            error!("Failed to get mint quote: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Check mint quote
#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_checkMintQuote(
    mut _env: JNIEnv,
    _class: JClass,
    quote_id: JString,
) -> jstring {
    let quote_id_binding = _env.get_string(&quote_id).unwrap();
    let quote_id_str = quote_id_binding.to_string_lossy();
    info!("Checking mint quote: quote_id={}", quote_id_str);
    let result: Result<jstring> = (|| {
        let runtime_guard = RUNTIME.lock().unwrap();
        let runtime = runtime_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Runtime not initialized"))?;
        
        let service_guard = MINTD_SERVICE.lock().unwrap();
        let service = service_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Mintd service not initialized"))?;
        
        let quote_id_str: String = _env.get_string(&quote_id)?.into();
        
        info!("Checking mint quote in runtime...");
        let quote = runtime.block_on(async {
            let service = service.lock().unwrap();
            service.check_mint_quote(&quote_id_str).await
        })?;
        
        let quote_json = serde_json::to_string(&quote)?;
        info!("Mint quote check result: {}", quote_json);
        
        let jni_string = _env.new_string(&quote_json)?;
        Ok(jni_string.into_raw())
    })();
    
    match result {
        Ok(jni_string) => {
            jni_string
        }
        Err(e) => {
            error!("Failed to check mint quote: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Get melt quote
#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_getMeltQuote(
    mut _env: JNIEnv,
    _class: JClass,
    amount: jlong,
    unit: JString,
    invoice: JString,
) -> jstring {
    let unit_binding = _env.get_string(&unit).unwrap();
    let unit_str = unit_binding.to_string_lossy();
    let invoice_binding = _env.get_string(&invoice).unwrap();
    let invoice_str = invoice_binding.to_string_lossy();
    info!("Getting melt quote: amount={}, unit={}, invoice={}", amount, unit_str, invoice_str);
    let result: Result<jstring> = (|| {
        let runtime_guard = RUNTIME.lock().unwrap();
        let runtime = runtime_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Runtime not initialized"))?;
        
        let service_guard = MINTD_SERVICE.lock().unwrap();
        let service = service_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Mintd service not initialized"))?;
        
        let unit_str: String = _env.get_string(&unit)?.into();
        let invoice_str: String = _env.get_string(&invoice)?.into();
        
        info!("Getting melt quote in runtime...");
        let quote = runtime.block_on(async {
            let service = service.lock().unwrap();
            service.get_melt_quote(amount as u64, &unit_str, &invoice_str).await
        })?;
        
        let quote_json = serde_json::to_string(&quote)?;
        info!("Melt quote: {}", quote_json);
        
        let jni_string = _env.new_string(&quote_json)?;
        Ok(jni_string.into_raw())
    })();
    
    match result {
        Ok(jni_string) => {
            jni_string
        }
        Err(e) => {
            error!("Failed to get melt quote: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Check melt quote
#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_checkMeltQuote(
    mut _env: JNIEnv,
    _class: JClass,
    quote_id: JString,
) -> jstring {
    let quote_id_binding = _env.get_string(&quote_id).unwrap();
    let quote_id_str = quote_id_binding.to_string_lossy();
    info!("Checking melt quote: quote_id={}", quote_id_str);
    let result: Result<jstring> = (|| {
        let runtime_guard = RUNTIME.lock().unwrap();
        let runtime = runtime_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Runtime not initialized"))?;
        
        let service_guard = MINTD_SERVICE.lock().unwrap();
        let service = service_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Mintd service not initialized"))?;
        
        let quote_id_str: String = _env.get_string(&quote_id)?.into();
        
        info!("Checking melt quote in runtime...");
        let quote = runtime.block_on(async {
            let service = service.lock().unwrap();
            service.check_melt_quote(&quote_id_str).await
        })?;
        
        let quote_json = serde_json::to_string(&quote)?;
        info!("Melt quote check result: {}", quote_json);
        
        let jni_string = _env.new_string(&quote_json)?;
        Ok(jni_string.into_raw())
    })();
    
    match result {
        Ok(jni_string) => {
            jni_string
        }
        Err(e) => {
            error!("Failed to check melt quote: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Check proofs
#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_checkProofs(
    mut _env: JNIEnv,
    _class: JClass,
    proofs_json: JString,
) -> jstring {
    let proofs_json_binding = _env.get_string(&proofs_json).unwrap();
    let proofs_json_str = proofs_json_binding.to_string_lossy();
    info!("Checking proofs: proofs_json={}", proofs_json_str);
    let result: Result<jstring> = (|| {
        let runtime_guard = RUNTIME.lock().unwrap();
        let runtime = runtime_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Runtime not initialized"))?;
        
        let service_guard = MINTD_SERVICE.lock().unwrap();
        let service = service_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Mintd service not initialized"))?;
        
        let proofs_json_str: String = _env.get_string(&proofs_json)?.into();
        let proofs: Vec<cdk::nuts::nut00::Proof> = serde_json::from_str(&proofs_json_str)?;
        
        info!("Checking proofs in runtime...");
        let response = runtime.block_on(async {
            let service = service.lock().unwrap();
            service.check_proofs(proofs).await
        })?;
        
        let response_json = serde_json::to_string(&response)?;
        info!("Proofs check result: {}", response_json);
        
        let jni_string = _env.new_string(&response_json)?;
        Ok(jni_string.into_raw())
    })();
    
    match result {
        Ok(jni_string) => {
            jni_string
        }
        Err(e) => {
            error!("Failed to check proofs: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Restore tokens
#[no_mangle]
pub extern "system" fn Java_com_example_purrmint_PurrmintNative_restoreTokens(
    mut _env: JNIEnv,
    _class: JClass,
    outputs_json: JString,
) -> jstring {
    let outputs_json_binding = _env.get_string(&outputs_json).unwrap();
    let outputs_json_str = outputs_json_binding.to_string_lossy();
    info!("Restoring tokens: outputs_json={}", outputs_json_str);
    let result: Result<jstring> = (|| {
        let runtime_guard = RUNTIME.lock().unwrap();
        let runtime = runtime_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Runtime not initialized"))?;
        
        let service_guard = MINTD_SERVICE.lock().unwrap();
        let service = service_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Mintd service not initialized"))?;
        
        let outputs_json_str: String = _env.get_string(&outputs_json)?.into();
        let outputs: Vec<cdk::nuts::nut00::BlindedMessage> = serde_json::from_str(&outputs_json_str)?;
        
        info!("Restoring tokens in runtime...");
        let response = runtime.block_on(async {
            let service = service.lock().unwrap();
            service.restore_tokens(outputs).await
        })?;
        
        let response_json = serde_json::to_string(&response)?;
        info!("Restore tokens result: {}", response_json);
        
        let jni_string = _env.new_string(&response_json)?;
        Ok(jni_string.into_raw())
    })();
    
    match result {
        Ok(jni_string) => {
            jni_string
        }
        Err(e) => {
            error!("Failed to restore tokens: {}", e);
            std::ptr::null_mut()
        }
    }
} 