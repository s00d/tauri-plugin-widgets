//! Open exactly one widget window from a fixture JSON, or watch an inbox path.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use tauri_plugin_widgets::models::{WidgetConfig, WidgetWindowConfig};
use tauri_plugin_widgets::WidgetExt;

const LABEL: &str = "widget-probe";
const GROUP: &str = "probe.group";
const WIDGET_ID: &str = "probe";

fn size_px(size: &str) -> (f64, f64) {
    match size {
        "medium" => (338.0, 158.0),
        "large" => (338.0, 354.0),
        _ => (158.0, 158.0),
    }
}

fn load_config(path: &Path) -> Result<WidgetConfig, String> {
    let raw = std::fs::read_to_string(path).map_err(|e| format!("read {path:?}: {e}"))?;
    serde_json::from_str(&raw).map_err(|e| format!("parse {path:?}: {e}"))
}

fn open_widget(app: &tauri::AppHandle, cfg: &WidgetConfig, size: &str) -> Result<(), String> {
    let w = app.widget();
    w.set_widget_config(cfg, GROUP, WIDGET_ID, true)
        .map_err(|e| e.to_string())?;

    let (width, height) = size_px(size);
    // create_widget_window closes any prior label on the GTK main thread.
    w.create_widget_window(WidgetWindowConfig {
        label: LABEL.into(),
        url: None,
        width,
        height,
        x: Some(40.0),
        y: Some(40.0),
        always_on_top: false,
        skip_taskbar: true,
        group: Some(GROUP.into()),
        widget_id: Some(WIDGET_ID.into()),
        size: Some(size.to_string()),
    })
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn parse_args() -> (Mode, String) {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let size_default = "small".to_string();

    if args.first().map(|s| s.as_str()) == Some("--watch") {
        let path = args
            .get(1)
            .cloned()
            .unwrap_or_else(|| "/work/out/inbox/linux.json".into());
        let size = args.get(2).cloned().unwrap_or(size_default);
        return (Mode::Watch(PathBuf::from(path)), size);
    }

    let fixture = args
        .first()
        .cloned()
        .unwrap_or_else(|| panic!("usage: widget-probe <fixture.json> [size] | --watch <inbox.json> [size]"));
    let size = args.get(1).cloned().unwrap_or(size_default);
    (Mode::Once(PathBuf::from(fixture)), size)
}

enum Mode {
    Once(PathBuf),
    Watch(PathBuf),
}

fn main() {
    let (mode, size) = parse_args();
    let size = Arc::new(size);

    tauri::Builder::default()
        .plugin(tauri_plugin_widgets::init())
        .setup(move |app| {
            let handle = app.handle().clone();
            let size_c = Arc::clone(&size);

            match mode {
                Mode::Once(path) => {
                    let cfg = load_config(&path).expect("fixture");
                    open_widget(&handle, &cfg, &size_c).expect("open widget");
                }
                Mode::Watch(path) => {
                    let last = Arc::new(Mutex::new((SystemTime::UNIX_EPOCH, String::new())));
                    if let Some(parent) = path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    let size_file = path.with_extension("size");
                    std::thread::spawn(move || loop {
                        std::thread::sleep(Duration::from_millis(400));
                        let meta = match std::fs::metadata(&path) {
                            Ok(m) => m,
                            Err(_) => continue,
                        };
                        let modified = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                        let body = match std::fs::read_to_string(&path) {
                            Ok(s) if !s.trim().is_empty() => s,
                            _ => continue,
                        };
                        {
                            let mut guard = last.lock().unwrap();
                            if guard.0 == modified && guard.1 == body {
                                continue;
                            }
                            *guard = (modified, body.clone());
                        }
                        let size_now = std::fs::read_to_string(&size_file)
                            .ok()
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .unwrap_or_else(|| size_c.as_str().to_string());
                        match serde_json::from_str::<WidgetConfig>(&body) {
                            Ok(cfg) => {
                                if let Err(e) = open_widget(&handle, &cfg, &size_now) {
                                    eprintln!("widget-probe watch: {e}");
                                } else {
                                    eprintln!(
                                        "widget-probe watch: reloaded {} ({size_now})",
                                        path.display()
                                    );
                                }
                            }
                            Err(e) => eprintln!("widget-probe watch parse: {e}"),
                        }
                    });
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running widget-probe");
}
