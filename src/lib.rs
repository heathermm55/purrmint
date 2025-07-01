//! PurrMint – high-level Cashu NIP-74 mint service.
//!
//! This crate wraps lower-level primitives (`cashu-mint-nip74`, `nostr-sdk`)
//! to offer a ready-to-run async service that:
//!
//! 0. Authenticates a Nostr account (local keys or NIP-46 remote signer);
//! 1. Publishes `MintInfo` to a configurable relay set;
//! 2. Connects / disconnects to relays on demand;
//! 3. Listens for `OperationRequest` events (`kind:27401`);
//! 4. Delegates request processing to user-provided handler and
//!    forwards the produced `OperationResult` (`kind:27402`).
//!
//! The public API is intentionally minimal so that it can be exposed via
//! JNI to Android callers.

#![forbid(unsafe_code)]
#![warn(missing_docs, rustdoc::bare_urls)]

use async_trait::async_trait;
use cashu_mint_nip74::{MintInfo, OperationRequest, OperationResult, Result as Nip74Result};
use nostr::prelude::*;
use nostr_sdk::{Client, RelayPoolNotification};
use std::sync::Arc;
use thiserror::Error;
use tracing::error;
use tokio::task::JoinHandle;

/// Re-export NIP-74 types.
pub use cashu_mint_nip74::{OperationMethod, ResultStatus};

pub mod handler;

/// Errors raised by the [`MintService`].
#[derive(Debug, Error)]
pub enum ServiceError {
    /// Underlying NIP-74 helper error.
    #[error(transparent)]
    Nip74(#[from] cashu_mint_nip74::Nip74Error),
    /// Nostr SDK error.
    #[error(transparent)]
    Nostr(#[from] nostr_sdk::client::Error),
    /// Tokio join error.
    #[error("join error: {0}")]
    Join(#[from] tokio::task::JoinError),
}

/// Allow `?` on `SignerError`
impl From<nostr::signer::SignerError> for ServiceError {
    fn from(e: nostr::signer::SignerError) -> Self {
        ServiceError::Nostr(nostr_sdk::client::Error::Signer(e))
    }
}

/// Abstraction over local keys or remote signer.
pub type DynSigner = Arc<dyn NostrSigner>;

/// Request handler trait – application implements custom business logic.
#[async_trait]
pub trait RequestHandler: Send + Sync + 'static {
    /// Handle an OperationRequest and return the OperationResult.
    async fn handle(&self, req: OperationRequest) -> Nip74Result<OperationResult>;
}

/// Mint service – manages relay connections and request processing.
pub struct MintService {
    signer: DynSigner,
    mint_info: MintInfo,
    relays: Vec<RelayUrl>,
    handler: Arc<dyn RequestHandler>,
    client: Client,
    _task: Option<JoinHandle<()>>, // background listener
}

impl MintService {
    /// Create a new service instance.
    pub async fn new<T>(
        signer: DynSigner,
        mint_info: MintInfo,
        relays: T,
        handler: Arc<dyn RequestHandler>,
    ) -> Result<Self, ServiceError>
    where
        T: IntoIterator<Item = RelayUrl>,
    {
        let client = Client::new(signer.clone());
        Ok(Self {
            signer,
            mint_info,
            relays: relays.into_iter().collect(),
            handler,
            client,
            _task: None,
        })
    }

    /// Start the service: connect to relays, broadcast `MintInfo`, spawn request loop.
    pub async fn start(&mut self) -> Result<(), ServiceError> {
        for url in &self.relays {
            self.client.add_relay(url.clone()).await?;
        }
        self.client.connect().await;
        // Wait up to 5 seconds for at least one relay to be connected.
        self.client.wait_for_connection(std::time::Duration::from_secs(5)).await;

        // Broadcast mint info.
        let event = self.mint_info.to_event_with_signer(&self.signer, None).await?;
        self.client.send_event(&event).await?;
        tracing::info!(id = %event.id, "MintInfo event sent");

        // Subscribe for OperationRequest events.
        let filter = Filter::new().kind(Kind::from(27401u16));
        let _ = self.client.subscribe(filter, None).await?;

        // Spawn background task.
        let signer = self.signer.clone();
        let handler = self.handler.clone();
        let client_clone = self.client.clone();
        let mut notifications = client_clone.notifications();
        self._task = Some(tokio::spawn(async move {
            while let Ok(notif) = notifications.recv().await {
                if let RelayPoolNotification::Event { event, .. } = notif {
                    if event.kind != Kind::from(27401u16) { continue; }
                    tracing::info!(id=%event.id, from=%event.pubkey, "Received 27401 OperationRequest event");
                    // Decrypt via signer
                    match signer.nip44_decrypt(&event.pubkey, &event.content).await {
                        Ok(plaintext) => {
                            match serde_json::from_str::<OperationRequest>(&plaintext) {
                                Ok(req) => {
                                    tracing::info!(method=?req.method, req_id=%req.request_id, "Parsed OperationRequest");
                                    // Process
                                    let res = handler.handle(req).await;
                                    match res {
                                        Ok(op_res) => {
                                            if let Ok(ev) = op_res.to_event_with_signer(&signer, &signer.get_public_key().await.unwrap(), &event.pubkey, &event.id, None).await {
                                                match client_clone.send_event(&ev).await {
                                                    Ok(out) => tracing::info!(sent=out.success.len(), failed=?out.failed, "OperationResult 27402 sent"),
                                                    Err(e) => tracing::error!(?e, "failed to send 27402"),
                                                }
                                            }
                                        }
                                        Err(e) => error!(?e, "handler error"),
                                    }
                                }
                                Err(e) => error!(?e, "request parse error"),
                            }
                        }
                        Err(e) => error!(?e, "decrypt error"),
                    }
                }
            }
        }));
        Ok(())
    }

    /// Stop the service (disconnect relays and cancel background task).
    pub async fn stop(&mut self) -> Result<(), ServiceError> {
        if let Some(task) = self._task.take() {
            task.abort();
            let _ = task.await;
        }
        self.client.disconnect().await;
        Ok(())
    }
} 