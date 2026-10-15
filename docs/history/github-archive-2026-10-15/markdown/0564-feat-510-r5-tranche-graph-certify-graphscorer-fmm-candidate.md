# #564 — feat(#510): R5 tranche — graph-certify GraphScorer::fmm_candidate to Option (certify 21->20)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-10T00:33:42Z
- **Updated:** 2026-10-09T13:09:00Z
- **Closed:** 2026-08-10T00:50:53Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/564
- **Merged:** 2026-08-10T00:50:53Z
- **Merge commit:** b295512d8997ceab92b802293a21ed81176a9240
- **Base / Head:** main / UOR-Foundation:r5/fmm-candidate-option
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## R5 tranche (#510): graph-certify `GraphScorer::fmm_candidate` → `Option`

`GraphScorer::fmm_candidate` builds the certifier-side low-rank far-field
candidate. Its only construction path, `FmmCandidateScorer::from_graph_parts`,
already reports failure as `None` (R5 tranche #561) — the wrapper merely
re-wrapped that `None` into a `Result<_, String>` with a synthetic message.
Drop the wrapper and return **`Option<FmmCandidateScorer>`** directly.

The sole caller (`tests/bdd.rs::load_fmm_candidate`, itself `Option`-returning)
drops its now-redundant `.ok()`.

### Audit
`graph-certify` audit-limits: **21 → 20**. No sanctioned error types introduced
— pure removal of a `Result<_, String>`.

### Gauntlet
`cargo fmt --check`, `cargo build --workspace --all-targets`, `clippy`
(graph-certify, all-targets), graph-certify lib + `fwda_roundtrip` suites — all
green.

### Remaining graph-certify (after this)
5 sites in `score.rs` (gate-C cluster + `recover_from_artifact`) and 15 in
`score_runtime.rs` — the interconnected `GraphScorer` scoring engine, converted
in coherent clusters next (the `from_artifact` + `verify_witness_replay` cluster,
then the `score_candidates`/`score_step` scoring path).

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

