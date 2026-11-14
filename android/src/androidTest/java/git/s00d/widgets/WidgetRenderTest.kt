package git.s00d.widgets

import android.appwidget.AppWidgetHost
import android.appwidget.AppWidgetHostView
import android.appwidget.AppWidgetManager
import android.appwidget.AppWidgetProviderInfo
import android.content.ComponentName
import android.content.Context
import android.widget.RemoteViews
import androidx.work.Configuration
import androidx.work.testing.WorkManagerTestInitHelper
import java.util.concurrent.CountDownLatch
import java.util.concurrent.Executors
import java.util.concurrent.TimeUnit
import android.os.Bundle
import android.os.ParcelFileDescriptor
import android.os.SystemClock
import android.view.View
import android.graphics.Bitmap
import android.graphics.Rect
import android.os.Build
import android.os.Handler
import android.os.Looper
import android.view.PixelCopy
import android.widget.FrameLayout
import androidx.annotation.RequiresApi
import androidx.test.core.app.ActivityScenario
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
 * The host view lives in a real hardware-rendered window ([CaptureActivity]) and is captured
 * with PixelCopy: software drawToBitmap() skips outline clips, so corner radii never showed.
 *
 * Env: scripts/sh/android-up.sh (grantbind, animations off, density 480, pinned clock).
 * Record one case:
 *   ./gradlew :connectedDebugAndroidTest \
 *     -Pgolden.record=true \
 *     -Pandroid.testInstrumentationRunnerArguments.case=weather.small
 */
@RunWith(AndroidJUnit4::class)
@RequiresApi(Build.VERSION_CODES.S) // outline clips (Glance cornerRadius) need 31+
class WidgetRenderTest {
    private companion object {
        const val HOST_ID = 0x7ADA
        // Headless emulator + Glance SessionWorker often exceeds 15s on cold start.
        const val DEADLINE_MS = 45_000L
        const val POLL_MS = 500L
        /** No RemoteViews for this long = Glance is done (follow-up compositions per size). */
        const val QUIET_MS = 1_500L
        // Goldens are xxhdpi (170dp → 510px).
        const val PINNED_DENSITY_DPI = 480
    }

    /** Host view that signals when Glance content (not the loading layout) lands. */
    private class LatchHostView(context: Context) : AppWidgetHostView(context) {
        @Volatile var latch = CountDownLatch(1)
        @Volatile var lastUpdateAt = 0L

        override fun updateAppWidget(remoteViews: RemoteViews?) {
            super.updateAppWidget(remoteViews)
            lastUpdateAt = SystemClock.uptimeMillis()
            if (remoteViews != null && remoteViews.layoutId != R.layout.tauri_widget_loading) latch.countDown()
        }

        /** Block until no RemoteViews arrived for [quietMs] (Glance recomposes per size). */
        fun awaitQuiet(quietMs: Long, deadlineMs: Long) {
            val t0 = SystemClock.uptimeMillis()
            while (SystemClock.uptimeMillis() - lastUpdateAt < quietMs && SystemClock.uptimeMillis() - t0 < deadlineMs) {
                SystemClock.sleep(100L)
            }
        }
    }

    private class LatchHost(context: Context, id: Int) : AppWidgetHost(context, id) {
        override fun onCreateView(context: Context, appWidgetId: Int, appWidget: AppWidgetProviderInfo?): AppWidgetHostView =
            LatchHostView(context)
    }

    private lateinit var host: AppWidgetHost
    private lateinit var scenario: ActivityScenario<CaptureActivity>
    private lateinit var activity: CaptureActivity

    @Before
    fun setUp() {
        val ctx = InstrumentationRegistry.getInstrumentation().targetContext
        // In-memory WorkManager with a real executor: no work persisted from earlier runs
        // (stale source fetches, sessions), Glance sessions still run.
        WorkManagerTestInitHelper.initializeTestWorkManager(
            ctx,
            Configuration.Builder().setExecutor(Executors.newFixedThreadPool(2)).build(),
        )
        shell("wm density $PINNED_DENSITY_DPI")
        host = LatchHost(ctx, HOST_ID)
        host.startListening()
        scenario = ActivityScenario.launch(CaptureActivity::class.java)
        scenario.onActivity { activity = it }
    }

    @After
    fun tearDown() {
        host.stopListening()
        scenario.close()
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

        // Gradle installs the test APK fresh on each run, so a grantbind from android-up.sh
        // (before install) does not stick. The shell uid may grant it here.
        shell("appwidget grantbind --package ${ctx.packageName}")
        val mgr = AppWidgetManager.getInstance(ctx)
        val provider = ComponentName(ctx, TauriGlanceWidgetReceiver::class.java)
        val failures = mutableListOf<String>()

        for (case in cases) {
            val id = host.allocateAppWidgetId()
            var hosted: View? = null
            try {
                applyCaseTheme(case)

                val bound = mgr.bindAppWidgetIdIfAllowed(id, provider)
                require(bound) {
                    "bindAppWidgetIdIfAllowed failed — run: " +
                        "adb shell appwidget grantbind --package ${ctx.packageName}"
                }

                // Config first: the options change below already starts a Glance update, and
                // with the store still empty (or the previous case's) that composition raced
                // the real one and was sometimes the one captured.
                VisualStore.reset(ctx)
                VisualStore.putConfig(ctx, id, Cases.fixtureJson(ctx, case))

                val (wDp, hDp) = case.dp()
                // Launchers report the size left inside the host view's default padding;
                // reporting the full frame made LocalSize larger than the drawn area
                // (slot-sized charts were cut at the bottom).
                @Suppress("DEPRECATION")
                val pad = AppWidgetHostView.getDefaultPaddingForWidget(ctx, provider, null)
                val optW = wDp - ((pad.left + pad.right) / density).toInt()
                val optH = hDp - ((pad.top + pad.bottom) / density).toInt()
                val opts = Bundle().apply {
                    putInt(AppWidgetManager.OPTION_APPWIDGET_MIN_WIDTH, optW)
                    putInt(AppWidgetManager.OPTION_APPWIDGET_MAX_WIDTH, optW)
                    putInt(AppWidgetManager.OPTION_APPWIDGET_MIN_HEIGHT, optH)
                    putInt(AppWidgetManager.OPTION_APPWIDGET_MAX_HEIGHT, optH)
                }
                mgr.updateAppWidgetOptions(id, opts)

                val info = mgr.getAppWidgetInfo(id)
                    ?: error("no AppWidgetProviderInfo for id=$id (provider not merged?)")

                // Create HostView before updateAll so it is listening when RemoteViews land.
                val wPx = (wDp * density).toInt()
                val hPx = (hDp * density).toInt()

                lateinit var view: LatchHostView
                InstrumentationRegistry.getInstrumentation().runOnMainSync {
                    view = host.createView(ctx, id, info) as LatchHostView
                    activity.root.addView(view, FrameLayout.LayoutParams(wPx, hPx))
                    hosted = view
                }

                view.latch = CountDownLatch(1)
                runBlocking { TauriGlanceWidget().updateAll(ctx) }
                // Wait for Glance's RemoteViews on the host view itself (no fixed sleep), then
                // let awaitStable settle follow-up compositions.
                require(view.latch.await(DEADLINE_MS, TimeUnit.MILLISECONDS)) {
                    "no Glance RemoteViews within ${DEADLINE_MS}ms (still the loading layout)"
                }
                view.awaitQuiet(QUIET_MS, DEADLINE_MS)

                val bmp = awaitStable(view, wPx, hPx)
                Golden.assertMatches(ctx, case.name, bmp)
            } catch (t: Throwable) {
                failures += "${case.name}: ${t.message}"
            } finally {
                hosted?.let { v -> InstrumentationRegistry.getInstrumentation().runOnMainSync { activity.root.removeView(v) } }
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
        // Configuration change is async: poll (with a deadline) until the app sees the new uiMode.
        val want = if (night == "yes") android.content.res.Configuration.UI_MODE_NIGHT_YES else android.content.res.Configuration.UI_MODE_NIGHT_NO
        val app = InstrumentationRegistry.getInstrumentation().targetContext.applicationContext
        val t0 = SystemClock.uptimeMillis()
        while ((app.resources.configuration.uiMode and android.content.res.Configuration.UI_MODE_NIGHT_MASK) != want &&
            SystemClock.uptimeMillis() - t0 < DEADLINE_MS
        ) {
            SystemClock.sleep(50L)
        }
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
            var rect: Rect? = null
            var childCount = -1
            InstrumentationRegistry.getInstrumentation().runOnMainSync {
                childCount = (view as? android.view.ViewGroup)?.childCount ?: -1
                // Settle state animations (CheckBox / Switch check marks) to their end state.
                view.jumpDrawablesToCurrentState()
                if (view.isLaidOut && view.width == wPx && view.height == hPx) {
                    val loc = IntArray(2)
                    view.getLocationInWindow(loc)
                    rect = Rect(loc[0], loc[1], loc[0] + wPx, loc[1] + hPx)
                }
            }
            lastChildren = childCount
            val r = rect
            if (r == null) { prev = null; continue } // not laid out in the window yet
            val frame = pixelCopy(r)
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

    /** Hardware-path capture of the window's last frame (applies outline clips / rounded corners). */
    private fun pixelCopy(r: Rect): Bitmap {
        val bmp = Bitmap.createBitmap(r.width(), r.height(), Bitmap.Config.ARGB_8888)
        val done = CountDownLatch(1)
        var result = -1
        PixelCopy.request(activity.window, r, bmp, { result = it; done.countDown() }, Handler(Looper.getMainLooper()))
        require(done.await(DEADLINE_MS, TimeUnit.MILLISECONDS)) { "PixelCopy timed out after ${DEADLINE_MS}ms" }
        check(result == PixelCopy.SUCCESS) { "PixelCopy failed: $result" }
        return bmp
    }

    private fun shell(cmd: String) {
        val automation = InstrumentationRegistry.getInstrumentation().uiAutomation
        val pfd = automation.executeShellCommand(cmd)
        ParcelFileDescriptor.AutoCloseInputStream(pfd).use { stream ->
            BufferedReader(InputStreamReader(stream)).use { it.readText() }
        }
    }
}
