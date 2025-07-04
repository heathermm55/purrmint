package com.purrmint.app

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import androidx.fragment.app.Fragment
import com.google.android.material.button.MaterialButton

class ServiceLogsFragment : Fragment() {
    
    private lateinit var logsText: TextView
    private lateinit var clearLogsButton: MaterialButton
    
    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View? {
        return inflater.inflate(R.layout.fragment_service_logs, container, false)
    }
    
    override fun onViewCreated(view: View, savedInstanceState: Bundle?) {
        super.onViewCreated(view, savedInstanceState)
        
        initializeViews(view)
        setupClickListeners()
    }
    
    private fun initializeViews(view: View) {
        logsText = view.findViewById(R.id.logsText)
        clearLogsButton = view.findViewById(R.id.clearLogsButton)
    }
    
    private fun setupClickListeners() {
        clearLogsButton.setOnClickListener {
            (activity as? MainActivity)?.clearLogs()
        }
    }
    
    fun appendLog(message: String) {
        val timestamp = java.text.SimpleDateFormat("HH:mm:ss", java.util.Locale.getDefault()).format(java.util.Date())
        val logEntry = "[$timestamp] $message\n"
        
        activity?.runOnUiThread {
            logsText.append(logEntry)
            logsText.layout?.let {
                val scrollAmount = it.getLineTop(logsText.lineCount) - logsText.height
                if (scrollAmount > 0) {
                    logsText.scrollTo(0, scrollAmount)
                }
            }
        }
    }
    
    fun clearLogs() {
        activity?.runOnUiThread {
            logsText.text = ""
            appendLog("Logs cleared")
        }
    }
} 