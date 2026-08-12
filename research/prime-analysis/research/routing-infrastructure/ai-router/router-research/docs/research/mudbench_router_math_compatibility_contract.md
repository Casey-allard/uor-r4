# MUDBench Router Math Compatibility Contract

This note records the frozen helper boundary that MUDBench relies on when it
consumes router math from `ai-router`.

## Scope

The shared boundary is intentionally narrow:

- `fibonacci_values_upto`
- `normalize_4d_coordinate`
- `allocate_pair_bins_scalar`
- `allocate_triplet_bins_budget`
- `hopf_coordinate_components_scalar`
- `hopf_phase_transport_components_scalar`
- `assign_sector_hopf_base_scalar`
- `assign_sector_hopf_transport_scalar`

These helpers provide scalar, deterministic behavior only. They are not a new
runtime routing API and they do not expose broader `hyperbolic_router_so8.py`
surfaces to MUDBench.

## Frozen MUDBench Expectations

The migration must preserve:

- router variant names:
  - `angular-hopf-base`
  - `angular-hopf-trans`
  - `legacy-phase4d_hopf_base`
  - `legacy-phase4d_hopf_transport`
  - `legacy-phase4d_hopf_product_phase`
- prompt-plan keys and prompt headings
- deterministic prompt-plan outputs for fixed payloads
- `coordinate_adapter_contract == "mudbench_prompt_state_v2"` at the
  MUDBench-facing plan boundary

## Non-Goals

This contract does not:

- move prompt assembly out of MUDBench
- change benchmark routing schemas
- introduce the prime-transport `C^2` backbone into runtime routing
- claim a new canonical inference-time router API

## Ownership Split

- `ai-router` owns canonical math helpers.
- MUDBench owns prompt/state adaptation, variant compatibility, and prompt-plan
  serialization.
