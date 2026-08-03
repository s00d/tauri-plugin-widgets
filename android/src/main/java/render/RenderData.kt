package git.s00d.widgets

import android.util.Log
import androidx.compose.runtime.Composable
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.glance.GlanceModifier
import androidx.glance.Image
import androidx.glance.ImageProvider
import androidx.glance.layout.Alignment
import androidx.glance.layout.Column
import androidx.glance.layout.ContentScale
import androidx.glance.layout.Spacer
import androidx.glance.layout.fillMaxWidth
import androidx.glance.layout.height
import androidx.glance.layout.size
import androidx.glance.layout.width
import androidx.glance.text.Text
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
            renderChartElement(context, modifier, el)
        }
        "list" -> {
            // Column (not LazyColumn): home-screen lists are short, and LazyColumn
            // often yields zero children under GlanceRemoteViews / Robolectric.
            // Glance Column hard-caps ~10 children — chunk into nested Columns.
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
                            val text = item.widgetString("text", item.widgetString("content", ""))
                            val hasCheckbox = item.has("checked") || item.has("isOn")
                            val checked = item.optBoolean("checked", false) || item.optBoolean("isOn", false)
                            val action = item.widgetString("action", "")
                            val payload = item.widgetString("payload", "")
                            val rowMod = applyAction(GlanceModifier.fillMaxWidth(), action, payload, "")
                            val prefix = when {
                                hasCheckbox && checked -> "✓ "
                                hasCheckbox -> "○ "
                                anyCheckbox -> "  "
                                else -> ""
                            }
                            renderElementText(context, el, "$prefix$text", rowMod)
                        }
                    }
                }
                if (truncated > 0) {
                    if (spacing > 0) Spacer(GlanceModifier.height(spacing.dp))
                    Text(
                        "+$truncated more",
                        style = TextStyle(color = semanticSecondaryProvider(context), fontSize = 11.sp)
                    )
                }
            }
        }
        else -> {
            val value = el.num("value", 0.0)
            val minV = if (type == "gauge") el.num("min", 0.0) else 0.0
            val maxV = if (type == "gauge") {
                el.num("max", 1.0)
            } else {
                el.num("total", 1.0)
            }.coerceAtLeast(minV + 0.0001)
            val pct = (((value - minV) / (maxV - minV)) * 100).toInt().coerceIn(0, 100)
            val barStyle = el.str("barStyle", "linear").lowercase(Locale.US)
            val gaugeStyle = el.str("gaugeStyle", "circular").lowercase(Locale.US)
            val expand = when {
                type == "gauge" && gaugeStyle == "linear" -> true
                type != "gauge" && barStyle != "circular" -> true
                else -> false
            }
            val colMod = if (expand) modifier.fillMaxWidth() else modifier
            Column(modifier = colMod, horizontalAlignment = Alignment.CenterHorizontally) {
                // Gauge: labels drawn inside bitmap. Progress: optional external label (linear only).
                if (type != "gauge" && barStyle != "circular") {
                    el.str("label", "").takeIf { it.isNotBlank() }?.let { lbl ->
                        val labelEl = JSONObject(el.raw.toString()).apply {
                            if (!has("color") || isNull("color")) {
                                opt("tint")?.let { put("color", it) }
                            }
                            put("fontSize", optDouble("fontSize", 10.0))
                        }
                        Text(lbl, style = textStyleFromElement(context, labelEl))
                    }
                }
                val bmp = if (type == "gauge") drawGaugeBitmap(context, el.raw, pct) else drawProgressBitmap(context, el.raw, pct)
                if (bmp != null) {
                    when {
                        type == "gauge" && gaugeStyle == "linear" -> {
                            Image(
                                provider = ImageProvider(bmp),
                                contentDescription = type,
                                modifier = GlanceModifier.fillMaxWidth().height(28.dp),
                                contentScale = ContentScale.Fit
                            )
                        }
                        type == "gauge" -> {
                            val sizeDp = el.obj("frame")?.optDouble("width", 72.0)?.toInt()?.coerceAtLeast(1) ?: 72
                            val hasLabel = el.str("label", "").isNotBlank()
                            val hDp = if (hasLabel) (sizeDp * 1.24f).toInt() else sizeDp
                            Image(
                                provider = ImageProvider(bmp),
                                contentDescription = type,
                                modifier = GlanceModifier.width(sizeDp.dp).height(hDp.dp),
                                contentScale = ContentScale.Fit
                            )
                        }
                        barStyle == "circular" -> {
                            Image(
                                provider = ImageProvider(bmp),
                                contentDescription = type,
                                modifier = GlanceModifier.size(40.dp),
                                contentScale = ContentScale.Fit
                            )
                        }
                        else -> {
                            Image(
                                provider = ImageProvider(bmp),
                                contentDescription = type,
                                modifier = GlanceModifier.fillMaxWidth().height(12.dp),
                                contentScale = ContentScale.Fit
                            )
                        }
                    }
                } else {
                    renderElementText(context, el, progressBar(pct))
                }
            }
        }
    }
}

@Composable
private fun renderChartElement(context: android.content.Context, modifier: GlanceModifier, el: El) {
    val chartType = el.str("chartType", "bar").lowercase(Locale.US)
    val bmp = drawChartBitmap(context, el.raw)
    if (bmp != null) {
        val imageMod = if (chartType == "pie") {
            modifier.size(80.dp)
        } else {
            modifier.fillMaxWidth().height(84.dp)
        }
        Image(
            provider = ImageProvider(bmp),
            contentDescription = "chart",
            modifier = imageMod,
            contentScale = ContentScale.Fit
        )
    } else {
        Text("chart: empty", modifier = modifier, style = textStyleFromElement(context, el.raw))
    }
}
