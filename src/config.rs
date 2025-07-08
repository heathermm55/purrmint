// Path import removed - not needed for basic Android functionality
use cdk::nuts::{CurrencyUnit, PublicKey};
use cdk::Amount;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};

// Lightning backend configuration removed - not needed for basic Android functionality

// =============================================================================
// Service Mode Configuration
// =============================================================================

/// Service operation mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ServiceMode {
    /// Only mintd service (HTTP API)
    MintdOnly,
    /// Only NIP-74 service (Nostr events)
    Nip74Only,
    /// Both mintd and NIP-74 services
    MintdAndNip74,
}

impl Default for ServiceMode {
    fn default() -> Self {
        ServiceMode::MintdOnly
    }
}

// Display implementation removed - not needed for basic functionality

// =============================================================================
// Configuration Structures
// =============================================================================

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct Info {
    pub url: String,
    pub listen_host: String,
    pub listen_port: u16,
    pub mnemonic: Option<String>,
    pub signatory_url: Option<String>,
    pub signatory_certs: Option<String>,
    pub input_fee_ppk: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum LnBackend {
    #[default]
    None,
    FakeWallet,
    LNbits,
    Cln,
    Lnd,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ln {
    pub ln_backend: LnBackend,
    pub invoice_description: Option<String>,
    pub min_mint: Amount,
    pub max_mint: Amount,
    pub min_melt: Amount,
    pub max_melt: Amount,
}

impl Default for Ln {
    fn default() -> Self {
        Ln {
            ln_backend: LnBackend::default(),
            invoice_description: None,
            min_mint: 1.into(),
            max_mint: 500_000.into(),
            min_melt: 1.into(),
            max_melt: 500_000.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FakeWallet {
    pub supported_units: Vec<CurrencyUnit>,
    pub fee_percent: f32,
    pub reserve_fee_min: Amount,
    pub min_delay_time: u64,
    pub max_delay_time: u64,
}

impl Default for FakeWallet {
    fn default() -> Self {
        Self {
            supported_units: vec![CurrencyUnit::Sat],
            fee_percent: 0.02,
            reserve_fee_min: 2.into(),
            min_delay_time: 1,
            max_delay_time: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LNbits {
    pub admin_api_key: String,
    pub invoice_api_key: String,
    pub lnbits_api: String,
    pub fee_percent: f32,
    pub reserve_fee_min: Amount,
}

impl Default for LNbits {
    fn default() -> Self {
        Self {
            admin_api_key: String::new(),
            invoice_api_key: String::new(),
            lnbits_api: String::new(),
            fee_percent: 0.02,
            reserve_fee_min: 2.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cln {
    pub rpc_path: String,
    pub bolt12: bool,
    pub fee_percent: f32,
    pub reserve_fee_min: Amount,
}

impl Default for Cln {
    fn default() -> Self {
        Self {
            rpc_path: String::new(),
            bolt12: false,
            fee_percent: 0.02,
            reserve_fee_min: 2.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseEngine {
    #[default]
    Sqlite,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Database {
    pub engine: DatabaseEngine,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MintInfo {
    pub name: String,
    pub pubkey: Option<PublicKey>,
    pub description: String,
    pub description_long: Option<String>,
    pub icon_url: Option<String>,
    pub motd: Option<String>,
    pub contact_nostr_public_key: Option<String>,
    pub contact_email: Option<String>,
    pub tos_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub info: Info,
    pub mint_info: MintInfo,
    pub ln: Ln,
    pub fake_wallet: Option<FakeWallet>,
    pub lnbits: Option<LNbits>,
    pub cln: Option<Cln>,
    pub database: Database,
    pub service_mode: ServiceMode,
}

// =============================================================================
// Android Configuration
// =============================================================================

/// Android-specific configuration (simplified JSON format for JNI)
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AndroidConfig {
    pub port: u16,
    pub host: String,
    pub mint_name: String,
    pub description: String,
    pub lightning_backend: String,
    pub mode: String,
    pub database_path: String,
    pub logs_path: String,
    pub lnbits_admin_api_key: Option<String>,
    pub lnbits_invoice_api_key: Option<String>,
    pub lnbits_api_url: Option<String>,
    pub cln_rpc_path: Option<String>,
    pub cln_bolt12: Option<bool>,
}

impl Default for AndroidConfig {
    fn default() -> Self {
        Self {
            port: 3338,
            host: "0.0.0.0".to_string(),
            mint_name: "PurrMint".to_string(),
            description: "Mobile Cashu Mint".to_string(),
            lightning_backend: "fakewallet".to_string(),
            mode: "mintd_only".to_string(),
            database_path: "".to_string(),
            logs_path: "".to_string(),
            lnbits_admin_api_key: None,
            lnbits_invoice_api_key: None,
            lnbits_api_url: None,
            cln_rpc_path: None,
            cln_bolt12: None,
        }
    }
}

// =============================================================================
// Configuration Management Functions
// =============================================================================

impl Settings {
    /// Create default settings with optional mnemonic
    pub fn default_with_mnemonic(mnemonic: Option<String>) -> Self {
        let info = Info {
            url: "http://localhost:3338/".to_string(),
            listen_host: "0.0.0.0".to_string(),
            listen_port: 3338,
            mnemonic,
            signatory_url: None,
            signatory_certs: None,
            input_fee_ppk: None,
        };

        let mint_info = MintInfo {
            name: "PurrMint".to_string(),
            pubkey: None,
            description: "PurrMint Cashu Mint".to_string(),
            description_long: None,
            icon_url: None,
            motd: None,
            contact_nostr_public_key: None,
            contact_email: None,
            tos_url: None,
        };

        let ln = Ln::default();
        let database = Database {
            engine: DatabaseEngine::Sqlite,
        };

        Settings {
            info,
            mint_info,
            ln,
            fake_wallet: Some(FakeWallet::default()),
            lnbits: None,
            cln: None,
            database,
            service_mode: ServiceMode::default(),
        }
    }

    // TOML file operations removed - Android uses JSON configuration
}

impl AndroidConfig {
    /// Convert AndroidConfig to Settings
    pub fn to_settings(&self, mnemonic: Option<String>) -> Settings {
        let mut settings = Settings::default_with_mnemonic(mnemonic);
        
        settings.info.listen_port = self.port;
        settings.info.listen_host = self.host.clone();
        settings.info.url = format!("http://{}:{}/", self.host, self.port);
        settings.mint_info.name = self.mint_name.clone();
        settings.mint_info.description = self.description.clone();
        
        // Set lightning backend
        settings.ln.ln_backend = match self.lightning_backend.as_str() {
            "fake" | "fakewallet" => LnBackend::FakeWallet,
            "lnbits" => LnBackend::LNbits,
            "cln" => LnBackend::Cln,
            _ => LnBackend::None,
        };
        
        // Set backend-specific configuration
        match self.lightning_backend.as_str() {
            "fake" | "fakewallet" => {
                // Keep default fake wallet config
            }
            "lnbits" => {
                // Set LNBits configuration
                if let (Some(admin_key), Some(invoice_key), Some(api_url)) = (
                    &self.lnbits_admin_api_key,
                    &self.lnbits_invoice_api_key,
                    &self.lnbits_api_url
                ) {
                    settings.lnbits = Some(LNbits {
                        admin_api_key: admin_key.clone(),
                        invoice_api_key: invoice_key.clone(),
                        lnbits_api: api_url.clone(),
                        fee_percent: 0.02,
                        reserve_fee_min: 1.into(),
                    });
                    // Clear fake wallet config when using LNBits
                    settings.fake_wallet = None;
                }
            }
            "cln" => {
                // Set CLN configuration
                if let Some(rpc_path) = &self.cln_rpc_path {
                    settings.cln = Some(Cln {
                        rpc_path: rpc_path.clone(),
                        bolt12: self.cln_bolt12.unwrap_or(false),
                        fee_percent: 0.02,
                        reserve_fee_min: 1.into(),
                    });
                    // Clear fake wallet config when using CLN
                    settings.fake_wallet = None;
                }
            }
            _ => {
                // Keep default fake wallet config for unrecognized backends
            }
        }
        
        // Set service mode
        settings.service_mode = match self.mode.as_str() {
            "MintdOnly" | "mintd_only" => ServiceMode::MintdOnly,
            "Nip74Only" | "nip74_only" => ServiceMode::Nip74Only,
            "MintdAndNip74" | "mintd_and_nip74" => ServiceMode::MintdAndNip74,
            _ => ServiceMode::MintdOnly,
        };
        
        settings
    }

    /// Convert AndroidConfig to JSON string
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| anyhow!("Failed to serialize Android config: {}", e))
    }

    /// Create AndroidConfig from JSON string
    pub fn from_json(json_str: &str) -> Result<Self> {
        serde_json::from_str(json_str)
            .map_err(|e| anyhow!("Failed to parse Android config JSON: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_android_config_json_roundtrip() {
        let config = AndroidConfig::default();
        let json = config.to_json().expect("Failed to serialize");
        let parsed = AndroidConfig::from_json(&json).expect("Failed to parse");
        
        assert_eq!(config.port, parsed.port);
        assert_eq!(config.mode, parsed.mode);
        assert_eq!(config.mint_name, parsed.mint_name);
    }

    #[test]
    fn test_lnbits_config() {
        let mut config = AndroidConfig::default();
        config.lightning_backend = "lnbits".to_string();
        config.lnbits_admin_api_key = Some("admin_key_123".to_string());
        config.lnbits_invoice_api_key = Some("invoice_key_456".to_string());
        config.lnbits_api_url = Some("https://lnbits.example.com".to_string());

        let settings = config.to_settings(None);
        assert_eq!(settings.ln.ln_backend, LnBackend::LNbits);
        assert!(settings.lnbits.is_some());
        
        let lnbits_config = settings.lnbits.unwrap();
        assert_eq!(lnbits_config.admin_api_key, "admin_key_123");
        assert_eq!(lnbits_config.invoice_api_key, "invoice_key_456");
        assert_eq!(lnbits_config.lnbits_api, "https://lnbits.example.com");
    }

    #[test]
    fn test_android_json_parsing() {
        let json_str = r#"{
            "port": 3338,
            "host": "0.0.0.0",
            "mintName": "Test Mint",
            "description": "Test Description",
            "lightningBackend": "lnbits",
            "mode": "mintd_only",
            "databasePath": "/tmp/db",
            "logsPath": "/tmp/logs",
            "lnbitsAdminApiKey": "admin_key_123",
            "lnbitsInvoiceApiKey": "invoice_key_456",
            "lnbitsApiUrl": "https://lnbits.example.com"
        }"#;

        let config = AndroidConfig::from_json(json_str).expect("Failed to parse JSON");
        
        assert_eq!(config.port, 3338);
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.mint_name, "Test Mint");
        assert_eq!(config.description, "Test Description");
        assert_eq!(config.lightning_backend, "lnbits");
        assert_eq!(config.lnbits_admin_api_key, Some("admin_key_123".to_string()));
        assert_eq!(config.lnbits_invoice_api_key, Some("invoice_key_456".to_string()));
        assert_eq!(config.lnbits_api_url, Some("https://lnbits.example.com".to_string()));

        // Test conversion to Settings
        let settings = config.to_settings(None);
        assert_eq!(settings.ln.ln_backend, LnBackend::LNbits);
        assert!(settings.lnbits.is_some());
        assert!(settings.fake_wallet.is_none()); // Should be cleared when using LNBits
    }
} 