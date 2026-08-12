# Prime Transport — Router Reintegration Experiment v3

**Experiment type:** Trajectory-level attention readout (Gumbel-softmax routing)
**Operator layer:** `geometry_native_operator_model_v25`
**No surface build.  No files modified.  No operators rebuilt.**

## What Changed from v2

| Aspect | v2 | v3 |
|--------|----|----|
| Readout | Terminal-state linear head (W_pred @ soft_tau_{D-1}) | Attention-weighted pooling over all D soft tau states |
| Attention | None | MLP: h_t=tanh(W_attn@st_t+b_attn), a_t=v@h_t, alpha=softmax(a) |
| Gradient to routing | From terminal state only | From all D tau states |
| Routing mechanism | Gumbel-softmax (same) | Gumbel-softmax (same) |
| Task / delays | Same | Same |
| Training budget | Same | Same |

**Why trajectory-level readout is the right fix for v2:**
In v2, all gradient signal for routing decisions at steps 0..D-2 flowed only through the FINAL soft tau state. If the token-identity signal is distributed across the tau trajectory (path-encoded), the terminal readout cannot recover it, and the gradients for early routing decisions are vanishingly small. The attention readout assigns a learned weight alpha_t to each step's soft tau. If the encoding step (step 0) carries discriminative signal about x0, the model can learn alpha_0 >> alpha_{D-1}, directly recovering the early-step encoding.

## Training and Evaluation Setup

- Task: first-token recall, VOCAB=4, delays D in [2, 4, 8, 16]
- Training: 800 batches x 32 = 25600 episodes/delay
- Evaluation: 1000 episodes per (model, delay)
- Temperature: 2.0 -> 0.1 (exponential annealing)
- Optimizer: SGD + gradient clipping (clip=1.0, lr=0.02)
- Attention: D_HIDDEN_ATTN=8; position-free (tau state only)

## Primary Results

### Accuracy by model and delay

| Model | D=2 | D=4 | D=8 | D=16 | best delta vs chance (0.250) |
|-------|-----|-----|-----|------|--------------------------------------|
| traj_router | 0.232 | 0.253 | 0.262 | 0.245 | +0.012 |
| fixed_Tx | 0.264 | 0.253 | 0.244 | 0.255 | +0.014 |
| random | 0.220 | 0.236 | 0.263 | 0.247 | +0.013 |
| non_transport | 0.262 | 0.251 | 0.248 | 0.258 | +0.012 |

### Direct comparison: v2 gumbel_router vs v3 traj_router

| D | v2 gumbel_router | v3 traj_router | delta |
|---|------------------|----------------|-------|
| 2 | 0.234 | 0.232 | -0.002 |
| 4 | 0.233 | 0.253 | +0.020 |
| 8 | 0.254 | 0.262 | +0.008 |
| 16 | 0.247 | 0.245 | -0.002 |

v3 traj_router beats v2 gumbel_router at 2/4 delays.
v3 traj_router exceeds chance at 2/4 delays.

## Attention Analysis

### Mean attention weights by step position (traj_router only)

Uniform attention = 1/D.  Values > uniform indicate the model attends to that step.

| D | step 0 | step 1 | step 2 | step 3 | step 4 | step 5 | step 6 | step 7 | step 8 | step 9 | step 10 | step 11 | step 12 | step 13 | step 14 | step 15 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 2 | 0.500(~) | 0.500(~) | — | — | — | — | — | — | — | — | — | — | — | — | — | — |
| 4 | 0.250(~) | 0.250(~) | 0.250(~) | 0.250(~) | — | — | — | — | — | — | — | — | — | — | — | — |
| 8 | 0.125(~) | 0.125(~) | 0.125(~) | 0.125(~) | 0.125(~) | 0.125(~) | 0.125(~) | 0.125(~) | — | — | — | — | — | — | — | — |
| 16 | 0.062(~) | 0.062(~) | 0.063(~) | 0.062(~) | 0.063(~) | 0.063(~) | 0.063(~) | 0.063(~) | 0.062(~) | 0.062(~) | 0.063(~) | 0.062(~) | 0.063(~) | 0.062(~) | 0.063(~) | 0.063(~) |

Attention weights do not consistently upweight the encoding step (0/4 delays). Attention appears nearly uniform or does not favour step 0.

## Operator Usage

### Transport fraction by model and delay

| Model | D=2 | D=4 | D=8 | D=16 |
|-------|-----|-----|-----|------|
| traj_router | 0.450 | 0.168 | 0.044 | 0.025 |
| fixed_Tx | 0.000 | 0.000 | 0.000 | 0.000 |
| random | 0.000 | 0.000 | 0.000 | 0.000 |
| non_transport | 0.000 | 0.000 | 0.000 | 0.000 |

### traj_router: per-operator usage fraction

| Operator | Cluster | D=2 | D=4 | D=8 | D=16 |
|----------|---------|-----|-----|-----|------|
| T_b | non_transport | 0.009 | 0.171 | 0.271 | 0.862 |
| T_x | non_transport | 0.374 | 0.409 | 0.000 | 0.113 |
| T_y | non_transport | 0.168 | 0.252 | 0.685 | 0.000 |
| T_c | transport | 0.007 | 0.077 | 0.000 | 0.023 |
| T_z' | transport | 0.064 | 0.006 | 0.003 | 0.000 |
| T_r* | transport | 0.379 | 0.085 | 0.041 | 0.002 |

### Routing collapse check

Max entropy (uniform over 6 ops): 1.792 bits

| D | entropy | collapsed (<0.3)? | mixed (>60% max)? |
|---|---------|-------------------|-------------------|
| 2 | 1.671 | no | yes |
| 4 | 1.740 | no | yes |
| 8 | 1.686 | no | yes |
| 16 | 1.710 | no | yes |

## Transport / Non-Transport Distinction

| D | transport @ step 0 (encoding) | transport @ steps 1+ (maint.) | diff |
|---|-------------------------------|-------------------------------|------|
| 2 | 0.408 | 0.491 | -0.083 |
| 4 | 0.247 | 0.142 | +0.105 |
| 8 | 0.131 | 0.032 | +0.099 |
| 16 | 0.147 | 0.017 | +0.130 |

**Router uses more transport at the encoding step** (0.233) than during maintenance (0.170), consistent with design hypothesis.

## Honesty Section

### What improved

- Trajectory readout gradient flows to all D routing steps, not just the final one.
- The model can now in principle detect path-encoded signal in early tau states.
- Routing remained mixed (entropy > 60% max) at 4/4 delays.
- v3 traj_router beats v2 gumbel_router at 2/4 delays.

### What failed or is limited

- Accuracy remained near chance (best 0.262, chance 0.250).
- The trajectory readout and attention mechanism did not unlock significant task performance improvement.
- This suggests that the tau trajectory may not carry discriminative token-identity information under the current first-token recall task setup.
- The operator layer applies deterministic geometric transformations to a shared state. Two different tokens x0=0 and x0=1, starting from the same warmup state, can only be distinguished by the routing weights w_0. However, the soft tau state soft_tau_0 = w_0 @ tau_nexts_0 may not reliably encode which token was seen, because tau_nexts_0 is fixed (depends only on hard state) and w_0 variation across tokens may be too small relative to noise.

### What remains uncertain

- Whether the fundamental limitation is insufficient token-to-tau pathway capacity, or insufficient training (800 batches may be too few).
- Whether a modified task where the token directly gates operator application (rather than routing being the only token-sensitive element) would succeed.
- Whether full exact spin_H is solved: No.

## Recommended Next Step

Modify the integration interface: instead of routing operator selection solely on the token embedding, **concatenate the token directly into the operator state transformation** (inject x0 as a bias into the tau update at each step, bypassing the routing bottleneck). This tests whether the operator layer can encode token identity when given a direct token-injection signal, rather than relying entirely on the routing MLP to differentiate tokens through soft operator selection from a shared geometric state.

