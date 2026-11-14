package git.s00d.widgets

import android.app.AlarmManager
import android.app.PendingIntent
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.pm.ActivityInfo
import android.os.Bundle
import androidx.test.core.app.ApplicationProvider
import kotlinx.coroutines.test.runTest
import org.json.JSONArray
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.Shadows.shadowOf
import org.robolectric.annotation.Config
import org.robolectric.shadows.ShadowAlarmManager

/** Storage keys, placement binding, group resolution, the action queue, alarms and source backoff. */
@RunWith(RobolectricTestRunner::class)
@Config(sdk = [34])
class WidgetPersistenceTest {
    private lateinit var ctx: Context
    private val pkg get() = ctx.packageName

    @Before
    fun setUp() {
        ctx = ApplicationProvider.getApplicationContext()
    }

    private fun prefs(group: String) = ctx.getSharedPreferences(group, Context.MODE_PRIVATE)

    /** Given storage keys, groups and placements, when placements are bound and removed, then each reads the expected widget id and group. */
    @Test
    fun placementBindingAndGroupResolution() {
        // Storage keys shared with the Rust store.
        assertEquals("config:home", WidgetStoreKeys.configKey("home"))
        assertTrue(WidgetStoreKeys.isConfigKey("config:home"))
        assertFalse(WidgetStoreKeys.isConfigKey("pending_actions"))
        assertEquals("widgetId:7", WidgetStoreKeys.instanceWidgetIdKey(7))
        assertEquals("group:7", WidgetStoreKeys.instanceGroupKey(7))
        assertEquals("w", WidgetStoreKeys.scopedId("w", null))
        assertEquals("w", WidgetStoreKeys.scopedId("w", ""))
        assertEquals("w#7", WidgetStoreKeys.scopedId("w", "7"))
        assertEquals(listOf("config:w#7", "config:w"), WidgetStoreKeys.lookupKeys("config:", "w", "7"))
        assertEquals(listOf("data:w"), WidgetStoreKeys.lookupKeys("data:", "w", null))
        assertEquals(listOf("data:w"), WidgetStoreKeys.lookupKeys("data:", "w", ""))
        assertEquals("w" to "7", WidgetStoreKeys.splitScoped("w#7"))
        assertEquals("w" to null, WidgetStoreKeys.splitScoped("w"))
        assertEquals("w" to null, WidgetStoreKeys.splitScoped("w#"))
        assertEquals("config:w#7", WidgetSanitizer.sanitizeKey("config:w#7"))
        assertEquals("a_b", WidgetSanitizer.sanitizeKey(" a/b "))
        assertEquals("_", WidgetSanitizer.sanitizeKey("  "))
        assertEquals("fallback", WidgetSanitizer.sanitizeGroup("   ", "fallback"))
        assertEquals("group.a_b", WidgetSanitizer.sanitizeGroup(" group.a/b ", "f"))
        assertEquals("x", WidgetSanitizer.sanitizeGroup("x", "f"))

        // Nothing bound or stored: "default" widget in the package-name group.
        assertEquals("default", WidgetStoreKeys.resolveWidgetId(ctx, 42))
        assertEquals(pkg, WidgetStoreKeys.resolveGroup(ctx))
        // A candidate group with a config wins.
        prefs("group.$pkg").edit().putString("config:w", "{}").commit()
        assertEquals("group.$pkg", WidgetStoreKeys.resolveGroup(ctx))
        // The receiver's `tauri_widget_group` meta-data comes first among candidates (blank ignored).
        val receiver = ActivityInfo().apply {
            packageName = pkg
            name = "git.s00d.widgets.TauriGlanceWidgetReceiver"
            metaData = Bundle().apply { putString("tauri_widget_group", " ") }
        }
        shadowOf(ctx.packageManager).addOrUpdateReceiver(receiver)
        assertEquals("group.$pkg", WidgetStoreKeys.resolveGroup(ctx))
        receiver.metaData = Bundle().apply { putString("tauri_widget_group", "group.meta") }
        shadowOf(ctx.packageManager).addOrUpdateReceiver(receiver)
        assertEquals("group.$pkg", WidgetStoreKeys.resolveGroup(ctx))
        prefs("group.meta").edit().putString("config:w", "{}").commit()
        assertEquals("group.meta", WidgetStoreKeys.resolveGroup(ctx))
        shadowOf(ctx.packageManager).removeReceiver(ComponentName(pkg, receiver.name))
        // Nothing has a config: the first candidate (meta-data gone → package name).
        prefs("group.meta").edit().clear().commit()
        prefs("group.$pkg").edit().clear().commit()
        assertEquals(pkg, WidgetStoreKeys.resolveGroup(ctx))
        prefs("group.$pkg").edit().putString("config:w", "{}").commit()
        // The active group, only when it has a config.
        WidgetStoreKeys.metaPrefs(ctx).edit()
            .putString(WidgetStoreKeys.KEY_ACTIVE_GROUP, "group.act")
            .putString(WidgetStoreKeys.KEY_ACTIVE_WIDGET_ID, "active").commit()
        assertEquals("group.$pkg", WidgetStoreKeys.resolveGroup(ctx))
        prefs("group.act").edit().putString("config:w", "{}").commit()
        assertEquals("group.act", WidgetStoreKeys.resolveGroup(ctx))
        assertEquals("active", WidgetStoreKeys.resolveWidgetId(ctx, 42))
        assertEquals("active", WidgetStoreKeys.resolveWidgetId(ctx))

        // Placements: bound ones read their own widget id and group (only when it has a config).
        WidgetStoreKeys.bindInstance(ctx, 7, "weather", "group.a")
        assertEquals("weather", WidgetStoreKeys.mappedWidgetId(ctx, 7))
        assertEquals("group.a", WidgetStoreKeys.mappedGroup(ctx, 7))
        assertEquals("weather", WidgetStoreKeys.resolveWidgetId(ctx, 7))
        assertEquals("group.act", WidgetStoreKeys.resolveGroup(ctx, 7))
        prefs("group.a").edit().putString("config:weather", "{}").commit()
        assertEquals("group.a", WidgetStoreKeys.resolveGroup(ctx, 7))
        assertTrue(WidgetStoreKeys.hasAnyConfig(prefs("group.a")))
        prefs("only-data").edit().putString("data:w", "{}").putString("pending_actions", "[]").commit()
        assertFalse(WidgetStoreKeys.hasAnyConfig(prefs("only-data")))
        assertFalse(WidgetStoreKeys.hasAnyConfig(prefs("empty")))
        // Without a group: the widget id only.
        WidgetStoreKeys.bindInstance(ctx, 8, "clock", null)
        assertEquals("clock", WidgetStoreKeys.mappedWidgetId(ctx, 8))
        assertNull(WidgetStoreKeys.mappedGroup(ctx, 8))
        // Ignored: negative id, blank widget id.
        WidgetStoreKeys.bindInstance(ctx, -1, "x", "g")
        WidgetStoreKeys.bindInstance(ctx, 9, "  ", "g")
        assertNull(WidgetStoreKeys.mappedWidgetId(ctx, 9))
        assertNull(WidgetStoreKeys.mappedWidgetId(ctx, -1))
        assertNull(WidgetStoreKeys.mappedGroup(ctx, -1))
        // id 0 is a real appWidgetId.
        WidgetStoreKeys.bindInstance(ctx, 0, "zero", "group.z")
        assertEquals("zero", WidgetStoreKeys.mappedWidgetId(ctx, 0))
        assertEquals("group.z", WidgetStoreKeys.mappedGroup(ctx, 0))
        // Removed placements fall back to the active widget; others keep theirs.
        WidgetStoreKeys.clearInstance(ctx, 0)
        assertNull(WidgetStoreKeys.mappedWidgetId(ctx, 0))
        WidgetStoreKeys.clearInstance(ctx, 7)
        assertNull(WidgetStoreKeys.mappedWidgetId(ctx, 7))
        assertNull(WidgetStoreKeys.mappedGroup(ctx, 7))
        assertEquals("active", WidgetStoreKeys.resolveWidgetId(ctx, 7))
        WidgetStoreKeys.clearInstance(ctx, -1)
        assertEquals("clock", WidgetStoreKeys.mappedWidgetId(ctx, 8))
    }

    /** Given an action queue, when taps are enqueued (and the queue is corrupted), then entries append in order and a corrupt queue restarts. */
    @Test
    fun actionQueueAppendsAndRecovers() {
        val g = "group.q"
        WidgetStoreKeys.enqueueAction(ctx, g, "tap", null, "w")
        WidgetStoreKeys.enqueueAction(ctx, g, "tap2", "  ", "w")
        WidgetStoreKeys.enqueueAction(ctx, g, "tap3", "p", "w#7")
        val arr = JSONArray(prefs(g).getString(WidgetStoreKeys.PENDING_ACTIONS, "[]"))
        assertEquals(3, arr.length())
        assertEquals(listOf("tap", "tap2", "tap3"), (0 until 3).map { arr.getJSONObject(it).getString("action") })
        assertFalse(arr.getJSONObject(0).has("payload"))
        assertFalse(arr.getJSONObject(1).has("payload"))
        assertEquals("p", arr.getJSONObject(2).getString("payload"))
        assertEquals("w#7", arr.getJSONObject(2).getString("widgetId"))
        assertEquals(g, arr.getJSONObject(2).getString("group"))
        assertTrue(arr.getJSONObject(2).getLong("ts") > 0)
        // A corrupt queue restarts instead of failing the tap.
        prefs(g).edit().putString(WidgetStoreKeys.PENDING_ACTIONS, "{broken").commit()
        WidgetStoreKeys.enqueueAction(ctx, g, "after", null, "w")
        assertEquals(1, JSONArray(prefs(g).getString(WidgetStoreKeys.PENDING_ACTIONS, "[]")).length())
    }

    private fun alarm(): ShadowAlarmManager.ScheduledAlarm? =
        shadowOf(ctx.getSystemService(Context.ALARM_SERVICE) as AlarmManager).peekNextScheduledAlarm()

    private fun nextAlarm(): Long? = alarm()?.triggerAtMs

    /** Given a pending inexact alarm, when earlier, later, stale and post-reboot wakes are scheduled, then the earliest pending wake wins. */
    @Test
    fun alarmSchedulingKeepsTheEarliestWake() {
        val now = System.currentTimeMillis()
        WidgetScheduler.schedule(ctx, now + 60_000)
        assertEquals(now + 60_000, nextAlarm())
        assertTrue("idle-allowed, no exact permission", alarm()!!.allowWhileIdle)
        assertEquals(AlarmManager.RTC, alarm()!!.type)
        // A later one does not replace an earlier pending one; an earlier one does.
        WidgetScheduler.schedule(ctx, now + 120_000)
        assertEquals(now + 60_000, nextAlarm())
        WidgetScheduler.schedule(ctx, now + 30_000)
        assertEquals(now + 30_000, nextAlarm())

        // An alarm whose time passed without firing is stale: a later one replaces it.
        WidgetScheduler.schedule(ctx, now - 1_000)
        WidgetScheduler.schedule(ctx, now + 200_000)
        assertEquals(now + 200_000, nextAlarm())

        // After the alarm fired (cleared), any time schedules again.
        WidgetScheduler.clear(ctx)
        WidgetScheduler.schedule(ctx, now + 300_000)
        assertEquals(now + 300_000, nextAlarm())

        // Reboot: alarm and PendingIntent gone, next_at still stored → re-armed.
        val am = ctx.getSystemService(Context.ALARM_SERVICE) as AlarmManager
        val pi = PendingIntent.getBroadcast(
            ctx, 0, Intent(ctx, WidgetTickReceiver::class.java),
            PendingIntent.FLAG_NO_CREATE or PendingIntent.FLAG_IMMUTABLE,
        )!!
        am.cancel(pi)
        pi.cancel()
        assertNull(nextAlarm())
        WidgetScheduler.schedule(ctx, now + 400_000)
        assertEquals(now + 400_000, nextAlarm())

        // From a stored config: the next entry; nothing upcoming or no config → no new alarm.
        WidgetScheduler.clear(ctx)
        val at = System.currentTimeMillis() + 3_600_000
        WidgetScheduler.scheduleFor(ctx, """{"entries":[{"at":$at}]}""", null, null)
        assertEquals(at, nextAlarm())
        WidgetScheduler.clear(ctx)
        WidgetScheduler.scheduleFor(ctx, null, null, null)
        WidgetScheduler.scheduleFor(ctx, "{}", null, null)
        assertEquals(at, nextAlarm())
    }

    /** Given a cached `source` response, when a fetch fails, then the cache is kept and the next attempt waits a full refresh interval. */
    @Test
    fun sourceFailureBacksOffLikeASuccess() = runTest {
        val id = "w#1"
        assertNull(WidgetSource.read(ctx, id).attempt)
        WidgetSource.write(ctx, id, """{"t":1}""", 1_000)
        WidgetSource.recordFailure(ctx, id, 5_000)
        val c = WidgetSource.read(ctx, id)
        // Last good response kept; the failure is the last attempt.
        assertEquals(1_000L, c.ts)
        assertEquals(5_000L, c.attempt)
        assertEquals(1, (c.data as org.json.JSONObject).getInt("t"))
        assertFalse(WidgetSource.isDue(c.attempt, 5_000 + 15 * 60_000L - 1, 15))
        assertTrue(WidgetSource.isDue(c.attempt, 5_000 + 15 * 60_000L, 15))
        assertTrue(WidgetSource.isDue(null, 0, 15))
        // A success clears the failure stamp.
        WidgetSource.write(ctx, id, "{}", 9_000)
        assertNull(WidgetSource.read(ctx, id).failedAt)
        // Not due: no network, cached data and the attempt time come back.
        val now = System.currentTimeMillis()
        WidgetSource.recordFailure(ctx, id, now)
        val cfg = """{"source":{"url":"https://example.invalid/x","refreshMinutes":15}}"""
        val (data, at) = WidgetSource.enqueueIfDue(ctx, cfg, "g", "w", "1")
        assertEquals(now, at)
        assertEquals(0, (data as org.json.JSONObject).length())
        // The scheduler wakes one refresh after the failed attempt, not in a minute.
        assertEquals(now + 15 * 60_000L, WidgetScheduler.nextWakeMs(org.json.JSONObject(cfg), at, now + 1, 15))
    }
}
