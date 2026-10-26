# ORDER: r-ranking-with-admission-intact, kimi-44

Lab: kimi. Item: [r-ranking-with-admission-intact](../../../../roadmap/items/r-ranking-with-admission-intact.md). Base: `87024eca8`. Supersedes kimi-43's unexecuted full-dose plan under the D68 review recorded in `../kimi-43/DECISION.md`.

## Goal

Close #2884 on its unchanged diagnostic done-when using the already sealed #2831 Q1 run and the recorded count-table comparisons. Lead with the same-admitted-row unigram loss and the admission ceiling. Do not claim the six full-dose runs, independent-seed replication, fresh transfer, above-unigram ranking or M1 progress.

The item's done-when remains:

> A run with admission demonstrably intact reports ranking results (top-1 among admitted rows) against the count tables.

## Cancel probe (run first)

Verified the sealed #2831 archive SHA-256 `4a744188f14c934f89e57a1be83ecc8f2cb211620290a6539217837f809bae55`, Q1 evaluation `6b924afa04103467814b6c71773f7df2c7e6df342f045d83434d3dafe8372943`, and Q1 checkpoint `9a7dd77037dc3e239a73a3900c62fad41279a936727b950b9224f11f2b273184`. Host counting of the stored GPU rows (no model execution) reproduces 4,292 admitted rows, 871 top-1, 1,455 top-8 and 972 admitted targets equal to the full-panel unigram argmax token 1. The diagnostic therefore already exists; another Stage 0 or full-dose launch is not needed for this item. `classification: instrument`. **Cancel the six-run plan, not the diagnostic close.**

## Files in scope

- `docs/labs/r-ranking-with-admission-intact/**`
- `docs/loop/tasks/r-ranking-with-admission-intact/**`
- `docs/research/notebook/m1-discrete-attention.md`
- `docs/roadmap/items/r-ranking-with-admission-intact.md`
- rendered `docs/roadmap/README.md`

No code, model, prep-sheet, mechanism or milestone edit. The original FIT and kimi-43 ORDER remain unchanged history.

## Done when

- D67's failed pre-run review and D68's narrow diagnostic-close review are posted on #2884; the D68 owner-log record precedes the status change.
- REPORT and lab record state Q1's 4,292 admitted rows, 871/4,292 top-1, 1,455/4,292 top-8, the 972/4,292 same-row unigram control, full-panel U/T2, the 245-row P_succ drop, and the admission ceiling for T2.
- Stage T is context only: its two seeds are trajectory-identical and neither arm reached Q4. The six-run ladder is `NOT_RUN`; any future version needs a repaired design, new sheet, FIT and D67. The existing #2878 coverage-schedule item remains the related longer-training route, not a claim that it executes this ladder.
- Item status is `done` only at diagnostic scope; roadmap renders, notebook gains one negative row, full gates and local policy pass, and fresh exact-head artifact/evidence review, queue, merge verification and cleanup complete.

## Stop and decide if

- Any statement would call the cross-checkpoint 754/1,920 comparison admission-matched to Q1, call seed 2 independent, or imply full-dose/fresh/M1 success.
- A new GPU run, model evaluation on host, source change, or edit outside this narrowed scope appears necessary.
- Evidence does not reproduce the sealed counts or the report would hide the unexecuted ladder and matched-baseline negative.
