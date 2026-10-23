# REPORT: object-store-consumer-m3, deepseek-60 (cross-crate interop control for the durable consumer)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #2873. Base: `127632bc3`. `References #2873`; item stays `ready`.

## What did not work, first

- **The delivered consumer has two independent framing implementations.** The integer crate's `session_codec` writes `UORMEM01` envelopes with the snapshot and scopes; the object-store's `durable_consumer` writes its own scope-state envelope. The object-store tests hand-build `UR4SESS1` bytes (`sample_ur4sess1_bytes`), so no test would fail if the model-side bytes changed. The recorded observation on #2873 named exactly this; this unit adds the missing control rather than a new framing.
- **A cross-crate dev-dependency was rejected for the same reason the pin was needed.** Adding `uor-r4-integer` to the object store's dev-dependencies changes the workspace root `Cargo.lock`, which is outside this item's scope (and a `--locked` build refuses a stale lock). The committed fixture + shared digest achieves the same interop pin with no dependency edge and no lock change; the object-store side reads the fixture bytes and the integer side re-derives them with the codec.
- **A mutation check was needed to prove the test is not vacuous.** It perturbs the integer `UORMEM01` magic in a scratch copy and expects the integer-side contract test to fail; then the tree is restored, so the control demonstrably catches the drift it was written for.

## What changed

| File | Change |
|---|---|
| `crates/uor-r4-object-store/tests/fixtures/integer_ur4sess1_v1.hex` (new) | 337 B canonical contract fixture produced by `uor_r4_integer::session_codec::encode` for the fixture state; sha256 `ef0ffeee…` |
| `crates/uor-r4-object-store/tests/integer_session_interop.rs` (new) | fixture digest + header pin; `set -> export -> import -> ur4sess1_snapshot` returns the exact fixture bytes; store-side `UORMEM01` identity pinned |
| `crates/uor-r4-integer/tests/session_codec_interop_fixture.rs` (new, `serving-wasm`) | `encode(sample)` equals the committed fixture byte-for-byte and by sha256; fixture decodes back to the typed state; integer envelope identity/embedding pinned |
| this folder | ORDER and REPORT |

## Measurements

| Command | Result |
|---|---|
| `cargo test -p uor-r4-object-store --test integer_session_interop` | 2 passed, 0 failed (new target) |
| `cargo test -p uor-r4-integer --features serving-wasm --test session_codec_interop_fixture` | 2 passed, 0 failed (new target) |
| `cargo test -p uor-r4-object-store` | baseline 60 + new target, all green |
| mutation: integer `UORMEM01` magic/version perturbed in a scratch copy | new interop test **fails** (drift caught); tree restored |

## Checks run at the head

`loop report --check`, `loop pr-check --local`, and `loop gates --full` (see closing line); all changed paths are inside the item's scope; no model, GPU, bar or training change.

## Skills used

- `solution-integrity`: the test was mutation-checked so it provably catches the framing drift it names, and the unit does not claim a stronger envelope interop than the public API allows.
- `scope-guard`: two new test files, one committed hex fixture and this folder only; no `Cargo.toml`, `Cargo.lock` or existing-source change.

## Still open

- The durable-memory bars still need a selected M1/M2 model, a frozen panel + pre-registration, a non-author FIT and Casey's typed release; the item stays `ready` and nothing was run on a model.
- The recorded "which consumer owns range reads" decision is unchanged; this unit adds tests, not a new consumer.

## Plain summary

- Added finite CPU controls that run the real integer session bytes through the object store's durable consumer and back, and pin the shared envelope identity, so a framing change in either crate now fails a test instead of silently drifting. The item's actual bars still need a model and a release; this is instrument evidence only.