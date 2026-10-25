# ORDER: r-lawful-pointer-forced-copy-oracle, antigravity-69

Lab: antigravity. Item: [r-lawful-pointer-forced-copy-oracle](../../../../roadmap/items/r-lawful-pointer-forced-copy-oracle.md). Base: `75c04dab3`.

## Goal

The item's done-when reads:

> A lawful pointer with finite controls, then the forced-copy oracle run once under a registered sheet that has a non-author TEST FITNESS: FIT review. The result is one row in a research notebook. The training step that follows (a slot-address key, at least 3 seeds, at least 120 memory rows, a gain of at least 5 rows of 40 over the control) is a separate item, not part of this one.

Deliver the ahead-of-time `nvcc` CUDA kernel (`lawful_pointer.cu` -> `lawful_pointer.ptx`), D29 PTX interlock, batch `N`/`O1`/`O2`/`W` forced-copy oracle evaluator with 16-block paired cluster jackknife $\text{SE}_{\Delta}$ and 4-branch diagnostic partition (`Branches A/B/C/D`), and GPU binary (`lawful-pointer-oracle`); run finite CUDA qualification, `m2-fresh256` 256-row span-coverage/control verification, and `tokens/s` measurement across two deterministic builds on an RTX 5090 pod; and run a fresh-context `D68` decision review to register the diagnostic bar and pin `lawful_pointer.ptx` in the sheet so only the bound M2 trunk checkpoint (`m1-trunk-assembly-and-training`, `#2844`) remains.

## Cancel probe (run first)

Ran `CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo test --no-fail-fast -p uor-r4-training --lib lawful_pointer` at `75c04dab3` (`9 passed; 0 failed`) and inspected `lawful_pointer.rs`, `build.rs`, `tests/d29_compiled_modules.rs`, and `docs/harness/prep/r-lawful-pointer-forced-copy-oracle-DRAFT-2026-10-23.md`. Class: **`instrument`** (task stands):
1. `lawful_pointer.rs` is CPU-only (`select_step`), with no `nvcc` CUDA kernel (`lawful_pointer.cu` -> `lawful_pointer.ptx`), no D29 PTX scan entry, no batch `N`/`O1`/`O2`/`W` evaluator (`evaluate_forced_copy_oracle_batch`, 16-block paired cluster $\text{SE}_{\Delta}$, 4-branch partition, and `N`/`W`/span-coverage stops), and no GPU binary (`lawful-pointer-oracle`).
2. `docs/harness/prep/r-lawful-pointer-forced-copy-oracle-DRAFT-2026-10-23.md` has `## Bar` marked `PROPOSED, NOT REGISTERED`, stale pre-D67/D68 wait clauses, and no pinned `lawful_pointer.ptx` SHA-256 or `m2-fresh256` 256-row span-coverage check.
3. Bound checkpoint `m1-trunk-assembly-and-training` (`#2844`) is not yet trained (`PLAN.md`).

## Files in scope

- `crates/uor-r4-training/**`
- `docs/harness/prep/r-lawful-pointer-forced-copy-oracle*`
- `docs/labs/r-lawful-pointer-forced-copy-oracle/**`

## Done when

1. `crates/uor-r4-training/src/lawful_pointer.cu` (`lawful_pointer.ptx` under `sieve-cuda`, verified by `tests/d29_compiled_modules.rs`), `lawful_pointer::device`, `evaluate_forced_copy_oracle_batch`, and `src/bin/lawful-pointer-oracle.rs` pass all host and GPU finite controls.
2. On an RTX 5090 pod (`scripts/pod/uor-pod`), `lawful-pointer-oracle` passes finite CUDA device-vs-host parity controls across all 3 `RouteAdmissionMode`s and all 4 oracle conditions (`N`, `O1`, `O2`, `W`), verifies `256/256` target and distractor spans and `0/128` `W` pair hits on `m2-fresh256`, measures `tokens/s` across two deterministic builds, passes `d29_compiled_modules` (`5/5`), and archives sealed artifacts to `caseyallard/uor-r4-store`.
3. Fresh-context `D68` decision review registers the `N`/`O1`/`O2`/`W` diagnostic bar on `#2880` and updates the prep sheet and lab README with the pinned `lawful_pointer.ptx` SHA-256 and GPU qualification numbers (`References #2880`).

## Stop and ask if

- Any edit would fall outside `crates/uor-r4-training/**`, `docs/harness/prep/r-lawful-pointer-forced-copy-oracle*`, `docs/labs/r-lawful-pointer-forced-copy-oracle/**`, or this task folder.
- Any path introduces floating-point arithmetic, squashing/exponentials (D23), matmul (D25/D30), or NVRTC (D29).
