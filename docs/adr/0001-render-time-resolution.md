# ADR 0001: One render-time contract with four ports

Status: Accepted

## Context

Until 0.7 a widget showed exactly the JSON the app last wrote. Anything time- or
device-dependent (a schedule, the device language, fresh data) needed the app running to
rewrite the config. WidgetKit, Glance, the desktop webview and Adaptive Cards draw from
four codebases in four languages; a feature in one of them drifts from the others.

## Decision

`WidgetConfig` gains `entries`, `strings`, `defaultLocale`, `data` and `source`. Each
renderer runs the same **resolution** on the stored JSON right before decoding, with what
only the device knows (time, locale, placement, fetched data):

1. active entry: last `at <= now`, per size family;
2. placeholders in every string: `{{t.key}}`, `{{data.path}}`, `{{locale}}`, `{{instance}}`;
3. output keeps only `version` and the size families, so existing decoders are untouched.

`src/resolve.rs` is the reference. `widget-src/resolve.ts`, `swift/…/Resolve.swift` and
`android/…/Resolve.kt` are line-by-line ports. `tests/fixtures/resolve/*.json` (config,
context, expected output, next change, source request) run in all four test suites; a
contract change starts with a fixture.

Resolution works on JSON values, not typed models, so it needs no codegen and survives
unknown fields.

## Consequences

- Placeholders fill **strings only**. `"value": "{{data.p}}"` would fail typed decoding on
  the host; numeric binding is left out until there is demand (upgrade path: resolve
  before the host's typed parse, accept string-or-number in the schema).
- Locale fallback order and number formatting (`21.0` → `21`, like JavaScript) are part of
  the contract and fixed by fixtures; a port that formats differently fails its tests.
- Scheduling differs per OS (WidgetKit timeline, Android inexact alarm, desktop timer,
  host rebuild for the Widgets Board) — see the capability matrix.
