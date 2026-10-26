# ORDER: m1-wider-code-carrier, antigravity-92

Lab: antigravity. Item: [m1-wider-code-carrier](../../../../roadmap/items/m1-wider-code-carrier.md). Base: `0cf81712c`.

## Goal

The item's done-when reads:

> A wider-carrier build passes finite CUDA qualification and a zero-training capacity measurement against the Stage 1 bar: the alias ceiling clears Bar (i) 1,941 net of admission (G-Cap), and admission at checkpoint 0 is above A8's 1,351 of 3,847 on the same panel. The synthesis also lists within-cell tie mass as a gate.

## Cancel probe (run first)

Ran on `/Users/casey.allard/uor-r4/.worktrees/antigravity-m1-wider-code-carrier` (`0cf81712c66ab8e75f7739fc47147f14e83eacba`):
1. `ls docs/harness/prep/m1-wider-code-carrier* docs/labs/m1-wider-code-carrier/`: both absent.
2. Inspected `crates/uor-r4-discrete/src/address_cells.rs`, `crates/uor-r4-discrete/src/discrete.cu`, `crates/uor-r4-discrete/build.rs`, `crates/uor-r4-discrete/src/d11_export.rs`, and `docs/labs/m1-address-key-full-vector/README.md`:
   - `Spec` (`lib.rs:84`), `discrete.cu` (`MAX_WIDTH = 128`, up to `B = 16` blocks of 8 integers), and `d11_export.rs` (`UORLUT01` / `uor-r4.lut-discrete/1`) admit `B in 2..=16`, while `FullVectorV1` in `address_cells.rs:457-461` fills blocks `3..B-1` with a context-blind XOR constant `((ic / 16) ^ (ic % 16) ^ b) & 15`, leaving multi-block interior carrier capacity (`B = 6` width `48` = `24` bits, `B = 8` width `64` = `32` bits) untested.
   - `controller.rs:176` (`validate_registered_corpus`) is part of `driver_source_sha256` (`04c5165464a18eea6d4edadc6970007dbdc7908ead420f4a2f575e22ea43061d`) and pins `blocks: 4` for the 4-block CLI paths, so wider-carrier CUDA qualification and zero-training pre-flight must live in dedicated opt-in modules (`wider_carrier_source.rs` compiled ahead of time by `nvcc` in `build.rs`, `wider_carrier.rs`, `src/bin/wider-carrier-qualify.rs`, and `src/bin/wider-carrier-run.rs`) to keep `driver_source_sha256` unchanged.
   - Classification: **mechanism** (`m1-wider-code-carrier` requires host multi-block residual predecessor placement, ahead-of-time `nvcc` wider-carrier PTX/cubin module, finite CUDA qualification, a FIT + D67 pre-registered zero-training capacity measurement on 1x RTX 5090, and lab + mechanism records).

## Files in scope

- `crates/uor-r4-discrete/**`
- `docs/harness/prep/m1-wider-code-carrier*`
- `docs/labs/m1-wider-code-carrier/**`
- `docs/mechanisms/discrete-learning.md`
- `docs/loop/tasks/m1-wider-code-carrier/antigravity-92/**`
- `docs/roadmap/items/m1-wider-code-carrier.md` (`status_only_edit` + `docs/roadmap/README.md`)

## Done when

- Step 2 design note is posted on `#2847`.
- `AddressCellsMode::WiderBlocksV1` and `AddressCellsMode::WiderBlocksShuffledV1 { seed }`, the ahead-of-time `nvcc`-compiled `wider-carrier` PTX/cubin module (`wider_carrier_source.rs` in `build.rs`), the finite CUDA qualifier (`wider-carrier-qualify`), and the zero-training evaluator/grader (`wider-carrier-run`) pass finite CPU and CUDA controls while preserving `driver_source_sha256`.
- `docs/harness/prep/m1-wider-code-carrier-2026-10-26.md` receives a fresh-context `TEST FITNESS: FIT` and `D67` pre-run review, the 1x RTX 5090 run executes and archives to `caseyallard/uor-r4-store:results/antigravity/m1-wider-code-carrier-2026-10-26/`, and the wider carrier clears the Stage 1 done-when (`G-Cap` alias ceiling net of admission `> 1,941`, `P_succ` admission at checkpoint 0 `> 1,351 / 3,847`, and `within_cell_tie_mass == 0`).
- `docs/labs/m1-wider-code-carrier/README.md`, `docs/mechanisms/discrete-learning.md`, `docs/roadmap/items/m1-wider-code-carrier.md` (`status: done`), and `REPORT.md` (`Closes #2847`) pass all gates and fresh-context D66 exact-head and claims review.

## Stop and ask if

- Finite CUDA qualification (`wider-carrier-qualify`) fails any host-vs-GPU parity or state-restoration control on the RTX 5090 pod.
- The wider-carrier (`B = 6` / `B = 8`) zero-training measurement fails to clear `1,351 / 3,847` on `P_succ` or `1,941` on `G-Cap` after ruling out instrument faults.
