package com.purrmint.app.ui.activities

import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.ServiceConnection
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.os.IBinder
import android.os.PowerManager
import android.provider.Settings
import android.util.Log
import android.view.Menu
import android.view.MenuItem
import com.purrmint.app.core.managers.LanguageManager
import android.widget.Button
import android.widget.TextView
import android.widget.ImageView
import android.widget.ImageButton
import android.widget.LinearLayout
import androidx.appcompat.app.AppCompatActivity
import androidx.drawerlayout.widget.DrawerLayout
import com.google.android.material.navigation.NavigationView
import com.google.android.material.appbar.MaterialToolbar
import com.google.android.material.button.MaterialButton
import com.google.android.material.textfield.TextInputEditText
import com.google.android.material.chip.Chip
import android.widget.Toast
import com.purrmint.app.R
import com.purrmint.app.core.managers.LoginManager
import com.purrmint.app.core.managers.ConfigManager
import com.purrmint.app.core.managers.PurrmintManager
import com.purrmint.app.core.services.PurrmintService
import com.purrmint.app.ui.dialogs.QRCodeDialog
import android.os.Handler
import android.os.Looper
import androidx.core.view.WindowCompat
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat

class MainActivity : AppCompatActivity(), NavigationView.OnNavigationItemSelectedListener {
    
    // UI Components
    private lateinit var drawerLayout: DrawerLayout
    private lateinit var navigationView: NavigationView
    private lateinit var toolbar: MaterialToolbar
    private lateinit var btnConfig: ImageButton
    private lateinit var statusIcon: ImageView
    private lateinit var statusChip: Chip
    private lateinit var statusTextView: TextView
    private lateinit var startButton: MaterialButton
    private lateinit var clearLogsButton: MaterialButton
    private lateinit var logsText: TextView
    
    // Mode selection components
    private lateinit var modeChipGroup: com.google.android.material.chip.ChipGroup
    private lateinit var localModeChip: com.google.android.material.chip.Chip
    private lateinit var torModeChip: com.google.android.material.chip.Chip
    private lateinit var addressLayout: LinearLayout
    private lateinit var addressTitle: TextView
    private lateinit var addressText: TextView
    private lateinit var copyAddressButton: ImageButton
    private lateinit var qrCodeButton: ImageButton
    
    // Service
    private var purrmintService: PurrmintService.LocalBinder? = null
    private var isServiceBound = false
    private var isLoggedIn = false
    private var isMintRunning = false
    private var isServiceStarting = false
    
    // Mode state
    private var currentMode: PurrmintManager.ServiceMode = PurrmintManager.ServiceMode.LOCAL
    private var onionAddress: String? = null
    private var localAddress: String? = null
    
    // Login Manager
    private lateinit var loginManager: LoginManager
    
    // Configuration Manager
    private lateinit var configManager: ConfigManager
    
    // Language Manager
    private lateinit var languageManager: LanguageManager
    
    companion object {
        private const val TAG = "MainActivity"
        private const val REQUEST_CONFIG = 1001
        private const val REQUEST_LOGIN = 1002
        private const val REQUEST_ACCOUNT = 1003
        private const val REQUEST_LANGUAGE = 1004
    }
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        // Enable edge-to-edge and handle window insets properly
        WindowCompat.setDecorFitsSystemWindows(window, false)
        
        // Initialize managers
        loginManager = LoginManager(this)
        configManager = ConfigManager(this)
        languageManager = LanguageManager(this)
        
        // Apply current language
        languageManager.updateConfiguration(resources)
        
        // Check login status
        if (!loginManager.isLoggedIn()) {
            // Not logged in, go to login activity
            startLoginActivity()
            return
        }
        
        // Already logged in, show main interface
        setContentView(R.layout.activity_main)

        // Initialize UI components
        initializeViews()
        
        // Handle window insets for edge-to-edge display
        setupWindowInsets()
        
        // Start foreground service (only after login)
        val intent = Intent(this, PurrmintService::class.java)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            startForegroundService(intent)
        } else {
            startService(intent)
        }

        // Bind to PurrmintService
        bindPurrmintService()

        // Setup click listeners
        setupClickListeners()

        // Show logged in state
        showLoggedInState()
        
        // Request battery optimization exemption
        requestBatteryOptimizationExemption()
        
        // Clear default logs text and show welcome message
        runOnUiThread {
            logsText.text = ""
        }
        appendLog("Welcome to Purrmint!")
        appendLog("✅ Login successful!")
        
        // Check initial state and show appropriate UI
        checkInitialState()
    }
    
    private fun startLoginActivity() {
        val intent = Intent(this, LoginActivity::class.java)
        startActivityForResult(intent, REQUEST_LOGIN)
    }
    
    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        
        when (requestCode) {
            REQUEST_LOGIN -> {
                if (resultCode == RESULT_OK && data != null) {
                    val loginSuccess = data.getBooleanExtra(LoginActivity.EXTRA_LOGIN_SUCCESS, false)
                    if (loginSuccess) {
                        // Login successful, restart activity to show main interface
                        recreate()
                    } else {
                        // Login failed or cancelled, finish activity
                        finish()
                    }
                } else {
                    // Login cancelled, finish activity
                    finish()
                }
            }
            REQUEST_CONFIG -> {
                if (resultCode == RESULT_OK && data != null) {
                    // Get configuration from ConfigActivity
                    val port = data.getStringExtra(ConfigActivity.EXTRA_PORT)?.toIntOrNull() ?: 3338
                    val mintName = data.getStringExtra(ConfigActivity.EXTRA_MINT_NAME) ?: "My Mint"
                    val description = data.getStringExtra(ConfigActivity.EXTRA_DESCRIPTION) ?: "A simple mint service"
                    val lightningBackend = data.getStringExtra(ConfigActivity.EXTRA_LIGHTNING_BACKEND) ?: "fakewallet"
                    
                    // Get lightning-specific configuration
                    val lnbitsAdminApiKey = data.getStringExtra(ConfigActivity.EXTRA_LNBITS_ADMIN_API_KEY)
                    val lnbitsInvoiceApiKey = data.getStringExtra(ConfigActivity.EXTRA_LNBITS_INVOICE_API_KEY)
                    val lnbitsApiUrl = data.getStringExtra(ConfigActivity.EXTRA_LNBITS_API_URL)
                    val nwcConnectionUri = data.getStringExtra(ConfigActivity.EXTRA_NWC_CONNECTION_URI)
                    
                    appendLog("Configuration received:")
                    appendLog("  Port: $port")
                    appendLog("  Mint Name: $mintName")
                    appendLog("  Description: $description")
                    appendLog("  Lightning Backend: $lightningBackend")
                    
                    if (lightningBackend == "lnbits") {
                        appendLog("  LNBits Admin API Key: ${lnbitsAdminApiKey?.take(8)}...")
                        appendLog("  LNBits Invoice API Key: ${lnbitsInvoiceApiKey?.take(8)}...")
                        appendLog("  LNBits API URL: $lnbitsApiUrl")
                    } else if (lightningBackend == "nwc") {
                        appendLog("  NWC Connection URI: ${nwcConnectionUri?.take(20)}...")
                    }
                    
                    // Configuration is already saved in ConfigActivity, just update UI
                    appendLog("✅ Configuration saved successfully")
                    appendLog("💡 You can now start the service manually")
                    
                    // Update UI to show configuration is ready
                    updateStatus("Configuration ready", false)
                    updateStartButton("Start Mint Service", false, true)
                }
            }
            REQUEST_ACCOUNT -> {
                if (resultCode == RESULT_OK && data != null) {
                    val logout = data.getBooleanExtra("logout", false)
                    if (logout) {
                        // User logged out, restart activity to show login screen
                        recreate()
                    }
                }
            }
            REQUEST_LANGUAGE -> {
                if (resultCode == RESULT_OK) {
                    // Language was changed, update UI texts
                    recreate()
                }
            }
        }
    }
    
    override fun onResume() {
        super.onResume()
        // Update UI text when returning from language settings
        updateUITexts()
    }
    
    private fun updateUITexts() {
        // Update all text elements with current language
        toolbar.title = getString(R.string.dashboard)
        startButton.text = if (isMintRunning) getString(R.string.stop_service) else getString(R.string.start_service)
        clearLogsButton.text = getString(R.string.clear_logs)
        statusTextView.text = getString(R.string.mint_status)
    }

    private fun initializeViews() {
        drawerLayout = findViewById(R.id.drawerLayout)
        navigationView = findViewById(R.id.navigationView)
        toolbar = findViewById(R.id.topAppBar)
        btnConfig = findViewById(R.id.btnConfig)
        statusIcon = findViewById(R.id.statusIcon)
        statusChip = findViewById(R.id.statusChip)
        statusTextView = findViewById(R.id.statusTextView)
        startButton = findViewById(R.id.startButton)
        clearLogsButton = findViewById(R.id.clearLogsButton)
        logsText = findViewById(R.id.logsText)
        
        // Initialize mode selection components
        modeChipGroup = findViewById(R.id.modeChipGroup)
        localModeChip = findViewById(R.id.localModeChip)
        torModeChip = findViewById(R.id.torModeChip)
        addressLayout = findViewById(R.id.addressLayout)
        addressTitle = findViewById(R.id.addressTitle)
        addressText = findViewById(R.id.addressText)
        copyAddressButton = findViewById(R.id.copyAddressButton)
        qrCodeButton = findViewById(R.id.qrCodeButton)
        
        // Setup toolbar and navigation drawer
        setSupportActionBar(toolbar)
        supportActionBar?.setDisplayHomeAsUpEnabled(true)
        supportActionBar?.setHomeAsUpIndicator(R.drawable.ic_menu)
        navigationView.setNavigationItemSelectedListener(this)
        
        // Set dynamic version in navigation header
        setupNavigationHeader()
        
        // Setup config button
        btnConfig.setOnClickListener {
            if (!isMintRunning && !isServiceStarting) {
                val intent = Intent(this, ConfigActivity::class.java)
                startActivityForResult(intent, REQUEST_CONFIG)
            } else {
                val actionName = if (isServiceStarting) getString(R.string.config_settings_starting) else getString(R.string.config_settings)
                showStopServiceDialog(actionName)
            }
        }
        
        startButton.setOnClickListener {
            if (isMintRunning) {
                stopMintService()
            } else {
                // Check if we have configuration
                if (configManager.hasConfiguration()) {
                    startMintServiceWithCurrentMode()
                } else {
                    // No configuration - generate default config and launch config activity
                    appendLog("📝 Generating default configuration...")
                    val success = configManager.generateAndSaveDefaultConfig()
                    if (success) {
                        appendLog("✅ Default configuration generated")
                        appendLog("📝 Opening configuration page for customization...")
                        val intent = Intent(this, ConfigActivity::class.java)
                        startActivityForResult(intent, REQUEST_CONFIG)
                    } else {
                        appendLog("❌ Failed to generate default configuration")
                        Toast.makeText(this, getString(R.string.failed_to_generate_config), Toast.LENGTH_SHORT).show()
                    }
                }
            }
        }
        
        // Setup mode selection listeners
        modeChipGroup.setOnCheckedChangeListener { group, checkedId ->
            when (checkedId) {
                R.id.localModeChip -> {
                    if (currentMode != PurrmintManager.ServiceMode.LOCAL) {
                        if (!isMintRunning && !isServiceStarting) {
                            currentMode = PurrmintManager.ServiceMode.LOCAL
                            
                            // Show local address with a small delay to ensure UI is ready
                            Handler(Looper.getMainLooper()).postDelayed({
                                showLocalAddress()
                            }, 100)
                            
                            appendLog("🌐 Selected Local Mode")
                        } else {
                            val actionName = if (isServiceStarting) getString(R.string.local_mode_starting) else getString(R.string.local_mode)
                            showStopServiceDialog(actionName)
                            // Reset selection to current mode
                            when (currentMode) {
                                PurrmintManager.ServiceMode.LOCAL -> localModeChip.isChecked = true
                                PurrmintManager.ServiceMode.TOR -> torModeChip.isChecked = true
                            }
                        }
                    }
                }
                R.id.torModeChip -> {
                    if (currentMode != PurrmintManager.ServiceMode.TOR) {
                        if (!isMintRunning && !isServiceStarting) {
                            currentMode = PurrmintManager.ServiceMode.TOR
                            showOnionAddress()
                            appendLog("🧅 Selected Tor Mode")
                        } else {
                            val actionName = if (isServiceStarting) getString(R.string.tor_mode_starting) else getString(R.string.tor_mode)
                            showStopServiceDialog(actionName)
                            // Reset selection to current mode
                            when (currentMode) {
                                PurrmintManager.ServiceMode.LOCAL -> localModeChip.isChecked = true
                                PurrmintManager.ServiceMode.TOR -> torModeChip.isChecked = true
                            }
                        }
                    }
                }
            }
        }
        
        // Setup copy address button
        copyAddressButton.setOnClickListener {
            copyAddressToClipboard()
        }
        
        // Setup QR code button
        qrCodeButton.setOnClickListener {
            showQRCodeDialog()
        }
        
        clearLogsButton.setOnClickListener {
            clearLogs()
        }
    }

    private fun setupClickListeners() {
        // Click listeners are handled in initializeViews
    }
    
    private fun setupWindowInsets() {
        // Apply window insets to handle edge-to-edge display properly
        ViewCompat.setOnApplyWindowInsetsListener(findViewById(R.id.drawerLayout)) { view, insets ->
            val systemBars = insets.getInsets(WindowInsetsCompat.Type.systemBars())
            
            // Apply padding to the main content, but let the AppBar extend under the status bar
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
    
    private fun showLoggedInState() {
        isLoggedIn = true
        
        // Show account info
        val npubAddress = loginManager.getNpubAddress()
        val nsecKey = loginManager.getNsecKey()
        val accountInfo = loginManager.getAccountInfo()
        
        appendLog("🔐 Login state debug:")
        appendLog("  - NPUB: ${npubAddress ?: "Not found"}")
        appendLog("  - NSEC: ${if (nsecKey != null) "[Present]" else "Not found"}")
        appendLog("  - Account Info: ${accountInfo ?: "Not found"}")
        
        if (npubAddress != null) {
            updateAccountInfo("Account: $npubAddress")
        } else if (accountInfo != null) {
                updateAccountInfo("Account: $accountInfo")
        }
        
        // Enable start button
        enableStartButton()
        
        updateStatus("Logged in", true)
        appendLog("Welcome back! You are logged in.")
    }
    
    fun logout() {
        // Clear login state
        loginManager.clearLoginState()
        
        // Show confirmation
        Toast.makeText(this, getString(R.string.logged_out_successfully), Toast.LENGTH_SHORT).show()
        
        // Restart activity to show login screen
        recreate()
    }

    private val serviceConnection = object : ServiceConnection {
        override fun onServiceConnected(name: ComponentName?, service: IBinder?) {
            try {
                if (service is PurrmintService.LocalBinder) {
                    purrmintService = service
                    isServiceBound = true
                    Log.i(TAG, "PurrmintService connected (same process)")
                    appendLog("✅ Service connected successfully")
                    
                    // Enable start button after service is bound
                    runOnUiThread {
                        enableStartButton()
                    }
                } else {
                    isServiceBound = true
                    Log.i(TAG, "PurrmintService connected (different process)")
                    appendLog("✅ Service connected (different process)")
                    
                    // Enable start button after service is bound
                    runOnUiThread {
                        enableStartButton()
                    }
                }
            } catch (e: Exception) {
                Log.e(TAG, "Error connecting to service", e)
                isServiceBound = false
                appendLog("❌ Service connection failed: ${e.message}")
            }
        }

        override fun onServiceDisconnected(name: ComponentName?) {
            purrmintService = null
            isServiceBound = false
            Log.i(TAG, "PurrmintService disconnected")
            appendLog("⚠️ Service disconnected")
            
            // Disable start button when service is disconnected
            runOnUiThread {
                disableStartButton()
            }
        }
    }

    private fun bindPurrmintService() {
        try {
        val intent = Intent(this, PurrmintService::class.java)
            val bound = bindService(intent, serviceConnection, Context.BIND_AUTO_CREATE)
            if (bound) {
                appendLog("🔗 Attempting to bind to service...")
            } else {
                appendLog("❌ Failed to bind to service")
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error binding to service", e)
            appendLog("❌ Error binding to service: ${e.message}")
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        if (isServiceBound) {
            unbindService(serviceConnection)
            isServiceBound = false
        }
    }
    
    private fun requestBatteryOptimizationExemption() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
            val powerManager = getSystemService(Context.POWER_SERVICE) as PowerManager
            val packageName = packageName
            
            if (!powerManager.isIgnoringBatteryOptimizations(packageName)) {
                try {
                    val intent = Intent(Settings.ACTION_REQUEST_IGNORE_BATTERY_OPTIMIZATIONS).apply {
                        data = Uri.parse("package:$packageName")
                    }
                    startActivity(intent)
                } catch (e: Exception) {
                    Log.w(TAG, "Could not request battery optimization exemption", e)
                }
            }
        }
    }

    fun startMintService() {
        // Check if we have saved configuration
        if (configManager.hasConfiguration()) {
            // Use saved configuration
            val config = configManager.getConfiguration()
            appendLog("Using saved configuration:")
            appendLog("  Port: ${config.port}")
            appendLog("  Mint Name: ${config.mintName}")
            appendLog("  Description: ${config.description}")
            appendLog("  Lightning Backend: ${config.lightningBackend}")
            
            if (config.lightningBackend == "lnbits") {
                appendLog("  LNBits Admin API Key: ${config.lnbitsAdminApiKey?.take(8)}...")
                appendLog("  LNBits Invoice API Key: ${config.lnbitsInvoiceApiKey?.take(8)}...")
                appendLog("  LNBits API URL: ${config.lnbitsApiUrl}")
            }
            
            startServiceWithConfig(
                config.port, config.mintName, config.description, config.lightningBackend,
                config.lnbitsAdminApiKey, config.lnbitsInvoiceApiKey, config.lnbitsApiUrl
            )
        } else {
            // No configuration exists - this should not happen as we generate config first
            appendLog("❌ No configuration found - please create a new mint first")
            Toast.makeText(this, getString(R.string.please_create_new_mint), Toast.LENGTH_SHORT).show()
        }
    }
    
    private fun startServiceWithConfig(port: Int, mintName: String, description: String, lightningBackend: String, lnbitsAdminApiKey: String? = null, lnbitsInvoiceApiKey: String? = null, lnbitsApiUrl: String? = null, nwcConnectionUri: String? = null) {
        try {
            updateStatus("Starting mint service...", false)
            appendLog("Starting mint service with configuration...")

            // Check if service is bound, if not, try to bind and retry
            if (!isServiceBound || purrmintService == null) {
                appendLog("⚠️ Service not bound, attempting to bind...")
                bindPurrmintService()
                
                // Wait longer for binding to complete (5 seconds)
                Handler(Looper.getMainLooper()).postDelayed({
                    if (isServiceBound && purrmintService != null) {
                        appendLog("✅ Service bound successfully, retrying start...")
                        startServiceWithConfig(port, mintName, description, lightningBackend, lnbitsAdminApiKey, lnbitsInvoiceApiKey, lnbitsApiUrl, nwcConnectionUri)
                    } else {
                        appendLog("❌ Service binding failed after 5 seconds")
                        appendLog("💡 Trying to restart service binding...")
                        
                        // Try one more time with a fresh bind
                        bindPurrmintService()
                        Handler(Looper.getMainLooper()).postDelayed({
            if (isServiceBound && purrmintService != null) {
                                appendLog("✅ Service bound successfully on retry, starting...")
                                startServiceWithConfig(port, mintName, description, lightningBackend, lnbitsAdminApiKey, lnbitsInvoiceApiKey, lnbitsApiUrl, nwcConnectionUri)
                            } else {
                                appendLog("❌ Service binding failed completely")
                                updateStatus("Service binding failed", false)
                                Toast.makeText(this, getString(R.string.service_binding_failed), Toast.LENGTH_LONG).show()
                            }
                        }, 3000) // Wait 3 more seconds
                    }
                }, 5000) // Wait 5 seconds
                return
            }

                val purrmintManager = purrmintService!!.getPurrmintManager()
            
            // Stop existing service first to ensure new configuration takes effect
            appendLog("🔄 Stopping existing service to apply new configuration...")
            val stopSuccess = purrmintManager.stopMintService()
            if (stopSuccess) {
                appendLog("✅ Existing service stopped successfully")
            } else {
                appendLog("⚠️ Failed to stop existing service, but continuing...")
            }
            
            // Wait a moment for service to fully stop
            Thread.sleep(1000)
                
                // Get current account's nsec for service
                val nsec = loginManager.getNsecKey()
            
            // Validate that we have an nsec
            if (nsec == null || nsec.isEmpty()) {
                appendLog("❌ No nsec key found - please login first")
                updateStatus("No nsec key found", false)
                return
            }
                
                // Auto-generate config if not exists
                if (!purrmintManager.configExists()) {
                    appendLog("Generating default config...")
                    val success = configManager.generateAndSaveDefaultConfig()
                    if (!success) {
                        appendLog("❌ Failed to generate default config")
                        updateStatus("Failed to generate config", false)
                        return
                    }
                }
                
                // Auto-generate Nostr account if not exists
                if (!purrmintManager.accountExists()) {
                    appendLog("Creating Nostr account...")
                    val account = purrmintManager.createNostrAccount()
                    if (account == null) {
                        appendLog("❌ Failed to create Nostr account")
                        updateStatus("Failed to create account", false)
                        return
                    }
                }

            // Create configuration JSON with lightning backend settings
            val configJson = buildString {
                append("""
                    {
                        "port": $port,
                        "host": "127.0.0.1",
                        "mintName": "$mintName",
                        "description": "$description",
                        "lightningBackend": "$lightningBackend",
                        "mode": "mintd_only",
                        "databasePath": "${filesDir.absolutePath}/database",
                        "logsPath": "${filesDir.absolutePath}/logs"
                """.trimIndent())
                
                if (lightningBackend == "lnbits" && lnbitsAdminApiKey != null && lnbitsInvoiceApiKey != null && lnbitsApiUrl != null) {
                    append(""",
                        "lnbitsAdminApiKey": "$lnbitsAdminApiKey",
                        "lnbitsInvoiceApiKey": "$lnbitsInvoiceApiKey",
                        "lnbitsApiUrl": "$lnbitsApiUrl"
                    """.trimIndent())
                } else if (lightningBackend == "nwc" && nwcConnectionUri != null) {
                    append(""",
                        "nwcConnectionUri": "$nwcConnectionUri"
                    """.trimIndent())
                }
                
                append("}")
            }
            
            // Validate JSON format before sending
            try {
                org.json.JSONObject(configJson)
                appendLog("✅ JSON configuration validated successfully")
            } catch (e: Exception) {
                appendLog("❌ Invalid JSON configuration: ${e.message}")
                appendLog("💡 Configuration file appears to be corrupted")
                appendLog("🧹 Clearing corrupted configuration...")
                
                // Clear the corrupted configuration
                val cleared = configManager.clearConfiguration()
                if (cleared) {
                    appendLog("✅ Corrupted configuration cleared")
                    appendLog("📝 Please reconfigure your mint settings")
                    updateStatus("Configuration corrupted", false)
                    updateStartButton("Create New Mint", false)
                    
                    // Show toast to user
                    Toast.makeText(this, "Configuration corrupted. Please reconfigure.", Toast.LENGTH_LONG).show()
                } else {
                    appendLog("❌ Failed to clear corrupted configuration")
                    updateStatus("Failed to clear config", false)
                }
                return
            }
            
            appendLog("Using configuration: $configJson")

            val success = purrmintManager.startMintServiceWithConfig(nsec, configJson)
                if (success) {
                    updateStatus("Service is running", true)
                    appendLog("✅ Mint service started successfully!")
                appendLog("✅ Service available at http://127.0.0.1:$port")
                    updateStartButton("Stop Service", true, true)
                } else {
                    updateStatus("Failed to start service", false)
                    appendLog("❌ Failed to start mint service")
            }
        } catch (e: Exception) {
            updateStatus("Error: ${e.message}", false)
            appendLog("❌ Error starting service: ${e.message}")
            Log.e(TAG, "Error starting service", e)
        }
    }
    
    private fun stopMintService() {
        try {
            updateStatus("Stopping mint service...", false)
            updateStartButton("Stopping...", false, false)
            appendLog("Stopping mint service...")
            
            if (isServiceBound && purrmintService != null) {
                val purrmintManager = purrmintService!!.getPurrmintManager()
                val success = purrmintManager.stopMintService()
                if (success) {
                    updateStatus("Service stopped", false)
                    appendLog("✅ Mint service stopped successfully!")
                    hideAddress()
                    updateStartButton("Start Mint Service", false, true)
                } else {
                    updateStatus("Failed to stop service", true)
                    appendLog("❌ Failed to stop mint service")
                    updateStartButton("Stop Service", true, true)
                }
            } else {
                updateStatus("Service stopped", false)
                appendLog("✅ Service stopped!")
                hideAddress()
                updateStartButton("Start Mint Service", false, true)
            }
        } catch (e: Exception) {
            updateStatus("Error: ${e.message}", true)
            appendLog("❌ Error stopping service: ${e.message}")
            Log.e(TAG, "Error stopping service", e)
            updateStartButton("Stop Service", true, true)
        }
    }

    private fun updateStatus(status: String, isOnline: Boolean) {
        isMintRunning = isOnline
        // Keep buttons enabled but handle clicks differently when service is running
        btnConfig.isEnabled = true
        localModeChip.isEnabled = true
        torModeChip.isEnabled = true
        
        statusTextView.text = status
        
        if (isOnline) {
            statusIcon.setImageResource(R.drawable.ic_status_online)
            statusIcon.setColorFilter(resources.getColor(R.color.success_color, null))
            statusChip.text = getString(R.string.online)
            statusChip.setChipBackgroundColorResource(R.color.success_container_color)
            statusChip.setTextColor(resources.getColor(R.color.success_color, null))
        } else {
            statusIcon.setImageResource(R.drawable.ic_status_offline)
            statusIcon.setColorFilter(resources.getColor(R.color.error_color, null))
            statusChip.text = getString(R.string.offline)
            statusChip.setChipBackgroundColorResource(R.color.error_container_color)
            statusChip.setTextColor(resources.getColor(R.color.error_color, null))
        }
    }
    
    private fun updateAccountInfo(info: String) {
        // You can add a TextView for account info if needed
        appendLog(info)
    }

    private fun enableStartButton() {
        startButton.isEnabled = true
        startButton.text = getString(R.string.start_mint_service)
        startButton.setIconResource(R.drawable.ic_play)
    }

    private fun disableStartButton() {
        startButton.isEnabled = false
        startButton.text = getString(R.string.service_unavailable)
        startButton.setIconResource(R.drawable.ic_status_offline)
    }

    private fun updateStartButton(text: String, isRunning: Boolean) {
        startButton.text = text
        if (isRunning) {
            startButton.isEnabled = true
            startButton.setIconResource(R.drawable.ic_stop)
        } else {
            startButton.isEnabled = true
            startButton.setIconResource(R.drawable.ic_play)
        }
    }
    
    private fun updateStartButton(text: String, isRunning: Boolean, isEnabled: Boolean) {
        startButton.text = text
        startButton.isEnabled = isEnabled
        if (isRunning) {
            startButton.setIconResource(R.drawable.ic_stop)
        } else {
            startButton.setIconResource(R.drawable.ic_play)
        }
    }

    private fun appendLog(message: String) {
        Log.i(TAG, message)
        
        // Update UI on main thread
        runOnUiThread {
            val timestamp = java.text.SimpleDateFormat("HH:mm:ss", java.util.Locale.getDefault()).format(java.util.Date())
            val logEntry = "[$timestamp] $message\n"
            logsText.append(logEntry)
            
            // Auto-scroll to bottom
            val scrollView = findViewById<android.widget.ScrollView>(R.id.logsScrollView)
            scrollView.post {
                scrollView.fullScroll(android.view.View.FOCUS_DOWN)
            }
        }
    }

    fun clearLogs() {
        runOnUiThread {
            logsText.text = ""
        }
        appendLog("Logs cleared")
    }

    private fun checkInitialState() {
        // Check if configuration exists
        if (!configManager.hasConfiguration()) {
            // No configuration exists - show "Create New Mint" state
            updateStatus("No configuration", false)
            updateStartButton("Create New Mint", false)
            appendLog("📝 No configuration found - please create a new mint")
        } else {
            // Configuration exists - check if service is running
            updateStatus("Configuration ready", false)
            updateStartButton("Start Mint Service", false)
            appendLog("✅ Configuration found - ready to start mint service")
            
            // Check if service is actually running
            if (isServiceBound && purrmintService != null) {
                val purrmintManager = purrmintService!!.getPurrmintManager()
                val status = purrmintManager.getServiceStatus()
                try {
                    val statusJson = org.json.JSONObject(status.toString())
                    val isRunning = statusJson.optString("status") == "running"
                    if (isRunning) {
                        updateStatus("Service is running", true)
                        updateStartButton("Stop Service", true, true)
                        appendLog("✅ Mint service is already running")
                    }
                } catch (e: Exception) {
                    appendLog("⚠️ Could not check service status: ${e.message}")
                }
            }
        }
    }
    
    // Menu handling
    override fun onCreateOptionsMenu(menu: Menu): Boolean {
        menuInflater.inflate(R.menu.main_menu, menu)
        return true
    }
    
    override fun onOptionsItemSelected(item: MenuItem): Boolean {
        return when (item.itemId) {
            R.id.action_user -> {
                val intent = Intent(this, AccountActivity::class.java)
                startActivityForResult(intent, REQUEST_ACCOUNT)
                true
            }
            else -> super.onOptionsItemSelected(item)
        }
    }
    
    override fun onSupportNavigateUp(): Boolean {
        // Handle navigation icon click (hamburger menu)
        drawerLayout.openDrawer(navigationView)
        return true
    }
    
    // Navigation drawer item selection
    override fun onNavigationItemSelected(item: MenuItem): Boolean {
        when (item.itemId) {
            R.id.nav_language -> {
                val intent = Intent(this, LanguageSettingsActivity::class.java)
                startActivityForResult(intent, REQUEST_LANGUAGE)
            }
            R.id.nav_version -> {
                showVersionInfo()
            }
        }
        
        // Close the drawer
        drawerLayout.closeDrawers()
        return true
    }
    
    private fun setupNavigationHeader() {
        val headerView = navigationView.getHeaderView(0)
        val versionTextView = headerView.findViewById<TextView>(R.id.navVersionText)
        
        try {
            val versionName = packageManager.getPackageInfo(packageName, 0).versionName
            versionTextView.text = "v$versionName"
        } catch (e: Exception) {
            versionTextView.text = "v0.0.3" // fallback
        }
    }
    
    private fun showVersionInfo() {
        val versionName = packageManager.getPackageInfo(packageName, 0).versionName
        val versionCode = packageManager.getPackageInfo(packageName, 0).versionCode
        val versionText = "Version $versionName ($versionCode)"
        
        Toast.makeText(this, versionText, Toast.LENGTH_LONG).show()
    }
    
    // =============================================================================
    // Mode switching methods
    // =============================================================================
    
    private fun startMintServiceWithCurrentMode() {
        try {
            val nsec = loginManager.getNsecKey()
            if (nsec == null) {
                appendLog("❌ No NSEC key found - please login again")
                Toast.makeText(this, "No NSEC key found", Toast.LENGTH_SHORT).show()
                return
            }
            
            appendLog("🚀 Starting mint service in ${currentMode.name.lowercase()} mode...")
            
            if (isServiceBound && purrmintService != null) {
                val purrmintManager = purrmintService!!.getPurrmintManager()
                
                // Always start service in background thread to prevent ANR and ensure thread safety
                appendLog("🔄 Starting mint service in background thread...")
                
                // Show loading state and set starting flag
                isServiceStarting = true
                updateStatus("Starting service...", false)
                updateStartButton("Starting...", false, false)
                
                // Start service in background thread
                Thread {
                    try {
                        // Stop existing service first to ensure clean state
                        appendLog("🧹 Ensuring clean service state...")
                        purrmintManager.stopMintService()
                        
                        // Wait for service to fully stop
                        Thread.sleep(1000)
                        
                        val success = when (currentMode) {
                            PurrmintManager.ServiceMode.TOR -> {
                                appendLog("🧅 Starting Tor mint service...")
                                purrmintManager.startTorMint(nsec)
                            }
                            PurrmintManager.ServiceMode.LOCAL -> {
                                appendLog("🌐 Starting local mint service...")
                                purrmintManager.startLocalMint(nsec)
                            }
                        }
                        
                        // Update UI on main thread
                        runOnUiThread {
                            isServiceStarting = false
                            if (success) {
                                val modeText = when (currentMode) {
                                    PurrmintManager.ServiceMode.TOR -> "Service is running (tor)"
                                    PurrmintManager.ServiceMode.LOCAL -> "Service is running (local)"
                                }
                                updateStatus(modeText, true)
                                appendLog("✅ Mint service started successfully!")
                                
                                if (currentMode == PurrmintManager.ServiceMode.TOR) {
                                    appendLog("🧅 Tor service starting - onion address will appear shortly...")
                                    showOnionAddress()
                                } else {
                                    // Show local address with a small delay to ensure UI is ready
                                    Handler(Looper.getMainLooper()).postDelayed({
                                        showLocalAddress()
                                    }, 100)
                                }
                                
                                updateStartButton("Stop Service", true, true)
                            } else {
                                updateStatus("Failed to start service", false)
                                appendLog("❌ Failed to start mint service")
                                updateStartButton("Start Service", false, true)
                            }
                        }
                    } catch (e: Exception) {
                        runOnUiThread {
                            isServiceStarting = false
                            updateStatus("Error: ${e.message}", false)
                            appendLog("❌ Error starting service: ${e.message}")
                            Log.e(TAG, "Error starting service", e)
                            updateStartButton("Start Service", false, true)
                        }
                    }
                }.start()
            } else {
                appendLog("❌ Service not bound - cannot start mint service")
                Toast.makeText(this, "Service not available", Toast.LENGTH_SHORT).show()
            }
        } catch (e: Exception) {
            updateStatus("Error: ${e.message}", false)
            appendLog("❌ Error starting service: ${e.message}")
            Log.e(TAG, "Error starting service", e)
        }
    }
    
    private fun restartServiceInNewMode() {
        // This function is called when a mode change is requested while the service is running.
        // It stops the current service, waits, and then starts it in the new mode.
        // This ensures a clean transition and prevents potential issues.
        
        appendLog("🔄 Restarting service in ${currentMode.name.lowercase()} mode...")
        
        // Stop the current service
        if (isServiceBound && purrmintService != null) {
            val purrmintManager = purrmintService!!.getPurrmintManager()
            purrmintManager.stopMintService()
            appendLog("⏹️ Service stopped")
        }
        
        // Wait longer for Tor service to fully stop, then restart
        Handler(Looper.getMainLooper()).postDelayed({
            appendLog("🚀 Starting service in ${currentMode.name.lowercase()} mode...")
            startMintServiceWithCurrentMode()
        }, 5000) // Wait 5 seconds for Tor service to fully stop
    }
    
    private fun showOnionAddress() {
        addressLayout.visibility = android.view.View.VISIBLE
        addressTitle.text = getString(R.string.onion_address)
        addressText.text = getString(R.string.onion_address_loading)
        startOnionAddressPolling()
    }
    
    private fun showLocalAddress() {
        try {
            // Ensure we're on the main thread
            runOnUiThread {
                try {
                    addressLayout.visibility = android.view.View.VISIBLE
                    addressTitle.text = getString(R.string.local_address)
                    
                    // Get port from config (now always returns a value)
                    val port = configManager.getPort() ?: 3338
                    localAddress = "http://127.0.0.1:$port"
                    addressText.text = localAddress
                    
                    appendLog("🌐 Local address: $localAddress")
                    appendLog("📱 Address layout visibility set to VISIBLE")
                    appendLog("🔧 Port used: $port")
                    
                    // Force a layout update
                    addressLayout.requestLayout()
                    addressLayout.invalidate()
                    
                } catch (e: Exception) {
                    appendLog("❌ Error showing local address: ${e.message}")
                    Log.e(TAG, "Error showing local address", e)
                }
            }
        } catch (e: Exception) {
            appendLog("❌ Error in showLocalAddress: ${e.message}")
            Log.e(TAG, "Error in showLocalAddress", e)
        }
    }
    
    private fun hideAddress() {
        addressLayout.visibility = android.view.View.GONE
        onionAddress = null
        localAddress = null
    }
    
    private fun startOnionAddressPolling() {
        // Poll for onion address every 5 seconds for up to 5 minutes (60 attempts)
        var attempts = 0
        val maxAttempts = 60  // 5 minutes total
        
        val handler = Handler(Looper.getMainLooper())
        val runnable = object : Runnable {
            override fun run() {
                attempts++
                
                if (attempts > maxAttempts) {
                    appendLog("⚠️ Onion address not available after ${maxAttempts * 5} seconds")
                    appendLog("💡 Tor service may still be starting up. Please wait a few more minutes.")
                    addressText.text = getString(R.string.onion_address_not_available)
                    return
                }
                
                // Show progress every 30 seconds
                if (attempts % 6 == 0) {
                    val elapsedSeconds = attempts * 5
                    appendLog("⏳ Still waiting for onion address... (${elapsedSeconds}s elapsed)")
                }
                
                // Try to get onion address from service status
                if (isServiceBound && purrmintService != null) {
                    val purrmintManager = purrmintService!!.getPurrmintManager()
                    
                    // First try the dedicated getOnionAddress method
                    val onionAddr = purrmintManager.getOnionAddress()
                    if (!onionAddr.isNullOrEmpty()) {
                        onionAddress = onionAddr
                        addressText.text = onionAddr
                        appendLog("🧅 Onion address: $onionAddr")
                        return
                    }
                    
                    // Fallback to service status
                    val status = purrmintManager.getServiceStatus()
                    
                    try {
                        val statusJson = org.json.JSONObject(status.toString())
                        val onionAddrFromStatus = statusJson.optString("onion_address", "")
                        
                        if (onionAddrFromStatus.isNotEmpty()) {
                            onionAddress = onionAddrFromStatus
                            addressText.text = onionAddrFromStatus
                            appendLog("🧅 Onion address: $onionAddrFromStatus")
                            return
                        }
                    } catch (e: Exception) {
                        // Ignore JSON parsing errors
                    }
                }
                
                // Continue polling
                handler.postDelayed(this, 5000) // Poll every 5 seconds
            }
        }
        
        handler.post(runnable)
    }
    
    private fun copyAddressToClipboard() {
        val address = when (currentMode) {
            PurrmintManager.ServiceMode.LOCAL -> localAddress ?: addressText.text.toString()
            PurrmintManager.ServiceMode.TOR -> onionAddress ?: addressText.text.toString()
        }
        
        if (address.isNotEmpty() && address != getString(R.string.onion_address_loading) && 
            address != getString(R.string.onion_address_not_available)) {
            
            val clipboard = getSystemService(Context.CLIPBOARD_SERVICE) as android.content.ClipboardManager
            val clipLabel = when (currentMode) {
                PurrmintManager.ServiceMode.LOCAL -> "Local Address"
                PurrmintManager.ServiceMode.TOR -> "Onion Address"
            }
            val clip = android.content.ClipData.newPlainText(clipLabel, address)
            clipboard.setPrimaryClip(clip)
            
            val toastMessage = when (currentMode) {
                PurrmintManager.ServiceMode.LOCAL -> getString(R.string.local_address_copied)
                PurrmintManager.ServiceMode.TOR -> getString(R.string.onion_address_copied)
            }
            Toast.makeText(this, toastMessage, Toast.LENGTH_SHORT).show()
            
            val logMessage = when (currentMode) {
                PurrmintManager.ServiceMode.LOCAL -> "📋 Local address copied to clipboard"
                PurrmintManager.ServiceMode.TOR -> "📋 Onion address copied to clipboard"
            }
            appendLog(logMessage)
        } else {
            val errorMessage = when (currentMode) {
                PurrmintManager.ServiceMode.LOCAL -> "No local address available"
                PurrmintManager.ServiceMode.TOR -> "No onion address available"
            }
            Toast.makeText(this, errorMessage, Toast.LENGTH_SHORT).show()
        }
    }
    
    /**
     * Show QR code dialog for the current address
     */
    private fun showQRCodeDialog() {
        val address = when (currentMode) {
            PurrmintManager.ServiceMode.LOCAL -> localAddress ?: addressText.text.toString()
            PurrmintManager.ServiceMode.TOR -> onionAddress ?: addressText.text.toString()
        }
        
        if (address.isNotEmpty() && address != getString(R.string.onion_address_loading) && 
            address != getString(R.string.onion_address_not_available)) {
            
            try {
                val dialog = when (currentMode) {
                    PurrmintManager.ServiceMode.LOCAL -> {
                        QRCodeDialog.newLocalAddressInstance(address)
                    }
                    PurrmintManager.ServiceMode.TOR -> {
                        QRCodeDialog.newOnionAddressInstance(address)
                    }
                }
                
                dialog.show(supportFragmentManager, "QRCodeDialog")
                appendLog("📱 Showing QR code for ${currentMode.name.lowercase()} address")
                
            } catch (e: Exception) {
                Log.e(TAG, "Error showing QR code dialog", e)
                appendLog("❌ Error showing QR code: ${e.message}")
                Toast.makeText(this, "Error showing QR code", Toast.LENGTH_SHORT).show()
            }
        } else {
            val errorMessage = when (currentMode) {
                PurrmintManager.ServiceMode.LOCAL -> "No local address available"
                PurrmintManager.ServiceMode.TOR -> "No onion address available"
            }
            Toast.makeText(this, errorMessage, Toast.LENGTH_SHORT).show()
        }
    }
    
    /**
     * Show dialog asking user if they want to stop the service
     * @param actionName The name of the action that requires stopping the service
     */
    private fun showStopServiceDialog(actionName: String) {
        val dialog = androidx.appcompat.app.AlertDialog.Builder(this)
            .setTitle(getString(R.string.service_running_title))
            .setMessage(getString(R.string.service_running_message, actionName))
            .setPositiveButton(getString(R.string.stop_service_button)) { _, _ ->
                stopMintService()
            }
            .setNegativeButton(getString(R.string.cancel), null)
            .create()
        
        dialog.show()
    }
} 