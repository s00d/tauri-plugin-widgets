//! IR → Adaptive Card 1.5 transpiler for Windows Widgets Board.
//!
//! Pure Rust — runs on any host. Elements Adaptive Cards cannot express natively
//! (`canvas`, `chart`, `gauge`, `shape`, `zstack`, gradients) are rasterized to
//! PNG data URIs when the `rasterize` feature is enabled; otherwise they may be
//! skipped / flattened and recorded in [`TranspileResult::skipped`].
//!
//! Element conversion is split across the six render groups (`layout`, `text`,
//! `media`, `data`, `interactive`, `spacing`) plus shared token helpers in
//! `style`, mirroring the Android/Swift renderers.

mod data;
mod interactive;
mod layout;
mod media;
mod spacing;
mod style;
mod text;

use crate::models::{WidgetConfig, WidgetElement};
use crate::receipt::SkippedElement;
use serde_json::{json, Value};
use std::cell::Cell;

pub(crate) use style::sf_symbol_glyph_public;

thread_local! {
    /// Clock pinned for one transpile (`board_card`, tests); `None` = system time.
    static NOW_MS: Cell<Option<u64>> = const { Cell::new(None) };
}

/// "Now" for time-driven values (timer text, `startAt`/`endAt` progress). The card is
/// static: the host rebuilds it on its timers.
pub(crate) fn now_ms() -> u64 {
    NOW_MS.with(Cell::get).unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    })
}

/// Run `f` with [`now_ms`] pinned to `ms`.
fn with_now<T>(ms: u64, f: impl FnOnce() -> T) -> T {
    let prev = NOW_MS.with(|c| c.replace(Some(ms)));
    let out = f();
    NOW_MS.with(|c| c.set(prev));
    out
}

/// Fraction `0…1` of `start..end` (Unix ms) elapsed at [`now_ms`]; `None` without a valid range.
pub(crate) fn time_fraction(start: Option<u64>, end: Option<u64>) -> Option<f64> {
    let (s, e) = (start?, end?);
    if e <= s {
        return None;
    }
    let now = now_ms() as f64;
    Some(((now - s as f64) / (e - s) as f64).clamp(0.0, 1.0))
}

/// Result of transpiling one IR root (plus skipped diagnostics).
#[derive(Debug, Clone)]
pub struct TranspileResult {
    pub card: Value,
    pub skipped: Vec<SkippedElement>,
}

/// Build an Adaptive Card 1.5 document from a root element.
pub fn to_adaptive_card(root: &WidgetElement) -> TranspileResult {
    let mut skipped = Vec::new();
    let body = el(root, &mut skipped);
    let mut card = json!({
        "type": "AdaptiveCard",
        "$schema": "http://adaptivecards.io/schemas/adaptive-card.json",
        "version": "1.5",
        "body": [body],
    });
    unique_ids(&mut card, &mut std::collections::HashMap::new());
    TranspileResult { card, skipped }
}

/// Style hints ride in `id` (`gap:10;fg:#fff`), so siblings with the same hints got the same
/// id, and Adaptive Cards renderers reject the card ("Collision detected for id"). Repeats
/// get a `;#n` token: readers split on `;` and match known prefixes, so it is ignored.
fn unique_ids(v: &mut Value, seen: &mut std::collections::HashMap<String, u32>) {
    match v {
        Value::Object(map) => {
            if let Some(Value::String(id)) = map.get_mut("id") {
                let n = seen.entry(id.clone()).or_insert(0);
                *n += 1;
                if *n > 1 {
                    *id = format!("{id};#{n}");
                }
            }
            for child in map.values_mut() {
                unique_ids(child, seen);
            }
        }
        Value::Array(items) => items.iter_mut().for_each(|c| unique_ids(c, seen)),
        _ => {}
    }
}

fn collect_ids<'a>(v: &'a Value, out: &mut Vec<&'a str>) {
    match v {
        Value::Object(map) => {
            if let Some(id) = map.get("id").and_then(|v| v.as_str()) {
                out.push(id);
            }
            map.values().for_each(|c| collect_ids(c, out));
        }
        Value::Array(items) => items.iter().for_each(|c| collect_ids(c, out)),
        _ => {}
    }
}

/// Pick a size branch from [`WidgetConfig`] and transpile.
pub fn to_adaptive_card_for_size(config: &WidgetConfig, size: &str) -> Option<TranspileResult> {
    let root = match size {
        "large" => config
            .large
            .as_ref()
            .or(config.small.as_ref())
            .or(config.medium.as_ref()),
        "medium" => config
            .medium
            .as_ref()
            .or(config.small.as_ref())
            .or(config.large.as_ref()),
        _ => config
            .small
            .as_ref()
            .or(config.medium.as_ref())
            .or(config.large.as_ref()),
    }?;
    Some(to_adaptive_card(root))
}

/// Structural check (no full JSON Schema dependency).
pub fn validate_card_structure(card: &Value) -> Result<(), String> {
    let obj = card
        .as_object()
        .ok_or_else(|| "card must be an object".to_string())?;
    if obj.get("type").and_then(|v| v.as_str()) != Some("AdaptiveCard") {
        return Err("type must be AdaptiveCard".into());
    }
    if obj.get("version").and_then(|v| v.as_str()) != Some("1.5") {
        return Err("version must be 1.5".into());
    }
    if !matches!(obj.get("body"), Some(Value::Array(_))) {
        return Err("body must be an array".into());
    }
    // Element ids must be unique: renderers reject the whole card otherwise.
    let mut ids = Vec::new();
    collect_ids(card, &mut ids);
    ids.sort_unstable();
    match ids.windows(2).find(|w| w[0] == w[1]) {
        Some(w) => Err(format!("duplicate id {:?}", w[0])),
        None => Ok(()),
    }
}

/// Storage keys written next to the config map for the C# provider.
/// Widgets Board card for "now": the active entry and `{{…}}` resolved with `data`
/// (`config.data` < `setWidgetData`), then the medium branch transpiled. Returns the card
/// and when it must be rebuilt (next scheduled entry). The board cannot fetch `source` by
/// itself; the host fills `data` while the app runs.
pub fn board_card(
    config: &WidgetConfig,
    host_data: Option<&Value>,
    now_ms: u64,
    locale: &str,
    instance: Option<&str>,
) -> Option<(TranspileResult, Option<u64>)> {
    let raw = serde_json::to_value(config).ok()?;
    let data = crate::resolve::merge_data([raw.get("data"), host_data]);
    // Board cards are not interactive: `state` shows its defaults.
    let state = crate::resolve::merge_data([raw.get("state")]);
    let ctx = crate::resolve::ResolveContext {
        now_ms,
        locale,
        instance,
        data: data.as_ref(),
        state: state.as_ref(),
    };
    let resolved: WidgetConfig =
        serde_json::from_value(crate::resolve::resolve(&raw, &ctx)).ok()?;
    let card = with_now(now_ms, || all_sizes_card(&resolved))?;
    Some((card, crate::resolve::next_change_ms(&raw, now_ms)))
}

/// One card for every board size: each distinct size body carries
/// `"$when": "${$host.widgetSize == \"small\" || …}"`, expanded by the Widgets Board
/// template engine. Sizes that fall back to the same branch share one body; a config
/// with a single branch yields a plain card.
fn all_sizes_card(config: &WidgetConfig) -> Option<TranspileResult> {
    let mut groups: Vec<(Vec<&str>, TranspileResult)> = Vec::new();
    for size in ["small", "medium", "large"] {
        let r = to_adaptive_card_for_size(config, size)?;
        match groups.iter_mut().find(|(_, g)| g.card == r.card) {
            Some((sizes, _)) => sizes.push(size),
            None => groups.push((vec![size], r)),
        }
    }
    if groups.len() == 1 {
        return groups.pop().map(|(_, r)| r);
    }
    let mut skipped = Vec::new();
    let mut body = Vec::new();
    for (sizes, r) in groups {
        let when = sizes
            .iter()
            .map(|s| format!("$host.widgetSize == \"{s}\""))
            .collect::<Vec<_>>()
            .join(" || ");
        for mut el in r.card["body"].as_array().cloned().unwrap_or_default() {
            if let Some(o) = el.as_object_mut() {
                o.insert("$when".into(), Value::String(format!("${{{when}}}")));
            }
            body.push(el);
        }
        skipped.extend(r.skipped);
    }
    let card = json!({
        "type": "AdaptiveCard",
        "$schema": "http://adaptivecards.io/schemas/adaptive-card.json",
        "version": "1.5",
        "body": body,
    });
    Some(TranspileResult { card, skipped })
}

pub fn ac_template_key(widget_id: &str) -> String {
    format!("ac:template:{widget_id}")
}

pub fn ac_data_key(widget_id: &str) -> String {
    format!("ac:data:{widget_id}")
}

/// Dispatch one IR element to its render group.
fn el(e: &WidgetElement, skipped: &mut Vec<SkippedElement>) -> Value {
    let v = el_inner(e, skipped);
    match style::style_of(e) {
        Some(s) => style::apply_common(v, s),
        None => v,
    }
}

fn el_inner(e: &WidgetElement, skipped: &mut Vec<SkippedElement>) -> Value {
    match e {
        // layout
        WidgetElement::VStack(v) => layout::vstack(v, skipped),
        WidgetElement::HStack(h) => layout::hstack(h, skipped),
        WidgetElement::Container(c) => layout::container(c, skipped),
        WidgetElement::Grid(g) => layout::grid(g, skipped),
        WidgetElement::ZStack(z) => layout::zstack(e, z, skipped),

        // text
        WidgetElement::Text(t) => text::text(t),
        WidgetElement::Date(d) => text::date(d),
        WidgetElement::Timer(t) => text::timer(t),
        WidgetElement::Label(l) => text::label(l),
        // Resolved before transpiling (board_card); unresolved ones draw nothing.
        WidgetElement::Repeat(_) | WidgetElement::Use(_) => {
            skipped.push(SkippedElement {
                type_name: e.type_name().into(),
                reason: "unresolved (run resolve first)".into(),
            });
            serde_json::json!({"type": "Container", "items": []})
        }

        // media
        WidgetElement::Image(i) => media::image(i),
        WidgetElement::Shape(_) => media::shape(e, skipped),
        WidgetElement::Canvas(_) => media::canvas(e, skipped),

        // data
        WidgetElement::Progress(p) => data::progress(e, p, skipped),
        WidgetElement::Gauge(_) => data::gauge(e, skipped),
        WidgetElement::Chart(_) => data::chart(e, skipped),
        WidgetElement::List(l) => data::list(l),

        // interactive
        WidgetElement::Button(b) => interactive::button(b),
        WidgetElement::Toggle(t) => interactive::toggle(t),
        WidgetElement::Link(l) => interactive::link(l, skipped),

        // spacing
        WidgetElement::Spacer(_) => spacing::spacer(),
        WidgetElement::Divider(d) => spacing::divider(d),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn fixtures_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
    }

    fn snapshots_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots/adaptive")
    }

    fn load_config(rel: &str) -> WidgetConfig {
        let path = fixtures_dir().join(rel);
        let raw =
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        // Fixtures may contain explicit nulls — strip via Value first.
        let v: Value = serde_json::from_str(&raw).expect("json");
        serde_json::from_value(v).unwrap_or_else(|e| panic!("parse {rel}: {e}"))
    }

    fn t(s: &str) -> WidgetElement {
        crate::models::text(s).into()
    }

    /// Transpile → structure check: a real card passes, and each broken invariant of AC 1.5
    /// (object, `type`, `version`, `body` array) is caught. Store keys are per widget.
    #[test]
    fn transpiled_card_passes_structure_check_and_broken_ones_fail() {
        assert_eq!(ac_template_key("w"), "ac:template:w");
        assert_eq!(ac_data_key("w"), "ac:data:w");
        let ok = to_adaptive_card(&t("x")).card;
        assert!(validate_card_structure(&ok).is_ok());
        for bad in [
            json!([]),
            json!({"type": "X", "version": "1.5", "body": []}),
            json!({"type": "AdaptiveCard", "version": "1.4", "body": []}),
            json!({"type": "AdaptiveCard", "version": "1.5"}),
            json!({"type": "AdaptiveCard", "version": "1.5", "body": {}}),
            json!({"type": "AdaptiveCard", "version": "1.5", "body": [{"id": "gap:6"}, {"id": "gap:6"}]}),
        ] {
            assert!(validate_card_structure(&bad).is_err(), "{bad}");
        }
    }

    /// Size choice: per-size layout → fallback to the nearest one → Widgets Board card with
    /// `$when` branches per host size → board card resolving data, instance, strings and
    /// timeline entries (with the next change time).
    #[test]
    fn size_fallback_then_board_branches_then_board_timeline() {
        // 1. Exact sizes, then fallbacks (small ← medium/large, medium ← large), none at all.
        let has = |c: &WidgetConfig, size: &str, want: &str| {
            to_adaptive_card_for_size(c, size)
                .unwrap()
                .card
                .to_string()
                .contains(want)
        };
        let all = WidgetConfig::small(t("S"))
            .with_medium(t("M"))
            .with_large(t("L"));
        assert!(has(&all, "large", "\"L\""));
        assert!(has(&all, "medium", "\"M\""));
        assert!(has(&all, "small", "\"S\""));
        let only_small = WidgetConfig::small(t("S"));
        assert!(has(&only_small, "large", "\"S\""));
        assert!(has(&only_small, "medium", "\"S\""));
        let only_large = WidgetConfig {
            large: Some(t("L")),
            ..Default::default()
        };
        assert!(has(&only_large, "medium", "\"L\""));
        assert!(to_adaptive_card_for_size(&WidgetConfig::default(), "small").is_none());

        // 2. Board card: one `$when` body per distinct layout; medium falls back to small.
        let cfg: WidgetConfig = serde_json::from_value(json!({
            "small": {"type": "text", "content": "S"},
            "large": {"type": "text", "content": "L"}
        }))
        .unwrap();
        let (r, _) = board_card(&cfg, None, 0, "en", None).unwrap();
        validate_card_structure(&r.card).unwrap();
        let body = r.card["body"].as_array().unwrap();
        assert_eq!(body.len(), 2, "{}", r.card);
        assert_eq!(
            body[0]["$when"],
            "${$host.widgetSize == \"small\" || $host.widgetSize == \"medium\"}"
        );
        assert!(body[0].to_string().contains("\"S\""));
        assert_eq!(body[1]["$when"], "${$host.widgetSize == \"large\"}");
        assert!(body[1].to_string().contains("\"L\""));
        // One branch: plain card, no `$when`.
        let one: WidgetConfig =
            serde_json::from_value(json!({"medium": {"type": "text", "content": "M"}})).unwrap();
        let (r, _) = board_card(&one, None, 0, "en", None).unwrap();
        assert!(!r.card.to_string().contains("$when"));

        // 3. Board timeline: host data overrides config data, instance fills in, the entry at
        //    100 is the next change; at 100 it is active (strings resolved) and nothing follows.
        let cfg: WidgetConfig = serde_json::from_value(json!({
            "data": {"city": "London", "t": 1},
            "medium": {"type": "text", "content": "{{data.city}} {{data.t}} {{instance}}"},
            "entries": [{"at": 100, "medium": {"type": "text", "content": "{{t.later}}"}}],
            "strings": {"en": {"later": "Later"}}
        }))
        .unwrap();
        let host = json!({"t": 21});
        let (now, next) = board_card(&cfg, Some(&host), 50, "en", Some("B1")).unwrap();
        let s = now.card.to_string();
        assert!(s.contains("London 21 B1"), "{s}");
        assert_eq!(next, Some(100));
        let (later, next) = board_card(&cfg, None, 100, "en", None).unwrap();
        assert!(later.card.to_string().contains("Later"));
        assert_eq!(next, None);
        assert!(board_card(&WidgetConfig::default(), None, 0, "en", None).is_none());
    }

    /// Nodes AC has no element for: zstack flattens, canvas / chart rasterize into an
    /// Image data URI (`rasterize`), and the card stays valid either way.
    #[test]
    fn non_ac_nodes_flatten_or_rasterize() {
        let cfg: WidgetConfig = serde_json::from_value(json!({
            "version": 1,
            "small": {
                "type": "vstack",
                "children": [
                    { "type": "text", "content": "hi" },
                    { "type": "canvas", "width": 10, "height": 10, "elements": [] },
                    { "type": "zstack", "children": [
                        { "type": "text", "content": "a" }
                    ] }
                ]
            }
        }))
        .unwrap();
        let result = to_adaptive_card_for_size(&cfg, "small").unwrap();
        let body = serde_json::to_string(&result.card["body"]).unwrap();
        assert!(body.contains("Container") || body.contains("Image") || body.contains("TextBlock"));
        validate_card_structure(&result.card).unwrap();

        #[cfg(feature = "rasterize")]
        {
            let cfg: WidgetConfig = serde_json::from_value(json!({
                "version": 1,
                "small": {
                    "type": "chart",
                    "chartType": "bar",
                    "chartData": [
                        { "label": "a", "value": 1 },
                        { "label": "b", "value": 2 }
                    ]
                }
            }))
            .unwrap();
            let result = to_adaptive_card_for_size(&cfg, "small").unwrap();
            assert!(result.skipped.is_empty(), "{:?}", result.skipped);
            assert_eq!(result.card["body"][0]["type"], "Image");
            let url = result.card["body"][0]["url"].as_str().unwrap();
            assert!(url.starts_with("data:image/png;base64,"));
        }
    }

    #[test]
    fn weather_small_matches_golden_or_writes_hint() {
        let cfg = load_config("presets/weather.json");
        let result = to_adaptive_card_for_size(&cfg, "small").unwrap();
        let golden_path = snapshots_dir().join("weather.small.json");
        if !golden_path.exists() {
            fs::create_dir_all(snapshots_dir()).unwrap();
            let pretty = serde_json::to_string_pretty(&result.card).unwrap();
            fs::write(&golden_path, pretty).unwrap();
            // First run creates golden — still assert structure.
            validate_card_structure(&result.card).unwrap();
            return;
        }
        let expected: Value =
            serde_json::from_str(&fs::read_to_string(&golden_path).unwrap()).unwrap();
        assert_eq!(
            result.card, expected,
            "adaptive card drift — update tests/snapshots/adaptive/weather.small.json if intentional"
        );
    }

    /// Replace PNG data URIs (font-dependent bytes) so snapshots hold across machines.
    fn redact_png(v: &mut Value) {
        match v {
            Value::String(s) if s.starts_with("data:image/png;base64,") => {
                *s = "data:image/png;base64,<png>".into();
            }
            Value::Array(a) => a.iter_mut().for_each(redact_png),
            Value::Object(o) => o.values_mut().for_each(redact_png),
            _ => {}
        }
    }

    /// v2 IR fields → AC 1.5 (`UPDATE_SNAPSHOTS=1` records). Needs `rasterize`: charts,
    /// shapes, tracks and gradients become (redacted) Image data URIs.
    #[test]
    #[cfg(feature = "rasterize")]
    fn v2_cases_match_snapshots() {
        let update = std::env::var("UPDATE_SNAPSHOTS").is_ok_and(|v| v == "1");
        let cases = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/cases");
        let mut drift = Vec::new();
        for name in [
            "v2-text.medium",
            "v2-layout.medium",
            "v2-controls.medium",
            "v2-chart.medium",
            "v2-shapes.medium",
            "v2-progress.small",
        ] {
            let case: Value = serde_json::from_str(
                &fs::read_to_string(cases.join(format!("{name}.json"))).unwrap(),
            )
            .unwrap();
            let cfg = load_config(&format!("{}.json", case["fixture"].as_str().unwrap()));
            let size = case["size"].as_str().unwrap();
            // 2027-01-15: fixtures use past times, so the result does not move with the clock.
            let result =
                with_now(1_800_000_000_000, || to_adaptive_card_for_size(&cfg, size)).unwrap();
            validate_card_structure(&result.card).unwrap();
            assert!(result.skipped.is_empty(), "{name}: {:?}", result.skipped);
            let mut card = result.card;
            redact_png(&mut card);
            let path = snapshots_dir().join(format!("{name}.json"));
            let pretty = serde_json::to_string_pretty(&card).unwrap();
            if update || !path.exists() {
                fs::write(&path, pretty).unwrap();
                continue;
            }
            let expected: Value =
                serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
            if expected != card {
                drift.push(name);
            }
        }
        assert!(
            drift.is_empty(),
            "adaptive drift (UPDATE_SNAPSHOTS=1 to accept): {drift:?}"
        );
    }

    #[test]
    fn v2_mapping_details() {
        let card = |v: Value| -> Value {
            let el: WidgetElement = serde_json::from_value(v).unwrap();
            with_now(1_500, || to_adaptive_card(&el)).card["body"][0].clone()
        };
        // hstack spaceBetween: stretch filler columns between the children.
        let h = card(
            json!({"type": "hstack", "justify": "spaceBetween", "children": [
            {"type": "text", "content": "a"}, {"type": "text", "content": "b"}]}),
        );
        let widths: Vec<_> = h["columns"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["width"].clone())
            .collect();
        assert_eq!(widths, vec![json!("auto"), json!("stretch"), json!("auto")]);
        // vstack end: leading filler + bottom alignment.
        let v = card(
            json!({"type": "vstack", "justify": "end", "children": [{"type": "text", "content": "a"}]}),
        );
        assert_eq!(v["verticalContentAlignment"], "Bottom");
        assert_eq!(v["items"][0]["height"], "stretch");
        // layoutPriority: the top-priority child keeps its size, others stretch.
        let p = card(json!({"type": "hstack", "children": [
            {"type": "text", "content": "keep", "layoutPriority": 1}, {"type": "text", "content": "shrink"}]}));
        assert_eq!(p["columns"][0]["width"], "auto");
        assert_eq!(p["columns"][1]["width"], "stretch");
        // text: lineLimit → maxLines; styled → RichTextBlock TextRun.
        let t = card(json!({"type": "text", "content": "x", "lineLimit": 3}));
        assert_eq!(
            (t["maxLines"].clone(), t["wrap"].clone()),
            (json!(3), json!(true))
        );
        let r =
            card(json!({"type": "text", "content": "x", "italic": true, "strikethrough": true}));
        assert_eq!(r["type"], "RichTextBlock");
        assert_eq!(r["inlines"][0]["italic"], true);
        assert_eq!(r["inlines"][0]["strikethrough"], true);
        let m = card(json!({"type": "text", "content": "12:05", "monospacedDigit": true}));
        assert_eq!(m["fontType"], "Monospace");
        let m = card(json!({"type": "text", "content": "Total 5", "monospacedDigit": true}));
        assert!(m.get("fontType").is_none());
        // button: variant / disabled.
        let b = card(
            json!({"type": "button", "label": "Go", "variant": "prominent", "disabled": true, "action": "go"}),
        );
        assert_eq!(b["actions"][0]["style"], "positive");
        assert_eq!(b["actions"][0]["isEnabled"], false);
        // progress startAt/endAt at now = 1500 → half.
        let pr = card(json!({"type": "progress", "value": 0, "startAt": 1000, "endAt": 2000}));
        assert_eq!(pr["items"][0]["columns"][0]["width"], 50);
        // margin + minHeight wrappers.
        let mg = card(
            json!({"type": "text", "content": "x", "margin": {"top": 4, "leading": 8}, "frame": {"minHeight": 30}}),
        );
        assert_eq!(mg["type"], "ColumnSet");
        assert_eq!(mg["columns"][0]["width"], "8px");
        let inner = &mg["columns"][1]["items"][0];
        assert_eq!(inner["items"][0]["minHeight"], "4px");
        assert_eq!(inner["items"][1]["minHeight"], "30px");
        // image: palette primary color, fallback glyph, circle clip, content mode.
        let i = card(
            json!({"type": "image", "url": "https://x/y.png", "clipShape": "circle", "contentMode": "center"}),
        );
        assert_eq!(
            (i["style"].clone(), i["size"].clone()),
            (json!("Person"), json!("Auto"))
        );
        let f = card(json!({"type": "image", "fallbackSystemName": "photo"}));
        assert_ne!(f["text"], "•");
    }

    /// Fixture sweep: the weather preset is one Container in a valid card, and every
    /// `tests/cases` fixture transpiles to a structurally valid card.
    #[test]
    fn fixtures_transpile_to_valid_cards() {
        let cfg = load_config("presets/weather.json");
        let result = to_adaptive_card_for_size(&cfg, "small").expect("small layout");
        validate_card_structure(&result.card).unwrap();
        assert_eq!(result.card["body"].as_array().unwrap().len(), 1);
        assert_eq!(result.card["body"][0]["type"], "Container");

        let cases_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/cases");
        let mut checked = 0;
        for entry in fs::read_dir(&cases_dir).unwrap().flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let case: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
            let fixture = case["fixture"].as_str().unwrap_or("");
            let size = case["size"].as_str().unwrap_or("small");
            let cfg_path = fixtures_dir().join(format!("{fixture}.json"));
            if !cfg_path.exists() {
                continue;
            }
            let cfg = load_config(&format!("{fixture}.json"));
            let Some(result) = to_adaptive_card_for_size(&cfg, size) else {
                continue;
            };
            validate_card_structure(&result.card)
                .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            checked += 1;
        }
        assert!(checked > 10, "only {checked} cases transpiled");
    }
}
