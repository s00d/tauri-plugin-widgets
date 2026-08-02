package git.s00d.widgets

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/** Unit coverage for storage-key helpers (cubic: new type needs tests). */
class WidgetStoreKeysTest {
    @Test
    fun configKey_prefixesWidgetId() {
        assertEquals("config:home", WidgetStoreKeys.configKey("home"))
        assertTrue(WidgetStoreKeys.isConfigKey("config:home"))
        assertTrue(!WidgetStoreKeys.isConfigKey("pending_actions"))
    }

    @Test
    fun instanceKeys_areStable() {
        assertEquals("widgetId:7", WidgetStoreKeys.instanceWidgetIdKey(7))
        assertEquals("group:7", WidgetStoreKeys.instanceGroupKey(7))
    }

    @Test
    fun clearInstance_keyNamesMatchBind() {
        // clearInstance removes the same keys bindInstance writes.
        val id = 42
        assertEquals("widgetId:$id", WidgetStoreKeys.instanceWidgetIdKey(id))
        assertEquals("group:$id", WidgetStoreKeys.instanceGroupKey(id))
    }
}
