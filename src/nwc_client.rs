use anyhow::{anyhow, Result};
use nostr::{
    Event, EventBuilder, Keys, Kind, Tag, NostrSigner, TagStandard,
};
use nostr_relay_pool::{
    RelayPool, RelayPoolNotification, RelayOptions,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};
use std::str::FromStr;

/// NWC Error codes as defined in NIP-47
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NWCErrorCode {
    RateLimited,
    NotImplemented,
    InsufficientBalance,
    QuotaExceeded,
    Restricted,
    Unauthorized,
    Internal,
    Other,
    PaymentFailed,
    NotFound,
}

impl std::fmt::Display for NWCErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NWCErrorCode::RateLimited => write!(f, "RATE_LIMITED"),
            NWCErrorCode::NotImplemented => write!(f, "NOT_IMPLEMENTED"),
            NWCErrorCode::InsufficientBalance => write!(f, "INSUFFICIENT_BALANCE"),
            NWCErrorCode::QuotaExceeded => write!(f, "QUOTA_EXCEEDED"),
            NWCErrorCode::Restricted => write!(f, "RESTRICTED"),
            NWCErrorCode::Unauthorized => write!(f, "UNAUTHORIZED"),
            NWCErrorCode::Internal => write!(f, "INTERNAL"),
            NWCErrorCode::Other => write!(f, "OTHER"),
            NWCErrorCode::PaymentFailed => write!(f, "PAYMENT_FAILED"),
            NWCErrorCode::NotFound => write!(f, "NOT_FOUND"),
        }
    }
}

/// NWC Error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NWCError {
    pub code: NWCErrorCode,
    pub message: String,
}

/// NWC Request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NWCRequest {
    pub method: String,
    pub params: serde_json::Value,
}

/// NWC Response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NWCResponse {
    pub result_type: String,
    pub error: Option<NWCError>,
    pub result: Option<serde_json::Value>,
}

/// NWC Notification structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NWCNotification {
    pub notification_type: String,
    pub notification: serde_json::Value,
}

/// NWC Connection URI parser
#[derive(Debug, Clone)]
pub struct NWCConnectionUri {
    pub wallet_service_pubkey: String,
    pub relay_url: String,
    pub secret: String,
    pub lud16: Option<String>,
}

impl NWCConnectionUri {
    /// Parse NWC connection URI
    pub fn from_uri(uri: &str) -> Result<Self> {
        if !uri.starts_with("nostr+walletconnect://") {
            return Err(anyhow!("Invalid NWC URI format"));
        }

        let uri = uri.replace("nostr+walletconnect://", "");
        let parts: Vec<&str> = uri.split('?').collect();
        
        if parts.len() != 2 {
            return Err(anyhow!("Invalid NWC URI format"));
        }

        let wallet_service_pubkey = parts[0].to_string();
        let query_params = parts[1];

        let mut relay_url = String::new();
        let mut secret = String::new();
        let mut lud16 = None;

        for param in query_params.split('&') {
            let key_value: Vec<&str> = param.split('=').collect();
            if key_value.len() == 2 {
                match key_value[0] {
                    "relay" => relay_url = urlencoding::decode(key_value[1])?.to_string(),
                    "secret" => secret = key_value[1].to_string(),
                    "lud16" => lud16 = Some(urlencoding::decode(key_value[1])?.to_string()),
                    _ => {}
                }
            }
        }

        if relay_url.is_empty() || secret.is_empty() {
            return Err(anyhow!("Missing required parameters in NWC URI"));
        }

        Ok(Self {
            wallet_service_pubkey,
            relay_url,
            secret,
            lud16,
        })
    }
}

/// NWC Client - connects to external Lightning wallet service via Nostr
#[derive(Clone)]
pub struct NWCClient {
    client_keys: Keys,
    wallet_service_pubkey: String,
    relay_url: String,
    relay_pool: Arc<RelayPool>,
    is_connected: bool,
    pending_requests: Arc<RwLock<HashMap<String, tokio::sync::oneshot::Sender<NWCResponse>>>>,
}

impl NWCClient {
    /// Create new NWC client from connection URI
    pub async fn from_uri(connection_uri: &str) -> Result<Self> {
        let uri = NWCConnectionUri::from_uri(connection_uri)?;
        
        // Create client keys from secret
        let client_keys = Keys::parse(&uri.secret)?;
        
        // Create relay pool
        let relay_pool = RelayPool::new();

        Ok(Self {
            client_keys,
            wallet_service_pubkey: uri.wallet_service_pubkey,
            relay_url: uri.relay_url,
            relay_pool: Arc::new(relay_pool),
            is_connected: false,
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Connect to the wallet service
    pub async fn connect(&mut self) -> Result<()> {
        info!("Connecting to NWC wallet service...");
        
        // Add relay to pool
        let relay_url = nostr::RelayUrl::from_str(&self.relay_url)?;
        let relay_opts = RelayOptions::new()
            .write(true)
            .read(true);
        
        self.relay_pool.add_relay(relay_url, relay_opts).await?;
        
        // Connect to relay
        self.relay_pool.connect().await;
        
        // Start listening for responses
        self.start_listening().await?;
        
        // Get wallet service info to verify connection
        self.get_info().await?;
        
        self.is_connected = true;
        info!("Connected to NWC wallet service");
        Ok(())
    }

    /// Disconnect from the wallet service
    pub async fn disconnect(&mut self) -> Result<()> {
        if !self.is_connected {
            return Ok(());
        }

        info!("Disconnecting from NWC wallet service...");
        self.relay_pool.shutdown().await;
        self.is_connected = false;
        info!("Disconnected from NWC wallet service");
        Ok(())
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.is_connected
    }

    /// Start listening for responses from wallet service
    async fn start_listening(&self) -> Result<()> {
        let relay_pool = self.relay_pool.clone();
        let pending_requests = self.pending_requests.clone();
        let client_keys = self.client_keys.clone();

        tokio::spawn(async move {
            let mut notifications = relay_pool.notifications();
            
            while let Ok(notification) = notifications.recv().await {
                match notification {
                    RelayPoolNotification::Event { event, relay_url: _, subscription_id: _ } => {
                        if event.kind == Kind::Custom(23195) {
                            // Handle NWC response
                            if let Err(e) = Self::handle_nwc_response(
                                &event,
                                &pending_requests,
                                &client_keys,
                            ).await {
                                error!("Failed to handle NWC response: {}", e);
                            }
                        } else if event.kind == Kind::Custom(23196) {
                            // Handle NWC notification
                            if let Err(e) = Self::handle_nwc_notification(&event, &client_keys).await {
                                error!("Failed to handle NWC notification: {}", e);
                            }
                        }
                    }
                    RelayPoolNotification::Shutdown => {
                        info!("Relay pool shutdown");
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }

    /// Handle NWC response (kind 23195)
    async fn handle_nwc_response(
        event: &Event,
        pending_requests: &Arc<RwLock<HashMap<String, tokio::sync::oneshot::Sender<NWCResponse>>>>,
        client_keys: &Keys,
    ) -> Result<()> {
        // Decrypt the content using NIP-04
        let decrypted_content = client_keys.nip04_decrypt(
            &event.pubkey,
            &event.content,
        ).await?;

        let response: NWCResponse = serde_json::from_str(&decrypted_content)?;
        debug!("Received NWC response: {:?}", response);

        // Find the corresponding request and send response
        if let Some(event_id) = event.tags.iter().find_map(|tag| {
            if let Some(TagStandard::Event { event_id, .. }) = tag.as_standardized() {
                Some(event_id)
            } else {
                None
            }
        }) {
            let mut requests = pending_requests.write().await;
            if let Some(sender) = requests.remove(&event_id.to_string()) {
                let _ = sender.send(response);
            }
        }

        Ok(())
    }

    /// Handle NWC notification (kind 23196)
    async fn handle_nwc_notification(
        event: &Event,
        client_keys: &Keys,
    ) -> Result<()> {
        // Decrypt the content using NIP-04
        let decrypted_content = client_keys.nip04_decrypt(
            &event.pubkey,
            &event.content,
        ).await?;

        let notification: NWCNotification = serde_json::from_str(&decrypted_content)?;
        info!("Received NWC notification: {:?}", notification);

        Ok(())
    }

    /// Send request to wallet service
    async fn send_request(&self, request: NWCRequest) -> Result<NWCResponse> {
        let request_json = serde_json::to_string(&request)?;
        
        // Create wallet service public key
        let wallet_service_pubkey = nostr::PublicKey::from_str(&self.wallet_service_pubkey)?;
        
        // Encrypt the request using NIP-04
        let encrypted_content = self.client_keys.nip04_encrypt(
            &wallet_service_pubkey,
            &request_json,
        ).await?;

        // Create request event
        let tags = vec![
            Tag::public_key(wallet_service_pubkey),
        ];

        let event = EventBuilder::new(
            Kind::Custom(23194),
            encrypted_content,
        )
        .tags(tags)
        .sign_with_keys(&self.client_keys)?;

        // Create response channel
        let (sender, receiver) = tokio::sync::oneshot::channel();
        {
            let mut requests = self.pending_requests.write().await;
            requests.insert(event.id.to_string(), sender);
        }

        // Send the event to relay
        let relay_url = nostr::RelayUrl::from_str(&self.relay_url)?;
        self.relay_pool.send_event_to([relay_url], &event).await?;

        // Wait for response with timeout
        match tokio::time::timeout(std::time::Duration::from_secs(30), receiver).await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(_)) => Err(anyhow!("Failed to receive response")),
            Err(_) => {
                // Remove from pending requests
                let mut requests = self.pending_requests.write().await;
                requests.remove(&event.id.to_string());
                Err(anyhow!("Request timeout"))
            }
        }
    }

    /// Get wallet service info
    pub async fn get_info(&self) -> Result<serde_json::Value> {
        let request = NWCRequest {
            method: "get_info".to_string(),
            params: serde_json::json!({}),
        };

        let response = self.send_request(request).await?;
        
        match response.error {
            Some(error) => Err(anyhow!("NWC error: {} - {}", error.code, error.message)),
            None => {
                if let Some(result) = response.result {
                    Ok(result)
                } else {
                    Err(anyhow!("No result in NWC response"))
                }
            }
        }
    }

    /// Pay a Lightning invoice
    pub async fn pay_invoice(&self, invoice: &str) -> Result<String> {
        let request = NWCRequest {
            method: "pay_invoice".to_string(),
            params: serde_json::json!({
                "invoice": invoice,
            }),
        };

        let response = self.send_request(request).await?;
        
        match response.error {
            Some(error) => Err(anyhow!("NWC error: {} - {}", error.code, error.message)),
            None => {
                if let Some(result) = response.result {
                    let preimage = result["preimage"]
                        .as_str()
                        .ok_or_else(|| anyhow!("Invalid preimage in NWC response"))?;
                    
                    Ok(preimage.to_string())
                } else {
                    Err(anyhow!("No result in NWC response"))
                }
            }
        }
    }

    /// Create a Lightning invoice
    pub async fn make_invoice(
        &self,
        amount_msat: u64,
        description: Option<String>,
        expiry: Option<u64>,
    ) -> Result<serde_json::Value> {
        let request = NWCRequest {
            method: "make_invoice".to_string(),
            params: serde_json::json!({
                "amount": amount_msat,
                "description": description.unwrap_or_else(|| "Cashu Mint Invoice".to_string()),
                "expiry": expiry.unwrap_or(3600),
            }),
        };

        let response = self.send_request(request).await?;
        
        match response.error {
            Some(error) => Err(anyhow!("NWC error: {} - {}", error.code, error.message)),
            None => {
                if let Some(result) = response.result {
                    Ok(result)
                } else {
                    Err(anyhow!("No result in NWC response"))
                }
            }
        }
    }

    /// Get balance
    pub async fn get_balance(&self) -> Result<u64> {
        let request = NWCRequest {
            method: "get_balance".to_string(),
            params: serde_json::json!({}),
        };

        let response = self.send_request(request).await?;
        
        match response.error {
            Some(error) => Err(anyhow!("NWC error: {} - {}", error.code, error.message)),
            None => {
                if let Some(result) = response.result {
                    let balance = result["balance"]
                        .as_u64()
                        .ok_or_else(|| anyhow!("Invalid balance in NWC response"))?;
                    
                    Ok(balance)
                } else {
                    Err(anyhow!("No result in NWC response"))
                }
            }
        }
    }

    /// Lookup invoice by payment hash
    pub async fn lookup_invoice(&self, payment_hash: &str) -> Result<Option<serde_json::Value>> {
        let request = NWCRequest {
            method: "lookup_invoice".to_string(),
            params: serde_json::json!({
                "payment_hash": payment_hash,
            }),
        };

        let response = self.send_request(request).await?;
        
        match response.error {
            Some(error) => {
                if error.code.to_string() == "NOT_FOUND" {
                    Ok(None)
                } else {
                    Err(anyhow!("NWC error: {} - {}", error.code, error.message))
                }
            }
            None => {
                if let Some(result) = response.result {
                    Ok(Some(result))
                } else {
                    Ok(None)
                }
            }
        }
    }

    /// List transactions
    pub async fn list_transactions(&self) -> Result<serde_json::Value> {
        let request = NWCRequest {
            method: "list_transactions".to_string(),
            params: serde_json::json!({}),
        };

        let response = self.send_request(request).await?;
        
        match response.error {
            Some(error) => Err(anyhow!("NWC error: {} - {}", error.code, error.message)),
            None => {
                if let Some(result) = response.result {
                    Ok(result)
                } else {
                    Err(anyhow!("No result in NWC response"))
                }
            }
        }
    }
}

impl Drop for NWCClient {
    fn drop(&mut self) {
        if self.is_connected {
            let _ = tokio::runtime::Handle::current().block_on(self.disconnect());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nwc_uri_parsing() {
        let uri = "nostr+walletconnect://b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4?relay=wss%3A%2F%2Frelay.damus.io&secret=71a8c14c1407c113601079c4302dab36460f0ccd0ad506f1f2dc73b5100e4f3c";
        
        let parsed = NWCConnectionUri::from_uri(uri).unwrap();
        assert_eq!(parsed.wallet_service_pubkey, "b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4");
        assert_eq!(parsed.relay_url, "wss://relay.damus.io");
        assert_eq!(parsed.secret, "71a8c14c1407c113601079c4302dab36460f0ccd0ad506f1f2dc73b5100e4f3c");
    }

    #[test]
    fn test_nwc_uri_with_lud16() {
        let uri = "nostr+walletconnect://b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4?relay=wss%3A%2F%2Frelay.damus.io&secret=71a8c14c1407c113601079c4302dab36460f0ccd0ad506f1f2dc73b5100e4f3c&lud16=test%40example.com";
        
        let parsed = NWCConnectionUri::from_uri(uri).unwrap();
        assert_eq!(parsed.lud16, Some("test@example.com".to_string()));
    }

    #[test]
    fn test_nwc_error_code_display() {
        assert_eq!(NWCErrorCode::RateLimited.to_string(), "RATE_LIMITED");
        assert_eq!(NWCErrorCode::PaymentFailed.to_string(), "PAYMENT_FAILED");
    }

    #[tokio::test]
    async fn test_nwc_client_creation() {
        let uri = "nostr+walletconnect://b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4?relay=wss%3A%2F%2Frelay.damus.io&secret=71a8c14c1407c113601079c4302dab36460f0ccd0ad506f1f2dc73b5100e4f3c";
        
        let client = NWCClient::from_uri(uri).await;
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_nwc_client_connection() {
        let uri = "nostr+walletconnect://b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4?relay=wss%3A%2F%2Frelay.damus.io&secret=71a8c14c1407c113601079c4302dab36460f0ccd0ad506f1f2dc73b5100e4f3c";
        
        let mut client = NWCClient::from_uri(uri).await.unwrap();
        assert!(!client.is_connected());
        
        // Note: This will fail in tests since we don't have a real relay
        // client.connect().await.unwrap();
        // assert!(client.is_connected());
        
        // client.disconnect().await.unwrap();
        // assert!(!client.is_connected());
    }
} 