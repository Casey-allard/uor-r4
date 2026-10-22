# ORDER: m1-address-as-read, antigravity-9

Lab: antigravity. Item: [m1-address-as-read](../../../../roadmap/items/m1-address-as-read.md). Base: `60a0c9cda`.

## Goal

Repair PR #2950 per DeepSeek's exact-head review (`https://github.com/UOR-Foundation/uor-r4/pull/2950#issuecomment-6336410319`). The item's done-when reads:

> Code on main; finite CPU tests green; the serving-time address oracle beats the 19/40 baseline on the frozen panel by the pre-registered bar (and only then, if needed, a trained form is pre-registered and run); one row in `docs/research/notebook/m3-durable-memory.md`.

## Cancel probe (run first)

Inspected DeepSeek's review on PR #2950 (`#2950#issuecomment-6336410319`) and ran `uor-harness ratchet d23` in `.worktrees/antigravity-m1-address-as-read`:
1. `crates/uor-r4-object-store/src/address_read.rs:7` contains the prohibited D23 token in a module doc comment (`Uses zero floating point, zero ...`), raising `crates/uor-r4-object-store/src/address_read.rs: 0 -> 1 (+1)` in `uor-harness ratchet d23`.
2. `docs/labs/m1-address-as-read/README.md` and `docs/loop/tasks/m1-address-as-read/antigravity-8/REPORT.md` state differing test counts (`66` vs `67`) without listing the per-target breakdown (`cargo test -p uor-r4-object-store` runs `61` tests across the 6 compiled test targets `lib` 1, `address_read` 6, `bindings` 7, `controls` 10, `objpeer_cli` 22, `peer` 15, plus `4` in `protocol_vectors` and `2` in `showcase` when examples/all targets are included).
Still needed: remove the D23 word from `crates/uor-r4-object-store/src/address_read.rs:7`, verify `uor-harness ratchet d23` and `uor-harness ratchet d25` are GREEN, align the test counts and per-target breakdown in `docs/labs/m1-address-as-read/README.md` and `docs/loop/tasks/m1-address-as-read/antigravity-9/REPORT.md`, and re-run `loop gates --full` and `loop pr-check --local`.

## Files in scope

- `crates/uor-r4-object-store/**`
- `docs/mechanisms/exact-memory.md`
- `docs/labs/m1-address-as-read/**`
- `docs/harness/prep/m1-address-as-read*`

## Done when

- `crates/uor-r4-object-store/src/address_read.rs` contains zero D23 or D25 ratchet tokens, and `uor-harness ratchet d23` and `uor-harness ratchet d25` both pass with 0 added sites.
- `docs/labs/m1-address-as-read/README.md` and `docs/loop/tasks/m1-address-as-read/antigravity-9/REPORT.md` state the exact `cargo test -p uor-r4-object-store` test count (`61 passed: 55 existing + 6 new in tests/address_read.rs`, with per-target breakdown `lib 1, address_read 6, bindings 7, controls 10, objpeer_cli 22, peer 15`, plus `protocol_vectors 4` and `showcase 2` under `--examples`).
- `uor-harness loop gates --full` and `uor-harness loop pr-check --local` pass.

## Stop and ask if

- Any file outside the declared `scope:` must be edited.
- Any GPU pod execution is needed.
- Any charter rule, milestone bar, hold, or spend cap is implicated.
