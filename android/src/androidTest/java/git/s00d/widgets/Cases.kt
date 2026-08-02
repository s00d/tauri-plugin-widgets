package git.s00d.widgets

import android.content.Context
import org.json.JSONObject
import java.io.File

data class VisualCase(
    val name: String,
    val fixture: String,
    val size: String,
    val theme: String,
    val locale: String,
) {
    fun dp(): Pair<Int, Int> = when (size) {
        "medium" -> 360 to 170
        "large" -> 360 to 380
        else -> 170 to 170
    }
}

object Cases {
    fun load(ctx: Context, filter: String? = null): List<VisualCase> {
        val am = ctx.assets
        val names = am.list("cases")?.filter { it.endsWith(".json") }?.sorted().orEmpty()
        require(names.isNotEmpty()) {
            "no cases in assets/cases — run syncVisualTestAssets / rebuild androidTest"
        }
        val out = mutableListOf<VisualCase>()
        for (file in names) {
            val name = file.removeSuffix(".json")
            if (!filter.isNullOrBlank() && name != filter) continue
            val text = am.open("cases/$file").bufferedReader().use { it.readText() }
            val json = JSONObject(text)
            out += VisualCase(
                name = name,
                fixture = json.getString("fixture"),
                size = json.getString("size"),
                theme = json.optString("theme", "dark"),
                locale = json.optString("locale", "en_US"),
            )
        }
        return out
    }

    fun fixtureJson(ctx: Context, case: VisualCase): String {
        val path = "fixtures/${case.fixture}.json"
        return ctx.assets.open(path).bufferedReader().use { it.readText() }
    }
}

object VisualStore {
    const val WIDGET_ID = "visual"

    fun group(ctx: Context): String = ctx.packageName

    fun reset(ctx: Context) {
        val g = group(ctx)
        ctx.getSharedPreferences(g, Context.MODE_PRIVATE).edit().clear().apply()
        WidgetStoreKeys.metaPrefs(ctx).edit().clear().apply()
    }

    fun putConfig(ctx: Context, appWidgetId: Int, configJson: String) {
        val g = group(ctx)
        ctx.getSharedPreferences(g, Context.MODE_PRIVATE)
            .edit()
            .putString(WidgetStoreKeys.configKey(WIDGET_ID), configJson)
            .putString(WidgetStoreKeys.META_NONCE, "1")
            .putString(WidgetStoreKeys.META_UPDATED_AT, System.currentTimeMillis().toString())
            .apply()
        WidgetStoreKeys.metaPrefs(ctx)
            .edit()
            .putString(WidgetStoreKeys.KEY_ACTIVE_GROUP, g)
            .putString(WidgetStoreKeys.KEY_ACTIVE_WIDGET_ID, WIDGET_ID)
            .apply()
        WidgetStoreKeys.bindInstance(ctx, appWidgetId, WIDGET_ID, g)
    }
}

object Golden {
    const val TOLERANCE = 0.02

    fun recordMode(_ctx: Context): Boolean {
        val args = androidx.test.platform.app.InstrumentationRegistry.getArguments()
        val fromArgs = args.getString("golden.record")
        if (fromArgs.equals("true", true) || fromArgs == "1") return true
        return System.getProperty("golden.record", "false").equals("true", true)
    }

    fun caseFilter(): String? {
        val args = androidx.test.platform.app.InstrumentationRegistry.getArguments()
        return args.getString("case")?.takeIf { it.isNotBlank() }
    }

    fun assertMatches(ctx: Context, caseName: String, actual: android.graphics.Bitmap) {
        val am = ctx.assets
        val goldenAsset = "golden/$caseName.png"
        val record = recordMode(ctx)
        val outDir = File(ctx.getExternalFilesDir(null), "widgets-out").apply { mkdirs() }
        val actualFile = File(outDir, "$caseName.actual.png")
        actualFile.outputStream().use { actual.compress(android.graphics.Bitmap.CompressFormat.PNG, 100, it) }

        if (record) {
            val recordDir = File(ctx.getExternalFilesDir(null), "widgets-golden").apply { mkdirs() }
            val dest = File(recordDir, "$caseName.png")
            actualFile.copyTo(dest, overwrite = true)
            // Also try writing golden into assets is impossible; adb pull documented.
            return
        }

        val expected = try {
            am.open(goldenAsset).use { android.graphics.BitmapFactory.decodeStream(it) }
        } catch (e: Exception) {
            throw AssertionError(
                "missing golden assets/$goldenAsset — record with -Pgolden.record=true " +
                    "and adb pull ${File(ctx.getExternalFilesDir(null), "widgets-golden")}",
                e,
            )
        }

        val frac = BitmapUtil.mismatchFraction(expected, actual)
        if (frac > TOLERANCE) {
            val diff = BitmapUtil.diffBitmap(expected, actual)
            File(outDir, "$caseName.diff.png").outputStream().use {
                diff.compress(android.graphics.Bitmap.CompressFormat.PNG, 100, it)
            }
            throw AssertionError(
                "golden mismatch $caseName: ${(frac * 100).format(2)}% pixels differ " +
                    "(tol ${TOLERANCE * 100}%). actual/diff → $outDir",
            )
        }
    }

    private fun Double.format(digits: Int) = "%.${digits}f".format(this)
}
