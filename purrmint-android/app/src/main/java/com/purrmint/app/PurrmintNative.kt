package com.purrmint.app

import android.util.Log

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

    // Basic initialization - matches Java_com_purrmint_app_PurrmintNative_initLogging
    external fun initLogging()
    
    // Nostr account management - matches Java_com_purrmint_app_PurrmintNative_createAccount
    external fun createAccount(): String?
    
    // Convert nsec to npub - matches Java_com_purrmint_app_PurrmintNative_nsecToNpub
    external fun nsecToNpub(nsec: String): String?
    
    // Configuration management - matches Java_com_purrmint_app_PurrmintNative_loadAndroidConfigFromFile
    external fun loadAndroidConfigFromFile(filePath: String): String?
    
    // Save config to file - matches Java_com_purrmint_app_PurrmintNative_saveAndroidConfigToFile
    external fun saveAndroidConfigToFile(filePath: String, configJson: String): Int
    
    // Generate default config - matches Java_com_purrmint_app_PurrmintNative_generateDefaultAndroidConfig
    external fun generateDefaultAndroidConfig(): String?
    
    // Load default config - matches Java_com_purrmint_app_PurrmintNative_loadConfig
    external fun loadConfig(): String?
    
    // Update config - matches Java_com_purrmint_app_PurrmintNative_updateConfig
    external fun updateConfig(configJson: String): String?
    
    // Service management - matches Java_com_purrmint_app_PurrmintNative_startMintWithConfig
    external fun startMintWithConfig(configJson: String, nsec: String): Int
    
    // Stop mint - matches Java_com_purrmint_app_PurrmintNative_stopMint
    external fun stopMint(): Int
    
    // Get mint status - matches Java_com_purrmint_app_PurrmintNative_getMintStatus
    external fun getMintStatus(): String?
} 