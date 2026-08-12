# INC-0081: Product Phase Translation Q04 Threshold Search

## Status
Confirm completed positive/narrow on 2026-03-12.

## Trigger
`INC-0080` confirmed that the fixed translated product systems branch now holds
its crossover at `Q08` through `max_train=30000`, but does not yet cross at
`Q04`:
- `max_train=24000`: first systems crossover at `Q08`
- `max_train=30000`: first systems crossover at `Q08`
- secondary-key search work still holds near `19%` of dense

The next honest question is whether `Q04` crossover exists at a higher bank or
whether `Q08` is the practical onset floor on this fixed stack.

## Branch Contract
- keep the confirmed `INC-0065` route law fixed
- keep the confirmed `INC-0071` secondary-key law fixed
- keep the confirmed `INC-0080` upper-bank onset map fixed
- change only the upper bank-size range and the minimal repeat bracket needed
  to test the `Q04` threshold
- do not reopen geometry, spectral probes, or retrieval scoring

## Minimal Scope
1. Carry only:
   - `DENSE`
   - `H4XH4_FIELD_A150_CPX8`
2. Extend the bank map upward again:
   - at least one bank above `30000`
3. Probe only the threshold bracket:
   - `Q04/Q08`
4. Use the existing translated hardware-profile tool to summarize:
   - whether `Q04` ever becomes the first systems crossover repeat
   - whether `Q08` still holds
   - whether the search-work ratio remains stable

## Working Hypothesis
If the software-side hardware story continues scaling, a sufficiently large
bank should either produce the first `Q04` crossover or establish a very clear
`Q08` onset floor with still-stable work ratios.

## Stop Rule
- If `Q08` fails above `30000`, do not escalate the hardware-efficiency claim
  further.
- If `Q04` still does not cross across the new upper-bank bracket, treat `Q08`
  as the current practical onset floor and pivot from boundary extension toward
  direct cost-accounting / memory-traffic analysis on the fixed `Q08` systems
  family.

## Evidence
- Screen config:
  - `configs/proxy_transfer_inc0081_product_phase_translation_q04_threshold_search_screen.json`
- Confirm config:
  - `configs/proxy_transfer_inc0081_product_phase_translation_q04_threshold_search_confirm.json`
- Screen analysis:
  - `results/analysis/inc0081_product_phase_translation_q04_threshold_search_screen.json`
- Confirm analysis:
  - `results/analysis/inc0081_product_phase_translation_q04_threshold_search_confirm.json`
- Screen profile:
  - `results/analysis/inc0081_product_phase_translation_q04_threshold_search_screen_profile.json`
  - `docs/reports/INC0081_PRODUCT_PHASE_TRANSLATION_Q04_THRESHOLD_SEARCH_SCREEN.md`
- Confirm profile:
  - `results/analysis/inc0081_product_phase_translation_q04_threshold_search_confirm_profile.json`
  - `docs/reports/INC0081_PRODUCT_PHASE_TRANSLATION_Q04_THRESHOLD_SEARCH_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260311_234946.md`
  - `docs/governance/gates/gate_20260312_002000.md`

## Screen Read
- The 2-seed threshold screen was mixed but clearly informative.
- `max_train=36000`
  - `DENSE_Q04_T36000`: `top1=0.04778`, `amortized=11.109s`
  - `H4XH4_FIELD_A150_CPX8_Q04_T36000`: `0.04694`, `cand_frac=0.18507`,
    `amortized=9.748s`
  - first systems crossover appeared at `Q04`
- `max_train=40000`
  - `DENSE_Q04_T40000`: `top1=0.04988`, `amortized=8.892s`
  - `H4XH4_FIELD_A150_CPX8_Q04_T40000`: `0.04783`, `cand_frac=0.18748`,
    `amortized=10.037s`
  - `H4XH4_FIELD_A150_CPX8_Q08_T40000`: `0.04783`, `0.18748`, `5.910s`
  - first systems crossover stayed at `Q08`
- Screen bank summaries:
  - `T36000`: first systems crossover at `Q04`, search-work ratio `0.185074`
  - `T40000`: first systems crossover at `Q08`, search-work ratio `0.187484`
- The screen justified confirm on the same `Q04/Q08` bracket because the
  threshold result was real but non-monotone.

## Confirm Read
- The 4-seed confirm preserved the same mixed threshold structure.
- `max_train=36000`
  - `DENSE_Q04_T36000`: `top1=0.04790`, `amortized=12.149s`
  - `H4XH4_FIELD_A150_CPX8_Q04_T36000`: `0.04707`, `cand_frac=0.19021`,
    `amortized=9.694s`
  - first confirmed systems crossover is `Q04`
- `max_train=40000`
  - `DENSE_Q04_T40000`: `top1=0.04885`, `amortized=9.024s`
  - `H4XH4_FIELD_A150_CPX8_Q04_T40000`: `0.04733`, `cand_frac=0.18376`,
    `amortized=9.990s`
  - `DENSE_Q08_T40000`: `0.04885`, `9.642s`
  - `H4XH4_FIELD_A150_CPX8_Q08_T40000`: `0.04733`, `0.18376`, `6.106s`
  - first confirmed systems crossover stays at `Q08`
- Confirmed bank summaries:
  - `T36000`: first systems crossover at `Q04`
  - `T40000`: first systems crossover at `Q08`
  - search-work ratio remains effectively stable:
    - `T36000`: `0.190206`
    - `T40000`: `0.183764`

## Reading
- The first real confirmed `Q04` systems crossover exists:
  - `H4XH4_FIELD_A150_CPX8_Q04_T36000`
- The onset law is not monotone in bank size on this software stack:
  - `Q04` crosses at `36000`
  - `Q04` does not cross at `40000`
  - `Q08` still does
- The candidate-fraction / search-work signal remains stable, so the
  non-monotone threshold is likely a cost-accounting effect rather than route
  collapse.
- This is therefore a positive but narrow hardware-side result:
  - there is now a confirmed `Q04` crossover point
  - but not yet a clean monotone onset law

## Decision
- Close `INC-0081` confirm positive/narrow.
- Promote `H4XH4_FIELD_A150_CPX8_Q04_T36000` as the earliest confirmed systems
  crossover point so far.
- Promote `H4XH4_FIELD_A150_CPX8_Q08_T40000` as the highest-bank confirmed
  systems crossover point so far.
- Do not keep extending the bank map blindly.
- Move next to direct cost-accounting / memory-traffic analysis on the fixed
  `Q04/Q08` systems family to explain the non-monotone upper-bank threshold.

## Resume Note
Resume from the confirmed `INC-0081` threshold artifacts and treat the next
branch as a direct cost-accounting audit on the fixed law.
