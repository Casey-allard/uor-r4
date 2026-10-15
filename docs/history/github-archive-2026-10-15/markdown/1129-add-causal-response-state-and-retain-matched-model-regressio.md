# #1129 — Add causal response state and retain matched model regressions

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-05T06:00:56Z
- **Updated:** 2026-10-09T13:17:42Z
- **Closed:** 2026-09-05T06:23:56Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1129
- **Merged:** 2026-09-05T06:23:56Z
- **Merge commit:** c5673901462017b5a07c18ef9f1f7d545b710f7f
- **Base / Head:** main / UOR-Foundation:codex/native-response-state
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Native response generation previously had no explicit query boundary or committed source occurrence between tokens. Add an opt-in `/5` reader that captures bounded query/posting state, scores one retained source successor, and commits provenance only after the model's predicted token is observed. Rust fitting, generation, CLI/HTTP continuation and versioned session checkpoints share that causal law. An artifact-bound option advances the query's geometric endpoint without widening admission; the default preserves the initial `/5` law and earlier artifacts retain their behavior.

Both matched fits are retained development negatives. On the same corrected source/readout and 6,548 supervised response/EOS targets, exact prose falls from `/4`'s 6/32 to 5/32 for either `/5` layout; Rust remains 0/32. Completion prediction also regresses. Captured-endpoint generation selects zero continuations and advancing-endpoint generation selects one. The existing `/4` remains the stronger matched artifact; no geometric superiority or alpha claim follows. Two separately inspected generated Rust programs compile and execute, but fail assertions (86 versus emitted 92, and 23 versus 120).

The implementation also preserves quantized rollout policy across training resumes, selects epochs using fixed-population correctness/coverage, accounts for the extra continuation alternative, and rejects service checkpoints above its restore limit before writing. The new record, compact evidence and current-state pointer preserve both failures, controls, actual output and cumulative resources. Computed-value writes and useful response composition remain unfinished. References #973; this does not close the native model objective.

Validation:

- 48 native core tests and 15 CLI/service tests, including causal state, mid-response resume, artifact/default compatibility, token-limit continuation, isolation and checkpoint limits.
- Three configurable-context tests and eight joint-probe tests.
- Two kernel/source and allocation checks: eight model variants × ten controls × 1,024 decisions = 81,920 decisions, with zero steady-state kernel allocations.
- Pinned Rust release build and saved-artifact behavior replay; formatting, native architecture policy and claim-wording checks.
- The final release reproduces all three saved artifacts across 1,088 case/control runs and 14,008 target decisions, including family metrics, predictions, output bytes/tokens/stops and response traces.

Delivery uses the protected merge queue. The four historical audit/fuzz/WASM/Gate-C statuses are compatibility acknowledgements; they are not results of those deferred checks.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

