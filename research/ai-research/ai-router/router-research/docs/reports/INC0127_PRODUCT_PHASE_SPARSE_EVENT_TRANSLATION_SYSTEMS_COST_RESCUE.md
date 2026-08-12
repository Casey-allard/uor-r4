# INC-0127 Product Phase Sparse Event Translation Systems Cost Rescue

- Branch outcome: `no_selective_retrieval_rescue_surface`
- Interpretation: The translated near-hard and translated soft-sparse routes differ only in sparse-event audit knobs, and those knobs are not wired into the translated retrieval surface in the current harness. The observed runtime gap is therefore not a selective retrieval-rescue branch.

## Contract Check
- Translation route diff keys: `event_gate_tau`
- Event-gate-only difference: `True`
- Event gate changes translated retrieval surface: `False`
- Reason: Current routed retrieval uses fixed routed train/eval embeddings; event-gate knobs are consumed only by sparse_event_training_audit.

## Observed Translated Deltas (Near-Hard Minus Soft Sparse)
- top-1 delta: `0.000000`
- candidate-fraction delta: `0.000000`
- online delta: `0.098602s`
- amortized delta: `0.117689s`
- route-query delta: `0.011108s`
- route-index-build delta: `0.018890s`
- retrieval-search delta: `0.087495s`

## Decision
- Close `INC-0127` as a rescue-feasibility branch, not as a translated tuning branch.
- Do not reopen candidate recovery or quality rescue from this surface.
- Move next to a route-coupled sparse-event translated pilot if we want downstream sparse-event behavior to matter.
