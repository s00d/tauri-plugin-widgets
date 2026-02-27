use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::collections::hash_map::DefaultHasher;
use std::env;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;
use std::time::Instant;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::models::{WidgetConfig, WidgetWindowConfig};

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

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "git.s00d.widgets";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_widgets);

pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> crate::Result<Widget<R>> {
    #[cfg(target_os = "android")]
    let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "WidgetBridgePlugin")?;
    #[cfg(target_os = "ios")]
    let handle = api.register_ios_plugin(init_plugin_widgets)?;
    Ok(Widget {
        handle,
        last_config_hash: Mutex::new(0),
        last_reload: Mutex::new(None),
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

// ── Widget ──────────────────────────────────────────────────────────────────

pub struct Widget<R: Runtime> {
    handle: tauri::plugin::PluginHandle<R>,
    last_config_hash: Mutex<u64>,
    last_reload: Mutex<Option<Instant>>,
}

impl<R: Runtime> Widget<R> {
    pub fn set_items(&self, key: &str, value: &str, group: &str) -> crate::Result<bool> {
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
    /// Returns `true` if the reload was actually dispatched.
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

    pub fn set_widget_config(&self, config: &WidgetConfig, group: &str, skip_reload: bool) -> crate::Result<bool> {
        let json = serde_json::to_string(config)
            .map_err(|e| crate::Error::new(format!("serialize config: {e}")))?;

        let mut hasher = DefaultHasher::new();
        json.hash(&mut hasher);
        let new_hash = hasher.finish();

        let changed = {
            let mut prev = self.last_config_hash.lock().unwrap();
            if *prev == new_hash {
                false
            } else {
                *prev = new_hash;
                true
            }
        };

        if changed {
            self.set_items("__widget_config__", &json, group)?;
            if !skip_reload {
                self.throttled_reload()?;
            }
        }
        Ok(true)
    }

    pub fn get_widget_config(&self, group: &str) -> crate::Result<Option<WidgetConfig>> {
        match self.get_items("__widget_config__", group)? {
            Some(json) => {
                let config: WidgetConfig = serde_json::from_str(&json)
                    .map_err(|e| crate::Error::new(format!("parse config: {e}")))?;
                Ok(Some(config))
            }
            None => Ok(None),
        }
    }

    pub fn poll_pending_actions(&self, group: &str) -> crate::Result<Vec<Value>> {
        let res: Value = self
            .handle
            .run_mobile_plugin("pollPendingActions", GroupPayload { group })?;
        Ok(res
            .get("results")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default())
    }
}
