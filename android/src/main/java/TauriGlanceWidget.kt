package git.s00d.widgets

import android.appwidget.AppWidgetManager
import android.content.Context
import android.util.Log
import androidx.compose.runtime.Composable
import androidx.compose.ui.unit.dp
import androidx.datastore.preferences.core.Preferences
import androidx.glance.GlanceModifier
import androidx.glance.currentState
import androidx.glance.appwidget.AppWidgetId
import androidx.glance.appwidget.GlanceAppWidget
import androidx.glance.appwidget.GlanceAppWidgetReceiver
import androidx.glance.appwidget.provideContent
import androidx.glance.layout.Alignment
import androidx.glance.layout.Box
import androidx.glance.layout.fillMaxHeight
import androidx.glance.layout.fillMaxSize
import androidx.glance.layout.fillMaxWidth
import androidx.glance.layout.padding
import androidx.glance.state.PreferencesGlanceStateDefinition
import androidx.glance.text.Text
import org.json.JSONArray
import org.json.JSONObject

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

    override fun onDeleted(context: Context, appWidgetIds: IntArray) {
        for (id in appWidgetIds) {
            WidgetStoreKeys.clearInstance(context, id)
        }
        super.onDeleted(context, appWidgetIds)
    }
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
        // Only bind when this instance already has a mapping or a real config —
        // otherwise a fresh tile would permanently latch onto fallback "default".
        val alreadyMapped = WidgetStoreKeys.mappedWidgetId(context, appWidgetId) != null
        if (appWidgetId >= 0 && (alreadyMapped || configRaw != null)) {
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
    Log.d(
        TAG,
        "WidgetRoot source=$source size=$size cfgHash=${cfgHash(effectiveConfig)} nonce=${stateNonce ?: "null"} len=${effectiveConfig?.length ?: 0}"
    )

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
    WIDGET_SURFACE_DARK.set(inferSurfaceDark(context, element.opt("background")))
    // Adaptive {light,dark} backgrounds must keep null override so isDarkMode wins.
    if (element.opt("background") is JSONObject) {
        val bg = element.optJSONObject("background")
        if (bg != null && bg.has("light") && bg.has("dark")) {
            WIDGET_SURFACE_DARK.set(null)
        }
    }
    // fillMaxSize on Column often collapses later siblings in Glance RemoteViews —
    // host Box fills the frame; content column only stretches width.
    Box(modifier = GlanceModifier.fillMaxSize()) {
        RenderElement(RenderScope(context, size), El(element), GlanceModifier.fillMaxWidth().fillMaxHeight())
    }
    WIDGET_SURFACE_DARK.remove()

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
            .put("trigger", "timeline")
            .put("rendered", JSONArray(RenderTrace.renderedTypes()))
            .put("skipped", RenderTrace.skippedJson())
            .put("ts", System.currentTimeMillis())
        WidgetReceipts.write(context, receipt)
    }
}
