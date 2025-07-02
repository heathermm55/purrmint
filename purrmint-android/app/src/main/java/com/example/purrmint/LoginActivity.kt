package com.example.purrmint

import android.content.Intent
import android.os.Bundle
import android.util.Log
import android.view.View
import android.widget.Button
import android.widget.EditText
import android.widget.TextView
import android.widget.Toast
import androidx.appcompat.app.AppCompatActivity
import kotlinx.coroutines.*

class LoginActivity : AppCompatActivity() {
    
    private lateinit var createAccountButton: Button
    private lateinit var importAccountButton: Button
    private lateinit var secretKeyInput: EditText
    private lateinit var statusText: TextView
    
    private val TAG = "LoginActivity"
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_login)
        
        initializeViews()
        setupButtons()
        
        // Check if user is already logged in
        checkExistingAccount()
    }
    
    private fun initializeViews() {
        createAccountButton = findViewById(R.id.createAccountButton)
        importAccountButton = findViewById(R.id.importAccountButton)
        secretKeyInput = findViewById(R.id.secretKeyInput)
        statusText = findViewById(R.id.statusText)
    }
    
    private fun setupButtons() {
        createAccountButton.setOnClickListener { createNewAccount() }
        importAccountButton.setOnClickListener { importExistingAccount() }
    }
    
    private fun checkExistingAccount() {
        CoroutineScope(Dispatchers.IO).launch {
            try {
                val accountInfo = PurrmintNative.getCurrentAccount()
                val json = PurrmintNative.parseJsonResponse(accountInfo)
                
                withContext(Dispatchers.Main) {
                    if (json != null && json.has("pubkey")) {
                        // User is already logged in, go to main activity
                        startMainActivity()
                    }
                }
            } catch (e: Exception) {
                Log.e(TAG, "Error checking existing account", e)
            }
        }
    }
    
    private fun createNewAccount() {
        createAccountButton.isEnabled = false
        statusText.text = "Creating new account..."
        
        CoroutineScope(Dispatchers.IO).launch {
            try {
                val account = PurrmintNative.createAccount()
                
                withContext(Dispatchers.Main) {
                    if (account != null && account.isValid()) {
                        statusText.text = "Account created successfully!"
                        Toast.makeText(this@LoginActivity, 
                            "Account created: ${account.getDisplayPubkey()}", 
                            Toast.LENGTH_SHORT).show()
                        
                        // Navigate to main activity
                        startMainActivity()
                    } else {
                        statusText.text = "Failed to create account"
                        createAccountButton.isEnabled = true
                        Toast.makeText(this@LoginActivity, 
                            "Failed to create account", 
                            Toast.LENGTH_SHORT).show()
                    }
                }
            } catch (e: Exception) {
                withContext(Dispatchers.Main) {
                    statusText.text = "Error: ${e.message}"
                    createAccountButton.isEnabled = true
                    Toast.makeText(this@LoginActivity, 
                        "Error creating account: ${e.message}", 
                        Toast.LENGTH_SHORT).show()
                }
                Log.e(TAG, "Error creating account", e)
            }
        }
    }
    
    private fun importExistingAccount() {
        val secretKey = secretKeyInput.text.toString().trim()
        
        if (secretKey.isEmpty()) {
            Toast.makeText(this, "Please enter your secret key", Toast.LENGTH_SHORT).show()
            return
        }
        
        importAccountButton.isEnabled = false
        statusText.text = "Importing account..."
        
        CoroutineScope(Dispatchers.IO).launch {
            try {
                val account = PurrmintNative.importAccount(secretKey)
                
                withContext(Dispatchers.Main) {
                    if (account != null && account.isValid()) {
                        statusText.text = "Account imported successfully!"
                        Toast.makeText(this@LoginActivity, 
                            "Account imported: ${account.getDisplayPubkey()}", 
                            Toast.LENGTH_SHORT).show()
                        
                        // Clear the input
                        secretKeyInput.text.clear()
                        
                        // Navigate to main activity
                        startMainActivity()
                    } else {
                        statusText.text = "Failed to import account"
                        importAccountButton.isEnabled = true
                        Toast.makeText(this@LoginActivity, 
                            "Invalid secret key", 
                            Toast.LENGTH_SHORT).show()
                    }
                }
            } catch (e: Exception) {
                withContext(Dispatchers.Main) {
                    statusText.text = "Error: ${e.message}"
                    importAccountButton.isEnabled = true
                    Toast.makeText(this@LoginActivity, 
                        "Error importing account: ${e.message}", 
                        Toast.LENGTH_SHORT).show()
                }
                Log.e(TAG, "Error importing account", e)
            }
        }
    }
    
    private fun startMainActivity() {
        val intent = Intent(this, MainActivity::class.java)
        startActivity(intent)
        finish() // Close login activity
    }
} 