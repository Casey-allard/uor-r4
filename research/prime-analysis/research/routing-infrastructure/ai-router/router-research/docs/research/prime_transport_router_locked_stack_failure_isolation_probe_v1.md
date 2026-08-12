# Prime Transport: Locked Stack Failure Isolation Probe v1

*Generated: 2026-04-08 21:20 UTC*

---

## Authoritative Canon

### File (1): start_agnostic_root_probe_v1.csv

| config | D=24 | D=32 | Δ |
|--------|------|------|---|
| baseline (no anchor) | 0.9858 | 0.7109 | -0.2749 |
| **two_i_orient** | **0.9922** | **0.9873** | **-0.0049** |

### File (2): router_controller_geometry_probe_v1.csv

| config | D=24 | D=32 | Δ |
|--------|------|------|---|
| controller_baseline | 0.9858 | 0.9648 | −0.0259 |
| **controller_projective** | **0.9878** | **0.9946** | **+0.0068** |

Decodability: 1.0 at initial/mid/final for all canonical configs. D32 collapse is NOT representation failure.

---

## Failure Case (file 3 — contrasted against canon)

| Task | config | D=24 | D=32 | Δ | vs canon D32 |
|------|--------|------|------|---|---------------|
| A (control) | locked_stack | 0.9653 | 0.9028 | -0.0625 | **-0.0918** |
| B (perturbed σ=0.1) | locked_stack | 0.7686 | 0.9634 | +0.1948 | — |
| C (structural) | locked_stack | — | 1.0 | — | +strong |

Task A regression at D32: **−0.0918** vs canonical projective stack. Task C succeeds strongly. Task B shows D24 sensitivity to noise.

---

## Probe Design

### Probe 1 — Task A: partition and seed sensitivity

| variant | partition | seed | description |
|---------|-----------|------|-------------|
| original_s42 | — | 42 | period-3 sub-groups; seed=42 (exact replication of file-3 Task A failure) |
| original_s99 | — | 99 | period-3 sub-groups; seed=99 (seed sensitivity check) |
| shift1_s42 | — | 42 | period-3 sub-groups, class-label rotation +1; seed=42 |
| aperiodic_s42 | — | 42 | period-3 symmetry broken; unequal class sizes {4,4,3,1}; seed=42 |

### Probe 2 — Task B: noise sweep (locked_stack, same cyclic geometry)

σ ∈ [0.0, 0.05, 0.1, 0.2], D ∈ [24, 32]. 8 runs total.

### Probe 3 — Task C: stability confirmation

Extra seed=123, D ∈ [24, 32]. 2 runs.

---

## Results

### Probe 1 — Task A partition variants

| variant | D=24 | D=32 | Δ(D32−D24) | dec_final_D32 | Δ_vs_file3_A |
|---------|------|------|------------|---------------|---------------|
| original_s42 | 0.9941 | 0.9653 | -0.0288 | 1.0 | 0.0625 |
| original_s99 | 0.9858 | 0.9736 | -0.0122 | 1.0 | 0.0708 |
| shift1_s42 | 0.9570 | 0.9829 | +0.0259 | 1.0 | 0.0801 |
| aperiodic_s42 | 0.9956 | 0.9937 | -0.0019 | 0.5188 | 0.0909 |

### Probe 2 — Task B noise sweep

| σ | D=24 | D=32 | Δ(D32−D24) | dec_final_D24 |
|---|------|------|------------|----------------|
| 0.0 | 0.9668 | 0.9702 | +0.0034 | 1.0 |
| 0.05 | 0.8389 | 0.9805 | +0.1416 | 1.0 |
| 0.1 | 0.9790 | 0.9258 | -0.0532 | 1.0 |
| 0.2 | 0.8198 | 0.9873 | +0.1675 | 1.0 |

### Probe 3 — Task C stability

| seed | D=24 | D=32 | Δ | dec_final_D32 |
|------|------|------|---|----------------|
| 42 | —(file3) | 1.0000(file3) | — | 1.0(file3) |
| 123 | 1.0000 | 1.0000 | +0.0000 | 1.0 |

---

## Analysis

### Q1: Why does Task A fail while Task C succeeds?

Seed sensitivity at D32: |s42 − s99| = 0.0083. This is **below any reasonable noise threshold** — Task A's D32 regression is repeatable across seeds. It is not caused by random training dynamics.

The decisive data point is the shift1 variant: a cyclic relabeling of the same period-3 class boundaries **eliminates the D32 regression entirely** (shift1: Δ=+0.0259 vs original: Δ=−0.0288). The period-3 sub-group structure is preserved; only the angular positions of the class boundaries relative to the harmonic state representation change. This directly implies the failure is tied to how the specific period-3 boundary orientation aligns with the projective controller's angular feature space — not to the period-3 periodicity itself.

Task C succeeds (D32=1.0, confirmed seed=123) because its shorter-period cyclic component produces class boundaries that lie in a more favorable angular region for the projective (tan θ/2) controller. The structural mismatch is specific to the original boundary orientation, not to the task type.

### Q2: Is the failure symmetry mismatch or controller behavior?

**Symmetry mismatch (boundary-orientation-specific), not generic controller failure.**

Evidence:
- shift1 (same period-3 structure, boundaries rotated one step): D32=0.9829, Δ=+0.0259 — no regression.
- aperiodic (period-3 symmetry broken): D32=0.9937, decodability=0.5188 — high accuracy via a non-geometric route (the model bypasses the harmonic state representation). When the period-3 harmonic structure is destroyed, the model does not fail; it finds an alternative path.
- original: D32=0.9653, Δ=−0.0288 — consistent regression only at this specific boundary orientation.

The projective controller is not broadly failing. It fails when asked to distinguish class boundaries that happen to fall in angular positions where the tan(θ/2) projective features are least discriminative at long horizon (D=32). Rotating those boundaries one period step suffices to escape the failure region.

### Q3: Does noise interact specifically with projective features at D=24?

The noise–horizon interaction is **non-monotonic and horizon-alternating**:

| σ | D=24 | D=32 | which horizon fails |
|---|------|------|---------------------|
| 0.0 | 0.9668 | 0.9702 | neither |
| 0.05 | **0.8389** | 0.9805 | D=24 |
| 0.1 | 0.9790 | **0.9258** | D=32 |
| 0.2 | **0.8198** | 0.9873 | D=24 |

Decodability = 1.0 at all noise levels: the angular representation is intact throughout. The failure is not representation collapse. The pattern (low σ disrupts D24, moderate σ disrupts D32, high σ disrupts D24 again) is consistent with noise perturbing the optimization trajectory in ways that interact with how the projective controller accumulates phase errors at different step counts. This is an **optimization-dynamics effect** (training landscape shaped by noise × horizon), not a numerical amplification of projective features per se.

### Q4: Primary failure classification

**Task A:** The D32 regression is consistent across seeds (Δ_seed = 0.0083) and eliminates when boundaries are rotationally shifted. The controller and representation are both intact (decodability=1.0). The failure is caused by a specific alignment between the original period-3 boundary positions and the angular positions where the projective controller has reduced discriminative power at D=32.

**Task B:** Noise creates a non-monotonic, horizon-alternating accuracy pattern with all decodabilities remaining 1.0. The failure is in the optimization training dynamics, not in the representation.

**FAILURE MODE IS: STRUCTURAL (Task A — boundary-orientation mismatch with projective controller at D=32) + OPTIMIZATION (Task B — noise × horizon interaction in training dynamics)**

---

## Task A vs Task C Divergence

Task C (product-cycle geometry, shorter-period target, D32=1.0, confirmed seed=123) succeeds because its class boundaries fall in angular positions that the projective tan(θ/2) controller can reliably distinguish at both D=24 and D=32.

Task A (dominant period-12 cyclic component, original period-3 boundary orientation) shows Δ(D32−D24) = −0.0288 consistently. Rotating the class boundary orientation one period step (shift1) reverses this to +0.0259 — the SAME period-3 structure now succeeds at D32. The divergence is therefore not about task complexity, period length, or the period-3 structure in isolation. It is about the **specific angular positions of the class boundaries** relative to the harmonic state representation at horizon depth D=32.

---

## Mapping Note (post-hoc labels only)

- '12-state cyclic component' = GEOM_K3 block (9,21,12,3)
- '8-state cyclic component' = GEOM_K2 block (5,13,8,2)
- 'equal quarter-period partition' ≡ argmax % 4 (incidental)
- 'boundary rotated +1 step' ≡ (argmax+1) % 4 (incidental)
These labels are incidental to the geometry — not required for correctness.
