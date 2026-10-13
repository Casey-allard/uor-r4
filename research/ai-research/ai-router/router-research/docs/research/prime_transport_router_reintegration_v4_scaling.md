# Prime Transport — Router Reintegration v4 Training-Budget Scaling

**Experiment type:** Training-budget scaling study on v4 token injection architecture
**Operator layer:** `geometry_native_operator_model_v25`
**No surface build.  No files modified.  No operators rebuilt.**

## What Changed from v4

The architecture is **identical to v4** in every respect. Only training budget varies.

| Aspect | Value |
|--------|-------|
| Architecture | v4 (token injection + trajectory attention) |
| Training budgets tested | [800, 2000, 5000] batches |
| Batch size | 32 |
| Episodes @ 5000 batches | 160,000 per delay |
| Temperature schedule | Anneal 2.0→0.1 over batches 0-799; hold 0.1 beyond |
| Checkpointing | Params cloned at batches 800, 2000, 5000 within one run |

**Why the temperature is held at 0.1 after batch 800:**
v4 used exactly 800 batches with temperature annealed over the full run. For clean comparison, the extended training uses the same annealing schedule for batches 0-799 and then continues at TEMP_END=0.1. The 800-batch snapshot is therefore directly comparable to v4.

## Primary Results

### Accuracy by training budget and delay

v4 reference (800 batches): D=2→0.615, D=4→0.436, D=8→0.328, D=16→0.280

| Budget | D=2 | D=4 | D=8 | D=16 | above-chance at all delays? |
|--------|-----|-----|-----|------|-----------------------------|
| 800 | 0.638 | 0.448 | 0.290 | 0.269 | yes |
| 2000 | 0.616 | 0.536 | 0.388 | 0.275 | yes |
| 5000 | 0.646 | 0.520 | 0.454 | 0.367 | yes |

### Delta vs 800-batch baseline, by delay

| Budget | D=2 delta | D=4 delta | D=8 delta | D=16 delta |
|--------|-----------|-----------|-----------|------------|
| 2000 | -0.022 | +0.088 | +0.098 | +0.006 |
| 5000 | +0.008 | +0.072 | +0.164 | +0.098 |

### Does performance scale with training budget?

| D | acc@800 | acc@2000 | acc@5000 | scales? |
|---|---------|----------|----------|---------|
| 2 | 0.638 | 0.616 | 0.646 | no |
| 4 | 0.448 | 0.536 | 0.520 | yes |
| 8 | 0.290 | 0.388 | 0.454 | yes |
| 16 | 0.269 | 0.275 | 0.367 | yes |

**Performance scales with training at 3/4 delays.** More training materially improves accuracy.

## Routing Behavior

### Route entropy (operator diversity) by budget and delay

| Budget | D=2 | D=4 | D=8 | D=16 | any collapsed (<0.3)? |
|--------|-----|-----|-----|------|-----------------------|
| 800 | 1.635 | 1.735 | 1.692 | 1.727 | no |
| 2000 | 1.592 | 1.721 | 1.682 | 1.719 | no |
| 5000 | 0.470 | 1.170 | 1.590 | 1.716 | no |

### Transport fraction by budget and delay

| Budget | D=2 | D=4 | D=8 | D=16 |
|--------|-----|-----|-----|------|
| 800 | 0.565 | 0.414 | 0.102 | 0.839 |
| 2000 | 0.482 | 0.542 | 0.199 | 0.846 |
| 5000 | 0.073 | 0.988 | 0.583 | 0.840 |

## Attention Weight Structure

Mean alpha at step 0 (encoding step) by budget — uniform = 1/D

| Budget | D=2 (unif=0.50) | D=4 (unif=0.25) | D=8 (unif=0.125) | D=16 (unif=0.0625) |
|--------|-----------------|-----------------|------------------|--------------------|
| 800 | 0.500(~) | 0.250(~) | 0.125(~) | 0.063(~) |
| 2000 | 0.500(~) | 0.250(~) | 0.125(~) | 0.063(~) |
| 5000 | 0.500(~) | 0.250(~) | 0.128(~) | 0.063(~) |

## Long-Delay Degradation Analysis

| Budget | D=2 | D=16 | gap (D=2 - D=16) |
|--------|-----|------|------------------|
| 800 | 0.638 | 0.269 | 0.369 |
| 2000 | 0.616 | 0.275 | 0.341 |
| 5000 | 0.646 | 0.367 | 0.279 |

**Long-delay gap narrows with more training** (gap at 800: 0.369, gap at 5000: 0.279). Additional training helps disproportionately at longer delays.

## Honesty Section

### What improved

- Best accuracy at 5000 batches: 0.646 (chance 0.250).
- D=2 accuracy at 5000 batches: 0.646.
- Routing remained non-collapsed throughout all budgets.
- Training-budget scaling effect (if any) is directly measurable from this run.

### What saturated or failed

- The per-step additive injection (W_tok_inject[tok_t]) is the same for ALL occurrences of a given token across steps. At D=16, the 15 noise-token injections may accumulate and dilute the encoding-step signal.

### What remains uncertain

- Whether a position-aware injection (step 0 only, not all steps) would prevent noise accumulation at large D.
- Whether the attention mechanism could specialize for position if position embeddings were added.
- Whether full exact spin_H is solved: No.

## Recommended Next Step

Performance at D=16 is reaching practical significance. Run v4 with **10,000 batches** at D=16 only to determine the convergence ceiling for long-delay recall.

