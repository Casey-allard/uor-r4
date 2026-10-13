# Prime Transport — Geometry Dependency Order Probe v1

**Branch:** geometry_dependency_order_probe_v1  
**Contract:** prompt_contract_v4.md — loaded and binding  
**Purpose:** Dependency ordering audit — NOT a mechanism proof  

---

## Overview

The current system is coupled and hierarchical. Prior probes incorrectly treated
downstream variables (D, radial quantities, training size) as capable of falsifying
upstream geometry choices.

This document enforces the correct dependency order and determines which layers are
still unresolved BEFORE any further D or ratio conclusions are allowed.

**Core rule:** A downstream variable may NOT be used to rule on an upstream variable.

---

## 1. Dependency-Order Table

| Layer | Name | Repo Components | Status |
|---|---|---|---|
| 1 | Ambient/operative space | BLOCKS_A config; R^16 hybrid (12 angular + 4 mag dims); H3 = h=3 harmonic within block | **UNRESOLVED** |
| 2 | Basis family | `apply_anchor_two_i`; unit-norm (cos,sin) pairs in TN_ang; sqrt(2)/2i amplitude | **PARTIALLY SPECIFIED** |
| 3 | Projection/operator family | `cos_metric`, `sin_metric`; tan excluded; joint/cos_only/sin_only in training probes | **PARTIALLY SPECIFIED** |
| 4 | Constant/ratio family | `full_radial=sqrt(10)`, `subspace_radial=1.0`, `radial_ratio=1/sqrt(10)`; carrier_scale sweep | **SETTLED** (as constants) |
| 5 | Comparison-support mechanism | H3 pair (dims 10-11); 2i-rotated prototypes; H3 tangent migration | **PARTIALLY SPECIFIED** |
| 6 | D / workspace | D swept [0,1,5,12,16,20,24,32]; eps=1.0 freezes H3 algebraically | **UNRESOLVED** |
| 7 | Training/reference structure | MINIMAL(0), STRUCTURED(500), EXPANDED(1500) steps; 758-param MLP | **UNRESOLVED** |

**Note on Layer 1:** "H3 tangent" is a label for the 3rd harmonic of a specific block within
a flat R^16 space. It is NOT a reference to hyperbolic H³ space. The block structure
(BLOCKS_A) and the choice of R^16 as the ambient space were established by convention.
R4 and other dimensionalities have never been tested.

---

## 2. Prior Conclusion Audit Table

| Conclusion | Source Probe | Layers Actually Tested | Unresolved Upstream | Classification |
|---|---|---|---|---|
| training_only_effect | h3_coupled_geometry_training_dependency_probe_v1 | Layer 6 (D), Layer 7 (training steps) | Layer 2 (MLP basis unverified), Layer 3 (joint fixed) | **B: CONDITIONAL ONLY** |
| d_irrelevance | radial_ratio_revalidation, train_free, h3_coupled | Layer 6 under eps=1.0 | Layer 1 (space type), Layers 2-5 | **B: CONDITIONAL ONLY** |
| radial_dead | h3_2i_radial_increment_probe_v1, radial_ratio_revalidation_probe_v1 | Layer 4 (radial constants) | None | **A: SAFE TO KEEP** |
| ratio_dead | d_aware_h3_synthesis_ratio_lock_probe_v1 | Layer 4 (radial ratio) | None | **A: SAFE TO KEEP** |
| cos_vs_sin_operator | h3_basis_separation_probe_v1, h3_tangent_projection_frame_sweep_v1 | Layer 3 (operator), Layer 2 (frame scenario) | Layer 1 (ambient space), Layer 2 (canonical frame) | **B: CONDITIONAL ONLY** |
| 2i_frame_alignment | h3_tangent_full_mechanism_migration_probe_v1 | Layer 2 (basis), Layer 5 (comparison) | Layer 1 (2i rotation defined within R^16) | **B: CONDITIONAL ONLY** |

**Classification key:**
- A: SAFE TO KEEP — conclusion is independent of upstream choices
- B: CONDITIONAL ONLY — conclusion holds locally but cannot be promoted globally
- C: INVALID AS GLOBAL CONCLUSIONS — none identified yet

---

## 3. Lightweight Validation Section

All checks below are LOCAL VALIDATION ONLY. No sweeps, no training, no optimization.

### LV-1: Layer 1 — Ambient Space Configuration Check

**Purpose:** Confirm the current ambient space is R^16 by convention.

| Item | Value |
|---|---|
| BLOCKS_A | (0,2,2,1), (2,7,5,1), (7,9,2,1), (9,21,12,3) |
| Angular dims (tau_ang) | 12 (6 unit-norm (cos,sin) pairs) |
| Magnitude dims (tau_mag) | 4 (one per block) |
| d_hyb | 16 |
| H3 pair location | dims 10-11 (3rd harmonic of block 3) |
| R4 alternative tested? | NO |
| Derivation from geometry principle? | NONE — convention only |

**Result:** Layer 1 status = UNRESOLVED. Choice of R^16 is a convention not derived
from any foundational geometric argument.

---

### LV-2: Layer 2 — 2i Frame Mismatch Confirmation

**Purpose:** Verify that cos(2i_state_k, raw_proto_k) = 0 for all k analytically.

`apply_anchor_two_i`: (cos θ, sin θ) → (−sin θ, cos θ) (90° rotation in each pair)

| k | 2i state | raw proto | cos(2i, raw) | cos(2i, 2i) |
|---|---|---|---|---|
| 0 | [0, +1] | [+1, 0] | 0.0 | 1.0 |
| 1 | [+1, 0] | [0, +1] | 0.0 | 1.0 |
| 2 | [0, −1] | [−1, 0] | 0.0 | 1.0 |
| 3 | [−1, 0] | [0, −1] | 0.0 | 1.0 |

**Result:** cos(2i_state, raw_proto) = 0 for ALL classes — systematic false null confirmed.
cos(2i_state, 2i_proto) = 1.0 for ALL classes — signal fully recovered in same frame.  
**This result is CONDITIONAL on Layer 1** (2i rotation defined within R^16).

---

### LV-3: Layer 3 — Tan Exclusion Confirmation

**Purpose:** Verify tan is structurally excluded by NaN at adjacent classes.

In the 2i-rotated 4-class frame, adjacent class pairs have cos(angle) = 0.
tan = sin/cos = NaN at these points. 4 of 4 adjacent pairs trigger NaN.

**Result:** tan is excluded by construction — 100% of adjacent class comparisons give NaN.

---

### LV-4: Layer 4 — Radial Constants Confirmation

**Purpose:** Confirm radial quantities are structural constants (analytic derivation).

| Quantity | Formula | Analytic Value | Discriminative? |
|---|---|---|---|
| full_radial | sqrt(6×1 + 4×1) | sqrt(10) ≈ 3.162278 | NO — constant |
| subspace_radial | sqrt(1) (H3 pair norm) | 1.0 | NO — constant |
| radial_ratio | subspace / full | 1/sqrt(10) ≈ 0.316228 | NO — constant |

**Result:** All three quantities are constants by construction — independent of D, eps,
carrier_scale, or any other variable. Layer 4 status = SETTLED.

---

### LV-5: Layer 6 — D Irrelevance Under eps=1.0

**Purpose:** Prove D is structurally irrelevant under eps_high=1.0 (not an empirical finding).

`apply_split_transport(eps_high=1.0)` for harmonic h ≥ 2:
```
  new_h = (1 - eps_high) × proposed + eps_high × previous
         = 0 × proposed + 1 × previous
         = previous   [EXACT PRESERVATION — algebraic identity]
```

H3 is h=3 ≥ 2, so: `tau_final[:,10:12] == tau_init[:,10:12]` for any D when eps_high=1.0.

**Result:** D irrelevance under eps=1.0 is a TAUTOLOGY, not an empirical result.
Prior "D irrelevant" claims under eps=1.0 are structural consequences of the architecture
and carry no information about D's role in synthesis (eps<1.0) dynamics.

---

## 4. Which conclusions are only conditional because upstream geometry remains unresolved?

**training_only_effect:**
The conclusion that training dominates over geometry was reached in
`h3_coupled_geometry_training_dependency_probe_v1`. This probe:
- Fixed the operator to `joint` (Layer 3 choice) without verifying Layer 2 basis alignment
  for the MLP training module
- Used D as the geometry proxy (Layer 6), which is irrelevant when eps=1.0 (LV-5 above)
- Did not verify whether the training module's internal representation uses the correct
  2i-rotated frame or raw frame

**Conditional on:** Layer 2 (basis alignment for MLP) and Layer 3 (operator choice).
**Cannot be promoted to:** "training always dominates over geometry" as a global conclusion.

---

**d_irrelevance:**
D is irrelevant under eps=1.0 by construction (LV-5). Prior probes measured D effects
under eps=1.0, where D's irrelevance is guaranteed algebraically. This tells us nothing
about D's role in the synthesis (eps<1.0) regime, which is the only regime where D
actually enters the computation. Under synthesis, D results are further entangled with
unresolved Layers 1-5.

**Conditional on:** eps_high=1.0 (tautology). Under synthesis: additionally conditional
on Layers 1, 2, 3, 5.

---

**cos_vs_sin_operator:**
The operator conclusion from `h3_basis_separation_probe_v1` showed that:
- In mismatch scenario (2i state, raw proto): sin wins
- In correct scenario (both 2i): cos wins

The "correct scenario" adopts the 2i-rotated prototype convention — but this convention
was chosen AFTER observing the false null in prior probes. The choice of which scenario
is the "true" operating condition of the system requires Layer 2 (basis family) to be
settled globally, which requires Layer 1 (ambient space) to be settled first.

**Conditional on:** Layer 1 (space type), Layer 2 (canonical frame).

---

**2i_frame_alignment:**
The 2i prototype alignment conclusion from the migration probe is valid within R^16.
But the 90° rotation `apply_anchor_two_i` is defined as a rotation in 2D subspaces
of R^16. If the ambient space is revised to R4 or another geometry, the rotation
operator and its effect on class separation must be rederived.

**Conditional on:** Layer 1 (ambient space must remain R^16).

---

## 5. What is the earliest unresolved layer that must be tested next?

**LAYER 1 — Ambient/operative space**

**Why Layer 1 blocks everything downstream:**
- Layer 2 (basis): the 2i rotation is defined WITHIN R^16; space change → rotation changes
- Layer 3 (operator): cos/sin are defined relative to the pair structure in R^16
- Layer 4 (constants): sqrt(10) and 1/sqrt(10) arise from the 16-dimensional structure
- Layer 5 (comparison): H3 pair subspace (dims 10-11) is defined within R^16
- Layer 6 (D): D operates on the R^16 state; space change → D semantics change
- Layer 7 (training): MLP inputs are the R^16 state vector

**What the next probe must test:**
`geometry_space_type_probe_v1` should:
1. Keep Layers 2-7 constant (hold all other variables fixed)
2. Compare class-separation signal (cos metric between matched vs mismatched)
   across at minimum: current R^16 hybrid vs R4 (4-dim flat)
3. Use only deterministic analytic structural comparison — no training
4. Confirm whether the geometric signal is space-specific or space-invariant
5. Must NOT change basis, operator, or comparison mechanism simultaneously

**Probes that remain invalid until Layer 1 is constrained:**
- D-sensitivity conclusions under synthesis (eps<1.0) — Layer 6
- Training-size conclusions as global claims — Layer 7
- Operator-family claims as global conclusions — Layer 3
- Any ratio/constant as a mechanism driver — beyond already-settled constants (Layer 4)

---

## 6. Final Status

```
GEOMETRY DEPENDENCY ORDER STATUS: LAYER_1 BLOCKING
```

| Category | Layers |
|---|---|
| Settled globally | Layer 4 (radial constants — structural facts of architecture) |
| Partially specified (conditional on Layer 1) | Layers 2, 3, 5 |
| Unresolved (blocking) | Layer 1 (earliest), Layer 6, Layer 7 |

**Safe conclusions (no downstream conditions):**
- Radial quantities (`full_radial`, `subspace_radial`, `radial_ratio`) are structural constants
- Radial quantities carry zero discriminative information
- Tan is excluded by NaN instability at adjacent classes

**Downgraded to CONDITIONAL ONLY:**
- `training_only_effect` — conditional on Layers 2, 3
- `d_irrelevance` — tautology under eps=1.0; uninformative for synthesis
- `cos_vs_sin_operator` — conditional on Layers 1, 2
- `2i_frame_alignment` — conditional on Layer 1

**No conclusions are currently classified as C (INVALID AS GLOBAL)**  
because no conclusion has been shown to be structurally impossible —
only conditional on upstream choices that remain open.
