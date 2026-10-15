# #303 — Terminal audit: R⁴ Hopf sector space is structurally under-addressed by the block-norm projection

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-31T02:34:02Z
- **Updated:** 2026-07-31T06:17:00Z
- **Closed:** 2026-07-31T05:52:02Z
- **Assignees:** auser, Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/303
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Sequencing: last. Open after every other issue on the track is closed.** Nothing here blocks or reorders existing work, and nothing here proposes a structural change. This is a lineage-audit finding recorded so it is not lost; it is deliberately scoped as a measurement first, with remediation left undecided pending that measurement.

---

### Summary

The Hopf machinery ported from prime-router is faithful. `hopf_coordinate_components_scalar`, `hopf_phase_transport_components_scalar`, and `assign_sector_hopf_transport_scalar` (`crates/uor-r4-core/src/lib.rs:272-345`) match `prime-router/server.py:248-300` line for line, including the `0.5 * λ * cos(2χ)` connection weight and the `wrap_to_pi` phase transport. No regression there.

The problem is upstream of it. The 4-vector fed to the Hopf map is produced by `get_state_4d_projection` (`crates/uor-r4-router/src/lib.rs:1193-1220`), which reduces the 512-dim session brain state to four **block L2 norms** (dims 0-127, 128-255, 256-383, 384-511), then unit-normalizes. Two structural consequences follow, neither of which is visible from reading the Hopf code itself:

**1. Sign restriction collapses the reachable sector space to roughly 1/8.**

Norms are non-negative, so `(a,b,c,d) ≥ 0` — the coordinate is confined to the positive orthant of S³ (1/16 of the sphere). Propagating that constraint through the Hopf coordinates:

| quantity | definition | natural range | reachable range under `a,b,c,d ≥ 0` |
|---|---|---|---|
| `theta1` | `atan2(b, a)` | `[-π, π]` | `[0, π/2]` |
| `theta2` | `atan2(d, c)` | `[-π, π]` | `[0, π/2]` |
| `delta` | `theta1 - theta2` | `[-π, π]` | `[-π/2, π/2]` (half) |
| `alpha` | `0.5(theta1 + theta2)` | `[-π, π]` | `[0, π/2]` (quarter) |
| `chi_u` | `sin²χ` | `[0, 1]` | `[0, 1]` (full) |

The binning in `assign_sector_hopf_transport_scalar` assumes the full circle:

```rust
let u_delta = (delta + PI) / (2.0 * PI);
let u_alpha = (transported_alpha + PI) / (2.0 * PI);
```

With `K = 512` (the value passed at `router/src/lib.rs:1359-1364`), `allocate_triplet_bins_budget` maximizes the product under a zero-spread preference and returns `(kchi, kdelta, kalpha) = (8, 8, 8)`. But `u_delta ∈ [0.25, 0.75]` reaches only bins 2-5, and `u_alpha ∈ [0.5, 0.75]` reaches only bins 4-5 (the transport shift perturbs this slightly but does not restore the range).

**Estimated reachable sectors: 8 × 4 × 2 = 64 of 512 (~12.5%).** The remaining 7/8 of the sector space cannot be addressed by any input, independent of corpus size or session length.

**2. The projection is magnitude-only.**

Each of the four components is a norm over 128 dimensions. All directional information within a block is discarded before χ/δ/α are computed. Whatever content-discriminating structure the brain state carries inside a block cannot reach the router — the geometry is operating on four scalars that are invariant under any rotation within each block.

### Why this is worth recording separately from #276

#276 attributes the near-neighbor failure to R⁴ capacity: *"32k tokens × contexts cannot be discriminated in R⁴; neighborhoods collapse into collision zones dominated by session trajectory rather than content."* The empirical observation is not in dispute. The proposed cause may be.

If the analysis above holds, the observed collapse is consistent with an alternative cause: the sector space is under-addressed by construction and the projection is rotation-invariant within blocks. Under that reading, R⁴ is not demonstrably out of capacity — it has never been measured at full addressing. The phrase "dominated by session trajectory rather than content" is what a magnitude-only projection of an evolving session state would be expected to produce, since block norms track accumulated session energy far more directly than they track query content.

This does not argue against Design P. A discrete phase-clock addressing scheme may well be the right destination regardless, and the mul-free runtime contract (§4, enforced in `bott_fock.rs`) is an independent reason continuous transcendental transport cannot survive into the deployed kernel. The point is narrower: **if Design P is adopted while the projection feeding it remains sign-restricted and magnitude-only, the same ceiling may reappear in torus coordinates**, and the post-hoc measurement would not distinguish "Design P underperformed" from "Design P was fed a rank-deficient input." Measuring the current addressing occupancy first makes that distinction available later at negligible cost.

### Non-duplication check

- **#246** (re-port 16-window zeta design-matrix + QR) — targets `prime-router/server.py:606-633`, the window-selection and eigenvalue stubs. Disjoint code path from `get_state_4d_projection`. #246 fixes *which window* is selected; this issue concerns *which sector within a window* is addressable. Complementary, no overlap.
- **#276** (Design P phase-clock addressing) — forward-looking redesign of the addressing scheme. This issue is a backward-looking measurement of the existing one. See the section above for the interaction.
- **#247** (session-manifold signature lane) — feeds persistent geometric state into the graph engine's signature channel. Consumes the projection output; does not touch how the projection is formed.
- **#259 / #263 / #264** (uor-addr provenance, kappa-addressed regions) — addressing/audit plumbing for stored coordinates. Orthogonal to coordinate formation.
- **#266** (proof-carrying inference, region-kappa + traversal witness) — verification layer over traversals. Would in fact inherit any under-addressing described here; not a duplicate, but a downstream consumer.
- **#290** (FMM far-field operator) — hierarchical operator over the multiresolution cover. Assumes a populated coordinate space; unaffected either way.

### Scope

Measurement only. No code changes to the projection, the Hopf functions, or the binning are proposed in this issue.

1. Instrument `assign_sector_hopf_transport_scalar` (or wrap it at the `router/src/lib.rs:1359` call site) to log `sector_id`, `chi_bin`, `delta_bin`, `alpha_bin`, and the raw `chi_u` / `u_delta` / `u_alpha` values.
2. Run the existing D3 held-out fixture set (same corpus as #299/#300 so results are comparable to the measurement rows already on the track).
3. Report: distinct `sector_id` count out of 512; per-axis bin occupancy histograms; and the empirical min/max of `u_delta` and `u_alpha` against their assumed `[0, 1]` range.
4. Separately, quantify the magnitude-only loss: correlation between the four block norms and a signed alternative (e.g. four signed projections onto fixed random unit vectors per block, or the first principal direction per block) over the same fixtures. This is a diagnostic measurement, not a proposed replacement.
5. Record findings in `docs/`. If occupancy is materially below 512, open a **separate** remediation issue — do not remediate under this one.

### Definition of Done

- Sector occupancy over the D3 fixtures is measured and recorded, with the distinct-sector count stated explicitly.
- Per-axis bin histograms for χ, δ, α are recorded, including empirical ranges of `u_delta` and `u_alpha`.
- The ~12.5% estimate above is confirmed or refuted with numbers. **Refutation is a fully successful outcome** — it closes a live question about #276's stated cause and the issue closes on the evidence either way.
- The magnitude-only diagnostic from item 4 is recorded.
- If under-addressing is confirmed, a remediation issue is filed and linked; this issue closes regardless.

### Notes

- Findings from a `prime-router` → `uor-r4` lineage audit, same audit lens as #246, different code path.
- A second, unrelated observation surfaced during the same audit and is recorded here only so it is not lost — it does **not** belong to this issue's scope and warrants its own issue if anyone considers it worth pursuing. In `compile_thought_internal` (`router/src/lib.rs:1077-1092`), the 256-bit UOR content address is reduced to `hash_accumulator: i32` by plain byte summation, then used to seed four independent sin/cos oscillators. The byte sum takes values in `[0, 8160]` and is CLT-concentrated near 4080, so the 256-bit address collapses to on the order of 10-13 usable bits; because all four coordinates derive from that one scalar, `r4_target` traces a one-dimensional Lissajous curve in R⁴. This path feeds the visualization target and `path_steps`, **not** the Hopf sector routing, so it does not bear on the analysis above.
- Sequencing rationale: everything here is diagnostic and none of it gates other work. Running it earlier would consume review attention without unblocking anything. Running it last, before any Design P post-mortem, is when the numbers are most useful.

## Comments (2)

### comment 5139705676 — 2026-07-31T05:43:22Z — auser

Measurement complete — PR #305 (closes this issue per DoD; remediation filed separately as #306).

**Headline: 7 of 512 sectors occupied (1.4%)** over 1500 routing samples / 596 held-out D3 articles (canonical split, corpus CID blake3:194db0ee…0132cf, fixed γ = 0.85, production evolve-then-route sequence).

DoD checklist:
- Distinct-sector count: **7/512**, recorded in docs/hopf_sector_occupancy_d3.md ✓
- Per-axis histograms: χ {3: 48, 4: 1452}, δ {3: 1487, 4: 13}, α {4: 1474, 5: 26} (of 8 bins each); empirical ranges u_delta ∈ [0.478, 0.503], u_alpha ∈ [0.617, 0.626] vs assumed [0, 1] ✓
- ~12.5% estimate: **confirmed in direction, refuted in magnitude** — static bound 64/512 holds, empirical concentration (near-constant block norms, 1.3–2.5% relative dispersion) lands far inside it ✓
- Magnitude-only diagnostic: per-block Pearson r between block norms and a fixed signed projection −0.658 / −0.356 / −0.570 / −0.721 ✓
- Under-addressing confirmed → remediation filed as #306, linked ✓

One code-reading note recorded in the doc: under the default Spectral geometry, `SpectralGeometry::ground` copies the session brain state into the grounded vector for any non-empty input (and the ground-failure fallback does the same), so the issue's 'session brain state' framing holds for the default path; under Vsa geometry the Hopf input would be the grounded query vector instead.

Harness (reproducible, ignored by default): `cargo test -p uor-r4-router --release --offline --test hopf_sector_occupancy -- --ignored --nocapture`

### comment 5139933270 — 2026-07-31T06:17:00Z — auser

Merged in 53a4bf89b254b34fcff212a79a9f3d4c167acb63 (PR #305). DoD evidence in the comment above and docs/hopf_sector_occupancy_d3.md; remediation tracked in #306.


---
_Comment/review ids in this file: 5139705676, 5139933270_

