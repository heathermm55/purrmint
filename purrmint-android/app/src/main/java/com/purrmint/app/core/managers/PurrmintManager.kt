package com.purrmint.app.core.managers

import android.content.Context
import android.util.Log
import java.io.File
import com.purrmint.app.PurrmintNative

/**
 * PurrMint service manager
 * Handles mint service operations using new JNI interface
 */
class PurrmintManager(private val context: Context) {
    private val native = PurrmintNative()
    
    companion object {
        private const val TAG = "PurrmintManager"
        private const val CONFIG_FILE_NAME = "android_config.json"
        private const val ACCOUNT_FILE_NAME = "nostr_account.json"
    }
    
    /**
     * Get the application's internal data directory (sandbox directory)
     */
    private fun getDataDir(): String {
        return context.filesDir.absolutePath
    }
    
    /**
     * Get the path to the config file
     */
    private fun getConfigFilePath(): String {
        return File(getDataDir(), CONFIG_FILE_NAME).absolutePath
    }
    
    /**
     * Get the path to the account file
     */
    private fun getAccountFilePath(): String {
        return File(getDataDir(), ACCOUNT_FILE_NAME).absolutePath
    }
    
    /**
     * Check if the config file exists
     */
    fun configExists(): Boolean {
        return File(getConfigFilePath()).exists()
    }
    
    /**
     * Check if the Nostr account file exists
     */
    fun accountExists(): Boolean {
        return File(getAccountFilePath()).exists()
    }
    
    /**
     * Create necessary directories for mint service
     */
    private fun createDirectories() {
        val dataDir = File(getDataDir())
        val databaseDir = File(dataDir, "database")
        val logsDir = File(dataDir, "logs")
        
        databaseDir.mkdirs()
        logsDir.mkdirs()
    }
    
    /**
     * Initialize logging
     */
    fun initLogging() {
        try {
            native.initLogging()
            Log.i(TAG, "Logging initialized")
        } catch (e: Exception) {
            Log.e(TAG, "Failed to initialize logging", e)
        }
    }
    
    /**
     * Generate default Android configuration
     * @return Configuration JSON string or null if failed
     */
    fun generateDefaultConfig(): String? {
        return try {
            // Generate default config with correct Android paths
            val dataDir = getDataDir()
            val defaultConfig = mapOf(
                "port" to 3338,
                "host" to "0.0.0.0",
                "mintName" to "PurrMint",
                "description" to "Mobile Cashu Mint",
                "lightningBackend" to "fakewallet",
                "mode" to "mintd_only",
                "databasePath" to "$dataDir/mint.db",
                "logsPath" to "$dataDir/logs"
            )
            
            // Convert to JSON string
            val json = org.json.JSONObject(defaultConfig).toString()
            Log.i(TAG, "Default configuration generated with paths: db=$dataDir/mint.db, logs=$dataDir/logs")
            json
        } catch (e: Exception) {
            Log.e(TAG, "Error generating default configuration", e)
            null
        }
    }
    
    /**
     * Save configuration to file
     * @param config Configuration JSON string
     * @return true if saved successfully
     */
    fun saveConfigToFile(config: String): Boolean {
        return try {
            createDirectories()
            val result = native.saveAndroidConfigToFile(getConfigFilePath(), config)
            val success = result == 0  // 0 = success in Rust
            if (success) {
                Log.i(TAG, "Configuration saved to file")
            } else {
                Log.e(TAG, "Failed to save configuration to file")
            }
            success
        } catch (e: Exception) {
            Log.e(TAG, "Error saving configuration to file", e)
            false
        }
    }
    
    /**
     * Load configuration from file
     * @return Configuration JSON string or null if failed
     */
    fun loadConfigFromFile(): String? {
        return try {
            val config = native.loadAndroidConfigFromFile(getConfigFilePath())
            if (config != null) {
                Log.i(TAG, "Configuration loaded from file")
            } else {
                Log.e(TAG, "Failed to load configuration from file")
            }
            config
        } catch (e: Exception) {
            Log.e(TAG, "Error loading configuration from file", e)
            null
        }
    }
    
    /**
     * Create a new Nostr account
     * @return Account JSON string or null if failed
     */
    fun createNostrAccount(): String? {
        return try {
            val account = native.createAccount()
            if (account != null) {
                // Save account to file
                val accountFile = File(getAccountFilePath())
                accountFile.writeText(account)
                Log.i(TAG, "Nostr account created and saved")
            } else {
                Log.e(TAG, "Failed to create Nostr account")
            }
            account
        } catch (e: Exception) {
            Log.e(TAG, "Error creating Nostr account", e)
            null
        }
    }
    
    /**
     * Convert nsec to npub
     * @param nsec The nsec key to convert
     * @return npub string or null if failed
     */
    fun convertNsecToNpub(nsec: String): String? {
        return try {
            val npub = native.nsecToNpub(nsec)
            if (npub != null) {
                Log.i(TAG, "Successfully converted nsec to npub")
            } else {
                Log.e(TAG, "Failed to convert nsec to npub")
            }
            npub
        } catch (e: Exception) {
            Log.e(TAG, "Error converting nsec to npub", e)
            null
        }
    }
    
    /**
     * Start mint service
     * @param nsec REQUIRED nsec key for mint service
     * @return true if service started successfully
     */
    fun startMintService(nsec: String): Boolean {
        return try {
            createDirectories()
            initLogging()
            
            // Validate nsec
            if (nsec.isEmpty()) {
                Log.e(TAG, "Cannot start mint service: nsec is required")
                return false
            }
            
            // Load or generate configuration
            val config = loadConfigFromFile() ?: generateDefaultConfig()
            if (config == null) {
                Log.e(TAG, "Failed to get configuration for service")
                return false
            }
            
            Log.i(TAG, "Starting mint service with nsec: ***provided***")
            
            // Start service with the nsec
            val result = native.startMintWithConfig(config, nsec)
            val success = result == 0  // 0 = success in Rust
            
            if (success) {
                Log.i(TAG, "Mint service started successfully")
            } else {
                Log.e(TAG, "Failed to start mint service - result code: $result")
            }
            
            success
        } catch (e: Exception) {
            Log.e(TAG, "Failed to start mint service", e)
            false
        }
    }
    
    /**
     * Start mint service with custom configuration
     * @param nsec REQUIRED nsec key for mint service
     * @param configJson Custom configuration JSON string
     * @return true if service started successfully
     */
    fun startMintServiceWithConfig(nsec: String, configJson: String): Boolean {
        return try {
            createDirectories()
            initLogging()
            
            // Validate nsec
            if (nsec.isEmpty()) {
                Log.e(TAG, "Cannot start mint service: nsec is required")
                return false
            }
            
            // Validate config JSON
            if (configJson.isEmpty()) {
                Log.e(TAG, "Cannot start mint service: config JSON is required")
                return false
            }
            
            Log.i(TAG, "Starting mint service with custom configuration")
            Log.d(TAG, "Config JSON: $configJson")
            
            // Start service with the custom config and nsec
            val result = native.startMintWithConfig(configJson, nsec)
            val success = result == 0  // 0 = success in Rust
            
            if (success) {
                Log.i(TAG, "Mint service started successfully with custom config")
            } else {
                Log.e(TAG, "Failed to start mint service with custom config - result code: $result")
            }
            
            success
        } catch (e: Exception) {
            Log.e(TAG, "Failed to start mint service with custom config", e)
            false
        }
    }
    
    /**
     * Start mint service using saved nsec from SharedPreferences
     * @return true if service started successfully
     */
    fun startMintServiceWithSavedNsec(): Boolean {
        return try {
            // Get saved nsec from SharedPreferences directly
            val prefs = context.getSharedPreferences("PurrmintLoginPrefs", Context.MODE_PRIVATE)
            val savedNsec = prefs.getString("nsec_key", null)
            
            if (savedNsec != null && savedNsec.isNotEmpty()) {
                Log.i(TAG, "Found saved nsec, starting mint service")
                startMintService(savedNsec)
            } else {
                Log.e(TAG, "No saved nsec found, cannot start mint service")
                false
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to start mint service with saved nsec", e)
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
            val success = result == 0  // 0 = success in Rust
            if (success) {
                Log.i(TAG, "Mint service stopped successfully")
            } else {
                Log.e(TAG, "Failed to stop mint service")
            }
            success
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
     * Get current Nostr account information from file
     * @return JSON string containing account info
     */
    fun getCurrentAccount(): String {
        return try {
            val accountFile = File(getAccountFilePath())
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
                // Ignore localhost connection failure
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
} 