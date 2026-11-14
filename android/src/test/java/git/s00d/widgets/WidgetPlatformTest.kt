package git.s00d.widgets

import android.app.Activity
import android.appwidget.AppWidgetManager
import android.content.Context
import android.content.Intent
import android.widget.ListView
import androidx.test.core.app.ApplicationProvider
import androidx.work.ListenableWorker
import androidx.work.WorkInfo
import androidx.work.WorkManager
import androidx.work.testing.TestListenableWorkerBuilder
import androidx.work.testing.WorkManagerTestInitHelper
import androidx.work.workDataOf
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.Robolectric
import org.robolectric.RobolectricTestRunner
import org.robolectric.Shadows.shadowOf
import org.robolectric.annotation.Config
import java.io.File

/** Configuration activity, source worker, content-URI images, composition-error fallback. */
@RunWith(RobolectricTestRunner::class)
@Config(sdk = [34])
class WidgetPlatformTest {
    private lateinit var c: Context

    @Before
    fun setUp() {
        c = ApplicationProvider.getApplicationContext()
        WorkManagerTestInitHelper.initializeTestWorkManager(c)
    }

    private fun configIntent(id: Int) = Intent(c, WidgetConfigActivity::class.java)
        .putExtra(AppWidgetManager.EXTRA_APPWIDGET_ID, id)

    /** Given a placement with and without `instances:` choices, when the config activity opens, then it finishes at once or stores the picked instance, and removal restores the appWidgetId. */
    @Test
    fun configActivityPicksPlacementInstance() {
        // No appWidgetId: cancelled.
        val bad = Robolectric.buildActivity(WidgetConfigActivity::class.java, Intent(c, WidgetConfigActivity::class.java)).create().get()
        assertEquals(Activity.RESULT_CANCELED, shadowOf(bad).resultCode)
        // No choices: finishes OK with the id.
        val a0 = Robolectric.buildActivity(WidgetConfigActivity::class.java, configIntent(11)).create().get()
        assertTrue(a0.isFinishing)
        assertEquals(Activity.RESULT_OK, shadowOf(a0).resultCode)
        assertEquals(11, shadowOf(a0).resultIntent.getIntExtra(AppWidgetManager.EXTRA_APPWIDGET_ID, -1))

        WidgetStoreKeys.bindInstance(c, 12, "weather", "g.cfg")
        c.getSharedPreferences("g.cfg", Context.MODE_PRIVATE).edit()
            .putString("config:weather", "{}")
            .putString("instances:weather", """[{"id":"nyc","title":"New York"},{"id":"ber"},{"title":"no id"}]""")
            .commit()
        assertEquals(listOf("nyc" to "New York", "ber" to "ber"), WidgetConfigActivity.options(c, 12))
        assertEquals("12", WidgetStoreKeys.placementInstance(c, 12))
        val a = Robolectric.buildActivity(WidgetConfigActivity::class.java, configIntent(12)).create().visible().get()
        assertFalse(a.isFinishing)
        assertEquals(Activity.RESULT_CANCELED, shadowOf(a).resultCode)
        val list = a.findViewById<android.view.ViewGroup>(android.R.id.content).getChildAt(0) as ListView
        list.performItemClick(list, 1, 1)
        assertTrue(a.isFinishing)
        assertEquals(Activity.RESULT_OK, shadowOf(a).resultCode)
        assertEquals("ber", WidgetStoreKeys.placementInstance(c, 12))
        assertEquals("ber", Placement.read(c, 12).instance)
        // Placement removed: back to the appWidgetId.
        WidgetStoreKeys.clearInstance(c, 12)
        assertEquals("12", WidgetStoreKeys.placementInstance(c, 12))
    }

    /** Given a due `source`, when it is enqueued twice and the worker fails, then one unique job exists, the worker retries and finally records the failure. */
    @Test
    fun sourceIsQueuedOnceAndWorkerBacksOff() = runTest {
        val cfg = """{"source":{"url":"https://example.invalid/x","refreshMinutes":15}}"""
        WidgetSource.enqueueIfDue(c, cfg, "g.src", "w", "3")
        WidgetSource.enqueueIfDue(c, cfg, "g.src", "w", "3")
        val infos = WorkManager.getInstance(c).getWorkInfosForUniqueWork(WidgetSource.workName("g.src", "w#3")).get()
        assertEquals(1, infos.size)
        assertEquals(WorkInfo.State.ENQUEUED, infos[0].state)
        // No source: nothing queued.
        WidgetSource.enqueueIfDue(c, "{}", "g.src", "v", null)
        assertTrue(WorkManager.getInstance(c).getWorkInfosForUniqueWork(WidgetSource.workName("g.src", "v")).get().isEmpty())

        c.getSharedPreferences("g.wk", Context.MODE_PRIVATE).edit()
            .putString("config:w", """{"source":{"url":"http://127.0.0.1:1/x"}}""").commit()
        val input = workDataOf("group" to "g.wk", "widgetId" to "w", "instance" to "5")
        val first = TestListenableWorkerBuilder<WidgetSourceWorker>(c).setInputData(input).build().doWork()
        assertEquals(ListenableWorker.Result.retry(), first)
        assertNull(WidgetSource.read(c, "w#5").failedAt)
        val last = TestListenableWorkerBuilder<WidgetSourceWorker>(c).setInputData(input)
            .setRunAttemptCount(WidgetSource.MAX_RETRIES).build().doWork()
        assertEquals(ListenableWorker.Result.failure(), last)
        assertNotNull(WidgetSource.read(c, "w#5").failedAt)
        val none = TestListenableWorkerBuilder<WidgetSourceWorker>(c)
            .setInputData(workDataOf("group" to "g.wk", "widgetId" to "missing")).build().doWork()
        assertEquals(ListenableWorker.Result.success(), none)
    }

    /** Given cached images and a composition error, when the renderer falls back, then cache files get content URIs and the error leaves a receipt and the error layout. */
    @Test
    fun renderFallbacks() {
        val dir = File(c.cacheDir, "tauri_widget_images").apply { mkdirs() }
        val img = File(dir, "a.png").apply { writeBytes(byteArrayOf(1, 2, 3)) }
        val uri = cachedImageUri(c, img.absolutePath)
        assertEquals("content", uri?.scheme)
        // Outside the cache dir, or missing: no URI (the renderer decodes a bitmap).
        assertNull(cachedImageUri(c, File(c.filesDir, "b.png").apply { writeBytes(byteArrayOf(1)) }.absolutePath))
        assertNull(cachedImageUri(c, File(dir, "missing.png").absolutePath))

        WidgetStoreKeys.bindInstance(c, 21, "w", "g.err")
        c.getSharedPreferences("g.err", Context.MODE_PRIVATE).edit().putString("config:w", "{}").commit()
        TauriGlanceWidget.writeErrorReceipt(c, 21, IllegalStateException("boom"))
        val r = WidgetReceipts.list(c, "g.err").getJSONObject(0)
        assertEquals("error", r.getString("trigger"))
        assertTrue("error recorded", r.optString("error").isNotEmpty())
        assertEquals(21, r.getInt("appWidgetId"))
        assertEquals(listOf(21), WidgetReceipts.liveInstances(c, "g.err", "w"))
        assertEquals(R.layout.tauri_widget_error, TauriGlanceWidget.errorViews(c).layoutId)
    }
}
