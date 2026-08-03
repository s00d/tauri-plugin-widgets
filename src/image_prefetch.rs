//! Prefetch remote `image.url` into `data:` URIs for hosts that cannot fetch
//! (WidgetKit). Android keeps its own `localPath` pipeline — Rust prefetch is a
//! no-op there so we do not inflate SharedPreferences / Binder with base64.
//!
//! Behind the `image-prefetch` feature (`ureq`). In-memory + disk cache by URL
//! with TTL so `startWidgetUpdater` does not hit the network every tick.
//! The command path only reads the cache; network fetches run on a background
//! thread so a slow host cannot stall `set_widget_config`.
//! Successful prefetch **keeps** `url` so Android / Adaptive Cards can still use it.

use crate::models::WidgetConfig;
#[cfg(all(feature = "image-prefetch", not(target_os = "android")))]
use crate::models::{ImageElement, WidgetElement};

/// Walk the config and fill `image.data` from http(s) URLs when needed.
///
/// - Does **not** clear `url` (consumers choose `data` / `localPath` / `url`).
/// - No-op on Android and when the `image-prefetch` feature is disabled.
pub fn prefetch_remote_images(config: &mut WidgetConfig) {
    #[cfg(all(feature = "image-prefetch", not(target_os = "android")))]
    {
        prefetch_remote_images_inner(config);
    }
    #[cfg(not(all(feature = "image-prefetch", not(target_os = "android"))))]
    {
        let _ = config;
    }
}

#[cfg(all(feature = "image-prefetch", not(target_os = "android")))]
mod imp {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    const MAX_BYTES: usize = 3 * 1024 * 1024;
    const TIMEOUT: Duration = Duration::from_secs(10);
    const CACHE_TTL: Duration = Duration::from_secs(300);

    #[derive(Clone)]
    struct CacheEntry {
        data_uri: String,
        fetched_at: Instant,
    }

    static MEMORY: Mutex<Option<HashMap<String, CacheEntry>>> = Mutex::new(None);
    static IN_FLIGHT: Mutex<Option<std::collections::HashSet<String>>> = Mutex::new(None);

    fn memory() -> std::sync::MutexGuard<'static, Option<HashMap<String, CacheEntry>>> {
        MEMORY.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub(super) fn prefetch_remote_images_inner(config: &mut WidgetConfig) {
        if let Some(el) = config.small.as_mut() {
            walk_el(el);
        }
        if let Some(el) = config.medium.as_mut() {
            walk_el(el);
        }
        if let Some(el) = config.large.as_mut() {
            walk_el(el);
        }
    }

    fn walk_el(el: &mut WidgetElement) {
        match el {
            WidgetElement::Image(img) => prefetch_image(img),
            WidgetElement::VStack(v) => {
                for c in &mut v.children {
                    walk_el(c);
                }
            }
            WidgetElement::HStack(v) => {
                for c in &mut v.children {
                    walk_el(c);
                }
            }
            WidgetElement::ZStack(v) => {
                for c in &mut v.children {
                    walk_el(c);
                }
            }
            WidgetElement::Grid(v) => {
                for c in &mut v.children {
                    walk_el(c);
                }
            }
            WidgetElement::Container(v) => {
                for c in &mut v.children {
                    walk_el(c);
                }
            }
            WidgetElement::Link(v) => {
                for c in &mut v.children {
                    walk_el(c);
                }
            }
            WidgetElement::List(_)
            | WidgetElement::Button(_)
            | WidgetElement::Toggle(_)
            | WidgetElement::Text(_)
            | WidgetElement::Label(_)
            | WidgetElement::Progress(_)
            | WidgetElement::Gauge(_)
            | WidgetElement::Divider(_)
            | WidgetElement::Spacer(_)
            | WidgetElement::Shape(_)
            | WidgetElement::Date(_)
            | WidgetElement::Timer(_)
            | WidgetElement::Chart(_)
            | WidgetElement::Canvas(_) => {}
        }
    }

    fn prefetch_image(img: &mut ImageElement) {
        let Some(url) = img.url.as_deref() else {
            return;
        };
        let url = url.trim();
        if !(url.starts_with("http://") || url.starts_with("https://")) {
            return;
        }
        if img
            .data
            .as_deref()
            .map(|d| !d.trim().is_empty())
            .unwrap_or(false)
        {
            return;
        }
        // Hot path: memory/disk only — never block `set_widget_config` on HTTP.
        if let Some(cached) = cache_get(url) {
            img.data = Some(cached);
            return;
        }
        spawn_background_fetch(url.to_string());
    }

    fn spawn_background_fetch(url: String) {
        {
            let mut guard = IN_FLIGHT.lock().unwrap_or_else(|e| e.into_inner());
            let set = guard.get_or_insert_with(std::collections::HashSet::new);
            if !set.insert(url.clone()) {
                return;
            }
        }
        std::thread::Builder::new()
            .name("widget-image-prefetch".into())
            .spawn(move || {
                match fetch_as_data_uri(&url) {
                    Ok(data_uri) => cache_put(&url, data_uri),
                    Err(err) => log::warn!("image.url prefetch failed for {url}: {err}"),
                }
                if let Ok(mut guard) = IN_FLIGHT.lock() {
                    if let Some(set) = guard.as_mut() {
                        set.remove(&url);
                    }
                }
            })
            .ok();
    }

    fn cache_get(url: &str) -> Option<String> {
        let mut guard = memory();
        let map = guard.get_or_insert_with(HashMap::new);
        if let Some(e) = map.get(url) {
            if e.fetched_at.elapsed() < CACHE_TTL {
                return Some(e.data_uri.clone());
            }
            map.remove(url);
        }
        disk_cache_get(url).map(|data_uri| {
            map.insert(
                url.into(),
                CacheEntry {
                    data_uri: data_uri.clone(),
                    fetched_at: Instant::now(),
                },
            );
            data_uri
        })
    }

    pub(super) fn cache_put(url: &str, data_uri: String) {
        {
            let mut guard = memory();
            let map = guard.get_or_insert_with(HashMap::new);
            map.insert(
                url.into(),
                CacheEntry {
                    data_uri: data_uri.clone(),
                    fetched_at: Instant::now(),
                },
            );
        }
        disk_cache_put(url, &data_uri);
    }

    fn disk_cache_dir() -> Option<std::path::PathBuf> {
        let base = dirs_next_cache().unwrap_or_else(std::env::temp_dir);
        let dir = base.join("tauri-plugin-widgets").join("image-prefetch");
        std::fs::create_dir_all(&dir).ok()?;
        Some(dir)
    }

    fn dirs_next_cache() -> Option<std::path::PathBuf> {
        #[cfg(target_os = "macos")]
        {
            std::env::var_os("HOME").map(|h| {
                std::path::PathBuf::from(h)
                    .join("Library")
                    .join("Caches")
            })
        }
        #[cfg(target_os = "windows")]
        {
            std::env::var_os("LOCALAPPDATA").map(std::path::PathBuf::from)
        }
        #[cfg(all(unix, not(target_os = "macos")))]
        {
            std::env::var_os("XDG_CACHE_HOME")
                .map(std::path::PathBuf::from)
                .or_else(|| {
                    std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".cache"))
                })
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows", unix)))]
        {
            None
        }
    }

    fn cache_key(url: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        url.hash(&mut h);
        format!("{:016x}", h.finish())
    }

    fn disk_cache_get(url: &str) -> Option<String> {
        let dir = disk_cache_dir()?;
        let path = dir.join(cache_key(url));
        let meta_path = path.with_extension("meta");
        let meta = std::fs::read_to_string(&meta_path).ok()?;
        let ts: u64 = meta.trim().parse().ok()?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()?
            .as_secs();
        if now.saturating_sub(ts) > CACHE_TTL.as_secs() {
            let _ = std::fs::remove_file(&path);
            let _ = std::fs::remove_file(&meta_path);
            return None;
        }
        std::fs::read_to_string(path).ok()
    }

    fn disk_cache_put(url: &str, data_uri: &str) {
        let Some(dir) = disk_cache_dir() else {
            return;
        };
        let path = dir.join(cache_key(url));
        let meta_path = path.with_extension("meta");
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let _ = std::fs::write(&path, data_uri);
        let _ = std::fs::write(&meta_path, ts.to_string());
    }

    fn fetch_as_data_uri(url: &str) -> Result<String, String> {
        use base64::{engine::general_purpose::STANDARD as B64, Engine};
        use std::io::Read;

        let resp = ureq::get(url)
            .timeout(TIMEOUT)
            .call()
            .map_err(|e| e.to_string())?;
        let content_type = resp
            .header("content-type")
            .unwrap_or("image/png")
            .split(';')
            .next()
            .unwrap_or("image/png")
            .trim()
            .to_string();
        if !content_type.starts_with("image/") && content_type != "application/octet-stream" {
            return Err(format!("unexpected content-type {content_type}"));
        }
        let mut buf = Vec::new();
        let mut reader = resp.into_reader();
        reader
            .by_ref()
            .take(MAX_BYTES as u64 + 1)
            .read_to_end(&mut buf)
            .map_err(|e| e.to_string())?;
        if buf.is_empty() {
            return Err("empty body".into());
        }
        if buf.len() > MAX_BYTES {
            return Err("image exceeds 3 MiB limit".into());
        }
        let mime = if content_type.starts_with("image/") {
            content_type
        } else {
            guess_mime(url).into()
        };
        Ok(format!("data:{mime};base64,{}", B64.encode(&buf)))
    }

    fn guess_mime(url: &str) -> &'static str {
        let lower = url.to_ascii_lowercase();
        if lower.contains(".jpg") || lower.contains(".jpeg") {
            "image/jpeg"
        } else if lower.contains(".webp") {
            "image/webp"
        } else if lower.contains(".gif") {
            "image/gif"
        } else {
            "image/png"
        }
    }
}

#[cfg(all(feature = "image-prefetch", not(target_os = "android")))]
use imp::prefetch_remote_images_inner;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ImageElement;

    #[test]
    fn leaves_non_http_url_alone() {
        let mut cfg = WidgetConfig {
            version: 1,
            small: Some(WidgetElement::Image(ImageElement {
                system_name: None,
                data: None,
                url: Some("data:image/png;base64,QQ==".into()),
                size: Some(16.0),
                color: None,
                content_mode: None,
                style: Default::default(),
            })),
            medium: None,
            large: None,
        };
        prefetch_remote_images(&mut cfg);
        match cfg.small.as_ref().unwrap() {
            WidgetElement::Image(img) => {
                assert_eq!(img.url.as_deref(), Some("data:image/png;base64,QQ=="));
                assert!(img.data.is_none());
            }
            _ => panic!("expected image"),
        }
    }

    #[test]
    fn skips_when_data_already_set() {
        let mut cfg = WidgetConfig {
            version: 1,
            small: Some(WidgetElement::Image(ImageElement {
                system_name: None,
                data: Some("data:image/png;base64,QQ==".into()),
                url: Some("https://example.com/a.png".into()),
                size: None,
                color: None,
                content_mode: None,
                style: Default::default(),
            })),
            medium: None,
            large: None,
        };
        prefetch_remote_images(&mut cfg);
        match cfg.small.as_ref().unwrap() {
            WidgetElement::Image(img) => {
                assert_eq!(img.url.as_deref(), Some("https://example.com/a.png"));
                assert!(img.data.as_ref().unwrap().starts_with("data:"));
            }
            _ => panic!("expected image"),
        }
    }

    #[test]
    fn keeps_url_when_serving_from_memory_cache() {
        #[cfg(all(feature = "image-prefetch", not(target_os = "android")))]
        {
            imp::cache_put(
                "https://example.com/cached.png",
                "data:image/png;base64,QQ==".into(),
            );
            let mut cfg = WidgetConfig {
                version: 1,
                small: Some(WidgetElement::Image(ImageElement {
                    system_name: None,
                    data: None,
                    url: Some("https://example.com/cached.png".into()),
                    size: None,
                    color: None,
                    content_mode: None,
                    style: Default::default(),
                })),
                medium: None,
                large: None,
            };
            prefetch_remote_images(&mut cfg);
            match cfg.small.as_ref().unwrap() {
                WidgetElement::Image(img) => {
                    assert_eq!(img.url.as_deref(), Some("https://example.com/cached.png"));
                    assert_eq!(img.data.as_deref(), Some("data:image/png;base64,QQ=="));
                }
                _ => panic!("expected image"),
            }
        }
    }
}
