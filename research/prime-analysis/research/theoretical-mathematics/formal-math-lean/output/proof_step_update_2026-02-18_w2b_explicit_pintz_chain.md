# Proof Step Update (2026-02-18): Explicit Pintz -> W2b Linear-Phase Chain

## Goal
Make the final-slot dependency chain explicit in Lean so context compaction cannot hide the route from the existing Pintz theorem-term boundary into the W2b slot.

## Changes
- Updated `research/formal/lean/PrimeRiemannBridgeW2bImportedInstance.lean`:
  - Added instance:
    - `importedLinearPhaseOnlyResultsOfPintz2017`
    - type: `[Pintz2017ZeroToOscillationFormalized] -> ImportedLinearPhaseOnlyResults`
  - Added endpoint theorem:
    - `endpoint_to_rh_from_pintz2017_via_w2b_linear_phase_slot`
  - Added RH closure theorem:
    - `rh_from_pintz2017_via_w2b_linear_phase_slot`
- Updated W2b contract artifacts to include these closure entrypoints.

## Verification
- `~/.elan/bin/lake build` passes.
- `python3 research/formal_axiom_audit.py ...` reports `axiom_count = 0`.

## State
- This step does not change the final mathematical blocker.
- Remaining open item is still one concrete imported theorem payload:
  - `ImportedLinearPhaseOnlyResults.linear_phase_of_model_import`
  - equivalently the existing `Pintz2017ZeroToOscillationFormalized.theorem_term` slot.
