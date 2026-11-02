# REPORT: deterministic CPU discrete checkpoint and restart, deepseek-13

Lab: deepseek. Item: [cpu-discrete-training-path](../../../../roadmap/items/cpu-discrete-training-path.md) (#3501). Base: `245f47089`.

## What did not work, first

- **The first checkpoint draft deserialized an unchecked schedule.** A forged `batch_rows = 0`, `coordinate_block = 0` or zero-target schedule could reach cursor arithmetic before its constructor rules ran. Restore now validates every schedule field before any derivation.
- **The first source identity was too narrow.** It bound the artifact digest and source hashes but did not independently compare the restored model's read mode, initialization, context, vocabulary, block/stage shape, kept cells, sweeps or objective with the source schema. A coherently rehashed checkpoint could therefore describe a different model shape.
- **The first round-trip fixture used the canonical schedule for active-leaf Flock and Relation.** Reference configuration requires their active-leaf read initialization to use the interleaved schedule. The test now exercises the matching schedule for each mode.
- **The first continuation test read six tokens from a three-token Route context.** It failed before proving restart semantics; the control now uses a valid three-token prefix.
- **The first draft tested serialization across calls but not an actual process boundary.** A child-process helper now writes the next reading and canonical checkpoint digest after reloading the file.
- **Fresh adversarial review found an overflow hole in the K>1 move bound.** `accepted_updates * K` used `Option::is_some_and`, so multiplication overflow skipped the upper-bound check. Overflow is now a refusal, with a coherently rehashed impossible-cursor control.

## What changed

- Added versioned `CpuDiscreteCheckpoint` and `CpuDiscreteCheckpointDriver` APIs.
- The checkpoint binds the exact source-artifact SHA-256, source snapshot/schema/kappa/kernel/geometry hashes, read mode and initialization, full model shape, seed and objective, the complete Route/Flock/Relation state, derived chooser state, model epoch, schedule and reference-compatible `Cursor`/`BlockCursor`.
- Restore validates schema, substrate and qualification labels, hashes, source geometry, model shape/objective, schedule, cursor derivations, model/chooser consistency and `model_epoch == cursor.accepted_updates` before constructing an applier.
- Canonical and interleaved schedules are supported. Carrier schedules are explicitly refused until CUDA-side carrier target selection is represented.
- A committed BVG-K batch advances the model epoch and `accepted_updates` once. A no-commit batch advances `completed_batches`, `next_target`, `next_coordinate` and family block cursors without changing accepted updates, model bytes or chooser bytes.
- Route, Flock, Relation, Table, level-Gate and relation-tail mutations continue byte-identically after restore. A K=3 canonical block resumes correctly after a truncated two-coordinate family boundary.
- Corrupt/truncated/trailing JSON, unknown fields, invalid schedules, source/model/chooser hash mismatches, shape/objective mismatches, missing block fields, completed-batch overflow and accepted-update/move-bound multiplication overflow all refuse without constructing or mutating a driver.

## Measurements

- Focused checkpoint tests: `7 passed`, including actual child-process reload.
- Full `uor-r4-cpu-train` release suite: `92 passed`, `0 failed`, `4 ignored release benchmarks`.
- No corpus, training run, GPU run or language result occurred.

## Checks run at the head

- `CARGO_TARGET_DIR=target/deepseek-cpu-resume CARGO_BUILD_JOBS=1 cargo test --release --locked -p uor-r4-cpu-train`: PASS after the known duplicate-artifact collision retry, `92 passed`, `4 ignored`.
- `CARGO_TARGET_DIR=target/deepseek-cpu-resume CARGO_BUILD_JOBS=1 cargo clippy --locked -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings`: PASS; dependency crates emitted pre-existing dead-code warnings.
- `cargo fmt --package uor-r4-cpu-train -- --check`: PASS.
- `git diff --check`: PASS.
- `UOR_LAB=deepseek uor-harness ratchet d23`: PASS, 4,451 sites, baseline unchanged.
- `UOR_LAB=deepseek uor-harness ratchet d25`: PASS, 93 sites, baseline unchanged.
- `UOR_LAB=deepseek uor-harness ratchet d32`: PASS, 31 sites, baseline unchanged.
- `UOR_LAB=deepseek uor-harness loop pr-check --local`: PASS, with only the repository's file-map advisory.
- `UOR_LAB=deepseek uor-harness loop gates --full`: PASS, all 11 gates green.

## Skills used

No repository skill was invoked. The checkpoint was checked against the reference `Cursor`, `BlockCursor`, `family_cursors`, schedule and checkpoint validation code, plus a read-only restart-semantics audit.

## Still open

- Corpus-scale CPU training and capability/fresh-phrasing outcomes remain outside this component unit.
- The bounded pilot and quantization/float-shadow work remain under their separate roadmap items.

## Plain summary

The discrete CPU path can now save its complete model and reference-compatible training cursor, reload it in a new process, and continue with the same next BVG-K decision, update, reading and checkpoint bytes. This closes the deterministic component gap; it does not claim useful learning.
