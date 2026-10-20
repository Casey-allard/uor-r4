# Orphan-trait closure: overview

## Problem

The `uor-foundation` crate exports 452 public traits derived from the UOR
Foundation ontology. Most have zero implementations in the workspace —
they are trait surfaces waiting for concrete carriers. Downstream
implementers have to write every impl themselves from scratch, with no
derivation path from the ontology to a usable type.

The goal is to close every *non-theory-deferred* trait orphan via a
mechanical codegen path, leaving only the traits whose implementation
awaits genuine theory advances. Downstream implementers should have a
clear signal: either "write an impl" (backed by a codegen-provided
scaffold) or "await theory advance" (tracked with an open research
question).

## Four paths

Every public trait in `foundation/src/` falls into exactly one category:

| PathKind              | Shape                                                              | Count (target) |
|-----------------------|--------------------------------------------------------------------|----------------|
| `AlreadyImplemented`  | Partition-algebra witnesses, Certificate subclasses, etc.          | ~20            |
| `Path1HandleResolver` | Pure-accessor bundle → codegen emits `{Foo}Handle` + `{Foo}Resolver` + `{Foo}Record` + `Resolved{Foo}` | ~150           |
| `Path2TheoremWitness` | Theorem-backed claim → codegen emits `{Foo}Witness` + `{Foo}MintInputs` + `impl VerifiedMint`     | ~80            |
| `Path3PrimitiveBacked`| Foundation already computes the quantity → hand-written blanket `impl` over `Validated<T, H>`     | ~100           |
| `Path4TheoryDeferred` | Awaiting grounding of cohomology / operad / parallel / stream theory | ~50            |
| `Skip`                | Enum classes, `Primitives` — not a trait                           | 20             |

## Phase sequence

Phase boundaries are commit boundaries. Each phase is independently
mergeable; orphan counts drop in documented tranches.

| Phase | Scope                                                     | Orphan delta | This doc                                  |
|-------|-----------------------------------------------------------|--------------|-------------------------------------------|
| 0     | Classification table + reports + tests                    | 0            | [phase-0-classification.md](./phase-0-classification.md) |
| 1     | Prerequisite corrections (capacity guard, entropy, decimal) | 0          | [phase-1-prerequisites.md](./phase-1-prerequisites.md) |
| 2     | Path-1 Handle+Resolver codegen rule                       | ~-150        | [phase-2-handle-resolver.md](./phase-2-handle-resolver.md) |
| 3     | Path-2 Witness+Mint codegen rule (stub bodies)            | ~-80         | [phase-3-witness-mint.md](./phase-3-witness-mint.md) |
| 4     | Path-3 blanket impls (hand-written)                       | ~-100        | [phase-4-blanket-impls.md](./phase-4-blanket-impls.md) |
| 5     | Path-2 primitive bodies (per theorem family, sequential)  | 0            | [phase-5-primitives.md](./phase-5-primitives.md) |
| 6     | Path-4 theory-deferred banner + register                  | 0            | [phase-6-theory-deferred.md](./phase-6-theory-deferred.md) |

Final state: orphan count drops from ~430 to ~50, all of which are
correctly labelled `Path4TheoryDeferred` with tracking issues.

## Guiding principle

**Correctness over backwards compatibility.** This work introduces
breaking changes where necessary (arithmetic bounds on
`HostTypes::Decimal`, by-value partition-factor accessors, etc.) and
rejects fallback patterns — no `Default::default()` catch-alls, no
unit-type default resolvers, no allow-lists of intentional exceptions.
Every "absent" sentinel is explicit (`EMPTY_DECIMAL`,
`ContentFingerprint::zero()`) and documented as absent, not empty/zero.

## Machine-readable classification

Phase 0 emits `classification_report.md` — a table of every class with
its PathKind, rationale, and (for Path-2) theorem IRI + entropy flag.
Regenerated on every `cargo run --bin uor-crate`; committed to the repo;
`git diff --exit-code docs/orphan-closure/classification_report.md`
gates drift.
