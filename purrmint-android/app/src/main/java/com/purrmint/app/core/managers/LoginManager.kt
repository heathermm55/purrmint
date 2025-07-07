package com.purrmint.app.core.managers

import android.content.Context
import android.content.SharedPreferences
import android.util.Log
import org.json.JSONObject
import org.json.JSONException

class LoginManager(private val context: Context) {
    
    companion object {
        private const val TAG = "LoginManager"
        private const val PREFS_NAME = "PurrmintLoginPrefs"
        private const val KEY_IS_LOGGED_IN = "is_logged_in"
        private const val KEY_NSEC_KEY = "nsec_key"
        private const val KEY_ACCOUNT_INFO = "account_info"
        private const val KEY_NPUB_ADDRESS = "npub_address"
        private const val KEY_LOGIN_TIME = "login_time"
    }
    
    private val prefs: SharedPreferences = context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
    private val purrmintManager = PurrmintManager(context)
    
    /**
     * Check if user is currently logged in
     */
    fun isLoggedIn(): Boolean {
        val isLoggedIn = prefs.getBoolean(KEY_IS_LOGGED_IN, false)
        val loginTime = prefs.getLong(KEY_LOGIN_TIME, 0)
        val currentTime = System.currentTimeMillis()
        
        // Check if login is still valid (within 30 days)
        val isValid = (currentTime - loginTime) < (30 * 24 * 60 * 60 * 1000L)
        
        if (isLoggedIn && !isValid) {
            // Login expired, clear it
            Log.i(TAG, "Login expired, clearing login state")
            clearLoginState()
            return false
        }
        
        return isLoggedIn && isValid
    }
    
    /**
     * Create a new Nostr account
     * @return true if account created successfully
     */
    fun createNewAccount(): Boolean {
        return try {
            val accountJson = purrmintManager.createNostrAccount()
            if (accountJson != null) {
                val account = parseAccountJson(accountJson)
                if (account != null) {
                    saveNewAccountState(accountJson, account.nsec, account.npub)
                    Log.i(TAG, "New account created successfully")
                    true
                } else {
                    Log.e(TAG, "Failed to parse account JSON")
                    false
                }
            } else {
                Log.e(TAG, "Failed to create new account")
                false
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error creating new account", e)
            false
        }
    }
    
    /**
     * Login with existing NSEC key
     * @param nsecKey The NSEC key to login with
     * @return true if login successful
     */
    fun loginWithNsec(nsecKey: String): Boolean {
        return try {
            val npub = purrmintManager.convertNsecToNpub(nsecKey)
            if (npub != null) {
                val accountInfo = createAccountInfo(nsecKey, npub)
                saveLoginState(nsecKey, accountInfo, npub)
                Log.i(TAG, "Login successful with NSEC")
                true
            } else {
                Log.e(TAG, "Failed to convert NSEC to NPUB")
                false
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error logging in with NSEC", e)
            false
        }
    }
    
    /**
     * Save login state with NSEC key
     */
    fun saveLoginState(nsecKey: String, accountInfo: String? = null, npubAddress: String? = null) {
        prefs.edit().apply {
            putBoolean(KEY_IS_LOGGED_IN, true)
            putString(KEY_NSEC_KEY, nsecKey)
            putString(KEY_ACCOUNT_INFO, accountInfo)
            putString(KEY_NPUB_ADDRESS, npubAddress)
            putLong(KEY_LOGIN_TIME, System.currentTimeMillis())
        }.apply()
        
        Log.i(TAG, "Login state saved")
    }
    
    /**
     * Save login state for new account creation
     */
    fun saveNewAccountState(accountInfo: String, nsecKey: String? = null, npubAddress: String? = null) {
        prefs.edit().apply {
            putBoolean(KEY_IS_LOGGED_IN, true)
            putString(KEY_ACCOUNT_INFO, accountInfo)
            if (nsecKey != null) {
                putString(KEY_NSEC_KEY, nsecKey)
            }
            if (npubAddress != null) {
                putString(KEY_NPUB_ADDRESS, npubAddress)
            }
            putLong(KEY_LOGIN_TIME, System.currentTimeMillis())
        }.apply()
        
        Log.i(TAG, "New account state saved")
    }
    
    /**
     * Get saved NSEC key
     */
    fun getNsecKey(): String? {
        return prefs.getString(KEY_NSEC_KEY, null)
    }
    
    /**
     * Get saved account info
     */
    fun getAccountInfo(): String? {
        return prefs.getString(KEY_ACCOUNT_INFO, null)
    }
    
    /**
     * Get saved npub address
     */
    fun getNpubAddress(): String? {
        return prefs.getString(KEY_NPUB_ADDRESS, null)
    }
    
    /**
     * Clear login state
     */
    fun clearLoginState() {
        prefs.edit().apply {
            remove(KEY_IS_LOGGED_IN)
            remove(KEY_NSEC_KEY)
            remove(KEY_ACCOUNT_INFO)
            remove(KEY_NPUB_ADDRESS)
            remove(KEY_LOGIN_TIME)
        }.apply()
        
        Log.i(TAG, "Login state cleared")
    }
    
    /**
     * Get login time
     */
    fun getLoginTime(): Long {
        return prefs.getLong(KEY_LOGIN_TIME, 0)
    }
    
    /**
     * Get current account from file or create new one
     * @return Account JSON string or null if failed
     */
    fun getCurrentOrCreateAccount(): String? {
        return try {
            val currentAccount = purrmintManager.getCurrentAccount()
            if (currentAccount != "{\"account\":\"none\"}") {
                currentAccount
            } else {
                // No account exists, create new one
                createNewAccount()
                purrmintManager.getCurrentAccount()
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error getting or creating account", e)
            null
        }
    }
    
    /**
     * Parse account JSON to extract nsec and npub
     */
    private fun parseAccountJson(accountJson: String): AccountInfo? {
        return try {
            val jsonObject = JSONObject(accountJson)
            val nsec = jsonObject.optString("nsec", "")
            val npub = jsonObject.optString("npub", "")
            
            if (nsec.isNotEmpty() && npub.isNotEmpty()) {
                AccountInfo(nsec, npub)
            } else {
                Log.e(TAG, "Account JSON missing nsec or npub")
                null
            }
        } catch (e: JSONException) {
            Log.e(TAG, "Error parsing account JSON", e)
            null
        }
    }
    
    /**
     * Create account info JSON string
     */
    private fun createAccountInfo(nsec: String, npub: String): String {
        return try {
            val json = JSONObject()
            json.put("nsec", nsec)
            json.put("npub", npub)
            json.put("created_at", System.currentTimeMillis())
            json.toString()
        } catch (e: JSONException) {
            Log.e(TAG, "Error creating account info", e)
            "{\"nsec\":\"$nsec\",\"npub\":\"$npub\"}"
        }
    }
    
    /**
     * Data class to hold account information
     */
    private data class AccountInfo(
        val nsec: String,
        val npub: String
    )
} 