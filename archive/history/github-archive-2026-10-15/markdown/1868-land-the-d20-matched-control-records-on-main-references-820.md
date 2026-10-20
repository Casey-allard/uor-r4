# #1868 — Land the D20 matched-control records on main (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T02:30:40Z
- **Updated:** 2026-10-09T13:19:49Z
- **Closed:** 2026-10-08T02:32:21Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1868
- **Merged:** 2026-10-08T02:32:21Z
- **Merge commit:** a9a054baa99cd2b582587f4d43de14700ecca99b
- **Base / Head:** main / UOR-Foundation:deepseek/d20-records
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this lands

The 2026-10-06 D20 matched-control battery — the run whose parameter-matched ordinary control the project now cites — existed only as uncommitted files in a worktree plus a pushed branch that was never merged. This brings the documented material under `docs/labs`, where the project keeps its records:

- `docs/labs/deepseek-d20-matched-control-2026-10-06.md` — the lab record for the matched control.
- `docs/labs/d20-run-2026-10-06/` — preregistration, run report, analysis, the analysis script and the runner.

## Scope and what is deliberately not carried

- **No source changes.** The branch's edits to `crates/uor-r4-training/examples/mqar-bench.rs` are superseded: main already carries the `lineage=none` parameter-matched control arm, so replaying those hunks either conflicts or duplicates. Verified by grepping both trees for `lineage=none` and `parameter_matched`, present in each.
- **No raw logs.** The per-run stdout/stderr/cost files and the raw `analysis.json` stay on `origin/deepseek/d20-run`; only the curated records move into the tree.
- Documentation and analysis only: no model, no training, no evaluation.

Verified on the exact head: `git diff --name-only origin/main...HEAD` lists only the five `docs/labs` paths above.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

