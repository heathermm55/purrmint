package com.purrmint.app.ui.screens

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.navigation.NavController
import com.purrmint.app.R
import com.purrmint.app.mint.MintService
import com.purrmint.app.wallet.WalletStatus
import com.purrmint.app.wallet.WalletBalance

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun HomeScreen(
    navController: NavController,
    mintService: MintService
) {
    var walletStatus by remember { mutableStateOf<WalletStatus?>(mintService.getWalletStatus()) }
    var walletBalance by remember { mutableStateOf<WalletBalance?>(mintService.getWalletBalance()) }
    
    LaunchedEffect(mintService) {
        // Update status periodically
        while (true) {
            walletStatus = mintService.getWalletStatus()
            walletBalance = mintService.getWalletBalance()
            kotlinx.coroutines.delay(5000) // Update every 5 seconds
        }
    }
    
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("PurrMint") },
                actions = {
                    IconButton(onClick = { /* TODO: Settings */ }) {
                        Icon(Icons.Default.Settings, contentDescription = stringResource(R.string.settings_button))
                    }
                }
            )
        }
    ) { paddingValues ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            // Wallet status card
            Card(
                modifier = Modifier.fillMaxWidth(),
                elevation = CardDefaults.cardElevation(defaultElevation = 4.dp)
            ) {
                Column(
                    modifier = Modifier.padding(16.dp),
                    verticalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    Text(
                        text = stringResource(R.string.lightning_wallet_status),
                        fontSize = 18.sp,
                        fontWeight = FontWeight.Bold
                    )
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        Icon(
                            imageVector = if (walletStatus?.isRunning == true) Icons.Default.CheckCircle else Icons.Default.Warning,
                            contentDescription = null,
                            tint = if (walletStatus?.isRunning == true) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.error
                        )
                        Text(text = if (walletStatus?.isRunning == true) stringResource(R.string.connected) else stringResource(R.string.disconnected))
                    }
                    
                    walletStatus?.nodeId?.let { nodeId ->
                        Text(
                            text = stringResource(R.string.node_id_format, nodeId.take(20) + "..."),
                            fontSize = 12.sp,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
            }
            
            // Balance card
            Card(
                modifier = Modifier.fillMaxWidth(),
                elevation = CardDefaults.cardElevation(defaultElevation = 4.dp)
            ) {
                Column(
                    modifier = Modifier.padding(16.dp),
                    verticalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    Text(
                        text = stringResource(R.string.lightning_network_balance),
                        fontSize = 18.sp,
                        fontWeight = FontWeight.Bold
                    )
                    Text(
                        text = "${walletBalance?.lightningBalanceMsat?.div(1000) ?: 0} sats",
                        fontSize = 24.sp,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.primary
                    )
                }
            }
            
            // Feature buttons
            LazyVerticalGrid(
                columns = GridCells.Fixed(2),
                horizontalArrangement = Arrangement.spacedBy(16.dp),
                verticalArrangement = Arrangement.spacedBy(16.dp)
            ) {
                item {
                    ActionCard(
                        title = stringResource(R.string.cashu_mint),
                        subtitle = stringResource(R.string.create_and_manage_tokens),
                        icon = Icons.Default.Home,
                        onClick = { navController.navigate("mint") }
                    )
                }
                item {
                    ActionCard(
                        title = stringResource(R.string.lightning_wallet),
                        subtitle = stringResource(R.string.manage_lightning_network),
                        icon = Icons.Default.Person,
                        onClick = { navController.navigate("wallet") }
                    )
                }
                item {
                    ActionCard(
                        title = stringResource(R.string.channel_management),
                        subtitle = stringResource(R.string.manage_lightning_channels),
                        icon = Icons.Default.List,
                        onClick = { navController.navigate("channels") }
                    )
                }
                item {
                    ActionCard(
                        title = stringResource(R.string.payment_history),
                        subtitle = stringResource(R.string.view_payment_records),
                        icon = Icons.Default.Info,
                        onClick = { navController.navigate("payments") }
                    )
                }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ActionCard(
    title: String,
    subtitle: String,
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    onClick: () -> Unit
) {
    Card(
        modifier = Modifier
            .fillMaxWidth()
            .height(140.dp),
        elevation = CardDefaults.cardElevation(defaultElevation = 4.dp),
        onClick = onClick
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(16.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.Center
        ) {
            Icon(
                imageVector = icon,
                contentDescription = null,
                modifier = Modifier.size(32.dp),
                tint = MaterialTheme.colorScheme.primary
            )
            Spacer(modifier = Modifier.height(8.dp))
            Text(
                text = title,
                fontSize = 16.sp,
                fontWeight = FontWeight.Bold,
                textAlign = TextAlign.Center
            )
            Text(
                text = subtitle,
                fontSize = 12.sp,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                textAlign = TextAlign.Center
            )
        }
    }
} 