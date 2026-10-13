# INC-0079: Product Phase Translation Large-Bank Boundary Extension

## Status
Confirm completed positive/narrow on 2026-03-11.

## Trigger
`INC-0078` confirmed a coherent crossover boundary on the fixed translated
product law:
- `max_train=3000`: no crossover through `Q24`
- `max_train=6000`: first systems crossover at `Q24`
- `max_train=12000`: first systems crossover already at `Q12`
- secondary-key search work stays pinned near `19%` of dense

The next honest question is whether that boundary keeps moving left at larger
banks, not whether a new route family should be invented.

## Branch Contract
- keep the confirmed `INC-0065` route law fixed
- keep the confirmed `INC-0071` secondary-key law fixed
- keep the confirmed `INC-0078` bank-boundary map fixed
- change only the upper bank-size range and the minimal repeat bracket around
  the expected larger-bank crossover
- do not reopen geometry, spectral probes, or retrieval scoring

## Minimal Scope
1. Carry only:
   - `DENSE`
   - `H4XH4_FIELD_A150_CPX8`
2. Extend the bank map upward:
   - at least one bank above `12000`
   - optionally one second larger bank if the harness stays stable
3. Extend repeat counts only around the expected earlier boundary:
   - bracket `Q08/Q12/Q16`
4. Use the existing translated hardware-profile tool to summarize:
   - first systems crossover repeat per larger bank
   - search-work ratio stability
   - amortized crossover-margin slope

## Working Hypothesis
If the translated product systems branch is a real software bridge to the
hardware-efficiency moonshot, then larger banks should shift the crossover
earlier without materially degrading the search-work ratio.

## Evidence
- Screen config:
  - `configs/proxy_transfer_inc0079_product_phase_translation_large_bank_boundary_extension_screen.json`
- Confirm config:
  - `configs/proxy_transfer_inc0079_product_phase_translation_large_bank_boundary_extension_confirm.json`
- Screen analysis:
  - `results/analysis/inc0079_product_phase_translation_large_bank_boundary_extension_screen.json`
- Confirm analysis:
  - `results/analysis/inc0079_product_phase_translation_large_bank_boundary_extension_confirm.json`
- Screen profile:
  - `results/analysis/inc0079_product_phase_translation_large_bank_boundary_extension_screen_profile.json`
  - `docs/reports/INC0079_PRODUCT_PHASE_TRANSLATION_LARGE_BANK_BOUNDARY_EXTENSION_SCREEN.md`
- Confirm profile:
  - `results/analysis/inc0079_product_phase_translation_large_bank_boundary_extension_confirm_profile.json`
  - `docs/reports/INC0079_PRODUCT_PHASE_TRANSLATION_LARGE_BANK_BOUNDARY_EXTENSION_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260311_222501.md`
  - `docs/governance/gates/gate_20260311_223841.md`

## Screen Read
- The 2-seed upper-bank screen was already coherent.
- `max_train=12000`
  - `DENSE_Q08_T12000`: `top1=0.04833`, `amortized=1.4237s`
  - `H4XH4_FIELD_A150_CPX8_Q08_T12000`: `0.04892`, `cand_frac=0.18418`,
    `amortized=1.7261s`
  - `H4XH4_FIELD_A150_CPX8_Q12_T12000`: `0.04892`, `0.18418`, `1.2707s`
  - onset stayed at `Q12`
- `max_train=18000`
  - `DENSE_Q08_T18000`: `top1=0.04750`, `amortized=3.2385s`
  - `H4XH4_FIELD_A150_CPX8_Q08_T18000`: `0.04733`, `cand_frac=0.18325`,
    `amortized=2.6714s`
  - onset already appeared at `Q08`
- The screen justified a confirm bracket on `Q08/Q12` for both banks.

## Confirm Read
- The 4-seed confirm preserved the same onset shift.
- `max_train=12000`
  - `DENSE_Q08_T12000`: `top1=0.04912`, `amortized=1.31774s`
  - `H4XH4_FIELD_A150_CPX8_Q08_T12000`: `0.04867`, `cand_frac=0.19032`,
    `amortized=1.63033s`
  - `DENSE_Q12_T12000`: `0.04912`, `1.30745s`
  - `H4XH4_FIELD_A150_CPX8_Q12_T12000`: `0.04867`, `0.19032`, `1.22292s`
  - first confirmed crossover remains `Q12`
- `max_train=18000`
  - `DENSE_Q08_T18000`: `top1=0.04864`, `amortized=3.05680s`
  - `H4XH4_FIELD_A150_CPX8_Q08_T18000`: `0.04767`, `cand_frac=0.18997`,
    `amortized=2.63782s`
  - `DENSE_Q12_T18000`: `0.04864`, `3.28121s`
  - `H4XH4_FIELD_A150_CPX8_Q12_T18000`: `0.04767`, `0.18997`, `1.96505s`
  - first confirmed crossover moves left to `Q08`
- Confirmed bank summaries:
  - `T12000`: first systems crossover at `Q12`
  - `T18000`: first systems crossover at `Q08`
  - search-work ratio remains effectively constant:
    - `T12000`: `0.190318`
    - `T18000`: `0.189969`

## Stop Rule
- If larger banks do not improve on the `12000` onset or the search-work ratio
  drifts upward materially, do not escalate toward stronger hardware claims.
- Keep the current result as positive but narrow translated evidence only.

## Reading
- The upper-bank extension is positive.
- The crossover boundary continues moving left as bank size grows:
  - `T12000`: onset at `Q12`
  - `T18000`: onset at `Q08`
- The key systems signature remains stable while that happens:
  - secondary-key search work stays pinned near `19%` of dense
- This is the strongest hardware-side software evidence in the repo so far:
  - repeated-query crossover exists
  - the bank boundary is coherent
  - and the larger bank shifts onset earlier rather than later
- The result is still narrow:
  - translated retrieval harness only
  - only one larger bank above `12000`
  - no direct FLOP or memory-traffic audit yet

## Decision
- Close `INC-0079` confirm positive/narrow.
- Promote `H4XH4_FIELD_A150_CPX8_Q08_T18000` as the earliest confirmed systems
  crossover point so far.
- Keep the fixed product secondary-key law as the live hardware-side systems
  family.
- Move next to a second larger-bank boundary extension instead of reopening
  geometry or retrieval rescue.

## Resume Note
Resume from the confirmed `INC-0079` larger-bank artifacts and treat the next
branch as a second upper-bank extension on the fixed law.
