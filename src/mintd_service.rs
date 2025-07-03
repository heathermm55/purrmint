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

use cdk::mint::{MintBuilder, MintMeltLimits};
use cdk::nuts::{MintVersion, ContactInfo};
use cdk::types::QuoteTTL;
use cdk_sqlite::MintSqliteDatabase;
use cdk_mintd::config::{Settings, DatabaseEngine, LnBackend, Info, MintInfo, Ln};
use cdk_axum::cache::HttpCache;

pub struct MintdService {
    mint: Option<Arc<cdk::mint::Mint>>,
    shutdown: Arc<Notify>,
    work_dir: PathBuf,
    config: Settings,
    is_running: bool,
    http_server: Option<tokio::task::JoinHandle<()>>,
}

impl MintdService {
    pub fn new(work_dir: PathBuf) -> Self {
        let config = Self::create_default_config();
        
        Self {
            mint: None,
            shutdown: Arc::new(Notify::new()),
            work_dir,
            config,
            is_running: false,
            http_server: None,
        }
    }

    fn create_default_config() -> Settings {
        let info = Info {
            url: "http://127.0.0.1:3338/".to_string(),
            listen_host: "127.0.0.1".to_string(),
            listen_port: 3338,
            mnemonic: None,
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

        let database = cdk_mintd::config::Database {
            engine: DatabaseEngine::Sqlite,
        };

        Settings {
            info,
            mint_info,
            ln,
            cln: None,
            lnbits: None,
            lnd: None,
            fake_wallet: Some(cdk_mintd::config::FakeWallet {
                supported_units: vec![cdk::nuts::CurrencyUnit::Sat],
                fee_percent: 0.02,
                reserve_fee_min: 1.into(),
                min_delay_time: 1,
                max_delay_time: 3,
            }),
            grpc_processor: None,
            database,
            mint_management_rpc: None,
            auth: None,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        if self.is_running {
            info!("Mintd service already running");
            return Ok(());
        }

        info!("Starting integrated mintd service");
        debug!("Work directory: {:?}", self.work_dir);

        // Create work directory if it doesn't exist
        std::fs::create_dir_all(&self.work_dir)?;

        // Build mint based on configuration
        let mint = self.build_mint().await?;
        
        // Check pending quotes
        mint.check_pending_mint_quotes().await?;
        mint.check_pending_melt_quotes().await?;

        // Convert config MintInfo to cdk::nuts::MintInfo for setting
        let cdk_mint_info = self.convert_mint_info().await?;
        mint.set_mint_info(cdk_mint_info).await?;
        mint.set_quote_ttl(QuoteTTL::new(10_000, 10_000)).await?;

        let mint_arc = Arc::new(mint);
        self.mint = Some(mint_arc.clone());

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
            let axum_result = axum::serve(listener, mint_service)
                .with_graceful_shutdown(async move {
                    shutdown.notified().await;
                });
            
            match axum_result.await {
                Ok(_) => {
                    info!("HTTP server stopped gracefully");
                }
                Err(err) => {
                    tracing::error!("HTTP server stopped with error: {}", err);
                }
            }
        });

        self.http_server = Some(http_server);
        Ok(())
    }

    async fn start_background_tasks(&self, mint: Arc<cdk::mint::Mint>) -> Result<()> {
        // Start background task for checking paid invoices
        let shutdown = self.shutdown.clone();
        let mint_clone = mint.clone();
        
        tokio::spawn(async move {
            mint_clone.wait_for_paid_invoices(shutdown).await;
        });

        info!("Background tasks started");
        Ok(())
    }

    async fn build_mint(&self) -> Result<cdk::mint::Mint> {
        let mut mint_builder = match self.config.database.engine {
            DatabaseEngine::Sqlite => {
                let sql_db_path = self.work_dir.join("cdk-mintd.sqlite");
                let sqlite_db = MintSqliteDatabase::new(&sql_db_path).await?;
                let db = Arc::new(sqlite_db);
                
                MintBuilder::new()
                    .with_localstore(db.clone())
                    .with_keystore(db)
            }
        };

        // Add contact info if available
        if let Some(nostr_contact) = &self.config.mint_info.contact_nostr_public_key {
            let nostr_contact = ContactInfo::new("nostr".to_string(), nostr_contact.to_string());
            mint_builder = mint_builder.add_contact_info(nostr_contact);
        }

        if let Some(email_contact) = &self.config.mint_info.contact_email {
            let email_contact = ContactInfo::new("email".to_string(), email_contact.to_string());
            mint_builder = mint_builder.add_contact_info(email_contact);
        }

        // Set mint version
        let mint_version = MintVersion::new(
            "purrmint".to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        );

        // Configure lightning backend
        let mint_melt_limits = MintMeltLimits {
            mint_min: self.config.ln.min_mint,
            mint_max: self.config.ln.max_mint,
            melt_min: self.config.ln.min_melt,
            melt_max: self.config.ln.max_melt,
        };

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

                    mint_builder = mint_builder
                        .add_ln_backend(
                            cdk::nuts::CurrencyUnit::Sat,
                            cdk::nuts::PaymentMethod::Bolt11,
                            mint_melt_limits,
                            Arc::new(fake_wallet),
                        )
                        .await?;

                    if let Some(input_fee) = self.config.info.input_fee_ppk {
                        mint_builder = mint_builder.set_unit_fee(&cdk::nuts::CurrencyUnit::Sat, input_fee)?;
                    }
                }
            }
            _ => {
                return Err(anyhow!("Unsupported lightning backend: {:?}", self.config.ln.ln_backend));
            }
        }

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
            .with_version(mint_version)
            .with_description(self.config.mint_info.description.clone());

        // Set seed (for now using a default seed, in production should be configurable)
        let default_mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let mnemonic = bip39::Mnemonic::from_str(default_mnemonic)?;
        mint_builder = mint_builder.with_seed(mnemonic.to_seed_normalized("").to_vec());

        // Build the mint
        let mint = mint_builder.build().await?;
        Ok(mint)
    }

    pub async fn stop(&mut self) -> Result<()> {
        if !self.is_running {
            info!("Mintd service not running");
            return Ok(());
        }

        info!("Stopping integrated mintd service");

        // Signal shutdown
        self.shutdown.notify_waiters();

        // Stop HTTP server
        if let Some(http_server) = self.http_server.take() {
            let _ = http_server.await;
        }

        // Clear mint instance
        self.mint = None;
        self.is_running = false;
        
        info!("Integrated mintd service stopped");
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
            mint.mint_info().await.map_err(|e| anyhow!("Failed to get mint info: {}", e))
        } else {
            Err(anyhow!("Mint service not running"))
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
            "config": {
                "name": self.config.mint_info.name,
                "description": self.config.mint_info.description,
                "ln_backend": format!("{:?}", self.config.ln.ln_backend),
                "listen_port": self.config.info.listen_port
            }
        })
    }

    // Direct API methods for JNI calls
    pub async fn handle_mint_request(&self, amount: u64, unit: &str) -> Result<Value> {
        if let Some(_mint) = &self.mint {
            // This would implement the actual mint logic
            // For now, return a mock response
            Ok(serde_json::json!({
                "status": "success",
                "amount": amount,
                "unit": unit,
                "quote_id": "mock_quote_id"
            }))
        } else {
            Err(anyhow!("Mint service not running"))
        }
    }

    pub async fn handle_melt_request(&self, quote_id: &str) -> Result<Value> {
        if let Some(_mint) = &self.mint {
            // This would implement the actual melt logic
            Ok(serde_json::json!({
                "status": "success",
                "quote_id": quote_id,
                "melted": true
            }))
        } else {
            Err(anyhow!("Mint service not running"))
        }
    }
}

impl Drop for MintdService {
    fn drop(&mut self) {
        if self.is_running {
            let _ = self.stop();
        }
    }
} 