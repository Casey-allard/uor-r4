//! Authoritative ontology inventory counts.
//!
//! **This is the single file to update when ontology terms change.**
//! All crates import from here. The spec crate's tests verify that
//! [`Ontology::full()`](crate::Ontology::full) produces exactly these counts.

/// Number of namespace modules.
///
/// Product/Coproduct Completion Amendment: +1 (foundation — layout
/// invariants complementing op-namespace theorems).
pub const NAMESPACES: usize = 34;

/// Total OWL classes across all namespaces.
///
/// v0.2.1 Phase 1: +13 (5 verdict classes for Inhabitance — InhabitanceCertificate,
/// InhabitanceImpossibilityWitness, InhabitanceSearchTrace, InhabitanceStep,
/// InhabitanceCheckpoint; 4 resolver subclasses — InhabitanceResolver,
/// TwoSatDecider, HornSatDecider, ResidualVerdictResolver; 1 schema:ValueTuple;
/// 1 reduction:FailureField; 1 resolver:CertifyMapping; 1 conformance:PreludeExport).
/// v0.2.1 Phase 7a: +3 (reduction:SatBound, reduction:TimingBound,
/// type:ConstraintDefaults — parametric metadata for codegen).
/// v0.2.2 Phase A: +1 (observable:LandauerBudget — sealed carrier for accumulated
/// Landauer cost; backs the Rust enforcement::LandauerBudget newtype that holds
/// one of the two clocks of UorTime).
/// v0.2.2 Phase C.4: +2 (cert:MultiplicationCertificate, resolver:MultiplicationResolver).
/// v0.2.2 Phase D (Q4): net 0 (-7 enumerated Constraint subclasses deleted:
/// ResidueConstraint, CarryConstraint, DepthConstraint, CompositeConstraint,
/// HammingConstraint, SiteConstraint, AffineConstraint; +3 parametric
/// classes: BoundConstraint, BoundShape, Conjunction; +4 observable
/// subclasses: observable:ValueModObservable, derivation:DerivationDepthObservable,
/// carry:CarryDepthObservable, partition:FreeRankObservable).
/// v0.2.2 Phase E: +5 (cert:PartitionCertificate, partition:PartitionComponent,
/// observable:GroundingSigma, observable:JacobianObservable,
/// derivation:DerivationTrace).
/// v0.2.2 T1.2 (cleanup): +1 (conformance:InteractionShape — backing class
/// for InteractionDeclarationBuilder, originally planned in Phase E).
/// Workstream C (v0.2.2 closure): +2 (cert:GenericImpossibilityCertificate
/// and cert:InhabitanceImpossibilityCertificate — the cert carriers for
/// resolver failure paths per target §4.2 `Certified<ImpossibilityWitness>`).
/// Product/Coproduct Completion Amendment: +3 (partition:CartesianPartitionProduct,
/// partition:TagSite, foundation:LayoutInvariant).
/// Wiki ADR-038: +1 (observable:AxisProjectionObservable — top-level
/// Observable subclass parallel to the seven internally-derived
/// categories, carrying axis-realized projection values from the
/// substrate-extension surface per ADR-030).
/// Wiki ADR-049: +1 (observable:SpectralObservable — top-level
/// Observable subclass hosting Walsh–Hadamard-parity-derived spectral
/// readings of the σ-projection's frequency-domain spectrum, parallel
/// to the seven internally-derived categories).
/// Wiki ADR-040 + ADR-049 catalog correspondence amendment: +1
/// (observable:ValueThresholdObservable — top-level Observable subclass
/// hosting byte-sequence threshold comparison readings of digests,
/// realizing the type:LexicographicLessEqBound bound-shape primitive's
/// dispatch path; foundation's LexicographicLessEqThreshold observable
/// falls under this subclass; canonical search-cost commitment alias
/// TargetCommitment = SingletonCommitment<LexicographicLessEqThreshold>
/// per ADR-048 consumes it).
pub const CLASSES: usize = 474;

/// Total properties including the global `uor:space` annotation.
///
/// v0.2.1 Phase 1: +31.
/// v0.2.1 Phase 7a: +7 (3 SatBound: maxVarCount/maxClauseCount/maxLiteralsPerClause;
/// 2 TimingBound: preflightBudgetNs/runtimeBudgetNs; 1 type:defaultValue;
/// 1 op:isRingOp — op:arity already existed as xsd:nonNegativeInteger).
/// v0.2.2 W8: +4 (schema:triadStratum, schema:triadSpectrum, schema:triadAddress,
/// state:groundedTriad — Triad bundling).
/// v0.2.2 Phase A: +1 (observable:landauerNats — accumulated Landauer cost on
/// LandauerBudget, unit observable:Nats).
/// v0.2.2 Phase C.4: +4 (cert:splittingFactor, cert:subMultiplicationCount,
/// cert:landauerCostNats, linear:stackBudgetBytes).
/// v0.2.2 Phase D (Q4): +4 (type:boundObservable, type:boundShape,
/// type:boundArguments, type:conjuncts).
/// v0.2.2 Phase E: +1 (derivation:traceEventCount).
/// Target §3 Sink/Sinking hardening: −1 (boundary:sinkProjection removed;
/// replaced by the `Sinking::ProjectionMap` Rust-side discipline that
/// carries the kind discriminator at the type level).
/// Target §3 inbound symmetric cleanup: −1 (boundary:sourceGrounding
/// removed; replaced by the `Grounding::Map` Rust-side discipline).
/// Product/Coproduct Completion Amendment: +8
/// (partition:leftCartesianFactor, partition:rightCartesianFactor,
/// partition:tagSiteOf, partition:tagValue, partition:productCategoryLevel,
/// foundation:layoutRule, type:variant, type:tagSite).
pub const PROPERTIES: usize = 948;

/// Namespace-level properties only (excludes global annotation).
pub const NAMESPACE_PROPERTIES: usize = 947;

/// Total named individuals across all namespaces.
/// Includes 1870 AST term individuals (LiteralExpression / ForAllDeclaration)
/// generated from identity lhs/rhs/forAll string values.
///
/// v0.2.1 Phase 1: +76.
/// v0.2.1 Phase 7a: +5 (TwoSatBound, HornSatBound, PreflightTimingBound,
/// RuntimeTimingBound, ResidueDefaultModulus).
/// v0.2.2 W4+W14: +5 (morphism:DigestGroundingMap, morphism:BinaryGroundingMap,
/// reduction:ShapeMismatch, two reduction:FailureField individuals for
/// ShapeMismatch's `expected` and `got` fields).
/// v0.2.2 Phase C.1: +4 (schema:W40, schema:W48, schema:W56, schema:W64 —
/// dense u64-backed Witt levels).
/// v0.2.2 Phase C.2: +8 (schema:W72, W80, W88, W96, W104, W112, W120, W128 —
/// dense u128-backed Witt levels).
/// v0.2.2 Phase C.3: +16 (schema:W160, W192, W224, W256, W384, W448, W512,
/// W520, W528, W1024, W2048, W4096, W8192, W12288, W16384, W32768 —
/// Limbs<N>-backed Witt levels covering semantically-meaningful intermediates
/// and powers-of-two above native).
/// v0.2.2 Phase C.4: +1 (resolver:multiplicationCertifyMapping).
/// v0.2.2 Phase D (Q4): +12 (6 BoundShape individuals: EqualBound, LessEqBound,
/// GreaterEqBound, RangeContainBound, ResidueClassBound, AffineEqualBound;
/// 6 BoundConstraint kind individuals: residue/hamming/depth/carry/site/affine
/// ConstraintKind).
/// v0.2.2 Phase E: +4 (partition:PartitionComponent individuals:
/// Irreducible, Reducible, Units, Exterior).
/// Target §3 Sink/Sinking hardening: +2 (morphism:DigestProjectionMap,
/// morphism:BinaryProjectionMap — kind-parity with GroundingMap duals).
/// Product/Coproduct Completion Amendment: +59. 11 new op:Identity
/// individuals (ST_6..ST_10, CPT_1..CPT_6); 11 new proof:AxiomaticDerivation
/// individuals (prf_ST_6..10, prf_CPT_1..6) — one per new op:Identity to
/// satisfy the identity-proof bijection validator; 4 new
/// foundation:LayoutInvariant individuals (ProductLayoutWidth,
/// CartesianLayoutWidth, CoproductLayoutWidth, CoproductTagEncoding); 33
/// derived AST-term individuals — each new op:Identity has exactly 3 Str
/// properties (lhs, rhs, forAll) which `schema::generate_ast_individuals`
/// expands into LiteralExpression / ForAllDeclaration individuals in the
/// schema namespace. Proof individuals carry no lhs/rhs/forAll strings so
/// they contribute no derived AST terms.
/// Wiki ADR-040: +1 (type:LexicographicLessEqBound — the 7th BoundShape
/// individual, the byte-sequence-valued comparison primitive paralleling
/// LessEqBound under canonical big-endian unsigned ordering).
/// Wiki ADR-053: +41 — six new PrimitiveOp/GeometricCharacter individuals
/// (op:div, op:mod, op:pow, op:Quotient, op:Remainder, op:IteratedScaling),
/// seven new op:Identity individuals (DV_1..4, PW_1..3) governing the
/// ring-axis completion Γ = {Add, Sub, Mul, Div, Mod, Pow}, seven matching
/// proof:AxiomaticDerivation individuals (prf_DV_1..4, prf_PW_1..3), and
/// twenty-one derived AST-term individuals (three lhs/rhs/forAll strings
/// per new op:Identity, expanded by schema::generate_ast_individuals).
pub const INDIVIDUALS: usize = 3601;

/// Number of SHACL test instance graphs.
///
/// v0.2.1 Phase 7a: +1 (test278 SatBound/TimingBound/ConstraintDefaults fixture).
/// v0.2.2 Phase C.4: +1 (test279 MultiplicationCertificate +
/// MultiplicationResolver + linear:stackBudgetBytes fixture).
/// v0.2.2 Phase E: +1 (test280 Phase E bridge namespace completion fixture).
/// Product/Coproduct Completion Amendment: +3 (test285 CartesianPartitionProduct,
/// test286 TagSite, test287 LayoutInvariant — instance fixtures for the three
/// new kernel/bridge classes; close the meta/shacl_fixture_coverage gaps).
/// Wiki ADR-038: +1 (test288 AxisProjectionObservable — instance fixture
/// for the closed-catalog extension's Observable subclass).
/// Wiki ADR-049: +1 (test289 SpectralObservable — instance fixture for
/// the Walsh–Hadamard-parity-derived spectral subclass).
pub const SHACL_TESTS: usize = 285;

/// Total conformance checks in the full suite.
///
/// v0.2.1 Phase 1: +1 from the test277 SHACL fixture.
/// v0.2.1 Phase 7a: +1 from test278 SatBound/TimingBound/ConstraintDefaults
/// fixture. v0.2.1 Phase 7g: +1 from the `lean4/rigor` banned-primitives
/// enforcement check.
/// v0.2.2 W5: +1 from the `docs/psi_leakage` validator.
/// v0.2.2 W6: +1 from the `rust/public_api_snapshot` validator.
/// v0.2.2 Phase A: +1 from the `rust/uor_time_surface` validator.
/// v0.2.2 Phase B: +1 from the `rust/phantom_tag` validator.
/// v0.2.2 Phase C.4: +1 from the `test279` MultiplicationCertificate fixture.
/// v0.2.2 Phase C verifiers: +1 from `rust/witt_tower_completeness`, +1 from
/// `rust/multiplication_resolver`.
/// v0.2.2 Phase D verifier: +1 from `rust/parametric_constraints`.
/// v0.2.2 Phase E: +1 from `rust/bridge_namespace_completion`, +1 from
/// `test280_bridge_completion` SHACL fixture.
/// v0.2.2 Phase F: +1 from `rust/driver_shape`.
/// v0.2.2 Phase G: +1 from `rust/const_fn_frontier`.
/// v0.2.2 Phase J: +1 from `rust/grounding_combinator_check`.
/// v0.2.2 Phase H: +6 from `rust/feature_flag_layout`,
/// `rust/escape_hatch_lint`, `rust/no_std_build_check`,
/// `rust/alloc_build_check`, `rust/all_features_build_check`,
/// `rust/uor_foundation_verify_build`.
/// v0.2.2 T1.5 (cleanup): +1 from `docs/concept_pages_count`.
/// v0.2.2 T2.0 (cleanup): +2 from `rust/public_api_functional`
/// (foundation_e2e + verify_round_trip).
/// v0.2.2 T2.3 (cleanup): +1 from `rust/ebnf_constraint_decl`.
/// v0.2.2 T6: +5 from `rust/calibration_presets_valid`,
/// `rust/pipeline_run_threads_input`, `rust/verify_trace_round_trip`,
/// `rust/trace_byte_layout_pinned`, `rust/error_trait_completeness`.
/// Phase A: +4 from sealed BaseMetric newtypes + Stratum + accessor anchors.
/// Phase B: -4 from deleted Primitives checks; Phase H: +1 libm validator;
/// Phase J.5: +1 no_std public-API snapshot companion.
/// Phase G: +1 grammar-surface coverage validator; Phase D: +1 resolver tower.
/// Phase E: +1 bridge enforcement validator; Phase F: +1 kernel enforcement;
/// Phase K: +1 W4 closure validator; Phase L: +1 const-ring-eval coverage;
/// Phase M: +1 driver must-use discipline.
/// Correctness suite: +14 Layer-2 behavioral tests + 1 test_assertion_depth + 1 endpoint_coverage.
/// Workstream A: +5 target-doc cross-reference validators (sealed_type_coverage,
/// resolver_signature_shape, constraint_encoder_completeness, w4_grounding_closure,
/// spectral_sequence_walk).
/// Phase 6 (orphan-closure): +1 `rust/theory_deferred_register` —
/// bijection between Path-4 classifications and `docs/theory_deferred.md`
/// rows.
/// Phase 7e (orphan-closure): +1 `rust/orphan_counts` — minimum-viable
/// orphan-count validator; greps `impl {Name}<H> for ...` sites.
/// Phase 9d (orphan-closure): +1 `rust/no_hardcoded_f64` — gate that
/// ensures foundation/src/ has zero hardcoded `f64` outside test code.
/// Phase 9e (orphan-closure): +1 `rust/host_types_discipline` — asserts
/// `HostTypes::Decimal: DecimalTranscendental` + libm impls.
/// Phase 10 (orphan-closure): +1 `rust/witness_scaffold_surface` — asserts
/// every Path-2 class has a `Mint{Foo}` + `Mint{Foo}Inputs<H>` +
/// `Certificate` + `OntologyVerifiedMint` scaffold in
/// `foundation/src/witness_scaffolds.rs` and a per-family primitive
/// stub module under `foundation/src/primitives/`.
/// Phase 11c (orphan-closure): +1 `rust/blanket_impls_exempt` — asserts
/// `foundation/src/blanket_impls.rs` exists, carries the
/// `// @codegen-exempt` banner, and contains every Path-3-allow-listed
/// blanket impl (Observable/ThermoObservable supertraits + 5 leaf
/// traits) on `Validated<T, Phase>`.
/// Phase 12 (orphan-closure): +1 `rust/phase12_no_stubs` — asserts no
/// `WITNESS_UNIMPLEMENTED_STUB:*` markers remain in
/// `foundation/src/primitives/*.rs`; every verify_* returns
/// `Ok(witness)` or a typed `GenericImpossibilityWitness`.
/// Phase 13a (orphan-closure): existing `rust/orphan_counts` validator
/// upgrades from minimum-viable to classifier-integrated; reports
/// per-category breakdown (null_stub / resolved_wrapper /
/// validated_blanket / verified_mint / hand_written) and asserts every
/// `Path{N}` classification matches its expected impl shape. No new
/// check added — the existing validator is upgraded in place.
/// Phase 13c (orphan-closure): +1 `rust/taxonomy_coverage` — asserts
/// the Phase 0 classification report at
/// `docs/orphan-closure/classification_report.md` agrees with
/// `classify_all` and that the `CLASSIFICATION_*` constants in this
/// file match the live counts.
/// ADR-059: +1 `ontology/inventory/convergence_codomain_stratification`
/// — pins the four-level Hopf convergence tower {R,C,H,O} at division-algebra
/// dimensions {1,2,4,8} as the operator-geometry codomain stratification of
/// κ-derivation (ADR-058), guarding the characteristic identities, Hopf fiber
/// spheres, and persistent residual Betti signatures the ADR makes normative.
pub const CONFORMANCE_CHECKS: usize = 546;

/// Number of amendments applied to the base ontology.
pub const AMENDMENTS: usize = 95;

/// Number of classes that become Rust enums/structs (not traits).
pub const ENUM_CLASSES: usize = 19;

/// Number of `op:Identity` individuals (and corresponding proofs).
///
/// Product/Coproduct Completion Amendment: +11 (ST_6..ST_10, CPT_1..CPT_6).
/// Wiki ADR-053: +7 (DV_1..4, PW_1..3) — Euclidean-division and modular-
/// exponentiation identities governing the new ring-axis completion ops.
pub const IDENTITY_COUNT: usize = 642;

/// Kernel-space namespace count.
pub const KERNEL_NAMESPACES: usize = 17;

/// Bridge-space namespace count.
///
/// Product/Coproduct Completion Amendment: +1 (foundation namespace,
/// classified as Space::Bridge — produced by the foundation layer,
/// consumed by downstream witness consumers).
pub const BRIDGE_NAMESPACES: usize = 14;

/// User-space namespace count.
pub const USER_NAMESPACES: usize = 3;

/// Number of trait methods generated (properties with domains,
/// excluding enum-class-domain and cross-namespace-domain properties).
///
/// v0.2.1 Phase 1: +31. Phase 7a: +7 from new parametric metadata properties.
/// v0.2.2 W8: +4 (triadStratum, triadSpectrum, triadAddress on schema:Triad;
/// groundedTriad on state:GroundedContext).
/// v0.2.2 Phase A: +1 (landauerNats on observable:LandauerBudget).
/// v0.2.2 Phase C.4: +4 (splittingFactor, subMultiplicationCount,
/// landauerCostNats on MultiplicationCertificate; stackBudgetBytes on LinearBudget).
/// v0.2.2 Phase D (Q4): +4 (boundObservable, boundShape, boundArguments on
/// BoundConstraint; conjuncts on Conjunction). The 11 properties previously
/// on the 7 deleted constraint subclasses are retained under new domains
/// (BoundConstraint or Conjunction), so no net loss.
/// v0.2.2 Phase E: +1 (derivation:traceEventCount on DerivationTrace).
/// Target §3 Sink/Sinking hardening: −1 (boundary:sinkProjection removed).
/// Target §3 inbound symmetric cleanup: −1 (boundary:sourceGrounding removed).
/// Product/Coproduct Completion Amendment: +8 — all 8 new properties
/// (leftCartesianFactor, rightCartesianFactor, tagSiteOf, tagValue,
/// productCategoryLevel, layoutRule, variant, tagSite) have
/// same-namespace domains and none target enum-class domains, so each
/// expands into a trait method. Actual value 911 matches the
/// `ontology/crate` conformance validator's count after cleanup.
pub const METHODS: usize = 911;

/// Number of individual constant modules generated.
///
/// Counts `pub mod foo { ... }` blocks emitted per non-enum-class
/// individual by the Rust codegen (codegen/src/individuals.rs). The
/// pre-amendment value 1501 had significant historical drift and was
/// docs-only (no conformance validator). Product/Coproduct Completion
/// Amendment corrects this to match actual codegen output: 3541 =
/// baseline 3482 + 59 amendment additions (11 op:Identity + 11 proof
/// + 4 foundation:LayoutInvariant + 33 derived AST-term individuals).
pub const CONSTANT_MODULES: usize = 3541;

/// Number of Lean 4 structures generated (classes minus enum classes).
///
/// v0.2.1 Phase 1: +13. Phase 7a: +3 (SatBound, TimingBound, ConstraintDefaults).
/// v0.2.2 Phase C.4: +2 (MultiplicationCertificate, MultiplicationResolver).
/// v0.2.2 Phase E: +4 (PartitionCertificate, GroundingSigma, JacobianObservable,
/// DerivationTrace; PartitionComponent is an enum class, not a structure).
/// v0.2.2 T1.2 (cleanup): +1 (InteractionShape — regular structure).
/// Product/Coproduct Completion Amendment: +3 new structures
/// (CartesianPartitionProduct, TagSite, LayoutInvariant — none are enum
/// classes). Actual value 452 reflects a 14-structure baseline drift
/// present in the pre-amendment counts.rs (452 = 449 baseline + 3 new),
/// corrected during amendment.
pub const LEAN_STRUCTURES: usize = 452;

/// Number of Lean 4 inductive + structure types generated for the enum layer.
///
/// Composition: 18 ontology enum classes (see `Ontology::enum_class_names()`),
/// plus 3 hardcoded types not in the ontology's class list (`Space`,
/// `SiteState`, `PrimitiveOp`), plus 1 `structure` for `WittLevel` (open-world,
/// not an `inductive`). Total: 22.
pub const LEAN_INDUCTIVES: usize = 23;

/// Number of Lean 4 individual constant namespaces generated.
///
/// One `namespace <name> ... end <name>` block is emitted per non-enum
/// named individual in the ontology. This is distinct from
/// `CONSTANT_MODULES`, which counts the per-namespace-module constant
/// files produced by the Rust codegen — those are container modules,
/// not per-individual namespace blocks.
///
/// v0.2.2 W4+W14: +5 (DigestGroundingMap, BinaryGroundingMap, ShapeMismatch,
/// shapeMismatch_expected_field, shapeMismatch_got_field).
/// v0.2.2 Phase C.4: +1 (multiplicationCertifyMapping — a resolver individual,
/// not a WittLevel, so it gets a namespace block like the other CertifyMappings).
///
/// **Note**: WittLevel individuals (Phase C.1+) are NOT counted here. WittLevel
/// is in `enum_class_names()` and its individuals are emitted as `def Wn` in
/// `lean4/UOR/Enums.lean`, not as `namespace ... end` blocks. They contribute
/// to the WittLevel def list (visible in `Enums.lean`) but not to the
/// per-individual constant namespace count.
///
/// Target §3 Sink/Sinking hardening: +2
/// (morphism:DigestProjectionMap, morphism:BinaryProjectionMap).
/// Product/Coproduct Completion Amendment: +59 — 11 new op:Identity
/// individuals + 11 new proof individuals + 4 new
/// foundation:LayoutInvariant individuals + 33 derived AST-term
/// individuals. Actual value 3422 matches the
/// `lean4/structure — Individual def count drift` validator's typed-def
/// count after regeneration.
/// Wiki ADR-040: +1 (type:LexicographicLessEqBound — 7th BoundShape
/// individual emitted by Lean codegen alongside the other six).
/// Wiki ADR-053: +35 — 6 PrimitiveOp + GeometricCharacter (op:div, op:mod,
/// op:pow, op:Quotient, op:Remainder, op:IteratedScaling) + 7 op:Identity
/// (DV_1..4, PW_1..3) + 7 proof:AxiomaticDerivation (prf_DV_1..4,
/// prf_PW_1..3) + 15 derived AST-term namespace blocks (lhs/rhs/forAll
/// expansions per the new identities; ring-axis identity exprs reuse
/// existing `add`/`mul`/`pow`/`div`/`mod`/literal AST nodes for 6 of 21
/// terms, leaving 15 fresh AST individuals).
pub const LEAN_CONSTANT_NAMESPACES: usize = 3458;

/// Number of concept pages on the website (one per content/concepts/*.md file).
/// Number of concept pages on the website (one per `website/content/concepts/*.md`,
/// excluding `prism.md` which is merged into the pipeline page).
///
/// v0.2.2 T1.5 (cleanup): corrected 27 → 12. The previous value (27) did not
/// match either `website/content/concepts/` (12 files after excluding
/// `prism.md`) or `docs/content/concepts/` (33 files). The discrepancy slipped
/// through because no validator enforced the constant. The new
/// `docs/concept_pages_count` validator walks `website/content/concepts/`
/// (the authoritative site source) and asserts the count matches this
/// constant.
pub const CONCEPT_PAGES: usize = 12;

/// Number of PRISM pipeline stages (Define / Resolve / Certify).
pub const PIPELINE_STAGES: usize = 3;

/// Minimum number of classes in a namespace to generate a class hierarchy SVG.
pub const MIN_HIERARCHY_CLASSES: usize = 3;

// ─── Phase 0 orphan-closure classification counts ────────────────────────
//
// These counts reflect the output of
// `uor_codegen::classification::classify_all(Ontology::full())` and are
// asserted by `codegen/tests/classification_counts.rs`. Drift between the
// ontology and the classification fails that test. See
// `docs/orphan-closure/overview.md` for the 4-path taxonomy.

/// Classes skipped during classification — enum classes that don't
/// become traits. Matches `Ontology::enum_class_names().len()`.
pub const CLASSIFICATION_SKIP: usize = 19;

/// Classes whose ontology-derived trait already has a concrete `impl`
/// in `foundation/src/`. Phase 0 baseline: the four partition-algebra
/// traits (`Partition`, `PartitionProduct`, `PartitionCoproduct`,
/// `CartesianPartitionProduct`) closed by the Product/Coproduct
/// Amendment §845c0ff.
pub const CLASSIFICATION_ALREADY_IMPLEMENTED: usize = 4;

/// Path 1 — pure-accessor-bundle classes closed by the Phase 2
/// `{Foo}Handle` + `{Foo}Resolver` + `Resolved{Foo}` codegen rule.
///
/// Phase 11a (orphan-closure): five observables move from Path 1 to
/// Path 3 (LandauerBudget, JacobianObservable, CarryDepthObservable,
/// DerivationDepthObservable, FreeRankObservable). Their Phase-7 Null
/// stubs and Phase-8 Resolved wrappers stay in place; Phase 11 adds
/// the `Validated<T, Phase>` blanket impl on top.
///
/// Wiki ADR-038: +1 (observable:AxisProjectionObservable — closed-
/// catalog extension carried through the standard Path-1 codegen path
/// alongside the seven internally-derived Observable categories).
/// Wiki ADR-049: +1 (observable:SpectralObservable — Walsh–Hadamard-
/// parity-derived spectral subclass added under Path-1 codegen).
/// Wiki ADR-040 + ADR-049 catalog correspondence amendment: +1
/// (observable:ValueThresholdObservable — byte-sequence threshold
/// subclass realizing the type:LexicographicLessEqBound dispatch path;
/// added under Path-1 codegen).
pub const CLASSIFICATION_PATH1: usize = 416;

/// Path 2 — theorem-witness classes closed by the Phase 3
/// `{Foo}Witness` + `{Foo}MintInputs` + `impl VerifiedMint` codegen
/// rule.
pub const CLASSIFICATION_PATH2: usize = 10;

/// Path 3 — primitive-backed classes closed by Phase 11 hand-written
/// blanket impls in `foundation/src/blanket_impls.rs`. Phase 11a
/// populates the allow-list with five observables backed by existing
/// `primitive_*` functions in `foundation/src/enforcement.rs`.
pub const CLASSIFICATION_PATH3: usize = 5;

/// Path 4 — theory-deferred classes (cohomology / operad / parallel /
/// stream) annotated by Phase 6. Correctly orphan by design.
pub const CLASSIFICATION_PATH4: usize = 20;

/// Total Null-stub count emitted to bridge/kernel/user namespaces
/// (ratchet — grows, never shrinks) as of Phase 7. Excludes the 14
/// hand-written `NullPartition<H>`-family stubs in `enforcement.rs`.
///
/// Breakdown at Phase 7 close:
///
///   - Phase 2: 181 Null stubs for `Path1HandleResolver` classes whose
///     reference closure lands entirely in the emitable set.
///   - Phase 3: +7 Null stubs for `Path2TheoremWitness` classes that
///     satisfy the same reference-closure invariant.
///   - Phase 7a–e: enum accessors, inherited assocs, cross-namespace
///     enum imports, Path-4 banner stubs, and the enum-accessor filter
///     drop — unblocking every remaining Path-1 / Path-2 cascade and
///     the 20 Path-4 theory-deferred classes. Total reaches 440+.
pub const CLASSIFICATION_PATH1_EMITTED: usize = 440;
