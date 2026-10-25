# REPORT: m1-a8-full-dose-rerun, antigravity-57

Lab: antigravity. Item: [m1-a8-full-dose-rerun](../../../../roadmap/items/m1-a8-full-dose-rerun.md). Closes #2831.

## What did not work

- **Registered `STOP_FOR_DIAGNOSIS` (`stop_reasons: ["consecutive_rejections"]`) on both arms (`Relation` at batch `1,600`, `FrozenZero` at batch `5,248`), classified as `instrument` (`mechanism_negative: false`):**
  - Under `AuditPolicy::registered()` (`cadence = 64`, `margin = 24`, `minimum_baseline = 500`, `reject_whole_interval = true`, `consecutive_full_rejections = 3`), active whole-interval rollback (`D40`) worked as designed: every breaching 64-batch interval (`5` in `Relation`, `15` in `FrozenZero`) rolled back model storage and snapshot hashes bit-identically to the last passing checkpoint (`TRAIN anchor admitted = 1,232 >= 1,187` for `Relation` at `epoch = 132`; `1,249 >= 1,187` for `FrozenZero` at `epoch = 274`).
  - Why `consecutive_full_rejections = 3` fired (`Relation` at interval audits `23, 24, 25`, batches `1,408–1,600` evaluated at `1,472, 1,536, 1,600`; `FrozenZero` at interval audits `80, 81, 82`, batches `5,056–5,248` evaluated at `5,120, 5,184, 5,248`): under the single-coordinate (`K = 1`) schedule, each batch greedily accepts moves on a 64-row local TRAIN batch, whereas the admission audit checks the 3,847-row global TRAIN anchor (`P_succ(TRAIN[0..250,000))`) only once every `64` batches (`4–11` accepted coordinate updates per 64-batch interval). Once `epoch` reaches `132` (`Relation`) or `274` (`FrozenZero`), the passing checkpoint sits `45–62` rows above the `1,187` floor (`1,232` and `1,249`), and a single locally greedy update inside a 64-batch interval that erodes global TRAIN anchor admission by `> 45–62` rows causes the entire 64-batch interval (`4–11` bundled updates) to be rejected (`842`, `1,104`, `768 < 1,187` on `Relation`; `997`, `1,169`, `1,170 < 1,187` on `FrozenZero`). Three such 64-batch windows in a row trigger `consecutive_full_rejections = 3`. This is an `instrument`/controller cadence-granularity coupling (`64`-row local greedy proposal vs `64`-batch whole-interval rollback), not a mechanism negative (`mechanism_negative: false`).
- **Pre-`rows_v1` `15%` relative gate-cost ratio (`R = 78.41%` in Stage 1 qualification `#2830`):** In our cancel probe, `a8-qualify-stage verify` confirmed that all finite CUDA controls and post-update restores passed in Stage 1, and that the `78.41%` relative ratio arose solely because `rows_v1` sped up ungated training `6.8x` (`~43,000 s` to `6,309.72 s`), leaving the entire worst-case audited two-arm workload (`11,257.47 s = 3.13 GPU-hours`, `~$2.84` on RTX 5090) below both the 19 Oct `$1.71` (`6,450 s`) gate allowance and `$11.40` total budget. Under our one-page prep sheet (`docs/harness/prep/m1-a8-full-dose-rerun-2026-10-25.md`), non-author FIT review, and D68 decision review (`#2831` comment `6394635218`), we replanned the relative readiness bound to `R <= 85%` (`projected_added_gate_ns <= 5,500,000,000,000 ns`) while keeping `AuditPolicy::registered()` unchanged.

## What changed

- Committed the one-page pre-registration and D68 replan sheet [`docs/harness/prep/m1-a8-full-dose-rerun-2026-10-25.md`](../../../harness/prep/m1-a8-full-dose-rerun-2026-10-25.md) and posted the fresh-context non-author `TEST FITNESS: FIT`, D68 decision review, and D67 pre-run review on `#2831` ([comment `6394635218`](https://github.com/UOR-Foundation/uor-r4/issues/2831#issuecomment-6394635218)).
- Updated [`crates/uor-r4-discrete/src/attention_audited.rs`](../../../../crates/uor-r4-discrete/src/attention_audited.rs), [`crates/uor-r4-discrete/src/driver_source_pin.rs`](../../../../crates/uor-r4-discrete/src/driver_source_pin.rs), and [`crates/uor-r4-discrete/src/bin/a8-qualify-stage.rs`](../../../../crates/uor-r4-discrete/src/bin/a8-qualify-stage.rs) (`stage-dose` and `verify-dose` subcommands plus `5` unit tests) to enforce the replanned post-`rows_v1` gate-cost readiness bound (`R <= 85%`, `projected_added_gate_ns <= 5,500,000,000,000 ns`), bind the Stage 1 `fd409fcdf3bc43d607e768bae6f8cf403aa75901` qualification/calibration artifacts by their invariant `config_sha256`, `kernel_sha256`, `geometry_sha256`, `lineage_sha256`, and `sheet_sha256`, and stage/verify both arms' audited dose runs.
- Executed both registered arms (`Relation` `LearnV1` seed 1 on GPU 0 and `FrozenZero` `FrozenZeroV1` seed 1 on GPU 1) in parallel via `discrete-stack attention-audit` on `vast-57627077` (`2 x NVIDIA GeForce RTX 5090`, driver `580.95.05`, `$2.021/h`, leased `~48 min` / `~$1.62`; active `discrete-stack` execution `1,443.82 s` = `24.06 min` / `~$0.81`), verified outputs via `a8-qualify-stage verify-dose` (`VERIFIED_AUDITED_DOSE_OUTPUTS`), archived the complete output (`a8-full-dose-rerun-20261025.tar.gz`, `4a744188f14c934f89e57a1be83ecc8f2cb211620290a6539217837f809bae55`) to `caseyallard/uor-r4-store:results/antigravity/a8-full-dose-rerun-2026-10-25/`, and committed [`docs/labs/m1-a8-full-dose-rerun/dose-summary.json`](../../../labs/m1-a8-full-dose-rerun/dose-summary.json) and [`docs/labs/m1-a8-full-dose-rerun/README.md`](../../../labs/m1-a8-full-dose-rerun/README.md).
- Added the `2026-10-25` result row to [`docs/research/notebook/m1-discrete-attention.md`](../../../research/notebook/m1-discrete-attention.md) and set [`docs/roadmap/items/m1-a8-full-dose-rerun.md`](../../../roadmap/items/m1-a8-full-dose-rerun.md) to `status: done`.
- Diff size: `12` files changed (`+1,399 / -60` lines: `tests 0 file(s) +0/-0, code 5 file(s) +895/-59, docs 6 file(s) +347/-1, generated 0 file(s) +0/-0, other 1 file(s) +157/-0`).

## Measurements

- **Pod & Environment:** `vast-57627077` (`2 x NVIDIA GeForce RTX 5090`, driver `580.95.05`, CUDA `12.8`, `$2.021/h`, leased `2026-10-25T06:59:51Z`–`2026-10-25T07:47:56Z` = `~48 min` / `~$1.62` total lease; active `discrete-stack` execution `1,443.82 s` = `24.06 min` / `~$0.81`).
- **Binary & Staged Plans (`da940043252432fcbd90e0432e3870b39e92c625`):**
  - `discrete-stack` (`b54d458af2f63990efea390c1f8e111e51454e73257c022565dcb1b89c561728`), `driver_source_sha256 = 31789abea8b955cb0e137257fc9e8b3e5eb42865fec7728396fa377c9d4228ea`.
  - `relation-plan.json` (`7ca3c1322fab29fb3e63ec96b2662387bc1e7cb542c4706b3ca112823427cd11`) and `frozen-zero-plan.json` (`4d3572b24ae8e65e60dfb1fdf32c1eb80c02bb16d174a322877ccc09cde6ada7`): both `PLAN_METADATA_VALID`.
- **Commands run on pod:**
  ```sh
  CUDA_VISIBLE_DEVICES=0 /workspace/uor-r4/target/release/discrete-stack attention-audit \
    /workspace/uor-r4/docs/labs/a8-training-configs-2026-10-19/evidence/a8-relation-seed1.json \
    /workspace/a8-dose/dose-private/relation-admission.json \
    /workspace/a8-dose/relation-out

  CUDA_VISIBLE_DEVICES=1 /workspace/uor-r4/target/release/discrete-stack attention-audit \
    /workspace/uor-r4/docs/labs/a8-training-configs-2026-10-19/evidence/a8-frozen-zero-seed1.json \
    /workspace/a8-dose/dose-private/frozen-zero-admission.json \
    /workspace/a8-dose/frozen-zero-out
  ```
- **Paired Arm Results (`dose-summary.json` `daad45e0c165472465823c7e22b0ba8e7832a7570fa4b1fea9ff8b03d9fec3c2`):**
  - **Initial Baseline (`batch = 0`, identical on both arms):**
    - `TRAIN anchor` (`3,847` rows, `anchor-00000.json`): `target_admitted_rows = 1,211` (floor `1,187`), `top1_correct = 43`, `top8_correct = 147`, `admitted_strict_win_rows = 43`, `objective_loss_numerator_sum = 3,656,320`.
    - `DEV P_succ` (`3,847` rows, `report.json` `baseline_b3.psucc` & `baseline-dev.json` `is_psucc` rows): `target_admitted_rows = 1,351` (Q1 floor `676`), `top1_correct = 62`, `top8_correct = 202`, `objective_loss_numerator_sum = 3,602,336`.
    - `DEV Full` (`14,287` rows, `baseline-dev.json`): `target_admitted_rows = 4,244`, `top1_correct = 133`, `top8_correct = 513`, `admitted_strict_win_rows = 133`, `objective_loss_numerator_sum = 13,979,232`.
  - **`FrozenZero` (`FrozenZeroV1`, Seed 1) — Q1 (`batch = 4,096`, `epoch = 258`, `q1.json` `1e436e66a243cf7f9367c14372ef44ff1b9459fce98bf172100f28c5323f5ef7`):**
    - `diagnostic_stop = false` (`PASS` Q1 gate):
    - `TRAIN anchor b_3(P_succ)` (`anchor-04096.json`) = **`1,258 / 3,847`** (`+47` over baseline `1,211`, `+71` over floor `1,187`), `top1_correct = 352 / 3,847` (`8.19x` over initial `43`), `top8_correct = 516 / 3,847`, `admitted_strict_win_rows = 294`, `objective_loss_numerator_sum = 3,512,640` (`-143,680`).
    - `DEV b_3(P_succ)` (`q1.json` `q1_b3.psucc`) = **`1,106 / 3,847`** (`>= 676` and `>= 500`; vs `857 / 3,847` in ungated `#2831` parent, `+249` rows preserved).
    - `DEV P_succ` (`q1-dev.json` `is_psucc` rows): `top1_correct` = **`229 / 3,847`** (`3.69x` over initial `62`), `top8_correct` = **`385 / 3,847`**, `objective_loss_numerator_sum` = `3,600,256` (`-2,080`).
    - `DEV b_3(Full)` (`q1-dev.json` `6b924afa04103467814b6c71773f7df2c7e6df342f045d83434d3dafe8372943`) = **`4,292 / 14,287`** (`+48` over initial `4,244 / 14,287`).
    - `DEV Full top1_correct` = **`871 / 14,287`** (`+738` over initial `133`, `6.55x`, and `+408` over best prior trained discrete model's `463 / 14,287`), `top8_correct` = **`1,455 / 14,287`**, `admitted_strict_win_rows` = **`679`**, `objective_loss_numerator_sum` = `13,288,896` (`-690,336`).
  - **`FrozenZero` Terminal (`batch = 5,248`, `report.json` `ca5f946945763142fa45e461014a7cc765cb4b53b5850143013a61ed8049a421`):**
    - `status = "STOP_FOR_DIAGNOSIS"`, `stop_reasons = ["consecutive_rejections"]`, `audits_count = 83` (`1` baseline + `82` interval audits), `rejected_intervals = 15`, `committed_accepted_updates = 274` (`190` vector + `84` combine + `0` relation; `356` relation proposals refused by `FrozenZeroV1`), `elapsed_ns = 1,099,022,329,790` (`1,099.02 s`).
    - Final restored checkpoint (`continuation-05248.json` `70935d37234a868ceeeff66a132fb4584e27a67710f06ec7b87db39358fbc36b`): `epoch = 274`, `TRAIN anchor admitted = 1,249 / 3,847 >= 1,187`.
  - **`Relation` (`LearnV1`, Seed 1) Terminal (`batch = 1,600`, `report.json` `eb02da3b46c72591e062025a7f1ecb0fecc8e493947d8efc7fd4e3e1e92ecbc1`):**
    - `status = "STOP_FOR_DIAGNOSIS"`, `stop_reasons = ["consecutive_rejections"]`, `audits_count = 26` (`1` baseline + `25` interval audits), `rejected_intervals = 5`, `committed_accepted_updates = 132` (`82` vector + `28` combine + `22` relation), `elapsed_ns = 1,443,815,125,983` (`1,443.82 s`).
    - Final restored checkpoint (`continuation-01600.json` `fccbaab5bc607893e4bfda8a7e324713e9161b1ea7a4c1f7e69c1322feea44ea`): `epoch = 132`, `TRAIN anchor admitted = 1,232 / 3,847 >= 1,187`.
  - **Fresh Slice Integrity:** `fresh_evaluations = 0`, `status = "SEALED_PENDING_PAIRED_DEV_DECISION"` on both arms.

## Checks run at the head

- `~/.cargo/bin/cargo fmt --check`: PASS.
- `~/.cargo/bin/cargo test --locked -p uor-r4-discrete --bin a8-qualify-stage --no-fail-fast`: `5/5` tests PASS.
- `~/.cargo/bin/cargo test --locked -p uor-r4-discrete --lib attention_audit -- --nocapture`: `40/40` tests PASS.
- `~/.cargo/bin/cargo test --locked -p uor-r4-discrete --lib driver_source_pin -- --nocapture`: `2/2` tests PASS.
- `~/.cargo/bin/cargo clippy --no-deps --locked -p uor-r4-discrete --bin a8-qualify-stage --tests`: PASS (`0` warnings/errors).
- `uor-harness loop report --check --order docs/loop/tasks/m1-a8-full-dose-rerun/antigravity-57/ORDER.md`: PASS.
- `uor-harness loop pr-check --local`: PASS.
- `uor-harness loop gates --full`: PASS.

## Skills used

- `uor-research-architect`, `scope-guard`, `solution-integrity`, `gates`, `context-guard`.

## Still open

- `m1-a8-full-dose-rerun` (`#2831`) is complete (`status: done`).
- On the M1 path in `PLAN.md`, the next open steps are `m1-bvg-k-block-apply-c2` (`#2835`, Step 3) and `m1-bvg-k-stage-t-training` (`#2836`, Step 4), where batched `K = 16` coordinate evaluation (`BVG-K`) replaces single-coordinate (`K = 1`) local-batch greedy updates.

## Plain summary

We ran the paired admission-preserving A8 attention training experiment (`#2831`) on two RTX 5090 GPUs, comparing the learned relation arm (`Relation`, where the model learns which earlier words to read) against the control arm (`FrozenZero`, where the relation gate is held fixed at zero). Every `64` training steps (an *interval*), the controller checked *admission* (whether the true next word's group, or *cell*, is on the model's 8-cell shortlist) on a fixed 3,847-example training subset (*TRAIN anchor*) of copy-successor examples (*P_succ*, where the next word appeared earlier in the text). Whenever a 64-step interval let training admission drop below `1,187` of `3,847` (the starting `1,211` minus the allowed `24`-row margin), the controller rolled the model back to the last passing checkpoint (*whole-interval rollback*, `D40`).

1. **Protecting admission preserved reach and raised development accuracy `6.55x` at the first quarter-dose check (`Q1`, step `4,096`):**
   - In the earlier ungated A8 run (`19 Oct`), training pushed target cells off the shortlist (`Relation` fell from `1,351` to `551 / 3,847` on development `P_succ`; `FrozenZero` fell to `857 / 3,847`).
   - With active whole-interval rollback (`D40`), `FrozenZero` passed `Q1` (`step 4,096`, `258` committed updates) with `1,258 / 3,847` training anchor admission (`+47` above start), **`1,106 / 3,847`** development `P_succ` admission (`+249` above the ungated run), and **`4,292 / 14,287`** full development panel admission (`+48` above the starting `4,244`).
   - With reach protected, `FrozenZero`'s exact first-choice accuracy (*top-1*) on the 14,287-row development panel rose from `133` at start to **`871 / 14,287`** (`6.55x`, and `+408` rows above the best previous trained discrete model's `463 / 14,287`), and its top-8 accuracy reached **`1,455 / 14,287`**.
2. **Both arms stopped on `consecutive_rejections` (`Relation` at step `1,600`, `FrozenZero` at step `5,248`), classified as an `instrument` cadence-granularity stop (`mechanism_negative: false`):**
   - Both arms preserved training anchor admission at their restored checkpoints (`1,232 / 3,847` for `Relation` after `132` committed updates; `1,249 / 3,847` for `FrozenZero` after `274` committed updates).
   - Because single-coordinate (`K = 1`) moves are accepted greedily on a tiny 64-example batch while admission is checked on the 3,847-example training anchor only once every `64` steps (`4–11` bundled updates), a single locally greedy update inside a 64-step window that erodes global admission causes the entire 64-step window to be rolled back; three such windows in a row trigger the `consecutive_full_rejections = 3` stop. The held-out test set (*fresh slice*) was never touched (`0` evaluations).
