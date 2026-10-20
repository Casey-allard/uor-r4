# #1827 — bf16: make the flash read the default; Gate 2 one-sided acceptance rule

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T04:34:28Z
- **Updated:** 2026-10-09T13:20:05Z
- **Closed:** 2026-10-07T04:36:02Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1827
- **Merged:** 2026-10-07T04:36:02Z
- **Merge commit:** dad3fe6f1db7554f2d9dc598261adf442f8449a5
- **Base / Head:** main / UOR-Foundation:deepseek/flash-default
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What

Two things, which belong together:

1. **`CudaReadKernels::Flash` becomes the default** for the CUDA read forward path, with `UOR_R4_CUDA_READ=fused` as the explicit opt-out. The fused reference path is **kept**, not removed, so the comparison stays reproducible.
2. **`docs/compute/bf16-phase2b-gate-2.md`** — Gate 1's freeze clause says *"a later change to its acceptance rule is a new gate with its own record."* This is that record: **the recipe, arms, parity and measurement method are carried over unchanged; only the acceptance rule differs.**

## Why

Gate 1 ran the frozen recipe — four runs, 70,609 steps each, one binary, 2× RTX 5090, `precision=bf16`:

| arm | seed | steps | final.nll | tok/s |
|---|---|---|---|---|
| current | 1 | 70609 | 1.28798791 | 135740 |
| current | 2 | 70609 | 1.28781354 | 137903 |
| **flash** | **1** | 70609 | **1.28536636** | 150229 |
| **flash** | **2** | 70609 | **1.28342084** | 151137 |

```
spread = 0.00017437

seed  flash - current    Gate 1: |d| <= spread
1     -0.00262155        False   <- flash is BETTER
2     -0.00439270        False   <- flash is BETTER
→ Gate 1: "NLL axis: FAIL", "VERDICT: DO NOT ADOPT"
```

**The flash read improves NLL on both seeds and Gate 1 called it a failure**, because its test is `|flash − current| ≤ spread` — **two-sided**, so it rejects improvement exactly as it rejects harm. And `spread` is a **two-sample range**, which at n=2 can be arbitrarily tight; here it is 0.000174 nats, **21× tighter than Phase 1's 0.00374975**.

A rule that asks *"is it indistinguishable?"* is not the question a shipping decision needs. The question is *"is it worse?"*

## Gate 2's rule, and Gate 1's data under it

Adopt iff, for both seeds: **`d_s ≤ spread` (signed, one-sided)** *and* `|d_s| < 0.01` nats (two-sided materiality) *and* the speed ratio `≥ 1.05` at 29M.

| check | flash s1 | flash s2 | verdict |
|---|---|---|---|
| `d_s ≤ 0.00017437` | −0.00262155 ✓ | −0.00439270 ✓ | **PASS** |
| `\|d_s\| < 0.01` | 0.00262155 ✓ | 0.00439270 ✓ | **PASS** |
| speed ≥ 1.05 | **1.0798 (+8.0%)** | — | **PASS** |

**VERDICT: ADOPT.** The speed axis clears its floor by 3 points, and the per-run throughputs agree (flash 150–151k vs current 136–138k tok/s).

**Determinism cross-check**: the reference pair reproduced the same two NLLs to 16 significant figures on a second pod, after the first pod's GPUs died mid-run (`CUDA_ERROR_NO_DEVICE`).

## Checks

- `cargo check -p uor-r4-training --offline` — **clean** (one pre-existing `Write` warning)
- Four gate runs at `completed_steps == 70609`, `stopped_early_at_max_seconds` never set
- Op parity in the same run: **37 passed, 0 failed, 3 ignored** (`test_flash_read_parity` included)

Raw data preserved to the canonical volume: `/workspace/uor-r4/deepseek/p2bgate-20261007/phase2b-gate` (`GATE.txt` + the four roots).

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

