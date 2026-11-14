package git.s00d.widgets

import android.app.PendingIntent
import android.appwidget.AppWidgetManager
import android.appwidget.AppWidgetProviderInfo
import android.content.Context
import android.util.Log
import android.widget.RemoteViews
import androidx.compose.runtime.Composable
import androidx.compose.ui.unit.DpSize
import androidx.compose.ui.unit.dp
import androidx.datastore.preferences.core.Preferences
import androidx.glance.LocalSize
import androidx.glance.appwidget.cornerRadius
import androidx.glance.appwidget.appWidgetBackground
import androidx.glance.appwidget.SizeMode
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

/**
 * Preview shown in the launcher's "Add widget?" dialog for `requestWidget`: the stored config,
 * with none of `provideGlance`'s side effects (no binding, source fetch or alarms).
 */
internal class PinPreviewWidget : GlanceAppWidget() {
    override suspend fun provideGlance(context: Context, id: androidx.glance.GlanceId) {
        provideContent {
            val s = LocalSize.current
            WidgetPreview(context, TauriGlanceWidget.familyFor(s.width.value, s.height.value))
        }
    }
}

class TauriGlanceWidget : GlanceAppWidget() {
    override val stateDefinition = PreferencesGlanceStateDefinition

    // Exact: one composition per size the launcher reports (portrait and landscape), so the
    // family and the chrome bitmaps follow resizes instead of the size at first bind.
    override val sizeMode: SizeMode = SizeMode.Exact

    override suspend fun provideGlance(context: Context, id: androidx.glance.GlanceId) {
        val appWidgetId = (id as? AppWidgetId)?.appWidgetId ?: -1
        val p = Placement.read(context, appWidgetId)
        val fetched = WidgetSource.enqueueIfDue(context, p.configRaw, p.group, p.widgetId, p.instance)
        Log.d(TAG, "provideGlance appWidgetId=$appWidgetId group=${p.group} widgetId=${p.widgetId} own=${p.ownConfig} prefCfgHash=${cfgHash(p.configRaw)}")
        // Only bind when this instance already has a mapping or a real config —
        // otherwise a fresh tile would permanently latch onto fallback "default".
        val alreadyMapped = WidgetStoreKeys.mappedWidgetId(context, appWidgetId) != null
        if (appWidgetId >= 0 && (alreadyMapped || p.configRaw != null)) {
            WidgetStoreKeys.bindInstance(context, appWidgetId, p.widgetId, p.group)
        }
        WidgetScheduler.scheduleFor(context, p.configRaw, fetched.second, p.instance)
        val hostCategory = hostCategory(context, appWidgetId)
        provideContent {
            val s = LocalSize.current
            WidgetRoot(context = context, size = familyFor(s.width.value, s.height.value, hostCategory), appWidgetId = appWidgetId)
        }
    }

    /**
     * A composition that throws (bad IR, Glance limits) shows the styled error layout instead
     * of the launcher's generic "Can't load widget", and leaves a receipt saying why.
     */
    override fun onCompositionError(
        context: Context,
        glanceId: androidx.glance.GlanceId,
        appWidgetId: Int,
        throwable: Throwable,
    ) {
        Log.e(TAG, "composition failed appWidgetId=$appWidgetId", throwable)
        runCatching { writeErrorReceipt(context, appWidgetId, throwable) }
        AppWidgetManager.getInstance(context).updateAppWidget(appWidgetId, errorViews(context))
    }

    /** Picker previews (Android 15+, `setWidgetPreviews`): the stored config, per size. */
    override val previewSizeMode = SizeMode.Responsive(
        setOf(DpSize(110.dp, 110.dp), DpSize(250.dp, 110.dp), DpSize(250.dp, 250.dp)),
    )

    override suspend fun providePreview(context: Context, widgetCategory: Int) {
        provideContent {
            val s = LocalSize.current
            WidgetPreview(context, familyFor(s.width.value, s.height.value))
        }
    }

    companion object {
        internal fun errorViews(context: Context): RemoteViews {
            val rv = RemoteViews(context.packageName, R.layout.tauri_widget_error)
            context.packageManager.getLaunchIntentForPackage(context.packageName)?.let { launch ->
                rv.setOnClickPendingIntent(
                    R.id.tauri_widget_error_root,
                    PendingIntent.getActivity(context, 0, launch, PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE),
                )
            }
            return rv
        }

        internal fun writeErrorReceipt(context: Context, appWidgetId: Int, throwable: Throwable) {
            val p = Placement.read(context, appWidgetId)
            WidgetReceipts.write(
                context,
                JSONObject()
                    .put("widgetId", p.widgetId)
                    .put("group", p.group)
                    .put("instance", p.instance ?: appWidgetId.toString())
                    .put("appWidgetId", appWidgetId)
                    .put("nonce", p.nonce)
                    .put("schema", 1)
                    .put("source", "prefs")
                    .put("trigger", "error")
                    .put("rendered", JSONArray())
                    .put("skipped", JSONArray().put(JSONObject().put("type", "widget").put("reason", "composition error: ${throwable.javaClass.simpleName}: ${throwable.message}")))
                    .put("error", "${throwable.javaClass.simpleName}: ${throwable.message}"),
            )
        }

        /** `OPTION_APPWIDGET_HOST_CATEGORY` of a placement (home screen when unknown). */
        internal fun hostCategory(context: Context, appWidgetId: Int): Int =
            if (appWidgetId < 0) AppWidgetProviderInfo.WIDGET_CATEGORY_HOME_SCREEN
            else runCatching {
                AppWidgetManager.getInstance(context).getAppWidgetOptions(appWidgetId)
                    .getInt(AppWidgetManager.OPTION_APPWIDGET_HOST_CATEGORY, AppWidgetProviderInfo.WIDGET_CATEGORY_HOME_SCREEN)
            }.getOrDefault(AppWidgetProviderInfo.WIDGET_CATEGORY_HOME_SCREEN)

        /**
         * Size family from the dp size being composed. A lock-screen host (keyguard: Galaxy
         * Flip cover screen, Android 16 lock-screen hub) gets `small`: it is seen at a glance,
         * often on a small display, however large the cell it reports.
         */
        fun familyFor(widthDp: Float, heightDp: Float, hostCategory: Int = AppWidgetProviderInfo.WIDGET_CATEGORY_HOME_SCREEN): String = when {
            hostCategory == AppWidgetProviderInfo.WIDGET_CATEGORY_KEYGUARD -> "small"
            widthDp >= 320 && heightDp >= 260 -> "large"
            widthDp >= 180 && heightDp >= 110 -> "medium"
            else -> "small"
        }
    }
}

/** Prefs-backed inputs of one placement; cheap SharedPreferences reads. */
internal class Placement(
    val group: String,
    val widgetId: String,
    val instance: String?,
    val ownConfig: Boolean,
    val configRaw: String?,
    val hostData: Any?,
    val nonce: Long,
) {
    val scopedId: String get() = WidgetStoreKeys.scopedId(widgetId, instance)

    companion object {
        fun read(context: Context, appWidgetId: Int): Placement {
            val group = WidgetStoreKeys.resolveGroup(context, appWidgetId)
            val widgetId = WidgetStoreKeys.resolveWidgetId(context, appWidgetId)
            val prefs = context.getSharedPreferences(group, Context.MODE_PRIVATE)
            // Placement id = the instance picked in WidgetConfigActivity, else the appWidgetId
            // (`setWidgetConfig(…, instance = "<appWidgetId>")`).
            val instance = WidgetStoreKeys.placementInstance(context, appWidgetId)
            return Placement(
                group = group,
                widgetId = widgetId,
                instance = instance,
                ownConfig = instance != null &&
                    prefs.getString(WidgetStoreKeys.configKey(WidgetStoreKeys.scopedId(widgetId, instance)), null) != null,
                configRaw = WidgetStoreKeys.lookupKeys(WidgetStoreKeys.CONFIG_PREFIX, widgetId, instance)
                    .firstNotNullOfOrNull { prefs.getString(it, null) },
                hostData = WidgetStoreKeys.lookupKeys(WidgetStoreKeys.DATA_PREFIX, widgetId, instance)
                    .firstNotNullOfOrNull { Resolve.json(prefs.getString(it, null)) },
                nonce = prefs.getString(WidgetStoreKeys.META_NONCE, "0")?.toLongOrNull() ?: 0L,
            )
        }
    }
}

@Composable
internal fun WidgetRoot(context: Context, size: String, appWidgetId: Int = -1) {
    val state = currentState<Preferences>()
    val stateNonce = state[NONCE_STATE_KEY]
    // update()/updateAll() on a running Glance session only recomposes (provideGlance is not
    // re-run), so prefs are read here: every state nonce bump re-reads config and data.
    val p = Placement.read(context, appWidgetId)
    // A placement with its own config ignores the widget-wide config pushed into state.
    val stateConfig = if (p.ownConfig) null else state[CONFIG_STATE_KEY]
    val effectiveConfig = stateConfig ?: p.configRaw
    val source = if (stateConfig != null) "state" else "prefs"
    val cfgObj = Resolve.json(effectiveConfig) as? JSONObject
    val cfgData = cfgObj?.opt("data")
    val widgetState = WidgetStoreKeys.readState(
        context.getSharedPreferences(p.group, Context.MODE_PRIVATE), p.widgetId, p.instance, cfgObj?.opt("state"),
    )
    // Fetched source data comes from its cache (WidgetSourceWorker refreshes it when due).
    val fetched = WidgetSource.read(context, p.scopedId).data
    WidgetRootBody(
        context,
        effectiveConfig,
        size,
        source,
        stateNonce,
        appWidgetId = appWidgetId,
        group = p.group,
        widgetId = p.widgetId,
        nonce = p.nonce,
        instance = p.instance,
        data = Resolve.mergeData(listOf(cfgData, p.hostData, fetched)),
        state = widgetState,
    )
}

/** Generated preview: the active widget's stored config, data and state (no placement). */
@Composable
internal fun WidgetPreview(context: Context, size: String) {
    val p = Placement.read(context, -1)
    val cfg = Resolve.json(p.configRaw) as? JSONObject
    WidgetRootBody(
        context,
        p.configRaw,
        size,
        source = "preview",
        stateNonce = null,
        widgetId = p.widgetId,
        data = Resolve.mergeData(listOf(cfg?.opt("data"), p.hostData)),
        state = WidgetStoreKeys.readState(
            context.getSharedPreferences(p.group, Context.MODE_PRIVATE), p.widgetId, null, cfg?.opt("state"),
        ),
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
    instance: String? = null,
    data: Any? = null,
    state: Any? = null,
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
    val config = runCatching {
        val raw = JSONObject(effectiveConfig)
        // Harness / unplaced renders have no stored state: the config defaults still apply.
        Resolve.resolve(raw, Resolve.Context(System.currentTimeMillis(), currentLocaleTag(context), instance, data, state ?: raw.opt("state")))
    }.getOrNull()
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
    // Root without its own cornerRadius: the system widget radius like every Apple widget
    // (developer.android.com "Implement rounded corners"; Glance applies it on API 31+).
    val rootMod = if (element.has("cornerRadius")) GlanceModifier.fillMaxSize()
    else GlanceModifier.fillMaxSize().appWidgetBackground().cornerRadius(android.R.dimen.system_app_widget_background_radius)
    Box(modifier = rootMod) {
        val slot = LocalSize.current
        RenderElement(
            RenderScope(context, size, slotW = slot.width.value, slotH = slot.height.value),
            El(element),
            GlanceModifier.fillMaxWidth().fillMaxHeight(),
        )
    }
    WIDGET_SURFACE_DARK.remove()

    if (appWidgetId >= 0 && group.isNotBlank()) {
        val mapNonce = nonce
        val receipt = JSONObject()
            .put("widgetId", widgetId)
            .put("group", group)
            .put("instance", instance ?: appWidgetId.toString())
            .put("appWidgetId", appWidgetId)
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


/** Device locale as BCP-47 (`pt-BR`). */
internal fun currentLocaleTag(context: Context): String {
    val conf = context.resources.configuration
    val loc = if (android.os.Build.VERSION.SDK_INT >= 24) conf.locales[0] else @Suppress("DEPRECATION") conf.locale
    return loc?.toLanguageTag() ?: ""
}
