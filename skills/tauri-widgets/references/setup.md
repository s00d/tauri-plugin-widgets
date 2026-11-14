# Setup

```bash
pnpm tauri add tauri-plugin-widgets
pnpm add tauri-plugin-widgets-api
npx tauri-widgets init        # detects platforms → init-ios / init-macos / init-windows
npx tauri-widgets doctor      # checks App Groups, signing, transport
```

- App Group: `plugins.widgets.appGroup` (default `group.<identifier>`); same group in the
  app and the extension entitlements.
- macOS transport: `widgetContainer` by default (works ad-hoc), `appGroup` for sandboxed
  App Store builds (`npx tauri-widgets signing --apply`).
- Android: provider sizes in `android/src/main/res/xml`; `setRegisterWidget` / `requestWidget` to pin.
- Debug delivery: `WIDGET_DEBUG=1`, `npx tauri-widgets trace --follow`, `getWidgetDiagnostics()`.
Docs: https://s00d.github.io/tauri-plugin-widgets/guide/setup
