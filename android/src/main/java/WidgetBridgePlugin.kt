package git.s00d.widgets

import android.app.Activity
import android.app.PendingIntent
import android.appwidget.AppWidgetManager
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.SharedPreferences
import android.os.Build
import android.util.Log
import android.webkit.WebView
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.glance.appwidget.GlanceAppWidgetManager
import androidx.glance.appwidget.state.updateAppWidgetState
import androidx.glance.appwidget.updateAll
import kotlinx.coroutines.runBlocking
import org.json.JSONArray
import org.json.JSONObject
import java.io.File
import java.io.FileOutputStream
import java.io.ByteArrayOutputStream
import java.net.HttpURLConnection
import java.net.URL
import java.security.MessageDigest

@InvokeArg
class SetItemsRequest {
    var key: String = ""
    var value: String = ""
    var group: String = ""
}

@InvokeArg
class GetItemsRequest {
    var key: String = ""
    var group: String = ""
}

@InvokeArg
class SetRegisterWidgetRequest {
    var widgets: Array<String> = arrayOf()
}

@InvokeArg
class ReloadTimelinesRequest {
    var ofKind: String = ""
}

@InvokeArg
class WidgetActionRequest {
    var action: String = ""
    var payload: String? = null
}

@InvokeArg
class SetWidgetConfigRequest {
    var config: String = ""
    var group: String = ""
    var widgetId: String = "default"
}

@InvokeArg
class GetWidgetConfigRequest {
    var group: String = ""
    var widgetId: String = "default"
}

@InvokeArg
class ReportReceiptRequest {
    var receiptJson: String = ""
}

@InvokeArg
class GroupOnlyRequest {
    var group: String = ""
}

@TauriPlugin
class WidgetBridgePlugin(private val activity: Activity) : Plugin(activity) {

    companion object {
        private const val DEFAULT_IMAGE_CACHE_TTL_MS = 15 * 60 * 1000L
        private const val CONFIG_STATE_KEY_NAME = "__widget_config_state__"
        private const val NONCE_STATE_KEY_NAME = "__widget_nonce_state__"
        private const val TAG = "WidgetBridgePlugin"
        private val CONFIG_STATE_KEY = stringPreferencesKey(CONFIG_STATE_KEY_NAME)
        private val NONCE_STATE_KEY = stringPreferencesKey(NONCE_STATE_KEY_NAME)


        @Volatile
        var pluginInstance: WidgetBridgePlugin? = null
            private set

        /** Plugin commands run on the UI thread: image downloads and DataStore writes go here.
         *  One thread keeps config writes in call order. */
        private val io = java.util.concurrent.Executors.newSingleThreadExecutor()
    }

    private fun getMetaPrefs(): SharedPreferences {
        return activity.applicationContext.getSharedPreferences(WidgetStoreKeys.META_PREFS, Context.MODE_PRIVATE)
    }

    private fun bumpMetaNonce(editor: SharedPreferences.Editor, prefs: SharedPreferences) {
        val next = (prefs.getString(WidgetStoreKeys.META_NONCE, "0")?.toLongOrNull() ?: 0L) + 1L
        editor.putString(WidgetStoreKeys.META_NONCE, next.toString())
        editor.putString(WidgetStoreKeys.META_UPDATED_AT, System.currentTimeMillis().toString())
    }

    private fun cfgHash(text: String?): String {
        if (text.isNullOrEmpty()) return "null"
        return runCatching { sha256Hex(text).take(12) }.getOrDefault("hash_err")
    }

    private fun getPrefs(group: String): SharedPreferences {
        val safeGroup = WidgetSanitizer.sanitizeGroup(group, activity.applicationContext.packageName)
        return activity.applicationContext.getSharedPreferences(safeGroup, Context.MODE_PRIVATE)
    }

    private fun cacheDir(): File {
        val dir = File(activity.applicationContext.cacheDir, "tauri_widget_images")
        if (!dir.exists()) dir.mkdirs()
        return dir
    }

    private fun sha256Hex(value: String): String {
        val digest = MessageDigest.getInstance("SHA-256").digest(value.toByteArray())
        return digest.joinToString("") { "%02x".format(it) }
    }

    private fun cacheImageFromUrl(url: String, ttlMs: Long): String? {
        val safeUrl = url.trim()
        if (!(safeUrl.startsWith("http://") || safeUrl.startsWith("https://"))) return null
        val ext = when {
            safeUrl.contains(".jpg", true) || safeUrl.contains(".jpeg", true) -> "jpg"
            safeUrl.contains(".webp", true) -> "webp"
            else -> "png"
        }
        val file = File(cacheDir(), "${sha256Hex(safeUrl)}.$ext")
        val effectiveTtl = ttlMs.coerceAtLeast(0L)
        if (file.exists() && file.length() > 0) {
            if (effectiveTtl == 0L) return file.absolutePath
            val age = System.currentTimeMillis() - file.lastModified()
            if (age in 0 until effectiveTtl) {
                Log.d(TAG, "image cache hit host=${runCatching { URL(safeUrl).host }.getOrNull()} ageMs=$age ttlMs=$effectiveTtl")
                return file.absolutePath
            }
        }

        val conn = (URL(safeUrl).openConnection() as HttpURLConnection).apply {
            instanceFollowRedirects = true
            connectTimeout = 8_000
            readTimeout = 10_000
            requestMethod = "GET"
        }
        return try {
            conn.connect()
            if (conn.responseCode !in 200..299) return null
            val maxBytes = 3 * 1024 * 1024
            val bytes = conn.inputStream.use { input ->
                val buf = ByteArray(8 * 1024)
                val out = ByteArrayOutputStream()
                var total = 0
                while (true) {
                    val n = input.read(buf)
                    if (n <= 0) break
                    total += n
                    if (total > maxBytes) return null
                    out.write(buf, 0, n)
                }
                out.toByteArray()
            }
            if (bytes.isEmpty()) return null
            FileOutputStream(file).use { it.write(bytes) }
            Log.d(TAG, "image cache refresh ok host=${runCatching { URL(safeUrl).host }.getOrNull()} bytes=${bytes.size}")
            file.absolutePath
        } catch (e: Exception) {
            Log.e(TAG, "image cache refresh failed url=$safeUrl error=${e.message}", e)
            if (file.exists() && file.length() > 0) file.absolutePath else null
        } finally {
            conn.disconnect()
        }
    }

    private fun cacheImagesInJson(node: Any?) {
        when (node) {
            is JSONObject -> {
                val type = node.optString("type", "")
                if (type == "image") {
                    val url = node.optString("url", "")
                    if (url.isNotBlank()) {
                        val ttlMs = when {
                            node.has("cacheTtlMs") -> node.optLong("cacheTtlMs", DEFAULT_IMAGE_CACHE_TTL_MS).coerceAtLeast(0L)
                            node.has("cacheTtlSec") -> {
                                val sec = node.optLong("cacheTtlSec", DEFAULT_IMAGE_CACHE_TTL_MS / 1000L)
                                    .coerceIn(0L, Long.MAX_VALUE / 1000L)
                                sec * 1000L
                            }
                            else -> DEFAULT_IMAGE_CACHE_TTL_MS
                        }
                        val localPath = cacheImageFromUrl(url, ttlMs)
                        if (!localPath.isNullOrBlank()) {
                            node.put("localPath", localPath)
                        }
                    }
                }
                val it = node.keys()
                while (it.hasNext()) {
                    val key = it.next()
                    cacheImagesInJson(node.opt(key))
                }
            }
            is JSONArray -> {
                for (i in 0 until node.length()) {
                    cacheImagesInJson(node.opt(i))
                }
            }
        }
    }

    private fun preprocessWidgetConfig(rawConfig: String): String {
        val root = runCatching { JSONObject(rawConfig) }.getOrNull() ?: return rawConfig
        cacheImagesInJson(root)
        return root.toString()
    }

    override fun load(webView: WebView) {
        super.load(webView)
        pluginInstance = this
    }

    override fun onDestroy() {
        // The process can outlive the Activity: widget taps must queue, not go to a dead WebView.
        if (pluginInstance === this) pluginInstance = null
        super.onDestroy()
    }

    /** True while the WebView can receive events. */
    fun isAlive(): Boolean = !activity.isFinishing && !activity.isDestroyed

    /** Plugin event to the WebView: the JS side drains the queue and emits `widget-action`. */
    fun notifyPendingActions(group: String) {
        val event = JSObject().put("group", group)
        activity.runOnUiThread { trigger("widget-action-pending", event) }
    }

    /** Contract v3: a tap changed a placement's state in-process (`widget-state-changed`). */
    fun notifyStateChanged(widgetId: String, instance: String?, state: Any?) {
        val event = JSObject()
            .put("widgetId", widgetId)
            .put("instance", instance ?: JSONObject.NULL)
            .put("state", state ?: JSONObject.NULL)
        activity.runOnUiThread { trigger("widget-state-changed", event) }
    }

    private fun JSObject.optText(key: String): String? =
        if (!has(key) || isNull(key)) null else optString(key).takeIf { it.isNotEmpty() }

    /** `config.state` defaults of the config this scope resolves to. */
    private fun stateDefaults(prefs: SharedPreferences, widgetId: String, instance: String?): Any? =
        WidgetStoreKeys.lookupKeys(WidgetStoreKeys.CONFIG_PREFIX, widgetId, instance)
            .firstNotNullOfOrNull { prefs.getString(it, null) }
            ?.let { (Resolve.json(it) as? JSONObject)?.opt("state") }

    /**
     * `{group, widgetId, instance?, patch | state}`: `patch` merges into that scope with the
     * `set` operators (`$toggle`, `$inc`, `$dec`), `patch: null` clears the scope, `state`
     * replaces it. Resolves `{state}` (merged).
     */
    @Command
    fun setWidgetState(invoke: Invoke) = io.execute {
        try {
            val args = invoke.getArgs()
            val widgetId = args.optText("widgetId") ?: return@execute invoke.reject("widgetId must not be empty")
            val instance = args.optText("instance")
            val prefs = getPrefs(args.optText("group") ?: "")
            val defaults = stateDefaults(prefs, widgetId, instance)
            val full = args.optJSONObject("state")
            val patch = args.optJSONObject("patch")
            when {
                full != null -> WidgetStoreKeys.writeState(prefs, widgetId, instance, JSONObject(full.toString()), replace = true)
                patch != null -> {
                    val current = WidgetStoreKeys.readState(prefs, widgetId, instance, defaults)
                    WidgetStoreKeys.writeState(prefs, widgetId, instance, WidgetStoreKeys.applySet(current, patch))
                }
                args.has("patch") -> WidgetStoreKeys.clearState(prefs, widgetId, instance)
                else -> return@execute invoke.reject("patch must be an object or null")
            }
            val merged = WidgetStoreKeys.readState(prefs, widgetId, instance, defaults)
            redrawFor(instance?.toIntOrNull())
            invoke.resolve(JSObject().put("state", merged ?: JSONObject.NULL))
        } catch (e: Exception) {
            invoke.reject("Failed to set widget state: ${e.message}")
        }
    }

    @Command
    fun getWidgetState(invoke: Invoke) {
        try {
            val args = invoke.getArgs()
            val widgetId = args.optText("widgetId") ?: return invoke.reject("widgetId must not be empty")
            val instance = args.optText("instance")
            val prefs = getPrefs(args.optText("group") ?: "")
            val merged = WidgetStoreKeys.readState(prefs, widgetId, instance, stateDefaults(prefs, widgetId, instance))
            invoke.resolve(JSObject().put("state", merged ?: JSONObject.NULL))
        } catch (e: Exception) {
            invoke.reject("Failed to get widget state: ${e.message}")
        }
    }

    /** One placement (appWidgetId) or every Tauri widget: state nonce bump + update. */
    private fun redrawFor(appWidgetId: Int?) {
        val context = activity.applicationContext
        if (appWidgetId == null) {
            bumpNonceForAllWidgets()
            runBlocking { TauriGlanceWidget().updateAll(context) }
            return
        }
        runBlocking {
            val glanceId = runCatching { GlanceAppWidgetManager(context).getGlanceIdBy(appWidgetId) }.getOrNull()
                ?: return@runBlocking
            runCatching {
                updateAppWidgetState(context, glanceId) { it[NONCE_STATE_KEY] = System.currentTimeMillis().toString() }
                TauriGlanceWidget().update(context, glanceId)
            }.onFailure { e -> Log.e(TAG, "redraw $appWidgetId failed ${e.message}", e) }
        }
    }

    /** Android 16 Live Updates: may this app post promoted notifications right now? */
    @Command
    fun canPostPromotedNotifications(invoke: Invoke) {
        invoke.resolve(JSObject().put("results", ActivityNotifications.canPostPromoted(activity.applicationContext)))
    }

    /** Opens the system page where the user allows Live Updates for this app. */
    @Command
    fun openPromotionSettings(invoke: Invoke) {
        val ok = runCatching { activity.startActivity(ActivityNotifications.promotionSettingsIntent(activity)) }.isSuccess
        invoke.resolve(JSObject().put("results", ok))
    }

    @Command
    fun setItems(invoke: Invoke) = io.execute { setItemsNow(invoke) }

    private fun setItemsNow(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(SetItemsRequest::class.java)
            val safeKey = WidgetSanitizer.sanitizeKey(args.key)
            val prefs = getPrefs(args.group)
            val editor = prefs.edit()
            var value = args.value
            if (WidgetStoreKeys.isConfigKey(safeKey)) {
                value = preprocessWidgetConfig(value)
            }
            editor.putString(safeKey, value)
            bumpMetaNonce(editor, prefs)
            editor.apply()
            if (WidgetStoreKeys.isConfigKey(safeKey)) {
                val safeGroup = WidgetSanitizer.sanitizeGroup(args.group, activity.applicationContext.packageName)
                val widgetId = safeKey.removePrefix(WidgetStoreKeys.CONFIG_PREFIX)
                getMetaPrefs().edit()
                    .putString(WidgetStoreKeys.KEY_ACTIVE_GROUP, safeGroup)
                    .putString(WidgetStoreKeys.KEY_ACTIVE_WIDGET_ID, widgetId)
                    .apply()
                syncConfigToGlanceState(safeGroup, widgetId, value)
                // Reload is owned by the Rust mobile layer (`skipReload`); do not force here.
            } else if (safeKey.startsWith(WidgetStoreKeys.DATA_PREFIX)) {
                // A running Glance session recomposes only on a state change; WidgetRoot then re-reads data.
                bumpNonceForAllWidgets()
            }
            invoke.resolve(JSObject().put("results", true))
        } catch (e: Exception) {
            invoke.reject("Failed to set item: ${e.message}")
        }
    }

    @Command
    fun getItems(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(GetItemsRequest::class.java)
            val safeKey = WidgetSanitizer.sanitizeKey(args.key)
            val value = getPrefs(args.group).getString(safeKey, null)
            invoke.resolve(JSObject().put("results", value))
        } catch (e: Exception) {
            invoke.reject("Failed to get item: ${e.message}")
        }
    }

    @Command
    fun setRegisterWidget(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(SetRegisterWidgetRequest::class.java)
            if (args.widgets.isEmpty()) {
                invoke.reject("widgets must be a non-empty array")
                return
            }
            getMetaPrefs().edit()
                .putStringSet(WidgetStoreKeys.KEY_REGISTERED, args.widgets.toSet())
                .apply()
            invoke.resolve(JSObject().put("results", true))
        } catch (e: Exception) {
            invoke.reject("Failed to register widgets: ${e.message}")
        }
    }

    @Command
    fun reloadAllTimelines(invoke: Invoke) = io.execute { reloadAllTimelinesNow(invoke) }

    private fun reloadAllTimelinesNow(invoke: Invoke) {
        try {
            Log.d(TAG, "reloadAllTimelines requested")
            reloadGenericWidgets()
            invoke.resolve(JSObject().put("results", true))
        } catch (e: Exception) {
            Log.e(TAG, "reloadAllTimelines failed ${e.message}", e)
            invoke.reject("Failed to reload all timelines: ${e.message}")
        }
    }

    @Command
    fun reloadTimelines(invoke: Invoke) = io.execute { reloadTimelinesNow(invoke) }

    private fun reloadTimelinesNow(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(ReloadTimelinesRequest::class.java)
            Log.d(TAG, "reloadTimelines requested ofKind=${args.ofKind}")
            val registered = getMetaPrefs().getStringSet(WidgetStoreKeys.KEY_REGISTERED, null)
            if (!registered.isNullOrEmpty() && !registered.contains(args.ofKind)) {
                Log.d(TAG, "reloadTimelines skipped: ofKind not in registered set")
                invoke.resolve(JSObject().put("results", false))
                return
            }
            reloadGenericWidgets()
            invoke.resolve(JSObject().put("results", true))
        } catch (e: Exception) {
            Log.e(TAG, "reloadTimelines failed ${e.message}", e)
            invoke.reject("Failed to reload timeline: ${e.message}")
        }
    }

    @Command
    fun requestWidget(invoke: Invoke) {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) {
            invoke.reject("requestWidget requires Android O (API 26) or higher")
            return
        }

        try {
            val context = activity.applicationContext
            val componentName = ComponentName(context, TauriGlanceWidgetReceiver::class.java)
            val appWidgetManager = AppWidgetManager.getInstance(context)

            if (!appWidgetManager.isRequestPinAppWidgetSupported) {
                invoke.reject("Widget pinning is not supported on this launcher")
                return
            }

            val successCallback = PendingIntent.getBroadcast(
                activity,
                0,
                Intent(),
                PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
            )

            // The pin dialog shows the stored config; composing it is off the main thread.
            io.execute {
                try {
                    val sent = runBlocking {
                        GlanceAppWidgetManager(context).requestPinGlanceAppWidget(
                            receiver = TauriGlanceWidgetReceiver::class.java,
                            preview = PinPreviewWidget(),
                            successCallback = successCallback,
                        )
                    }
                    if (!sent) appWidgetManager.requestPinAppWidget(componentName, null, successCallback)
                    invoke.resolve(JSObject().put("results", true))
                } catch (e: Exception) {
                    invoke.reject("Failed to request widget: ${e.message}")
                }
            }
        } catch (e: Exception) {
            invoke.reject("Failed to request widget: ${e.message}")
        }
    }

    @Command
    fun setWidgetConfig(invoke: Invoke) = io.execute { setWidgetConfigNow(invoke) }

    private fun setWidgetConfigNow(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(SetWidgetConfigRequest::class.java)
            if (args.widgetId.isBlank()) {
                invoke.reject("widgetId must not be empty")
                return
            }
            val safeGroup = WidgetSanitizer.sanitizeGroup(args.group, activity.applicationContext.packageName)
            val processedConfig = preprocessWidgetConfig(args.config)
            val key = WidgetStoreKeys.configKey(args.widgetId)
            // `weather#42`: config of placement appWidgetId 42 only.
            val (logicalId, instance) = WidgetStoreKeys.splitScoped(args.widgetId)
            Log.d(
                TAG,
                "setWidgetConfig group=$safeGroup widgetId=${args.widgetId} rawLen=${args.config.length} " +
                    "processedLen=${processedConfig.length}"
            )
            val prefs = activity.applicationContext.getSharedPreferences(safeGroup, Context.MODE_PRIVATE)
            val previous = prefs.getString(key, null)
            if (previous == processedConfig) {
                // Same bytes after prefetch — skip nonce bump / Glance sync / reload.
                getMetaPrefs().edit()
                    .putString(WidgetStoreKeys.KEY_ACTIVE_GROUP, safeGroup)
                    .putString(WidgetStoreKeys.KEY_ACTIVE_WIDGET_ID, logicalId)
                    .apply()
                invoke.resolve(JSObject().put("results", true))
                return
            }
            val editor = prefs.edit()
            editor.putString(key, processedConfig)
            bumpMetaNonce(editor, prefs)
            editor.apply()
            getMetaPrefs().edit()
                .putString(WidgetStoreKeys.KEY_ACTIVE_GROUP, safeGroup)
                .putString(WidgetStoreKeys.KEY_ACTIVE_WIDGET_ID, logicalId)
                .apply()
            if (instance == null) {
                syncConfigToGlanceState(safeGroup, logicalId, processedConfig)
            } else {
                syncInstanceState(instance.toIntOrNull(), logicalId, safeGroup, processedConfig)
            }
            publishPreviews()
            // Reload is owned by the Rust mobile layer (`skipReload`); do not force here.
            invoke.resolve(JSObject().put("results", true))
        } catch (e: Exception) {
            Log.e(TAG, "setWidgetConfig failed ${e.message}", e)
            invoke.reject("Failed to set widget config: ${e.message}")
        }
    }

    private fun syncConfigToGlanceState(group: String, widgetId: String, config: String) {
        val context = activity.applicationContext
        val appWidgetManager = AppWidgetManager.getInstance(context)
        val receiverComponent = ComponentName(context, TauriGlanceWidgetReceiver::class.java)
        val ids = appWidgetManager.getAppWidgetIds(receiverComponent).toMutableList()
        // Prefer recently-confirmed live instances from receipts.
        val live = WidgetReceipts.liveInstances(context, group, widgetId)
        if (live.isNotEmpty()) {
            ids.retainAll(live.toSet())
            // Always include currently mapped ids for this widgetId even if receipt aged out.
            for (id in appWidgetManager.getAppWidgetIds(receiverComponent)) {
                if (WidgetStoreKeys.mappedWidgetId(context, id) == widgetId && id !in ids) {
                    ids.add(id)
                }
            }
        }
        Log.d(TAG, "syncConfigToGlanceState widgetId=$widgetId appWidgetIds=${ids.size} live=${live.size} cfgHash=${cfgHash(config)}")

        runBlocking {
            val manager = GlanceAppWidgetManager(context)
            var idSuccess = 0
            var matched = 0
            var boundUnmapped = 0
            val hasMappedTarget = ids.any { WidgetStoreKeys.mappedWidgetId(context, it) == widgetId }
                || appWidgetManager.getAppWidgetIds(receiverComponent)
                    .any { WidgetStoreKeys.mappedWidgetId(context, it) == widgetId }
            var claimedUnmapped = false
            val allIds = if (ids.isEmpty()) {
                appWidgetManager.getAppWidgetIds(receiverComponent).toList()
            } else {
                ids
            }
            val groupPrefs = context.getSharedPreferences(group, Context.MODE_PRIVATE)
            for (appWidgetId in allIds) {
                // A placement with its own config keeps it (see TauriGlanceWidget.WidgetRoot).
                if (groupPrefs.contains(WidgetStoreKeys.configKey(WidgetStoreKeys.scopedId(widgetId, WidgetStoreKeys.placementInstance(context, appWidgetId))))) {
                    continue
                }
                val mapped = WidgetStoreKeys.mappedWidgetId(context, appWidgetId)
                val shouldUpdate = when {
                    mapped == widgetId && WidgetStoreKeys.mappedGroup(context, appWidgetId).let { g ->
                        g.isNullOrBlank() || g == group
                    } -> true
                    live.contains(appWidgetId) && WidgetStoreKeys.mappedGroup(context, appWidgetId).let { g ->
                        g.isNullOrBlank() || g == group
                    } -> true
                    // No instance yet for this logical id: claim exactly one unmapped slot.
                    mapped.isNullOrBlank() && !hasMappedTarget && !claimedUnmapped -> {
                        WidgetStoreKeys.bindInstance(context, appWidgetId, widgetId, group)
                        claimedUnmapped = true
                        boundUnmapped += 1
                        true
                    }
                    else -> false
                }
                if (!shouldUpdate) continue
                matched += 1
                val glanceId = runCatching { manager.getGlanceIdBy(appWidgetId) }.getOrNull() ?: continue
                runCatching {
                    updateAppWidgetState(context, glanceId) { prefs ->
                        prefs[CONFIG_STATE_KEY] = config
                        prefs[NONCE_STATE_KEY] = System.currentTimeMillis().toString()
                    }
                }.onSuccess { idSuccess += 1 }
                 .onFailure { e -> Log.e(TAG, "updateAppWidgetState by appWidgetId=$appWidgetId failed ${e.message}", e) }
            }
            Log.d(
                TAG,
                "syncConfigToGlanceState done matched=$matched boundUnmapped=$boundUnmapped idSuccess=$idSuccess"
            )
        }
    }

    /** Instance config: refresh only that placement (its own prefs key is already written). */
    private fun syncInstanceState(appWidgetId: Int?, widgetId: String, group: String, config: String) {
        if (appWidgetId == null) return
        val context = activity.applicationContext
        // Map now: an unmapped placement would otherwise resolve another widgetId and miss its own config.
        WidgetStoreKeys.bindInstance(context, appWidgetId, widgetId, group)
        runBlocking {
            val glanceId = runCatching { GlanceAppWidgetManager(context).getGlanceIdBy(appWidgetId) }.getOrNull()
                ?: return@runBlocking
            runCatching {
                updateAppWidgetState(context, glanceId) { prefs ->
                    prefs[CONFIG_STATE_KEY] = config
                    prefs[NONCE_STATE_KEY] = System.currentTimeMillis().toString()
                }
                TauriGlanceWidget().update(context, glanceId)
            }.onFailure { e -> Log.e(TAG, "syncInstanceState $appWidgetId failed ${e.message}", e) }
        }
    }

    /** Android 15+: regenerate the picker previews from the new config (system rate-limits it). */
    private fun publishPreviews() {
        if (Build.VERSION.SDK_INT < 35) return
        runCatching {
            runBlocking { GlanceAppWidgetManager(activity.applicationContext).setWidgetPreviews(TauriGlanceWidgetReceiver::class) }
        }.onFailure { Log.w(TAG, "setWidgetPreviews failed ${it.message}") }
    }

    /** Placed widgets: one entry per appWidgetId (exact). */
    @Command
    fun getActiveWidgets(invoke: Invoke) {
        try {
            val context = activity.applicationContext
            val awm = AppWidgetManager.getInstance(context)
            val ids = awm.getAppWidgetIds(ComponentName(context, TauriGlanceWidgetReceiver::class.java))
            val arr = org.json.JSONArray()
            for (id in ids) {
                val o = awm.getAppWidgetOptions(id)
                val w = o.getInt(AppWidgetManager.OPTION_APPWIDGET_MIN_WIDTH, 0)
                val h = o.getInt(AppWidgetManager.OPTION_APPWIDGET_MIN_HEIGHT, 0)
                val size = TauriGlanceWidget.familyFor(
                    w.toFloat(), h.toFloat(),
                    o.getInt(AppWidgetManager.OPTION_APPWIDGET_HOST_CATEGORY, android.appwidget.AppWidgetProviderInfo.WIDGET_CATEGORY_HOME_SCREEN),
                )
                arr.put(
                    JSObject()
                        .put("widgetId", WidgetStoreKeys.resolveWidgetId(context, id))
                        .put("instance", WidgetStoreKeys.placementInstance(context, id))
                        .put("size", size)
                )
            }
            invoke.resolve(JSObject().put("widgets", arr).put("exactInstances", true))
        } catch (e: Exception) {
            invoke.reject("getActiveWidgets failed: ${e.message}")
        }
    }

    @Command
    fun startActivity(invoke: Invoke) = io.execute { postActivity(invoke, start = true) }

    @Command
    fun updateActivity(invoke: Invoke) = io.execute { postActivity(invoke, start = false) }

    private fun postActivity(invoke: Invoke, start: Boolean) {
        try {
            val args = invoke.getArgs()
            val content = org.json.JSONObject(args.getJSObject("content").toString())
            val group = args.getString("group") ?: ""
            val context = activity.applicationContext
            val id = content.optString("id")
            if (!start && !ActivityNotifications.isActive(context, id)) {
                invoke.reject("activity \"$id\" is not running")
                return
            }
            // `applied: false`: dropped as stale (`timestamp` older than the last applied one).
            val applied = ActivityNotifications.post(context, content, group)
            if (start) {
                invoke.resolve(JSObject().put("id", id).put("surface", "android").put("applied", applied))
            } else {
                invoke.resolve(JSObject().put("results", true).put("applied", applied))
            }
        } catch (e: Exception) {
            invoke.reject(e.message ?: "activity failed")
        }
    }

    // Same `io` queue as start/update, so an end never overtakes a queued start.
    @Command
    fun endActivity(invoke: Invoke) = io.execute {
        val id = invoke.getArgs().getString("id") ?: ""
        invoke.resolve(JSObject().put("ended", ActivityNotifications.end(activity.applicationContext, id)))
    }

    @Command
    fun listActivities(invoke: Invoke) = io.execute {
        val ids = org.json.JSONArray(ActivityNotifications.list(activity.applicationContext))
        invoke.resolve(JSObject().put("ids", ids))
    }

    private fun resolveActiveGroupForReload(): String {
        val context = activity.applicationContext
        val active = getMetaPrefs().getString(WidgetStoreKeys.KEY_ACTIVE_GROUP, null)
        if (!active.isNullOrBlank()) return active
        val pkg = context.packageName
        return "group.${pkg.replace('_', '-')}"
    }

    private fun resolveActiveWidgetId(): String {
        return getMetaPrefs().getString(WidgetStoreKeys.KEY_ACTIVE_WIDGET_ID, null) ?: "default"
    }

    private fun activeConfigKey(): String = WidgetStoreKeys.configKey(resolveActiveWidgetId())

    private fun bumpNonceForAllWidgets() {
        val context = activity.applicationContext
        runBlocking {
            val manager = GlanceAppWidgetManager(context)
            val glanceIds = runCatching { manager.getGlanceIds(TauriGlanceWidget::class.java) }
                .getOrElse { emptyList() }
            val nonce = System.currentTimeMillis().toString()
            var success = 0
            for (glanceId in glanceIds) {
                runCatching {
                    updateAppWidgetState(context, glanceId) { prefs ->
                        prefs[NONCE_STATE_KEY] = nonce
                    }
                }.onSuccess { success += 1 }
                    .onFailure { e -> Log.e(TAG, "bumpNonce failed ${e.message}", e) }
            }
            Log.d(TAG, "bumpNonceForAllWidgets success=$success ids=${glanceIds.size} nonce=$nonce")
        }
    }

    @Command
    fun getWidgetConfig(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(GetWidgetConfigRequest::class.java)
            val widgetId = args.widgetId.ifBlank { "default" }
            val value = getPrefs(args.group).getString(WidgetStoreKeys.configKey(widgetId), null)
            invoke.resolve(JSObject().put("results", value))
        } catch (e: Exception) {
            invoke.reject("Failed to get widget config: ${e.message}")
        }
    }

    @Command
    fun widgetAction(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(WidgetActionRequest::class.java)
            val result = JSObject()
            result.put("action", args.action)
            result.put("payload", args.payload)
            result.put("ts", System.currentTimeMillis())
            result.put("widgetId", "")
            result.put("group", "")
            trigger("widget-action", result)
            invoke.resolve(JSObject().put("results", true))
        } catch (e: Exception) {
            invoke.reject("Failed to emit widget action: ${e.message}")
        }
    }

    @Command
    fun pollPendingActions(invoke: Invoke) = io.execute { pollPendingActionsNow(invoke) }

    private fun pollPendingActionsNow(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(GetWidgetConfigRequest::class.java)
            val prefs = getPrefs(args.group)
            val arr = synchronized(WidgetStoreKeys.QUEUE_LOCK) {
                val pending = prefs.getString(WidgetStoreKeys.PENDING_ACTIONS, "[]") ?: "[]"
                val taken = try { org.json.JSONArray(pending) } catch (_: Exception) { org.json.JSONArray() }
                if (taken.length() > 0) {
                    val editor = prefs.edit()
                    editor.putString(WidgetStoreKeys.PENDING_ACTIONS, "[]")
                    bumpMetaNonce(editor, prefs)
                    editor.commit()
                }
                taken
            }
            val out = org.json.JSONArray()
            if (arr.length() > 0) {
                for (i in 0 until arr.length()) {
                    val item = arr.opt(i)
                    var actionName: String? = null
                    var payload: String? = null
                    var widgetId = ""
                    var group = args.group
                    var ts = System.currentTimeMillis()
                    if (item is org.json.JSONObject) {
                        actionName = if (item.has("action") && !item.isNull("action")) {
                            item.optString("action", "")
                        } else null
                        payload = if (item.has("payload") && !item.isNull("payload")) {
                            item.optString("payload", "")
                        } else null
                        widgetId = item.optString("widgetId", "")
                        group = item.optString("group", args.group)
                        ts = item.optLong("ts", ts)
                    } else if (item is String) {
                        actionName = item
                    }
                    if (actionName.isNullOrEmpty()) continue
                    val event = org.json.JSONObject()
                    event.put("action", actionName)
                    event.put("payload", payload)
                    event.put("ts", ts)
                    event.put("widgetId", widgetId)
                    event.put("group", group)
                    out.put(event)
                }
            }
            invoke.resolve(JSObject().put("results", out))
        } catch (e: Exception) {
            invoke.reject("Failed to poll actions: ${e.message}")
        }
    }

    @Command
    fun reportReceipt(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(ReportReceiptRequest::class.java)
            val receipt = JSONObject(args.receiptJson.ifBlank { "{}" })
            WidgetReceipts.write(activity.applicationContext, receipt)
            invoke.resolve(JSObject().put("results", true))
        } catch (e: Exception) {
            Log.e(TAG, "reportReceipt failed ${e.message}", e)
            invoke.reject("Failed to report receipt: ${e.message}")
        }
    }

    @Command
    fun getWidgetDiagnostics(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(GroupOnlyRequest::class.java)
            val arr = WidgetReceipts.list(activity.applicationContext, args.group)
            invoke.resolve(JSObject().put("results", arr))
        } catch (e: Exception) {
            invoke.reject("Failed to get diagnostics: ${e.message}")
        }
    }

    @Suppress("DEPRECATION")
    private fun reloadGenericWidgets() {
        try {
            val context = activity.applicationContext
            val activeGroup = resolveActiveGroupForReload()
            val cfg = context.getSharedPreferences(activeGroup, Context.MODE_PRIVATE)
                .getString(activeConfigKey(), null)
            Log.d(TAG, "reloadGenericWidgets activeGroup=$activeGroup prefsCfgHash=${cfgHash(cfg)}")
            if (!cfg.isNullOrBlank()) {
                syncConfigToGlanceState(activeGroup, resolveActiveWidgetId(), cfg)
            }
            // Every placement (own-config ones too) must see a state change to recompose.
            bumpNonceForAllWidgets()
            runBlocking {
                val widget = TauriGlanceWidget()
                Log.d(TAG, "reloadGenericWidgets updateAll start")
                widget.updateAll(context)
                Log.d(TAG, "reloadGenericWidgets updateAll done")
            }
        } catch (e: Exception) {
            Log.e(TAG, "reloadGenericWidgets failed ${e.message}", e)
        }
    }
}
