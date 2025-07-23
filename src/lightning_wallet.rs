use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use anyhow::{Result, Context};
use ldk_node::{
    Builder, Node, Config, Network, LogLevel, 
    Bolt11Invoice, Bolt11InvoiceDescription, PaymentId, PaymentStatus, PaymentDirection,
    ChannelDetails, PaymentDetails, BalanceDetails, PublicKey, SocketAddress
};
use tokio::runtime::Runtime;
use tracing::{info, error, warn};

/// Lightning Wallet Manager
/// Manages LDK Node instance and provides Lightning Network functionality
pub struct LightningWallet {
    node: Arc<Mutex<Option<Node>>>,
    runtime: Arc<Runtime>,
    storage_path: PathBuf,
    is_initialized: bool,
}

impl LightningWallet {
    /// Create a new Lightning Wallet instance
    pub fn new(storage_path: PathBuf) -> Result<Self> {
        let runtime = Runtime::new()
            .context("Failed to create Tokio runtime")?;
        
        Ok(Self {
            node: Arc::new(Mutex::new(None)),
            runtime: Arc::new(runtime),
            storage_path,
            is_initialized: false,
        })
    }

    /// Initialize the Lightning wallet
    pub fn initialize(&mut self) -> Result<()> {
        if self.is_initialized {
            return Ok(());
        }

        info!("Initializing Lightning wallet at {:?}", self.storage_path);
        
        // Create storage directory if it doesn't exist
        std::fs::create_dir_all(&self.storage_path)
            .context("Failed to create storage directory")?;

        // Create LDK Node configuration
        let config = Config {
            storage_dir_path: self.storage_path.to_string_lossy().to_string(),
            network: Network::Testnet, // Use testnet for development
            listening_addresses: None,
            announcement_addresses: None,
            node_alias: None,
            trusted_peers_0conf: vec![],
            probing_liquidity_limit_multiplier: 3,
            anchor_channels_config: None,
            sending_parameters: None,
        };

        // Build and start the node
        let node = self.runtime.block_on(async {
            let builder = Builder::from_config(config);
            
            // Set chain source (Esplora for testnet)
            builder.set_chain_source_esplora(
                "https://blockstream.info/testnet/api".to_string(),
                None
            );
            
            // Set gossip source (P2P)
            builder.set_gossip_source_p2p();
            
            // Set filesystem logger
            let log_file = self.storage_path.join("ldk_node.log");
            builder.set_filesystem_logger(
                Some(log_file.to_string_lossy().to_string()),
                Some(LogLevel::Info)
            );
            
            // Build the node
            let node = builder.build()
                .context("Failed to build LDK Node")?;
            
            // Start the node
            node.start()
                .context("Failed to start LDK Node")?;
            
            Ok::<Node, anyhow::Error>(node)
        })?;

        // Store the node
        {
            let mut node_guard = self.node.lock().unwrap();
            *node_guard = Some(node);
        }

        self.is_initialized = true;
        info!("Lightning wallet initialized successfully");
        Ok(())
    }

    /// Get wallet status
    pub fn get_status(&self) -> Result<WalletStatus> {
        if !self.is_initialized {
            return Ok(WalletStatus {
                is_running: false,
                node_id: None,
                is_connected: false,
            });
        }

        let node_guard = self.node.lock().unwrap();
        if let Some(node) = &*node_guard {
            let node_id = node.node_id().to_string();
            let status = node.status();
            
            Ok(WalletStatus {
                is_running: status.is_running(),
                node_id: Some(node_id),
                is_connected: status.is_running(),
            })
        } else {
            Ok(WalletStatus {
                is_running: false,
                node_id: None,
                is_connected: false,
            })
        }
    }

    /// Get wallet balance
    pub fn get_balance(&self) -> Result<Option<WalletBalance>> {
        if !self.is_initialized {
            return Ok(None);
        }

        let node_guard = self.node.lock().unwrap();
        if let Some(node) = &*node_guard {
            let balance_details = node.list_balances();
            
            Ok(Some(WalletBalance {
                lightning_balance_msat: balance_details.lightning_balance_msat,
                onchain_balance_sats: balance_details.onchain_balance_sats,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get channels list
    pub fn get_channels(&self) -> Result<Vec<ChannelInfo>> {
        if !self.is_initialized {
            return Ok(vec![]);
        }

        let node_guard = self.node.lock().unwrap();
        if let Some(node) = &*node_guard {
            let channels = node.list_channels();
            
            Ok(channels.into_iter().map(|channel| ChannelInfo {
                channel_id: channel.channel_id.to_string(),
                peer_id: channel.counterparty_node_id.to_string(),
                capacity_msat: channel.channel_value_sats * 1000, // Convert to msat
                balance_msat: channel.balance_msat,
                status: if channel.is_channel_ready { "OPEN".to_string() } else { "PENDING".to_string() },
            }).collect())
        } else {
            Ok(vec![])
        }
    }

    /// Get payment history
    pub fn get_payments(&self) -> Result<Vec<PaymentInfo>> {
        if !self.is_initialized {
            return Ok(vec![]);
        }

        let node_guard = self.node.lock().unwrap();
        if let Some(node) = &*node_guard {
            let payments = node.list_payments();
            
            Ok(payments.into_iter().map(|payment| PaymentInfo {
                payment_id: payment.id.to_string(),
                amount_msat: payment.amount_msat,
                description: payment.description.unwrap_or_else(|| "No description".to_string()),
                status: match payment.status {
                    PaymentStatus::Succeeded => "SUCCEEDED".to_string(),
                    PaymentStatus::Pending => "PENDING".to_string(),
                    PaymentStatus::Failed => "FAILED".to_string(),
                },
                is_incoming: payment.direction == PaymentDirection::Inbound,
                timestamp: payment.timestamp,
            }).collect())
        } else {
            Ok(vec![])
        }
    }

    /// Create a Lightning invoice
    pub fn create_invoice(&self, amount_sats: u64, description: String) -> Result<String> {
        if !self.is_initialized {
            anyhow::bail!("Wallet not initialized");
        }

        let node_guard = self.node.lock().unwrap();
        if let Some(node) = &*node_guard {
            let invoice_description = Bolt11InvoiceDescription::Direct(description);
            
            let invoice = self.runtime.block_on(async {
                node.bolt11_payment().receive(
                    amount_sats * 1000, // Convert to msat
                    invoice_description,
                    3600 // 1 hour expiry
                )
            })?;
            
            Ok(invoice.to_string())
        } else {
            anyhow::bail!("Node not available")
        }
    }

    /// Pay a Lightning invoice
    pub fn pay_invoice(&self, invoice_str: String) -> Result<String> {
        if !self.is_initialized {
            anyhow::bail!("Wallet not initialized");
        }

        let node_guard = self.node.lock().unwrap();
        if let Some(node) = &*node_guard {
            let invoice = Bolt11Invoice::from_str(&invoice_str)?;
            
            let payment_id = self.runtime.block_on(async {
                node.bolt11_payment().send(invoice, None)
            })?;
            
            Ok(payment_id.to_string())
        } else {
            anyhow::bail!("Node not available")
        }
    }

    /// Open a Lightning channel
    pub fn open_channel(&self, node_id: String, address: String, amount_sats: u64) -> Result<String> {
        if !self.is_initialized {
            anyhow::bail!("Wallet not initialized");
        }

        if amount_sats < 10_000 {
            anyhow::bail!("Channel amount must be at least 10,000 sats");
        }

        let node_guard = self.node.lock().unwrap();
        if let Some(node) = &*node_guard {
            let peer_node_id = PublicKey::from_str(&node_id)?;
            let peer_address = SocketAddress::from_str(&address)?;
            
            // Connect to peer first
            self.runtime.block_on(async {
                node.connect(peer_node_id, peer_address, true)
            })?;
            
            // Open channel
            let channel_id = self.runtime.block_on(async {
                node.open_channel(
                    peer_node_id,
                    peer_address,
                    amount_sats,
                    None, // push_to_counterparty_msat
                    None, // channel_config
                )
            })?;
            
            Ok(channel_id.to_string())
        } else {
            anyhow::bail!("Node not available")
        }
    }

    /// Cleanup resources
    pub fn cleanup(&mut self) -> Result<()> {
        if self.is_initialized {
            info!("Cleaning up Lightning wallet");
            
            let mut node_guard = self.node.lock().unwrap();
            if let Some(node) = node.take() {
                self.runtime.block_on(async {
                    node.stop()
                })?;
            }
            
            self.is_initialized = false;
            info!("Lightning wallet cleaned up successfully");
        }
        
        Ok(())
    }
}

impl Drop for LightningWallet {
    fn drop(&mut self) {
        if let Err(e) = self.cleanup() {
            error!("Failed to cleanup Lightning wallet: {}", e);
        }
    }
}

/// Wallet status information
#[derive(Debug, Clone)]
pub struct WalletStatus {
    pub is_running: bool,
    pub node_id: Option<String>,
    pub is_connected: bool,
}

/// Wallet balance information
#[derive(Debug, Clone)]
pub struct WalletBalance {
    pub lightning_balance_msat: u64,
    pub onchain_balance_sats: u64,
}

/// Channel information
#[derive(Debug, Clone)]
pub struct ChannelInfo {
    pub channel_id: String,
    pub peer_id: String,
    pub capacity_msat: u64,
    pub balance_msat: u64,
    pub status: String,
}

/// Payment information
#[derive(Debug, Clone)]
pub struct PaymentInfo {
    pub payment_id: String,
    pub amount_msat: u64,
    pub description: String,
    pub status: String,
    pub is_incoming: bool,
    pub timestamp: u64,
} 