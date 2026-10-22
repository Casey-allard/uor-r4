# REPORT: m1-normalization-mechanism, antigravity-13

Lab: antigravity. Item: [m1-normalization-mechanism](../../../../roadmap/items/m1-normalization-mechanism.md). Order: [ORDER.md](ORDER.md).

References #2842.

## What did not work, first

- **Three-level relative links (`../../../`) in `docs/loop/tasks/m1-normalization-mechanism/antigravity-12/REPORT.md`:** Because task folders sit four levels under `docs/` (`docs/loop/tasks/<item>/<lab>-<n>/`), the 6 relative links to `docs/harness/prep/...`, `docs/labs/...`, `docs/model/...`, `docs/geometry/...`, and `docs/mechanisms/...` in `antigravity-12/REPORT.md` resolved one directory too shallow (`docs/loop/...`) and produced 6 new broken links in CI's baseline-compared `xtask links` gate (caught by Kimi's `CHANGES` review at `77780020c9de35e079a7761d4c19a28bf26ac75d`).
- **Un-normalized multi-stage residual accumulation (`UnnormalizedControlV1`) and un-scaled `snap_1` (`UnitSnapControlV1`):** Across $S = 16$ sequential residual stages ($x^{(s)} = x^{(s-1)} + \rho_{r_s}$), block $L_1$ norms drift linearly to $\|x^{(S)}\|_1 = 68$ and $\|x^{(S)}\|_\infty = 34$, pushing endpoint cell-admission distances (`endpoint_address_distance`) above $\{0, 4, 8, 12, 16\}$; un-scaled `snap_1` on $3\rho_{r_0} + \rho_{r_{\text{orth}}}$ ties on $L_1$ distance ($12 = 12$) and flips to the 1-vote minority root $r_{\text{orth}}$ by Hamming distance ($2 < 4$), whereas multiplicity-matched `snap_4` ($n = 4$) strictly selects the 3-vote majority root `0` ($L_1 = 8 < 24$).
- **GPU trunk training (`N_snap` vs. `C_unscaled` and `C_raw`):** Not run in this CPU-scoped unit; the Stage 3 trunk (`m1-trunk-assembly-and-training`) is not yet assembled and GPU runs require a non-author `TEST FITNESS: FIT` review of `docs/harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md`, owner confirmation of the `#2028` bar decision card, and a typed D40 owner release.

## What changed

- **Repair in `antigravity-13`:**
  - Corrected all 6 relative links in [`docs/loop/tasks/m1-normalization-mechanism/antigravity-12/REPORT.md`](../antigravity-12/REPORT.md) from `../../../` to `../../../../` (`../../../../harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md`, `../../../../labs/m1-normalization-mechanism/README.md`, `../../../../model/architecture.md`, `../../../../geometry/e8-h4-and-the-600-cell.md`, `../../../../geometry/quaternions-and-rotation.md`, and `../../../../mechanisms/hamilton-gate.md`) and verified zero new broken links against `origin/main` using `cargo run -q --locked -p xtask -- links --baseline <base-links.json> --renames-from origin/main`.
- **Discrete normalization module (`crates/uor-r4-discrete/src/normalization_mechanism.rs`, wired in `crates/uor-r4-discrete/src/lib.rs`):**
  - Implemented `NormalizationMode` (`ScaleMatchedSnapV1`, `UnitSnapControlV1`, `UnnormalizedControlV1`), `BlockNormInvariants`, `SnapProjection`, `NormFrameTable`, `root_scaled_cost`, `normalize_block_with_frame`, `normalize_block_mode`, `normalize_carrier_vector`, and `endpoint_address_distance` over `geometry.rs` tables (`ROOTS`, `AXES`, `GENERATORS`, `IDENTITY`), using pure integer arithmetic (`0` floats, `0` smooth gates/exponentials, `0` dense matmuls) and 6 finite CPU unit tests.
- **Pre-registration draft, lab report, and architecture/geometry docs:**
  - Added [`docs/harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md`](../../../../harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md) (`TEST FITNESS: PENDING`) and [`docs/labs/m1-normalization-mechanism/README.md`](../../../../labs/m1-normalization-mechanism/README.md).
  - Updated [`docs/model/architecture.md`](../../../../model/architecture.md), [`docs/geometry/e8-h4-and-the-600-cell.md`](../../../../geometry/e8-h4-and-the-600-cell.md), [`docs/geometry/quaternions-and-rotation.md`](../../../../geometry/quaternions-and-rotation.md), and [`docs/mechanisms/hamilton-gate.md`](../../../../mechanisms/hamilton-gate.md).

## Measurements

- `~/.cargo/bin/cargo test -p uor-r4-discrete --lib normalization_mechanism` -> `6 passed; 0 failed; 0 ignored` (`209 passed` across full `uor-r4-discrete --lib`).
- `~/.cargo/bin/cargo run -q --locked -p xtask -- links --baseline /tmp/antigravity-m1-norm-base-links.json --renames-from origin/main` -> `0` new broken links vs `origin/main`.

## Checks run at the head

- `~/.cargo/bin/cargo test -p uor-r4-discrete --lib normalization_mechanism` -> `6 passed; 0 failed; 0 ignored`.
- `~/.cargo/bin/cargo run -q --locked -p xtask -- links --baseline /tmp/antigravity-m1-norm-base-links.json --renames-from origin/main` -> `0` new broken links.
- `uor-harness loop gates --full` and `uor-harness loop report --receipts` -> 6 gates `GREEN`, receipts recorded in [`receipts.jsonl`](receipts.jsonl).
- `uor-harness loop report --check` and `uor-harness loop pr-check --local` -> `PASS`.

## Skills used

- `uor-research-architect`
- `solution-integrity`

## Still open

- Non-author `TEST FITNESS: FIT` review of `docs/harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md`, owner decision on the `#2028` bar registration card (`https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6338596024`), trunk assembly (`m1-trunk-assembly-and-training`), and a typed D40 owner release before running `N_snap`, `C_unscaled`, and `C_raw` on GPU (tracked on `#2842`).

## Plain summary

Repaired the 6 relative markdown links in `docs/loop/tasks/m1-normalization-mechanism/antigravity-12/REPORT.md` (`../../../` $\to$ `../../../../`) caught by Kimi's `CHANGES` review on PR `#2958`, and verified zero new broken links against `origin/main` alongside the `OrbitRootSnapNormV1` discrete normalization module, its 6 finite CPU controls, and its pre-registration sheet.
