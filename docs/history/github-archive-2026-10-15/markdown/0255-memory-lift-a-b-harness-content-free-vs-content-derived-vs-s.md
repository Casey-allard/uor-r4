# #255 — Memory-lift A/B harness: content-free vs content-derived vs shuffled-vector arms for the angular store (#245 follow-up)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-28T11:50:03Z
- **Updated:** 2026-07-29T20:51:10Z
- **Closed:** 2026-07-29T20:51:10Z
- **Labels:** enhancement
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/255
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

### Summary
#245 (merged in #252) accidentally ended a production run of the "corrupted memory" control condition: content-free vectors stored where content-derived ones belong. That makes the cleanest available real-world memory-lift experiment — but the patch landed before an under-the-stub baseline was captured, so the design becomes a **three-arm A/B harness** rather than a before/after series (statistically equivalent; all arms on one code vintage):

1. **content-free** (reconstructed stub: route with `state_vector=None`, the pre-#245 behavior — reproducible exactly),
2. **content-derived** (current, post-#245),
3. **shuffled-vector control** (content-derived vectors permuted across sentences).

### Measure
Retrieval quality (resonance ranking of held-out queries against an indexed corpus) and generation coherence on the geometric path, per arm. The κ/deficit-angle/γ telemetry already emitted per generation step is the eval instrumentation — no new plumbing needed.

### Why
If resonance retrieval coming alive moves the coherence needle, that is an empirical data point for the content-specific-memory-beats-content-free-structure claim measured on production code; if it does not, that is the domain-priming failure mode — equally worth knowing. Patch-and-measure, not patch-and-vibe.

### DoD
Harness with an arm toggle; metrics table across the three arms on a fixed corpus/query set; result posted regardless of direction. Router crate only.

*From Maura's cross-reference of the geometric lineage audit; companion to #245/#246.*


## Comments (1)

### comment 5123252912 — 2026-07-29T20:51:09Z — Casey-allard

## Three-arm table — measured (2026-07-29, deterministic fixture harness, release)

Harness merged in PR #287 (`crates/uor-r4-router/tests/memory_lift_ab.rs`, arm toggle per the DoD); fixed corpus/query set = the harness fixture (10 sentences, 6 held-out probes).

| Arm | top1 | MRR |
|---|---|---|
| 1 — content-free (reconstructed pre-#245 stub, `state_vector=None`) | 0.17 | 0.376 |
| 2 — content-derived (production, post-#245) | 0.17 | 0.360 |
| 3 — shuffled-vector control | 0.00 | 0.176 |

## Reading

- **Direction: the domain-priming failure mode, posted as promised regardless.** Content-derived vectors do NOT outperform content-free structure on this fixture — arm 2 sits at parity on top1 and 0.016 MRR *below* arm 1. Resonance retrieval coming alive did not move the coherence needle here.
- Both real arms clear the shuffled control decisively (0.36-0.38 vs 0.18 MRR), so the angular store is retrieving structure — it is just not *content-specific* lift.
- Caveat: 10×6 fixture — adequate for the DoD's fixed-set requirement, too small to close the broader claim. The honest statement: on production code at fixture scale, the content-specific-memory-beats-content-free-structure claim is **unsupported**; #246 (re-ported zeta design-matrix) and #247 (session-manifold signature lane) are where a bigger swing would come from, and this harness is the regression gate they should be measured against.

DoD complete: harness with arm toggle merged (#287, commit 944ac0d), metrics table posted, result recorded regardless of direction. Closing.


---
_Comment/review ids in this file: 5123252912_

