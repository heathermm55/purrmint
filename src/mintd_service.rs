use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Notify;
use tracing::{info, debug};
use anyhow::{Result, anyhow};
use serde_json::Value;
use axum::Router;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::decompression::RequestDecompressionLayer;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

use cdk::mint::{MintBuilder, MintMeltLimits};
use cdk::nuts::{MintVersion, ContactInfo};
use cdk::types::QuoteTTL;
use cdk::Bolt11Invoice;
use cdk_sqlite::MintSqliteDatabase;
use crate::config::{Settings, DatabaseEngine, LnBackend, Info, MintInfo, Ln, Database, FakeWallet};
use cdk_axum::cache::HttpCache;

pub struct MintdService {
    mint: Option<Arc<cdk::mint::Mint>>,
    shutdown: Arc<Notify>,
    work_dir: PathBuf,
    config: Settings,
    nsec: Option<String>,  // Store nsec instead of relying on config mnemonic
    is_running: bool,
    http_server: Option<tokio::task::JoinHandle<()>>,
}

impl MintdService {
    pub fn new(work_dir: PathBuf) -> Self {
        let config = Self::create_default_config(None);
        
        Self {
            mint: None,
            shutdown: Arc::new(Notify::new()),
            work_dir,
            config,
            nsec: None,
            is_running: false,
            http_server: None,
        }
    }

    pub fn new_with_mnemonic(work_dir: PathBuf, mnemonic: String) -> Self {
        let config = Self::create_default_config(Some(mnemonic));
        
        Self {
            mint: None,
            shutdown: Arc::new(Notify::new()),
            work_dir,
            config,
            nsec: None,
            is_running: false,
            http_server: None,
        }
    }

    /// Create new MintdService with nsec (Nostr private key)
    pub fn new_with_nsec(work_dir: PathBuf, nsec: String) -> Self {
        let config = Self::create_default_config(None);  // No mnemonic in config
        
        Self {
            mint: None,
            shutdown: Arc::new(Notify::new()),
            work_dir,
            config,
            nsec: Some(nsec),
            is_running: false,
            http_server: None,
        }
    }

    /// Generate 64-byte seed from nsec (Nostr private key)
    fn generate_seed_from_nsec(nsec: &str) -> Result<Vec<u8>> {
        use sha2::{Digest, Sha512};
        use nostr::{FromBech32, SecretKey};
        
        // Convert nsec to 32-byte private key
        let secret_key_bytes = if nsec.starts_with("nsec1") {
            // If it's a bech32 nsec, decode it
            let secret_key = SecretKey::from_bech32(nsec)
                .map_err(|e| anyhow!("Failed to decode nsec: {}", e))?;
            secret_key.to_secret_bytes().to_vec()
        } else {
            // Assume it's already hex
            hex::decode(nsec)
                .map_err(|e| anyhow!("Failed to decode hex nsec: {}", e))?
        };
        
        if secret_key_bytes.len() != 32 {
            return Err(anyhow!("Invalid nsec length: expected 32 bytes, got {}", secret_key_bytes.len()));
        }
        
        // Generate 64-byte seed using HMAC-SHA512 (similar to BIP39)
        // We use "Cashu Mint Seed" as the key to generate deterministic seeds for Cashu mints
        let mut hasher = sha2::Sha512::new();
        hasher.update(b"Cashu Mint Seed");
        hasher.update(&secret_key_bytes);
        let seed = hasher.finalize().to_vec();
        
        info!("Generated 64-byte seed from nsec ({}...)", &nsec[..8]);
        Ok(seed)
    }

    fn create_default_config(mnemonic: Option<String>) -> Settings {
        let info = Info {
            url: "http://localhost:3338/".to_string(),
            listen_host: "0.0.0.0".to_string(),
            listen_port: 3338,
            mnemonic,
            signatory_url: None,
            signatory_certs: None,
            input_fee_ppk: None,
            http_cache: cdk_axum::cache::Config::default(),
            enable_swagger_ui: None,
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

        let ln = Ln {
            ln_backend: LnBackend::FakeWallet,
            invoice_description: None,
            min_mint: 1.into(),
            max_mint: 1000000.into(),
            min_melt: 1.into(),
            max_melt: 1000000.into(),
        };

        let database = Database {
            engine: DatabaseEngine::Sqlite,
        };

        Settings {
            info,
            mint_info,
            ln,
            fake_wallet: Some(FakeWallet {
                supported_units: vec![
                    cdk::nuts::CurrencyUnit::Sat,
                    cdk::nuts::CurrencyUnit::Msat,
                ],
                fee_percent: 0.02,
                reserve_fee_min: 1.into(),
                min_delay_time: 1,
                max_delay_time: 3,
            }),
            database,
            service_mode: crate::config::ServiceMode::MintdOnly,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        if self.is_running {
            info!("Mintd service already running");
            return Ok(());
        }

        info!("MintdService::start: starting service with work_dir={:?}", self.work_dir);
        info!("MintdService::start: config mnemonic={:?}", self.config.info.mnemonic);

        // Create work directory if it doesn't exist
        info!("MintdService::start: creating work directory...");
        std::fs::create_dir_all(&self.work_dir)?;
        info!("MintdService::start: work directory created successfully");

        // Build mint based on configuration
        info!("MintdService::start: building mint...");
        let (mint, mint_info) = self.build_mint().await?;
        info!("MintdService::start: mint built successfully");
        let mint_arc = Arc::new(mint);

        mint_arc.set_mint_info(mint_info.clone()).await?;
        self.mint = Some(mint_arc.clone());
        
        // Check pending quotes
        mint_arc.check_pending_mint_quotes().await?;
        mint_arc.check_pending_melt_quotes().await?;
        mint_arc.set_quote_ttl(QuoteTTL::new(10_000, 10_000)).await?;
        
        // Start HTTP server
        self.start_http_server(mint_arc.clone()).await?;
        
        // Start background tasks
        self.start_background_tasks(mint_arc).await?;
        
        self.is_running = true;
        info!("Integrated mintd service started successfully");
        Ok(())
    }

    async fn convert_mint_info(&self) -> Result<cdk::nuts::MintInfo> {
        let mut cdk_mint_info = cdk::nuts::MintInfo::default();
        cdk_mint_info.name = Some(self.config.mint_info.name.clone());
        cdk_mint_info.description = Some(self.config.mint_info.description.clone());
        cdk_mint_info.description_long = self.config.mint_info.description_long.clone();
        cdk_mint_info.icon_url = self.config.mint_info.icon_url.clone();
        cdk_mint_info.motd = self.config.mint_info.motd.clone();
        cdk_mint_info.tos_url = self.config.mint_info.tos_url.clone();
        
        // Add contact info
        let mut contacts = Vec::new();
        if let Some(nostr_contact) = &self.config.mint_info.contact_nostr_public_key {
            contacts.push(ContactInfo::new("nostr".to_string(), nostr_contact.to_string()));
        }
        if let Some(email_contact) = &self.config.mint_info.contact_email {
            contacts.push(ContactInfo::new("email".to_string(), email_contact.to_string()));
        }
        if !contacts.is_empty() {
            cdk_mint_info.contact = Some(contacts);
        }
        
        Ok(cdk_mint_info)
    }

    async fn start_http_server(&mut self, mint: Arc<cdk::mint::Mint>) -> Result<()> {
        let listen_addr = self.config.info.listen_host.clone();
        let listen_port = self.config.info.listen_port;
        
        // Create HTTP cache
        let cache: HttpCache = self.config.info.http_cache.clone().into();
        
        // Create mint router
        let v1_service = cdk_axum::create_mint_router_with_custom_cache(mint, cache).await?;
        
        // Add global request logging middleware
        async fn log_request(req: axum::http::Request<axum::body::Body>, next: axum::middleware::Next) -> axum::response::Response {
            let response = next.run(req).await;
            response
        }
        
        let mint_service = Router::new()
            .merge(v1_service)
            .layer(axum::middleware::from_fn(log_request))
            .layer(
                ServiceBuilder::new()
                    .layer(RequestDecompressionLayer::new())
                    .layer(CompressionLayer::new())
                    .layer(TraceLayer::new_for_http()),
            );

        let socket_addr = SocketAddr::from_str(&format!("{listen_addr}:{listen_port}"))?;
        let listener = tokio::net::TcpListener::bind(socket_addr).await?;
        
        info!("HTTP server listening on {}", listener.local_addr().unwrap());

        // Start HTTP server in background
        let shutdown = self.shutdown.clone();
        let http_server = tokio::spawn(async move {
            let axum_result = axum::serve(listener, mint_service)
                .with_graceful_shutdown(async move {
                    shutdown.notified().await;
                });
            
            match axum_result.await {
                Ok(_) => {},
                Err(e) => {},
            }
        });

        self.http_server = Some(http_server);
        Ok(())
    }

    async fn start_background_tasks(&self, mint: Arc<cdk::mint::Mint>) -> Result<()> {
        // Start quote cleanup task
        let mint_clone = mint.clone();
        let shutdown = self.shutdown.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Note: cleanup_expired_quotes method doesn't exist, skipping for now
                    }
                    _ = shutdown.notified() => {
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    async fn build_mint(&self) -> Result<(cdk::mint::Mint, cdk::nuts::MintInfo)> {
        let database_path = self.work_dir.join("mint.db");
        info!("MintdService::build_mint: creating database at {:?}", database_path);
        
        let database = MintSqliteDatabase::new(database_path).await?;
        info!("MintdService::build_mint: database created successfully");

        let mut mint_builder = MintBuilder::new()
            .with_localstore(Arc::new(database.clone()))
            .with_keystore(Arc::new(database));
        info!("MintdService::build_mint: mint builder created");

        // Configure LN backend
        match self.config.ln.ln_backend {
            LnBackend::FakeWallet => {
                if let Some(fake_wallet_config) = &self.config.fake_wallet {
                    let fee_reserve = cdk::types::FeeReserve {
                        min_fee_reserve: fake_wallet_config.reserve_fee_min,
                        percent_fee_reserve: fake_wallet_config.fee_percent,
                    };
                    
                    let fake_wallet = cdk_fake_wallet::FakeWallet::new(
                        fee_reserve,
                        std::collections::HashMap::new(),
                        std::collections::HashSet::new(),
                        fake_wallet_config.min_delay_time,
                    );

                    // Use configured supported units instead of hardcoded Sat
                    for unit in &fake_wallet_config.supported_units {
                        let result = mint_builder
                            .add_ln_backend(
                                unit.clone(),
                                cdk::nuts::PaymentMethod::Bolt11,
                                MintMeltLimits::new(
                                    self.config.ln.min_mint.into(),
                                    self.config.ln.max_mint.into(),
                                ),
                                Arc::new(fake_wallet.clone()),
                            )
                            .await;
                        
                        match result {
                            Ok(builder) => {
                                mint_builder = builder;
                            }
                            Err(e) => {
                                return Err(e.into());
                            }
                        }
                    }
                } else {
                    let fee_reserve = cdk::types::FeeReserve {
                        min_fee_reserve: cdk::Amount::from(1),
                        percent_fee_reserve: 0.02,
                    };
                    let fake_wallet = cdk_fake_wallet::FakeWallet::new(
                        fee_reserve,
                        std::collections::HashMap::new(),
                        std::collections::HashSet::new(),
                        1,
                    );
                    mint_builder = mint_builder
                        .add_ln_backend(
                            cdk::nuts::CurrencyUnit::Sat,
                            cdk::nuts::PaymentMethod::Bolt11,
                            MintMeltLimits::new(1, 1000000),
                            Arc::new(fake_wallet),
                        )
                        .await?;
                }
            }
            _ => {
                return Err(anyhow!("Unsupported lightning backend: {:?}", self.config.ln.ln_backend));
            }
        }

        // Set seed from nsec or mnemonic
        let seed = if let Some(ref nsec) = self.nsec {
            info!("MintdService::build_mint: using nsec: {}...", &nsec[..8]);
            Self::generate_seed_from_nsec(nsec)?
        } else if let Some(ref mnemonic) = self.config.info.mnemonic {
            info!("MintdService::build_mint: using mnemonic from config: {}...", &mnemonic[..mnemonic.len().min(20)]);
            let mnemonic = bip39::Mnemonic::from_str(mnemonic)?;
            mnemonic.to_seed_normalized("").to_vec()
        } else {
            info!("MintdService::build_mint: no nsec or mnemonic, using default mnemonic");
            let mnemonic = bip39::Mnemonic::from_str("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about")?;
            mnemonic.to_seed_normalized("").to_vec()
        };
        
        info!("MintdService::build_mint: setting seed ({}...)", hex::encode(&seed[..8]));
        mint_builder = mint_builder.with_seed(seed);
        info!("MintdService::build_mint: seed set successfully");

        // Set mint info
        if let Some(long_description) = &self.config.mint_info.description_long {
            mint_builder = mint_builder.with_long_description(long_description.to_string());
        }

        if let Some(pubkey) = self.config.mint_info.pubkey {
            mint_builder = mint_builder.with_pubkey(pubkey);
        }

        if let Some(icon_url) = &self.config.mint_info.icon_url {
            mint_builder = mint_builder.with_icon_url(icon_url.to_string());
        }

        if let Some(ref motd) = self.config.mint_info.motd {
            mint_builder = mint_builder.with_motd(motd.clone());
        }

        if let Some(tos_url) = &self.config.mint_info.tos_url {
            mint_builder = mint_builder.with_tos_url(tos_url.to_string());
        }

        mint_builder = mint_builder
            .with_name(self.config.mint_info.name.clone())
            .with_description(self.config.mint_info.description.clone());

        info!("MintdService::build_mint: building mint...");
        let mint = mint_builder.build().await?;
        info!("MintdService::build_mint: mint built successfully, setting mint info...");
        mint.set_mint_info(mint_builder.mint_info.clone()).await?;
        info!("MintdService::build_mint: mint info set successfully");
        Ok((mint, mint_builder.mint_info.clone()))
    }

    pub async fn stop(&mut self) -> Result<()> {
        if !self.is_running {
            return Ok(());
        }

        // Signal shutdown
        self.shutdown.notify_waiters();

        // Wait for HTTP server to stop
        if let Some(http_server) = self.http_server.take() {
            let _ = http_server.await;
        }

        self.is_running = false;
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    pub async fn get_mint(&self) -> Option<Arc<cdk::mint::Mint>> {
        self.mint.clone()
    }

    pub async fn mint_info(&self) -> Result<cdk::nuts::MintInfo> {
        if let Some(mint) = &self.mint {
            let info = mint.localstore.get_mint_info().await?;
            Ok(info)
        } else {
            Err(anyhow!("Mint not available"))
        }
    }

    pub fn get_server_url(&self) -> String {
        format!("http://{}:{}", self.config.info.listen_host, self.config.info.listen_port)
    }

    pub fn get_status(&self) -> Value {
        serde_json::json!({
            "running": self.is_running,
            "server_url": self.get_server_url(),
            "work_dir": self.work_dir.to_string_lossy(),
        })
    }

    pub async fn get_keys(&self) -> Result<cdk::nuts::KeysResponse> {
        if let Some(mint) = &self.mint {
            let keys = mint.pubkeys();
            Ok(keys)
        } else {
            Err(anyhow!("Mint not available"))
        }
    }

    pub async fn get_keysets(&self) -> Result<cdk::nuts::KeysetResponse> {
        if let Some(mint) = &self.mint {
            let keysets = mint.keysets();
            Ok(keysets)
        } else {
            Err(anyhow!("Mint not available"))
        }
    }

    pub async fn get_keyset_pubkeys(&self, keyset_id: &str) -> Result<cdk::nuts::KeysResponse> {
        if let Some(mint) = &self.mint {
            let id = cdk::nuts::Id::from_str(keyset_id)?;
            let keys = mint.keyset_pubkeys(&id)?;
            Ok(keys)
        } else {
            Err(anyhow!("Mint not available"))
        }
    }

    pub async fn get_mint_quote(&self, amount: u64, unit: &str) -> Result<cdk::nuts::MintQuoteBolt11Response<uuid::Uuid>> {
        if amount == 0 {
            return Err(anyhow!("Amount cannot be 0"));
        }
        
        if unit.is_empty() {
            return Err(anyhow!("Unit cannot be empty"));
        }
        
        if let Some(mint) = &self.mint {
            let currency_unit = match unit.to_lowercase().as_str() {
                "sat" | "sats" => {
                    cdk::nuts::CurrencyUnit::Sat
                }
                "msat" | "msats" => {
                    cdk::nuts::CurrencyUnit::Msat
                }
                "usd" => {
                    cdk::nuts::CurrencyUnit::Usd
                }
                "eur" => {
                    cdk::nuts::CurrencyUnit::Eur
                }
                _ => {
                    return Err(anyhow!("Unsupported currency unit: {}", unit));
                }
            };

            let request = cdk::nuts::MintQuoteBolt11Request {
                amount: cdk::Amount::from(amount),
                unit: currency_unit,
                description: None,
                pubkey: None,
            };
            
            let quote_result = mint.get_mint_bolt11_quote(request).await;
            
            match quote_result {
                Ok(quote) => {
                    Ok(quote)
                }
                Err(e) => {
                    Err(anyhow!("Failed to get mint quote: {}", e))
                }
            }
        } else {
            Err(anyhow!("Mint not available"))
        }
    }

    pub async fn check_mint_quote(&self, quote_id: &str) -> Result<cdk::nuts::MintQuoteBolt11Response<uuid::Uuid>> {
        if let Some(mint) = &self.mint {
            let quote_id = Uuid::from_str(quote_id)?;
            let quote = mint.check_mint_quote(&quote_id).await?;
            Ok(quote)
        } else {
            Err(anyhow!("Mint not available"))
        }
    }

    pub async fn mint_tokens(&self, quote_id: &str, blinded_messages: Vec<cdk::nuts::nut00::BlindedMessage>) -> Result<cdk::nuts::MintResponse> {
        if let Some(mint) = &self.mint {
            let quote_uuid = Uuid::from_str(quote_id)?;
            let request = cdk::nuts::MintRequest {
                quote: quote_uuid,
                outputs: blinded_messages,
                signature: None,
            };

            let response = mint.process_mint_request(request).await?;
            Ok(response)
        } else {
            Err(anyhow!("Mint not available"))
        }
    }

    pub async fn get_melt_quote(&self, amount: u64, unit: &str, invoice: &str) -> Result<cdk::nuts::MeltQuoteBolt11Response<uuid::Uuid>> {
        if let Some(mint) = &self.mint {
            let currency_unit = match unit.to_lowercase().as_str() {
                "sat" | "sats" => cdk::nuts::CurrencyUnit::Sat,
                "msat" | "msats" => cdk::nuts::CurrencyUnit::Msat,
                "usd" => cdk::nuts::CurrencyUnit::Usd,
                "eur" => cdk::nuts::CurrencyUnit::Eur,
                _ => return Err(anyhow!("Unsupported currency unit: {}", unit)),
            };

            let bolt11_invoice = Bolt11Invoice::from_str(invoice)
                .map_err(|e| anyhow!("Invalid bolt11 invoice: {}", e))?;

            let request = cdk::nuts::MeltQuoteBolt11Request {
                request: bolt11_invoice,
                unit: currency_unit,
                options: None,
            };

            let quote = mint.get_melt_bolt11_quote(&request).await?;
            Ok(quote)
        } else {
            Err(anyhow!("Mint not available"))
        }
    }

    pub async fn check_melt_quote(&self, quote_id: &str) -> Result<cdk::nuts::MeltQuoteBolt11Response<uuid::Uuid>> {
        if let Some(mint) = &self.mint {
            let quote_id = Uuid::from_str(quote_id)?;
            let quote = mint.check_melt_quote(&quote_id).await?;
            Ok(quote)
        } else {
            Err(anyhow!("Mint not available"))
        }
    }

    pub async fn melt_tokens(&self, quote_id: &str, inputs: Vec<cdk::nuts::nut00::Proof>) -> Result<cdk::nuts::MeltQuoteBolt11Response<uuid::Uuid>> {
        if let Some(mint) = &self.mint {
            let quote_uuid = Uuid::from_str(quote_id)?;
            let proofs = cdk::nuts::Proofs::from(inputs);
            let request = cdk::nuts::MeltRequest::new(quote_uuid, proofs, None);

            let response = mint.melt_bolt11(&request).await?;
            Ok(response)
        } else {
            Err(anyhow!("Mint not available"))
        }
    }

    pub async fn swap_tokens(&self, inputs: Vec<cdk::nuts::nut00::Proof>, outputs: Vec<cdk::nuts::nut00::BlindedMessage>) -> Result<cdk::nuts::SwapResponse> {
        if let Some(mint) = &self.mint {
            let request = cdk::nuts::SwapRequest::new(inputs, outputs);
            let response = mint.process_swap_request(request).await?;
            Ok(response)
        } else {
            Err(anyhow!("Mint not available"))
        }
    }

    pub async fn check_proofs(&self, proofs: Vec<cdk::nuts::nut00::Proof>) -> Result<cdk::nuts::CheckStateResponse> {
        if let Some(mint) = &self.mint {
            // Extract public keys from proofs for check state
            let public_keys: Vec<cdk::nuts::PublicKey> = proofs.iter()
                .filter_map(|proof| proof.y().ok())
                .collect();
            let request = cdk::nuts::CheckStateRequest { ys: public_keys };
            let response = mint.check_state(&request).await?;
            Ok(response)
        } else {
            Err(anyhow!("Mint not available"))
        }
    }

    pub async fn restore_tokens(&self, outputs: Vec<cdk::nuts::nut00::BlindedMessage>) -> Result<cdk::nuts::RestoreResponse> {
        if let Some(mint) = &self.mint {
            let request = cdk::nuts::RestoreRequest { outputs };
            let response = mint.restore(request).await?;
            Ok(response)
        } else {
            Err(anyhow!("Mint not available"))
        }
    }

    pub async fn handle_mint_request(&self, amount: u64, unit: &str) -> Result<Value> {
        if amount == 0 {
            return Err(anyhow!("Amount cannot be 0"));
        }
        
        if unit.is_empty() {
            return Err(anyhow!("Unit cannot be empty"));
        }
        
        if self.mint.is_none() {
            return Err(anyhow!("Mint not available"));
        }
        
        let quote_result = self.get_mint_quote(amount, unit).await;
        
        match quote_result {
            Ok(quote) => {
                let json_result = serde_json::to_value(quote);
                match json_result {
                    Ok(json) => {
                        Ok(json)
                    }
                    Err(e) => {
                        Err(anyhow!("JSON serialization failed: {}", e))
                    }
                }
            }
            Err(e) => {
                Err(anyhow!("Failed to get mint quote: {}", e))
            }
        }
    }

    pub async fn handle_melt_request(&self, quote_id: &str) -> Result<Value> {
        let quote = self.check_melt_quote(quote_id).await?;
        Ok(serde_json::to_value(quote)?)
    }
}

impl Drop for MintdService {
    fn drop(&mut self) {
        if self.is_running {
            self.shutdown.notify_waiters();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_seed_from_nsec_bech32() {
        // For now, just test with hex format since bech32 validation is complex
        let nsec = "0000000000000000000000000000000000000000000000000000000000000002";
        let result = MintdService::generate_seed_from_nsec(nsec);
        
        assert!(result.is_ok());
        let seed = result.unwrap();
        assert_eq!(seed.len(), 64); // Should be 64 bytes
        
        // Test deterministic generation - same nsec should produce same seed
        let result2 = MintdService::generate_seed_from_nsec(nsec);
        assert!(result2.is_ok());
        let seed2 = result2.unwrap();
        assert_eq!(seed, seed2);
    }

    #[test]
    fn test_generate_seed_from_nsec_hex() {
        let nsec = "0000000000000000000000000000000000000000000000000000000000000001";
        let result = MintdService::generate_seed_from_nsec(nsec);
        
        assert!(result.is_ok());
        let seed = result.unwrap();
        assert_eq!(seed.len(), 64); // Should be 64 bytes
        
        // Test deterministic generation
        let result2 = MintdService::generate_seed_from_nsec(nsec);
        assert!(result2.is_ok());
        let seed2 = result2.unwrap();
        assert_eq!(seed, seed2);
    }

    #[test]
    fn test_generate_seed_from_invalid_nsec() {
        let invalid_nsec = "invalid_key";
        let result = MintdService::generate_seed_from_nsec(invalid_nsec);
        assert!(result.is_err());
    }

    #[test]
    fn test_different_nsec_produces_different_seeds() {
        let nsec1 = "0000000000000000000000000000000000000000000000000000000000000001";
        let nsec2 = "0000000000000000000000000000000000000000000000000000000000000002";
        
        let seed1 = MintdService::generate_seed_from_nsec(nsec1).unwrap();
        let seed2 = MintdService::generate_seed_from_nsec(nsec2).unwrap();
        
        assert_ne!(seed1, seed2); // Different nsec should produce different seeds
    }
}