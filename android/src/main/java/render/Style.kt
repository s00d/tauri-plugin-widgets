package git.s00d.widgets

import android.content.Context
import androidx.compose.runtime.Composable
import androidx.compose.ui.unit.dp
import androidx.glance.GlanceModifier
import androidx.glance.ImageProvider
import androidx.glance.LocalSize
import androidx.glance.appwidget.cornerRadius
import androidx.glance.background
import androidx.glance.layout.height
import androidx.glance.layout.padding
import androidx.glance.layout.width
import org.json.JSONObject
import java.util.Locale

@Composable
internal fun applyCommonStyle(context: Context, modifier: GlanceModifier, el: El): GlanceModifier {
    var m = modifier

    val slotSize = runCatching { LocalSize.current }.getOrNull()
    val chrome = buildStyleChromeBitmap(context, el.raw, slotSize)
    if (chrome != null) {
        m = m.background(ImageProvider(chrome))
    } else {
        val bg = resolveBackgroundProvider(context, el.opt("background"))
        if (bg != null) m = m.background(bg)
    }

    val frame = el.obj("frame")
    if (frame != null) {
        val w = frame.optDouble("width", -1.0)
        val h = frame.optDouble("height", -1.0)
        if (w > 0) m = m.width(w.toInt().dp)
        if (h > 0) m = m.height(h.toInt().dp)
    }

    val p = el.opt("padding")
    if (p is Number) {
        m = m.padding(p.toInt().dp)
    } else if (p is JSONObject) {
        val top = p.optDouble("top", 0.0).toInt().dp
        val bottom = p.optDouble("bottom", 0.0).toInt().dp
        val start = p.optDouble("leading", p.optDouble("start", 0.0)).toInt().dp
        val end = p.optDouble("trailing", p.optDouble("end", 0.0)).toInt().dp
        m = m.padding(start = start, top = top, end = end, bottom = bottom)
    }

    val explicitRadius = el.num("cornerRadius", -1.0).toFloat()
    val clipShape = el.str("clipShape", "").lowercase(Locale.US)
    val frameW = frame?.optDouble("width", -1.0)?.toFloat() ?: -1f
    val frameH = frame?.optDouble("height", -1.0)?.toFloat() ?: -1f
    val clipRadius = when {
        explicitRadius > 0f -> explicitRadius
        clipShape == "circle" -> {
            val base = listOf(frameW, frameH, el.num("size", -1.0).toFloat())
                .filter { it > 0f }
                .minOrNull() ?: 24f
            (base / 2f).coerceAtLeast(1f)
        }
        clipShape == "capsule" -> {
            val base = listOf(frameW, frameH, el.num("size", -1.0).toFloat())
                .filter { it > 0f }
                .minOrNull() ?: 28f
            (base / 2f).coerceAtLeast(1f)
        }
        clipShape == "rectangle" -> 0f
        else -> 0f
    }
    if (clipRadius > 0f) {
        m = m.cornerRadius(clipRadius.dp)
    }

    return m
}
