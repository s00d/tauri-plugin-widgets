package git.s00d.widgets

import org.json.JSONObject
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/** Unit coverage for El typed JSON accessors. */
class ElTest {
    @Test
    fun str_returnsDefaultWhenMissing() {
        val el = El(JSONObject("""{"type":"text"}"""))
        assertEquals("", el.str("content"))
        assertEquals("x", el.str("content", "x"))
    }

    @Test
    fun str_readsString() {
        val el = El(JSONObject("""{"content":"hello"}"""))
        assertEquals("hello", el.str("content"))
    }

    @Test
    fun num_nullWhenMissing() {
        val el = El(JSONObject("{}"))
        assertNull(el.num("x"))
    }

    @Test
    fun num_readsDouble() {
        val el = El(JSONObject("""{"x":3.5}"""))
        assertEquals(3.5, el.num("x")!!, 0.001)
    }

    @Test
    fun num_defaultWhenMissing() {
        val el = El(JSONObject("{}"))
        assertEquals(2.0, el.num("x", 2.0), 0.001)
    }

    @Test
    fun bool_defaultsFalse() {
        val el = El(JSONObject("{}"))
        assertFalse(el.bool("checked"))
        assertTrue(el.bool("checked", true))
    }

    @Test
    fun bool_readsValue() {
        val el = El(JSONObject("""{"checked":true}"""))
        assertTrue(el.bool("checked"))
    }

    @Test
    fun bool_nullJsonIsDefault() {
        val el = El(JSONObject("""{"checked":null}"""))
        assertFalse(el.bool("checked"))
        assertTrue(el.bool("checked", true))
    }
}
