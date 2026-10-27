# ORDER: m1-close-saved-model, kimi-51

Lab: kimi. Item: [m1-close-saved-model](../../../../roadmap/items/m1-close-saved-model.md). Base: `3b0c0059e`.

## Goal

Repair the prep-sheet section names required by `preprules::tests::the_real_sheets_pass_the_shape_rule`, without changing the executed M1 result, bars, thresholds, evidence or issue state.

## Cancel probe (run first)

Harness CI on #3319 fails only `preprules::tests::the_real_sheets_pass_the_shape_rule`: `docs/harness/prep/m1-close-saved-model-2026-10-26.md` lacks Base artifact, Data split and Steps blocks. Its section 3 already holds checkpoints and corpus, and section 4 holds the GPU protocol. `classification: instrument`. **Not cancelled: rename/split those headings only.**

## Files in scope

- `docs/harness/prep/m1-close-saved-model-2026-10-26.md`
- `docs/loop/tasks/m1-close-saved-model/kimi-51/**`

## Done when

- The sheet has Base artifact, Data split and Steps sections, with all result text unchanged.
- The real-sheet shape tests and full harness tests pass, plus full loop gates/local policy and REPORT.
- Fresh exact-head review, queue and fresh-main verification complete.

## Stop and decide if

- Any result, bar, threshold, evidence or status needs changing; that is out of scope.
- The render PR needs anything beyond merge-forward after this instrument repair.
