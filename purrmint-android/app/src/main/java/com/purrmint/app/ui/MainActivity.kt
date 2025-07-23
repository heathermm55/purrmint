package com.purrmint.app.ui

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import com.purrmint.app.mint.MintService
import com.purrmint.app.ui.screens.*
import com.purrmint.app.ui.theme.PurrmintTheme

class MainActivity : ComponentActivity() {
    
    private lateinit var mintService: MintService
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        // Initialize Mint service
        mintService = MintService(this)
        mintService.initialize()
        
        setContent {
            PurrmintTheme {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    PurrmintApp(mintService)
                }
            }
        }
    }
    
    override fun onDestroy() {
        super.onDestroy()
        mintService.cleanup()
    }
}

@Composable
fun PurrmintApp(mintService: MintService) {
    val navController = rememberNavController()
    
    NavHost(navController = navController, startDestination = "home") {
        composable("home") { HomeScreen(navController, mintService) }
        composable("mint") { MintScreen(navController, mintService) }
        composable("wallet") { WalletScreen(navController, mintService) }
        composable("channels") { ChannelsScreen(navController, mintService) }
        composable("payments") { PaymentsScreen(navController, mintService) }
        composable("create_invoice") { CreateInvoiceScreen(navController, mintService) }
        composable("pay_invoice") { PayInvoiceScreen(navController, mintService) }
        composable("open_channel") { OpenChannelScreen(navController, mintService) }
    }
} 