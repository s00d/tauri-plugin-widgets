package git.s00d.widgets

import android.app.AlarmManager
import android.app.PendingIntent
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import androidx.glance.appwidget.updateAll
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import org.json.JSONObject

/**
 * Redraws widgets when the app is closed: at the next scheduled `entries[].at` and when a
 * `source` is due. One inexact alarm for all placements (the earliest wins; every redraw
 * schedules the next one), so no exact-alarm permission is needed.
 */
object WidgetScheduler {
    private const val PREFS = "__tauri_widget_schedule__"
    private const val KEY_NEXT = "next_at"

    /** Earliest time this config needs a redraw after `now`, or null. */
    fun nextWakeMs(config: JSONObject, fetchedTs: Long?, now: Long, refreshMinutes: Int?): Long? {
        val entry = Resolve.nextChangeMs(config, now)
        val source = refreshMinutes?.let { (fetchedTs ?: now) + it * 60_000L }?.let { if (it <= now) now + 60_000L else it }
        return listOfNotNull(entry, source).minOrNull()
    }

    fun scheduleFor(context: Context, configRaw: String?, fetchedTs: Long?, instance: String?) {
        val config = Resolve.json(configRaw) as? JSONObject ?: return
        val now = System.currentTimeMillis()
        val refresh = Resolve.sourceRequest(config, Resolve.Context(now, currentLocaleTag(context), instance))?.refreshMinutes
        val at = nextWakeMs(config, fetchedTs, now, refresh) ?: return
        schedule(context, at)
    }

    /** Keep the earliest pending wake-up; a later one never replaces an earlier one. */
    fun schedule(context: Context, atMs: Long) {
        val prefs = context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
        val pending = prefs.getLong(KEY_NEXT, 0L)
        val now = System.currentTimeMillis()
        val intent = Intent(context, WidgetTickReceiver::class.java)
        // A reboot / force-stop drops alarms but not prefs: trust next_at only while the alarm exists.
        val alive = PendingIntent.getBroadcast(
            context, 0, intent, PendingIntent.FLAG_NO_CREATE or PendingIntent.FLAG_IMMUTABLE,
        ) != null
        if (alive && pending in (now + 1) until atMs) return
        prefs.edit().putLong(KEY_NEXT, atMs).apply()
        val am = context.getSystemService(Context.ALARM_SERVICE) as? AlarmManager ?: return
        val pi = PendingIntent.getBroadcast(
            context, 0, intent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
        )
        am.setAndAllowWhileIdle(AlarmManager.RTC, atMs, pi) // minSdk 23
    }

    internal fun clear(context: Context) {
        context.getSharedPreferences(PREFS, Context.MODE_PRIVATE).edit().remove(KEY_NEXT).apply()
    }
}

/** Alarm target: redraw every Tauri widget (each redraw schedules the next alarm). */
class WidgetTickReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        val pending = goAsync()
        WidgetScheduler.clear(context)
        CoroutineScope(Dispatchers.Default).launch {
            try {
                TauriGlanceWidget().updateAll(context)
            } finally {
                pending.finish()
            }
        }
    }
}
