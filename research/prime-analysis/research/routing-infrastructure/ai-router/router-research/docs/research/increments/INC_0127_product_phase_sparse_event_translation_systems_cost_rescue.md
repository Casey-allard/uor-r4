# INC-0127: Product Phase Sparse Event Translation Systems Cost Rescue

## Status
Completed negative/explanatory on 2026-03-12.

## Trigger
`INC-0126` showed that the translated near-hard failure is not a quality or
candidate-set problem:
- near-hard translated top-1 matches the translated soft sparse and continuous
  references
- candidate fraction also matches exactly
- the failure is dominated by extra translated systems cost, especially
  retrieval search plus route-index build

That made a narrow translated systems-cost rescue look like the next honest
branch, but only if the sparse-event knobs were actually wired into the
translated retrieval surface.

## Evidence
- Rescue-feasibility audit tool:
  `tools/sparse_event_translation_systems_cost_rescue.py`
- Rescue-feasibility analysis:
  `results/analysis/inc0127_product_phase_sparse_event_translation_systems_cost_rescue.json`
- Report:
  `docs/reports/INC0127_PRODUCT_PHASE_SPARSE_EVENT_TRANSLATION_SYSTEMS_COST_RESCUE.md`
- Guardrail code/tests:
  - `tasks/router_retrieval_eval.py`
  - `tests/test_router_retrieval_eval.py`
  - `tests/test_sparse_event_translation_systems_cost_rescue.py`

## Reading
- The compared translated routes differ only on one knob:
  - `event_gate_tau`
- In the current translated harness, sparse-event knobs are audit-only:
  - `event_gate_retrieval_surface_active = 0`
  - translated retrieval still uses fixed routed train/eval embeddings
- That means the observed translated deltas do not define a selective
  near-hard rescue surface:
  - top-1 delta = `0.000000`
  - candidate-fraction delta = `0.000000`
  - online delta = `+0.098602s`
  - amortized delta = `+0.117689s`
  - retrieval-search delta = `+0.087495s`
- So the branch closes the reopen rather than rescuing it:
  - the translated near-hard gap is not a route-law or candidate-law problem
  - the current harness simply does not let sparse-event settings change
    translated retrieval behavior downstream

## Decision
- Close `INC-0127` negative/explanatory.
- Do not keep queuing translated systems-cost rescue on this audit-only
  surface.
- Keep the hardened near-hard proxy point as valid.
- Treat the translated sparse-event reopen as unresolved until sparse-event
  behavior is coupled into the translated route or retrieval surface itself.

## Resume Note
Resume from the completed `INC-0127` rescue-feasibility audit, not from the
old downstream packet loop and not from the old translated cost-rescue plan.
The next honest branch is route-coupled sparse-event translated behavior.
