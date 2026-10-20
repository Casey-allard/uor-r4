# Skip-mix lane gap diagnostic (#904, follow-up to #897)

Empirical record. Diagnoses the #897 LOWERING-FIDELITY GAP
(`docs/skipmix_lane_897_result.json`: deployed-lane follow 41/87, below the
predeclared 53/87 bar) before any fix is chosen, per Casey's "diagnose the
gap first" sign-off. Records in `docs/` are appended to, not rewritten.

Harness: `crates/uor-r4-api/tests/skipmix_gap_diagnostic_904.rs`
(`--ignored`; reference-only, off the serving default; no production code
changes). Machine result: `docs/skipmix_gap_diagnostic_904_result.json`.
Run contract: #904 issue comments.

## Correction to the #897 record

`docs/skipmix_lane_897_spotcheck.md` named "the per-key top-64 cap" as a
suspect. It is not: the reference `Tables` construction and the deployed
`skipmix_fit::fit_skipmix_tables` both cap at the identical
`CAP = DEFAULT_TOP_K = 64`. This diagnostic supersedes that attribution —
the cap was never a differentiator between the two arms.

## Method

Reusing the exact fit -> emit(real emitter) -> consume machinery of
`skipmix_lane_spotcheck_897.rs` (same corpus, same fit, same real bundle),
the same 87 reference-favorable pairs (174 positions) were replayed with
three added, decomposed measurements, plus one instrument-fidelity control:

1. **Candidate coverage ceiling (arm 1).** The deployed re-rank
   (`predict_decision_candidates_with_skipmix`) chooses only among
   `candidates.ranked()`, hard-capped at
   `StepCandidates::STEP_TOP_CANDIDATES = 8`
   (`uor-r4-graph-certify/src/score_runtime.rs:1898`). The reference arm's
   `skipmix_scores` is evaluated over `d1_cands`, a suffix/content/joint
   union up to ~97 tokens. For each pair, is the teacher target present at
   all in the engine's real top-8 list on each side? This bounds the
   maximum any re-ranking policy tied to this candidate list could ever
   follow.
2. **Reference math restricted to the narrow (real, top-8) candidate list
   (arm 2).** The validated, unquantized, floating-point `skipmix_scores`
   formula, evaluated only over each position's actual top-8 tokens —
   isolates the pure candidate-breadth effect on an otherwise-idealized
   scorer.
3. **Real base `ScoreQ` + a unit-safe, non-additive combine (arm 3).** The
   deployed lane adds `segment_fit::quantize_rate`'s linear, `2^20`-scaled
   rate directly onto `ScoreQ.raw()` (a log-probability scaled by `2^16`;
   `skipmix_scale_by_lambda_and_support` is the identity function — no
   calibration reconciles the two scales). This arm instead ranks candidates
   by (has skip-mix support, contribution magnitude, base `ScoreQ`, token id)
   — never summing across the two incompatible scales — to test whether the
   additive combination itself, independent of breadth, is costing
   follow-throughs.

**Instrument-fidelity control**: a manual, map-based replica of the engine's
own additive combine rule was required to reproduce the engine's actual
served token on every one of the 174 positions (`instrument_mismatches ==
0`), and the recomputed deployed-lane follow count was required to match
the #897 record exactly (`arm0_deployed_follow == 41`) — both passed,
confirming arms 1-3 measure what they claim to.

## Result

| quantity | value |
|---|---|
| reference favorable pairs (reproduced) | 87 / 87 expected |
| positions (served / abstained / unservable) | 174 / 0 / 0 |
| instrument-fidelity control | 0 mismatches (pass) |
| **arm 0 — deployed lane (reproduced)** | **41 / 87** (matches #897 exactly) |
| **arm 1 — candidate coverage ceiling** | **52 / 87** |
| **arm 2 — idealized reference math, boxed to the real top-8 list** | **52 / 87** |
| **arm 3 — real base score + unit-safe combine, top-8 list** | **40 / 87** |

## Read

**BREADTH-BOUND.** The candidate-list breadth (`StepCandidates::
STEP_TOP_CANDIDATES = 8`) is the dominant, primary cause of the gap, not
quantization or the additive combine formula:

* **35 of 87 pairs (40%) are structurally unreachable**: at least one side's
  teacher target is not even among the engine's own top-8 base-candidate
  list, before any skip-mix scoring runs. No combine-rule change, no
  calibration, no re-quantization can ever recover these — only a wider
  served candidate list could.
* **Among the 52 reachable pairs, the idealized (unquantized, float)
  reference formula follows all 52 (100%)** when boxed to the exact same
  top-8 list the deployed engine sees. This shows the *signal itself*
  survives the narrowing perfectly; nothing about restricting to 8
  candidates destroys the reference math's discriminative power on its own
  best cases.
* **The deployed (quantized, additive) lane only follows 41 of those same
  52 reachable pairs (79%)** — an 11-pair residual gap attributable to
  quantization/formula effects, separate from breadth.
* **The proposed unit-safe (non-additive) combine (arm 3) does not close
  that residual gap — it follows 40/87, slightly *fewer* than the deployed
  additive combine's 41/87.** The scale-mismatch hypothesis from #897's
  record, as a dominant defect, is not supported: replacing the additive
  combine with a rule that never sums across the two scales does not help,
  and mildly hurts. The residual 11-pair gap among reachable pairs remains
  unexplained by either combine-rule variant tested here; it is not
  attempted further in this diagnostic.

**A structural ceiling, not a tunable gap.** The coverage ceiling itself —
52/87 = 59.8% — falls *below* the predeclared 53/87 (60%) faithfulness bar.
Under the current `STEP_TOP_CANDIDATES = 8`, even a perfect scorer bound to
the engine's own top-8 candidate list could never clear the bar that #897
predeclared. Closing the gap requires widening the served candidate list
itself (a change to a shared architectural constant used throughout the
base engine's serving path, not a skip-mix-lane-local fix) — not a
combine-rule or quantization change confined to the skip-mix lane.

## Scope / conformance

Off-serving diagnostic; no production code changed. The lane remains
**dormant**: no `model/ids.toml` serving row, `CONFORMANCE.md` unchanged.
This record supersedes `docs/skipmix_lane_897_spotcheck.md`'s cap
attribution and narrows the #897 gap's cause to candidate-list breadth,
with a smaller (11/52, ~21%) secondary quantization/formula residual among
reachable pairs.

## Implications (for the maintainer; not actioned here)

Any future attempt to close the #897 gap fully would need to widen
`StepCandidates::STEP_TOP_CANDIDATES` (or otherwise broaden the base
engine's own candidate generation) — a larger-blast-radius change than the
skip-mix lane itself, since that constant is shared serving-path
infrastructure, not skip-mix-specific. A narrower fix targeting only the
11-pair quantization/formula residual among the 52 reachable pairs could
not, by itself, ever clear the predeclared 60% bar (its ceiling is 52/87 =
59.8%). Whether to attempt the wider candidate-list change, accept a
relaxed/different fidelity bar, or leave the lane dormant alongside the
#836 segment lane is left to the maintainer; this issue records the
measurement only.
