# #1508 — [Epic] Native geometric model: grounded conversation and durable memory first

- **Type:** Issue
- **State:** closed (not_planned)
- **Author:** @Casey-allard
- **Created:** 2026-09-29T20:06:34Z
- **Updated:** 2026-10-09T14:42:24Z
- **Closed:** 2026-10-09T14:42:24Z
- **Labels:** roadmap, epic, roadmap:active
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1508
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

<!-- active-roadmap-2026-10-01 -->
**October 1 adopted roadmap — ACTIVE.** PR #1564 / D19; adoption task #1563.

Own the integrated user capability on one native model: learned text→relation/act/span→exact versioned memory→generated answer, saved/reloaded and faithfully served under D11, then broader language and coding/reasoning.

Immediate execution: #1552; shared architecture/artifacts #973. Memory #962 and grounding #954 are first capability owners. Serving #964, cost #963, reasoning #955, Rust coding #1088, API/WASM #1172, Studio #1173 and release #965 retain their acceptance.

Consume current E4/classifier/data results. No automatic old A1/A2/143M-token schedule, arbitrary larger fit or stale recovery hold is active. The unchanged sealed §8 milestone is an intermediate product qualification, not the entire coding/reasoning alpha. Do not close this epic on component scores or oracle-assisted generation.

[Canonical plan](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/project-track.md) · [Mechanism admissibility](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/mechanism-admissibility-2026-10.md). Historical gates/results and full capability acceptance remain unchanged. Live comments own newer work; this header does not reset cumulative resources or transfer another lab's live files/jobs.
<!-- /active-roadmap-2026-10-01 -->


<details>
<summary>Preserved pre-adoption issue body and original acceptance</summary>

**Current coordination update (30 September 2026):** Current sequence: restore useful retrieval/copying, integrate exact persisted memory/context and one reloadable session path, preserve behavior through D11/D5 serving, then qualify local conversation and extend reasoning/coding. Claude's current source claim and pre-run amendments are on #1511; shared selector coordination is on #1512. Heavy/model execution is held by recovery #1520. The earlier arm list/dose below is historical planning: current versioned work cards, council dispositions and measured learning curves govern successor experiments; old outcomes/criteria are not rewritten. No useful-chat or frontier result is asserted.

---

The native geometric chat model that proves the mechanisms end to end under D11. Plan of record: docs/plans/2026-09-29-path-to-chat.md (#1507).

**Gates, in order:**
- **A1, retrieval (Lab 1):** held-out-value recall ≥ 0.9 on M-world v2 plus MQAR, on a 1.5M stack. Four arms: plain, pointer, flock k=7, and a dot control.
- **A2, context malleability (Kimi):** truncated prefixes, a log-bucketed age table, a D11 ring buffer. At 1,024 context, in-window tokens are ≤ 0.02 nats worse.
- **A3, 7M fit (Lab 1):** 143M tokens (chat-v0, M-world v2, teacher paraphrases), with a pre-registered dose-response curve.
- **Then:** the persisted-log memory, QAT, S1.2 bundle and reload, the blind qualification panel, and the cost report.

**Parity rule:** a geometric mechanism stays whenever it is within 0.02 nats (or 0.03 accuracy) of its matmul equivalent, judged on paired arms with ≥ 2 seeds.

</details>


## Comments (2)

### comment 5904792707 — 2026-09-30T05:34:00Z — Casey-allard

Owner-requested external research assessment: Skrit-Bit, pinned `e5d8c871daec411c3b47af56244e1a2fe70f9314`. [Assessment and three specialist audits](https://github.com/UOR-Foundation/uor-r4/tree/9892d08ac46be53c843dfe51f925be0f7cb0cf7b/research-reviews/20260930-skrit-bit). References #820.

Useful lead: explicit context elaboration, finite typed-action constraints and checked derivation traces around the existing retrieval/session path. Closest current seam is `stack_store.rs` (`write_from`, pinned `read_at`, exact occurrence/version identities, 16 status/view codes); its declared host bridge is not wired into stack forward and is not a qualified D11 kernel. Preserve those boundaries.

The external repo implements a small TS lexer/parser/stack VM; its dependent-type checker, proof kernel and subsumption resolver are not implemented in that path. Source audits identify ownership/cross-runtime defects. The PDFs' O(1) whole-tokenization, SSA-implies-termination and general truth-filter claims are unsupported or incorrect. All18PDFpages inspected, three separate same-provider specialist audits, independent synthesis correction applied, finite Wolfram checks retained. No external source executed/imported; no Cargo or model run.

Proposed discriminator, NOT ADOPTED/NOT RUN: same exact executor with (1) gold explicit actions as backend diagnostic only, (2) learned explicit actions, (3) learned inherited-context actions with matched information/capacity/budget, plus the current end-to-end baseline and source/NoRead interventions. Test scope resets, ambiguity, corrections, same-value assertions, previous/current queries, unseen values/phrasing and reload. Keep answer_oracle offline. If the oracle arm works but learned parsing fails, the learning bottleneck remains; no theorem/bitmask claim can substitute for it.

This is a design lead for the existing retrieval/session work card, not a new model/language programme or authorization to resume compute. #1536/#1520 holds remain.


### comment 6083177829 — 2026-10-09T14:42:23Z — Casey-allard

Epic folded into tracker #2028 (milestones #2030 and #2031).


---
_Comment/review ids in this file: 5904792707, 6083177829_

