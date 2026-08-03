# Maintainer scripts

Citty CLI (`pnpm -C scripts cli …`) plus shell/PowerShell stands.

## Consumer vs maintainer

| Audience | Entry | Lives in |
|----------|--------|----------|
| **App authors** | `tauri-widgets` / `npx tauri-widgets` (`dist-cli/cli.mjs`) | published package — `init`, `preview`, `signing`, `validate`, `doctor`, `trace`, `clean` |
| **Plugin maintainers** | `pnpm -C scripts cli …` (root aliases below) | this folder — stands, goldens, docs codegen |

Do not put consumer-facing commands under `scripts/`; keep them on the published bin. Conversely, visual-stand hosts and golden runners stay here and are not part of the npm API surface.

## Layout

| Path | Role |
|------|------|
| `src/` | TypeScript commands (docs, triage, hosts/shot/test, thin wrappers) |
| `sh/` | Bash stands: `*-up`, `shot-linux`, `build-example-artifacts`, `win-{bootstrap,pack,sideload}` |
| `win/` | Windows VM helpers (`sync.sh`, `*.ps1`) |
| `apple.env.example` | Template for local Apple/Android secrets → copy to `apple.env` (gitignored) |

## Consolidated maintainer commands

```bash
# Hosts (visual stand)
pnpm hosts up                 # all: ios android linux linux-wl win
pnpm hosts up linux win       # subset
pnpm hosts status             # simctl / adb / docker / ssh / widget-probe
pnpm hosts down               # docker rm -f wshot wshot-wl

# One-shot capture
pnpm shot weather.small              # linux via scripts/sh/shot-linux.sh (+ size from case JSON)
pnpm shot weather.small desktop      # prints just/pnpm commands for other platforms

# Test suites (alias avoids clashing with package pretest)
pnpm cli:test -- --platform desktop
pnpm cli:test -- --platform geometry --update
pnpm cli:test -- --platform android --case weather.small
```

Equivalent: `pnpm -C scripts cli hosts|shot|test …`.

## Other root aliases

`pnpm docs:generate`, `pnpm triage`, `pnpm ios-up`, `pnpm build:example`, `pnpm win-pack`, …  
Justfile recipes call the same `scripts/sh` / `scripts/win` paths (and still expose `just hosts`).
