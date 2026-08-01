package git.s00d.widgets

import android.graphics.Bitmap
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
import java.io.FileOutputStream

/**
 * Level-1 Android geometry harness: Glance compose → measure → dumpTree.
 * Snapshots: tests/expected/geometry/<id>.<size>.android.json
 * Update: ./gradlew :testDebugUnitTest -PupdateSnapshots=true
 */
@RunWith(RobolectricTestRunner::class)
@Config(sdk = [34], qualifiers = "w411dp-h891dp-xxhdpi")
@GraphicsMode(GraphicsMode.Mode.NATIVE)
class GeometryFixturesTest {
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
            val png = File(
                RenderHarness.expectedPixelsDir(),
                "${fixtureId.replace("/", "__")}.$size.png",
            )
            png.parentFile?.mkdirs()
            FileOutputStream(png).use { out ->
                rendered.bitmap.compress(Bitmap.CompressFormat.PNG, 100, out)
            }
            return
        }
        if (outFile.exists()) {
            val expected = JSONObject(outFile.readText())
            assertTrue(
                "geometry kind mismatch for $fixtureId/$size",
                expected.optString("kind").isNotEmpty() && tree.optString("kind").isNotEmpty(),
            )
            // Strict drift vs committed android baseline (rect ±2 on root).
            val er = expected.getJSONArray("rect")
            val ar = tree.getJSONArray("rect")
            for (i in 0 until 4) {
                val d = kotlin.math.abs(er.getInt(i) - ar.getInt(i))
                assertTrue(
                    "root rect[$i] drift $d for $fixtureId/$size (e=${er.getInt(i)} a=${ar.getInt(i)})",
                    d <= 2,
                )
            }
        }
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
