package com.example.purrmint

/**
 * Represents a Nostr account with public and secret keys
 */
data class NostrAccount(
    var pubkey: String = "",
    var secretKey: String = "",
    var isImported: Boolean = false
) {
    /**
     * Get the public key as a display string (truncated)
     */
    fun getDisplayPubkey(): String {
        return if (pubkey.length > 16) {
            "${pubkey.take(8)}...${pubkey.takeLast(8)}"
        } else {
            pubkey
        }
    }
    
    /**
     * Check if the account is valid
     */
    fun isValid(): Boolean {
        return pubkey.isNotEmpty() && secretKey.isNotEmpty()
    }
    
    override fun toString(): String {
        return "NostrAccount(pubkey='${getDisplayPubkey()}', isImported=$isImported)"
    }
} 