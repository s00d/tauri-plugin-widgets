//! Text group: text / date / timer / label.

use crate::models::{DateElement, LabelElement, TextElement, TimerCounting, TimerElement};
use serde_json::{json, Value};

use super::style::{
    ac_color, align, approx_hex_semantic, color_hex, sf_symbol_glyph, size_bucket, weight_token,
};

pub(super) fn text(t: &TextElement) -> Value {
    let TextElement {
        content,
        font_size,
        font_weight,
        color,
        alignment,
        line_limit,
        underline,
        strikethrough,
        italic,
        monospaced_digit,
        ..
    } = t;
    let flag = |b: &Option<bool>| b.unwrap_or(false);
    let rich = flag(underline) || flag(strikethrough) || flag(italic);
    // AC has no tabular-figures switch: a monospace font only when the text is all digits.
    let mono = flag(monospaced_digit) && is_numeric(content);
    let mut obj = if rich {
        // TextBlock has no italic / underline / strikethrough: one styled TextRun.
        // RichTextBlock always wraps (no maxLines), so lineLimit is dropped here.
        let mut run = json!({
            "type": "TextRun",
            "text": content,
            "size": size_bucket(*font_size),
            "weight": weight_token(font_weight.as_ref()),
        });
        for (key, on) in [
            ("italic", italic),
            ("underline", underline),
            ("strikethrough", strikethrough),
        ] {
            if flag(on) {
                run[key] = json!(true);
            }
        }
        if let Some(c) = ac_color(color.as_ref()).or_else(|| approx_hex_semantic(color.as_ref())) {
            run["color"] = json!(c);
        }
        if mono {
            run["fontType"] = json!("Monospace");
        }
        json!({"type": "RichTextBlock", "inlines": [run]})
    } else {
        let wrap = !matches!(line_limit, Some(n) if *n <= 1);
        let mut obj = json!({
            "type": "TextBlock",
            "text": content,
            "wrap": wrap,
            "size": size_bucket(*font_size),
            "weight": weight_token(font_weight.as_ref()),
        });
        if let Some(n) = line_limit.filter(|n| *n > 1) {
            obj["maxLines"] = json!(n);
        }
        if let Some(c) = ac_color(color.as_ref()).or_else(|| approx_hex_semantic(color.as_ref())) {
            obj["color"] = json!(c);
        }
        if mono {
            obj["fontType"] = json!("Monospace");
        }
        obj
    };
    // Preserve exact hex for composite (muted slates stay slate, not AC "Light").
    if let Some(hex) = color_hex(color.as_ref()) {
        obj["id"] = json!(format!("fg:{hex}"));
    }
    if let Some(a) = align(alignment.as_ref()) {
        obj["horizontalAlignment"] = json!(a);
    }
    obj
}

/// Digits plus number punctuation only (`12,345.67`, `-3 %`, `12:05`).
fn is_numeric(s: &str) -> bool {
    s.chars().any(|c| c.is_ascii_digit())
        && s.chars()
            .all(|c| c.is_ascii_digit() || " .,:+-\u{2212}%$\u{20ac}\u{a3}/".contains(c))
}

pub(super) fn date(d: &DateElement) -> Value {
    let DateElement { date, .. } = d;
    json!({
        "type": "TextBlock",
        "text": date,
        "wrap": true,
        "size": "Default",
    })
}

/// Static timer text at [`super::now_ms`] (the card is rebuilt by the host's timers):
/// `down` = time left to `targetDate` (`completedText` / `0:00` once passed, held at the
/// full range before `startDate`); `up` = time since `startDate` (else `targetDate`).
pub(super) fn timer(t: &TimerElement) -> Value {
    let TimerElement {
        target_date,
        counting,
        font_size,
        font_weight,
        color,
        start_date,
        show_hours,
        completed_text,
        ..
    } = t;
    let now = super::now_ms() as i64;
    let start = start_date.as_deref().and_then(parse_iso_ms);
    let text = match parse_iso_ms(target_date) {
        None => target_date.clone(),
        Some(target) => {
            let show_hours = show_hours.unwrap_or(false);
            if matches!(counting, Some(TimerCounting::Up)) {
                format_duration((now - start.unwrap_or(target)).max(0), show_hours)
            } else {
                let left = target - start.map_or(now, |s| now.max(s));
                if left <= 0 {
                    completed_text.clone().unwrap_or_else(|| "0:00".into())
                } else {
                    format_duration(left, show_hours)
                }
            }
        }
    };
    let mut obj = json!({
        "type": "TextBlock",
        "text": text,
        "wrap": true,
        "size": size_bucket(*font_size),
    });
    if font_weight.is_some() {
        obj["weight"] = json!(weight_token(font_weight.as_ref()));
    }
    if let Some(c) = ac_color(color.as_ref()).or_else(|| approx_hex_semantic(color.as_ref())) {
        obj["color"] = json!(c);
    }
    obj
}

/// `M:SS`, or `H:MM:SS` when hours are non-zero or forced.
fn format_duration(ms: i64, show_hours: bool) -> String {
    let secs = ms.max(0) / 1000;
    let (h, m, s) = (secs / 3600, secs % 3600 / 60, secs % 60);
    if h > 0 || show_hours {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

/// ISO-8601 `YYYY-MM-DD[THH:MM[:SS[.fff]]][Z|±HH[:]MM]` → Unix ms (no zone = UTC).
fn parse_iso_ms(s: &str) -> Option<i64> {
    let s = s.trim();
    let num = |r: std::ops::Range<usize>, src: &str| -> Option<i64> {
        let part = src.get(r)?;
        part.bytes()
            .all(|b| b.is_ascii_digit())
            .then(|| part.parse().ok())?
    };
    if s.get(4..5)? != "-" || s.get(7..8)? != "-" {
        return None;
    }
    let (y, mo, d) = (num(0..4, s)?, num(5..7, s)?, num(8..10, s)?);
    let mut rest = &s[10..];
    let (mut hh, mut mm, mut ss, mut frac) = (0, 0, 0, 0);
    if let Some(r) = rest.strip_prefix(['T', 't', ' ']) {
        if r.get(2..3)? != ":" {
            return None;
        }
        hh = num(0..2, r)?;
        mm = num(3..5, r)?;
        rest = &r[5..];
        if let Some(r) = rest.strip_prefix(':') {
            ss = num(0..2, r)?;
            rest = &r[2..];
        }
        if let Some(r) = rest.strip_prefix(['.', ',']) {
            let digits = r.bytes().take_while(u8::is_ascii_digit).count();
            let ms3 = format!("{:0<3}", &r[..digits.min(3)]);
            frac = ms3.parse().ok()?;
            rest = &r[digits..];
        }
    }
    let offset_min = match rest {
        "" | "Z" | "z" => 0,
        tz => {
            let sign = match tz.as_bytes()[0] {
                b'+' => 1,
                b'-' => -1,
                _ => return None,
            };
            let digits: String = tz[1..].chars().filter(|c| *c != ':').collect();
            let oh = num(0..2, &digits)?;
            let om = if digits.len() > 2 {
                num(2..4, &digits)?
            } else {
                0
            };
            sign * (oh * 60 + om)
        }
    };
    if !(1..=12).contains(&mo) || !(1..=31).contains(&d) {
        return None;
    }
    // Howard Hinnant's days_from_civil.
    let y = if mo <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (mo + if mo > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    Some(((days * 86_400 + hh * 3600 + mm * 60 + ss - offset_min * 60) * 1000) + frac)
}

pub(super) fn label(l: &LabelElement) -> Value {
    let LabelElement {
        text,
        system_name,
        color,
        ..
    } = l;
    let prefix = if system_name.is_empty() {
        String::new()
    } else {
        format!("{} ", sf_symbol_glyph(system_name))
    };
    let mut obj = json!({
        "type": "TextBlock",
        "text": format!("{prefix}{text}"),
        "wrap": true,
        "weight": "Bolder",
        "size": "Small",
    });
    if let Some(c) = ac_color(color.as_ref()).or_else(|| approx_hex_semantic(color.as_ref())) {
        obj["color"] = json!(c);
    } else {
        obj["color"] = json!("Light");
    }
    obj
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Timer text chain: ISO dates parse (offsets, fractions, date-only, invalid) →
    /// durations format → a timer element renders its text at a pinned "now".
    #[test]
    fn iso_dates_durations_then_timer_text() {
        assert_eq!(parse_iso_ms("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(parse_iso_ms("2001-01-01T00:00:00Z"), Some(978_307_200_000));
        assert_eq!(parse_iso_ms("2001-01-01"), Some(978_307_200_000));
        assert_eq!(
            parse_iso_ms("2001-01-01T02:00:00.5+02:00"),
            Some(978_307_200_500)
        );
        assert_eq!(
            parse_iso_ms("2024-02-29T12:30:00-0130"),
            Some(1_709_215_200_000)
        );
        assert_eq!(parse_iso_ms("nope"), None);
        assert_eq!(parse_iso_ms("2001-13-01"), None);
        assert_eq!(format_duration(65_000, false), "1:05");
        assert_eq!(format_duration(65_000, true), "0:01:05");
        assert_eq!(format_duration(3_725_000, false), "1:02:05");
        assert!(is_numeric("12,345.67") && !is_numeric("12 apples") && !is_numeric("..."));

        let t = |v: serde_json::Value| -> String {
            let el: TimerElement = serde_json::from_value(v).unwrap();
            super::super::with_now(978_307_200_000, || timer(&el))["text"]
                .as_str()
                .unwrap()
                .to_string()
        };
        let target = "2001-01-01T00:01:05Z";
        assert_eq!(t(json!({"targetDate": target})), "1:05");
        assert_eq!(
            t(json!({"targetDate": target, "showHours": true})),
            "0:01:05"
        );
        assert_eq!(
            t(json!({"targetDate": "2000-12-31T23:59:00Z", "completedText": "Done"})),
            "Done"
        );
        assert_eq!(t(json!({"targetDate": "2000-12-31T23:59:00Z"})), "0:00");
        assert_eq!(
            t(json!({"targetDate": "2000-12-31T23:59:00Z", "counting": "up"})),
            "1:00"
        );
        assert_eq!(
            t(json!({"targetDate": target, "startDate": "2001-01-01T00:01:00Z"})),
            "0:05"
        );
        assert_eq!(t(json!({"targetDate": "soon"})), "soon");
    }
}
