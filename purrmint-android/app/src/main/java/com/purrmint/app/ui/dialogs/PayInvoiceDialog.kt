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

class PayInvoiceDialog : DialogFragment() {

    private lateinit var mintService: MintService
    private var onPaymentSent: ((String) -> Unit)? = null

    override fun onCreateDialog(savedInstanceState: Bundle?): Dialog {
        mintService = MintService(requireContext())
        
        val view = LayoutInflater.from(context).inflate(R.layout.dialog_pay_invoice, null)
        
        val invoiceInput = view.findViewById<TextInputEditText>(R.id.invoiceInput)
        val payButton = view.findViewById<MaterialButton>(R.id.payButton)
        val cancelButton = view.findViewById<MaterialButton>(R.id.cancelButton)
        
        payButton.setOnClickListener {
            val invoice = invoiceInput.text.toString().trim()
            
            if (invoice.isEmpty()) {
                invoiceInput.error = getString(R.string.error_enter_invoice)
                return@setOnClickListener
            }
            
            if (!invoice.startsWith("lnbc")) {
                invoiceInput.error = getString(R.string.error_invalid_invoice)
                return@setOnClickListener
            }
            
            payButton.isEnabled = false
            payButton.text = getString(R.string.paying)
            
            CoroutineScope(Dispatchers.Main).launch {
                try {
                    val paymentId = withContext(Dispatchers.IO) {
                        mintService.payInvoice(invoice)
                    }
                    
                    onPaymentSent?.invoke(paymentId)
                    dismiss()
                    
                } catch (e: Exception) {
                    Toast.makeText(context, getString(R.string.payment_failed, e.message), Toast.LENGTH_SHORT).show()
                    payButton.isEnabled = true
                    payButton.text = getString(R.string.pay_invoice)
                }
            }
        }
        
        cancelButton.setOnClickListener {
            dismiss()
        }
        
        return AlertDialog.Builder(requireContext())
            .setTitle(getString(R.string.pay_invoice_title))
            .setView(view)
            .setCancelable(false)
            .create()
    }
    
    fun setOnPaymentSentListener(listener: (String) -> Unit) {
        onPaymentSent = listener
    }
    
    companion object {
        const val TAG = "PayInvoiceDialog"
    }
} 