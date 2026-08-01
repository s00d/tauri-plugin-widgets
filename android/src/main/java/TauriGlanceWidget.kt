package git.s00d.widgets

import android.appwidget.AppWidgetManager
import android.content.Context
import android.content.Intent
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.Canvas
import android.graphics.LinearGradient
import android.graphics.Paint
import android.graphics.Path
import android.graphics.RectF
import android.graphics.Shader
import android.graphics.Typeface
import android.net.Uri
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
import androidx.glance.layout.fillMaxHeight
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
import java.util.Collections
import java.util.Date
import java.util.Locale
import java.util.TimeZone

private const val GLANCE_CONTAINER_LIMIT = 10
private const val GLANCE_LIST_LIMIT = 50
private const val CONFIG_STATE_KEY_NAME = "__widget_config_state__"
private const val NONCE_STATE_KEY_NAME = "__widget_nonce_state__"
private const val TAG = "TauriGlanceWidget"
// accessOrder LRU — all get/put/evict must run under the same lock
private val BASE64_CACHE: MutableMap<Int, Bitmap> =
    Collections.synchronizedMap(LinkedHashMap(64, 0.75f, true))
private val CONFIG_STATE_KEY = stringPreferencesKey(CONFIG_STATE_KEY_NAME)
private val NONCE_STATE_KEY = stringPreferencesKey(NONCE_STATE_KEY_NAME)
@Volatile private var LAST_ROOT_HASH: String = ""
@Volatile private var LAST_ROOT_SOURCE: String = ""
/** When non-null, ink/track colors follow widget background luminance instead of system night mode. */
@Volatile private var WIDGET_SURFACE_DARK: Boolean? = null
private const val GLANCE_LIST_CHUNK = 9

/** Collect rendered / skipped element types for receipts (thread-local per compose pass). */
internal object RenderTrace {
    private val rendered = ThreadLocal.withInitial { mutableListOf<String>() }
    private val skipped = ThreadLocal.withInitial { mutableListOf<Pair<String, String>>() }

    fun begin() {
        rendered.get().clear()
        skipped.get().clear()
    }

    fun type(t: String) {
        if (t.isNotBlank()) rendered.get().add(t)
    }

    fun skip(t: String, reason: String) {
        skipped.get().add(t to reason)
    }

    fun renderedTypes(): List<String> = rendered.get().toList()

    fun skippedJson(): org.json.JSONArray {
        val arr = org.json.JSONArray()
        for ((t, reason) in skipped.get()) {
            arr.put(org.json.JSONObject().put("type", t).put("reason", reason))
        }
        return arr
    }
}

class TauriGlanceWidgetReceiver : GlanceAppWidgetReceiver() {
    override val glanceAppWidget: GlanceAppWidget = TauriGlanceWidget()
}

class TauriGlanceWidget : GlanceAppWidget() {
    override val stateDefinition = PreferencesGlanceStateDefinition

    override suspend fun provideGlance(context: Context, id: androidx.glance.GlanceId) {
        val appWidgetId = (id as? AppWidgetId)?.appWidgetId ?: -1
        val group = WidgetStoreKeys.resolveGroup(context, appWidgetId)
        val logicalWidgetId = WidgetStoreKeys.resolveWidgetId(context, appWidgetId)
        val prefs = context.getSharedPreferences(group, Context.MODE_PRIVATE)
        val configRaw = prefs.getString(WidgetStoreKeys.configKey(logicalWidgetId), null)
        val size = resolveSize(context, id)
        val nonce = prefs.getString(WidgetStoreKeys.META_NONCE, "0")?.toLongOrNull() ?: 0L
        Log.d(TAG, "provideGlance appWidgetId=$appWidgetId size=$size group=$group widgetId=$logicalWidgetId prefCfgHash=${cfgHash(configRaw)}")
        if (appWidgetId >= 0) {
            WidgetStoreKeys.bindInstance(context, appWidgetId, logicalWidgetId, group)
        }
        provideContent {
            WidgetRoot(
                context = context,
                configRaw = configRaw,
                size = size,
                appWidgetId = appWidgetId,
                group = group,
                widgetId = logicalWidgetId,
                nonce = nonce,
            )
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
        val appWidgetId = (glanceId as? AppWidgetId)?.appWidgetId ?: -1
        val group = WidgetStoreKeys.resolveGroup(context, appWidgetId)
        val action = parameters[ACTION_KEY]
        val payload = parameters[PAYLOAD_KEY]
        val url = parameters[URL_KEY]
        val normalizedUrl = url?.trim().orEmpty().let {
            if (it.equals("null", ignoreCase = true) || it.equals("undefined", ignoreCase = true)) "" else it
        }

        // Action-driven buttons are the primary flow in widgets.
        // Only attempt URL open when no action is provided.
        if (!action.isNullOrBlank()) {
            val logicalWidgetId = WidgetStoreKeys.resolveWidgetId(context, appWidgetId)
            val toggled = applyLocalListToggleIfNeeded(context, glanceId, group, logicalWidgetId, action)
            if (toggled) {
                Log.d(TAG, "action local list toggle applied action=$action")
            }
            val prefs = context.getSharedPreferences(group, Context.MODE_PRIVATE)
            val pendingRaw = prefs.getString(WidgetStoreKeys.PENDING_ACTIONS, "[]") ?: "[]"
            val arr = runCatching { JSONArray(pendingRaw) }.getOrElse { JSONArray() }
            val obj = JSONObject()
            obj.put("action", action)
            if (!payload.isNullOrBlank()) obj.put("payload", payload)
            obj.put("ts", System.currentTimeMillis())
            obj.put("widgetId", logicalWidgetId)
            obj.put("group", group)
            arr.put(obj)
            prefs.edit().putString(WidgetStoreKeys.PENDING_ACTIONS, arr.toString()).apply()

            val plugin = WidgetBridgePlugin.pluginInstance
            if (plugin != null) {
                plugin.emitWidgetAction(action, payload, logicalWidgetId, group)
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
        logicalWidgetId: String,
        action: String,
    ): Boolean {
        val prefs = context.getSharedPreferences(group, Context.MODE_PRIVATE)
        val raw = prefs.getString(WidgetStoreKeys.configKey(logicalWidgetId), null) ?: return false
        val root = runCatching { JSONObject(raw) }.getOrNull() ?: return false
        var changed = false
        listOf("small", "medium", "large").forEach { key ->
            changed = toggleListItemsInElement(root.optJSONObject(key), action) || changed
        }
        if (!changed) return false
        val updated = root.toString()
        prefs.edit().putString(WidgetStoreKeys.configKey(logicalWidgetId), updated).apply()
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
        if (el.widgetString("type", "") == "list") {
            val items = el.optJSONArray("items") ?: JSONArray()
            for (i in 0 until items.length()) {
                val item = items.optJSONObject(i) ?: continue
                if (item.widgetString("action", "") != action) continue
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
}

@Composable
internal fun WidgetRoot(
    context: Context,
    configRaw: String?,
    size: String,
    appWidgetId: Int = -1,
    group: String = "",
    widgetId: String = "default",
    nonce: Long = 0L,
) {
    val stateConfig = currentState<Preferences>()[CONFIG_STATE_KEY]
    val stateNonce = currentState<Preferences>()[NONCE_STATE_KEY]
    val effectiveConfig = stateConfig ?: configRaw
    val source = if (stateConfig != null) "state" else "prefs"
    WidgetRootBody(
        context,
        effectiveConfig,
        size,
        source,
        stateNonce,
        appWidgetId = appWidgetId,
        group = group,
        widgetId = widgetId,
        nonce = nonce,
    )
}

/** Headless / unit-test entry: no Glance Preferences state (avoids currentState). */
@Composable
internal fun WidgetRootDirect(context: Context, configRaw: String?, size: String) {
    WidgetRootBody(
        context,
        configRaw,
        size,
        source = "harness",
        stateNonce = null,
    )
}

@Composable
private fun WidgetRootBody(
    context: Context,
    effectiveConfig: String?,
    size: String,
    source: String,
    stateNonce: String?,
    appWidgetId: Int = -1,
    group: String = "",
    widgetId: String = "default",
    nonce: Long = 0L,
) {
    RenderTrace.begin()
    val combinedHash = cfgHash(effectiveConfig) + ":" + (stateNonce ?: "no_nonce")
    if (combinedHash != LAST_ROOT_HASH || source != LAST_ROOT_SOURCE) {
        LAST_ROOT_HASH = combinedHash
        LAST_ROOT_SOURCE = source
        Log.d(
            TAG,
            "WidgetRoot source=$source size=$size cfgHash=${cfgHash(effectiveConfig)} nonce=${stateNonce ?: "null"} len=${effectiveConfig?.length ?: 0}"
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
    WIDGET_SURFACE_DARK = inferSurfaceDark(context, element.opt("background"))
    // fillMaxSize on Column often collapses later siblings in Glance RemoteViews —
    // host Box fills the frame; content column only stretches width.
    Box(modifier = GlanceModifier.fillMaxSize()) {
        RenderElement(context, element, GlanceModifier.fillMaxWidth().fillMaxHeight(), size)
    }
    WIDGET_SURFACE_DARK = null

    if (appWidgetId >= 0 && group.isNotBlank()) {
        val mapNonce = nonce
        val receipt = JSONObject()
            .put("widgetId", widgetId)
            .put("group", group)
            .put("instance", appWidgetId.toString())
            .put("nonce", mapNonce)
            .put("size", size)
            .put("schema", 1)
            .put("source", source)
            .put("rendered", JSONArray(RenderTrace.renderedTypes()))
            .put("skipped", RenderTrace.skippedJson())
            .put("ts", System.currentTimeMillis())
        WidgetReceipts.write(context, receipt)
    }
}

@Composable
internal fun RenderElement(
    context: Context,
    el: JSONObject,
    modifier: GlanceModifier = GlanceModifier,
    sizeFamily: String = "small",
    inHorizontal: Boolean = false,
) {
    val type = el.widgetString("type")
    RenderTrace.type(type)
    val spacing = el.optDouble("spacing", 0.0).toInt().coerceAtLeast(0)
    val baseModifier = applyCommonStyle(context, modifier, el)
    when (type) {
        "vstack" -> {
            val hAlign = parseHorizontalStackAlignment(el.widgetString("alignment", ""))
            // Inside a Row, fillMaxWidth steals space from trailing spacer/button.
            val colMod = if (inHorizontal) baseModifier else baseModifier.fillMaxWidth()
            Column(
                modifier = colMod,
                horizontalAlignment = hAlign
            ) {
                renderChildrenVertical(
                    context,
                    el.optJSONArray("children"),
                    spacing,
                    sizeFamily = sizeFamily,
                    expandWidth = !inHorizontal,
                )
            }
        }
        "hstack" -> {
            val vAlign = parseVerticalStackAlignment(el.widgetString("alignment", ""))
            Row(
                modifier = baseModifier.fillMaxWidth(),
                verticalAlignment = vAlign
            ) {
                renderChildrenHorizontal(context, el.optJSONArray("children"), spacing, sizeFamily)
            }
        }
        "zstack" -> {
            val alignment = parseContentAlignment(el.widgetString("alignment", "center").ifBlank { "center" })
            // fillMaxSize inside a Row steals all remaining width/height and zeros siblings
            // (nested-dashboard badge crushed the title/button/body). Wrap in hstacks.
            val boxMod = if (inHorizontal) baseModifier else baseModifier.fillMaxWidth()
            Box(modifier = boxMod, contentAlignment = alignment) {
                val children = el.optJSONArray("children")
                if (children != null && children.length() > 0) {
                    for (i in 0 until children.length()) {
                        val child = children.optJSONObject(i) ?: continue
                        // Do not fillMaxSize on children — that breaks Box contentAlignment.
                        RenderElement(context, child, GlanceModifier, sizeFamily, inHorizontal)
                    }
                }
            }
        }
        "container" -> {
            val alignment = parseContentAlignment(el.widgetString("contentAlignment", "center"))
            Box(modifier = baseModifier, contentAlignment = alignment) {
                val children = el.optJSONArray("children")
                when {
                    children != null && children.length() == 1 -> {
                        // Single child: let Box contentAlignment position it (no full-width Column).
                        RenderElement(
                            context,
                            children.getJSONObject(0),
                            GlanceModifier,
                            sizeFamily,
                            inHorizontal = false,
                        )
                    }
                    children != null && children.length() > 0 -> {
                        Column(
                            modifier = GlanceModifier,
                            horizontalAlignment = Alignment.CenterHorizontally,
                        ) {
                            renderChildrenVertical(
                                context,
                                children,
                                spacing,
                                sizeFamily = sizeFamily,
                                expandWidth = false,
                            )
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
            val rowSpacing = el.optDouble("rowSpacing", spacing.toDouble()).toInt().coerceAtLeast(0)
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
                var rowIdx = 0
                while (i < children.length() && i < maxChildren) {
                    if (rowIdx > 0 && rowSpacing > 0) {
                        Spacer(GlanceModifier.height(rowSpacing.dp))
                    }
                    Row(modifier = GlanceModifier.fillMaxWidth()) {
                        for (c in 0 until visibleCols) {
                            if (c > 0 && spacing > 0) {
                                Spacer(GlanceModifier.width(spacing.dp))
                            }
                            val child = children.optJSONObject(i + c)
                            if (child != null) {
                                Box(modifier = GlanceModifier.defaultWeight()) {
                                    RenderElement(context, child, GlanceModifier.fillMaxWidth(), sizeFamily, inHorizontal = true)
                                }
                            } else {
                                Spacer(GlanceModifier.defaultWeight())
                            }
                        }
                    }
                    i += visibleCols
                    rowIdx += 1
                }
            }
        }
        "text", "date", "timer" -> {
            val content = when (type) {
                "date" -> formatDateValue(el.widgetString("date", ""), el.widgetString("dateStyle", "date"))
                "timer" -> formatTimerValue(
                    el.widgetString("targetDate", ""),
                    el.widgetString("counting", "down")
                )
                else -> el.widgetString("content", el.widgetString("text", ""))
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
                    contentDescription = el.widgetString("alt", ""),
                    modifier = imageModifier,
                    contentScale = parseContentScale(el.widgetString("contentMode", "fit"))
                )
            } else {
                val systemName = el.widgetString("systemName", "")
                val url = el.widgetString("url", "")
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
            val minV = if (type == "gauge") el.optDouble("min", 0.0) else 0.0
            val maxV = if (type == "gauge") {
                el.optDouble("max", 1.0)
            } else {
                el.optDouble("total", 1.0)
            }.coerceAtLeast(minV + 0.0001)
            val pct = (((value - minV) / (maxV - minV)) * 100).toInt().coerceIn(0, 100)
            val barStyle = el.widgetString("barStyle", "linear").lowercase(Locale.US)
            val gaugeStyle = el.widgetString("gaugeStyle", "circular").lowercase(Locale.US)
            val expand = when {
                type == "gauge" && gaugeStyle == "linear" -> true
                type != "gauge" && barStyle != "circular" -> true
                else -> false
            }
            val colMod = if (expand) baseModifier.fillMaxWidth() else baseModifier
            Column(modifier = colMod, horizontalAlignment = Alignment.CenterHorizontally) {
                // Gauge: labels drawn inside bitmap. Progress: optional external label (linear only).
                if (type != "gauge" && barStyle != "circular") {
                    el.widgetString("label", "").takeIf { it.isNotBlank() }?.let { lbl ->
                        val labelEl = JSONObject(el.toString()).apply {
                            if (!has("color") || isNull("color")) {
                                opt("tint")?.let { put("color", it) }
                            }
                            put("fontSize", optDouble("fontSize", 10.0))
                        }
                        Text(lbl, style = textStyleFromElement(context, labelEl))
                    }
                }
                val bmp = if (type == "gauge") drawGaugeBitmap(context, el, pct) else drawProgressBitmap(context, el, pct)
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
                            val sizeDp = el.optJSONObject("frame")?.optDouble("width", 72.0)?.toInt()?.coerceAtLeast(1) ?: 72
                            val hasLabel = el.widgetString("label", "").isNotBlank()
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
        "link" -> {
            val action = el.widgetString("action", "")
            val payload = el.widgetString("payload", "")
            val url = el.widgetString("url", "")
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
                renderElementText(context, el, el.widgetString("label", "link"), wrapped)
            }
        }
        "toggle" -> {
            val label = mediaGlyphFallback(
                el.widgetString("label", el.widgetString("content", "toggle"))
            )
            val isOn = el.optBoolean("isOn", false)
            val action = el.widgetString("action", "")
            val payload = el.widgetString("payload", "")
            val url = el.widgetString("url", "")
            val actionModifier = applyAction(baseModifier, action, payload, url)
            val tint = resolveColorProvider(context, el.opt("tint"))
                ?: ColorProvider(Color(0xFF4CAF50))
            val ink = resolveColorProvider(context, el.opt("color"))
                ?: ColorProvider(if (surfaceIsDark(context)) Color(0xFFE2E8F0) else Color(0xFF111111))
            val glyph = if (isOn) "\u2713" else "\u25CB"
            Row(
                modifier = actionModifier.padding(vertical = 2.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    glyph,
                    style = TextStyle(
                        color = if (isOn) tint else ColorProvider(Color(0xFF999999)),
                        fontSize = 16.sp,
                        fontWeight = FontWeight.Bold
                    )
                )
                Spacer(GlanceModifier.width(6.dp))
                Text(label, style = TextStyle(color = ink, fontSize = 14.sp))
            }
        }
        "button" -> {
            val labelRaw = el.widgetString("label", el.widgetString("content", "button"))
            val label = mediaGlyphFallback(labelRaw)
            val action = el.widgetString("action", "")
            val payload = el.widgetString("payload", "")
            val url = el.widgetString("url", "")
            val bg = resolveColorProvider(context, el.opt("backgroundColor")) ?: resolveColorProvider(context, el.opt("tint"))
            val actionModifier = applyAction(baseModifier, action, payload, url)
            val text = label
            val buttonAlignRaw = el.widgetString("textAlignment", el.widgetString("alignment", "center"))
            val textElement = JSONObject(el.toString()).apply { put("alignment", buttonAlignRaw) }
            val contentAlign = parseButtonContentAlignment(buttonAlignRaw)
            val radius = el.optDouble("cornerRadius", -1.0)
            var boxMod = actionModifier
            if (bg != null) boxMod = boxMod.background(bg)
            if (radius >= 0) boxMod = boxMod.cornerRadius(radius.toInt().coerceAtLeast(0).dp)
            boxMod = boxMod.padding(horizontal = 10.dp, vertical = 6.dp)
            if (bg != null) {
                Box(modifier = boxMod) {
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
            // RemoteViews often collapses 1.dp empty boxes — keep ≥2dp.
            val thicknessDp = el.optDouble("thickness", 1.0).toFloat().coerceAtLeast(2f)
            val thickness = thicknessDp.dp
            if (inHorizontal) {
                Box(
                    modifier = baseModifier
                        .width(thickness)
                        .height(48.dp)
                        .background(line)
                ) {}
            } else {
                // Built-in vertical breath so section separators never sit flush against
                // neighbors (parent Column spacing alone still looked tight on Glance).
                val edge = 4.dp
                Column(modifier = baseModifier.fillMaxWidth()) {
                    Spacer(GlanceModifier.height(edge))
                    Box(
                        modifier = GlanceModifier
                            .fillMaxWidth()
                            .height(thickness)
                            .background(line)
                    ) {}
                    Spacer(GlanceModifier.height(edge))
                }
            }
        }
        "spacer" -> {
            // Flex / no-minLength spacers keep Column/Row weight; fixed height would kill flex.
            if (el.has("minLength")) {
                val h = el.optDouble("minLength", 8.0).toInt().coerceAtLeast(1)
                Spacer(baseModifier.height(h.dp))
            } else {
                Spacer(baseModifier)
            }
        }
        "chart" -> {
            renderChartElement(context, baseModifier, el)
        }
        "canvas" -> {
            renderCanvasElement(context, baseModifier, el)
        }
        "shape" -> {
            val size = el.optDouble("size", 18.0).toInt().coerceAtLeast(1)
            val shapeType = el.widgetString("shapeType", "circle")
            val isCapsule = shapeType.equals("capsule", ignoreCase = true)
            // Capsule contract: width = 2*size, height = size (not square)
            val shapeModifier = if (isCapsule) {
                baseModifier.width((size * 2).dp).height(size.dp)
            } else {
                baseModifier.size(size.dp)
            }
            val shapeBmp = drawShapeBitmap(context, el, size)
            if (shapeBmp != null) {
                Image(
                    provider = ImageProvider(shapeBmp),
                    contentDescription = shapeType,
                    modifier = shapeModifier,
                    contentScale = ContentScale.FillBounds
                )
            } else {
                val symbol = when (shapeType.lowercase(Locale.US)) {
                    "capsule" -> "[====]"
                    "rectangle" -> "[##]"
                    else -> "(o)"
                }
                Text(symbol, modifier = shapeModifier, style = textStyleFromElement(context, el))
            }
        }
        "label" -> {
            val txt = el.widgetString("text", "")
            val icon = iconGlyph(el.widgetString("systemName", "*"))
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
            // Column (not LazyColumn): home-screen lists are short, and LazyColumn
            // often yields zero children under GlanceRemoteViews / Robolectric.
            // Glance Column hard-caps ~10 children — chunk into nested Columns.
            val items = el.optJSONArray("items") ?: JSONArray()
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
            Column(modifier = baseModifier.fillMaxWidth()) {
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
            if (type.isNotBlank()) RenderTrace.skip(type, "unsupported")
            renderElementText(context, el, type, baseModifier)
        }
    }
}

@Composable
private fun ColumnScope.renderChildrenVertical(
    context: Context,
    children: JSONArray?,
    spacing: Int,
    sizeFamily: String = "small",
    expandWidth: Boolean = true,
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
        if (idx > 0 && spacing > 0) {
            // Spacer between children — padding(bottom) on thin dividers often collapses in RemoteViews.
            Spacer(GlanceModifier.height(spacing.dp))
        }
        val child = visible[idx]
        val flex = child.optDouble("flex", 0.0)
        val type = child.widgetString("type", "")
        // Buttons/images/shapes hug content — fillMaxWidth makes Tap a full-bleed bar.
        val hugWidth = type in setOf("button", "toggle", "image", "shape", "label")
        val widthMod = when {
            hugWidth -> GlanceModifier
            expandWidth -> GlanceModifier.fillMaxWidth()
            else -> GlanceModifier
        }
        val base = when {
            flex > 0.0 -> widthMod.defaultWeight()
            type == "spacer" && !child.has("minLength") -> widthMod.defaultWeight()
            type == "spacer" -> {
                val h = child.optDouble("minLength", 8.0).toInt().coerceAtLeast(1)
                widthMod.height(h.dp)
            }
            else -> widthMod
        }
        RenderElement(context, child, base, sizeFamily)
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
    val allButtons = visible.isNotEmpty() && visible.all {
        it.widgetString("type", "") in setOf("button", "toggle")
    }
    for (idx in visible.indices) {
        val child = visible[idx]
        if (idx > 0 && spacing > 0) {
            Spacer(GlanceModifier.width(spacing.dp))
        }
        val t = child.widgetString("type", "")
        when (t) {
            "divider" -> {
                val th = child.optDouble("thickness", 1.0).toFloat().coerceAtLeast(2f).toInt()
                Box(modifier = GlanceModifier.width(th.dp).height(48.dp)) {
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
                Spacer(modifier = base)
            }
            else -> {
                // Only flex / expanding spacer / equal buttons take weight.
                // Title columns wrap to intrinsic width so labels aren't ellipsized
                // while the trailing spacer still pushes Sync to the end.
                val wantsWeight = when {
                    child.optDouble("flex", 0.0) > 0.0 -> true
                    hasFlex -> false
                    allButtons -> true
                    else -> false
                }
                val base = if (wantsWeight) GlanceModifier.defaultWeight() else GlanceModifier
                Box(modifier = base) {
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

    val chrome = buildStyleChromeBitmap(context, el)
    if (chrome != null) {
        m = m.background(ImageProvider(chrome))
    } else {
        val bg = resolveBackgroundProvider(context, el.opt("background"))
        if (bg != null) m = m.background(bg)
    }

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
    val clipShape = el.widgetString("clipShape", "").lowercase(Locale.US)
    val frameW = frame?.optDouble("width", -1.0)?.toFloat() ?: -1f
    val frameH = frame?.optDouble("height", -1.0)?.toFloat() ?: -1f
    val clipRadius = when {
        explicitRadius > 0f -> explicitRadius
        clipShape == "circle" -> {
            val base = listOf(frameW, frameH, el.optDouble("size", -1.0).toFloat())
                .filter { it > 0f }
                .minOrNull() ?: 24f
            (base / 2f).coerceAtLeast(1f)
        }
        clipShape == "capsule" -> {
            val base = listOf(frameW, frameH, el.optDouble("size", -1.0).toFloat())
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

/**
 * Bake gradient / border / soft shadow into a bitmap background for Glance
 * (no live gradient brush or shadow APIs).
 */
private fun buildStyleChromeBitmap(context: Context, el: JSONObject): Bitmap? {
    val bg = el.opt("background")
    val border = el.optJSONObject("border")
    val shadow = el.optJSONObject("shadow")
    val isGradient = bg is JSONObject && bg.has("colors")
    if (!isGradient && border == null && shadow == null) return null

    val frame = el.optJSONObject("frame")
    val baseW = frame?.optDouble("width", 200.0)?.toInt()?.coerceAtLeast(48) ?: 200
    val baseH = frame?.optDouble("height", 200.0)?.toInt()?.coerceAtLeast(48) ?: 200
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
            val dir = bg.widgetString("direction", "topToBottom").lowercase(Locale.US)
            val (x0, y0, x1, y1) = when (dir) {
                "bottomtotop" -> listOf(content.left, content.bottom, content.left, content.top)
                "leadingtotrailing" -> listOf(content.left, content.top, content.right, content.top)
                "trailingtoleading" -> listOf(content.right, content.top, content.left, content.top)
                "topleadingtobottomtrailing" -> listOf(content.left, content.top, content.right, content.bottom)
                "toptrailingtobottomleading" -> listOf(content.right, content.top, content.left, content.bottom)
                else -> listOf(content.left, content.top, content.left, content.bottom)
            }
            fillPaint.shader = LinearGradient(
                x0, y0, x1, y1,
                stops.toIntArray(),
                null,
                Shader.TileMode.CLAMP
            )
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

private fun resolveBackgroundProvider(context: Context, value: Any?): ColorProvider? {
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
            Log.d(TAG, "background gradient degraded to first color stop (Glance limitation)")
            return colorProviderFromString(context, arr.optString(0, ""))
        }
    }
    return null
}

private fun resolveColorProvider(context: Context, value: Any?): ColorProvider? {
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

private fun colorProviderFromString(context: Context, raw: String): ColorProvider? {
    val c = parseColor(context, raw) ?: return null
    return ColorProvider(c)
}

private fun parseColor(context: Context, raw: String): Color? {
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

/** `#abc` → `#aabbcc`, `#abcd` → `#aabbccdd` (alpha last in CSS; Android wants AARRGGBB). */
private fun expandShortHex(raw: String): String {
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
        else -> raw
    }
}

private fun semanticColor(context: Context, raw: String): Color? {
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

private fun isDarkMode(context: Context): Boolean {
    val mode = context.resources.configuration.uiMode and android.content.res.Configuration.UI_MODE_NIGHT_MASK
    return mode == android.content.res.Configuration.UI_MODE_NIGHT_YES
}

/** Prefer widget background luminance; fall back to system night mode. */
private fun surfaceIsDark(context: Context): Boolean {
    return WIDGET_SURFACE_DARK ?: isDarkMode(context)
}

private fun inferSurfaceDark(context: Context, background: Any?): Boolean? {
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

private fun luminanceOf(color: Color): Float {
    val r = color.red
    val g = color.green
    val b = color.blue
    return 0.2126f * r + 0.7152f * g + 0.0722f * b
}

/** Track color ≈ tint @ 25% alpha over transparent (matches desktop tintTrack). */
private fun trackColorFromTint(tintArgb: Int, surfaceDark: Boolean): Int {
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

private fun textStyleFromElement(context: Context, el: JSONObject): TextStyle {
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
    return ColorProvider(if (surfaceIsDark(context)) Color(0xFFF2F2F7) else Color(0xFF111111))
}

private fun semanticSecondaryProvider(context: Context): ColorProvider {
    return ColorProvider(if (surfaceIsDark(context)) Color(0xFFC7C7CC) else Color(0xFF555555))
}

private fun mediaGlyphFallback(label: String): String {
    return when (label.trim()) {
        "⏸", "❚❚", "pause" -> "||"
        "▶", "►", "play" -> ">"
        "⏮", "previous", "prev" -> "|<"
        "⏭", "next" -> ">|"
        "⏯" -> ">|"
        else -> label
    }
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

@Composable
private fun renderChartElement(context: Context, modifier: GlanceModifier, el: JSONObject) {
    val chartType = el.widgetString("chartType", "bar").lowercase(Locale.US)
    val bmp = drawChartBitmap(context, el)
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

private fun drawCanvasBitmap(context: Context, el: JSONObject): Bitmap? {
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
                val path = parseSimpleSvgPath(cmd.widgetString("d", "")) { sx(it) }
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

private fun parseSimpleSvgPath(raw: String, sx: (Double) -> Float): Path {
    val path = Path()
    val normalized = raw.replace(",", " ")
    val cmds = Regex("""([MLZmlz])([^MLZmlz]*)""").findAll(normalized)
    for (m in cmds) {
        val cmd = m.groupValues[1]
        val nums = Regex("""[-+]?(?:\d+\.?\d*|\.\d+)""")
            .findAll(m.groupValues[2])
            .map { it.value.toDouble() }
            .toList()
        when (cmd) {
            "M", "m" -> if (nums.size >= 2) {
                if (cmd == "M") path.moveTo(sx(nums[0]), sx(nums[1]))
                else path.rMoveTo(sx(nums[0]), sx(nums[1]))
                var i = 2
                while (i + 1 < nums.size) {
                    if (cmd == "M") path.lineTo(sx(nums[i]), sx(nums[i + 1]))
                    else path.rLineTo(sx(nums[i]), sx(nums[i + 1]))
                    i += 2
                }
            }
            "L", "l" -> {
                var i = 0
                while (i + 1 < nums.size) {
                    if (cmd == "L") path.lineTo(sx(nums[i]), sx(nums[i + 1]))
                    else path.rLineTo(sx(nums[i]), sx(nums[i + 1]))
                    i += 2
                }
            }
            "Z", "z" -> path.close()
        }
    }
    return path
}

private fun drawProgressBitmap(context: Context, el: JSONObject, percent: Int): Bitmap? {
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

private fun drawGaugeBitmap(context: Context, el: JSONObject, percent: Int): Bitmap? {
    val gaugeStyle = el.widgetString("gaugeStyle", "circular").lowercase(Locale.US)
    val tint = colorProviderArgb(resolveColorProvider(context, el.opt("tint")), context, Color(0xFF4FC3F7).toArgb())
    val dark = surfaceIsDark(context)
    val ink = colorProviderArgb(
        resolveColorProvider(context, el.opt("color")),
        context,
        if (dark) Color(0xFFF2F2F7).toArgb() else Color(0xFF111111).toArgb()
    )
    val label = el.widgetString("label", "")
    val valueLabel = el.widgetString("currentValueLabel", "").ifBlank {
        if (el.has("currentValueLabel") || el.has("value")) "$percent%" else ""
    }.ifBlank { "$percent%" }

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
        "time" -> SimpleDateFormat("h:mm a", Locale.US).format(date)
        "relative" -> formatRelativeSpan(date.time - now)
        "offset" -> formatDuration(date.time - now)
        "timer" -> formatDuration(date.time - now)
        else -> SimpleDateFormat("yyyy-MM-dd", Locale.getDefault()).format(date)
    }
}

private fun formatRelativeSpan(diffMs: Long): String {
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
        "person.fill" -> "\uD83D\uDC64"
        "person.2.fill" -> "\uD83D\uDC65"
        "person.badge.plus" -> "\uD83D\uDC64+"
        "heart.fill" -> "\u2764"
        "star.fill" -> "\u2B50"
        "gear", "gearshape", "gearshape.fill" -> "\u2699"
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

/**
 * org.json [JSONObject.optString] returns the literal "null" when the value is JSON null.
 * Treat JSON null / "null" / "undefined" as missing so they never render as text.
 */
private fun JSONObject.widgetString(key: String, fallback: String = ""): String {
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



