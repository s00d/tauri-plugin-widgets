package git.s00d.widgets

import android.content.Context

/** Shared storage contract keys (0.4 — no legacy `__widget_config__`). */
object WidgetStoreKeys {
    const val CONFIG_PREFIX = "config:"
    const val PENDING_ACTIONS = "pending_actions"
    const val META_NONCE = "__meta_nonce__"
    const val META_UPDATED_AT = "__meta_updated_at__"
    const val META_PREFS = "__tauri_widget_meta__"
    const val RECEIPTS_PREFS = "__tauri_widget_receipts__"
    const val KEY_ACTIVE_GROUP = "active_group"
    const val KEY_ACTIVE_WIDGET_ID = "active_widget_id"
    const val KEY_REGISTERED = "registered_kinds"
    const val INSTANCE_WIDGET_ID_PREFIX = "widgetId:"
    const val INSTANCE_GROUP_PREFIX = "group:"

    fun configKey(widgetId: String): String = CONFIG_PREFIX + widgetId

    fun isConfigKey(key: String): Boolean = key.startsWith(CONFIG_PREFIX)

    fun instanceWidgetIdKey(appWidgetId: Int): String = INSTANCE_WIDGET_ID_PREFIX + appWidgetId

    fun instanceGroupKey(appWidgetId: Int): String = INSTANCE_GROUP_PREFIX + appWidgetId

    fun hasAnyConfig(prefs: android.content.SharedPreferences): Boolean {
        return prefs.all.keys.any { isConfigKey(it) }
    }

    fun metaPrefs(context: Context) =
        context.getSharedPreferences(META_PREFS, Context.MODE_PRIVATE)

    fun mappedWidgetId(context: Context, appWidgetId: Int): String? {
        if (appWidgetId < 0) return null
        return metaPrefs(context).getString(instanceWidgetIdKey(appWidgetId), null)
    }

    fun mappedGroup(context: Context, appWidgetId: Int): String? {
        if (appWidgetId < 0) return null
        return metaPrefs(context).getString(instanceGroupKey(appWidgetId), null)
    }

    /** Bind an AppWidget instance to a logical widgetId (+ group). */
    fun bindInstance(context: Context, appWidgetId: Int, widgetId: String, group: String?) {
        if (appWidgetId < 0 || widgetId.isBlank()) return
        val ed = metaPrefs(context).edit()
            .putString(instanceWidgetIdKey(appWidgetId), widgetId)
        if (!group.isNullOrBlank()) {
            ed.putString(instanceGroupKey(appWidgetId), group)
        }
        ed.apply()
    }

    fun resolveGroup(context: Context, appWidgetId: Int = -1): String {
        mappedGroup(context, appWidgetId)?.takeIf { it.isNotBlank() }?.let { mapped ->
            val prefs = context.getSharedPreferences(mapped, Context.MODE_PRIVATE)
            if (hasAnyConfig(prefs)) return mapped
        }

        val active = metaPrefs(context).getString(KEY_ACTIVE_GROUP, null)
        if (!active.isNullOrBlank()) {
            val activePrefs = context.getSharedPreferences(active, Context.MODE_PRIVATE)
            if (hasAnyConfig(activePrefs)) return active
        }

        val packageName = context.packageName
        val packageNameHyphen = packageName.replace('_', '-')
        val metaGroup = runCatching {
            val ai = context.packageManager.getReceiverInfo(
                android.content.ComponentName(context, "git.s00d.widgets.TauriGlanceWidgetReceiver"),
                android.content.pm.PackageManager.GET_META_DATA,
            )
            ai.metaData?.getString("tauri_widget_group")
        }.getOrNull()
        val candidates = listOfNotNull(
            metaGroup?.takeIf { it.isNotBlank() },
            packageName,
            "group.$packageName",
            packageNameHyphen,
            "group.$packageNameHyphen",
        ).distinct()
        for (group in candidates) {
            val prefs = context.getSharedPreferences(group, Context.MODE_PRIVATE)
            if (hasAnyConfig(prefs)) return group
        }
        return candidates.first()
    }

    fun resolveWidgetId(context: Context, appWidgetId: Int = -1): String {
        mappedWidgetId(context, appWidgetId)?.takeIf { it.isNotBlank() }?.let { return it }
        return metaPrefs(context).getString(KEY_ACTIVE_WIDGET_ID, null) ?: "default"
    }
}
