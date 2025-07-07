package com.purrmint.app.core.native

import android.content.Context
import android.util.Log
import java.io.File

/**
 * Native interface for PurrMint
 * Provides JNI calls to Rust library
 */
class PurrmintNative {
    companion object {
        private const val TAG = "PurrmintNative"
        
        // Load the appropriate library based on device
        init {
            try {
                // Try to load the standard library first
                System.loadLibrary("purrmint")
                Log.i(TAG, "Loaded standard purrmint library")
            } catch (e: UnsatisfiedLinkError) {
                Log.w(TAG, "Standard library failed, trying 16KB version", e)
                try {
                    // Try 16KB version if standard fails
                    System.loadLibrary("purrmint_16k")
                    Log.i(TAG, "Loaded 16KB purrmint library")
                } catch (e2: UnsatisfiedLinkError) {
                    Log.e(TAG, "Failed to load any purrmint library", e2)
                    throw e2
                }
            }
        }
    }

    // Basic initialization
    external fun initLogging()
    
    // Nostr account management
    external fun createNostrAccount(): String?
    external fun convertNsecToNpub(nsec: String): String?
    
    // Configuration management
    external fun generateAndroidConfig(): String?
    external fun saveConfigToFile(config: String, filePath: String): Boolean
    external fun loadConfigFromFile(filePath: String): String?
    
    // Service management
    external fun startService(config: String, nsec: String?): Boolean
    external fun stopService(): Boolean
    external fun getServiceStatus(): String?
    
    // Memory management
    external fun freeString(ptr: Long)
} 