//! SVG builders + optional PNG rasterize for Adaptive Cards Image data URIs.
//!
//! Used for `chart` / `canvas` / `gauge` / `shape` / `zstack` / gradient
//! backgrounds that Adaptive Cards cannot express natively.

use crate::models::{
    BackgroundValue, CanvasDrawCommand, CanvasElement, ChartDataPoint, ChartElement,
    ChartInterpolation, ChartType, ColorValue, ElementStyle, GaugeElement, GaugeStyle,
    GradientConfig, GradientDirection, GradientType, ImageElement, ProgressElement, ProgressStyle,
    ShapeElement, ShapeType, TextElement, UnitPoint, WidgetElement, ZStackElement,
};
use std::f64::consts::PI;

const DEFAULT_TINT: &str = "#4CAF50";
const PIE_COLORS: &[&str] = &[
    "#3b82f6", "#22c55e", "#f97316", "#ef4444", "#a855f7", "#eab308", "#ec4899", "#14b8a6",
];

/// Build an SVG document for chart/canvas/gauge/shape; `None` for other element types.
pub fn element_to_svg(el: &WidgetElement) -> Option<String> {
    match el {
        WidgetElement::Chart(c) => Some(chart_svg(c)),
        WidgetElement::Canvas(CanvasElement {
            width,
            height,
            elements,
            ..
        }) => Some(canvas_svg(*width, *height, elements)),
        WidgetElement::Gauge(GaugeElement {
            value,
            min,
            max,
            tint,
            gauge_style,
            current_value_label,
            label,
            min_label,
            max_label,
            ..
        }) => Some(gauge_svg(&GaugeSpec {
            value: *value,
            min: min.unwrap_or(0.0),
            max: max.unwrap_or(1.0),
            tint: tint.as_ref(),
            style: gauge_style.as_ref(),
            current: current_value_label.as_deref(),
            label: label.as_deref(),
            track: None,
            thickness: None,
            min_label: min_label.as_deref(),
            max_label: max_label.as_deref(),
        })),
        WidgetElement::Progress(ProgressElement {
            value,
            total,
            tint,
            label,
            bar_style: Some(ProgressStyle::Circular),
            track_color,
            thickness,
            ..
        }) => Some(gauge_svg(&GaugeSpec {
            value: *value,
            min: 0.0,
            max: if *total <= 0.0 { 1.0 } else { *total },
            tint: tint.as_ref(),
            style: Some(&GaugeStyle::Circular),
            current: None,
            label: label.as_deref(),
            track: track_color.as_ref(),
            thickness: *thickness,
            min_label: None,
            max_label: None,
        })),
        // Linear bars only need a picture for what AC columns cannot draw.
        WidgetElement::Progress(ProgressElement {
            value,
            total,
            tint,
            track_color,
            thickness,
            ..
        }) if track_color.is_some() || thickness.is_some() => {
            let total = if *total <= 0.0 { 1.0 } else { *total };
            Some(progress_bar_svg(
                (value / total).clamp(0.0, 1.0),
                tint.as_ref(),
                track_color.as_ref(),
                *thickness,
            ))
        }
        WidgetElement::Shape(s) => Some(shape_svg(s)),
        WidgetElement::ZStack(ZStackElement {
            children, style, ..
        }) => zstack_svg(children, style),
        _ => None,
    }
}

/// Number for SVG attributes: at most 2 decimals, no trailing zeros (`4`, `0.7`, `5.14`).
fn num(v: f64) -> String {
    let s = format!("{v:.2}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" {
        "0".into()
    } else {
        s.into()
    }
}

/// Build an SVG gradient fill for Adaptive Cards `backgroundImage` baking.
/// `stops` replace `colors`; `startPoint` / `endPoint` (unit space) replace `direction`
/// (radial: `startPoint` is the center).
pub fn gradient_to_svg(g: &GradientConfig, width: f64, height: f64) -> String {
    // Id from the content: deterministic output, and two gradients inlined into one
    // composite SVG only share an id when their definitions are identical.
    // FNV-1a: stable across Rust versions (DefaultHasher is not), so snapshots hold.
    let id = {
        let h = format!("{g:?}{width}{height}")
            .bytes()
            .fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
                (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
            });
        format!("g{h:x}")
    };
    let w = width.max(8.0);
    let h = height.max(8.0);
    let stops: String = if g.stops.is_empty() {
        let colors = if g.colors.is_empty() {
            vec!["#6366f1".into(), "#a855f7".into()]
        } else {
            g.colors.clone()
        };
        colors
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let off = if colors.len() == 1 {
                    0.0
                } else {
                    i as f64 / (colors.len() - 1) as f64
                };
                format!(
                    r#"<stop offset="{:.0}%" stop-color="{}"/>"#,
                    off * 100.0,
                    esc(c)
                )
            })
            .collect()
    } else {
        g.stops
            .iter()
            .map(|s| {
                format!(
                    r#"<stop offset="{}" stop-color="{}"/>"#,
                    num(s.location.clamp(0.0, 1.0)),
                    esc(&s.color)
                )
            })
            .collect()
    };
    let pct = |v: f64| format!("{}%", num(v * 100.0));
    let points = (g.start_point.is_some() || g.end_point.is_some()).then(|| {
        let s = g.start_point.unwrap_or(UnitPoint { x: 0.5, y: 0.0 });
        let e = g.end_point.unwrap_or(UnitPoint { x: 0.5, y: 1.0 });
        (pct(s.x), pct(s.y), pct(e.x), pct(e.y))
    });
    let s = |v: &str| v.to_string();
    match g.gradient_type {
        GradientType::Radial => {
            let (cx, cy) = g
                .start_point
                .map_or((s("50%"), s("50%")), |p| (pct(p.x), pct(p.y)));
            format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}"><defs><radialGradient id="{id}" cx="{cx}" cy="{cy}" r="70%">{stops}</radialGradient></defs><rect width="100%" height="100%" fill="url(#{id})"/></svg>"#
            )
        }
        GradientType::Angular | GradientType::Linear => {
            // Angular is approximated as a linear sweep — good enough for AC bake.
            let (x1, y1, x2, y2) = points.unwrap_or_else(|| {
                let (a, b, c, d) = match (&g.gradient_type, g.direction.as_ref()) {
                    (GradientType::Angular, _) => ("0%", "0%", "100%", "100%"),
                    (_, Some(GradientDirection::LeadingToTrailing)) => ("0%", "0%", "100%", "0%"),
                    (_, Some(GradientDirection::TrailingToLeading)) => ("100%", "0%", "0%", "0%"),
                    (_, Some(GradientDirection::BottomToTop)) => ("0%", "100%", "0%", "0%"),
                    (_, Some(GradientDirection::TopLeadingToBottomTrailing)) => {
                        ("0%", "0%", "100%", "100%")
                    }
                    (_, Some(GradientDirection::TopTrailingToBottomLeading)) => {
                        ("100%", "0%", "0%", "100%")
                    }
                    _ => ("0%", "0%", "0%", "100%"), // topToBottom default
                };
                (s(a), s(b), s(c), s(d))
            });
            format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}"><defs><linearGradient id="{id}" x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}">{stops}</linearGradient></defs><rect width="100%" height="100%" fill="url(#{id})"/></svg>"#
            )
        }
    }
}

/// Rasterize a gradient background to a PNG data URI when `rasterize` is enabled.
pub fn gradient_to_png_data_uri(g: &GradientConfig) -> Result<String, String> {
    svg_to_data_uri(&gradient_to_svg(g, 320.0, 200.0))
}

/// Intrinsic size of a zstack: its `frame`, or for a badge-like overlay (only shapes, images
/// and text) its largest sized shape / image. `None` = it fills its slot (layout zstacks with
/// stacks inside, like a root with corner badges): drawn on 160x160 and stretched.
/// The raster canvas and the Adaptive Card image both use it, so a 20 pt badge is not drawn
/// (and stretched) as a 160 px square.
pub(crate) fn zstack_size(children: &[WidgetElement], style: &ElementStyle) -> Option<(f64, f64)> {
    if let Some(wh) = style
        .frame
        .as_ref()
        .and_then(|f| Some((f.width?, f.height?)))
    {
        return Some(wh);
    }
    let leaf = |c: &WidgetElement| {
        matches!(
            c,
            WidgetElement::Shape(_) | WidgetElement::Image(_) | WidgetElement::Text(_)
        )
    };
    if !children.iter().all(leaf) {
        return None;
    }
    children
        .iter()
        .filter_map(|c| match c {
            WidgetElement::Shape(ShapeElement { size: Some(s), .. })
            | WidgetElement::Image(ImageElement { size: Some(s), .. }) => Some((*s, *s)),
            _ => None,
        })
        .reduce(|a, b| (a.0.max(b.0), a.1.max(b.1)))
        .filter(|(w, h)| *w > 0.0 && *h > 0.0)
}

fn zstack_svg(children: &[WidgetElement], style: &ElementStyle) -> Option<String> {
    let (w, h) = zstack_size(children, style).unwrap_or((160.0, 160.0));
    let mut layers = String::new();
    // Bake background so AC Image isn't transparent when style.background is set.
    if let Some(bg) = &style.background {
        match bg {
            BackgroundValue::Solid(s) => {
                let fill = normalize_color(s);
                layers.push_str(&format!(
                    r#"<rect width="100%" height="100%" fill="{fill}"/>"#
                ));
            }
            BackgroundValue::Adaptive { light, .. } => {
                let fill = normalize_color(light);
                layers.push_str(&format!(
                    r#"<rect width="100%" height="100%" fill="{fill}"/>"#
                ));
            }
            BackgroundValue::Gradient(g) => {
                let grad_svg = gradient_to_svg(g, w, h);
                let inner = strip_outer_svg(&grad_svg);
                layers.push_str(&format!(
                    r#"<svg x="0" y="0" width="{w}" height="{h}" viewBox="0 0 {w} {h}" preserveAspectRatio="none">{inner}</svg>"#
                ));
            }
        }
    }
    let mut drew = 0usize;
    for child in children {
        match child {
            WidgetElement::Text(TextElement {
                content,
                font_size,
                color,
                ..
            }) => {
                let size = font_size.unwrap_or(16.0);
                let fill = color_str(color.as_ref(), "#ffffff");
                layers.push_str(&format!(
                    r#"<text x="{:.1}" y="{:.1}" text-anchor="middle" dominant-baseline="middle" font-size="{size}" fill="{fill}" font-family="system-ui,sans-serif">{}</text>"#,
                    w / 2.0,
                    h / 2.0,
                    esc(content)
                ));
                drew += 1;
            }
            WidgetElement::Image(ImageElement {
                system_name,
                size,
                color,
                ..
            }) => {
                let glyph = system_name
                    .as_deref()
                    .map(|n| crate::adaptive_card::sf_symbol_glyph_public(n))
                    .unwrap_or("•");
                let sz = size.unwrap_or(28.0);
                let fill = color_str(color.as_ref(), "#ffffff");
                layers.push_str(&format!(
                    r#"<text x="{:.1}" y="{:.1}" text-anchor="middle" dominant-baseline="middle" font-size="{sz}" fill="{fill}">{}</text>"#,
                    w / 2.0,
                    h / 2.0,
                    esc(glyph)
                ));
                drew += 1;
            }
            other => {
                let Some(inner) = element_to_svg(other) else {
                    // Any unrenderable child → fall back so AC can render the full tree.
                    return None;
                };
                let (vbx, vby, vbw, vbh) = extract_viewbox(&inner).unwrap_or((0.0, 0.0, w, h));
                layers.push_str(&format!(
                    r#"<svg x="0" y="0" width="{w}" height="{h}" viewBox="{vbx} {vby} {vbw} {vbh}" preserveAspectRatio="xMidYMid meet">{inner_body}</svg>"#,
                    inner_body = strip_outer_svg(&inner)
                ));
                drew += 1;
            }
        }
    }
    if drew == 0 {
        return None;
    }
    Some(format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">{layers}</svg>"#
    ))
}

fn extract_viewbox(svg: &str) -> Option<(f64, f64, f64, f64)> {
    let re = regex_lite_viewbox(svg)?;
    Some(re)
}

/// Tiny viewBox extractor — avoids a regex crate dependency.
fn regex_lite_viewbox(svg: &str) -> Option<(f64, f64, f64, f64)> {
    let key = "viewBox=\"";
    let start = svg.find(key)? + key.len();
    let end = svg[start..].find('"')? + start;
    let parts: Vec<f64> = svg[start..end]
        .split_whitespace()
        .filter_map(|p| p.parse().ok())
        .collect();
    if parts.len() == 4 {
        Some((parts[0], parts[1], parts[2], parts[3]))
    } else {
        None
    }
}

fn strip_outer_svg(svg: &str) -> String {
    let s = svg.trim();
    if let Some(start) = s.find('>') {
        let inner = &s[start + 1..];
        if let Some(end) = inner.rfind("</svg>") {
            return inner[..end].to_string();
        }
    }
    s.to_string()
}

/// Rasterize SVG to `data:image/png;base64,…` when the `rasterize` feature is on.
pub fn svg_to_data_uri(svg: &str) -> Result<String, String> {
    #[cfg(feature = "rasterize")]
    {
        svg_to_data_uri_impl(svg)
    }
    #[cfg(not(feature = "rasterize"))]
    {
        let _ = svg;
        Err("rasterize feature disabled".into())
    }
}

/// A 1x1 PNG of one colour (`#rrggbb` / `#rrggbbaa`) as a data URI, no resvg needed.
/// Adaptive Cards has only six host-themed container styles; a repeated 1x1
/// `backgroundImage` is the only way to an exact colour. `None` for an unparsable colour.
pub fn solid_png_data_uri(hex: &str) -> Option<String> {
    use base64::Engine;
    let short = hex.strip_prefix('#')?;
    // `#rgb` / `#rgba` -> `#rrggbb` / `#rrggbbaa`.
    let long: String = if matches!(short.len(), 3 | 4) {
        short.chars().flat_map(|c| [c, c]).collect()
    } else {
        short.to_string()
    };
    let h = long.as_str();
    let byte = |i: usize| u8::from_str_radix(h.get(i..i + 2)?, 16).ok();
    let (r, g, b) = (byte(0)?, byte(2)?, byte(4)?);
    let a = match h.len() {
        6 => 255,
        8 => byte(6)?,
        _ => return None,
    };
    fn crc32(bytes: &[u8]) -> u32 {
        let mut c = !0u32;
        for &x in bytes {
            c ^= x as u32;
            for _ in 0..8 {
                c = if c & 1 != 0 {
                    0xEDB8_8320 ^ (c >> 1)
                } else {
                    c >> 1
                };
            }
        }
        !c
    }
    fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
        out.extend_from_slice(&(data.len() as u32).to_be_bytes());
        let start = out.len();
        out.extend_from_slice(kind);
        out.extend_from_slice(data);
        let crc = crc32(&out[start..]);
        out.extend_from_slice(&crc.to_be_bytes());
    }
    // One scanline: filter byte 0 + RGBA, in a zlib stream with one stored deflate block.
    let raw = [0u8, r, g, b, a];
    let (mut s1, mut s2) = (1u32, 0u32);
    for &x in &raw {
        s1 = (s1 + x as u32) % 65521;
        s2 = (s2 + s1) % 65521;
    }
    let mut zlib = vec![0x78, 0x01, 0x01, 5, 0, !5u8, 0xff];
    zlib.extend_from_slice(&raw);
    zlib.extend_from_slice(&((s2 << 16) | s1).to_be_bytes());
    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    chunk(&mut png, b"IHDR", &[0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0, 0, 0]);
    chunk(&mut png, b"IDAT", &zlib);
    chunk(&mut png, b"IEND", &[]);
    let b64 = base64::engine::general_purpose::STANDARD.encode(png);
    Some(format!("data:image/png;base64,{b64}"))
}

/// Convenience: element → PNG data URI.
pub fn element_to_png_data_uri(el: &WidgetElement) -> Result<String, String> {
    let svg =
        element_to_svg(el).ok_or_else(|| "element is not chart/canvas/gauge/shape".to_string())?;
    svg_to_data_uri(&svg)
}

#[cfg(feature = "rasterize")]
fn svg_to_data_uri_impl(svg: &str) -> Result<String, String> {
    use base64::Engine;
    // System fonts are scanned once per process: loading them opens and parses every font
    // file, which was most of the time of each rasterize call (Widgets Board cards).
    static FONTS: std::sync::LazyLock<std::sync::Arc<resvg::usvg::fontdb::Database>> =
        std::sync::LazyLock::new(|| {
            let mut db = resvg::usvg::fontdb::Database::new();
            db.load_system_fonts();
            std::sync::Arc::new(db)
        });
    let opts = resvg::usvg::Options {
        fontdb: FONTS.clone(),
        ..Default::default()
    };
    let tree = resvg::usvg::Tree::from_str(svg, &opts).map_err(|e| format!("usvg: {e}"))?;
    let size = tree.size();
    let w = size.width().ceil().max(1.0) as u32;
    let h = size.height().ceil().max(1.0) as u32;
    let mut pixmap =
        resvg::tiny_skia::Pixmap::new(w, h).ok_or_else(|| "pixmap alloc failed".to_string())?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );
    let png = pixmap
        .encode_png()
        .map_err(|e| format!("png encode: {e}"))?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(png);
    Ok(format!("data:image/png;base64,{b64}"))
}

fn color_str(c: Option<&ColorValue>, fallback: &str) -> String {
    match c {
        Some(ColorValue::Solid(s)) => normalize_color(s),
        Some(ColorValue::Adaptive { light, .. }) => normalize_color(light),
        None => fallback.to_string(),
    }
}

fn normalize_color(s: &str) -> String {
    let t = s.trim();
    if t.starts_with('#') || t.starts_with("rgb") {
        return t.to_string();
    }
    // Named / semantic tokens → fallback hex.
    match t.to_ascii_lowercase().as_str() {
        "accent" | "blue" => "#2196F3".into(),
        "good" | "success" | "green" => "#4CAF50".into(),
        "warning" | "orange" => "#FF9800".into(),
        "attention" | "error" | "danger" | "red" => "#F44336".into(),
        "label" | "dark" | "black" => "#212121".into(),
        "secondarylabel" | "light" | "white" => "#FAFAFA".into(),
        _ => {
            if t.is_empty() {
                DEFAULT_TINT.into()
            } else {
                t.to_string()
            }
        }
    }
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// One chart series: name (empty for unnamed), color, points in data order.
struct Series<'a> {
    name: &'a str,
    color: String,
    pts: Vec<&'a ChartDataPoint>,
}

/// Group points by `series` (first-appearance order). A single series keeps `tint`
/// (per-point colors apply to bars / slices only); further series take their first
/// point color, else the palette.
fn chart_series<'a>(pts: &'a [ChartDataPoint], tint: &str) -> Vec<Series<'a>> {
    let mut names: Vec<Option<&str>> = Vec::new();
    for p in pts {
        if !names.contains(&p.series.as_deref()) {
            names.push(p.series.as_deref());
        }
    }
    let multi = names.len() > 1;
    names
        .iter()
        .enumerate()
        .map(|(i, n)| {
            let pts: Vec<&ChartDataPoint> =
                pts.iter().filter(|p| p.series.as_deref() == *n).collect();
            let color = match pts.iter().find_map(|p| p.color.as_ref()) {
                Some(c) if multi => color_str(Some(c), tint),
                _ if i == 0 => tint.to_string(),
                _ => PIE_COLORS[i % PIE_COLORS.len()].to_string(),
            };
            Series {
                name: n.unwrap_or(""),
                color,
                pts,
            }
        })
        .collect()
}

/// Path segments after the first point: straight, monotone cubic (Fritsch–Carlson,
/// no overshoot) or step (hold the value until the next x).
fn curve_segments(xy: &[(f64, f64)], interp: Option<ChartInterpolation>) -> String {
    let mut out = String::new();
    match interp {
        Some(ChartInterpolation::Step) => {
            for w in xy.windows(2) {
                out.push_str(&format!(" H{:.2} V{:.2}", w[1].0, w[1].1));
            }
        }
        Some(ChartInterpolation::Monotone) if xy.len() > 2 => {
            let n = xy.len();
            let d: Vec<f64> = xy
                .windows(2)
                .map(|w| (w[1].1 - w[0].1) / (w[1].0 - w[0].0).max(1e-9))
                .collect();
            let mut m = vec![0.0; n];
            m[0] = d[0];
            m[n - 1] = d[n - 2];
            for i in 1..n - 1 {
                m[i] = if d[i - 1] * d[i] <= 0.0 {
                    0.0
                } else {
                    (d[i - 1] + d[i]) / 2.0
                };
            }
            for i in 0..n - 1 {
                if d[i] == 0.0 {
                    m[i] = 0.0;
                    m[i + 1] = 0.0;
                    continue;
                }
                let (a, b) = (m[i] / d[i], m[i + 1] / d[i]);
                let s = a * a + b * b;
                if s > 9.0 {
                    let t = 3.0 / s.sqrt();
                    m[i] = t * a * d[i];
                    m[i + 1] = t * b * d[i];
                }
            }
            for i in 0..n - 1 {
                let ((x0, y0), (x1, y1)) = (xy[i], xy[i + 1]);
                let dx = (x1 - x0) / 3.0;
                out.push_str(&format!(
                    " C{:.2},{:.2} {:.2},{:.2} {x1:.2},{y1:.2}",
                    x0 + dx,
                    y0 + m[i] * dx,
                    x1 - dx,
                    y1 - m[i + 1] * dx
                ));
            }
        }
        _ => {
            for &(x, y) in &xy[1.min(xy.len())..] {
                out.push_str(&format!(" L{x:.2},{y:.2}"));
            }
        }
    }
    out
}

/// Legend row (swatch + name) starting at `x`, baseline `y`.
fn legend_svg(items: &[(&str, &str)], x: f64, y: f64) -> String {
    let mut out = String::new();
    let mut cx = x;
    for (name, color) in items.iter().filter(|(n, _)| !n.is_empty()) {
        out.push_str(&format!(
            r##"<rect x="{}" y="{}" width="7" height="7" rx="1" fill="{color}"/><text x="{}" y="{}" font-size="8" fill="#999">{}</text>"##,
            num(cx),
            num(y - 7.0),
            num(cx + 10.0),
            num(y),
            esc(name)
        ));
        cx += 18.0 + 5.0 * name.chars().count() as f64;
    }
    out
}

/// y-axis gutter labels (`hi` at the top, `lo` at `plot_h`) and the axis line.
fn y_axis_svg(gutter: f64, plot_h: f64, lo: f64, hi: f64) -> String {
    format!(
        r##"<text x="{x}" y="8" font-size="8" fill="#999" text-anchor="end">{}</text><text x="{x}" y="{}" font-size="8" fill="#999" text-anchor="end">{}</text><line x1="{g}" y1="0" x2="{g}" y2="{}" stroke="#999" stroke-width="0.5"/>"##,
        num(hi),
        num(plot_h),
        num(lo),
        num(plot_h),
        x = num(gutter - 3.0),
        g = num(gutter),
    )
}

/// Chart → SVG. Defaults keep the original geometry; options add bands around it:
/// y axis = left gutter, x labels (bars by default, lines when `showXAxis: true`) and
/// legend = rows below. `yMin`/`yMax` fix the scale (values clamped into it).
/// Any v2 option (as `usesSwiftCharts` in RenderData.swift and `greedy` in layout.ts).
fn is_v2_chart(c: &ChartElement) -> bool {
    matches!(c.chart_type, ChartType::Point)
        || c.show_x_axis.is_some()
        || c.show_y_axis.is_some()
        || c.show_legend.is_some()
        || c.y_min.is_some()
        || c.y_max.is_some()
        || c.interpolation.is_some()
        || c.inner_radius.is_some()
        || c.chart_data.iter().any(|p| p.series.is_some())
}

fn chart_svg(c: &ChartElement) -> String {
    let pts = &c.chart_data;
    let tint = color_str(c.tint.as_ref(), DEFAULT_TINT);
    let series = chart_series(pts, &tint);
    let max_v = pts
        .iter()
        .map(|p| p.value)
        .fold(1.0_f64, f64::max)
        .max(1e-6);
    let lo = c.y_min.unwrap_or(0.0);
    let hi = c.y_max.unwrap_or(max_v);
    let span = (hi - lo).max(1e-6);
    let bounded = c.y_min.is_some() || c.y_max.is_some();
    let frac = |v: f64| {
        let f = (v - lo) / span;
        if bounded {
            f.clamp(0.0, 1.0)
        } else {
            f
        }
    };
    let show_y = c.show_y_axis == Some(true);
    let gutter = if show_y { 24.0 } else { 0.0 };
    let legend: Vec<(&str, &str)> = if c.show_legend == Some(true) {
        series.iter().map(|s| (s.name, s.color.as_str())).collect()
    } else {
        Vec::new()
    };
    let legend_h = if legend.iter().any(|(n, _)| !n.is_empty()) {
        12.0
    } else {
        0.0
    };
    // Plot in its own group when a gutter shifts it (keeps default output unchanged).
    let wrap = |body: String, plot_h: f64| {
        if show_y {
            format!(
                r#"{}<g transform="translate({gutter},0)">{body}</g>"#,
                y_axis_svg(gutter, plot_h, lo, hi)
            )
        } else {
            body
        }
    };

    match c.chart_type {
        ChartType::Line | ChartType::Area | ChartType::Point => {
            // A v2 chart fills its slot on Apple / desktop: rasterize it taller (5:3, not 10:3)
            // so the stretched Adaptive Card image is not a thin strip in a narrow column.
            let h = if is_v2_chart(c) { 120.0_f64 } else { 60.0_f64 };
            let w = c
                .style
                .aspect_ratio
                .filter(|r| *r > 0.0)
                .map_or(200.0, |r| h * r);
            let mut body = String::new();
            for s in &series {
                let n = s.pts.len().max(1);
                let tint = &s.color;
                let xy: Vec<(f64, f64)> = s
                    .pts
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        (
                            (i as f64 / (n - 1).max(1) as f64) * w,
                            h - frac(p.value) * h,
                        )
                    })
                    .collect();
                if xy.is_empty() {
                    continue;
                }
                let segs = curve_segments(&xy, c.interpolation);
                if matches!(c.chart_type, ChartType::Area) {
                    let area = format!(
                        "M0,{h:.2} L{:.2},{:.2}{segs} L{w:.2},{h:.2} Z",
                        xy[0].0, xy[0].1
                    );
                    body.push_str(&format!(
                        r#"<path d="{area}" fill="{tint}" opacity="0.3"/>"#
                    ));
                }
                if matches!(c.chart_type, ChartType::Point) {
                    for (x, y) in &xy {
                        body.push_str(&format!(
                            r#"<circle cx="{x:.2}" cy="{y:.2}" r="3" fill="{tint}"/>"#
                        ));
                    }
                } else {
                    let path = format!("M{:.2},{:.2}{segs}", xy[0].0, xy[0].1);
                    body.push_str(&format!(
                        r#"<path d="{path}" fill="none" stroke="{tint}" stroke-width="2"/>"#
                    ));
                }
            }
            let x_h = if c.show_x_axis == Some(true) {
                if let Some(first) = series.first() {
                    let n = first.pts.len();
                    for (i, p) in first.pts.iter().enumerate() {
                        let x = (i as f64 / (n.max(2) - 1) as f64) * w;
                        let anchor = match i {
                            0 => "start",
                            _ if i + 1 == n => "end",
                            _ => "middle",
                        };
                        body.push_str(&format!(
                            r##"<text x="{x:.2}" y="{}" font-size="8" fill="#999" text-anchor="{anchor}">{}</text>"##,
                            num(h + 10.0),
                            esc(&p.label)
                        ));
                    }
                }
                12.0
            } else {
                0.0
            };
            body.push_str(&legend_svg(&legend, 0.0, h + x_h + 10.0));
            let (tw, th) = (gutter + w, h + x_h + legend_h);
            format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {tw} {th}" width="{tw}" height="{th}">{}</svg>"#,
                wrap(body, h)
            )
        }
        ChartType::Pie => {
            let total: f64 = pts.iter().map(|p| p.value).sum::<f64>().max(1e-6);
            let r = 40.0;
            let cx = 50.0;
            let cy = 50.0;
            let ir = r * c.inner_radius.unwrap_or(0.0).clamp(0.0, 0.95);
            let mut ca = -90.0_f64;
            let mut body = String::new();
            let mut slices: Vec<(&str, String)> = Vec::new();
            for (i, p) in pts.iter().enumerate() {
                let angle = (p.value / total) * 360.0;
                let sr = ca * PI / 180.0;
                let er = (ca + angle) * PI / 180.0;
                let x1 = cx + r * sr.cos();
                let y1 = cy + r * sr.sin();
                let x2 = cx + r * er.cos();
                let y2 = cy + r * er.sin();
                let lf = if angle > 180.0 { 1 } else { 0 };
                let fill = color_str(p.color.as_ref(), PIE_COLORS[i % PIE_COLORS.len()]);
                if ir > 0.0 {
                    // Donut: outer arc forward, inner arc back.
                    let (ix1, iy1) = (cx + ir * sr.cos(), cy + ir * sr.sin());
                    let (ix2, iy2) = (cx + ir * er.cos(), cy + ir * er.sin());
                    body.push_str(&format!(
                        r#"<path d="M{x1:.2},{y1:.2} A{r},{r} 0 {lf},1 {x2:.2},{y2:.2} L{ix2:.2},{iy2:.2} A{ir:.2},{ir:.2} 0 {lf},0 {ix1:.2},{iy1:.2} Z" fill="{fill}"/>"#
                    ));
                } else {
                    body.push_str(&format!(
                        r#"<path d="M{cx},{cy} L{x1:.2},{y1:.2} A{r},{r} 0 {lf},1 {x2:.2},{y2:.2} Z" fill="{fill}"/>"#
                    ));
                }
                slices.push((&p.label, fill));
                ca += angle;
            }
            // Pie legend: one row per slice under the disc.
            let rows: Vec<&(&str, String)> = if c.show_legend == Some(true) {
                slices.iter().filter(|(l, _)| !l.is_empty()).collect()
            } else {
                Vec::new()
            };
            for (i, (label, fill)) in rows.iter().enumerate() {
                body.push_str(&legend_svg(
                    &[(*label, fill.as_str())],
                    8.0,
                    100.0 + 12.0 * i as f64 + 9.0,
                ));
            }
            if rows.is_empty() {
                format!(
                    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" width="80" height="80">{body}</svg>"#
                )
            } else {
                let vh = 100.0 + 12.0 * rows.len() as f64;
                format!(
                    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 {vh}" width="80" height="{}">{body}</svg>"#,
                    num(vh * 0.8)
                )
            }
        }
        ChartType::Bar => {
            let gap = 4.0;
            let h = 70.0;
            let w = c
                .style
                .aspect_ratio
                .filter(|r| *r > 0.0)
                .map_or(200.0, |r| h * r);
            let show_x = c.show_x_axis != Some(false);
            let label = |body: &mut String, x: f64, text: &str| {
                if show_x {
                    body.push_str(&format!(
                        r#"<text x="{:.2}" y="{:.2}" font-size="8" fill="{}" text-anchor="middle">{}</text>"#,
                        x,
                        h - 1.0,
                        "#999",
                        esc(text)
                    ));
                }
            };
            let mut body = String::new();
            if series.len() <= 1 {
                let n = pts.len().max(1) as f64;
                let bar_w = ((w - gap * (n + 1.0)) / n).max(2.0);
                for (i, p) in pts.iter().enumerate() {
                    let bh = (frac(p.value) * 60.0).max(2.0);
                    let x = gap + i as f64 * (bar_w + gap);
                    let y = h - 10.0 - bh;
                    let fill = color_str(p.color.as_ref(), &tint);
                    body.push_str(&format!(
                        r#"<rect x="{x:.2}" y="{y:.2}" width="{bar_w:.2}" height="{bh:.2}" fill="{fill}" rx="2"/>"#
                    ));
                    label(&mut body, x + bar_w / 2.0, &p.label);
                }
            } else {
                // Grouped: one slot per label, one bar per series inside it.
                let mut cats: Vec<&str> = Vec::new();
                for p in pts {
                    if !cats.contains(&p.label.as_str()) {
                        cats.push(&p.label);
                    }
                }
                let nc = cats.len().max(1) as f64;
                let group_w = ((w - gap * (nc + 1.0)) / nc).max(2.0);
                let bar_w = (group_w / series.len() as f64).max(1.0);
                for (ci, cat) in cats.iter().enumerate() {
                    let gx = gap + ci as f64 * (group_w + gap);
                    for (si, s) in series.iter().enumerate() {
                        let Some(p) = s.pts.iter().find(|p| p.label == *cat) else {
                            continue;
                        };
                        let bh = (frac(p.value) * 60.0).max(2.0);
                        let x = gx + si as f64 * bar_w;
                        let y = h - 10.0 - bh;
                        let fill = color_str(p.color.as_ref(), &s.color);
                        body.push_str(&format!(
                            r#"<rect x="{x:.2}" y="{y:.2}" width="{bar_w:.2}" height="{bh:.2}" fill="{fill}" rx="1"/>"#
                        ));
                    }
                    label(&mut body, gx + group_w / 2.0, cat);
                }
            }
            body.push_str(&legend_svg(&legend, 0.0, h + 10.0));
            let (tw, th) = (gutter + w, h + legend_h);
            format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {tw} {th}" width="{tw}" height="{th}">{}</svg>"#,
                wrap(body, 60.0)
            )
        }
    }
}

fn canvas_svg(width: f64, height: f64, elements: &[CanvasDrawCommand]) -> String {
    let mut body = String::new();
    for cmd in elements {
        match cmd {
            CanvasDrawCommand::Circle {
                cx,
                cy,
                r,
                fill,
                stroke,
                stroke_width,
            } => {
                body.push_str(&format!(
                    r#"<circle cx="{cx}" cy="{cy}" r="{r}" fill="{}" stroke="{}" stroke-width="{}"/>"#,
                    color_str(fill.as_ref(), "none"),
                    color_str(stroke.as_ref(), "none"),
                    stroke_width.unwrap_or(1.0)
                ));
            }
            CanvasDrawCommand::Line {
                x1,
                y1,
                x2,
                y2,
                stroke,
                stroke_width,
                line_cap,
            } => {
                body.push_str(&format!(
                    r#"<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="{}" stroke-width="{}" stroke-linecap="{}"/>"#,
                    color_str(stroke.as_ref(), "#ffffff"),
                    stroke_width.unwrap_or(1.0),
                    line_cap.as_deref().unwrap_or("butt")
                ));
            }
            CanvasDrawCommand::Rect {
                x,
                y,
                width: rw,
                height: rh,
                fill,
                stroke,
                stroke_width,
                corner_radius,
            } => {
                let rx = corner_radius.unwrap_or(0.0);
                body.push_str(&format!(
                    r#"<rect x="{x}" y="{y}" width="{rw}" height="{rh}" rx="{rx}" ry="{rx}" fill="{}" stroke="{}" stroke-width="{}"/>"#,
                    color_str(fill.as_ref(), "none"),
                    color_str(stroke.as_ref(), "none"),
                    stroke_width.unwrap_or(1.0)
                ));
            }
            CanvasDrawCommand::Arc {
                cx,
                cy,
                r,
                start_angle,
                end_angle,
                fill,
                stroke,
                stroke_width,
            } => {
                let sa = start_angle * PI / 180.0;
                let ea = end_angle * PI / 180.0;
                let sx = cx + r * sa.cos();
                let sy = cy + r * sa.sin();
                let ex = cx + r * ea.cos();
                let ey = cy + r * ea.sin();
                let lf = if (ea - sa).abs() > PI { 1 } else { 0 };
                let fill_s = color_str(fill.as_ref(), "none");
                let d = if fill_s != "none" {
                    format!("M{cx},{cy} L{sx:.2},{sy:.2} A{r},{r} 0 {lf} 1 {ex:.2},{ey:.2} Z")
                } else {
                    format!("M{sx:.2},{sy:.2} A{r},{r} 0 {lf} 1 {ex:.2},{ey:.2}")
                };
                body.push_str(&format!(
                    r#"<path d="{d}" fill="{fill_s}" stroke="{}" stroke-width="{}"/>"#,
                    color_str(stroke.as_ref(), "none"),
                    stroke_width.unwrap_or(1.0)
                ));
            }
            CanvasDrawCommand::Text {
                x,
                y,
                content,
                font_size,
                color,
                anchor,
            } => {
                let anchor = match anchor.as_deref() {
                    Some("middle") => "middle",
                    Some("end") => "end",
                    _ => "start",
                };
                body.push_str(&format!(
                    r#"<text x="{x}" y="{y}" font-size="{}" fill="{}" text-anchor="{anchor}">{}</text>"#,
                    font_size.unwrap_or(12.0),
                    color_str(color.as_ref(), "#ffffff"),
                    esc(content)
                ));
            }
            CanvasDrawCommand::Path {
                d,
                fill,
                stroke,
                stroke_width,
            } => {
                body.push_str(&format!(
                    r#"<path d="{}" fill="{}" stroke="{}" stroke-width="{}"/>"#,
                    esc(d),
                    color_str(fill.as_ref(), "none"),
                    color_str(stroke.as_ref(), "none"),
                    stroke_width.unwrap_or(1.0)
                ));
            }
        }
    }
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">{body}</svg>"#
    )
}

/// Inputs of [`gauge_svg`] (gauge and circular progress).
struct GaugeSpec<'a> {
    value: f64,
    min: f64,
    max: f64,
    tint: Option<&'a ColorValue>,
    style: Option<&'a GaugeStyle>,
    current: Option<&'a str>,
    label: Option<&'a str>,
    /// Ring track color (default `#e0e0e0`).
    track: Option<&'a ColorValue>,
    /// Ring width in points (default 4 view units).
    thickness: Option<f64>,
    min_label: Option<&'a str>,
    max_label: Option<&'a str>,
}

fn gauge_svg(g: &GaugeSpec) -> String {
    let GaugeSpec {
        value,
        min,
        max,
        current,
        label,
        min_label,
        max_label,
        ..
    } = *g;
    let tint = color_str(g.tint, DEFAULT_TINT);
    let track = color_str(g.track, "#e0e0e0");
    let pct = (((value - min) / (max - min).max(1e-6)) * 100.0).clamp(0.0, 100.0);
    let has_ends = min_label.is_some() || max_label.is_some();
    // Labels at both ends: `start` anchored at `x0`, `end` anchored at `x1`, baseline `y`.
    let ends = |x0: f64, x1: f64, y: f64, size: f64| {
        let mut out = String::new();
        for (text, x, anchor) in [(min_label, x0, "start"), (max_label, x1, "end")] {
            if let Some(t) = text {
                out.push_str(&format!(
                    r#"<text x="{}" y="{}" font-size="{}" fill="{tint}" opacity="0.7" text-anchor="{anchor}">{}</text>"#,
                    num(x),
                    num(y),
                    num(size),
                    esc(t)
                ));
            }
        }
        out
    };

    if matches!(g.style, Some(GaugeStyle::Linear)) {
        let mut body = String::new();
        if let Some(l) = label {
            body.push_str(&format!(
                r#"<text x="0" y="10" font-size="10" fill="{tint}" opacity="0.7">{}</text>"#,
                esc(l)
            ));
        }
        if let Some(c) = current {
            body.push_str(&format!(
                r#"<text x="120" y="10" font-size="11" font-weight="600" fill="{tint}" text-anchor="end">{}</text>"#,
                esc(c)
            ));
        }
        body.push_str(&format!(
            r#"<rect x="0" y="16" width="120" height="6" rx="3" fill="{track}"/>"#
        ));
        let fw = (120.0 * pct / 100.0).max(0.0);
        body.push_str(&format!(
            r#"<rect x="0" y="16" width="{fw:.2}" height="6" rx="3" fill="{tint}"/>"#
        ));
        let h = if has_ends {
            body.push_str(&ends(0.0, 120.0, 33.0, 8.0));
            38
        } else {
            28
        };
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 120 {h}" width="120" height="{h}">{body}</svg>"#
        )
    } else {
        let sw = num(g.thickness.map_or(4.0, |t| t * 36.0 / 56.0));
        let mut body = String::new();
        body.push_str(&format!(
            r#"<path d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831" fill="none" stroke="{track}" stroke-width="{sw}"/>"#
        ));
        body.push_str(&format!(
            r#"<path d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831" fill="none" stroke="{tint}" stroke-width="{sw}" stroke-dasharray="{pct:.1}, 100" stroke-linecap="round"/>"#
        ));
        if let Some(c) = current {
            let white = "#ffffff";
            body.push_str(&format!(
                r#"<text x="18" y="20" font-size="8" font-weight="600" fill="{white}" text-anchor="middle">{}</text>"#,
                esc(c)
            ));
        }
        let ends_h = if has_ends {
            body.push_str(&ends(1.0, 35.0, 42.0, 6.0));
            8.0
        } else {
            0.0
        };
        let label_h = if label.is_some() { 14.0 } else { 0.0 };
        if let Some(l) = label {
            // Lighten label vs ring tint so it stays readable on dark widget goldens.
            let label_fill = "#ffffff";
            body.push_str(&format!(
                r#"<text x="18" y="{}" font-size="8" fill="{label_fill}" text-anchor="middle">{}</text>"#,
                num(48.0 + ends_h),
                esc(l)
            ));
        }
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 36 {:.0}" width="56" height="{:.0}">{body}</svg>"#,
            36.0 + label_h + ends_h,
            56.0 + label_h + ends_h
        )
    }
}

/// Linear progress bar with a track color and thickness (AC columns cannot draw either).
fn progress_bar_svg(
    frac: f64,
    tint: Option<&ColorValue>,
    track: Option<&ColorValue>,
    thickness: Option<f64>,
) -> String {
    let tint = color_str(tint, DEFAULT_TINT);
    let track = color_str(track, "#e0e0e0");
    let th = thickness.unwrap_or(6.0).max(1.0);
    let (t, r) = (num(th), num(th / 2.0));
    let mut body =
        format!(r#"<rect x="0" y="0" width="200" height="{t}" rx="{r}" ry="{r}" fill="{track}"/>"#);
    if frac > 0.0 {
        body.push_str(&format!(
            r#"<rect x="0" y="0" width="{}" height="{t}" rx="{r}" ry="{r}" fill="{tint}"/>"#,
            num(200.0 * frac)
        ));
    }
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 {t}" width="200" height="{t}">{body}</svg>"#
    )
}

/// Inline image clipped to a rounded rectangle (`rx` = corner radius; `h / 2` = capsule).
pub fn clipped_image_svg(href: &str, w: f64, h: f64, rx: f64) -> String {
    let (w, h, rx) = (num(w), num(h), num(rx));
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="{w}" height="{h}" viewBox="0 0 {w} {h}"><defs><clipPath id="c"><rect width="{w}" height="{h}" rx="{rx}" ry="{rx}"/></clipPath></defs><image width="{w}" height="{h}" preserveAspectRatio="xMidYMid slice" clip-path="url(#c)" xlink:href="{}"/></svg>"#,
        esc(href)
    )
}

/// Shape frame like the Swift / Android renderers: `frame.width/height` (else
/// `minWidth/minHeight`), the missing side derived from `aspectRatio`.
fn shape_frame(s: &ShapeElement) -> (Option<f64>, Option<f64>) {
    let f = s.style.frame.as_ref();
    let mut w = f.and_then(|f| f.width.or(f.min_width));
    let mut h = f.and_then(|f| f.height.or(f.min_height));
    if let Some(r) = s.style.aspect_ratio.filter(|r| *r > 0.0) {
        match (w, h) {
            (Some(fw), None) => h = Some(fw / r),
            (None, Some(fh)) => w = Some(fh * r),
            _ => {}
        }
    }
    (w, h)
}

fn shape_svg(s: &ShapeElement) -> String {
    let fill_s = color_str(s.fill.as_ref(), DEFAULT_TINT);
    let stroke_s = color_str(s.stroke.as_ref(), "none");
    let sw = s.stroke_width.unwrap_or(1.0);
    let size = s.size.unwrap_or(24.0);
    let rect = |w: f64, h: f64, rx: f64| {
        let rxa = if rx > 0.0 {
            format!(r#" rx="{rx}" ry="{rx}""#)
        } else {
            String::new()
        };
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}"><rect x="0" y="0" width="{w}" height="{h}"{rxa} fill="{fill_s}" stroke="{stroke_s}" stroke-width="{sw}"/></svg>"#
        )
    };
    match s.shape_type {
        ShapeType::Circle => {
            let r = size / 2.0;
            format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 {size} {size}"><circle cx="{r}" cy="{r}" r="{r}" fill="{fill_s}" stroke="{stroke_s}" stroke-width="{sw}"/></svg>"#
            )
        }
        ShapeType::Capsule => rect(size * 2.0, size, size / 2.0),
        ShapeType::RoundedRectangle => {
            let (fw, fh) = shape_frame(s);
            let side = s.size.unwrap_or(fw.unwrap_or(24.0).min(fh.unwrap_or(24.0)));
            let (w, h) = (fw.unwrap_or(side), fh.unwrap_or(side));
            rect(w, h, s.style.corner_radius.unwrap_or(0.0))
        }
        ShapeType::Ellipse => {
            // size → circle-sized; frame / aspectRatio → oval.
            let (fw, fh) = shape_frame(s);
            let w = fw.or(s.size).or(fh).unwrap_or(24.0);
            let h = fh.or(s.size).or(fw).unwrap_or(24.0);
            format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}"><ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{fill_s}" stroke="{stroke_s}" stroke-width="{sw}"/></svg>"#,
                w / 2.0,
                h / 2.0,
                w / 2.0,
                h / 2.0
            )
        }
        ShapeType::Rectangle => rect(size, size, 0.0),
    }
}

#[cfg(test)]
mod snapshot_tests {
    //! Exact SVG output per element: any change to the geometry shows up as a diff.
    //! `UPDATE_SNAPSHOTS=1 cargo test --lib rasterize::snapshot_tests` rewrites
    //! `tests/snapshots/rasterize/*.svg` after an intended change.
    use super::*;
    use serde_json::json;

    fn el(v: serde_json::Value) -> WidgetElement {
        serde_json::from_value(v).unwrap()
    }

    fn cases() -> Vec<(&'static str, String)> {
        let mut out: Vec<(&'static str, String)> = Vec::new();
        let pts = json!([{"label": "a", "value": 3}, {"label": "b", "value": 7, "color": "#00ff00"}, {"label": "c", "value": 5}]);
        for (name, ty) in [
            ("chart-bar", "bar"),
            ("chart-line", "line"),
            ("chart-area", "area"),
            ("chart-pie", "pie"),
        ] {
            let e =
                el(json!({"type": "chart", "chartType": ty, "chartData": pts, "tint": "accent"}));
            out.push((name, element_to_svg(&e).unwrap()));
        }
        let single = el(
            json!({"type": "chart", "chartType": "line", "chartData": [{"label": "x", "value": 2}]}),
        );
        out.push(("chart-line-single", element_to_svg(&single).unwrap()));
        let zero = el(
            json!({"type": "chart", "chartType": "pie", "chartData": [{"label": "x", "value": 0}, {"label": "y", "value": 0}]}),
        );
        out.push(("chart-pie-zero", element_to_svg(&zero).unwrap()));
        let neg = el(
            json!({"type": "chart", "chartType": "bar", "chartData": [{"label": "x", "value": -2}, {"label": "y", "value": 4}]}),
        );
        out.push(("chart-bar-negative", element_to_svg(&neg).unwrap()));
        let big = el(
            json!({"type": "chart", "chartType": "pie", "chartData": [{"label": "x", "value": 3}, {"label": "y", "value": 1}]}),
        );
        out.push(("chart-pie-big-slice", element_to_svg(&big).unwrap()));
        let half = el(
            json!({"type": "chart", "chartType": "pie", "chartData": [{"label": "x", "value": 1}, {"label": "y", "value": 1}]}),
        );
        out.push(("chart-pie-halves", element_to_svg(&half).unwrap()));
        out.push((
            "chart-empty",
            element_to_svg(&el(
                json!({"type": "chart", "chartType": "bar", "chartData": []}),
            ))
            .unwrap_or_default(),
        ));

        let canvas = el(
            json!({"type": "canvas", "width": 120, "height": 80, "elements": [
                {"draw": "circle", "cx": 20, "cy": 20, "r": 10, "fill": "red", "stroke": "#fff", "strokeWidth": 2},
                {"draw": "line", "x1": 0, "y1": 0, "x2": 50, "y2": 40, "stroke": "#123", "strokeWidth": 3, "lineCap": "round"},
                {"draw": "rect", "x": 5, "y": 6, "width": 30, "height": 20, "fill": {"light": "#eee", "dark": "#111"}, "cornerRadius": 4},
                {"draw": "arc", "cx": 60, "cy": 40, "r": 15, "startAngle": 0, "endAngle": 270, "stroke": "good", "strokeWidth": 4},
                {"draw": "arc", "cx": 60, "cy": 40, "r": 10, "startAngle": 90, "endAngle": 120, "fill": "warning"},
                {"draw": "arc", "cx": 60, "cy": 40, "r": 12, "startAngle": 0, "endAngle": 180, "stroke": "#000"},
                {"draw": "text", "x": 10, "y": 70, "content": "a<b & \"c\"", "fontSize": 9, "color": "label", "anchor": "middle"},
                {"draw": "text", "x": 110, "y": 70, "content": "end", "anchor": "end"},
            {"draw": "path", "d": "M0 0 L10 10 Z", "fill": "none", "stroke": "danger"}
            ]}),
        );
        out.push(("canvas-all", element_to_svg(&canvas).unwrap()));

        for (name, style, value) in [
            ("gauge-circular", "circular", 0.25),
            ("gauge-linear", "linear", 0.6),
            ("gauge-default", "", 1.5),
        ] {
            let mut g = json!({"type": "gauge", "value": value, "min": 0, "max": 1, "label": "L", "currentValueLabel": "25%", "tint": "#ff0000"});
            if !style.is_empty() {
                g["gaugeStyle"] = json!(style);
            }
            out.push((name, element_to_svg(&el(g)).unwrap()));
        }
        for (name, total) in [
            ("progress-circular", 2.0),
            ("progress-circular-zero-total", 0.0),
        ] {
            let p = el(
                json!({"type": "progress", "value": 0.5, "total": total, "barStyle": "circular", "label": "P", "tint": "#0f0"}),
            );
            out.push((name, element_to_svg(&p).unwrap()));
        }
        let ranged = el(json!({"type": "gauge", "value": 30, "min": 20, "max": 40}));
        out.push(("gauge-range", element_to_svg(&ranged).unwrap()));

        for (name, shape) in [
            ("shape-circle", "circle"),
            ("shape-capsule", "capsule"),
            ("shape-rect", "rectangle"),
        ] {
            let e = el(
                json!({"type": "shape", "shapeType": shape, "fill": "#abcdef", "stroke": "#000", "strokeWidth": 2, "size": 40,
                "frame": {"width": 60, "height": 30}, "cornerRadius": 6}),
            );
            out.push((name, element_to_svg(&e).unwrap()));
        }

        for (name, ty, dir) in [
            ("gradient-linear-ttb", "linear", "topToBottom"),
            ("gradient-linear-btt", "linear", "bottomToTop"),
            ("gradient-linear-ltr", "linear", "leadingToTrailing"),
            ("gradient-linear-rtl", "linear", "trailingToLeading"),
            (
                "gradient-linear-diag",
                "linear",
                "topLeadingToBottomTrailing",
            ),
            (
                "gradient-linear-diag2",
                "linear",
                "topTrailingToBottomLeading",
            ),
            ("gradient-radial", "radial", "topToBottom"),
            ("gradient-angular", "angular", "topToBottom"),
        ] {
            let g: GradientConfig = serde_json::from_value(json!({"gradientType": ty, "colors": ["#ff0000", "#00ff00", "#0000ff"], "direction": dir})).unwrap();
            out.push((name, gradient_to_svg(&g, 100.0, 50.0)));
        }

        let z = el(
            json!({"type": "zstack", "frame": {"width": 80, "height": 80}, "background": "#222", "cornerRadius": 8, "children": [
                {"type": "shape", "shapeType": "circle", "fill": "#f00", "size": 40},
                {"type": "gauge", "value": 0.5, "gaugeStyle": "circular"},
                {"type": "text", "content": "ignored"}
            ]}),
        );
        out.push(("zstack", element_to_svg(&z).unwrap_or_default()));
        let zs = el(json!({"type": "zstack", "children": [
            {"type": "shape", "shapeType": "circle", "fill": "#f00", "size": 40},
            {"type": "gauge", "value": 0.5}
        ]}));
        out.push((
            "zstack-svg-only",
            element_to_svg(&zs).unwrap_or_else(|| "none".into()),
        ));
        let zt = el(
            json!({"type": "zstack", "children": [{"type": "text", "content": "T", "fontSize": 12}]}),
        );
        out.push((
            "zstack-text",
            element_to_svg(&zt).unwrap_or_else(|| "none".into()),
        ));
        let zi = el(
            json!({"type": "zstack", "children": [{"type": "image", "systemName": "star.fill", "size": 20}]}),
        );
        out.push((
            "zstack-image",
            element_to_svg(&zi).unwrap_or_else(|| "none".into()),
        ));
        let zti = el(
            json!({"type": "zstack", "children": [{"type": "text", "content": "A"}, {"type": "image", "systemName": "star"}, {"type": "text", "content": "B"}]}),
        );
        out.push((
            "zstack-text-image",
            element_to_svg(&zti).unwrap_or_else(|| "none".into()),
        ));

        // v2: series, axes, legend, scale bounds, curves, donut, point.
        let two = json!([
            {"label": "M", "value": 2, "series": "A"}, {"label": "T", "value": 6, "series": "A"},
            {"label": "W", "value": 4, "series": "A"}, {"label": "M", "value": 5, "series": "B", "color": "#f472b6"},
            {"label": "T", "value": 3, "series": "B", "color": "#f472b6"}, {"label": "W", "value": 12, "series": "B", "color": "#f472b6"}
        ]);
        for (name, extra) in [
            (
                "chart-series-line-legend",
                json!({"chartType": "line", "showLegend": true, "showYAxis": true, "yMin": 0, "yMax": 10, "interpolation": "monotone"}),
            ),
            (
                "chart-area-step",
                json!({"chartType": "area", "interpolation": "step", "showXAxis": true}),
            ),
            (
                "chart-bar-grouped",
                json!({"chartType": "bar", "showLegend": true, "yMax": 10, "showYAxis": true}),
            ),
            (
                "chart-point-series",
                json!({"chartType": "point", "showXAxis": false, "tint": "#22c55e"}),
            ),
        ] {
            let mut c = json!({"type": "chart", "chartData": two});
            c.as_object_mut()
                .unwrap()
                .extend(extra.as_object().unwrap().clone());
            out.push((name, element_to_svg(&el(c)).unwrap()));
        }
        out.push((
            "chart-donut-legend",
            element_to_svg(&el(json!({"type": "chart", "chartType": "pie", "innerRadius": 0.6, "showLegend": true,
                "chartData": [{"label": "a", "value": 3, "color": "#0ea5e9"}, {"label": "b", "value": 2}, {"label": "c", "value": 1}]})))
            .unwrap(),
        ));
        out.push((
            "chart-line-aspect",
            element_to_svg(&el(
                json!({"type": "chart", "chartType": "line", "aspectRatio": 2, "chartData": pts, "showXAxis": false, "showLegend": true}),
            ))
            .unwrap(),
        ));
        let stops: GradientConfig = serde_json::from_value(json!({"gradientType": "linear",
            "stops": [{"color": "#0f172a", "location": 0}, {"color": "#1e3a8a", "location": 0.7}, {"color": "#7c3aed", "location": 1}],
            "startPoint": {"x": 0, "y": 0}, "endPoint": {"x": 1, "y": 0.5}})).unwrap();
        out.push((
            "gradient-stops-points",
            gradient_to_svg(&stops, 100.0, 50.0),
        ));
        let radial: GradientConfig = serde_json::from_value(json!({"gradientType": "radial",
            "stops": [{"color": "#fff", "location": 0.2}, {"color": "#000", "location": 1}],
            "startPoint": {"x": 0.25, "y": 0.75}}))
        .unwrap();
        out.push((
            "gradient-radial-center",
            gradient_to_svg(&radial, 100.0, 50.0),
        ));
        let start_only: GradientConfig = serde_json::from_value(
            json!({"gradientType": "angular", "colors": ["#f00", "#00f"], "startPoint": {"x": 1, "y": 0}}),
        )
        .unwrap();
        out.push((
            "gradient-angular-start",
            gradient_to_svg(&start_only, 100.0, 50.0),
        ));
        for (name, shape) in [
            (
                "shape-rounded-rect",
                json!({"type": "shape", "shapeType": "roundedRectangle", "fill": "#22c55e", "cornerRadius": 10, "size": 44}),
            ),
            (
                "shape-rounded-aspect",
                json!({"type": "shape", "shapeType": "roundedRectangle", "fill": "#22c55e", "cornerRadius": 4, "aspectRatio": 2, "frame": {"minHeight": 20}}),
            ),
            (
                "shape-ellipse",
                json!({"type": "shape", "shapeType": "ellipse", "fill": "#f59e0b", "size": 44}),
            ),
            (
                "shape-ellipse-frame",
                json!({"type": "shape", "shapeType": "ellipse", "fill": "#f59e0b", "aspectRatio": 2, "frame": {"width": 60}}),
            ),
        ] {
            out.push((name, element_to_svg(&el(shape)).unwrap()));
        }
        for (name, p) in [
            (
                "progress-bar-track",
                json!({"type": "progress", "value": 0.3, "tint": "#22c55e", "trackColor": "#374151", "thickness": 8}),
            ),
            (
                "progress-bar-empty",
                json!({"type": "progress", "value": 0, "total": 0, "trackColor": "#374151"}),
            ),
            (
                "progress-circular-track",
                json!({"type": "progress", "value": 0.5, "barStyle": "circular", "trackColor": "#374151", "thickness": 8, "label": "P"}),
            ),
            (
                "gauge-linear-ends",
                json!({"type": "gauge", "value": 0.4, "gaugeStyle": "linear", "minLabel": "0", "maxLabel": "100", "tint": "#f59e0b"}),
            ),
            (
                "gauge-circular-ends",
                json!({"type": "gauge", "value": 0.4, "minLabel": "lo", "maxLabel": "hi", "label": "L", "currentValueLabel": "40"}),
            ),
        ] {
            out.push((name, element_to_svg(&el(p)).unwrap()));
        }
        out.push((
            "clip-rounded-image",
            clipped_image_svg("data:image/png;base64,AAAA", 40.0, 40.0, 8.0),
        ));
        out
    }

    #[test]
    fn svg_snapshots() {
        let dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots/rasterize");
        let update = std::env::var("UPDATE_SNAPSHOTS").is_ok_and(|v| v == "1");
        let mut drift = Vec::new();
        for (name, svg) in cases() {
            let path = dir.join(format!("{name}.svg"));
            if update || !path.exists() {
                std::fs::create_dir_all(&dir).unwrap();
                std::fs::write(&path, &svg).unwrap();
                continue;
            }
            if std::fs::read_to_string(&path).unwrap() != svg {
                drift.push(name);
            }
        }
        assert!(
            drift.is_empty(),
            "SVG drift (UPDATE_SNAPSHOTS=1 to accept): {drift:?}"
        );
    }

    #[test]
    fn helpers() {
        assert_eq!(
            regex_lite_viewbox(r#"<svg viewBox="1 2.5 30 40">"#),
            Some((1.0, 2.5, 30.0, 40.0))
        );
        assert_eq!(
            extract_viewbox(r#"<svg viewBox="0 0 10 20" width="1">"#),
            Some((0.0, 0.0, 10.0, 20.0))
        );
        assert_eq!(regex_lite_viewbox(r#"<svg viewBox="0 0 10">"#), None);
        assert_eq!(regex_lite_viewbox("<svg>"), None);
        assert_eq!(regex_lite_viewbox(r#"<svg viewBox="0 0 10 20"#), None);
        assert_eq!(
            strip_outer_svg(r#" <svg a="1"><rect/><g/></svg> "#),
            "<rect/><g/>"
        );
        assert_eq!(strip_outer_svg("plain"), "plain");
        assert_eq!(esc(r#"<a & "b">"#), "&lt;a &amp; &quot;b&quot;&gt;");
        for (i, o) in [
            ("#fff", "#fff"),
            (" rgb(1,2,3) ", "rgb(1,2,3)"),
            ("Accent", "#2196F3"),
            ("blue", "#2196F3"),
            ("success", "#4CAF50"),
            ("orange", "#FF9800"),
            ("error", "#F44336"),
            ("label", "#212121"),
            ("secondaryLabel", "#FAFAFA"),
            ("purple", "purple"),
        ] {
            assert_eq!(normalize_color(i), o, "{i}");
        }
        assert_eq!(normalize_color("  "), DEFAULT_TINT);
        assert_eq!(color_str(None, "#abc"), "#abc");
        assert_eq!(
            color_str(Some(&ColorValue::Solid("red".into())), "#abc"),
            "#F44336"
        );
        assert_eq!(
            color_str(
                Some(&ColorValue::Adaptive {
                    light: "#111".into(),
                    dark: "#222".into()
                }),
                "#abc"
            ),
            "#111"
        );
    }

    /// Element → SVG → PNG chain: chart bars / shape circle as SVG, then line chart and
    /// gradient as PNG data URIs; non-graphic elements and non-SVG input are refused.
    #[test]
    fn elements_render_to_svg_then_png() {
        let bar = el(json!({"type": "chart", "chartType": "bar",
            "chartData": [{"label": "a", "value": 3}, {"label": "b", "value": 5}]}));
        let svg = element_to_svg(&bar).unwrap();
        assert!(svg.contains("<svg") && svg.contains("<rect"), "{svg}");
        let circle = el(json!({"type": "shape", "shapeType": "circle", "size": 32}));
        assert!(element_to_svg(&circle).unwrap().contains("<circle"));

        #[cfg(feature = "rasterize")]
        {
            let line = el(json!({"type": "chart", "chartType": "line",
                "chartData": [{"label": "a", "value": 1}, {"label": "b", "value": 2}]}));
            let uri = element_to_png_data_uri(&line).unwrap();
            assert!(uri.starts_with("data:image/png;base64,") && uri.len() > 64);
            let g: GradientConfig = serde_json::from_value(
                json!({"gradientType": "linear", "colors": ["#f00", "#00f"]}),
            )
            .unwrap();
            let uri = gradient_to_png_data_uri(&g).unwrap();
            assert!(uri.starts_with("data:image/png;base64,") && uri.len() > 60);
            assert!(element_to_png_data_uri(&el(json!({"type": "text", "content": "x"}))).is_err());
            assert!(svg_to_data_uri("not svg").is_err());
        }
    }
}

#[cfg(test)]
mod solid_png_tests {
    use super::solid_png_data_uri;

    /// Given a colour, when encoded as a 1x1 PNG, then a real PNG decoder reads that pixel back.
    #[test]
    fn solid_png_round_trips() {
        assert_eq!(solid_png_data_uri("red"), None);
        assert_eq!(solid_png_data_uri("#12345"), None);
        for (hex, rgba) in [
            ("#1a1a2e", [0x1a, 0x1a, 0x2e, 0xff]),
            ("#4caf5040", [0x4c, 0xaf, 0x50, 0x40]),
            ("#fff", [0xff, 0xff, 0xff, 0xff]),
        ] {
            let uri = solid_png_data_uri(hex).unwrap();
            let b64 = uri.strip_prefix("data:image/png;base64,").unwrap();
            #[cfg(feature = "rasterize")]
            {
                use base64::Engine;
                let png = base64::engine::general_purpose::STANDARD
                    .decode(b64)
                    .unwrap();
                let px = resvg::tiny_skia::Pixmap::decode_png(&png).unwrap();
                assert_eq!((px.width(), px.height()), (1, 1));
                // tiny-skia stores premultiplied colour.
                let c = px.pixel(0, 0).unwrap().demultiply();
                assert_eq!([c.red(), c.green(), c.blue(), c.alpha()], rgba, "{hex}");
            }
            #[cfg(not(feature = "rasterize"))]
            let _ = (b64, rgba);
        }
    }
}

#[cfg(test)]
mod zstack_size_tests {
    use super::zstack_size;
    use crate::models::{ElementStyle, WidgetElement};

    fn el(v: serde_json::Value) -> WidgetElement {
        serde_json::from_value(v).unwrap()
    }

    /// Given zstacks, when sized, then a badge keeps its shape size, a layout zstack fills its
    /// slot (None), and a frame wins.
    #[test]
    fn badge_layout_and_frame() {
        let badge = [
            el(serde_json::json!({"type": "shape", "shapeType": "rectangle", "size": 20})),
            el(serde_json::json!({"type": "text", "content": "S"})),
        ];
        let none = ElementStyle::default();
        assert_eq!(zstack_size(&badge, &none), Some((20.0, 20.0)));
        let layout = [el(serde_json::json!({"type": "vstack", "children": []}))];
        assert_eq!(zstack_size(&layout, &none), None);
        let framed: ElementStyle =
            serde_json::from_value(serde_json::json!({"frame": {"width": 40, "height": 30}}))
                .unwrap();
        assert_eq!(zstack_size(&layout, &framed), Some((40.0, 30.0)));
    }
}
