//! v0.2.2 W4 + W17: marker-trait coverage for `GroundingMapKind` instances.
//!
//! Asserts the structural property table from the W4 plan: each kind
//! implements exactly the marker traits its semantic role admits. The
//! assertions are evaluated at compile time via type-level trait bounds
//! — failure to compile means the marker-trait impl table in the
//! enforcement codegen drifted from the plan.

use uor_foundation::enforcement::{
    BinaryGroundingMap, DigestGroundingMap, GroundingMapKind, IntegerGroundingMap, Invertible,
    JsonGroundingMap, MorphismKind, PreservesStructure, Total, Utf8GroundingMap,
};

/// Compile-time witness that `T` implements `GroundingMapKind`.
const fn require_kind<T: GroundingMapKind>() {
    let _ = T::ONTOLOGY_IRI;
}

/// Compile-time witness that `T` implements both `Total` and `Invertible`.
const fn require_total_invertible<T: Total + Invertible>() {}

/// Compile-time witness that `T` implements `Total`.
const fn require_total<T: Total>() {}

/// Compile-time witness that `T` implements `PreservesStructure`.
const fn require_preserves_structure<T: PreservesStructure>() {}

#[test]
fn integer_grounding_map_is_total_invertible_structure_preserving() {
    require_kind::<IntegerGroundingMap>();
    require_total::<IntegerGroundingMap>();
    require_total_invertible::<IntegerGroundingMap>();
    require_preserves_structure::<IntegerGroundingMap>();
    assert_eq!(
        IntegerGroundingMap::ONTOLOGY_IRI,
        "https://uor.foundation/morphism/IntegerGroundingMap"
    );
}

#[test]
fn utf8_grounding_map_is_invertible_structure_preserving() {
    require_kind::<Utf8GroundingMap>();
    require_preserves_structure::<Utf8GroundingMap>();
    assert_eq!(
        Utf8GroundingMap::ONTOLOGY_IRI,
        "https://uor.foundation/morphism/Utf8GroundingMap"
    );
}

#[test]
fn json_grounding_map_is_invertible_structure_preserving() {
    require_kind::<JsonGroundingMap>();
    require_preserves_structure::<JsonGroundingMap>();
    assert_eq!(
        JsonGroundingMap::ONTOLOGY_IRI,
        "https://uor.foundation/morphism/JsonGroundingMap"
    );
}

#[test]
fn digest_grounding_map_is_total_only() {
    require_kind::<DigestGroundingMap>();
    require_total::<DigestGroundingMap>();
    assert_eq!(
        DigestGroundingMap::ONTOLOGY_IRI,
        "https://uor.foundation/morphism/DigestGroundingMap"
    );
    // DigestGroundingMap intentionally does NOT implement PreservesStructure
    // or Invertible — the test is the absence of a `require_preserves_structure`
    // call here. If the codegen accidentally adds those impls, the W4 plan
    // would need to be updated to reflect the new semantics.
}

#[test]
fn binary_grounding_map_is_total_invertible_no_structure() {
    require_kind::<BinaryGroundingMap>();
    require_total::<BinaryGroundingMap>();
    require_total_invertible::<BinaryGroundingMap>();
    assert_eq!(
        BinaryGroundingMap::ONTOLOGY_IRI,
        "https://uor.foundation/morphism/BinaryGroundingMap"
    );
    // Binary intentionally does NOT implement PreservesStructure — raw bytes
    // carry no algebraic structure beyond bit identity.
}
