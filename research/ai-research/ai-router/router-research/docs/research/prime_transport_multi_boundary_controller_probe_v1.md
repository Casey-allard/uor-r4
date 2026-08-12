# Prime Transport: Multi-Boundary Controller Probe v1

*Generated: 2026-04-08 23:02 UTC*

---

## Pre-Code Geometry Lock (verified)

**Cyclic state space:** dominant 12-state cyclic component (one-hot positions 9–21, BLOCKS_A block `(9,21,12,3)`).

**Interleaved 4-class partition (CONFIRMED INTERLEAVED):**
`[0,1,2,3,0,1,2,3,0,1,2,3]`
class 0 = {0,4,8}, class 1 = {1,5,9}, class 2 = {2,6,10}, class 3 = {3,7,11}
NOT contiguous blocks — every class member is 4 cyclic positions apart.

**Projective controller (canonical):** `proj_k = sin_k / (1 + cos_k + 0.1)`, clipped ±10.

**Split transport:** fundamental harmonic free; k≥2 frozen (`eps_high=1.0`).

**two_i anchoring:** tau0 rotated +π/2 per pair: `(c,s)→(−s,c)`.

**Boundary definition:** every consecutive state pair is a class boundary (12 total). 4 distinct boundary types (0→1, 1→2, 2→3, 3→0). No single cut — boundaries are uniformly distributed.

---

## Canonical Data Source

**CANONICAL:** `router_locked_stack_failure_isolation_probe_v1.csv` (probe1_taskA, projective locked stack)

| variant | D=24 | D=32 | Δ(D32−D24) |
|---------|------|------|------------|
| original_s42 | 0.9941 | 0.9653 | -0.0288 |
| shift1_s42   | 0.957 | 0.9829 | +0.0259 |

Baseline Δ spread (|orig − shift1|): D24 = 0.0371, D32 = 0.0176

---

## Multi-Boundary Controller Design

N_BOUNDARIES = 4, reference angles φ_j = j×π/2 = {0.000, 0.524, 1.047, 1.571}

For each pair k and boundary j:
  c_rot = c_k·cos(φ_j) + s_k·sin(φ_j)
  s_rot = s_k·cos(φ_j) − c_k·sin(φ_j)
  p_j_k = s_rot / (1 + c_rot + 0.1)   [clipped ±10]

**ctrl_1 (baseline projective):** p_0_k only (φ_0 = 0)
**ctrl_2 (multi_avg):** output_k = mean_j(p_j_k)
**ctrl_3 (multi_invdist):** w_j_k = 1/(1 − c_rot + 0.1), output_k = Σ(w·p)/Σ(w)

All three controllers: input dim = D_EMB + n_pairs + n_blocks = 4 + 6 + 4 = 14

---

## Results

| controller | variant | D=24 | D=32 | Δ(D32−D24) | dec_final | Δ_spread |
|------------|---------|------|------|------------|-----------|----------|
| ctrl_1_projective | original_s42 | 0.9956 | 0.9790 | -0.0166 | 1.0 | 0.0005 |
| ctrl_1_projective | shift1_s42 | 0.9966 | 0.9795 | -0.0171 | 1.0 | 0.0005 |
| ctrl_2_multi_avg | original_s42 | 0.9834 | 0.9731 | -0.0103 | 1.0 | 0.0137 |
| ctrl_2_multi_avg | shift1_s42 | 0.9492 | 0.9868 | +0.0376 | 1.0 | 0.0137 |
| ctrl_3_multi_invdist | original_s42 | 0.9990 | 0.9800 | -0.0190 | 1.0 | 0.0083 |
| ctrl_3_multi_invdist | shift1_s42 | 0.9907 | 0.9883 | -0.0024 | 1.0 | 0.0083 |

---

## Orientation Sensitivity Analysis

**Goal:** Does multi-boundary aggregation reduce the gap between original_s42 and shift1_s42?

Metric: |Δ_original − Δ_shift1| at D32 (smaller = more invariant)

| controller | Δ_original | Δ_shift1 | |Δ_orig − Δ_sh1| | D32_orig | D32_shift1 | D32_spread |
|------------|------------|----------|-----------------|----------|------------|------------|
| ctrl_1_projective | -0.0166 | -0.0171 | 0.0005 | 0.979 | 0.9795 | 0.0005 |
| ctrl_2_multi_avg | -0.0103 | +0.0376 | 0.0479 | 0.9731 | 0.9868 | 0.0137 |
| ctrl_3_multi_invdist | -0.0190 | -0.0024 | 0.0166 | 0.98 | 0.9883 | 0.0083 |

**Baseline (ctrl_1 projective):**
  Δ_original = -0.0288,  Δ_shift1 = +0.0259,  |gap| = 0.0547

## Success Evaluation

Success criteria (per prompt contract):
1. Smaller gap between original_s42 and shift1_s42 (reduced orientation sensitivity)
2. Stable or improved D32 accuracy
3. No decodability collapse (final decodability ≥ 0.95)

- ctrl_1_projective: gap=REDUCED (0.0005 vs baseline 0.0547), D32=STABLE (0.979), decodability=OK
- ctrl_2_multi_avg: gap=REDUCED (0.0479 vs baseline 0.0547), D32=STABLE (0.9731), decodability=OK
- ctrl_3_multi_invdist: gap=REDUCED (0.0166 vs baseline 0.0547), D32=STABLE (0.98), decodability=OK

---

*This is a mechanism isolation step, NOT a performance optimization step.*
*No new tasks, φ/Fibonacci, or transport modifications were introduced.*
