package git.s00d.widgets

import android.app.Activity
import android.appwidget.AppWidgetManager
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.widget.ArrayAdapter
import android.widget.ListView
import androidx.core.content.FileProvider
import androidx.glance.appwidget.GlanceAppWidgetManager
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import org.json.JSONArray
import java.io.File

/**
 * Widget configuration (`android:configure`, `widgetFeatures="reconfigurable|configuration_optional"`).
 * A trampoline: with no `instances:{widgetId}` choices (`setWidgetInstances`) it finishes at
 * once; otherwise the user picks one and that becomes the placement's instance id (keys
 * `config:{id}#{instance}`, `data:…`, `state:…`, `{{instance}}`).
 */
class WidgetConfigActivity : Activity() {
    private var appWidgetId = AppWidgetManager.INVALID_APPWIDGET_ID

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        appWidgetId = intent?.extras?.getInt(AppWidgetManager.EXTRA_APPWIDGET_ID, AppWidgetManager.INVALID_APPWIDGET_ID)
            ?: AppWidgetManager.INVALID_APPWIDGET_ID
        // Back / dismiss = cancel (the launcher drops a new placement).
        setResult(RESULT_CANCELED, resultIntent())
        if (appWidgetId == AppWidgetManager.INVALID_APPWIDGET_ID) return finish()
        val options = options(this, appWidgetId)
        if (options.isEmpty()) return finishWidgetConfiguration(null)
        setTitle(R.string.tauri_widget_configure_title)
        val list = ListView(this)
        list.adapter = ArrayAdapter(this, android.R.layout.simple_list_item_1, options.map { it.second })
        list.setOnItemClickListener { _, _, pos, _ -> finishWidgetConfiguration(options[pos].first) }
        setContentView(list)
    }

    private fun resultIntent() = Intent().putExtra(AppWidgetManager.EXTRA_APPWIDGET_ID, appWidgetId)

    /** Store the choice (null keeps the default), redraw the placement, report RESULT_OK. */
    private fun finishWidgetConfiguration(instance: String?) {
        if (instance != null) WidgetStoreKeys.setPlacementInstance(this, appWidgetId, instance)
        val app = applicationContext
        val id = appWidgetId
        CoroutineScope(Dispatchers.Default).launch {
            runCatching { TauriGlanceWidget().update(app, GlanceAppWidgetManager(app).getGlanceIdBy(id)) }
        }
        setResult(RESULT_OK, resultIntent())
        finish()
    }

    companion object {
        /** `(id, title)` choices for the widget this placement shows. */
        fun options(context: Context, appWidgetId: Int): List<Pair<String, String>> {
            val group = WidgetStoreKeys.resolveGroup(context, appWidgetId)
            val widgetId = WidgetStoreKeys.resolveWidgetId(context, appWidgetId)
            val raw = context.getSharedPreferences(group, Context.MODE_PRIVATE)
                .getString(WidgetSanitizer.sanitizeKey(WidgetStoreKeys.INSTANCES_PREFIX + widgetId), null)
            return parseOptions(raw)
        }

        fun parseOptions(raw: String?): List<Pair<String, String>> {
            val arr = runCatching { JSONArray(raw ?: return emptyList()) }.getOrNull() ?: return emptyList()
            return (0 until arr.length()).mapNotNull { i ->
                val o = arr.optJSONObject(i) ?: return@mapNotNull null
                val id = o.optString("id").takeIf { it.isNotEmpty() } ?: return@mapNotNull null
                id to o.optString("title").ifEmpty { id }
            }
        }
    }
}

/** Hands cached widget images to the launcher by content URI (no RemoteViews bitmap budget). */
class TauriWidgetFileProvider : FileProvider()

/**
 * Content URI for a file in the plugin image cache, readable by the home-screen app(s);
 * null when the file is elsewhere or the provider is missing (callers fall back to a bitmap).
 */
internal fun cachedImageUri(context: Context, path: String): Uri? = runCatching {
    val file = File(path)
    val dir = File(context.cacheDir, "tauri_widget_images")
    if (!file.isFile || file.canonicalFile.parentFile != dir.canonicalFile) return null
    val authority = context.packageManager
        .getProviderInfo(ComponentName(context, TauriWidgetFileProvider::class.java), 0).authority
    val uri = FileProvider.getUriForFile(context, authority, file)
    val home = Intent(Intent.ACTION_MAIN).addCategory(Intent.CATEGORY_HOME)
    @Suppress("DEPRECATION")
    context.packageManager.queryIntentActivities(home, 0).map { it.activityInfo.packageName }.distinct().forEach {
        context.grantUriPermission(it, uri, Intent.FLAG_GRANT_READ_URI_PERMISSION)
    }
    uri
}.getOrNull()
