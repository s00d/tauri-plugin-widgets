package git.s00d.widgets

import android.util.Log
import android.content.Context
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.glance.GlanceModifier
import androidx.glance.Image
import androidx.glance.ImageProvider
import androidx.glance.LocalSize
import androidx.glance.appwidget.lazy.LazyColumn
import androidx.glance.appwidget.lazy.itemsIndexed
import androidx.glance.layout.padding
import androidx.glance.layout.Alignment
import androidx.glance.layout.Column
import androidx.glance.layout.Row
import androidx.glance.layout.ContentScale
import androidx.glance.layout.Spacer
import androidx.glance.layout.fillMaxWidth
import androidx.glance.layout.height
import androidx.glance.layout.size
import androidx.glance.layout.width
import androidx.glance.text.FontWeight
import androidx.glance.text.Text
import androidx.glance.unit.ColorProvider
import androidx.glance.text.TextStyle
import org.json.JSONArray
import org.json.JSONObject
import java.util.Locale

@Composable
internal fun RenderData(scope: RenderScope, el: El, modifier: GlanceModifier) {
    val context = scope.context
    val type = el.type
    val spacing = el.num("spacing", 0.0).toInt().coerceAtLeast(0)
    when (type) {
        "chart" -> {
            renderChartElement(scope, modifier, el)
        }
        "list" -> {
            // Column while the rows fit (LazyColumn often yields zero children under
            // GlanceRemoteViews / Robolectric); LazyColumn once they overflow (items only).
            // Glance Column hard-caps ~10 children — chunk into nested Columns.
            val customChildren = el.arr("children")
            if (customChildren != null && customChildren.length() > 0) {
                // Rich rows (stress-nest): arbitrary widget children, not ListItem[].
                val childScope = scope.vertical()
                val limit = customChildren.length().coerceAtMost(GLANCE_LIST_LIMIT)
                Column(modifier = modifier.fillMaxWidth()) {
                    for (i in 0 until limit) {
                        if (i > 0 && spacing > 0) {
                            Spacer(GlanceModifier.height(spacing.dp))
                        }
                        val child = customChildren.optJSONObject(i) ?: continue
                        RenderElement(childScope, El(child), GlanceModifier.fillMaxWidth())
                    }
                }
            } else {
                val items = el.arr("items") ?: JSONArray()
                val itemList = mutableListOf<JSONObject>()
                for (i in 0 until items.length()) {
                    items.optJSONObject(i)?.let { itemList += it }
                    if (itemList.size >= GLANCE_LIST_LIMIT) break
                }
                val truncated = items.length() - itemList.size
                if (truncated > 0) {
                    Log.w(
                        TAG,
                        "list items truncated: requested=${items.length()} rendered=${itemList.size} limit=$GLANCE_LIST_LIMIT"
                    )
                }
                val anyCheckbox = itemList.any { it.has("checked") || it.has("isOn") }
                @Composable
                fun row(item: JSONObject, mod: GlanceModifier) {
                    val text = item.widgetString("text", item.widgetString("content", ""))
                    val hasCheckbox = item.has("checked") || item.has("isOn")
                    val checked = item.optBoolean("checked", false) || item.optBoolean("isOn", false)
                    val action = item.widgetString("action", "")
                    val payload = item.widgetString("payload", "")
                    val rowMod = applyAction(mod, action, payload, "")
                    val prefix = when {
                        hasCheckbox && checked -> "✓ "
                        hasCheckbox -> "○ "
                        anyCheckbox -> "  "
                        else -> ""
                    }
                    renderElementText(context, el, "$prefix$text", rowMod)
                }
                val more = "+$truncated more"
                val moreStyle = TextStyle(color = semanticSecondaryProvider(context), fontSize = 11.sp)
                val rows = itemList.size + if (truncated > 0) 1 else 0
                if (listScrolls(rows, el.num("fontSize", 14.0), spacing, verticalPaddingDp(el.opt("padding")), LocalSize.current.height.value)) {
                    // More rows than the widget is tall: a scrolling ListView (RemoteViews
                    // collection; Glance forbids nesting, rows here are plain text).
                    LazyColumn(modifier = modifier.fillMaxWidth()) {
                        itemsIndexed(itemList) { index, item ->
                            val top = if (index > 0) spacing else 0
                            row(item, GlanceModifier.fillMaxWidth().padding(top = top.dp))
                        }
                        if (truncated > 0) item {
                            Text(more, style = moreStyle, modifier = GlanceModifier.padding(top = spacing.dp))
                        }
                    }
                    return
                }
                val chunks = itemList.chunked(GLANCE_LIST_CHUNK)
                Column(modifier = modifier.fillMaxWidth()) {
                    chunks.forEachIndexed { chunkIdx, chunk ->
                        Column(modifier = GlanceModifier.fillMaxWidth()) {
                            chunk.forEachIndexed { index, item ->
                                if (index > 0 && spacing > 0) {
                                    Spacer(GlanceModifier.height(spacing.dp))
                                } else if (index == 0 && chunkIdx > 0 && spacing > 0) {
                                    Spacer(GlanceModifier.height(spacing.dp))
                                }
                                row(item, GlanceModifier.fillMaxWidth())
                            }
                        }
                    }
                    if (truncated > 0) {
                        if (spacing > 0) Spacer(GlanceModifier.height(spacing.dp))
                        Text(more, style = moreStyle)
                    }
                }
            }
        }
        else -> {
            val minV = if (type == "gauge") el.num("min", 0.0) else 0.0
            val maxV = if (type == "gauge") {
                el.num("max", 1.0)
            } else {
                el.num("total", 1.0)
            }.coerceAtLeast(minV + 0.0001)
            // startAt/endAt (Unix ms): the elapsed fraction at render time. Glance cannot
            // animate it; it moves with each widget update.
            val value = timeDrivenFraction(el)?.let { minV + it * (maxV - minV) } ?: el.num("value", 0.0)
            val pct = (((value - minV) / (maxV - minV)) * 100).toInt().coerceIn(0, 100)
            val barStyle = el.str("barStyle", "linear").lowercase(Locale.US)
            val gaugeStyle = el.str("gaugeStyle", "circular").lowercase(Locale.US)
            val desc = valueDescription(el, type, value, pct)
            if (if (type == "gauge") gaugeStyle == "linear" else barStyle != "circular") {
                LinearBar(context, el, type == "gauge", pct, desc, modifier)
                return
            }
            Column(modifier = modifier, horizontalAlignment = Alignment.CenterHorizontally) {
                val bmp = if (type == "gauge") drawGaugeBitmap(context, el.raw, pct) else drawProgressBitmap(context, el.raw, pct)
                if (bmp == null) {
                    renderElementText(context, el, progressBar(pct))
                } else if (type == "gauge") {
                    val sizeDp = el.obj("frame")?.optDouble("width", GAUGE_RING_DP.toDouble())?.toInt()?.coerceAtLeast(1) ?: GAUGE_RING_DP
                    val hasLabel = el.str("label", "").isNotBlank()
                    val hasEnds = el.str(WireField.MIN_LABEL, "").isNotBlank() || el.str(WireField.MAX_LABEL, "").isNotBlank()
                    val hDp = (sizeDp * ((if (hasLabel) GAUGE_LABEL_RATIO else 1f) + (if (hasEnds) 0.18f else 0f))).toInt()
                    Image(
                        provider = ImageProvider(bmp),
                        contentDescription = desc,
                        modifier = GlanceModifier.width(sizeDp.dp).height(hDp.dp),
                        contentScale = ContentScale.Fit
                    )
                } else {
                    Image(
                        provider = ImageProvider(bmp),
                        contentDescription = desc,
                        modifier = GlanceModifier.size(40.dp),
                        contentScale = ContentScale.Fit
                    )
                }
            }
        }
    }
}

/**
 * Linear progress / gauge as the Apple renderers lay it out: a leading label row (gauge: label
 * leading, `currentValueLabel` trailing), a capsule bar filling the width, gauge end labels.
 * Text stays Glance text (never scaled inside a bitmap); the bar bitmap is drawn at the widget
 * width and stretched to the slot.
 */
@Composable
private fun LinearBar(context: Context, el: El, gauge: Boolean, pct: Int, desc: String, modifier: GlanceModifier) {
    val (tint, track) = barColors(context, el.raw)
    val ink = colorProviderArgb(
        resolveColorProvider(context, el.opt("color")), context,
        if (surfaceIsDark(context)) 0xFFF2F2F7.toInt() else 0xFF111111.toInt(),
    )
    fun inkAlpha(a: Float) = ColorProvider(Color(ink).copy(alpha = a))
    val label = el.str("label", "")
    val valueLabel = if (gauge) el.str("currentValueLabel", "") else ""
    val barH = (if (gauge) 6.0 else el.num(WireField.THICKNESS) ?: 6.0).toInt().coerceIn(1, 48)
    // ponytail: bitmap width = whole widget, FillBounds squeezes it to the slot; a narrow slot
    // flattens the fill's round end slightly. Measure the slot if that ever shows.
    val widthDp = LocalSize.current.width.value.takeIf { it > 0f } ?: 180f
    val bmp = drawBarBitmap(tint, track, pct, dpToPx(context, widthDp), dpToPx(context, barH.toFloat()))
    Column(modifier = modifier.fillMaxWidth()) {
        if (label.isNotBlank() || valueLabel.isNotBlank()) {
            // No 2 pt gap as on Apple: the TextView font padding already leaves it.
            Row(modifier = GlanceModifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                val labelColor = when {
                    gauge -> inkAlpha(0.7f)
                    else -> resolveColorProvider(context, el.opt("color"))
                        ?: resolveColorProvider(context, el.opt("tint"))
                        ?: semanticSecondaryProvider(context)
                }
                Text(label, maxLines = 1, modifier = GlanceModifier.defaultWeight(),
                    style = TextStyle(color = labelColor, fontSize = el.num("fontSize", 11.0).toFloat().sp))
                if (valueLabel.isNotBlank()) {
                    Text(valueLabel, maxLines = 1,
                        style = TextStyle(color = inkAlpha(1f), fontSize = 12.sp, fontWeight = FontWeight.Medium))
                }
            }
        }
        Image(
            provider = ImageProvider(bmp),
            contentDescription = desc,
            modifier = GlanceModifier.fillMaxWidth().height(barH.dp),
            contentScale = ContentScale.FillBounds
        )
        val minLabel = el.str(WireField.MIN_LABEL, "")
        val maxLabel = el.str(WireField.MAX_LABEL, "")
        if (gauge && (minLabel.isNotBlank() || maxLabel.isNotBlank())) {
            val endStyle = TextStyle(color = inkAlpha(0.6f), fontSize = 9.sp)
            Row(modifier = GlanceModifier.fillMaxWidth().padding(top = 2.dp)) {
                Text(minLabel, maxLines = 1, style = endStyle, modifier = GlanceModifier.defaultWeight())
                Text(maxLabel, maxLines = 1, style = endStyle)
            }
        }
    }
}

/** Text-row height per sp of font size (Android TextView line ≈ 1.17–1.36 × size). */
// ponytail: estimate, not a measure; the list scrolls once its rows overflow the whole widget
// height (its own padding counted, sibling elements not). Measure in RemoteViews if it misjudges.
internal const val LIST_ROW_PER_SP = 1.3

/** True when `rows` text rows (plus spacing and padding) are taller than `heightDp`. */
internal fun listScrolls(rows: Int, fontSp: Double, spacingDp: Int, paddingDp: Double, heightDp: Float): Boolean =
    heightDp > 0 && rows > 0 &&
        rows * fontSp * LIST_ROW_PER_SP + (rows - 1) * spacingDp + paddingDp > heightDp

/** Top + bottom of an IR `padding` (number or `{top, bottom, …}`, as [paddingModifier]). */
internal fun verticalPaddingDp(padding: Any?): Double = when (padding) {
    is Number -> padding.toDouble() * 2
    is JSONObject -> padding.optDouble("top", 0.0) + padding.optDouble("bottom", 0.0)
    else -> 0.0
}

/** Spoken text of a progress / gauge bitmap: the value, not just "progress". */
internal fun valueDescription(el: El, type: String, value: Double, pct: Int): String {
    el.str(WireField.ACCESSIBILITY_LABEL, "").takeIf { it.isNotBlank() }?.let { return it }
    val label = el.str("label", "").trim()
    val amount = if (type == "gauge") {
        el.str("currentValueLabel", "").ifBlank { git.s00d.widgets.Resolve.text(value) }
    } else "$pct%"
    return listOf(label, amount).filter { it.isNotEmpty() }.joinToString(" ")
}

/** Spoken text of a chart bitmap: type and the first points (`Mon 3, Tue 5, …`). */
internal fun chartDescription(el: El, maxPoints: Int = 10): String {
    el.str(WireField.ACCESSIBILITY_LABEL, "").takeIf { it.isNotBlank() }?.let { return it }
    val pts = el.raw.optJSONArray("chartData")
    val parts = (0 until minOf(pts?.length() ?: 0, maxPoints)).mapNotNull { i ->
        val p = pts!!.optJSONObject(i) ?: return@mapNotNull null
        listOf(p.optString("label", ""), git.s00d.widgets.Resolve.text(p.opt("value")))
            .filter { it.isNotEmpty() }.joinToString(" ")
    }
    val more = if ((pts?.length() ?: 0) > maxPoints) ", …" else ""
    val kind = "${el.str("chartType", "bar").lowercase(Locale.US)} chart"
    return if (parts.isEmpty()) kind else "$kind: ${parts.joinToString(", ")}$more"
}

/** Elapsed fraction 0…1 between `startAt` and `endAt` (Unix ms), or null when unset. */
internal fun timeDrivenFraction(el: El, now: Long = System.currentTimeMillis()): Double? {
    val start = el.num(WireField.START_AT) ?: return null
    val end = el.num(WireField.END_AT) ?: return null
    if (end <= start) return if (now >= end) 1.0 else 0.0
    return ((now - start) / (end - start)).coerceIn(0.0, 1.0)
}

@Composable
private fun renderChartElement(scope: RenderScope, modifier: GlanceModifier, el: El) {
    val context = scope.context
    val chartType = el.str("chartType", "bar").lowercase(Locale.US)
    // A v2 chart (Swift Charts on Apple) in a row takes the row's height and its width
    // share; the bitmap is drawn at that estimated slot so Fit fills it.
    val fill = chartType != "pie" && scope.inHorizontal && isV2Chart(el.raw) &&
        el.obj("frame") == null && scope.slotW > 0f && scope.slotH > 0f
    val bmp = if (fill) drawChartBitmap(context, el.raw, scope.slotW, scope.slotH) else drawChartBitmap(context, el.raw)
    if (bmp != null) {
        val imageMod = if (chartType == "pie") {
            modifier.size(80.dp)
        } else if (fill) {
            modifier.fillMaxWidth().height(scope.slotH.dp)
        } else {
            modifier.fillMaxWidth().height(84.dp)
        }
        Image(
            provider = ImageProvider(bmp),
            contentDescription = chartDescription(el),
            modifier = imageMod,
            contentScale = ContentScale.Fit
        )
    } else {
        Text("chart: empty", modifier = modifier, style = textStyleFromElement(context, el.raw))
    }
}
