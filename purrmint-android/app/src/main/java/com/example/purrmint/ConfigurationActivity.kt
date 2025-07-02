package com.example.purrmint

import android.os.Bundle
import android.util.Log
import android.view.View
import android.widget.*
import androidx.appcompat.app.AppCompatActivity
import com.google.android.material.textfield.TextInputEditText
import kotlinx.coroutines.*
import org.json.JSONObject

class ConfigurationActivity : AppCompatActivity() {
    
    private lateinit var mintIdentifierInput: TextInputEditText
    private lateinit var relayUrlsInput: TextInputEditText
    private lateinit var lightningBackendTypeSpinner: AutoCompleteTextView
    private lateinit var saveConfigButton: com.google.android.material.button.MaterialButton
    
    // Lightning config layouts
    private lateinit var lightningConfigContainer: LinearLayout
    private lateinit var clnConfigLayout: LinearLayout
    private lateinit var lndConfigLayout: LinearLayout
    private lateinit var lnbitsConfigLayout: LinearLayout
    private lateinit var fakeWalletConfigLayout: LinearLayout
    
    // CLN inputs
    private lateinit var clnRpcPathInput: TextInputEditText
    private lateinit var clnFeePercentInput: TextInputEditText
    private lateinit var clnReserveFeeMinInput: TextInputEditText
    
    // LND inputs
    private lateinit var lndAddressInput: TextInputEditText
    private lateinit var lndMacaroonFileInput: TextInputEditText
    private lateinit var lndCertFileInput: TextInputEditText
    private lateinit var lndFeePercentInput: TextInputEditText
    private lateinit var lndReserveFeeMinInput: TextInputEditText
    
    // LNBits inputs
    private lateinit var lnbitsAdminApiKeyInput: TextInputEditText
    private lateinit var lnbitsInvoiceApiKeyInput: TextInputEditText
    private lateinit var lnbitsApiUrlInput: TextInputEditText
    private lateinit var lnbitsRetroApiCheckbox: com.google.android.material.checkbox.MaterialCheckBox
    
    // Fake Wallet inputs
    private lateinit var fakeWalletSupportedUnitsInput: TextInputEditText
    private lateinit var fakeWalletFeePercentInput: TextInputEditText
    private lateinit var fakeWalletReserveFeeMinInput: TextInputEditText
    private lateinit var fakeWalletMinDelayTimeInput: TextInputEditText
    private lateinit var fakeWalletMaxDelayTimeInput: TextInputEditText
    
    private val TAG = "ConfigurationActivity"
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_configuration)
        
        initializeViews()
        setupLightningBackendSpinner()
        setupButtons()
        loadCurrentConfiguration()
    }
    
    private fun initializeViews() {
        // Basic inputs
        mintIdentifierInput = findViewById(R.id.mintIdentifierInput)
        relayUrlsInput = findViewById(R.id.relayUrlsInput)
        lightningBackendTypeSpinner = findViewById(R.id.lightningBackendTypeSpinner)
        saveConfigButton = findViewById(R.id.saveConfigButton)
        
        // Lightning config container
        lightningConfigContainer = findViewById(R.id.lightningConfigContainer)
        
        // CLN inputs
        clnConfigLayout = findViewById(R.id.clnConfigLayout)
        clnRpcPathInput = findViewById(R.id.clnRpcPathInput)
        clnFeePercentInput = findViewById(R.id.clnFeePercentInput)
        clnReserveFeeMinInput = findViewById(R.id.clnReserveFeeMinInput)
        
        // LND inputs
        lndConfigLayout = findViewById(R.id.lndConfigLayout)
        lndAddressInput = findViewById(R.id.lndAddressInput)
        lndMacaroonFileInput = findViewById(R.id.lndMacaroonFileInput)
        lndCertFileInput = findViewById(R.id.lndCertFileInput)
        lndFeePercentInput = findViewById(R.id.lndFeePercentInput)
        lndReserveFeeMinInput = findViewById(R.id.lndReserveFeeMinInput)
        
        // LNBits inputs
        lnbitsConfigLayout = findViewById(R.id.lnbitsConfigLayout)
        lnbitsAdminApiKeyInput = findViewById(R.id.lnbitsAdminApiKeyInput)
        lnbitsInvoiceApiKeyInput = findViewById(R.id.lnbitsInvoiceApiKeyInput)
        lnbitsApiUrlInput = findViewById(R.id.lnbitsApiUrlInput)
        lnbitsRetroApiCheckbox = findViewById(R.id.lnbitsRetroApiCheckbox)
        
        // Fake Wallet inputs
        fakeWalletConfigLayout = findViewById(R.id.fakeWalletConfigLayout)
        fakeWalletSupportedUnitsInput = findViewById(R.id.fakeWalletSupportedUnitsInput)
        fakeWalletFeePercentInput = findViewById(R.id.fakeWalletFeePercentInput)
        fakeWalletReserveFeeMinInput = findViewById(R.id.fakeWalletReserveFeeMinInput)
        fakeWalletMinDelayTimeInput = findViewById(R.id.fakeWalletMinDelayTimeInput)
        fakeWalletMaxDelayTimeInput = findViewById(R.id.fakeWalletMaxDelayTimeInput)
    }
    
    private fun setupLightningBackendSpinner() {
        val backendTypes = arrayOf("cln", "lnd", "lnbits", "fake_wallet")
        val adapter = ArrayAdapter(this, android.R.layout.simple_dropdown_item_1line, backendTypes)
        lightningBackendTypeSpinner.setAdapter(adapter)
        
        lightningBackendTypeSpinner.setOnItemClickListener { _, _, position, _ ->
            val selectedType = backendTypes[position]
            showLightningConfig(selectedType)
        }
    }
    
    private fun showLightningConfig(backendType: String) {
        // Hide all config layouts
        clnConfigLayout.visibility = View.GONE
        lndConfigLayout.visibility = View.GONE
        lnbitsConfigLayout.visibility = View.GONE
        fakeWalletConfigLayout.visibility = View.GONE
        
        // Show selected config layout
        when (backendType) {
            "cln" -> clnConfigLayout.visibility = View.VISIBLE
            "lnd" -> lndConfigLayout.visibility = View.VISIBLE
            "lnbits" -> lnbitsConfigLayout.visibility = View.VISIBLE
            "fake_wallet" -> fakeWalletConfigLayout.visibility = View.VISIBLE
        }
    }
    
    private fun setupButtons() {
        saveConfigButton.setOnClickListener { saveConfiguration() }
    }
    
    private fun loadCurrentConfiguration() {
        // TODO: Load current configuration from storage
        // For now, set default values
        mintIdentifierInput.setText("purrmint")
        relayUrlsInput.setText("wss://relay.damus.io\nwss://nos.lol")
        lightningBackendTypeSpinner.setText("cln", false)
        showLightningConfig("cln")
        
        // Set default values for CLN
        clnFeePercentInput.setText("0.04")
        clnReserveFeeMinInput.setText("4")
        
        // Set default values for LND
        lndFeePercentInput.setText("0.04")
        lndReserveFeeMinInput.setText("4")
        
        // Set default values for Fake Wallet
        fakeWalletSupportedUnitsInput.setText("sat")
        fakeWalletFeePercentInput.setText("0.02")
        fakeWalletReserveFeeMinInput.setText("1")
        fakeWalletMinDelayTimeInput.setText("1")
        fakeWalletMaxDelayTimeInput.setText("3")
    }
    
    private fun saveConfiguration() {
        val mintIdentifier = mintIdentifierInput.text.toString().trim()
        val relayUrls = relayUrlsInput.text.toString().trim()
        val lightningBackendType = lightningBackendTypeSpinner.text.toString().trim()
        
        if (mintIdentifier.isEmpty()) {
            Toast.makeText(this, "Please enter a mint identifier", Toast.LENGTH_SHORT).show()
            return
        }
        
        if (relayUrls.isEmpty()) {
            Toast.makeText(this, "Please enter at least one relay URL", Toast.LENGTH_SHORT).show()
            return
        }
        
        if (lightningBackendType.isEmpty()) {
            Toast.makeText(this, "Please select a lightning backend", Toast.LENGTH_SHORT).show()
            return
        }
        
        saveConfigButton.isEnabled = false
        
        CoroutineScope(Dispatchers.IO).launch {
            try {
                // Create configuration JSON
                val relayUrlsList = relayUrls.split("\n").filter { it.isNotEmpty() }
                val lightningConfig = createLightningConfig(lightningBackendType)
                
                val config = JSONObject().apply {
                    put("identifier", mintIdentifier)
                    put("relays", JSONObject().apply {
                        put("urls", relayUrlsList)
                    })
                    put("lightning_backend", lightningConfig)
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
    
    private fun createLightningConfig(backendType: String): JSONObject {
        return when (backendType) {
            "cln" -> {
                val rpcPath = clnRpcPathInput.text.toString().trim()
                val feePercent = clnFeePercentInput.text.toString().trim().toDoubleOrNull() ?: 0.04
                val reserveFeeMin = clnReserveFeeMinInput.text.toString().trim().toIntOrNull() ?: 4
                
                JSONObject().apply {
                    put("type", "cln")
                    put("config", JSONObject().apply {
                        put("rpc_path", rpcPath)
                        put("fee_percent", feePercent)
                        put("reserve_fee_min", reserveFeeMin)
                    })
                }
            }
            "lnd" -> {
                val address = lndAddressInput.text.toString().trim()
                val macaroonFile = lndMacaroonFileInput.text.toString().trim()
                val certFile = lndCertFileInput.text.toString().trim()
                val feePercent = lndFeePercentInput.text.toString().trim().toDoubleOrNull() ?: 0.04
                val reserveFeeMin = lndReserveFeeMinInput.text.toString().trim().toIntOrNull() ?: 4
                
                JSONObject().apply {
                    put("type", "lnd")
                    put("config", JSONObject().apply {
                        put("address", address)
                        put("macaroon_file", macaroonFile)
                        put("cert_file", certFile)
                        put("fee_percent", feePercent)
                        put("reserve_fee_min", reserveFeeMin)
                    })
                }
            }
            "lnbits" -> {
                val adminApiKey = lnbitsAdminApiKeyInput.text.toString().trim()
                val invoiceApiKey = lnbitsInvoiceApiKeyInput.text.toString().trim()
                val apiUrl = lnbitsApiUrlInput.text.toString().trim()
                val retroApi = lnbitsRetroApiCheckbox.isChecked
                
                JSONObject().apply {
                    put("type", "lnbits")
                    put("config", JSONObject().apply {
                        put("admin_api_key", adminApiKey)
                        put("invoice_api_key", invoiceApiKey)
                        put("lnbits_api", apiUrl)
                        put("retro_api", retroApi)
                    })
                }
            }
            "fake_wallet" -> {
                val supportedUnits = fakeWalletSupportedUnitsInput.text.toString().trim()
                val feePercent = fakeWalletFeePercentInput.text.toString().trim().toDoubleOrNull() ?: 0.02
                val reserveFeeMin = fakeWalletReserveFeeMinInput.text.toString().trim().toIntOrNull() ?: 1
                val minDelayTime = fakeWalletMinDelayTimeInput.text.toString().trim().toIntOrNull() ?: 1
                val maxDelayTime = fakeWalletMaxDelayTimeInput.text.toString().trim().toIntOrNull() ?: 3
                
                JSONObject().apply {
                    put("type", "fake_wallet")
                    put("config", JSONObject().apply {
                        put("supported_units", supportedUnits.split(",").map { it.trim() })
                        put("fee_percent", feePercent)
                        put("reserve_fee_min", reserveFeeMin)
                        put("min_delay_time", minDelayTime)
                        put("max_delay_time", maxDelayTime)
                    })
                }
            }
            else -> {
                JSONObject().apply {
                    put("type", backendType)
                    put("config", JSONObject())
                }
            }
        }
    }
} 