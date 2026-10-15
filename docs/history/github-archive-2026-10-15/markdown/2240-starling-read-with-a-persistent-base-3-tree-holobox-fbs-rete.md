# #2240 — Starling read with a persistent base-3 tree (HoloBox/FBS): retention fixed, recall flat across distance at 6-36 candidates/query; readout now limits (row 3) (References #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-12T04:32:32Z
- **Updated:** 2026-10-12T04:34:01Z
- **Closed:** 2026-10-12T04:33:38Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2240
- **Merged:** 2026-10-12T04:33:38Z
- **Merge commit:** aec570c560089647dc2e5a84825b9fc41ca757c7
- **Base / Head:** main / UOR-Foundation:claude/starling-tree
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Line:** starling read · the tree's own target is met, so under D22 the tree is kept and the count resets; recall is not met · headline "softmax at runtime: yes" → yes. References #2032.

Murmuration ladder, step 3 ([pre-registration](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6117320792) with its amendment). [Record](docs/labs/starling-tree-2026-10-12/README.md).

**Code (`read_route=starling:…:tree`):**
- **Per 600-cell vertex:** the recent ring plus a **base-3 odometer** of blocks. A third block merges the three, the M most salient keys survive and carry up a level (HoloBox persistent path-copy, FBS carry).
- **Read:** queries read the full stores of their 13 vertices, so the cost is O(M log₃ n).
- **Salience:** a learned map of the detached stream, used only through the sort. Its training-only regression targets the teacher's mean future attention, centred per row.
- **Cost counter:** measured candidates per query.
- **Bench option:** `mqar-bench starling_salience=W`.

**Result at 1536:**

| | Recall, mean | Hit rate | d1000 recall | Candidates per query |
|---|---:|---|---|---|
| L, softmax | 0.999 | — | 1.00 | 1536 |
| W03L, ring | 0.241 | 0.56–0.61 | 0.006–0.137 | ≤ 52 |
| **WT, tree** | **0.329** | **0.99 / 0.68 / 0.90** | **0.035 / 0.584 / 0.316** | **5.6–36 (max 190)** |

**Decision: row 3.** Retention is fixed and recall no longer decays with distance. The readout is the limit: on seed 1 the source is kept 99% of the time, yet recall is 0.03 even with it forced in.

**Exact-head checks** (pod CPU at `f57af1c2b`; this head's code tree is byte-identical after the rebase):
- starling lib tests: **15/15**;
- mqar-bench tests: **18/18**;
- negative gate: removing the merge tie-break fails `starling_tree_select_matches_a_batch_reference`, and restoring it passes;
- rustfmt clean on this PR's hunks.

**Runs:** 3 × 3,600 steps, `device=cuda`, no early stop. Outputs are in the cloud store (md5 eb050b33…).

**Unchanged:** non-tree routes. The ring path is untouched, and the tests pin tree = ring without evictions.

**Next:** the readout at 1536, with the tree on.

**Would the next step differ if this came out the other way?** Yes. A row-4 outcome would have sent the line to an oracle-salience diagnostic instead.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### review 5486568338 — 2026-10-12T04:32:34Z — Casey-allard — COMMENTED

Self-review at exact head 182cc92d5fed77babf144feab08f224098f0d405: the code tree equals f57af1c2b, where the pod ran starling 15/15 and mqar-bench 18/18 plus the tie-rule negative gate; the review's code verdict was OK and its four fitness items were fixed before compute (amendment posted); the rows are scored per the pre-registration (row 3); non-tree routes are unchanged (tree = ring without evictions is pinned). APPROVE (self).


---
_Comment/review ids in this file: 5486568338_

