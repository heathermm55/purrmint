package com.example.purrmint

import android.os.Bundle
import android.util.Log
import android.widget.Button
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity
import org.json.JSONObject
import kotlinx.coroutines.*

class MainActivity : AppCompatActivity() {
    
    private lateinit var outputText: TextView
    private lateinit var testButton: Button
    private lateinit var createAccountButton: Button
    private lateinit var getInfoButton: Button
    private lateinit var getStatusButton: Button
    
    private val TAG = "PurrMint"
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)
        
        initializeViews()
        setupButtons()
        
        // Test FFI on startup
        testFfiInterface()
    }
    
    private fun initializeViews() {
        outputText = findViewById(R.id.outputText)
        testButton = findViewById(R.id.testButton)
        createAccountButton = findViewById(R.id.createAccountButton)
        getInfoButton = findViewById(R.id.getInfoButton)
        getStatusButton = findViewById(R.id.getStatusButton)
    }
    
    private fun setupButtons() {
        testButton.setOnClickListener { testFfiInterface() }
        createAccountButton.setOnClickListener { createNostrAccount() }
        getInfoButton.setOnClickListener { getMintInfo() }
        getStatusButton.setOnClickListener { getMintStatus() }
    }
    
    private fun testFfiInterface() {
        CoroutineScope(Dispatchers.IO).launch {
            try {
                val result = PurrmintNative.testFfi()
                val json = PurrmintNative.parseJsonResponse(result)
                
                withContext(Dispatchers.Main) {
                    if (json != null) {
                        val status = json.optString("status", "unknown")
                        val message = json.optString("message", "")
                        outputText.text = "FFI Test: $status - $message"
                        Log.d(TAG, "FFI test successful: $result")
                    } else {
                        outputText.text = "FFI test failed"
                        Log.e(TAG, "FFI test failed: $result")
                    }
                }
            } catch (e: Exception) {
                withContext(Dispatchers.Main) {
                    outputText.text = "FFI test error: ${e.message}"
                    Log.e(TAG, "FFI test error", e)
                }
            }
        }
    }
    
    private fun createNostrAccount() {
        CoroutineScope(Dispatchers.IO).launch {
            try {
                val account = PurrmintNative.createAccount()
                
                withContext(Dispatchers.Main) {
                    if (account != null && account.isValid()) {
                        outputText.text = "Account created: ${account.getDisplayPubkey()}"
                        Log.d(TAG, "Account created: ${account.pubkey}")
                    } else {
                        outputText.text = "Failed to create account"
                        Log.e(TAG, "Failed to create account")
                    }
                }
            } catch (e: Exception) {
                withContext(Dispatchers.Main) {
                    outputText.text = "Account error: ${e.message}"
                    Log.e(TAG, "Account error", e)
                }
            }
        }
    }
    
    private fun getMintInfo() {
        CoroutineScope(Dispatchers.IO).launch {
            try {
                val info = PurrmintNative.getMintInfo()
                val json = PurrmintNative.parseJsonResponse(info)
                
                withContext(Dispatchers.Main) {
                    if (json != null) {
                        val status = json.optString("status", "unknown")
                        val version = json.optString("version", "unknown")
                        outputText.text = "Mint Info: $status v$version"
                        Log.d(TAG, "Mint info: $info")
                    } else {
                        outputText.text = "Failed to get mint info"
                        Log.e(TAG, "Failed to get mint info: $info")
                    }
                }
            } catch (e: Exception) {
                withContext(Dispatchers.Main) {
                    outputText.text = "Mint info error: ${e.message}"
                    Log.e(TAG, "Mint info error", e)
                }
            }
        }
    }
    
    private fun getMintStatus() {
        CoroutineScope(Dispatchers.IO).launch {
            try {
                val status = PurrmintNative.getMintStatus()
                val json = PurrmintNative.parseJsonResponse(status)
                
                withContext(Dispatchers.Main) {
                    if (json != null) {
                        val running = json.optBoolean("running", false)
                        outputText.text = "Mint Status: ${if (running) "Running" else "Stopped"}"
                        Log.d(TAG, "Mint status: $status")
                    } else {
                        outputText.text = "Failed to get mint status"
                        Log.e(TAG, "Failed to get mint status: $status")
                    }
                }
            } catch (e: Exception) {
                withContext(Dispatchers.Main) {
                    outputText.text = "Mint status error: ${e.message}"
                    Log.e(TAG, "Mint status error", e)
                }
            }
        }
    }
} 