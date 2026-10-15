# #417 — feat(certify): #399 — gated two-pass generation + draft-anchor drift probe: falsifier 1 FIRED

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-04T22:41:44Z
- **Updated:** 2026-10-09T13:09:07Z
- **Closed:** 2026-08-04T23:01:48Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/417
- **Merged:** 2026-08-04T23:01:48Z
- **Merge commit:** 7fe1799368347cb82bb3b12c74c07d8c25d9d1d6
- **Base / Head:** main / UOR-Foundation:i399-twopass
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #399. Ships the gated two-pass generation path (`two_pass_infill_generate`: pass-1 greedy draft with witness-status recording, pass-2 pure re-rank of gated free positions via `score_candidates_infill`) AND the drift probe that was queued as falsifier 1 for the B′ mechanism reading — with a NEGATIVE result that the record must carry.

**Drift probe (schema 20, natural wiki-obs, 100,306 held-out; all prior rows byte-identical):** the DRAFT-gated arm predicts the anchor from a drafted context (engine's own tokens for the intermediate positions) instead of the teacher-forced context the B′ arms used.

| arm | all | live slice |
|---|---|---|
| gated self (teacher-forced ctx) | 40.0% / 7.8332 | **45.0 vs 39.4** (63,567) |
| **DRAFT-gated (real two-pass)** | **35.9% / 9.3847** | **40.4 vs 41.3** (62,989) |

Win/loss draft vs 1+2 on live: +966 / −1,581 — **net negative**. Draft drift in the intermediate tokens degrades the anchor prediction enough to flip the channel below Rule 1+2.

**Honest scoreboard after falsifier 1:** A-mode infill serving (anchors GIVEN as inputs — no prediction, no drift) stands fully validated at +4.2pp live (M2 row, unchanged). Standalone two-pass B-mode in its naive greedy form is REFUTED — the B′ self-anchor result depended on true intermediate context, exactly as the queued falsifier suspected. Possible rescues (iterative re-draft, beam draft, per-step confidence gating) are future work and unmeasured. The two-pass code ships as the measurement vehicle and for A-mode-adjacent uses; its standalone use is contraindicated by this record.

Tests: 7/7 fwda_roundtrip incl. gate-fails-closed; workspace clippy + test-compile clean.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

