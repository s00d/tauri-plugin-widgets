//! Emit `guest-js/generated/widget-types.ts` from the Rust SoT template.

use std::env;
use std::fs;
use std::path::PathBuf;
use tauri_plugin_widgets::codegen::emit_widget_types_ts;

fn main() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out = root.join("guest-js/generated/widget-types.ts");
    fs::create_dir_all(out.parent().unwrap()).unwrap();
    let ts = emit_widget_types_ts();
    fs::write(&out, ts).expect("write widget-types.ts");
    eprintln!("wrote {}", out.display());
}
