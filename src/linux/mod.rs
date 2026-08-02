//! Linux desktop-widget pinning: X11 `_NET_WM_*` hints and optional gtk-layer-shell.

use tauri::{Runtime, WebviewWindow};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt as XprotoConnectionExt, PropMode};
use x11rb::wrapper::ConnectionExt as WrapperConnectionExt;

/// Pin a widget webview as a desktop-layer surface when possible.
///
/// - **X11**: `_NET_WM_WINDOW_TYPE_DESKTOP` (+ `_NET_WM_STATE_SKIP_TASKBAR` when requested).
/// - **Wayland** (`feature = "layer-shell"`): remap into a gtk-layer-shell Background surface.
/// - Failures are logged; the ordinary frameless window remains usable (fallback).
pub fn pin_widget_window<R: Runtime>(win: &WebviewWindow<R>, skip_taskbar: bool) {
    #[cfg(feature = "layer-shell")]
    {
        if std::env::var_os("WAYLAND_DISPLAY").is_some() {
            match apply_layer_shell(win) {
                Ok(()) => {
                    log::debug!("linux: gtk-layer-shell Background applied");
                    return;
                }
                Err(e) => {
                    log::warn!("linux: layer-shell failed ({e}); keeping normal window");
                }
            }
        }
    }

    if std::env::var_os("WAYLAND_DISPLAY").is_none() {
        if let Err(e) = apply_x11_desktop_hints(win, skip_taskbar) {
            log::warn!("linux: X11 desktop hints failed: {e}");
        }
    }
}

fn apply_x11_desktop_hints<R: Runtime>(
    win: &WebviewWindow<R>,
    skip_taskbar: bool,
) -> Result<(), String> {
    let xid = x11_xid(win).ok_or_else(|| "no X11 window id".to_string())?;

    let (conn, _) = x11rb::connect(None).map_err(|e| format!("x11 connect: {e}"))?;

    let type_atom = intern(&conn, b"_NET_WM_WINDOW_TYPE")?;
    let desktop_atom = intern(&conn, b"_NET_WM_WINDOW_TYPE_DESKTOP")?;
    WrapperConnectionExt::change_property32(
        &conn,
        PropMode::REPLACE,
        xid,
        type_atom,
        AtomEnum::ATOM,
        &[desktop_atom],
    )
    .map_err(|e| format!("change_property TYPE: {e}"))?;

    if skip_taskbar {
        let state_atom = intern(&conn, b"_NET_WM_STATE")?;
        let skip_atom = intern(&conn, b"_NET_WM_STATE_SKIP_TASKBAR")?;
        // Merge with existing state so ABOVE / sticky bits survive (REPLACE would wipe them).
        // get_property → Cookie (ConnectionError); reply() → ReplyError — different Err types.
        let mut atoms: Vec<u32> = match XprotoConnectionExt::get_property(
            &conn,
            false,
            xid,
            state_atom,
            AtomEnum::ATOM,
            0,
            64,
        )
        .ok()
        .and_then(|c| c.reply().ok())
        {
            Some(reply) if reply.format == 32 => reply
                .value32()
                .map(|it| it.collect())
                .unwrap_or_default(),
            _ => Vec::new(),
        };
        if !atoms.contains(&skip_atom) {
            atoms.push(skip_atom);
        }
        WrapperConnectionExt::change_property32(
            &conn,
            PropMode::REPLACE,
            xid,
            state_atom,
            AtomEnum::ATOM,
            &atoms,
        )
        .map_err(|e| format!("change_property STATE: {e}"))?;
    }

    conn.flush().map_err(|e| format!("x11 flush: {e}"))?;
    log::debug!("linux: X11 DESKTOP hints set on xid={xid}");
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

fn intern(conn: &impl Connection, name: &[u8]) -> Result<u32, String> {
    Ok(XprotoConnectionExt::intern_atom(conn, false, name)
        .map_err(|e| format!("intern_atom: {e}"))?
        .reply()
        .map_err(|e| format!("intern_atom reply: {e}"))?
        .atom)
}

#[cfg(feature = "layer-shell")]
fn apply_layer_shell<R: Runtime>(win: &WebviewWindow<R>) -> Result<(), String> {
    use gtk::prelude::*;
    use gtk_layer_shell::{Edge, Layer, LayerShell};

    // Window must not be mapped when init_layer_shell runs — hide first.
    let _ = win.hide();

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
    layer_win.add(&vbox);

    layer_win.init_layer_shell();
    layer_win.set_layer(Layer::Background);
    layer_win.set_anchor(Edge::Left, false);
    layer_win.set_anchor(Edge::Right, false);
    layer_win.set_anchor(Edge::Top, false);
    layer_win.set_anchor(Edge::Bottom, false);
    // Keyboard interactivity — best-effort across gtk-layer-shell versions.
    #[allow(unused_must_use)]
    {
        layer_win.set_keyboard_interactivity(false);
    }

    let (w, h) = win
        .inner_size()
        .map(|s| (s.width as i32, s.height as i32))
        .unwrap_or((340, 340));
    layer_win.set_default_size(w, h);
    layer_win.show_all();

    Ok(())
}
