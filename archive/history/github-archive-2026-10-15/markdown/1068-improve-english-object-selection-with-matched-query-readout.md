# #1068 — Improve English object selection with matched query readout (#1067)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-02T22:54:35Z
- **Updated:** 2026-10-09T14:16:51Z
- **Closed:** 2026-09-02T22:55:45Z
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1068
- **Merged:** 2026-09-02T22:55:44Z
- **Merge commit:** 385fa32dcaa33e1a34c253e96edea69ee7b93da4
- **Base / Head:** main / UOR-Foundation:issue-1067-query-readout
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Changing the question rarely changed the retained English model’s answer. This PR adds and completes one matched fresh fit that moves the supervised answer readout from colon position 40 to query-object position 37, retaining all 41 input tokens, construction bytes/labels, source cell, seed, sampler/dropout shapes, optimizer and 3,920-update dose.

Construction accuracy rises from **2,396/8,192 (29.2480%) to 3,735/8,192 (45.5933%)**, a **16.3452-percentage-point gain**; NLL falls by 0.229062326 nats. Object-changing question pairs reach 447/2,048 both correct (previously 6), while owner-changing pairs reach 47/2,048 (previously 14). Wrong-owner/same-object confusion and a strong displayed-position association remain. This preserves substantial partial progress while retaining the frozen terminal **QUERY_OBJECT_READOUT_CONSTRUCTION_MISS**.

## Evidence and scope

- Executable, preparation and fresh development identities published before fitting; unchanged construction/vocabulary files and a new disjoint development population.
- Exactly 3,920 updates / 2,007,040 presentations, with the full work ledger matching #1063. One final artifact, no extra dose or checkpoint selection.
- Construction remains below 8,111/8,192, so new development has **zero model decisions**, NOT_RUN_CONSTRUCTION_MISS. R4 inference is NOT_RUN_SEPARATE_INFERENCE_STEP.
- Fresh-process replay exactly reproduces the complete evidence, with immutable model state and no training. Combined fit/evaluation/replay: **290.31 seconds, 0.7764 GiB peak RSS**.
- Eight focused source/data/trainer/decision checks passed. Independent source, preparation, measured-evidence and documentation review. Broad QA remains dormant; required queue statuses are transport acknowledgements.

The appended record and raw preparation/fit/result/replay envelopes retain all measured identities. Six current claim mirrors now describe the gain, owner/position limitations, unscored development and the next proposed matched owner-plus-object query encoding experiment. Historical #1063/#1065 evidence and qualified #1059/#1061 attention results remain intact. #973 stays open; #954 remains blocked.

Record: docs/r4_zoology_query_readout_1067.md

Closes #1067
References #973, #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

