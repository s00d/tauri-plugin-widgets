//! Host-registered data sources (eww `defpoll` / `deflisten`), Rust API only: a closure
//! polled on an interval, or a channel / iterator, feeding `set_widget_data`.

use std::time::Duration;

use tauri::{Emitter, Manager, Runtime};

use super::Widget;

impl<R: Runtime> Widget<R> {
    /// `setWidgetData` from Rust: store `data` for `{{data.…}}`, push it to open widget
    /// windows and reload native widgets. Unchanged data is a no-op (`Ok(false)`).
    /// `group: None` uses the configured default group.
    pub fn set_widget_data(
        &self,
        widget_id: &str,
        data: &serde_json::Value,
        instance: Option<&str>,
        group: Option<&str>,
    ) -> crate::Result<bool> {
        let group = crate::group::resolve_group(group)?;
        crate::group::validate_widget_id(widget_id)?;
        let instance = crate::group::validate_instance_id(instance)?;
        let key = crate::store::data_key(widget_id, instance);
        let raw = if data.is_null() {
            String::new()
        } else {
            serde_json::to_string(data)?
        };
        if raw.len() > crate::resolve::MAX_SOURCE_BYTES {
            return Err(crate::Error::new(format!(
                "widget data is {} bytes; the limit is {}",
                raw.len(),
                crate::resolve::MAX_SOURCE_BYTES
            )));
        }
        if self.get_items(&key, &group)?.as_deref() == Some(raw.as_str()) {
            return Ok(false);
        }
        self.set_items(&key, &raw, &group)?;
        self.data_changed(&group, widget_id, instance);
        let _ = self.app.emit(
            "widget-data-push",
            serde_json::json!({
                "group": group,
                "widgetId": widget_id,
                "instance": instance,
                "data": data,
            }),
        );
        self.reload_all_timelines()?;
        Ok(true)
    }

    /// Call `poll` every `interval` (first call right away) and store the result as the
    /// widget's data (default group). Registering a widget id again replaces its source.
    ///
    /// ```no_run
    /// # fn f<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    /// use tauri_plugin_widgets::WidgetExt;
    /// app.widget().register_poll_source("cpu", std::time::Duration::from_secs(2), || {
    ///     serde_json::json!({ "load": 0.42 })
    /// });
    /// # }
    /// ```
    pub fn register_poll_source<F>(&self, widget_id: &str, interval: Duration, mut poll: F)
    where
        F: FnMut() -> serde_json::Value + Send + 'static,
    {
        let interval = interval.max(Duration::from_millis(100));
        self.spawn_source(widget_id, move |alive, push| {
            while alive() {
                push(poll());
                std::thread::sleep(interval);
            }
        });
    }

    /// Store every value `values` yields (a `std::sync::mpsc::Receiver`, any blocking
    /// iterator) as the widget's data until it ends or the source is replaced.
    pub fn register_listen_source<I>(&self, widget_id: &str, values: I)
    where
        I: IntoIterator<Item = serde_json::Value>,
        I::IntoIter: Send + 'static,
    {
        let iter = values.into_iter();
        self.spawn_source(widget_id, move |alive, push| {
            for v in iter {
                if !alive() {
                    break;
                }
                push(v);
            }
        });
    }

    /// Stop a poll / listen source. A listen source stops at its next value.
    pub fn unregister_source(&self, widget_id: &str) -> bool {
        self.sources.lock().unwrap().remove(widget_id).is_some()
    }

    fn spawn_source<B>(&self, widget_id: &str, body: B)
    where
        B: FnOnce(&dyn Fn() -> bool, &dyn Fn(serde_json::Value)) + Send + 'static,
    {
        // Never reused, so a stopped thread cannot mistake a newer source for its own.
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let gen = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.sources
            .lock()
            .unwrap()
            .insert(widget_id.to_string(), gen);
        let app = self.app.clone();
        let id = widget_id.to_string();
        std::thread::spawn(move || {
            let alive = || {
                app.try_state::<Widget<R>>()
                    .is_some_and(|w| w.sources.lock().unwrap().get(&id) == Some(&gen))
            };
            let push = |v: serde_json::Value| {
                if let Some(w) = app.try_state::<Widget<R>>() {
                    if let Err(e) = w.set_widget_data(&id, &v, None, None) {
                        log::warn!("widgets: data source {id}: {e}");
                    }
                }
            };
            body(&alive, &push);
            if let Some(w) = app.try_state::<Widget<R>>() {
                let mut s = w.sources.lock().unwrap();
                if s.get(&id) == Some(&gen) {
                    s.remove(&id);
                }
            }
        });
    }
}
