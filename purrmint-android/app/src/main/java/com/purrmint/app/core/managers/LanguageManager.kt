package com.purrmint.app.core.managers

import android.content.Context
import android.content.SharedPreferences
import android.content.res.Configuration
import android.content.res.Resources
import android.os.Build
import java.util.*

class LanguageManager(private val context: Context) {
    
    companion object {
        private const val PREFS_NAME = "language_prefs"
        private const val KEY_LANGUAGE = "selected_language"
        
        const val LANGUAGE_ENGLISH = "en"
        const val LANGUAGE_CHINESE = "zh"
        const val LANGUAGE_JAPANESE = "ja"
        const val LANGUAGE_PORTUGUESE = "pt"
    }
    
    private val prefs: SharedPreferences = context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
    
    /**
     * Get the currently selected language
     */
    fun getCurrentLanguage(): String {
        return prefs.getString(KEY_LANGUAGE, LANGUAGE_ENGLISH) ?: LANGUAGE_ENGLISH
    }
    
    /**
     * Set the language for the app
     */
    fun setLanguage(languageCode: String) {
        prefs.edit().putString(KEY_LANGUAGE, languageCode).apply()
    }
    
    /**
     * Get available languages
     */
    fun getAvailableLanguages(): List<Language> {
        return listOf(
            Language(LANGUAGE_ENGLISH, "English", "English"),
            Language(LANGUAGE_CHINESE, "中文", "Chinese"),
            Language(LANGUAGE_JAPANESE, "日本語", "Japanese"),
            Language(LANGUAGE_PORTUGUESE, "Português", "Portuguese")
        )
    }
    
    /**
     * Apply the selected language to the context
     */
    fun applyLanguage(context: Context): Context {
        val language = getCurrentLanguage()
        val locale = Locale(language)
        Locale.setDefault(locale)
        
        val config = Configuration(context.resources.configuration)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
            config.setLocale(locale)
        } else {
            @Suppress("DEPRECATION")
            config.locale = locale
        }
        
        return context.createConfigurationContext(config)
    }
    
    /**
     * Update the configuration for the current activity
     */
    fun updateConfiguration(resources: Resources) {
        val language = getCurrentLanguage()
        val locale = Locale(language)
        Locale.setDefault(locale)
        
        val config = Configuration(resources.configuration)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
            config.setLocale(locale)
        } else {
            @Suppress("DEPRECATION")
            config.locale = locale
        }
        
        @Suppress("DEPRECATION")
        resources.updateConfiguration(config, resources.displayMetrics)
    }
    
    /**
     * Get the display name for a language code
     */
    fun getLanguageDisplayName(languageCode: String): String {
        return when (languageCode) {
            LANGUAGE_ENGLISH -> "English"
            LANGUAGE_CHINESE -> "中文"
            LANGUAGE_JAPANESE -> "日本語"
            LANGUAGE_PORTUGUESE -> "Português"
            else -> "English"
        }
    }
    
    /**
     * Check if the current language is RTL (Right-to-Left)
     */
    fun isRTL(): Boolean {
        val language = getCurrentLanguage()
        return language == "ar" || language == "he" || language == "fa"
    }
    
    data class Language(
        val code: String,
        val nativeName: String,
        val englishName: String
    )
} 