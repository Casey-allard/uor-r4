# Hopf Sector Occupancy on the D3 Held-Out Fixtures (Issue #303)

- **Date:** 2026-07-31
- **Issue:** [#303](https://github.com/UOR-Foundation/uor-r4/issues/303)
- **Harness:** `crates/uor-r4-router/tests/hopf_sector_occupancy.rs` (ignored by
  default; see Reproduction)
- **Corpus:** `simple-wiki-20231101-sample`, `corpus_cid
  blake3:194db0eebf2d49823ece01ee935447a0cc9edeaf018454ceea480ce7590132cf`
  (manifest-verified; 3000 articles, 596 held out)
- **Claim language:** per `docs/formal_vocabulary.md`. Every measured statement
  below is an **Empirical Criterion** with status **Empirical** — a pinned-corpus
  measurement with a declared protocol, not a proof.

## 1. What feeds the Hopf map (verified against the code)

The sector assignment chain is:

1. `route_query_to_manifold_internal_generic`
   (`crates/uor-r4-router/src/lib.rs:1322`) obtains `active_state_refined`, the
   first 512 dims of the grounded VSA vector. Under the default `Spectral`
   geometry, `SpectralGeometry::ground`
   (`crates/uor-r4-router/src/geometry.rs:65`) copies the **session brain
   state** into those dims for any non-empty input, and the ground-failure
   fallback does the same — so in practice the Hopf input is the evolved
   512-dim session state. (Under `Vsa` geometry it would be the grounded
   query vector instead; that path was not measured here.)
2. `get_state_4d_projection` (`crates/uor-r4-router/src/lib.rs:1206`) reduces
   the 512 dims to four block L2 norms (128 dims each), unit-normalized —
   non-negative by construction.
3. `assign_sector_hopf_transport_scalar`
   (`crates/uor-r4-core/src/lib.rs:325`) computes Hopf coordinates, applies
   phase transport, and bins `(χ_u, u_delta, u_alpha)` into
   `(kchi, kdelta, kalpha) = (8, 8, 8)` — the result of
   `allocate_triplet_bins_budget(512, hint, 2, 2)` for every per-identity
   `hopf_chi_bins` hint in `{2, 3, 4}` (8³ = 512 is the unique zero-spread
   maximum).

**Static reachable-range bound.** With `a,b,c,d ≥ 0`: `θ₁,θ₂ ∈ [0, π/2]`,
so `u_delta ∈ [0.25, 0.75]` (δ-bins 2–5 of 8) and, absent the transport
shift, `u_alpha ∈ [0.5, 0.75]` (α-bins 4–5 of 8). The transport shift
(λ ∈ [0.70, 1.30] per identity) perturbs `u_alpha` modestly. At most
8 × 4 × 2 = 64 of 512 sectors (~12.5%) are reachable by any input; the
remaining 7/8 are unreachable by construction. This is the bound stated in
the issue; the code reading confirms it.

## 2. Protocol (**Empirical Criterion** declarations)

- **Split:** canonical D3 rule — held-out ⇔ `blake3(id as utf-8)[0] % 5 == 0`
  (596 of 3000 articles).
- **Session protocol:** one session identity per article (`identity =
  article id`); article text fed in 2000-char chunks; per chunk,
  `evolve_state(identity, chunk, γ)` then route — the same sequence as
  `src/server.rs` POST /api/chat, with fixed `γ = 0.85` (the server
  autotunes γ per request; a fixed value keeps runs comparable).
- **Samples:** 1500 routing decisions, each recording `sector_id`,
  `chi_bin`, `delta_bin`, `alpha_bin` (first-class fields added to
  `HopfResult` under this issue), plus `chi_u`, `u_delta`, `u_alpha`,
  `phase_transport_lambda`, and the raw 512-dim Hopf input vector.
- **Uncertainty:** none declared; the run is a full enumeration of the
  pinned held-out partition, not a sample of it. Counts are exact for this
  corpus and protocol. Generalization beyond the D3 distribution is not
  claimed.

## 3. Results (status: **Empirical**, per §2 protocol)

**Occupancy.** Distinct `sector_id` values observed: **7 of 512 (1.4%)**.

Per-axis bin histograms (of 8 bins per axis):

| axis | occupied bins (count) | empirical range | binning assumes |
|---|---|---|---|
| `chi_bin` | 3 (48), 4 (1452) | `chi_u ∈ [0.4799, 0.5419]` | `[0, 1]` |
| `delta_bin` | 3 (1487), 4 (13) | `u_delta ∈ [0.4783, 0.5029]` | `[0, 1]` |
| `alpha_bin` | 4 (1474), 5 (26) | `u_alpha ∈ [0.6169, 0.6260]` | `[0, 1]` |

`phase_transport_lambda` spanned its full per-identity range [0.70, 1.30].

**On the issue's ~12.5% estimate:** confirmed in direction, refuted in
magnitude. The static reachable bound is 64 sectors (12.5%); the empirical
occupancy is 7 (1.4%). The gap is concentration, not reach: the evolved
session states have near-equal block norms (block-norm means 0.478–0.510
with standard deviations 0.006–0.012, i.e. 1.3–2.5% relative dispersion),
so `θ₁ ≈ θ₂ ≈ π/4`, giving `δ ≈ 0` (`u_delta ≈ 0.5`), `α ≈ π/4`
(`u_alpha ≈ 0.625`), and `χ_u ≈ 0.5`. The reachable region is small by
construction; the visited region is a narrow band inside it because EMA
blending equalizes block energies.

**Magnitude-only diagnostic (scope item 4).** Per 128-dim block, Pearson
correlation across the 1500 samples between the block L2 norm (what the
projection keeps) and the signed projection of the same block onto a fixed
deterministic unit vector, blake3-derived (what it discards):

| block | r(norm, signed) | norm σ | signed σ |
|---|---|---|---|
| 0 | −0.658 | 0.0080 | 0.0122 |
| 1 | −0.356 | 0.0120 | 0.0093 |
| 2 | −0.570 | 0.0063 | 0.0344 |
| 3 | −0.721 | 0.0073 | 0.0311 |

Read conservatively: the norms are near-constant (relative dispersion
1.3–2.5%), so they carry little discriminating signal of any kind; and the
variation they do carry overlaps only partially with any single signed
direction (|r| between 0.36 and 0.72, none near 1). Both observations are
consistent with the issue's hypothesis that the projection discards
content-discriminating structure before the Hopf map sees it. This is a
diagnostic, not a proposed replacement.

## 4. Consequences for #276 (Design P)

The measurement supports the issue's motivation for measuring before
remediation: under the current projection, the sector space is
under-addressed by construction (≤64/512 reachable) and by concentration
(7/512 observed). If a redesigned addressing scheme (Design P,
`docs/phase_clock_address_design.md`) is fed the same near-constant four
block norms, the same ceiling applies in the new coordinates, and a
post-hoc measurement could not distinguish "new scheme underperformed"
from "input was rank-deficient". A remediation issue for the projection
itself is filed separately (linked from #303); per the issue's DoD, no
remediation lands under #303.

This measurement neither supports nor refutes #276's capacity claim
("32k tokens × contexts cannot be discriminated in R⁴"): the router has
never been measured at full addressing, and this run does not measure it
either — it measures the addressing deficit directly.

## 5. Reproduction

```bash
# corpus (if absent): python3 scripts/fetch_d3_corpus.py
cargo test -p uor-r4-router --release --offline \
  --test hopf_sector_occupancy -- --ignored --nocapture
```

Writes `target/hopf_sector_occupancy/report.json` (override with
`HOPF_OCCUPANCY_REPORT_PATH`). Skips vacuously when the corpus is absent.
The numbers above are exact for the pinned corpus CID and protocol in §2;
any change to session evolution, geometry, or the projection invalidates
direct comparison (era note, same convention as κ re-pins).
