# ORDER: repair the workspace dependency lock for the Wick diagnostics

Lab: codex
Item: [written-state-count-model](../../../../roadmap/items/written-state-count-model.md) (#4063)

[Original ORDER](../codex-1/ORDER.md). The role trace ran and both seeds replayed at 39a95e7b1. The count collection and replay were recorded in 07f23e658. The notebook merge preserved both labs’ rows at a38aa28d6. CI on that head failed before running checks because the root Cargo.lock missed the standalone role scorer’s direct serde dependency. The root workspace includes the scorer transitively.

Scope: synchronize just that dependency in the root Cargo.lock, rerun locked workspace metadata and PR CI, validate the existing evidence, and publish the exact-head report. Cancel if the dependency is already present. Stop if any unrelated dependency update would be required. The released inputs, scoring code and result files remain bound to the first ORDER.
