//! PurrMint – high-level Cashu NIP-74 mint service.

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod nip74_service;
pub mod service;
pub mod core;
pub mod config;
pub mod mintd_service;
pub mod nostr;

#[cfg(feature = "jni-support")]
pub mod jni;

// NIP-74 service types and functions
pub use nip74_service::{
    Nip74Error, Nip74Result, ResultStatus, ResultError, OperationMethod, 
    OperationRequest, OperationResult, new_request_id, build_mint_info_event,
    DefaultRequestHandler, DefaultMintHandler
};

// Service types and functions
pub use service::{ServiceError, MintService};
pub use nip74_service::RequestHandler;

// Configuration types
pub use config::ServiceMode;

pub use cdk::nuts::nut06::MintInfo; 