# INC-0141: Optimal-Dim Hopf Angular Routing — Max-Correlation 4D Subspace

## Status
Closed: KILL. Routing with optimal-dim (46,117,62,78) Hopf subspace does not discriminate real from col-perm embeddings; pre-screen TV=0.109 signal does not survive routing (OPT_ORIG pmax=0.379 vs OPT_COL_PERM pmax=0.388, rel_diff=0.025, wrong direction). Hash embedding lacks semantic angular structure required for Stage 2 discrimination regardless of dim choice.

## Results (2026-03-13 — screen, 2 seeds)

Config: `configs/proxy_transfer_inc0141_optimal_dim_hopf_screen.json`  
Commit: `00d65da` (branch: `codex/RR-061-measure-consistent-route-law`)

| route_id    | phase4_dims    | input_transform | pmax_after (mean) | seeds           | sector_entropy |
|-------------|----------------|-----------------|-------------------|-----------------|----------------|
| OPT_ORIG    | 46,117,62,78   | none            | 0.3786            | [0.3896, 0.3676] | 1.6308        |
| OPT_COL_PERM| 46,117,62,78   | col_perm        | 0.3880            | [0.3968, 0.3792] | 1.6681        |
| OPT_GAUSSIAN| 46,117,62,78   | gaussian        | 0.1438            | [0.1456, 0.1420] | 2.8937        |
| CTRL_0246   | 0,2,4,6        | none            | 0.6006            | [0.5936, 0.6076] | 1.3220        |

**Key comparison (Stage 2 criterion):**  
- OPT_ORIG pmax_after = 0.3786  
- OPT_COL_PERM pmax_after = 0.3880  
- |diff| = 0.0094, rel_diff = **0.025** (threshold: 0.2)  
- Direction: WRONG — ORIG < PERM (col-perm is MORE concentrated than real)  
- CTRL_0246 pmax=0.6006 exactly replicates INC-0140 ANG_ORIG (confirms experimental consistency)

## Analysis

**Pre-screen prediction failed:** Pre-screen on the val set showed ORIG pmax=0.338 > PERM pmax=0.305 (TV=0.109), direction correct. On the test set with 2500 tokens, the direction reverses. 

**Root cause of failure:** The pre-screen TV=0.109 signal was real (within-pair correlation destroys sector distribution differently than marginals) but the SIGN of the effect reverses across splits. This indicates the "signal" was sampling noise on a fundamentally uniform distribution: the hash feature at dims (46,117,62,78) concentrates differently in val vs test. Neither direction is semantically meaningful.

**Cascade conclusion (INC-0140 + INC-0141):**  
Both INC-0140 (dims 0,2,4,6) and INC-0141 (optimal dims 46,117,62,78) kill at rel_diff < 0.03. The root cause is not dim selection — it is that the wikitext2 HASH embedding has no semantic angular structure in ANY 4D Hopf subspace. Hash hashing is designed to produce uniform, isotropic coverage; by construction it cannot produce semantically concentrated Hopf sectors. Stage 2 is proxy-task-blocked.

## Pre-screen Findings (2026-03-13)

### Hypothesis correction: raw embeddings (VOID)
The INC-0141 queue doc originally proposed testing raw (non-L2-normalized) embeddings.
This hypothesis was voided by a mathematical proof before running any experiment:

**chi_u and delta are provably scale-invariant:**
- `chi_u = rho2^2 / (rho1^2 + rho2^2)` — ratio of quadratic forms, homogeneous degree 0
- `delta = arctan2(z_j, z_i) - arctan2(z_l, z_k)` — arctan2 is scale-invariant

Therefore: for any scalar c > 0, z → c·z gives identical chi_u and delta.
L2-normalization is a global scaling, so raw embeddings give EXACTLY the same Hopf
coordinates. The INC-0140 "root cause" attribution to L2-normalization was incorrect.

**Verified empirically:** pre-screen script shows chi_u and delta distributions are
numerically identical between raw and L2-normalized embeddings (RAW == NORM to 4 decimal
places).

### Correct root cause: hash embedding is isotropic at (0,2,4,6)
Dims (0,2,4,6) used in INC-0140 had corr ≈ −0.04 (near-zero). Phase angle arctan2(z_j,z_i)
concentrates only when the two dimensions are correlated. Near-zero correlation → near-isotropic
phase angle distribution → no routing discrimination.

### New finding: hash embedding has high-correlation dim pairs elsewhere
Full 128×128 correlation matrix shows:
- dims (46, 117): |corr| = 0.479  ← highest
- dims (46, 103): |corr| = 0.319
- dims (18, 46):  |corr| = 0.264
- dims (62, 78):  |corr| = 0.202
- mean off-diagonal |corr| = 0.024

Best 4D subspace (46, 117, 62, 78):
- TV distance under col-perm = 0.109  (vs 0.010 for dims 0,2,4,6)
- pmax ORIG = 0.338, pmax PERM = 0.305 → rel_diff = 0.097 (below 0.2 threshold)
- Direction: ORIG > PERM ✓ (semantically concentrated → more concentrated routing)

This is the strongest Stage 2 discrimination signal found so far. Routing may amplify
the 9.7% raw sector difference to exceed the 20% success threshold.

## Summary
INC-0136 through INC-0140 failed to discriminate real from col-perm embeddings using
dims (0,2,4,6). Pre-screen of INC-0141 identifies that:
1. L2-normalization is irrelevant (scale-invariance proof)
2. The hash embedding has structured correlations at other dims
3. Dims (46,117,62,78) provide TV=0.109 sector discrimination — first genuine signal

**This increment tests whether routing with optimal dims (46,117,62,78) reaches the
Stage 2 success threshold after full training, amplifying the 9.7% pre-screen signal.**

## Kill-List Stage
Primary: 2. Measure-Consistent Shell Routing (testing optimal-dim Hopf routing)

## Mathematical Object Under Test
- First-factor H^4 routing manifold, Hopf base projection
- Within-pair angular correlations in the hash embedding at dims (46,117,62,78)
  vs (0,2,4,6): corr=−0.479/0.202 vs corr=−0.039/−0.018
- Whether routing with high-correlation dims amplifies pre-screen TV=0.109 signal
  to pmax_after discrimination ≥ 0.2

## Success Condition
With `sector_mode=phase4d_hopf_base`, `phase4_dims=46,117,62,78`, `learn_so8=0`:
- |pmax_after ORIG - pmax_after COL_PERM| / mean > 0.2

This would demonstrate that H^4 Hopf angular routing IS semantically discriminative
when the embedding has within-pair angular correlations, resolving Stage 2 by identifying
what embedding structure the routing geometry requires.

## Falsification Condition
pmax_after|ORIG-PERM|/mean < 0.1 — routing does not amplify the pre-screen TV=0.109
signal. Stage 2 is then proxy-task-blocked: the hash embedding lacks sufficient angular
structure for H^4 routing discrimination regardless of which 4D subspace is chosen,
and Stage 2 validation requires semantically structured embeddings (LLM activations).

## Trigger
INC-0140 Closed: KILL (2026-03-13). Pre-screen for INC-0141 (2026-03-13):
- Scale-invariance proof voids raw-embedding hypothesis
- Max-correlation search identifies dims (46,117,62,78) with TV=0.109 pre-screen signal
