# INC-0084 Product Phase Translation Warm Cache Onset Map

## Confirm Read
- Warm-cache onset now holds at `Q01` on the fixed `T40000` translated stack.
- Dense confirm means:
  - `DENSE_Q01_T40000`: `top1=0.048850`, `amortized=9.536s`
  - `DENSE_Q02_T40000`: `0.048850`, `9.173s`
  - `DENSE_Q04_T40000`: `0.048850`, `9.364s`
  - `DENSE_Q08_T40000`: `0.048850`, `9.244s`
- Routed confirm means:
  - `H4XH4_FIELD_A150_CPX8_Q01_T40000`: `top1=0.047325`,
    `cand_frac=0.183764`, `amortized=2.204s`
  - `H4XH4_FIELD_A150_CPX8_Q02_T40000`: `0.047325`, `0.183764`, `2.022s`
  - `H4XH4_FIELD_A150_CPX8_Q04_T40000`: `0.047325`, `0.183764`, `1.924s`
  - `H4XH4_FIELD_A150_CPX8_Q08_T40000`: `0.047325`, `0.183764`, `1.868s`
- All routed runs hit both caches:
  - `chart_cache_hit=1.0`
  - `route_cache_hit=1.0`

## Reading
- The fixed translated product stack is now confirm-stage system-positive even
  at `Q01` under the persisted-bank assumption.
- The win does not come from changing the geometry or search signal:
  - routed top-1 stays fixed
  - candidate fraction stays fixed
  - cache reuse removes the one-time build cost that had been blocking the
    earlier onset
- The next honest question is where this `Q01` warm-cache onset begins in bank
  size, not whether the `T40000` point is real.
