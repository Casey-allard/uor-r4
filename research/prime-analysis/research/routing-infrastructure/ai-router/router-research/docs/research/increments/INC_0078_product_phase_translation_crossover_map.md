# INC-0078: Product Phase Translation Crossover Map

## Status
Confirm completed positive/narrow on 2026-03-11.

## Trigger
`INC-0077` showed that the fixed product phase-field branch survives the first
explicit hardware-cost profile, but the crossover boundary still needed an
explicit map:
- at `max_train=12000`, the tested boundary had only been checked down to `Q16`
- at `max_train=6000`, crossover survived only at `Q24`
- secondary-key search work remained near `19%` of dense at both scales

The next honest question is therefore the crossover boundary itself, not more
retrieval rescue or new geometry.

## Branch Contract
- keep the confirmed `INC-0065` route law fixed
- keep the confirmed `INC-0071` secondary-key law fixed
- keep the confirmed `INC-0076/0077` crossover points fixed
- change only bank size and repeat-count coverage around the crossover boundary
- do not reopen geometry, spectral probes, or retrieval scoring

## Minimal Scope
1. Sweep only the fixed crossover family:
   - `DENSE`
   - `H4XH4_FIELD_A150`
   - `H4XH4_FIELD_A150_CPX8`
2. Extend bank sizes just enough to map the boundary:
   - at least one smaller bank below `6000`
   - current `12000`
   - optionally one larger bank if the harness stays stable
3. Extend repeat counts only around the boundary:
   - bracket `Q12/Q16/Q20/Q24`
4. Use the existing hardware-profile tool to summarize:
   - first crossover repeat per bank size
   - search-work ratio stability
   - amortized crossover margin slope

## Working Hypothesis
The secondary-key product branch may become more favorable as the bank grows,
because the search-work ratio is already stable while dense cost scales more
directly with bank size. If that slope holds, the repo will have a clearer
software-side path from geometric routing to the hardware-efficiency moonshot.

## Evidence
- Screen config:
  - `configs/proxy_transfer_inc0078_product_phase_translation_crossover_map_screen.json`
- Confirm config:
  - `configs/proxy_transfer_inc0078_product_phase_translation_crossover_map_confirm.json`
- Screen analysis:
  - `results/analysis/inc0078_product_phase_translation_crossover_map_screen.json`
- Confirm analysis:
  - `results/analysis/inc0078_product_phase_translation_crossover_map_confirm.json`
- Screen profile:
  - `results/analysis/inc0078_product_phase_translation_crossover_map_screen_profile.json`
  - `docs/reports/INC0078_PRODUCT_PHASE_TRANSLATION_CROSSOVER_MAP_SCREEN.md`
- Confirm profile:
  - `results/analysis/inc0078_product_phase_translation_crossover_map_confirm_profile.json`
  - `docs/reports/INC0078_PRODUCT_PHASE_TRANSLATION_CROSSOVER_MAP_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260311_155644.md`
  - `docs/governance/gates/gate_20260311_161119.md`

## Screen Read
- The 2-seed boundary map was coherent immediately.
- `max_train=3000`
  - no routed crossover survived through `Q24`
  - `H4XH4_FIELD_A150_CPX8_Q24_T3000` still trailed dense amortized cost by
    about `0.010s`
- `max_train=6000`
  - no routed crossover at `Q12/Q16/Q20`
  - first systems crossover at `Q24`
- `max_train=12000`
  - first systems crossover already appeared at `Q12`
  - `H4XH4_FIELD_A150_CPX8_Q12_T12000` screened at `top1=0.04892`,
    `cand_frac=0.18418`, `amortized=1.2171s`
- The secondary-key search-work ratio stayed essentially fixed at about `0.19`
  across all three bank sizes.
- The screen therefore justified a confirm bracket at the actual boundary:
  - `T3000`: `Q20/Q24`
  - `T6000`: `Q20/Q24`
  - `T12000`: `Q12/Q16`

## Confirm Read
- The 4-seed confirm preserved the same boundary map.
- `max_train=3000`
  - `DENSE_Q20_T3000`: `top1=0.04983`, `amortized=0.14169s`
  - `H4XH4_FIELD_A150_CPX8_Q20_T3000`: `0.04483`, `cand_frac=0.19170`,
    `amortized=0.19290s`
  - `DENSE_Q24_T3000`: `0.04983`, `0.12743s`
  - `H4XH4_FIELD_A150_CPX8_Q24_T3000`: `0.04483`, `0.19170`, `0.17065s`
  - no routed crossover survived through `Q24`
- `max_train=6000`
  - `DENSE_Q20_T6000`: `top1=0.04867`, `amortized=0.38043s`
  - `H4XH4_FIELD_A150_CPX8_Q20_T6000`: `0.04708`, `cand_frac=0.18723`,
    `amortized=0.40697s`
  - `DENSE_Q24_T6000`: `0.04867`, `0.37993s`
  - `H4XH4_FIELD_A150_CPX8_Q24_T6000`: `0.04708`, `0.18723`, `0.36377s`
  - first confirmed crossover remains `Q24`
- `max_train=12000`
  - `DENSE_Q12_T12000`: `top1=0.04912`, `amortized=1.31959s`
  - `H4XH4_FIELD_A150_CPX8_Q12_T12000`: `0.04867`, `cand_frac=0.19032`,
    `amortized=1.24529s`
  - `DENSE_Q16_T12000`: `0.04912`, `1.37060s`
  - `H4XH4_FIELD_A150_CPX8_Q16_T12000`: `0.04867`, `0.19032`, `1.02595s`
  - first confirmed crossover already occurs by `Q12`
- Confirmed bank summaries:
  - `T3000`: no crossover through `Q24`
  - `T6000`: first systems crossover at `Q24`
  - `T12000`: first systems crossover at `Q12`
  - `H4XH4_FIELD_A150_CPX8` keeps a flat search-work ratio:
    - `T6000`: `0.187229`
    - `T12000`: `0.190318`

## Stop Rule
- If the crossover boundary does not improve or at least remain coherent as bank
  size grows, do not escalate to stronger hardware-efficiency claims.
- Keep the translated branch as positive but narrow software evidence only.

## Reading
- The crossover boundary is now explicitly confirmed and coherent.
- The secondary-key product branch improves as the bank grows:
  - no crossover through `Q24` at `3000`
  - first crossover at `Q24` for `6000`
  - first crossover already by `Q12` for `12000`
- The search-work reduction is not drifting while that happens:
  - secondary-key branch stays pinned at about `19%` of dense candidate scan
    work
- This is stronger hardware-side evidence than `INC-0077` alone:
  - not just one smaller-bank crossover point
  - an actual monotone boundary map on the fixed law
- The evidence is still narrow:
  - translated retrieval harness only
  - no direct larger-than-`12000` bank yet
  - no explicit FLOP or memory-traffic model yet

## Decision
- Close `INC-0078` confirm positive/narrow.
- Promote the confirmed boundary map:
  - `T3000`: no crossover through `Q24`
  - `T6000`: first systems crossover at `Q24`
  - `T12000`: first systems crossover at `Q12`
- Keep `H4XH4_FIELD_A150_CPX8` as the live systems family for the hardware-side
  branch.
- Move next to a larger-bank extension of the same boundary map instead of any
  new geometry or retrieval rescue.

## Resume Note
Resume from the confirmed `INC-0078` boundary-map artifacts and treat the next
branch as a larger-bank extension on the fixed law, not a search for new route
families.
