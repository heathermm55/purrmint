package com.purrmint.app.ui.dialogs

import android.app.Dialog
import android.content.Context
import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.Toast
import androidx.appcompat.app.AlertDialog
import androidx.fragment.app.DialogFragment
import com.google.android.material.button.MaterialButton
import com.google.android.material.textfield.TextInputEditText
import com.purrmint.app.R
import com.purrmint.app.mint.MintService
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

class OpenChannelDialog : DialogFragment() {

    private lateinit var mintService: MintService
    private var onChannelOpened: ((String) -> Unit)? = null

    override fun onCreateDialog(savedInstanceState: Bundle?): Dialog {
        mintService = MintService(requireContext())
        
        val view = LayoutInflater.from(context).inflate(R.layout.dialog_open_channel, null)
        
        val nodeIdInput = view.findViewById<TextInputEditText>(R.id.nodeIdInput)
        val addressInput = view.findViewById<TextInputEditText>(R.id.addressInput)
        val amountInput = view.findViewById<TextInputEditText>(R.id.amountInput)
        val openButton = view.findViewById<MaterialButton>(R.id.openButton)
        val cancelButton = view.findViewById<MaterialButton>(R.id.cancelButton)
        
        openButton.setOnClickListener {
            val nodeId = nodeIdInput.text.toString().trim()
            val address = addressInput.text.toString().trim()
            val amountText = amountInput.text.toString()
            
            if (nodeId.isEmpty()) {
                nodeIdInput.error = getString(R.string.error_enter_node_id)
                return@setOnClickListener
            }
            
            if (address.isEmpty()) {
                addressInput.error = getString(R.string.error_enter_address)
                return@setOnClickListener
            }
            
            if (amountText.isEmpty()) {
                amountInput.error = getString(R.string.error_enter_channel_amount)
                return@setOnClickListener
            }
            
            val amountSats = amountText.toLongOrNull()
            if (amountSats == null || amountSats <= 0) {
                amountInput.error = getString(R.string.error_invalid_amount)
                return@setOnClickListener
            }
            
            if (amountSats < 10000) {
                amountInput.error = getString(R.string.error_min_channel_amount)
                return@setOnClickListener
            }
            
            openButton.isEnabled = false
            openButton.text = getString(R.string.opening)
            
            CoroutineScope(Dispatchers.Main).launch {
                try {
                    val channelId = withContext(Dispatchers.IO) {
                        mintService.openChannel(nodeId, address, amountSats)
                    }
                    
                    onChannelOpened?.invoke(channelId)
                    dismiss()
                    
                } catch (e: Exception) {
                    Toast.makeText(context, getString(R.string.open_channel_failed, e.message), Toast.LENGTH_SHORT).show()
                    openButton.isEnabled = true
                    openButton.text = getString(R.string.open_new_channel)
                }
            }
        }
        
        cancelButton.setOnClickListener {
            dismiss()
        }
        
        return AlertDialog.Builder(requireContext())
            .setTitle(getString(R.string.open_channel_title))
            .setView(view)
            .setCancelable(false)
            .create()
    }
    
    fun setOnChannelOpenedListener(listener: (String) -> Unit) {
        onChannelOpened = listener
    }
    
    companion object {
        const val TAG = "OpenChannelDialog"
    }
} 