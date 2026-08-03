# Starter — preview a widget without a native build

```bash
npx degit s00d/tauri-plugin-widgets/templates/starter my-widget
cd my-widget
# from a checkout of the plugin, or after: pnpm add -g tauri-plugin-widgets-api
npx tauri-widgets preview ./widget.json --size medium --watch --open
```

Edit `widget.json` and the browser reloads.

Validate before wiring into an app:

```bash
npx tauri-widgets validate ./widget.json --platforms ios,android,desktop
```

## Next: real Tauri app

1. `pnpm create tauri-app` (or add the plugin to an existing app)
2. `pnpm tauri add tauri-plugin-widgets`
3. `npx tauri-widgets init`
4. `npx tauri-widgets signing --apply`
5. Call `setWidgetConfig` with this JSON (keep `$schema` for the editor)

Docs: https://s00d.github.io/tauri-plugin-widgets/
