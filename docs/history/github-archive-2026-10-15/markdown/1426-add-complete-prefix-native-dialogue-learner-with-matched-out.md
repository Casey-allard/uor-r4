# #1426 — Add complete-prefix native dialogue learner with matched output witness

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-27T08:28:04Z
- **Updated:** 2026-10-09T13:17:15Z
- **Closed:** 2026-09-27T08:29:15Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1426
- **Merged:** 2026-09-27T08:29:15Z
- **Merge commit:** c046cf19bd856eb9087e78ee3b5d60cd146f29ae
- **Base / Head:** main / UOR-Foundation:codex/dialogue-prefix-learning-20260927
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The retained R1d dialogue learner previously sampled windows that often omitted the response’s original request/history, and mainline could replay its wide checkpoint but could not carry a complete-prefix update through saved reload. This adds one integrated offline Rust path: exact complete-response episodes, full-prefix versus role-only conditioning with identical response/EOS exposure, global supervised-token gradient reduction, fresh parameter-only initialization, strict wide checkpoint/Adam reload and exact-ID multi-turn generation.

Two sequential two-update witnesses completed from source `1af89bf5`: each sees the same 32 response visits and 1,991 supervised targets; each reload reproduces its last-batch loss with zero delta and retains parameter/Adam fingerprints. All 10 actual replies are preserved and independently reviewed. Neither arm answers Paris or recalls Alex in the final memory turn. This is integration evidence; the substantive fixed 1,024-update study remains NOT_RUN. Canonical/integer loaders and the native multiplier-free serving target retain their contracts.

Validation: 8 focused Rust checks pass (4 episode, 2 offline-checkpoint, 1 unequal-token shard/padding-causality, 1 retained canonical checkpoint), final release build, both complete-file-set verifications, 51 publication file bindings, formatting/diff and claim-wording checks. Model processes took 40.528 seconds combined; both builds/check passes took 124.796 seconds. Resource receipts include failed preflight orchestration and preserve all artifacts. Next-update resume equivalence was not executed.

The result packet, current state and roadmap select a separately projected paired learning study with explicit source-stratified open development, actual request/history output and no automatic dose/seed/decoder sweep. Other labs’ trainer/CLI/stack ownership is preserved.

References #973. References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

