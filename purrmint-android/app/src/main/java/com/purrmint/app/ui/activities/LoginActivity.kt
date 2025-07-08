package com.purrmint.app.ui.activities

import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.ServiceConnection
import android.os.Build
import android.os.Bundle
import android.os.IBinder
import android.util.Log
import android.view.View
import android.widget.TextView
import android.widget.Toast
import androidx.appcompat.app.AppCompatActivity
import com.google.android.material.button.MaterialButton
import com.google.android.material.textfield.TextInputEditText
import com.purrmint.app.R
import com.purrmint.app.core.managers.LoginManager
import com.purrmint.app.core.services.PurrmintService

class LoginActivity : AppCompatActivity() {
    
    // UI Components
    private lateinit var nsecInput: TextInputEditText
    private lateinit var btnCreateAccount: MaterialButton
    private lateinit var btnLogin: MaterialButton
    private lateinit var statusText: TextView
    
    // Service
    private var purrmintService: PurrmintService? = null
    private var isServiceBound = false
    
    // Login Manager
    private lateinit var loginManager: LoginManager
    
    companion object {
        private const val TAG = "LoginActivity"
        const val EXTRA_LOGIN_SUCCESS = "login_success"
        const val EXTRA_ACCOUNT_INFO = "account_info"
    }
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_login)

        // Initialize login manager
        loginManager = LoginManager(this)
        
        // Initialize UI components
        initializeViews()
        
        // Bind to PurrmintService (service will be started when needed)
        bindPurrmintService()

        // Setup click listeners
        setupClickListeners()
    }

    private fun initializeViews() {
        nsecInput = findViewById(R.id.nsecInput)
        btnCreateAccount = findViewById(R.id.btnCreateAccount)
        btnLogin = findViewById(R.id.btnLogin)
        statusText = findViewById(R.id.statusText)
    }

    private fun setupClickListeners() {
        btnCreateAccount.setOnClickListener {
            createAccount()
        }
        
        btnLogin.setOnClickListener {
            login()
        }
    }

    private val serviceConnection = object : ServiceConnection {
        override fun onServiceConnected(name: ComponentName?, service: IBinder?) {
            try {
                if (service is PurrmintService.LocalBinder) {
                    purrmintService = service.getService()
                    isServiceBound = true
                    Log.i(TAG, "PurrmintService connected (same process)")
                } else {
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
        try {
            val intent = Intent(this, PurrmintService::class.java)
            bindService(intent, serviceConnection, Context.BIND_AUTO_CREATE)
        } catch (e: Exception) {
            Log.e(TAG, "Error binding to service", e)
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        if (isServiceBound) {
            unbindService(serviceConnection)
            isServiceBound = false
        }
    }

    private fun goToMainActivity() {
        // Go to MainActivity and clear back stack
        val intent = Intent(this, MainActivity::class.java)
        intent.flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TASK
        startActivity(intent)
        finish()
    }

    private fun createAccount() {
        try {
            showStatus(getString(R.string.creating_new_account))
            
            // Use LoginManager to create account
            val success = loginManager.createNewAccount()
            
            if (success) {
                // Start the service
                startPurrmintService()
                
                // Go to main activity
                goToMainActivity()
            } else {
                showStatus(getString(R.string.failed_to_create_account))
                Toast.makeText(this, getString(R.string.failed_to_create_account), Toast.LENGTH_SHORT).show()
            }
        } catch (e: Exception) {
            showStatus("Error: ${e.message}")
            Toast.makeText(this, getString(R.string.error_creating_account, e.message), Toast.LENGTH_SHORT).show()
            Log.e(TAG, "Error creating account", e)
        }
    }

    private fun login() {
        try {
            val nsecKey = nsecInput.text.toString().trim()
            
            if (nsecKey.isEmpty()) {
                Toast.makeText(this, getString(R.string.please_enter_nsec_key), Toast.LENGTH_SHORT).show()
                return
            }
            
            showStatus(getString(R.string.logging_in))
            
            // Use LoginManager to login with NSEC
            val success = loginManager.loginWithNsec(nsecKey)
            
            if (success) {
                // Start the service
                startPurrmintService()
                
                // Go to main activity
                goToMainActivity()
            } else {
                showStatus(getString(R.string.login_failed_invalid_nsec))
                Toast.makeText(this, getString(R.string.login_failed_invalid_nsec), Toast.LENGTH_SHORT).show()
            }
        } catch (e: Exception) {
            showStatus("Error: ${e.message}")
            Toast.makeText(this, getString(R.string.error_logging_in, e.message), Toast.LENGTH_SHORT).show()
            Log.e(TAG, "Error logging in", e)
        }
    }

    private fun startPurrmintService() {
        try {
            val intent = Intent(this, PurrmintService::class.java)
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                startForegroundService(intent)
            } else {
                startService(intent)
            }
            Log.i(TAG, "PurrmintService started after login")
        } catch (e: Exception) {
            Log.e(TAG, "Error starting service", e)
        }
    }

    private fun showStatus(message: String) {
        runOnUiThread {
            statusText.text = message
            statusText.visibility = View.VISIBLE
        }
    }
} 