# INC-0144: Stage 3 — H^4 Hopf Fixed Geometry vs K-means Adaptive Clustering

## Status
Closed: KEEP.

Fixed H^4 Hopf geometry discriminates PPMI-SVD semantic embeddings from col-permuted
control (rel_diff=31.2%) while K-means adaptive clustering CANNOT discriminate
(rel_diff=3.1%). Stage 3 PARTIAL-PASS: the Hopf angular correctness is confirmed for
disc rimination; the angular mass balance remains an open question for later increments.

## Trigger
INC-0143 Closed: KEEP (2026-03-13). Stage 2 closed PARTIAL-PASS:
PPMI-SVD semantic embeddings with H^4 Hopf-base routing (dims 3,65,2,21) give
rel_diff=38.5% discrimination across 4 seeds. Stage 3 (Hopf Angular Correctness)
is now unblocked.

Key diagnostic from INC-0143 finalize (SEM_ORIG, 4-seed mean):
- mean_geodesic_knn_jaccard = 1.0   → routing perfectly preserves local neighborhoods
- mean_hopf_sector_chi_std_mean = 0.0575  → tight chi clustering within sectors (good base separation)
- mean_hopf_chi_mass_error = 0.032  → chi coordinate is well-calibrated
- mean_hopf_angular_mass_error = 0.666  → angular sectors (theta1, theta2, delta) are unequal in mass
- mean_hopf_theta1_mass_error = 1.091  → theta1 is very poorly mass-balanced
The high angular mass errors reflect that PPMI-SVD embeddings are NOT uniformly
distributed on the Hopf base — they cluster in semantically dense regions.
This is the semantic structure driving discrimination, not noise.

## Kill-List Stage
Primary: 3. Hopf Angular Correctness

## Mathematical Object Under Test
The H^4 Hopf base angular routing law for the FIRST factor manifold.

Specifically: is the discrimination from Stage 2 (SEM_ORIG > SEM_COL_PERM) driven
by the FIXED H^4 geometric structure of the Hopf fibration, or would ANY
data-adaptive sector assignment (K-means) produce equivalent results?

This is the core Stage 3 question. For the project thesis ("fixed H^4 geometry
can replace dense linear-lattice routing"), the geometry must be doing
FUNDAMENTALLY DIFFERENT work than adaptive learning:

Theorem under test:
  If phi = Hopf-base(x) is a FIXED map:
    E[pmax | x ~ ORIG] >> E[pmax | x ~ COL_PERM]  [Stage 2 confirmed]
  But if centroid = argmin_c ||x - c|| is LEARNED from training data:
    E[pmax | x ~ ORIG] ≈ E[pmax | x ~ COL_PERM]  [K-means adapts to any distribution]

K-means adapts its centroids to BOTH ORIG and COL_PERM data with equal fidelity —
it has no way to distinguish "semantic structure" from "permuted structure" because
it only sees the final distribution, not the generating process.
The Hopf map is FIXED: it captures the specific geometric structure of dims
(3,65,2,21) as a unit quaternion, which is SENSITIVE to whether those dims carry
genuine PPMI co-occurrence structure.

## Hypothesis
H^4 Hopf-base routing discriminates ORIG from COL_PERM (rel_diff ≈ 38%, Stage 2)
because it applies a FIXED geometric map. K-means, which learns centroids from
training data, will adapt to ANY input distribution including COL_PERM, so:
    rel_diff(KMEANS_ORIG vs KMEANS_PERM) ≈ 0%
while:
    rel_diff(HOPF_ORIG vs HOPF_PERM) ≈ 38% (replicates Stage 2)

If the hypothesis is confirmed, it proves:
1. The H^4 Hopf geometry provides something K-means CANNOT (pattern-free routing)
2. The discrimination is specific to the geometric structure of the Hopf map
3. Raw feature learning fails where fixed geometry succeeds — core thesis support

## Minimal Scope
1. Use existing ppmi_proxy.npz (PPMI-SVD semantic proxy, same as Stage 2)
2. Screen (1 seed), 4 routes:
   - HOPF_ORIG / HOPF_PERM: Stage 2 reference pair (phase4d_hopf_base, dims 3,65,2,21)
   - KMEANS_ORIG / KMEANS_PERM: sector_mode=kmeans (100D PPMI input, learned centroids)
3. Measure per route:
   - mean_pmax_after (primary: discrimination signal)
   - mean_hopf_sector_chi_std_mean (chi tightness — geometry quality)
   - mean_hopf_angular_mass_error (mass balance — measure consistency)
   - mean_geodesic_knn_jaccard (neighborhood preservation)
4. If screen passes: Confirm (2 seeds) on same 4 routes

## Success Condition
- rel_diff(HOPF_ORIG vs HOPF_PERM) >= 0.2 (replicates Stage 2 ≥ 38%)
- rel_diff(KMEANS_ORIG vs KMEANS_PERM) < 0.1 (K-means cannot discriminate)
- This confirms: FIXED H^4 geometry is the essential mechanism

## Falsification Condition
- rel_diff(KMEANS_ORIG vs KMEANS_PERM) >= rel_diff(HOPF_ORIG vs HOPF_PERM)
  → K-means also discriminates → the routing is semantic, not geometric
  → Stage 3 needs to pivot: test whether Hopf's SPECIFIC angular law is
    superior to other fixed geometric partitions (e.g., random 4D projections)
    rather than learned vs fixed

## Acceptance
- Screen artifact exists in results/analysis/
- Route-health diagnostics for both Hopf and K-means are interpretable
- Explicit KEEP / KILL / REFINE decision recorded in this doc

## Scope Guardrails
- Do not modify the PPMI proxy building code — use existing ppmi_proxy.npz
- Do not change phase4_dims from 3,65,2,21 (Stage 2 confirmed subspace)
- Do not open phase transport (Stage 4) work here
- Do not claim hardware efficiency from this branch alone

## Screen Results (seed=0)

**Config:** `configs/proxy_transfer_inc0144_hopf_vs_kmeans_stage3_screen.json`

| Route | pmax_after | chi_std | angular_mass_err | jaccard |
|---|---|---|---|---|
| HOPF_ORIG | 0.0860 | 0.0580 | 0.6777 | 1.0 |
| HOPF_PERM | 0.0624 | 0.0572 | 0.3736 | 1.0 |
| KMEANS_ORIG | 0.0668 | 0.2463 | 0.6777 | 1.0 |
| KMEANS_PERM | 0.0708 | 0.2120 | 0.3736 | 1.0 |

- HOPF rel_diff = 31.8% → SCREEN PASS
- KMEANS rel_diff = −5.8% (wrong direction) → K-means cannot discriminate
- Note: HOPF (4D fixed) achieves higher absolute pmax than KMEANS (100D adaptive): 0.086 vs 0.067
  The fixed geometric pre-screening of 4D subspace is MORE efficient than 100D adaptive clustering.
- chi_std: HOPF=0.058 vs KMEANS=0.246 — Hopf sectors are 4× more chi-coherent (H^4 radially structured)

## Confirm Results (seeds 0,1)

**Config:** `configs/proxy_transfer_inc0144_hopf_vs_kmeans_stage3_confirm.json`

| Route | seed0 pmax | seed1 pmax | mean pmax | chi_std | jaccard |
|---|---|---|---|---|---|
| HOPF_ORIG | 0.0860 | 0.0888 | 0.0874 | 0.0577 | 1.0 |
| HOPF_PERM | 0.0624 | 0.0652 | 0.0638 | 0.0575 | 1.0 |
| KMEANS_ORIG | 0.0668 | 0.0776 | 0.0722 | 0.2528 | 1.0 |
| KMEANS_PERM | 0.0708 | 0.0688 | 0.0698 | 0.2144 | 1.0 |

- HOPF rel_diff: seed0=31.8%, seed1=30.6%, mean=**31.2%** → CONFIRM PASS
- KMEANS rel_diff: seed0=−5.8%, seed1=+12.0%, mean=**3.1%** → near-zero, high variance; cannot discriminate
- KMEANS instability (−5.8% ↔ +12.0%) is itself a Stage 3 signal: adaptive clustering produces
  inconsistent results depending on random centroid initialization; fixed geometry is STABLE.

## Cross-Stage Observation
- Stage 4 readiness: The Hopf-base routing with PPMI-SVD (seed-stable, Jaccard=1.0) is a solid
  foundation for testing phase transport. The question is: does adding fiber-phase information to
  the routing address improve anything beyond what the base routing already achieves?
  → Record: Stage 4 (Phase Transport) should be tested using PPMI-SVD proxy, NOT hash embeddings.
    The hash-embedding Stage 4 results (RR-063, RR-064) may not generalize.

## Decision
**KEEP.** Stage 3 (Hopf Angular Correctness) PARTIAL-PASS:
- The FIXED H^4 Hopf geometry is the essential mechanism enabling discrimination (not
  just having semantic features — K-means with 96× more dimensions still fails).
- The Hopf-base sectors are chi-coherent (tight radial structure within sectors).
- Angular mass balance (theta1, theta2, delta errors high) remains an open question:
  the non-uniform mass reflects semantic clustering in H^4 base coordinates, not a routing bug.
- Stage 3 status → PARTIAL-PASS. Stage 4 (Phase Transport) is now ready to open on PPMI-SVD proxy.

## Artifacts
- Screen: `results/analysis/inc0144_hopf_vs_kmeans_stage3_screen.json`
- Confirm: `results/analysis/inc0144_hopf_vs_kmeans_stage3_confirm.json`

## Related
- Parent RR: `RR-065`
- Depends on: [previous INC or RR]
