# ORDER: d41-driver-math-pilot, antigravity-58

Lab: antigravity. Item: [d41-driver-math-pilot](../../../../roadmap/items/d41-driver-math-pilot.md). Base: `53b40ecd6`.

## Goal

Restore the line-anchored `TEST FITNESS: FIT` verdict block in `docs/harness/prep/d41-driver-math-pilot-2026-10-25.md` (`#2821#issuecomment-6394217694`) that was omitted when PR `#3190` merged `2d83c7778` (`a50cbf564`) ahead of commit `4b59cd5b3`, causing `uor-r4-harness` unit test `prep::tests::the_real_sheets_are_readable` (`harness rules and tests` workflow) to fail on `origin/main` and on downstream PRs (`#3192`).

## Cancel probe (run first)

- Command: `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test -p uor-r4-harness --lib prep::tests::the_real_sheets_are_readable`
- Cancel if: `prep::tests::the_real_sheets_are_readable` already passes on `origin/main` (`53b40ecd6`).
- Probe result (`instrument`): At `53b40ecd6`, `prep::tests::the_real_sheets_are_readable` fails with `docs/harness/prep/d41-driver-math-pilot-2026-10-25.md: no verdict line found` because line 60 contains `Status: **FIT** (\`TEST FITNESS: FIT\`, 2026-10-25)` inline rather than the line-anchored `TEST FITNESS: FIT` verdict line parsed by `crates/uor-r4-harness/src/prep.rs`.

## Files in scope

- `docs/harness/prep/**`

## Done when

- `docs/harness/prep/d41-driver-math-pilot-2026-10-25.md` carries the line-anchored `TEST FITNESS: FIT` verdict block citing `#2821#issuecomment-6394217694`;
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test -p uor-r4-harness --lib prep::tests` passes (`the_real_sheets_are_readable` included);
- `uor-harness loop gates --full`, `uor-harness loop report --check`, and `uor-harness loop pr-check --local` pass.

## Stop and ask if

- Any owner-only path or driver source file would need to be modified.
