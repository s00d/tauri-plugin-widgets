//! Widget window modes, placement and remembered positions (tauri side of
//! [`super::placement`]).

use tauri::{AppHandle, Manager, PhysicalPosition, Runtime, WebviewWindow, WindowEvent};

use super::placement::{
    effect_for, pick_monitor, place, still_visible, Rect, SavedRect, WindowModes,
};
use super::Widget;
use crate::models::{WidgetWindowConfig, WindowEffectHint, WindowLayer};

/// Where a new window opens: logical position for the builder, plus the exact physical
/// spot (set after build, so mixed-DPI setups land on the right monitor).
#[derive(Debug, PartialEq)]
pub(crate) struct Target {
    pub logical: (f64, f64),
    pub physical: Option<PhysicalPosition<i32>>,
}

/// What [`target_for`] needs from one monitor (physical px).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Monitor {
    pub name: Option<String>,
    pub frame: Rect,
    pub work_area: Rect,
    pub scale: f64,
}

fn rect(x: i32, y: i32, w: u32, h: u32) -> Rect {
    Rect {
        x: x as f64,
        y: y as f64,
        w: w as f64,
        h: h as f64,
    }
}

impl From<&tauri::Monitor> for Monitor {
    fn from(m: &tauri::Monitor) -> Self {
        let (p, s, wa) = (m.position(), m.size(), m.work_area());
        Monitor {
            name: m.name().cloned(),
            frame: rect(p.x, p.y, s.width, s.height),
            work_area: rect(wa.position.x, wa.position.y, wa.size.width, wa.size.height),
            scale: m.scale_factor(),
        }
    }
}

/// A monitor query, `None` when it fails or panics.
pub(crate) fn monitor_query<T>(f: impl FnOnce() -> tauri::Result<T>) -> Option<T> {
    // ponytail: MockRuntime's monitor queries are `unimplemented!()`; catching the panic
    // keeps window / activity commands testable there. Drop when tauri's mock returns Ok.
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f))
        .ok()?
        .ok()
}

/// Remembered position (when still on a connected monitor), else `placement`, else
/// `x` / `y`.
pub(crate) fn resolve_target<R: Runtime>(
    app: &AppHandle<R>,
    cfg: &WidgetWindowConfig,
) -> Option<Target> {
    let monitors: Vec<Monitor> = monitor_query(|| app.available_monitors())
        .unwrap_or_default()
        .iter()
        .map(Monitor::from)
        .collect();
    let saved = app
        .try_state::<Widget<R>>()
        .and_then(|w| w.window_store.position(&cfg.label));
    let primary = monitor_query(|| app.primary_monitor())
        .flatten()
        .map(|m| Monitor::from(&m));
    target_for(cfg, saved, &monitors, primary.as_ref())
}

/// [`resolve_target`] without tauri: `saved` is the remembered frame, `primary` the
/// primary monitor (matched by name and position).
pub(crate) fn target_for(
    cfg: &WidgetWindowConfig,
    saved: Option<SavedRect>,
    monitors: &[Monitor],
    primary: Option<&Monitor>,
) -> Option<Target> {
    let saved = saved.filter(|_| cfg.remember_position != Some(false));
    if let Some(s) = saved {
        let rects: Vec<Rect> = monitors.iter().map(|m| m.frame).collect();
        if still_visible(s.rect(), &rects) {
            let scale = monitors
                .iter()
                .find(|m| still_visible(s.rect(), std::slice::from_ref(&m.frame)))
                .map_or(1.0, |m| m.scale);
            return Some(Target {
                logical: (s.x as f64 / scale, s.y as f64 / scale),
                physical: Some(PhysicalPosition::new(s.x, s.y)),
            });
        }
    }
    if let (Some(p), false) = (&cfg.placement, monitors.is_empty()) {
        let names: Vec<Option<String>> = monitors.iter().map(|m| m.name.clone()).collect();
        let primary = primary.and_then(|pm| {
            monitors.iter().position(|m| {
                m.name == pm.name && (m.frame.x, m.frame.y) == (pm.frame.x, pm.frame.y)
            })
        });
        let m = &monitors[pick_monitor(p.monitor.as_ref(), &names, primary)];
        let scale = m.scale;
        let (x, y) = place(
            p,
            m.work_area,
            (cfg.width * scale, cfg.height * scale),
            scale,
        );
        return Some(Target {
            logical: (x / scale, y / scale),
            physical: Some(PhysicalPosition::new(x.round() as i32, y.round() as i32)),
        });
    }
    match (cfg.x, cfg.y) {
        (Some(x), Some(y)) => Some(Target {
            logical: (x, y),
            physical: None,
        }),
        _ => None,
    }
}

/// Which setters [`apply_modes`] calls and which native macOS level it picks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ModeSetters {
    /// `always_on_top` / `always_on_bottom` (off where macOS sets the level natively:
    /// tao's async level change would race it).
    pub layer: bool,
    pub focusable: bool,
    pub click_through: bool,
    /// The island window (floats above the menu bar on macOS).
    pub island: bool,
    /// macOS desktop level.
    pub desktop: bool,
}

/// New windows get their focus / click-through from the builder, so only non-defaults
/// are set; a reused window gets every mode reset.
pub(crate) fn mode_setters(label: &str, m: &WindowModes, reused: bool, macos: bool) -> ModeSetters {
    let island = label == super::island::LABEL;
    let desktop = m.layer == WindowLayer::Desktop;
    ModeSetters {
        layer: !(macos && (desktop || island)),
        focusable: reused || !m.focusable,
        click_through: reused || m.click_through,
        island,
        desktop,
    }
}

/// `set_effects` argument for a hint on `os`.
pub(crate) fn effects_config(
    hint: Option<WindowEffectHint>,
    os: &str,
) -> Option<tauri::utils::config::WindowEffectsConfig> {
    let e = effect_for(hint?, os)?;
    Some(tauri::utils::config::WindowEffectsConfig {
        effects: vec![e],
        ..Default::default()
    })
}

/// Apply the layer / input / workspace / material modes (new or reused window).
pub(crate) fn apply_modes<R: Runtime>(
    win: &WebviewWindow<R>,
    m: &WindowModes,
    effects: Option<WindowEffectHint>,
    reused: bool,
) {
    let log = |what: &str, r: tauri::Result<()>| {
        if let Err(e) = r {
            log::debug!("widgets: {what} on '{}': {e}", win.label());
        }
    };
    let set = mode_setters(win.label(), m, reused, cfg!(target_os = "macos"));
    if set.layer {
        log("always_on_top", win.set_always_on_top(m.on_top()));
        log("always_on_bottom", win.set_always_on_bottom(m.on_bottom()));
    }
    log(
        "visible_on_all_workspaces",
        win.set_visible_on_all_workspaces(m.all_workspaces),
    );
    if set.focusable {
        log("focusable", win.set_focusable(m.focusable));
    }
    if set.click_through {
        log(
            "ignore_cursor_events",
            win.set_ignore_cursor_events(m.click_through),
        );
    }
    if let Some(s) = m.shadow {
        log("shadow", win.set_shadow(s));
    }
    if let Some(c) = effects_config(effects, std::env::consts::OS) {
        log("effects", win.set_effects(c));
    }
    // ponytail: MockRuntime hands out a dangling NSView, so `ns_window()` would crash in
    // tauri itself; the type name is the only tell. Drop when the mock returns an error.
    #[cfg(target_os = "macos")]
    if !std::any::type_name::<R>().ends_with("MockRuntime") {
        if set.island {
            super::island::macos_float(win);
        } else {
            macos_desktop_level(win, set.desktop);
        }
    }
}

/// Remember where the user drags the window (flushed to disk 500 ms after the last move).
pub(crate) fn track_position<R: Runtime>(win: &WebviewWindow<R>) {
    let app = win.app_handle().clone();
    let label = win.label().to_string();
    win.on_window_event(move |e| {
        let WindowEvent::Moved(p) = e else { return };
        let (Some(w), Some(win)) = (app.try_state::<Widget<R>>(), app.get_webview_window(&label))
        else {
            return;
        };
        let Ok(size) = win.outer_size() else { return };
        let r = SavedRect {
            x: p.x,
            y: p.y,
            w: size.width,
            h: size.height,
        };
        if w.window_store.set_position(&label, r) {
            let app = app.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(500));
                if let Some(w) = app.try_state::<Widget<R>>() {
                    w.window_store.save();
                }
            });
        }
    });
}

/// `NSWindowCollectionBehavior` bits the desktop layer toggles: stationary in Mission
/// Control, out of the Cmd-` cycle.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
const DESKTOP_BEHAVIOUR: usize = 0x50; // 1 << 4 | 1 << 6 (disjoint bits: `|` == `^`)

/// New collection behaviour for the desktop layer on / off; `None` when unchanged.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn desktop_behaviour(current: usize, desktop: bool) -> Option<usize> {
    let next = if desktop {
        current | DESKTOP_BEHAVIOUR
    } else {
        current & !DESKTOP_BEHAVIOUR
    };
    (next != current).then_some(next)
}

/// Desktop layer on macOS: just above the Finder desktop icons (still clickable), below
/// every app window, stationary in Mission Control and out of the Cmd-` cycle.
#[cfg(target_os = "macos")]
fn macos_desktop_level<R: Runtime>(win: &WebviewWindow<R>, desktop: bool) {
    use std::ffi::{c_char, c_void};
    #[link(name = "objc")]
    extern "C" {
        fn sel_registerName(name: *const c_char) -> *const c_void;
        fn objc_msgSend();
    }
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGWindowLevelForKey(key: i32) -> i32;
    }
    const DESKTOP_ICON_LEVEL_KEY: i32 = 18; // kCGDesktopIconWindowLevelKey
    let Ok(ns) = win.ns_window() else { return };
    // SAFETY: `ns` is the live NSWindow of this webview window and we are on the main
    // thread (window creation runs there); each objc_msgSend is cast to the exact
    // signature of the selector it sends.
    unsafe {
        let sel = |s: &[u8]| sel_registerName(s.as_ptr().cast());
        let msg = objc_msgSend as unsafe extern "C" fn();
        let get: unsafe extern "C" fn(*mut c_void, *const c_void) -> usize =
            std::mem::transmute(msg);
        let set_u: unsafe extern "C" fn(*mut c_void, *const c_void, usize) =
            std::mem::transmute(msg);
        let set_i: unsafe extern "C" fn(*mut c_void, *const c_void, isize) =
            std::mem::transmute(msg);
        if let Some(next) = desktop_behaviour(get(ns, sel(b"collectionBehavior\0")), desktop) {
            set_u(ns, sel(b"setCollectionBehavior:\0"), next);
        }
        if desktop {
            let level = desktop_level(CGWindowLevelForKey(DESKTOP_ICON_LEVEL_KEY));
            set_i(ns, sel(b"setLevel:\0"), level);
        }
    }
}

/// One level above the desktop icons (`kCGDesktopIconWindowLevel`).
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn desktop_level(icon_level: i32) -> isize {
    icon_level as isize + 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{MonitorSelector, PlacementAnchor, WidgetPlacement};

    fn mon(name: &str, x: f64, w: f64, scale: f64) -> Monitor {
        let frame = Rect {
            x,
            y: 0.0,
            w,
            h: 1080.0 * scale,
        };
        Monitor {
            name: Some(name.into()),
            frame,
            // Menu bar / panel: 50 px off the top.
            work_area: Rect {
                y: 50.0,
                h: frame.h - 50.0,
                ..frame
            },
            scale,
        }
    }

    /// Where a window opens over a session: monitor queries that fail → x / y; a
    /// placement on the picked monitor (DPI-scaled) or the primary, matched by name and
    /// position. Remembered spots: `restored_only_when_grabbable_else_placed`.
    #[test]
    fn open_position_over_a_session() {
        // Tauri monitor data (physical ints) → rects; failing / panicking queries → none.
        assert_eq!(
            rect(-5, 10, 300, 200),
            Rect {
                x: -5.0,
                y: 10.0,
                w: 300.0,
                h: 200.0
            }
        );
        assert_eq!(monitor_query(|| Ok(5)), Some(5));
        assert_eq!(
            monitor_query::<i32>(|| Err(tauri::Error::WindowNotFound)),
            None
        );
        assert_eq!(monitor_query::<i32>(|| unimplemented!()), None);

        // No monitor info: only plain x / y (both needed) position the window.
        let mut cfg = WidgetWindowConfig {
            label: "w".into(),
            width: 100.0,
            height: 50.0,
            x: Some(12.0),
            y: Some(34.0),
            ..Default::default()
        };
        let plain = Some(Target {
            logical: (12.0, 34.0),
            physical: None,
        });
        assert_eq!(target_for(&cfg, None, &[], None), plain);
        cfg.y = None;
        assert_eq!(target_for(&cfg, None, &[], None), None);

        // Laptop A (1x) and external B (2x, primary).
        let ms = vec![mon("A", 0.0, 1920.0, 1.0), mon("B", 1920.0, 3840.0, 2.0)];
        let placement = |anchor, monitor| {
            Some(WidgetPlacement {
                monitor,
                anchor,
                offset_x: 10.0,
                offset_y: 10.0,
            })
        };
        // A placement without monitors still needs x / y.
        cfg.placement = placement(
            PlacementAnchor::BottomRight,
            Some(MonitorSelector::Index(1)),
        );
        assert_eq!(target_for(&cfg, None, &[], None), None);
        // B work area x 1920..5760, y 50..2160; window 200x100 physical, 20 px offsets.
        let t = target_for(&cfg, None, &ms, None).unwrap();
        assert_eq!(t.physical, Some(PhysicalPosition::new(5540, 2040)));
        assert_eq!(t.logical, (2770.0, 1020.0));

        // Top-left on the primary, matched by name and position.
        cfg.placement = placement(PlacementAnchor::TopLeft, None);
        let at = |p: Option<&Monitor>| target_for(&cfg, None, &ms, p).unwrap().physical;
        let on_a = Some(PhysicalPosition::new(10, 60));
        let on_b = Some(PhysicalPosition::new(1940, 70));
        assert_eq!(at(None), on_a);
        assert_eq!(at(Some(&ms[1])), on_b);
        // Same name somewhere else, or same spot under another name: not that monitor.
        let moved = Monitor {
            frame: Rect {
                x: 5000.0,
                ..ms[1].frame
            },
            ..ms[1].clone()
        };
        assert_eq!(at(Some(&moved)), on_a);
        let renamed = Monitor {
            name: Some("Z".into()),
            ..ms[1].clone()
        };
        assert_eq!(at(Some(&renamed)), on_a);
    }

    /// Monitors side by side (any sizes / scales / menu bars), a remembered frame anywhere
    /// (zero-size included) and a placement: the window reopens where it was only when
    /// that spot still shows ≥ 48 px (or all of a smaller window) on some monitor, in that
    /// monitor's scale; otherwise it falls back to the placement, inside a work area.
    fn monitor_row() -> impl Strategy<Value = Vec<Monitor>> {
        let one = (
            800u32..4000,
            900u32..2400,
            0u32..60,
            proptest::sample::select(vec![1.0, 1.25, 2.0]),
        );
        proptest::collection::vec(one, 1..4).prop_map(|ms| {
            let mut x = -1000.0;
            ms.into_iter()
                .enumerate()
                .map(|(i, (w, h, bar, scale))| {
                    let frame = Rect {
                        x,
                        y: 0.0,
                        w: w as f64,
                        h: h as f64,
                    };
                    x += w as f64;
                    Monitor {
                        name: Some(format!("m{i}")),
                        frame,
                        work_area: Rect {
                            y: bar as f64,
                            h: frame.h - bar as f64,
                            ..frame
                        },
                        scale,
                    }
                })
                .collect()
        })
    }

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn restored_only_when_grabbable_else_placed(
            ms in monitor_row(),
            (sx, sy, sw, sh) in (-3000i32..14000, -1500i32..3000, 0u32..800, 0u32..600),
            remember in any::<bool>(),
            anchor in super::super::placement::tests::any_anchor(),
            (ox, oy) in (0i32..40, 0i32..40),
            (w, h) in (50u32..300, 50u32..300),
            index in proptest::option::of(0u32..5),
        ) {
            let cfg = WidgetWindowConfig {
                label: "w".into(),
                width: w as f64,
                height: h as f64,
                remember_position: (!remember).then_some(false),
                placement: Some(WidgetPlacement {
                    monitor: index.map(MonitorSelector::Index),
                    anchor,
                    offset_x: ox as f64,
                    offset_y: oy as f64,
                }),
                ..Default::default()
            };
            let saved = SavedRect { x: sx, y: sy, w: sw, h: sh };
            let t = target_for(&cfg, Some(saved), &ms, None).unwrap();
            let p = t.physical.unwrap();
            let (px, py) = (p.x as f64, p.y as f64);
            // Overlap of a frame with a monitor on both axes.
            let overlap = |r: Rect, m: &Rect| {
                (
                    (r.x + r.w).min(m.x + m.w) - r.x.max(m.x),
                    (r.y + r.h).min(m.y + m.h) - r.y.max(m.y),
                )
            };
            let grabbable = ms.iter().find(|m| {
                let (ow, oh) = overlap(saved.rect(), &m.frame);
                ow > 0.0 && oh > 0.0 && ow >= sw.min(48) as f64 && oh >= sh.min(48) as f64
            });
            match grabbable.filter(|_| remember) {
                Some(m) => {
                    prop_assert_eq!((p.x, p.y), (sx, sy));
                    prop_assert_eq!(t.logical, (sx as f64 / m.scale, sy as f64 / m.scale));
                }
                None => {
                    // Placed: the whole window inside the picked monitor's work area.
                    let i = index.map_or(0, |i| i as usize).min(ms.len());
                    let m = ms.get(i).unwrap_or(&ms[0]);
                    let (ww, wh) = (w as f64 * m.scale, h as f64 * m.scale);
                    let wa = m.work_area;
                    prop_assert!(px >= wa.x - 0.5 && px + ww <= wa.x + wa.w + 0.5, "{p:?} {wa:?}");
                    prop_assert!(py >= wa.y - 0.5 && py + wh <= wa.y + wa.h + 0.5, "{p:?} {wa:?}");
                    prop_assert!((t.logical.0 * m.scale - px).abs() <= 0.5);
                    prop_assert!((t.logical.1 * m.scale - py).abs() <= 0.5);
                }
            }
        }
    }

    fn modes(layer: WindowLayer, focusable: bool, click_through: bool) -> WindowModes {
        WindowModes {
            layer,
            click_through,
            focusable,
            all_workspaces: false,
            shadow: None,
        }
    }

    /// Which setters a window gets: new with defaults → new with non-defaults → reused
    /// (everything reset) → the macOS desktop layer / island set natively → material →
    /// desktop collection behaviour toggled on and back off.
    #[test]
    fn window_modes_applied_over_reuse_and_platforms() {
        let normal = modes(WindowLayer::Normal, true, false);
        let desktop = modes(WindowLayer::Desktop, true, false);

        // New window, defaults: the builder already did focus / click-through.
        assert_eq!(
            mode_setters("w", &normal, false, false),
            ModeSetters {
                layer: true,
                focusable: false,
                click_through: false,
                island: false,
                desktop: false,
            }
        );
        let s = mode_setters("w", &modes(WindowLayer::Normal, false, true), false, false);
        assert!(s.focusable && s.click_through);
        // Reused for another config: every mode reset.
        let s = mode_setters("w", &normal, true, false);
        assert!(s.focusable && s.click_through);

        // macOS sets the desktop / island level itself (tao's async change would race).
        let s = mode_setters("w", &desktop, false, true);
        assert!(!s.layer && s.desktop && !s.island);
        assert!(mode_setters("w", &desktop, false, false).layer);
        assert!(mode_setters("w", &normal, false, true).layer);
        let island = mode_setters(super::super::island::LABEL, &normal, false, true);
        assert!(!island.layer && island.island && !island.desktop);

        // Material: one native effect, none where the OS has no match.
        let c = effects_config(Some(WindowEffectHint::Mica), "windows").unwrap();
        assert_eq!(c.effects, vec![tauri::window::Effect::Mica]);
        assert_eq!(effects_config(None, "windows"), None);
        assert_eq!(effects_config(Some(WindowEffectHint::Mica), "linux"), None);

        // Desktop layer on: stationary + out of the cycle, other bits kept; already set →
        // no call; off again → bits cleared; already clear → no call.
        assert_eq!(desktop_behaviour(0x1, true), Some(0x51));
        assert_eq!(desktop_behaviour(0x51, true), None);
        assert_eq!(desktop_behaviour(0x51, false), Some(0x1));
        assert_eq!(desktop_behaviour(0x1, false), None);
        // Just above kCGDesktopIconWindowLevel.
        assert_eq!(desktop_level(-2147483603), -2147483602);
    }
}
