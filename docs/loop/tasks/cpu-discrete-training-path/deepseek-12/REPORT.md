# REPORT: repair BVG-K decision parity and batch atomicity, deepseek-12

Lab: deepseek. Item: [cpu-discrete-training-path](../../../../roadmap/items/cpu-discrete-training-path.md) (#3501). Base: `4b1d71121`. Repair of [deepseek-11](../deepseek-11/ORDER.md).

## What did not work, first

- **The first BVG-K apply validated active lanes too late.** Gate lanes `3..=15` and Route/Table lanes above `15` could reach mutation; a bad later Gate member could leave an earlier member written before the batch refused.
- **The first decision silently filtered malformed records.** A mismatched kind or read mode was removed from member selection instead of reaching CUDA-compatible apply refusals `14` or `9`.
- **The first failure tests covered only status `17`, duplicate `15` and a subset of single-apply refusals.** They did not exercise status order, every batch refusal, epoch exhaustion or a bad later member.
- **The first full gate run hit a transient test-harness race.** Its fake `gh` executable returned `Text file busy`; the unchanged full gate rerun passed.

## What changed

- Member selection now uses status, eligibility, the active lane and inventory range, while preserving malformed kind and mode for apply-time validation.
- Batch apply validates kind `14`, epoch `3`, mode `9`, lane/inventory `4`, improvement/epoch exhaustion `5`, duplicate index `15` and empty commit `16` before any write.
- All committed moves are staged in a cloned model. Model and chooser state are assigned together only after every move succeeds, and epoch advances once.
- Added failure and immutability controls for every listed status, Route lane `16`, Gate lane `3`, and a valid first member followed by a bad later member.

## Measurements

- The full `uor-r4-cpu-train` release suite passed `83` tests with `4` ignored release benchmarks.
- The new focused failure/immutability test passed with the full suite.
- No corpus, training run or language result occurred.

## Checks run at the head

- `CARGO_TARGET_DIR=target/deepseek-bvgk-repair CARGO_BUILD_JOBS=1 cargo test --release --locked -p uor-r4-cpu-train`: PASS after the known duplicate-artifact collision retry, `83 passed`, `4 ignored`.
- `CARGO_TARGET_DIR=target/deepseek-bvgk-repair CARGO_BUILD_JOBS=1 cargo clippy --locked -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings`: PASS; dependency crates emitted pre-existing dead-code warnings.
- `cargo fmt --package uor-r4-cpu-train -- --check`: PASS.
- `UOR_LAB=deepseek uor-harness ratchet d23`: PASS, 4,451 sites, baseline unchanged.
- `UOR_LAB=deepseek uor-harness ratchet d25`: PASS, 93 sites, baseline unchanged.
- `UOR_LAB=deepseek uor-harness ratchet d32`: PASS, 31 sites, baseline unchanged.
- `UOR_LAB=deepseek uor-harness loop pr-check --local`: PASS, including ORDER-first, scope, report, banned-presence, rule-source and read-path checks.
- `UOR_LAB=deepseek uor-harness loop gates --full`: PASS on the unchanged rerun, all 11 gates green.

## Skills used

No repository skill was invoked. The repair was checked against `bvgk_source.rs` and reviewed independently in read-only subagent audits.

## Still open

- Deterministic cursor, checkpoint, restart and replay.
- A corpus or bounded CPU pilot after the exact path is complete.

## Plain summary

This repair makes a multi-coordinate BVG-K update all-or-nothing. It also makes malformed records fail with the same status code and ordering as the CUDA implementation, and it proves a late bad member cannot leave earlier edits behind.
