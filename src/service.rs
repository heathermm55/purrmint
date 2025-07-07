//! MintService and related traits/structs

use async_trait::async_trait;
use nostr::prelude::*;
use nostr_sdk::{Client, RelayPoolNotification};
use std::sync::Arc;
use thiserror::Error;
use tokio::task::JoinHandle;
use tracing::error;
use crate::{OperationRequest, OperationResult, Nip74Result, Nip74Error};
use crate::nip74_service::build_mint_info_event;
use crate::mintd_service::MintdService;
use cdk::nuts::nut06::MintInfo as cdkMintInfo;
use crate::config::{LightningConfig, ServiceMode};
use nostr::signer::NostrSigner;
use nostr::{Filter, Kind, RelayUrl};



/// Errors raised by the [`MintService`].
#[derive(Debug, Error)]
pub enum ServiceError {
    /// Underlying NIP-74 helper error.
    #[error(transparent)]
    Nip74(#[from] Nip74Error),
    /// Nostr SDK error.
    #[error(transparent)]
    Nostr(#[from] nostr_sdk::client::Error),
    /// Tokio join error.
    #[error("join error: {0}")]
    Join(#[from] tokio::task::JoinError),
    /// Mintd integration error.
    #[error("mintd error: {0}")]
    Mintd(#[from] Box<dyn std::error::Error + Send + Sync>),
    /// Invalid service mode.
    #[error("invalid service mode")]
    InvalidMode,
}

/// Allow `?` on `SignerError`
impl From<nostr::signer::SignerError> for ServiceError {
    fn from(e: nostr::signer::SignerError) -> Self {
        ServiceError::Nostr(nostr_sdk::client::Error::Signer(e))
    }
}

/// Abstraction over local keys or remote signer.
pub type DynSigner = Arc<dyn NostrSigner>;

/// Request handler trait – application implements custom business logic.
#[async_trait]
pub trait RequestHandler: Send + Sync + 'static {
    /// Handle an OperationRequest and return the OperationResult.
    async fn handle(&self, req: OperationRequest) -> Nip74Result<OperationResult>;
}

/// Mint service – manages relay connections and request processing.
pub struct MintService {
    mode: ServiceMode,
    mint_info: cdkMintInfo,
    lightning_config: LightningConfig,
    relays: Vec<RelayUrl>,
    mintd: Option<MintdService>,
    mintd_port: u16,
    client: Option<Client>,
    _nip74_task: Option<JoinHandle<()>>,
    signer: Option<Arc<dyn NostrSigner>>,
    handler: Option<Arc<dyn RequestHandler + Send + Sync>>,
}

impl MintService {
    /// Create a new service instance.
    pub async fn new<T>(
        mode: ServiceMode,
        mint_info: cdkMintInfo,
        lightning_config: LightningConfig,
        relays: T,
        config_dir: std::path::PathBuf,
        mintd_port: u16,
    ) -> Result<Self, ServiceError>
    where
        T: IntoIterator<Item = RelayUrl>,
    {
        let (signer, handler, client) = match mode {
            ServiceMode::MintdOnly => (None, None, None),
            ServiceMode::Nip74Only | ServiceMode::MintdAndNip74 => {
                // For NIP-74 modes, we need a signer and handler
                // These will be set later via set_signer and set_handler
                (None, None, None)
            }
        };

        let mintd = match mode {
            ServiceMode::MintdOnly | ServiceMode::MintdAndNip74 => {
                // Try to read mnemonic from config file using the new config management
                let config_file = config_dir.join("mintd.toml");
                tracing::info!("MintService::new: checking config file at {:?}", config_file);
                
                let mnemonic = if config_file.exists() {
                    match crate::config::Settings::load_from_file(&config_file) {
                        Ok(settings) => {
                            tracing::info!("MintService::new: config file loaded successfully");
                            if let Some(mnemonic) = settings.info.mnemonic {
                                tracing::info!("MintService::new: found mnemonic in config: {}...", &mnemonic[..mnemonic.len().min(20)]);
                                mnemonic
                            } else {
                                tracing::warn!("MintService::new: no mnemonic in config, using default");
                                "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string()
                            }
                        },
                        Err(e) => {
                            tracing::warn!("MintService::new: failed to parse config file: {:?}, using default mnemonic", e);
                            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string()
                        }
                    }
                } else {
                    tracing::warn!("MintService::new: config file not found, using default mnemonic");
                    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string()
                };
                
                tracing::info!("MintService::new: creating MintdService with mnemonic: {}...", &mnemonic[..mnemonic.len().min(20)]);
                Some(MintdService::new_with_mnemonic(config_dir.clone(), mnemonic))
            }
            ServiceMode::Nip74Only => None,
        };

        Ok(Self {
            mode,
            signer,
            mint_info,
            lightning_config,
            relays: relays.into_iter().collect(),
            handler,
            client,
            _nip74_task: None,
            mintd,
            mintd_port,
        })
    }

    /// Set the Nostr signer (required for NIP-74 modes)
    pub fn set_signer(&mut self, signer: DynSigner) -> Result<(), ServiceError> {
        match self.mode {
            ServiceMode::MintdOnly => Err(ServiceError::InvalidMode),
            ServiceMode::Nip74Only | ServiceMode::MintdAndNip74 => {
                self.signer = Some(signer);
                Ok(())
            }
        }
    }

    /// Set the request handler (required for NIP-74 modes)
    pub fn set_handler(&mut self, handler: Arc<dyn RequestHandler>) -> Result<(), ServiceError> {
        match self.mode {
            ServiceMode::MintdOnly => Err(ServiceError::InvalidMode),
            ServiceMode::Nip74Only | ServiceMode::MintdAndNip74 => {
                self.handler = Some(handler);
                Ok(())
            }
        }
    }

    /// Auto-configure the service with appropriate handlers based on mode
    pub fn auto_configure(&mut self) -> Result<(), ServiceError> {
        match self.mode {
            ServiceMode::MintdOnly => {
                // No handler needed for mintd-only mode
                Ok(())
            }
            ServiceMode::Nip74Only | ServiceMode::MintdAndNip74 => {
                // Create default request handler that proxies to mintd
                let handler = Arc::new(crate::nip74_service::DefaultRequestHandler::new(self.mintd_port));
                self.set_handler(handler)?;
                Ok(())
            }
        }
    }

    /// Start the service based on the configured mode
    pub async fn start(&mut self) -> Result<(), ServiceError> {
        match self.mode {
            ServiceMode::MintdOnly => self.start_mintd_only().await,
            ServiceMode::Nip74Only => self.start_nip74_only().await,
            ServiceMode::MintdAndNip74 => self.start_mintd_and_nip74().await,
        }
    }

    /// Start mintd-only mode
    async fn start_mintd_only(&mut self) -> Result<(), ServiceError> {
        if let Some(mintd) = &mut self.mintd {
            mintd.start().await.map_err(|e| ServiceError::Mintd(e.into()))?;
            tracing::info!("Mintd service started on port {}", self.mintd_port);
        }
        Ok(())
    }

    /// Start NIP-74-only mode
    async fn start_nip74_only(&mut self) -> Result<(), ServiceError> {
        let signer = self.signer.as_ref()
            .ok_or(ServiceError::InvalidMode)?;
        let handler = self.handler.as_ref()
            .ok_or(ServiceError::InvalidMode)?;

        let client = Client::new(signer.clone());
        
        // Connect to relays
        for url in &self.relays {
            client.add_relay(url.clone()).await?;
        }
        client.connect().await;
        client.wait_for_connection(std::time::Duration::from_secs(5)).await;

        // Broadcast MintInfo event
        let identifier = self.mint_info.name.clone().unwrap_or_else(|| "mint".to_owned());
        let event = build_mint_info_event(
            &self.mint_info,
            signer,
            &identifier,
            &self.relays,
            "running",
            None,
        ).await?;
        client.send_event(&event).await?;
        tracing::info!(id = %event.id, "MintInfo event sent");

        // Subscribe for OperationRequest events
        let filter = Filter::new().kind(Kind::from(27401u16));
        let _ = client.subscribe(filter, None).await?;

        // Spawn background NIP-74 listener
        let signer = signer.clone();
        let handler = handler.clone();
        let client_clone = client.clone();
        let mut notifications = client_clone.notifications();
        self._nip74_task = Some(tokio::spawn(async move {
            while let Ok(notif) = notifications.recv().await {
                if let RelayPoolNotification::Event { event, .. } = notif {
                    if event.kind != Kind::from(27401u16) { continue; }
                    tracing::info!(id=%event.id, from=%event.pubkey, "Received 27401 OperationRequest event");
                    
                    // Use NIP-44 decryption
                    match signer.nip44_decrypt(&event.pubkey, &event.content).await {
                        Ok(plaintext) => {
                            match serde_json::from_str::<OperationRequest>(&plaintext) {
                                Ok(req) => {
                                    tracing::info!(method=?req.method, req_id=%req.request_id, "Parsed OperationRequest");
                                    // Process
                                    let res = handler.handle(req).await;
                                    match res {
                                        Ok(op_res) => {
                                            let event_result = op_res
                                                .to_event_with_signer(
                                                    &signer,
                                                    &signer.get_public_key().await.unwrap(),
                                                    &event.pubkey,
                                                    &event.id,
                                                    None,
                                                )
                                                .await
                                                .map_err(|e| e.to_string());
                                                
                                            match event_result {
                                                Ok(ev) => {
                                                    match client_clone.send_event(&ev).await {
                                                        Ok(out) => tracing::info!(sent=out.success.len(), failed=?out.failed, "OperationResult 27402 sent"),
                                                        Err(e) => tracing::error!(error = %e, "failed to send 27402"),
                                                    };
                                                }
                                                Err(e) => {
                                                    tracing::error!(error = %e, "failed to create 27402 event");
                                                }
                                            }
                                        }
                                        Err(e) => error!(?e, "handler error"),
                                    }
                                }
                                Err(e) => error!(?e, "request parse error"),
                            }
                        }
                        Err(e) => error!(?e, "decrypt error"),
                    }
                }
            }
        }));

        self.client = Some(client);
        tracing::info!("NIP-74 service started");
        Ok(())
    }

    /// Start mintd + NIP-74 mode
    async fn start_mintd_and_nip74(&mut self) -> Result<(), ServiceError> {
        // Start mintd first
        if let Some(mintd) = &mut self.mintd {
            mintd.start().await.map_err(|e| ServiceError::Mintd(e.into()))?;
            tracing::info!("Mintd service started on port {}", self.mintd_port);
        }

        // Then start NIP-74 service
        self.start_nip74_only().await?;
        
        tracing::info!("Mintd + NIP-74 service started");
        Ok(())
    }

    /// Stop the service
    pub async fn stop(&mut self) -> Result<(), ServiceError> {
        // Stop NIP-74 task
        if let Some(task) = self._nip74_task.take() {
            task.abort();
            let _ = task.await;
        }

        // Disconnect Nostr client
        if let Some(client) = &self.client {
            client.disconnect().await;
        }

        // Stop mintd
        if let Some(mintd) = &mut self.mintd {
            mintd.stop().await.map_err(|e| ServiceError::Mintd(e.into()))?;
        }

        tracing::info!("Service stopped");
        Ok(())
    }

    /// Get service status
    pub fn get_status(&self) -> serde_json::Value {
        let mintd_running = self.mintd.as_ref().map(|m| m.is_running()).unwrap_or(false);
        let nip74_running = self._nip74_task.is_some();

        serde_json::json!({
            "mode": match self.mode {
                ServiceMode::MintdOnly => "mintd_only",
                ServiceMode::Nip74Only => "nip74_only", 
                ServiceMode::MintdAndNip74 => "mintd_and_nip74",
            },
            "mintd_running": mintd_running,
            "nip74_running": nip74_running,
            "mintd_port": self.mintd_port,
            "relays": self.relays,
        })
    }

    /// Get access URLs
    pub fn get_access_urls(&self) -> serde_json::Value {
        let mut urls = serde_json::Map::new();
        
        // Add mintd HTTP API URL if running
        if self.mode != ServiceMode::Nip74Only {
            // Use localhost instead of hardcoded IP
            urls.insert("http_api".to_string(), 
                serde_json::Value::String(format!("http://localhost:{}", self.mintd_port)));
        }

        // Add NIP-74 info if running
        if self.mode != ServiceMode::MintdOnly {
            urls.insert("nip74_relays".to_string(), 
                serde_json::Value::Array(self.relays.iter().map(|r| serde_json::Value::String(r.to_string())).collect()));
        }

        serde_json::Value::Object(urls)
    }

    /// Proxy request to mintd (for mintd modes)
    pub async fn proxy_request(&self, endpoint: &str, payload: serde_json::Value) -> Result<serde_json::Value, ServiceError> {
        // For now, return a mock response since we're using integrated mintd service
        // In the future, this could make direct calls to the mint instance
        Ok(serde_json::json!({
            "status": "success",
            "endpoint": endpoint,
            "message": "Integrated mintd service - direct API calls not yet implemented"
        }))
    }
} 