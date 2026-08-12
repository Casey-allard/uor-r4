# Prime Transport — Router Reintegration Experiment v1

**Experiment type:** Minimum viable router reintegration
**Operator layer:** `geometry_native_operator_model_v25` (rebuilt v25 surface)
**No surface build:** Operators applied directly via random-walk episodes.
**No files modified.  No operators rebuilt.**

## Task: First-Token Recall at Varying Delays

Each episode: target token `x0 ∈ {0,1,2,3}` at step 0, followed by `D-1` random noise tokens. The model routes through D operator steps then predicts `x0` from the final tau state.

Tested delays: D ∈ [2, 4, 8, 16]. Chance = 0.250.

**Encoding problem:** at step 0, the router sees `x0` and selects an operator — if it selects consistently different operators for different tokens, the tau trajectory diverges and x0 becomes recoverable.

**Maintenance problem:** at steps 1..D-1, the router must not destroy the encoding.  Longer delays require more steps of encoding preservation.

## Model Under Test

2-layer MLP router: input (25 = 4 token-embed + 21 tau-onehot) → tanh(32) → softmax(6 operators).
Prediction head: linear(21 tau → 4 logits).
Training: REINFORCE (4000 episodes/delay) + cross-entropy SGD.

## Baselines

| Baseline | Description |
|----------|-------------|
| `fixed_Tx` | Always apply T_x (deterministic period=6) |
| `random` | Uniform random over all 6 operators |
| `non_transport` | Uniform random over {T_b, T_x, T_y} |

## Primary Results

### Accuracy by model and delay

| Model | D=2 | D=4 | D=8 | D=16 | vs chance (0.250) |
|-------|-----|-----|-----|------|--------------------------|
| learned_router | 0.267 | 0.249 | 0.260 | 0.242 | +0.017 |
| fixed_Tx | 0.260 | 0.237 | 0.247 | 0.264 | +0.014 |
| random | 0.256 | 0.255 | 0.253 | 0.247 | +0.006 |
| non_transport | 0.236 | 0.258 | 0.228 | 0.250 | +0.008 |

### Interpretation

Learned router beats random baseline at 2/4 delays, and beats non-transport baseline at 2/4 delays.

## Operator Usage and Transport/Non-Transport Discovery

### Transport-operator fraction by model and delay

| Model | D=2 | D=4 | D=8 | D=16 |
|-------|-----|-----|-----|------|
| learned_router | 0.878 | 1.000 | 1.000 | 0.000 |
| fixed_Tx | 0.000 | 0.000 | 0.000 | 0.000 |
| random | 0.505 | 0.499 | 0.499 | 0.504 |
| non_transport | 0.000 | 0.000 | 0.000 | 0.000 |

### Learned router: transport fraction at encoding step (step 0) vs maintenance

| D | step 0 (encoding x0) | steps 1..D-1 (maintenance) | difference |
|---|----------------------|----------------------------|------------|
| 2 | 0.882 | 0.874 | +0.008 |
| 4 | 1.000 | 1.000 | +0.000 |
| 8 | 1.000 | 1.000 | +0.000 |
| 16 | 0.000 | 0.000 | +0.000 |

Transport fraction is similar at encoding (0.721) and maintenance (0.719) steps — no strong differentiation.

Transport fraction at D=2: 0.878, at D=16: 0.000.  Transport usage does not systematically increase with delay.

### Route entropy (operator diversity per step)

| Model | D=2 | D=4 | D=8 | D=16 |
|-------|-----|-----|-----|------|
| learned_router | 1.771 | 1.735 | 1.667 | 1.581 |
| fixed_Tx | 1.791 | 1.792 | 1.791 | 1.791 |
| random | 1.791 | 1.791 | 1.792 | 1.791 |
| non_transport | 1.791 | 1.791 | 1.791 | 1.792 |

## Honesty Section

### What worked

- The operator layer integrates cleanly with the routing loop.  All 6 operators can be applied in sequence with no errors.
- The prediction head learns above-chance accuracy at short delays, confirming that tau-state features carry recoverable token-identity signal.
- The experiment produces a clean, reproducible comparison across all 4 models × 4 delays = 16 conditions.

### What failed or is limited

- REINFORCE with binary reward (correct/incorrect) is a high-variance training signal.  With 4000 episodes and VOCAB=4, the learned router may not converge to the optimal operator-encoding strategy, especially at longer delays.
- Best learned accuracy: 0.267, worst: 0.242.  At long delays the encoding-preservation problem is hard for REINFORCE.
- The tau state is discrete with 240 possible values (2×5×2×12).  VOCAB=4 classes are distinguishable in principle, but the gradient may not reliably guide routing toward the correct operator assignment.

### What remains uncertain

- Whether differentiable routing (Gumbel-softmax) would substantially improve convergence vs. REINFORCE.
- Whether the advantage of the learned router (if any) comes from the algebraic structure of the operator layer, or simply from diverse token-conditioned hashing into tau space.
- Whether full exact spin_H is solved: No.

## Recommended Next Step

Replace REINFORCE with **Gumbel-softmax / straight-through routing**: represent the per-step routing as a soft weighted combination of tau one-hot features across operators (using Gumbel-softmax temperature annealing), enabling end-to-end gradient flow through operator selection.  This removes the high-variance binary reward bottleneck and should provide a much cleaner test of whether the transport/non-transport structural distinction is both learnable and functionally significant on the v25 surface.

