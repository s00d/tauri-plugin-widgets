package git.s00d.widgets

import android.content.Context
import android.graphics.Bitmap
import android.graphics.Canvas
import android.view.View
import android.view.ViewGroup
import android.widget.FrameLayout
import android.widget.TextView
import androidx.compose.ui.unit.DpSize
import androidx.compose.ui.unit.dp
import androidx.glance.appwidget.ExperimentalGlanceRemoteViewsApi
import androidx.glance.appwidget.GlanceRemoteViews
import kotlinx.coroutines.runBlocking
import org.json.JSONArray
import org.json.JSONObject
import org.robolectric.RuntimeEnvironment
import java.io.File

data class Rendered(
    val bitmap: Bitmap,
    val tree: JSONObject,
)

object RenderHarness {
    private val sizeViewports = mapOf(
        "small" to (170 to 170),
        "medium" to (360 to 170),
        "large" to (360 to 380),
    )

    fun viewportFor(size: String): Pair<Int, Int> =
        sizeViewports[size] ?: (170 to 170)

    @OptIn(ExperimentalGlanceRemoteViewsApi::class)
    fun render(json: String, size: String): Rendered {
        val context = RuntimeEnvironment.getApplication()
        val (wDp, hDp) = viewportFor(size)
        val density = context.resources.displayMetrics.density
        val wPx = (wDp * density).toInt()
        val hPx = (hDp * density).toInt()

        val remoteViews = runBlocking {
            GlanceRemoteViews().compose(
                context = context,
                size = DpSize(wDp.dp, hDp.dp),
            ) {
                WidgetRootDirect(context, json, size)
            }.remoteViews
        }

        val host = FrameLayout(context)
        val view = remoteViews.apply(context, host)
        host.addView(view)

        val wSpec = View.MeasureSpec.makeMeasureSpec(wPx, View.MeasureSpec.EXACTLY)
        val hSpec = View.MeasureSpec.makeMeasureSpec(hPx, View.MeasureSpec.EXACTLY)
        host.measure(wSpec, hSpec)
        host.layout(0, 0, host.measuredWidth, host.measuredHeight)

        val bmp = Bitmap.createBitmap(
            host.measuredWidth.coerceAtLeast(1),
            host.measuredHeight.coerceAtLeast(1),
            Bitmap.Config.ARGB_8888,
        )
        host.draw(Canvas(bmp))
        return Rendered(bmp, dumpTree(host))
    }

    fun dumpTree(v: View): JSONObject {
        val obj = JSONObject()
        obj.put("kind", v.javaClass.simpleName)
        obj.put(
            "rect",
            JSONArray(listOf(v.left, v.top, v.width, v.height)),
        )
        if (v is TextView) {
            val t = v.text?.toString()?.trim().orEmpty()
            if (t.isNotEmpty()) {
                obj.put("text", t)
                obj.put("px", v.textSize.toInt())
                obj.put("color", "#%08X".format(v.currentTextColor))
            }
        }
        if (v is ViewGroup) {
            val children = JSONArray()
            for (i in 0 until v.childCount) {
                children.put(dumpTree(v.getChildAt(i)))
            }
            if (children.length() > 0) obj.put("children", children)
        }
        return obj
    }

    fun fixturesRoot(): File {
        val prop = System.getProperty("fixtures.root")
        if (!prop.isNullOrBlank()) return File(prop)
        // user.dir is android/ when running Gradle unit tests
        val fromCwd = File(System.getProperty("user.dir"), "../tests/fixtures").canonicalFile
        if (fromCwd.isDirectory) return fromCwd
        return File(RuntimeEnvironment.getApplication().filesDir, "../../../../../tests/fixtures")
            .canonicalFile
    }

    fun expectedGeometryDir(): File {
        val prop = System.getProperty("expected.geometry")
        if (!prop.isNullOrBlank()) return File(prop)
        val fromCwd = File(System.getProperty("user.dir"), "../tests/expected/geometry").canonicalFile
        if (fromCwd.isDirectory || fromCwd.parentFile?.isDirectory == true) return fromCwd
        return File(fixturesRoot().parentFile, "expected/geometry")
    }

    fun expectedPixelsDir(): File {
        val prop = System.getProperty("expected.pixels")
        if (!prop.isNullOrBlank()) return File(prop)
        return File(expectedGeometryDir().parentFile, "pixels/android")
    }

    fun updateSnapshots(): Boolean =
        System.getProperty("update.snapshots", "false").equals("true", ignoreCase = true) ||
            System.getenv("UPDATE_SNAPSHOTS") == "1"
}
