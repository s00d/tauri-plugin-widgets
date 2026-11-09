//! App Group / widget group identifier validation.
//!
//! Fail-closed at the Tauri command boundary so path joins (macOS Containers,
//! Linux store files) never see traversal characters.

use crate::error::{Error, Result};

/// Validate a widget `group` string before any store / transport work.
///
/// - Non-empty
/// - Charset: ASCII alphanumeric, `.`, `_`, `-` only
/// - Rejects `.` / `..` (path-segment traps)
/// - On Apple targets: must start with `group.` and have a non-trap suffix
pub fn validate_group(group: &str) -> Result<&str> {
    if group.is_empty() {
        return Err(Error::new("group must not be empty"));
    }
    if !group
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
    {
        return Err(Error::new("group contains invalid characters"));
    }
    if group == "." || group == ".." {
        return Err(Error::new("group is not a valid identifier"));
    }
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        const PREFIX: &str = "group.";
        if !group.starts_with(PREFIX) {
            return Err(Error::new(
                "group must be an App Group id starting with 'group.'",
            ));
        }
        let rest = &group[PREFIX.len()..];
        if rest.is_empty() || rest == "." || rest == ".." {
            return Err(Error::new("group App Group suffix is invalid"));
        }
    }
    Ok(group)
}

/// Validate a `widgetId`: it becomes the `config:{widgetId}` store key, which native
/// stores sanitize differently, so only a safe charset is accepted.
pub fn validate_widget_id(widget_id: &str) -> Result<&str> {
    if widget_id.is_empty()
        || !widget_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
    {
        return Err(Error::new(format!(
            "widgetId \"{widget_id}\" must be non-empty and use only letters, digits, '.', '_' or '-'"
        )));
    }
    Ok(widget_id)
}

/// Process-wide default group: `plugins.widgets.appGroup`, else `group.<identifier>`.
/// Set once by plugin init (the plugin is a singleton per app process).
static DEFAULT_GROUP: std::sync::OnceLock<String> = std::sync::OnceLock::new();

/// Called by plugin init with the effective (defaulted) App Group.
pub(crate) fn set_default_group(group: &str) {
    let _ = DEFAULT_GROUP.set(group.to_string());
}

/// The group used when a command omits `group`.
pub fn default_group() -> Option<&'static str> {
    DEFAULT_GROUP.get().map(String::as_str)
}

/// Group for a command: the requested one (validated) or the default.
///
/// On Apple the host process is entitled to exactly one App Group and every transport
/// is built for it, so another group is rejected instead of silently writing into the
/// configured group's store.
pub fn resolve_group(requested: Option<&str>) -> Result<String> {
    resolve_with(
        requested.map(str::trim).filter(|s| !s.is_empty()),
        default_group(),
    )
}

fn resolve_with(requested: Option<&str>, default: Option<&str>) -> Result<String> {
    let Some(group) = requested.or(default) else {
        return Err(Error::new(
            "no widget group: pass `group` or set plugins.widgets.appGroup in tauri.conf.json",
        ));
    };
    validate_group(group)?;
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    if let Some(default) = default {
        if group != default {
            return Err(Error::new(format!(
                "group \"{group}\" does not match plugins.widgets.appGroup \"{default}\"; \
                 an Apple app has one App Group, omit `group` or pass \"{default}\""
            )));
        }
    }
    Ok(group.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_reverse_dns() {
        assert!(validate_group("group.com.example.app").is_ok());
        #[cfg(not(any(target_os = "macos", target_os = "ios")))]
        assert!(validate_group("com.example.app").is_ok());
    }

    #[test]
    fn resolve_uses_default_when_omitted() {
        assert_eq!(
            resolve_with(None, Some("group.com.a")).unwrap(),
            "group.com.a"
        );
        assert!(resolve_with(None, None).is_err());
    }

    #[test]
    fn resolve_rejects_other_group_on_apple_only() {
        let other = resolve_with(Some("group.com.b"), Some("group.com.a"));
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        assert!(other.is_err());
        #[cfg(not(any(target_os = "macos", target_os = "ios")))]
        assert_eq!(other.unwrap(), "group.com.b");
        assert_eq!(
            resolve_with(Some("group.com.a"), Some("group.com.a")).unwrap(),
            "group.com.a"
        );
    }

    #[test]
    fn rejects_path_chars() {
        assert!(validate_group("group.com/../../tmp").is_err());
        assert!(validate_group("group.com\\evil").is_err());
        assert!(validate_group("").is_err());
        assert!(validate_group("..").is_err());
    }

    #[test]
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    fn apple_requires_group_prefix() {
        assert!(validate_group("com.example.app").is_err());
        assert!(validate_group("group.").is_err());
        assert!(validate_group("group..").is_err());
        assert!(validate_group("group...").is_err());
    }
}
