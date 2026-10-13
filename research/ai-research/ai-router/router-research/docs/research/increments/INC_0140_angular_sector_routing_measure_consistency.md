# INC-0140: Measure-Consistent Angular Routing — Does Sector Assignment Discriminate Real Structure?

## Status
Closed: KILL.

Angular sector routing (phase4d_hopf_base, learn_so8=0) is measure-degenerate on
L2-normalized embeddings. ANG_ORIG and ANG_COL_PERM are indistinguishable at the
prediction level despite per-sample routing differing by 66%. Degeneracy is genuine
(confirmed by forensic audit). L2-normalization destroys within-pair angular signal.

## Experiment Results (screen, 2026-03-13, commit 881f87b)

| Route | pmax_after | sector_entropy | eval_shells | description |
|---|---|---|---|---|
| ANG_ORIG | 0.6006 | 1.322 | 1 | real inputs, no shells |
| ANG_COL_PERM | 0.6030 | 1.323 | 1 | col-perm inputs, no shells |
| ANG_GAUSSIAN | 0.0582 | 3.195 | 1 | Gaussian noise baseline |
| ANG_ORIG_SHELLS | 0.5284 | 1.322 | 2 | real inputs, shells enabled |

**Separation ratios:**
- |pmax_after diff| / mean (ORIG vs COL_PERM): 0.004 — **indistinguishable** (threshold: 0.2)
- |sector_entropy diff| / mean (ORIG vs COL_PERM): 0.0008 — **indistinguishable**
- ORIG vs GAUSSIAN pmax_after ratio: 10.3× — valid noise separation

**Falsification condition met.** Sector metrics are indistinguishable between real
and col-perm inputs.

## Forensic Audit (2026-03-13)

**Question:** Is the indistinguishability genuine (theoretical finding) or an artifact
of col-perm not actually changing the routing?

**Code audit findings:**
- `_make_chart` with `learn_so8=0, learn_scale=0` → `R = np.eye(d)`, no scale →
  `z = v` exactly (raw L2-normalized inputs, unmodified)
- Col-perm implementation: `v[:,j] = rng.permutation(v[:,j])` for each j independently —
  this independently shuffles columns 0, 2, 4, 6, destroying within-pair correlations
- Hopf coordinate path is unchanged: `phase4d_hopf_base` → `assign_sectors_phase4d_hopf_base`
  → chi_u × delta grid (kchi=5, kdelta=5, K=25)

**Empirical diagnostic (eval split, N=40,000):**
| Metric | Value | Interpretation |
|---|---|---|
| chi_u KS-stat (ORIG vs COL_PERM) | 0.194 | Large: col-perm changes chi_u distribution |
| delta KS-stat (ORIG vs COL_PERM) | 0.621 | Very large: col-perm changes delta distribution |
| Sector assignment agreement | 34% (random=4%) | 66% of per-sample sectors change |
| Sector count TV distance | 0.009 | Sector SIZE distribution nearly identical |
| corr(z[0], z[2]) | −0.039 | Near-zero within-pair correlation |
| corr(z[4], z[6]) | −0.018 | Near-zero within-pair correlation |

**Audit verdict:** The kill is GENUINE. The col-perm test is valid (it changes 66% of
sector assignments). The indistinguishability arises because:

1. pmax_after is governed by sector CONCENTRATION (sector_entropy=1.322, max=4.64),
   not by semantic alignment of specific samples to sectors.
2. The sector SIZE distribution has TV=0.009 between ORIG and PERM — nearly identical.
3. A routing with near-identical sector sizes will produce near-identical pmax_after
   regardless of which specific samples are assigned to which sector.
4. The within-pair correlations (corr ≈ −0.04, −0.02) are near-zero because
   L2-normalization collapses all points onto S^127 — the specific 4-dimensional
   Hopf projection subspace (dims 0,2,4,6) sees no structured within-pair angular signal.

**The root constraint:** Fixed Hopf base projection onto a specific 4D subspace
(z[0],z[2],z[4],z[6]) of a 128D L2-normalized embedding is near-isotropic in those
dimensions. The angular signal needed for semantic routing is destroyed by L2 normalization
spreading the embedding mass isotropically across all dimensions.

## Summary
INC-0136 through INC-0139 collectively establish that the Hopf fiber balance
(shell routing dimension) is a structural dead end for Stage 2 on L2-normalized
embeddings:

- **INC-0136:** Direct geodesic shell substitution KILLED (shell_pmax=0.886)
- **INC-0137:** Shell pressure blend KILLED (all weights worsen routing vs baseline)
- **INC-0138:** r ≡ 1.0 — shells are fiber-balance-driven, not radius-driven;
  real vs col-perm INDISTINGUISHABLE at shell level
- **INC-0139:** SO(8) learning nominally passes discrimination threshold (|diff|=0.0622)
  but via generic concentration; routing quality destroyed (pmax_after collapses to 0.10)

**Stage 2 redirect:** The angular routing law (sector assignment via Hopf base
coordinates delta, chi, theta1/theta2) showed quantitatively meaningful real vs
Gaussian separation in INC-0138 (buckets=15.5 vs 50.0 Gaussian, ~3× difference;
sector_entropy gap). This increment formally tests whether sector-level angular
routing is measure-consistent with the H^4 angular measure.

**The new test for Stage 2:** Does angular sector routing distribute tokens
consistently with the H^4 surface measure over the Hopf base S^3 — and does
it discriminate semantic structure (real embeddings) more than col-perm controls?

## Kill-List Stage
Primary: 2. Measure-Consistent Shell Routing (redirect to angular law)
Cross-reference: 3. Hopf Angular Correctness (overlapping question)

## Mathematical Object Under Test
- First-factor H^4 routing manifold, Hopf base projection (delta, chi, theta angles)
- Sector assignment consistency with H^4 angular measure on S^3 base
- Whether sector routing produces semantically meaningful distribution differences
  between real vs col-perm inputs at the sector level (not shell level)

## Success Condition
With `sector_mode=phase4d_hopf_base`, `learn_so8=0`, `learn_scale=0`:
- Sector-level `pmax_after` or `sector_entropy` DIFFERS between GEOM_ORIG and
  GEOM_COL_PERM by more than double the noise floor (|diff|/mean > 0.2), OR
- The Hopf angular mass error (`mean_hopf_base_mass_error`, `mean_hopf_angular_mass_error`)
  is meaningfully lower for real embeddings than for col-perm
This would show angular routing carries semantic information that shells cannot.

## Falsification Condition
Sector metrics (pmax_after, sector_entropy, hopf mass errors) are indistinguishable
between real and col-perm inputs at the sector level. This would confirm that Stage 2
cannot be resolved via current routing geometry alone and the resolution must come
from a different architectural modification (non-L2-normalized embeddings, or a
fundamentally different sector law).

## Trigger
INC-0139 Closed: REFINE (2026-03-13) — fiber balance / SO(8) path exhausted.
Angular sector dimension showed real vs Gaussian separation in INC-0138 but
the real vs col-perm comparison at sector level was not fully characterized.

## Decision
Closed: KILL.

The falsification condition is met. Angular sector routing (phase4d_hopf_base,
learn_so8=0) cannot discriminate real from col-perm embeddings at the sector level.

Stage 2 conclusion after INC-0136 through INC-0140:
- Shell routing (radial/fiber-balance): DEAD (INC-0136, 0137, 0138)
- Angular sector routing without learned alignment: DEAD (INC-0140)
- Learned SO(8) alignment: destroys routing quality (INC-0139, pmax 0.5→0.1)

**Root structural finding:** L2-normalization collapses all embeddings onto S^127.
Fixed Hopf projection onto a specific 4D subspace (dims 0,2,4,6) is near-isotropic
(within-pair correlations ≈ −0.04, −0.02) — no structured within-pair angular signal
exists to route by.

**Next increment (INC-0141):** Test whether removing L2-normalization (raw embeddings)
restores within-pair Hopf angular structure sufficient for measure-consistent routing.
This is the minimal corrective action not yet tested. If raw embeddings also fail, Stage 2
must be declared structurally blocked and the kill-list stage should be revised.
