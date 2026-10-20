# #1665 — Measure native geometric readout credit and calibration directions

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-03T22:40:15Z
- **Updated:** 2026-10-09T13:17:28Z
- **Closed:** 2026-10-03T23:22:02Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1665
- **Merged:** not merged
- **Merge commit:** 71449445493e03e3fda685886d917589f4ecabb2
- **Base / Head:** main / UOR-Foundation:codex/geometric-q4-readout-direction-20261003
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The trained geometric source realizer often ranks the wrong Copy token. This adds a Rust diagnostic that reuses its saved checkpoint and existing prepared learner, checks selected analytic credit, and measures ten independent legal q4 readout changes with context and terminal scores fixed. It performs zero optimizer updates and adopts no candidate.

All five checked adjoints agree with independent reconstruction (max absolute gap1.28e−8). Four geometric coordinates have native directions that lower joint and conditional Copy CE, but Copy-only top1 stays20/74; some descents worsen joint Copy/Period decisions. One descent preserves baseline46/114 correctness, while an opposite direction raises CE but reaches48/114. Candidate own-prefix generation is NOT_RUN. These findings support one context-frozen continuation of the existing Copy/Period/Stop readouts under unchanged answer CE, with generated replies and stage regressions accompanying checkpoint assessment.

Validation: six focused Rust tests and optimized build pass at executed source7a517dff; CPU-only Linux diagnostic exits0 in30.399s. Independent reviews reconstruct1,140 candidate rows, selected gradients, all119 returned hashes and ten exact single-q changes, unchanged context/terminal scores and frozen other packed payloads. Final code/dependencies equal executed source; fresh-main reconciliation adds documentation/CI only. Formatting, claim-wording and diff checks pass. Metal unavailable on Linux and not counted. No GPU, GitHub research runner or shared session change.

Current-state, roadmap and source-bound evidence are updated. The Runpod job card is released and ownedarea is about5.21GiB; results returned locally. References #1552 and #1512. No complete-chat, capacity or held-out qualification.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5974554244 — 2026-10-03T23:22:01Z — Casey-allard

Superseded by #1666, which includes all direction-diagnostic source/report/evidence plus the executed frozen-context readout continuation and current roadmap. Both are delivered to main through the new independently reviewed protected PR. Original branch and all research artifacts are preserved; this closure is supersession, not a merge claim.


---
_Comment/review ids in this file: 5974554244_

