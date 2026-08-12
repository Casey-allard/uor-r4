# INC-0050: Dynamic H4 State

## Status
In progress.

## Trigger
The current route still treats time mostly as a scalar schedule parameter. If the geometry itself is evolving, a static shell/sector law may be the wrong mathematical object.

## Hypothesis
The next missing piece may be a dynamic state space, and there are now two live formulations:
1. `H^4` position plus tangent-flow state in `T_xH^4`
2. a coupled product geometry `H^4 x H^4`

The stronger user hypothesis is that the advantage may require the second formulation:
- one `H^4` for position / routing location
- one `H^4` for dynamic flow, memory pressure, or convergent/divergent transport state

That would still give 8 real coordinates, but with hyperbolic structure on both halves instead of one hyperbolic half plus one auxiliary Euclidean half.

## Why This Is Not Plain `R^8`
- `R^8` as a flat replacement would throw away the hyperbolic global-alignment structure that has repeatedly mattered in this project.
- The first stronger formulation is a tangent-bundle / geodesic-flow picture:
  - `x = (r, chi, theta1, theta2)`
  - `u = (p_r, p_chi, p_theta1, p_theta2)`
- The even stronger formulation is a product manifold:
  - `(x, y) in H^4 x H^4`
  - `x` carries the routing location
  - `y` carries dynamic pressure, transport state, or memory-field state
- In other words: 8 real state variables, but potentially with hyperbolic structure on both halves.

## Current Mathematical Preference
- Do not assume plain `R^8`.
- Keep `H^4 x H^4` live as a distinct branch, not just a rewording of tangent flow.
- First formal test should compare:
  1. `H^4` plus tangent flow
  2. `H^4 x H^4`
  3. explain which one better matches:
     - time-evolving geometry
     - divergence/convergence
     - Poincare-ball global alignment
     - Hopf angular routing

## Minimal Scope
1. Keep current systems work separate.
2. Write the formal route law for:
   - `H^4 + T_xH^4`
   - `H^4 x H^4`
3. Build a Slice A surrogate diagnostic on the ordered LM-proxy stream:
   - static `H^4`
   - tangent surrogate `H^4 + T_xH^4`
   - product surrogate `H^4 x H^4`
4. Define what divergence/convergence means in each formulation.
5. Decide whether the second `H^4` should represent:
   - flow / momentum
   - memory pressure
   - imaginary / retrieval field
6. Only then decide whether it needs a route-law code pilot.

## Slice A Artifacts
- Formalism:
  - `docs/research/MATH_REVIEW_DYNAMIC_H4_STATE_20260306.md`
- Learned knowledge:
  - `docs/research/LEARNED_KNOWLEDGE.md`
- Evaluator:
  - `tasks/dynamic_h4_state_eval.py`
- Planned screen:
  - `configs/proxy_transfer_inc0050_dynamic_h4_screen.json`

## Slice A Result
- Screen:
  - `results/analysis/inc0050_dynamic_h4_screen.json`
  - `docs/governance/gates/gate_20260306_122447.md`
- Confirm:
  - `results/analysis/inc0050_dynamic_h4_confirm.json`
  - `docs/governance/gates/gate_20260306_122733.md`

4-seed confirm means:
- `STATIC_H4`: `mse=0.004314443`, `top1=0.02758`, `total=8.569s`
- `TXH4_W050`: `0.004303599`, `0.03200`, `8.458s`
- `H4XH4_W025`: `0.004305430`, `0.03767`, `8.454s`

## Reading
- Dynamic state is real on the ordered LM-proxy stream.
- The tangent surrogate `H^4 + T_xH^4` is the cleaner primary winner on the main MSE objective.
- The product surrogate `H^4 x H^4` is not dead; its clearer signal is top-1 rather than best MSE.
- The intended full product branch should still be understood as hyperbolic polar on both `H^4` factors, not Euclidean 8D polar.

## Next Preferred Work
1. `INC-0054` tangent-flow route law pilot
2. `INC-0055` product `H^4 x H^4` retrieval-field pilot

## Decision Rule
- Promote this branch only if the formalism explains a failure that the current static route cannot explain cleanly.
- Do not let it interrupt the current translated retrieval cost-rescue branch unless the systems rescue fails.
