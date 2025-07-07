//! Nostr operations module
//! Provides Nostr key generation and conversion utilities

use std::ffi::{CStr, CString};
use std::str::FromStr;
use std::os::raw::c_char;
use nostr::{Keys, ToBech32};
use tracing::error;

/// Nostr Account structure for Android
#[repr(C)]
pub struct NostrAccount {
    pub pubkey: *mut c_char,
    pub secret_key: *mut c_char,
    pub is_imported: bool,
}

/// Nostr operation results
pub type NostrResult<T> = Result<T, NostrError>;

/// Nostr operation errors
#[derive(Debug)]
pub enum NostrError {
    InvalidKey,
    ConversionError,
    NullPointer,
    Unknown(String),
}

impl std::fmt::Display for NostrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NostrError::InvalidKey => write!(f, "Invalid nostr key"),
            NostrError::ConversionError => write!(f, "Key conversion failed"),
            NostrError::NullPointer => write!(f, "Null pointer encountered"),
            NostrError::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl std::error::Error for NostrError {}

// =============================================================================
// Nostr Account Management
// =============================================================================

/// Create a new Nostr account (internal function)
pub fn create_nostr_account() -> *mut NostrAccount {
    // Generate new keys
    let keys = Keys::generate();
    let pubkey = CString::new(keys.public_key().to_string()).unwrap();
    let secret_key = CString::new(keys.secret_key().to_secret_hex()).unwrap();
    
    let account = Box::new(NostrAccount {
        pubkey: pubkey.into_raw(),
        secret_key: secret_key.into_raw(),
        is_imported: false,
    });
    
    Box::into_raw(account)
}

/// Free Nostr account memory
pub fn free_nostr_account(account: *mut NostrAccount) {
    if account.is_null() {
        return;
    }
    
    unsafe {
        let account = Box::from_raw(account);
        if !account.pubkey.is_null() {
            drop(CString::from_raw(account.pubkey));
        }
        if !account.secret_key.is_null() {
            drop(CString::from_raw(account.secret_key));
        }
    }
}

// =============================================================================
// Nostr Key Operations
// =============================================================================

/// Create a new Nostr account and return nsec in bech32 format
pub fn create_account() -> NostrResult<String> {
    // Create account via internal function
    let account = create_nostr_account();
    if account.is_null() {
        error!("Failed to create nostr account");
        return Err(NostrError::Unknown("Account creation failed".to_string()));
    }
    
    // Convert account to nsec format
    let nsec = unsafe {
        let account_ptr = account as *const NostrAccount;
        let account_ref = &*account_ptr;
        
        let secret_str = CStr::from_ptr(account_ref.secret_key).to_str()
            .map_err(|_| NostrError::ConversionError)?;
        
        // Parse the secret key to convert to bech32 format
        let nsec = match Keys::from_str(secret_str) {
            Ok(keys) => keys.secret_key().to_bech32()
                .map_err(|_| NostrError::ConversionError)?,
            Err(_) => {
                error!("Failed to parse secret key: {}", secret_str);
                return Err(NostrError::InvalidKey);
            }
        };
        
        // Free the account before returning
        free_nostr_account(account);
        
        nsec
    };
    
    Ok(nsec)
}

/// Convert nsec to npub format
pub fn nsec_to_npub(nsec: &str) -> NostrResult<String> {
    if nsec.is_empty() {
        return Err(NostrError::InvalidKey);
    }
    
    // Parse the nsec and convert to npub
    match Keys::from_str(nsec) {
        Ok(keys) => {
            match keys.public_key().to_bech32() {
                Ok(npub) => Ok(npub),
                Err(e) => {
                    error!("Failed to convert public key to bech32: {:?}", e);
                    Err(NostrError::ConversionError)
                }
            }
        },
        Err(e) => {
            error!("Failed to parse nsec '{}': {:?}", nsec, e);
            Err(NostrError::InvalidKey)
        }
    }
}

/// Validate if a string is a valid nsec
pub fn is_valid_nsec(nsec: &str) -> bool {
    nsec.starts_with("nsec1") && Keys::from_str(nsec).is_ok()
}

/// Validate if a string is a valid npub
pub fn is_valid_npub(npub: &str) -> bool {
    npub.starts_with("npub1") && npub.len() == 63
}

/// Generate a new set of Nostr keys
pub fn generate_keys() -> NostrResult<(String, String)> {
    let keys = Keys::generate();
    
    let nsec = keys.secret_key().to_bech32()
        .map_err(|e| {
            error!("Failed to convert secret key to bech32: {:?}", e);
            NostrError::ConversionError
        })?;
    
    let npub = keys.public_key().to_bech32()
        .map_err(|e| {
            error!("Failed to convert public key to bech32: {:?}", e);
            NostrError::ConversionError
        })?;
    
    Ok((nsec, npub))
}

/// Extract public key from secret key (both in bech32 format)
pub fn get_public_key_from_secret(nsec: &str) -> NostrResult<String> {
    match Keys::from_str(nsec) {
        Ok(keys) => {
            match keys.public_key().to_bech32() {
                Ok(npub) => Ok(npub),
                Err(e) => {
                    error!("Failed to convert public key to bech32: {:?}", e);
                    Err(NostrError::ConversionError)
                }
            }
        },
        Err(e) => {
            error!("Failed to parse nsec: {:?}", e);
            Err(NostrError::InvalidKey)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_account() {
        let nsec = create_account().unwrap();
        assert!(nsec.starts_with("nsec1"));
        assert!(is_valid_nsec(&nsec));
    }

    #[test]
    fn test_nsec_to_npub() {
        let test_nsec = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";
        let npub = nsec_to_npub(test_nsec).unwrap();
        assert!(npub.starts_with("npub1"));
        assert!(is_valid_npub(&npub));
    }

    #[test]
    fn test_generate_keys() {
        let (nsec, npub) = generate_keys().unwrap();
        assert!(is_valid_nsec(&nsec));
        assert!(is_valid_npub(&npub));
        
        // Verify they match
        let derived_npub = nsec_to_npub(&nsec).unwrap();
        assert_eq!(npub, derived_npub);
    }

    #[test]
    fn test_invalid_keys() {
        assert!(nsec_to_npub("invalid").is_err());
        assert!(nsec_to_npub("").is_err());
        assert!(!is_valid_nsec("invalid"));
        assert!(!is_valid_npub("invalid"));
    }

    #[test]
    fn test_get_public_key_from_secret() {
        let test_nsec = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";
        let npub = get_public_key_from_secret(test_nsec).unwrap();
        assert!(npub.starts_with("npub1"));
        assert_eq!(npub, nsec_to_npub(test_nsec).unwrap());
    }
} 