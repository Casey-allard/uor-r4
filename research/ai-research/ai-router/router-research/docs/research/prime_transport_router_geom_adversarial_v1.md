# Prime Transport Router: Adversarial Robustness of Hard Geometric Inference v1

**Generated:** 2026-04-08T06:39:13Z  
**Config:** D_HIDDEN=32, B=256, CPU  
**Baseline:** hard_geom=1.0000, soft_mlp=1.0000 (D=24, clean)  

---

## Phase A — Angular Direction Noise

Angular perturbation δ ~ N(0, σ) radians per phase pair applied to cur_dir before ang_sim.  
Same perturbation applied to tau_prev direction for soft inference (fair comparison).

| noise_std (rad) | noise (°) | hard_acc | hard_std | Δ_hard | soft_acc | soft_std | Δ_soft | hard_fail? | soft_fail? |
|-----------------|-----------|----------|----------|--------|---------|---------|--------|-----------|----------|
| 0.000 | 0.0 | 1.0000 | 0.0000 | -0.0000 | 1.0000 | 0.0000 | -0.0000 | no | no |
| 0.050 | 2.9 | 1.0000 | 0.0000 | -0.0000 | 1.0000 | 0.0000 | -0.0000 | no | no |
| 0.100 | 5.7 | 1.0000 | 0.0000 | -0.0000 | 1.0000 | 0.0000 | -0.0000 | no | no |
| 0.200 | 11.5 | 1.0000 | 0.0000 | -0.0000 | 1.0000 | 0.0000 | -0.0000 | no | no |
| 0.500 | 28.6 | 1.0000 | 0.0000 | -0.0000 | 1.0000 | 0.0000 | -0.0000 | no | no |
| 1.000 | 57.3 | 1.0000 | 0.0000 | -0.0000 | 1.0000 | 0.0000 | -0.0000 | no | no |
| 1.570 | 90.0 | 1.0000 | 0.0000 | -0.0000 | 1.0000 | 0.0000 | -0.0000 | no | no |
| 3.140 | 179.9 | 1.0000 | 0.0000 | -0.0000 | 1.0000 | 0.0000 | -0.0000 | no | no |

**Hard geom failure:** NONE across all tested noise levels.  
**Soft failure:** NONE across all tested noise levels.  

---

## Phase B1 — D Extrapolation

D=64 trained model evaluated at D > 64. pos0_mask zero-padded to new length. No retraining.

| D_eval | OOD? | hard_acc | soft_acc | hard_fail? | soft_fail? |
|--------|------|----------|----------|-----------|----------|
| 64 | no | 1.0000 | 1.0000 | no | no |
| 80 | yes | 1.0000 | 1.0000 | no | no |
| 96 | yes | 1.0000 | 1.0000 | no | no |
| 128 | yes | 1.0000 | 1.0000 | no | no |

**No failure observed in D extrapolation range.**  

---

## Phase B2 — Adversarial Pool

Uniform random pool from all 343K states vs training-distribution pool.

| pool | hard_acc | soft_acc | hard_fail? |
|------|----------|----------|-----------|
| training_pool | 1.0000 | 1.0000 | no |
| adversarial_pool | 1.0000 | 1.0000 | no |

---

## Phase C — Tie / Ambiguity Stress

### C1: Margin Distribution

Natural top-1 vs top-2 angular similarity gap during clean hard geom inference:  
- mean = 0.7103  std = 0.5537  
- p10 = 0.118  p50 = 0.634  p90 = 1.5  
- steps with margin < 0.01: 1552/24576 (6.3%)  

### C2: Random Tie-Breaking

| tie_threshold | acc | std | Δ | frac_affected | fail? |
|---------------|-----|-----|---|---------------|-------|
| tie_thr=0.5 | 1.0000 | 0.0000 | +0.0000 | (see note) | no |
| tie_thr=0.2 | 1.0000 | 0.0000 | +0.0000 | (see note) | no |
| tie_thr=0.1 | 1.0000 | 0.0000 | +0.0000 | (see note) | no |
| tie_thr=0.05 | 1.0000 | 0.0000 | +0.0000 | (see note) | no |
| tie_thr=0.01 | 1.0000 | 0.0000 | +0.0000 | (see note) | no |

- tie_thr=0.5: random tie-break when margin<0.5; 43.9% steps affected  
- tie_thr=0.2: random tie-break when margin<0.2; 23.4% steps affected  
- tie_thr=0.1: random tie-break when margin<0.1; 9.5% steps affected  
- tie_thr=0.05: random tie-break when margin<0.05; 6.6% steps affected  
- tie_thr=0.01: random tie-break when margin<0.01; 6.3% steps affected  

### C3: Targeted Swap Attack

Noise calibrated to current margin magnitude at each step.

| swap_factor | acc | std | Δ | fail? |
|-------------|-----|-----|---|-------|
| swap_factor=0.25 | 1.0000 | 0.0000 | +0.0000 | no |
| swap_factor=0.5 | 1.0000 | 0.0000 | +0.0000 | no |
| swap_factor=1.0 | 1.0000 | 0.0000 | +0.0000 | no |
| swap_factor=2.0 | 1.0000 | 0.0000 | +0.0000 | no |

### C4: Clustered-State Subset

Eval restricted to states where TN candidates are most geometrically similar.

| subset | hard_acc | soft_acc | hard_fail? |
|--------|----------|----------|-----------|
| top50pct | 1.0000 | 1.0000 | no |
| top25pct | 1.0000 | 1.0000 | no |
| top10pct | 1.0000 | 1.0000 | no |

- top50pct: 50% most-clustered states (n=2004); mean_cluster_sim≥0.780  
- top25pct: 25% most-clustered states (n=1000); mean_cluster_sim≥1.119  
- top10pct: 10% most-clustered states (n=400); mean_cluster_sim≥1.437  

---

## Failure Summary

Hard geometric inference survived all adversarial conditions tested.  
No accuracy below 0.99 was observed in any phase.  

---

## HARD GEOMETRIC INFERENCE IS: ROBUST

(0/27 tested conditions resulted in failure [acc < 0.99])  
