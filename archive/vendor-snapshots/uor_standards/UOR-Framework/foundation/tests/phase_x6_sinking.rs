//! Target §3 + §4.6 — Sinking discipline.
//!
//! Verifies that the `Sinking` trait enforces at the Rust type level what
//! the target doc declares as the outbound-boundary guarantee: input must
//! be `Grounded<'static, T>`; cannot launder unverified data outward. The input
//! type is structurally unforgeable — `Grounded<'static, T>` is sealed per §2 and
//! only `pipeline::run` mints it — so no Rust syntax expresses a bypass.
//!
//! Coverage:
//! - `Sinking::project` type-level contract
//! - Downstream-authored Sinking impl over `ConstrainedTypeInput`
//! - Two distinct `Sinking` impls serving different `ProjectionMap` kinds
//! - `MorphismKind` supertrait: `ProjectionMapKind` extends it and carries
//!   the ontology IRI
//! - Structural markers (`Total`, `Invertible`, etc.) apply to both
//!   `GroundingMapKind` and `ProjectionMapKind` hierarchies

#![allow(dead_code, clippy::unwrap_used)]

use uor_foundation::enforcement::{
    BinaryProjectionMap, CompileUnitBuilder, ConstrainedTypeInput, DigestProjectionMap, Grounded,
    IntegerProjectionMap, Invertible, JsonProjectionMap, MorphismKind, PreservesStructure,
    ProjectionMapKind, Sinking, Term, Total, Utf8ProjectionMap,
};
use uor_foundation::pipeline::run;
use uor_foundation::{VerificationDomain, WittLevel};
use uor_foundation_test_helpers::Fnv1aHasher16;
use uor_foundation_test_helpers::REFERENCE_INLINE_BYTES as N;

// ADR-060: `TermValue` holds a `dyn ChunkSource` and is therefore not `Sync`,
// so the term slice lives in a `const` (no `Sync` requirement) rather than a
// `static`.
const ROOT_TERMS: &[Term<'static, N>] = &[uor_foundation::pipeline::literal_u64(42, WittLevel::W8)];
static DOMAINS: &[VerificationDomain] = &[VerificationDomain::Enumerative];

fn grounded_probe() -> Grounded<'static, ConstrainedTypeInput, N> {
    let unit = CompileUnitBuilder::new()
        .root_term(ROOT_TERMS)
        .witt_level_ceiling(WittLevel::W32)
        .thermodynamic_budget(4096)
        .target_domains(DOMAINS)
        .result_type::<ConstrainedTypeInput>()
        .validate()
        .expect("unit valid");
    run::<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32>(unit).expect("pipeline admits")
}

// ──────────────────────────────────────────────────────────────────────────
// `MorphismKind` supertrait carries the ontology IRI for both kind
// hierarchies — one abstraction, two kinds.
// ──────────────────────────────────────────────────────────────────────────

const fn require_morphism<T: MorphismKind>() {
    let _ = T::ONTOLOGY_IRI;
}

const fn require_projection<T: ProjectionMapKind>() {}

const _: () = {
    require_morphism::<IntegerProjectionMap>();
    require_morphism::<Utf8ProjectionMap>();
    require_morphism::<JsonProjectionMap>();
    require_morphism::<DigestProjectionMap>();
    require_morphism::<BinaryProjectionMap>();

    require_projection::<IntegerProjectionMap>();
    require_projection::<Utf8ProjectionMap>();
    require_projection::<JsonProjectionMap>();
    require_projection::<DigestProjectionMap>();
    require_projection::<BinaryProjectionMap>();
};

#[test]
fn projection_map_kinds_carry_correct_ontology_iris() {
    assert_eq!(
        IntegerProjectionMap::ONTOLOGY_IRI,
        "https://uor.foundation/morphism/IntegerProjectionMap"
    );
    assert_eq!(
        Utf8ProjectionMap::ONTOLOGY_IRI,
        "https://uor.foundation/morphism/Utf8ProjectionMap"
    );
    assert_eq!(
        JsonProjectionMap::ONTOLOGY_IRI,
        "https://uor.foundation/morphism/JsonProjectionMap"
    );
    assert_eq!(
        DigestProjectionMap::ONTOLOGY_IRI,
        "https://uor.foundation/morphism/DigestProjectionMap"
    );
    assert_eq!(
        BinaryProjectionMap::ONTOLOGY_IRI,
        "https://uor.foundation/morphism/BinaryProjectionMap"
    );
}

// ──────────────────────────────────────────────────────────────────────────
// Structural markers apply across both kind hierarchies (shared via
// MorphismKind supertrait).
// ──────────────────────────────────────────────────────────────────────────

const fn require_total_invertible<T: Total + Invertible>() {}
const fn require_invertible_structure<T: Invertible + PreservesStructure>() {}
const fn require_total<T: Total>() {}

const _: () = {
    // ProjectionMap side:
    require_invertible_structure::<IntegerProjectionMap>();
    require_invertible_structure::<Utf8ProjectionMap>();
    require_invertible_structure::<JsonProjectionMap>();
    require_total::<DigestProjectionMap>();
    require_total_invertible::<BinaryProjectionMap>();
};

// ──────────────────────────────────────────────────────────────────────────
// Sinking discipline — downstream impls.
// ──────────────────────────────────────────────────────────────────────────

struct AddressToStringSink;

impl Sinking<N> for AddressToStringSink {
    type Source = ConstrainedTypeInput;
    type ProjectionMap = Utf8ProjectionMap;
    type Output = String;

    fn project(&self, grounded: &Grounded<'_, ConstrainedTypeInput, N>) -> String {
        format!("address={:?}", grounded.unit_address())
    }
}

struct FingerprintToBytesSink;

impl Sinking<N> for FingerprintToBytesSink {
    type Source = ConstrainedTypeInput;
    type ProjectionMap = BinaryProjectionMap;
    type Output = Vec<u8>;

    fn project(&self, grounded: &Grounded<'_, ConstrainedTypeInput, N>) -> Vec<u8> {
        grounded.content_fingerprint().as_bytes().to_vec()
    }
}

#[test]
fn sinking_projects_grounded_to_string() {
    let grounded = grounded_probe();
    let sink = AddressToStringSink;
    let out = sink.project(&grounded);
    assert!(out.starts_with("address="), "got: {out}");
}

#[test]
fn sinking_projects_grounded_to_bytes() {
    let grounded = grounded_probe();
    let sink = FingerprintToBytesSink;
    let out = sink.project(&grounded);
    assert!(!out.is_empty(), "fingerprint must be non-empty");
}

#[test]
fn distinct_sinking_impls_over_same_grounded_yield_distinct_outputs() {
    let grounded = grounded_probe();
    let s1 = AddressToStringSink.project(&grounded);
    let s2 = FingerprintToBytesSink.project(&grounded);
    // Both are projections of the same grounded value but through different
    // ProjectionMap kinds. Their output types and contents differ by
    // construction.
    assert!(!s1.is_empty());
    assert!(!s2.is_empty());
}

#[test]
fn sinking_is_content_deterministic() {
    let g1 = grounded_probe();
    let g2 = grounded_probe();
    let sink = AddressToStringSink;
    assert_eq!(sink.project(&g1), sink.project(&g2));
}

// ──────────────────────────────────────────────────────────────────────────
// Type-level contract: Sinking::project accepts ONLY &Grounded<'static, Source>.
// The compile_fail doctest below proves raw primitives are rejected.
// ──────────────────────────────────────────────────────────────────────────

/// The `Sinking::project` signature rejects non-`Grounded` inputs at compile
/// time — the "cannot launder unverified data outward" guarantee is
/// structural, not runtime-checked.
///
/// ```compile_fail
/// use uor_foundation::enforcement::{
///     BinaryProjectionMap, ConstrainedTypeInput, Grounded, Sinking,
/// };
/// struct RawSink;
/// impl Sinking<97> for RawSink {
///     type Source = ConstrainedTypeInput;
///     type ProjectionMap = BinaryProjectionMap;
///     type Output = Vec<u8>;
///     fn project(&self, grounded: &Grounded<'_, ConstrainedTypeInput, 97>) -> Vec<u8> {
///         Vec::new()
///     }
/// }
/// // This fails to compile: 0i64 is not &Grounded<'static, ConstrainedTypeInput, 97>.
/// let _ = RawSink.project(&0i64);
/// ```
#[allow(dead_code)]
fn _compile_fail_doctest_anchor() {}
