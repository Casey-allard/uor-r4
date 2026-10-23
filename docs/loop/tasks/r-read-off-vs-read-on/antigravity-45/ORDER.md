# ORDER: r-read-off-vs-read-on, antigravity-45

Lab: antigravity. Item: [r-read-off-vs-read-on](../../../../roadmap/items/r-read-off-vs-read-on.md). Base: `e662c9a6c`.

## Goal

The item's done-when reads:

> Paired read-on versus read-off results on the same retained checkpoints are recorded as notebook rows, with the classification instrument versus mechanism stated. The bar for a "read adds something" verdict is not yet registered; registering it, with the score it is read on, is part of How to start. For scale only: on the 14,287 development rows a context-free successor table scores 6,300 top-1, and the trained read scored 149.

This host-only unit (`antigravity-45`) completes the pre-GPU deliverables of [`docs/roadmap/items/r-read-off-vs-read-on.md`](../../../../roadmap/items/r-read-off-vs-read-on.md) (`## How to start` steps 1–3 and the finite host controls for step 4) without touching `source_sha256()` or `driver_source_sha256()` in `crates/uor-r4-discrete/src/`:

1. **Retained-checkpoint inventory & `eval-corpus` refusal audit (step 1):** Audit retained checkpoints on disk and in the HF store (`P8`, `A8`, `S8`, `P16`, `A16` at `accepted_updates == 0` in `vast-56660582` / `vast-56670997`, vs Rung-0 Q4, K16-A/R s1/s2, and A8 Dose 1 / `vast-56670997` `Relation` Q1 / `FrozenZero` Q4 with `accepted_updates > 0`), verify `checkpoint.cursor.accepted_updates != 0 -> Error::TrainingHeld` in `crates/uor-r4-discrete/src/main.rs:108-110`, and post the inventory and PM question on `#2885`.
2. **Design note (step 2):** Write [`docs/labs/r-read-off-vs-read-on/README.md`](../../../../labs/r-read-off-vs-read-on/README.md) and post it on `#2885` specifying `RouteBypassV1` (last-position query bypass) for Route read-off and `ZeroGatesV1` (`gates == 0`) for Flock read-off, proving both produce bit-for-bit identical writer inputs on the same codebook, why neither uses softmax/matmul/floats, the 6 finite controls, and the D31 instrument-vs-mechanism classification.
3. **Finite host controls (step 2c / step 4 host):** Add [`crates/uor-r4-discrete/tests/read_off_ablation_controls.rs`](../../../../../crates/uor-r4-discrete/tests/read_off_ablation_controls.rs) verifying the 6 finite integer controls against `OUT_DIR/geometry.rs`.
4. **Pre-registration sheet (step 3):** Draft [`docs/harness/prep/r-read-off-vs-read-on-DRAFT-2026-10-23.md`](../../../../harness/prep/r-read-off-vs-read-on-DRAFT-2026-10-23.md) (`TEST FITNESS: PENDING`).

## Cancel probe (run first)

- Probe: `uor-harness loop gates --quick` (cancelled with SIGINT)
- Result: exited cleanly with `-2` (`SIGINT`) after `1.5s` with `0` orphan processes.

## Files in scope

- `crates/uor-r4-discrete/**`
- `docs/harness/prep/r-read-off-vs-read-on*`
- `docs/labs/r-read-off-vs-read-on/**`

## Done when

- `docs/labs/r-read-off-vs-read-on/README.md` is committed and posted on `#2885` with the checkpoint inventory and PM question.
- `crates/uor-r4-discrete/tests/read_off_ablation_controls.rs` passes all 6 finite integer controls via `~/.cargo/bin/cargo test -p uor-r4-discrete --test read_off_ablation_controls`.
- `docs/harness/prep/r-read-off-vs-read-on-DRAFT-2026-10-23.md` passes `uor-harness check --only harness,prep`.
- `uor-harness loop gates --full`, `uor-harness loop report --check`, and `uor-harness loop pr-check --local` pass.

## Stop and ask if

- Any GPU pod lease or `discrete.cu` device execution is needed (CHARTER §4.5 / §5.1).
- Any file outside the item's scope must change.
