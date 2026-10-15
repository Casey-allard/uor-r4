# #60 — Multiresolution cover induction over traced observations

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-22T18:29:17Z
- **Updated:** 2026-07-22T20:46:05Z
- **Closed:** 2026-07-22T20:46:05Z
- **Labels:** graph-compiler
- **Milestone:** Phase 2: Overlapping multiresolution regions
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/60
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The Phase 2 core (M.V.G. critical path): recursive/spherical k-means or medoid/density variants over compiler-side representations + teacher distributions from the observation pipeline; candidate partitions scored by predictive entropy reduction, neighbor preservation, bootstrap stability, probe cost, artifact bytes (PDF §22); calibrated overlapping membership (masks+radii, top-M, nearest-class fallback floor retained); parent/child refinement edges and lateral neighbor edges from co-activation; explicit overlap nodes only with held-out gain; reference classifier frozen as normative semantics with routing recall measured against it. Depends on the observation pipeline v2 issue. Gate C comparison harness runs graph vs TLA3 baseline at exit.

## Comments (1)

### comment 5051302010 — 2026-07-22T20:46:04Z — auser

First slice landed in 20cca92 (pushed to main), with the recall numbers the issue required.

Engine: mini-batch spherical k-means over the deterministic context-bundle lane (bundle_plain geometry), blake3-seeded farthest-point init, f64 partial sums in global observation order (batch-size- and thread-invisible; T=1 vs T=4 byte-identical — single-threaded v1 per slice), convergence by bit-identical centroids. Recursive splits only when support >= 64 AND parent-minus-children next-token entropy gain > 0.25 bits AND region/depth budgets allow. Membership: top-3 masked-Hamming per depth with nearest-region fallback; frozen ReferenceClassifier as normative semantics. Radii: 95th percentile of member distances. Edges: E_r from recursion, E_o at co-activation >= max(2, n_train/200), degree cap 8.

Fixture corpus (150k, 80/20 split): 42 regions (8/14/20 at depths 1-3), 17 splits with gains 0.28-0.94 bits, 208 edges (42 E_r + 166 E_o), radii 0-144. Reference top-1 recall 78.9% / 69.8% / 58.9% (d1/d2/d3); top-M 93.7% / 82.8% / 72.4%. Class-cover co-assignment recall 87.0/84.7/78.1 (top-1). Frontier width mean 2.4, max 3. R4G1 artifact (36,220 B) passes GraphView stage-1+2 and CID verification; double-run byte-identical.

Recorded deviations: f32 observation matrix held resident (~147 MB at fixture scale — the observe.rs quantized shard lane is the stated v2); EXCT omitted by slice; edge score_q = 0 with co-activation counts in the report; f64 log2 in entropy is libm-sensitive cross-platform (same status as the macOS-pinned kappa baseline, resolved by the D2 canonical mode). Verified by the maintainer agent: 44 suites, clippy -D warnings, fmt, kappa-reproduction green.


---
_Comment/review ids in this file: 5051302010_

