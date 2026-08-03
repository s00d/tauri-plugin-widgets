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
    val isGradient = bg is JSONObject && bg.has("colors")
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
            val arr = bg.optJSONArray("colors") ?: return null
            val stops = mutableListOf<Int>()
            for (i in 0 until arr.length()) {
                val c = parseColor(context, arr.optString(i, ""))?.toArgb()
                if (c != null) stops += c
            }
            if (stops.isEmpty()) return null
            if (stops.size == 1) stops += stops[0]
            val gType = bg.widgetString("gradientType", "linear").lowercase(Locale.US)
            fillPaint.shader = when (gType) {
                "radial" -> {
                    val cx = content.centerX()
                    val cy = content.centerY()
                    val r = maxOf(content.width(), content.height()) / 2f
                    RadialGradient(cx, cy, r.coerceAtLeast(1f), stops.toIntArray(), null, Shader.TileMode.CLAMP)
                }
                "angular" -> SweepGradient(content.centerX(), content.centerY(), stops.toIntArray(), null)
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
                        null,
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

internal fun imageProviderFromElement(context: Context, el: JSONObject): ImageProvider? {
    val localPath = el.widgetString("localPath", "")
    if (localPath.isNotBlank()) {
        BitmapFactory.decodeFile(localPath)?.let { return ImageProvider(it) }
    }

    val data = el.widgetString("data", "")
    if (data.isNotBlank()) {
        decodeBase64Bitmap(data)?.let { return ImageProvider(it) }
    }

    val url = el.widgetString("url", "")
    if (url.startsWith("data:", true)) {
        decodeBase64Bitmap(url)?.let { return ImageProvider(it) }
    } else if (url.startsWith("file://")) {
        val filePath = Uri.parse(url).path.orEmpty()
        if (filePath.isNotBlank()) {
            BitmapFactory.decodeFile(filePath)?.let { return ImageProvider(it) }
        }
    } else if (url.startsWith("/")) {
        BitmapFactory.decodeFile(url)?.let { return ImageProvider(it) }
    }

    val systemName = el.widgetString("systemName", "")
    if (systemName.isNotBlank()) {
        resolveSystemImageProvider(context, systemName)?.let { return it }
    }

    return null
}

internal fun resolveSystemImageProvider(context: Context, systemName: String): ImageProvider? {
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
        if (appDrawable != 0) return ImageProvider(appDrawable)
        val appMipmap = context.resources.getIdentifier(name, "mipmap", context.packageName)
        if (appMipmap != 0) return ImageProvider(appMipmap)

        val androidDrawable = context.resources.getIdentifier(name, "drawable", "android")
        if (androidDrawable != 0) return ImageProvider(androidDrawable)
    }
    return null
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
    val bmp = BitmapFactory.decodeByteArray(bytes, 0, bytes.size) ?: return null
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

internal fun drawChartBitmap(context: Context, el: JSONObject): Bitmap? {
    val data = el.optJSONArray("chartData") ?: return null
    if (data.length() == 0) return null

    val points = mutableListOf<Pair<Float, Int>>()
    for (i in 0 until data.length()) {
        val p = data.optJSONObject(i) ?: continue
        val value = p.optDouble("value", 0.0).toFloat()
        val pointColorProvider = resolveColorProvider(context, p.opt("color"))
        val pointColor = pointColorProvider?.let { colorProviderArgb(it, context, Color.Transparent.toArgb()) }
        val fallback = pointColor ?: colorProviderArgb(resolveColorProvider(context, el.opt("tint")), context, Color(0xFF4FC3F7).toArgb())
        points += (value to fallback)
    }
    if (points.isEmpty()) return null

    val chartType = el.widgetString("chartType", "bar").lowercase(Locale.US)
    val frame = el.optJSONObject("frame")
    val widthDp = if (chartType == "pie") {
        frame?.optDouble("width", 80.0)?.toFloat() ?: 80f
    } else {
        frame?.optDouble("width", 180.0)?.toFloat() ?: 180f
    }
    val heightDp = if (chartType == "pie") {
        widthDp
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

    val pad = dpToPx(context, 6f).toFloat()
    val chartRect = RectF(pad, pad, w - pad, h - pad)
    val max = points.maxOf { it.first }.coerceAtLeast(0.0001f)
    val min = points.minOf { it.first }
    val span = (max - min).takeIf { it > 0.0001f } ?: 1f

    when (chartType) {
        "line", "area" -> {
            val stroke = Paint(Paint.ANTI_ALIAS_FLAG).apply {
                style = Paint.Style.STROKE
                color = points.first().second
                strokeWidth = dpToPx(context, 2.5f).toFloat()
                strokeJoin = Paint.Join.ROUND
                strokeCap = Paint.Cap.ROUND
            }
            val fill = Paint(Paint.ANTI_ALIAS_FLAG).apply {
                style = Paint.Style.FILL
                color = (points.first().second and 0x55FFFFFF)
            }
            val xs = FloatArray(points.size)
            val ys = FloatArray(points.size)
            for (i in points.indices) {
                xs[i] = chartRect.left + chartRect.width() * (i.toFloat() / (points.size - 1).coerceAtLeast(1))
                val yNorm = (points[i].first - min) / span
                ys[i] = chartRect.bottom - yNorm * chartRect.height()
            }
            val path = Path()
            val area = Path()
            path.moveTo(xs[0], ys[0])
            area.moveTo(xs[0], chartRect.bottom)
            area.lineTo(xs[0], ys[0])
            if (points.size == 2) {
                path.lineTo(xs[1], ys[1])
                area.lineTo(xs[1], ys[1])
            } else {
                for (i in 0 until points.size - 1) {
                    val x1 = xs[i]
                    val y1 = ys[i]
                    val x2 = xs[i + 1]
                    val y2 = ys[i + 1]
                    val cx1 = x1 + (x2 - x1) / 3f
                    val cx2 = x1 + 2f * (x2 - x1) / 3f
                    path.cubicTo(cx1, y1, cx2, y2, x2, y2)
                    area.cubicTo(cx1, y1, cx2, y2, x2, y2)
                }
            }
            if (chartType == "area") {
                area.lineTo(chartRect.right, chartRect.bottom)
                area.close()
                canvas.drawPath(area, fill)
            }
            canvas.drawPath(path, stroke)
            // Point markers for readability on small tiles.
            val dot = Paint(Paint.ANTI_ALIAS_FLAG).apply { style = Paint.Style.FILL }
            val r = dpToPx(context, 2.5f).toFloat()
            for (i in points.indices) {
                dot.color = points[i].second
                canvas.drawCircle(xs[i], ys[i], r, dot)
            }
        }
        "pie" -> {
            val total = points.sumOf { it.first.toDouble() }.toFloat().coerceAtLeast(0.0001f)
            var start = -90f
            for ((value, color) in points) {
                val sweep = (value / total) * 360f
                val paint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
                    style = Paint.Style.FILL
                    this.color = color
                }
                canvas.drawArc(chartRect, start, sweep, true, paint)
                start += sweep
            }
        }
        else -> {
            // Scale from 0 (or min(0, dataMin)) so the smallest positive bar isn't zero-height.
            val dataMin = points.minOf { it.first }
            val barMin = minOf(0f, dataMin)
            val barMax = points.maxOf { it.first }.coerceAtLeast(0.0001f)
            val barSpan = (barMax - barMin).takeIf { it > 0.0001f } ?: 1f
            val gap = dpToPx(context, 3f).toFloat()
            val barW = ((chartRect.width() - gap * (points.size - 1)) / points.size).coerceAtLeast(1f)
            for (i in points.indices) {
                val value = points[i].first
                val x = chartRect.left + i * (barW + gap)
                val yNorm = (value - barMin) / barSpan
                val y = chartRect.bottom - yNorm * chartRect.height()
                val paint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
                    style = Paint.Style.FILL
                    color = points[i].second
                }
                canvas.drawRoundRect(RectF(x, y, x + barW, chartRect.bottom), barW / 4f, barW / 4f, paint)
            }
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
    val tint = colorProviderArgb(resolveColorProvider(context, el.opt("tint")), context, Color(0xFF4FC3F7).toArgb())
    val dark = surfaceIsDark(context)
    val bgColor = trackColorFromTint(tint, dark)
    if (barStyle == "circular") {
        val sizeDp = el.optJSONObject("frame")?.optDouble("width", 40.0)?.toFloat() ?: 40f
        val s = dpToPx(context, sizeDp).coerceAtLeast(32)
        val bmp = Bitmap.createBitmap(s, s, Bitmap.Config.ARGB_8888)
        val canvas = Canvas(bmp)
        val stroke = (s * 0.12f).coerceAtLeast(3f)
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

    val frame = el.optJSONObject("frame")
    val widthDp = frame?.optDouble("width", 180.0)?.toFloat() ?: 180f
    val heightDp = frame?.optDouble("height", 18.0)?.toFloat() ?: 18f
    val w = dpToPx(context, widthDp).coerceAtLeast(80)
    val h = dpToPx(context, heightDp).coerceAtLeast(8)
    val bmp = Bitmap.createBitmap(w, h, Bitmap.Config.ARGB_8888)
    val canvas = Canvas(bmp)

    val radius = (h / 2f).coerceAtLeast(2f)
    val outer = RectF(0f, 0f, w.toFloat(), h.toFloat())
    val filledW = (w * (percent.coerceIn(0, 100) / 100f)).coerceAtLeast(0f)
    val inner = RectF(0f, 0f, filledW, h.toFloat())

    val bgPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply { style = Paint.Style.FILL; color = bgColor }
    val fgPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply { style = Paint.Style.FILL; color = tint }
    canvas.drawRoundRect(outer, radius, radius, bgPaint)
    if (filledW > 0f) canvas.drawRoundRect(inner, radius, radius, fgPaint)
    return bmp
}

internal fun drawGaugeBitmap(context: Context, el: JSONObject, percent: Int): Bitmap? {
    val gaugeStyle = el.widgetString("gaugeStyle", "circular").lowercase(Locale.US)
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

    if (gaugeStyle == "linear") {
        val frame = el.optJSONObject("frame")
        val widthDp = frame?.optDouble("width", 160.0)?.toFloat() ?: 160f
        val barH = dpToPx(context, 8f).coerceAtLeast(6)
        val labelH = if (label.isNotBlank() || valueLabel.isNotBlank()) dpToPx(context, 14f) else 0
        val w = dpToPx(context, widthDp).coerceAtLeast(80)
        val totalH = barH + labelH + if (labelH > 0) dpToPx(context, 4f) else 0
        val bmp = Bitmap.createBitmap(w, totalH, Bitmap.Config.ARGB_8888)
        val canvas = Canvas(bmp)
        var y = 0f
        if (labelH > 0) {
            val lp = Paint(Paint.ANTI_ALIAS_FLAG).apply {
                color = ink
                textSize = labelH * 0.7f
                typeface = Typeface.DEFAULT
            }
            if (label.isNotBlank()) {
                lp.textAlign = Paint.Align.LEFT
                canvas.drawText(label, 0f, labelH - lp.descent(), lp)
            }
            if (valueLabel.isNotBlank()) {
                lp.textAlign = Paint.Align.RIGHT
                lp.typeface = Typeface.DEFAULT_BOLD
                canvas.drawText(valueLabel, w.toFloat(), labelH - lp.descent(), lp)
            }
            y = labelH + dpToPx(context, 4f).toFloat()
        }
        val trackColor = trackColorFromTint(tint, dark)
        val radius = barH / 2f
        val outer = RectF(0f, y, w.toFloat(), y + barH)
        val filledW = (w * (percent.coerceIn(0, 100) / 100f)).coerceAtLeast(0f)
        canvas.drawRoundRect(outer, radius, radius, Paint(Paint.ANTI_ALIAS_FLAG).apply {
            style = Paint.Style.FILL; color = trackColor
        })
        if (filledW > 0f) {
            canvas.drawRoundRect(RectF(0f, y, filledW, y + barH), radius, radius, Paint(Paint.ANTI_ALIAS_FLAG).apply {
                style = Paint.Style.FILL; color = tint
            })
        }
        return bmp
    }

    val frame = el.optJSONObject("frame")
    val sizeDp = frame?.optDouble("width", 72.0)?.toFloat() ?: 72f
    val ring = dpToPx(context, sizeDp).coerceAtLeast(48)
    val labelH = if (label.isNotBlank()) (ring * 0.22f).toInt().coerceAtLeast(12) else 0
    val s = ring
    val totalH = ring + labelH
    val bmp = Bitmap.createBitmap(s, totalH, Bitmap.Config.ARGB_8888)
    val canvas = Canvas(bmp)
    val stroke = (ring * 0.12f).coerceAtLeast(4f)
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
            textSize = ring * 0.14f
            typeface = Typeface.DEFAULT
        }
        val ly = ring + labelH / 2f - (labelPaint.descent() + labelPaint.ascent()) / 2f
        canvas.drawText(label, cx, ly, labelPaint)
    }
    return bmp
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

internal fun drawShapeBitmap(context: Context, el: JSONObject, sizeDp: Int): Bitmap? {
    val shape = el.widgetString("shapeType", "circle").lowercase(Locale.US)
    val heightPx = dpToPx(context, sizeDp.toFloat()).coerceAtLeast(8)
    // Capsule: width = 2*size, height = size
    val widthPx = if (shape == "capsule") {
        dpToPx(context, (sizeDp * 2).toFloat()).coerceAtLeast(8)
    } else {
        heightPx
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
    val strokeW = el.optDouble("strokeWidth", 0.0).toFloat().coerceAtLeast(0f)

    val fill = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        style = Paint.Style.FILL
        color = fillColor
    }
    val capsuleRadius = heightPx / 2f
    when (shape) {
        "capsule" -> canvas.drawRoundRect(rect, capsuleRadius, capsuleRadius, fill)
        "rectangle" -> {
            val rr = el.optDouble("cornerRadius", 0.0).toFloat().coerceAtLeast(0f)
            canvas.drawRoundRect(rect, rr, rr, fill)
        }
        else -> canvas.drawOval(rect, fill)
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
            "rectangle" -> {
                val rr = el.optDouble("cornerRadius", 0.0).toFloat().coerceAtLeast(0f)
                canvas.drawRoundRect(sr, rr, rr, stroke)
            }
            else -> canvas.drawOval(sr, stroke)
        }
    }
    return bmp
}
