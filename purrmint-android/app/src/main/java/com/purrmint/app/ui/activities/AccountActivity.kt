package com.purrmint.app.ui.activities

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.Intent
import android.os.Bundle
import android.util.Log
import android.widget.Toast
import androidx.appcompat.app.AlertDialog
import androidx.appcompat.app.AppCompatActivity
import androidx.appcompat.widget.Toolbar
import com.google.android.material.appbar.MaterialToolbar
import com.google.android.material.button.MaterialButton
import com.google.android.material.textfield.TextInputEditText
import com.purrmint.app.R
import com.purrmint.app.core.managers.LoginManager
import com.purrmint.app.core.services.PurrmintService
import org.json.JSONObject

class AccountActivity : AppCompatActivity() {
    
    private lateinit var npubInput: TextInputEditText
    private lateinit var nsecInput: TextInputEditText
    private lateinit var btnCopyNpub: MaterialButton
    private lateinit var btnCopyNsec: MaterialButton
    private lateinit var btnLogout: MaterialButton
    private lateinit var toolbar: MaterialToolbar
    
    private lateinit var loginManager: LoginManager
    
    companion object {
        private const val TAG = "AccountActivity"
    }
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_account)
        
        // Initialize login manager
        loginManager = LoginManager(this)
        
        initializeViews()
        setupClickListeners()
        loadAccountInfo()
    }
    
    private fun initializeViews() {
        npubInput = findViewById(R.id.npubInput)
        nsecInput = findViewById(R.id.nsecInput)
        btnCopyNpub = findViewById(R.id.btnCopyNpub)
        btnCopyNsec = findViewById(R.id.btnCopyNsec)
        btnLogout = findViewById(R.id.btnLogout)
        toolbar = findViewById(R.id.topAppBar)
    }
    
    private fun setupClickListeners() {
        toolbar.setNavigationOnClickListener {
            finish()
        }
        
        btnCopyNpub.setOnClickListener {
            copyToClipboard("NPUB", npubInput.text.toString())
        }
        
        btnCopyNsec.setOnClickListener {
            copyToClipboard("NSEC", nsecInput.text.toString())
        }
        
        btnLogout.setOnClickListener {
            showLogoutDialog()
        }
    }
    
    private fun loadAccountInfo() {
        try {
            val accountInfo = loginManager.getAccountInfo()
            if (accountInfo != null) {
                try {
                    // Try to parse as JSON
                    val json = JSONObject(accountInfo)
                    val npub = json.optString("npub", "")
                    val nsec = json.optString("nsec", "")
                    
                    if (npub.isNotEmpty()) {
                        npubInput.setText(npub)
                    } else {
                        npubInput.setText("NPUB not available")
                    }
                    
                    if (nsec.isNotEmpty()) {
                        nsecInput.setText(nsec)
                    } else {
                        nsecInput.setText("NSEC not available")
                    }
                } catch (e: Exception) {
                    // If not JSON, treat as simple string
                    npubInput.setText(accountInfo)
                    nsecInput.setText("NSEC not available")
                }
            } else {
                npubInput.setText("No account info available")
                nsecInput.setText("No account info available")
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error loading account info", e)
            npubInput.setText("Error loading account info")
            nsecInput.setText("Error loading account info")
        }
    }
    
    private fun copyToClipboard(label: String, text: String) {
        try {
            val clipboard = getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
            val clip = ClipData.newPlainText(label, text)
            clipboard.setPrimaryClip(clip)
            Toast.makeText(this, "$label copied to clipboard", Toast.LENGTH_SHORT).show()
        } catch (e: Exception) {
            Log.e(TAG, "Error copying to clipboard", e)
            Toast.makeText(this, "Failed to copy $label", Toast.LENGTH_SHORT).show()
        }
    }
    
    private fun showLogoutDialog() {
        AlertDialog.Builder(this)
            .setTitle("Logout")
            .setMessage("Are you sure you want to logout? This will stop the mint service and clear your account information.")
            .setPositiveButton("Logout") { _, _ ->
                logout()
            }
            .setNegativeButton("Cancel", null)
            .show()
    }
    
    private fun logout() {
        try {
            // Clear login state
            loginManager.clearLoginState()
            // Stop mint service if running
            val intent = Intent(this, PurrmintService::class.java)
            stopService(intent)
            // Show confirmation toast
            Toast.makeText(this, "Logged out successfully", Toast.LENGTH_SHORT).show()
            // Start LoginActivity and clear back stack
            val loginIntent = Intent(this, LoginActivity::class.java)
            loginIntent.flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TASK
            startActivity(loginIntent)
            // Finish AccountActivity to prevent returning here
            finish()
        } catch (e: Exception) {
            Log.e(TAG, "Error during logout", e)
            Toast.makeText(this, "Error during logout: ${e.message}", Toast.LENGTH_SHORT).show()
        }
    }
} 