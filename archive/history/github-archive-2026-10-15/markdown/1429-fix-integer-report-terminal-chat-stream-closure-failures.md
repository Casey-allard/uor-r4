# #1429 — fix(integer): report terminal chat stream closure failures

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-27T10:12:53Z
- **Updated:** 2026-10-09T13:17:08Z
- **Closed:** 2026-09-27T14:48:15Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1429
- **Merged:** 2026-09-27T14:48:15Z
- **Merge commit:** 9657b503029c699a31f9cd0ef784230545d38446
- **Base / Head:** main / UOR-Foundation:codex/chat-stream-errors-20260927
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

A failed terminal model write could be discarded by `ChatTokenStream`, and `uor-chat` treated iterator exhaustion as successful generation without checking the retained error. The stream now records `ModelError` for failed closure writes; the CLI reports that cause and exits 1 before success telemetry. Successful closure keeps the same state updates and sampler behavior, and telemetry identifies the actual stop reason.

Two focused synthetic library regressions verify closure failures and successful EOS/turn/custom/cap/cycle state, history and RNG preservation against direct stepping. One CLI completion-helper regression passes, and the optimized CLI builds. `cargo fmt --check -p uor-r4-integer`, claim wording and diff checks pass. The source-bound receipt preserves the initial test-only slice-coercion compile failure and its correction; all build/check attempts total 34.816 seconds.

The CLI test exercises its completion helper, not a spawned exit-status process. Existing stream-construction errors remain visible and return to the prompt; exit 0 alone still does not prove every earlier command succeeded. No rollback/recovery policy or learned-model execution is added, and no chat-quality claim follows. Google PR #1419's reviewed geometry/cache hunks remain separate; the frozen dialogue study and packed-runtime comparison binaries are unchanged.

References #973. Coordination: #1419.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

