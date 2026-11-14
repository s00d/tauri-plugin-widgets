# Architecture decision records

One file per decision that shapes more than one platform. Format: **Context** (what exists
and why it is not enough), **Decision**, **Consequences** (costs, limits, what is left out).
Status is `Accepted`, `Proposed` or `Superseded by NNNN`. Number new records in order; do
not rewrite accepted ones — supersede them.

| # | Decision |
| --- | --- |
| [0001](0001-render-time-resolution.md) | One render-time contract (`entries`, `{{…}}`) with four ports and shared fixtures |
| [0002](0002-placement-instances.md) | Per-placement instances: `widgetId#instance` keys |
| [0003](0003-widget-fetched-source.md) | Widgets fetch their own JSON (`source`), cached outside the config map |
| [0004](0004-live-activities.md) | One live-activity request for iOS, Android and desktop |
| [0005](0005-action-listener-gate.md) | Queued taps wait for a JavaScript listener |
| [0006](0006-mutation-testing.md) | Mutation testing for every language, one tool at a time |
