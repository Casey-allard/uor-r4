# #1067 — A1Q-H: test query-object answer readout with the matched English learning dose

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-09-02T22:28:23Z
- **Updated:** 2026-09-02T22:57:54Z
- **Closed:** 2026-09-02T22:55:46Z
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1067
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Decision

Continue #973 after #1065 / #1066. Run one fresh matched learning experiment changing only the supervised answer readout from constant-colon position 40 to query-object position 37. The diagnostic identified QUESTION_READOUT behavior (3,974/4,096 question pairs invariant), not an internal cause. Test whether this placement is sufficient at the retained dose before increasing geometry.

## Frozen scope

Preserve the exact #1063 construction container, vocabulary, all 10,240 inputs/targets/group/variant/type metadata, full 41-token trunk, source cell/config, seed 123, sampler/dropout random sequence, AdamW, cosine schedule, 2,352 supported plus 1,568 mixed updates, batch 512 and 3,920 total updates. The loader changes only supervised positions 40 to 37. This predicts an explicit answer label at the query object; the literal next input token is '?'. Fresh initialization, never loading old learned/optimizer state. Preserve the old artifact and negatives.

Freeze 256 new development groups (seed 10672), same lexical grammar/held-out owner-object partition and five variants, excluding historical development semantic worlds and all serialized variants regardless of display order. Recreate historical identities from the frozen generator/seed and verify the old CID for exclusion; do not load the old development payload. Freeze/audit new data before fitting. Training sees construction only.

## Predeclared outcomes

- Construction supported score below 8,111/8,192: readout placement is not sufficient under this fixed dose. Retain and report accuracy/NLL, question/location responses and each question type against the saved baseline (2,396 correct; 20/4,096 question pairs both correct, 6/2,048 same-owner and 14/2,048 same-object). Preserve partial gains; new-development model decisions remain zero. No additional dose.
- Construction fits: score the sole final artifact on the new frozen development population. Require 973/1,024 supported answers, 231/256 complete groups, 116/128 per question type, and 244/256 unknown answers. A development miss establishes construction learning only and redirects to compositional transfer.
- All ordinary gates pass: preserve this bounded answer-readout artifact; next separately test unchanged R4 inference preservation. Geometry, R4/frame execution, generation and broader language claims are outside this fit. New development measures fresh combinations in a familiar task, not a new language task.

## Resources and checks

Use the unchanged 1,800-second/4-GiB combined fit/evaluation/replay cap, eight CPU Apple Accelerate threads, one inter-op thread, and first-eight-update admission counted in the dose. Prior fit/evaluation/replay cost was 249.37 seconds; preserving the full trunk preserves work and RNG shape. Every construction row has the answer in the selected position's causal prefix; no reachability restriction excludes the target metric.

Activate only focused construction-position/data-disjointness checks, training-clock matching, conditional-development access/decision checks, final artifact/state binding and fresh-process exact replay, with independent review. Broad QA remains dormant; queue acknowledgements are transport only.

## Done

Publish executable/preparation/data CIDs before training. Complete the single admitted dose, retain model/checkpoint and truthful partial state, evaluate according to the construction branch, and replay exactly without retraining. Append measured evidence and update current claim mirrors, #973, #820 and milestone 21. Deliver via protected PR, verify actual main tree, and close only this experiment.

Owner: @Casey-allard. Completed through #1068 at `385fa32dcaa33e1a34c253e96edea69ee7b93da4`; the matched readout fit and exact replay are complete. See the completion comment and appended record for the construction miss, retained partial gain and next recommendation.

## Comments (2)

### comment 5517425865 — 2026-09-02T22:39:40Z — Casey-allard

The one readout-placement experiment is frozen and published before fitting. Executable/contract commit: `24a8f3cd`, on `issue-1067-query-readout`.

- Preparation: `blake3:b6df21b50fe67696910e8e01ae2aa590c9e9b6aebf15b090cc11b98edc5a82d3`
- Implementation: `blake3:9d368e194b2fad64aebadaf87ca71b4927134b13c33bb949c0e7cc0ca6e81b48` (223 files)
- Dataset manifest: `blake3:da7be1fba1bb02d3891f4c08840cdd368828b28a2e0a19119f08ca8aa44a5f28`
- Fresh development: `blake3:5f20f7f5f9e079e73886bd885cb88dc0948effc73a58d2a111f387ae5604b780`
- Unchanged construction: `blake3:d767fafdf544f01db99d9acb317c76df55e9f9d28f99785d2a6ae62b663731a2`

Eight named focused checks passed, with independent source review. Preparation verified exact copied construction/vocabulary, all labels, 768 fresh variant worlds, zero world/input overlap with historical development or construction, all 64 held-out pairs, and target/slot balance. No fitted model ran during these checks. Historical development identities were regenerated and CID-checked without opening its retained payload.

Next action now: the fresh seed-123 fit, identical numerical trainer and 3,920-update dose, with positions37 replacing40 and the full41-token trunk retained. The first eight updates count in the dose and supply unchanged admission. Final construction must reach 8,111/8,192 before new development is model-scored. Partial construction gains remain reported if it misses. One final evaluation plus exact fresh-process replay follows, within the combined1,800s/4GiB cap. No R4/geometry execution, extra dose or retry.


### comment 5517584555 — 2026-09-02T22:57:53Z — Casey-allard

Completed through [PR #1068](https://github.com/UOR-Foundation/uor-r4/pull/1068) at `385fa32dcaa33e1a34c253e96edea69ee7b93da4`. The delivered main tree `01a085cf7002b0bb054fe7e8fc596c49990e7e9a` exactly matches reviewed head `4295239e7ada76c2ac502f9fa7019e15887925e7`.

DoD evidence:

- Source/contract freeze `24a8f3cd` and preparation publication `7145a01c` preceded the first fit update. Construction/vocabulary bytes and all original metadata were preserved; only returned answer positions changed 40→37, with all 41 input tokens retained.
- Exactly 3,920 updates / 2,007,040 presentations, matching the #1063 ledger (1,846,452 supported; 160,588 unknown). One fresh seed 123 fit, no extra dose or earlier artifact reads.
- Final construction 3,735/8,192=45.5933% versus 2,396=29.2480%: **+1,339 correct / +16.3452 percentage points**, NLL −0.229062326. Object-changing pairs both correct 447/2,048 versus 6; owner-changing 47 versus 14. Wrong-owner/same-object error majority and strong slots 2+4 association remain.
- Terminal QUERY_OBJECT_READOUT_CONSTRUCTION_MISS because 3,735<8,111. New development model decisions 0, NOT_RUN_CONSTRUCTION_MISS; R4 NOT_RUN_SEPARATE_INFERENCE_STEP. No generation or geometry change.
- Fresh-process exact complete evidence replay, PIDs 11644/11650, immutable model state and zero replay optimizer updates. Combined 290.31099 s; peak 0.77635 GiB.
- All 8 named focused checks plus independent source/preparation/evidence/documentation reviews passed. Broad QA NOT_RUN; protected queue statuses were transport acknowledgements.

[Appended record and raw evidence](https://github.com/UOR-Foundation/uor-r4/blob/385fa32dcaa33e1a34c253e96edea69ee7b93da4/docs/r4_zoology_query_readout_1067.md). Result CID `blake3:c6dfcb3a856963ab4493c3d26bf729f6d9cad70147316ef2b9b62e87c3116369`; replay CID `blake3:98c799c7844e36d68b56c6948824c1dacb53fb36e9f38bc7c052ec6fe0873fac`.

Next: one separately frozen fresh matched joint-query fit adding the causal owner embedding at the retained object/readout position, keeping model size and dose. Require owner-changing both-correct improvement and preservation of object/overall behavior; freeze exact criteria and new development first. #973 remains open; #954 blocked. More geometry deferred. This closes only the completed #1067 experiment.



---
_Comment/review ids in this file: 5517425865, 5517584555_

