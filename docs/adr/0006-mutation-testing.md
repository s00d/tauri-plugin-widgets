# ADR 0006: Mutation testing for every language

Status: Accepted

## Context

The plugin is Rust, TypeScript, Swift and Kotlin. Line coverage said little about whether
tests check results.

## Decision

- Rust: cargo-mutants (`.cargo/mutants.toml`), excluding code that only runs inside a
  Tauri app or on another OS. CI runs it on the lines a PR changes.
- TypeScript: Stryker with vitest on logic with unit tests (`widget-src/resolve.ts`).
- Kotlin: PIT, applied only with `-Ppitest=true` so apps that include the Android module
  never resolve the plugin.
- Swift: Muter on the package (`swift/muter.conf.yml`).
- Tools run one at a time; test speed is fixed first (dependencies optimized in the dev
  profile, a debug-info-free `mutants` profile).

## Consequences

Surviving mutants are fixed by better tests, not by excluding code. Rendering code is
covered by golden images, not by mutation runs.
