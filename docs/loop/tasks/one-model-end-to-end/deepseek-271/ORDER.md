# ORDER: one-model-end-to-end, deepseek-271 (active controller test wiring)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Base: `455f183a1`.

## Goal

Make the merged bake-off leave the instrument crate testable on `main`. The
copy-all control intentionally fails four frozen rules, but auto-discovery runs
it as an ordinary test. Select the picked current controller as the active
implementation and auto-run only the scored V3 rules. V1/V2 test files remain
as frozen bake-off history.

## Cancel probe

Fresh `main` fails `cargo test --offline --features integer-reader` in
`tests/reply_control_bakeoff_rules.rs`: four assertions fail because the active
module is still the dumb control. The unit stands.

## Files in scope

- `docs/labs/one-model-end-to-end/instrument/Cargo.toml`
- `docs/labs/one-model-end-to-end/instrument/src/reply_control_controller.rs`
- `docs/loop/tasks/one-model-end-to-end/deepseek-271/**`

## Done when

1. The active controller is the current greedy/cycle rule selected by V3.
2. `autotests = false` and an explicit V3 test target prevent historical V1/V2
   and control-only rule failure from running as ordinary crate tests.
3. Full instrument `cargo test --features integer-reader` passes.
4. The V3 bake-off record still checks GREEN.
5. Clippy, report check and local PR check pass, or a pre-existing warning is
   named.

## Stop and report

Stop if changing test discovery alters the frozen V3 bake-off record.
