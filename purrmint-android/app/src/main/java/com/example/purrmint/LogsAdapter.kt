package com.example.purrmint

import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import androidx.recyclerview.widget.RecyclerView

/**
 * Adapter for displaying log entries in a RecyclerView
 */
class LogsAdapter(private val logs: List<LogEntry>) : RecyclerView.Adapter<LogsAdapter.LogViewHolder>() {
    
    class LogViewHolder(itemView: View) : RecyclerView.ViewHolder(itemView) {
        val logText: TextView = itemView.findViewById(R.id.logText)
    }
    
    override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): LogViewHolder {
        val view = LayoutInflater.from(parent.context)
            .inflate(R.layout.item_log, parent, false)
        return LogViewHolder(view)
    }
    
    override fun onBindViewHolder(holder: LogViewHolder, position: Int) {
        val logEntry = logs[position]
        holder.logText.text = logEntry.getDisplayText()
        holder.logText.setTextColor(logEntry.level.color)
    }
    
    override fun getItemCount(): Int = logs.size
} 