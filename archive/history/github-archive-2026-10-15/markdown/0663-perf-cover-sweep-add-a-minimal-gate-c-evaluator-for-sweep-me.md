# #663 — perf(cover-sweep): add a minimal Gate C evaluator for sweep metrics (#611)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-13T13:27:22Z
- **Updated:** 2026-10-09T14:17:05Z
- **Closed:** 2026-08-13T13:47:07Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/663
- **Merged:** 2026-08-13T13:47:07Z
- **Merge commit:** 67b78ef04fe1c757b713de131588f6731da4be4f
- **Base / Head:** main / UOR-Foundation:i611-slim-gatec
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary
The cover sweep read only three of `evaluate_gate_c`'s ~forty metric arms (Rule 1 reconstruction, Rule 1+2 agreement, TLA3 baseline) plus the analytic unigram nulls for the #456 null-arm harness. The full evaluator also builds and scores the #66 ablations, #80 variants, the #399 forward-anchor family, the #446 right-context family, per-status breakdowns, alpha sweeps, EXCT probes, and win/loss — plus the whole-corpus forward-anchor / right-context / two-sided / latent table passes those arms need. All of that ran nine times per sweep and was discarded.

## What's included
- **`evaluate_gate_c_sweep` (+ `SweepGateC`)** in graph-certify — builds **two** scorers instead of seven and runs one lean per-position pass, producing exactly the metrics the sweep and null-arm harness consume: `rule1_chain`, `rule12_precedence`, `tla3_baseline`, and `GateCNulls`, with the #467 sampled/census provenance (`held_out_population`, `positions_sampled`) and the `SAMPLED DECISION RUN` announcement preserved.
- **Parity by construction:** it calls the SAME primitives the full evaluator's per-position row uses — `AssignTables` + `code_plain_with` for the code (identical to the full path's `left_codes`), `score_candidates_coded`, `outcome_bits`, `predict_witness_plain`, `witten_bell_probability`, the #390 unigram-null derivation, and `gate_c_sample` — so its numbers equal the full evaluator's within the deterministic FP contract.
- **Full `evaluate_gate_c` untouched** — remains the certification/research evaluator.
- **Both consumers wired:** `run_point` (the hot nine-point loop) and `reconstruction_null` (reads `rule1_chain` + `nulls`). Field names match, so the row mapping is unchanged.

## Acceptance criteria
- Reports every field consumed by `SweepRow`, `SweepReport`, and the null-arm harness ✔ (`rule1_chain`, `rule12_precedence`, `tla3_baseline`, `nulls`).
- Required metrics match the full evaluator within the deterministic FP contract ✔ (measured, below).
- Full Gate C path remains available and unchanged for certification ✔.
- Benchmarks show a material reduction in per-point wall time and allocations ✔ (measured, below).
- Sampled decision runs vs full census remain explicit ✔ (`positions_sampled`/`held_out_population` + announcement).

## Verification (measured, real fixtures)
- **Parity** — on the default cover point (300 held-out), the slim evaluator equals the full evaluator **exactly**: rule1 `0.016667 top1 / 17.0431 bpt`, rule12 `0.350000 / 8.7587`, baseline `0.390000 / 8.4363`, and every `GateCNulls` field. New `#[ignore]` fixture-gated test asserts the row metrics (slim-derived) == full, and slim == full directly incl. nulls.
- **Benchmark** — full Gate C **11,565 ms** vs slim **245 ms** on the same artifact → **47.2× faster** (two scorers not seven; no whole-corpus arm passes; fewer allocations). Gate C was the dominant per-point stage in the #609 timing, so this is the sweep's largest per-point cost.
- **Local CI parity, all green:** `fmt --check`, `clippy --workspace --all-targets --all-features -D warnings`, `wasm32 --lib`, `cargo test --workspace`, `check-model` (R1), `audit-deferral` (R4 — nothing deferred), `repo-conformance` (R2/R3), `cargo deny` (R6).

## Breaking changes
None. `evaluate_gate_c` and its `GateCOutcome` are unchanged; `evaluate_gate_c_sweep`/`SweepGateC` are additive. Sweep report metrics and artifact bytes are identical.

Closes #611

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

