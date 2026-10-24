# ORDER: object-store-consumer-m3, deepseek-62 (freeze the durable-memory panel)

Lab: deepseek. Item: [object-store-consumer-m3](../../../../roadmap/items/object-store-consumer-m3.md) (#2873). Base: `30a2aaa4f`. Partial unit: `References #2873`; the item stays `ready` (the on-model bars still need an M1/M2 model, a non-author FIT and Casey's typed release).

## Goal

The draft pre-registration in `docs/labs/object-store-consumer-m3/README.md` names a 384-item `m3-durable-memory-panel-v1` panel but leaves it "to be frozen". This unit freezes it: a canonical JSONL artifact with one deterministic item per line (64 per sub-panel across provenance, correction, abstention, non_memory_prose, reset_isolation, restart_restore), its whole-object SHA-256, and the `Manifest::version()` produced by folding it into `uor_r4_object_store::Store`. A finite CPU test regenerates the panel, asserts the committed bytes equal a fresh generation, folds them, and pins both values, so the frozen artifact cannot drift silently. No model, GPU, panel scoring, FIT or owner release is claimed.

## Cancel probe (run first)

At base `30a2aaa4f`: `docs/labs/object-store-consumer-m3/README.md` says the panel is "to be frozen", no `m3-durable-memory-panel-v1.jsonl` or generator exists, and no test folds the panel bytes. Task stands.

## Files in scope

- `docs/labs/object-store-consumer-m3/**` (panel artifact + README status/pins)
- `crates/uor-r4-object-store/tests/m3_durable_memory_panel.rs` (new)
- this task folder

## Done when

- The frozen JSONL panel is committed with 384 canonical items (64 per sub-panel) and named in the pre-registration with its byte size, whole-object SHA-256 and `Manifest::version()`.
- `cargo test -p uor-r4-object-store` regenerates the panel, asserts byte-equality with the committed artifact, folds it into the store, and pins the SHA-256 and Manifest version; tampering/drift fails the target.
- The README keeps the sheet status DRAFT (FIT/owner/model still open) and records the frozen pins and the frozen panel test as the drift guard.
- `loop report --check`, `loop pr-check --local` and `loop gates --full` pass; a counted non-deepseek review records at the exact head.

## Stop and ask if

- A bar threshold, owner release, model run, or a file outside the listed scope is needed.