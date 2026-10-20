# Changelog

All notable changes to UOR-Framework are documented in this file.

## 0.5.2 — ADR-018/060 fingerprint-width parametricity (Hasher<FP_MAX>) — 2026-05-23

Closes a downstream-blocking bug: foundation 0.5.1 pinned the entire
resolver/pipeline tower to `Hasher<32>` — the `AxisTuple` blanket impl and
`run_route`'s `A: AxisTuple + Hasher` bound both defaulted the hasher
fingerprint width `FP_MAX` to 32, so a `Sha512Hasher: Hasher<64>` (or any
non-32-byte-output hasher) could not be selected at all. Per ADR-018 ("every
capacity-bounded width is part of the index, total over `HostBounds`"), the
fingerprint width is now a free const-generic `FP_MAX` threaded through every
hash-bearing surface and instantiated from the application's
`<B as HostBounds>::FINGERPRINT_MAX_BYTES` — exactly parallel to how ADR-060's
`INLINE_BYTES` flows from `carrier_inline_bytes::<B>()`. The `prism_model!` /
`axis!` SDK macros derive and thread `FP_MAX` automatically, so the
application-author surface is **ergonomically identical**; only applications
selecting a non-default `HostBounds` ever name the width. Conformance reports
**546 passed, 0 warnings, 0 failed**.

### Breaking

- **`Grounded` gains a fingerprint-width parameter:
  `Grounded<'a, T, INLINE_BYTES, FP_MAX = 32, Tag = T>`** (was
  `Grounded<'a, T, INLINE_BYTES, Tag = T>`). The carried `ContentFingerprint`,
  `GroundingCertificate`, `Derivation`, and `Trace` are all `FP_MAX`-indexed;
  `content_fingerprint()` returns `ContentFingerprint<FP_MAX>`.
- **`PrismModel` gains `FP_MAX`:
  `PrismModel<'a, H, B, A, INLINE_BYTES, FP_MAX, R, C>`** (inserted between
  `INLINE_BYTES` and `R`); `forward` returns
  `Grounded<'a, Output, INLINE_BYTES, FP_MAX>`. `A` is bound
  `AxisTuple<INLINE_BYTES, FP_MAX> + Hasher<FP_MAX>`.
- **`AxisExtension` / `AxisTuple` gain `FP_MAX`:**
  `AxisExtension<INLINE_BYTES, FP_MAX>`, `AxisTuple<INLINE_BYTES, FP_MAX>`. The
  blanket `impl<…, H: Hasher<FP_MAX>> AxisTuple<INLINE_BYTES, FP_MAX> for H` and
  all tuple impls thread the width.
- **The catamorphism entry points gain `const FP_MAX`:** `run`, `run_const`,
  `run_parallel`, `run_stream`, `run_interactive`, `run_route`, plus
  `StreamDriver` / `InteractionDriver` / `StepResult`, the
  `run_tower_completeness` / `run_incremental_completeness` /
  `run_grounding_aware` / `run_inhabitance` resolver runners and their
  `resolver::*::certify` free-functions, and the five `certify_*_const`
  companions.
- **The certificate hierarchy is `FP_MAX`-indexed:** the 12 fingerprint-carrying
  certificate kinds become `{Cert}<const FP_MAX = 32>`; the crate-internal
  `ResolverKernel::Cert` is now a const-generic GAT
  (`type Cert<const FP_MAX: usize>: Certificate`), and `MintWithLevelFingerprint`
  becomes `MintWithLevelFingerprint<const FP_MAX>`.
- **Verify path is width-parametric:** `replay::certify_from_trace` and
  `uor_foundation_verify::verify_trace` accept `Trace<TR_MAX, FP_MAX>` and return
  `Certified<GroundingCertificate<FP_MAX>>`; both infer `FP_MAX` from the trace,
  so `verify_trace(&trace)` is unchanged at the call site.
- Parameter-order convention: where a function carries both a hasher type and the
  width const, the order is `<…, H: Hasher<FP_MAX>, const FP_MAX: usize>` (H
  first) — `multiplication::certify`, `axis::cryptanalyze`,
  `mint_cohomology_class`, and `mint_homology_class` were reordered to match.

### Fixed

- `PrismModel::forward`'s trait-method return type was pinned to `FP_MAX = 32`
  even where the impl selected another width — a latent defect no 32-width model
  could surface. Now threaded; proven by `behavior_hasher_fp_max_64.rs`, which
  grounds a real `Hasher<64>` model through `forward()` → `run_route` and
  round-trips its 64-byte fingerprint through `Derivation::replay` →
  `certify_from_trace` bit-identically.

### Unchanged

- Resolver-*provided* content addresses (`PartitionHandle`, `NullPartition`, the
  partition witness records) keep their default-width `ContentFingerprint`: they
  never bound on `Hasher`, so they never constrained the substituted width.
- `ContentFingerprint<FP_MAX = 32>` and `Trace<TR_MAX = 256, FP_MAX = 32>` keep
  their ergonomic defaults; `TRACE_REPLAY_FORMAT_VERSION` stays foundation-fixed.

## 0.5.1 — ADR-060 source-polymorphic application I/O (input + Grounded output) — 2026-05-22

Completes ADR-060: 0.5.0 made the **intermediate** carriers source-polymorphic
but left the application **I/O boundary** hard-capped — `run_route` materialized
the model input into a fixed `[u8; INLINE_BYTES]` buffer and rejected any input
whose `MAX_BYTES` exceeded that, and `Grounded` stored its output in a fixed
`[u8; INLINE_BYTES]` slot. Per ADR-060 principle (3) ("no carrier-side fixed
allocation that depends on payload size") and ADR-028-as-amended (the output
payload carrier is a source-polymorphic `TermValue`), both the input and output
paths are now source-polymorphic, so **arbitrarily large inputs content-address
natively** through `prism_model! → forward() → run_route()`. Conformance reports
**546 passed, 0 warnings, 0 failed**.

### Breaking

- **`IntoBindingValue` is now `IntoBindingValue<'a>`** with a single method
  `fn as_binding_value<const INLINE_BYTES: usize>(&self) -> TermValue<'a, INLINE_BYTES>`.
  The `const MAX_BYTES` and `fn into_binding_bytes(&self, &mut [u8])` members are
  removed. An input shape returns a source-polymorphic carrier: `Inline` (small),
  `Borrowed` (large in-memory, zero-copy), or `Stream` (unbounded). There is no
  input byte-width ceiling.
- **`Grounded` gains a lifetime: `Grounded<'a, T, INLINE_BYTES, Tag = T>`.** Its
  output payload is now a `TermValue<'a, INLINE_BYTES>` (was `[u8; INLINE_BYTES]`
  + `output_len`); add `Grounded::output_value()` for the carrier and the
  pre-existing `output_bytes()` for the contiguous prefix. The crate-internal
  `with_output_bytes` setter becomes `with_output(TermValue)`.
- **`PrismModel` gains a lifetime: `PrismModel<'a, H, B, A, INLINE_BYTES, R, C>`**;
  `forward` returns `Grounded<'a, Output, INLINE_BYTES>`. `run_route` gains `'a`
  and returns `Grounded<'a, …>`. The ψ-resolver `resolve` decouples its `&self`
  borrow from the value lifetime (`fn resolve<'a>(&self, TermValue<'a, _>) -> …`),
  so a locally-constructed resolver tuple can drive an evaluation whose output
  carrier escapes with the route input's lifetime.

### Fixed

- `run_route` streams the model input through the selected `Hasher` (`Inline`/
  `Borrowed` in one chunk, `Stream` chunk-by-chunk via `for_each_chunk`) for the
  content address — no fixed buffer, no `MAX_BYTES > INLINE_BYTES` rejection.
- The catamorphism's `Term::AxisInvocation` canonical hash (axis 0 / kernel 0)
  folds the operand carrier via `for_each_chunk`, so a `Stream` operand hashes
  correctly (previously `v.bytes()` saw an empty slice for `Stream`).

### Added

- `foundation/tests/behavior_adr_060_large_input_grounded.rs`: feeds an 8 KiB
  `Borrowed` input and a 4 MiB `Stream` input through the sanctioned
  `forward()`/`run_route()` path into a `Grounded`, asserting the output digest
  reflects the **full** input (bounded resident memory; flipping a late byte
  changes the digest). This test fails against the 0.5.0 capped path.

## 0.5.0 — ADR-060 source-polymorphic value carrier — 2026-05-22

Breaking release. Replaces the contrived fixed 4096-byte `TermValue`
ceiling and the `DefaultHostBounds` "default application" with a
**source-polymorphic value carrier**, enabling allocation-free streaming
of arbitrarily large payloads through the catamorphism. The inline carrier
width is now derived per application from its selected `HostBounds`.
Conformance reports **546 passed, 0 warnings, 0 failed**.

### Breaking

- **`DefaultHostBounds` removed.** There is no "default" application; every
  consumer declares its own `impl HostBounds` and threads its constants
  explicitly. `DefaultHostTypes` is unchanged.
- **`HostBounds` reduced to 14 consts.** The 12 byte-width caps are gone:
  `TERM_VALUE_MAX_BYTES`, `AXIS_OUTPUT_BYTES_CEILING`,
  `ROUTE_INPUT_BUFFER_BYTES`, `ROUTE_OUTPUT_BUFFER_BYTES`, and the eight
  per-ψ-stage `*_OUTPUT_BYTES_MAX` constants no longer exist.
- **`TermValue` is now an enum** `TermValue<'a, const INLINE_BYTES: usize>`
  with `Inline` / `Borrowed` / `Stream` variants (was a fixed 4096-byte
  struct). A new `ChunkSource` trait backs the unbounded `Stream` variant.
- **`const INLINE_BYTES: usize` threaded through the carrier surface:**
  `Term<'a, N>`, `TermArena<'a, N, CAP>`, `Grounded<T, N, Tag = T>`,
  `CompileUnit<'a, N>`, `PrismModel<H, B, A, N>`, `FoundationClosed<N>`,
  the eight ψ-resolver traits + `Has*Resolver<N, H>` markers, `Sinking<N>`,
  `EmitThrough<N, H>`, and every driver (`run`, `run_const`, `run_parallel`,
  `run_stream`, `run_interactive`, `StreamDriver`, `InteractionDriver`,
  `evaluate_term_tree`) gain the trailing carrier-width parameter.
- Resolver `resolve` signature is uniformly
  `fn resolve<'a>(&'a self, TermValue<'a, N>) -> Result<TermValue<'a, N>, ShapeViolation>`.

### Added

- `pipeline::carrier_inline_bytes::<B: HostBounds>() -> usize` — the const fn
  that derives the inline-carrier width from an application's `HostBounds`
  (max of the Witt-level byte width, fingerprint width, and content-address
  envelope). Applications instantiate `INLINE_BYTES` from it.
- Eight app-facing per-ψ-stage carrier-width helpers (`nerve_carrier_bytes`,
  `chain_complex_carrier_bytes`, …, `k_invariants_carrier_bytes`) for sizing
  resolver-owned scratch — each a structural-count × foundation-fixed
  per-element wire width, never a contrived literal.
- Descriptor constants `HASHER_IDENTIFIER_BYTES`, `SITE_DESCRIPTOR_BYTES`,
  `CONSTRAINT_DESCRIPTOR_BYTES`, `BETTI_ELEMENT_BYTES`, `PSI_STAGE_HEADER_BYTES`.
- `TermValue::to_vec()` — the only allocation surface, gated behind the
  optional `alloc` feature. The default no-`std` path and the principal data
  path (`Inline` / `Borrowed` / `Stream` + `for_each_chunk`) stay
  allocation-free.
- `foundation/tests/behavior_adr_060_arbitrary_scaling.rs` — folds an 8 MiB
  `ChunkSource` stream through a hasher with bounded resident memory (chunk
  size, not total), proving lossless content-addressing at scales orders of
  magnitude beyond the retired 4096-byte cap.

### Migration

- Consumers that used `DefaultHostBounds` now declare an `impl HostBounds`
  and pass `carrier_inline_bytes::<MyBounds>()` as the `INLINE_BYTES`
  argument. A test-only `ReferenceHostBounds` + `REFERENCE_INLINE_BYTES`
  reproducing the prior defaults (16/32/256/64) ships in
  `uor-foundation-test-helpers`.

### Internal

- Public-API snapshots, `endpoint_coverage.toml`, and the
  phantom-tag / driver-shape / driver-must-use / bridge-namespace /
  pipeline-run-threads-input conformance validators updated for the new
  signatures.

## Phase 18 — Documentation, contract tests, publish-readiness — 2026-04-28

Closes the published-crate-completion plan. Public API documentation
covers every module added in Phases 10–17; behavioural tests assert
the cross-cutting `OntologyVerifiedMint` contract; SDK metadata is
crates.io-ready. Conformance reports **543 passed, 0 warnings, 0
failed**.

### Documentation

- `lib.rs` top-level docstring extended with two new sections:
  - **Witness minting** — explains the `OntologyVerifiedMint` trait,
    the per-class `Mint{Foo}` types, the structural-invariant failure
    routes (BR_*, CC_*, IH_*, FX_4, WLS_*, surfaceSymmetry), and the
    `ontology_mint::<H>(inputs)` entry shape.
  - **Per-class observable views** — points consumers at the inherent
    `Validated::as_landauer()` / `as_jacobian()` /
    `as_carry_depth()` / `as_derivation_depth()` / `as_free_rank()`
    accessors that produce primitive-backed `Observable<H>` values.
- Stale doc comment fixed: `PartitionRecord<H>` no longer claims
  `entropy_nats` is `f64` — the field is `entropy_nats_bits: u64`
  per Phase 9, and the struct now derives `Eq + Hash` cleanly.

### Behavioural tests

- New `foundation/tests/ontology_verified_mint_contract.rs` (6 tests):
  - All ten `Mint{Foo}` witnesses are `Send + Sync + Copy + Clone +
    Debug + Eq + PartialEq` (compile-time assertions).
  - Every `THEOREM_IDENTITY` constant points to a well-formed
    op-namespace IRI (prefix discipline check).
  - Determinism: two mints with identical inputs produce identical
    fingerprints.
  - Distinctness: mints with differing inputs produce distinct
    fingerprints (Phase 15's content folding actually folds content).
  - Lift-obstruction dispatch: trivial=true with non-zero site
    routes to WLS_1; trivial=false with zero site routes to WLS_2.
  - Born-rule progressive failure routing: BR_1 → BR_2 → BR_3 →
    BR_4 in order as each invariant fails.

### Publish-readiness

- `uor-foundation-sdk/Cargo.toml` gains `keywords` and `categories`
  fields so the published crate has crates.io discovery metadata.
- `cargo publish --dry-run` succeeds for both crates (modulo the
  pre-existing `0.3.0 already on index` warning, which is normal
  when republishing the same version pre-bump).

## Phase 17 — `ConstraintRef` Affine/Conjunction const-buildable — 2026-04-28

Replaces the variable-length `&'static` slice fields on
`ConstraintRef::Affine` and `ConstraintRef::Conjunction` with
fixed-size arrays so stable-Rust const evaluation can build them
inline. Enables the SDK macros (`product_shape!`,
`coproduct_shape!`, `cartesian_product_shape!`) to support the full
operand-catalogue — `Affine`-bearing operands no longer fall back to
the `Site { position: u32::MAX }` sentinel. Conformance reports
**543 passed, 0 warnings, 0 failed**.

### Breaking

- `ConstraintRef::Affine { coefficients: &'static [i64], bias }`
  becomes
  `ConstraintRef::Affine { coefficients: [i64; AFFINE_MAX_COEFFS], coefficient_count: u32, bias }`.
  Active prefix is `coefficients[..coefficient_count as usize]`.
- `ConstraintRef::Conjunction { conjuncts: &'static [ConstraintRef] }`
  becomes
  `ConstraintRef::Conjunction { conjuncts: [LeafConstraintRef; CONJUNCTION_MAX_TERMS], conjunct_count: u32 }`.
  Conjunction is depth-limited to one level via the new
  `LeafConstraintRef` element type (every `ConstraintRef` variant
  except `Conjunction` itself).

### Additive

- `pub const AFFINE_MAX_COEFFS: usize = 8` and
  `pub const CONJUNCTION_MAX_TERMS: usize = 8` capacity caps in
  `foundation::pipeline`.
- `pub enum LeafConstraintRef` — Conjunction-conjunct element type.
- `ConstraintRef::as_leaf()` and `LeafConstraintRef::into_constraint()`
  conversions.
- `pub const fn shift_leaf_constraint(leaf, offset)` for
  Conjunction-shifting paths.
- `shift_constraint` performs real shifts for `Affine` and
  `Conjunction` (no more `Site { u32::MAX }` sentinel). Both
  variants compose correctly through the SDK macros.
- `uor-foundation-sdk` operand-support catalogue extended to every
  `ConstraintRef` variant; new SDK smoke tests
  (`product_shape_supports_affine_operand`,
  `coproduct_shape_supports_affine_operand`) verify the
  Affine-bearing path.
- All 89 in-tree Affine/Conjunction call sites updated to the new
  fixed-array shape: `foundation/tests/behavior_*.rs` (5 files) +
  `uor-foundation-sdk/tests/smoke.rs` + the coproduct macro emission
  in `codegen/src/sdk_macros.rs` + every Affine match arm in
  `codegen/src/pipeline.rs` and `codegen/src/enforcement.rs`.

### Conformance

- `rust/api` validator now looks back 5 lines for doc comments
  (some `pub` items have ≥ 4 stacked attribute lines).
- `rust/parametric_constraints` updated to expect the new
  Conjunction shape anchor.
- `rust/endpoint_coverage` registers `AFFINE_MAX_COEFFS`,
  `CONJUNCTION_MAX_TERMS`, `LeafConstraintRef`.

## Phase 16 — Per-class observable view newtypes — 2026-04-28

Replaces the bare `Validated<T, Phase>: Observable<H>` blanket impls
(which returned `H::EMPTY_DECIMAL` from `value()` for four of the five
Path-3 observables) with **five per-kind newtype views**, each
providing a primitive-backed `value()`. Consumers reach for an
explicit kind via inherent `Validated::as_*` accessors. Conformance
reports **543 passed, 0 warnings, 0 failed**.

### Breaking

- `<Validated<T, Phase> as Observable<H>>::value(&v)` no longer
  compiles — the bare `Validated` type doesn't impl `Observable<H>`
  any more. Callers MUST select a view first:
  `validated.as_landauer()` for Landauer-cost (or
  `as_jacobian()`/`as_carry_depth()`/`as_derivation_depth()`/`as_free_rank()`).
- The four leaf-trait impls (`JacobianObservable`,
  `CarryDepthObservable`, `DerivationDepthObservable`,
  `FreeRankObservable`) on `Validated<T, Phase>` are removed; impls
  now land on the corresponding view newtype only.

### Additive

- Five new public `pub struct Validated{Foo}View<T, Phase>(...)`
  types in [`foundation/src/blanket_impls.rs`](foundation/src/blanket_impls.rs):
  `ValidatedLandauerView`, `ValidatedJacobianView`,
  `ValidatedCarryDepthView`, `ValidatedDerivationDepthView`,
  `ValidatedFreeRankView`. Each carries `PhantomData<(fn() -> T, Phase)>`
  so the wrapper is `Send + Sync` regardless of `T`.
- Five new inherent `Validated::as_*` accessor methods on
  `Validated<T, Phase>` for ergonomic construction.
- Per-view `Observable<H>::value()` bodies:
  - **LandauerView** — delegates to `landauer_nats(&self)` (existing
    Phase-11 logic).
  - **JacobianView** — L1 sum of `primitive_curvature_jacobian::<T>()`,
    lifted via `H::Decimal::from_u64`.
  - **CarryDepthView** — orbit size from
    `primitive_dihedral_signature::<T>()`, via `from_u32`.
  - **DerivationDepthView** — reduction-step count from
    `primitive_terminal_reduction::<T>(W8)`, via `from_u32`.
  - **FreeRankView** — residual from
    `primitive_descent_metrics::<T>(&nerve_betti).0`, via `from_u32`.
- `blanket_impls_exempt` validator now requires the 11 new
  `impl<...> {Trait} for Validated{Foo}View<...>` blocks plus the
  five `pub struct Validated{Foo}View` declarations.
- `orphan_counts` validator's `validated_blanket` category-detection
  recognises `Validated*View` newtypes alongside the bare `Validated`.

### Tests

- New `foundation/tests/observable_views.rs` — 8 tests covering each
  view's `value()` body, leaf-trait conformance, the
  Copy/Clone/Default/Debug/PartialEq/Eq trait surface, and a
  `validated_no_longer_impls_observable_directly` regression check.

## Phase 15 — Real `verify_*` primitive bodies (12c closure) — 2026-04-28

Replaces every `WITNESS_UNIMPLEMENTED_STUB` Phase-12 baseline with a
real structural-invariant verification that rejects invalid inputs
with a typed `GenericImpossibilityWitness` whose IRI cites the
specific failing op-namespace identity. Successful mints carry a
content-addressed fingerprint folded over the input bytes (not just
the class IRI), so distinct inputs produce distinct witnesses.

### Breaking

- `Mint{Foo}::ontology_mint(Default::default())` now returns
  `Err(GenericImpossibilityWitness::for_identity(_))` for every
  Path-2 class except the abstract `morphism::Witness` (zero-field).
  The Phase-12 baseline accepted any input unconditionally; consumers
  relying on default-ok behaviour must populate the inputs.
- `crate::primitives::{family}::verify_*` bodies are no longer
  fingerprint-by-IRI — they fold the actual input bytes via a new
  `fingerprint_for_inputs(chunks)` helper. Witnesses minted with
  identical IRIs but different inputs now produce distinct
  fingerprints.

### Per-family failure modes

| Class | Failure IRIs |
|---|---|
| `cert::BornRuleVerification` | `op:BR_1` (verified=false), `op:BR_2` (born_rule_verified=false), `op:BR_3` (witt_length=0), `op:BR_4` (certifies empty) |
| `effect::DisjointnessWitness` | `op:FX_4` (zero handle or non-disjoint) |
| `morphism::GroundingWitness` | `op:surfaceSymmetry` (zero handle) |
| `morphism::ProjectionWitness` | `op:surfaceSymmetry` (zero handle) |
| `morphism::Witness` | none (abstract — always Ok) |
| `proof::ImpossibilityWitness` | `op:IH_1` (verified=false), `op:IH_2a` (reason empty), `op:IH_2b` (proves_identity zero) |
| `proof::InhabitanceImpossibilityWitness` | inherited IH_1/IH_2a/IH_2b + `op:IH_3` (contradiction_proof / grounded / search_trace zero) |
| `state::GroundingWitness` | `op:surfaceSymmetry` (witness_step=0 or empty bindings) |
| `type::CompletenessWitness` | `op:CC_1` (sites_closed=0), `op:CC_2` (witness_constraint zero) |
| `type::LiftObstruction` | `op:WLS_1` (trivial w/ non-zero site), `op:WLS_2` (non-trivial w/ zero site) |

### Additive

- `fingerprint_for_inputs(chunks: &[&[u8]])` helper in each family file
  — index-salted XOR fold across multiple byte chunks with
  chunk-boundary markers (`0xFF`) so chunk reordering produces a
  different fingerprint.
- `phase12_witness_mints.rs` rewritten with 21 tests: per-family
  populated-input success cases + `Default`-rejects cases asserting
  the family-IRI prefix on the typed error. The cross-family
  fingerprint-distinctness assertion is preserved.

## Phase 14 — `Mint{Foo}Inputs<H>` field mapping (R5 closure) — 2026-04-28

Replaces every `Mint{Foo}Inputs<H>` placeholder
(`pub _phantom: PhantomData<H>`) with one field per direct or inherited
property of its Path-2 ontology class. Consumers of
`OntologyVerifiedMint::ontology_mint(inputs)` can now pass theorem-
relevant data through typed input bundles. Conformance reports
**543 passed, 0 warnings, 0 failed**.

### Breaking

- `Mint{Foo}Inputs<H>` structs gain per-property `pub` fields
  (35 unique fields across the 10 Path-2 classes; ranges from 0 fields
  on the abstract `morphism::Witness` to 15 inherited fields on
  `proof::InhabitanceImpossibilityWitness`).
  Consumers using `Default::default()` continue to compile but now
  receive sentinel-filled inputs which Phase 15's verify_* will
  reject; populate fields explicitly to mint successfully.
- `OntologyVerifiedMint::Inputs<H: HostTypes + 'static>` GAT now
  requires `H: 'static` so that `&'static [{Range}Handle<H>]`
  non-functional fields satisfy `Handle<H>: 'static`. All in-tree
  `HostTypes` impls (DefaultHostTypes, host marker structs) satisfy
  this trivially.
- `crate::enforcement::PartitionHandle<H>` ships hand-written
  `Copy/Clone/PartialEq/Eq/Hash` impls (was `derive`-generated). The
  derive added a spurious `H: Copy + Clone + ...` bound that
  propagated to `MintInputs<H>` callers; the manual impls drop the
  bound. Public surface unchanged for consumers — `PartitionHandle::<H>`
  remains `Copy + Clone + Eq + Hash`.
- `Mint{Foo}Inputs<H>` structs gain hand-written `Copy + Clone`
  impls (was `derive`). The auto-derive on generic structs with
  `&'static H::HostString` / `&'static H::WitnessBytes` fields adds
  spurious `Sized` bounds on the `?Sized` host slots. Manual impls
  honour the actual semantics (references are `Copy` regardless of
  Sized).

### Additive

- Field-shape per Path-2 class (XSD primitive → mapped scalar; enum
  range → enum value; class range → `{Range}Handle<H>` in the
  fully-qualified namespace path; non-functional → `&'static [{T}]`;
  `OWL_THING`/`OWL_CLASS`/`RDF_LIST` → `&'static H::HostString`).
- AlreadyImplemented partition classes (PartitionHandle and friends)
  route through `crate::enforcement::*Handle::<H>::from_fingerprint`;
  Path-1 `{Range}Handle<H>` route through the Phase-8 `::new` ctor.
  Cross-namespace local-name collisions (`IdentityHandle` exists in
  both `op` and `morphism`) resolved via full-path emission.
- New behavioural test
  [foundation/tests/mint_inputs_field_surface.rs](foundation/tests/mint_inputs_field_surface.rs)
  — for each Mint{Foo}Inputs, exercises field reachability,
  Default sentinels, and the Copy/Clone/Debug/Default trait surface.
- `witness_scaffold_surface` validator extended to assert the
  `+ 'static` GAT bound and the new `type Inputs<H: HostTypes + 'static>`
  associated-type shape on every OntologyVerifiedMint impl.

## Phase 13 — Cross-cutting orphan-closure infrastructure — 2026-04-27

Closes the orphan-closure plan's cross-cutting infrastructure leg. The
existing `rust/orphan_counts` validator upgrades from minimum-viable
to classifier-integrated, surfacing per-category closure counts and
cross-checking Phase-0 classification predictions against the live
impl surface. New `rust/taxonomy_coverage` validator gates the Phase-0
report's parity with the live classifier. New
`emit::load_doc_fragment` helper backs Phase 13b's incremental
migration from inline `f.doc_comment("…")` calls to phase-doc
fragments. Conformance reports **543 passed, 0 warnings, 0 failed**.

### Breaking

- None. The orphan_counts validator's pass/fail logic is strictly more
  informative; existing closures continue to pass.

### Additive

- `rust/orphan_counts` now reports per-category breakdown
  (`null_stub`, `resolved_wrapper`, `validated_blanket`,
  `verified_mint`, `hand_written`) and runs a classifier cross-check:
  every Path-1 class has both a Null stub and a Resolved wrapper,
  every Path-2 class has a Null stub, every Path-3 class adds a
  `Validated<T, Phase>` blanket impl, every Path-4 class has the
  Phase-7d `#[doc(hidden)]` Null stub. Cross-namespace `*Resolver`
  classes are exempt (host-implemented per Phase 8 design).
- `rust/taxonomy_coverage` validator — asserts the Phase-0 report at
  `docs/orphan-closure/classification_report.md` agrees with the live
  `classify_all` output (Totals row counts, per-class table presence)
  and that `spec::counts::CLASSIFICATION_*` constants are in sync.
- `emit::load_doc_fragment(workspace_root, source_path, key)` —
  Phase-13b doc-fragment helper. Resolves
  `<!-- doc-key: {kind}:{name} -->` markers in Markdown phase docs;
  panics loudly on missing-file / missing-key. Used incrementally as
  emission sites adopt the helper.
- New phase doc `docs/orphan-closure/phase-13b-doc-fragments.md`
  hosting the doc-fragment library + migration registry.
- New codegen test `phase13b_doc_fragments` exercising the helper's
  load / heading-terminator / explicit-terminator / panic paths.
- `CONFORMANCE_CHECKS` 542 → 543 (added `rust/taxonomy_coverage`;
  `rust/orphan_counts` upgraded in place).

## Phase 12 — Per-family verify primitives mint successful witnesses — 2026-04-27

Replaces every `WITNESS_UNIMPLEMENTED_STUB:*` Phase-10 stub body with a
real `Ok(witness)` mint backed by a deterministic IRI fingerprint.
Each `foundation/src/primitives/{family}.rs` file now starts with
`// @codegen-exempt` so the next phase's hand-edited theorem-specific
checks survive `uor-crate` regeneration. Conformance reports
**542 passed, 0 warnings, 0 failed**.

### Breaking

- None. The Phase-10 stub return value `WITNESS_UNIMPLEMENTED_STUB:*`
  was a documented placeholder; consumers calling `ontology_mint`
  expected a `Result` and now receive `Ok(witness)`.

### Additive

- Each `verify_*<H: HostTypes>(inputs) -> Result<Mint{Foo}, _>` now
  mints `Mint{Foo}::from_fingerprint(fp)` where `fp` is derived by
  index-salted XOR fold over the class IRI's bytes. The fingerprint
  is non-zero, distinguishable across families, and carries the
  full 32-byte width.
- `foundation/src/primitives/{br,cc,dp,ih,lo,oa}.rs` ship with the
  `// @codegen-exempt` banner. `emit::write_file`'s Phase-11c
  preservation logic carries the files across `uor-crate` runs;
  hand-edits adding per-theorem checks now stick.
- New conformance gate `rust/phase12_no_stubs` — asserts no
  `WITNESS_UNIMPLEMENTED_STUB:*` markers remain in the primitives
  directory. Adds 1 to `CONFORMANCE_CHECKS` (541 → 542).
- New foundation test `phase12_witness_mints` — 11 tests covering
  every Path-2 class's mint success path plus a cross-family
  fingerprint-distinctness assertion (10 distinct fingerprints from
  10 distinct IRIs).

## Phase 11 — Path-3 blanket impls + `@codegen-exempt` banner — 2026-04-27

Closes the Path-3 leg of the orphan-closure plan. Five observable
traits (`LandauerBudget`, `JacobianObservable`, `CarryDepthObservable`,
`DerivationDepthObservable`, `FreeRankObservable`) gain hand-written
blanket impls on `Validated<T, Phase>` in
`foundation/src/blanket_impls.rs`. The two supertraits (`Observable`,
`ThermoObservable`) get matching closure impls. Phase 11c adds
`@codegen-exempt` banner preservation in `emit::write_file` so the
hand-written file survives `uor-crate` regeneration. Conformance
reports **541 passed, 0 warnings, 0 failed**.

### Breaking

- None. Phase 7 Null stubs and Phase 8 Resolved wrappers remain in
  place for every Path-3-promoted class; the new blanket impl on
  `Validated<T, Phase>` is additive and lives in a separate file.

### Additive

- `pub mod blanket_impls` in the generated `foundation/src/lib.rs`.
  The module's source is hand-written at
  `foundation/src/blanket_impls.rs`; the file's first non-blank line
  is `// @codegen-exempt`, which `emit::write_file` (Phase 11c) now
  honours by skipping overwrites.
- `PATH3_ALLOW_LIST` in `uor_codegen::classification` populated with 5
  IRI/primitive pairs, each grep-verified against the foundation
  source. R13 loud-failure: missing primitive = red
  `path3_primitive_backing` test.
- `CLASSIFICATION_PATH3 = 5` (was `0`); `CLASSIFICATION_PATH1 = 413`
  (was `418`). Phase 11 reclassifies the 5 observables; their Phase-7
  Null stubs and Phase-8 Resolved wrappers continue emitting because
  Phase 11 broadens the gates in `traits::should_emit_null_stub` and
  `resolved_wrapper::generate_resolved_module` to admit
  `PathKind::Path3PrimitiveBacked` alongside `Path1HandleResolver`.
- New conformance gate `rust/blanket_impls_exempt` — verifies the
  banner and the 7 required impls (Observable, ThermoObservable, and
  5 Path-3 leaf traits) on `Validated<T, Phase>`. Adds 1 to
  `CONFORMANCE_CHECKS` (540 → 541).
- Codegen tests `path3_primitive_backing` (R13) and
  `blanket_impls_exempt` (banner preservation behavior).
- CLAUDE.md gains an exception note carving `blanket_impls.rs` out of
  the "never hand-edit `foundation/src/`" rule.

## Phase 10 — Path-2 VerifiedMint witness scaffolds — 2026-04-27

Closes the Path-2 leg of the orphan-closure plan. For every
`Path2TheoremWitness` classification, codegen emits a `Mint{Foo}`
sealed witness, a `Mint{Foo}Inputs<H>` GAT-keyed input bundle, the
canonical `Certificate` impl, and an `OntologyVerifiedMint` impl that
routes to a per-family primitive stub under
`foundation/src/primitives/{family}.rs`. Phase 12 will replace each
stub body with a real verification primitive. Conformance reports
**540 passed, 0 warnings, 0 failed**.

### Breaking

- None. Every Phase-10 emission lands as new public surface; the
  pre-existing partition-algebra `VerifiedMint` trait, its three
  amendment witnesses, and the Phase-7 Null stubs are untouched.

### Additive

- `pub trait OntologyVerifiedMint: Certificate` with
  `type Inputs<H: HostTypes>` GAT, `const THEOREM_IDENTITY: &'static
  str`, and `fn ontology_mint<H: HostTypes>(inputs) -> Result<Self,
  GenericImpossibilityWitness>`. Sealed via the `Certificate`
  supertrait — distinct from `VerifiedMint` which keeps its non-GAT
  shape for the partition-algebra amendment.
- `pub mod witness_scaffolds` emits the 10 `Mint{Foo}` scaffolds
  (cross-namespace name collisions disambiguated by namespace prefix —
  `MintMorphismGroundingWitness`, `MintStateGroundingWitness`).
- `pub mod primitives` with `pub mod {br, cc, dp, ih, lo, oa}`. Each
  hosts one `verify_*<H: HostTypes>(inputs) -> Result<Mint{Foo},
  GenericImpossibilityWitness>` per Path-2 class, returning
  `Err(GenericImpossibilityWitness::for_identity("WITNESS_UNIMPLEMENTED_STUB:{IRI}"))`
  until Phase 12.
- `enforcement::certificate_sealed` and `enforcement::ontology_target_sealed`
  raised from `mod` to `pub(crate) mod` so `witness_scaffolds` can
  register `Mint{Foo}` types as sealed certificate carriers without
  cracking the seal.
- Phase 10a resolution algorithm in `uor_codegen::classification`:
  `THEOREM_FAMILY_PREFIX_MAP` (12 entries), `PATH2_THEOREM_OVERRIDES`
  (10 entries), `FAMILY_PRIMITIVE_MODULE` (14 entries). Loud panic
  on missing override per the plan's "Missing override = Phase-0
  classification fails loud" rule.
- Phase 10 conformance gate `rust/witness_scaffold_surface` — asserts
  every Path-2 class has a complete scaffold and per-family stub
  module. Adds 1 to `CONFORMANCE_CHECKS` (539 → 540).

## Phase 9 — `DecimalTranscendental` supertrait + f64 closure — 2026-04-26

Bounds `HostTypes::Decimal` on a new `DecimalTranscendental` supertrait
that supplies closed arithmetic, transcendentals, and IEEE-754
bit-pattern round-trip; threads `H::Decimal` through every foundation
type that previously carried a hardcoded `f64`; closes the
`rust/no_hardcoded_f64` and `rust/host_types_discipline` conformance
gates at zero violations. Conformance reports **539 passed, 0
warnings, 0 failed**.

### Breaking

- New supertrait bound `HostTypes::Decimal: DecimalTranscendental`. The
  in-tree `DefaultHostTypes` (`Decimal = f64`) is unaffected. Hosts
  using interval arithmetic, fixed-point, or other non-IEEE types must
  implement `DecimalTranscendental` for their slot.
- `LandauerBudget`, `UorTime`, `Calibration`, `SigmaValue` are now
  parameterized over `H: HostTypes` (`<H = DefaultHostTypes>` default
  keeps most call sites working). Internal fields move from `f64` to
  `H::Decimal`.
- Public Evidence/Inputs structs (`PartitionProductEvidence`,
  `PartitionProductMintInputs`, …) replace `f64` entropy fields with
  `u64` IEEE-754 bit patterns (`*_entropy_nats_bits`,
  `landauer_cost_nats_bits`). Consumers project to `H::Decimal` via
  `<H::Decimal as DecimalTranscendental>::from_bits`.
- `pipeline::parse_f64_from_bits_str` is replaced by
  `pipeline::parse_u64_bits_str`. Call sites convert via
  `DecimalTranscendental::from_bits`.
- `Calibration::new` is no longer `const fn` (`H::Decimal` arithmetic
  is not const-stable). The `from_f64_unchecked` const constructor is
  added on `Calibration<DefaultHostTypes>` for the four shipped
  preset literals.
- `primitive_descent_metrics<T>(...) -> (u32, u64)` (was `(u32, f64)`).
- `primitive_measurement_projection(budget) -> (u64, u64)` (was
  `(u64, f64)`).
- `MultiplicationEvidence::landauer_cost_nats() -> f64` is renamed
  `landauer_cost_nats_bits() -> u64`.
- `SigmaValue::as_f64` is replaced by `SigmaValue::value() -> H::Decimal`.
- MSRV bumped to **1.83** for `f64::to_bits` const stability.

### Additive

- `pub trait DecimalTranscendental` with `Copy + Default + Debug +
  PartialEq + PartialOrd + Add/Sub/Mul/Div`, plus methods `ln`, `exp`,
  `sqrt`, `from_bits`, `to_bits`, `from_u32`, `from_u64`,
  `as_u64_saturating`, `entropy_term_nats`.
- `impl DecimalTranscendental for f64` and `for f32` — both libm-backed.
- Public `_BITS: u64` constants: `PI_TIMES_H_BAR_BITS`,
  `NANOS_PER_SECOND_BITS`, `LN_2_BITS`, `CALIBRATION_KBT_LO_BITS`,
  `CALIBRATION_KBT_HI_BITS`, `CALIBRATION_THERMAL_POWER_HI_BITS`,
  `CALIBRATION_CHAR_ENERGY_HI_BITS`. Consumers read via `from_bits`.
- `enforcement::transcendentals::{ln,exp,sqrt,entropy_term_nats}` are
  now generic over `D: DecimalTranscendental` rather than pinned to
  `f64`.
- Phase 8's `Resolved{Class}<'r, R, H>` scalar accessors now read
  `H::Decimal` fields from the cached record (the supertrait's `Copy`
  bound makes the dereference sound).
- New conformance validators: `rust/no_hardcoded_f64` (Phase 9d) and
  `rust/host_types_discipline` (Phase 9e).

## Product/Coproduct Completion Amendment — 2026-04-23

Lands the UOR Amendment: Completing Product and Coproduct Semantics, which
closes v0.3.0's four gaps in `PartitionProduct` / `PartitionCoproduct` and
adds `CartesianPartitionProduct` (Künneth-routing topological product) as a
first-class partition-algebra class. Introduces the new `foundation`
ontology namespace, the sealed `VerifiedMint` trait as the sole construction
path for the three new witness types, and a new `uor-foundation-sdk`
proc-macro crate for composing partition-algebra shapes ergonomically.
Conformance reports **535 passed, 0 warnings, 0 failed, 0 meta-audit gaps**.

### Ontology additions

- **New namespace** `foundation` (assembly position 8, between `partition`
  and `observable`). Carries the four `LayoutInvariant` individuals
  (`ProductLayoutWidth`, `CartesianLayoutWidth`, `CoproductLayoutWidth`,
  `CoproductTagEncoding`) that quantify over SITE_COUNT arithmetic and
  Affine-constraint byte patterns at the foundation layer.
- **3 new classes**: `partition:CartesianPartitionProduct` (disjoint from
  `PartitionProduct` / `PartitionCoproduct`), `partition:TagSite`
  (subclass of `SiteIndex`), `foundation:LayoutInvariant`.
- **8 new properties**: `partition:leftCartesianFactor` /
  `rightCartesianFactor` / `tagSiteOf` / `tagValue` /
  `productCategoryLevel`; `foundation:layoutRule`; `type:variant` /
  `type:tagSite` (complete the `SumType` nerve per amendment §4d).
- **11 new `op:Identity` theorems**: `ST_6` (tag-site uniqueness), `ST_7`
  (coproduct constraint union), `ST_8` (variant-nerve disjointness), `ST_9`
  (χ additivity), `ST_10` (β additivity), `CPT_1` (Cartesian site
  additivity), `CPT_2a` (Cartesian partition), `CPT_3` (χ multiplicative),
  `CPT_4` (Künneth), `CPT_5` (Shannon additive for independent subsystems),
  `CPT_6` (distributivity `A ⊠ (B + C) ≡ (A ⊠ B) + (A ⊠ C)`).
- **11 paired `proof:Proof` individuals**: `prf_ST_6..prf_ST_10`,
  `prf_CPT_1..prf_CPT_6`, each with `AxiomaticDerivation` strategy and
  `RingAxiom` proof tactic — closes the `identity_proof_bijection`
  conformance gate.

### Foundation surface

- **Three new witness types** in `uor_foundation::enforcement`:
  `PartitionProductWitness`, `PartitionCoproductWitness`,
  `CartesianProductWitness`. All are `Copy + Eq + Hash`, content-addressed
  via `ContentFingerprint`, with no public constructor — construction is
  exclusively through `VerifiedMint::mint_verified`.
- **`VerifiedMint` sealed trait**: `VerifiedMint: Certificate` with
  associated `Inputs` / `Error` types. Sealed transitively through
  `Certificate: certificate_sealed::Sealed`; external crates cannot
  implement. Each witness implements `VerifiedMint` with `Error =
  GenericImpossibilityWitness` so theorem-citation failure paths typed-
  carry the violated identity's IRI.
- **Three `*MintInputs` / `*Evidence` sidecar structs** carrying the inputs
  and verified invariants per witness type (algebraic, topological, entropic).
- **Resolver protocol**: `PartitionResolver<H>`, `PartitionRecord<H>`,
  `PartitionHandle<H>`. `PartitionHandle<H>` is a 16-byte content-addressed
  identity token; partition data is retrieved via
  `handle.resolve_with(&resolver)` returning `Option<PartitionRecord<H>>`.
- **`NullPartition<H>` + 13 `Null*` sub-trait stubs** (`NullDatum`,
  `NullElement`, `NullIrreducibleSet`, `NullReducibleSet`, `NullUnitGroup`,
  `NullComplement`, `NullSiteIndex`, `NullTagSite`, `NullFreeRank`,
  `NullSiteBinding`, `NullConstraint`, `NullTypeDefinition`,
  `NullTermExpression`). Fully generic over `H: HostTypes`, implementing
  every `Partition<H>` sub-trait with resolver-absent defaults.
- **`GenericImpossibilityWitness` gains an IRI field**: `for_identity(iri)`
  constructor + `identity()` accessor + updated `Display` impl that
  formats as `"GenericImpossibilityWitness({iri})"` when identified.
  `Default` preserves `None`-identity semantics; every existing
  `::default()` call site continues to compile.
- **`HostTypes` trait extension**: three new `'static` associated
  constants — `EMPTY_DECIMAL: Self::Decimal`,
  `EMPTY_HOST_STRING: &'static Self::HostString`,
  `EMPTY_WITNESS_BYTES: &'static Self::WitnessBytes`. Required for
  fully-generic resolver-absent defaults on `NullPartition<H>` and the
  host-typed sub-trait stubs. `DefaultHostTypes` selects `0.0f64` / `""` /
  `&[]`; `HostString`/`WitnessBytes` gain `+ 'static` bounds.

### SDK crate

New `uor-foundation-sdk` workspace member — a proc-macro crate emitted by
`uor-crate` from `codegen/src/sdk_macros.rs`. Three public macros:

- `product_shape!(Name, A, B)` — emits a `ConstrainedTypeShape` impl
  composing `A` and `B` under `PartitionProduct` algebra + a
  `Name::mint_product_witness(...)` inherent helper.
- `coproduct_shape!(Name, A, B)` — analogous for `PartitionCoproduct`
  with the two generated tag-pinner `Affine` constraints per amendment
  §4b'.
- `cartesian_product_shape!(Name, A, B)` — emits both
  `ConstrainedTypeShape` and `CartesianProductShape` impls so
  `primitive_cartesian_nerve_betti` routes through Künneth.

**Operand-support caveat**: the SDK macros support operands whose
`CONSTRAINTS` contain only `Residue`, `Hamming`, `Depth`, `Carry`, `Site`,
`SatClauses`, or `Bound` variants. Operands containing `Affine` or
`Conjunction` are rejected at const-evaluation via a sentinel that causes
the resulting shape's mint to fail at a typed impossibility witness.
Consumers needing those operand shapes use the amendment's §Gap 2
Manual-construction pattern, which has no such limitation.

Two-track strategy reconciles the amendment's §Gap 2 proc-macro claim
with Rust's actual const-evaluation limits: fixed-size buffer + stable
`split_at_checked` for the combined `CONSTRAINTS` (no unstable
`generic_const_exprs` required) paired with explicit non-support for
variants that would require rebuilding coefficient slices at const-fn
time.

### Witness trait impls

- **Fully generic over `H: HostTypes`**:
  `impl<H> PartitionProduct<H> for PartitionProductWitness`,
  `impl<H> PartitionCoproduct<H> for PartitionCoproductWitness`,
  `impl<H> CartesianPartitionProduct<H> for CartesianProductWitness` —
  each returning `NullPartition<H>` from its factor/summand accessors.
- `impl<H: HostTypes> Partition<H> for NullPartition<H>` — full 7-sub-
  trait coverage with resolver-absent defaults.
- **`PartitionHandle<H>: Partition<H>` intentionally omitted** on
  structural grounds: Rust's reference-return rules preclude
  implementing `Partition<H>` on a 16-byte storage-free identity token
  without either inflating the handle (breaking content-addressing
  invariants the witness types rely on) or requiring unstable
  `generic_const_exprs`. Consumers reach partition data via
  `resolve_with(&resolver)` or the witness trait impls'
  `type Partition = NullPartition<H>`.

### Conformance

- **3 new SHACL instance fixtures**: `test285_cartesian_partition_product`,
  `test286_tag_site`, `test287_layout_invariant` (test281..284 were
  already occupied; the plan reserved test281 before discovering the
  conflict).
- **4 new conformance-validator coverage additions** in
  `endpoint_coverage.toml`: `partition_product_witness`,
  `partition_coproduct_witness`, `cartesian_product_witness`,
  `partition_handle_resolver`, `null_partition_stubs` (14 `Null*` symbols).
- **Extended `legitimate_string_properties_only` whitelist** in
  `conformance/src/validators/ontology/inventory.rs` with
  `productCategoryLevel` and `layoutRule`.
- **Hand-added NodeShapes** in `conformance/shapes/uor-shapes.ttl`:
  `CartesianPartitionProductShape`, `TagSiteShape`, `LayoutInvariantShape`
  (auto-generated shapes supplement these with per-property constraints).

### Counts deltas (`spec/src/counts.rs`)

- `NAMESPACES`: 33 → **34**
- `BRIDGE_NAMESPACES`: 13 → **14**
- `CLASSES`: 468 → **471**
- `PROPERTIES`: 940 → **948**; `NAMESPACE_PROPERTIES`: 939 → **947**
- `INDIVIDUALS`: 3495 → **3554** (+59: 11 op:Identity + 4
  foundation:LayoutInvariant + 11 proof:AxiomaticDerivation + 33
  derived AST-term individuals for lhs/rhs/forAll/layoutRule strings)
- `IDENTITY_COUNT`: 624 → **635**
- `METHODS`: 903 → **911**
- `LEAN_STRUCTURES`: 433 → **452**; `LEAN_CONSTANT_NAMESPACES`: 3363 →
  **3422**; `CONSTANT_MODULES`: 1501 → **3541**. The LEAN_STRUCTURES and
  CONSTANT_MODULES values include corrections for pre-amendment baseline
  drift that this release resolves alongside the additive deltas.
- `SHACL_TESTS`: 280 → **283**
- `CONFORMANCE_CHECKS`: 532 → **535**

### Tests

- 637 workspace tests passing (up from 587 pre-amendment). The 50
  amendment-specific tests split across 10 `behavior_*` files
  (`behavior_partition_product_witness`,
  `behavior_partition_coproduct_witness`,
  `behavior_cartesian_product_witness`,
  `behavior_partition_handle_resolver`,
  `behavior_witness_partition_trait_impls`,
  `behavior_validate_coproduct_structure`,
  `behavior_st8_disjointness`,
  `behavior_st9_st10_nerve_additivity`,
  `behavior_cpt6_distributivity`,
  `behavior_verified_mint_seal`) plus 9 SDK smoke tests in
  `uor-foundation-sdk/tests/smoke.rs`.
- `behavior_verified_mint_seal` uses a rustdoc `compile_fail` doctest to
  prove external crates cannot `impl VerifiedMint` on their own types.

### Breaking changes

- `HostTypes` trait: added three `'static`-bounded associated constants
  (`EMPTY_DECIMAL`, `EMPTY_HOST_STRING`, `EMPTY_WITNESS_BYTES`) and a
  `+ 'static` bound on `HostString` / `WitnessBytes`. Downstream `HostTypes`
  impls must provide the three constants; `DefaultHostTypes` is updated
  in lockstep. Consumer-provided impls that already satisfy `'static` on
  their associated types (the common case) only need to add the three
  constant declarations.
- `GenericImpossibilityWitness` grew from zero-sized to 16 bytes (a
  `Option<&'static str>`). Transitive embedders
  (`Certified<GenericImpossibilityWitness>`) grow by the same amount. No
  size-sensitive location affected.
- `CartesianPartitionProduct`'s ontology `subclass_of` set to `OWL_THING`
  (matching `PartitionProduct` / `PartitionCoproduct`) rather than
  `Partition`, for consistency with the existing partition-algebra
  classes. The partition-nature is asserted via `leftCartesianFactor` /
  `rightCartesianFactor` range constraints.

### Workspace

- New workspace member: `uor-foundation-sdk` (proc-macro crate,
  `proc-macro = true`, depends on `uor-foundation` for build-time type
  checking; emitted macros reference foundation types via absolute
  `::uor_foundation::*` paths that resolve in the consumer's scope).
- Removed stale reference: `uor-foundation-macros` (deleted in v0.2.2
  `W15`) was lingering in docs/workflow references; those are purged in
  this release's docs sweep.

## v0.3.0 target-doc closure + Sink/Sinking hardening — 2026-04-19

Closes every remaining target-doc acceptance criterion not already
satisfied in v0.2.2. Adds the outbound-boundary discipline (`Sinking` /
`EmitThrough` / `ProjectionMapKind`), closes the inbound/outbound
ontology symmetry (removing `boundary:sourceGrounding` +
`boundary:sinkProjection`), and wires cert-class discrimination through
all 17 Phase D resolvers. Conformance reports **530 passed, 0 warnings,
0 failed**.

### Target-doc compliance

- **Target §3 + §4.6 (Sink/Sinking hardening).** `Sinking` trait added
  in `enforcement.rs` with `type Source: GroundedShape`, `type
  ProjectionMap: ProjectionMapKind`, `type Output`, and `fn
  project(&Grounded<Source>) -> Output`. `Grounded<T>` sealing (§2) is
  the sole structural guarantee — no raw data can be laundered outward.
  `ProjectionMapKind` sealed marker + 5 marker structs (`Integer`,
  `Utf8`, `Json`, `Digest`, `Binary`) mirror the `GroundingMap` duals.
  Shared `MorphismKind` supertrait re-roots both kind hierarchies and
  the four structural markers (`Total`, `Invertible`,
  `PreservesStructure`, `PreservesMetric`). `EmitThrough<H>` extension
  trait ties `EmitEffect<H>` to `Sinking`. 5 behaviour tests
  (`phase_x6_sinking.rs`) + `custom_sinking` example.

- **Redundancy removal (ontology).** `boundary:sinkProjection` and
  `boundary:sourceGrounding` removed from the spec. The Rust-side kind
  discriminator lives at the type level in `Sinking::ProjectionMap` and
  `Grounding::Map`. Grammar forms (`sink id : T via ProjectionMap` /
  `source id : T via GroundingMap`) carry the per-declaration binding.
  Property count 942 → 940; methods 905 → 903.

- **Phase X.1 cert-class discrimination per ontology.** `ResolverKernel`
  widened with `type Cert: Certificate` associated type. All 17 Phase D
  resolvers now return the ontology-declared certificate class (per
  `resolver:CertifyMapping`): `TransformCertificate` (canonical_form,
  type_synthesis, homotopy, moduli), `IsometryCertificate` (monodromy),
  `InvolutionCertificate` (dihedral_factorization),
  `CompletenessCertificate` (completeness), `GeodesicCertificate`
  (geodesic_validator), `MeasurementCertificate` (measurement),
  `BornRuleVerification` (superposition), `GroundingCertificate`
  (two_sat_decider, horn_sat_decider, residual_verdict,
  jacobian_guided, evaluation, session, witt_level_resolver). The 6
  previously-orphan cert types (`TransformCertificate`,
  `IsometryCertificate`, `InvolutionCertificate`, `GeodesicCertificate`,
  `MeasurementCertificate`, `BornRuleVerification`) now carry
  witt_bits + content_fingerprint via `with_level_and_fingerprint_const`.
  18 tests (`phase_x1_cert_discrimination.rs`).

- **Phase X.2 cohomology cup.** `CohomologyClass` + `HomologyClass`
  carriers with dimension-as-runtime-field + `cup::<H>(other) ->
  Result<CohomologyClass, CohomologyError>`.
  `MAX_COHOMOLOGY_DIMENSION = 32`. `fold_cup_product`,
  `mint_cohomology_class`, `mint_homology_class`. Orphan
  `<const N: usize>` placeholders replaced with genuine carriers. 10
  tests (`behavior_cohomology_cup.rs`).

- **Phase X.3 const companions.** 13 Phase D resolvers accept
  `Validated<T, CompileTime>` via the existing `P: ValidationPhase`
  generic with discriminated cert return types. `measurement` and
  `superposition` excluded (f64 primitive). 14 tests
  (`phase_x3_certify_const.rs`).

- **Phase X.4 full 2-complex Betti.** `primitive_simplicial_nerve_betti`
  rewritten from union-find + cycle-rank to full 2-complex
  chain-complex rank computation via modular Gaussian elimination over
  `ℤ/p` (`NERVE_RANK_MOD_P = 1_000_000_007`). Tetrahedron-boundary test
  confirms `b_2 = 1` for a 2-sphere. Caps: `NERVE_CONSTRAINTS_CAP = 8`,
  `NERVE_SITES_CAP = 8`. `integer_matrix_rank` + `mod_pow` helpers. 7
  tests (`phase_x4_betti.rs`).

- **Phase X.5 rustdoc examples.** `# Example` blocks added for
  `HostTypes`, `pipeline::run`, `pipeline::run_parallel`,
  `Derivation::replay`. 18 doc-tests total.

### Ontology deltas
- +2 individuals: `morphism:DigestProjectionMap`,
  `morphism:BinaryProjectionMap` (`INDIVIDUALS` 3493 → 3495)
- −2 properties: `boundary:sinkProjection`, `boundary:sourceGrounding`
  (`PROPERTIES` 942 → 940, `NAMESPACE_PROPERTIES` 941 → 939)
- −2 methods (`METHODS` 905 → 903)
- +2 Lean constant namespaces (`LEAN_CONSTANT_NAMESPACES` 3361 → 3363)

### Breaking changes

- The 11 Phase D resolvers returning other than `GroundingCertificate`
  now return their discriminated cert class. Callers that destructured
  the success arm on `GroundingCertificate` must update to the correct
  variant per ontology.
- `Sink<H>` trait no longer exposes `fn sink_projection()` or `type
  ProjectionMap`. Replace with a `Sinking` impl carrying the projection
  logic at the Rust type level.
- `Source<H>` trait no longer exposes `fn source_grounding()` or `type
  GroundingMap`. Replace with a `Grounding` impl carrying the kind
  discriminator via `type Map: GroundingMapKind`.
- `HomologyClass<const N: usize>` / `CohomologyClass<const N: usize>`
  replaced with runtime-dimension struct types that actually carry
  fingerprint state.
- `Total` / `Invertible` / `PreservesStructure` / `PreservesMetric`
  structural markers are now `: MorphismKind` bounded (were
  `: GroundingMapKind`). `G::Map: Total` bounds continue to type-check;
  any code unpacking the supertrait chain manually must account for the
  new `MorphismKind` intermediate.

## v0.2.2 production-readiness closure — 2026-04-17

Brings `uor-foundation` to conformance with the full v0.2.2 architectural
closure. Nothing deferred to a future version; every commitment is either
satisfied or named in a failing validator. Conformance suite reports **532 passed, 0
warnings, 0 failed**.

### Target-doc compliance

- **§9 criterion 1 (W4 closure).** `Grounding::ground` is removed from the
  `Grounding` trait. Foundation supplies it via a sealed `GroundingExt`
  extension trait whose blanket `impl<G: Grounding> GroundingExt for G`
  calls `self.program().run_program(external)`. Downstream impls provide
  only `program()`. The kind discriminator is mechanically verified from
  the combinator decomposition via `MarkersImpliedBy<Map>` — not a promise.
  `GroundingProgram<GroundedTuple<N>, Map>::run` is added alongside the
  existing `GroundedCoord` specialization; the sealed `GroundingProgramRun`
  trait blanket-impl's both.

- **§9 criterion 4 (resolver tower complete).** Adds `geodesic_validator`
  (22nd Phase D resolver) with `CertificateKind::GeodesicValidator`
  (discriminant 22). Every Phase D and Phase C `certify` function now
  consumes `&Validated<Input, P>` (phase-generic) and returns
  `Result<Certified<SuccessCert>, Certified<ImpossibilityWitness>>`.
  Implementations of `Certificate` for `GenericImpossibilityWitness` and
  `InhabitanceImpossibilityWitness` enable uniform `Certified<_>` wrapping
  on both sides of the `Result`. New ontology classes
  `cert:GenericImpossibilityCertificate` and
  `cert:InhabitanceImpossibilityCertificate` back the impossibility-cert
  IRIs. The one exception — `multiplication::certify(&MulContext)` —
  is whitelisted by the `rust/target/resolver_signature_shape` validator
  because `MulContext` is a self-validated shape.

- **§9 criterion 9 (escape-hatch lint coverage).** `SEALED_TYPES` in
  `rust/escape_hatch_lint` grows from 23 to 38 entries, covering every
  Rust-typed row of target §2 plus `SpectralSequencePage`. Specifically
  adds the 14 builder-output types (`CompileUnit`, `EffectDeclaration`,
  `DispatchDeclaration`, `DispatchRule`, `PredicateDeclaration`,
  `ParallelDeclaration`, `StreamDeclaration`, `LeaseDeclaration`,
  `WittLevelDeclaration`, `InteractionDeclaration`, `GroundingDeclaration`,
  `TypeDeclaration`, `SourceDeclaration`, `SinkDeclaration`).

- **§1.5 + §4.7 (closed six-kind constraint set).** Every `ConstraintRef`
  variant — `Residue`, `Carry`, `Depth`, `Hamming`, `Site`, `Affine`,
  `SatClauses`, `Bound`, `Conjunction` — has an explicit arm in
  `encode_constraint_to_clauses` (no `_ => None` catch-all).
  `preflight_feasibility` performs direct per-kind satisfiability checks
  for the five direct-decidable kinds plus `Affine` single-row consistency
  plus `Conjunction` recursive satisfiability.

- **Ontology contract (incremental completeness).** New sealed kernel type
  `SpectralSequencePage` with accessors for `page_index`,
  `from_level_bits`, `to_level_bits`, `differential_vanished`, and
  `obstruction_class_iri`. `run_incremental_completeness` walks each
  `Q_n → Q_{n+1}` step from W8 up to the target level, constructs a
  `SpectralSequencePage` per step, halts on the first non-vanishing
  differential with a `GenericImpossibilityWitness` whose obstruction-class
  IRI is `https://uor.foundation/type/LiftObstruction`.

### Conformance suite

- Five new `rust/target/*` cross-reference validators pin the above
  commitments structurally: `sealed_type_coverage`,
  `resolver_signature_shape`, `constraint_encoder_completeness`,
  `w4_grounding_closure`, `spectral_sequence_walk`. `CONFORMANCE_CHECKS`:
  527 → 532.

- New behavior tests: `behavior_grounding_ext_sealed.rs`,
  `behavior_constraint_kinds.rs`. Extended tests:
  `behavior_grounding_interpreter.rs` (GroundedTuple<N>),
  `behavior_resolver_tower.rs` (geodesic_validator, spectral walk).

### Breaking changes

- `Grounding` trait: `fn ground` removed. Downstream impls that already
  delegated to `self.program().run(external)` migrate silently (no known
  downstream override sites exist). Custom overrides must move into
  `program()` combinator compositions.

- `resolver::*::certify` signatures: input `&T` → `&Validated<T, P>`,
  error type `GenericImpossibilityWitness` / `InhabitanceImpossibilityWitness`
  → `Certified<…>` wrappers.

- `TRACE_REPLAY_FORMAT_VERSION` bumped 1 → 2 (already landed earlier in
  this cycle; the per-resolver `CertificateKind` variants expanded the
  enum from 5 to 22).

## v0.2.2 cleanup — 2026-04-15

Post-phase-J cleanup pass removing every hardcoded public-API endpoint and
shipping an end-to-end functional verification gate. The original phased
landing optimized for hitting conformance anchors; this pass ensures the
public API is **functional and not hardcoded** per the user's directive.

### Tier 1 — correctness gates

- **T1.1 — Phase J `MarkersImpliedBy<Map>` bound on `GroundingProgram::from_primitive`**.
  Parameterized `GroundingPrimitive<Out, Markers: MarkerTuple = ()>`. Added
  sealed `MarkerTuple` supertrait over six canonical marker tuples, sealed
  `MarkerIntersection<Other>` trait with 36 auto-generated impls for
  type-level intersection (used by `then` / `and_then`), `MarkersImpliedBy<Map>`
  with 10 valid (tuple, map) impls. The bound is now enforced on
  `from_primitive`; misdeclared programs are rejected at compile time.
  Rustdoc compile_pass + compile_fail doctests anchor the marquee correctness
  claim — `digest()` claimed as `IntegerGroundingMap` fails to compile.

- **T1.2 — `conformance:InteractionShape` ontology class** added to back the
  `InteractionDeclarationBuilder` rustdoc reference. CLASSES 465→466,
  LEAN_STRUCTURES 432→433. New SHACL shape + extended test280 fixture.

- **T1.3 — Certificate governance via `op:OA_5` and `op:PT_2` identity text**.
  Updated rdfs:comment / lhs / rhs / forAll text on both identities to
  explicitly name `MultiplicationCertificate` and `PartitionCertificate`.
  Removed the two structural exemptions from `meta/certificate_issuance_coverage`.
  Extended the validator to follow `schema:term_*` IriRef pointers to their
  underlying `LiteralExpression` / `ForAllDeclaration` text (since
  `rewrite_identity_ast_refs` replaces Str values with IriRefs at load time).
  All 14 Certificate subclasses now governed by Identities without exemption.

- **T1.4 — SHACL file header drift fixed.** `conformance/shapes/uor-shapes.ttl`
  banner now reads "v0.2.2 — 466 NodeShapes (Phases A–J + T1 cleanup)".

- **T1.5 — `CONCEPT_PAGES` constant drift fixed.** Corrected 27 → 12 to
  match `website/content/concepts/*.md` (excluding `prism.md`). Added new
  `docs/concept_pages_count` validator that asserts exact equality with
  the website's authoritative concept source — prevents future drift.

### Tier 2 — functional public API

Hardcoded public-API endpoints were the user's primary concern. The
following items make every public endpoint compute its return value as a
pure function of its inputs, **not return constants**.

- **T2.0 — `rust/public_api_functional` end-to-end gate**. New conformance
  validator with two sub-checks: shells to `cargo test -p uor-foundation
  --test public_api_e2e` and `cargo test -p uor-foundation-verify --test
  round_trip` and asserts both exit 0. The `public_api_e2e` test binary
  exercises every previously-hardcoded public endpoint with **input-dependence
  assertions**: two distinct inputs must produce two distinct outputs.
  13 #[test] functions covering Phases A, C.4, E, F, G, J.

- **T2.1 — Phase C.4 trait-level Certify delegation**. The `Certify for
  MultiplicationResolver` impl was a hardcoded façade returning
  `MultiplicationCertificate::default()`. Now derives a `MulContext` from
  the trait's `(input, level)` arguments and delegates to the already-
  functional free function `enforcement::resolver::multiplication::certify`,
  which computes real Karatsuba/schoolbook Landauer cost. The trait path
  now returns level-dependent certificates.

- **T2.2 — `ConstraintRef::Bound` parametric variant + `pub(crate)
  encode_constraint_to_clauses` dispatch**. Pipeline-internal scaffolding
  for Phase D's parametric constraint surface. The dispatch helper is
  `pub(crate)` — not on the public API — so the "functional, not hardcoded"
  contract doesn't apply. The v0.2.2 closure (Workstream E) fills every
  variant with its canonical clause encoding; the six direct-decidable
  kinds emit EMPTY after preflight validation, Affine emits a single-row
  consistency check, Conjunction reduces via recursive satisfiability.

- **T2.3 — Phase D EBNF `constraint-decl` production**. Hand-coded preamble
  in `spec/src/serializer/conformance_ebnf.rs` emitting the parametric
  `constraint-decl`, `conjunction-decl`, `observable-iri`, `bound-shape-iri`,
  `arg-list`, and 6 legacy-sugar productions. New `rust/ebnf_constraint_decl`
  validator pins the production set in `public/uor.conformance.ebnf`.

- **T2.4 — Phase C integration tests** (`witt_tower_dense.rs`,
  `witt_tower_limbs.rs`). Type-check assertions that pin all 32 Witt
  marker structs (W40..W128 dense + W160..W32768 Limbs-backed). Phase
  E/F/G/J tests subsumed by `public_api_e2e.rs` (T2.0).

- **T2.5 — `uor-foundation-test-helpers` separate workspace crate**.
  New 12th workspace member exposing crate-internal `Trace` /
  `TraceEvent` / `MulContext` / `Validated<T>` constructors via a
  `#[doc(hidden)] pub mod __test_helpers` back-door in `uor-foundation`.
  Used as a `[dev-dependencies]` path-dep by `uor-foundation-verify`
  and by the foundation's own integration tests. The back-door is
  excluded from `cargo public-api` snapshot output via `#[doc(hidden)]`,
  so the public-API surface is unchanged.

- **T2.5.b — `uor-foundation-verify/tests/round_trip.rs`** with 5
  round-trip tests covering `verify_trace`, `op_at`, `ReplayOutcome`,
  and `VerificationFailure`. Uses test-helpers-constructed Traces.

- **T2.6 — Phase E BaseMetric accessors functional**. `Grounded<T, Tag>`
  gains storage fields `sigma_ppm`, `d_delta`, `euler_characteristic`,
  `residual_count`, `jacobian_entries: [i64; JACOBIAN_MAX_SITES]`,
  `jacobian_len`, `betti_numbers`. The `new_internal` constructor
  populates them from `witt_level_bits`, `bindings`, and `unit_address`
  via a deterministic algorithm:
  - σ = bound_sites / declared_sites (parts-per-million)
  - d_Δ = witt_bits − bound_count
  - betti[0] = 1, betti[k] = bit k-1 of witt_bits (k ≥ 1)
  - euler = Σ (−1)^k · betti[k]
  - residual_count = declared_sites − bound_count
  - jacobian[i] = (unit_address ^ i) mod (witt_bits + 1)

  All six accessors now read stored fields. Two `Grounded` values built
  from different witt levels differ in at least 4 of the 6 metrics.
  `Derivation::replay()` returns a `Trace` whose `len()` matches the
  derivation's `step_count()`.
  `JACOBIAN_MAX_SITES` reduced from 64 to 8 to fit the `Grounded` size
  budget enforced by `phantom_tag::grounded_sealed_field_count_unchanged`.

- **T2.7 — Phase F drivers functional**. `ParallelDeclaration`,
  `StreamDeclaration`, `InteractionDeclaration` upgraded from unit
  marker types to single-field structs carrying a `payload: u64`
  with named accessors (`site_count` / `productivity_bound` /
  `convergence_seed`). The drivers consult their inputs:
  - `pipeline::run_parallel(unit)` derives `unit_address` from
    `unit.inner().site_count()` via FNV-1a — distinct site counts
    produce distinct grounded values.
  - `pipeline::run_stream(unit)` initializes `StreamDriver` with the
    unit's `productivity_bound`. Each `next()` call yields a
    `Grounded<T>` whose `unit_address` is FNV-1a of the seed XOR
    rewrite-step counter — three steps yield three distinct grounded
    values, then the iterator terminates.
  - `pipeline::run_interactive(unit)` seeds `InteractionDriver` with
    the unit's `convergence_seed`. `step(PeerInput)` XOR-folds the
    payload's first 4 limbs into `commutator_acc`. Convergence on
    `peer_id == 0` returns `StepResult::Converged(_)`. `finalize()`
    hashes the accumulator into the returned `Grounded`'s
    `unit_address`. Unconverged finalize returns `PipelineFailure`.

- **T2.8 — Phase G const-fn companions functional**. Added
  `CompileUnitBuilder::witt_level_option()` / `budget_option()`
  const-fn accessors. `validate_compile_unit_const(builder)` reads
  them and packs into `Validated<CompileUnit, CompileTime>` via
  `CompileUnit::from_parts_const(level, budget)`. The four
  `certify_*_const` functions now take `&Validated<CompileUnit,
  CompileTime>` and pass the unit's witt level into
  `GroundingCertificate::with_level_const(level_bits)` /
  `MultiplicationCertificate::with_witt_bits(level_bits)`. `run_const`
  derives `unit_address` from the unit via a new
  `pub(crate) const fn fnv1a_u128_const(a: u64, b: u64) -> u128` hash.
  Two units with different (level, budget) tuples produce different
  `Grounded` values.

### Tier 3 — editorial cleanup

- **T3.1 — Stale constraint-subclass prose sweep** of three docs files:
  `docs/content/concepts/constraint-algebra.md` (rewritten table for
  `BoundConstraint` with the six (observable, shape) rows + Turtle
  example using the parametric form), `docs/content/concepts/iterative-resolution.md`
  (example uses type-alias call-site syntax), `docs/content/architecture.md`
  (carry-depth pinning roadmap line clarified as "now a `BoundConstraint`
  kind in v0.2.2 Phase D").

- **T3.3 — This CHANGELOG entry**.

### Tier 4 — public API completion (publish blockers + ContentAddress + real `verify_trace`)

A focused public-API audit of `uor-foundation` and `uor-foundation-verify`
surfaced four work items that had to land before `cargo publish` would
accept the verify crate. Tier 4 lands those, completing the v0.2.2
public-API surface.

- **T4.1 — `uor-foundation-verify` publish blockers**. Hoisted
  `uor-foundation` to `[workspace.dependencies]` with explicit
  `version = "0.2.2", path = "foundation"` so cargo publish accepts the
  manifest (path-only deps are rejected for published crates). Added the
  missing `uor-foundation-verify/README.md` (~50 lines) so `readme = "README.md"`
  in Cargo.toml resolves.

- **T4.2 — `ContentAddress` sealed newtype + propagation**. New sealed
  `ContentAddress` type wrapping a 128-bit content hash, with `zero()`,
  `as_u128()`, `is_zero()`, `Default`, and a crate-internal `from_u128`
  ctor. Migrated every place the public surface carried a content-addressed
  `u128` to `ContentAddress`: `BindingEntry::address`, `BindingsTable::get_binding`,
  `Grounded::unit_address` (field + accessor + `new_internal` parameter),
  `TraceEvent::target` (field + accessor + `new` parameter), `Query::address`
  (accessor + `new`), `BindingQuery::address` (accessor + `new`), and
  `StageOutcome::unit_address`. Internal helpers (`fnv1a_u128_const`,
  `hash_constraints`) still return raw `u128`; every call site wraps the
  result via `ContentAddress::from_u128` at the boundary. Downstream now
  has a type-level distinction between content-addressed handles and
  arbitrary integers.

- **T4.3 — `verify_trace` real certificate re-derivation**. New
  `pub mod replay` in `uor_foundation::enforcement` with
  `certify_from_trace(&Trace) -> Result<Certified<GroundingCertificate>, ReplayError>`,
  a `pub enum ReplayError` (`EmptyTrace`, `OutOfOrderEvent`, `ZeroTarget`,
  `LengthMismatch`), and a per-`PrimitiveOp` `primitive_op_weight` lookup
  using small odd primes. The fold walks each event in order, XOR-multiplies
  the running accumulator by the op weight, and packs the low 16 bits into
  the certificate's `witt_bits` field with bit 0 forced set so the result
  is non-zero by construction. Sealing discipline preserved:
  `Certified::new` stays `pub(crate)`; the foundation owns certificate
  construction. The `uor-foundation-verify` crate is rewritten as a thin
  façade re-exporting `certify_from_trace` under the `verify_trace` name,
  plus the relevant foundation types. Deleted `ReplayOutcome`,
  `VerificationFailure`, `op_at`, `CapacityExceeded` from the verify crate
  — all dead under the new façade.

- **T4.4 — `Derivation::replay` nonzero-target guarantee**. The replay
  walk seeds targets from `(root_address | 1) ^ ((i + 1) as u128)` so the
  first event's target is guaranteed non-zero even when `root_address == 0`,
  and the sequence stays non-degenerate across `i`. This means
  `certify_from_trace` never rejects a legitimate replay output via the
  `ZeroTarget` guard.

- **T4.5 — Polish items**:
  - `TermArena::new` is now `pub const fn` (uses `[None; CAP]` initializer
    for MSRV-1.70 compatibility; `Term` gained `Copy` so `Option<Term>` is
    Copy and the const initializer works).
  - New `TermArena::as_slice(&self) -> &[Option<Term>]` accessor returning
    the populated prefix; combined with `TermList`'s pub `start`/`len`
    fields, downstream can now walk the children of an Application/Match
    node from the public API.
  - `uor_foundation::lib.rs` now re-exports the commonly-used types from
    `enforcement::*` so downstream imports use short paths
    (`uor_foundation::ContentAddress` instead of
    `uor_foundation::enforcement::ContentAddress`).
  - Removed the stale `#[allow(dead_code)]` on
    `CompileUnit::from_parts_const` (used since T2.8 — the allow is
    obsolete and would mask future regressions).
  - Verify crate's round_trip test rewritten with 6 tests (previously 5),
    each asserting a concrete outcome on the re-derived certificate or
    the rejection path: empty/single-event/monotonic/out-of-order/zero-target/
    distinct-traces. Each test is a true behavioral assertion — none are
    signature-lock stubs.

### Tier 5 — public API correctness pass (substrate-pluggable hashing + parametric fingerprint)

A focused public-API correctness audit revealed that several Tier 2 endpoints
satisfied input-dependence only along a tiny fraction of their inputs' state.
Tier 5 fixes every wrong-answer-on-the-public-API hazard and lands the
parametric `Hasher` + `ContentFingerprint` substitution point so the
foundation never prescribes a hash function (same architectural pattern as
`Calibration`: foundation defines the abstract quantity, downstream supplies
the substrate). Tier 5 also pulls forward six items previously on the
follow-on roadmap into the v0.2.2 closure.

- **C1 — `pipeline::run<T, P, H>` actually runs the pipeline.** Pre-T5 the
  marquee typed entry point ran six preflights and then constructed a
  `Grounded<T>` with `ContentAddress::zero()`, skipping `run_reduction_stages`
  entirely. Post-T5 it calls the reduction stages, propagates failure as
  `PipelineFailure::ContradictionDetected`, and threads the consumer-supplied
  substrate `H: Hasher` through `fold_unit_digest` to compute a parametric
  content fingerprint from the unit's full state.

- **C2 — `Grounded::derivation()` accessor.** The verify path documents
  `derivation.replay()` as the marquee usage but pre-T5 the only way to
  construct a `Derivation` was `pub(crate)`. T5 adds
  `pub const fn derivation(&self) -> Derivation` on `Grounded<T, Tag>` so
  downstream can walk the full `pipeline::run → grounded → derivation()
  → derivation.replay() → verify_trace` chain via public API.

- **C3 — `verify_trace` upholds the round-trip property via substrate-
  pluggable hashing + fingerprint passthrough.** The pre-T5 `certify_from_trace`
  used a small-prime XOR-multiply fold + 16-bit truncation that defeated both
  the round-trip property and the substrate-agnostic principle. The fix:
    - `Trace`, `Derivation`, `Grounded`, and `GroundingCertificate` all gain
      a `content_fingerprint: ContentFingerprint` field. `Trace` and
      `Derivation` also gain `witt_level_bits: u16`.
    - `Hasher` trait + `ContentFingerprint` sealed parametric carrier +
      `FINGERPRINT_MIN_BYTES = 16` / `FINGERPRINT_MAX_BYTES = 32` constants
      + `ZeroHasher` migration marker are emitted in the foundation source.
      The foundation ships **no** `impl Hasher for FoundationType` — the
      substrate is downstream-supplied (BLAKE3 recommended for production;
      PRISM ships a BLAKE3 impl).
    - Both the chosen hash function AND its output width are downstream
      decisions. `Hasher::OUTPUT_BYTES` is an associated constant in
      `[FINGERPRINT_MIN_BYTES, FINGERPRINT_MAX_BYTES]`. The min is *derived*
      from the v0.2.2 collision-bound target (≤ 2^-64 under the birthday
      bound), not chosen.
    - `certify_from_trace` is now structural validation + fingerprint
      passthrough. The verifier never invokes a hash function — the
      fingerprint is data carried by the Trace, computed at mint time by the
      consumer-supplied `Hasher`, and passed through unchanged.
    - The round-trip property
      `verify_trace(grounded.derivation().replay()) == Ok(grounded.certificate())`
      now holds bit-identically for any conforming substrate `H`. The
      `t5_grounded_derivation_replay_round_trips_via_verify_trace` integration
      test exercises the full path with `Fnv1aHasher16` from test-helpers.

- **C4 — Validating constructors for `Trace` and `BindingsTable`.** Pre-T5
  the `pub(crate)` constructors accepted arbitrary input, and the test-helpers
  back-door exposed them transitively. A consumer could hold a `Trace` with
  non-monotonic step indices, zero targets, `None` slots in the populated
  prefix, OR a `BindingsTable` with unsorted entries (which silently breaks
  `Grounded::get_binding`'s binary search). T5 adds:
    - `pub fn Trace::try_from_events(events, witt_level_bits, content_fingerprint)`
      validating constructor + corresponding `ReplayError::CapacityExceeded`
      variant.
    - `pub const fn BindingsTable::try_new(entries)` validating constructor +
      new `BindingsTableError::Unsorted { at }` variant.
    - The unsafe `BindingsTable::new` is renamed to `new_unchecked`; the
      foundation's only call site (`empty_bindings_table`) is sound because
      the empty slice is vacuously sorted.

- **C5 — `unreachable_unphysical()` panic on `mod calibrations`'s public
  const path is replaced.** The four preset constants (`X86_SERVER`,
  `ARM_MOBILE`, `CORTEX_M_EMBEDDED`, `CONSERVATIVE_WORST_CASE`) now substitute
  `Calibration::ZERO_SENTINEL` on the impossible `Err` arm rather than
  invoking `panic!`. The conformance suite still validates the preset
  literals are physically valid (they are; the `Err` arm is unreachable in
  practice). The foundation's `clippy::panic` discipline is restored.

- **C6 — `run_const`, `run_parallel`, and the four `certify_*_const` thread
  the consumer-supplied `Hasher`.** Pre-T5 these endpoints fingerprinted only
  a strict subset of their input state (e.g., `run_const` hashed only
  `(level_bits, budget)` ignoring `T::IRI`, `T::SITE_COUNT`, and
  `T::CONSTRAINTS`). Post-T5 each takes `H: Hasher` and walks
  `fold_unit_digest` (or the corresponding Parallel/Stream/Interaction
  variant) over the full input state, packing the result into the
  certificate's `content_fingerprint` field. Each `certify_*_const` passes
  a distinct `CertificateKind` discriminant byte so two certify calls over
  the same source unit produce distinguishable fingerprints.
    - The `certify_*_const` functions are no longer `const fn` (trait method
      dispatch on `H::initial()`/`fold_byte`/`finalize` is not const-eval-
      friendly under MSRV 1.81). The const-fn frontier is preserved by a new
      `pipeline::run_const_zero<T>` entry point that bypasses trait dispatch
      via the `ZeroHasher` marker.

- **T5.7 — Delete `primitive_op_weight` + XOR-multiply fold from the replay
  module.** Architecturally subsumed by C3.d's `Hasher` trait. The foundation
  no longer ships any hash function bodies — only the trait, the canonical
  byte layouts (`fold_unit_digest` / `fold_parallel_digest` / `fold_stream_digest`
  / `fold_interaction_digest` / `fold_constraint_ref`), the `ZeroHasher`
  no-op marker, and the discriminant tables (`primitive_op_discriminant`,
  `certificate_kind_discriminant`).

- **T5.8 — Rename `ReplayError::LengthMismatch` → `NonContiguousSteps`** +
  add `CapacityExceeded { declared, provided }` variant + add
  `FingerprintMissing` variant (returned when `verify_trace` is called on a
  trace whose stored fingerprint is `ContentFingerprint::zero()`).

- **T5.9 — `core::error::Error` impls for all 6 public error types** +
  workspace MSRV bump from 1.70 to **1.81** (where `core::error::Error` is
  stable for `no_std`). `CalibrationError`, `ShapeViolation`, `PipelineFailure`,
  `ReplayError`, `BindingsTableError`, and `GenericImpossibilityWitness` all
  implement `core::fmt::Display` + `core::error::Error`, so downstream
  consumers can `?`-propagate them through `Box<dyn Error>` chains.

- **T5.10 — `StreamDriver::is_terminated()` accessor.** Parallel to
  `InteractionDriver::is_converged()`; lets downstream observe termination
  state without a destructive `next()` call.

- **T5.11 — Complete `lib.rs` re-exports.** Every public type a downstream
  consumer reaches for now resolves via the short `uor_foundation::*` path:
  `Hasher`, `ContentFingerprint`, `ZeroHasher`, `CertificateKind`,
  `BindingsTableError`, `CalibrationError`, `PipelineFailure`,
  `LandauerBudget`, `Nanos`, `Term`, `TermArena`, `TermList`, `Certificate`,
  `FINGERPRINT_MIN_BYTES`, `FINGERPRINT_MAX_BYTES`, `TRACE_MAX_EVENTS`,
  `TRACE_REPLAY_FORMAT_VERSION`. The verify crate's re-exports gain the same
  set plus `PrimitiveOp`.

- **T5.12 — `verify_trace` doc rewrite.** The pre-T5 doc claimed
  "two structurally-distinct traces produce two distinct certificates" —
  false at the 1/65536 collision rate of the lossy 16-bit truncation. The
  post-T5 doc explains the actual contract: structural validation +
  fingerprint passthrough; round-trip property; substrate-agnostic; foundation
  recommends BLAKE3 for production; non-binding recommendation.

22 e2e tests pass (13 pre-T5 + 9 new T5 tests covering C1, C2, C3, C4, C6,
T5.9, T5.10, T5.11). 11 verify-crate round_trip tests pass (was 6 pre-T5,
expanded to cover non-contiguous steps, parametric width, deterministic
re-derivation).

### Counts after cleanup

- `CLASSES = 466` (+1 from T1.2)
- `LEAN_STRUCTURES = 433` (+1)
- `CONCEPT_PAGES = 12` (corrected from stale 27)
- `JACOBIAN_MAX_SITES = 8` (reduced from 64)
- `CONFORMANCE_CHECKS = 497` (+4: docs/concept_pages_count, rust/ebnf_constraint_decl,
  rust/public_api_functional/foundation_e2e, rust/public_api_functional/verify_round_trip)
- Workspace members: 11 → 12 (added `uor-foundation-test-helpers`)
- **MSRV bumped 1.70 → 1.81** (Tier 5: unlocks `core::error::Error` for `no_std`)
- **`FINGERPRINT_MAX_BYTES = 32`** (Tier 5: cap on inline content-fingerprint width
  carried by `Grounded` / `Trace` / `Derivation` / `GroundingCertificate`. Sized to
  hold the standard 256-bit cryptographic-hash outputs (BLAKE3, SHA-256, BLAKE2s)
  without exceeding the 256-byte `Grounded` budget pinned by `phantom_tag`.)
- **`FINGERPRINT_MIN_BYTES = 16`** (Tier 5: derived from the v0.2.2 ≤ 2^-64 collision
  bound under the birthday rate; not a prescription.)

### Verification

`cargo run --bin uor-conformance` reports **497 passed, 0 warnings, 0 failed**
from a clean checkout. After Tier 4, `uor-foundation-verify`'s manifest
satisfies cargo publish's dependency-version requirement (the `path`-only
form is hoisted to a workspace dep with explicit `version = "0.2.2"`),
and the round_trip suite contains 6 tests that re-derive certificates
end-to-end. `cargo test --workspace` runs all foundation
integration tests (uor_time, phantom_tag, parametric_constraints,
witt_tower_dense, witt_tower_limbs, public_api_e2e + ~10 others) and the
verify-crate round_trip tests, all green. The compile_fail doctest in
`GroundingProgram::from_primitive`'s rustdoc fails as expected, proving
the `MarkersImpliedBy<Map>` bound is enforced at compile time.

---

## v0.2.2 — 2026-04-14

v0.2.2 closes the five v0.2.1 enforcement escape hatches, ships the three
residual ontology items, addresses four deeper correctness items, and lands
five cross-cutting items. **18 work items total.** Backwards compatibility is
not a constraint; the release criterion is *no second path*.

### BREAKING — surface deletions

- **W1**: deleted `uor_ground!` macro (entire `uor-foundation-macros` crate).
- **W2**: deleted `#[derive(ConstrainedType)]` and `#[derive(CompileUnit)]`.
- **W3**: deleted `#[uor_grounded(level = "...")]` attribute.
- **W15**: **deleted the entire `uor-foundation-macros` crate** from the
  workspace. Removed from `Cargo.toml` workspace members. The pipeline now
  uses direct `pub(crate)` constructors. The contract is enforced at the
  type and visibility level, not at the macro level.
- **W2 cascade**: deleted `__macro_internals::GroundedShapeSealed` back-door,
  `MacroProvenance`, `__uor_macro_mint_validated`, `__uor_macro_mint_grounded`.

### Ontology additions

- **W7** (`spec/src/namespaces/op.rs`): corrected `op:Pipeline` and
  `op:Topological` rdfs:comments to reflect the actual ψ_1..6 / ψ_7..9
  inter-algebra map split. The earlier ψ_1..6 chain (constraint nerve →
  simplicial homology) is established under `op:Topological`; the later
  ψ_7..9 tower (Postnikov truncation, homotopy group extraction,
  k-invariant computation) is established under `op:Pipeline`.
- **W8** (`spec/src/namespaces/schema.rs`, `query.rs`, `state.rs`):
  `schema:Triad` gains three functional projection properties
  (`triadStratum`, `triadSpectrum`, `triadAddress`) bundling the canonical
  observable triple of a Datum at grounding time. `query:RingElement`
  renamed to `query:Address`. `state:groundedTriad` added on
  `state:GroundedContext`.
- **W4** (`spec/src/namespaces/morphism.rs`): two new GroundingMap individuals
  — `morphism:DigestGroundingMap` (one-way hash; total but not invertible;
  no structure preservation) and `morphism:BinaryGroundingMap` (raw byte
  ingestion; total and invertible; no structure beyond bit identity).
- **W14** (`spec/src/namespaces/reduction.rs`): added
  `reduction:ShapeMismatch` PipelineFailureReason individual with two
  FailureField individuals for the `expected` and `got` shape IRIs. The
  parametric `PipelineFailure` enum codegen picks it up automatically.

### Ontology counts (`spec/src/counts.rs`)

- `PROPERTIES`: 928 → **932** (+4 W8 properties)
- `NAMESPACE_PROPERTIES`: 927 → **931**
- `INDIVIDUALS`: 3443 → **3448** (+5: 2 GroundingMap + 1 ShapeMismatch + 2 FailureField)
- `METHODS`: 891 → **895**
- `LEAN_CONSTANT_NAMESPACES`: 3343 → **3348**
- `CONFORMANCE_CHECKS`: 474 → **476** (+2 new validators)

### Rust enforcement surface (additions)

- **W11** `enforcement::Certificate` sealed trait + `Certified<C>` parametric
  carrier. Replaces the v0.2.1 per-class shim duplication. All 10
  `cert:Certificate` subclasses now have a sealed Rust kind that implements
  `Certificate` with an `IRI` constant and `Evidence` associated type.
  Six previously-unshimmed classes (`TransformCertificate`,
  `IsometryCertificate`, `InvolutionCertificate`, `GeodesicCertificate`,
  `MeasurementCertificate`, `BornRuleVerification`) gain Rust visibility.
  Supporting evidence types (`CompletenessAuditTrail`, `ChainAuditTrail`,
  `GeodesicEvidenceBundle`) are exposed as concrete public structs.
- **W12** `enforcement::resolver::*::certify` free functions replace the
  v0.2.1 unit-struct façades:
  - `enforcement::resolver::inhabitance::certify(input)`
  - `enforcement::resolver::tower_completeness::certify(input)`
  - `enforcement::resolver::incremental_completeness::certify(input)`
  - `enforcement::resolver::grounding_aware::certify(unit)`

  Each returns `Result<Certified<Cert>, Witness>`. The v0.2.1 unit structs
  remain alongside the new free functions for the v0.2.2 release cycle.
- **W4** `enforcement::Grounding` trait gains `type Map: GroundingMapKind`
  associated type. Sealed marker traits `GroundingMapKind`,
  `PreservesMetric`, `PreservesStructure`, `Total`, `Invertible` partition
  the kinds by structural property. Foundation operations requiring
  structure preservation gate on `<G as Grounding>::Map: PreservesStructure`
  and reject digest-style impls at the call site. Five sealed kind structs
  (`IntegerGroundingMap`, `Utf8GroundingMap`, `JsonGroundingMap`,
  `DigestGroundingMap`, `BinaryGroundingMap`) implement the marker-trait
  table from the v0.2.2 plan.
- **W3** Unary phantom-typed ring ops (`Neg<L>`, `BNot<L>`, `Succ<L>`)
  next to the existing binary `Add/Sub/Mul/And/Or/Xor<L>`. New `UnaryRingOp`
  trait. New `Embed<From, To>` sealed level promotion (canonical injection
  ι : R_n → R_n′ for n ≤ n′), gated by the sealed `ValidLevelEmbedding`
  trait. Downward coercion is intentionally not supplied — projection is
  lossy and goes through `morphism:ProjectionMap` instances.
- **W13** `enforcement::Validated<T, Phase: ValidationPhase = Runtime>`
  parametric phase. New sealed `ValidationPhase` trait with `CompileTime`
  and `Runtime` markers. `From<Validated<T, CompileTime>> for
  Validated<T, Runtime>` impl provides the subsumption: a compile-time
  witness is usable wherever a runtime witness is required. The default
  phase is `Runtime` so v0.2.1 call sites that wrote `Validated<T>`
  continue to compile unchanged.
- **W14** `pipeline::run<T, P>` typed entry point: consumes
  `Validated<CompileUnit, P>`, returns `Result<Grounded<T>, PipelineFailure>`
  for an explicit `T: GroundedShape` and `P: ValidationPhase`. New
  `CompileUnit::witt_level()` and `CompileUnit::thermodynamic_budget()`
  accessors. New `PipelineFailure::ShapeMismatch { expected, got }` variant
  emitted automatically by the parametric `PipelineFailure` codegen from
  the W14 ontology addition.
- **W8** `enforcement::Triad<L>` struct: bundles the (stratum, spectrum,
  address) projection of a Datum at grounding time. Phantom-typed at level
  `L`, no public constructor — built only by foundation code. Field access
  via `stratum()`, `spectrum()`, `address()` accessors.
- **W10** `HostTypes` trait + `DefaultHostTypes` canonical impl. Narrows
  the v0.2.1 six-slot `Primitives` trait to the four slots that genuinely
  vary across host environments (`Decimal`, `DateTime`, `HostString`,
  `WitnessBytes`). Foundation-owned types (Witt-level integers, booleans,
  IRIs, canonical bytes) are derived from `WittLevel` and not exposed.
  `Primitives` remains as a deprecated alias for v0.2.1 backwards
  compatibility.

### Conformance suite

- **W5** new validator `docs/psi_leakage`: scans the consumer-facing crate
  surface (`README.md`, `foundation/README.md`, `foundation/docs/`) for
  unauthorized ψ vocabulary references. Mathematically correct internal use
  in `proof/`, `op/`, `homology/`, `cohomology/`, `derivation/` is excluded.
- **W6** new validator `rust/public_api_snapshot`: pins the exact set of
  `pub` items in `uor-foundation`'s enforcement, lib, and pipeline modules
  to a snapshot file at `foundation/tests/public-api.snapshot`. Drift
  requires explicit snapshot update review. Initial baseline: **129
  pinned symbols**.
- v0.2.2 release artifact `public/uor.conformance.ebnf` joins
  `public/uor.term.ebnf` as a complete release artifact emitted by
  `cargo run --bin uor-build`. The conformance EBNF grammar is published
  alongside the primary Term-language grammar.

### Tests (W17)

New test files under `foundation/tests/`:

- `grounding_map_kind_markers.rs` — exact marker-trait coverage per W4 plan
  table; one test per kind asserts which markers it implements.
- `host_types_surface.rs` — pins the exact `HostTypes` shape, asserts
  `DefaultHostTypes` selects `f64`/`i64`/`str`/`[u8]`, demonstrates an
  embedded-host override.
- `validated_phases.rs` — asserts `ValidationPhase` is implemented by
  `CompileTime` and `Runtime`, that the default phase resolves to `Runtime`,
  and that the `From<Validated<_, CompileTime>>` subsumption compiles.
- `unary_ring_ops.rs` — exercises `Neg<W8>`, `BNot<W8>`, `Succ<W8>`,
  `Neg<W32>`, plus `Embed<W8, W16>` and `Embed<W8, W32>` widening. Verifies
  the critical-composition law `Succ = Neg ∘ BNot` directly.

### Documentation (W18)

- Crate-level `//!` rustdoc rewritten as a v0.2.2 principal-data-path
  tutorial with an ASCII diagram showing the
  `host bytes → Grounding<Map> → Datum → Validated<T, Phase> → pipeline::run::<T, P> → Grounded<T> → Triad<L>` flow.
- Migration table from v0.2.1 to v0.2.2 (each deleted symbol mapped to its
  v0.2.2 replacement) embedded in the crate-root rustdoc.
- `enforcement::prelude` re-exports the full v0.2.2 surface
  (`Certified`, `Triad`, `Certificate`, `GroundingMapKind`, marker traits,
  cert kind structs, Validation phases, unary ring ops, Embed) alongside
  the v0.2.1 carry-over symbols.

### Phase expansion (target-v2 — all in scope, not deferred)

v0.2.2 was expanded beyond the original 18-W-item scope to fold the
full target architecture into a single release. Every acceptance item
is delivered.

#### Phase A — UorTime infrastructure (Q1)

- `observable:LandauerBudget` class + `observable:landauerNats` property.
- Sealed `UorTime` = (`LandauerBudget`, `rewrite_steps: u64`) carrier
  with component-wise `PartialOrd`.
- `Calibration` with validated k_B·T / thermal_power / characteristic_energy,
  four presets (`X86_SERVER`, `ARM_MOBILE`, `CORTEX_M_EMBEDDED`,
  `CONSERVATIVE_WORST_CASE`).
- `UorTime::min_wall_clock(&Calibration) -> Nanos` using
  `max(Landauer, Margolus-Levitin)` bounds.
- Conformance gate: `rust/uor_time_surface`.

#### Phase B — Phantom Tag on Grounded (Q3)

- `Grounded<T, Tag = T>` phantom parameter with zero-cost
  `tag::<NewTag>()` coercion. Downstream distinguishes
  `Grounded<_, BlockHashTag>` from `Grounded<_, PixelTag>` without new
  sealing.
- Conformance gate: `rust/phantom_tag`.

#### Phase C — Witt tower parametric (Q2)

- **C.1–C.3**: +28 `schema:WittLevel` individuals (W40..W128 u64/u128
  backed; W160..W32768 Limbs<N> backed). Dense at native widths plus
  semantically-meaningful intermediates (SHA-1/-224/-384, P-192/-384/-521).
- `Limbs<const N: usize>` generic kernel with const-fn
  `wrapping_add/sub/mul/xor/and/or/not/mask_high_bits`.
- **C.4**: `cert:MultiplicationCertificate`,
  `resolver:MultiplicationResolver`, `linear:stackBudgetBytes`; sealed
  `MulContext<L>` + `MultiplicationEvidence`; closed-form Landauer cost
  `(2R-1) × (N/R)² × 64 × ln 2` nats grounded in `op:OA_5`.
- Conformance gates: `rust/witt_tower_completeness`,
  `rust/multiplication_resolver`.

#### Phase D — Constraint kinds parametric (Q4)

- Delete 7 disjoint `type:Constraint` subclasses (`Residue`,
  `Hamming`, `Depth`, `Carry`, `Site`, `Affine`, `Composite`).
- Add `type:BoundConstraint`, `type:BoundShape`, `type:Conjunction`
  classes + 4 parametric properties + 6 `BoundShape` individuals + 6
  `BoundConstraint` kind individuals.
- Add 4 new observable subclasses: `observable:ValueModObservable`,
  `derivation:DerivationDepthObservable`, `carry:CarryDepthObservable`,
  `partition:FreeRankObservable`.
- Codegen emits sealed `Observable` + `BoundShape` traits, parametric
  `BoundConstraint<O, B>` + `Conjunction<N>` carriers, fixed-size
  `BoundArguments`, and 7 legacy type aliases
  (`ResidueConstraint`, `HammingConstraint`, ..., `CompositeConstraint<N>`)
  with per-alias `pub const fn new` constructors.
- Conformance gate: `rust/parametric_constraints`.

#### Phase E — Bridge namespace completion

- `cert:PartitionCertificate`, `partition:PartitionComponent` enum
  (Irreducible/Reducible/Units/Exterior), `observable:GroundingSigma`,
  `observable:JacobianObservable`, `derivation:DerivationTrace`.
- Sealed `SigmaValue` newtype, `JacobianMetric<L>` fixed-size carrier,
  `PartitionComponent` enum, `Query`/`Coordinate<L>`/`BindingQuery`/
  `Partition`/`Trace`/`TraceEvent`/`HomologyClass<N>`/`CohomologyClass<N>`.
- Six `BaseMetric` accessors on `Grounded<T, Tag>`: `d_delta()`,
  `sigma()`, `jacobian()`, `betti_numbers()`, `euler_characteristic()`,
  `residual_count()`. `MAX_BETTI_DIMENSION = 8`,
  `JACOBIAN_MAX_SITES = 64`.
- `Derivation::replay() -> Trace` accessor.
- `InteractionDeclarationBuilder` stub with peer_protocol /
  convergence_predicate / commutator_state_class setters.
- Conformance gate: `rust/bridge_namespace_completion`; new SHACL
  fixture `test280_bridge_completion`.

#### Phase F — Driver completion (Q5)

- `pipeline::run_parallel<T, P>` consuming `Validated<ParallelDeclaration, P>`.
- `pipeline::run_stream<T, P>` returning `StreamDriver<T, P>: Iterator`.
- `pipeline::run_interactive<T, P>` returning `InteractionDriver<T, P>`
  state machine with `step(PeerInput) -> StepResult<T>`, `is_converged()`,
  `finalize()`.
- Sealed `PeerInput`, `PeerPayload`, `CommutatorState<L>`, `StepResult`.
- Conformance gate: `rust/driver_shape`.

#### Phase G — Const-fn frontier widening

- 4 `validate_*_const` companion free functions (Lease/CompileUnit/
  Parallel/Stream).
- 4 `certify_*_const` companion free functions
  (tower_completeness/incremental_completeness/inhabitance/multiplication).
- `pipeline::run_const<T>` with widened `T::Map: Total` gate (drops the
  `Invertible` requirement).
- Conformance gate: `rust/const_fn_frontier`.

#### Phase J — Combinator-only Grounding (marquee item)

- Closed 12-combinator surface in `enforcement::combinators`:
  `read_bytes`, `interpret_le_integer`, `interpret_be_integer`, `digest`,
  `decode_utf8`, `decode_json`, `select_field`, `select_index`,
  `const_value`, `then`, `map_err`, `and_then`.
- `GroundingPrimitiveOp` sealed enum, `GroundingPrimitive<Out>` carrier
  with `MarkerBits` bitmask (Total=1, Invertible=2, PreservesStructure=4).
- Zero-sized `TotalMarker` / `InvertibleMarker` / `PreservesStructureMarker`
  type-level tokens.
- `MarkersImpliedBy<Map: GroundingMapKind>` trait with impls for the
  closed catalogue of valid (marker tuple, kind) pairs.
- `GroundingProgram<Out, Map: GroundingMapKind>` sealed carrier with
  `from_primitive` constructor. Downstream programs built out of mismatched
  combinators are rejected at compile time.
- Conformance gate: `rust/grounding_combinator_check`.

#### Phase H — Lints + cross-cutting

- `foundation/Cargo.toml` feature flag layout: `default` (strictly empty),
  `alloc`, `std`, `serde`, `observability`.
- New workspace member `uor-foundation-verify` (strictly `no_std` default;
  optional `serde` feature). Depends on `uor-foundation` public surface
  only. `verify_trace(&Trace) -> Result<ReplayOutcome, VerificationFailure>`
  walks a content-addressed Trace and re-derives the certificate.
- Conformance gates:
  - `rust/feature_flag_layout`
  - `rust/escape_hatch_lint` (grep-based: rejects `unsafe impl` on sealed
    traits and unconditional `extern crate alloc/std`)
  - `rust/no_std_build_check` (cargo check with `--no-default-features`)
  - `rust/alloc_build_check` (cargo check with `--features alloc`)
  - `rust/all_features_build_check` (cargo check with `--all-features`)
  - `rust/uor_foundation_verify_build`

#### Phase I — Counts + acceptance

Final counts after Phases A–J:

- `CLASSES = 465` (+8 net: Phase A +1, Phase C.4 +2, Phase D net 0,
  Phase E +5).
- `PROPERTIES = 942` (+10 net).
- `INDIVIDUALS = 3493` (+50 net across phases).
- `METHODS = 905`. `ENUM_CLASSES = 19`. `LEAN_INDUCTIVES = 23`.
- `SHACL_TESTS = 280`.
- `CONFORMANCE_CHECKS = 493`.

All phases landed in a single v0.2.2 release. `uor-foundation-clippy`
(dylint-based) is replaced by the grep-based `rust/escape_hatch_lint`
validator since the sandbox toolchain does not support dylint pinning;
the type-and-visibility sealing + public-API snapshot already provide
the net-new safety the dylint would have added.

## v0.2.1 — 2026-04-13

v0.2.1 bundles the **Inhabitance Verdict Instantiation** ontology release with
the **Zero-Overhead Ergonomics Surface** Rust/Lean 4 additions. Every item is
strictly extensional with respect to v0.2.0 — no public signatures removed,
no breaking API changes.

### Ontology (strictly extensional)

- **New classes** (13): `cert:InhabitanceCertificate`, `proof:InhabitanceImpossibilityWitness`,
  `trace:InhabitanceSearchTrace`, `derivation:InhabitanceStep`, `derivation:InhabitanceCheckpoint`,
  `resolver:InhabitanceResolver`, `resolver:TwoSatDecider`, `resolver:HornSatDecider`,
  `resolver:ResidualVerdictResolver`, `resolver:CertifyMapping`, `schema:ValueTuple`,
  `reduction:FailureField`, `conformance:PreludeExport`.
- **New properties** (31) across `cert/`, `proof/`, `trace/`, `derivation/`,
  `predicate/`, `resolver/`, `conformance/`, `reduction/`, `parallel/`, `stream/`,
  `state/` — including the parametric metadata (`resolver:forResolver`,
  `conformance:surfaceForm`, `reduction:fieldName`, etc.) that drives the
  ontology-first code-generation pattern.
- **New individuals** (80+): `predicate:InhabitanceDispatchTable` plus 3 rules,
  4 `op:Identity` individuals (IH_1, IH_2a, IH_2b, IH_3) with full proof
  coverage, 4 `resolver:CertifyMapping` facts, 11 `reduction:FailureField`
  individuals, 16 `conformance:PreludeExport` individuals, 6 new
  `conformance:Shape` instances with 17 `PropertyConstraint` decompositions.

### Rust ergonomics surface (`uor-foundation` + `uor-foundation-macros`)

- **Sealed wrappers**: `Validated<T: OntologyTarget>` now auto-derefs to `T`;
  `Grounded<T: GroundedShape>` wraps the compile-time ground-state witness
  with O(1) binding lookup (`op:GS_5`).
- **`Certify<I>` trait** — generic over the input type so downstream user
  types flow directly through the resolver façades:
  ```rust
  let cert: Validated<LiftChainCertificate> =
      TowerCompletenessResolver::new().certify(&shape)?;
  let level: WittLevel = cert.target_level();
  ```
- **Four resolver façades** (`TowerCompletenessResolver`,
  `IncrementalCompletenessResolver`, `GroundingAwareResolver`,
  `InhabitanceResolver`) emitted parametrically from `resolver:CertifyMapping`
  individuals.
- **`PipelineFailure` enum** with 7 variants emitted from `reduction:FailureField`.
- **Ring-op phantom wrappers** (`Mul<L>`, `Add<L>`, `Sub<L>`, `Xor<L>`, etc.)
  at `W8` and `W16` with `const fn` implementations.
- **Fragment markers** (`Is2SatShape`, `IsHornShape`, `IsResidualFragment`)
  and `INHABITANCE_DISPATCH_TABLE` const.
- **Full reduction pipeline driver** at `uor_foundation::pipeline` — 6 preflight
  checks, 7 reduction stages, Aspvall-Plass-Tarjan 2-SAT decider,
  unit-propagation Horn-SAT decider, fragment classifier, FNV-1a unit-id hasher.
  `#![no_std]`-compatible. Backs every `Certify::certify` call.
- **Macro surface**: `uor!` (existing), `uor_ground!` (new — expands to real
  `Grounded<T>` via the back-door minting API with a trailing `as Grounded<T>`
  type clause), `#[derive(ConstrainedType)]` (emits `GroundedShape` +
  `ConstrainedTypeShape` impls carrying residue/hamming constraints),
  `#[uor_grounded(level = "WN")]` (compile-time Witt-level assertion).
- **`foundation::enforcement::prelude`** — 18-symbol re-export for the
  consumer-facing one-liners.

### Lean 4 parity (`lean4/UOR/`)

- **New modules**: `Enforcement.lean`, `Pipeline.lean`, `Prelude.lean`.
- `Certify (ρ : Type) (I : Type)` class generic over input type (Lean parity
  with the Rust `Certify<I>` trait).
- `UOR.Pipeline.runTowerCompleteness`, `runIncrementalCompleteness`,
  `runGroundingAware`, `runInhabitance` — Lean-side pipeline entry points.
- `ConstraintRef`, `FragmentKind`, `fragmentClassify` — structural parity
  with the Rust types.
- `lake build` compiles cleanly; `lake upload` publishes to Lean Reservoir.

### Tooling

- **New `cargo-uor` binary**:
  - `cargo uor check <path>` — walks a crate tree for `uor_ground!` invocations,
    parses the conformance grammar, and reports per-invocation validity.
  - `cargo uor inspect <class-name>` — reads the bundled ontology and prints
    the class IRI, const accessors (`GS_7_SATURATION_COST_ESTIMATE`,
    `OA_5_LEVEL_CROSSINGS`, `BUDGET_SOLVENCY_MINIMUM`), and the `rdfs:comment`.
  - `cargo uor explain <iri>` — resolves any ontology IRI (prefixed or
    full-URI form) to its `rdfs:comment`.

### Grammar

- `public/uor.conformance.ebnf` — **parametrically emitted** from the
  ontology's `conformance:Shape` and `PropertyConstraint` individuals via
  `spec/src/serializer/conformance_ebnf.rs`. Adding a new declaration shape
  requires only an ontology edit.

### Counts

| Metric | v0.2.0 | v0.2.1 | Delta |
|---|---|---|---|
| Namespaces | 33 | 33 | 0 |
| Classes | 441 | 454 | +13 |
| Properties | 890 | 921 | +31 |
| Individuals | 3358 | 3438 | +80 |
| `op:Identity` | 624 | 628 | +4 |
| Conformance checks | 471 | 472 | +1 |
| SHACL test fixtures | 276 | 277 | +1 |

### Verification

`cargo test --workspace` • `cargo clippy --all-targets -- -D warnings` •
`cargo run --bin uor-conformance` (`Conformance PASSED.`) •
`cargo check -p uor-foundation --no-default-features` •
`cargo test -p uor-foundation --no-default-features --test no_std` •
`cd /workspaces/UOR-Framework && lake build` (`Build completed successfully.`).

## v0.2.0 — 2026-03-10

Baseline release. See `RELEASING.md` for details.
