# Skip-mix lane deployed bounded spot-check (#897)

Empirical record. S1 redesign lowering (#822); phase-0 confirmation verdict
`SELECT-1-token` (`docs/skipmix_confirm_897_result.json`). This is a **bounded**
minimal-pairs spot-check of the deployed 1-token skip-mix lane on the real
#833 canonical bundle — not the full end-to-end causal run (#833-shaped).
Records in `docs/` are appended to, not rewritten.

Harness: `crates/uor-r4-api/tests/skipmix_lane_spotcheck_897.rs`
(`--ignored`; reference-only, off the serving default). Machine result:
`docs/skipmix_lane_897_result.json`. Run contract: #897 issue comments.

## Question

The #897 phase-0 confirmation measured the `skipmix` arm (1-token conditioning:
joint tables keyed by `(content_token, last_window_token)`, Ψ-bag fallback,
residual against the 2-token suffix rate) at +56.2‰ over the suffix baseline
(paired 95% CI [53.6, 58.7]) and beating the D1-selected 2-token `mix` arm
(paired lower bound +23.1‰) — clearing the 25‰ opening bar. That measurement is
over the reference arm's Rust closures (`skipmix_confirm_897.rs::Tables`), not
the deployed `R4Engine` path.

Tasks 2-5 of the implementation lowered this onto the deployed engine: a real
fixed-capacity open-addressed hash table (SKMX) plus a Ψ-bag fallback (PSIB),
a compiler fit (`skipmix_fit::fit_skipmix_tables`), a runtime contribution +
argmax integration (`predict_decision_candidates_with_skipmix`), and — per
Casey's decision point B — an extension of the **real release emitter**
(`ScoredGraphSections` / `emit_scored_r4g1`), not a side converter. This check
asks the cheap question before any end-to-end causal run: on the real bundle,
does the deployed lowering (fit → emit via the real emitter → consume) realize
the reference arm's signal on its own best cases — the exact minimal pairs the
`skipmix` arm resolved — within its known ceiling (integer `ScoreQ`
quantization, the decided candidate list, the bounded per-key cap)?

## Method

1. **Reproduce the phase-0 `skipmix` favorable pairs** deterministically:
   rebuild the reference tables (`suffix_next`/`content_next`/`joint_next`/
   `d4skip_next`) from the corpus TRAIN split exactly as
   `skipmix_confirm_897.rs::Tables` does, mine the minimal pairs (same 2-token
   suffix, different story, different teacher argmax, one pair per suffix
   key), and keep exactly the pairs the `skipmix` arm followed. This
   reproduced `skipmix_follow = 87 / 4722`
   (`docs/skipmix_confirm_897_result.json`) exactly — a reproduction control
   the harness asserts, alongside the phase-0 `base_follow = 0` control
   degeneracy.
2. **Fit + emit + consume the deployed tables, via the REAL release emitter**:
   `skipmix_fit::fit_skipmix_tables` (top-64/key, quantized to integer
   `ScoreQ`) over the same corpus → 811,421 learned joint keys + 19,710 Ψ-bag
   keys → `score::emit_scored_r4g1` (the exact function the production
   `score` CLI command calls) over the real teacher artifacts, the real
   recovered cover (regions/structural, from the cached `graph-cover/
   cover.r4g1`), the real store (`tless_store.bin`), and the real compiled
   transitions/context-rows/forward-anchor-rows/emissions → `R4Engine`, which
   consumed both sections (asserted via `skipmix_tables_present()`).
3. **Replay the deployed lane** on each favorable position: a freshly reset
   engine (no persistent session object needed — the skip-mix evidence is
   entirely the current window), and the served token taken from
   `predict_decision_candidates_with_skipmix`. A pair "follows" when the lane
   serves each side's own teacher target and the two differ.

## Emission-path note (contrast with the #886 precedent)

The #836 segment lane's spot-check had to caveat its result: that lane's only
emitter (`convert_r4g1::convert_with_segment_table`) is not the emitter that
produced the released `graph/score.r4g1`, so re-emitting could not reproduce
the released graph's held-out serving behavior — 6/20 favorable positions came
back UNSERVABLE-AT-EMISSION. This lowering instead extends the REAL release
emitter, so the re-emitted graph here **is** built the same way the release
graph is. That shows up in the result: **0 of 174** replayed positions were
unservable (100% served) — the emission-path confound #886 had to work around
does not apply here. The 41/87 follow rate below is therefore a clean read on
the lowering's own fidelity, not an artifact of a mismatched emitter.

## Result

| quantity | value |
|---|---|
| reference favorable pairs (reproduced) | 87 / 87 expected |
| reference baseline-follow (control) | 0 |
| fitted rows | SKMX 811,421 joint keys / PSIB 19,710 Ψ-bag keys |
| positions evaluated | 174 |
| served / abstained / unservable | 174 / 0 / 0 |
| **deployed-lane follow** | **41 / 87** |
| predeclared threshold for "faithful" | ≥ 53 / 87 (60%, the #886 bar) |

Every replayed position served (no emission-path or engine-load confound).
41 of 87 favorable pairs — 47.1% — reproduced the reference arm's
minimal-pair distinction; the remaining 46 pairs served but did not follow
(the deployed re-rank served a token other than the reference-arm-tracked
teacher target on at least one side of the pair).

## Read (predeclared, before the run)

**LOWERING-FIDELITY GAP.** The deployed lane followed 41/87 (47.1%) < the
predeclared 53/87 (60%) bar, so the deployed lowering does not yet track the
phase-0 `skipmix` arm's signal on its own best cases, even though the lowering
is correctly implemented and fully servable end-to-end via the real release
emitter. This is the predeclared `< 53/87` branch and an evidence-backed
negative — a legitimate outcome, recorded as such, not a defect in tasks 2-5's
implementation (SKMX/PSIB format, the fit, the runtime integration, and the
real-emitter extension all behave exactly as designed and are fully tested).

The gap is plausibly attributable to the integer `ScoreQ` quantization plus
the per-key top-64 cap combined with the decided-candidate-list bound the
reference arm's floating-point residual scoring does not carry — the same
class of ceiling effect #886 recorded for the #836 segment lane, but measured
here without the emission-path confound.

## Scope / conformance

Reference-only / off the serving default; measurement, not deployment. The
lane ships **dormant**: no `model/ids.toml` serving row, `CONFORMANCE.md`
unchanged, no end-to-end causal run launched (the reachability-gate probe and
the #833-shaped run were conditional on this spot-check clearing; it did not).
RF-21/22/27/28-shaped discipline. No hours-long run launched.

## Implications (for the maintainer; not actioned here)

The lowering (SKMX/PSIB wire format, the compiler fit, the runtime
contribution+argmax integration, and the real-emitter extension) is solid,
tested, and mergeable independent of promotion — it is the infrastructure a
future fidelity fix (a wider per-key cap, a different quantization scale, or a
revised candidate list) would build on without further wire-format or
emission-path work. Whether to invest in closing the 47%→60% gap, or to treat
this arm as dormant infrastructure alongside the #836 segment lane, is left to
the maintainer; this issue records the measurement only.
