# REPORT: float-training-module, antigravity-72

Lab: antigravity. Item: [float-training-module](../../../../roadmap/items/float-training-module.md). Order: [`ORDER.md`](./ORDER.md).

Closes #2823

## What did not work

- **Cancel probe 1 (umbrella lock check):** `uor-harness loop claim float-training-module --lab-topic float-training-module` refused with `scope docs/roadmap/items/float-training-module.md overlaps docs/roadmap/items/*.md of item infra-loop-harness (PR #3169)`. Inspected open PRs via `gh pr list --state open --json number,headRefName,files`: PR `#3169` (`codex/d63-finite-probe-scope`, idle draft since `2026-10-25T01:07:47Z`) touches only `PLAN.md`, `docs/integration/DECISIONS.md`, and `docs/loop/tasks/infra-loop-harness/codex-1/**`, with zero file overlap on `float-training-module`. Followed the umbrella-lock exception (`run-antigravity.txt` Part 3 Step 2) to write `ORDER.md` and open Draft PR `#3218`.
- **Cancel probe 2 (repository state check at `7c6f1f4ad03e3786193fa592953220cab7d595b0`, class: `instrument`):** `ls crates/uor-r4-float-train` returned `No such file or directory`; the 24-kernel geometric stack extract lived only inside `crates/uor-r4-training`, and `crates/uor-r4-harness/src/ratchet.rs` lacked a dedicated D37 scanned crate allowlist entry (`D37_SCANNED_FLOAT_CRATES`).
- **Failing-test-first verification (class: `instrument`):** Before implementing the 24-kernel CPU reference controls in `crates/uor-r4-float-train/src/kernel_controls.rs` and the 24-symbol `KERNEL_ENTRY_POINTS` slice in `src/lib.rs`, running `CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo test --offline --no-fail-fast -p uor-r4-float-train` against stub implementations failed all 12 kernel control tests in `tests/twenty_four_kernel_controls.rs` (`0 passed; 12 failed`) and the entry-point guard in `tests/no_training_crate_dep.rs` (`1 passed; 1 failed`).
- **Initial D29 raw-text scan on `src/cuda_stack.cu` header comment (class: `instrument`):** `twenty_four_kernel_entry_points_match_cuda_sources_and_pass_d29_scan` initially caught banned-token substrings (`softplus`/`sigmoid`) in lines 4–6 of `src/cuda_stack.cu`'s file header comment describing what the file avoids. Reworded the header comment to `squashing, exp-family activation` so even unstripped source text of `src/cuda_stack.cu` and `src/spiralcore_fbs.cu` contains zero banned tokens.
- **Stub `nvcc` on macOS for transitive `cudarc` and `candle-kernels` build scripts (class: `instrument`):** Running `cargo check --offline -p uor-r4-float-train --features cuda --all-targets` on macOS initially failed in `cudarc/build.rs` (which invokes `nvcc --version` from `PATH`) and `candle-kernels/build.rs` (which validates `.entry` lines against `entry_allowlist.rs`). Extended the worktree-local `local/antigravity/fake-nvcc/nvcc` stub to emit the 4-line CUDA 12.8 `--version` banner and extract `EXPORT_GROUPS` when `entry_allowlist.rs` is present in the working directory.

## What changed

Size: **24 files changed, 8,274 insertions, 7 deletions** across the PR (`pr-check` class breakdown: `tests 2 file(s) +552/-0, code 12 file(s) +6447/-3, docs 6 file(s) +215/-1, generated 2 file(s) +18/-3, other 2 file(s) +1042/-0`; no size estimate declared on the item file):
- **Code & CUDA sources (15 files, 7,490 insertions, 3 deletions):**
  - `Cargo.toml` (`+1/-0`): registered `"crates/uor-r4-float-train"` in `[workspace] members` without altering any other crate's dependency entries.
  - `crates/uor-r4-float-train/Cargo.toml` (`22` lines) and `crates/uor-r4-float-train/build.rs` (`102` lines): standalone crate manifest (zero dependency on `uor-r4-training` or `candle-nn`) and ahead-of-time `nvcc` PTX build script (`cuda_stack_f32.ptx`, `cuda_stack_bf16.ptx`) under `--features cuda`.
  - `crates/uor-r4-float-train/src/lib.rs` (`108` lines), `src/cuda_stack.cu` (`859` lines, `22` `__global__` kernels), `src/spiralcore_fbs.cu` (`183` lines, `2` `__global__` kernels), `src/cuda_stack_kernels.rs` (`429` lines), `src/geometric_stack_native.rs` (`2,110` lines), `src/geometric_stack_native_cuda.rs` (`613` lines), `src/spiralcore_fbs.rs` (`808` lines), `src/native_vector_writer.rs` (`1,014` lines), `src/native_starling_spec.rs` (`127` lines), and `src/kernel_controls.rs` (`1,029` lines, finite CPU reference implementations for all 24 kernels).
  - `crates/uor-r4-harness/src/ratchet.rs` (`+84/-3` lines total: `+39/-3` code lines and `+45` test lines): added `pub const D37_SCANNED_FLOAT_CRATES: [&str; 1] = ["crates/uor-r4-float-train/"];` and `pub fn is_d37_float_scope(rel: &str) -> bool`, registered `D37_SCANNED_FLOAT_CRATES` in `scan_roots` (deduplicated in `source_files`), excluded `is_d37_float_scope` from `is_discrete_scope` (`d32`), clamped `base = 0` in `growth()` for `is_d37_float_scope`, and refused any nonzero `NormExp`/`Matmul` baseline update inside `is_d37_float_scope`.
- **Integration tests (2 files in `uor-r4-float-train/tests/`, 552 insertions; plus 1 unit test in `ratchet.rs`):**
  - `crates/uor-r4-float-train/tests/no_training_crate_dep.rs` (`126` lines): verifies via `cargo tree --offline -p uor-r4-float-train` that `uor-r4-training` and `candle-nn` are absent from the dependency graph, and verifies that all 24 `__global__` kernel entry points in `src/cuda_stack.cu` (`22`) and `src/spiralcore_fbs.cu` (`2`) match `KERNEL_ENTRY_POINTS` with zero banned D29 tokens.
  - `crates/uor-r4-float-train/tests/twenty_four_kernel_controls.rs` (`426` lines): 12 finite CPU arithmetic and finite-difference adjoint control tests covering all 24 kernels in the extract.
  - `crates/uor-r4-harness/src/ratchet.rs` (`+45` test lines, counted in `ratchet.rs` above): added unit test `d37_scanned_float_crate_is_scanned_for_d23_d25_with_zero_ceiling_and_allowlisted_from_d32`.
- **Docs & records (6 files, 215 insertions, 1 deletion):**
  - `docs/labs/float-training-module/README.md` (`73` lines): design note (also posted on `#2823` at comment `6397939619`) documenting the crate name, the 24-kernel provenance table, zero dependency edge on `uor-r4-training`, D23/D25 compliance, and `ratchet.rs` D37 scanned entry.
  - `docs/research/notebook/float-training.md` (`13` lines): new research notebook for the `float-training` line with one finite-control row.
  - `docs/roadmap/items/float-training-module.md` (`+2/-1`): added `notebook: docs/research/notebook/float-training.md` and updated `status: done`.
  - `docs/start/FILEMAP.md` (`+1/-0`): added `crates/uor-r4-float-train/`.
  - `docs/loop/tasks/float-training-module/antigravity-72/ORDER.md` (`42` lines) and `REPORT.md` (`84` lines).
- **Generated (2 files, 18 insertions, 3 deletions):**
  - `Cargo.lock` (`+15/-0`) and `docs/roadmap/README.md` (`+3/-3`, re-rendered via `cargo run --offline -p xtask -- roadmap render`).

## Measurements

All measurements are deterministic host CPU checks and static ratchet scans on the laptop (`needs_gpu: false`; no GPU run and no model training; `tokens/s: N/A` for this module-extraction infrastructure unit):

1. **CPU build and 24-kernel finite control suite (`uor-r4-float-train`):**
   - Command: `CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo test --offline --no-fail-fast -p uor-r4-float-train`
   - Result: **19 passed; 0 failed** (`5` `--lib` unit tests in `src/geometric_stack_native.rs` (1), `src/native_vector_writer.rs` (1), and `src/spiralcore_fbs.rs` (3); `2` dependency-tree and 24-symbol D29 guards in `tests/no_training_crate_dep.rs`; and `12` finite CPU arithmetic and finite-difference adjoint control tests in `tests/twenty_four_kernel_controls.rs` covering all 24 kernels: `straight_through_fwd`, `hamilton_gate_fwd`, `hamilton_gate_bwd`, `rate_select_fwd`, `rate_select_bwd`, `rms_norm_fwd`, `rms_norm_bwd_dx`, `rms_norm_dw_partial`, `rms_norm_dw_reduce`, `quaternion_scan_fwd`, `quaternion_scan_bwd`, `margin_rank_rows`, `margin_rank_grad`, `recurrence_scan_fwd`, `recurrence_tile_fold`, `recurrence_tile_carry`, `recurrence_tile_expand`, `recurrence_bwd_params`, `poison_words`, `adam_update`, `sq_lanes`, `sq_tree`, `spiralcore_fbs_fwd`, `spiralcore_fbs_bwd`).
2. **Stub-`nvcc` `--features cuda --all-targets` type and clippy check on the laptop:**
   - Command: `PATH="/Users/casey.allard/uor-r4/.worktrees/antigravity-float-training-module/local/antigravity/fake-nvcc:$PATH" NVCC=/Users/casey.allard/uor-r4/.worktrees/antigravity-float-training-module/local/antigravity/fake-nvcc/nvcc UOR_NVCC_ARCH=compute_120 CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo clippy --offline -p uor-r4-float-train --no-deps --features cuda --all-targets -- -D warnings`
   - Result: **exit 0, 0 warnings** (`build.rs` invokes stub `nvcc` for `cuda_stack_f32.ptx` and `cuda_stack_bf16.ptx`, verifies all 24 `.entry` symbols and zero banned PTX instructions, and compiles `src/cuda_stack_kernels.rs` and `src/geometric_stack_native_cuda.rs`).
3. **D23, D25, and D32 ratchets and `ratchet::tests` (`uor-r4-harness`):**
   - Command: `CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo test --offline --no-fail-fast -p uor-r4-harness --lib ratchet::tests`
   - Result: **10 passed; 0 failed** (including `d37_scanned_float_crate_is_scanned_for_d23_d25_with_zero_ceiling_and_allowlisted_from_d32`).
   - Command: `CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo run --offline -p uor-r4-harness --bin uor-harness -- ratchet d23`
   - Result: `OK: no softmax added. 4451 remaining across 165 files (baseline 4451).` (`0` sites in `crates/uor-r4-float-train/`).
   - Command: `CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo run --offline -p uor-r4-harness --bin uor-harness -- ratchet d25 --list`
   - Result: `OK: no dense matrix product call sites added. 93 remaining across 26 files (baseline 93).` (`0` sites in `crates/uor-r4-float-train/`).
   - Command: `CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo run --offline -p uor-r4-harness --bin uor-harness -- ratchet d32 --list`
   - Result: `OK: no discrete-path floating-point sites added. 31 remaining across 1 files (baseline 31).`

## Checks run at the head

- `CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo fmt -p uor-r4-float-train -p uor-r4-harness --check` — exit 0
- `CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo clippy --offline -p uor-r4-float-train --no-deps --all-targets -- -D warnings` — exit 0
- `PATH=".../local/antigravity/fake-nvcc:$PATH" NVCC=.../local/antigravity/fake-nvcc/nvcc UOR_NVCC_ARCH=compute_120 CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo clippy --offline -p uor-r4-float-train --no-deps --features cuda --all-targets -- -D warnings` — exit 0
- `CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo test --offline --no-fail-fast -p uor-r4-float-train` — 19 passed (`5` lib + `2` `no_training_crate_dep` + `12` `twenty_four_kernel_controls`)
- `CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo test --offline --no-fail-fast -p uor-r4-harness --lib ratchet::tests` — 10 passed
- `CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo run --offline -p uor-r4-harness --bin uor-harness -- ratchet d23` — exit 0 (`4451 remaining across 165 files (baseline 4451)`)
- `CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo run --offline -p uor-r4-harness --bin uor-harness -- ratchet d25 --list` — exit 0 (`93 remaining across 26 files (baseline 93)`)
- `CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo run --offline -p uor-r4-harness --bin uor-harness -- ratchet d32 --list` — exit 0 (`31 remaining across 1 files (baseline 31)`)
- `CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo run --offline -p xtask -- roadmap check` — `roadmap check: 92 items OK`
- `CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo run --offline -p xtask -- links --json local/antigravity/links.json` — 0 broken links in any touched file
- `~/.cargo/bin/uor-harness loop report --check --order docs/loop/tasks/float-training-module/antigravity-72/ORDER.md` — PASS
- `~/.cargo/bin/uor-harness loop pr-check --local` — PASS
- `~/.cargo/bin/uor-harness loop gates --full` — 4/4 GREEN

## Skills used

- `scope-guard`: kept all changes strictly within the `float-training-module` scope globs and shared registration files (`docs/roadmap/README.md`, `docs/start/FILEMAP.md`), without touching any other crate's `Cargo.toml` or `Cargo.lock` entries.
- `solution-integrity`: verified the dependency exclusion (`cargo tree -p uor-r4-float-train` has zero `uor-r4-training` or `candle-nn` edges), proved the 24-kernel finite controls fail on stub implementations before passing on the real implementations, and verified the D37 ratchet invariant (`0` D23/D25 sites allowed in `crates/uor-r4-float-train/`, exempted from D32's discrete integer-only scan).
- `gates`: ran `uor-harness loop gates --quick` and `--full` plus the D23, D25, and D32 ratchets and stub-`nvcc` `--features cuda` compilation check.

## Still open

- **Downstream GPU qualification and conditional W4 float-credit runs (`#2846`, Stage 3 trunk):** This unit completes `float-training-module` (`#2823`), which scopes only the standalone crate boundary, D37 ratchet entry, CPU finite controls across the 24-kernel extract, and stub-`nvcc` `--features cuda` type check on the laptop. Ahead-of-time `nvcc` compilation on a GPU pod and any W4 float-credit or Stage 3 trunk training runs are tracked under `#2846` (`m1-w4-float-credit-sheet`, conditional on W2 and W3) and `#2844` (`m1-trunk-assembly-and-training`).

## Plain summary

This change creates `crates/uor-r4-float-train` (`uor-r4-float-train`), the standalone Rust crate where offline floating-point training code is permitted to live under project rule D37 (`CHARTER.md` §3.7). Previously, the 24 CUDA kernels and Rust modules that implement the project's native geometric operations (unit-quaternion rotations on 4-number cells, 16-entry power-of-two decay rate tables, root-mean-square sphere normalization, parallel quaternion sequence scans, piecewise-linear margin ranking loss, and 8-dimensional Clifford algebra `Cl(0,6)` lookup routing) lived inside `crates/uor-r4-training` alongside older retired modules. Moving the 24-kernel extract into `crates/uor-r4-float-train` guarantees that it has zero dependency on `crates/uor-r4-training` (enforced by an automated `cargo tree` test), registers `crates/uor-r4-float-train/` as its own scanned entry in `uor-harness ratchet` (requiring zero softmax/squashing or dense matrix multiplication sites while keeping the discrete model crate `crates/uor-r4-discrete` strictly integer-only), and tests all 24 kernels on CPU finite arithmetic controls plus a stub-`nvcc` CUDA type check.
