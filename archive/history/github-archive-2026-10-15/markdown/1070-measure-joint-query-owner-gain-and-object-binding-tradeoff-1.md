# #1070 — Measure joint-query owner gain and object-binding tradeoff (#1069)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-02T23:26:43Z
- **Updated:** 2026-10-09T14:16:51Z
- **Closed:** 2026-09-02T23:28:58Z
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1070
- **Merged:** 2026-09-02T23:28:58Z
- **Merge commit:** 87c1610a7e58980e82eaf14d1ab1852bfc021fac
- **Base / Head:** main / UOR-Foundation:issue-1069-joint-query
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The retained query-object readout rarely selected the correct owner in matched questions. This PR adds and completes one fresh matched experiment that supplies the owner word embedding directly at the existing object/readout position, before unchanged source dropout and attention. Model parameters, all 41 input tokens, construction bytes/labels, seed, sampler, optimizer and 3,920-update dose are preserved.

Construction accuracy rises from **3,735/8,192 (45.5933%) to 4,118/8,192 (50.2686%)**, while owner-changing both-correct pairs rise from **47 to 338/2,048**. Object-changing both-correct pairs fall from **447 to 376/2,048**, so the frozen result is **JOINT_QUERY_PRESERVATION_MISS**. The record preserves the gain and regression and retains #1067 as the reference.

## Evidence and scope

- Source and preparation identities were published before fitting. The model envelope binds the parameter-free query encoding; fresh fit, resume and checked reload apply the same operation.
- One completed 3,920-update fit, with the exact prior 2,007,040-presentation ledger and no extra dose or checkpoint selection.
- Strong position dependence remains: target-slot accuracies **35.8887%, 36.0352%, 29.4922%, 99.6582%**. Slot 4 gains 690 correct answers while slots 1–3 collectively lose 307; this is not general position robustness.
- Construction misses 8,111/8,192, leaving fresh development at **zero model decisions**, NOT_RUN_CONSTRUCTION_MISS. R4 is NOT_RUN_SEPARATE_INFERENCE_STEP.
- Exact fresh-process complete-evidence replay; immutable model state, zero future attention and no replay training. Combined **286.89 seconds**, peak **0.7856 GiB**.
- **Twelve focused methods passed**, with independent source, preparation, evidence and documentation review. Broad QA remains dormant; queue statuses acknowledge transport only.

Raw preparation/fit/result/replay envelopes and the appended record retain all identities. Six current claim mirrors record the tradeoff and recommend one separately frozen cyclic fact-order augmentation fit on the plain #1067 readout, preserving the full task and fixed dose. Historical attention positives remain intact; #973 stays open and #954 blocked.

Record: docs/r4_zoology_joint_query_1069.md

Closes #1069
References #973, #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

