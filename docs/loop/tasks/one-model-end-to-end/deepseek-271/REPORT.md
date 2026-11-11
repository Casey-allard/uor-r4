# REPORT: one-model-end-to-end, deepseek-271 (active controller test wiring)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Laptop only; no model or pod.

**In one line:** fresh `main` now runs the picked current controller and only
the scored V3 bake-off rules as ordinary tests.

## What did not work, first

- Fresh-main `cargo test --features integer-reader` failed because Cargo
  auto-discovered every bake-off rule file. The intentionally failing copy-all
  control ran as an ordinary test and failed four rules.
- V1 and V2 rule files are historical bake-off inputs, not the active crate's
  test suite.

## What worked

- `reply_control_controller` now holds the V3 pick: current greedy/cycle.
- `autotests = false` stops Cargo from treating frozen V1/V2/control-era files
  as ordinary tests.
- An explicit `reply_control_bakeoff_v3_rules` target runs the picked rule set.
- The V3 bake-off record still checks GREEN because its frozen files and
  candidate branches are unchanged.

## What changed

- `instrument/Cargo.toml` disables auto-tests and registers the V3 test target.
- `src/reply_control_controller.rs` selects current greedy/cycle.
- This ORDER and REPORT record the packaging repair.

## Measurements

- Fresh-main failure: four of eight V1 control-era rules ran and failed.
- Repaired crate: 23 tests passed, 0 failed; V3 runs eight of eight.
- Bake-off record: unchanged and GREEN.

## Checks run at the head

- `cargo test --offline --features integer-reader`: 23 passed, 0 failed
  (6 corpus-EOS, 2 exclusions, 7 raw-id, 8 V3 rules; two empty harnesses).
- `cargo clippy --offline --features integer-reader --all-targets -- -D warnings`:
  pass.
- Bake-off V3 check: GREEN.
- `git diff --check`: pass.
- Report and local PR checks run at the committed head.

## Skills used

`bake-off` defined the distinction between frozen control-era test history and
the selected rule set. `solution-integrity` treated the fresh-main test failure
as a packaging defect, not a candidate result.

## Still open

- No further controller work. The bake-off result remains current-rule
  retention on one arm and seed.

## Plain summary

The bake-off tests were accidentally running as normal tests after merge,
including the dumb control that is supposed to fail. The picked controller is
now active, only the final V3 rules run by default, and the whole instrument
crate is green again.
