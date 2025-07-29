use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

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

/// NWC Connection Info
#[derive(Debug, Clone)]
pub struct NWCConnection {
    pub client_pubkey: String,
    pub relay_url: String,
    pub secret: String,
    pub lud16: Option<String>,
    pub capabilities: Vec<String>,
    pub notifications: Vec<String>,
}

/// NWC Service Configuration
#[derive(Debug, Clone)]
pub struct NWCConfig {
    pub relay_urls: Vec<String>,
    pub supported_methods: Vec<String>,
    pub supported_notifications: Vec<String>,
    pub lud16: Option<String>,
}

/// NWC Service - simplified implementation
pub struct NWCService {
    config: NWCConfig,
    connections: Arc<RwLock<HashMap<String, NWCConnection>>>,
    is_running: bool,
}

impl NWCService {
    /// Create new NWC service
    pub fn new(config: NWCConfig) -> Result<Self> {
        Ok(Self {
            config,
            connections: Arc::new(RwLock::new(HashMap::new())),
            is_running: false,
        })
    }

    /// Start the NWC service
    pub async fn start(&mut self) -> Result<()> {
        if self.is_running {
            return Ok(());
        }

        info!("Starting NWC service...");
        
        // TODO: Implement actual Nostr relay connection
        // For now, just mark as running
        self.is_running = true;
        info!("NWC service started successfully");
        Ok(())
    }

    /// Stop the NWC service
    pub async fn stop(&mut self) -> Result<()> {
        if !self.is_running {
            return Ok(());
        }

        info!("Stopping NWC service...");
        self.is_running = false;
        info!("NWC service stopped");
        Ok(())
    }

    /// Generate connection URI for a client
    pub async fn generate_connection_uri(&self, client_secret: String, relay_url: String) -> Result<String> {
        // TODO: Implement proper key derivation
        let client_pubkey = format!("client_{}", client_secret[..8].to_string());
        
        let connection = NWCConnection {
            client_pubkey: client_pubkey.clone(),
            relay_url: relay_url.clone(),
            secret: client_secret.clone(),
            lud16: self.config.lud16.clone(),
            capabilities: self.config.supported_methods.clone(),
            notifications: self.config.supported_notifications.clone(),
        };

        // Store connection
        {
            let mut connections = self.connections.write().await;
            connections.insert(client_pubkey.clone(), connection);
        }

        // Generate URI
        let uri = format!(
            "nostr+walletconnect://{}?relay={}&secret={}",
            "wallet_service_pubkey", // TODO: Use actual wallet service pubkey
            urlencoding::encode(&relay_url),
            client_secret
        );

        Ok(uri)
    }

    /// Handle NWC request
    pub async fn handle_request(&self, request: NWCRequest) -> Result<NWCResponse> {
        debug!("Handling NWC request: {:?}", request);

        let response = match request.method.as_str() {
            "pay_invoice" => self.handle_pay_invoice(&request.params).await,
            "make_invoice" => self.handle_make_invoice(&request.params).await,
            "get_balance" => self.handle_get_balance(&request.params).await,
            "get_info" => self.handle_get_info(&request.params).await,
            "lookup_invoice" => self.handle_lookup_invoice(&request.params).await,
            "list_transactions" => self.handle_list_transactions(&request.params).await,
            _ => Err(anyhow!("Method not implemented: {}", request.method)),
        };

        match response {
            Ok(result_value) => Ok(NWCResponse {
                result_type: request.method,
                error: None,
                result: Some(result_value),
            }),
            Err(e) => Ok(NWCResponse {
                result_type: request.method,
                error: Some(NWCError {
                    code: NWCErrorCode::Internal,
                    message: e.to_string(),
                }),
                result: None,
            }),
        }
    }

    // Handler methods for different NWC commands
    async fn handle_pay_invoice(&self, params: &serde_json::Value) -> Result<serde_json::Value> {
        let _invoice = params["invoice"].as_str()
            .ok_or_else(|| anyhow!("Missing invoice parameter"))?;
        
        // TODO: Implement actual payment logic
        // For now, return a mock response
        Ok(serde_json::json!({
            "preimage": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            "fees_paid": 0
        }))
    }

    async fn handle_make_invoice(&self, params: &serde_json::Value) -> Result<serde_json::Value> {
        let amount = params["amount"].as_u64()
            .ok_or_else(|| anyhow!("Missing amount parameter"))?;
        
        let description = params["description"].as_str().unwrap_or("NWC Invoice");
        
        // TODO: Implement actual invoice creation logic
        // For now, return a mock response
        Ok(serde_json::json!({
            "type": "incoming",
            "invoice": "lnbc1mockinvoice...",
            "description": description,
            "payment_hash": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            "amount": amount,
            "fees_paid": 0,
            "created_at": chrono::Utc::now().timestamp(),
            "expires_at": chrono::Utc::now().timestamp() + 3600,
            "metadata": {}
        }))
    }

    async fn handle_get_balance(&self, _params: &serde_json::Value) -> Result<serde_json::Value> {
        // TODO: Implement actual balance retrieval
        // For now, return a mock response
        Ok(serde_json::json!({
            "balance": 1000000 // 1 BTC in msats
        }))
    }

    async fn handle_get_info(&self, _params: &serde_json::Value) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "alias": "Purrmint NWC",
            "color": "ff6600",
            "pubkey": "wallet_service_pubkey",
            "network": "mainnet",
            "block_height": 800000,
            "block_hash": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            "methods": self.config.supported_methods,
            "notifications": self.config.supported_notifications
        }))
    }

    async fn handle_lookup_invoice(&self, params: &serde_json::Value) -> Result<serde_json::Value> {
        let payment_hash = params["payment_hash"].as_str()
            .ok_or_else(|| anyhow!("Missing payment_hash parameter"))?;
        
        // TODO: Implement actual invoice lookup
        // For now, return a mock response
        Ok(serde_json::json!({
            "type": "incoming",
            "invoice": "lnbc1mockinvoice...",
            "description": "NWC Invoice",
            "payment_hash": payment_hash,
            "amount": 1000,
            "fees_paid": 0,
            "created_at": chrono::Utc::now().timestamp(),
            "expires_at": chrono::Utc::now().timestamp() + 3600,
            "settled_at": chrono::Utc::now().timestamp(),
            "metadata": {}
        }))
    }

    async fn handle_list_transactions(&self, _params: &serde_json::Value) -> Result<serde_json::Value> {
        // TODO: Implement actual transaction listing
        // For now, return a mock response
        Ok(serde_json::json!({
            "transactions": []
        }))
    }

    /// Send notification to client
    pub async fn send_notification(
        &self,
        client_pubkey: &str,
        notification_type: &str,
        notification_data: serde_json::Value,
    ) -> Result<()> {
        let notification = NWCNotification {
            notification_type: notification_type.to_string(),
            notification: notification_data,
        };

        // TODO: Implement actual notification sending via Nostr
        info!("Sending notification to {}: {:?}", client_pubkey, notification);
        Ok(())
    }

    /// Check if service is running
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// Get service status
    pub fn get_status(&self) -> serde_json::Value {
        serde_json::json!({
            "running": self.is_running,
            "supported_methods": self.config.supported_methods,
            "supported_notifications": self.config.supported_notifications,
            "relay_urls": self.config.relay_urls,
            "lud16": self.config.lud16,
        })
    }
}

impl Drop for NWCService {
    fn drop(&mut self) {
        if self.is_running {
            let _ = tokio::runtime::Handle::current().block_on(self.stop());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_nwc_service_creation() {
        let config = NWCConfig {
            relay_urls: vec!["wss://relay.damus.io".to_string()],
            supported_methods: vec![
                "pay_invoice".to_string(),
                "make_invoice".to_string(),
                "get_balance".to_string(),
            ],
            supported_notifications: vec![
                "payment_received".to_string(),
                "payment_sent".to_string(),
            ],
            lud16: Some("test@example.com".to_string()),
        };

        let service = NWCService::new(config);
        assert!(service.is_ok());
    }

    #[test]
    fn test_nwc_error_code_display() {
        assert_eq!(NWCErrorCode::RateLimited.to_string(), "RATE_LIMITED");
        assert_eq!(NWCErrorCode::PaymentFailed.to_string(), "PAYMENT_FAILED");
    }

    #[tokio::test]
    async fn test_nwc_request_handling() {
        let config = NWCConfig {
            relay_urls: vec!["wss://relay.damus.io".to_string()],
            supported_methods: vec!["get_balance".to_string()],
            supported_notifications: vec![],
            lud16: None,
        };

        let service = NWCService::new(config).unwrap();
        
        let request = NWCRequest {
            method: "get_balance".to_string(),
            params: serde_json::json!({}),
        };

        let response = service.handle_request(request).await.unwrap();
        assert_eq!(response.result_type, "get_balance");
        assert!(response.error.is_none());
        assert!(response.result.is_some());
    }
} 