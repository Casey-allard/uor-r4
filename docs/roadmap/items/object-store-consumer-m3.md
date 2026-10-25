---
id: object-store-consumer-m3
title: Wire the object store into the model and run the durable-memory bars
milestone: M3
area: architecture
status: held
open_to: anyone
needs_gpu: false
depends_on: []
scope: [crates/uor-r4-core/**, crates/uor-r4-integer/**, crates/uor-r4-object-store/**, docs/mechanisms/exact-memory.md, docs/mechanisms/object-store.md, docs/labs/object-store-consumer-m3/**]
---

## Why it matters

The object store is the project's exact, hashed durable storage, and M3 (durable conversation memory) is the milestone that needs it. Today nothing reads or writes it from the model path, so the memory bars cannot run. A consumer also gives later replies exact bytes to quote from. See [the transformer-to-geometry map](../../start/transformer-vs-geometry.md) and [the object store page](../../mechanisms/object-store.md).

## What is missing

The object store (D19) keeps bytes exactly in a hashed tree and has 60 tests and a client conformance pass. The durable-memory consumer and its finite controls are delivered (`4797ca883`, `#2960`), including `export_scope`/`import_scope`, and a cross-crate interop contract now pins the integer session bytes against the store (`#3106`). The durable-memory bars remain unrun because they need a selected M1/M2 model, a non-author FIT and the owner release.

## Done when

A consumer reads and writes through the object store on the model path, and the durable-memory bars run with recorded results. The M3 bar in [`docs/integration/project-track.md`](../../integration/project-track.md) is: on the selected M1/M2 artifact, `m3-memory` passes provenance, contradiction and correction, abstention on unstated or retracted facts, explicit reset and process restart. Saved and restored sessions must continue identically token-for-token (`IntegerStackSession` + `UR4SESS1`), and non-memory prose must not silently overwrite stored facts (#1718). A numeric threshold for each bar is not yet registered. The bars need a selected M1/M2 model, and M1 is not met and M2 is held, so the on-model run cannot start now. Registering the bars and building the consumer behind finite CPU controls can.

## How to start

1. Read [`docs/mechanisms/object-store.md`](../../mechanisms/object-store.md) (especially "What has never been tested" and "What would make it ACTIVE"), [`docs/mechanisms/exact-memory.md`](../../mechanisms/exact-memory.md), the [D19 record](../../../archive/labs/d19-object-store-2026-10-15/README.md), the [conformance record](../../labs/uorobj01-conformance-2026-10-18/README.md) and [`docs/research/notebook/m3-durable-memory.md`](../../research/notebook/m3-durable-memory.md).
2. Post a design note on the issue before writing code. It must pick the first consumer and answer the open decisions above (which consumer comes first, where exact identity meets geometry, who owns range reads); the dossier's remaining item, pre-registering the bar, is step 5. It must also state that exact token, occurrence and version identity stays separate from geometric addresses, so the store adds nothing to the discrete path. The note must also state that there is no softmax, no matmul and no float on this path.
3. Write the failing tests first, as finite CPU controls with no trained model. Put them next to the existing suites in `crates/uor-r4-object-store/tests/` (for example a new file beside `bindings.rs` and `controls.rs`). They must cover: write then read back byte-identical; restart then restore byte-identical; a retracted or unstated fact refused rather than guessed; explicit reset; and a hostile or corrupt record refused. Run `~/.cargo/bin/cargo test -p uor-r4-object-store` before and after. The baseline is 60 passing tests.
4. Implement the consumer on the model path. The saved-session format `UR4SESS1` lives in `crates/uor-r4-integer/src/session_codec.rs`. Use the `export_scope` and `import_scope` hooks (the only definition found is `crates/uor-r4-core/src/native_geometric/durable_memory.rs`, in the older engine outside the selected build, so decide whether to port it) and the occurrence bindings in `crates/uor-r4-object-store/src/bindings.rs`. Keep the change small and run `~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm` too (without that feature the serving modules do not compile).
5. Draft the pre-registration for the durable-memory bars: the panel, how it is frozen and hashed, the pass thresholds, the stop rules, and the model it runs on. The bar thresholds are not yet registered. Running the bars on a model is a GPU or serving run that needs a non-author FIT review and an owner release first.
6. Open a PR that says `References #<this issue>` for the consumer and tests, and `Closes` only after the bars have run with recorded results. Record the result as one row in [`docs/research/notebook/m3-durable-memory.md`](../../research/notebook/m3-durable-memory.md).

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:22`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:104`](../../../docs/mechanisms/README.md)
- [`docs/start/transformer-vs-geometry.md:34`](../../../docs/start/transformer-vs-geometry.md)
- [`docs/mechanisms/object-store.md`](../../mechanisms/object-store.md)
- [`docs/integration/project-track.md`](../../integration/project-track.md)
