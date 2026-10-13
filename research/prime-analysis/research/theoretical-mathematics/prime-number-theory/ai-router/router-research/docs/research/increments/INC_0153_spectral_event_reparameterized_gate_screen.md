# INC-0153: Stage 6 — Re-Parameterized Event Gate Correlation Probe (Screen)

## Status
Closed: REFINE.

## Trigger
INC-0152 Closed: REFINE (2026-03-14). Event gate saturated at INC-0125 TAU002
parameters (threshold=0.0, tau=0.02). Gate_mean=0.959, active_frac=100%,
gate_std=0.004 — no discrimination. Spectral roughness ↔ error correlation
confirmed (Spearman r ≈ 0.47–0.53 on margin signal), but gate has no dynamic
range to test geometry-specific prediction.

## Kill-List Stage
Primary: **6. Sparse Event-Driven Trainability**

## Mathematical Object Under Test
Same as INC-0152: per-sample correlation between H^4 Poincaré graph Laplacian
spectral roughness and event-gate activation. Re-parameterized gate to center on
the actual error distribution.

## Gate Re-Parameterization Rationale
INC-0152 error statistics: error_mean=0.063, error_std=0.002.
- INC-0125 gate: threshold=0.0, tau=0.02 → sigmoid((0.063-0.0)/0.02)=0.959 (saturated)
- INC-0153 gate: threshold=0.063, tau=0.002 → sigmoid((0.063-0.063)/0.002)=0.500 (centered)
  - At error_mean−1σ: gate=0.269 (quiescent)
  - At error_mean+1σ: gate=0.731 (active)
  - Spread across ±1σ: 0.462 (vs 0.008 in INC-0152)

## Hypothesis

**H_screen:**
With a properly parameterized gate that has genuine dynamic range, per-sample
spectral roughness on the Poincaré-4d routing graph predicts event-gate
activation. Rough samples (high Dirichlet energy) fire the gate; smooth samples
are quiescent. The correlation is stronger on poincaré_4d than ambient_euclidean.

## Experiment Protocol

### Screen (1 seed)
Run spectral_event_correlation_probe with seed=0 for both graph modes:
- `spectral_inc0153_event_corr_repar_screen_poincare.json`
- `spectral_inc0153_event_corr_repar_screen_ambient.json`

### Routes
- HOPF_BASE_K75
- HOPF_TRANS_K75_L0

### Parameters
- seeds=[0], max_points=384, knn_k=12, lowfreq_modes=8
- event_gate_mode=soft_error, **event_gate_threshold=0.063**, **event_gate_tau=0.002**

## Success Condition
**KEEP (promote to confirm):** Spearman r > 0.3 between roughness and gate,
with poincaré_4d showing >10pp stronger correlation than ambient_euclidean.
Gate must have genuine variance (gate_std > 0.1, active_frac between 20%–80%).

## Falsification Condition
**KILL:** No significant correlation (|r| < 0.1) despite proper gate variance.

**REFINE:** Correlation exists but ambient_euclidean matches poincaré_4d
(geometry is not the source of the structure).

## Scope Guardrails
- Same tools as INC-0152 — no code changes
- Only gate parameters change (threshold, tau)
- 1 seed (screen protocol), no grid sweeps

## Results

### Gate statistics (re-parameterized: threshold=0.063, tau=0.002)
- HOPF_BASE_K75: gate_mean=0.5505, gate_std=0.1722, active_frac=79.4%
- HOPF_TRANS_K75_L0: gate_mean=0.5413, gate_std=0.1658, active_frac=77.3%
- Gate has genuine variance (std ≈ 0.17 vs 0.004 in INC-0152). Re-parameterization worked.

### Key correlations (margin signal, roughness_vs_gate Spearman)

| Route | poincaré_4d | ambient_euclidean | delta |
|---|---|---|---|
| HOPF_BASE_K75 | +0.529 | +0.502 | +0.027 |
| HOPF_TRANS_K75_L0 | +0.468 | +0.417 | +0.051 |

### Key correlations (margin signal, roughness_vs_gate Pearson)

| Route | poincaré_4d | ambient_euclidean | delta |
|---|---|---|---|
| HOPF_BASE_K75 | +0.250 | +0.238 | +0.012 |
| HOPF_TRANS_K75_L0 | +0.168 | +0.177 | −0.009 |

### Key correlations (error signal, highfreq_vs_gate Pearson)

| Route | poincaré_4d | ambient_euclidean | delta |
|---|---|---|---|
| HOPF_BASE_K75 | −0.461 | −0.458 | −0.003 |
| HOPF_TRANS_K75_L0 | −0.600 | −0.575 | −0.025 |

### Key correlations (error signal, highfreq_vs_gate Spearman)

| Route | poincaré_4d | ambient_euclidean | delta |
|---|---|---|---|
| HOPF_BASE_K75 | −0.103 | −0.096 | −0.007 |
| HOPF_TRANS_K75_L0 | −0.212 | −0.139 | −0.073 |

### Analysis

1. **Gate re-parameterization succeeded:** gate_std=0.17 (vs 0.004 in INC-0152),
   active_frac≈78% (vs 100%). The gate now has genuine dynamic range.

2. **Spearman correlations are identical to INC-0152:** Margin-signal
   roughness_vs_gate Spearman matches roughness_vs_error Spearman exactly. This
   is mathematically expected — the sigmoid gate is a strictly monotone function
   of error, so rank order is preserved. Re-parameterization doesn't change ranks.

3. **Pearson correlation improved:** roughness_vs_gate Pearson rose from
   +0.120/−0.005 (INC-0152, saturated) to +0.250/+0.168 (INC-0153, centered).
   The centered gate exposes the linear component of the relationship. But
   geometry delta remains tiny (±1.2pp).

4. **Correlation > 0.3 criterion passes on Spearman:** +0.529 (BASE), +0.468
   (TRANS). Both exceed 0.3 on both poincaré and ambient graphs.

5. **Geometry delta < 10pp criterion fails again:** +2.7pp (BASE), +5.1pp
   (TRANS). Consistently positive (poincaré always higher) but well below 10pp.

6. **The spectral-error-gate chain is real but geometry-agnostic at per-sample
   level:** The mechanism works — spectrally rough samples (high Dirichlet energy
   on the routing graph) have higher prediction error and thus higher gate
   activation. But this mechanism works nearly as well on the ambient Euclidean
   graph as on the Poincaré-4d graph.

### Interpretation

The per-sample correlation is not the right level to look for geometry advantage.
Stage 5 showed large aggregate differences (+40–77% on task-signal lowfreq and
sector metrics), but at the per-sample level the advantage is only +2.7–5.1pp.
This suggests the geometric advantage is structural/collective: the Poincaré
graph organizes routing neighbourhoods in a way that concentrates spectral energy
differently, but individual samples show similar roughness-error relationships
on either graph.

The next step for Stage 6 should test aggregate computational efficiency:
does the better aggregate spectral organization (Stage 5) lead to fewer
total gate firings or faster convergence, even if per-sample predictions
are geometry-agnostic?

### Decision
**Closed: REFINE.**

Gate re-parameterization confirmed: genuine gate variance achieved.
Per-sample spectral roughness ↔ gate correlation exists (Spearman r > 0.3) but
is geometry-agnostic at the per-sample level (delta < 10pp). The spectral-event
bridge operates through the error surface, not through per-sample geometric
structure.

Next direction: test aggregate Stage 6 efficiency — does the confirmed
superior spectral organization (Stage 5: +40–77%) produce measurably fewer
gate firings or faster convergence when training with event gating vs without?

### Raw output files
- `results/analysis/inc0153_event_corr_repar_screen_poincare_seed0.json`
- `results/analysis/inc0153_event_corr_repar_screen_poincare.json`
- `results/analysis/inc0153_event_corr_repar_screen_ambient_seed0.json`
- `results/analysis/inc0153_event_corr_repar_screen_ambient.json`
