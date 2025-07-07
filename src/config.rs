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
    pub database: Database,
    pub service_mode: ServiceMode,
}

// =============================================================================
// Android Configuration
// =============================================================================

/// Android-specific configuration (simplified JSON format for JNI)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AndroidConfig {
    pub port: u16,
    pub host: String,
    pub mint_name: String,
    pub description: String,
    pub lightning_backend: String,
    pub mode: String,
    pub database_path: String,
    pub logs_path: String,
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
            _ => LnBackend::None,
        };
        
        // Set service mode
        settings.service_mode = match self.mode.as_str() {
            "MintdOnly" | "mintd_only" => ServiceMode::MintdOnly,
            "Nip74Only" | "nip74_only" => ServiceMode::Nip74Only,
            "MintdAndNip74" | "mintd_and_nip74" => ServiceMode::MintdAndNip74,
            _ => ServiceMode::MintdOnly,
        };
        
        settings
    }

    /// Create AndroidConfig from JSON string
    pub fn from_json(json_str: &str) -> Result<Self> {
        serde_json::from_str(json_str)
            .map_err(|e| anyhow!("Failed to parse Android config JSON: {}", e))
    }

    /// Convert AndroidConfig to JSON string
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| anyhow!("Failed to serialize Android config: {}", e))
    }

    /// Update configuration with validation
    pub fn update_from_json(&mut self, json_str: &str) -> Result<()> {
        let update: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| anyhow!("Invalid JSON format: {}", e))?;
        
        // Validate and update fields
        if let Some(port) = update.get("port").and_then(|p| p.as_u64()) {
            if port > 0 && port < 65536 {
                self.port = port as u16;
            }
        }
        
        if let Some(host) = update.get("host").and_then(|h| h.as_str()) {
            if !host.is_empty() {
                self.host = host.to_string();
            }
        }
        
        if let Some(mint_name) = update.get("mintName").and_then(|n| n.as_str()) {
            if !mint_name.is_empty() {
                self.mint_name = mint_name.to_string();
            }
        }
        
        if let Some(description) = update.get("description").and_then(|d| d.as_str()) {
            self.description = description.to_string();
        }
        
        if let Some(lightning_backend) = update.get("lightningBackend").and_then(|l| l.as_str()) {
            self.lightning_backend = lightning_backend.to_string();
        }
        
        if let Some(mode) = update.get("mode").and_then(|m| m.as_str()) {
            self.mode = mode.to_string();
        }
        
        if let Some(database_path) = update.get("databasePath").and_then(|d| d.as_str()) {
            self.database_path = database_path.to_string();
        }
        
        if let Some(logs_path) = update.get("logsPath").and_then(|l| l.as_str()) {
            self.logs_path = logs_path.to_string();
        }
        
        Ok(())
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
} 