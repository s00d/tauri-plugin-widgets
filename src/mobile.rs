use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::env;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;
use std::time::Instant;
use tauri::{plugin::PluginApi, AppHandle, Emitter, Runtime};

use crate::models::{WidgetConfig, WidgetWindowConfig};
use crate::receipt::{ReceiptStore, WidgetRenderReceipt};
use crate::store::config_key;

/// Default minimum interval between WidgetKit reload calls.
/// Can be overridden with `TAURI_WIDGET_MIN_RELOAD_SECS`.
#[cfg(debug_assertions)]
const DEFAULT_RELOAD_MIN_INTERVAL_SECS: u64 = 0;
#[cfg(not(debug_assertions))]
const DEFAULT_RELOAD_MIN_INTERVAL_SECS: u64 = 15 * 60;

fn reload_min_interval_secs() -> u64 {
    match env::var("TAURI_WIDGET_MIN_RELOAD_SECS") {
        Ok(v) => v
            .trim()
            .parse::<u64>()
            .unwrap_or(DEFAULT_RELOAD_MIN_INTERVAL_SECS),
        Err(_) => DEFAULT_RELOAD_MIN_INTERVAL_SECS,
    }
}

#[cfg(all(target_os = "android", feature = "android"))]
const PLUGIN_IDENTIFIER: &str = "git.s00d.widgets";

#[cfg(all(target_os = "ios", feature = "ios"))]
tauri::ios_plugin_binding!(init_plugin_widgets);

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> crate::Result<Widget<R>> {
    #[cfg(all(target_os = "android", feature = "android"))]
    let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "WidgetBridgePlugin")?;
    #[cfg(all(target_os = "ios", feature = "ios"))]
    let handle = api.register_ios_plugin(init_plugin_widgets)?;

    #[cfg(all(target_os = "android", not(feature = "android")))]
    compile_error!("tauri-plugin-widgets: enable feature `android` when targeting Android");
    #[cfg(all(target_os = "ios", not(feature = "ios")))]
    compile_error!("tauri-plugin-widgets: enable feature `ios` when targeting iOS");

    Ok(Widget {
        app: app.clone(),
        handle,
        last_config_hash: Mutex::new(HashMap::new()),
        last_reload: Mutex::new(None),
        known_groups: Mutex::new(HashSet::new()),
        receipts: ReceiptStore::new(),
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
    last_config_hash: Mutex<HashMap<(String, String), u64>>,
    last_reload: Mutex<Option<Instant>>,
    known_groups: Mutex<HashSet<String>>,
    receipts: ReceiptStore,
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
    fn throttled_reload(&self) -> crate::Result<bool> {
        let min_interval = reload_min_interval_secs();
        if min_interval == 0 {
            return self.reload_all_timelines();
        }

        let mut last = self.last_reload.lock().unwrap();
        let now = Instant::now();
        if let Some(prev) = *last {
            if now.duration_since(prev).as_secs() < min_interval {
                return Ok(false);
            }
        }
        *last = Some(now);
        drop(last);
        self.reload_all_timelines()
    }

    pub fn reload_timelines(&self, of_kind: &str) -> crate::Result<bool> {
        self.handle
            .run_mobile_plugin("reloadTimelines", ReloadPayload { of_kind })
            .map(|_: Value| true)
            .map_err(Into::into)
    }

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

    pub fn set_widget_config(
        &self,
        config: &WidgetConfig,
        group: &str,
        widget_id: &str,
        skip_reload: bool,
    ) -> crate::Result<bool> {
        if widget_id.is_empty() {
            return Err(crate::Error::new("widget_id must not be empty"));
        }
        self.remember_group(group);

        let json = serde_json::to_string(config)
            .map_err(|e| crate::Error::new(format!("serialize config: {e}")))?;

        let mut hasher = DefaultHasher::new();
        json.hash(&mut hasher);
        let new_hash = hasher.finish();
        let hash_key = (group.to_string(), widget_id.to_string());

        let changed = {
            let mut prev = self.last_config_hash.lock().unwrap();
            if prev.get(&hash_key) == Some(&new_hash) {
                false
            } else {
                prev.insert(hash_key, new_hash);
                true
            }
        };

        if !changed {
            return Ok(true);
        }

        crate::capabilities::log_capabilities(config);

        // Prefer native setWidgetConfig (Android image preprocess + Glance sync).
        // Falls back to set_items with config:{widgetId} key.
        let native_ok: Result<Value, _> = self.handle.run_mobile_plugin(
            "setWidgetConfig",
            SetWidgetConfigPayload {
                config: &json,
                group,
                widget_id,
            },
        );

        match native_ok {
            Ok(_) => {
                if !skip_reload {
                    self.throttled_reload()?;
                }
                Ok(true)
            }
            Err(_) => {
                self.set_items(&config_key(widget_id), &json, group)?;
                if !skip_reload {
                    self.throttled_reload()?;
                }
                Ok(true)
            }
        }
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

    pub fn poll_pending_actions(&self, group: &str) -> crate::Result<Vec<Value>> {
        self.remember_group(group);
        let res: Value = self
            .handle
            .run_mobile_plugin("pollPendingActions", GroupPayload { group })?;
        Ok(res
            .get("results")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default())
    }

    pub fn report_receipt(&self, receipt: WidgetRenderReceipt) -> crate::Result<bool> {
        self.remember_group(&receipt.group);
        self.receipts.upsert(receipt.clone());
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Payload {
            receipt_json: String,
        }
        let receipt_json = serde_json::to_string(&receipt)
            .map_err(|e| crate::Error::new(e.to_string()))?;
        let _: Result<Value, _> = self
            .handle
            .run_mobile_plugin("reportReceipt", Payload { receipt_json });
        Ok(true)
    }

    pub fn get_widget_diagnostics(
        &self,
        group: &str,
    ) -> crate::Result<Vec<WidgetRenderReceipt>> {
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
}
