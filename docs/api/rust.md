---
title: Rust API
---

# Rust API

This page is a narrative guide for calling the plugin from Rust. The full type
reference lives on [docs.rs/tauri-plugin-widgets](https://docs.rs/tauri-plugin-widgets/).

<div class="doc-illust">

![Host updating widgets from background Rust code](/illustrations/updating.jpg)

</div>

## When you need Rust

Use the Rust side when:

- a **background task** (timer, network, push) must refresh a widget without a webview
- you embed the plugin inside another Rust library / sidecar
- you want **compile-time** layout construction instead of hand-written JSON

You do **not** need Rust for ordinary JS/TS apps — `setWidgetConfig` from
`guest-js` is enough.

## Installation and init

```no_run
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_widgets::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Optional plugin config (`plugins.widgets` in `tauri.conf.json`) controls the
macOS transport (`appGroup` vs `widgetContainer`) and App Group id. See the
platform setup guides under Contributing.

## Building a config

### `json!` — the default path

Zero new types, copies straight from gallery / JS docs, checked only at runtime:

```rust
use tauri_plugin_widgets::models::WidgetConfig;

let cfg: WidgetConfig = serde_json::from_value(serde_json::json!({
    "small": {
        "type": "vstack",
        "padding": 12,
        "children": [
            { "type": "text", "content": "72°", "fontSize": 36, "fontWeight": "bold" }
        ]
    }
}))?;
```

### Typed structs — when you want the compiler

`WidgetElement` variants are newtypes over named structs, so `..Default::default()`
works:

```rust
use tauri_plugin_widgets::models::{
    text, vstack, ElementStyle, FontWeight, PaddingValue, TextElement, VStackElement,
    WidgetConfig, WidgetElement,
};

let cfg = WidgetConfig::small(WidgetElement::VStack(VStackElement {
    children: vec![WidgetElement::Text(TextElement {
        content: "72°".into(),
        font_size: Some(36.0),
        font_weight: Some(FontWeight::Bold),
        ..Default::default()
    })],
    spacing: Some(8.0),
    style: ElementStyle {
        padding: Some(PaddingValue::Uniform(12.0)),
        ..Default::default()
    },
    ..Default::default()
}));
```

Short helpers used in examples:

```rust
use tauri_plugin_widgets::models::{text, vstack, WidgetConfig};

let cfg = WidgetConfig::small(vstack(vec![
    text("72°").font_size(36.0).into(),
]));
```

## Updating from a background thread

Typical host path:

```no_run
use tauri::Manager;
use tauri_plugin_widgets::{
    models::{text, vstack, WidgetConfig},
    WidgetExt,
};

fn refresh(app: &tauri::AppHandle) -> tauri_plugin_widgets::Result<()> {
    let cfg = WidgetConfig::small(vstack(vec![text("72°").font_size(36.0).into()]));
    let w = app.widget();
    w.set_widget_config(&cfg, "group.com.example.app", "weather", false)?;
    w.reload_all_timelines()?;
    Ok(())
}
```

`set_widget_config` writes the IR into shared storage and returns `ApplyOutcome`
(`written`, `reload`, optional `skip`). A successful call is not proof that a native
reload ran — inspect `reload` (`Ok` / `Throttled` / `Skipped` / `Failed`).
`reload_all_timelines` asks WidgetKit / the desktop webview to repaint.

## Receiving actions

Two complementary paths:

1. Listen for the `widget-action` Tauri event (emitted when pending actions are drained).
2. Call `poll_pending_actions(group)` yourself — returns
   `Vec<WidgetActionEnvelope>` (typed, not raw `Value`).

## Diagnostics

`get_widget_diagnostics(group)` returns recent [`WidgetRenderReceipt`](https://docs.rs/tauri-plugin-widgets/latest/tauri_plugin_widgets/struct.WidgetRenderReceipt.html)
entries (what each surface actually painted, plus skipped nodes on Adaptive Cards).

## Errors

[`Error`](https://docs.rs/tauri-plugin-widgets/latest/tauri_plugin_widgets/enum.Error.html)
covers IO / JSON / unsupported operations. `Error::Unsupported` is the desktop
signal for APIs that only exist on mobile (see below).

## Platform asymmetry

| API | Android | iOS / macOS | Desktop |
|---|---|---|---|
| `set_register_widget` | stores provider FQCNs | stores WidgetKit kinds (advisory) | **no-op** (`Ok(true)`) |
| `request_widget` | pin-widget UI | bridge (may be OS no-op) | **`Err(Unsupported)`** — use `create_widget_window` |
| `create_widget_window` | unsupported | unsupported | creates a webview widget window |
| `reload_*_timelines` | AppWidget update | WidgetKit reload | emits `widget-reload` (+ macOS WidgetKit when linked) |

## Full reference

→ [docs.rs/tauri-plugin-widgets](https://docs.rs/tauri-plugin-widgets/)
