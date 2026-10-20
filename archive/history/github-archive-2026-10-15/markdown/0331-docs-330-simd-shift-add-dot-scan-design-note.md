# #331 — docs: #330 SIMD shift-add dot scan design note

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-01T18:16:19Z
- **Updated:** 2026-10-09T14:17:14Z
- **Closed:** 2026-08-01T18:26:58Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/331
- **Merged:** 2026-08-01T18:26:58Z
- **Merge commit:** 38025e353d97fde257578d47e60be6e08bd1058f
- **Base / Head:** main / UOR-Foundation:issue-330-simd-dot-scan
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Refs #330. Design note only — no code authorized.

Proposes vectorizing the shift-add dot scan (~95% of serving cost, op-census measured) within the runtime contract: per-lane variable shifts + lane-wise i64 add/sub + compare/blend sign fixup — no SIMD multiply, no dot-product instructions, no FMA. Mirrors the simd.rs pattern (safe scalar normative, isolated unsafe adapter, proptest witness, runtime dispatch). Empirical Criterion: ≥3× isolated scan / ≥2× end-to-end at bit-identical outputs, else recorded negative result.

Prerequisite noted inside: the ns/token serving wall-clock harness (Phase 0). Depends on #329 (TLA7) for the residual-path baseline. Claim-wording gate passes locally.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

