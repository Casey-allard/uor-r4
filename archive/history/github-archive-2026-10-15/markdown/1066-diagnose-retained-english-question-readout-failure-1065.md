# #1066 — Diagnose retained English question-readout failure (#1065)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-02T21:39:39Z
- **Updated:** 2026-10-09T14:16:51Z
- **Closed:** 2026-09-02T21:41:05Z
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1066
- **Merged:** 2026-09-02T21:41:05Z
- **Merge commit:** e1d3c431ce520a59572e9158828fecef50e4f793
- **Base / Head:** main / UOR-Foundation:issue-1065-english-diagnostic
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The retained English model rarely changes its answer when the question changes. This construction-only diagnostic reproduces #1063's full score/logit/prediction/attention record exactly, then separates factual confounds, displayed positions and matched question/location responses with explicit eligibility denominators. The frozen result selects `QUESTION_READOUT`: 3,974/4,096 question pairs retain the same prediction, only 20 pairs are both correct, and no overall position or attribute-confound majority fires.

The model selects a supplied location in 6,905/8,192 rows and UNKNOWN in 1,287; accuracy remains 2,396/8,192. Raw results and exact fresh-process replay are included. Combined execution was 3.43 seconds with 0.775 GiB peak RSS. This performs zero training, new development decisions, development/checkpoint/frame payload reads, or geometry changes. The established #1059/#1061 attention results remain intact.

Validation: 11 named focused checks; independent source, evidence and documentation review; exact 13-field source reproduction; exact complete-evidence replay in a separate process; unchanged model/source identities. Broad QA remains dormant. Required queue checks acknowledge transport only and are not test evidence. Executable/preparation were published before inference at 58bd1559/6bf314b6.

The current summaries now recommend one separately frozen fresh fit changing only the supervised answer readout from colon position 40 to query-object position 37, with matched cell/construction/seed/dose and both question types reported. This is an unproven follow-up hypothesis, not work performed by this PR; new transfer claims require new frozen development data.

Record: `docs/r4_zoology_english_diagnostic_1065.md`. Result CID: `blake3:65b23631b10fe62b215411932cd9fe45f76b43d6b8503d0f2e74dc3d256c9b61`. Replay CID: `blake3:7222a680c300552ab097ce184500c90c0e44ede8248c4c3f752aa09f4232c0ca`.

Closes #1065. References #973 and #820; #973 remains open and #954 blocked.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

