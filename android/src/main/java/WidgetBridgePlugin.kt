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

        const val ACTION_WIDGET_EVENT = "git.s00d.widgets.WIDGET_EVENT_TO_APP"
        const val EXTRA_EVENT_ACTION = "event_action"
        const val EXTRA_EVENT_PAYLOAD = "event_payload"

        @Volatile
        var pluginInstance: WidgetBridgePlugin? = null
            private set
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

    fun emitWidgetAction(action: String, payload: String?, widgetId: String? = null, group: String? = null) {
        val event = JSObject()
        event.put("action", action)
        event.put("payload", payload)
        event.put("ts", System.currentTimeMillis())
        event.put("widgetId", widgetId ?: "")
        event.put("group", group ?: "")
        runCatching {
            activity.runOnUiThread { trigger("widget-action", event) }
        }.onFailure {
            trigger("widget-action", event)
        }
    }

    @Command
    fun setItems(invoke: Invoke) {
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
    fun reloadAllTimelines(invoke: Invoke) {
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
    fun reloadTimelines(invoke: Invoke) {
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

            appWidgetManager.requestPinAppWidget(componentName, null, successCallback)
            invoke.resolve(JSObject().put("results", true))
        } catch (e: Exception) {
            invoke.reject("Failed to request widget: ${e.message}")
        }
    }

    @Command
    fun setWidgetConfig(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(SetWidgetConfigRequest::class.java)
            if (args.widgetId.isBlank()) {
                invoke.reject("widgetId must not be empty")
                return
            }
            val safeGroup = WidgetSanitizer.sanitizeGroup(args.group, activity.applicationContext.packageName)
            val processedConfig = preprocessWidgetConfig(args.config)
            val key = WidgetStoreKeys.configKey(args.widgetId)
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
                    .putString(WidgetStoreKeys.KEY_ACTIVE_WIDGET_ID, args.widgetId)
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
                .putString(WidgetStoreKeys.KEY_ACTIVE_WIDGET_ID, args.widgetId)
                .apply()
            syncConfigToGlanceState(safeGroup, args.widgetId, processedConfig)
            reloadGenericWidgets()
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
            for (appWidgetId in allIds) {
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
    fun pollPendingActions(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(GetWidgetConfigRequest::class.java)
            val prefs = getPrefs(args.group)
            val pending = prefs.getString(WidgetStoreKeys.PENDING_ACTIONS, "[]") ?: "[]"
            val arr = try { org.json.JSONArray(pending) } catch (_: Exception) { org.json.JSONArray() }
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
                val editor = prefs.edit()
                editor.putString(WidgetStoreKeys.PENDING_ACTIONS, "[]")
                bumpMetaNonce(editor, prefs)
                editor.apply()
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
            } else {
                bumpNonceForAllWidgets()
            }
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
