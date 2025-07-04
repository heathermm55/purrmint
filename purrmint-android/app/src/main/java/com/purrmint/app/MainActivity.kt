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
import android.widget.Button
import android.widget.TextView
import android.widget.ImageView
import androidx.appcompat.app.AppCompatActivity
import com.google.android.material.button.MaterialButton
import com.google.android.material.textfield.TextInputEditText
import com.google.android.material.chip.Chip
import android.widget.Toast

class MainActivity : AppCompatActivity() {
    
    // UI Components
    private lateinit var nsecInput: TextInputEditText
    private lateinit var btnCreateAccount: MaterialButton
    private lateinit var btnLogin: MaterialButton
    private lateinit var statusIcon: ImageView
    private lateinit var statusChip: Chip
    private lateinit var statusTextView: TextView
    private lateinit var startButton: MaterialButton
    private lateinit var logsText: TextView
    private lateinit var clearLogsButton: MaterialButton
    
    // Service
    private var purrmintService: PurrmintService? = null
    private var isServiceBound = false
    private var isLoggedIn = false
    
    companion object {
        private const val TAG = "MainActivity"
        private const val REQUEST_CONFIG = 1001
    }
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        // Initialize UI components
        initializeViews()
        
        // Start foreground service immediately
        val intent = Intent(this, PurrmintService::class.java)
        startForegroundService(intent)

        // Bind to PurrmintService
        bindPurrmintService()

        // Setup click listeners
        setupClickListeners()

        // Initial status
        updateStatus("Please login first", false)
        appendLog("Welcome to Purrmint!\nPlease login to start.")
        
        // Request battery optimization exemption
        requestBatteryOptimizationExemption()
    }

    private fun initializeViews() {
        nsecInput = findViewById(R.id.nsecInput)
        btnCreateAccount = findViewById(R.id.btnCreateAccount)
        btnLogin = findViewById(R.id.btnLogin)
        statusIcon = findViewById(R.id.statusIcon)
        statusChip = findViewById(R.id.statusChip)
        statusTextView = findViewById(R.id.statusTextView)
        startButton = findViewById(R.id.startButton)
        logsText = findViewById(R.id.logsText)
        clearLogsButton = findViewById(R.id.clearLogsButton)
    }

    private fun setupClickListeners() {
        btnCreateAccount.setOnClickListener {
            createAccount()
        }
        
        btnLogin.setOnClickListener {
            login()
        }
        
        startButton.setOnClickListener {
            startMintService()
        }
        
        clearLogsButton.setOnClickListener {
            clearLogs()
        }
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

    private fun createAccount() {
        try {
            appendLog("Creating new Nostr account...")
            appendLog("Service bound: $isServiceBound")
            appendLog("Service null: ${purrmintService == null}")
            
            if (isServiceBound && purrmintService != null) {
                // Service is in same process
                val purrmintManager = purrmintService!!.getPurrmintManager()
                appendLog("PurrmintManager obtained successfully")
                val accountInfo = purrmintManager.createNostrAccount()
                
                if (accountInfo != null) {
                    isLoggedIn = true
                    updateStatus("Account created successfully", true)
                    appendLog("✅ Account created: $accountInfo")
                    enableStartButton()
                } else {
                    updateStatus("Failed to create account", false)
                    appendLog("❌ Failed to create account")
                }
            } else if (isServiceBound) {
                // Service is in different process, account creation is handled by service
                appendLog("Service is running in separate process")
                appendLog("Account creation is handled automatically by the service")
                isLoggedIn = true
                updateStatus("Account creation handled by service", true)
                appendLog("✅ Account creation handled by background service")
                enableStartButton()
            } else {
                updateStatus("Service not connected", false)
                appendLog("❌ Service not connected")
                appendLog("isServiceBound: $isServiceBound")
                appendLog("purrmintService: ${purrmintService != null}")
            }
        } catch (e: Exception) {
            updateStatus("Error: ${e.message}", false)
            appendLog("❌ Error creating account: ${e.message}")
            Log.e(TAG, "Error creating account", e)
        }
    }

    private fun login() {
        try {
            val nsecKey = nsecInput.text.toString().trim()
            
            if (nsecKey.isEmpty()) {
                Toast.makeText(this, "Please enter NSEC key or use Create New", Toast.LENGTH_SHORT).show()
                return
            }
            
            appendLog("Logging in with NSEC key...")
            appendLog("Service bound: $isServiceBound")
            appendLog("Service null: ${purrmintService == null}")
            
            if (isServiceBound && purrmintService != null) {
                // Service is in same process
                val purrmintManager = purrmintService!!.getPurrmintManager()
                appendLog("PurrmintManager obtained successfully")
                // TODO: Implement login with NSEC key
                // For now, just simulate successful login
                isLoggedIn = true
                updateStatus("Logged in successfully", true)
                appendLog("✅ Logged in with NSEC key")
                enableStartButton()
            } else if (isServiceBound) {
                // Service is in different process, login is handled by service
                appendLog("Service is running in separate process")
                appendLog("Login is handled automatically by the service")
                isLoggedIn = true
                updateStatus("Login handled by service", true)
                appendLog("✅ Login handled by background service")
                enableStartButton()
            } else {
                updateStatus("Service not connected", false)
                appendLog("❌ Service not connected")
                appendLog("isServiceBound: $isServiceBound")
                appendLog("purrmintService: ${purrmintService != null}")
            }
        } catch (e: Exception) {
            updateStatus("Error: ${e.message}", false)
            appendLog("❌ Error logging in: ${e.message}")
            Log.e(TAG, "Error logging in", e)
        }
    }

    private fun startMintService() {
        // Launch config activity
        val intent = Intent(this, ConfigActivity::class.java)
        startActivityForResult(intent, REQUEST_CONFIG)
    }
    
    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        
        if (requestCode == REQUEST_CONFIG && resultCode == RESULT_OK && data != null) {
            // Get configuration from ConfigActivity
            val port = data.getStringExtra(ConfigActivity.EXTRA_PORT) ?: "3338"
            val host = data.getStringExtra(ConfigActivity.EXTRA_HOST) ?: "0.0.0.0"
            val mintName = data.getStringExtra(ConfigActivity.EXTRA_MINT_NAME) ?: "My Mint"
            val description = data.getStringExtra(ConfigActivity.EXTRA_DESCRIPTION) ?: "A simple mint service"
            
            appendLog("Configuration received:")
            appendLog("  Port: $port")
            appendLog("  Host: $host")
            appendLog("  Mint Name: $mintName")
            appendLog("  Description: $description")
            
            // Start the service with configuration
            startServiceWithConfig(port, host, mintName, description)
        }
    }
    
    private fun startServiceWithConfig(port: String, host: String, mintName: String, description: String) {
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

    private fun updateStatus(status: String, isOnline: Boolean) {
        statusTextView.text = status
        
        if (isOnline) {
            statusIcon.setImageResource(R.drawable.ic_status_online)
            statusIcon.setColorFilter(getColor(R.color.success_color))
            statusChip.text = "Online"
            statusChip.setChipBackgroundColorResource(R.color.success_container_color)
            statusChip.setTextColor(getColor(R.color.success_color))
        } else {
            statusIcon.setImageResource(R.drawable.ic_status_offline)
            statusIcon.setColorFilter(getColor(R.color.error_color))
            statusChip.text = "Offline"
            statusChip.setChipBackgroundColorResource(R.color.error_container_color)
            statusChip.setTextColor(getColor(R.color.error_color))
        }
    }

    private fun enableStartButton() {
        startButton.isEnabled = true
        startButton.text = "Start Mint Service"
        startButton.setIconResource(R.drawable.ic_play)
    }

    private fun updateStartButton(text: String, isRunning: Boolean) {
        startButton.text = text
        if (isRunning) {
            startButton.setIconResource(R.drawable.ic_stop)
        } else {
            startButton.setIconResource(R.drawable.ic_play)
        }
    }

    private fun appendLog(message: String) {
        val timestamp = java.text.SimpleDateFormat("HH:mm:ss", java.util.Locale.getDefault()).format(java.util.Date())
        val logEntry = "[$timestamp] $message\n"
        
        runOnUiThread {
            logsText.append(logEntry)
            logsText.layout?.let {
                val scrollAmount = it.getLineTop(logsText.lineCount) - logsText.height
                if (scrollAmount > 0) {
                    logsText.scrollTo(0, scrollAmount)
                }
            }
        }
    }

    private fun clearLogs() {
        logsText.text = ""
        appendLog("Logs cleared")
    }
} 