# INC-0077: Product Phase Translation Hardware Profile

## Status
Confirm completed positive/narrow on 2026-03-11.

## Trigger
`INC-0076` confirmed that the fixed product phase-field branch now crosses
dense exact retrieval on amortized translated cost by `Q16`:
- `H4XH4_FIELD_A150_Q16` matches dense top-1 while beating dense amortized cost
- `H4XH4_FIELD_A150_CPX8_Q16` crosses earlier on the systems axis with stronger
  pruning and a small top-1 regression
- `H4XH4_FIELD_A150_CPX8_Q24` remains the stabilized dense-frontier systems
  point

The next kill-list-aligned question was whether that crossover survives the
first explicit hardware-cost profile under one smaller-bank reference scale.

## Branch Contract
- keep the confirmed `INC-0065` route law fixed
- keep the confirmed `INC-0071` secondary-key law fixed
- keep the confirmed `INC-0076` crossover points fixed
- change only cost-profile and workload-scale diagnostics
- do not reopen geometry, spectral probes, or retrieval scoring

## Minimal Scope
1. Carry only:
   - `DENSE_Q16`
   - `DENSE_Q24`
   - `H4XH4_FIELD_A150_Q16`
   - `H4XH4_FIELD_A150_Q24`
   - `H4XH4_FIELD_A150_CPX8_Q16`
   - `H4XH4_FIELD_A150_CPX8_Q24`
2. Add explicit hardware-facing derived summaries for:
   - candidates scanned
   - dense-vs-routed search work ratio
   - offline build share
   - online-per-repeat share
   - amortized crossover margin
3. Add one bank-scale axis:
   - smaller bank `max_train=6000`
   - current bank `max_train=12000`

## Tooling
- Profiling tool:
  - `tools/translated_hardware_profile.py`
- Regression coverage:
  - `tests/test_translated_hardware_profile.py`

## Evidence
- Screen config:
  - `configs/proxy_transfer_inc0077_product_phase_translation_hardware_profile_screen.json`
- Confirm config:
  - `configs/proxy_transfer_inc0077_product_phase_translation_hardware_profile_confirm.json`
- Screen analysis:
  - `results/analysis/inc0077_product_phase_translation_hardware_profile_screen.json`
- Confirm analysis:
  - `results/analysis/inc0077_product_phase_translation_hardware_profile_confirm.json`
- Screen profile:
  - `results/analysis/inc0077_product_phase_translation_hardware_profile_screen_profile.json`
  - `docs/reports/INC0077_PRODUCT_PHASE_TRANSLATION_HARDWARE_PROFILE_SCREEN.md`
- Confirm profile:
  - `results/analysis/inc0077_product_phase_translation_hardware_profile_confirm_profile.json`
  - `docs/reports/INC0077_PRODUCT_PHASE_TRANSLATION_HARDWARE_PROFILE_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260311_152632.md`
  - `docs/governance/gates/gate_20260311_153047.md`

## Screen Read
- The smaller-bank screen already showed the main slope change:
  - at `train=6000`, no routed crossover survived at `Q16`
  - at `train=6000`, the secondary-key branch crossed dense at `Q24`
  - the search-work ratios stayed almost unchanged from the `12000` run:
    - `H4XH4_FIELD_A150_CPX8_Q24_T6000`: `work_ratio_vs_dense=0.190`
    - `H4XH4_FIELD_A150_Q24_T6000`: `work_ratio_vs_dense=0.315`
- The screen therefore justified confirm on the same smaller-bank `Q16/Q24`
  bracket.

## Confirm Read
- The smaller-bank confirm preserved the profile.
- `train=6000`, `Q16`
  - `DENSE_Q16_T6000`: `top1=0.04867`, `amortized=0.381s`
  - `H4XH4_FIELD_A150_Q16_T6000`: `0.04450`, `cand_frac=0.30833`, `0.490s`
  - `H4XH4_FIELD_A150_CPX8_Q16_T6000`: `0.04708`, `cand_frac=0.18723`, `0.464s`
  - no routed crossover survived
- `train=6000`, `Q24`
  - `DENSE_Q24_T6000`: `top1=0.04867`, `amortized=0.382s`
  - `H4XH4_FIELD_A150_Q24_T6000`: `0.04450`, `cand_frac=0.30833`, `0.394s`
  - `H4XH4_FIELD_A150_CPX8_Q24_T6000`: `0.04708`, `cand_frac=0.18723`, `0.351s`
  - smaller-bank crossover survives only on the secondary-key systems point
- Combined confirmed hardware profile versus the existing `train=12000` confirm:
  - `train=6000`, `Q24`, `H4XH4_FIELD_A150_CPX8_Q24_T6000`
    - `work_ratio_vs_dense=0.187`
    - `offline_share=0.590`
    - `online_share=0.363`
    - `amortized_margin_vs_dense=+0.032s`
    - `top1_delta_vs_dense=-0.00158`
  - `train=12000`, `Q16`, `H4XH4_FIELD_A150_Q16`
    - `work_ratio_vs_dense=0.315`
    - `amortized_margin_vs_dense=+0.159s`
    - `top1_delta_vs_dense=0.00000`
  - `train=12000`, `Q16`, `H4XH4_FIELD_A150_CPX8_Q16`
    - `work_ratio_vs_dense=0.190`
    - `amortized_margin_vs_dense=+0.286s`
    - `top1_delta_vs_dense=-0.00046`

## Reading
- The product branch survives the first explicit hardware-cost profile.
- The search-work reduction is extremely stable:
  - secondary-key branch stays near `19%` of dense candidate scan work
  - plain product branch stays near `31%`
- The crossover is scale-dependent rather than fragile:
  - at `12000`, crossover begins by `Q16`
  - at `6000`, crossover shifts to `Q24`
- The current best bridge to the hardware-efficiency story is therefore narrow
  but real:
  - strong candidate-scan reduction
  - strong online-per-repeat reduction
  - a repeated-query amortized crossover that survives at both scales, though
    later at the smaller bank
- The quality-matched crossover is not scale-stable yet:
  - `H4XH4_FIELD_A150_Q16` works at `12000`
  - the smaller-bank check leaves only the secondary-key systems point alive

## Decision
- Close `INC-0077` confirm positive/narrow.
- Promote `H4XH4_FIELD_A150_CPX8_Q24_T6000` as the first confirmed smaller-bank
  crossover point.
- Keep `H4XH4_FIELD_A150_Q16` as the quality-matched crossover at the current
  `12000` bank.
- Keep `H4XH4_FIELD_A150_CPX8_Q16` and `H4XH4_FIELD_A150_CPX8_Q24` as the main
  current-bank systems crossover points.
- Move next to crossover-boundary mapping on the same fixed law instead of any
  new geometry or retrieval rescue.

## Resume Note
Resume from the confirmed `INC-0076` and `INC-0077` profile artifacts only.
The live question is now the crossover boundary over bank size and repeat count.
