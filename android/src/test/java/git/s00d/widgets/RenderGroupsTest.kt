package git.s00d.widgets

import org.json.JSONObject
import org.junit.Assert.assertTrue
import org.junit.Assert.fail
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

/**
 * Guard for the six-group [RenderElement] dispatcher.
 * Renders one fixture covering layout/text/media/data/interactive/spacing and
 * asserts [RenderTrace] saw each group — catches a type dropped from the when().
 */
@RunWith(RobolectricTestRunner::class)
@Config(sdk = [34], qualifiers = "w411dp-h891dp-xxhdpi")
@GraphicsMode(GraphicsMode.Mode.NATIVE)
class RenderGroupsTest {
    @Test
    fun allSixDocsGroupsReachRenderTrace() {
        val cfg = JSONObject(
            """
            {
              "small": {
                "type": "vstack",
                "spacing": 4,
                "children": [
                  { "type": "text", "content": "hi", "fontSize": 12 },
                  { "type": "shape", "shapeType": "circle", "size": 12, "fill": "#00ff00" },
                  { "type": "progress", "value": 0.4, "total": 1, "barStyle": "linear" },
                  { "type": "button", "label": "Go", "action": "tap" },
                  { "type": "spacer", "minLength": 6 },
                  { "type": "divider" }
                ]
              }
            }
            """.trimIndent(),
        ).toString()

        RenderTrace.begin()
        val rendered = RenderHarness.render(cfg, "small")
        assertTrue("empty bitmap", rendered.bitmap.width > 0 && rendered.bitmap.height > 0)

        val types = RenderTrace.renderedTypes().toSet()
        val required = listOf(
            "vstack", // layout
            "text", // text
            "shape", // media
            "progress", // data
            "button", // interactive
            "spacer", // spacing
            "divider", // spacing
        )
        val missing = required.filter { it !in types }
        if (missing.isNotEmpty()) {
            fail("RenderTrace missing $missing — saw $types (dispatcher drop?)")
        }
    }
}
