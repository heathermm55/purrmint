package com.purrmint.app.utils

import android.graphics.Bitmap
import android.graphics.Color
import com.google.zxing.BarcodeFormat
import com.google.zxing.EncodeHintType
import com.google.zxing.WriterException
import com.google.zxing.common.BitMatrix
import com.google.zxing.qrcode.QRCodeWriter
import java.util.*

/**
 * QR Code generator utility class
 * Provides methods to generate QR codes for various types of data
 */
object QRCodeGenerator {
    
    /**
     * Generate QR code bitmap for the given content
     * @param content The content to encode in the QR code
     * @param size The size of the QR code (width and height in pixels)
     * @return Bitmap containing the generated QR code
     */
    fun generateQRCode(content: String, size: Int = 512): Bitmap? {
        return try {
            val hints = EnumMap<EncodeHintType, Any>(EncodeHintType::class.java)
            hints[EncodeHintType.CHARACTER_SET] = "UTF-8"
            hints[EncodeHintType.MARGIN] = 1
            
            val writer = QRCodeWriter()
            val bitMatrix: BitMatrix = writer.encode(content, BarcodeFormat.QR_CODE, size, size, hints)
            
            val bitmap = Bitmap.createBitmap(size, size, Bitmap.Config.ARGB_8888)
            
            for (x in 0 until size) {
                for (y in 0 until size) {
                    bitmap.setPixel(x, y, if (bitMatrix[x, y]) Color.BLACK else Color.WHITE)
                }
            }
            
            bitmap
        } catch (e: WriterException) {
            e.printStackTrace()
            null
        }
    }
    
    /**
     * Generate QR code for a local address
     * @param localAddress The local address (e.g., http://127.0.0.1:3338)
     * @param size The size of the QR code
     * @return Bitmap containing the generated QR code
     */
    fun generateLocalAddressQR(localAddress: String, size: Int = 512): Bitmap? {
        return generateQRCode(localAddress, size)
    }
    
    /**
     * Generate QR code for a Tor onion address
     * @param onionAddress The onion address (e.g., abc123def456.onion)
     * @param size The size of the QR code
     * @return Bitmap containing the generated QR code
     */
    fun generateOnionAddressQR(onionAddress: String, size: Int = 512): Bitmap? {
        // For onion addresses, we might want to include additional context
        val content = if (onionAddress.startsWith("http://") || onionAddress.startsWith("https://")) {
            onionAddress
        } else {
            "http://$onionAddress"
        }
        return generateQRCode(content, size)
    }
    
    /**
     * Generate QR code for a complete service URL
     * @param baseAddress The base address (local or onion)
     * @param endpoint Optional endpoint path
     * @param size The size of the QR code
     * @return Bitmap containing the generated QR code
     */
    fun generateServiceURLQR(baseAddress: String, endpoint: String? = null, size: Int = 512): Bitmap? {
        val url = if (endpoint != null) {
            "$baseAddress/$endpoint"
        } else {
            baseAddress
        }
        return generateQRCode(url, size)
    }
}
