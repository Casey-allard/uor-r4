# #421 — Router reconnection: router-selected anchors feeding the infill engine (the original hybrid, #393 §8)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-05T01:04:29Z
- **Updated:** 2026-08-05T16:35:23Z
- **Closed:** 2026-08-05T16:35:23Z
- **Labels:** enhancement
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/421
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The #393 §8 'router-stack reconnection' track, promoted to a named issue (completeness review 2026-08-05: the only program row with neither an issue nor a measurement; the original geometric lineage is parked-not-refuted).

**Claim to test:** the original hybrid architecture — the R4 router (zeta-grid state, Hopf transport, content-bearing storage #245) supplies content anchors every ~4th token; the graph engine completes syntax between them. This is the deployment role the serving stack was built for and it became cheaply testable when #419 merged: `infill_fill` / `r4 graph infill --skeleton` accepts an arbitrary anchor grid from any source.

**M-R1 (first measurement):** on the natural wiki-obs stack, produce router-selected anchor grids (stride 4), fill with the A-mode fused scorer, grade against (a) A-mode true-anchor row (39.6%/live 40.9 — ceiling), (b) unigram/fwd-null floors, (c) base Rule 1+2 (36.5%). Exit rule pre-declared before the run.
**M-R2:** the #273 declared-but-never-run falsifier (shuffled-state vs routed-state injection control) rides the same harness.

Prereqs that feed this: #306's promised retrieval-quality measurement (separate issue). Never-measured ancestry this revives: zeta-grid routing at scale (#246 was fixture-only), VSA/spectral geometry.

## Comments (3)

### comment 5186987458 — 2026-08-05T02:54:30Z — Casey-allard

M-R1 POSITIVE (PR #428): router anchors 7.6% free top-1 vs unigram null 2.1% (+5.5pp, rule ≥2pp) and vs shuffled-router control 3.1% (+4.55pp, rule >0). Anchor accuracy: router 9.3% vs unigram 6.0%. True-anchor ceiling 31.5%. The original hybrid carries real signal end-to-end; gap to ceiling = the router-quality work (#422 feeds this).

### comment 5187724821 — 2026-08-05T04:57:12Z — Casey-allard

ARM B2 (non-oracle, PR #429): draft-context router anchors 4.9% free top-1 vs unigram 2.1% (+2.74pp, rule ≥2) vs shuffled 3.1% (+1.79pp) → POSITIVE. The M-R1 result stands without the oracle-context caveat (oracle B 7.6%; drift costs ~2.7pp, signal survives).

### comment 5194542844 — 2026-08-05T16:35:22Z — Casey-allard

Closing: the declared scope is fully measured — M-R1 (router anchors 7.6% vs nulls 2.1/3.1, POSITIVE, PR #428), B2 non-oracle (4.9%, POSITIVE, PR #429), M-R2 at scale (NEGATIVE, composition-scoped — see #399 falsifier-2 record). All results regime-scoped per the corpus-composition finding. The unmeasured ancestry (zeta-grid at scale, VSA/spectral) is tracked in the new issue above, not orphaned. Router-quality continuation lives in #422; broad-corpus questions in #433.


---
_Comment/review ids in this file: 5186987458, 5187724821, 5194542844_

