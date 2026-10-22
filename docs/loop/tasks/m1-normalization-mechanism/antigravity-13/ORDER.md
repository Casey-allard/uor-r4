# ORDER: m1-normalization-mechanism, antigravity-13

Lab: antigravity. Item: [m1-normalization-mechanism](../../../../roadmap/items/m1-normalization-mechanism.md). Base: `77780020c`.

## Goal

Repair the 6 broken relative markdown links in `docs/loop/tasks/m1-normalization-mechanism/antigravity-12/REPORT.md` identified by Kimi's `CHANGES` review on PR `#2958` (`https://github.com/UOR-Foundation/uor-r4/pull/2958#issuecomment-6339039690`), and verify zero new broken links via `cargo run -q --locked -p xtask -- links --baseline <base-links.json> --renames-from origin/main`.

## Cancel probe (run first)

Ran `uor-harness loop review --check --pr 2958`, which reported `comment 2: [kimi] CHANGES at 77780020c9de35e079a7761d4c19a28bf26ac75d on PR 2958: counted`, and verified the 6 three-level relative paths (`../../../`) in `docs/loop/tasks/m1-normalization-mechanism/antigravity-12/REPORT.md` lines 20–21 require four levels (`../../../../`) from `docs/loop/tasks/m1-normalization-mechanism/antigravity-12/`.

## Files in scope

- `crates/uor-r4-discrete/**`
- `docs/geometry/e8-h4-and-the-600-cell.md`
- `docs/geometry/quaternions-and-rotation.md`
- `docs/harness/prep/m1-normalization-mechanism*`
- `docs/mechanisms/hamilton-gate.md`
- `docs/model/architecture.md`
- `docs/labs/m1-normalization-mechanism/**`

## Done when

1. All 6 relative links in `docs/loop/tasks/m1-normalization-mechanism/antigravity-12/REPORT.md` use `../../../../` and resolve cleanly.
2. `docs/loop/tasks/m1-normalization-mechanism/antigravity-13/REPORT.md` records the repair and the baseline-compared `xtask links` verification (`0` new broken links vs `origin/main`).
3. `uor-harness loop gates --full`, `uor-harness loop report --receipts`, `uor-harness loop report --check`, and `uor-harness loop pr-check --local` pass.

## Stop and ask if

- Any file outside the declared scope needs modification.
