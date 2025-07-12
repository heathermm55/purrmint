// Path import removed - not needed for basic Android functionality
use cdk::nuts::{CurrencyUnit, PublicKey};
use cdk::Amount;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};

// =============================================================================
// Tor Configuration
// =============================================================================

/// Tor startup mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TorStartupMode {
    /// Disable Tor completely
    Disabled,
    /// Use system Tor (if available)
    System,
    /// Use embedded Arti Tor client
    Embedded,
    /// Use embedded Arti with custom configuration
    Custom,
}

impl Default for TorStartupMode {
    fn default() -> Self {
        TorStartupMode::Disabled
    }
}

/// Tor configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorConfig {
    /// Tor startup mode
    pub startup_mode: TorStartupMode,
    /// Enable hidden services
    pub enable_hidden_services: bool,
    /// Number of introduction points for hidden services
    pub num_intro_points: u32,
    /// Tor data directory
    pub data_dir: Option<String>,
    /// Tor socks port
    pub socks_port: Option<u16>,
    /// Tor control port
    pub control_port: Option<u16>,
    /// Bridge configuration
    pub bridges: Vec<String>,
    /// Enable bridge mode
    pub use_bridges: bool,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    /// Enable logging
    pub enable_logging: bool,
    /// Log level
    pub log_level: String,
}

impl Default for TorConfig {
    fn default() -> Self {
        Self {
            startup_mode: TorStartupMode::Disabled,
            enable_hidden_services: false,
            num_intro_points: 3,
            data_dir: None,
            socks_port: None,
            control_port: None,
            bridges: Vec::new(),
            use_bridges: false,
            connection_timeout: 60,
            enable_logging: true,
            log_level: "info".to_string(),
        }
    }
}

impl TorConfig {
    /// Create a new Tor configuration with embedded mode
    pub fn embedded() -> Self {
        Self {
            startup_mode: TorStartupMode::Embedded,
            enable_hidden_services: true,
            ..Default::default()
        }
    }

    /// Create a new Tor configuration with system mode
    pub fn system() -> Self {
        Self {
            startup_mode: TorStartupMode::System,
            enable_hidden_services: true,
            ..Default::default()
        }
    }

    /// Create a new Tor configuration with custom settings
    pub fn custom(
        data_dir: String,
        socks_port: u16,
        enable_hidden_services: bool,
    ) -> Self {
        Self {
            startup_mode: TorStartupMode::Custom,
            enable_hidden_services,
            data_dir: Some(data_dir),
            socks_port: Some(socks_port),
            ..Default::default()
        }
    }

    /// Check if Tor is enabled
    pub fn is_enabled(&self) -> bool {
        self.startup_mode != TorStartupMode::Disabled
    }

    /// Check if hidden services are enabled
    pub fn hidden_services_enabled(&self) -> bool {
        self.is_enabled() && self.enable_hidden_services
    }

    /// Get the data directory path
    pub fn get_data_dir(&self) -> Option<String> {
        self.data_dir.clone().or_else(|| {
            if self.is_enabled() {
                Some("tor_data".to_string())
            } else {
                None
            }
        })
    }

    /// Get the socks port
    pub fn get_socks_port(&self) -> u16 {
        self.socks_port.unwrap_or(9050)
    }

    /// Get the control port
    pub fn get_control_port(&self) -> Option<u16> {
        self.control_port
    }
}

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
    pub tor: TorConfig,
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
    // Tor configuration
    pub tor_enabled: Option<bool>,
    pub tor_mode: Option<String>,
    pub tor_data_dir: Option<String>,
    pub tor_socks_port: Option<u16>,
    pub tor_enable_hidden_services: Option<bool>,
    pub tor_num_intro_points: Option<u32>,
    pub tor_bridges: Option<Vec<String>>,
    pub tor_use_bridges: Option<bool>,
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
            // Tor defaults
            tor_enabled: Some(false),
            tor_mode: Some("disabled".to_string()),
            tor_data_dir: None,
            tor_socks_port: None,
            tor_enable_hidden_services: Some(false),
            tor_num_intro_points: Some(3),
            tor_bridges: None,
            tor_use_bridges: Some(false),
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
        let tor = TorConfig::default();

        Settings {
            info,
            mint_info,
            ln,
            fake_wallet: Some(FakeWallet::default()),
            lnbits: None,
            cln: None,
            database,
            service_mode: ServiceMode::default(),
            tor,
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

        // Set Tor configuration
        settings.tor = self.to_tor_config();
        
        settings
    }

    /// Convert AndroidConfig to TorConfig
    pub fn to_tor_config(&self) -> TorConfig {
        let startup_mode = if let Some(enabled) = self.tor_enabled {
            if !enabled {
                TorStartupMode::Disabled
            } else {
                match self.tor_mode.as_deref() {
                    Some("system") => TorStartupMode::System,
                    Some("embedded") => TorStartupMode::Embedded,
                    Some("custom") => TorStartupMode::Custom,
                    _ => TorStartupMode::Embedded, // Default to embedded if enabled
                }
            }
        } else {
            TorStartupMode::Disabled
        };

        TorConfig {
            startup_mode,
            enable_hidden_services: self.tor_enable_hidden_services.unwrap_or(false),
            num_intro_points: self.tor_num_intro_points.unwrap_or(3),
            data_dir: self.tor_data_dir.clone(),
            socks_port: self.tor_socks_port,
            control_port: None, // Not exposed in Android config
            bridges: self.tor_bridges.clone().unwrap_or_default(),
            use_bridges: self.tor_use_bridges.unwrap_or(false),
            connection_timeout: 60,
            enable_logging: true,
            log_level: "info".to_string(),
        }
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

    #[test]
    fn test_tor_config() {
        let mut config = AndroidConfig::default();
        config.tor_enabled = Some(true);
        config.tor_mode = Some("embedded".to_string());
        config.tor_enable_hidden_services = Some(true);
        config.tor_data_dir = Some("/tmp/tor_data".to_string());
        config.tor_socks_port = Some(9050);

        let tor_config = config.to_tor_config();
        assert_eq!(tor_config.startup_mode, TorStartupMode::Embedded);
        assert!(tor_config.enable_hidden_services);
        assert_eq!(tor_config.data_dir, Some("/tmp/tor_data".to_string()));
        assert_eq!(tor_config.socks_port, Some(9050));
    }

    #[test]
    fn test_tor_disabled() {
        let mut config = AndroidConfig::default();
        config.tor_enabled = Some(false);

        let tor_config = config.to_tor_config();
        assert_eq!(tor_config.startup_mode, TorStartupMode::Disabled);
        assert!(!tor_config.is_enabled());
    }


} 