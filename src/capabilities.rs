//! Platform capability matrix for widget IR elements.
//!
//! Source of truth for what each renderer supports. Used for docs generation
//! and non-blocking warnings on [`crate::Widget::set_widget_config`](set path).

use crate::models::{WidgetConfig, WidgetElement};

/// Target renderer family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WidgetPlatform {
    Ios,
    Macos,
    Android,
    Desktop,
    /// Windows Widgets Board via Adaptive Cards (WinAppSDK).
    Windows,
}

impl WidgetPlatform {
    pub fn all() -> [WidgetPlatform; 5] {
        [
            WidgetPlatform::Ios,
            WidgetPlatform::Macos,
            WidgetPlatform::Android,
            WidgetPlatform::Desktop,
            WidgetPlatform::Windows,
        ]
    }

    pub fn as_str(self) -> &'static str {
        match self {
            WidgetPlatform::Ios => "ios",
            WidgetPlatform::Macos => "macos",
            WidgetPlatform::Android => "android",
            WidgetPlatform::Desktop => "desktop",
            WidgetPlatform::Windows => "windows",
        }
    }

    /// Platform matching the current compile target.
    pub fn current() -> WidgetPlatform {
        if cfg!(target_os = "ios") {
            WidgetPlatform::Ios
        } else if cfg!(target_os = "macos") {
            WidgetPlatform::Macos
        } else if cfg!(target_os = "android") {
            WidgetPlatform::Android
        } else if cfg!(target_os = "windows") {
            // Prefer Widgets Board Adaptive Cards on Windows.
            WidgetPlatform::Windows
        } else {
            WidgetPlatform::Desktop
        }
    }
}

/// How well an element (or feature) is supported on a platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Support {
    Full,
    Degraded,
    Unsupported,
}

impl Support {
    pub fn as_str(self) -> &'static str {
        match self {
            Support::Full => "full",
            Support::Degraded => "degraded",
            Support::Unsupported => "unsupported",
        }
    }
}

/// One matrix cell.
#[derive(Debug, Clone, Copy)]
pub struct CapabilityEntry {
    pub element: &'static str,
    pub platform: WidgetPlatform,
    pub support: Support,
    pub note: &'static str,
}

/// Warning produced while walking a config tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityWarning {
    pub path: String,
    pub element: String,
    pub platform: WidgetPlatform,
    pub support: Support,
    pub note: String,
}

/// Wire `type` names for every [`WidgetElement`] variant (serde rename).
pub const ELEMENT_TYPES: &[&str] = &[
    "vstack",
    "hstack",
    "zstack",
    "grid",
    "container",
    "text",
    "image",
    "progress",
    "gauge",
    "button",
    "toggle",
    "divider",
    "spacer",
    "date",
    "chart",
    "list",
    "link",
    "shape",
    "timer",
    "canvas",
    "label",
];

pub use crate::snapshot::{CORE_ELEMENTS, EXTENDED_ELEMENTS};
/// Feature flags checked in addition to element type (e.g. image.url).
pub const FEATURE_KEYS: &[&str] = &[
    "image.url",
    "image.systemName",
    "background.gradient",
    "canvas.path",
    "timer.live",
];

fn cell(
    element: &'static str,
    platform: WidgetPlatform,
    support: Support,
    note: &'static str,
) -> CapabilityEntry {
    CapabilityEntry {
        element,
        platform,
        support,
        note,
    }
}

fn apple_full(element: &'static str) -> [CapabilityEntry; 2] {
    [
        cell(element, WidgetPlatform::Ios, Support::Full, ""),
        cell(element, WidgetPlatform::Macos, Support::Full, ""),
    ]
}

/// Full capability table (elements + feature keys).
pub fn capability_table() -> Vec<CapabilityEntry> {
    use Support::*;
    use WidgetPlatform::*;

    let mut out = Vec::with_capacity(ELEMENT_TYPES.len() * 5 + FEATURE_KEYS.len() * 5);

    let layout = [
        "vstack", "hstack", "grid", "container", "text", "spacer", "divider", "label",
        "progress", "button", "toggle", "date", "link",
    ];
    for el in layout {
        out.extend_from_slice(&apple_full(el));
        out.push(cell(el, Android, Full, ""));
        out.push(cell(el, Desktop, Full, ""));
        out.push(cell(el, Windows, Full, "Adaptive Cards 1.5"));
    }

    // zstack — degraded flatten; shape — rasterized PNG
    out.extend_from_slice(&apple_full("zstack"));
    out.push(cell("zstack", Android, Full, ""));
    out.push(cell("zstack", Desktop, Full, ""));
    out.push(cell("zstack", Windows, Degraded, "flattened Container, no overlay"));

    out.extend_from_slice(&apple_full("shape"));
    out.push(cell("shape", Android, Full, ""));
    out.push(cell("shape", Desktop, Full, ""));
    out.push(cell("shape", Windows, Degraded, "rasterized PNG"));

    out.extend_from_slice(&apple_full("gauge"));
    out.push(cell("gauge", Android, Full, ""));
    out.push(cell("gauge", Desktop, Full, ""));
    out.push(cell("gauge", Windows, Degraded, "rasterized PNG"));

    // image: systemName / url differ
    out.extend_from_slice(&apple_full("image"));
    out.push(cell("image", Android, Degraded, "systemName via glyph map; url via localPath preprocess"));
    out.push(cell("image", Desktop, Full, ""));
    out.push(cell("image", Windows, Degraded, "url/data URI; systemName unsupported"));

    out.extend_from_slice(&apple_full("chart"));
    out.push(cell("chart", Android, Degraded, "simplified bar/line rendering"));
    out.push(cell("chart", Desktop, Full, "SVG"));
    out.push(cell("chart", Windows, Degraded, "rasterized PNG"));

    out.extend_from_slice(&apple_full("list"));
    out.push(cell("list", Android, Full, "Glance LazyColumn; depth/children limited"));
    out.push(cell("list", Desktop, Full, ""));
    out.push(cell("list", Windows, Degraded, "flattened TextBlocks"));

    out.extend_from_slice(&apple_full("timer"));
    out.push(cell("timer", Android, Degraded, "static snapshot, not live Chronometer in all hosts"));
    out.push(cell("timer", Desktop, Full, "setInterval"));
    out.push(cell("timer", Windows, Degraded, "static TextBlock of targetDate"));

    out.extend_from_slice(&apple_full("canvas"));
    out.push(cell("canvas", Android, Degraded, "bitmap canvas; path support limited"));
    out.push(cell("canvas", Desktop, Full, "SVG"));
    out.push(cell("canvas", Windows, Degraded, "rasterized PNG"));

    // Feature rows
    out.push(cell("image.url", Ios, Unsupported, "prefetch into shared container not wired"));
    out.push(cell("image.url", Macos, Unsupported, "prefetch into shared container not wired"));
    out.push(cell("image.url", Android, Full, "preprocess to localPath on setWidgetConfig"));
    out.push(cell("image.url", Desktop, Full, ""));
    out.push(cell("image.url", Windows, Full, "Adaptive Cards Image.url"));

    out.push(cell("image.systemName", Ios, Full, "SF Symbols"));
    out.push(cell("image.systemName", Macos, Full, "SF Symbols"));
    out.push(cell("image.systemName", Android, Degraded, "glyph / drawable name map"));
    out.push(cell("image.systemName", Desktop, Degraded, "placeholder glyph"));
    out.push(cell("image.systemName", Windows, Unsupported, "no SF Symbols on Adaptive Cards"));

    out.push(cell("background.gradient", Ios, Degraded, "linear primary; radial/angular limited"));
    out.push(cell("background.gradient", Macos, Degraded, "linear primary; radial/angular limited"));
    out.push(cell("background.gradient", Android, Degraded, "first color stop only (Glance)"));
    out.push(cell("background.gradient", Desktop, Full, "linear/radial/angular CSS/SVG"));
    out.push(cell("background.gradient", Windows, Unsupported, "Container style=emphasis only"));

    out.push(cell("canvas.path", Ios, Degraded, "M/L/H/V/Z subset"));
    out.push(cell("canvas.path", Macos, Degraded, "M/L/H/V/Z subset"));
    out.push(cell("canvas.path", Android, Degraded, "limited path commands"));
    out.push(cell("canvas.path", Desktop, Full, "SVG path"));
    out.push(cell("canvas.path", Windows, Degraded, "rasterized via SVG"));

    out.push(cell("timer.live", Ios, Full, "Text(..., .timer)"));
    out.push(cell("timer.live", Macos, Full, "Text(..., .timer)"));
    out.push(cell("timer.live", Android, Unsupported, "no live timer in Glance snapshot"));
    out.push(cell("timer.live", Desktop, Full, "JS interval"));
    out.push(cell("timer.live", Windows, Unsupported, "static only"));

    out
}

use std::sync::OnceLock;

/// Lookup support for an element or feature key.
pub fn support_for(element: &str, platform: WidgetPlatform) -> CapabilityEntry {
    static TABLE: OnceLock<Vec<CapabilityEntry>> = OnceLock::new();
    let table = TABLE.get_or_init(capability_table);
    table
        .iter()
        .find(|e| e.element == element && e.platform == platform)
        .copied()
        .unwrap_or(CapabilityEntry {
            element: "",
            platform,
            support: Support::Unsupported,
            note: "unknown element",
        })
}

impl WidgetElement {
    /// Serde `type` tag for this element.
    pub fn type_name(&self) -> &'static str {
        match self {
            WidgetElement::VStack { .. } => "vstack",
            WidgetElement::HStack { .. } => "hstack",
            WidgetElement::ZStack { .. } => "zstack",
            WidgetElement::Grid { .. } => "grid",
            WidgetElement::Container { .. } => "container",
            WidgetElement::Text { .. } => "text",
            WidgetElement::Image { .. } => "image",
            WidgetElement::Progress { .. } => "progress",
            WidgetElement::Gauge { .. } => "gauge",
            WidgetElement::Button { .. } => "button",
            WidgetElement::Toggle { .. } => "toggle",
            WidgetElement::Divider { .. } => "divider",
            WidgetElement::Spacer { .. } => "spacer",
            WidgetElement::Date { .. } => "date",
            WidgetElement::Chart { .. } => "chart",
            WidgetElement::List { .. } => "list",
            WidgetElement::Link { .. } => "link",
            WidgetElement::Shape { .. } => "shape",
            WidgetElement::Timer { .. } => "timer",
            WidgetElement::Canvas { .. } => "canvas",
            WidgetElement::Label { .. } => "label",
        }
    }

    fn children_ref(&self) -> &[WidgetElement] {
        match self {
            WidgetElement::VStack { children, .. }
            | WidgetElement::HStack { children, .. }
            | WidgetElement::ZStack { children, .. }
            | WidgetElement::Grid { children, .. }
            | WidgetElement::Container { children, .. }
            | WidgetElement::Link { children, .. } => children,
            _ => &[],
        }
    }

    fn style_background_is_gradient(&self) -> bool {
        use crate::models::{BackgroundValue, ElementStyle};
        let style: Option<&ElementStyle> = match self {
            WidgetElement::VStack { style, .. }
            | WidgetElement::HStack { style, .. }
            | WidgetElement::ZStack { style, .. }
            | WidgetElement::Grid { style, .. }
            | WidgetElement::Container { style, .. }
            | WidgetElement::Text { style, .. }
            | WidgetElement::Image { style, .. }
            | WidgetElement::Progress { style, .. }
            | WidgetElement::Gauge { style, .. }
            | WidgetElement::Button { style, .. }
            | WidgetElement::Toggle { style, .. }
            | WidgetElement::Divider { style, .. }
            | WidgetElement::Date { style, .. }
            | WidgetElement::Chart { style, .. }
            | WidgetElement::List { style, .. }
            | WidgetElement::Link { style, .. }
            | WidgetElement::Shape { style, .. }
            | WidgetElement::Timer { style, .. }
            | WidgetElement::Canvas { style, .. }
            | WidgetElement::Label { style, .. } => Some(style),
            WidgetElement::Spacer { .. } => None,
        };
        matches!(style.and_then(|s| s.background.as_ref()), Some(BackgroundValue::Gradient(_)))
    }
}

fn push_warn(
    out: &mut Vec<CapabilityWarning>,
    path: &str,
    element: &str,
    platform: WidgetPlatform,
) {
    let entry = support_for(element, platform);
    if entry.support == Support::Full {
        return;
    }
    out.push(CapabilityWarning {
        path: path.to_string(),
        element: element.to_string(),
        platform,
        support: entry.support,
        note: entry.note.to_string(),
    });
}

fn walk_element(el: &WidgetElement, path: &str, platform: WidgetPlatform, out: &mut Vec<CapabilityWarning>) {
    let ty = el.type_name();
    push_warn(out, path, ty, platform);

    if el.style_background_is_gradient() {
        push_warn(out, path, "background.gradient", platform);
    }

    if let WidgetElement::Image { url, system_name, .. } = el {
        if url.as_ref().map(|s| !s.is_empty()).unwrap_or(false) {
            push_warn(out, path, "image.url", platform);
        }
        if system_name.as_ref().map(|s| !s.is_empty()).unwrap_or(false) {
            push_warn(out, path, "image.systemName", platform);
        }
    }

    if let WidgetElement::Timer { .. } = el {
        push_warn(out, path, "timer.live", platform);
    }

    if let WidgetElement::Canvas { elements, .. } = el {
        if elements.iter().any(|c| matches!(c, crate::models::CanvasDrawCommand::Path { .. })) {
            push_warn(out, path, "canvas.path", platform);
        }
    }

    for (i, child) in el.children_ref().iter().enumerate() {
        walk_element(child, &format!("{path}/{ty}[{i}]"), platform, out);
    }
}

/// Walk config layouts and collect non-full capability warnings for `platform`.
pub fn validate_config(config: &WidgetConfig, platform: WidgetPlatform) -> Vec<CapabilityWarning> {
    let mut out = Vec::new();
    if let Some(el) = &config.small {
        walk_element(el, "small", platform, &mut out);
    }
    if let Some(el) = &config.medium {
        walk_element(el, "medium", platform, &mut out);
    }
    if let Some(el) = &config.large {
        walk_element(el, "large", platform, &mut out);
    }
    out
}

/// Log warnings for the current compile target (non-blocking).
pub fn log_capabilities(config: &WidgetConfig) {
    let platform = WidgetPlatform::current();
    for w in validate_config(config, platform) {
        log::warn!(
            "widget capability {}: {} at {} on {} — {}",
            w.support.as_str(),
            w.element,
            w.path,
            w.platform.as_str(),
            w.note
        );
    }
}

/// Render markdown capability matrix (elements only, not feature keys).
pub fn render_capability_matrix_md() -> String {
    let mut md = String::from(
        "# Capability matrix (element × platform)\n\n\
         Generated from `tauri_plugin_widgets::capabilities`. Do not edit by hand.\n\n\
         ## Core (strict snapshot contract)\n\n\
         | Element | iOS | macOS | Android | Desktop | Windows |\n\
         |---------|-----|-------|---------|---------|----------|\n",
    );

    for el in CORE_ELEMENTS {
        let ios = support_for(el, WidgetPlatform::Ios);
        let mac = support_for(el, WidgetPlatform::Macos);
        let and = support_for(el, WidgetPlatform::Android);
        let desk = support_for(el, WidgetPlatform::Desktop);
        let win = support_for(el, WidgetPlatform::Windows);
        md.push_str(&format!(
            "| `{el}` | {} | {} | {} | {} | {} |\n",
            cell_md(&ios),
            cell_md(&mac),
            cell_md(&and),
            cell_md(&desk),
            cell_md(&win),
        ));
    }

    md.push_str(
        "\n## Extended (best-effort, platform-dependent)\n\n\
         | Element | iOS | macOS | Android | Desktop | Windows |\n\
         |---------|-----|-------|---------|---------|----------|\n",
    );
    for el in EXTENDED_ELEMENTS {
        let ios = support_for(el, WidgetPlatform::Ios);
        let mac = support_for(el, WidgetPlatform::Macos);
        let and = support_for(el, WidgetPlatform::Android);
        let desk = support_for(el, WidgetPlatform::Desktop);
        let win = support_for(el, WidgetPlatform::Windows);
        md.push_str(&format!(
            "| `{el}` | {} | {} | {} | {} | {} |\n",
            cell_md(&ios),
            cell_md(&mac),
            cell_md(&and),
            cell_md(&desk),
            cell_md(&win),
        ));
    }

    md.push_str("\n## Feature notes\n\n");
    md.push_str("| Feature | iOS | macOS | Android | Desktop | Windows |\n");
    md.push_str("|---------|-----|-------|---------|---------|----------|\n");
    for feat in FEATURE_KEYS {
        let ios = support_for(feat, WidgetPlatform::Ios);
        let mac = support_for(feat, WidgetPlatform::Macos);
        let and = support_for(feat, WidgetPlatform::Android);
        let desk = support_for(feat, WidgetPlatform::Desktop);
        let win = support_for(feat, WidgetPlatform::Windows);
        md.push_str(&format!(
            "| `{feat}` | {} | {} | {} | {} | {} |\n",
            cell_md(&ios),
            cell_md(&mac),
            cell_md(&and),
            cell_md(&desk),
            cell_md(&win),
        ));
    }
    md
}

fn cell_md(e: &CapabilityEntry) -> String {
    if e.note.is_empty() {
        e.support.as_str().to_string()
    } else {
        format!("{} ({})", e.support.as_str(), e.note)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ChartDataPoint, ChartType, WidgetConfig, WidgetElement};

    #[test]
    fn all_element_types_have_five_platforms() {
        for el in ELEMENT_TYPES {
            for p in WidgetPlatform::all() {
                let e = support_for(el, p);
                assert_eq!(e.element, *el);
                assert_eq!(e.platform, p);
            }
        }
    }

    #[test]
    fn validate_flags_image_url_on_ios() {
        let cfg = WidgetConfig {
            version: 1,
            small: Some(WidgetElement::Image {
                system_name: None,
                data: None,
                url: Some("https://example.com/a.png".into()),
                size: Some(32.0),
                color: None,
                content_mode: None,
                style: Default::default(),
            }),
            medium: None,
            large: None,
        };
        let warns = validate_config(&cfg, WidgetPlatform::Ios);
        assert!(
            warns.iter().any(|w| w.element == "image.url" && w.support == Support::Unsupported),
            "{warns:?}"
        );
    }

    #[test]
    fn matrix_markdown_mentions_vstack() {
        let md = render_capability_matrix_md();
        assert!(md.contains("`vstack`"));
        assert!(md.contains("image.url"));
    }

    #[test]
    fn chart_roundtrip_type_name() {
        let el = WidgetElement::Chart {
            chart_type: ChartType::Bar,
            chart_data: vec![ChartDataPoint {
                label: "a".into(),
                value: 1.0,
                color: None,
            }],
            tint: None,
            style: Default::default(),
        };
        assert_eq!(el.type_name(), "chart");
    }
}

#[cfg(test)]
mod write_docs {
    #[test]
    fn capability_matrix_doc_matches() {
        let expected = super::render_capability_matrix_md();
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/capability-matrix.md");
        if !path.exists() {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, &expected).unwrap();
            return;
        }
        let on_disk = std::fs::read_to_string(&path).unwrap();
        assert_eq!(
            on_disk, expected,
            "docs/capability-matrix.md drifted — regenerate with:\n\
             cargo test --lib write_docs::capability_matrix_doc_matches -- --ignored\n\
             or delete the file and re-run this test"
        );
    }
}
