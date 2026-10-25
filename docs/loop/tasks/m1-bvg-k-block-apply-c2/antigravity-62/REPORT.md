# REPORT: m1-bvg-k-block-apply-c2, antigravity-62

Lab: antigravity. Item: [m1-bvg-k-block-apply-c2](../../../../roadmap/items/m1-bvg-k-block-apply-c2.md). Closes #2835.

## What did not work

- **No Stage T GPU training run executes in this unit**: Per the roadmap item (`needs_gpu: false`), `m1-bvg-k-block-apply-c2` (`#2835`) verifies the Stage C2 block-apply qualification receipt (`2fff942fa2b802e180c9bd87336692730d6beaf22f2c0bac8867afb0dea345f3`) and registers the one-page Stage T pre-registration sheet (`docs/harness/prep/m1-bvgk-stage-t-training-2026-10-25.md`) with its non-author `TEST FITNESS: FIT` and D68 decision review. The Stage T GPU training run (`~$12`, two seeds) is the separate downstream unit `m1-bvg-k-stage-t-training` (`#2836`, Step 4 in `PLAN.md`), which runs under its own D67 pre-run review.
- **Stage C2 fixture controls left three driver/device checks outside C2's fixture scope**: In Stage C2 (`docs/labs/injective-preflight-and-c2-qualify-2026-10-19/README.md`), all `3,419` updates ran on synthetic fixture devices (`VOCABULARY = 16`, `CONTEXT = 12`, `kept_cells = 4`), leaving (1) single-move parity between `discrete_bvgk_apply` and the ordinary `Device::apply` kernel (`AKd`), (2) `K = 1` identity between `transition_block` and the production `transition` law (`V1d`), and (3) driver-level block-transition accounting, `FrozenZeroV1` relation-trit mask integrity, and batch-64 checkpoint/resume bitwise parity (`KId`, `V8d`, `V9d`) outside C2. Rather than leaving those unverified during Stage T, Section 4 of `docs/harness/prep/m1-bvgk-stage-t-training-2026-10-25.md` registers all three (`AKd`, `V1d`, `KId`/`V8d`/`V9d`) as mandatory Step 0 GPU controls before any Stage T training metric is read.
- **Predecessor 19 October draft sheet (`m1-bvgk-staget-learning-law-DRAFT-2026-10-19.md`) could not be activated without binding `#2831` and `#3179`**: The 19 October draft left the base lineage (`B3_init` and `Q1` pass) and the W2 admission-matched comparators unresolved (`TEST FITNESS: PENDING`). Now that `#2831` (`m1-a8-full-dose-rerun`, merged as `dc5f409b0`) and `#3179` (`m1-w2-code-utilization`, merged in PR `#3193`) are both `done`, the new sheet `docs/harness/prep/m1-bvgk-stage-t-training-2026-10-25.md` binds both exact artifacts and comparators.

## What changed

Closes #2835.

- **Size**: `6` files changed (`0` code, `0` test, `6` docs, `0` generated), `+261 / -2` lines (`263` lines touched vs a `~250`-line estimate for the one-page Stage T sheet, lab summary, notebook row, item status, ORDER, and REPORT; ratio `1.05x`).
- **Files modified / added**:
  - [`docs/harness/prep/m1-bvgk-stage-t-training-2026-10-25.md`](../../../harness/prep/m1-bvgk-stage-t-training-2026-10-25.md): One-page Stage T pre-registration sheet binding:
    - Geometric construction (`K = 16` coordinates scored per GPU launch via `score_rows_k` in `rowsk.cu` / `bvgk.cu`, `S_t` chosen via `discrete_bvgk_choose`, scratch-clone `L(S_t)` verified when `|S_t| >= 2`, `S_t` committed iff `L(S_t) < L({best})` else `{best}` via `discrete_bvgk_apply` in one epoch increment; integer rank-margin hinge loss, E8 orbit / SpiralCore table actions, and integer L1/Hamming additions; zero softmax/squashing §3.2 and zero matmul §3.3).
    - Passed C1 (`d5ec5348d64e2abd74a334319d4f78e6ed7b174b5a2fc0bb381505dc46e09e56`) and C2 (`2fff942fa2b802e180c9bd87336692730d6beaf22f2c0bac8867afb0dea345f3`) qualification receipts.
    - Mandatory Step 0 GPU controls (`AKd`, `V1d`, `KId`, `V8d`, `V9d`) covering all three items left outside Stage C2 before any training result is read.
    - Qualified A8 address-structured base codebook (`address-structured-v1.json`, `6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1`) and active D40 64-batch whole-interval admission-preserving rollback (`AuditPolicy::registered()`) from `#2831`.
    - Stage 2 bar across two seeds (`seed 1`, `seed 2`): keep DEV admission (`>= 4,220 / 14,287`), beat `T1` (`K = 1`), and beat the unigram table (`1,226 / 14,287`), then the admission-matched next-word table produced by W2 (`#2845` / `#3179`: `1,138 / 14,287` restricted code table and `1,482 / 14,287` restricted bigram on baseline, plus checkpoint-matched `T_restr(A_W)` / `G_restr(A_W)` on the checkpoint's admitted rows `A_W`, not the `6,300` full-panel `T2` table).
    - Compute budget (`~$12` carried estimate, hard cap `$15.00`, on `2x RTX 5090` inside `<= 4` pods and `<= $8/h` fleet caps), stop conditions, uncovered scope, and stamped `TEST FITNESS: FIT` (`#2835` comment `6395412609`).
  - [`docs/labs/m1-bvg-k-block-apply-c2/README.md`](../../../labs/m1-bvg-k-block-apply-c2/README.md): Preserved the 2026-10-22 `deepseek` historical section intact (`CHARTER §4.7`) and appended the 2026-10-25 Stage C2 qualification (`8 / 8` controls PASS) and Stage T pre-registration summary.
  - [`docs/research/notebook/m1-discrete-attention.md`](../../../research/notebook/m1-discrete-attention.md): Appended one row for `2026-10-25` recording the Stage C2 receipt verification and Stage T pre-registration (`positive (qualification and Stage T pre-registration complete; unlocks #2836 Stage T GPU training; no training run in #2835)`).
  - [`docs/roadmap/items/m1-bvg-k-block-apply-c2.md`](../../../roadmap/items/m1-bvg-k-block-apply-c2.md): Updated `status: ready` to `status: done`.
  - [`docs/loop/tasks/m1-bvg-k-block-apply-c2/antigravity-62/ORDER.md`](ORDER.md) and [`REPORT.md`](REPORT.md): Task order and report.

## Measurements

- **Stage C2 Qualification Receipt Verification (host check)**:
  - Command: `shasum -a 256 docs/labs/injective-preflight-and-c2-qualify-2026-10-19/evidence/bvgk-c2-qualify-20261019/report/bvgk-qualification.json`
  - Result: `2fff942fa2b802e180c9bd87336692730d6beaf22f2c0bac8867afb0dea345f3` (`status = "PASS"`, `8 / 8` controls `V0`, `RC` `64` cases / `14,080` lanes, `UT`/`JR`/`DC` `804` proposals = `260` joint wins + `59` fallback-to-best + `345` single + `140` empty with `319/319` multi-coordinate JR parity, `V3` `664/664` block commits = `260` joint, `RF` `9/9`, `C1R` `5,130/5,130` across `2,028` blocks, `3,419` fixture updates on RTX 5090).
- **Discrete Crate Host Check**:
  - Command: `~/.cargo/bin/cargo check -p uor-r4-discrete`
  - Result: `exit 0` (`Finished dev profile` in `14.34s` on host without `cuda`).
- **Pre-Registration Sheet Structural Validation**:
  - Command: `~/.cargo/bin/cargo test --locked -p uor-r4-harness --lib prep::tests::the_real_sheets_are_readable -- --nocapture`
  - Result: `test prep::tests::the_real_sheets_are_readable ... ok` (`1 passed; 0 failed`).
- **Fresh-Context Non-Author `TEST FITNESS` & `D68` Decision Review**:
  - Reviewer: `antigravity` fresh-context non-author reviewer (`bb65a68d-8e8b-4e09-8d64-f624827d645f`), posted on [`#2835` comment `6395412609`](https://github.com/UOR-Foundation/uor-r4/issues/2835#issuecomment-6395412609).
  - Result: `TEST FITNESS: FIT` and `D68 DECISION REVIEW: PASS`.

## Checks run at the head

- `shasum -a 256 docs/labs/injective-preflight-and-c2-qualify-2026-10-19/evidence/bvgk-c2-qualify-20261019/report/bvgk-qualification.json` (`2fff942fa2b802e180c9bd87336692730d6beaf22f2c0bac8867afb0dea345f3`, `PASS`)
- `~/.cargo/bin/cargo check -p uor-r4-discrete` (`exit 0`, `PASS`)
- `~/.cargo/bin/cargo test --locked -p uor-r4-harness --lib prep::tests::the_real_sheets_are_readable` (`1 passed; 0 failed`, `PASS`)
- `uor-harness loop report --check --order docs/loop/tasks/m1-bvg-k-block-apply-c2/antigravity-62/ORDER.md` (`PASS`)
- `uor-harness loop pr-check --local` (`PASS`)
- `uor-harness loop gates --full` (`4/4 PASS`)

## Skills used

- `scope-guard`: Kept all changes strictly inside the item's declared `docs/harness/prep/m1-bvgk-stage-t*`, `docs/labs/m1-bvg-k-block-apply-c2/**`, `docs/loop/tasks/m1-bvg-k-block-apply-c2/**`, `docs/research/notebook/m1-discrete-attention.md`, and the item status line in `docs/roadmap/items/m1-bvg-k-block-apply-c2.md`.
- `solution-integrity`: Bound the Stage T pre-registration sheet to the exact SHA-256 artifacts from `#2831` (A8 address-structured codebook and D40 rollback controller) and `#3179` (W2 admission-matched count-table comparators), and turned all three items left outside Stage C2 (`AKd`, `V1d`, `KId`/`V8d`/`V9d`) into mandatory Step 0 GPU controls before any Stage T training result is read.
- `gates`: Verified `prep::tests::the_real_sheets_are_readable`, `uor-harness loop report --check`, `uor-harness loop pr-check --local`, and `uor-harness loop gates --full`.

## Still open

- **Step 4 (`m1-bvg-k-stage-t-training`, `#2836`)**: Now unblocked by `#2835` and `#3179`. Executes the registered Stage T two-seed GPU run (`~$12` budget) under `#2836`'s own D67 pre-run review.
- **Steps 6–8 (`#2839`, `#2841`, `#2837`)**: Downstream M1 export gap, fresh-panel transfer (`fresh_v2` remains sealed with `0` evaluations), and M1 closing bar (`CHARTER §4.8`).

## Plain summary

Earlier in M1, the discrete language model updated only one parameter coordinate (`K = 1`) per 64-row training batch. In `#2831` (`m1-a8-full-dose-rerun`), combining the A8 address-structured starting codebook (where each token's cell sits near its predecessors' cells) with the D40 64-batch admission-preserving rollback controller lifted the model's development top-1 next-word score to `871` out of `14,287` rows at batch `4,096` (`6.55x` its starting `133`), but then stopped because greedy single-coordinate updates on 64 rows caused whole 64-batch intervals to roll back whenever one move inside the interval hurt global shortlist admission (**admission** is whether the true next word's cell makes the model's 8-cell shortlist). Meanwhile, `#3179` (`m1-w2-code-utilization`) showed that on the rows the writer already admits, an admission-matched count table keyed by the model's own read code gets `1,138 / 14,287` and an admission-matched bigram table gets `1,482 / 14,287`.

This unit (`#2835`) completes the bridge to **BVG-K Stage T (`#2836`)**, which scores `K = 16` coordinates in one GPU launch (`Stage C1`) and commits a joint block of improving moves in one epoch when their combined loss beats the best single move (`Stage C2`, already qualified on GPU with receipt `2fff942fa2b802e180c9bd87336692730d6beaf22f2c0bac8867afb0dea345f3`). It registers the one-page Stage T pre-registration sheet (`docs/harness/prep/m1-bvgk-stage-t-training-2026-10-25.md`), covers the three remaining driver/device checks (`AKd`, `V1d`, `KId`/`V8d`/`V9d`) as mandatory Step 0 GPU checks before any training number is read, pins the Stage 2 bar (beat the `1,226` unigram table, then the W2 admission-matched next-word tables across two seeds for about `$12`), records its fresh-context non-author `TEST FITNESS: FIT` and D68 decision review on `#2835`, and unblocks `#2836`.
