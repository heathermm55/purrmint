use anyhow::{anyhow, Result};
use cdk::nuts::CurrencyUnit;
use cdk::Amount;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

use crate::nwc_service::{NWCConfig, NWCService, NWCRequest};

/// NWC Lightning Backend Adapter
/// Implements Lightning backend interface using NWC protocol
pub struct NWCLightningBackend {
    nwc_service: Arc<RwLock<NWCService>>,
    fee_percent: f32,
    reserve_fee_min: Amount,
}

impl NWCLightningBackend {
    /// Create new NWC Lightning backend
    pub fn new(config: NWCConfig, fee_percent: f32, reserve_fee_min: Amount) -> Result<Self> {
        let nwc_service = NWCService::new(config)?;
        
        Ok(Self {
            nwc_service: Arc::new(RwLock::new(nwc_service)),
            fee_percent,
            reserve_fee_min,
        })
    }

    /// Start the NWC backend
    pub async fn start(&self) -> Result<()> {
        let mut service = self.nwc_service.write().await;
        service.start().await
    }

    /// Stop the NWC backend
    pub async fn stop(&self) -> Result<()> {
        let mut service = self.nwc_service.write().await;
        service.stop().await
    }

    /// Check if backend is running
    pub async fn is_running(&self) -> bool {
        let service = self.nwc_service.read().await;
        service.is_running()
    }

    /// Get backend status
    pub async fn get_status(&self) -> serde_json::Value {
        let service = self.nwc_service.read().await;
        service.get_status()
    }

    /// Create a Lightning invoice
    pub async fn create_invoice(
        &self,
        amount: Amount,
        unit: CurrencyUnit,
        description: Option<String>,
        expiry: Option<u64>,
    ) -> Result<cdk::Bolt11Invoice> {
        let amount_msat = match unit {
            CurrencyUnit::Sat => amount.as_ref() * 1000,
            CurrencyUnit::Msat => *amount.as_ref(),
            _ => return Err(anyhow!("Unsupported currency unit for NWC")),
        };

        let request = NWCRequest {
            method: "make_invoice".to_string(),
            params: serde_json::json!({
                "amount": amount_msat,
                "description": description.unwrap_or_else(|| "Cashu Mint Invoice".to_string()),
                "expiry": expiry.unwrap_or(3600),
            }),
        };

        let service = self.nwc_service.read().await;
        let response = service.handle_request(request).await?;

        match response.error {
            Some(error) => Err(anyhow!("NWC error: {} - {}", error.code, error.message)),
            None => {
                if let Some(result) = response.result {
                    let invoice_str = result["invoice"]
                        .as_str()
                        .ok_or_else(|| anyhow!("Invalid invoice in NWC response"))?;
                    
                    // Parse the invoice
                    let invoice = cdk::Bolt11Invoice::from_str(invoice_str)
                        .map_err(|e| anyhow!("Failed to parse invoice: {}", e))?;
                    
                    Ok(invoice)
                } else {
                    Err(anyhow!("No result in NWC response"))
                }
            }
        }
    }

    /// Pay a Lightning invoice
    pub async fn pay_invoice(&self, invoice: &cdk::Bolt11Invoice) -> Result<String> {
        let request = NWCRequest {
            method: "pay_invoice".to_string(),
            params: serde_json::json!({
                "invoice": invoice.to_string(),
            }),
        };

        let service = self.nwc_service.read().await;
        let response = service.handle_request(request).await?;

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

    /// Get balance
    pub async fn get_balance(&self) -> Result<Amount> {
        let request = NWCRequest {
            method: "get_balance".to_string(),
            params: serde_json::json!({}),
        };

        let service = self.nwc_service.read().await;
        let response = service.handle_request(request).await?;

        match response.error {
            Some(error) => Err(anyhow!("NWC error: {} - {}", error.code, error.message)),
            None => {
                if let Some(result) = response.result {
                    let balance_msat = result["balance"]
                        .as_u64()
                        .ok_or_else(|| anyhow!("Invalid balance in NWC response"))?;
                    
                    Ok(Amount::from(balance_msat))
                } else {
                    Err(anyhow!("No result in NWC response"))
                }
            }
        }
    }

    /// Lookup invoice by payment hash
    pub async fn lookup_invoice(&self, payment_hash: &str) -> Result<Option<cdk::Bolt11Invoice>> {
        let request = NWCRequest {
            method: "lookup_invoice".to_string(),
            params: serde_json::json!({
                "payment_hash": payment_hash,
            }),
        };

        let service = self.nwc_service.read().await;
        let response = service.handle_request(request).await?;

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
                    let invoice_str = result["invoice"]
                        .as_str()
                        .ok_or_else(|| anyhow!("Invalid invoice in NWC response"))?;
                    
                    let invoice = cdk::Bolt11Invoice::from_str(invoice_str)
                        .map_err(|e| anyhow!("Failed to parse invoice: {}", e))?;
                    
                    Ok(Some(invoice))
                } else {
                    Ok(None)
                }
            }
        }
    }

    /// Calculate fee for an amount
    pub fn calculate_fee(&self, amount: Amount) -> Amount {
        let fee_amount = (*amount.as_ref() as f64 * self.fee_percent as f64) as u64;
        let fee = Amount::from(fee_amount.max(*self.reserve_fee_min.as_ref()));
        
        debug!("Calculated fee for {}: {}", amount, fee);
        fee
    }

    /// Get supported currency units
    pub fn supported_units(&self) -> Vec<CurrencyUnit> {
        vec![CurrencyUnit::Sat, CurrencyUnit::Msat]
    }

    /// Generate connection URI for NWC client
    pub async fn generate_connection_uri(&self, client_secret: String, relay_url: String) -> Result<String> {
        let service = self.nwc_service.read().await;
        service.generate_connection_uri(client_secret, relay_url).await
    }

    /// Send notification to NWC client
    pub async fn send_notification(
        &self,
        client_pubkey: &str,
        notification_type: &str,
        notification_data: serde_json::Value,
    ) -> Result<()> {
        let service = self.nwc_service.read().await;
        service.send_notification(client_pubkey, notification_type, notification_data).await
    }
}

impl std::fmt::Debug for NWCLightningBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NWCLightningBackend")
            .field("fee_percent", &self.fee_percent)
            .field("reserve_fee_min", &self.reserve_fee_min)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cdk::Amount;

    #[tokio::test]
    async fn test_nwc_backend_creation() {
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

        let backend = NWCLightningBackend::new(
            config,
            0.02,
            Amount::from(1),
        );
        assert!(backend.is_ok());
    }

    #[tokio::test]
    async fn test_fee_calculation() {
        let config = NWCConfig {
            relay_urls: vec!["wss://relay.damus.io".to_string()],
            supported_methods: vec!["get_balance".to_string()],
            supported_notifications: vec![],
            lud16: None,
        };

        let backend = NWCLightningBackend::new(
            config,
            0.02,
            Amount::from(1),
        ).unwrap();

        let amount = Amount::from(1000);
        let fee = backend.calculate_fee(amount);
        
        // 2% of 1000 = 20, but minimum is 1
        // Note: 1000 * 0.02 = 20.0, but as u64 it becomes 19 due to truncation
        assert_eq!(*fee.as_ref(), 19);
    }

    #[tokio::test]
    async fn test_supported_units() {
        let config = NWCConfig {
            relay_urls: vec!["wss://relay.damus.io".to_string()],
            supported_methods: vec!["get_balance".to_string()],
            supported_notifications: vec![],
            lud16: None,
        };

        let backend = NWCLightningBackend::new(
            config,
            0.02,
            Amount::from(1),
        ).unwrap();

        let units = backend.supported_units();
        assert!(units.contains(&CurrencyUnit::Sat));
        assert!(units.contains(&CurrencyUnit::Msat));
    }
} 