# #422 — Hopf post-remediation retrieval quality (the measurement #306 conditioned on)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-05T01:04:31Z
- **Updated:** 2026-08-05T17:09:26Z
- **Closed:** 2026-08-05T07:05:43Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/422
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

#306 (sign-loss remediation, 6.1× occupancy gain) explicitly conditions 'treated as an improvement' on retrieval-quality/collision measurement that never ran (completeness review 2026-08-05). Extend the occupancy harness (hopf_sector_occupancy.rs) with the #303 D3 protocol to a retrieval-rank metric. Feeds the router-reconnection track.

## Comments (4)

### comment 5187312217 — 2026-08-05T03:49:35Z — Casey-allard

Picking this up now (overnight measurement slot) — it is the direct lever on the #421 M-R1 ceiling gap (router anchor accuracy 9.3% vs true-anchor ceiling 31.5%). Ari: still yours to join when you're up; harness design will be posted here with the first numbers.

### comment 5187725023 — 2026-08-05T04:57:14Z — Casey-allard

First measurement recorded NEGATIVE (PR #430): shipped post-#306 projection filtered MRR 0.0045 vs pre-#306 shim 0.0743 (rule needed +0.020; measured −0.0699). Occupancy spread (456/512 vs 16/512) replicates but carries no retrieval value; sector agreement under the shipped arm correlates with WORSE base MRR (0.0918 vs 0.2380). #306's improvement claim refuted on its own deferred criterion. Real scope now: content-aligned projection redesign — the router-quality lever #421 needs. Ari: yours to join.

### comment 5188234132 — 2026-08-05T06:11:52Z — Casey-allard

Phase-2 candidates measured (PR pending): data-PCA 0.0176/448-sectors, mag+sign 0.0308/129, mean-direction 0.0749/53 — all NEGATIVE vs the ≥0.020 rule, each beats shuffled. Clean spread-vs-retrieval frontier; G matches pre-#306 MRR at 3.3× occupancy but no candidate wins both. Cheap projection swaps are exhausted — next phase needs binning-geometry or supervised-direction work. Ari: the frontier table is your starting map.

### comment 5194568873 — 2026-08-05T16:37:59Z — Casey-allard

Phase-3 record (PR above): supervised directions ALSO negative — Fisher 0.0195/430, supervised-sign 0.0428/121 vs rule ≥0.0763. Seven projections (two supervised) on one frontier: retrieval signal is not linearly recoverable from block inputs at this binning. Design space status: linear projections EXHAUSTED. Next honest scope: binning geometry or non-block representation. Ari — this is now cleanly your kind of problem; full frontier + harness in the test file.


---
_Comment/review ids in this file: 5187312217, 5187725023, 5188234132, 5194568873_

