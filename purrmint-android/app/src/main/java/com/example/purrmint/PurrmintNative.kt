package com.example.purrmint

import android.content.Context
import android.util.Log
import java.io.File

/**
 * Simplified native interface for PurrMint
 * Provides basic FFI calls to Rust library
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

    // Basic FFI functions
    external fun testFfi(): String?
    external fun startMint(configDir: String, mnemonic: String, port: Int): Int
    external fun stopMint(): Int
    external fun getMintStatus(): String?
    external fun getMintInfo(): String?
    external fun generateConfig(configDir: String, mnemonic: String, port: Int): Int
    external fun createAccount(): String?
    external fun getCurrentAccount(): String?
} 