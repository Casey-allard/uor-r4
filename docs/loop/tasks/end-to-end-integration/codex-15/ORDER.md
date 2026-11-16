# ORDER: end-to-end-integration, codex-15

Lab: codex. Item: [end-to-end-integration](../../../../roadmap/items/end-to-end-integration.md).
Parent: [codex-14](../codex-14/ORDER.md). Reviewed head: `1c80805c7`.

## Why this unit exists

ADR-0007 I3 says e-means attaches behind `Codebook::fit` "at the four k-means sites already in the
tree". None of the four is compiled: `addressing_arms` and `joint_admission` are `#[cfg(any())]`,
and the transformerless compiler and graph-compiler induction are inactive evidence behind
`native_support.rs` lib targets. This unit builds the fitter as live code, attaches it to the only
live codebook stage, and measures the result before anyone treats it as a win.

## Claim record

`loop claim end-to-end-integration --lab-topic e-means-codebook` was refused because the item is
claimed by PR #4032 (this lab's codex-14, in the merge queue). The claim is this lab's own; the unit
adds no dependency and does not change `Cargo.lock`.

## Method

- `crates/uor-r4-discrete/src/e_means.rs` (new, live): mini-batch k-means with EMA counts/sums, a
  cosine-annealed step, dead-cluster revival, seeded k-means++ and a last-epoch-mean finalization.
  Constants fixed at this scale: 4 epochs, 32 batches, `alpha_0 = 0.3`.
- `AddressCellsMode::EMeansV1 { seed }` (A8 address cells, ACTIVE): each token's predecessor
  home-cell histogram is the feature; 256 clusters are fitted; each cluster takes the cell of least
  weighted cell-L1 to its members; overfull cells move their lowest-frequency tokens to the nearest
  cell with room. The default `StructuredV1` path is untouched.
- `address-cells-codebook` accepts `e-means` and `e-means:SEED`.
- ADR-0007 gains a factual Amendment 2 recording that the four named sites are uncompiled and that
  this arm measures worse; it changes no rule.

## Files in scope

- `crates/uor-r4-discrete/src/e_means.rs` (new)
- `crates/uor-r4-discrete/src/lib.rs` (module registration)
- `crates/uor-r4-discrete/src/address_cells.rs` (the mode, the fit, one test)
- `crates/uor-r4-discrete/src/bin/address-cells-codebook.rs` (mode parsing)
- `docs/reference/adr/0007-stage-interfaces-for-end-to-end-integration.md` (Amendment 2)
- `docs/loop/tasks/end-to-end-integration/codex-15/**`

Out of scope: promoting the arm, changing the default placement, the serving artifact, the D86 gate,
any float-training or GPU run.

## Done when

1. The fitter reaches Lloyd-16 quality at 4 epochs (MSE no worse than 1.10x) and is deterministic.
2. `EMeansV1` produces a valid injective table at the exact quota, deterministically, and the
   default `StructuredV1` table is unchanged.
3. The measured placement cost is reported whether it wins or loses.
4. The library suite is green except the pre-existing `apply_calls_are_confined...` failure, which
   also fails on clean `origin/main`.
5. `git diff --check`, the harness contract rule, the REPORT check and the exact-head review pass.

## Cancel

Stop if the fitter needs more than a third of Lloyd's epochs to reach parity, or if the placement is
not deterministic. Do not tune the arm until it wins on this fixture; a loss is the result.

STOP and report when done.
