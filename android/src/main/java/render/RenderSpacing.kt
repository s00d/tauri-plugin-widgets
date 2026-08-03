package git.s00d.widgets

import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.glance.GlanceModifier
import androidx.glance.background
import androidx.glance.layout.Box
import androidx.glance.layout.Column
import androidx.glance.layout.Spacer
import androidx.glance.layout.fillMaxWidth
import androidx.glance.layout.height
import androidx.glance.layout.width
import androidx.glance.unit.ColorProvider

@Composable
internal fun RenderSpacing(scope: RenderScope, el: El, modifier: GlanceModifier) {
    val context = scope.context
    when (el.type) {
        "divider" -> {
            val line = resolveColorProvider(context, el.opt("color")) ?: ColorProvider(Color.Gray)
            // RemoteViews often collapses 1.dp empty boxes — keep ≥2dp.
            val thicknessDp = el.num("thickness", 1.0).toFloat().coerceAtLeast(2f)
            val thickness = thicknessDp.dp
            if (scope.axis == RenderScope.Axis.Horizontal) {
                Box(
                    modifier = modifier
                        .width(thickness)
                        .height(48.dp)
                        .background(line)
                ) {}
            } else {
                // Built-in vertical breath so section separators never sit flush against
                // neighbors (parent Column spacing alone still looked tight on Glance).
                val edge = 4.dp
                Column(modifier = modifier.fillMaxWidth()) {
                    Spacer(GlanceModifier.height(edge))
                    Box(
                        modifier = GlanceModifier
                            .fillMaxWidth()
                            .height(thickness)
                            .background(line)
                    ) {}
                    Spacer(GlanceModifier.height(edge))
                }
            }
        }
        else -> {
            // Flex / no-minLength spacers keep Column/Row weight; fixed height would kill flex.
            if (el.has("minLength")) {
                val h = el.num("minLength", 8.0).toInt().coerceAtLeast(1)
                Spacer(modifier.height(h.dp))
            } else {
                Spacer(modifier)
            }
        }
    }
}
