package com.purrmint.app.utils

import android.content.Context
import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import android.net.wifi.WifiManager
import android.util.Log
import java.net.InetAddress
import java.net.NetworkInterface
import java.util.*

/**
 * Network utility class for getting local network information
 * Automatically detects local network IP addresses
 */
object NetworkUtils {
    
    private const val TAG = "NetworkUtils"
    
    /**
     * Get the local network IP address
     * @param port The port number to append
     * @return Local network address (e.g., "https://192.168.1.100:8443") or null if not available
     */
    fun getLocalNetworkAddress(port: Int, useHttps: Boolean = false): String? {
        return try {
            val localIp = getLocalIpAddress()
            if (localIp != null) {
                val protocol = if (useHttps) "https" else "http"
                val httpsPort = if (useHttps && port == 3338) 8443 else port
                "$protocol://$localIp:$httpsPort"
            } else {
                null
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error getting local network address", e)
            null
        }
    }
    
    /**
     * Get the local IP address on the current network
     * @return Local IP address string or null if not available
     */
    private fun getLocalIpAddress(): String? {
        return try {
            // Try to get IP from network interfaces
            val networkInterfaces = NetworkInterface.getNetworkInterfaces()
            
            while (networkInterfaces.hasMoreElements()) {
                val networkInterface = networkInterfaces.nextElement()
                
                // Skip loopback and down interfaces
                if (networkInterface.isLoopback || !networkInterface.isUp) {
                    continue
                }
                
                val inetAddresses = networkInterface.inetAddresses
                while (inetAddresses.hasMoreElements()) {
                    val inetAddress = inetAddresses.nextElement()
                    
                    // Skip IPv6 addresses for now (focus on IPv4)
                    val hostAddress = inetAddress.hostAddress
                    if (hostAddress?.contains(":") == true) {
                        continue
                    }
                    
                    // Check if it's a local network address
                    if (hostAddress != null && isLocalNetworkAddress(hostAddress)) {
                        Log.d(TAG, "Found local network address: $hostAddress")
                        return hostAddress
                    }
                }
            }
            
            Log.w(TAG, "No local network address found")
            null
            
        } catch (e: Exception) {
            Log.e(TAG, "Error getting local IP address", e)
            null
        }
    }
    
    /**
     * Check if the given IP address is a local network address
     * @param ipAddress The IP address to check
     * @return true if it's a local network address
     */
    private fun isLocalNetworkAddress(ipAddress: String): Boolean {
        return try {
            // Common local network ranges
            val localRanges = listOf(
                "192.168.",      // 192.168.0.0/16
                "10.",           // 10.0.0.0/8 (including emulator addresses)
                "172.16.",       // 172.16.0.0/12
                "172.17.",
                "172.18.",
                "172.19.",
                "172.20.",
                "172.21.",
                "172.22.",
                "172.23.",
                "172.24.",
                "172.25.",
                "172.26.",
                "172.27.",
                "172.28.",
                "172.29.",
                "172.30.",
                "172.31."
            )
            
            localRanges.any { ipAddress.startsWith(it) }
            
        } catch (e: Exception) {
            Log.e(TAG, "Error checking local network address", e)
            false
        }
    }
    
    /**
     * Check if the given IP address is an emulator address
     * @param ipAddress The IP address to check
     * @return true if it's an emulator address
     */
    fun isEmulatorAddress(ipAddress: String): Boolean {
        return ipAddress == "10.0.2.15" || ipAddress == "10.0.2.2"
    }
    
    /**
     * Check if the given IP address is accessible from external devices
     * @param ipAddress The IP address to check
     * @return true if it's accessible from external devices
     */
    fun isExternallyAccessible(ipAddress: String): Boolean {
        return !isEmulatorAddress(ipAddress)
    }
    
    /**
     * Get all available local network addresses
     * @return List of local network addresses
     */
    fun getAllLocalNetworkAddresses(port: Int, useHttps: Boolean = false): List<String> {
        val addresses = mutableListOf<String>()
        
        try {
            val networkInterfaces = NetworkInterface.getNetworkInterfaces()
            
            while (networkInterfaces.hasMoreElements()) {
                val networkInterface = networkInterfaces.nextElement()
                
                if (networkInterface.isLoopback || !networkInterface.isUp) {
                    continue
                }
                
                val inetAddresses = networkInterface.inetAddresses
                while (inetAddresses.hasMoreElements()) {
                    val inetAddress = inetAddresses.nextElement()
                    
                    val hostAddress = inetAddress.hostAddress
                    if (hostAddress?.contains(":") == true) {
                        continue
                    }
                    
                    if (hostAddress != null && isLocalNetworkAddress(hostAddress)) {
                        val protocol = if (useHttps) "https" else "http"
                        val httpsPort = if (useHttps && port == 3338) 8443 else port
                        addresses.add("$protocol://$hostAddress:$httpsPort")
                    }
                }
            }
            
        } catch (e: Exception) {
            Log.e(TAG, "Error getting all local network addresses", e)
        }
        
        return addresses
    }
    
    /**
     * Get the best available network address for external access
     * @param port The port number to append
     * @return Best network address or null if none available
     */
    fun getBestNetworkAddress(port: Int, useHttps: Boolean = false): String? {
        val allAddresses = getAllLocalNetworkAddresses(port, useHttps)
        
        if (allAddresses.isEmpty()) {
            return null
        }
        
        // Prioritize real network addresses over emulator addresses
        val realAddresses = allAddresses.filter { address ->
            !isEmulatorAddress(address.replace("https://", "").replace("http://", "").split(":")[0])
        }
        
        // Return the first real network address, or the first available address if none are real
        return realAddresses.firstOrNull() ?: allAddresses.first()
    }
    
    /**
     * Check if device is connected to a network
     * @param context Android context
     * @return true if connected to a network
     */
    fun isNetworkConnected(context: Context): Boolean {
        val connectivityManager = context.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager
        
        return try {
            val network = connectivityManager.activeNetwork
            if (network != null) {
                val networkCapabilities = connectivityManager.getNetworkCapabilities(network)
                networkCapabilities?.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET) == true
            } else {
                false
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error checking network connectivity", e)
            false
        }
    }
}
