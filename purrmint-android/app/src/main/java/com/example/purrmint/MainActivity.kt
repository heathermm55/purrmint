package com.example.purrmint

import android.os.Bundle
import android.widget.Button
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity

class MainActivity : AppCompatActivity() {
    private lateinit var statusTextView: TextView
    private lateinit var infoTextView: TextView
    private lateinit var logsTextView: TextView
    private lateinit var selectedModeText: TextView
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        // Initialize views
        statusTextView = findViewById(R.id.statusTextView)
        infoTextView = findViewById(R.id.infoTextView)
        logsTextView = findViewById(R.id.logsText)
        selectedModeText = findViewById(R.id.selectedModeText)

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
    }

    private fun startMintService() {
        try {
            // For now, just show a simple message
            statusTextView.text = "Service starting..."
            logsTextView.text = "Starting mint service...\n"
        } catch (e: Exception) {
            statusTextView.text = "Error: ${e.message}"
            logsTextView.text = "Error starting service: ${e.message}\n"
        }
    }

    private fun stopMintService() {
        try {
            statusTextView.text = "Service stopping..."
            logsTextView.text = "Stopping mint service...\n"
        } catch (e: Exception) {
            statusTextView.text = "Error: ${e.message}"
            logsTextView.text = "Error stopping service: ${e.message}\n"
        }
    }

    private fun checkStatus() {
        try {
            statusTextView.text = "Service status: Not running"
            logsTextView.text = "Checking service status...\n"
        } catch (e: Exception) {
            statusTextView.text = "Error: ${e.message}"
            logsTextView.text = "Error checking status: ${e.message}\n"
        }
    }

    private fun generateConfig() {
        try {
            infoTextView.text = "Generating configuration..."
            logsTextView.text = "Generating mint configuration...\n"
        } catch (e: Exception) {
            infoTextView.text = "Error: ${e.message}"
            logsTextView.text = "Error generating config: ${e.message}\n"
        }
    }

    private fun createAccount() {
        try {
            infoTextView.text = "Creating Nostr account..."
            logsTextView.text = "Creating new Nostr account...\n"
        } catch (e: Exception) {
            infoTextView.text = "Error: ${e.message}"
            logsTextView.text = "Error creating account: ${e.message}\n"
        }
    }

    private fun getInfo() {
        try {
            infoTextView.text = "Getting service information..."
            logsTextView.text = "Getting service info...\n"
        } catch (e: Exception) {
            infoTextView.text = "Error: ${e.message}"
            logsTextView.text = "Error getting info: ${e.message}\n"
        }
    }

    private fun getAccessUrls() {
        try {
            infoTextView.text = "Getting access URLs..."
            logsTextView.text = "Getting access URLs...\n"
        } catch (e: Exception) {
            infoTextView.text = "Error: ${e.message}"
            logsTextView.text = "Error getting URLs: ${e.message}\n"
        }
    }

    private fun clearLogs() {
        logsTextView.text = "Logs cleared\n"
    }
} 