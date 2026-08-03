package git.s00d.widgets

import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.glance.GlanceModifier
import androidx.glance.action.actionParametersOf
import androidx.glance.action.clickable
import androidx.glance.appwidget.action.actionRunCallback
import androidx.glance.appwidget.cornerRadius
import androidx.glance.background
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
            val wrapped = applyAction(modifier, action, payload, url)
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
        "toggle" -> {
            val label = mediaGlyphFallback(
                el.str("label", el.str("content", "toggle"))
            )
            val isOn = el.bool("isOn", false)
            val action = el.str("action", "")
            // Next state when no explicit payload — matches iOS/desktop toggle contract.
            val payload = el.str("payload", if (isOn) "false" else "true")
            val url = el.str("url", "")
            val actionModifier = applyAction(modifier, action, payload, url)
            val tint = resolveColorProvider(context, el.opt("tint"))
                ?: ColorProvider(Color(0xFF4CAF50))
            val ink = resolveColorProvider(context, el.opt("color"))
                ?: ColorProvider(if (surfaceIsDark(context)) Color(0xFFE2E8F0) else Color(0xFF111111))
            val glyph = if (isOn) "\u2713" else "\u25CB"
            Row(
                modifier = actionModifier.padding(vertical = 2.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    glyph,
                    style = TextStyle(
                        color = if (isOn) tint else ColorProvider(Color(0xFF999999)),
                        fontSize = 16.sp,
                        fontWeight = FontWeight.Bold
                    )
                )
                Spacer(GlanceModifier.width(6.dp))
                Text(label, style = TextStyle(color = ink, fontSize = 14.sp))
            }
        }
        else -> {
            val labelRaw = el.str("label", el.str("content", "button"))
            val label = mediaGlyphFallback(labelRaw)
            val action = el.str("action", "")
            val payload = el.str("payload", "")
            val url = el.str("url", "")
            val bg = resolveColorProvider(context, el.opt("backgroundColor")) ?: resolveColorProvider(context, el.opt("tint"))
            val actionModifier = applyAction(modifier, action, payload, url)
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
            if (bg != null) {
                Box(modifier = boxMod) {
                    if (el.has("textAlignment")) {
                        Box(modifier = GlanceModifier.fillMaxWidth(), contentAlignment = contentAlign) {
                            renderElementText(context, textElement, text)
                        }
                    } else {
                        renderElementText(context, textElement, text)
                    }
                }
            } else {
                if (el.has("textAlignment")) {
                    Box(modifier = actionModifier.fillMaxWidth(), contentAlignment = contentAlign) {
                        renderElementText(context, textElement, text)
                    }
                } else {
                    renderElementText(context, textElement, text, actionModifier)
                }
            }
        }
    }
}

internal fun applyAction(modifier: GlanceModifier, action: String, payload: String, url: String): GlanceModifier {
    if (action.isBlank() && url.isBlank()) return modifier
    return modifier.clickable(
        actionRunCallback<WidgetActionCallback>(
            actionParametersOf(
                ACTION_KEY to action,
                PAYLOAD_KEY to payload,
                URL_KEY to url
            )
        )
    )
}
