//! SVG builders + optional PNG rasterize for Adaptive Cards Image data URIs.
//!
//! Used for `chart` / `canvas` / `gauge` nodes that Adaptive Cards cannot express natively.

use crate::models::{
    CanvasDrawCommand, ChartDataPoint, ChartType, ColorValue, GaugeStyle, ShapeType, WidgetElement, GaugeElement, ChartElement, ShapeElement, CanvasElement,
};
use std::f64::consts::PI;

const DEFAULT_TINT: &str = "#4CAF50";
const PIE_COLORS: &[&str] = &[
    "#3b82f6", "#22c55e", "#f97316", "#ef4444", "#a855f7", "#eab308", "#ec4899", "#14b8a6",
];

/// Build an SVG document for chart/canvas/gauge/shape; `None` for other element types.
pub fn element_to_svg(el: &WidgetElement) -> Option<String> {
    match el {
        WidgetElement::Chart(ChartElement {
            chart_type,
            chart_data,
            tint,
            ..
        }) => Some(chart_svg(chart_type, chart_data, tint.as_ref())),
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
            ..
        }) => Some(gauge_svg(
            *value,
            min.unwrap_or(0.0),
            max.unwrap_or(1.0),
            tint.as_ref(),
            gauge_style.as_ref(),
            current_value_label.as_deref(),
            label.as_deref(),
        )),
        WidgetElement::Shape(ShapeElement {
            shape_type,
            fill,
            stroke,
            stroke_width,
            size,
            ..
        }) => Some(shape_svg(
            shape_type,
            fill.as_ref(),
            stroke.as_ref(),
            stroke_width.unwrap_or(1.0),
            size.unwrap_or(24.0),
        )),
        _ => None,
    }
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

/// Convenience: element → PNG data URI.
pub fn element_to_png_data_uri(el: &WidgetElement) -> Result<String, String> {
    let svg =
        element_to_svg(el).ok_or_else(|| "element is not chart/canvas/gauge/shape".to_string())?;
    svg_to_data_uri(&svg)
}

#[cfg(feature = "rasterize")]
fn svg_to_data_uri_impl(svg: &str) -> Result<String, String> {
    use base64::Engine;
    let mut opts = resvg::usvg::Options::default();
    opts.fontdb_mut().load_system_fonts();
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

fn chart_svg(chart_type: &ChartType, pts: &[ChartDataPoint], tint: Option<&ColorValue>) -> String {
    let tint = color_str(tint, DEFAULT_TINT);
    let max_v = pts
        .iter()
        .map(|p| p.value)
        .fold(1.0_f64, f64::max)
        .max(1e-6);

    match chart_type {
        ChartType::Line | ChartType::Area => {
            let w = 200.0_f64;
            let h = 60.0_f64;
            let n = pts.len().max(1);
            let mut path = String::new();
            for (i, p) in pts.iter().enumerate() {
                let x = (i as f64 / (n - 1).max(1) as f64) * w;
                let y = h - (p.value / max_v) * h;
                if i == 0 {
                    path.push_str(&format!("M{x:.2},{y:.2}"));
                } else {
                    path.push_str(&format!(" L{x:.2},{y:.2}"));
                }
            }
            let mut body = String::new();
            if matches!(chart_type, ChartType::Area) {
                let mut area = format!("M0,{h:.2}");
                for (i, p) in pts.iter().enumerate() {
                    let x = (i as f64 / (n - 1).max(1) as f64) * w;
                    let y = h - (p.value / max_v) * h;
                    area.push_str(&format!(" L{x:.2},{y:.2}"));
                }
                area.push_str(&format!(" L{w:.2},{h:.2} Z"));
                body.push_str(&format!(
                    r#"<path d="{area}" fill="{tint}" opacity="0.3"/>"#
                ));
            }
            body.push_str(&format!(
                r#"<path d="{path}" fill="none" stroke="{tint}" stroke-width="2"/>"#
            ));
            format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}">{body}</svg>"#
            )
        }
        ChartType::Pie => {
            let total: f64 = pts.iter().map(|p| p.value).sum::<f64>().max(1e-6);
            let r = 40.0;
            let cx = 50.0;
            let cy = 50.0;
            let mut ca = -90.0_f64;
            let mut body = String::new();
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
                body.push_str(&format!(
                    r#"<path d="M{cx},{cy} L{x1:.2},{y1:.2} A{r},{r} 0 {lf},1 {x2:.2},{y2:.2} Z" fill="{fill}"/>"#
                ));
                ca += angle;
            }
            format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" width="80" height="80">{body}</svg>"#
            )
        }
        ChartType::Bar => {
            let n = pts.len().max(1) as f64;
            let gap = 4.0;
            let w = 200.0;
            let h = 70.0;
            let bar_w = ((w - gap * (n + 1.0)) / n).max(2.0);
            let mut body = String::new();
            for (i, p) in pts.iter().enumerate() {
                let bh = ((p.value / max_v) * 60.0).max(2.0);
                let x = gap + i as f64 * (bar_w + gap);
                let y = h - 10.0 - bh;
                let fill = color_str(p.color.as_ref(), &tint);
                body.push_str(&format!(
                    r#"<rect x="{x:.2}" y="{y:.2}" width="{bar_w:.2}" height="{bh:.2}" fill="{fill}" rx="2"/>"#
                ));
                body.push_str(&format!(
                    r#"<text x="{:.2}" y="{:.2}" font-size="8" fill="{}" text-anchor="middle">{}</text>"#,
                    x + bar_w / 2.0,
                    h - 1.0,
                    "#999",
                    esc(&p.label)
                ));
            }
            format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}">{body}</svg>"#
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

fn gauge_svg(
    value: f64,
    min: f64,
    max: f64,
    tint: Option<&ColorValue>,
    style: Option<&GaugeStyle>,
    current: Option<&str>,
    label: Option<&str>,
) -> String {
    let tint = color_str(tint, DEFAULT_TINT);
    let track = "#e0e0e0";
    let pct = (((value - min) / (max - min).max(1e-6)) * 100.0).clamp(0.0, 100.0);

    if matches!(style, Some(GaugeStyle::Linear)) {
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
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 120 28" width="120" height="28">{body}</svg>"#
        )
    } else {
        let mut body = String::new();
        body.push_str(&format!(
            r#"<path d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831" fill="none" stroke="{track}" stroke-width="4"/>"#
        ));
        body.push_str(&format!(
            r#"<path d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831" fill="none" stroke="{tint}" stroke-width="4" stroke-dasharray="{pct:.1}, 100" stroke-linecap="round"/>"#
        ));
        if let Some(c) = current {
            let white = "#ffffff";
            body.push_str(&format!(
                r#"<text x="18" y="20" font-size="8" font-weight="600" fill="{white}" text-anchor="middle">{}</text>"#,
                esc(c)
            ));
        }
        let label_h = if label.is_some() { 14.0 } else { 0.0 };
        if let Some(l) = label {
            // Lighten label vs ring tint so it stays readable on dark widget goldens.
            let label_fill = "#ffffff";
            body.push_str(&format!(
                r#"<text x="18" y="48" font-size="8" fill="{label_fill}" text-anchor="middle">{}</text>"#,
                esc(l)
            ));
        }
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 36 {:.0}" width="56" height="{:.0}">{body}</svg>"#,
            36.0 + label_h,
            56.0 + label_h
        )
    }
}

fn shape_svg(
    shape_type: &ShapeType,
    fill: Option<&ColorValue>,
    stroke: Option<&ColorValue>,
    stroke_width: f64,
    size: f64,
) -> String {
    let fill_s = color_str(fill, DEFAULT_TINT);
    let stroke_s = color_str(stroke, "none");
    let sw = stroke_width;
    match shape_type {
        ShapeType::Circle => {
            let r = size / 2.0;
            format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 {size} {size}"><circle cx="{r}" cy="{r}" r="{r}" fill="{fill_s}" stroke="{stroke_s}" stroke-width="{sw}"/></svg>"#
            )
        }
        ShapeType::Capsule => {
            let w = size * 2.0;
            let h = size;
            let rx = size / 2.0;
            format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}"><rect x="0" y="0" width="{w}" height="{h}" rx="{rx}" ry="{rx}" fill="{fill_s}" stroke="{stroke_s}" stroke-width="{sw}"/></svg>"#
            )
        }
        ShapeType::Rectangle => format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 {size} {size}"><rect x="0" y="0" width="{size}" height="{size}" fill="{fill_s}" stroke="{stroke_s}" stroke-width="{sw}"/></svg>"#
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ChartDataPoint, ChartType, ShapeType, WidgetElement};

    #[test]
    fn chart_svg_non_empty() {
        let el = WidgetElement::Chart(ChartElement {
            chart_type: ChartType::Bar,
            chart_data: vec![
                ChartDataPoint {
                    label: "a".into(),
                    value: 3.0,
                    color: None,
                },
                ChartDataPoint {
                    label: "b".into(),
                    value: 5.0,
                    color: None,
                },
            ],
            tint: None,
            style: Default::default(),
        });
        let svg = element_to_svg(&el).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("<rect"));
    }

    #[test]
    fn shape_svg_circle() {
        let el = WidgetElement::Shape(ShapeElement {
            shape_type: ShapeType::Circle,
            fill: None,
            stroke: None,
            stroke_width: None,
            size: Some(32.0),
            style: Default::default(),
        });
        let svg = element_to_svg(&el).unwrap();
        assert!(svg.contains("<circle"));
    }

    #[cfg(feature = "rasterize")]
    #[test]
    fn chart_png_data_uri() {
        let el = WidgetElement::Chart(ChartElement {
            chart_type: ChartType::Line,
            chart_data: vec![
                ChartDataPoint {
                    label: "a".into(),
                    value: 1.0,
                    color: None,
                },
                ChartDataPoint {
                    label: "b".into(),
                    value: 2.0,
                    color: None,
                },
            ],
            tint: None,
            style: Default::default(),
        });
        let uri = element_to_png_data_uri(&el).unwrap();
        assert!(uri.starts_with("data:image/png;base64,"));
        assert!(uri.len() > 64);
    }
}
