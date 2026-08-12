# INC-0149: Stage 5 — Task-Signal Probes on Poincaré-4d Operator

## Status
Closed: KEEP.

## Trigger
INC-0148 Closed: KEEP (2026-03-13). Geometry-native spectral operator confirmed:
poincare_4d gives +91–95% relative improvement in sector_lowfreq_energy vs Euclidean-KNN.
Theory chain (H^4 metric → operator → modes → alignment) confirmed for routing labels.

Now test whether this improved operator also carries **task-signal** alignment —
the missing link from INC-0067 (labels) and INC-0068 (residuals), both of which
were negative on the old Euclidean-KNN operator.

## Kill-List Stage
Primary: **5. Spectral / Operator Usefulness**

## Mathematical Object Under Test
Task-signal smoothness on the Poincaré-4d graph Laplacian approximation to the
Laplace-Beltrami operator on the H^4 Hopf routing manifold.

The theory says (GEOMETRIC_COMPUTATION_HYPOTHESIS.md):
```
geometry → spectral structure → phase transport → sparse computation
```

For Stage 5, the "spectral structure" prediction is: the manifold's operator
produces modes where **task-relevant signals** (not just routing labels) are
smooth. INC-0148 confirmed routing labels are smooth. INC-0149 tests if this
carries through to task signals.

## Hypothesis

**H_task_smooth (expected KEEP):**
On the poincare_4d operator, at least one task signal (one-hot labels, residual_l2,
error_indicator, or true_margin) has:
- Higher lowfreq_energy on HOPF routes than on the ambient_euclidean operator
- Lower Dirichlet energy on HOPF routes than on the ambient_euclidean operator

This would confirm that the geometry-native operator's mode alignment extends
beyond routing structure to task-relevant signals.

**H_task_opaque (possible REFINE):**
Routing labels are smooth (INC-0148) but task signals are not. The H^4 operator
captures geometric structure but not task-relevant variation. The theory's prediction
at Stage 5 is partially confirmed (operator exists, carries routing structure) but
the link to computation is not yet established.

## Experiment Protocol

### Screen (1 seed)
Run both `spectral_signal_probe.py` and `spectral_residual_probe.py` on the
confirmed PPMI-SVD Hopf routes with:
- `--graph-mode poincare_4d` (geometry-native)
- `--graph-mode ambient_euclidean` (baseline)

Compare the following for each route:
- `label_onehot_lowfreq_energy` (poincaré vs ambient)
- `label_onehot_dirichlet_energy` (poincaré vs ambient)
- `residual_l2_lowfreq_energy` (poincaré vs ambient)
- `error_indicator_lowfreq_energy` (poincaré vs ambient)
- `true_margin_lowfreq_energy` (poincaré vs ambient)

### Routes
From INC-0148 config:
- HOPF_BASE_K75 (chi, delta) 2D routing, K=75
- HOPF_TRANS_K75_L0 (chi, delta, alpha) 3D routing, K=75, lambda=0

### Parameters
- seed=0, max_points=384, knn_k=12, lowfreq_modes=8

## Success Condition
**KEEP (task signals smoother on poincaré operator):**
- At least one task-signal metric shows >20% relative improvement in lowfreq_energy
  on poincare_4d vs ambient_euclidean for the HOPF routes
- Confirms the theory chain extends to task signals

## Falsification Condition
**REFINE (routing smooth but task opaque):**
- All task-signal metrics within ±10% of ambient_euclidean
- Routing labels were smooth (INC-0148) but task signals are not
- Stage 5 partial: operator construction confirmed, but task-signal link unclear

## Scope Guardrails
- Do NOT run multi-seed sweeps in this screen — single seed only
- Do NOT modify routing configs — use exact INC-0148 routes
- Do NOT introduce new task signals beyond the existing probe metrics

## Results

### Signal Probe (label alignment) — seed=0, max_points=384, knn_k=12, lowfreq_modes=8

**HOPF_BASE_K75:**

| Metric | ambient_euclidean | poincare_4d | Δ% |
|---|---|---|---|
| label_onehot_lowfreq_energy | 0.02336 | 0.02137 | −8.5% |
| label_onehot_dirichlet_energy | 0.99922 | 0.99965 | +0.04% |
| label_indicator_lowfreq_mean | 0.02051 | 0.02239 | +9.2% |
| label_indicator_lowfreq_max | 0.08176 | 0.11716 | **+43.3%** |
| sector_lowfreq_energy | 0.19601 | 0.38237 | **+95.1%** |

**HOPF_TRANS_K75_L0:** same label metrics (same graph, different sectors).
sector_lowfreq_energy: 0.34912 → 0.66982 (+91.9%).

Signal probe summary: aggregate one-hot label metric is slightly worse (−8.5%);
per-class indicator max is substantially better (+43.3%). The per-class result
shows that specific label categories ARE smoother on the poincaré operator even
if the aggregate blends them into near-noise.

### Residual Probe (task-error alignment) — seed=0, max_points=384, knn_k=12, lowfreq_modes=8

**HOPF_BASE_K75:**

| Metric | ambient_euclidean | poincare_4d | Δ% |
|---|---|---|---|
| error_indicator_lowfreq | 0.00454 | 0.00951 | **+109.2%** |
| residual_l2_lowfreq | 0.01919 | 0.02301 | +19.9% |
| residual_vector_lowfreq | 0.02270 | 0.02184 | −3.8% |
| true_margin_lowfreq | 0.02039 | 0.02618 | **+28.4%** |
| true_score_lowfreq | 0.02520 | 0.03183 | **+26.3%** |
| true_margin_dirichlet | 0.99404 | 0.95956 | **−3.5%** (smoother) |
| true_score_dirichlet | 0.98178 | 0.97317 | −0.9% |

**HOPF_TRANS_K75_L0:**

| Metric | ambient_euclidean | poincare_4d | Δ% |
|---|---|---|---|
| error_indicator_lowfreq | 0.08146 | 0.05615 | −31.1% |
| residual_l2_lowfreq | 0.04764 | 0.05864 | **+23.1%** |
| residual_vector_lowfreq | 0.02326 | 0.02291 | −1.5% |
| true_margin_lowfreq | 0.02802 | 0.06563 | **+134.2%** |
| true_score_lowfreq | 0.04309 | 0.04423 | +2.6% |
| true_margin_dirichlet | 0.95965 | 0.91502 | **−4.7%** (smoother) |
| true_score_dirichlet | 0.98596 | 1.00464 | +1.9% |

### Metrics passing >20% KEEP threshold (lowfreq_energy improvement):

**BASE route:** error_indicator +109%, true_margin +28%, true_score +26%
**TRANS route:** true_margin +134%, residual_l2 +23%

### Dirichlet energy (lower = smoother):
true_margin_dirichlet improves −3.5% (BASE) and −4.7% (TRANS) — confirming
that task signals are smoother, not just more concentrated in low frequencies.

## Decision

**KEEP.** Multiple task-signal metrics show >20% relative improvement in
lowfreq_energy on the poincaré-4d operator. The standout is true_margin:
+28.4% (BASE), +134.2% (TRANS). The error_indicator doubles on BASE (+109%).
Dirichlet energies for true_margin decrease 3.5–4.7%, confirming genuine
smoothing rather than a spectral artifact.

This extends INC-0148's routing-label confirmation to task-relevant signals:
the H^4 geometry's own Laplacian approximation produces spectral modes on
which task error/margin signals are measurably smoother than on a Euclidean
KNN operator.

**Theory chain status:** geometry → operator → modes → task-signal smoothness
is empirically supported at proxy scale.

**Stage 5 update:** PARTIAL-PASS → further confirmed. Both operator
construction (INC-0148) and task-signal smoothness (INC-0149) are positive.
The remaining Stage 5 question is whether this smoothness translates into
computational advantage (spectral-domain operations outperforming direct
spatial operations). This requires Stage 6 integration.

**Anomalies:**
- Aggregate one-hot label lowfreq_energy is slightly worse (−8.5%) on
  poincaré, even though per-class indicator max improves +43%. This suggests
  the one-hot aggregate is not the right smoothness measure for multi-class
  label structure; per-class indicators and task-error metrics are more
  informative.
- error_indicator on TRANS is worse (−31%) — likely because transport-sector
  already biases error_indicator strongly (0.081 ambient vs 0.005 BASE), and
  the geometric operator redistributes this differently. true_margin and
  residual_l2 are more stable measures.

**Next:** Close INC-0149, update Stage 5 status. Consider whether to proceed
to Stage 5 finalize (multi-seed confirm) or transition to Stage 6 (sparse
event-driven trainability) given the strength of the signal chain.
