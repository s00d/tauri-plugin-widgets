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
import androidx.glance.layout.fillMaxSize
import androidx.glance.layout.fillMaxWidth
import androidx.glance.layout.height
import androidx.glance.layout.width
import org.json.JSONArray
import org.json.JSONObject

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
                )
            }
        }
        "hstack" -> {
            val vAlign = parseVerticalStackAlignment(el.str("alignment", ""))
            Row(
                modifier = modifier.fillMaxWidth(),
                verticalAlignment = vAlign
            ) {
                renderChildrenHorizontal(scope, el.arr("children"), spacing)
            }
        }
        "zstack" -> {
            val alignment = parseContentAlignment(el.str("alignment", "center").ifBlank { "center" })
            // fillMaxSize inside a Row steals all remaining width/height and zeros siblings
            // (nested-dashboard badge crushed the title/button/body). Wrap in hstacks.
            val boxMod = if (inHorizontal) modifier else modifier.fillMaxWidth()
            Box(modifier = boxMod, contentAlignment = alignment) {
                val children = el.arr("children")
                if (children != null && children.length() > 0) {
                    for (i in 0 until children.length()) {
                        val child = children.optJSONObject(i) ?: continue
                        // Do not fillMaxSize on children — that breaks Box contentAlignment.
                        RenderElement(scope, El(child), GlanceModifier)
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
    for (idx in visible.indices) {
        if (idx > 0 && spacing > 0) {
            // Spacer between children — padding(bottom) on thin dividers often collapses in RemoteViews.
            Spacer(GlanceModifier.height(spacing.dp))
        }
        val child = visible[idx]
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
}

@Composable
private fun RowScope.renderChildrenHorizontal(
    scope: RenderScope,
    children: JSONArray?,
    spacing: Int,
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
    val allButtons = visible.isNotEmpty() && visible.all {
        it.widgetString("type", "") in setOf("button", "toggle")
    }
    val childScope = scope.horizontal()
    for (idx in visible.indices) {
        val child = visible[idx]
        if (idx > 0 && spacing > 0) {
            Spacer(GlanceModifier.width(spacing.dp))
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
                    allButtons -> true
                    else -> false
                }
                val base = if (wantsWeight) GlanceModifier.defaultWeight() else GlanceModifier
                Box(modifier = base) {
                    RenderElement(childScope, El(child), GlanceModifier)
                }
            }
        }
    }
}
