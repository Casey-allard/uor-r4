# INC-0084: Product Phase Translation Warm Cache Onset Map

## Status
Confirm completed positive/narrow on 2026-03-12.

## Trigger
`INC-0083` confirmed that persistent chart/train-route cache reuse removes
almost all static offline cost on the fixed translated product stack without
changing routed quality or search work:
- `Q04 T40000` cold miss:
  - `H4XH4_FIELD_A150_CPX8_Q04_T40000`: `10.389s`
  - `DENSE_Q04_T40000`: `9.347s`
- `Q04 T40000` warm rescue:
  - `H4XH4_FIELD_A150_CPX8_Q04_T40000`: `1.972s`
  - `DENSE_Q04_T40000`: `9.246s`
- `Q08 T40000` warm systems point also strengthens:
  - `H4XH4_FIELD_A150_CPX8_Q08_T40000`: `1.891s`
  - `DENSE_Q08_T40000`: `9.113s`

The next honest question is no longer “can warm cache rescue `Q04`?” That is
already answered. The next question is “where is the real warm-cache onset on
the fixed upper-bank stack?”

## Branch Contract
- keep the confirmed `INC-0065` route law fixed
- keep the confirmed `INC-0071` secondary-key law fixed
- keep the confirmed `INC-0083` persistent-cache implementation fixed
- keep the bank fixed at `max_train=40000` on the first pass
- change only the query-repeat bracket under warm-cache conditions
- do not reopen geometry, retrieval scoring, or bank-size search

## Minimal Scope
1. Carry only:
   - `DENSE`
   - `H4XH4_FIELD_A150_CPX8`
2. Use persistent cache enabled for the routed runs.
3. Probe only the repeat onset bracket:
   - `Q01`
   - `Q02`
   - `Q04`
   - `Q08`
4. Determine:
   - whether warm-cache crossover already holds at `Q01`
   - whether `Q02` is the first stable onset if `Q01` misses
   - whether top-1 and candidate fraction remain unchanged under reuse

## Working Hypothesis
Once static chart and train-route state are persisted, the fixed translated
product branch should cross dense exact retrieval at `Q01` or `Q02` on the
`T40000` bank, because the remaining routed cost is dominated by the already
advantaged online search term rather than by one-time build work.

## Stop Rule
- If warm-cache routed runs change candidate fraction or top-1, treat that as
  a cache bug, not a systems win.
- If warm-cache routed runs still do not cross by `Q02`, treat persistent
  cache as a strong repeated-query rescue but not yet as a near-single-query
  promotion.

## Evidence
- Screen config:
  - `configs/proxy_transfer_inc0084_product_phase_translation_warm_cache_onset_map_screen.json`
- Confirm config:
  - `configs/proxy_transfer_inc0084_product_phase_translation_warm_cache_onset_map_confirm.json`
- Screen analysis:
  - `results/analysis/inc0084_product_phase_translation_warm_cache_onset_map_screen.json`
- Confirm analysis:
  - `results/analysis/inc0084_product_phase_translation_warm_cache_onset_map_confirm.json`
- Report:
  - `docs/reports/INC0084_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_ONSET_MAP.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_015936.md`
  - `docs/governance/gates/gate_20260312_021141.md`

## Screen Read
- The 2-seed warm-cache screen was already unambiguous.
- Dense controls stayed near the same amortized band across repeats:
  - `DENSE_Q01_T40000`: `top1=0.049875`, `amortized=10.179s`
  - `DENSE_Q02_T40000`: `0.049875`, `9.585s`
  - `DENSE_Q04_T40000`: `0.049875`, `9.238s`
  - `DENSE_Q08_T40000`: `0.049875`, `9.258s`
- The warm-cache routed point crossed dense at every tested repeat:
  - `H4XH4_FIELD_A150_CPX8_Q01_T40000`: `top1=0.047825`,
    `cand_frac=0.187484`, `amortized=2.466s`
  - `H4XH4_FIELD_A150_CPX8_Q02_T40000`: `0.047825`, `0.187484`, `2.141s`
  - `H4XH4_FIELD_A150_CPX8_Q04_T40000`: `0.047825`, `0.187484`, `1.942s`
  - `H4XH4_FIELD_A150_CPX8_Q08_T40000`: `0.047825`, `0.187484`, `2.035s`
- All routed warm runs hit both caches:
  - `chart_cache_hit=1.0`
  - `route_cache_hit=1.0`
- The screen justified confirm because the onset had already moved to `Q01`
  on the exact fixed upper-bank stack.

## Confirm Read
- The 4-seed confirm preserved the same result.
- Dense controls:
  - `DENSE_Q01_T40000`: `top1=0.048850`, `amortized=9.536s`
  - `DENSE_Q02_T40000`: `0.048850`, `9.173s`
  - `DENSE_Q04_T40000`: `0.048850`, `9.364s`
  - `DENSE_Q08_T40000`: `0.048850`, `9.244s`
- Warm-cache routed point:
  - `H4XH4_FIELD_A150_CPX8_Q01_T40000`: `top1=0.047325`,
    `cand_frac=0.183764`, `amortized=2.204s`
  - `H4XH4_FIELD_A150_CPX8_Q02_T40000`: `0.047325`, `0.183764`, `2.022s`
  - `H4XH4_FIELD_A150_CPX8_Q04_T40000`: `0.047325`, `0.183764`, `1.924s`
  - `H4XH4_FIELD_A150_CPX8_Q08_T40000`: `0.047325`, `0.183764`, `1.868s`
- Cache behavior remained exact across the routed bracket:
  - `chart_cache_hit=1.0`
  - `route_cache_hit=1.0`
- Quality/search signal remained invariant across repeats under reuse:
  - `top1=0.047325`
  - `cand_frac=0.183764`

## Reading
- Persistent cache reuse does not just rescue the old repeated-query onset.
  It moves the fixed translated product stack all the way to a confirmed
  single-query warm-cache crossover at `T40000`.
- This is the first confirm-stage persisted-bank single-query systems result
  on the fixed product phase-field branch.
- The result is still narrow:
  - it depends on the persisted-bank assumption
  - it is established only at the `T40000` bank so far
- It is still important:
  - the route law, secondary-key law, and search-work ratio all stayed fixed
  - the win comes from operationalizing the fixed geometry-native bank reuse
    story rather than from retuning the route itself

## Decision
- Close `INC-0084` confirm positive/narrow.
- Promote `H4XH4_FIELD_A150_CPX8_Q01_T40000` as the current warm-cache
  single-query crossover point.
- Keep the route law, secondary-key law, bank, and cache implementation fixed.
- Move next to a bank-boundary search for the earliest `Q01` warm-cache
  crossover (`INC-0085`) rather than reopening geometry.

## Resume Note
Resume from the `INC-0084` confirm artifacts and treat the next branch as a
warm-cache `Q01` bank-boundary search on the fixed translated stack.
