package git.s00d.widgets

import org.json.JSONArray
import org.json.JSONObject
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import java.io.File

/** Same fixtures as `src/resolve.rs`, the Swift and the TypeScript ports, plus Kotlin-side scenarios. */
@RunWith(RobolectricTestRunner::class)
@Config(sdk = [34])
class ResolveTest {
    private fun obj(s: String) = JSONObject(s)

    /** Order-independent JSON comparison. */
    private fun canon(v: Any?): Any? = when (v) {
        null, JSONObject.NULL -> null
        is JSONObject -> v.keys().asSequence().sorted().associateWith { canon(v.opt(it)) }
        is JSONArray -> (0 until v.length()).map { canon(v.opt(it)) }
        is Number -> v.toDouble()
        else -> v
    }

    private fun ctx(now: Long = 0, locale: String = "en", data: Any? = null, instance: String? = null, state: Any? = null) =
        Resolve.Context(now, locale, instance, data, state)

    /** Given the shared resolve fixtures, when each resolves, then the output matches the expected JSON of every port. */
    @Test
    fun sharedFixtures() {
        val dir = File(System.getProperty("fixtures.root") ?: "../tests/fixtures", "resolve")
        val files = dir.listFiles { f -> f.name.endsWith(".json") }!!.sortedBy { it.name }
        assertTrue("fixtures in $dir", files.size >= 5)
        val ran = mutableListOf<String>()
        for (f in files) {
            val c = obj(f.readText())
            // Fixture format v2: `platforms` allow-list, `skip: {kotlin: reason}`.
            val platforms = c.optJSONArray("platforms")
            if (platforms != null && (0 until platforms.length()).none { platforms.optString(it) == "kotlin" }) {
                println("resolve fixture skipped (platforms): ${f.name}")
                continue
            }
            val skip = c.optJSONObject("skip")
            if (skip != null && skip.has("kotlin")) {
                println("resolve fixture skipped (${skip.optString("kotlin")}): ${f.name}")
                continue
            }
            val config = c.getJSONObject("config")
            val x = c.getJSONObject("ctx")
            val rc = Resolve.Context(
                x.getLong("nowMs"),
                x.optString("locale"),
                if (x.has("instance")) x.getString("instance") else null,
                Resolve.mergeData(listOf(config.opt("data"), x.opt("data"))),
                // `state` beside `data` (case input, or the case root): merged over `config.state`.
                Resolve.mergeData(listOf(config.opt("state"), c.opt("state"), x.opt("state"))),
            )
            ran += f.name
            assertEquals(f.name, canon(c.get("expected")), canon(Resolve.resolve(config, rc)))
            val want = if (c.has("nextChangeMs")) c.getLong("nextChangeMs") else null
            assertEquals(f.name, want, Resolve.nextChangeMs(config, rc.nowMs))
            if (c.has("source") && c.isNull("source")) assertNull(f.name, Resolve.sourceRequest(config, rc))
            c.optJSONObject("source")?.let { s ->
                val got = Resolve.sourceRequest(config, rc)!!
                assertEquals(f.name, s.getString("url"), got.url)
                assertEquals(f.name, s.getString("method"), got.method)
                assertEquals(f.name, canon(s.getJSONObject("headers")), got.headers)
                assertEquals(f.name, s.optString("body"), got.body)
                assertEquals(f.name, s.getInt("refreshMinutes"), got.refreshMinutes)
            }
        }
        println("resolve fixtures executed (kotlin): ${ran.joinToString(",")}")
        assertTrue("no fixture executed", ran.isNotEmpty())
    }

    /** Given a base layout and timed entries, when time passes each `at`, then the layout switches and the scheduler wakes at the next one. */
    @Test
    fun scheduledEntriesSwitchLayoutsAndWakeTheScheduler() {
        val cfg = obj("""{"small":{"c":"a"},"accessory":{"c":"lock"},"entries":[{"at":300,"small":{"c":"c"},"accessory":{"c":"lock2"}},{"at":100,"small":{"c":"b"}},{"at":100,"medium":{"c":"m"}},{"at":-1,"small":{"c":"neg"}},{"at":1.5,"small":{"c":"f"}},{"small":{"c":"noat"}},5]}""")
        fun small(now: Long) = Resolve.activeLayouts(cfg, now).getJSONObject("small").getString("c")
        // Before the first entry: the base layout; next wake at 100.
        assertEquals("a", small(99))
        assertEquals(100L, Resolve.nextChangeMs(cfg, 99))
        assertEquals(100L, WidgetScheduler.nextWakeMs(cfg, null, 99, null))
        // At 100 two entries tie: the later one in the list wins (its medium, base small kept).
        val at100 = Resolve.activeLayouts(cfg, 100)
        assertEquals("a", at100.getJSONObject("small").getString("c"))
        assertEquals("m", at100.getJSONObject("medium").getString("c"))
        assertEquals(300L, Resolve.nextChangeMs(cfg, 100))
        // At 300: the last switch, accessory family included; nothing upcoming.
        assertEquals("c", small(300))
        assertEquals("lock2", Resolve.activeLayouts(cfg, 300).getJSONObject("accessory").getString("c"))
        assertEquals(canon(obj("""{"c":"lock2"}""")), canon(Resolve.resolve(cfg, ctx(now = 300)).get("accessory")))
        assertNull(Resolve.nextChangeMs(cfg, 300))
        assertNull(WidgetScheduler.nextWakeMs(cfg, null, 300, null))
        // A null family in an entry keeps the base; no families → empty.
        assertEquals(mapOf("small" to 1.0), canon(Resolve.activeLayouts(obj("""{"small":1,"entries":[{"at":0,"small":null}]}"""), 0)))
        assertEquals(emptyMap<String, Any>(), canon(Resolve.activeLayouts(obj("{}"), 0)))
        // `at` as Long (real Unix ms, or built in code): 0 counts, negative never does.
        val far = obj("""{"small":{"c":"a"},"entries":[{"at":4102444800000,"small":{"c":"b"}}]}""")
        assertEquals("b", Resolve.activeLayouts(far, 4_102_444_800_000).getJSONObject("small").getString("c"))
        assertEquals(4_102_444_800_000L, Resolve.nextChangeMs(far, 0))
        val longs = JSONObject().put("small", "base").put(
            "entries", JSONArray().put(JSONObject().put("at", 0L).put("small", "zero")).put(JSONObject().put("at", -5L).put("small", "neg")),
        )
        assertEquals("zero", Resolve.activeLayouts(longs, 0).getString("small"))
        assertEquals(0L, Resolve.nextChangeMs(longs, -10))

        // Scheduler: entries vs a `source` refresh.
        val wake = obj("""{"entries":[{"at":5000},{"at":900000}]}""")
        assertEquals(5000L, WidgetScheduler.nextWakeMs(wake, null, 0, null))
        assertNull(WidgetScheduler.nextWakeMs(obj("{}"), null, 0, null))
        // Source due 15 min after the last fetch; an earlier entry wins.
        assertEquals(5000L, WidgetScheduler.nextWakeMs(wake, 0, 0, 15))
        assertEquals(900_000L, WidgetScheduler.nextWakeMs(obj("{}"), 0, 0, 15))
        // Never fetched: one refresh from now.
        assertEquals(1_000 + 900_000L, WidgetScheduler.nextWakeMs(obj("{}"), null, 1_000, 15))
        // Due exactly now counts as overdue; overdue retries in a minute, not in a tight loop.
        assertEquals(900_000L + 60_000L, WidgetScheduler.nextWakeMs(obj("{}"), 0, 900_000, 15))
        assertEquals(10_000_000L + 60_000L, WidgetScheduler.nextWakeMs(obj("{}"), 0, 10_000_000, 15))
    }

    /** Given string tables and device locales with region / script / extension tags, when `{{t.key}}` resolves, then the best table is used. */
    @Test
    fun localeNegotiationPicksTablesAndStrings() {
        val cfg = obj("""{"defaultLocale":"de","strings":{"en":{},"de":{},"pt-BR":{},"ru":{}}}""")
        assertEquals("pt-BR", Resolve.pickLocaleTable(cfg, "pt_BR"))
        assertEquals("pt-BR", Resolve.pickLocaleTable(cfg, "PT-pt"))
        assertEquals("pt-BR", Resolve.pickLocaleTable(cfg, "pt-BR-u-ca-gregory"))
        assertEquals("ru", Resolve.pickLocaleTable(cfg, "ru-RU"))
        assertEquals("de", Resolve.pickLocaleTable(cfg, "fr"))
        assertEquals("de", Resolve.pickLocaleTable(cfg, ""))
        assertEquals("en", Resolve.pickLocaleTable(obj("""{"strings":{"fr":{},"en":{}}}"""), "ja"))
        assertEquals("de", Resolve.pickLocaleTable(obj("""{"strings":{"fr":{},"de":{}}}"""), "ja"))
        assertEquals("de", Resolve.pickLocaleTable(obj("""{"defaultLocale":"xx","strings":{"fr":{},"de":{}}}"""), "ja"))
        assertNull(Resolve.pickLocaleTable(obj("{}"), "en"))
        assertNull(Resolve.pickLocaleTable(obj("""{"strings":{}}"""), "en"))
        // A 4-letter alphabetic script subtag is ignored for the region match (`a`…`z` inclusive) ...
        val scripts = obj("""{"strings":{"az":{},"az-IR":{},"zh":{},"zh-TW":{},"de":{},"de-CH":{}}}""")
        assertEquals("az-IR", Resolve.pickLocaleTable(scripts, "az-Arab-IR"))
        assertEquals("zh-TW", Resolve.pickLocaleTable(scripts, "zh-Zyyy-TW"))
        // ... a 4-digit variant is not a script.
        assertEquals("de", Resolve.pickLocaleTable(scripts, "de-1996-CH"))
        // A tag that is only an extension normalizes to empty (default table, not a literal match).
        val odd = obj("""{"strings":{"-u-x":{},"en":{}}}""")
        assertEquals("en", Resolve.pickLocaleTable(odd, "-u-x"))
        assertEquals("en", Resolve.pickLocaleTable(odd, ""))

        // Picked table first, then the default table, then the key itself.
        val t = obj("""{"defaultLocale":"en","strings":{"en":{"a":"A","b":"B","n":3},"ru":{"a":"А"}}}""")
        assertEquals("АBc3", Resolve.interpolate(t, ctx(locale = "ru"), "{{t.a}}{{t.b}}{{t.c}}{{t.n}}"))
        assertEquals("ABc", Resolve.interpolate(t, ctx(locale = "en"), "{{t.a}}{{t.b}}{{t.c}}"))
    }

    /** Given data layers and placeholders, when text and `bind` resolve, then values print JS-like and binds keep their type. */
    @Test
    fun placeholdersFormatAndBindKeepTypes() {
        val data = obj(
            """{"t":72.0,"f":1.5,"i":7,"big":12345678901,"b":true,"z":false,"n":null,"list":["x","y"],"o":{"k":1},
              "tiny":1e-6,"tinier":1e-7,"huge":1e21,"huger":1.5e21,"frac":12345678.5,
              "ten":["i0","i1","i2","i3","i4","i5","i6","i7","i8","i9"]}""",
        )
        fun fmt(s: String) = Resolve.interpolate(obj("{}"), ctx(data = data), s)
        assertEquals(
            """72|1.5|7|12345678901|true|false||y|{"k":1}|||72||""",
            fmt("{{data.t}}|{{data.f}}|{{data.i}}|{{data.big}}|{{data.b}}|{{data.z}}|{{data.n}}|{{data.list.1}}|{{data.o}}|{{data.missing}}|{{data.list.9}}|{{ data.t }}|{{data.list.x}}|{{data.t.deep}}"),
        )
        // Exponent only outside 1e-6 ≤ |x| < 1e21, like JS `String(x)`.
        assertEquals("0.000001|1e-7|1e+21|1.5e+21|12345678.5", fmt("{{data.tiny}}|{{data.tinier}}|{{data.huge}}|{{data.huger}}|{{data.frac}}"))
        // Array indices are decimal digits 0…9, inside the array.
        assertEquals("i0|i9||", fmt("{{data.ten.0}}|{{data.ten.9}}|{{data.ten.10}}|{{data.ten.-1}}"))
        assertEquals("{{oops}} ru {{data.x", Resolve.interpolate(obj("{}"), ctx(locale = "ru"), "{{oops}} {{locale}} {{data.x"))
        assertEquals("[nyc]", Resolve.interpolate(obj("{}"), ctx(instance = "nyc"), "[{{instance}}]"))
        assertEquals("[]{{index}}", Resolve.interpolate(obj("{}"), ctx(), "[{{instance}}]{{data.a}}{{index}}"))
        assertEquals("5", Resolve.interpolate(obj("{}"), ctx(data = 5), "{{data}}"))
        assertEquals(mapOf("version" to 2.0), canon(Resolve.resolve(obj("""{"version":2}"""), ctx())))
        assertEquals(mapOf("version" to 1.0), canon(Resolve.resolve(obj("{}"), ctx())))
        val walked = Resolve.resolve(obj("""{"small":{"a":["{{locale}}",1,null],"n":3}}"""), ctx(locale = "ru"))
        assertEquals(mapOf("a" to listOf("ru", 1.0, null), "n" to 3.0), canon(walked.get("small")))

        // `bind`: data / state keep JSON types, other roots render text, missing keeps the literal.
        val bound = Resolve.resolve(
            obj("""{"small":{"type":"gauge","value":0.1,"label":"L","bind":{"value":"data.f","isOn":"state.on","label":"locale","miss":"data.nope","raw":"t.k","bad":5}}}"""),
            ctx(locale = "ru", data = data, state = obj("""{"on":true}""")),
        ).getJSONObject("small")
        assertEquals(1.5, bound.get("value"))
        assertEquals(true, bound.get("isOn"))
        assertEquals("ru", bound.get("label"))
        assertFalse(bound.has("miss"))
        assertEquals("k", bound.get("raw"))
        assertFalse(bound.has("bind"))

        // Activity templates resolve any JSON value the same way.
        val tpl = Resolve.resolveValue(obj("""{"title":"Hi {{data.list.0}}","keep":4}"""), ctx(data = data)) as JSONObject
        assertEquals("Hi x", tpl.getString("title"))
        assertEquals(4, tpl.getInt("keep"))
        assertEquals("plain", Resolve.resolveValue("plain", ctx()))

        // Data layers: later objects merge over earlier ones; a scalar replaces; inputs untouched.
        assertEquals(mapOf("x" to 1.0, "y" to 2.0), canon(Resolve.mergeData(listOf(obj("""{"x":1,"y":1}"""), null, JSONObject.NULL, obj("""{"y":2}""")))))
        assertEquals(5, Resolve.mergeData(listOf(obj("""{"x":1}"""), 5)))
        assertEquals(mapOf("y" to 2.0), canon(Resolve.mergeData(listOf(5, obj("""{"y":2}""")))))
        assertNull(Resolve.mergeData(emptyList()))
        val a = obj("""{"x":1}""")
        Resolve.mergeData(listOf(a, obj("""{"x":2}""")))
        assertEquals(1, a.getInt("x"))
        assertNull(Resolve.json(null))
        assertNull(Resolve.json("  "))
        assertNull(Resolve.json("{bad"))
        assertEquals(5, Resolve.json("5"))
    }

    /** Given `when`, `repeat` and `use` nodes, when the tree resolves, then conditions, loop limits / names and template args / depth follow the contract. */
    @Test
    fun conditionsRepeatAndTemplatesExpand() {
        val data = obj(
            """{"n":5,"s":"x","e":"","b":true,"z":0,"arr":[1,"a"],"arrN":[null],"none":[],"o":{"k":[1,2]},"nul":null,
              "items":[{"v":1},{"v":2},{"v":3}]}""",
        )
        val cases = listOf(
            """{"path":"data.n","lt":5}""" to false, """{"path":"data.n","lt":6}""" to true,
            """{"path":"data.n","lte":5}""" to true, """{"path":"data.n","lte":4}""" to false,
            """{"path":"data.n","gt":5}""" to false, """{"path":"data.n","gt":4}""" to true,
            """{"path":"data.n","gte":5}""" to true, """{"path":"data.n","gte":6}""" to false,
            """{"path":"data.n","gte":1,"lte":9}""" to true, """{"path":"data.n","gte":1,"lte":4}""" to false,
            // Comparisons need two numbers; booleans are not numbers.
            """{"path":"data.s","gt":1}""" to false, """{"path":"data.n","gt":"1"}""" to false,
            """{"path":"data.b","gt":0}""" to false, """{"path":"data.n","lt":true}""" to false,
            // Truthiness.
            """{"path":"data.s"}""" to true, """{"path":"data.e"}""" to false, """{"path":"data.z"}""" to false,
            """{"path":"data.n"}""" to true, """{"path":"data.none"}""" to false, """{"path":"data.arr"}""" to true,
            """{"path":"data.o"}""" to true, """{"path":"data.nul"}""" to false, """{"path":"data.missing"}""" to false,
            """{"path":"data.b"}""" to true,
            // Equality: numbers by value, deep arrays / objects, null only when present.
            """{"path":"data.n","eq":5.0}""" to true, """{"path":"data.b","eq":1}""" to false, """{"path":"data.s","eq":1}""" to false,
            """{"path":"data.arr","eq":[1,"a"]}""" to true, """{"path":"data.arr","eq":[1,"b"]}""" to false,
            """{"path":"data.arr","eq":[1]}""" to false, """{"path":"data.arrN","eq":[null]}""" to true,
            """{"path":"data.arrN","eq":[1]}""" to false,
            """{"path":"data.o","eq":{"k":[1,2]}}""" to true, """{"path":"data.o","eq":{"k":[1,3]}}""" to false,
            """{"path":"data.o","eq":{"j":[1,2]}}""" to false, """{"path":"data.o","eq":{"k":[1,2],"x":1}}""" to false,
            """{"path":"data.nul","eq":null}""" to false, """{"path":"data.missing","ne":1}""" to false,
            """{"path":"data.n","ne":4}""" to true, """{"path":"data.n","ne":5}""" to false,
            """{"path":"data.s","in":["y","x"]}""" to true, """{"path":"data.s","in":["y"]}""" to false,
            """{"path":"data.s","in":"x"}""" to false,
            """{"path":"data.missing","exists":false}""" to true, """{"path":"data.n","exists":true}""" to true,
            """{"path":"data.n","exists":false}""" to false,
            // Combinators and bad shapes.
            """{"all":[{"path":"data.n"},{"path":"data.s"}]}""" to true, """{"all":[{"path":"data.n"},{"path":"data.e"}]}""" to false,
            """{"all":[]}""" to true, """{"any":[{"path":"data.e"},{"path":"data.s"}]}""" to true,
            """{"any":[{"path":"data.e"},{"path":"data.z"}]}""" to false, """{"any":[]}""" to false,
            """{"not":{"path":"data.n"}}""" to false, """{"not":{"path":"data.e"}}""" to true,
            """{"path":5}""" to false, "\"data.n\"" to false, """{"all":5}""" to false,
        )
        val kids = JSONArray(cases.mapIndexed { i, (cond, _) -> obj("""{"type":"text","content":"c$i","when":$cond}""") })
        val shown = Resolve.resolve(JSONObject().put("small", JSONObject().put("type", "vstack").put("children", kids)), ctx(data = data))
            .getJSONObject("small").getJSONArray("children")
        val got = (0 until shown.length()).map { shown.getJSONObject(it).getString("content") }.toSet()
        cases.forEachIndexed { i, (cond, want) -> assertEquals(cond, want, "c$i" in got) }

        // repeat: `limit` must be a whole non-negative number (else the 50 cap), `as` names the item.
        fun rep(extra: JSONObject.() -> Unit): List<String> {
            val r = obj("""{"type":"repeat","each":"data.items","template":{"type":"text","content":"{{index}}:{{item.v}}"}}""").apply(extra)
            val out = Resolve.resolve(JSONObject().put("small", JSONObject().put("type", "hstack").put("children", JSONArray().put(r))), ctx(data = data))
            val k = out.getJSONObject("small").getJSONArray("children")
            return (0 until k.length()).map { k.getJSONObject(it).getString("content") }
        }
        val all = listOf("0:1", "1:2", "2:3")
        assertEquals(all, rep {})
        assertEquals(emptyList<String>(), rep { put("limit", 0) })
        assertEquals(emptyList<String>(), rep { put("limit", 0L) })
        assertEquals(emptyList<String>(), rep { put("limit", 0.0) })
        assertEquals(all.take(2), rep { put("limit", 2.0) })
        assertEquals(all.take(1), rep { put("limit", 1L) })
        for (bad in listOf<Any>(2.5, -1, -1.0, -5_000_000_000L, "2", true)) assertEquals("limit $bad", all, rep { put("limit", bad) })
        assertEquals(all, rep { put("as", "") })
        // Renamed item: `item` is no longer a root, the placeholder stays literal.
        assertEquals(listOf("0:{{item.v}}", "1:{{item.v}}", "2:{{item.v}}"), rep { put("as", "row") })
        assertEquals(emptyList<String>(), rep { put("each", "data.n") })
        assertEquals(emptyList<String>(), rep { remove("each") })
        assertEquals(emptyList<String>(), rep { remove("template") })
        assertEquals(emptyList<String>(), rep { put("when", obj("""{"path":"data.e"}""")) })
        val sixty = JSONObject().put("l", JSONArray((1..60).toList()))
        val capped = Resolve.resolve(
            obj("""{"small":{"type":"repeat","each":"data.l","limit":99,"template":{"type":"text","content":"{{item}}"}}}"""),
            ctx(data = sixty),
        ).getJSONObject("small")
        // A repeat at the family root becomes a vstack.
        assertEquals("vstack", capped.getString("type"))
        assertEquals(50, capped.getJSONArray("children").length())

        // use: args keep the type of a lone `{{expr}}`, keys on `use` override the template root.
        val tcfg = obj(
            """{"templates":{
                 "row":{"type":"text","color":"blue","content":"{{arg.label}}|{{arg.lit}}|{{arg.empty}}|{{arg.miss}}|{{arg.two}}","bind":{"value":"arg.n"}},
                 "nest":{"type":"vstack","children":[{"type":"use","template":"nest"}]}},
               "small":{"type":"use","template":"row","color":"red","when":{"path":"data.n"},
                 "args":{"label":"{{data.s}}-x","n":"{{data.n}}","lit":5,"empty":"{{}}","miss":"{{data.nope}}","two":"{{data.s}}{{data.s}}"}},
               "medium":{"type":"use","template":"nest"},
               "large":{"type":"use","template":"nope"}}""",
        )
        val out = Resolve.resolve(tcfg, ctx(data = data))
        val row = out.getJSONObject("small")
        assertEquals("x-x|5|||xx", row.getString("content"))
        assertEquals(5, row.get("value"))
        assertEquals("red", row.getString("color"))
        assertFalse(row.has("template") || row.has("args") || row.has("when"))
        // Unknown template: the family is dropped. Self-recursion stops at 8 levels.
        assertFalse(out.has("large"))
        var depth = 0
        var node: JSONObject? = out.getJSONObject("medium")
        while (node != null) {
            depth++
            node = node.getJSONArray("children").optJSONObject(0)
        }
        assertEquals(8, depth)
        // `use` dropped by its own `when`, or without a template name.
        assertFalse(Resolve.resolve(obj("""{"templates":{"t":{"type":"text"}},"small":{"type":"use","template":"t","when":{"path":"data.e"}}}"""), ctx(data = data)).has("small"))
        assertFalse(Resolve.resolve(obj("""{"templates":{"t":{"type":"text"}},"small":{"type":"use","template":5}}"""), ctx()).has("small"))
        assertNotNull(Resolve.resolve(obj("""{"templates":{"t":{"type":"text"}},"small":{"type":"use","template":"t"}}"""), ctx()).optJSONObject("small"))
    }

    /** Given `requires.resolve` above the renderer contract, when it resolves, then `fallback` (or a notice) shows, keeping version / strings. */
    @Test
    fun newerContractFallsBack() {
        assertEquals(3, Resolve.CONTRACT_VERSION)
        fun shown(requires: String, withFallback: Boolean = true): String {
            val cfg = obj(
                """{"version":2,"defaultLocale":"en","strings":{"en":{"old":"Old app"}},"requires":{"resolve":$requires},
                  "small":{"type":"text","content":"new"}}""",
            )
            if (withFallback) cfg.put("fallback", obj("""{"small":{"type":"text","content":"{{t.old}}"}}"""))
            val out = Resolve.resolve(cfg, ctx())
            assertEquals(requires, 2, out.getInt("version"))
            return out.getJSONObject("small").getString("content")
        }
        for (ok in listOf("3", "0", "3.5", "-4.0", "9.2e18", "1e19", "\"9\"")) assertEquals(ok, "new", shown(ok))
        for (newer in listOf("4", "4.0", "9000000000")) assertEquals(newer, "Old app", shown(newer))
        assertEquals("Update the app to show this widget", shown("4", withFallback = false))
        // No version on the newer config: defaults to 1, like any config.
        assertEquals(1, Resolve.resolve(obj("""{"requires":{"resolve":4}}"""), ctx()).getInt("version"))
    }

    /** Given `source` configs, when the request is built, then only https / dev localhost are allowed and method, headers and refresh are templated and clamped. */
    @Test
    fun remoteSourceRules() {
        fun get(src: String) = Resolve.sourceRequest(obj("""{"source":$src}"""), ctx(locale = "ru"))
        assertNull(get("""{"url":"http://example.com"}"""))
        for (bad in listOf("http://localhost.evil.com/a", "http://localhost@evil.com/a", "http://127.0.0.1.nip.io/a", "http://localhost:80x/a", "ftp://localhost/a", "localhost")) {
            assertFalse(bad, Resolve.sourceUrlAllowed(bad))
        }
        assertTrue(Resolve.sourceUrlAllowed("http://LOCALHOST"))
        assertTrue(Resolve.sourceUrlAllowed("http://127.0.0.1:8080?x=1"))
        assertTrue(Resolve.sourceUrlAllowed("https://anything"))
        assertNull(get("{}"))
        assertNull(get("""{"url":5}"""))
        assertNull(Resolve.sourceRequest(obj("{}"), ctx()))
        assertEquals(Resolve.SourceRequest("http://localhost:1/a", "GET", emptyMap(), null, 30), get("""{"url":"http://localhost:1/a"}"""))
        assertNotNull(get("""{"url":"http://127.0.0.1/a"}"""))
        assertEquals(
            Resolve.SourceRequest("https://a/ru", "POST", mapOf("A" to "ru"), "ru", 15),
            get("""{"url":"https://a/{{locale}}","method":"post","body":"{{locale}}","refreshMinutes":1,"headers":{"A":"{{locale}}","B":1}}"""),
        )
        assertEquals(1440, get("""{"url":"https://a","body":"x","refreshMinutes":99999}""")!!.refreshMinutes)
        assertNull(get("""{"url":"https://a","body":"x"}""")!!.body)
        assertEquals(30, get("""{"url":"https://a","refreshMinutes":-5}""")!!.refreshMinutes)
        assertEquals(20, get("""{"url":"https://a","refreshMinutes":20}""")!!.refreshMinutes)
    }
}
