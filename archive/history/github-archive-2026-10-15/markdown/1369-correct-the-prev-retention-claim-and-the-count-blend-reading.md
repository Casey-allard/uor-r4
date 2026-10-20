# #1369 — Correct the prev-retention claim and the count-blend reading after K3 review; order the adjudication + depth ladder (References #820, #973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-23T21:39:15Z
- **Updated:** 2026-10-09T13:17:48Z
- **Closed:** 2026-09-23T21:40:26Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1369
- **Merged:** 2026-09-23T21:40:26Z
- **Merge commit:** f5fa0977a106598b97f161bf63de4cf5130e521a
- **Base / Head:** main / UOR-Foundation:codex/prev-retention-correction-20260923
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this changes

Documentation-only corrections required by the **K3 architecture review** of the merged count-blend result
(PR #1368, `4b638b2e`). No code and no measured number changes; three merged records carry a premise the review
found refuted, and the count-blend reading is qualified.

## Corrections

1. **`prev`-retention premise corrected.** The merged state-probe record's title clause and §3 ("the state does
   not retain the second-most-recent token") is **withdrawn as a general statement**. An unmerged controlled
   diagnostic (`codex/observer-transport-20260923`) run on **byte-identical dev states** (SHA `c5334071…992f`,
   matching the sealed `olx-probe-4` receipt) recovers `prev` at **75.90 %** (3,209/4,228) with a transport-aware
   decoder versus the merged raw-embedding **2.06 %** (87/4,228) on the same denominator, with rotated-label
   (0.67 %) and wrong-frame (0.14 %) controls. What stands: `prev` is unavailable to the **linear readout class**
   (the `P − C = +0.729` gap) and to a raw-embedding probe (2.1 %). The claim is annotated in the state-probe,
   readout-refit and current-state records.
2. **Count-blend reading qualified.** The K3 review confirmed the decision (do not build an exact addressed local
   read) but required: (a) the tune→dev collapse printed (pool beats `C` by 0.0736 on tune but +0.0049 on dev);
   (b) the frequent-head qualification (−0.0417 [−0.0686, −0.0106] **beats** `C` while the 422 rare targets lose
   ≈+0.552 bits/target); (c) the decision's ground stated as **cost and threshold** (a ≈4096×4096 4-bit addressed
   table is ≈8–16 MiB vs the 466,711-byte artifact for ≈0.04–0.08 bits over serving `C` alone), not "absence";
   (d) the unmerged `codex/observer-transport-20260923` two-parameter result (−0.037612
   [−0.057254, −0.016462] vs calibrated counts) registered as **live contested evidence**.
3. **Next milestone repointed** in current-state, project-track and README: a **frozen two-parameter
   adjudication + context-classed λ + transport-decode depth ladder** (`x_(t-2)`, `x_(t-3)`, `x_(t-4)`) before
   the longer-context control.

The K3 review is recorded as obtained (it independently verified the exact criterion symbolically and matched the
observer's state SHA to the sealed probe receipt). The contradicting evidence remains **unmerged and not yet
reproduced on main**; its independent adjudication is the ordered next milestone.

`python3 scripts/check_claim_wording.py` passes. References #820, #973.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

