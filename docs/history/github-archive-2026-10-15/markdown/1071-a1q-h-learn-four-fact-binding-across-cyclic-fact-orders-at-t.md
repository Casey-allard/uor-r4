# #1071 — A1Q-H: learn four-fact binding across cyclic fact orders at the matched dose

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-09-02T23:35:31Z
- **Updated:** 2026-09-03T00:04:28Z
- **Closed:** 2026-09-03T00:03:16Z
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1071
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Decision

Continue #973 after #1069 / #1070. Run one fresh matched fixed-dose fit with cyclic fact-order augmentation, using the plain #1067 query-object readout and no owner residual. #1069 improved owner and aggregate scores but failed object-pair preservation and scored 99.6582% at target slot 4 versus 29.4922–36.0352% elsewhere. Rehearse the same bindings across positions before changing geometry.

## Frozen intervention and dose

Keep original #1063 construction/vocabulary bytes, labels, four-fact semantics, all 41 tokens, BOS/query suffix, readout 37, source model 303,744 parameters, seed 123, optimizer and sampler/RNG trajectory. After each sampler batch, rotate the four complete eight-token fact blocks by phase-local traversal index modulo 4. Supported traversal=16 batches; mixed traversal=20. Offset resets 0 at update 2352. All variants of a world share an offset within a traversal. No extra random draws, rows presented, parameters or updates.

Retain 2,352 supported plus 1,568 mixed updates, batch 512, total 3,920 and 2,007,040 presentations (1,846,452 supported; 160,588 unknown). Final presentations by offset must be 507,904 / 507,904 / 501,760 / 489,472. Derive actual supported/unknown exposure from the existing counters; mixed ends after 78 complete traversals plus eight batches at offset 2. The first eight real updates count in the dose and admission.

Before any model scoring or fitting, freeze source/data/decision bindings. Freeze256 fresh development worlds/groups at seed 10712, all five variants, original grammar/held-out-pair/slot/location balance. Exclude all construction and #1063/#1067/#1069 development canonical worlds and exact inputs. Recreate historical populations with unchanged generators and verify canonical CIDs without opening retained development payloads.

## Comparison and predeclared actions

After the fixed fit, score candidate and retained #1067 on the same 8,192 supported construction rows at each unique cyclic offset 0–3. Rotation 0 is canonical, not a fifth population. The reference's canonical full evidence must reproduce its published record before interpretation. Rotations and two question pairs per world are correlated views; report world-level all-order correctness and per-order pairs/slots rather than treating views as independent trials.

To adopt the augmented recipe require, in EVERY rotation:

- Each question type's both-correct count >= min(reference same-rotation count+205,2048), with ceiling-limited cases flagged. Known canonical headroom is 2,001 owner pairs and 1,601 object pairs, so the +205 target is reachable there.
- Every target slot >=1024/2048 correct (50%).
- Overall correct >=reference same-rotation correct. Report any family regression separately even if another family improves.

If all behavior requirements pass but any construction rotation is below 8,111/8192, retain the improved construction recipe and its remaining-error profile; development stays unscored. If a gain/slot requirement misses without regression, retain partial progress but keep #1067 as reference. If any family or overall score regresses, report the tradeoff and retain #1067. No extra dose or retuning follows in this issue.

Fresh development opens only if BOTH the behavior requirements and all four construction>=8111/8192 gates pass. Evaluate the sole final candidate in all four orders; require per order 973/1024 supported,231/256 complete groups,116/128 complete per type,244/256 unknown. A transfer miss retains construction only; all pass permits a separate R4 preservation step. No R4/geometry/frame execution or generation here. Incomplete/resource/integrity outcomes remain explicit, not scientific negatives.

## Resources, focused checks and Done

Use 1,800 seconds / 4 GiB total across fit, reference+candidate scoring and exact fresh-process replay, eight Apple Accelerate CPU threads and one interop thread. Prior fit 277 seconds and single-view score 1.8 seconds give an expected fit plus eight-view comparison/replay below six minutes; the original 60-second evaluation allowance is retained and total budget enforced. Full fact rotation preserves labels because every world has distinct owner-object keys.

Activate only rotation semantics/causality/RNG/boundary/exposure checks, full-trainer AST matching apart from batch transform and derived ledger, data copying/exclusions/access, matched-decision/conditional-development/correlated-view accounting, exact reference reproduction, final artifact/state binding and fresh-process replay. Independent source/preparation/evidence review. Broad QA dormant; queue statuses transport only.

Publish preparation before fitting, finish the one admitted dose, preserve model/checkpoint and raw evidence, append the result, update six current mirrors and native #973/#820/milestone 21, deliver via protected PR, verify main tree and close only this experiment. Owner @Casey-allard. Next action: freeze this cyclic-order implementation and preparation.


## Comments (2)

### comment 5518115535 — 2026-09-02T23:48:57Z — Casey-allard

Preparation published before fitting or model scoring in source commit `98ea9564` and preparation commit `fe23bee3` on `issue-1071-cyclic-facts`.

- Preparation: `blake3:92ffef681fe6bd4cfc6532bd811496b6e9c4d052b978c172d3a5ca9bcef8ef05`
- Implementation: `blake3:4693fca24e67aa4bab43ddfc71ee144cb3771c7fca32abea5cb55cdb4356b761` (254 files)
- Dataset: `blake3:6eeb4831d9558dbbb0ac1235b314fb47e1412a561aad429d2af56acd81dc0a54`
- New development: `blake3:31dca3ff6327c75bd5208758dcd748661002c8bb109346d8c2d4a3b414c02ed9`

All 15 focused checks pass; independent source review found no blockers. The prepared development has 1280 rows / 768 canonical worlds, excludes all 8448 construction+historical worlds and 14080 exact inputs, and remains model-unscored. Plain #1067 reference is byte-verified during preparation and never loaded into fitting.

Frozen schedule: phase-local traversal modulo four, reset at update2352, right rotation of intact facts, exact3920updates/2007040presentations. Per-offset totals507904/507904/501760/489472. Criteria in every matching rotation: owner and object both-correct >=min(reference+205,2048), every target slot>=1024/2048, overall and individual-question-family preservation. A regression retains#1067. Both behavior and every construction>=8111 must pass before four-view fresh development. Cumulative1800seconds/4GiB, eightCPUthreads, original admission rule. No geometry or generation.

Next action: independent preparation verification, then the sole admitted fixed fit followed by matched final scoring and exact fresh-process replay. [Frozen record](https://github.com/UOR-Foundation/uor-r4/blob/fe23bee3/docs/r4_zoology_cyclic_facts_1071.md).

### comment 5518242861 — 2026-09-03T00:04:28Z — Casey-allard

Completed #1071 through [PR #1072](https://github.com/UOR-Foundation/uor-r4/pull/1072), merged as `88b084450067aa770099072f85348b00adbc1e6b`. Main tree `4464c3ec1b0725b6222e098a40b10343e2342fae` matches reviewed head `f459392c9ff14e16c50053fd52b1521e9487eb57` exactly.

**Terminal: CYCLIC_FACTS_PRESERVATION_MISS. Keep #1067 as reference.**

| Rotation | Reference correct /8192 | Candidate correct /8192 | Owner both correct: reference→candidate /2048 | Object both correct: reference→candidate /2048 |
| --- | ---: | ---: | ---: | ---: |
| 0 | 3735 | 1702 | 47→11 | 447→20 |
| 1 | 3354 | 1721 | 23→18 | 329→23 |
| 2 | 3487 | 1705 | 35→17 | 392→24 |
| 3 | 3347 | 1711 | 33→13 | 352→30 |

Every frozen behavior criterion misses in every order. Canonical accuracy falls 45.5933%→20.7764%; every candidate slot is 19.1406–22.9004%, below 50%. No candidate world has a complete quartet in any rotation. Canonical supported questions receive 2,647 unknown answers/8192=32.3120%. Increased owner-change responsiveness does not improve paired correctness. Similar low accuracy across orders is not invariant-prediction or binding evidence.

The reference's full canonical score AND diagnostic reproduce #1067 exactly. Sole fresh fit completed 3,920 updates and 2,007,040 presentations, with 1,846,452 supported / 160,588 unknown. Actual offset totals 507,904/507904/501760/489472 and the 844-unknown partial tail reconcile. All 15 focused methods and independent source/preparation/evidence/record/final-mirror reviews pass. Exact fresh-process complete-evidence replay passes; fit+comparison+replay 315.589262 seconds, peak 843,907,072 bytes = 0.785949707 GiB. No extra fit or dose. Fresh development remains NOT_RUN_CONSTRUCTION_MISS with zero model decisions; R4 NOT_RUN_SEPARATE_INFERENCE_STEP. No geometry/frame execution or generation. Broad QA NOT_RUN; protected statuses acknowledge transport only.

Next recommendation: one separately frozen fact-level learned Q/K/V compound-binding prototype on the same four-fact lexical task. Independent owner–object query/key projections, shared fact encoder without slot identity, location values, learned null, ordinary softmax, full 4,096-token head. Fixed grammar supplies roles; no equality matcher or target-derived routing. Freeze controls, dose and newly excluded development before fitting. A positive would establish structured binding, not learned English parsing. No successor prototype ran here.

[Binding record and raw evidence](https://github.com/UOR-Foundation/uor-r4/blob/88b084450067aa770099072f85348b00adbc1e6b/docs/r4_zoology_cyclic_facts_1071.md). Result CID `blake3:b5ab27771843d347d9188d4541b46c34bc4ab1d860d956387b8726a100f513be`; replay CID `blake3:08d671be8d4e7db36860493d59c85e0ddec2bfab2259b94c628c6d23436685de`. Six current mirrors reconciled; historical evidence and #1059/#1061 attention positives preserved. #973 stays open and #954 remains blocked. Original unrelated checkout artifacts untouched.


---
_Comment/review ids in this file: 5518115535, 5518242861_

