# ORDER: m1-position-mechanism, kimi-33

Lab: kimi. Item: [m1-position-mechanism](../../../../roadmap/items/m1-position-mechanism.md). Base: `e8d6a624b`.

## Goal

The item's done-when reads:

> A position mechanism with finite controls that obeys CHARTER §3 (geometry only, no softmax or squashing, no matmul), plus a pre-registered test that it helps order. The bar for "helps order" is not yet registered; step 4 registers it.

## Cancel probe (run first)

`position_mechanism.rs` and its five finite controls are merged, and #3108 gave the sheet non-author FIT, but `PositionMode` has no consumer outside that module. The GPU read still uses endpoint BFS transport, so the registered arms cannot run. This is missing instrument wiring, not a mechanism negative. Continue with host-only integration; do not repeat the completed sheet repair or run the held experiment.

## Files in scope

- `crates/uor-r4-discrete/**`
- `docs/loop/tasks/m1-position-mechanism/kimi-33/**`

## Done when

An optional `PositionMode` and validated `PositionStepTable` reach the discrete GPU Flock/Relation read through driver config, device binding and snapshot round trips; absent position config preserves existing serialized defaults and endpoint BFS behavior. Prefix holonomy and the matched endpoint control use the same table and exact generated geometry, with identity behavior and invalid-mode/table refusal covered. Focused Rust/config/snapshot tests and host PTX admission pass, followed by full gates, REPORT and local PR checks and an other-vendor exact-head review. The PR says `References #2843`, leaves the item `ready`, and runs no GPU model work.

## Implementation plan

Add failing config/binding and kernel-source controls first. Then carry the optional mode/table through the existing config and snapshot pattern, append the mode to the GPU spec and the table to the device read ABI, and implement the exact prefix/endpoint action in `discrete.cu`. Keep the legacy path byte-for-byte when position config is absent. Run the focused suites before the full discrete and loop checks.

## Stop and ask if

A GPU run, training schedule or owner release becomes necessary; existing defaults cannot be preserved; a new dependency or non-geometric operator is needed; or any file outside the stated scope must change.
