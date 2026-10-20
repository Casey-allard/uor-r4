# The D23 and D25 ratchets in Rust (2026-10-13)

**Decisions:** D23 (no normalised exponential), D25 (no dense matrix product), D30 (Rust only; Python is a
transitional shim). Owner, 13 October: Rust ports are every lab's job, not only DeepSeek's.

**Line:** tooling · no model line · headlines unchanged. No model compute.

## What changed
- **New command:** `uor-harness ratchet d23|d25 [--update-baseline] [--list] [--quiet]`, in
  `crates/uor-r4-harness/src/ratchet.rs`. It ports `scripts/check_no_softmax.py` and `scripts/check_no_matmul.py`
  with the same behaviour:
  - the same scan: `crates/`, suffixes `.rs .cu .cuh .wgsl`, `target/` skipped;
  - for D25, the same comment and plain-string stripper, with raw strings kept;
  - the same baseline files and verdicts, and an update that may only lower counts.

  `softmax` and `matmul` are accepted as aliases.
- **Self-matching avoided.** The port contains no banned word: the D23 word and the tensor-core needles are
  assembled with `concat!`, so the Rust checker never counts against itself. (The Python scripts escaped this
  only because `.py` files are not scanned.)
- **D23 baseline lowered 1648 → 1647.** `crates/uor-r4-training/src/cuda_stack_kernels.rs` goes 6 → 5, the
  site #2344 removed.

## Parity with the Python ratchets (`daa74c9c5`)
- **D23:** the Rust `--update-baseline` rewrote the 144-file baseline. The diff against the Python-written
  file is one line, the 6 → 5 drop above. Every other file's count and the file's formatting are
  byte-identical.
- **D25:** the rewritten 26-file, 93-site baseline is byte-identical to the Python one.
- **Growth is refused** live as well as in the tests. The first build, before the self-matching fix, failed its
  own check (`ratchet.rs 0 -> 31`, `main.rs 0 -> 9`), and `--update-baseline` refused the rise.

## Checks
- `cargo test -p uor-r4-harness`: 138 + 1 pass, including 6 new ratchet tests:
  - the stripper (nested comments, escaped strings, raw strings kept, lines preserved);
  - whole-word `gemm`;
  - non-overlapping tensor-core counting;
  - the case-insensitive D23 count;
  - the Python-identical baseline rendering;
  - lowering accepted, a rise refused, and `target/` and `docs/` skipped.
- `cargo clippy --all-targets` and `cargo fmt --check` are clean.

## Not done here (DeepSeek, harness owner; #2028 6150176928)
- Point prlint and CI at the Rust command, then retire the two Python scripts. AGENTS.md and D24 still name
  the Python script as the interlock, and the PM updates those.
- **Limitation, unchanged by the port:** both ratchets are lexical. A dense contraction written as scalar
  loops passes them, which is how Codex found two in #2344's modules. Those modules are being quarantined
  in the next Claude PR.
