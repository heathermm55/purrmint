package com.purrmint.app.ui.fragments

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import androidx.fragment.app.Fragment
import com.google.android.material.button.MaterialButton
import com.google.android.material.chip.Chip
import android.widget.ImageView
import com.purrmint.app.R
import com.purrmint.app.ui.activities.MainActivity

class MintStatusFragment : Fragment() {
    
    private lateinit var statusIcon: ImageView
    private lateinit var statusChip: Chip
    private lateinit var statusTextView: TextView
    private lateinit var startButton: MaterialButton
    private lateinit var deleteButton: MaterialButton
    private lateinit var logoutButton: MaterialButton
    private lateinit var accountInfoText: TextView
    
    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View? {
        return inflater.inflate(R.layout.fragment_mint_status, container, false)
    }
    
    override fun onViewCreated(view: View, savedInstanceState: Bundle?) {
        super.onViewCreated(view, savedInstanceState)
        
        initializeViews(view)
        setupClickListeners()
    }
    
    private fun initializeViews(view: View) {
        statusIcon = view.findViewById(R.id.statusIcon)
        statusChip = view.findViewById(R.id.statusChip)
        statusTextView = view.findViewById(R.id.statusTextView)
        startButton = view.findViewById(R.id.startButton)
        deleteButton = view.findViewById(R.id.deleteButton)
        logoutButton = view.findViewById(R.id.logoutButton)
        accountInfoText = view.findViewById(R.id.accountInfoText)
    }
    
    private fun setupClickListeners() {
        startButton.setOnClickListener {
            (activity as? MainActivity)?.startMintService()
        }
        
        deleteButton.setOnClickListener {
            (activity as? MainActivity)?.showDeleteServiceConfirmationDialog()
        }
        
        logoutButton.setOnClickListener {
            (activity as? MainActivity)?.logout()
        }
    }
    
    fun updateStatus(status: String, isOnline: Boolean) {
        statusTextView.text = status
        
        if (isOnline) {
            statusIcon.setImageResource(R.drawable.ic_status_online)
            statusIcon.setColorFilter(requireContext().getColor(R.color.success_color))
            statusChip.text = "Online"
            statusChip.setChipBackgroundColorResource(R.color.success_container_color)
            statusChip.setTextColor(requireContext().getColor(R.color.success_color))
        } else {
            statusIcon.setImageResource(R.drawable.ic_status_offline)
            statusIcon.setColorFilter(requireContext().getColor(R.color.error_color))
            statusChip.text = "Offline"
            statusChip.setChipBackgroundColorResource(R.color.error_container_color)
            statusChip.setTextColor(requireContext().getColor(R.color.error_color))
        }
    }
    
    fun enableStartButton() {
        startButton.isEnabled = true
        startButton.text = "Start Mint Service"
        startButton.setIconResource(R.drawable.ic_play)
    }
    
    fun updateStartButton(text: String, isRunning: Boolean) {
        startButton.text = text
        if (isRunning) {
            startButton.setIconResource(R.drawable.ic_stop)
        } else {
            startButton.setIconResource(R.drawable.ic_play)
        }
    }
    
    fun enableDeleteButton() {
        deleteButton.isEnabled = true
        deleteButton.text = getString(R.string.delete_mint_service)
        deleteButton.setIconResource(R.drawable.ic_delete)
    }
    
    fun disableDeleteButton() {
        // Keep delete button enabled for cleanup purposes
        deleteButton.isEnabled = true
        deleteButton.text = getString(R.string.delete_mint_service)
        deleteButton.setIconResource(R.drawable.ic_delete)
    }
    
    fun updateDeleteButton(text: String, isEnabled: Boolean) {
        deleteButton.text = text
        deleteButton.isEnabled = isEnabled
        if (isEnabled) {
            deleteButton.setIconResource(R.drawable.ic_delete)
        } else {
            deleteButton.setIconResource(R.drawable.ic_delete)
        }
    }
    
    fun updateAccountInfo(accountInfo: String) {
        accountInfoText.text = accountInfo
    }
} 