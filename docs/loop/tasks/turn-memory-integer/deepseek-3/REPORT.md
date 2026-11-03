# REPORT: turn-memory-integer, deepseek-3

Lab: deepseek. Item:
[turn-memory-integer](../../../../roadmap/items/turn-memory-integer.md) (#3392).
Order: [ORDER.md](ORDER.md). Base:
`c58b80a7a72b391bdbaf42cfdc2b4dbdab95cdda`.

## What did not work, first

- **No GPU execution was available.** `nvidia-smi` is absent on this host and
  `uor-pod status` reported zero running pods and no free GPU offers. The
  exact readout command is recorded below as `NOT_RUN`; no CPU result is
  substituted for GPU execution.
- **Strict float Clippy is not green at the repository baseline.** It reports
  five unrelated lints in `flock_read_memory.rs`, `flock_read_register.rs` and
  `in_space_long.rs`. The changed files have no diagnostic. A non-denying run
  is retained.
- **Strict integer Clippy is also not green at baseline.** It reports 70
  library warnings and an unrelated `unused_io_amount` error in
  `tests/openai_loopback.rs`. No warning points into `fbs_snapshot.rs`.
- **Package formatting is not green at baseline.** `fbs_export.rs` already
  contains unrelated rustfmt drift. The changed blocks are formatted, and the
  other two changed Rust files pass `rustfmt --check`. The exact package-level
  diff is retained rather than reformatting unrelated code.
- **This is not yet a layer-state trace.** It supplies a D11 readout under the
  complete served contract. A subsequent instrument must expose per-layer
  residual values before Claude can localize the first divergent layer.

## What changed

`fbs-snapshot` now accepts `--served-readout` with the fixed readout
sequences, `--dev`, or `--hamming-trace`. In this mode it requires exactly one
readout path and runs that D11 readout through
`with_served_surrogate_training`: `SERVED_SNAPS`, Q12 query, Q16 residual,
dynamic i16 activations and Q16 projection outputs.

`FbsFloatReadout` gains a backward-compatible `served_readout` marker. It
defaults to false and is omitted from old/plain/export/snapped D11 JSON; a
true marker is refused outside D11. The mode is therefore visible without
changing the mode name `fbs-compare` already consumes.

The option parser accepts `--dev` and `--served-readout` in either order and
refuses duplicates or malformed combinations before opening the GPU. The same
flag selects the served contract for Hamming-trace readouts.

The Rust change is 224 added and 27 removed lines across three files, including
two focused tests. Documentation adds this report and one notebook row.

## Measurements

No runtime measurement is claimed.

- Float library: **218 passed, 0 failed**.
- `fbs-snapshot` binary: **7 passed, 0 failed**.
- Integer release library and binaries: **468 passed, 0 failed, 1 ignored**.
- The host test validates every served flag is on inside the wrapper and all
  prior thread-local state is restored after success and error.
- The GPU command to run once a checkpoint/pod is available is:

  ```sh
  fbs-snapshot --dev DATA_ROOT --served-readout \
    MODEL_DIR SNAPSHOT.json SERVED_READOUT.json
  ```

  It is `NOT_RUN`: no local GPU and no free pod.

## Checks run at the head

- `cargo test --locked -p uor-r4-float-train --lib --bin fbs-snapshot`:
  passed; full output in
  [float-readout-tests.txt](evidence/float-readout-tests.txt).
- `cargo test --locked --release -p uor-r4-integer --features serving-wasm
  --lib --bins`: passed; full output in
  [integer-release-tests.txt](evidence/integer-release-tests.txt).
- `cargo clippy --locked -p uor-r4-float-train --all-targets --no-deps -- -D
  warnings`: failed only on five pre-existing files; changed files clean.
- `cargo clippy --locked -p uor-r4-integer --features serving-wasm
  --all-targets --no-deps`: completed except for one pre-existing denied
  warning; changed file clean.
- Package `rustfmt` check and `git diff --check` are retained separately.
- `uor-pod status`: zero running pods, no free GPU offers.

## Skills used

No available Codex skill applied.

## Still open

- Run the served readout and snapshot against claude-265's sealed checkpoints
  once a GPU is available.
- Add per-layer residual capture, which this unit does not claim. That is the
  next step for locating the remaining I-3 parity sites.
- Keep the output as an instrument: no model, serving artifact or capability
  result changes here.

## Plain summary

The model can already be trained under the exact rounding rules used by the
integer server, but the old diagnostic readout did not use those same rules.
That made it unsafe for finding why a raw-trained model and its served copy
still disagree. This unit adds one explicit option that runs the diagnostic
under the complete served contract and labels the output as such. It passes
all host tests and leaves old output unchanged. The remaining step is to run
it on a GPU and add finer per-layer values.
