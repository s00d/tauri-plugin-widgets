package git.s00d.widgets

import android.content.Context
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.Canvas
import android.graphics.LinearGradient
import android.graphics.Matrix
import android.graphics.Paint
import android.graphics.Path
import android.graphics.RadialGradient
import android.graphics.RectF
import android.graphics.Shader
import android.graphics.SweepGradient
import android.graphics.Typeface
import android.net.Uri
import android.util.Base64
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.unit.DpSize
import androidx.core.graphics.PathParser
import androidx.glance.ImageProvider
import androidx.glance.appwidget.ImageProvider
import org.json.JSONObject
import java.util.Collections
import java.util.Locale
import kotlin.math.roundToInt

// accessOrder LRU — all get/put/evict must run under the same lock
internal val BASE64_CACHE: MutableMap<Int, Bitmap> =
    Collections.synchronizedMap(LinkedHashMap(64, 0.75f, true))

/**
 * Bake gradient / border / soft shadow into a bitmap background for Glance
 * (no live gradient brush or shadow APIs). Uses [slotSize] when the IR has no frame.
 */
internal fun buildStyleChromeBitmap(
    context: Context,
    el: JSONObject,
    slotSize: DpSize? = null,
): Bitmap? {
    val bg = el.opt("background")
    val border = el.optJSONObject("border")
    val shadow = el.optJSONObject("shadow")
    val isGradient = bg is JSONObject && (bg.has("colors") || bg.has("stops"))
    if (!isGradient && border == null && shadow == null) return null

    val density = context.resources.displayMetrics.density
    val frame = el.optJSONObject("frame")
    val slotW = slotSize?.width?.value?.let { (it * density).roundToInt() }
    val slotH = slotSize?.height?.value?.let { (it * density).roundToInt() }
    val baseW = when {
        frame != null && frame.optDouble("width", 0.0) > 0 ->
            frame.optDouble("width").toInt().coerceAtLeast(48)
        slotW != null && slotW > 0 -> slotW.coerceAtLeast(48)
        else -> 200
    }
    val baseH = when {
        frame != null && frame.optDouble("height", 0.0) > 0 ->
            frame.optDouble("height").toInt().coerceAtLeast(48)
        slotH != null && slotH > 0 -> slotH.coerceAtLeast(48)
        else -> 200
    }
    val shadowPad = if (shadow != null) {
        ((shadow.optDouble("radius", 8.0) + kotlin.math.abs(shadow.optDouble("y", 4.0))).toInt() + 4)
            .coerceIn(4, 24)
    } else 0
    val w = baseW + shadowPad * 2
    val h = baseH + shadowPad * 2
    val bmp = Bitmap.createBitmap(w, h, Bitmap.Config.ARGB_8888)
    val canvas = Canvas(bmp)
    val radius = el.optDouble("cornerRadius", 0.0).toFloat().coerceAtLeast(0f)
    val content = RectF(
        shadowPad.toFloat(),
        shadowPad.toFloat(),
        (w - shadowPad).toFloat(),
        (h - shadowPad).toFloat()
    )

    if (shadow != null) {
        val sx = shadow.optDouble("x", 0.0).toFloat()
        val sy = shadow.optDouble("y", 4.0).toFloat()
        val shadowColor = parseColor(context, shadow.widgetString("color", "rgba(0,0,0,0.35)"))
            ?.toArgb() ?: Color(0x59000000).toArgb()
        val shadowRect = RectF(
            content.left + sx,
            content.top + sy,
            content.right + sx,
            content.bottom + sy
        )
        canvas.drawRoundRect(
            shadowRect,
            radius,
            radius,
            Paint(Paint.ANTI_ALIAS_FLAG).apply {
                style = Paint.Style.FILL
                color = shadowColor
            }
        )
    }

    val fillPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply { style = Paint.Style.FILL }
    when {
        isGradient && bg is JSONObject -> {
            val (stops, positions) = gradientStops(context, bg) ?: return null
            val pos = positions?.toFloatArray()
            val gType = bg.widgetString("gradientType", "linear").lowercase(Locale.US)
            val sp = bg.optJSONObject("startPoint")
            val ep = bg.optJSONObject("endPoint")
            fillPaint.shader = when {
                gType == "radial" -> {
                    val cx = content.centerX()
                    val cy = content.centerY()
                    val r = maxOf(content.width(), content.height()) / 2f
                    RadialGradient(cx, cy, r.coerceAtLeast(1f), stops.toIntArray(), pos, Shader.TileMode.CLAMP)
                }
                gType == "angular" -> SweepGradient(content.centerX(), content.centerY(), stops.toIntArray(), pos)
                sp != null && ep != null -> LinearGradient(
                    content.left + content.width() * sp.optDouble("x", 0.0).toFloat(),
                    content.top + content.height() * sp.optDouble("y", 0.0).toFloat(),
                    content.left + content.width() * ep.optDouble("x", 0.0).toFloat(),
                    content.top + content.height() * ep.optDouble("y", 1.0).toFloat(),
                    stops.toIntArray(),
                    pos,
                    Shader.TileMode.CLAMP,
                )
                else -> {
                    val dir = bg.widgetString("direction", "topToBottom").lowercase(Locale.US)
                    val (x0, y0, x1, y1) = when (dir) {
                        "bottomtotop" -> listOf(content.left, content.bottom, content.left, content.top)
                        "leadingtotrailing" -> listOf(content.left, content.top, content.right, content.top)
                        "trailingtoleading" -> listOf(content.right, content.top, content.left, content.top)
                        "topleadingtobottomtrailing" -> listOf(content.left, content.top, content.right, content.bottom)
                        "toptrailingtobottomleading" -> listOf(content.right, content.top, content.left, content.bottom)
                        else -> listOf(content.left, content.top, content.left, content.bottom)
                    }
                    LinearGradient(
                        x0, y0, x1, y1,
                        stops.toIntArray(),
                        pos,
                        Shader.TileMode.CLAMP
                    )
                }
            }
            canvas.drawRoundRect(content, radius, radius, fillPaint)
        }
        bg is String -> {
            val c = parseColor(context, bg)?.toArgb() ?: return null
            fillPaint.color = c
            canvas.drawRoundRect(content, radius, radius, fillPaint)
        }
        bg is JSONObject && (bg.has("light") || bg.has("dark")) -> {
            val pick = if (surfaceIsDark(context)) {
                bg.widgetString("dark", bg.widgetString("light", ""))
            } else {
                bg.widgetString("light", bg.widgetString("dark", ""))
            }
            val c = parseColor(context, pick)?.toArgb() ?: return null
            fillPaint.color = c
            canvas.drawRoundRect(content, radius, radius, fillPaint)
        }
        else -> {
            // border/shadow only — keep transparent fill
        }
    }

    if (border != null) {
        val bw = border.optDouble("width", 1.0).toFloat().coerceAtLeast(1f)
        val bc = parseColor(context, border.widgetString("color", "#FFFFFF"))?.toArgb()
            ?: Color.White.toArgb()
        val inset = bw / 2f
        val strokeRect = RectF(
            content.left + inset,
            content.top + inset,
            content.right - inset,
            content.bottom - inset
        )
        canvas.drawRoundRect(
            strokeRect,
            (radius - inset).coerceAtLeast(0f),
            (radius - inset).coerceAtLeast(0f),
            Paint(Paint.ANTI_ALIAS_FLAG).apply {
                style = Paint.Style.STROKE
                strokeWidth = bw
                color = bc
            }
        )
    }
    return bmp
}

/**
 * Gradient colors and optional positions: `stops` (`{color, location}`) replace `colors`.
 * Positions are sorted and clamped to 0…1 (Android shaders need them increasing).
 */
internal fun gradientStops(context: Context, bg: JSONObject): Pair<List<Int>, List<Float>?>? {
    val stopsArr = bg.optJSONArray("stops")
    if (stopsArr != null && stopsArr.length() > 0) {
        val pairs = (0 until stopsArr.length()).mapNotNull { i ->
            val o = stopsArr.optJSONObject(i) ?: return@mapNotNull null
            val c = parseColor(context, o.widgetString("color", ""))?.toArgb() ?: return@mapNotNull null
            o.optDouble("location", 0.0).toFloat().coerceIn(0f, 1f) to c
        }.sortedBy { it.first }
        if (pairs.isNotEmpty()) {
            val list = if (pairs.size == 1) pairs + pairs else pairs
            return list.map { it.second } to list.map { it.first }
        }
    }
    val arr = bg.optJSONArray("colors") ?: return null
    val colors = (0 until arr.length()).mapNotNull { parseColor(context, arr.optString(it, ""))?.toArgb() }
    if (colors.isEmpty()) return null
    return (if (colors.size == 1) colors + colors else colors) to null
}

/** Longest side of a decoded widget image. RemoteViews reject bitmaps above the
 *  screen-size memory budget, and a full-resolution photo is far above it. */
private const val MAX_IMAGE_PX = 1024

/** Options with `inSampleSize` so the decoded bitmap stays within [MAX_IMAGE_PX]. */
private fun boundedOptions(probe: (BitmapFactory.Options) -> Unit): BitmapFactory.Options {
    val bounds = BitmapFactory.Options().apply { inJustDecodeBounds = true }
    probe(bounds)
    var sample = 1
    while (bounds.outWidth / (sample * 2) >= MAX_IMAGE_PX || bounds.outHeight / (sample * 2) >= MAX_IMAGE_PX) {
        sample *= 2
    }
    return BitmapFactory.Options().apply { inSampleSize = sample }
}

private fun decodeBoundedFile(path: String): Bitmap? =
    BitmapFactory.decodeFile(path, boundedOptions { BitmapFactory.decodeFile(path, it) })

internal fun imageProviderFromElement(context: Context, el: JSONObject): ImageProvider? {
    val localPath = el.widgetString("localPath", "")
    if (localPath.isNotBlank()) {
        // Plugin-cached download: a content URI the launcher decodes itself, so a photo does
        // not count against the RemoteViews bitmap budget. Bitmap when no provider / grant.
        cachedImageUri(context, localPath)?.let { return ImageProvider(it) }
        decodeBoundedFile(localPath)?.let { return ImageProvider(it) }
    }

    val data = el.widgetString("data", "")
    if (data.isNotBlank()) {
        decodeBase64Bitmap(data)?.let { return ImageProvider(clipImageBitmap(context, el, it)) }
    }

    val url = el.widgetString("url", "")
    if (url.startsWith("data:", true)) {
        decodeBase64Bitmap(url)?.let { return ImageProvider(clipImageBitmap(context, el, it)) }
    } else if (url.startsWith("file://")) {
        val filePath = Uri.parse(url).path.orEmpty()
        if (filePath.isNotBlank()) {
            decodeBoundedFile(filePath)?.let { return ImageProvider(clipImageBitmap(context, el, it)) }
        }
    } else if (url.startsWith("/")) {
        decodeBoundedFile(url)?.let { return ImageProvider(clipImageBitmap(context, el, it)) }
    }

    val systemName = el.widgetString("systemName", "")
    if (systemName.isNotBlank()) {
        resolveSystemImageProvider(context, systemName)?.let { return it }
    }

    return null
}

/**
 * `clipShape` / `cornerRadius` baked into an image bitmap at its `size` (dp), scaled per
 * `contentMode` (fill = center crop). Glance clips only through the view outline, which
 * needs API 31+ and a hardware canvas; a baked mask holds everywhere. Unclipped → [src].
 */
internal fun clipImageBitmap(context: Context, el: JSONObject, src: Bitmap): Bitmap {
    val shape = el.widgetString("clipShape", "").lowercase(Locale.US)
    val radiusDp = el.optDouble("cornerRadius", 0.0).toFloat()
    if (shape !in setOf("circle", "ellipse", "capsule") && radiusDp <= 0f) return src
    val s = dpToPx(context, el.optDouble("size", 24.0).toFloat())
    val out = Bitmap.createBitmap(s, s, Bitmap.Config.ARGB_8888)
    val canvas = Canvas(out)
    val box = RectF(0f, 0f, s.toFloat(), s.toFloat())
    val r = if (radiusDp > 0f && shape !in setOf("circle", "ellipse", "capsule")) radiusDp * context.resources.displayMetrics.density else s / 2f
    canvas.clipPath(Path().apply { addRoundRect(box, r, r, Path.Direction.CW) })
    val fill = el.widgetString("contentMode", "fit").equals("fill", true)
    val scale = if (fill) maxOf(s.toFloat() / src.width, s.toFloat() / src.height) else minOf(s.toFloat() / src.width, s.toFloat() / src.height)
    val w = src.width * scale
    val h = src.height * scale
    canvas.drawBitmap(src, null, RectF((s - w) / 2f, (s - h) / 2f, (s + w) / 2f, (s + h) / 2f), Paint(Paint.FILTER_BITMAP_FLAG))
    return out
}

internal fun resolveSystemImageProvider(context: Context, systemName: String): ImageProvider? =
    systemIconResId(context, systemName, mipmap = true).takeIf { it != 0 }?.let { ImageProvider(it) }

/**
 * Drawable id for a symbol name (`play.fill` → `play.fill`, `play_fill`, …) in the app, then
 * the framework; 0 when none. [mipmap] also searches app mipmaps (not for notification icons).
 */
@android.annotation.SuppressLint("DiscouragedApi") // the resource lives in the app, looked up by name
internal fun systemIconResId(context: Context, systemName: String, mipmap: Boolean = false): Int {
    val candidates = mutableListOf<String>()
    val normalized = systemName.trim()
    if (normalized.isNotBlank()) {
        candidates += normalized
        candidates += normalized.replace('.', '_')
        candidates += normalized.replace('-', '_')
        candidates += normalized.replace('.', '_').replace('-', '_')
    }

    for (name in candidates.distinct()) {
        val appDrawable = context.resources.getIdentifier(name, "drawable", context.packageName)
        if (appDrawable != 0) return appDrawable
        if (mipmap) {
            val appMipmap = context.resources.getIdentifier(name, "mipmap", context.packageName)
            if (appMipmap != 0) return appMipmap
        }

        val androidDrawable = context.resources.getIdentifier(name, "drawable", "android")
        if (androidDrawable != 0) return androidDrawable
    }
    return 0
}

internal fun decodeBase64Bitmap(raw: String): Bitmap? {
    val key = raw.hashCode()
    synchronized(BASE64_CACHE) {
        BASE64_CACHE[key]?.let { return it }
    }

    val normalized = if (raw.startsWith("data:", true)) {
        raw.substringAfter(',', missingDelimiterValue = "").trim()
    } else {
        raw.trim()
    }
    if (normalized.isEmpty()) return null
    val bytes = runCatching {
        Base64.decode(normalized, Base64.DEFAULT)
    }.recoverCatching {
        Base64.decode(normalized, Base64.URL_SAFE or Base64.NO_WRAP)
    }.getOrNull() ?: return null
    val bmp = BitmapFactory.decodeByteArray(bytes, 0, bytes.size, boundedOptions { BitmapFactory.decodeByteArray(bytes, 0, bytes.size, it) }) ?: return null
    // Upscale tiny (1×1) fixtures so Glance Image sizing is visible.
    val out = if (bmp.width <= 2 || bmp.height <= 2) {
        Bitmap.createScaledBitmap(bmp, 32.coerceAtLeast(bmp.width * 16), 32.coerceAtLeast(bmp.height * 16), false)
    } else bmp
    synchronized(BASE64_CACHE) {
        BASE64_CACHE[key] = out
        while (BASE64_CACHE.size > 80) {
            val first = BASE64_CACHE.keys.firstOrNull() ?: break
            BASE64_CACHE.remove(first)
        }
    }
    return out
}

private class ChartPt(val value: Float, val color: Int, val explicit: Int?, val label: String, val series: String)

/** Colors for series after the first (which takes `tint`), when a series has no point color. */
private val CHART_PALETTE = intArrayOf(
    0xFF4FC3F7.toInt(), 0xFFF472B6.toInt(), 0xFF34D399.toInt(), 0xFFFBBF24.toInt(), 0xFFA78BFA.toInt(), 0xFFF87171.toInt(),
)

private fun axisNumber(v: Float): String =
    if (v == v.toLong().toFloat()) v.toLong().toString() else String.format(Locale.US, "%.1f", v)

/**
 * Chart bitmap (Glance has no chart). Types bar / line / area / point / pie (donut with
 * `innerRadius`); `series` groups points into several lines or grouped bars; `yMin`/`yMax`
 * fix the scale; `interpolation` linear / monotone / step; axes and legend are drawn
 * inside the same bitmap. x labels on v2 charts unless `showXAxis: false` (legacy: only `true`).
 */
/** Any v2 chart option (as `usesSwiftCharts` in RenderData.swift / `greedy` in layout.ts). */
internal fun isV2Chart(el: JSONObject): Boolean {
    if (el.optString("chartType") == "point") return true
    if (listOf("showXAxis", "showYAxis", "showLegend", "yMin", "yMax", "interpolation", "innerRadius").any { el.has(it) && !el.isNull(it) }) return true
    val data = el.optJSONArray("chartData") ?: return false
    return (0 until data.length()).any { data.optJSONObject(it)?.let { p -> p.has("series") && !p.isNull("series") } == true }
}

internal fun drawChartBitmap(context: Context, el: JSONObject, slotWDp: Float = 0f, slotHDp: Float = 0f): Bitmap? {
    val data = el.optJSONArray("chartData") ?: return null
    if (data.length() == 0) return null

    val tint = resolveColorProvider(context, el.opt("tint"))
    val points = mutableListOf<ChartPt>()
    for (i in 0 until data.length()) {
        val p = data.optJSONObject(i) ?: continue
        val value = p.optDouble("value", 0.0).toFloat()
        val pointColorProvider = resolveColorProvider(context, p.opt("color"))
        val pointColor = pointColorProvider?.let { colorProviderArgb(it, context, Color.Transparent.toArgb()) }
        val fallback = pointColor ?: colorProviderArgb(tint, context, Color(0xFF4FC3F7).toArgb())
        points += ChartPt(value, fallback, pointColor, p.widgetString("label", ""), p.widgetString("series", ""))
    }
    if (points.isEmpty()) return null

    val chartType = el.widgetString("chartType", "bar").lowercase(Locale.US)
    val frame = el.optJSONObject("frame")
    val widthDp = if (chartType == "pie") {
        frame?.optDouble("width", 80.0)?.toFloat() ?: 80f
    } else if (slotWDp > 0f) {
        slotWDp
    } else {
        frame?.optDouble("width", 180.0)?.toFloat() ?: 180f
    }
    val heightDp = if (chartType == "pie") {
        widthDp
    } else if (slotHDp > 0f) {
        slotHDp
    } else {
        frame?.optDouble("height", 84.0)?.toFloat() ?: 84f
    }
    val w = dpToPx(context, widthDp).coerceAtLeast(if (chartType == "pie") 64 else 80)
    val h = dpToPx(context, heightDp).coerceAtLeast(if (chartType == "pie") 64 else 44)
    val bmp = Bitmap.createBitmap(w, h, Bitmap.Config.ARGB_8888)
    val canvas = Canvas(bmp)

    val bgProvider = resolveBackgroundProvider(context, el.opt("background"))
    if (bgProvider != null) {
        canvas.drawColor(colorProviderArgb(bgProvider, context, Color.Transparent.toArgb()))
    }

    // Series in order of first appearance; a point without `series` belongs to "".
    val seriesMap = LinkedHashMap<String, MutableList<ChartPt>>()
    for (p in points) seriesMap.getOrPut(p.series) { mutableListOf() } += p
    val series = seriesMap.entries.toList()
    val multi = series.size > 1
    val seriesColors = series.mapIndexed { i, (_, pts) ->
        pts.firstNotNullOfOrNull { it.explicit }
            ?: if (i == 0) colorProviderArgb(tint, context, CHART_PALETTE[0]) else CHART_PALETTE[i % CHART_PALETTE.size]
    }

    val isPie = chartType == "pie"
    // x labels: Swift Charts / desktop show them on v2 charts unless `showXAxis: false`;
    // legacy charts only with `showXAxis: true`.
    val showX = !isPie && el.optBoolean("showXAxis", isV2Chart(el))
    val showY = !isPie && el.optBoolean("showYAxis", false)
    val legend: List<Pair<String, Int>> = when {
        !el.optBoolean("showLegend", false) -> emptyList()
        isPie -> points.map { it.label to it.color }.filter { it.first.isNotBlank() }
        else -> series.mapIndexed { i, (name, _) -> name to seriesColors[i] }.filter { it.first.isNotBlank() }
    }
    val ink = colorProviderArgb(semanticSecondaryProvider(context), context, Color.Gray.toArgb())
    val textPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply { color = ink; textSize = dpToPx(context, 9f).toFloat() }

    val yMinOpt = if (el.has("yMin") && !el.isNull("yMin")) el.optDouble("yMin").toFloat() else null
    val yMaxOpt = if (el.has("yMax") && !el.isNull("yMax")) el.optDouble("yMax").toFloat() else null
    val dataMax = points.maxOf { it.value }
    val dataMin = points.minOf { it.value }
    val (lo, hi) = when (chartType) {
        "line", "area", "point" -> (yMinOpt ?: dataMin) to (yMaxOpt ?: dataMax.coerceAtLeast(0.0001f))
        else -> (yMinOpt ?: minOf(0f, dataMin)) to (yMaxOpt ?: dataMax.coerceAtLeast(0.0001f))
    }
    val span = (hi - lo).takeIf { it > 0.0001f } ?: 1f

    val pad = dpToPx(context, 6f).toFloat()
    val textH = textPaint.textSize * 1.4f
    val legendH = if (legend.isNotEmpty()) textH else 0f
    val xAxisH = if (showX) textH else 0f
    val yGutter = if (showY) maxOf(textPaint.measureText(axisNumber(hi)), textPaint.measureText(axisNumber(lo))) + dpToPx(context, 4f) else 0f
    val chartRect = RectF(pad + yGutter, pad, w - pad, h - pad - xAxisH - legendH)
    if (isPie && legendH > 0f) {
        // Keep the pie round above the legend row.
        val side = minOf(chartRect.width(), chartRect.height())
        chartRect.set(chartRect.centerX() - side / 2, chartRect.top, chartRect.centerX() + side / 2, chartRect.top + side)
    }
    fun yOf(v: Float) = chartRect.bottom - ((v.coerceIn(lo, hi) - lo) / span) * chartRect.height()
    val maxLen = series.maxOf { it.value.size }
    fun xOf(i: Int) = chartRect.left + chartRect.width() * (i.toFloat() / (maxLen - 1).coerceAtLeast(1))

    when (chartType) {
        "line", "area", "point" -> {
            val interpolation = el.widgetString("interpolation", "")
            series.forEachIndexed { si, (_, pts) ->
                val color = if (multi) seriesColors[si] else pts.first().color
                val xs = FloatArray(pts.size) { xOf(it) }
                val ys = FloatArray(pts.size) { yOf(pts[it].value) }
                if (chartType != "point" && pts.size > 1) {
                    val stroke = Paint(Paint.ANTI_ALIAS_FLAG).apply {
                        style = Paint.Style.STROKE
                        this.color = color
                        strokeWidth = dpToPx(context, 2.5f).toFloat()
                        strokeJoin = Paint.Join.ROUND
                        strokeCap = Paint.Cap.ROUND
                    }
                    val path = Path()
                    val area = Path()
                    path.moveTo(xs[0], ys[0])
                    area.moveTo(xs[0], chartRect.bottom)
                    area.lineTo(xs[0], ys[0])
                    for (i in 0 until pts.size - 1) {
                        val x1 = xs[i]; val y1 = ys[i]; val x2 = xs[i + 1]; val y2 = ys[i + 1]
                        when {
                            interpolation == "step" -> {
                                path.lineTo(x2, y1); path.lineTo(x2, y2)
                                area.lineTo(x2, y1); area.lineTo(x2, y2)
                            }
                            interpolation == "linear" || pts.size == 2 -> {
                                path.lineTo(x2, y2); area.lineTo(x2, y2)
                            }
                            else -> {
                                // Horizontal tangents at each point: smooth, never overshoots (monotone).
                                val cx1 = x1 + (x2 - x1) / 3f
                                val cx2 = x1 + 2f * (x2 - x1) / 3f
                                path.cubicTo(cx1, y1, cx2, y2, x2, y2)
                                area.cubicTo(cx1, y1, cx2, y2, x2, y2)
                            }
                        }
                    }
                    if (chartType == "area") {
                        area.lineTo(xs.last(), chartRect.bottom)
                        area.close()
                        canvas.drawPath(area, Paint(Paint.ANTI_ALIAS_FLAG).apply {
                            style = Paint.Style.FILL
                            this.color = (color and 0x55FFFFFF)
                        })
                    }
                    canvas.drawPath(path, stroke)
                }
                // Point markers (the whole chart for `point`).
                val dot = Paint(Paint.ANTI_ALIAS_FLAG).apply { style = Paint.Style.FILL }
                val r = dpToPx(context, if (chartType == "point") 3.5f else 2.5f).toFloat()
                for (i in pts.indices) {
                    dot.color = if (multi) color else pts[i].color
                    canvas.drawCircle(xs[i], ys[i], r, dot)
                }
            }
        }
        "pie" -> {
            val total = points.sumOf { it.value.toDouble() }.toFloat().coerceAtLeast(0.0001f)
            var start = -90f
            for (p in points) {
                val sweep = (p.value / total) * 360f
                val paint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
                    style = Paint.Style.FILL
                    this.color = p.color
                }
                canvas.drawArc(chartRect, start, sweep, true, paint)
                start += sweep
            }
            val inner = el.optDouble("innerRadius", 0.0).toFloat().coerceIn(0f, 0.95f)
            if (inner > 0f) {
                canvas.drawCircle(
                    chartRect.centerX(), chartRect.centerY(), chartRect.width() / 2f * inner,
                    Paint(Paint.ANTI_ALIAS_FLAG).apply { xfermode = android.graphics.PorterDuffXfermode(android.graphics.PorterDuff.Mode.CLEAR) },
                )
                bgProvider?.let {
                    // Re-fill the hole with the chart background so it is not see-through.
                    canvas.drawCircle(
                        chartRect.centerX(), chartRect.centerY(), chartRect.width() / 2f * inner,
                        Paint(Paint.ANTI_ALIAS_FLAG).apply { color = colorProviderArgb(it, context, Color.Transparent.toArgb()) },
                    )
                }
            }
        }
        else -> {
            // Scale from 0 (or min(0, dataMin)) so the smallest positive bar isn't zero-height.
            val gap = dpToPx(context, 3f).toFloat()
            val groupW = ((chartRect.width() - gap * (maxLen - 1)) / maxLen).coerceAtLeast(1f)
            val inner = if (multi) dpToPx(context, 1f).toFloat() else 0f
            val barW = ((groupW - inner * (series.size - 1)) / series.size).coerceAtLeast(1f)
            series.forEachIndexed { si, (_, pts) ->
                for (i in pts.indices) {
                    val x = chartRect.left + i * (groupW + gap) + si * (barW + inner)
                    val y = yOf(pts[i].value)
                    val paint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
                        style = Paint.Style.FILL
                        color = if (multi) seriesColors[si] else pts[i].color
                    }
                    canvas.drawRoundRect(RectF(x, y, x + barW, chartRect.bottom), barW / 4f, barW / 4f, paint)
                }
            }
        }
    }

    if (showY) {
        textPaint.textAlign = Paint.Align.RIGHT
        val x = chartRect.left - dpToPx(context, 3f)
        canvas.drawText(axisNumber(hi), x, chartRect.top - textPaint.ascent(), textPaint)
        canvas.drawText(axisNumber(lo), x, chartRect.bottom, textPaint)
        canvas.drawLine(chartRect.left, chartRect.top, chartRect.left, chartRect.bottom, Paint(textPaint).apply { alpha = 0x60 })
    }
    if (showX) {
        textPaint.textAlign = Paint.Align.CENTER
        val labels = series.maxByOrNull { it.value.size }!!.value
        val y = chartRect.bottom + textH - textPaint.descent()
        val gap = dpToPx(context, 3f).toFloat()
        val groupW = (chartRect.width() - gap * (maxLen - 1)) / maxLen
        labels.forEachIndexed { i, p ->
            val x = if (chartType == "bar" || chartType !in setOf("line", "area", "point")) {
                chartRect.left + i * (groupW + gap) + groupW / 2f
            } else xOf(i)
            canvas.drawText(p.label, x.coerceIn(textPaint.textSize, w - textPaint.textSize), y, textPaint)
        }
    }
    if (legend.isNotEmpty()) {
        textPaint.textAlign = Paint.Align.LEFT
        val y = h - pad - textPaint.descent()
        val r = textPaint.textSize / 3f
        var x = pad + yGutter
        val dot = Paint(Paint.ANTI_ALIAS_FLAG)
        for ((name, color) in legend) {
            if (x > w - pad) break
            dot.color = color
            canvas.drawCircle(x + r, y - textPaint.textSize / 3f, r, dot)
            x += r * 2 + dpToPx(context, 3f)
            canvas.drawText(name, x, y, textPaint)
            x += textPaint.measureText(name) + dpToPx(context, 8f)
        }
    }
    return bmp
}

internal fun drawCanvasBitmap(context: Context, el: JSONObject): Bitmap? {
    val commands = el.optJSONArray("elements") ?: return null
    if (commands.length() == 0) return null
    val density = context.resources.displayMetrics.density
    fun sx(v: Double): Float = (v * density).toFloat()
    val width = dpToPx(context, el.optDouble("width", 180.0).toFloat()).coerceAtLeast(40)
    val height = dpToPx(context, el.optDouble("height", 100.0).toFloat()).coerceAtLeast(30)
    val bmp = Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888)
    val canvas = Canvas(bmp)

    val bg = resolveBackgroundProvider(context, el.opt("background"))?.getColor(context)
    if (bg != null) canvas.drawColor(bg.toArgb())

    for (i in 0 until commands.length()) {
        val cmd = commands.optJSONObject(i) ?: continue
        val draw = cmd.widgetString("draw", "")
        when (draw) {
            "circle" -> {
                val cx = sx(cmd.optDouble("cx", 0.0))
                val cy = sx(cmd.optDouble("cy", 0.0))
                val r = sx(cmd.optDouble("r", 0.0))
                val fill = paintFill(context, cmd.opt("fill"), "#000000")
                canvas.drawCircle(cx, cy, r, fill)
                val strokeRaw = cmd.opt("stroke")
                if (strokeRaw != null) {
                    val stroke = paintStroke(
                        context,
                        strokeRaw,
                        sx(cmd.optDouble("strokeWidth", 1.0)),
                        "#000000"
                    )
                    if (stroke != null) canvas.drawCircle(cx, cy, r, stroke)
                }
            }
            "line" -> {
                val p = paintStroke(
                    context,
                    cmd.opt("stroke"),
                    sx(cmd.optDouble("strokeWidth", 1.0)),
                    "#000000"
                )
                if (p == null) continue
                canvas.drawLine(
                    sx(cmd.optDouble("x1", 0.0)),
                    sx(cmd.optDouble("y1", 0.0)),
                    sx(cmd.optDouble("x2", 0.0)),
                    sx(cmd.optDouble("y2", 0.0)),
                    p
                )
            }
            "rect" -> {
                val x = sx(cmd.optDouble("x", 0.0))
                val y = sx(cmd.optDouble("y", 0.0))
                val w = sx(cmd.optDouble("width", 0.0))
                val h = sx(cmd.optDouble("height", 0.0))
                val rr = sx(cmd.optDouble("cornerRadius", 0.0))
                val rect = RectF(x, y, x + w, y + h)
                val fill = paintFill(context, cmd.opt("fill"), "#000000")
                canvas.drawRoundRect(rect, rr, rr, fill)
                val strokeRaw = cmd.opt("stroke")
                if (strokeRaw != null) {
                    val stroke = paintStroke(
                        context,
                        strokeRaw,
                        sx(cmd.optDouble("strokeWidth", 1.0)),
                        "#000000"
                    )
                    if (stroke != null) canvas.drawRoundRect(rect, rr, rr, stroke)
                }
            }
            "arc" -> {
                val cx = sx(cmd.optDouble("cx", 0.0))
                val cy = sx(cmd.optDouble("cy", 0.0))
                val r = sx(cmd.optDouble("r", 0.0))
                val start = cmd.optDouble("startAngle", 0.0).toFloat()
                val end = cmd.optDouble("endAngle", 0.0).toFloat()
                val rect = RectF(cx - r, cy - r, cx + r, cy + r)
                val strokeW = sx(cmd.optDouble("strokeWidth", 1.0))
                val stroke = paintStroke(context, cmd.opt("stroke"), strokeW, null)
                val fallbackStroke = if (stroke == null) {
                    paintStroke(context, cmd.opt("fill"), strokeW, "#000000")
                } else stroke
                if (fallbackStroke == null) continue
                canvas.drawArc(rect, start, end - start, false, fallbackStroke)
            }
            "text" -> {
                val paint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
                    style = Paint.Style.FILL
                    color = colorProviderArgb(resolveColorProvider(context, cmd.opt("color")), context, Color.Black.toArgb())
                    textSize = sx(cmd.optDouble("fontSize", 12.0))
                }
                canvas.drawText(
                    cmd.widgetString("content", ""),
                    sx(cmd.optDouble("x", 0.0)),
                    sx(cmd.optDouble("y", 0.0)),
                    paint
                )
            }
            "path" -> {
                val d = cmd.widgetString("d", "")
                val path = try {
                    PathParser.createPathFromPathData(d)
                } catch (_: RuntimeException) {
                    continue
                }
                path.transform(Matrix().apply { setScale(density, density) })
                val fillRaw = cmd.opt("fill")
                if (fillRaw != null && fillRaw.toString().isNotBlank() && fillRaw.toString() != "none") {
                    canvas.drawPath(path, paintFill(context, fillRaw, "#000000"))
                }
                val strokeRaw = cmd.opt("stroke")
                if (strokeRaw != null) {
                    val stroke = paintStroke(
                        context,
                        strokeRaw,
                        sx(cmd.optDouble("strokeWidth", 1.0)),
                        "#000000"
                    )
                    if (stroke != null) canvas.drawPath(path, stroke)
                }
            }
        }
    }
    return bmp
}

internal fun drawProgressBitmap(context: Context, el: JSONObject, percent: Int): Bitmap? {
    val barStyle = el.widgetString("barStyle", "linear").lowercase(Locale.US)
    val (tint, bgColor) = barColors(context, el)
    val thickness = el.optDouble("thickness", -1.0).toFloat()
    if (barStyle == "circular") {
        val sizeDp = el.optJSONObject("frame")?.optDouble("width", 40.0)?.toFloat() ?: 40f
        val s = dpToPx(context, sizeDp).coerceAtLeast(32)
        val bmp = Bitmap.createBitmap(s, s, Bitmap.Config.ARGB_8888)
        val canvas = Canvas(bmp)
        val stroke = if (thickness > 0f) dpToPx(context, thickness).toFloat().coerceAtMost(s / 2f) else (s * 0.12f).coerceAtLeast(3f)
        val pad = stroke / 2f + 1f
        val rect = RectF(pad, pad, s - pad, s - pad)
        val track = Paint(Paint.ANTI_ALIAS_FLAG).apply {
            style = Paint.Style.STROKE
            strokeWidth = stroke
            color = bgColor
            strokeCap = Paint.Cap.ROUND
        }
        val prog = Paint(Paint.ANTI_ALIAS_FLAG).apply {
            style = Paint.Style.STROKE
            strokeWidth = stroke
            color = tint
            strokeCap = Paint.Cap.ROUND
        }
        canvas.drawArc(rect, -90f, 360f, false, track)
        canvas.drawArc(rect, -90f, 360f * (percent.coerceIn(0, 100) / 100f), false, prog)
        return bmp
    }

    return drawBarBitmap(tint, bgColor, percent, dpToPx(context, 180f), dpToPx(context, if (thickness > 0f) thickness else 6f))
}

/** Linear capsule bar (track + fill) at `w`×`h` px; the caller draws it at its real width. */
internal fun drawBarBitmap(tint: Int, track: Int, percent: Int, w: Int, h: Int): Bitmap {
    val bmp = Bitmap.createBitmap(w.coerceAtLeast(1), h.coerceAtLeast(1), Bitmap.Config.ARGB_8888)
    val canvas = Canvas(bmp)
    val radius = bmp.height / 2f
    val filledW = bmp.width * (percent.coerceIn(0, 100) / 100f)
    canvas.drawRoundRect(RectF(0f, 0f, bmp.width.toFloat(), bmp.height.toFloat()), radius, radius,
        Paint(Paint.ANTI_ALIAS_FLAG).apply { style = Paint.Style.FILL; color = track })
    if (filledW > 0f) {
        // At least a dot, like Apple's `max(width * frac, 2)`.
        canvas.drawRoundRect(RectF(0f, 0f, filledW.coerceAtLeast(bmp.height.toFloat()), bmp.height.toFloat()), radius, radius,
            Paint(Paint.ANTI_ALIAS_FLAG).apply { style = Paint.Style.FILL; color = tint })
    }
    return bmp
}

/** Tint and track ARGB of a progress / gauge bar (`trackColor` or 25 % of the tint). */
internal fun barColors(context: Context, el: JSONObject): Pair<Int, Int> {
    val tint = colorProviderArgb(resolveColorProvider(context, el.opt("tint")), context, Color(0xFF4FC3F7).toArgb())
    val track = resolveColorProvider(context, el.opt("trackColor"))
        ?.let { colorProviderArgb(it, context, trackColorFromTint(tint, surfaceIsDark(context))) }
        ?: trackColorFromTint(tint, surfaceIsDark(context))
    return tint to track
}

/** Circular gauge ring (dp) when no frame width; Apple draws 56 pt. */
internal const val GAUGE_RING_DP = 56

/** Ring + label height over ring size (caption2 + a 4 pt gap, minus Android font padding ≈ 0.28 of 56 pt). */
internal const val GAUGE_LABEL_RATIO = 1.28f

internal fun drawGaugeBitmap(context: Context, el: JSONObject, percent: Int): Bitmap? {
    val tint = colorProviderArgb(resolveColorProvider(context, el.opt("tint")), context, Color(0xFF4FC3F7).toArgb())
    val dark = surfaceIsDark(context)
    val ink = colorProviderArgb(
        resolveColorProvider(context, el.opt("color")),
        context,
        if (dark) Color(0xFFF2F2F7).toArgb() else Color(0xFF111111).toArgb()
    )
    val label = el.widgetString("label", "")
    val valueLabel = when {
        el.has("currentValueLabel") -> el.widgetString("currentValueLabel", "")
        el.has("value") -> "$percent%"
        else -> ""
    }

    val minLabel = el.widgetString("minLabel", "")
    val maxLabel = el.widgetString("maxLabel", "")
    val hasEnds = minLabel.isNotBlank() || maxLabel.isNotBlank()
    val frame = el.optJSONObject("frame")
    // Apple: 56 pt ring, 5 pt stroke, caption value, caption2 label 4 pt below.
    val sizeDp = frame?.optDouble("width", GAUGE_RING_DP.toDouble())?.toFloat() ?: GAUGE_RING_DP.toFloat()
    val ring = dpToPx(context, sizeDp).coerceAtLeast(48)
    val labelH = if (label.isNotBlank()) (ring * (GAUGE_LABEL_RATIO - 1f)).toInt().coerceAtLeast(12) else 0
    val endsH = if (hasEnds) (ring * 0.18f).toInt().coerceAtLeast(10) else 0
    val s = ring
    val totalH = ring + labelH + endsH
    val bmp = Bitmap.createBitmap(s, totalH, Bitmap.Config.ARGB_8888)
    val canvas = Canvas(bmp)
    val stroke = (ring * 0.09f).coerceAtLeast(4f)
    val pad = stroke / 2f + 1f
    val rect = RectF(pad, pad, s - pad, ring - pad)

    val track = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        style = Paint.Style.STROKE
        strokeWidth = stroke
        color = trackColorFromTint(tint, dark)
        strokeCap = Paint.Cap.ROUND
    }
    val prog = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        style = Paint.Style.STROKE
        strokeWidth = stroke
        color = tint
        strokeCap = Paint.Cap.ROUND
    }
    canvas.drawArc(rect, -90f, 360f, false, track)
    canvas.drawArc(rect, -90f, 360f * (percent.coerceIn(0, 100) / 100f), false, prog)

    val valuePaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        color = ink
        textAlign = Paint.Align.CENTER
        textSize = ring * 0.22f
        typeface = Typeface.DEFAULT_BOLD
    }
    val cx = s / 2f
    val cy = ring / 2f - (valuePaint.descent() + valuePaint.ascent()) / 2f
    canvas.drawText(valueLabel, cx, cy, valuePaint)

    if (label.isNotBlank()) {
        val labelPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
            color = ink
            textAlign = Paint.Align.CENTER
            textSize = ring * 0.2f
            typeface = Typeface.DEFAULT
        }
        val ly = ring + labelH / 2f - (labelPaint.descent() + labelPaint.ascent()) / 2f
        canvas.drawText(label, cx, ly, labelPaint)
    }
    if (hasEnds) drawEndLabels(canvas, minLabel, maxLabel, 0f, s.toFloat(), totalH.toFloat(), endsH * 0.85f, ink)
    return bmp
}

/** Gauge `minLabel` / `maxLabel`: small text at the bottom-left / bottom-right. */
private fun drawEndLabels(canvas: Canvas, min: String, max: String, left: Float, right: Float, bottom: Float, size: Float, ink: Int) {
    val p = Paint(Paint.ANTI_ALIAS_FLAG).apply { color = ink; textSize = size; alpha = 0xB0 }
    val y = bottom - p.descent()
    if (min.isNotBlank()) { p.textAlign = Paint.Align.LEFT; canvas.drawText(min, left, y, p) }
    if (max.isNotBlank()) { p.textAlign = Paint.Align.RIGHT; canvas.drawText(max, right, y, p) }
}

internal fun paintFill(context: Context, color: Any?, fallbackHex: String): Paint {
    return Paint(Paint.ANTI_ALIAS_FLAG).apply {
        style = Paint.Style.FILL
        this.color = colorProviderArgb(resolveColorProvider(context, color), context, parseColor(context, fallbackHex)?.toArgb() ?: Color.Black.toArgb())
    }
}

internal fun paintStroke(context: Context, color: Any?, width: Float, fallbackHex: String?): Paint? {
    if (color == null && fallbackHex == null) return null
    return Paint(Paint.ANTI_ALIAS_FLAG).apply {
        style = Paint.Style.STROKE
        val fallback = fallbackHex?.let { parseColor(context, it)?.toArgb() } ?: Color.Black.toArgb()
        this.color = colorProviderArgb(resolveColorProvider(context, color), context, fallback)
        strokeWidth = width.coerceAtLeast(1f)
    }
}

/** [frameW]/[frameH] (dp): draw at the frame's aspect so corners and ellipses are not stretched. */
internal fun drawShapeBitmap(context: Context, el: JSONObject, sizeDp: Int, frameW: Float = -1f, frameH: Float = -1f): Bitmap? {
    val shape = el.widgetString("shapeType", "circle").lowercase(Locale.US)
    val framed = frameW > 0f && frameH > 0f
    val heightPx = dpToPx(context, if (framed) frameH else sizeDp.toFloat()).coerceAtLeast(8)
    // Capsule: width = 2*size, height = size
    val widthPx = when {
        framed -> dpToPx(context, frameW).coerceAtLeast(8)
        shape == "capsule" -> dpToPx(context, (sizeDp * 2).toFloat()).coerceAtLeast(8)
        else -> heightPx
    }
    val bmp = Bitmap.createBitmap(widthPx, heightPx, Bitmap.Config.ARGB_8888)
    val canvas = Canvas(bmp)
    val rect = RectF(0f, 0f, widthPx.toFloat(), heightPx.toFloat())
    val fillColor = colorProviderArgb(
        resolveColorProvider(context, el.opt("fill")),
        context,
        Color(0xFF888888).toArgb()
    )
    val strokeProvider = resolveColorProvider(context, el.opt("stroke"))
    val density = context.resources.displayMetrics.density
    // strokeWidth / cornerRadius are points (dp), like every other IR length.
    val strokeW = el.optDouble("strokeWidth", 0.0).toFloat().coerceAtLeast(0f) * density

    val fill = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        style = Paint.Style.FILL
        color = fillColor
    }
    val capsuleRadius = minOf(widthPx, heightPx) / 2f
    val roundedR = el.optDouble("cornerRadius", 0.0).toFloat().coerceAtLeast(0f) * density
    when (shape) {
        "capsule" -> canvas.drawRoundRect(rect, capsuleRadius, capsuleRadius, fill)
        "rectangle", "roundedrectangle" -> canvas.drawRoundRect(rect, roundedR, roundedR, fill)
        else -> canvas.drawOval(rect, fill) // circle, ellipse
    }

    if (strokeProvider != null && strokeW > 0f) {
        val stroke = Paint(Paint.ANTI_ALIAS_FLAG).apply {
            style = Paint.Style.STROKE
            color = colorProviderArgb(strokeProvider, context, Color.Black.toArgb())
            strokeWidth = strokeW
        }
        val inset = strokeW / 2f
        val sr = RectF(inset, inset, widthPx - inset, heightPx - inset)
        when (shape) {
            "capsule" -> canvas.drawRoundRect(sr, capsuleRadius, capsuleRadius, stroke)
            "rectangle", "roundedrectangle" -> canvas.drawRoundRect(sr, roundedR, roundedR, stroke)
            else -> canvas.drawOval(sr, stroke)
        }
    }
    return bmp
}
