# ORDER: d41-driver-math-pilot, antigravity-59

Lab: antigravity. Item: [d41-driver-math-pilot](../../../../roadmap/items/d41-driver-math-pilot.md). Base: `53b40ecd6`.

## Finished steps

- `bec81d644`: Added `docs/loop/tasks/d41-driver-math-pilot/antigravity-58/ORDER.md`.
- `3b62aaa5f`: Appended Section 7 (`TEST FITNESS: FIT`) to `docs/harness/prep/d41-driver-math-pilot-2026-10-25.md` and added `docs/loop/tasks/d41-driver-math-pilot/antigravity-58/REPORT.md`, fixing `prep::tests::the_real_sheets_are_readable` (`12 passed; 0 failed`).

## Goal

Fix the remaining `uor-harness check` (`prep.sheet_shape`) error on `docs/harness/prep/d41-driver-math-pilot-2026-10-25.md` (`missing block(s): Base artifact, Data split, Steps, Minimum test power`, run `40170831943`) so `uor-harness check` and the GitHub `harness rules and tests` workflow both exit `0` (`0 error`).

## Cancel probe (run first)

- Command: `uor-harness check`
- Cancel if: `uor-harness check` already reports `0 error` on `docs/harness/prep/d41-driver-math-pilot-2026-10-25.md`.
- Probe result (`instrument`): At `3b62aaa5f`, `uor-harness check` reports `1 error`: `error prep.sheet_shape [docs/harness/prep/d41-driver-math-pilot-2026-10-25.md] missing block(s): Base artifact, Data split, Steps, Minimum test power` because `crates/uor-r4-harness/src/prep.rs` checks section headings for the substrings `base`, `split`, `step`, and `power`.

## Files in scope

- `docs/harness/prep/**`

## Done when

- `docs/harness/prep/d41-driver-math-pilot-2026-10-25.md` satisfies all 8 `REQUIRED_BLOCKS` in `crates/uor-r4-harness/src/prep.rs` (`Bar`, `Base artifact`, `Data split`, `Steps`, `Stop rule`, `Cost`, `Minimum test power`, `TEST FITNESS line`);
- `uor-harness check` exits `0` (`0 error`) and `cargo test -p uor-r4-harness --lib prep::tests --no-fail-fast` passes (`12 passed; 0 failed`);
- `uor-harness loop gates --full`, `uor-harness loop report --check`, and `uor-harness loop pr-check --local` pass.

## Stop and ask if

- Any owner-only path or driver source file would need to be modified.
