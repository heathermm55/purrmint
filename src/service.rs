//! Service management for the mint

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;
use serde_json::Value;
use crate::mintd_service::MintdService;

/// Service configuration
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub database_path: String,
    pub logs_path: String,
    pub port: u16,
    pub mode: ServiceMode,
}

/// Service mode
#[derive(Debug, Clone)]
pub enum ServiceMode {
    MintOnly,
    MintAndNip74,
}

/// Global service state
pub struct GlobalState {
    pub mint_service: Option<Arc<Mutex<MintdService>>>,
    pub config: Option<ServiceConfig>,
}

impl GlobalState {
    pub fn new() -> Self {
        Self {
            mint_service: None,
            config: None,
        }
    }
}

/// Main service coordinator
pub struct MintService {
    state: Arc<Mutex<GlobalState>>,
    work_dir: PathBuf,
    is_running: bool,
}

impl MintService {
    /// Create a new service instance
    pub fn new(work_dir: PathBuf) -> Self {
        Self {
            state: Arc::new(Mutex::new(GlobalState::new())),
            work_dir,
            is_running: false,
        }
    }

    /// Create a new service instance with nsec
    pub fn new_with_nsec(work_dir: PathBuf, nsec: String) -> Self {
        info!("Creating MintService with nsec: {}...", &nsec[..8]);
        let service = Self::new(work_dir.clone());
        
        // Initialize mint service with nsec
        let state = service.state.clone();
        tokio::spawn(async move {
            let mintd = MintdService::new_with_nsec(work_dir, nsec);
            let mut state = state.lock().await;
            state.mint_service = Some(Arc::new(Mutex::new(mintd)));
        });
        
        service
    }

    /// Start the service
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.is_running {
            return Ok(());
        }

        let state = self.state.lock().await;
        if let Some(mint_service) = &state.mint_service {
            let mut mintd = mint_service.lock().await;
            mintd.start().await?;
            info!("Mint service started successfully");
        }
        
        self.is_running = true;
        Ok(())
    }

    /// Stop the service
    pub async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.is_running {
            return Ok(());
        }

        let state = self.state.lock().await;
        if let Some(mint_service) = &state.mint_service {
            let mut mintd = mint_service.lock().await;
            mintd.stop().await?;
            info!("Mint service stopped");
        }
        
        self.is_running = false;
        Ok(())
    }

    /// Get service status
    pub async fn get_status(&self) -> Value {
        let state = self.state.lock().await;
        if let Some(mint_service) = &state.mint_service {
            let mintd = mint_service.lock().await;
            mintd.get_status()
        } else {
            serde_json::json!({
                "running": false,
                "server_url": "",
                "work_dir": self.work_dir.to_string_lossy(),
            })
        }
    }

    /// Check if service is running
    pub fn is_running(&self) -> bool {
        self.is_running
    }
} 