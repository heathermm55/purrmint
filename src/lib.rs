//! PurrMint – high-level Cashu NIP-74 mint service.

pub mod nip74_service;
pub mod service;
pub mod mintd_service;
pub mod jni;
pub mod core;
pub mod nostr;
pub mod config;
pub mod tor_service;

// Re-export key types
pub use service::MintService;
pub use core::*;
pub use config::*;
pub use nip74_service::*;

/// Initialize logging for the library
pub fn init_logging() {
    tracing_subscriber::fmt::init();
} 