# CLAUDE.md — UOR-Framework

## Project overview

Rust workspace encoding the UOR Foundation ontology as typed data structures, a generated `#![no_std]` trait crate (`uor-foundation`), and validated serializations (JSON-LD, Turtle, N-Triples, OWL RDF/XML, JSON Schema, SHACL Shapes, EBNF). All source code, documentation, and web artifacts are machine-generated from the authoritative ontology defined in `spec/`.

## Authoritative source

The authoritative source for `uor-foundation` is the project wiki: <https://github.com/UOR-Foundation/UOR-Framework/wiki>. Consult it for canonical definitions, design rationale, and ontology semantics; treat it as the source of truth when wiki content and other docs disagree.

The wiki specifies **Prism** as the *standard library* (ADR-031), realized as a façade crate `prism` (sibling-repo `[UOR-Foundation/Prism]`) that re-exports `uor-foundation`'s architecture surface plus four standard-library Layer-3 sub-crates (`prism-numerics`, `prism-crypto`, `prism-tensor`, `prism-fhe`). This `UOR-Framework` repository hosts the architecture: the wiki (the normative specification) and the `uor-foundation` Rust crates (the architecture's Rust implementation, including `uor-foundation-sdk` and `uor-foundation-verify`). The `Prism` repository hosts the standard library; `prism-verify` (the wiki's replay surface) lives there too. This repo's responsibility for ADR-031 is to expose every symbol the façade re-exports as `pub` on `uor-foundation` / `uor-foundation-sdk`. When evolving the implementation toward the wiki, prefer making `uor-foundation` *scalable along the wiki's substitution axes* (ADR-030 generalized the third axis from `Hasher` to `AxisTuple`) over cosmetic crate-renaming.

## Substitution axes (wiki §2 + ADR-007 + ADR-018)

The wiki names four substitution parameters the application author selects against (three substitution axes + one resolver-tuple substrate parameter per ADR-036):

| Parameter | Trait (in `uor-foundation`) | What the application varies |
|---|---|---|
| `HostTypes` | `pub trait HostTypes` ([foundation/src/lib.rs](foundation/src/lib.rs)) | Host-environment representations: `Decimal`, `HostString`, `WitnessBytes`. Default impl: `DefaultHostTypes`. |
| `HostBounds` | `pub trait HostBounds` ([foundation/src/lib.rs](foundation/src/lib.rs)) | Every capacity bound that varies along the principal data path (wiki ADR-018): fingerprint output width range, trace event-count ceiling, algebraic-level bit-width ceiling. Default impl: `DefaultHostBounds` (16/32/256/64). |
| `AxisTuple` | `pub trait AxisTuple` ([foundation/src/pipeline.rs](foundation/src/pipeline.rs)) — formerly `Hasher`, generalized per ADR-030 | Substrate-extension axes selected by the application. The 1-tuple `(HashAxis<MyHasher>,)` (foundation-built [`HashAxis<H>`](foundation/src/enforcement.rs) adapter) reproduces the legacy single-Hasher behavior; multi-tuple forms `(HashAxis<H>, MyTensorAxis, MyFheAxis)` add user-declared substrate axes via the [`axis!`](uor-foundation-sdk/src/lib.rs) SDK macro. The legacy `pub trait Hasher<const FP_MAX: usize = 32>` ([foundation/src/enforcement.rs](foundation/src/enforcement.rs)) stays in this release as the content-addressing-at-mint-time concept (TC-05); per ADR-031 it relocates to `prism-crypto` as `HashAxis` in a future major. |
| `ResolverTuple` (4th model-declaration parameter per ADR-036) | `pub trait ResolverTuple` ([foundation/src/pipeline.rs](foundation/src/pipeline.rs)) | Application-provided per-value content for the eight resolver-bound ψ-Term variants (ADR-035): Nerve, ChainComplex, HomologyGroups, CochainComplex, CohomologyGroups, PostnikovTower, HomotopyGroups, KInvariants. Default impl: `NullResolverTuple` (every accessor returns a Null* resolver whose `resolve` emits `RESOLVER_ABSENT`). Custom impls via the [`resolver!`](uor-foundation-sdk/src/lib.rs) SDK macro. The eight per-category traits (`NerveResolver<H: Hasher>`, etc.) and eight marker traits (`HasNerveResolver<H>`, etc.) carry the typed accessor surface; `MAX_RESOLVER_TUPLE_ARITY = 16` caps the per-application tuple width. |

### How `HostBounds` flows through the type system (stable Rust 1.83)

ADR-018 mandates that "every signature in `prism`'s and `uor-foundation`'s public API that admits a value with a capacity-bounded width parameterizes that width through the application's selected `HostBounds`." The wiki's example pattern `[u8; H::FINGERPRINT_MAX_BYTES]` requires nightly `generic_const_exprs`. On stable Rust 1.83 (the workspace MSRV, matching the sibling [`UOR-Foundation/prism`](https://github.com/UOR-Foundation/prism) repo), the equivalent is **min-const-generics**: capacity-bearing types carry a `<const N: usize>` parameter, and applications populate it with `<MyBounds as HostBounds>::CONST` at instantiation sites.

The carriers:

| Type | Const-generic | Default | Sourced from |
|---|---|---|---|
| `Hasher<const FP_MAX: usize = 32>` | fingerprint output buffer width | 32 | `<DefaultHostBounds as HostBounds>::FINGERPRINT_MAX_BYTES` |
| `ContentFingerprint<const FP_MAX: usize = 32>` | inline fingerprint buffer width | 32 | same |
| `Trace<const TR_MAX: usize = 256>` | inline event-count ceiling | 256 | `<DefaultHostBounds as HostBounds>::TRACE_MAX_EVENTS` |

Applications using `DefaultHostBounds` reach these types under their default const-generic and never write turbofish. Applications selecting a different `HostBounds` impl write e.g. `ContentFingerprint::<64>` or `Trace::<1024>` and the type system propagates. Capacity-bearing functions (`Derivation::replay`, `replay::certify_from_trace`, `unit_address_from_buffer`, the `__test_helpers::trace_*` ctors) carry the matching const-generic on the function itself; type-annotated bindings or turbofish populate the parameter from the application's `HostBounds`.

`TRACE_REPLAY_FORMAT_VERSION` stays foundation-fixed (wiki ADR-018 carve-out for wire-format identifiers — cross-implementation interop requires a single shared value).

There are no free-standing `FINGERPRINT_MIN_BYTES` / `FINGERPRINT_MAX_BYTES` / `TRACE_MAX_EVENTS` constants on `uor-foundation`'s public surface — collapsing the substitution axis is exactly what ADR-018's "Rejected alternative 1" rules out. Applications and downstream crates (including [`uor-prism`](https://github.com/UOR-Foundation/prism)) read capacities through `<MyBounds as HostBounds>::CONST`.

## Categorical structure (wiki ADR-019)

`uor-foundation`'s vocabulary is the **signature category** of Prism's typed routes. The vocabulary names the structure explicitly:

| Concept | Realization |
|---|---|
| Signature endofunctor F | The twenty-one [`enforcement::Term`](foundation/src/enforcement.rs) variants — `Literal`, `Application`, `Lift`, `Project`, `Variable`, `Match`, `Recurse`, `Unfold`, `Try`, `AxisInvocation` (ADR-030, replaces the legacy `HasherProjection`), `ProjectField` (ADR-033), `FirstAdmit` (ADR-034), plus the nine ψ-chain variants `Nerve`, `ChainComplex`, `HomologyGroups`, `Betti`, `CochainComplex`, `CohomologyGroups`, `PostnikovTower`, `HomotopyGroups`, `KInvariants` (ADR-035) |
| Initial algebra of F | [`enforcement::Term`](foundation/src/enforcement.rs) itself — the free term language F generates |
| Catamorphism into the runtime carrier | [`pipeline::run`](foundation/src/pipeline.rs) — unique homomorphism induced by initiality |
| Anamorphism's witness object | `Trace` — produced by [`enforcement::Derivation::replay`](foundation/src/enforcement.rs) and consumed by [`enforcement::replay::certify_from_trace`](foundation/src/enforcement.rs) |
| Fixed points of the typed pipeline endofunctor | The four UOR-domain sealed types (`Datum`, `Triad`, `Derivation`, `FreeRank`) and three Prism-mechanism sealed types (`Validated`, `Grounded`, `Certified`) |

Initiality and uniqueness of the catamorphism hold *within each fixed choice of the three substitution axes* (`HostTypes`, `HostBounds`, `Hasher`). ADR-018's capacity-completeness — "the indexing of carriers is total over `HostBounds`" — is the categorical statement that every capacity-bounded width is part of the index. Closure (ADR-013) and zero-cost runtime (TC-01) are two halves of the same theorem: closure is the precondition that makes F's signature complete; completeness lets the catamorphism be discharged at the application's compile time, with no runtime indirection.

## `PrismModel` — the application author's typed iso (wiki ADR-020 + ADR-022)

[`pipeline::PrismModel`](foundation/src/pipeline.rs) codifies the application author's typed-iso contract. ADR-022 D4 parameterizes the trait over the three substitution axes (the H-indexed family of carriers, ADR-019 Consequences):

```rust
pub trait PrismModel<H, B, A>: __sdk_seal::Sealed
where
    H: HostTypes,
    B: HostBounds,
    A: Hasher,
{
    type Input: ConstrainedTypeShape;
    type Output: ConstrainedTypeShape + GroundedShape;
    type Route: FoundationClosed;
    fn forward(input: Self::Input) -> Result<Grounded<Self::Output>, PipelineFailure>;
}
```

`Route` is a type-level witness of the term tree mapping `Input` to `Output`; the `FoundationClosed` bound enforces closure under foundation vocabulary at the application's compile time per UORassembly (TC-04, ADR-006).

ADR-022 D1: the seal is [`pipeline::__sdk_seal::Sealed`](foundation/src/pipeline.rs) — `#[doc(hidden)] pub mod __sdk_seal { pub trait Sealed {} }`. The doc-hidden naming-convention pair is the ecosystem-standard idiom for cross-crate-extensible-but-controlled traits; the [`prism_model!`](uor-foundation-sdk/src/lib.rs) macro from `uor-foundation-sdk` emits `impl __sdk_seal::Sealed for <Model>`, `impl __sdk_seal::Sealed for <RouteWitness>`, `impl FoundationClosed for <RouteWitness>`, and `impl PrismModel<H, B, A> for <Model>` together. Foundation sanctions the identity-route impl on `ConstrainedTypeInput` directly; non-trivial routes go through the macro.

ADR-022 D5: [`pipeline::run_route<H, B, A, M>(input)`](foundation/src/pipeline.rs) is the canonical catamorphism call-site. It reads the route's term arena via `<M::Route as FoundationClosed>::arena_slice()`, builds a `Validated<CompileUnit, FinalPhase>` from it (using the application's `<B as HostBounds>::WITT_LEVEL_MAX_BITS` to derive the Witt-level ceiling), and dispatches to `pipeline::run`. The macro-emitted `forward` body is exactly `run_route::<H, B, A, Self>(input)`. The lower-level [`pipeline::run`](foundation/src/pipeline.rs) remains for callers (test harnesses, conformance suites, alternative SDK surfaces) that construct the `CompileUnit` themselves.

ADR-022 D2: [`enforcement::TermArena<CAP>::from_slice`](foundation/src/enforcement.rs) is the const constructor the macro can emit (`const ROUTE: TermArena<CAP> = TermArena::from_slice(ROUTE_SLICE)`); on stable Rust where every `Route` is required to expose its term tree as a `&'static [Term]` slice, the macro emits `const ROUTE_TERMS_FOR_<MODEL>: &'static [Term] = &[…]` and the route witness's `FoundationClosed::arena_slice()` returns it. Either form is fully `const` and the catamorphism is monomorphized at the application's compile time.

ADR-022 D3: [`uor-foundation-sdk::prism_model!`](uor-foundation-sdk/src/lib.rs) accepts the closure-bodied form the wiki specifies as the maximally-Rust-native syntax:

```rust
prism_model! {
    pub struct MyModel;
    pub struct MyRoute;
    impl PrismModel<DefaultHostTypes, DefaultHostBounds, MyHasher> for MyModel {
        type Input  = ConstrainedTypeInput;
        type Output = ConstrainedTypeInput;
        type Route  = MyRoute;
        fn route(input: Self::Input) -> Self::Output {
            // closure body — Rust expression syntax parsed by the macro
            // into a Term tree at expansion time. Recognised forms:
            //   - integer literals  → Term::Literal
            //   - identifier `input` → Term::Variable { name_index: 0 }
            //   - lowercase PrimitiveOp calls → Term::Application:
            //       add / sub / mul / xor / and / or  (binary)
            //       neg / bnot / succ / pred           (unary)
            // Anything else is a closure violation per ADR-020 — the
            // macro emits a compile error pointing at the offending
            // call site.
            add(input, 1)
        }
    }
}
```

The body is consumed at macro time and never executes as Rust at runtime; the macro-time mapping produces both the term-tree slice (the value-level state) and the impl block (the type-level state). [Smoke tests in uor-foundation-sdk/tests/smoke.rs](uor-foundation-sdk/tests/smoke.rs) pin the macro's `add(2, 3)` and `succ(input)` paths.

`forward()` is the catamorphism into `pipeline::run_route`'s runtime carrier (per ADR-019); together with the trace-witnessed anamorphism through [`enforcement::replay::certify_from_trace`](foundation/src/enforcement.rs) it forms the verifiable round-trip ADR-021 names as a normative architectural property.

## `IntoBindingValue` — runtime input flow (wiki ADR-023)

ADR-023 closes the architectural gap ADR-022 left open: how does an `M::Input` value supplied at runtime flow into the `CompileUnit` binding table? Foundation declares [`pipeline::IntoBindingValue`](foundation/src/pipeline.rs) as the trait every `M::Input` MUST implement. `pipeline::run_route` calls `into_binding_bytes` to fill a stack buffer (sized by the foundation-fixed [`pipeline::ROUTE_INPUT_BUFFER_BYTES`](foundation/src/pipeline.rs) ceiling), hashes the result through the application's selected `Hasher`, and constructs a transient `Binding` for the route's input slot (`Term::Variable { name_index: 0 }` per ADR-022 D3 G2).

The trait surface:

```rust
pub trait IntoBindingValue: ConstrainedTypeShape + __sdk_seal::Sealed {
    const MAX_BYTES: usize;
    fn into_binding_bytes(&self, out: &mut [u8]) -> Result<usize, ShapeViolation>;
}
```

- The seal supertrait is the same `__sdk_seal::Sealed` that gates `FoundationClosed` and `PrismModel` (ADR-022 D1) — only foundation and the SDK shape macros emit impls.
- Foundation sanctions the identity-route impl on `ConstrainedTypeInput` directly (`MAX_BYTES = 0`, `into_binding_bytes` returns `Ok(0)`).
- The SDK shape macros (`product_shape!`, `coproduct_shape!`, `cartesian_product_shape!`) emit the `IntoBindingValue` impl alongside the `ConstrainedTypeShape` impl, so application authors using shape macros write nothing.

`PrismModel::Input` carries the bound: `type Input: ConstrainedTypeShape + IntoBindingValue`. ADR-018's substitution-axis discipline carries through here too — `ROUTE_INPUT_BUFFER_BYTES` is the stable-Rust 1.83 equivalent of nightly's `[u8; <T as IntoBindingValue>::MAX_BYTES]` form: stable Rust cannot size a buffer with a generic associated constant (that requires `generic_const_exprs`), so the foundation-fixed ceiling caps the stack buffer; inputs declaring `MAX_BYTES > ROUTE_INPUT_BUFFER_BYTES` are rejected at runtime by `run_route`.

## Catamorphism evaluation + Output payload (wiki ADR-027 + ADR-028 + ADR-029)

The catamorphism `pipeline::run` actually evaluates the route's Term tree (ADR-029) — not just validates the CompileUnit metadata. [`pipeline::evaluate_term_tree`](foundation/src/pipeline.rs) walks the arena per the per-variant fold-rules and produces a `TermValue` (fixed-capacity byte buffer, ceiling [`pipeline::TERM_VALUE_MAX_BYTES`](foundation/src/pipeline.rs) = 4096 — sized to the max of `ROUTE_INPUT_BUFFER_BYTES` and `ROUTE_OUTPUT_BUFFER_BYTES` so a `TermValue` carries either an input-shaped value (ADR-023) or an evaluation result (ADR-028) without truncation). `pipeline::run_route` calls the evaluator, populates the `Grounded`'s output payload (ADR-028), and returns. The `Grounded<T>` carrier exposes [`output_bytes()`](foundation/src/enforcement.rs); the output buffer ceiling is [`pipeline::ROUTE_OUTPUT_BUFFER_BYTES`](foundation/src/pipeline.rs) = 4096 (parallel to the input ceiling from ADR-023).

`PrismModel::Output` is bound by `ConstrainedTypeShape + GroundedShape + IntoBindingValue`. `GroundedShape` is now sealed via the same `__sdk_seal::Sealed` supertrait foundation uses for `FoundationClosed`, `PrismModel`, and `IntoBindingValue` (ADR-027). The [`output_shape!`](uor-foundation-sdk/src/lib.rs) SDK macro is the sanctioned construction path: applications declaring custom Output shapes invoke it; the macro emits `__sdk_seal::Sealed`, `GroundedShape`, `IntoBindingValue`, and `ConstrainedTypeShape` together. The foundation-sanctioned identity output `ConstrainedTypeInput` retains its direct impl.

`Term::AxisInvocation { axis_index, kernel_id, input_index }` — the substitution-axis-realized verb form (ADR-029 + ADR-030, replaces the legacy `HasherProjection`): the catamorphism delegates evaluation to the application's selected axis at `axis_index` via the `kernel_id` selector. The foundation-canonical case (`axis_index = 0`, `kernel_id = 0`) folds the input bytes through `<A as Hasher>::initial().fold_bytes(...)` and emits `<A as Hasher>::finalize()`; user-defined axes declared via the `axis!` SDK macro extend the dispatch surface. The `prism_model!` macro emits the canonical case from the closure-body form `hash(input)` (ADR-026 G19).

`Term::ProjectField { source_index, byte_offset, byte_length }` — the eleventh Term variant (ADR-033 G20): byte-slice projection over a `partition_product`-shaped input. Lowered from the closure-body forms `<expr>.<index>` (positional) and `<expr>.<field_name>` (named); the proc-macro synthesizes a const-eval lookup against `<SourceTy as PartitionProductFields>::FIELDS[idx]` so the offset/length are computed by the trait impl at the consumer's compile time.

`Term::FirstAdmit { domain_size_index, predicate_index }` — the twelfth Term variant (ADR-034 Mechanism 2): bounded structural search with early termination. The catamorphism iterates `idx` ascending in `0..<DomainTy>::CYCLE_SIZE`, evaluating the predicate body with `FIRST_ADMIT_IDX_NAME_INDEX = u32::MAX - 4` bound to the current `idx` packed at the domain's byte width. The fold returns a coproduct value: `(0x01, idx_bytes)` on the first non-zero predicate result, or `(0x00, idx-width zero bytes)` if no idx admits. Lowered by `prism_model!`/`verb!` from the closure-body form `first_admit(<DomainTy>, |idx| <pred>)` (G16). ADR-034 Mechanism 1 also extends `Term::Recurse`: the two-parameter closure form `recurse(measure, base, |self, idx| step)` admits a fresh idx-ident the proc-macro lowers to `Term::Variable { name_index: RECURSE_IDX_NAME_INDEX = u32::MAX - 3 }`; the catamorphism's `Term::Recurse` fold-rule binds it to the current descent-measure value (the iteration counter).

## Three-layer algebraic closure (wiki ADR-024 + ADR-025 + ADR-026)

Per ADR-024, the architecture commits to three layers of algebraic closure, each with its own carrier, operator set, and closure check:

1. **Substrate closure** (`uor-foundation`): the `Term` enum's variants, the `PrimitiveOp` discriminants, the `ConstraintRef` variants, and the `WittLevel` ceiling. Operators per ADR-025: composer ops `×` (partition_product) and `+` (partition_coproduct), plus the endomorphism family `after_op` for `op ∈ Γ = {+, −, ×, ÷, ^}`.
2. **Prism closure** (route-level): the seven prism operators per ADR-026 — `compose`, `parallel_compose`, `fold_n`, `tree_fold`, `first_admit`, `partition_product`, `partition_coproduct` — plus the substitution-axis verb form (G19 `hash`), the field-access projection (G20 `<expr>.<index>` / `<expr>.<field_name>`, ADR-033), and the nine ψ-chain forms (G21–G29 per ADR-035): `nerve`, `chain_complex`, `homology_groups`, `betti`, `cochain_complex`, `cohomology_groups`, `postnikov_tower`, `homotopy_groups`, `k_invariants`. The closure-body grammar G1–G29 is the syntactic surface; the `prism_model!` macro recognizes the reserved identifiers and emits the corresponding Term variants. [`pipeline::FOLD_UNROLL_THRESHOLD`](foundation/src/pipeline.rs) = 8 fixes the `fold_n` unroll-vs-`Term::Recurse` lowering rule (ADR-026 G14). G13 `parallel(f, g)` lowers to `Term::Application(Or, [f, g])` — the partition-product structural-combine form per ADR-024's three-way responsibility split (foundation emits the structural declaration; implementations override the runtime). G15 `tree_fold(reducer, [a, b, c, …])` lowers to a pairwise reduction chain (depth `ceil(log2(n))`). G16 `first_admit(domain, |i| pred)` lowers to `Term::FirstAdmit` with the domain's `<DomainTy as ConstrainedTypeShape>::CYCLE_SIZE` (ADR-032) as descent measure and the predicate body bound via `FIRST_ADMIT_IDX_NAME_INDEX` (ADR-034). G17 `partition_product!` and G18 `partition_coproduct!` are the architectural-name SDK macros for type-level shape composition; on stable Rust 1.83 the in-position variadic form `partition_product!(<A>, <B>, …)` is the architecturally-equivalent named form `partition_product!(<Name>, <A>, <B>, [<C>, …])` because generic-const-expr-based per-pair `IRI`/`CONSTRAINTS` synthesis requires nightly. The named form preserves PT_3 / ST_10 canonical structure and emits all five sealed-trait impls (`__sdk_seal::Sealed`, `ConstrainedTypeShape`, `IntoBindingValue`, `GroundedShape`, `PartitionProductFields`). G20 (ADR-033) admits field access on `partition_product`-shaped inputs; the proc-macro synthesizes a const-eval lookup against `<SourceTy as PartitionProductFields>::FIELDS[idx]` for the byte offset and length. G21–G29 (ADR-035) admit the ψ-chain ψ_1..ψ_9 — eight resolver-bound forms consult the model declaration's `ResolverTuple` (ADR-036) at evaluation time; G24 `betti` is a resolver-free pure computation on resolved homology groups.
3. **Implementation closure** (verb-level): each implementation declares named, reusable compositions of prism operators applied to substrate primitives via the [`verb!`](uor-foundation-sdk/src/lib.rs) SDK macro. Cross-implementation imports proceed through the [`use_verbs!`](uor-foundation-sdk/src/lib.rs) macro. Verbs are structural declarations; their runtime is implementation-owned per the three-way responsibility split (substrate owns primitives, prism owns operators, implementation owns runtime).

## V&V framework alignment (wiki ADR-021)

ADR-021 names the four V&V Decisions Prism resolves under the hylomorphism framing:

| Decision | Resolution |
|---|---|
| 1. Context of Use | "UOR Framework as a production substrate for compiled prism applications, with the catamorphism + anamorphism pair providing internal round-trip verification." |
| 2. External validation referent | The published UOR Foundation mathematics (Witt-tower theory) governs spec faithfulness via Oberkampf-Roy + the [`lean4/`](lean4/) zero-`sorry` corpus. The trace-replay round-trip is the **internal** referent — a normative architectural property, not a test fixture. |
| 3. Independence (V vs IV&V) | Structural and built-in: `uor-foundation`'s pipeline is the V agent (catamorphism); [`uor-foundation-verify`](uor-foundation-verify/) is the IV&V agent (anamorphism via [`certify_from_trace`](foundation/src/enforcement.rs)). The trace is the artifact crossing the boundary. |
| 4. Integrity Level | Per consumer class: IL 1 (toy demos) → IL 3 (Bitcoin PoW substrate) → IL 3-4 (FHE) → IL 4 (safety-of-life, out of scope). Foundation floor is IL 3. |

The normative round-trip property is exercised by [`uor-foundation-verify/tests/round_trip.rs`](uor-foundation-verify/tests/round_trip.rs), whose head-comment explicitly names it as ADR-021's V&V Decision 2 instantiation. The eight wiki validators (V1–V8), the Lean 4 corpus, the conformance suite, and the V/IV&V agent split realize the framework directly — ADR-021 names them rather than introducing new mechanisms.

## Workspace layout

| Crate | Path | Published | Purpose |
|---|---|---|---|
| `uor-ontology` | `spec/` | no | Ontology source of truth (classes, properties, individuals, serializers) |
| `uor-codegen` | `codegen/` | no | Ontology-to-Rust trait generator |
| `uor-foundation` | `foundation/` | **crates.io** | Generated `#![no_std]` trait library — never edit manually |
| `uor-foundation-sdk` | `uor-foundation-sdk/` | **crates.io** (pending first release) | Procedural-macro ergonomics (`product_shape!`, `coproduct_shape!`, `cartesian_product_shape!`) for composing `ConstrainedTypeShape` operands — emitted by `uor-crate` from `codegen/src/sdk_macros.rs`. |
| `uor-foundation-verify` | `uor-foundation-verify/` | **crates.io** (pending) | Trace-replay verifier — thin façade re-exporting `certify_from_trace`, `Certified`, the wire-format types, and the `HostBounds` substitution axis. Wiki name: `prism-verify`. |
| `uor-conformance` | `conformance/` | no | Conformance suite (OWL, SHACL, RDF, Rust API, docs, website) — check count in `spec/src/counts.rs` |
| `uor-docs` | `docs/` | no | Documentation generator |
| `uor-website` | `website/` | no | Static site generator |
| `uor-lean-codegen` | `lean-codegen/` | no | Ontology-to-Lean 4 structure generator |
| `uor-clients` | `clients/` | no | CLI binaries: `uor-build`, `uor-crate`, `uor-lean`, `uor-docs`, `uor-website`, `uor-conformance` |
| `cargo-uor` | `cargo-uor/` | no | Cargo subcommand binary for UOR tooling |

## Critical rules

- **Never hand-edit `foundation/src/` or `lean4/`** — they are regenerated from `spec/` by `uor-crate` and `uor-lean`. CI enforces `git diff --exit-code` on both.
  - **Exception (Phase 11):** `foundation/src/blanket_impls.rs` is hand-written and starts with `// @codegen-exempt`. The codegen `emit::write_file` preserves files carrying that banner; the `rust/blanket_impls_exempt` conformance gate enforces both the banner and the required Path-3 blanket impls.
- **On release**, Lean 4 cloud release builds are uploaded via `lake upload`. Lean Reservoir indexes this repo directly (root `lakefile.lean` + `lake-manifest.json`).
- **All clippy warnings are errors.** CI runs `cargo clippy --all-targets -- -D warnings`.
- **Every crate denies:** `clippy::unwrap_used`, `clippy::expect_used`, `clippy::panic`, `missing_docs`, `clippy::missing_errors_doc`.
- **Formatting is enforced.** CI runs `cargo fmt --check`.
- **The conformance suite must pass.** `cargo run --bin uor-conformance` — zero failures allowed (check count in `spec/src/counts.rs`).
- **No `unsafe` code.** The `uor-foundation` crate is `#![no_std]` with zero dependencies.
- **Bracket-escape doc comments.** Use `normalize_comment()` to prevent rustdoc intra-doc link warnings on `[text]` in comments.

## Build commands

```sh
cargo fmt --check                    # Format check
cargo clippy --all-targets -- -D warnings  # Lint
cargo test                           # Unit + integration tests
cargo run --bin uor-crate            # Regenerate foundation/src/ from spec/
cargo run --bin uor-lean             # Regenerate lean4/ from spec/
cargo run --bin uor-build            # Emit JSON-LD, Turtle, N-Triples to public/
cargo run --bin uor-docs             # Generate documentation site
cargo run --bin uor-website          # Generate website
cargo run --bin uor-conformance      # Run full conformance suite
```

Docs/website/conformance binaries accept `PUBLIC_BASE_PATH` env var for URL prefixing.

## CI pipeline (in order)

`cargo fmt --check` → `cargo clippy` → `cargo test` → `cargo run --bin uor-crate` → `git diff --exit-code foundation/src/ uor-foundation-sdk/src/` → `cargo check -p uor-foundation --no-default-features` → `cargo publish --dry-run` (uor-foundation + uor-foundation-sdk) → `uor-lean` → `git diff --exit-code lean4/` → `uor-build` → `uor-docs` → `uor-website` → `uor-conformance` → deploy pages

## Ontology architecture

Counts below are mirrored from `spec/src/counts.rs`, which is the single source of truth.

- **34 namespaces**, assembly order: `u → schema → op → query → resolver → type → partition → foundation → observable → carry → homology → cohomology → proof → derivation → trace → cert → morphism → state → reduction → convergence → division → interaction → monoidal → operad → effect → predicate → parallel → stream → failure → linear → recursion → region → boundary → conformance`
- **Space classification:** Kernel (17: `u`, `schema`, `op`, `carry`, `reduction`, `convergence`, `division`, `monoidal`, `operad`, `effect`, `predicate`, `parallel`, `stream`, `failure`, `linear`, `recursion`, `region`), Bridge (14: `query`, `resolver`, `partition`, `foundation`, `observable`, `homology`, `cohomology`, `proof`, `derivation`, `trace`, `cert`, `interaction`, `boundary`, `conformance`), User (`type`, `morphism`, `state`)
- **471 classes** → 452 traits + 19 enum classes (includes WittLevel newtype struct)
- **948 properties** → 911 trait methods (generic over `P: Primitives`)
- **3559 named individuals** → 3546 constant modules
- **19 enum classes:** `AchievabilityStatus`, `ComplexityClass`, `ExecutionPolicyKind`, `GeometricCharacter`, `GroundingPhase`, `MeasurementUnit`, `MetricAxis`, `PartitionComponent`, `PhaseBoundaryType`, `ProofStrategy`, `QuantifierKind`, `RewriteRule`, `SessionBoundaryType`, `TriadProjection`, `ValidityScopeKind`, `VarianceAnnotation`, `VerificationDomain`, `ViolationKind`, `WittLevel`

## Code generation patterns

- All traits are generic over `P: Primitives` (no hardcoded XSD types)
- Enum classes are detected by `detect_vocabulary_enum()` and skip trait generation; WittLevel is a struct (not enum) but also skips trait generation
- `object_property_enum_override()` maps ObjectProperties to enum/struct return types (delegates to `enum_class_names()`)
- Multi-value IriRef properties on individuals → `&[&str]` slices via `BTreeMap` grouping
- `RustFile::finish()` trims trailing whitespace to match `cargo fmt`
- Module declarations in `mod.rs` are sorted alphabetically
- Cross-namespace domain properties and enum-class domain properties are not generated

## Lean 4 code generation patterns

- All structures are parametric over `(P : Primitives)` — mirrors the Rust `<P: Primitives>` generic
- OWL classes → `structure` (not `class`); only `Primitives` uses `class` (genuine typeclass)
- Enum classes → `inductive` with `deriving DecidableEq, Repr, BEq, Hashable, Inhabited`
- WittLevel → `structure` (open-world, not `inductive`)
- Self-referential properties → `Option` wrapping for functional, `Array` for non-functional
- Inheritance → `extends ParentA P, ParentB P`; cross-namespace uses qualified `UOR.Space.Module.ClassName P`
- Non-functional properties → `Array` type (idiomatic Lean 4)
- Lean keyword escaping → guillemets `«keyword»` (e.g., `«type»`)
- Individual constants → `namespace name ... end name` blocks with `def` constants
- Cross-namespace domain properties are NOT generated (same rule as Rust codegen)
- Import DAG follows the ontology assembly order (acyclic)
- `autoImplicit = false` in lakefile prevents implicit variable surprises

## Conformance categories

1. **Rust source** — formatting, line width, public API surface
2. **Ontology inventory** — exact namespace/class/property/individual counts
3. **JSON-LD 1.1** — `@context`, `@graph`, non-functional property arrays
4. **OWL 2 DL** — disjointness, functionality, domain/range constraints
5. **RDF / Turtle** — serialization format, prefixes, IRIs
6. **SHACL** — shapes (1:1 with classes), instance test graphs (counts in `spec/src/counts.rs`)
7. **Generated crate** — trait/method/enum/constant counts, `#![no_std]` build
8. **Documentation + Website** — completeness, accessibility, broken links
9. **Lean 4 formalization** — structure/field/enum/individual completeness, sorry audit

## Centralized counts

All inventory counts are in **`spec/src/counts.rs`** — the single file to update when ontology terms change. All crates import from `uor_ontology::counts`. Enum class names are centralized in `Ontology::enum_class_names()` in `spec/src/model.rs`. The version string is auto-derived from `Cargo.toml` via `env!("CARGO_PKG_VERSION")`.

## Editing workflow

1. Modify the ontology in `spec/src/namespaces/`
2. Update counts in `spec/src/counts.rs` (single file)
3. Run `cargo run --bin uor-crate` to regenerate `foundation/src/`
4. Run `cargo fmt`
5. Run `cargo clippy --all-targets -- -D warnings`
6. Run `cargo test`
7. Run `cargo run --bin uor-conformance` (full validation)

## Release process

See `RELEASING.md`. Summary: bump version in root `Cargo.toml`, regenerate, commit, tag `vX.Y.Z`, push. CI publishes to crates.io and GitHub Pages.

## Toolchain

- Rust stable (edition 2021, MSRV 1.81 — bumped from 1.70 in v0.2.2 Tier 5 to unlock `core::error::Error` on `no_std`)
- Components: `rustfmt`, `clippy`
- `clippy.toml`: `too-many-lines-threshold = 100`, `avoid-breaking-exported-api = false`
- License: MIT
