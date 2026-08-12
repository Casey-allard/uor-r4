# INC-0154: Event-Gate Efficiency Interaction Screen

## Status
Open — screen in progress.

## Kill-List Stage
6 — Sparse Event-Driven Trainability

## Mathematical Object Under Test
Aggregate event-gate efficiency: does the superior spectral organization of geometric
routing (Stage 5: +40–77% lowfreq energy) produce measurably fewer gate firings or
better MSE-vs-cost tradeoff during event-gated training?

## Theory
INC-0152/0153 established that per-sample spectral roughness $f_i (Lf)_i$ correlates
with per-sample error (Spearman $r \approx 0.5$), but this correlation is
geometry-agnostic (delta +2.7–5.1pp between Poincaré and ambient graphs).

The mathematical reason: per-sample roughness is the *spatial* decomposition of the
quadratic form $f^T L f = \sum_i f_i (Lf)_i$. It depends only on each sample's local
KNN neighbors — any reasonable proximity graph detects similar local disagreements.

Stage 5 showed the advantage is in the *spectral* decomposition:
$f^T L f = \sum_k \lambda_k \alpha_k^2$. The Poincaré graph's eigenvectors
concentrate +40–77% more task-signal energy in low-frequency modes. This is a global
eigenstructural property, not a per-sample one.

**The correct test level is aggregate**: geometric routing creates more homogeneous
buckets (Stage 2/3: ORIG rel_diff=31–38% over COL_PERM), which should produce lower
per-sample errors during training, which should cause the event gate to fire less
(lower gate_mean), achieving the same or better MSE with fewer updates.

## Experimental Design

**2×2×2 factorial:**

| Factor | Level 0 | Level 1 |
|---|---|---|
| Routing geometry | HOPF_BASE_K75 (chi,delta) | HOPF_TRANS_K75 (chi,delta,alpha) |
| Input structure | ORIG (real PPMI) | COL_PERM (scrambled) |
| Event gate | OFF | T070 (threshold=0.07, tau=0.01) |

This gives 8 routes. The key comparisons are:

**Primary (gate efficiency × geometry interaction):**
- `gate_mean(ORIG_T070) < gate_mean(PERM_T070)` by >10pp for both BASE and TRANS
- MSE degradation ratio: $(MSE_{gated} - MSE_{off}) / MSE_{off}$ is smaller for ORIG

**Secondary (transport amplifies gate efficiency):**
- `gate_mean(TRANS_ORIG_T070) < gate_mean(BASE_ORIG_T070)` — transport further reduces updates
- Transport gate advantage > base gate advantage in absolute terms

## Config
- Screen: `configs/proxy_transfer_inc0154_event_gate_efficiency_screen.json`
- 8 routes: {BASE,TRANS} × {ORIG,PERM} × {GATEOFF,T070}
- Gate T070 params: threshold=0.07, tau=0.01 (INC-0125 reference, gate_mean≈0.32)
- Common: PPMI-SVD, K=75, phase4_dims=3,65,2,21, epochs=1, max_train=5000

## Success Condition
- `gate_mean(BASE_ORIG_T070) + 10pp < gate_mean(BASE_PERM_T070)`
- `gate_mean(TRANS_ORIG_T070) + 10pp < gate_mean(TRANS_PERM_T070)`
- AND: `mse(BASE_ORIG_T070) ≤ mse(BASE_ORIG_GATEOFF)` (gating doesn't hurt quality for ORIG)

## Falsification Condition
- `|gate_mean(ORIG_T070) - gate_mean(PERM_T070)| < 5pp` for both routing types
- AND: MSE degradation ratio is similar (within 20% relative) for ORIG and PERM

## Results

**Screen (seed 0), 8 routes:**

| Route | MSE | gate_mean | gate_active | error_mean |
|---|---|---|---|---|
| BASE_K75_ORIG_GATEOFF | 0.003995 | 1.000000 | 1.0000 | 0.063189 |
| BASE_K75_ORIG_T070 | 0.003891 | 0.319312 | 0.0000 | 0.062409 |
| BASE_K75_PERM_GATEOFF | 0.003997 | 1.000000 | 1.0000 | 0.063193 |
| BASE_K75_PERM_T070 | 0.003893 | 0.319628 | 0.0000 | 0.062427 |
| TRANS_K75_ORIG_GATEOFF | 0.003987 | 1.000000 | 1.0000 | 0.063185 |
| TRANS_K75_ORIG_T070 | 0.003891 | 0.319112 | 0.0000 | 0.062395 |
| TRANS_K75_PERM_GATEOFF | 0.003999 | 1.000000 | 1.0000 | 0.063201 |
| TRANS_K75_PERM_T070 | 0.003893 | 0.319570 | 0.0000 | 0.062423 |

**Key comparisons:**
- BASE gate_mean delta (PERM − ORIG): +0.0pp (threshold: >10pp)
- TRANS gate_mean delta (PERM − ORIG): +0.0pp (threshold: >10pp)
- BASE MSE degradation from gating: ORIG −2.59%, PERM −2.61% (gating helps slightly)
- TRANS MSE degradation from gating: ORIG −2.42%, PERM −2.66% (gating helps slightly)
- error_mean is ~0.0624 for ALL conditions (identical to 4th decimal)

**Falsification criterion met:** `|gate_mean(ORIG) − gate_mean(PERM)| < 0.1pp ≪ 5pp` for
both BASE and TRANS. MSE degradation ratios are within 7% relative of each other.

### Mathematical Interpretation

The event gate computes `sigmoid((error_mag − 0.07) / 0.01)`. The per-sample prediction
error is what determines gate firing, and prediction error is determined by the EMA
prototype training dynamics — **not by the routing quality**.

After training, each bucket's prototypes adapt to predict the local neighborhood of
whatever samples are routed there. Whether routing is semantically meaningful (ORIG) or
scrambled (COL_PERM), the prototypes converge to similar per-sample MSE because:

1. Column permutation preserves the marginal distributions of each dimension
2. EMA prototyping is a localized learning rule that fits to whatever data appears in each bucket
3. The routing quality affects WHICH samples share a bucket, not HOW WELL those samples
   are locally predicted

This means the geometric advantage from Stages 2–5 operates at the **bucket-organization
level** (which tokens are grouped together), not at the per-sample prediction-error level
that the event gate sees. The Stages 2–5 spectral differences (+40–77% lowfreq_energy)
manifest in aggregate structural metrics, but the per-sample error surface — which is what
drives the event gate — is equalized by the EMA training process.

### Implications for Stage 6

The event gate as currently designed (error-magnitude-based gating) does not interact
with routing quality. The geometric advantage from fixed H^4 routing is real (Stages 2–5
confirmed), but it does not propagate to the error-based sparse event mechanism because
the EMA prototype learning equalizes per-sample errors regardless of bucket quality.

**For sparse event-driven trainability to become geometry-dependent, the gate must use
a signal that depends on routing quality rather than per-sample prediction error.**

Possible REFINE directions:
1. **Route-quality gate**: gate on bucket-level coherence (routing metric → sparsity)
   rather than per-sample error
2. **Spectral gate**: gate on per-sample low-frequency projection rather than error magnitude
3. **Multi-epoch cumulative**: test whether gate behavior diverges over multiple epochs
   (routing quality might affect convergence RATE rather than 1-epoch error distribution)
4. **Architecture-level sparsity**: abandon the per-sample gate entirely and measure
   hardware savings from routing compression (fewer buckets needed for same MSE)

## Decision
Closed: REFINE.

Per-sample prediction error is equalized by EMA prototype training, making the error-based
event gate routing-agnostic. The geometric advantage operates at the bucket-organization
level, not the per-sample prediction level. Stage 6 gate must use a routing-quality signal
instead of error magnitude, or the sparsity mechanism must be redefined at the architecture
level (routing compression rather than per-sample gating).

