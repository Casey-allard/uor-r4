# #467 — Measurement runtime: sampled fast paths for the instrument and Gate C decision runs

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-06T17:07:57Z
- **Updated:** 2026-08-07T05:08:48Z
- **Closed:** 2026-08-07T05:08:48Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/467
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Problem:** our long runs buy precision we never use, and that is what makes guess-and-check expensive. At 402,802 held-out positions with top-1 around 26%, the standard error on the headline is **0.07pp** — while every exit rule we have ever pre-declared is written at **±2pp**. We are measuring roughly 30x finer than any decision requires, and paying hours for it.

**Arithmetic (the case):**

| positions | standard error | 95% interval | vs a 2pp rule |
|---|---|---|---|
| 402,802 (today) | 0.07pp | ±0.14pp | 14x finer than needed |
| 40,000 | 0.22pp | ±0.43pp | 4.6x finer |
| 10,000 | 0.44pp | ±0.86pp | 2.3x finer |

A 10k-position decision run is ~40x cheaper and still resolves every threshold we set. The full pass stays justified for a **record row** we intend to pin and cite — not for deciding what to do next.

**Scope:**

1. **Sampled Gate C decision mode** — deterministic stratified subsample of held-out positions (env-overridable N, default full so nothing changes silently), reporting the sample size and the standard error alongside every rate so a reader can see the precision. Full-pass behaviour must remain bit-identical when the knob is unset.

2. **Instrument fast path** (`capacity_scaling`, ~12 min today) — three stacking fixes: sample ~100k records for the population statistics (occupancy and records-per-key are estimates, not a census), parallelize the code-assignment pass with rayon the way `evaluate_gate_c` already parallelizes its rows, and reuse codes the compile already computed rather than recomputing them. Target: **under a minute**, ideally ~10s. A gate nobody wants to wait for is a gate nobody runs — this is what makes the #460 discipline rules enforceable in practice.

3. **Report the precision, always** — any sampled row prints n and the standard error, so a sampled number can never be quoted as if it were a census.

**Out of scope here (separate issue if pursued):** the `assign_for_bundle` brute-force nearest-centroid scan (4 stages x 256 centroids x 512 dims per record, ~1 trillion multiply-adds across 2.11M records). A coarse-to-fine index or triangle-inequality pruning is plausibly another 5-20x and would speed the **production compile**, not just instruments — but it touches the κ-pinned path and needs its own κ-neutrality proof.

**Definition of done:** instrument under a minute on the 2.11M corpus with verdicts matching the full-pass run; sampled Gate C reproduces the full-pass headline within its stated interval on the same artifact; defaults unchanged; the AGENTS.md discipline rules (PR #466) reference the new fast paths.


## Comments (1)

### comment 5212669398 — 2026-08-07T05:08:47Z — Casey-allard

Closing: scope shipped and verified in PR #468.

`R4_GATE_C_SAMPLE` exists (`score.rs` `GATE_C_SAMPLE_ENV`, applied via `gate_c_sample(held_out)`, CLI banner in graph-cli), the instrument fast path landed in `tests/capacity_scaling.rs` (rayon + `assign_code_for_bundle`, no Store build), full-pass behaviour is bit-identical with the knob unset (verified by a field-by-field diff of two `score_report.json` documents built from origin/main and the branch), and Gate C dropped 597s to 60s. Report schema 24 to 25.

Both residual costs found while building it were spun out rather than left implicit: the vectorized-kernel finding became #469, and the full-corpus table construction became #471. Nothing identifiable remains here.


---
_Comment/review ids in this file: 5212669398_

