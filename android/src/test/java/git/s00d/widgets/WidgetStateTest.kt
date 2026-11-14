package git.s00d.widgets

import android.content.Context
import android.content.SharedPreferences
import androidx.glance.action.actionParametersOf
import androidx.glance.appwidget.AppWidgetId
import androidx.test.core.app.ApplicationProvider
import io.kotest.property.Arb
import io.kotest.property.PropTestConfig
import io.kotest.property.arbitrary.bind
import io.kotest.property.arbitrary.choice
import io.kotest.property.arbitrary.element
import io.kotest.property.arbitrary.list
import io.kotest.property.arbitrary.map
import io.kotest.property.checkAll
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.test.runTest
import org.json.JSONArray
import org.json.JSONObject
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

/** Contract v3 local state: placement → `set` taps → render, layered over widget state and defaults. */
@RunWith(RobolectricTestRunner::class)
@Config(sdk = [34])
class WidgetStateTest {
    private lateinit var c: Context

    @Before
    fun setUp() {
        c = ApplicationProvider.getApplicationContext()
    }

    private fun prefs(g: String) = c.getSharedPreferences(g, Context.MODE_PRIVATE)

    private val layout = """{"state":{"done":false,"count":5},"small":{"type":"vstack","children":[
        {"type":"text","content":"{{state.count}}|{{state.label}}|{{instance}}"},
        {"type":"toggle","bind":{"isOn":"state.done"}},
        {"type":"text","content":"done!","when":{"path":"state.done"}}
    ]}}"""

    /** What the placement shows: the widget pipeline (placement → stored state → resolve). */
    private fun shown(appWidgetId: Int): String {
        val p = Placement.read(c, appWidgetId)
        val cfg = JSONObject(p.configRaw!!)
        val st = WidgetStoreKeys.readState(prefs(p.group), p.widgetId, p.instance, cfg.opt("state"))
        val kids = Resolve.resolve(cfg, Resolve.Context(0, "en", p.instance, state = st))
            .getJSONObject("small").getJSONArray("children")
        val toggle = kids.getJSONObject(1)
        return kids.getJSONObject(0).getString("content") + "|" + toggle.opt("isOn") + "|" + kids.length()
    }

    private suspend fun tap(appWidgetId: Int, set: String, action: String = "") =
        WidgetActionCallback().onAction(
            c, AppWidgetId(appWidgetId),
            actionParametersOf(ACTION_KEY to action, PAYLOAD_KEY to "", URL_KEY to "", SET_KEY to set),
        )

    private fun queued(group: String) = JSONArray(prefs(group).getString(WidgetStoreKeys.PENDING_ACTIONS, "[]")).length()

    /** Given a placed widget with `config.state`, when `set` taps, widget writes, an instance pick and clears happen, then each render shows the merged state layers. */
    @Test
    fun placedWidgetTapsThroughStateLayers() = runTest {
        val g = "g.tap"
        val p = prefs(g)
        // Placed: the placement id is the appWidgetId until a configured instance is picked.
        WidgetStoreKeys.bindInstance(c, 42, "todo", g)
        WidgetStoreKeys.bindInstance(c, 43, "todo", g)
        p.edit().putString("config:todo", layout).commit()
        assertEquals("42", WidgetStoreKeys.placementInstance(c, 42))
        assertNull(WidgetStoreKeys.placementInstance(c, -1))
        assertEquals("0", WidgetStoreKeys.placementInstance(c, 0))
        WidgetStoreKeys.setPlacementInstance(c, -1, "x")
        WidgetStoreKeys.setPlacementInstance(c, 42, "")
        assertEquals("42", WidgetStoreKeys.placementInstance(c, 42))
        assertNull(WidgetStoreKeys.metaPrefs(c).getString(WidgetStoreKeys.INSTANCE_CHOICE_PREFIX + "-1", null))
        WidgetStoreKeys.setPlacementInstance(c, 0, "zero")
        assertEquals("zero", WidgetStoreKeys.placementInstance(c, 0))

        // Nothing stored: `config.state` defaults (`when` false drops the third child).
        assertEquals("""{"done":false,"count":5}""", WidgetStoreKeys.readState(p, "todo", "42", JSONObject("""{"done":false,"count":5}""")).toString())
        assertNull(WidgetStoreKeys.readState(p, "todo", "42", null))
        assertEquals("5||42|false|2", shown(42))

        // A tap with `set` + action: placement state written, action queued, render follows.
        tap(42, """{"done":"${'$'}toggle","count":"${'$'}dec"}""", action = "tapped")
        assertEquals("""{"done":true,"count":4}""", p.getString("state:todo#42", null))
        assertEquals(1, queued(g))
        assertEquals("4||42|true|3", shown(42))
        // `set` only: state flips back, nothing queued.
        tap(42, """{"done":"${'$'}toggle"}""")
        assertEquals("4||42|false|2", shown(42))
        assertEquals(1, queued(g))
        // Not an object: ignored.
        tap(42, "[1]")
        assertEquals("""{"done":false,"count":4}""", p.getString("state:todo#42", null))

        // Widget layer: the placement's own keys still win; the sibling sees the widget layer.
        WidgetStoreKeys.writeState(p, "todo", null, JSONObject("""{"count":100,"label":"W"}"""))
        assertEquals("4|W|42|false|2", shown(42))
        assertEquals("100|W|43|false|2", shown(43))
        tap(43, """{"count":"${'$'}inc","done":"${'$'}toggle"}""")
        assertEquals("101|W|43|true|3", shown(43))

        // Configured instance: a new scope over the same widget layer.
        WidgetStoreKeys.setPlacementInstance(c, 42, "home")
        assertEquals("100|W|home|false|2", shown(42))
        tap(42, """{"count":"${'$'}inc"}""")
        tap(42, """{"label":"H"}""")
        assertEquals("""{"count":101,"label":"H"}""", p.getString("state:todo#home", null))
        // Replace stores exactly the object; a plain write merges.
        WidgetStoreKeys.writeState(p, "todo", "home", JSONObject("""{"z":1}"""), replace = true)
        assertEquals("""{"z":1}""", p.getString("state:todo#home", null))
        assertEquals("""{"z":1,"y":2}""", WidgetStoreKeys.writeState(p, "todo", "home", JSONObject("""{"y":2}""")).toString())
        assertEquals("100|W|home|false|2", shown(42))

        // Clear placement, then widget: back to defaults.
        WidgetStoreKeys.clearState(p, "todo", "home")
        assertNull(p.getString("state:todo#home", null))
        WidgetStoreKeys.clearState(p, "todo", null)
        assertEquals("5||home|false|2", shown(42))
        // Placement removed: the choice goes, the appWidgetId scope (still stored) comes back.
        WidgetStoreKeys.clearInstance(c, 42)
        WidgetStoreKeys.bindInstance(c, 42, "todo", g)
        assertEquals("4||42|false|2", shown(42))
    }

    /** Given current state of every type, when `set` operators apply, then numbers keep their type and non-numbers count from 0. */
    @Test
    fun setOperatorsKeepNumberTypes() {
        val cur = JSONObject("""{"on":true,"n":2,"f":1.5,"s":"x","s2":"str"}""").put("l", 5_000_000_000L)
        val out = WidgetStoreKeys.applySet(
            cur,
            JSONObject("""{"on":"${'$'}toggle","off":"${'$'}toggle","s2":"${'$'}toggle","n":"${'$'}inc","m":"${'$'}dec","f":"${'$'}inc","l":"${'$'}dec","s":"${'$'}inc","v":"y","o":{"k":1},"z":null}"""),
        )
        assertEquals(false, out.get("on"))
        assertEquals(true, out.get("off"))
        assertEquals(true, out.get("s2"))
        assertEquals(3L, out.get("n"))
        assertEquals(-1L, out.get("m"))
        assertEquals(2.5, out.get("f"))
        assertEquals(4_999_999_999L, out.get("l"))
        // Not a number: counts from 0.
        assertEquals(1L, out.get("s"))
        assertEquals("y", out.get("v"))
        assertEquals(1, out.getJSONObject("o").getInt("k"))
        assertEquals(JSONObject.NULL, out.get("z"))
        // No current state, or a non-object one.
        for (none in listOf(null, 5, "x")) {
            val r = WidgetStoreKeys.applySet(none, JSONObject("""{"n":"${'$'}inc","b":"${'$'}toggle"}"""))
            assertEquals(1L, r.get("n"))
            assertEquals(true, r.get("b"))
        }
    }

    // ---- model-based chain: the prefs-backed store against a plain map model ----

    private sealed interface Cmd {
        val scope: String?
    }
    private data class SetPatch(override val scope: String?, val key: String, val value: Any?) : Cmd
    private data class Clear(override val scope: String?) : Cmd
    private data class Replace(override val scope: String?, val key: String, val value: Any?) : Cmd
    private data class Read(override val scope: String?) : Cmd

    private object JNull

    private val scopes = listOf(null, "1", "2")
    private val keys = listOf("a", "b", "c")
    private val values: List<Any?> = listOf(
        "${'$'}toggle", "${'$'}inc", "${'$'}dec", null, true, false, 0, 7, -3, 1.5, "s",
    )

    private val cmdArb: Arb<Cmd> = Arb.choice(
        Arb.bind(Arb.element(scopes), Arb.element(keys), Arb.element(values)) { s, k, v -> SetPatch(s, k, v) },
        Arb.bind(Arb.element(scopes), Arb.element(keys), Arb.element(values.drop(3))) { s, k, v -> Replace(s, k, v) },
        Arb.element(scopes).map { Clear(it) },
        Arb.element(scopes).map { Read(it) },
    )

    private class Model(val defaults: Map<String, Any>?) {
        val layers = mutableMapOf<String?, MutableMap<String, Any>>()

        fun merged(scope: String?): Map<String, Any>? {
            val parts = listOfNotNull(defaults, layers[null], scope?.let { layers[it] })
            if (parts.isEmpty()) return null
            return parts.fold(mutableMapOf()) { acc, m -> acc.apply { putAll(m) } }
        }

        fun op(old: Any?, v: Any?): Any = when (v) {
            "${'$'}toggle" -> if (old is Boolean) !old else true
            "${'$'}inc", "${'$'}dec" -> {
                val d = if (v == "${'$'}inc") 1 else -1
                if (old is Double && old != Math.rint(old)) old + d else ((old as? Number)?.toLong() ?: 0L) + d
            }
            null -> JNull
            else -> v
        }
    }

    /** JSON → comparable Kotlin (numbers by value, JSON null kept as [JNull]). */
    private fun canon(v: Any?): Any? = when (v) {
        null -> null
        JSONObject.NULL, JNull -> JNull
        is JSONObject -> v.keys().asSequence().associateWith { canon(v.opt(it)) }
        is Map<*, *> -> v.entries.associate { (k, x) -> k to canon(x) }
        is Number -> v.toDouble()
        else -> v
    }

    private fun json(v: Any?): Any = if (v == null || v == JNull) JSONObject.NULL else v

    /** Given generated patch / replace / clear sequences, when run on the prefs store and a map model, then every scope reads the same after every step. */
    @OptIn(io.kotest.common.ExperimentalKotest::class)
    @Test
    fun stateStoreAgreesWithAMapModel(): Unit = runBlocking {
        val defaultsArb = Arb.element(listOf(null, mapOf("a" to false, "b" to 2), mapOf("c" to 1.5)))
        var run = 0
        checkAll(PropTestConfig(seed = 20261113L, iterations = 150), defaultsArb, Arb.list(cmdArb, 1..25)) { defaults, cmds ->
            val p: SharedPreferences = prefs("g.model.${run++}")
            val model = Model(defaults)
            val defaultsJson = defaults?.let { JSONObject(it) }
            for ((i, cmd) in cmds.withIndex()) {
                val sc = WidgetStoreKeys.scopedId("w", cmd.scope)
                when (cmd) {
                    // `setWidgetState(patch)` / a tap's `set`: applied to the merged state, stored in the scope.
                    is SetPatch -> {
                        val cur = WidgetStoreKeys.readState(p, "w", cmd.scope, defaultsJson)
                        val patch = JSONObject().put(cmd.key, json(cmd.value))
                        WidgetStoreKeys.writeState(p, "w", cmd.scope, WidgetStoreKeys.applySet(cur, patch))
                        val old = model.merged(cmd.scope)?.get(cmd.key)
                        model.layers.getOrPut(cmd.scope) { mutableMapOf() }[cmd.key] = model.op(old, cmd.value)
                    }
                    is Replace -> {
                        WidgetStoreKeys.writeState(p, "w", cmd.scope, JSONObject().put(cmd.key, json(cmd.value)), replace = true)
                        model.layers[cmd.scope] = mutableMapOf(cmd.key to (cmd.value ?: JNull))
                    }
                    is Clear -> {
                        WidgetStoreKeys.clearState(p, "w", cmd.scope)
                        model.layers.remove(cmd.scope)
                    }
                    is Read -> Unit
                }
                for (s in scopes) {
                    assertEquals("step $i $cmd scope $s ($sc) in $cmds", canon(model.merged(s)), canon(WidgetStoreKeys.readState(p, "w", s, defaultsJson)))
                }
            }
            assertTrue(cmds.isNotEmpty())
        }
        Unit
    }
}
