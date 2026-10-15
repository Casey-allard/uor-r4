# #485 — test(router): #484 — the lexical weight is on a plateau because the cosine is noise

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-07T20:56:22Z
- **Updated:** 2026-08-07T21:31:40Z
- **Closed:** 2026-08-07T21:14:48Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/485
- **Merged:** 2026-08-07T21:14:48Z
- **Merge commit:** 19c9b0da8a8e9fb2ae72f7884033e5afcc796605
- **Base / Head:** main / UOR-Foundation:issue-484-lexical-weight
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

I filed #484 on my own hypothesis from #480's record: that `shared_count * 100` might be **suppressing** a geometric signal. It is not. There is no signal to suppress.

## The sweep is flat

`W` over five decades, everything else at the deployed default, on #480's corpus / probes / deranged null **verbatim** so the rows are comparable rather than merely similar:

| W | top-1 | MRR | recall@20 | null MRR |
|---:|---:|---:|---:|---:|
| 0 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| 1 | 0.6240 | 0.7179 | 0.9720 | 0.0000 |
| 10 | 0.6240 | 0.7179 | 0.9720 | 0.0000 |
| **100 (shipped)** | **0.6240** | **0.7179** | **0.9720** | 0.0000 |
| 1,000 | 0.6240 | 0.7179 | 0.9720 | 0.0000 |
| 100,000 | 0.6240 | 0.7179 | 0.9720 | 0.0000 |

**NEGATIVE** against the pre-declared +0.05 MRR bar: the best alternative weight moves MRR by +0.0000. The `W=100` arm reproduces #480's shipped row exactly — that is the harness's binding validity check, asserted against pinned constants rather than eyeballed.

**Why it is flat, and this had never been measured:** the geometric term's dynamic range is **~0.37**, not ~1. `slice_norm` is small, so any weight above ~0.4 already produces strict lexicographic order. The shipped `100.0` sits on a plateau whose left edge is below 1 — it is not a tuning parameter at all.

## The all-zero arm, interrogated instead of reported

`W=0` scoring 0.0000 on everything is exactly the pattern this repo has learned to treat as a harness bug until proven otherwise. Two checks were needed and **both mattered**:

1. **`W=0` is not a cosine ranking.** `slice_norm` is a per-window-**bucket** scalar, so with the lexical term gone the order is driven by which bucket has the largest slice norm. Added `set_unscaled_geometric_term` to rank by bare `sim`. It also scores 0.0000 — so the confound didn't change the answer, but publishing the scaled row as "the cosine comparator" would have been the wrong quantity under the right label.
2. **A truncated `recall@20` of zero is ambiguous.** It cannot separate "no signal" from "broken" from "ranked well, just outside twenty". So the target is ranked against **all 46,342** candidates:

| ranking | probe | containment | median rank | full-list MRR |
|---|---|---:|---:|---:|
| shipped (W=100) | shipped | 1.0000 | **1** | 0.7186 |
| bare cosine | shipped | 1.0000 | 21,401 | 0.0001 |
| bare cosine | ordered | 1.0000 | 21,401 | 0.0001 |
| bare cosine | **identity** | 1.0000 | **21,082** | 0.0001 |

Random over this set is median 23,171. Containment is 1.0000 everywhere — retrieval is perfect and **all** of the difference is ordering.

The `identity` row closes it: querying with the **exact stored sentence**, the cosine still ranks that sentence at median 21,082 of 46,342. The `ordered` row removes the reversal alone and changes nothing. So it is not the probe form, not subsampling, not word order — **on this path the geometry does not identify a sentence from itself.**

## What it settles

- #480's negative is now explained mechanistically, not just empirically.
- **#442's 0.8948 is a cosine-ranked figure and does not describe `get_top_resonances_native`.** README quoted it as a serving number; that is now qualified in place.
- Vector-shape / query-projection work on this path should not be re-proposed on the premise that a geometric gain is masked. The thread #480 opened is closed.

**Nothing shipped.** `DEFAULT_LEXICAL_WEIGHT` stays 100.0, both knobs default off. The sweep is flat, so there is no ordering change to gate through #421 / `router_reconnect`.

## Also closes #483

The 2.11M run happened, with the #471 phase timing on: Gate C is **51.94s** there (right-context pass 40.07s), not 85 minutes, and core-time per record **falls** with scale (204 µs at 500k → 152 µs at 2.11M). No scale cliff; the 85 minutes was never Gate C. Scope limit stated in the record: that corpus is the fixture replicated 4.22×, valid for the content-independent code pass and **not** for the scoring or cover-induction costs.

Records: `docs/lexical_weight_484.md`, `docs/gate_c_arm_skip_471.md`

Closes #484
Closes #483


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

