# Prime Transport: φ-R4-Dimension Alignment Probe v1

*Generated: 2026-04-08 23:56 UTC*

---

## Canonical Source Declaration

**CANONICAL** (source of truth for all reasoning):
- `prime_transport_router_start_agnostic_root_probe_v1.csv`
  - two_i_rot: D24=0.9922, D32=0.9873, Δ=−0.0049
  - Decodability=1.0 throughout all configs
- `prime_transport_router_controller_geometry_probe_v1.csv`
  - controller_projective: D24=0.9878, D32=0.9946, Δ=+0.0068
  - Best stack: two_i_rot anchor + projective controller + eps_high=1.0

**CONTRAST**: none (no other files used as truth)

## Pre-Code Geometry Lock (verified)

- 12-state cyclic dominant component: block [9:21] in tau0_oh (n_h=3)
- Partition: (argmax[9:21]) % 4 → interleaved [0,1,2,3,...]
- original_s42: labels = argmax % 4, seed=42
- shift1_s42:   labels = (argmax + 1) % 4, seed=42
- Partitions NOT redefined. Block structure NOT changed.
- GEOM_K3: [(0,2,2,1),(2,7,5,1),(7,9,2,1),(9,21,12,3)]

## Experiment Design

### Fixed components (canonical best stack)
- Anchor: two_i_rot (rotate each (c,s) pair by +π/2)
- Transport: split, eps_high=1.0 (k≥2 frozen)
- D_HIDDEN=32, LR=0.02, BATCH=512, seed=42, MAX_STEPS=3500

### Controller variants

**1. ctrl_baseline** (`ctrl_mode=baseline`)
  - Standard projective: proj_k = sin_k / (1 + cos_k + 0.1)
  - Single reference frame — identical to controller_projective in prior probe
  - d_ctrl: D_EMB(4) + n_pairs(6) + n_blocks(4) = 14

**2. ctrl_phi** (`ctrl_mode=phi`)
  - φ = (1+√5)/2 ≈ 1.618034
  - φ-offsets for n_pairs=6: [0.0, 3.8832, 1.4833, 5.3665, 2.9665, 0.5666]
  - (NOT evenly spaced; successive differences are irrational)
  - Apply rotation by φ_k to each (c,s) pair, then compute projective
  - d_ctrl: D_EMB(4) + n_pairs(6) + n_blocks(4) = 14

**3. ctrl_phi_r4** (`ctrl_mode=phi_r4`)
  - Same φ-offsets + double-angle R^4 coupling:
    c2 = c'^2 - s'^2  (cos 2θ'),  s2 = 2c's'  (sin 2θ')
    r4_k = s2 / (1 + c2 + 0.1)
  - No learned parameters added — purely Fourier double-angle
  - d_ctrl: D_EMB(4) + n_pairs(6) + n_pairs(6) + n_blocks(4) = 20

### Task orientation variants
- original_s42: standard interleaved 4-class, seed=42
- shift1_s42: same partition shifted by +1 label (cyclic), seed=42

### Dimension sweep: D ∈ [24, 32]  (INCLUDE_OPTIONAL=False)

## Full Results Table

| Config | Ctrl | Orient | D | Accuracy | Solve | No-Last | Dec_i | Dec_m | Dec_f | α_last | RT(s) |
|--------|------|--------|---|----------|-------|---------|-------|-------|-------|--------|-------|
| ctrl_baseline_original_s42 | baseline | original_s42 | 24 | **0.9941** | — | 0.9932 | 1.0 | 1.0 | 1.0 | 0.0408 | 111.4 |
| ctrl_baseline_shift1_s42 | baseline | shift1_s42 | 24 | **0.9976** | — | 0.9971 | 1.0 | 1.0 | 1.0 | 0.0322 | 110.5 |
| ctrl_phi_original_s42 | phi | original_s42 | 24 | **0.9893** | — | 0.9922 | 1.0 | 1.0 | 1.0 | 0.0353 | 124.5 |
| ctrl_phi_shift1_s42 | phi | shift1_s42 | 24 | **0.8638** | — | 0.8403 | 1.0 | 1.0 | 1.0 | 0.0388 | 124.5 |
| ctrl_phi_r4_original_s42 | phi_r4 | original_s42 | 24 | **0.7285** | — | 0.709 | 1.0 | 1.0 | 1.0 | 0.0412 | 175.2 |
| ctrl_phi_r4_shift1_s42 | phi_r4 | shift1_s42 | 24 | **0.9683** | — | 0.9692 | 1.0 | 1.0 | 1.0 | 0.0508 | 175.8 |
| ctrl_baseline_original_s42 | baseline | original_s42 | 32 | **0.9707** | — | 0.9717 | 1.0 | 1.0 | 1.0 | 0.0272 | 147.5 |
| ctrl_baseline_shift1_s42 | baseline | shift1_s42 | 32 | **0.9805** | — | 0.9868 | 1.0 | 1.0 | 1.0 | 0.0303 | 148.8 |
| ctrl_phi_original_s42 | phi | original_s42 | 32 | **0.9912** | — | 0.9927 | 1.0 | 1.0 | 1.0 | 0.0259 | 170.3 |
| ctrl_phi_shift1_s42 | phi | shift1_s42 | 32 | **0.9937** | — | 0.9927 | 1.0 | 1.0 | 1.0 | 0.0242 | 169.0 |
| ctrl_phi_r4_original_s42 | phi_r4 | original_s42 | 32 | **0.9946** | — | 0.9961 | 1.0 | 1.0 | 1.0 | 0.0272 | 240.2 |
| ctrl_phi_r4_shift1_s42 | phi_r4 | shift1_s42 | 32 | **0.981** | — | 0.9854 | 1.0 | 1.0 | 1.0 | 0.0262 | 240.0 |

## Δ(D32 − D24) Analysis

| Ctrl | Orient | D24 | D32 | Δ(D32−D24) |
|------|--------|-----|-----|------------|
| [REF] projective (ctrl_geometry) | original | 0.9878 | 0.9946 | +0.0068 |
| baseline | original_s42 | 0.9941 | 0.9707 | -0.0234 |
| baseline | shift1_s42 | 0.9976 | 0.9805 | -0.0171 |
| phi | original_s42 | 0.9893 | 0.9912 | +0.0019 |
| phi | shift1_s42 | 0.8638 | 0.9937 | +0.1299 |
| phi_r4 | original_s42 | 0.7285 | 0.9946 | +0.2661 |
| phi_r4 | shift1_s42 | 0.9683 | 0.9810 | +0.0127 |

## Orientation Gap Analysis

orientation_gap = |Δ_original − Δ_shift1|
A smaller gap means the controller is more robust to orientation variant.

| Ctrl | Δ_original | Δ_shift1 | Orientation Gap |
|------|------------|----------|-----------------|
| baseline | -0.0234 | -0.0171 | 0.0063 |
| phi | +0.0019 | +0.1299 | 0.1280  (Δvs_base: +0.1217) |
| phi_r4 | +0.2661 | +0.0127 | 0.2534  (Δvs_base: +0.2471) |

## Hypothesis Verdicts

### H0 (NULL): Current best stack is sufficient

**SUPPORTED**: Neither φ nor φ+R^4 reduces the orientation gap vs baseline. baseline_gap=0.0063, phi_gap=0.1280 (+0.1217), phi_r4_gap=0.2534 (+0.2471). Orientation robustness is NOT improved by either modification. Null hypothesis stands.

### H1 (PHI ANGLE): φ-based controller improves orientation robustness

Verdict: **NOT SUPPORTED: φ gap=0.1280 ≥ baseline gap=0.0063 (orientation robustness not improved)**

### H2 (PHI + R4): φ improvement only effective with R^4 coupling

Verdict: **NOT SUPPORTED: R^4 does not reduce orientation gap (gap 0.1280→0.2534, WORSENED). Failure condition: φ+R^4 orientation gap ≥ φ alone.**

### H3 (DIMENSION ALIGNMENT): Non-monotonic performance vs D

Verdict: **NOT TESTED (INCLUDE_OPTIONAL=False; only D={24,32} run)**

## Decodability Collapse Check

No decodability collapse. All final decodability values ≥ 0.9.

## Failure Condition Audit

Per task spec, hypothesis is NOT supported if ANY of:

- 1. **FAIL (φ beats D32 but collapses at D24 — not robust)**
- 2. **FAIL (R^4 does not improve φ in both orientations)**
- 3. **NOT TESTED (INCLUDE_OPTIONAL=False; only D={24,32} run)**
- 4. **FAIL (single-orientation only or D24 collapse)**

## Limitations

- Single seed (42) — all results are point estimates
- Only GEOM_K3 tested
- φ-offsets use golden-angle Fibonacci distribution; other irrational angle sequences not tested
- R^4 coupling uses double-angle only; other geometry-native expansions not tested
- Orientation variants use cyclic label shift only; other orientation perturbations not tested
- H3 not tested: only D ∈ {24, 32} run (INCLUDE_OPTIONAL=False)

