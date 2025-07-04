package com.purrmint.app

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

class MainActivity : AppCompatActivity() {
    
    // UI Components
    private lateinit var btnAccount: ImageButton
    private lateinit var btnConfig: ImageButton
    private lateinit var statusIcon: ImageView
    private lateinit var statusChip: Chip
    private lateinit var statusTextView: TextView
    private lateinit var startButton: MaterialButton
    private lateinit var clearLogsButton: MaterialButton
    
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
        
        appendLog("✅ Login successful! Mint service is starting...")
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
                    val port = data.getStringExtra(ConfigActivity.EXTRA_PORT) ?: "3338"
                    val host = data.getStringExtra(ConfigActivity.EXTRA_HOST) ?: "0.0.0.0"
                    val mintName = data.getStringExtra(ConfigActivity.EXTRA_MINT_NAME) ?: "My Mint"
                    val description = data.getStringExtra(ConfigActivity.EXTRA_DESCRIPTION) ?: "A simple mint service"
                    val lightningBackend = data.getStringExtra(ConfigActivity.EXTRA_LIGHTNING_BACKEND) ?: "fakewallet"
                    
                    // Save configuration for future use
                    configManager.saveConfiguration(port, host, mintName, description, lightningBackend)
                    
                    appendLog("Configuration received:")
                    appendLog("  Port: $port")
                    appendLog("  Host: $host")
                    appendLog("  Mint Name: $mintName")
                    appendLog("  Description: $description")
                    appendLog("  Lightning Backend: $lightningBackend")
                    
                    // Start the service with configuration
                    startServiceWithConfig(port, host, mintName, description, lightningBackend)
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
                startMintService()
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
        if (npubAddress != null) {
            updateAccountInfo("Account: $npubAddress")
        } else {
            val accountInfo = loginManager.getAccountInfo()
            if (accountInfo != null) {
                updateAccountInfo("Account: $accountInfo")
            }
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
                } else {
                    isServiceBound = true
                    Log.i(TAG, "PurrmintService connected (different process)")
                    appendLog("✅ Service connected (different process)")
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
            appendLog("  Host: ${config.host}")
            appendLog("  Mint Name: ${config.mintName}")
            appendLog("  Description: ${config.description}")
            appendLog("  Lightning Backend: ${config.lightningBackend}")
            
            startServiceWithConfig(config.port, config.host, config.mintName, config.description, config.lightningBackend)
        } else {
            // First time, launch config activity
            appendLog("First time setup - launching configuration...")
            val intent = Intent(this, ConfigActivity::class.java)
            startActivityForResult(intent, REQUEST_CONFIG)
        }
    }
    
    private fun startServiceWithConfig(port: String, host: String, mintName: String, description: String, lightningBackend: String) {
        try {
            updateStatus("Starting mint service...", false)
            appendLog("Starting mint service with configuration...")

            if (isServiceBound && purrmintService != null) {
                val purrmintManager = purrmintService!!.getPurrmintManager()
                
                // Auto-generate config if not exists
                if (!purrmintManager.configExists()) {
                    appendLog("Generating config...")
                    purrmintManager.generateConfig()
                }
                
                // Auto-generate Nostr account if not exists
                if (!purrmintManager.accountExists()) {
                    appendLog("Creating Nostr account...")
                    purrmintManager.createNostrAccount()
                }

                val success = purrmintManager.startMintService()
                if (success) {
                    updateStatus("Service is running", true)
                    appendLog("✅ Mint service started successfully!")
                    appendLog("✅ Service available at http://$host:$port")
                    updateStartButton("Stop Service", true)
                } else {
                    updateStatus("Failed to start service", false)
                    appendLog("❌ Failed to start mint service")
                }
            } else {
                updateStatus("Service running", true)
                appendLog("✅ Service is running!")
                appendLog("✅ Service available at http://$host:$port")
                updateStartButton("Stop Service", true)
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
        // For now, we'll use a simple approach - you can implement a proper log view later
        Log.i(TAG, message)
        // You can add a TextView or ScrollView to display logs in the UI
    }

    fun clearLogs() {
        // Clear logs implementation
        appendLog("Logs cleared")
    }
} 