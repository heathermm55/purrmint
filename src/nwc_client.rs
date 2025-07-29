use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

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

/// NWC Client - simplified implementation for connecting to external Lightning wallet service
#[derive(Clone)]
pub struct NWCClient {
    wallet_service_pubkey: String,
    relay_url: String,
    secret: String,
    is_connected: bool,
}

impl NWCClient {
    /// Create new NWC client from connection URI
    pub async fn from_uri(connection_uri: &str) -> Result<Self> {
        let uri = NWCConnectionUri::from_uri(connection_uri)?;
        
        Ok(Self {
            wallet_service_pubkey: uri.wallet_service_pubkey,
            relay_url: uri.relay_url,
            secret: uri.secret,
            is_connected: false,
        })
    }

    /// Connect to the wallet service
    pub async fn connect(&mut self) -> Result<()> {
        info!("Connecting to NWC wallet service...");
        
        // TODO: Implement actual Nostr relay connection
        // For now, just mark as connected
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
        self.is_connected = false;
        info!("Disconnected from NWC wallet service");
        Ok(())
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.is_connected
    }

    /// Send request to wallet service (mock implementation)
    async fn send_request(&self, request: NWCRequest) -> Result<NWCResponse> {
        if !self.is_connected {
            return Err(anyhow!("Not connected to wallet service"));
        }

        debug!("Sending NWC request: {:?}", request);

        // TODO: Implement actual Nostr communication
        // For now, return mock responses
        match request.method.as_str() {
            "get_info" => Ok(NWCResponse {
                result_type: request.method,
                error: None,
                result: Some(serde_json::json!({
                    "alias": "Mock NWC Wallet",
                    "color": "ff6600",
                    "pubkey": self.wallet_service_pubkey,
                    "network": "mainnet",
                    "block_height": 800000,
                    "block_hash": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                    "methods": ["pay_invoice", "make_invoice", "get_balance", "get_info", "lookup_invoice", "list_transactions"],
                    "notifications": ["payment_received", "payment_sent"]
                })),
            }),
            "pay_invoice" => {
                let invoice = request.params["invoice"].as_str()
                    .ok_or_else(|| anyhow!("Missing invoice parameter"))?;
                
                Ok(NWCResponse {
                    result_type: request.method,
                    error: None,
                    result: Some(serde_json::json!({
                        "preimage": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                        "fees_paid": 0
                    })),
                })
            },
            "make_invoice" => {
                let amount = request.params["amount"].as_u64()
                    .ok_or_else(|| anyhow!("Missing amount parameter"))?;
                
                let description = request.params["description"].as_str().unwrap_or("Cashu Mint Invoice");
                
                Ok(NWCResponse {
                    result_type: request.method,
                    error: None,
                    result: Some(serde_json::json!({
                        "type": "incoming",
                        "invoice": "lnbc1mockinvoice...",
                        "description": description,
                        "payment_hash": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                        "amount": amount,
                        "fees_paid": 0,
                        "created_at": chrono::Utc::now().timestamp(),
                        "expires_at": chrono::Utc::now().timestamp() + 3600,
                        "metadata": {}
                    })),
                })
            },
            "get_balance" => Ok(NWCResponse {
                result_type: request.method,
                error: None,
                result: Some(serde_json::json!({
                    "balance": 1000000 // 1 BTC in msats
                })),
            }),
            "lookup_invoice" => {
                let payment_hash = request.params["payment_hash"].as_str()
                    .ok_or_else(|| anyhow!("Missing payment_hash parameter"))?;
                
                Ok(NWCResponse {
                    result_type: request.method,
                    error: None,
                    result: Some(serde_json::json!({
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
                    })),
                })
            },
            "list_transactions" => Ok(NWCResponse {
                result_type: request.method,
                error: None,
                result: Some(serde_json::json!({
                    "transactions": []
                })),
            }),
            _ => {
                let method = request.method.clone();
                Ok(NWCResponse {
                    result_type: method.clone(),
                    error: Some(NWCError {
                        code: NWCErrorCode::NotImplemented,
                        message: format!("Method {} not implemented", method),
                    }),
                    result: None,
                })
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
        
        client.connect().await.unwrap();
        assert!(client.is_connected());
        
        client.disconnect().await.unwrap();
        assert!(!client.is_connected());
    }
} 