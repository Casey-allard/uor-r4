# #1243 — Record decoder refit failure and reassess geometric attention learning

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-13T05:25:52Z
- **Updated:** 2026-10-09T13:17:20Z
- **Closed:** 2026-09-13T05:26:58Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1243
- **Merged:** 2026-09-13T05:26:57Z
- **Merge commit:** 9ef3739d05d78ce44d898159ba6282bcefa36a9b
- **Base / Head:** main / UOR-Foundation:codex/geometric-context-decoder-refit
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The frozen-state decoder comparison fails its prespecified development gate: all four method contrasts and numeric model gates fail. Adapting the head to varied contexts lowers recombination NLL but loses 19 correct positions; the learned varied state then worsens both loss and accuracy against the matched parent-state refit. All 32 Full continuations remain incoherent at 96 bytes/no EOS. Retain 15baec48; no model promotion.

Four fits completed and the baseline branch payload exactly reproduced 8f8. Attempt-1 exceeded its report capacity after fitting and remains sealed INCOMPLETE. A separate Rust recovery driver validates those same saved heads and completes evaluation with zero refits, preserving parsed equality to completed reports. Both source identities, binaries, attempts and full artifact lineage are retained.

Following the owner's research direction, the current plan now suspends the isolated state/head sequence. A source-linked reassessment of UOR/Prism/NEMESIS, Hamming sparse memory and differentiable LUT/Boolean learning identifies the missing connection between exact objects, learned geometric access, typed results and emission. The next deliverable is an implementation-ready learning specification; this PR does not implement or qualify that proposed route. Production runtime and training source are unchanged.

Validation: two focused release test invocations pass; exact baseline reproduction; original experiment exit 101 for report capacity; evaluation-only recovery passes execution checks and reports the failed model gate; 28,161 prior rows and 120 outputs replay; 141 current source bindings, 44 attempt files, 19 older sealed roots / 689 files and parked material verify. Cargo fmt --check, claim-wording and diff/link checks pass. Allocation NOT_RUN. CI/queue compatibility statuses do not execute tests.

Build/test/model charge 222,406 / 420,000 ms; parent 2,897,050 / 3,200,000 ms; shared 122,647,623 / 132,950,000 ms. Necessary local extensions were projected before use. No paid compute, cleanup or model calls after the owner's research pivot.

References #973, #964, #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

