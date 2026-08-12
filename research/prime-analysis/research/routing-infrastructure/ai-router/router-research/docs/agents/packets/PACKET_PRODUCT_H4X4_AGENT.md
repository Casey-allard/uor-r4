# Product H4xH4 Agent Packet

## Purpose
Resume or extend the `H^4 x H^4` retrieval-field branch without re-deriving the dynamic-geometry context.

## Must Read
- `docs/research/increments/INC_0050_dynamic_h4_state.md`
- `docs/research/increments/INC_0054_tangent_flow_route_law.md`
- `docs/research/increments/INC_0055_product_h4x4_retrieval_field.md`
- `docs/research/MATH_REVIEW_DYNAMIC_H4_STATE_20260306.md`
- `docs/research/LEARNED_KNOWLEDGE.md`
- `docs/research/CURRENT_DIRECTION.md`
- `docs/research/HANDOFF_CURRENT.md`
- `results/analysis/inc0050_dynamic_h4_confirm.json`
- `results/analysis/inc0054_tangent_flow_route_law_screen.json`

## Current Result
- `TXH4_W050` is still the main-objective dynamic winner.
- `H4XH4_BUCKET_W025` is the product quality/top-1 reference.
- `H4XH4_CPX13_W025` is the product discrete-key efficiency reference.
- `INC-0054` proved that same-bucket Hopf locality is real, but the tangent bucketed branch did not beat the global dynamic baseline on MSE.
- `INC-0055` proved that discrete complex route-key storage on the second `H^4` is efficiency-positive.
- The product branch remains the stronger retrieval/discrete-decision candidate.

## Current Mathematical Reading
- The intended object is `H^4 x H^4` with hyperbolic polar structure on both factors.
- The second `H^4` should not be treated as a duplicate continuous latent unless evidence forces that simplification.
- The new live sub-hypothesis is that route keys may want discrete storage in a complex / imaginary field associated with the second `H^4`.

## Current Code Surface
- evaluator: `tasks/dynamic_h4_state_eval.py`
- sweep support: `tools/proxy_sweep.py`
- summary export: `tools/summarize.py`

## Next Preferred Work
1. Translate the complex-key product branch into the routed retrieval harness.
2. Compare:
   - product same-bucket baseline
   - product same-bucket plus discrete complex route-key storage
3. Prefer candidate-pruning, top-1, bounded fallback, and online/amortized runtime over pure regression MSE alone.

## Do Not Do
- do not flatten this into Euclidean `R^8`
- do not erase the distinction between tangent flow and product geometry
- do not treat complex route-key storage as the whole geometry; it is a routing/storage law on top of the second `H^4`

## Translation Update
- `INC-0056` is now complete and positive.
- The product complex-key law survived translation into `tasks/router_retrieval_eval.py`.
- New translated reference:
  - `HOPF_RET_CPX_P1_Q24`
  - `cand_frac=0.2095`
  - `amortized=0.6129s`
  - `fallback=0.0000`
- Current reading:
  - the complex / imaginary key is not confined to the product-state evaluator
  - the main remaining weakness is recall/top-1, not fallback or candidate explosion
- Next preferred work:
  - hierarchical complex-key backfill to recover top-1 while preserving pruning
