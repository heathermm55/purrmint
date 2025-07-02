//! MintService and related traits/structs

use async_trait::async_trait;
use nostr::prelude::*;
use nostr_sdk::{Client, RelayPoolNotification};
use std::sync::Arc;
use thiserror::Error;
use tokio::task::JoinHandle;
use tracing::error;
use crate::{MintInfo, OperationRequest, OperationResult, Nip74Result, Nip74Error};
use crate::helpers::build_mint_info_event;

/// Errors raised by the [`MintService`].
#[derive(Debug, Error)]
pub enum ServiceError {
    /// Underlying NIP-74 helper error.
    #[error(transparent)]
    Nip74(#[from] Nip74Error),
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

        // Compose identifier from `name` field or fallback to pubkey slug.
        let identifier = self
            .mint_info
            .name
            .clone()
            .unwrap_or_else(|| "mint".to_owned());

        let event = build_mint_info_event(
            &self.mint_info,
            &self.signer,
            &identifier,
            &self.relays,
            "running",
            None,
        )
        .await?;
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
                    
                    // Use NIP-44 decryption
                    match signer.nip44_decrypt(&event.pubkey, &event.content).await {
                        Ok(plaintext) => {
                            match serde_json::from_str::<OperationRequest>(&plaintext) {
                                Ok(req) => {
                                    tracing::info!(method=?req.method, req_id=%req.request_id, "Parsed OperationRequest");
                                    // Process
                                    let res = handler.handle(req).await;
                                    match res {
                                        Ok(op_res) => {
                                            let event_result = op_res
                                                .to_event_with_signer(
                                                    &signer,
                                                    &signer.get_public_key().await.unwrap(),
                                                    &event.pubkey,
                                                    &event.id,
                                                    None,
                                                )
                                                .await
                                                .map_err(|e| e.to_string());
                                                
                                            match event_result {
                                                Ok(ev) => {
                                                    match client_clone.send_event(&ev).await {
                                                        Ok(out) => tracing::info!(sent=out.success.len(), failed=?out.failed, "OperationResult 27402 sent"),
                                                        Err(e) => tracing::error!(error = %e, "failed to send 27402"),
                                                    };
                                                }
                                                Err(e) => {
                                                    tracing::error!(error = %e, "failed to create 27402 event");
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