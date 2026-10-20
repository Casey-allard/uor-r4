# #2241 — Starling readout self-distillation with the tree: one seed at softmax-level recall (0.964) at ~69 candidates/query, two never learn (row 4) (References #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-12T05:14:10Z
- **Updated:** 2026-10-12T05:15:40Z
- **Closed:** 2026-10-12T05:15:13Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2241
- **Merged:** 2026-10-12T05:15:13Z
- **Merge commit:** 462820abf770d246625cb5c6c6e57b08cab0754b
- **Base / Head:** main / UOR-Foundation:claude/starling-readout
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Line:** starling read · count 1/3 · headline "softmax at runtime: yes" → yes. References #2032.

Murmuration ladder, step 3b ([pre-registration](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6117824256) with its amendment; CODE OK, FIT after fixes). [Record](docs/labs/starling-readout-2026-10-12/README.md).

**Code** (training only; the served read is unchanged and softmax-free):
- `set_starling_rank_distill` regresses the served rank-table × gate weights on the softmax of the content score over the kept slots. The gate is recomputed from the detached score, so the term trains the readout only.
- A measured effective readout slot count.
- `mqar-bench starling_rank_distill=W`.

**Result at 1536:**

| Seed | WT | **WTR** | WTR forced-in | WTR candidates per query |
|---|---:|---:|---:|---|
| 1 | 0.031 | 0.063 | 0.063 | 19.0 (max 189) |
| 2 | 0.615 | **0.964** | 0.970 | 68.9 (max 228) |
| 3 | 0.340 | 0.104 | 0.131 | 16.5 (max 171) |

**Decision: row 4** (mean 0.377).

**What the seeds show:** seed 2 matches softmax (0.997) at about 4.5% of its reads, with recall flat to d1000. Seeds 1 and 3 never learn the task: train NLL stays near 3.9, and recall is near chance even with the source forced in. The limit is learning reliability after the dense helper anneals out.

**Exact-head checks:** pod CPU at `56500ce93`: starling lib tests 16/16, mqar-bench tests 18/18. This head adds only a rustfmt hunk and docs; its code is otherwise identical. The runs used `device=cuda`, all 3,600 steps with no early stop, and are archived in the cloud store (md5 925b7843…). STATUS, ROADMAP and #2028 are updated.

**Next:** the helper schedule.

**Would the next step differ if this came out the other way?** Yes. A row-2 outcome would have gone straight to 4096.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### review 5486744900 — 2026-10-12T05:14:12Z — Casey-allard — COMMENTED

Self-review at exact head 5f230bf0950e71ef4593e874435de32d1fce726a: the code equals the tested 56500ce93 plus a formatting-only hunk; the review verdict was CODE OK and its three fitness items were fixed before compute; the rows are scored per the amended pre-registration (row 4); the seed-level observations are reported separately from the decision. APPROVE (self).


---
_Comment/review ids in this file: 5486744900_

