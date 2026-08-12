# K1-W4 Obligation Table (2026-02-24)

## Target contract

`ExplicitFormulaFiniteDecayingPhaseCorrectionsAssumptions`

## Obligations and status

1. `W4-O1` (import boundary)
- Statement: global decomposition from right-half zero (`E(x)=A*x^beta*cos(phase(x))+R(x)`, `R(x)/x^beta -> 0`).
- Lean field: `zero_to_global_decomposition`.
- Status: import-boundary.

2. `W4-O2` (open math; single unconditional blocker)
- Statement: finite decaying mode phase representation (`phase = tau*log + phi + finite decaying mode sum`).
- Lean field: `finite_decaying_phase_correction_of_model`.
- Status: open-math.

3. `W4-O3` (closed in repo)
- Statement: finite-mode correction implies asymptotically linear phase.
- Lean field: `asymptoticallyLinearAssumptionsOfFiniteDecayingCorrections`.
- Status: derived-in-repo.

4. `W4-O4` (closed in repo)
- Statement: asymptotically linear phase implies linear witness and RH chain.
- Lean path: `concreteLinearPhaseWitnessProviderOfAsymptoticallyLinear` -> `rh_from_asymptotically_linear_phase_instance`.
- Status: derived-in-repo.

5. `W4-O5` (closed in repo, conditional on O2 instantiation)
- Statement: finite-mode nonempty provider implies RH in final lock file.
- Lean theorem: `rh_of_nonempty_concrete_finite_decaying_phase_corrections_provider`.
- Status: derived-in-repo.

## Exact remaining unconditional blocker

- `W4-O2` only.

Once `W4-O2` is instantiated non-circularly, the existing Lean chain closes RH in this repository.
