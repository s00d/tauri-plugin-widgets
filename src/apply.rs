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
    use proptest::prelude::*;

    proptest! {
        /// Throttle model: no interval or interval elapsed → not throttled, else the rest.
        /// Content hash: equal bytes → equal hash.
        #[test]
        fn throttle_and_hash_follow_the_model(elapsed in 0u64..2_000, interval in 0u64..2_000, json in ".{0,40}") {
            let want = (interval != 0 && elapsed < interval).then(|| interval - elapsed);
            prop_assert_eq!(throttle_remaining_secs(elapsed, interval), want);
            prop_assert_eq!(config_content_hash(&json), config_content_hash(&json.clone()));
        }
    }

    /// A throttled reload on the wire (mobile release throttle; the unchanged outcome over
    /// IPC is in tests/offline_model.rs): camelCase tag and remaining seconds.
    #[test]
    fn throttled_reload_on_the_wire() {
        assert_eq!(throttle_remaining_secs(900, 900), None);
        let remaining_secs = throttle_remaining_secs(60, 900).unwrap();
        assert_eq!(remaining_secs, 840);
        let v = serde_json::to_value(ReloadOutcome::Throttled { remaining_secs }).unwrap();
        assert_eq!(v["outcome"], "throttled");
        assert_eq!(v["remainingSecs"], 840);
        assert_ne!(
            config_content_hash(r#"{"version":1}"#),
            config_content_hash(r#"{"version":2}"#)
        );
    }
}
