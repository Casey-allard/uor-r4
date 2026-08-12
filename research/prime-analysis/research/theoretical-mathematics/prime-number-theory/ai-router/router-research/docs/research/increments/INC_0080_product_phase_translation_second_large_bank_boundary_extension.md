# INC-0080: Product Phase Translation Second Large-Bank Boundary Extension

## Status
Confirm completed positive/narrow on 2026-03-11.

## Trigger
`INC-0079` confirmed that the fixed translated product systems branch moves the
crossover earlier again at a larger bank:
- `max_train=12000`: first systems crossover at `Q12`
- `max_train=18000`: first systems crossover at `Q08`
- secondary-key search work still holds near `19%` of dense

The next honest question is whether that improvement keeps holding at a second
larger bank or starts to saturate.

## Branch Contract
- keep the confirmed `INC-0065` route law fixed
- keep the confirmed `INC-0071` secondary-key law fixed
- keep the confirmed `INC-0079` upper-bank onset map fixed
- change only the upper bank-size range and the minimal repeat bracket around
  the expected still-earlier crossover
- do not reopen geometry, spectral probes, or retrieval scoring

## Minimal Scope
1. Carry only:
   - `DENSE`
   - `H4XH4_FIELD_A150_CPX8`
2. Extend the bank map upward again:
   - at least one bank above `18000`
3. Extend repeat counts only around the expected onset:
   - bracket `Q04/Q08/Q12`
4. Use the existing translated hardware-profile tool to summarize:
   - first systems crossover repeat at the new bank
   - search-work ratio stability
   - amortized crossover-margin slope

## Working Hypothesis
If the software-side hardware story is real, a second larger bank should either
hold the `Q08` onset or move it earlier, without materially degrading the
search-work ratio.

## Stop Rule
- If the new bank does not improve on or at least hold the `Q08` onset, do not
  escalate the hardware-efficiency claim further.
- Keep the current translated result as positive but narrow evidence only.

## Evidence
- Screen config:
  - `configs/proxy_transfer_inc0080_product_phase_translation_second_large_bank_boundary_extension_screen.json`
- Confirm config:
  - `configs/proxy_transfer_inc0080_product_phase_translation_second_large_bank_boundary_extension_confirm.json`
- Screen analysis:
  - `results/analysis/inc0080_product_phase_translation_second_large_bank_boundary_extension_screen.json`
- Confirm analysis:
  - `results/analysis/inc0080_product_phase_translation_second_large_bank_boundary_extension_confirm.json`
- Screen profile:
  - `results/analysis/inc0080_product_phase_translation_second_large_bank_boundary_extension_screen_profile.json`
  - `docs/reports/INC0080_PRODUCT_PHASE_TRANSLATION_SECOND_LARGE_BANK_BOUNDARY_EXTENSION_SCREEN.md`
- Confirm profile:
  - `results/analysis/inc0080_product_phase_translation_second_large_bank_boundary_extension_confirm_profile.json`
  - `docs/reports/INC0080_PRODUCT_PHASE_TRANSLATION_SECOND_LARGE_BANK_BOUNDARY_EXTENSION_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260311_230536.md`
  - `docs/governance/gates/gate_20260311_232657.md`

## Screen Read
- The 2-seed second upper-bank screen was already coherent.
- `max_train=24000`
  - `DENSE_Q04_T24000`: `top1=0.04933`, `amortized=5.162s`
  - `H4XH4_FIELD_A150_CPX8_Q04_T24000`: `0.04925`, `cand_frac=0.19144`,
    `amortized=6.413s`
  - `H4XH4_FIELD_A150_CPX8_Q08_T24000`: `0.04925`, `0.19144`, `3.781s`
  - first systems crossover stayed at `Q08`
- `max_train=30000`
  - `DENSE_Q04_T30000`: `top1=0.04773`, `amortized=7.944s`
  - `H4XH4_FIELD_A150_CPX8_Q04_T30000`: `0.04677`, `cand_frac=0.18229`,
    `amortized=8.254s`
  - `H4XH4_FIELD_A150_CPX8_Q08_T30000`: `0.04677`, `0.18229`, `4.847s`
  - first systems crossover also stayed at `Q08`
- Screen bank summaries:
  - `T24000`: first systems crossover at `Q08`, search-work ratio `0.191436`
  - `T30000`: first systems crossover at `Q08`, search-work ratio `0.182290`
- The screen justified a confirm bracket on `Q04/Q08` for both banks.

## Confirm Read
- The 4-seed confirm preserved the same onset floor.
- `max_train=24000`
  - `DENSE_Q04_T24000`: `top1=0.04935`, `amortized=5.152s`
  - `H4XH4_FIELD_A150_CPX8_Q04_T24000`: `0.04883`, `cand_frac=0.19293`,
    `amortized=7.425s`
  - `DENSE_Q08_T24000`: `0.04935`, `5.491s`
  - `H4XH4_FIELD_A150_CPX8_Q08_T24000`: `0.04883`, `0.19293`, `3.867s`
  - first confirmed systems crossover stayed at `Q08`
- `max_train=30000`
  - `DENSE_Q04_T30000`: `top1=0.04802`, `amortized=7.901s`
  - `H4XH4_FIELD_A150_CPX8_Q04_T30000`: `0.04717`, `cand_frac=0.18886`,
    `amortized=8.946s`
  - `DENSE_Q08_T30000`: `0.04802`, `7.560s`
  - `H4XH4_FIELD_A150_CPX8_Q08_T30000`: `0.04717`, `0.18886`, `5.394s`
  - first confirmed systems crossover also stayed at `Q08`
- Confirmed bank summaries:
  - `T24000`: first systems crossover at `Q08`
  - `T30000`: first systems crossover at `Q08`
  - search-work ratio stays effectively stable:
    - `T24000`: `0.192929`
    - `T30000`: `0.188863`

## Reading
- The second upper-bank extension is positive.
- The fixed translated product systems branch now holds its `Q08` onset through
  both `24000` and `30000`.
- The search-work ratio stays nearly fixed and even improves slightly at the
  largest confirmed bank:
  - about `19.3%` of dense at `24000`
  - about `18.9%` of dense at `30000`
- The branch did not move earlier to `Q04`.
- The hardware-side scaling story is therefore stronger, but the next honest
  question is now threshold discovery rather than another generic bank map:
  - does `Q04` ever cross at a larger bank
  - or is `Q08` the practical onset floor on this fixed software stack

## Decision
- Close `INC-0080` confirm positive/narrow.
- Keep `H4XH4_FIELD_A150_CPX8_Q08_T18000` as the earliest confirmed systems
  crossover point.
- Promote `H4XH4_FIELD_A150_CPX8_Q08_T30000` as the highest-bank confirmed
  systems crossover point so far.
- Move next to an explicit `Q04` threshold search above `30000` rather than
  pretending the onset already moved earlier than it did.

## Resume Note
Resume from the confirmed `INC-0080` upper-bank artifacts and treat the next
branch as a `Q04` threshold search on the fixed law.
