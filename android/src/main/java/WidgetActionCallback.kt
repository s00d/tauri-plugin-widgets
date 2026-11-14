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
import org.json.JSONObject

private const val CONFIG_STATE_KEY_NAME = "__widget_config_state__"
private const val NONCE_STATE_KEY_NAME = "__widget_nonce_state__"
internal val CONFIG_STATE_KEY = stringPreferencesKey(CONFIG_STATE_KEY_NAME)
internal val NONCE_STATE_KEY = stringPreferencesKey(NONCE_STATE_KEY_NAME)

internal val ACTION_KEY = ActionParameters.Key<String>("action")
internal val PAYLOAD_KEY = ActionParameters.Key<String>("payload")
internal val URL_KEY = ActionParameters.Key<String>("url")
internal val SET_KEY = ActionParameters.Key<String>("set")

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
        val set = parameters[SET_KEY]
        val normalizedUrl = url?.trim().orEmpty().let {
            if (it.equals("null", ignoreCase = true) || it.equals("undefined", ignoreCase = true)) "" else it
        }
        val logicalWidgetId = WidgetStoreKeys.resolveWidgetId(context, appWidgetId)

        // Contract v3 `set`: write the placement's state in-process first, then the action.
        val stateChanged = !set.isNullOrBlank() && applySet(context, appWidgetId, set)

        // Action-driven buttons are the primary flow in widgets.
        // Only attempt URL open when no action is provided.
        if (!action.isNullOrBlank()) {
            // Always queue (one delivery path, nothing lost). A running app is woken to drain
            // it now; otherwise the host drains on Ready/Resumed.
            WidgetStoreKeys.enqueueAction(context, group, action, payload, logicalWidgetId)
            WidgetBridgePlugin.pluginInstance?.takeIf { it.isAlive() }?.notifyPendingActions(group)
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

        if (stateChanged) {
            // A running session recomposes only on a Glance state change; WidgetRoot re-reads prefs.
            runCatching {
                updateAppWidgetState(context, glanceId) { it[NONCE_STATE_KEY] = System.currentTimeMillis().toString() }
            }.onFailure { e -> Log.e(TAG, "state nonce bump failed=${e.message}", e) }
        }
        runCatching { TauriGlanceWidget().update(context, glanceId) }
            .onFailure { e -> Log.e(TAG, "action-triggered update failed=${e.message}", e) }
    }

    /** Apply element `set` to the placement's stored state; false when nothing was written. */
    private fun applySet(context: Context, appWidgetId: Int, setRaw: String): Boolean {
        val set = Resolve.json(setRaw) as? JSONObject ?: return false
        val p = Placement.read(context, appWidgetId)
        val prefs = context.getSharedPreferences(p.group, Context.MODE_PRIVATE)
        val defaults = (Resolve.json(p.configRaw) as? JSONObject)?.opt("state")
        val current = WidgetStoreKeys.readState(prefs, p.widgetId, p.instance, defaults)
        WidgetStoreKeys.writeState(prefs, p.widgetId, p.instance, WidgetStoreKeys.applySet(current, set))
        val merged = WidgetStoreKeys.readState(prefs, p.widgetId, p.instance, defaults)
        WidgetBridgePlugin.pluginInstance?.takeIf { it.isAlive() }?.notifyStateChanged(p.widgetId, p.instance, merged)
        return true
    }
}
