# #1234 — Add tied categorical learner and record failed method gate

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-13T01:21:41Z
- **Updated:** 2026-10-09T13:17:32Z
- **Closed:** 2026-09-13T01:22:54Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1234
- **Merged:** 2026-09-13T01:22:54Z
- **Merge commit:** 92d227e747a7e6536ba3e1074d47665c416eb81c
- **Base / Head:** main / UOR-Foundation:codex/geometric-tied-learning
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The previous occurrence-credit approximation missed changes in future parameter accesses. Add a bounded Rust categorical learner that holds each sampled pair fixed across complete hard trajectories and exports only discrete parameters; preserve older artifacts and bind the new fit provenance. The integer serving kernel is unchanged.

The frozen experiment **fails** its method gate: learned sampling improves its exploratory expected loss in4/4 seeds and beats coordinate search4/4, but beats fixed joint sampling0/4 (three ties, one loss). All exports change only read1930; complete construction accuracy regresses79→67/672 and all Full continuations remain incoherent. Retain15baec48; no promotion, fresh holdout or full-model campaign. Record all artifacts, controls, oracle costs, resources and the next full-construction-objective comparison.

Validation:27 focused release shared-core tests on final source; one actual sealed four-seed comparison with equal2,050 calls per arm and14,400 oracle settings; actual-candidate512-step zero-allocation census; formatting and claim-wording checks. Independent source/result review completed. Prior55 sealed files across9 roots and parked22 dirty paths preserved. Failed compilation and final rebuild charges retained:299,956/540,000ms total; no paid compute.

References #973 and #964. Programme #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

