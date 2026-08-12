# INC-0051: Retrieval Amortization

## Status
Queued.

## Trigger
`INC-0049` showed that routed retrieval now wins on online/query-time cost, but still loses on total wall-clock because offline chart/index build is paid inside the same batch.

## Objective
Measure the break-even point for translated routed retrieval under repeated-query amortization over one offline route/index build.

## Minimal Scope
1. Keep geometry fixed.
2. Keep only the translated branches that matter:
   - `DENSE`
   - `HOPF_RET_P1`
   - `HOPF_PHI2_RET_P1`
3. Add repeated-query evaluation over the same built route/index:
   - `query_repeats in {1, 4, 8, 16, 32}`
4. Record:
   - total wall-clock
   - offline cost
   - online cost per repeat
   - amortized total per repeat
5. Promote routed retrieval only if amortized cost crosses dense while keeping the existing candidate-pruning ratio and quality within tolerance.

## Decision Rule
- Keep translated retrieval as an active integration path if a routed branch beats `DENSE` on amortized cost at a practically small repeat count.
- If amortization still does not rescue the path, stop opening retrieval-specific geometry and move next either to dense-side systems baselines or back to the deeper dynamic-geometry branch.

## Planned Artifacts
- Config:
  - `configs/proxy_transfer_inc0051_retrieval_amortization_screen.json`
- Analysis:
  - `results/analysis/inc0051_retrieval_amortization_screen.json`
- Gate:
  - `docs/governance/gates/gate_*.md`
