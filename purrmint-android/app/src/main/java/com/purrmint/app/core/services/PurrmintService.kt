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

class PurrmintService : Service() {
    companion object {
        private const val TAG = "PurrmintService"
        private const val NOTIFICATION_ID = 1001
        private const val CHANNEL_ID = "purrmint_service_channel"
    }

    private val binder = LocalBinder()
    private lateinit var purrmintManager: PurrmintManager

    inner class LocalBinder : Binder() {
        fun getService(): PurrmintService = this@PurrmintService
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
        
        // Start mint service in background thread
        Thread {
            try {
                purrmintManager.startMintService()
                Log.i(TAG, "Mint service started successfully")
            } catch (e: Exception) {
                Log.e(TAG, "Failed to start mint service", e)
            }
        }.start()
        
        return START_STICKY // Restart service if killed
    }

    override fun onBind(intent: Intent?): IBinder {
        return binder
    }

    override fun onDestroy() {
        super.onDestroy()
        Log.i(TAG, "PurrmintService destroyed")
        purrmintManager.stopMintService()
    }



    fun getPurrmintManager(): PurrmintManager {
        return purrmintManager
    }
    
    private fun createNotificationChannel() {
        // Only create notification channel for API 26+ (Android 8.0+)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "PurrMint Service",
                NotificationManager.IMPORTANCE_MIN
            ).apply {
                description = "Keeps PurrMint service running in background"
                setShowBadge(false)
                enableLights(false)
                enableVibration(false)
                setSound(null, null)
            }
            
            val notificationManager = getSystemService(NotificationManager::class.java)
            notificationManager.createNotificationChannel(channel)
        }
    }
    
    private fun createMinimalNotification(): Notification {
        val intent = Intent(this, MainActivity::class.java)
        val pendingIntent = PendingIntent.getActivity(
            this, 0, intent,
            PendingIntent.FLAG_IMMUTABLE
        )

        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle("PurrMint")
            .setContentText("Mint service running")
            .setSmallIcon(android.R.drawable.ic_dialog_info)
            .setContentIntent(pendingIntent)
            .setOngoing(true)
            .setSilent(true)
            .setPriority(NotificationCompat.PRIORITY_MIN)
            .setCategory(NotificationCompat.CATEGORY_SERVICE)
            .setVisibility(NotificationCompat.VISIBILITY_SECRET)
            .build()
    }
} 