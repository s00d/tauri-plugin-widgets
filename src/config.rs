//! Plugin config from `tauri.conf.json` → `plugins.widgets`.

use serde::Deserialize;

/// Which Apple host→widget channel to use.
///
/// Pick this at build time (you know your signing). Do not rely on runtime fan-out.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TransportKind {
    /// App Group shared container file (`widget_data.json`). Needs a real Team ID.
    AppGroup,
    /// App Group `UserDefaults` suite. Same signing requirements as [`AppGroup`].
    UserDefaults,
    /// Widget extension container file. Works with ad-hoc signing; macOS host must not be sandboxed.
    WidgetContainer,
    /// One-shot probe at startup (dev only). Prefer pinning an explicit driver in conf.
    Auto,
}

impl Default for TransportKind {
    fn default() -> Self {
        // Production-safe default: shared App Group file. Use `auto` explicitly for ad-hoc probing.
        Self::AppGroup
    }
}

impl TransportKind {
    /// Parse `WIDGET_TRANSPORT` / conf string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "appgroup" | "app_group" | "app-group" => Some(Self::AppGroup),
            "userdefaults" | "user_defaults" | "user-defaults" | "defaults" => {
                Some(Self::UserDefaults)
            }
            "widgetcontainer" | "widget_container" | "widget-container" | "container" => {
                Some(Self::WidgetContainer)
            }
            "auto" => Some(Self::Auto),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::AppGroup => "appGroup",
            Self::UserDefaults => "userDefaults",
            Self::WidgetContainer => "widgetContainer",
            Self::Auto => "auto",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WidgetsPluginConfig {
    #[serde(default)]
    pub transport: TransportKind,
    /// App Group id, e.g. `group.com.example.app`.
    pub app_group: Option<String>,
    /// Widget extension bundle id (macOS container path). Default: `{app_id}.widgetkit`.
    pub extension_bundle_id: Option<String>,
}

/// `WIDGET_TRANSPORT` overrides `plugins.widgets.transport` when set.
pub fn env_transport_override() -> Option<TransportKind> {
    std::env::var("WIDGET_TRANSPORT")
        .ok()
        .filter(|s| !s.is_empty())
        .and_then(|s| TransportKind::parse(&s))
}

/// Merge conf + env into the effective kind.
pub fn effective_transport(cfg: &WidgetsPluginConfig) -> TransportKind {
    env_transport_override().unwrap_or(cfg.transport)
}
