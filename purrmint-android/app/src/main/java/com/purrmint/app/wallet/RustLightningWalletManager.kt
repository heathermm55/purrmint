package com.purrmint.app.wallet

import android.content.Context
import android.util.Log
import com.purrmint.app.R
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.File

/**
 * Rust Lightning Wallet Manager
 * Integrates with Rust LDK Node implementation via JNI
 */
class RustLightningWalletManager(private val context: Context) {

    private var isInitialized = false

    companion object {
        private const val TAG = "RustLightningWalletManager"
        
        init {
            try {
                System.loadLibrary("purrmint")
                Log.d(TAG, "PurrMint native library loaded successfully")
            } catch (e: UnsatisfiedLinkError) {
                Log.e(TAG, "Failed to load PurrMint native library", e)
            }
        }
    }

    suspend fun initialize(): Result<Unit> {
        return try {
            Log.d(TAG, "Initializing Rust Lightning wallet...")
            
            withContext(Dispatchers.IO) {
                // Create storage directory
                val storageDir = File(context.filesDir, "lightning_wallet")
                if (!storageDir.exists()) {
                    storageDir.mkdirs()
                }
                
                // Initialize via JNI
                val success = nativeInitialize(storageDir.absolutePath)
                if (!success) {
                    throw Exception("Failed to initialize Lightning wallet via JNI")
                }
                
                isInitialized = true
                Log.d(TAG, "Rust Lightning wallet initialized successfully")
            }
            
            Result.success(Unit)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to initialize Rust Lightning wallet", e)
            Result.failure(e)
        }
    }

    fun getWalletStatus(): WalletStatus {
        return if (isInitialized) {
            try {
                val status = nativeGetStatus()
                if (status != null) {
                    status
                } else {
                    WalletStatus(
                        isRunning = false,
                        nodeId = null,
                        isConnected = false
                    )
                }
            } catch (e: Exception) {
                Log.e(TAG, "Failed to get wallet status", e)
                WalletStatus(
                    isRunning = false,
                    nodeId = null,
                    isConnected = false
                )
            }
        } else {
            WalletStatus(
                isRunning = false,
                nodeId = null,
                isConnected = false
            )
        }
    }

    fun getWalletBalance(): WalletBalance? {
        return if (isInitialized) {
            try {
                val balance = nativeGetBalance()
                balance
            } catch (e: Exception) {
                Log.e(TAG, "Failed to get wallet balance", e)
                null
            }
        } else {
            null
        }
    }

    fun getChannels(): List<ChannelInfo> {
        return if (isInitialized) {
            try {
                val channels = nativeGetChannels()
                channels?.toList() ?: emptyList()
            } catch (e: Exception) {
                Log.e(TAG, "Failed to get channels", e)
                emptyList()
            }
        } else {
            emptyList()
        }
    }

    fun getRecentPayments(): List<PaymentInfo> {
        return if (isInitialized) {
            try {
                val payments = nativeGetPayments()
                payments?.toList() ?: emptyList()
            } catch (e: Exception) {
                Log.e(TAG, "Failed to get payments", e)
                emptyList()
            }
        } else {
            emptyList()
        }
    }

    fun getPaymentHistory(): List<PaymentInfo> {
        return getRecentPayments()
    }

    suspend fun createInvoice(amountSats: Long, description: String): String {
        return withContext(Dispatchers.IO) {
            if (!isInitialized) {
                throw Exception(context.getString(R.string.error_wallet_not_initialized))
            }
            
            try {
                val invoice = nativeCreateInvoice(amountSats, description)
                if (invoice != null) {
                    Log.d(TAG, "Created invoice: $invoice")
                    invoice
                } else {
                    throw Exception("Failed to create invoice")
                }
            } catch (e: Exception) {
                Log.e(TAG, "Failed to create invoice", e)
                throw e
            }
        }
    }

    suspend fun payInvoice(invoice: String): String {
        return withContext(Dispatchers.IO) {
            if (!isInitialized) {
                throw Exception(context.getString(R.string.error_wallet_not_initialized))
            }
            
            if (!invoice.startsWith("lnbc")) {
                throw Exception(context.getString(R.string.error_invalid_invoice_format))
            }
            
            try {
                val paymentId = nativePayInvoice(invoice)
                if (paymentId != null) {
                    Log.d(TAG, "Payment sent with ID: $paymentId")
                    paymentId
                } else {
                    throw Exception("Failed to send payment")
                }
            } catch (e: Exception) {
                Log.e(TAG, "Failed to pay invoice", e)
                throw e
            }
        }
    }

    suspend fun openChannel(nodeId: String, address: String, amountSats: Long): String {
        return withContext(Dispatchers.IO) {
            if (!isInitialized) {
                throw Exception(context.getString(R.string.error_wallet_not_initialized))
            }
            
            if (amountSats < 10000) {
                throw Exception(context.getString(R.string.error_min_channel_amount))
            }
            
            try {
                val channelId = nativeOpenChannel(nodeId, address, amountSats)
                if (channelId != null) {
                    Log.d(TAG, "Opened channel: $channelId")
                    channelId
                } else {
                    throw Exception("Failed to open channel")
                }
            } catch (e: Exception) {
                Log.e(TAG, "Failed to open channel", e)
                throw e
            }
        }
    }

    suspend fun cleanup(): Result<Unit> {
        return try {
            Log.d(TAG, "Cleaning up Rust Lightning wallet...")
            
            withContext(Dispatchers.IO) {
                val success = nativeCleanup()
                if (!success) {
                    throw Exception("Failed to cleanup Lightning wallet via JNI")
                }
            }
            
            isInitialized = false
            Log.d(TAG, "Rust Lightning wallet cleaned up successfully")
            Result.success(Unit)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to cleanup Rust Lightning wallet", e)
            Result.failure(e)
        }
    }

    // Native JNI methods
    private external fun nativeInitialize(storagePath: String): Boolean
    private external fun nativeGetStatus(): WalletStatus?
    private external fun nativeGetBalance(): WalletBalance?
    private external fun nativeGetChannels(): Array<ChannelInfo>?
    private external fun nativeGetPayments(): Array<PaymentInfo>?
    private external fun nativeCreateInvoice(amountSats: Long, description: String): String?
    private external fun nativePayInvoice(invoice: String): String?
    private external fun nativeOpenChannel(nodeId: String, address: String, amountSats: Long): String?
    private external fun nativeCleanup(): Boolean
} 