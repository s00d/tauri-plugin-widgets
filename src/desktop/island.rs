//! Desktop "Dynamic Island" surface for live activities (`activitySurface: "island"`).
//!
//! One borderless window per app, centred at the top of the primary display: a black pill
//! that wraps the camera notch on notched Macs (leading region left of it, trailing right)
//! and floats just under the menu bar / at the top edge elsewhere. The newest activity is
//! the compact pill, older ones are minimal circles beside it (like iOS). Hover (or a
//! click) expands the pill to the expanded regions + bottom layout.
//!
//! The window is always exactly the visible shapes, so clicks next to the pill reach the
//! windows below. Geometry ([`layout`]) and region defaults ([`regions`]) are pure and
//! unit-tested; the window code is at the bottom.

use serde::Serialize;

use super::placement::Rect;
use crate::models::{
    ActivityContent, ColorValue, FontWeight, ImageElement, TextElement, TimerCounting,
    TimerElement, WidgetElement,
};

/// Window label. `activity_window_label` never yields `_` followed by `i`, so no
/// activity id maps onto it.
pub(crate) const LABEL: &str = "activity-_island";
/// Widget id the island page is opened with (the renderer switches on it).
pub(crate) const WIDGET_ID: &str = "activity.__island__";

/// Compact region width on each side of the notch / camera gap (logical px).
const SIDE: f64 = 72.0;
/// Pill height without a notch.
const PILL_H: f64 = 32.0;
/// Fake camera gap between the compact regions without a notch.
const CAMERA: f64 = 36.0;
/// Distance from the menu bar / work-area top when the pill floats.
const MARGIN: f64 = 6.0;
/// Space between the pill and a minimal circle.
const GAP: f64 = 6.0;
/// Minimal circles shown at most (older activities beyond that wait their turn).
const MAX_MINIMAL: usize = 3;
/// Expanded pill: minimum width and the height added below the top band.
const EXPANDED_W: f64 = 400.0;
const EXPANDED_BODY: f64 = 132.0;

/// The display the island sits on (logical px, top-left origin of the virtual desktop).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Screen {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    /// Space above the pill's top band: 0 at a notch, else menu bar / panel height.
    pub top_inset: f64,
    /// Camera housing `(width, height)` on notched Macs.
    pub notch: Option<(f64, f64)>,
}

impl Screen {
    /// No monitor information (headless session, mock runtime).
    pub const FALLBACK: Screen = Screen {
        x: 0.0,
        y: 0.0,
        w: 1440.0,
        top_inset: 25.0,
        notch: None,
    };
}

/// Where everything goes for one state.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Layout {
    /// Window frame (absolute logical px).
    pub window: Rect,
    /// Pill size (horizontally centred in the window, flush with its top).
    pub pill: (f64, f64),
    /// Minimal circles: centre x relative to the window centre (diameter = `band`).
    pub minimal: Vec<f64>,
}

/// Width of the gap between leading and trailing (notch or fake camera) and the top band height.
fn band(s: &Screen) -> (f64, f64) {
    s.notch.unwrap_or((CAMERA, PILL_H))
}

/// Compact and expanded pill sizes.
pub(crate) fn pill_sizes(s: &Screen) -> ((f64, f64), (f64, f64)) {
    let (gap, h) = band(s);
    let compact = (gap + 2.0 * SIDE, h);
    let expanded = (EXPANDED_W.max(compact.0 + 40.0), h + EXPANDED_BODY);
    (compact, expanded)
}

/// Window and shapes for `others` activities behind the front one. The window stays
/// centred on the display (so it can be resized without moving the pill); minimal
/// circles alternate right, left, right like iOS puts the detached one on the right.
/// Expanded hides the circles.
pub(crate) fn layout(s: &Screen, others: usize, expanded: bool) -> Layout {
    let (compact, big) = pill_sizes(s);
    let (_, h) = band(s);
    let pill = if expanded { big } else { compact };
    let n = if expanded { 0 } else { others.min(MAX_MINIMAL) };
    let step = GAP + h;
    let minimal: Vec<f64> = (0..n)
        .map(|i| {
            let k = (i / 2) as f64;
            let off = pill.0 / 2.0 + GAP + h / 2.0 + k * step;
            if i % 2 == 0 {
                off
            } else {
                -off
            }
        })
        .collect();
    let reach = n.div_ceil(2) as f64 * step;
    let w = pill.0 + 2.0 * reach;
    let top = if s.notch.is_some() {
        0.0
    } else {
        s.top_inset + MARGIN
    };
    Layout {
        window: Rect {
            x: s.x + (s.w - w) / 2.0,
            y: s.y + top,
            w,
            h: pill.1,
        },
        pill,
        minimal,
    }
}

/// The island's regions for one activity, with the iOS defaults (`TauriLiveActivity`):
/// compact leading / minimal = icon, compact trailing = countdown or progress %,
/// expanded leading / trailing / center only when given, bottom = `layout`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Regions {
    pub compact_leading: Option<WidgetElement>,
    pub compact_trailing: Option<WidgetElement>,
    pub minimal: Option<WidgetElement>,
    pub leading: Option<WidgetElement>,
    pub trailing: Option<WidgetElement>,
    pub center: Option<WidgetElement>,
    pub bottom: WidgetElement,
}

pub(crate) fn regions(c: &ActivityContent) -> Regions {
    let tint = c.tint.clone().map(ColorValue::Solid);
    let icon = WidgetElement::Image(ImageElement {
        system_name: Some(c.icon.clone().unwrap_or_else(|| "circle.fill".into())),
        size: Some(15.0),
        color: tint.clone(),
        ..Default::default()
    });
    let status = match (c.ends_at, c.progress) {
        (Some(end), _) => Some(WidgetElement::Timer(TimerElement {
            target_date: crate::models::iso8601_utc(end),
            counting: Some(TimerCounting::Down),
            // `4:52`, not `00:04:52`: the compact side is narrow.
            show_hours: Some(false),
            font_size: Some(13.0),
            font_weight: Some(FontWeight::Semibold),
            color: tint.clone(),
            ..Default::default()
        })),
        (None, Some(p)) => Some(WidgetElement::Text(TextElement {
            content: format!("{}%", (p * 100.0).round() as i64),
            font_size: Some(13.0),
            font_weight: Some(FontWeight::Semibold),
            color: tint,
            monospaced_digit: Some(true),
            line_limit: Some(1),
            ..Default::default()
        })),
        _ => None,
    };
    Regions {
        compact_leading: c.compact_leading.clone().or_else(|| Some(icon.clone())),
        compact_trailing: c.compact_trailing.clone().or(status),
        minimal: c.minimal.clone().or(Some(icon)),
        leading: c.island_leading.clone(),
        trailing: c.island_trailing.clone(),
        center: c.island_center.clone(),
        bottom: c.effective_layout(),
    }
}

/// Running activities on the island, oldest first (the last one is the pill).
#[derive(Default)]
pub(crate) struct Island {
    entries: Vec<(String, ActivityContent)>,
    pub expanded: bool,
    /// The hover poller thread runs ([`Island::hover_step`]).
    pub polling: bool,
}

/// What the hover poller does after one cursor sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Hover {
    Nothing,
    Expand,
    /// Cursor just left: the page animates the pill closed, then shrinks the window.
    AskCollapse,
    /// Still expanded long after leaving (page did not answer): shrink now.
    Collapse,
}

/// Poller decision from `inside` (cursor over the window), the island state and how many
/// samples in a row the cursor has been outside (`outside`, updated here).
pub(crate) fn hover_step(inside: bool, expanded: bool, outside: &mut u32) -> Hover {
    if inside {
        *outside = 0;
        return if expanded {
            Hover::Nothing
        } else {
            Hover::Expand
        };
    }
    *outside = outside.saturating_add(1);
    match (expanded, *outside) {
        (true, 2) => Hover::AskCollapse,
        (true, n) if n >= 15 => Hover::Collapse,
        _ => Hover::Nothing,
    }
}

impl Island {
    pub fn contains(&self, id: &str) -> bool {
        self.entries.iter().any(|(_, c)| c.id == id)
    }

    /// New activities go to the front (the pill); updates keep their place.
    pub fn upsert(&mut self, content: ActivityContent, group: &str) {
        match self.entries.iter_mut().find(|(_, c)| c.id == content.id) {
            Some(e) => *e = (group.to_string(), content),
            None => self.entries.push((group.to_string(), content)),
        }
    }

    pub fn remove(&mut self, id: &str) -> bool {
        let before = self.entries.len();
        self.entries.retain(|(_, c)| c.id != id);
        if self.entries.is_empty() {
            self.expanded = false;
        }
        self.entries.len() != before
    }

    /// Bring a minimal activity to the pill.
    pub fn focus(&mut self, id: &str) -> bool {
        let Some(i) = self.entries.iter().position(|(_, c)| c.id == id) else {
            return false;
        };
        let e = self.entries.remove(i);
        self.entries.push(e);
        true
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Group of the front activity (the window's own `group` query value).
    pub fn front_group(&self) -> Option<String> {
        self.entries.last().map(|(g, _)| g.clone())
    }

    pub fn layout(&self, s: &Screen) -> Layout {
        layout(s, self.entries.len().saturating_sub(1), self.expanded)
    }

    /// What the island page draws: front activity first, then the minimal ones.
    pub fn view(&self, s: &Screen) -> View {
        let (compact, expanded_size) = pill_sizes(s);
        let (gap, band) = band(s);
        let l = self.layout(s);
        View {
            expanded: self.expanded,
            flush: s.notch.is_some(),
            gap,
            band,
            compact,
            expanded_size,
            minimal: l.minimal,
            activities: self
                .entries
                .iter()
                .rev()
                .take(1 + MAX_MINIMAL)
                .map(|(group, c)| ViewActivity {
                    id: c.id.clone(),
                    group: group.clone(),
                    widget_id: crate::models::activity_widget_id(&c.id),
                    url: c.url.clone(),
                    keyline: c.keyline_tint.clone(),
                    regions: regions(c),
                })
                .collect(),
        }
    }
}

/// `activity-island` event payload / `activity_island` command result.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct View {
    pub expanded: bool,
    /// Pill hangs from the top edge (notch): square top corners.
    pub flush: bool,
    pub gap: f64,
    pub band: f64,
    pub compact: (f64, f64),
    pub expanded_size: (f64, f64),
    pub minimal: Vec<f64>,
    pub activities: Vec<ViewActivity>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ViewActivity {
    pub id: String,
    pub group: String,
    pub widget_id: String,
    pub url: Option<String>,
    pub keyline: Option<String>,
    pub regions: Regions,
}

// ---- window ---------------------------------------------------------------------------

use tauri::{AppHandle, Runtime, WebviewWindow};

/// The display under the island: the notch from AppKit on Apple silicon Macs, else the
/// primary monitor's work area (below the macOS menu bar / Linux top panel). `None` when
/// no monitor can be queried. Call on the main thread (AppKit).
pub(crate) fn screen<R: Runtime>(app: &AppHandle<R>) -> Option<Screen> {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    if let Some(s) = macos_notch_screen() {
        return Some(s);
    }
    let m = primary_monitor(app)?;
    let (pos, size, wa) = (m.position(), m.size(), m.work_area());
    Some(work_area_screen(
        (pos.x, pos.y),
        size.width,
        wa.position.y,
        m.scale_factor(),
    ))
}

/// Logical [`Screen`] of a monitor at physical `pos`, `width` wide, whose work area
/// starts at physical `work_top` (below the menu bar / top panel).
fn work_area_screen(pos: (i32, i32), width: u32, work_top: i32, scale: f64) -> Screen {
    Screen {
        x: pos.0 as f64 / scale,
        y: pos.1 as f64 / scale,
        w: width as f64 / scale,
        top_inset: ((work_top - pos.1) as f64 / scale).max(0.0),
        notch: None,
    }
}

/// `AppHandle::primary_monitor`, `None` when it fails or cannot be queried.
pub(crate) fn primary_monitor<R: Runtime>(app: &AppHandle<R>) -> Option<tauri::Monitor> {
    super::window::monitor_query(|| app.primary_monitor()).flatten()
}

/// Move / resize the island window to `r` (logical).
pub(crate) fn place<R: Runtime>(win: &WebviewWindow<R>, r: Rect) {
    let _ = win.set_size(tauri::LogicalSize::new(r.w, r.h));
    let _ = win.set_position(tauri::LogicalPosition::new(r.x, r.y));
    // A gtk-layer-shell surface (Wayland) is a different toplevel than the tauri window:
    // it is anchored top-centre, so only its size changes.
    #[cfg(all(target_os = "linux", feature = "layer-shell"))]
    {
        use gtk::prelude::*;
        if let Some(top) = crate::linux::native("default_vbox", || win.default_vbox())
            .ok()
            .and_then(|v| v.toplevel())
            .and_then(|t| t.downcast::<gtk::Window>().ok())
        {
            top.resize(r.w.round() as i32, r.h.round() as i32);
        }
    }
}

/// `NSWindowCollectionBehavior` of the island: canJoinAllSpaces (1), stationary
/// (1 << 4), ignoresCycle (1 << 6), fullScreenAuxiliary (1 << 8).
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
const FLOAT_BEHAVIOUR: usize = 0x151;
/// Window level: `NSMainMenuWindowLevel` (24) + 3, as boring.notch.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
const FLOAT_LEVEL: isize = 27;

/// Collection behaviour with the island bits added to the window's `current` ones.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn float_behaviour(current: usize) -> usize {
    current | FLOAT_BEHAVIOUR
}

/// macOS: above the menu bar, on every Space, over full-screen apps, out of Mission
/// Control and the Cmd-` cycle, no shadow.
#[cfg(target_os = "macos")]
pub(crate) fn macos_float<R: Runtime>(win: &WebviewWindow<R>) {
    use std::ffi::{c_char, c_void};
    #[link(name = "objc")]
    extern "C" {
        fn sel_registerName(name: *const c_char) -> *const c_void;
        fn objc_msgSend();
    }
    let Ok(ns) = win.ns_window() else { return };
    // SAFETY: `ns` is the live NSWindow of this webview window, we are on the main thread
    // (window setup runs there) and each objc_msgSend is cast to the selector's signature.
    unsafe {
        let sel = |s: &[u8]| sel_registerName(s.as_ptr().cast());
        let msg = objc_msgSend as unsafe extern "C" fn();
        let get: unsafe extern "C" fn(*mut c_void, *const c_void) -> usize =
            std::mem::transmute(msg);
        let set_u: unsafe extern "C" fn(*mut c_void, *const c_void, usize) =
            std::mem::transmute(msg);
        let set_i: unsafe extern "C" fn(*mut c_void, *const c_void, isize) =
            std::mem::transmute(msg);
        let set_b: unsafe extern "C" fn(*mut c_void, *const c_void, bool) =
            std::mem::transmute(msg);
        let behaviour = get(ns, sel(b"collectionBehavior\0"));
        set_u(
            ns,
            sel(b"setCollectionBehavior:\0"),
            float_behaviour(behaviour),
        );
        set_i(ns, sel(b"setLevel:\0"), FLOAT_LEVEL);
        set_b(ns, sel(b"setHasShadow:\0"), false);
    }
}

/// Notched main display from AppKit: `safeAreaInsets.top` is the notch height, the
/// screen width minus `auxiliaryTopLeftArea` / `auxiliaryTopRightArea` its width.
/// `None` without a notch (the work-area path handles that). Apple silicon only: there
/// NSRect / NSEdgeInsets come back in registers from plain `objc_msgSend` (Intel Macs
/// need `_stret` and never have a notch).
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn macos_notch_screen() -> Option<Screen> {
    use std::ffi::{c_char, c_void};
    #[link(name = "objc")]
    extern "C" {
        fn objc_getClass(name: *const c_char) -> *mut c_void;
        fn sel_registerName(name: *const c_char) -> *const c_void;
        fn objc_msgSend();
    }
    /// NSRect {x, y, w, h} / NSEdgeInsets {top, left, bottom, right}.
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Quad([f64; 4]);
    // SAFETY: AppKit class / selectors that exist since macOS 12 (checked with
    // respondsToSelector:); called on the main thread; each objc_msgSend is cast to the
    // selector's signature (HFA structs return in d0-d3 on arm64).
    unsafe {
        let sel = |s: &[u8]| sel_registerName(s.as_ptr().cast());
        let msg = objc_msgSend as unsafe extern "C" fn();
        let obj: unsafe extern "C" fn(*mut c_void, *const c_void) -> *mut c_void =
            std::mem::transmute(msg);
        let responds: unsafe extern "C" fn(*mut c_void, *const c_void, *const c_void) -> bool =
            std::mem::transmute(msg);
        let quad: unsafe extern "C" fn(*mut c_void, *const c_void) -> Quad =
            std::mem::transmute(msg);
        let cls = objc_getClass(c"NSScreen".as_ptr());
        if cls.is_null() {
            return None;
        }
        let screens = obj(cls, sel(b"screens\0"));
        if screens.is_null() {
            return None;
        }
        let main = obj(screens, sel(b"firstObject\0"));
        if main.is_null()
            || !responds(
                main,
                sel(b"respondsToSelector:\0"),
                sel(b"auxiliaryTopLeftArea\0"),
            )
        {
            return None;
        }
        let top = quad(main, sel(b"safeAreaInsets\0")).0[0];
        let width = quad(main, sel(b"frame\0")).0[2];
        let left = quad(main, sel(b"auxiliaryTopLeftArea\0")).0[2];
        let right = quad(main, sel(b"auxiliaryTopRightArea\0")).0[2];
        notch_screen(width, top, left, right)
    }
}

/// Screen `width` wide whose notch is `top` tall between the `left` / `right` menu-bar
/// areas (widths); `None` when any is missing (no notch).
#[cfg_attr(
    not(all(target_os = "macos", target_arch = "aarch64")),
    allow(dead_code)
)]
fn notch_screen(width: f64, top: f64, left: f64, right: f64) -> Option<Screen> {
    if top <= 0.0 || left <= 0.0 || right <= 0.0 {
        return None;
    }
    Some(Screen {
        x: 0.0,
        y: 0.0,
        w: width,
        top_inset: 0.0,
        notch: Some((width - left - right, top)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn activity(id: &str) -> ActivityContent {
        ActivityContent {
            id: id.into(),
            title: id.into(),
            ..Default::default()
        }
    }

    /// Where the [`Screen`] comes from: AppKit's notch metrics on a MacBook (none when an
    /// area is missing), else a monitor's work area in logical px; the macOS window joins
    /// every Space above the menu bar keeping its own behaviour bits.
    #[test]
    fn screen_sources_and_macos_float() {
        assert_eq!(notch_screen(1512.0, 0.0, 663.5, 663.5), None);
        assert_eq!(notch_screen(1512.0, 32.0, 0.0, 663.5), None);
        assert_eq!(notch_screen(1512.0, 32.0, 663.5, 0.0), None);
        assert_eq!(
            notch_screen(1512.0, 32.0, 663.5, 663.5),
            Some(Screen {
                x: 0.0,
                y: 0.0,
                w: 1512.0,
                top_inset: 0.0,
                notch: Some((185.0, 32.0)),
            })
        );
        // 2x monitor at physical (3840, -200), 1440 logical wide, 25 pt menu bar.
        assert_eq!(
            work_area_screen((3840, -200), 2880, -150, 2.0),
            Screen {
                x: 1920.0,
                y: -100.0,
                w: 1440.0,
                top_inset: 25.0,
                notch: None,
            }
        );
        // Work area above the frame (bogus): no negative inset.
        assert_eq!(work_area_screen((0, 0), 100, -10, 1.0).top_inset, 0.0);
        assert_eq!(float_behaviour(0), 0x151);
        assert_eq!(float_behaviour(0x2 | 0x10), 0x153);
    }

    /// Region defaults follow iOS: icon on the leading side / minimal, progress % then a
    /// countdown on the trailing side, given regions win, the bottom is the layout.
    #[test]
    fn activity_regions_as_the_activity_changes() {
        let red = || Some(ColorValue::Solid("#ff0000".into()));
        let json = |e: &WidgetElement| serde_json::to_value(e).unwrap();
        let percent = |content: &str, color| {
            json(&WidgetElement::Text(TextElement {
                content: content.into(),
                font_size: Some(13.0),
                font_weight: Some(FontWeight::Semibold),
                color,
                monospaced_digit: Some(true),
                line_limit: Some(1),
                ..Default::default()
            }))
        };

        // Started with progress and an icon.
        let c = ActivityContent {
            id: "a".into(),
            title: "Build".into(),
            progress: Some(0.42),
            icon: Some("hammer".into()),
            ..Default::default()
        };
        let r = regions(&c);
        let WidgetElement::Image(i) = r.compact_leading.as_ref().unwrap() else {
            panic!("icon")
        };
        assert_eq!(i.system_name.as_deref(), Some("hammer"));
        assert_eq!(
            json(r.compact_trailing.as_ref().unwrap()),
            percent("42%", None)
        );
        assert!(r.leading.is_none() && r.trailing.is_none() && r.center.is_none());
        assert_eq!(json(&r.bottom), json(&c.effective_layout()));

        // Tinted, no icon: the default icon and the percentage take the tint.
        let tinted = ActivityContent {
            icon: None,
            progress: Some(0.5),
            tint: Some("#ff0000".into()),
            ..c.clone()
        };
        let r = regions(&tinted);
        let icon = json(&WidgetElement::Image(ImageElement {
            system_name: Some("circle.fill".into()),
            size: Some(15.0),
            color: red(),
            ..Default::default()
        }));
        assert_eq!(json(r.compact_leading.as_ref().unwrap()), icon);
        assert_eq!(json(r.minimal.as_ref().unwrap()), icon);
        assert_eq!(
            json(r.compact_trailing.as_ref().unwrap()),
            percent("50%", red())
        );

        // An end time arrives: the countdown wins over progress.
        let timed = ActivityContent {
            ends_at: Some(0),
            ..tinted.clone()
        };
        let timer = json(&WidgetElement::Timer(TimerElement {
            target_date: crate::models::iso8601_utc(0),
            counting: Some(TimerCounting::Down),
            show_hours: Some(false),
            font_size: Some(13.0),
            font_weight: Some(FontWeight::Semibold),
            color: red(),
            ..Default::default()
        }));
        assert_eq!(
            json(regions(&timed).compact_trailing.as_ref().unwrap()),
            timer
        );

        // Explicit regions replace every default.
        let given = WidgetElement::Text(TextElement {
            content: "x".into(),
            ..Default::default()
        });
        let custom = ActivityContent {
            compact_leading: Some(given.clone()),
            compact_trailing: Some(given.clone()),
            minimal: Some(given.clone()),
            island_center: Some(given.clone()),
            ..timed
        };
        let r = regions(&custom);
        for e in [
            &r.compact_leading,
            &r.compact_trailing,
            &r.minimal,
            &r.center,
        ] {
            assert_eq!(json(e.as_ref().unwrap()), json(&given));
        }
    }

    // ---- state machine: the island as the poller and commands drive it ---------------

    use proptest::prelude::*;
    use proptest_state_machine::{prop_state_machine, ReferenceStateMachine, StateMachineTest};

    /// Samples in a row outside before a forced collapse (`hover_step`).
    const FORCE_COLLAPSE: u32 = 15;

    #[derive(Debug, Clone)]
    enum Step {
        /// Start (new id) or update (running id) an activity.
        Upsert {
            id: u8,
            group: u8,
        },
        End(u8),
        Focus(u8),
        /// One cursor sample over the window.
        Enter,
        /// `samples` cursor samples outside; `answers`: the page acknowledges the
        /// collapse request (it animates closed, then calls `activity_island(false)`).
        Leave {
            samples: u32,
            answers: bool,
        },
        Screen(Screen),
    }

    /// Reference model: running `(id, group)` oldest first (last = pill), expanded flag,
    /// outside-sample counter, display.
    #[derive(Debug, Clone)]
    struct Model {
        running: Vec<(u8, u8)>,
        expanded: bool,
        outside: u32,
        screen: Screen,
    }

    fn any_screen() -> impl Strategy<Value = Screen> {
        (
            -3000i32..3000,
            -100i32..100,
            800u32..4000,
            0u32..60,
            prop::option::of((100u32..320, 24u32..44)),
        )
            .prop_map(|(x, y, w, inset, notch)| Screen {
                x: x as f64,
                y: y as f64,
                w: w as f64,
                top_inset: if notch.is_some() { 0.0 } else { inset as f64 },
                notch: notch.map(|(g, h)| (g as f64, h as f64)),
            })
    }

    impl ReferenceStateMachine for Model {
        type State = Model;
        type Transition = Step;

        fn init_state() -> BoxedStrategy<Model> {
            any_screen()
                .prop_map(|screen| Model {
                    running: vec![],
                    expanded: false,
                    outside: 0,
                    screen,
                })
                .boxed()
        }

        fn transitions(_: &Model) -> BoxedStrategy<Step> {
            prop_oneof![
                3 => (0u8..6, 0u8..3).prop_map(|(id, group)| Step::Upsert { id, group }),
                1 => (0u8..6).prop_map(Step::End),
                1 => (0u8..6).prop_map(Step::Focus),
                2 => Just(Step::Enter),
                2 => (1u32..20, any::<bool>())
                    .prop_map(|(samples, answers)| Step::Leave { samples, answers }),
                1 => any_screen().prop_map(Step::Screen),
            ]
            .boxed()
        }

        /// The hover poller only runs while the island exists.
        fn preconditions(m: &Model, s: &Step) -> bool {
            !matches!(s, Step::Enter | Step::Leave { .. }) || !m.running.is_empty()
        }

        fn apply(mut m: Model, s: &Step) -> Model {
            match *s {
                Step::Upsert { id, group } => match m.running.iter_mut().find(|e| e.0 == id) {
                    Some(e) => e.1 = group,
                    None => m.running.push((id, group)),
                },
                Step::End(id) => {
                    m.running.retain(|e| e.0 != id);
                    if m.running.is_empty() {
                        // Island closed: no expansion; a new poller starts from zero.
                        m.expanded = false;
                        m.outside = 0;
                    }
                }
                Step::Focus(id) => {
                    if let Some(i) = m.running.iter().position(|e| e.0 == id) {
                        let e = m.running.remove(i);
                        m.running.push(e);
                    }
                }
                Step::Enter => {
                    m.outside = 0;
                    m.expanded = true;
                }
                Step::Leave { samples, answers } => {
                    for _ in 0..samples {
                        m.outside += 1;
                        // Second sample outside: grace over, the page is asked to close.
                        if m.expanded && m.outside == 2 && answers {
                            m.expanded = false;
                        }
                        if m.expanded && m.outside >= FORCE_COLLAPSE {
                            m.expanded = false;
                        }
                    }
                }
                Step::Screen(s) => m.screen = s,
            }
            m
        }
    }

    struct Sut {
        island: Island,
        outside: u32,
        screen: Screen,
    }

    fn content(id: u8) -> ActivityContent {
        activity(&format!("a{id}"))
    }

    struct IslandMachine;

    impl StateMachineTest for IslandMachine {
        type SystemUnderTest = Sut;
        type Reference = Model;

        fn init_test(m: &Model) -> Sut {
            Sut {
                island: Island::default(),
                outside: 0,
                screen: m.screen,
            }
        }

        fn apply(mut sut: Sut, m: &Model, s: Step) -> Sut {
            match s {
                Step::Upsert { id, group } => sut.island.upsert(content(id), &format!("g{group}")),
                Step::End(id) => {
                    let was = sut.island.contains(&format!("a{id}"));
                    assert_eq!(sut.island.remove(&format!("a{id}")), was);
                    if sut.island.is_empty() {
                        sut.outside = 0;
                    }
                }
                Step::Focus(id) => {
                    let was = sut.island.contains(&format!("a{id}"));
                    assert_eq!(sut.island.focus(&format!("a{id}")), was);
                }
                Step::Enter => {
                    let h = hover_step(true, sut.island.expanded, &mut sut.outside);
                    assert_ne!(h, Hover::AskCollapse);
                    if h == Hover::Expand {
                        sut.island.expanded = true;
                    }
                }
                Step::Leave { samples, answers } => {
                    for _ in 0..samples {
                        match hover_step(false, sut.island.expanded, &mut sut.outside) {
                            Hover::AskCollapse if answers => sut.island.expanded = false,
                            Hover::Collapse => sut.island.expanded = false,
                            Hover::Expand => panic!("expanded from outside"),
                            _ => {}
                        }
                    }
                }
                Step::Screen(s) => sut.screen = s,
            }
            assert_eq!(sut.outside, m.outside);
            sut
        }

        fn check_invariants(sut: &Sut, m: &Model) {
            let (s, isl) = (&sut.screen, &sut.island);
            // Order: the pill is the newest / focused activity, then the minimal ones.
            for id in 0..6u8 {
                let running = m.running.iter().any(|e| e.0 == id);
                assert_eq!(isl.contains(&format!("a{id}")), running);
            }
            assert_eq!(isl.is_empty(), m.running.is_empty());
            assert_eq!(
                isl.front_group(),
                m.running.last().map(|e| format!("g{}", e.1))
            );
            let view = isl.view(s);
            let want: Vec<String> = m
                .running
                .iter()
                .rev()
                .take(1 + MAX_MINIMAL)
                .map(|e| format!("a{}", e.0))
                .collect();
            let got: Vec<String> = view.activities.iter().map(|a| a.id.clone()).collect();
            assert_eq!(got, want);

            // Expanded only while hovered, or within the grace period after leaving.
            assert_eq!(isl.expanded, m.expanded);
            assert!(!isl.expanded || (!m.running.is_empty() && m.outside < FORCE_COLLAPSE));
            assert_eq!(view.expanded, isl.expanded);

            // Pill: notch (or fake camera) between two SIDE regions; expanded grows down.
            let (gap, band) = s.notch.unwrap_or((CAMERA, PILL_H));
            assert_eq!(
                (view.gap, view.band, view.flush),
                (gap, band, s.notch.is_some())
            );
            let compact = (gap + 2.0 * SIDE, band);
            let big = (EXPANDED_W.max(compact.0 + 40.0), band + EXPANDED_BODY);
            assert_eq!((view.compact, view.expanded_size), (compact, big));
            let l = isl.layout(s);
            assert_eq!(l.pill, if isl.expanded { big } else { compact });
            assert_eq!(view.minimal, l.minimal);

            // At most MAX_MINIMAL circles (none expanded): right, left, right.
            let n = if isl.expanded {
                0
            } else {
                m.running.len().saturating_sub(1).min(MAX_MINIMAL)
            };
            assert_eq!(l.minimal.len(), n);
            let first = l.pill.0 / 2.0 + GAP + band / 2.0;
            let want = [first, -first, first + GAP + band];
            assert_eq!(l.minimal[..], want[..n]);

            // Window: centred on the screen, top band at the notch / under the menu bar,
            // exactly as tall as the pill and as wide as the farthest shape on either side.
            let top = if s.notch.is_some() {
                s.y
            } else {
                s.y + s.top_inset + MARGIN
            };
            assert_eq!(l.window.y, top);
            assert_eq!(l.window.h, l.pill.1);
            let centre = l.window.x + l.window.w / 2.0;
            assert!((centre - (s.x + s.w / 2.0)).abs() < 1e-9, "{l:?} on {s:?}");
            let reach = l
                .minimal
                .iter()
                .map(|dx| dx.abs() + band / 2.0)
                .fold(l.pill.0 / 2.0, f64::max);
            assert!((l.window.w / 2.0 - reach).abs() < 1e-9, "{l:?}");
        }
    }

    prop_state_machine! {
        #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]
        #[test]
        fn island_state_machine(sequential 1..40 => IslandMachine);
    }
}
