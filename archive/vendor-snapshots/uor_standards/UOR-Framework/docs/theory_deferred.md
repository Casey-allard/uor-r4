# Theory-deferred register

This register pairs every class classified `Path4TheoryDeferred` by
[phase-0 classification](./orphan-closure/phase-0-classification.md) with
an open research question. Downstream implementers should **not** try to
write a concrete impl for these traits until the linked theory advance
lands — the traits are orphan by design.

The [phase-6 doc](./orphan-closure/phase-6-theory-deferred.md) describes
the row format; the conformance check
`rust/theory_deferred_register` verifies bijection between this register
and the classification report.

| Class IRI | Namespace | Research question |
|---|---|---|
| `cohomology:CochainComplex` | `cohomology` | OB_P1/P2/P3 aren't yet grounded in a computable pipeline — the cohomology machinery is stated at the ontology level but has no no_std-fittable chain-complex arithmetic. Blocked on: grounding the `homology:ChainComplex` primitives first. |
| `cohomology:CohomologyGroup` | `cohomology` | Same blocker as `CochainComplex`. The cohomology group is computed from the cochain complex; no cochain machinery → no cohomology group. |
| `cohomology:GluingObstruction` | `cohomology` | The OB_P_ gluing-obstruction family assumes a sheaf whose restriction maps can be evaluated — currently no `RestrictionMap` primitive exists. |
| `cohomology:RestrictionMap` | `cohomology` | Sheaf-theoretic restriction is not grounded: the foundation has no `Sheaf`/`Stalk` machinery to restrict from. |
| `cohomology:Section` | `cohomology` | Sections of a sheaf; blocked on `Sheaf` grounding. |
| `cohomology:Sheaf` | `cohomology` | No computable-sheaf formalism in the foundation yet. Theory work needed: define a no_std-fittable sheaf type with explicit stalks and restriction maps over the constraint nerve. |
| `cohomology:Stalk` | `cohomology` | Blocked on `Sheaf`. |
| `monoidal:MonoidalProduct` | `monoidal` | The operad-level monoidal construction is declared in `kernel/monoidal.rs` but OP_1–OP_5 (especially OP_3's Leibniz-rule grounding) don't yet specify a computation path. Theory work: formalize the Leibniz rule for monoidal products over `ConstrainedTypeShape` shapes. |
| `operad:OperadComposition` | `operad` | Same OP_3 dependency as `MonoidalProduct`. |
| `parallel:DisjointnessCertificate` | `parallel` | Parallel composition requires a runtime-integration story (concurrent site access semantics) that the no_std foundation deliberately lacks. |
| `parallel:ParallelProduct` | `parallel` | Same runtime-integration blocker. |
| `parallel:ParallelTrace` | `parallel` | Same. |
| `parallel:SitePartitioning` | `parallel` | Same — the partition of sites across parallel branches presupposes a concurrency model. |
| `parallel:SynchronizationPoint` | `parallel` | Same. |
| `stream:Epoch` | `stream` | Reactive/stream semantics depend on a runtime-integration amendment that hasn't been drafted. no_std semantics for streams (bounded, eager, or lazy) need to be selected and formalized first. |
| `stream:EpochBoundary` | `stream` | Same blocker as `Epoch`. |
| `stream:ProductiveStream` | `stream` | Same. |
| `stream:StreamMorphism` | `stream` | Same. |
| `stream:StreamPrefix` | `stream` | Same. |
| `stream:Unfold` | `stream` | Same. |

## How to contribute to closing an entry

1. Identify the blocker — all 20 classes fall into three clusters:
   cohomology grounding, monoidal/operad grounding, and
   runtime-integration (parallel + stream).
2. Open a research note under `external/` describing the theory path.
3. Once theory lands, re-classify the target class out of Path-4 in
   [codegen/src/classification.rs](../codegen/src/classification.rs)
   and author the corresponding concrete type. The Null-stub
   mechanism from Phase 2 is NOT the right closure for these classes
   — they need real theorem-backed impls.

## Current closure

20/20 classes documented. Conformance-checked: yes
(`rust/theory_deferred_register`).
