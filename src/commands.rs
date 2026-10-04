use tauri::{AppHandle, Emitter, Runtime, State};

use crate::apply::ApplyOutcome;
use crate::error::Error;
use crate::group::validate_group;
use crate::models::{WidgetConfig, WidgetWindowConfig};

#[cfg(desktop)]
use crate::desktop::Widget;
#[cfg(mobile)]
use crate::mobile::Widget;

#[tauri::command]
pub fn set_items<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    key: String,
    value: String,
    group: String,
) -> Result<bool, Error> {
    let group = validate_group(&group)?;
    widget.set_items(&key, &value, group)
}

#[tauri::command]
pub fn get_items<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    key: String,
    group: String,
) -> Result<Option<String>, Error> {
    let group = validate_group(&group)?;
    widget.get_items(&key, group)
}

#[tauri::command]
pub fn set_register_widget<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    widgets: Vec<String>,
) -> Result<bool, Error> {
    widget.set_register_widget(widgets)
}

#[tauri::command]
pub fn reload_all_timelines<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
) -> Result<bool, Error> {
    widget.reload_all_timelines()
}

#[tauri::command]
pub fn reload_timelines<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    of_kind: String,
) -> Result<bool, Error> {
    widget.reload_timelines(&of_kind)
}

#[tauri::command]
pub fn request_widget<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
) -> Result<bool, Error> {
    widget.request_widget()
}

#[tauri::command]
pub async fn create_widget_window<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    config: WidgetWindowConfig,
) -> Result<bool, Error> {
    if let Some(ref group) = config.group {
        validate_group(group)?;
    }
    widget.create_widget_window(config)
}

#[tauri::command]
pub async fn close_widget_window<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    label: String,
) -> Result<bool, Error> {
    widget.close_widget_window(&label)
}

#[tauri::command]
pub fn set_widget_config<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    config: WidgetConfig,
    group: String,
    widget_id: String,
    skip_reload: Option<bool>,
) -> Result<ApplyOutcome, Error> {
    let group = validate_group(&group)?;
    widget.set_widget_config(&config, group, &widget_id, skip_reload.unwrap_or(false))
}

/// Read and deserialize a [`WidgetConfig`] JSON file from disk.
///
/// `path` must be readable by the app process (absolute path or resolved resource /
/// app-data path). Network URLs are not supported.
pub(crate) fn load_widget_config_from_path(path: &str) -> Result<WidgetConfig, Error> {
    let path = path.trim();
    if path.is_empty() {
        return Err(Error::new("set_widget_config_from_path: path must not be empty"));
    }
    let raw = std::fs::read_to_string(path).map_err(|e| {
        Error::Io(format!("read config from {path}: {e}"))
    })?;
    serde_json::from_str(&raw).map_err(|e| {
        Error::SerdeJson(format!("parse WidgetConfig from {path}: {e}"))
    })
}

/// Load a [`WidgetConfig`] JSON file from disk, then apply it like [`set_widget_config`].
///
/// `path` must be readable by the app process (absolute path or resolved resource /
/// app-data path). Network URLs are not supported.
#[tauri::command]
pub fn set_widget_config_from_path<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    path: String,
    group: String,
    widget_id: String,
    skip_reload: Option<bool>,
) -> Result<ApplyOutcome, Error> {
    let group = validate_group(&group)?;
    widget.set_widget_config_from_path(&path, group, &widget_id, skip_reload.unwrap_or(false))
}

#[tauri::command]
pub fn get_widget_config<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    group: String,
    widget_id: String,
) -> Result<Option<WidgetConfig>, Error> {
    let group = validate_group(&group)?;
    widget.get_widget_config(group, &widget_id)
}

#[tauri::command]
pub fn widget_action<R: Runtime>(
    app: AppHandle<R>,
    action: String,
    payload: Option<String>,
    widget_id: Option<String>,
    group: Option<String>,
) -> Result<bool, Error> {
    let group = match group.as_deref() {
        Some(g) if !g.is_empty() => validate_group(g)?.to_string(),
        _ => String::new(),
    };
    let data = serde_json::json!({
        "action": action,
        "payload": payload,
        "ts": crate::store::now_ms(),
        "widgetId": widget_id.unwrap_or_default(),
        "group": group,
    });
    app.emit("widget-action", data)
        .map_err(|e| Error::new(format!("emit widget-action: {e}")))?;
    Ok(true)
}

#[tauri::command]
pub fn poll_pending_actions<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    group: String,
) -> Result<Vec<crate::WidgetActionEnvelope>, Error> {
    let group = validate_group(&group)?;
    widget.poll_pending_actions(group)
}

#[tauri::command]
pub fn report_receipt<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    receipt: crate::receipt::WidgetRenderReceipt,
) -> Result<bool, Error> {
    widget.report_receipt(receipt)
}

#[tauri::command]
pub fn get_widget_diagnostics<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    group: String,
) -> Result<Vec<crate::receipt::WidgetRenderReceipt>, Error> {
    let group = validate_group(&group)?;
    widget.get_widget_diagnostics(group)
}

#[tauri::command]
pub fn get_widget_trace<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    group: String,
    since_ms: Option<u64>,
) -> Result<crate::trace::WidgetTrace, Error> {
    let group = validate_group(&group)?;
    widget.get_widget_trace(group, since_ms)
}

#[tauri::command]
pub fn flush_widget_trace<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
) -> Result<bool, Error> {
    widget.flush_widget_trace()
}

#[cfg(test)]
mod from_path_tests {
    use super::load_widget_config_from_path;
    use std::path::PathBuf;

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/presets")
            .join(name)
    }

    #[test]
    fn loads_weather_fixture() {
        let cfg = load_widget_config_from_path(fixture("weather.json").to_str().unwrap())
            .expect("weather.json parses");
        assert!(cfg.small.is_some());
    }

    #[test]
    fn loads_stress_nest_keeps_list_children() {
        use crate::models::WidgetElement;

        fn find_list_children(el: &WidgetElement) -> Option<usize> {
            match el {
                WidgetElement::List(list) => Some(list.children.len()),
                WidgetElement::VStack(v) => v.children.iter().find_map(find_list_children),
                WidgetElement::HStack(v) => v.children.iter().find_map(find_list_children),
                WidgetElement::ZStack(v) => v.children.iter().find_map(find_list_children),
                WidgetElement::Grid(v) => v.children.iter().find_map(find_list_children),
                WidgetElement::Container(v) => v.children.iter().find_map(find_list_children),
                WidgetElement::Link(v) => v.children.iter().find_map(find_list_children),
                _ => None,
            }
        }

        let cfg = load_widget_config_from_path(fixture("stress-nest.json").to_str().unwrap())
            .expect("stress-nest.json parses");
        let root = cfg.large.expect("large root");
        let n = find_list_children(&root).expect("list node present");
        assert_eq!(n, 3, "rich list children must survive from_path deserialize");
    }

    #[test]
    fn rejects_empty_path() {
        assert!(load_widget_config_from_path("").is_err());
        assert!(load_widget_config_from_path("   ").is_err());
    }

    #[test]
    fn rejects_missing_file() {
        let err = load_widget_config_from_path("/no/such/widget-config-xyz.json").unwrap_err();
        assert!(err.to_string().contains("IO error") || err.to_string().contains("read config"));
    }
}
