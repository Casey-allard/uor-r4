# REPORT: m1-address-key-full-vector, antigravity-37

Lab: antigravity. Item: [m1-address-key-full-vector](../../../../roadmap/items/m1-address-key-full-vector.md).

## What did not work

- **Cancel probe failure:** Before adding `AddressCellsMode::FullVectorV1`, `cell_envelope_l1`, `full_vector_cell_l1`, and `admit_cells_by_distances` in `crates/uor-r4-discrete/src/address_cells.rs`, running `~/.cargo/bin/cargo test -p uor-r4-discrete --lib address_cells::tests::full_vector_address_metric_expands_levels_and_lifts_fixture_admission` executed `0 tests`. With a deliberate break that drops the interior-block distance in `full_vector_cell_l1` (reverting to `cell_outer_l1`), the test fails (`FAILED`, `distinct_levels == 5`, `outer_tie_bucket == 30`). Restoring the full-vector implementation returns `11 passed; 0 failed`.
- **Exact 30-cell $d_{\text{outer}} = 8$ boundary tie shell (`discrete.cu:274–276`):** In the 16-root SpiralCore orbit (`build.rs:232–241`), every root has $(1, 2, 13)$ orbit roots at $L_1$ distances $(0, 4, 8)$, so the 2-block outer `address_distance` partitions the 256 writer cells into 5 shells of exact sizes $(1, 4, 30, 52, 169)$ at distances $(0, 4, 8, 12, 16)$. Because $1 + 4 = 5 < 8$, both the $K = 8$ boundary (slots $6..8$) and the $K = 16$ boundary (slots $6..16$) fall inside the **30-cell $d_{\text{outer}} = 8$ tie shell**, where `cell_precedes` selects purely by raw cell index `first_root * 16 + last_root`.
- **Interior collapse in `StructuredV1` (`address_cells.rs`):** Because `StructuredV1` assigns `block 1 = ROOTS[k]` and `block 2 = ROOTS[k]` for within-cell rank $k \in 0..\text{quota}$ identically in every cell, even an unweighted full-vector cell-envelope minimum `cell_envelope_l1` stays collapsed at `5` distinct distance levels on `StructuredV1`. Furthermore, greedy interior pull without a global interior-pair quota (`interior_load[ic] < quota`) collapses tree-structured vocabularies onto the root's interior pair; adding balanced interior quota (`interior_load[ic] < quota` + within-cell uniqueness `!used_interior[cell][ic]`) resolves both.
- **No GPU execution or capability claim:** No CUDA kernel change has been run on a GPU pod yet. The one-page pre-registration sheet `docs/harness/prep/m1-address-key-full-vector-DRAFT-2026-10-23.md` is staged at `TEST FITNESS: PENDING` and awaits non-author `TEST FITNESS: FIT` and owner/PM registration on `#2834`. Item `m1-address-key-full-vector` stays `status: ready`.

## What changed

- `crates/uor-r4-discrete/src/address_cells.rs` (lines 26–51, 106–240, 290–430, 640–795): Added `AddressCellsMode::FullVectorV1` (`address_full_vector_v1`) and `AddressCellsMode::FullVectorShuffledV1 { seed }`, integer-only host metric primitives (`block_l1`, `cell_outer_l1`, `token_full_l1`, `cell_envelope_l1`, `full_vector_cell_l1`, `admit_cells_by_distances`), and 3 new finite CPU control unit tests (`11 / 11` total tests pass).
- `crates/uor-r4-discrete/src/bin/address-cells-codebook.rs` (lines 21–39): Added CLI mode support for `full-vector` and `full-vector-shuffled:SEED`.
- `docs/labs/m1-address-key-full-vector/README.md` (lines 1–82): Recorded the design note and finite CPU control results (`distinct_vectors = 512 / 512`, `dev_alias_ceiling_rows = 512 / 512`, `within_cell_tie_mass = 0`, distance levels `5 -> 7 -> 15`, $K=8$ boundary tie bucket `30 -> 6`, and predecessor-query $K=8$ fixture admission `186 -> 349 -> 446 / 511`).
- `docs/harness/prep/m1-address-key-full-vector-DRAFT-2026-10-23.md` (lines 1–125): Staged the one-page zero-update GPU pre-flight pre-registration sheet (`TEST FITNESS: PENDING`, all 8 required `prep.sheet_shape` blocks).

## Measurements

- Fixed synthetic chain fixture (`V = 512, B = 4, C = 8, K = 8` in `crates/uor-r4-discrete/src/address_cells.rs`):
  - `distinct_vectors`: `512 / 512` (`occupied_cells = 256 / 256`, `dev_alias_ceiling_rows = 512 / 512`, `within_cell_tie_mass = 0`).
  - Distinct distance levels across 256 cells on `q0`: `5` (`cell_outer_l1`) $\to$ `7` (`cell_envelope_l1` on `FullVectorV1`) $\to$ `15` (`full_vector_cell_l1` on `FullVectorV1`).
  - $K = 8$ boundary tie-bucket size on `q0`: `30` cells (`cell_outer_l1`) $\to$ `6` cells (`full_vector_cell_l1` on `FullVectorV1`).
  - Predecessor-query (`q = table[t/2]`) target cell admitted at $K = 8$ across `511` chain edges: `186 / 511` (`36.4%`, `cell_outer_l1`) $\to$ `349 / 511` (`68.3%`, `cell_envelope_l1`) $\to$ `446 / 511` (`87.3%`, `full_vector_cell_l1`).

## Checks run at the head

- `~/.cargo/bin/cargo test -p uor-r4-discrete --lib address_cells`: `11 passed; 0 failed`.
- `uor-harness loop report --check`: `PASS`.
- `uor-harness loop pr-check --local`: `PASS`.
- `uor-harness loop gates --full`: `11/11 GREEN`.

## Skills used

- `uor-research-architect`, `solution-integrity`, `scope-guard`, `gates`.

## Still open

- CUDA kernel dispatch for `cell_envelope_l1_v1` / `full_vector_cell_l1_v1` in `crates/uor-r4-discrete/src/discrete.cu` and finite CUDA qualification on a GPU pod, plus non-author `TEST FITNESS: FIT` and owner/PM registration on `#2834` for `docs/harness/prep/m1-address-key-full-vector-DRAFT-2026-10-23.md`. Item `m1-address-key-full-vector` stays `status: ready`. References #2834, #2029.

## Plain summary

This task delivers steps 3–5 of `m1-address-key-full-vector` (`#2834`): the integer-only, softmax-free, matmul-free full-vector / interior-block cell placement and address metrics (`AddressCellsMode::FullVectorV1`, `cell_envelope_l1`, `full_vector_cell_l1`) and their finite CPU controls (`11 / 11` pass) in `crates/uor-r4-discrete/src/address_cells.rs`, the design note in `docs/labs/m1-address-key-full-vector/README.md`, and the one-page pre-registration sheet `docs/harness/prep/m1-address-key-full-vector-DRAFT-2026-10-23.md`.
