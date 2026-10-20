# Phase 3: Path-2 orphan closure (witness-trait stubs)

## Contract

Phase 3 extends the Phase 2 Null-stub emission to cover classes
classified `Path2TheoremWitness`. The emission pattern is identical —
`Null{Class}<H>` struct + `impl {Class}<H> for Null{Class}<H>` with
absent-sentinel defaults — and uses the same fixed-point
`emitable_null_set` closure.

## Why the same pattern

Path-2 classes are ontology-derived witness traits (`BornRuleVerification`,
`GroundingWitness`, `ImpossibilityWitness`, etc.). Their traits have
accessor methods just like Path-1 traits; the Path-2 label denotes that
downstream typically provides a theorem-backed concrete implementation,
but the **orphan contract** is identical — an ontology-derived trait
with zero concrete impls.

The Null stub closes the orphan trivially (resolver-absent defaults,
`&H::EMPTY_*` sentinels). A Phase 5 primitive body can later fill in
real theorem verification on top of a separate `{Class}Witness` mint
type (see "deferred mint scaffolding" below).

## What Phase 3 adds

Codegen change: `traits::should_emit_null_stub` now accepts both
`PathKind::Path1HandleResolver` and `PathKind::Path2TheoremWitness`.
All other emission logic (R3 field mapping, R4 absent sentinels,
transitive supertrait impls, fixed-point reference closure) is shared
with Phase 2.

Closure: 7 of 10 Path-2 classes enter the emitable set (the remaining 3
cascade out because their reference graph touches enum-class accessors
or Path-3/Path-4 classes without stubs). Total Null-stub count rises
from 181 to 188 (plus the 14 hand-written NullPartition-family stubs in
enforcement.rs).

## Tests

- **`path1_null_emission::phase2_emission_produces_at_least_min_stubs`**
  — ratchet updated: minimum 188 Null stubs. Drop below = regression.
- **`phase3_path2_coverage`** (this phase) — at least some Path-2
  classes are emitted; the overall count is reported.

## Deferred: VerifiedMint scaffolds per Path-2 class

The plan's original Phase 3 contract was more ambitious: emit
`{Class}Witness` + `{Class}MintInputs<H>` + `impl VerifiedMint for
{Class}Witness` with stub bodies returning
`Err("WITNESS_UNIMPLEMENTED_STUB:{theorem_iri}")`. This full scaffold
(R5, R6, R7) is deferred — the Null-stub alternative closes the orphan
mechanically without the complications of:

- Theorem-identity resolution (R6) per class — the ontology doesn't yet
  declare the `op:Identity`↔class linkage cleanly for all 10 Path-2 classes.
- Entropy-bearing detection (R7) — requires precise `#[derive(Hash)]`
  management per witness.
- MintInputs field shape (R5) — different per theorem family; cleaner
  to author hand-written per theorem rather than emit stubs.

Phase 5 introduces per-theorem-family primitive verification files
under `foundation/src/primitives/{family}.rs`; those primitives will
produce concrete `{Class}Witness`-style structs rather than routing
through a generic VerifiedMint scaffold. The Null stub remains the
orphan-closure mechanism.

## Conformance

- `cargo run --bin uor-conformance` passes all 535 checks.
- Orphan count drops by at least 7 (Path-2 closures) on top of Phase 2.
