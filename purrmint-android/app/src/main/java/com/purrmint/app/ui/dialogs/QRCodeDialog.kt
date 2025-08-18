package com.purrmint.app.ui.dialogs

import android.app.Dialog
import android.content.Context
import android.graphics.Bitmap
import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.ImageView
import android.widget.TextView
import androidx.appcompat.app.AlertDialog
import androidx.fragment.app.DialogFragment
import com.purrmint.app.R
import com.purrmint.app.utils.QRCodeGenerator

/**
 * Dialog fragment for displaying QR codes
 * Supports both local and onion addresses
 */
class QRCodeDialog : DialogFragment() {
    
    companion object {
        private const val ARG_ADDRESS = "address"
        private const val ARG_ADDRESS_TYPE = "address_type"
        private const val ARG_TITLE = "title"
        
        /**
         * Create a new QR code dialog for local address
         */
        fun newLocalAddressInstance(localAddress: String): QRCodeDialog {
            return QRCodeDialog().apply {
                arguments = Bundle().apply {
                    putString(ARG_ADDRESS, localAddress)
                    putString(ARG_ADDRESS_TYPE, "local")
                    putString(ARG_TITLE, "Local Address QR Code")
                }
            }
        }
        
        /**
         * Create a new QR code dialog for onion address
         */
        fun newOnionAddressInstance(onionAddress: String): QRCodeDialog {
            return QRCodeDialog().apply {
                arguments = Bundle().apply {
                    putString(ARG_ADDRESS, onionAddress)
                    putString(ARG_ADDRESS_TYPE, "onion")
                    putString(ARG_TITLE, "Onion Address QR Code")
                }
            }
        }
    }
    
    private var address: String? = null
    private var addressType: String? = null
    private var title: String? = null
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        arguments?.let {
            address = it.getString(ARG_ADDRESS)
            addressType = it.getString(ARG_ADDRESS_TYPE)
            title = it.getString(ARG_TITLE)
        }
    }
    
    override fun onCreateDialog(savedInstanceState: Bundle?): Dialog {
        val builder = AlertDialog.Builder(requireContext())
        builder.setTitle(title)
        
        val view = LayoutInflater.from(context).inflate(R.layout.dialog_qr_code, null)
        val qrImageView = view.findViewById<ImageView>(R.id.qr_code_image)
        val addressTextView = view.findViewById<TextView>(R.id.address_text)
        val typeTextView = view.findViewById<TextView>(R.id.address_type_text)
        
        // Set the address text
        addressTextView.text = address
        
        // Set the address type text
        val typeText = when (addressType) {
            "local" -> "Local Address"
            "onion" -> "Onion Address (Tor)"
            else -> "Unknown Address Type"
        }
        typeTextView.text = typeText
        
        // Generate and display QR code
        address?.let { addr ->
            val qrBitmap = when (addressType) {
                "local" -> QRCodeGenerator.generateLocalAddressQR(addr, 512)
                "onion" -> QRCodeGenerator.generateOnionAddressQR(addr, 512)
                else -> QRCodeGenerator.generateQRCode(addr, 512)
            }
            
            qrBitmap?.let { bitmap ->
                qrImageView.setImageBitmap(bitmap)
            } ?: run {
                qrImageView.setImageResource(R.drawable.ic_error)
                addressTextView.text = "Error generating QR code"
            }
        }
        
        builder.setView(view)
        builder.setPositiveButton("Close") { _, _ -> dismiss() }
        
        return builder.create()
    }
}
