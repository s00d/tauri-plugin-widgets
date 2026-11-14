package git.s00d.widgets

import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.glance.GlanceModifier
import androidx.glance.Image
import androidx.glance.ImageProvider
import androidx.glance.appwidget.cornerRadius
import androidx.glance.background
import androidx.glance.layout.Box
import androidx.glance.layout.ContentScale
import androidx.glance.layout.fillMaxSize
import androidx.glance.layout.height
import androidx.glance.layout.size
import androidx.glance.layout.width
import androidx.glance.text.Text
import androidx.glance.text.TextStyle
import java.util.Locale

@Composable
internal fun RenderMedia(scope: RenderScope, el: El, modifier: GlanceModifier) {
    val context = scope.context
    when (el.type) {
        "image" -> {
            val size = el.num("size", 24.0).toInt().coerceAtLeast(1)
            val imageModifier = modifier.size(size.dp)
            val provider = imageProviderFromElement(context, el.raw)
            if (provider != null) {
                Image(
                    provider = provider,
                    contentDescription = el.str("alt", ""),
                    modifier = imageModifier,
                    contentScale = parseContentScale(el.str("contentMode", "fit"))
                )
            } else {
                // fallbackSystemName: the symbol shown when url / data did not load.
                val systemName = el.str("systemName", "").ifBlank { el.str(WireField.FALLBACK_SYSTEM_NAME, "") }
                val url = el.str("url", "")
                val fallback = when {
                    systemName.isNotBlank() -> iconGlyph(systemName)
                    url.isNotBlank() -> "img:${safeHost(url)}"
                    else -> "img"
                }
                // Material glyphs are one color: symbolRendering palette → first paletteColor
                // (when no `color`); hierarchical / multicolor / symbolWeight have no equivalent.
                val tint = colorProviderArgb(
                    resolveColorProvider(context, el.opt("color"))
                        ?: resolveColorProvider(context, el.arr(WireField.PALETTE_COLORS)?.opt(0)),
                    context,
                    semanticLabelProvider(context).getColor(context).toArgb()
                )
                val glyphBitmap = if (systemName.isNotBlank()) {
                    drawSystemIconBitmap(context, systemName, size, tint)
                } else {
                    drawGlyphBitmap(context, fallback, size, tint)
                }
                if (glyphBitmap != null) {
                    Image(
                        provider = ImageProvider(glyphBitmap),
                        contentDescription = systemName.ifBlank { "image" },
                        modifier = imageModifier,
                        contentScale = ContentScale.Fit
                    )
                } else {
                    val baseStyle = textStyleFromElement(context, el.raw)
                    val colored = resolveColorProvider(context, el.opt("color"))
                    val styled = if (colored != null) {
                        TextStyle(color = colored, fontSize = size.sp, fontWeight = baseStyle.fontWeight, textAlign = baseStyle.textAlign)
                    } else {
                        TextStyle(fontSize = size.sp, fontWeight = baseStyle.fontWeight, textAlign = baseStyle.textAlign)
                    }
                    Text(fallback, modifier = modifier, style = styled)
                }
            }
        }
        "shape" -> {
            val shapeType = el.str("shapeType", "circle")
            val isCapsule = shapeType.equals("capsule", ignoreCase = true)
            val (frameW, frameH) = frameSizeDp(el)
            val explicitSize = el.num("size", -1.0)
            val rectLike = shapeType.equals("rectangle", ignoreCase = true) ||
                shapeType.equals("roundedRectangle", ignoreCase = true)
            // Full-bleed rectangle underlay from zstack (modifier already fillMaxSize).
            val fillBleed =
                rectLike &&
                    explicitSize <= 0 &&
                    frameW <= 0 &&
                    frameH <= 0
            if (fillBleed) {
                val fill = resolveColorProvider(context, el.opt("fill"))
                val r = el.num("cornerRadius", 0.0)
                val bleedMod = if (r > 0) modifier.fillMaxSize().cornerRadius(r.toInt().dp) else modifier.fillMaxSize()
                if (fill != null) {
                    Box(modifier = bleedMod.background(fill)) {}
                } else {
                    Box(modifier = modifier.fillMaxSize()) {}
                }
            } else {
                val size = (if (explicitSize > 0) explicitSize else 18.0).toInt().coerceAtLeast(1)
                // Capsule contract: width = 2*size, height = size (not square)
                val shapeModifier = when {
                    frameW > 0 && frameH > 0 ->
                        modifier.width(frameW.toInt().dp).height(frameH.toInt().dp)
                    isCapsule -> modifier.width((size * 2).dp).height(size.dp)
                    else -> modifier.size(size.dp)
                }
                // New shape types draw at the frame aspect; legacy ones keep their bitmap.
                val newShape = shapeType.equals("roundedRectangle", true) || shapeType.equals("ellipse", true)
                val shapeBmp = if (newShape && frameW > 0 && frameH > 0) {
                    drawShapeBitmap(context, el.raw, size, frameW.toFloat(), frameH.toFloat())
                } else {
                    drawShapeBitmap(context, el.raw, size)
                }
                if (shapeBmp != null) {
                    Image(
                        provider = ImageProvider(shapeBmp),
                        contentDescription = shapeType,
                        modifier = shapeModifier,
                        contentScale = ContentScale.FillBounds
                    )
                } else {
                    val symbol = when (shapeType.lowercase(Locale.US)) {
                        "capsule" -> "[====]"
                        "rectangle", "roundedrectangle" -> "[##]"
                        else -> "(o)"
                    }
                    Text(symbol, modifier = shapeModifier, style = textStyleFromElement(context, el.raw))
                }
            }
        }
        else -> {
            renderCanvasElement(context, modifier, el)
        }
    }
}

@Composable
private fun renderCanvasElement(context: android.content.Context, modifier: GlanceModifier, el: El) {
    val bmp = drawCanvasBitmap(context, el.raw)
    if (bmp != null) {
        val widthDp = el.num("width", 0.0).toInt()
        val heightDp = el.num("height", 0.0).toInt()
        var imageModifier = modifier
        if (widthDp > 0) imageModifier = imageModifier.width(widthDp.dp)
        if (heightDp > 0) imageModifier = imageModifier.height(heightDp.dp)
        Image(
            provider = ImageProvider(bmp),
            contentDescription = "canvas",
            modifier = imageModifier,
            contentScale = ContentScale.FillBounds
        )
    } else {
        Text("canvas: empty", modifier = modifier, style = textStyleFromElement(context, el.raw))
    }
}
