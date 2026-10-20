# #1065 — A1Q-H: diagnose retained English construction binding without training

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-09-02T21:19:56Z
- **Updated:** 2026-09-02T21:42:14Z
- **Closed:** 2026-09-02T21:41:06Z
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1065
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Decision and scope

Continue #973 after the completed #1063 English curriculum (#1064). Preserve that negative and the qualified MQAR/R4 results. Execute one separately frozen construction-only diagnostic on the exact retained final model, then exact fresh-process replay. Zero training, optimizer steps, development model decisions, new data, native-frame payload reads, or geometry changes.

## Frozen diagnostic

Use the same 8,192 supported construction rows in canonical order, batch 256, eight CPU threads, one inter-op thread, full 4,096-token head and returned attention. First reproduce #1063's 2,396 correct, NLL, predictions, full selected logits and attention identities exactly. A mismatch stops interpretation. Bind published preparation/fit/result/replay envelopes, all 199 predecessor source records, exact model, construction and vocabulary bytes directly; do not invoke predecessor whole-data validators.

Classify outputs relative to each actual query and ordered history: target, same-owner confound, same-object confound, unrelated fact location, out-of-history location, UNKNOWN, other vocabulary. Report q0/q1, question type, base/swapped and displayed-slot strata; category-eligible rows and fact slots; target-slot exposure, selections and correctness. q0 has one of each confound; q1 has one absent confound and two unrelated facts. Report 4,096 question contrasts and 4,096 location-swap contrasts, prediction changes, both-correct counts, full-head differences and fixed-target logit response delta = [z_left(a)-z_left(b)] - [z_right(a)-z_right(b)].

## Predeclared action mapping

Strict majorities are descriptive follow-up routing rules, not capability thresholds or internal causal claims. If one displayed slot receives more than half of in-history selections and an attribute confound exceeds half of q0 in-history errors, investigate position versus attribute jointly. Slot-only majority: position/readout focus. Same-owner confound majority: object disambiguation. Same-object confound majority: owner disambiguation. Neither concentration, but most question pairs retain their prediction: question-to-readout sensitivity. Otherwise: distributed binding errors. Publish every flag and denominator; empty denominators are unavailable. No branch automatically authorizes a new fit.

## Resources and named checks

300 seconds combined run plus replay, 2 GiB peak RSS per process. Expected seconds-scale inference from #1063; no hours-scale run. Activate only focused semantic/classification/pair-margin/decision and source-access/binding checks, exact source reproduction, fresh-process replay, and independent review. Broad QA remains dormant; queue acknowledgements are transport only.

## Done

- Freeze code, source bindings, categories, resource cap and action mapping before model execution; publish preparation CID.
- Run once, preserve immutable raw result and fresh-process replay, and verify unchanged model/source state.
- Explain observed failure profile and the resulting one next recommendation, with uncertainty and scope boundaries.
- Append evidence record and update current claim mirrors, #973, #820 and milestone 21.
- Deliver through protected PR workflow, record actual main merge and close only this completed diagnostic.

Owner: @Casey-allard. Next action: freeze the diagnostic implementation and exact preparation before construction inference.

## Comments (2)

### comment 5516736490 — 2026-09-02T21:34:19Z — Casey-allard

The diagnostic executable is frozen at `58bd1559`, and its preparation was published at `6bf314b6` before fitted inference.

- Preparation: `blake3:5679f721fa60c16601a4d3a8ca46397055b89d7769dfae1ca099a1e3f3fbe5a9`
- Implementation: `blake3:d22cde07d38be3e3042039a93c6cdc06c5f4c772c7ba5490905889215f24827b` (211 files, including all 199 historical source records)
- Retained model: `blake3:a4eb5ef76c387ca6ebe9f185b1a5ad023c81291ce4cc9000bb5d23248aaef282`

All 11 named focused checks passed; independent source review found no blocker. These checks performed no fitted inference. The exact population, categories, eligibility denominators, pair contrasts, literal strict-majority action mapping and 300-second/2-GiB ceiling are published in `docs/r4_zoology_english_diagnostic_1065.md` and its preparation envelope on `issue-1065-english-diagnostic`.

Next action now: execute the single 8,192-row construction pass, require exact #1063 score/logit/prediction/attention reproduction before interpreting it, then verify in a separate process. No fitting, development payload, optimizer checkpoint or frame payload reads, new data, or geometry changes.


### comment 5516839805 — 2026-09-02T21:42:14Z — Casey-allard

Completed and delivered through [PR #1066](https://github.com/UOR-Foundation/uor-r4/pull/1066), merged at `e1d3c431ce520a59572e9158828fecef50e4f793`. Fetched main tree `2cb0214692cf9da51b576b0f680bd96344362e47` exactly matches reviewed head `1f3269b96d20323248d380afa524da5e94ffbbe9`.

Terminal: `CONSTRUCTION_DIAGNOSTIC_COMPLETE`; frozen descriptive focus: `QUESTION_READOUT`. All 13 #1063 construction score fields reproduced exactly, including 2,396/8,192 correct, NLL, logits, predictions and attention. Question changes left 3,974/4,096 answers invariant (97.0215%); only 20 pairs had both answers correct. No overall displayed-slot or pooled attribute-confound majority fired. The complete record preserves smaller/type-specific effects and unavailable q1 category denominators.

The model selected an in-history location in 6,905 rows and UNKNOWN in 1,287, with zero outside-history or other-vocabulary outputs. Fixed-target question-logit shifts split 2,040 positive / 2,056 negative. Location swaps changed 1,134 predictions, with 247 both correct. This localizes a question-to-answer failure profile without identifying an internal cause.

All 11 named focused checks passed; independent source/evidence/documentation review found no blocker. Fresh-process replay reproduced the complete evidence exactly. Recorded run + replay: 3.433826875 seconds, peak 0.774902344 GiB. Model/source identities unchanged. Zero optimizer updates, new development decisions, development/checkpoint/frame payload reads, new rows or geometry changes. Queue acknowledgements were transport only.

Evidence: [record](https://github.com/UOR-Foundation/uor-r4/blob/e1d3c431ce520a59572e9158828fecef50e4f793/docs/r4_zoology_english_diagnostic_1065.md), with bound preparation/result/replay JSON. Result CID `blake3:65b23631b10fe62b215411932cd9fe45f76b43d6b8503d0f2e74dc3d256c9b61`; replay CID `blake3:7222a680c300552ab097ce184500c90c0e44ede8248c4c3f752aa09f4232c0ca`.

README, AGENTS, ROADMAP, RESEARCH, intelligence plan and geometric programme are current. #973, #820 and milestone 21 now point to one separately frozen readout-placement learning experiment: fresh seed-123 fit with supervised answer position 37 instead of 40, matched cell/construction/labels/sampler/optimizer/3,920-update dose, both question types reported, and new frozen development for any new transfer claim. No follow-on fit ran here. Preserve #1063's negative and #1059/#1061's attention positives; #973 stays open, #954 blocked, and geometry expansion deferred.



---
_Comment/review ids in this file: 5516736490, 5516839805_

