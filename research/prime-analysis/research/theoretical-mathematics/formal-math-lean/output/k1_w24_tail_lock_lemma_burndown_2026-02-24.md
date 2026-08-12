# K1-W24 Tail-Lock Lemma Burndown (for `r6DualBandAsymptoticTailBound`)

Target theorem to close:
- `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeSpinningTopFrontier.lean`
  - `PrimeRiemannBridgeSpinningTopFrontier.r6DualBandAsymptoticTailBound`

## L1 (Open)
Derive omitted-mode explicit-formula remainder representation for normalized residual:
- `R(x)/x^beta = Tail(x)` for `x >= x1`.

## L2 (Closed)
Prove absolute majorization of `Tail(x)` by a tractable envelope `Env(x)`:
- `|Tail(x)| <= Env(x)` for `x >= x1`.

Lean closure:
- `PrimeRiemannBridgeSpinningTopFrontier.abs_decaying_phase_mode_list_correction_le_l1`
- `PrimeRiemannBridgeSpinningTopFrontier.abs_decaying_phase_mode_term_le_model_eta`

## L3 (Closed)
Prove envelope power decay with explicit constants:
- `Env(x) <= C * x^(-eta)` for `x >= x1`.

Lean closure:
- `PrimeRiemannBridgeSpinningTopFrontier.omitted_mode_tail_pack_of_finite_omitted_modes`
  (via `coeff_l1_bound` + `model_eta_le_omitted`)

## L4 (Closed)
Compose L2+L3 into final asymptotic tail lock:
- `|R(x)/x^beta| <= C * x^(-eta)` for `x >= x1`.

Lean closure:
- `PrimeRiemannBridgeSpinningTopFrontier.r6_dual_band_asymptotic_tail_bound_of_omitted_mode_tail_pack`
- `PrimeRiemannBridgeSpinningTopFrontier.r6_dual_band_witness_with_window_and_tail_of_omitted_tail`

## L5 (Open)
Instantiate `R6DualBandWitnessWithWindowAndTailProvider` with:
- finite-window certificate (already done numerically)
- L4 theorem term (missing)

Then endpoint theorem applies:
- `rh_from_r6_dual_band_witness_with_window_and_tail_provider`.

## Current status
- Finite-window side: repeatedly certified up to `x_max=1e22`.
- Asymptotic all-`x` tail theorem is reduced to one representation gate (L1):
  `R(x)/x^beta = decayingPhaseModeListCorrection omitted_modes x` on `x >= x1`,
  plus coefficient/eta side conditions.
