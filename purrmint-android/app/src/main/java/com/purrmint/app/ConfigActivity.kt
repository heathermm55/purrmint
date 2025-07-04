package com.purrmint.app

import android.content.Intent
import android.os.Bundle
import android.util.Log
import android.view.View
import android.widget.ArrayAdapter
import android.widget.AutoCompleteTextView
import androidx.appcompat.app.AppCompatActivity
import com.google.android.material.button.MaterialButton
import com.google.android.material.textfield.TextInputEditText
import com.google.android.material.appbar.MaterialToolbar

class ConfigActivity : AppCompatActivity() {
    
    private lateinit var portInput: TextInputEditText
    private lateinit var hostInput: TextInputEditText
    private lateinit var mintNameInput: TextInputEditText
    private lateinit var descriptionInput: TextInputEditText
    private lateinit var btnCancel: MaterialButton
    private lateinit var btnStart: MaterialButton
    private lateinit var toolbar: MaterialToolbar
    
    // Lightning configuration views
    private lateinit var lightningBackendSpinner: AutoCompleteTextView
    private lateinit var clnConfigLayout: View
    private lateinit var lndConfigLayout: View
    private lateinit var lnbitsConfigLayout: View
    private lateinit var fakeWalletConfigLayout: View
    
    // CLN inputs
    private lateinit var clnRpcPathInput: TextInputEditText
    private lateinit var clnFeePercentInput: TextInputEditText
    
    // LND inputs
    private lateinit var lndAddressInput: TextInputEditText
    private lateinit var lndMacaroonFileInput: TextInputEditText
    private lateinit var lndCertFileInput: TextInputEditText
    
    // LNBits inputs
    private lateinit var lnbitsAdminApiKeyInput: TextInputEditText
    private lateinit var lnbitsInvoiceApiKeyInput: TextInputEditText
    private lateinit var lnbitsApiUrlInput: TextInputEditText
    
    // Fake Wallet inputs
    private lateinit var fakeWalletFeePercentInput: TextInputEditText
    private lateinit var fakeWalletReserveFeeMinInput: TextInputEditText
    
    companion object {
        private const val TAG = "ConfigActivity"
        const val EXTRA_PORT = "port"
        const val EXTRA_HOST = "host"
        const val EXTRA_MINT_NAME = "mint_name"
        const val EXTRA_DESCRIPTION = "description"
        const val EXTRA_LIGHTNING_BACKEND = "lightning_backend"
        const val EXTRA_CLN_RPC_PATH = "cln_rpc_path"
        const val EXTRA_CLN_FEE_PERCENT = "cln_fee_percent"
        const val EXTRA_LND_ADDRESS = "lnd_address"
        const val EXTRA_LND_MACAROON_FILE = "lnd_macaroon_file"
        const val EXTRA_LND_CERT_FILE = "lnd_cert_file"
        const val EXTRA_LNBITS_ADMIN_API_KEY = "lnbits_admin_api_key"
        const val EXTRA_LNBITS_INVOICE_API_KEY = "lnbits_invoice_api_key"
        const val EXTRA_LNBITS_API_URL = "lnbits_api_url"
        const val EXTRA_FAKE_WALLET_FEE_PERCENT = "fake_wallet_fee_percent"
        const val EXTRA_FAKE_WALLET_RESERVE_FEE_MIN = "fake_wallet_reserve_fee_min"
    }
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_config)
        
        initializeViews()
        setupLightningBackendSpinner()
        setupClickListeners()
        loadDefaultValues()
    }
    
    private fun initializeViews() {
        portInput = findViewById(R.id.portInput)
        hostInput = findViewById(R.id.hostInput)
        mintNameInput = findViewById(R.id.mintNameInput)
        descriptionInput = findViewById(R.id.descriptionInput)
        btnCancel = findViewById(R.id.btnCancel)
        btnStart = findViewById(R.id.btnStart)
        toolbar = findViewById(R.id.topAppBar)
        
        // Lightning configuration views
        lightningBackendSpinner = findViewById(R.id.lightningBackendSpinner)
        clnConfigLayout = findViewById(R.id.clnConfigLayout)
        lndConfigLayout = findViewById(R.id.lndConfigLayout)
        lnbitsConfigLayout = findViewById(R.id.lnbitsConfigLayout)
        fakeWalletConfigLayout = findViewById(R.id.fakeWalletConfigLayout)
        
        // CLN inputs
        clnRpcPathInput = findViewById(R.id.clnRpcPathInput)
        clnFeePercentInput = findViewById(R.id.clnFeePercentInput)
        
        // LND inputs
        lndAddressInput = findViewById(R.id.lndAddressInput)
        lndMacaroonFileInput = findViewById(R.id.lndMacaroonFileInput)
        lndCertFileInput = findViewById(R.id.lndCertFileInput)
        
        // LNBits inputs
        lnbitsAdminApiKeyInput = findViewById(R.id.lnbitsAdminApiKeyInput)
        lnbitsInvoiceApiKeyInput = findViewById(R.id.lnbitsInvoiceApiKeyInput)
        lnbitsApiUrlInput = findViewById(R.id.lnbitsApiUrlInput)
        
        // Fake Wallet inputs
        fakeWalletFeePercentInput = findViewById(R.id.fakeWalletFeePercentInput)
        fakeWalletReserveFeeMinInput = findViewById(R.id.fakeWalletReserveFeeMinInput)
    }
    
    private fun setupLightningBackendSpinner() {
        val backends = arrayOf("fakewallet", "cln", "lnd", "lnbits")
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
        lndConfigLayout.visibility = View.GONE
        lnbitsConfigLayout.visibility = View.GONE
        fakeWalletConfigLayout.visibility = View.GONE
        
        // Show the appropriate config layout
        when (backend) {
            "cln" -> clnConfigLayout.visibility = View.VISIBLE
            "lnd" -> lndConfigLayout.visibility = View.VISIBLE
            "lnbits" -> lnbitsConfigLayout.visibility = View.VISIBLE
            "fakewallet" -> fakeWalletConfigLayout.visibility = View.VISIBLE
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
    
    private fun loadDefaultValues() {
        // Load default values or previously saved values
        portInput.setText("3338")
        hostInput.setText("0.0.0.0")
        mintNameInput.setText("My Mint")
        descriptionInput.setText("A simple mint service")
        
        // Set default lightning backend
        lightningBackendSpinner.setText("fakewallet", false)
        updateLightningConfigVisibility("fakewallet")
    }
    
    private fun startService() {
        try {
            val port = portInput.text.toString().trim()
            val host = hostInput.text.toString().trim()
            val mintName = mintNameInput.text.toString().trim()
            val description = descriptionInput.text.toString().trim()
            val lightningBackend = lightningBackendSpinner.text.toString().trim()
            
            // Validate required inputs
            if (port.isEmpty() || host.isEmpty() || mintName.isEmpty() || lightningBackend.isEmpty()) {
                Log.w(TAG, "Required fields are empty")
                return
            }
            
            // Create result intent
            val resultIntent = Intent().apply {
                putExtra(EXTRA_PORT, port)
                putExtra(EXTRA_HOST, host)
                putExtra(EXTRA_MINT_NAME, mintName)
                putExtra(EXTRA_DESCRIPTION, description)
                putExtra(EXTRA_LIGHTNING_BACKEND, lightningBackend)
                
                // Add lightning-specific configuration
                when (lightningBackend) {
                    "cln" -> {
                        putExtra(EXTRA_CLN_RPC_PATH, clnRpcPathInput.text.toString().trim())
                        putExtra(EXTRA_CLN_FEE_PERCENT, clnFeePercentInput.text.toString().trim())
                    }
                    "lnd" -> {
                        putExtra(EXTRA_LND_ADDRESS, lndAddressInput.text.toString().trim())
                        putExtra(EXTRA_LND_MACAROON_FILE, lndMacaroonFileInput.text.toString().trim())
                        putExtra(EXTRA_LND_CERT_FILE, lndCertFileInput.text.toString().trim())
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
                }
            }
            
            setResult(RESULT_OK, resultIntent)
            finish()
            
        } catch (e: Exception) {
            Log.e(TAG, "Error starting service", e)
        }
    }
} 