package com.purrmint.app.ui.activities

import android.os.Bundle
import android.widget.Toast
import androidx.appcompat.app.AppCompatActivity
import androidx.recyclerview.widget.LinearLayoutManager
import androidx.recyclerview.widget.RecyclerView
import com.google.android.material.appbar.MaterialToolbar
import com.google.android.material.button.MaterialButton
import com.google.android.material.card.MaterialCardView
import com.google.android.material.chip.Chip
import com.google.android.material.textview.MaterialTextView
import com.purrmint.app.R
import com.purrmint.app.mint.MintService
import com.purrmint.app.wallet.WalletBalance
import com.purrmint.app.wallet.WalletStatus
import com.purrmint.app.wallet.PaymentInfo
import com.purrmint.app.ui.dialogs.CreateInvoiceDialog
import com.purrmint.app.ui.dialogs.PayInvoiceDialog
import android.os.Handler
import android.os.Looper
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import androidx.recyclerview.widget.RecyclerView.Adapter
import androidx.recyclerview.widget.RecyclerView.ViewHolder
import java.text.SimpleDateFormat
import java.util.*
import android.content.Context

class WalletActivity : AppCompatActivity() {

    private lateinit var toolbar: MaterialToolbar
    private lateinit var walletStatusChip: Chip
    private lateinit var walletBalanceText: MaterialTextView
    private lateinit var nodeIdText: MaterialTextView
    private lateinit var createInvoiceButton: MaterialButton
    private lateinit var payInvoiceButton: MaterialButton
    private lateinit var paymentsRecyclerView: RecyclerView
    private lateinit var paymentsAdapter: PaymentsAdapter
    
    private lateinit var mintService: MintService
    private val handler = Handler(Looper.getMainLooper())
    private val updateRunnable = object : Runnable {
        override fun run() {
            updateWalletInfo()
            handler.postDelayed(this, 5000) // Update every 5 seconds
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_wallet)
        
        // Initialize MintService
        mintService = MintService(this)
        
        // Initialize views
        initializeViews()
        
        // Setup toolbar
        setupToolbar()
        
        // Setup click listeners
        setupClickListeners()
        
        // Start periodic updates
        handler.post(updateRunnable)
    }

    private fun initializeViews() {
        toolbar = findViewById(R.id.toolbar)
        walletStatusChip = findViewById(R.id.walletStatusChip)
        walletBalanceText = findViewById(R.id.walletBalanceText)
        nodeIdText = findViewById(R.id.nodeIdText)
        createInvoiceButton = findViewById(R.id.createInvoiceButton)
        payInvoiceButton = findViewById(R.id.payInvoiceButton)
        paymentsRecyclerView = findViewById(R.id.paymentsRecyclerView)
        
        // Setup RecyclerView
        paymentsAdapter = PaymentsAdapter()
        paymentsRecyclerView.layoutManager = LinearLayoutManager(this)
        paymentsRecyclerView.adapter = paymentsAdapter
    }

    private fun setupToolbar() {
        setSupportActionBar(toolbar)
        supportActionBar?.setDisplayHomeAsUpEnabled(true)
        supportActionBar?.title = getString(R.string.lightning_wallet)
    }

    private fun setupClickListeners() {
        createInvoiceButton.setOnClickListener {
            showCreateInvoiceDialog()
        }
        
        payInvoiceButton.setOnClickListener {
            showPayInvoiceDialog()
        }
    }

    private fun showCreateInvoiceDialog() {
        val dialog = CreateInvoiceDialog()
        dialog.setOnInvoiceCreatedListener { invoice ->
            // Show success message and copy invoice to clipboard
            Toast.makeText(this, getString(R.string.invoice_created_success), Toast.LENGTH_SHORT).show()
            
            // Copy invoice to clipboard
            val clipboard = getSystemService(Context.CLIPBOARD_SERVICE) as android.content.ClipboardManager
            val clip = android.content.ClipData.newPlainText("Lightning Invoice", invoice)
            clipboard.setPrimaryClip(clip)
            
            Toast.makeText(this, getString(R.string.invoice_copied_clipboard), Toast.LENGTH_SHORT).show()
        }
        dialog.show(supportFragmentManager, CreateInvoiceDialog.TAG)
    }

    private fun showPayInvoiceDialog() {
        val dialog = PayInvoiceDialog()
        dialog.setOnPaymentSentListener { paymentId ->
            Toast.makeText(this, getString(R.string.payment_sent_success, paymentId), Toast.LENGTH_LONG).show()
        }
        dialog.show(supportFragmentManager, PayInvoiceDialog.TAG)
    }

    private fun updateWalletInfo() {
        val walletStatus = mintService.getWalletStatus()
        val walletBalance = mintService.getWalletBalance()
        val recentPayments = mintService.getRecentPayments()
        
        runOnUiThread {
            // Update wallet status
            if (walletStatus.isRunning) {
                walletStatusChip.text = getString(R.string.connected)
                walletStatusChip.setChipBackgroundColorResource(R.color.success_container_color)
                walletStatusChip.setTextColor(resources.getColor(R.color.success_color, null))
            } else {
                walletStatusChip.text = getString(R.string.disconnected)
                walletStatusChip.setChipBackgroundColorResource(R.color.error_container_color)
                walletStatusChip.setTextColor(resources.getColor(R.color.error_color, null))
            }
            
            // Update wallet balance
            val balanceSats = walletBalance?.lightningBalanceMsat?.div(1000) ?: 0
            walletBalanceText.text = "$balanceSats sats"
            
            // Update node ID
            walletStatus.nodeId?.let { nodeId ->
                nodeIdText.text = getString(R.string.node_id_format, nodeId.take(20))
            } ?: run {
                nodeIdText.text = getString(R.string.node_id_not_connected)
            }
            
            // Update payments list
            paymentsAdapter.updatePayments(recentPayments)
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        handler.removeCallbacks(updateRunnable)
    }

    override fun onSupportNavigateUp(): Boolean {
        onBackPressed()
        return true
    }

    // Payments RecyclerView Adapter
    private inner class PaymentsAdapter : Adapter<PaymentsAdapter.PaymentViewHolder>() {
        private var payments: List<PaymentInfo> = emptyList()

        fun updatePayments(newPayments: List<PaymentInfo>) {
            payments = newPayments
            notifyDataSetChanged()
        }

        override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): PaymentViewHolder {
            val view = LayoutInflater.from(parent.context)
                .inflate(R.layout.item_payment, parent, false)
            return PaymentViewHolder(view)
        }

        override fun onBindViewHolder(holder: PaymentViewHolder, position: Int) {
            holder.bind(payments[position])
        }

        override fun getItemCount(): Int = payments.size

        inner class PaymentViewHolder(itemView: View) : ViewHolder(itemView) {
            private val amountText: TextView = itemView.findViewById(R.id.amountText)
            private val statusText: TextView = itemView.findViewById(R.id.statusText)
            private val dateText: TextView = itemView.findViewById(R.id.dateText)
            private val descriptionText: TextView = itemView.findViewById(R.id.descriptionText)

            fun bind(payment: PaymentInfo) {
                val amountSats = payment.amountMsat / 1000
                amountText.text = "${if (payment.isIncoming) "+" else "-"}$amountSats sats"
                amountText.setTextColor(resources.getColor(
                    if (payment.isIncoming) R.color.success_color else R.color.error_color, null
                ))
                
                statusText.text = when (payment.status) {
                    "SUCCEEDED" -> getString(R.string.payment_status_succeeded)
                    "PENDING" -> getString(R.string.payment_status_pending)
                    "FAILED" -> getString(R.string.payment_status_failed)
                    else -> payment.status
                }
                
                val dateFormat = SimpleDateFormat("yyyy-MM-dd HH:mm", Locale.getDefault())
                dateText.text = dateFormat.format(Date(payment.timestamp))
                
                descriptionText.text = payment.description ?: getString(R.string.no_description)
            }
        }
    }
} 