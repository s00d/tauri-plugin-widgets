package git.s00d.widgets

import android.content.Context
import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.Paint
import android.graphics.Typeface
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import java.util.Locale


internal fun iconGlyph(systemName: String): String {
    val key = systemName.lowercase(Locale.US)
    return when (key) {
        "star" -> "\u2B50"
        "star.fill" -> "\u2B50"
        "heart" -> "\u2764"
        "heart.fill" -> "\u2764"
        "person" -> "\uD83D\uDC64"
        "person.fill" -> "\uD83D\uDC64"
        "person.2.fill" -> "\uD83D\uDC65"
        "person.badge.plus" -> "\uD83D\uDC64"
        "gear" -> "\u2699"
        "gearshape" -> "\u2699"
        "gearshape.fill" -> "\u2699"
        "checkmark" -> "\u2713"
        "checkmark.circle" -> "\u2713"
        "checkmark.circle.fill" -> "\u2713"
        "xmark" -> "\u2715"
        "xmark.circle" -> "\u2715"
        "plus" -> "+"
        "plus.circle" -> "+"
        "minus" -> "\u2212"
        "minus.circle" -> "\u2212"
        "bell" -> "\uD83D\uDD14"
        "bell.fill" -> "\uD83D\uDD14"
        "house" -> "\u2302"
        "house.fill" -> "\u2302"
        "magnifyingglass" -> "\uD83D\uDD0D"
        "calendar" -> "\uD83D\uDCC5"
        "calendar.badge.clock" -> "\uD83D\uDCC5"
        "calendar.circle" -> "\uD83D\uDCC5"
        "clock" -> "\u23F1"
        "clock.fill" -> "\u23F1"
        "globe" -> "\uD83C\uDF10"
        "cloud.fill" -> "\u2601"
        "cloud.sun.fill" -> "\u26C5"
        "cloud.rain.fill" -> "\u2614"
        "cloud.bolt.fill" -> "\u26A1"
        "sun.max" -> "\u2600"
        "sun.max.fill" -> "\u2600"
        "moon.fill" -> "\u263E"
        "moon.stars.fill" -> "\u263E"
        "bolt" -> "\u26A1"
        "bolt.fill" -> "\u26A1"
        "location.fill" -> "\u2302"
        "flame.fill" -> "\uD83D\uDD25"
        "drop.fill" -> "\u25CF"
        "leaf.fill" -> "\u273F"
        "eye.fill" -> "\u25C9"
        "bubble.left.fill" -> "\u25D1"
        "music.note" -> "\u266A"
        "music.note.list" -> "\u266A"
        "hourglass" -> "\u23F3"
        "hourglass.bottomhalf.filled" -> "\u23F3"
        "figure.run" -> "\u27A4"
        "paintpalette.fill" -> "\u2698"
        "quote.opening" -> "\u201C"
        "bitcoinsign.circle.fill" -> "\u20BF"
        "battery.100" -> "\u25AE"
        "iphone" -> "\u25AF"
        "ipad" -> "\u25AD"
        "applewatch" -> "\u231A"
        "airpodspro" -> "\u25CB"
        "desktopcomputer" -> "\u25A3"
        "internaldrive" -> "\u25A0"
        "network" -> "\u2261"
        "envelope.fill" -> "\u2709"
        "envelope" -> "\u2709"
        "phone.fill" -> "\u260E"
        "phone" -> "\u260E"
        "message.fill" -> "\uD83D\uDCAC"
        "trash" -> "\uD83D\uDDD1"
        "trash.fill" -> "\uD83D\uDDD1"
        "folder" -> "\uD83D\uDCC1"
        "folder.fill" -> "\uD83D\uDCC1"
        "doc" -> "\uD83D\uDCC4"
        "doc.fill" -> "\uD83D\uDCC4"
        "photo" -> "\uD83D\uDDBC"
        "photo.fill" -> "\uD83D\uDDBC"
        "camera" -> "\uD83D\uDCF7"
        "camera.fill" -> "\uD83D\uDCF7"
        "map" -> "\uD83D\uDDFA"
        "map.fill" -> "\uD83D\uDDFA"
        "cart" -> "\uD83D\uDED2"
        "cart.fill" -> "\uD83D\uDED2"
        "creditcard" -> "\uD83D\uDCB3"
        "creditcard.fill" -> "\uD83D\uDCB3"
        "wifi" -> "\uD83D\uDCF6"
        "antenna.radiowaves.left.and.right" -> "\uD83D\uDCE1"
        "lock.fill" -> "\uD83D\uDD12"
        "lock" -> "\uD83D\uDD12"
        "lock.open.fill" -> "\uD83D\uDD13"
        "key.fill" -> "\uD83D\uDD11"
        "paperplane.fill" -> "\u27A4"
        "arrow.right" -> "\u2192"
        "arrow.left" -> "\u2190"
        "arrow.up" -> "\u2191"
        "arrow.down" -> "\u2193"
        "chevron.right" -> "\u203A"
        "chevron.left" -> "\u2039"
        "chevron.up" -> "\u02C6"
        "chevron.down" -> "\u02C7"
        "info.circle" -> "\u2139"
        "info.circle.fill" -> "\u2139"
        "exclamationmark.triangle" -> "\u26A0"
        "exclamationmark.triangle.fill" -> "\u26A0"
        "play.fill" -> "\u25B6"
        "pause.fill" -> "\u23F8"
        "stop.fill" -> "\u23F9"
        "forward.fill" -> "\u23E9"
        "backward.fill" -> "\u23EA"
        else -> {
            // Never take(1) — that rendered a random letter. Neutral bullet + Material ligature name.
            if (systemName.isBlank()) "\u25CF" else "\u25CF"
        }
    }
}


internal fun iconMaterialName(systemName: String): String {
    val key = systemName.lowercase(Locale.US)
    return when (key) {
        "star" -> "star"
        "star.fill" -> "star"
        "heart" -> "favorite"
        "heart.fill" -> "favorite"
        "person" -> "person"
        "person.fill" -> "person"
        "person.2.fill" -> "group"
        "person.badge.plus" -> "person_add"
        "gear" -> "settings"
        "gearshape" -> "settings"
        "gearshape.fill" -> "settings"
        "checkmark" -> "check"
        "checkmark.circle" -> "check_circle"
        "checkmark.circle.fill" -> "check_circle"
        "xmark" -> "close"
        "xmark.circle" -> "cancel"
        "plus" -> "add"
        "plus.circle" -> "add_circle"
        "minus" -> "remove"
        "minus.circle" -> "do_not_disturb_on"
        "bell" -> "notifications"
        "bell.fill" -> "notifications"
        "house" -> "home"
        "house.fill" -> "home"
        "magnifyingglass" -> "search"
        "calendar" -> "calendar_today"
        "calendar.badge.clock" -> "event"
        "calendar.circle" -> "calendar_today"
        "clock" -> "schedule"
        "clock.fill" -> "schedule"
        "globe" -> "public"
        "cloud.fill" -> "cloud"
        "cloud.sun.fill" -> "partly_cloudy_day"
        "cloud.rain.fill" -> "rainy"
        "cloud.bolt.fill" -> "thunderstorm"
        "sun.max" -> "wb_sunny"
        "sun.max.fill" -> "wb_sunny"
        "moon.fill" -> "dark_mode"
        "moon.stars.fill" -> "clear_night"
        "bolt" -> "bolt"
        "bolt.fill" -> "bolt"
        "location.fill" -> "location_on"
        "flame.fill" -> "local_fire_department"
        "drop.fill" -> "water_drop"
        "leaf.fill" -> "eco"
        "eye.fill" -> "visibility"
        "bubble.left.fill" -> "chat_bubble"
        "music.note" -> "music_note"
        "music.note.list" -> "queue_music"
        "hourglass" -> "hourglass_empty"
        "hourglass.bottomhalf.filled" -> "hourglass_bottom"
        "figure.run" -> "directions_run"
        "paintpalette.fill" -> "palette"
        "quote.opening" -> "format_quote"
        "bitcoinsign.circle.fill" -> "currency_bitcoin"
        "battery.100" -> "battery_full"
        "iphone" -> "phone_iphone"
        "ipad" -> "tablet_mac"
        "applewatch" -> "watch"
        "airpodspro" -> "headphones"
        "desktopcomputer" -> "desktop_windows"
        "internaldrive" -> "hard_drive"
        "network" -> "lan"
        "envelope.fill" -> "mail"
        "envelope" -> "mail"
        "phone.fill" -> "call"
        "phone" -> "call"
        "message.fill" -> "message"
        "trash" -> "delete"
        "trash.fill" -> "delete"
        "folder" -> "folder"
        "folder.fill" -> "folder"
        "doc" -> "description"
        "doc.fill" -> "description"
        "photo" -> "photo"
        "photo.fill" -> "photo"
        "camera" -> "photo_camera"
        "camera.fill" -> "photo_camera"
        "map" -> "map"
        "map.fill" -> "map"
        "cart" -> "shopping_cart"
        "cart.fill" -> "shopping_cart"
        "creditcard" -> "credit_card"
        "creditcard.fill" -> "credit_card"
        "wifi" -> "wifi"
        "antenna.radiowaves.left.and.right" -> "cell_tower"
        "lock.fill" -> "lock"
        "lock" -> "lock"
        "lock.open.fill" -> "lock_open"
        "key.fill" -> "key"
        "paperplane.fill" -> "send"
        "arrow.right" -> "arrow_forward"
        "arrow.left" -> "arrow_back"
        "arrow.up" -> "arrow_upward"
        "arrow.down" -> "arrow_downward"
        "chevron.right" -> "chevron_right"
        "chevron.left" -> "chevron_left"
        "chevron.up" -> "expand_less"
        "chevron.down" -> "expand_more"
        "info.circle" -> "info"
        "info.circle.fill" -> "info"
        "exclamationmark.triangle" -> "warning"
        "exclamationmark.triangle.fill" -> "warning"
        "play.fill" -> "play_arrow"
        "pause.fill" -> "pause"
        "stop.fill" -> "stop"
        "forward.fill" -> "fast_forward"
        "backward.fill" -> "fast_rewind"
        else -> key.trimEnd().removeSuffix(".fill").replace('.', '_').ifBlank { "circle" }
    }
}


private var materialSymbolsTypefaceCache: Typeface? = null
private var materialSymbolsLoaded = false

/** Optional asset: `assets/material_symbols_outlined.ttf` (Material Symbols Outlined). */
internal fun materialSymbolsTypeface(context: Context): Typeface? {
    if (materialSymbolsLoaded) return materialSymbolsTypefaceCache
    materialSymbolsLoaded = true
    materialSymbolsTypefaceCache = listOf(
        "material_symbols_outlined.ttf",
        "fonts/MaterialSymbolsOutlined.ttf",
        "fonts/material_symbols_outlined.ttf",
    ).firstNotNullOfOrNull { path ->
        runCatching { Typeface.createFromAsset(context.assets, path) }.getOrNull()
    }
    return materialSymbolsTypefaceCache
}


internal fun drawSystemIconBitmap(context: Context, systemName: String, sizeDp: Int, argb: Int): Bitmap? {
    val ms = materialSymbolsTypeface(context)
    if (ms != null) {
        return drawGlyphBitmap(context, iconMaterialName(systemName), sizeDp, argb, ms)
    }
    return drawGlyphBitmap(context, iconGlyph(systemName), sizeDp, argb)
}



internal fun drawGlyphBitmap(
    context: Context,
    glyph: String,
    sizeDp: Int,
    argb: Int,
    typeface: Typeface = Typeface.DEFAULT_BOLD,
): Bitmap? {
    if (glyph.isBlank()) return null
    val px = dpToPx(context, sizeDp.toFloat()).coerceAtLeast(16)
    val bmp = Bitmap.createBitmap(px, px, Bitmap.Config.ARGB_8888)
    val canvas = Canvas(bmp)
    val paint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        color = argb
        textAlign = Paint.Align.CENTER
        textSize = px * 0.78f
        this.typeface = typeface
    }
    val x = px / 2f
    val y = px / 2f - (paint.descent() + paint.ascent()) / 2f
    canvas.drawText(glyph, x, y, paint)
    return bmp
}

