# INC-0152: Stage 6 — Spectral-Event Correlation Probe (Screen)

## Status
Closed: REFINE.

## Trigger
INC-0151 Closed: KEEP (2026-03-13). Stage 5 finalized: task-signal smoothness on
Poincaré-4d operator confirmed at 4 seeds. true_margin_lowfreq +40–48%,
label_indicator_max +57%, sector +72–77%.

Stage 5 remaining question: does spectral smoothness translate to computational
advantage? This bridges to Stage 6.

## Kill-List Stage
Primary: **6. Sparse Event-Driven Trainability**

## Mathematical Object Under Test
Per-sample correlation between H^4 Poincaré graph Laplacian spectral roughness
(Stage 5) and event-gate quiescence (Stage 6). The Dirichlet energy per sample
f_i × (L f)_i measures local disagreement with graph neighbours. The event gate
sigmoid((error - threshold) / tau) measures whether a sample triggers compute.

## Hypothesis

**H_screen:**
Spectrally smooth regions on the Poincaré-4d routing graph correspond to regions
where the event gate is quiescent (low error → gate inactive). This would
demonstrate that Stage 5's confirmed spectral structure directly predicts
Stage 6's sparse-event behaviour — the geometry controls which samples need
compute.

## Experiment Protocol

### Screen (1 seed)
Run spectral_event_correlation_probe.py with seed=0 for both graph modes:
- `spectral_inc0152_event_corr_screen_poincare.json` (graph_mode: poincare_4d)
- `spectral_inc0152_event_corr_screen_ambient.json` (graph_mode: ambient_euclidean)

### Routes
- HOPF_BASE_K75
- HOPF_TRANS_K75_L0

### Parameters
- seeds=[0], max_points=384, knn_k=12, lowfreq_modes=8
- event_gate_mode=soft_error, event_gate_threshold=0.0, event_gate_tau=0.02

### Signals analyzed
Two task signals (same as Stage 5):
1. `error_indicator` — binary classification error (0/1)
2. `true_margin` — margin between true-class score and best-other score

### Correlation measures
- Per-sample local Dirichlet roughness vs event-gate value (Pearson + Spearman)
- Per-sample high-frequency residual vs event-gate value (Pearson + Spearman)
- Per-sample roughness vs error magnitude (validation link)

## Success Condition
**KEEP (promote to confirm):** Pearson or Spearman rank correlation > 0.3
between local spectral roughness and event-gate activation, with poincaré_4d
graph showing meaningfully stronger correlation (>10pp) than ambient_euclidean
baseline.

## Falsification Condition
**KILL:** No significant correlation (|r| < 0.1) on either graph mode.

**REFINE:** Correlation exists but ambient_euclidean matches poincaré_4d
(geometry is not the source of the structure).

## Scope Guardrails
- Uses existing router_proxy_eval.py event gate infrastructure (no changes)
- Uses existing spectral_route_audit.py decomposition (no changes)
- New tool: spectral_event_correlation_probe.py (bridge probe)
- New tool: spectral_event_sweep.py (multi-seed orchestrator)
- 1 seed (screen protocol), no grid sweeps

## Prior Stage 6 Work
- INC-0125: Near-hard sparse event proxy trainability (TAU002 reference: MSE=0.003859)
- INC-0130: Soft score-bias coupling pilot (SBI030 balanced candidate)
- INC-0131: 4-seed carry-forward confirm (quality lift confirmed under prewarm)
- Gap between proxy and translated systems remains the key Stage 6 blocker

## Results

### Gate statistics
- gate_mean = 0.959, gate_std = 0.004, active_frac = 100%
- error_mean = 0.063, error_std = 0.002
- sigmoid((0.063 - 0.0) / 0.02) = 0.959 → **gate is saturated**
  Every sample has gate ≈ 0.96. The event gate at INC-0125 TAU002 parameters
  (threshold=0.0, tau=0.02) does not discriminate on PPMI-SVD proxy.

### Key correlations (margin signal, roughness_vs_gate Spearman)

| Route | poincaré_4d | ambient_euclidean | delta |
|---|---|---|---|
| HOPF_BASE_K75 | +0.529 | +0.502 | +0.027 |
| HOPF_TRANS_K75_L0 | +0.468 | +0.417 | +0.051 |

### Key correlations (margin signal, roughness_vs_error Spearman)

| Route | poincaré_4d | ambient_euclidean | delta |
|---|---|---|---|
| HOPF_BASE_K75 | +0.529 | +0.502 | +0.027 |
| HOPF_TRANS_K75_L0 | +0.468 | +0.417 | +0.051 |

### Key correlations (error signal, highfreq_vs_gate Pearson)

| Route | poincaré_4d | ambient_euclidean | delta |
|---|---|---|---|
| HOPF_BASE_K75 | −0.636 | −0.634 | −0.002 |
| HOPF_TRANS_K75_L0 | −0.759 | −0.736 | −0.023 |

### Analysis

1. **Spectral roughness ↔ error correlation exists (positive):** Margin-signal
   Dirichlet roughness correlates with error magnitude at Spearman r ≈ 0.47–0.53.
   Direction is correct: spectrally rough samples have higher prediction error.
   This validates the Stage 5 → Stage 6 bridge hypothesis at the signal level.

2. **Event gate is saturated:** With threshold=0.0 and tau=0.02, the gate sigmoid
   maps the actual error range (mean=0.063 ± 0.002) to gate ≈ 0.959 ± 0.004.
   There is almost no gate variance to correlate with. The roughness_vs_gate
   Spearman tracks roughness_vs_error because gate is a near-monotone transform
   of error, but the gate has essentially no dynamic range.

3. **Poincaré vs ambient delta is small:** Geometry discrimination is +2.7pp (BASE)
   to +5.1pp (TRANS) on margin-signal roughness_vs_gate Spearman. This is below
   the 10pp threshold specified in the success condition.

4. **High-frequency residual vs gate (Pearson) is strong but geometry-agnostic:**
   r ≈ −0.64 to −0.76 on both graphs (delta < 2.3pp). The negative sign indicates
   high-frequency samples have lower gate values — but this is an artifact of
   the saturated gate (lower gate = closer to the saturation floor near 1.0, driven
   by slightly lower error in high-freq regions of the error-indicator signal).

### Diagnosis

The event gate parameterization from INC-0125 (threshold=0.0, tau=0.02) was
calibrated for a different error regime. On PPMI-SVD proxy with the Stage 5
confirmed routes, error magnitudes (~0.063) are far above threshold (0.0),
pushing the sigmoid into saturation. The gate does not discriminate.

The spectral roughness ↔ error correlation (r ≈ 0.47–0.53) is genuine and exceeds
the 0.3 threshold. But the event gate's lack of dynamic range means we cannot
test whether geometry-native spectral structure predicts sparse-event firing.

### Decision
**Closed: REFINE.**

Spectral roughness ↔ prediction error correlation exists (Spearman r > 0.3),
confirming the Stage 5 → Stage 6 bridge at the signal level. But the event gate
is saturated at INC-0125 parameters — threshold=0.0 is too far below the actual
error distribution (mean=0.063). Need to re-parameterize the gate:

- **INC-0153:** Re-run with threshold centered on error distribution (threshold ≈ 0.06,
  tau ≈ 0.01) to create genuine gate variance. Then test whether poincaré_4d
  spectral structure predicts which samples the gate fires on.

### Raw output files
- `results/analysis/inc0152_event_corr_screen_poincare_seed0.json`
- `results/analysis/inc0152_event_corr_screen_poincare.json`
- `results/analysis/inc0152_event_corr_screen_ambient_seed0.json`
- `results/analysis/inc0152_event_corr_screen_ambient.json`
