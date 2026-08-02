use tauri::{AppHandle, Emitter, Runtime, State};

use crate::apply::ApplyOutcome;
use crate::error::Error;
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
    widget.set_items(&key, &value, &group)
}

#[tauri::command]
pub fn get_items<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    key: String,
    group: String,
) -> Result<Option<String>, Error> {
    widget.get_items(&key, &group)
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
    widget.set_widget_config(&config, &group, &widget_id, skip_reload.unwrap_or(false))
}

#[tauri::command]
pub fn get_widget_config<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    group: String,
    widget_id: String,
) -> Result<Option<WidgetConfig>, Error> {
    widget.get_widget_config(&group, &widget_id)
}

#[tauri::command]
pub fn widget_action<R: Runtime>(
    app: AppHandle<R>,
    action: String,
    payload: Option<String>,
    widget_id: Option<String>,
    group: Option<String>,
) -> Result<bool, Error> {
    let data = serde_json::json!({
        "action": action,
        "payload": payload,
        "ts": crate::store::now_ms(),
        "widgetId": widget_id.unwrap_or_default(),
        "group": group.unwrap_or_default(),
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
    widget.poll_pending_actions(&group)
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
    widget.get_widget_diagnostics(&group)
}

#[tauri::command]
pub fn get_widget_trace<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    group: String,
    since_ms: Option<u64>,
) -> Result<crate::trace::WidgetTrace, Error> {
    widget.get_widget_trace(&group, since_ms)
}

#[tauri::command]
pub fn flush_widget_trace<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
) -> Result<bool, Error> {
    widget.flush_widget_trace()
}
