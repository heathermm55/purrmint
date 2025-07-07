use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Notify;
use tracing::info;
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
    pub work_dir: PathBuf,
    pub config: Settings,
    nsec: Option<String>,
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
        let config = Self::create_default_config(None);
        
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
            let secret_key = SecretKey::from_bech32(nsec)
                .map_err(|e| anyhow!("Failed to decode nsec: {}", e))?;
            secret_key.to_secret_bytes().to_vec()
        } else {
            hex::decode(nsec)
                .map_err(|e| anyhow!("Failed to decode hex nsec: {}", e))?
        };
        
        if secret_key_bytes.len() != 32 {
            return Err(anyhow!("Invalid nsec length: expected 32 bytes, got {}", secret_key_bytes.len()));
        }
        
        // Generate 64-byte seed using HMAC-SHA512
        let mut hasher = sha2::Sha512::new();
        hasher.update(b"Cashu Mint Seed");
        hasher.update(&secret_key_bytes);
        let seed = hasher.finalize().to_vec();
        
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
            return Ok(());
        }

        info!("Starting MintdService at {:?}", self.work_dir);

        // Create work directory
        std::fs::create_dir_all(&self.work_dir)?;

        // Build and start mint
        let (mint, mint_info) = self.build_mint().await?;
        let mint_arc = Arc::new(mint);

        mint_arc.set_mint_info(mint_info).await?;
        self.mint = Some(mint_arc.clone());
        
        // Initialize mint
        mint_arc.check_pending_mint_quotes().await?;
        mint_arc.check_pending_melt_quotes().await?;
        mint_arc.set_quote_ttl(QuoteTTL::new(10_000, 10_000)).await?;
        
        // Start HTTP server
        self.start_http_server(mint_arc).await?;
        
        self.is_running = true;
        info!("MintdService started successfully");
        Ok(())
    }

    async fn start_http_server(&mut self, mint: Arc<cdk::mint::Mint>) -> Result<()> {
        let listen_addr = self.config.info.listen_host.clone();
        let listen_port = self.config.info.listen_port;
        
        // Create mint router with default cache
        let v1_service = cdk_axum::create_mint_router_with_custom_cache(mint, HttpCache::default()).await?;
        
        let mint_service = Router::new()
            .merge(v1_service)
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
            let _ = axum::serve(listener, mint_service)
                .with_graceful_shutdown(async move {
                    shutdown.notified().await;
                })
                .await;
        });

        self.http_server = Some(http_server);
        Ok(())
    }

    async fn build_mint(&self) -> Result<(cdk::mint::Mint, cdk::nuts::MintInfo)> {
        let database_path = self.work_dir.join("mint.db");
        let database = MintSqliteDatabase::new(database_path).await?;

        let mut mint_builder = MintBuilder::new()
            .with_localstore(Arc::new(database.clone()))
            .with_keystore(Arc::new(database));

        // Configure FakeWallet backend
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

            for unit in &fake_wallet_config.supported_units {
                mint_builder = mint_builder
                    .add_ln_backend(
                        unit.clone(),
                        cdk::nuts::PaymentMethod::Bolt11,
                        MintMeltLimits::new(
                            self.config.ln.min_mint.into(),
                            self.config.ln.max_mint.into(),
                        ),
                        Arc::new(fake_wallet.clone()),
                    )
                    .await?;
            }
        }

        // Set seed from nsec or mnemonic
        let seed = if let Some(ref nsec) = self.nsec {
            Self::generate_seed_from_nsec(nsec)?
        } else if let Some(ref mnemonic) = self.config.info.mnemonic {
            let mnemonic = bip39::Mnemonic::from_str(mnemonic)?;
            mnemonic.to_seed_normalized("").to_vec()
        } else {
            // Default mnemonic for development
            let mnemonic = bip39::Mnemonic::from_str("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about")?;
            mnemonic.to_seed_normalized("").to_vec()
        };
        
        mint_builder = mint_builder.with_seed(seed);

        // Set mint info
        mint_builder = mint_builder
            .with_name(self.config.mint_info.name.clone())
            .with_description(self.config.mint_info.description.clone());

        if let Some(long_description) = &self.config.mint_info.description_long {
            mint_builder = mint_builder.with_long_description(long_description.clone());
        }

        if let Some(pubkey) = self.config.mint_info.pubkey {
            mint_builder = mint_builder.with_pubkey(pubkey);
        }

        if let Some(icon_url) = &self.config.mint_info.icon_url {
            mint_builder = mint_builder.with_icon_url(icon_url.clone());
        }

        if let Some(motd) = &self.config.mint_info.motd {
            mint_builder = mint_builder.with_motd(motd.clone());
        }

        if let Some(tos_url) = &self.config.mint_info.tos_url {
            mint_builder = mint_builder.with_tos_url(tos_url.clone());
        }

        let mint = mint_builder.build().await?;
        mint.set_mint_info(mint_builder.mint_info.clone()).await?;
        
        Ok((mint, mint_builder.mint_info.clone()))
    }

    pub async fn stop(&mut self) -> Result<()> {
        if !self.is_running {
            return Ok(());
        }

        self.shutdown.notify_waiters();

        if let Some(http_server) = self.http_server.take() {
            let _ = http_server.await;
        }

        self.is_running = false;
        info!("MintdService stopped");
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    pub fn get_status(&self) -> Value {
        serde_json::json!({
            "running": self.is_running,
            "server_url": format!("http://{}:{}", self.config.info.listen_host, self.config.info.listen_port),
            "work_dir": self.work_dir.to_string_lossy(),
        })
    }

    // Mint operations
    pub async fn get_mint_quote(&self, amount: u64, unit: &str) -> Result<cdk::nuts::MintQuoteBolt11Response<uuid::Uuid>> {
        let mint = self.mint.as_ref().ok_or_else(|| anyhow!("Mint not available"))?;
        
        let currency_unit = match unit.to_lowercase().as_str() {
            "sat" | "sats" => cdk::nuts::CurrencyUnit::Sat,
            "msat" | "msats" => cdk::nuts::CurrencyUnit::Msat,
            "usd" => cdk::nuts::CurrencyUnit::Usd,
            "eur" => cdk::nuts::CurrencyUnit::Eur,
            _ => return Err(anyhow!("Unsupported currency unit: {}", unit)),
        };

        let request = cdk::nuts::MintQuoteBolt11Request {
            amount: cdk::Amount::from(amount),
            unit: currency_unit,
            description: None,
            pubkey: None,
        };
        
        let quote = mint.get_mint_bolt11_quote(request).await?;
        Ok(quote)
    }

    pub async fn check_mint_quote(&self, quote_id: &str) -> Result<cdk::nuts::MintQuoteBolt11Response<uuid::Uuid>> {
        let mint = self.mint.as_ref().ok_or_else(|| anyhow!("Mint not available"))?;
        let quote_id = Uuid::from_str(quote_id)?;
        let quote = mint.check_mint_quote(&quote_id).await?;
        Ok(quote)
    }

    pub async fn mint_tokens(&self, quote_id: &str, blinded_messages: Vec<cdk::nuts::nut00::BlindedMessage>) -> Result<cdk::nuts::MintResponse> {
        let mint = self.mint.as_ref().ok_or_else(|| anyhow!("Mint not available"))?;
        let quote_uuid = Uuid::from_str(quote_id)?;
        let request = cdk::nuts::MintRequest {
            quote: quote_uuid,
            outputs: blinded_messages,
            signature: None,
        };

        let response = mint.process_mint_request(request).await?;
        Ok(response)
    }

    pub async fn get_melt_quote(&self, amount: u64, unit: &str, invoice: &str) -> Result<cdk::nuts::MeltQuoteBolt11Response<uuid::Uuid>> {
        let mint = self.mint.as_ref().ok_or_else(|| anyhow!("Mint not available"))?;
        
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
    }

    pub async fn check_melt_quote(&self, quote_id: &str) -> Result<cdk::nuts::MeltQuoteBolt11Response<uuid::Uuid>> {
        let mint = self.mint.as_ref().ok_or_else(|| anyhow!("Mint not available"))?;
        let quote_id = Uuid::from_str(quote_id)?;
        let quote = mint.check_melt_quote(&quote_id).await?;
        Ok(quote)
    }

    pub async fn melt_tokens(&self, quote_id: &str, inputs: Vec<cdk::nuts::nut00::Proof>) -> Result<cdk::nuts::MeltQuoteBolt11Response<uuid::Uuid>> {
        let mint = self.mint.as_ref().ok_or_else(|| anyhow!("Mint not available"))?;
        let quote_uuid = Uuid::from_str(quote_id)?;
        let proofs = cdk::nuts::Proofs::from(inputs);
        let request = cdk::nuts::MeltRequest::new(quote_uuid, proofs, None);

        let response = mint.melt_bolt11(&request).await?;
        Ok(response)
    }

    pub async fn swap_tokens(&self, inputs: Vec<cdk::nuts::nut00::Proof>, outputs: Vec<cdk::nuts::nut00::BlindedMessage>) -> Result<cdk::nuts::SwapResponse> {
        let mint = self.mint.as_ref().ok_or_else(|| anyhow!("Mint not available"))?;
        let request = cdk::nuts::SwapRequest::new(inputs, outputs);
        let response = mint.process_swap_request(request).await?;
        Ok(response)
    }

    pub async fn check_proofs(&self, proofs: Vec<cdk::nuts::nut00::Proof>) -> Result<cdk::nuts::CheckStateResponse> {
        let mint = self.mint.as_ref().ok_or_else(|| anyhow!("Mint not available"))?;
        let public_keys: Vec<cdk::nuts::PublicKey> = proofs.iter()
            .filter_map(|proof| proof.y().ok())
            .collect();
        let request = cdk::nuts::CheckStateRequest { ys: public_keys };
        let response = mint.check_state(&request).await?;
        Ok(response)
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
    fn test_generate_seed_from_nsec_hex() {
        let nsec = "0000000000000000000000000000000000000000000000000000000000000001";
        let result = MintdService::generate_seed_from_nsec(nsec);
        
        assert!(result.is_ok());
        let seed = result.unwrap();
        assert_eq!(seed.len(), 64);
        
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
        
        assert_ne!(seed1, seed2);
    }
}