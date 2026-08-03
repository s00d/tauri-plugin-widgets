package git.s00d.widgets

import org.json.JSONObject

/** Typed read-only view over one widget-IR node; keeps `org.json` quirks in one place. */
@JvmInline
value class El(val raw: JSONObject) {
    val type: String get() = str("type")

    fun str(key: String, def: String = ""): String = raw.widgetString(key, def)

    fun num(key: String): Double? =
        if (!raw.has(key) || raw.isNull(key)) null else raw.optDouble(key).takeIf { !it.isNaN() }

    fun num(key: String, def: Double): Double =
        if (!raw.has(key) || raw.isNull(key)) def else {
            val v = raw.optDouble(key, def); if (v.isNaN()) def else v
        }

    fun dp(key: String, def: Int): Int = num(key)?.toInt()?.coerceAtLeast(0) ?: def

    fun int(key: String, def: Int): Int =
        if (!raw.has(key) || raw.isNull(key)) def else raw.optInt(key, def)

    fun bool(key: String, def: Boolean = false): Boolean =
        if (!raw.has(key) || raw.isNull(key)) def else raw.optBoolean(key, def)

    fun obj(key: String): JSONObject? = raw.optJSONObject(key)

    fun arr(key: String): org.json.JSONArray? = raw.optJSONArray(key)

    fun has(key: String): Boolean = raw.has(key)

    fun isNull(key: String): Boolean = raw.isNull(key)

    fun opt(key: String): Any? = raw.opt(key)

    val children: List<El>
        get() = arr("children")?.let { a ->
            (0 until a.length()).mapNotNull { a.optJSONObject(it)?.let(::El) }
        } ?: emptyList()
}
