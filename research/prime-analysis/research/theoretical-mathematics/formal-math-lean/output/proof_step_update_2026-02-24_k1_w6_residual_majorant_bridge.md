# Proof Step Update (2026-02-24): K1-W6 Residual-Majorant Bridge

## Completed in this step

1. Added a new contract that decomposes the prior finite-mode blocker into explicit sub-obligations:
- finite decaying mode list plus residual term,
- residual bounded by `C * x^{-eta}` with `eta > 0`.

2. Proved that this contract implies asymptotically linear phase.

3. Wired the new provider class to RH endpoint theorems in the oscillatory module.

4. Wired nonempty-provider bridge to RH in the final lock module.

## New Lean objects

File: `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeOscillatoryReduction.lean`

- `ExplicitFormulaFiniteModeResidualMajorantAssumptions` (`:491`)
- `finiteModeResidualMajorantAssumptionsOfLinearPhaseOnly` (`:521`)
- `tendsto_zero_of_abs_le_const_rpow_neg` (`:620`)
- `asymptoticallyLinearAssumptionsOfFiniteModeResidualMajorant` (`:685`)
- `ConcreteFiniteModeResidualMajorantProvider` (`:1363`)
- `concreteAsymptoticallyLinearPhaseProviderOfFiniteModeResidualMajorant` (`:1378`)
- `endpoint_to_rh_from_finite_mode_residual_majorant_instance` (`:1451`)
- `rh_from_finite_mode_residual_majorant_instance` (`:1457`)
- `concreteFiniteModeResidualMajorantProviderOfLinearPhaseOnly` (`:1498`)
- `rh_from_linear_phase_only_via_finite_mode_residual_majorant_instance` (`:1511`)

File: `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeSchlagePuchta2019ImportedInstance.lean`

- `nonempty_concrete_asymptotically_linear_phase_provider_of_nonempty_concrete_finite_mode_residual_majorant_provider` (`:788`)
- `rh_of_nonempty_concrete_finite_mode_residual_majorant_provider` (`:796`)

## Verification

- `lake build PrimeRiemannBridgeOscillatoryReduction PrimeRiemannBridgeSchlagePuchta2019ImportedInstance` passed.
- `lake build` passed.

## Numerical support run (same step)

Used cached multi-mode probe with up to 8 modes:

- `/Users/adminamn/Documents/New project/research/output/k1_multimode_phase_probe_2026-02-24_x1e18_m8.json`
- `/Users/adminamn/Documents/New project/research/output/k1_multimode_phase_probe_2026-02-24_x1e20_m8.json`
- `/Users/adminamn/Documents/New project/research/output/k1_multimode_phase_probe_2026-02-24_x1e22_m8.json`
- Trend summary: `/Users/adminamn/Documents/New project/research/output/k1_multimode_window_trend_2026-02-24_m8.md`
- Candidate pack: `/Users/adminamn/Documents/New project/research/output/k1_residual_majorant_candidate_pack_2026-02-24.md`

Observed best finite-range case:

- tail sup/amp total: `~1.2976` (still above strict `<=1.0` gate)
- status: useful for shaping B1/B2 assumptions, insufficient for unconditional formal instantiation

## Exact remaining unconditional blocker

- Still one blocker: construct a non-circular instance of
`ConcreteFiniteModeResidualMajorantProvider` (equivalently a theorem payload for
`finite_mode_plus_majorized_residual_of_model`).

Once this is instantiated, the current Lean chain already closes to RH in-repo.
