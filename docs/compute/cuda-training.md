# CUDA training runbook (offline, optional)

Written 3 October 2026 (Eastern Time) by the support lab from [PR #1649](https://github.com/UOR-Foundation/uor-r4/pull/1649) (merged at `dc28b495`; read at main `63ec8388`), `crates/uor-r4-training` and the device-parity evidence. The Claude lab administers the GPU pod and owns the training code; if this page disagrees with the code or with `docs/evidence/cuda-delivery-2026-10-03.json`, those win.

## 1. Scope and non-claims

- **What it is:** an optional `cuda` Cargo feature of `uor-r4-training` and a `device=cuda` option of the `geometric-stack` example, for **offline training and evaluation only**.
- **What it is not:** no CUDA serving. The integer serving path (D11) is unchanged and does not use this feature.
- **Unsafe code:** the default build still forbids `unsafe`. With `cuda`, only the kernel launcher module `cuda_stack_kernels::cuda` allows it (one `unsafe` launch block).
- **No bitwise CPU/CUDA reproducibility is claimed**, except for the AdamW update (evidence `known_gaps`). Treat a CUDA run and a CPU/Metal run of the same configuration as different measurements.
- **Where things run:** training on the rented Runpod 2×RTX 4090 pod (Claude-administered); grading on the M1 laptop; GitHub hosted runners only for `main`'s required PR/merge-queue checks (owner direction, 3 October). Never push `codex/ci/*` branches to get GPU or CPU results.

## 2. What was verified, and where

| Item | Value (from the delivery evidence) |
|---|---|
| Delivery-head run | source `c17411b8`, fresh shallow clone, 0 changed paths |
| Host | Runpod, 2× RTX 4090, driver 595.91.07, CUDA 12.6, `CUDA_COMPUTE_CAP=89` |
| Toolchain | rustc 1.97.1; release profile; `CARGO_INCREMENTAL=0` |
| Command | `UOR_REQUIRE_CUDA=1 cargo test --release -p uor-r4-training --features cuda --test cuda_stack_ops_parity -- --test-threads=1` |
| Result | **18 passed, 0 failed, 1 ignored** (`bench_training_size_read_and_recurrence`); nothing skipped |
| Test binary | `cuda_stack_ops_parity-0bb11c69ad774b8a`, sha256 `a7997c92…0b21` (full digest in the evidence JSON) |
| Earlier runs | source `28f50a4e` on 1× RTX 4090; same 18/0/1; worst reported max relative error 3.4e-6; no NaN/Inf. **Executable digest UNAVAILABLE** (pod destroyed before digests were recorded); no hash was substituted. |

## 3. Prerequisites on a GPU host

- NVIDIA driver and a **CUDA toolkit** on the build host. `cudarc` is built with `driver`, `nvrtc`, `cuda-version-from-build-system` and `dynamic-linking`, so the toolkit must be discoverable at build time and the driver at run time. CUDA 12.6 with driver 595.91.07 is what was verified; other versions are untested here.
- **`nvcc` must be on `PATH` at build time.** The `cuda-version-from-build-system` feature of `cudarc` runs `nvcc` from its build script to pick the CUDA version, and the build fails without it (for example `export PATH=/usr/local/cuda/bin:$PATH`). This is separate from the stack kernels themselves, which are compiled at run time with NVRTC.
- Set `CUDA_COMPUTE_CAP` for the card (89 for RTX 4090) as the verified run did.
- A pinned Rust toolchain matching the verified run is preferable; record `rustc -V`.
- Kernels are compiled at run time with NVRTC; `test_cuda_kernels_compile_with_nvrtc` needs only the toolkit, not a device.

## 4. Build and run a training job

The training driver is the `geometric-stack` example. Build once per pod and **record the executable's sha256 before the pod is destroyed**; the earlier runs could not be bound for exactly that reason.

```sh
export PATH=/usr/local/cuda/bin:$PATH   # nvcc for cudarc's build script
CARGO_INCREMENTAL=0 CUDA_COMPUTE_CAP=89 \
  cargo build --release -p uor-r4-training --features cuda --example geometric-stack
shasum -a 256 target/release/examples/geometric-stack   # record this
```

Select the device explicitly: `device=cuda`. There is **no implicit fallback**; omitting `device=` runs on the CPU. Options come from the example's `--help`; the ones that matter operationally are:

- `train`: `out=NEW_REPORT_ROOT`, `arch=geometric`, `read=lorentz|l2|dot` (**default `lorentz`**; the current scale ladder uses `read=l2`, so pass it explicitly), `rotation_group=quaternion|u1`, `steps`, `batch`, `lr`, `warmup`, `eval_every`, `checkpoint_every`, `resume=OLD_ROOT/checkpoint`, `max_seconds`, `width`, `heads`, `layers`, `context`.
- `dialogue-train` (chat fine-tunes): the same device/checkpoint options plus tokenizer, train/dev token+mask+manifest paths, `policy=`, `protocol=1|2`, `data_seed`.
- Report roots are created exclusively (a new `out=` every run); never reuse a sealed root.
- Use `checkpoint_every` and `max_seconds` so a stop (pod preemption, wall limit) resumes via `resume=`. A configured limit is a stop, not a budget.
- `UOR_CUDA_PROFILE=1` synchronizes after each launched custom kernel and prints its wall time on stderr. It is diagnostic only: it serializes the GPU, and it does not see cuBLAS calls or host/device copies.
- `UOR_NAN_TRACE=1` checks every gradient and parameter each step. The retained 8M Lorentz seed-2 trace ran to step 4000 with 0 non-finite reports.

## 5. Checks before trusting a GPU result

1. **Parity test**, serially, with the device required (the command in §2). `UOR_REQUIRE_CUDA=1` is mandatory: without a device each device test prints that it was skipped and returns `Ok`, so a pass count proves nothing.
2. Run with `--test-threads=1`. A first *parallel* run failed only the SwiGLU speed assertion; run serially it passed (8.43× reported). The speed test is timing-sensitive.
3. Record on the PR or issue: head SHA, a clean tree, the exact command, exit codes and the `test result:` lines, the test binary and training executable sha256, GPU model/driver/CUDA version.
4. Compare configurations only within the same device class unless the comparison is declared; see §1.

## 6. Known gaps (from the evidence; check the code before relying on this list)

- RoPE and flock selection inside the fused read use the **host (CPU) fallback**, so a RoPE-based control runs mostly on the CPU even with `device=cuda`. A CUDA RoPE kernel is tracked separately.
- The pointer mixture has CUDA forward and backward kernels for Dot and Lorentz scores over every source; a `pointer_select` or `pointer_route` configuration still uses the host fallback.
- Operations without a GPU kernel run on host copies; GPU speed-up is therefore workload-dependent.
- The reported single-RTX-4090 training throughput at 8M, 20M and 29M was 55.4k, 67.8k and 49.6k tokens/s (a log summary in the evidence; not a benchmark of this page's recipe).

## 7. Operating rules

- Training and long arms run on the owner-authorized pod or the M1, not on hosted runners. No other paid or external compute is authorized.
- One cargo process at a time on a given host; do not edit another lab's worktree or jobs.
- Keep retained checkpoints and negative results. Never delete unique artifacts; a destroyed pod takes its disk with it, so copy checkpoints, logs and hashes off the pod first.
- Report training time, evaluation time and orchestration time separately, and charge the cumulative ledger.
