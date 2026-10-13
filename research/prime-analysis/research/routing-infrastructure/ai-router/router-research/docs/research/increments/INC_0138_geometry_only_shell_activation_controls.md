# INC-0138: Geometry-Only Shell Activation — Real vs Destructive Controls

## Status
Closed: REFINE.

Screen experiment `inc0138_geometry_only_shell_activation_screen` completed
(2 seeds, 4 routes). Fixed H^4 geometry + adaptive shell activation produces
non-collapsed 2-shell structure. Real embeddings separate from Gaussian noise
via bucket concentration, but do NOT separate from column-permuted controls
at the shell level. Shell structure is driven by norm geometry, not semantic content.

### Post-experiment structural finding (2026-03-13)
**All embeddings are L2-normalized: `||v|| = 1.0` (std ≈ 2.7e-8).**

This means the chart radius `r = safe_norm(route_z) ≡ 1.0` for all vectors
when `learn_scale=0, learn_so8=0` (identity chart, rotation preserves norms).

Shells therefore CANNOT form from radial magnitude variation alone.

Verified empirically:
- `adaptive_shell_growth=0.0`: `shell_multiplier=1.0` for all, 1 shell, `shell_pmax=1.0`
- `adaptive_shell_growth=1.6`: `shell_multiplier` varies (std=0.414, range 1.96–2.85),
  2 shells form, `shell_pmax=0.595`

The mechanism is:
```
shell_multiplier = exp(adaptive_shell_growth * div_score * (1 + adaptive_shell_balance * |balance|) - shell_converge)
balance = (rho1 - rho2) / (rho1 + rho2)   [Hopf fiber energy asymmetry]
rho1 = ||(z[0], z[2])||,  rho2 = ||(z[4], z[6])||
r_eff = r * shell_multiplier  [r=1, so r_eff = shell_multiplier]
```

**The Hopf fiber balance DOES vary significantly:** mean=0.066, std=0.621, range [-1, +1]. This is the geometric driver of shell spread — not the hyperbolic radius.

**Restated: shells form because Hopf fiber energy is asymmetric across embeddings, and `adaptive_shell_growth` amplifies this asymmetry into effective-radius spread.**

Column-permuted controls produce similar `balance` distributions because column permutation preserves each column's marginal distribution, and `balance` depends on marginal norms of column pairs — hence the shell-level indistinguishability.

**The corrected question for Stage 2:**
Does Hopf fiber balance variation (from real semantic embedding geometry) produce meaningfully different shell assignments than Hopf balance variation from semantically-destroyed controls? And can chart rotation learning (SO(8)) enhance fiber balance variation to make shells semantically discriminating?

### Screen Results Summary (2026-03-13)

| Route | eval_shells | shell_pmax | shell_entropy | pmax_after | sector_entropy | buckets | test_unseen_rate | test_mse_after | hopf_base_mass_error | hopf_angular_mass_error |
|---|---|---|---|---|---|---|---|---|---|---|
| GEOM_ORIG (original) | 2 | 0.584 | 0.679 | 0.529 | 1.322 | 15.5 | 0.001 | 0.0039 | 1.005 | 1.281 |
| GEOM_COL_PERM (col-permuted) | 2 | 0.532 | 0.691 | 0.422 | 1.323 | 29.0 | 0.000 | 0.0039 | 1.004 | 1.280 |
| GEOM_GAUSSIAN (matched noise) | 2 | 0.645 | 0.651 | 0.048 | 3.195 | 50.0 | 0.000 | 0.0040 | 0.134 | 0.338 |
| R0 (kmeans, learned ref.) | 1 | 1.000 | 0.000 | 0.261 | 1.953 | 8.0 | 0.000 | 0.0039 | — | — |

Conditions: `sector_mode=phase4d_hopf_base`, `learn_so8=0`, `learn_scale=0`,
`adaptive_shell_growth=1.6`, `adaptive_shell_balance=1.0`

Result file: `results/analysis/inc0138_geometry_only_shell_activation_screen.json`

### Interpretation

**1. Shell collapse?**
No for all geometry routes. R0 (learned kmeans) collapsed to 1 shell.
Fixed geometry produces stable 2-shell structure at `shell_pmax ≈ 0.58`.

**2. Nontrivial shell structure from fixed geometry + threshold activation?**
Yes. Two stable, non-collapsed shells with consistent shell_entropy ≈ 0.68
across both seeds. The R0 collapse (shell_pmax=1.0) confirms the geometry-only
path is strictly better at maintaining shell distribution than the kmeans baseline.

**3. Did original embeddings separate from controls?**
Partially — in the sector/bucket dimension, not the shell dimension:
- `buckets`: GEOM_ORIG=15.5 vs GEOM_COL_PERM=29.0 vs GEOM_GAUSSIAN=50.0
  Real embeddings use 3× fewer buckets than Gaussian noise.
- `pmax_after`: GEOM_ORIG=0.529 vs GEOM_GAUSSIAN=0.048
  Real embeddings concentrate strongly; Gaussian spreads uniformly.
- `sector_entropy`: GEOM_ORIG=1.32 vs GEOM_GAUSSIAN=3.20
  Real embeddings cluster angularly; Gaussian is near-uniform.
- Shell level: GEOM_ORIG and GEOM_COL_PERM nearly identical on all shell metrics.
  Column permutation preserves norms and hence the radial/shell assignment.
  Shell activation responds to norm geometry, not embedding semantics.

**4. Structure from geometry alone, geometry + threshold, or neither?**
Geometry + threshold dynamics produces the bucket-level structure.
Shell structure is driven by norm geometry (not semantic content).
Angular structure (Hopf base) shows weak semantic separation: real vs Gaussian
is large, real vs col-permuted is small.

**Key finding:** The geometry-only shell activation does produce reproducible,
non-collapsed routing structure. The separation real vs Gaussian is strong at
basket/sector granularity. However, the discriminating dimension is the
Hopf-base angular assignment, not the shell level. The shell level alone
cannot tell real from semantically-destroyed (col-perm) inputs.

### Decision Rationale
REFINE: the experiment answers the primary theory question positively (structure
does emerge from geometry alone, no learned weights), but reveals that the
meaningful structure lives in the angular/Hopf dimension, not the shell dimension.
The next increment should test whether the column-permuted failure at the shell
level can be broken by changing shell law parameters, or whether it is fundamental
to the norm-preserving nature of column permutation.

Stage 2 (`measure-consistent shell routing`) remains OPEN: shell_pmax=0.584 >
INC-0137 baseline (0.5222), and the control comparison shows the shell law behaves
as expected, but we have not yet achieved a meaningfully lower shell_pmax.

### Cross-Stage Observation
The Hopf-base angular dimension (sector_entropy, pmax_after, buckets) is the
primary carrier of semantic structure in the fixed-geometry path. This has direct
implications for Stage 3 (Hopf Angular Correctness): improving the angular law
is more urgent than improving the shell law for semantic routing quality.
Record in SESSION_LEDGER.md only — do not diverge on this branch.

### Next Step
INC-0139: test whether the col-perm/orig shell indistinguishability can be broken
by making the shell law more sensitive to angular-norm joint geometry (e.g.,
increasing K, tightening delta_r, or using phi_log with a higher growth parameter),
OR accept that shell assignment is fundamentally norm-driven and focus Stage 2
effort on the angular law.

## Trigger
`ACTIVE_STATE.md` queued INC-0138 as a shell-density controller / occupancy
equalizer. The task was broadened to include the primary theory question
(geometry-only vs controls) as an explicit baseline before further controller
engineering.

## Kill-List Stage
Primary: 2. Measure-Consistent Shell Routing
Secondary: 3. Hopf Angular Correctness (cross-stage observation)

## Mathematical Object Under Test
- First-factor H^4 routing manifold
- Adaptive shell activation `(adaptive_shell_growth=1.6, adaptive_shell_balance=1.0)`
- No learned routing weights (`learn_so8=0, learn_scale=0`)
- Controls: column-permuted (destroys semantic structure, preserves norms),
  Gaussian matched-mean/std (destroys both structure and norms)
- The test is whether non-trivial routing structure emerges from the geometry
  alone and whether that structure separates real from destroyed embeddings

## Hypothesis
Fixed H^4 geometry + threshold-like shell activation without any learned routing
weights should produce meaningful routing structure if the geometry is doing real
work. A destructive control (column permutation, Gaussian) should produce
qualitatively different routing behavior if structure is geometry-driven.
