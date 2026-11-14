//! Linux desktop-widget pinning: GDK/EWMH hints before the first map and optional
//! gtk-layer-shell on Wayland.

use tauri::{Runtime, WebviewWindow};

use crate::desktop::placement::WindowModes;
use crate::models::{WidgetWindowConfig, WindowLayer};

/// Pin a **not yet shown** widget window.
///
/// - **Wayland** (`feature = "layer-shell"`, layer other than `normal`): the webview moves
///   into a gtk-layer-shell surface (anchors / margins from the position, monitor,
///   namespace = label) which is shown here. Returns `true`.
/// - Otherwise sets GDK hints that land in the first map, so the window manager never
///   sees an ordinary window: `_NET_WM_WINDOW_TYPE_DESKTOP` for the `desktop` layer,
///   `SKIP_TASKBAR` / `SKIP_PAGER`, `STICKY`, `BELOW`. Returns `false`: the caller shows
///   the window and re-applies its position (WMs often ignore it for DESKTOP windows).
///
/// Failures are logged; the ordinary frameless window remains usable.
pub(crate) fn pin_widget_window<R: Runtime>(
    win: &WebviewWindow<R>,
    cfg: &WidgetWindowConfig,
    modes: &WindowModes,
    pos: Option<(f64, f64)>,
) -> bool {
    // Prefer the actual native backend over env vars: X11 apps can still run when
    // WAYLAND_DISPLAY is set (XWayland).
    let is_x11 = x11_xid(win).is_some();

    #[cfg(feature = "layer-shell")]
    if !is_x11 && modes.layer != WindowLayer::Normal {
        match apply_layer_shell(win, cfg, modes, pos) {
            Ok(()) => {
                log::debug!("linux: gtk-layer-shell surface for '{}'", cfg.label);
                return true;
            }
            Err(e) => log::warn!("linux: layer-shell failed ({e}); keeping normal window"),
        }
    }
    #[cfg(not(feature = "layer-shell"))]
    let _ = (pos, is_x11);

    let island = cfg.label == crate::desktop::island::LABEL;
    if let Err(e) = apply_gtk_hints(win, modes, cfg.skip_taskbar, island) {
        log::warn!("linux: desktop hints failed: {e}");
    }
    false
}

fn apply_gtk_hints<R: Runtime>(
    win: &WebviewWindow<R>,
    modes: &WindowModes,
    skip_taskbar: bool,
    island: bool,
) -> Result<(), String> {
    use gtk::prelude::*;
    let gw = win.gtk_window().map_err(|e| format!("gtk_window: {e}"))?;
    if modes.layer == WindowLayer::Desktop {
        gw.set_type_hint(gtk::gdk::WindowTypeHint::Desktop);
    }
    // Activity island: a DOCK stays above normal windows on every desktop, unfocused.
    if island {
        gw.set_type_hint(gtk::gdk::WindowTypeHint::Dock);
    }
    if skip_taskbar {
        gw.set_skip_taskbar_hint(true);
        gw.set_skip_pager_hint(true);
    }
    if modes.all_workspaces {
        gw.stick();
    }
    gw.set_keep_below(modes.on_bottom());
    gw.set_keep_above(modes.on_top());
    Ok(())
}

fn x11_xid<R: Runtime>(win: &WebviewWindow<R>) -> Option<u32> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    let handle = win.window_handle().ok()?;
    match handle.as_raw() {
        RawWindowHandle::Xlib(h) => Some(h.window as u32),
        RawWindowHandle::Xcb(h) => Some(h.window.get()),
        _ => None,
    }
}

#[cfg(feature = "layer-shell")]
fn apply_layer_shell<R: Runtime>(
    win: &WebviewWindow<R>,
    cfg: &WidgetWindowConfig,
    modes: &WindowModes,
    pos: Option<(f64, f64)>,
) -> Result<(), String> {
    use crate::desktop::placement::{layer_anchor, Rect};
    use gtk::prelude::*;
    use gtk_layer_shell::{Edge, Layer, LayerShell};

    let island = cfg.label == crate::desktop::island::LABEL;
    let layer = match modes.layer {
        _ if island => Layer::Overlay,
        WindowLayer::Desktop => Layer::Background,
        WindowLayer::Bottom => Layer::Bottom,
        WindowLayer::Top => Layer::Top,
        WindowLayer::Normal => return Err("normal layer".into()),
    };

    // The Tauri window is still unmapped: init_layer_shell needs that.
    let old = win.gtk_window().map_err(|e| format!("gtk_window: {e}"))?;
    let app = old
        .application()
        .ok_or_else(|| "gtk window has no application".to_string())?;
    let vbox = win
        .default_vbox()
        .map_err(|e| format!("default_vbox: {e}"))?;
    old.remove(&vbox);

    let layer_win = gtk::ApplicationWindow::new(&app);
    layer_win.set_app_paintable(true);
    if let Some(v) = WidgetExt::screen(&layer_win).and_then(|s| s.rgba_visual()) {
        layer_win.set_visual(Some(&v));
    }
    layer_win.add(&vbox);

    layer_win.init_layer_shell();
    layer_win.set_layer(layer);
    layer_win.set_namespace(&cfg.label);

    let monitor = gtk::gdk::Display::default().and_then(|d| match pos {
        Some((x, y)) => d.monitor_at_point(x as i32, y as i32),
        None => d.primary_monitor(),
    });
    let geom = monitor.as_ref().map_or(Rect::default(), |m| {
        let g = m.geometry();
        Rect {
            x: g.x() as f64,
            y: g.y() as f64,
            w: g.width() as f64,
            h: g.height() as f64,
        }
    });
    if let Some(m) = &monitor {
        layer_win.set_monitor(m);
    }
    let mut a = layer_anchor(cfg.placement.as_ref(), pos, geom);
    // Activity island: top edge only, so the compositor keeps it centred as it resizes.
    if island {
        a.left = None;
        a.right = None;
    }
    for (edge, margin) in [
        (Edge::Top, a.top),
        (Edge::Bottom, a.bottom),
        (Edge::Left, a.left),
        (Edge::Right, a.right),
    ] {
        layer_win.set_anchor(edge, margin.is_some());
        layer_win.set_layer_shell_margin(edge, margin.unwrap_or(0));
    }
    #[allow(deprecated)]
    layer_win.set_keyboard_interactivity(modes.focusable);
    if modes.click_through {
        layer_win.input_shape_combine_region(Some(&gtk::cairo::Region::create()));
    }

    layer_win.set_default_size(cfg.width.round() as i32, cfg.height.round() as i32);
    layer_win.show_all();

    // The webview now lives in `layer_win`; close it together with the (hidden) Tauri
    // window, otherwise close_widget_window leaves it on screen.
    let layer = layer_win.clone();
    old.connect_destroy(move |_| layer.close());

    Ok(())
}
