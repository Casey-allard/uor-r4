# ORDER: object-store-consumer-m3, deepseek-60 (cross-crate interop control for the durable consumer)

Lab: deepseek. Item: [object-store-consumer-m3](../../../../roadmap/items/object-store-consumer-m3.md) (#2873). Base: `127632bc3`. Partial unit: `References #2873`; the item stays `ready` (the durable-memory bars still need a selected model, a FIT and Casey's release).

## Goal

Close the recorded drift risk from the post-merge observation on #2873 (2026-10-22T11:40Z): the object-store durable-consumer tests build `UR4SESS1` bytes by hand, so a framing change in either crate would not fail the other, and the two crates implement the `UORMEM01` framing independently. Add finite CPU interop controls that feed the integer codec's real bytes through the object store's durable consumer and back, assert the restored typed state and byte identity, and pin the shared magic/version identity across crates. No model, no bars, no compute.

## Cancel probe (run first)

At base `127632bc3`: `crates/uor-r4-object-store/tests/durable_consumer.rs` builds `UR4SESS1` bytes with its local `sample_ur4sess1_bytes` helper; `crates/uor-r4-object-store/Cargo.toml` has no `uor-r4-integer` dev-dependency; nothing in the object-store tests parses bytes produced by `uor_r4_integer::session_codec`. Task stands (the drift risk is real and recorded on #2873).

## Files in scope

- `crates/uor-r4-object-store/tests/integer_session_interop.rs` (new)
- `crates/uor-r4-object-store/tests/fixtures/integer_ur4sess1_v1.hex` (new contract fixture, hex text; no binary committed)
- `crates/uor-r4-integer/tests/session_codec_interop_fixture.rs` (new, `serving-wasm`-gated)
- this task folder

## Done when

- A committed contract fixture (337-byte payload as hex text, sha256 `ef0ffeee…`) is byte-identical to the integer codec's `encode` output for the fixture state and to what the store returns after `set -> export -> import`; `Ur4Sess1HeaderView::parse` accepts it; the same digest constant is asserted on both crate sides, so drift on either side fails a test.
- Cross-crate constants agree: both crates' `UORMEM01` magic/version are asserted against the contract (`b"UORMEM01"`, 1); the integer `export_scope` envelope embeds the exact snapshot bytes and round-trips through `import_scope`.
- A mutation check shows the integer-side test fails if the framing drifts (magic perturbed), then the tree is restored; a cross-crate dev-dependency is deliberately avoided because it would change the root `Cargo.lock`, which is outside this item's scope, whereas the fixture+digest pin exercises the same bytes with no dependency edge.
- `cargo test -p uor-r4-object-store` is green (baseline 60 tests, plus the new target); `loop report --check`, `loop pr-check --local`, and `loop gates --full` pass; a counted non-deepseek review records at the exact head.

## Stop and ask if

- A model run, bar registration, FIT or owner release is needed (out of scope here); a file outside the three listed paths is needed.