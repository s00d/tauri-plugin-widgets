package git.s00d.widgets

import android.content.Context
import android.net.Uri
import android.util.Log
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.unit.sp
import androidx.glance.layout.Alignment
import androidx.glance.layout.ContentScale
import androidx.glance.text.FontWeight
import androidx.glance.text.TextAlign
import androidx.glance.text.TextStyle
import androidx.glance.unit.ColorProvider
import org.json.JSONObject
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import java.util.TimeZone

/** Per-render luminance override — ThreadLocal so concurrent Glance paints don't cross-contaminate. */
internal val WIDGET_SURFACE_DARK = ThreadLocal<Boolean?>()

internal fun resolveBackgroundProvider(context: Context, value: Any?): ColorProvider? {
    if (value == null) return null
    if (value is String) return colorProviderFromString(context, value)
    if (value !is JSONObject) return null

    if (value.has("light") || value.has("dark")) {
        val isDark = surfaceIsDark(context)
        val pick = if (isDark) value.widgetString("dark", value.widgetString("light", "")) else value.widgetString("light", value.widgetString("dark", ""))
        return colorProviderFromString(context, pick)
    }
    if (value.has("colors")) {
        val arr = value.optJSONArray("colors")
        if (arr != null && arr.length() > 0) {
            // Glance has no gradient brush here — degrade to first stop explicitly.
            Log.d("TauriGlanceWidget", "background gradient degraded to first color stop (Glance limitation)")
            return colorProviderFromString(context, arr.optString(0, ""))
        }
    }
    return null
}

internal fun resolveColorProvider(context: Context, value: Any?): ColorProvider? {
    if (value == null) return null
    if (value is String) return colorProviderFromString(context, value)
    if (value is JSONObject) {
        if (value.has("light") || value.has("dark")) {
            val isDark = surfaceIsDark(context)
            val pick = if (isDark) value.widgetString("dark", value.widgetString("light", "")) else value.widgetString("light", value.widgetString("dark", ""))
            return colorProviderFromString(context, pick)
        }
    }
    return null
}

internal fun colorProviderFromString(context: Context, raw: String): ColorProvider? {
    val c = parseColor(context, raw) ?: return null
    return ColorProvider(c)
}

internal fun parseColor(context: Context, raw: String): Color? {
    val s0 = raw.trim()
    if (s0.isEmpty()) return null
    val semantic = semanticColor(context, s0)
    if (semantic != null) return semantic
    val rgba = Regex(
        """rgba?\(\s*([0-9.]+)\s*,\s*([0-9.]+)\s*,\s*([0-9.]+)(?:\s*,\s*([0-9.]+))?\s*\)""",
        RegexOption.IGNORE_CASE
    ).matchEntire(s0)
    if (rgba != null) {
        val r = rgba.groupValues[1].toFloat().coerceIn(0f, 255f) / 255f
        val g = rgba.groupValues[2].toFloat().coerceIn(0f, 255f) / 255f
        val b = rgba.groupValues[3].toFloat().coerceIn(0f, 255f) / 255f
        val a = rgba.groupValues.getOrNull(4)?.takeIf { it.isNotBlank() }?.toFloat()?.coerceIn(0f, 1f) ?: 1f
        return Color(r, g, b, a)
    }
    // Expand #RGB / #RGBA → full form (android.graphics.Color.parseColor rejects short hex).
    val s = expandShortHex(s0)
    return runCatching {
        when {
            s.startsWith("#") -> Color(android.graphics.Color.parseColor(s))
            s.equals("white", true) -> Color.White
            s.equals("black", true) -> Color.Black
            s.equals("red", true) -> Color(0xFFE53935)
            s.equals("green", true) -> Color(0xFF43A047)
            s.equals("blue", true) -> Color(0xFF1E88E5)
            s.equals("gray", true) || s.equals("grey", true) -> Color.Gray
            else -> Color(android.graphics.Color.parseColor(s))
        }
    }.getOrNull()
}

/** `#abc` → `#aabbcc`, `#abcd` → `#aabbccdd` (alpha last in CSS; Android wants AARRGGBB).
 *  Shared wire format is `#RRGGBBAA`; Android `Color.parseColor` expects `#AARRGGBB`. */
internal fun expandShortHex(raw: String): String {
    if (!raw.startsWith("#")) return raw
    val h = raw.substring(1)
    return when (h.length) {
        3 -> "#" + h.map { "$it$it" }.joinToString("")
        4 -> {
            // CSS #RGBA → Android #AARRGGBB
            val r = "${h[0]}${h[0]}"
            val g = "${h[1]}${h[1]}"
            val b = "${h[2]}${h[2]}"
            val a = "${h[3]}${h[3]}"
            "#$a$r$g$b"
        }
        8 -> {
            // Wire #RRGGBBAA → Android #AARRGGBB
            val rr = h.substring(0, 2)
            val gg = h.substring(2, 4)
            val bb = h.substring(4, 6)
            val aa = h.substring(6, 8)
            "#$aa$rr$gg$bb"
        }
        else -> raw
    }
}

internal fun semanticColor(context: Context, raw: String): Color? {
    val dark = surfaceIsDark(context)
    return when (raw.lowercase(Locale.US)) {
        "label" -> if (dark) Color(0xFFF2F2F7) else Color(0xFF111111)
        "secondarylabel" -> if (dark) Color(0xFFC7C7CC) else Color(0xFF555555)
        "separator" -> if (dark) Color(0xFF3A3A3C) else Color(0xFFD1D1D6)
        "systembackground" -> if (dark) Color(0xFF1C1C1E) else Color(0xFFFFFFFF)
        "accent", "systemblue" -> if (dark) Color(0xFF89B4FA) else Color(0xFF2563EB)
        "systemred" -> if (dark) Color(0xFFF87171) else Color(0xFFDC2626)
        "systemgreen" -> if (dark) Color(0xFF86EFAC) else Color(0xFF16A34A)
        "systemorange" -> if (dark) Color(0xFFFBBF24) else Color(0xFFEA580C)
        "systempurple" -> if (dark) Color(0xFFC4B5FD) else Color(0xFF7C3AED)
        else -> null
    }
}

internal fun isDarkMode(context: Context): Boolean {
    val mode = context.resources.configuration.uiMode and android.content.res.Configuration.UI_MODE_NIGHT_MASK
    return mode == android.content.res.Configuration.UI_MODE_NIGHT_YES
}

/** Prefer widget background luminance; fall back to system night mode. */
internal fun surfaceIsDark(context: Context): Boolean {
    return WIDGET_SURFACE_DARK.get() ?: isDarkMode(context)
}

internal fun inferSurfaceDark(context: Context, background: Any?): Boolean? {
    if (background == null || background === JSONObject.NULL) return null
    when (background) {
        is String -> {
            val s = background.trim()
            if (s.equals("null", true) || s.equals("undefined", true) || s.isEmpty()) return null
            if (s.equals("systemBackground", true) || s.equals("adaptive", true)) return null
            val c = parseColor(context, s) ?: return null
            return luminanceOf(c) < 0.45f
        }
        is JSONObject -> {
            // adaptive: { light, dark } — use light for harness luminance; production follows system
            val hex = background.widgetString("light", background.widgetString("dark", ""))
            if (hex.isBlank()) return null
            val c = parseColor(context, hex) ?: return null
            return luminanceOf(c) < 0.45f
        }
        else -> return null
    }
}

internal fun luminanceOf(color: Color): Float {
    val r = color.red
    val g = color.green
    val b = color.blue
    return 0.2126f * r + 0.7152f * g + 0.0722f * b
}

/** Track color ≈ tint @ 25% alpha over transparent (matches desktop tintTrack). */
internal fun trackColorFromTint(tintArgb: Int, surfaceDark: Boolean): Int {
    val a = 0x40 // ~25%
    val r = android.graphics.Color.red(tintArgb)
    val g = android.graphics.Color.green(tintArgb)
    val b = android.graphics.Color.blue(tintArgb)
    if (a > 0) {
        // Composite onto dark/light surface for opaque bitmap pixels
        val sr = if (surfaceDark) 0x0d else 0xe5
        val sg = if (surfaceDark) 0x11 else 0xe7
        val sb = if (surfaceDark) 0x17 else 0xeb
        val af = a / 255f
        val cr = (r * af + sr * (1 - af)).toInt().coerceIn(0, 255)
        val cg = (g * af + sg * (1 - af)).toInt().coerceIn(0, 255)
        val cb = (b * af + sb * (1 - af)).toInt().coerceIn(0, 255)
        return android.graphics.Color.rgb(cr, cg, cb)
    }
    return android.graphics.Color.rgb(r, g, b)
}

internal fun textStyleFromElement(context: Context, el: JSONObject): TextStyle {
    val color = resolveColorProvider(context, el.opt("color")) ?: semanticLabelProvider(context)
    val semantic = semanticTextSizeSp(el.widgetString("textStyle", ""))
    val explicit = el.optDouble("fontSize", -1.0)
    // textStyle overrides fontSize when both are set (models.rs contract)
    val size = when {
        semantic > 0f -> semantic.toDouble()
        explicit > 0 -> explicit
        else -> -1.0
    }
    val weight = when (el.widgetString("fontWeight", "").lowercase(Locale.US)) {
        "bold", "heavy", "black", "semibold", "600", "700", "800", "900" -> FontWeight.Bold
        "medium", "500" -> FontWeight.Medium
        else -> FontWeight.Normal
    }
    val align = when (el.widgetString("alignment", "").lowercase(Locale.US)) {
        "center", "middle" -> TextAlign.Center
        "trailing", "right", "end" -> TextAlign.End
        else -> TextAlign.Start
    }
    return if (size > 0) {
        TextStyle(color = color, fontSize = size.toFloat().sp, fontWeight = weight, textAlign = align)
    } else {
        TextStyle(color = color, fontWeight = weight, textAlign = align)
    }
}

internal fun semanticTextSizeSp(style: String): Float {
    return when (style.lowercase(Locale.US)) {
        "largeTitle".lowercase(Locale.US) -> 34f
        "title", "title1" -> 28f
        "title2" -> 22f
        "title3" -> 20f
        "headline" -> 17f
        "subheadline" -> 15f
        "body" -> 16f
        "callout" -> 16f
        "footnote" -> 13f
        "caption" -> 12f
        "caption2" -> 11f
        else -> -1f
    }
}

internal fun semanticLabelProvider(context: Context): ColorProvider {
    return ColorProvider(if (surfaceIsDark(context)) Color(0xFFF2F2F7) else Color(0xFF111111))
}

internal fun semanticSecondaryProvider(context: Context): ColorProvider {
    return ColorProvider(if (surfaceIsDark(context)) Color(0xFFC7C7CC) else Color(0xFF555555))
}

internal fun mediaGlyphFallback(label: String): String {
    return when (label.trim()) {
        "⏸", "❚❚", "pause" -> "||"
        "▶", "►", "play" -> ">"
        "⏮", "previous", "prev" -> "|<"
        "⏭", "next" -> ">|"
        "⏯" -> ">|"
        else -> label
    }
}

internal fun parseContentAlignment(raw: String): Alignment {
    return when (raw.lowercase(Locale.US)) {
        "center" -> Alignment.Center
        "top", "topcenter" -> Alignment.TopCenter
        "bottom", "bottomcenter" -> Alignment.BottomCenter
        "leading", "start", "centerstart", "centerleading" -> Alignment.CenterStart
        "trailing", "end", "centerend", "centertrailing" -> Alignment.CenterEnd
        "topleading", "topstart" -> Alignment.TopStart
        "toptrailing", "topend" -> Alignment.TopEnd
        "bottomleading", "bottomstart" -> Alignment.BottomStart
        "bottomtrailing", "bottomend" -> Alignment.BottomEnd
        else -> Alignment.TopStart
    }
}

internal fun parseHorizontalStackAlignment(raw: String): Alignment.Horizontal {
    return when (raw.lowercase(Locale.US)) {
        "center", "middle" -> Alignment.CenterHorizontally
        "trailing", "end", "right" -> Alignment.End
        else -> Alignment.Start
    }
}

internal fun parseVerticalStackAlignment(raw: String): Alignment.Vertical {
    return when (raw.lowercase(Locale.US)) {
        "top" -> Alignment.Top
        "bottom" -> Alignment.Bottom
        else -> Alignment.CenterVertically
    }
}

internal fun parseButtonContentAlignment(raw: String): Alignment {
    return when (raw.lowercase(Locale.US)) {
        "center", "middle" -> Alignment.Center
        "trailing", "right", "end" -> Alignment.CenterEnd
        else -> Alignment.CenterStart
    }
}

internal fun parseContentScale(mode: String): ContentScale {
    return when (mode.lowercase(Locale.US)) {
        "fill" -> ContentScale.Crop
        "fit" -> ContentScale.Fit
        "stretch" -> ContentScale.FillBounds
        else -> ContentScale.Fit
    }
}

internal fun progressBar(percent: Int): String {
    val clamped = percent.coerceIn(0, 100)
    val blocks = (clamped / 10).coerceIn(0, 10)
    return "[" + "#".repeat(blocks) + "-".repeat(10 - blocks) + "] $clamped%"
}

internal fun colorProviderArgb(provider: ColorProvider?, context: Context, fallback: Int): Int {
    val value: Any = provider?.getColor(context) ?: return fallback
    return when (value) {
        is Color -> value.toArgb()
        is Long -> Color(value).toArgb()
        is Int -> value
        else -> fallback
    }
}

internal fun formatDateValue(raw: String, style: String): String {
    val date = parseIsoDate(raw) ?: return raw
    val now = System.currentTimeMillis()
    return when (style.lowercase(Locale.US)) {
        "time" -> SimpleDateFormat("h:mm a", Locale.US).format(date)
        "relative" -> formatRelativeSpan(date.time - now)
        "offset" -> formatDuration(date.time - now)
        "timer" -> formatDuration(date.time - now)
        else -> SimpleDateFormat("yyyy-MM-dd", Locale.getDefault()).format(date)
    }
}

internal fun formatRelativeSpan(diffMs: Long): String {
    val past = diffMs < 0
    val abs = kotlin.math.abs(diffMs)
    val sec = abs / 1000
    val min = sec / 60
    val hr = min / 60
    val day = hr / 24
    val mon = day / 30
    val yr = day / 365
    val (n, unit) = when {
        yr >= 1L -> yr to if (yr == 1L) "yr" else "yrs"
        mon >= 1L -> mon to if (mon == 1L) "mo" else "mos"
        day >= 1L -> day to if (day == 1L) "day" else "days"
        hr >= 1L -> hr to if (hr == 1L) "hr" else "hrs"
        min >= 1L -> min to if (min == 1L) "min" else "mins"
        else -> sec to if (sec == 1L) "sec" else "secs"
    }
    return if (past) "$n $unit ago" else "in $n $unit"
}

internal fun formatTimerValue(raw: String, counting: String): String {
    val date = parseIsoDate(raw) ?: return raw
    val now = System.currentTimeMillis()
    val diff = if (counting.equals("up", true)) now - date.time else date.time - now
    return formatDuration(diff)
}

internal fun formatDuration(diffMs: Long): String {
    val sign = if (diffMs < 0) "-" else ""
    val sec = kotlin.math.abs(diffMs) / 1000
    val h = sec / 3600
    val m = (sec % 3600) / 60
    val s = sec % 60
    return "%s%02d:%02d:%02d".format(Locale.US, sign, h, m, s)
}

internal fun parseIsoDate(raw: String): Date? {
    if (raw.isBlank()) return null
    val patterns = listOf(
        "yyyy-MM-dd'T'HH:mm:ss.SSSX",
        "yyyy-MM-dd'T'HH:mm:ssX",
        "yyyy-MM-dd'T'HH:mmX",
        "yyyy-MM-dd"
    )
    for (p in patterns) {
        val sdf = SimpleDateFormat(p, Locale.US).apply { timeZone = TimeZone.getTimeZone("UTC") }
        val parsed = runCatching { sdf.parse(raw) }.getOrNull()
        if (parsed != null) return parsed
    }
    return raw.toLongOrNull()?.let { Date(it) }
}

internal fun safeHost(rawUrl: String): String {
    return runCatching { Uri.parse(rawUrl).host }.getOrNull().orEmpty().ifBlank { "remote" }
}

/** Material Symbols ligature for optional Typeface draw. */
internal fun cfgHash(text: String?): String {
    if (text.isNullOrEmpty()) return "null"
    return text.hashCode().toUInt().toString(16)
}

/**
 * org.json [JSONObject.optString] returns the literal "null" when the value is JSON null.
 * Treat JSON null / "null" / "undefined" as missing so they never render as text.
 */
internal fun JSONObject.widgetString(key: String, fallback: String = ""): String {
    if (!has(key) || isNull(key)) return fallback
    val raw = opt(key) ?: return fallback
    if (raw === JSONObject.NULL) return fallback
    val s = when (raw) {
        is String -> raw
        else -> raw.toString()
    }.trim()
    if (s.isEmpty() || s.equals("null", ignoreCase = true) || s.equals("undefined", ignoreCase = true)) {
        return fallback
    }
    return s
}

internal fun dpToPx(context: Context, dp: Float): Int {
    return (dp * context.resources.displayMetrics.density).toInt().coerceAtLeast(1)
}
