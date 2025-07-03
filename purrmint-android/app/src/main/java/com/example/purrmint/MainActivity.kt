package com.example.purrmint

import android.os.Bundle
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
    
    private lateinit var purrmintManager: PurrmintManager
    
    companion object {
        private const val TAG = "MainActivity"
    }
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        // Initialize PurrmintManager
        purrmintManager = PurrmintManager(this)

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
    }

    private fun startMintService() {
        try {
            statusTextView.text = "Service starting..."
            logsTextView.text = "Starting mint service...\n"

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
                statusTextView.text = "Service status: Running"
                logsTextView.text = logsTextView.text.toString() + "Mint service started successfully!\n"
            } else {
                statusTextView.text = "Service status: Failed to start"
                logsTextView.text = logsTextView.text.toString() + "Failed to start mint service\n"
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
            
            val success = purrmintManager.stopMintService()
            
            if (success) {
                statusTextView.text = "Service status: Stopped"
                logsTextView.text = logsTextView.text.toString() + "Mint service stopped successfully!\n"
            } else {
                statusTextView.text = "Service status: Failed to stop"
                logsTextView.text = logsTextView.text.toString() + "Failed to stop mint service\n"
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
            
            val status = purrmintManager.getServiceStatus()
            statusTextView.text = "Service status: $status"
            logsTextView.text = logsTextView.text.toString() + "Status: $status\n"
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
            
            val success = purrmintManager.generateConfig()
            
            if (success) {
                infoTextView.text = "Configuration generated successfully"
                logsTextView.text = logsTextView.text.toString() + "Configuration generated successfully!\n"
            } else {
                infoTextView.text = "Failed to generate configuration"
                logsTextView.text = logsTextView.text.toString() + "Failed to generate configuration\n"
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
            
            val accountInfo = purrmintManager.createNostrAccount()
            
            if (accountInfo != null) {
                infoTextView.text = "Account created successfully"
                accountInfoText.text = accountInfo
                logsTextView.text = logsTextView.text.toString() + "Account created: $accountInfo\n"
            } else {
                infoTextView.text = "Failed to create account"
                logsTextView.text = logsTextView.text.toString() + "Failed to create account\n"
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
            
            val info = purrmintManager.getServiceInfo()
            infoTextView.text = "Service info: $info"
            logsTextView.text = logsTextView.text.toString() + "Info: $info\n"
        } catch (e: Exception) {
            infoTextView.text = "Error: ${e.message}"
            logsTextView.text = logsTextView.text.toString() + "Error getting info: ${e.message}\n"
            Log.e(TAG, "Error getting info", e)
        }
    }

    private fun getAccessUrls() {
        try {
            infoTextView.text = "Getting access URLs..."
            logsTextView.text = logsTextView.text.toString() + "Getting access URLs...\n"
            
            // For now, just show a placeholder
            val urls = "{\"http\":\"http://127.0.0.1:3338\",\"nip74\":\"nostr://...\"}"
            infoTextView.text = "Access URLs: $urls"
            logsTextView.text = logsTextView.text.toString() + "URLs: $urls\n"
        } catch (e: Exception) {
            infoTextView.text = "Error: ${e.message}"
            logsTextView.text = logsTextView.text.toString() + "Error getting URLs: ${e.message}\n"
            Log.e(TAG, "Error getting URLs", e)
        }
    }

    private fun clearLogs() {
        logsTextView.text = "Logs cleared\n"
    }
    
    private fun testFfiConnection() {
        try {
            logsTextView.text = logsTextView.text.toString() + "Testing FFI connection...\n"
            val result = purrmintManager.testFfi()
            logsTextView.text = logsTextView.text.toString() + "FFI test result: $result\n"
        } catch (e: Exception) {
            logsTextView.text = logsTextView.text.toString() + "FFI test failed: ${e.message}\n"
            Log.e(TAG, "FFI test failed", e)
        }
    }
} 