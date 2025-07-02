package com.example.purrmint

import org.json.JSONObject

/**
 * Native interface to PurrMint library
 * Provides JNI bindings to the Rust FFI functions
 */
object PurrmintNative {
    
    init {
        System.loadLibrary("purrmint-jni")
    }
    
    /**
     * Test the FFI interface
     * @return JSON string with test results
     */
    external fun testFfi(): String
    
    /**
     * Create a new Nostr account
     * @return NostrAccount object or null if failed
     */
    external fun createAccount(): NostrAccount?
    
    /**
     * Get mint information
     * @return JSON string with mint info
     */
    external fun getMintInfo(): String
    
    /**
     * Get mint status
     * @return JSON string with mint status
     */
    external fun getMintStatus(): String
    
    /**
     * Configure the mint service
     * @param configJson JSON configuration string
     * @return FfiError code
     */
    external fun configureMint(configJson: String): Int
    
    /**
     * Start the mint service
     * @return FfiError code
     */
    external fun startMint(): Int
    
    /**
     * Stop the mint service
     * @return FfiError code
     */
    external fun stopMint(): Int
    
    /**
     * FFI Error codes
     */
    enum class FfiError(val code: Int) {
        SUCCESS(0),
        NULL_POINTER(1),
        INVALID_INPUT(2),
        SERVICE_ERROR(3),
        NOT_INITIALIZED(4);
        
        companion object {
            fun fromCode(code: Int): FfiError? {
                return values().find { it.code == code }
            }
        }
    }
    
    /**
     * Helper function to parse JSON response
     */
    fun parseJsonResponse(jsonString: String): JSONObject? {
        return try {
            JSONObject(jsonString)
        } catch (e: Exception) {
            null
        }
    }
} 