# #1238 — Measure conditional decoder regularization; preserve failed model gate

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-13T02:49:00Z
- **Updated:** 2026-10-09T13:17:23Z
- **Closed:** 2026-09-13T02:50:33Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1238
- **Merged:** 2026-09-13T02:50:33Z
- **Merge commit:** f66efc02eba6e0d044ea82949599605f6473d233
- **Base / Head:** main / UOR-Foundation:codex/geometric-decoder-validation
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The previous angular emitter improved construction fit but worsened opened likelihood. This change executes one frozen conditional decoder-selection experiment: four depth/support configurations, three document folds and one final refit. It selects depth3/minimum-leaf16, yielding 8f8e5c34 with 36 trees / 184 nodes. Production training, artifact and serving source remain unchanged.

**FAIL_DECODER_VALIDATION_DEVELOPMENT_GATE; no promotion.** Opened Full NLL improves to 3.653604 and accuracy is 52/381, below the previous tree's 57. Preservation fails with 195/180/199 correct-to-wrong rows against the three priors separately. All four Full generations remain incoherent and hit 96 bytes without EOS. Retain 15baec48. Fold fitting excludes validation labels from incremental decoder fitting, but the fixed parent saw all construction documents; this is conditional selection, not fresh end-to-end qualification.

Executed validation: offline release build, focused document-fold/weighted-selection/tie test, single sealed actual experiment, source-identical actual-candidate 512-step zero-allocation/bounded-comparison census, formatting, claim wording, links and source/report hashes. All 9,477 prior rows and 36 prior outputs replay exactly; 13 prior sealed roots / 111 files and the parked worktree are preserved. Model/build/test charge 97,800 ms; no allowance extension or external spending. CI compatibility acknowledgements are not model tests.

The evidence, current-state pointer and independent review recommend one bounded complete-construction recurrent/read-pair intervention with this decoder fixed. No additional model fit or promotion is included.

References #973, #964. Programme #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

