package com.example.purrmint

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
import androidx.appcompat.app.AppCompatActivity

class MainActivity : AppCompatActivity() {
    private lateinit var statusTextView: TextView
    private lateinit var infoTextView: TextView
    private lateinit var logsTextView: TextView
    private lateinit var selectedModeText: TextView
    private lateinit var accountInfoText: TextView
    
    private var purrmintService: PurrmintService? = null
    private var isServiceBound = false
    
    companion object {
        private const val TAG = "MainActivity"
    }
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        // Start foreground service immediately
        val intent = Intent(this, PurrmintService::class.java)
        startForegroundService(intent)

        // Bind to PurrmintService
        bindPurrmintService()

        // Initialize views
        statusTextView = findViewById(R.id.statusTextView)
        infoTextView = findViewById(R.id.infoTextView)
        logsTextView = findViewById(R.id.logsText)
        selectedModeText = findViewById(R.id.selectedModeText)
        accountInfoText = findViewById(R.id.accountInfoText)

        // Setup buttons
        findViewById<Button>(R.id.startButton).setOnClickListener {
            startMintService()
        }

        findViewById<Button>(R.id.stopButton).setOnClickListener {
            stopMintService()
        }

        findViewById<Button>(R.id.btnCheckStatus).setOnClickListener {
            checkStatus()
        }

        findViewById<Button>(R.id.btnGenerateConfig).setOnClickListener {
            generateConfig()
        }

        findViewById<Button>(R.id.btnCreateAccount).setOnClickListener {
            createAccount()
        }

        findViewById<Button>(R.id.btnGetInfo).setOnClickListener {
            getInfo()
        }

        findViewById<Button>(R.id.btnGetAccessUrls).setOnClickListener {
            getAccessUrls()
        }

        findViewById<Button>(R.id.clearLogsButton).setOnClickListener {
            clearLogs()
        }

        // Initialize with default mode
        selectedModeText.text = "Selected Mode: Mintd Only (HTTP API)"
        
        // Initial status check
        checkStatus()
        
        // Test FFI connection
        testFfiConnection()
        
        // Request battery optimization exemption
        requestBatteryOptimizationExemption()
    }

    private val serviceConnection = object : ServiceConnection {
        override fun onServiceConnected(name: ComponentName?, service: IBinder?) {
            try {
                // Check if service is in same process (LocalBinder) or different process (BinderProxy)
                if (service is PurrmintService.LocalBinder) {
                    purrmintService = service.getService()
                    isServiceBound = true
                    Log.i(TAG, "PurrmintService connected (same process)")
                } else {
                    // Service is in different process, we can't directly access it
                    // But we can still communicate via Intent
                    isServiceBound = true
                    Log.i(TAG, "PurrmintService connected (different process)")
                }
            } catch (e: Exception) {
                Log.e(TAG, "Error connecting to service", e)
                isServiceBound = false
            }
        }

        override fun onServiceDisconnected(name: ComponentName?) {
            purrmintService = null
            isServiceBound = false
            Log.i(TAG, "PurrmintService disconnected")
        }
    }

    private fun bindPurrmintService() {
        val intent = Intent(this, PurrmintService::class.java)
        bindService(intent, serviceConnection, Context.BIND_AUTO_CREATE)
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

    private fun startMintService() {
        try {
            statusTextView.text = "Service starting..."
            logsTextView.text = "Starting mint service...\n"

            // Service should already be started in onCreate, just check status
            if (isServiceBound && purrmintService != null) {
                // Service is in same process, we can access PurrmintManager directly
                val purrmintManager = purrmintService!!.getPurrmintManager()
                
                // Auto-generate config if not exists
                if (!purrmintManager.configExists()) {
                    logsTextView.text = logsTextView.text.toString() + "Generating config...\n"
                    purrmintManager.generateConfig()
                }
                // Auto-generate Nostr account if not exists
                if (!purrmintManager.accountExists()) {
                    logsTextView.text = logsTextView.text.toString() + "Creating Nostr account...\n"
                    purrmintManager.createNostrAccount()
                }

                val success = purrmintManager.startMintService()
                if (success) {
                    statusTextView.text = "Service status: Running (Foreground)"
                    logsTextView.text = logsTextView.text.toString() + "Mint service started successfully in foreground!\n"
                } else {
                    statusTextView.text = "Service status: Failed to start"
                    logsTextView.text = logsTextView.text.toString() + "Failed to start mint service\n"
                }
            } else {
                // Service is in different process, just show that it's running
                statusTextView.text = "Service status: Running"
                logsTextView.text = logsTextView.text.toString() + "Service is running!\n"
                logsTextView.text = logsTextView.text.toString() + "Service is running in separate process for better reliability.\n"
            }
        } catch (e: Exception) {
            statusTextView.text = "Error: ${e.message}"
            logsTextView.text = logsTextView.text.toString() + "Error starting service: ${e.message}\n"
            Log.e(TAG, "Error starting service", e)
        }
    }

    private fun stopMintService() {
        try {
            statusTextView.text = "Service stopping..."
            logsTextView.text = logsTextView.text.toString() + "Stopping mint service...\n"
            
            if (isServiceBound && purrmintService != null) {
                // Service is in same process, we can access PurrmintManager directly
                val purrmintManager = purrmintService!!.getPurrmintManager()
                val success = purrmintManager.stopMintService()
                
                if (success) {
                    statusTextView.text = "Service status: Stopped"
                    logsTextView.text = logsTextView.text.toString() + "Mint service stopped successfully!\n"
                } else {
                    statusTextView.text = "Service status: Failed to stop"
                    logsTextView.text = logsTextView.text.toString() + "Failed to stop mint service\n"
                }
            } else {
                // Service is in different process, stop it via Intent
                val intent = Intent(this, PurrmintService::class.java)
                stopService(intent)
                statusTextView.text = "Service status: Stopped"
                logsTextView.text = logsTextView.text.toString() + "Service stopped successfully!\n"
            }
        } catch (e: Exception) {
            statusTextView.text = "Error: ${e.message}"
            logsTextView.text = logsTextView.text.toString() + "Error stopping service: ${e.message}\n"
            Log.e(TAG, "Error stopping service", e)
        }
    }

    private fun checkStatus() {
        try {
            statusTextView.text = "Checking status..."
            logsTextView.text = logsTextView.text.toString() + "Checking service status...\n"
            
            if (isServiceBound && purrmintService != null) {
                // Service is in same process, we can access PurrmintManager directly
                val purrmintManager = purrmintService!!.getPurrmintManager()
                val status = purrmintManager.getServiceStatus()
                statusTextView.text = "Service status: $status"
                logsTextView.text = logsTextView.text.toString() + "Status: $status\n"
            } else if (isServiceBound) {
                // Service is in different process, check via HTTP
                statusTextView.text = "Service status: Running"
                logsTextView.text = logsTextView.text.toString() + "Service is running (separate process)\n"
                
                // Test HTTP connection to verify service is working
                Thread {
                    try {
                        val deviceIp = getDeviceIpAddress()
                        val url = "http://$deviceIp:3338/v1/info"
                        
                        val connection = java.net.URL(url).openConnection() as java.net.HttpURLConnection
                        connection.connectTimeout = 5000
                        connection.readTimeout = 5000
                        connection.requestMethod = "GET"
                        
                        val responseCode = connection.responseCode
                        runOnUiThread {
                            if (responseCode == 200) {
                                logsTextView.text = logsTextView.text.toString() + "✅ HTTP connection successful: $url\n"
                                logsTextView.text = logsTextView.text.toString() + "✅ Mint service is running and responding!\n"
                            } else {
                                logsTextView.text = logsTextView.text.toString() + "⚠️ HTTP connection returned code: $responseCode\n"
                            }
                        }
                    } catch (e: Exception) {
                        runOnUiThread {
                            logsTextView.text = logsTextView.text.toString() + "⚠️ HTTP connection failed: ${e.message}\n"
                        }
                    }
                }.start()
            } else {
                statusTextView.text = "Service status: Not connected"
                logsTextView.text = logsTextView.text.toString() + "Service not connected\n"
            }
        } catch (e: Exception) {
            statusTextView.text = "Error: ${e.message}"
            logsTextView.text = logsTextView.text.toString() + "Error checking status: ${e.message}\n"
            Log.e(TAG, "Error checking status", e)
        }
    }

    private fun generateConfig() {
        try {
            infoTextView.text = "Generating configuration..."
            logsTextView.text = logsTextView.text.toString() + "Generating mint configuration...\n"
            
            if (isServiceBound && purrmintService != null) {
                // Service is in same process, we can access PurrmintManager directly
                val purrmintManager = purrmintService!!.getPurrmintManager()
                val success = purrmintManager.generateConfig()
                
                if (success) {
                    infoTextView.text = "Configuration generated successfully"
                    logsTextView.text = logsTextView.text.toString() + "Configuration generated successfully!\n"
                } else {
                    infoTextView.text = "Failed to generate configuration"
                    logsTextView.text = logsTextView.text.toString() + "Failed to generate configuration\n"
                }
            } else if (isServiceBound) {
                // Service is in different process, configuration should be handled by the service itself
                infoTextView.text = "Configuration handled by service"
                logsTextView.text = logsTextView.text.toString() + "Configuration generation is handled by the service.\n"
                logsTextView.text = logsTextView.text.toString() + "The service will auto-generate config when needed.\n"
            } else {
                infoTextView.text = "Service not connected"
                logsTextView.text = logsTextView.text.toString() + "Service not connected\n"
            }
        } catch (e: Exception) {
            infoTextView.text = "Error: ${e.message}"
            logsTextView.text = logsTextView.text.toString() + "Error generating config: ${e.message}\n"
            Log.e(TAG, "Error generating config", e)
        }
    }

    private fun createAccount() {
        try {
            infoTextView.text = "Creating Nostr account..."
            logsTextView.text = logsTextView.text.toString() + "Creating new Nostr account...\n"
            
            if (isServiceBound && purrmintService != null) {
                // Service is in same process, we can access PurrmintManager directly
                val purrmintManager = purrmintService!!.getPurrmintManager()
                val accountInfo = purrmintManager.createNostrAccount()
                
                if (accountInfo != null) {
                    infoTextView.text = "Account created successfully"
                    accountInfoText.text = accountInfo
                    logsTextView.text = logsTextView.text.toString() + "Account created: $accountInfo\n"
                } else {
                    infoTextView.text = "Failed to create account"
                    logsTextView.text = logsTextView.text.toString() + "Failed to create account\n"
                }
            } else if (isServiceBound) {
                // Service is in different process, account creation should be handled by the service itself
                infoTextView.text = "Account creation handled by service"
                logsTextView.text = logsTextView.text.toString() + "Account creation is handled by the service.\n"
                logsTextView.text = logsTextView.text.toString() + "The service will auto-create account when needed.\n"
            } else {
                infoTextView.text = "Service not connected"
                logsTextView.text = logsTextView.text.toString() + "Service not connected\n"
            }
        } catch (e: Exception) {
            infoTextView.text = "Error: ${e.message}"
            logsTextView.text = logsTextView.text.toString() + "Error creating account: ${e.message}\n"
            Log.e(TAG, "Error creating account", e)
        }
    }

    private fun getInfo() {
        try {
            infoTextView.text = "Getting service information..."
            logsTextView.text = logsTextView.text.toString() + "Getting service info...\n"
            
            if (isServiceBound && purrmintService != null) {
                // Service is in same process, we can access PurrmintManager directly
                val purrmintManager = purrmintService!!.getPurrmintManager()
                val info = purrmintManager.getServiceInfo()
                infoTextView.text = "Service info: $info"
                logsTextView.text = logsTextView.text.toString() + "Info: $info\n"
            } else if (isServiceBound) {
                // Service is in different process, show basic info
                val deviceIp = getDeviceIpAddress()
                val info = "Foreground service running on $deviceIp:3338"
                infoTextView.text = "Service info: $info"
                logsTextView.text = logsTextView.text.toString() + "Info: $info\n"
                logsTextView.text = logsTextView.text.toString() + "Service is running in separate process for better reliability.\n"
            } else {
                infoTextView.text = "Service not connected"
                logsTextView.text = logsTextView.text.toString() + "Service not connected\n"
            }
        } catch (e: Exception) {
            infoTextView.text = "Error: ${e.message}"
            logsTextView.text = logsTextView.text.toString() + "Error getting info: ${e.message}\n"
            Log.e(TAG, "Error getting info", e)
        }
    }

    private fun getAccessUrls() {
        try {
            infoTextView.text = "Testing HTTP connection..."
            logsTextView.text = logsTextView.text.toString() + "Testing HTTP connection to mint service...\n"
            
            if (isServiceBound && purrmintService != null) {
                // Service is in same process, we can access PurrmintManager directly
                val purrmintManager = purrmintService!!.getPurrmintManager()
                
                // Run network test in background thread
                Thread {
                    try {
                        val connectionSuccessful = purrmintManager.testHttpConnection()
                        
                        // Update UI on main thread
                        runOnUiThread {
                            if (connectionSuccessful) {
                                val deviceIp = purrmintManager.getDeviceIpAddress()
                                val urls = "{\"http\":\"http://$deviceIp:3338\",\"nip74\":\"nostr://...\",\"status\":\"connected\"}"
                                infoTextView.text = "✅ HTTP connection successful!"
                                logsTextView.text = logsTextView.text.toString() + "✅ HTTP connection to http://$deviceIp:3338 successful!\n"
                                logsTextView.text = logsTextView.text.toString() + "URLs: $urls\n"
                            } else {
                                val deviceIp = purrmintManager.getDeviceIpAddress()
                                val urls = "{\"http\":\"http://$deviceIp:3338\",\"nip74\":\"nostr://...\",\"status\":\"failed\"}"
                                infoTextView.text = "❌ HTTP connection failed"
                                logsTextView.text = logsTextView.text.toString() + "❌ HTTP connection to http://$deviceIp:3338 failed!\n"
                                logsTextView.text = logsTextView.text.toString() + "URLs: $urls\n"
                            }
                        }
                    } catch (e: Exception) {
                        runOnUiThread {
                            infoTextView.text = "Error: ${e.message}"
                            logsTextView.text = logsTextView.text.toString() + "Error testing HTTP connection: ${e.message}\n"
                            Log.e(TAG, "Error testing HTTP connection", e)
                        }
                    }
                }.start()
            } else if (isServiceBound) {
                // Service is in different process, test via direct HTTP connection
                Thread {
                    try {
                        val deviceIp = getDeviceIpAddress()
                        val url = "http://$deviceIp:3338/v1/info"
                        
                        val connection = java.net.URL(url).openConnection() as java.net.HttpURLConnection
                        connection.connectTimeout = 5000
                        connection.readTimeout = 5000
                        connection.requestMethod = "GET"
                        
                        val responseCode = connection.responseCode
                        runOnUiThread {
                            if (responseCode == 200) {
                                val baseUrl = "http://$deviceIp:3338"
                                val urls = "{\"http\":\"$baseUrl\",\"nip74\":\"nostr://...\",\"status\":\"connected\"}"
                                infoTextView.text = "✅ HTTP connection successful!"
                                logsTextView.text = logsTextView.text.toString() + "✅ HTTP connection to $url successful!\n"
                                logsTextView.text = logsTextView.text.toString() + "✅ Mint service is running and responding!\n"
                                logsTextView.text = logsTextView.text.toString() + "URLs: $urls\n"
                            } else {
                                val baseUrl = "http://$deviceIp:3338"
                                val urls = "{\"http\":\"$baseUrl\",\"nip74\":\"nostr://...\",\"status\":\"failed\"}"
                                infoTextView.text = "❌ HTTP connection failed"
                                logsTextView.text = logsTextView.text.toString() + "❌ HTTP connection to $url failed (code: $responseCode)!\n"
                                logsTextView.text = logsTextView.text.toString() + "URLs: $urls\n"
                            }
                        }
                    } catch (e: Exception) {
                        runOnUiThread {
                            val deviceIp = getDeviceIpAddress()
                            val baseUrl = "http://$deviceIp:3338"
                            val urls = "{\"http\":\"$baseUrl\",\"nip74\":\"nostr://...\",\"status\":\"failed\"}"
                            infoTextView.text = "❌ HTTP connection failed"
                            logsTextView.text = logsTextView.text.toString() + "❌ HTTP connection to $baseUrl failed: ${e.message}\n"
                            logsTextView.text = logsTextView.text.toString() + "URLs: $urls\n"
                            Log.e(TAG, "Error testing HTTP connection", e)
                        }
                    }
                }.start()
            } else {
                infoTextView.text = "Service not connected"
                logsTextView.text = logsTextView.text.toString() + "Service not connected\n"
            }
            
        } catch (e: Exception) {
            infoTextView.text = "Error: ${e.message}"
            logsTextView.text = logsTextView.text.toString() + "Error testing HTTP connection: ${e.message}\n"
            Log.e(TAG, "Error testing HTTP connection", e)
        }
    }

    private fun clearLogs() {
        logsTextView.text = "Logs cleared\n"
    }
    
    private fun getDeviceIpAddress(): String {
        try {
            val wifiManager = applicationContext.getSystemService(Context.WIFI_SERVICE) as android.net.wifi.WifiManager
            val wifiInfo = wifiManager.connectionInfo
            if (wifiInfo != null && wifiInfo.ipAddress != 0) {
                return android.text.format.Formatter.formatIpAddress(wifiInfo.ipAddress)
            }
        } catch (e: Exception) {
            Log.w(TAG, "Could not get WiFi IP address", e)
        }
        
        // Fallback to localhost
        return "127.0.0.1"
    }
    
    private fun testFfiConnection() {
        try {
            logsTextView.text = logsTextView.text.toString() + "Testing FFI connection...\n"
            if (isServiceBound && purrmintService != null) {
                // Service is in same process, we can access PurrmintManager directly
                val purrmintManager = purrmintService!!.getPurrmintManager()
                val result = purrmintManager.testFfi()
                logsTextView.text = logsTextView.text.toString() + "FFI test result: $result\n"
            } else if (isServiceBound) {
                // Service is in different process, FFI is handled by the service
                logsTextView.text = logsTextView.text.toString() + "FFI is handled by foreground service (separate process)\n"
            } else {
                logsTextView.text = logsTextView.text.toString() + "Service not connected\n"
            }
        } catch (e: Exception) {
            logsTextView.text = logsTextView.text.toString() + "FFI test failed: ${e.message}\n"
            Log.e(TAG, "FFI test failed", e)
        }
    }
} 