//! Pure window maths and the remembered window state (`widget-windows.json`):
//! placement in a work area, restore-visibility check, window-mode mapping, effects,
//! and Wayland layer-shell anchors. No tauri types here, so it is unit-tested directly.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::models::{
    MonitorSelector, PlacementAnchor, WidgetPlacement, WidgetWindowConfig, WindowEffectHint,
    WindowLayer,
};

/// Axis-aligned rectangle (units depend on the caller: logical or physical px).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub(crate) struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// Top-left of a `size` window anchored inside `area` (same units as `area`).
pub(crate) fn place(p: &WidgetPlacement, area: Rect, size: (f64, f64), scale: f64) -> (f64, f64) {
    use PlacementAnchor::*;
    let (w, h) = size;
    let (ox, oy) = (p.offset_x * scale, p.offset_y * scale);
    let x = match p.anchor {
        TopLeft | Left | BottomLeft => area.x + ox,
        Top | Center | Bottom => area.x + (area.w - w) / 2.0 + ox,
        TopRight | Right | BottomRight => area.x + area.w - w - ox,
    };
    let y = match p.anchor {
        TopLeft | Top | TopRight => area.y + oy,
        Left | Center | Right => area.y + (area.h - h) / 2.0 + oy,
        BottomLeft | Bottom | BottomRight => area.y + area.h - h - oy,
    };
    (x, y)
}

/// Index of the monitor a selector names; `primary` (or 0) when it names none.
pub(crate) fn pick_monitor(
    sel: Option<&MonitorSelector>,
    names: &[Option<String>],
    primary: Option<usize>,
) -> usize {
    let fallback = primary.filter(|i| *i < names.len()).unwrap_or(0);
    match sel {
        Some(MonitorSelector::Index(i)) if (*i as usize) < names.len() => *i as usize,
        Some(MonitorSelector::Name(n)) => names
            .iter()
            .position(|m| m.as_deref() == Some(n.as_str()))
            .unwrap_or(fallback),
        _ => fallback,
    }
}

/// A remembered window still shows enough of itself on some monitor to be grabbed:
/// at least 48 px (or the whole window when smaller) overlap on both axes.
pub(crate) fn still_visible(saved: Rect, monitors: &[Rect]) -> bool {
    const MIN: f64 = 48.0;
    monitors.iter().any(|m| {
        let ow = (saved.x + saved.w).min(m.x + m.w) - saved.x.max(m.x);
        let oh = (saved.y + saved.h).min(m.y + m.h) - saved.y.max(m.y);
        ow >= MIN.min(saved.w) && oh >= MIN.min(saved.h) && ow > 0.0 && oh > 0.0
    })
}

/// Effective window behaviour after defaults.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WindowModes {
    pub layer: WindowLayer,
    pub click_through: bool,
    pub focusable: bool,
    pub all_workspaces: bool,
    pub shadow: Option<bool>,
}

impl WindowModes {
    /// `always_on_top` (tauri) for this layer.
    pub fn on_top(&self) -> bool {
        self.layer == WindowLayer::Top
    }
    /// `always_on_bottom` (tauri) for this layer.
    pub fn on_bottom(&self) -> bool {
        matches!(self.layer, WindowLayer::Bottom | WindowLayer::Desktop)
    }
}

/// Defaults: `layer` from `alwaysOnTop`, else `desktop` on Linux (the historical X11
/// DESKTOP pin) and `normal` elsewhere; `allWorkspaces` on for the desktop layer.
pub(crate) fn window_modes(cfg: &WidgetWindowConfig, linux: bool) -> WindowModes {
    let layer = cfg.layer.unwrap_or(if cfg.always_on_top {
        WindowLayer::Top
    } else if linux {
        WindowLayer::Desktop
    } else {
        WindowLayer::Normal
    });
    WindowModes {
        layer,
        click_through: cfg.click_through.unwrap_or(false),
        focusable: cfg.focusable.unwrap_or(true),
        all_workspaces: cfg.all_workspaces.unwrap_or(layer == WindowLayer::Desktop),
        shadow: cfg.shadow,
    }
}

/// Native material for an effect hint on one OS (`"macos"` / `"windows"`); none elsewhere.
pub(crate) fn effect_for(hint: WindowEffectHint, os: &str) -> Option<tauri::window::Effect> {
    use tauri::window::Effect;
    use WindowEffectHint::*;
    match (os, hint) {
        ("macos", Vibrancy | Acrylic) => Some(Effect::HudWindow),
        ("macos", Mica | Blur) => Some(Effect::UnderWindowBackground),
        ("windows", Mica) => Some(Effect::Mica),
        ("windows", Vibrancy | Acrylic) => Some(Effect::Acrylic),
        ("windows", Blur) => Some(Effect::Blur),
        _ => None,
    }
}

/// Wayland layer-shell edges: `Some(margin)` anchors that edge (logical px).
#[cfg_attr(
    not(all(target_os = "linux", feature = "layer-shell")),
    allow(dead_code)
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct LayerAnchor {
    pub top: Option<i32>,
    pub bottom: Option<i32>,
    pub left: Option<i32>,
    pub right: Option<i32>,
}

/// Anchors + margins on `monitor` (logical): right / bottom anchors keep their offset
/// when the output resizes; anything else pins the top-left at `pos`. No position and
/// no placement: no anchors (the compositor centers the surface).
#[cfg_attr(
    not(all(target_os = "linux", feature = "layer-shell")),
    allow(dead_code)
)]
pub(crate) fn layer_anchor(
    placement: Option<&WidgetPlacement>,
    pos: Option<(f64, f64)>,
    monitor: Rect,
) -> LayerAnchor {
    use PlacementAnchor::*;
    let Some((x, y)) = pos else {
        return LayerAnchor::default();
    };
    let anchor = placement.map(|p| p.anchor);
    let mut a = LayerAnchor::default();
    match (anchor, placement) {
        (Some(TopRight | Right | BottomRight), Some(p)) => a.right = Some(p.offset_x as i32),
        _ => a.left = Some((x - monitor.x).round() as i32),
    }
    match (anchor, placement) {
        (Some(BottomLeft | Bottom | BottomRight), Some(p)) => a.bottom = Some(p.offset_y as i32),
        _ => a.top = Some((y - monitor.y).round() as i32),
    }
    a
}

/// Remembered geometry (physical px of the outer frame).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub(crate) struct SavedRect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl SavedRect {
    pub fn rect(&self) -> Rect {
        Rect {
            x: self.x as f64,
            y: self.y as f64,
            w: self.w as f64,
            h: self.h as f64,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct WindowStateFile {
    #[serde(default)]
    positions: BTreeMap<String, SavedRect>,
    /// Widget windows open when the app last ran (reopened by `restoreWindowsOnStart`).
    #[serde(default)]
    open: BTreeMap<String, WidgetWindowConfig>,
}

/// `{app_data}/widget-windows.json`: positions per label and the open window set.
#[derive(Default)]
pub(crate) struct WindowStore {
    path: Option<PathBuf>,
    data: Mutex<WindowStateFile>,
    /// A delayed flush is queued (window drags fire many `Moved` events).
    pub flush_pending: AtomicBool,
}

impl WindowStore {
    pub fn load(path: Option<PathBuf>) -> Self {
        let data = path
            .as_ref()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        Self {
            path,
            data: Mutex::new(data),
            flush_pending: AtomicBool::new(false),
        }
    }

    pub fn position(&self, label: &str) -> Option<SavedRect> {
        self.data.lock().unwrap().positions.get(label).copied()
    }

    /// Record a move in memory; `true` when the caller should schedule a flush.
    pub fn set_position(&self, label: &str, r: SavedRect) -> bool {
        let changed = self.data.lock().unwrap().positions.insert(label.into(), r) != Some(r);
        changed && !self.flush_pending.swap(true, Ordering::SeqCst)
    }

    pub fn set_open(&self, label: &str, cfg: Option<&WidgetWindowConfig>) {
        {
            let mut d = self.data.lock().unwrap();
            match cfg {
                Some(c) => d.open.insert(label.into(), c.clone()),
                None => d.open.remove(label),
            };
        }
        self.save();
    }

    pub fn open_windows(&self) -> Vec<WidgetWindowConfig> {
        self.data.lock().unwrap().open.values().cloned().collect()
    }

    pub fn save(&self) {
        self.flush_pending.store(false, Ordering::SeqCst);
        let Some(path) = &self.path else { return };
        let json = match serde_json::to_vec_pretty(&*self.data.lock().unwrap()) {
            Ok(j) => j,
            Err(_) => return,
        };
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Err(e) = super::atomic_write(path, &json) {
            log::warn!("widgets: save {}: {e}", path.display());
        }
    }
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;

    const LAPTOP: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 1920.0,
        h: 1080.0,
    };
    const EXTERNAL: Rect = Rect {
        x: 1920.0,
        y: 0.0,
        w: 2560.0,
        h: 1440.0,
    };

    fn at(anchor: PlacementAnchor, ox: f64, oy: f64) -> WidgetPlacement {
        WidgetPlacement {
            monitor: None,
            anchor,
            offset_x: ox,
            offset_y: oy,
        }
    }

    pub(crate) fn any_anchor() -> impl proptest::strategy::Strategy<Value = PlacementAnchor> {
        use PlacementAnchor::*;
        proptest::sample::select(vec![
            TopLeft,
            Top,
            TopRight,
            Left,
            Center,
            Right,
            BottomLeft,
            Bottom,
            BottomRight,
        ])
    }

    proptest::proptest! {
        /// Any anchor, offset, work area, window size and scale: the window keeps exactly
        /// the scaled offset from its anchored edge (or from the centre line).
        #[test]
        fn placement_keeps_the_offset_from_its_anchor(
            anchor in any_anchor(),
            (ox, oy) in (-50i32..200, -50i32..200),
            (ax, ay, aw, ah) in (-4000i32..4000, -100i32..1500, 400u32..4000, 300u32..2400),
            (w, h) in (20u32..600, 20u32..600),
            scale in proptest::sample::select(vec![1.0, 1.25, 1.5, 2.0]),
        ) {
            use PlacementAnchor::*;
            let area = Rect { x: ax as f64, y: ay as f64, w: aw as f64, h: ah as f64 };
            let (w, h) = (w as f64, h as f64);
            let (x, y) = place(&at(anchor, ox as f64, oy as f64), area, (w, h), scale);
            let dx = match anchor {
                TopLeft | Left | BottomLeft => x - area.x,
                TopRight | Right | BottomRight => area.x + area.w - (x + w),
                Top | Center | Bottom => (x + w / 2.0) - (area.x + area.w / 2.0),
            };
            let dy = match anchor {
                TopLeft | Top | TopRight => y - area.y,
                BottomLeft | Bottom | BottomRight => area.y + area.h - (y + h),
                Left | Center | Right => (y + h / 2.0) - (area.y + area.h / 2.0),
            };
            proptest::prop_assert!((dx - ox as f64 * scale).abs() < 1e-6, "dx {dx}");
            proptest::prop_assert!((dy - oy as f64 * scale).abs() < 1e-6, "dy {dy}");
        }
    }

    /// A window opened on the external monitor, dragged around, remembered, and brought
    /// back after monitors change: unplugged → falls back to the placement on another
    /// monitor (by index, name or primary); edge cases of "still grabbable".
    #[test]
    fn remembered_position_across_monitor_changes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("widget-windows.json");
        let monitors = [LAPTOP, EXTERNAL];

        // The user drags it to the external monitor; the first move queues a flush.
        let store = WindowStore::load(Some(path.clone()));
        let dragged = SavedRect {
            x: 3000,
            y: 1200,
            w: 300,
            h: 200,
        };
        assert_eq!(
            dragged.rect(),
            Rect {
                x: 3000.0,
                y: 1200.0,
                w: 300.0,
                h: 200.0
            }
        );
        assert!(store.set_position("w", dragged));
        let moved = SavedRect {
            x: 100,
            y: 100,
            ..dragged
        };
        assert!(!store.set_position("w", moved)); // flush already queued
        store.save();
        assert!(!store.set_position("w", moved)); // unchanged
        store.set_open(
            "w",
            Some(&WidgetWindowConfig {
                label: "w".into(),
                width: 280.0,
                ..Default::default()
            }),
        );

        // Next launch: position and open set come back from disk.
        let back = WindowStore::load(Some(path));
        assert_eq!(back.position("w"), Some(moved));
        assert_eq!(back.open_windows()[0].width, 280.0);
        assert!(still_visible(moved.rect(), &monitors));
        assert!(still_visible(dragged.rect(), &monitors)); // on the external monitor

        // External monitor unplugged: its spot is gone, the placement picks a monitor.
        assert!(!still_visible(dragged.rect(), &monitors[..1]));
        let names = vec![Some("A".to_string()), Some("B".to_string()), None];
        let pick =
            |sel: Option<MonitorSelector>, primary| pick_monitor(sel.as_ref(), &names, primary);
        assert_eq!(pick(Some(MonitorSelector::Index(2)), Some(1)), 2);
        assert_eq!(pick(Some(MonitorSelector::Index(9)), Some(1)), 1);
        assert_eq!(pick(Some(MonitorSelector::Index(3)), Some(1)), 1); // one past the end
        assert_eq!(pick(Some(MonitorSelector::Name("A".into())), Some(1)), 0);
        assert_eq!(pick(Some(MonitorSelector::Name("Z".into())), None), 0);
        assert_eq!(pick(None, Some(7)), 0);
        assert_eq!(pick(None, Some(3)), 0);
        let sel: MonitorSelector = serde_json::from_str("1").unwrap();
        assert_eq!(sel, MonitorSelector::Index(1));

        // How much must stay on screen: 48 px (or the whole window when smaller).
        let r = |x, y| Rect {
            x,
            y,
            w: 300.0,
            h: 200.0,
        };
        assert!(!still_visible(r(5000.0, 100.0), &monitors)); // unplugged third monitor
        assert!(!still_visible(r(1900.0, 1070.0), &monitors[..1])); // 20x10 sliver
        assert!(!still_visible(r(100.0, 1070.0), &monitors[..1])); // 10 px tall strip
        assert!(still_visible(r(1872.0, 0.0), &monitors[..1])); // 48 px left on screen
        assert!(still_visible(r(-252.0, 0.0), &monitors[..1])); // 48 px right edge left
        assert!(!still_visible(r(-290.0, 0.0), &monitors));
        let sized = |w, h| Rect {
            x: 10.0,
            y: 10.0,
            w,
            h,
        };
        assert!(still_visible(sized(20.0, 20.0), &monitors)); // tiny but whole
                                                              // Zero-size frames never count (0 >= min(48, 0) alone would pass).
        assert!(!still_visible(sized(0.0, 200.0), &monitors));
        assert!(!still_visible(sized(300.0, 0.0), &monitors));

        // The window is closed: it no longer reopens on start.
        back.set_open("w", None);
        assert!(back.open_windows().is_empty());
    }

    /// A config through to native behaviour: parsing (camelCase, optional fields) →
    /// effective modes and their defaults → material per OS → Wayland anchors.
    #[test]
    fn window_config_to_native_modes() {
        let c: WidgetWindowConfig = serde_json::from_str(
            r#"{"label":"w","width":200,"height":100,"layer":"desktop","clickThrough":true,
                "allWorkspaces":false,"effects":"mica",
                "placement":{"monitor":"DP-1","anchor":"bottom-right","offsetX":16}}"#,
        )
        .unwrap();
        assert_eq!(c.layer, Some(WindowLayer::Desktop));
        assert_eq!(c.effects, Some(WindowEffectHint::Mica));
        let p = c.placement.clone().unwrap();
        assert_eq!(p.anchor, PlacementAnchor::BottomRight);
        assert_eq!(p.monitor, Some(MonitorSelector::Name("DP-1".into())));
        assert_eq!(p.offset_y, 0.0);
        let m = window_modes(&c, false);
        assert!(m.on_bottom() && m.click_through && !m.all_workspaces);

        // An old config without the new fields: historical defaults.
        let mut old: WidgetWindowConfig =
            serde_json::from_str(r#"{"label":"w","width":1,"height":1}"#).unwrap();
        assert!(old.skip_taskbar && old.layer.is_none() && old.placement.is_none());
        assert_eq!(window_modes(&old, true).layer, WindowLayer::Desktop);
        assert!(window_modes(&old, true).all_workspaces);
        assert_eq!(window_modes(&old, false).layer, WindowLayer::Normal);
        assert!(!window_modes(&old, false).all_workspaces);
        old.always_on_top = true;
        let m = window_modes(&old, true);
        assert_eq!(m.layer, WindowLayer::Top);
        assert!(m.on_top() && !m.on_bottom());
        old.layer = Some(WindowLayer::Bottom);
        old.click_through = Some(true);
        old.focusable = Some(false);
        let m = window_modes(&old, false);
        assert!(m.on_bottom() && !m.on_top() && m.click_through && !m.focusable);

        // Materials per OS.
        use tauri::window::Effect;
        use WindowEffectHint::*;
        let effects = [
            (Mica, "windows", Some(Effect::Mica)),
            (Acrylic, "windows", Some(Effect::Acrylic)),
            (Blur, "windows", Some(Effect::Blur)),
            (Vibrancy, "macos", Some(Effect::HudWindow)),
            (Blur, "macos", Some(Effect::UnderWindowBackground)),
            (Mica, "linux", None),
        ];
        for (hint, os, want) in effects {
            assert_eq!(effect_for(hint, os), want, "{hint:?} on {os}");
        }

        // Wayland layer-shell anchors on the external output.
        let mon = EXTERNAL;
        assert_eq!(layer_anchor(None, None, mon), LayerAnchor::default());
        // Plain x/y: top-left margins relative to the output.
        assert_eq!(
            layer_anchor(None, Some((2000.0, 50.0)), mon),
            LayerAnchor {
                top: Some(50),
                left: Some(80),
                ..Default::default()
            }
        );
        let br = at(PlacementAnchor::BottomRight, 16.0, 24.0);
        assert_eq!(
            layer_anchor(Some(&br), Some((4000.0, 1300.0)), mon),
            LayerAnchor {
                bottom: Some(24),
                right: Some(16),
                ..Default::default()
            }
        );
        let c = at(PlacementAnchor::Center, 0.0, 0.0);
        assert_eq!(
            layer_anchor(Some(&c), Some((3100.0, 670.0)), mon),
            LayerAnchor {
                top: Some(670),
                left: Some(1180),
                ..Default::default()
            }
        );
        // Output below another one: top margin relative to the output's own top.
        let lower = Rect { y: 1080.0, ..mon };
        assert_eq!(
            layer_anchor(None, Some((2000.0, 1130.0)), lower).top,
            Some(50)
        );
    }
}
