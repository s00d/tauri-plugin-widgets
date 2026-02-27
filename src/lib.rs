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
//! ```rust,ignore
//! fn main() {
//!     tauri::Builder::default()
//!         .plugin(tauri_plugin_widgets::init())
//!         .run(tauri::generate_context!())
//!         .expect("error while running tauri application");
//! }
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
//! 4. `build-widget.sh` is called automatically via `beforeBundleCommand`
//! 5. After `tauri build`, run `embed-widget.sh` to copy the `.appex`
//!    into `Contents/PlugIns/` and re-sign
//!
//! ## Rust API
//!
//! Access widget methods from Rust via [`WidgetExt`]:
//!
//! ```rust,ignore
//! use tauri::Manager;
//! use tauri_plugin_widgets::WidgetExt;
//!
//! fn update(app: &tauri::AppHandle) {
//!     let w = app.widget();
//!     w.set_items("key", "value", "group.com.example.myapp").unwrap();
//!     w.reload_all_timelines().unwrap();
//! }
//! ```

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

#[cfg(desktop)]
use std::borrow::Cow;

#[cfg(desktop)]
pub mod desktop;
#[cfg(mobile)]
pub mod mobile;

mod commands;
pub mod error;
pub mod models;

pub use error::{Error, Result};

#[cfg(desktop)]
pub use desktop::Widget;
#[cfg(mobile)]
pub use mobile::Widget;

/// Extension trait for convenient access from any Tauri manager.
pub trait WidgetExt<R: Runtime> {
    fn widget(&self) -> &Widget<R>;
}

impl<R: Runtime, T: Manager<R>> WidgetExt<R> for T {
    fn widget(&self) -> &Widget<R> {
        self.state::<Widget<R>>().inner()
    }
}

/// Initialize the widgets plugin. Register it with `tauri::Builder::plugin()`.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    let builder = Builder::new("widgets")
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
        ])
        .setup(|app, api| {
            #[cfg(mobile)]
            let widget = mobile::init(app, api)?;
            #[cfg(desktop)]
            let widget = desktop::init(app, api)?;
            app.manage(widget);
            Ok(())
        });

    #[cfg(desktop)]
    let builder = builder.register_uri_scheme_protocol(
        desktop::BUILTIN_PROTOCOL,
        |_app, _request| {
            const HTML: &[u8] = include_bytes!("../widget.html");
            tauri::http::Response::builder()
                .header("content-type", "text/html; charset=utf-8")
                .body(Cow::Borrowed(HTML))
                .unwrap()
        },
    );

    builder.build()
}
