# #568 — feat(#510): R5 tranche — graph-certify step_* scoring path to Option (certify 10->5)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-10T02:10:13Z
- **Updated:** 2026-10-09T13:09:04Z
- **Closed:** 2026-08-10T02:29:04Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/568
- **Merged:** 2026-08-10T02:29:04Z
- **Merge commit:** a3c665eb2430670e8f6ef0e8e42491d2391af6d5
- **Base / Head:** main / UOR-Foundation:r5/step-scoring-option
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## R5 tranche (#510): graph-certify `step_*` scoring path → `Option`

Converts the deployed allocation-free scoring-step methods in `score_runtime.rs`
to a total surface. Each operates on a validated scorer and caller-owned
`StepState`; a failure means the step cannot produce an outcome — a state that
cannot exist for this scorer (vocab bound, zero membership width, a mismatched
state, predicted-cloud emissions on, a legacy TLS1 store), a self-produced data
invariant (a node or token outside the emission tables / HEAD vocabulary), or an
empty candidate set.

- `step_state` → `Option<StepState>`
- `step_accumulate_node` → `Option<()>`
- `score_step` / `score_step_with_recent` / `score_step_coded_with_recent` →
  `Option<StepOutcome>`

### Callers adapted in-PR (api engine deployed serving path)
- `score_step_coded_with_recent` → `InferenceError::Scorer` via `.ok_or_else`.
- `R4Engine::load`'s `step_state` → `LoadError::Scorer` via `.ok_or_else`.
- `R4Engine::reset`'s `if let Ok(step)` → `if let Some(step)`.
- The `status_policy` probe suite's `step_state` / `score_step` /
  `score_step_with_recent` call sites are unchanged — they already use
  `Option`-compatible `.expect(...)`.

### Audit
`graph-certify` audit-limits: **10 → 5**. `score_runtime.rs` is now at **0**; the
remaining graph-certify sites are all in `score.rs` (the gate-C cluster +
`recover_from_artifact`). No sanctioned error types introduced — pure removal of
`Result<_, String>`.

### Gauntlet
`cargo fmt --check`, `cargo build --workspace --all-targets`, `clippy` (certify +
api, all-targets), `wasm32-unknown-unknown` build of graph-certify, and the
graph-certify lib / api lib / `status_policy` (+census) suites — all green.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

