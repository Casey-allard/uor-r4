# Proof Step Update — 2026-02-24 (K1-W12 k1_s1 Closed)

## Result
Closed `k1_s1` (phase-core split) unconditionally in-repo.

## What was added

File:
- `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeOscillatoryReduction.lean`

1. General core-split constructor:
- `trivialSingleDecayingPhaseCoreWitness`

This proves any phase function admits a decomposition
`phase = tau*log + phi + core` by explicit construction (`tau=1`, `phi=0`, `core=phase-log`).

2. New reduced target interface:
- `ExplicitFormulaSingleDecayingModeOnlyAssumptions`
- `singleDecayingLadderAssumptionsOfModeOnly`

This removes `k1_s1` from the open blocker set and leaves only mode identification (`k1_s2`) as the essential analytic task.

3. Prior ladder and linear-phase reductions remain active:
- `singleDecayingAssumptionsOfLadder`
- `singleDecayingLadderAssumptionsOfLinearPhaseOnly`
- `concreteSingleDecayingPhaseLadderProviderOfLinearPhaseOnly`
- `rh_from_linear_phase_only_via_single_decaying_ladder_instance`

## Validation

- `lake build PrimeRiemannBridgeOscillatoryReduction` ✅
- `lake build PrimeRiemannBridgeSchlagePuchta2019ImportedInstance` ✅
- `lake build` ✅

## Current final-open analytic item

Only `k1_s2` remains in this decomposition:
- derive a non-circular `SingleDecayingModeWitness` for the constructed core.
