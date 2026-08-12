# Prime Transport — Router Reintegration v5: Gated Token Injection

**Experiment type:** Gated (multiplicative) token injection
**Operator layer:** `geometry_native_operator_model_v25`
**No surface build. No files modified. No operators rebuilt.**

## What Changed from v4

Single injection interface change:

| | v4 (additive) | v5 (multiplicative) |
|-|---------------|---------------------|
| Injection | `soft_tau = (w @ tau_nexts) + W_tok_inject[tok]` | `soft_tau = (w @ tau_nexts) * (1 + W_gate[tok])` |
| New param | `W_tok_inject` (VOCAB × D_TAU) | `W_gate` (VOCAB × D_TAU, same shape) |
| Init | small random (σ=0.05) | small random (σ=0.05) → (1+gate) ≈ 1 |
| d_gate | additive update | `d_gate = d_soft_tau * base_val` |

All other components unchanged: Gumbel-softmax router, trajectory attention pooling, linear prediction head, training schedule.

## Why Multiplicative Injection

In v4, the additive bias `W_tok_inject[tok]` is added equally at every step. At D=16, 15 noise-token steps each inject their own additive offset, accumulating and diluting the encoding-step (step 0) signal. The ceiling at D=16 (~0.382 at 10k batches) reflects this structural signal dilution.

Multiplicative gating `(w @ tau_nexts) * (1 + W_gate[tok])` scales each individual tau dimension by a per-token factor. This modulates the geometry of the tau state rather than shifting it, so the operator-mix output is shaped by token identity at every dimension independently. The signal is multiplicatively preserved through subsequent steps rather than overwritten by additive accumulation.

## Training and Evaluation Setup

| Aspect | Value |
|--------|-------|
| Task | first-token recall, VOCAB=4 |
| Delays | D ∈ {2, 4, 8, 16} |
| Training budgets | 800 batches (vs. v4), 5000 batches (vs. v4_scaling) |
| Batch size | 32 |
| Temperature schedule | anneal 2.0→0.1 over batches 0-799, hold 0.1 |
| Eval episodes | 1000 per delay per checkpoint |
| Seed | 42 |

## Primary Results

### Accuracy at budget=800 (vs. v4 standalone)

| D | v5 gated | v4 additive | delta |
|---|----------|-------------|-------|
|  2 | 0.312 | 0.615 | -0.303 |
|  4 | 0.265 | 0.436 | -0.171 |
|  8 | 0.210 | 0.328 | -0.118 |
| 16 | 0.227 | 0.280 | -0.053 |

### Accuracy at budget=5000 (vs. v4_scaling)

| D | v5 gated | v4 additive | delta | beats v4? |
|---|----------|-------------|-------|-----------|
|  2 | 0.544 | 0.646 | -0.102 | no |
|  4 | 0.312 | 0.520 | -0.208 | no |
|  8 | 0.266 | 0.454 | -0.188 | no |
| 16 | 0.240 | 0.367 | -0.127 | no |

### D=16 Focus: Does gated injection break the additive ceiling?

| Benchmark | acc@D=16 |
|-----------|----------|
| v4 standalone (800b) | 0.280 |
| v4_scaling (5000b)   | 0.367 |
| v4 d16_ceiling (10000b) | 0.382 |
| **v5 gated (5000b)**    | **0.240** |

**Gated injection underperformed additive at D=16** (0.240 vs. 0.367). Multiplicative modulation did not improve long-delay retention.

## Root Cause: Why Multiplicative Gating Failed

The tau representation is constructed from concatenated one-hot sub-vectors (SWAP=2, COUPLED=5, TWIST=2, LIFT=12). The routed state `base = w @ tau_nexts` is a convex combination of one-hot-like vectors: for any tau dimension `d` that is **not activated** by any operator in the mix, `base[d] = 0`.

Under multiplicative gating: `soft_tau[d] = base[d] * (1 + gate[d]) = 0 * (1 + gate[d]) = 0`.

The gate is **entirely ineffective** on unactivated dimensions, regardless of `W_gate` values. Additive injection `base[d] + inject[d]` has no such restriction — it can write a non-zero value to any dimension unconditionally.

Because the one-hot tau encoding leaves most dimensions zero at any given step, multiplicative gating is strictly less expressive than additive injection for this representation. This explains why v5 performs worse across all delays, not just D=16.

## Routing Behavior

| Budget | D | transport_frac | route_entropy | collapsed? |
|--------|---|---------------|---------------|------------|
|   800 |  2 | 0.504 | 1.663 | no |
|   800 |  4 | 0.449 | 1.741 | no |
|   800 |  8 | 0.060 | 1.691 | no |
|   800 | 16 | 0.855 | 1.729 | no |
|  5000 |  2 | 0.892 | 1.466 | no |
|  5000 |  4 | 0.438 | 1.712 | no |
|  5000 |  8 | 0.097 | 1.692 | no |
|  5000 | 16 | 0.902 | 1.718 | no |

## Attention Weight at Step 0 (encoding step)

Uniform baselines: D=2→0.500, D=4→0.250, D=8→0.125, D=16→0.0625

| Budget | D | alpha[0] | near uniform? |
|--------|---|----------|---------------|
|   800 |  2 | 0.4998 | yes |
|   800 |  4 | 0.2501 | yes |
|   800 |  8 | 0.1251 | yes |
|   800 | 16 | 0.0625 | yes |
|  5000 |  2 | 0.4995 | yes |
|  5000 |  4 | 0.2501 | yes |
|  5000 |  8 | 0.1251 | yes |
|  5000 | 16 | 0.0625 | yes |

**Attention remained uniform at D=16.** alpha[0]=0.0625 ≈ uniform (0.0625). Gated injection does not by itself break the uniform attention failure mode.

## Honesty Section

### What improved

- Routing remained non-collapsed throughout all experiments.
- Minimal code change confirmed: only the injection formula changed.

### What failed or did not improve

- **Gated injection underperformed additive injection at every delay and every budget.** v5 is not a neutral swap: it is strictly worse.
- Root cause: the one-hot-sparse tau encoding makes multiplicative gating ineffective on unactivated dimensions (`base[d]=0 → output[d]=0` regardless of gate). Additive injection has no such restriction.
- D=16 gated accuracy at 5000b (0.240) is near chance (0.250), a regression from v4's 0.367.
- Attention remained uniform at D=16. Multiplicative gating did not introduce positional structure.
- Long-delay gap (D=2 vs. D=16) persists structurally regardless of injection type.

### What remains uncertain

- Whether position-aware injection (step 0 only, not all steps) would further improve D=16 by eliminating noise-step modulation entirely.
- Whether a combined approach (gated at step 0, no injection elsewhere) would outperform both v4 and v5 at long delays.
- Whether full exact spin_H is solved: **No.**

## Recommended Next Step

Gated injection at all steps did not materially improve over additive injection. Implement **step-0-only injection** (v6): inject token identity only at step 0 (the encoding step) and apply no injection at subsequent steps (t > 0). This eliminates noise-token injection entirely, preventing accumulation of non-encoding-step signal across the 15 noise steps of D=16.
