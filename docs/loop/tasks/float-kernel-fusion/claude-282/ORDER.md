# ORDER: float-kernel-fusion, claude-282 (done-when 1: the generated kernel list)

Lab: claude. Item: [float-kernel-fusion](../../../../roadmap/items/float-kernel-fusion.md) (#3678). Base: `a0c0dd73d` (main after claude-281). Draft PR #3684.

## Claim check

- **The step it advances:** THE ORDER of 4 Nov (#2028 6634218223), step 5, the kernel unit, part (b): the generated kernel list. Part (a), the item, is claude-280. Part (c), the measured fusion, needs this list first: a fused kernel added to `cuda_stack.cu` would otherwise move two hand copies of 26 names and four literal counts.
- **What it changes in the model that trains or serves:** no math, no weight and no scored token. Every kernel launch in training and scoring (`dispatch_module`) now passes a check against the list `build.rs` generates from the sources its module compiles. Today only the D29 deny list and the PTX load stand between a name and its launch.

## Goal

The item's done-when 1, in full:

> 1. **The kernel list is generated.**
>    - `build.rs` scans each `MODULES` entry's combined `sources` (`build.rs:26-54`) for `extern "C" __global__ void <name>` on every build, CPU builds included. It writes one list per PTX module, which `lib.rs` includes. The arms module is built from three files, `spiralcore_fbs_arms_prelude.cu`, `spiralcore_fbs.cu` and `spiralcore_fbs_arms.cu` (`:44-53`). So its list is the 4 FBS kernels plus `spiralcore_fbs_arm_fwd` and `spiralcore_fbs_arm_bwd`.
>    - The dispatch refuses a name not in its own module's list. The arms kernels, launched through `launch_prebuilt` (`geometric_stack_native_cuda.rs:798-805`), must pass that check, not fail it.
>    - No literal kernel count is left in code, tests or their doc comments: `lib.rs:3-4, :33-35`, `tests/no_training_crate_dep.rs:3-4`, and the messages at `:83` and `:104`.
>    - The test re-scans the sources itself and compares with the generated lists. The banned-token scan stays.

Done-when 2 to 5 (the measurement, then the fused quantizers if the cancel rule lets them start) are later units.

## Cancel probe (run first)

- **Is done-when 1 already met on main?** `build.rs` has no scan, `lib.rs:36` and `tests/no_training_crate_dep.rs:9` are still two literal 26-name arrays, and `dispatch_module` checks only `HISTORICAL_KERNELS`.
- **Does an open PR touch these files?** `loop claim`'s overlap check.
- **Classed:** instrument (build and test plumbing; no mechanism is tested).

## Cancel probe result (4 Nov, about 23:30Z)

- **Not met on main** at `a0c0dd73d`: `build.rs` scans nothing, and the two 26-name copies and the counts at `lib.rs:3-4, :33-36` and `tests/no_training_crate_dep.rs:3-4, :9, :83, :104` stand. `dispatch_module` (`cuda_stack_kernels.rs:347`) checks `HISTORICAL_KERNELS` (`:356`), then loads (`:376`).
- **No overlap:** `loop claim` passed its overlap check and opened #3684.
- **What the scan meets:** every line with `__global__` in the four `.cu` files is an `extern "C" __global__ void <name>(` declaration, and none sits inside an `#if` block (the `#if`s in `cuda_stack.cu` are at `:61-85` and `:120-131`, before the first kernel at `:160`). So a text scan gives the same list for the f32 and bf16 builds.
- **One load path:** `dispatch_module` is the only `get_or_load_custom_func` in the crate. It serves three modules: `uor_r4_geometric_stack` and `uor_r4_geometric_stack_bf16` (`cuda_stack_kernels.rs:35, :38`), and `uor_r4_spiralcore_fbs_arms` (`geometric_stack_native_cuda.rs:728`, through `launch_prebuilt`).

## Files in scope

- `crates/uor-r4-float-train/build.rs`
- `crates/uor-r4-float-train/src/lib.rs`
- `crates/uor-r4-float-train/src/cuda_stack.cu`
- `crates/uor-r4-float-train/src/cuda_stack_kernels.rs`
- `crates/uor-r4-float-train/src/fused_quantize*.rs`
- `crates/uor-r4-float-train/src/hamilton_sheet.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-float-train/tests/no_training_crate_dep.rs`
- `crates/uor-r4-float-train/tests/twenty_four_kernel_controls.rs`
- `crates/uor-r4-float-train/tests/fused_*.rs`
- `crates/uor-r4-float-train/Cargo.toml`
- `docs/harness/prep/float-kernel-fusion*`
- `docs/loop/tasks/float-kernel-fusion/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/float-kernel-fusion.md`

This unit touches only `build.rs`, `lib.rs`, `cuda_stack_kernels.rs`, the two tests, `Cargo.toml` and its task folder.

## Done when

1. **Generated lists.** `build.rs` writes `kernel_lists.rs` into `OUT_DIR` on every build, before the CPU early return: one list per module, scanned from that module's sources, plus each module's PTX file and sources. A line with `__global__` that is not an `extern "C" __global__ void <name>` declaration stops the build, as do a module with no kernel and a name listed twice. `lib.rs` includes the file.
2. **The dispatch check.** `dispatch_module` refuses a name its module's list does not hold, and a module it has no list for. The check is a plain function outside the `cuda` module, so CPU tests cover it:
   - each module's own kernels pass;
   - another module's kernel fails, as do `HISTORICAL_KERNELS` names and an unknown module;
   - the arms module's name is the one `geometric_stack_native_cuda.rs` loads its PTX under;
   - with the check's own lines set aside, the refusal test fails (the negative gate).
3. **No literal count** at `lib.rs:3-4, :33-36`, `tests/no_training_crate_dep.rs:3-4, :9, :83, :104`, in `build.rs:4`, in `tests/twenty_four_kernel_controls.rs:1-5` or in `Cargo.toml`'s description.
4. **The test re-scans:** `tests/no_training_crate_dep.rs` reads each module's sources itself, scans them with its own code and compares with the generated lists. It checks that every `src/*.cu` file is in some module. The banned-token scan covers every module's sources.
5. **Checks:**
   - the crate's CPU suite and the stub-nvcc `cargo check --features cuda --all-targets` pass on the laptop;
   - on one RTX 5090 at the head, the crate's CUDA library suite passes with `--no-fail-fast`, including `spiralcore_fbs_arms::gpu_tests`, which launch the arms kernels through `launch_prebuilt`. This is a finite kernel test, so it needs no prep sheet (§4.5).
6. **REPORT** with negatives first; a fresh exact-head review; `loop report --check` and `loop pr-check --local` pass.

## Stop and decide (never wait) if

- the GPU suite fails on a test this unit did not touch: classify it (instrument or mechanism), and repair it only if it is in scope; otherwise record it and land done-when 1 on the tests that cover it;
- a module's text scan cannot match what nvcc compiles (a kernel under `#if`): refuse the declaration at build time and name the rule in the REPORT;
- the pod costs more than $1: extend once, note it on #3678, and never cancel a healthy run.
