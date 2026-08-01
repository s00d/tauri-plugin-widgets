package git.s00d.widgets

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import org.json.JSONArray
import org.json.JSONObject

class WidgetActionReceiver : BroadcastReceiver() {
    companion object {
        const val ACTION_WIDGET_ACTION = "git.s00d.widgets.WIDGET_ACTION"
        const val EXTRA_ACTION_NAME = "action_name"
        const val EXTRA_GROUP = "group"
        const val EXTRA_WIDGET_ID = "widget_id"
    }

    override fun onReceive(context: Context, intent: Intent) {
        val actionName = intent.getStringExtra(EXTRA_ACTION_NAME) ?: return
        val groupRaw = intent.getStringExtra(EXTRA_GROUP) ?: return
        val group = WidgetSanitizer.sanitizeGroup(groupRaw, context.packageName)
        val payload = intent.getStringExtra(WidgetBridgePlugin.EXTRA_EVENT_PAYLOAD)
        val widgetId = intent.getStringExtra(EXTRA_WIDGET_ID) ?: "default"

        val plugin = WidgetBridgePlugin.pluginInstance
        if (plugin != null) {
            plugin.emitWidgetAction(actionName, payload, widgetId, group)
            return
        }

        val prefs = context.getSharedPreferences(group, Context.MODE_PRIVATE)
        val existing = prefs.getString(WidgetStoreKeys.PENDING_ACTIONS, "[]") ?: "[]"
        val arr = try { JSONArray(existing) } catch (_: Exception) { JSONArray() }
        val obj = JSONObject()
        obj.put("action", actionName)
        if (payload != null) obj.put("payload", payload)
        obj.put("ts", System.currentTimeMillis())
        obj.put("widgetId", widgetId)
        obj.put("group", group)
        arr.put(obj)
        prefs.edit().putString(WidgetStoreKeys.PENDING_ACTIONS, arr.toString()).apply()
    }
}
