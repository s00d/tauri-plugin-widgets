# ADR 0005: Queued taps wait for a JavaScript listener

Status: Accepted

## Context

A tap on a widget that launches the app is queued (`pending_actions`). The host drained the
queue early — mobile on `RunEvent::Ready`, macOS from a 500 ms poller — and emitted
`widget-action` before the webview had loaded and subscribed. The tap was removed from
the queue and lost.

## Decision

`store::ListenerGate`: the host keeps the queue until JavaScript shows it listens — its
first `poll_pending_actions` call, which `onWidgetAction` now makes right after
subscribing (delivering queued taps once through `widget-action`). The gate opens on that
call even if the poll itself fails. Apps that only `listen("widget-action")` are covered by
a fallback: the gate opens 10 s after start. On mobile a drain is scheduled for that moment
after a cold start; the desktop macOS poller checks the gate every 500 ms.

## Consequences

- Cold-start taps arrive exactly once with `onWidgetAction`.
- Re-emitted actions carry the delivery time as `ts`, not the tap time.
