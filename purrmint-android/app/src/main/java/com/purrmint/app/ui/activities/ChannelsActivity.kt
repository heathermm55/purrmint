package com.purrmint.app.ui.activities

import android.os.Bundle
import android.widget.Toast
import androidx.appcompat.app.AppCompatActivity
import androidx.recyclerview.widget.LinearLayoutManager
import androidx.recyclerview.widget.RecyclerView
import com.google.android.material.appbar.MaterialToolbar
import com.google.android.material.button.MaterialButton
import com.google.android.material.chip.Chip
import com.google.android.material.textview.MaterialTextView
import com.purrmint.app.R
import com.purrmint.app.mint.MintService
import com.purrmint.app.wallet.ChannelInfo
import com.purrmint.app.ui.dialogs.OpenChannelDialog
import android.os.Handler
import android.os.Looper
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import androidx.recyclerview.widget.RecyclerView.Adapter
import androidx.recyclerview.widget.RecyclerView.ViewHolder

class ChannelsActivity : AppCompatActivity() {

    private lateinit var toolbar: MaterialToolbar
    private lateinit var totalChannelsText: MaterialTextView
    private lateinit var totalCapacityText: MaterialTextView
    private lateinit var openChannelButton: MaterialButton
    private lateinit var channelsRecyclerView: RecyclerView
    private lateinit var channelsAdapter: ChannelsAdapter
    
    private lateinit var mintService: MintService
    private val handler = Handler(Looper.getMainLooper())
    private val updateRunnable = object : Runnable {
        override fun run() {
            updateChannelsInfo()
            handler.postDelayed(this, 5000) // Update every 5 seconds
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_channels)
        
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
        totalChannelsText = findViewById(R.id.totalChannelsText)
        totalCapacityText = findViewById(R.id.totalCapacityText)
        openChannelButton = findViewById(R.id.openChannelButton)
        channelsRecyclerView = findViewById(R.id.channelsRecyclerView)
        
        // Setup RecyclerView
        channelsAdapter = ChannelsAdapter()
        channelsRecyclerView.layoutManager = LinearLayoutManager(this)
        channelsRecyclerView.adapter = channelsAdapter
    }

    private fun setupToolbar() {
        setSupportActionBar(toolbar)
        supportActionBar?.setDisplayHomeAsUpEnabled(true)
        supportActionBar?.title = getString(R.string.channel_management)
    }

    private fun setupClickListeners() {
        openChannelButton.setOnClickListener {
            showOpenChannelDialog()
        }
    }

    private fun showOpenChannelDialog() {
        val dialog = OpenChannelDialog()
        dialog.setOnChannelOpenedListener { channelId ->
            Toast.makeText(this, getString(R.string.channel_opened_success, channelId), Toast.LENGTH_LONG).show()
        }
        dialog.show(supportFragmentManager, OpenChannelDialog.TAG)
    }

    private fun updateChannelsInfo() {
        val channels = mintService.getChannels()
        
        runOnUiThread {
            // Update channels summary
            val totalChannels = channels.size
            val totalCapacity = channels.sumOf { it.capacityMsat / 1000 }
            totalChannelsText.text = getString(R.string.total_channels) + ": $totalChannels"
            totalCapacityText.text = getString(R.string.total_capacity) + ": $totalCapacity sats"
            
            // Update channels list
            channelsAdapter.updateChannels(channels)
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

    // Channels RecyclerView Adapter
    private inner class ChannelsAdapter : Adapter<ChannelsAdapter.ChannelViewHolder>() {
        private var channels: List<ChannelInfo> = emptyList()

        fun updateChannels(newChannels: List<ChannelInfo>) {
            channels = newChannels
            notifyDataSetChanged()
        }

        override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): ChannelViewHolder {
            val view = LayoutInflater.from(parent.context)
                .inflate(R.layout.item_channel, parent, false)
            return ChannelViewHolder(view)
        }

        override fun onBindViewHolder(holder: ChannelViewHolder, position: Int) {
            holder.bind(channels[position])
        }

        override fun getItemCount(): Int = channels.size

        inner class ChannelViewHolder(itemView: View) : ViewHolder(itemView) {
            private val peerIdText: TextView = itemView.findViewById(R.id.peerIdText)
            private val capacityText: TextView = itemView.findViewById(R.id.capacityText)
            private val balanceText: TextView = itemView.findViewById(R.id.balanceText)
            private val statusText: TextView = itemView.findViewById(R.id.statusText)
            private val channelIdText: TextView = itemView.findViewById(R.id.channelIdText)

            fun bind(channel: ChannelInfo) {
                peerIdText.text = getString(R.string.node_format, channel.peerId.take(20) + "...")
                capacityText.text = getString(R.string.capacity_format, channel.capacityMsat / 1000)
                balanceText.text = getString(R.string.balance_format_sats, channel.balanceMsat / 1000)
                
                statusText.text = when (channel.status) {
                    "OPEN" -> getString(R.string.channel_status_open)
                    "PENDING" -> getString(R.string.channel_status_pending)
                    "CLOSING" -> getString(R.string.channel_status_closing)
                    "CLOSED" -> getString(R.string.channel_status_closed)
                    else -> channel.status
                }
                
                channelIdText.text = "通道 ID: ${channel.channelId.take(16)}..."
            }
        }
    }
} 