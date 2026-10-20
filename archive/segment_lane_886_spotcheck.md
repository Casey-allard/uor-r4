# Segment-lane deployed bounded spot-check (#886)

Empirical record. S1 follow-up A of #836 (verdict REVISE, lane dormant); parent
programme #822. This is a **bounded** minimal-pairs spot-check of the deployed
segment lane on the real #833 canonical bundle — not the full N≈24k causal run
(that outcome is already fixed by the #836 ceiling < floor arithmetic). Records
in `docs/` are appended to, not rewritten.

Harness: `crates/uor-r4-api/tests/segment_lane_spotcheck_886.rs`
(`--ignored`; reference-only, off the serving default). Machine result:
`docs/segment_lane_886_result.json`. Run contract: #886 issue comment.

## Question

#836 lowered the #835 segment lane onto the deployed `R4Engine` and closed
REVISE by reachability arithmetic — the reference-arm ceiling (+17.5‰, CI upper
19.0‰) is below the 20‰ causal floor, and the deployed lane is strictly weaker
(integer-`ScoreQ` quantization, the top-8 candidate bound, and the 8-slot content
ring). The #836 tests exercise the lane on **synthetic** fixtures only.

This check asks the cheap question #886 poses: on the **real** bundle, does the
deployed lowering (fit → emit → consume) realize the whole-prompt-content signal
on its own best cases — the exact minimal pairs the #834 §6.2 reference arm
resolved — within that ceiling?

## Method

1. **Reproduce the §6.2 favorable pairs** deterministically: rebuild the
   reference arm's suffix/content tables from the corpus TRAIN split, mine the
   minimal pairs (same 2-token suffix, different story, different teacher
   argmax, one pair per suffix key), and keep exactly the pairs the reference Ψ
   scorer followed. This reproduced `psi_follow = 10 / 4722`
   (`docs/psi_arm_834_result.json`) exactly — a reproduction control the harness
   asserts, alongside the §6.2 `baseline_follow = 0` control degeneracy.
2. **Fit + emit + consume the deployed table**:
   `segment_fit::fit_segment_table` (top-64/key, quantized to integer `ScoreQ`)
   over the same corpus → 19,710 learned content keys →
   `convert_r4g1::convert_with_segment_table` over the real store/artifacts →
   `R4Engine`, which consumed every fitted row (asserted via
   `segment_learned_rows`).
3. **Replay the deployed lane** on each favorable position: a freshly reset
   serving session, the whole-prompt window folded into an active
   `SegmentSession`, and the served token taken from
   `predict_decision_candidates_with_segment` (the P-4 quantized top-8 re-rank).
   A pair "follows" when the lane serves each side's own teacher target and the
   two differ.

## Emission-path note (measured, load-bearing for interpretation)

The segment lane's only emission path is
`convert_r4g1::convert_with_segment_table` (the transformerless TLS→R4G1
converter). The **released** bundle's serving graph `graph/score.r4g1` is
emitted by a different path (the graph-compiler cover/score emitter,
`score::emit_scored_r4g1`) and carries no PSTATE section, so it cannot drive the
lane. "The deployed lane on the real bundle" is therefore the fitted table
emitted by `convert_with_segment_table` and consumed by `R4Engine` — exactly
what #836 lowered — not the released cover/score graph. The two emitters are not
serving-equivalent on held-out data: under the deployed policy the re-emitted
graph left **6 of the 20** favorable-pair positions **unservable** (the widen
re-probe the released cover graph supports is not satisfiable on the re-emitted
graph; the harness catches and classifies these rather than aborting), whereas
`causal_prompt_run_834` served 20k held-out windows on the released graph.

## Result

| quantity | value |
|---|---|
| reference favorable pairs (reproduced) | 10 / 10 expected |
| reference baseline-follow (control) | 0 |
| fitted learned content keys | 19,710 |
| positions evaluated | 20 |
| served / abstained / unservable | 14 / 0 / 6 |
| **deployed-lane follow** | **1 / 10** |
| predeclared threshold for "faithful" | ≥ 6 / 10 |

Among the 6 pairs where **both** positions served, 1 followed; the other 4 pairs
each had at least one unservable position. Served tokens concentrated on a small
set of high-frequency candidates (e.g. tokens 28 and 30), so the re-rank rarely
tracked the position-specific teacher target.

## Read (predeclared, § Definition of done)

**LOWERING-FIDELITY GAP.** The deployed lane followed 1/10 < 6/10 of the
reference arm's favorable pairs, so the deployed lowering does **not** track its
own ceiling even on its best cases. The result is robust to the 6 unservable
positions: excluding them, follow was 1/6. This is the predeclared `< 6/10`
branch and an evidence-backed negative — a legitimate outcome, recorded as such.

Consistent with #836's REVISE (the deployed lane is real but sub-floor and
dormant), this check adds that on the real bundle the deployed lowering is
weaker than the +17.5‰ reference ceiling would suggest, and that the lane's only
emission path does not reproduce the released serving graph's held-out behavior.

## Scope / conformance

Reference-only / off the serving default; measurement, not deployment. The lane
stays **dormant**: no `model/ids.toml` serving row, `CONFORMANCE.md` unchanged,
the #836 activation gate unchanged (re-clear the 20‰ causal floor on the packed
serving path). RF-21, RF-22, RF-27, RF-28. No hours-long run launched.

## Implications (for the maintainer; not actioned here)

The gap and the emission-path non-equivalence bear on the ordering of the
remaining S1 follow-ups (#887 floor governance, #888 candidate-support widening):
a mechanism widening (#888) sits atop a lowering that does not yet track its own
ceiling, and #887's floor question is about a lane whose deployed realization is
weaker than the reference ceiling it was measured against. Left to the maintainer
to weigh; this issue records the measurement only.
