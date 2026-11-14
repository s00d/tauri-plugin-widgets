//! Live activity as a system notification (feature `notify`): Linux notify-rust with a
//! replace-id, Windows toast with an in-place progress bar. Without the feature, or on
//! macOS, [`Notifier::show`] fails and the caller keeps the window surface.

use crate::models::ActivityContent;

#[cfg(all(feature = "notify", target_os = "linux"))]
type Handle = notify_rust::NotificationHandle;
#[cfg(all(feature = "notify", target_os = "windows"))]
type Handle = String; // toast tag

#[derive(Default)]
pub(crate) struct Notifier {
    #[cfg(all(feature = "notify", any(target_os = "linux", target_os = "windows")))]
    shown: std::sync::Mutex<std::collections::HashMap<String, Handle>>,
}

/// `"Title"`, `"Body · 40%"` for the notification text.
pub(crate) fn notification_text(c: &ActivityContent) -> (String, String) {
    let pct = c
        .progress
        .map(|p| format!("{:.0}%", p.clamp(0.0, 1.0) * 100.0));
    let body = match (c.body.as_deref().filter(|b| !b.is_empty()), pct) {
        (Some(b), Some(p)) => format!("{b} · {p}"),
        (Some(b), None) => b.to_string(),
        (None, Some(p)) => p,
        (None, None) => String::new(),
    };
    (c.title.clone(), body)
}

impl Notifier {
    /// Whether this build can show activities as notifications at all.
    pub const AVAILABLE: bool = cfg!(all(
        feature = "notify",
        any(target_os = "linux", target_os = "windows")
    ));

    pub fn has(&self, id: &str) -> bool {
        #[cfg(all(feature = "notify", any(target_os = "linux", target_os = "windows")))]
        return self.shown.lock().unwrap().contains_key(id);
        #[cfg(not(all(feature = "notify", any(target_os = "linux", target_os = "windows"))))]
        {
            let _ = id;
            false
        }
    }

    /// Show or replace the notification for `content.id`.
    #[allow(unused_variables)]
    pub fn show(&self, app_name: &str, content: &ActivityContent) -> crate::Result<()> {
        let (title, body) = notification_text(content);
        #[cfg(all(feature = "notify", target_os = "linux"))]
        {
            use notify_rust::{Hint, Notification};
            let mut shown = self.shown.lock().unwrap();
            let mut n = Notification::new();
            n.appname(app_name).summary(&title).body(&body);
            if let Some(p) = content.progress {
                // GNOME / KDE progress hint.
                n.hint(Hint::CustomInt(
                    "value".into(),
                    (p.clamp(0.0, 1.0) * 100.0) as i32,
                ));
            }
            if let Some(prev) = shown.get(&content.id) {
                n.id(prev.id());
            }
            let handle = n
                .show()
                .map_err(|e| crate::Error::new(format!("notification: {e}")))?;
            shown.insert(content.id.clone(), handle);
            Ok(())
        }
        #[cfg(all(feature = "notify", target_os = "windows"))]
        {
            use tauri_winrt_notification::{Progress, Toast};
            // An unpackaged dev build has no registered AppUserModelID.
            let app_id = if cfg!(debug_assertions) {
                Toast::POWERSHELL_APP_ID
            } else {
                app_name
            };
            let mut shown = self.shown.lock().unwrap();
            let err =
                |e: tauri_winrt_notification::Error| crate::Error::new(format!("toast: {e:?}"));
            let progress = content.progress.map(|p| Progress {
                tag: content.id.clone(),
                title: title.clone(),
                status: content.body.clone().unwrap_or_default(),
                value: p.clamp(0.0, 1.0) as f32,
                value_string: format!("{:.0}%", p.clamp(0.0, 1.0) * 100.0),
            });
            // Progress updates in place; text-only content posts a fresh toast.
            if let (Some(p), true) = (&progress, shown.contains_key(&content.id)) {
                if Toast::new(app_id).set_progress(p).is_ok() {
                    return Ok(());
                }
            }
            let mut toast = Toast::new(app_id).title(&title).text1(&body);
            if let Some(p) = &progress {
                toast = toast.progress(p);
            }
            toast.show().map_err(err)?;
            shown.insert(content.id.clone(), content.id.clone());
            Ok(())
        }
        #[cfg(not(all(feature = "notify", any(target_os = "linux", target_os = "windows"))))]
        Err(crate::Error::Unsupported(
            "activity notifications need the `notify` feature on Linux / Windows".into(),
        ))
    }

    /// Close (Linux) or forget (Windows toasts expire on their own) the notification.
    pub fn close(&self, id: &str) -> bool {
        #[cfg(all(feature = "notify", target_os = "linux"))]
        {
            let h = self.shown.lock().unwrap().remove(id);
            let found = h.is_some();
            if let Some(h) = h {
                h.close();
            }
            found
        }
        #[cfg(all(feature = "notify", target_os = "windows"))]
        return self.shown.lock().unwrap().remove(id).is_some();
        #[cfg(not(all(feature = "notify", any(target_os = "linux", target_os = "windows"))))]
        {
            let _ = id;
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Notification text as an activity fills in (title only → progress → body → progress
    /// clamped to 100 %), then showing it: without a backend `show` fails (so the caller
    /// keeps the window) and nothing is tracked or closable.
    #[test]
    fn activity_text_then_show_without_backend() {
        let mut c = ActivityContent {
            id: "dl".into(),
            title: "Download".into(),
            ..Default::default()
        };
        assert_eq!(notification_text(&c).1, "");
        c.progress = Some(0.4);
        assert_eq!(notification_text(&c).1, "40%");
        c.body = Some("file.zip".into());
        assert_eq!(
            notification_text(&c),
            ("Download".into(), "file.zip · 40%".into())
        );
        c.progress = Some(7.0);
        assert_eq!(notification_text(&c).1, "file.zip · 100%");

        let n = Notifier::default();
        if !Notifier::AVAILABLE {
            assert!(n.show("app", &c).is_err());
            assert!(!n.has(&c.id) && !n.close(&c.id));
        }
    }
}
