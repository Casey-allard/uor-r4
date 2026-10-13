# Prime Transport Router v6 Torch — Context-Length Scaling

**Status:** Complete

## Setup

- Script: `run_v6_torch_context_scaling.py` (wraps `run_router_reintegration_v6_torch.py`)
- Delays tested: [16, 24, 32, 48]
- Budget: 10000 batches each
- Model: D_HIDDEN=32, B=256, torch.jit.script backend — unchanged from v6_torch
- Injection: v6 step-0 additive only — unchanged

## Results

| D   | Accuracy | vs Chance | Route H | Transport | alpha0 | Verdict |
|-----|----------|-----------|---------|-----------|--------|---------|
|  16 | 1.000    | 16.00×      | 1.574   | 0.321      | 0.594  | solved |
|  24 | 0.503    | 12.07×      | 1.683   | 0.117      | 0.042  | learning_not_solved |
|  32 | 0.408    | 13.06×      | 1.683   | 0.991      | 0.031  | learning_not_solved |
|  48 | 0.275    | 13.20×      | 1.743   | 0.011      | 0.021  | learning_not_solved |

## Analysis

**First degradation:** D=24
**First near-chance failure:** none — no near-chance failure observed

### Attention alpha0

At D=16 (solved): alpha0 = 0.594 — strongly concentrated on the first timestep.
The model learned to attend to the position where the token injection fires.

At D=24, 32, 48 (not solved): alpha0 = 0.042, 0.031, 0.021 — these are exactly 1/D
(uniform attention). The attention layer assigns equal weight to all timesteps and
never concentrates on position 0. This is a clean signature: the gradient signal from
the loss cannot propagate back through D BPTT steps to teach the attention to focus
on the injection position. Uniform alpha0 at unsolved delays is not "partial learning"
— it is structural non-convergence of the attention readout.

### Transport usage across context lengths

- D=16: transport_fraction=0.321
- D=24: transport_fraction=0.117
- D=32: transport_fraction=0.991
- D=48: transport_fraction=0.011

### Failure mode diagnosis

No delay reaches near-chance accuracy — vs_chance stays at 12–16× across all D.
However, two distinct regimes are visible:

- **D=16 (solved):** alpha0 = 0.594 (concentrated), H = 1.574 (converging), transport = 0.321.
  The model found a structured routing policy and learned to attend to the injection position.

- **D=24–48 (not solved):** alpha0 ≈ 1/D (exactly uniform), H = 1.683–1.743 (near ln(6)=1.792,
  i.e., near-random routing). The attention readout never concentrates. vs_chance is
  roughly constant at 12–13× regardless of D — suggesting the model exploits some
  information in the operator state trajectory, but the injection signal at t=0 is not
  being retrieved by the attention.

The constant ~12–13× vs_chance for D=24, 32, 48 (despite D growing) suggests the router
is picking up structural information from the operator state machine independent of D,
rather than routing through the step-0 injection path. The tau injection signal is not
being propagated to the attention readout.

Primary candidates for this failure:
1. **BPTT gradient dilution**: the gradient of the loss w.r.t. W_tok_inject at t=0
   is multiplied by D Jacobians of tanh activations. For D≥24, this signal is too weak
   to drive alpha0 concentration. This is the most likely cause.
2. **Training budget**: route entropy remains high (1.683+) at all failing delays,
   consistent with the optimizer not having converged. More budget may help D=24
   if the BPTT signal is weak but nonzero.
Both are present. Budget extension is the faster test to determine which dominates.

## Honesty

- **What worked:** D=16
- **What degraded (learning, not solved):** D=24, D=32, D=48
- **What near-chance failed:** none
- **Uncertain:** Whether degraded delays would solve with more budget or hit a hard representation ceiling.
- No files modified. No operators rebuilt. D_HIDDEN unchanged at 32.

Total wall time: 578.6s

## Next Step

**Extend training budget for D=24.** First degradation at D=24 (acc=0.503) with high route entropy (1.683), suggesting it is budget-limited. Run at 20,000–30,000 batches before concluding it is a hard ceiling.
