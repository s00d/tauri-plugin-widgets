package git.s00d.widgets

import android.content.Context
import android.os.Build
import android.os.SystemClock
import android.util.TypedValue
import android.widget.RemoteViews
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.toArgb
import androidx.glance.GlanceModifier
import androidx.glance.appwidget.AndroidRemoteViews
import androidx.glance.layout.Row
import androidx.glance.layout.fillMaxWidth
import androidx.glance.text.Text
import androidx.glance.text.TextStyle

@Composable
internal fun RenderText(scope: RenderScope, el: El, modifier: GlanceModifier) {
    val context = scope.context
    val type = el.type
    when (type) {
        "label" -> {
            val txt = el.str("text", "")
            val icon = iconGlyph(el.str("systemName", "*"))
            val iconColor = resolveColorProvider(context, el.opt("iconColor"))
            if (iconColor != null) {
                Row(modifier = modifier.fillMaxWidth()) {
                    Text(icon, style = TextStyle(color = iconColor))
                    renderElementText(context, el, " $txt")
                }
            } else {
                renderElementText(context, el, "$icon $txt", modifier)
            }
        }
        else -> {
            when {
                type == "timer" -> renderLiveTimer(context, el, modifier)
                else -> {
                    val content = when (type) {
                        "date" -> formatDateValue(el.str("date", ""), el.str("dateStyle", "date"))
                        "timer" -> formatTimerValue(
                            el.str("targetDate", ""),
                            el.str("counting", "down")
                        )
                        else -> el.str("content", el.str("text", ""))
                    }
                    renderElementText(context, el, content, modifier)
                }
            }
        }
    }
}

@Composable
private fun renderLiveTimer(context: Context, el: El, modifier: GlanceModifier) {
    val target = parseIsoDate(el.str("targetDate", ""))
    val countingDown = !el.str("counting", "down").equals("up", ignoreCase = true)
    if (target == null || (countingDown && Build.VERSION.SDK_INT < 24)) {
        renderElementText(
            context,
            el,
            formatTimerValue(el.str("targetDate", ""), el.str("counting", "down")),
            modifier,
        )
        return
    }
    val wallDelta = target.time - System.currentTimeMillis()
    val base = if (countingDown) {
        SystemClock.elapsedRealtime() + wallDelta
    } else {
        SystemClock.elapsedRealtime() - (System.currentTimeMillis() - target.time)
    }
    val argb = colorProviderArgb(
        resolveColorProvider(context, el.opt("color")),
        context,
        semanticLabelProvider(context).getColor(context).toArgb(),
    )
    val fontSize = el.num(
        "fontSize",
        semanticTextSizeSp(el.str("textStyle", "")).toDouble(),
    ).toFloat().takeIf { it > 0f } ?: 14f
    val rv = RemoteViews(context.packageName, R.layout.tauri_chronometer).apply {
        setChronometer(R.id.tauri_chrono, base, null, true)
        if (Build.VERSION.SDK_INT >= 24) {
            setChronometerCountDown(R.id.tauri_chrono, countingDown)
        }
        setTextColor(R.id.tauri_chrono, argb)
        setTextViewTextSize(R.id.tauri_chrono, TypedValue.COMPLEX_UNIT_SP, fontSize)
    }
    AndroidRemoteViews(remoteViews = rv, modifier = modifier)
}

@Composable
internal fun renderElementText(
    context: Context,
    el: El,
    content: String,
    modifier: GlanceModifier = GlanceModifier,
) {
    val style = textStyleFromElement(context, el.raw)
    val limit = el.int("lineLimit", -1)
    if (limit > 0) {
        Text(content, modifier = modifier, style = style, maxLines = limit)
    } else {
        Text(content, modifier = modifier, style = style)
    }
}
