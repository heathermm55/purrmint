package com.purrmint.app.ui.activities

import android.content.Intent
import android.os.Bundle
import android.util.Log
import android.view.View
import android.widget.ArrayAdapter
import android.widget.AutoCompleteTextView
import androidx.appcompat.app.AppCompatActivity
import androidx.core.view.WindowCompat
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat
import com.google.android.material.button.MaterialButton
import com.google.android.material.textfield.TextInputEditText
import com.google.android.material.appbar.MaterialToolbar
import com.purrmint.app.R
import com.purrmint.app.core.managers.LanguageManager

class ConfigActivity : AppCompatActivity() {
    
    private lateinit var portInput: TextInputEditText
    private lateinit var mintNameInput: TextInputEditText
    private lateinit var descriptionInput: TextInputEditText
    private lateinit var btnCancel: MaterialButton
    private lateinit var btnStart: MaterialButton
    private lateinit var toolbar: MaterialToolbar
    
    // Global fee configuration views
    private lateinit var globalFeePercentInput: TextInputEditText
    private lateinit var globalReserveFeeMinInput: TextInputEditText
    
    // Lightning configuration views
    private lateinit var lightningBackendSpinner: AutoCompleteTextView
    private lateinit var clnConfigLayout: View
    private lateinit var lnbitsConfigLayout: View
    private lateinit var fakeWalletConfigLayout: View
    private lateinit var nwcConfigLayout: View
    
    // CLN inputs
    private lateinit var clnRpcPathInput: TextInputEditText
    
    // LNBits inputs
    private lateinit var lnbitsAdminApiKeyInput: TextInputEditText
    private lateinit var lnbitsInvoiceApiKeyInput: TextInputEditText
    private lateinit var lnbitsApiUrlInput: TextInputEditText
    
    // Fake Wallet inputs
    private lateinit var fakeWalletFeePercentInput: TextInputEditText
    private lateinit var fakeWalletReserveFeeMinInput: TextInputEditText
    
    // NWC inputs
    private lateinit var nwcConnectionUriInput: TextInputEditText
    
    companion object {
        private const val TAG = "ConfigActivity"
        const val EXTRA_PORT = "port"
        const val EXTRA_MINT_NAME = "mint_name"
        const val EXTRA_DESCRIPTION = "description"
        const val EXTRA_LIGHTNING_BACKEND = "lightning_backend"
        const val EXTRA_CLN_RPC_PATH = "cln_rpc_path"
        const val EXTRA_CLN_FEE_PERCENT = "cln_fee_percent"
        const val EXTRA_LNBITS_ADMIN_API_KEY = "lnbits_admin_api_key"
        const val EXTRA_LNBITS_INVOICE_API_KEY = "lnbits_invoice_api_key"
        const val EXTRA_LNBITS_API_URL = "lnbits_api_url"
        const val EXTRA_FAKE_WALLET_FEE_PERCENT = "fake_wallet_fee_percent"
        const val EXTRA_FAKE_WALLET_RESERVE_FEE_MIN = "fake_wallet_reserve_fee_min"
        const val EXTRA_NWC_CONNECTION_URI = "nwc_connection_uri"
        const val EXTRA_GLOBAL_FEE_PERCENT = "global_fee_percent"
        const val EXTRA_GLOBAL_RESERVE_FEE_MIN = "global_reserve_fee_min"
    }
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        // Enable edge-to-edge and handle window insets properly
        WindowCompat.setDecorFitsSystemWindows(window, false)
        
        // Apply current language
        val languageManager = LanguageManager(this)
        languageManager.updateConfiguration(resources)
        
        setContentView(R.layout.activity_config)
        
        initializeViews()
        setupWindowInsets()
        setupLightningBackendSpinner()
        setupClickListeners()
        loadDefaultValues()
    }
    
    private fun initializeViews() {
        portInput = findViewById(R.id.portInput)
        mintNameInput = findViewById(R.id.mintNameInput)
        descriptionInput = findViewById(R.id.descriptionInput)
        btnCancel = findViewById(R.id.btnCancel)
        btnStart = findViewById(R.id.btnStart)
        toolbar = findViewById(R.id.topAppBar)
        
        // Global fee configuration views
        globalFeePercentInput = findViewById(R.id.globalFeePercentInput)
        globalReserveFeeMinInput = findViewById(R.id.globalReserveFeeMinInput)
        
        // Lightning configuration views
        lightningBackendSpinner = findViewById(R.id.lightningBackendSpinner)
        clnConfigLayout = findViewById(R.id.clnConfigLayout)
        lnbitsConfigLayout = findViewById(R.id.lnbitsConfigLayout)
        fakeWalletConfigLayout = findViewById(R.id.fakeWalletConfigLayout)
        nwcConfigLayout = findViewById(R.id.nwcConfigLayout)
        
        // CLN inputs
        clnRpcPathInput = findViewById(R.id.clnRpcPathInput)
        
        // LNBits inputs
        lnbitsAdminApiKeyInput = findViewById(R.id.lnbitsAdminApiKeyInput)
        lnbitsInvoiceApiKeyInput = findViewById(R.id.lnbitsInvoiceApiKeyInput)
        lnbitsApiUrlInput = findViewById(R.id.lnbitsApiUrlInput)
        
        // Fake Wallet inputs
        fakeWalletFeePercentInput = findViewById(R.id.fakeWalletFeePercentInput)
        fakeWalletReserveFeeMinInput = findViewById(R.id.fakeWalletReserveFeeMinInput)
        
        // NWC inputs
        nwcConnectionUriInput = findViewById(R.id.nwcConnectionUriInput)
    }
    
    private fun setupLightningBackendSpinner() {
        val backends = arrayOf("fakewallet", "cln", "lnbits", "nwc")
        val adapter = ArrayAdapter(this, com.google.android.material.R.layout.support_simple_spinner_dropdown_item, backends)
        lightningBackendSpinner.setAdapter(adapter)
        
        lightningBackendSpinner.setOnItemClickListener { _, _, position, _ ->
            val selectedBackend = backends[position]
            updateLightningConfigVisibility(selectedBackend)
        }
    }
    
    private fun updateLightningConfigVisibility(backend: String) {
        // Hide all config layouts first
        clnConfigLayout.visibility = View.GONE
        lnbitsConfigLayout.visibility = View.GONE
        fakeWalletConfigLayout.visibility = View.GONE
        nwcConfigLayout.visibility = View.GONE // Hide NWC config by default
        
        // Show the appropriate config layout
        when (backend) {
            "cln" -> clnConfigLayout.visibility = View.VISIBLE
            "lnbits" -> lnbitsConfigLayout.visibility = View.VISIBLE
            "fakewallet" -> fakeWalletConfigLayout.visibility = View.VISIBLE
            "nwc" -> nwcConfigLayout.visibility = View.VISIBLE // Show NWC config
        }
    }
    
    private fun setupClickListeners() {
        toolbar.setNavigationOnClickListener {
            finish()
        }
        
        btnCancel.setOnClickListener {
            finish()
        }
        
        btnStart.setOnClickListener {
            saveConfiguration()
        }
    }
    
    private fun setupWindowInsets() {
        // Apply window insets to handle edge-to-edge display properly
        ViewCompat.setOnApplyWindowInsetsListener(findViewById(R.id.coordinatorLayout)) { view, insets ->
            val systemBars = insets.getInsets(WindowInsetsCompat.Type.systemBars())
            
            // Apply padding to the AppBarLayout for status bar
            val appBarLayout = findViewById<com.google.android.material.appbar.AppBarLayout>(R.id.appBarLayout)
            appBarLayout?.setPadding(0, systemBars.top, 0, 0)
            
            // Apply bottom padding to the nested scroll view for navigation bar
            val nestedScrollView = findViewById<androidx.core.widget.NestedScrollView>(R.id.nestedScrollView)
            nestedScrollView?.setPadding(
                nestedScrollView.paddingLeft,
                nestedScrollView.paddingTop,
                nestedScrollView.paddingRight,
                systemBars.bottom
            )
            
            insets
        }
    }
    
    private fun loadDefaultValues() {
        // Try to load existing configuration first
        val configManager = com.purrmint.app.core.managers.ConfigManager(this)
        val existingConfig = configManager.loadConfiguration()
        
        if (existingConfig != null) {
            // Load existing configuration
            portInput.setText(existingConfig.port.toString())
            mintNameInput.setText(existingConfig.mintName)
            descriptionInput.setText(existingConfig.description)
            lightningBackendSpinner.setText(existingConfig.lightningBackend, false)
            updateLightningConfigVisibility(existingConfig.lightningBackend)
            
            // Load global fee configuration with fallback to defaults
            val feePercent = existingConfig.feePercent ?: 0.02f
            val reserveFeeMin = existingConfig.reserveFeeMin ?: 1L
            
            globalFeePercentInput.setText(feePercent.toString())
            globalReserveFeeMinInput.setText(reserveFeeMin.toString())
            
            // Load NWC configuration if available
            if (existingConfig.lightningBackend == "nwc" && existingConfig.nwcConnectionUri != null) {
                nwcConnectionUriInput.setText(existingConfig.nwcConnectionUri)
            }
        } else {
            // Load default values
            portInput.setText("3338")
            mintNameInput.setText("My Mint")
            descriptionInput.setText("A simple mint service")
            lightningBackendSpinner.setText("fakewallet", false)
            updateLightningConfigVisibility("fakewallet")
            
            // Set default global fee values
            globalFeePercentInput.setText("0.02")
            globalReserveFeeMinInput.setText("1")
        }
    }
    
    private fun saveConfiguration() {
        try {
            val port = portInput.text.toString().toIntOrNull() ?: 3338
            val mintName = mintNameInput.text.toString()
            val description = descriptionInput.text.toString()
            val lightningBackend = lightningBackendSpinner.text.toString()
            
            // Get global fee configuration
            val feePercent = globalFeePercentInput.text.toString().toFloatOrNull() ?: 0.02f
            val reserveFeeMin = globalReserveFeeMinInput.text.toString().toLongOrNull() ?: 1L
            
            // Get Lightning backend specific configuration
            val lnbitsAdminApiKey = if (lightningBackend == "lnbits") lnbitsAdminApiKeyInput.text.toString() else null
            val lnbitsInvoiceApiKey = if (lightningBackend == "lnbits") lnbitsInvoiceApiKeyInput.text.toString() else null
            val lnbitsApiUrl = if (lightningBackend == "lnbits") lnbitsApiUrlInput.text.toString() else null
            
            val clnRpcPath = if (lightningBackend == "cln") clnRpcPathInput.text.toString() else null
            val clnBolt12 = if (lightningBackend == "cln") false else null  // Default to false for now
            
            val nwcConnectionUri = if (lightningBackend == "nwc") nwcConnectionUriInput.text.toString() else null
            
            val configManager = com.purrmint.app.core.managers.ConfigManager(this)
            val success = configManager.saveConfiguration(
                port = port,
                mintName = mintName,
                description = description,
                lightningBackend = lightningBackend,
                lnbitsAdminApiKey = lnbitsAdminApiKey,
                lnbitsInvoiceApiKey = lnbitsInvoiceApiKey,
                lnbitsApiUrl = lnbitsApiUrl,
                clnRpcPath = clnRpcPath,
                clnBolt12 = clnBolt12,
                feePercent = feePercent,
                reserveFeeMin = reserveFeeMin,
                nwcConnectionUri = nwcConnectionUri
            )
            
            if (success) {
                // Configuration saved successfully
                val intent = Intent()
                intent.putExtra(EXTRA_PORT, port)
                intent.putExtra(EXTRA_MINT_NAME, mintName)
                intent.putExtra(EXTRA_DESCRIPTION, description)
                intent.putExtra(EXTRA_LIGHTNING_BACKEND, lightningBackend)
                intent.putExtra(EXTRA_GLOBAL_FEE_PERCENT, feePercent)
                intent.putExtra(EXTRA_GLOBAL_RESERVE_FEE_MIN, reserveFeeMin)
                
                setResult(RESULT_OK, intent)
                finish()
            } else {
                // Failed to save configuration
                Log.e(TAG, "Failed to save configuration")
                // You can show an error message to the user here
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error saving configuration", e)
            // You can show an error message to the user here
        }
    }
} 