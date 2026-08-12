# Prime Transport Router Reintegration v6 — D=16 Extended Budget Scaling

**Status:** Complete
**Surface version:** v25 (geometry_native_operator_model_v25)
**Execution path:** run_router_reintegration_v6_opt.py (batched, state_trans_cache)
**Script:** tools/prime_transport/run_v6_d16_scaling.py

## Purpose

Determine whether D=16 accuracy continues to improve materially beyond 5000 batches
on the optimized v6 path, or whether it is approaching a true ceiling for the current
step-0-only additive injection interface.

## Setup

- Delay: D=16 only (chance = 0.250)
- Budgets: 5000, 7500, 10000, 15000 batches
- Batch size: 32 episodes per batch
- Vocab: 4 tokens
- Temperature: matches v6_opt for first 5000 batches (TEMP_START=2.0→TEMP_END=0.1),
  then holds 0.1 for batches 5000+

### Was 5000b reused or rerun?

Re-run. The 5000b checkpoint is produced as part of this continuous training run to 15000b.
The temperature schedule matches v6_opt exactly for the first 5000 batches, making the 5000b
checkpoint directly comparable to the v6_opt serial and batched references.

- v6 serial reference @5000b = 0.419  (run_router_reintegration_v6.py)
- v6 opt reference    @5000b = 0.478  (run_router_reintegration_v6_opt.py, seed=42+D)

## Results

| metric | 5000b | 7500b | 10000b | 15000b |
| --- | -------- | -------- | -------- | -------- |
| accuracy | 0.421 | 0.734 | 1.0 | 1.0 |
| vs_chance | +0.1710 | +0.4840 | +0.7500 | +0.7500 |
| delta_vs_prev | n/a | +0.3130 | +0.2660 | +0.0000 |
| delta_vs_v6serial | +0.0020 | +0.3150 | +0.5810 | +0.5810 |
| route_entropy | 1.6044 | 1.3098 | 1.1329 | 1.1310 |
| transport_frac | 0.9854 | 1.0000 | 1.0000 | 1.0000 |
| alpha0 | 0.0625 | 0.0627 | 0.7143 | 0.7366 |
| temp_at_end | 0.1000 | 0.1000 | 0.1000 | 0.1000 |

## Is Improvement Material or Saturating?

**Solved.** D=16 accuracy reaches **1.000 at 10,000b**. The final interval (10000b → 15000b) shows
+0.000 gain because accuracy is already perfect — saturation is at the ceiling, not at a partial value.

Key trajectory:
- 5000b → 7500b: +0.313 (rapid climb begins)
- 7500b → 10000b: +0.266 (convergence to 1.000)
- 10000b → 15000b: +0.000 (already perfect)

Final accuracy at 15000b: **1.0000** (vs chance=0.250, Δ=+0.7500).

**Critical revision of prior conclusion:** The D=16 "ceiling" observed at 0.419 (5000b) and 0.478 (v6_opt
5000b) was a **training budget artifact**, not a structural limit of the additive step-0 injection
interface. With 10,000 batches, D=16 achieves the same perfect accuracy as D=2/4/8.

Peak single-interval gain: +0.3130.

## Routing Behavior

Route entropy remained above 1.0 throughout all budgets. Routing is non-collapsed and mixed.

Route entropy values: 1.6044, 1.3098, 1.1329, 1.1310.

## Transport Usage

Transport usage fraction: 0.9854, 1.0000, 1.0000, 1.0000.
Trend: increasing with training budget.

## Attention alpha0 Behavior

Expected for D=16 uniform attention: 1/16 = 0.0625.

alpha0 = 0.7366 at 15000b — well above uniform. The model learned to strongly concentrate
attention on the encoding step (t=0) once sufficient training was provided.

**Key transition:** alpha0 was near-uniform (0.0625) through 7500b, then jumped to 0.7143 at
10000b — coinciding exactly with the accuracy jump to 1.000. This confirms the same mechanism
as D=2/4/8: the model learns to upweight the injection step, which concentrates the
step-0-encoded token identity back into the prediction.

alpha0 values by budget: 0.0625, 0.0627, 0.7143, 0.7366.

## Cache Performance

- Cache hit rate: 96.0%
- Wall time for D=16 full study: 129.9s
- Total script time: 132.1s

## Honesty Section

**What improved:**
- D=16 accuracy improved from the v6_serial reference of 0.419 to **1.000 at 10,000b** (+0.581).
- alpha0 jumped from near-uniform (0.0625 at 5000–7500b) to 0.714 at 10,000b, confirming the
  attention mechanism learned to concentrate on the encoding step.
- Route entropy narrowed (1.604 → 1.133), indicating more specialized routing.
- The v6 step-0-only injection interface is NOT limited to partial D=16 accuracy; it achieves
  perfect accuracy with sufficient training.

**What saturated:**
- Accuracy reached 1.000 at 10,000b and did not change at 15,000b (saturation at the ceiling).
- Transport usage fraction reached 1.000 at 7,500b and remained there.

**What remains uncertain:**
- Whether the ~10,000b threshold for D=16 is robust across different random seeds, or whether
  this particular seed happens to converge faster/slower than average.
- The budget required for perfect accuracy at D > 16 (D=24, 32, 48) — this is the natural
  next experiment.

## Next Step

**Extend context-length scaling beyond D=16.** All tested delays now achieve 1.000 accuracy:
D=2/4/8 at 5,000b and D=16 at 10,000b. The interface is validated across the full tested range.

The next scientifically motivated experiment is a context-length scaling study over
D in {16, 24, 32, 48} at a fixed budget (10,000b) using the current v6_opt path,
to identify at what delay length perfect accuracy first fails and whether the degradation
is gradual or sharp.

## Files

- Script (wrapper): `tools/prime_transport/run_v6_d16_scaling.py`
- No core files modified (spin_H_core_v6, sigma_family_holonomy_law_v6,
  coupled_holonomy_residue_v6, operator models v20–v25 all untouched).
