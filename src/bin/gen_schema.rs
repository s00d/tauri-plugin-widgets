//! Write widget + plugin JSON Schemas from Rust models (schemars).

use schemars::schema_for;
use std::env;
use std::fs;
use std::path::PathBuf;
use tauri_plugin_widgets::config::WidgetsPluginConfig;
use tauri_plugin_widgets::models::WidgetConfig;

fn main() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let schemas = root.join("schemas");
    fs::create_dir_all(&schemas).unwrap();

    let widget = schema_for!(WidgetConfig);
    let widget_path = schemas.join("widget-config.v1.json");
    fs::write(
        &widget_path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&widget).expect("serialize widget schema")
        ),
    )
    .expect("write widget schema");
    eprintln!("wrote {}", widget_path.display());

    let plugin = schema_for!(WidgetsPluginConfig);
    let plugin_path = schemas.join("plugin-config.v1.json");
    fs::write(
        &plugin_path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&plugin).expect("serialize plugin schema")
        ),
    )
    .expect("write plugin schema");
    eprintln!("wrote {}", plugin_path.display());

    // Keep capabilities.json in sync with the matrix (same SoT as markdown).
    let caps = tauri_plugin_widgets::capabilities::render_capabilities_json();
    let caps_path = schemas.join("capabilities.json");
    fs::write(&caps_path, caps).expect("write capabilities.json");
    eprintln!("wrote {}", caps_path.display());
}
