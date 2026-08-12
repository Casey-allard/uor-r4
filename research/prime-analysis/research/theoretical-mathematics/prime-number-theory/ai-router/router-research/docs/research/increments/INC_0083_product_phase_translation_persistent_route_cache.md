# INC-0083: Product Phase Translation Persistent Route Cache

## Status
Confirm completed positive/narrow on 2026-03-12.

## Trigger
`INC-0082` explained the non-monotone `Q04` threshold directly from cost
composition on the fixed translated product stack:
- `T36000 Q04` crosses because online savings exceed the offline penalty
- `T40000 Q04` misses because the offline penalty still exceeds the online
  savings
- `T40000 Q08` still crosses because the same static offline cost is amortized
  over more repeats

The next honest question is whether a persistent route-cache implementation can
remove enough static build cost to turn the fixed `Q04 T40000` systems point
into a warm-cache crossover without changing the geometry or retrieval law.

## Branch Contract
- keep the confirmed `INC-0065` product route law fixed
- keep the confirmed `INC-0071` secondary-key law fixed
- keep the confirmed `INC-0081` onset map fixed
- keep the `INC-0082` accounting read fixed
- change only the offline route-build reuse surface in
  `tasks/router_retrieval_eval.py`
- do not reopen geometry, spectral probes, retrieval scoring, or bank search

## Minimal Scope
1. Activate the already-exposed retrieval cache surface:
   - `--cache_chart`
   - `--cache_routes`
2. Cache enough of the fixed translated retrieval state to skip redundant
   offline route-build work on warm runs.
3. Carry only the narrow bracket needed to test the rescue:
   - `DENSE_Q04_T40000`
   - `H4XH4_FIELD_A150_CPX8_Q04_T40000`
   - `DENSE_Q08_T40000`
   - `H4XH4_FIELD_A150_CPX8_Q08_T40000`
4. Report cold versus warm-cache timing separately.

## Working Hypothesis
If the hardware-side translated systems story is real, warm-cache reuse of the
static route-build stage should recover most of the `Q04 T40000` amortized
miss without materially changing top-1, candidate fraction, or search-work
ratio.

## Stop Rule
- If warm-cache reuse changes candidate fraction or retrieval quality, treat
  that as a bug, not a systems win.
- If warm-cache reuse does not materially reduce the routed offline cost, do
  not claim a `Q04 T40000` rescue.

## Evidence
- Retrieval cache implementation:
  - `tasks/router_retrieval_eval.py`
  - `tools/translated_cache_compare.py`
- Tests:
  - `tests/test_cache_contract.py`
  - `tests/test_router_retrieval_eval.py`
- Screen configs:
  - `configs/proxy_transfer_inc0083_product_phase_translation_persistent_route_cache_screen_cold.json`
  - `configs/proxy_transfer_inc0083_product_phase_translation_persistent_route_cache_screen_warm.json`
- Confirm configs:
  - `configs/proxy_transfer_inc0083_product_phase_translation_persistent_route_cache_confirm_cold.json`
  - `configs/proxy_transfer_inc0083_product_phase_translation_persistent_route_cache_confirm_warm.json`
- Screen analyses:
  - `results/analysis/inc0083_product_phase_translation_persistent_route_cache_screen_cold.json`
  - `results/analysis/inc0083_product_phase_translation_persistent_route_cache_screen_warm.json`
  - `results/analysis/inc0083_product_phase_translation_persistent_route_cache_screen_compare.json`
  - `docs/reports/INC0083_PRODUCT_PHASE_TRANSLATION_PERSISTENT_ROUTE_CACHE_SCREEN_COMPARE.md`
- Confirm analyses:
  - `results/analysis/inc0083_product_phase_translation_persistent_route_cache_confirm_cold.json`
  - `results/analysis/inc0083_product_phase_translation_persistent_route_cache_confirm_warm.json`
  - `results/analysis/inc0083_product_phase_translation_persistent_route_cache_confirm_compare.json`
  - `docs/reports/INC0083_PRODUCT_PHASE_TRANSLATION_PERSISTENT_ROUTE_CACHE_CONFIRM_COMPARE.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_010935.md`
  - `docs/governance/gates/gate_20260312_011438.md`
  - `docs/governance/gates/gate_20260312_012911.md`
  - `docs/governance/gates/gate_20260312_013859.md`

## Screen Read
- The cold/warm 2-seed screen was immediately decisive.
- `Q04 T40000`
  - cold:
    - `DENSE_Q04_T40000`: `top1=0.049875`, `amortized=9.773s`
    - `H4XH4_FIELD_A150_CPX8_Q04_T40000`: `top1=0.047825`,
      `cand_frac=0.187484`, `amortized=10.641s`
  - warm:
    - `DENSE_Q04_T40000`: `top1=0.049875`, `amortized=9.883s`
    - `H4XH4_FIELD_A150_CPX8_Q04_T40000`: `top1=0.047825`,
      `cand_frac=0.187484`, `amortized=2.021s`
    - routed cache hits:
      - `chart_cache_hit=1.0`
      - `route_cache_hit=1.0`
- `Q08 T40000`
  - cold:
    - `DENSE_Q08_T40000`: `top1=0.049875`, `amortized=9.558s`
    - `H4XH4_FIELD_A150_CPX8_Q08_T40000`: `top1=0.047825`,
      `cand_frac=0.187484`, `amortized=6.250s`
  - warm:
    - `DENSE_Q08_T40000`: `top1=0.049875`, `amortized=9.822s`
    - `H4XH4_FIELD_A150_CPX8_Q08_T40000`: `top1=0.047825`,
      `cand_frac=0.187484`, `amortized=2.080s`
    - routed cache hits:
      - `chart_cache_hit=1.0`
      - `route_cache_hit=1.0`
- The screen justified confirm because warm-cache reuse removed nearly all
  routed offline cost while preserving routed quality and candidate fraction.

## Confirm Read
- The 4-seed confirm kept the same read.
- `Q04 T40000`
  - cold:
    - `DENSE_Q04_T40000`: `top1=0.048850`, `amortized=9.347s`
    - `H4XH4_FIELD_A150_CPX8_Q04_T40000`: `top1=0.047325`,
      `cand_frac=0.183764`, `amortized=10.389s`
    - routed cold margin vs dense: `-1.042s/repeat`
  - warm:
    - `DENSE_Q04_T40000`: `top1=0.048850`, `amortized=9.246s`
    - `H4XH4_FIELD_A150_CPX8_Q04_T40000`: `top1=0.047325`,
      `cand_frac=0.183764`, `amortized=1.972s`
    - routed warm margin vs dense: `+7.274s/repeat`
    - routed cache hits:
      - `chart_cache_hit=1.0`
      - `route_cache_hit=1.0`
    - routed offline cost:
      - cold: `8.397s/repeat`
      - warm: `0.011s/repeat`
- `Q08 T40000`
  - cold:
    - `DENSE_Q08_T40000`: `top1=0.048850`, `amortized=9.506s`
    - `H4XH4_FIELD_A150_CPX8_Q08_T40000`: `top1=0.047325`,
      `cand_frac=0.183764`, `amortized=6.165s`
    - routed cold margin vs dense: `+3.342s/repeat`
  - warm:
    - `DENSE_Q08_T40000`: `top1=0.048850`, `amortized=9.113s`
    - `H4XH4_FIELD_A150_CPX8_Q08_T40000`: `top1=0.047325`,
      `cand_frac=0.183764`, `amortized=1.891s`
    - routed warm margin vs dense: `+7.222s/repeat`
    - routed cache hits:
      - `chart_cache_hit=1.0`
      - `route_cache_hit=1.0`
    - routed offline cost:
      - cold: `4.251s/repeat`
      - warm: `0.006s/repeat`
- Stability checks all passed:
  - top-1 stayed unchanged between cold and warm routed runs
  - candidate fraction stayed unchanged between cold and warm routed runs
  - dense controls stayed effectively unchanged aside from runtime noise

## Reading
- `INC-0082` was correct about the failure mode:
  - the route signal was already good enough
  - the blocking term was static offline chart / train-route build cost
- Persistent cache reuse removes almost all of that fixed cost on the routed
  translated stack without changing the geometry, retrieval law, top-1, or
  candidate fraction.
- `Q04 T40000` is now a clear warm-cache systems crossover rather than a
  cold-start miss.
- This is not a new geometry result. It is the first clean software-side
  operationalization of the fixed translated product law at the upper bank.

## Decision
- Close `INC-0083` confirm positive/narrow.
- Keep the confirmed route law and secondary-key law fixed.
- Promote `H4XH4_FIELD_A150_CPX8_Q04_T40000` as the warm-cache rescued upper-bank
  crossover point.
- Treat `H4XH4_FIELD_A150_CPX8_Q08_T40000` as the warm-cache stabilized
  high-bank systems point.
- Move next to a warm-cache onset map on the fixed `T40000` bank rather than
  reopening geometry or bank-size search.

## Resume Note
Resume from the `INC-0083` confirm artifacts and treat the next branch as a
warm-cache onset map on the fixed translated stack.
