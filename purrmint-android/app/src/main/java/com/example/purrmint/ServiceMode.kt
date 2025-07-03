package com.example.purrmint

/**
 * Service mode enumeration for mint service
 */
enum class ServiceMode {
    MintdOnly,      // Only mintd HTTP API
    Nip74Only,      // Only NIP-74 Nostr interface
    MintdAndNip74   // Both mintd and NIP-74
} 