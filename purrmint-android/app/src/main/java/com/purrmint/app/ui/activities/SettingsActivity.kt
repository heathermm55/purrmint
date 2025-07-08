package com.purrmint.app.ui.activities

import android.content.Intent
import android.os.Bundle
import android.widget.Button
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity
import com.purrmint.app.R

class SettingsActivity : AppCompatActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_settings)

        val btnAccount = findViewById<Button>(R.id.btnAccount)
        val btnLanguage = findViewById<Button>(R.id.btnLanguage)
        val txtVersion = findViewById<TextView>(R.id.txtVersion)

        btnAccount.setOnClickListener {
            startActivity(Intent(this, AccountActivity::class.java))
        }
        btnLanguage.setOnClickListener {
            startActivity(Intent(this, LanguageSettingsActivity::class.java))
        }
        // 获取版本号
        val versionName = packageManager.getPackageInfo(packageName, 0).versionName
        txtVersion.text = getString(R.string.current_version, versionName)
    }
} 