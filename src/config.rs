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
    /// Host→widget transport driver. Unset: `widgetContainer` on macOS (a Developer ID
    /// extension without a provisioning profile cannot open a `group.*` container),
    /// `appGroup` elsewhere.
    #[serde(default)]
    pub transport: Option<TransportKind>,
    /// App Group id, e.g. `group.com.example.app`.
    pub app_group: Option<String>,
    /// Widget extension bundle id (macOS container path). Default: `{app_id}.widgetkit`.
    pub extension_bundle_id: Option<String>,
    /// Desktop: reopen the widget windows that were open when the app last quit.
    #[serde(default)]
    pub restore_windows_on_start: bool,
    /// Desktop surface for live activities: `"island"` (default) a Dynamic-Island pill at
    /// the top centre of the primary display (around the notch on notched Macs, expands on
    /// hover); `"window"` a small always-on-top window in the top-right corner;
    /// `"notification"` a system notification (feature `notify`, Linux / Windows).
    #[serde(default)]
    pub activity_surface: Option<String>,
}

impl WidgetsPluginConfig {
    /// Fill `appGroup` with `group.<identifier>` when the app did not set it, and make it
    /// the default group for commands that omit `group`.
    pub fn with_defaults(mut self, identifier: &str) -> Self {
        let missing = self
            .app_group
            .as_deref()
            .map(str::trim)
            .unwrap_or("")
            .is_empty();
        if missing && !identifier.trim().is_empty() {
            self.app_group = Some(format!("group.{}", identifier.trim()));
        }
        if let Some(group) = self.app_group.as_deref() {
            crate::group::set_default_group(group);
        }
        self
    }
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
    env_transport_override()
        .or(cfg.transport)
        .unwrap_or(if cfg!(target_os = "macos") {
            TransportKind::WidgetContainer
        } else {
            TransportKind::AppGroup
        })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// Serializes lib tests that set process env (`WIDGET_TRANSPORT`, `WIDGET_*` paths).
    pub(crate) static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    const KINDS: [TransportKind; 4] = [
        TransportKind::AppGroup,
        TransportKind::UserDefaults,
        TransportKind::WidgetContainer,
        TransportKind::Auto,
    ];

    /// `transport` string lifecycle: every accepted spelling parses → unknown ones are
    /// rejected → the canonical `as_str` spelling parses back to the same kind.
    #[test]
    fn transport_kind_spellings_parse_reject_and_roundtrip() {
        // 1. Common spellings (case, `_` / `-`, short aliases, surrounding blanks).
        let accepted = [
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
        for (raw, want) in accepted {
            assert_eq!(TransportKind::parse(raw), Some(want), "parse({raw:?})");
        }
        // 2. Anything else is not a transport (no fuzzy match, inner blanks are not separators).
        for raw in ["", "app group", "fanout"] {
            assert_eq!(TransportKind::parse(raw), None, "parse({raw:?})");
        }
        // 3. The conf spelling we print round-trips, and the default is the production one.
        for kind in KINDS {
            assert_eq!(TransportKind::parse(kind.as_str()), Some(kind));
        }
        assert_eq!(TransportKind::default(), TransportKind::AppGroup);
    }

    /// `plugins.widgets` from tauri.conf → `with_defaults(identifier)` → effective transport
    /// (unset → OS default, conf value, `WIDGET_TRANSPORT` override, empty env ignored).
    #[test]
    fn plugin_config_defaults_then_effective_transport() {
        // 1. Conf with a blank appGroup: filled from the identifier (trimmed).
        let conf: WidgetsPluginConfig =
            serde_json::from_value(serde_json::json!({"appGroup": "  "})).unwrap();
        assert_eq!(conf.transport, None);
        assert!(!conf.restore_windows_on_start);
        let filled = conf.with_defaults(" com.ex.app ");
        assert_eq!(filled.app_group.as_deref(), Some("group.com.ex.app"));
        assert_eq!(
            WidgetsPluginConfig::default()
                .with_defaults("com.ex.app")
                .app_group
                .as_deref(),
            Some("group.com.ex.app")
        );
        // 2. An explicit group and transport are kept as written.
        let kept: WidgetsPluginConfig = serde_json::from_value(
            serde_json::json!({"appGroup": "group.custom", "transport": "appGroup"}),
        )
        .unwrap();
        let kept = kept.with_defaults("com.ex.app");
        assert_eq!(kept.app_group.as_deref(), Some("group.custom"));
        assert_eq!(kept.transport, Some(TransportKind::AppGroup));
        // 3. No identifier → nothing to derive from.
        assert_eq!(
            WidgetsPluginConfig::default().with_defaults("  ").app_group,
            None
        );

        // 4. Effective transport: conf wins over the OS default, env wins over conf.
        let _env = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev = std::env::var_os("WIDGET_TRANSPORT");
        std::env::remove_var("WIDGET_TRANSPORT");
        let os_default = if cfg!(target_os = "macos") {
            TransportKind::WidgetContainer
        } else {
            TransportKind::AppGroup
        };
        assert_eq!(effective_transport(&filled), os_default);
        assert_eq!(effective_transport(&kept), TransportKind::AppGroup);
        std::env::set_var("WIDGET_TRANSPORT", "widget-container");
        assert_eq!(
            env_transport_override(),
            Some(TransportKind::WidgetContainer)
        );
        assert_eq!(effective_transport(&kept), TransportKind::WidgetContainer);
        std::env::set_var("WIDGET_TRANSPORT", "");
        assert_eq!(
            effective_transport(&kept),
            TransportKind::AppGroup,
            "empty env must not override"
        );
        std::env::set_var("WIDGET_TRANSPORT", "bogus");
        assert_eq!(effective_transport(&kept), TransportKind::AppGroup);
        match prev {
            Some(v) => std::env::set_var("WIDGET_TRANSPORT", v),
            None => std::env::remove_var("WIDGET_TRANSPORT"),
        }
    }
}
