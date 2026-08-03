//! Plugin config from `tauri.conf.json` → `plugins.widgets`.

use serde::Deserialize;

#[cfg(feature = "schema")]
use schemars::JsonSchema;

/// Which Apple host→widget channel to use.
///
/// Pick this at build time (you know your signing). Do not rely on runtime fan-out.
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum TransportKind {
    /// App Group shared container file (`widget_data.json`). Needs a real Team ID.
    /// Production-safe default; use `auto` explicitly for ad-hoc probing.
    #[default]
    AppGroup,
    /// App Group `UserDefaults` suite. Same signing requirements as [`AppGroup`].
    UserDefaults,
    /// Widget extension container file. Works with ad-hoc signing; macOS host must not be sandboxed.
    WidgetContainer,
    /// One-shot probe at startup (dev only). Prefer pinning an explicit driver in conf.
    Auto,
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

    /// Wire / conf string for this kind (`appGroup`, `auto`, …).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AppGroup => "appGroup",
            Self::UserDefaults => "userDefaults",
            Self::WidgetContainer => "widgetContainer",
            Self::Auto => "auto",
        }
    }
}

/// `plugins.widgets` object from `tauri.conf.json`.
#[derive(Debug, Clone, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct WidgetsPluginConfig {
    /// Host→widget transport driver.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_accepts_common_spellings() {
        let cases = [
            ("appGroup", TransportKind::AppGroup),
            ("appgroup", TransportKind::AppGroup),
            ("app_group", TransportKind::AppGroup),
            ("app-group", TransportKind::AppGroup),
            (" App-Group ", TransportKind::AppGroup),
            ("userDefaults", TransportKind::UserDefaults),
            ("user_defaults", TransportKind::UserDefaults),
            ("user-defaults", TransportKind::UserDefaults),
            ("defaults", TransportKind::UserDefaults),
            ("widgetContainer", TransportKind::WidgetContainer),
            ("widget_container", TransportKind::WidgetContainer),
            ("widget-container", TransportKind::WidgetContainer),
            ("container", TransportKind::WidgetContainer),
            ("auto", TransportKind::Auto),
            ("AUTO", TransportKind::Auto),
        ];
        for (raw, want) in cases {
            assert_eq!(TransportKind::parse(raw), Some(want), "parse({raw:?})");
        }
    }

    #[test]
    fn parse_rejects_unknown() {
        assert_eq!(TransportKind::parse(""), None);
        assert_eq!(TransportKind::parse("app group"), None);
        assert_eq!(TransportKind::parse("fanout"), None);
    }

    #[test]
    fn as_str_roundtrips_through_parse() {
        for kind in [
            TransportKind::AppGroup,
            TransportKind::UserDefaults,
            TransportKind::WidgetContainer,
            TransportKind::Auto,
        ] {
            assert_eq!(TransportKind::parse(kind.as_str()), Some(kind));
        }
    }

    #[test]
    fn env_overrides_conf() {
        let cfg = WidgetsPluginConfig {
            transport: TransportKind::AppGroup,
            ..Default::default()
        };
        // SAFETY: serial tests; we restore afterwards.
        let prev = std::env::var_os("WIDGET_TRANSPORT");
        std::env::set_var("WIDGET_TRANSPORT", "widget-container");
        assert_eq!(effective_transport(&cfg), TransportKind::WidgetContainer);
        std::env::set_var("WIDGET_TRANSPORT", "");
        assert_eq!(
            effective_transport(&cfg),
            TransportKind::AppGroup,
            "empty env must not override"
        );
        match prev {
            Some(v) => std::env::set_var("WIDGET_TRANSPORT", v),
            None => std::env::remove_var("WIDGET_TRANSPORT"),
        }
    }
}
