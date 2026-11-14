use serde::Serialize;
use serde_json::Value;
use std::collections::HashSet;
use std::env;
use std::sync::Mutex;
use std::time::Instant;
use tauri::{plugin::PluginApi, AppHandle, Emitter, Manager, Runtime};

use crate::apply::{config_content_hash, ApplyOutcome, ReloadOutcome};
use crate::config::WidgetsPluginConfig;
use crate::models::{WidgetConfig, WidgetWindowConfig};
use crate::receipt::{ReceiptStore, WidgetRenderReceipt};
use crate::store::config_key;
use crate::trace::{TraceEvent, TraceStore, WidgetTrace};
use crate::transport::validate_mobile_transport;

/// Default minimum interval between plugin-triggered reloads (**iOS/Android host only**): none.
///
/// Commands come from the running app, and WidgetKit does not charge reloads while the
/// containing app is in the foreground; Android has no reload budget at all. A default
/// throttle only left widgets stale for minutes after a tap. Set
/// `TAURI_WIDGET_MIN_RELOAD_SECS` to throttle background-driven updates yourself.
const DEFAULT_RELOAD_MIN_INTERVAL_SECS: u64 = 0;

fn reload_min_interval_secs() -> u64 {
    match env::var("TAURI_WIDGET_MIN_RELOAD_SECS") {
        Ok(v) => v
            .trim()
            .parse::<u64>()
            .unwrap_or(DEFAULT_RELOAD_MIN_INTERVAL_SECS),
        Err(_) => DEFAULT_RELOAD_MIN_INTERVAL_SECS,
    }
}

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "git.s00d.widgets";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_widgets);

pub fn init<R: Runtime>(
    app: &AppHandle<R>,
    api: PluginApi<R, Option<WidgetsPluginConfig>>,
) -> crate::Result<Widget<R>> {
    let cfg = api
        .config()
        .clone()
        .unwrap_or_default()
        .with_defaults(&app.config().identifier);
    let _ = validate_mobile_transport(&cfg)?;

    #[cfg(target_os = "android")]
    let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "WidgetBridgePlugin")?;
    #[cfg(target_os = "ios")]
    let handle = api.register_ios_plugin(init_plugin_widgets)?;

    Ok(Widget {
        app: app.clone(),
        handle,
        last_reload: Mutex::new(None),
        known_groups: Mutex::new(HashSet::new()),
        receipts: ReceiptStore::new(),
        trace: TraceStore::new(),
        listener_gate: crate::store::ListenerGate::new(
            crate::store::now_ms(),
            crate::store::ListenerGate::DEFAULT_FALLBACK_MS,
        ),
        fallback_drain_scheduled: std::sync::atomic::AtomicBool::new(false),
    })
}

// ── Payloads ────────────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SetItemPayload<'a> {
    key: &'a str,
    value: &'a str,
    group: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetItemsPayload<'a> {
    key: &'a str,
    group: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RegisterPayload {
    widgets: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ReloadPayload<'a> {
    of_kind: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GroupPayload<'a> {
    group: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SetWidgetConfigPayload<'a> {
    config: &'a str,
    group: &'a str,
    widget_id: &'a str,
}

// ── Widget ──────────────────────────────────────────────────────────────────

pub struct Widget<R: Runtime> {
    app: AppHandle<R>,
    handle: tauri::plugin::PluginHandle<R>,
    last_reload: Mutex<Option<Instant>>,
    known_groups: Mutex<HashSet<String>>,
    receipts: ReceiptStore,
    trace: TraceStore,
    /// Hold queued taps until JavaScript listens (see [`crate::store::ListenerGate`]).
    listener_gate: crate::store::ListenerGate,
    /// One delayed drain for the gate fallback.
    fallback_drain_scheduled: std::sync::atomic::AtomicBool,
}

impl<R: Runtime> Widget<R> {
    fn remember_group(&self, group: &str) {
        if group.is_empty() {
            return;
        }
        self.known_groups.lock().unwrap().insert(group.to_string());
    }

    /// Drain pending actions for all known groups and emit `widget-action`.
    pub fn drain_pending_actions_to_events(&self) {
        let now = crate::store::now_ms();
        if !self.listener_gate.is_open(now) {
            // Cold start: drain once when the fallback opens the gate, for apps that only
            // `listen("widget-action")` (onWidgetAction drains earlier by itself).
            if !self
                .fallback_drain_scheduled
                .swap(true, std::sync::atomic::Ordering::SeqCst)
            {
                let app = self.app.clone();
                let wait = self.listener_gate.remaining_ms(now);
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(wait + 10));
                    if let Some(w) = app.try_state::<Widget<R>>() {
                        w.inner().drain_pending_actions_to_events();
                    }
                });
            }
            return;
        }
        let groups: Vec<String> = self.known_groups.lock().unwrap().iter().cloned().collect();
        for group in groups {
            match self.poll_pending_actions(&group) {
                Ok(actions) => {
                    for action in actions {
                        let _ = self.app.emit("widget-action", action);
                    }
                }
                Err(e) => {
                    log::debug!("poll_pending_actions({group}) failed: {e}");
                }
            }
        }
    }

    pub fn set_items(&self, key: &str, value: &str, group: &str) -> crate::Result<bool> {
        self.remember_group(group);
        self.handle
            .run_mobile_plugin("setItems", SetItemPayload { key, value, group })
            .map(|_: Value| true)
            .map_err(Into::into)
    }

    pub fn get_items(&self, key: &str, group: &str) -> crate::Result<Option<String>> {
        let res: Value = self
            .handle
            .run_mobile_plugin("getItems", GetItemsPayload { key, group })?;
        Ok(res
            .get("results")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()))
    }

    /// Register native widget provider ids.
    ///
    /// | Platform | Behaviour |
    /// |---|---|
    /// | Android | stores fully-qualified provider class names |
    /// | iOS / macOS | stores WidgetKit kind strings (advisory) |
    /// | Desktop | **no-op**, accepted for API symmetry |
    pub fn set_register_widget(&self, widgets: Vec<String>) -> crate::Result<bool> {
        if widgets.is_empty() {
            return Err(crate::Error::new(
                "set_register_widget: widgets must be a non-empty array",
            ));
        }
        self.handle
            .run_mobile_plugin("setRegisterWidget", RegisterPayload { widgets })
            .map(|_: Value| true)
            .map_err(Into::into)
    }

    pub fn reload_all_timelines(&self) -> crate::Result<bool> {
        self.handle
            .run_mobile_plugin("reloadAllTimelines", ())
            .map(|_: Value| true)
            .map_err(Into::into)
    }

    /// Rate-limited reload: skips the actual WidgetKit call if the last
    /// reload happened less than `reload_min_interval_secs()` ago.
    ///
    /// Never returns a silent “success” — callers must surface [`ReloadOutcome`].
    fn throttled_reload(&self) -> crate::Result<ReloadOutcome> {
        let min_interval = reload_min_interval_secs();
        if min_interval == 0 {
            return match self.reload_all_timelines() {
                Ok(_) => Ok(ReloadOutcome::Ok),
                Err(e) => Ok(ReloadOutcome::Failed {
                    error: e.to_string(),
                }),
            };
        }

        let mut last = self.last_reload.lock().unwrap();
        let now = Instant::now();
        if let Some(prev) = *last {
            let elapsed = now.duration_since(prev).as_secs();
            if elapsed < min_interval {
                return Ok(ReloadOutcome::Throttled {
                    remaining_secs: crate::apply::throttle_remaining_secs(elapsed, min_interval)
                        .unwrap_or(0),
                });
            }
        }
        drop(last);
        match self.reload_all_timelines() {
            Ok(_) => {
                *self.last_reload.lock().unwrap() = Some(now);
                Ok(ReloadOutcome::Ok)
            }
            Err(e) => Ok(ReloadOutcome::Failed {
                error: e.to_string(),
            }),
        }
    }

    pub fn reload_timelines(&self, of_kind: &str) -> crate::Result<bool> {
        self.handle
            .run_mobile_plugin("reloadTimelines", ReloadPayload { of_kind })
            .map(|_: Value| true)
            .map_err(Into::into)
    }

    /// Request that the OS show the "add widget" / pin UI.
    ///
    /// | Platform | Behaviour |
    /// |---|---|
    /// | Android | opens the pin-widget flow |
    /// | iOS / macOS | bridge call (may be a no-op on the OS side) |
    /// | Desktop | **error** — use `create_widget_window` instead |
    pub fn request_widget(&self) -> crate::Result<bool> {
        self.handle
            .run_mobile_plugin("requestWidget", ())
            .map(|_: Value| true)
            .map_err(Into::into)
    }

    pub fn create_widget_window(&self, _cfg: WidgetWindowConfig) -> crate::Result<bool> {
        Err(crate::Error::Unsupported(
            "Webview widgets are desktop only".into(),
        ))
    }

    pub fn close_widget_window(&self, _label: &str) -> crate::Result<bool> {
        Err(crate::Error::Unsupported(
            "Webview widgets are desktop only".into(),
        ))
    }

    /// Load a [`WidgetConfig`] JSON file from disk, then apply it like [`Self::set_widget_config`].
    pub fn set_widget_config_from_path(
        &self,
        path: impl AsRef<std::path::Path>,
        group: &str,
        widget_id: &str,
        skip_reload: bool,
    ) -> crate::Result<ApplyOutcome> {
        if widget_id.is_empty() {
            return Err(crate::Error::new("widget_id must not be empty"));
        }
        let path = path.as_ref();
        let path_str = path
            .to_str()
            .ok_or_else(|| crate::Error::new(format!("invalid UTF-8 path: {}", path.display())))?;
        let config = crate::commands::load_widget_config_from_path(path_str)?;
        self.set_widget_config(&config, group, widget_id, skip_reload)
    }

    pub fn set_widget_config(
        &self,
        config: &WidgetConfig,
        group: &str,
        widget_id: &str,
        skip_reload: bool,
    ) -> crate::Result<ApplyOutcome> {
        if widget_id.is_empty() {
            return Err(crate::Error::new("widget_id must not be empty"));
        }
        self.remember_group(group);

        let mut config =
            crate::normalize::normalize(config, crate::capabilities::WidgetPlatform::current())
                .config;
        crate::image_prefetch::prefetch_remote_images(&mut config);

        let json = serde_json::to_string(&config)
            .map_err(|e| crate::Error::new(format!("serialize config: {e}")))?;
        let hash = config_content_hash(&json);
        let key = config_key(widget_id);

        let existing = self.get_items(&key, group)?;
        let changed = existing.as_deref() != Some(json.as_str());

        if !changed {
            let outcome = ApplyOutcome::unchanged(hash);
            self.trace.push(TraceEvent::ConfigSet {
                widget_id: widget_id.into(),
                nonce: 0,
                bytes: json.len(),
                changed: false,
                skip: Some(crate::apply::SkipReason::Unchanged { hash }),
            });
            self.trace.push(TraceEvent::Reload {
                performed: false,
                reason: outcome.reload.clone(),
            });
            return Ok(outcome);
        }

        crate::capabilities::log_capabilities(&config);

        // Prefer native setWidgetConfig (Android image preprocess + Glance sync).
        // Falls back to set_items with config:{widgetId} key.
        let t0 = Instant::now();
        let native_ok: Result<Value, _> = self.handle.run_mobile_plugin(
            "setWidgetConfig",
            SetWidgetConfigPayload {
                config: &json,
                group,
                widget_id,
            },
        );
        let write_ms = t0.elapsed().as_millis() as u32;

        let transports: Vec<String> = match &native_ok {
            Ok(_) => vec!["native".into()],
            Err(_) => {
                self.set_items(&key, &json, group)?;
                vec!["items".into()]
            }
        };
        for name in &transports {
            self.trace.push(TraceEvent::Write {
                transport: name.clone(),
                ok: true,
                duration_ms: write_ms,
                error: None,
            });
        }

        let reload = if skip_reload {
            ReloadOutcome::Skipped {
                why: "skip_reload".into(),
            }
        } else {
            self.throttled_reload()?
        };

        self.trace.push(TraceEvent::ConfigSet {
            widget_id: widget_id.into(),
            nonce: 0,
            bytes: json.len(),
            changed: true,
            skip: None,
        });
        self.trace.push(TraceEvent::Reload {
            performed: matches!(reload, ReloadOutcome::Ok),
            reason: reload.clone(),
        });

        Ok(ApplyOutcome {
            written: true,
            reload,
            transports,
            skip: None,
        })
    }

    pub fn get_widget_config(
        &self,
        group: &str,
        widget_id: &str,
    ) -> crate::Result<Option<WidgetConfig>> {
        if widget_id.is_empty() {
            return Err(crate::Error::new("widget_id must not be empty"));
        }
        // Try native getWidgetConfig first (Android), then key lookup.
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct GetCfg<'a> {
            group: &'a str,
            widget_id: &'a str,
        }
        if let Ok(res) = self
            .handle
            .run_mobile_plugin::<Value>("getWidgetConfig", GetCfg { group, widget_id })
        {
            if let Some(s) = res.get("results").and_then(|v| v.as_str()) {
                let config: WidgetConfig = serde_json::from_str(s)
                    .map_err(|e| crate::Error::new(format!("parse config: {e}")))?;
                return Ok(Some(config));
            }
            if res.get("results").map(|v| v.is_null()).unwrap_or(false) {
                return Ok(None);
            }
        }

        match self.get_items(&config_key(widget_id), group)? {
            Some(json) => {
                let config: WidgetConfig = serde_json::from_str(&json)
                    .map_err(|e| crate::Error::new(format!("parse config: {e}")))?;
                Ok(Some(config))
            }
            None => Ok(None),
        }
    }

    /// Native list: Android `AppWidgetManager` ids, iOS `WidgetCenter` configurations
    /// (instance = Edit Widget choice); merged with recent render receipts.
    pub fn get_active_widgets(
        &self,
        group: &str,
    ) -> crate::Result<Vec<crate::store::ActiveWidget>> {
        #[derive(Serialize)]
        struct G<'a> {
            group: &'a str,
        }
        let res: Value = self
            .handle
            .run_mobile_plugin("getActiveWidgets", G { group })?;
        let live: Vec<crate::store::ActiveWidget> =
            serde_json::from_value(res.get("widgets").cloned().unwrap_or(Value::Array(vec![])))?;
        let seen = crate::store::active_from_receipts(
            &self.get_widget_diagnostics(group).unwrap_or_default(),
            crate::store::now_ms(),
            crate::store::ACTIVE_RECEIPT_MAX_AGE_MS,
        );
        let exact = res
            .get("exactInstances")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        Ok(crate::store::merge_native_active(live, seen, exact))
    }

    /// JavaScript drains the queue itself: queued taps may be emitted from now on.
    pub fn mark_listener_ready(&self) {
        self.listener_gate.open();
    }

    /// Native widgets re-read data on reload; nothing to rebuild host-side.
    pub fn data_changed(&self, _group: &str, _widget_id: &str, _instance: Option<&str>) {}

    /// Native `setWidgetState {group, widgetId, instance, patch}`: apply the patch to
    /// `state:{widgetId}[#instance]` with the `set` operators (`patch: null` clears the
    /// layer) and resolve `{state}` with the placement's merged state.
    pub fn set_widget_state(
        &self,
        group: &str,
        widget_id: &str,
        instance: Option<&str>,
        patch: &Value,
    ) -> crate::Result<Option<Value>> {
        let res: Value = self.handle.run_mobile_plugin(
            "setWidgetState",
            StatePayload {
                group,
                widget_id,
                instance,
                patch: Some(patch),
            },
        )?;
        Ok(res.get("state").cloned().filter(Value::is_object))
    }

    /// Native `getWidgetState {group, widgetId, instance}` → `{state}` (merged state).
    pub fn get_widget_state(
        &self,
        group: &str,
        widget_id: &str,
        instance: Option<&str>,
    ) -> crate::Result<Option<Value>> {
        let res: Value = self.handle.run_mobile_plugin(
            "getWidgetState",
            StatePayload {
                group,
                widget_id,
                instance,
                patch: None,
            },
        )?;
        Ok(res.get("state").cloned().filter(Value::is_object))
    }

    pub fn start_activity(
        &self,
        content: crate::models::ActivityContent,
        group: &str,
    ) -> crate::Result<crate::models::ActivityHandle> {
        let res: Value = self.handle.run_mobile_plugin(
            "startActivity",
            ActivityPayload {
                content: &content,
                group,
            },
        )?;
        Ok(serde_json::from_value(res)?)
    }

    pub fn update_activity(
        &self,
        content: crate::models::ActivityContent,
        group: &str,
    ) -> crate::Result<bool> {
        self.handle
            .run_mobile_plugin(
                "updateActivity",
                ActivityPayload {
                    content: &content,
                    group,
                },
            )
            // `applied: false`: dropped as older than the last update (`timestamp`).
            .map(|res: Value| res.get("applied").and_then(Value::as_bool).unwrap_or(true))
            .map_err(Into::into)
    }

    pub fn end_activity(&self, id: &str, dismiss_after_ms: Option<u64>) -> crate::Result<bool> {
        self.end_activity_with(id, dismiss_after_ms, None, None)
    }

    /// Native `endActivity {id, dismissAfterMs, content?, dismissalPolicy?}`: `content` is
    /// the final state, `dismissalPolicy` `default` | `immediate` (`dismissAfterMs` wins).
    pub fn end_activity_with(
        &self,
        id: &str,
        dismiss_after_ms: Option<u64>,
        content: Option<&crate::models::ActivityContent>,
        dismissal_policy: Option<&str>,
    ) -> crate::Result<bool> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct E<'a> {
            id: &'a str,
            dismiss_after_ms: Option<u64>,
            #[serde(skip_serializing_if = "Option::is_none")]
            content: Option<&'a crate::models::ActivityContent>,
            #[serde(skip_serializing_if = "Option::is_none")]
            dismissal_policy: Option<&'a str>,
        }
        let res: Value = self.handle.run_mobile_plugin(
            "endActivity",
            E {
                id,
                dismiss_after_ms,
                content,
                dismissal_policy,
            },
        )?;
        Ok(res.get("ended").and_then(Value::as_bool).unwrap_or(true))
    }

    /// Native `getWidgetPushToken {group}` → `{token}` (WidgetKit push token, iOS 26+).
    pub fn widget_push_token(&self, group: &str) -> crate::Result<Option<String>> {
        let res: Value = self
            .handle
            .run_mobile_plugin("getWidgetPushToken", GroupPayload { group })?;
        Ok(res.get("token").and_then(Value::as_str).map(str::to_string))
    }

    /// iOS 17.2+: ActivityKit push-to-start token. Android has no equivalent (`None`).
    pub fn activity_push_to_start_token(&self) -> crate::Result<Option<String>> {
        #[cfg(target_os = "ios")]
        {
            let res: Value = self.handle.run_mobile_plugin("getPushToStartToken", ())?;
            Ok(res.get("token").and_then(Value::as_str).map(String::from))
        }
        #[cfg(not(target_os = "ios"))]
        Ok(None)
    }

    /// Android 16+ Live Updates promotion check; `true` on iOS (no such gate).
    pub fn can_promote_activities(&self) -> crate::Result<bool> {
        #[cfg(target_os = "android")]
        {
            let res: Value = self
                .handle
                .run_mobile_plugin("canPostPromotedNotifications", ())?;
            Ok(res.get("results").and_then(Value::as_bool).unwrap_or(true))
        }
        #[cfg(not(target_os = "android"))]
        Ok(true)
    }

    /// Android: opens the promotion settings screen; `false` on iOS.
    pub fn open_activity_promotion_settings(&self) -> crate::Result<bool> {
        #[cfg(target_os = "android")]
        {
            let res: Value = self.handle.run_mobile_plugin("openPromotionSettings", ())?;
            Ok(res.get("results").and_then(Value::as_bool).unwrap_or(false))
        }
        #[cfg(not(target_os = "android"))]
        Ok(false)
    }

    pub fn list_activities(&self) -> crate::Result<Vec<String>> {
        let res: Value = self.handle.run_mobile_plugin("listActivities", ())?;
        Ok(serde_json::from_value(
            res.get("ids").cloned().unwrap_or(Value::Array(vec![])),
        )?)
    }

    pub fn poll_pending_actions(
        &self,
        group: &str,
    ) -> crate::Result<Vec<crate::WidgetActionEnvelope>> {
        self.remember_group(group);
        let res: Value = self
            .handle
            .run_mobile_plugin("pollPendingActions", GroupPayload { group })?;
        let arr = res
            .get("results")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let mut out = Vec::with_capacity(arr.len());
        for item in arr {
            match serde_json::from_value::<crate::WidgetActionEnvelope>(item) {
                // A `set` tap in the widget: tell the app, never as a `widget-action`.
                Ok(env) if env.action == crate::store::STATE_ACTION => {
                    if let Some(ev) = crate::store::state_event(&env) {
                        let _ = self.app.emit("widget-state-changed", ev);
                    }
                }
                Ok(env) => out.push(env),
                Err(e) => {
                    log::warn!("poll_pending_actions: skip malformed envelope: {e}");
                }
            }
        }
        Ok(out)
    }

    pub fn report_receipt(&self, receipt: WidgetRenderReceipt) -> crate::Result<bool> {
        self.remember_group(&receipt.group);
        let trigger = receipt.trigger.clone().unwrap_or_else(|| "timeline".into());
        self.trace.push(TraceEvent::Render {
            instance: receipt.instance.clone(),
            nonce: receipt.nonce,
            source: receipt.source.clone(),
            trigger,
            lag_ms: 0,
            skipped: receipt.skipped.clone(),
        });
        self.receipts.upsert(receipt.clone());
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Payload {
            receipt_json: String,
        }
        let receipt_json =
            serde_json::to_string(&receipt).map_err(|e| crate::Error::new(e.to_string()))?;
        let _: Result<Value, _> = self
            .handle
            .run_mobile_plugin("reportReceipt", Payload { receipt_json });
        Ok(true)
    }

    pub fn get_widget_diagnostics(&self, group: &str) -> crate::Result<Vec<WidgetRenderReceipt>> {
        self.remember_group(group);
        #[derive(Serialize)]
        struct Group<'a> {
            group: &'a str,
        }
        if let Ok(res) = self
            .handle
            .run_mobile_plugin::<Value>("getWidgetDiagnostics", Group { group })
        {
            if let Some(arr) = res.get("results").and_then(|v| v.as_array()) {
                let mut out = Vec::new();
                for item in arr {
                    if let Ok(r) = serde_json::from_value::<WidgetRenderReceipt>(item.clone()) {
                        self.receipts.upsert(r.clone());
                        out.push(r);
                    }
                }
                if !out.is_empty() {
                    return Ok(out);
                }
            }
        }
        Ok(self.receipts.list(group))
    }

    pub fn get_widget_trace(
        &self,
        group: &str,
        since_ms: Option<u64>,
    ) -> crate::Result<WidgetTrace> {
        Ok(WidgetTrace {
            enabled: crate::trace::trace_enabled(),
            events: self.trace.list_since(since_ms),
            receipts: self.receipts.history(group),
        })
    }

    pub fn flush_widget_trace(&self) -> crate::Result<bool> {
        // Mobile: memory-only unless host later adds a shared file path.
        Ok(true)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StatePayload<'a> {
    group: &'a str,
    widget_id: &'a str,
    instance: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    patch: Option<&'a Value>,
}

#[derive(Serialize)]
struct ActivityPayload<'a> {
    content: &'a crate::models::ActivityContent,
    group: &'a str,
}
