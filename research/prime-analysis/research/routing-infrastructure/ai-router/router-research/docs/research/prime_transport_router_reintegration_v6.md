# Prime Transport Router Reintegration Experiment v6

**Status:** Complete

## Hypothesis

In v4, token injection (`W_tok_inject[tok]`) was applied at every step. For D=16, this means steps t=1..15 each inject a noise token (random distractor), progressively diluting the encoding signal from step t=0. Restricting injection to t=0 should eliminate 15 noise injections at D=16 while preserving the encoding expressiveness at the one step that matters.

Expected outcome: D=16 accuracy above v4 ceiling (~0.382).

## Method

### Core change (v6 vs v4)

```
v4:  soft_tau_t = (w_t @ tau_nexts_t) + W_tok_inject[tok_t]   # ALL steps
v6:  soft_tau_0 = (w_0 @ tau_nexts_0) + W_tok_inject[x0]      # step 0 ONLY
     soft_tau_t = (w_t @ tau_nexts_t)                          # t > 0
```

Backward: `W_tok_inject` gradient is accumulated only at t=0.

### Unchanged from v4
- Gumbel-softmax routing (temperature annealing 2.0→0.1)
- Trajectory attention pooling
- Prediction head (D_TAU→VOCAB)
- Task: first-token recall at D ∈ {2,4,8,16}
- All operator definitions and geometry

### Runtime optimizations (from runtime_optimization_v1)
- tau_nexts memoization with full 10-field deterministic key
- Warmup pool (4000 pre-warmed states)

### Training budgets
- Checkpoints at 800, 2000, 5000 batches × 32 episodes
- Evaluation: 1000 episodes per configuration

## Results

### Accuracy by delay and budget

| D  | chance | v6@800 | v6@2000 | v6@5000 | v4@5000 | v5@5000 | Δ v6-v4 @5k |
|-----|--------|--------|---------|---------|---------|---------|-------------|
|  2  | 0.250  | 0.993 | 1.000  | 1.000  | 0.646  | 0.544  | +0.354       |
|  4  | 0.250  | 0.463 | 0.997  | 1.000  | 0.520  | 0.312  | +0.480       |
|  8  | 0.250  | 0.320 | 0.523  | 1.000  | 0.454  | 0.266  | +0.546       |
| 16  | 0.250  | 0.272 | 0.306  | 0.419  | 0.367  | 0.240  | +0.052       |

### Routing and attention metrics at 5000 batches

| D  | transport_frac | route_entropy | attention α[0] |
|----|---------------|--------------|----------------|
|  2 | 0.763           | 1.556          | 0.527           |
|  4 | 0.996           | 1.521          | 0.780           |
|  8 | 0.413           | 1.617          | 0.703           |
| 16 | 0.752           | 1.735          | 0.063           |

## Comparison vs v4 and v5

- **D=16:** v6 achieved 0.419 vs v4 ceiling 0.367 (Δ=+0.052). Step-0 injection **improved** long-delay retention.
- **D=2:** v6=1.000 vs v4=0.646 (Δ=+0.354)
- **D=4:** v6=1.000 vs v4=0.520 (Δ=+0.480)
- **D=8:** v6=1.000 vs v4=0.454 (Δ=+0.546)

v5 (multiplicative gating) underperformed at all delays due to sparse tau zeros blocking gate writes. v6 (step-0 additive) avoids that failure.

## Interpretation

### D=2, D=4, D=8: perfect accuracy achieved

Step-0 injection unlocks near-perfect or perfect recall at D=2, D=4, and D=8. The
model learns a clear strategy: encode x0 into `soft_tau_0` via `W_tok_inject[x0]`,
then have the attention mechanism upweight that step (α[0] rises to 0.527, 0.780,
0.703 respectively). The prediction head recovers x0 from the pooled trajectory.

v4 at 5000 batches reached only 0.646 / 0.520 / 0.454 for D=2/4/8 — well below the
perfect ceiling. The v4 noise injection at t=1..D-1 was not neutral; it was actively
diluting the encoding signal.

### D=16: improvement confirmed but ceiling not broken

Step-0 injection lifted D=16 from v4's 0.367 to 0.419 (Δ=+0.052). This confirms
that noise injection was causing some dilution at long delays. However, α[0]=0.063
at D=16 remains near uniform (uniform attention = 1/16=0.0625), meaning the attention
mechanism cannot isolate the encoding step across 16 evolution steps. The x0 signal
injected at t=0 contributes approximately 1/16 of the pooled trajectory vector — the
prediction head is working with a heavily diluted encoding.

The fundamental bottleneck at D=16 is signal persistence: soft_tau evolves through 15
uninjected operator applications, and the step-0 encoding does not dominate the
trajectory.

### Dilution hypothesis: partially confirmed

Noise injection (v4) was confirmed as a real bottleneck for D=2..8. For D=16,
step-0 injection helps (+0.052) but leaves a large gap vs shorter delays, pointing to
a residual bottleneck: 15 steps of unguided tau evolution dilute the step-0 encoding
below the attention mechanism's ability to concentrate on it.

## Limitations

- Chance baseline = 0.250 for all delays (VOCAB=4; the label is always one of 4 tokens
  regardless of delay length). D=2,4,8 all reach 1.000 — far above chance.
  D=16 at 0.419 is also well above chance (0.250), but note the gap from shorter delays.
- Pool-based warmup introduces a different state distribution from v4 (which used per-episode sequential warmup). Results are not numerically identical but are methodologically equivalent.
- 5000 batches × 32 episodes is a fixed budget; further scaling may shift results.

## Honesty Section

**What improved:**
- D=16 accuracy increased from v4 ceiling 0.367 to 0.419.
- D=2 accuracy improved: 0.646→1.000.
- D=4 accuracy improved: 0.520→1.000.
- D=8 accuracy improved: 0.454→1.000.

**What failed or did not change:**
- Attention α[0]=0.063 at D=16 remained near uniform (1/16=0.0625), so attention did not learn to focus on the encoding step.

**What remains uncertain:**
- Whether further training (>5000 batches) would widen or close the v6 vs v4 gap.
- Whether the attention mechanism can structurally learn to focus α[0] with a stronger training signal.
- Whether combining step-0 injection with a longer encoding embedding or a stronger MLP head would unlock further improvement.

## Files Modified

- No existing files were modified.
- `tools/prime_transport/run_router_reintegration_v6.py` (created)
- `docs/research/prime_transport_router_reintegration_v6.md` (created)
- `results/prime_transport_recursive_system/prime_transport_router_reintegration_v6.csv` (created)

## Next Step

Step-0 injection produced a meaningful improvement at D=16. **Next step: run a scaling study on v6 (analogous to v4_scaling) to determine whether the improvement continues to grow with budget or plateaus, and establish the v6 ceiling at D=16.**
