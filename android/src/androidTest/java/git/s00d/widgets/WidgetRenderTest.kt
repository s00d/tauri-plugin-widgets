package git.s00d.widgets

import android.appwidget.AppWidgetHost
import android.appwidget.AppWidgetHostView
import android.appwidget.AppWidgetManager
import android.content.ComponentName
import android.os.Bundle
import android.os.ParcelFileDescriptor
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
import java.io.BufferedReader
import java.io.InputStreamReader

/**
 * Level-2 visual stand: AppWidgetHost → real Glance/RemoteViews → bitmap assert.
 *
 * Env: scripts/sh/android-up.sh (grantbind, animations off, density 480, pinned clock).
 * Record one case:
 *   ./gradlew :connectedDebugAndroidTest \
 *     -Pgolden.record=true \
 *     -Pandroid.testInstrumentationRunnerArguments.case=weather.small
 */
@RunWith(AndroidJUnit4::class)
class WidgetRenderTest {
    private companion object {
        const val HOST_ID = 0x7ADA
        // Headless emulator + Glance SessionWorker often exceeds 15s on cold start.
        const val DEADLINE_MS = 45_000L
        const val POLL_MS = 500L
        // Goldens are xxhdpi (170dp → 510px).
        const val PINNED_DENSITY_DPI = 480
    }

    private lateinit var host: AppWidgetHost

    @Before
    fun setUp() {
        val ctx = InstrumentationRegistry.getInstrumentation().targetContext
        shell("wm density $PINNED_DENSITY_DPI")
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

        val density = ctx.resources.displayMetrics.density
        require(density in 2.9f..3.1f) {
            "display density=$density (dpi=${ctx.resources.displayMetrics.densityDpi}); " +
                "need ~3.0 (xxhdpi). Run scripts/sh/android-up.sh or: adb shell wm density $PINNED_DENSITY_DPI"
        }

        val mgr = AppWidgetManager.getInstance(ctx)
        val provider = ComponentName(ctx, TauriGlanceWidgetReceiver::class.java)
        val failures = mutableListOf<String>()

        for (case in cases) {
            val id = host.allocateAppWidgetId()
            try {
                applyCaseTheme(case)

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

                val info = mgr.getAppWidgetInfo(id)
                    ?: error("no AppWidgetProviderInfo for id=$id (provider not merged?)")

                // Create HostView before updateAll so it is listening when RemoteViews land.
                val wPx = (wDp * density).toInt()
                val hPx = (hDp * density).toInt()

                lateinit var view: AppWidgetHostView
                InstrumentationRegistry.getInstrumentation().runOnMainSync {
                    view = host.createView(ctx, id, info)
                    val wSpec = View.MeasureSpec.makeMeasureSpec(wPx, View.MeasureSpec.EXACTLY)
                    val hSpec = View.MeasureSpec.makeMeasureSpec(hPx, View.MeasureSpec.EXACTLY)
                    view.measure(wSpec, hSpec)
                    view.layout(0, 0, view.measuredWidth, view.measuredHeight)
                }

                VisualStore.reset(ctx)
                val fixtureJson = Cases.fixtureJson(ctx, case)
                VisualStore.putConfig(ctx, id, fixtureJson)
                runBlocking { TauriGlanceWidget().updateAll(ctx) }
                // Glance applies RemoteViews on SessionWorker; give the worker a quiet
                // window before we start hammering runOnMainSync in awaitStable.
                SystemClock.sleep(3_000L)

                val bmp = awaitStable(view, wPx, hPx)
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

    private fun applyCaseTheme(case: VisualCase) {
        val night = if (case.theme.equals("light", ignoreCase = true)) "no" else "yes"
        shell("cmd uimode night $night")
        // Configuration change is async; brief settle before Glance reads uiMode.
        SystemClock.sleep(500L)
    }

    /** Poll until two consecutive frames match and content is non-uniform. */
    private fun awaitStable(view: View, wPx: Int, hPx: Int): android.graphics.Bitmap {
        val t0 = SystemClock.uptimeMillis()
        var prev: String? = null
        var last: android.graphics.Bitmap? = null
        var lastChildren = -1
        while (SystemClock.uptimeMillis() - t0 < DEADLINE_MS) {
            // Sleep first so SessionWorker can use the main looper between captures.
            SystemClock.sleep(POLL_MS)
            var bmp: android.graphics.Bitmap? = null
            var childCount = -1
            InstrumentationRegistry.getInstrumentation().runOnMainSync {
                val wSpec = View.MeasureSpec.makeMeasureSpec(wPx, View.MeasureSpec.EXACTLY)
                val hSpec = View.MeasureSpec.makeMeasureSpec(hPx, View.MeasureSpec.EXACTLY)
                view.measure(wSpec, hSpec)
                view.layout(0, 0, view.measuredWidth, view.measuredHeight)
                childCount = (view as? android.view.ViewGroup)?.childCount ?: -1
                bmp = view.drawToBitmap()
            }
            lastChildren = childCount
            val frame = bmp!!
            last = frame
            val h = BitmapUtil.sha256(frame)
            if (h == prev && !BitmapUtil.isUniform(frame)) {
                if (BitmapUtil.looksLikeGlanceError(frame)) {
                    error(
                        "Glance error placeholder (bitmap budget / compose fail) " +
                            "size=${frame.width}x${frame.height} children=$lastChildren — " +
                            "check logcat for 'exceeds maximum bitmap memory usage'",
                    )
                }
                return frame
            }
            prev = h
        }
        error(
            "widget did not stabilize within ${DEADLINE_MS}ms " +
                "(lastUniform=${last?.let { BitmapUtil.isUniform(it) }} " +
                "size=${last?.width}x${last?.height} children=$lastChildren)",
        )
    }

    private fun shell(cmd: String) {
        val automation = InstrumentationRegistry.getInstrumentation().uiAutomation
        val pfd = automation.executeShellCommand(cmd)
        ParcelFileDescriptor.AutoCloseInputStream(pfd).use { stream ->
            BufferedReader(InputStreamReader(stream)).use { it.readText() }
        }
    }
}
