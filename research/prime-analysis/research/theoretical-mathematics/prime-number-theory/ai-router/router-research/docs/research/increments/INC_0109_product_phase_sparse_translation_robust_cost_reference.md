# INC-0109: Product Phase Sparse Translation Robust Cost Reference

## Status
Completed positive/explanatory on 2026-03-12.

## Trigger
`INC-0108` showed that the bounded backfill sparse translated systems leads
remain positive on mean, but repeated wallclock measurements still flip sign
within the same seed at both anchors. Candidate fraction is stable; microtiming
is not.

## Branch Contract
- keep the confirmed `INC-0065` product route law fixed
- keep the fixed sparse translated quality/reference points frozen
- keep the fixed bounded backfill sparse translated systems leads frozen
- reopen translated work only through a robust cost-reference audit on the
  completed `INC-0104`, `INC-0105`, and `INC-0108` evidence
- do not introduce new retrieval heuristics, new sparse controllers, or new
  bank/repeat mapping inside this branch

## Evidence
- Analyses:
  - `results/analysis/inc0104_product_phase_soft_sparse_translation_backfill_recovery_confirm.json`
  - `results/analysis/inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_confirm.json`
  - `results/analysis/inc0108_product_phase_sparse_translation_repeated_timing_hardening_lower_r2.json`
  - `results/analysis/inc0108_product_phase_sparse_translation_repeated_timing_hardening_lower_r3.json`
  - `results/analysis/inc0108_product_phase_sparse_translation_repeated_timing_hardening_upper_r2.json`
  - `results/analysis/inc0108_product_phase_sparse_translation_repeated_timing_hardening_upper_r3.json`
  - `results/analysis/inc0109_product_phase_sparse_translation_robust_cost_reference.json`
- Report:
  - `docs/reports/INC0109_PRODUCT_PHASE_SPARSE_TRANSLATION_ROBUST_COST_REFERENCE.md`
- Tooling:
  - `tools/translated_robust_cost_reference.py`

## Read
- The bounded backfill upper-bank systems point survives the robust reference
  cleanly against both routed baselines:
  - versus the upper soft sparse reference:
    - amortized median `-0.232755s`
    - amortized trimmed mean `-0.109013s`
    - candidate-fraction median `-0.001524`
    - candidate-count median `-60.9563`
  - versus the upper continuous translated product reference:
    - amortized median `-0.270871s`
    - amortized trimmed mean `-0.274464s`
    - candidate-fraction median `-0.001524`
    - candidate-count median `-60.9563`
- The bounded backfill lower-bank point also survives robustly against the
  continuous translated product reference:
  - amortized median `-0.011214s`
  - amortized trimmed mean `-0.014959s`
  - candidate-fraction median `-0.003160`
  - candidate-count median `-7.8996`
- The only remaining ambiguity is lower-bank bounded backfill versus the fixed
  soft sparse translated reference:
  - amortized median `+0.003406s`
  - amortized trimmed mean `-0.020222s`
  - candidate-fraction median `-0.003160`
  - candidate-count median `-7.8996`
  - this is a robust pruning win, but not a clean robust wallclock promotion
- Top-1 remains effectively unchanged under the robust summaries in all four
  comparisons:
  - lower-bank median top-1 delta `0.000000`
  - upper-bank median top-1 delta `0.000000`
- The robust reference therefore narrows the systems story rather than
  revoking it:
  - upper-bank bounded backfill remains the promoted sparse translated systems
    lead
  - lower-bank bounded backfill remains the promoted sparse translated systems
    lead versus the continuous translated product reference
  - lower-bank bounded backfill should be treated as pruning-first rather than
    as a clean robust wallclock leader versus the fixed soft sparse translated
    reference

## Decision
- Close `INC-0109` positive/explanatory.
- Keep `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000` as the
  promoted upper-bank sparse translated systems lead.
- Keep `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500` as the
  lower-bank bounded backfill systems point, but describe it as:
  - robust versus the continuous translated product reference
  - pruning-first versus the fixed soft sparse translated reference
- Treat stable candidate-fraction and candidate-count reduction as the clean
  robust signal across the full sparse translated branch.
- Move next to robust dense-frontier hardening (`INC-0110`) rather than to
  more sparse translated recovery heuristics.

## Acceptance
- produce a stable cost-reference read for the sparse translated systems leads
- keep the repo honest about what is stable signal versus noisy timing

## Resume Note
Resume from the completed `INC-0109` robust cost reference. The next branch is
repeated dense-frontier hardening on the fixed sparse translated points, not
another bounded recovery search.
