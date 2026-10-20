# #231 — Teacher floor returns inf bits/token on the certified legacy path

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @maurathat
- **Created:** 2026-07-27T03:26:36Z
- **Updated:** 2026-07-28T04:10:24Z
- **Closed:** 2026-07-28T04:10:24Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/231
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

`certify` on the legacy llama2.c path reports:

```
corpus: 150000 tokens, 754 stories, held-out 30036
teacher floor inf bits/token | teacher ceiling 69.5%
```

Certified values are a floor of 1.5960 bits/token and a ceiling of 70.4%. An infinite floor means the teacher assigned probability zero — or a value underflowing to zero — to at least one observed token, on its own reference path, on the pinned platform. The ceiling also moved (69.5% vs 70.4%), which corroborates that the teacher-side computation has shifted rather than a single degenerate token being at fault.

## Why it matters beyond this run

The teacher-probability computation is shared between corpus observation and evaluation. A path that can produce a zero probability during scoring can plausibly produce distorted probabilities during observation, which would mean compiled artifacts inherit the distortion rather than merely being mis-measured.

There is a second data point consistent with this. A recent HF-path `evaluate-report` on SmolLM2-135M-Instruct returned a **teacher floor of 14.84 bits/token** against a uniform bound of roughly 15.6 for a ~49k vocabulary — i.e. the teacher scoring barely better than chance on its own held-out set, with the artifact at 37.15 bits. Two teacher-side anomalies on two different paths.

## Suggested first checks

1. Is the teacher log-probability path computed in log space end to end, or is there a linear-space product that can underflow?
2. Instrument a certified run to count observed tokens receiving p below f32 minimum, and report the count and the tokens.

## Reproduction

The environment block below; `cargo run --release -- certify`.

## Environment

Commit `0ffb71b94d78ff722af089f1337a08fce42dccff` (HEAD of main, 2026-07-26). rustc 1.95.0, `--release`. macOS 26.5.1, arm64 — the platform the κ baseline is pinned to. Fresh clone; no pre-migration store reused. Fixture `/tmp/ref/out/model.bin` present and size-verified (60,816,028 bytes), source κ confirmed by `certify` as `blake3:0ae73395…`. Corpus built with `gen 300 150000`: 150,000 tokens, 754 stories, 30,036 held out.


## Comments (1)

### comment 5099751465 — 2026-07-28T03:55:57Z — Casey-allard

Taking this (assigned). Verified diagnosis differs from the log-space hypothesis — plan:

**Root cause** (code-verified): the legacy floor is not computed from live logits at all. `certify.rs:150-158` reads the corpus-recorded **integer-percent quantized** top-3 weights (`softmax_top3_sample`, `compiler.rs:295`, rounds to whole percents). Any top-3 token with renormalized p < 0.5% records weight **0**; corpus `next` is sampled from the full CDF (`compiler.rs:311-321`) so it can land on that token → `-ln(0/100)` = +inf; one collision poisons the whole sum. Both softmaxes are max-stabilized — no underflow anywhere. The certified 1.5960 run simply had no collision. Artifacts are unaffected (only argmax/top-k *ids* are stored; zero weights are dropped at store build by `if weight > 0`).

**Fix** (branch `issue-231-teacher-floor-clamp`):
- Extract a pure `recorded_next_prob(top_tokens, top_weights, next) -> (f64, bool)`: weight-0 collisions clamp to **0.005** (half a quantization step — the same clamp discipline the HF path already applies at `graph-cli/src/lib.rs:1444`), flagged.
- Print the collision counter in the floor line (`zero-quantized-weight collisions: N/ntest`) — the instrumentation this issue requested, aimed at quantization rather than f32-min.
- Unit tests on the extracted function (top-0 hit, zero-weight top-2 hit + clamp flag, outside-top-3 fallback, finiteness).

**Not changed**: the HF-path floor (already clamped, and its ≈-uniform anomaly is the tokenizer defect, #242).



---
_Comment/review ids in this file: 5099751465_

