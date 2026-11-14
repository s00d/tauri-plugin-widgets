package git.s00d.widgets

import org.json.JSONArray
import org.json.JSONObject
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode
import androidx.compose.ui.graphics.toArgb
import java.io.File

/** Glance renderer: layout math helpers, then rendered trees for the v2 IR fields. */
@RunWith(RobolectricTestRunner::class)
@Config(sdk = [34], qualifiers = "w411dp-h891dp-xxhdpi")
@GraphicsMode(GraphicsMode.Mode.NATIVE)
class V2RenderTest {
    private fun kids(n: Int) = List(n) { JSONObject().put("type", "text") }

    /** Given host sizes, children and timed / labelled values, when the layout helpers run, then family, justify plan, list scrolling, progress fraction and spoken descriptions follow the rules. */
    @Test
    fun layoutMathAndDescriptions() {
        // Family follows the composed size; lock-screen hosts get the small layout.
        assertEquals("small", TauriGlanceWidget.familyFor(170f, 170f))
        assertEquals("medium", TauriGlanceWidget.familyFor(360f, 170f))
        assertEquals("medium", TauriGlanceWidget.familyFor(180f, 110f))
        assertEquals("large", TauriGlanceWidget.familyFor(360f, 380f))
        assertEquals("small", TauriGlanceWidget.familyFor(400f, 100f))
        assertEquals("small", TauriGlanceWidget.familyFor(352f, 339f, android.appwidget.AppWidgetProviderInfo.WIDGET_CATEGORY_KEYGUARD))
        assertEquals("large", TauriGlanceWidget.familyFor(352f, 339f, android.appwidget.AppWidgetProviderInfo.WIDGET_CATEGORY_HOME_SCREEN))

        assertEquals(JustifyPlan(0, 1, 0, true), justifyPlan("spaceBetween", kids(3), 4))
        assertEquals(JustifyPlan(1, 2, 1, false), justifyPlan("spaceAround", kids(3), 0))
        assertEquals(JustifyPlan(1, 0, 1, true), justifyPlan("center", kids(2), 6))
        // Over the 10-child budget: fixed spacing dropped, then spaceAround → spaceEvenly.
        assertEquals(JustifyPlan(1, 1, 1, false), justifyPlan("spaceAround", kids(4), 4))
        assertEquals(JustifyPlan(0, 0, 0, true), justifyPlan("spaceEvenly", kids(6), 4))
        // A flex spacer already takes the free space.
        assertEquals(JustifyPlan(0, 0, 0, false), justifyPlan("center", kids(2) + JSONObject().put("type", "spacer"), 0))

        // list-20-items (large, 360×380 dp): 20 rows of 12 sp fit, 30 overflow and scroll.
        assertFalse(listScrolls(20, 12.0, 2, verticalPaddingDp(8), 380f))
        assertTrue(listScrolls(30, 12.0, 2, verticalPaddingDp(8), 380f))
        assertTrue(listScrolls(8, 14.0, 4, 0.0, 110f))
        assertFalse(listScrolls(3, 14.0, 2, 0.0, 110f))
        assertFalse(listScrolls(0, 14.0, 2, 0.0, 110f))
        assertFalse("unknown size never scrolls", listScrolls(99, 14.0, 2, 0.0, 0f))
        assertEquals(16.0, verticalPaddingDp(8), 0.0)
        assertEquals(7.0, verticalPaddingDp(JSONObject("{\"top\":3,\"bottom\":4,\"leading\":9}")), 0.0)
        assertEquals(0.0, verticalPaddingDp(null), 0.0)

        val timed = El(JSONObject().put("startAt", 1000).put("endAt", 2000))
        assertEquals(0.5, timeDrivenFraction(timed, 1500)!!, 1e-9)
        assertEquals(1.0, timeDrivenFraction(timed, 5000)!!, 1e-9)
        assertEquals(0.0, timeDrivenFraction(timed, 0)!!, 1e-9)
        assertNull(timeDrivenFraction(El(JSONObject().put("value", 1)), 0))

        fun el(json: String) = El(JSONObject(json))
        assertEquals("Steps 45%", valueDescription(el("""{"type":"progress","value":0.45,"label":"Steps"}"""), "progress", 0.45, 45))
        assertEquals("72 bpm", valueDescription(el("""{"type":"gauge","value":72,"currentValueLabel":"72 bpm"}"""), "gauge", 72.0, 50))
        assertEquals("Done", valueDescription(el("""{"type":"progress","value":1,"accessibilityLabel":"Done"}"""), "progress", 1.0, 100))
        val chart = el("""{"type":"chart","chartType":"line","chartData":[{"label":"Mon","value":3},{"label":"Tue","value":5.5}]}""")
        assertEquals("line chart: Mon 3, Tue 5.5", chartDescription(chart))
        assertEquals("line chart: Mon 3, …", chartDescription(chart, maxPoints = 1))
    }

    private fun texts(node: JSONObject, out: MutableList<String> = mutableListOf()): List<String> {
        node.optString("text").takeIf { it.isNotEmpty() }?.let { out += it }
        val c = node.optJSONArray("children") ?: return out
        for (i in 0 until c.length()) texts(c.getJSONObject(i), out)
        return out
    }

    private fun kinds(node: JSONObject, out: MutableList<String> = mutableListOf()): List<String> {
        out += node.optString("kind")
        val c = node.optJSONArray("children") ?: return out
        for (i in 0 until c.length()) kinds(c.getJSONObject(i), out)
        return out
    }

    private fun fixture(name: String) = File(RenderHarness.fixturesRoot(), "core/$name.json").readText()

    private fun small(el: JSONObject) = JSONObject().put("small", el).toString()

    /** Given fixtures for every element group and v2 field, when they render, then each group reaches the dispatcher, nothing is skipped, deep trees are cut and long lists scroll. */
    @Test
    fun renderedTreeHonoursV2Fields() {
        // All six docs groups (layout/text/media/data/interactive/spacing) reach RenderTrace.
        RenderTrace.begin()
        val groups = RenderHarness.render(
            small(
                JSONObject("""{"type":"vstack","spacing":4,"children":[
                  {"type":"text","content":"hi","fontSize":12},
                  {"type":"shape","shapeType":"circle","size":12,"fill":"#00ff00"},
                  {"type":"progress","value":0.4,"total":1,"barStyle":"linear"},
                  {"type":"button","label":"Go","action":"tap"},
                  {"type":"spacer","minLength":6},
                  {"type":"divider"}]}"""),
            ),
            "small",
        )
        assertTrue("empty bitmap", groups.bitmap.width > 0 && groups.bitmap.height > 0)
        val seen = RenderTrace.renderedTypes().toSet()
        val missing = listOf("vstack", "text", "shape", "progress", "button", "spacer", "divider").filter { it !in seen }
        assertTrue("RenderTrace missing $missing — saw $seen (dispatcher drop?)", missing.isEmpty())

        // v2 fixtures: nothing skipped; timer completion text, text styles, controls.
        RenderTrace.begin()
        val progress = RenderHarness.render(fixture("v2-progress"), "small").tree
        assertTrue(texts(progress).toString(), "Done!" in texts(progress))
        for (f in listOf("v2-text", "v2-layout", "v2-controls", "v2-chart", "v2-shapes")) {
            RenderTrace.begin()
            val t = RenderHarness.render(fixture(f), "medium").tree
            assertTrue("$f: ${RenderTrace.skippedJson()}", RenderTrace.skippedJson().length() == 0)
            if (f == "v2-text") assertTrue(texts(t).containsAll(listOf("Under", "Strike", "Italic", "S p a c e")))
            if (f == "v2-controls") {
                assertTrue(kinds(t).toString(), kinds(t).any { it.contains("CheckBox") })
                assertTrue(texts(t).containsAll(listOf("Bordered", "Go", "Plain", "Off")))
            }
        }

        // Depth guard: too-deep subtrees dropped and reported, shallow siblings kept.
        var node = JSONObject().put("type", "text").put("content", "deep")
        repeat(40) { i ->
            val k = JSONArray().put(node)
            if (i == 39) k.put(JSONObject().put("type", "text").put("content", "top"))
            node = JSONObject().put("type", if (i % 2 == 0) "vstack" else "hstack").put("children", k)
        }
        RenderTrace.begin()
        val deep = texts(RenderHarness.render(small(node), "small").tree)
        assertTrue(deep.toString(), "top" in deep && "deep" !in deep)
        assertTrue(RenderTrace.skippedJson().toString(), RenderTrace.skippedJson().toString().contains("\"depth\""))

        // Switch variant is a compound button.
        val sw = kinds(RenderHarness.render(small(JSONObject().put("type", "toggle").put("variant", "switch").put("isOn", true).put("label", "S")), "small").tree)
        assertTrue(sw.toString(), sw.any { it.contains("Switch") })

        // Head / middle truncation.
        val long = "abcdefghijklmnopqrstuvwxyz0123456789"
        fun trunc(mode: String) = JSONObject().put("type", "text").put("content", long).put("lineLimit", 1)
            .put("truncationMode", mode).put("frame", JSONObject().put("width", 80))
        val cut = texts(RenderHarness.render(small(JSONObject().put("type", "vstack").put("children", JSONArray().put(trunc("head")).put(trunc("middle")))), "small").tree)
        assertTrue(cut.toString(), cut.any { it.startsWith("…") && it.endsWith("9") })
        assertTrue(cut.toString(), cut.any { it.startsWith("a") && it.endsWith("9") && "…" in it })

        // A list scrolls (ListView) only when its rows overflow the large family.
        fun list(rows: Int) = JSONObject().put("version", 1).put(
            "large",
            JSONObject().put("type", "list").put("fontSize", 12).put("spacing", 2).put("padding", 8)
                .put("items", JSONArray((1..rows).map { JSONObject().put("text", "Row $it") })),
        ).toString()
        assertTrue(kinds(RenderHarness.render(list(40), "large").tree).any { it.contains("ListView") })
        assertFalse(kinds(RenderHarness.render(list(20), "large").tree).any { it.contains("ListView") })
    }

    /** Given shape, image and button IR the Apple renderers draw, when the Android bitmaps are baked, then "transparent" stays clear, strokes are dp, image clips hold without a view outline, a bar fills its width and transport glyphs map to icons. */
    @Test
    fun bakedShapesMatchApple() {
        val ctx = androidx.test.core.app.ApplicationProvider.getApplicationContext<android.content.Context>()
        val d = ctx.resources.displayMetrics.density
        assertEquals(0, parseColor(ctx, "transparent")!!.toArgb() ushr 24)

        // flex-stroke-grid B: transparent circle with a 3 pt stroke → clear centre, stroke ≥ 3 dp.
        val ring = drawShapeBitmap(ctx, JSONObject("""{"shapeType":"circle","fill":"transparent","stroke":"#38bdf8","strokeWidth":3}"""), 36)!!
        assertEquals(0, ring.getPixel(ring.width / 2, ring.height / 2) ushr 24)
        assertTrue("stroke drawn in dp", ring.getPixel(ring.width / 2, (2.5f * d).toInt()) ushr 24 > 0)

        // image-sources Data: circle clip baked → corner clear, centre opaque.
        val src = android.graphics.Bitmap.createBitmap(4, 4, android.graphics.Bitmap.Config.ARGB_8888).apply { eraseColor(android.graphics.Color.RED) }
        val clipped = clipImageBitmap(ctx, JSONObject("""{"size":32,"contentMode":"fill","clipShape":"circle"}"""), src)
        assertEquals(0, clipped.getPixel(0, 0) ushr 24)
        assertEquals(android.graphics.Color.RED, clipped.getPixel(clipped.width / 2, clipped.height / 2))
        assertTrue("no clip → source as is", clipImageBitmap(ctx, JSONObject("""{"size":32}"""), src) === src)

        // Linear bar: track to the right edge, fill to the fraction.
        val bar = drawBarBitmap(android.graphics.Color.GREEN, android.graphics.Color.GRAY, 50, 200, 12)
        assertEquals(android.graphics.Color.GREEN, bar.getPixel(60, 6))
        assertEquals(android.graphics.Color.GRAY, bar.getPixel(150, 6))

        assertEquals(android.R.drawable.ic_media_previous, mediaIconRes("\u23ee"))
        assertEquals(android.R.drawable.ic_media_pause, mediaIconRes("\u23f8"))
        assertEquals(android.R.drawable.ic_media_next, mediaIconRes("\u23ed"))
        assertNull(mediaIconRes("Go"))
    }
}
