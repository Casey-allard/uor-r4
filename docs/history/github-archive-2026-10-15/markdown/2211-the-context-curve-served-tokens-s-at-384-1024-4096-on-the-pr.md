# #2211 — The context curve: served tokens/s at 384/1024/4096 on the product path, and the O(n) prize is bounded at ~1.7x (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-11T20:17:40Z
- **Updated:** 2026-10-11T20:27:23Z
- **Closed:** 2026-10-11T20:21:54Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2211
- **Merged:** 2026-10-11T20:21:54Z
- **Merge commit:** 60c12c248f331b00a595a485d0458b6516651bbb
- **Base / Head:** main / UOR-Foundation:deepseek/context-curve
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**The question no evaluation in this project could ask — because all of them sit at context ≤ 384, where the quadratic read is cheapest.** Measured on the **product path** (`d11-evaluate`, both integer engines, one architecture, only the served window differing — the window *is* the model's `context`):

| context | **D11 tokens/s** | D10 tokens/s | wall, 4 windows | bit-exact |
|---:|---:|---:|---:|---|
| 384 | **63.41** | 105.49 | 24.2 s | ✓ `0.0` / `0` |
| 1,024 | **56.19** | 77.40 | 72.9 s | ✓ |
| 4,096 | **30.43** | 37.79 | 538.4 s | ✓ |

**The rate falls 2.1× for 10.7× the context** — per-window cost ≈ **n^1.31**, not n². So **the O(n) prize is bounded: at 4,096 (where the audit puts the read at 40.5 % of MACs) an infinitely cheap read buys at most ≈ 1.7× the served rate**, and less at every length we serve today. Two new facts ride along: the multiplier-free engine's **relative** cost **falls** with context (**D10/D11 1.66× → 1.24×** — a favourable fact for D11), and **bit-exactness holds at 4,096** (`d11 − d10 nll = 0.0`, `max |Δlogit| = 0`, `top1_agreement 1.0`).

Line: context scaling of the served read (measurement cycle) · count n/a — **no capability claim** · headline unchanged: reply `acceptable` 24, memory 24/40

## What it decides

The highest-value compute move is **not** replacing the read with an address: the MLP (41.9 % of MACs at 384) plus the recurrence (32.7 %) plus the head are paid at every length and set the floor — **82 % of the arithmetic at 384, ~55 % at 4,096**. The read is a **long-context optimisation** whose ceiling is now a number instead of an assumption. The port stays worth doing for long-context serving; it is no longer the first thing to do, and the pre-registered next step on this line changes accordingly.

## Method, and the fix that made it possible

The 4 M-token training stream was **built locally and pushed to the pod (8 MB)** after the previous attempt hit a pod whose volume carried none of this lab's data — so this run could not fail on unreachable inputs. One RTX PRO 6000: three 300-step fits, three exports, three evaluations, **≈ $2.00**, inputs verified before the run, **released and deleted at the last evaluation — $0/h since**. Two invocation errors (`missing arch=`, stale export roots from the failed first attempt) cost ~2 minutes and are recorded rather than hidden.

## Checks

- [x] `python3 scripts/check_claim_wording.py` **pass**; record, `current-state` entry, `STATUS.md` row.
- [x] Both integer engines agree bit for bit at every context — the D11 contract's own test — so the served numbers are the model's, not an approximation's.

**Declared:** three short fits trained for **rate, not quality** — no panel, no capability claim, and the models are **not candidates**; one pod, four windows per context, so the **ratios** are the result rather than the absolute rates; context 4,096 is **10.7× the length every capability evaluation uses**, and nothing here says quality would hold there.

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

