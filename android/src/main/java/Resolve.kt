package git.s00d.widgets

import org.json.JSONArray
import org.json.JSONObject

/**
 * Port of `src/resolve.rs`: scheduled entries and `{{…}}` placeholders, run on the stored
 * JSON right before rendering. Checked against `tests/fixtures/resolve/` like the Rust,
 * Swift and TypeScript ports.
 */
object Resolve {
    data class Context(
        val nowMs: Long,
        val locale: String,
        val instance: String? = null,
        /** Data layers merged with [mergeData] (JSONObject, JSONArray, String, Number, Boolean). */
        val data: Any? = null,
        /** Placement state merged over `config.state` defaults (contract v3). */
        val state: Any? = null,
    )

    data class SourceRequest(
        val url: String,
        val method: String,
        val headers: Map<String, String>,
        val body: String?,
        val refreshMinutes: Int,
    )

    const val DEFAULT_REFRESH_MINUTES = 30
    const val MAX_SOURCE_BYTES = 256 * 1024
    // `accessory`: Apple Lock Screen family; resolved and kept like the others (unused on Android).
    private val FAMILIES = listOf("small", "medium", "large", "accessory")

    private fun present(v: Any?): Boolean = v != null && v != JSONObject.NULL

    /** Non-negative whole-number `at` (JSON ints only, like Rust `as_u64`). */
    private fun at(v: Any?): Long? = when (v) {
        is Int -> v.toLong().takeIf { it >= 0 }
        is Long -> v.takeIf { it >= 0 }
        else -> null
    }

    private fun entries(config: JSONObject): List<Pair<Long, JSONObject>> {
        val list = config.optJSONArray("entries") ?: return emptyList()
        return (0 until list.length()).mapNotNull { i ->
            val e = list.opt(i) as? JSONObject ?: return@mapNotNull null
            val a = at(e.opt("at")) ?: return@mapNotNull null
            a to e
        }
    }

    fun activeLayouts(config: JSONObject, nowMs: Long): JSONObject {
        val out = JSONObject()
        for (f in FAMILIES) if (present(config.opt(f))) out.put(f, config.opt(f))
        var active: JSONObject? = null
        var activeAt = -1L
        for ((a, e) in entries(config)) {
            if (a <= nowMs && a >= activeAt) {
                active = e
                activeAt = a
            }
        }
        active?.let { e -> for (f in FAMILIES) if (present(e.opt(f))) out.put(f, e.opt(f)) }
        return out
    }

    fun nextChangeMs(config: JSONObject, nowMs: Long): Long? =
        entries(config).map { it.first }.filter { it > nowMs }.minOrNull()

    fun mergeData(layers: List<Any?>): Any? {
        var out: Any? = null
        for (layer in layers) {
            if (!present(layer)) continue
            val acc = out
            out = if (acc is JSONObject && layer is JSONObject) {
                val merged = JSONObject(acc.toString())
                layer.keys().forEach { k -> merged.put(k, layer.opt(k)) }
                merged
            } else {
                layer
            }
        }
        return out
    }

    /** Lower case, `_` → `-`, Unicode / private-use extensions (`-u-…`, `-x-…`) dropped. */
    private fun normalize(tag: String): String {
        val t = tag.trim().replace('_', '-').lowercase()
        val cut = listOf("-u-", "-x-").map { t.indexOf(it) }.filter { it >= 0 }.minOrNull()
        return if (cut == null) t else t.substring(0, cut)
    }

    /** `zh-hant-tw` → `zh-tw`: the tag without a 4-letter script subtag. */
    private fun languageRegion(tag: String): String =
        tag.split('-').filterIndexed { i, p -> !(i == 1 && p.length == 4 && p.all { it in 'a'..'z' }) }.joinToString("-")
    private fun language(tag: String) = tag.split('-').first()

    fun pickLocaleTable(config: JSONObject, locale: String): String? {
        val tables = config.optJSONObject("strings") ?: return null
        val keys = tables.keys().asSequence().toList().sorted()
        if (keys.isEmpty()) return null
        fun find(pred: (String) -> Boolean) = keys.firstOrNull { pred(normalize(it)) }
        val want = normalize(locale)
        if (want.isNotEmpty()) {
            val lang = language(want)
            val lr = languageRegion(want)
            (find { it == want } ?: find { languageRegion(it) == lr } ?: find { it == lang } ?: find { language(it) == lang })?.let { return it }
        }
        (config.opt("defaultLocale") as? String)?.let { def ->
            find { it == normalize(def) }?.let { return it }
        }
        return find { it == "en" } ?: keys.first()
    }

    private fun lookup(root: Any, path: String): Any? {
        if (path.isEmpty()) return root
        var cur: Any? = root
        for (seg in path.split('.')) {
            cur = when (val c = cur) {
                is JSONArray -> if (seg.isNotEmpty() && seg.all { it in '0'..'9' }) {
                    seg.toIntOrNull()?.let { i -> if (i < c.length()) c.opt(i) else null }
                } else null
                is JSONObject -> if (c.has(seg)) c.opt(seg) else null
                else -> null
            }
        }
        return cur
    }

    fun text(v: Any?): String = when {
        !present(v) -> ""
        v is String -> v
        v is Boolean -> v.toString()
        v is Int || v is Long -> v.toString()
        v is Number -> {
            // Whole floats print as integers (JS) up to 1e21; other floats as the shortest
            // round-trip decimal without exponent (`12345678.5`, not `1.23456785E7`).
            val d = v.toDouble()
            val bd = java.math.BigDecimal(d.toString()).stripTrailingZeros()
            when {
                d == Math.rint(d) && Math.abs(d) < 1e21 -> java.math.BigDecimal(d).toPlainString()
                // Like JS: exponent outside 1e-6 ≤ |x| < 1e21 (`4e-323`, `1.5e+21`).
                d != 0.0 && (Math.abs(d) < 1e-6 || Math.abs(d) >= 1e21) ->
                    bd.toString().lowercase().replace("e+", "e").replace(Regex("e(\\d)"), "e+$1")
                else -> bd.toPlainString()
            }
        }
        else -> v.toString()
    }

    /** Renderer contract version: a config with `requires.resolve` above it shows `fallback`. */
    const val CONTRACT_VERSION = 3
    private const val MAX_REPEAT = 50
    private const val MAX_TEMPLATE_DEPTH = 8

    private fun uint(v: Any?): Long? = when (v) {
        is Int -> v.toLong().takeIf { it >= 0 }
        is Long -> v.takeIf { it >= 0 }
        is Number -> v.toDouble().takeIf { it >= 0 && it == Math.rint(it) && it < 9.2e18 }?.toLong()
        else -> null
    }

    private fun split(expr: String): Pair<String, String?> {
        val dot = expr.indexOf('.')
        return if (dot < 0) expr to null else expr.substring(0, dot) to expr.substring(dot + 1)
    }

    private fun number(v: Any?): Double? = if (v is Number) v.toDouble() else null

    private fun truthy(v: Any?): Boolean = when {
        !present(v) -> false
        v is Boolean -> v
        v is Number -> v.toDouble() != 0.0
        v is String -> v.isNotEmpty()
        v is JSONArray -> v.length() > 0
        else -> true
    }

    /** JSON equality; numbers by value, booleans never equal numbers. */
    private fun jsonEq(a: Any?, b: Any?): Boolean = when {
        a is Boolean || b is Boolean -> a == b
        a is Number && b is Number -> a.toDouble() == b.toDouble()
        a is String && b is String -> a == b
        a is JSONArray && b is JSONArray ->
            a.length() == b.length() && (0 until a.length()).all { jsonEq(a.opt(it), b.opt(it)) }
        a is JSONObject && b is JSONObject ->
            a.length() == b.length() && a.keys().asSequence().all { b.has(it) && jsonEq(a.opt(it), b.opt(it)) }
        else -> !present(a) && !present(b)
    }

    private class Scope(
        val strings: JSONObject?,
        val fallback: JSONObject?,
        val ctx: Context,
        val templates: JSONObject?,
        /** Loop / template variables, innermost last. */
        val locals: List<Pair<String, Any?>> = emptyList(),
        val index: Int? = null,
        val depth: Int = 0,
    ) {
        fun child(name: String, value: Any?, index: Int? = this.index, depth: Int = this.depth) =
            Scope(strings, fallback, ctx, templates, locals + (name to value), index, depth)

        fun local(name: String): Pair<String, Any?>? = locals.lastOrNull { it.first == name }

        /** Typed value of an expression; locals shadow the built-in roots. */
        fun lookupExpr(raw: String): Any? {
            val expr = raw.trim()
            val (head, rest) = split(expr)
            local(head)?.let { (_, v) -> return if (present(v)) lookup(v!!, rest ?: "") else null }
            return when {
                expr == "index" -> index
                expr == "locale" -> ctx.locale
                expr == "instance" -> ctx.instance ?: ""
                expr.startsWith("t.") -> {
                    val key = expr.substring(2)
                    listOf(strings, fallback).firstOrNull { it != null && it.has(key) }?.opt(key)
                }
                head == "data" || head == "state" -> {
                    val d = if (head == "data") ctx.data else ctx.state
                    if (!present(d)) null else lookup(d!!, rest ?: "")
                }
                else -> null
            }
        }

        /** `data` / `state` (and their paths), locals and `index` keep JSON types in `bind`. */
        fun typedRoot(expr: String): Boolean = split(expr).first.let { it == "data" || it == "state" || local(it) != null } ||
            (expr == "index" && index != null)

        fun knownRoot(expr: String): Boolean = typedRoot(expr) || expr == "locale" || expr == "instance"

        fun placeholder(raw: String): String {
            val expr = raw.trim()
            if (local(split(expr).first) == null && expr.startsWith("t.")) {
                val key = expr.substring(2)
                for (t in listOf(strings, fallback)) if (t != null && t.has(key)) return text(t.opt(key))
                return key
            }
            if (!knownRoot(expr)) return "{{$expr}}"
            return text(lookupExpr(expr))
        }

        fun interpolate(s: String): String {
            val out = StringBuilder()
            var rest = s
            while (true) {
                val start = rest.indexOf("{{")
                if (start < 0) break
                val end = rest.indexOf("}}", start + 2)
                if (end < 0) break
                out.append(rest, 0, start).append(placeholder(rest.substring(start + 2, end)))
                rest = rest.substring(end + 2)
            }
            return out.append(rest).toString()
        }

        /** Structured condition (see `src/resolve.rs`). Bad shapes are false. */
        fun evalWhen(cond: Any?): Boolean {
            val c = cond as? JSONObject ?: return false
            c.optJSONArray("all")?.let { a -> return (0 until a.length()).all { evalWhen(a.opt(it)) } }
            c.optJSONArray("any")?.let { a -> return (0 until a.length()).any { evalWhen(a.opt(it)) } }
            if (c.has("not")) return !evalWhen(c.opt("not"))
            val path = c.opt("path") as? String ?: return false
            val v = lookupExpr(path)
            val has = present(v)
            val ops = listOf("eq", "ne", "gt", "gte", "lt", "lte", "exists", "in").filter { c.has(it) }
            if (ops.isEmpty()) return truthy(v)
            return ops.all { op ->
                val x = c.opt(op)
                when (op) {
                    "exists" -> has == (x == true)
                    "eq" -> has && jsonEq(v, x)
                    "ne" -> has && !jsonEq(v, x)
                    "in" -> has && x is JSONArray && (0 until x.length()).any { jsonEq(v, x.opt(it)) }
                    else -> {
                        val a = if (v is Boolean) null else number(v)
                        val b = if (x is Boolean) null else number(x)
                        if (a == null || b == null) false else when (op) {
                            "gt" -> a > b
                            "gte" -> a >= b
                            "lt" -> a < b
                            else -> a <= b
                        }
                    }
                }
            }
        }

        /** One array element: zero or more (`repeat` splices, `when` drops, `use` expands). */
        fun expand(el: Any?): List<Any?> {
            val m = el as? JSONObject ?: return listOf(walk(el))
            if (m.has("when") && !evalWhen(m.opt("when"))) return emptyList()
            when (m.opt("type")) {
                "repeat" -> {
                    val each = m.opt("each") as? String ?: return emptyList()
                    val items = lookupExpr(each) as? JSONArray ?: return emptyList()
                    val template = m.optJSONObject("template") ?: return emptyList()
                    val name = (m.opt("as") as? String)?.takeIf { it.isNotEmpty() } ?: "item"
                    val limit = uint(m.opt("limit"))?.coerceAtMost(MAX_REPEAT.toLong())?.toInt() ?: MAX_REPEAT
                    return (0 until minOf(limit, items.length())).flatMap { i ->
                        child(name, items.opt(i), index = i).expand(template)
                    }
                }
                "use" -> {
                    val name = m.opt("template") as? String ?: return emptyList()
                    val tpl = templates?.optJSONObject(name) ?: return emptyList()
                    if (depth >= MAX_TEMPLATE_DEPTH) return emptyList()
                    val args = JSONObject()
                    m.optJSONObject("args")?.let { a -> a.keys().forEach { k -> args.put(k, argValue(a.opt(k)) ?: JSONObject.NULL) } }
                    // Keys on the `use` element override the template root (instance wins).
                    val root = JSONObject(tpl.toString())
                    m.keys().forEach { k -> if (k !in setOf("type", "template", "args", "when")) root.put(k, m.opt(k)) }
                    return child("arg", args, depth = depth + 1).expand(root)
                }
            }
            return listOf(walk(el))
        }

        /** An argument that is exactly one `{{expr}}` keeps the expression's JSON type. */
        fun argValue(v: Any?): Any? {
            val s = v as? String ?: return v
            if (s.length >= 4 && s.startsWith("{{") && s.endsWith("}}")) {
                val inner = s.substring(2, s.length - 2)
                if (!inner.contains("{{") && !inner.contains("}}")) return lookupExpr(inner) ?: ""
            }
            return interpolate(s)
        }

        fun walk(v: Any?): Any? = when (v) {
            is String -> if (v.contains("{{")) interpolate(v) else v
            is JSONArray -> JSONArray().also { a -> for (i in 0 until v.length()) expand(v.opt(i)).forEach { a.put(it) } }
            is JSONObject -> JSONObject().also { o ->
                v.keys().forEach { k -> if (k != "bind" && k != "when") o.put(k, walk(v.opt(k))) }
                v.optJSONObject("bind")?.let { b ->
                    b.keys().forEach { field ->
                        (b.opt(field) as? String)?.let { bound(it) }?.let { o.put(field, it) }
                    }
                }
            }
            else -> v
        }

        /** `bind: {field: expr}`: data / loop / arg values keep their JSON type; other
         *  expressions render as text; a missing value keeps the literal. */
        fun bound(raw: String): Any? {
            val expr = raw.trim()
            if (typedRoot(expr)) return lookupExpr(expr).takeIf { present(it) }
            return placeholder(expr)
        }

        /** A family root: `use` / `repeat` / `when` apply there too (a dropped root is omitted). */
        fun walkRoot(v: Any?): Any? {
            if (v is JSONObject && v.opt("type") == "repeat") {
                return JSONObject().put("type", "vstack").put("children", JSONArray(expand(v)))
            }
            return expand(v).firstOrNull()
        }
    }

    private fun scope(config: JSONObject, ctx: Context): Scope {
        val tables = config.optJSONObject("strings")
        val t = pickLocaleTable(config, ctx.locale)
        val d = pickLocaleTable(config, "")
        return Scope(
            t?.let { tables?.optJSONObject(it) },
            d?.let { tables?.optJSONObject(it) },
            ctx,
            config.optJSONObject("templates"),
        )
    }

    fun interpolate(config: JSONObject, ctx: Context, s: String): String = scope(config, ctx).interpolate(s)

    /** Resolve any JSON value (activity templates): `when` / `repeat` / `use` / `{{…}}` / `bind`. */
    fun resolveValue(value: Any?, ctx: Context, config: JSONObject = JSONObject()): Any? = scope(config, ctx).walk(value)

    /** `requires.resolve` above [CONTRACT_VERSION]: `fallback` (or a built-in notice). */
    private fun effectiveConfig(config: JSONObject): JSONObject {
        val need = uint(config.optJSONObject("requires")?.opt("resolve")) ?: return config
        if (need <= CONTRACT_VERSION) return config
        val out = config.optJSONObject("fallback")?.let { JSONObject(it.toString()) }
            ?: JSONObject().put("small", JSONObject().put("type", "text").put("content", "Update the app to show this widget"))
        for (k in listOf("version", "strings", "defaultLocale")) if (config.has(k)) out.put(k, config.opt(k))
        return out
    }

    /** Active entry, then templates / repeat / when / placeholders / bind; output has
     *  `version` and the size families only. */
    fun resolve(config: JSONObject, ctx: Context): JSONObject {
        val cfg = effectiveConfig(config)
        val sc = scope(cfg, ctx)
        val out = JSONObject()
        out.put("version", if (present(cfg.opt("version"))) cfg.opt("version") else 1)
        val layouts = activeLayouts(cfg, ctx.nowMs)
        for (f in FAMILIES) if (layouts.has(f)) sc.walkRoot(layouts.opt(f))?.let { out.put(f, it) }
        return out
    }

    /** `https://` anywhere; `http://` only to exactly `localhost` / `127.0.0.1`. */
    fun sourceUrlAllowed(url: String): Boolean {
        if (url.startsWith("https://")) return true
        if (!url.startsWith("http://")) return false
        val authority = url.substring(7).takeWhile { it != '/' && it != '?' && it != '#' }
        if (authority.contains('@')) return false
        val host = authority.substringBefore(':')
        val port = if (authority.contains(':')) authority.substringAfter(':') else ""
        return (host.equals("localhost", ignoreCase = true) || host == "127.0.0.1") && port.all { it in '0'..'9' }
    }

    fun sourceRequest(config: JSONObject, ctx: Context): SourceRequest? {
        val src = config.optJSONObject("source") ?: return null
        val rawUrl = src.opt("url") as? String ?: return null
        val url = interpolate(config, ctx, rawUrl)
        if (!sourceUrlAllowed(url)) return null
        val method = if ((src.opt("method") as? String)?.lowercase() == "post") "POST" else "GET"
        val headers = linkedMapOf<String, String>()
        src.optJSONObject("headers")?.let { h ->
            h.keys().forEach { k -> (h.opt(k) as? String)?.let { headers[k] = interpolate(config, ctx, it) } }
        }
        val body = if (method == "POST") (src.opt("body") as? String)?.let { interpolate(config, ctx, it) } else null
        val minutes = at(src.opt("refreshMinutes"))?.coerceIn(15, 1440)?.toInt() ?: DEFAULT_REFRESH_MINUTES
        return SourceRequest(url, method, headers, body, minutes)
    }

    /** Parse any JSON text (object, array, scalar); null when blank or invalid. */
    fun json(raw: String?): Any? {
        if (raw.isNullOrBlank()) return null
        return runCatching { org.json.JSONTokener(raw).nextValue() }.getOrNull()
    }
}
