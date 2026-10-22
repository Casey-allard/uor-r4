# ORDER: m1-normalization-mechanism, antigravity-12

Lab: antigravity. Item: [m1-normalization-mechanism](../../../../roadmap/items/m1-normalization-mechanism.md). Base: `3e3967c76`.

## Goal

The item's done-when reads:

> A normalization mechanism with finite controls obeying CHARTER §3, plus a pre-registered test of its effect in the trunk. The size of effect that would count is not yet registered; step 4 registers it.

## Cancel probe (run first)

Ran `rg -n "normalization_mechanism" crates/uor-r4-discrete/src/` (`0` matches), verified `docs/labs/m1-normalization-mechanism/` does not exist at base `3e3967c76`, and ran `~/.cargo/bin/cargo test -p uor-r4-discrete --lib` (`203 passed; 0 failed; 1 ignored`). Issue `#2842` had 0 comments prior to our claim (`https://github.com/UOR-Foundation/uor-r4/issues/2842#issuecomment-6338517969`). The task stands.

## Files in scope

- `crates/uor-r4-discrete/**`
- `docs/geometry/e8-h4-and-the-600-cell.md`
- `docs/geometry/quaternions-and-rotation.md`
- `docs/harness/prep/m1-normalization-mechanism*`
- `docs/mechanisms/hamilton-gate.md`
- `docs/model/architecture.md`
- `docs/labs/m1-normalization-mechanism/**`

## Done when

1. Design note posted on `#2842` naming the stabilized block invariants (`L1` block norm $= 4$, squared `L2` block norm $= 8$, coordinate bound $\|x_b\|_\infty = 2$, support size $= 2$), the lawful integer construction (`OrbitRootSnapNormV1`: multiplicity-matched `snap_n` E8 orbit projection onto the 16 roots $\mathcal{O}_{16} \subset \{-2, 0, 2\}^8$ minimizing `(L1(x, n * ρ_r), Hamming(x, n * ρ_r), r)` with optional per-block `Cl(0,6)` signed-axis frame alignment), why it obeys CHARTER §3.1–§3.3 (`D23`, `D25`, `D32`), its exact 4-bit-per-block (`4 * B` bits total) information budget, and its whole-number `D11` export (`u8` root and candidate indices in `0..16`).
2. `crates/uor-r4-discrete/src/normalization_mechanism.rs` implemented and wired in `crates/uor-r4-discrete/src/lib.rs`, with 6 finite CPU controls (`norm_stability_and_bounded_range`, `idempotence_two_pass_stability`, `scale_equivariance_on_orbit_multiples`, `deterministic_tie_breaking_on_zero_and_midpoints`, `cl06_frame_equivariance_across_all_64_actions`, and `multistage_residual_accumulation_vs_unnormalized_drift`) passing under `~/.cargo/bin/cargo test -p uor-r4-discrete --lib`.
3. Draft one-page pre-registration sheet `docs/harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md` (`TEST FITNESS: PENDING`, decision card posted on `#2028` per CHARTER §5.2 for the numeric bar) comparing the normalized trunk (`N_snap` — `NormalizationMode::ScaleMatchedSnapV1`), un-scaled snap control (`C_unscaled` — `NormalizationMode::UnitSnapControlV1`), and un-normalized raw residual accumulation (`C_raw` — `NormalizationMode::UnnormalizedControlV1`), plus `docs/labs/m1-normalization-mechanism/README.md`, documentation updates in `docs/model/architecture.md`, `docs/geometry/e8-h4-and-the-600-cell.md`, `docs/geometry/quaternions-and-rotation.md`, and `docs/mechanisms/hamilton-gate.md`, and `REPORT.md` (`References #2842`).

## Stop and ask if

- Any file outside the declared scope needs modification.
- Any GPU execution is needed (requires a non-author `TEST FITNESS: FIT` review, owner bar confirmation on `#2028`, and a typed D40 owner release).
