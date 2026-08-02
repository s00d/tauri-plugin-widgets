//! Write `schemas/widget-config.v1.json` from Rust models (schemars).

use schemars::schema_for;
use std::env;
use std::fs;
use std::path::PathBuf;
use tauri_plugin_widgets::models::WidgetConfig;

fn main() {
    let schema = schema_for!(WidgetConfig);
    let json = serde_json::to_string_pretty(&schema).expect("serialize schema");
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out = root.join("schemas/widget-config.v1.json");
    fs::create_dir_all(out.parent().unwrap()).unwrap();
    fs::write(&out, format!("{json}\n")).expect("write schema");
    eprintln!("wrote {}", out.display());
}
