# Orphan-Trait Closure: Completion Plan

## Purpose

Phases 0–6 shipped a working scaffold: 188 Null stubs closed, Phase-0
classification table committed, capacity-guard fail-fast landed, and
Path-4 theory-deferred register wired to CI. Along the way several
pieces of the original plan were narrowed or deferred under the working
justification "make it land". That was the right tactical call for the
first pass; it is the wrong *steady* state. This plan identifies every
narrowing and every deferral, and commits each to a completion phase
with the same correctness-over-backwards-compatibility discipline as
Phases 0–6.

## Non-deferment policy (binding)

**Weakening, narrowing, or deferring any feature listed in this plan
is not authorized.** If a phase encounters blockers:

1. The phase **grows** — additional sub-tasks, tests, or prerequisite
   corrections are added to the same phase until the original scope
   lands.
2. A new phase may be **inserted** between existing phases if the
   prerequisite turns out to be large, but the downstream phase
   keeps its full scope.
3. The only acceptable deferral remains Phase 14 (theory-deferred
   classes awaiting research grounding). Every phase before Phase 14
   ships its full scope before Phase 14 is declared closed.

Language like "out of scope", "deferred to a future phase", "initial
closure", "partial closure", "no-op at this closure", or "scope
decision" is **not used** in any phase's close condition. A phase
either lands full scope or remains open.

**Explicit counter-example.** Phase 2's Null-stub substitution for
the Resolved-wrapper design (shipped in Phase 2's first pass) is
precisely the pattern this policy forbids going forward. Phase 8
corrects it by landing the full Resolved-wrapper design on top of
the Null-stub foundation. No substitution of that kind is permitted
in Phases 7–13.

**Enforcement.** Every phase's close condition is a test or
conformance check that either passes or fails — no human judgement
call about "good enough". If the check fails, the phase stays open;
if it passes, the phase is done. The gate is mechanical.

## Audit — what's outstanding

### A. Phase 1c (HostTypes::Decimal arithmetic bounds) — fully deferred
- `HostTypes::Decimal` has **zero bounds** today. Primitives that
  produce observable values return `f64` directly.
- **104** occurrences of `f64` in non-test code across
  `enforcement.rs`, `bridge/trace.rs`, `user/state.rs`,
  `kernel/reduction.rs`, `pipeline.rs`.
- No `no_hardcoded_f64` conformance gate exists.
- No `HostTypesDiscipline` conformance category exists (R16).

### B. Phase 2 design substitution — Resolved-wrapper replaced with Null-stub
- The plan's resolved design (`{Foo}Handle<H>` + `{Foo}Resolver<H>` +
  `{Foo}Record<H>` + `Resolved{Foo}<'r, R, H>`) ships as the "strong
  contracts" path. None of it was emitted.
- What shipped is a PhantomData-only `Null{Foo}<H>` that satisfies the
  trait with absent-sentinel returns. This closes orphans but gives
  hosts no content-addressed-resolution surface.
- `PartitionHandle` / `PartitionResolver` / `PartitionRecord` exist
  only for the 4 partition-algebra classes from the amendment.

### C. Phase 2 cascade drops — 240 orphans still open
- **237** Path-1 classes and **3** Path-2 classes cascade out of the
  emitable set via reference chains to enum accessors or Path-4
  classes. These traits remain orphan.
- Concrete subgap list:
  - **C1 Enum defaults.** `#[derive(Default)]` on every generated
    enum with `#[default]` on the first variant — reverted in Phase 4
    because `WittLevel` is a struct, not an enum.
  - **C2 WittLevel.** The only struct in `enum_class_names()`. Needs
    either a `Default` impl or a special-case stub emission.
  - **C3 Inherited associated types.** When `impl C for Null{X}` and
    `impl P for Null{X}` both declare `type Y`, Rust rejects the
    duplicate. Needs `collect_inherited_assoc_types` wiring into Null
    impls.
  - **C4 Cross-namespace enum imports.** Null stubs that reference
    enums from another namespace need per-stub import generation.
  - **C5 Path-4 reference satisfaction.** Classes with Path-4
    references (cohomology, monoidal, operad, parallel, stream)
    currently cascade. Need to decide: emit Null stubs for Path-4
    too (with tracked-deferred banners), or hand-roll marker-only
    stubs to satisfy type bounds.

### D. Phase 3 VerifiedMint scaffolds — fully deferred
- No `{Foo}Witness` / `{Foo}MintInputs<H>` / `impl VerifiedMint`
  emission for Path-2 classes beyond the 4 amendment witnesses.
- R6 (`THEOREM_IDENTITY` via `op:provesFor` inverse lookup) not
  implemented. R7 (`entropy_bearing` detection) not wired into
  emission. R5 (`MintInputs` field mapping) not implemented.

### E. Phase 4 Path-3 blanket impls — entirely empty
- `PATH3_ALLOW_LIST` is empty. No classes classify `Path3PrimitiveBacked`.
- `foundation/src/blanket_impls.rs` doesn't exist. R8
  (`@codegen-exempt` preservation) not implemented. R9 (lib.rs
  module registration) not needed yet because (c) doesn't exist.
- R13 (Path-3 loud failure on missing primitive) not active.

### F. Phase 5 per-theorem primitive bodies — no-op
- `foundation/src/primitives/` directory doesn't exist.
- No new per-theorem verification bodies beyond the amendment's
  PT_1/PT_3/PT_4, ST_1/ST_2/ST_6–ST_10, CPT_1/CPT_3–CPT_5.
- R10 (primitive-file lifecycle), R15 (theorem-family worklist
  ordering) both dormant.

### G. Cross-cutting infrastructure — partial
- **R1 orphan count** is not implemented as a conformance check. The
  current count (188 stubs) is a grep result from tests; no
  `rust/orphan_counts` validator exists.
- **R11 load_doc_fragment** (rustdoc-from-markdown extraction) not
  implemented. Phase-2/3 emissions embed hand-written rustdoc
  comments rather than extracting from phase docs.
- **R16 conformance categories**: only `TheoryDeferredRegister`
  landed. `TaxonomyCoverage`, `OrphanCounts`, `HostTypesDiscipline`
  are not registered.

### Summary numbers
- **260** traits still orphan (240 cascade + 20 theory-deferred).
- **104** hardcoded `f64` sites.
- **6** concrete sub-tracks deferred (A, B, C, D, E, F).

## Guiding principle (unchanged from the original plan)

**Correctness over backwards compatibility. No further deferment.**

Every deferral flagged above is assumed to ship unless this plan
states an explicit no-op with theory justification. Sub-phase scope
changes during execution are allowed only to add *more* work, never
less. Completion criteria: every sub-track has a committed phase and
a green conformance run.

## Phases

### Phase 7 — Cascade unblockers (closes ~200 of the 240 open orphans)

Five sub-tasks, shipped as one Phase 7 commit sequence (7a–7e). Each
sub-task has a red test before implementation; the phase closes when
the cumulative emitable set covers all classes except genuinely
Path-4 theory-deferred.

**7a — Enum `Default`.** Two codegen changes in
[codegen/src/enums.rs](codegen/src/enums.rs):

1. **Enum branch.** Inside the `for e in &enums { ... }` emission
   loop, add `Default` to the derive list and emit `#[default]` on
   the first variant emitted. Variant ordering is determined by the
   ontology individual order, so the first variant is
   spec-stable — re-verified by the Phase-0 classification report
   regen on every `uor-crate` run.

2. **WittLevel special case.** `WittLevel` is a hand-emitted struct
   (not enum) defined near the bottom of
   [codegen/src/enums.rs](codegen/src/enums.rs). Append a hand-written
   impl after the `WittLevel::new` declaration:

   ```rust
   impl Default for WittLevel {
       /// `W8` is the spec-defined minimum Witt level and the
       /// canonical base referenced by `schema:WittLevel` individuals.
       fn default() -> Self { Self::W8 }
   }
   ```

   W8 is chosen because it is the smallest level in
   `schema:WittLevel` and every downstream needing a "zero Witt
   level" already uses W8. No other level has the same spec-stability.

Red test: [codegen/tests/enum_defaults.rs](codegen/tests/enum_defaults.rs)
instantiates `{Enum}::default()` for every name in
`Ontology::enum_class_names()` plus `WittLevel`, asserting each
returns the spec-canonical first-variant sentinel.

**7b — Inherited associated-type dedup.** Thread the existing
`collect_inherited_assoc_types(class, all_props_by_domain) ->
HashSet<String>` helper at
[codegen/src/traits.rs:39](codegen/src/traits.rs#L39) into the
Null-impl emission path. The helper already returns names declared
by transitive parents; it just isn't called from the Null-impl
emitter.

Concrete plumbing:

1. `emit_null_stub` computes `inherited = collect_inherited_assoc_types(class, all_props_by_domain)` once.
2. `emit_null_stub` passes `&inherited` to `emit_null_impl_for_trait`.
3. `emit_null_impl_for_trait` passes `&inherited` to `emit_null_method_body`.
4. `emit_null_method_body`'s object-accessor branch checks
   `inherited.contains(&assoc_name) || emitted_assoc.contains(&assoc_name)`
   and skips the `type {assoc_name} = ...` line if either is true.
   The method body itself (`fn m(&self) -> &Self::{assoc_name}`) is
   still emitted — only the `type =` declaration is deduplicated.

Red test:
[codegen/tests/inherited_assoc_dedup.rs](codegen/tests/inherited_assoc_dedup.rs)
— picks a class with a multi-level inheritance chain (e.g.,
`bridge::partition::IrreducibleSet` extends `Component`), generates
the Null stub output, and asserts each associated type name appears
exactly once across all supertrait impls combined.

**7c — Cross-namespace enum imports.** `generate_namespace_module`
currently collects `enum_imports` from the module's own properties
only (existing loop at
[codegen/src/traits.rs:118](codegen/src/traits.rs#L118)). After Phase
7a adds enum-default emission to Null impls, a second pass is
required to catch enums referenced via transitive supertrait
properties.

Concrete plumbing:

1. After the existing `enum_imports` collection loop, add a second
   pass walking `module.classes`.
2. For each class, call `transitive_supertraits(class, ontology)` and
   for each parent, walk `all_properties_by_domain.get(parent)`.
3. For every property of a parent that has enum range (via
   `datatype_enum_override` or `object_property_enum_override`),
   append the enum name to `enum_imports`.
4. The existing dedup via `enum_imports.contains(&name)` handles the
   merge; the subsequent `sort_unstable` keeps output deterministic.

Red test: [codegen/tests/cross_namespace_enum_imports.rs](codegen/tests/cross_namespace_enum_imports.rs)
— pick a namespace module whose Null stub references a
cross-namespace enum (e.g., `kernel/op.rs` with `MetricAxis` enum
from `user/type_/` namespace), generate the module output, assert
`use crate::enums::MetricAxis;` appears at the top.

**7d — Path-4 reference-satisfaction stubs.** Path-4 classes
(cohomology, monoidal, operad, parallel, stream) have no Null stub
today, so any Path-1/2 class referencing a Path-4 type cascades out
of the emitable set. Emit `Null{PathFour}<H>` for these classes with
the following specific conventions:

1. **Banner** (both attributes required, exact strings):

   ```rust
   #[doc(hidden)]
   #[doc = "THEORY-DEFERRED — not a valid implementation; see [docs/theory_deferred.md]. Exists only to satisfy downstream trait-bound references."]
   pub struct Null{PathFour}<H: HostTypes> { ... }
   ```

   `#[doc(hidden)]` hides the stub from rustdoc; the explicit `#[doc
   = "THEORY-DEFERRED…"]` is there for anyone grepping the source, so
   the banner is self-documenting even without rustdoc.

2. **Body** — same absent-sentinel impl as any other Null stub per
   R4. The stub is semantically a *type-system lie*: it satisfies the
   trait-bound requirement without claiming correctness. Phase 14
   replaces each with a real theory-grounded impl.

3. **Theory-deferred register preservation.** Phase 6's
   `rust/theory_deferred_register` check remains authoritative for
   research-question tracking in `docs/theory_deferred.md`. The
   Phase 7d stubs do NOT replace that register; both coexist.

4. **Validator augmentation.** The
   `rust/theory_deferred_register` validator gains one additional
   assertion: for every row in `docs/theory_deferred.md`, the
   generated source contains `pub struct Null{Class}<H: HostTypes>`
   preceded by the exact `#[doc(hidden)]` + THEORY-DEFERRED banner
   combination. Missing banner or missing stub = validator fail.

Red test:
[codegen/tests/path4_stub_banners.rs](codegen/tests/path4_stub_banners.rs)
— every Path-4 class has the emitted stub with the exact banner
combination.

**7e — Remove `enum accessor` cascade filter.** With (a)–(d) in
place, the `should_emit_null_stub` check excluding classes with any
enum accessor is no longer needed. Delete the filter and confirm the
emitable-set fixed point grows to cover all classes except those
explicitly classified `Skip`. Red-then-green test: `path1_null_emission`
ratchet climbs from 188 to the full Path-1 + Path-2 + Path-4 count
(~432).

**Phase 7 close condition.** The minimum-viable `rust/orphan_counts`
validator **moves from Phase 13 to Phase 7e** (it is required here
to assert closure mechanically). Phase 13 retains the advanced
classifier-integrated version; Phase 7e ships the bare grep:

1. Parse every `pub trait {Name}<H: HostTypes>` declaration across
   `foundation/src/**/*.rs`.
2. For each trait, search the workspace for regex
   `^\s*impl(<[^>]*>)?\s+(crate::)?([\w_]+::)*{Name}(<[^>]*>)?\s+for\s+`
   excluding lines inside `#[cfg(test)]` blocks (tracked by simple
   brace-depth counter).
3. Zero impl matches ⇒ orphan.
4. Pass ↔ orphan count ≤ 20 (the Path-4 theory-deferred count, each
   of which now has a Phase-7d stub with the exact banner combination
   — so the count is really "classes whose only impl is the
   `#[doc(hidden)]` theory-deferred stub"). Anything else fails.

### Phase 8 — Resolved-wrapper infrastructure (the original Phase 2 design)

Adds the content-addressed resolver surface on top of the Null
stubs. The Null stub remains the resolver-absent default; the
Resolved wrapper provides real-data access for hosts.

**8a — Emit `{Foo}Handle<H>`** per Path-1 class: `#[derive(Copy, Clone,
Debug, Eq, Hash, PartialEq)]` carrying `ContentFingerprint` +
`PhantomData<H>`.

**8b — Emit `{Foo}Resolver<H>` trait** per Path-1 class: single method
`resolve(handle) -> Option<{Foo}Record<H>>`. No default impl
(resolvers must be concrete — guiding principle rejects unit-type
resolvers).

**8c — Emit `{Foo}Record<H>` struct** per Path-1 class. Fields per R3:
handle-typed for object accessors, scalar for datatype accessors,
slice for non-functional. Cross-class references use `{Other}Handle<H>`.

**8d — Emit `Resolved{Foo}<'r, R: {Foo}Resolver<H>, H: HostTypes>`**
per Path-1 class with `impl {Foo}<H>` delegating every accessor
through `self.resolver.resolve(self.handle)`. Return-type table per
R4 for `None` returns (same absent sentinels the Null stub uses).

**8e — `collect_inherited_assoc_types` extends to the wrapper**
(same fix as 7b).

**Resolved-wrapper storage rule (concrete).** The plan's open
question — how `Resolved{Foo}<'r, R, H>` returns `&Self::Assoc`
without storing it — is resolved as follows:

```rust
pub struct Resolved{Foo}<'r, R: {Foo}Resolver<H>, H: HostTypes> {
    handle: {Foo}Handle<H>,
    resolver: &'r R,
    /// Resolved record cached at construction. `None` means the
    /// resolver returned None at construction time; accessors then
    /// return `&<Null{Range}<H>>::ABSENT` via the Phase-7 stubs.
    record: Option<{Foo}Record<H>>,
}

impl<'r, R: {Foo}Resolver<H>, H: HostTypes> Resolved{Foo}<'r, R, H> {
    pub fn new(handle: {Foo}Handle<H>, resolver: &'r R) -> Self {
        let record = resolver.resolve(handle);
        Self { handle, resolver, record }
    }
    pub const fn handle(&self) -> {Foo}Handle<H> { self.handle }
    pub const fn resolver(&self) -> &'r R { self.resolver }
}

impl<'r, R: {Foo}Resolver<H>, H: HostTypes> {Foo}<H>
    for Resolved{Foo}<'r, R, H>
{
    type {Assoc} = Null{RangeClass}<H>;

    // Object accessor returning `&Self::Assoc`. Record stores the
    // range class's handle, not a full sub-Resolved. Returns the
    // absent sentinel regardless — hosts needing the sub-Resolved
    // use `resolve_{m}` below.
    fn m(&self) -> &Self::{Assoc} {
        &<Null{RangeClass}<H>>::ABSENT
    }

    // Scalar accessor. Record is Some ⇒ return field by value.
    // Record is None ⇒ return the absent sentinel per R4.
    fn scalar(&self) -> H::Decimal {
        match &self.record {
            Some(r) => r.scalar,
            None => H::EMPTY_DECIMAL,
        }
    }

    // Non-functional accessor returning `&[T]`. Record Some ⇒
    // return the slice field; Record None ⇒ `&[]`.
    fn many(&self) -> &[Self::{Assoc}] {
        match &self.record {
            Some(r) => &r.many,
            None => &[],
        }
    }
}

impl<'r, R: {Foo}Resolver<H>, H: HostTypes> Resolved{Foo}<'r, R, H> {
    /// Chain-resolver: promote the handle stored in `record.m` into
    /// a `Resolved{RangeClass}` by supplying a resolver for the
    /// range class. Present regardless of which impl of the trait
    /// `m()` is returning — the chain-resolver is an *extra*
    /// method, outside the trait surface, that hosts call when
    /// they want to chain resolution.
    pub fn resolve_{m}<'r2, R2: {RangeClass}Resolver<H>>(
        &self,
        r: &'r2 R2,
    ) -> Option<Resolved{RangeClass}<'r2, R2, H>> {
        let record = self.record.as_ref()?;
        Some(Resolved{RangeClass}::new(record.m_handle, r))
    }
}
```

The associated type `type {Assoc} = Null{RangeClass}<H>` aligns
with Phase 7's Null stubs — **Phase 8 therefore depends on Phase 7
having emitted `Null{RangeClass}<H>` first**, because
`Resolved{Foo}` references `Null{RangeClass}::ABSENT`. The
sequencing section already reflects this dependency.

Functional scalar accessors (`H::Decimal`, `bool`, `u64`, etc.)
return by value from the record's cached fields. Non-functional
accessors return `&[T]` via the record's slice field. The
chain-resolver method `resolve_{m}` is generated alongside every
object-accessor method whose trait method returns `&Self::{Assoc}`
— it's not part of the trait, but an ergonomic extra on
`Resolved{Foo}` so hosts don't have to manually construct
`Resolved{RangeClass}::new(record.m_handle, r)`.

Red tests: `path1_round_trip` (insert record, resolve via
`Resolved{Foo}::new`, observe accessor returns), `path1_absent_semantics`
(resolver returns None, observe `EMPTY_*` sentinels and
`&<Null{Range}<H>>::ABSENT` returns), `path1_chain_resolver`
(the `resolve_{m}` method chains through nested handles).

Phase 8 close condition: every Path-1 class has `{Foo}Handle<H>`,
`{Foo}Resolver<H>`, `{Foo}Record<H>`, `Resolved{Foo}<...>`, and a
round-trip test confirms non-trivial data flow. The `rust/orphan_counts`
validator continues to pass (Null stubs remain as alternative
impls — coherence discussion in §Composition).

### Phase 9 — `HostTypes::Decimal` arithmetic (Phase 1c completed)

**9a — Add arithmetic bounds to `HostTypes::Decimal`** per R2:
`Copy + Default + PartialOrd + Add<Output=Self> + Sub<Output=Self> +
Mul<Output=Self> + Div<Output=Self> + From<u32>`. Breaking change
documented in CHANGELOG.md.

**9b — Thread `H::Decimal`** through every foundation type carrying
an `f64` field or accessor. Explicit list (ripgrep-verified at
plan-time against the current workspace):

| Current type (pre-Phase-9) | Phase-9 target |
|---|---|
| `LandauerBudget { nats: f64, _sealed: () }` | `LandauerBudget<H: HostTypes> { nats: H::Decimal, _sealed: () }` |
| `UorTime { landauer_nats: LandauerBudget, rewrite_steps: u64, _sealed: () }` | `UorTime<H: HostTypes> { landauer_nats: LandauerBudget<H>, rewrite_steps: u64, _sealed: () }` |
| `Calibration { k_b_t: f64, thermal_power: f64, characteristic_energy: f64, … }` | `Calibration<H: HostTypes> { k_b_t: H::Decimal, thermal_power: H::Decimal, characteristic_energy: H::Decimal, … }` |
| `DDeltaMetric { value: f64 }` | `DDeltaMetric<H: HostTypes> { value: H::Decimal }` |
| `SigmaValue { value: f64 }` | `SigmaValue<H: HostTypes> { value: H::Decimal }` |
| `BettiMetric`, `EulerMetric`, `ResidualMetric`, `JacobianMetric` (integer-backed) | Parameterize `<H: HostTypes>` even when fields are integer-only, for arithmetic consistency when combined with `H::Decimal` observables downstream |
| `pipeline::parse_f64_from_bits_str(s: &str) -> f64` | Split into `pipeline::parse_u64_bits_str(s: &str) -> u64` (raw bits) + call-site `H::Decimal::from_bits(u64)` via the Phase-9c `DecimalTranscendental::from_bits` method |
| `bridge/trace.rs::POST_COLLAPSE_LANDAUER_COST: f64 = 0.6931471805599453` | `POST_COLLAPSE_LANDAUER_COST_BITS: u64 = f64::to_bits(0.6931471805599453)` + call-site `H::Decimal::from_bits(POST_COLLAPSE_LANDAUER_COST_BITS)` |
| `bridge/trace.rs::PRE_COLLAPSE_ENTROPY: f64`, `user/state.rs::CONTEXT_TEMPERATURE: f64`, `user/state.rs::GROUNDING_DEGREE: f64`, `kernel/reduction.rs::PRESSURE_THRESHOLD: f64` | Same pattern — `{NAME}_BITS: u64 = f64::to_bits(<value>)` + call-site conversion |
| `enforcement::primitive_descent_metrics<T>(…) -> (u32, f64)` | `primitive_descent_metrics<T, H>(…) -> (u32, H::Decimal)` |
| `enforcement::primitive_measurement_projection(budget: u64) -> (u64, f64)` | `primitive_measurement_projection<H>(budget: u64) -> (u64, H::Decimal)` |
| `enforcement::fold_descent_metrics<H: Hasher>(hasher: H, residual_count: u32, entropy: f64) -> H` | Replace `entropy: f64` with `entropy_bits: u64` (passes raw bit pattern through the hasher — the hasher's output doesn't care about the type distinction) |
| `enforcement::math::{ln, exp, sqrt, entropy_term_nats}(x: f64) -> f64` | `math::{ln, exp, sqrt, entropy_term_nats}<D: DecimalTranscendental>(x: D) -> D` — delegates to the trait methods defined in Phase 9c |

**Every renamed type is a breaking change.** CHANGELOG.md (Phase
13d) enumerates each; `MIGRATION-0.4.md` (Phase 9 release artifact)
provides per-type migration snippets. Crate version bumps from
0.3.x → **0.4.0** at Phase 9 release.

Public re-exports in `lib.rs` carry the generic parameter. `uor!`
and SDK macros in `uor-foundation-sdk` propagate the parameter
through their expansion; downstream specifying `<DefaultHostTypes>`
is the one-line fix for most call sites.

Red test:
[foundation/tests/orphan_closure_decimal_threading.rs](foundation/tests/orphan_closure_decimal_threading.rs)
defines a stub `HostTypes` with `type Decimal = MyFixed256` (a
fixed-point type implementing the Phase-9c arithmetic bounds)
and exercises the full pipeline `validate → run → certify`,
asserting every decimal-bearing observable is `MyFixed256`, not
`f64`.

**9c — Remove all 104 `f64` hardcodes.** `libm` remains the math
substrate for the default `f64` / `f32` `HostTypes`. The boundary to
`H::Decimal` is a new supertrait in `foundation/src/lib.rs`:

```rust
/// Transcendental math on `H::Decimal`. Satisfied by `f64` and
/// `f32` via `libm::{log, exp, sqrt}`; downstream host types
/// provide their own impls (interval arithmetic, arbitrary-precision,
/// fixed-point, etc.).
pub trait DecimalTranscendental:
    Copy + Default + PartialOrd
    + core::ops::Add<Output = Self>
    + core::ops::Sub<Output = Self>
    + core::ops::Mul<Output = Self>
    + core::ops::Div<Output = Self>
    + From<u32>
{
    fn ln(self) -> Self;
    fn exp(self) -> Self;
    fn sqrt(self) -> Self;
    fn from_bits(bits: u64) -> Self;
    fn to_bits(self) -> u64;
    fn entropy_term_nats(self) -> Self {
        if self == Self::default() { return Self::default(); }
        self * self.ln()
    }
}

impl DecimalTranscendental for f64 {
    fn ln(self) -> Self { libm::log(self) }
    fn exp(self) -> Self { libm::exp(self) }
    fn sqrt(self) -> Self { libm::sqrt(self) }
    fn from_bits(bits: u64) -> Self { f64::from_bits(bits) }
    fn to_bits(self) -> u64 { f64::to_bits(self) }
}

impl DecimalTranscendental for f32 {
    fn ln(self) -> Self { libm::logf(self) }
    fn exp(self) -> Self { libm::expf(self) }
    fn sqrt(self) -> Self { libm::sqrtf(self) }
    // f32 has no natural `from_bits: u64 -> f32`; widen then narrow.
    fn from_bits(bits: u64) -> Self { f64::from_bits(bits) as f32 }
    fn to_bits(self) -> u64 { (self as f64).to_bits() }
}
```

`HostTypes::Decimal: DecimalTranscendental` becomes the
arithmetic-bound supertrait (per R2). Every primitive previously
returning `f64` gains a generic parameter `<H>` and returns
`H::Decimal`; the body calls `decimal.ln()` / `.exp()` / `.sqrt()`
directly via trait dispatch.

**ln(2) and other math constants** become bit-pattern const
definitions. Example:

```rust
// Before (bridge/trace.rs):
pub const POST_COLLAPSE_LANDAUER_COST: f64 = 0.6931471805599453;

// After (generated from codegen; constant is in raw bits form):
pub const POST_COLLAPSE_LANDAUER_COST_BITS: u64 = 0x3FE62E42FEFA39EF;
// Call-site:
let cost: H::Decimal = H::Decimal::from_bits(POST_COLLAPSE_LANDAUER_COST_BITS);
```

`0x3FE62E42FEFA39EF` is the IEEE-754 bit pattern of `ln(2)`; it
matches `f64::to_bits(f64::consts::LN_2)`. Documented inline so
readers don't need a hex calculator.

Tests verify bit-identical output to the pre-change behavior when
`H = DefaultHostTypes` (i.e., `H::Decimal = f64`).

No allow-list in the Phase-9d `no_hardcoded_f64` gate. Every `: f64`
or `-> f64` in non-test foundation code gets rewritten or the phase
doesn't close.

**9d — `no_hardcoded_f64` conformance gate.** Greps
`foundation/src/**/*.rs` for `: f64` and `-> f64` outside `#[cfg(test)]`.
Zero matches at Phase 9 close. No allow-list.

**9e — `HostTypesDiscipline` conformance category (R16).** Verifies
the new bounds are in place and the transcendentals module consumes
`H::Decimal` through the arithmetic trait.

Red tests: `decimal_threading` (stub `HostTypes` with custom Decimal
type compiles through every primitive), `no_hardcoded_f64` (gate).

Phase 9 close condition: 0 f64 matches, `rust/no_hardcoded_f64` and
`rust/host_types_discipline` pass, every in-tree `HostTypes` impl
(`DefaultHostTypes`, `EmbeddedHost` example) satisfies the new bounds.

### Phase 10 — VerifiedMint scaffolds for Path-2 (Phase 3 completed)

**10a — R6 `THEOREM_IDENTITY` resolution** via the
`proof:provesIdentity` inverse lookup.

**Ontology fact.** Grep confirms
[spec/src/namespaces/proof.rs:339](spec/src/namespaces/proof.rs#L339)
defines `proof:provesIdentity` as the object property linking
`proof:AxiomaticDerivation` individuals to their `op:Identity`
target. **`op:provesFor` does not exist** — the original plan's R6
text referenced a nonexistent property.

**Resolution algorithm** (implemented in
`classification::classify_path2_theorem`):

1. Extract the Path-2 class's local name (e.g., `PartitionProduct`).
2. Apply the family prefix map below to derive a candidate theorem
   family prefix.
3. Enumerate all `op:Identity` individuals whose IRI contains the
   prefix; collect candidate IRIs.
4. If exactly one candidate: use that IRI as `THEOREM_IDENTITY`.
5. If zero candidates OR ≥ 2 candidates: fall back to the
   hand-override table
   `PATH2_THEOREM_OVERRIDES: &[(&str, &str)]` in
   [codegen/src/classification.rs](codegen/src/classification.rs).
   Missing override = **Phase-0 classification fails loud**.

**Family prefix map** (class local-name suffix → theorem family):

| Class local-name suffix | Theorem family |
|---|---|
| `PartitionProduct` | `PT_` |
| `PartitionCoproduct` | `ST_` |
| `CartesianPartitionProduct` | `CPT_` |
| `Obstruction` | `OB_` |
| `InhabitanceWitness`, `InhabitanceImpossibilityWitness` | `IH_` |
| `LiftObstruction` | `LO_` |
| `GroundingWitness`, `ProjectionWitness` | `OA_` |
| `BornRuleVerification` | `BR_` |
| `CompletenessWitness` | `CC_` |
| `DisjointnessWitness` | `DP_` |

**New theorem-family prefixes (BR_, CC_, DP_).** If any of these
prefixes doesn't yet appear in
[spec/src/namespaces/op.rs](spec/src/namespaces/op.rs), Phase 10a's
first step is adding the missing `op:Identity` individuals + the
matching `proof:AxiomaticDerivation` individuals to maintain the
identity-proof bijection. This is ontology work, not codegen work —
and it's part of Phase 10a, not a deferral.

**All 10 Path-2 classes must resolve.** The Phase-0 classification
test
[codegen/tests/path2_theorem_linkage.rs](codegen/tests/path2_theorem_linkage.rs)
asserts every Path-2 class resolves to a unique `op:Identity` IRI
via either the family prefix or the override table, AND the
resolved IRI exists as a real individual in `Ontology::full()`.
Failure = Phase-0 test red = Phase 10a cannot close.

**10b — R5 `{Foo}MintInputs<H>` field mapping.** For each Path-2 class,
walk its properties and emit a field per object-property (`{Range}Handle<H>`)
+ a field per datatype-property (xsd-mapped scalar).

**10c — R7 `entropy_bearing` in emission.** When a Path-2 class's
MintInputs carries entropy (per R7 set), the emitted witness omits
`Hash` from the derive; when not, it includes it. Verified by R14's
`no_entropy_hash` gate which already exists.

**10d — Emit witness scaffolds** with explicit family-to-primitive-
module routing. Rename
`generate_product_coproduct_amendment` at
[codegen/src/enforcement.rs:11090](codegen/src/enforcement.rs#L11090)
into `generate_witness_scaffolds` (generalized). For every Path-2
class with a resolved `THEOREM_IDENTITY` (per 10a):

```rust
// Hash omitted iff entropy_bearing per R7 / Phase 1b.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct {Foo}Witness { content_fingerprint: ContentFingerprint }

impl {Foo}Witness {
    pub const THEOREM_IDENTITY: &'static str = "{resolved op:Identity IRI}";
    pub const fn content_fingerprint(&self) -> ContentFingerprint {
        self.content_fingerprint
    }
}

pub struct {Foo}MintInputs<H: HostTypes> {
    // One field per direct object property of {Foo}:
    //   {range}_handle: {Range}Handle<H>
    //   (or Null{Range}<H> for Path-4 range classes)
    // One field per direct datatype property:
    //   {datatype_field}: H::Decimal  |  u64  |  &'static str  |  etc.
}

impl Certificate for {Foo}Witness { /* canonical Certificate impl */ }

impl VerifiedMint for {Foo}Witness {
    type Inputs<H: HostTypes> = {Foo}MintInputs<H>;
    fn mint<H: HostTypes>(inputs: Self::Inputs<H>)
        -> Result<Self, GenericImpossibilityWitness>
    {
        crate::primitives::{family}::verify_{ident}(inputs)
    }
}
```

**Family-to-primitive-module routing table** (mirrors the Phase 12
`foundation/src/primitives/{family}.rs` layout):

| `THEOREM_IDENTITY` prefix | Primitive module |
|---|---|
| `PT_` | `crate::primitives::pt` |
| `ST_` | `crate::primitives::st` |
| `CPT_` | `crate::primitives::cpt` |
| `OB_` (covers OB_M, OB_C, OB_H, OB_P) | `crate::primitives::ob` |
| `IH_` | `crate::primitives::ih` |
| `LO_` | `crate::primitives::lo` |
| `OA_` | `crate::primitives::oa` |
| `BR_` | `crate::primitives::br` |
| `CC_` | `crate::primitives::cc` |
| `DP_` | `crate::primitives::dp` |

The `{ident}` is the `to_snake_case` form of the `op:Identity`'s
local name — e.g., `pt_1`, `pt_2a`, `cpt_2a`, `ob_p_1`, `oa_3`. Phase
12 supplies each primitive file with stub bodies returning
`Err(GenericImpossibilityWitness::for_identity("WITNESS_UNIMPLEMENTED_STUB:{IRI}"))`;
Phase 12c replaces them with real verification.

**Partition-algebra amendment carve-out.** The amendment's three
existing witnesses (`PartitionProductWitness`, `PartitionCoproductWitness`,
`CartesianProductWitness`) are classified `AlreadyImplemented` in
Phase 0 and excluded from Phase-10d emission so the amendment code
is not touched.

Red tests:
[codegen/tests/witness_scaffold_emission.rs](codegen/tests/witness_scaffold_emission.rs)
— every Path-2 class has `{Foo}Witness`, `{Foo}MintInputs<H>`,
`impl Certificate`, and `impl VerifiedMint for {Foo}Witness`
emitted with correct family routing.
[codegen/tests/path2_hashable_rule.rs](codegen/tests/path2_hashable_rule.rs)
— `Hash` derive presence matches the R7 `entropy_bearing`
classification: `entropy_bearing == true` ⇒ no `Hash`.
[codegen/tests/path2_theorem_linkage.rs](codegen/tests/path2_theorem_linkage.rs)
— every `THEOREM_IDENTITY` resolves to a real `op:Identity`
individual.

Phase 10 close condition: every Path-2 class has full witness
scaffolding with real primitive routing (stubbed bodies are
acceptable at Phase 10 close because Phase 12 fills them; the stub
return value contains the theorem IRI, so the worklist is mechanical
and finite). The Null stub from Phase 7 is still emitted as an
alternative impl target; hosts pick which to use. The amendment's
three witnesses remain untouched.

### Phase 11 — Path-3 blanket impls (Phase 4 completed)

**11a — Populate `PATH3_ALLOW_LIST`** with a concrete first-pass
table. Each pair grep-verified (at plan time) to name a real
primitive in `foundation/src/enforcement.rs` or
`foundation/src/pipeline.rs`:

```rust
// codegen/src/classification.rs
const PATH3_ALLOW_LIST: &[(&str, &str)] = &[
    // observable:LandauerBudget — accumulator via primitive_descent_metrics.
    ("https://uor.foundation/observable/LandauerBudget",
     "primitive_descent_metrics"),
    // observable:BettiObservable — constraint-nerve Betti tuple.
    ("https://uor.foundation/observable/BettiObservable",
     "primitive_simplicial_nerve_betti"),
    // observable:EulerObservable — Euler characteristic from Betti tuple.
    ("https://uor.foundation/observable/EulerObservable",
     "primitive_euler_characteristic"),
    // observable:JacobianObservable — curvature Jacobian primitive.
    ("https://uor.foundation/observable/JacobianObservable",
     "primitive_curvature_jacobian"),
    // carry:CarryDepthObservable — depth via dihedral signature.
    ("https://uor.foundation/carry/CarryDepthObservable",
     "primitive_dihedral_signature"),
    // derivation:DerivationDepthObservable — terminal-reduction trace.
    ("https://uor.foundation/derivation/DerivationDepthObservable",
     "primitive_terminal_reduction"),
    // partition:FreeRankObservable — residual count from descent metrics.
    ("https://uor.foundation/partition/FreeRankObservable",
     "primitive_descent_metrics"),
];
```

**Second-pass sweep (11a').** After first pass lands, iterate all
Path-1-classified classes in `observable/`, `reduction/`, `schema/`
namespaces; match any whose accessor names against an existing
primitive function. Each match moves the class from Path-1 to
Path-3 and adds an entry to the allow-list. The fixed-point
classifier re-runs; any class whose inclusion breaks the emitable
set fails loud per Phase 7e's `rust/orphan_counts` gate.

**R13 loud failure gate.** The classifier's `classify()` function
checks each `Path3PrimitiveBacked` allow-list entry against the
primitive grep; missing primitive = `classification_counts` test
red at Phase 0. Adding an entry without the corresponding
primitive is therefore mechanically impossible without also
adding the primitive.

Red test:
[codegen/tests/path3_primitive_backing.rs](codegen/tests/path3_primitive_backing.rs)
— greps `foundation/src/enforcement.rs` +
`foundation/src/pipeline.rs` for each `{primitive}` name in
`PATH3_ALLOW_LIST`. Any miss = test red.

**11b — R13 loud failure.** Classifier now checks each
`Path3PrimitiveBacked` entry against the primitive grep; misses fail
`classification_counts` at Phase 0 time. Forces allow-list additions
to include an existing primitive.

**11c — R8 `@codegen-exempt` banner preservation** in `emit::write_file`:
if an existing file starts with `// @codegen-exempt`, don't
overwrite it.

**11d — R9 `pub mod blanket_impls;`** in generated `foundation/src/lib.rs`.

**11e — Write `foundation/src/blanket_impls.rs`** (hand-written,
`@codegen-exempt`). Full impl shape per allow-list entry:

```rust
impl<T, H> {Foo}<H> for crate::enforcement::Validated<T, H>
where
    T: crate::pipeline::ConstrainedTypeShape + ?Sized,
    H: HostTypes,
    H::Decimal: DecimalTranscendental,  // per Phase 9c
{
    // Scalar accessors: delegate to the primitive with (T, H)
    // generics; the primitive returns H::Decimal (post-Phase-9).
    fn value(&self) -> H::Decimal {
        crate::enforcement::primitive_foo_observable::<T, H>(self)
    }

    // Object accessors: use Phase-7 Null stub for the range class.
    // The associated type is Null{Range}<H>; return `&<Null{Range}<H>>::ABSENT`.
    // Reason: a Validated<T, H> doesn't carry content-addressed
    // handles for its abstract range classes — those come from the
    // Resolved-wrapper path (Phase 8). For Path-3 blanket impls,
    // the content of the observable IS the return; sub-handles
    // aren't materialized.
    type {Assoc} = crate::Null{RangeClass}<H>;
    fn m(&self) -> &Self::{Assoc} {
        &<crate::Null{RangeClass}<H>>::ABSENT
    }
}
```

**Coherence with Phase 7 + Phase 8.** Three concrete impls per
Path-3-allow-listed class:

1. `impl {Foo}<H> for Null{Foo}<H>` (Phase 7 — resolver-absent stub)
2. `impl {Foo}<H> for Resolved{Foo}<'r, R, H>` (Phase 8 —
   content-addressed resolution)
3. `impl {Foo}<H> for Validated<T, H>` (Phase 11 — primitive-backed)

Rust coherence is satisfied because `Null{Foo}<H>`,
`Resolved{Foo}<'r, R, H>`, and `Validated<T, H>` are mutually
disjoint concrete types (different shapes, different generics).
Each impl closes the orphan in the R1 grep count.

**Supertrait closure.** Every supertrait of `{Foo}` that the blanket
impl claims must ALSO have a blanket impl on `Validated<T, H>`. If
the supertrait is itself Path-3-allow-listed, that's automatic. If
the supertrait is a plain Path-1 trait (e.g., `Component<H>` is a
supertrait of many observables), its blanket impl ALSO lands in
`blanket_impls.rs`. The classifier pre-computes the transitive
closure of `PATH3_ALLOW_LIST` supertraits and requires all of
them to be present; missing supertrait blanket = compile error at
first `Validated<T, H>` use site.

**`@codegen-exempt` banner.** First line of `blanket_impls.rs`:

```rust
// @codegen-exempt — hand-written blanket impls for Path-3 classes.
// See docs/orphan-closure/completion-plan.md §Phase 11 for the full
// allow-list and supertrait-closure rule.
```

Phase 11c's `emit::write_file` banner-check preserves this file
across `uor-crate` regen runs.

**CHANGELOG.md** notes each new blanket impl at trait granularity
so downstream users see the new primitive-backed impl surface.

**11f — Update CLAUDE.md** to carve out `blanket_impls.rs` from the
"never hand-edit" rule.

Red tests: `path3_blanket_smoke` (instantiate blanket impl, invoke
methods), `path3_primitive_backing` (every allow-list entry names a
real primitive), `blanket_impls_exempt` (regen doesn't overwrite).

Phase 11 close condition: ~20–30 blanket impls land, backing ~100
observable/reduction traits. Orphan count unchanged (they already
had Null stubs from Phase 2/7); quality increases — downstream
using `Validated<T, H>` gets real observables, not absent sentinels.

### Phase 12 — Per-theorem primitive bodies (Phase 5 completed)

**12a — Create `foundation/src/primitives/` directory (R10).** `mod.rs`
(generated, empty preamble) + per-family files `pt.rs`, `st.rs`,
`cpt.rs`, `ob_p.rs`, `cpt_extended.rs` (each `@codegen-exempt`).

**12b — R15 theorem-family worklist.** Walk
`spec/src/namespaces/op.rs` in document order; each `op:Identity`
without an existing primitive implementation goes into a per-family
todo list committed into the per-family file's module doc.

**12c — Implement verification bodies** for every `op:Identity`
individual in
[spec/src/namespaces/op.rs](spec/src/namespaces/op.rs) that lacks an
amendment-covered body. Grep-verified enumeration (re-verify at
Phase-12 start):

**Amendment-covered** (no Phase-12 work):

| Family | Identities |
|---|---|
| PT | PT_1, PT_3, PT_4 |
| ST | ST_1, ST_2, ST_6, ST_7, ST_8, ST_9, ST_10 |
| CPT | CPT_1, CPT_3, CPT_4, CPT_5 |

**Phase-12 targets** (every identity below gets a hand-written
verification body in the matching primitive file):

| Family | Identities | Primitive file |
|---|---|---|
| PT | PT_2, PT_2a, PT_2b | `foundation/src/primitives/pt.rs` |
| ST | ST_3, ST_4, ST_5 | `foundation/src/primitives/st.rs` |
| CPT | CPT_2a, CPT_6 | `foundation/src/primitives/cpt.rs` |
| IH | IH_1, IH_2a, IH_2b, IH_3 | `foundation/src/primitives/ih.rs` |
| OA | OA_1, OA_2, OA_3, OA_4, OA_5 | `foundation/src/primitives/oa.rs` |
| OB_M | OB_M1, OB_M2, OB_M3, OB_M4, OB_M5, OB_M6 | `foundation/src/primitives/ob.rs` |
| OB_C | OB_C1, OB_C2, OB_C3 | `foundation/src/primitives/ob.rs` |
| OB_H | OB_H1, OB_H2, OB_H3 | `foundation/src/primitives/ob.rs` |
| OB_P | OB_P1, OB_P2, OB_P3 | `foundation/src/primitives/ob.rs` † |
| LO | (enumerate from op.rs at phase start) | `foundation/src/primitives/lo.rs` |
| BR | BR_1..BR_n (added to op.rs per Phase 10a) | `foundation/src/primitives/br.rs` |
| CC | CC_1..CC_n (added to op.rs per Phase 10a) | `foundation/src/primitives/cc.rs` |
| DP | DP_1..DP_n (added to op.rs per Phase 10a) | `foundation/src/primitives/dp.rs` |

† **OB_P cohomology caveat (binding per non-deferment policy).**
OB_P1/P2/P3 depend on computable cohomology machinery. If Phase 12
reaches these before cohomology grounds in Phase 14, OB_P bodies
ship as:

```rust
pub fn verify_ob_p_1<H: HostTypes>(_inputs: OBP1MintInputs<H>)
    -> Result<OBP1Witness, GenericImpossibilityWitness>
{
    Err(GenericImpossibilityWitness::for_identity(
        "THEORY_DEFERRED:OB_P_1",
    ))
}
```

This is **not a deferral** per the non-deferment policy — the
classes OB_P1/P2/P3 are already tracked by Phase 6's theory-deferred
register. The `THEORY_DEFERRED:OB_P_*` error IRI is the explicit
tracker; Phase 14 replaces the body when cohomology lands. The
impossibility witness lets calling code distinguish "theorem failed
to verify" from "theory for this theorem isn't grounded yet".

**Per-theorem body shape.** Bodies follow the amendment pattern
(`pc_primitive_partition_coproduct`, etc.): verify
`{Foo}MintInputs<H>` fields against the theorem statement; on
match, return `Ok({Foo}Witness { content_fingerprint: fp })` where
`fp` is computed over `(THEOREM_IDENTITY, canonical(inputs))`; on
mismatch, return a typed impossibility with a specific failure-mode
IRI documented in the per-family phase-5 subsection.

**Red tests (per family).**
[foundation/tests/orphan_closure_theorem_pt.rs](foundation/tests/orphan_closure_theorem_pt.rs),
[foundation/tests/orphan_closure_theorem_st.rs](foundation/tests/orphan_closure_theorem_st.rs),
[foundation/tests/orphan_closure_theorem_cpt.rs](foundation/tests/orphan_closure_theorem_cpt.rs),
[foundation/tests/orphan_closure_theorem_ih.rs](foundation/tests/orphan_closure_theorem_ih.rs),
[foundation/tests/orphan_closure_theorem_oa.rs](foundation/tests/orphan_closure_theorem_oa.rs),
[foundation/tests/orphan_closure_theorem_ob.rs](foundation/tests/orphan_closure_theorem_ob.rs),
[foundation/tests/orphan_closure_theorem_lo.rs](foundation/tests/orphan_closure_theorem_lo.rs),
[foundation/tests/orphan_closure_theorem_br.rs](foundation/tests/orphan_closure_theorem_br.rs),
[foundation/tests/orphan_closure_theorem_cc.rs](foundation/tests/orphan_closure_theorem_cc.rs),
[foundation/tests/orphan_closure_theorem_dp.rs](foundation/tests/orphan_closure_theorem_dp.rs)
— per family, golden-input cases return `Ok(witness)`; documented
failure-mode cases return the specific impossibility IRI. Family
completion criterion: `path2_stub_rejection`'s rows for that family
drop to zero; the only remaining stub-rejection assertions are the
`THEORY_DEFERRED:OB_P_*` entries which are a Phase-14 artifact.

**12d — Wire VerifiedMint → primitive.** Phase 10's stub body
becomes a real call once the primitive exists. Remove the
`WITNESS_UNIMPLEMENTED_STUB:*` return; return `Ok(witness)` on
verification success, typed impossibility on documented failure
modes.

Red tests: `theorem_pt`, `theorem_st`, `theorem_cpt`, `theorem_ob_p`
— per family, golden inputs return `Ok(witness)`, documented
failure modes return specific impossibility IRIs.

Phase 12 close condition: no stub body remains in
`foundation/src/primitives/*.rs`; every mint call either returns
`Ok` or a typed impossibility. Cohomology-dependent theorems (OB_P_*)
may stay deferred but are explicitly tracked under Phase 14.

### Phase 13 — Cross-cutting infrastructure

**13a — R1 orphan count as conformance check.** The minimum-viable
version shipped in Phase 7e (per Edit 6). Phase 13a expands it to
the full classifier-integrated validator
`conformance/src/validators/rust/orphan_counts.rs`:

**Algorithm (4 steps):**

1. **Trait enumeration.** Parse every
   `pub trait {Name}<H: HostTypes>` declaration across
   `foundation/src/**/*.rs`. Skip classes in
   `Ontology::enum_class_names()` (they don't produce traits).

2. **Impl search.** For each trait, search the full workspace
   (`foundation/`, `uor-foundation-sdk/`, `conformance/`,
   `uor-foundation-test-helpers/`, `clients/`, `cargo-uor/`) for
   the impl regex:

   ```text
   ^\s*impl(<[^>]*>)?\s+(crate::)?([\w_]+(::[\w_]+)*::)?({Name})(<[^>]*>)?\s+for\s+
   ```

   Exclude lines inside `#[cfg(test)]` blocks. Tracking is via a
   simple brace-depth counter: when a line opens a block following
   `#[cfg(test)]`, increment the "inside-test" depth; decrement on
   closing brace.

3. **Categorize matches.** Each impl match is classified by target:

   | Target prefix | Category |
   |---|---|
   | `Null{Name}<H>` | `null_stub` |
   | `Resolved{Name}<'r, R, H>` | `resolved_wrapper` |
   | `Validated<T, H>` | `validated_blanket` |
   | `{Name}Witness` or `*Certificate` | `verified_mint` |
   | Anything else | `hand_written` |

4. **Report.** A trait is **closed** iff ≥ 1 impl matches. Pass ↔
   orphan count ≤ 20 (the Path-4 theory-deferred count, whose only
   impl is the Phase-7d `#[doc(hidden)]` stub). Per-category
   closure counts emitted as validator details.

**Classifier cross-check.** Phase-0 classification predicts each
trait's closure set; Phase 13a asserts the prediction matches the
actual impl surface. Specifically:

- Every `AlreadyImplemented` class has ≥ 1 `hand_written` or
  `verified_mint` impl (amendment witnesses).
- Every `Path1HandleResolver` class has ≥ 1 `null_stub`, ≥ 1
  `resolved_wrapper`, and (if Path-3-allow-listed) ≥ 1
  `validated_blanket`.
- Every `Path2TheoremWitness` class has ≥ 1 `null_stub` and
  ≥ 1 `verified_mint` impl.
- Every `Path3PrimitiveBacked` class has ≥ 1 `validated_blanket`.
- Every `Path4TheoryDeferred` class has exactly 1 `null_stub`
  (the `#[doc(hidden)]` THEORY-DEFERRED stub from Phase 7d).

Cross-check mismatch = hard fail with a diagnostic listing which
class's prediction differs from actual impl surface.

**13b — R11 `load_doc_fragment`.** Add the helper to
[codegen/src/emit.rs](codegen/src/emit.rs):

```rust
/// Loads a named fragment from a Markdown file for inline rustdoc
/// emission. See §Phase-13b of the completion plan for the marker
/// format and resolution rules.
///
/// # Panics
/// - `{source_path}` doesn't exist
/// - `{key}` not found in `{source_path}`
pub fn load_doc_fragment(source_path: &str, key: &str) -> String {
    // parse Markdown for `<!-- doc-key: {key} -->` marker, extract
    // the following paragraph(s) up to the terminator, strip
    // leading/trailing whitespace, return.
}
```

**Marker format** in `docs/orphan-closure/phase-*.md` files:

```markdown
<!-- doc-key: {kind}:{name} -->
The paragraph(s) of rustdoc content to inline. Plain Markdown — no
`//!` / `///` prefixes. The fragment ends at the first of:

- the next `<!-- doc-key: ... -->` marker
- a `##` or `###` Markdown heading
- the literal terminator `<!-- /doc-key -->`
<!-- /doc-key -->
```

**Accepted `{kind}` values** — the codegen's call site names which
kind it's emitting, and the phase doc answers with matching kind:

| `{kind}` | Emitted at |
|---|---|
| `class` | trait body (Phase 2's existing trait rustdoc) |
| `trait` | alias for `class` |
| `handle` | `{Foo}Handle<H>` struct (Phase 8) |
| `resolver` | `{Foo}Resolver<H>` trait (Phase 8) |
| `record` | `{Foo}Record<H>` struct (Phase 8) |
| `resolved` | `Resolved{Foo}<'r, R, H>` wrapper (Phase 8) |
| `null-stub` | `Null{Foo}<H>` struct (Phases 7/7d) |
| `witness` | `{Foo}Witness` struct (Phase 10) |
| `mint-inputs` | `{Foo}MintInputs<H>` struct (Phase 10) |
| `blanket-impl` | `impl {Foo}<H> for Validated<T, H>` (Phase 11) |
| `primitive` | `fn verify_{ident}` (Phase 12) |

`{name}` is the ontology class's local name, OR the primitive
`op:Identity` local name for `kind=primitive`.

**Resolution semantics:**

- `source_path` is relative to the workspace root (e.g.,
  `docs/orphan-closure/phase-8-resolved-wrapper.md`).
- Missing file ⇒ `panic!("load_doc_fragment: file not found:
  {source_path}")`.
- Missing key in file ⇒ `panic!("load_doc_fragment: missing key
  '{key}' in {source_path}")`.
- Found ⇒ the paragraph content with leading/trailing blank lines
  trimmed and Markdown preserved for rustdoc rendering.

**Migration.** Phases 2, 3, and 7 currently embed hand-written
rustdoc strings (e.g., in
[codegen/src/traits.rs](codegen/src/traits.rs) `emit_null_stub`).
Phase 13b migrates every one of those calls to `load_doc_fragment`
against the appropriate phase doc. Every emitted type that
currently has hand-written rustdoc must have a matching
`<!-- doc-key -->` marker in the phase doc before Phase 13b closes.

**Every Null-stub / Resolved-wrapper / VerifiedMint-scaffold /
blanket-impl / primitive emission** — i.e., every codegen emission
that produces a public Rust type — calls `load_doc_fragment`
exactly once. Hand-written rustdoc in codegen is forbidden
starting Phase 13b close; the `codegen/tests/no_hand_written_rustdoc.rs`
test greps the codegen source for `f.doc_comment("...")` calls
with string literal arguments (not `load_doc_fragment(...)`
results) and fails on any match outside a small allow-list of
legitimately-hardcoded preambles.

Red test:
[codegen/tests/doc_key_resolution.rs](codegen/tests/doc_key_resolution.rs)
— for every emitted type, asserts the emitted rustdoc text matches
the corresponding phase-doc fragment verbatim.

**13c — R16 remaining conformance categories.** `TaxonomyCoverage`
(Phase 0 report parity), `OrphanCounts` (13a), `HostTypesDiscipline`
(9e) — all registered in `conformance/src/lib.rs` and counted in
`CONFORMANCE_CHECKS`.

**13d — CHANGELOG.md** documents every Phase 7–13 breaking change
(HostTypes::Decimal bounds, LandauerBudget parameterization,
Calibration parameterization, etc.).

**13e — Spec counts update.** `spec/src/counts.rs` gains
`CLASSIFICATION_PATH3` population, `CLASSIFICATION_PATH4_CLOSED` for
the theory-deferred set, removes the obsolete
`CLASSIFICATION_PATH1_EMITTED` ratchet in favor of the real orphan
count.

Phase 13 close condition: `rust/orphan_counts` reports the closure
ratio. `CONFORMANCE_CHECKS` arithmetic matches the number of active
validators.

### Phase 14 — Theory-deferred grounding (optional follow-up)

Out of current scope but explicitly tracked: the 20 theory-deferred
classes have open research questions in `docs/theory_deferred.md`.
Closing them requires advancing the theory. This phase doesn't ship
code; it's the placeholder that closes the completion plan when
theory catches up.

The `rust/theory_deferred_register` check from Phase 6 automatically
shrinks as classes re-classify out of Path-4.

## Composition and integration across phases

Per class, multiple concrete types coexist as distinct orphan
closures. Rust coherence is satisfied because the target types
are mutually disjoint.

**Per-classification coherence matrix:**

| Classification | Null stub (Phase 7) | Resolved wrapper (Phase 8) | Validated blanket (Phase 11) | VerifiedMint witness (Phase 10) |
|---|---|---|---|---|
| Path-1 (not Path-3-allow-listed) | yes | yes | no | no |
| Path-1 (Path-3-allow-listed) | yes | yes | yes | no |
| Path-2 (theorem witness) | yes | no | no | yes |
| Path-3 (explicit — rare as distinct bucket) | no | no | yes | no |
| Path-4 (theory-deferred) | yes (7d, `#[doc(hidden)]` THEORY-DEFERRED banner) | no | no | no |
| AlreadyImplemented (Partition-algebra amendment) | no (amendment owns) | no | no | yes (amendment) |

Every row has ≥ 1 "yes" in the right half, which is what makes the
R1 orphan-count validator pass.

**Concrete-type disjointness.** The target types per classification
are all distinct:

- `Null{Foo}<H>` — PhantomData-only unit struct (Phase 7).
- `Resolved{Foo}<'r, R, H>` — wrapper over handle + resolver +
  optional record (Phase 8).
- `Validated<T, H>` — existing foundation type, host data shape
  carrier (Phase 11 blanket impl target).
- `{Foo}Witness` — content-fingerprint-carrying sealed type
  (Phase 10 / amendment).

Rust's orphan rules treat each as a separate impl target; no
coherence conflict arises from the 4-way matrix above.

**Dependency graph across phases:**

```
Phase 7 (cascade unblockers, Null stubs + enum Default)
    │
    ├─▶ Phase 8 (Resolved wrapper — depends on Phase-7 Null stubs
    │                existing because Resolved{Foo} references
    │                Null{Range}::ABSENT)
    │
    ├─▶ Phase 9 (HostTypes::Decimal — independent of Phases 7/8 at
    │                the trait level but re-releases every generic
    │                type, so emits after)
    │
    └─▶ Phase 10 (VerifiedMint scaffolds — uses Phase-7 Null stubs
                     as MintInputs field types; uses Phase-10a theorem
                     identity resolution)

Phase 11 (Path-3 blanket impls) depends on:
    - Phase 9 (H::Decimal arithmetic bounds for primitive calls)
    - Phase 7 (Null stubs for object-accessor associated types)

Phase 12 (per-theorem primitives) depends on:
    - Phase 10 (VerifiedMint scaffolds route to primitives)
    - Phase 10a (theorem identity resolution per family)

Phase 13 (cross-cutting) depends on:
    - All prior phases (orphan-count validator references the
      cumulative impl surface)
```

**Semver impact per phase:**

- Phase 7: **additive** (new Null stubs, new enum `Default` impls,
  new Phase-7d THEORY-DEFERRED stubs). Minor bump.
- Phase 8: **additive** (new `{Foo}Handle`, `{Foo}Resolver`,
  `{Foo}Record`, `Resolved{Foo}` per Path-1 class). Minor bump.
- Phase 9: **major** (`HostTypes::Decimal` gains arithmetic bounds;
  `LandauerBudget`, `UorTime`, `Calibration`, `DDeltaMetric`,
  `SigmaValue`, etc. parameterize on `<H>`). Version bump
  0.3.x → **0.4.0**. Downstream migration documented in
  `MIGRATION-0.4.md`.
- Phase 10: **additive** on top of 0.4.0 (new witness/mint-inputs
  per Path-2 class). Minor bump.
- Phase 11: **additive** (new blanket impls on `Validated<T, H>`).
  Minor bump.
- Phase 12: **additive** (new primitive modules + real mint bodies).
  Minor bump.
- Phase 13: **internal** (conformance-only, no public-surface
  changes). Patch bump.

The crate's release cadence: ship Phase 7 on the 0.3.x line; ship
the 0.4.0 release at Phase 9; ship Phases 10–13 on the 0.4.x line.

## Sequencing

```
┌─ Phase 7 ─ cascade unblockers ─────────────────────────────────┐
│  7a enum Default → 7b inherited assoc → 7c enum imports →      │
│  7d Path-4 marker stubs → 7e drop enum filter                   │
│  Closes ~200 orphans; orphan count → ~40                        │
├─ Phase 8 ─ Resolved-wrapper design ────────────────────────────┤
│  8a Handle → 8b Resolver → 8c Record → 8d Resolved →           │
│  8e inherited-assoc wiring                                       │
│  Strong contracts for every Path-1 class; orphan count          │
│  unchanged (Null stubs remain as alternative)                   │
├─ Phase 9 ─ HostTypes::Decimal ─────────────────────────────────┤
│  9a bounds → 9b threading → 9c f64 removal → 9d gate → 9e cat  │
│  No orphan delta; unblocks Phase 11 arithmetic                  │
├─ Phase 10 ─ VerifiedMint scaffolds ────────────────────────────┤
│  10a R6 → 10b R5 → 10c R7 → 10d emission                       │
│  Every Path-2 class gets full witness surface                   │
├─ Phase 11 ─ Path-3 blanket impls ──────────────────────────────┤
│  11a allow-list → 11b R13 → 11c R8 → 11d R9 → 11e impls        │
│  ~100 traits gain primitive-backed impls                        │
├─ Phase 12 ─ Per-theorem primitives ────────────────────────────┤
│  12a directory → 12b R15 → 12c bodies → 12d wire               │
│  Every VerifiedMint mint returns real verification              │
├─ Phase 13 ─ Cross-cutting infrastructure ──────────────────────┤
│  13a R1 → 13b R11 → 13c R16 → 13d CHANGELOG → 13e counts       │
│  Conformance becomes authoritative                              │
└─ Phase 14 ─ theory-deferred grounding (out of scope) ──────────┘
```

Total: **7 more phases**, each independently mergeable. Every phase
ends with `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`,
`cargo run --bin uor-crate && git diff --exit-code`, and `cargo run
--bin uor-conformance` all green.

## Migration and breaking changes

Every breaking change in this plan is enumerated here.
[CHANGELOG.md](CHANGELOG.md) (updated in Phase 13d) mirrors this
list; `MIGRATION-0.4.md` (shipped with the Phase-9 release) provides
per-type migration snippets.

| Change | Phase | Severity | Rationale |
|---|---|---|---|
| `HostTypes::Decimal` gains arithmetic bounds (`DecimalTranscendental` supertrait — Copy + Default + PartialOrd + Add/Sub/Mul/Div + From<u32> + ln/exp/sqrt/from_bits/to_bits) | 9a+9c | **Major** | Can't make blanket impls or primitives sound without closed arithmetic; guiding principle is correctness over BC |
| `LandauerBudget` → `LandauerBudget<H>` | 9b | **Major** | `nats: f64` becomes `nats: H::Decimal`; downstream renames every reference |
| `UorTime` → `UorTime<H>` | 9b | **Major** | Embeds `LandauerBudget<H>`; transitive through every `Grounded`, `Certificate`, `Triad` |
| `Calibration` → `Calibration<H>` | 9b | **Major** | Physics constants (k_b_t, thermal_power, characteristic_energy) become `H::Decimal` |
| `DDeltaMetric` → `DDeltaMetric<H>`, `SigmaValue<H>`, `BettiMetric<H>`, `EulerMetric<H>`, `ResidualMetric<H>`, `JacobianMetric<H>` | 9b | **Major** | Observable metrics parameterized |
| `primitive_simplicial_nerve_betti` returns `Result<...>` | 1a (shipped) | Major (shipped) | Capacity fail-fast |
| `primitive_descent_metrics` returns `(u32, H::Decimal)` | 9b | **Major** | Decimal threading |
| `primitive_measurement_projection` returns `(u64, H::Decimal)` | 9b | **Major** | Same |
| Const renames: `POST_COLLAPSE_LANDAUER_COST` → `POST_COLLAPSE_LANDAUER_COST_BITS: u64` (same semantics, different call-site usage) | 9b | **Major** | Generic const constructors require bit-pattern form |
| New `pub trait DecimalTranscendental` | 9c | Additive | New supertrait required by `HostTypes::Decimal` bound |
| `{Foo}Handle<H>`, `{Foo}Resolver<H>`, `{Foo}Record<H>`, `Resolved{Foo}<'r, R, H>` per Path-1 class | 8a–8d | **Additive** | New public types; no existing API removed |
| `{Foo}Witness`, `{Foo}MintInputs<H>` per Path-2 class | 10d | **Additive** | New public types; amendment's three witnesses unchanged |
| `pub mod blanket_impls` in `foundation/src/lib.rs` | 11d | **Additive** | New hand-written module |
| `pub mod primitives` in `foundation/src/lib.rs` | 12a | **Additive** | New hand-written module for theorem-family primitive files |
| `Null{Class}<H>` per Path-1/2/4 class | 7d, existing + new | **Additive** | Phase 7 extends existing emission set |
| Enum `Default` derive + `#[default]` on first variant | 7a | **Additive** | Enums that didn't derive `Default` before now do; existing consumers unaffected |
| `WittLevel` gains `impl Default` returning `W8` | 7a | **Additive** | New trait impl; existing consumers unaffected |

**Crate version lifecycle:**

- **0.3.x** (current) — Phases 0–7 ship incrementally. Each minor
  bump adds phase work; no breakage to the public surface.
- **0.4.0** — Phase 9 release. Breaking generics rollout (all
  "Major" rows above lumped into a single major-version bump).
- **0.4.x** — Phases 10, 11, 12, 13 ship incrementally on the
  0.4 baseline. Each minor bump adds phase work; Phase 13 is a
  patch bump.
- **0.5.0** — Reserved for any follow-on breakage (deprecated-
  removal, cohomology grounding from Phase 14, etc.).

**Semver discipline.** No breakage except at a major version. The
Phase-9 release gates all subsequent phases — Phase 10+ merge only
after 0.4.0 publishes to crates.io.

**Phase 9 release artifacts** (delivered alongside the 0.4.0
version bump):

1. [CHANGELOG.md](CHANGELOG.md) with a dedicated 0.4.0 section
   enumerating every "Major" row above.
2. `MIGRATION-0.4.md` at repo root with per-type code diffs
   showing the pre- and post-migration form.
3. Deprecation warnings NOT used — 0.3.x items that become
   removed-or-renamed in 0.4.0 are simply removed at the version
   bump (no deprecation period). Rationale: the guiding principle
   prohibits half-states; deprecation shims create a half-state.

## End-state targets

At Phase 13 close:

| Metric | Target |
|---|---|
| Orphan count (R1 grep, classifier-integrated) | ≤ 20 (theory-deferred only, each with Phase-7d `#[doc(hidden)]` stub) |
| Hardcoded `f64` sites (`: f64` or `-> f64` outside `#[cfg(test)]`) | 0 |
| Path-1 classes with Null stub (Phase 7) | 100% |
| Path-1 classes with `{Foo}Handle<H>` / `{Foo}Resolver<H>` / `{Foo}Record<H>` / `Resolved{Foo}<'r, R, H>` (Phase 8) | 100% |
| Path-2 classes with Null stub + `{Foo}Witness` + `{Foo}MintInputs<H>` + `impl VerifiedMint` (Phases 7 + 10) | 100% |
| Path-3 classes with blanket impl on `Validated<T, H>` (Phase 11) | matches `PATH3_ALLOW_LIST` (≥ 7 initial + second-pass expansion) |
| Path-4 classes with theory-deferred Null stub + register row (Phases 7d + 6) | 20/20 |
| `primitives/{family}.rs` files (PT / ST / CPT / IH / OA / OB / LO / BR / CC / DP) | 10/10 (OB_P1..3 may carry `THEORY_DEFERRED:OB_P_*` body; every other body verifies) |
| `CONFORMANCE_CHECKS` | 540+ (adds TaxonomyCoverage, OrphanCounts, HostTypesDiscipline; Phase 6's TheoryDeferredRegister already landed) |
| `HostTypes::Decimal` bounds via `DecimalTranscendental` | full arithmetic closure + transcendentals |
| `CHANGELOG.md` | every breaking change documented at 0.4.0 section |
| `MIGRATION-0.4.md` published with Phase 9 release | yes |
| Crate version at completion | 0.4.0 (Phase 9) → 0.4.x (Phases 10–13) |
| `load_doc_fragment` coverage (hand-written rustdoc in codegen) | zero outside small allow-list; all emission routed through phase docs |

**Every phase in this completion plan is committed in full.** No
sub-task ships in a "narrowed" form; no sub-task is deferred to a
later phase. If a sub-task encounters a blocker that wasn't
anticipated, **the phase grows**: more commits, more tests, more
prerequisite corrections until the original sub-task lands. A new
phase may be **inserted** between existing phases for a large
prerequisite correction, but the downstream phase preserves its
full scope.

**Weakening or deferring planned features is not authorized.** The
phase lands, or the phase stays open. The only class of "unclosed"
artifacts permitted at completion-plan close is the Phase 14
theory-deferred set — research-blocked classes whose register
tracks *why* they're unclosed — and those already have Phase-7d
`#[doc(hidden)]` stubs in the source tree, so they satisfy the R1
orphan count without claiming semantic correctness. Every other
outcome is covered by the Phase 7–13 scopes defined above.

If a reviewer notices any phase shipping less than its full scope,
that phase is reopened. There is no "good enough" close; the
conformance gates are the arbiter, and they are mechanical.
