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
import com.purrmint.app.core.managers.LanguageManager
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
        
        // Apply current language
        val languageManager = LanguageManager(this)
        languageManager.updateConfiguration(resources)
        
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
                        npubInput.setText(getString(R.string.npub_not_available))
                    }
                    
                    if (nsec.isNotEmpty()) {
                        nsecInput.setText(nsec)
                    } else {
                        nsecInput.setText(getString(R.string.nsec_not_available))
                    }
                } catch (e: Exception) {
                    // If not JSON, treat as simple string
                    npubInput.setText(accountInfo)
                    nsecInput.setText(getString(R.string.nsec_not_available))
                }
            } else {
                npubInput.setText(getString(R.string.no_account_info_available))
                nsecInput.setText(getString(R.string.no_account_info_available))
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error loading account info", e)
            npubInput.setText(getString(R.string.error_loading_account_info))
            nsecInput.setText(getString(R.string.error_loading_account_info))
        }
    }
    
    private fun copyToClipboard(label: String, text: String) {
        try {
            val clipboard = getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
            val clip = ClipData.newPlainText(label, text)
            clipboard.setPrimaryClip(clip)
            Toast.makeText(this, getString(R.string.copied_to_clipboard, label), Toast.LENGTH_SHORT).show()
        } catch (e: Exception) {
            Log.e(TAG, "Error copying to clipboard", e)
            Toast.makeText(this, getString(R.string.failed_to_copy, label), Toast.LENGTH_SHORT).show()
        }
    }
    
    private fun showLogoutDialog() {
        AlertDialog.Builder(this)
            .setTitle(getString(R.string.logout_title))
            .setMessage(getString(R.string.logout_message))
            .setPositiveButton(getString(R.string.logout_button)) { _, _ ->
                logout()
            }
            .setNegativeButton(getString(R.string.cancel), null)
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
            Toast.makeText(this, getString(R.string.logged_out_successfully), Toast.LENGTH_SHORT).show()
            // Start LoginActivity and clear back stack
            val loginIntent = Intent(this, LoginActivity::class.java)
            loginIntent.flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TASK
            startActivity(loginIntent)
            // Finish AccountActivity to prevent returning here
            finish()
        } catch (e: Exception) {
            Log.e(TAG, "Error during logout", e)
            Toast.makeText(this, getString(R.string.error_during_logout, e.message), Toast.LENGTH_SHORT).show()
        }
    }
} 