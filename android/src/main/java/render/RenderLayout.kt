package git.s00d.widgets

import android.util.Log
import androidx.compose.runtime.Composable
import androidx.compose.ui.unit.dp
import androidx.glance.GlanceModifier
import androidx.glance.layout.Alignment
import androidx.glance.layout.Box
import androidx.glance.layout.Column
import androidx.glance.layout.ColumnScope
import androidx.glance.layout.Row
import androidx.glance.layout.RowScope
import androidx.glance.layout.Spacer
import androidx.glance.layout.fillMaxHeight
import androidx.glance.layout.fillMaxSize
import androidx.glance.layout.fillMaxWidth
import androidx.glance.layout.height
import androidx.glance.layout.width
import org.json.JSONArray
import org.json.JSONObject
import java.util.Locale

/** True when a flex spacer should absorb adjacent VStack/HStack spacing. */
private fun isExpandingSpacer(child: JSONObject): Boolean {
    if (child.widgetString("type", "") != "spacer") return false
    return !child.has("minLength") || child.optDouble("flex", 0.0) > 0.0
}

/**
 * ZStack children that need the full slot (so Column spacers expand) vs badges that must
 * keep intrinsic size for [Box] contentAlignment.
 */
private fun zstackChildFills(child: JSONObject): Boolean {
    val type = child.widgetString("type", "")
    if (type in setOf("vstack", "hstack", "container", "grid", "zstack")) return true
    if (type == "shape") {
        val frame = child.optJSONObject("frame")
        val hasFrame =
            (frame?.optDouble("width", -1.0) ?: -1.0) > 0 &&
                (frame?.optDouble("height", -1.0) ?: -1.0) > 0
        val hasSize = child.optDouble("size", -1.0) > 0
        // Bare rectangle underlay (stress-align) → fill; sized circle/capsule → hug.
        if (!hasFrame && !hasSize) return true
        val shapeType = child.widgetString("shapeType", "circle").lowercase(Locale.US)
        if (shapeType == "rectangle" && !hasFrame) return true
    }
    return false
}

private fun zstackChildModifier(child: JSONObject): GlanceModifier =
    if (zstackChildFills(child)) GlanceModifier.fillMaxSize() else GlanceModifier

@Composable
internal fun RenderLayout(scope: RenderScope, el: El, modifier: GlanceModifier) {
    val inHorizontal = scope.axis == RenderScope.Axis.Horizontal
    val spacing = el.num("spacing", 0.0).toInt().coerceAtLeast(0)
    when (el.type) {
        "vstack" -> {
            val hAlign = parseHorizontalStackAlignment(el.str("alignment", ""))
            // Inside a Row, fillMaxWidth steals space from trailing spacer/button.
            val colMod = if (inHorizontal) modifier else modifier.fillMaxWidth()
            Column(
                modifier = colMod,
                horizontalAlignment = hAlign
            ) {
                renderChildrenVertical(
                    scope,
                    el.arr("children"),
                    spacing,
                    expandWidth = !inHorizontal,
                    justify = el.str(WireField.JUSTIFY, ""),
                )
            }
        }
        "hstack" -> {
            val vAlign = parseVerticalStackAlignment(el.str("alignment", ""))
            Row(
                modifier = modifier.fillMaxWidth(),
                verticalAlignment = vAlign
            ) {
                renderChildrenHorizontal(scope, el.arr("children"), spacing, el.str(WireField.JUSTIFY, ""))
            }
        }
        "zstack" -> {
            val alignment = parseContentAlignment(el.str("alignment", "center").ifBlank { "center" })
            // fillMaxSize inside a Row steals all remaining width/height and zeros siblings
            // (nested-dashboard badge crushed the title/button/body). Wrap in hstacks.
            val children = el.arr("children")
            // Full height only when a child fills it; a badge stack (sized circle + icon)
            // hugs, or it would take the whole column and zero the sibling flex spacers.
            val fills = children != null &&
                (0 until children.length()).any { children.optJSONObject(it)?.let(::zstackChildFills) == true }
            val boxMod = when {
                inHorizontal -> modifier
                fills -> modifier.fillMaxWidth().fillMaxHeight()
                else -> modifier.fillMaxWidth()
            }
            Box(modifier = boxMod, contentAlignment = alignment) {
                if (children != null && children.length() > 0) {
                    for (i in 0 until children.length()) {
                        val child = children.optJSONObject(i) ?: continue
                        // Stacks / full-bleed shapes must fill so Spacers get a height budget.
                        // Fixed badges (text/image/circle) stay intrinsic — fillMaxSize on those
                        // breaks Box contentAlignment.
                        RenderElement(scope, El(child), zstackChildModifier(child))
                    }
                }
            }
        }
        "container" -> {
            val alignment = parseContentAlignment(el.str("contentAlignment", "center"))
            Box(modifier = modifier, contentAlignment = alignment) {
                val children = el.arr("children")
                when {
                    children != null && children.length() == 1 -> {
                        // Single child: let Box contentAlignment position it (no full-width Column).
                        RenderElement(
                            scope.vertical(),
                            El(children.getJSONObject(0)),
                            GlanceModifier,
                        )
                    }
                    children != null && children.length() > 0 -> {
                        Column(
                            modifier = GlanceModifier,
                            horizontalAlignment = Alignment.CenterHorizontally,
                        ) {
                            renderChildrenVertical(
                                scope,
                                children,
                                spacing,
                                expandWidth = false,
                            )
                        }
                    }
                    else -> {
                        val elType = el.opt("type")?.toString().orEmpty()
                        Log.w(TAG, "container has no children; elType=$elType")
                    }
                }
            }
        }
        else -> {
            val children = el.arr("children") ?: JSONArray()
            val cols = el.int("columns", 2).coerceAtLeast(1)
            val rowSpacing = el.num("rowSpacing", spacing.toDouble()).toInt().coerceAtLeast(0)
            Column(modifier = modifier.fillMaxWidth()) {
                val maxRows = GLANCE_CONTAINER_LIMIT
                val visibleCols = cols.coerceAtMost(GLANCE_CONTAINER_LIMIT)
                val maxChildren = maxRows * visibleCols
                if (children.length() > maxChildren) {
                    Log.w(
                        TAG,
                        "grid truncated: requested=${children.length()} rendered=$maxChildren cols=$cols visibleCols=$visibleCols"
                    )
                }
                var i = 0
                var rowIdx = 0
                while (i < children.length() && i < maxChildren) {
                    if (rowIdx > 0 && rowSpacing > 0) {
                        Spacer(GlanceModifier.height(rowSpacing.dp))
                    }
                    Row(modifier = GlanceModifier.fillMaxWidth()) {
                        for (c in 0 until visibleCols) {
                            if (c > 0 && spacing > 0) {
                                Spacer(GlanceModifier.width(spacing.dp))
                            }
                            val child = children.optJSONObject(i + c)
                            if (child != null) {
                                Box(modifier = GlanceModifier.defaultWeight()) {
                                    RenderElement(scope.horizontal(), El(child), GlanceModifier.fillMaxWidth())
                                }
                            } else {
                                Spacer(GlanceModifier.defaultWeight())
                            }
                        }
                    }
                    i += visibleCols
                    rowIdx += 1
                }
            }
        }
    }
}

@Composable
private fun ColumnScope.renderChildrenVertical(
    scope: RenderScope,
    children: JSONArray?,
    spacing: Int,
    expandWidth: Boolean = true,
    justify: String = "",
) {
    if (children == null) return
    val visible = mutableListOf<JSONObject>()
    for (i in 0 until children.length()) {
        val child = children.optJSONObject(i) ?: continue
        visible += child
        if (visible.size >= GLANCE_CONTAINER_LIMIT) break
    }
    if (children.length() > visible.size) {
        Log.w(
            TAG,
            "column children truncated: requested=${children.length()} rendered=${visible.size} limit=$GLANCE_CONTAINER_LIMIT"
        )
    }
    if (visible.isEmpty()) return

    val childScope = scope.vertical()
    val plan = justifyPlan(justify, visible, spacing)
    repeat(plan.lead) { Spacer(GlanceModifier.defaultWeight()) }
    for (idx in visible.indices) {
        val child = visible[idx]
        if (idx > 0) repeat(plan.between) { Spacer(GlanceModifier.defaultWeight()) }
        if (idx > 0 && plan.fixedSpacing) {
            val prev = visible[idx - 1]
            // Fixed spacing Spacers around flex spacers steal height and crush the last row
            // (stress-align BL/BR → 2dp lines). Flex spacer already provides the gap.
            if (!isExpandingSpacer(prev) && !isExpandingSpacer(child)) {
                Spacer(GlanceModifier.height(spacing.dp))
            }
        }
        val flex = child.optDouble("flex", 0.0)
        val type = child.widgetString("type", "")
        // Buttons/images/shapes hug content — fillMaxWidth makes Tap a full-bleed bar.
        val hugWidth = type in setOf("button", "toggle", "image", "shape", "label")
        val widthMod = when {
            hugWidth -> GlanceModifier
            expandWidth -> GlanceModifier.fillMaxWidth()
            else -> GlanceModifier
        }
        val base = when {
            flex > 0.0 -> widthMod.defaultWeight()
            type == "spacer" && !child.has("minLength") -> widthMod.defaultWeight()
            type == "spacer" -> {
                val h = child.optDouble("minLength", 8.0).toInt().coerceAtLeast(1)
                widthMod.height(h.dp)
            }
            else -> widthMod
        }
        RenderElement(childScope, El(child), base)
    }
    repeat(plan.trail) { Spacer(GlanceModifier.defaultWeight()) }
}

@Composable
private fun RowScope.renderChildrenHorizontal(
    scope: RenderScope,
    children: JSONArray?,
    spacing: Int,
    justify: String = "",
) {
    if (children == null) return
    val visible = mutableListOf<JSONObject>()
    for (i in 0 until children.length()) {
        val child = children.optJSONObject(i) ?: continue
        visible += child
        if (visible.size >= GLANCE_CONTAINER_LIMIT) break
    }
    if (children.length() > visible.size) {
        Log.w(
            TAG,
            "row children truncated: requested=${children.length()} rendered=${visible.size} limit=$GLANCE_CONTAINER_LIMIT"
        )
    }
    val hasFlex = visible.any { it.optDouble("flex", 0.0) > 0.0 }
    // Buttons fill an equal share of the row (Apple / desktop: `maxWidth: .infinity`);
    // toggles hug their label, so a row of toggles packs at its spacing.
    val allButtons = visible.isNotEmpty() && visible.all { it.widgetString("type", "") == "button" }
    val childScope = scope.horizontal()
    // Width share of flex children (slot less fixed-width siblings and spacing).
    val fixedW = visible.filter { it.optDouble("flex", 0.0) <= 0.0 }
        .sumOf { it.optJSONObject("frame")?.optDouble("width", 0.0) ?: 0.0 }.toFloat()
    val flexSum = visible.sumOf { it.optDouble("flex", 0.0).coerceAtLeast(0.0) }.toFloat()
    val flexPool = (scope.slotW - fixedW - spacing * (visible.size - 1).coerceAtLeast(0)).coerceAtLeast(0f)
    fun scopeFor(child: JSONObject): RenderScope {
        val f = child.optDouble("flex", 0.0).toFloat()
        val fw = child.optJSONObject("frame")?.optDouble("width", 0.0)?.toFloat() ?: 0f
        return when {
            fw > 0f -> childScope.copy(slotW = fw)
            f > 0f && flexSum > 0f && scope.slotW > 0f -> childScope.copy(slotW = flexPool * f / flexSum)
            else -> childScope
        }
    }
    val plan = justifyPlan(justify, visible, spacing)
    // layoutPriority: lower-priority siblings take weight so they shrink (ellipsize) first.
    val prios = visible.map { it.optDouble(WireField.LAYOUT_PRIORITY, 0.0).takeIf { p -> !p.isNaN() } ?: 0.0 }
    val maxPrio = prios.maxOrNull() ?: 0.0
    val mixedPrio = prios.any { it != maxPrio }
    repeat(plan.lead) { Spacer(GlanceModifier.defaultWeight()) }
    for (idx in visible.indices) {
        val child = visible[idx]
        if (idx > 0) repeat(plan.between) { Spacer(GlanceModifier.defaultWeight()) }
        if (idx > 0 && plan.fixedSpacing) {
            val prev = visible[idx - 1]
            if (!isExpandingSpacer(prev) && !isExpandingSpacer(child)) {
                Spacer(GlanceModifier.width(spacing.dp))
            }
        }
        val t = child.widgetString("type", "")
        when (t) {
            "divider" -> {
                val th = child.optDouble("thickness", 1.0).toFloat().coerceAtLeast(2f).toInt()
                Box(modifier = GlanceModifier.width(th.dp).height(48.dp)) {
                    RenderElement(childScope, El(child), GlanceModifier.fillMaxSize())
                }
            }
            "spacer" -> {
                val min = child.optDouble("minLength", 8.0).toInt().coerceAtLeast(1)
                val flex = child.optDouble("flex", 0.0)
                val base = if (flex > 0.0 || !child.has("minLength")) {
                    GlanceModifier.defaultWeight()
                } else {
                    GlanceModifier.width(min.dp)
                }
                Spacer(modifier = base)
            }
            else -> {
                // Only flex / expanding spacer / equal buttons take weight.
                // Title columns wrap to intrinsic width so labels aren't ellipsized
                // while the trailing spacer still pushes Sync to the end.
                val wantsWeight = when {
                    child.optDouble("flex", 0.0) > 0.0 -> true
                    hasFlex -> false
                    plan.active -> false
                    mixedPrio -> prios[idx] < maxPrio
                    allButtons -> true
                    // Linear bars fill the width like Apple's `maxWidth: .infinity`; a bare
                    // fillMaxWidth would let the first one take the whole row (fitness Cal / H2O).
                    isLinearBar(child) -> true
                    else -> false
                }
                val base = if (wantsWeight) GlanceModifier.defaultWeight() else GlanceModifier
                Box(modifier = base) {
                    // A weighted button fills its share, label centred, like iOS / desktop.
                    val fill = wantsWeight && t == "button"
                    RenderElement(scopeFor(child), El(child), if (fill) GlanceModifier.fillMaxWidth() else GlanceModifier)
                }
            }
        }
    }
    repeat(plan.trail) { Spacer(GlanceModifier.defaultWeight()) }
}

internal fun isLinearBar(child: JSONObject): Boolean = when (child.widgetString("type", "")) {
    "progress" -> !child.widgetString("barStyle", "linear").equals("circular", true)
    "gauge" -> child.widgetString("gaugeStyle", "circular").equals("linear", true)
    else -> false
}

/** Weight spacers emulating `justify` (Glance has no Arrangement for RemoteViews). */
internal data class JustifyPlan(val lead: Int, val between: Int, val trail: Int, val fixedSpacing: Boolean) {
    val active: Boolean get() = lead + between + trail > 0
}

/**
 * `spaceAround` = 1 weight at each edge, 2 in each gap (edges get half a gap).
 * Glance Row/Column keep at most [GLANCE_CONTAINER_LIMIT] children: over budget, fixed
 * spacing goes first, then spaceAround degrades to spaceEvenly, then justify is dropped.
 */
internal fun justifyPlan(justify: String, visible: List<JSONObject>, spacing: Int): JustifyPlan {
    val n = visible.size
    val (lead, between, trail) = when (justify) {
        "center" -> Triple(1, 0, 1)
        "end" -> Triple(1, 0, 0)
        "spaceBetween" -> if (n > 1) Triple(0, 1, 0) else Triple(0, 0, 0)
        "spaceAround" -> Triple(1, 2, 1)
        "spaceEvenly" -> Triple(1, 1, 1)
        else -> Triple(0, 0, 0)
    }
    // An expanding spacer child already absorbs the free space: justify has nothing left.
    if (lead + between + trail == 0 || visible.any { isExpandingSpacer(it) || it.optDouble("flex", 0.0) > 0.0 }) {
        return JustifyPlan(0, 0, 0, spacing > 0)
    }
    val gaps = (n - 1).coerceAtLeast(0)
    fun total(b: Int, fixed: Boolean) = n + lead + trail + gaps * b + if (fixed) gaps else 0
    var b = between
    var fixed = spacing > 0
    if (total(b, fixed) > GLANCE_CONTAINER_LIMIT) fixed = false
    if (total(b, fixed) > GLANCE_CONTAINER_LIMIT && b == 2) b = 1
    if (total(b, fixed) > GLANCE_CONTAINER_LIMIT) return JustifyPlan(0, 0, 0, spacing > 0)
    return JustifyPlan(lead, b, trail, fixed)
}
