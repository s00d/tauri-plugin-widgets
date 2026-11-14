package git.s00d.widgets

import android.content.Context
import android.os.Build
import android.os.SystemClock
import android.text.format.DateUtils
import android.util.TypedValue
import android.widget.RemoteViews
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.toArgb
import androidx.glance.GlanceModifier
import androidx.glance.appwidget.AndroidRemoteViews
import androidx.glance.layout.Alignment
import androidx.glance.layout.Box
import androidx.glance.layout.Row
import androidx.glance.layout.fillMaxWidth
import androidx.glance.layout.width
import androidx.glance.LocalSize
import androidx.glance.text.FontStyle
import androidx.glance.text.FontWeight
import androidx.glance.text.Text
import androidx.glance.text.TextAlign
import androidx.glance.text.TextStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import android.graphics.Paint
import android.graphics.Typeface
import android.text.Spannable
import android.text.SpannableString
import android.text.TextPaint
import android.text.style.StrikethroughSpan
import android.text.style.StyleSpan
import android.text.style.UnderlineSpan
import android.view.Gravity

@Composable
internal fun RenderText(scope: RenderScope, el: El, modifier: GlanceModifier) {
    val context = scope.context
    val type = el.type
    when (type) {
        "label" -> {
            val txt = el.str("text", "")
            val icon = iconGlyph(el.str("systemName", "*"))
            val iconColor = resolveColorProvider(context, el.opt("iconColor"))
            if (iconColor != null) {
                Row(modifier = modifier.fillMaxWidth()) {
                    Text(icon, style = TextStyle(color = iconColor))
                    renderElementText(context, el, " $txt")
                }
            } else {
                renderElementText(context, el, "$icon $txt", modifier)
            }
        }
        else -> {
            when {
                type == "timer" -> renderLiveTimer(context, el, modifier)
                else -> {
                    val content = when (type) {
                        "date" -> formatDateValue(el.str("date", ""), el.str("dateStyle", "date"))
                        "timer" -> formatTimerValue(
                            el.str("targetDate", ""),
                            el.str("counting", "down")
                        )
                        else -> el.str("content", el.str("text", ""))
                    }
                    renderElementText(context, el, content, modifier, scope.sizeFamily)
                }
            }
        }
    }
}

@Composable
private fun renderLiveTimer(context: Context, el: El, modifier: GlanceModifier) {
    val target = parseIsoDate(el.str("targetDate", ""))
    val countingDown = !el.str("counting", "down").equals("up", ignoreCase = true)
    if (target == null || (countingDown && Build.VERSION.SDK_INT < 24)) {
        renderElementText(
            context,
            el,
            formatTimerValue(el.str("targetDate", ""), el.str("counting", "down")),
            modifier,
        )
        return
    }
    val now = System.currentTimeMillis()
    // completedText: shown when the countdown already ended at render time (a countdown
    // that ends while on screen keeps ticking below zero until the next update).
    val completed = el.str(WireField.COMPLETED_TEXT, "")
    if (countingDown && completed.isNotEmpty() && target.time <= now) {
        renderElementText(context, el, completed, modifier)
        return
    }
    // Range timer: counting up runs from startDate and stops at targetDate.
    val start = parseIsoDate(el.str(WireField.START_DATE, ""))
    if (!countingDown && start != null && target.time <= now) {
        renderElementText(context, el, formatDuration(target.time - start.time), modifier)
        return
    }
    val wallDelta = target.time - now
    val base = if (countingDown) {
        SystemClock.elapsedRealtime() + wallDelta
    } else {
        SystemClock.elapsedRealtime() - (now - (start ?: target).time)
    }
    // showHours: Chronometer adds hours only from 1:00:00; a countdown under an hour
    // never reaches it, so prefix "0:". Count-up timers grow past an hour: left as is.
    val format = if (countingDown && el.bool(WireField.SHOW_HOURS) && wallDelta < 3_600_000L) "0:%s" else null
    val argb = colorProviderArgb(
        resolveColorProvider(context, el.opt("color")),
        context,
        semanticLabelProvider(context).getColor(context).toArgb(),
    )
    val fontSize = el.num(
        "fontSize",
        semanticTextSizeSp(el.str("textStyle", "")).toDouble(),
    ).toFloat().takeIf { it > 0f } ?: 14f
    val rv = RemoteViews(context.packageName, R.layout.tauri_chronometer).apply {
        setChronometer(R.id.tauri_chrono, base, format, true)
        if (Build.VERSION.SDK_INT >= 24) {
            setChronometerCountDown(R.id.tauri_chrono, countingDown)
        }
        setTextColor(R.id.tauri_chrono, argb)
        setTextViewTextSize(R.id.tauri_chrono, TypedValue.COMPLEX_UNIT_SP, fontSize)
    }
    // Glance hosts the RemoteViews in a box that does not report the Chronometer's width to
    // a wrap-content column (the column sizes to its siblings and clips "632314:51:59" to
    // "632314:"): give it the width of the text it shows now.
    val shown = String.format(format ?: "%s", DateUtils.formatElapsedTime(kotlin.math.abs(wallDelta) / 1000))
    // ponytail: a count-up timer grows (9:59 → 10:00 → 1:00:00): two digits of slack.
    val paint = Paint().apply { textSize = TypedValue.applyDimension(TypedValue.COMPLEX_UNIT_SP, fontSize, context.resources.displayMetrics) }
    val textPx = paint.measureText(if (countingDown) shown else shown + "00")
    val widthDp = kotlin.math.ceil(textPx / context.resources.displayMetrics.density).toInt() + 2
    AndroidRemoteViews(remoteViews = rv, modifier = modifier.width(widthDp.dp))
}

@Composable
internal fun renderElementText(
    context: Context,
    el: El,
    content: String,
    modifier: GlanceModifier = GlanceModifier,
    sizeFamily: String = "",
) {
    val frame = el.obj("frame")
    val hasFixedBox =
        frame != null &&
            frame.optDouble("width", -1.0) > 0 &&
            frame.optDouble("height", -1.0) > 0
    // Badge boxes (TL/TR/BL/BR): center glyph inside frame+background.
    // Glance Text ignores vertical alignment on the modifier alone.
    val styleEl =
        if (hasFixedBox && el.str("alignment", "").isBlank()) {
            El(org.json.JSONObject(el.raw.toString()).put("alignment", "center"))
        } else {
            el
        }
    var style = textStyleFromElement(context, styleEl.raw)
    val limit = el.int("lineLimit", -1)
    var text = content
    if (el.type == "text") {
        val fit = fitText(context, el, content, style, limit, sizeFamily)
        text = fit.first
        style = fit.second
        if (el.num(WireField.LINE_HEIGHT) != null || el.num(WireField.LETTER_SPACING) != null) {
            renderSpacedText(context, el, text, style, limit, modifier)
            return
        }
    }
    renderGlanceText(text, style, limit, hasFixedBox, modifier)
}

@Composable
private fun renderGlanceText(content: String, style: TextStyle, limit: Int, hasFixedBox: Boolean, modifier: GlanceModifier) {
    if (hasFixedBox) {
        Box(modifier = modifier, contentAlignment = Alignment.Center) {
            if (limit > 0) {
                Text(content, style = style, maxLines = limit)
            } else {
                Text(content, style = style)
            }
        }
    } else if (limit > 0) {
        Text(content, modifier = modifier, style = style, maxLines = limit)
    } else {
        Text(content, modifier = modifier, style = style)
    }
}

/**
 * Width a text may use, in px: its frame width, else the widget width less typical
 * padding. Glance gives no per-node constraints, and with SizeMode.Single LocalSize is the
 * provider's minimum size, so the nominal family width (small 170, else 360 dp) is a floor.
 * Over-estimating only falls back to the TextView's tail ellipsis.
 */
@Composable
private fun availableTextWidthPx(context: Context, el: El, sizeFamily: String): Float {
    val dm = context.resources.displayMetrics
    val frameW = el.obj("frame")?.optDouble("width", -1.0) ?: -1.0
    val dp = if (frameW > 0) frameW.toFloat() else {
        val slot = runCatching { LocalSize.current.width.value }.getOrNull() ?: 0f
        val nominal = when (sizeFamily) {
            "" -> 0f
            "small" -> 170f
            else -> 360f
        }
        (maxOf(slot, nominal) - 24f).takeIf { it > 0f } ?: return Float.MAX_VALUE
    }
    return dp * dm.density
}

/** `truncationMode` head/middle and `minimumScaleFactor`, both by measuring with a TextPaint. */
@Composable
private fun fitText(context: Context, el: El, content: String, style: TextStyle, limit: Int, sizeFamily: String): Pair<String, TextStyle> {
    val mode = el.str(WireField.TRUNCATION_MODE, "tail")
    val minScale = el.num(WireField.MINIMUM_SCALE_FACTOR)?.coerceIn(0.1, 1.0)
    if ((mode == "tail" || limit != 1) && minScale == null) return content to style
    val avail = availableTextWidthPx(context, el, sizeFamily)
    if (avail == Float.MAX_VALUE) return content to style
    val sizeSp = style.fontSize?.value ?: 14f
    val dm = context.resources.displayMetrics
    val paint = TextPaint(Paint.ANTI_ALIAS_FLAG).apply {
        textSize = TypedValue.applyDimension(TypedValue.COMPLEX_UNIT_SP, sizeSp, dm)
        if (style.fontWeight == FontWeight.Bold) typeface = Typeface.DEFAULT_BOLD
    }
    var outStyle = style
    val lines = if (limit > 0) limit else 1
    if (minScale != null) {
        val need = paint.measureText(content)
        if (need > avail * lines) {
            val scale = (avail * lines / need).toDouble().coerceAtLeast(minScale).toFloat()
            outStyle = style.copy(fontSize = (sizeSp * scale).sp)
            paint.textSize *= scale
        }
    }
    if (limit == 1 && (mode == "head" || mode == "middle")) {
        return ellipsizeHeadOrMiddle(content, paint, avail, mode == "head") to outStyle
    }
    return content to outStyle
}

/**
 * `…end` (head) or `sta…end` (middle) fitting [avail] px. Hand-rolled (TextUtils.ellipsize
 * is stubbed under Robolectric). ponytail: char-based cut, may split a surrogate pair.
 */
internal fun ellipsizeHeadOrMiddle(text: String, paint: Paint, avail: Float, head: Boolean): String {
    if (paint.measureText(text) <= avail) return text
    val dots = "\u2026"
    fun build(keep: Int) = if (head) dots + text.takeLast(keep) else text.take((keep + 1) / 2) + dots + text.takeLast(keep / 2)
    var lo = 0
    var hi = text.length - 1
    while (lo < hi) {
        val mid = (lo + hi + 1) / 2
        if (paint.measureText(build(mid)) <= avail) lo = mid else hi = mid - 1
    }
    return build(lo)
}

/**
 * `lineHeight` / `letterSpacing`: Glance Text has neither, so the text goes into a plain
 * RemoteViews TextView (the plugin's chronometer layout, never started) whose
 * `setLetterSpacing` / `setLineHeight` (API 28+) are remotable. Weight, italic and
 * decorations ride along as spans.
 */
@Composable
private fun renderSpacedText(context: Context, el: El, content: String, style: TextStyle, limit: Int, modifier: GlanceModifier) {
    val dm = context.resources.displayMetrics
    val sizeSp = style.fontSize?.value ?: 14f
    val argb = colorProviderArgb(style.color, context, semanticLabelProvider(context).getColor(context).toArgb())
    val spans = SpannableString(content)
    val whole = Spannable.SPAN_INCLUSIVE_EXCLUSIVE
    val bold = style.fontWeight == FontWeight.Bold || style.fontWeight == FontWeight.Medium
    val italic = style.fontStyle == FontStyle.Italic
    val typefaceStyle = when {
        bold && italic -> Typeface.BOLD_ITALIC
        bold -> Typeface.BOLD
        italic -> Typeface.ITALIC
        else -> Typeface.NORMAL
    }
    if (typefaceStyle != Typeface.NORMAL) spans.setSpan(StyleSpan(typefaceStyle), 0, content.length, whole)
    if (el.bool(WireField.UNDERLINE)) spans.setSpan(UnderlineSpan(), 0, content.length, whole)
    if (el.bool(WireField.STRIKETHROUGH)) spans.setSpan(StrikethroughSpan(), 0, content.length, whole)
    val rv = RemoteViews(context.packageName, R.layout.tauri_chronometer).apply {
        setTextViewText(R.id.tauri_chrono, spans)
        setTextColor(R.id.tauri_chrono, argb)
        setTextViewTextSize(R.id.tauri_chrono, TypedValue.COMPLEX_UNIT_SP, sizeSp)
        if (limit > 0) setInt(R.id.tauri_chrono, "setMaxLines", limit)
        el.num(WireField.LETTER_SPACING)?.let { pt ->
            // letterSpacing is in ems on Android; the IR gives points.
            setFloat(R.id.tauri_chrono, "setLetterSpacing", (pt / sizeSp).toFloat())
        }
        el.num(WireField.LINE_HEIGHT)?.takeIf { Build.VERSION.SDK_INT >= 28 }?.let { pt ->
            setInt(R.id.tauri_chrono, "setLineHeight", TypedValue.applyDimension(TypedValue.COMPLEX_UNIT_SP, pt.toFloat(), dm).toInt())
        }
        val gravity = when (style.textAlign) {
            TextAlign.Center -> Gravity.CENTER_HORIZONTAL
            TextAlign.End -> Gravity.END
            else -> Gravity.START
        }
        setInt(R.id.tauri_chrono, "setGravity", gravity or Gravity.CENTER_VERTICAL)
    }
    AndroidRemoteViews(remoteViews = rv, modifier = modifier)
}
