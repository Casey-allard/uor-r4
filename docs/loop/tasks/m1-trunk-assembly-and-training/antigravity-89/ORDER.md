# ORDER: m1-trunk-assembly-and-training, antigravity-89

Lab: antigravity. Item: [m1-trunk-assembly-and-training](../../../../roadmap/items/m1-trunk-assembly-and-training.md). Base: `ea3730908`.

## Goal

Assemble and train the full lawful discrete Stage 3 trunk (`crates/uor-r4-discrete`) combining the discrete Hamilton-gate feed-forward block (`Cl(0,6)` / 16-root E8 orbit cross-block rotation gate + scale-matched `OrbitRootSnapNormV1`), the causal quaternion-orbit prefix recurrence (`PositionHolonomyV1`), and the `Flock` and `Relation` reads under two-seed `BVG-K` (`K = 16`) training, `fresh_v2` evaluation, and `D11` (`uor-r4.lut-discrete/1`) export.

The item's done-when reads:

> Stage 3 bar: 'beat Stage 2's best by more than the seed-to-seed noise, with a fresh-panel number.'

## Cancel probe (run first)

- Probe: check at base `ea373090854f4f8ee46b87460d876563c69590ed` whether `#2844` is already satisfied, whether its four dependencies (`#2843`, `#2842`, `#2836`, `#2838`) are `done`, and whether a Stage 3 trunk assembly module, runner binary, prep sheet, or two-seed GPU record exists.
- Result (run at `ea373090854f4f8ee46b87460d876563c69590ed`, classed `mechanism` / `instrument`):
  1. All four dependencies (`#2843`, `#2842`, `#2836`, `#2838`) are `done` on `origin/main`.
  2. Neither `docs/harness/prep/m1-trunk-assembly-and-training*` nor `docs/labs/m1-trunk-assembly-and-training/` exists yet.
  3. In `crates/uor-r4-discrete`, the component mechanisms exist (`ReadSpec::Flock`/`Relation`, `NormalizationMode::ScaleMatchedSnapV1`, `PositionMode::PrefixHolonomyV1`, `BVG-K` `K = 16`, `d11_export::export_d11`), while `stage_t.rs` refuses `position.is_some()` and no module or binary assembles the discrete Hamilton-gate feed-forward block, quaternion-orbit recurrence, and `Flock`/`Relation` reads into a Stage 3 trunk trainer and evaluator.

## Plan

1. Add `crates/uor-r4-discrete/src/trunk.rs` (wired in `lib.rs`) and `crates/uor-r4-discrete/src/bin/trunk-run.rs` assembling the discrete Hamilton-gate feed-forward block (`DiscreteHamiltonGateV1`), quaternion-orbit recurrence (`PositionHolonomyV1`), `OrbitRootSnapNormV1`, `Flock`/`Relation` reads, checkpoint-admitted trunk readout, two-seed `BVG-K` (`K = 16`) training + `fresh_v2` evaluation, and `D11` (`uor-r4.lut-discrete/1`) export, with finite controls.
2. Author one-page pre-registration `docs/harness/prep/m1-trunk-assembly-and-training-2026-10-25.md` and `docs/labs/m1-trunk-assembly-and-training/README.md`, and run fresh-context `FIT`, `D68`, and `D67` reviews.
3. Run the two-seed Stage 3 trunk experiment on RTX 5090 via `scripts/pod/uor-pod`, archive artifacts to `caseyallard/uor-r4-store`, record results in `README.md`, `docs/research/notebook/m1-discrete-attention.md`, and `REPORT.md`, and close `#2844` if met.

## Files in scope

- `crates/uor-r4-discrete/**`
- `docs/harness/prep/m1-trunk-assembly-and-training*`
- `docs/labs/m1-trunk-assembly-and-training/**`
- `docs/loop/tasks/m1-trunk-assembly-and-training/**`
- `docs/research/notebook/m1-discrete-attention.md`
- `docs/roadmap/items/m1-trunk-assembly-and-training.md`

## Done when

Stage 3 bar: 'beat Stage 2's best by more than the seed-to-seed noise, with a fresh-panel number' (two seeds on `ft/dev` `14,287` tokens and sealed `fresh_v2` `13,845` tokens, with `D11` artifact and trunk checkpoint SHA pinned for `#2833`, `#2851`, `#2867`, `#2865`).

## Stop and ask if

- Any component requires softmax/squashing (`D23`), dense matmul (`D25`), or floating point in `crates/uor-r4-discrete` (`D32`).
- Pod spend approaches the global cap (`4` pods or `$8/h`).
