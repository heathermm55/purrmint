package com.example.purrmint

import java.text.SimpleDateFormat
import java.util.*

/**
 * Represents a log entry with message, level, and timestamp
 */
data class LogEntry(
    val message: String,
    val level: LogLevel,
    val timestamp: Long
) {
    /**
     * Get formatted timestamp string
     */
    fun getFormattedTimestamp(): String {
        val dateFormat = SimpleDateFormat("HH:mm:ss", Locale.getDefault())
        return dateFormat.format(Date(timestamp))
    }
    
    /**
     * Get display text for the log entry
     */
    fun getDisplayText(): String {
        return "[${getFormattedTimestamp()}] ${level.emoji} $message"
    }
}

/**
 * Log levels with associated emojis and colors
 */
enum class LogLevel(val emoji: String, val color: Int) {
    INFO("ℹ️", 0xFF2196F3.toInt()),
    SUCCESS("✅", 0xFF4CAF50.toInt()),
    WARNING("⚠️", 0xFFFF9800.toInt()),
    ERROR("❌", 0xFFF44336.toInt())
} 