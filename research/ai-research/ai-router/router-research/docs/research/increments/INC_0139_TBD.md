# INC-0139: Does Hopf Fiber Balance Variation Drive Semantically Meaningful Shell Assignment?

## Status
Closed: REFINE. SO(8) nominally passes the shell-discrimination threshold but
via generic concentration (both inputs concentrated into fewer shells, pmax_after
collapses from ~0.50 to ~0.10). The fiber balance path is exhausted. Stage 2
formally redirects to the angular law (INC-0140).

## Summary
Post-INC-0138 structural finding: all embeddings are L2-normalized (`||v||=1.0`),
so the chart radius `r ≡ 1.0` (identity chart). Shells form ONLY because the Hopf
fiber balance `balance=(rho1-rho2)/(rho1+rho2)` varies across embeddings and the
`adaptive_shell_growth` parameter amplifies this into effective-radius spread.

The column-permuted control produces similar `balance` distributions because
column permutation preserves each column's marginal, and `balance` depends only
on marginal norms of column pairs.

**The corrected test for Stage 2:** Can chart rotation (SO(8) learning) concentrate
Hopf fiber energy asymmetry in ways that make shell assignment semantically
discriminating — i.e., does the learned chart create a fiber balance distribution
that differs between real and col-perm inputs?

This increment must decide:

**Option A (LEARN):** Run with `learn_so8=1` and measure whether the learned rotation
changes the `balance` distribution enough to make shells discriminate real from col-perm.

**Option B (ACCEPT CONSTRAINT):** Accept that on unit-norm embeddings with identity chart,
shells are fiber-balance-driven and fiber balance is marginal-norm-driven — hence
semantically blind. Document this as a Stage 2 architectural constraint and formally
redirect Stage 2 to the angular law (where real vs col-perm separation was observed).

## Kill-List Stage
Primary: 2. Measure-Consistent Shell Routing

## Mathematical Object Under Test
- First-factor H^4 routing manifold, Hopf fiber balance field
- Whether SO(8) chart learning changes `balance` distribution between real and col-perm inputs
- Identity chart (learn_so8=0): baseline — fiber balance is marginal-norm-driven
- Learned chart (learn_so8=1): test — does learned rotation create semantic fiber asymmetry?

## Success Condition
With `learn_so8=1`, the shell-level separation |shell_pmax(GEOM_ORIG) - shell_pmax(GEOM_COL_PERM)| > 0.05
across 2 seeds, OR the `balance` distribution differs between real and col-perm by KL > 0.1.
This would show the geometry (via chart learning) can make fiber balance semantically meaningful.

## Falsification Condition
With `learn_so8=1`, shell metrics remain indistinguishable between real and col-perm inputs
(|shell_pmax diff| < 0.02). This confirms fiber balance on unit-norm embeddings is
structurally marginal-norm-driven regardless of rotation, and Stage 2 must redirect
to the angular law as the primary semantic routing dimension.

## Trigger
INC-0138 Closed: REFINE (2026-03-13) + post-experiment structural finding:
`r ≡ 1.0` on unit-norm embeddings with identity chart; shells driven by Hopf
fiber balance via adaptive_shell_growth; column permutation preserves balance
distribution because it preserves column marginals.

## Config
`configs/proxy_transfer_inc0139_so8_fiber_balance_screen.json`
Routes: GEOM_ORIG, GEOM_COL_PERM, LEARN_ORIG, LEARN_COL_PERM
Seeds: 0, 1 (screen)

## Results (2-seed means)

| Route | learn_so8 | transform | shells | shell_pmax | pmax_after | buckets | sec_ent |
|---|---|---|---|---|---|---|---|
| GEOM_ORIG | 0 | none | 2.00 | 0.5846 | 0.5284 | 15.5 | 1.3220 |
| GEOM_COL_PERM | 0 | col_perm | 2.00 | 0.5320 | 0.4222 | 29.0 | 1.3231 |
| LEARN_ORIG | 1 | none | 2.00 | 0.7848 | 0.0966 | 45.0 | 3.0706 |
| LEARN_COL_PERM | 1 | col_perm | 2.50 | 0.7226 | 0.0992 | 50.5 | 2.9913 |

**Key comparison:**
- Identity chart: |GEOM_ORIG - GEOM_COL_PERM| shell_pmax = **0.0526** (baseline)
- SO(8) learned: |LEARN_ORIG - LEARN_COL_PERM| shell_pmax = **0.0622** (> 0.05 threshold)

## Analysis

**Technical criterion met:** |diff| = 0.0622 > 0.05 threshold.

**But the mechanism is degenerate:**

1. SO(8) learning increases shell_pmax for BOTH input types by ~+0.19 (real: 0.58→0.78,
   col-perm: 0.53→0.72). This is a generic concentration effect — the rotation drives
   all embeddings toward a single dominant shell regardless of semantic content.

2. `pmax_after` collapses from ~0.50 (identity chart) to ~0.10 (SO(8) learned). After shell
   assignment, the bucket-level routing becomes nearly uniform (random). Routing quality
   is destroyed, not improved.

3. `buckets` explodes from 15.5 → 45.0 (real) and 29.0 → 50.5 (col-perm). The SO(8) chart
   creates many micro-buckets with no discriminating structure.

4. `sector_entropy` doubles (1.32 → 3.07). Sector routing becomes near-uniform.

5. The incremental |diff| improvement over identity chart is only 0.0622 - 0.0526 = **0.0096**
   — well below any meaningful signal level.

**Conclusion:** SO(8) learning concentrates fiber balance degenerate — it rotates the
embedding space so that the fiber balance metric collapses all inputs toward one extreme,
not by distinguishing semantic content but by finding a generic "most concentrated" rotation.
The 0.0622 detection is not semantically meaningful discrimination; it is an artifact of
different concentration behaviors in real vs structurally-randomized inputs.

**Main result:** Hopf fiber balance cannot be made semantically discriminating through
SO(8) chart rotation on L2-normalized embeddings without destroying routing quality.
The fiber balance direction is exhausted as a Stage 2 routing primitive.

**Structural constraint established (INC-0136 through INC-0139):**
- INC-0136: Direct geodesic shell substitution KILLED (shell_pmax=0.886)
- INC-0137: Shell pressure blend KILLED (all blends worse than baseline)
- INC-0138: r ≡ 1.0; shells are fiber-balance-driven, not radius-driven
- INC-0139: SO(8) amplifies generic concentration, not semantic discrimination
→ Shell routing on L2-normalized embeddings is a structural dead end for Stage 2.

**Redirect:** Stage 2 now formally tests the angular routing law as the primary
measure-consistent primitive. INC-0138 showed angular routing discriminates real
from structured noise (buckets=15.5 vs 50.0 Gaussian, meaningful sector_entropy gap).
INC-0140 will test whether sector-level routing is measure-consistent with the H^4
angular measure (Hopf fiber + delta + chi angles).

