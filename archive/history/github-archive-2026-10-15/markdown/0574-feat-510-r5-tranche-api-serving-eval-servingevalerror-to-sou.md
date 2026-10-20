# #574 — feat(#510): R5 tranche — api serving_eval ServingEvalError to SourceUnavailable; api at 0

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-10T04:49:15Z
- **Updated:** 2026-10-09T13:09:03Z
- **Closed:** 2026-08-10T05:10:02Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/574
- **Merged:** 2026-08-10T05:10:02Z
- **Merge commit:** 629eacea8e939057375250444b44802ef88a4109
- **Base / Head:** main / UOR-Foundation:r5/api-serving-eval-source-unavailable
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## R5 tranche (#510): api `serving_eval` `ServingEvalError` → `SourceUnavailable` — **api at 0**

`evaluate_serving_bundle` measures the serving surface over a compiled host
bundle: it reads the bundle's files, parses the corpus pair, loads an engine, and
runs held-out decisions. Every failure means the host bundle could not be
evaluated — exactly `SourceUnavailable`. Replaces the composite `ServingEvalError`
enum with it:

- a missing/unreadable bundle file (`Io`) → `SourceUnavailable("<path>: …")`
- a corpus pair that doesn't parse or a non-UTF-8 path (`Corpus`) →
  `SourceUnavailable("corpus: …")`
- the engine-load rejection (`Load`) → **propagated directly**: `R4Engine::load`
  now returns `SourceUnavailable` (#573), so `?` carries its reason
- an empty held-out partition (`EmptyHeldOut`) → `SourceUnavailable`
- a serving decision hitting the token-vocabulary bound (`ObservedBound` from
  `predict_decision`) → folded into `SourceUnavailable` with context

The `ServingEvalError` enum and its `Display`/`Error` impls are removed. The only
consumer (the `r4` binary's `certify_serving_row`) prints the error via `Display`,
so folding is transparent — no caller change.

### Audit — milestone
api audit-limits: **1 → 0**. With graph-compiler and graph-certify already at 0,
**graph-compiler, graph-certify, and api now name only sanctioned `Result`
surfaces.** graph-cli (42) is the last crate.

### Gauntlet
`cargo fmt --check`, `cargo build --workspace --all-targets`, `clippy` (api,
all-targets), and the full api lib + integration suites — all green. (api is not a
wasm target.)

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

