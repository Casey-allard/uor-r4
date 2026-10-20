# #566 — feat(#510): R5 tranche — graph-certify regions_from_view to Option (certify 18->17)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-10T01:20:43Z
- **Updated:** 2026-10-09T13:09:04Z
- **Closed:** 2026-08-10T01:39:39Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/566
- **Merged:** 2026-08-10T01:39:39Z
- **Merge commit:** 67d94e872a8a1b84689d2caad9a4ee433c60020c
- **Base / Head:** main / UOR-Foundation:r5/regions-from-view-option
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## R5 tranche (#510): graph-certify `regions_from_view` → `Option`

`regions_from_view` recovers region parameters from a view the caller has already
parse- and CID-validated. Its failure branches — a missing HEAD or ROUT section,
zero declared nodes, a node record missing inside the declared count, or a
prototype window pointing outside ROUT — all mean the bytes are not a scorer's
graph, so convert `Result<Vec<RegionParams>, String>` to
**`Option<Vec<RegionParams>>`**.

### Callers adapted
- `GraphScorer::from_artifact` (already `Option`-returning) drops its `.ok()`
  bridge → `regions_from_view(&view)?`.
- `score::recover_from_artifact` (still `Result<_, String>`, the `--cover` CLI
  path) maps `None` via `.ok_or_else(...)`.

### Audit
`graph-certify` audit-limits: **18 → 17** (score_runtime.rs 13 → 12). No
sanctioned error types introduced — pure removal of a `Result<_, String>`.

### Gauntlet
`cargo fmt --check`, `cargo build --workspace --all-targets`, `clippy`
(graph-certify, all-targets), graph-certify lib / `fwda_roundtrip` /
`status_policy` (+census) suites — all green.

### Remaining graph-certify (after this)
12 sites: the `score_runtime.rs` scoring path (`score_candidates`,
`score_candidates_coded`, `probe_code`, `score_candidates_infill`/`legacy`,
`two_pass_infill_generate`, `infill_fill`, and the `step_*` streaming methods)
plus the score.rs gate-C cluster + `recover_from_artifact`. Converted next in
coherent clusters.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

