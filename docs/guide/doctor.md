---
title: Doctor
---

# `tauri-widgets doctor`

Static + optional runtime checks for widget transport, App Groups, signing, bundle layout, receipt schema, and the host trace journal.

```bash
pnpm tauri-widgets doctor
# or against an app checkout:
pnpm tauri-widgets doctor /path/to/my-app
```

## What it checks

| Section | Passes when |
| --- | --- |
| **transport** | `tauri.conf.json` found; `plugins.widgets.appGroup` set; warns on `auto` / `widgetContainer` misuse |
| **entitlements** | `.entitlements` under `src-tauri` list the same App Group as conf |
| **app group (3 places)** | conf + entitlements + project files (`Info.plist` / `.pbxproj` / widget targets / `gen/apple`) mention the group |
| **signing** | Keychain identities (before build): `appGroup` without Team ID → error; `widgetContainer` with Team ID → warn. Also inspects built `.app` when present |
| **bundle** | `.appex` present in release artifacts (warn if not built yet) |
| **runtime (trace)** | Reads `widget_trace.json`; fails on throttled reloads; warns near WidgetKit daily budget |
| **plugin-config** | `plugins.widgets` shape (transport enum, appGroup required) |
| **schema** | Rust / Swift / Kotlin receipt `schema` defaults still match (`1`) |

Related commands:

```bash
npx tauri-widgets signing                     # identities + transport verdict
npx tauri-widgets signing --apply             # write plugins.widgets + macos entitlements
npx tauri-widgets signing --write-entitlements
npx tauri-widgets signing --dev-cert          # Windows DevCert.ps1
```

macOS `build-widget.sh` regenerates `App.entitlements` / `TauriWidgetExtension.entitlements` from `plugins.widgets.appGroup` on every build (files are gitignored after `init-macos`).

Exit code `1` when any ✗ error is reported; ⚠ warnings alone still exit `0`.

## When to run it

- After `init-macos` / `init-ios`
- Before publishing / filing a “widget blank / no update” bug
- When investigating throttled reloads (`getWidgetTrace` / `WIDGET_DEBUG=1`)

See also: [Troubleshooting](/guide/troubleshooting) · [Transport](/guide/transport) · [JS API — getWidgetTrace](/api/js).
