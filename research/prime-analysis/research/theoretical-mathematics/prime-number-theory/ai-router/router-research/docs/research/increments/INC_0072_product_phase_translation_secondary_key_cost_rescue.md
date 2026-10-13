# INC-0072: Product Phase Secondary-Key Cost Rescue

## Status
Confirm completed positive on 2026-03-11.

## Trigger
`INC-0071` confirmed that the second `H^4` can act as a useful discrete
translated addressing field on top of the fixed product geometry:
- `H4XH4_FIELD_A150_CPX8` improved top-1 materially over the fixed product
  baseline
- it also beat the main Hopf translated control on top-1 while pruning much
  harder

The remaining failure is systems-level, not geometric:
- online and amortized time did not beat `HOPF_K25_BASE_PHI`
- the current secondary-key implementation is not cashing out the stronger
  pruning into lower runtime

## Branch Contract
- keep the confirmed `INC-0065` product route law fixed
- keep the confirmed `INC-0071` secondary-key law fixed
- change only the translated retrieval implementation/system surface
- do not reopen route-law search, field coupling, or key-law selection here

## Minimal Scope
1. Carry forward:
   - `HOPF_BASE_K25_PHI`
   - `HOPF_K25_BASE_PHI`
   - `H4XH4_FIELD_A150`
   - `H4XH4_FIELD_A150_CPX8`
2. Optimize only the cost path around translated secondary-key retrieval.
3. Preserve the confirmed quality/pruning behavior while reducing:
   - online per-repeat time
   - amortized per-repeat time
4. Compare against:
   - the fixed product baseline
   - the main Hopf translated control

## Working Hypothesis
The confirmed secondary-key law already gives the right discrete addressing
behavior, but the current retrieval implementation is paying too much grouping
or lookup overhead. If that systems overhead is reduced without changing the
geometry, the pruning win may finally turn into a real translated runtime win.

## Acceptance
- keep `top1` within noise of the confirmed `H4XH4_FIELD_A150_CPX8` result
- keep `cand_frac` materially below the Hopf translated controls
- reduce online or amortized cost enough to make the branch a clearly useful
  translated systems tradeoff

## Implemented Surface
- Added prepared grouped same-bucket retrieval plans in
  `tasks/router_retrieval_eval.py` so repeated queries stop rebuilding the same
  exact-bucket retrieval state.
- Reused eval routing across identical `query_repeats` on the routed retrieval
  path, while still rerunning the actual retrieval search.
- Added regression coverage in `tests/test_router_retrieval_eval.py`.

## Evidence
- Screen config:
  - `configs/proxy_transfer_inc0072_product_phase_translation_secondary_key_cost_rescue_screen.json`
- Confirm config:
  - `configs/proxy_transfer_inc0072_product_phase_translation_secondary_key_cost_rescue_confirm.json`
- Screen analysis:
  - `results/analysis/inc0072_product_phase_translation_secondary_key_cost_rescue_screen.json`
- Confirm analysis:
  - `results/analysis/inc0072_product_phase_translation_secondary_key_cost_rescue_confirm.json`
- Gate notes:
  - `docs/governance/gates/gate_20260311_133356.md`
  - `docs/governance/gates/gate_20260311_133731.md`

## Screen Read
- The systems-only rescue immediately changed the live translated tradeoff.
- `H4XH4_FIELD_A150_CPX8`
  - `top1=0.04767`
  - `cand_frac=0.19046`
  - `online=0.1332s`
  - `amortized=0.3516s`
- Versus `HOPF_K25_BASE_PHI` on the screen:
  - top-1 delta `+0.00017`
  - candidate-fraction delta `-0.15839`
  - online delta `-0.0626s`
  - amortized delta `-0.1910s`
- That justified confirm on the exact same fixed route/key law.

## Confirm Read
- The 4-seed confirm preserved the same systems result.
- `H4XH4_FIELD_A150_CPX8`
  - `top1=0.04708`
  - `cand_frac=0.18723`
  - `fallback=0.00333`
  - `online=0.1356s`
  - `amortized=0.3544s`
- Versus `HOPF_K25_BASE_PHI`:
  - top-1 delta `+0.00025`
  - candidate-fraction delta `-0.16384`
  - online delta `-0.0557s`
  - amortized delta `-0.0803s`
- Versus the fixed `H4XH4_FIELD_A150` baseline:
  - top-1 delta `+0.00258`
  - candidate-fraction delta `-0.12110`
  - online delta `-0.0285s`
  - amortized delta `-0.0647s`

## Reading
- The confirmed geometric/addressing signal from `INC-0071` now cashes out as
  a real translated systems win.
- This is not a new route law. The win came from implementation/system rescue
  on top of the fixed confirmed product route and key law.
- The second `H^4` is now positive on all three linked surfaces:
  - phase/address mechanism
  - discrete translated addressing
  - translated systems cost

## Decision
- Close `INC-0072` confirm positive.
- Promote `H4XH4_FIELD_A150_CPX8` as the translated secondary-key systems lead
  on the fixed product branch.
- Do not jump to a broad moonshot claim yet; first harden the result under
  larger load.

## Next Step
- Move next to `INC-0073`: larger-load hardening of the confirmed translated
  systems lead.

## Resume Note
Resume from the confirmed `INC-0072` artifacts, not from the queued state.
Keep the route law and secondary-key law fixed while stress-testing the systems
win.
