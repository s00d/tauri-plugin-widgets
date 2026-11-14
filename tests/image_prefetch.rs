//! Stories: remote images in a config are fetched by the host before the store write
//! (WidgetKit cannot fetch), through `setWidgetConfig`.
#![cfg(all(
    feature = "image-prefetch",
    not(any(target_os = "android", target_os = "ios"))
))]

mod common;
use common::*;

use std::io::{Read as _, Write as _};
use std::net::TcpListener;

/// Local HTTP server answering `hits` requests with a PNG body; returns its base URL.
fn serve_png(hits: usize) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for stream in listener.incoming().take(hits) {
            let mut s = stream.unwrap();
            let mut req = [0u8; 2048];
            let _ = s.read(&mut req);
            let body = [9u8; 10];
            let head = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = s.write_all(head.as_bytes());
            let _ = s.write_all(&body);
        }
    });
    format!("http://127.0.0.1:{port}")
}

/// Given a config with an image (nested in a stack) at a loopback http URL,
/// When the app sets the config,
/// Then the stored image carries the fetched bytes as a `data:` URI and keeps its `url`.
#[test]
fn remote_image_is_embedded_before_the_write() {
    let app = App::new();
    let url = format!("{}/pic.png", serve_png(1));
    let config = json!({"version": 1, "small": {"type": "vstack", "children": [
        {"type": "image", "url": url}
    ]}});
    app.ok(
        "set_widget_config",
        json!({ "widgetId": "prefetch-ok", "config": config, "skipReload": true }),
    );
    let img =
        &app.ok("get_widget_config", json!({ "widgetId": "prefetch-ok" }))["small"]["children"][0];
    assert_eq!(img["url"], json!(url));
    let data = img["data"].as_str().expect("image data filled");
    assert!(data.starts_with("data:image/png;base64,"), "{data}");
}

/// Given images the host must not fetch (a non-http URL; an image whose `data` is set),
/// When the app sets the config,
/// Then both are stored as sent.
#[test]
fn non_http_and_filled_images_are_left_alone() {
    let app = App::new();
    let config = json!({"version": 1,
        "small": {"type": "image", "url": "data:image/png;base64,QQ==", "size": 16},
        "medium": {"type": "image", "url": "https://example.invalid/a.png",
                   "data": "data:image/png;base64,QQ=="}});
    app.ok(
        "set_widget_config",
        json!({ "widgetId": "prefetch-skip", "config": config, "skipReload": true }),
    );
    let cfg = app.ok("get_widget_config", json!({ "widgetId": "prefetch-skip" }));
    assert_eq!(cfg["small"]["url"], json!("data:image/png;base64,QQ=="));
    assert!(cfg["small"].get("data").is_none(), "{cfg}");
    assert_eq!(cfg["medium"]["url"], json!("https://example.invalid/a.png"));
    assert_eq!(cfg["medium"]["data"], json!("data:image/png;base64,QQ=="));
}
