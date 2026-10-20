# Phase 5: Path-2 primitive bodies (no-op at this closure)

## Status: no-op

Phase 5 was scoped against the plan's Phase-3 design: emit
`{Foo}Witness` + `{Foo}MintInputs<H>` + `impl VerifiedMint for
{Foo}Witness` with `Err("WITNESS_UNIMPLEMENTED_STUB:...")` bodies, then
replace those bodies per theorem family (PT_, ST_, CPT_, OB_P_, …).

The actual Phase 3 shipped a *Null-stub closure* instead: Path-2
classes now have `Null{Foo}<H>` structs with absent-sentinel impls,
identical in shape to Phase 2's Path-1 emission. No `VerifiedMint`
scaffolding was generated; no stub-body primitives exist to replace.

The Product/Coproduct Amendment's existing theorem-backed witnesses
(`PartitionProductWitness`, `PartitionCoproductWitness`,
`CartesianProductWitness`) remain the canonical theorem-verification
surface. Their mint primitives already verify PT_1 / PT_3 / PT_4 /
ST_1 / ST_2 / ST_6–ST_10 / CPT_1 / CPT_3–CPT_5 and the
`foundation:*LayoutWidth` invariants at mint time. No new primitives
are required at this closure.

## Future theorem-family work

When additional theorem families need runtime verification, the
entry point is:

1. Classify the target ontology class as `Path2TheoremWitness` with
   a real `theorem_identity` (blocked today by R6 — the
   `op:Identity`↔class back-reference isn't encoded in the ontology
   model in a form `classification::classify` can resolve cleanly).
2. Author a hand-written `{Foo}Witness` struct + `VerifiedMint` impl
   that routes through a per-family primitive in `foundation/src/
   primitives/{family}.rs`. The amendment's partition-algebra witnesses
   are the canonical template.
3. Remove the Phase 3 Null-stub for that class so the theorem-backed
   witness is the sole closure of the ontology trait.

## Verification

Phase 5 ships no source changes. The Phase 4 verification state
(96 test suites green, 535 conformance checks green, 188 Null stubs)
is unchanged.
