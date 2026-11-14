use tauri::{AppHandle, Emitter, Runtime, State};

use crate::apply::ApplyOutcome;
use crate::error::Error;
use crate::group::{resolve_group, validate_instance_id, validate_widget_id};
use crate::models::{ActivityContent, ActivityHandle, WidgetConfig, WidgetWindowConfig};
use crate::store::{scoped_id, ActiveWidget, WidgetInstanceOption};

#[cfg(desktop)]
use crate::desktop::Widget;
#[cfg(mobile)]
use crate::mobile::Widget;

#[tauri::command]
pub async fn set_items<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    key: String,
    value: String,
    group: Option<String>,
) -> Result<bool, Error> {
    let group = resolve_group(group.as_deref())?;
    widget.set_items(&key, &value, &group)
}

#[tauri::command]
pub async fn get_items<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    key: String,
    group: Option<String>,
) -> Result<Option<String>, Error> {
    let group = resolve_group(group.as_deref())?;
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
    mut config: WidgetWindowConfig,
) -> Result<bool, Error> {
    if let Some(id) = config.widget_id.as_deref() {
        validate_widget_id(id)?;
    }
    validate_instance_id(config.instance.as_deref())?;
    // Built-in renderer reads this group; default it like every other command.
    if config.url.as_deref().map_or(true, str::is_empty) || config.group.is_some() {
        config.group = Some(resolve_group(config.group.as_deref())?);
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
pub async fn set_widget_config<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    config: WidgetConfig,
    group: Option<String>,
    widget_id: String,
    skip_reload: Option<bool>,
    instance: Option<String>,
) -> Result<ApplyOutcome, Error> {
    let group = resolve_group(group.as_deref())?;
    let id = checked_scoped_id(&widget_id, instance.as_deref())?;
    widget.set_widget_config(&config, &group, &id, skip_reload.unwrap_or(false))
}

/// `widgetId` or `widgetId#instance`, both validated.
fn checked_scoped_id(widget_id: &str, instance: Option<&str>) -> Result<String, Error> {
    validate_widget_id(widget_id)?;
    Ok(scoped_id(widget_id, validate_instance_id(instance)?))
}

/// Read and deserialize a [`WidgetConfig`] JSON file from disk.
///
/// `path` must be readable by the app process (absolute path or resolved resource /
/// app-data path). Network URLs are not supported.
pub(crate) fn load_widget_config_from_path(path: &str) -> Result<WidgetConfig, Error> {
    let path = path.trim();
    if path.is_empty() {
        return Err(Error::new(
            "set_widget_config_from_path: path must not be empty",
        ));
    }
    // Regular files only, bounded: `/dev/zero` or a FIFO would otherwise exhaust memory
    // or block forever.
    const MAX_BYTES: u64 = 4 * 1024 * 1024;
    let meta =
        std::fs::metadata(path).map_err(|e| Error::Io(format!("read config from {path}: {e}")))?;
    if !meta.is_file() {
        return Err(Error::Io(format!(
            "read config from {path}: not a regular file"
        )));
    }
    if meta.len() > MAX_BYTES {
        return Err(Error::Io(format!(
            "read config from {path}: {} bytes exceeds the {MAX_BYTES} byte limit",
            meta.len()
        )));
    }
    let raw = std::fs::read_to_string(path)
        .map_err(|e| Error::Io(format!("read config from {path}: {e}")))?;
    serde_json::from_str(&raw)
        .map_err(|e| Error::SerdeJson(format!("parse WidgetConfig from {path}: {e}")))
}

/// Load a [`WidgetConfig`] JSON file from disk, then apply it like [`set_widget_config`].
///
/// `path` must be readable by the app process (absolute path or resolved resource /
/// app-data path). Network URLs are not supported.
#[tauri::command]
pub async fn set_widget_config_from_path<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    path: String,
    group: Option<String>,
    widget_id: String,
    skip_reload: Option<bool>,
    instance: Option<String>,
) -> Result<ApplyOutcome, Error> {
    let group = resolve_group(group.as_deref())?;
    let id = checked_scoped_id(&widget_id, instance.as_deref())?;
    widget.set_widget_config_from_path(&path, &group, &id, skip_reload.unwrap_or(false))
}

#[tauri::command]
pub async fn get_widget_config<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    group: Option<String>,
    widget_id: String,
    instance: Option<String>,
) -> Result<Option<WidgetConfig>, Error> {
    let group = resolve_group(group.as_deref())?;
    let id = checked_scoped_id(&widget_id, instance.as_deref())?;
    widget.get_widget_config(&group, &id)
}

/// Values for `{{data.…}}` without resending the layout. `null` clears them.
#[tauri::command]
pub async fn set_widget_data<R: Runtime>(
    app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    widget_id: String,
    data: serde_json::Value,
    instance: Option<String>,
    group: Option<String>,
    skip_reload: Option<bool>,
) -> Result<bool, Error> {
    let group = resolve_group(group.as_deref())?;
    validate_widget_id(&widget_id)?;
    let instance = validate_instance_id(instance.as_deref())?;
    let key = crate::store::data_key(&widget_id, instance);
    let raw = if data.is_null() {
        String::new()
    } else {
        serde_json::to_string(&data)?
    };
    if raw.len() > crate::resolve::MAX_SOURCE_BYTES {
        return Err(Error::new(format!(
            "widget data is {} bytes; the limit is {}",
            raw.len(),
            crate::resolve::MAX_SOURCE_BYTES
        )));
    }
    widget.set_items(&key, &raw, &group)?;
    widget.data_changed(&group, &widget_id, instance);
    // Open desktop windows redraw from the event; native widgets from the reload.
    let _ = app.emit(
        "widget-data-push",
        serde_json::json!({
            "group": group,
            "widgetId": widget_id,
            "instance": instance,
            "data": data,
        }),
    );
    if !skip_reload.unwrap_or(false) {
        widget.reload_all_timelines()?;
    }
    Ok(true)
}

/// JSON in the store, `None` when missing, empty or not JSON.
#[cfg(desktop)]
fn read_json<R: Runtime>(
    widget: &Widget<R>,
    key: &str,
    group: &str,
) -> Result<Option<serde_json::Value>, Error> {
    Ok(widget
        .get_items(key, group)?
        .filter(|s| !s.is_empty())
        .and_then(|s| serde_json::from_str(&s).ok()))
}

/// The config's `state` defaults (the placement's own config first, then the widget's).
fn state_defaults<R: Runtime>(
    widget: &Widget<R>,
    group: &str,
    widget_id: &str,
    instance: Option<&str>,
) -> Result<Option<serde_json::Value>, Error> {
    let own = match instance {
        Some(_) => widget.get_widget_config(group, &scoped_id(widget_id, instance))?,
        None => None,
    };
    let cfg = match own {
        Some(c) => Some(c),
        None => widget.get_widget_config(group, widget_id)?,
    };
    Ok(cfg.and_then(|c| c.state).map(serde_json::Value::Object))
}

/// Defaults, then `state:{widgetId}`, then `state:{widgetId}#{instance}` from the store.
#[cfg(desktop)]
fn stored_state<R: Runtime>(
    widget: &Widget<R>,
    group: &str,
    widget_id: &str,
    instance: Option<&str>,
    defaults: Option<&serde_json::Value>,
) -> Result<serde_json::Map<String, serde_json::Value>, Error> {
    use crate::store::state_key;
    let shared = read_json(widget, &state_key(widget_id, None), group)?;
    let own = match instance {
        Some(_) => read_json(widget, &state_key(widget_id, instance), group)?,
        None => None,
    };
    Ok(crate::store::merged_state(&[
        defaults,
        shared.as_ref(),
        own.as_ref(),
    ]))
}

/// Change a placement's local state (`{{state.…}}`) like a `set` tap does: operators
/// `$toggle` / `$inc` / `$dec`, `null` removes a key, `patch: null` clears the stored
/// state. Returns the merged state (defaults included) and emits `widget-state-changed`.
#[tauri::command]
pub async fn set_widget_state<R: Runtime>(
    app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    widget_id: String,
    patch: serde_json::Value,
    instance: Option<String>,
    group: Option<String>,
    skip_reload: Option<bool>,
) -> Result<serde_json::Value, Error> {
    let group = resolve_group(group.as_deref())?;
    validate_widget_id(&widget_id)?;
    let instance = validate_instance_id(instance.as_deref())?;
    if !(patch.is_null() || patch.is_object()) {
        return Err(Error::new("widget state patch must be an object or null"));
    }
    if serde_json::to_vec(&patch)?.len() > crate::resolve::MAX_SOURCE_BYTES {
        return Err(Error::new("widget state patch is too large"));
    }
    let defaults = state_defaults(&widget, &group, &widget_id, instance)?;
    #[cfg(desktop)]
    let state = {
        let key = crate::store::state_key(&widget_id, instance);
        let raw = match patch.as_object() {
            None => String::new(),
            Some(p) => {
                let current =
                    stored_state(&widget, &group, &widget_id, instance, defaults.as_ref())?;
                let mut layer = read_json(&widget, &key, &group)?
                    .and_then(|v| v.as_object().cloned())
                    .unwrap_or_default();
                crate::store::apply_state_patch(&mut layer, &current, p);
                serde_json::to_string(&layer)?
            }
        };
        if raw.len() > crate::resolve::MAX_SOURCE_BYTES {
            return Err(Error::new("widget state is too large"));
        }
        widget.set_items(&key, &raw, &group)?;
        stored_state(&widget, &group, &widget_id, instance, defaults.as_ref())?
    };
    // Native writes the layer (same operators) and returns its merged state; merging the
    // defaults again is harmless if it already did.
    #[cfg(mobile)]
    let state = {
        let native = widget.set_widget_state(&group, &widget_id, instance, &patch)?;
        crate::store::merged_state(&[defaults.as_ref(), native.as_ref()])
    };
    let state = serde_json::Value::Object(state);
    emit_state_changed(&app, &group, &widget_id, instance, &state);
    widget.data_changed(&group, &widget_id, instance);
    if !skip_reload.unwrap_or(false) {
        widget.reload_all_timelines()?;
    }
    Ok(state)
}

/// `widget-state-changed` `{group, widgetId, instance, state}` (open desktop windows redraw).
fn emit_state_changed<R: Runtime>(
    app: &AppHandle<R>,
    group: &str,
    widget_id: &str,
    instance: Option<&str>,
    state: &serde_json::Value,
) {
    let _ = app.emit(
        "widget-state-changed",
        serde_json::json!({
            "group": group,
            "widgetId": widget_id,
            "instance": instance,
            "state": state,
        }),
    );
}

/// A placement's merged local state: config `state` defaults, then the stored state.
#[tauri::command]
pub async fn get_widget_state<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    widget_id: String,
    instance: Option<String>,
    group: Option<String>,
) -> Result<serde_json::Value, Error> {
    let group = resolve_group(group.as_deref())?;
    validate_widget_id(&widget_id)?;
    let instance = validate_instance_id(instance.as_deref())?;
    let defaults = state_defaults(&widget, &group, &widget_id, instance)?;
    #[cfg(desktop)]
    let state = stored_state(&widget, &group, &widget_id, instance, defaults.as_ref())?;
    #[cfg(mobile)]
    let state = {
        let native = widget.get_widget_state(&group, &widget_id, instance)?;
        crate::store::merged_state(&[defaults.as_ref(), native.as_ref()])
    };
    Ok(serde_json::Value::Object(state))
}

/// Store a Live Activity layout under `name` (`activity-template:{name}`): activities
/// with `template: name` then send only `data`. `null` removes it.
#[tauri::command]
pub async fn register_activity_template<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    name: String,
    layout: serde_json::Value,
    group: Option<String>,
) -> Result<bool, Error> {
    let group = resolve_group(group.as_deref())?;
    validate_widget_id(&name)?;
    let raw = match &layout {
        serde_json::Value::Null => String::new(),
        serde_json::Value::Object(_) => serde_json::to_string(&layout)?,
        _ => return Err(Error::new("activity template must be an object or null")),
    };
    if raw.len() > crate::resolve::MAX_SOURCE_BYTES {
        return Err(Error::new(format!(
            "activity template is {} bytes; the limit is {}",
            raw.len(),
            crate::resolve::MAX_SOURCE_BYTES
        )));
    }
    widget.set_items(&crate::store::activity_template_key(&name), &raw, &group)
}

/// Fill (or not) and validate what a surface receives. A templated request is checked as
/// sent (only its data counts toward 4 KiB); desktop then resolves the template from the
/// store, mobile passes it through and the native renderer reads the template itself.
fn prepare_activity<R: Runtime>(
    widget: &Widget<R>,
    content: ActivityContent,
    group: &str,
) -> Result<ActivityContent, Error> {
    let content = if content.template.is_some() {
        content
    } else {
        content.filled()
    };
    content.validate()?;
    #[cfg(desktop)]
    let content = if content.template.is_some() {
        crate::models::resolve_activity(content, |k| widget.get_items(k, group).ok().flatten())?
            .filled()
    } else {
        content
    };
    #[cfg(mobile)]
    let _ = (widget, group);
    Ok(content)
}

/// Choices for the per-placement picker (Apple Edit Widget sheet; listed for Android
/// and desktop so the app can label placements).
#[tauri::command]
pub async fn set_widget_instances<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    widget_id: String,
    instances: Vec<WidgetInstanceOption>,
    group: Option<String>,
) -> Result<bool, Error> {
    let group = resolve_group(group.as_deref())?;
    validate_widget_id(&widget_id)?;
    const MAX_INSTANCES: usize = 200;
    if instances.len() > MAX_INSTANCES {
        return Err(Error::new(format!(
            "at most {MAX_INSTANCES} instances per widget"
        )));
    }
    for i in &instances {
        if i.title.len() > 200 {
            return Err(Error::new("instance title is longer than 200 bytes"));
        }
        if validate_instance_id(Some(&i.id))?.is_none() {
            return Err(Error::new("instance id must not be empty"));
        }
    }
    let raw = serde_json::to_string(&instances)?;
    widget.set_items(&crate::store::instances_key(&widget_id), &raw, &group)?;
    widget.reload_all_timelines()
}

/// Widgets placed right now (see [`ActiveWidget`]).
#[tauri::command]
pub async fn get_active_widgets<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    group: Option<String>,
) -> Result<Vec<ActiveWidget>, Error> {
    let group = resolve_group(group.as_deref())?;
    widget.get_active_widgets(&group)
}

/// Start (or update, when the id runs) a live activity.
#[tauri::command]
pub async fn start_activity<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    content: ActivityContent,
    group: Option<String>,
) -> Result<ActivityHandle, Error> {
    let group = resolve_group(group.as_deref())?;
    let content = prepare_activity(&widget, content, &group)?;
    widget.start_activity(content, &group)
}

/// Replace the content of a running live activity.
#[tauri::command]
pub async fn update_activity<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    content: ActivityContent,
    group: Option<String>,
) -> Result<bool, Error> {
    let group = resolve_group(group.as_deref())?;
    let content = prepare_activity(&widget, content, &group)?;
    widget.update_activity(content, &group)
}

/// End a live activity; `dismissAfterMs` keeps the final state visible that long
/// (iOS / desktop; Android removes the notification at once). `content` is the final
/// state; `dismissalPolicy` `"default"` | `"immediate"` (`dismissAfterMs` wins).
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn end_activity<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    id: String,
    dismiss_after_ms: Option<u64>,
    content: Option<ActivityContent>,
    dismissal_policy: Option<String>,
    group: Option<String>,
) -> Result<bool, Error> {
    validate_widget_id(&id)?;
    if let Some(p) = dismissal_policy.as_deref() {
        if !matches!(p, "default" | "immediate") {
            return Err(Error::new(
                "dismissalPolicy must be \"default\" or \"immediate\"",
            ));
        }
    }
    let content = match content {
        Some(c) => {
            if c.id != id {
                return Err(Error::new("final content id must match the activity id"));
            }
            let group = resolve_group(group.as_deref())?;
            Some((prepare_activity(&widget, c, &group)?, group))
        }
        None => None,
    };
    #[cfg(mobile)]
    {
        widget.end_activity_with(
            &id,
            dismiss_after_ms,
            content.as_ref().map(|(c, _)| c),
            dismissal_policy.as_deref(),
        )
    }
    // Desktop: show the final content, then close (immediate = no delay).
    #[cfg(desktop)]
    {
        if let Some((c, group)) = content {
            let _ = widget.update_activity(c, &group);
        }
        let delay =
            dismiss_after_ms.or((dismissal_policy.as_deref() == Some("immediate")).then_some(0));
        widget.end_activity(&id, delay)
    }
}

/// WidgetKit push token (hex) for server-driven widget reloads; `null` where none exists.
#[tauri::command]
pub async fn get_widget_push_token<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    group: Option<String>,
) -> Result<Option<String>, Error> {
    let group = resolve_group(group.as_deref())?;
    #[cfg(mobile)]
    {
        widget.widget_push_token(&group)
    }
    #[cfg(desktop)]
    {
        let _ = (widget, group);
        Ok(None)
    }
}

/// iOS 17.2+ push-to-start token (hex) for your APNs server; `null` elsewhere / not yet issued.
#[tauri::command]
pub async fn get_activity_push_to_start_token<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
) -> Result<Option<String>, Error> {
    widget.activity_push_to_start_token()
}

/// Android 16+: whether ongoing activity notifications may be promoted to Live Updates
/// (status-bar chip, lock screen). `true` where the question does not apply.
#[tauri::command]
pub async fn can_promote_activities<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
) -> Result<bool, Error> {
    widget.can_promote_activities()
}

/// Android: opens the system setting that allows Live Updates for this app. `false` when
/// there is no such screen (other platforms, older Android).
#[tauri::command]
pub async fn open_activity_promotion_settings<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
) -> Result<bool, Error> {
    widget.open_activity_promotion_settings()
}

/// Ids of the running live activities this app started.
#[tauri::command]
pub async fn list_activities<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
) -> Result<Vec<String>, Error> {
    widget.list_activities()
}

/// Desktop activity island page → host: `expanded` grows / shrinks the island (hover,
/// click), `focus` brings a minimal activity to the pill. Returns what the island shows
/// (`null` on mobile or without an island). Used by the built-in renderer.
#[tauri::command]
pub async fn activity_island<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    expanded: Option<bool>,
    focus: Option<String>,
) -> Result<Option<serde_json::Value>, Error> {
    #[cfg(desktop)]
    {
        widget.activity_island(expanded, focus.as_deref())
    }
    #[cfg(mobile)]
    {
        let _ = (widget, expanded, focus);
        Ok(None)
    }
}

#[tauri::command]
pub fn widget_action<R: Runtime>(
    app: AppHandle<R>,
    action: String,
    payload: Option<String>,
    widget_id: Option<String>,
    group: Option<String>,
) -> Result<bool, Error> {
    let group = resolve_group(group.as_deref()).unwrap_or_default();
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
pub async fn poll_pending_actions<R: Runtime>(
    app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    group: Option<String>,
) -> Result<Vec<crate::WidgetActionEnvelope>, Error> {
    // JavaScript listens even if this poll fails (e.g. no App Group configured yet).
    widget.mark_listener_ready();
    let group = resolve_group(group.as_deref())?;
    // `$state` envelopes (a `set` tap in a native widget) go out as `widget-state-changed`
    // (mobile already routes them while polling).
    let mut actions = widget.poll_pending_actions(&group)?;
    actions.retain(|env| match crate::store::state_event(env) {
        Some(ev) => {
            let _ = app.emit("widget-state-changed", ev);
            false
        }
        None => true,
    });
    Ok(actions)
}

#[tauri::command]
pub fn report_receipt<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    mut receipt: crate::receipt::WidgetRenderReceipt,
) -> Result<bool, Error> {
    // The group feeds the macOS action poller: only the app's own group is accepted.
    receipt.group = resolve_group(Some(&receipt.group))?;
    widget.report_receipt(receipt)
}

#[tauri::command]
pub fn get_widget_diagnostics<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    group: Option<String>,
) -> Result<Vec<crate::receipt::WidgetRenderReceipt>, Error> {
    let group = resolve_group(group.as_deref())?;
    widget.get_widget_diagnostics(&group)
}

#[tauri::command]
pub fn get_widget_trace<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
    group: Option<String>,
    since_ms: Option<u64>,
) -> Result<crate::trace::WidgetTrace, Error> {
    let group = resolve_group(group.as_deref())?;
    widget.get_widget_trace(&group, since_ms)
}

#[tauri::command]
pub fn flush_widget_trace<R: Runtime>(
    _app: AppHandle<R>,
    widget: State<'_, Widget<R>>,
) -> Result<bool, Error> {
    widget.flush_widget_trace()
}
