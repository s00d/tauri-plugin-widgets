package git.s00d.widgets

import org.json.JSONObject
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Assert.fail
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode
import java.io.File

/**
 * Level-1 Android geometry harness: Glance compose → measure → dumpTree.
 * Snapshots: tests/expected/geometry/<id>.<size>.android.json
 * Update: ./gradlew :testDebugUnitTest -PupdateSnapshots=true
 */
@RunWith(RobolectricTestRunner::class)
@Config(sdk = [34], qualifiers = "w411dp-h891dp-xxhdpi")
@GraphicsMode(GraphicsMode.Mode.NATIVE)
class GeometryFixturesTest {
    /** Given every fixture and committed Android geometry, when each composes, then the whole tree (kind, rect ±2 px, text, children) matches. */
    @Test
    fun allFixturesGeometry() {
        val cases = collectCases()
        assertTrue(
            "no fixtures under ${RenderHarness.fixturesRoot()} " +
                "(fixtures.root=${System.getProperty("fixtures.root")})",
            cases.isNotEmpty(),
        )

        val failures = mutableListOf<String>()
        for ((fixtureId, file, size) in cases) {
            try {
                runOne(fixtureId, file, size)
            } catch (t: Throwable) {
                failures += "$fixtureId[$size]: ${t.message}"
            }
        }
        if (failures.isNotEmpty()) {
            fail(failures.joinToString("\n"))
        }
    }

    private fun runOne(fixtureId: String, fixtureFile: File, size: String) {
        val json = fixtureFile.readText()
        val rendered = RenderHarness.render(json, size)
        val tree = rendered.tree

        assertFalse("literal null text in $fixtureId", containsLiteralNull(tree))
        assertTrue("empty bitmap $fixtureId", rendered.bitmap.width > 0 && rendered.bitmap.height > 0)

        val outFile = File(
            RenderHarness.expectedGeometryDir(),
            "${fixtureId.replace("/", "__")}.$size.android.json",
        )
        if (RenderHarness.updateSnapshots()) {
            outFile.parentFile?.mkdirs()
            outFile.writeText(tree.toString(2) + "\n")
            // Pixel goldens are Level-2 AppWidgetHost (tests/golden/android), not Robolectric.
            return
        }
        if (outFile.exists()) {
            val expected = JSONObject(outFile.readText())
            assertTrue(
                "geometry kind mismatch for $fixtureId/$size",
                expected.optString("kind").isNotEmpty() && tree.optString("kind").isNotEmpty(),
            )
            val diffs = mutableListOf<String>()
            // Clock-driven fixtures (dates, timers): structure only, sizes follow today's text.
            diffTree(expected, tree, "root", diffs, structureOnly = fixtureId in CLOCK_DRIVEN)
            assertTrue(
                "geometry drift for $fixtureId/$size (update with -PupdateSnapshots=true):\n" +
                    diffs.take(5).joinToString("\n"),
                diffs.isEmpty(),
            )
        }
    }

    /** Whole-tree drift: kind, rect (±2 px), text, child count. Clock-driven text skipped. */
    private fun diffTree(
        e: JSONObject,
        a: JSONObject,
        path: String,
        out: MutableList<String>,
        structureOnly: Boolean = false,
    ) {
        val kind = e.optString("kind")
        if (kind != a.optString("kind")) {
            out += "$path: kind ${kind} != ${a.optString("kind")}"
            return
        }
        val er = e.optJSONArray("rect")
        val ar = a.optJSONArray("rect")
        if (!structureOnly && er != null && ar != null) {
            for (i in 0 until 4) {
                if (kotlin.math.abs(er.getInt(i) - ar.getInt(i)) > 2) {
                    out += "$path($kind): rect $er != $ar"
                    break
                }
            }
        }
        if (!structureOnly && kind != "Chronometer" && e.optString("text") != a.optString("text")) {
            out += "$path($kind): text '${e.optString("text")}' != '${a.optString("text")}'"
        }
        val ec = e.optJSONArray("children")
        val ac = a.optJSONArray("children")
        val en = ec?.length() ?: 0
        val an = ac?.length() ?: 0
        if (en != an) {
            out += "$path($kind): $en children != $an"
            return
        }
        for (i in 0 until en) diffTree(ec!!.getJSONObject(i), ac!!.getJSONObject(i), "$path/$i", out, structureOnly)
    }

    private fun containsLiteralNull(node: JSONObject): Boolean {
        val t = node.optString("text", "")
        if (t == "null" || t == "undefined") return true
        val children = node.optJSONArray("children") ?: return false
        for (i in 0 until children.length()) {
            if (containsLiteralNull(children.getJSONObject(i))) return true
        }
        return false
    }

    companion object {
        val CLOCK_DRIVEN = setOf("core/date-timer")

        fun collectCases(): List<Triple<String, File, String>> {
            val root = RenderHarness.fixturesRoot()
            val out = mutableListOf<Triple<String, File, String>>()
            for (folder in listOf("core", "bugs", "presets")) {
                val dir = File(root, folder)
                if (!dir.isDirectory) continue
                dir.listFiles()?.filter { it.extension == "json" }?.sortedBy { it.name }?.forEach { file ->
                    val raw = JSONObject(file.readText())
                    for (size in listOf("small", "medium", "large")) {
                        if (!raw.has(size) || raw.isNull(size)) continue
                        out.add(Triple("$folder/${file.nameWithoutExtension}", file, size))
                    }
                }
            }
            return out
        }
    }
}
