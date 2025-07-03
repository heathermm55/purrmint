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
    let result: Result<i32> = (|| {
        let work_dir_str: String = _env.get_string(&work_dir)?.into();
        let work_dir_path = std::path::PathBuf::from(work_dir_str);
        
        info!("Initializing mintd service with work_dir: {:?}", work_dir_path);
        
        // Initialize tokio runtime
        let runtime = Runtime::new()?;
        {
            let mut global_runtime = RUNTIME.lock().unwrap();
            *global_runtime = Some(runtime);
        }
        
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
        Ok(_) => 0,
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
    let result: Result<i32> = (|| {
        let runtime_guard = RUNTIME.lock().unwrap();
        let runtime = runtime_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Runtime not initialized"))?;
        
        let service_guard = MINTD_SERVICE.lock().unwrap();
        let service = service_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Mintd service not initialized"))?;
        
        runtime.block_on(async {
            let mut service = service.lock().unwrap();
            service.start().await?;
            Ok::<(), anyhow::Error>(())
        })?;
        
        info!("Mintd service started successfully");
        Ok(0)
    })();
    
    match result {
        Ok(_) => 0,
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
    let result: Result<i32> = (|| {
        let runtime_guard = RUNTIME.lock().unwrap();
        let runtime = runtime_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Runtime not initialized"))?;
        
        let service_guard = MINTD_SERVICE.lock().unwrap();
        let service = service_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Mintd service not initialized"))?;
        
        runtime.block_on(async {
            let mut service = service.lock().unwrap();
            service.stop().await?;
            Ok::<(), anyhow::Error>(())
        })?;
        
        info!("Mintd service stopped successfully");
        Ok(0)
    })();
    
    match result {
        Ok(_) => 0,
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
    let result: Result<bool> = (|| {
        let service_guard = MINTD_SERVICE.lock().unwrap();
        let service = service_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Mintd service not initialized"))?;
        
        let service = service.lock().unwrap();
        Ok(service.is_running())
    })();
    
    match result {
        Ok(running) => if running { 1 } else { 0 },
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
        Ok(jni_string) => jni_string,
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
        Ok(jni_string) => jni_string,
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
    let result: Result<jstring> = (|| {
        let service_guard = MINTD_SERVICE.lock().unwrap();
        let service = service_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Mintd service not initialized"))?;
        
        let unit_str: String = _env.get_string(&unit)?.into();
        let runtime_guard = RUNTIME.lock().unwrap();
        let runtime = runtime_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Runtime not initialized"))?;
        
        let response = runtime.block_on(async {
            let service = service.lock().unwrap();
            service.handle_mint_request(amount as u64, &unit_str).await
        })?;
        
        let response_json = serde_json::to_string(&response)?;
        let jni_string = _env.new_string(&response_json)?;
        Ok(jni_string.into_raw())
    })();
    
    match result {
        Ok(jni_string) => jni_string,
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
    let result: Result<jstring> = (|| {
        let service_guard = MINTD_SERVICE.lock().unwrap();
        let service = service_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Mintd service not initialized"))?;
        
        let quote_id_str: String = _env.get_string(&quote_id)?.into();
        let runtime_guard = RUNTIME.lock().unwrap();
        let runtime = runtime_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Runtime not initialized"))?;
        
        let response = runtime.block_on(async {
            let service = service.lock().unwrap();
            service.handle_melt_request(&quote_id_str).await
        })?;
        
        let response_json = serde_json::to_string(&response)?;
        let jni_string = _env.new_string(&response_json)?;
        Ok(jni_string.into_raw())
    })();
    
    match result {
        Ok(jni_string) => jni_string,
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
    let result: Result<jstring> = (|| {
        let service_guard = MINTD_SERVICE.lock().unwrap();
        let service = service_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Mintd service not initialized"))?;
        
        let runtime_guard = RUNTIME.lock().unwrap();
        let runtime = runtime_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Runtime not initialized"))?;
        
        let mint_info = runtime.block_on(async {
            let service = service.lock().unwrap();
            service.mint_info().await
        })?;
        
        let info_json = serde_json::to_string(&mint_info)?;
        let jni_string = _env.new_string(&info_json)?;
        Ok(jni_string.into_raw())
    })();
    
    match result {
        Ok(jni_string) => jni_string,
        Err(e) => {
            error!("Failed to get mint info: {}", e);
            std::ptr::null_mut()
        }
    }
} 