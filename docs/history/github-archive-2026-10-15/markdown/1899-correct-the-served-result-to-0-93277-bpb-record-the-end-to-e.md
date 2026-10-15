# #1899 — Correct the served result to 0.93277 BPB; record the end-to-end reproduction (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T12:37:10Z
- **Updated:** 2026-10-09T13:22:06Z
- **Closed:** 2026-10-08T12:38:51Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1899
- **Merged:** 2026-10-08T12:38:51Z
- **Merge commit:** 9c0a8d3d543bf3bc494d154daec450cd60da6dfd
- **Base / Head:** main / UOR-Foundation:deepseek/served-correction
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## A published number was wrong, and it's corrected here

Round 10 published the multiplier-free served result as **0.9550 bits per byte**. Recomputing from the same recorded nll on the same byte basis (2.837427 bytes/token, bits = nll / ln 2):

| quantity | nll | correct BPB | published |
|---|---:|---:|---:|
| fused float reference | 1.7250754982233047 | **0.87712** | 0.8770 ✓ |
| transport-snapped | 1.7975700236856937 | **0.91398** | 0.9138 ✓ |
| **multiplier-free D11 engine** | 1.8345338586523747 | **0.93277** | **0.9550 ✗** |

The effective claim is **better** than published: **0.93277** against the 4.3885 served today is a **4.70×** improvement, and the served-to-fused gap is **0.0552 BPB**, not 0.078. This PR amends the evidence document and `current-state.md`, and the round-10 comment is corrected on #820.

## The reproduction result, which matters more

Rounds 16–18 retrained the same 19.9 M stack (round 10's pod was deleted, so the model was **rebuilt rather than quoted**) and re-ran the whole chain:

| quantity | round 10 | round 18 | difference |
|---|---:|---:|---:|
| fused nll | 1.7250754982 | 1.7259248123 | 8.5e-4 |
| snapped nll | 1.7975700237 | 1.7963021882 | 1.3e-3 |
| **d11 nll** | **1.8345338587** | **1.8345359994** | **2.1e-6** |
| d11 top-1 agreement | 1.0 | 1.0 | 0 |

The multiplier-free served number reproduces to **six significant figures** across an independent retrain, a **different GPU** (5090 → 4090), a different binary commit and a different pod. That's the execution policy's *"do not claim bitwise cross-backend reproducibility without measuring it"* case — now measured rather than assumed.

## Also recorded

**Job contention is real and measurable**: the identical training recipe ran at 218.6 k tok/s on an idle pod and **126.9 k tok/s** with a CPU-bound evaluation holding six-plus cores alongside it — a 42 % slowdown. The runbook's "one job per GPU" rule doesn't cover a competitor that isn't on a GPU at all.

The finer export arm (256 calibration windows, damp 0.001) is still running; its number lands next.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

