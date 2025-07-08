package com.purrmint.app.ui.activities

import android.content.Intent
import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity
import androidx.recyclerview.widget.LinearLayoutManager
import androidx.recyclerview.widget.RecyclerView
import com.google.android.material.appbar.MaterialToolbar
import com.purrmint.app.R
import com.purrmint.app.core.managers.LanguageManager

class LanguageSettingsActivity : AppCompatActivity() {
    
    private lateinit var languageManager: LanguageManager
    private lateinit var recyclerView: RecyclerView
    private lateinit var toolbar: MaterialToolbar
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        // Initialize language manager
        languageManager = LanguageManager(this)
        
        // Apply current language
        languageManager.applyLanguage(this)
        setContentView(R.layout.activity_language_settings)
        
        initializeViews()
        setupRecyclerView()
    }
    
    private fun initializeViews() {
        recyclerView = findViewById(R.id.languageRecyclerView)
        toolbar = findViewById(R.id.topAppBar)
        
        toolbar.setNavigationOnClickListener {
            finish()
        }
    }
    
    private fun setupRecyclerView() {
        recyclerView.layoutManager = LinearLayoutManager(this)
        recyclerView.adapter = LanguageAdapter(languageManager.getAvailableLanguages()) { languageCode ->
            onLanguageSelected(languageCode)
        }
    }
    
    private fun onLanguageSelected(languageCode: String) {
        languageManager.setLanguage(languageCode)
        
        // Restart the app to apply the new language
        val intent = Intent(this, MainActivity::class.java)
        intent.flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TASK
        startActivity(intent)
        finish()
    }
    
    private class LanguageAdapter(
        private val languages: List<LanguageManager.Language>,
        private val onLanguageSelected: (String) -> Unit
    ) : RecyclerView.Adapter<LanguageAdapter.LanguageViewHolder>() {
        
        override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): LanguageViewHolder {
            val view = LayoutInflater.from(parent.context)
                .inflate(R.layout.item_language, parent, false)
            return LanguageViewHolder(view)
        }
        
        override fun onBindViewHolder(holder: LanguageViewHolder, position: Int) {
            val language = languages[position]
            holder.bind(language)
            holder.itemView.setOnClickListener {
                onLanguageSelected(language.code)
            }
        }
        
        override fun getItemCount(): Int = languages.size
        
        class LanguageViewHolder(itemView: View) : RecyclerView.ViewHolder(itemView) {
            private val nativeNameText: TextView = itemView.findViewById(R.id.nativeNameText)
            private val englishNameText: TextView = itemView.findViewById(R.id.englishNameText)
            private val checkIcon: View = itemView.findViewById(R.id.checkIcon)
            
            fun bind(language: LanguageManager.Language) {
                nativeNameText.text = language.nativeName
                englishNameText.text = language.englishName
                
                // Show check icon for current language
                val currentLanguage = LanguageManager(itemView.context).getCurrentLanguage()
                checkIcon.visibility = if (language.code == currentLanguage) View.VISIBLE else View.GONE
            }
        }
    }
} 