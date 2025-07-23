package com.purrmint.app.wallet

/**
 * Wallet status data class
 */
data class WalletStatus(
    val isRunning: Boolean,
    val nodeId: String?,
    val isConnected: Boolean
)

/**
 * Wallet balance data class
 */
data class WalletBalance(
    val lightningBalanceMsat: Long,
    val onchainBalanceSats: Long
)

/**
 * Channel information data class
 */
data class ChannelInfo(
    val channelId: String,
    val peerId: String,
    val capacityMsat: Long,
    val balanceMsat: Long,
    val status: String
)

/**
 * Payment information data class
 */
data class PaymentInfo(
    val paymentId: String,
    val amountMsat: Long,
    val description: String?,
    val status: String,
    val isIncoming: Boolean,
    val timestamp: Long
) 