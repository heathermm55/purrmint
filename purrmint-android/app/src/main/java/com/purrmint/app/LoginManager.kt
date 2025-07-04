package com.purrmint.app

import android.content.Context
import android.content.SharedPreferences
import android.util.Log

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
    fun saveNewAccountState(accountInfo: String, npubAddress: String? = null) {
        prefs.edit().apply {
            putBoolean(KEY_IS_LOGGED_IN, true)
            putString(KEY_ACCOUNT_INFO, accountInfo)
            putString(KEY_NPUB_ADDRESS, npubAddress)
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
} 