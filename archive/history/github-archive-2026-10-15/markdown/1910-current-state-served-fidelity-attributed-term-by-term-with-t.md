# #1910 — Current state: served fidelity attributed term by term, with two honest nulls (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T14:02:36Z
- **Updated:** 2026-10-09T13:22:37Z
- **Closed:** 2026-10-08T14:04:16Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1910
- **Merged:** 2026-10-08T14:04:16Z
- **Merge commit:** c623d9735695abbd2da1da8efff007bd3eb41d32
- **Base / Head:** main / UOR-Foundation:deepseek/state-rounds19-31
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this integrates

`docs/integration/current-state.md` was 12 rounds stale — its last entry covered up to round 19. This adds rounds 19–31 as the newest section.

**The gap decomposes, and both quantisers sit at an optimum.** Snap **0.03578 BPB** (64.8 %) + quantisation **0.01944 BPB** (35.2 %), with **97 % of the quantisation term in the recurrence/read weight matrices** (`geometric-stack.rs:3374`) rather than the scalars. Measured on one model and one integer evaluation: **GPTQ beats round-to-nearest by 0.00376 BPB**, and 64 windows with damp 0.01 is an **interior optimum** (4× windows with finer damping: 0.00091 worse; no calibration: 0.00376 worse). **The export path has nothing left to tune.**

**The largest term has a cause and a price.** `transport_snap=none|icosian` is a *training-time* option the served model never passed — it trained continuous and was snapped post hoc. Training with it costs **~an order of magnitude of throughput** (GPU 100 % → 15–25 %; 336 % CPU, 38–40 min CPU in 11–12 min wall).

**Two nulls, recorded as served results.** A 500-step snapped fine-tune moved the **float** score the right way (**0.91333 → 0.89113**, snap becoming a *gain*) but the **exported artifact regressed to 0.95150** against the untuned **0.93277**, because quantisation loss grew **3.1×**. Second time tonight a float gain failed to survive to the deliverable — and the sharper case, since the float moved correctly while the artifact regressed. **The served number is the result.**

**Two tool limits**, recorded so they aren't rediscovered: `rounding-attribution` refuses snapped artifacts (it measures the D10 comparator's free transport); `d11-evaluate` needs `model=` for a snapped artifact. The round-30 attribution follow-up is **withdrawn** on that basis.

**Reproduction:** independent retrain, different GPU/commit/pod → served nll to **2.1e-6**, top-1 agreement 1.0. **Job contention** costs 42 % of training throughput.

**Next, ordered in the section:** finish the fine-tune-length sweep reading the *served* number; **buy data coverage before parameters** (0.1592 vs 0.0837) — blocked, no chat corpus is published; and find a route for snapped-training quantisation that the existing attribution tool can't measure.

Round 16's **0.93277 remains the best served result on real held-out chat.**

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

