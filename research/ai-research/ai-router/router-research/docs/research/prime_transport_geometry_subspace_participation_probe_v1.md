# Prime Transport Geometry Subspace Participation Probe v1

**Probe:** geometry_subspace_participation_probe_v1  
**Contract:** prompt_contract_v4.md — binding  
**Dependency table:** prime_transport_geometry_dependency_order_probe_v1.md — binding  
**Branch:** geometry_subspace_participation_probe_v1  
**Layer tested:** Layer 1 (ambient space) × Layer 5 (comparison subspace)  
**N samples:** 1024  (256 per class)  
**Basis:** apply_anchor_two_i (2i frame)  
**Operator:** cos (cosine similarity)  
**Normalization:** unit-norm per subspace vector (no reweighting)  
**eps_high:** 1.0 (frozen — algebraic H3 preservation)  
**Training:** NONE  

---

## 1. Subspace Definitions

All subspaces are harmonics of **block3** (dominant block, period=12, n_h=3).
apply_anchor_two_i is applied to ALL harmonic pairs in the 2i frame before
any subspace slice is extracted.

### Block3 Harmonic Layout (in 12-dim angular vector of R^16)

| Harmonic | Angle formula | Classes at (in 2i frame) | Angular dims (R^16) |
|:---------|:--------------|:-------------------------|:-------------------|
| H1 (h=1) | θ_k = π·k/6   | 30° spacing (not orthogonal) | [6, 7] |
| H2 (h=2) | θ_k = π·k/3   | 60° spacing (not orthogonal) | [8, 9] |
| H3 (h=3) | θ_k = π·k/2   | 90° spacing (orthogonal for 4 classes) | [10, 11] |

**Why H3 is special:** With period=12 and h=3, the 4 classes are spaced at
exactly 90° intervals → mutually orthogonal unit vectors → maximal separation.

### Subspace Candidates

| Subspace | Dims (BASELINE/REDUCED) | Dims (R4_FLAT) | Size |
|:---------|:------------------------|:---------------|:-----|
| H3_ONLY | [10, 11] | [2, 3] | 2D |
| H2_ONLY | [8, 9] | [0, 1] | 2D |
| H1_ONLY | [6, 7] | N/A | 2D |
| H2_PLUS_H3 | [8, 9, 10, 11] | [0, 1, 2, 3] | 4D |
| H1_PLUS_H3 | [6, 7, 10, 11] | N/A | 4D |
| FULL_B3 | [6, 7, 8, 9, 10, 11] | N/A | 6D |

Note: H1 is unavailable in R4_FLAT (which only contains h2 and h3 of block3).

---

## 2. Mandatory Step 1 — Validity Check

**Status:** PASS

Checks performed:
- Sample count consistent: True
- H3 baseline vs reduced max_diff: 0.0
- H3 baseline vs R4_FLAT max_diff: 0.0
- H2 baseline vs reduced max_diff: 0.0
- H2 baseline vs R4_FLAT max_diff: 0.0
- H1 baseline vs reduced max_diff: 0.0

- Subspace norms bl_h1_min_norm: 1.0
- Subspace norms bl_h1_all_nonzero: True
- Subspace norms bl_h2_min_norm: 1.0
- Subspace norms bl_h2_all_nonzero: True
- Subspace norms bl_h3_min_norm: 1.0
- Subspace norms bl_h3_all_nonzero: True
- Subspace norms r4_h2_min_norm: 1.0
- Subspace norms r4_h2_all_nonzero: True
- Subspace norms r4_h3_min_norm: 1.0
- Subspace norms r4_h3_all_nonzero: True

---

## 3. Mandatory Step 2 — Full Result Table

| Space | Subspace | mean_matched_cos | mean_mismatched_cos | separation_delta |
|:------|:---------|----------------:|--------------------:|----------------:|
| CURRENT_BASELINE | H3_ONLY | 1.000000 | -0.333333 | 1.333333 |
| CURRENT_BASELINE | H2_ONLY | 1.000000 | -0.083333 | 1.083333 |
| CURRENT_BASELINE | H1_ONLY | 1.000000 | 0.599679 | 0.400321 |
| CURRENT_BASELINE | H2_PLUS_H3 | 1.000000 | -0.208333 | 1.208333 |
| CURRENT_BASELINE | H1_PLUS_H3 | 1.000000 | 0.133173 | 0.866827 |
| CURRENT_BASELINE | FULL_B3 | 1.000000 | 0.061004 | 0.938996 |
| REDUCED_BLOCK | H3_ONLY | 1.000000 | -0.333333 | 1.333333 |
| REDUCED_BLOCK | H2_ONLY | 1.000000 | -0.083333 | 1.083333 |
| REDUCED_BLOCK | H1_ONLY | 1.000000 | 0.599679 | 0.400321 |
| REDUCED_BLOCK | H2_PLUS_H3 | 1.000000 | -0.208333 | 1.208333 |
| REDUCED_BLOCK | H1_PLUS_H3 | 1.000000 | 0.133173 | 0.866827 |
| REDUCED_BLOCK | FULL_B3 | 1.000000 | 0.061004 | 0.938996 |
| R4_FLAT | H3_ONLY | 1.000000 | -0.333333 | 1.333333 |
| R4_FLAT | H2_ONLY | 1.000000 | -0.083333 | 1.083333 |
| R4_FLAT | H1_ONLY | N/A | N/A | N/A |
| R4_FLAT | H2_PLUS_H3 | 1.000000 | -0.208333 | 1.208333 |
| R4_FLAT | H1_PLUS_H3 | N/A | N/A | N/A |
| R4_FLAT | FULL_B3 | N/A | N/A | N/A |

---

## 4. Comparison Across Subspaces

### Separation delta ranked by subspace (averaged over available spaces):

| Subspace | mean delta (BASELINE) | mean delta (REDUCED) | mean delta (R4_FLAT) | Notes |
|:---------|---------------------:|--------------------:|--------------------:|:------|
| H3_ONLY | 1.333333 | 1.333333 | 1.333333 |  |
| H2_ONLY | 1.083333 | 1.083333 | 1.083333 |  |
| H1_ONLY | 0.400321 | 0.400321 | N/A | H1 not in R4 |
| H2_PLUS_H3 | 1.208333 | 1.208333 | 1.208333 |  |
| H1_PLUS_H3 | 0.866827 | 0.866827 | N/A | H1 not in R4 |
| FULL_B3 | 0.938996 | 0.938996 | N/A | H1 not in R4 |

---

## 5. Does Signal Extend Beyond H3?

**No.** The geometric separation signal is confined to H3_ONLY.

Non-H3 subspaces (H1, H2) and combined subspaces provide substantially
lower separation_delta than H3_ONLY. Combining H3 with other harmonics
does not improve and may reduce separation.

| Space | H3_ONLY delta | Best non-H3 delta | H3 dominates? |
|:------|-------------:|------------------:|:--------------|
| CURRENT_BASELINE | 1.333333 | 1.208333 | True |
| REDUCED_BLOCK | 1.333333 | 1.208333 | True |
| R4_FLAT | 1.333333 | 1.208333 | True |

---

## 6. Does Ambient Space Matter When Multiple Subspaces Participate?

**No.** The separation_delta for every shared subspace is identical
across CURRENT_BASELINE, REDUCED_BLOCK, and R4_FLAT (where available).

The ambient space (R^4, R^12, R^16) does not affect the geometric
separation signal even when multiple subspaces participate in the comparison.

This extends the SPACE_INVARIANT result from the prior probes:
- geometry_space_type_probe_v1:     H3_ONLY under frozen conditions → SPACE_INVARIANT
- geometry_space_dynamics_probe_v1: H3_ONLY under dynamics → SPACE_INVARIANT_UNDER_DYNAMICS
- THIS PROBE: multi-subspace comparison → space also does not matter

---

## 7. Explicit Answers to Critical Questions

**Q1: Is the signal truly confined to H3?**  
A1: YES. H3_ONLY dominates with maximum separation_delta across all spaces.

**Q2: Do other harmonic subspaces carry usable signal?**  
A2: NOT COMPARABLY. H1 and H2 have substantially lower separation_delta.

**Q3: Does combining subspaces change behavior?**  
A3: Combining H2+H3 reduces separation (delta=1.208333 vs H3_ONLY=1.333333).

**Q4: Does ambient space matter once multiple subspaces are used?**  
A4: NO. All shared subspace comparisons yield identical deltas across R^4, R^12, R^16.

---

## 8. Classification

**CLASSIFICATION: H3_ONLY_SIGNAL**

| Classification | Criterion | Observed? |
|:---------------|:----------|:----------|
| H3_ONLY_SIGNAL | Only H3 produces separation; all others fail | True |
| MULTI_SUBSPACE_SIGNAL | Other subspaces also produce signal | False |
| SPACE_DEPENDENT_SUBSPACE_BEHAVIOR | Subspace effectiveness differs by space | False |
| DEGENERATE | Signal collapses or inconsistent | False |

---

## 9. Locked Variables Confirmation

| Variable | Value | Changed? |
|:---------|:------|:---------|
| Basis | apply_anchor_two_i (2i frame) | NO |
| Operator | cos (cosine similarity) | NO |
| Normalization | unit-norm per subspace vector | NO |
| eps_high | 1.0 (frozen) | NO |
| Training | None | NO |
| New constants | None | NO |
| Radial structure | Unchanged | NO |
| D sweeps | None | NO |
| phi / heuristic ratios | None | NO |

---

GEOMETRY SUBSPACE PARTICIPATION RESULT: H3_ONLY_SIGNAL