# REPORT: d11-exporter-float-trunk, claude-128

Lab: claude. Order: [ORDER.md](ORDER.md). Prep sheet: [d11-exporter-float-trunk-gpu-evaluator-2026-10-26](../../../../harness/prep/d11-exporter-float-trunk-gpu-evaluator-2026-10-26.md) (FIT round 1; D67 pre-run review GO).

References #2820.

## What did not work, first

- **The export gap is still not measured.** This unit qualifies an instrument and measures nothing about the model. `G-D11-ExportGap` stays blocked until the next unit scores the panel with this evaluator.
- **The gap bar's loss half is not built yet.** It is the `MarginHinge(rivals = 8)` loss from each scored Q12 query. The evaluator writes those queries, but the float sheet does not yet compute the loss from them on the GPU. That is part of the next unit.
- **One GPU thread is slow.** A thread steps a sequence at about 13 positions per second: 54 windows of 192 took 14.5 s. The evaluator's throughput comes from running sequences side by side, and this run kept only 54 threads busy, so its wall times are an upper bound. Nothing is parallel within a sequence.
- **Two builder subagents landed nothing, about 330k tokens in all.**
  - The first spent about 180k reading before the loop guard refused its first write: a builder needs a suborder (`uor-harness loop suborder`), and none had been written.
  - The second started near the 150k hand-back mark and stopped before writing anything.
  - Every change here was made in the main session instead. The friction log has the convention.
- **One saved checksum does not verify: `evalq/job.log`.** The job wrote `SHA256SUMS` and only then appended its last log line (`finished, EXIT 0`), so `shasum -c` reports `job.log: FAILED`. The other 74 files of `evalq/`, and `evalq-bin/`, verify. The sealed store is left as written (L-002). The job script for the next run must hash the log after its last line.
- **`uor-pod down` could not verify the archive over SSH,** as in claude-126. The upload was confirmed with `uor-store ls`, and `down` was re-run with `--confirm-archived`.

## What changed

Size against `origin/main`:
- integer crate: +336/−130 across `stack/session.rs`, `synthetic.rs`, `tests.rs`, `mod.rs` and `Cargo.toml`;
- instrument: 2,268 lines, of which 425 are its `Cargo.lock`;
- docs: the ORDER, the prep sheet, the lab README, this REPORT and the notebook row.

**Changes:**
- **`IntegerStackModel::fbs_image`** returns `FbsImage`, `FbsLayerImage` and `FbsMapImage`. These are a read-only, zero-copy view of a parsed FBS model's integers, with the step's exponents and its geometric drive weights as constants. The view adds no arithmetic, and a dense stack returns `None`. Tested by `fbs_image_matches_its_shape`.
- **`stack::synthetic`**, behind the new `fbs-synthetic` feature, holds the synthetic FBS controls that claude-122 and claude-127 kept in `tests.rs`. They are now shared with the evaluator, through `fbs_synthetic_non_square` and `fbs_synthetic_tiny_gain`. The draws are unchanged: every FBS test prints the counts it printed before.
- **`docs/labs/d11-exporter-float-trunk/instrument/`** is a standalone crate outside the workspace, modelled on `m1-d11-artifact-scorer`:
  - **`fbs_core.h`** is an integer-only port of `stack_step_fbs`, its kernels and the native vector writer, for the FBS shape: geometric gates, snapped transport, rotations and the Hamilton gate.
    - It keeps every floor, round-half-up, saturation and wrapping convention of the Rust kernels.
    - Its products are exact modulo 2^64 or 2^128, as the Rust table products are.
    - A sequence's buffers are carved from one region (`fbs_work_carve`), identically on the host and on the device.
  - **`host_kernel.cpp`** is the host twin. It qualifies the port and never scores.
  - **`evaluator.cu`** runs one thread per sequence, 64 per block.
  - **`build.rs`** runs nvcc ahead of time, then the integer PTX admission of the template with its self-test, then cubin assembly. No NVRTC.
  - **`device.rs`** holds the FBS image in device memory, through cudarc.
  - **The `d11-fbs-eval` CLI:**
    - `qualify` holds the synthetic controls against both references;
    - `run … [--verify]` evaluates any sequence file;
    - `windows` and `singles` write the inputs.
  - **The tests:** `tests/parity.rs` holds the host twin against `step_native` at every position. `tests/trained.rs` is opt-in host parity on a trained artifact.

## Measurements

**The qualification, on one RTX 5090** (driver 595.71.05, nvcc 12.8, `sm_120`). The binary run was built on the pod from `ac47d01d4aacb94b9f98b5da20320af812e3449c`:

| Gate | Input | Positions | G = H | G = R |
|---|---|---|---|---|
| `Q-Synthetic` | small trunk: single tokens, then 16 sequences of 8 | 256 + 128 | yes | yes |
| `Q-Synthetic` | non-square trunk: single tokens, then 16 sequences of 16 | 512 + 256 | yes | yes |
| `Q-Synthetic` | tiny-gain trunk: single tokens, then 16 sequences of 8 | 512 + 128 | yes | yes |
| `Q-Synthetic` | non-square trunk, context 64: single tokens, then 16 sequences of 64 | 512 + 1,024 | yes | yes |
| `Q-Trained` | seed 1: 4,096 single tokens, then 54 DEV windows of 192 | 4,096 + 10,368 | yes | yes |
| `Q-Trained` | seed 2: the same | 4,096 + 10,368 | yes | yes |

Equal means bit for bit, at every position: the token, every kept cell in order, and all 256 Q12 query values. That is 32,256 positions in all, with no exception.

**The instrument checks:**
- **`I-Build`:** the PTX was admitted. The cubin targets `sm_120`, equal to the device. The pod's own host twin passed the host parity tests (3/3).
- **`I-Inputs`:** all four inputs matched the sheet's pinned sha256s.
- **`I-Finite`:** every step exited 0.
- **`I-Identity`:**
  - PTX `013d19fd71333ee9dddf98edcbbbceaa923ce6165875dbdf24922110b63a4a5f`;
  - cubin `73eccb708f6428b78b23497e532920a164adfa1aba1204a3d5f8b79ffea8dcc3`;
  - binary `f35527fc193489853480ec842a9509e221b25dfc967b05d97ed84b5b58391b34`;
  - artifacts `314c3909…` and `d5b43596…`, recorded in each output.

**Readings, with no bar:**
- **GPU wall time:** the 54 windows of 192 took 14.46 and 14.39 s. The 4,096 single tokens took 0.085 s.
- **ptxas:** 104 registers, a 256-byte stack frame, and no spills.
- **The projection for the full 14,287-row panel.** Its targets lie in 428 DEV windows of 192 (82,176 positions).
  - The pre-registered linear projection is 82,176 × 1.395 ms ≈ 115 s per seed. That number is a projection, not a measured bound.
  - The 428 windows fit in one launch of 428 threads, so the expected wall time is close to one window's serial time, about 15 s. That is an inference, not a measurement.
- **Evidence:** in the HF store at `results/claude/d11-fbs-gpu-evaluator-2026-10-26/`, each folder with a `SHA256SUMS`:
  - `evalq/` holds every step's stdout, stderr and exit code, every output, and `job.log`;
  - `evalq-bin/` holds the binary run.

**Host diagnostic before the pod (laptop, not counted):** the host twin equals the Rust session at all 14,464 positions of each trained artifact.

## Checks run at the head

See the exact-head review record on the PR.

**Hardening after the run.** Round 1 of the code review hardened the CLI and the PTX self-test after the qualification:
- `run` and `windows` refuse empty input;
- `sequences_sha256` hashes the bytes parsed;
- the output carries the ptxas report;
- the self-test probes every forbidden component;
- `first_difference` reports differing dimensions.

`fbs_core.h` and `evaluator.cu` are byte-identical to the run's source `ac47d01d4…`, and the nvcc arguments are unchanged. So, by inference, this head builds the qualified PTX and cubin. Nothing was rebuilt on a GPU at this head. A header comment that the core trusts the loader's bounds was left out, so as not to touch the qualified source.

**Suites:**
- integer serving library 326 and no-feature library 55;
- `fbs_image_matches_its_shape`, now comparing every map's atoms with the snapshot's routes;
- the instrument's host parity 3/3, on the Mac and on the pod.

**Build checks:**
- the stub-nvcc CUDA check and clippy of the instrument are clean;
- `cargo fmt` is clean;
- the `fbs-synthetic` feature builds without `test` and with no warnings.

**Loop checks:** `loop report --check`, `loop pr-check --local` (PASS, with advisory WARNs for the ORDER's size and the sheet's "Cost and stop rule" heading) and `loop gates --full` (exit 0).

## Skills used

- **holding-the-goal.** The goal is the artifact's own gap, so this unit built the evaluator with exact parity as its only bar. It did not stop at a GPU number that merely looks plausible.
- **writing-meaningful-tests.** The parity tests compare the whole output (token, kept cells and every query value) at every position. A truncated norm and a dropped recurrence lane each fail them.

## Still open

- **Score the artifact's own export gap with this evaluator** (the next unit, under an amended Stage 4b sheet). It needs:
  - G's tokens at the 14,287 panel targets against claude-126's stored float predictions, for the top-1 half (b, c and F_I);
  - the MarginHinge loss computed on the GPU from G's Q12 queries, for the loss half.
- **Parallelism within a sequence,** if the panel ever needs to run faster. It does not today.
- **The trained trunk's address collapse** on seed 1, and the residual cause that claude-127 left untested, carry over unchanged.

## Plain summary

The shipped integer file of the trained model can now be run on a GPU exactly as it runs on a laptop.
- **The check:** a new GPU program computes the file's output one token at a time. On every one of about 32,000 positions it gave the same answer, to the last bit, as the file's own laptop runtime.
- **Why it matters:** the next step can measure how much accuracy the export costs, on the GPU, using the real file rather than an imitation of it.
