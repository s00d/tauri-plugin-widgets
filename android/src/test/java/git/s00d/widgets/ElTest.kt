package git.s00d.widgets

import org.json.JSONObject
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

/**
 * El typed JSON accessors. Robolectric: plain JVM Android stubs (`isReturnDefaultValues`)
 * make org.json return empty defaults, so "reads value" assertions would pass falsely.
 */
@RunWith(RobolectricTestRunner::class)
@Config(sdk = [34])
class ElTest {
    /** Given an element with present, missing and JSON-null fields, when each typed accessor reads them, then present values come back and the rest fall back to defaults. */
    @Test
    fun accessorsReadValuesAndFallBack() {
        val el = El(JSONObject("""{"type":"text","content":"hello","x":3.5,"checked":true,"off":null}"""))
        // Present values.
        assertEquals("hello", el.str("content"))
        assertEquals(3.5, el.num("x")!!, 0.001)
        assertTrue(el.bool("checked"))
        // Missing: "" / null / given defaults.
        assertEquals("", el.str("missing"))
        assertEquals("x", el.str("missing", "x"))
        assertNull(el.num("missing"))
        assertEquals(2.0, el.num("missing", 2.0), 0.001)
        assertFalse(el.bool("missing"))
        assertTrue(el.bool("missing", true))
        // JSON null counts as missing.
        assertFalse(el.bool("off"))
        assertTrue(el.bool("off", true))
    }
}
