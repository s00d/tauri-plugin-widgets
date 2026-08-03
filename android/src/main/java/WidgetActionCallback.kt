package git.s00d.widgets

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.util.Log
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.glance.action.ActionParameters
import androidx.glance.appwidget.AppWidgetId
import androidx.glance.appwidget.action.ActionCallback
import androidx.glance.appwidget.state.updateAppWidgetState
import org.json.JSONArray
import org.json.JSONObject

private const val CONFIG_STATE_KEY_NAME = "__widget_config_state__"
private const val NONCE_STATE_KEY_NAME = "__widget_nonce_state__"
internal val CONFIG_STATE_KEY = stringPreferencesKey(CONFIG_STATE_KEY_NAME)
internal val NONCE_STATE_KEY = stringPreferencesKey(NONCE_STATE_KEY_NAME)

internal val ACTION_KEY = ActionParameters.Key<String>("action")
internal val PAYLOAD_KEY = ActionParameters.Key<String>("payload")
internal val URL_KEY = ActionParameters.Key<String>("url")

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
            val toggled = applyLocalListToggleIfNeeded(
                context, glanceId, group, logicalWidgetId, action, payload,
            )
            if (toggled) {
                Log.d(TAG, "action local list toggle applied action=$action")
            }
            val plugin = WidgetBridgePlugin.pluginInstance
            if (plugin != null) {
                plugin.emitWidgetAction(action, payload, logicalWidgetId, group)
            } else {
                // Queue only when the host plugin is unavailable (matches WidgetActionReceiver).
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
        payload: String?,
    ): Boolean {
        val prefs = context.getSharedPreferences(group, Context.MODE_PRIVATE)
        val raw = prefs.getString(WidgetStoreKeys.configKey(logicalWidgetId), null) ?: return false
        val root = runCatching { JSONObject(raw) }.getOrNull() ?: return false
        var changed = false
        listOf("small", "medium", "large").forEach { key ->
            changed = toggleListItemsInElement(root.optJSONObject(key), action, payload) || changed
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

    private fun toggleListItemsInElement(el: JSONObject?, action: String, payload: String?): Boolean {
        if (el == null) return false
        var changed = false
        if (el.widgetString("type", "") == "list") {
            val items = el.optJSONArray("items") ?: JSONArray()
            for (i in 0 until items.length()) {
                val item = items.optJSONObject(i) ?: continue
                if (item.widgetString("action", "") != action) continue
                if (!payload.isNullOrBlank()) {
                    val itemPayload = item.widgetString("payload", "")
                    if (itemPayload != payload) continue
                }
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
                changed = toggleListItemsInElement(children.optJSONObject(i), action, payload) || changed
            }
        }
        return changed
    }
}
