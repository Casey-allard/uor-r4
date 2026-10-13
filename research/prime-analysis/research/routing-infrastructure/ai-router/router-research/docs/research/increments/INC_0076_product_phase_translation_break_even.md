# INC-0076: Product Phase Translation Break-Even

## Status
Confirm completed positive/narrow on 2026-03-11.

## Trigger
`INC-0075` closed negative for bounded quality rescue on the fixed dense-frontier
systems lead. That means the next kill-list-consistent question is not another
retrieval tweak. It is whether the fixed product branch now crosses dense exact
retrieval on amortized repeated-query cost at a practical repeat count.

## Branch Contract
- keep the confirmed `INC-0065` product route law fixed
- keep the confirmed `INC-0071` secondary-key law fixed
- keep the confirmed `INC-0072/0073/0074` translated systems and dense-frontier
  surfaces fixed
- change only `query_repeats`
- do not reopen geometry, phase coupling, routing, or retrieval scoring

## Minimal Scope
1. Screen repeat counts:
   - `Q01`
   - `Q08`
   - `Q16`
   - `Q24`
   - `Q32`
2. Carry only:
   - `DENSE`
   - `H4XH4_FIELD_A150`
   - `H4XH4_FIELD_A150_CPX8`
3. Confirm only the crossover bracket:
   - `Q08`
   - `Q16`
   - `Q24`
4. Judge:
   - `top1`
   - `cand_frac`
   - `online_per_repeat`
   - `amortized_per_repeat`
   - `fallback`

## Working Hypothesis
The fixed product branch already holds a major pruning and online-cost edge. If
the offline build is amortized over enough translated queries, the branch should
cross dense exact retrieval on total amortized cost without needing any new
geometry.

## Evidence
- Screen config:
  - `configs/proxy_transfer_inc0076_product_phase_translation_break_even_screen.json`
- Confirm config:
  - `configs/proxy_transfer_inc0076_product_phase_translation_break_even_confirm.json`
- Screen analysis:
  - `results/analysis/inc0076_product_phase_translation_break_even_screen.json`
- Confirm analysis:
  - `results/analysis/inc0076_product_phase_translation_break_even_confirm.json`
- Gate notes:
  - `docs/governance/gates/gate_20260311_145722.md`
  - `docs/governance/gates/gate_20260311_151013.md`

## Screen Read
- The 2-seed screen showed the crossover region cleanly.
- `Q08`
  - `DENSE_Q08`: `top1=0.04833`, `amortized=1.438s`
  - `H4XH4_FIELD_A150_Q08`: `0.05092`, `1.855s`
  - `H4XH4_FIELD_A150_CPX8_Q08`: `0.04892`, `1.642s`
  - No routed crossover at `Q08`.
- `Q16`
  - `DENSE_Q16`: `top1=0.04833`, `amortized=1.341s`
  - `H4XH4_FIELD_A150_Q16`: `0.05092`, `1.168s`
  - `H4XH4_FIELD_A150_CPX8_Q16`: `0.04892`, `1.010s`
  - First clear crossover on both routed points.
- `Q24`
  - `DENSE_Q24`: `top1=0.04833`, `amortized=1.328s`
  - `H4XH4_FIELD_A150_Q24`: `0.05092`, `0.921s`
  - `H4XH4_FIELD_A150_CPX8_Q24`: `0.04892`, `0.827s`
- `Q32`
  - `DENSE_Q32`: `top1=0.04833`, `amortized=1.306s`
  - `H4XH4_FIELD_A150_Q32`: `0.05092`, `0.816s`
  - `H4XH4_FIELD_A150_CPX8_Q32`: `0.04892`, `0.734s`
- The screen justified confirm on the crossover bracket `Q08/Q16/Q24`.

## Confirm Read
- The 4-seed confirm preserved the crossover.
- `Q08`
  - `DENSE_Q08`: `top1=0.04912`, `amortized=1.377s`
  - `H4XH4_FIELD_A150_Q08`: `0.04912`, `1.838s`
  - `H4XH4_FIELD_A150_CPX8_Q08`: `0.04867`, `1.691s`
  - No routed crossover at `Q08`.
- `Q16`
  - `DENSE_Q16`: `top1=0.04912`, `amortized=1.322s`
  - `H4XH4_FIELD_A150_Q16`: `0.04912`, `1.163s`
  - `H4XH4_FIELD_A150_CPX8_Q16`: `0.04867`, `1.036s`
  - First confirm-stage routed crossover.
- `Q24`
  - `DENSE_Q24`: `top1=0.04912`, `amortized=1.328s`
  - `H4XH4_FIELD_A150_Q24`: `0.04912`, `0.931s`
  - `H4XH4_FIELD_A150_CPX8_Q24`: `0.04867`, `0.831s`
  - The crossover strengthens at the existing dense-frontier repeat count.

## Reading
- The fixed product branch now has a confirmed amortized cost crossover against
  dense exact retrieval.
- The earliest confirmed practical crossover is `Q16`.
- The two useful routed points are now distinct:
  - `H4XH4_FIELD_A150_Q16` is the quality-matched break-even point
  - `H4XH4_FIELD_A150_CPX8_Q16` is the first stronger-pruning systems crossover
- `Q24` remains the stabilized dense-frontier systems point:
  - deeper amortized margin
  - the same small top-1 regression on the secondary-key branch
- This is still narrow evidence:
  - translated proxy pipeline only
  - fixed dataset size
  - no direct FLOP or memory-traffic audit yet

## Decision
- Close `INC-0076` confirm positive/narrow.
- Promote `H4XH4_FIELD_A150_Q16` as the first quality-matched break-even point.
- Promote `H4XH4_FIELD_A150_CPX8_Q16` as the first systems crossover point.
- Keep `H4XH4_FIELD_A150_CPX8_Q24` as the stabilized dense-frontier systems
  point.
- Move next to hardware-cost profiling and scale mapping on these fixed
  crossover points instead of any new retrieval rescue or geometry branch.

## Resume Note
Resume from the confirmed `INC-0076` artifacts, not from `INC-0075`. The live
question is now hardware-cost slope and scaling on the fixed crossover points.
