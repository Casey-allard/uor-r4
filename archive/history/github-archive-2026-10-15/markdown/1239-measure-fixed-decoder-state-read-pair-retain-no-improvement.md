# #1239 — Measure fixed-decoder state/read pair; retain no-improvement result

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-13T03:18:55Z
- **Updated:** 2026-10-09T13:17:31Z
- **Closed:** 2026-09-13T03:19:57Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1239
- **Merged:** 2026-09-13T03:19:57Z
- **Merge commit:** 55755ccc26a9c4b21e37c073fc7cda23e56d354e
- **Base / Head:** main / UOR-Foundation:codex/geometric-tree-state-pair
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The selected angular decoder still produces incoherent continuations. This change tests whether the previously identified recurrent/read pair can improve complete-construction behavior with that decoder fixed. One exhaustive 14,400-setting causal evaluation selects unchanged [119,119] as the unique numerical minimum: NLL 2.839534, 166/672 correct; next-best 3.029702. No setting improves loss or exceeds parent accuracy.

**FAIL_TREE_STATE_PAIR_DEVELOPMENT_GATE; no promotion.** The source-bound test-only witness changes zero parameters and exports no new normal model artifact. All six panels and four Full continuations match 8f8e5c34. Opened 52/381 and incoherent 96-byte/no-EOS outputs remain unchanged. Lost-correct totals 0/199/195/180 against 8f8/1198/0f82/ade9 are inherited differences, not new regressions. Production training/runtime/artifact code and decoder stay unchanged.

Executed validation: release build, focused witness-binding/forbidden-field/tie test, single sealed actual experiment, 512-step operation census (maximum 23 prediction angular comparisons, bound 27), formatting, claim wording, links and source/report hashes. Allocation measurement is NOT_RUN in this step. All 12,636 prior rows, 48 prior outputs, 14 sealed roots/158 files and 22 parked worktree paths verify. Model/build/test charge 104,429 ms; local-cycle 200,000 ms extension recorded before use within unchanged shared ceiling; no paid compute or cleanup. CI compatibility acknowledgements are not tests.

The evidence and current-state pointer close this fixed-head pair question. The recorded next hypothesis is one restricted coupled state/emitter coordinate profile with fixed head configuration, exact baseline reproduction and preserved gates; that successor has not run.

References #973, #964. Programme #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

