# #1063 — Learn English supplied-context binding with the qualified attention cell

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-09-02T20:33:44Z
- **Updated:** 2026-09-02T21:05:09Z
- **Closed:** 2026-09-02T21:01:51Z
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1063
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Implement one small English supplied-context binding curriculum using the established Zoology causal-attention architecture, then qualify its frozen final artifact through the unchanged #1059/#1061 coherent R4 inference adapter. This is the next authorized step after #1061 and is a child of #973. Preserve all existing checkpoints, historical terminals, and geometry.

## Frozen task and data
- Fresh seed-123 initialization of the same 64-wide, two-layer, one-head source cell, vocabulary head 4,096, positional capacity 120, tied embeddings/head, source dropout and double initialization.
- An explicit reversible lexical map supplies 16 owners, 16 objects, eight English location words, grammar/punctuation and the answer token `unknown`. No teacher, pretrained tokenizer, prior checkpoint remapping, provider, role labels, or answer parser participates in inference.
- Each input contains four short English facts, followed by a location question and `Answer:`; one supervised answer token, with targets outside inputs.
- Crossed distractors require both owner and object: base facts `(A,x),(A,y),(B,x),(C,z)`, with distinct A/B/C, x/y/z, and four distinct locations.
- Each matched quartet asks two questions of each of two histories; history two swaps their location assignments. Half of groups compare same-owner/different-object questions, half same-object/different-owner questions.
- A fifth matched history removes the queried binding by exchanging the object words in facts zero and three, giving `(A,z),(A,y),(B,x),(C,x)`. It has exactly the same lexical bag, fact count, sequence length, locations and question; its answer is `unknown`.
- Globally hold out owner-object pairs satisfying `(owner_index + object_index) mod 4 == 0`; construction facts/questions, including missing-binding variants, exclude those pairs. All lexical items are represented in construction. Counterfactual variants stay together.
- Freeze 2,048 construction world groups (8,192 supported / 10,240 mixed rows) and 256 development world groups (1,024 supported + 256 unknown rows), deterministic seeds 10631/10632 and canonical development order.
- An independent parser of decoded serialized input IDs verifies labels, split exclusions, absence of conflicting bindings, same-bag counterfactuals, target positions and vocabulary coverage before training. It is a preparation instrument, never an answer path.
- Freeze the complete implementation, lexical map, exact data bytes, predecessor result identities and unchanged native frame bundle before fitting.

## One bounded learning contract
Metric to move: novel-combination English supplied-context answer accuracy, initially unmeasured.
Reachability ceiling: every supported row visibly contains its unique answer fact; four-row swaps make 100% exact supported/quartet accuracy reachable from the input. Missing-binding rows contain no queried pair but retain both query words elsewhere. All question inputs are causal.
Instrument: serialized-input/split/control validation must pass before any optimizer update. The first eight updates of the actual frozen trajectory provide one CPU time/memory admission check; they are retained and count toward the dose.
Learning: batch 512; AdamW lr 0.00046415888336127773, weight decay 0.1, source defaults; 3,920 total updates, first 2,352 supported-only then 1,568 mixed. One cosine schedule, T_max 64 blocks, stepped every 196 updates. Exactly 2,007,040 answer presentations if complete. No development-based checkpoint choice, sweep, adaptive retry, extra dose, or post-reveal tuning.
Execution: one CPU process, eight intra-op threads, one inter-op thread. Checkpoint model/optimizer/scheduler/sampler/RNG and cumulative elapsed time every 16 updates. A resumed fit preserves the same trajectory and spent budget. Maximum combined fitting, final evaluation and fresh-process replay: 1,800 seconds and 4 GiB peak RSS.
Admission: after eight actual updates, project remaining fit from their mean step time with 1.25 safety factor plus a fixed 60-second evaluation/replay allowance, counting elapsed time already spent. If it cannot fit, retain the partial artifact and stop as unavailable; do not restart or sweep.
Prior cost basis: #1057's 3,920 updates at T120/V4096/eight queries took 1,096.9868 s. This task has T41/one query and no larger model; the prior conservative combined estimate is about 1,413.5 s.

## Fixed decisions
Final ordinary-model gates, reported separately:
1. Construction supported answer accuracy >=99% (at least 8,111/8,192).
2. Development supported accuracy >=95% (973/1,024).
3. All four decisions correct in >=90% of development groups (231/256), including >=90% independently for each question-pair type (116/128).
4. Development missing-binding `unknown` accuracy >=95% (244/256).

Only if all ordinary-language gates pass, apply the unchanged R4 adapter to that same final artifact:
- all 1,280 development top-1 predictions identical;
- max full-vocabulary selected-logit delta <=0.005, attention delta <=1e-5, NLL delta <=1e-5;
- unchanged learned tensors/tied head, zero optimizer updates, zero future attention weights/contributing reads.
Then exercise inconsistent source-frame transport separately, preserving support and work; require >=50 percentage-point supported-accuracy loss for strong transport sensitivity. Report its actual result without erasing any successful ordinary/R4 preservation.

Report complete group/error counts, same-question counterfactual answers, same-history question dependence, missing-binding behavior, NLL, learned/data/code/frame CIDs, training dose, resource measurements, and exact independent replay. Development remains unopened to training; only the final artifact is evaluated. Preparation may validate dataset semantics without model outcomes.

If positive: preserve the first bounded English-form binding artifact and propose a separately frozen broader language/context application.
If construction misses: retain the artifact and diagnose learning of this fixed lexical recipe before any scale or geometry change.
If construction fits but development misses: retain the fitted artifact and investigate the compositional-transfer seam separately.
If language passes but R4 differs: isolate numerical/adapter integration without retraining.
If unavailable: preserve partial state and report the resource limit; no forced continuation.

## Definition of done and activated checks
Deliver the named data/instrument, fixed-dose fit and frozen final evaluation, conditional unchanged R4 comparison/control, exact fresh-process artifact replay, append-only measurement record and true current-status mirrors. Independent review covers data leakage, fixed learning clock, budgets, checkpoint/replay identity and conditional claims. Only focused checks for those newly introduced seams are activated; all broad workspace, BDD, no_std, native/wasm, kappa, legacy campaigns and release QA stay dormant. Protected PR queue acknowledgements are transport only.

This work does not establish general English understanding, reasoning, correctness, chat readiness, H4 superiority, more geometry capacity, softmax removal, recurrence, integer/table lowering, or closure of #973/#954.

## Comments (2)

### comment 5516187720 — 2026-09-02T20:48:21Z — Casey-allard

Pre-fit freeze published at commit 09056978 (implementation freeze 25ab0ab2), before any optimizer update or fitted evaluation.

Preparation CID: blake3:c926e16516ef0f1d8242dc0af39a04be46cb082bb6c76590bc73f2717e027ca8
Implementation closure: 199 files, blake3:da90e0ecb9a7fe82d9eaccc3f0c38ae597036d5de5e68539b13746c21388fbef
Dataset tree: blake3:173e2275b014f87fc5ebb06a443e3aba6d44f8c2017c42574d6070e71352d144
Dataset manifest: blake3:fbf3c3d6b5694dea16b2d5c1f5e4fb5d198b339b36a80b0dab91d4714ce04d7d
Unchanged native frame bundle: blake3:94762441a43b03f596a66131ec34af15bba3afbc2bbc5d28ab7dfdabd9b6d68c

The serialized-input instrument passed all 11,520 rows, with 192 construction versus 64 development owner-object pairs and zero overlap. Location targets and relevant-fact positions balance within question types. All 52 active tokens occur in construction; inference retains the full 4,096-token head.

Independent source review is GO. Three new data checks and five new decision checks passed. A review finding about rehashing the final artifact after evaluation was fixed and checked before freeze. Broad QA remains dormant.

The exact fixed dose is 2,352 supported-only + 1,568 mixed updates, batch 512, totaling 2,007,040 answer presentations if complete. First-eight-update time admission is part of that same trajectory; no restart, sweep, development checkpoint selection or extra dose. The published issue's divergent decisions and 1,800-second / 4-GiB budget bind the run.

Preparation audits development semantics; the optimizer receives zero development rows or decisions. Now launching the one frozen CPU fit.

[Full contract and preparation](https://github.com/UOR-Foundation/uor-r4/blob/09056978/docs/r4_zoology_english_binding_1063.md).

### comment 5516386240 — 2026-09-02T21:05:09Z — Casey-allard

Completed through protected PR #1064, merged at 6df75837e55a4ffbbc760bb2bd9650ab27fea433. Fresh origin/main has the exact reviewed tree 3e4ae3f54999273d0d93c6568dc2c8a1fa559ff1 from head 311db033f566a7f094abd4f269ebf12dad7378ba.

The declared experiment is complete; its scientific terminal is `ENGLISH_BINDING_CONSTRUCTION_MISS`, not a successful English model. The single frozen fit completed all 3,920 updates / 2,007,040 answer presentations. Final construction: 2,396/8,192; supported development: 218/1,024; complete counterfactual groups: 0/256; missing-binding answers: 37/256. Only 12/512 same-history question changes changed the predicted answer. Conditional R4/control are `NOT_RUN_ENGLISH_BINDING_MISS`.

Eight focused data/decision checks, independent source/evidence review, and exact fresh-process final-artifact replay passed. Combined fit/scoring/replay: 249.365619167 seconds, maximum RSS 0.485626221 GiB. The final model and full checkpoint/data remain retained. No code, population, dose or threshold changed after the public pre-fit freeze.

Preparation: blake3:c926e16516ef0f1d8242dc0af39a04be46cb082bb6c76590bc73f2717e027ca8
Model: blake3:a4eb5ef76c387ca6ebe9f185b1a5ad023c81291ce4cc9000bb5d23248aaef282
Result: blake3:aaca100c5c2b8abfb126937523c5cce44bb7e6ca2eb8d48260f42e9281606e0f
Replay: blake3:dd5984c22d507faa1e2cea0f9b0c8051fbd3ec923cf53c896768e62708295e02

#973/#820 and milestone 21 now carry the actual outcome and the narrow next recommendation: a separately frozen construction-only diagnostic on this retained artifact, with zero training, to distinguish owner/object/position distractor choices before another learning change. That follow-up is unrun. #973 remains open and #954 blocked. Prior #1059/#1061 attention evidence is preserved, and geometry expansion remains deferred.

[Full record and raw evidence](https://github.com/UOR-Foundation/uor-r4/blob/6df75837e55a4ffbbc760bb2bd9650ab27fea433/docs/r4_zoology_english_binding_1063.md). Protected queue names were transport acknowledgements; broad QA stayed dormant.


---
_Comment/review ids in this file: 5516187720, 5516386240_

