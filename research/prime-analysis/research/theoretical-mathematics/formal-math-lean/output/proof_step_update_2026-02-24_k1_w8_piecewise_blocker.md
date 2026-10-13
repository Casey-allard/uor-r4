# Proof Step Update (2026-02-24): K1-W8 Piecewise Blocker Decomposition

## Objective

Break the final blocker into independently derivable mathematical pieces.

## New formal decomposition (implemented)

File:
- `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeOscillatoryReduction.lean`

Added piecewise assumptions structure:
- `ExplicitFormulaFiniteModeResidualMajorantPiecesAssumptions` (`:538`)

This splits the blocker into:
1. `phase_split_of_model`:
- derive `phase = tau*log + phi + finite_modes + residual`.
2. `residual_majorant_of_phase_split`:
- prove `|residual(x)| <= C*x^{-eta}` with `eta > 0`.
3. `zero_to_global_decomposition`:
- existing decomposition requirement (already present in chain).

Compositional closure added:
- `finiteModeResidualMajorantAssumptionsOfPieces` (`:582`)
- `ConcreteFiniteModeResidualMajorantPiecesProvider` (`:1363`)
- `concreteFiniteModeResidualMajorantProviderOfPieces` (`:1384`)
- `rh_from_finite_mode_residual_majorant_pieces_instance` (`:1473`)

Final-lock bridge for piecewise provider:
- `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeSchlagePuchta2019ImportedInstance.lean`
- `nonempty_concrete_finite_mode_residual_majorant_provider_of_nonempty_concrete_finite_mode_residual_majorant_pieces_provider`
- `rh_of_nonempty_concrete_finite_mode_residual_majorant_pieces_provider`

## Equivalence lock retained

- `rh_iff_nonempty_concrete_finite_mode_residual_majorant_provider` (RH <-> blocker)
- `nonempty_concrete_finite_mode_residual_majorant_provider_iff_nonempty_pintz2017_zero_to_oscillation_formalized`

So there are no hidden extra stages beyond these explicit piecewise obligations.

## Build status

- `lake build PrimeRiemannBridgeOscillatoryReduction PrimeRiemannBridgeSchlagePuchta2019ImportedInstance` passed.
- `lake build` passed.

## Exact remaining derivation work

To close internally without imports, we must non-circularly prove the two open piece fields:

1. `phase_split_of_model`
2. `residual_majorant_of_phase_split`

Once those are provided, the repo already closes RH through the existing chain.
