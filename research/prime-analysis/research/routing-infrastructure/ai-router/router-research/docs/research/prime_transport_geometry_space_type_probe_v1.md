# Prime Transport Geometry Space Type Probe v1

**Probe:** geometry_space_type_probe_v1  
**Contract:** prompt_contract_v4.md  
**Branch:** geometry_space_type_probe_v1  
**Layer tested:** Layer 1 — Ambient/operative space  
**N samples:** 1024  (256 per class)  

---

## 1. Space Representations Tested

### CURRENT_BASELINE (R^16)
- **Dimensions:** 16 (12 angular + 4 magnitude)
- **Structure:** BLOCKS_A = [(0,2,2,1),(2,7,5,1),(7,9,2,1),(9,21,12,3)]
- **Angular part:** 6 harmonic pairs; apply_anchor_two_i applied to all pairs
  - Block0 h1: dims [0,1];  Block1 h1: dims [2,3];  Block2 h1: dims [4,5]
  - Block3 h1: dims [6,7];  Block3 h2: dims [8,9];  Block3 h3: dims [10,11]
- **Magnitude part:** 4 dims (one per block), all fixed to 1.0
- **H3 comparison subspace:** angular dims [10, 11]
- **Basis:** Existing H3-tangent representation — unchanged

### R4_FLAT (R^4)
- **Dimensions:** 4 (no magnitude dims)
- **Structure:** h2 and h3 pairs of dominant block (block3, period=12)
- **Source:** angular dims [8, 9, 10, 11] from CURRENT_BASELINE
  - R4 dim 0 = baseline angular dim 8:  h2 of block3
  - R4 dim 1 = baseline angular dim 9:  h2 of block3
  - R4 dim 2 = baseline angular dim 10: h3 of block3
  - R4 dim 3 = baseline angular dim 11: h3 of block3
- **apply_anchor_two_i:** inherited (applied in full-state build; slice preserves it)
- **H3 comparison subspace:** dims [2, 3] within R4
- **Rationale:** Minimal representation preserving H3 signal with unit-norm angular pairs

### REDUCED_BLOCK (R^12)
- **Dimensions:** 12 (angular only; 4 magnitude dims removed from R^16)
- **Structure:** Same 6 harmonic pairs as CURRENT_BASELINE angular part
- **apply_anchor_two_i:** applied to all 6 pairs (identical to baseline)
- **H3 comparison subspace:** dims [10, 11] (identical to CURRENT_BASELINE)
- **Rationale:** Tests whether magnitude dimensions contribute to separation signal

---

## 2. Structural Equivalence Check

**Status:** PASS

Verified:
- Identical 1024 samples (same class labels) across all candidates
- Identical prototype construction: class tokens {0,1,2,3}, apply_anchor_two_i applied
- H3 values: baseline vs R4_FLAT max_diff = 0.0
- H3 values: baseline vs REDUCED_BLOCK max_diff = 0.0
- Identical operator: cos_metric on H3-equivalent subspace

---

## 3. Result Table

| Space Type | mean_matched_cos | mean_mismatched_cos | separation_delta |
|:-----------|----------------:|--------------------:|----------------:|
| CURRENT_BASELINE | 1.000000 | -0.333333 | 1.333333 |
| R4_FLAT | 1.000000 | -0.333333 | 1.333333 |
| REDUCED_BLOCK | 1.000000 | -0.333333 | 1.333333 |

---

## 4. Classification Result

**CLASSIFICATION: SPACE_INVARIANT_SIGNAL**

All three space candidates produce identical separation_delta values.
The geometric separation signal is determined entirely by the H3 subspace content,
which is preserved numerically unchanged across all three ambient space representations.

Changing the ambient space dimension (R^4 / R^12 / R^16) does not alter the
cos_metric separation between matched and mismatched class pairs.

The range of separation_delta across all candidates is < 0.05 (threshold).

---

## 5. Is Geometric Signal Dependent on Ambient Space?

**NO.**

The geometric separation signal (`separation_delta = mean_matched_cos − mean_mismatched_cos`)
is **invariant** across ambient space representations.

The signal originates from the H3 harmonic pair — the third harmonic of the
dominant cyclic block (block3, period=12). This pair encodes each of the 4 classes
as one of four unit-norm, 2i-rotated vectors:

  - class 0 → H3 = ( 0,  1)
  - class 1 → H3 = (−1,  0)
  - class 2 → H3 = ( 0, −1)
  - class 3 → H3 = ( 1,  0)

With eps_high=1.0, the H3 pair is frozen by algebraic construction across all D steps.
Consequently, including or excluding additional dimensions (h1 pairs of other blocks,
magnitude dims) does not affect the cos_metric comparison on the H3 subspace.

**Layer 1 interpretation:** The ambient space dimensionality is not the source of
the geometric separation signal. The signal is localised in the H3 harmonic pair
and is invariant to the ambient embedding (R^4, R^12, or R^16).

---

## 6. Locked Variables Confirmation

| Variable | Value | Changed? |
|:---------|:------|:---------|
| Basis | apply_anchor_two_i (2i frame) | NO |
| Operator | cos_metric | NO |
| Comparison subspace | H3 pair (per-space equivalent) | NO |
| eps_high | 1.0 | NO |
| Training | None | NO |
| New constants | None | NO |
| Radial structure | Unchanged | NO |

---

GEOMETRY SPACE PROBE RESULT: SPACE_INVARIANT_SIGNAL