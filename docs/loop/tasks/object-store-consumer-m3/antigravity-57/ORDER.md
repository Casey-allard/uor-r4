# ORDER: object-store-consumer-m3, antigravity-57

Lab: antigravity. Item: [object-store-consumer-m3](../../../../roadmap/items/object-store-consumer-m3.md). Base: `a50cbf564`.

## Goal

The item's done-when reads:

> A consumer reads and writes through the object store on the model path, and the durable-memory bars run with recorded results. The M3 bar in [`docs/integration/project-track.md`](../../integration/project-track.md) is: on the selected M1/M2 artifact, `m3-memory` passes provenance, contradiction and correction, abstention on unstated or retracted facts, explicit reset and process restart. Saved and restored sessions must continue identically token-for-token (`IntegerStackSession` + `UR4SESS1`), and non-memory prose must not silently overwrite stored facts (#1718). A numeric threshold for each bar is not yet registered. The bars need a selected M1/M2 model, and M1 is not met and M2 is held, so the on-model run cannot start now. Registering the bars and building the consumer behind finite CPU controls can.

## Cancel probe (run first)

- Command:
  - `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test -p uor-r4-object-store --no-fail-fast`
  - `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm --lib session_codec --no-fail-fast`
- Cancel if: the existing `uor-r4-object-store` (`76` tests) or `uor-r4-integer` (`session_codec`, `5` tests) suites fail on host.
- Probe result (2026-10-25, classed `instrument`): PASS on existing unit tests (`76 passed; 0 failed; 1 ignored` in `uor-r4-object-store`, `5 passed; 0 failed` in `uor-r4-integer::session_codec`), with two `instrument` defects discovered when auditing `crates/uor-r4-object-store/tests/m3_durable_memory_panel.rs` against `DurableObjectMemoryStore` (`crates/uor-r4-object-store/src/durable_consumer.rs`):
  1. `correction_entry` (`m3_durable_memory_panel.rs:73-89`) asserts `(name, "hometown") = city` in Turn 0 and revises `(other, "hometown") = other_city` for a different entity `other = entity(i + 5).0` in Turn 1, leaving `previous_value_bytes = None` on both slots (`0/64` prior-lineage recall under `DurableObjectMemoryStore`).
  2. `abstention_entry` (`m3_durable_memory_panel.rs:91-114`, `mode == "retracted"`) calls `Forget {other}'s phone number` in Turn 1 without ever asserting `(other, "phone_number")` in Turn 0, causing `DurableObjectMemoryStore::retract_fact` to fail with `Err(Error::SchemaMismatch)` on all `32` retracted items.
  3. Neither a deterministic 384-item `m3-durable-memory-panel-v1` consumer + paired `no-memory` negative-control evaluator (`L-010`) nor the `docs/research/notebook/m3-durable-memory.md` result row or non-author `TEST FITNESS: FIT` review has been recorded yet, and `docs/mechanisms/object-store.md:64` still states that `export_scope` and `import_scope` are unwired.

## Files in scope

- `crates/uor-r4-core/**`
- `crates/uor-r4-integer/**`
- `crates/uor-r4-object-store/**`
- `docs/mechanisms/exact-memory.md`
- `docs/mechanisms/object-store.md`
- `docs/labs/object-store-consumer-m3/**`

## Done when

- The two `m3-durable-memory-panel-v1` instrument defects (`correction_entry` split-owner revision and `abstention_entry` unasserted retraction) are shown to fail a new 384-item `DurableObjectMemoryStore` + `UR4SESS1` panel evaluator first and pass after repair (`64/64` provenance, `64/64` correction current + `64/64` prior lineage, `128/128` abstention refusal with R14 `AnsweredFactHitRate = 1.00` and `SelectiveAbstentionGap = 1.00`, `64/64` `#1718` non-memory prose, `64/64` reset & scope isolation, `64/64` `UR4SESS1` restart/restore, vs `0/64` recall on the paired `no-memory` control `L-010`); `docs/labs/object-store-consumer-m3/m3-durable-memory-panel-v1.jsonl` and `docs/labs/object-store-consumer-m3/README.md` are updated and receive a fresh-context non-author `TEST FITNESS: FIT` review posted on `#2873`; `docs/mechanisms/object-store.md` and `docs/research/notebook/m3-durable-memory.md` are updated with the measured result row; and under a fresh-context D68 decision review (`CHARTER.md` §5.1, `PLAN.md` M1→M2→M3 route), `docs/roadmap/items/object-store-consumer-m3.md` is set to `status: held` pending M1/M2 model selection for the on-model M3 bar run.

## Stop and ask if

- Any owner-only path (`CHARTER.md`, `AGENTS.md`, `docs/integration/DECISIONS.md`, `docs/harness/owner-paths.json`) or `Cargo.lock` dependency change would be required.
