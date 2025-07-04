package com.purrmint.app

import android.content.Context
import android.content.SharedPreferences
import android.util.Log

data class MintConfig(
    val port: String,
    val host: String,
    val mintName: String,
    val description: String,
    val lightningBackend: String
)

class ConfigManager(context: Context) {
    
    private val sharedPreferences: SharedPreferences = context.getSharedPreferences(
        "mint_config", Context.MODE_PRIVATE
    )
    
    companion object {
        private const val TAG = "ConfigManager"
        private const val KEY_PORT = "port"
        private const val KEY_HOST = "host"
        private const val KEY_MINT_NAME = "mint_name"
        private const val KEY_DESCRIPTION = "description"
        private const val KEY_LIGHTNING_BACKEND = "lightning_backend"
        private const val KEY_HAS_CONFIG = "has_config"
    }
    
    fun saveConfiguration(port: String, host: String, mintName: String, description: String, lightningBackend: String) {
        try {
            sharedPreferences.edit().apply {
                putString(KEY_PORT, port)
                putString(KEY_HOST, host)
                putString(KEY_MINT_NAME, mintName)
                putString(KEY_DESCRIPTION, description)
                putString(KEY_LIGHTNING_BACKEND, lightningBackend)
                putBoolean(KEY_HAS_CONFIG, true)
            }.apply()
            
            Log.i(TAG, "Configuration saved successfully")
        } catch (e: Exception) {
            Log.e(TAG, "Error saving configuration", e)
        }
    }
    
    fun getConfiguration(): MintConfig {
        return MintConfig(
            port = sharedPreferences.getString(KEY_PORT, "3338") ?: "3338",
            host = sharedPreferences.getString(KEY_HOST, "0.0.0.0") ?: "0.0.0.0",
            mintName = sharedPreferences.getString(KEY_MINT_NAME, "My Mint") ?: "My Mint",
            description = sharedPreferences.getString(KEY_DESCRIPTION, "A simple mint service") ?: "A simple mint service",
            lightningBackend = sharedPreferences.getString(KEY_LIGHTNING_BACKEND, "fakewallet") ?: "fakewallet"
        )
    }
    
    fun hasConfiguration(): Boolean {
        return sharedPreferences.getBoolean(KEY_HAS_CONFIG, false)
    }
    
    fun clearConfiguration() {
        try {
            sharedPreferences.edit().clear().apply()
            Log.i(TAG, "Configuration cleared successfully")
        } catch (e: Exception) {
            Log.e(TAG, "Error clearing configuration", e)
        }
    }
} 