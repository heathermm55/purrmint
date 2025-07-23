package com.purrmint.app.mint

import android.content.Context
import android.util.Log
import com.purrmint.app.wallet.RustLightningWalletManager
import com.purrmint.app.wallet.WalletStatus
import com.purrmint.app.wallet.WalletBalance
import com.purrmint.app.wallet.ChannelInfo
import com.purrmint.app.wallet.PaymentInfo
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch

/**
 * Mint Service
 * Integrates Lightning wallet functionality, provides Cashu Mint service
 */
class MintService(private val context: Context) {
    
    private val walletManager = RustLightningWalletManager(context)
    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    
    companion object {
        private const val TAG = "MintService"
    }
    
    /**
     * Initialize Mint service
     */
    fun initialize() {
        scope.launch {
            walletManager.initialize().onSuccess {
                Log.d(TAG, "Mint service initialized with Rust Lightning wallet")
            }.onFailure { error ->
                Log.e(TAG, "Failed to initialize Mint service", error)
            }
        }
    }
    
    /**
     * Get wallet status
     */
    fun getWalletStatus(): WalletStatus {
        return walletManager.getWalletStatus()
    }
    
    /**
     * Get wallet balance
     */
    fun getWalletBalance(): WalletBalance? {
        return walletManager.getWalletBalance()
    }
    
    /**
     * Get channels list
     */
    fun getChannels(): List<ChannelInfo> {
        return walletManager.getChannels()
    }
    
    /**
     * Get payment history
     */
    fun getPaymentHistory(): List<PaymentInfo> {
        return walletManager.getPaymentHistory()
    }

    fun getRecentPayments(): List<PaymentInfo> {
        return walletManager.getRecentPayments()
    }

    suspend fun createInvoice(amountSats: Long, description: String): String {
        return walletManager.createInvoice(amountSats, description)
    }

    suspend fun payInvoice(invoice: String): String {
        return walletManager.payInvoice(invoice)
    }

    suspend fun openChannel(nodeId: String, address: String, amountSats: Long): String {
        return walletManager.openChannel(nodeId, address, amountSats)
    }
    
    /**
     * Cleanup resources
     */
    fun cleanup() {
        scope.launch {
            walletManager.cleanup().onSuccess {
                Log.d(TAG, "Mint service cleaned up")
            }.onFailure { error ->
                Log.e(TAG, "Failed to cleanup Mint service", error)
            }
        }
    }
} 