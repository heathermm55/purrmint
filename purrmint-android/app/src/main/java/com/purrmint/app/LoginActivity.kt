package com.purrmint.app

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

    private fun createAccount() {
        try {
            showStatus("Creating new Nostr account...")
            
            if (isServiceBound && purrmintService != null) {
                // Service is in same process
                val purrmintManager = purrmintService!!.getPurrmintManager()
                val accountInfo = purrmintManager.createNostrAccount()
                
                if (accountInfo != null) {
                    // Save login state
                    loginManager.saveNewAccountState(accountInfo)
                    
                    // Start the service
                    startPurrmintService()
                    
                    // Return success
                    val resultIntent = Intent().apply {
                        putExtra(EXTRA_LOGIN_SUCCESS, true)
                        putExtra(EXTRA_ACCOUNT_INFO, accountInfo)
                    }
                    setResult(RESULT_OK, resultIntent)
                    finish()
                } else {
                    showStatus("Failed to create account")
                    Toast.makeText(this, "Failed to create account", Toast.LENGTH_SHORT).show()
                }
            } else if (isServiceBound) {
                // Service is in different process, account creation is handled by service
                showStatus("Account creation handled by service")
                
                // Try to get account info from service
                try {
                    val accountInfo = purrmintService?.getPurrmintManager()?.createNostrAccount()
                    if (accountInfo != null) {
                        loginManager.saveNewAccountState(accountInfo)
                        
                        // Start the service
                        startPurrmintService()
                        
                        val resultIntent = Intent().apply {
                            putExtra(EXTRA_LOGIN_SUCCESS, true)
                            putExtra(EXTRA_ACCOUNT_INFO, accountInfo)
                        }
                        setResult(RESULT_OK, resultIntent)
                        finish()
                    } else {
                        // Fallback to simulated account
                        val fallbackAccountInfo = "{\"npub\":\"npub1placeholder\",\"nsec\":\"nsec1placeholder\"}"
                        loginManager.saveNewAccountState(fallbackAccountInfo)
                        
                        // Start the service
                        startPurrmintService()
                        
                        val resultIntent = Intent().apply {
                            putExtra(EXTRA_LOGIN_SUCCESS, true)
                            putExtra(EXTRA_ACCOUNT_INFO, fallbackAccountInfo)
                        }
                        setResult(RESULT_OK, resultIntent)
                        finish()
                    }
                } catch (e: Exception) {
                    Log.e(TAG, "Error creating account via service", e)
                    // Fallback to simulated account
                    val fallbackAccountInfo = "{\"npub\":\"npub1placeholder\",\"nsec\":\"nsec1placeholder\"}"
                    loginManager.saveNewAccountState(fallbackAccountInfo)
                    
                    // Start the service
                    startPurrmintService()
                    
                    val resultIntent = Intent().apply {
                        putExtra(EXTRA_LOGIN_SUCCESS, true)
                        putExtra(EXTRA_ACCOUNT_INFO, fallbackAccountInfo)
                    }
                    setResult(RESULT_OK, resultIntent)
                    finish()
                }
            } else {
                showStatus("Service not connected")
                Toast.makeText(this, "Service not connected", Toast.LENGTH_SHORT).show()
            }
        } catch (e: Exception) {
            showStatus("Error: ${e.message}")
            Toast.makeText(this, "Error creating account: ${e.message}", Toast.LENGTH_SHORT).show()
            Log.e(TAG, "Error creating account", e)
        }
    }

    private fun login() {
        try {
            val nsecKey = nsecInput.text.toString().trim()
            
            if (nsecKey.isEmpty()) {
                Toast.makeText(this, "Please enter NSEC key", Toast.LENGTH_SHORT).show()
                return
            }
            
            showStatus("Logging in...")
            
            if (isServiceBound && purrmintService != null) {
                // Service is in same process
                val purrmintManager = purrmintService!!.getPurrmintManager()
                
                // TODO: Implement actual login validation with NSEC key
                // For now, just simulate successful login
                val accountInfo = "Logged in with NSEC key"
                
                // Save login state
                loginManager.saveLoginState(nsecKey, accountInfo)
                
                // Start the service
                startPurrmintService()
                
                // Return success
                val resultIntent = Intent().apply {
                    putExtra(EXTRA_LOGIN_SUCCESS, true)
                    putExtra(EXTRA_ACCOUNT_INFO, accountInfo)
                }
                setResult(RESULT_OK, resultIntent)
                finish()
            } else if (isServiceBound) {
                // Service is in different process, login is handled by service
                showStatus("Login handled by service")
                
                // Try to validate NSEC key and get account info
                try {
                    // For now, create a simple account info with the provided NSEC
                    val accountInfo = "{\"npub\":\"npub1fromnsec\",\"nsec\":\"$nsecKey\"}"
                    loginManager.saveLoginState(nsecKey, accountInfo)
                    
                    // Start the service
                    startPurrmintService()
                    
                    val resultIntent = Intent().apply {
                        putExtra(EXTRA_LOGIN_SUCCESS, true)
                        putExtra(EXTRA_ACCOUNT_INFO, accountInfo)
                    }
                    setResult(RESULT_OK, resultIntent)
                    finish()
                } catch (e: Exception) {
                    Log.e(TAG, "Error logging in via service", e)
                    showStatus("Error: ${e.message}")
                    Toast.makeText(this, "Error logging in: ${e.message}", Toast.LENGTH_SHORT).show()
                }
            } else {
                showStatus("Service not connected")
                Toast.makeText(this, "Service not connected", Toast.LENGTH_SHORT).show()
            }
        } catch (e: Exception) {
            showStatus("Error: ${e.message}")
            Toast.makeText(this, "Error logging in: ${e.message}", Toast.LENGTH_SHORT).show()
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