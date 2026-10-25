# Notebook: D37 offline float training (`float-training`)

This notebook records results on the **D37 offline float training** line (`CHARTER.md` §3.7, D37): offline floating-point forward, backward and optimizer steps inside the separately scanned `crates/uor-r4-float-train` module, where every trained candidate must export through the D11 integer exporter and report its export gap against the discrete evaluator. No language capability or trained float-to-integer export is established here yet.

Background: [the geometric stack](../../mechanisms/geometric-stack.md) describes the 24-kernel native geometric stack extract (`src/cuda_stack.cu` + `src/spiralcore_fbs.cu`), and [`docs/labs/float-training-module/README.md`](../../labs/float-training-module/README.md) documents the standalone crate boundary and D37 item 6 ratchet entry.

## Results

Verdict words are defined on the [format page](./README.md#the-five-verdict-words).

| Date | What ran | Number | Bar | Verdict | Artifact |
|---|---|---|---|---|---|
| 2026-10-25 | Built the standalone D37 offline float training crate `crates/uor-r4-float-train` (`uor-r4-float-train`) carrying the 24-kernel geometric stack extract (`22` `__global__` kernels in `src/cuda_stack.cu` + `2` in `src/spiralcore_fbs.cu`), ran the CPU dependency-graph exclusion test (`cargo tree --offline -p uor-r4-float-train`), 24-kernel finite CPU arithmetic controls, stub-`nvcc` `--features cuda --all-targets` type check, and the D23/D25/D32 ratchets under `D37_SCANNED_FLOAT_CRATES = ["crates/uor-r4-float-train/"]`. No GPU run and no training. | 19 of 19 CPU tests pass (`5` lib contract tests, `2` dependency/D29 entry-point guards, `12` finite control tests covering all 24 kernels); 0 `uor-r4-training` or `candle-nn` edges in `cargo tree`; 0 D23 sites and 0 D25 sites in `crates/uor-r4-float-train/`; stub-`nvcc` `--features cuda --all-targets` exits 0 | Standalone crate builds CPU-only with CPU tests, passes the banned-token scan under its own scanned entry (`D37_SCANNED_FLOAT_CRATES`), carries the 24-kernel extract with 0 dependency edges on `crates/uor-r4-training`, and passes the stub-`nvcc` `--features cuda` type check (`#2823`) | positive (finite CPU controls and static module boundary only; GPU qualification and W4 training are separate downstream items) | [float-training-module](../../labs/float-training-module/README.md) |
