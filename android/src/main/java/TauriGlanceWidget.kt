package git.s00d.widgets

import android.appwidget.AppWidgetManager
import android.content.Context
import android.content.Intent
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.Canvas
import android.graphics.Paint
import android.graphics.Path
import android.graphics.RectF
import android.graphics.Typeface
import android.net.Uri
import android.text.format.DateUtils
import android.util.Base64
import android.util.Log
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.glance.GlanceModifier
import androidx.glance.Image
import androidx.glance.ImageProvider
import androidx.glance.currentState
import androidx.glance.action.ActionParameters
import androidx.glance.action.actionParametersOf
import androidx.glance.action.clickable
import androidx.glance.appwidget.AppWidgetId
import androidx.glance.appwidget.GlanceAppWidget
import androidx.glance.appwidget.GlanceAppWidgetReceiver
import androidx.glance.appwidget.action.actionRunCallback
import androidx.glance.appwidget.action.ActionCallback
import androidx.glance.appwidget.cornerRadius
import androidx.glance.appwidget.lazy.LazyColumn
import androidx.glance.appwidget.lazy.itemsIndexed
import androidx.glance.appwidget.provideContent
import androidx.glance.appwidget.state.updateAppWidgetState
import androidx.glance.state.PreferencesGlanceStateDefinition
import androidx.glance.background
import androidx.glance.layout.Alignment
import androidx.glance.layout.Box
import androidx.glance.layout.Column
import androidx.glance.layout.ColumnScope
import androidx.glance.layout.ContentScale
import androidx.glance.layout.Row
import androidx.glance.layout.RowScope
import androidx.glance.layout.Spacer
import androidx.glance.layout.fillMaxSize
import androidx.glance.layout.fillMaxWidth
import androidx.glance.layout.height
import androidx.glance.layout.padding
import androidx.glance.layout.size
import androidx.glance.layout.width
import androidx.glance.text.Text
import androidx.glance.text.TextAlign
import androidx.glance.text.TextStyle
import androidx.glance.text.FontWeight
import androidx.glance.unit.ColorProvider
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.core.stringPreferencesKey
import org.json.JSONArray
import org.json.JSONObject
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import java.util.TimeZone

private const val GLANCE_CONTAINER_LIMIT = 10
private const val META_PREFS = "__tauri_widget_meta__"
private const val KEY_ACTIVE_GROUP = "active_group"
private const val CONFIG_STATE_KEY_NAME = "__widget_config_state__"
private const val NONCE_STATE_KEY_NAME = "__widget_nonce_state__"
private const val TAG = "TauriGlanceWidget"
private val BASE64_CACHE = LinkedHashMap<Int, Bitmap>(64, 0.75f, true)
private val CONFIG_STATE_KEY = stringPreferencesKey(CONFIG_STATE_KEY_NAME)
private val NONCE_STATE_KEY = stringPreferencesKey(NONCE_STATE_KEY_NAME)
@Volatile private var LAST_ROOT_HASH: String = ""
@Volatile private var LAST_ROOT_SOURCE: String = ""

class TauriGlanceWidgetReceiver : GlanceAppWidgetReceiver() {
    override val glanceAppWidget: GlanceAppWidget = TauriGlanceWidget()
}

class TauriGlanceWidget : GlanceAppWidget() {
    override val stateDefinition = PreferencesGlanceStateDefinition

    override suspend fun provideGlance(context: Context, id: androidx.glance.GlanceId) {
        val group = resolveGroup(context)
        val configRaw = context.getSharedPreferences(group, Context.MODE_PRIVATE)
            .getString("__widget_config__", null)
        val size = resolveSize(context, id)
        val widgetId = (id as? AppWidgetId)?.appWidgetId ?: -1
        Log.d(TAG, "provideGlance widgetId=$widgetId size=$size group=$group prefCfgHash=${cfgHash(configRaw)}")
        provideContent {
            WidgetRoot(context, configRaw, size)
        }
    }

    private fun resolveSize(context: Context, id: androidx.glance.GlanceId): String {
        if (id !is AppWidgetId) return "small"
        val options = AppWidgetManager.getInstance(context).getAppWidgetOptions(id.appWidgetId)
        val minWidth = options.getInt(AppWidgetManager.OPTION_APPWIDGET_MIN_WIDTH, 0)
        val minHeight = options.getInt(AppWidgetManager.OPTION_APPWIDGET_MIN_HEIGHT, 0)
        return when {
            minWidth >= 320 && minHeight >= 260 -> "large"
            minWidth >= 180 && minHeight >= 110 -> "medium"
            else -> "small"
        }
    }

    private fun resolveGroup(context: Context): String {
        val active = context.getSharedPreferences(META_PREFS, Context.MODE_PRIVATE)
            .getString(KEY_ACTIVE_GROUP, null)
        if (!active.isNullOrBlank()) {
            val activePrefs = context.getSharedPreferences(active, Context.MODE_PRIVATE)
            if (activePrefs.contains("__widget_config__")) return active
        }

        val packageName = context.packageName
        val packageNameHyphen = packageName.replace('_', '-')
        val candidates = listOf(
            packageName,
            "group.$packageName",
            packageNameHyphen,
            "group.$packageNameHyphen",
        ).distinct()
        for (group in candidates) {
            val prefs = context.getSharedPreferences(group, Context.MODE_PRIVATE)
            if (prefs.contains("__widget_config__")) return group
        }
        return candidates.first()
    }
}

private val ACTION_KEY = ActionParameters.Key<String>("action")
private val PAYLOAD_KEY = ActionParameters.Key<String>("payload")
private val URL_KEY = ActionParameters.Key<String>("url")

class WidgetActionCallback : ActionCallback {
    override suspend fun onAction(
        context: Context,
        glanceId: androidx.glance.GlanceId,
        parameters: ActionParameters,
    ) {
        val group = resolveGroup(context)
        val action = parameters[ACTION_KEY]
        val payload = parameters[PAYLOAD_KEY]
        val url = parameters[URL_KEY]
        val normalizedUrl = url?.trim().orEmpty().let {
            if (it.equals("null", ignoreCase = true) || it.equals("undefined", ignoreCase = true)) "" else it
        }

        // Action-driven buttons are the primary flow in widgets.
        // Only attempt URL open when no action is provided.
        if (!action.isNullOrBlank()) {
            val toggled = applyLocalListToggleIfNeeded(context, glanceId, group, action)
            if (toggled) {
                Log.d(TAG, "action local list toggle applied action=$action")
            }
            val prefs = context.getSharedPreferences(group, Context.MODE_PRIVATE)
            val pendingRaw = prefs.getString("__widget_pending_actions__", "[]") ?: "[]"
            val arr = runCatching { JSONArray(pendingRaw) }.getOrElse { JSONArray() }
            if (!payload.isNullOrBlank()) {
                val obj = JSONObject()
                obj.put("action", action)
                obj.put("payload", payload)
                arr.put(obj)
            } else {
                arr.put(action)
            }
            prefs.edit().putString("__widget_pending_actions__", arr.toString()).apply()

            val plugin = WidgetBridgePlugin.pluginInstance
            if (plugin != null) {
                plugin.emitWidgetAction(action, payload)
            }
        } else if (normalizedUrl.isNotBlank()) {
            val uri = Uri.parse(normalizedUrl)
            val intent = Intent(Intent.ACTION_VIEW, uri).apply {
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            }
            val canHandle = intent.resolveActivity(context.packageManager) != null
            if (canHandle) {
                runCatching { context.startActivity(intent) }
                    .onFailure { e -> Log.e(TAG, "action open url failed=${e.message} url=$normalizedUrl", e) }
            } else {
                Log.w(TAG, "action open url skipped: no handler url=$normalizedUrl")
            }
        }

        runCatching { TauriGlanceWidget().update(context, glanceId) }
            .onFailure { e -> Log.e(TAG, "action-triggered update failed=${e.message}", e) }
    }

    private suspend fun applyLocalListToggleIfNeeded(
        context: Context,
        glanceId: androidx.glance.GlanceId,
        group: String,
        action: String,
    ): Boolean {
        val prefs = context.getSharedPreferences(group, Context.MODE_PRIVATE)
        val raw = prefs.getString("__widget_config__", null) ?: return false
        val root = runCatching { JSONObject(raw) }.getOrNull() ?: return false
        var changed = false
        listOf("small", "medium", "large").forEach { key ->
            changed = toggleListItemsInElement(root.optJSONObject(key), action) || changed
        }
        if (!changed) return false
        val updated = root.toString()
        prefs.edit().putString("__widget_config__", updated).apply()
        runCatching {
            updateAppWidgetState(context, glanceId) { state ->
                state[CONFIG_STATE_KEY] = updated
                state[NONCE_STATE_KEY] = System.currentTimeMillis().toString()
            }
        }.onFailure { e ->
            Log.e(TAG, "local toggle update state failed=${e.message}", e)
        }
        return true
    }

    private fun toggleListItemsInElement(el: JSONObject?, action: String): Boolean {
        if (el == null) return false
        var changed = false
        if (el.optString("type", "") == "list") {
            val items = el.optJSONArray("items") ?: JSONArray()
            for (i in 0 until items.length()) {
                val item = items.optJSONObject(i) ?: continue
                if (item.optString("action", "") != action) continue
                val hasChecked = item.has("checked")
                val hasIsOn = item.has("isOn")
                if (!hasChecked && !hasIsOn) continue
                val current = item.optBoolean("checked", item.optBoolean("isOn", false))
                val next = !current
                if (hasChecked || !hasIsOn) item.put("checked", next)
                if (hasIsOn) item.put("isOn", next)
                changed = true
            }
        }
        val children = el.optJSONArray("children")
        if (children != null) {
            for (i in 0 until children.length()) {
                changed = toggleListItemsInElement(children.optJSONObject(i), action) || changed
            }
        }
        return changed
    }

    private fun resolveGroup(context: Context): String {
        val packageName = context.packageName
        val packageNameHyphen = packageName.replace('_', '-')
        val candidates = listOf(
            packageName,
            "group.$packageName",
            packageNameHyphen,
            "group.$packageNameHyphen",
        ).distinct()
        for (group in candidates) {
            val prefs = context.getSharedPreferences(group, Context.MODE_PRIVATE)
            if (prefs.contains("__widget_config__")) return group
        }
        return candidates.first()
    }
}

@Composable
private fun WidgetRoot(context: Context, configRaw: String?, size: String) {
    val stateConfig = currentState<Preferences>()[CONFIG_STATE_KEY]
    val stateNonce = currentState<Preferences>()[NONCE_STATE_KEY]
    val effectiveConfig = stateConfig ?: configRaw
    val source = if (stateConfig != null) "state" else "prefs"
    val combinedHash = cfgHash(effectiveConfig) + ":" + (stateNonce ?: "no_nonce")
    if (combinedHash != LAST_ROOT_HASH || source != LAST_ROOT_SOURCE) {
        LAST_ROOT_HASH = combinedHash
        LAST_ROOT_SOURCE = source
        Log.d(
            TAG,
            "WidgetRoot source=$source size=$size cfgHash=${cfgHash(effectiveConfig)} nonce=${stateNonce ?: "null"} len=${effectiveConfig?.length ?: 0} stateNull=${stateConfig == null}"
        )
    }

    if (effectiveConfig.isNullOrBlank()) {
        Log.w(TAG, "WidgetRoot no configuration")
        Box(
            modifier = GlanceModifier.fillMaxSize().padding(12.dp),
            contentAlignment = Alignment.Center
        ) { Text("No configuration") }
        return
    }
    val config = runCatching { JSONObject(effectiveConfig) }.getOrNull()
    if (config == null) {
        Log.e(TAG, "WidgetRoot invalid JSON cfgHash=${cfgHash(effectiveConfig)}")
        Box(modifier = GlanceModifier.fillMaxSize().padding(12.dp), contentAlignment = Alignment.Center) {
            Text("Invalid config")
        }
        return
    }
    val element = when (size) {
        "large" -> config.optJSONObject("large") ?: config.optJSONObject("medium") ?: config.optJSONObject("small")
        "medium" -> config.optJSONObject("medium") ?: config.optJSONObject("large") ?: config.optJSONObject("small")
        else -> config.optJSONObject("small") ?: config.optJSONObject("medium") ?: config.optJSONObject("large")
    }
    if (element == null) {
        Log.e(TAG, "WidgetRoot no layout branch for size=$size cfgHash=${cfgHash(effectiveConfig)}")
        Box(modifier = GlanceModifier.fillMaxSize().padding(12.dp), contentAlignment = Alignment.Center) {
            Text("No layout")
        }
        return
    }
    RenderElement(context, element, GlanceModifier.fillMaxSize(), size)
}

@Composable
private fun RenderElement(
    context: Context,
    el: JSONObject,
    modifier: GlanceModifier = GlanceModifier,
    sizeFamily: String = "small",
    inHorizontal: Boolean = false,
) {
    val type = el.optString("type")
    val spacing = el.optDouble("spacing", 0.0).toInt().coerceAtLeast(0)
    val baseModifier = applyCommonStyle(context, modifier, el)
    when (type) {
        "vstack" -> {
            val hAlign = parseHorizontalStackAlignment(el.optString("alignment", ""))
            Column(
                modifier = baseModifier.fillMaxWidth(),
                horizontalAlignment = hAlign
            ) {
                renderChildrenVertical(context, el.optJSONArray("children"), spacing, sizeFamily = sizeFamily)
            }
        }
        "hstack" -> {
            val vAlign = parseVerticalStackAlignment(el.optString("alignment", ""))
            Row(
                modifier = baseModifier.fillMaxWidth(),
                verticalAlignment = vAlign
            ) {
                renderChildrenHorizontal(context, el.optJSONArray("children"), spacing, sizeFamily)
            }
        }
        "zstack" -> {
            val alignment = parseContentAlignment(el.optString("contentAlignment", ""))
            Box(modifier = baseModifier.fillMaxSize(), contentAlignment = alignment) {
                val children = el.optJSONArray("children")
                if (children != null && children.length() > 0) {
                    for (i in 0 until children.length()) {
                        val child = children.optJSONObject(i) ?: continue
                        RenderElement(context, child, GlanceModifier.fillMaxSize(), sizeFamily, inHorizontal)
                    }
                }
            }
        }
        "container" -> {
            val alignment = parseContentAlignment(el.optString("contentAlignment", ""))
            Box(modifier = baseModifier, contentAlignment = alignment) {
                val children = el.optJSONArray("children")
                when {
                    children != null && children.length() > 0 -> {
                        Column(modifier = GlanceModifier.fillMaxWidth()) {
                            renderChildrenVertical(context, children, spacing, sizeFamily)
                        }
                    }
                    else -> {
                        val elType = el.opt("type")?.toString().orEmpty()
                        Log.w(TAG, "container has no children; elType=$elType")
                    }
                }
            }
        }
        "grid" -> {
            val children = el.optJSONArray("children") ?: JSONArray()
            val cols = el.optInt("columns", 2).coerceAtLeast(1)
            Column(modifier = baseModifier.fillMaxWidth()) {
                val maxRows = GLANCE_CONTAINER_LIMIT
                val visibleCols = cols.coerceAtMost(GLANCE_CONTAINER_LIMIT)
                val maxChildren = maxRows * visibleCols
                if (children.length() > maxChildren) {
                    Log.w(
                        TAG,
                        "grid truncated: requested=${children.length()} rendered=$maxChildren cols=$cols visibleCols=$visibleCols"
                    )
                }
                var i = 0
                while (i < children.length() && i < maxChildren) {
                    Row(modifier = GlanceModifier.fillMaxWidth()) {
                        for (c in 0 until visibleCols) {
                            val child = children.optJSONObject(i + c)
                            if (child != null) {
                                val cellBase = GlanceModifier.defaultWeight()
                                Box(
                                    modifier = if (spacing > 0 && c < visibleCols - 1) {
                                        cellBase.padding(end = spacing.dp)
                                    } else {
                                        cellBase
                                    }
                                ) {
                                    RenderElement(context, child, GlanceModifier, sizeFamily, inHorizontal = true)
                                }
                            } else {
                                Spacer(GlanceModifier.width(0.dp))
                            }
                        }
                    }
                    i += visibleCols
                }
            }
        }
        "text", "date", "timer" -> {
            val content = when (type) {
                "date" -> formatDateValue(el.optString("date", ""), el.optString("dateStyle", "date"))
                "timer" -> formatTimerValue(
                    el.optString("targetDate", ""),
                    el.optString("counting", "down")
                )
                else -> el.optString("content", el.optString("text", ""))
            }
            renderElementText(context, el, content, baseModifier)
        }
        "image" -> {
            val size = el.optDouble("size", 24.0).toInt().coerceAtLeast(1)
            val imageModifier = baseModifier.size(size.dp)
            val provider = imageProviderFromElement(context, el)
            if (provider != null) {
                Image(
                    provider = provider,
                    contentDescription = el.optString("alt", ""),
                    modifier = imageModifier,
                    contentScale = parseContentScale(el.optString("contentMode", "fit"))
                )
            } else {
                val systemName = el.optString("systemName", "")
                val url = el.optString("url", "")
                val fallback = when {
                    systemName.isNotBlank() -> iconGlyph(systemName)
                    url.isNotBlank() -> "img:${safeHost(url)}"
                    else -> "img"
                }
                val tint = colorProviderArgb(
                    resolveColorProvider(context, el.opt("color")),
                    context,
                    semanticLabelProvider(context).getColor(context).toArgb()
                )
                val glyphBitmap = drawGlyphBitmap(context, fallback, size, tint)
                if (glyphBitmap != null) {
                    Image(
                        provider = ImageProvider(glyphBitmap),
                        contentDescription = systemName.ifBlank { "image" },
                        modifier = imageModifier,
                        contentScale = ContentScale.Fit
                    )
                } else {
                    val baseStyle = textStyleFromElement(context, el)
                    val colored = resolveColorProvider(context, el.opt("color"))
                    val styled = if (colored != null) {
                        TextStyle(color = colored, fontSize = size.sp, fontWeight = baseStyle.fontWeight, textAlign = baseStyle.textAlign)
                    } else {
                        TextStyle(fontSize = size.sp, fontWeight = baseStyle.fontWeight, textAlign = baseStyle.textAlign)
                    }
                    Text(fallback, modifier = baseModifier, style = styled)
                }
            }
        }
        "progress", "gauge" -> {
            val value = el.optDouble("value", 0.0)
            val total = el.optDouble("total", el.optDouble("max", 1.0)).coerceAtLeast(0.0001)
            val pct = ((value / total) * 100).toInt().coerceIn(0, 100)
            Column(modifier = baseModifier.fillMaxWidth()) {
                el.optString("label", "").takeIf { it.isNotBlank() }?.let { Text(it, style = textStyleFromElement(context, el)) }
                val bmp = if (type == "gauge") drawGaugeBitmap(context, el, pct) else drawProgressBitmap(context, el, pct)
                if (bmp != null) {
                    Image(
                        provider = ImageProvider(bmp),
                        contentDescription = type,
                        modifier = GlanceModifier.fillMaxWidth(),
                        contentScale = ContentScale.FillBounds
                    )
                } else {
                    renderElementText(context, el, progressBar(pct))
                }
            }
        }
        "link" -> {
            val action = el.optString("action", "")
            val payload = el.optString("payload", "")
            val url = el.optString("url", "")
            val wrapped = applyAction(baseModifier, action, payload, url)
            val children = el.optJSONArray("children")
            if (children != null && children.length() > 0) {
                Box(modifier = wrapped) {
                    for (i in 0 until children.length()) {
                        val child = children.optJSONObject(i) ?: continue
                        RenderElement(context, child, GlanceModifier, sizeFamily, inHorizontal)
                    }
                }
            } else {
                renderElementText(context, el, el.optString("label", "link"), wrapped)
            }
        }
        "button", "toggle" -> {
            val label = el.optString("label", el.optString("content", if (type == "toggle") "toggle" else "button"))
            val action = el.optString("action", "")
            val payload = el.optString("payload", "")
            val url = el.optString("url", "")
            val bg = resolveColorProvider(context, el.opt("backgroundColor")) ?: resolveColorProvider(context, el.opt("tint"))
            val actionModifier = applyAction(baseModifier, action, payload, url)
            val text = if (type == "toggle") "${if (el.optBoolean("isOn", false)) "[x]" else "[ ]"} $label" else label
            val buttonAlignRaw = el.optString("textAlignment", el.optString("alignment", "center"))
            val textElement = JSONObject(el.toString()).apply { put("alignment", buttonAlignRaw) }
            val contentAlign = parseButtonContentAlignment(buttonAlignRaw)
            if (bg != null) {
                Box(modifier = actionModifier.background(bg).padding(horizontal = 10.dp, vertical = 6.dp)) {
                    if (el.has("textAlignment")) {
                        Box(modifier = GlanceModifier.fillMaxWidth(), contentAlignment = contentAlign) {
                            renderElementText(context, textElement, text)
                        }
                    } else {
                        renderElementText(context, textElement, text)
                    }
                }
            } else {
                if (el.has("textAlignment")) {
                    Box(modifier = actionModifier.fillMaxWidth(), contentAlignment = contentAlign) {
                        renderElementText(context, textElement, text)
                    }
                } else {
                    renderElementText(context, textElement, text, actionModifier)
                }
            }
        }
        "divider" -> {
            val line = resolveColorProvider(context, el.opt("color")) ?: ColorProvider(Color.Gray)
            val thickness = el.optDouble("thickness", 1.0).toInt().coerceAtLeast(1).dp
            if (inHorizontal) {
                Box(
                    modifier = baseModifier
                        .width(thickness)
                        .height(48.dp)
                        .background(line)
                ) {}
            } else {
                Box(
                    modifier = baseModifier
                        .fillMaxWidth()
                        .height(thickness)
                        .background(line)
                ) {}
            }
        }
        "spacer" -> {
            val h = el.optDouble("minLength", 8.0).toInt().coerceAtLeast(1)
            Spacer(baseModifier.height(h.dp))
        }
        "chart" -> {
            renderChartElement(context, baseModifier, el)
        }
        "canvas" -> {
            renderCanvasElement(context, baseModifier, el)
        }
        "shape" -> {
            val size = el.optDouble("size", 18.0).toInt().coerceAtLeast(1)
            val shapeBmp = drawShapeBitmap(context, el, size)
            if (shapeBmp != null) {
                Image(
                    provider = ImageProvider(shapeBmp),
                    contentDescription = el.optString("shapeType", "shape"),
                    modifier = baseModifier.size(size.dp),
                    contentScale = ContentScale.FillBounds
                )
            } else {
                val symbol = when (el.optString("shapeType", "circle")) {
                    "capsule" -> "[====]"
                    "rectangle" -> "[##]"
                    else -> "(o)"
                }
                Text(symbol, modifier = baseModifier.size(size.dp), style = textStyleFromElement(context, el))
            }
        }
        "label" -> {
            val txt = el.optString("text", "")
            val icon = iconGlyph(el.optString("systemName", "*"))
            val iconColor = resolveColorProvider(context, el.opt("iconColor"))
            if (iconColor != null) {
                Row(modifier = baseModifier.fillMaxWidth()) {
                    Text(icon, style = TextStyle(color = iconColor))
                    renderElementText(context, el, " $txt")
                }
            } else {
                renderElementText(context, el, "$icon $txt", baseModifier)
            }
        }
        "list" -> {
            val items = el.optJSONArray("items") ?: JSONArray()
            val maxRows = when (sizeFamily) {
                "small" -> 2
                "medium" -> 3
                else -> 4
            }.coerceAtMost(GLANCE_CONTAINER_LIMIT)
            val willHaveTail = items.length() > maxRows
            val visibleItems = minOf(items.length(), if (willHaveTail) maxRows - 1 else maxRows)
            if (items.length() > visibleItems) {
                Log.w(
                    TAG,
                    "list truncated: requested=${items.length()} rendered=$visibleItems sizeFamily=$sizeFamily maxRows=$maxRows"
                )
            }
            val itemList = mutableListOf<JSONObject>()
            for (i in 0 until visibleItems) {
                items.optJSONObject(i)?.let { itemList += it }
            }
            LazyColumn(modifier = baseModifier.fillMaxWidth()) {
                itemsIndexed(itemList) { index, item ->
                    val text = item.optString("text", item.optString("content", ""))
                    val hasCheckbox = item.has("checked") || item.has("isOn")
                    val checked = item.optBoolean("checked", false) || item.optBoolean("isOn", false)
                    val action = item.optString("action", "")
                    val payload = item.optString("payload", "")
                    val rowMod = applyAction(GlanceModifier.fillMaxWidth(), action, payload, "")
                    val spaced = if (spacing > 0 && index < itemList.size - 1) rowMod.padding(bottom = spacing.dp) else rowMod
                    val prefix = if (hasCheckbox) (if (checked) "✓ " else "○ ") else ""
                    renderElementText(context, el, "$prefix$text", spaced)
                }
                val remaining = items.length() - itemList.size
                if (remaining > 0) {
                    item {
                        renderElementText(context, el, "... +$remaining")
                    }
                }
            }
        }
        else -> renderElementText(context, el, type, baseModifier)
    }
}

@Composable
private fun ColumnScope.renderChildrenVertical(
    context: Context,
    children: JSONArray?,
    spacing: Int,
    sizeFamily: String = "small",
) {
    if (children == null) return
    val visible = mutableListOf<JSONObject>()
    for (i in 0 until children.length()) {
        val child = children.optJSONObject(i) ?: continue
        visible += child
        if (visible.size >= GLANCE_CONTAINER_LIMIT) break
    }
    if (children.length() > visible.size) {
        Log.w(
            TAG,
            "column children truncated: requested=${children.length()} rendered=${visible.size} limit=$GLANCE_CONTAINER_LIMIT"
        )
    }
    if (visible.isEmpty()) return

    for (idx in visible.indices) {
        val child = visible[idx]
        val isLastVisible = idx >= visible.size - 1
        val flex = child.optDouble("flex", 0.0)
        val type = child.optString("type", "")
        val base = when {
            flex > 0.0 -> GlanceModifier.fillMaxWidth().defaultWeight()
            type == "spacer" && !child.has("minLength") -> GlanceModifier.fillMaxWidth().defaultWeight()
            type == "spacer" -> {
                val h = child.optDouble("minLength", 8.0).toInt().coerceAtLeast(1)
                GlanceModifier.fillMaxWidth().height(h.dp)
            }
            else -> GlanceModifier.fillMaxWidth()
        }
        val childModifier = if (spacing > 0 && !isLastVisible) {
            base.padding(bottom = spacing.dp)
        } else {
            base
        }
        RenderElement(context, child, childModifier, sizeFamily)
    }
}

@Composable
private fun RowScope.renderChildrenHorizontal(
    context: Context,
    children: JSONArray?,
    spacing: Int,
    sizeFamily: String,
) {
    if (children == null) return
    val visible = mutableListOf<JSONObject>()
    for (i in 0 until children.length()) {
        val child = children.optJSONObject(i) ?: continue
        visible += child
        if (visible.size >= GLANCE_CONTAINER_LIMIT) break
    }
    if (children.length() > visible.size) {
        Log.w(
            TAG,
            "row children truncated: requested=${children.length()} rendered=${visible.size} limit=$GLANCE_CONTAINER_LIMIT"
        )
    }
    val hasFlex = visible.any { it.optDouble("flex", 0.0) > 0.0 }
    for (idx in visible.indices) {
        val child = visible[idx]
        val isLast = idx >= visible.size - 1
        val t = child.optString("type", "")
        when (t) {
            "divider" -> {
                val th = child.optDouble("thickness", 1.0).toInt().coerceAtLeast(1)
                Box(
                    modifier = if (spacing > 0 && !isLast) {
                        GlanceModifier.width(th.dp).height(48.dp).padding(end = spacing.dp)
                    } else {
                        GlanceModifier.width(th.dp).height(48.dp)
                    }
                ) {
                    RenderElement(context, child, GlanceModifier.fillMaxSize(), sizeFamily, inHorizontal = true)
                }
            }
            "spacer" -> {
                val min = child.optDouble("minLength", 8.0).toInt().coerceAtLeast(1)
                val flex = child.optDouble("flex", 0.0)
                val base = if (flex > 0.0 || !child.has("minLength")) {
                    GlanceModifier.defaultWeight()
                } else {
                    GlanceModifier.width(min.dp)
                }
                Spacer(
                    modifier = if (spacing > 0 && !isLast) {
                        base.padding(end = spacing.dp)
                    } else {
                        base
                    }
                )
            }
            else -> {
                val base = if (hasFlex) {
                    if (child.optDouble("flex", 0.0) > 0.0) GlanceModifier.defaultWeight() else GlanceModifier
                } else {
                    GlanceModifier.defaultWeight()
                }
                val cellMod = if (spacing > 0 && !isLast) {
                    base.padding(end = spacing.dp)
                } else {
                    base
                }
                Box(modifier = cellMod) {
                    RenderElement(context, child, GlanceModifier, sizeFamily, inHorizontal = true)
                }
            }
        }
    }
}

private fun applyAction(modifier: GlanceModifier, action: String, payload: String, url: String): GlanceModifier {
    if (action.isBlank() && url.isBlank()) return modifier
    return modifier.clickable(
        actionRunCallback<WidgetActionCallback>(
            actionParametersOf(
                ACTION_KEY to action,
                PAYLOAD_KEY to payload,
                URL_KEY to url
            )
        )
    )
}

private fun applyCommonStyle(context: Context, modifier: GlanceModifier, el: JSONObject): GlanceModifier {
    var m = modifier

    val bg = resolveBackgroundProvider(context, el.opt("background"))
    if (bg != null) m = m.background(bg)

    val frame = el.optJSONObject("frame")
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

    val explicitRadius = el.optDouble("cornerRadius", -1.0).toFloat()
    val clipShape = el.optString("clipShape", "").lowercase(Locale.US)
    val frameW = frame?.optDouble("width", -1.0)?.toFloat() ?: -1f
    val frameH = frame?.optDouble("height", -1.0)?.toFloat() ?: -1f
    val clipRadius = when {
        explicitRadius > 0f -> explicitRadius
        clipShape == "circle" -> {
            val base = listOf(frameW, frameH).filter { it > 0f }.minOrNull() ?: 24f
            (base / 2f).coerceAtLeast(1f)
        }
        clipShape == "capsule" -> {
            val base = listOf(frameW, frameH).filter { it > 0f }.minOrNull() ?: 28f
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

private fun resolveBackgroundProvider(context: Context, value: Any?): ColorProvider? {
    if (value == null) return null
    if (value is String) return colorProviderFromString(context, value)
    if (value !is JSONObject) return null

    if (value.has("light") || value.has("dark")) {
        val isDark = isDarkMode(context)
        val pick = if (isDark) value.optString("dark", value.optString("light", "")) else value.optString("light", value.optString("dark", ""))
        return colorProviderFromString(context, pick)
    }
    if (value.has("colors")) {
        val arr = value.optJSONArray("colors")
        if (arr != null && arr.length() > 0) return colorProviderFromString(context, arr.optString(0, ""))
    }
    return null
}

private fun resolveColorProvider(context: Context, value: Any?): ColorProvider? {
    if (value == null) return null
    if (value is String) return colorProviderFromString(context, value)
    if (value is JSONObject) {
        if (value.has("light") || value.has("dark")) {
            val isDark = isDarkMode(context)
            val pick = if (isDark) value.optString("dark", value.optString("light", "")) else value.optString("light", value.optString("dark", ""))
            return colorProviderFromString(context, pick)
        }
    }
    return null
}

private fun colorProviderFromString(context: Context, raw: String): ColorProvider? {
    val c = parseColor(context, raw) ?: return null
    return ColorProvider(c)
}

private fun parseColor(context: Context, raw: String): Color? {
    val s = raw.trim()
    if (s.isEmpty()) return null
    val semantic = semanticColor(context, s)
    if (semantic != null) return semantic
    return runCatching {
        when {
            s.startsWith("#") -> {
                when (s.length) {
                    7 -> Color(android.graphics.Color.parseColor(s))
                    9 -> Color(android.graphics.Color.parseColor(s))
                    else -> Color(android.graphics.Color.parseColor(s))
                }
            }
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

private fun semanticColor(context: Context, raw: String): Color? {
    val dark = isDarkMode(context)
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

private fun isDarkMode(context: Context): Boolean {
    val mode = context.resources.configuration.uiMode and android.content.res.Configuration.UI_MODE_NIGHT_MASK
    return mode == android.content.res.Configuration.UI_MODE_NIGHT_YES
}

private fun textStyleFromElement(context: Context, el: JSONObject): TextStyle {
    val color = resolveColorProvider(context, el.opt("color")) ?: semanticLabelProvider(context)
    val semantic = semanticTextSizeSp(el.optString("textStyle", ""))
    val explicit = el.optDouble("fontSize", -1.0)
    val size = if (explicit > 0) explicit else semantic.toDouble()
    val weight = when (el.optString("fontWeight", "").lowercase(Locale.US)) {
        "bold", "heavy", "black", "semibold", "600", "700", "800", "900" -> FontWeight.Bold
        "medium", "500" -> FontWeight.Medium
        else -> FontWeight.Normal
    }
    val align = when (el.optString("alignment", "").lowercase(Locale.US)) {
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

private fun semanticTextSizeSp(style: String): Float {
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

private fun semanticLabelProvider(context: Context): ColorProvider {
    return ColorProvider(if (isDarkMode(context)) Color(0xFFF2F2F7) else Color(0xFF111111))
}

private fun parseContentAlignment(raw: String): Alignment {
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

private fun parseHorizontalStackAlignment(raw: String): Alignment.Horizontal {
    return when (raw.lowercase(Locale.US)) {
        "center", "middle" -> Alignment.CenterHorizontally
        "trailing", "end", "right" -> Alignment.End
        else -> Alignment.Start
    }
}

private fun parseVerticalStackAlignment(raw: String): Alignment.Vertical {
    return when (raw.lowercase(Locale.US)) {
        "top" -> Alignment.Top
        "bottom" -> Alignment.Bottom
        else -> Alignment.CenterVertically
    }
}

private fun parseButtonContentAlignment(raw: String): Alignment {
    return when (raw.lowercase(Locale.US)) {
        "center", "middle" -> Alignment.Center
        "trailing", "right", "end" -> Alignment.CenterEnd
        else -> Alignment.CenterStart
    }
}

private fun imageProviderFromElement(context: Context, el: JSONObject): ImageProvider? {
    val localPath = el.optString("localPath", "")
    if (localPath.isNotBlank()) {
        BitmapFactory.decodeFile(localPath)?.let { return ImageProvider(it) }
    }

    val data = el.optString("data", "")
    if (data.isNotBlank()) {
        decodeBase64Bitmap(data)?.let { return ImageProvider(it) }
    }

    val url = el.optString("url", "")
    if (url.startsWith("file://")) {
        val filePath = Uri.parse(url).path.orEmpty()
        if (filePath.isNotBlank()) {
            BitmapFactory.decodeFile(filePath)?.let { return ImageProvider(it) }
        }
    } else if (url.startsWith("/")) {
        BitmapFactory.decodeFile(url)?.let { return ImageProvider(it) }
    }

    val systemName = el.optString("systemName", "")
    if (systemName.isNotBlank()) {
        resolveSystemImageProvider(context, systemName)?.let { return it }
    }

    return null
}

private fun resolveSystemImageProvider(context: Context, systemName: String): ImageProvider? {
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

private fun decodeBase64Bitmap(raw: String): Bitmap? {
    val key = raw.hashCode()
    BASE64_CACHE[key]?.let { return it }

    val normalized = if (raw.startsWith("data:", true)) raw.substringAfter(',', "") else raw
    val bytes = runCatching { Base64.decode(normalized, Base64.DEFAULT) }.getOrNull() ?: return null
    val bmp = BitmapFactory.decodeByteArray(bytes, 0, bytes.size) ?: return null
    BASE64_CACHE[key] = bmp
    if (BASE64_CACHE.size > 80) {
        val first = BASE64_CACHE.entries.firstOrNull()?.key
        if (first != null) BASE64_CACHE.remove(first)
    }
    return bmp
}

@Composable
private fun renderChartElement(context: Context, modifier: GlanceModifier, el: JSONObject) {
    val bmp = drawChartBitmap(context, el)
    if (bmp != null) {
        Image(
            provider = ImageProvider(bmp),
            contentDescription = "chart",
            modifier = modifier.fillMaxWidth(),
            contentScale = ContentScale.FillBounds
        )
    } else {
        Text("chart: empty", modifier = modifier, style = textStyleFromElement(context, el))
    }
}

@Composable
private fun renderCanvasElement(context: Context, modifier: GlanceModifier, el: JSONObject) {
    val bmp = drawCanvasBitmap(context, el)
    if (bmp != null) {
        val widthDp = el.optDouble("width", 0.0).toInt()
        val heightDp = el.optDouble("height", 0.0).toInt()
        var imageModifier = modifier
        if (widthDp > 0) imageModifier = imageModifier.width(widthDp.dp)
        if (heightDp > 0) imageModifier = imageModifier.height(heightDp.dp)
        Image(
            provider = ImageProvider(bmp),
            contentDescription = "canvas",
            modifier = imageModifier,
            contentScale = ContentScale.FillBounds
        )
    } else {
        Text("canvas: empty", modifier = modifier, style = textStyleFromElement(context, el))
    }
}

private fun parseContentScale(mode: String): ContentScale {
    return when (mode.lowercase(Locale.US)) {
        "fill" -> ContentScale.Crop
        "fit" -> ContentScale.Fit
        "stretch" -> ContentScale.FillBounds
        else -> ContentScale.Fit
    }
}

@Composable
private fun renderElementText(
    context: Context,
    el: JSONObject,
    content: String,
    modifier: GlanceModifier = GlanceModifier,
) {
    val style = textStyleFromElement(context, el)
    val limit = el.optInt("lineLimit", -1)
    if (limit > 0) {
        Text(content, modifier = modifier, style = style, maxLines = limit)
    } else {
        Text(content, modifier = modifier, style = style)
    }
}

private fun progressBar(percent: Int): String {
    val clamped = percent.coerceIn(0, 100)
    val blocks = (clamped / 10).coerceIn(0, 10)
    return "[" + "#".repeat(blocks) + "-".repeat(10 - blocks) + "] $clamped%"
}

private fun sparkline(values: List<Double>): String {
    if (values.isEmpty()) return ""
    val ticks = charArrayOf('_', '.', '-', '~', '*', '+', 'x', '%', '#')
    val min = values.minOrNull() ?: 0.0
    val max = values.maxOrNull() ?: 1.0
    val span = (max - min).takeIf { it > 0.00001 } ?: 1.0
    val sb = StringBuilder()
    for (v in values.take(24)) {
        val idx = (((v - min) / span) * (ticks.size - 1)).toInt().coerceIn(0, ticks.size - 1)
        sb.append(ticks[idx])
    }
    return sb.toString()
}

private fun drawChartBitmap(context: Context, el: JSONObject): Bitmap? {
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

    val frame = el.optJSONObject("frame")
    val widthDp = frame?.optDouble("width", 180.0)?.toFloat() ?: 180f
    val heightDp = frame?.optDouble("height", 84.0)?.toFloat() ?: 84f
    val w = dpToPx(context, widthDp).coerceAtLeast(80)
    val h = dpToPx(context, heightDp).coerceAtLeast(44)
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
    val chartType = el.optString("chartType", "bar").lowercase(Locale.US)

    when (chartType) {
        "line", "area" -> {
            val stroke = Paint(Paint.ANTI_ALIAS_FLAG).apply {
                style = Paint.Style.STROKE
                color = points.first().second
                strokeWidth = dpToPx(context, 2f).toFloat()
            }
            val fill = Paint(Paint.ANTI_ALIAS_FLAG).apply {
                style = Paint.Style.FILL
                color = (points.first().second and 0x55FFFFFF)
            }
            val path = Path()
            val area = Path()
            for (i in points.indices) {
                val x = chartRect.left + chartRect.width() * (i.toFloat() / (points.size - 1).coerceAtLeast(1))
                val yNorm = (points[i].first - min) / span
                val y = chartRect.bottom - yNorm * chartRect.height()
                if (i == 0) {
                    path.moveTo(x, y)
                    area.moveTo(x, chartRect.bottom)
                    area.lineTo(x, y)
                } else {
                    path.lineTo(x, y)
                    area.lineTo(x, y)
                }
            }
            if (chartType == "area") {
                area.lineTo(chartRect.right, chartRect.bottom)
                area.close()
                canvas.drawPath(area, fill)
            }
            canvas.drawPath(path, stroke)
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
            val gap = dpToPx(context, 3f).toFloat()
            val barW = ((chartRect.width() - gap * (points.size - 1)) / points.size).coerceAtLeast(1f)
            for (i in points.indices) {
                val value = points[i].first
                val x = chartRect.left + i * (barW + gap)
                val yNorm = (value - min) / span
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

private fun drawCanvasBitmap(context: Context, el: JSONObject): Bitmap? {
    val commands = el.optJSONArray("elements") ?: return null
    if (commands.length() == 0) return null
    val width = dpToPx(context, el.optDouble("width", 180.0).toFloat()).coerceAtLeast(40)
    val height = dpToPx(context, el.optDouble("height", 100.0).toFloat()).coerceAtLeast(30)
    val bmp = Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888)
    val canvas = Canvas(bmp)

    val bg = resolveBackgroundProvider(context, el.opt("background"))?.getColor(context)
    if (bg != null) canvas.drawColor(bg.toArgb())

    for (i in 0 until commands.length()) {
        val cmd = commands.optJSONObject(i) ?: continue
        val draw = cmd.optString("draw", "")
        when (draw) {
            "circle" -> {
                val cx = cmd.optDouble("cx", 0.0).toFloat()
                val cy = cmd.optDouble("cy", 0.0).toFloat()
                val r = cmd.optDouble("r", 0.0).toFloat()
                val fill = paintFill(context, cmd.opt("fill"), "#000000")
                canvas.drawCircle(cx, cy, r, fill)
                val strokeRaw = cmd.opt("stroke")
                if (strokeRaw != null) {
                    val stroke = paintStroke(context, strokeRaw, cmd.optDouble("strokeWidth", 1.0).toFloat(), "#000000")
                    if (stroke != null) canvas.drawCircle(cx, cy, r, stroke)
                }
            }
            "line" -> {
                val p = paintStroke(context, cmd.opt("stroke"), cmd.optDouble("strokeWidth", 1.0).toFloat(), "#000000")
                if (p == null) continue
                canvas.drawLine(
                    cmd.optDouble("x1", 0.0).toFloat(),
                    cmd.optDouble("y1", 0.0).toFloat(),
                    cmd.optDouble("x2", 0.0).toFloat(),
                    cmd.optDouble("y2", 0.0).toFloat(),
                    p
                )
            }
            "rect" -> {
                val x = cmd.optDouble("x", 0.0).toFloat()
                val y = cmd.optDouble("y", 0.0).toFloat()
                val w = cmd.optDouble("width", 0.0).toFloat()
                val h = cmd.optDouble("height", 0.0).toFloat()
                val rr = cmd.optDouble("cornerRadius", 0.0).toFloat()
                val rect = RectF(x, y, x + w, y + h)
                val fill = paintFill(context, cmd.opt("fill"), "#000000")
                canvas.drawRoundRect(rect, rr, rr, fill)
                val strokeRaw = cmd.opt("stroke")
                if (strokeRaw != null) {
                    val stroke = paintStroke(context, strokeRaw, cmd.optDouble("strokeWidth", 1.0).toFloat(), "#000000")
                    if (stroke != null) canvas.drawRoundRect(rect, rr, rr, stroke)
                }
            }
            "arc" -> {
                val cx = cmd.optDouble("cx", 0.0).toFloat()
                val cy = cmd.optDouble("cy", 0.0).toFloat()
                val r = cmd.optDouble("r", 0.0).toFloat()
                val start = cmd.optDouble("startAngle", 0.0).toFloat()
                val end = cmd.optDouble("endAngle", 0.0).toFloat()
                val rect = RectF(cx - r, cy - r, cx + r, cy + r)
                val stroke = paintStroke(context, cmd.opt("stroke"), cmd.optDouble("strokeWidth", 1.0).toFloat(), null)
                val fallbackStroke = if (stroke == null) {
                    paintStroke(context, cmd.opt("fill"), cmd.optDouble("strokeWidth", 1.0).toFloat(), "#000000")
                } else stroke
                if (fallbackStroke == null) continue
                canvas.drawArc(rect, start, end - start, false, fallbackStroke)
            }
            "text" -> {
                val paint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
                    style = Paint.Style.FILL
                    color = colorProviderArgb(resolveColorProvider(context, cmd.opt("color")), context, Color.Black.toArgb())
                    textSize = cmd.optDouble("fontSize", 12.0).toFloat()
                }
                canvas.drawText(
                    cmd.optString("content", ""),
                    cmd.optDouble("x", 0.0).toFloat(),
                    cmd.optDouble("y", 0.0).toFloat(),
                    paint
                )
            }
        }
    }
    return bmp
}

private fun drawProgressBitmap(context: Context, el: JSONObject, percent: Int): Bitmap? {
    val frame = el.optJSONObject("frame")
    val widthDp = frame?.optDouble("width", 180.0)?.toFloat() ?: 180f
    val heightDp = frame?.optDouble("height", 18.0)?.toFloat() ?: 18f
    val w = dpToPx(context, widthDp).coerceAtLeast(80)
    val h = dpToPx(context, heightDp).coerceAtLeast(8)
    val bmp = Bitmap.createBitmap(w, h, Bitmap.Config.ARGB_8888)
    val canvas = Canvas(bmp)

    val bgColor = if (isDarkMode(context)) Color(0xFF2E2E35).toArgb() else Color(0xFFE5E7EB).toArgb()
    val tint = colorProviderArgb(resolveColorProvider(context, el.opt("tint")), context, Color(0xFF4FC3F7).toArgb())
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

private fun drawGaugeBitmap(context: Context, el: JSONObject, percent: Int): Bitmap? {
    val frame = el.optJSONObject("frame")
    val sizeDp = frame?.optDouble("width", 72.0)?.toFloat() ?: 72f
    val s = dpToPx(context, sizeDp).coerceAtLeast(48)
    val bmp = Bitmap.createBitmap(s, s, Bitmap.Config.ARGB_8888)
    val canvas = Canvas(bmp)
    val stroke = (s * 0.12f).coerceAtLeast(4f)
    val pad = stroke / 2f + 1f
    val rect = RectF(pad, pad, s - pad, s - pad)

    val track = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        style = Paint.Style.STROKE
        strokeWidth = stroke
        color = if (isDarkMode(context)) Color(0xFF2E2E35).toArgb() else Color(0xFFE5E7EB).toArgb()
        strokeCap = Paint.Cap.ROUND
    }
    val prog = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        style = Paint.Style.STROKE
        strokeWidth = stroke
        color = colorProviderArgb(resolveColorProvider(context, el.opt("tint")), context, Color(0xFF4FC3F7).toArgb())
        strokeCap = Paint.Cap.ROUND
    }
    canvas.drawArc(rect, -90f, 360f, false, track)
    canvas.drawArc(rect, -90f, 360f * (percent.coerceIn(0, 100) / 100f), false, prog)
    return bmp
}

private fun paintFill(context: Context, color: Any?, fallbackHex: String): Paint {
    return Paint(Paint.ANTI_ALIAS_FLAG).apply {
        style = Paint.Style.FILL
        this.color = colorProviderArgb(resolveColorProvider(context, color), context, parseColor(context, fallbackHex)?.toArgb() ?: Color.Black.toArgb())
    }
}

private fun paintStroke(context: Context, color: Any?, width: Float, fallbackHex: String?): Paint? {
    if (color == null && fallbackHex == null) return null
    return Paint(Paint.ANTI_ALIAS_FLAG).apply {
        style = Paint.Style.STROKE
        val fallback = fallbackHex?.let { parseColor(context, it)?.toArgb() } ?: Color.Black.toArgb()
        this.color = colorProviderArgb(resolveColorProvider(context, color), context, fallback)
        strokeWidth = width.coerceAtLeast(1f)
    }
}

private fun dpToPx(context: Context, dp: Float): Int {
    return (dp * context.resources.displayMetrics.density).toInt().coerceAtLeast(1)
}

private fun drawGlyphBitmap(context: Context, glyph: String, sizeDp: Int, argb: Int): Bitmap? {
    if (glyph.isBlank()) return null
    val px = dpToPx(context, sizeDp.toFloat()).coerceAtLeast(16)
    val bmp = Bitmap.createBitmap(px, px, Bitmap.Config.ARGB_8888)
    val canvas = Canvas(bmp)
    val paint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        color = argb
        textAlign = Paint.Align.CENTER
        textSize = px * 0.78f
        typeface = Typeface.DEFAULT_BOLD
    }
    val x = px / 2f
    val y = px / 2f - (paint.descent() + paint.ascent()) / 2f
    canvas.drawText(glyph, x, y, paint)
    return bmp
}

private fun drawShapeBitmap(context: Context, el: JSONObject, sizeDp: Int): Bitmap? {
    val px = dpToPx(context, sizeDp.toFloat()).coerceAtLeast(8)
    val bmp = Bitmap.createBitmap(px, px, Bitmap.Config.ARGB_8888)
    val canvas = Canvas(bmp)
    val rect = RectF(0f, 0f, px.toFloat(), px.toFloat())
    val shape = el.optString("shapeType", "circle").lowercase(Locale.US)
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
    when (shape) {
        "capsule" -> canvas.drawRoundRect(rect, px / 2f, px / 2f, fill)
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
        val sr = RectF(inset, inset, px - inset, px - inset)
        when (shape) {
            "capsule" -> canvas.drawRoundRect(sr, px / 2f, px / 2f, stroke)
            "rectangle" -> {
                val rr = el.optDouble("cornerRadius", 0.0).toFloat().coerceAtLeast(0f)
                canvas.drawRoundRect(sr, rr, rr, stroke)
            }
            else -> canvas.drawOval(sr, stroke)
        }
    }
    return bmp
}

private fun colorProviderArgb(provider: ColorProvider?, context: Context, fallback: Int): Int {
    val value: Any = provider?.getColor(context) ?: return fallback
    return when (value) {
        is Color -> value.toArgb()
        is Long -> Color(value).toArgb()
        is Int -> value
        else -> fallback
    }
}

private fun formatDateValue(raw: String, style: String): String {
    val date = parseIsoDate(raw) ?: return raw
    val now = System.currentTimeMillis()
    return when (style.lowercase(Locale.US)) {
        "time" -> SimpleDateFormat("HH:mm", Locale.getDefault()).format(date)
        "relative" -> DateUtils.getRelativeTimeSpanString(
            date.time,
            now,
            DateUtils.MINUTE_IN_MILLIS,
            DateUtils.FORMAT_ABBREV_RELATIVE
        ).toString()
        "offset" -> formatDuration(date.time - now)
        "timer" -> formatDuration(date.time - now)
        else -> SimpleDateFormat("yyyy-MM-dd", Locale.getDefault()).format(date)
    }
}

private fun formatTimerValue(raw: String, counting: String): String {
    val date = parseIsoDate(raw) ?: return raw
    val now = System.currentTimeMillis()
    val diff = if (counting.equals("up", true)) now - date.time else date.time - now
    return formatDuration(diff)
}

private fun formatDuration(diffMs: Long): String {
    val sign = if (diffMs < 0) "-" else ""
    val sec = kotlin.math.abs(diffMs) / 1000
    val h = sec / 3600
    val m = (sec % 3600) / 60
    val s = sec % 60
    return "%s%02d:%02d:%02d".format(Locale.US, sign, h, m, s)
}

private fun parseIsoDate(raw: String): Date? {
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

private fun safeHost(rawUrl: String): String {
    return runCatching { Uri.parse(rawUrl).host }.getOrNull().orEmpty().ifBlank { "remote" }
}

private fun iconGlyph(systemName: String): String {
    return when (systemName.lowercase(Locale.US)) {
        "cloud.sun.fill" -> "\u26C5"
        "sun.max.fill" -> "\u2600"
        "cloud.rain.fill" -> "\u2614"
        "cloud.fill" -> "\u2601"
        "cloud.bolt.fill", "bolt.fill" -> "\u26A1"
        "moon.stars.fill" -> "\u263E"
        "paintpalette.fill" -> "\u2698"
        "location.fill" -> "\u2302"
        "person.fill" -> "\u25C9"
        "person.2.fill" -> "\u25C9\u25C9"
        "person.badge.plus" -> "\u25C9+"
        "heart.fill" -> "\u2764"
        "star.fill" -> "\u2B50"
        "drop.fill" -> "\u25CF"
        "leaf.fill" -> "\u273F"
        "flame.fill" -> "\uD83D\uDD25"
        "hourglass", "hourglass.bottomhalf.filled" -> "\u23F3"
        "music.note", "music.note.list" -> "\u266A"
        "bitcoinsign.circle.fill" -> "\u20BF"
        "quote.opening" -> "\u201C"
        "figure.run" -> "\u27A4"
        "globe" -> "\u25CE"
        "clock" -> "\u23F0"
        "desktopcomputer" -> "\u25A3"
        "network" -> "\u2261"
        "internaldrive" -> "\u25A0"
        "battery.100" -> "\u25AE"
        "iphone" -> "\u25AF"
        "applewatch" -> "\u231A"
        "airpodspro" -> "\u25CB"
        "ipad" -> "\u25AD"
        "eye.fill" -> "\u25C9"
        "bubble.left.fill" -> "\u25ED"
        else -> if (systemName.isNotBlank()) systemName.take(1) else "*"
    }
}

private fun cfgHash(text: String?): String {
    if (text.isNullOrEmpty()) return "null"
    return text.hashCode().toUInt().toString(16)
}

private fun countRenderableChildren(children: JSONArray): Int {
    var count = 0
    for (i in 0 until children.length()) {
        if (children.optJSONObject(i) != null) count += 1
    }
    return count.coerceAtLeast(1)
}

private fun maxRenderableItems(@Suppress("UNUSED_PARAMETER") spacing: Int): Int {
    return GLANCE_CONTAINER_LIMIT
}



