package com.example.purrmint

import android.content.Context
import android.util.Log
import java.io.File

/**
 * Simplified PurrMint service manager
 * Handles basic mint service operations
 */
class PurrmintManager(private val context: Context) {
    private val native = PurrmintNative()
    
    companion object {
        private const val TAG = "PurrmintManager"
        private const val DEFAULT_PORT = 3338
        private const val DEFAULT_MNEMONIC = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
        private const val CONFIG_FILE_NAME = "mintd.conf"
        private const val ACCOUNT_FILE_NAME = "nostr_account.json"
    }
    
    /**
     * Get the application's internal data directory (sandbox directory)
     */
    private fun getDataDir(): String {
        return context.filesDir.absolutePath
    }
    
    /**
     * Check if the mintd config file exists in the sandbox directory
     */
    fun configExists(): Boolean {
        val configFile = File(getDataDir(), CONFIG_FILE_NAME)
        return configFile.exists()
    }
    
    /**
     * Check if the Nostr account file exists in the sandbox directory
     */
    fun accountExists(): Boolean {
        val accountFile = File(getDataDir(), ACCOUNT_FILE_NAME)
        return accountFile.exists()
    }
    
    /**
     * Create necessary directories for mint service (database, logs)
     */
    private fun createDirectories() {
        val dataDir = File(getDataDir())
        val databaseDir = File(dataDir, "database")
        val logsDir = File(dataDir, "logs")
        
        databaseDir.mkdirs()
        logsDir.mkdirs()
        
        Log.d(TAG, "Created directories: ${dataDir.absolutePath}")
    }
    
    /**
     * Extract mintd binary from assets to app internal storage
     */
    private fun extractMintdBinary() {
        try {
            val mintdFile = File(getDataDir(), "mintd")
            if (!mintdFile.exists()) {
                val inputStream = context.assets.open("mintd")
                val outputStream = mintdFile.outputStream()
                
                inputStream.copyTo(outputStream)
                inputStream.close()
                outputStream.close()
                
                // Make executable with full permissions
                mintdFile.setExecutable(true, true) // true for owner, true for all
                mintdFile.setReadable(true, true)
                mintdFile.setWritable(true, true)
                
                Log.d(TAG, "Mintd binary extracted to: ${mintdFile.absolutePath}")
                Log.d(TAG, "File permissions: executable=${mintdFile.canExecute()}, readable=${mintdFile.canRead()}, writable=${mintdFile.canWrite()}")
            } else {
                // Ensure existing file has correct permissions
                if (!mintdFile.canExecute()) {
                    mintdFile.setExecutable(true, true)
                    Log.d(TAG, "Updated permissions for existing mintd binary")
                }
                Log.d(TAG, "Mintd binary already exists: ${mintdFile.absolutePath}")
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to extract mintd binary", e)
        }
    }
    
    /**
     * Start mint service
     * @return true if service started successfully
     */
    fun startMintService(): Boolean {
        return try {
            createDirectories()
            extractMintdBinary()
            
            val configDir = getDataDir()
            val mnemonic = DEFAULT_MNEMONIC
            
            Log.d(TAG, "Starting mint service")
            Log.d(TAG, "Config directory: $configDir")
            Log.d(TAG, "Port: $DEFAULT_PORT")
            
            val result = native.startMint(configDir, mnemonic, DEFAULT_PORT)
            
            when (result) {
                0 -> {
                    Log.i(TAG, "Mint service started successfully")
                    true
                }
                else -> {
                    Log.e(TAG, "Failed to start service, error code: $result")
                    false
                }
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to start mint service", e)
            false
        }
    }
    
    /**
     * Stop mint service
     * @return true if service stopped successfully
     */
    fun stopMintService(): Boolean {
        return try {
            Log.d(TAG, "Stopping mint service")
            val result = native.stopMint()
            
            if (result == 0) {
                Log.i(TAG, "Mint service stopped successfully")
                true
            } else {
                Log.e(TAG, "Failed to stop service, error code: $result")
                false
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to stop mint service", e)
            false
        }
    }
    
    /**
     * Get mint service status
     * @return JSON string containing service status
     */
    fun getServiceStatus(): String {
        return try {
            native.getMintStatus() ?: "{\"status\":\"unknown\"}"
        } catch (e: Exception) {
            Log.e(TAG, "Failed to get service status", e)
            "{\"status\":\"error\",\"message\":\"${e.message}\"}"
        }
    }
    
    /**
     * Get mint service information
     * @return JSON string containing service info
     */
    fun getServiceInfo(): String {
        return try {
            native.getMintInfo() ?: "{\"info\":\"unknown\"}"
        } catch (e: Exception) {
            Log.e(TAG, "Failed to get service info", e)
            "{\"info\":\"error\",\"message\":\"${e.message}\"}"
        }
    }
    
    /**
     * Generate configuration file in the sandbox directory
     * @return true if configuration generated successfully
     */
    fun generateConfig(): Boolean {
        return try {
            val configDir = getDataDir()
            val mnemonic = DEFAULT_MNEMONIC
            
            Log.d(TAG, "Generating configuration")
            Log.d(TAG, "Config directory: $configDir")
            
            val result = native.generateConfig(configDir, mnemonic, DEFAULT_PORT)
            
            if (result == 0) {
                Log.i(TAG, "Configuration generated successfully")
                true
            } else {
                Log.e(TAG, "Failed to generate configuration, error code: $result")
                false
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to generate configuration", e)
            false
        }
    }
    
    /**
     * Create a new Nostr account and save to sandbox directory
     * @return JSON string containing account info or null if failed
     */
    fun createNostrAccount(): String? {
        return try {
            Log.d(TAG, "Creating new Nostr account")
            val accountJson = native.createAccount()
            if (accountJson != null) {
                val accountFile = File(getDataDir(), ACCOUNT_FILE_NAME)
                accountFile.writeText(accountJson)
            }
            accountJson
        } catch (e: Exception) {
            Log.e(TAG, "Failed to create Nostr account", e)
            null
        }
    }
    
    /**
     * Get current Nostr account information from sandbox directory
     * @return JSON string containing account info
     */
    fun getCurrentAccount(): String {
        return try {
            val accountFile = File(getDataDir(), ACCOUNT_FILE_NAME)
            if (accountFile.exists()) {
                accountFile.readText()
            } else {
                "{\"account\":\"none\"}"
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to get current account", e)
            "{\"account\":\"error\",\"message\":\"${e.message}\"}"
        }
    }
    
    /**
     * Test FFI interface
     * @return Test result string
     */
    fun testFfi(): String {
        return try {
            native.testFfi() ?: "{\"test\":\"failed\"}"
        } catch (e: Exception) {
            Log.e(TAG, "FFI test failed", e)
            "{\"test\":\"error\",\"message\":\"${e.message}\"}"
        }
    }
} 