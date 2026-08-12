# INC-0107: Product Phase Sparse Translation Component Stability Audit

## Status
Completed positive/explanatory on 2026-03-12.

## Trigger
`INC-0106` explained the average lower- and upper-bank systems gains, but it
left one practical ambiguity: whether the upper-bank route-query and
retrieval-search deltas were stable enough to guide the next optimization
branch, or whether the average was hiding seed-level sign flips.

## Branch Contract
- keep the confirmed `INC-0065` product route law fixed
- keep the confirmed sparse translated quality/reference points fixed:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500`
  - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
- keep the confirmed sparse translated systems leads fixed:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
  - `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
- reopen translated work only through per-seed component stability on the
  fixed confirm artifacts
- do not introduce new retrieval heuristics, new sparse controllers, or new
  bank/repeat mapping inside this branch

## Evidence
- Analyses:
  - `results/analysis/inc0104_product_phase_soft_sparse_translation_backfill_recovery_confirm.json`
  - `results/analysis/inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_confirm.json`
  - `results/analysis/inc0107_product_phase_sparse_translation_component_stability_audit.json`
- Report:
  - `docs/reports/INC0107_PRODUCT_PHASE_SPARSE_TRANSLATION_COMPONENT_STABILITY_AUDIT.md`
- Tooling:
  - `tools/translated_component_stability.py`

## Read
- Lower soft sparse versus bounded backfill is not seed-uniform:
  - `2/4` seed wins
  - `2/4` seed losses
  - average amortized delta still favors bounded backfill (`-0.046992s`)
  - candidate fraction improvement is stable across all seeds
- Lower continuous product versus bounded backfill is stable:
  - `4/4` seed wins on amortized cost
  - candidate fraction and retrieval-search deltas are both sign-stable
    improvements
  - route-query changes sign across seeds, so it is not the stable mechanism
- Upper-bank bounded backfill remains mean-positive but seed-fragile:
  - versus the soft sparse reference: `3/4` seed wins, `1/4` seed loss
  - versus the continuous translated product reference: `3/4` seed wins,
    `1/4` seed loss
  - candidate fraction improvement is stable across all seeds
  - route-query changes sign across seeds in both upper-bank comparisons
  - route-index build is an average penalty at the upper bank
  - the large upper-bank gains and losses are dominated by retrieval-search
    variability, not by a clean stable route-query savings law
- The bounded backfill systems leads therefore remain valid as mean systems
  leads, but not as seed-uniform improvements over every routed reference.

## Decision
- Close `INC-0107` positive/explanatory.
- Keep the bounded backfill points as the sparse translated systems leads at
  the lower and upper anchors.
- Treat stable candidate-fraction reduction as the most reliable signal across
  seeds.
- Do not use the upper-bank `route_query` delta as direct optimization
  guidance.
- Move next to repeated timing hardening on the exact same fixed sparse
  translated pairs (`INC-0108`) to separate timing noise from genuine
  data-sensitive variance.

## Resume Note
Resume from the fixed lower/upper sparse translated quality/reference points
and the fixed bounded backfill systems leads. The next branch is repeated
timing hardening on those exact points, not a new heuristic search.
