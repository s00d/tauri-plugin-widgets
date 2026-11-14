package git.s00d.widgets

import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.glance.GlanceModifier
import androidx.glance.Image
import androidx.glance.ImageProvider
import androidx.glance.action.Action
import androidx.glance.appwidget.CheckBox
import androidx.glance.appwidget.CheckboxDefaults
import androidx.glance.appwidget.Switch
import androidx.glance.appwidget.SwitchDefaults
import androidx.glance.layout.size
import androidx.glance.action.actionParametersOf
import androidx.glance.action.clickable
import androidx.glance.appwidget.action.actionRunCallback
import androidx.glance.appwidget.cornerRadius
import androidx.glance.background
import androidx.glance.ColorFilter
import androidx.glance.layout.Alignment
import androidx.glance.layout.Box
import androidx.glance.layout.Row
import androidx.glance.layout.Spacer
import androidx.glance.layout.fillMaxWidth
import androidx.glance.layout.padding
import androidx.glance.layout.width
import androidx.glance.text.FontWeight
import androidx.glance.text.Text
import androidx.glance.text.TextStyle
import androidx.glance.unit.ColorProvider
import org.json.JSONObject

@Composable
internal fun RenderInteractive(scope: RenderScope, el: El, modifier: GlanceModifier) {
    val context = scope.context
    when (el.type) {
        "link" -> {
            val action = el.str("action", "")
            val payload = el.str("payload", "")
            val url = el.str("url", "")
            val wrapped = applyAction(modifier, action, payload, url, el.setJson())
            val children = el.arr("children")
            if (children != null && children.length() > 0) {
                Box(modifier = wrapped) {
                    for (i in 0 until children.length()) {
                        val child = children.optJSONObject(i) ?: continue
                        RenderElement(scope, El(child), GlanceModifier)
                    }
                }
            } else {
                renderElementText(context, el, el.str("label", "link"), wrapped)
            }
        }
        "toggle" -> if (el.has(WireField.VARIANT)) renderCompoundToggle(scope, el, modifier) else {
            val label = mediaGlyphFallback(
                el.str("label", el.str("content", "toggle"))
            )
            val isOn = el.bool("isOn", false)
            val action = el.str("action", "")
            // Next state when no explicit payload — matches iOS/desktop toggle contract.
            val payload = el.str("payload", if (isOn) "false" else "true")
            val url = el.str("url", "")
            val actionModifier = applyAction(modifier, action, payload, url, el.setJson())
            val tint = resolveColorProvider(context, el.opt("tint"))
                ?: ColorProvider(Color(0xFF4CAF50))
            val ink = resolveColorProvider(context, el.opt("color"))
                ?: ColorProvider(if (surfaceIsDark(context)) Color(0xFFE2E8F0) else Color(0xFF111111))
            val markColor = if (isOn) tint else resolveColorProvider(context, el.opt(WireField.UNCHECKED_TINT)) ?: ColorProvider(Color(0xFF999999))
            Row(
                modifier = actionModifier.padding(vertical = 2.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Image(
                    provider = ImageProvider(drawCheckCircleBitmap(context, isOn, colorProviderArgb(markColor, context, android.graphics.Color.GRAY), 20)),
                    contentDescription = if (isOn) "on" else "off",
                    modifier = GlanceModifier.size(20.dp)
                )
                Spacer(GlanceModifier.width(6.dp))
                Text(label, style = TextStyle(color = ink, fontSize = 14.sp))
            }
        }
        else -> if (el.has(WireField.VARIANT) || el.has(WireField.SYSTEM_NAME) || el.bool(WireField.DISABLED)) {
            renderStyledButton(scope, el, modifier)
        } else {
            val labelRaw = el.str("label", el.str("content", "button"))
            val label = mediaGlyphFallback(labelRaw)
            val action = el.str("action", "")
            val payload = el.str("payload", "")
            val url = el.str("url", "")
            val bg = resolveColorProvider(context, el.opt("backgroundColor")) ?: resolveColorProvider(context, el.opt("tint"))
            val actionModifier = applyAction(modifier, action, payload, url, el.setJson())
            val text = label
            val buttonAlignRaw = el.str("textAlignment", el.str("alignment", "center"))
            val textElement = El(JSONObject(el.raw.toString()).apply { put("alignment", buttonAlignRaw) })
            val contentAlign = parseButtonContentAlignment(buttonAlignRaw)
            val radius = el.num("cornerRadius", -1.0)
            var boxMod = actionModifier
            if (bg != null) boxMod = boxMod.background(bg)
            if (radius >= 0) boxMod = boxMod.cornerRadius(radius.toInt().coerceAtLeast(0).dp)
            // Honor DSL padding as content insets via applyCommonStyle; default only when absent.
            val pad = el.opt("padding")
            boxMod = if (pad == null || pad === JSONObject.NULL) {
                boxMod.padding(horizontal = 10.dp, vertical = 6.dp)
            } else {
                // Explicit padding already applied on baseModifier by applyCommonStyle.
                boxMod
            }
            val icon = mediaIconRes(labelRaw)
            @Composable
            fun content(mod: GlanceModifier = GlanceModifier) {
                if (icon == null) {
                    renderElementText(context, textElement, text, mod)
                } else {
                    val ink = resolveColorProvider(context, el.opt("color")) ?: semanticLabelProvider(context)
                    val px = el.num("fontSize", 14.0).toInt().coerceAtLeast(8)
                    Image(ImageProvider(icon), contentDescription = labelRaw, colorFilter = ColorFilter.tint(ink),
                        modifier = mod.size(px.dp))
                }
            }
            if (bg != null) {
                // Label centered by default, like iOS and desktop (was start-aligned here).
                Box(modifier = boxMod, contentAlignment = contentAlign) {
                    if (el.has("textAlignment")) {
                        Box(modifier = GlanceModifier.fillMaxWidth(), contentAlignment = contentAlign) {
                            content()
                        }
                    } else {
                        content()
                    }
                }
            } else {
                if (el.has("textAlignment")) {
                    Box(modifier = actionModifier.fillMaxWidth(), contentAlignment = contentAlign) {
                        content()
                    }
                } else {
                    content(actionModifier)
                }
            }
        }
    }
}

internal fun applyAction(modifier: GlanceModifier, action: String, payload: String, url: String, set: String = ""): GlanceModifier {
    val a = actionFor(action, payload, url, set) ?: return modifier
    return modifier.clickable(a)
}

/** Element `set` (contract v3) as JSON text for the tap callback; empty when absent. */
internal fun El.setJson(): String = obj("set")?.takeIf { it.length() > 0 }?.toString().orEmpty()

internal fun actionFor(action: String, payload: String, url: String, set: String = ""): Action? {
    if (action.isBlank() && url.isBlank() && set.isBlank()) return null
    return actionRunCallback<WidgetActionCallback>(
        actionParametersOf(
            ACTION_KEY to action,
            PAYLOAD_KEY to payload,
            URL_KEY to url,
            SET_KEY to set,
        )
    )
}

private fun ColorProvider.withAlpha(context: android.content.Context, alpha: Float): ColorProvider {
    val argb = colorProviderArgb(this, context, android.graphics.Color.GRAY)
    return ColorProvider(Color(argb).copy(alpha = Color(argb).alpha * alpha))
}

/**
 * `variant` (bordered | prominent | plain | outline), `systemName` icon, `disabled`.
 * A styled Box (Glance Button has no icon slot and fixed Material styling): bordered =
 * accent at 20 % behind an accent label, prominent = solid accent with a white label,
 * plain = label only, outline = baked rounded stroke. Disabled = 40 % alpha, no click.
 */
@Composable
private fun renderStyledButton(scope: RenderScope, el: El, modifier: GlanceModifier) {
    val context = scope.context
    val variant = el.str(WireField.VARIANT, "bordered")
    val disabled = el.bool(WireField.DISABLED)
    val label = mediaGlyphFallback(el.str("label", el.str("content", "")))
    val accent = resolveColorProvider(context, el.opt("backgroundColor"))
        ?: resolveColorProvider(context, el.opt("tint"))
        ?: resolveColorProvider(context, "accent")!!
    val fade = if (disabled) 0.4f else 1f
    val ink = (resolveColorProvider(context, el.opt("color"))
        ?: if (variant == "prominent") ColorProvider(Color.White) else accent).withAlpha(context, fade)
    val fontSize = el.num("fontSize", 14.0).toFloat()
    val radius = el.num("cornerRadius", 8.0).toFloat().coerceAtLeast(0f)
    var box = if (disabled) modifier else applyAction(modifier, el.str("action", ""), el.str("payload", ""), el.str("url", ""), el.setJson())
    when (variant) {
        "prominent" -> box = box.background(accent.withAlpha(context, fade))
        "bordered" -> box = box.background(accent.withAlpha(context, 0.2f * fade))
        "outline" -> {
            // Baked at the estimated button size so the stroke is not stretched much.
            val iconW = if (el.has(WireField.SYSTEM_NAME)) fontSize + 4f else 0f
            val wDp = label.length * fontSize * 0.6f + iconW + 16f
            val hDp = fontSize * 1.3f + 12f
            outlineBitmap(context, wDp, hDp, radius, colorProviderArgb(ink, context, android.graphics.Color.GRAY))
                ?.let { box = box.background(ImageProvider(it)) }
        }
    }
    if (variant != "outline" && variant != "plain" && radius > 0f) box = box.cornerRadius(radius.dp)
    val pad = el.opt("padding")
    if (pad == null || pad === JSONObject.NULL) box = box.padding(horizontal = 8.dp, vertical = 6.dp)
    val style = textStyleFromElement(context, el.raw).copy(color = ink, fontSize = fontSize.sp)
    Row(modifier = box, horizontalAlignment = Alignment.CenterHorizontally, verticalAlignment = Alignment.CenterVertically) {
        val icon = el.str(WireField.SYSTEM_NAME, "")
        if (icon.isNotBlank()) {
            val bmp = drawSystemIconBitmap(context, icon, fontSize.toInt(), colorProviderArgb(ink, context, android.graphics.Color.GRAY))
            if (bmp != null) {
                Image(ImageProvider(bmp), contentDescription = null, modifier = GlanceModifier.size(fontSize.toInt().dp))
                if (label.isNotEmpty()) Spacer(GlanceModifier.width(4.dp))
            }
        }
        if (label.isNotEmpty()) Text(label, style = style, maxLines = 1)
    }
}

private fun outlineBitmap(context: android.content.Context, wDp: Float, hDp: Float, radiusDp: Float, argb: Int): android.graphics.Bitmap? {
    val d = context.resources.displayMetrics.density
    val w = (wDp * d).toInt().coerceAtLeast(8)
    val h = (hDp * d).toInt().coerceAtLeast(8)
    val bmp = android.graphics.Bitmap.createBitmap(w, h, android.graphics.Bitmap.Config.ARGB_8888)
    val stroke = 1.5f * d
    val r = radiusDp * d
    android.graphics.Canvas(bmp).drawRoundRect(
        android.graphics.RectF(stroke / 2, stroke / 2, w - stroke / 2, h - stroke / 2), r, r,
        android.graphics.Paint(android.graphics.Paint.ANTI_ALIAS_FLAG).apply {
            style = android.graphics.Paint.Style.STROKE
            strokeWidth = stroke
            color = argb
        },
    )
    return bmp
}

/**
 * `variant` switch / checkbox: Glance Switch / CheckBox (native compound buttons on API 31+,
 * Glance's image backport below). Checked = `tint`, unchecked = `uncheckedTint`.
 */
@Composable
private fun renderCompoundToggle(scope: RenderScope, el: El, modifier: GlanceModifier) {
    val context = scope.context
    val isOn = el.bool("isOn", false)
    val label = mediaGlyphFallback(el.str("label", el.str("content", "")))
    val onChange = actionFor(el.str("action", ""), el.str("payload", if (isOn) "false" else "true"), el.str("url", ""), el.setJson())
    val tint = resolveColorProvider(context, el.opt("tint")) ?: ColorProvider(Color(0xFF4CAF50))
    val off = resolveColorProvider(context, el.opt(WireField.UNCHECKED_TINT)) ?: ColorProvider(Color(0xFF999999))
    val ink = resolveColorProvider(context, el.opt("color"))
        ?: ColorProvider(if (surfaceIsDark(context)) Color(0xFFE2E8F0) else Color(0xFF111111))
    val style = TextStyle(color = ink, fontSize = 14.sp)
    if (el.str(WireField.VARIANT, "switch") == "checkbox") {
        CheckBox(
            checked = isOn,
            onCheckedChange = onChange,
            modifier = modifier,
            text = label,
            style = style,
            colors = CheckboxDefaults.colors(checkedColor = tint, uncheckedColor = off),
        )
    } else {
        Switch(
            checked = isOn,
            onCheckedChange = onChange,
            modifier = modifier,
            text = label,
            style = style,
            colors = SwitchDefaults.colors(
                checkedThumbColor = tint,
                uncheckedThumbColor = off,
                checkedTrackColor = tint.withAlpha(context, 0.5f),
                uncheckedTrackColor = off.withAlpha(context, 0.4f),
            ),
        )
    }
}
