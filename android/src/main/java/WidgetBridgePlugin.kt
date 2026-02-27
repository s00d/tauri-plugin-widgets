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
}

@InvokeArg
class GetWidgetConfigRequest {
    var group: String = ""
}

@TauriPlugin
class WidgetBridgePlugin(private val activity: Activity) : Plugin(activity) {

    companion object {
        private const val META_PREFS = "__tauri_widget_meta__"
        private const val KEY_ACTIVE_GROUP = "active_group"
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
        return activity.applicationContext.getSharedPreferences(META_PREFS, Context.MODE_PRIVATE)
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
                            node.has("cacheTtlMs") -> node.optLong("cacheTtlMs", DEFAULT_IMAGE_CACHE_TTL_MS)
                            node.has("cacheTtlSec") -> node.optLong("cacheTtlSec", DEFAULT_IMAGE_CACHE_TTL_MS / 1000L) * 1000L
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

    fun emitWidgetAction(action: String, payload: String?) {
        val event = JSObject()
        event.put("action", action)
        event.put("payload", payload)
        runCatching {
            activity.runOnUiThread { trigger("widget-action", event) }
        }.onFailure {
            // Fallback for edge cases when UI thread dispatch is unavailable.
            trigger("widget-action", event)
        }
    }

    @Command
    fun setItems(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(SetItemsRequest::class.java)
            val safeKey = WidgetSanitizer.sanitizeKey(args.key)
            val editor = getPrefs(args.group).edit()
            editor.putString(safeKey, args.value)
            editor.apply()
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
            // Legacy API: ignored after full migration to Glance.
            invoke.parseArgs(SetRegisterWidgetRequest::class.java)
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
            // Legacy kind-based reload removed; Glance updates all instances.
            val args = invoke.parseArgs(ReloadTimelinesRequest::class.java)
            Log.d(TAG, "reloadTimelines requested ofKind=${args.ofKind}")
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
            val safeGroup = WidgetSanitizer.sanitizeGroup(args.group, activity.applicationContext.packageName)
            val processedConfig = preprocessWidgetConfig(args.config)
            Log.d(
                TAG,
                "setWidgetConfig group=$safeGroup rawLen=${args.config.length} rawHash=${cfgHash(args.config)} " +
                    "processedLen=${processedConfig.length} processedHash=${cfgHash(processedConfig)}"
            )
            val editor = activity.applicationContext.getSharedPreferences(safeGroup, Context.MODE_PRIVATE).edit()
            editor.putString("__widget_config__", processedConfig)
            editor.apply()
            getMetaPrefs().edit().putString(KEY_ACTIVE_GROUP, safeGroup).apply()
            syncConfigToGlanceState(processedConfig)
            reloadGenericWidgets()
            invoke.resolve(JSObject().put("results", true))
        } catch (e: Exception) {
            Log.e(TAG, "setWidgetConfig failed ${e.message}", e)
            invoke.reject("Failed to set widget config: ${e.message}")
        }
    }

    private fun syncConfigToGlanceState(config: String) {
        val context = activity.applicationContext
        val appWidgetManager = AppWidgetManager.getInstance(context)
        val receiverComponent = ComponentName(context, TauriGlanceWidgetReceiver::class.java)
        val ids = appWidgetManager.getAppWidgetIds(receiverComponent)
        Log.d(TAG, "syncConfigToGlanceState appWidgetIds=${ids.size} cfgHash=${cfgHash(config)}")

        runBlocking {
            val manager = GlanceAppWidgetManager(context)
            var idSuccess = 0
            for (appWidgetId in ids) {
                val glanceId = runCatching { manager.getGlanceIdBy(appWidgetId) }.getOrNull() ?: continue
                runCatching {
                    updateAppWidgetState(context, glanceId) { prefs ->
                        prefs[CONFIG_STATE_KEY] = config
                        prefs[NONCE_STATE_KEY] = System.currentTimeMillis().toString()
                    }
                }.onSuccess { idSuccess += 1 }
                 .onFailure { e -> Log.e(TAG, "updateAppWidgetState by appWidgetId=$appWidgetId failed ${e.message}", e) }
            }
            // Fallback: update state by all known glance ids too.
            val glanceIds = runCatching { manager.getGlanceIds(TauriGlanceWidget::class.java) }
                .getOrElse { emptyList() }
            var glanceSuccess = 0
            for (glanceId in glanceIds) {
                runCatching {
                    updateAppWidgetState(context, glanceId) { prefs ->
                        prefs[CONFIG_STATE_KEY] = config
                        prefs[NONCE_STATE_KEY] = System.currentTimeMillis().toString()
                    }
                }.onSuccess { glanceSuccess += 1 }
                 .onFailure { e -> Log.e(TAG, "updateAppWidgetState by glanceId failed ${e.message}", e) }
            }
            Log.d(TAG, "syncConfigToGlanceState done idSuccess=$idSuccess glanceSuccess=$glanceSuccess glanceIds=${glanceIds.size}")
        }
    }

    private fun resolveActiveGroupForReload(): String {
        val context = activity.applicationContext
        val active = getMetaPrefs().getString(KEY_ACTIVE_GROUP, null)
        if (!active.isNullOrBlank()) return active
        val pkg = context.packageName
        return "group.${pkg.replace('_', '-')}"
    }

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
            val value = getPrefs(args.group).getString("__widget_config__", null)
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
            val pending = prefs.getString("__widget_pending_actions__", "[]") ?: "[]"
            val arr = try { org.json.JSONArray(pending) } catch (_: Exception) { org.json.JSONArray() }
            val out = org.json.JSONArray()
            if (arr.length() > 0) {
                for (i in 0 until arr.length()) {
                    val item = arr.opt(i)
                    var actionName: String? = null
                    var payload: String? = null
                    if (item is org.json.JSONObject) {
                        actionName = if (item.has("action") && !item.isNull("action")) {
                            item.optString("action", "")
                        } else {
                            null
                        }
                        payload = if (item.has("payload") && !item.isNull("payload")) {
                            item.optString("payload", "")
                        } else {
                            null
                        }
                    } else if (item is String) {
                        actionName = item
                    }
                    if (actionName.isNullOrEmpty()) continue
                    val event = org.json.JSONObject()
                    event.put("action", actionName)
                    event.put("payload", payload)
                    out.put(event)
                }
                prefs.edit().putString("__widget_pending_actions__", "[]").apply()
            }
            // Return actions array in `results` so JS invoke receives payload directly.
            invoke.resolve(JSObject().put("results", out))
        } catch (e: Exception) {
            invoke.reject("Failed to poll actions: ${e.message}")
        }
    }

    @Suppress("DEPRECATION")
    private fun reloadGenericWidgets() {
        try {
            val context = activity.applicationContext
            val activeGroup = resolveActiveGroupForReload()
            val cfg = context.getSharedPreferences(activeGroup, Context.MODE_PRIVATE)
                .getString("__widget_config__", null)
            Log.d(TAG, "reloadGenericWidgets activeGroup=$activeGroup prefsCfgHash=${cfgHash(cfg)}")
            if (!cfg.isNullOrBlank()) {
                syncConfigToGlanceState(cfg)
            } else {
                // Even without config change, bump nonce to force recomposition.
                bumpNonceForAllWidgets()
            }
            runBlocking {
                val widget = TauriGlanceWidget()
                // Official Glance bulk update path.
                Log.d(TAG, "reloadGenericWidgets updateAll start")
                widget.updateAll(context)
                Log.d(TAG, "reloadGenericWidgets updateAll done")
            }
        } catch (e: Exception) {
            Log.e(TAG, "reloadGenericWidgets failed ${e.message}", e)
        }
    }
}
