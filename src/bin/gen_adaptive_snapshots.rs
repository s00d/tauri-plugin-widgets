//! Dump Adaptive Card JSON snapshots for `tests/cases` → `tests/snapshots/adaptive/`.
//!
//! ```bash
//! cargo run --bin gen-adaptive-snapshots
//! CASE=weather.small cargo run --bin gen-adaptive-snapshots
//! # chart/canvas/gauge data-URIs need: --features rasterize
//! ```

use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use tauri_plugin_widgets::adaptive_card::to_adaptive_card_for_size;
use tauri_plugin_widgets::models::WidgetConfig;

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cases_dir = root.join("tests/cases");
    let fixtures = root.join("tests/fixtures");
    let ac_dir = root.join("tests/snapshots/adaptive");
    fs::create_dir_all(&ac_dir).unwrap();

    let only = std::env::var("CASE").ok().filter(|s| !s.is_empty());
    let mut entries: Vec<_> = fs::read_dir(&cases_dir)
        .expect("tests/cases")
        .flatten()
        .collect();
    entries.sort_by_key(|e| e.file_name());

    let mut wrote = 0usize;
    for entry in entries {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        if let Some(ref only) = only {
            if only != &name {
                continue;
            }
        }

        let case: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let fixture = case["fixture"].as_str().unwrap_or("");
        let size = case["size"].as_str().unwrap_or("small");
        let cfg_path = fixtures.join(format!("{fixture}.json"));
        if !cfg_path.exists() {
            eprintln!("skip {name}: missing fixture {fixture}");
            continue;
        }
        let raw = fs::read_to_string(&cfg_path).unwrap();
        let v: Value = serde_json::from_str(&raw).unwrap();
        let cfg: WidgetConfig = match serde_json::from_value(v) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("skip {name}: parse {e}");
                continue;
            }
        };
        let Some(result) = to_adaptive_card_for_size(&cfg, size) else {
            eprintln!("skip {name}: no layout for size={size}");
            continue;
        };

        let dest = ac_dir.join(format!("{name}.json"));
        let pretty = serde_json::to_string_pretty(&result.card).unwrap();
        fs::write(&dest, pretty).unwrap();
        println!("ac {}", dest.display());
        wrote += 1;
    }
    println!("wrote {wrote} adaptive snapshots → {}", ac_dir.display());
}
