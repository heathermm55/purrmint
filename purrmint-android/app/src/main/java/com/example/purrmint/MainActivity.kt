package com.example.purrmint

import android.content.Intent
import android.os.Bundle
import android.util.Log
import android.view.Menu
import android.view.MenuItem
import android.widget.Button
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity
import androidx.recyclerview.widget.LinearLayoutManager
import androidx.recyclerview.widget.RecyclerView
import com.google.android.material.bottomnavigation.BottomNavigationView
import com.google.android.material.floatingactionbutton.FloatingActionButton
import org.json.JSONObject
import kotlinx.coroutines.*

class MainActivity : AppCompatActivity() {
    
    private lateinit var accountInfoText: TextView
    private lateinit var serviceStatusText: TextView
    private lateinit var startServiceButton: Button
    private lateinit var stopServiceButton: Button
    private lateinit var pauseServiceButton: Button
    private lateinit var bottomNavigation: BottomNavigationView
    private lateinit var logsRecyclerView: RecyclerView
    private lateinit var logsAdapter: LogsAdapter
    
    private val TAG = "MainActivity"
    private val logs = mutableListOf<LogEntry>()
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)
        
        initializeViews()
        setupBottomNavigation()
        setupButtons()
        setupLogsRecyclerView()
        
        // Load initial data
        loadAccountInfo()
        loadServiceStatus()
    }
    
    private fun initializeViews() {
        accountInfoText = findViewById(R.id.accountInfoText)
        serviceStatusText = findViewById(R.id.serviceStatusText)
        startServiceButton = findViewById(R.id.startServiceButton)
        stopServiceButton = findViewById(R.id.stopServiceButton)
        pauseServiceButton = findViewById(R.id.pauseServiceButton)
        bottomNavigation = findViewById(R.id.bottomNavigation)
        logsRecyclerView = findViewById(R.id.logsRecyclerView)
    }
    
    private fun setupBottomNavigation() {
        bottomNavigation.setOnItemSelectedListener { item ->
            when (item.itemId) {
                R.id.nav_dashboard -> {
                    showDashboard()
                    true
                }
                R.id.nav_config -> {
                    showConfiguration()
                    true
                }
                R.id.nav_logs -> {
                    showLogs()
                    true
                }
                else -> false
            }
        }
    }
    
    private fun setupButtons() {
        startServiceButton.setOnClickListener { startMintService() }
        stopServiceButton.setOnClickListener { stopMintService() }
        pauseServiceButton.setOnClickListener { pauseMintService() }
        
        // Setup clear logs button
        findViewById<Button>(R.id.clearLogsButton).setOnClickListener { clearLogs() }
    }
    
    private fun setupLogsRecyclerView() {
        logsAdapter = LogsAdapter(logs)
        logsRecyclerView.layoutManager = LinearLayoutManager(this)
        logsRecyclerView.adapter = logsAdapter
    }
    
    private fun loadAccountInfo() {
        CoroutineScope(Dispatchers.IO).launch {
            try {
                val accountInfo = PurrmintNative.getCurrentAccount()
                val json = PurrmintNative.parseJsonResponse(accountInfo)
                
                withContext(Dispatchers.Main) {
                    if (json != null && json.has("pubkey")) {
                        val pubkey = json.optString("pubkey", "")
                        val isImported = json.optBoolean("is_imported", false)
                        accountInfoText.text = "Account: ${pubkey.take(8)}...${pubkey.takeLast(8)}\nType: ${if (isImported) "Imported" else "New"}"
                        addLog("Account info loaded", LogLevel.INFO)
                    } else {
                        accountInfoText.text = "No account found"
                        addLog("No account found", LogLevel.WARNING)
                    }
                }
            } catch (e: Exception) {
                withContext(Dispatchers.Main) {
                    accountInfoText.text = "Error loading account info"
                    addLog("Error loading account: ${e.message}", LogLevel.ERROR)
                }
                Log.e(TAG, "Error loading account info", e)
            }
        }
    }
    
    private fun loadServiceStatus() {
        CoroutineScope(Dispatchers.IO).launch {
            try {
                val status = PurrmintNative.getMintStatus()
                val json = PurrmintNative.parseJsonResponse(status)
                
                withContext(Dispatchers.Main) {
                    if (json != null) {
                        val running = json.optBoolean("running", false)
                        val uptime = json.optLong("uptime", 0)
                        val totalRequests = json.optInt("total_requests", 0)
                        
                        serviceStatusText.text = "Status: ${if (running) "Running" else "Stopped"}\nUptime: ${uptime}s\nRequests: $totalRequests"
                        
                        // Update button states
                        startServiceButton.isEnabled = !running
                        stopServiceButton.isEnabled = running
                        pauseServiceButton.isEnabled = running
                        
                        addLog("Service status loaded", LogLevel.INFO)
                    } else {
                        serviceStatusText.text = "Status: Unknown"
                        addLog("Failed to load service status", LogLevel.ERROR)
                    }
                }
            } catch (e: Exception) {
                withContext(Dispatchers.Main) {
                    serviceStatusText.text = "Status: Error"
                    addLog("Error loading status: ${e.message}", LogLevel.ERROR)
                }
                Log.e(TAG, "Error loading service status", e)
            }
        }
    }
    
    private fun startMintService() {
        CoroutineScope(Dispatchers.IO).launch {
            try {
                val result = PurrmintNative.startMint()
                val error = PurrmintNative.FfiError.fromCode(result)
                
                withContext(Dispatchers.Main) {
                    if (error == PurrmintNative.FfiError.SUCCESS) {
                        addLog("Service started successfully", LogLevel.SUCCESS)
                        loadServiceStatus() // Refresh status
                    } else {
                        addLog("Failed to start service: ${error?.name ?: "Unknown error"}", LogLevel.ERROR)
                    }
                }
            } catch (e: Exception) {
                withContext(Dispatchers.Main) {
                    addLog("Error starting service: ${e.message}", LogLevel.ERROR)
                }
                Log.e(TAG, "Error starting service", e)
            }
        }
    }
    
    private fun stopMintService() {
        CoroutineScope(Dispatchers.IO).launch {
            try {
                val result = PurrmintNative.stopMint()
                val error = PurrmintNative.FfiError.fromCode(result)
                
                withContext(Dispatchers.Main) {
                    if (error == PurrmintNative.FfiError.SUCCESS) {
                        addLog("Service stopped successfully", LogLevel.SUCCESS)
                        loadServiceStatus() // Refresh status
                    } else {
                        addLog("Failed to stop service: ${error?.name ?: "Unknown error"}", LogLevel.ERROR)
                    }
                }
            } catch (e: Exception) {
                withContext(Dispatchers.Main) {
                    addLog("Error stopping service: ${e.message}", LogLevel.ERROR)
                }
                Log.e(TAG, "Error stopping service", e)
            }
        }
    }
    
    private fun pauseMintService() {
        // TODO: Implement pause functionality
        addLog("Pause functionality not yet implemented", LogLevel.WARNING)
    }
    
    private fun clearLogs() {
        logs.clear()
        logsAdapter.notifyDataSetChanged()
        addLog("Logs cleared", LogLevel.INFO)
    }
    
    private fun showDashboard() {
        // Show dashboard content
        findViewById<android.view.View>(R.id.dashboardLayout).visibility = android.view.View.VISIBLE
        findViewById<android.view.View>(R.id.configLayout).visibility = android.view.View.GONE
        findViewById<android.view.View>(R.id.logsLayout).visibility = android.view.View.GONE
    }
    
    private fun showConfiguration() {
        // Launch configuration activity
        val intent = Intent(this, ConfigurationActivity::class.java)
        startActivity(intent)
    }
    
    private fun showLogs() {
        // Show logs content
        findViewById<android.view.View>(R.id.dashboardLayout).visibility = android.view.View.GONE
        findViewById<android.view.View>(R.id.configLayout).visibility = android.view.View.GONE
        findViewById<android.view.View>(R.id.logsLayout).visibility = android.view.View.VISIBLE
    }
    
    private fun addLog(message: String, level: LogLevel) {
        val logEntry = LogEntry(message, level, System.currentTimeMillis())
        logs.add(0, logEntry) // Add to beginning
        
        // Keep only last 100 logs
        if (logs.size > 100) {
            logs.removeAt(logs.size - 1)
        }
        
        logsAdapter.notifyDataSetChanged()
    }
    
    override fun onCreateOptionsMenu(menu: Menu?): Boolean {
        menuInflater.inflate(R.menu.main_menu, menu)
        return true
    }
    
    override fun onOptionsItemSelected(item: MenuItem): Boolean {
        return when (item.itemId) {
            R.id.action_logout -> {
                // TODO: Implement logout functionality
                addLog("Logout requested", LogLevel.INFO)
                true
            }
            else -> super.onOptionsItemSelected(item)
        }
    }
} 