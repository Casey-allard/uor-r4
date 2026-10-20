# #1077 — Learn a position-independent language interface for compound binding

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-09-03T01:21:34Z
- **Updated:** 2026-09-03T01:52:17Z
- **Closed:** 2026-09-03T01:52:17Z
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1077
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Continue #973 after #1075/#1076 with one bounded learned language-interface experiment. Replace fixed-position owner/object/location extraction while retaining the qualified 286,976-parameter compound-binding core and current native geometry.

Train only a small role reader over controlled sentence variations. All binding embeddings/projections/null/output/decoder weights stay frozen. The reader must attend over actual clause tokens without role-position lookup, entity-type filtering, answer filtering, or labels at inference. Include same-type distractors and matched role-changing examples so lexical token classification alone cannot qualify. Use soft role mixtures feeding the original core, with explicit role supervision confined to construction.

Freeze exact architecture, lexical mapping, source/model identities, data generation/splits, training dose, runtime cap and decision criteria before any learned run. Development must contain predeclared unseen wording/order combinations and remain closed until construction qualification. Preserve observed source-world provenance: a new language rendering is not a new independent semantic-world result.

Ordinary interface learning is the decision in this issue. Retain #1075 R4 preservation as the anchor; a qualified new interface may receive a separately frozen R4 qualification next. No geometry expansion, core retraining, free generation, correctness/reasoning/chat work, or broad QA.

DoD: publish frozen source/preparation and reachability/preflight; execute one bounded fit, construction/causal controls and conditional development; exact fresh-process evidence replay; append measured record and update six current mirrors, native parents and milestone21; deliver through protected PR and verify closure. A miss retains the working binding core and records the specific interface defect without retuning or replacement fit.

Next action: finalize and independently review the concrete reader, controlled syntax split, source bindings and run contract, then execute once. #973 remains open; #954 remains blocked.

### #1077 completion (2026-09-02)

LANGUAGE_INTERFACE_HELDOUT_PASSED, protected PR #1078 at `6ebcf4bf48ed82addd0a38c7fe368b989815f772`; merged tree `04c329b228c88e171cab5292fac9dfa7c6bccace` exactly matches reviewed head. [Complete DoD evidence and next action](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5519108070); [record](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/r4_zoology_language_interface_1077.md). Frozen source/preparation were published before the sole fit; conditional evaluation and exact replay completed. Six current mirrors, native parents and milestone21 are reconciled. The issue scope is complete. Next work is separately frozen unchanged-R4 qualification of the learned soft interface. #973 remains open and #954 blocked.


## Comments (2)

### comment 5519018141 — 2026-09-03T01:41:08Z — Casey-allard

Frozen before candidate fitting. Source `f52afce506897f8b477b1ebafd37f47272699410`; preparation `blake3:0395b826049dbeed351a647960c7b66cc4d65fc19b65eb3c522fcdd807aaad69`; 295-file implementation `blake3:baa22e6e908dbf3a7c694679f386815736ca65db444e98ce823d1b5a26dfdb83`; data manifest `blake3:dc349b9931556cb45604295382766c78d173d26a2dca8491deacefae49fbd0d1`.

[Full contract](https://github.com/UOR-Foundation/uor-r4/blob/issue-1077-language-interface/docs/r4_zoology_language_interface_1077.md) and [immutable preparation](https://github.com/UOR-Foundation/uor-r4/blob/issue-1077-language-interface/docs/r4_zoology_language_interface_1077_preparation.json).

Train only the 141,571-parameter contextual reader over all valid clause tokens. The 286,976-parameter core and full fact-plus-null mixture stay fixed. Same-type distractors and identical-bag questions with different answers require context. Construction has 20,480 rows in two seen wording combinations; conditional development has 5,120 rows comparing seen and withheld combinations on already observed #1073 semantics. There are no new semantic worlds.

One 512-update AdamW fit, batch 128, learning rate 0.003: 65,536 row presentations and 917,504 role labels. No answer loss or core updates. Synthetic four/eight-thread CPU timing with zero optimizer updates and loss/gradient agreement within 1e-6 selects the plan. The first eight actual steps are retained for admission. The cap is 900 seconds cumulatively through replay and 4 GiB. All 16 focused checks and independent source review passed; broad QA stays dormant.

In every primary view require at least 95% supported and UNKNOWN accuracy, 95% complete quartets per family, 99% overall and per-role pointer accuracy, and 95% complete paired syntax answers. The construction value-cycle control requires exact attention, equal work, at least 50-point supported loss and at least 95% reassigned/UNKNOWN accuracy. Only full construction and control qualification opens development. One fresh process must reproduce the complete evidence exactly.

A positive result retains the reader and leads to a separately frozen R4 qualification. A miss preserves the core and identifies the failed interface criterion without retuning or a replacement fit. Given clause boundaries, known lexicon and one seen question template limit the claim; no R4 execution, geometry increase or generation occurs here. #973 remains open and #954 blocked.

Actual preparation review is the final launch gate. Next action: selected-plan sole fit, conditional evaluation and exact replay.

### comment 5519109070 — 2026-09-03T01:52:16Z — Casey-allard

#1077 completion — `LANGUAGE_INTERFACE_HELDOUT_PASSED` (2026-09-02).

Delivered through protected PR #1078 at `6ebcf4bf48ed82addd0a38c7fe368b989815f772`. Reviewed head `32f81cfac830660ac6545682284f2fb4a75c81bb` and merged main share tree `04c329b228c88e171cab5292fac9dfa7c6bccace`.

The 141,571-parameter contextual reader completed one 512-update fit over 65,536 row presentations / 917,504 role labels. The 286,976-parameter compound-binding core remained frozen, with no core load or answer loss during fitting. Full soft role mixtures feed the original four-fact-plus-null Q/K/V operation and full 4,096-token head.

All 25,600 primary answers and 358,400 role decisions are correct. In each of two construction views: 8,192/8,192 supported, 2,048/2,048 UNKNOWN, 2,048/2,048 same-bag owner-changing pairs and 1,024 complete quartets per family. In each of four development views: 1,024/1,024 supported, 256/256 UNKNOWN, 256/256 syntax pairs and 128 quartets per family. Two development views use seen combinations; two use the withheld clause/owner-phrase combinations. Both construction controls give 0/8,192 original supported answers, 8,192/8,192 reassigned answers and 2,048/2,048 UNKNOWN, with exact role/binding attention and equal work.

One fresh process reproduces the complete evidence exactly. Fit 25.369493s, evaluation 12.174104s, replay 12.376540s: 49.920137s cumulative, peak 1,564,622,848 bytes (1.457169 GiB). Four CPU threads won the synthetic 4/8-thread comparison with identical losses/gradients. All 16 focused checks and independent source/preparation/evidence/record reviews passed. Broad QA is NOT_RUN; queue statuses acknowledge transport only.

[Complete record and immutable envelopes](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/r4_zoology_language_interface_1077.md). Fit `blake3:7c5a46f0b044ee3a9da3aa2126b4fb31f3088e239a5fd6b9f4e276334a97770d`; result `blake3:294fe7f488237c196525a3470f48c3b55f5a14232e23c3243b88f579da85e1c1`; replay `blake3:8e5b1f11be99835ec3dddd197357da462353e6bb9c7f5b8f603e1db766d0770f`; complete evidence `blake3:cc8b771dd5e8e22d34218558f570e99d5d59efd4d28405a4d200b6d835dcbdbb`.

Scope: supervised local owner disambiguation with supplied clause boundaries, known vocabulary, disjoint object/location lexica, one seen question form and previously observed #1073 worlds. This does not establish general English, segmentation, fresh semantic-world generalization, generation, correctness, reasoning, chat or geometry advantage. Maximum learned-interface versus canonical hard-field oracle logit difference is 0.0623424053; all answers agree but numerical equality is not claimed.

Next action: retain reader and core, keep current geometry, and separately freeze unchanged-R4 qualification of this learned ordinary soft path, preserving both role mixtures and the full fact-plus-null aggregation. Its reference must be the learned interface, not the old hard-position oracle. No extra fit, R4 run or geometry change occurred in #1077. #973 stays open and #954 blocked. Six current mirrors are updated. [Previous #1075 completion and preserved history](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5518792331).

Native parents and milestone21 are reconciled; closing only #1077 with completed DoD evidence.


---
_Comment/review ids in this file: 5519018141, 5519109070_

