# State-tracking experiment report (agent: statetrack)

The subagent could not write this file, so the lead saved it from its hand-back message. Code and raw data are in exp/statetrack/.

## Setup
- **Tasks.** Word problems where the target at every position is the running product.
  - A5: 60 classes, 3 generators.
  - Z60: abelian control.
  - S5: 120 classes.
- **Architecture shared by all models.** Real state D=32, with the same RMS-normalized MLP readout (32→64→classes).
- **Models.**
  - quat: 8 free unit-quaternion lanes, q = normalize(raw[x]).
  - quat_r: the repo form, q = normalize(e0 + 0.1·raw).
  - hh: H(v)H(e0), the repo control.
  - hh_r: hh at the repo's 0.1/√2 scale.
  - dprod: 4 generalized Householders per step (DeltaProduct-like).
  - gru: 32 units.
  - diag (0,1) and diag (−1,1): diagonal gates.
  - cplx_u: 16 unit-complex lanes, the commutative twin of quat.
  - lru: LRU-style recurrence.
- **Training.** Curriculum 2→4→8→16→32, batch 64, 1,000 Adam steps. The learning rate was chosen per model from {3e-3, 1e-2, 3e-2}.
- **Evaluation.** The same 512 sequences for every model, at lengths up to 512. Accuracy is averaged over positions (L/2, L].

## A5 results (chance 0.017)
| Model | Runs | Length 32 | Length 512 | Exact at 512 |
|---|---|---|---|---|
| quat, free (128 recurrent params) | 5 seeds | 0.62±0.47 | 0.61±0.48 | 3/5 (5/7 including the learning-rate sweep); bimodal |
| quat_r, repo form | 3 learning rates + 3,000 steps | ≤0.016 | ≤0.017 | 0/4 |
| hh = v·x·v, free | 3 seeds | 0.97±0.04 | 0.71±0.20 | 0/3 (best 0.987) |
| hh_r, repo scale | 3 learning rates | ≤0.015 | ≤0.017 | 0/3 |
| dprod | 3 seeds | 1.00 | 0.017 (0.53 at length 128) | 0/5 |
| gru (3,488 params) | 3 learning rates + 3,000 steps | ≤0.024 | ≤0.017 | 0/4 |
| diag / diag± / cplx_u / lru | 3 learning rates each | ≤0.020 | ≤0.017 | 0/12 |

## Exact serving (snap to the 600-cell, then a 120-state integer automaton: state = T[state, tok], class = C[state])
- **A5 quat, successful runs.**
  - Table-served: 4/5 runs are 1.000 at every length up to 4,096; the fifth is 0.984 at all lengths.
  - Float32 originals of two runs drift to 0.737 and 0.620 at lengths 2,048–4,096.
- **Z60 cplx_u.** Snapped to exact characters: 1.000, 0.967 and 0.749 across seeds. The float models are near 0 by length 128.
- **A5 hh.** Not compressible: 14,400–28,800 states.

## Other findings
- **The repo control is a quaternion map.** H(v)H(e0)x = v·x·v (verified to 1.3e-15), so it is non-commutative.
- **S5 cannot be tracked by rotation-only 4-D lanes** (proof sketch in the agent message). quat reaches 0.016 at length 512, i.e. parity-only. dprod fits in distribution but collapses at 4–16×.
- **Z[φ] kernel.** A 2I rotation costs 24 integer add/sub plus 8 shifts, and 0 for the 8 axis elements. Coefficients stay bounded (max |coef| = 6 over 20,000 steps).
- **The repo form blocks learning.** α = 0.1 near identity is effectively commutative to first order; the A5 generators need |raw| ≈ 10.
- **Learnability is fragile.** Only 1–2 of 8 lanes converge. A full 60-letter alphabet was not learned within 1,500 steps by quat or GRU in this setup. The math agent's setup did learn it with 4 lanes in 2,000 steps.
- **Language implication (hypothesis).** This is a capability niche (code, variable and entity tracking), not a perplexity fix. Probe "role confusion" before building.

## Recommendations
- Add an A5 probe to the D8 ladder, with a commutative arm.
- Put a subset of lanes in group-action form: token-conditioned, α ≥ 1, pure transport.
- Snap converged lanes.
- Build a Krohn–Rhodes-motivated lane mixture: 2I lanes, C_N phase lanes, and reset/decay lanes, plus reflection lanes for S_n.
- Use PaTH attention with exact 2I frames.
- Diagnose role confusion first.

## Compute
About 35 CPU-minutes in total.
