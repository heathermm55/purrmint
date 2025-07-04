package com.purrmint.app

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
        
        
    }
    
    /**
     * No longer needed - we use JNI interface directly
     * This method is kept for compatibility but does nothing
     */
    private fun extractMintdBinary() {
        // No longer needed since we removed cdk-mintd dependency
        // The mint service is now handled directly through JNI
        
    }
    
    /**
     * Start mint service using JNI interface
     * @return true if service started successfully
     */
    fun startMintService(): Boolean {
        return try {
            createDirectories()
            extractMintdBinary() // This now just logs a message
            
            // Initialize logging first
            native.initLogging()
            
            val configDir = getDataDir()
            val mnemonic = DEFAULT_MNEMONIC
            

            
            // Start service in background thread to avoid blocking main thread
            Thread {
                try {
                    // Use the Android-specific startMint function with parameters
                    // Mode 0 = MintdOnly
                    val result = native.startMintAndroid(0, configDir, mnemonic, DEFAULT_PORT)

                    
                    when (result) {
                        0 -> Log.i(TAG, "Mint service started successfully via JNI")
                        1 -> Log.e(TAG, "Failed to start service: Invalid configuration")
                        2 -> Log.e(TAG, "Failed to start service: Service already running")
                        3 -> Log.e(TAG, "Failed to start service: Internal error")
                        else -> Log.e(TAG, "Failed to start service, unknown error code: $result")
                    }
                } catch (e: Exception) {
                    Log.e(TAG, "Error in background thread starting mint service", e)
                }
            }.start()
            
            // Return immediately to avoid blocking main thread
            true
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
     * Get device IP address
     */
    fun getDeviceIpAddress(): String {
        return try {
            val wifiManager = context.getSystemService(Context.WIFI_SERVICE) as android.net.wifi.WifiManager
            val wifiInfo = wifiManager.connectionInfo
            val ipAddress = wifiInfo.ipAddress
            if (ipAddress != 0) {
                val ip = String.format(
                    "%d.%d.%d.%d",
                    ipAddress and 0xff,
                    ipAddress shr 8 and 0xff,
                    ipAddress shr 16 and 0xff,
                    ipAddress shr 24 and 0xff
                )

                ip
            } else {

                "127.0.0.1"
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to get device IP", e)
            "127.0.0.1"
        }
    }
    
    /**
     * Test HTTP connection to mint service
     * @return true if connection successful
     */
    fun testHttpConnection(): Boolean {
        return try {
            val deviceIp = getDeviceIpAddress()
            val url = "http://$deviceIp:3338"

            
            // First try localhost connection
            try {
                val localhostSocket = java.net.Socket()
                localhostSocket.connect(java.net.InetSocketAddress("127.0.0.1", 3338), 2000)
                val localhostConnected = localhostSocket.isConnected
                localhostSocket.close()
                
                if (localhostConnected) {
    
                    return true
                }
            } catch (e: Exception) {

            }
            
            // Then try device IP connection
            try {
                val socket = java.net.Socket()
                socket.connect(java.net.InetSocketAddress(deviceIp, 3338), 3000)
                val connected = socket.isConnected
                socket.close()
                

                return connected
            } catch (e: Exception) {
                Log.e(TAG, "Device IP connection failed: ${e.message}")
                return false
            }
        } catch (e: Exception) {
            Log.e(TAG, "HTTP connection test failed", e)
            false
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