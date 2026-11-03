# REPORT: served-export contract guard, turn-memory-integer deepseek-2

Lab: deepseek. Item:
[turn-memory-integer](../../../../roadmap/items/turn-memory-integer.md) (#3392).
Order: [ORDER.md](ORDER.md). Base:
`66c50fa19a3acc783819c198c598f24a2a55fe67`.

## What did not work

- The unrestricted integer release suite is not green in this worktree. Six
  adversarial token-boundary tests hard-code missing
  `/Users/casey.allard/uor-r4-investigations/...` fixtures and fail before
  reaching this change. The retained output is
  [integer-release-full-attempt.txt](evidence/integer-release-full-attempt.txt).
- Strict Clippy is not green at this base under the local Rust 1.97.1
  toolchain. It reports 70 pre-existing lints across the integer library and
  its tests, none in `bin/fbs-export.rs`. The output is retained in
  [integer-clippy.txt](evidence/integer-clippy.txt).
- A non-denying Clippy run completed the exporter target with 51 pre-existing
  library warnings and no diagnostic in the changed file:
  [fbs-export-clippy-warn.txt](evidence/fbs-export-clippy-warn.txt).
- This guard only runs when the caller supplies `--run-config`; it cannot
  infer training history from a snapshot alone.
- Pre-claude-265 configs also used `train_as_served: true` for the snapped
  contract. Those historical checkpoints require their matching source binary;
  the current guard deliberately refuses to reinterpret them as the current
  unsnapped contract.

## What changed

`fbs-export --run-config` now parses `train_as_served` before constructing the
artifact. A true value requires both `--raw-gates` and `--raw-transition`,
matching the current `SERVED_SNAPS` contract. A non-boolean value is refused.
A JSON root that is not an object is refused. Absent and false values preserve
the caller's existing snapped/raw choice.

The rejection occurs before artifact construction and before
`write_new_file`, so a refused contract cannot create its requested output.
The documented command surface and export error name the required flags.

The implementation is 104 added and 4 removed lines in
`crates/uor-r4-integer/src/bin/fbs-export.rs`: the pure contract check, its
call, three focused tests and the updated command documentation.

## Measurements

- Focused `fbs-export` tests: **3 passed, 0 failed**. They cover all four raw
  flag combinations, absent and false legacy configs, malformed JSON, null,
  integer, and string values for `train_as_served`, and array and string JSON
  roots.
- Integer library, `fbs-compare`, `fbs-export`, `fbs-toks`, and `uor-chat`
  tests in release: **467 passed, 0 failed, 1 ignored**.
- `rustfmt --check --edition 2021` on the changed exporter: passed.
- Package-wide `cargo fmt --check` reports one pre-existing formatting drift
  in `bin/fbs-toks.rs`, outside this item; the output is retained in
  [integer-fmt-check.txt](evidence/integer-fmt-check.txt).
- `git diff --check`: passed.

No model, corpus, artifact bytes or serving arithmetic changed. No GPU or
pre-registered experiment was needed for this host-only instrument repair.

## Checks run at the head

- `cargo test --locked --package uor-r4-integer --features serving-wasm
  --bin fbs-export`: 3 passed.
- `cargo test --locked --package uor-r4-integer --features serving-wasm
  --release --lib --bins`: 467 passed, 1 ignored.
- The full package test was attempted; its unrelated missing-fixture failure
  is recorded above.
- Strict Clippy was attempted; its pre-existing baseline failures are recorded
  above, and a non-denying exporter run reached the changed target with no
  diagnostic there.
- Changed-file formatting and whitespace checks passed.

## Skills used

No available Codex skill applied. The exporter, run configuration and
unsnapped training constant were inspected directly.

## Still open

- Re-record the served contract explicitly in future run configs, so absence
  of the historical `train_as_served` ambiguity can eventually be rejected.
- Make `--run-config` mandatory for exports produced by the current
  served-training command.
- Return to Claude's full-length run when results or local pod state become
  available; this unit neither waits on nor edits that branch.

## Plain summary

A model trained through the unsnapped serving path could previously be
exported with snapped gates if the caller forgot two flags, even after passing
its run configuration to the exporter. The exporter now reads the training
flag and refuses that mismatch before writing anything. Older non-served
exports behave as before.
