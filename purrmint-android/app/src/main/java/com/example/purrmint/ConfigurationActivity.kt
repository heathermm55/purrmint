package com.example.purrmint

import android.os.Bundle
import android.util.Log
import android.widget.Button
import android.widget.EditText
import android.widget.Toast
import androidx.appcompat.app.AppCompatActivity
import kotlinx.coroutines.*
import org.json.JSONObject

class ConfigurationActivity : AppCompatActivity() {
    
    private lateinit var mintIdentifierInput: EditText
    private lateinit var relayUrlsInput: EditText
    private lateinit var lightningBackendInput: EditText
    private lateinit var saveConfigButton: Button
    
    private val TAG = "ConfigurationActivity"
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_configuration)
        
        initializeViews()
        setupButtons()
        loadCurrentConfiguration()
    }
    
    private fun initializeViews() {
        mintIdentifierInput = findViewById(R.id.mintIdentifierInput)
        relayUrlsInput = findViewById(R.id.relayUrlsInput)
        lightningBackendInput = findViewById(R.id.lightningBackendInput)
        saveConfigButton = findViewById(R.id.saveConfigButton)
    }
    
    private fun setupButtons() {
        saveConfigButton.setOnClickListener { saveConfiguration() }
    }
    
    private fun loadCurrentConfiguration() {
        // TODO: Load current configuration from storage
        // For now, set default values
        mintIdentifierInput.setText("purrmint")
        relayUrlsInput.setText("wss://relay.damus.io\nwss://nos.lol")
        lightningBackendInput.setText("lnd")
    }
    
    private fun saveConfiguration() {
        val mintIdentifier = mintIdentifierInput.text.toString().trim()
        val relayUrls = relayUrlsInput.text.toString().trim()
        val lightningBackend = lightningBackendInput.text.toString().trim()
        
        if (mintIdentifier.isEmpty()) {
            Toast.makeText(this, "Please enter a mint identifier", Toast.LENGTH_SHORT).show()
            return
        }
        
        if (relayUrls.isEmpty()) {
            Toast.makeText(this, "Please enter at least one relay URL", Toast.LENGTH_SHORT).show()
            return
        }
        
        saveConfigButton.isEnabled = false
        
        CoroutineScope(Dispatchers.IO).launch {
            try {
                // Create configuration JSON
                val relayUrlsList = relayUrls.split("\n").filter { it.isNotEmpty() }
                val config = JSONObject().apply {
                    put("identifier", mintIdentifier)
                    put("relays", JSONObject().apply {
                        put("urls", relayUrlsList)
                    })
                    put("lightning_backend", JSONObject().apply {
                        put("type", lightningBackend)
                        put("config", JSONObject())
                    })
                }
                
                val result = PurrmintNative.configureMint(config.toString())
                val error = PurrmintNative.FfiError.fromCode(result)
                
                withContext(Dispatchers.Main) {
                    if (error == PurrmintNative.FfiError.SUCCESS) {
                        Toast.makeText(this@ConfigurationActivity, 
                            "Configuration saved successfully", 
                            Toast.LENGTH_SHORT).show()
                        finish()
                    } else {
                        Toast.makeText(this@ConfigurationActivity, 
                            "Failed to save configuration: ${error?.name ?: "Unknown error"}", 
                            Toast.LENGTH_SHORT).show()
                        saveConfigButton.isEnabled = true
                    }
                }
            } catch (e: Exception) {
                withContext(Dispatchers.Main) {
                    Toast.makeText(this@ConfigurationActivity, 
                        "Error saving configuration: ${e.message}", 
                        Toast.LENGTH_SHORT).show()
                    saveConfigButton.isEnabled = true
                }
                Log.e(TAG, "Error saving configuration", e)
            }
        }
    }
} 