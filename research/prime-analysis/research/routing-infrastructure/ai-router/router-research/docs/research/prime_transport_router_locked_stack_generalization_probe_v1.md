# Prime Transport: Locked Stack Generalization Probe v1

*Generated: 2026-04-08 19:29 UTC*

---

## Mandatory First Reads (grounded in CSV, not markdown)

### start_agnostic_root_probe_v1.csv

| Config | D=24 | D=32 | Δ | Interpretation |
|--------|------|------|---|----------------|
| baseline (none) | 0.9858 | 0.7109 | −0.2749 | **D32 collapse confirmed** |
| sqrt2_radial | 0.9829 | 0.9785 | −0.0044 | stabilizes |
| **two_i_orient** | **0.9922** | **0.9873** | **−0.0049** | **best single anchor** |
| combined | 0.9844 | 0.9536 | −0.0308 | weaker than singles |

Decodability: **1.0** across initial/mid/final for all configs ✓

### router_controller_geometry_probe_v1.csv

| Config | D=24 | D=32 | Δ | Interpretation |
|--------|------|------|---|----------------|
| controller_baseline (sin/cos) | 0.9907 | 0.9648 | −0.0259 | horizon regression |
| **controller_projective** | **0.9878** | **0.9946** | **+0.0068** | **projective improves D32** |
| controller_hybrid | 0.9912 | 0.9771 | −0.0141 | hybrid intermediate |

Decodability: **1.0** across initial/mid/final for all configs ✓

**Confirmed baseline state:**
- D32 collapse without anchoring: 0.7109 ✓
- two_i_orient stabilizes: 0.9873 ✓
- Projective controller further improves D32 to 0.9946 ✓
- Decodability = 1.0 throughout ✓

---

## Experiment Design

### Locked stack components (DO NOT MODIFY)
| Component | Setting |
|-----------|--------|
| Transport | Split, eps_high=1.0 (k≥2 frozen) |
| State | Harmonic sin/cos encoding |
| Start anchor | two_i_orient: rotate (c,s)→(−s,c) per pair |
| Controller | Projective: proj_k=sin_k/(1+cos_k+0.1), clipped |

### Task variants
| Task | Label | State space | Critical harmonic | Geometry | Perturbation |
|------|-------|-------------|-------------------|----------|--------------|
| A | mod12_quarter | Z_{*}×Z_{12} (cache) | k=3 | GEOM_K3 | None (control) |
| B | mod12_quarter_noisy | Same as A | k=3 | GEOM_K3 | Embedding noise σ=0.1 |
| C | mod8_quarter | Z_5×Z_8 (40 states) | k=2 | GEOM_K2 | None (structural change) |

### Controller comparison (one only)
| Config | Controller mode | Anchor | Transport |
|--------|----------------|--------|----------|
| locked_stack | Projective (tangent) | two_i_orient | Split eps=1.0 |
| sincos_baseline | Baseline (sin/cos) | two_i_orient | Split eps=1.0 |

**Note:** Both configs share the same anchor and transport. Only the controller input geometry differs.

**Total runs:** 2 configs × 3 tasks × 2 horizons = 12 runs

---

## Results

### Full Results Table

| Task | Config | Mode | D | Accuracy | Solve | No-Last | Dec Init | Dec Mid | Dec Final | α_last | Runtime(s) |
|------|--------|------|---|----------|-------|---------|----------|---------|-----------|--------|------------|
| mod12_quarter | locked_stack | projective | 24 | **0.9653** | — | 0.9717 | 1.0 | 1.0 | 1.0 | 0.0349 | 107.5 |
| mod12_quarter | sincos_baseline | baseline | 24 | **0.9844** | — | 0.9761 | 1.0 | 1.0 | 1.0 | 0.0394 | 85.0 |
| mod12_quarter_noisy | locked_stack | projective | 24 | **0.7686** | — | 0.7593 | 1.0 | 1.0 | 1.0 | 0.0321 | 109.5 |
| mod12_quarter_noisy | sincos_baseline | baseline | 24 | **0.96** | — | 0.9653 | 1.0 | 1.0 | 1.0 | 0.0474 | 88.8 |
| mod8_quarter | locked_stack | projective | 24 | **0.9536** | 1500 | 0.9517 | 1.0 | 1.0 | 1.0 | 0.0058 | 73.5 |
| mod8_quarter | sincos_baseline | baseline | 24 | **0.896** | — | 0.9048 | 1.0 | 1.0 | 1.0 | 0.0877 | 63.9 |
| mod12_quarter | locked_stack | projective | 32 | **0.9028** | — | 0.9185 | 1.0 | 1.0 | 1.0 | 0.0173 | 151.9 |
| mod12_quarter | sincos_baseline | baseline | 32 | **0.9912** | — | 0.9956 | 1.0 | 1.0 | 1.0 | 0.0172 | 121.5 |
| mod12_quarter_noisy | locked_stack | projective | 32 | **0.9634** | — | 0.9731 | 1.0 | 1.0 | 1.0 | 0.035 | 154.7 |
| mod12_quarter_noisy | sincos_baseline | baseline | 32 | **0.9937** | — | 0.9941 | 1.0 | 1.0 | 1.0 | 0.044 | 122.6 |
| mod8_quarter | locked_stack | projective | 32 | **1.0** | 1500 | 1.0 | 1.0 | 1.0 | 1.0 | 0.024 | 96.2 |
| mod8_quarter | sincos_baseline | baseline | 32 | **0.8037** | — | 0.8062 | 1.0 | 1.0 | 1.0 | 0.0257 | 85.4 |

### Horizon Sensitivity: Δ(D32 − D24) by Task and Config

| Task | Config | D24 | D32 | Δ(D32−D24) | Decodability D32 |
|------|--------|-----|-----|------------|------------------|
| task_A (mod12_quarter) | locked_stack | 0.9653 | 0.9028 | **−0.0625** | 1.0 |
| task_A (mod12_quarter) | sincos_baseline | 0.9844 | 0.9912 | +0.0068 | 1.0 |
| task_B (mod12_quarter_noisy) | locked_stack | 0.7686 | 0.9634 | **+0.1948** | 1.0 |
| task_B (mod12_quarter_noisy) | sincos_baseline | 0.9600 | 0.9937 | +0.0337 | 1.0 |
| task_C (mod8_quarter) | locked_stack | 0.9536 | 1.0000 | **+0.0464** | 1.0 |
| task_C (mod8_quarter) | sincos_baseline | 0.8960 | 0.8037 | **−0.0923** | 1.0 |

### Locked Stack vs Sincos Baseline: D32 Accuracy Gap

| Task | D=32 locked_stack | D=32 sincos_baseline | Gap (ls−sc) | Winner |
|------|-------------------|---------------------|-------------|--------|
| task_A (mod12_quarter) | 0.9028 | 0.9912 | −0.0884 | sincos |
| task_B (mod12_quarter_noisy) | 0.9634 | 0.9937 | −0.0303 | sincos |
| task_C (mod8_quarter) | **1.0000** | 0.8037 | **+0.1963** | **locked_stack** |

---

## Key Questions

**Q1. Does the locked stack maintain performance across tasks?**

Mixed. Task A (control): locked_stack 0.9028 vs sincos 0.9912 — sincos wins by 0.0884. Task B (noisy): locked_stack 0.9634 vs sincos 0.9937 — sincos wins by 0.0303. Task C (structural): locked_stack **1.0000** vs sincos 0.8037 — locked_stack wins by 0.1963. The locked stack does not maintain uniform performance; it is task-sensitive.

**Q2. Does the locked stack maintain low or near-zero horizon regression?**

Partially. Task A: locked_stack regresses −0.0625 (clear), while sincos improves +0.0068. Task B: locked_stack dramatically *recovers* +0.1948 at D32 (noise effect diminishes with longer horizon), while sincos improves +0.0337. Task C: locked_stack improves +0.0464 to perfect 1.0, while sincos *regresses* −0.0923. The locked_stack avoids regression except on Task A.

**Q3. Does the locked stack consistently outperform the sin/cos controller?**

No. Sincos wins 2/3 tasks at D32 (Tasks A and B). Locked_stack wins 1/3 (Task C, strongly). The advantage is concentrated on structurally distinct tasks (different harmonic composition, k=2 vs k=3), not universal.

**Q4. Are there failure modes where the system breaks?**

Yes — one clear failure mode: **noise sensitivity at short horizons (D24)**. Task B locked_stack D24 = 0.7686 vs sincos D24 = 0.9600 (gap: −0.1914). The projective controller is more sensitive to embedding noise because its tangent-space features are a nonlinear function of the noisy embeddings, amplifying noise in the controller input. This recovers at D32 (gap narrows to −0.0303), suggesting the issue is gradient-quality-limited at short horizons.

**Q5. Does behavior suggest a coherent underlying mechanism?**

Partially. Two patterns suggest mechanism, not artifact: (1) Decodability = 1.0 in all 36 probes across all tasks, configs, and horizons — the geometric representation is invariant. (2) Task C: locked_stack achieves perfect 1.0 with solve_step=1500 and sincos *regresses* at D32, suggesting the projective controller exploits k=2 harmonic structure that sincos cannot. However, the Task A reversal (sincos dominates, contrary to controller_geometry_probe) is unexplained by a single seed and may reflect variance rather than mechanism.

---

## LOCKED STACK GENERALIZATION IS: **PARTIAL**

*Locked_stack wins strongly on structurally distinct harmonic tasks (Task C: +0.1963 at D32, perfect 1.0000). Sincos wins on mod12 tasks (Tasks A and B). Noise is a D24 failure mode for projective controller.*

---

## Honesty Section

### Where the locked stack holds
- **Task C (mod8, k=2 harmonic)**: locked_stack achieves 1.0000 at D32 with solve_step=1500, while sincos regresses to 0.8037. This is the clearest evidence of task-specific geometric advantage.
- **Noise at D32**: locked_stack recovers from 0.7686 → 0.9634 (+0.1948) across horizons, suggesting longer trajectory integration mitigates noise sensitivity.
- **Decodability**: 1.0 invariant holds perfectly across all 12 runs × 3 probe points = 36 checks. Harmonic state representation is never destroyed.

### Where the locked stack weakens
- **Task A (mod12, k=3 control)**: sincos wins 0.9912 vs 0.9028 at D32. This *contradicts* the prior controller_geometry_probe (where projective won 0.9946 vs 0.9648 on a mod12 task). Likely single-seed variance; cannot determine mechanism vs artifact at N=1.
- **Task B (noisy D24)**: projective drops to 0.7686 — a serious degradation. The projective nonlinearity amplifies embedding noise at short horizons. sincos is more noise-stable (0.9600) because its features are linear in sin/cos (which are bounded in [−1,1]).

### Whether failures look structural or incidental
- **Task B noise failure (D24)**: likely structural. Projective features proj_k = sin_k / (1 + cos_k + 0.1) are nonlinear and noise-amplifying when embeddings are perturbed. This is an architectural sensitivity.
- **Task A reversal**: likely incidental (single-seed variance). The architecture is identical to the prior probe; the discrepancy (0.9028 here vs 0.9878 in controller_geometry_probe at D24) is within plausible variance for a single seed run. Cannot classify as structural without multi-seed confirmation.

### Prior probe discrepancy
The controller_geometry_probe found projective D32=0.9946 > sincos D32=0.9648 on a mod12 task. This probe finds sincos D32=0.9912 > locked_stack D32=0.9028 on a mod12 task. Both are single-seed measurements. The source of the reversal is unknown but points to high variance in single-seed evaluation — a fundamental limitation of this experiment design.

### Limitations
- Single seed (42) — all numbers are point estimates
- Task B noise level (σ=0.1) is moderate but not calibrated to signal scale
- Task C uses only 40 states (vs larger pool for Tasks A/B); results may be noisier
- No hyperparameter re-tuning for Task C — LR, budget identical to Tasks A/B
- Only 2 controller modes tested per task (by design: locked stack vs sincos only)
- No D=40 tested (kept out to avoid scope expansion)
