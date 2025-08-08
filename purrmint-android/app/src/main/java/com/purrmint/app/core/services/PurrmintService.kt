package com.purrmint.app.core.services

import android.app.*
import android.content.Intent
import android.os.Binder
import android.os.Build
import android.os.IBinder
import android.util.Log
import androidx.core.app.NotificationCompat
import com.purrmint.app.R
import com.purrmint.app.core.managers.PurrmintManager
import com.purrmint.app.ui.activities.MainActivity
import java.util.concurrent.atomic.AtomicBoolean

class PurrmintService : Service() {
    companion object {
        private const val TAG = "PurrmintService"
        private const val NOTIFICATION_ID = 1001
        private const val CHANNEL_ID = "purrmint_service_channel"
    }

    private val binder = LocalBinder()
    private lateinit var purrmintManager: PurrmintManager
    private val isStarting = AtomicBoolean(false)
    private val isStopping = AtomicBoolean(false)

    inner class LocalBinder : Binder() {
        fun getService(): PurrmintService = this@PurrmintService
        fun getPurrmintManager(): PurrmintManager = purrmintManager
    }

    override fun onCreate() {
        super.onCreate()
        Log.i(TAG, "PurrmintService created")
        purrmintManager = PurrmintManager(this)
        createNotificationChannel()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        Log.i(TAG, "PurrmintService started")
        
        // Start foreground service with minimal notification
        startForeground(NOTIFICATION_ID, createMinimalNotification())
        
        // Start mint service in background thread with thread safety
        if (!isStarting.get() && !isStopping.get()) {
            isStarting.set(true)
            Thread {
                try {
                    Log.i(TAG, "Starting mint service in background thread...")
                    val success = purrmintManager.startMintServiceWithSavedNsec()
                    if (success) {
                        Log.i(TAG, "Mint service started successfully")
                    } else {
                        Log.e(TAG, "Failed to start mint service")
                    }
                } catch (e: Exception) {
                    Log.e(TAG, "Failed to start mint service", e)
                } finally {
                    isStarting.set(false)
                }
            }.start()
        } else {
            Log.w(TAG, "Service operation already in progress, skipping start")
        }
        
        return START_STICKY // Restart service if killed
    }

    override fun onBind(intent: Intent?): IBinder {
        return binder
    }

    override fun onDestroy() {
        super.onDestroy()
        Log.i(TAG, "PurrmintService destroyed")
        
        // Stop mint service with thread safety
        if (!isStopping.get()) {
            isStopping.set(true)
            try {
                purrmintManager.stopMintService()
                Log.i(TAG, "Mint service stopped successfully")
            } catch (e: Exception) {
                Log.e(TAG, "Failed to stop mint service", e)
            } finally {
                isStopping.set(false)
            }
        }
    }

    /**
     * Create notification channel for Android 8.0+
     */
    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "PurrMint Service",
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = "PurrMint background service"
                setShowBadge(false)
            }
            
            val notificationManager = getSystemService(NotificationManager::class.java)
            notificationManager.createNotificationChannel(channel)
        }
    }

    /**
     * Create minimal notification for foreground service
     */
    private fun createMinimalNotification(): Notification {
        val intent = Intent(this, MainActivity::class.java).apply {
            flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TASK
        }
        
        val pendingIntent = PendingIntent.getActivity(
            this, 0, intent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )
        
        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle("PurrMint")
            .setContentText("Mint service is running")
            .setSmallIcon(android.R.drawable.ic_dialog_info)
            .setContentIntent(pendingIntent)
            .setOngoing(true)
            .setSilent(true)
            .build()
    }
} 