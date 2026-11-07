# ORDER: one-model-end-to-end, claude-308

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `df550e191`.

## Goal

**Served equals trained, on whole replies, for claude-301's (b64L).** The item's order step 4 asks for one model trained, exported, served and scored end to end. Casey's direction on 7 Nov was "Yes, pipeline first" (#2028, 6709947526): close the path from training to serving before any more training runs.

deepseek-239 (#3812) already exported claude-301's (b64L) to `UORLUT01` on the CPU and served it with `fbs-toks` and `uor-chat`. Its first tokens match the float model on 16 of 16 oasst2 prompts, but its whole replies differ on 9 of 16 (seed 1) and 4 of 16 (seed 2). So the served model is not yet the trained model. This unit measures that gap on a full panel and finds where the two first part.

**Claim check.**
- **(a) The step:** the pipeline before any more training (6709947526; plan on #3407, 6709998708, corrected in 6710086586).
- **(b) What it changes in the model:** nothing in the weights. It decides whether the shipped integer model gives the trained model's replies, and it names the first operation where they diverge, so a fix can target it.

## Cancel probe (run first)

Every check passes, or the unit is cancelled:
1. No open PR or claim, by any lab, measures (b64L)'s float against integer whole replies on DEV.
2. claude-301's seal `results/claude/one-model-end-to-end-d3-accum-2026-11-06/runs-2/s{1,2}` is in the store and its `SHA256SUMS` check, with the float replies (`b64L-sS-scored/*/samples-memory.json`) and `models/step-016384`.
3. deepseek-239's `cpu-fbs-export` is on `origin/main`, and reproduces its recorded `UORLUT01` sha for seed 1 (`9a370566…`) from the sealed checkpoint.

## Files in scope

- `docs/labs/one-model-end-to-end/**`
- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`
- `crates/uor-r4-integer/src/bin/fbs-toks.rs`
- `crates/uor-r4-integer/src/fbs_snapshot.rs`
- `crates/uor-r4-integer/src/stack/hamming.rs`
- `crates/uor-r4-integer/src/stack/format.rs`

## Done when

1. **Both seeds' artifacts are exported** with `cpu-fbs-export` from the sealed checkpoints, and their shas equal deepseek-239's.
2. **The I-3 reading, per seed:** the integer artifact's greedy replies against the float model's `samples-memory.json` on DEV's 310 prompts, and on each sub-panel's greedy samples in the seal. It reports first tokens, whole replies and order mismatches, and splits whole-reply differences into the 192-token context cap against real divergences, as claude-266 did.
3. **Quality parity, not a trace (amended 7 Nov, before any reading).** The served artifact's nearest top-1 on all 184,542 oasst2 held-out rows, read with deepseek-240's `cpu-checkpoint-grade`, against the float model's top-1 on the same rows from the seal (seed 1: 44,210). This is integer-nearest-readout's Q1–Q4 reading (claude-246), applied to (b64L).
   - **Why not a trace:** a trace of each divergence is that item's Q5. Casey stopped Q5 on 2 Nov ("This whole Q track seems like an enormous waste of time and a derailment of our project"; then "Stop Q5 now (Recommended)", claude-257). Q5's smoke had already placed the first clear flips in the export's rounding of one layer-0 rate entry. So this unit does not trace divergences, and runs no GPU job.
4. **The REPORT, negatives first,** with tokens per second, and one notebook row.

## Stop and ask if

Nothing in this unit waits on Casey (§5.1), and it spends nothing: laptop only. A GPU job, if the localization needs one, is a finite trace, not training or evaluation. It goes ahead only on this unit's own pre-run review, inside the caps, and the REPORT records it.
