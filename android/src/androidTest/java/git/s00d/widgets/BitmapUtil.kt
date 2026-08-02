package git.s00d.widgets

import android.content.Context
import android.graphics.Bitmap
import android.graphics.Color
import java.security.MessageDigest
import kotlin.math.abs

internal object BitmapUtil {
    fun sha256(bmp: Bitmap): String {
        val md = MessageDigest.getInstance("SHA-256")
        val tmp = IntArray(bmp.width * bmp.height)
        bmp.getPixels(tmp, 0, bmp.width, 0, 0, bmp.width, bmp.height)
        val bytes = ByteArray(tmp.size * 4)
        var i = 0
        for (px in tmp) {
            bytes[i++] = (px ushr 24).toByte()
            bytes[i++] = (px ushr 16).toByte()
            bytes[i++] = (px ushr 8).toByte()
            bytes[i++] = px.toByte()
        }
        return md.digest(bytes).joinToString("") { "%02x".format(it) }
    }

    /** True when sampled pixels are (nearly) one color — empty/dead render. */
    fun isUniform(bmp: Bitmap, step: Int = 8): Boolean {
        if (bmp.width <= 0 || bmp.height <= 0) return true
        var first = Color.TRANSPARENT
        var have = false
        var y = 0
        while (y < bmp.height) {
            var x = 0
            while (x < bmp.width) {
                val px = bmp.getPixel(x, y)
                if (!have) {
                    first = px
                    have = true
                } else if (px != first) {
                    return false
                }
                x += step
            }
            y += step
        }
        return true
    }

    /**
     * Fraction of pixels whose max channel delta exceeds [threshold].
     * Returns 1.0 if sizes differ.
     */
    fun mismatchFraction(a: Bitmap, b: Bitmap, threshold: Int = 8): Double {
        if (a.width != b.width || a.height != b.height) return 1.0
        val n = a.width * a.height
        if (n == 0) return 1.0
        val pa = IntArray(n)
        val pb = IntArray(n)
        a.getPixels(pa, 0, a.width, 0, 0, a.width, a.height)
        b.getPixels(pb, 0, b.width, 0, 0, b.width, b.height)
        var bad = 0
        for (i in 0 until n) {
            val ca = pa[i]
            val cb = pb[i]
            val dr = abs(((ca shr 16) and 0xff) - ((cb shr 16) and 0xff))
            val dg = abs(((ca shr 8) and 0xff) - ((cb shr 8) and 0xff))
            val db = abs((ca and 0xff) - (cb and 0xff))
            val da = abs(((ca ushr 24) and 0xff) - ((cb ushr 24) and 0xff))
            if (maxOf(dr, dg, db, da) > threshold) bad++
        }
        return bad.toDouble() / n
    }

    fun diffBitmap(a: Bitmap, b: Bitmap): Bitmap {
        val w = minOf(a.width, b.width)
        val h = minOf(a.height, b.height)
        val out = Bitmap.createBitmap(w, h, Bitmap.Config.ARGB_8888)
        for (y in 0 until h) {
            for (x in 0 until w) {
                val ca = a.getPixel(x, y)
                val cb = b.getPixel(x, y)
                if (ca == cb) {
                    out.setPixel(x, y, Color.argb(255, 20, 20, 20))
                } else {
                    out.setPixel(x, y, Color.MAGENTA)
                }
            }
        }
        return out
    }
}
