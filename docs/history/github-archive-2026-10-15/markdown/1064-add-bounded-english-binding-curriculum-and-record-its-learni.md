# #1064 — Add bounded English binding curriculum and record its learning miss

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-02T20:59:58Z
- **Updated:** 2026-10-09T14:16:51Z
- **Closed:** 2026-09-02T21:01:49Z
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1064
- **Merged:** 2026-09-02T21:01:49Z
- **Merge commit:** 6df75837e55a4ffbbc760bb2bd9650ab27fea433
- **Base / Head:** main / UOR-Foundation:issue-1063-english-binding
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The qualified MQAR checkpoints did not test whether ordinary English questions select the right owner–object fact. This adds one frozen English curriculum with an explicit lexical map, crossed distractors, matched question/location-swap groups, bag-preserving missing-binding cases, and globally disjoint construction/development owner–object combinations. The existing source attention cell and coherent R4 adapter are unchanged.

The fixed 3,920-update fit completed but returned `ENGLISH_BINDING_CONSTRUCTION_MISS`: construction 2,396/8,192 (29.2480%), supported development 218/1,024 (21.2891%), complete groups 0/256, and missing-binding answers 37/256 (14.4531%). Changing the question with identical facts changed only 12/512 predictions. Conditional R4 inference/control remain `NOT_RUN_ENGLISH_BINDING_MISS`. The final model, checkpoint and exact data are preserved; the prior #1059/#1061 attention evidence is unchanged.

The append-only record and six current-status mirrors now recommend a separately frozen, construction-only diagnostic of target/distractor selection and question responses before another fit or geometry change.

Validation:
- Three focused data checks and five focused decision checks passed.
- One fixed fit: 2,007,040 answer presentations; zero development optimizer decisions.
- Exact fresh-process final-artifact replay passed; independent source/evidence review approved the result.
- Combined fit/scoring/replay: 249.3656 seconds; maximum RSS 0.4856 GiB; eight Apple Accelerate threads.
- Preparation binds 199 implementation files, exact data, lexical encoding and the unchanged native frame bundle. No code, dose, threshold or data change after freeze.

Broad research/release QA remains dormant. Required queue acknowledgements are transport only.

Closes #1063. References #973, which remains open; #954 remains blocked.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

