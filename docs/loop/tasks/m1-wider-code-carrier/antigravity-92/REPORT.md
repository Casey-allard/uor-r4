# REPORT: m1-wider-code-carrier, antigravity-92

Lab: antigravity. Item: [m1-wider-code-carrier](../../../../roadmap/items/m1-wider-code-carrier.md). Order: [ORDER.md](ORDER.md).
Closes #2847.

## What did not work

1. **No `P_succ` cell-admission gain over $B = 4$ under `CellEnvelopeL1V1` (`s[15] = 1`):**
   - On the 3,847-row `P_succ` copy-successor sub-panel under `CellEnvelopeL1V1`, $B = 4$ (`FV8-B4-Env`) already admits `1,980 / 3,847` (`51.5%`), whereas $B = 6$ (`WB6-Env`) admits `1,901 / 3,847` (paired McNemar `WB6-Env` vs. `FV8-B4-Env`: `b / c = 419 / 498`, `b - c = -79`, floor `F = 61`, `clears = false`) and $B = 8$ (`WB8-Env`) admits `1,965 / 3,847` (paired `FV8-B4-Env` vs. `WB8-Env`: `b / c = 439 / 424`, `b - c = +15`, floor `F = 59`, `clears = false`).
   - Thus `P_succ` cell admission under `CellEnvelopeL1V1` is already saturated at $B = 4$; widening the carrier does not raise `CellEnvelopeL1V1` `P_succ` cell admission.
2. **Decode throughput scales inversely with carrier width $B$:**
   - Full 14,287-row DEV decode speed (`V = 4,096, C = 192, K = 8, Relation { kept: 8, sweeps: 1 } ActiveLeafV1` on 1x RTX 5090 `sm_120`) drops from **`858` / `831` / `829` tokens/s** at $B = 4$ (`OuterL1V1` / `CellEnvelopeL1V1` / `FullVectorCellL1V1`) to **`668` / `643` / `644` tokens/s** at $B = 6$ (`-22.6%`) and **`548` / `527` / `527` tokens/s** at $B = 8$ (`-36.6%`).
3. **Zero-training scope and serving/training boundary:**
   - Every measurement in this unit is at checkpoint 0 (`applied_updates = 0`, `training: "HELD"`).
   - `d11_export::export_d11` (`UORLUT01`) and `CorpusUpdateAdmission` remain gated to `OuterL1V1` until `#2836`.
4. **Three pre-treatment instrument stops (`fault1`–`fault3`, archived in `caseyallard/uor-r4-store:results/antigravity/m1-wider-code-carrier-2026-10-26/fault{1,2,3}/`):**
   - `fault1`: `G-Q` Control 4 initially checked `dev_fv.propose().is_err()`, whereas the non-outer metric update guard lives in `training.rs` (`CorpusUpdateAdmission::guarded`), while `d11_export::export_d11` and foreign-batch `inspect_wider_carrier_rows` are the device/export boundary checks. Fixed in `8d41f62d0` before any panel arm ran.
   - `fault2`: `wider-carrier-job.sh` compared `.controls` raw JSON across Build 1 and Build 2 without stripping `.controls[3].detail.d11_exports[].artifact_sha256` (which embeds `Snapshot::source_sha256` containing `ptxas` compile-time lines across `target-wider-1` vs. `target-wider-2`). Fixed in `fdf6f78d2` before any panel arm ran.
   - `fault3`: `wider-carrier-job.sh` and `grade_wider_suite` initially asserted `WB6-Outer == A8-Outer` on the corpus arm, overlooking that `address-cells-codebook` computes a $B$-native PrimeZeta fallback (`roots[B-1]` from zeta-zero channels `4(B-1)..4B-1`) and `writer` ranks within-cell tokens by `token_distance` across all $B$ blocks (`G-Q` Control 3 already proves outer-endpoint and cell-admission invariance across $B \in \{4, 6, 8\}$ on a shared fallback). Fixed in `99b251ccb` before any Stage 5 treatment arm ran.

## What changed

- [`crates/uor-r4-discrete/src/address_cells.rs`](../../../../crates/uor-r4-discrete/src/address_cells.rs) and [`crates/uor-r4-discrete/src/bin/address-cells-codebook.rs`](../../../../crates/uor-r4-discrete/src/bin/address-cells-codebook.rs): added `AddressCellsMode::WiderBlocksV1`, `AddressCellsMode::WiderBlocksShuffledV1 { seed }`, and `latin_interior_anchor_stage` for multi-pair residual bidirectional bigram-pulled interior placement across $P = \lfloor (B - 1) / 2 \rfloor$ interior pairs (`P = 2` at $B = 6$, `P = 3` at $B = 8$).
- [`crates/uor-r4-discrete/build.rs`](../../../../crates/uor-r4-discrete/build.rs), [`crates/uor-r4-discrete/src/wider_carrier_source.rs`](../../../../crates/uor-r4-discrete/src/wider_carrier_source.rs), [`crates/uor-r4-discrete/src/wider_carrier_device.rs`](../../../../crates/uor-r4-discrete/src/wider_carrier_device.rs), [`crates/uor-r4-discrete/src/wider_carrier.rs`](../../../../crates/uor-r4-discrete/src/wider_carrier.rs), [`crates/uor-r4-discrete/src/bin/wider-carrier-qualify.rs`](../../../../crates/uor-r4-discrete/src/bin/wider-carrier-qualify.rs), and [`crates/uor-r4-discrete/src/bin/wider-carrier-run.rs`](../../../../crates/uor-r4-discrete/src/bin/wider-carrier-run.rs): added ahead-of-time `nvcc`-compiled wider-carrier CUDA inspection kernels (`wider_carrier.ptx` / `wider_carrier.cubin`), the 4-control finite CUDA qualifier (`wider-carrier-qualify`), and the 15-arm zero-training evaluator and deterministic McNemar grader (`wider-carrier-run`), preserving `driver_source_sha256 = 04c5165464a18eea6d4edadc6970007dbdc7908ead420f4a2f575e22ea43061d`.
- [`docs/harness/prep/m1-wider-code-carrier-2026-10-26.md`](../../../harness/prep/m1-wider-code-carrier-2026-10-26.md), [`docs/labs/m1-wider-code-carrier/README.md`](../../../labs/m1-wider-code-carrier/README.md), [`docs/labs/m1-wider-code-carrier/wider-carrier-qualification.json`](../../../labs/m1-wider-code-carrier/wider-carrier-qualification.json), [`docs/labs/m1-wider-code-carrier/grade.json`](../../../labs/m1-wider-code-carrier/grade.json), [`docs/labs/m1-wider-code-carrier/wider-carrier-job.sh`](../../../labs/m1-wider-code-carrier/wider-carrier-job.sh), and [`docs/labs/m1-wider-code-carrier/configs/`](../../../labs/m1-wider-code-carrier/configs/): recorded the FIT + D67 pre-registration sheet, the 12 wider-carrier `DriverConfig` JSONs, the RTX 5090 qualification and 15-arm evaluation artifacts, and the lab synthesis.
- [`docs/mechanisms/discrete-learning.md`](../../../mechanisms/discrete-learning.md), [`docs/roadmap/items/m1-wider-code-carrier.md`](../../../roadmap/items/m1-wider-code-carrier.md), and [`docs/roadmap/README.md`](../../../roadmap/README.md): added the `2026-10-26` wider-carrier measurement row, explicitly listed within-cell tie mass (`tokens_in_alias_groups == 0` and `dev_rows_targeting_non_representative == 0`) as a Stage 1 capacity gate alongside `G-Cap`, and marked `m1-wider-code-carrier` `status: done`.

## Measurements

- **Hardware:** 1x NVIDIA GeForce RTX 5090 (`vast-57809208`, driver `580.95.05`, `32,607 MiB`, `sm_120`, `$0.563/h`, torn down immediately after archiving).
- **Sealed artifacts:**
  - `docs/labs/m1-wider-code-carrier/wider-carrier-qualification.json`: `sha256: 384ce237403b82e8745b5f6fe3f9525fedb5f852e06dc9a0e087c9f34ac2418f` (`status: "PASS"`, all 4 `G-Q` controls pass on both Build 1 and Build 2; `wider-carrier.cubin` `sha256: 407b0c08f460a11a2ac83c770b00b111bcad593ba56e5a4bd4f95f791cf1a0d8`, resource report `sha256: bcf76eb6465b759fa73ff1642af2fadabb49bddcd2adc00d6c77ae814a05a5d9`).
  - `docs/labs/m1-wider-code-carrier/grade.json`: `sha256: 7766aae432500f632d4ffb01f9e7fb4f963c1b56c8ca3d81c0968b292a7300e0` (`overall_outcome: "PASS"`).
  - HF store: `caseyallard/uor-r4-store:results/antigravity/m1-wider-code-carrier-2026-10-26/run/` (84 files) and `tables/` (8 files).
- **Decode speed (`tokens/s` on full 14,287-row DEV panel; `rows/s` on 64-row timed batches):**
  - $B = 4$ (`width = 32`): `858` / `831` / `829` tokens/s (`1,756` / `1,678` / `1,678` rows/s) for `OuterL1V1` / `CellEnvelopeL1V1` / `FullVectorCellL1V1`.
  - $B = 6$ (`width = 48`): `668` / `643` / `644` tokens/s (`1,685` / `1,574` / `1,584` rows/s).
  - $B = 8$ (`width = 64`): `548` / `527` / `527` tokens/s (`1,544` / `1,433` / `1,433` rows/s).
- **15-Arm zero-training DEV evaluation (`14,287` rows, `3,847` `P_succ`; all 15 arms report `distinct_vectors = 4,096`, `occupied_cells = 256`, `within_cell_tie_tokens = 0`, `within_cell_tie_dev_rows = 0`, `dev_alias_ceiling_rows = 14,287`):**
  - `A8-Outer` ($B=4$): `P_succ = 1,351`, `Full-DEV = 4,244`, `Top-1 = 133`, `Top-8 = 513` (`G-Exact` exact match).
  - `FV8-B4-Env` ($B=4$): `P_succ = 1,980`, `Full-DEV = 5,709`, `Top-1 = 118`, `Top-8 = 1,126` (`G-Exact` exact match).
  - `FV8-B4-TwoScale` ($B=4$): `P_succ = 1,802`, `Full-DEV = 5,666`, `Top-1 = 118`, `Top-8 = 1,179` (`G-Exact` exact match).
  - `WB6-Outer` ($B=6$): `P_succ = 1,809`, `Full-DEV = 5,349`, `Top-1 = 265`, `Top-8 = 1,476` (`G-D11` `a328561d...`).
  - `WB6-Env` ($B=6$, `PASS`): `P_succ = 1,901` (vs. `A8-Outer`: `862 / 312`, `+550 > 69`; vs. `WBS6-Env = 728`: `1,609 / 436`, `+1,173 > 91`; vs. `WB6-Outer`: `336 / 244`, `+92 > 49`), `Full-DEV = 6,302`, `Top-1 = 295`, `Top-8 = 1,852`.
  - `WB6-TwoScale` ($B=6$, `PASS`): `P_succ = 1,846` (vs. `A8-Outer`: `803 / 308`, `+495 > 67`; vs. `WBS6-TwoScale = 825`: `1,534 / 513`, `+1,021 > 91`), `Full-DEV = 6,322`, `Top-1 = 295`, `Top-8 = 1,678`.
  - `WB8-Outer` ($B=8$): `P_succ = 1,471`, `Full-DEV = 4,639`, `Top-1 = 223`, `Top-8 = 1,686` (`G-D11` `042c2cb0...`).
  - `WB8-Env` ($B=8$, `PASS`): `P_succ = 1,965` (vs. `A8-Outer`: `925 / 311`, `+614 > 71`; vs. `WBS8-Env = 722`: `1,627 / 384`, `+1,243 > 90`; vs. `FV8-B8-Pad-Env = 1,710`: `457 / 202`, `+255 > 52`; vs. `WB8-Outer`: `641 / 147`, `+494 > 57`), `Full-DEV = 6,342`, `Top-1 = 240`, `Top-8 = 2,103`.
  - `WB8-TwoScale` ($B=8$, `PASS`): `P_succ = 1,977` (vs. `A8-Outer`: `927 / 301`, `+626 > 71`; vs. `WBS8-TwoScale = 789`: `1,594 / 406`, `+1,188 > 90`; vs. `FV8-B8-Pad-TwoScale = 1,716`: `458 / 197`, `+261 > 52`; vs. `WB8-Outer`: `624 / 118`, `+506 > 55`; vs. `FV8-B4-TwoScale = 1,802`: `572 / 397`, `+175 > 63`), `Full-DEV = 6,353`, `Top-1 = 240`, `Top-8 = 1,919`.

## Checks run at the head

- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test -p uor-r4-discrete --no-fail-fast`: 344 unit tests + integration/CLI binaries pass.
- `CARGO_TARGET_DIR=local/target-cuda NVCC=local/antigravity/fake-nvcc/nvcc UOR_NVCC_ARCH=compute_120 ~/.cargo/bin/cargo check --offline --features cuda -p uor-r4-discrete --all-targets`: 0 warnings, 0 errors.
- `uor-harness loop report --check --order docs/loop/tasks/m1-wider-code-carrier/antigravity-92/ORDER.md`: PASS.
- `uor-harness loop pr-check --local`: PASS.
- `uor-harness loop gates --full`: PASS.

## Skills used

- `uor-research-architect`, `scope-guard`, `solution-integrity`, `context-guard`.

## Still open

- None for `#2847` (`m1-wider-code-carrier` is `status: done`). Whether to carry $B = 4$ (`FV8-B4-Env`, `831 tokens/s`) or a wider carrier ($B = 6$ `WB6-Env`, `643 tokens/s`, `Top-1 = 295`, `Full-DEV = 6,302`) into training and D11 serving under a full-vector address metric belongs to `#2836` (`m1-stage1-admission-bar-gate`) and `#2835` (`m1-bvg-k-block-apply-c2`).

## Plain summary

We built and GPU-qualified a wider code carrier ($B = 6$ blocks = `48` integers and $B = 8$ blocks = `64` integers per token) with multi-pair residual bidirectional bigram-pulled interior placement (`AddressCellsMode::WiderBlocksV1`) and ran a 15-arm zero-training capacity pre-flight on 1x RTX 5090 (`vast-57809208`). All 4 finite CUDA qualification controls (`G-Q`), exact D11 integer export round-trip (`G-D11`), and occupancy/tie-mass gates (`G-Occ`, `within_cell_tie_mass == 0`, `G-Cap = 6,302..6,353 > 1,941`) passed across two independent `sm_120` builds. All four wider-carrier treatments (`WB6-Env`, `WB6-TwoScale`, `WB8-Env`, `WB8-TwoScale`) cleared the Stage 1 `P_succ` Reach bar against `A8-Outer` (`1,846..1,977 > 1,351`) and the Geometry bar against matched shuffled controls (`+1,021..+1,243`), lifted full-DEV cell admission from `5,709` ($B=4$) to `6,302..6,353`, and reversed `#2834`'s checkpoint-0 `Top-1` dip (`118` at $B=4$ to `295` at $B=6$ and `240` at $B=8$) at a linear decode-speed cost (`831 -> 643 -> 527 tokens/s`).
