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
import android.widget.Button
import android.widget.TextView
import android.widget.ImageView
import android.widget.ImageButton
import androidx.appcompat.app.AppCompatActivity
import com.google.android.material.button.MaterialButton
import com.google.android.material.textfield.TextInputEditText
import com.google.android.material.chip.Chip
import android.widget.Toast
import com.purrmint.app.R
import com.purrmint.app.core.managers.LoginManager
import com.purrmint.app.core.managers.ConfigManager
import com.purrmint.app.core.services.PurrmintService
import android.os.Handler
import android.os.Looper

class MainActivity : AppCompatActivity() {
    
    // UI Components
    private lateinit var btnAccount: ImageButton
    private lateinit var btnConfig: ImageButton
    private lateinit var statusIcon: ImageView
    private lateinit var statusChip: Chip
    private lateinit var statusTextView: TextView
    private lateinit var startButton: MaterialButton
    private lateinit var clearLogsButton: MaterialButton
    private lateinit var logsText: TextView
    
    // Service
    private var purrmintService: PurrmintService? = null
    private var isServiceBound = false
    private var isLoggedIn = false
    private var isMintRunning = false
    
    // Login Manager
    private lateinit var loginManager: LoginManager
    
    // Configuration Manager
    private lateinit var configManager: ConfigManager
    
    companion object {
        private const val TAG = "MainActivity"
        private const val REQUEST_CONFIG = 1001
        private const val REQUEST_LOGIN = 1002
        private const val REQUEST_ACCOUNT = 1003
    }
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        // Initialize managers
        loginManager = LoginManager(this)
        configManager = ConfigManager(this)
        
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
                    
                    // Save configuration for future use
                    configManager.saveConfiguration(
                        port = port,
                        mintName = mintName,
                        description = description,
                        lightningBackend = lightningBackend,
                        lnbitsAdminApiKey = lnbitsAdminApiKey,
                        lnbitsInvoiceApiKey = lnbitsInvoiceApiKey,
                        lnbitsApiUrl = lnbitsApiUrl
                    )
                    
                    appendLog("Configuration received:")
                    appendLog("  Port: $port")
                    appendLog("  Mint Name: $mintName")
                    appendLog("  Description: $description")
                    appendLog("  Lightning Backend: $lightningBackend")
                    
                    if (lightningBackend == "lnbits") {
                        appendLog("  LNBits Admin API Key: ${lnbitsAdminApiKey?.take(8)}...")
                        appendLog("  LNBits Invoice API Key: ${lnbitsInvoiceApiKey?.take(8)}...")
                        appendLog("  LNBits API URL: $lnbitsApiUrl")
                    }
                    
                    // Start the service with configuration
                    startServiceWithConfig(port, mintName, description, lightningBackend, lnbitsAdminApiKey, lnbitsInvoiceApiKey, lnbitsApiUrl)
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
        }
    }

    private fun initializeViews() {
        btnAccount = findViewById(R.id.btnAccount)
        btnConfig = findViewById(R.id.btnConfig)
        statusIcon = findViewById(R.id.statusIcon)
        statusChip = findViewById(R.id.statusChip)
        statusTextView = findViewById(R.id.statusTextView)
        startButton = findViewById(R.id.startButton)
        clearLogsButton = findViewById(R.id.clearLogsButton)
        logsText = findViewById(R.id.logsText)
        
        // Restore config button functionality
        btnConfig.setImageResource(R.drawable.ic_settings)
        btnConfig.contentDescription = "Configure Mint"
        
        btnAccount.setOnClickListener {
            val intent = Intent(this, AccountActivity::class.java)
            startActivity(intent)
        }
        btnConfig.setOnClickListener {
            if (!isMintRunning) {
                val intent = Intent(this, ConfigActivity::class.java)
                startActivityForResult(intent, REQUEST_CONFIG)
            } else {
                Toast.makeText(this, "请先停止Mint服务再修改配置", Toast.LENGTH_SHORT).show()
            }
        }
        
        startButton.setOnClickListener {
            if (isMintRunning) {
                stopMintService()
            } else {
                // Check if we have configuration
                if (configManager.hasConfiguration()) {
                    startMintService()
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
                        Toast.makeText(this, "Failed to generate default configuration", Toast.LENGTH_SHORT).show()
                    }
                }
            }
        }
        
        clearLogsButton.setOnClickListener {
            clearLogs()
        }
    }

    private fun setupClickListeners() {
        // Click listeners are handled in initializeViews
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
        Toast.makeText(this, "Logged out successfully", Toast.LENGTH_SHORT).show()
        
        // Restart activity to show login screen
        recreate()
    }

    private val serviceConnection = object : ServiceConnection {
        override fun onServiceConnected(name: ComponentName?, service: IBinder?) {
            try {
                if (service is PurrmintService.LocalBinder) {
                    purrmintService = service.getService()
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
            Toast.makeText(this, "Please create a new mint first", Toast.LENGTH_SHORT).show()
        }
    }
    
    private fun startServiceWithConfig(port: Int, mintName: String, description: String, lightningBackend: String, lnbitsAdminApiKey: String? = null, lnbitsInvoiceApiKey: String? = null, lnbitsApiUrl: String? = null) {
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
                        startServiceWithConfig(port, mintName, description, lightningBackend, lnbitsAdminApiKey, lnbitsInvoiceApiKey, lnbitsApiUrl)
                    } else {
                        appendLog("❌ Service binding failed after 5 seconds")
                        appendLog("💡 Trying to restart service binding...")
                        
                        // Try one more time with a fresh bind
                        bindPurrmintService()
                        Handler(Looper.getMainLooper()).postDelayed({
                            if (isServiceBound && purrmintService != null) {
                                appendLog("✅ Service bound successfully on retry, starting...")
                                startServiceWithConfig(port, mintName, description, lightningBackend, lnbitsAdminApiKey, lnbitsInvoiceApiKey, lnbitsApiUrl)
                            } else {
                                appendLog("❌ Service binding failed completely")
                                updateStatus("Service binding failed", false)
                                Toast.makeText(this, "Service binding failed. Please restart the app.", Toast.LENGTH_LONG).show()
                            }
                        }, 3000) // Wait 3 more seconds
                    }
                }, 5000) // Wait 5 seconds
                return
            }

            val purrmintManager = purrmintService!!.getPurrmintManager()
            
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
                updateStartButton("Stop Service", true)
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
            appendLog("Stopping mint service...")
            
            if (isServiceBound && purrmintService != null) {
                val purrmintManager = purrmintService!!.getPurrmintManager()
                val success = purrmintManager.stopMintService()
                if (success) {
                    updateStatus("Service stopped", false)
                    appendLog("✅ Mint service stopped successfully!")
                    updateStartButton("Start Mint Service", false)
                } else {
                    updateStatus("Failed to stop service", true)
                    appendLog("❌ Failed to stop mint service")
                }
            } else {
                updateStatus("Service stopped", false)
                appendLog("✅ Service stopped!")
                updateStartButton("Start Mint Service", false)
            }
        } catch (e: Exception) {
            updateStatus("Error: ${e.message}", true)
            appendLog("❌ Error stopping service: ${e.message}")
            Log.e(TAG, "Error stopping service", e)
        }
    }

    private fun updateStatus(status: String, isOnline: Boolean) {
        isMintRunning = isOnline
        btnConfig.isEnabled = !isMintRunning
        
        statusTextView.text = status
        
        if (isOnline) {
            statusIcon.setImageResource(R.drawable.ic_status_online)
            statusIcon.setColorFilter(resources.getColor(R.color.success_color, null))
            statusChip.text = "Online"
            statusChip.setChipBackgroundColorResource(R.color.success_container_color)
            statusChip.setTextColor(resources.getColor(R.color.success_color, null))
        } else {
            statusIcon.setImageResource(R.drawable.ic_status_offline)
            statusIcon.setColorFilter(resources.getColor(R.color.error_color, null))
            statusChip.text = "Offline"
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
        startButton.text = "Start Mint Service"
        startButton.setIconResource(R.drawable.ic_play)
    }

    private fun disableStartButton() {
        startButton.isEnabled = false
        startButton.text = "Service Unavailable"
        startButton.setIconResource(R.drawable.ic_status_offline)
    }

    private fun updateStartButton(text: String, isRunning: Boolean) {
        startButton.text = text
        startButton.isEnabled = true
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
                    val statusJson = org.json.JSONObject(status)
                    val isRunning = statusJson.optString("status") == "running"
                    if (isRunning) {
                        updateStatus("Service is running", true)
                        updateStartButton("Stop Service", true)
                        appendLog("✅ Mint service is already running")
                    }
                } catch (e: Exception) {
                    appendLog("⚠️ Could not check service status: ${e.message}")
                }
            }
        }
    }
} 