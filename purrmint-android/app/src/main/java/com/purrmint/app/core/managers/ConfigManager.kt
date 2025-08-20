package com.purrmint.app.core.managers

import android.content.Context
import android.util.Log
import org.json.JSONException
import org.json.JSONObject

data class AndroidConfig(
    val host: String,
    val port: Int,
    val mintName: String,
    val description: String,
    val lightningBackend: String,
    val databasePath: String,
    val logsPath: String,
    val lnbitsAdminApiKey: String? = null,
    val lnbitsInvoiceApiKey: String? = null,
    val lnbitsApiUrl: String? = null,
    val clnRpcPath: String? = null,
    val clnBolt12: Boolean? = null,
    // Global fee configuration for all Lightning backends
    val feePercent: Float? = null,
    val reserveFeeMin: Long? = null,
    val nwcConnectionUri: String? = null,
    // HTTPS configuration
    val enableHttps: Boolean = true,
    val httpsPort: Int = 8443,
    val sslCertPath: String? = null,  // Will be set dynamically
    val sslKeyPath: String? = null    // Will be set dynamically
)

class ConfigManager(private val context: Context) {
    
    private val purrmintManager = PurrmintManager(context)
    
    companion object {
        private const val TAG = "ConfigManager"
        private const val DEFAULT_HOST = "0.0.0.0"
        private const val DEFAULT_PORT = 3338
        private const val DEFAULT_MINT_NAME = "My Mint"
        private const val DEFAULT_DESCRIPTION = "A simple mint service"
        private const val DEFAULT_LIGHTNING_BACKEND = "fakewallet"
    }
    
    /**
     * Generate default configuration
     * @return AndroidConfig object with default values
     */
    fun generateDefaultConfig(): AndroidConfig {
        val dataDir = context.filesDir.absolutePath
        return AndroidConfig(
            host = DEFAULT_HOST,
            port = DEFAULT_PORT,
            mintName = DEFAULT_MINT_NAME,
            description = DEFAULT_DESCRIPTION,
            lightningBackend = DEFAULT_LIGHTNING_BACKEND,
            databasePath = "$dataDir/database",
            logsPath = "$dataDir/logs",
            lnbitsAdminApiKey = null,
            lnbitsInvoiceApiKey = null,
            lnbitsApiUrl = null,
            clnRpcPath = null,
            clnBolt12 = null,
            feePercent = 0.02f,        // Default 2% fee
            reserveFeeMin = 1L,         // Default 1 msat minimum
            nwcConnectionUri = null,
            enableHttps = true,
            httpsPort = 8443,
            sslCertPath = "$dataDir/ssl/purrmint_cert_pem",
            sslKeyPath = "$dataDir/ssl/purrmint_key_pem"
        )
    }
    
    /**
     * Save configuration to file
     * @param config The configuration to save
     * @return true if saved successfully
     */
    fun saveConfiguration(config: AndroidConfig): Boolean {
        return try {
            val json = configToJson(config)
            val success = purrmintManager.saveConfigToFile(json)
            
            if (success) {
                Log.i(TAG, "Configuration saved successfully")
            } else {
                Log.e(TAG, "Failed to save configuration")
            }
            
            success
        } catch (e: Exception) {
            Log.e(TAG, "Error saving configuration", e)
            false
        }
    }
    
    /**
     * Save configuration with individual parameters
     * @return true if saved successfully
     */
    fun saveConfiguration(
        host: String = DEFAULT_HOST,
        port: Int = DEFAULT_PORT,
        mintName: String = DEFAULT_MINT_NAME,
        description: String = DEFAULT_DESCRIPTION,
        lightningBackend: String = DEFAULT_LIGHTNING_BACKEND,
        lnbitsAdminApiKey: String? = null,
        lnbitsInvoiceApiKey: String? = null,
        lnbitsApiUrl: String? = null,
        clnRpcPath: String? = null,
        clnBolt12: Boolean? = null,
        feePercent: Float = 0.02f,     // Default 2% fee
        reserveFeeMin: Long = 1L,       // Default 1 msat minimum
        nwcConnectionUri: String? = null,
        enableHttps: Boolean = true,
        httpsPort: Int = 8443,
        sslCertPath: String? = null,  // Will be set dynamically
        sslKeyPath: String? = null    // Will be set dynamically
    ): Boolean {
        val dataDir = context.filesDir.absolutePath
        val config = AndroidConfig(
            host = host,
            port = port,
            mintName = mintName,
            description = description,
            lightningBackend = lightningBackend,
            databasePath = "$dataDir/database",
            logsPath = "$dataDir/logs",
            lnbitsAdminApiKey = lnbitsAdminApiKey,
            lnbitsInvoiceApiKey = lnbitsInvoiceApiKey,
            lnbitsApiUrl = lnbitsApiUrl,
            clnRpcPath = clnRpcPath,
            clnBolt12 = clnBolt12,
            feePercent = feePercent,
            reserveFeeMin = reserveFeeMin,
            nwcConnectionUri = nwcConnectionUri,
            enableHttps = enableHttps,
            httpsPort = httpsPort,
            sslCertPath = sslCertPath ?: "$dataDir/ssl/purrmint_cert_pem",
            sslKeyPath = sslKeyPath ?: "$dataDir/ssl/purrmint_key_pem"
        )
        return saveConfiguration(config)
    }
    
    /**
     * Load configuration from file
     * @return AndroidConfig object or null if failed
     */
    fun loadConfiguration(): AndroidConfig? {
        return try {
            val json = purrmintManager.loadConfigFromFile()
            if (json != null) {
                val config = jsonToConfig(json)
                Log.i(TAG, "Configuration loaded successfully")
                config
            } else {
                Log.w(TAG, "No configuration file found")
                null
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error loading configuration", e)
            null
        }
    }
    
    /**
     * Get configuration, loading from file or generating default
     * @return AndroidConfig object
     */
    fun getConfiguration(): AndroidConfig {
        return loadConfiguration() ?: generateDefaultConfig()
    }
    
    /**
     * Check if configuration file exists
     * @return true if configuration exists
     */
    fun hasConfiguration(): Boolean {
        return purrmintManager.configExists()
    }
    
    /**
     * Generate and save default configuration
     * @return true if generated and saved successfully
     */
    fun generateAndSaveDefaultConfig(): Boolean {
        return try {
            val defaultConfigJson = purrmintManager.generateDefaultConfig()
            if (defaultConfigJson != null) {
                val success = purrmintManager.saveConfigToFile(defaultConfigJson)
                if (success) {
                    Log.i(TAG, "Default configuration generated and saved")
                } else {
                    Log.e(TAG, "Failed to save default configuration")
                }
                success
            } else {
                Log.e(TAG, "Failed to generate default configuration")
                false
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error generating and saving default configuration", e)
            false
        }
    }
    
    /**
     * Clear configuration file
     * @return true if cleared successfully
     */
    fun clearConfiguration(): Boolean {
        return try {
            val configFile = java.io.File(context.filesDir, "android_config.json")
            val success = if (configFile.exists()) {
                configFile.delete()
            } else {
                true
            }
            
            if (success) {
                Log.i(TAG, "Configuration cleared successfully")
            } else {
                Log.e(TAG, "Failed to clear configuration")
            }
            
            success
        } catch (e: Exception) {
            Log.e(TAG, "Error clearing configuration", e)
            false
        }
    }
    
    /**
     * Convert AndroidConfig to JSON string
     */
    private fun configToJson(config: AndroidConfig): String {
        return try {
            val json = JSONObject()
            json.put("host", config.host)
            json.put("port", config.port)
            json.put("mintName", config.mintName)
            json.put("description", config.description)
            json.put("lightningBackend", config.lightningBackend)
            json.put("mode", "mintd_only")
            json.put("databasePath", config.databasePath)
            json.put("logsPath", config.logsPath)
            json.put("lnbitsAdminApiKey", config.lnbitsAdminApiKey)
            json.put("lnbitsInvoiceApiKey", config.lnbitsInvoiceApiKey)
            json.put("lnbitsApiUrl", config.lnbitsApiUrl)
            json.put("clnRpcPath", config.clnRpcPath)
            json.put("clnBolt12", config.clnBolt12)
            json.put("feePercent", config.feePercent)
            json.put("reserveFeeMin", config.reserveFeeMin)
            json.put("nwcConnectionUri", config.nwcConnectionUri)
            json.put("enableHttps", config.enableHttps)
            json.put("httpsPort", config.httpsPort)
            json.put("sslCertPath", config.sslCertPath)
            json.put("sslKeyPath", config.sslKeyPath)
            json.toString()
        } catch (e: JSONException) {
            Log.e(TAG, "Error converting config to JSON", e)
            throw e
        }
    }
    
    /**
     * Get port from configuration
     * @return port number or default port if not available
     */
    fun getPort(): Int? {
        return try {
            val config = loadConfiguration()
            if (config != null) {
                Log.i(TAG, "Port loaded from configuration: ${config.port}")
                config.port
            } else {
                Log.w(TAG, "No configuration found, using default port: $DEFAULT_PORT")
                DEFAULT_PORT
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error getting port from configuration", e)
            Log.w(TAG, "Using default port due to error: $DEFAULT_PORT")
            DEFAULT_PORT
        }
    }
    
    /**
     * Convert JSON string to AndroidConfig
     */
    private fun jsonToConfig(json: String): AndroidConfig {
        return try {
            val jsonObject = JSONObject(json)
            
            // Safely parse feePercent with proper null checking
            val feePercent = if (jsonObject.has("feePercent") && !jsonObject.isNull("feePercent")) {
                try {
                    val value = jsonObject.optDouble("feePercent")
                    if (value.isNaN() || value == 0.0) {
                        0.02f
                    } else {
                        value.toFloat()
                    }
                } catch (e: Exception) {
                    Log.e(TAG, "Error parsing feePercent, using default: 0.02f", e)
                    0.02f
                }
            } else {
                0.02f
            }
            
            // Safely parse reserveFeeMin with proper null checking
            val reserveFeeMin = if (jsonObject.has("reserveFeeMin") && !jsonObject.isNull("reserveFeeMin")) {
                try {
                    val value = jsonObject.optLong("reserveFeeMin")
                    if (value == 0L) {
                        1L
                    } else {
                        value
                    }
                } catch (e: Exception) {
                    Log.e(TAG, "Error parsing reserveFeeMin, using default: 1L", e)
                    1L
                }
            } else {
                1L
            }
            
            // Safely parse enableHttps with proper null checking
            val enableHttps = if (jsonObject.has("enableHttps") && !jsonObject.isNull("enableHttps")) {
                jsonObject.optBoolean("enableHttps")
            } else {
                true
            }

            // Safely parse httpsPort with proper null checking
            val httpsPort = if (jsonObject.has("httpsPort") && !jsonObject.isNull("httpsPort")) {
                jsonObject.optInt("httpsPort")
            } else {
                8443
            }

            // Safely parse sslCertPath with proper null checking
            val sslCertPath = if (jsonObject.has("sslCertPath") && !jsonObject.isNull("sslCertPath")) {
                jsonObject.optString("sslCertPath")
            } else {
                null
            }

            // Safely parse sslKeyPath with proper null checking
            val sslKeyPath = if (jsonObject.has("sslKeyPath") && !jsonObject.isNull("sslKeyPath")) {
                jsonObject.optString("sslKeyPath")
            } else {
                null
            }
            
            AndroidConfig(
                host = jsonObject.optString("host", DEFAULT_HOST),
                port = jsonObject.optInt("port", DEFAULT_PORT),
                mintName = jsonObject.optString("mintName", DEFAULT_MINT_NAME),
                description = jsonObject.optString("description", DEFAULT_DESCRIPTION),
                lightningBackend = jsonObject.optString("lightningBackend", DEFAULT_LIGHTNING_BACKEND),
                databasePath = jsonObject.optString("databasePath", "${context.filesDir.absolutePath}/database"),
                logsPath = jsonObject.optString("logsPath", "${context.filesDir.absolutePath}/logs"),
                lnbitsAdminApiKey = jsonObject.optString("lnbitsAdminApiKey", null),
                lnbitsInvoiceApiKey = jsonObject.optString("lnbitsInvoiceApiKey", null),
                lnbitsApiUrl = jsonObject.optString("lnbitsApiUrl", null),
                clnRpcPath = jsonObject.optString("clnRpcPath", null),
                clnBolt12 = if (jsonObject.has("clnBolt12")) jsonObject.optBoolean("clnBolt12") else null,
                feePercent = feePercent,
                reserveFeeMin = reserveFeeMin,
                nwcConnectionUri = jsonObject.optString("nwcConnectionUri", null),
                enableHttps = enableHttps,
                httpsPort = httpsPort,
                sslCertPath = sslCertPath,
                sslKeyPath = sslKeyPath
            )
        } catch (e: JSONException) {
            Log.e(TAG, "Error parsing JSON to config", e)
            throw e
        }
    }
} 