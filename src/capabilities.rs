//! Platform capability matrix for widget IR elements.
//!
//! Source of truth for what each renderer supports. Used for docs generation
//! and non-blocking warnings on [`crate::Widget::set_widget_config`](set path).

use crate::models::{
    ButtonElement, CanvasElement, ChartElement, ContainerElement, DateElement, DividerElement,
    GaugeElement, GridElement, HStackElement, ImageElement, LabelElement, LinkElement, ListElement,
    ProgressElement, ShapeElement, TextElement, TimerElement, ToggleElement, VStackElement,
    WidgetConfig, WidgetElement, ZStackElement,
};

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
    "repeat",
    "use",
];

pub use crate::snapshot::{CORE_ELEMENTS, EXTENDED_ELEMENTS};
/// Feature flags checked in addition to element type (e.g. image.url).
pub const FEATURE_KEYS: &[&str] = &[
    "image.url",
    "image.systemName",
    "background.gradient",
    "canvas.path",
    "timer.live",
    "config.entries",
    "config.strings",
    "config.source",
    "config.instance",
    "style.privacySensitive",
    "style.accessibilityLabel",
    "style.bind",
    "style.when",
    "style.transition",
    "config.state",
    "activity",
    "activity.template",
    "activity.island",
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
        "vstack",
        "hstack",
        "grid",
        "container",
        "text",
        "spacer",
        "divider",
        "label",
        "progress",
        "button",
        "toggle",
        "date",
        "link",
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
    out.push(cell(
        "zstack",
        Windows,
        Degraded,
        "rasterized PNG overlay when possible; else flattened Container",
    ));

    out.extend_from_slice(&apple_full("shape"));
    out.push(cell("shape", Android, Full, ""));
    out.push(cell("shape", Desktop, Full, ""));
    out.push(cell("shape", Windows, Degraded, "rasterized PNG"));

    out.extend_from_slice(&apple_full("gauge"));
    out.push(cell("gauge", Android, Full, ""));
    out.push(cell("gauge", Desktop, Full, ""));
    out.push(cell("gauge", Windows, Degraded, "rasterized PNG"));

    // image (url/data): Full everywhere — systemName fidelity lives in feature row.
    out.extend_from_slice(&apple_full("image"));
    out.push(cell(
        "image",
        Android,
        Full,
        "url via localPath preprocess; see image.systemName",
    ));
    out.push(cell("image", Desktop, Full, ""));
    out.push(cell(
        "image",
        Windows,
        Full,
        "url/data URI; see image.systemName",
    ));

    out.extend_from_slice(&apple_full("chart"));
    out.push(cell("chart", Android, Full, "bitmap bar/line/area/pie"));
    out.push(cell("chart", Desktop, Full, "SVG"));
    out.push(cell("chart", Windows, Degraded, "rasterized PNG"));

    out.extend_from_slice(&apple_full("list"));
    out.push(cell(
        "list",
        Android,
        Full,
        "Column chunking; soft cap ~50 items",
    ));
    out.push(cell("list", Desktop, Full, ""));
    out.push(cell("list", Windows, Degraded, "Adaptive Cards Table"));

    out.extend_from_slice(&apple_full("timer"));
    out.push(cell(
        "timer",
        Android,
        Full,
        "Chronometer via AndroidRemoteViews",
    ));
    out.push(cell("timer", Desktop, Full, "setInterval"));
    out.push(cell(
        "timer",
        Windows,
        Degraded,
        "provider minute push + static TextBlock",
    ));

    out.extend_from_slice(&apple_full("canvas"));
    out.push(cell(
        "canvas",
        Android,
        Degraded,
        "bitmap canvas (full SVG path via PathParser)",
    ));
    out.push(cell("canvas", Desktop, Full, "SVG"));
    out.push(cell("canvas", Windows, Degraded, "rasterized PNG"));

    // Feature rows
    out.push(cell(
        "image.url",
        Ios,
        Full,
        "host prefetch to data URI on setWidgetConfig",
    ));
    out.push(cell(
        "image.url",
        Macos,
        Full,
        "host prefetch to data URI on setWidgetConfig",
    ));
    out.push(cell(
        "image.url",
        Android,
        Full,
        "preprocess to localPath on setWidgetConfig",
    ));
    out.push(cell("image.url", Desktop, Full, ""));
    out.push(cell("image.url", Windows, Full, "Adaptive Cards Image.url"));

    out.push(cell("image.systemName", Ios, Full, "SF Symbols"));
    out.push(cell("image.systemName", Macos, Full, "SF Symbols"));
    out.push(cell(
        "image.systemName",
        Android,
        Degraded,
        "SF→Material / emoji map (not SF Symbols)",
    ));
    out.push(cell(
        "image.systemName",
        Desktop,
        Degraded,
        "SF→Material / emoji map (not SF Symbols)",
    ));
    out.push(cell(
        "image.systemName",
        Windows,
        Degraded,
        "emoji TextBlock by default; glyph PNG Image with feature rasterize",
    ));

    out.push(cell(
        "background.gradient",
        Ios,
        Full,
        "linear/radial/angular SwiftUI",
    ));
    out.push(cell(
        "background.gradient",
        Macos,
        Full,
        "linear/radial/angular SwiftUI",
    ));
    out.push(cell(
        "background.gradient",
        Android,
        Full,
        "baked bitmap at LocalSize / frame",
    ));
    out.push(cell(
        "background.gradient",
        Desktop,
        Full,
        "linear/radial/angular CSS/SVG",
    ));
    out.push(cell(
        "background.gradient",
        Windows,
        Degraded,
        "rasterized PNG backgroundImage when rasterize enabled",
    ));

    out.push(cell("canvas.path", Ios, Full, "SVG path grammar"));
    out.push(cell("canvas.path", Macos, Full, "SVG path grammar"));
    out.push(cell(
        "canvas.path",
        Android,
        Full,
        "PathParser full SVG path",
    ));
    out.push(cell("canvas.path", Desktop, Full, "SVG path"));
    out.push(cell("canvas.path", Windows, Degraded, "rasterized via SVG"));

    out.push(cell("timer.live", Ios, Full, "Text(..., .timer)"));
    out.push(cell("timer.live", Macos, Full, "Text(..., .timer)"));
    out.push(cell(
        "timer.live",
        Android,
        Full,
        "Chronometer via AndroidRemoteViews",
    ));
    out.push(cell("timer.live", Desktop, Full, "JS interval"));
    out.push(cell(
        "timer.live",
        Windows,
        Degraded,
        "provider pushes UpdateWidget ~1/min",
    ));

    // Render-time resolution (src/resolve.rs and its Swift / Kotlin / TS ports).
    out.push(cell(
        "config.entries",
        Ios,
        Full,
        "WidgetKit timeline entries",
    ));
    out.push(cell(
        "config.entries",
        Macos,
        Full,
        "WidgetKit timeline entries",
    ));
    out.push(cell(
        "config.entries",
        Android,
        Full,
        "inexact alarm redraw (may run late in Doze)",
    ));
    out.push(cell("config.entries", Desktop, Full, "renderer timer"));
    out.push(cell(
        "config.entries",
        Windows,
        Degraded,
        "card rebuilt by the host while the app runs",
    ));
    for p in [Ios, Macos, Android, Desktop, Windows] {
        out.push(cell("config.strings", p, Full, ""));
    }
    out.push(cell(
        "config.source",
        Ios,
        Full,
        "fetched in the timeline (8 s budget)",
    ));
    out.push(cell(
        "config.source",
        Macos,
        Full,
        "fetched in the timeline (8 s budget)",
    ));
    out.push(cell(
        "config.source",
        Android,
        Full,
        "fetched on redraw, alarm when due",
    ));
    out.push(cell(
        "config.source",
        Desktop,
        Full,
        "fetched while the window is open",
    ));
    out.push(cell(
        "config.source",
        Windows,
        Unsupported,
        "board cannot fetch; push setWidgetData from the app",
    ));
    out.push(cell(
        "config.instance",
        Ios,
        Full,
        "Edit Widget choice (TauriInstanceIntent)",
    ));
    out.push(cell(
        "config.instance",
        Macos,
        Full,
        "Edit Widget choice (TauriInstanceIntent)",
    ));
    out.push(cell("config.instance", Android, Full, "appWidgetId"));
    out.push(cell(
        "config.instance",
        Desktop,
        Full,
        "window instance / label",
    ));
    out.push(cell("config.instance", Windows, Full, "Widgets Board id"));
    out.push(cell(
        "style.privacySensitive",
        Ios,
        Full,
        ".privacySensitive()",
    ));
    out.push(cell(
        "style.privacySensitive",
        Macos,
        Full,
        ".privacySensitive()",
    ));
    out.push(cell(
        "style.privacySensitive",
        Android,
        Unsupported,
        "no home-screen redaction",
    ));
    out.push(cell(
        "style.privacySensitive",
        Desktop,
        Unsupported,
        "data-privacy-sensitive attribute only",
    ));
    out.push(cell(
        "style.privacySensitive",
        Windows,
        Unsupported,
        "no redaction",
    ));
    out.push(cell("style.accessibilityLabel", Ios, Full, ""));
    out.push(cell("style.accessibilityLabel", Macos, Full, ""));
    out.push(cell(
        "style.accessibilityLabel",
        Android,
        Full,
        "Glance semantics contentDescription",
    ));
    out.push(cell(
        "style.accessibilityLabel",
        Desktop,
        Full,
        "aria-label",
    ));
    out.push(cell("style.accessibilityLabel", Windows, Unsupported, ""));
    for p in [Ios, Macos, Android, Desktop, Windows] {
        out.push(cell("style.bind", p, Full, ""));
    }
    // Expanded by the shared resolve step before any renderer draws.
    for p in [Ios, Macos, Android, Desktop, Windows] {
        out.push(cell("repeat", p, Full, "expanded before rendering"));
        out.push(cell("use", p, Full, "expanded before rendering"));
        out.push(cell("style.when", p, Full, "evaluated before rendering"));
    }
    // Contract v3.
    for p in [Ios, Macos] {
        out.push(cell(
            "style.transition",
            p,
            Full,
            "contentTransition(.numericText()) / .transition",
        ));
        out.push(cell(
            "config.state",
            p,
            Full,
            "set taps run an AppIntent in the extension (iOS 17 / macOS 14)",
        ));
    }
    out.push(cell(
        "style.transition",
        Android,
        Unsupported,
        "ignored (Glance has no content transitions)",
    ));
    out.push(cell(
        "style.transition",
        Desktop,
        Full,
        "animation when the content changes",
    ));
    out.push(cell(
        "style.transition",
        Windows,
        Unsupported,
        "ignored (Adaptive Cards)",
    ));
    out.push(cell(
        "config.state",
        Android,
        Full,
        "set taps handled by the widget's broadcast receiver",
    ));
    out.push(cell(
        "config.state",
        Desktop,
        Full,
        "set taps handled by the widget window",
    ));
    out.push(cell(
        "config.state",
        Windows,
        Degraded,
        "Widgets Board shows the state; set taps are not handled",
    ));
    for p in [Ios, Macos, Android, Desktop, Windows] {
        out.push(cell(
            "activity.template",
            p,
            Full,
            "layout stored on the device, data-only updates",
        ));
    }
    out.push(cell(
        "activity",
        Ios,
        Full,
        "Live Activity + Dynamic Island (iOS 16.2+)",
    ));
    out.push(cell(
        "activity",
        Macos,
        Degraded,
        "island pill at the notch (default) or corner window; no Lock Screen",
    ));
    out.push(cell(
        "activity",
        Android,
        Full,
        "ongoing notification; Live Update chip on Android 16",
    ));
    out.push(cell(
        "activity",
        Desktop,
        Degraded,
        "top-centre island pill (default) or corner window; no Lock Screen",
    ));
    out.push(cell(
        "activity",
        Windows,
        Degraded,
        "top-centre island pill (default) or corner window; no Lock Screen",
    ));
    out.push(cell(
        "activity.island",
        Ios,
        Full,
        "Dynamic Island compact / minimal / expanded regions",
    ));
    out.push(cell(
        "activity.island",
        Macos,
        Full,
        "pill around the notch, under the menu bar without one; hover expands",
    ));
    out.push(cell(
        "activity.island",
        Android,
        Unsupported,
        "no island; the Live Update chip shows shortCriticalText",
    ));
    out.push(cell(
        "activity.island",
        Desktop,
        Full,
        "top-centre pill: X11 dock window, Wayland layer-shell overlay",
    ));
    out.push(cell(
        "activity.island",
        Windows,
        Full,
        "top-centre topmost pill, no taskbar button, no focus steal",
    ));

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
            WidgetElement::VStack(_) => "vstack",
            WidgetElement::HStack(_) => "hstack",
            WidgetElement::ZStack(_) => "zstack",
            WidgetElement::Grid(_) => "grid",
            WidgetElement::Container(_) => "container",
            WidgetElement::Text(_) => "text",
            WidgetElement::Image(_) => "image",
            WidgetElement::Progress(_) => "progress",
            WidgetElement::Gauge(_) => "gauge",
            WidgetElement::Button(_) => "button",
            WidgetElement::Toggle(_) => "toggle",
            WidgetElement::Divider(_) => "divider",
            WidgetElement::Spacer(_) => "spacer",
            WidgetElement::Date(_) => "date",
            WidgetElement::Chart(_) => "chart",
            WidgetElement::List(_) => "list",
            WidgetElement::Link(_) => "link",
            WidgetElement::Shape(_) => "shape",
            WidgetElement::Timer(_) => "timer",
            WidgetElement::Canvas(_) => "canvas",
            WidgetElement::Label(_) => "label",
            WidgetElement::Repeat(_) => "repeat",
            WidgetElement::Use(_) => "use",
        }
    }

    fn children_ref(&self) -> &[WidgetElement] {
        match self {
            WidgetElement::VStack(VStackElement { children, .. })
            | WidgetElement::HStack(HStackElement { children, .. })
            | WidgetElement::ZStack(ZStackElement { children, .. })
            | WidgetElement::Grid(GridElement { children, .. })
            | WidgetElement::Container(ContainerElement { children, .. })
            | WidgetElement::Link(LinkElement { children, .. }) => children,
            WidgetElement::Repeat(r) => std::slice::from_ref(&*r.template),
            _ => &[],
        }
    }

    fn style_ref(&self) -> Option<&crate::models::ElementStyle> {
        match self {
            WidgetElement::VStack(VStackElement { style, .. })
            | WidgetElement::HStack(HStackElement { style, .. })
            | WidgetElement::ZStack(ZStackElement { style, .. })
            | WidgetElement::Grid(GridElement { style, .. })
            | WidgetElement::Container(ContainerElement { style, .. })
            | WidgetElement::Text(TextElement { style, .. })
            | WidgetElement::Image(ImageElement { style, .. })
            | WidgetElement::Progress(ProgressElement { style, .. })
            | WidgetElement::Gauge(GaugeElement { style, .. })
            | WidgetElement::Button(ButtonElement { style, .. })
            | WidgetElement::Toggle(ToggleElement { style, .. })
            | WidgetElement::Divider(DividerElement { style, .. })
            | WidgetElement::Date(DateElement { style, .. })
            | WidgetElement::Chart(ChartElement { style, .. })
            | WidgetElement::List(ListElement { style, .. })
            | WidgetElement::Link(LinkElement { style, .. })
            | WidgetElement::Shape(ShapeElement { style, .. })
            | WidgetElement::Timer(TimerElement { style, .. })
            | WidgetElement::Canvas(CanvasElement { style, .. })
            | WidgetElement::Label(LabelElement { style, .. }) => Some(style),
            WidgetElement::Repeat(r) => Some(&r.style),
            WidgetElement::Use(u) => Some(&u.style),
            WidgetElement::Spacer(_) => None,
        }
    }

    fn style_background_is_gradient(&self) -> bool {
        matches!(
            self.style_ref().and_then(|s| s.background.as_ref()),
            Some(crate::models::BackgroundValue::Gradient(_))
        )
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

fn walk_element(
    el: &WidgetElement,
    path: &str,
    platform: WidgetPlatform,
    out: &mut Vec<CapabilityWarning>,
) {
    let ty = el.type_name();
    push_warn(out, path, ty, platform);

    if el.style_background_is_gradient() {
        push_warn(out, path, "background.gradient", platform);
    }
    if let Some(style) = el.style_ref() {
        if style.privacy_sensitive == Some(true) {
            push_warn(out, path, "style.privacySensitive", platform);
        }
        if style
            .accessibility_label
            .as_deref()
            .is_some_and(|l| !l.is_empty())
        {
            push_warn(out, path, "style.accessibilityLabel", platform);
        }
        if style
            .transition
            .is_some_and(|t| t != crate::models::Transition::None)
        {
            push_warn(out, path, "style.transition", platform);
        }
    }
    let sets_state = match el {
        WidgetElement::Button(b) => b.set.is_some(),
        WidgetElement::Toggle(t) => t.set.is_some(),
        WidgetElement::Link(l) => l.set.is_some(),
        _ => false,
    };
    if sets_state {
        push_warn(out, path, "config.state", platform);
    }

    if let WidgetElement::Image(ImageElement {
        url, system_name, ..
    }) = el
    {
        if url.as_ref().map(|s| !s.is_empty()).unwrap_or(false) {
            push_warn(out, path, "image.url", platform);
        }
        if system_name.as_ref().map(|s| !s.is_empty()).unwrap_or(false) {
            push_warn(out, path, "image.systemName", platform);
        }
    }

    if let WidgetElement::Timer(_) = el {
        push_warn(out, path, "timer.live", platform);
    }

    if let WidgetElement::Canvas(CanvasElement { elements, .. }) = el {
        if elements
            .iter()
            .any(|c| matches!(c, crate::models::CanvasDrawCommand::Path { .. }))
        {
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
    for (i, entry) in config.entries.iter().enumerate() {
        if i == 0 {
            push_warn(&mut out, "entries", "config.entries", platform);
        }
        for (family, el) in [
            ("small", &entry.small),
            ("medium", &entry.medium),
            ("large", &entry.large),
        ] {
            if let Some(el) = el {
                walk_element(el, &format!("entries[{i}].{family}"), platform, &mut out);
            }
        }
    }
    if config.source.is_some() {
        push_warn(&mut out, "source", "config.source", platform);
    }
    if config.state.is_some() {
        push_warn(&mut out, "state", "config.state", platform);
    }
    if config
        .transition
        .is_some_and(|t| t != crate::models::Transition::None)
    {
        push_warn(&mut out, "transition", "style.transition", platform);
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

/// Machine-readable capability matrix for CLI `validate` and docs tooling.
pub fn render_capabilities_json() -> String {
    use serde_json::{json, Map, Value};

    let mut elements = Map::new();
    for el in ELEMENT_TYPES {
        let mut platforms = Map::new();
        for p in WidgetPlatform::all() {
            let e = support_for(el, p);
            platforms.insert(
                p.as_str().to_string(),
                json!({
                    "support": e.support.as_str(),
                    "note": e.note,
                }),
            );
        }
        elements.insert((*el).to_string(), Value::Object(platforms));
    }

    let mut features = Map::new();
    for feat in FEATURE_KEYS {
        let mut platforms = Map::new();
        for p in WidgetPlatform::all() {
            let e = support_for(feat, p);
            platforms.insert(
                p.as_str().to_string(),
                json!({
                    "support": e.support.as_str(),
                    "note": e.note,
                }),
            );
        }
        features.insert((*feat).to_string(), Value::Object(platforms));
    }

    let doc = json!({
        "version": 1,
        "platforms": WidgetPlatform::all().map(|p| p.as_str()),
        "core": CORE_ELEMENTS,
        "extended": EXTENDED_ELEMENTS,
        "elements": elements,
        "features": features,
    });
    format!(
        "{}\n",
        serde_json::to_string_pretty(&doc).expect("serialize capabilities.json")
    )
}

/// Render markdown capability matrix for docs (`###` headings — embed under
/// a single `## Capability matrix` on the tiers page).
pub fn render_capability_matrix_md() -> String {
    let mut notes: Vec<String> = Vec::new();
    let mut md = String::from(
        "# Capability matrix (element × platform)\n\n\
         Table is authored in `src/capabilities.rs`; this page embeds it automatically.\n\n",
    );

    md.push_str("### Element lists\n\n");
    md.push_str(&format!(
        "**Core** (`CORE_ELEMENTS` in `src/snapshot.rs`): {}\n\n",
        linked_element_list(CORE_ELEMENTS)
    ));
    md.push_str(&format!(
        "**Extended** (`EXTENDED_ELEMENTS`): {}\n\n",
        linked_element_list(EXTENDED_ELEMENTS)
    ));

    md.push_str("### Core\n\n");
    md.push_str(&platform_table_header("Element"));
    for el in CORE_ELEMENTS {
        md.push_str(&element_row(el, &mut notes));
    }

    md.push_str("\n### Extended\n\n");
    md.push_str(&platform_table_header("Element"));
    for el in EXTENDED_ELEMENTS {
        md.push_str(&element_row(el, &mut notes));
    }

    md.push_str("\n### Feature notes\n\n");
    md.push_str(&platform_table_header("Feature"));
    for feat in FEATURE_KEYS {
        md.push_str(&feature_row(feat, &mut notes));
    }

    md.push_str("\n### Choosing a surface set\n\n");
    md.push_str(
        "Pick the platforms you ship, then stay in the **full** set for that profile. \
         Degraded cells still render, but check the [Notes](#notes) and element pages.\n\n",
    );
    md.push_str(&profile_block(
        "Apple only",
        "iOS + macOS",
        &[WidgetPlatform::Ios, WidgetPlatform::Macos],
    ));
    md.push_str(&profile_block(
        "Apple + Android",
        "iOS + macOS + Android",
        &[
            WidgetPlatform::Ios,
            WidgetPlatform::Macos,
            WidgetPlatform::Android,
        ],
    ));
    md.push_str(&profile_block(
        "All five matrix columns",
        "iOS + macOS + Android + Desktop + Windows",
        &WidgetPlatform::all(),
    ));

    if !notes.is_empty() {
        md.push_str("\n### Notes\n\n");
        for (i, note) in notes.iter().enumerate() {
            md.push_str(&format!("{}. {}\n", i + 1, note));
        }
    }
    md
}

fn platform_table_header(first: &str) -> String {
    format!(
        "| {first} | iOS | macOS | Android | Desktop | Windows |\n\
         |---------|-----|-------|---------|---------|----------|\n"
    )
}

fn linked_element_list(els: &[&str]) -> String {
    els.iter()
        .map(|e| format!("[`{e}`]({})", element_doc_href(e)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn element_row(el: &str, notes: &mut Vec<String>) -> String {
    format!(
        "| [`{el}`]({}) | {} | {} | {} | {} | {} |\n",
        element_doc_href(el),
        cell_md(&support_for(el, WidgetPlatform::Ios), notes),
        cell_md(&support_for(el, WidgetPlatform::Macos), notes),
        cell_md(&support_for(el, WidgetPlatform::Android), notes),
        cell_md(&support_for(el, WidgetPlatform::Desktop), notes),
        cell_md(&support_for(el, WidgetPlatform::Windows), notes),
    )
}

fn feature_row(feat: &str, notes: &mut Vec<String>) -> String {
    format!(
        "| [`{feat}`]({}) | {} | {} | {} | {} | {} |\n",
        element_doc_href(feat),
        cell_md(&support_for(feat, WidgetPlatform::Ios), notes),
        cell_md(&support_for(feat, WidgetPlatform::Macos), notes),
        cell_md(&support_for(feat, WidgetPlatform::Android), notes),
        cell_md(&support_for(feat, WidgetPlatform::Desktop), notes),
        cell_md(&support_for(feat, WidgetPlatform::Windows), notes),
    )
}

fn profile_block(title: &str, platforms_label: &str, platforms: &[WidgetPlatform]) -> String {
    let (full_core, watch_core) = partition_by_full(CORE_ELEMENTS, platforms);
    let (full_ext, watch_ext) = partition_by_full(EXTENDED_ELEMENTS, platforms);
    let mut out = format!("#### {title}\n\nPlatforms: **{platforms_label}**.\n\n");
    out.push_str(&format!(
        "- **Full core:** {}\n",
        if full_core.is_empty() {
            "_none_".into()
        } else {
            linked_element_list(&full_core)
        }
    ));
    if !watch_core.is_empty() {
        out.push_str(&format!(
            "- **Core with degraded/unsupported cells:** {}\n",
            linked_element_list(&watch_core)
        ));
    }
    out.push_str(&format!(
        "- **Full extended:** {}\n",
        if full_ext.is_empty() {
            "_none_".into()
        } else {
            linked_element_list(&full_ext)
        }
    ));
    if !watch_ext.is_empty() {
        out.push_str(&format!(
            "- **Extended with degraded/unsupported cells:** {}\n",
            linked_element_list(&watch_ext)
        ));
    }
    out.push('\n');
    out
}

fn partition_by_full<'a>(
    els: &[&'a str],
    platforms: &[WidgetPlatform],
) -> (Vec<&'a str>, Vec<&'a str>) {
    let mut full = Vec::new();
    let mut watch = Vec::new();
    for el in els {
        let all_full = platforms
            .iter()
            .all(|p| support_for(el, *p).support == Support::Full);
        if all_full {
            full.push(*el);
        } else {
            watch.push(*el);
        }
    }
    (full, watch)
}

/// Docs path for an element or feature key (`image.url` → image page).
fn element_doc_href(key: &str) -> String {
    let base = key.split('.').next().unwrap_or(key);
    if key == "background.gradient" || base == "style" {
        return "/elements/style".into();
    }
    match key {
        "config.entries" => return "/features/timelines".into(),
        "config.strings" => return "/features/dynamic-content#localized-strings-strings".into(),
        "config.source" => return "/features/remote-data".into(),
        "config.state" => return "/features/interactive#local-state-state-set".into(),
        "activity.template" => return "/features/live-activities".into(),
        "config.instance" => return "/features/instances".into(),
        "activity.island" => return "/features/dynamic-island".into(),
        "activity" => return "/features/live-activities".into(),
        _ => {}
    }
    let page = match base {
        "vstack" | "hstack" | "zstack" | "grid" | "container" => "layout",
        "text" | "label" | "date" | "timer" => "text",
        "image" | "shape" | "canvas" => "media",
        "progress" | "gauge" | "chart" | "list" => "data",
        "button" | "toggle" | "link" => "interactive",
        "spacer" | "divider" => "spacing",
        _ => return "/elements/".into(),
    };
    format!("/elements/{page}#{base}")
}

fn cell_md(e: &CapabilityEntry, notes: &mut Vec<String>) -> String {
    if e.note.is_empty() {
        return e.support.as_str().to_string();
    }
    let idx = note_index(notes, e.note);
    format!("{}<sup>{}</sup>", e.support.as_str(), idx)
}

fn note_index(notes: &mut Vec<String>, note: &str) -> usize {
    if let Some(i) = notes.iter().position(|n| n == note) {
        return i + 1;
    }
    notes.push(note.to_string());
    notes.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(v: serde_json::Value) -> WidgetConfig {
        serde_json::from_value(v).unwrap()
    }

    fn found(ws: &[CapabilityWarning]) -> Vec<(String, String, &'static str)> {
        ws.iter()
            .map(|w| (w.path.clone(), w.element.clone(), w.support.as_str()))
            .collect()
    }

    /// The matrix: every element type has a cell per platform → the rendered Markdown names
    /// elements, features, sections, doc links and footnotes → element type names match it.
    #[test]
    fn matrix_has_every_cell_and_renders_to_markdown() {
        for el in ELEMENT_TYPES {
            for p in WidgetPlatform::all() {
                let e = support_for(el, p);
                assert_eq!(e.element, *el);
                assert_eq!(e.platform, p);
            }
        }
        let md = render_capability_matrix_md();
        assert!(md.contains("`vstack`"));
        assert!(md.contains("image.url"));
        assert!(md.contains("### Core\n"));
        assert!(!md.contains("## Core "));
        assert!(md.contains("/elements/layout#vstack"));
        assert!(md.contains("### Choosing a surface set"));
        assert!(md.contains("<sup>"));
        let chart = cfg(
            serde_json::json!({"small": {"type": "chart", "chartType": "bar",
            "chartData": [{"label": "a", "value": 1}]}}),
        );
        assert_eq!(chart.small.unwrap().type_name(), "chart");
    }

    /// `validate_config` per platform: Windows walks children, entries and config features
    /// (exact warning list) → iOS (full renderer, `image.url` is full after prefetch) has
    /// none → Android flags only the SF-symbol name → an empty config has none anywhere.
    #[test]
    fn validate_config_warnings_by_platform() {
        // 1. Windows: nested children, timeline entries and config-level features.
        let c = cfg(serde_json::json!({
            "small": {"type": "vstack", "children": [
                {"type": "zstack", "background": {"gradientType": "linear", "colors": ["#000", "#fff"]}},
                {"type": "timer", "targetDate": "2030-01-01T00:00:00Z"},
                {"type": "canvas", "width": 10, "height": 10, "elements": [{"draw": "path", "d": "M0 0"}]},
                {"type": "text", "content": "x", "privacySensitive": true, "accessibilityLabel": "X"},
                {"type": "text", "content": "y", "privacySensitive": false, "accessibilityLabel": ""}
            ]},
            "entries": [{"at": 1, "medium": {"type": "shape", "shapeType": "circle"}}],
            "source": {"url": "https://a"}
        }));
        let w = found(&validate_config(&c, WidgetPlatform::Windows));
        let want = [
            ("small/vstack[0]", "zstack", "degraded"),
            ("small/vstack[0]", "background.gradient", "degraded"),
            ("small/vstack[1]", "timer", "degraded"),
            ("small/vstack[1]", "timer.live", "degraded"),
            ("small/vstack[2]", "canvas", "degraded"),
            ("small/vstack[2]", "canvas.path", "degraded"),
            ("small/vstack[3]", "style.privacySensitive", "unsupported"),
            ("small/vstack[3]", "style.accessibilityLabel", "unsupported"),
            ("entries", "config.entries", "degraded"),
            ("entries[0].medium", "shape", "degraded"),
            ("source", "config.source", "unsupported"),
        ];
        for (p, e, s) in want {
            assert!(
                w.contains(&(p.into(), e.into(), s)),
                "missing {p} {e} {s} in {w:?}"
            );
        }
        assert_eq!(w.len(), want.len(), "{w:?}");

        // 2. iOS / Android on images: url is full on iOS (prefetched), systemName degrades
        //    on Android; empty url / systemName are not checked.
        let c = cfg(serde_json::json!({
            "small": {"type": "image", "url": "https://example.com/a.png", "size": 32},
            "medium": {"type": "image", "url": "https://x/a.png", "systemName": "star"},
            "large": {"type": "image", "url": "", "systemName": ""},
            "strings": {"en": {"a": "b"}}
        }));
        assert!(validate_config(&c, WidgetPlatform::Ios).is_empty());
        assert_eq!(
            found(&validate_config(&c, WidgetPlatform::Android)),
            vec![("medium".into(), "image.systemName".into(), "degraded")]
        );

        // 3. Nothing to warn about.
        assert!(validate_config(&WidgetConfig::default(), WidgetPlatform::Windows).is_empty());
    }
}

#[cfg(test)]
mod write_docs {
    #[test]
    fn capability_matrix_doc_matches() {
        let expected = super::render_capability_matrix_md();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("docs/generated/capability-matrix.md");
        if !path.exists() {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, &expected).unwrap();
            return;
        }
        let on_disk = std::fs::read_to_string(&path).unwrap();
        assert_eq!(
            on_disk, expected,
            "docs/generated/capability-matrix.md drifted — regenerate with:\n\
             cargo test --lib write_docs::capability_matrix_doc_matches -- --ignored\n\
             or delete the file and re-run this test"
        );
    }

    #[test]
    fn capabilities_json_matches() {
        let expected = super::render_capabilities_json();
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("schemas/capabilities.json");
        if !path.exists() {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, &expected).unwrap();
            return;
        }
        let on_disk = std::fs::read_to_string(&path).unwrap();
        assert_eq!(
            on_disk, expected,
            "schemas/capabilities.json drifted — delete it and re-run this test, or:\n\
             cargo test --lib capabilities::write_docs::capabilities_json_matches"
        );
    }
}
