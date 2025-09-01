//! Core functionality for PurrMint
//! Internal module containing shared functions for Android integration

use std::sync::{Arc, Mutex, OnceLock};
use std::ffi::{CString, c_char};
use serde_json::json;
use tracing::{info, error, warn};

use crate::config::AndroidConfig;
use crate::nostr::{nsec_to_npub as nostr_nsec_to_npub};
use crate::mintd_service::MintdService;
use crate::tor_service::TorService;

/// Global state for the mint service
static mut MINT_SERVICE: Option<Arc<Mutex<Option<MintdService>>>> = None;

/// Global state for the Tor service
static mut TOR_SERVICE: Option<Arc<Mutex<Option<TorService>>>> = None;

/// Global runtime for service management
static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

/// Service state lock to prevent concurrent start/stop operations
static SERVICE_LOCK: OnceLock<Arc<Mutex<()>>> = OnceLock::new();

/// Initialize global state and runtime
fn init_globals() {
    unsafe {
        if MINT_SERVICE.is_none() {
            MINT_SERVICE = Some(Arc::new(Mutex::new(None)));
        }
        if TOR_SERVICE.is_none() {
            TOR_SERVICE = Some(Arc::new(Mutex::new(None)));
        }
    }
    
    // Initialize runtime if not already done
    RUNTIME.get_or_init(|| {
        tokio::runtime::Runtime::new().expect("Failed to create global runtime")
    });
    
    // Initialize service lock
    SERVICE_LOCK.get_or_init(|| {
        Arc::new(Mutex::new(()))
    });
}

/// Get service lock for thread-safe operations
fn get_service_lock() -> Arc<Mutex<()>> {
    init_globals();
    SERVICE_LOCK.get().unwrap().clone()
}

// =============================================================================
// Basic functionality
// =============================================================================

/// Initialize logging for Android
#[no_mangle]
pub extern "C" fn init_logging() -> *mut c_char {
    match init_logging_internal() {
        Ok(_) => {
            let result = CString::new("Logging initialized successfully").unwrap();
            result.into_raw()
        }
        Err(e) => {
            let error_msg = format!("Failed to initialize logging: {}", e);
            let result = CString::new(error_msg).unwrap();
            result.into_raw()
        }
    }
}

/// Initialize logging for Android (internal function)
pub fn init_logging_internal() -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(log::LevelFilter::Debug)
                .with_tag("PurrMint")
        );
    }
    
    #[cfg(not(target_os = "android"))]
    {
        tracing_subscriber::fmt()
            .with_env_filter("purrmint=debug,tracing=debug")
            .with_target(false)
            .init();
    }
    
    info!("PurrMint logging initialized");
    info!("Log level set to debug");
    info!("Android logger configured for logcat output");
    Ok(())
}

/// Convert nsec to npub
#[no_mangle]
pub extern "C" fn nsec_to_npub(nsec: *const c_char) -> *mut c_char {
    let nsec_str = unsafe {
        match CString::from_raw(nsec as *mut c_char).into_string() {
            Ok(s) => s,
            Err(_) => {
                let error = CString::new("Invalid nsec string").unwrap();
                return error.into_raw();
            }
        }
    };
    
    match nostr_nsec_to_npub(&nsec_str) {
        Ok(npub) => {
            let result = CString::new(npub).unwrap();
            result.into_raw()
        }
        Err(e) => {
            let error_msg = format!("Failed to convert nsec to npub: {}", e);
            let result = CString::new(error_msg).unwrap();
            result.into_raw()
        }
    }
}

/// Convert nsec to npub (wrapper for nostr module function)
pub fn nsec_to_npub_internal(nsec: &str) -> Result<String, String> {
    nostr_nsec_to_npub(nsec).map_err(|e| e.to_string())
}

/// Load Android configuration from file
#[no_mangle]
pub extern "C" fn load_android_config_from_file(file_path: *const c_char) -> *mut c_char {
    let file_path_str = unsafe {
        match CString::from_raw(file_path as *mut c_char).into_string() {
            Ok(s) => s,
            Err(_) => {
                let error = CString::new("Invalid file path").unwrap();
                return error.into_raw();
            }
        }
    };
    
    match load_android_config_from_file_internal(&file_path_str) {
        Ok(config) => {
            let result = CString::new(config).unwrap();
            result.into_raw()
        }
        Err(e) => {
            let error_msg = format!("Failed to load config: {}", e);
            let result = CString::new(error_msg).unwrap();
            result.into_raw()
        }
    }
}

/// Load Android configuration from JSON file
pub fn load_android_config_from_file_internal(file_path: &str) -> Result<String, String> {
    info!("Loading Android config from file: {}", file_path);
    
    if !std::path::Path::new(file_path).exists() {
        error!("Config file does not exist: {}", file_path);
        return Err(format!("Config file does not exist: {}", file_path));
    }
    
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;
    
    // Validate by parsing
    let config = AndroidConfig::from_json(&content)
        .map_err(|e| format!("Invalid config file format: {}", e))?;
    
    let json = config.to_json()
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    
    info!("Android config loaded successfully");
    Ok(json)
}

/// Save Android configuration to file
#[no_mangle]
pub extern "C" fn save_android_config_to_file(file_path: *const c_char, config_json: *const c_char) -> *mut c_char {
    let file_path_str = unsafe {
        match CString::from_raw(file_path as *mut c_char).into_string() {
            Ok(s) => s,
            Err(_) => {
                let error = CString::new("Invalid file path").unwrap();
                return error.into_raw();
            }
        }
    };
    
    let config_json_str = unsafe {
        match CString::from_raw(config_json as *mut c_char).into_string() {
            Ok(s) => s,
            Err(_) => {
                let error = CString::new("Invalid config JSON").unwrap();
                return error.into_raw();
            }
        }
    };
    
    match save_android_config_to_file_internal(&file_path_str, &config_json_str) {
        Ok(_) => {
            let result = CString::new("Configuration saved successfully").unwrap();
            result.into_raw()
        }
        Err(e) => {
            let error_msg = format!("Failed to save config: {}", e);
            let result = CString::new(error_msg).unwrap();
            result.into_raw()
        }
    }
}

/// Save Android configuration to JSON file
pub fn save_android_config_to_file_internal(file_path: &str, config_json: &str) -> Result<(), String> {
    info!("Saving Android config to file: {}", file_path);
    
    // Validate JSON by parsing it
    let config = AndroidConfig::from_json(config_json)
        .map_err(|e| format!("Invalid config JSON: {}", e))?;
    
    // Create parent directory if needed
    if let Some(parent) = std::path::Path::new(file_path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }
    
    let json = config.to_json()
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    
    std::fs::write(file_path, &json)
        .map_err(|e| format!("Failed to write config file: {}", e))?;
    
    info!("Android config saved successfully");
    Ok(())
}

/// Generate default Android configuration
#[no_mangle]
pub extern "C" fn generate_default_android_config() -> *mut c_char {
    match generate_default_android_config_internal() {
        Ok(config) => {
            let result = CString::new(config).unwrap();
            result.into_raw()
        }
        Err(e) => {
            let error_msg = format!("Failed to generate default config: {}", e);
            let result = CString::new(error_msg).unwrap();
            result.into_raw()
        }
    }
}

/// Generate default Android configuration JSON
pub fn generate_default_android_config_internal() -> Result<String, String> {
    let config = AndroidConfig::default();
    config.to_json().map_err(|e| format!("Failed to serialize default config: {}", e))
}

// =============================================================================
// Service management
// =============================================================================

/// Start Android service with configuration
pub fn start_android_service(config: &AndroidConfig, nsec: &str) -> Result<(), String> {
    info!("Starting Android service...");
    
    if nsec.is_empty() {
        return Err("nsec is empty".to_string());
    }
    
    info!("Service configuration: port={}, host={}", config.port, config.host);
    
    // Acquire service lock to prevent concurrent start/stop operations
    let lock = get_service_lock();
    let _lock = lock.lock().map_err(|e| format!("Failed to acquire service lock: {}", e))?;
    
    // Always stop existing service first to ensure clean state
    info!("Ensuring clean service state...");
    if let Err(e) = stop_service_internal() {
        warn!("Failed to stop existing service: {}", e);
        // Continue anyway, as the service might not be running
    }
    
    let config_path = std::path::Path::new(&config.database_path)
        .parent()
        .ok_or("Invalid database path")?
        .to_path_buf();
    
    // Create directory if needed
    std::fs::create_dir_all(&config_path)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;
    
    // Check if service is already running (double-check after stop)
    init_globals();
    unsafe {
        if let Some(service_guard) = MINT_SERVICE.as_ref() {
            if let Ok(guard) = service_guard.lock() {
                if let Some(service) = guard.as_ref() {
                    if service.is_running() {
                        info!("Service is still running after stop attempt, waiting...");
                        // Give the service a moment to fully stop
                        std::thread::sleep(std::time::Duration::from_millis(500));
                        
                        // Try stopping again
                        if let Err(e) = stop_service_internal() {
                            return Err(format!("Failed to stop existing service: {}", e));
                        }
                    }
                }
            }
        }
    }
    
    // Start Tor service if enabled
    if config.tor_enabled.unwrap_or(false) {
        info!("Starting Tor service...");
        let tor_config = config.to_tor_config();
        let mut tor_service = TorService::with_config(tor_config)
            .map_err(|e| format!("Failed to create Tor service: {}", e))?;
        
        let rt = RUNTIME.get().unwrap();
        rt.block_on(async {
            match tor_service.start().await {
                Ok(()) => {
                    info!("Tor service started successfully");
                    
                    // Store Tor service in global state
                    unsafe {
                        if let Some(tor_service_guard) = TOR_SERVICE.as_ref() {
                            if let Ok(mut guard) = tor_service_guard.lock() {
                                *guard = Some(tor_service);
                            }
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to start Tor service: {}", e);
                    Err(format!("Failed to start Tor service: {}", e))
                }
            }
        })?;
        
        // Create hidden service if enabled
        if config.tor_enable_hidden_services.unwrap_or(false) {
            info!("Creating Tor hidden service...");
            let rt = RUNTIME.get().unwrap();
            rt.block_on(async {
                unsafe {
                    if let Some(tor_service_guard) = TOR_SERVICE.as_ref() {
                        if let Ok(guard) = tor_service_guard.lock() {
                            if let Some(tor_service) = guard.as_ref() {
                                // Use nsec as nickname for the hidden service
                                let nickname = format!("mint_{}", &nsec[..8]);
                                match tor_service.create_hidden_service(&nickname).await {
                                    Ok(info) => {
                                        info!("Hidden service created: {}", info.onion_address);
                                        Ok(())
                                    }
                                    Err(e) => {
                                        error!("Failed to create hidden service: {}", e);
                                        Err(format!("Failed to create hidden service: {}", e))
                                    }
                                }
                            } else {
                                Err("Tor service not available".to_string())
                            }
                        } else {
                            Err("Failed to lock Tor service".to_string())
                        }
                    } else {
                        Err("Tor service not initialized".to_string())
                    }
                }
            })?;
        }
    }
    
    // Create and start mint service using global runtime
    let mut mint_service = MintdService::new_with_android_config(config_path, config, nsec.to_string())
        .map_err(|e| format!("Failed to create mint service: {}", e))?;
    
    let rt = RUNTIME.get().unwrap();
    rt.block_on(async move {
        match mint_service.start().await {
            Ok(()) => {
                info!("MintdService started successfully");
                
                // Store service in global state
                unsafe {
                    if let Some(service_guard) = MINT_SERVICE.as_ref() {
                        if let Ok(mut guard) = service_guard.lock() {
                            *guard = Some(mint_service);
                        }
                    }
                }
                Ok(())
            }
            Err(e) => {
                error!("Failed to start MintdService: {}", e);
                Err(format!("Failed to start MintdService: {}", e))
            }
        }
    })
}

/// Internal stop service function (without lock)
fn stop_service_internal() -> Result<(), String> {
    info!("Stopping mint service (internal)...");
    init_globals();
    
    unsafe {
        if let Some(service_guard) = MINT_SERVICE.as_ref() {
            if let Ok(mut guard) = service_guard.lock() {
                if let Some(mut service) = guard.take() {
                    let rt = RUNTIME.get().unwrap();
                    
                    return rt.block_on(async move {
                        service.stop().await
                            .map_err(|e| format!("Failed to stop service: {}", e))
                    });
                }
            }
        }
    }
    
    info!("No running service found to stop");
    Ok(())
}

/// Stop mint service (thread-safe)
pub fn stop_service() -> Result<(), String> {
    info!("Stopping mint service...");
    
    // Acquire service lock to prevent concurrent start/stop operations
    let lock = get_service_lock();
    let _lock = lock.lock().map_err(|e| format!("Failed to acquire service lock: {}", e))?;
    
    stop_service_internal()
}

/// Delete mint service and clean up resources
pub fn delete_mint_service() -> Result<(), String> {
    info!("Deleting mint service and cleaning up resources...");
    
    // Acquire service lock to prevent concurrent operations
    let lock = get_service_lock();
    let _lock = lock.lock().map_err(|e| format!("Failed to acquire service lock: {}", e))?;
    
    // Stop the service first if it's running
    if let Err(e) = stop_service_internal() {
        warn!("Failed to stop service before deletion: {}", e);
        // Continue with deletion anyway
    }
    
    // Clean up global state
    init_globals();
    unsafe {
        // Clear mint service
        if let Some(service_guard) = MINT_SERVICE.as_ref() {
            if let Ok(mut guard) = service_guard.lock() {
                *guard = None;
            }
        }
        
        // Clear Tor service
        if let Some(tor_service_guard) = TOR_SERVICE.as_ref() {
            if let Ok(mut guard) = tor_service_guard.lock() {
                *guard = None;
            }
        }
    }
    
    // Clean up files and directories
    if let Err(e) = cleanup_mint_files() {
        warn!("Failed to clean up some files: {}", e);
        // Continue with deletion anyway
    }
    
    // Clean up Android-specific configuration files
    if let Err(e) = cleanup_android_config_files() {
        warn!("Failed to clean up Android config files: {}", e);
        // Continue with deletion anyway
    }
    
    // Verify cleanup was successful
    if let Err(e) = verify_cleanup() {
        warn!("Cleanup verification failed: {}", e);
        // Continue anyway, as some files might be locked
    }
    
    info!("Mint service deleted and resources cleaned up");
    Ok(())
}

/// Clean up mint-related files and directories
fn cleanup_mint_files() -> Result<(), String> {
    info!("Cleaning up mint files and directories...");
    
    // Get all paths to clean up from configuration
    let paths_to_clean = get_paths_to_clean_from_config();
    
    for path_str in paths_to_clean {
        let path = std::path::Path::new(&path_str);
        if path.exists() {
            if let Err(e) = remove_path(path) {
                warn!("Failed to remove path {}: {}", path_str, e);
            } else {
                info!("Removed path: {}", path_str);
            }
        }
    }
    
    // Try to remove the main data directory if it's empty
    let data_dir = get_android_data_dir();
    let data_path = std::path::Path::new(&data_dir);
    if data_path.exists() && data_path.is_dir() {
        if let Ok(entries) = std::fs::read_dir(data_path) {
            if entries.count() == 0 {
                if let Err(e) = std::fs::remove_dir(data_path) {
                    warn!("Failed to remove empty data directory {}: {}", data_dir, e);
                } else {
                    info!("Removed empty data directory: {}", data_dir);
                }
            }
        }
    }
    
    Ok(())
}

/// Clean up Android-specific configuration files
fn cleanup_android_config_files() -> Result<(), String> {
    info!("Cleaning up Android configuration files...");
    
    // Get all paths to clean up from configuration
    let mut paths_to_clean = get_paths_to_clean_from_config();
    
    // Note: We do NOT delete the main android_config.json file
    // This preserves user configuration for future use
    // We only clean up runtime data (database, logs, etc.)
    
    // Remove duplicates
    paths_to_clean.sort();
    paths_to_clean.dedup();
    
    info!("Android paths to clean up: {:?}", paths_to_clean);
    
    for path_str in paths_to_clean {
        let path = std::path::Path::new(&path_str);
        if path.exists() {
            if let Err(e) = std::fs::remove_file(path) {
                warn!("Failed to remove config file {}: {}", path_str, e);
            } else {
                info!("Removed config file: {}", path_str);
            }
        }
    }
    
    // Note: We do NOT clean up:
    // - android_config.json (preserves user configuration)
    // - shared_prefs (preserves user preferences)
    // - nostr_account.json (preserves user account)
    // - tor_data (preserves Tor configuration)
    // These contain user data and should be preserved for security and user experience
    
    Ok(())
}

/// Remove a file or directory recursively
fn remove_path(path: &std::path::Path) -> Result<(), String> {
    if path.is_file() {
        std::fs::remove_file(path)
            .map_err(|e| format!("Failed to remove file {}: {}", path.display(), e))
    } else if path.is_dir() {
        std::fs::remove_dir_all(path)
            .map_err(|e| format!("Failed to remove directory {}: {}", path.display(), e))
    } else {
        // Path doesn't exist or is a symlink, ignore
        Ok(())
    }
}

/// Get Android data directory for mint service
fn get_android_data_dir() -> String {
    // Try to get from environment variable first
    if let Ok(data_dir) = std::env::var("ANDROID_DATA_DIR") {
        return data_dir;
    }
    
    // Try to get from existing configuration if available
    let config_paths = get_config_file_paths();
    
    for config_path in config_paths {
        if let Ok(config_json) = std::fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str::<crate::config::AndroidConfig>(&config_json) {
                // Extract directory from database_path
                if let Some(db_path) = std::path::Path::new(&config.database_path).parent() {
                    let data_dir = db_path.to_string_lossy().to_string();
                    info!("Using data directory from config: {}", data_dir);
                    return data_dir;
                }
                
                // If database_path is empty, try logs_path
                if let Some(logs_path) = std::path::Path::new(&config.logs_path).parent() {
                    let data_dir = logs_path.to_string_lossy().to_string();
                    info!("Using data directory from logs_path: {}", data_dir);
                    return data_dir;
                }
            }
        }
    }
    
    // No fallback - return empty string if no config found
    warn!("No configuration found, cannot determine data directory");
    "".to_string()
}

/// Check if mint service exists
pub fn mint_service_exists() -> bool {
    init_globals();
    
    unsafe {
        if let Some(service_guard) = MINT_SERVICE.as_ref() {
            if let Ok(guard) = service_guard.lock() {
                guard.is_some()
            } else {
                false
            }
        } else {
            false
        }
    }
}

/// Get service status
pub fn get_service_status() -> String {
    init_globals();
    
    unsafe {
        if let Some(service_guard) = MINT_SERVICE.as_ref() {
            if let Ok(guard) = service_guard.lock() {
                if let Some(service) = guard.as_ref() {
                    return service.get_status().to_string();
                }
            }
        }
    }
    
    json!({
        "running": false,
        "details": "Service not initialized"
    }).to_string()
}

/// Get onion address if available
#[no_mangle]
pub extern "C" fn get_onion_address() -> *mut c_char {
    match get_onion_address_internal() {
        Some(address) => {
            let result = CString::new(address).unwrap();
            result.into_raw()
        }
        None => {
            let result = CString::new("No onion address available").unwrap();
            result.into_raw()
        }
    }
}

/// Get onion address if available
pub fn get_onion_address_internal() -> Option<String> {
    init_globals();
    
    unsafe {
        if let Some(tor_service_guard) = TOR_SERVICE.as_ref() {
            if let Ok(guard) = tor_service_guard.lock() {
                if let Some(tor_service) = guard.as_ref() {
                    // Get the first hidden service's onion address
                    let rt = RUNTIME.get().unwrap();
                    return rt.block_on(async {
                        let services = tor_service.list_hidden_services().await;
                        match services {
                            Ok(services) => {
                                if let Some(first_service) = services.first() {
                                    Some(first_service.onion_address.clone())
                                } else {
                                    None
                                }
                            }
                            Err(_) => None
                        }
                    });
                }
            }
        }
    }
    
    None
}

/// Free string allocated by Rust
#[no_mangle]
pub extern "C" fn free_string(s: *mut c_char) {
    unsafe {
        if !s.is_null() {
            let _ = CString::from_raw(s);
        }
    }
}

/// Check if cleanup was successful
fn verify_cleanup() -> Result<(), String> {
    info!("Verifying cleanup...");
    
    let data_dir = get_android_data_dir();
    let data_path = std::path::Path::new(&data_dir);
    
    // Check if main data directory still exists
    if data_path.exists() {
        // Check if it contains any important files
        if let Ok(entries) = std::fs::read_dir(data_path) {
            let remaining_files: Vec<_> = entries
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    let file_name = entry.file_name();
                    let name = file_name.to_string_lossy();
                    // Check if any important files remain
                    // Note: SSL certificates are intentionally preserved for security
                    name.contains("mint.db") || 
                    name.contains("android_config") || 
                    name.contains("nostr_account")
                    // SSL certificates are NOT checked: name.contains("purrmint_cert") || name.contains("purrmint_key")
                })
                .collect();
            
            if !remaining_files.is_empty() {
                warn!("Some files still remain after cleanup: {:?}", 
                    remaining_files.iter().map(|e| e.file_name()).collect::<Vec<_>>());
                return Err("Some files could not be removed during cleanup".to_string());
            }
        }
    }
    
    // Check Android-specific paths dynamically from config
    let mut android_paths: Vec<String> = vec![
        "/data/data/com.purrmint.app/files/android_config.json".to_string(),
        "/data/data/com.purrmint.app/files/nostr_account.json".to_string(),
    ];
    
    // Try to load configuration to get additional paths to check
    if let Ok(config_json) = std::fs::read_to_string("/data/data/com.purrmint.app/files/android_config.json") {
        if let Ok(config) = serde_json::from_str::<crate::config::AndroidConfig>(&config_json) {
            if !config.database_path.is_empty() {
                android_paths.push(config.database_path.clone());
            }
            if !config.logs_path.is_empty() {
                android_paths.push(config.logs_path.clone());
            }
        }
    }
    
    for path_str in android_paths {
        let path = std::path::Path::new(&path_str);
        if path.exists() {
            warn!("Android config file still exists: {}", path_str);
            return Err("Android configuration files could not be removed".to_string());
        }
    }
    
    info!("Cleanup verification successful");
    Ok(())
}

/// Get cleanup status and remaining files
pub fn get_cleanup_status() -> String {
    let data_dir = get_android_data_dir();
    let data_path = std::path::Path::new(&data_dir);
    
    let mut status = serde_json::json!({
        "data_directory": data_dir,
        "data_directory_exists": data_path.exists(),
        "remaining_files": Vec::<String>::new(),
        "cleanup_required": false
    });
    
    if data_path.exists() {
        if let Ok(entries) = std::fs::read_dir(data_path) {
            let remaining_files: Vec<String> = entries
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.file_name().to_string_lossy().to_string())
                .collect();
            
            status["remaining_files"] = serde_json::Value::Array(
                remaining_files.iter().map(|f| serde_json::Value::String(f.clone())).collect()
            );
            status["cleanup_required"] = serde_json::Value::Bool(!remaining_files.is_empty());
        }
    }
    
    // Check Android-specific paths
    // Note: SSL certificates are intentionally preserved for security
    let mut android_paths: Vec<String> = Vec::new();
    
    // Try to load configuration to get paths
    let config_paths = get_config_file_paths();
    for config_path in config_paths {
        if let Ok(config_json) = std::fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str::<crate::config::AndroidConfig>(&config_json) {
                if !config.database_path.is_empty() {
                    android_paths.push(config.database_path.clone());
                }
                if !config.logs_path.is_empty() {
                    android_paths.push(config.logs_path.clone());
                }
            }
        }
    }
    
    // SSL certificates are NOT checked: "/data/data/com.purrmint.app/files/purrmint_cert.pem",
    // SSL certificates are NOT checked: "/data/data/com.purrmint.app/files/purrmint_key.pem",
    
    let mut android_status = Vec::new();
    for path_str in android_paths {
        let path = std::path::Path::new(&path_str);
        android_status.push(serde_json::json!({
            "path": path_str,
            "exists": path.exists(),
            "size": if path.exists() && path.is_file() {
                std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
            } else {
                0
            }
        }));
    }
    
    status["android_files"] = serde_json::Value::Array(android_status);
    
    status.to_string()
}

/// Get all paths that need to be cleaned up from configuration
fn get_paths_to_clean_from_config() -> Vec<String> {
    let mut paths_to_clean = Vec::new();
    
    // Try to load configuration to get actual paths
    let config_paths = get_config_file_paths();
    
    for config_path in config_paths {
        if let Ok(config_json) = std::fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str::<crate::config::AndroidConfig>(&config_json) {
                info!("Loaded configuration from: {}", config_path);
                
                // Add database paths
                if !config.database_path.is_empty() {
                    paths_to_clean.push(config.database_path.clone());
                    info!("Added database path: {}", config.database_path);
                    
                    // Also add SQLite temporary files
                    if let Some(db_dir) = std::path::Path::new(&config.database_path).parent() {
                        let db_name = std::path::Path::new(&config.database_path).file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("mint.db");
                        let db_stem = db_name.trim_end_matches(".db");
                        
                        let temp_files = vec![
                            format!("{}/{}-shm", db_dir.display(), db_stem),
                            format!("{}/{}-wal", db_dir.display(), db_stem),
                            format!("{}/{}-journal", db_dir.display(), db_stem),
                        ];
                        
                        paths_to_clean.extend_from_slice(&temp_files);
                        info!("Added SQLite temp files: {:?}", temp_files);
                    }
                }
                
                // Add logs path
                if !config.logs_path.is_empty() {
                    paths_to_clean.push(config.logs_path.clone());
                    info!("Added logs path: {}", config.logs_path);
                }
                
                // Add Tor-specific paths if Tor is enabled
                // Note: We do NOT delete Tor data as it contains user configuration
                // if config.tor_enabled.unwrap_or(false) {
                //     if let Some(tor_data_dir) = &config.tor_data_dir {
                //         if !tor_data_dir.is_empty() {
                //             paths_to_clean.push(tor_data_dir.clone());
                //             info!("Added Tor data path: {}", tor_data_dir);
                //         }
                //     }
                // }
                
                // SSL certificates are NOT removed for security reasons
                // Only remove them if they are explicitly marked as temporary
                // if config.enable_https.unwrap_or(false) {
                //     if let Some(ssl_cert_path) = &config.ssl_cert_path {
                //         if !ssl_cert_path.is_empty() {
                //             paths_to_clean.push(ssl_cert_path.clone());
                //         }
                //     }
                //     if let Some(ssl_key_path) = &config.ssl_key_path {
                //         if !ssl_key_path.is_empty() {
                //             paths_to_clean.push(ssl_key_path.clone());
                //         }
                //     }
                // }
                
                // Found and loaded config, break
                break;
            }
        }
    }
    
    // Note: We don't add hardcoded paths anymore - everything comes from config
    // If no config is found, only the config files themselves will be cleaned
    
    // Remove duplicates and sort
    paths_to_clean.sort();
    paths_to_clean.dedup();
    
    info!("Total paths to clean up: {:?}", paths_to_clean);
    paths_to_clean
}

/// Get possible configuration file paths dynamically
fn get_config_file_paths() -> Vec<String> {
    let mut config_paths = Vec::new();
    
    // Try to get from environment variable first
    if let Ok(data_dir) = std::env::var("ANDROID_DATA_DIR") {
        config_paths.push(format!("{}/android_config.json", data_dir));
    }
    
    // Try common Android app data directories
    let common_paths = vec![
        "/data/data/com.purrmint.app/files",
        "/data/data/com.purrmint.app",
        "/storage/emulated/0/Android/data/com.purrmint.app/files",
        "/storage/emulated/0/Android/data/com.purrmint.app",
    ];
    
    for base_path in common_paths {
        config_paths.push(format!("{}/android_config.json", base_path));
    }
    
    // Add relative path as last resort
    config_paths.push("android_config.json".to_string());
    
    config_paths
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nsec_to_npub() {
        let nsec = "5c0c523f52a5b6fad39ed2403092df8cebc36318b39383bca6c00808626fab3a";
        let npub = nostr_nsec_to_npub(nsec).unwrap();
        assert_eq!(npub, "02eec7245d6b7d2ccb30380bfbe2a3648cd7a942653f5aa340edcea1f283686619");
    }

    #[test]
    fn test_generate_default_android_config() {
        let config = generate_default_android_config_internal().unwrap();
        assert!(config.contains("port"));
        assert!(config.contains("host"));
    }

    #[test]
    fn test_config_roundtrip() {
        let config = generate_default_android_config_internal().unwrap();
        let parsed: AndroidConfig = serde_json::from_str(&config).unwrap();
        let serialized = serde_json::to_string(&parsed).unwrap();
        
        // Parse both JSON strings to compare the actual data, not formatting
        let original_parsed: serde_json::Value = serde_json::from_str(&config).unwrap();
        let roundtrip_parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(original_parsed, roundtrip_parsed);
    }
} 