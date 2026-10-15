# #445 — feat(compiler): #435 — corpus-relative cover split criteria: capacity scaling works, threshold tuning refuted

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-06T01:26:04Z
- **Updated:** 2026-10-09T14:17:06Z
- **Closed:** 2026-08-06T01:48:52Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/445
- **Merged:** 2026-08-06T01:48:52Z
- **Merge commit:** 27bfa9bd7d9102399647be34d5749a696324d69d
- **Base / Head:** main / UOR-Foundation:i435-relative-split
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #435. All new criteria are **default-off**; default cover induction is asserted byte-identical against pinned baselines measured on main (κ `blake3:b27e8c72…` and `blake3:7ce48b99…`), so no artifact/κ/CI churn.

**The defect reproduces directly:** on the 500k stack regions go 12 → 12 → 10 → **8** as data grows 7.7×, and at 401,655 observations **no split clears the 0.25-bit bar at all** (mean gain 0.0777, max 0.1897).

**Criteria measured** — relative gain (θ calibrated to the reference: mean H(parent) = 6.518 bits ⇒ θ=0.0384), MDL/BIC, and a capacity law `k0_eff = k0·(n/n_ref)^α`, α=0.45 from the measured class-growth exponent.

**Result at 2,110,111 records:** the fixed-θ relative criterion **fails** — H(parent) itself grows with n (6.52 → 9.65 bits), so θ·H becomes a *stricter* absolute bar and the cover collapses to 8 regions. MDL likewise collapses. **Only capacity scaling passes:** regions monotone 14 → 23 → 25, and it is the only arm to beat the baseline on held-out top-1 (0.0470 vs 0.0438). Contrast fails for every arm at that scale including the baseline (0.1441), so that clause is failed by the corpus, not the criterion.

**Conclusion: scale the capacity, don't tune the threshold.**

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

