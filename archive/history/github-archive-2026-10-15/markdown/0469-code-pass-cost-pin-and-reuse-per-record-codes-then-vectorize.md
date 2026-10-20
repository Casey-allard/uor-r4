# #469 — Code-pass cost: pin and reuse per-record codes, then vectorize assign_for_bundle

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-06T18:47:58Z
- **Updated:** 2026-08-07T08:24:08Z
- **Closed:** 2026-08-07T08:24:08Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/469
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Two related levers on the same bottleneck, filed together because the second may make the first unnecessary for most runs.

**The bottleneck, measured (#467):** the per-record code pass is 468s of a 613s instrument run on the 2.11M corpus and ~585s of the store build — 4 stages x 256 classes x 288 dims of branchy shift-add per record, roughly 5e11 term applications, already saturating the available cores. Sampling could not remove it (the construction-split statistics cannot be subsampled without biasing distinct-key counts), so the remaining moves are: compute it faster, or stop recomputing it.

**Lever A — pin and reuse per-record codes (probably the bigger win).** The compile currently persists only *token* codes (`hierarchical_codes.json`); per-record graded codes are recomputed from scratch by every consumer — the store build, the instrument, each harness run. They are a deterministic function of (artifact kappa, corpus kappa, record index), so they can be emitted once at compile time and read back, keyed by both kappas so a stale cache is impossible to load silently. Scope: emit a codes sidecar next to the compiled artifact (record index to `[u8; STAGES]`, so ~4 bytes/record, about 8.5 MB for 2.11M records); readers verify both kappas and fall back to computing when absent or mismatched; a regression test asserts the read-back codes are identical to the computed ones. Expected effect: the second and every subsequent consumer of a compiled artifact drops the code pass entirely — instrument, harnesses, and repeat score runs all become near-free on that axis.

**Lever B — vectorize `assign_for_bundle`.** The kernel is a brute-force nearest-centroid scan: 256 candidates per stage, 288-dim distance each, times 4 stages, per record. Options in increasing order of risk: SIMD/wide-lane arithmetic over the existing integer form; triangle-inequality pruning (precompute inter-centroid distances, skip candidates that cannot win); a coarse-to-fine index over the codebook. Plausibly 5-20x on top of anything else, and unlike lever A it speeds the **production compile**, not just instruments and harnesses.

**Hard constraint on lever B:** this path is kappa-pinned. Any change must be proven bit-identical on the canonical path — `TLESS_CANONICAL_DETERMINISTIC=1`, `kappa_reproduction` passing against the adopted fixtures — before it can ship. An assignment that is faster and *almost* identical is worthless here; it silently invalidates every pinned measurement. Recommended order: implement, prove bit-identity on the fixture corpus, then re-measure compile wall-clock.

**Definition of done:** (A) codes sidecar emitted and consumed, kappa-keyed, with a regression test proving read-back equals recompute, and a measured before/after on a repeat instrument run; (B) assignment kernel measurably faster with `kappa_reproduction` bit-identical and the canonical-mode container kappa unchanged.

**Suggested sequencing:** A first — it is kappa-neutral by construction (a cache of an existing computation), lands quickly, and immediately makes the #460 discipline gates cheap to run. B second, on its own branch, with the kappa proof as the gate.


## Comments (1)

### comment 5212670402 — 2026-08-07T05:08:55Z — Casey-allard

**Status: lever A shipped, lever B open — and it is smaller than this issue implies.**

Lever A is done (PR #470): `code_sidecar.rs`, `R4CS` v1, keyed on artifact kappa + corpus kappa, blake3-verified, fallback-safe, wired into `evaluate_gate_c`, `capacity_scaling.rs`, and the graph-cli score path. Measured 624.9s to 38.9s on the instrument (16.1x), 605s to 20s on the code pass.

**Lever B, precisely.** `assign_for_bundle` / `assign_code_for_bundle` (`runtime.rs`) are still scalar: `dot_stage_top` / `dot_score_plain` with a plain `for class in table.chunks_exact(D)` argmax, no pruning, no early exit. But a vectorized kernel **already exists in the tree** — `simd::DotTables::from_packed` / `simd::dot_argmax` (`simd.rs`) — and is currently used only by the kernel-counting scorer (`code_from_bundle_dot`), not by the assign path. So the remaining work is routing the plain assign path through the existing kernel (or adding per-stage pruning), not writing a new one.

**Hard constraint, unchanged:** this path is kappa-pinned. Any change must be proven bit-identical via `tests/kappa_reproduction.rs` under `TLESS_CANONICAL_DETERMINISTIC=1` before it can ship — an assignment that is faster and *almost* identical silently invalidates every pinned measurement.

Unassigning so it is claimable; the work above is self-contained.


---
_Comment/review ids in this file: 5212670402_

