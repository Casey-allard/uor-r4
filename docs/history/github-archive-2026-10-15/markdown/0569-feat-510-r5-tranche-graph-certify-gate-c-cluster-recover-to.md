# #569 — feat(#510): R5 tranche — graph-certify gate-C cluster + recover to total; graph-certify at 0

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-10T02:35:33Z
- **Updated:** 2026-10-09T13:09:00Z
- **Closed:** 2026-08-10T02:54:17Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/569
- **Merged:** 2026-08-10T02:54:17Z
- **Merge commit:** 93daf4d555cf7289de1d4b8440aa5326f2d30ef2
- **Base / Head:** main / UOR-Foundation:r5/gate-c-recover-total
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## R5 tranche (#510): graph-certify gate-C cluster + recover → total — **graph-certify at 0**

Converts the last five graph-certify `Result` surfaces, all in `score.rs`:

- **`generate_greedy_repetition_rate` → `f64`.** Mirrors
  `baseline_greedy_repetition_rate` (already bare); its scorer call became a
  self-produced-defect `.expect` in #567, so the wrapper `Result` carried nothing.
- **`evaluate_gate_c_row` → `GateCRow`.** With its scorer calls already `.expect`
  (#567), no fallible operation remained; the per-row Rayon map now collects into
  a plain `Vec`.
- **`evaluate_gate_c` → `Option<GateCOutcome>`.** The one real failure is an empty
  held-out split (nothing to measure) → `None`; the six self-produced scorer
  rebuilds stay `.expect`, and the greedy/row calls are now infallible.
- **`recover_from_artifact` → `Result<_, NotAProduct>`.** The `--cover` bytes are
  an external product: `GraphView::parse` propagates `NotAProduct` directly, a CID
  mismatch folds through `KappaError::as_format`, and a view that parses and
  verifies but yields no scoring regions is reported as a missing NODE section.

Callers adapted: graph-cli `cover_sweep` (×2) and `score_command` map the `Option`
to their `Result` CLI surface via `.ok_or_else`; the `--cover` path maps
`NotAProduct` via `.to_string()`.

### Audit — milestone
`graph-certify` audit-limits: **5 → 0**. Combined with #568 (`score_runtime.rs`
`step_*` → `Option`, already merged), **the entire graph-certify crate now names
only sanctioned `Result` surfaces** (`NotAProduct` / `ObservedBound` /
`KappaError` / `SourceUnavailable`) or total forms. Verified locally by rebasing
onto the #568 base: `audit-limits` reports zero graph-certify sites. Remaining
workspace: api (20), graph-cli (42).

### Gauntlet
`cargo fmt --check`, `cargo build --workspace --all-targets`, `clippy` (certify +
cli, all-targets), `wasm32-unknown-unknown` build of graph-certify, and the
graph-certify lib / `fwda_roundtrip` / graph-cli lib / `status_policy` (+census)
suites — all green.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

