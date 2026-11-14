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
/// - On iOS: must start with `group.` and have a non-trap suffix
/// - On macOS: `group.<suffix>` or `<TEAMID>.<suffix>` (10 upper-case letters / digits). A
///   Team-ID group belongs to the app itself on macOS: no "access data from other apps"
///   checks, which `group.` ids get there (they stall file access and prompt)
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
        let Some(rest) = app_group_suffix(group) else {
            return Err(Error::new(if cfg!(target_os = "macos") {
                "group must be an App Group id: 'group.<id>' or '<TEAMID>.<id>'"
            } else {
                "group must be an App Group id starting with 'group.'"
            }));
        };
        if rest.is_empty() || rest == "." || rest == ".." {
            return Err(Error::new("group App Group suffix is invalid"));
        }
    }
    Ok(group)
}

/// The part after the App Group prefix: `group.` (iOS, macOS) or a Team ID `ABCDE12345.`
/// (macOS only); `None` when the id has neither.
pub fn app_group_suffix(group: &str) -> Option<&str> {
    if let Some(rest) = group.strip_prefix("group.") {
        return Some(rest);
    }
    let (team, rest) = group.split_once('.')?;
    let team_id = team.len() == 10
        && team
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit());
    (cfg!(target_os = "macos") && team_id).then_some(rest)
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

/// Validate an optional placement id (same charset as a widget id; empty means none).
pub fn validate_instance_id(instance: Option<&str>) -> Result<Option<&str>> {
    match instance.map(str::trim).filter(|s| !s.is_empty()) {
        None => Ok(None),
        Some(i)
            if i.chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-')) =>
        {
            Ok(Some(i))
        }
        Some(i) => Err(Error::new(format!(
            "instance \"{i}\" must use only letters, digits, '.', '_' or '-'"
        ))),
    }
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
    use proptest::prelude::*;

    const APPLE: bool = cfg!(any(target_os = "macos", target_os = "ios"));

    fn safe_charset(s: &str) -> bool {
        s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
    }

    /// Reference model of `validate_group`.
    fn group_ok(g: &str) -> bool {
        let base = !g.is_empty() && safe_charset(g) && g != "." && g != "..";
        if !APPLE {
            return base;
        }
        let rest = g.strip_prefix("group.").or_else(|| {
            g.strip_prefix("ABCDE12345.")
                .filter(|_| cfg!(target_os = "macos"))
        });
        base && rest.is_some_and(|rest| !rest.is_empty() && rest != "." && rest != "..")
    }

    /// Ids built from the safe charset, path characters, blanks and the `group.` / Team ID
    /// prefixes (a lower-case "team" is not one).
    fn ident() -> impl Strategy<Value = String> {
        (
            prop::option::of(prop_oneof![
                Just("group."),
                Just("ABCDE12345."),
                Just("abcde12345.")
            ]),
            "[a-z0-9._/\\\\# -]{0,6}",
            prop::option::of(Just(" ")),
        )
            .prop_map(|(p, body, pad)| format!("{}{}{}", p.unwrap_or(""), body, pad.unwrap_or("")))
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 512, ..ProptestConfig::default() })]
        /// Group, widget id and instance validators agree with their reference models for
        /// arbitrary input (path chars, dot segments, blanks, Apple `group.` prefix).
        #[test]
        fn validators_match_reference_models(s in ident()) {
            prop_assert_eq!(validate_group(&s).is_ok(), group_ok(&s), "group {:?}", s);
            prop_assert_eq!(validate_widget_id(&s).is_ok(), !s.is_empty() && safe_charset(&s), "widgetId {:?}", s);
            let trimmed = s.trim();
            let want = if trimmed.is_empty() { Some(None) } else if safe_charset(trimmed) { Some(Some(trimmed)) } else { None };
            prop_assert_eq!(validate_instance_id(Some(&s)).ok(), want, "instance {:?}", s);
        }
    }

    /// Rejections name the offending value (path chars, separators); edge cases are
    /// in the reference model above and the IPC model (tests/offline_model.rs).
    #[test]
    fn rejections_name_the_bad_value() {
        let e = validate_widget_id("a/b").unwrap_err().to_string();
        assert!(e.contains("widgetId \"a/b\""), "{e}");
        let e = validate_instance_id(Some("a#b")).unwrap_err().to_string();
        assert!(e.contains("instance \"a#b\""), "{e}");
        for g in [".", ".."] {
            let e = validate_group(g).unwrap_err().to_string();
            assert!(e.contains("not a valid identifier"), "{g}: {e}");
        }
    }

    /// What IPC cannot reach (the plugin always configures a default group): without a
    /// default and a request there is no group; an invalid requested group fails
    /// validation; the plugin's default feeds `resolve_group`. Blank / other / invalid
    /// groups over IPC: tests/offline_model.rs.
    #[test]
    fn group_resolution_without_a_default() {
        assert!(resolve_with(None, None).is_err());
        assert_eq!(
            resolve_with(None, Some("group.com.a")).unwrap(),
            "group.com.a"
        );
        let other = resolve_with(Some("group.com.b"), Some("group.com.a"));
        assert_eq!(other.is_ok(), !APPLE);
        assert!(resolve_with(Some("group.a/b"), None).is_err());

        set_default_group("group.unit.test");
        // OnceLock: another test may have set it first; either way it is a real group.
        let g = default_group().expect("default set");
        assert!(g.starts_with("group.") && g.len() > "group.".len(), "{g}");
        assert_eq!(resolve_group(None).unwrap(), g);
    }
}
