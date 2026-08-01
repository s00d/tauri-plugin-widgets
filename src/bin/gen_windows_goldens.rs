//! Generate `tests/golden/windows/<case>.png` from Adaptive Card transpile + SVG composite.
//!
//! ```bash
//! cargo run --bin gen-windows-goldens --features rasterize -- --dump-ac
//! CASE=weather.small cargo run --bin gen-windows-goldens --features rasterize -- --dump-ac
//! ```

use base64::Engine;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use tauri_plugin_widgets::adaptive_card::to_adaptive_card_for_size;
use tauri_plugin_widgets::models::WidgetConfig;

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cases_dir = root.join("tests/cases");
    let fixtures = root.join("tests/fixtures");
    let out_dir = root.join("tests/golden/windows");
    let ac_dir = root.join("tests/snapshots/adaptive");
    fs::create_dir_all(&out_dir).unwrap();
    fs::create_dir_all(&ac_dir).unwrap();

    let dump_ac = std::env::args().any(|a| a == "--dump-ac");
    let only = std::env::var("CASE").ok().filter(|s| !s.is_empty());

    let mut entries: Vec<_> = fs::read_dir(&cases_dir)
        .expect("tests/cases")
        .flatten()
        .collect();
    entries.sort_by_key(|e| e.file_name());

    let mut wrote = 0usize;
    for entry in entries {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        if let Some(ref only) = only {
            if only != &name {
                continue;
            }
        }

        let case: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let fixture = case["fixture"].as_str().unwrap_or("");
        let size = case["size"].as_str().unwrap_or("small");
        let cfg_path = fixtures.join(format!("{fixture}.json"));
        if !cfg_path.exists() {
            eprintln!("skip {name}: missing fixture {fixture}");
            continue;
        }
        let raw = fs::read_to_string(&cfg_path).unwrap();
        let v: Value = serde_json::from_str(&raw).unwrap();
        let cfg: WidgetConfig = match serde_json::from_value(v) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("skip {name}: parse {e}");
                continue;
            }
        };
        let Some(result) = to_adaptive_card_for_size(&cfg, size) else {
            eprintln!("skip {name}: no layout for size={size}");
            continue;
        };

        if dump_ac {
            let dest = ac_dir.join(format!("{name}.json"));
            let pretty = serde_json::to_string_pretty(&result.card).unwrap();
            fs::write(&dest, pretty).unwrap();
            println!("ac {}", dest.display());
        }

        let (w, min_h) = size_px(size);
        let png = render_card_png(&result.card, w, min_h).unwrap_or_else(|e| {
            eprintln!("skip {name}: {e}");
            Vec::new()
        });
        if png.is_empty() {
            continue;
        }
        let dest = out_dir.join(format!("{name}.png"));
        fs::write(&dest, &png).unwrap();
        println!("wrote {}", dest.display());
        wrote += 1;
    }

    if only.is_none() && wrote == 0 {
        eprintln!("no goldens written");
        std::process::exit(1);
    }
    println!("done: {wrote} golden(s)");
}

fn size_px(size: &str) -> (u32, u32) {
    match size {
        "large" => (338, 354),
        "medium" => (338, 158),
        _ => (158, 158),
    }
}

fn render_card_png(card: &Value, w: u32, min_h: u32) -> Result<Vec<u8>, String> {
    let body = card
        .get("body")
        .and_then(|b| b.as_array())
        .cloned()
        .unwrap_or_default();
    // Single Stretch image (canvas/chart): fill the widget frame, centered.
    if body.len() == 1
        && body[0].get("type").and_then(|t| t.as_str()) == Some("Image")
        && body[0].get("size").and_then(|s| s.as_str()) == Some("Stretch")
    {
        if let Some(url) = body[0].get("url").and_then(|u| u.as_str()) {
            if let Some(rest) = url.strip_prefix("data:image/png;base64,") {
                let bytes = base64::engine::general_purpose::STANDARD
                    .decode(rest)
                    .map_err(|e| e.to_string())?;
                let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                let pad = 8.0_f64;
                let iw = (w as f64) - pad * 2.0;
                let ih = (min_h as f64) - pad * 2.0;
                let bg = "#0f172a";
                let aspect = "xMidYMid meet";
                let svg = format!(
                    r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{min_h}" viewBox="0 0 {w} {min_h}"><rect width="100%" height="100%" fill="{bg}"/><image x="{pad}" y="{pad}" width="{iw}" height="{ih}" href="data:image/png;base64,{b64}" preserveAspectRatio="{aspect}"/></svg>"#
                );
                return svg_png(&svg);
            }
        }
    }

    let max_w = (w as f64) - 16.0;
    let pad_x = 8.0_f64;

    // Expand spacers (" " TextBlocks) to vertically center content in the frame.
    let spacer_idxs: Vec<usize> = body
        .iter()
        .enumerate()
        .filter(|(_, el)| is_spacer_el(el))
        .map(|(i, _)| i)
        .collect();

    let mut measured = Vec::with_capacity(body.len());
    let mut content_h = 0.0_f64;
    for el in &body {
        if is_spacer_el(el) {
            measured.push(0.0);
            continue;
        }
        let mut discard = String::new();
        let y1 = append_el(&mut discard, el, pad_x, 0.0, max_w, true, None)?;
        measured.push(y1);
        content_h += y1;
    }
    let slack = ((min_h as f64) - content_h).max(0.0);
    let spacer_h = if spacer_idxs.is_empty() {
        0.0
    } else {
        slack / spacer_idxs.len() as f64
    };
    // Nested spacers own vertical centering — don't also pad the root.
    let has_nested_spacers = body.iter().any(container_has_spacers);
    let top_pad = if spacer_idxs.is_empty()
        && !has_nested_spacers
        && slack > 8.0
        && body_wants_center(&body)
    {
        slack / 2.0
    } else {
        8.0
    };

    let mut parts = String::new();
    let mut y = top_pad;
    for (i, el) in body.iter().enumerate() {
        if is_spacer_el(el) {
            y += spacer_h.max(8.0);
            continue;
        }
        // Root container fills remaining frame so nested spacers can expand.
        let fill = Some((min_h as f64) - y - 8.0);
        y = append_el(&mut parts, el, pad_x, y, max_w, true, fill)?;
        let _ = measured[i];
    }
    let h = ((y + 12.0).ceil() as u32).max(min_h);
    let bg = "#0f172a";
    let svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}"><rect width="100%" height="100%" fill="{bg}"/>{parts}</svg>"#
    );
    svg_png(&svg)
}

fn id_tokens(el: &Value) -> impl Iterator<Item = &str> {
    el.get("id")
        .and_then(|i| i.as_str())
        .unwrap_or("")
        .split(';')
        .filter(|s| !s.is_empty())
}

fn id_has(el: &Value, token: &str) -> bool {
    id_tokens(el).any(|t| t == token)
}

fn id_value<'a>(el: &'a Value, prefix: &str) -> Option<&'a str> {
    id_tokens(el).find_map(|t| t.strip_prefix(prefix))
}

fn id_gap(el: &Value) -> Option<f64> {
    id_value(el, "gap:")?.parse().ok()
}

fn is_spacer_el(el: &Value) -> bool {
    el.get("type").and_then(|t| t.as_str()) == Some("TextBlock")
        && el
            .get("text")
            .and_then(|t| t.as_str())
            .map(|t| t.trim().is_empty())
            .unwrap_or(false)
}

fn container_has_spacers(el: &Value) -> bool {
    el.get("items")
        .and_then(|i| i.as_array())
        .map(|a| a.iter().any(is_spacer_el))
        .unwrap_or(false)
}

fn is_glyph_text(el: &Value) -> bool {
    el.get("type").and_then(|t| t.as_str()) == Some("TextBlock")
        && el
            .get("text")
            .and_then(|t| t.as_str())
            .map(|t| {
                let t = t.trim();
                !t.is_empty() && t.chars().count() <= 2
            })
            .unwrap_or(false)
}

fn is_badge_pair(items: &[Value]) -> bool {
    items.len() == 2
        && items[0].get("type").and_then(|t| t.as_str()) == Some("Image")
        && is_glyph_text(&items[1])
}

fn body_wants_center(body: &[Value]) -> bool {
    body.iter().any(|el| {
        el.get("horizontalAlignment").and_then(|a| a.as_str()) == Some("Center")
            || id_has(el, "badge:overlay")
            || el
                .get("items")
                .and_then(|i| i.as_array())
                .map(|items| {
                    items.iter().any(|c| {
                        c.get("horizontalAlignment").and_then(|a| a.as_str()) == Some("Center")
                            || id_has(c, "badge:overlay")
                    })
                })
                .unwrap_or(false)
    })
}

fn append_el(
    parts: &mut String,
    el: &Value,
    x: f64,
    y: f64,
    max_w: f64,
    allow_center: bool,
    fill_h: Option<f64>,
) -> Result<f64, String> {
    let ty = el.get("type").and_then(|t| t.as_str()).unwrap_or("");
    // Parent `allow_center` propagates to children (VStack/Container alignment).
    let center = allow_center
        || el.get("horizontalAlignment").and_then(|a| a.as_str()) == Some("Center")
        || id_has(el, "badge:overlay");
    match ty {
        "TextBlock" => {
            let text = el.get("text").and_then(|t| t.as_str()).unwrap_or("");
            if text.trim().is_empty() {
                return Ok(y + 4.0);
            }
            let fill = id_value(el, "fg:")
                .filter(|h| h.starts_with('#'))
                .map(|h| h.to_string())
                .or_else(|| {
                    el.get("color")
                        .and_then(|c| c.as_str())
                        .and_then(|c| match c {
                            "Good" => Some("#66BB6A".into()),
                            "Accent" => Some("#93C5FD".into()),
                            "Warning" => Some("#FFB74D".into()),
                            "Attention" => Some("#FCA5A5".into()),
                            "Light" => Some("#F8FAFC".into()),
                            "Dark" => Some("#E2E8F0".into()),
                            "Default" => Some("#F8FAFC".into()),
                            _ => None,
                        })
                })
                .unwrap_or_else(|| "#F8FAFC".into());
            let font = match el.get("size").and_then(|s| s.as_str()) {
                Some("ExtraLarge") => 28.0,
                Some("Large") => 20.0,
                Some("Medium") => 18.0,
                Some("Small") => 11.0,
                _ => 13.0,
            };
            let weight_bold = el
                .get("weight")
                .and_then(|w| w.as_str())
                .map(|w| w == "Bolder")
                .unwrap_or(false);
            let fw = if weight_bold { "bold" } else { "normal" };
            let align = el
                .get("horizontalAlignment")
                .and_then(|a| a.as_str())
                .unwrap_or(if center { "Center" } else { "Left" });
            let (anchor, tx) = match align {
                "Center" => ("middle", x + max_w / 2.0),
                "Right" => ("end", x + max_w),
                _ => ("start", x),
            };
            let wrap = el
                .get("wrap")
                .and_then(|w| w.as_bool())
                .unwrap_or(true);
            let approx_chars = ((max_w / (font * 0.58)).floor() as usize).max(4);
            let mut yy = y;
            let lines = if wrap {
                wrap_text(text, approx_chars)
            } else {
                let mut t = text.to_string();
                if t.chars().count() > approx_chars {
                    t = format!(
                        "{}…",
                        t.chars().take(approx_chars.saturating_sub(1)).collect::<String>()
                    );
                }
                vec![t]
            };
            for chunk in lines {
                let esc = chunk
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;");
                parts.push_str(&format!(
                    r#"<text x="{tx}" y="{}" font-size="{font}" font-weight="{fw}" fill="{fill}" text-anchor="{anchor}" font-family="sans-serif">{}</text>"#,
                    yy + font,
                    esc
                ));
                yy += font + 3.0;
            }
            Ok(yy)
        }
        "Image" => {
            let url = el.get("url").and_then(|u| u.as_str()).unwrap_or("");
            if let Some(rest) = url.strip_prefix("data:image/png;base64,") {
                let bytes = base64::engine::general_purpose::STANDARD
                    .decode(rest)
                    .map_err(|e| e.to_string())?;
                let img_h = match el.get("size").and_then(|s| s.as_str()) {
                    Some("Small") => 24.0,
                    Some("Medium") => 48.0,
                    Some("Large") => 72.0,
                    Some("Stretch") => max_w.min(120.0),
                    _ => 40.0,
                };
                let img_w = img_h.min(max_w);
                let align = el
                    .get("horizontalAlignment")
                    .and_then(|a| a.as_str())
                    .unwrap_or(if center { "Center" } else { "Left" });
                let ix = match align {
                    "Center" => x + (max_w - img_w) / 2.0,
                    "Right" => x + max_w - img_w,
                    _ => x,
                };
                let aspect = "xMidYMid meet";
                parts.push_str(&format!(
                    r#"<image x="{ix}" y="{y}" width="{img_w}" height="{img_h}" href="data:image/png;base64,{}" preserveAspectRatio="{aspect}"/>"#,
                    base64::engine::general_purpose::STANDARD.encode(&bytes)
                ));
                return Ok(y + img_h + 6.0);
            }
            Ok(y + 8.0)
        }
        "ActionSet" => {
            let action = el
                .get("actions")
                .and_then(|a| a.as_array())
                .and_then(|a| a.first());
            let title = action
                .and_then(|a| a.get("title"))
                .and_then(|t| t.as_str())
                .unwrap_or("action");
            let id = action
                .and_then(|a| a.get("id"))
                .and_then(|i| i.as_str())
                .unwrap_or("");
            let mut btn = "#2563eb";
            let mut fill = "#ffffff";
            for part in id.split(';') {
                if let Some(hex) = part.strip_prefix("bg:") {
                    btn = hex;
                }
                if let Some(hex) = part.strip_prefix("fg:") {
                    let _ = hex;
                    fill = "#ffffff";
                }
            }
            let esc = title
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;");
            let nchars = esc.chars().count();
            let fs = if nchars <= 2 {
                14.0
            } else if nchars <= 4 {
                12.0
            } else {
                12.0
            };
            // Hug label width instead of stretching across the column.
            let bw = ((nchars as f64) * fs * 0.72 + 28.0)
                .min(max_w)
                .max(56.0);
            let bh = 28.0_f64;
            let bx = if center {
                x + (max_w - bw) / 2.0
            } else {
                x
            };
            parts.push_str(&format!(
                r#"<rect x="{bx}" y="{y}" width="{bw}" height="{bh}" rx="8" fill="{btn}"/><text x="{}" y="{}" font-size="{fs}" font-weight="bold" fill="{fill}" text-anchor="middle" font-family="sans-serif">{}</text>"#,
                bx + bw / 2.0,
                y + bh * 0.68,
                esc
            ));
            Ok(y + bh + 4.0)
        }
        "Container" | "Column" => {
            if let Some(items) = el.get("items").and_then(|i| i.as_array()) {
                let child_center = center
                    || el.get("horizontalAlignment").and_then(|a| a.as_str()) == Some("Center")
                    || id_has(el, "badge:overlay");
                // Icon badge overlay: only Image + short glyph (✓/Z), not labels like "Card".
                let explicit_badge = id_has(el, "badge:overlay");
                if is_badge_pair(items) || (explicit_badge && is_badge_pair(items)) {
                    return overlay_image_text(
                        parts,
                        &items[0],
                        &items[1],
                        x,
                        y,
                        max_w,
                        true,
                    );
                }
                if explicit_badge && items.len() >= 2 {
                    if let (Some(img), Some(tb)) = (
                        items
                            .iter()
                            .find(|c| c.get("type").and_then(|t| t.as_str()) == Some("Image")),
                        items.iter().find(|c| is_glyph_text(c)),
                    ) {
                        return overlay_image_text(parts, img, tb, x, y, max_w, true);
                    }
                }
                if el.get("separator").and_then(|s| s.as_bool()) == Some(true) && items.is_empty()
                {
                    let line = id_value(el, "rule:").unwrap_or("#334155");
                    parts.push_str(&format!(
                        r#"<rect x="{x}" y="{}" width="{max_w}" height="1.5" rx="0.75" fill="{line}"/>"#,
                        y + 6.0
                    ));
                    return Ok(y + 14.0);
                }
                let chip = el.get("selectAction").is_some();
                let card_fill = id_value(el, "card:")
                    .map(|s| s.to_string())
                    .or_else(|| {
                        // emphasis without explicit hex → slate card
                        if el.get("style").and_then(|s| s.as_str()) == Some("emphasis")
                            && !chip
                            && !id_has(el, "badge:overlay")
                        {
                            Some("#1e293b".into())
                        } else {
                            None
                        }
                    });
                // Root page bg matching canvas — skip painting opaque card.
                let card_fill = card_fill.filter(|c| c != "#0f172a" && c != "#0b1220");
                let pad = if card_fill.is_some() {
                    12.0
                } else if chip {
                    6.0
                } else {
                    0.0
                };
                let start_y = y;
                let child_gap = id_gap(el).unwrap_or(0.0);

                // Nested spacers: expand to fill remaining height.
                let spacer_idxs: Vec<usize> = items
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| is_spacer_el(c))
                    .map(|(i, _)| i)
                    .collect();
                let mut fixed_h = 0.0_f64;
                let mut saw_content = false;
                for c in items.iter() {
                    if is_spacer_el(c) {
                        continue;
                    }
                    if saw_content {
                        fixed_h += child_gap;
                    }
                    saw_content = true;
                    let mut discard = String::new();
                    let y1 =
                        append_el(&mut discard, c, x, 0.0, max_w - pad * 2.0, child_center, None)?;
                    fixed_h += y1;
                }
                let target = fill_h.unwrap_or(fixed_h + 16.0 * spacer_idxs.len() as f64);
                let slack = (target - fixed_h - pad * 2.0).max(0.0);
                let spacer_h = if spacer_idxs.is_empty() {
                    0.0
                } else {
                    slack / spacer_idxs.len() as f64
                };

                let mut yy = y + pad;
                let mut inner = String::new();
                let mut first_content = true;
                for c in items.iter() {
                    if is_spacer_el(c) {
                        yy += spacer_h.max(6.0);
                        continue;
                    }
                    if !first_content {
                        yy += child_gap;
                    }
                    first_content = false;
                    yy = append_el(
                        &mut inner,
                        c,
                        x + pad,
                        yy,
                        max_w - pad * 2.0,
                        child_center,
                        None,
                    )?;
                }
                if let Some(ref fill) = card_fill {
                    let h = (yy - start_y + pad).max(28.0);
                    parts.push_str(&format!(
                        r#"<rect x="{x}" y="{start_y}" width="{max_w}" height="{h}" rx="10" fill="{fill}"/>"#
                    ));
                } else if chip {
                    let chip_bg = "#1e293b";
                    let h = (yy - start_y + pad).max(22.0);
                    parts.push_str(&format!(
                        r#"<rect x="{x}" y="{start_y}" width="{max_w}" height="{h}" rx="12" fill="{chip_bg}"/>"#
                    ));
                }
                parts.push_str(&inner);
                return Ok(yy + pad);
            }
            Ok(y)
        }
        "ColumnSet" => {
            let cols = el
                .get("columns")
                .and_then(|c| c.as_array())
                .cloned()
                .unwrap_or_default();
            let all_empty = cols.iter().all(|c| {
                c.get("items")
                    .and_then(|i| i.as_array())
                    .map(|a| a.is_empty())
                    .unwrap_or(true)
            });
            if all_empty && !cols.is_empty() {
                let widths: Vec<f64> = cols
                    .iter()
                    .map(|c| match c.get("width") {
                        Some(Value::Number(n)) => n.as_f64().unwrap_or(1.0),
                        Some(Value::String(s)) => s.parse().unwrap_or(1.0),
                        _ => 1.0,
                    })
                    .collect();
                let sum: f64 = widths.iter().sum::<f64>().max(1.0);
                let bar_h = 12.0_f64;
                // Track under the fill so rest of the bar stays visible on dark cards.
                let track = "#334155";
                parts.push_str(&format!(
                    r#"<rect x="{x:.2}" y="{y:.2}" width="{max_w:.2}" height="{bar_h}" fill="{track}" rx="6"/>"#
                ));
                let mut xx = x;
                for (i, c) in cols.iter().enumerate() {
                    let wcol = max_w * (widths[i] / sum);
                    if column_has_progress_fill(c) {
                        let fill = column_fill_owned(c);
                        parts.push_str(&format!(
                            r#"<rect x="{xx:.2}" y="{y:.2}" width="{wcol:.2}" height="{bar_h}" fill="{fill}" rx="6"/>"#
                        ));
                    }
                    xx += wcol;
                }
                return Ok(y + bar_h + 4.0);
            }
            let weights: Vec<f64> = cols.iter().map(|c| column_weight(c)).collect();
            let sum: f64 = weights.iter().sum::<f64>().max(1.0);
            let mark_row = cols.iter().any(|c| id_has(c, "mark:col"));
            let gap = id_gap(el).unwrap_or(if mark_row { 4.0 } else { 8.0 });
            let n = cols.len().max(1) as f64;
            let inner_w = (max_w - gap * (n - 1.0)).max(20.0);
            // Measure column heights first so we can vertically center short columns (header row).
            let mut heights = Vec::with_capacity(cols.len());
            let mut widths = Vec::with_capacity(cols.len());
            for (i, col) in cols.iter().enumerate() {
                let cw = (inner_w * (weights[i] / sum)).max(12.0);
                widths.push(cw);
                let mut discard = String::new();
                let y1 = append_el(&mut discard, col, 0.0, 0.0, cw, false, None)?;
                heights.push(y1);
            }
            let row_h = heights.iter().cloned().fold(0.0_f64, f64::max);
            let mut max_y = y;
            let mut xx = x;
            for (i, col) in cols.iter().enumerate() {
                let cw = widths[i];
                let y_off = ((row_h - heights[i]) / 2.0).max(0.0);
                let yy = append_el(parts, col, xx, y + y_off, cw, false, None)?;
                max_y = max_y.max(yy);
                xx += cw + gap;
            }
            Ok(y + row_h)
        }
        _ => Ok(y),
    }
}

fn column_weight(col: &Value) -> f64 {
    match col.get("width") {
        Some(Value::Number(n)) => n.as_f64().unwrap_or(1.0).max(1.0),
        Some(Value::String(s)) if s.eq_ignore_ascii_case("stretch") => {
            let items = col
                .get("items")
                .and_then(|i| i.as_array())
                .cloned()
                .unwrap_or_default();
            if items.is_empty() {
                36.0 // pure spacer
            } else {
                160.0 // title / main content marked stretch
            }
        }
        Some(Value::String(s)) if s.eq_ignore_ascii_case("auto") => {
            let items = col
                .get("items")
                .and_then(|i| i.as_array())
                .cloned()
                .unwrap_or_default();
            if items.is_empty() {
                return 40.0; // spacer — fills remainder, don't starve titles
            }
            let only = items.len() == 1;
            if only && items[0].get("type").and_then(|t| t.as_str()) == Some("ActionSet") {
                return 36.0;
            }
            if only
                && (id_has(&items[0], "badge:overlay")
                    || items[0]
                        .get("items")
                        .and_then(|i| i.as_array())
                        .map(|a| is_badge_pair(a))
                        .unwrap_or(false))
            {
                return badge_column_weight(&items[0]);
            }
            // Short auto TextBlock (prices, marks) — hug; don't steal flex title space.
            if only && items[0].get("type").and_then(|t| t.as_str()) == Some("TextBlock") {
                let t = items[0]
                    .get("text")
                    .and_then(|t| t.as_str())
                    .unwrap_or("");
                let n = t.chars().count();
                if id_has(col, "mark:col") || n <= 2 {
                    return 18.0;
                }
                // "$12.50" / short trailing labels
                return ((n as f64) * 6.8 + 14.0).clamp(32.0, 84.0);
            }
            if id_has(col, "mark:col") {
                return 18.0;
            }
            // Titles / body text columns need the bulk of the row.
            160.0
        }
        Some(Value::String(s)) => s.parse().unwrap_or(50.0),
        _ => 50.0,
    }
}

fn badge_column_weight(badge: &Value) -> f64 {
    let size = badge
        .get("items")
        .and_then(|i| i.as_array())
        .and_then(|items| {
            items.iter().find_map(|c| {
                (c.get("type").and_then(|t| t.as_str()) == Some("Image"))
                    .then(|| c.get("size").and_then(|s| s.as_str()))
                    .flatten()
            })
        })
        .or_else(|| badge.get("size").and_then(|s| s.as_str()));
    match size {
        Some("Small") => 24.0,
        Some("Large") => 64.0,
        _ => 52.0,
    }
}

fn column_has_progress_fill(c: &Value) -> bool {
    if id_value(c, "fill:").is_some() {
        return true;
    }
    matches!(
        c.get("style").and_then(|s| s.as_str()),
        Some("good")
            | Some("Good")
            | Some("attention")
            | Some("Attention")
            | Some("warning")
            | Some("Warning")
            | Some("accent")
            | Some("Accent")
            | Some("emphasis")
    )
}

fn column_fill_owned(c: &Value) -> String {
    if let Some(hex) = id_value(c, "fill:") {
        if hex.starts_with('#') {
            return hex.to_string();
        }
    }
    match c.get("style").and_then(|s| s.as_str()) {
        Some("good") | Some("Good") | Some("emphasis") => "#4CAF50".into(),
        Some("attention") | Some("Attention") => "#e94560".into(),
        Some("warning") | Some("Warning") => "#FBBF24".into(),
        Some("accent") | Some("Accent") => "#22D3EE".into(),
        _ => "#333333".into(),
    }
}

fn overlay_image_text(
    parts: &mut String,
    img: &Value,
    tb: &Value,
    x: f64,
    y: f64,
    max_w: f64,
    center: bool,
) -> Result<f64, String> {
    let url = img.get("url").and_then(|u| u.as_str()).unwrap_or("");
    let text = tb.get("text").and_then(|t| t.as_str()).unwrap_or("");
    let preferred: f64 = match img.get("size").and_then(|s| s.as_str()) {
        Some("Small") => 20.0,
        Some("Large") => 56.0,
        Some("Stretch") => 48.0,
        _ => 44.0, // match nested-dashboard shape size
    };
    let size = preferred.min(max_w);
    let ix = if center {
        x + (max_w - size) / 2.0
    } else {
        x
    };
    if let Some(rest) = url.strip_prefix("data:image/png;base64,") {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(rest)
            .map_err(|e| e.to_string())?;
        parts.push_str(&format!(
            r#"<image x="{ix}" y="{y}" width="{size}" height="{size}" href="data:image/png;base64,{}" preserveAspectRatio="xMidYMid meet"/>"#,
            base64::engine::general_purpose::STANDARD.encode(&bytes)
        ));
    }
    let esc = text
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    let fg = "#ffffff";
    parts.push_str(&format!(
        r#"<text x="{}" y="{}" font-size="16" font-weight="bold" fill="{fg}" text-anchor="middle" dominant-baseline="central" font-family="sans-serif">{}</text>"#,
        ix + size / 2.0,
        y + size / 2.0,
        esc
    ));
    Ok(y + size + 6.0)
}

fn wrap_text(s: &str, width: usize) -> Vec<String> {
    if s.chars().count() <= width {
        return vec![s.to_string()];
    }
    let mut lines = Vec::new();
    let mut cur = String::new();
    for word in s.split_whitespace() {
        if cur.is_empty() {
            cur = word.to_string();
        } else if cur.chars().count() + 1 + word.chars().count() <= width {
            cur.push(' ');
            cur.push_str(word);
        } else {
            lines.push(cur);
            cur = word.to_string();
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    if lines.is_empty() {
        lines.push(s.chars().take(width).collect());
    }
    lines
}

fn svg_png(svg: &str) -> Result<Vec<u8>, String> {
    let mut opts = resvg::usvg::Options::default();
    opts.fontdb_mut().load_system_fonts();
    let tree = resvg::usvg::Tree::from_str(svg, &opts).map_err(|e| format!("usvg: {e}"))?;
    let size = tree.size();
    let w = size.width().ceil().max(1.0) as u32;
    let h = size.height().ceil().max(1.0) as u32;
    let mut pixmap =
        resvg::tiny_skia::Pixmap::new(w, h).ok_or_else(|| "pixmap".to_string())?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );
    pixmap.encode_png().map_err(|e| format!("png: {e}"))
}

#[allow(dead_code)]
fn _touch_path(_: &Path) {}
