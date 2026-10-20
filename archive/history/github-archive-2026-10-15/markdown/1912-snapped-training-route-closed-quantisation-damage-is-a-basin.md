# #1912 — Snapped-training route closed: quantisation damage is a basin effect, not a dose (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T14:13:45Z
- **Updated:** 2026-10-09T13:22:25Z
- **Closed:** 2026-10-08T14:15:24Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1912
- **Merged:** 2026-10-08T14:15:24Z
- **Merge commit:** bd8b9b92ec125799c5b84cc135e882ac3b4687ef
- **Base / Head:** main / UOR-Foundation:deepseek/snap-sweep
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## The sweep

Same 19.9 M chat stack, same held-out stream (`e5f400b0…`), same evaluation — `snap-evaluate` (512 windows) for the float side, `export` with GPTQ at the measured optimum + `d11-evaluate` (24,576 targets) for the served side:

| arm | snapped float | **served (D11)** | quantisation cost |
|---|---:|---:|---:|
| no tune | 0.91333 | **0.93277** ← best | 0.01944 BPB |
| 250-step | 0.89420 | 0.95540 | **0.06117 BPB** |
| 500-step | 0.89113 | 0.95150 | 0.06033 BPB |

## Three findings

1. **Both tuned arms serve worse than no tune** (+0.02258 and +0.01868 BPB) despite better float scores. The float gain does not survive the artifact.
2. **The damage is not dose-dependent.** Quantisation cost **jumps to 0.06117 on entering the snapped basin with 250 steps and then plateaus** — 500 steps costs slightly *less*. The first snapped steps do the damage; more steps don't add to it.
3. **So the mechanism is a basin effect, not a length effect** — and a shorter tune is not the fix. That was exactly the hypothesis this sweep was run to test, and it fails.

It also shows the served ordering (no tune < 500 < 250) is **not** the float ordering (500 < 250 < no tune): **anyone selecting on the float proxy picks the wrong arm.**

## What it closes, and what stays open

**Closed:** recovering the 0.03578 BPB transport-snap term by training in the snapped representation and exporting with GPTQ. It costs more in servability than it returns, at every length tested, and the cost appears at the *first* snapped step.

**Open, and now better aimed:** the quantiser does not tolerate the snapped basin. That needs either a bit allocation that does (round 20 says the weight matrices carry the loss — but `rounding-attribution` refuses snapped artifacts, so the measurement doesn't exist yet), or serving the snap differently rather than training into it. **Neither is a sweep, and neither is claimed here.**

**The best served result on real held-out chat remains the untuned 0.93277 BPB — 4.70× better than the 4.3885 this project serves today.**

Scope: one model, one corpus, one stream, two tune lengths, single seed each, one quantiser setting; arms differ only in fine-tune steps. Evidence: `docs/evidence/snapped_tune_sweep_2026-10-08.txt`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

