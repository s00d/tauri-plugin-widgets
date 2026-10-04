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
