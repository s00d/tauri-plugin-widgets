package git.s00d.widgets

import android.content.Context
import org.json.JSONArray
import org.json.JSONObject

/**
 * Render receipts live outside the config SharedPreferences map so writes never
 * bump `__meta_nonce__`. Prefs name: [WidgetStoreKeys.RECEIPTS_PREFS].
 */
object WidgetReceipts {
    private const val KEY = "receipts"

    /** Max age for "live" instances used by sync targeting (15 min). */
    const val LIVE_MAX_AGE_MS = 15 * 60 * 1000L

    fun write(context: Context, receipt: JSONObject) {
        if (!receipt.has("ts")) {
            receipt.put("ts", System.currentTimeMillis())
        }
        val prefs = context.getSharedPreferences(WidgetStoreKeys.RECEIPTS_PREFS, Context.MODE_PRIVATE)
        val arr = runCatching { JSONArray(prefs.getString(KEY, "[]") ?: "[]") }
            .getOrElse { JSONArray() }
        val instance = receipt.optString("instance", "")
        val group = receipt.optString("group", "")
        // Replace same (group, instance)
        val next = JSONArray()
        for (i in 0 until arr.length()) {
            val item = arr.optJSONObject(i) ?: continue
            if (item.optString("instance") == instance && item.optString("group") == group) {
                continue
            }
            next.put(item)
        }
        next.put(receipt)
        // Cap bag size
        while (next.length() > 64) {
            next.remove(0)
        }
        prefs.edit().putString(KEY, next.toString()).apply()
    }

    fun list(context: Context, group: String? = null): JSONArray {
        val prefs = context.getSharedPreferences(WidgetStoreKeys.RECEIPTS_PREFS, Context.MODE_PRIVATE)
        val arr = runCatching { JSONArray(prefs.getString(KEY, "[]") ?: "[]") }
            .getOrElse { JSONArray() }
        if (group.isNullOrBlank()) return arr
        val out = JSONArray()
        for (i in 0 until arr.length()) {
            val item = arr.optJSONObject(i) ?: continue
            if (item.optString("group") == group) out.put(item)
        }
        return out
    }

    /** Live appWidgetId strings that recently rendered [widgetId] for [group]. */
    fun liveInstances(context: Context, group: String, widgetId: String): List<Int> {
        val now = System.currentTimeMillis()
        val out = mutableListOf<Int>()
        val arr = list(context, group)
        for (i in 0 until arr.length()) {
            val item = arr.optJSONObject(i) ?: continue
            if (item.optString("widgetId") != widgetId) continue
            val ts = item.optLong("ts", 0L)
            if (now - ts > LIVE_MAX_AGE_MS) continue
            item.optString("instance").toIntOrNull()?.let { out.add(it) }
        }
        return out
    }
}
