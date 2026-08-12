# INC-0049: Retrieval Cost Rescue

## Status
Complete.

## Trigger
`INC-0048` proved that translated routed retrieval preserves candidate-pruning signal, but loses operationally because chart plus routed local search cost dominates.

## Objective
Rescue the translated retrieval path by reducing routed local-search overhead and separating offline chart/index cost from online retrieval cost.

## Minimal Scope
1. Keep geometry fixed.
2. Keep only the translated branches that matter:
   - `DENSE`
   - `HOPF_RET_P1`
   - `HOPF_PHI2_RET_P1`
3. Add a vectorized same-bucket retrieval path for `probe_buckets=1`.
4. Record timing decomposition so offline and online costs are explicit.

## Decision Rule
- Keep the translated path active only if the vectorized `probe_buckets=1` routed branch materially narrows the runtime gap while preserving the existing candidate-pruning ratio.
- If that fails, move next to translation-cost decomposition and amortization analysis rather than new geometry.

## Planned Artifacts
- Config:
  - `configs/proxy_transfer_inc0049_retrieval_cost_rescue_screen.json`
- Analysis:
  - `results/analysis/inc0049_retrieval_cost_rescue_screen.json`
- Gate:
  - `docs/governance/gates/gate_20260306_113201.md`

## Screen Result
- `DENSE`
  - `mse=0.004318726`
  - `top1=0.049667`
  - `total=1.332s`
  - `offline=0.000s`
  - `online=0.879s`
  - `cand_frac=1.0`
- `HOPF_RET_P1`
  - `mse=0.004325216`
  - `top1=0.047500`
  - `total=10.687s`
  - `offline=9.694s`
  - `online=0.401s`
  - `cand_frac=0.3488`
- `HOPF_PHI2_RET_P1`
  - `mse=0.004326332`
  - `top1=0.045500`
  - `total=8.525s`
  - `offline=7.664s`
  - `online=0.299s`
  - `cand_frac=0.3415`

## Reading
- Vectorized same-bucket retrieval materially reduced routed online cost.
- Routed retrieval is now faster than dense exact retrieval on online/query-time only.
- Total wall-clock still loses because offline chart/index build dominates.
- `HOPF_PHI2_RET_P1` is the translated routed online-speed lead.

## Decision
- Keep translated retrieval active.
- Do not promote routed retrieval on single-batch total wall-clock.
- Move next to repeated-query amortization analysis instead of fresh geometry.
- Next increment: `INC_0051_retrieval_amortization.md`.
