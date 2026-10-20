# #1266 — Learn typed completion for unresolved language continuations

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-14T03:59:10Z
- **Updated:** 2026-10-09T13:17:11Z
- **Closed:** 2026-09-14T04:00:14Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1266
- **Merged:** 2026-09-14T04:00:14Z
- **Merge commit:** 33b5c199d7d74911db648bc061432123cda6caa5
- **Base / Head:** main / UOR-Foundation:codex/continuation-completion
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The retained language scheduler emitted an intermediate answer plus EOS when another required question had no compatible or unique continuation. A sealed frozen-artifact diagnostic reproduced this on all 1,024 unresolved rows. The new completion policy observes pending-clause presence separately from route availability and learns a typed Unresolved action; only one warm-started policy row changes. Reader, updater, writer and the shared transition executor remain unchanged.

PASS_TYPED_LANGUAGE_COMPLETION: 896/896 training and development targets, including 384 correct answers and 512 unresolved outcomes. Removing pending information or the unresolved action loses all 512 unresolved successes. All 2,816 earlier successful Full core traces transfer exactly. Legacy control replay is separately verified through the unchanged parent. No global-absence, general-prose or independent-final-holdout claim; model15baec48 stays retained and unpromoted.

Validation: optimized offline core compilation; 40 focused tests; actual sealed diagnostic and single fit/evaluation; artifact reload/malformed checks; cargo fmt --check; claim wording and git diff --check. Preserved all 49 earlier sealed roots / 1,234 files, frozen predecessor source/executable and original dirty checkouts. Historical queue statuses are compatibility acknowledgements, not tests.

Model/build/diagnostic/fit/controls cost 376,916 ms under the 1,800,000 ms projection. The parent storage allowance was extended by 256 MiB before work under standing authorization, retaining the 128 MiB margin. No external cost or cleanup. Current state, roadmap, evidence and local restart notes record the next bounded multiword-answer step.

References #973.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

