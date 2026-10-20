# #305 — feat(router): #303 Hopf sector occupancy measurement — D3 held-out, 7/512 sectors (1.4%)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-07-31T05:41:45Z
- **Updated:** 2026-07-31T05:52:01Z
- **Closed:** 2026-07-31T05:52:01Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/305
- **Merged:** 2026-07-31T05:52:01Z
- **Merge commit:** 53a4bf89b254b34fcff212a79a9f3d4c167acb63
- **Base / Head:** main / UOR-Foundation:issue-303-hopf-sector-occupancy
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #303.

## What

Measurement only, per the issue's scope — no remediation. Instruments the Hopf sector call site and measures sector occupancy over the D3 held-out fixtures.

- `HopfResult` gains `chi_bin`/`delta_bin`/`alpha_bin` (previously computed by `assign_sector_hopf_transport_scalar` and discarded at the call site); new `route_query_to_manifold_native_with_hopf_input` exposes the 512-d projection input for diagnostics. Existing methods delegate and discard — **no routing behavior change**.
- `crates/uor-r4-router/tests/hopf_sector_occupancy.rs`: ignored-by-default harness driving the production evolve-then-route sequence (same order as `src/server.rs` POST /api/chat) over the 596 held-out D3 articles, fixed γ = 0.85, 1500 routing samples. Skips vacuously without the corpus (kappa_reproduction convention).
- `docs/hopf_sector_occupancy_d3.md`: findings, claim-language per `docs/formal_vocabulary.md` (Empirical Criterion / status Empirical, declared protocol, exact counts on the pinned corpus CID).

## DoD evidence

- **Distinct sectors: 7 of 512 (1.4%)** over 1500 samples / 596 held-out articles (canonical `blake3(id)[0] % 5 == 0` split, corpus CID `blake3:194db0ee…0132cf`).
- Per-axis histograms (of 8 bins): χ {3: 48, 4: 1452}; δ {3: 1487, 4: 13}; α {4: 1474, 5: 26}. Empirical ranges: `chi_u ∈ [0.480, 0.542]`, `u_delta ∈ [0.478, 0.503]`, `u_alpha ∈ [0.617, 0.626]` against a binning that assumes `[0, 1]`.
- **~12.5% estimate: confirmed in direction, refuted in magnitude.** The static reachable bound (64/512, sign restriction) holds; empirical occupancy is 7/512 because EMA-blended session states concentrate (block-norm relative dispersion 1.3–2.5% → θ₁ ≈ θ₂ ≈ π/4).
- Magnitude-only diagnostic: per-block Pearson r between block norms and a fixed deterministic signed projection: −0.658 / −0.356 / −0.570 / −0.721, with norms near-constant (σ 0.006–0.012 on means ≈ 0.48–0.51).
- Code-reading correction recorded in the doc: under the default Spectral geometry the Hopf input *is* the session brain state (ground copies it; the fallback does the same); under Vsa it would be the grounded query vector.
- Remediation issue filed separately and linked from #303 (per DoD, none lands here).

## Gates

- `cargo fmt --check` ✓ · `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings` ✓ · no_std ladder (both configs) ✓ · `python3 scripts/check_claim_wording.py` ✓
- `cargo test --workspace --offline`: all suites green **except** `uor-r4-core` `allocation_census`, which fails locally against the stale on-disk `.uor-models` store (predates the u32 migration — AGENTS.md 'things that bite'; a recompile is a maintainer decision). Independence from this PR: `uor-r4-core` has zero dependency on `uor-r4-router` (the only crate touched), and CI on `main` at the branch point (783c6f9) is green. Full workspace suite re-run with `--skip allocation_census`: zero failures.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

