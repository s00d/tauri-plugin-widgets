package git.s00d.widgets

import android.Manifest
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Build
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import org.json.JSONObject

/**
 * Live activities on Android: an ongoing notification (title, body, progress, countdown
 * chip, up to three actions). Android 16 promotes it to a Live Update when the user allows
 * it (`android.requestPromotedOngoing`). Taps on actions are queued like widget taps with
 * `widgetId = "activity.<id>"`.
 */
object ActivityNotifications {
    const val CHANNEL_ID = "tauri_widget_activities"
    private const val PREFS = "__tauri_widget_activities__"
    private const val KEY_NEXT_ID = "__next_id__"
    /** Notification ids used by this plugin start here (stays clear of small app ids). */
    private const val ID_BASE = 0x7A110000
    const val EXTRA_ID = "git.s00d.widgets.activity.id"
    const val EXTRA_ACTION = "git.s00d.widgets.activity.action"
    const val EXTRA_GROUP = "git.s00d.widgets.activity.group"

    private fun prefs(c: Context) = c.getSharedPreferences(PREFS, Context.MODE_PRIVATE)

    /** Stable notification id per activity id; stored, so ids never collide. */
    fun notificationId(c: Context, id: String): Int {
        val p = prefs(c)
        val existing = p.getInt("id:$id", -1)
        if (existing >= 0) return existing
        val next = p.getInt(KEY_NEXT_ID, 0)
        val nid = ID_BASE + next
        p.edit().putInt("id:$id", nid).putInt(KEY_NEXT_ID, (next + 1) % 0xFFFF).commit()
        return nid
    }

    fun canPost(c: Context): Boolean =
        Build.VERSION.SDK_INT < 33 ||
            c.checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS) == PackageManager.PERMISSION_GRANTED

    private fun ensureChannel(c: Context) {
        if (Build.VERSION.SDK_INT < 26) return
        val nm = c.getSystemService(NotificationManager::class.java) ?: return
        if (nm.getNotificationChannel(CHANNEL_ID) != null) return
        val label = runCatching { c.applicationInfo.loadLabel(c.packageManager).toString() }.getOrDefault("Activities")
        nm.createNotificationChannel(
            NotificationChannel(CHANNEL_ID, label, NotificationManager.IMPORTANCE_DEFAULT).apply {
                setSound(null, null)
                enableVibration(false)
            }
        )
    }

    /**
     * Status-bar icon must be a monochrome drawable: the launcher icon (often adaptive)
     * renders as a white square and can crash on API 26. Apps ship `tauri_activity_icon`.
     */
    @android.annotation.SuppressLint("DiscouragedApi") // name lookup: the resource lives in the app, not here
    private fun smallIcon(c: Context): Int =
        c.resources.getIdentifier("tauri_activity_icon", "drawable", c.packageName)
            .takeIf { it != 0 } ?: android.R.drawable.stat_notify_more

    /**
     * Contract v3 §3: `template` names a layout stored under `activity-template:{name}`; its
     * strings are resolved with `data` (`{{data.x}}`, `when`, `repeat`, …) and fields set
     * directly on the content win. Unknown template: the content as sent.
     */
    internal fun expandTemplate(c: Context, content: JSONObject, group: String): JSONObject {
        val name = content.opt("template") as? String ?: return content
        val prefs = c.getSharedPreferences(WidgetSanitizer.sanitizeGroup(group, c.packageName), Context.MODE_PRIVATE)
        val raw = prefs.getString(WidgetSanitizer.sanitizeKey(WidgetStoreKeys.ACTIVITY_TEMPLATE_PREFIX + name), null)
        val tpl = Resolve.json(raw) as? JSONObject ?: return content
        val ctx = Resolve.Context(System.currentTimeMillis(), currentLocaleTag(c), data = content.opt("data"))
        val out = Resolve.resolveValue(tpl, ctx) as JSONObject
        content.keys().forEach { k ->
            val v = content.opt(k)
            // Like Rust `resolve_activity`: a blank `title` takes the template's.
            val blankTitle = k == "title" && (v as? String)?.isBlank() == true
            if (k != "template" && k != "data" && !blankTitle && v != JSONObject.NULL) out.put(k, v)
        }
        return out
    }

    /** Android 16: the user lets this app post promoted (Live Update) notifications. */
    fun canPostPromoted(c: Context): Boolean {
        if (Build.VERSION.SDK_INT < 36) return false
        val nm = c.getSystemService(NotificationManager::class.java) ?: return false
        return runCatching { nm.javaClass.getMethod("canPostPromotedNotifications").invoke(nm) as Boolean }.getOrDefault(false)
    }

    /** Settings page for Live Updates (API 36), else the app's notification settings. */
    fun promotionSettingsIntent(c: Context): Intent = when {
        Build.VERSION.SDK_INT >= 36 -> Intent("android.settings.APP_NOTIFICATION_PROMOTION_SETTINGS")
            .putExtra(android.provider.Settings.EXTRA_APP_PACKAGE, c.packageName)
        Build.VERSION.SDK_INT >= 26 -> Intent(android.provider.Settings.ACTION_APP_NOTIFICATION_SETTINGS)
            .putExtra(android.provider.Settings.EXTRA_APP_PACKAGE, c.packageName)
        else -> Intent(android.provider.Settings.ACTION_APPLICATION_DETAILS_SETTINGS, Uri.parse("package:${c.packageName}"))
    }

    /** ProgressStyle plan: segments (length, argb) and points (position, argb) on a 0…total track. */
    internal data class ProgressPlan(val progress: Int, val segments: List<Pair<Int, Int?>>, val points: List<Pair<Int, Int?>>)

    private fun argb(hex: Any?): Int? = (hex as? String)?.let { runCatching { android.graphics.Color.parseColor(expandShortHex(it)) }.getOrNull() }

    /**
     * `progress` (0…1) on one `tint` segment; optional `progressSegments: [{length, color}]`
     * and `progressPoints: [{at: 0…1, color}]` replace / mark the track. Null without progress.
     */
    internal fun progressPlan(content: JSONObject): ProgressPlan? {
        val segs = content.optJSONArray("progressSegments")
        if ((!content.has("progress") || content.isNull("progress")) && segs == null) return null
        val segments = (0 until (segs?.length() ?: 0)).mapNotNull { i ->
            val o = segs!!.optJSONObject(i) ?: return@mapNotNull null
            o.optInt("length", 0).takeIf { it > 0 }?.let { it to argb(o.opt("color")) }
        }.ifEmpty { listOf(1000 to argb(content.opt("tint"))) }
        val total = segments.sumOf { it.first }
        val pts = content.optJSONArray("progressPoints")
        val points = (0 until (pts?.length() ?: 0)).mapNotNull { i ->
            val o = pts!!.optJSONObject(i) ?: return@mapNotNull null
            (o.optDouble("at", 0.0).coerceIn(0.0, 1.0) * total).toInt() to argb(o.opt("color"))
        }
        val progress = (content.optDouble("progress", 0.0).coerceIn(0.0, 1.0) * total).toInt()
        return ProgressPlan(progress, segments, points)
    }

    /**
     * API 36 Live Update: rebuild the compat notification with the framework builder
     * (`setRequestPromotedOngoing`, `setShortCriticalText`, `ProgressStyle`). Reflection,
     * because the plugin compiles against an older SDK. Null when unavailable.
     */
    private fun promoted(c: Context, n: android.app.Notification, content: JSONObject): android.app.Notification? {
        if (Build.VERSION.SDK_INT < 36) return null
        return runCatching {
            val fb = android.app.Notification.Builder.recoverBuilder(c, n)
            val cls = android.app.Notification.Builder::class.java
            cls.getMethod("setRequestPromotedOngoing", Boolean::class.javaPrimitiveType).invoke(fb, true)
            content.optString("shortCriticalText").takeIf { it.isNotEmpty() }?.let {
                cls.getMethod("setShortCriticalText", String::class.java).invoke(fb, it)
            }
            progressPlan(content)?.let { plan ->
                val style = Class.forName("android.app.Notification\$ProgressStyle")
                val seg = Class.forName("android.app.Notification\$ProgressStyle\$Segment")
                val pt = Class.forName("android.app.Notification\$ProgressStyle\$Point")
                val ps = style.getConstructor().newInstance()
                val segments = plan.segments.map { (len, color) ->
                    seg.getConstructor(Int::class.javaPrimitiveType).newInstance(len).also { s ->
                        color?.let { seg.getMethod("setColor", Int::class.javaPrimitiveType).invoke(s, it) }
                    }
                }
                val points = plan.points.map { (at, color) ->
                    pt.getConstructor(Int::class.javaPrimitiveType).newInstance(at).also { p ->
                        color?.let { pt.getMethod("setColor", Int::class.javaPrimitiveType).invoke(p, it) }
                    }
                }
                style.getMethod("setProgressSegments", List::class.java).invoke(ps, segments)
                if (points.isNotEmpty()) style.getMethod("setProgressPoints", List::class.java).invoke(ps, points)
                style.getMethod("setProgress", Int::class.javaPrimitiveType).invoke(ps, plan.progress)
                fb.setStyle(ps as android.app.Notification.Style)
            }
            fb.build()
        }.onFailure { android.util.Log.w(TAG, "live update promotion unavailable: ${it.message}") }.getOrNull()
    }

    /**
     * Out-of-order guard (as iOS): an update of a running activity whose `timestamp` is older
     * than the last applied one is dropped. A fresh start always applies.
     */
    internal fun shouldApply(active: Boolean, last: Long?, ts: Long?): Boolean =
        !active || ts == null || last == null || ts >= last

    /**
     * Post or replace the notification for this request. Throws when it cannot be shown;
     * false when dropped as stale (`timestamp` older than the last applied one).
     */
    fun post(c: Context, request: JSONObject, group: String): Boolean {
        val content = expandTemplate(c, request, group)
        val id = content.optString("id")
        require(id.isNotBlank()) { "activity id must not be empty" }
        check(canPost(c)) {
            "POST_NOTIFICATIONS is not granted; ask for it first (e.g. @tauri-apps/plugin-notification requestPermission())"
        }
        val ts = content.optLong("timestamp", -1L).takeIf { content.has("timestamp") && !content.isNull("timestamp") && it >= 0 }
        val last = prefs(c).getLong("ts:$id", -1L).takeIf { it >= 0 }
        val wasActive = isActive(c, id)
        if (!shouldApply(wasActive, last, ts)) {
            android.util.Log.i(TAG, "activity $id: stale update dropped (timestamp $ts < $last)")
            return false
        }
        ensureChannel(c)
        val nid = notificationId(c, id)
        val b = NotificationCompat.Builder(c, CHANNEL_ID)
            .setSmallIcon(smallIcon(c))
            .setContentTitle(content.optString("title"))
            .setOngoing(true)
            .setOnlyAlertOnce(true)
            .setSilent(true)
            .setCategory(category(content.optString("category")))
            .setVisibility(visibility(content.optString("visibility")))
        content.optString("body").takeIf { it.isNotEmpty() }?.let { b.setContentText(it) }
        content.optString("subText").takeIf { it.isNotEmpty() }?.let { b.setSubText(it) }
        // Android 16 promoted chip text: the extra here (older SDKs / fallback); on API 36
        // [promoted] also calls `setShortCriticalText`.
        content.optString("shortCriticalText").takeIf { it.isNotEmpty() }?.let {
            b.extras.putString(EXTRA_SHORT_CRITICAL_TEXT, it)
        }
        content.optString("tint").takeIf { it.isNotEmpty() }?.let { hex ->
            runCatching { android.graphics.Color.parseColor(hex) }.getOrNull()?.let { b.setColor(it) }
        }
        if (content.has("progress") && !content.isNull("progress")) {
            val p = (content.optDouble("progress", 0.0).coerceIn(0.0, 1.0) * 1000).toInt()
            b.setProgress(1000, p, false)
        }
        if (content.has("endsAt") && !content.isNull("endsAt")) {
            b.setWhen(content.optLong("endsAt")).setShowWhen(true).setUsesChronometer(true)
            if (Build.VERSION.SDK_INT >= 24) b.setChronometerCountDown(true)
        } else if (content.has("startedAt") && !content.isNull("startedAt")) {
            // Stopwatch counting up from startedAt.
            b.setWhen(content.optLong("startedAt")).setShowWhen(true).setUsesChronometer(true)
        }
        b.setContentIntent(openApp(c, content.optString("url").takeIf { it.isNotEmpty() }, nid))
        val actions = content.optJSONArray("actions")
        if (actions != null) {
            for (i in 0 until minOf(actions.length(), 3)) {
                val a = actions.optJSONObject(i) ?: continue
                val url = a.optString("url").takeIf { it.isNotEmpty() && it != "null" }
                // `url`: open the deep link instead of queuing a widget-action.
                val pi = if (url != null) {
                    openApp(c, url, nid * 4 + i)
                } else {
                    val intent = Intent(c, ActivityActionReceiver::class.java)
                        .putExtra(EXTRA_ID, id)
                        .putExtra(EXTRA_ACTION, a.optString("id"))
                        .putExtra(EXTRA_GROUP, group)
                    PendingIntent.getBroadcast(
                        c, nid * 4 + i, intent, PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
                    )
                }
                val icon = a.optString("systemName").takeIf { it.isNotEmpty() && it != "null" }
                    ?.let { actionIcon(c, it) } ?: 0
                b.addAction(icon, a.optString("label"), pi)
            }
        }
        // Android 16 Live Updates: ask for promotion; the system decides with the user setting.
        b.extras.putBoolean("android.requestPromotedOngoing", true)
        val vis = content.optString("visibility")
        // Lock screen: `private` shows a redacted copy (app name, timer, progress; no text,
        // no actions); `secret` hides it (no public version needed).
        if (vis == "private") b.setPublicVersion(publicVersion(c, content))
        if (isSamsung() && Build.VERSION.SDK_INT < 36) {
            samsungExtras(content, appLabel(c), android.graphics.drawable.Icon.createWithResource(c, smallIcon(c)))
                ?.let { b.addExtras(it) }
        }
        val built = b.build()
        NotificationManagerCompat.from(c).notify(nid, promoted(c, built, content) ?: built)
        prefs(c).edit().putString("active:$id", group).apply {
            // No timestamp: keep the last one (iOS does too); a fresh start forgets an old one.
            if (ts != null) putLong("ts:$id", ts) else if (!wasActive) remove("ts:$id")
        }.apply()
        return true
    }

    private fun appLabel(c: Context): String =
        runCatching { c.applicationInfo.loadLabel(c.packageManager).toString() }.getOrDefault("")

    /** Redacted lock-screen copy for `visibility: private`: app name, timer and progress only. */
    internal fun publicVersion(c: Context, content: JSONObject): android.app.Notification {
        val p = NotificationCompat.Builder(c, CHANNEL_ID)
            .setSmallIcon(smallIcon(c))
            .setContentTitle(appLabel(c))
            .setOngoing(true)
            .setSilent(true)
            .setCategory(category(content.optString("category")))
        argb(content.opt("tint"))?.let { p.setColor(it) }
        if (content.has("progress") && !content.isNull("progress")) {
            p.setProgress(1000, (content.optDouble("progress", 0.0).coerceIn(0.0, 1.0) * 1000).toInt(), false)
        }
        if (content.has("endsAt") && !content.isNull("endsAt")) {
            p.setWhen(content.optLong("endsAt")).setShowWhen(true).setUsesChronometer(true)
            if (Build.VERSION.SDK_INT >= 24) p.setChronometerCountDown(true)
        } else if (content.has("startedAt") && !content.isNull("startedAt")) {
            p.setWhen(content.optLong("startedAt")).setShowWhen(true).setUsesChronometer(true)
        }
        return p.build()
    }

    internal fun isSamsung(): Boolean = Build.MANUFACTURER.equals("samsung", ignoreCase = true)

    /**
     * Samsung One UI 7 Live Notifications / Now Bar (`android.ongoingActivityNoti.*`, read
     * with the `com.samsung.android.support.ongoing_activity` manifest flag; Samsung shows
     * them only for allow-listed apps). Applied below Android 16 only: on One UI 8+ the
     * Now Bar takes AOSP Live Updates, and `style >= 1` would move the notification off that
     * path. `secret` gets none (the Now Bar is on the lock screen); `private` shows the app
     * name there. Null when there is nothing to show.
     */
    internal fun samsungExtras(content: JSONObject, appLabel: String, chipIcon: android.graphics.drawable.Icon?): android.os.Bundle? {
        val vis = content.optString("visibility")
        val title = content.optString("title")
        if (vis == "secret" || title.isBlank()) return null
        val body = content.optString("body").takeIf { it.isNotEmpty() && it != "null" }
        val chip = content.optString("shortCriticalText").takeIf { it.isNotEmpty() && it != "null" } ?: title
        return android.os.Bundle().apply {
            putInt(SAMSUNG + "style", 1)
            putString(SAMSUNG + "primaryInfo", title)
            body?.let { putString(SAMSUNG + "secondaryInfo", it) }
            putString(SAMSUNG + "chipExpandedText", chip)
            chipIcon?.let { putParcelable(SAMSUNG + "chipIcon", it) }
            argb(content.opt("tint"))?.let {
                putInt(SAMSUNG + "chipBgColor", it)
                putInt(SAMSUNG + "progressSegments.progressColor", it)
            }
            if (content.has("progress") && !content.isNull("progress")) {
                putInt(SAMSUNG + "progress", (content.optDouble("progress", 0.0).coerceIn(0.0, 1.0) * 100).toInt())
                putInt(SAMSUNG + "progressMax", 100)
            }
            if ((content.optJSONArray("actions")?.length() ?: 0) > 0) {
                putInt(SAMSUNG + "actionType", 1)
                putInt(SAMSUNG + "actionPrimarySet", 0)
            }
            if (vis == "private") {
                putString(SAMSUNG + "nowbarPrimaryInfo", appLabel)
                putString(SAMSUNG + "nowbarSecondaryInfo", "")
            }
        }
    }

    internal const val SAMSUNG = "android.ongoingActivityNoti."

    const val EXTRA_SHORT_CRITICAL_TEXT = "android.shortCriticalText"

    private val CATEGORIES = setOf("progress", "navigation", "transport", "workout", "status", "alarm", "service", "event")

    /** IR categories are the framework's `Notification.CATEGORY_*` strings. */
    internal fun category(raw: String): String = raw.takeIf { it in CATEGORIES } ?: NotificationCompat.CATEGORY_PROGRESS

    internal fun visibility(raw: String): Int = when (raw) {
        "private" -> NotificationCompat.VISIBILITY_PRIVATE
        "secret" -> NotificationCompat.VISIBILITY_SECRET
        else -> NotificationCompat.VISIBILITY_PUBLIC
    }

    /** App / framework drawable named like the symbol, else a public framework media icon, else 0. */
    internal fun actionIcon(c: Context, systemName: String): Int =
        systemIconResId(c, systemName).takeIf { it != 0 } ?: when (systemName.removeSuffix(".fill")) {
            "play" -> android.R.drawable.ic_media_play
            "pause" -> android.R.drawable.ic_media_pause
            "forward" -> android.R.drawable.ic_media_ff
            "backward" -> android.R.drawable.ic_media_rew
            "xmark" -> android.R.drawable.ic_menu_close_clear_cancel
            else -> 0
        }

    private fun openApp(c: Context, url: String?, requestCode: Int): PendingIntent? {
        val launch = c.packageManager.getLaunchIntentForPackage(c.packageName)
            // No launcher activity: a deep link still opens through the app's VIEW filter.
            ?: url?.let { Intent(Intent.ACTION_VIEW).setPackage(c.packageName).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK) }
            ?: return null
        if (url != null) {
            launch.action = Intent.ACTION_VIEW
            launch.data = Uri.parse(url)
        }
        return PendingIntent.getActivity(c, requestCode, launch, PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
    }

    fun isActive(c: Context, id: String): Boolean = prefs(c).contains("active:$id")

    fun end(c: Context, id: String): Boolean {
        if (!isActive(c, id)) return false
        NotificationManagerCompat.from(c).cancel(notificationId(c, id))
        prefs(c).edit().remove("active:$id").remove("ts:$id").apply()
        return true
    }

    /** Ids still shown (the user may have dismissed some on Android 14+). */
    fun list(c: Context): List<String> {
        val ids = prefs(c).all.keys.filter { it.startsWith("active:") }.map { it.removePrefix("active:") }
        val nm = c.getSystemService(NotificationManager::class.java) ?: return ids.sorted()
        val shown = nm.activeNotifications.map { it.id }.toSet()
        val (alive, gone) = ids.partition { notificationId(c, it) in shown }
        if (gone.isNotEmpty()) prefs(c).edit().apply { gone.forEach { remove("active:$it"); remove("ts:$it") } }.apply()
        return alive.sorted()
    }
}

/** Action button tap: queue it for the app (delivered as `widget-action`). */
class ActivityActionReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        val id = intent.getStringExtra(ActivityNotifications.EXTRA_ID) ?: return
        val action = intent.getStringExtra(ActivityNotifications.EXTRA_ACTION) ?: return
        val group = intent.getStringExtra(ActivityNotifications.EXTRA_GROUP) ?: return
        WidgetStoreKeys.enqueueAction(context, WidgetSanitizer.sanitizeGroup(group, context.packageName), action, null, "activity.$id")
        WidgetBridgePlugin.pluginInstance?.takeIf { it.isAlive() }?.notifyPendingActions(group)
    }
}
