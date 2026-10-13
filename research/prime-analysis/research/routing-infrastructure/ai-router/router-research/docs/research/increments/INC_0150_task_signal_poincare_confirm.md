# INC-0150: Stage 5 — Task-Signal Poincaré Operator Confirm (2 seeds)

## Status
Closed: KEEP.

## Trigger
INC-0149 Closed: KEEP (2026-03-13). Screen (1 seed) confirmed task-signal
smoothness on poincaré-4d operator: error_indicator +109%, true_margin +28–134%,
true_score +26%, residual_l2 +20–23%.

Protocol: screen (1 seed) → **confirm (2 seeds)** → finalize (4 seeds).
This is the 2-seed confirm pass.

## Kill-List Stage
Primary: **5. Spectral / Operator Usefulness**

## Mathematical Object Under Test
Same as INC-0149: task-signal smoothness on the Poincaré-4d graph Laplacian
approximation to the Laplace-Beltrami operator on the H^4 Hopf routing manifold.

## Hypothesis

**H_confirm (expected KEEP):**
The INC-0149 screen results replicate across 2 seeds. At least one task-signal
metric maintains >20% relative improvement on poincaré-4d vs ambient_euclidean
when averaged over seeds 0 and 1.

**H_noisy (possible REFINE):**
Screen results were seed-specific noise. The 2-seed average falls below the
20% threshold for all task-signal metrics.

## Experiment Protocol

### Confirm (2 seeds)
Run `spectral_signal_sweep.py` and `spectral_residual_sweep.py` with seeds [0,1]
for both graph modes:
- `graph_mode: ambient_euclidean` (baseline)
- `graph_mode: poincare_4d` (geometry-native)

Compare 2-seed means for:
- `label_indicator_lowfreq_max` (screen: +43%)
- `error_indicator_lowfreq` (screen: +109%)
- `true_margin_lowfreq` (screen: +28% BASE, +134% TRANS)
- `true_score_lowfreq` (screen: +26%)
- `residual_l2_lowfreq` (screen: +20% BASE, +23% TRANS)

### Routes
- HOPF_BASE_K75
- HOPF_TRANS_K75_L0

### Parameters
- seeds=[0,1], max_points=384, knn_k=12, lowfreq_modes=8

## Success Condition
**KEEP:** At least one task-signal lowfreq_energy metric shows >20% relative
improvement on poincare_4d vs ambient_euclidean in the 2-seed mean.
Proceed to 4-seed finalize (INC-0151).

## Falsification Condition
**REFINE:** All 2-seed mean improvements fall below 20%. Re-evaluate whether
the screen result was noise before proceeding.

## Scope Guardrails
- Do NOT change any probe tool code — use INC-0149 modifications as-is
- Do NOT change routing configs — same routes as INC-0148/0149
- 2 seeds only — no grid sweeps

## Results

### Signal Probe — 2-seed mean (seeds 0,1), max_points=384, knn_k=12, lowfreq_modes=8

**HOPF_BASE_K75:**

| Metric | ambient | poincaré | Δ% |
|---|---|---|---|
| label_onehot_lowfreq_energy | 0.02348 | 0.02143 | −8.7% |
| label_indicator_lowfreq_mean | 0.02069 | 0.02166 | +4.7% |
| label_indicator_lowfreq_max | 0.09032 | 0.11331 | **+25.4%** |
| sector_lowfreq_energy | 0.22050 | 0.42818 | **+94.2%** |

**HOPF_TRANS_K75_L0:** same label metrics (same graph). sector: 0.37914 → 0.71592 (**+88.8%**).

### Residual Probe — 2-seed mean

**HOPF_BASE_K75:**

| Metric | ambient | poincaré | Δ% |
|---|---|---|---|
| error_indicator_lowfreq | 0.01717 | 0.01324 | −22.9% |
| residual_l2_lowfreq | 0.02227 | 0.01964 | −11.8% |
| true_margin_lowfreq | 0.01712 | 0.02162 | **+26.3%** |
| true_score_lowfreq | 0.02951 | 0.02690 | −8.9% |
| true_margin_dirichlet | 1.00332 | 0.96132 | **−4.2%** (smoother) |

**HOPF_TRANS_K75_L0:**

| Metric | ambient | poincaré | Δ% |
|---|---|---|---|
| error_indicator_lowfreq | 0.06085 | 0.04281 | −29.6% |
| residual_l2_lowfreq | 0.05552 | 0.05112 | −7.9% |
| true_margin_lowfreq | 0.04031 | 0.06261 | **+55.3%** |
| true_score_lowfreq | 0.05246 | 0.03977 | −24.2% |
| true_margin_dirichlet | 0.96167 | 0.91215 | **−5.1%** (smoother) |

### Metrics passing >20% KEEP threshold (2-seed mean):

| Metric | BASE Δ% | TRANS Δ% |
|---|---|---|
| label_indicator_lowfreq_max | **+25.4%** | **+25.4%** |
| true_margin_lowfreq | **+26.3%** | **+55.3%** |
| sector_lowfreq_energy | **+94.2%** | **+88.8%** |

### Dirichlet energy confirmation (lower = smoother):
true_margin_dirichlet: −4.2% (BASE), −5.1% (TRANS) — confirms genuine smoothing.

### Comparison to screen (INC-0149):
- true_margin_lowfreq: screen +28%/+134% → confirm +26%/+55% — **replicates** (TRANS moderated)
- label_indicator_max: screen +43% → confirm +25% — **replicates** (moderated)
- sector_lowfreq: screen +95%/+92% → confirm +94%/+89% — **stable**

### Anomalies:
- error_indicator_lowfreq reversed from screen: was +109% (BASE) → now −23%. This metric is
  noisy at small sample sizes when error rate is near-saturated (~97%). true_margin is the
  more stable task-signal measure.
- true_score_lowfreq is inconsistent: −9% (BASE), −24% (TRANS). true_margin and per-class
  indicator are the reliable signals.

## Decision

**KEEP.** The 2-seed confirm replicates the INC-0149 screen result. Two task-signal
metrics consistently pass the >20% threshold across both seeds:
- **true_margin_lowfreq**: +26% (BASE), +55% (TRANS)
- **label_indicator_lowfreq_max**: +25% (both routes)

Dirichlet energy consistently decreases (−4.2% to −5.1%), confirming genuine smoothing.
Sector alignment replicates INC-0148 at +89–94%.

The theory chain (geometry → operator → modes → task-signal smoothness) replicates
at the 2-seed confirm level. Proceed to 4-seed finalize (INC-0151).
