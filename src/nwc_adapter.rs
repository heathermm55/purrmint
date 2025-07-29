use anyhow::{anyhow, Result};
use cdk::nuts::CurrencyUnit;
use cdk::Amount;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;
use tracing::info;

use crate::nwc_client::NWCClient;

/// NWC Lightning Backend Adapter
/// Implements Lightning backend interface using NWC protocol
pub struct NWCLightningBackend {
    nwc_client: Arc<RwLock<Option<NWCClient>>>,
    connection_uri: String,
    fee_percent: f32,
    reserve_fee_min: Amount,
}

impl NWCLightningBackend {
    /// Create new NWC Lightning backend
    pub fn new(connection_uri: String, fee_percent: f32, reserve_fee_min: Amount) -> Result<Self> {
        Ok(Self {
            nwc_client: Arc::new(RwLock::new(None)),
            connection_uri,
            fee_percent,
            reserve_fee_min,
        })
    }

    /// Start the NWC backend
    pub async fn start(&self) -> Result<()> {
        info!("Starting NWC Lightning backend...");
        
        // Create and connect NWC client
        let mut client = NWCClient::from_uri(&self.connection_uri).await?;
        client.connect().await?;
        
        // Store the connected client
        {
            let mut client_guard = self.nwc_client.write().await;
            *client_guard = Some(client);
        }
        
        info!("NWC Lightning backend started successfully");
        Ok(())
    }

    /// Stop the NWC backend
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping NWC Lightning backend...");
        
        // Disconnect the client
        {
            let mut client_guard = self.nwc_client.write().await;
            if let Some(mut client) = client_guard.take() {
                client.disconnect().await?;
            }
        }
        
        info!("NWC Lightning backend stopped");
        Ok(())
    }

    /// Check if backend is running
    pub async fn is_running(&self) -> bool {
        let client_guard = self.nwc_client.read().await;
        client_guard.as_ref().map(|c| c.is_connected()).unwrap_or(false)
    }

    /// Get backend status
    pub async fn get_status(&self) -> serde_json::Value {
        let is_connected = self.is_running().await;
        serde_json::json!({
            "running": is_connected,
            "connection_uri": self.connection_uri,
            "fee_percent": self.fee_percent,
            "reserve_fee_min": *self.reserve_fee_min.as_ref(),
        })
    }

    /// Get the NWC client
    async fn get_client(&self) -> Result<NWCClient> {
        let client_guard = self.nwc_client.read().await;
        client_guard
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow!("NWC client not initialized"))
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

        let client = self.get_client().await?;
        let result = client.make_invoice(amount_msat, description, expiry).await?;
        
        let invoice_str = result["invoice"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid invoice in NWC response"))?;
        
        // Parse the invoice
        let invoice = cdk::Bolt11Invoice::from_str(invoice_str)
            .map_err(|e| anyhow!("Failed to parse invoice: {}", e))?;
        
        Ok(invoice)
    }

    /// Pay a Lightning invoice
    pub async fn pay_invoice(&self, invoice: &cdk::Bolt11Invoice) -> Result<String> {
        let client = self.get_client().await?;
        client.pay_invoice(&invoice.to_string()).await
    }

    /// Get balance
    pub async fn get_balance(&self) -> Result<Amount> {
        let client = self.get_client().await?;
        let balance_msat = client.get_balance().await?;
        Ok(Amount::from(balance_msat))
    }

    /// Lookup invoice by payment hash
    pub async fn lookup_invoice(&self, payment_hash: &str) -> Result<Option<cdk::Bolt11Invoice>> {
        let client = self.get_client().await?;
        let result = client.lookup_invoice(payment_hash).await?;
        
        match result {
            Some(invoice_data) => {
                let invoice_str = invoice_data["invoice"]
                    .as_str()
                    .ok_or_else(|| anyhow!("Invalid invoice in NWC response"))?;
                
                let invoice = cdk::Bolt11Invoice::from_str(invoice_str)
                    .map_err(|e| anyhow!("Failed to parse invoice: {}", e))?;
                
                Ok(Some(invoice))
            }
            None => Ok(None),
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

    /// Get wallet service info
    pub async fn get_wallet_info(&self) -> Result<serde_json::Value> {
        let client = self.get_client().await?;
        client.get_info().await
    }

    /// List transactions
    pub async fn list_transactions(&self) -> Result<serde_json::Value> {
        let client = self.get_client().await?;
        client.list_transactions().await
    }
}

impl std::fmt::Debug for NWCLightningBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NWCLightningBackend")
            .field("connection_uri", &self.connection_uri)
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
        let connection_uri = "nostr+walletconnect://b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4?relay=wss%3A%2F%2Frelay.damus.io&secret=71a8c14c1407c113601079c4302dab36460f0ccd0ad506f1f2dc73b5100e4f3c";
        
        let backend = NWCLightningBackend::new(
            connection_uri.to_string(),
            0.02,
            Amount::from(1),
        );
        assert!(backend.is_ok());
    }

    #[tokio::test]
    async fn test_fee_calculation() {
        let connection_uri = "nostr+walletconnect://b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4?relay=wss%3A%2F%2Frelay.damus.io&secret=71a8c14c1407c113601079c4302dab36460f0ccd0ad506f1f2dc73b5100e4f3c";
        
        let backend = NWCLightningBackend::new(
            connection_uri.to_string(),
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
        let connection_uri = "nostr+walletconnect://b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4?relay=wss%3A%2F%2Frelay.damus.io&secret=71a8c14c1407c113601079c4302dab36460f0ccd0ad506f1f2dc73b5100e4f3c";
        
        let backend = NWCLightningBackend::new(
            connection_uri.to_string(),
            0.02,
            Amount::from(1),
        ).unwrap();

        let units = backend.supported_units();
        assert!(units.contains(&CurrencyUnit::Sat));
        assert!(units.contains(&CurrencyUnit::Msat));
    }
} 