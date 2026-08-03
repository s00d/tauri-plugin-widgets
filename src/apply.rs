//! Outcomes for `set_widget_config` — no silent `Ok(true)`.

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Why a config write was skipped.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "camelCase")]
pub enum SkipReason {
    /// Store already holds identical JSON for this widget id.
    Unchanged {
        /// Hash of the unchanged config bytes.
        hash: u64,
    },
    /// No live widget instances to deliver to.
    NoInstances,
    /// Selected transport is unavailable.
    TransportUnavailable {
        /// Transport name that was unavailable.
        name: String,
    },
}

/// Result of a WidgetKit / AppWidget reload attempt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "camelCase")]
pub enum ReloadOutcome {
    /// Native reload was invoked.
    Ok,
    /// Mobile release throttle skipped the reload.
    Throttled {
        /// Seconds until the next reload is allowed.
        #[serde(rename = "remainingSecs")]
        remaining_secs: u64,
    },
    /// Reload intentionally not called.
    Skipped {
        /// Human-readable why (`unchanged`, `skip_reload`, …).
        why: String,
    },
    /// Reload was attempted but failed.
    Failed {
        /// Error message.
        error: String,
    },
}

/// Full result of [`crate::desktop::Widget::set_widget_config`] /
/// [`crate::mobile::Widget::set_widget_config`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyOutcome {
    /// `true` when the config map was written (bytes differed from store).
    pub written: bool,
    /// What happened on the native reload path.
    pub reload: ReloadOutcome,
    /// Transport names that received the map (empty when `written` is false).
    pub transports: Vec<String>,
    /// Present when the write was skipped.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skip: Option<SkipReason>,
}

impl ApplyOutcome {
    /// Unchanged store — push may still fire on desktop; reload skipped.
    pub fn unchanged(hash: u64) -> Self {
        Self {
            written: false,
            reload: ReloadOutcome::Skipped {
                why: "unchanged".into(),
            },
            transports: Vec::new(),
            skip: Some(SkipReason::Unchanged { hash }),
        }
    }
}

/// Stable content hash for config JSON bytes (dedup vs store).
pub fn config_content_hash(json: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    json.hash(&mut hasher);
    hasher.finish()
}

/// Seconds remaining until the next reload is allowed, if currently throttled.
pub fn throttle_remaining_secs(elapsed_secs: u64, min_interval: u64) -> Option<u64> {
    if min_interval == 0 || elapsed_secs >= min_interval {
        None
    } else {
        Some(min_interval.saturating_sub(elapsed_secs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unchanged_serializes_camel_case() {
        let o = ApplyOutcome::unchanged(42);
        let v = serde_json::to_value(&o).unwrap();
        assert_eq!(v["written"], false);
        assert_eq!(v["reload"]["outcome"], "skipped");
        assert_eq!(v["skip"]["reason"], "unchanged");
        assert_eq!(v["skip"]["hash"], 42);
    }

    #[test]
    fn throttled_remaining_secs_camel() {
        let r = ReloadOutcome::Throttled {
            remaining_secs: 840,
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["outcome"], "throttled");
        assert_eq!(v["remainingSecs"], 840);
    }

    #[test]
    fn throttle_remaining_math() {
        assert_eq!(throttle_remaining_secs(0, 0), None);
        assert_eq!(throttle_remaining_secs(100, 0), None);
        assert_eq!(throttle_remaining_secs(60, 900), Some(840));
        assert_eq!(throttle_remaining_secs(900, 900), None);
        assert_eq!(throttle_remaining_secs(901, 900), None);
    }

    #[test]
    fn same_json_same_hash() {
        assert_eq!(
            config_content_hash(r#"{"version":1}"#),
            config_content_hash(r#"{"version":1}"#)
        );
        assert_ne!(
            config_content_hash(r#"{"version":1}"#),
            config_content_hash(r#"{"version":2}"#)
        );
    }
}
