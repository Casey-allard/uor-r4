# #1069 — A1Q-H: test direct owner-plus-object query encoding at the matched learning dose

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-09-02T23:01:32Z
- **Updated:** 2026-09-02T23:29:50Z
- **Closed:** 2026-09-02T23:28:59Z
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1069
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Decision

Continue #973 after #1067 / #1068. Execute one fresh matched full-task fit adding the causal query owner embedding at query-object/readout position 37: x37=E(token37)+P37+E(token35), before unchanged source embedding dropout and attention layers. All other input positions remain unchanged. #1067 improved construction from 29.2480% to 45.5933% but owner-changing both-correct pairs reached only 47/2048, versus 447/2048 for object changes. Test this explicit joint-query access intervention before increasing geometry.

## Frozen scope

Keep #1063 construction/vocabulary files byte-for-byte, all 10,240 input/target/group/variant/type rows, full 41-token tensors, readout 37, two layers/one head/width 64, tied 4096-token head and 303,744 parameters. Fresh seed 123 initialization; retain optimizer, sampler/dropout shapes and RNG draws, batch 512, supported 2352 plus mixed 1568 updates, total 3920 and 2,007,040 presentations (1,846,452 supported;160,588 unknown). No old model/checkpoint loading, extra dose, new parameter, geometry/frame execution or R4 inference. Preserve all historical artifacts.

Freeze 256 fresh development groups at seed 10692 before fitting, same grammar/held-out pairs/five variants and slot/location balance. Exclude all construction and #1063/#1067 development canonical worlds (base, swapped, absent) and exact input rows. Recreate both historical populations from frozen generators and verify their canonical CIDs without opening retained payloads. New development is scored only if construction reaches the retained 99% criterion.

## Predeclared decisions

The primary bounded behavior improvement requires **owner-changing both-correct >=150/2048**, up from 47: at least 103 additional pairs /5.0293 percentage points. Also require **overall correct >=3735/8192** and **object-changing both-correct >=447/2048**. These are fixed-population effect/preservation criteria, not statistical-significance or full-binding claims. There are 1024 same-object worlds with two owner contrasts each; report 0/1/2 successful owner-pair world counts, question/location responses and per-slot behavior. Do not treat the 2048 pairs as independent experimental units.

- Partial behavior-positive but construction below 8111/8192: preserve this improved joint-query artifact as the next learning baseline and address its remaining error pattern in a separately frozen step; new development 0 model decisions. No extra updates here.
- Owner criterion or either preservation floor misses: reject this explicit owner residual as the next baseline; retain #1067 and redirect to the binding learning recipe. Report all gains/regressions, not just a boolean.
- Construction>=8111/8192: score the sole final artifact on new development. Full binding requires 973/1024 supported,231/256 complete groups,116/128 complete per question type and 244/256 unknown. A transfer miss preserves construction only; full pass permits a separate unchanged-R4 inference preservation step.
- An interrupted fit retains the exact partial artifact/work and actual limit; never report an incomplete fit as a scientific negative.

The owner already lay in the causal prefix. This intervention changes direct access, embedding scale and gradient paths together; it does not establish that owner information was absent or isolate an internal cause. It remains an explicit answer-readout task, not unrestricted next-token generation.

## Resources and named checks

The whole fit/evaluation/replay retains 1800 seconds/4 GiB caps, eight CPU Apple Accelerate threads and one interop thread. #1067 cost 290.31 seconds with 0.7764 GiB peak; all 8192 supported answers can access both query attributes, so no reachability ceiling prevents a gain. The first 8 real updates count toward the dose and supply the unchanged admission projection.

Activate only adapter locality/causality/gradient and state/export identity checks, complete-trainer AST equivalence apart from adapter and output identities, data copying/new-population exclusions/access boundaries, decision/preservation/development-gate checks, final artifact binding and exact fresh-process replay. Independent source/preparation/evidence review. Broad QA stays dormant; protected queue statuses are transport only.

## Done

Publish source/contract/preparation CIDs before fitting. Complete one admitted dose, retain model/checkpoint and truthful evidence, score only the allowed population, replay exactly without retraining. Append results, reconcile six current claim mirrors, #973/#820 and milestone 21; deliver via protected PR and verify main tree and issue closure. Close only this experiment. Owner: @Casey-allard. Completed through #1070 at `87c1610a7e58980e82eaf14d1ab1852bfc021fac`. The single fit, exact replay and record are complete; see the completion comment for the measured preservation miss and the next separate learning recommendation.


## Comments (2)

### comment 5517712514 — 2026-09-02T23:12:32Z — Casey-allard

Source/contract freeze `b3d8dc60` is ready for the single matched joint-query fit. All 12 focused methods passed, and independent source review found no blocking issue.

The prefit [record](https://github.com/UOR-Foundation/uor-r4/blob/issue-1069-joint-query/docs/r4_zoology_joint_query_1069.md) and [preparation envelope](https://github.com/UOR-Foundation/uor-r4/blob/issue-1069-joint-query/docs/r4_zoology_joint_query_1069_preparation.json) bind 238 source files, exact original construction/vocabulary bytes, and the fresh seed 10692 development population. Actual data preparation audited all labels, 768 fresh variant worlds, both historical-world exclusions, slot/location balance and 64 held-out owner-object pairs. No fitted model or retained historical development payload was read; fresh development model decisions are 0.

Preparation: `blake3:e2709d32436f7979aeda795f2ec735d99932cdd1037e7e17321d72a7009ad7e1`
Implementation: `blake3:e5436a8c0d8e2e92312721f54164b05269c5a6138a52f54e4574bb3fa7de1e2d`
Dataset manifest: `blake3:3597f258750242bd1b9482e234b4ed939375d4a32c08c37a76d1a500cfe9e490`
Fresh development: `blake3:cbee180e7e37fe302d9cffd198d7af91ed42ed9851bbbf91e7ee4814d0e0a9b1`

One fresh seed 123 model; x37=E(token37)+P37+E(token35) before unchanged dropout/attention; no added parameters/random draws. Fixed 3,920 updates with matched sampler/optimizer/dose. Primary owner-changing both-correct>=150/2048 plus overall>=3735/8192 and object-changing both-correct>=447/2048. Report partial gain, preservation loss, or below-declared owner gain distinctly. Only construction>=8111/8192 opens new development. No extra fit/dose, geometry, R4 inference or generation.

Retain 1,800-second/4-GiB combined budget and eight Apple Accelerate CPU threads. The first eight real updates provide admission and count in the dose. Prior comparable cost 290.31 seconds, peak 0.7764 GiB. Broad QA stays dormant; queue acknowledgements are transport only.


### comment 5517922648 — 2026-09-02T23:29:49Z — Casey-allard

Completed the single matched joint-query experiment through [PR #1070](https://github.com/UOR-Foundation/uor-r4/pull/1070), merged as 87c1610a7e58980e82eaf14d1ab1852bfc021fac. Main tree 4cf574357512041de34d9522c70eacdb3d2d240f matches reviewed head b232cd59c1320f2bca4fe88b5f287c91beae87e4 exactly.

DoD evidence:

- Source/contract freeze b3d8dc60 and preparation publication d8490459 preceded fitting. The preparation binds 238 implementation files, the exact original construction/vocabulary, fresh seed 10692 development exclusions, and coefficient-one owner 35→query 37 encoding.
- One completed 3,920-update fit; 2,007,040 presentations, comprising 1,846,452 supported and 160,588 unknown. Model size, full 41-token tensors, seed, sampler and optimizer clock match #1067. No additional dose.
- Overall construction 3,735→4,118/8,192 (45.5933%→50.2686%); owner-changing both-correct 47→338/2,048, passing 150; object-changing both-correct 447→376, failing preservation. Terminal **JOINT_QUERY_PRESERVATION_MISS**. Retain all measured gains/regressions and #1067 as the reference.
- Target-slot accuracies 35.8887%,36.0352%,29.4922%,99.6582%. Slot 4 contributes +690 correct; slots 1–3 contribute −307. Their combined accuracy declines 38.8021%→33.8053%; the restricted last-slot positive does not establish position-robust retrieval.
- Construction remains below 8,111/8,192, so fresh development has zero model decisions, NOT_RUN_CONSTRUCTION_MISS. R4 is NOT_RUN_SEPARATE_INFERENCE_STEP. Geometry/frame execution and generation were not run.
- Exact complete evidence replay in separate processes 13611/13869, unchanged model state, zero replay optimizer updates. Combined 286.891776 seconds, peak 0.785583 GiB.
- All 12 focused methods and independent source/preparation/evidence/documentation reviews passed. Broad QA NOT_RUN; queue statuses transport only.

Result CID: blake3:bc1066eb0e9bbf08304ab296ca0c1681b7e8af4b0ea9026945ebef83c7fb9d53. Replay CID: blake3:6a6ad3ce5ef9e9541fec4994006b39d7a15ce52432a1af3cff351f0b9d96fcf2.

[Binding record and raw evidence](https://github.com/UOR-Foundation/uor-r4/blob/87c1610a7e58980e82eaf14d1ab1852bfc021fac/docs/r4_zoology_joint_query_1069.md).

Next recommendation: one separately frozen fixed-dose cyclic fact-order augmentation fit, using plain #1067 readout without the owner residual, the same four-fact task and paired variants. Judge both question types and worst-slot behavior across four cyclic rotations. No such fit occurred in #1069. More geometry deferred. #973 remains open; #954 blocked. This closes only #1069.



---
_Comment/review ids in this file: 5517712514, 5517922648_

