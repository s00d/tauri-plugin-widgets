package git.s00d.widgets

import android.content.Context

/** Shared storage contract keys (0.4 — no legacy `__widget_config__`). */
object WidgetStoreKeys {
    const val CONFIG_PREFIX = "config:"
    const val PENDING_ACTIONS = "pending_actions"

    /** Guards every read-modify-write of [PENDING_ACTIONS] in this process. */
    val QUEUE_LOCK = Any()

    /** Append one widget action to the group's queue (the host drains it). */
    fun enqueueAction(context: android.content.Context, group: String, action: String, payload: String?, widgetId: String) {
        synchronized(QUEUE_LOCK) {
            val prefs = context.getSharedPreferences(group, android.content.Context.MODE_PRIVATE)
            val raw = prefs.getString(PENDING_ACTIONS, "[]") ?: "[]"
            val arr = runCatching { org.json.JSONArray(raw) }.getOrElse { org.json.JSONArray() }
            val obj = org.json.JSONObject()
            obj.put("action", action)
            if (!payload.isNullOrBlank()) obj.put("payload", payload)
            obj.put("ts", System.currentTimeMillis())
            obj.put("widgetId", widgetId)
            obj.put("group", group)
            arr.put(obj)
            prefs.edit().putString(PENDING_ACTIONS, arr.toString()).commit()
        }
    }
    const val META_NONCE = "__meta_nonce__"
    const val META_UPDATED_AT = "__meta_updated_at__"
    const val META_PREFS = "__tauri_widget_meta__"
    const val RECEIPTS_PREFS = "__tauri_widget_receipts__"
    const val KEY_ACTIVE_GROUP = "active_group"
    const val KEY_ACTIVE_WIDGET_ID = "active_widget_id"
    const val KEY_REGISTERED = "registered_kinds"
    const val INSTANCE_WIDGET_ID_PREFIX = "widgetId:"
    const val INSTANCE_GROUP_PREFIX = "group:"
    /** Meta key: instance id picked in [WidgetConfigActivity] for an appWidgetId. */
    const val INSTANCE_CHOICE_PREFIX = "instanceChoice:"
    /** `instances:{widgetId}`: `[{id, title}]` choices (`setWidgetInstances`). */
    const val INSTANCES_PREFIX = "instances:"

    fun configKey(widgetId: String): String = CONFIG_PREFIX + widgetId

    const val DATA_PREFIX = "data:"
    /** Contract v3 local state: `state:{widgetId}` and `state:{widgetId}#{instance}`. */
    const val STATE_PREFIX = "state:"
    /** Contract v3 activity layouts stored on the device (`registerActivityTemplate`). */
    const val ACTIVITY_TEMPLATE_PREFIX = "activity-template:"

    /** Guards read-modify-write of `state:` keys (tap receiver vs plugin thread). */
    private val STATE_LOCK = Any()

    /** Merged state: `config.state` [defaults], then `state:{id}`, then `state:{id}#{instance}`. */
    fun readState(prefs: android.content.SharedPreferences, widgetId: String, instance: String?, defaults: Any?): Any? =
        Resolve.mergeData(
            listOf(defaults) + lookupKeys(STATE_PREFIX, widgetId, instance).reversed()
                .map { Resolve.json(prefs.getString(WidgetSanitizer.sanitizeKey(it), null)) },
        )

    /**
     * Write the state of exactly this scope (the placement when [instance] is set): [patch]
     * shallow-merged over what is stored, or stored as is when [replace]. Returns the stored object.
     */
    fun writeState(
        prefs: android.content.SharedPreferences,
        widgetId: String,
        instance: String?,
        patch: org.json.JSONObject,
        replace: Boolean = false,
    ): org.json.JSONObject = synchronized(STATE_LOCK) {
        val key = WidgetSanitizer.sanitizeKey(STATE_PREFIX + scopedId(widgetId, instance))
        val base = if (replace) null else Resolve.json(prefs.getString(key, null))
        val next = Resolve.mergeData(listOf(base as? org.json.JSONObject ?: org.json.JSONObject(), patch)) as org.json.JSONObject
        prefs.edit().putString(key, next.toString()).commit()
        next
    }

    /** Drop the stored state of exactly this scope (`setWidgetState` with `patch: null`). */
    fun clearState(prefs: android.content.SharedPreferences, widgetId: String, instance: String?) {
        synchronized(STATE_LOCK) {
            prefs.edit().remove(WidgetSanitizer.sanitizeKey(STATE_PREFIX + scopedId(widgetId, instance))).commit()
        }
    }

    /**
     * Element `set` against the current merged state: `"$toggle"` flips a boolean (missing →
     * `true`), `"$inc"` / `"$dec"` add / subtract 1 (missing → 0 first); other values as is.
     */
    fun applySet(current: Any?, set: org.json.JSONObject): org.json.JSONObject {
        val cur = current as? org.json.JSONObject
        val out = org.json.JSONObject()
        set.keys().forEach { k ->
            val old = cur?.opt(k)
            val v = set.opt(k)
            out.put(k, when (v) {
                "\$toggle" -> if (old is Boolean) !old else true
                "\$inc", "\$dec" -> {
                    val n = (old as? Number) ?: 0
                    val d = if (v == "\$inc") 1 else -1
                    if (n is Int || n is Long) n.toLong() + d else n.toDouble() + d
                }
                else -> v
            })
        }
        return out
    }
    const val FETCHED_PREFS = "__tauri_widget_fetched__"
    /** Matches Rust `store::INSTANCE_SEPARATOR`. */
    const val INSTANCE_SEPARATOR = "#"

    /** `widgetId` or `widgetId#instance`. */
    fun scopedId(widgetId: String, instance: String?): String =
        if (instance.isNullOrEmpty()) widgetId else widgetId + INSTANCE_SEPARATOR + instance

    /** Placement's own key first, then the widget's (Rust `store::lookup_keys`). */
    fun lookupKeys(prefix: String, widgetId: String, instance: String?): List<String> =
        if (instance.isNullOrEmpty()) listOf(prefix + widgetId)
        else listOf(prefix + scopedId(widgetId, instance), prefix + widgetId)

    /** `w#7` → (`w`, `7`); `w` → (`w`, null). */
    fun splitScoped(id: String): Pair<String, String?> {
        val i = id.indexOf(INSTANCE_SEPARATOR)
        return if (i < 0) id to null else id.substring(0, i) to id.substring(i + 1).ifEmpty { null }
    }

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

    /** Clear binding when an AppWidgetId is deleted / recycled. */
    fun clearInstance(context: Context, appWidgetId: Int) {
        if (appWidgetId < 0) return
        metaPrefs(context).edit()
            .remove(instanceWidgetIdKey(appWidgetId))
            .remove(instanceGroupKey(appWidgetId))
            .remove(INSTANCE_CHOICE_PREFIX + appWidgetId)
            .apply()
    }

    fun setPlacementInstance(context: Context, appWidgetId: Int, instance: String) {
        if (appWidgetId < 0 || instance.isEmpty()) return
        metaPrefs(context).edit().putString(INSTANCE_CHOICE_PREFIX + appWidgetId, instance).commit()
    }

    /** Instance id of a placement: the configured choice, else the appWidgetId; null when unplaced. */
    fun placementInstance(context: Context, appWidgetId: Int): String? {
        if (appWidgetId < 0) return null
        return metaPrefs(context).getString(INSTANCE_CHOICE_PREFIX + appWidgetId, null) ?: appWidgetId.toString()
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
