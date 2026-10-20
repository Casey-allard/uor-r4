# Protocol v1 conformance vectors

Date: 2026-10-17. Lab: deepseek. Branch: deepseek/object-store-vectors. Follows #2596.

## What changed

- Added crates/uor-r4-object-store/tests/protocol_vectors.rs: a deterministic 96-byte object whose
  payload hex, SHA-256, canonical UOROBJ01 manifest hex, manifest version, lengths/roots, root record
  hex and first-leaf record hex are frozen as constants. A port of the protocol can reproduce these
  bytes exactly and use them as its first self-test.
- The test walks the tree from the root to the first leaf and validates the record bytes end to end.
  No production code or format changed.

## Verification

- protocol_vectors 1/1; full package suites unchanged (peer 15/15, objpeer_cli 8/8, controls 10/10,
  showcase 5/5); Clippy -D warnings and fmt clean.
- The vectors were generated from the current implementation and then frozen as assertions.

## Why it matters

The [specification](../object-store-protocol-spec-2026-10-17/README.md) says what the bytes are; these
vectors prove a port has reproduced them. Together they give a non-Rust agent (container/VM/MVM or a
registry service) a byte-for-byte conformance gate before it talks to a real store.
