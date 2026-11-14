//! Prefetch remote `image.url` into `data:` URIs for hosts that cannot fetch
//! (WidgetKit). Android keeps its own `localPath` pipeline — Rust prefetch is a
//! no-op there so we do not inflate SharedPreferences / Binder with base64.
//!
//! Behind the `image-prefetch` feature (`ureq`). In-memory + disk cache by URL
//! with TTL so `startWidgetUpdater` does not hit the network every tick.
//! Cache hits are instant; cold misses fetch synchronously (≤ TIMEOUT) so the
//! store write from `set_widget_config` already embeds `data:` for WidgetKit.
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
    /// A failed URL is not retried before this; otherwise every updater tick waits on it.
    const FAILURE_TTL: Duration = Duration::from_secs(60);
    /// Memory cache bound: cache-busting URLs (`?t=…`) must not grow the heap forever.
    const MAX_ENTRIES: usize = 64;

    #[derive(Clone)]
    struct CacheEntry {
        data_uri: String,
        fetched_at: Instant,
    }

    static MEMORY: Mutex<Option<HashMap<String, CacheEntry>>> = Mutex::new(None);
    static FAILED: Mutex<Option<HashMap<String, Instant>>> = Mutex::new(None);

    fn recently_failed(url: &str) -> bool {
        let mut guard = FAILED.lock().unwrap_or_else(|e| e.into_inner());
        let map = guard.get_or_insert_with(HashMap::new);
        map.retain(|_, at| at.elapsed() < FAILURE_TTL);
        map.contains_key(url)
    }

    /// Drop expired entries, then the oldest ones, down to `MAX_ENTRIES - 1`.
    fn evict(map: &mut HashMap<String, CacheEntry>) {
        map.retain(|_, e| e.fetched_at.elapsed() < CACHE_TTL);
        while map.len() >= MAX_ENTRIES {
            let Some(oldest) = map
                .iter()
                .min_by_key(|(_, e)| e.fetched_at)
                .map(|(k, _)| k.clone())
            else {
                break;
            };
            map.remove(&oldest);
        }
    }

    fn memory() -> std::sync::MutexGuard<'static, Option<HashMap<String, CacheEntry>>> {
        MEMORY.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub(super) fn prefetch_remote_images_inner(config: &mut WidgetConfig) {
        for el in config.layouts_mut() {
            walk_el(el);
        }
    }

    fn walk_el(el: &mut WidgetElement) {
        match el {
            WidgetElement::Image(img) => prefetch_image(img),
            WidgetElement::Repeat(r) => walk_el(&mut r.template),
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
            WidgetElement::List(v) => {
                for c in &mut v.children {
                    walk_el(c);
                }
            }
            WidgetElement::Button(_)
            | WidgetElement::Toggle(_)
            | WidgetElement::Text(_)
            | WidgetElement::Label(_)
            | WidgetElement::Use(_)
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
        // Hot path: memory/disk only.
        if let Some(cached) = cache_get(url) {
            img.data = Some(cached);
            return;
        }
        if recently_failed(url) {
            return;
        }
        // Cold miss: fetch now so the upcoming store write includes `data:`.
        // WidgetKit cannot HTTP-fetch; a background-only fill never rewrites the store.
        match fetch_as_data_uri(url) {
            Ok(data_uri) => {
                cache_put(url, data_uri.clone());
                img.data = Some(data_uri);
            }
            Err(err) => {
                log::warn!("image.url prefetch failed for {url}: {err}");
                FAILED
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .get_or_insert_with(HashMap::new)
                    .insert(url.into(), Instant::now());
            }
        }
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
        disk_cache_get(url).inspect(|data_uri| {
            evict(map);
            map.insert(
                url.into(),
                CacheEntry {
                    data_uri: data_uri.clone(),
                    fetched_at: Instant::now(),
                },
            );
        })
    }

    pub(super) fn cache_put(url: &str, data_uri: String) {
        {
            let mut guard = memory();
            let map = guard.get_or_insert_with(HashMap::new);
            evict(map);
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
            std::env::var_os("HOME")
                .map(|h| std::path::PathBuf::from(h).join("Library").join("Caches"))
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
        let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
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
        // Expired entries are otherwise only removed when the same URL is asked again.
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for e in entries.flatten() {
                let old = e
                    .metadata()
                    .and_then(|m| m.modified())
                    .ok()
                    .and_then(|t| t.elapsed().ok())
                    .is_some_and(|age| age > CACHE_TTL);
                if old {
                    let _ = std::fs::remove_file(e.path());
                }
            }
        }
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
    #[cfg(test)]
    mod imp_tests {
        use super::*;
        use std::io::{Read as _, Write as _};
        use std::net::TcpListener;

        /// One-shot local HTTP server: answers every request with `content_type` + `body`.
        fn serve(content_type: &'static str, body: Vec<u8>, hits: usize) -> String {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let port = listener.local_addr().unwrap().port();
            std::thread::spawn(move || {
                for stream in listener.incoming().take(hits) {
                    let mut s = stream.unwrap();
                    let mut req = [0u8; 2048];
                    let _ = s.read(&mut req);
                    let head = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        body.len()
                    );
                    let _ = s.write_all(head.as_bytes());
                    let _ = s.write_all(&body);
                }
            });
            format!("http://127.0.0.1:{port}")
        }

        fn decoded_len(uri: &str) -> usize {
            use base64::Engine;
            let b64 = uri.split_once(',').unwrap().1;
            base64::engine::general_purpose::STANDARD
                .decode(b64)
                .unwrap()
                .len()
        }

        fn entry(age: Duration) -> CacheEntry {
            CacheEntry {
                data_uri: "d".into(),
                fetched_at: Instant::now().checked_sub(age).unwrap(),
            }
        }

        fn age_memory(url: &str, age: Duration) {
            memory()
                .get_or_insert_with(HashMap::new)
                .get_mut(url)
                .expect("cached in memory")
                .fetched_at = Instant::now().checked_sub(age).unwrap();
        }

        /// Fetch rules: the 3 MiB limit (2 MiB and exactly 3 MiB pass, one byte more fails),
        /// image content types only (octet-stream guesses the type from the URL), empty
        /// bodies fail, and the URL-based MIME guess table.
        #[test]
        fn fetch_limits_types_and_mime_guess() {
            // 2 MiB must pass: catches a limit computed wrong (e.g. 3 + 1024 * 1024).
            let base = serve("image/png", vec![1u8; 2 * 1024 * 1024], 1);
            assert!(fetch_as_data_uri(&format!("{base}/a.png")).is_ok());
            let base = serve("image/png; charset=x", vec![1u8; MAX_BYTES], 1);
            let uri = fetch_as_data_uri(&format!("{base}/a.png")).unwrap();
            assert!(uri.starts_with("data:image/png;base64,"), "{}", &uri[..30]);
            assert_eq!(decoded_len(&uri), MAX_BYTES);
            let base = serve("image/png", vec![1u8; MAX_BYTES + 1], 1);
            assert!(fetch_as_data_uri(&format!("{base}/a.png"))
                .unwrap_err()
                .contains("3 MiB"));

            let base = serve("text/html", b"<html>".to_vec(), 1);
            assert!(fetch_as_data_uri(&format!("{base}/a.png"))
                .unwrap_err()
                .contains("content-type"));
            let base = serve("application/octet-stream", vec![7u8; 5000], 1);
            let uri = fetch_as_data_uri(&format!("{base}/pic.JPG")).unwrap();
            assert!(uri.starts_with("data:image/jpeg;base64,"));
            assert_eq!(decoded_len(&uri), 5000);
            let base = serve("image/gif", Vec::new(), 1);
            assert!(fetch_as_data_uri(&format!("{base}/a.gif"))
                .unwrap_err()
                .contains("empty"));

            assert_eq!(guess_mime("x/a.jpg"), "image/jpeg");
            assert_eq!(guess_mime("x/a.JPEG?x=1"), "image/jpeg");
            assert_eq!(guess_mime("x/a.webp"), "image/webp");
            assert_eq!(guess_mime("x/a.gif"), "image/gif");
            assert_eq!(guess_mime("x/a"), "image/png");
        }

        /// One URL through its whole life. The stub server answers once, so every later
        /// fill proves it came from a cache: fetch → memory hit (through the public config
        /// walk, nested, `url` kept) → memory stale → disk hit → disk stale → refetch fails →
        /// failure remembered (no retry, blank `data` not counted as filled) → failure expires.
        #[test]
        fn url_lifecycle_fetch_cache_stale_failure() {
            // 1. Cold miss: fetched now (URL trimmed), cached in memory and on disk.
            let base = serve("image/png", vec![9u8; 10], 1);
            let url = format!("{base}/ok.png");
            let mut img = ImageElement {
                url: Some(format!(" {url} ")),
                ..Default::default()
            };
            prefetch_image(&mut img);
            let data = img.data.clone().expect("filled");
            assert!(data.starts_with("data:image/png"), "{data}");
            let disk = disk_cache_dir().unwrap().join(cache_key(&url));
            assert!(disk.exists() && disk.with_extension("meta").exists());

            // 2. Memory hit via the public walk: nested in a stack, `url` stays for Android/AC.
            let mut cfg: WidgetConfig = serde_json::from_value(serde_json::json!({
                "small": {"type": "vstack", "children": [{"type": "image", "url": url}]}
            }))
            .unwrap();
            prefetch_remote_images(&mut cfg);
            let Some(WidgetElement::VStack(v)) = &cfg.small else {
                panic!("vstack")
            };
            let WidgetElement::Image(got) = &v.children[0] else {
                panic!("image")
            };
            assert_eq!(got.url.as_deref(), Some(url.as_str()));
            assert_eq!(got.data.as_deref(), Some(data.as_str()));

            // 3. Memory copy stale → served from disk and put back in memory.
            age_memory(&url, CACHE_TTL + Duration::from_secs(1));
            assert_eq!(cache_get(&url).as_deref(), Some(data.as_str()));
            assert!(memory().as_ref().unwrap()[&url].fetched_at.elapsed() < CACHE_TTL);

            // 4. Both stale → miss; the dead server fails the refetch: nothing filled, the
            //    stale disk files are gone and the failure is remembered.
            age_memory(&url, CACHE_TTL + Duration::from_secs(1));
            std::fs::write(disk.with_extension("meta"), "0").unwrap();
            let mut again = ImageElement {
                url: Some(url.clone()),
                ..Default::default()
            };
            prefetch_image(&mut again);
            assert_eq!(again.data, None);
            assert!(!disk.exists());
            assert!(recently_failed(&url));
            assert!(!recently_failed(&format!("{base}/other.png")));

            // 5. Blank `data` is not "filled": it is attempted (and fails) like an empty one.
            let mut blank = ImageElement {
                url: Some("http://127.0.0.1:1/nothing.png".into()),
                data: Some("  ".into()),
                ..Default::default()
            };
            prefetch_image(&mut blank);
            assert_eq!(blank.data.as_deref(), Some("  "));
            assert!(recently_failed("http://127.0.0.1:1/nothing.png"));

            // 6. The failure expires after FAILURE_TTL.
            FAILED.lock().unwrap().as_mut().unwrap().insert(
                url.clone(),
                Instant::now()
                    .checked_sub(FAILURE_TTL + Duration::from_secs(1))
                    .unwrap(),
            );
            assert!(!recently_failed(&url));
        }

        /// Cache bookkeeping: memory eviction (expired first, then oldest, below the cap) →
        /// memory-only entries are served → disk round trip, expiry and stable hashed keys
        /// under the OS cache dir → the disk sweep removes only old files.
        #[test]
        fn cache_eviction_disk_store_and_sweep() {
            // 1. Eviction.
            let mut map = HashMap::new();
            map.insert("old".to_string(), entry(CACHE_TTL + Duration::from_secs(1)));
            map.insert("fresh".to_string(), entry(Duration::from_secs(1)));
            evict(&mut map);
            assert!(map.contains_key("fresh") && !map.contains_key("old"));
            let mut full = HashMap::new();
            for i in 0..MAX_ENTRIES {
                full.insert(format!("k{i}"), entry(Duration::from_secs(10 + i as u64)));
            }
            evict(&mut full);
            assert_eq!(full.len(), MAX_ENTRIES - 1);
            assert!(!full.contains_key(&format!("k{}", MAX_ENTRIES - 1)));
            assert!(full.contains_key("k0"));

            // 2. Memory-only entry (no disk copy) and a put → get.
            let tag = format!("{}-{:?}", std::process::id(), Instant::now());
            let mem = format!("http://cache.test/mem-{tag}.png");
            memory()
                .get_or_insert_with(HashMap::new)
                .insert(mem.clone(), entry(Duration::from_secs(1)));
            assert_eq!(cache_get(&mem).as_deref(), Some("d"));
            let put = format!("http://cache.test/put-{tag}.png");
            cache_put(&put, "data:image/png;base64,AA==".into());
            assert_eq!(
                cache_get(&put).as_deref(),
                Some("data:image/png;base64,AA==")
            );

            // 3. Disk store: location, round trip, expiry removes the file, keys.
            let base = dirs_next_cache().expect("cache base");
            #[cfg(target_os = "macos")]
            assert!(base.ends_with("Library/Caches"), "{base:?}");
            #[cfg(not(target_os = "macos"))]
            assert!(base.is_absolute() && base.as_os_str().len() > 1, "{base:?}");
            let dir = disk_cache_dir().unwrap();
            assert!(dir.ends_with("tauri-plugin-widgets/image-prefetch"));
            let url = format!("http://disk.test/{}.png", std::process::id());
            disk_cache_put(&url, "data:x");
            assert_eq!(disk_cache_get(&url).as_deref(), Some("data:x"));
            let path = dir.join(cache_key(&url));
            std::fs::write(path.with_extension("meta"), "0").unwrap();
            assert_eq!(disk_cache_get(&url), None);
            assert!(!path.exists());
            assert_eq!(cache_key("a").len(), 16);
            assert_eq!(cache_key("a"), cache_key("a"));
            assert_ne!(cache_key("a"), cache_key("b"));

            // 4. Sweep on put: files older than the TTL go, fresh ones stay.
            let old = dir.join(format!("sweep-old-{}", std::process::id()));
            let fresh = dir.join(format!("sweep-fresh-{}", std::process::id()));
            std::fs::write(&old, "x").unwrap();
            std::fs::write(&fresh, "x").unwrap();
            let f = std::fs::File::options().write(true).open(&old).unwrap();
            f.set_modified(SystemTime::now() - CACHE_TTL - Duration::from_secs(5))
                .unwrap();
            disk_cache_put(
                &format!("http://disk.test/sweep-{}.png", std::process::id()),
                "data:y",
            );
            assert!(!old.exists());
            assert!(fresh.exists());
            let _ = std::fs::remove_file(fresh);
        }
    }
}

#[cfg(all(feature = "image-prefetch", not(target_os = "android")))]
use imp::prefetch_remote_images_inner;
