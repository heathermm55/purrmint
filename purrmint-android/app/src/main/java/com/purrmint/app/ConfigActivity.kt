package com.purrmint.app

import android.content.Intent
import android.os.Bundle
import android.util.Log
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
    
    companion object {
        private const val TAG = "ConfigActivity"
        const val EXTRA_PORT = "port"
        const val EXTRA_HOST = "host"
        const val EXTRA_MINT_NAME = "mint_name"
        const val EXTRA_DESCRIPTION = "description"
    }
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_config)
        
        initializeViews()
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
    }
    
    private fun startService() {
        try {
            val port = portInput.text.toString().trim()
            val host = hostInput.text.toString().trim()
            val mintName = mintNameInput.text.toString().trim()
            val description = descriptionInput.text.toString().trim()
            
            // Validate inputs
            if (port.isEmpty() || host.isEmpty() || mintName.isEmpty()) {
                Log.w(TAG, "Required fields are empty")
                return
            }
            
            // Create result intent
            val resultIntent = Intent().apply {
                putExtra(EXTRA_PORT, port)
                putExtra(EXTRA_HOST, host)
                putExtra(EXTRA_MINT_NAME, mintName)
                putExtra(EXTRA_DESCRIPTION, description)
            }
            
            setResult(RESULT_OK, resultIntent)
            finish()
            
        } catch (e: Exception) {
            Log.e(TAG, "Error starting service", e)
        }
    }
} 