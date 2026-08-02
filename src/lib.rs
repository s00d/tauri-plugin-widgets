//! # tauri-plugin-widgets
//!
//! A Tauri v2 plugin for building native widgets on Android, iOS, macOS,
//! Windows, and Linux from a single JSON UI configuration.
//!
//! ## Overview
//!
//! - **Widget Config API** — send a declarative [`WidgetConfig`](models::WidgetConfig)
//!   describing layouts and elements. The native widget renders it using
//!   SwiftUI (Apple), RemoteViews (Android), or HTML/CSS (desktop).
//!
//! - **Data API** — key-value storage shared with native widget extensions
//!   via the App Group shared container (Apple), SharedPreferences (Android),
//!   or JSON files (desktop).
//!
//! - **Desktop widget windows** — frameless, transparent Tauri webview windows
//!   that render the same JSON config as HTML/CSS.
//!
//! ## Architecture
//!
//! The plugin acts as a **library**, not a builder. It does NOT compile or
//! inject widget extensions at runtime. Instead, it provides:
//!
//! 1. **Rust side** — commands for data storage and WidgetKit reload
//! 2. **Swift Package** (`swift/TauriWidgets`) — public SwiftUI views and
//!    models that developers import into their own Widget Extension target
//! 3. **Templates** (`templates/`) — ready-to-use scripts and Swift files
//!
//! This follows Apple's guidelines: the extension is built by Xcode, signed
//! with the developer's certificate, and included in the app bundle at
//! compile time.
//!
//! ## Quick Start (Rust)
//!
//! ```no_run
//! tauri::Builder::default()
//!     .plugin(tauri_plugin_widgets::init());
//! ```
//!
//! ## iOS Setup
//!
//! 1. Open `gen/apple/*.xcodeproj` in Xcode
//! 2. File → New → Target → Widget Extension
//! 3. Add `swift/` as a Local Swift Package dependency
//! 4. Add `TauriWidgets` library to the Widget Extension target
//! 5. Enable **App Groups** in both targets (App + Widget Extension)
//! 6. Use the template from `templates/ios-widget/MyWidget.swift`
//!
//! ## macOS Setup ("Satellite Project")
//!
//! Tauri for macOS does not generate an `.xcodeproj`, so the widget
//! extension must be built as a separate Xcode project:
//!
//! 1. Create `src-tauri/widget-extension/` with an Xcode project
//!    containing a Widget Extension target
//! 2. Add `swift/` as a Local Swift Package dependency
//! 3. Enable **App Groups** in both the main app entitlements and
//!    the widget extension entitlements
//! 4. `build-widget.sh` runs via `beforeBundleCommand` (builds + signs `.appex`)
//! 5. `bundle.macOS.files` copies the `.appex` into `Contents/PlugIns/`
//!    during a normal `tauri build` (Tauri nested-codesigns PlugIns)
//! 6. Set `plugins.widgets.transport` (`appGroup` with Team ID, or
//!    `widgetContainer` for ad-hoc) and `plugins.widgets.appGroup`
//!
//! ## Rust API
//!
//! Build a config with typed helpers (compile-checked, not executed here):
//!
//! ```
//! use tauri_plugin_widgets::models::{text, vstack, WidgetConfig};
//!
//! let _cfg = WidgetConfig::small(vstack(vec![
//!     text("72°").font_size(36.0).into(),
//! ]));
//! ```
//!
//! Then call [`WidgetExt::widget`] on an `AppHandle` to `set_widget_config` /
//! `reload_all_timelines` (requires a running Tauri app).

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]

#[cfg(mobile)]
use tauri::RunEvent;
use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

#[cfg(desktop)]
use std::borrow::Cow;

#[cfg(desktop)]
#[allow(missing_docs)]
#[cfg_attr(docsrs, doc(cfg(desktop)))]
pub mod desktop;
#[cfg(mobile)]
#[allow(missing_docs)]
#[cfg_attr(docsrs, doc(cfg(mobile)))]
pub mod mobile;

/// Adaptive Cards transpiler (Windows Widgets Board).
#[allow(missing_docs)]
pub mod adaptive_card;
/// Outcomes for `set_widget_config` (written / reload / skip).
pub mod apply;
/// Element × platform capability matrix.
#[allow(missing_docs)]
pub mod capabilities;
/// TypeScript IR emitter (`gen-ts`).
#[allow(missing_docs)]
pub mod codegen;
mod commands;
/// Plugin configuration (`plugins.widgets` in `tauri.conf.json`).
#[allow(missing_docs)]
pub mod config;
/// Plugin error type.
pub mod error;
/// Widget IR models (`WidgetConfig`, `WidgetElement`, …).
///
/// Element structs and their fields carry rustdoc used by `schemars` / docs site.
#[allow(missing_docs)]
pub mod models;
/// SVG / PNG helpers for Adaptive Cards fallbacks.
#[allow(missing_docs)]
pub mod rasterize;
/// Render receipts written by native / desktop surfaces.
#[allow(missing_docs)]
pub mod receipt;
/// Canonical layout dumps for snapshot tests.
#[allow(missing_docs)]
pub mod snapshot;
/// Shared key-value store helpers and action envelopes.
#[allow(missing_docs)]
pub mod store;
/// Host black-box journal (`WIDGET_DEBUG` / debug builds).
#[allow(missing_docs)]
pub mod trace;
/// macOS / desktop config transport selection.
#[allow(missing_docs)]
pub mod transport;

#[cfg(target_os = "windows")]
#[allow(missing_docs)]
#[cfg_attr(docsrs, doc(cfg(windows)))]
pub mod windows;

#[cfg(all(target_os = "linux", feature = "linux"))]
#[allow(missing_docs)]
#[cfg_attr(docsrs, doc(cfg(all(target_os = "linux", feature = "linux"))))]
pub mod linux;

#[cfg(target_os = "macos")]
#[allow(missing_docs)]
#[cfg_attr(docsrs, doc(cfg(macos)))]
pub mod macos_transport;

pub use adaptive_card::{to_adaptive_card, to_adaptive_card_for_size, TranspileResult};
pub use apply::{ApplyOutcome, ReloadOutcome, SkipReason};
pub use config::{TransportKind, WidgetsPluginConfig};
pub use error::{Error, Result};
pub use receipt::{SkippedElement, WidgetRenderReceipt};
pub use store::WidgetActionEnvelope;
pub use trace::{TraceEntry, TraceEvent, WidgetTrace};
pub use transport::{Receipt, Transport};

#[cfg(desktop)]
pub use desktop::Widget;
#[cfg(mobile)]
pub use mobile::Widget;

/// Extension trait for convenient access from any Tauri manager.
pub trait WidgetExt<R: Runtime> {
    /// Returns the managed [`Widget`] state.
    fn widget(&self) -> &Widget<R>;
}

impl<R: Runtime, T: Manager<R>> WidgetExt<R> for T {
    fn widget(&self) -> &Widget<R> {
        self.state::<Widget<R>>().inner()
    }
}

/// Initialize the widgets plugin. Register it with `tauri::Builder::plugin()`.
pub fn init<R: Runtime>() -> TauriPlugin<R, Option<WidgetsPluginConfig>> {
    let builder = Builder::<R, Option<WidgetsPluginConfig>>::new("widgets")
        .invoke_handler(tauri::generate_handler![
            commands::set_items,
            commands::get_items,
            commands::set_register_widget,
            commands::reload_all_timelines,
            commands::reload_timelines,
            commands::request_widget,
            commands::create_widget_window,
            commands::close_widget_window,
            commands::set_widget_config,
            commands::get_widget_config,
            commands::widget_action,
            commands::poll_pending_actions,
            commands::report_receipt,
            commands::get_widget_diagnostics,
            commands::get_widget_trace,
            commands::flush_widget_trace,
        ])
        .setup(|app, api| {
            #[cfg(mobile)]
            let widget = mobile::init(app, api)?;
            #[cfg(desktop)]
            let widget = desktop::init(app, api)?;
            app.manage(widget);
            Ok(())
        });

    #[cfg(mobile)]
    let builder = builder.on_event(|app, event| match event {
        RunEvent::Ready | RunEvent::Resumed => {
            if let Some(widget) = app.try_state::<Widget<R>>() {
                widget.inner().drain_pending_actions_to_events();
            }
        }
        _ => {}
    });

    #[cfg(desktop)]
    let builder =
        builder.register_uri_scheme_protocol(desktop::BUILTIN_PROTOCOL, |_app, _request| {
            const HTML: &[u8] = include_bytes!("../widget.html");
            tauri::http::Response::builder()
                .header("content-type", "text/html; charset=utf-8")
                .body(Cow::Borrowed(HTML))
                .unwrap()
        });

    builder.build()
}
