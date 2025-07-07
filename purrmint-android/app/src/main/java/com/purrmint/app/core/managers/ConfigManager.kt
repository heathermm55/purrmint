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
    val lnbitsApiUrl: String? = null
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
            lnbitsApiUrl = null
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
        lnbitsApiUrl: String? = null
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
            lnbitsApiUrl = lnbitsApiUrl
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
            json.put("mint_name", config.mintName)
            json.put("description", config.description)
            json.put("lightning_backend", config.lightningBackend)
            json.put("database_path", config.databasePath)
            json.put("logs_path", config.logsPath)
            json.put("lnbits_admin_api_key", config.lnbitsAdminApiKey)
            json.put("lnbits_invoice_api_key", config.lnbitsInvoiceApiKey)
            json.put("lnbits_api_url", config.lnbitsApiUrl)
            json.toString()
        } catch (e: JSONException) {
            Log.e(TAG, "Error converting config to JSON", e)
            throw e
        }
    }
    
    /**
     * Convert JSON string to AndroidConfig
     */
    private fun jsonToConfig(json: String): AndroidConfig {
        return try {
            val jsonObject = JSONObject(json)
            AndroidConfig(
                host = jsonObject.optString("host", DEFAULT_HOST),
                port = jsonObject.optInt("port", DEFAULT_PORT),
                mintName = jsonObject.optString("mint_name", DEFAULT_MINT_NAME),
                description = jsonObject.optString("description", DEFAULT_DESCRIPTION),
                lightningBackend = jsonObject.optString("lightning_backend", DEFAULT_LIGHTNING_BACKEND),
                databasePath = jsonObject.optString("database_path", "${context.filesDir.absolutePath}/database"),
                logsPath = jsonObject.optString("logs_path", "${context.filesDir.absolutePath}/logs"),
                lnbitsAdminApiKey = jsonObject.optString("lnbits_admin_api_key", null),
                lnbitsInvoiceApiKey = jsonObject.optString("lnbits_invoice_api_key", null),
                lnbitsApiUrl = jsonObject.optString("lnbits_api_url", null)
            )
        } catch (e: JSONException) {
            Log.e(TAG, "Error parsing JSON to config", e)
            throw e
        }
    }
} 