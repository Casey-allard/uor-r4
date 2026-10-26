# REPORT: m1-export-gap-and-toks, antigravity-96

Lab: antigravity. Order: [ORDER.md](ORDER.md). Sheet: [`m1-export-gap-and-toks-DRAFT-2026-10-23.md`](../../../../harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md). Lab record: [`docs/labs/m1-export-gap-and-toks/README.md`](../../../../labs/m1-export-gap-and-toks/README.md).
Closes #2839.

## What did not work, first

- **Raw `UORLUT01` bit-flip perturbation on `discrete.cells[0]` vs `discrete.routes[0]`:** When wiring the container-level tamper check for planted-fault control `B4` (`src/lib.rs`), I initially expected `flip_container_table_byte(artifact_bytes, "discrete.routes")` (`routes[0] ^= 1`) to be rejected by `IntegerDiscreteModel::from_bytes` in the same way `cells[0] ^= 1` is (`discrete.cells entry disagrees with discrete.table endpoint roots`). Instead, `routes[0] ^ 1` remains a valid action index in `0..64`, so `from_bytes` accepts the mutated container with a changed `artifact_sha256`, whereas `routes[0] ^= 0x40` (`>= 64`) is rejected by `from_bytes` (`discrete.routes contains action index >= 64`). I classified this as an instrument-check distinction and updated `planted_fault_control` to verify all three: `cells[0] ^= 1` rejected by `from_bytes`, `routes[0] ^= 1` changing `artifact_sha256` (failing the container hash pin), and `routes[0] ^= 0x40` rejected by `from_bytes`, alongside the in-memory `cells` and `routes`/`gates` perturbations on the `14,287` DEV rows.
- **Cargo host/target `deps/` collision without `--target`:** Running `cargo test` in `docs/labs/m1-d11-artifact-scorer/instrument` without an explicit `--target` flag fails (`can't find crate for uor_r4_core`) because `uor-r4-integer` builds both `cdylib` and `rlib` while `uor-r4-discrete/build.rs` runs on the host. Passing `--target aarch64-apple-darwin` on macOS and `--target x86_64-unknown-linux-gnu` on the Linux pod (as documented in `claude-116/REPORT.md:72`) resolves it cleanly.
- **Laptop CPU contention during the first `toks` pass:** The initial laptop `toks` run overlapped with a short `rustc` compile in a sibling worktree, depressing `p95` on `t16-relation-seed1` (`56,526 tok/s` prefill `p95`). Re-running `d11-scorer toks` across all four Stage T `.lut` artifacts once no concurrent `cargo`/`rustc` processes were running gave clean, repeatable timings.

## What changed

All code changes are confined to the detached instrument workspace `docs/labs/m1-d11-artifact-scorer/instrument/`, with zero edits to any crate under `crates/`:

- **GPU kernel & FFI (`src/kernel_core.h`, `src/scorer.cu`, `src/host_kernel.cpp`, `src/kernel.rs`, `src/device.rs`):** Extended `D11RowOut` (`156` $\rightarrow$ `161` `uint32_t` words) and `d11_evaluate_row` to compute `token_hinge`, `address_hinge`, `rank_hinge`, `loss_ta` (`(token_hinge << 3) + width * address_hinge` via shift and repeated addition — zero hardware multipliers, zero floats), and `loss_tr` (`(token_hinge << 3) + width * rank_hinge`), matching `IntegerDiscreteModel::observe_target` (`crates/uor-r4-integer/src/discrete_d11.rs:1278-1378`).
- **Reference check, planted-fault control `B4`, and training-eval `gap` comparator (`src/lib.rs`, `src/main.rs`, `tests/parity.rs`):**
  - Extended `compare_row_with_target` (`--reference`) to verify `token_hinge`, `address_hinge`, `rank_hinge`, `loss_ta`, and `loss_tr` on every row against `IntegerDiscreteModel::observe_target`.
  - Added `planted_fault_control` (`B4`) and `compare_with_train_eval` (`--train-eval TRAIN_EVAL_JSON GAP_REPORT_JSON`), plus `d11-scorer gap`.
  - Added `planted_fault_b4_and_train_eval_gap_verify_and_catch_faults` in `tests/parity.rs` and verified negative gates on `token_hinge` and `rank_hinge` in `src/kernel_core.h`.
- **Stage 4 artifacts, lab record, notebook row, and roadmap status:**
  - Updated [`docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md`](../../../../harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md) with the pinned Stage T (`#2836`) SHA-256 digests and recorded the fresh-context FIT & D67 pre-run review (`ac559562-b8ce-4d82-b1d8-d8ae66015505`, [#2839 comment 6422279606](https://github.com/UOR-Foundation/uor-r4/issues/2839#issuecomment-6422279606)).
  - Wrote [`docs/labs/m1-export-gap-and-toks/README.md`](../../../../labs/m1-export-gap-and-toks/README.md) and [`docs/labs/m1-export-gap-and-toks/evidence/`](../../../../labs/m1-export-gap-and-toks/evidence/), added the Stage 4 result row to [`docs/research/notebook/m4-serving.md`](../../../../research/notebook/m4-serving.md), and set `status: done` in [`docs/roadmap/items/m1-export-gap-and-toks.md`](../../../../roadmap/items/m1-export-gap-and-toks.md).

## Measurements (Lead with `tok/s`, CHARTER §3.9)

### 1. Client Laptop Product Serving Throughput (`B3`, CHARTER §3.6)

Measured on Apple M1 (`4` performance + `4` efficiency cores, `16 GiB` RAM, macOS `27.0` `26A5378n`, AC power) over the default deterministic `192`-token prompt (`664c3be9…`), `1` warmup + `20` timed runs each for prompt-reading (`192`-token context prefill) and reply-writing (`64`-token autoregressive decode):

| Arm (`#2836`) | Exported `.lut` SHA-256 (`542,260` B) | Reply-Writing Decode Median (`64` tok) | Reply-Writing Decode `p95` (rank `19/20`) | Prompt-Reading Prefill Median (`192` tok) | Prompt-Reading Prefill `p95` (rank `19/20`) | Reply SHA-256 (`21/21` runs) |
|---|---|---:|---:|---:|---:|---|
| `t16-frozen-zero-seed1` | `fc31bcc7a88250f3…` | `12,630 tok/s` (`5,067 µs`) | `12,092 tok/s` (`5,292 µs`) | `2,395,627 tok/s` (`80 µs`) | `2,262,150 tok/s` (`84 µs`) | `48feb9d729845a77…` |
| `t16-frozen-zero-seed2` | `a75ea7f5907275fd…` | `12,523 tok/s` (`5,110 µs`) | `11,616 tok/s` (`5,509 µs`) | `2,789,359 tok/s` (`68 µs`) | `2,231,494 tok/s` (`86 µs`) | `48feb9d729845a77…` |
| `t16-relation-seed1` | `d64e127c79c1e3be…` | `1,197 tok/s` (`53,432 µs`) | `702 tok/s` (`91,102 µs`) | `373,889 tok/s` (`513 µs`) | `359,550 tok/s` (`534 µs`) | `f3cbe66f56287985…` |
| `t16-relation-seed2` | `c21d992fb4a70b93…` | `1,118 tok/s` (`57,226 µs`) | `861 tok/s` (`74,322 µs`) | `317,815 tok/s` (`604 µs`) | `119,409 tok/s` (`1,607 µs`) | `f3cbe66f56287985…` |

All `20` raw nanosecond samples per arm are tabulated in [`docs/labs/m1-export-gap-and-toks/README.md`](../../../../labs/m1-export-gap-and-toks/README.md) and archived in `caseyallard/uor-r4-store:results/antigravity/m1-export-gap-and-toks-2026-10-26/laptop/`.

### 2. GPU `14,287`-Row DEV Export Gap and Planted-Fault Control (`B2` & `B4`, CHARTER §3.5)

Executed on 1 leased RTX 5090 pod (`vast-57845148`, compute capability `12.0`, `nvcc 12.8.93`, `260` pod-seconds from `17:05:06Z` to `17:09:25Z` at `$1.106/h` $\approx \$0.08$, torn down with `0` pods left running) at commit `9f06e835504415be600934920866bccc8f5b3934`:

| Arm (`#2836`) | `top1` (Train / D11 / $\Delta$) | `top8` (Train / D11 / $\Delta$) | `admitted` (Train / D11 / $\Delta$) | `loss_ta` num (Train / D11 / $\Delta$) | `loss_tr` num (Train / D11 / $\Delta$) | Row Parity (`observe_target` & `final-dev.json`) | Planted-Fault `B4` (`cells` / `gates` mismatches) | GPU Kernel Time |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `t16-frozen-zero-seed1` | `455 / 455 / 0` | `1,484 / 1,484 / 0` | `5,009 / 5,009 / 0` | `12,920,928 / 12,920,928 / 0` | `29,531,840 / 29,531,840 / 0` | `14,287 / 14,287` (`0` refused) | PASS (`574` / `5,436` rows) | `42,766 µs` |
| `t16-frozen-zero-seed2` | `455 / 455 / 0` | `1,484 / 1,484 / 0` | `5,009 / 5,009 / 0` | `12,920,928 / 12,920,928 / 0` | `29,531,840 / 29,531,840 / 0` | `14,287 / 14,287` (`0` refused) | PASS (`574` / `5,436` rows) | `44,790 µs` |
| `t16-relation-seed1` | `371 / 371 / 0` | `1,124 / 1,124 / 0` | `3,827 / 3,827 / 0` | `13,604,128 / 13,604,128 / 0` | `31,562,816 / 31,562,816 / 0` | `14,287 / 14,287` (`0` refused) | PASS (`458` / `7,189` rows) | `366,019 µs` |
| `t16-relation-seed2` | `371 / 371 / 0` | `1,124 / 1,124 / 0` | `3,827 / 3,827 / 0` | `13,604,128 / 13,604,128 / 0` | `31,562,816 / 31,562,816 / 0` | `14,287 / 14,287` (`0` refused) | PASS (`458` / `7,189` rows) | `365,261 µs` |

### 3. Static Zero-Multiply Instruction Audits (`B1`)

- **Native ARM64 (`uor-r4-stack`, SHA-256 `83efb81e5f685a2e58587beda200adf4fbe2d6c0af2fa84df1689b77997088c5`):** `FULL PASS: 0 forbidden instructions across 44 symbol ranges AND 98 reachable call-graph functions` (`Class I Multipliers = 0`, `Class II Dividers = 0`, `Class III Floats = 0`).
- **WebAssembly (`uor_r4_integer_bg.wasm`, `416,816` B, SHA-256 `1332818275dfb9725415a61711e6cf11d80eb421758ae86df1e937c7d8168456`):** `7 / 7` serving symbols PASS (`mul = 0`, `div = 0`, `float = 0`, `unclassified = 0`).

## Checks run at the head

- **GPU test suite & Stage 4 evaluation on `vast-57845148` (`9f06e8355`):** `17 / 17` finite + panel tests pass (`--include-ignored`, including `gpu_scores_are_bit_identical_to_the_reference_and_the_host_build`, `planted_fault_b4_and_train_eval_gap_verify_and_catch_faults`, and `the_registered_dev_panel_has_14287_rows`). Sealed in `caseyallard/uor-r4-store:results/antigravity/m1-export-gap-and-toks-2026-10-26/pod/` (`SHA256SUMS` SHA-256 `4292a3ce60fb219568d5fe5682801a7a8f3d953a1dbd29d2d7e7d0879718a17a`).
- **Host tests, negative gates, clippy, fmt, and harness gates:** `16` host unit tests pass (`1` ignored off-pod); setting aside `token_hinge` or `rank_hinge` in `src/kernel_core.h` fails `score_totals_count_the_reference_predictions` and passes on byte-identical restore; `cargo fmt --check`, `cargo clippy` (with and without `cuda`), `xtask roadmap check`, `loop report --check`, `loop pr-check --local`, and `loop gates --full` pass.
- **Fresh-context reviews:** FIT + D67 pre-run review (`ac559562-b8ce-4d82-b1d8-d8ae66015505`, `GO`), plus fresh-context D66 exact-head code and claims reviews.

## Skills used

- `uor-research-architect`: enforced exact integer parity across training-side CUDA `eval-corpus`, GPU `d11-scorer`, and `IntegerDiscreteModel::observe_target`.
- `scope-guard`: kept all code changes inside `docs/labs/m1-d11-artifact-scorer/instrument/` with zero edits to `crates/**`.
- `solution-integrity`: verified planted-fault control `B4` and negative-gate perturbations before launching the GPU pod.

## Still open

- **Step 7 (`m1-fresh-panel-transfer`, `#2841`) and Step 8 (`m1-stage-5-outcome-table`, `#2842` / `#2837`):** With Stage 4 (`#2839`) complete and `done`, the uncontaminated fresh transfer slice (`4bd108ad…`) and Stage 5 outcome table are unblocked on the M1 path.
- **Mechanism board status note (`docs/mechanisms/lut-serving.md`):** Updating `docs/mechanisms/lut-serving.md` can be bundled in a follow-up `infra-loop-harness` unit when the umbrella lock is free.

## Plain summary

Step 6 of the M1 plan (`#2839`) measures how accurately and how fast a model trained under today's geometric rules runs after being exported to the integer lookup-table file (`UORLUT01`). First, I wired the integer loss terms (`token_hinge`, `address_hinge`, `rank_hinge`, `loss_ta`, `loss_tr`), the planted-fault control (`B4`), and the training-vs-export `gap` check into the GPU artifact scorer (`d11-scorer`). Then I exported all four saved Stage T checkpoints from `#2836` (`t16-frozen-zero-seed{1,2}` and `t16-relation-seed{1,2}`), verified that the native and WebAssembly serving binaries have zero floating-point, multiply, or divide instructions (`B1`), and scored the `14,287`-row DEV panel on a rented RTX 5090 (`~$0.08`). Every single row and aggregate metric matched the training-side evaluator bit for bit (`Δtop1 = 0`, `Δtop8 = 0`, `Δadm = 0`, `Δloss = 0`, `14,287 / 14,287` rows), while planted faults broke the scores as required (`B4`). Finally, on the Apple M1 laptop (`B3`), the exported `FrozenZero` integer artifacts read prompts at `2.40M–2.79M tok/s` median and wrote replies at `12,630` and `12,523 tok/s` median (`Relation` artifacts: `373,889` and `317,815 tok/s` prompt-reading, `1,197` and `1,118 tok/s` reply-writing), closing `#2839`.

