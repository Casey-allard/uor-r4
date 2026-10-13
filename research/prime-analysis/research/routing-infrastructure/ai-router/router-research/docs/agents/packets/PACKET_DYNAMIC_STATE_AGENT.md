# Dynamic State Agent Packet

## Purpose
Resume or extend the dynamic geometry branch without re-deriving the project context.

## Must Read
- `docs/research/increments/INC_0050_dynamic_h4_state.md`
- `docs/research/MATH_REVIEW_DYNAMIC_H4_STATE_20260306.md`
- `docs/research/LEARNED_KNOWLEDGE.md`
- `docs/research/CURRENT_DIRECTION.md`
- `docs/research/HANDOFF_CURRENT.md`
- `results/analysis/inc0050_dynamic_h4_screen.json`
- `results/analysis/inc0050_dynamic_h4_confirm.json`

## Current Result
- `TXH4_W050` is the current dynamic winner on proxy MSE and runtime.
- `H4XH4_W025` is still live because it improved top-1 over static `H^4`.
- Neither branch has been turned into a route-law or translated retrieval branch yet.

## Current Code Surface
- evaluator: `tasks/dynamic_h4_state_eval.py`
- sweep support: `tools/proxy_sweep.py`
- summary export: `tools/summarize.py`

## Next Preferred Work
1. `INC-0054` tangent-flow route law pilot.
2. `INC-0055` product `H^4 x H^4` retrieval-field pilot.

## Do Not Do
- do not flatten the branch into plain `R^8`
- do not claim end-to-end model improvement from Slice A alone
- do not erase the distinction between MSE improvement and top-1 improvement
