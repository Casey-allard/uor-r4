# INC-0075: Product Phase Dense-Frontier Quality Recovery

## Status
Confirm completed negative on 2026-03-11.

## Trigger
`INC-0074` confirmed that the fixed product phase-field branch is directly
positive against dense exact retrieval on the translated pipeline:
- `H4XH4_FIELD_A150_CPX8` cut candidate fraction from `1.0` to about `0.1903`
- online and amortized cost both dropped sharply versus dense
- but top-1 regressed slightly versus dense exact and the best routed quality
  points

The next honest question was therefore not new geometry. It was whether the
current dense-frontier systems lead could recover that tiny top-1 gap without
giving back the systems win.

## Branch Contract
- keep the confirmed `INC-0065` product route law fixed
- keep the confirmed `INC-0071` secondary-key law fixed
- keep the confirmed `INC-0072/0073` systems rescue fixed
- keep the confirmed `INC-0074` dense-frontier route set fixed
- change only narrow retrieval-quality rescue surfaces on top of
  `H4XH4_FIELD_A150_CPX8`

## Minimal Scope
1. Carry forward:
   - `DENSE_Q24`
   - `H4XH4_FIELD_A150`
   - `H4XH4_FIELD_A150_CPX8`
2. Test only one or two bounded rescue surfaces that do not reopen route-law
   search.
3. Judge:
   - `top1`
   - `cand_frac`
   - `online`
   - `amortized`
   - `fallback`

## Working Hypothesis
The remaining gap was a small within-candidate ordering problem on top of an
already-confirmed systems win. A bounded rescue surface might have recovered
that gap without erasing the dense-frontier pruning/runtime advantage.

## Evidence
- Screen config:
  - `configs/proxy_transfer_inc0075_product_phase_translation_dense_quality_recovery_screen.json`
- Confirm config:
  - `configs/proxy_transfer_inc0075_product_phase_translation_dense_quality_recovery_confirm.json`
- Screen analysis:
  - `results/analysis/inc0075_product_phase_translation_dense_quality_recovery_screen.json`
- Confirm analysis:
  - `results/analysis/inc0075_product_phase_translation_dense_quality_recovery_confirm.json`
- Gate notes:
  - `docs/governance/gates/gate_20260311_142843.md`
  - `docs/governance/gates/gate_20260311_144445.md`

## Screen Read
- The rerank variants did not improve the systems lead.
- `DENSE_Q24`
  - `top1=0.04833`
  - `cand_frac=1.00000`
  - `online=1.359s`
  - `amortized=1.359s`
- `H4XH4_FIELD_A150`
  - `top1=0.05092`
  - `cand_frac=0.30694`
  - `online=0.486s`
  - `amortized=0.901s`
- `H4XH4_FIELD_A150_CPX8`
  - `top1=0.04892`
  - `cand_frac=0.18418`
  - `online=0.365s`
  - `amortized=0.818s`
- `H4XH4_FIELD_A150_CPX8_R025`
  - `top1=0.04825`
  - `cand_frac=0.18418`
  - `online=0.468s`
  - `amortized=0.862s`
- `H4XH4_FIELD_A150_CPX8_R050`
  - `top1=0.04850`
  - `cand_frac=0.18418`
  - `online=0.414s`
  - `amortized=0.834s`
- The screen justified confirm only on the unchanged frontier trio:
  - `DENSE_Q24`
  - `H4XH4_FIELD_A150`
  - `H4XH4_FIELD_A150_CPX8`

## Confirm Read
- The 4-seed confirm reproduced the `INC-0074` dense-frontier split.
- `DENSE_Q24`
  - `top1=0.04912`
  - `cand_frac=1.00000`
  - `online=1.355s`
  - `amortized=1.355s`
- `H4XH4_FIELD_A150`
  - `top1=0.04912`
  - `cand_frac=0.31475`
  - `online=0.507s`
  - `amortized=0.935s`
- `H4XH4_FIELD_A150_CPX8`
  - `top1=0.04867`
  - `cand_frac=0.19032`
  - `online=0.470s`
  - `amortized=0.893s`
  - `fallback=0.00338`

## Reading
- The bounded retrieval-quality rescue failed.
- The rerank surfaces were not worth carrying.
- The fixed frontier itself did not move:
  - `H4XH4_FIELD_A150` remains the quality-matched routed point
  - `H4XH4_FIELD_A150_CPX8` remains the dense-frontier systems lead
- The remaining question is therefore no longer quality rescue.
- The next honest question is where repeated-query amortization turns the fixed
  product branch into a real cost crossover against dense exact retrieval.

## Decision
- Close `INC-0075` confirm negative.
- Keep the `INC-0074` dense-frontier result as the canonical frontier state.
- Kill bounded rerank quality rescue on the fixed dense-frontier systems lead.
- Move next to scale / break-even mapping on the unchanged fixed frontier.

## Stop Rule
- If bounded rescue attempts cannot recover the top-1 gap without materially
  eroding candidate fraction or amortized cost, stop quality rescue.
- Move next to scale/break-even mapping on the confirmed dense-frontier systems
  lead rather than reopening geometry.

## Resume Note
Resume from the confirmed `INC-0074` dense-frontier artifacts only as the
frontier reference. Do not reopen `INC-0075`; it is closed negative.
