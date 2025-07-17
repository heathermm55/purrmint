//! Tor service module for PurrMint
//! Provides onion network access and hidden service functionality using Arti

use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;
use anyhow::{Result, anyhow};
use arti_client::{TorClient, TorClientConfig};
use arti_client::config::{BridgeConfigBuilder, CfgPath, BoolOrAuto};
use arti_client::config::onion_service::OnionServiceConfigBuilder;
// Removed pt import as it's not available in current arti-client version
use tor_rtcompat::PreferredRuntime;
use tor_hsservice::{
    RunningOnionService, 
    RendRequest, handle_rend_requests
};
use tor_proto::stream::IncomingStreamRequest;
use tor_cell::relaycell::msg::Connected;
use tor_hsrproxy::config::{ProxyConfigBuilder, ProxyRule, ProxyPattern, ProxyAction, TargetAddr, Encapsulation};
use futures::StreamExt;
use tracing::{info, warn, error};

use crate::config::{TorConfig, TorStartupMode};

/// Tor service for managing hidden services and Tor network connections
pub struct TorService {
    client: Option<Arc<TorClient<PreferredRuntime>>>,
    running_services: Arc<Mutex<HashMap<String, Arc<RunningOnionService>>>>,
    config: TorClientConfig,
    tor_config: TorConfig,
}

impl TorService {
    /// Create a new Tor service instance with default configuration
    pub fn new() -> Result<Self> {
        let config = TorClientConfig::default();
        let tor_config = TorConfig::default();
        Ok(Self {
            client: None,
            running_services: Arc::new(Mutex::new(HashMap::new())),
            config,
            tor_config,
        })
    }

    /// Create a new Tor service instance with custom configuration
    pub fn with_config(tor_config: TorConfig) -> Result<Self> {
        let mut builder = TorClientConfig::builder();

        // Configure data directory
        if let Some(data_dir) = tor_config.get_data_dir() {
            builder.storage().state_dir(CfgPath::new(data_dir.clone().into()));
            builder.storage().cache_dir(CfgPath::new(data_dir.into()));
        }

        // Configure bridges (supports obfs4 and other pluggable transports)
        if tor_config.use_bridges && !tor_config.bridges.is_empty() {
            for bridge_line in &tor_config.bridges {
                let bridge: BridgeConfigBuilder = bridge_line.parse()
                    .map_err(|e| anyhow!("Invalid bridge line '{}': {}", bridge_line, e))?;
                builder.bridges().bridges().push(bridge);
            }
            builder.bridges().enabled(BoolOrAuto::Explicit(true));
            
            // Note: Transport configuration is handled differently in current arti-client
            // The transport configuration is now part of the bridge configuration itself
        }

        // Additional parameters can be configured as needed
        let config = builder.build()?;

        Ok(Self {
            client: None,
            running_services: Arc::new(Mutex::new(HashMap::new())),
            config,
            tor_config,
        })
    }

    /// Start the Tor client and bootstrap connection to the Tor network
    pub async fn start(&mut self) -> Result<()> {
        if !self.tor_config.is_enabled() {
            info!("Tor is disabled in configuration");
            return Ok(());
        }

        info!("Starting Tor client with mode: {:?}", self.tor_config.startup_mode);
        
        match self.tor_config.startup_mode {
            TorStartupMode::Disabled => {
                info!("Tor is disabled, skipping startup");
                return Ok(());
            }
            TorStartupMode::System => {
                info!("Using system Tor (not implemented yet)");
                return Err(anyhow!("System Tor mode not implemented"));
            }
            TorStartupMode::Embedded | TorStartupMode::Custom => {
        // Create and bootstrap the Tor client
        let client = TorClient::create_bootstrapped(self.config.clone()).await
            .map_err(|e| anyhow!("Failed to bootstrap Tor client: {}", e))?;
        
        self.client = Some(Arc::new(client));
        info!("Tor client started successfully");
            }
        }
        
        Ok(())
    }

    /// Stop the Tor service and all running hidden services
    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping Tor service...");
        
        // Stop all running hidden services
        let mut services = self.running_services.lock().await;
        for (nickname, service) in services.drain() {
            info!("Stopping hidden service: {}", nickname);
            drop(service); // This will terminate the service when dropped
        }
        
        // Clear the client
        self.client = None;
        info!("Tor service stopped");
        Ok(())
    }

    /// Get the status of the Tor service
    pub fn status(&self) -> TorServiceStatus {
        if self.client.is_some() {
            TorServiceStatus::Running
        } else {
            TorServiceStatus::Stopped
        }
    }

    /// Get the Tor configuration
    pub fn get_config(&self) -> &TorConfig {
        &self.tor_config
    }

    /// Check if hidden services are enabled
    pub fn hidden_services_enabled(&self) -> bool {
        self.tor_config.hidden_services_enabled()
    }

    /// Create a new hidden service with the given nickname
    pub async fn create_hidden_service(&self, nickname: &str) -> Result<HiddenServiceInfo> {
        if !self.hidden_services_enabled() {
            return Err(anyhow!("Hidden services are disabled in configuration"));
        }

        let client = self.client.as_ref()
            .ok_or_else(|| anyhow!("Tor client not started"))?;

        info!("Creating hidden service with nickname: {}", nickname);
        
        // Create the hidden service configuration
        let svc_config = OnionServiceConfigBuilder::default()
            .nickname(nickname.parse()?)
            .num_intro_points(self.tor_config.num_intro_points.try_into().unwrap_or(3))
            .build()?;

        // Create proxy configuration to forward port 80 to local mint service
        let mut proxy_config_builder = ProxyConfigBuilder::default();
        proxy_config_builder.proxy_ports().push(ProxyRule::new(
            ProxyPattern::one_port(80)?,
            ProxyAction::Forward(
                Encapsulation::Simple,
                TargetAddr::Inet("127.0.0.1:3338".parse()?)
            )
        ));
        let proxy_config = proxy_config_builder.build()?;

        // Launch the hidden service
        let (service, request_stream) = client.launch_onion_service(svc_config)?;
        
        // Get the onion address
        let onion_address = service.onion_address()
            .ok_or_else(|| anyhow!("Failed to get onion address"))?;
        
        // Store the running service
        let mut services = self.running_services.lock().await;
        services.insert(nickname.to_string(), service);
        
        info!("Hidden service created successfully: {}", onion_address);
        info!("Port mapping: 80 -> 127.0.0.1:3338");
        
        // Create reverse proxy to handle port forwarding
        let proxy = tor_hsrproxy::OnionServiceReverseProxy::new(proxy_config);
        
        // Handle incoming requests with proxy
        let nickname_clone = nickname.to_string();
        let runtime = tor_rtcompat::PreferredRuntime::current()?;
        let nickname_parsed = nickname_clone.parse()?;
        tokio::spawn(async move {
            if let Err(e) = proxy.handle_requests(runtime, nickname_parsed, request_stream).await {
                error!("Error handling hidden service requests: {}", e);
            }
        });
        
        Ok(HiddenServiceInfo {
            nickname: nickname.to_string(),
            onion_address: onion_address.to_string(),
            status: HiddenServiceStatus::Starting,
        })
    }

    /// Create a hidden service using mint pubkey as nickname
    /// This ensures the onion address is tied to the mint's identity
    pub async fn create_hidden_service_for_mint(&self, mint_pubkey: &str) -> Result<HiddenServiceInfo> {
        // Clean the pubkey to make it a valid nickname
        // Remove any non-alphanumeric characters and limit length
        let nickname = mint_pubkey
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
            .take(50) // Limit length for nickname
            .collect::<String>();
        
        if nickname.is_empty() {
            return Err(anyhow!("Invalid mint pubkey for nickname"));
        }
        
        info!("Creating hidden service for mint with pubkey: {}", mint_pubkey);
        info!("Using nickname: {}", nickname);
        
        self.create_hidden_service(&nickname).await
    }

    /// Get hidden service info for a mint pubkey
    pub async fn get_hidden_service_info_for_mint(&self, mint_pubkey: &str) -> Result<Option<HiddenServiceInfo>> {
        let nickname = mint_pubkey
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
            .take(50)
            .collect::<String>();
        
        if nickname.is_empty() {
            return Ok(None);
        }
        
        self.get_hidden_service_info(&nickname).await
    }

    /// Get information about a running hidden service
    pub async fn get_hidden_service_info(&self, nickname: &str) -> Result<Option<HiddenServiceInfo>> {
        let services = self.running_services.lock().await;
        
        if let Some(service) = services.get(nickname) {
            let onion_address = service.onion_address()
                .map(|addr| addr.to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            
            let status = if service.status().state().is_fully_reachable() {
                HiddenServiceStatus::Running
            } else {
                HiddenServiceStatus::Starting
            };
            
            Ok(Some(HiddenServiceInfo {
                nickname: nickname.to_string(),
                onion_address,
                status,
            }))
        } else {
            Ok(None)
        }
    }

    /// List all running hidden services
    pub async fn list_hidden_services(&self) -> Result<Vec<HiddenServiceInfo>> {
        let services = self.running_services.lock().await;
        let mut result = Vec::new();
        
        for (nickname, service) in services.iter() {
            let onion_address = service.onion_address()
                .map(|addr| addr.to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            
            let status = if service.status().state().is_fully_reachable() {
                HiddenServiceStatus::Running
            } else {
                HiddenServiceStatus::Starting
            };
            
            result.push(HiddenServiceInfo {
                nickname: nickname.clone(),
                onion_address,
                status,
            });
        }
        
        Ok(result)
    }

    /// Stop a specific hidden service
    pub async fn stop_hidden_service(&self, nickname: &str) -> Result<bool> {
        let mut services = self.running_services.lock().await;
        
        if services.remove(nickname).is_some() {
            info!("Stopped hidden service: {}", nickname);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Make an HTTP request through the Tor network
    pub async fn make_tor_request(&self, url: &str) -> Result<String> {
        if !self.tor_config.is_enabled() {
            return Err(anyhow!("Tor is disabled in configuration"));
        }

        let _client = self.client.as_ref()
            .ok_or_else(|| anyhow!("Tor client not started"))?;

        info!("Making Tor request to: {}", url);
        
        // Parse the URL and create a request
        let url_parsed = url.parse::<http::Uri>()
            .map_err(|e| anyhow!("Invalid URL: {}", e))?;
        
        // Create a simple HTTP request
        let _request = http::Request::builder()
            .method("GET")
            .uri(url_parsed.clone())
            .body(())
            .map_err(|e| anyhow!("Failed to create request: {}", e))?;
        
        // For now, return a mock response since we need to implement proper HTTP client
        // In a real implementation, you would use a proper HTTP client that works with Tor
        Ok(format!("Mock response for Tor request to: {}", url))
    }

    /// Test the Tor connection
    pub async fn test_connection(&self) -> Result<bool> {
        if !self.tor_config.is_enabled() {
            info!("Tor is disabled, connection test skipped");
            return Ok(false);
        }

        let client = self.client.as_ref()
            .ok_or_else(|| anyhow!("Tor client not started"))?;

        info!("Testing Tor connection...");
        
        // Try to resolve a simple hostname to test the connection
        match client.resolve("check.torproject.org").await {
            Ok(_) => {
                info!("Tor connection test successful");
                Ok(true)
            }
            Err(e) => {
                warn!("Tor connection test failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Handle incoming requests for a hidden service
    pub async fn handle_hidden_service_requests(
        &self,
        nickname: &str,
        request_stream: impl StreamExt<Item = RendRequest> + Send + Sync + 'static,
    ) -> Result<()> {
        Self::handle_hidden_service_requests_static(nickname, request_stream).await
    }

    /// Static method to handle hidden service requests (for use in spawned tasks)
    async fn handle_hidden_service_requests_static(
        nickname: &str,
        request_stream: impl StreamExt<Item = RendRequest> + Send + Sync + 'static,
    ) -> Result<()> {
        info!("Starting to handle requests for hidden service: {}", nickname);
        
        let stream_requests = handle_rend_requests(request_stream);
        tokio::pin!(stream_requests);
        
        while let Some(stream_request) = stream_requests.next().await {
            match stream_request.request() {
                IncomingStreamRequest::Begin(begin) => {
                    info!("Received connection request on port {} for service {}", 
                          begin.port(), nickname);
                    
                    // Accept the connection
                    let _onion_service_stream = stream_request.accept(Connected::new_empty()).await?;
                    
                    // In a real implementation, you would handle the stream here
                    // For now, we just log the connection
                    info!("Accepted connection for service: {}", nickname);
                }
                _ => {
                    // Reject other types of requests
                    stream_request.shutdown_circuit()?;
                }
            }
        }
        
        info!("Finished handling requests for hidden service: {}", nickname);
        Ok(())
    }
}

/// Status of the Tor service
#[derive(Debug, Clone, PartialEq)]
pub enum TorServiceStatus {
    Running,
    Stopped,
}

/// Status of a hidden service
#[derive(Debug, Clone, PartialEq)]
pub enum HiddenServiceStatus {
    Starting,
    Running,
    Stopped,
}

/// Information about a hidden service
#[derive(Debug, Clone)]
pub struct HiddenServiceInfo {
    pub nickname: String,
    pub onion_address: String,
    pub status: HiddenServiceStatus,
}

impl std::fmt::Display for TorServiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TorServiceStatus::Running => write!(f, "Running"),
            TorServiceStatus::Stopped => write!(f, "Stopped"),
        }
    }
}

impl std::fmt::Display for HiddenServiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HiddenServiceStatus::Starting => write!(f, "Starting"),
            HiddenServiceStatus::Running => write!(f, "Running"),
            HiddenServiceStatus::Stopped => write!(f, "Stopped"),
        }
    }
} 