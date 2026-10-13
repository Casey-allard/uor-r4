# INC-0073: Product Phase Secondary-Key Large-Load Hardening

## Status
Confirm completed positive/narrow on 2026-03-11.

## Trigger
`INC-0072` is the first branch where the fixed product route law plus the
confirmed secondary-key law is also a translated systems win:
- `H4XH4_FIELD_A150_CPX8` beat `HOPF_K25_BASE_PHI` on top-1
- it pruned much harder
- and it also beat the Hopf translated control on online and amortized cost

The repo’s discipline is explicit: do not overclaim a systems win until it
survives a harder load.

## Branch Contract
- keep the confirmed `INC-0065` product route law fixed
- keep the confirmed `INC-0071` secondary-key law fixed
- keep the confirmed `INC-0072` retrieval implementation rescue fixed
- change only the evaluation load, not the geometry or key law

## Minimal Scope
1. Carry forward:
   - `HOPF_BASE_K25_PHI`
   - `HOPF_K25_BASE_PHI`
   - `H4XH4_FIELD_A150`
   - `H4XH4_FIELD_A150_CPX8`
2. Increase translated retrieval load relative to `INC-0072`:
   - larger `max_train`
   - larger `max_eval`
3. Keep the same route-health gate.
4. Judge:
   - top-1
   - candidate fraction
   - online per-repeat time
   - amortized per-repeat time

## Working Hypothesis
The translated systems win is now real on the current confirm subset. The next
question is whether it survives larger retrieval load, or whether the gain was
only a smaller-subset effect like earlier cost-rescue branches.

## Acceptance
- `H4XH4_FIELD_A150_CPX8` remains health-passing
- keeps a material pruning gain
- and preserves a clearly useful systems tradeoff versus the Hopf translated
  controls under the larger load

## Evidence
- Screen config:
  - `configs/proxy_transfer_inc0073_product_phase_translation_secondary_key_large_load_screen.json`
- Confirm config:
  - `configs/proxy_transfer_inc0073_product_phase_translation_secondary_key_large_load_confirm.json`
- Screen analysis:
  - `results/analysis/inc0073_product_phase_translation_secondary_key_large_load_screen.json`
- Confirm analysis:
  - `results/analysis/inc0073_product_phase_translation_secondary_key_large_load_confirm.json`
- Gate notes:
  - `docs/governance/gates/gate_20260311_134522.md`
  - `docs/governance/gates/gate_20260311_135226.md`

## Screen Read
- The 2-seed larger-load screen narrowed the quality lead but did not kill the
  systems signal.
- `H4XH4_FIELD_A150_CPX8`
  - `top1=0.04892`
  - `cand_frac=0.18418`
  - `online=0.3855s`
  - `amortized=0.9367s`
  - `fallback=0.00325`
- Versus `HOPF_K25_BASE_PHI`:
  - top-1 delta `-0.00033`
  - candidate-fraction delta `-0.16221`
  - online delta `-0.1857s`
  - amortized delta `-0.0978s`
- That was strong enough to justify confirm because the larger-load pruning and
  runtime advantage was still substantial on the fixed law.

## Confirm Read
- The 4-seed confirm preserved the larger-load systems advantage while
  narrowing the quality story further.
- `H4XH4_FIELD_A150_CPX8`
  - `top1=0.04867`
  - `cand_frac=0.19032`
  - `online=0.3804s`
  - `amortized=0.7949s`
  - `fallback=0.00338`
- Versus `HOPF_K25_BASE_PHI`:
  - top-1 delta `-0.00238`
  - candidate-fraction delta `-0.14646`
  - online delta `-0.2486s`
  - amortized delta `-0.2732s`
- Versus `HOPF_BASE_K25_PHI`:
  - top-1 delta `-0.00238`
  - candidate-fraction delta `-0.11382`
  - online delta `-0.1382s`
  - amortized delta `-0.1466s`
- Versus the fixed `H4XH4_FIELD_A150` product baseline:
  - top-1 delta `-0.00046`
  - candidate-fraction delta `-0.12443`
  - online delta `-0.1702s`
  - amortized delta `-0.1613s`

## Reading
- The `INC-0072` translated systems win survives larger translated load.
- The surviving signal is specifically a pruning and systems-cost result on the
  fixed `INC-0071` secondary-key law, not a renewed top-1 frontier claim.
- This keeps the project on the kill-list track:
  - `INC-0071` remains the translated-addressing proof point for the second
    `H^4`
  - `INC-0072` and `INC-0073` now show that the same fixed law also survives as
    a translated systems/pruning surface under harder load
  - the next responsible question is whether that fixed law is compelling
    against dense exact retrieval, not whether new geometry should be invented

## Decision
- Close `INC-0073` confirm positive/narrow.
- Keep `H4XH4_FIELD_A150_CPX8` as the translated secondary-key systems lead.
- Narrow the claim explicitly:
  - the larger-load systems/pruning advantage survives
  - the smaller-load top-1 edge does not

## Next Step
- Move next to `INC-0074`: dense-vs-routed translated frontier evaluation on
  the fixed `INC-0071` secondary-key law and the fixed `INC-0072/0073`
  systems-rescue surface.

## Resume Note
Resume from the confirmed `INC-0073` artifacts, not from the queued state.
Keep the route law, secondary-key law, and systems rescue fixed while testing
the translated frontier directly against dense exact retrieval.
