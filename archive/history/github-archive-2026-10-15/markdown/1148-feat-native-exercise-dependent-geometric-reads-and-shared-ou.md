# #1148 — feat(native): exercise dependent geometric reads and shared output

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-06T04:54:50Z
- **Updated:** 2026-10-09T13:17:51Z
- **Closed:** 2026-09-06T05:24:04Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1148
- **Merged:** 2026-09-06T05:24:04Z
- **Merge commit:** f964ba449b12467a5ee37861e6ebe633583cfd9a
- **Base / Head:** main / UOR-Foundation:codex/recurrent-geometric-attention
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The first learned H4 block used independent reads and disrupted complete responses. This revision lets the first selected H4 value change the second query, then learns one bounded Base/Emit/EOS decision after the inherited predictor. Rust training fits hard route choices and sparse additive output scores; the changed serving kernel uses integer/table operations and fixed scratch. Old artifacts preserve their declared semantics and identity.

The direct comparison is a development negative: angular reaches 120/735 OPEN next tokens versus 144/735 for matched exact-code selection, all eight target continuations fail, and preservation is 6/62 versus 0/62. Disabling the intermediate connection reduces angular to 53/735, establishing sensitivity only. Both exact generated Rust continuations fail compilation. Retain accepted parent 067adbf0; the #1139 handoff remains unmet and #1140 remains subsequent.

The roadmap and current documentation now direct the next revision to existing exact word/relation sources and selected payload operators. The accepted role-reader supplies a working comparator; its completed handoff is not repeated. More route depth and new metrics remain conditional on observed need. The first failed fit, correction, raw outputs and costs remain preserved.

Validation: corrected release CLI/probe build; two allocation tests and the forbidden arithmetic/float source check; full artifact/CLI reproduction on the recorded Rust prompt, including the old conditional artifact; 62 complete disabled-parent generations agree except the declared control label; formatting, architecture policy and claim wording pass. At unchanged Rust source c36720f9, protected CI run 34012685323 passes 124 native unit tests (including all three recurrent tests), 3 context tests, 8 allocation tests and 20 CLI tests. Four other required status names acknowledge compatibility only. The final documentation update goes through current-head protected checks. New relation transfer and final held-out evaluation remain NOT_RUN.

Cycle model work is 35.844/120 seconds including failures, correction, replay and generated-code compilation; cumulative 1446.496/1800 seconds. Corrected build and focused checks fit the 1200-second engineering envelope. No storage increase, deletion or paid compute. See docs/native_geometric_learned_routing_1139.md and its compact evidence for exact source, artifact, complete work and timing scope.

Refs #1139, #1140, #973.

A direct first-decision check confirms identical raw eight-token inputs, complete routes, parent token and shared-output flags for three different required answers (14, Rome, blue). The current interface cannot separate these tasks by score tuning alone. The 3.124-second Rust check reuses the unchanged release library; exact source/output and hashes are included in the evidence.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

