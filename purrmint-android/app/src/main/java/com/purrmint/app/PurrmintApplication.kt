package com.purrmint.app

import android.app.Application
import android.content.Context
import com.purrmint.app.core.managers.LanguageManager

class PurrmintApplication : Application() {
    
    override fun attachBaseContext(base: Context) {
        val languageManager = LanguageManager(base)
        val context = languageManager.applyLanguageToBaseContext(base)
        super.attachBaseContext(context)
    }
    
    override fun onCreate() {
        super.onCreate()
        
        // Apply language configuration
        val languageManager = LanguageManager(this)
        languageManager.updateConfiguration(resources)
    }
    
    // Remove the problematic getResources override
} 