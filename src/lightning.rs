use serde::{Deserialize, Serialize};

/// Lightning backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightningConfig {
    pub backend_type: LightningBackendType,
    pub config: serde_json::Value,
}

/// Supported lightning backend types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LightningBackendType {
    Cln,
    Lnd,
    Lnbits,
    FakeWallet,
}

impl Default for LightningConfig {
    fn default() -> Self {
        Self {
            backend_type: LightningBackendType::FakeWallet,
            config: serde_json::json!({}),
        }
    }
} 