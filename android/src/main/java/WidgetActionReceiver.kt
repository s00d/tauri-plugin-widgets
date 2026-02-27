package git.s00d.widgets

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import org.json.JSONArray

class WidgetActionReceiver : BroadcastReceiver() {
    companion object {
        const val ACTION_WIDGET_ACTION = "git.s00d.widgets.WIDGET_ACTION"
        const val EXTRA_ACTION_NAME = "action_name"
        const val EXTRA_GROUP = "group"
        const val PENDING_KEY = "__widget_pending_actions__"
    }

    override fun onReceive(context: Context, intent: Intent) {
        val actionName = intent.getStringExtra(EXTRA_ACTION_NAME) ?: return
        val groupRaw = intent.getStringExtra(EXTRA_GROUP) ?: return
        val group = WidgetSanitizer.sanitizeGroup(groupRaw, context.packageName)
        val payload = intent.getStringExtra(WidgetBridgePlugin.EXTRA_EVENT_PAYLOAD)

        val plugin = WidgetBridgePlugin.pluginInstance
        if (plugin != null) {
            plugin.emitWidgetAction(actionName, payload)
            return
        }

        val prefs = context.getSharedPreferences(group, Context.MODE_PRIVATE)
        val existing = prefs.getString(PENDING_KEY, "[]") ?: "[]"
        val arr = try { JSONArray(existing) } catch (_: Exception) { JSONArray() }
        if (payload != null) {
            val obj = org.json.JSONObject()
            obj.put("action", actionName)
            obj.put("payload", payload)
            arr.put(obj)
        } else {
            arr.put(actionName)
        }
        prefs.edit().putString(PENDING_KEY, arr.toString()).apply()
    }
}
