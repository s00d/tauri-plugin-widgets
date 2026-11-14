package git.s00d.widgets

import android.content.Context
import androidx.compose.runtime.Composable
import androidx.compose.ui.unit.dp
import androidx.glance.GlanceModifier
import androidx.glance.ImageProvider
import androidx.glance.LocalSize
import androidx.glance.appwidget.cornerRadius
import androidx.glance.background
import androidx.glance.semantics.contentDescription
import androidx.glance.semantics.semantics
import androidx.glance.layout.height
import androidx.glance.layout.padding
import androidx.glance.layout.width
import org.json.JSONObject
import java.util.Locale

@Composable
internal fun applyCommonStyle(context: Context, modifier: GlanceModifier, el: El): GlanceModifier {
    var m = modifier
    // Screen-reader text; `privacySensitive` has no home-screen redaction on Android.
    el.str("accessibilityLabel").takeIf { it.isNotEmpty() }?.let { label ->
        m = m.semantics { contentDescription = label }
    }

    val slotSize = runCatching { LocalSize.current }.getOrNull()
    val chrome = buildStyleChromeBitmap(context, el.raw, slotSize)
    if (chrome != null) {
        m = m.background(ImageProvider(chrome))
    } else {
        val bg = resolveBackgroundProvider(context, el.opt("background"))
        if (bg != null) m = m.background(bg)
    }

    val frame = el.obj("frame")
    if (frame != null || el.has(WireField.ASPECT_RATIO)) {
        val (w, h) = frameSizeDp(el)
        if (w > 0) m = m.width(w.toInt().dp)
        if (h > 0) m = m.height(h.toInt().dp)
    }

    m = paddingModifier(m, el.opt("padding"))

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
        // Glance clips only with a corner radius: an ellipse becomes a circle/capsule of
        // the short side (exact for square frames).
        clipShape == "ellipse" -> {
            val base = listOf(frameW, frameH, el.num("size", -1.0).toFloat())
                .filter { it > 0f }
                .minOrNull() ?: 24f
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

/** Number or per-edge `{top, leading, …}` → padding (shared by `padding` and `margin`). */
internal fun paddingModifier(modifier: GlanceModifier, p: Any?): GlanceModifier = when (p) {
    is Number -> modifier.padding(p.toInt().dp)
    is JSONObject -> modifier.padding(
        start = p.optDouble("leading", p.optDouble("start", 0.0)).toInt().dp,
        top = p.optDouble("top", 0.0).toInt().dp,
        end = p.optDouble("trailing", p.optDouble("end", 0.0)).toInt().dp,
        bottom = p.optDouble("bottom", 0.0).toInt().dp,
    )
    else -> modifier
}

/**
 * Fixed frame size in dp (-1 = unset). Glance has no min-size or aspect-ratio modifier:
 * `minWidth` / `minHeight` stand in for a missing width / height on intrinsic-less
 * elements (shape, image), and `aspectRatio` (w/h) derives the missing side from the other.
 */
internal fun frameSizeDp(el: El): Pair<Double, Double> {
    val frame = el.obj("frame")
    var w = frame?.optDouble("width", -1.0) ?: -1.0
    var h = frame?.optDouble("height", -1.0) ?: -1.0
    if (el.type == "shape" || el.type == "image") {
        if (w <= 0) w = frame?.optDouble("minWidth", -1.0) ?: -1.0
        if (h <= 0) h = frame?.optDouble("minHeight", -1.0) ?: -1.0
    }
    val ratio = el.num(WireField.ASPECT_RATIO, -1.0)
    if (ratio > 0) {
        if (w > 0 && h <= 0) h = w / ratio
        else if (h > 0 && w <= 0) w = h * ratio
    }
    return w to h
}
