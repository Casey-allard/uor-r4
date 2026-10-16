# Standalone `C-R1..C-R6` GPU Fixture Runner (`uor-r4.admission-rank-fixture-runner/1`)

**Latest restart (4), 16 October:** [merged `5fc68a9a6`](../k16-qualification-restart4-2026-10-16/README.md)
includes the approved C-R6 consumer repair and passes the fixture CUDA release
build. Its mandatory clippy gate fails at `src/main.rs:564` on `3 * 4 + 0`
(`clippy::identity_op`, `-D warnings`). Execution halted with no patch/retry;
all six GPU controls remain NOT_RUN. Repair/review/merge is required before
another full step-0 qualification attempt, not a mechanism rejection.

**Earlier restart, 16 October:** the identity-access repair passed a pod CUDA check
and merged in #2533, but [source preflight](../k16-qualification-restart-2026-10-16/README.md)
found a separate C-R6 receipt mismatch: it reads `evaluation` instead of the
driver's top-level `aggregate`. All fixtures remain NOT_RUN; no receipts were
transformed and no model compute occurred on the restart lease. A reviewed
schema repair precedes the next qualification attempt.

**16 October qualification attempt:** the [pod-side CUDA build](../k16-qualification-2026-10-16/README.md)
at `3c651df97` failed with E0609 in `cuda_aggregate.rs:93/96` and
`main.rs:168–170` (`compiled_identity()` returns JSON). Step 0 halted before
clippy or any fixture/model execution. All six controls remain NOT_RUN;
source repair and exact-head review are required before another qualification
attempt. The original source-only delivery account below is retained.

- **Date:** 2026-10-15
- **Lab:** Antigravity (`antigravity`)
- **Milestone / Track:** `#2029` (Milestone 1 — Grounded Conversation & Durable Memory) / Rung 0 Stage A Admission-Rank Redirect (`C-R1..C-R6` GPU Qualification Executor)
- **Governing Decisions:** `D23` (no softmax), `D24` (no CPU model compute), `D25` (no dense matrix multiplication), `D32` (discrete integer substrate; zero floating-point in model/instrument paths), `D35` (shared evaluation surface)
- **Governing Pre-Registration & Sources:**
  - FIT-reviewed sheet: [`docs/harness/prep/m1-admission-rank-redirect-2026-10-15.md`](../../harness/prep/m1-admission-rank-redirect-2026-10-15.md) (`TEST FITNESS: FIT`)
  - Control scaffold: [`crates/uor-r4-discrete/src/objective.rs:53-128`](../../../crates/uor-r4-discrete/src/objective.rs) (`CONTROL_SCAFFOLD` / `control_scaffold()`)
  - Redirect qualification gate: [`crates/uor-r4-discrete/src/redirect_admission.rs:18-35, 357-367`](../../../crates/uor-r4-discrete/src/redirect_admission.rs) (`uor-r4.redirect-qualification/1`)
- **Execution Status:** **`NOT_RUN` / `UNMEASURED` on GPU** — source-only pre-lease instrument delivery. Zero pods leased, zero GPU model compute executed, zero `.apply` updates (`apply_calls: 0`). Both the global training hold and the Stage-B / M2 holds remain **HELD**.

---

## 1. Purpose & Non-Interference Architecture

Per [`crates/uor-r4-discrete/src/objective.rs:103, 124`](../../../crates/uor-r4-discrete/src/objective.rs), [`crates/uor-r4-discrete/src/redirect_admission.rs:18, 357-367`](../../../crates/uor-r4-discrete/src/redirect_admission.rs), and [#2029 comment 6201569519](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6201569519):

1. `objective::control_scaffold()` in `crates/uor-r4-discrete/src/objective.rs` defines the static `C-R1..C-R6` control contract (`status: "NOT_RUN"`, `C-R3 status: "GPU_FIXTURE_EXECUTOR_OWED"`, `C-R6 status: "GPU_CORPUS_QUALIFICATION_OWED"`).
2. `validate-redirect-qualification` (`redirect_admission.rs:357-367`) requires six ordered `ControlReceipt` entries (`C-R1..C-R6`) in `qualification.json`, each with `status == "PASS"` and a non-empty digest-verified `FilePin` (`c-r1/report.json .. c-r6/report.json`).
3. Following the exact standalone dated-record pattern of [`docs/labs/k16-timing-2026-10-15/instrument/`](../k16-timing-2026-10-15/instrument/Cargo.toml) (`#2524`) and [`docs/labs/fresh-scoring-eval-2026-10-15/instrument/`](../fresh-scoring-eval-2026-10-15/instrument/Cargo.toml) (`#2523`), [`instrument/Cargo.toml`](instrument/Cargo.toml) declares a detached `[workspace]` with a path dependency on `../../../../crates/uor-r4-discrete` so `crates/uor-r4-discrete/` remains **100% untouched**:
   - `training::driver_source_sha256()`: `ed4655a22dbec24316cfe5df16a43407860651ce6aa7a690ee8fcd5bedaf05d8`
   - `crates/uor-r4-discrete/src/discrete.cu`: `f679f8736500925014ff721ab035f3c284446baa6af0763f27b09bc388763008`
   - `crates/uor-r4-discrete/src/objective.rs`: `b813e855be6c4ea460c8711493527d52f1caf1a02bb09b046145f88c7f4ee4d0`

---

## 2. Subcommands & Mandatory Step-0 Pod CUDA Build/Clippy Gate

### 2.1 Host Subcommands (No `--features cuda`, Zero Model Compute)

- `cargo run --locked --offline --manifest-path docs/labs/admission-rank-fixture-runner-2026-10-15/instrument/Cargo.toml -- plan`
  - Emits the 6-control `uor-r4.admission-rank-fixture-runner/1` plan (`status: "NOT_RUN"`, `model_compute_executed: false`, `apply_calls: 0`) aligned with `uor_r4_discrete::objective::control_scaffold()`.
- `cargo run --locked --offline --manifest-path docs/labs/admission-rank-fixture-runner-2026-10-15/instrument/Cargo.toml -- check`
  - Verifies on host (`cuda_context_created: false`, `model_compute_executed: false`, `apply_calls: 0`):
    1. All 6 ordered `CONTROL_SCAFFOLD` IDs and names (`C-R1..C-R6`),
    2. Objective ABI constants (`ROW_WORDS = 14`, `HISTOGRAM_START = 29`, `HISTOGRAM_BINS = [641, 73, 256]`, `QUANTILE_START = 999`, `AGGREGATE_WORDS = 1020`),
    3. All `19` `Spec` / `ReadSpec` configurations used across `C-R1..C-R6` against `Spec::validate`,
    4. All `16` static E8 anchor-orbit roots (`ROOTS`) and 16-candidate generator root-image tables (`ROOT{0,1,2,3,15}_CANDIDATE_ROOTS`),
    5. The `C-R6` stored baseline guard (`admitted_cells in 1..=16`) and the sealed 36-point Stage-A $K$-sweep (`docs/labs/completion-k-sweep-2026-10-15/stage-a-pilot-seed1-k8-16/report.json`, SHA-256 `cc81ca6f76f3a38ac61ffe1d4665a1442d1e42961f4451691327b91b438f472c`).

### 2.2 Mandatory Step-0 Pod-Side CUDA Build/Clippy Gate & GPU Execution (`--features cuda`)

When the owner releases a qualification pod lease, before executing `run`:

1. **Step-0 CUDA compile & clippy gate (`UOR_NVCC_ARCH=compute_120`):**
   ```bash
   UOR_NVCC_ARCH=compute_120 cargo build --release --locked --offline --features cuda \
     --manifest-path docs/labs/admission-rank-fixture-runner-2026-10-15/instrument/Cargo.toml
   UOR_NVCC_ARCH=compute_120 cargo clippy --locked --offline --features cuda --all-targets \
     --manifest-path docs/labs/admission-rank-fixture-runner-2026-10-15/instrument/Cargo.toml -- -D warnings
   ```
   If either command exits non-zero, **stop and report immediately**.
2. **GPU `C-R1..C-R6` execution (after `G-Pre` zero-update corpus evaluation completes into `${OUT_DIR}`):**
   ```bash
   ./docs/labs/admission-rank-fixture-runner-2026-10-15/instrument/target/release/admission-rank-fixture-runner \
     run "${OUT_DIR}/c-r-fixtures" --g-pre-dir "${OUT_DIR}"
   ```
   Atomically writes (`create_new`, refusing any existing target directory/file):
   - `${OUT_DIR}/c-r-fixtures/c-r1/report.json`
   - `${OUT_DIR}/c-r-fixtures/c-r2/report.json`
   - `${OUT_DIR}/c-r-fixtures/c-r3/report.json`
   - `${OUT_DIR}/c-r-fixtures/c-r4/report.json`
   - `${OUT_DIR}/c-r-fixtures/c-r5/report.json`
   - `${OUT_DIR}/c-r-fixtures/c-r6/report.json`
   - `${OUT_DIR}/c-r-fixtures/c-r-suite-summary.json` (containing the ready-to-embed `controls: Vec<ControlReceipt>` with `id`, `status: "PASS"`, and `receipt: FilePin { path, sha256 }` for `uor-r4.redirect-qualification/1`).

---

## 3. Control-by-Control GPU Execution Contract (`0` `.apply` Calls, `0` CPU Model Reference)

| Control | GPU Implementation in [`instrument/src/main.rs`](instrument/src/main.rs) & [`instrument/src/cuda_aggregate.rs`](instrument/src/cuda_aggregate.rs) | Verified Analytic / Stored Invariants |
|---|---|---|
| **`C-R1`** `rank_hinge_exact_admission_equivalence` | Runs `Device::from_snapshot` + `Device::carry` + `Device::evaluate` across $K = 1..=16$ on both **Variant 1A-i** (mixed distances `[(0,0),(4,2),(4,15),(4,32),(8,255)]`) and **Variant 1A-ii** (5-way equal-distance tie `[(16,0),(16,1),(16,15),(16,16),(16,255)]` via `routes = [0, 42, 0, 42, 0, 42, 0, 42]`, `objective.rs:87`), plus **Fixture 1B** (all `256` cells occupied at $K = 16$, target cell `255`, `objective.rs:90`). | `occupied_cells == 5`, `target_cell_rank == 3`, `cell_rank_hinge == [2,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0]`, `target_admitted == [false,false,true,...,true]`; 256-cell boundary yields `occupied_cells == 256`, `target_cell_rank == 256`, `cell_rank_hinge == 240`, `weighted_cell_rank_hinge_sum == 7680`. |
| **`C-R2`** `single_count_per_occupied_cell_invariance` | **Part 2A:** Crowds rival `cell 1` (`objective.rs:95`) with $m \in \{1, 2, 4\}$ tokens at $K \in \{1, 2, 3\}$. **Part 2B:** Moves `token 4` from `cell 255` into newly occupied preceding `cell 2` (`objective.rs:97`) across $K \in \{1, 2, 3, 4\}$. | Part 2A: `target_cell_rank == [3, 3, 3]`, `cell_rank_hinge` invariant (`[2,2,2]` at $K=1$, `[1,1,1]` at $K=2$, `[0,0,0]` at $K=3$) while `admitted_token_count` grows `[3, 4, 6]`. Part 2B: `occupied_cells` increases `5 -> 6`, `target_cell_rank` increases `3 -> 4`, and `cell_rank_hinge` across $K \in \{1, 2, 3, 4\}$ shifts from `[2, 1, 0, 0]` to `[3, 2, 1, 0]`. |
| **`C-R3`** `virtual_vs_applied_candidate_identity_on_rank_and_occupancy` | Tests all 5 mutation classes (`create_cell`, `empty_cell`, `move_target_cell`, `interior_block_only`, `identity_candidate`, `objective.rs:100`) across both `NearestAddressMarginV1` and `OccupiedCellRankHingeV1` and all `16` candidate ordinals (`160` GPU candidate evaluations). Compares `Device::propose` `candidate_losses()[c]` against an independently restored candidate `Snapshot` admitted via `Device::from_snapshot` and scored via `Device::evaluate` on GPU (`0` `.apply` calls). | `proposal.candidate_losses()[c] == cand_eval.aggregate.objective_loss_numerator_sum` for all `160` evaluations; `create_cell` observes `5 -> 6` occupied cells; `empty_cell` observes `5 -> 4` occupied cells; `move_target_cell` changes `target_cell_rank`; `interior_block_only` keeps `occupied_cells`, `target_cell_rank`, `cell_rank_hinge`, and `address_hinge` 100% invariant; `c = 0` matches all 8 `compare_fields`. |
| **`C-R4`** `absent_rival_and_occupancy_under_k_bounds` | **Part 4A:** Runs $M = 1$ and $M = 16$ occupied cells at $K = 16, V = 16$. **Part 4B:** Runs `c_r4_absent_rival_table` at $K = 1$ under both policies for a 1-row wholly ineligible batch (`tr == 0`) and a 2-row mixed batch (`targets = [0, 1]`). | Part 4A: `cell_rank_hinge == 0` and `target_admitted == true` whenever `occupied_cells <= K`. Part 4B: wholly ineligible batch refuses with CUDA status `8` under both policies; mixed batch skips the `tr == 0` row identically under both policies. |
| **`C-R5`** `bounded_integer_arithmetic_dual_loss_and_distribution_schema` | **Part 5A:** Runs multi-row `Device::evaluate` + `Device::aggregate_evaluations` under both policies and compares against single-pass evaluation, verifying `verify_hinge_distribution` across all three distributions. **Part 5B ([`cuda_aggregate.rs`](instrument/src/cuda_aggregate.rs)):** Verifies `sha256(discrete.ptx) == compiled_identity().kernel_sha256` and launches `discrete_evaluation_aggregate` on GPU across all 11 out-of-bounds and `u64` overflow inputs (`T=641`, `A=73`, `R=256`, `rank=0/257`, `occ=0/257`, `adm=0/17`, `loss_ta/loss_tr` mismatch, cross-row `occ` mismatch, `"u64::MAX plus 1"`, `"weighted R overflow"`, `"quantile threshold overflow"`). | Part 5A: All dual-loss sums, `ExactRatio` shares (`address_loss_share_ta`, `rank_loss_share_tr`), `mean_admitted_token_count`, `[641, 73, 256]` histograms ($\sum\text{bins} == \text{scored\_rows}$, `zero_rows == bins[0]`, `mean`), and `min/p25/p50/p75/p90/p99/max` nearest-rank quantiles match exact integer identities. Part 5B: Valid baseline row returns `status == 0`; all 11 out-of-bounds/overflow cases return CUDA `status == 12`. |
| **`C-R6`** `k16_baseline_guard_and_untrained_ceiling_check` | **Part 6A:** Verifies `1..=16` guard and sealed 36-point Stage-A $K$-sweep (`cc81ca6f...`). **Part 6B:** Initializes `Device::new_with_objective` at `V4096/B4/C192` across `(seed, K) in {(1,8),(1,16),(2,8),(2,16)}`. **Part 6C:** Verifies the `G-Pre` zero-update corpus evaluation receipts in `--g-pre-dir`. | Checkpoint-0 `V4096/B4/C192` has `occupied_cells == 16` and `100%` admission at $K = 16$; `G-Pre` `seed = 1, K = 8` matches `5680 / 17 / 173 / 105 / 261824 / 16`; `seed in {1, 2}, K = 16` satisfies `target_admitted_rows >= 4894` with `0` updates. |
