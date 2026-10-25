# REPORT: native-geometric-reregister-hinge, antigravity-64

Lab: antigravity. Item: [native-geometric-reregister-hinge](../../../../roadmap/items/native-geometric-reregister-hinge.md). Order: [ORDER.md](ORDER.md).

References #2871.

## What did not work

- **No pre-registration sheet, `native-geometric` excluded by facade, D29 quarantined CUDA kernels, and docs-only item scope (`instrument`)**: Running the cancel probe on `native-geometric-reregister-hinge` (`#2871`) at base `188af113f` uncovered four pre-execution blockers:
  1. No pre-registration sheet existed under `docs/harness/prep/native-geometric-reregister-hinge*`, and no lab directory existed under `docs/labs/native-geometric-reregister-hinge/`.
  2. `crates/uor-r4-core/src/native_support.rs:1-15` excludes the historical `native_geometric` executors via its facade, and `CHARTER.md` §2.4 keeps `native-geometric` `PARKED`.
  3. `crates/uor-r4-training/src/native_geometric_cuda_kernels.rs:23-28,65-79` quarantines the 15 October native-context CUDA kernels under `D29` (`HISTORICAL_KERNELS` bail on launch) because they contained banned exponential normalizers (`D23`) and a learned dense `4 -> 120/33` contraction (`D29`/`D30`, `docs/mechanisms/native-geometric.md:55,96`).
  4. `docs/roadmap/items/native-geometric-reregister-hinge.md` line 10 restricts `scope:` to `[docs/harness/prep/native-geometric-reregister-hinge*, docs/labs/native-geometric-reregister-hinge/**, docs/loop/tasks/native-geometric-reregister-hinge/**]`, excluding all Rust crates (`crates/uor-r4-core`, `crates/uor-r4-training`, `crates/uor-r4-integer`).
  - Per `CHARTER.md` §3.5, §4.2, §4.5, §5.1 (D68), and `PLAN.md` lines 75–86, this unit completed all in-scope deliverables (the FIT-reviewed pre-registration sheet and lab specification) and placed `#2871` on `status: held` under D68 until the D29 kernel rebuild, shared-writer join, and M1 (`#2029`) close land.

## What changed

- **Size**: 6 files changed (0 code, 0 test, 5 docs +235/-1, 1 generated +3/-3; total +238/-4 lines against an item without a line estimate; ratio 1.0x against the ORDER plan).
  - [`docs/harness/prep/native-geometric-reregister-hinge-2026-10-25.md`](../../../../harness/prep/native-geometric-reregister-hinge-2026-10-25.md) (+93 lines): One-page pre-registration sheet (`CHARTER.md` §4.5, `docs/mechanisms/native-geometric.md:92-99`, `docs/harness/panels/m2-fresh256.json`) replacing the banned softplus pooled-ranking loss $\ell = \operatorname{softplus}(\log(m_{\text{rival}}/m_{\text{gold}}))$ and log-sum-exp episode aggregation with the lawful piecewise-linear top-$k$ (`k = 8`, $\delta = 2^{22}$) rank-margin hinge $\ell_{\text{hinge}}$ and uniform episode average $J(e)$ over the `115,200` Q4 coefficients of the $8 \times 120 \times 120$ cross-state $H_4$ banks (`saved177` parent `dfb945d5...`). Defines four arms (`H_hinge_q4`, `C_parent177`, `C_rival_shuf`, `C_question_blind`), six finite controls (`I-1..I-6`), two seeds (`1001, 1002`), cluster-adjusted noise bands (`6.24` rows on `m2-reply512`, `8.64` rows / `4.32` pairs on `m2-fresh256`), dual-panel bars (`>= 256/512` dev, `pairs_both_complete_distinct >= 52/128` and `complete >= 104/256` on `m2-fresh256`, D37 export gap `<= 12` rows), SHA-256 pins, cost/stop rule, and the non-author `TEST FITNESS: FIT` stamp (`5f00472e-08a8-477c-9cf6-8946b52f4175`, [`#2871` comment `6396265555`](https://github.com/UOR-Foundation/uor-r4/issues/2871#issuecomment-6396265555)).
  - [`docs/labs/native-geometric-reregister-hinge/README.md`](../../../../labs/native-geometric-reregister-hinge/README.md) (+46 lines): Codebase audit of `native_support.rs`, `native_geometric_cuda_kernels.rs`, `m2-pooled-rank-2026-10-10`, and `m2-fresh256`, the lawful rank-margin hinge formulation, and the prerequisites to unhold and execute on GPU.
  - [`docs/roadmap/items/native-geometric-reregister-hinge.md`](../../../../roadmap/items/native-geometric-reregister-hinge.md) (+1/-1 lines): Set `status: held` under the fresh-context D68 decision review posted on [`#2871` comment `6396265555`](https://github.com/UOR-Foundation/uor-r4/issues/2871#issuecomment-6396265555).
  - [`docs/roadmap/README.md`](../../../../roadmap/README.md) (+3/-3 lines): Re-rendered via `cargo run -p xtask -- roadmap render`.
  - [`docs/loop/tasks/native-geometric-reregister-hinge/antigravity-64/ORDER.md`](ORDER.md) (+43 lines) and [`docs/loop/tasks/native-geometric-reregister-hinge/antigravity-64/REPORT.md`](REPORT.md) (+52 lines): Task order and report.

## Measurements

- **Cancel probe and codebase verification** (macOS host, commit `f7eac7735`, `uor-harness check` and `xtask links`):
  - `crates/uor-r4-core/src/native_support.rs:1-15` and `crates/uor-r4-training/src/native_geometric_cuda_kernels.rs:23-28`: confirmed `native_geometric` is excluded via facade and `HISTORICAL_KERNELS` bail under `D29`.
  - `docs/harness/prep/native-geometric-reregister-hinge-2026-10-25.md`: `0` `prep.sheet_shape` errors and `0` `prep.fit_before_run` warnings under `uor-harness check`.
  - `cargo run -p xtask -- links --json local/antigravity/links.json`: `0` broken links in `native-geometric-reregister-hinge` files.

## Checks run at the head

- `uor-harness loop gates --full` (`PASS`: all 11 gates green)
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run -p xtask -- roadmap check` (`92 items OK: ready 43, held 7, decision 1, done 41`)
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run -p xtask -- docs-catalog --check` (`1175 records, 20 lines, 0 paths not found`)
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run -p xtask -- links --json local/antigravity/links.json` (`0` broken links in touched files)
- `uor-harness loop report --check --order docs/loop/tasks/native-geometric-reregister-hinge/antigravity-64/ORDER.md` (`PASS`)
- `uor-harness loop pr-check --local` (`PASS` on all hard rules)

## Skills used

- `working-discipline`: Verified repository state (`native_support.rs`, `native_geometric_cuda_kernels.rs`, `docs/mechanisms/native-geometric.md`, `docs/mechanisms/rank-margin.md`, `archive/labs/m2-pooled-rank-2026-10-10/README.md`, and `docs/harness/panels/m2-fresh256.json`) before authoring the pre-registration sheet.
- `scope-guard`: Kept all edits strictly inside `native-geometric-reregister-hinge`'s declared scope (`docs/harness/prep/native-geometric-reregister-hinge*`, `docs/labs/native-geometric-reregister-hinge/**`, `docs/loop/tasks/native-geometric-reregister-hinge/**`), the item's `status:` line, and the shared `docs/roadmap/README.md` index.
- `solution-integrity`: Formulated the linear rank-margin hinge replacement (`k = 8`, $\delta = 2^{22}$) and six finite controls (`I-1..I-6`) with zero exponential/squashing functions, zero dense matmul, separate exact occurrence/version identity (`CHARTER.md` §3.1–§3.7, §3.12), and the sealed 256-row `m2-fresh256` transfer gate.

## Still open

- Rebuilding the 15 quarantined D29 native-context CUDA kernels without hidden dense contractions (`docs/mechanisms/native-geometric.md:96`), joining token/action emission to `crates/uor-r4-training/src/native_vector_writer.rs` (`docs/mechanisms/native-geometric.md:97`), and running the two-seed GPU test on `m2-reply512` and `m2-fresh256` after M1 (`#2029`) closes (`PLAN.md` lines 75–77).

## Plain summary

The earlier "native geometric engine" (`saved177` -> `saved437`) learned 4-bit ("Q4") adjustments over 4D icosian rotation tables (`H4`) and answered 437 of 512 development questions, but scored 0 of 128 on fresh questions and relied on two things the project now bans: a softplus/log-sum-exp ranking loss and CUDA kernels containing a hidden dense matrix contraction (`D29`). Because this roadmap item's file scope only covers documentation and pre-registration files, this unit authored and FIT-reviewed the one-page pre-registration sheet (`docs/harness/prep/native-geometric-reregister-hinge-2026-10-25.md`) and lab specification (`docs/labs/native-geometric-reregister-hinge/README.md`) replacing the banned softplus loss with the lawful piecewise-linear rank-margin hinge across two seeds on `m2-reply512` (`>= 256/512`) and the sealed 256-row `m2-fresh256` held-out panel (`>= 52/128` opposite-role pairs), and placed `#2871` on `held` under D68 until the D29 kernel rebuild and M1 close land.
