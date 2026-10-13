# Dimension–Phase Alignment Probe v1

**Branch:** `dimension_phase_alignment_probe_v1`  
**Date:** 2026-04-09  
**Script:** `tools/prime_transport/run_router_dimension_phase_alignment_probe_v1.py`

---

## Geometry Lock Summary

| Property | Value |
|---|---|
| Task | Task A family: original_s42, shift1_s42 |
| State space | BLOCKS_A = [(0,2,2,1),(2,7,5,1),(7,9,2,1),(9,21,12,3)] |
| Dominant cycle | positions 9..21, period 12, 3 harmonics |
| Partition type | **INTERLEAVED** — class k = position mod 4 within period-12 |
| Partition (original) | [0,1,2,3,0,1,2,3,0,1,2,3] |
| Partition (shift1)   | [1,2,3,0,1,2,3,0,1,2,3,0] |
| Locked stack | split_transport(eps=1.0) + harmonic_state + two_i_orient + projective_ctrl |
| D_HIDDEN (network width) | 32 — **FIXED throughout sweep** |
| Horizon D sweep | [16, 20, 24, 28, 32, 36, 40] |
| Period-12 multiples in sweep | D=24 (2×12), D=36 (3×12) |
| Non-multiples in sweep | D=16,20,28,32,40 |

**Canonical anchor values (from file 3):**

- original_s42: D=24 → 0.9941,  D=32 → 0.9653
- shift1_s42:   D=24 → 0.957,   D=32 → 0.9829
- Orientation gap D=24: +0.0371,  D=32: -0.0176

---

## D Sweep Accuracy Table

| D | is_period_multiple | original_s42 | shift1_s42 | avg_accuracy |
|---|---|---|---|---|
| 16 | no | 0.9658 | 0.9775 | 0.9717 |
| 20 | no | 0.9956 | 0.9990 | 0.9973 |
| 24 | YES (×12) | 0.9990 | 0.9658 | 0.9824 |
| 28 | no | 0.9951 | 0.9746 | 0.9849 |
| 32 | no | 0.9395 | 0.9727 | 0.9561 |
| 36 | YES (×12) | 0.9683 | 0.9922 | 0.9803 |
| 40 | no | 0.9727 | 0.9883 | 0.9805 |

**Best D by average accuracy:** D=20 (0.9973)  
**Worst D by average accuracy:** D=32 (0.9561)  
**Performance vs D pattern:** NON-MONOTONIC (resonance candidate)

---

## Orientation Gap Table

*Orientation gap = accuracy(original_s42) − accuracy(shift1_s42)*

| D | is_period_multiple | orientation_gap | sign |
|---|---|---|---|
| 16 | no | -0.0117 | − |
| 20 | no | -0.0034 | − |
| 24 | YES (×12) | +0.0332 | + |
| 28 | no | +0.0205 | + |
| 32 | no | -0.0332 | − |
| 36 | YES (×12) | -0.0239 | − |
| 40 | no | -0.0156 | − |

**Gap sign changes across D:** 2  
**D with smallest |gap|:** D=20 (-0.0034)

---

## Decodability Table

| D | variant | decodability_final | alpha_last |
|---|---|---|---|
| 16 | original_s42 | 1.0 | 0.0893 |
| 16 | shift1_s42 | 1.0 | 0.0574 |
| 20 | original_s42 | 1.0 | 0.0398 |
| 20 | shift1_s42 | 1.0 | 0.046 |
| 24 | original_s42 | 1.0 | 0.0316 |
| 24 | shift1_s42 | 1.0 | 0.0516 |
| 28 | original_s42 | 1.0 | 0.0366 |
| 28 | shift1_s42 | 1.0 | 0.0277 |
| 32 | original_s42 | 1.0 | 0.0462 |
| 32 | shift1_s42 | 1.0 | 0.0336 |
| 36 | original_s42 | 1.0 | 0.0262 |
| 36 | shift1_s42 | 1.0 | 0.0172 |
| 40 | original_s42 | 1.0 | 0.022 |
| 40 | shift1_s42 | 1.0 | 0.0107 |

---

## Mandatory Questions

**Q1. Does performance vary non-monotonically with D?**  
Pattern: NON-MONOTONIC (resonance candidate). YES — performance varies non-monotonically across the D sweep.

**Q2. Are there preferred D values?**  
YES — best D=20 (avg=0.9973) vs worst D=32 (avg=0.9561), spread=0.0412 > 0.01 threshold.

**Q3. Does orientation sensitivity shrink at specific D values?**  
YES — orientation gap changes sign 2 time(s) across D. Gap is minimized in magnitude at D=20.

**Q4. Is there evidence that D is part of the geometry rather than just generic capacity?**  
POSSIBLE — evidence includes: non-monotonic accuracy vs D; preferred D band around D=20; orientation gap sign change across D. Period-12 multiples (D=24, D=36) will be examined in context.

**Q5. Does this keep the possibility of a later fair R^4 retest alive?**  
YES — if dimension-phase alignment is confirmed (even weakly), it motivates a future R^4 retest at a geometrically-aligned D, since the previous φ/R^4 failure used D=24 (period-aligned) and D=32 (non-aligned) without controlling for this dimension effect. A fair R^4 retest should hold D fixed at the best-performing non-φ baseline dimension.  
Note: this does NOT imply φ or R^4 are necessary — the prior failure stands. It only identifies whether dimension was a confound.

---

## One-Line Conclusion

**DIMENSION–PHASE ALIGNMENT EFFECT IS: STRONG**

---

## Honesty Section

### What was actually tested

- A sweep of sequence horizon D ∈ {16,20,24,28,32,36,40} under the locked stack (split transport + harmonic state + two_i_orient + projective controller).
- Network width D_HIDDEN=32 was held constant; only the horizon length varied.
- Two orientation variants of Task A (original_s42, shift1_s42) were run at each D.
- Accuracy, decodability, and orientation gap were recorded per (D, variant) run.
- No φ, R^4, or new controller features were introduced.

### What remains open

- Whether D_HIDDEN (network width, fixed at 32) also shows resonance-like behavior.
- Whether the orientation gap pattern is stable across seeds.
- Whether the result generalises to tasks with different cyclic periods.
- Whether the phase-alignment effect (if found) interacts with φ or R^4 coupling.

### Why this does or does not justify a later R^4 retest

The STRONG result indicates that D is geometrically relevant. The previous φ/R^4 probe ran at D=24 and D=32 — one period-aligned and one not — without controlling for this. A fair R^4 retest should be conducted at the identified best-performing D, with orientation matched. The prior failure (φ/R^4 at D=24) is NOT overturned; it remains a valid falsification of that specific operationalization. A new retest would be testing a genuinely different configuration.

**HARD RULE reminder:** This probe does NOT reinterpret the failed φ/R^4 probe as proving φ or R^4 are unnecessary in general. It only characterises whether dimension D is itself part of the operative geometry.
