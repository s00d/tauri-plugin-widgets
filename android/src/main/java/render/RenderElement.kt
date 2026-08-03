package git.s00d.widgets

import android.content.Context
import androidx.compose.runtime.Composable
import androidx.glance.GlanceModifier
import org.json.JSONObject

/** Ambient render state threaded through the element tree (host context, slot size, layout axis). */
internal data class RenderScope(
    val context: Context,
    val sizeFamily: String,
    val axis: Axis = Axis.Vertical,
) {
    enum class Axis { Vertical, Horizontal, Overlay }

    fun horizontal() = copy(axis = Axis.Horizontal)

    fun vertical() = copy(axis = Axis.Vertical)

    val inHorizontal: Boolean get() = axis == Axis.Horizontal
}

@Composable
internal fun RenderElement(scope: RenderScope, el: El, modifier: GlanceModifier = GlanceModifier) {
    RenderTrace.type(el.type)
    val m = applyCommonStyle(scope.context, modifier, el)
    when (el.type) {
        "vstack", "hstack", "zstack", "grid", "container" -> RenderLayout(scope, el, m)
        "text", "label", "date", "timer" -> RenderText(scope, el, m)
        "image", "shape", "canvas" -> RenderMedia(scope, el, m)
        "progress", "gauge", "chart", "list" -> RenderData(scope, el, m)
        "button", "toggle", "link" -> RenderInteractive(scope, el, m)
        "spacer", "divider" -> RenderSpacing(scope, el, m)
        else -> {
            if (el.type.isNotBlank()) RenderTrace.skip(el.type, "unsupported")
            renderElementText(scope.context, el, el.type, m)
        }
    }
}

@Composable
internal fun RenderElement(
    context: Context,
    el: JSONObject,
    modifier: GlanceModifier = GlanceModifier,
    sizeFamily: String = "small",
    inHorizontal: Boolean = false,
) {
    val axis = if (inHorizontal) RenderScope.Axis.Horizontal else RenderScope.Axis.Vertical
    RenderElement(RenderScope(context, sizeFamily, axis), El(el), modifier)
}
