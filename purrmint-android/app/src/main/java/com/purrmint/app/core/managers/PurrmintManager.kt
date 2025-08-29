package com.purrmint.app.core.managers

import android.content.Context
import android.util.Log
import java.io.File
import com.purrmint.app.PurrmintNative
import com.purrmint.app.R
import org.json.JSONObject

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
        private const val SSL_CERT_NAME = "purrmint_cert_pem"
        private const val SSL_KEY_NAME = "purrmint_key_pem"
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
        return safeExecute("generate default configuration") {
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
                "logsPath" to "$dataDir/logs",
                "enableHttps" to true,
                "httpsPort" to 8443,
                "sslCertPath" to "$dataDir/ssl/purrmint_cert_pem",
                "sslKeyPath" to "$dataDir/ssl/purrmint_key_pem"
            )
            
            // Convert to JSON string
            val json = org.json.JSONObject(defaultConfig).toString()
            Log.i(TAG, "Default configuration generated with paths: db=$dataDir/mint.db, logs=$dataDir/logs, HTTPS enabled on port 8443")
            Log.d(TAG, "SSL Certificate Path: ${defaultConfig["sslCertPath"]}")
            Log.d(TAG, "SSL Key Path: ${defaultConfig["sslKeyPath"]}")
            Log.d(TAG, "Full config JSON: $json")
            json
        }
    }
    
    /**
     * Save configuration to file
     * @param config Configuration JSON string
     * @return true if saved successfully
     */
    fun saveConfigToFile(config: String): Boolean {
        return safeExecute("save configuration to file") {
            createDirectories()
            val result = native.saveAndroidConfigToFile(getConfigFilePath(), config)
            val success = result == 0  // 0 = success in Rust
            if (success) {
                Log.i(TAG, "Configuration saved to file")
            } else {
                Log.e(TAG, "Failed to save configuration to file")
            }
            success
        } ?: false
    }
    
    /**
     * Load configuration from file
     * @return Configuration JSON string or null if failed
     */
    fun loadConfigFromFile(): String? {
        return safeExecute("load configuration from file") {
            val config = native.loadAndroidConfigFromFile(getConfigFilePath())
            if (config != null) {
                Log.i(TAG, "Configuration loaded from file")
            } else {
                Log.e(TAG, "Failed to load configuration from file")
            }
            config
        }
    }
    
    /**
     * Create a new Nostr account
     * @return Account JSON string or null if failed
     */
    fun createNostrAccount(): String? {
        return safeExecute("create Nostr account") {
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
        }
    }
    
    /**
     * Convert nsec to npub
     * @param nsec The nsec key to convert
     * @return npub string or null if failed
     */
    fun convertNsecToNpub(nsec: String): String? {
        return safeExecute("convert nsec to npub") {
            val npub = native.nsecToNpub(nsec)
            if (npub != null) {
                Log.i(TAG, "Successfully converted nsec to npub")
            } else {
                Log.e(TAG, "Failed to convert nsec to npub")
            }
            npub
        }
    }
    
    /**
     * Start mint service with nsec
     * @param nsec REQUIRED nsec key for mint service
     * @return true if service started successfully
     */
    fun startMintService(nsec: String): Boolean {
        return safeExecute("start mint service") {
            // Stop existing service first to ensure clean state
            Log.i(TAG, "Ensuring clean service state before starting...")
            stopMintService()
            
            // Wait a moment for service to fully stop
            Thread.sleep(500)
            
            createDirectories()
            ensureSslCertificatesExist()  // Ensure SSL certificates are copied
            initLogging()
            
            // Validate nsec
            if (nsec.isEmpty()) {
                Log.e(TAG, "Cannot start mint service: nsec is required")
                return@safeExecute false
            }
            
            // Load or generate configuration
            val config = loadConfigFromFile() ?: generateDefaultConfig()
            if (config == null) {
                Log.e(TAG, "Failed to get configuration for service")
                return@safeExecute false
            }
            
            // Log the configuration being sent to Rust for debugging
            Log.d(TAG, "Configuration being sent to Rust: $config")
            
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
        } ?: false
    }
    
    /**
     * Start mint service with custom configuration
     * @param nsec REQUIRED nsec key for mint service
     * @param configJson Custom configuration JSON string
     * @return true if service started successfully
     */
    fun startMintServiceWithConfig(nsec: String, configJson: String): Boolean {
        return safeExecute("start mint service with custom config") {
            // Stop existing service first to ensure clean state
            Log.i(TAG, "Ensuring clean service state before starting with custom config...")
            stopMintService()
            
            // Wait a moment for service to fully stop
            Thread.sleep(500)
            
            createDirectories()
            ensureSslCertificatesExist()  // Ensure SSL certificates are copied
            initLogging()
            
            // Validate nsec
            if (nsec.isEmpty()) {
                Log.e(TAG, "Cannot start mint service: nsec is required")
                return@safeExecute false
            }
            
            // Validate config JSON
            if (configJson.isEmpty()) {
                Log.e(TAG, "Cannot start mint service: config JSON is required")
                return@safeExecute false
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
        } ?: false
    }
    
    /**
     * Start mint service using saved nsec from SharedPreferences
     * @return true if service started successfully
     */
    fun startMintServiceWithSavedNsec(): Boolean {
        return safeExecute("start mint service with saved nsec") {
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
        } ?: false
    }
    
    /**
     * Stop mint service
     * @return true if service stopped successfully
     */
    fun stopMintService(): Boolean {
        return safeExecute("stop mint service") {
            val result = native.stopMint()
            val success = result == 0  // 0 = success in Rust
            if (success) {
                Log.i(TAG, "Mint service stopped successfully")
            } else {
                Log.e(TAG, "Failed to stop mint service")
            }
            success
        } ?: false
    }
    
    /**
     * Get mint service status
     * @return Status JSON string
     */
    fun getServiceStatus(): String {
        return try {
            native.getMintStatus() ?: "{\"status\":\"unknown\"}"
        } catch (e: Exception) {
            Log.e(TAG, "Failed to get mint status", e)
            "{\"status\":\"error\",\"error\":\"${e.message}\"}"
        }
    }
    
    /**
     * Delete mint service and clean up resources
     * @return true if successful, false otherwise
     */
    fun deleteMintService(): Boolean {
        return try {
            Log.i(TAG, "Deleting mint service and cleaning up resources...")
            Log.i(TAG, "Note: User data (Nostr account, Tor settings, preferences) will be preserved")
            
            val result = native.deleteMint()
            if (result == 0) {
                Log.i(TAG, "Mint service deleted successfully")
                Log.i(TAG, "Cleanup completed: Service runtime data removed, user data preserved")
                true
            } else {
                Log.e(TAG, "Failed to delete mint service, result: $result")
                false
            }
        } catch (e: Exception) {
            Log.e(TAG, "Exception while deleting mint service", e)
            false
        }
    }
    
    /**
     * Check if mint service exists
     * @return true if mint service exists, false otherwise
     */
    fun mintServiceExists(): Boolean {
        return try {
            val result = native.mintExists()
            result == 1
        } catch (e: Exception) {
            Log.e(TAG, "Exception while checking mint existence", e)
            false
        }
    }
    
    /**
     * Get cleanup status and remaining files
     * @return Cleanup status JSON string
     */
    fun getCleanupStatus(): String {
        return try {
            Log.i(TAG, "Getting cleanup status...")
            val status = native.getCleanupStatus() ?: "{\"status\":\"unknown\"}"
            Log.i(TAG, "Cleanup status retrieved: $status")
            Log.i(TAG, "Note: Remaining files may include user data that is intentionally preserved")
            status
        } catch (e: Exception) {
            Log.e(TAG, "Failed to get cleanup status", e)
            "{\"status\":\"error\",\"error\":\"${e.message}\"}"
        }
    }
    
    /**
     * Get current Nostr account information from file
     * @return JSON string containing account info
     */
    fun getCurrentAccount(): String {
        return safeExecute("get current account") {
            val accountFile = File(getAccountFilePath())
            if (accountFile.exists()) {
                accountFile.readText()
            } else {
                "{\"account\":\"none\"}"
            }
        } ?: "{\"account\":\"error\",\"message\":\"Failed to get current account\"}"
    }
    
    /**
     * Get device IP address
     */
    fun getDeviceIpAddress(): String {
        return safeExecute("get device IP address") {
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
        } ?: "127.0.0.1"
    }
    
    /**
     * Test HTTP connection to mint service
     * @return true if connection successful
     */
    fun testHttpConnection(): Boolean {
        return safeExecute("test HTTP connection") {
            val deviceIp = getDeviceIpAddress()
            
            // First try localhost connection
            try {
                val localhostSocket = java.net.Socket()
                localhostSocket.connect(java.net.InetSocketAddress("127.0.0.1", 3338), 2000)
                val localhostConnected = localhostSocket.isConnected
                localhostSocket.close()
                
                if (localhostConnected) {
                    return@safeExecute true
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
                return@safeExecute connected
            } catch (e: Exception) {
                Log.e(TAG, "Device IP connection failed: ${e.message}")
                return@safeExecute false
            }
        } ?: false
    }

    // =============================================================================
    // New mode-specific methods
    // =============================================================================

    private var currentMode: ServiceMode = ServiceMode.LOCAL
    private var isRunning = false

    enum class ServiceMode {
        LOCAL,
        TOR
    }

    /**
     * Start mint service in local mode
     */
    fun startLocalMint(nsec: String): Boolean {
        Log.i(TAG, "Starting local mint service")
        
        try {
            createDirectories()
            ensureSslCertificatesExist()  // Ensure SSL certificates are copied
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
            
            val result = native.startLocalMint(config, nsec)
            if (result == 0) {
                currentMode = ServiceMode.LOCAL
                isRunning = true
                Log.i(TAG, "Local mint service started successfully")
                return true
            } else {
                Log.e(TAG, "Failed to start local mint service")
                return false
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error starting local mint service", e)
            return false
        }
    }

    /**
     * Start mint service in Tor mode
     */
    fun startTorMint(nsec: String): Boolean {
        Log.i(TAG, "Starting Tor mint service")
        
        try {
            createDirectories()
            ensureSslCertificatesExist()  // Ensure SSL certificates are copied
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
            val result = native.startTorMint(config, nsec)
            if (result == 0) {
                currentMode = ServiceMode.TOR
                isRunning = true
                Log.i(TAG, "Tor mint service started successfully")
                return true
            } else {
                Log.e(TAG, "Failed to start Tor mint service")
                return false
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error starting Tor mint service", e)
            return false
        }
    }

    /**
     * Get current service mode
     */
    fun getCurrentMode(): ServiceMode {
        return currentMode
    }

    /**
     * Check if service is running
     */
    fun isServiceRunning(): Boolean {
        return isRunning
    }

    /**
     * Get onion address if available
     * @return Onion address string or null if not available
     */
    fun getOnionAddress(): String? {
        return safeExecute("get onion address") {
            val address = native.getOnionAddress()
            if (address != null) {
                Log.i(TAG, "Onion address retrieved: $address")
            } else {
                Log.d(TAG, "No onion address available yet")
            }
            address
        }
    }

    /**
     * Get service status with detailed information
     * @return JSON string containing detailed service status
     */
    fun getDetailedServiceStatus(): String {
        return safeExecute("get detailed service status") {
            val status = getServiceStatus()
            val statusJson = JSONObject(status)
            
            // Add mode information
            statusJson.put("mode", currentMode.name.lowercase())
            statusJson.put("isRunning", isRunning)
            
            // Add onion address if in Tor mode
            if (currentMode == ServiceMode.TOR) {
                val onionAddress = getOnionAddress()
                if (onionAddress != null) {
                    statusJson.put("onionAddress", onionAddress)
                } else {
                    statusJson.put("onionAddress", "loading...")
                }
            }
            
            statusJson.toString()
        } ?: "{\"status\":\"error\",\"message\":\"Failed to get detailed service status\"}"
    }

    // Private helper methods to eliminate code duplication
    private fun ensureSslCertificatesExist() {
        try {
            val dataDir = getDataDir()
            val sslDir = File(dataDir, "ssl")
            if (!sslDir.exists()) {
                sslDir.mkdirs()
                Log.d(TAG, "Created SSL directory: ${sslDir.absolutePath}")
            }
            val certFile = File(sslDir, SSL_CERT_NAME)
            val keyFile = File(sslDir, SSL_KEY_NAME)
            
            Log.d(TAG, "SSL Certificate target path: ${certFile.absolutePath}")
            Log.d(TAG, "SSL Key target path: ${keyFile.absolutePath}")
            
            if (!certFile.exists()) {
                context.resources.openRawResource(R.raw.purrmint_cert_pem).use { input ->
                    certFile.outputStream().use { output ->
                        input.copyTo(output)
                    }
                }
                Log.i(TAG, "SSL certificate copied to: ${certFile.absolutePath}")
            } else {
                Log.d(TAG, "SSL certificate already exists at: ${certFile.absolutePath}")
            }
            if (!keyFile.exists()) {
                context.resources.openRawResource(R.raw.purrmint_key_pem).use { input ->
                    keyFile.outputStream().use { output ->
                        input.copyTo(output)
                    }
                }
                Log.i(TAG, "SSL private key copied to: ${keyFile.absolutePath}")
            } else {
                Log.d(TAG, "SSL private key already exists at: ${keyFile.absolutePath}")
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error ensuring SSL certificates exist", e)
        }
    }
    
    private fun <T> safeExecute(operation: String, block: () -> T): T? {
        return try {
            block()
        } catch (e: Exception) {
            Log.e(TAG, "Failed to $operation", e)
            null
        }
    }
} 