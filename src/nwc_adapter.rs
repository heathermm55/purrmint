use anyhow::{anyhow, Result};
use async_trait::async_trait;
use cdk::{Amount, nuts::CurrencyUnit};
use cdk_common::lightning_invoice::Bolt11Invoice;
use cdk_common::payment::{MintPayment, CreateIncomingPaymentResponse, MakePaymentResponse, PaymentQuoteResponse};
use cdk_common::mint::MeltQuote;
use cdk_common::MeltOptions;
use futures::Stream;
use nwc::prelude::*;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

/// NWC Lightning backend implementation
#[derive(Debug, Clone)]
pub struct NWCLightningBackend {
    pub(crate) nwc: Arc<Mutex<Option<NWC>>>,
    pub(crate) connection_uri: String,
    pub fee_percent: f32,        // Fee percentage
    pub reserve_fee_min: u64,    // Minimum fee in msat
}

impl NWCLightningBackend {
    /// Create a new NWC Lightning backend
    pub fn new(connection_uri: String) -> Self {
        Self {
            nwc: Arc::new(Mutex::new(None)),
            connection_uri,
            fee_percent: 0.02, // Default fee
            reserve_fee_min: 1000, // Default reserve fee
        }
    }
    
    /// Create a new NWC Lightning backend with custom fee configuration
    pub fn new_with_fees(connection_uri: String, fee_percent: f32, reserve_fee_min: u64) -> Self {
        Self {
            nwc: Arc::new(Mutex::new(None)),
            connection_uri,
            fee_percent,
            reserve_fee_min,
        }
    }

    /// Get the NWC client, creating it if necessary
    fn get_nwc(&self) -> Result<NWC> {
        let mut nwc_guard = self.nwc.lock().map_err(|_| anyhow!("Failed to acquire lock"))?;
        
        if nwc_guard.is_none() {
            tracing::info!("Attempting to parse NWC URI: {}", self.connection_uri);
            let uri = NostrWalletConnectURI::from_str(&self.connection_uri)
                .map_err(|e| anyhow!("Failed to parse NWC URI '{}': {}", self.connection_uri, e))?;
            tracing::info!("Successfully parsed NWC URI");
            let nwc = NWC::new(uri);
            *nwc_guard = Some(nwc);
        }
        
        // Clone the NWC client since we can't return a reference from the mutex
        let nwc = nwc_guard.as_ref().unwrap().clone();
        Ok(nwc)
    }
}

#[async_trait]
impl MintPayment for NWCLightningBackend {
    type Err = cdk_common::payment::Error;

    async fn get_settings(&self) -> Result<serde_json::Value, Self::Err> {
        Ok(serde_json::json!({
            "mpp": true,
            "unit": "msat",
            "invoice_description": true,
            "amountless": true,
        }))
    }

    async fn create_incoming_payment_request(
        &self,
        amount: Amount,
        unit: &CurrencyUnit,
        description: String,
        unix_expiry: Option<u64>,
    ) -> Result<CreateIncomingPaymentResponse, Self::Err> {
        let nwc = self.get_nwc()
            .map_err(|e| Self::Err::Anyhow(anyhow!("Failed to get NWC client: {}", e)))?;
        
        let amount_msat = match unit {
            CurrencyUnit::Sat => *amount.as_ref() * 1000,
            CurrencyUnit::Msat => *amount.as_ref(),
            _ => return Err(Self::Err::UnsupportedUnit),
        };
        
        let request = MakeInvoiceRequest {
            amount: amount_msat,
            description: Some(description.clone()),
            description_hash: None,
            expiry: unix_expiry,
        };
        
        let response = nwc.make_invoice(request).await
            .map_err(|e| Self::Err::Anyhow(anyhow!("Failed to create invoice: {}", e)))?;
        
        let _invoice = Bolt11Invoice::from_str(&response.invoice)
            .map_err(|e| Self::Err::Parse(e))?;
        
        Ok(CreateIncomingPaymentResponse {
            request_lookup_id: response.payment_hash,
            request: response.invoice,
            expiry: unix_expiry,
        })
    }

    async fn get_payment_quote(
        &self,
        request: &str,
        unit: &CurrencyUnit,
        _options: Option<MeltOptions>,
    ) -> Result<PaymentQuoteResponse, Self::Err> {
        let _nwc = self.get_nwc()
            .map_err(|e| Self::Err::Anyhow(anyhow!("Failed to get NWC client: {}", e)))?;
        
        let invoice = Bolt11Invoice::from_str(request)
            .map_err(|e| Self::Err::Parse(e))?;
        
        let amount_msat = invoice.amount_milli_satoshis()
            .ok_or_else(|| Self::Err::Anyhow(anyhow!("Invoice has no amount")))?;
        
        let amount = match unit {
            CurrencyUnit::Sat => Amount::from(amount_msat / 1000),
            CurrencyUnit::Msat => Amount::from(amount_msat),
            _ => return Err(Self::Err::UnsupportedUnit),
        };
        
        // Calculate fee using configurable parameters instead of hardcoded values
        let fee_amount = (*amount.as_ref() as f64 * self.fee_percent as f64) as u64;
        let fee = std::cmp::max(fee_amount, self.reserve_fee_min);
        
        Ok(PaymentQuoteResponse {
            request_lookup_id: invoice.payment_hash().to_string(),
            amount,
            fee: Amount::from(fee),
            state: cdk_common::MeltQuoteState::Unpaid,
            unit: unit.clone(),
        })
    }

    async fn make_payment(
        &self,
        melt_quote: MeltQuote,
        _partial_amount: Option<Amount>,
        _max_fee_amount: Option<Amount>,
    ) -> Result<MakePaymentResponse, Self::Err> {
        let nwc = self.get_nwc()
            .map_err(|e| Self::Err::Anyhow(anyhow!("Failed to get NWC client: {}", e)))?;
        
        let request = PayInvoiceRequest::new(melt_quote.request.clone());
        
        let response = nwc.pay_invoice(request).await
            .map_err(|e| Self::Err::Anyhow(anyhow!("Failed to pay invoice: {}", e)))?;
        
        Ok(MakePaymentResponse {
            payment_lookup_id: melt_quote.request_lookup_id,
            payment_proof: Some(response.preimage),
            status: cdk_common::MeltQuoteState::Paid,
            total_spent: melt_quote.amount,
            unit: melt_quote.unit,
        })
    }

    async fn wait_any_incoming_payment(
        &self,
    ) -> Result<Pin<Box<dyn Stream<Item = String> + Send>>, Self::Err> {
        // NWC doesn't support waiting for incoming payments in the same way
        // This would need to be implemented differently for NWC
        Err(Self::Err::Anyhow(anyhow!("wait_any_incoming_payment not supported for NWC")))
    }

    fn is_wait_invoice_active(&self) -> bool {
        false // NWC doesn't support this
    }

    fn cancel_wait_invoice(&self) {
        // NWC doesn't support this
    }

    async fn check_incoming_payment_status(
        &self,
        request_lookup_id: &str,
    ) -> Result<cdk_common::MintQuoteState, Self::Err> {
        let nwc = self.get_nwc()
            .map_err(|e| Self::Err::Anyhow(anyhow!("Failed to get NWC client: {}", e)))?;
        
        let request = LookupInvoiceRequest {
            payment_hash: Some(request_lookup_id.to_string()),
            invoice: None,
        };
        
        let response = nwc.lookup_invoice(request).await
            .map_err(|e| Self::Err::Anyhow(anyhow!("Failed to lookup invoice: {}", e)))?;
        
        // Check if the invoice has been paid
        if response.settled_at.is_some() {
            Ok(cdk_common::MintQuoteState::Paid)
        } else {
            Ok(cdk_common::MintQuoteState::Unpaid)
        }
    }

    async fn check_outgoing_payment(
        &self,
        request_lookup_id: &str,
    ) -> Result<MakePaymentResponse, Self::Err> {
        let nwc = self.get_nwc()
            .map_err(|e| Self::Err::Anyhow(anyhow!("Failed to get NWC client: {}", e)))?;
        
        let request = LookupInvoiceRequest {
            payment_hash: Some(request_lookup_id.to_string()),
            invoice: None,
        };
        
        let response = nwc.lookup_invoice(request).await
            .map_err(|e| Self::Err::Anyhow(anyhow!("Failed to lookup invoice: {}", e)))?;
        
        let status = if response.settled_at.is_some() {
            cdk_common::MeltQuoteState::Paid
        } else {
            cdk_common::MeltQuoteState::Unpaid
        };
        
        Ok(MakePaymentResponse {
            payment_lookup_id: request_lookup_id.to_string(),
            payment_proof: Some(response.preimage.unwrap_or_default()),
            status,
            total_spent: Amount::from(response.amount),
            unit: CurrencyUnit::Msat,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nwc_backend_creation() {
        let uri = "nostr+walletconnect://test?relay=wss://test.com&secret=test";
        let backend = NWCLightningBackend::new(uri.to_string());
        
        assert_eq!(backend.connection_uri, uri);
        assert_eq!(backend.fee_percent, 0.02);
        assert_eq!(backend.reserve_fee_min, 1000);
        assert!(backend.nwc.lock().unwrap().is_none());
    }
    
    #[test]
    fn test_nwc_backend_creation_with_fees() {
        let uri = "nostr+walletconnect://test?relay=wss://test.com&secret=test";
        let backend = NWCLightningBackend::new_with_fees(uri.to_string(), 0.03, 500);
        
        assert_eq!(backend.connection_uri, uri);
        assert_eq!(backend.fee_percent, 0.03);
        assert_eq!(backend.reserve_fee_min, 500);
        assert!(backend.nwc.lock().unwrap().is_none());
    }
} 