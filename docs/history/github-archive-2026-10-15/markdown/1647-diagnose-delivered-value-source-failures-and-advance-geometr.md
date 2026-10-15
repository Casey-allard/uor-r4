# #1647 — Diagnose delivered-value source failures and advance geometric consumer

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-03T05:19:03Z
- **Updated:** 2026-10-09T13:17:30Z
- **Closed:** 2026-10-03T05:57:08Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1647
- **Merged:** 2026-10-03T05:57:07Z
- **Merge commit:** b482a161ae51d061f8e86588a27d6dbde6825d85
- **Base / Head:** main / UOR-Foundation:codex/geometric-source-binding-diagnostic-20261003
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The retained chat report could not distinguish wrong recall from wrong source consumption because it omitted recalled bytes and actual emitter inputs. This standalone Rust diagnostic now captures those values and executes fixed selected-value × distractor contrasts with predicted compiler/store and each arm’s own generated history.

All20 read-on cases receive the correct value in the current input;16 satisfy the frozen complete-answer set, and4 answer with the distractor instead. Read-off is0/20 under that finite set, but5 replies mention the selected value. This is exposed development evidence, with own-history and BPE-segmentation limitations, not general-chat qualification or native serving. There were176 turns, zero updates,184.366s worker time on one low-priority CPU thread/no GPU, and~153MiB maximum sampled process-tree RSS.

The roadmap now advances to a learned geometric selected-record occurrence consumer with exact BPE identities and NoRead/Generate/Copy/Stop, rather than an unchanged capture/scale sweep. Full final-input/action/record/reply traces are included in the evidence extract; preceding outcomes remain sealed locally. No shared session, model training, corpus or peer-worktree edit.

Validation: independent exact-source review3b95af0e; hosted M1 run37100025252 passed formatting, two focused Unicode-span/complete-answer tests and release compilation. Exact tested driver SHA f50ec54688523584a4fb353b15f3e03e8ca2db08d0c4e339ef496e2b93be1c04 is unchanged at final delivery head6fb0dd4a. Later changes add only result documentation/extract and roadmap. Full report SHA f60171bbcbab23c46fca71f6864834223f9164bc17a553e4e9f92de334d1f089. Independent outcome/document review APPROVED exact delivery head6fb0dd4a; every committed evidence row was checked against the full report. Five required historical CI statuses are queue acknowledgements, not tests.

References #1552 and #1512. Existing capability gates remain unchanged.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5966104305 — 2026-10-03T05:51:15Z — Casey-allard

Independent principal engineer APPROVED exact delivery head `6fb0dd4a6959c0ec69c4618d1f2273f658c98d99`: numerical source is unchanged from compiled/tested `3b95af0e`; all40 extract rows match the sealed full report, predicted actions/store bytes/current suffixes/actual histories and scorer flags were independently checked. Reviewed result/roadmap preserve own-history, finite-scoring, BPE-segmentation and hybrid/native limitations. No remaining material finding; no extra model run requested.

Executed checks are the hosted M1 release build/two focused tests at run37100025252 and the actual40-arm CPU diagnostic, not the five queue acknowledgement statuses. Full local review retained in result-review.md. Advisory delivery-envelope status is separate and must not be reported as model validation.



---
_Comment/review ids in this file: 5966104305_

