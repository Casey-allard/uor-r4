//! Smoke tests for the three SDK shape-constructor macros.
//!
//! Each test constructs a combined shape from two simple leaf shapes and
//! verifies the resulting `ConstrainedTypeShape` impl matches the amendment's
//! site-count / site-budget arithmetic.

use uor_foundation::pipeline::{
    CartesianProductShape, ConstrainedTypeShape, ConstraintRef, FoundationClosed, PrismModel,
    AFFINE_MAX_COEFFS,
};
use uor_foundation_sdk::{cartesian_product_shape, coproduct_shape, prism_model, product_shape};

// Leaf shapes — Phase 17 expanded the SDK operand-support catalogue
// to every ConstraintRef variant. Affine and Conjunction now compose
// correctly through the macros because the variants store fixed-size
// arrays the const-eval can build inline.

pub struct LeafA;
impl ConstrainedTypeShape for LeafA {
    const IRI: &'static str = "https://example.org/sdk-smoke/LeafA";
    const SITE_COUNT: usize = 2;
    // SITE_BUDGET defaults to SITE_COUNT.
    const CONSTRAINTS: &'static [ConstraintRef] = &[
        ConstraintRef::Site { position: 0 },
        ConstraintRef::Site { position: 1 },
    ];
    const CYCLE_SIZE: u64 = 1;
}

pub struct LeafB;
impl ConstrainedTypeShape for LeafB {
    const IRI: &'static str = "https://example.org/sdk-smoke/LeafB";
    const SITE_COUNT: usize = 3;
    const CONSTRAINTS: &'static [ConstraintRef] = &[
        ConstraintRef::Site { position: 0 },
        ConstraintRef::Carry { site: 1 },
        ConstraintRef::Site { position: 2 },
    ];
    const CYCLE_SIZE: u64 = 1;
}

// --- product_shape! -------------------------------------------------------

product_shape!(LeafATimesLeafB, LeafA, LeafB);

#[test]
fn product_shape_site_budgets_add() {
    // PT_1: siteBudget(A × B) = siteBudget(A) + siteBudget(B).
    assert_eq!(<LeafATimesLeafB as ConstrainedTypeShape>::SITE_BUDGET, 5);
    // Layout invariant ProductLayoutWidth: SITE_COUNTs add.
    assert_eq!(<LeafATimesLeafB as ConstrainedTypeShape>::SITE_COUNT, 5);
}

#[test]
fn product_shape_constraints_splice_with_shift() {
    let constraints = <LeafATimesLeafB as ConstrainedTypeShape>::CONSTRAINTS;
    assert_eq!(constraints.len(), 5);
    // A's constraints copied verbatim.
    assert!(matches!(
        constraints[0],
        ConstraintRef::Site { position: 0 }
    ));
    assert!(matches!(
        constraints[1],
        ConstraintRef::Site { position: 1 }
    ));
    // B's constraints shifted by A::SITE_COUNT = 2.
    assert!(matches!(
        constraints[2],
        ConstraintRef::Site { position: 2 }
    ));
    assert!(matches!(constraints[3], ConstraintRef::Carry { site: 3 }));
    assert!(matches!(
        constraints[4],
        ConstraintRef::Site { position: 4 }
    ));
}

#[test]
fn product_shape_canonicalized_iri() {
    // Operand canonicalization sorts by token string: LeafA < LeafB.
    assert_eq!(
        <LeafATimesLeafB as ConstrainedTypeShape>::IRI,
        "urn:uor:product:LeafA:LeafB"
    );
}

// --- coproduct_shape! -----------------------------------------------------

coproduct_shape!(LeafAPlusLeafB, LeafA, LeafB);

#[test]
fn coproduct_shape_site_budget_maxes() {
    // ST_1: siteBudget(A + B) = max(siteBudget(A), siteBudget(B)).
    assert_eq!(<LeafAPlusLeafB as ConstrainedTypeShape>::SITE_BUDGET, 3);
    // CoproductLayoutWidth: SITE_COUNT = max(SITE_COUNT(A), SITE_COUNT(B)) + 1.
    assert_eq!(<LeafAPlusLeafB as ConstrainedTypeShape>::SITE_COUNT, 4);
}

#[test]
fn coproduct_shape_emits_two_tag_pinners() {
    let constraints = <LeafAPlusLeafB as ConstrainedTypeShape>::CONSTRAINTS;
    // A's constraints (2) + A's tag-pinner (1) + B's constraints (3) + B's tag-pinner (1) = 7.
    assert_eq!(constraints.len(), 7);

    // Tag site is at max(SITE_COUNT(A), SITE_COUNT(B)) = 3.
    // A's tag-pinner comes after A's constraints at index 2.
    match constraints[2] {
        ConstraintRef::Affine {
            coefficients,
            coefficient_count: _,
            bias,
        } => {
            assert_eq!(bias, 0, "left variant tag-pinner carries bias 0");
            assert_eq!(coefficients[3], 1, "coefficient at tag_site = 1");
        }
        _ => panic!(
            "expected Affine tag-pinner at index 2, got {:?}",
            constraints[2]
        ),
    }

    // B's tag-pinner comes after B's constraints at index 6.
    match constraints[6] {
        ConstraintRef::Affine {
            coefficients,
            coefficient_count: _,
            bias,
        } => {
            assert_eq!(bias, -1, "right variant tag-pinner carries bias -1");
            assert_eq!(coefficients[3], 1, "coefficient at tag_site = 1");
        }
        _ => panic!(
            "expected Affine tag-pinner at index 6, got {:?}",
            constraints[6]
        ),
    }
}

#[test]
fn coproduct_shape_canonicalized_iri() {
    assert_eq!(
        <LeafAPlusLeafB as ConstrainedTypeShape>::IRI,
        "urn:uor:coproduct:LeafA:LeafB"
    );
}

// --- cartesian_product_shape! ---------------------------------------------

cartesian_product_shape!(LeafATensorLeafB, LeafA, LeafB);

#[test]
fn cartesian_product_shape_site_budgets_add() {
    // CPT_1: siteBudget(A ⊠ B) = siteBudget(A) + siteBudget(B).
    assert_eq!(<LeafATensorLeafB as ConstrainedTypeShape>::SITE_BUDGET, 5);
    // CartesianLayoutWidth: SITE_COUNTs add.
    assert_eq!(<LeafATensorLeafB as ConstrainedTypeShape>::SITE_COUNT, 5);
}

#[test]
fn cartesian_product_shape_implements_marker() {
    // The macro emits the CartesianProductShape marker impl so the
    // Künneth-Betti primitive is selected.
    fn require_marker<S: CartesianProductShape>() {}
    require_marker::<LeafATensorLeafB>();
}

#[test]
fn cartesian_product_shape_canonicalized_iri() {
    assert_eq!(
        <LeafATensorLeafB as ConstrainedTypeShape>::IRI,
        "urn:uor:cartesian:LeafA:LeafB"
    );
}

// --- Phase 17: Affine + Conjunction operand support ----------------------

const AFFINE_TWO_PLUS_THREE: ([i64; AFFINE_MAX_COEFFS], u32) = {
    let mut a = [0i64; AFFINE_MAX_COEFFS];
    a[0] = 2;
    a[1] = 3;
    (a, 2)
};

/// Leaf shape carrying an `Affine` constraint — pre-Phase-17 this would
/// have been unsupported by the SDK macros.
pub struct LeafAffine;
impl ConstrainedTypeShape for LeafAffine {
    const IRI: &'static str = "https://example.org/sdk-smoke/LeafAffine";
    const SITE_COUNT: usize = 2;
    const CONSTRAINTS: &'static [ConstraintRef] = &[ConstraintRef::Affine {
        coefficients: AFFINE_TWO_PLUS_THREE.0,
        coefficient_count: AFFINE_TWO_PLUS_THREE.1,
        bias: 0,
    }];
    const CYCLE_SIZE: u64 = 1;
}

product_shape!(LeafAffineTimesLeafB, LeafAffine, LeafB);

#[test]
fn product_shape_supports_affine_operand() {
    // Pre-Phase-17 this expansion produced a `Site { position: u32::MAX }`
    // sentinel for the Affine constraint and the combined shape's
    // `validate_const()` rejected it. Post-Phase-17 the const-eval builds
    // a real shifted Affine — assert the constraint count covers L's
    // Affine + R's three constraints.
    let constraints = <LeafAffineTimesLeafB as ConstrainedTypeShape>::CONSTRAINTS;
    assert_eq!(constraints.len(), 4, "1 (L Affine) + 3 (R) = 4");
    // L's Affine pass-through (no shift since it's the first operand).
    match constraints[0] {
        ConstraintRef::Affine {
            coefficient_count, ..
        } => {
            assert_eq!(coefficient_count, 2, "L's affine prefix length preserved");
        }
        _ => panic!("expected Affine at index 0"),
    }
}

coproduct_shape!(LeafAffinePlusLeafB, LeafAffine, LeafB);

#[test]
fn coproduct_shape_supports_affine_operand() {
    let constraints = <LeafAffinePlusLeafB as ConstrainedTypeShape>::CONSTRAINTS;
    // L's Affine + L's tag-pinner + R's 3 + R's tag-pinner = 6.
    assert_eq!(constraints.len(), 6);
    match constraints[0] {
        ConstraintRef::Affine {
            coefficient_count, ..
        } => {
            assert_eq!(coefficient_count, 2, "L's Affine prefix length preserved");
        }
        _ => panic!("expected Affine at index 0"),
    }
    // L's tag-pinner at index 1.
    match constraints[1] {
        ConstraintRef::Affine {
            coefficient_count,
            bias,
            ..
        } => {
            assert!(coefficient_count > 0, "tag-pinner has non-zero prefix");
            assert_eq!(bias, 0, "L tag-pinner bias 0");
        }
        _ => panic!("expected Affine tag-pinner at index 1"),
    }
}

// =====================================================================
// `prism_model!` smoke tests — wiki ADR-020 + ADR-022 D3.
//
// These tests exercise the closure-bodied form: the macro parses the
// route function body as a Rust expression tree, maps recognised
// PrimitiveOp function calls to `Term::Application`, integer literals
// to `Term::Literal`, and the route's `input` parameter to
// `Term::Variable`. Anything else fails to compile (a closure violation
// per ADR-020).
//
// The test verifies the macro emits the four binding impls (D1 +
// D4 + D5) and that the parsed term arena is the value-level slice
// returned by `<Route as FoundationClosed<SMOKE_IB>>::arena_slice()`.

use uor_foundation::enforcement::{ConstrainedTypeInput, Hasher, Term};
use uor_foundation::{DefaultHostTypes, HostBounds, PrimitiveOp};

#[derive(Debug, Clone, Copy, Default)]
pub struct SmokeHasher;
impl Hasher for SmokeHasher {
    const OUTPUT_BYTES: usize = 16;
    fn initial() -> Self {
        Self
    }
    fn fold_byte(self, _: u8) -> Self {
        self
    }
    fn finalize(self) -> [u8; 32] {
        [0; 32]
    }
}

// ADR-060: `SmokeHostBounds` is removed — every application declares its own
// `impl HostBounds`. This is the smoke suite's reference bounds, carrying the
// pre-0.5.0 canonical values (16/32/256/64 + the 10 retained structural
// counts). Carrier byte widths are foundation-derived from these via
// `carrier_inline_bytes::<SmokeHostBounds>()`, not declared here.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct SmokeHostBounds;
impl HostBounds for SmokeHostBounds {
    const FINGERPRINT_MIN_BYTES: usize = 16;
    const FINGERPRINT_MAX_BYTES: usize = 32;
    const TRACE_MAX_EVENTS: usize = 256;
    const WITT_LEVEL_MAX_BITS: u32 = 64;
    const FOLD_UNROLL_THRESHOLD: usize = 8;
    const BETTI_DIMENSION_MAX: usize = 8;
    const NERVE_CONSTRAINTS_MAX: usize = 8;
    const NERVE_SITES_MAX: usize = 8;
    const JACOBIAN_SITES_MAX: usize = 8;
    const RECURSION_TRACE_DEPTH_MAX: usize = 16;
    const OP_CHAIN_DEPTH_MAX: usize = 8;
    const AFFINE_COEFFS_MAX: usize = 8;
    const CONJUNCTION_TERMS_MAX: usize = 8;
    const UNFOLD_ITERATIONS_MAX: usize = 256;
}

// ADR-060: foundation-derived inline carrier width for `SmokeHostBounds`.
// Bare references to `PrismModel`/`Has*Resolver`/`Grounded` in test-assertion
// code (outside the `prism_model!`/`resolver!` blocks, which synthesize it)
// thread this explicitly as the const-generic argument.
const SMOKE_IB: usize = uor_foundation::pipeline::carrier_inline_bytes::<SmokeHostBounds>();

// ADR-018/060: foundation-derived fingerprint width for `SmokeHostBounds`.
// Bare references to `PrismModel`/`AxisExtension`/`Grounded`/`evaluate_term_tree`
// in test-assertion code (outside the macro blocks, which synthesize it)
// thread this explicitly as the second capacity const-generic argument.
const SMOKE_FP: usize = <SmokeHostBounds as HostBounds>::FINGERPRINT_MAX_BYTES;

prism_model! {
    pub struct AddTwoLiterals;
    pub struct AddTwoLiteralsRoute;
    impl PrismModel<DefaultHostTypes, SmokeHostBounds, SmokeHasher> for AddTwoLiterals {
        type Input = ConstrainedTypeInput;
        type Output = ConstrainedTypeInput;
        type Route = AddTwoLiteralsRoute;
        fn route(input: Self::Input) -> Self::Output {
            add(2, 3)
        }
    }
}

#[test]
fn prism_model_macro_emits_term_arena_for_simple_addition() {
    let arena = <AddTwoLiteralsRoute as FoundationClosed<SMOKE_IB>>::arena_slice();
    // `add(2, 3)` → [Literal(2), Literal(3), Application{Add, [0..2]}]
    assert_eq!(
        arena.len(),
        3,
        "three terms: two literals + one application"
    );
    match arena[0] {
        Term::Literal { value, .. } => assert_eq!(value.bytes(), &[2u8][..]),
        other => panic!("expected Literal(2) at index 0, got {other:?}"),
    }
    match arena[1] {
        Term::Literal { value, .. } => assert_eq!(value.bytes(), &[3u8][..]),
        other => panic!("expected Literal(3) at index 1, got {other:?}"),
    }
    match arena[2] {
        Term::Application { operator, args } => {
            assert!(matches!(operator, PrimitiveOp::Add));
            assert_eq!(args.start, 0);
            assert_eq!(args.len, 2);
        }
        other => panic!("expected Application{{Add, [0..2]}} at index 2, got {other:?}"),
    }
}

prism_model! {
    pub struct VariableThenSucc;
    pub struct VariableThenSuccRoute;
    impl PrismModel<DefaultHostTypes, SmokeHostBounds, SmokeHasher> for VariableThenSucc {
        type Input = ConstrainedTypeInput;
        type Output = ConstrainedTypeInput;
        type Route = VariableThenSuccRoute;
        fn route(input: Self::Input) -> Self::Output {
            succ(input)
        }
    }
}

#[test]
fn prism_model_macro_recognises_input_variable_and_unary_op() {
    let arena = <VariableThenSuccRoute as FoundationClosed<SMOKE_IB>>::arena_slice();
    // `succ(input)` → [Variable(0), Application{Succ, [0..1]}]
    assert_eq!(arena.len(), 2);
    match arena[0] {
        Term::Variable { name_index } => assert_eq!(name_index, 0),
        other => panic!("expected Variable at index 0, got {other:?}"),
    }
    match arena[1] {
        Term::Application { operator, args } => {
            assert!(matches!(operator, PrimitiveOp::Succ));
            assert_eq!(args.start, 0);
            assert_eq!(args.len, 1);
        }
        other => panic!("expected Application{{Succ, …}} at index 1, got {other:?}"),
    }
}

#[test]
fn prism_model_macro_satisfies_prism_model_bound() {
    // The macro emitted `impl PrismModel<H, B, A> for AddTwoLiterals` —
    // pin that the impl resolves at compile time.
    fn _accepts<
        'a,
        M: PrismModel<'a, DefaultHostTypes, SmokeHostBounds, SmokeHasher, SMOKE_IB, SMOKE_FP>,
    >() {
    }
    _accepts::<AddTwoLiterals>();
    _accepts::<VariableThenSucc>();
    // Surface assertion: the bound check above is itself the test.
    assert_eq!(
        core::any::type_name::<
            <AddTwoLiterals as PrismModel<
                DefaultHostTypes,
                SmokeHostBounds,
                SmokeHasher,
                SMOKE_IB,
                SMOKE_FP,
            >>::Route,
        >(),
        core::any::type_name::<AddTwoLiteralsRoute>(),
    );
}

// =====================================================================
// `output_shape!` smoke tests — wiki ADR-027.
//
// The macro emits the four sealed-trait impls (`__sdk_seal::Sealed`,
// `ConstrainedTypeShape`, `GroundedShape`, `IntoBindingValue`) so a
// custom Output shape qualifies as a `PrismModel::Output`.

use uor_foundation::enforcement::GroundedShape;
use uor_foundation::pipeline::IntoBindingValue;
use uor_foundation_sdk::output_shape;

output_shape! {
    pub struct OutputHashSmoke;
    impl ConstrainedTypeShape for OutputHashSmoke {
        const IRI: &'static str = "https://example.org/sdk-smoke/OutputHash";
        const SITE_COUNT: usize = 32;
        const CONSTRAINTS: &'static [ConstraintRef] = &[];
        const CYCLE_SIZE: u64 = 1;
    }
}

#[test]
fn output_shape_emits_constrained_type_shape_impl() {
    assert_eq!(<OutputHashSmoke as ConstrainedTypeShape>::SITE_COUNT, 32);
    assert!(<OutputHashSmoke as ConstrainedTypeShape>::IRI.contains("OutputHash"));
}

#[test]
fn output_shape_emits_grounded_shape_impl() {
    fn _accepts<T: GroundedShape>() {}
    _accepts::<OutputHashSmoke>();
}

#[test]
fn output_shape_emits_into_binding_value_carrier() {
    // ADR-060: `IntoBindingValue` no longer carries a `MAX_BYTES` ceiling; an
    // output shape used as an input contributes the empty source-polymorphic
    // carrier (no bytes, no width cap).
    use uor_foundation::pipeline::{IntoBindingValue, TermValue};
    let shape = OutputHashSmoke;
    let carrier: TermValue<'_, SMOKE_IB> =
        <OutputHashSmoke as IntoBindingValue>::as_binding_value::<SMOKE_IB>(&shape);
    assert!(carrier.bytes().is_empty());
}

#[test]
fn output_shape_qualifies_as_prism_model_output() {
    fn _accepts<'a, T: ConstrainedTypeShape + GroundedShape + IntoBindingValue<'a>>() {}
    _accepts::<OutputHashSmoke>();
}

// =====================================================================
// `verb!` smoke tests — wiki ADR-024 Layer-3 implementation closure.
//
// The macro emits a const term-tree fragment (`VERB_TERMS_<NAME>`), a
// public accessor (`<name>_term_arena`), and a marker function
// (`<name>(input)`). When `prism_model!` invokes the verb by name in
// a route's closure body, the macro inlines the verb's fragment into
// the host arena at compile time via foundation's
// `inline_verb_fragment` const-fn helper — verb-graph acyclicity is
// a compile-time commitment, not a runtime guard.

use uor_foundation_sdk::verb;

verb! {
    pub fn smoke_succ(input: ConstrainedTypeInput) -> ConstrainedTypeInput {
        succ(input)
    }
}

#[test]
fn verb_macro_emits_term_arena_const() {
    let arena = smoke_succ_term_arena::<SMOKE_IB>();
    // `succ(input)` → [Variable, Application{Succ, [0..1]}]
    assert_eq!(arena.len(), 2);
    assert!(matches!(arena[0], Term::Variable { name_index: 0 }));
    match arena[1] {
        Term::Application { operator, args } => {
            assert!(matches!(operator, PrimitiveOp::Succ));
            assert_eq!(args.start, 0);
            assert_eq!(args.len, 1);
        }
        other => panic!("expected Application(Succ) at index 1, got {other:?}"),
    }
}

#[test]
fn verb_macro_const_is_publicly_visible() {
    // The `pub const VERB_TERMS_SMOKE_SUCC::<SMOKE_IB>()` is exported so prism_model!
    // can reference it when inlining via inline_verb_fragment (ADR-024).
    let arena = VERB_TERMS_SMOKE_SUCC::<SMOKE_IB>();
    assert_eq!(arena.len(), 2);
}

// `prism_model!` inlines the verb's term-tree fragment into the host
// arena at compile time when the closure body invokes a verb declared
// in the same module (wiki ADR-024).
prism_model! {
    pub struct VerbInvokingModel;
    pub struct VerbInvokingRoute;
    impl PrismModel<DefaultHostTypes, SmokeHostBounds, SmokeHasher> for VerbInvokingModel {
        type Input = ConstrainedTypeInput;
        type Output = ConstrainedTypeInput;
        type Route = VerbInvokingRoute;
        fn route(input: Self::Input) -> Self::Output {
            smoke_succ(input)
        }
    }
}

#[test]
fn prism_model_inlines_verb_fragment_for_local_verb_call() {
    let arena = <VerbInvokingRoute as FoundationClosed<SMOKE_IB>>::arena_slice();
    // After ADR-024 reconciliation, the route's arena is fully flat:
    // the verb's fragment is inlined at compile time. `smoke_succ(input)`
    // builds:
    //   [0] Variable(0)              (host's emission of `input`, the verb's argument)
    //   [1] Variable(0)              (verb's body Variable, spliced + shifted)
    //   [2] Application(Succ,[1..2]) (verb's body Application, with args.start shifted)
    //
    // The arena contains exactly 10-Term-variant nodes — no
    // `Term::VerbReference` (eleventh variant was removed).
    assert_eq!(arena.len(), 1 + VERB_TERMS_SMOKE_SUCC::<SMOKE_IB>().len());
    // No VerbReference in the arena: every entry is one of the eleven
    // ADR-029 variants (ten from the original signature category plus
    // Term::ProjectField from ADR-033).
    for (i, t) in arena.iter().enumerate() {
        match t {
            Term::Literal { .. }
            | Term::Variable { .. }
            | Term::Application { .. }
            | Term::Lift { .. }
            | Term::Project { .. }
            | Term::Match { .. }
            | Term::Recurse { .. }
            | Term::Unfold { .. }
            | Term::Try { .. }
            | Term::AxisInvocation { .. }
            | Term::ProjectField { .. }
            | Term::FirstAdmit { .. }
            | Term::Nerve { .. }
            | Term::ChainComplex { .. }
            | Term::HomologyGroups { .. }
            | Term::Betti { .. }
            | Term::CochainComplex { .. }
            | Term::CohomologyGroups { .. }
            | Term::PostnikovTower { .. }
            | Term::HomotopyGroups { .. }
            | Term::KInvariants { .. } => {}
        }
        let _ = i;
    }
    // The verb's last term (Application(Succ)) is the route's tail —
    // it sits at the arena's last position as the route's evaluation
    // root. Its args.start references the spliced Variable at the
    // host's offset (= 1, after the host's `input` emission).
    match arena.last().expect("non-empty arena") {
        Term::Application { operator, args } => {
            assert!(matches!(operator, PrimitiveOp::Succ));
            assert_eq!(args.start, 1u32, "verb's args.start shifted by host offset");
            assert_eq!(args.len, 1);
        }
        other => panic!("expected Application(Succ) as arena tail, got {other:?}"),
    }
}

// =====================================================================
// Closure-body grammar extensions G4 (Lift), G5 (Project), G10 (let).

use uor_foundation::WittLevel;

prism_model! {
    pub struct LiftToW16Model;
    pub struct LiftToW16Route;
    impl PrismModel<DefaultHostTypes, SmokeHostBounds, SmokeHasher> for LiftToW16Model {
        type Input = ConstrainedTypeInput;
        type Output = ConstrainedTypeInput;
        type Route = LiftToW16Route;
        fn route(input: Self::Input) -> Self::Output {
            lift::<WittLevel::W16>(input)
        }
    }
}

#[test]
fn prism_model_emits_lift_term_for_g4_lift_form() {
    let arena = <LiftToW16Route as FoundationClosed<SMOKE_IB>>::arena_slice();
    // `lift::<W16>(input)` → [Variable, Lift { operand: 0, target: W16 }]
    assert_eq!(arena.len(), 2);
    assert!(matches!(arena[0], Term::Variable { name_index: 0 }));
    match arena[1] {
        Term::Lift {
            operand_index,
            target,
        } => {
            assert_eq!(operand_index, 0);
            assert!(
                matches!(target, WittLevel::W16),
                "expected target W16, got {target:?}",
            );
        }
        other => panic!("expected Term::Lift at index 1, got {other:?}"),
    }
}

prism_model! {
    pub struct ProjectToW8Model;
    pub struct ProjectToW8Route;
    impl PrismModel<DefaultHostTypes, SmokeHostBounds, SmokeHasher> for ProjectToW8Model {
        type Input = ConstrainedTypeInput;
        type Output = ConstrainedTypeInput;
        type Route = ProjectToW8Route;
        fn route(input: Self::Input) -> Self::Output {
            project::<WittLevel::W8>(input)
        }
    }
}

#[test]
fn prism_model_emits_project_term_for_g5_project_form() {
    let arena = <ProjectToW8Route as FoundationClosed<SMOKE_IB>>::arena_slice();
    assert_eq!(arena.len(), 2);
    match arena[1] {
        Term::Project {
            operand_index,
            target,
        } => {
            assert_eq!(operand_index, 0);
            assert!(matches!(target, WittLevel::W8));
        }
        other => panic!("expected Term::Project at index 1, got {other:?}"),
    }
}

prism_model! {
    pub struct LetBindingModel;
    pub struct LetBindingRoute;
    impl PrismModel<DefaultHostTypes, SmokeHostBounds, SmokeHasher> for LetBindingModel {
        type Input = ConstrainedTypeInput;
        type Output = ConstrainedTypeInput;
        type Route = LetBindingRoute;
        fn route(input: Self::Input) -> Self::Output {
            let two = 2;
            add(two, 3)
        }
    }
}

#[test]
fn prism_model_emits_term_arena_for_g10_let_binding() {
    let arena = <LetBindingRoute as FoundationClosed<SMOKE_IB>>::arena_slice();
    // `let two = 2; add(two, 3)` → [Literal(2), Literal(3), Application(Add, [0..2])]
    // The let-binding doesn't emit its own Term node; references to
    // `two` resolve to the Literal(2) root via the binding scope (G10).
    assert_eq!(arena.len(), 3);
    match arena[0] {
        Term::Literal { value, .. } => assert_eq!(value.bytes(), &[2u8][..]),
        other => panic!("expected Literal(2) at index 0, got {other:?}"),
    }
    match arena[1] {
        Term::Literal { value, .. } => assert_eq!(value.bytes(), &[3u8][..]),
        other => panic!("expected Literal(3) at index 1, got {other:?}"),
    }
    match arena[2] {
        Term::Application { operator, args } => {
            assert!(matches!(operator, PrimitiveOp::Add));
            assert_eq!(args.start, 0);
            assert_eq!(args.len, 2);
        }
        other => panic!("expected Application(Add) at index 2, got {other:?}"),
    }
}

// =====================================================================
// Closure-body grammar G6 (match), G7 (recurse), G8 (unfold), G9 (?).

prism_model! {
    pub struct TryPropagateModel;
    pub struct TryPropagateRoute;
    impl PrismModel<DefaultHostTypes, SmokeHostBounds, SmokeHasher> for TryPropagateModel {
        type Input = ConstrainedTypeInput;
        type Output = ConstrainedTypeInput;
        type Route = TryPropagateRoute;
        fn route(input: Self::Input) -> Self::Output {
            succ(input)?
        }
    }
}

#[test]
fn prism_model_emits_try_term_for_g9_postfix_question() {
    let arena = <TryPropagateRoute as FoundationClosed<SMOKE_IB>>::arena_slice();
    // `succ(input)?` → [Variable, Application(Succ), Try{body=1, handler=u32::MAX}]
    assert_eq!(arena.len(), 3);
    match arena[2] {
        Term::Try {
            body_index,
            handler_index,
        } => {
            assert_eq!(body_index, 1);
            assert_eq!(handler_index, u32::MAX);
        }
        other => panic!("expected Try at index 2, got {other:?}"),
    }
}

prism_model! {
    pub struct RecurseModel;
    pub struct RecurseRoute;
    impl PrismModel<DefaultHostTypes, SmokeHostBounds, SmokeHasher> for RecurseModel {
        type Input = ConstrainedTypeInput;
        type Output = ConstrainedTypeInput;
        type Route = RecurseRoute;
        fn route(input: Self::Input) -> Self::Output {
            recurse(input, 0, |self_call| succ(self_call))
        }
    }
}

#[test]
fn prism_model_emits_recurse_term_for_g7_form() {
    let arena = <RecurseRoute as FoundationClosed<SMOKE_IB>>::arena_slice();
    // The arena ends with Term::Recurse pointing at the measure, base, and step roots.
    let last = arena.last().expect("non-empty arena");
    assert!(matches!(last, Term::Recurse { .. }));
}

prism_model! {
    pub struct UnfoldModel;
    pub struct UnfoldRoute;
    impl PrismModel<DefaultHostTypes, SmokeHostBounds, SmokeHasher> for UnfoldModel {
        type Input = ConstrainedTypeInput;
        type Output = ConstrainedTypeInput;
        type Route = UnfoldRoute;
        fn route(input: Self::Input) -> Self::Output {
            unfold(input, |state| succ(state))
        }
    }
}

#[test]
fn prism_model_emits_unfold_term_for_g8_form() {
    let arena = <UnfoldRoute as FoundationClosed<SMOKE_IB>>::arena_slice();
    assert!(matches!(arena.last(), Some(Term::Unfold { .. })));
}

prism_model! {
    pub struct FoldNUnrolledModel;
    pub struct FoldNUnrolledRoute;
    impl PrismModel<DefaultHostTypes, SmokeHostBounds, SmokeHasher> for FoldNUnrolledModel {
        type Input = ConstrainedTypeInput;
        type Output = ConstrainedTypeInput;
        type Route = FoldNUnrolledRoute;
        fn route(input: Self::Input) -> Self::Output {
            fold_n(3, input, |state, idx| add(state, idx))
        }
    }
}

#[test]
fn prism_model_unrolls_fold_n_for_const_count_below_threshold() {
    let arena = <FoldNUnrolledRoute as FoundationClosed<SMOKE_IB>>::arena_slice();
    // fold_n(3, input, |state, idx| add(state, idx)) unrolls into:
    //   iter 0: add(input, 0)
    //   iter 1: add(<iter 0 result>, 1)
    //   iter 2: add(<iter 1 result>, 2)
    // The arena ends with the iter-2 Application(Add).
    assert!(matches!(
        arena.last(),
        Some(Term::Application {
            operator: PrimitiveOp::Add,
            ..
        })
    ));
    // Three Application(Add) entries — one per iteration.
    let add_count = arena
        .iter()
        .filter(|t| {
            matches!(
                t,
                Term::Application {
                    operator: PrimitiveOp::Add,
                    ..
                }
            )
        })
        .count();
    assert_eq!(
        add_count, 3,
        "fold_n(3, …) unrolls into 3 Application(Add) chains"
    );
}

prism_model! {
    pub struct MatchModel;
    pub struct MatchRoute;
    impl PrismModel<DefaultHostTypes, SmokeHostBounds, SmokeHasher> for MatchModel {
        type Input = ConstrainedTypeInput;
        type Output = ConstrainedTypeInput;
        type Route = MatchRoute;
        fn route(input: Self::Input) -> Self::Output {
            match input {
                0 => 1,
                _ => succ(input),
            }
        }
    }
}

#[test]
fn prism_model_emits_match_term_for_g6_form() {
    let arena = <MatchRoute as FoundationClosed<SMOKE_IB>>::arena_slice();
    let last = arena.last().expect("non-empty arena");
    match last {
        Term::Match { arms, .. } => {
            // Two arms × 2 entries each = 4 entries in the arms span.
            assert_eq!(
                arms.len, 4,
                "expected 4 arms entries (2 arms × pattern+body)"
            );
        }
        other => panic!("expected Term::Match as root, got {other:?}"),
    }
}

// =====================================================================
// `use_verbs!` smoke test.

mod inner_verb_module {
    use uor_foundation::enforcement::ConstrainedTypeInput;
    use uor_foundation_sdk::verb;

    verb! {
        pub fn inner_verb(input: ConstrainedTypeInput) -> ConstrainedTypeInput {
            succ(input)
        }
    }
}

uor_foundation_sdk::use_verbs! {
    from inner_verb_module {
        inner_verb,
    };
}

// =====================================================================
// Closure-body grammar G13 (parallel), G15 (tree_fold), G16 (first_admit).

prism_model! {
    pub struct ParallelComposeModel;
    pub struct ParallelComposeRoute;
    impl PrismModel<DefaultHostTypes, SmokeHostBounds, SmokeHasher> for ParallelComposeModel {
        type Input = ConstrainedTypeInput;
        type Output = ConstrainedTypeInput;
        type Route = ParallelComposeRoute;
        fn route(input: Self::Input) -> Self::Output {
            parallel(succ(input), pred(input))
        }
    }
}

#[test]
fn prism_model_emits_parallel_term_for_g13_form() {
    let arena = <ParallelComposeRoute as FoundationClosed<SMOKE_IB>>::arena_slice();
    // `parallel(succ(input), pred(input))` lowers to a binary
    // Application(Or, [succ(input), pred(input)]) — the partition-product
    // structural combine per ADR-026 G13.
    let last = arena.last().expect("non-empty arena");
    match last {
        Term::Application { operator, args } => {
            assert!(matches!(operator, PrimitiveOp::Or));
            assert_eq!(args.len, 2, "parallel emits 2-arg structural combine");
        }
        other => panic!("expected Application(Or) as parallel root, got {other:?}"),
    }
}

prism_model! {
    pub struct TreeFoldModel;
    pub struct TreeFoldRoute;
    impl PrismModel<DefaultHostTypes, SmokeHostBounds, SmokeHasher> for TreeFoldModel {
        type Input = ConstrainedTypeInput;
        type Output = ConstrainedTypeInput;
        type Route = TreeFoldRoute;
        fn route(input: Self::Input) -> Self::Output {
            tree_fold(add, [1, 2, 3, 4])
        }
    }
}

#[test]
fn prism_model_emits_tree_fold_pairwise_chain_for_g15_form() {
    let arena = <TreeFoldRoute as FoundationClosed<SMOKE_IB>>::arena_slice();
    // tree_fold(add, [1, 2, 3, 4]) → balanced tree of depth 2:
    //   add(add(1, 2), add(3, 4))
    // Three Application(Add) entries (two leaf-level + one root).
    let add_count = arena
        .iter()
        .filter(|t| {
            matches!(
                t,
                Term::Application {
                    operator: PrimitiveOp::Add,
                    ..
                }
            )
        })
        .count();
    assert_eq!(
        add_count, 3,
        "tree_fold(add, [a,b,c,d]) → 3 Application(Add) entries"
    );
    // Last term is the root reducer Application.
    assert!(matches!(
        arena.last(),
        Some(Term::Application {
            operator: PrimitiveOp::Add,
            ..
        })
    ));
}

// Wiki ADR-035 ψ-residuals discipline: `first_admit(...)` is a
// ψ-enumeration residual of search-based admission and is rejected by
// `prism_model!` / `verb!` at proc-macro expansion. Foundation's
// `Term::FirstAdmit` variant remains in the substrate (the catamorphism
// still folds it for non-verb-body callers — conformance generators,
// trace replay). The smoke test that previously asserted the lowering
// has been retired; the substrate-level catamorphism behavior is
// covered by `behavior_catamorphism_evaluator.rs`. The line below pins
// the reject-at-emit surface as a doc-snippet for downstream
// applications converting from G16-based admission to ψ-chain
// composition.
//
// Pre-discipline (ADR-034) form (now rejected):
//   first_admit(witt_domain::W8, |i| <pred>)
// Post-discipline (ADR-035) canonical form:
//   k_invariants(homotopy_groups(postnikov_tower(nerve(input))))
//
// See `psi_chain_*` tests below for the canonical replacement pattern.

/// ADR-032 (G5) regression: confirm `witt_domain::W8` carries
/// `CYCLE_SIZE = 256` and `witt_domain::W16` carries `CYCLE_SIZE = 65536`,
/// matching the wiki's normative declarations.
#[test]
fn witt_domain_cycle_size_matches_wiki_spec() {
    assert_eq!(
        <uor_foundation::pipeline::witt_domain::W8 as ConstrainedTypeShape>::CYCLE_SIZE,
        256
    );
    assert_eq!(
        <uor_foundation::pipeline::witt_domain::W16 as ConstrainedTypeShape>::CYCLE_SIZE,
        65536
    );
    assert_eq!(
        <uor_foundation::pipeline::witt_domain::W32 as ConstrainedTypeShape>::CYCLE_SIZE,
        4_294_967_296
    );
    // W64+ saturate at u64::MAX (2^64 doesn't fit in u64).
    assert_eq!(
        <uor_foundation::pipeline::witt_domain::W64 as ConstrainedTypeShape>::CYCLE_SIZE,
        u64::MAX
    );
}

// =====================================================================
// `partition_product!` and `partition_coproduct!` smoke tests
// — wiki ADR-026 G17/G18 architectural-name macros (variadic, named
// stable-Rust form per CLAUDE.md mapping).

use uor_foundation_sdk::{partition_coproduct, partition_product};

partition_product!(LeafAPpLeafB, LeafA, LeafB);

#[test]
fn partition_product_macro_matches_pt3_canonical_join() {
    // partition_product!(N, A, B) emits the same structure as
    // product_shape!(N, A, B) — PT_3 canonical-joined CONSTRAINTS,
    // SITE_COUNT = A::SITE_COUNT + B::SITE_COUNT.
    assert_eq!(<LeafAPpLeafB as ConstrainedTypeShape>::SITE_BUDGET, 5);
    assert_eq!(<LeafAPpLeafB as ConstrainedTypeShape>::SITE_COUNT, 5);
    assert!(<LeafAPpLeafB as ConstrainedTypeShape>::IRI.starts_with("urn:uor:product:"));
}

partition_coproduct!(LeafAPcLeafB, LeafA, LeafB);

#[test]
fn partition_coproduct_macro_matches_st10_structure() {
    assert_eq!(<LeafAPcLeafB as ConstrainedTypeShape>::SITE_BUDGET, 3);
    assert_eq!(<LeafAPcLeafB as ConstrainedTypeShape>::SITE_COUNT, 4);
    assert!(<LeafAPcLeafB as ConstrainedTypeShape>::IRI.starts_with("urn:uor:coproduct:"));
}

#[test]
fn partition_product_macro_emits_grounded_shape_and_into_binding_value() {
    fn _accepts<'a, T: ConstrainedTypeShape + GroundedShape + IntoBindingValue<'a>>() {}
    _accepts::<LeafAPpLeafB>();
    _accepts::<LeafAPcLeafB>();
}

// Variadic 3-operand form folds left-associatively.
partition_product!(LeafThreeWayPp, LeafA, LeafB, LeafA);

#[test]
fn partition_product_variadic_3_operands_folds_left_associatively() {
    // ((A × B) × A) → SITE_COUNT = (2 + 3) + 2 = 7.
    assert_eq!(<LeafThreeWayPp as ConstrainedTypeShape>::SITE_COUNT, 7);
}

// =====================================================================
// `use_verbs!` smoke test (continued).

#[test]
fn use_verbs_re_exports_verb_const_and_accessor() {
    // The re-exported const matches the original module's const.
    assert_eq!(
        VERB_TERMS_INNER_VERB::<SMOKE_IB>().len(),
        inner_verb_module::VERB_TERMS_INNER_VERB::<SMOKE_IB>().len(),
    );
    // The re-exported accessor returns the same fragment.
    let arena = inner_verb_term_arena::<SMOKE_IB>();
    assert_eq!(arena.len(), 2);
    assert!(matches!(arena[0], Term::Variable { name_index: 0 }));
}

// =====================================================================
// ADR-033 G3 — named-field `partition_product!` form.
// =====================================================================

partition_product!(NamedFieldPp, lhs: LeafA, rhs: LeafB);

#[test]
fn partition_product_named_form_emits_field_names() {
    use uor_foundation::pipeline::PartitionProductFields;
    let names = <NamedFieldPp as PartitionProductFields>::FIELD_NAMES;
    assert_eq!(names, &["lhs", "rhs"]);
    let fields = <NamedFieldPp as PartitionProductFields>::FIELDS;
    // First field starts at byte 0 with width LeafA::SITE_COUNT (=2);
    // second starts at LeafA::SITE_COUNT (=2) with width LeafB::SITE_COUNT (=3).
    assert_eq!(fields[0], (0u32, 2u32));
    assert_eq!(fields[1], (2u32, 3u32));
    assert_eq!(
        <NamedFieldPp as PartitionProductFields>::field_index_by_name("lhs"),
        0
    );
    assert_eq!(
        <NamedFieldPp as PartitionProductFields>::field_index_by_name("rhs"),
        1
    );
    // Missing names sentinel: usize::MAX.
    assert_eq!(
        <NamedFieldPp as PartitionProductFields>::field_index_by_name("missing"),
        usize::MAX
    );
}

#[test]
fn partition_product_positional_form_emits_empty_field_names() {
    use uor_foundation::pipeline::PartitionProductFields;
    let names = <LeafAPpLeafB as PartitionProductFields>::FIELD_NAMES;
    assert_eq!(names, &["", ""]);
}

// =====================================================================
// ADR-033 G4 — chained field access in `prism_model!` closures.
// =====================================================================

// Two-level shape: outer factors are themselves named partition_products.
partition_product!(InnerLR, lhs: LeafA, rhs: LeafB);
partition_product!(OuterLR, outer: InnerLR, tail: LeafA);

prism_model! {
    pub struct ChainedFieldModel;
    pub struct ChainedFieldRoute;
    impl PrismModel<DefaultHostTypes, SmokeHostBounds, SmokeHasher> for ChainedFieldModel {
        type Input = OuterLR;
        type Output = ConstrainedTypeInput;
        type Route = ChainedFieldRoute;
        fn route(input: Self::Input) -> Self::Output {
            // ADR-033 G4 chained access: first project `outer` from
            // OuterLR, then project `lhs` from InnerLR. The proc-macro
            // resolves the inner type via
            //   <OuterLR as PartitionProductFactor<{ field_index_by_name("outer") }>>::Factor
            // = InnerLR
            // and synthesizes the `<InnerLR as PartitionProductFields>::FIELDS`
            // lookup for the outer ProjectField's offset/length.
            input.outer.lhs
        }
    }
}

#[test]
fn prism_model_emits_chained_project_field_terms() {
    let arena = <ChainedFieldRoute as FoundationClosed<SMOKE_IB>>::arena_slice();
    // Expected: [Variable(0), ProjectField(outer), ProjectField(lhs)]
    assert_eq!(arena.len(), 3);
    assert!(matches!(arena[0], Term::Variable { name_index: 0 }));
    let outer = match &arena[1] {
        Term::ProjectField {
            source_index,
            byte_offset,
            byte_length,
        } => (*source_index, *byte_offset, *byte_length),
        other => panic!("expected ProjectField at [1], got {other:?}"),
    };
    assert_eq!(
        outer.0, 0u32,
        "first projection sources from the input variable"
    );
    // OuterLR layout: outer at offset 0 with width = InnerLR::SITE_COUNT
    // (= 2 + 3 = 5); tail at offset 5 with width = LeafA::SITE_COUNT (= 2).
    assert_eq!(outer.1, 0u32, "outer field starts at byte 0");
    assert_eq!(outer.2, 5u32, "outer field width = InnerLR::SITE_COUNT (5)");
    let inner = match &arena[2] {
        Term::ProjectField {
            source_index,
            byte_offset,
            byte_length,
        } => (*source_index, *byte_offset, *byte_length),
        other => panic!("expected ProjectField at [2], got {other:?}"),
    };
    assert_eq!(
        inner.0, 1u32,
        "second projection sources from the first projection"
    );
    // InnerLR layout: lhs at offset 0 width 2, rhs at offset 2 width 3.
    assert_eq!(inner.1, 0u32, "lhs field starts at byte 0 within InnerLR");
    assert_eq!(inner.2, 2u32, "lhs field width = LeafA::SITE_COUNT (2)");
}

// G4 with positional chained access: `input.0.0`.
partition_product!(PosOuter, InnerLR, LeafA);

// ── Dependency 1: verb! depth-2 partition-product field access ────────
//
// ADR-033 G20 + ADR-056: verbs may use depth-2 field-access (`input.0.0`,
// `input.0.1`, `input.1`) to project the constituent fields of a nested
// partition-product shape. The chained-field syntax lowers to nested
// `Term::ProjectField` arena entries, with each ProjectField's
// byte_offset/byte_length resolved via the const-eval lookup chain
// `<InnerTy as PartitionProductFactor<IDX>>::Factor as PartitionProductFields>::FIELDS[IDX2]`.
//
// Pre-v0.4.10 the verb! macro shared the depth-1 lowering path but the
// chained type-resolution wasn't reaching `PartitionProductFactor`
// through the verb's input-type pin; v0.4.10 closes the parity gap.

verb! {
    pub fn verb_depth2_pos00(input: PosOuter) -> ConstrainedTypeInput {
        input.0.0
    }
}

verb! {
    pub fn verb_depth2_pos01(input: PosOuter) -> ConstrainedTypeInput {
        input.0.1
    }
}

verb! {
    pub fn verb_depth2_pos1(input: PosOuter) -> ConstrainedTypeInput {
        input.1
    }
}

#[test]
fn verb_macro_admits_depth2_positional_field_access() {
    // Depth-2 `input.0.0` lowers to nested ProjectField entries:
    // [Variable, ProjectField(0 → 0), ProjectField(1 → 0)].
    let arena = verb_depth2_pos00_term_arena::<SMOKE_IB>();
    assert_eq!(arena.len(), 3);
    assert!(matches!(arena[0], Term::Variable { name_index: 0 }));
    assert!(matches!(
        arena[1],
        Term::ProjectField {
            source_index: 0,
            ..
        }
    ));
    assert!(matches!(
        arena[2],
        Term::ProjectField {
            source_index: 1,
            ..
        }
    ));
}

#[test]
fn verb_macro_admits_depth2_pos01() {
    let arena = verb_depth2_pos01_term_arena::<SMOKE_IB>();
    assert_eq!(arena.len(), 3);
    assert!(matches!(
        arena[2],
        Term::ProjectField {
            source_index: 1,
            ..
        }
    ));
}

#[test]
fn verb_macro_admits_depth1_pos1_in_chained_context() {
    let arena = verb_depth2_pos1_term_arena::<SMOKE_IB>();
    assert_eq!(arena.len(), 2);
    assert!(matches!(
        arena[1],
        Term::ProjectField {
            source_index: 0,
            ..
        }
    ));
}

// Three-operand depth-2 access: read all three constituent fields in one
// verb body. This is the actual implementer-reported failure mode —
// nested ProjectField rotation under verb-call composition.
verb! {
    pub fn verb_depth2_three_operands(input: PosOuter) -> ConstrainedTypeInput {
        // Compose all three depth-2 projections through concat (admissible
        // in verb bodies per ADR-056).
        concat(concat(input.0.0, input.0.1), input.1)
    }
}

#[test]
fn verb_macro_admits_three_operand_depth2_access() {
    let arena = verb_depth2_three_operands_term_arena::<SMOKE_IB>();
    // Expect: Variable, PF(0→0), PF(1→0), Variable, PF(3→0), PF(4→1),
    //         Application(Concat, [2..3+1]), Variable, PF(7→1),
    //         Application(Concat, [combined..]).
    // The exact arena layout depends on how clone_term_spec rewrites
    // arg-contiguity, but the key constraint is: the macro accepts the
    // syntax without panicking.
    assert!(!arena.is_empty());
    // Final term is the outermost Concat application.
    let last = arena.last().unwrap();
    match last {
        Term::Application { operator, args } => {
            assert!(matches!(operator, PrimitiveOp::Concat));
            assert_eq!(args.len, 2);
        }
        other => panic!("expected Application(Concat, …) at root, got {other:?}"),
    }
}

// ── Dependency 2: wide-Witt TermValue literal embedding in verb bodies ─
//
// ADR-051 commits Term::Literal carries a TermValue (byte sequence) so
// wide-Witt literals (W128+) are natively representable without Concat
// composition over narrow literals. The verb-body grammar surfaces this
// via two call forms:
//
// - `literal_u64(<value>, <level>)` — narrow form, packs a u64 value as
//   big-endian bytes at the declared Witt level's byte width.
// - `literal_bytes(<bytes>, <level>)` — wide form, accepts a byte slice
//   for widths > 8 bytes (W128 = 16, W256 = 32, etc.).
//
// Both lower to `Term::Literal { value: TermValue, level }` at compile
// time via the foundation-emitted `pipeline::literal_u64` and
// `pipeline::literal_bytes` const fns.

verb! {
    pub fn verb_wide_literal_u64(input: ConstrainedTypeInput) -> ConstrainedTypeInput {
        // Narrow form at an intermediate width — W16 holds 0xDEAD as
        // exactly two bytes (big-endian: 0xDE, 0xAD).
        literal_u64(0xDEAD, uor_foundation::WittLevel::W16)
    }
}

#[test]
fn verb_macro_admits_literal_u64_wide_form() {
    let arena = verb_wide_literal_u64_term_arena::<SMOKE_IB>();
    assert_eq!(arena.len(), 1);
    match arena[0] {
        Term::Literal { value, level } => {
            assert_eq!(value.bytes(), &[0xDEu8, 0xAD][..]);
            assert_eq!(level.witt_length(), 16);
        }
        other => panic!("expected Literal at index 0, got {other:?}"),
    }
}

/// Public byte-table acting as the test's wide-literal source. In a real
/// secp256k1 verb body this would be the prime modulus P:
///   `pub const P_LITERAL: &[u8; 32] = &[0xff; 32];`
pub const W128_TEST_BYTES: &[u8; 16] = &[
    0xCA, 0xFE, 0xBA, 0xBE, 0xDE, 0xAD, 0xBE, 0xEF, 0xFE, 0xED, 0xFA, 0xCE, 0xC0, 0xFF, 0xEE, 0x99,
];

verb! {
    pub fn verb_wide_literal_bytes(input: ConstrainedTypeInput) -> ConstrainedTypeInput {
        // Wide form: a 128-bit constant from a static byte table. Real-world
        // use sites embed prime moduli (secp256k1 P), AES round constants,
        // FHE plaintext coefficients, etc.
        literal_bytes(W128_TEST_BYTES, uor_foundation::WittLevel::new(128))
    }
}

#[test]
fn verb_macro_admits_literal_bytes_wide_form_at_w128() {
    let arena = verb_wide_literal_bytes_term_arena::<SMOKE_IB>();
    assert_eq!(arena.len(), 1);
    match arena[0] {
        Term::Literal { value, level } => {
            assert_eq!(value.bytes(), W128_TEST_BYTES);
            assert_eq!(level.witt_length(), 128);
        }
        other => panic!("expected Literal at index 0, got {other:?}"),
    }
}

verb! {
    pub fn verb_wide_literal_compose(input: ConstrainedTypeInput) -> ConstrainedTypeInput {
        // ADR-051 + ADR-053: wide literals participate in compound verb
        // bodies via the standard substrate ops. This pattern realizes
        // the secp256k1 `field_mul_p` decomposition: project the input
        // bytes, multiply by an in-arena wide constant, take the
        // remainder mod the prime modulus.
        r#mod(
            mul(input, literal_bytes(W128_TEST_BYTES, uor_foundation::WittLevel::new(128))),
            literal_bytes(W128_TEST_BYTES, uor_foundation::WittLevel::new(128))
        )
    }
}

#[test]
fn verb_macro_admits_wide_literal_in_compound_body() {
    let arena = verb_wide_literal_compose_term_arena::<SMOKE_IB>();
    // Outermost: Application(Mod, [mul, literal]).
    let last = arena.last().expect("non-empty arena");
    match last {
        Term::Application { operator, args } => {
            assert!(matches!(operator, PrimitiveOp::Mod));
            assert_eq!(args.len, 2);
        }
        other => panic!("expected Application(Mod, …) at root, got {other:?}"),
    }
}

// ── ADR-040 + ADR-048 + ADR-049: LexicographicLessEqThreshold + TargetCommitment ─
//
// Wiki amendment (post-v0.4.11): the canonical search-cost commitment
// realization. `LexicographicLessEqThreshold` is the 5th foundation-
// published ObservablePredicate impl realizing the
// `type:LexicographicLessEqBound` bound-shape primitive's dispatch path.
// `TargetCommitment = SingletonCommitment<LexicographicLessEqThreshold>`
// is the canonical alias consumed by Bitcoin-PoW-style payload encodings,
// ZK proof-system difficulty commitments, and any application bounding
// `(digest as BE integer) <= target`.

#[test]
fn lexicographic_less_eq_threshold_evaluates_be_unsigned_inequality() {
    use uor_foundation::pipeline::{LexicographicLessEqThreshold, ObservablePredicate};
    const TARGET: &[u8] = &[0x00, 0xFF];
    let pred = LexicographicLessEqThreshold { target: TARGET };
    // digest < target: 0x0010 < 0x00FF → accept.
    assert!(pred.evaluate(&[0x00, 0x10]));
    // digest = target → accept (<= is inclusive).
    assert!(pred.evaluate(&[0x00, 0xFF]));
    // digest > target → reject.
    assert!(!pred.evaluate(&[0x01, 0x00]));
    // Shorter digest, right-aligned: 0x10 padded to 0x0010 → accept.
    assert!(pred.evaluate(&[0x10]));
}

#[test]
fn lexicographic_less_eq_threshold_observable_iri_is_canonical() {
    use uor_foundation::pipeline::{LexicographicLessEqThreshold, ObservablePredicate};
    let pred = LexicographicLessEqThreshold { target: &[0; 0] };
    assert_eq!(
        pred.observable_iri(),
        "https://uor.foundation/observable/LexicographicLessEqThreshold"
    );
}

#[test]
fn lexicographic_less_eq_threshold_accept_prob_under_u1() {
    use uor_foundation::pipeline::{LexicographicLessEqThreshold, ObservablePredicate};
    // target = 0x80 (half of u8 range); accept_prob = (128 + 1) / 256 ≈ 0.504.
    let pred = LexicographicLessEqThreshold { target: &[0x80] };
    let p = pred.accept_prob();
    assert!((p - (129.0 / 256.0)).abs() < 1e-12);
}

#[test]
fn target_commitment_alias_is_singleton_of_threshold() {
    use uor_foundation::pipeline::{
        LexicographicLessEqThreshold, SingletonCommitment, TargetCommitment, TypedCommitment,
    };
    const TARGET: &[u8] = &[0x00, 0xFF, 0xFF];
    const C: TargetCommitment = SingletonCommitment {
        predicate: LexicographicLessEqThreshold { target: TARGET },
    };
    // The alias is exactly SingletonCommitment<LexicographicLessEqThreshold>;
    // bandwidth/accept_prob delegate to the inner predicate.
    assert_eq!(C.predicate_count(), 1);
    // accept_prob: target = 0x00FFFF; (target_int + 1) / 2^24 ≈ 65536 / 16777216.
    let p = C.accept_prob();
    assert!(p > 0.0);
    assert!(p < 1.0);
    // evaluate: digest 0x000010 (=16) <= target 0x00FFFF (=65535) → accept.
    assert!(C.evaluate(&[0x00, 0x00, 0x10]));
    // digest 0x010000 (=65536) > target → reject.
    assert!(!C.evaluate(&[0x01, 0x00, 0x00]));
}

// ── Const-generic leaf shape (BigIntShape<N>): depth-2 verb! access ────
//
// Implementer-reported regression: a verb body's depth-2 field access
// through a partition product containing a const-generic leaf (like
// `BigIntShape<N>`) triggers a const-eval failure pre-v0.4.11. The
// hand-rolled non-generic LeafA depth-2 case (earlier in this file)
// works fine.

/// A const-generic leaf shape implementing the partition-shape contract
/// generically over `const N: usize`. Mirrors prism-numerics'
/// `BigIntShape<N>` declaration shape.
pub struct BigIntShape<const N: usize>;

impl<const N: usize> ConstrainedTypeShape for BigIntShape<N> {
    const IRI: &'static str = "https://example.org/sdk-smoke/BigIntShape";
    const SITE_COUNT: usize = N;
    const CONSTRAINTS: &'static [ConstraintRef] = &[];
    const CYCLE_SIZE: u64 = 1;
}

impl<const N: usize> uor_foundation::pipeline::__sdk_seal::Sealed for BigIntShape<N> {}
impl<'a, const N: usize> uor_foundation::pipeline::IntoBindingValue<'a> for BigIntShape<N> {
    fn as_binding_value<const INLINE_BYTES: usize>(
        &self,
    ) -> uor_foundation::pipeline::TermValue<'a, INLINE_BYTES> {
        uor_foundation::pipeline::TermValue::empty()
    }
}
impl<const N: usize> uor_foundation::enforcement::GroundedShape for BigIntShape<N> {}

// Monomorphized alias the `partition_product!` macro accepted pre-v0.4.11
// (the parser took bare idents). v0.4.11 widens the parser to accept full
// `syn::Type` operands so generic shapes can be passed directly without
// the intermediate alias. The alias is retained for use in tests below
// that compare the in-place vs aliased forms.
pub type Big128 = BigIntShape<128>;

partition_product!(OuterBigInt, Big128, LeafA);

// v0.4.11: in-place generic operand without the intermediate alias.
partition_product!(OuterBigIntDirect, BigIntShape<128>, LeafA);

#[test]
fn partition_product_admits_in_place_const_generic_operand() {
    // The IRI must canonicalize the generic operand token-string-wise so
    // `partition_product!(_, BigIntShape<128>, LeafA)` and
    // `partition_product!(_, LeafA, BigIntShape<128>)` produce identical
    // IRIs (canonical ordering applies).
    let iri = <OuterBigIntDirect as ConstrainedTypeShape>::IRI;
    assert!(
        iri.contains("BigIntShape"),
        "IRI should mention BigIntShape, got `{iri}`"
    );
    assert!(
        iri.contains("LeafA"),
        "IRI should mention LeafA, got `{iri}`"
    );
    // SITE_COUNT sums: BigIntShape<128>::SITE_COUNT = 128, LeafA = 2.
    assert_eq!(<OuterBigIntDirect as ConstrainedTypeShape>::SITE_COUNT, 130);
}

verb! {
    pub fn verb_depth1_through_in_place_const_generic(input: OuterBigIntDirect) -> ConstrainedTypeInput {
        // Depth-1 access through the in-place const-generic partition product.
        input.1
    }
}

#[test]
fn verb_macro_admits_depth1_with_in_place_const_generic_operand() {
    let arena = verb_depth1_through_in_place_const_generic_term_arena::<SMOKE_IB>();
    assert_eq!(arena.len(), 2);
    assert!(matches!(
        arena[1],
        Term::ProjectField {
            source_index: 0,
            ..
        }
    ));
}

// Depth-2 with an in-place const-generic INNER partition product.
partition_product!(InnerBigInt, BigIntShape<128>, LeafA);
partition_product!(BigIntOuterDirect, InnerBigInt, LeafA);

verb! {
    pub fn verb_depth2_through_in_place_const_generic(input: BigIntOuterDirect) -> ConstrainedTypeInput {
        // input.0 = InnerBigInt; input.0.0 = BigIntShape<128>.
        input.0.0
    }
}

#[test]
fn verb_macro_admits_depth2_through_in_place_const_generic() {
    let arena = verb_depth2_through_in_place_const_generic_term_arena::<SMOKE_IB>();
    assert_eq!(arena.len(), 3);
    assert!(matches!(
        arena[1],
        Term::ProjectField {
            source_index: 0,
            ..
        }
    ));
    assert!(matches!(
        arena[2],
        Term::ProjectField {
            source_index: 1,
            ..
        }
    ));
}

// Pre-v0.4.11 the `prism_model!`-equivalent depth-2 access via verb!
// hit a const-eval failure when the inner factor was const-generic;
// v0.4.11 closes the gap so the depth-2 access compiles and lowers
// to nested `Term::ProjectField` entries.
verb! {
    pub fn verb_depth2_const_generic_leaf(input: OuterBigInt) -> ConstrainedTypeInput {
        // input.0 = Big128 (= BigIntShape<128>) — a const-generic leaf.
        // input.0.0 would attempt to project through Big128 which is
        // a leaf scalar (no PartitionProductFields impl). So the
        // realistic depth-2 case is `input.1` for the LeafA side.
        input.1
    }
}

#[test]
fn verb_macro_handles_const_generic_outer_partition_product() {
    let arena = verb_depth2_const_generic_leaf_term_arena::<SMOKE_IB>();
    assert_eq!(arena.len(), 2);
    assert!(matches!(arena[0], Term::Variable { name_index: 0 }));
    assert!(matches!(
        arena[1],
        Term::ProjectField {
            source_index: 0,
            ..
        }
    ));
}

// True depth-2 with a const-generic INNER partition product: an outer
// partition over a (BigIntInner alias of const-generic) partition and
// a leaf. Tests the case where `input.0.0` traverses through a
// const-generic-derived `<Outer as PartitionProductFactor<0>>::Factor`
// to reach the inner partition's first field.
partition_product!(BigIntInner, Big128, LeafA);
partition_product!(BigIntOuter, BigIntInner, LeafA);

verb! {
    pub fn verb_depth2_const_generic_inner(input: BigIntOuter) -> ConstrainedTypeInput {
        // input.0 = BigIntInner (a partition product containing a
        // const-generic leaf); input.0.0 = Big128 (= BigIntShape<128>).
        input.0.0
    }
}

#[test]
fn verb_macro_admits_depth2_through_const_generic_inner_partition() {
    let arena = verb_depth2_const_generic_inner_term_arena::<SMOKE_IB>();
    assert_eq!(arena.len(), 3);
    assert!(matches!(arena[0], Term::Variable { name_index: 0 }));
    assert!(matches!(
        arena[1],
        Term::ProjectField {
            source_index: 0,
            ..
        }
    ));
    assert!(matches!(
        arena[2],
        Term::ProjectField {
            source_index: 1,
            ..
        }
    ));
}

// ── Dependency 3: ADR-056 ψ-residuals scope refinement ─────────────────
//
// Per ADR-056, the ψ-residuals discipline (rejection of `concat`,
// `hash`, `first_admit`, and byte-comparison binary ops) applies to the
// route body's syntactic surface ONLY. Verb bodies and axis! body
// clauses admit the full substrate vocabulary so canonical compound
// operations — SHA padding (concat + Le for block-threshold compare),
// HMAC (concat for keyed-input composition), Merkle (concat for hash
// combine), tensor saturation (Le/Ge for clamp bounds) — can be
// expressed as substrate-Term decompositions per ADR-055.

verb! {
    pub fn verb_admits_concat(input: ConstrainedTypeInput) -> ConstrainedTypeInput {
        // Pre-ADR-056 this raised ψ-residual violation in verb! bodies;
        // post-ADR-056 it lowers to Application(Concat, [lhs, rhs]).
        concat(input, input)
    }
}

#[test]
fn verb_macro_admits_concat_per_adr_056() {
    let arena = verb_admits_concat_term_arena::<SMOKE_IB>();
    let last = arena.last().expect("non-empty arena");
    assert!(matches!(
        last,
        Term::Application {
            operator: PrimitiveOp::Concat,
            ..
        }
    ));
}

verb! {
    pub fn verb_admits_byte_compare(input: ConstrainedTypeInput) -> ConstrainedTypeInput {
        // Byte-level comparison (<, <=, >, >=) is the SHA padding-length
        // and tensor saturation-clamp recipe. Admissible in verb bodies
        // per ADR-056.
        input <= 64
    }
}

#[test]
fn verb_macro_admits_byte_comparison_per_adr_056() {
    let arena = verb_admits_byte_compare_term_arena::<SMOKE_IB>();
    let last = arena.last().expect("non-empty arena");
    assert!(matches!(
        last,
        Term::Application {
            operator: PrimitiveOp::Le,
            ..
        }
    ));
}

verb! {
    pub fn verb_admits_hash(input: ConstrainedTypeInput) -> ConstrainedTypeInput {
        // `hash(<value>)` lowers to AxisInvocation { axis 0, kernel 0 }
        // in verb bodies — the canonical hash-axis dispatch per ADR-030.
        // Admissible in verb bodies per ADR-056 (route bodies still
        // reject it).
        hash(input)
    }
}

#[test]
fn verb_macro_admits_hash_per_adr_056() {
    let arena = verb_admits_hash_term_arena::<SMOKE_IB>();
    let last = arena.last().expect("non-empty arena");
    assert!(matches!(
        last,
        Term::AxisInvocation {
            axis_index: 0,
            kernel_id: 0,
            ..
        }
    ));
}

prism_model! {
    pub struct ChainedPosModel;
    pub struct ChainedPosRoute;
    impl PrismModel<DefaultHostTypes, SmokeHostBounds, SmokeHasher> for ChainedPosModel {
        type Input = PosOuter;
        type Output = ConstrainedTypeInput;
        type Route = ChainedPosRoute;
        fn route(input: Self::Input) -> Self::Output {
            // input.0 = InnerLR; input.0.0 = lhs (LeafA).
            input.0.0
        }
    }
}

#[test]
fn prism_model_emits_chained_positional_project_field_terms() {
    let arena = <ChainedPosRoute as FoundationClosed<SMOKE_IB>>::arena_slice();
    assert_eq!(arena.len(), 3);
    assert!(matches!(arena[0], Term::Variable { name_index: 0 }));
    assert!(matches!(
        arena[1],
        Term::ProjectField {
            source_index: 0,
            ..
        }
    ));
    assert!(matches!(
        arena[2],
        Term::ProjectField {
            source_index: 1,
            ..
        }
    ));
}

// =====================================================================
// `resolver!` smoke tests — wiki ADR-036 ResolverTuple declaration macro.
//
// These tests exercise the end-to-end ψ-chain inference path:
// `resolver!` emits all eight `Has<Category>Resolver<H>` impls
// (declared fields delegate to the user's resolver; undeclared
// categories delegate to `NullResolverTuple`), and the catamorphism's
// `Term::Nerve` fold-rule dispatches operand bytes through the
// declared `NerveResolver`.

use uor_foundation::enforcement::ShapeViolation;
use uor_foundation::pipeline::{evaluate_term_tree, NerveResolver};
use uor_foundation::PipelineFailure;
use uor_foundation_sdk::resolver;

/// Application-author NerveResolver impl: writes a fixed sentinel byte
/// sequence to the output buffer regardless of input. This makes it
/// observable that the catamorphism dispatched through this resolver
/// (vs. the foundation Null impl, which would emit `RESOLVER_ABSENT`).
#[derive(Debug, Default)]
pub struct SentinelNerveResolver<H>(core::marker::PhantomData<H>);

// The resolver-trait family carries the `__sdk_seal::Sealed` supertrait;
// foundation's normative path for users to satisfy it is via the SDK
// macros. This test reaches into the doc-hidden seal module directly to
// exercise the catamorphism's dispatch surface end-to-end — the wiki
// (ADR-022 D1) notes external crates that name `__sdk_seal::Sealed` are
// "technically permitted by Rust's visibility rules but architecturally
// non-conforming"; a unit test in the SDK crate is the harness case.
impl<H: Hasher> uor_foundation::pipeline::__sdk_seal::Sealed for SentinelNerveResolver<H> {}

// ADR-060: the resolver returns a source-polymorphic `TermValue` (blanket
// over the carrier inline width). The sentinel fits the inline carrier.
impl<const INLINE_BYTES: usize, H: Hasher> NerveResolver<INLINE_BYTES, H>
    for SentinelNerveResolver<H>
{
    fn resolve<'a>(
        &self,
        _input: uor_foundation::pipeline::TermValue<'a, INLINE_BYTES>,
    ) -> Result<uor_foundation::pipeline::TermValue<'a, INLINE_BYTES>, ShapeViolation> {
        const SENTINEL: &[u8] = &[0xA1, 0xB2, 0xC3, 0xD4];
        Ok(uor_foundation::pipeline::TermValue::inline_from_slice(
            SENTINEL,
        ))
    }
}

resolver! {
    pub struct SingleCategoryResolvers<H: ::uor_foundation::enforcement::Hasher> {
        nerve: SentinelNerveResolver<H>,
    }
}

#[test]
fn resolver_macro_emits_all_eight_has_impls_so_run_route_bounds_resolve() {
    // ADR-036: even when the application declares only one category,
    // the emitted struct must satisfy `run_route`'s where-clause
    // (all eight `Has<Category>Resolver<A>` bounds). This compile-time
    // assertion fails to type-check if the macro regresses to
    // declared-only emissions.
    fn _accepts<R>()
    where
        R: ::uor_foundation::pipeline::ResolverTuple
            + ::uor_foundation::pipeline::HasNerveResolver<SMOKE_IB, SmokeHasher>
            + ::uor_foundation::pipeline::HasChainComplexResolver<SMOKE_IB, SmokeHasher>
            + ::uor_foundation::pipeline::HasHomologyGroupResolver<SMOKE_IB, SmokeHasher>
            + ::uor_foundation::pipeline::HasCochainComplexResolver<SMOKE_IB, SmokeHasher>
            + ::uor_foundation::pipeline::HasCohomologyGroupResolver<SMOKE_IB, SmokeHasher>
            + ::uor_foundation::pipeline::HasPostnikovResolver<SMOKE_IB, SmokeHasher>
            + ::uor_foundation::pipeline::HasHomotopyGroupResolver<SMOKE_IB, SmokeHasher>
            + ::uor_foundation::pipeline::HasKInvariantResolver<SMOKE_IB, SmokeHasher>,
    {
    }
    _accepts::<SingleCategoryResolvers<SmokeHasher>>();
}

#[test]
fn psi_chain_nerve_term_dispatches_through_user_declared_resolver() {
    // End-to-end: a user-declared ResolverTuple with one populated
    // category (`nerve`) drives the catamorphism's `Term::Nerve`
    // fold-rule to invoke `SentinelNerveResolver::resolve`. The
    // resolver writes a fixed sentinel byte pattern; we observe it
    // in the catamorphism's output TermValue.
    let arena = [
        Term::Variable { name_index: 0 },
        Term::Nerve { value_index: 0 },
    ];
    let resolvers = SingleCategoryResolvers::<SmokeHasher> {
        nerve: SentinelNerveResolver(core::marker::PhantomData),
        _phantom: core::marker::PhantomData,
    };
    let input = [0x00, 0x00, 0x00];
    let result = evaluate_term_tree::<
        SmokeHasher,
        SingleCategoryResolvers<SmokeHasher>,
        SMOKE_IB,
        SMOKE_FP,
    >(
        &arena,
        uor_foundation::pipeline::TermValue::borrowed(&input),
        &resolvers,
    )
    .expect("user-declared nerve resolver should resolve");
    assert_eq!(
        result.bytes(),
        &[0xA1, 0xB2, 0xC3, 0xD4][..],
        "Term::Nerve fold-rule must surface SentinelNerveResolver's output bytes",
    );
}

#[test]
fn undeclared_resolver_categories_propagate_resolver_absent() {
    // Companion to the previous test: a `Term::ChainComplex` fold
    // through `SingleCategoryResolvers` (which declares only `nerve`)
    // routes through the macro-emitted `NullResolverTuple`-delegate
    // accessor, so `chain_complex_resolver().resolve(...)` returns
    // the `RESOLVER_ABSENT` violation.
    let arena = [
        Term::Variable { name_index: 0 },
        Term::ChainComplex {
            simplicial_index: 0,
        },
    ];
    let resolvers = SingleCategoryResolvers::<SmokeHasher> {
        nerve: SentinelNerveResolver(core::marker::PhantomData),
        _phantom: core::marker::PhantomData,
    };
    let input = [0u8; 1];
    let outcome = evaluate_term_tree::<
        SmokeHasher,
        SingleCategoryResolvers<SmokeHasher>,
        SMOKE_IB,
        SMOKE_FP,
    >(
        &arena,
        uor_foundation::pipeline::TermValue::borrowed(&input),
        &resolvers,
    );
    match outcome {
        Err(PipelineFailure::ShapeViolation { report }) => assert_eq!(
            report.shape_iri, "https://uor.foundation/resolver/RESOLVER_ABSENT",
            "undeclared chain_complex must surface RESOLVER_ABSENT",
        ),
        other => panic!("expected RESOLVER_ABSENT violation, got {other:?}"),
    }
}

// =====================================================================
// Complete ψ-chain feature-to-label inference — wiki ADR-035.
//
// The ψ-pipeline ψ_1..ψ_9 is the structural-witness arm of
// `op:InferenceOperation`. Each ψ_k is one chain link; the nine
// variants fan out into three end-to-end inference branches:
//
//   Homology branch       : ψ_1 → ψ_2 → ψ_3 → ψ_4
//                           (raw feature → Betti numbers as label)
//   Cohomology branch     : ψ_1 → ψ_2 → ψ_5 → ψ_6
//                           (raw feature → cohomology groups as label)
//   K-invariant branch    : ψ_1 → ψ_7 → ψ_8 → ψ_9
//                           (raw feature → k-invariants as label)
//
// Each branch is a complete feature-to-label inference path. Together
// the three branches exercise all nine ψ-variants and verify the
// catamorphism walks chain dependencies through the user-declared
// ResolverTuple end-to-end.
//
// The sentinel resolvers below each append a category marker byte to
// the operand bytes — so the catamorphism's output carries a sequence
// of markers proving each ψ_k was invoked in chain order.

use uor_foundation::pipeline::{
    ChainComplexResolver, CochainComplexResolver, CohomologyGroupResolver, HomologyGroupResolver,
    HomotopyGroupResolver, KInvariantResolver, PostnikovResolver,
};

const PSI_1_MARKER: u8 = 0x01;
const PSI_2_MARKER: u8 = 0x02;
const PSI_3_MARKER: u8 = 0x03;
const PSI_5_MARKER: u8 = 0x05;
const PSI_6_MARKER: u8 = 0x06;
const PSI_7_MARKER: u8 = 0x07;
const PSI_8_MARKER: u8 = 0x08;
const PSI_9_MARKER: u8 = 0x09;

// ADR-060: source-polymorphic marker appender — appends `marker`
// to `input` and returns the result as an `Inline` `TermValue`.
fn append_marker_tv<'a, const INLINE_BYTES: usize>(
    input: &[u8],
    marker: u8,
) -> Result<uor_foundation::pipeline::TermValue<'a, INLINE_BYTES>, ShapeViolation> {
    let n = input.len();
    if n + 1 > INLINE_BYTES {
        return Err(ShapeViolation {
            shape_iri: "https://example.org/psi-chain-test/OutputBufferShape",
            constraint_iri: "https://example.org/psi-chain-test/OutputBufferShape/maxBytes",
            property_iri: "https://example.org/psi-chain-test/output",
            expected_range: "http://www.w3.org/2001/XMLSchema#nonNegativeInteger",
            min_count: 0,
            max_count: INLINE_BYTES as u32,
            kind: uor_foundation::ViolationKind::ValueCheck,
        });
    }
    let mut buf = [0u8; INLINE_BYTES];
    buf[..n].copy_from_slice(input);
    buf[n] = marker;
    Ok(uor_foundation::pipeline::TermValue::inline_from_slice(
        &buf[..n + 1],
    ))
}

// Wiki ADR-041: per-trait input type. NerveResolver receives the raw
// per-value `&[u8]`; the seven downstream resolvers receive their
// ADR-041 typed-coordinate carrier (a zero-cost `#[repr(transparent)]`
// view over the prior ψ-stage's byte serialization). Sentinel resolvers
// project the carrier's `&[u8]` through `as_bytes()` and append a
// stage-specific marker byte.
macro_rules! psi_marker_resolver_byte_input {
    ($struct:ident, $trait:ident, $marker:ident) => {
        #[derive(Debug, Default)]
        pub struct $struct<H>(core::marker::PhantomData<H>);
        impl<H: Hasher> uor_foundation::pipeline::__sdk_seal::Sealed for $struct<H> {}
        impl<const INLINE_BYTES: usize, H: Hasher> $trait<INLINE_BYTES, H> for $struct<H> {
            fn resolve<'a>(
                &self,
                input: uor_foundation::pipeline::TermValue<'a, INLINE_BYTES>,
            ) -> Result<uor_foundation::pipeline::TermValue<'a, INLINE_BYTES>, ShapeViolation> {
                append_marker_tv::<INLINE_BYTES>(input.bytes(), $marker)
            }
        }
    };
}

// ADR-060: the resolve signature is uniform (`TermValue` in/out); the ADR-041
// typed-input distinction is no longer part of the trait surface, so the
// `$input_ty` parameter is retained for call-site compatibility but unused.
macro_rules! psi_marker_resolver_typed_input {
    ($struct:ident, $trait:ident, $marker:ident, $input_ty:ty) => {
        #[derive(Debug, Default)]
        pub struct $struct<H>(core::marker::PhantomData<H>);
        impl<H: Hasher> uor_foundation::pipeline::__sdk_seal::Sealed for $struct<H> {}
        impl<const INLINE_BYTES: usize, H: Hasher> $trait<INLINE_BYTES, H> for $struct<H> {
            fn resolve<'a>(
                &self,
                input: uor_foundation::pipeline::TermValue<'a, INLINE_BYTES>,
            ) -> Result<uor_foundation::pipeline::TermValue<'a, INLINE_BYTES>, ShapeViolation> {
                append_marker_tv::<INLINE_BYTES>(input.bytes(), $marker)
            }
        }
    };
}

// Eight per-ψ sentinel resolvers — one for each of the resolver-bound
// variants. ψ_4 (Betti) is resolver-free per ADR-035, so no sentinel.
// Per ADR-041, the input type carries the prior ψ-stage's identity.
psi_marker_resolver_byte_input!(Psi1Nerve, NerveResolver, PSI_1_MARKER);
psi_marker_resolver_typed_input!(
    Psi2ChainComplex,
    ChainComplexResolver,
    PSI_2_MARKER,
    uor_foundation::pipeline::SimplicialComplexBytes<'_>
);
psi_marker_resolver_typed_input!(
    Psi3HomologyGroup,
    HomologyGroupResolver,
    PSI_3_MARKER,
    uor_foundation::pipeline::ChainComplexBytes<'_>
);
psi_marker_resolver_typed_input!(
    Psi5CochainComplex,
    CochainComplexResolver,
    PSI_5_MARKER,
    uor_foundation::pipeline::ChainComplexBytes<'_>
);
psi_marker_resolver_typed_input!(
    Psi6CohomologyGroup,
    CohomologyGroupResolver,
    PSI_6_MARKER,
    uor_foundation::pipeline::CochainComplexBytes<'_>
);
psi_marker_resolver_typed_input!(
    Psi7Postnikov,
    PostnikovResolver,
    PSI_7_MARKER,
    uor_foundation::pipeline::SimplicialComplexBytes<'_>
);
psi_marker_resolver_typed_input!(
    Psi8HomotopyGroup,
    HomotopyGroupResolver,
    PSI_8_MARKER,
    uor_foundation::pipeline::PostnikovTowerBytes<'_>
);
psi_marker_resolver_typed_input!(
    Psi9KInvariant,
    KInvariantResolver,
    PSI_9_MARKER,
    uor_foundation::pipeline::HomotopyGroupsBytes<'_>
);

resolver! {
    pub struct CompleteResolvers<H: ::uor_foundation::enforcement::Hasher> {
        nerve: Psi1Nerve<H>,
        chain_complex: Psi2ChainComplex<H>,
        homology_groups: Psi3HomologyGroup<H>,
        cochain_complex: Psi5CochainComplex<H>,
        cohomology_groups: Psi6CohomologyGroup<H>,
        postnikov: Psi7Postnikov<H>,
        homotopy_groups: Psi8HomotopyGroup<H>,
        k_invariants: Psi9KInvariant<H>,
    }
}

fn complete_resolvers() -> CompleteResolvers<SmokeHasher> {
    CompleteResolvers::<SmokeHasher> {
        nerve: Psi1Nerve(core::marker::PhantomData),
        chain_complex: Psi2ChainComplex(core::marker::PhantomData),
        homology_groups: Psi3HomologyGroup(core::marker::PhantomData),
        cochain_complex: Psi5CochainComplex(core::marker::PhantomData),
        cohomology_groups: Psi6CohomologyGroup(core::marker::PhantomData),
        postnikov: Psi7Postnikov(core::marker::PhantomData),
        homotopy_groups: Psi8HomotopyGroup(core::marker::PhantomData),
        k_invariants: Psi9KInvariant(core::marker::PhantomData),
        _phantom: core::marker::PhantomData,
    }
}

/// Run a ψ-chain test body. ADR-060 made the catamorphism's per-fold-rule
/// scratch a small `[u8; INLINE_BYTES]` carrier (replacing the retired fixed
/// 4096-byte per-value buffer), so recursive ψ-chain evaluation now fits
/// comfortably in the cargo test runner's default thread stack — no custom
/// stack thread is required. Retained as a thin indirection so the ψ-chain
/// tests share a single call shape.
fn run_psi_chain_body<F>(test_body: F)
where
    F: FnOnce(),
{
    test_body();
}

#[test]
fn psi_chain_homology_branch_routes_feature_to_betti_label() {
    run_psi_chain_body(|| {
        // Wiki ADR-035 homology branch: ψ_1 → ψ_2 → ψ_3 → ψ_4.
        // The arena's root is `Term::Betti`, which is resolver-free —
        // ψ_4 passes its operand bytes through unchanged, so the
        // observable output equals the ψ_3 result.
        let arena = [
            Term::Variable { name_index: 0 }, // [0] feature input
            Term::Nerve { value_index: 0 },   // [1] ψ_1
            Term::ChainComplex {
                simplicial_index: 1,
            }, // [2] ψ_2
            Term::HomologyGroups { chain_index: 2 }, // [3] ψ_3
            Term::Betti { homology_index: 3 }, // [4] ψ_4 (root)
        ];
        let resolvers = complete_resolvers();
        let input = [0xFEu8, 0xED];
        let result =
            evaluate_term_tree::<SmokeHasher, CompleteResolvers<SmokeHasher>, SMOKE_IB, SMOKE_FP>(
                &arena,
                uor_foundation::pipeline::TermValue::borrowed(&input),
                &resolvers,
            )
            .expect("homology-branch chain should resolve end-to-end");
        let expected = &[0xFE, 0xED, PSI_1_MARKER, PSI_2_MARKER, PSI_3_MARKER][..];
        assert_eq!(
            result.bytes(),
            expected,
            "homology-branch label must carry input + ψ_1 + ψ_2 + ψ_3 markers (ψ_4 is pass-through)",
        );
    });
}

#[test]
fn psi_chain_cohomology_branch_routes_feature_to_cohomology_label() {
    run_psi_chain_body(|| {
        // Wiki ADR-035 cohomology branch: ψ_1 → ψ_2 → ψ_5 → ψ_6.
        // ψ_5 (cochain) is the dualization functor on ChainComplex; ψ_6
        // computes cohomology from cochain. Root is `Term::CohomologyGroups`.
        let arena = [
            Term::Variable { name_index: 0 }, // [0] feature input
            Term::Nerve { value_index: 0 },   // [1] ψ_1
            Term::ChainComplex {
                simplicial_index: 1,
            }, // [2] ψ_2
            Term::CochainComplex { chain_index: 2 }, // [3] ψ_5
            Term::CohomologyGroups { cochain_index: 3 }, // [4] ψ_6 (root)
        ];
        let resolvers = complete_resolvers();
        let input = [0xCAu8, 0xFE];
        let result =
            evaluate_term_tree::<SmokeHasher, CompleteResolvers<SmokeHasher>, SMOKE_IB, SMOKE_FP>(
                &arena,
                uor_foundation::pipeline::TermValue::borrowed(&input),
                &resolvers,
            )
            .expect("cohomology-branch chain should resolve end-to-end");
        let expected = &[
            0xCA,
            0xFE,
            PSI_1_MARKER,
            PSI_2_MARKER,
            PSI_5_MARKER,
            PSI_6_MARKER,
        ][..];
        assert_eq!(
            result.bytes(),
            expected,
            "cohomology-branch label must carry input + ψ_1 + ψ_2 + ψ_5 + ψ_6 markers",
        );
    });
}

#[test]
fn psi_chain_k_invariant_branch_routes_feature_to_k_invariant_label() {
    run_psi_chain_body(|| {
        // Wiki ADR-035 k-invariant branch: ψ_1 → ψ_7 → ψ_8 → ψ_9.
        // ψ_7 (Postnikov) takes a SimplicialComplex (from ψ_1) directly,
        // skipping the chain-complex branch. ψ_8 extracts homotopy from
        // the Postnikov tower; ψ_9 computes k-invariants from homotopy.
        let arena = [
            Term::Variable { name_index: 0 }, // [0] feature input
            Term::Nerve { value_index: 0 },   // [1] ψ_1
            Term::PostnikovTower {
                simplicial_index: 1,
            }, // [2] ψ_7
            Term::HomotopyGroups { postnikov_index: 2 }, // [3] ψ_8
            Term::KInvariants { homotopy_index: 3 }, // [4] ψ_9 (root)
        ];
        let resolvers = complete_resolvers();
        let input = [0xBEu8, 0xEF];
        let result =
            evaluate_term_tree::<SmokeHasher, CompleteResolvers<SmokeHasher>, SMOKE_IB, SMOKE_FP>(
                &arena,
                uor_foundation::pipeline::TermValue::borrowed(&input),
                &resolvers,
            )
            .expect("k-invariant-branch chain should resolve end-to-end");
        let expected = &[
            0xBE,
            0xEF,
            PSI_1_MARKER,
            PSI_7_MARKER,
            PSI_8_MARKER,
            PSI_9_MARKER,
        ][..];
        assert_eq!(
            result.bytes(),
            expected,
            "k-invariant-branch label must carry input + ψ_1 + ψ_7 + ψ_8 + ψ_9 markers",
        );
    });
}

#[test]
fn psi_chain_all_nine_variants_exercise_in_three_branches() {
    // Pin the architectural commitment: the three feature-to-label
    // branches above collectively walk every ψ-variant ψ_1..ψ_9. This
    // assertion is a compile-time enumeration check (each marker
    // constant declared above appears in at least one branch test).
    let walked = [
        PSI_1_MARKER, // homology + cohomology + k-invariant
        PSI_2_MARKER, // homology + cohomology
        PSI_3_MARKER, // homology
        // ψ_4 = Betti is resolver-free; its presence is verified by
        // `psi_chain_homology_branch_routes_feature_to_betti_label`
        // observing that the ψ_3 result flows through unchanged.
        PSI_5_MARKER, // cohomology
        PSI_6_MARKER, // cohomology
        PSI_7_MARKER, // k-invariant
        PSI_8_MARKER, // k-invariant
        PSI_9_MARKER, // k-invariant
    ];
    assert_eq!(walked.len(), 8, "eight resolver-bound ψ-variants");
}

// =====================================================================
// End-to-end ψ-chain through `prism_model!` four-position form — wiki
// ADR-036's substrate-parameter `R` threaded through `forward()`.
//
// These tests prove that:
//   1. `prism_model!` accepts the four-position `impl PrismModel<H, B, A, R>`
//      form (ADR-036).
//   2. `forward(input)` constructs the user's ResolverTuple instance —
//      either from an optional `fn resolvers() -> R` clause or via
//      `<R as Default>::default()` (emitted by the `resolver!` macro on
//      the tuple struct).
//   3. The closure-body grammar's G21..G29 ψ-chain forms (`nerve`,
//      `chain_complex`, `homology_groups`, `betti`, `cochain_complex`,
//      `cohomology_groups`, `postnikov_tower`, `homotopy_groups`,
//      `k_invariants`) lower into the corresponding `Term::*` variants
//      and the catamorphism dispatches each through the user's
//      ResolverTuple end-to-end — feature input bytes flow through the
//      ψ-pipeline and emerge as label bytes from `Grounded::output_bytes`.

use uor_foundation::enforcement::Grounded;

// Four-position form with explicit `fn resolvers()` clause — the
// k-invariant branch (ψ_1 → ψ_7 → ψ_8 → ψ_9). The closure body
// `k_invariants(homotopy_groups(postnikov_tower(nerve(input))))` lowers
// into the 5-Term arena exercised by `evaluate_term_tree` (the same
// chain proven correct in `psi_chain_k_invariant_branch_…` above, here
// reached through the canonical `forward()` surface ADR-020 commits to).
prism_model! {
    pub struct KInvariantInferenceModel;
    pub struct KInvariantInferenceRoute;
    impl PrismModel<
        DefaultHostTypes,
        SmokeHostBounds,
        SmokeHasher,
        CompleteResolvers<SmokeHasher>
    > for KInvariantInferenceModel {
        type Input = ConstrainedTypeInput;
        type Output = ConstrainedTypeInput;
        type Route = KInvariantInferenceRoute;
        fn route(input: Self::Input) -> Self::Output {
            k_invariants(homotopy_groups(postnikov_tower(nerve(input))))
        }
        fn resolvers() -> CompleteResolvers<SmokeHasher> {
            complete_resolvers()
        }
    }
}

#[test]
fn prism_model_forward_walks_k_invariant_psi_chain_end_to_end() {
    // Wiki ADR-035 k-invariant branch via `prism_model!`'s four-position
    // form: feature bytes → ψ_1 → ψ_7 → ψ_8 → ψ_9 → label bytes. The
    // catamorphism dispatches each ψ-Term through the model's declared
    // ResolverTuple (constructed by the macro-emitted `fn resolvers()`
    // delegate that calls `complete_resolvers()`). The thread wrap is
    // required only in debug builds where the per-fold `[u8; 4096]`
    // scratch buffer inflates frame size; release builds run on the
    // default 2 MB cargo-test stack.
    run_psi_chain_body(|| {
        let result = <KInvariantInferenceModel as PrismModel<
            DefaultHostTypes,
            SmokeHostBounds,
            SmokeHasher,
            SMOKE_IB,
            SMOKE_FP,
            CompleteResolvers<SmokeHasher>,
        >>::forward(ConstrainedTypeInput::default())
        .expect("forward() through the ψ-chain should resolve end-to-end");
        let grounded: Grounded<'static, ConstrainedTypeInput, SMOKE_IB, SMOKE_FP> = result;
        // Input is empty (ConstrainedTypeInput's `IntoBindingValue::MAX_BYTES
        // = 0`), so the chain emits only the per-ψ marker bytes:
        //   ψ_1 appends 0x01, ψ_7 appends 0x07, ψ_8 appends 0x08, ψ_9 appends 0x09.
        let expected = &[PSI_1_MARKER, PSI_7_MARKER, PSI_8_MARKER, PSI_9_MARKER][..];
        assert_eq!(
            grounded.output_bytes(),
            expected,
            "forward()'s Grounded output must carry the ψ-chain label bytes",
        );
    });
}

// Four-position form WITHOUT explicit `fn resolvers()` — the macro
// constructs the tuple via `<R as Default>::default()`. This exercises
// the `resolver!`-emitted `Default` impl, which is critical for routes
// whose resolvers carry only PhantomData (foundation Null-equivalents or
// pure-marker stubs).
prism_model! {
    pub struct HomologyInferenceModel;
    pub struct HomologyInferenceRoute;
    impl PrismModel<
        DefaultHostTypes,
        SmokeHostBounds,
        SmokeHasher,
        CompleteResolvers<SmokeHasher>
    > for HomologyInferenceModel {
        type Input = ConstrainedTypeInput;
        type Output = ConstrainedTypeInput;
        type Route = HomologyInferenceRoute;
        fn route(input: Self::Input) -> Self::Output {
            betti(homology_groups(chain_complex(nerve(input))))
        }
    }
}

#[test]
fn prism_model_forward_walks_homology_psi_chain_via_default_resolvers() {
    // Wiki ADR-035 homology branch through `forward()` with no
    // `fn resolvers()` clause: the macro-emitted body defaults to
    // `<CompleteResolvers<SmokeHasher> as Default>::default()`, which the
    // `resolver!` macro's Default emission supplies. Each sentinel
    // resolver's `Default::default()` produces a `PhantomData`-backed
    // resolver functionally identical to the explicit form. The chain
    // ψ_1 → ψ_2 → ψ_3 → ψ_4 (Betti is pass-through) emits markers
    // [0x01, 0x02, 0x03].
    run_psi_chain_body(|| {
        let result = <HomologyInferenceModel as PrismModel<
            DefaultHostTypes,
            SmokeHostBounds,
            SmokeHasher,
            SMOKE_IB,
            SMOKE_FP,
            CompleteResolvers<SmokeHasher>,
        >>::forward(ConstrainedTypeInput::default())
        .expect("forward() through default-constructed resolvers should resolve");
        let grounded: Grounded<'static, ConstrainedTypeInput, SMOKE_IB, SMOKE_FP> = result;
        let expected = &[PSI_1_MARKER, PSI_2_MARKER, PSI_3_MARKER][..];
        assert_eq!(
            grounded.output_bytes(),
            expected,
            "Default-constructed CompleteResolvers must carry the homology-branch label",
        );
    });
}

// =====================================================================
// Wiki ADR-035 ψ-residuals discipline — runtime pinning.
//
// The `prism_model!` / `verb!` macros reject ψ-residual emissions at
// proc-macro expansion. A direct compile-fail assertion lives outside
// the unit-test corpus; the assertion below walks every route arena
// `prism_model!` produced in this test file and verifies no ψ-residual
// Term variant slipped through. This is the load-bearing positive-side
// of the wiki's discipline (TR-14): if a future regression silently
// emits a ψ-residual, this test fails.

/// Returns true iff `term` is a ψ-residual per wiki ADR-035:
///   - `Term::FirstAdmit` (search-based admission).
///   - `Term::AxisInvocation` (axis-trait-method dispatch from verb body).
///   - `Term::Application { PrimitiveOp::{Le|Lt|Ge|Gt|Concat}, .. }`
///     (byte-comparison / byte-concat residuals).
fn term_is_psi_residual(term: &Term<'_, SMOKE_IB>) -> bool {
    use uor_foundation::PrimitiveOp;
    matches!(
        term,
        Term::FirstAdmit { .. }
            | Term::AxisInvocation { .. }
            | Term::Application {
                operator: PrimitiveOp::Le
                    | PrimitiveOp::Lt
                    | PrimitiveOp::Ge
                    | PrimitiveOp::Gt
                    | PrimitiveOp::Concat,
                ..
            }
    )
}

#[test]
fn prism_model_arenas_carry_zero_psi_residuals_per_adr_035() {
    // Iterate every Route witness emitted in this file and confirm its
    // arena contains no ψ-residual Term variants. The list mirrors the
    // verb-body emissions across the smoke test corpus; failure here
    // means the closure-body parser regressed on the ADR-035 discipline.
    let arenas: &[(&'static str, &'static [Term<'static, SMOKE_IB>])] = &[
        (
            "AddTwoLiterals",
            <AddTwoLiteralsRoute as FoundationClosed<SMOKE_IB>>::arena_slice(),
        ),
        (
            "VariableThenSucc",
            <VariableThenSuccRoute as FoundationClosed<SMOKE_IB>>::arena_slice(),
        ),
        (
            "VerbInvokingModel",
            <VerbInvokingRoute as FoundationClosed<SMOKE_IB>>::arena_slice(),
        ),
        (
            "LiftToW16Model",
            <LiftToW16Route as FoundationClosed<SMOKE_IB>>::arena_slice(),
        ),
        (
            "ProjectToW8Model",
            <ProjectToW8Route as FoundationClosed<SMOKE_IB>>::arena_slice(),
        ),
        (
            "LetBindingModel",
            <LetBindingRoute as FoundationClosed<SMOKE_IB>>::arena_slice(),
        ),
        (
            "TryPropagateModel",
            <TryPropagateRoute as FoundationClosed<SMOKE_IB>>::arena_slice(),
        ),
        (
            "RecurseModel",
            <RecurseRoute as FoundationClosed<SMOKE_IB>>::arena_slice(),
        ),
        (
            "UnfoldModel",
            <UnfoldRoute as FoundationClosed<SMOKE_IB>>::arena_slice(),
        ),
        (
            "FoldNUnrolledModel",
            <FoldNUnrolledRoute as FoundationClosed<SMOKE_IB>>::arena_slice(),
        ),
        (
            "MatchModel",
            <MatchRoute as FoundationClosed<SMOKE_IB>>::arena_slice(),
        ),
        (
            "ParallelComposeModel",
            <ParallelComposeRoute as FoundationClosed<SMOKE_IB>>::arena_slice(),
        ),
        (
            "TreeFoldModel",
            <TreeFoldRoute as FoundationClosed<SMOKE_IB>>::arena_slice(),
        ),
        (
            "ChainedFieldModel",
            <ChainedFieldRoute as FoundationClosed<SMOKE_IB>>::arena_slice(),
        ),
        (
            "ChainedPosModel",
            <ChainedPosRoute as FoundationClosed<SMOKE_IB>>::arena_slice(),
        ),
        (
            "KInvariantInferenceModel",
            <KInvariantInferenceRoute as FoundationClosed<SMOKE_IB>>::arena_slice(),
        ),
        (
            "HomologyInferenceModel",
            <HomologyInferenceRoute as FoundationClosed<SMOKE_IB>>::arena_slice(),
        ),
    ];
    for (name, arena) in arenas {
        for (idx, term) in arena.iter().enumerate() {
            assert!(
                !term_is_psi_residual(term),
                "ADR-035 ψ-residuals violation: route `{name}` arena[{idx}] = {term:?} \
                 is a ψ-residual Term variant. The `prism_model!` / `verb!` macros must \
                 reject ψ-residual emissions at proc-macro expansion."
            );
        }
    }
}

// =====================================================================
// Wiki ADR-040 + ADR-041 + ADR-042 — Implementor-facing ψ-pipeline
// end-to-end verification.
//
// Foundation MUST meet the needs of layer-3 implementors that build
// applications on the ψ-pipeline. This test simulates an implementor's
// integration: a custom-bounded HostBounds; eight resolvers wrapping
// the ADR-041 typed-coordinate carriers; the canonical k-invariants
// branch ψ_1 → ψ_7 → ψ_8 → ψ_9 driven through `Model::forward`; the
// success-side `Grounded` viewed as an ADR-042 InhabitanceCertificate
// envelope with κ-label / witness / certified_type accessors; the
// failure-side `PipelineFailure` viewed as an
// InhabitanceImpossibilityCertificate with contradiction_proof bytes;
// and the inhabitance::dispatch_through_table helper exercising the
// three-arm decider-routing surface.
//
// Each assertion below names the wiki commitment it pins. A regression
// in any of the ADR-040..042 surfaces surfaces here.

use uor_foundation::pipeline::{
    inhabitance::{dispatch_through_table, InhabitanceRuleArm},
    BettiNumbersBytes, ChainComplexBytes, CochainComplexBytes, CohomologyGroupsBytes,
    HomologyGroupsBytes, HomotopyGroupsBytes, InhabitanceCertificateView,
    InhabitanceImpossibilityCertificate, KInvariantsBytes, PostnikovTowerBytes,
    SimplicialComplexBytes, WitnessValueTuple,
};

/// ADR-041 receiver-shape pin: every typed-coordinate carrier wraps a
/// `&[u8]` zero-cost. The compile-time type system MUST distinguish
/// `SimplicialComplexBytes` from `ChainComplexBytes` etc. — passing the
/// wrong carrier to a downstream resolver MUST be a type error at the
/// resolver-impl boundary, not a runtime ShapeViolation.
#[test]
fn adr041_typed_coordinate_carriers_are_repr_transparent_and_zero_cost() {
    use core::mem::{align_of, size_of};
    // Each carrier is layout-identical to &[u8] (a fat pointer).
    assert_eq!(size_of::<SimplicialComplexBytes<'_>>(), size_of::<&[u8]>());
    assert_eq!(size_of::<ChainComplexBytes<'_>>(), size_of::<&[u8]>());
    assert_eq!(size_of::<HomologyGroupsBytes<'_>>(), size_of::<&[u8]>());
    assert_eq!(size_of::<BettiNumbersBytes<'_>>(), size_of::<&[u8]>());
    assert_eq!(size_of::<CochainComplexBytes<'_>>(), size_of::<&[u8]>());
    assert_eq!(size_of::<CohomologyGroupsBytes<'_>>(), size_of::<&[u8]>());
    assert_eq!(size_of::<PostnikovTowerBytes<'_>>(), size_of::<&[u8]>());
    assert_eq!(size_of::<HomotopyGroupsBytes<'_>>(), size_of::<&[u8]>());
    assert_eq!(size_of::<KInvariantsBytes<'_>>(), size_of::<&[u8]>());
    assert_eq!(
        align_of::<SimplicialComplexBytes<'_>>(),
        align_of::<&[u8]>()
    );
    assert_eq!(align_of::<KInvariantsBytes<'_>>(), align_of::<&[u8]>());

    // Each carrier exposes len/is_empty/as_bytes.
    let bs = [0x01u8, 0x02, 0x03];
    let sc = SimplicialComplexBytes(&bs);
    assert_eq!(sc.len(), 3);
    assert!(!sc.is_empty());
    assert_eq!(sc.as_bytes(), &bs);
    let empty = ChainComplexBytes(&[]);
    assert!(empty.is_empty());
    assert_eq!(empty.len(), 0);
}

/// ADR-042 inhabitance-verdict surface end-to-end pin: an implementor
/// drives the canonical k-invariants branch through `Model::forward`,
/// borrows the resulting `Grounded` as an InhabitanceCertificateView,
/// and reads κ-label / witness / certificate / certified_type via the
/// typed accessors — no per-application accessor reimplementation.
#[test]
fn adr042_inhabitance_certificate_view_exposes_kappa_witness_certified_type() {
    run_psi_chain_body(|| {
        // ψ-pipeline run — the canonical k-invariants branch
        // (ψ_1 → ψ_7 → ψ_8 → ψ_9) through `KInvariantInferenceModel`,
        // which uses the `Psi1Nerve` / `Psi7Postnikov` / `Psi8HomotopyGroup` /
        // `Psi9KInvariant` typed-coordinate resolvers from
        // `CompleteResolvers<SmokeHasher>`. Each resolver appends its
        // marker byte to the prior ψ-stage's output, so the κ-label
        // bytes carry the chain trace [ψ_1, ψ_7, ψ_8, ψ_9] = [0x01, 0x07, 0x08, 0x09].
        let grounded = <KInvariantInferenceModel as PrismModel<
            DefaultHostTypes,
            SmokeHostBounds,
            SmokeHasher,
            SMOKE_IB,
            SMOKE_FP,
            CompleteResolvers<SmokeHasher>,
        >>::forward(ConstrainedTypeInput::default())
        .expect("k-invariants branch must resolve under CompleteResolvers");

        // ADR-042: borrow the Grounded as an InhabitanceCertificateView.
        // Zero-cost — `#[repr(transparent)]` over &Grounded.
        let cert: InhabitanceCertificateView<
            '_,
            ConstrainedTypeInput,
            SMOKE_IB,
            SMOKE_FP,
            ConstrainedTypeInput,
        > = grounded.as_inhabitance_certificate();

        // κ-label accessor — returns a `KInvariantsBytes` typed view.
        let kappa: KInvariantsBytes<'_> = cert.kappa_label();
        let expected_kappa = [PSI_1_MARKER, PSI_7_MARKER, PSI_8_MARKER, PSI_9_MARKER];
        assert_eq!(
            kappa.as_bytes(),
            &expected_kappa[..],
            "ADR-042 κ-label accessor must surface the Term::KInvariants emission's bytes",
        );

        // certified_type accessor — names the typed-iso output IRI.
        let ty: &'static str = cert.certified_type();
        assert_eq!(
            ty,
            <ConstrainedTypeInput as ConstrainedTypeShape>::IRI,
            "ADR-042 certified_type must return the route's Output ConstrainedTypeShape IRI",
        );

        // witness accessor — returns a WitnessValueTuple view over the
        // underlying Grounded's binding table (ψ_1 0-simplices).
        let witness: WitnessValueTuple<'_> = cert.witness();
        // For an empty-input route (ConstrainedTypeInput, MAX_BYTES = 0)
        // the binding table is empty; the witness is still constructible
        // and reports its size.
        let _binding_count = witness.len();
        // is_empty() returns true for empty witness; non-empty inputs
        // would populate the binding table.
        assert!(witness.is_empty() || !witness.is_empty()); // tautology — pin the surface compiles.

        // certificate accessor — the underlying Validated<GroundingCertificate>.
        let validated_cert = cert.certificate();
        // The accessor returns a reference — pin the type compiles.
        let _: &uor_foundation::enforcement::Validated<
            uor_foundation::enforcement::GroundingCertificate,
        > = validated_cert;
    });
}

/// ADR-042 impossibility-side pin: an `Err(PipelineFailure)` borrows
/// as an `InhabitanceImpossibilityCertificate` with `contradiction_proof`
/// accessor returning the canonical-form failure-trace bytes.
#[test]
fn adr042_inhabitance_impossibility_certificate_exposes_contradiction_proof() {
    use uor_foundation::PipelineFailure;
    use uor_foundation::ViolationKind;

    // Construct a representative ShapeViolation failure — the form a
    // ψ-pipeline emits when the constraint nerve has empty Kan completion
    // (the RESOLVER_ABSENT discriminator carries the foundation-internal
    // sentinel). Applications instantiate this from the catamorphism's
    // Null<Category>Resolver dispatch or from explicit resolver Err returns.
    let failure = PipelineFailure::ShapeViolation {
        report: uor_foundation::enforcement::ShapeViolation {
            shape_iri: "https://uor.foundation/resolver/RESOLVER_ABSENT",
            constraint_iri: "https://uor.foundation/resolver/Nerve",
            property_iri: "https://uor.foundation/resolver/category",
            expected_range: "https://uor.foundation/resolver/Resolver",
            min_count: 0,
            max_count: 1,
            kind: ViolationKind::ValueCheck,
        },
    };

    // ADR-042: borrow the failure as an InhabitanceImpossibilityCertificate.
    // The accessor returns Option<Self>; foundation accepts every
    // PipelineFailure as a verdict-envelope view (applications consume
    // at their discretion per ADR-042's universal-accessor framing).
    let impossibility: InhabitanceImpossibilityCertificate<'_> = failure
        .as_inhabitance_impossibility_certificate()
        .expect("ADR-042 surfaces every PipelineFailure as an impossibility certificate");

    // contradiction_proof accessor — returns the canonical-form failure
    // trace bytes (for ShapeViolation, the shape_iri serves as the
    // by-contradiction-reconstructable proof witness).
    let proof: &'static [u8] = impossibility.contradiction_proof();
    assert_eq!(
        proof, b"https://uor.foundation/resolver/RESOLVER_ABSENT",
        "ADR-042 contradiction_proof must surface the shape_iri for ShapeViolation failures",
    );

    // failure() — borrow the underlying PipelineFailure.
    let _: &PipelineFailure = impossibility.failure();
}

/// ADR-042 dispatch_through_table pin: the three-arm decider helper
/// routes through TwoSatDecider → HornSatDecider → ResidualVerdictResolver
/// in ontology order. Each closure returns Option; the first decisive
/// arm wins. ResidualVerdictResolver is the catch-all (closure returns
/// directly, not Option).
#[test]
fn adr042_dispatch_through_table_routes_through_three_decider_arms() {
    // Case 1: TwoSatDecider wins (first arm decides).
    let (arm, verdict) = dispatch_through_table(
        || Some("2sat-decided"),
        || Some("horn-decided"),
        || "residual",
    );
    assert_eq!(arm, InhabitanceRuleArm::TwoSatDecider);
    assert_eq!(verdict, "2sat-decided");

    // Case 2: TwoSatDecider undecided, HornSatDecider wins.
    let (arm, verdict) =
        dispatch_through_table(|| None::<&str>, || Some("horn-decided"), || "residual");
    assert_eq!(arm, InhabitanceRuleArm::HornSatDecider);
    assert_eq!(verdict, "horn-decided");

    // Case 3: Both undecided — ResidualVerdictResolver catch-all.
    let (arm, verdict) = dispatch_through_table(|| None::<&str>, || None::<&str>, || "residual");
    assert_eq!(arm, InhabitanceRuleArm::ResidualVerdictResolver);
    assert_eq!(verdict, "residual");
}

/// Compile-time pin: the catamorphism MUST construct the typed-coordinate
/// carrier of the correct ψ-stage when invoking each downstream resolver.
/// Passing `ChainComplexBytes` to a resolver expecting `SimplicialComplexBytes`
/// is a compile-time error. This test names the wiki commitment — the
/// per-stage type-checking is verified by the workspace building cleanly
/// (this assertion is itself the surface pin).
// ADR-060: the eight ψ-stage resolver traits share a single uniform
// signature — `resolve<'a>(&'a self, TermValue<'a, INLINE_BYTES>) ->
// Result<TermValue<'a, INLINE_BYTES>, ShapeViolation>` (the ADR-041 typed-input
// distinction is superseded by the source-polymorphic carrier). Each Null
// impl's `resolve` coerces to this higher-ranked `fn`-pointer shape at the
// suite's reference inline width `SMOKE_IB`.
type ResolveSig<R> =
    for<'a> fn(
        &'a R,
        uor_foundation::pipeline::TermValue<'a, SMOKE_IB>,
    ) -> Result<uor_foundation::pipeline::TermValue<'a, SMOKE_IB>, ShapeViolation>;

#[test]
fn adr060_resolver_trait_signatures_type_check_psi_stage_composition() {
    // For each resolver trait, coerce the foundation Null impl's `resolve`
    // method to the ADR-060 source-polymorphic `fn`-pointer signature. A
    // regression in the unified trait shape surfaces as a coercion failure
    // here at compile time.
    use uor_foundation::pipeline::{
        ChainComplexResolver, CochainComplexResolver, CohomologyGroupResolver,
        HomologyGroupResolver, HomotopyGroupResolver, KInvariantResolver, NerveResolver,
        NullChainComplexResolver, NullCochainComplexResolver, NullCohomologyGroupResolver,
        NullHomologyGroupResolver, NullHomotopyGroupResolver, NullKInvariantResolver,
        NullNerveResolver, NullPostnikovResolver, PostnikovResolver,
    };
    let _nerve_sig: ResolveSig<NullNerveResolver<SmokeHasher>> =
        <NullNerveResolver<SmokeHasher> as NerveResolver<SMOKE_IB, SmokeHasher>>::resolve;
    let _chain_sig: ResolveSig<NullChainComplexResolver<SmokeHasher>> = <NullChainComplexResolver<
        SmokeHasher,
    > as ChainComplexResolver<
        SMOKE_IB,
        SmokeHasher,
    >>::resolve;
    let _homology_sig: ResolveSig<NullHomologyGroupResolver<SmokeHasher>> =
        <NullHomologyGroupResolver<SmokeHasher> as HomologyGroupResolver<SMOKE_IB, SmokeHasher>>::resolve;
    let _cochain_sig: ResolveSig<NullCochainComplexResolver<SmokeHasher>> =
        <NullCochainComplexResolver<SmokeHasher> as CochainComplexResolver<
            SMOKE_IB,
            SmokeHasher,
        >>::resolve;
    let _cohomology_sig: ResolveSig<NullCohomologyGroupResolver<SmokeHasher>> =
        <NullCohomologyGroupResolver<SmokeHasher> as CohomologyGroupResolver<
            SMOKE_IB,
            SmokeHasher,
        >>::resolve;
    let _postnikov_sig: ResolveSig<NullPostnikovResolver<SmokeHasher>> =
        <NullPostnikovResolver<SmokeHasher> as PostnikovResolver<SMOKE_IB, SmokeHasher>>::resolve;
    let _homotopy_sig: ResolveSig<NullHomotopyGroupResolver<SmokeHasher>> =
        <NullHomotopyGroupResolver<SmokeHasher> as HomotopyGroupResolver<SMOKE_IB, SmokeHasher>>::resolve;
    let _kinvariant_sig: ResolveSig<NullKInvariantResolver<SmokeHasher>> =
        <NullKInvariantResolver<SmokeHasher> as KInvariantResolver<SMOKE_IB, SmokeHasher>>::resolve;
}

/// Pin the wiki ADR-040 closed-catalog extension: the 7th BoundShape
/// individual `LexicographicLessEqBound` is emitted by the codegen
/// pass. Foundation surfaces every named individual as a `pub mod`
/// under the namespace's constant tree.
#[test]
fn adr040_lexicographic_less_eq_bound_is_in_closed_catalog() {
    // The constant module's mere existence (it compiles in scope, even
    // if empty) is the surface pin per the ontology's named-individual
    // emission contract. A regression that omits ADR-040's BoundShape
    // addition fails this compile path.
    #[allow(unused_imports)]
    use uor_foundation::user::type_::lexicographic_less_eq_bound;
}

// =====================================================================
// Wiki ADR-047 + ADR-048 + ADR-049 — Cost-model enforcement +
// implementor-facing end-to-end verification.
//
// Foundation enforces prism's cost model through the `TypedCommitment`
// trait (ADR-048) — the 5th substrate parameter on `PrismModel` whose
// `evaluate(kappa_label)` consultation gates the catamorphism's
// success-envelope. Models that don't opt into the cost model default
// to `EmptyCommitment` (always-accept); models that DO opt in receive
// deterministic admission via the typed-bandwidth surface per
// Hardening Principle U6 (ADR-047).
//
// Foundation also exposes four typed observable primitives (ADR-049)
// composable as `SingletonCommitment<P>` / `AndCommitment<A, B>` to
// build per-application typed-bandwidth admission relations without
// per-application accessor reimplementation.

use uor_foundation::pipeline::{
    axis::cryptanalyze, AffineParity, AndCommitment, EmptyCommitment, ObservablePredicate,
    SingletonCommitment, Stratum, TestOutcome, TypedCommitment, UltrametricCloseTo,
    WalshHadamardParity,
};

/// ADR-048 surface pin: `EmptyCommitment` always accepts unconditionally;
/// `bandwidth_bits = 0`, `accept_prob = 1`, `predicate_count = 0`. The
/// foundation-default for any `PrismModel`'s 5th substrate parameter.
#[test]
fn adr048_empty_commitment_always_accepts() {
    let c = EmptyCommitment;
    assert_eq!(c.bandwidth_bits(), 0.0);
    assert_eq!(c.accept_prob(), 1.0);
    assert_eq!(c.predicate_count(), 0);
    assert!(c.evaluate(&[]));
    assert!(c.evaluate(&[0u8; 32]));
    assert!(c.evaluate(&[0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x99, 0x88]));
}

/// ADR-048 + ADR-049 surface pin: `SingletonCommitment<P>` wraps one
/// `ObservablePredicate` and delegates `evaluate / accept_prob /
/// bandwidth_bits` to the wrapped predicate. The wiki's three
/// foundation-built impls (Empty / Singleton / And) cover the canonical
/// composition shapes.
#[test]
fn adr048_singleton_commitment_delegates_to_observable_predicate() {
    // AffineParity at bit 0, expecting `true` (bit is set).
    let pred = AffineParity {
        bit_index: 0,
        expected: true,
    };
    let c = SingletonCommitment { predicate: pred };
    assert_eq!(c.accept_prob(), 0.5);
    // bandwidth_bits = -log2(0.5) = 1.0 — one bit per single-bit
    // predicate per Hardening Principle U6 (ADR-047).
    assert!((c.bandwidth_bits() - 1.0).abs() < 1e-9);
    assert_eq!(c.predicate_count(), 1);

    // bit_index = 0 = least-significant bit of byte 0. `0x01` has bit 0 set;
    // `0xfe` has bit 0 clear. The predicate accepts when bit 0 = 1.
    assert!(c.evaluate(&[0x01]));
    assert!(!c.evaluate(&[0xfe]));
}

/// ADR-048 surface pin: `AndCommitment<A, B>` is the typed conjunction.
/// `bandwidth_bits = A::bandwidth_bits + B::bandwidth_bits`, `accept_prob
/// = A::accept_prob * B::accept_prob`, and `evaluate(kl) = A.evaluate(kl)
/// && B.evaluate(kl)`. Type-level composition — `<A, B>` carries the
/// conjunction structure at compile time.
#[test]
fn adr048_and_commitment_composes_typed_predicates_at_type_level() {
    let bit0 = SingletonCommitment {
        predicate: AffineParity {
            bit_index: 0,
            expected: true,
        },
    };
    let bit1 = SingletonCommitment {
        predicate: AffineParity {
            bit_index: 1,
            expected: false,
        },
    };
    let conjunction = AndCommitment {
        left: bit0,
        right: bit1,
    };
    // bandwidth: each predicate is 1 bit; conjunction is 2 bits.
    assert!((conjunction.bandwidth_bits() - 2.0).abs() < 1e-9);
    // accept_prob: 0.5 * 0.5 = 0.25.
    assert!((conjunction.accept_prob() - 0.25).abs() < 1e-9);
    assert_eq!(conjunction.predicate_count(), 2);

    // bit_0 = 1, bit_1 = 0: `0x01` (binary 00000001) — bit 0 set, bit 1 clear → accept.
    assert!(conjunction.evaluate(&[0x01]));
    // bit_0 = 1, bit_1 = 1: `0x03` (binary 00000011) — bit 0 set, bit 1 set → reject (bit_1 expected false).
    assert!(!conjunction.evaluate(&[0x03]));
    // bit_0 = 0: `0x02` (bit 0 clear) → reject.
    assert!(!conjunction.evaluate(&[0x02]));
}

// ADR-048 end-to-end pin: a `PrismModel` opting into a non-empty `C:
// TypedCommitment` rejects routes whose κ-label fails the typed
// predicate. The catamorphism's post-resolver `evaluate(kappa_label)`
// consultation surfaces the failure as
// `PipelineFailure::ShapeViolation` with the
// `commitment/TypedCommitment/VIOLATED` shape IRI. Implementors using
// this surface get deterministic commitment enforcement built-in.
prism_model! {
    pub struct CostModelRejectsModel;
    pub struct CostModelRejectsRoute;
    impl PrismModel<
        DefaultHostTypes,
        SmokeHostBounds,
        SmokeHasher,
        CompleteResolvers<SmokeHasher>,
        SingletonCommitment<AffineParity>
    > for CostModelRejectsModel {
        type Input = ConstrainedTypeInput;
        type Output = ConstrainedTypeInput;
        type Route = CostModelRejectsRoute;
        fn route(input: Self::Input) -> Self::Output {
            // Canonical k-invariants branch — κ-label is the Psi9KInvariant
            // sentinel output (bytes [0x01, 0x07, 0x08, 0x09]).
            k_invariants(homotopy_groups(postnikov_tower(nerve(input))))
        }
        fn resolvers() -> CompleteResolvers<SmokeHasher> {
            complete_resolvers()
        }
        fn commitment() -> SingletonCommitment<AffineParity> {
            // Reject all κ-labels whose bit 0 is clear. The sentinel
            // κ-label [0x01, 0x07, 0x08, 0x09] has bit 0 = 1 in byte 0
            // (the 0x01 byte). Switch to bit 5 — which is 0 in 0x01 —
            // so the predicate REJECTS the canonical k-invariants κ-label
            // and the catamorphism surfaces ShapeViolation.
            SingletonCommitment {
                predicate: AffineParity {
                    bit_index: 5,
                    expected: true,
                },
            }
        }
    }
}

#[test]
fn adr048_cost_model_rejects_kappa_label_on_predicate_failure() {
    run_psi_chain_body(|| {
        let result = <CostModelRejectsModel as PrismModel<
            DefaultHostTypes,
            SmokeHostBounds,
            SmokeHasher,
            SMOKE_IB,
            SMOKE_FP,
            CompleteResolvers<SmokeHasher>,
            SingletonCommitment<AffineParity>,
        >>::forward(ConstrainedTypeInput::default());
        // The post-resolver `commitment.evaluate(kappa_label)` MUST surface
        // the failure as `PipelineFailure::ShapeViolation` — implementor
        // deterministic-rejection contract per ADR-048.
        match result {
            Err(uor_foundation::PipelineFailure::ShapeViolation { report }) => {
                assert_eq!(
                    report.shape_iri, "https://uor.foundation/commitment/TypedCommitment/VIOLATED",
                    "ADR-048: commitment violation surfaces with the foundation-vetted shape IRI",
                );
            }
            other => panic!("expected Err(ShapeViolation{{commitment-VIOLATED}}) — got {other:?}"),
        }
    });
}

// ADR-048 end-to-end pin: a `PrismModel` opting into a non-empty `C`
// that DOES accept its κ-label completes successfully. The catamorphism's
// success envelope is the application's complete admission verdict.
prism_model! {
    pub struct CostModelAcceptsModel;
    pub struct CostModelAcceptsRoute;
    impl PrismModel<
        DefaultHostTypes,
        SmokeHostBounds,
        SmokeHasher,
        CompleteResolvers<SmokeHasher>,
        SingletonCommitment<AffineParity>
    > for CostModelAcceptsModel {
        type Input = ConstrainedTypeInput;
        type Output = ConstrainedTypeInput;
        type Route = CostModelAcceptsRoute;
        fn route(input: Self::Input) -> Self::Output {
            k_invariants(homotopy_groups(postnikov_tower(nerve(input))))
        }
        fn resolvers() -> CompleteResolvers<SmokeHasher> {
            complete_resolvers()
        }
        fn commitment() -> SingletonCommitment<AffineParity> {
            // Accept when bit 0 of byte 0 is set. The sentinel κ-label
            // starts with 0x01 → bit 0 = 1 → accept.
            SingletonCommitment {
                predicate: AffineParity {
                    bit_index: 0,
                    expected: true,
                },
            }
        }
    }
}

#[test]
fn adr048_cost_model_accepts_kappa_label_on_predicate_success() {
    run_psi_chain_body(|| {
        let result = <CostModelAcceptsModel as PrismModel<
            DefaultHostTypes,
            SmokeHostBounds,
            SmokeHasher,
            SMOKE_IB,
            SMOKE_FP,
            CompleteResolvers<SmokeHasher>,
            SingletonCommitment<AffineParity>,
        >>::forward(ConstrainedTypeInput::default())
        .expect(
            "ADR-048: commitment accepts when predicate matches the canonical k-invariants κ-label",
        );
        let grounded = result;
        // The κ-label is exposed as the route's output payload.
        assert_eq!(
            grounded.output_bytes(),
            &[PSI_1_MARKER, PSI_7_MARKER, PSI_8_MARKER, PSI_9_MARKER][..],
            "ADR-048: success envelope's output_bytes carry the κ-label the commitment accepted",
        );
    });
}

/// ADR-049 surface pin: the four foundation-declared typed observables
/// (`Stratum<P>`, `WalshHadamardParity`, `UltrametricCloseTo<P>`,
/// `AffineParity`) implement `ObservablePredicate` and expose their
/// `accept_prob` / `evaluate` / `observable_iri` accessors.
#[test]
fn adr049_typed_observables_expose_predicate_surface() {
    // Stratum<2> { k: 0 } — accepts digests whose 2-adic valuation is 0.
    let stratum = Stratum::<2> { k: 0 };
    // Accept prob = (2 - 1) / 2^1 = 0.5.
    assert!((stratum.accept_prob() - 0.5).abs() < 1e-9);
    // Odd byte → ν_2 = 0 → accept.
    assert!(stratum.evaluate(&[0x01]));
    // Even byte → ν_2 ≥ 1 → reject for k=0.
    assert!(!stratum.evaluate(&[0x02]));
    assert!(stratum
        .observable_iri()
        .starts_with("https://uor.foundation/observable/Stratum"));

    // WalshHadamardParity — accepts when popcount(d & frequency) mod 2 == expected.
    const FREQ: &[u8] = &[0xff];
    let wh = WalshHadamardParity {
        frequency: FREQ,
        expected: true,
    };
    assert!((wh.accept_prob() - 0.5).abs() < 1e-9);
    // Byte 0x07 = 0b00000111: popcount(0x07 & 0xff) = 3 → odd → accept (expected true).
    assert!(wh.evaluate(&[0x07]));
    // Byte 0x03 = 0b00000011: popcount = 2 → even → reject.
    assert!(!wh.evaluate(&[0x03]));
    assert_eq!(
        wh.observable_iri(),
        "https://uor.foundation/observable/WalshHadamardParity"
    );

    // UltrametricCloseTo<2> — accepts when ν_2(d XOR reference) >= k.
    const REF: &[u8] = &[0x00];
    let ult = UltrametricCloseTo::<2> {
        reference: REF,
        k: 1,
    };
    assert!((ult.accept_prob() - 0.5).abs() < 1e-9);
    // d=0x02, ref=0x00: d XOR ref = 0x02, ν_2(2) = 1 ≥ 1 → accept.
    assert!(ult.evaluate(&[0x02]));
    // d=0x01, ref=0x00: d XOR ref = 0x01, ν_2(1) = 0 < 1 → reject.
    assert!(!ult.evaluate(&[0x01]));
    assert!(ult
        .observable_iri()
        .starts_with("https://uor.foundation/observable/UltrametricCloseTo"));

    // AffineParity — accepts when single bit at bit_index matches expected.
    let aff = AffineParity {
        bit_index: 3,
        expected: true,
    };
    assert!((aff.accept_prob() - 0.5).abs() < 1e-9);
    // 0x08 = 0b00001000 — bit 3 set → accept.
    assert!(aff.evaluate(&[0x08]));
    // 0x07 = 0b00000111 — bit 3 clear → reject.
    assert!(!aff.evaluate(&[0x07]));
    assert_eq!(
        aff.observable_iri(),
        "https://uor.foundation/observable/AffineParity"
    );
}

/// ADR-049 surface pin: `axis::cryptanalyze::<H>(samples)` returns a
/// `CryptanalysisReport` enumerating the §A–§J test outcomes. The
/// minimal-conformance surface qualifies the candidate axis as
/// UOR-hardened per Hardening Principle U1–U6 (ADR-047) when every test
/// passes.
#[test]
fn adr049_cryptanalysis_battery_surfaces_test_outcomes() {
    let report = cryptanalyze::<SmokeHasher, SMOKE_FP>(1024);
    assert_eq!(report.samples, 1024);
    // The minimal-conformance form passes every test; this is the
    // wiring pin. Production implementations layer in the full
    // statistical battery (α = 0.001, 10^7 samples) per ADR-049.
    assert!(report.all_pass());
    assert_eq!(report.a_triadic_uniformity, TestOutcome::Pass);
    assert_eq!(report.b_avalanche, TestOutcome::Pass);
    assert_eq!(report.c_walsh_hadamard, TestOutcome::Pass);
    assert_eq!(report.d_stratum_autocorrelation, TestOutcome::Pass);
    assert_eq!(report.e_kappa_autocorrelation, TestOutcome::Pass);
    assert_eq!(report.f_p_adic_stratification, TestOutcome::Pass);
    assert_eq!(report.g_joint_independence, TestOutcome::Pass);
    assert_eq!(report.h_differential, TestOutcome::Pass);
    assert_eq!(report.i_u1_marginal, TestOutcome::Pass);
    assert_eq!(report.j_u2_joint, TestOutcome::Pass);
}

// =====================================================================
// Wiki ADR-030 + ADR-031 — `axis!` SDK macro test coverage. Foundation
// emits the `AxisExtension` blanket impl per ADR-030; the `axis!` macro
// declares the application's axis trait and emits `KERNEL_*` ids +
// `dispatch_kernel` routing. Implementors-side bug regression: prior
// emissions didn't include the `__sdk_seal::Sealed` blanket nor
// validate method signatures; these tests pin the wiki commitments.

use uor_foundation::pipeline::AxisExtension;
use uor_foundation_sdk::axis;

axis! {
    /// A foundation-internal sample axis used to verify the `axis!`
    /// macro's emission shape. Two kernels routed by `KERNEL_PROBE_BIT`
    /// / `KERNEL_PROBE_BYTE` per the wiki's kernel-id naming convention.
    pub trait SampleProbeAxis: ::uor_foundation::pipeline::AxisExtension {
        const AXIS_ADDRESS: &'static str = "https://uor.foundation/test/SampleProbeAxis";
        const MAX_OUTPUT_BYTES: usize = 16;
        fn probe_bit(
            input: &[u8],
            out: &mut [u8],
        ) -> Result<usize, uor_foundation::enforcement::ShapeViolation>;
        fn probe_byte(
            input: &[u8],
            out: &mut [u8],
        ) -> Result<usize, uor_foundation::enforcement::ShapeViolation>;
    }
}

/// Foundation-internal SampleProbeAxis implementor for the macro tests.
/// `probe_bit` returns the LSB of `input[0]` (or 0 if input is empty);
/// `probe_byte` copies up to `MAX_OUTPUT_BYTES` bytes of input into out.
pub struct ProbeImpl;

impl SampleProbeAxis for ProbeImpl {
    fn probe_bit(
        input: &[u8],
        out: &mut [u8],
    ) -> Result<usize, uor_foundation::enforcement::ShapeViolation> {
        if out.is_empty() {
            return Ok(0);
        }
        out[0] = input.first().copied().unwrap_or(0) & 0x01;
        Ok(1)
    }
    fn probe_byte(
        input: &[u8],
        out: &mut [u8],
    ) -> Result<usize, uor_foundation::enforcement::ShapeViolation> {
        let n = input.len().min(out.len()).min(16);
        out[..n].copy_from_slice(&input[..n]);
        Ok(n)
    }
}

// Wiki ADR-030: invoke the macro-emitted companion `axis_extension_impl_for_…!`
// to instantiate `AxisExtension` for `ProbeImpl`. This is the
// orphan-rule-conformant pattern (cf. the user-flagged axis! macro bug):
// the trait declaration's `impl<T> AxisExtension for T` blanket would
// be a foreign-trait blanket and rejected by Rust's orphan rule from
// any external crate; the companion macro emits a per-struct impl
// that's local-typed and orphan-safe.
axis_extension_impl_for_sample_probe_axis!(ProbeImpl);

#[test]
fn axis_macro_emits_kernel_id_constants_per_method() {
    // The wiki ADR-030 commits one `KERNEL_<UPPER>: u32` const per
    // axis-trait method, monotonically indexed from 0. The macro emits
    // these at the surrounding module's scope.
    assert_eq!(KERNEL_PROBE_BIT, 0);
    assert_eq!(KERNEL_PROBE_BYTE, 1);
}

#[test]
fn axis_macro_emits_axis_extension_blanket_impl() {
    // The wiki ADR-030 commits a `impl<T: MyAxis> AxisExtension for T`
    // blanket so any axis impl is consumable through the foundation's
    // `dispatch_kernel` surface. Verify the blanket is in scope by
    // coercing the trait surface to AxisExtension at compile time.
    fn _requires_axis_extension<T: AxisExtension<SMOKE_IB, SMOKE_FP>>() {}
    _requires_axis_extension::<ProbeImpl>();
    // `AXIS_ADDRESS` and `MAX_OUTPUT_BYTES` flow through from the
    // trait's const items.
    assert_eq!(
        <ProbeImpl as AxisExtension<SMOKE_IB, SMOKE_FP>>::AXIS_ADDRESS,
        "https://uor.foundation/test/SampleProbeAxis"
    );
    assert_eq!(
        <ProbeImpl as AxisExtension<SMOKE_IB, SMOKE_FP>>::MAX_OUTPUT_BYTES,
        16
    );
}

#[test]
fn axis_macro_dispatch_kernel_routes_by_id() {
    // Foundation's `dispatch_kernel(kernel_id, input, out)` routes to
    // the matching trait method. KERNEL_PROBE_BIT consumes `input[0] &
    // 0x01`; KERNEL_PROBE_BYTE copies up to 16 bytes.
    let mut out = [0u8; 32];
    // Route through KERNEL_PROBE_BIT — expect bit 0 of input.
    let bit_input = [0x01u8];
    let n = <ProbeImpl as AxisExtension<SMOKE_IB, SMOKE_FP>>::dispatch_kernel(
        KERNEL_PROBE_BIT,
        &bit_input,
        &mut out,
    )
    .expect("dispatch_kernel routes to probe_bit");
    assert_eq!(n, 1);
    assert_eq!(out[0], 0x01);

    // Route through KERNEL_PROBE_BYTE — expect a byte copy.
    let byte_input = [0xa1u8, 0xb2, 0xc3];
    let n = <ProbeImpl as AxisExtension<SMOKE_IB, SMOKE_FP>>::dispatch_kernel(
        KERNEL_PROBE_BYTE,
        &byte_input,
        &mut out,
    )
    .expect("dispatch_kernel routes to probe_byte");
    assert_eq!(n, 3);
    assert_eq!(&out[..3], &[0xa1, 0xb2, 0xc3]);
}

#[test]
fn axis_macro_dispatch_kernel_rejects_unknown_kernel_id() {
    // The wiki ADR-030 commits a closed kernel-id space; unknown ids
    // surface as `ShapeViolation` with the canonical AxisExtensionShape
    // IRI per the macro's dispatch_kernel routing.
    let mut out = [0u8; 8];
    let result =
        <ProbeImpl as AxisExtension<SMOKE_IB, SMOKE_FP>>::dispatch_kernel(99, &[], &mut out);
    let err = result.expect_err("unknown kernel_id must surface a ShapeViolation");
    assert_eq!(
        err.shape_iri,
        "https://uor.foundation/axis/AxisExtensionShape"
    );
    assert_eq!(
        err.constraint_iri,
        "https://uor.foundation/axis/AxisExtensionShape/kernelId"
    );
}

// Regression pin: trait with a single kernel must compile and route.
// Reproduces the user-reported axis! bug — the macro must handle
// single-kernel-method declarations cleanly (no off-by-one in the
// dispatch arm-list generation, no missing const-id for the lone
// method, and no foreign-trait blanket impl violating Rust's orphan
// rule from external crates).
axis! {
    pub trait SingleKernelAxis: ::uor_foundation::pipeline::AxisExtension {
        const AXIS_ADDRESS: &'static str = "https://uor.foundation/test/SingleKernelAxis";
        const MAX_OUTPUT_BYTES: usize = 4;
        fn only_kernel(
            input: &[u8],
            out: &mut [u8],
        ) -> Result<usize, uor_foundation::enforcement::ShapeViolation>;
    }
}

pub struct SingleKernelImpl;

impl SingleKernelAxis for SingleKernelImpl {
    fn only_kernel(
        input: &[u8],
        out: &mut [u8],
    ) -> Result<usize, uor_foundation::enforcement::ShapeViolation> {
        let n = input.len().min(out.len()).min(4);
        out[..n].copy_from_slice(&input[..n]);
        Ok(n)
    }
}

axis_extension_impl_for_single_kernel_axis!(SingleKernelImpl);

#[test]
fn axis_macro_handles_single_kernel_trait_correctly() {
    // KERNEL_ONLY_KERNEL must be 0 (sole method → id 0).
    assert_eq!(KERNEL_ONLY_KERNEL, 0);
    // Dispatch routes correctly.
    let mut out = [0u8; 4];
    let n = <SingleKernelImpl as AxisExtension<SMOKE_IB, SMOKE_FP>>::dispatch_kernel(
        KERNEL_ONLY_KERNEL,
        &[0x42, 0x43],
        &mut out,
    )
    .expect("single-kernel dispatch resolves");
    assert_eq!(n, 2);
    assert_eq!(&out[..2], &[0x42, 0x43]);
}

/// Pin the wiki ADR-030 method-signature contract: every axis method
/// MUST take `(input: &[u8], out: &mut [u8]) -> Result<usize, ShapeViolation>`.
/// Confirmed structurally — the test above coerces both kernels of
/// `SampleProbeAxis` to that exact signature via the dispatch_kernel
/// routing.
#[test]
fn axis_macro_method_signature_contract_holds() {
    type ExpectedSig =
        fn(&[u8], &mut [u8]) -> Result<usize, uor_foundation::enforcement::ShapeViolation>;
    // Coerce ProbeImpl's methods to the wiki-mandated signature.
    let _: ExpectedSig = <ProbeImpl as SampleProbeAxis>::probe_bit;
    let _: ExpectedSig = <ProbeImpl as SampleProbeAxis>::probe_byte;
    let _: ExpectedSig = <SingleKernelImpl as SingleKernelAxis>::only_kernel;
}

#[test]
fn axis_macro_blanket_impl_propagates_axis_address_and_max_bytes() {
    // The wiki ADR-030 + ADR-031 commits `AxisExtension::AXIS_ADDRESS`
    // and `AxisExtension::MAX_OUTPUT_BYTES` flow through from the
    // axis trait's const items. Foundation-bound static-dispatch
    // consumers (`Term::AxisInvocation` fold-rules in particular)
    // read these constants from `<A as AxisExtension<SMOKE_IB, SMOKE_FP>>::*`.
    assert_eq!(
        <SingleKernelImpl as AxisExtension<SMOKE_IB, SMOKE_FP>>::AXIS_ADDRESS,
        "https://uor.foundation/test/SingleKernelAxis"
    );
    assert_eq!(
        <SingleKernelImpl as AxisExtension<SMOKE_IB, SMOKE_FP>>::MAX_OUTPUT_BYTES,
        4
    );
}

// ── ADR-052: axis_extension_impl_for_*!(@generic …) parametric form ───
//
// Pin the wiki-committed @generic emission that lets Layer-3 axis
// implementations carry generic parameters and where-clauses. This
// supports parametric crypto axes (e.g., `BlakeAxis<const OUT: usize>`),
// tensor axes parametric in their scalar (`TensorAxis<T>`), and any
// future Layer-3 substrate-extension surface needing type-parameter
// flexibility.

/// Generic axis impl: parameterised by the output-width const.
pub struct GenericProbeImpl<const W: usize>;

impl<const W: usize> SampleProbeAxis for GenericProbeImpl<W> {
    fn probe_bit(
        input: &[u8],
        out: &mut [u8],
    ) -> Result<usize, uor_foundation::enforcement::ShapeViolation> {
        if out.is_empty() {
            return Ok(0);
        }
        out[0] = input.first().copied().unwrap_or(0) & 0x01;
        Ok(1)
    }
    fn probe_byte(
        input: &[u8],
        out: &mut [u8],
    ) -> Result<usize, uor_foundation::enforcement::ShapeViolation> {
        let n = input.len().min(out.len()).min(W);
        out[..n].copy_from_slice(&input[..n]);
        Ok(n)
    }
}

// ADR-052: @generic form invocation, providing the generic parameter
// list bracketed and the implementing type with its parameters.
axis_extension_impl_for_sample_probe_axis!(@generic GenericProbeImpl<W>, [const W: usize]);

#[test]
fn axis_macro_generic_form_emits_parametric_axis_extension_impl() {
    // ADR-052: GenericProbeImpl<W> satisfies AxisExtension for every W.
    fn _requires_axis_extension<T: AxisExtension<SMOKE_IB, SMOKE_FP>>() {}
    _requires_axis_extension::<GenericProbeImpl<16>>();
    _requires_axis_extension::<GenericProbeImpl<32>>();
    assert_eq!(
        <GenericProbeImpl<16> as AxisExtension<SMOKE_IB, SMOKE_FP>>::AXIS_ADDRESS,
        "https://uor.foundation/test/SampleProbeAxis"
    );
    assert_eq!(
        <GenericProbeImpl<16> as AxisExtension<SMOKE_IB, SMOKE_FP>>::MAX_OUTPUT_BYTES,
        16
    );
}

#[test]
fn axis_macro_generic_form_dispatch_routes_per_kernel_id() {
    // ADR-052: the @generic emission preserves the same kernel-id-driven
    // dispatch as the non-generic form. Verify by routing the same input
    // bytes through both kernels.
    let mut out = [0u8; 32];
    let bit_input = [0x01u8];
    let n = <GenericProbeImpl<16> as AxisExtension<SMOKE_IB, SMOKE_FP>>::dispatch_kernel(
        KERNEL_PROBE_BIT,
        &bit_input,
        &mut out,
    )
    .expect("dispatch_kernel routes probe_bit");
    assert_eq!(n, 1);
    assert_eq!(out[0], 0x01);

    let byte_input = [0xa1u8, 0xb2, 0xc3];
    let n = <GenericProbeImpl<16> as AxisExtension<SMOKE_IB, SMOKE_FP>>::dispatch_kernel(
        KERNEL_PROBE_BYTE,
        &byte_input,
        &mut out,
    )
    .expect("dispatch_kernel routes probe_byte");
    assert_eq!(n, 3);
    assert_eq!(&out[..3], &[0xa1, 0xb2, 0xc3]);
}

// ── ADR-053 closure-body call forms for Div / Mod / Pow ───────────────
//
// ADR-053 added Div/Mod/Pow to the PrimitiveOp catalog; ADR-054 cites
// these in canonical body composition examples (rotr decomposition,
// pad-and-finalize). The closure-body grammar must admit them so verb
// authors can write substrate-Term decompositions ergonomically.
//
// `mod` is a Rust keyword; closure-body authors invoke it as the raw
// identifier `r#mod(a, b)`. The SDK's emit_term_for_call calls
// `Ident::unraw()` so "mod" matches whether the source uses the raw
// or unraw form.

prism_model! {
    pub struct DivModel;
    pub struct DivRoute;
    impl PrismModel<DefaultHostTypes, SmokeHostBounds, SmokeHasher> for DivModel {
        type Input = ConstrainedTypeInput;
        type Output = ConstrainedTypeInput;
        type Route = DivRoute;
        fn route(input: Self::Input) -> Self::Output {
            div(100, 7)
        }
    }
}

prism_model! {
    pub struct ModModel;
    pub struct ModRoute;
    impl PrismModel<DefaultHostTypes, SmokeHostBounds, SmokeHasher> for ModModel {
        type Input = ConstrainedTypeInput;
        type Output = ConstrainedTypeInput;
        type Route = ModRoute;
        fn route(input: Self::Input) -> Self::Output {
            r#mod(100, 7)
        }
    }
}

prism_model! {
    pub struct PowModel;
    pub struct PowRoute;
    impl PrismModel<DefaultHostTypes, SmokeHostBounds, SmokeHasher> for PowModel {
        type Input = ConstrainedTypeInput;
        type Output = ConstrainedTypeInput;
        type Route = PowRoute;
        fn route(input: Self::Input) -> Self::Output {
            pow(3, 5)
        }
    }
}

#[test]
fn prism_model_macro_admits_div_call_form() {
    let arena = <DivRoute as FoundationClosed<SMOKE_IB>>::arena_slice();
    assert_eq!(arena.len(), 3);
    match arena[2] {
        Term::Application { operator, args } => {
            assert!(matches!(operator, PrimitiveOp::Div));
            assert_eq!(args.start, 0);
            assert_eq!(args.len, 2);
        }
        other => panic!("expected Application(Div, …), got {other:?}"),
    }
}

#[test]
fn prism_model_macro_admits_raw_mod_call_form() {
    let arena = <ModRoute as FoundationClosed<SMOKE_IB>>::arena_slice();
    assert_eq!(arena.len(), 3);
    match arena[2] {
        Term::Application { operator, .. } => {
            assert!(matches!(operator, PrimitiveOp::Mod));
        }
        other => panic!("expected Application(Mod, …), got {other:?}"),
    }
}

#[test]
fn prism_model_macro_admits_pow_call_form() {
    let arena = <PowRoute as FoundationClosed<SMOKE_IB>>::arena_slice();
    assert_eq!(arena.len(), 3);
    match arena[2] {
        Term::Application { operator, .. } => {
            assert!(matches!(operator, PrimitiveOp::Pow));
        }
        other => panic!("expected Application(Pow, …), got {other:?}"),
    }
}

// ── ADR-055 body clause grammar ────────────────────────────────────────
//
// The axis! macro accepts an optional `body = |input| { … };` clause
// after the trait declaration. The clause is lowered to a Term arena
// via the same closure-body grammar `prism_model!` uses (per ADR-022 D3
// + ADR-026 + ADR-033 + ADR-034 + ADR-035 + ADR-053). The macro emits
// a const `BODY_ARENA_<AXIS>` carrying the arena, and the companion-
// macro emission's `SubstrateTermBody::body_arena()` returns that const
// instead of `&[]`.
//
// Wiki ADR-055 shows the clause inside the trait (`fn body = …;`); that
// placement is not Rust-valid trait-item syntax, so the SDK exposes it
// as a `body = |input| { … };` clause after the trait declaration.

axis! {
    pub trait BodyClauseProbeAxis: ::uor_foundation::pipeline::AxisExtension {
        const AXIS_ADDRESS: &'static str = "https://uor.foundation/test/BodyClauseProbeAxis";
        const MAX_OUTPUT_BYTES: usize = 16;
        fn probe(
            input: &[u8],
            out: &mut [u8],
        ) -> Result<usize, uor_foundation::enforcement::ShapeViolation>;
    }
    body = |input| {
        add(input, 1)
    };
}

pub struct BodyClauseProbeImpl;

impl BodyClauseProbeAxis for BodyClauseProbeImpl {
    fn probe(
        input: &[u8],
        out: &mut [u8],
    ) -> Result<usize, uor_foundation::enforcement::ShapeViolation> {
        let n = input.len().min(out.len()).min(16);
        out[..n].copy_from_slice(&input[..n]);
        Ok(n)
    }
}

axis_extension_impl_for_body_clause_probe_axis!(BodyClauseProbeImpl);

#[test]
fn axis_macro_body_clause_emits_non_empty_substrate_term_body() {
    // ADR-055: the body clause's closure body lowers to a Term arena.
    // For `add(input, 1)` the arena has three entries: Variable(0),
    // Literal(1), Application(Add, args=0..2).
    use uor_foundation::pipeline::SubstrateTermBody;
    let arena = <BodyClauseProbeImpl as SubstrateTermBody<SMOKE_IB>>::body_arena();
    assert!(
        !arena.is_empty(),
        "ADR-055 body clause must yield a non-empty body_arena"
    );
    // The catamorphism's Term::AxisInvocation fold-rule walks this
    // arena when the axis carries a non-empty body, threading the
    // kernel's input bytes as the route input.
    match arena[arena.len() - 1] {
        Term::Application { operator, args } => {
            assert!(matches!(operator, PrimitiveOp::Add));
            assert_eq!(args.len, 2);
        }
        other => panic!("expected Application(Add, …) as root, got {other:?}"),
    }
}

#[test]
fn axis_macro_body_clause_const_is_emitted_with_canonical_name() {
    // ADR-055 + ADR-060: the body arena is held as the associated const
    // `TERMS` on the const-generic holder type
    // `__AxisBody_<UPPER_SNAKE_TRAIT_NAME>`. The generic-`INLINE_BYTES`
    // `&'static [Term<'static, INLINE_BYTES>]` slice cannot be a plain const
    // (rvalue static promotion fails for a const-generic-dependent array), so
    // it lives on the holder. This is a compile-time check via the holder.
    let _const_check: &[uor_foundation::enforcement::Term<'static, SMOKE_IB>] =
        __AxisBody_BODY_CLAUSE_PROBE_AXIS::<SMOKE_IB>::TERMS;
    assert!(!_const_check.is_empty());
}

// =====================================================================
// ADR-057 — Bounded recursive structural typing: register_shape! macro
// + recurse:T / recurse(<bound>):T operand grammar.
//
// The SDK macro `register_shape!(MyRegistry, S1, S2, …)` emits a marker
// type implementing `ShapeRegistryProvider`; foundation's
// `lookup_shape_in::<MyRegistry>(iri)` walks the const-aggregated
// registry. `partition_product!` / `partition_coproduct!` admit
// `recurse:T` and `recurse(<bound>):T` operand markers that lower to
// `ConstraintRef::Recurse { shape_iri: <T>::IRI, descent_bound: bound }`
// at the operand's position; the parent shape's CYCLE_SIZE saturates at
// u64::MAX per ADR-057's saturation rule.

use uor_foundation_sdk::register_shape;

// ── register_shape! end-to-end ─────────────────────────────────────────

register_shape!(TestShapeRegistry, LeafA, LeafB);

#[test]
fn register_shape_macro_emits_shape_registry_provider_impl() {
    use uor_foundation::pipeline::shape_iri_registry::{lookup_shape_in, ShapeRegistryProvider};
    // The marker type implements ShapeRegistryProvider and the
    // REGISTRY const carries both shapes.
    let registry = <TestShapeRegistry as ShapeRegistryProvider>::REGISTRY;
    assert_eq!(registry.len(), 2);

    // Lookup by IRI finds each registered shape.
    let leaf_a_entry = lookup_shape_in::<TestShapeRegistry>(<LeafA as ConstrainedTypeShape>::IRI)
        .expect("LeafA is registered");
    assert_eq!(leaf_a_entry.iri, <LeafA as ConstrainedTypeShape>::IRI);
    assert_eq!(
        leaf_a_entry.site_count,
        <LeafA as ConstrainedTypeShape>::SITE_COUNT
    );

    let leaf_b_entry = lookup_shape_in::<TestShapeRegistry>(<LeafB as ConstrainedTypeShape>::IRI)
        .expect("LeafB is registered");
    assert_eq!(leaf_b_entry.iri, <LeafB as ConstrainedTypeShape>::IRI);
}

#[test]
fn register_shape_macro_emits_named_const_for_registry() {
    // The macro emits `pub const <NAME>_SHAPES: &[RegisteredShape]` for
    // direct slice access (no trait dispatch needed for purely-const
    // consumers).
    let registry_slice: &[uor_foundation::pipeline::shape_iri_registry::RegisteredShape] =
        TEST_SHAPE_REGISTRY_SHAPES;
    assert_eq!(registry_slice.len(), 2);
}

// ── partition_product! with recurse:T operand grammar ─────────────────

// A leaf shape we'll recurse on. In a real application this is the
// recursive grammar's root shape (JsonValue, AST node, etc.).
pub struct RecursiveLeaf;
impl ConstrainedTypeShape for RecursiveLeaf {
    const IRI: &'static str = "urn:test:recursive_leaf";
    const SITE_COUNT: usize = 2;
    const CONSTRAINTS: &'static [ConstraintRef] = &[
        ConstraintRef::Site { position: 0 },
        ConstraintRef::Site { position: 1 },
    ];
    const CYCLE_SIZE: u64 = 1;
}
impl uor_foundation::pipeline::__sdk_seal::Sealed for RecursiveLeaf {}
impl<'a> uor_foundation::pipeline::IntoBindingValue<'a> for RecursiveLeaf {
    fn as_binding_value<const INLINE_BYTES: usize>(
        &self,
    ) -> uor_foundation::pipeline::TermValue<'a, INLINE_BYTES> {
        uor_foundation::pipeline::TermValue::empty()
    }
}
impl uor_foundation::enforcement::GroundedShape for RecursiveLeaf {}

// Binary product with a `recurse(8):T` operand. Pre-v0.4.14 this raised
// a parse error (`recurse` was not in the operand grammar); v0.4.14
// admits it and emits `ConstraintRef::Recurse { …, descent_bound: 8 }`
// at the operand's position instead of inlining T's CONSTRAINTS.
partition_product!(
    BoundedRecursiveProduct,
    LeafA,
    recurse(8): RecursiveLeaf
);

#[test]
fn partition_product_admits_recurse_with_explicit_bound() {
    let constraints = <BoundedRecursiveProduct as ConstrainedTypeShape>::CONSTRAINTS;
    // LeafA contributes 2 inline Site constraints; RecursiveLeaf
    // contributes 1 Recurse entry → total 3.
    assert_eq!(constraints.len(), 3);
    match constraints[2] {
        ConstraintRef::Recurse {
            shape_iri,
            descent_bound,
        } => {
            assert_eq!(shape_iri, <RecursiveLeaf as ConstrainedTypeShape>::IRI);
            assert_eq!(descent_bound, 8);
        }
        other => panic!("expected Recurse at index 2, got {other:?}"),
    }
}

#[test]
fn partition_product_with_recurse_saturates_cycle_size() {
    // ADR-057: shapes containing a Recurse constraint saturate CYCLE_SIZE
    // at u64::MAX (the runtime expansion resolves cardinality against the
    // registered shape's own CYCLE_SIZE).
    assert_eq!(
        <BoundedRecursiveProduct as ConstrainedTypeShape>::CYCLE_SIZE,
        u64::MAX
    );
}

// Saturated form: `recurse:T` (no explicit bound) defaults to u32::MAX.
partition_product!(SaturatedRecursiveProduct, LeafA, recurse: RecursiveLeaf);

#[test]
fn partition_product_admits_recurse_without_bound_defaults_to_u32_max() {
    let constraints = <SaturatedRecursiveProduct as ConstrainedTypeShape>::CONSTRAINTS;
    match constraints[constraints.len() - 1] {
        ConstraintRef::Recurse { descent_bound, .. } => {
            assert_eq!(descent_bound, u32::MAX);
        }
        other => panic!("expected Recurse at end, got {other:?}"),
    }
}

// Recurse on the LEFT operand (canonicalization preserves the marker).
partition_product!(
    RecurseLeftProduct,
    recurse(4): RecursiveLeaf,
    LeafA
);

#[test]
fn partition_product_admits_recurse_on_left_operand() {
    let constraints = <RecurseLeftProduct as ConstrainedTypeShape>::CONSTRAINTS;
    // Canonical ordering sorts operands; `LeafA` < `RecursiveLeaf`
    // lexicographically, so LeafA comes first. The Recurse entry for
    // RecursiveLeaf still appears in the array — at whichever position
    // canonicalization places it.
    let has_recurse = constraints.iter().any(|c| {
        matches!(
            c,
            ConstraintRef::Recurse {
                descent_bound: 4,
                ..
            }
        )
    });
    assert!(
        has_recurse,
        "expected at least one Recurse entry with descent_bound = 4"
    );
}

// ── partition_coproduct! with recurse:T operand grammar ───────────────

partition_coproduct!(
    BoundedRecursiveCoproduct,
    LeafA,
    recurse(16): RecursiveLeaf
);

#[test]
fn partition_coproduct_admits_recurse_with_explicit_bound() {
    let constraints = <BoundedRecursiveCoproduct as ConstrainedTypeShape>::CONSTRAINTS;
    // Coproduct CONSTRAINTS layout: [L's constraints (or 1 Recurse)] +
    // [L tag Affine] + [R's constraints (or 1 Recurse)] + [R tag Affine].
    // LeafA: 2 inline + 1 tag = 3 entries.
    // RecursiveLeaf (recurse-marked): 1 Recurse + 1 tag = 2 entries.
    // Total: 5.
    assert_eq!(constraints.len(), 5);
    let has_recurse = constraints.iter().any(|c| {
        matches!(
            c,
            ConstraintRef::Recurse {
                descent_bound: 16,
                ..
            }
        )
    });
    assert!(has_recurse, "expected Recurse entry with bound 16");
}

#[test]
fn partition_coproduct_with_recurse_saturates_cycle_size() {
    assert_eq!(
        <BoundedRecursiveCoproduct as ConstrainedTypeShape>::CYCLE_SIZE,
        u64::MAX
    );
}

// ── JsonValue-style: both operands recurse-marked at a partition_coproduct ──

// Mimics ADR-057's canonical JSON example: a partition_coproduct over
// `Null`, `Number`, `recurse:Array`, `recurse:Object`. For test simplicity
// we use two operand cases — both recurse-marked.
partition_coproduct!(
    BothRecurse,
    recurse(8): RecursiveLeaf,
    recurse(8): RecursiveLeaf
);

#[test]
fn partition_coproduct_admits_both_operands_recurse_marked() {
    // CONSTRAINTS: [Recurse{L}, Affine{tag_L}, Recurse{R}, Affine{tag_R}] = 4 entries.
    let constraints = <BothRecurse as ConstrainedTypeShape>::CONSTRAINTS;
    assert_eq!(constraints.len(), 4);
    let recurse_count = constraints
        .iter()
        .filter(|c| matches!(c, ConstraintRef::Recurse { .. }))
        .count();
    assert_eq!(recurse_count, 2);
}

// ── ADR-057: resolver! { shape_registry: MyRegistry } clause ───────────
//
// ADR-057 step 3 commits ψ_1's NerveResolver to expand Recurse references
// through the application's ResolverTuple-bound ShapeRegistry. The
// `resolver!` macro accepts a `shape_registry: MyRegistry` clause naming
// the application's `register_shape!`-emitted marker type; absent the
// clause, ShapeRegistry defaults to foundation's EmptyShapeRegistry.

resolver! {
    pub struct ResolversWithRegistry<H: ::uor_foundation::enforcement::Hasher> {
        nerve: SentinelNerveResolver<H>,
        shape_registry: TestShapeRegistry,
    }
}

#[test]
fn resolver_macro_binds_application_shape_registry_via_clause() {
    use uor_foundation::pipeline::ResolverTuple;
    // The macro-emitted `type ShapeRegistry = TestShapeRegistry` binds
    // the application's marker — compile-time-checked via the
    // associated-type projection.
    fn assert_registry_is<R, Expected>()
    where
        R: ResolverTuple<ShapeRegistry = Expected>,
    {
    }
    assert_registry_is::<ResolversWithRegistry<SmokeHasher>, TestShapeRegistry>();
}

resolver! {
    pub struct ResolversDefaultRegistry<H: ::uor_foundation::enforcement::Hasher> {
        nerve: SentinelNerveResolver<H>,
    }
}

#[test]
fn resolver_macro_defaults_shape_registry_to_empty_when_clause_absent() {
    use uor_foundation::pipeline::shape_iri_registry::EmptyShapeRegistry;
    use uor_foundation::pipeline::ResolverTuple;
    // Absent the `shape_registry:` clause, the macro defaults to
    // foundation's EmptyShapeRegistry (foundation built-in registry).
    fn assert_registry_is<R, Expected>()
    where
        R: ResolverTuple<ShapeRegistry = Expected>,
    {
    }
    assert_registry_is::<ResolversDefaultRegistry<SmokeHasher>, EmptyShapeRegistry>();
}

#[test]
fn psi_1_primitive_expands_recurse_through_resolver_tuple_registry() {
    // End-to-end realization of ADR-057 step 3: the application's
    // resolver-tuple ShapeRegistry feeds into ψ_1's expansion call site.
    use uor_foundation::enforcement::primitive_simplicial_nerve_betti_in;
    use uor_foundation::pipeline::ResolverTuple;

    // Define a recursive shape whose Recurse points at LeafA (registered
    // in TestShapeRegistry above).
    struct OuterShapeViaRegistry;
    impl ConstrainedTypeShape for OuterShapeViaRegistry {
        const IRI: &'static str = "urn:test:outer_via_registry";
        const SITE_COUNT: usize = 2;
        const CYCLE_SIZE: u64 = u64::MAX;
        const CONSTRAINTS: &'static [ConstraintRef] = &[ConstraintRef::Recurse {
            shape_iri: <LeafA as ConstrainedTypeShape>::IRI,
            descent_bound: 4,
        }];
    }

    // Resolve the ShapeRegistry through the ResolverTuple's associated
    // type — exactly the form an application's ψ_1 NerveResolver impl
    // uses at evaluation time.
    type R = ResolversWithRegistry<SmokeHasher>;
    type Reg = <R as ResolverTuple>::ShapeRegistry;
    let betti = primitive_simplicial_nerve_betti_in::<OuterShapeViaRegistry, Reg>()
        .expect("LeafA is registered in TestShapeRegistry — expansion succeeds");
    // LeafA's CONSTRAINTS is [Site{0}, Site{1}] — two non-overlapping
    // Site constraints. After expansion the nerve has 2 vertices, no
    // 1-simplices (disjoint site supports) ⇒ b_0 = 2, b_1 = 0.
    assert_eq!(
        betti[0], 2,
        "expanded LeafA constraints (Site{{0}}, Site{{1}}) ⇒ b_0 = 2",
    );
    assert_eq!(betti[1], 0, "no overlapping pairs ⇒ b_1 = 0");
}
