# REPORT: object-store-consumer-m3, antigravity-12

References #2873.

## What did not work, first

1. **Missing `durable_consumer` module and ByteLevel BPE fixture merges on initial TDD test run:**
   - Executing the finite CPU control suite (`~/.cargo/bin/cargo test -p uor-r4-object-store --test durable_consumer`) before implementing the consumer failed to compile with `error[E0432]: unresolved import uor_r4_object_store::durable_consumer`.
   - On the first test run after adding `durable_consumer.rs`, `ByteBpeTokenizer::from_tokenizer_json_bytes` returned `None` (`panicked at crates/uor-r4-object-store/tests/durable_consumer.rs:37:10: valid fixture tokenizer`) because the `ByteLevel` pre-tokenizer maps ASCII space (`0x20`) to `"Ġ"` rather than `" "` and requires every intermediate merge token (`"Al"`, `"Ali"`, `"Alic"`, `"Alice"`, `"Li"`, `"Lis"`, `"Lisb"`, `"Lisbo"`, `"Lisbon"`, `"80"`, `"8080"`) to appear explicitly in `model.vocab`. Adding `"Ġ"` and the merged vocabulary entries resolved the tokenizer fixture failure and all 6 controls passed.
2. **On-model M3 `m3-memory` qualification cannot run yet:**
   - The M3 durable-memory bars in `docs/integration/project-track.md` require a selected M1/M2 artifact, ratified numeric thresholds, a non-author `TEST FITNESS: FIT` review, and a typed owner release. Because M1 is not yet met and M2 is HELD, only the object-store consumer (`export_scope` / `import_scope` + `UR4SESS1` + occurrence-bound fact quotation), the 6 finite CPU controls, and the M3 bar pre-registration + CHARTER §5.2 decision card can land in this PR.

## What changed

- [`crates/uor-r4-object-store/src/durable_consumer.rs`](../../../../../crates/uor-r4-object-store/src/durable_consumer.rs) and [`crates/uor-r4-object-store/src/lib.rs`](../../../../../crates/uor-r4-object-store/src/lib.rs):
  - Implemented `DurableObjectMemoryStore`, `IdentityScope`, `FactAction` (`Assert`, `Revise`, `Retract`), `FactProvenance`, `FactVersionRecord`, `FactReadOutcome` (`Quoted`, `RefusedUnstated`, `RefusedRetracted`), and `Ur4Sess1HeaderView` using exact integer binary framing (`UORMEM01` + `UR4SESS1`) over `Store`, `Manifest`, and `Bindings` (`UORBND01`).
  - Wired `export_scope` (`fold` of canonical `UORMEM01` envelope into `Store`) and `import_scope` / `import_scope_expecting` (`Store::rebuild` whole-object SHA-256 verification and state restoration), `assert_fact`, `revise_fact`, `retract_fact`, `ingest_non_memory_prose` (`#1718`), `read_fact` / `read_fact_from` (zero backend fetches on unstated or retracted queries), and `reset_scope`.
- [`crates/uor-r4-integer/src/session_codec.rs`](../../../../../crates/uor-r4-integer/src/session_codec.rs):
  - Added `ScopedSessionIdent`, `export_scope`, and `import_scope` producing and decoding the same `UORMEM01` binary envelope wrapping `UR4SESS1` (`SerializedStackSession`), plus unit test `export_and_import_scope_round_trip_and_refuse_corrupt_envelope`.
- [`crates/uor-r4-object-store/tests/durable_consumer.rs`](../../../../../crates/uor-r4-object-store/tests/durable_consumer.rs):
  - Added 6 finite CPU integration controls verifying byte-identical write/read and occurrence quotation, process restart and restore with `UR4SESS1`, unstated/retracted fact refusal with zero backend fetches, `#1718` non-memory prose non-overwrite, explicit reset and `IdentityScope` isolation, and fail-closed refusal of corrupt/hostile records, manifests, and envelopes.
- [`docs/labs/object-store-consumer-m3/README.md`](../../../../labs/object-store-consumer-m3/README.md), [`docs/mechanisms/object-store.md`](../../../../mechanisms/object-store.md), and [`docs/mechanisms/exact-memory.md`](../../../../mechanisms/exact-memory.md):
  - Recorded the resolved open design decisions, the 6 CPU controls, and the drafted M3 `m3-memory` pre-registration sheet.

## Measurements

- `crates/uor-r4-object-store`: **73 passed, 0 failed** (up from 67 baseline tests; all 6 new controls in `tests/durable_consumer.rs` pass).
- `crates/uor-r4-integer --features serving-wasm --lib session_codec`: **5 passed, 0 failed** (up from 4 baseline tests).
- Unstated and retracted fact queries perform **0** `RecordBackend::fetch` calls (`CountingBackend` verified).
- Non-memory prose turns (`#1718`): **0** silent overwrites and **0** spurious conflicts across distractor turns sharing entity tokens.

## Checks run at the head

- `~/.cargo/bin/cargo fmt -p uor-r4-object-store -p uor-r4-integer -- --check`
- `~/.cargo/bin/cargo test -p uor-r4-object-store`
- `~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm --lib session_codec`
- `uor-harness loop gates --full`
- `uor-harness loop report --receipts`
- `uor-harness loop report --check`
- `uor-harness loop pr-check --local`

## Skills used

- `uor-research-architect`: Checked D11, D19, D23, D30, and D32 boundaries so exact token, occurrence, and version identity stays in `uor-r4-object-store` without adding float, softmax, or matmul.
- `scope-guard`: Kept all changes strictly within `object-store-consumer-m3`'s declared `scope`.
- `solution-integrity`: Verified fail-closed behavior on corrupt records, forged manifests, truncated envelopes, and cross-scope imports.

## Still open

- Ratification of the M3 `m3-memory` bar thresholds on the CHARTER §5.2 decision card (`https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6339008955`), selection of an M1/M2 model artifact, a non-author `TEST FITNESS: FIT` review of the pre-registration in `docs/labs/object-store-consumer-m3/README.md`, and a typed owner release before running `m3-memory` on a trained model (tracked on `#2873`).

## Plain summary

Wired `uor-r4-object-store` and `uor-r4-integer`'s `UR4SESS1` session codec together through a pure-integer durable-memory consumer (`export_scope` / `import_scope`, occurrence-bound fact quotation, unstated/retracted refusal, `#1718` non-memory prose protection, and explicit scope reset), verified all 6 finite CPU controls (`73/73` object-store tests and `5/5` session codec tests passing), and drafted the M3 durable-memory bar pre-registration and CHARTER §5.2 threshold decision card.
