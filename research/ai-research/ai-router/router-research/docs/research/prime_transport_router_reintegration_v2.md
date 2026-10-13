# Prime Transport — Router Reintegration Experiment v2

**Experiment type:** Differentiable Gumbel-softmax router reintegration
**Operator layer:** `geometry_native_operator_model_v25`
**No surface build.  No files modified.  No operators rebuilt.**

## What Changed from v1

| Aspect | v1 | v2 |
|--------|----|----|
| Routing mechanism | REINFORCE (discrete, high-variance) | Gumbel-softmax (differentiable) |
| Gradient flow | Binary reward only | End-to-end through soft tau chain |
| Training | 4,000 single episodes per delay | 800 batches x 32 = 25,600 episodes |
| Tau signal to prediction | Hard operator tau | Soft tau mix (w-weighted average) |
| Temperature | N/A | Annealed 2.0 -> 0.1 over training |

**Why Gumbel-softmax is the right fix for v1:**
In v1, REINFORCE updates the router only via a binary correct/incorrect reward at the end of each episode. With D steps and VOCAB=4, the variance is too high for reliable learning. Gumbel-softmax creates a continuous relaxation of operator selection. The soft tau state (w-weighted average of all 6 next-tau candidates) flows directly into the prediction head, and gradients flow backward through the soft tau chain -> routing weights -> router MLP -> token embedding. Every routing decision gets a dense, low-variance gradient signal on every episode.

## Training and Evaluation Setup

- Task: first-token recall, VOCAB=4, delays D in [2, 4, 8, 16]
- Training: 800 batches x 32 = 25600 episodes per delay
- Evaluation: 1000 episodes per (model, delay)
- Temperature annealing: 2.0 -> 0.1 (exponential schedule)
- Optimizer: SGD with gradient clipping (clip=1.0, lr=0.02)

## Primary Results

### Accuracy by model and delay

| Model | D=2 | D=4 | D=8 | D=16 | best delta vs chance (0.250) |
|-------|-----|-----|-----|------|--------------------------------------|
| gumbel_router | 0.234 | 0.233 | 0.254 | 0.247 | +0.004 |
| fixed_Tx | 0.240 | 0.272 | 0.245 | 0.252 | +0.022 |
| random | 0.245 | 0.240 | 0.247 | 0.259 | +0.009 |
| non_transport | 0.227 | 0.242 | 0.228 | 0.264 | +0.014 |

### Direct comparison: v1 learned_router vs v2 gumbel_router

| D | v1 learned_router | v2 gumbel_router | delta |
|---|-------------------|------------------|-------|
| 2 | 0.267 | 0.234 | -0.033 |
| 4 | 0.249 | 0.233 | -0.016 |
| 8 | 0.260 | 0.254 | -0.006 |
| 16 | 0.242 | 0.247 | +0.005 |

## Operator Usage

### Transport fraction by model and delay

| Model | D=2 | D=4 | D=8 | D=16 |
|-------|-----|-----|-----|------|
| gumbel_router | 0.398 | 0.625 | 0.347 | 0.114 |
| fixed_Tx | 0.000 | 0.000 | 0.000 | 0.000 |
| random | 0.000 | 0.000 | 0.000 | 0.000 |
| non_transport | 0.000 | 0.000 | 0.000 | 0.000 |

### Gumbel router: per-operator usage fraction

| Operator | Cluster | D=2 | D=4 | D=8 | D=16 |
|----------|---------|-----|-----|-----|------|
| T_b | non_transport | 0.007 | 0.107 | 0.013 | 0.869 |
| T_x | non_transport | 0.429 | 0.042 | 0.160 | 0.005 |
| T_y | non_transport | 0.165 | 0.225 | 0.479 | 0.012 |
| T_c | transport | 0.003 | 0.089 | 0.133 | 0.034 |
| T_z' | transport | 0.046 | 0.139 | 0.009 | 0.075 |
| T_r* | transport | 0.349 | 0.397 | 0.205 | 0.004 |

### Routing collapse check

Max entropy (uniform over 6 ops): 1.792 bits

| D | entropy | collapsed (<0.3)? | mixed (>60% max)? |
|---|---------|-------------------|-------------------|
| 2 | 1.664 | no | yes |
| 4 | 1.718 | no | yes |
| 8 | 1.747 | no | yes |
| 16 | 1.712 | no | yes |

## Transport / Non-Transport Distinction Discovery

| D | transport @ step 0 (encoding) | transport @ steps 1+ (maintenance) | diff |
|---|-------------------------------|-------------------------------------|------|
| 2 | 0.385 | 0.411 | -0.026 |
| 4 | 0.722 | 0.593 | +0.129 |
| 8 | 0.409 | 0.338 | +0.071 |
| 16 | 0.317 | 0.100 | +0.217 |

**Router uses more transport at encoding step** (0.458) than during maintenance (0.361), consistent with design hypothesis.

Transport fraction across delays: D=2:0.398, D=4:0.625, D=8:0.347, D=16:0.114. Varies with delay.

### Route entropy (operator diversity per step)

| Model | D=2 | D=4 | D=8 | D=16 |
|-------|-----|-----|-----|------|
| gumbel_router | 1.664 | 1.718 | 1.747 | 1.712 |
| fixed_Tx | 0.000 | 0.000 | 0.000 | 0.000 |
| random | 1.792 | 1.792 | 1.792 | 1.792 |
| non_transport | 1.099 | 1.099 | 1.099 | 1.099 |

## Honesty Section

### What improved

- Full end-to-end gradient flow works: no crashes, no NaN across all delays.
- Gumbel router beats v1 learned_router at 1/4 delays.
- Gumbel router exceeds chance (0.250) at 1/4 delays.
- Training signal is dense and low-variance vs v1 REINFORCE.

### What failed or is limited

- Accuracy remains near chance: best 0.254 (chance: 0.250).
- Root cause: the operator layer transforms tau based on geometric state, not on the token being processed. Each operator maps the current tau state to a new tau state deterministically. Two tokens that land in the same warmup state will produce the same tau trajectory regardless of which operator is selected. The token only influences WHICH operator is chosen, not HOW the operator transforms the state -- so the token-identity signal must be encoded in the operator-selection pattern across steps, which is an extremely weak signal for a linear prediction head.
- The prediction head reads only the FINAL tau state. If the encoding is in the trajectory shape rather than the terminal state, a linear readout from the final step cannot recover it.

### What remains uncertain

- Whether trajectory-level readout (attention over all D soft taus) would substantially improve performance.
- Whether the tau state carries any token-identity signal at all on the bounded v25 surface, or whether the operators are too geometric and state-local for arbitrary token encoding.
- Whether full exact spin_H is solved: No.

## Recommended Next Step

Redesign the readout architecture: replace the final-state linear prediction head with an **attention-weighted pooling over all D soft tau states** (a learned attention score computed from each step's soft tau). This tests whether the token-identity signal is distributed across the full tau trajectory rather than concentrated in the terminal state -- and directly addresses the hypothesis that the operator layer encodes information in the *path* through tau space rather than in the endpoint.

