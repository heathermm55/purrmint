package com.purrmint.app.wallet

import android.content.Context
import android.util.Log
import com.purrmint.app.R
import kotlinx.coroutines.delay
import java.util.*

class MockLightningWalletManager(private val context: Context) {

    private var isInitialized = false
    private var isRunning = false
    private var nodeId: String? = null
    private var lightningBalanceMsat: Long = 0L

    companion object {
        private const val TAG = "MockLightningWalletManager"
    }

    suspend fun initialize(): Result<Unit> {
        return try {
            Log.d(TAG, "Initializing mock Lightning wallet...")
            delay(2000) // Simulate initialization delay
            
            isInitialized = true
            isRunning = true
            nodeId = "02eec7245d6b7d2ccb30380bfbe2a3648cd7a942653f5aa340edcea1f283686619"
            lightningBalanceMsat = 500000L // 500 sats
            
            Log.d(TAG, "Mock Lightning wallet initialized successfully")
            Result.success(Unit)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to initialize mock Lightning wallet", e)
            Result.failure(e)
        }
    }

    fun getWalletStatus(): WalletStatus {
        return WalletStatus(
            isRunning = isRunning,
            nodeId = nodeId,
            isConnected = isRunning
        )
    }

    fun getWalletBalance(): WalletBalance? {
        return if (isInitialized) {
            WalletBalance(
                lightningBalanceMsat = lightningBalanceMsat,
                onchainBalanceSats = 0L
            )
        } else {
            null
        }
    }

    fun getChannels(): List<ChannelInfo> {
        return if (isInitialized) {
            listOf(
                ChannelInfo(
                    channelId = "1234567890abcdef",
                    peerId = "02eec7245d6b7d2ccb30380bfbe2a3648cd7a942653f5aa340edcea1f283686619",
                    capacityMsat = 1000000L, // 1000 sats
                    balanceMsat = 500000L,   // 500 sats
                    status = "OPEN"
                ),
                ChannelInfo(
                    channelId = "abcdef1234567890",
                    peerId = "03a5076b9d3546a113f42e180f40c6f69d34202a6d3e3d3e3d3e3d3e3d3e3d3e3d",
                    capacityMsat = 2000000L, // 2000 sats
                    balanceMsat = 1500000L,  // 1500 sats
                    status = "OPEN"
                )
            )
        } else {
            emptyList()
        }
    }

    fun getRecentPayments(): List<PaymentInfo> {
        return if (isInitialized) {
            val now = System.currentTimeMillis()
            listOf(
                PaymentInfo(
                    paymentId = "payment_1",
                    amountMsat = 100000L, // 100 sats
                    description = context.getString(R.string.payment_description_test),
                    status = "SUCCEEDED",
                    isIncoming = true,
                    timestamp = now - 3600000 // 1 hour ago
                ),
                PaymentInfo(
                    paymentId = "payment_2",
                    amountMsat = 50000L, // 50 sats
                    description = context.getString(R.string.payment_description_coffee),
                    status = "SUCCEEDED",
                    isIncoming = false,
                    timestamp = now - 7200000 // 2 hours ago
                ),
                PaymentInfo(
                    paymentId = "payment_3",
                    amountMsat = 200000L, // 200 sats
                    description = context.getString(R.string.payment_description_lunch),
                    status = "PENDING",
                    isIncoming = false,
                    timestamp = now - 10800000 // 3 hours ago
                )
            )
        } else {
            emptyList()
        }
    }

    fun getPaymentHistory(): List<PaymentInfo> {
        return getRecentPayments()
    }

    suspend fun createInvoice(amountSats: Long, description: String): String {
        delay(1500) // Simulate network delay
        
        if (!isInitialized) {
            throw Exception(context.getString(R.string.error_wallet_not_initialized))
        }
        
        // Generate mock invoice
        val randomId = generateRandomString(20)
        val mockInvoice = "lnbc${amountSats}u1p${randomId}qpp5${generateRandomString(50)}"
        
        Log.d(TAG, "Created mock invoice: $mockInvoice")
        return mockInvoice
    }

    suspend fun payInvoice(invoice: String): String {
        delay(2000) // Simulate payment processing
        
        if (!isInitialized) {
            throw Exception(context.getString(R.string.error_wallet_not_initialized))
        }
        
        if (!invoice.startsWith("lnbc")) {
            throw Exception(context.getString(R.string.error_invalid_invoice_format))
        }
        
        // Generate mock payment ID
        val paymentId = "payment_${generateRandomString(16)}"
        
        Log.d(TAG, "Mock payment sent with ID: $paymentId")
        return paymentId
    }

    suspend fun openChannel(nodeId: String, address: String, amountSats: Long): String {
        delay(3000) // Simulate channel opening
        
        if (!isInitialized) {
            throw Exception(context.getString(R.string.error_wallet_not_initialized))
        }
        
        if (amountSats < 10000) {
            throw Exception(context.getString(R.string.error_min_channel_amount))
        }
        
        // Generate mock channel ID
        val channelId = "channel_${generateRandomString(16)}"
        
        Log.d(TAG, "Opened mock channel: $channelId")
        return channelId
    }

    suspend fun cleanup(): Result<Unit> {
        return try {
            Log.d(TAG, "Cleaning up mock Lightning wallet...")
            delay(1000) // Simulate cleanup delay
            
            isInitialized = false
            isRunning = false
            nodeId = null
            lightningBalanceMsat = 0L
            
            Log.d(TAG, "Mock Lightning wallet cleaned up successfully")
            Result.success(Unit)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to cleanup mock Lightning wallet", e)
            Result.failure(e)
        }
    }

    private fun generateRandomString(length: Int): String {
        val chars = "abcdefghijklmnopqrstuvwxyz0123456789"
        return (1..length)
            .map { chars.random() }
            .joinToString("")
    }
} 