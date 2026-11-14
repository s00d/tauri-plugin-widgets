package git.s00d.widgets

import android.Manifest
import android.app.Notification
import android.app.NotificationManager
import android.content.Context
import androidx.core.app.NotificationCompat
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
import org.json.JSONArray
import org.json.JSONObject
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

/** Live activities as ongoing notifications: lifecycle, out-of-order guard, content mapping. */
@RunWith(RobolectricTestRunner::class)
@Config(sdk = [34])
class ActivityNotificationsTest {
    private lateinit var c: Context
    private val g = "group.test"

    @Before
    fun setUp() {
        c = ApplicationProvider.getApplicationContext()
        shadowOf(c as android.app.Application).grantPermissions(Manifest.permission.POST_NOTIFICATIONS)
    }

    private fun shown(id: String): Notification? =
        shadowOf(c.getSystemService(NotificationManager::class.java)).getNotification(ActivityNotifications.notificationId(c, id))

    private fun postAndGet(content: JSONObject): Notification {
        ActivityNotifications.post(c, content, g)
        return shown(content.getString("id"))!!
    }

    private fun title(n: Notification?) = n?.extras?.getCharSequence(Notification.EXTRA_TITLE)?.toString()

    /** Given a registered template, when an activity starts, updates out of order and ends, then stale updates drop and a fresh start applies again. */
    @Test
    fun activityLifecycleWithTemplateAndStaleGuard() {
        c.getSharedPreferences(g, Context.MODE_PRIVATE).edit().putString(
            "activity-template:ride",
            """{"title":"Driver {{data.name}}","body":"ETA {{data.eta}} min","subText":"keep"}""",
        ).commit()
        // Template expansion: fields on the request win, a blank title takes the template's.
        val start = JSONObject("""{"id":"r1","title":"Override","template":"ride","data":{"name":"Ann","eta":4},"timestamp":2000}""")
        val expanded = ActivityNotifications.expandTemplate(c, start, g)
        assertEquals("Override", expanded.getString("title"))
        assertEquals("ETA 4 min", expanded.getString("body"))
        assertEquals("keep", expanded.getString("subText"))
        assertFalse(expanded.has("template") || expanded.has("data"))
        val blank = ActivityNotifications.expandTemplate(c, JSONObject("""{"id":"r1","title":" ","template":"ride","data":{"name":"Bo"},"subText":null}"""), g)
        assertEquals("Driver Bo", blank.getString("title"))
        assertEquals("keep", blank.getString("subText"))
        val unknown = JSONObject("""{"id":"r2","title":"T","template":"nope"}""")
        assertEquals(unknown.toString(), ActivityNotifications.expandTemplate(c, unknown, g).toString())

        // Start.
        assertFalse(ActivityNotifications.isActive(c, "r1"))
        assertTrue(ActivityNotifications.post(c, start, g))
        assertTrue(ActivityNotifications.isActive(c, "r1"))
        assertEquals("ETA 4 min", shown("r1")!!.extras.getCharSequence(Notification.EXTRA_TEXT)?.toString())
        assertEquals(listOf("r1"), ActivityNotifications.list(c))
        // Update with an older timestamp: dropped, the notification stays.
        assertFalse(ActivityNotifications.post(c, JSONObject("""{"id":"r1","title":"old","timestamp":1000}"""), g))
        assertEquals("Override", title(shown("r1")))
        // Same timestamp applies; no timestamp applies and keeps the last one as the guard.
        assertTrue(ActivityNotifications.post(c, JSONObject("""{"id":"r1","title":"same","timestamp":2000}"""), g))
        assertTrue(ActivityNotifications.post(c, JSONObject("""{"id":"r1","title":"v3"}"""), g))
        assertFalse(ActivityNotifications.post(c, JSONObject("""{"id":"r1","title":"late","timestamp":1500}"""), g))
        assertEquals("v3", title(shown("r1")))
        // End clears the guard: a start with an older timestamp applies.
        assertTrue(ActivityNotifications.end(c, "r1"))
        assertNull(shown("r1"))
        assertFalse(ActivityNotifications.end(c, "r1"))
        assertTrue(ActivityNotifications.post(c, JSONObject("""{"id":"r1","title":"again","timestamp":10}"""), g))
        assertEquals("again", title(shown("r1")))
    }

    // ---- model-based chain for the timestamp guard ----

    private sealed interface Cmd
    private data class Post(val id: String, val ts: Long?) : Cmd
    private data class End(val id: String) : Cmd

    /** Given generated post / end sequences, when run on notifications and a model, then accepted / dropped outcomes and the shown title agree after every step. */
    @OptIn(io.kotest.common.ExperimentalKotest::class)
    @Test
    fun timestampGuardAgreesWithModel(): Unit = runBlocking {
        val ids = listOf("a", "b")
        val cmd: Arb<Cmd> = Arb.choice(
            Arb.bind(Arb.element(ids), Arb.element(listOf<Long?>(null, 0, 1, 2, 3, 5, 8))) { id, ts -> Post(id, ts) },
            Arb.element(ids).map { End(it) },
        )
        var run = 0
        checkAll(PropTestConfig(seed = 20261113L, iterations = 60), Arb.list(cmd, 1..20)) { cmds ->
            val prefix = "m${run++}."
            val active = mutableMapOf<String, Boolean>()
            val last = mutableMapOf<String, Long?>()
            val shownTitle = mutableMapOf<String, String>()
            for ((step, x) in cmds.withIndex()) {
                when (x) {
                    is Post -> {
                        val id = prefix + x.id
                        val was = active[id] == true
                        val prev = last[id]
                        val want = !was || x.ts == null || prev == null || x.ts >= prev
                        val req = JSONObject().put("id", id).put("title", "s$step")
                        x.ts?.let { req.put("timestamp", it) }
                        assertEquals("$step $x in $cmds", want, ActivityNotifications.post(c, req, g))
                        if (want) {
                            active[id] = true
                            shownTitle[id] = "s$step"
                            if (x.ts != null) last[id] = x.ts else if (!was) last[id] = null
                        }
                    }
                    is End -> {
                        val id = prefix + x.id
                        assertEquals("$step $x in $cmds", active[id] == true, ActivityNotifications.end(c, id))
                        active[id] = false
                        last[id] = null
                    }
                }
                for (id in ids.map { prefix + it }) {
                    val on = active[id] == true
                    assertEquals("$step active $id in $cmds", on, ActivityNotifications.isActive(c, id))
                    assertEquals("$step shown $id in $cmds", if (on) shownTitle[id] else null, title(shown(id)))
                }
            }
        }
        Unit
    }

    /** Given activity requests with timers, visibility and actions, when posted, then the notification maps every field (incl. the redacted copy, Samsung, Live Update plan). */
    @Test
    fun notificationContentMapsFields() {
        val ride = postAndGet(
            JSONObject().put("id", "ride.1").put("title", "Ride").put("subText", "Driver").put("shortCriticalText", "5 min")
                .put("startedAt", 1_000_000L).put("visibility", "secret").put("category", "navigation").put(
                    "actions",
                    JSONArray()
                        .put(JSONObject().put("id", "stop").put("label", "Stop").put("systemName", "pause.fill"))
                        .put(JSONObject().put("id", "open").put("label", "Open").put("url", "myapp://ride/1")),
                ),
        )
        assertEquals("Driver", ride.extras.getCharSequence(Notification.EXTRA_SUB_TEXT)?.toString())
        assertEquals("5 min", ride.extras.getString(ActivityNotifications.EXTRA_SHORT_CRITICAL_TEXT))
        assertEquals(Notification.VISIBILITY_SECRET, ride.visibility)
        assertEquals(Notification.CATEGORY_NAVIGATION, ride.category)
        // startedAt without endsAt → stopwatch counting up.
        assertEquals(1_000_000L, ride.`when`)
        assertTrue(ride.extras.getBoolean(Notification.EXTRA_SHOW_CHRONOMETER))
        assertFalse(ride.extras.getBoolean(Notification.EXTRA_CHRONOMETER_COUNT_DOWN))
        assertEquals(2, ride.actions.size)
        assertEquals(android.R.drawable.ic_media_pause, ride.actions[0].icon)
        assertTrue(shadowOf(ride.actions[0].actionIntent).isBroadcastIntent)
        val open = shadowOf(ride.actions[1].actionIntent)
        assertTrue(open.isActivityIntent)
        assertEquals("myapp://ride/1", open.savedIntent.dataString)
        // secret: no public version.
        assertNull(ride.publicVersion)

        // Defaults; endsAt wins over startedAt (countdown); unknown symbol → no icon.
        val order = postAndGet(
            JSONObject().put("id", "order.2").put("title", "Order").put("endsAt", 9_000_000L).put("startedAt", 1_000L)
                .put("actions", JSONArray().put(JSONObject().put("id", "x").put("label", "X").put("systemName", "nope.symbol"))),
        )
        assertEquals(Notification.VISIBILITY_PUBLIC, order.visibility)
        assertEquals(Notification.CATEGORY_PROGRESS, order.category)
        assertNull(order.extras.getCharSequence(Notification.EXTRA_SUB_TEXT))
        assertNull(order.extras.getString(ActivityNotifications.EXTRA_SHORT_CRITICAL_TEXT))
        assertEquals(9_000_000L, order.`when`)
        assertTrue(order.extras.getBoolean(Notification.EXTRA_CHRONOMETER_COUNT_DOWN))
        assertEquals(0, order.actions[0].icon)
        assertNull(order.publicVersion)

        // private: a redacted lock-screen copy (app name, progress, timer; no text, no actions).
        val lock = postAndGet(
            JSONObject().put("id", "lock.1").put("title", "Secret ride").put("body", "To 5 Main St")
                .put("visibility", "private").put("progress", 0.5).put("endsAt", 9_000_000L)
                .put("actions", JSONArray().put(JSONObject().put("id", "x").put("label", "X"))),
        )
        val pub = lock.publicVersion!!
        assertEquals(c.applicationInfo.loadLabel(c.packageManager).toString(), title(pub))
        assertNull(pub.extras.getCharSequence(Notification.EXTRA_TEXT))
        assertEquals(500, pub.extras.getInt(Notification.EXTRA_PROGRESS))
        assertEquals(9_000_000L, pub.`when`)
        assertTrue(pub.actions.isNullOrEmpty())

        // Mappings.
        assertEquals(NotificationCompat.VISIBILITY_PRIVATE, ActivityNotifications.visibility("private"))
        assertEquals(NotificationCompat.VISIBILITY_PUBLIC, ActivityNotifications.visibility("bogus"))
        for (cat in listOf("progress", "navigation", "transport", "workout", "status", "alarm", "service", "event")) {
            assertEquals(cat, ActivityNotifications.category(cat))
        }
        assertEquals(NotificationCompat.CATEGORY_PROGRESS, ActivityNotifications.category("bogus"))

        // Samsung One UI extras: only on Samsung, mapped from the content, redacted / hidden by visibility.
        val k = ActivityNotifications.SAMSUNG
        assertFalse(postAndGet(JSONObject().put("id", "sam.1").put("title", "T")).extras.containsKey(k + "style"))
        org.robolectric.shadows.ShadowBuild.setManufacturer("samsung")
        try {
            val n = postAndGet(JSONObject().put("id", "sam.1").put("title", "T"))
            assertEquals(1, n.extras.getInt(k + "style"))
            assertTrue(n.extras.containsKey(k + "chipIcon"))
        } finally {
            org.robolectric.shadows.ShadowBuild.reset()
        }
        val b = ActivityNotifications.samsungExtras(
            JSONObject().put("title", "Pizza").put("body", "Out for delivery").put("shortCriticalText", "5 min")
                .put("tint", "#ff0000").put("progress", 0.42)
                .put("actions", JSONArray().put(JSONObject().put("id", "x").put("label", "X"))),
            "App", null,
        )!!
        assertEquals(1, b.getInt(k + "style"))
        assertEquals("Pizza", b.getString(k + "primaryInfo"))
        assertEquals("Out for delivery", b.getString(k + "secondaryInfo"))
        assertEquals("5 min", b.getString(k + "chipExpandedText"))
        assertEquals(0xFFFF0000.toInt(), b.getInt(k + "chipBgColor"))
        assertEquals(42, b.getInt(k + "progress"))
        assertEquals(100, b.getInt(k + "progressMax"))
        assertEquals(1, b.getInt(k + "actionType"))
        assertFalse(b.containsKey(k + "nowbarPrimaryInfo"))
        val plain = ActivityNotifications.samsungExtras(JSONObject().put("title", "T"), "App", null)!!
        assertEquals("T", plain.getString(k + "chipExpandedText"))
        assertFalse(plain.containsKey(k + "progress"))
        assertFalse(plain.containsKey(k + "actionType"))
        assertFalse(plain.containsKey(k + "secondaryInfo"))
        val priv = ActivityNotifications.samsungExtras(JSONObject().put("title", "T").put("body", "B").put("visibility", "private"), "App", null)!!
        assertEquals("App", priv.getString(k + "nowbarPrimaryInfo"))
        assertEquals("", priv.getString(k + "nowbarSecondaryInfo"))
        assertNull(ActivityNotifications.samsungExtras(JSONObject().put("title", "T").put("visibility", "secret"), "App", null))

        // Live Update progress plan (API 36 ProgressStyle); on API 34 no promotion, app settings page.
        assertNull(ActivityNotifications.progressPlan(JSONObject("""{"id":"a"}""")))
        val one = ActivityNotifications.progressPlan(JSONObject("""{"progress":0.25,"tint":"#ff0000"}"""))!!
        assertEquals(250, one.progress)
        assertEquals(listOf(1000 to android.graphics.Color.RED), one.segments)
        val multi = ActivityNotifications.progressPlan(
            JSONObject("""{"progress":0.5,"progressSegments":[{"length":30,"color":"#00ff00"},{"length":70},{"length":0}],"progressPoints":[{"at":0.3,"color":"#0000ff"}]}"""),
        )!!
        assertEquals(50, multi.progress)
        assertEquals(listOf(30 to android.graphics.Color.GREEN, 70 to null), multi.segments)
        assertEquals(listOf(30 to android.graphics.Color.BLUE), multi.points)
        assertFalse(ActivityNotifications.canPostPromoted(c))
        val settings = ActivityNotifications.promotionSettingsIntent(c)
        assertEquals(android.provider.Settings.ACTION_APP_NOTIFICATION_SETTINGS, settings.action)
        assertEquals(c.packageName, settings.getStringExtra(android.provider.Settings.EXTRA_APP_PACKAGE))
    }
}
