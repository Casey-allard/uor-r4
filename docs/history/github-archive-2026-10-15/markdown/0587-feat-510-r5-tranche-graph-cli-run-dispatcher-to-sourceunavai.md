# #587 — feat(#510): R5 tranche — graph-cli run() dispatcher to SourceUnavailable; graph-cli at 0 (1->0)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-10T19:14:38Z
- **Updated:** 2026-10-09T13:09:03Z
- **Closed:** 2026-08-10T19:33:20Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/587
- **Merged:** 2026-08-10T19:33:19Z
- **Merge commit:** 22be057addf2c41592d0c4df09ab3cdd735bf8b1
- **Base / Head:** main / UOR-Foundation:r5/graph-cli-run-dispatcher
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## R5 tranche — graph-cli run() dispatcher → SourceUnavailable; graph-cli at 0

The final graph-cli site. Now that every command `run()` dispatches to returns a
sanctioned surface (`SourceUnavailable`, or `()` for the infallible demo
commands), `run()` itself becomes `Result<(), SourceUnavailable>`: the 12 per-arm
`.map_err(|e| e.to_string())` wrappers are dropped so each arm propagates its
`SourceUnavailable` via `?`; the `setup`/`gen`/`compile-native`/`store`/
`scenarios`/`teacher-kappa`/`cd-compile`/`quantum-eval`/help arms (which return
`()`) are unchanged.

Root edits: `src/main.rs`'s four `transformerless_command::run(...)` call sites
map `SourceUnavailable` back to `RunError::Command` via `to_string`.

audit-limits: graph-cli **1 → 0**. The **whole-repo audit is now clean**: every
shipped crate's `Result` error surface names only the four sanctioned types —
`NotAProduct` / `ObservedBound` / `KappaError` / `SourceUnavailable`. Gauntlet
green: graph-cli + root + api build, fmt/clippy/test; `xtask audit-limits` reports
"no shipped crate returns an unsanctioned error (R5)".

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

