# Proof Step Update (2026-02-18): Ingham Boundary Wiring

## Goal
Pin the last import boundary to a concrete classical theorem interface used in analytic number theory.

## Changes
- Updated `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeConcretePackInstantiation.lean`:
  - Added class `Ingham1932ZeroToOmegaFormalized` with source locks and theorem payload matching the existing zero-to-lower-envelope signature.
  - Added instance `pintz2017OfIngham1932` to reuse the existing Pintz/W2b closure chain.
  - Added helper constructor `ingham1932FormalizedOfTerm` to convert a raw theorem term into the class instance directly.
  - Added theorem `rh_from_ingham1932_formalized`.
  - Added theorem `rh_from_ingham1932_term` for one-step closure from the raw theorem payload.
- Updated `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeW2bImportedInstance.lean`:
  - Added theorem `rh_from_ingham1932_via_w2b_linear_phase_slot`.

## Verification
- `~/.elan/bin/lake build PrimeRiemannBridgeConcretePackInstantiation` passed.
- `~/.elan/bin/lake build PrimeRiemannBridgeW2bImportedInstance` passed.
- `~/.elan/bin/lake build` passed.
- Formal audit: `axiom_count = 0`.

## State
- Final blocker remains exactly one imported theorem-term instance.
- The preferred final payload is now explicit:
  - instantiate `Ingham1932ZeroToOmegaFormalized.theorem_term`.
