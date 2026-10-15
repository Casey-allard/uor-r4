# #1830 — docs: bf16 Phase 2b report — flash read ADOPTED

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T05:18:50Z
- **Updated:** 2026-10-09T13:21:27Z
- **Closed:** 2026-10-07T05:20:22Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1830
- **Merged:** 2026-10-07T05:20:22Z
- **Merge commit:** 32448584aa455d069da0795db0cedf3eb84246a9
- **Base / Head:** main / UOR-Foundation:deepseek/p2b-report
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The reporting artifact **handoff §7** requires, carrying §6's items.

## Outcome

**ADOPTED.** Flash is the default CUDA read forward path as of **#1827**; `UOR_R4_CUDA_READ=fused` is the explicit opt-out. Implementation was **#1809**.

## Measured

**NLL** (four runs, one binary, 70,609 steps, all reached `completed_steps == 70609`):

| arm | seed | final.nll |
|---|---|---|
| current | 1 / 2 | 1.28798791 / 1.28781354 |
| **flash** | **1 / 2** | **1.28536636 / 1.28342084** |

`spread = 0.00017437`; `d_s` = **−0.00262155** and **−0.00439270** — **flash improves on both seeds**, mean **+0.00351 nats**. Gate 1's two-sided `|d_s| ≤ spread` called this `DO NOT ADOPT`; Gate 2's one-sided rule adopts it ([`bf16-phase2b-gate-2.md`](docs/compute/bf16-phase2b-gate-2.md)).

**Speed** — 29M: 110,316 → 119,118 tok/s, **ratio 1.0798**, clears the 1.05 floor. **96M: 56,309 vs 56,179 tok/s, ratio 0.9977 — the gain does not carry to 96M.** Reported as a limitation; the frozen floor is stated at 29M, so the verdict is unchanged.

**Op parity**: 37 passed, 0 failed, 3 ignored, `test_flash_read_parity` included, at the frozen `1e-4 + 1e-3·|reference|` bound.

## UNAVAILABLE — marked, not approximated

- **`nsys` before/after profile**: Nsight Systems is **not on the pod image** (`/opt/nvidia` has `nsight-compute` only; no `nsight-systems` apt package). The *before* half is on trunk (Phase 2a, #1757); the *after* half needs an image with `nsys`.
- **Parity measured maxima**: the suite asserts the bound rather than reporting headroom.

## Evidence

`/workspace/uor-r4/deepseek/p2bgate-20261007/phase2b-gate` (1.2 GB) and `/workspace/uor-r4/deepseek/p2b-profile-20261007` (1.5 GB). Reference pair reproduced to 16 significant figures on a second pod after the first pod's GPUs died mid-run.

References #820. Closes the Phase 2b work item of the handoff.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

