# Phase 4: Path-3 blanket impls + cross-namespace domain fix

## What this phase ships

1. **Cross-namespace-domain bug fix in Null-stub emission.** The Phase 2/3
   emission was using the workspace-wide `all_props_by_domain` lookup to
   populate trait impl bodies. That lookup included properties declared
   *outside* a trait's defining namespace (cross-namespace domain
   properties). Per CLAUDE.md, cross-namespace properties are *not*
   generated as trait methods, so the Null impls were emitting method
   bodies for methods that don't exist on the trait.

   The fix: Null impls now walk the trait's **declaring module**'s
   `properties` list and filter by `domain == trait_iri`, mirroring the
   trait-generation logic. This eliminates ~1800 lines of spurious
   "method not in trait" emission across foundation/src/ without
   changing the Null-stub count (still 188 new + 14 pre-existing).

2. **Phase 4 scope decision: no new blanket impls.** The plan envisioned
   hand-written `impl {Observable}<H> for Validated<T, H>` delegating
   to `primitive_*` functions. Two preconditions are unmet:

   - `PATH3_ALLOW_LIST` in `classification.rs` is empty at Phase 0 close;
     no classes are classified `Path3PrimitiveBacked`.
   - Phase 1c (`HostTypes::Decimal` arithmetic bounds) was deferred
     pending Path-3 demand — which per point 1 doesn't materialize.

   Closing Path-3 orphans requires either populating the allow-list AND
   adding the arithmetic bounds AND writing the blanket impls, or
   routing Path-3 candidates through the Phase-2 Null-stub mechanism
   (which is how they effectively close today — many Path-3 candidate
   classes from strategy doc §Path 3 are classified Path-1 and already
   Null-stubbed).

3. **Enum-extension attempt reverted.** Phase 4 initially tried adding
   `Default` derivation to every generated enum with `#[default]` on the
   first variant, and emitting `{Enum}::default()` in Null impls for
   classes with enum accessors. Reverted because:

   - `WittLevel` is in `enum_class_names()` but is a `struct`, not an
     `enum` — it has no derived `Default`.
   - Cross-namespace enum imports in Null stubs require an additional
     collection pass in `generate_namespace_module`.
   - Inherited associated types (parent trait declares `type X`,
     child's impl re-declares) conflict when supertrait closure emits
     multiple impls per stub.

   Each is a small fix individually; together they put Phase 4's
   enum-path above the line-of-effort this phase has budget for. The
   7 Path-2 classes currently missing from the emitable set would
   become emittable after these fixes, but Phase 5/6 arrive before the
   ROI justifies the complication.

## Orphan count after Phase 4

Unchanged from Phase 3: **188 Null stubs** in bridge/kernel/user +
**14 hand-written NullPartition-family stubs** in enforcement.rs.

The 237 Path-1 classes that cascaded out of the emitable set, the
3 Path-2 classes that cascaded out, and the 20 Path-4 classes remain
orphan (design); Phase 5 writes theorem-family primitives, Phase 6
formalizes the Path-4 tracking register.

## Verification

- `cargo fmt --check` / `cargo clippy -D warnings` — clean.
- `cargo test` — all 96 test suites green.
- `cargo run --bin uor-crate` regenerates foundation with the
  cross-namespace fix; `git diff --exit-code` stays clean after the
  commit.
- `cargo run --bin uor-conformance` — 535 passed, 0 failed.
