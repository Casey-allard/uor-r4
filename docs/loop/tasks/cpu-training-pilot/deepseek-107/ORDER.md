# ORDER: cpu-training-pilot, deepseek-107 (remove the CPU build race)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `6d88bf103`.

## Goal

Remove the Cargo host/target collision that intermittently makes ordinary
`uor-r4-cpu-train` builds fail with missing `uor_r4_integer::stack` or
`uor_r4_core` imports. The collision is an instrument defect already recorded
by Claude:

- `uor-r4-discrete/build.rs` depends on `uor-r4-core`;
- default `uor-r4-core` depends on `uor-r4-integer`;
- `uor-r4-cpu-train` requests `uor-r4-integer` with `serving-wasm` and
  `fbs-synthetic`, so Cargo builds two feature-distinct integer crates;
- the integer crate is also a `cdylib`, and its unsuffixed outputs collide.

The build script needs only the exact SpiralCore/prime-route metadata, not the
integer runtime. Give `uor-r4-core` an explicit default-enabled runtime feature
and let the discrete build dependency disable it.

This is a build-graph repair. It runs no model comparison and changes no model
artifact or arithmetic.

## Files in scope

- `crates/uor-r4-core/Cargo.toml`
- `crates/uor-r4-core/src/native_support.rs`
- `crates/uor-r4-discrete/Cargo.toml`
- `docs/loop/tasks/cpu-training-pilot/deepseek-107/**`
- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/research/cpu-training-program.md`

The item scope is widened by this ORDER to include those two Cargo manifests
and `uor-r4-core/src/native_support.rs`; no Claude-owned float, FBS, integer or
Studio file is touched.

## Design

- Add a default-enabled `integer-runtime` feature to `uor-r4-core`, making
  `uor-r4-integer` optional and activated only by that feature.
- Keep the existing default core API unchanged.
- Gate only the core modules that need the integer runtime:
  `native_geometric`, `report_output` and `transformerless`.
- Keep the metadata-only `spiralcore_operator`, `prime_route_attention`,
  `zeta_zeros` and `answer_oracle` available without the feature.
- Change `uor-r4-discrete`'s build dependency to
  `uor-r4-core = { path = "../uor-r4-core", default-features = false }`.

## Done when

1. Default `uor-r4-core` keeps its existing exported surface and tests pass.
2. `uor-r4-core --no-default-features --lib` compiles and exposes the geometry
   metadata used by the discrete build script.
3. `cargo tree` shows the `uor-r4-discrete` build path no longer reaches
   `uor-r4-integer`.
4. Ordinary `uor-r4-cpu-train` checks/tests run in one shared target without
   requiring a fresh target directory or producing the collision.
5. Package tests, strict package Clippy where the head is already green,
   formatting, report validation, exact-head review, protected merge and
   fresh-main verification pass.

## Cancel probe

- `main` is clean at `6d88bf103`.
- Claude's #3629 changes only `train-in-space` and its docs, not these files.
- The discrete build script imports only `uor_r4_core::spiralcore_operator`.
- The four metadata modules compile without `uor-r4-integer`.

## Stop and report

Stop if the minimal core feature changes default callers or requires edits to
Claude-owned runtime files. Do not weaken or skip a failing crate test to make
the build race appear fixed, and do not claim a model or CPU-training result.
