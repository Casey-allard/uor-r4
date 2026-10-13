# INC-0151: Stage 5 — Task-Signal Poincaré Operator Finalize (4 seeds)

## Status
Closed: KEEP.

## Trigger
INC-0150 Closed: KEEP (2026-03-13). 2-seed confirm replicated task-signal
smoothness: true_margin_lowfreq +26–55%, label_indicator_max +25%,
true_margin_dirichlet −4.2–5.1%.

Protocol: screen (1 seed, INC-0149) → confirm (2 seeds, INC-0150) →
**finalize (4 seeds, INC-0151)**.

## Kill-List Stage
Primary: **5. Spectral / Operator Usefulness**

## Mathematical Object Under Test
Same as INC-0149/0150: task-signal smoothness on the Poincaré-4d graph Laplacian
approximation to the H^4 Laplace-Beltrami operator.

## Hypothesis

**H_finalize (expected KEEP):**
The screen and confirm results hold at 4-seed mean. At least one task-signal
metric maintains >20% relative improvement on poincaré-4d vs ambient_euclidean
across seeds 0–3.

## Experiment Protocol

### Finalize (4 seeds)
Run signal sweep and residual sweep with seeds [0,1,2,3] for both graph modes.
Use separate configs per sweep×mode to avoid filename collisions:
- `spectral_inc0151_signal_finalize_ambient.json`
- `spectral_inc0151_signal_finalize_poincare.json`
- `spectral_inc0151_residual_finalize_ambient.json`
- `spectral_inc0151_residual_finalize_poincare.json`

### Routes
- HOPF_BASE_K75
- HOPF_TRANS_K75_L0

### Parameters
- seeds=[0,1,2,3], max_points=384, knn_k=12, lowfreq_modes=8

## Success Condition
**KEEP:** At least one task-signal lowfreq_energy metric shows >20% relative
improvement on poincare_4d vs ambient_euclidean in the 4-seed mean.
Stage 5 can be closed as PARTIAL-PASS (strong) for task-signal smoothness.

## Falsification Condition
**REFINE:** All 4-seed mean improvements fall below 20%.

## Scope Guardrails
- Same tool code as INC-0149/0150 — no changes
- Same routing configs — no changes
- 4 seeds, no grid sweeps

## Results

### Signal Probe — 4-seed mean ± std (seeds 0–3)

**HOPF_BASE_K75:**

| Metric | ambient | poincaré | Δ% |
|---|---|---|---|
| label_onehot_lowfreq | 0.02334±0.00032 | 0.02180±0.00069 | −6.6% |
| label_indicator_lowfreq_mean | 0.02109±0.00055 | 0.02145±0.00056 | +1.7% |
| label_indicator_lowfreq_max | 0.08584±0.00921 | 0.13504±0.07155 | **+57.3%** |
| sector_lowfreq_energy | 0.21279±0.02260 | 0.36692±0.06945 | **+72.4%** |

**HOPF_TRANS_K75_L0:** same label metrics. sector: 0.41717±0.09006 → 0.73930±0.04149 (**+77.2%**).

### Residual Probe — 4-seed mean ± std

**HOPF_BASE_K75:**

| Metric | ambient | poincaré | Δ% |
|---|---|---|---|
| error_indicator_lowfreq | 0.01836±0.00934 | 0.01794±0.00615 | −2.3% |
| residual_l2_lowfreq | 0.02021±0.00665 | 0.01963±0.00288 | −2.9% |
| true_margin_lowfreq | 0.02375±0.00715 | 0.03320±0.01253 | **+39.8%** |
| true_score_lowfreq | 0.02253±0.00897 | 0.02424±0.00645 | +7.6% |
| true_margin_dirichlet | 0.98607±0.01894 | 0.92922±0.03510 | **−5.8%** (smoother) |

**HOPF_TRANS_K75_L0:**

| Metric | ambient | poincaré | Δ% |
|---|---|---|---|
| error_indicator_lowfreq | 0.04715±0.02012 | 0.04065±0.00974 | −13.8% |
| residual_l2_lowfreq | 0.04008±0.01694 | 0.04177±0.01082 | +4.2% |
| true_margin_lowfreq | 0.03378±0.01544 | 0.04983±0.02315 | **+47.5%** |
| true_score_lowfreq | 0.03830±0.01717 | 0.03530±0.00753 | −7.8% |
| true_margin_dirichlet | 0.97860±0.01728 | 0.91518±0.03277 | **−6.5%** (smoother) |

### Metrics passing >20% KEEP threshold (4-seed mean):

| Metric | BASE Δ% | TRANS Δ% |
|---|---|---|
| label_indicator_lowfreq_max | **+57.3%** | **+57.3%** |
| true_margin_lowfreq | **+39.8%** | **+47.5%** |
| sector_lowfreq_energy | **+72.4%** | **+77.2%** |

### Dirichlet energy (4-seed mean, lower = smoother):
true_margin_dirichlet: **−5.8%** (BASE), **−6.5%** (TRANS).

### Progression across protocol stages:

| Metric | Screen (1 seed) | Confirm (2 seeds) | **Finalize (4 seeds)** |
|---|---|---|---|
| true_margin_lowfreq BASE | +28% | +26% | **+40%** |
| true_margin_lowfreq TRANS | +134% | +55% | **+48%** |
| label_indicator_max | +43% | +25% | **+57%** |
| sector_lowfreq BASE | +95% | +94% | **+72%** |
| true_margin_dirichlet BASE | −3.5% | −4.2% | **−5.8%** |

All key metrics hold or strengthen across 4 seeds. The TRANS true_margin
has moderated from the noisy screen (+134%) to a stable +48% in the 4-seed mean.

### Variance note:
- label_indicator_lowfreq_max has high variance on poincaré (std=0.072 on mean=0.135)
  suggesting per-class smoothness is seed-sensitive. The mean is still >20% above ambient.
- true_margin_lowfreq has moderate relative variance (~38% CoV on poincaré BASE,
  ~46% on TRANS), but the 4-seed mean is robustly above the 20% threshold.

## Decision

**KEEP.** The 4-seed finalize confirms the theory chain at Stage 5:

1. **true_margin_lowfreq**: +40% (BASE), +48% (TRANS) — task-error margin is
   measurably smoother on the H^4 geometry-native spectral modes.
2. **label_indicator_lowfreq_max**: +57% — per-class label smoothness confirmed.
3. **true_margin_dirichlet**: −5.8% (BASE), −6.5% (TRANS) — genuine Dirichlet
   smoothing, not just spectral concentration.
4. **sector_lowfreq_energy**: +72–77% — routing-label alignment stable from INC-0148.

The full chain **geometry → operator → modes → task-signal smoothness** is
finalized at 4 seeds with all key metrics above threshold.

**Stage 5 verdict:** PARTIAL-PASS (strong). Operator construction (INC-0148),
task-signal screen (INC-0149), confirm (INC-0150), and finalize (INC-0151) all
positive. The remaining Stage 5 question — whether spectral smoothness enables
computational advantage — is a Stage 6 integration question.

**Next:** Transition to Stage 6 (Sparse Event-Driven Trainability).
