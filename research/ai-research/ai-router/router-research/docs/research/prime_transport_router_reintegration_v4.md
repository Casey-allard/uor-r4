# Prime Transport — Router Reintegration Experiment v4

**Experiment type:** Token injection into tau state
**Operator layer:** `geometry_native_operator_model_v25`
**No surface build.  No files modified.  No operators rebuilt.**

## What Changed from v3

| Aspect | v3 | v4 |
|--------|----|----|
| Token signal path | Token affects routing weights only | Token injected directly into tau state |
| Tau update | soft_tau = w @ tau_nexts | soft_tau = w @ tau_nexts + W_tok_inject[tok] |
| New parameters | None | W_tok_inject: (VOCAB, D_TAU) = (4, 21) = 84 params |
| Routing mechanism | Gumbel-softmax (same) | Gumbel-softmax (same) |
| Readout | Trajectory attention (same) | Trajectory attention (same) |
| Baselines | No injection | No injection (constraint baselines unchanged) |

**Why token injection tests the correct hypothesis:**
In v1-v3, token x0 could only influence the tau trajectory by changing which routing weights w_t are applied. The geometric tau state itself (which operator transforms are available) was token-blind. With additive injection, soft_tau_t = w_t @ tau_nexts_t + W_tok_inject[tok_t], the token leaves a direct additive trace in the tau state at every step. At step 0 (where tok_t = x0), this trace encodes x0. The attention mechanism (from v3) can now learn to upweight step 0, and the prediction head can recover x0 from the pooled trajectory. This tests whether the operator layer CAN encode token identity when given a direct signal.

## Training and Evaluation Setup

- Task: first-token recall, VOCAB=4, delays D in [2, 4, 8, 16]
- Training: 800 batches x 32 = 25600 episodes/delay
- Evaluation: 1000 episodes per (model, delay)
- Temperature: 2.0 -> 0.1 (exponential annealing)
- Optimizer: SGD + gradient clipping (clip=1.0, lr=0.02)
- Token injection: W_tok_inject (4 x 21 = 84 params), learned, applied at each step
- Constraint baselines: no injection (identical to v3)

## Primary Results

### Accuracy by model and delay

| Model | D=2 | D=4 | D=8 | D=16 | best delta vs chance (0.250) |
|-------|-----|-----|-----|------|--------------------------------------|
| traj_inject_router | 0.615 | 0.436 | 0.328 | 0.280 | +0.365 |
| fixed_Tx | 0.265 | 0.241 | 0.262 | 0.238 | +0.015 |
| random | 0.269 | 0.264 | 0.234 | 0.255 | +0.019 |
| non_transport | 0.239 | 0.278 | 0.242 | 0.217 | +0.028 |

### Direct comparison: v3 traj_router vs v4 traj_inject_router

| D | v3 traj_router | v4 traj_inject_router | delta |
|---|----------------|-----------------------|-------|
| 2 | 0.232 | 0.615 | +0.383 |
| 4 | 0.253 | 0.436 | +0.183 |
| 8 | 0.262 | 0.328 | +0.066 |
| 16 | 0.245 | 0.280 | +0.035 |

v4 traj_inject_router beats v3 traj_router at 4/4 delays.
v4 traj_inject_router exceeds chance (0.250) at 4/4 delays.

## Attention Analysis

### Mean attention weights by step (traj_inject_router only)

Uniform = 1/D. '↑' = > 5% above uniform (model attends to this step more).

| D | step 0 | step 1 | step 2 | step 3 | step 4 | step 5 | step 6 | step 7 | step 8 | step 9 | step 10 | step 11 | step 12 | step 13 | step 14 | step 15 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 2 | 0.500(~) | 0.500(~) | — | — | — | — | — | — | — | — | — | — | — | — | — | — |
| 4 | 0.250(~) | 0.250(~) | 0.250(~) | 0.250(~) | — | — | — | — | — | — | — | — | — | — | — | — |
| 8 | 0.125(~) | 0.125(~) | 0.125(~) | 0.125(~) | 0.125(~) | 0.125(~) | 0.125(~) | 0.125(~) | — | — | — | — | — | — | — | — |
| 16 | 0.063(~) | 0.063(~) | 0.063(~) | 0.063(~) | 0.062(~) | 0.063(~) | 0.063(~) | 0.063(~) | 0.063(~) | 0.062(~) | 0.062(~) | 0.062(~) | 0.062(~) | 0.062(~) | 0.062(~) | 0.062(~) |

Attention weights do not consistently upweight the encoding step (0/4 delays). Attention remains approximately uniform even with token injection.

## Operator Usage

### Transport fraction by model and delay

| Model | D=2 | D=4 | D=8 | D=16 |
|-------|-----|-----|-----|------|
| traj_inject_router | 0.559 | 0.558 | 0.598 | 0.425 |
| fixed_Tx | 0.000 | 0.000 | 0.000 | 0.000 |
| random | 0.000 | 0.000 | 0.000 | 0.000 |
| non_transport | 0.000 | 0.000 | 0.000 | 0.000 |

### traj_inject_router: per-operator usage fraction

| Operator | Cluster | D=2 | D=4 | D=8 | D=16 |
|----------|---------|-----|-----|-----|------|
| T_b | non_transport | 0.008 | 0.207 | 0.138 | 0.182 |
| T_x | non_transport | 0.307 | 0.234 | 0.019 | 0.042 |
| T_y | non_transport | 0.126 | 0.002 | 0.245 | 0.350 |
| T_c | transport | 0.000 | 0.467 | 0.597 | 0.414 |
| T_z' | transport | 0.076 | 0.090 | 0.002 | 0.011 |
| T_r* | transport | 0.483 | 0.002 | 0.000 | 0.000 |

### Routing collapse check

Max entropy (uniform over 6 ops): 1.792 bits

| D | entropy | collapsed (<0.3)? | mixed (>60% max)? |
|---|---------|-------------------|-------------------|
| 2 | 1.634 | no | yes |
| 4 | 1.694 | no | yes |
| 8 | 1.685 | no | yes |
| 16 | 1.731 | no | yes |

## Transport / Non-Transport Distinction

| D | transport @ step 0 (encoding) | transport @ steps 1+ (maint.) | diff |
|---|-------------------------------|-------------------------------|------|
| 2 | 0.501 | 0.618 | -0.117 |
| 4 | 0.608 | 0.541 | +0.067 |
| 8 | 0.494 | 0.613 | -0.119 |
| 16 | 0.454 | 0.423 | +0.031 |

## Honesty Section

### What improved

- v4 traj_inject_router beats v3 traj_router at 4/4 delays (direct injection provides at least marginal improvement).
- Best accuracy: 0.615 vs chance 0.250 (+0.365 delta).
- Token injection gradient flows correctly through the soft tau chain.
- No NaN or training instability.

### What failed or is limited

- Best accuracy: 0.615. Improvement over v3 but still far below ceiling.

### What remains uncertain

- Whether more training (larger N_BATCHES) would allow the injection signal to dominate over routing noise.
- Whether the task first-token recall is actually learnable with this architecture at all delays, or whether D=8,16 are fundamentally beyond the capacity of an 800-batch SGD run with 32-per-batch.
- Whether full exact spin_H is solved: No.

## Recommended Next Step

Token injection produced usable above-chance signal. Run the same experiment with **5,000 training batches** (rather than 800) to determine whether the signal scales with training budget and converges toward a reliable above-chance ceiling.

