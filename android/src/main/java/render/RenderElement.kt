package git.s00d.widgets

import android.content.Context
import androidx.compose.runtime.Composable
import androidx.glance.GlanceModifier
import androidx.glance.layout.Box
import org.json.JSONObject

/** Ambient render state threaded through the element tree (host context, slot size, layout axis). */
internal data class RenderScope(
    val context: Context,
    val sizeFamily: String,
    val axis: Axis = Axis.Vertical,
    /** Element nesting level (depth guard, see [GLANCE_MAX_ELEMENT_DEPTH]). */
    val depth: Int = 0,
    /**
     * Estimated slot size in dp (0 = unknown). Glance gives no per-node constraints: the root
     * starts from LocalSize, stacks subtract their padding and a row splits its width between
     * flex children. Bitmap elements (charts) size themselves to it.
     */
    val slotW: Float = 0f,
    val slotH: Float = 0f,
) {
    enum class Axis { Vertical, Horizontal, Overlay }

    fun horizontal() = copy(axis = Axis.Horizontal)

    fun vertical() = copy(axis = Axis.Vertical)

    val inHorizontal: Boolean get() = axis == Axis.Horizontal

    /** Slot left inside `padding` (number or per-edge object, as [paddingModifier]). */
    fun insetBy(p: Any?): RenderScope {
        val (h, v) = when (p) {
            is Number -> p.toFloat() * 2 to p.toFloat() * 2
            is JSONObject -> (p.optDouble("leading", p.optDouble("start", 0.0)) + p.optDouble("trailing", p.optDouble("end", 0.0))).toFloat() to
                (p.optDouble("top", 0.0) + p.optDouble("bottom", 0.0)).toFloat()
            else -> 0f to 0f
        }
        return copy(slotW = (slotW - h).coerceAtLeast(0f), slotH = (slotH - v).coerceAtLeast(0f))
    }
}

@Composable
internal fun RenderElement(scope0: RenderScope, el: El, modifier: GlanceModifier = GlanceModifier) {
    // Glance rejects trees deeper than 50 emittables (the whole widget turns into the error
    // layout): drop the subtree instead and say so in the receipt.
    if (scope0.depth >= GLANCE_MAX_ELEMENT_DEPTH) {
        RenderTrace.skip(el.type, "depth")
        return
    }
    val scope = scope0.copy(depth = scope0.depth + 1)
    // `margin`: outer Box carries the parent's layout modifier (weight, fill) plus the
    // margin as padding; the element draws its background inside it.
    val margin = el.opt(WireField.MARGIN)
    if (margin is Number || margin is JSONObject) {
        Box(modifier = paddingModifier(modifier, margin)) {
            RenderElementBody(scope, el, GlanceModifier)
        }
    } else {
        RenderElementBody(scope, el, modifier)
    }
    // offsetX / offsetY / rotation / scale: RemoteViews have no transforms; the element
    // keeps its untransformed size and place (nothing is dropped). accentable: Apple only.
}

@Composable
private fun RenderElementBody(scope: RenderScope, el: El, modifier: GlanceModifier) {
    RenderTrace.type(el.type)
    val m = applyCommonStyle(scope.context, modifier, el)
    when (el.type) {
        "vstack", "hstack", "zstack", "grid", "container" -> RenderLayout(scope.insetBy(el.opt("padding")), el, m)
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
