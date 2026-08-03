package git.s00d.widgets

import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.glance.GlanceModifier
import androidx.glance.Image
import androidx.glance.ImageProvider
import androidx.glance.layout.ContentScale
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
                val systemName = el.str("systemName", "")
                val url = el.str("url", "")
                val fallback = when {
                    systemName.isNotBlank() -> iconGlyph(systemName)
                    url.isNotBlank() -> "img:${safeHost(url)}"
                    else -> "img"
                }
                val tint = colorProviderArgb(
                    resolveColorProvider(context, el.opt("color")),
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
            val size = el.num("size", 18.0).toInt().coerceAtLeast(1)
            val shapeType = el.str("shapeType", "circle")
            val isCapsule = shapeType.equals("capsule", ignoreCase = true)
            // Capsule contract: width = 2*size, height = size (not square)
            val shapeModifier = if (isCapsule) {
                modifier.width((size * 2).dp).height(size.dp)
            } else {
                modifier.size(size.dp)
            }
            val shapeBmp = drawShapeBitmap(context, el.raw, size)
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
                    "rectangle" -> "[##]"
                    else -> "(o)"
                }
                Text(symbol, modifier = shapeModifier, style = textStyleFromElement(context, el.raw))
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
