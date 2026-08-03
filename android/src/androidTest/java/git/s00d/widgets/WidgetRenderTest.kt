package git.s00d.widgets

import android.appwidget.AppWidgetHost
import android.appwidget.AppWidgetHostView
import android.appwidget.AppWidgetManager
import android.content.ComponentName
import android.os.Bundle
import android.os.SystemClock
import android.view.View
import androidx.core.view.drawToBitmap
import androidx.glance.appwidget.updateAll
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

/**
 * Level-2 visual stand: AppWidgetHost → real Glance/RemoteViews → bitmap assert.
 *
 * Env: scripts/sh/android-up.sh (grantbind, animations off, pinned clock).
 * Record one case:
 *   ./gradlew :connectedDebugAndroidTest \
 *     -Pgolden.record=true \
 *     -Pandroid.testInstrumentationRunnerArguments.case=weather.small
 */
@RunWith(AndroidJUnit4::class)
class WidgetRenderTest {
    private companion object {
        const val HOST_ID = 0x7ADA
        const val DEADLINE_MS = 15_000L
        const val POLL_MS = 250L
    }

    private lateinit var host: AppWidgetHost

    @Before
    fun setUp() {
        val ctx = InstrumentationRegistry.getInstrumentation().targetContext
        host = AppWidgetHost(ctx, HOST_ID)
        host.startListening()
    }

    @After
    fun tearDown() {
        host.stopListening()
    }

    @Test
    fun renderCases() {
        val ctx = InstrumentationRegistry.getInstrumentation().targetContext
        val filter = Golden.caseFilter()
        val cases = Cases.load(ctx, filter)
        assertTrue("no cases loaded (filter=$filter)", cases.isNotEmpty())

        val mgr = AppWidgetManager.getInstance(ctx)
        val provider = ComponentName(ctx, TauriGlanceWidgetReceiver::class.java)
        val failures = mutableListOf<String>()

        for (case in cases) {
            val id = host.allocateAppWidgetId()
            try {
                val bound = mgr.bindAppWidgetIdIfAllowed(id, provider)
                require(bound) {
                    "bindAppWidgetIdIfAllowed failed — run: " +
                        "adb shell appwidget grantbind --package ${ctx.packageName}"
                }

                val (wDp, hDp) = case.dp()
                val opts = Bundle().apply {
                    putInt(AppWidgetManager.OPTION_APPWIDGET_MIN_WIDTH, wDp)
                    putInt(AppWidgetManager.OPTION_APPWIDGET_MAX_WIDTH, wDp)
                    putInt(AppWidgetManager.OPTION_APPWIDGET_MIN_HEIGHT, hDp)
                    putInt(AppWidgetManager.OPTION_APPWIDGET_MAX_HEIGHT, hDp)
                }
                mgr.updateAppWidgetOptions(id, opts)

                VisualStore.reset(ctx)
                val fixtureJson = Cases.fixtureJson(ctx, case)
                VisualStore.putConfig(ctx, id, fixtureJson)
                runBlocking { TauriGlanceWidget().updateAll(ctx) }

                val info = mgr.getAppWidgetInfo(id)
                    ?: error("no AppWidgetProviderInfo for id=$id (provider not merged?)")

                lateinit var view: AppWidgetHostView
                InstrumentationRegistry.getInstrumentation().runOnMainSync {
                    view = host.createView(ctx, id, info)
                    val density = ctx.resources.displayMetrics.density
                    val wPx = (wDp * density).toInt()
                    val hPx = (hDp * density).toInt()
                    val wSpec = View.MeasureSpec.makeMeasureSpec(wPx, View.MeasureSpec.EXACTLY)
                    val hSpec = View.MeasureSpec.makeMeasureSpec(hPx, View.MeasureSpec.EXACTLY)
                    view.measure(wSpec, hSpec)
                    view.layout(0, 0, view.measuredWidth, view.measuredHeight)
                }

                val bmp = awaitStable(view)
                Golden.assertMatches(ctx, case.name, bmp)
            } catch (t: Throwable) {
                failures += "${case.name}: ${t.message}"
            } finally {
                host.deleteAppWidgetId(id)
                VisualStore.reset(ctx)
            }
        }

        if (failures.isNotEmpty()) {
            throw AssertionError(failures.joinToString("\n"))
        }
    }

    /** Poll until two consecutive frames match and content is non-uniform. */
    private fun awaitStable(view: View): android.graphics.Bitmap {
        val t0 = SystemClock.uptimeMillis()
        var prev: String? = null
        var last: android.graphics.Bitmap? = null
        while (SystemClock.uptimeMillis() - t0 < DEADLINE_MS) {
            var bmp: android.graphics.Bitmap? = null
            InstrumentationRegistry.getInstrumentation().runOnMainSync {
                bmp = view.drawToBitmap()
            }
            val frame = bmp!!
            last = frame
            val h = BitmapUtil.sha256(frame)
            if (h == prev && !BitmapUtil.isUniform(frame)) {
                return frame
            }
            prev = h
            SystemClock.sleep(POLL_MS)
        }
        error(
            "widget did not stabilize within ${DEADLINE_MS}ms " +
                "(lastUniform=${last?.let { BitmapUtil.isUniform(it) }})",
        )
    }
}
