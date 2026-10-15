# #567 — feat(#510): R5 tranche — graph-certify score_candidates path to Option (certify 17->10)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-10T01:46:53Z
- **Updated:** 2026-10-09T13:09:04Z
- **Closed:** 2026-08-10T02:05:29Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/567
- **Merged:** 2026-08-10T02:05:29Z
- **Merge commit:** 40161eb0d541f2f8e9b283e5130a644eab5d7bbd
- **Base / Head:** main / UOR-Foundation:r5/score-candidates-option
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## R5 tranche (#510): graph-certify `score_candidates` path → `Option`

Converts the witness-emitting candidate-scoring path in `score_runtime.rs` to a
total surface. Each function scores a signature against a validated
`GraphScorer`; a failure means no candidate distribution could be produced (an
empty candidate set, an unresolvable TLA6 graded code, or a propagated inner
`None`) — a total "no outcome" answer, not one of the sanctioned error
conditions.

- `score_candidates` / `score_candidates_coded` / `score_candidates_infill` →
  `Option<ScoreOutcome>`
- `score_candidates_legacy` → `Option<LegacyOutcome>`
- `probe_code` → `Option<[u8; STAGES]>`
- `two_pass_infill_generate` → `Option<TwoPassGeneration>`
- `infill_fill` → `Option<Vec<u32>>`

### Callers adapted in-PR
- **score.rs gate-C** rebuilds score the just-emitted self-produced artifact and
  its own observation sigs → `None` is a defect → `.expect("gate C: scoring
  self-produced sig")` (these fns keep their `Result` surface until the gate-C
  conversion; `.expect` is forward-safe).
- **`verify_witness_replay`** (already `Option<ReplayError>`) maps a `None`
  outcome to `ReplayError::Artifact`.
- **api engine** serving (score + witness paths) maps `None` to
  `InferenceError::Scorer` via `.ok_or_else(...)`.
- **graph-cli** `graph_infill` maps `None` to its own `Result` via
  `.ok_or_else(...)`.
- The `fwda_roundtrip` / `router_reconnect` / `status_policy` test call sites are
  unchanged — they already use `Option`-compatible `.expect(...)` / `.unwrap()`.

### Audit
`graph-certify` audit-limits: **17 → 10** (score_runtime.rs 12 → 5). Remaining
score_runtime sites are the `step_*` streaming methods; score.rs keeps its
gate-C cluster + `recover_from_artifact`. No sanctioned error types introduced —
pure removal of `Result<_, String>`.

### Gauntlet
`cargo fmt --check`, `cargo build --workspace --all-targets`, `clippy` (certify +
cli + api, all-targets), `wasm32-unknown-unknown` build of graph-certify, and the
graph-certify lib / `fwda_roundtrip` / api lib / `status_policy` (+census) suites
— all green.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

