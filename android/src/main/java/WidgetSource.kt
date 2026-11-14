package git.s00d.widgets

import android.content.Context
import android.util.Log
import androidx.glance.appwidget.updateAll
import androidx.work.BackoffPolicy
import androidx.work.Constraints
import androidx.work.CoroutineWorker
import androidx.work.ExistingWorkPolicy
import androidx.work.NetworkType
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.WorkManager
import androidx.work.WorkerParameters
import androidx.work.workDataOf
import java.util.concurrent.TimeUnit
import org.json.JSONObject
import java.net.HttpURLConnection
import java.net.URL

/**
 * `source` fetch for a placement: at most once per `refreshMinutes`, response cached in
 * its own prefs file (never in the shared config map, so a fetch cannot overwrite a newer
 * config the app wrote). The request runs in WorkManager ([WidgetSourceWorker]: network
 * constraint, exponential backoff), never inside a render.
 */
internal object WidgetSource {
    private const val TIMEOUT_MS = 8_000
    /** Retries (with backoff) before an attempt counts as failed until the next refresh. */
    internal const val MAX_RETRIES = 3

    /** Cached response plus the last failed attempt. */
    internal data class Cached(val data: Any?, val ts: Long?, val failedAt: Long?) {
        /** Last attempt, ok or failed: failures back off by `refreshMinutes` too (no retry storm). */
        val attempt: Long? get() = listOfNotNull(ts, failedAt).maxOrNull()
    }

    internal fun isDue(attempt: Long?, now: Long, refreshMinutes: Int): Boolean =
        attempt == null || now - attempt >= refreshMinutes * 60_000L

    internal fun workName(group: String, scopedId: String) = "tauri-widget-source:$group:$scopedId"

    /**
     * Cached data and the last attempt time; when a fetch is due, queue it (unique per
     * placement, so redraws never stack requests). The worker redraws widgets when done.
     */
    fun enqueueIfDue(context: Context, configRaw: String?, group: String, widgetId: String, instance: String?): Pair<Any?, Long?> {
        val scopedId = WidgetStoreKeys.scopedId(widgetId, instance)
        val cache = read(context, scopedId)
        val cached = cache.data to cache.attempt
        val config = Resolve.json(configRaw) as? JSONObject ?: return cached
        val req = Resolve.sourceRequest(
            config,
            Resolve.Context(System.currentTimeMillis(), currentLocaleTag(context), instance),
        ) ?: return cached
        if (!isDue(cache.attempt, System.currentTimeMillis(), req.refreshMinutes)) return cached
        val work = OneTimeWorkRequestBuilder<WidgetSourceWorker>()
            .setConstraints(Constraints.Builder().setRequiredNetworkType(NetworkType.CONNECTED).build())
            .setBackoffCriteria(BackoffPolicy.EXPONENTIAL, 30, TimeUnit.SECONDS)
            .setInputData(
                workDataOf("group" to group, "widgetId" to widgetId, "instance" to instance),
            )
            .build()
        runCatching {
            WorkManager.getInstance(context).enqueueUniqueWork(workName(group, scopedId), ExistingWorkPolicy.KEEP, work)
        }.onFailure { Log.w(TAG, "source fetch not queued for $scopedId: ${it.message}") }
        return cached
    }

    fun get(req: Resolve.SourceRequest): String? {
        val conn = URL(req.url).openConnection() as HttpURLConnection
        try {
            conn.connectTimeout = TIMEOUT_MS
            conn.readTimeout = TIMEOUT_MS
            conn.requestMethod = req.method
            req.headers.forEach { (k, v) -> conn.setRequestProperty(k, v) }
            if (req.body != null) {
                conn.doOutput = true
                conn.outputStream.use { it.write(req.body.toByteArray()) }
            }
            if (conn.responseCode !in 200..299) return null
            val bytes = conn.inputStream.use { input ->
                val buf = java.io.ByteArrayOutputStream()
                val chunk = ByteArray(8192)
                while (true) {
                    val n = input.read(chunk)
                    if (n < 0) break
                    buf.write(chunk, 0, n)
                    if (buf.size() > Resolve.MAX_SOURCE_BYTES) return null
                }
                buf.toByteArray()
            }
            return String(bytes, Charsets.UTF_8)
        } finally {
            conn.disconnect()
        }
    }

    fun read(context: Context, scopedId: String): Cached {
        val raw = prefs(context).getString(scopedId, null) ?: return Cached(null, null, null)
        val obj = runCatching { JSONObject(raw) }.getOrNull() ?: return Cached(null, null, null)
        return Cached(
            Resolve.json(obj.optString("raw")),
            if (obj.has("ts")) obj.optLong("ts") else null,
            if (obj.has("failedAt")) obj.optLong("failedAt") else null,
        )
    }

    fun write(context: Context, scopedId: String, text: String, ts: Long) {
        prefs(context).edit().putString(scopedId, JSONObject().put("raw", text).put("ts", ts).toString()).apply()
    }

    /** Keep the last good response; only stamp the failure. */
    fun recordFailure(context: Context, scopedId: String, at: Long) {
        val p = prefs(context)
        val obj = p.getString(scopedId, null)?.let { runCatching { JSONObject(it) }.getOrNull() } ?: JSONObject()
        p.edit().putString(scopedId, obj.put("failedAt", at).toString()).apply()
    }

    private fun prefs(context: Context) =
        context.getSharedPreferences(WidgetStoreKeys.FETCHED_PREFS, Context.MODE_PRIVATE)
}

/** One `source` request for one placement; redraws Tauri widgets after a new response. */
class WidgetSourceWorker(context: Context, params: WorkerParameters) : CoroutineWorker(context, params) {
    override suspend fun doWork(): Result {
        val group = inputData.getString("group") ?: return Result.failure()
        val widgetId = inputData.getString("widgetId") ?: return Result.failure()
        val instance = inputData.getString("instance")
        val ctx = applicationContext
        val scopedId = WidgetStoreKeys.scopedId(widgetId, instance)
        val prefs = ctx.getSharedPreferences(group, Context.MODE_PRIVATE)
        val configRaw = WidgetStoreKeys.lookupKeys(WidgetStoreKeys.CONFIG_PREFIX, widgetId, instance)
            .firstNotNullOfOrNull { prefs.getString(it, null) }
        val config = Resolve.json(configRaw) as? JSONObject ?: return Result.success()
        val req = Resolve.sourceRequest(config, Resolve.Context(System.currentTimeMillis(), currentLocaleTag(ctx), instance))
            ?: return Result.success()
        val text = runCatching { WidgetSource.get(req) }.getOrNull()
        if (text == null || Resolve.json(text) == null) {
            if (runAttemptCount < WidgetSource.MAX_RETRIES) return Result.retry()
            Log.w(TAG, "source fetch failed for $scopedId; keeping cache")
            WidgetSource.recordFailure(ctx, scopedId, System.currentTimeMillis())
            return Result.failure()
        }
        WidgetSource.write(ctx, scopedId, text, System.currentTimeMillis())
        runCatching { TauriGlanceWidget().updateAll(ctx) }
        return Result.success()
    }
}
