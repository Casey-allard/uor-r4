# ORDER: object-store-consumer-m3, antigravity-12

Lab: antigravity. Item: [object-store-consumer-m3](../../../../roadmap/items/object-store-consumer-m3.md). Base: `ec980ee57`.

## Goal

The item's done-when reads:

> A consumer reads and writes through the object store on the model path, and the durable-memory bars run with recorded results. The M3 bar in [`docs/integration/project-track.md`](../../../../integration/project-track.md) is: on the selected M1/M2 artifact, `m3-memory` passes provenance, contradiction and correction, abstention on unstated or retracted facts, explicit reset and process restart. Saved and restored sessions must continue identically token-for-token (`IntegerStackSession` + `UR4SESS1`), and non-memory prose must not silently overwrite stored facts (#1718). A numeric threshold for each bar is not yet registered. The bars need a selected M1/M2 model, and M1 is not met and M2 is held, so the on-model run cannot start now. Registering the bars and building the consumer behind finite CPU controls can.

## Cancel probe (run first)

Ran `~/.cargo/bin/cargo test -p uor-r4-object-store` (67 tests pass: 1 unit, 6 `address_read`, 7 `bindings`, 10 `controls`, 22 `objpeer_cli`, 15 `peer`, 1 `protocol_vectors`, 5 `showcase`) and `~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm --lib session_codec` (4 tests pass). Confirmed via `grep -rn "export_scope\|import_scope" crates/` that `export_scope` and `import_scope` exist only in the historical `crates/uor-r4-core/src/native_geometric/durable_memory.rs` module outside the selected build, and nothing in `crates/uor-r4-object-store` or `crates/uor-r4-integer` wires `UR4SESS1` (`session_codec`) or scoped durable fact sessions through `uor-r4-object-store::Store` yet.

## Files in scope

- `crates/uor-r4-core/**`
- `crates/uor-r4-integer/**`
- `crates/uor-r4-object-store/**`
- `docs/mechanisms/exact-memory.md`
- `docs/mechanisms/object-store.md`
- `docs/labs/object-store-consumer-m3/**`

## Done when

1. Design note posted on `#2873` before writing code, naming the first consumer (durable-session persistence + occurrence-bound fact quotation over `uor-r4-object-store::Store`), specifying where exact identity meets geometry, naming who owns range reads (`RecordBackend` shared interface with `Store::range_from`), confirming that exact token, occurrence, and version identity stays separate from geometric addresses, and confirming zero softmax, zero matmul, and zero float on this path.
2. CHARTER §5.2 decision card posted on `#2028` for registering the M3 durable-memory bar thresholds and pre-registration sheet (`docs/labs/object-store-consumer-m3/README.md`).
3. Object-store durable consumer (`export_scope` / `import_scope` + `UR4SESS1` session persistence + versioned fact assertion, revision, retraction, non-memory prose protection `#1718`, explicit reset, and occurrence-bound quotation) implemented in `crates/uor-r4-object-store` and wired into `crates/uor-r4-integer` (`serving-wasm` `IntegerStackSession` / `StackChat` durable object-store hooks), with finite CPU controls in `crates/uor-r4-object-store/tests/durable_consumer.rs` covering write-then-read byte-identical, restart-then-restore byte-identical (`UR4SESS1`), retracted or unstated fact refused rather than guessed, non-memory prose non-overwrite (`#1718`), explicit reset, and hostile/corrupt record refused.
4. `docs/mechanisms/object-store.md`, `docs/mechanisms/exact-memory.md`, and `docs/labs/object-store-consumer-m3/README.md` updated, and `uor-harness loop gates --full`, `uor-harness loop report --check`, and `uor-harness loop pr-check --local` pass.

## Stop and ask if

- Any file outside `scope` needs modification.
- GPU or on-model M3 bar execution is required before M1/M2 selection, a non-author `TEST FITNESS: FIT` review, and a typed owner release.
