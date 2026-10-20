# #562 — feat(#510): R5 tranche — graph-certify score.rs emit path to total (35->30 certify)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T23:35:11Z
- **Updated:** 2026-10-09T13:09:04Z
- **Closed:** 2026-08-10T00:02:49Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/562
- **Merged:** 2026-08-10T00:02:49Z
- **Merge commit:** 9a032d501c166927e8de50cdf44a56e463a5c275
- **Base / Head:** main / UOR-Foundation:r5/score-emit-path-total
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## R5 tranche (#510): graph-certify `score.rs` emit path → total

Converts the scored-artifact **emit path** in `graph-certify`'s `score.rs` to a
total surface. Every input to this cluster is compiler-produced, so each former
`Result<_, String>` error branch is a **self-produced-bytes invariant** whose
violation is a defect, not a runtime outcome:

- `verify_theorem_7_wired` → `Option<String>` (`None` = wiring holds); the caller
  turns `Some` into a panic.
- `encode_context_rows` / `encode_forward_anchor_rows` → `Vec<u8>`; the NGRAM/FWDA
  size ceilings and canonical-sort/key invariants become `assert!`.
- `emit_residual_exct` → `(Vec<u8>, QuantizationErrorStats)`; EXCT size ceilings
  become `expect`/`panic`.
- `emit_scored_r4g1` → `(Vec<u8>, ScoredGraphInfo)`; Theorem 7 wiring, the section
  size ceilings, and the `GraphView::parse` + `verify_cids` re-validation of the
  just-emitted bytes now **panic closed** rather than returning `Err`.

Callers adapted in the same change: graph-cli `score_command` and the two
`cover_sweep` emit sites drop their `?`; the `graph_infill_cli`, `fwda_roundtrip`,
and `status_policy_common` fixtures drop `.expect(...)`.

### Audit
`graph-certify` audit-limits: **35 → 30** (score.rs 11 → 6). No sanctioned error
types are introduced; the removed surface is pure `Result<_, String>`.

### Gauntlet
`cargo build --workspace --all-targets`, `clippy` (certify + cli, all-targets),
`wasm32-unknown-unknown` build of graph-certify, and the touched test suites
(`fwda_roundtrip`, `graph_infill_cli`, both lib suites, `status_policy`,
`status_policy_census`) all green.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

