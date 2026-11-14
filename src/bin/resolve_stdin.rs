//! Test oracle for the TypeScript / Swift / Kotlin ports: reads JSON lines
//! `{"config":…, "ctx":{"nowMs","locale","instance","data","state"}}` from stdin and prints one line
//! per case: `{"resolved", "nextChangeMs", "source"}` computed by `src/resolve.rs`.

use std::io::{BufRead, Write};
use tauri_plugin_widgets::resolve::{
    merge_data, next_change_ms, resolve, source_request, ResolveContext,
};

fn main() {
    let stdin = std::io::stdin();
    let mut out = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line.expect("stdin");
        if line.trim().is_empty() {
            continue;
        }
        let case: serde_json::Value = serde_json::from_str(&line).expect("case json");
        let config = &case["config"];
        let c = &case["ctx"];
        let data = merge_data([config.get("data"), c.get("data")]);
        let state = merge_data([config.get("state"), c.get("state")]);
        let ctx = ResolveContext {
            now_ms: c["nowMs"].as_u64().unwrap_or(0),
            locale: c["locale"].as_str().unwrap_or(""),
            instance: c.get("instance").and_then(serde_json::Value::as_str),
            data: data.as_ref(),
            state: state.as_ref(),
        };
        let source = source_request(config, &ctx).map(|s| {
            serde_json::json!({
                "url": s.url, "method": s.method, "body": s.body,
                "refreshMinutes": s.refresh_minutes,
                "headers": s.headers.into_iter().collect::<std::collections::BTreeMap<_, _>>(),
            })
        });
        let row = serde_json::json!({
            "resolved": resolve(config, &ctx),
            "nextChangeMs": next_change_ms(config, ctx.now_ms),
            "source": source,
        });
        writeln!(out, "{row}").expect("stdout");
    }
}
