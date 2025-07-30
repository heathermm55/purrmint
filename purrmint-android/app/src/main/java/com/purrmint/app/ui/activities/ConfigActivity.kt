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
    
    // Lightning configuration views
    private lateinit var lightningBackendSpinner: AutoCompleteTextView
    private lateinit var clnConfigLayout: View
    private lateinit var lnbitsConfigLayout: View
    private lateinit var fakeWalletConfigLayout: View
    private lateinit var nwcConfigLayout: View
    
    // CLN inputs
    private lateinit var clnRpcPathInput: TextInputEditText
    private lateinit var clnFeePercentInput: TextInputEditText
    
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
        
        // Lightning configuration views
        lightningBackendSpinner = findViewById(R.id.lightningBackendSpinner)
        clnConfigLayout = findViewById(R.id.clnConfigLayout)
        lnbitsConfigLayout = findViewById(R.id.lnbitsConfigLayout)
        fakeWalletConfigLayout = findViewById(R.id.fakeWalletConfigLayout)
        nwcConfigLayout = findViewById(R.id.nwcConfigLayout)
        
        // CLN inputs
        clnRpcPathInput = findViewById(R.id.clnRpcPathInput)
        clnFeePercentInput = findViewById(R.id.clnFeePercentInput)
        
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
        val adapter = ArrayAdapter(this, android.R.layout.simple_dropdown_item_1line, backends)
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
            startService()
        }
    }
    
    private fun setupWindowInsets() {
        // Apply window insets to handle edge-to-edge display properly
        ViewCompat.setOnApplyWindowInsetsListener(findViewById(R.id.coordinatorLayout)) { view, insets ->
            val systemBars = insets.getInsets(WindowInsetsCompat.Type.systemBars())
            
            // Apply padding to the AppBarLayout for status bar
            val appBarLayout = findViewById<com.google.android.material.appbar.AppBarLayout>(R.id.topAppBar).parent as com.google.android.material.appbar.AppBarLayout
            appBarLayout.setPadding(0, systemBars.top, 0, 0)
            
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
            
            // Load NWC configuration if available
            if (existingConfig.lightningBackend == "nwc" && existingConfig.nwcConnectionUri != null) {
                nwcConnectionUriInput.setText(existingConfig.nwcConnectionUri)
            }
        } else {
            // Load default values
        portInput.setText("3338")
        mintNameInput.setText("My Mint")
        descriptionInput.setText("A simple mint service")
        
        // Set default lightning backend
        lightningBackendSpinner.setText("fakewallet", false)
        updateLightningConfigVisibility("fakewallet")
        }
    }
    
    private fun startService() {
        try {
            val port = portInput.text.toString().trim()
            val mintName = mintNameInput.text.toString().trim()
            val description = descriptionInput.text.toString().trim()
            val lightningBackend = lightningBackendSpinner.text.toString().trim()
            
            // Validate required inputs
            if (port.isEmpty() || mintName.isEmpty() || lightningBackend.isEmpty()) {
                Log.w(TAG, "Required fields are empty")
                return
            }
            
            // Validate lightning backend
            if (lightningBackend !in listOf("fakewallet", "cln", "lnbits", "nwc")) {
                Log.w(TAG, "Invalid lightning backend: '$lightningBackend'")
                return
            }
            
            // Create result intent
            val resultIntent = Intent().apply {
                putExtra(EXTRA_PORT, port)
                putExtra(EXTRA_MINT_NAME, mintName)
                putExtra(EXTRA_DESCRIPTION, description)
                putExtra(EXTRA_LIGHTNING_BACKEND, lightningBackend)
                
                // Add lightning-specific configuration
                when (lightningBackend) {
                    "cln" -> {
                        putExtra(EXTRA_CLN_RPC_PATH, clnRpcPathInput.text.toString().trim())
                        putExtra(EXTRA_CLN_FEE_PERCENT, clnFeePercentInput.text.toString().trim())
                    }
                    "lnbits" -> {
                        putExtra(EXTRA_LNBITS_ADMIN_API_KEY, lnbitsAdminApiKeyInput.text.toString().trim())
                        putExtra(EXTRA_LNBITS_INVOICE_API_KEY, lnbitsInvoiceApiKeyInput.text.toString().trim())
                        putExtra(EXTRA_LNBITS_API_URL, lnbitsApiUrlInput.text.toString().trim())
                    }
                    "fakewallet" -> {
                        putExtra(EXTRA_FAKE_WALLET_FEE_PERCENT, fakeWalletFeePercentInput.text.toString().trim())
                        putExtra(EXTRA_FAKE_WALLET_RESERVE_FEE_MIN, fakeWalletReserveFeeMinInput.text.toString().trim())
                    }
                    "nwc" -> {
                        putExtra(EXTRA_NWC_CONNECTION_URI, nwcConnectionUriInput.text.toString().trim())
                    }
                }
            }
            
            setResult(RESULT_OK, resultIntent)
            finish()
            
        } catch (e: Exception) {
            Log.e(TAG, "Error starting service", e)
        }
    }
} 