# REPORT: object-store-consumer-m3, antigravity-57

Lab: antigravity. Item: [object-store-consumer-m3](../../../../roadmap/items/object-store-consumer-m3.md) (References #2873). Under a fresh-context D68 decision review (`CHARTER.md` §5.1, `PLAN.md` M1→M2→M3 route, logged on `#2873` at `issuecomment-6394411611`), the item status is updated from `ready` to `held` pending M1/M2 model selection after repairing two generator instrument bugs in `m3-durable-memory-panel-v1.jsonl`, verifying all 384 items through `DurableObjectMemoryStore` + `UR4SESS1` against a paired `no-memory` negative control (`L-010`), recording the result row in `docs/research/notebook/m3-durable-memory.md`, and completing the non-author `TEST FITNESS: FIT` review (`issuecomment-6394411357`).

## What did not work, first

1. **Running the initial `deepseek-62` `m3-durable-memory-panel-v1.jsonl` (`01cb94df…`) through `DurableObjectMemoryStore` (`panel_executes_through_durable_consumer_and_beats_no_memory_control`):** Before applying the generator fix (`run-antigravity.txt:62`), running `cargo test -p uor-r4-object-store --test m3_durable_memory_panel` failed on two instrument defects in `crates/uor-r4-object-store/tests/m3_durable_memory_panel.rs`:
   - In `abstention_entry` (`mode == "retracted"`), Turn 0 asserted `(name, "hometown")` while Turn 1 called `"Forget {other}'s phone number."` (`owner = other`, `relation = "phone_number"`), which had never been asserted, causing `DurableObjectMemoryStore::retract_fact` to panic with `retract stated fact: SchemaMismatch` at `m3_durable_memory_panel.rs:478`.
   - In `correction_entry`, Turn 0 asserted `(name, "hometown") = city` while Turn 1 revised `(other, "hometown") = other_city` for a different owner `other = entity(i + 5).0`, leaving `previous_value_bytes = None` on both slots (`0/64` prior-lineage recall).
   Both were classed `instrument` (`CHARTER.md` §4.2, `L-007`) and repaired in `m3_durable_memory_panel.rs` so `correction_entry` revises the same `(name, "hometown")` slot (`64/64` current and `64/64` prior lineage) and `abstention_entry` (`retracted`) retracts the asserted `(name, "hometown")` slot (`32/32` `RefusedRetracted` + `96/96` `RefusedUnstated` = `128/128`).
2. **Stale mechanism status note in `docs/mechanisms/object-store.md:64`:** `docs/mechanisms/object-store.md` still stated `"export_scope and import_scope are unchanged and unwired"`, contradicting row 58 on the same page (`4797ca883`, `#2960` and `#3106`). We updated the `"What has never been tested"` section to state accurately that `durable_consumer` and `export_scope`/`import_scope` are built and consumer-verified on `m3-durable-memory-panel-v1`, while the end-to-end on-model `m3-memory` run awaits M1/M2 model selection.

## What changed

- [`crates/uor-r4-object-store/tests/m3_durable_memory_panel.rs`](../../../../crates/uor-r4-object-store/tests/m3_durable_memory_panel.rs) and [`docs/labs/object-store-consumer-m3/m3-durable-memory-panel-v1.jsonl`](../../../labs/object-store-consumer-m3/m3-durable-memory-panel-v1.jsonl): Repaired `correction_entry` (same-owner revision on `(name, "hometown")`) and `abstention_entry` (retracting the asserted `(name, "hometown")` slot), regenerated `m3-durable-memory-panel-v1.jsonl` (`179,096` bytes, whole-object SHA-256 `582e38798e66432a79fec340a9f87d6f4cb1bfd8c3d2ab594f8e6e305694f9d1`, folded `Manifest::version()` `88c27b94526c32b0f06fed7fabeb7a2abcf0b31a2f2534517b8656240eddce70`), and added `panel_executes_through_durable_consumer_and_beats_no_memory_control` executing all 384 items through `DurableObjectMemoryStore` + `PinnedTokenizer` + `UR4SESS1` against a paired `no-memory` negative control (`L-010`).
- [`docs/labs/object-store-consumer-m3/README.md`](../../../labs/object-store-consumer-m3/README.md), [`docs/mechanisms/object-store.md`](../../../mechanisms/object-store.md), and [`docs/research/notebook/m3-durable-memory.md`](../../../research/notebook/m3-durable-memory.md): Updated the frozen panel pins, recorded the 384-item consumer + `no-memory` control results (`77` passing `uor-r4-object-store` tests) as a new row in `docs/research/notebook/m3-durable-memory.md`, reconciled `docs/mechanisms/object-store.md`, and stamped the non-author `TEST FITNESS: FIT` review (`issuecomment-6394411357`).
- [`docs/roadmap/items/object-store-consumer-m3.md`](../../../roadmap/items/object-store-consumer-m3.md) and [`docs/roadmap/README.md`](../../../roadmap/README.md): Updated `status: ready` → `status: held` under a fresh-context D68 decision review (`issuecomment-6394411611`) pending M1/M2 model selection, and re-rendered `docs/roadmap/README.md`.

## Measurements

- **`m3_durable_memory_panel::panel_executes_through_durable_consumer_and_beats_no_memory_control` (384 items across 6 sub-panels):**
  - `provenance`: `64/64` exact byte-span + occurrence quote (`0/64` on paired `no-memory` control, `L-010`)
  - `correction`: `64/64` current value + `64/64` prior-version lineage (`0/64` on paired `no-memory` control, `L-010`)
  - `abstention`: `128/128` refusal (`32/32` `RefusedRetracted` + `96/96` `RefusedUnstated`)
  - `non_memory_prose` (`#1718`): `64/64` retained with `0` silent overwrites across `4..8` distractor turns per item
  - `reset_isolation`: `64/64` (`32/32` post-reset refusal + `32/32` cross-scope isolation)
  - `restart_restore` (`UORMEM01` + `UR4SESS1`): `64/64` byte-identical `UR4SESS1` snapshot and fact restore
- **Full `uor-r4-object-store` test suite:** `77 passed; 0 failed; 1 ignored`.
- **`uor-r4-integer --features serving-wasm --lib session_codec`:** `5 passed; 0 failed`.

## Checks run at the head

- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test -p uor-r4-object-store --no-fail-fast` -> `77 passed; 0 failed; 1 ignored`
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm --lib session_codec --no-fail-fast` -> `5 passed; 0 failed`
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run -p xtask -- roadmap check` -> `PASS`
- `uor-harness loop gates --full` -> `11 / 11 GREEN`
- `uor-harness loop report --check --order docs/loop/tasks/object-store-consumer-m3/antigravity-57/ORDER.md` -> `PASS`
- `uor-harness loop pr-check --local` -> `PASS`

## Skills used

- `uor-research-architect`: Diagnosed and repaired the two generator instrument defects in `m3-durable-memory-panel-v1` and built the 384-item `DurableObjectMemoryStore` + `UR4SESS1` evaluator with a paired `no-memory` negative control (`L-010`).
- `scope-guard`: Kept all changes strictly within `object-store-consumer-m3`'s declared scope (`crates/uor-r4-object-store/**`, `docs/labs/object-store-consumer-m3/**`, `docs/mechanisms/object-store.md`, plus always-allowed task folder, notebook row in `docs/research/notebook/m3-durable-memory.md`, item status line, and shared registration file `docs/roadmap/README.md`).
- `solution-integrity`: Verified whole-project milestone ordering (`PLAN.md` M1→M2→M3) via a fresh-context D68 decision review before setting `status: held`.

## Still open

- **On-model `m3-memory` evaluation (`#2873`):** Once M1 (`#2029`) and M2 (`#2030`) select a candidate integer stack artifact (`UORLUT01`), run the FIT'd 384-item `m3-durable-memory-panel-v1.jsonl` (`SHA-256` `582e38798e66432a79fec340a9f87d6f4cb1bfd8c3d2ab594f8e6e305694f9d1`) on the selected model against the owner-registered thresholds (`#2028#issuecomment-6356034555`).

## Plain summary

This unit repairs two generator instrument bugs in the 384-item M3 durable-memory panel (`m3-durable-memory-panel-v1.jsonl`), adds a deterministic 384-item `DurableObjectMemoryStore` + `UR4SESS1` evaluator paired with a `no-memory` negative control (`64/64` across all six sub-panels and `128/128` abstention refusals vs `0/64` on `no-memory`, `L-010`), records the result row in `docs/research/notebook/m3-durable-memory.md`, completes the non-author `TEST FITNESS: FIT` review (`issuecomment-6394411357`), and sets `docs/roadmap/items/object-store-consumer-m3.md` to `status: held` under a D68 decision review (`issuecomment-6394411611`) until an M1/M2 model artifact is selected for the on-model `m3-memory` run.
