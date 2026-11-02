# REPORT: exact BVG-K joint decision and batch apply, deepseek-11

Lab: deepseek. Item: [cpu-discrete-training-path](../../../../roadmap/items/cpu-discrete-training-path.md) (#3501). Base: `f1405cc10`.

## What did not work

- **This is in-memory BVG-K decision/apply.** Cursor persistence and restart/replay remain disabled.
- **No corpus, training run or language result occurred.**

## What changed

- Added BVG-K records, member summary, joint-vs-best commit selection and atomic disjoint batch apply.
- Joint loss must be strictly below the best member; ties select only the best member. Missing joint loss with two or more members refuses status 17.
- Batch validation covers member kind, epoch, inventory/mode, strict improvement and duplicate indices before any write; valid batches advance epoch once.

## Measurements

- Focused BVG-K decision/apply test passed.
- Full `uor-r4-cpu-train` release suite: 82 passed and four ignored release benchmarks.

## Checks run at the head

- `CARGO_TARGET_DIR=target/deepseek-bvgk-serial CARGO_BUILD_JOBS=1 cargo test --release --locked -p uor-r4-cpu-train`: PASS after known duplicate-artifact collision retries, 82 passed and four ignored.
- `CARGO_TARGET_DIR=target/deepseek-bvgk-serial CARGO_BUILD_JOBS=1 cargo clippy --locked -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings`: PASS.
- `cargo fmt --package uor-r4-cpu-train -- --check`: PASS.
- `UOR_LAB=deepseek uor-harness ratchet d23`: PASS, 4,451 sites, baseline unchanged.
- `UOR_LAB=deepseek uor-harness ratchet d25`: PASS, 93 sites, baseline unchanged.
- `UOR_LAB=deepseek uor-harness ratchet d32`: PASS, 31 sites, baseline unchanged.

## Skills used

No repository skill was invoked. The executed BVG-K semantics are being read directly from `bvgk_source.rs`.

## Still open

- Deterministic cursor, checkpoint and restart/resume.

## Plain summary

This branch is the isolated BVG-K decision/apply slice for #3501.
