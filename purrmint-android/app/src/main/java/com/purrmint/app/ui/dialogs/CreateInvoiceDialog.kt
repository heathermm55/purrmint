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

class CreateInvoiceDialog : DialogFragment() {

    private lateinit var mintService: MintService
    private var onInvoiceCreated: ((String) -> Unit)? = null

    override fun onCreateDialog(savedInstanceState: Bundle?): Dialog {
        mintService = MintService(requireContext())
        
        val view = LayoutInflater.from(context).inflate(R.layout.dialog_create_invoice, null)
        
        val amountInput = view.findViewById<TextInputEditText>(R.id.amountInput)
        val descriptionInput = view.findViewById<TextInputEditText>(R.id.descriptionInput)
        val createButton = view.findViewById<MaterialButton>(R.id.createButton)
        val cancelButton = view.findViewById<MaterialButton>(R.id.cancelButton)
        
        createButton.setOnClickListener {
            val amountText = amountInput.text.toString()
            val description = descriptionInput.text.toString()
            
            if (amountText.isEmpty()) {
                amountInput.error = getString(R.string.error_enter_amount)
                return@setOnClickListener
            }
            
            val amountSats = amountText.toLongOrNull()
            if (amountSats == null || amountSats <= 0) {
                amountInput.error = getString(R.string.error_invalid_amount)
                return@setOnClickListener
            }
            
            createButton.isEnabled = false
            createButton.text = getString(R.string.creating)
            
            CoroutineScope(Dispatchers.Main).launch {
                try {
                    val invoice = withContext(Dispatchers.IO) {
                        mintService.createInvoice(amountSats, description.ifEmpty { getString(R.string.payment_description_test) })
                    }
                    
                    onInvoiceCreated?.invoke(invoice)
                    dismiss()
                    
                } catch (e: Exception) {
                    Toast.makeText(context, getString(R.string.create_invoice_failed, e.message), Toast.LENGTH_SHORT).show()
                    createButton.isEnabled = true
                    createButton.text = getString(R.string.create_invoice)
                }
            }
        }
        
        cancelButton.setOnClickListener {
            dismiss()
        }
        
        return AlertDialog.Builder(requireContext())
            .setTitle(getString(R.string.create_invoice_title))
            .setView(view)
            .setCancelable(false)
            .create()
    }
    
    fun setOnInvoiceCreatedListener(listener: (String) -> Unit) {
        onInvoiceCreated = listener
    }
    
    companion object {
        const val TAG = "CreateInvoiceDialog"
    }
} 