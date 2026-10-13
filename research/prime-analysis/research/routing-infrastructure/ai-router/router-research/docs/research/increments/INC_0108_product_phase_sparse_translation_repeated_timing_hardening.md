# INC-0108: Product Phase Sparse Translation Repeated Timing Hardening

## Status
Completed positive/explanatory on 2026-03-12.

## Trigger
`INC-0107` showed that the bounded backfill systems leads remain valid on mean,
but the upper-bank amortized and component deltas are still seed-fragile.
Candidate fraction improves stably, while `route_query` and
`retrieval_search` change sign across some seed comparisons.

## Branch Contract
- keep the confirmed `INC-0065` product route law fixed
- keep the fixed sparse translated quality/reference points frozen
- keep the fixed bounded backfill systems leads frozen
- reopen translated work only through repeated measurement hardening on the
  exact `INC-0104` and `INC-0105` confirm pairs
- do not introduce new retrieval heuristics, new sparse controllers, or new
  bank/repeat mapping inside this branch

## Minimal Scope
1. Re-run the exact lower- and upper-bank sparse translated comparison pairs
   on repeated timed packets using the same seeds.
2. Measure whether the large upper-bank positive/negative swings reproduce on
   repeat:
   - route-index build
   - query-route
   - retrieval-search
   - amortized total
3. Stop at audit/report stage unless repeated timing hardening exposes a clean
   stable systems delta that justifies a new optimization branch.

## Evidence
- Configs:
  - `configs/proxy_transfer_inc0108_product_phase_sparse_translation_repeated_timing_hardening_lower_prewarm_r2.json`
  - `configs/proxy_transfer_inc0108_product_phase_sparse_translation_repeated_timing_hardening_lower_r2.json`
  - `configs/proxy_transfer_inc0108_product_phase_sparse_translation_repeated_timing_hardening_lower_prewarm_r3.json`
  - `configs/proxy_transfer_inc0108_product_phase_sparse_translation_repeated_timing_hardening_lower_r3.json`
  - `configs/proxy_transfer_inc0108_product_phase_sparse_translation_repeated_timing_hardening_upper_prewarm_r2.json`
  - `configs/proxy_transfer_inc0108_product_phase_sparse_translation_repeated_timing_hardening_upper_r2.json`
  - `configs/proxy_transfer_inc0108_product_phase_sparse_translation_repeated_timing_hardening_upper_prewarm_r3.json`
  - `configs/proxy_transfer_inc0108_product_phase_sparse_translation_repeated_timing_hardening_upper_r3.json`
- Analyses:
  - `results/analysis/inc0104_product_phase_soft_sparse_translation_backfill_recovery_confirm.json`
  - `results/analysis/inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_confirm.json`
  - `results/analysis/inc0108_product_phase_sparse_translation_repeated_timing_hardening_lower_r2.json`
  - `results/analysis/inc0108_product_phase_sparse_translation_repeated_timing_hardening_lower_r3.json`
  - `results/analysis/inc0108_product_phase_sparse_translation_repeated_timing_hardening_upper_r2.json`
  - `results/analysis/inc0108_product_phase_sparse_translation_repeated_timing_hardening_upper_r3.json`
  - `results/analysis/inc0108_product_phase_sparse_translation_repeated_timing_hardening.json`
- Reports:
  - `docs/reports/INC0108_PRODUCT_PHASE_SPARSE_TRANSLATION_REPEATED_TIMING_HARDENING.md`
- Tooling:
  - `tools/translated_repeated_timing_hardening.py`

## Read
- Repeated timing hardening does not recover a clean seed-stable wallclock
  story for the bounded backfill sparse translated systems leads.
- Lower soft sparse versus bounded backfill:
  - all `4/4` seeds flip sign across repeats on amortized cost
  - aggregate amortized delta still favors bounded backfill slightly:
    `-0.006282s`
  - candidate fraction improvement remains fully stable:
    mean delta `-0.003963`
- Lower continuous translated product versus bounded backfill:
  - only seed `0` stays sign-stable improvement across repeats
  - seeds `1`, `2`, and `3` flip sign across repeats
  - aggregate amortized delta still favors bounded backfill on mean:
    `-0.011151s`
  - candidate fraction improvement again remains fully stable
- Upper soft sparse versus bounded backfill:
  - seeds `0`, `1`, and `3` flip sign across repeats
  - only seed `2` stays sign-stable improvement
  - aggregate amortized delta still favors bounded backfill on mean:
    `-0.068437s`
- Upper continuous translated product versus bounded backfill:
  - seeds `0` and `1` flip sign across repeats
  - seeds `2` and `3` stay sign-stable improvement
  - aggregate amortized delta still favors bounded backfill on mean:
    `-0.212149s`
- The stable cross-repeat signal is candidate-fraction reduction, not a clean
  wallclock component law:
  - all four comparisons keep sign-stable candidate-fraction improvement
  - `route_query` and `retrieval_search` remain repeat-mixed on many seeds
  - lower and upper packet timings are therefore still noisy enough that
    microtiming should not drive the next optimization branch directly

## Decision
- Close `INC-0108` positive/explanatory.
- Keep the bounded backfill points as mean sparse translated systems leads at
  the lower and upper anchors.
- Treat stable candidate-fraction reduction as the most reliable systems signal.
- Treat single-packet wallclock component splits as too noisy for direct
  optimization guidance at this granularity.
- Move next to a robust sparse translated cost-reference audit (`INC-0109`)
  rather than to more retrieval heuristics.

## Acceptance
- determine whether the upper-bank variance is mostly timing noise or a real
  data-sensitive search-cost effect
- keep the systems-lead interpretation honest without reopening new heuristics

## Resume Note
Resume from the fixed sparse translated quality/system split and the completed
`INC-0107` stability audit. This branch is now closed; the next branch is a
robust cost-reference audit, not another rescue heuristic.
