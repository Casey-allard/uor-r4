# #2257 — The operator phase: geometric replacements for the gate, projections and head (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-12T23:24:49Z
- **Updated:** 2026-10-12T23:26:44Z
- **Closed:** 2026-10-12T23:26:17Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2257
- **Merged:** 2026-10-12T23:26:17Z
- **Merge commit:** aaa210ef46301623f10164ae5af586d7f558483a
- **Base / Head:** main / UOR-Foundation:kimi/operator-phase-2026-10-12
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What

Design + pre-registration record for the **operator phase**, implementing the owner's 2026-10-12 decisions ([#2029 comment 6133330776](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6133330776)): no sigmoid/tanh anywhere, full matmul removal (training included), rung 0b first after the phase, Kimi as project manager.

- `docs/labs/operator-phase-2026-10-12/README.md` — OP1 geometric gate (G1 rectified-square, G2 Hamilton), OP2 native Hamilton-projection CUDA kernels (today's quaternion weights execute through cuBLAS — a banned GEMM), OP3 matmul-free head (OP3a product-key engineering arm, OP3b murmuration fixed-address research arm, fallback predeclared), OP4/rung-0b rank-read kernel; frozen pre-ban artifacts as controls (no new banned comparator runs); bars 0.02 nats / 2 seeds; prospective cost ≤ \3 total.
- `docs/integration/current-state.md` — newest-first entry.
- `docs/integration/project-track.md` — the operator phase as the critical path + line assignments.

Docs only. No compute. Each rung waits for TEST FITNESS: FIT.

## Checks

- `python3 scripts/check_claim_wording.py` — PASS
- `python3 scripts/check_no_softmax.py` — unaffected (docs are not scanned); ratchet still OK at 1649 on this head.

References #2029

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

