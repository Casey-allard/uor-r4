# d11-exporter-float-trunk: the FBS artifact evaluator

This is the instrument of [d11-exporter-float-trunk](../../roadmap/items/d11-exporter-float-trunk.md), built in claude-128 (#2820). It is a GPU evaluator of a `UORLUT01` SpiralCore/FBS stack artifact. It exists so that the exported artifact itself, not an emulation of it, can be scored on the GPU panel (CHARTER §3.6).

- **`instrument/src/fbs_core.h`** is an integer-only port of the Rust session's FBS step (`stack_step_fbs`, its kernels and the native vector writer in `crates/uor-r4-integer/src/stack/`). It is compiled two ways:
  - by nvcc into the evaluator;
  - by the host C++ compiler into a twin that only qualifies the port. The twin never scores a panel.
- **The artifact's integers** come from `uor-r4-integer`'s `IntegerStackModel::fbs_image`. There is no second parser.
- **The crate is standalone,** outside the workspace (its own `[workspace]`), so CUDA and `unsafe` stay out of the serving crate.

## Build and test on the Mac

```bash
cd docs/labs/d11-exporter-float-trunk/instrument
CARGO_TARGET_DIR=<worktree>/local/target-instr ~/.cargo/bin/cargo test --target aarch64-apple-darwin
```

`tests/parity.rs` holds the host twin equal to `step_native`, bit for bit, at every position. It runs every single token of the three synthetic FBS controls (`uor_r4_integer::stack::synthetic`), and sequences that carry state for up to 64 positions.

## The CUDA evaluator and its CLI

`build.rs` builds `src/evaluator.cu` ahead of time with nvcc (no NVRTC), admits the PTX as integer-only, and assembles it to a cubin. The CLI needs the `cuda` feature and a GPU:

```bash
UOR_NVCC_ARCH=compute_120 cargo build --release --locked --target x86_64-unknown-linux-gnu --features cuda --bin d11-fbs-eval
d11-fbs-eval qualify OUT_JSON                              # synthetic controls against both references
d11-fbs-eval windows READOUT_JSON WINDOWS_JSON             # a float readout's DEV windows
d11-fbs-eval singles ARTIFACT SINGLES_JSON                 # every token, one per sequence
d11-fbs-eval run ARTIFACT SEQUENCES_JSON OUT_JSON --verify # evaluate; hold against both references
```

On the Mac, type-check the CUDA build with a stub nvcc: a script that, in `-ptx` mode, writes one `.visible .entry NAME() { ret; }` per `__global__` kernel, and in `-cubin` mode an ELF header. Set it as `NVCC` with `UOR_NVCC_ARCH=compute_120`.

claude-128 qualified it on an RTX 5090: it equals the host twin and the Rust session at all 32,256 positions of the synthetic controls and both Stage 4b artifacts. The record is the [prep sheet](../../harness/prep/d11-exporter-float-trunk-gpu-evaluator-2026-10-26.md) and the [REPORT](../../loop/tasks/d11-exporter-float-trunk/claude-128/REPORT.md).
