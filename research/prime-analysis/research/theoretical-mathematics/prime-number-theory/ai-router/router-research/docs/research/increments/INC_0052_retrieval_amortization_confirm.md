# INC-0052: Retrieval Amortization Confirm

## Status
Complete.

## Trigger
`INC-0051` showed the first credible amortized crossover:
- `HOPF_RET_P1_Q24`: `amortized_per_repeat_sec=0.5399`
- `DENSE_Q24`: `amortized_per_repeat_sec=0.5545`

The margin is small enough that it needs a tighter confirm before it becomes doctrine.

## Objective
Confirm whether the `HOPF_RET_P1` translated retrieval branch truly beats dense exact retrieval on amortized per-repeat cost at practical repeat counts.

## Minimal Scope
1. Keep geometry fixed.
2. Drop the widened `HOPF_PHI2` retrieval branch from the live frontier.
3. Confirm only matched repeat counts:
   - `DENSE_Q24`
   - `DENSE_Q32`
   - `HOPF_RET_P1_Q24`
   - `HOPF_RET_P1_Q32`
4. Use 4 seeds with the current translated retrieval harness.
5. Promote only if routed amortized cost stays below dense at one practical repeat count without a large quality regression.

## Decision Rule
- Promote translated routed retrieval as a practical systems branch only if the amortized crossover survives 4 seeds at `Q24` or `Q32`.
- If the crossover fails, keep translated retrieval as a research-positive pruning signal but not as an operational path.

## Planned Artifacts
- Config:
  - `configs/proxy_transfer_inc0052_retrieval_amortization_confirm.json`
- Analysis:
  - `results/analysis/inc0052_retrieval_amortization_confirm.json`
- Gate:
  - `docs/governance/gates/gate_20260306_115931.md`

## Confirm Result
- `DENSE_Q24`
  - `mse=0.004321788`
  - `amortized_per_repeat=0.5051s`
- `HOPF_RET_P1_Q24`
  - `mse=0.004324992`
  - `amortized_per_repeat=0.5938s`
  - `cand_frac=0.3511`
- `DENSE_Q32`
  - `mse=0.004321788`
  - `amortized_per_repeat=0.5586s`
- `HOPF_RET_P1_Q32`
  - `mse=0.004324992`
  - `amortized_per_repeat=0.6544s`
  - `cand_frac=0.3511`

## Reading
- The amortized crossover found in `INC-0051` did not survive 4-seed confirm.
- Plain Hopf still keeps the pruning signal and a lower online-per-repeat routed cost.
- That systems gain is not large enough yet to overcome the offline build under realistic repeat counts.

## Decision
- Kill translated retrieval promotion as an operational branch for now.
- Keep translated retrieval as positive evidence that the routing law preserves useful structure under task translation.
- Reopen the next deep geometry branch:
  - `INC_0050_dynamic_h4_state.md`
