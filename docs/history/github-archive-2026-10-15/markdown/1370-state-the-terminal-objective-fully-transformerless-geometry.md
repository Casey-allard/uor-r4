# #1370 — State the terminal objective: fully transformerless, geometry replacing float matmul; register the Hamiltonian/vectors lead (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-23T22:03:40Z
- **Updated:** 2026-10-09T13:17:45Z
- **Closed:** 2026-09-23T22:04:48Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1370
- **Merged:** 2026-09-23T22:04:48Z
- **Merge commit:** d5c83bf94d17b35d22b4ce61fccd84399577cc4f
- **Base / Head:** main / UOR-Foundation:codex/objective-transformerless-20260923
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this changes

Docs-only. Records **owner direction (2026-09-23)** in the project objective: the terminal goal is a **fully
transformerless geometric language model in which geometry *replaces* floating-point matrix multiplication**, not
merely a table-native output layer. No code, no model run, no artifact, no compute.

## Edits

- `docs/integration/project-track.md` (canonical plan, **Goal**): adds the explicit terminal-objective paragraph;
  registers the owner-supplied **"vectors and Hamiltonians"** lead as a *hypothesis to test* (mapping to the
  existing quaternionic substrate — `H4` = Hamilton's quaternions, 2I/icosian `Z[phi]`, `S3`/Hopf,
  spin/chirality), not an adopted mechanism.
- `README.md` (Goal and serving contract): states the same terminal goal publicly.
- `docs/integration/agent-execution-policy.md` (Architecture and scope): states that the serving contract is the
  intended **end-state**, not only a boundary on the current prototype.

## No conflict with existing policy

The rule was already the stable machine contract (`agent-execution-policy.json`):
`serving_kernel_multiplier_instructions_allowed=false`, `serving_kernel_floating_point_allowed=false`,
`serving_transcendentals_allowed=false`, `serving_weight_max_bits=4`,
`dense_transformer_disguised_as_lookup_allowed=false`, and training may use float/matmul while final inference is
`learned_geometric_operators_with_bounded_routing_state_transitions_and_integer_table_lookup`. This change makes
the **goal framing** explicit; it does not alter any numeric or machine contract. The JSON was deliberately left
untouched to avoid changing a machine-readable contract.

`python3 scripts/check_claim_wording.py` passes. References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

