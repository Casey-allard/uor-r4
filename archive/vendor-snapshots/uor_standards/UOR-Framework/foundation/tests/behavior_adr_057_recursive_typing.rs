//! ADR-057 bounded recursive structural typing.
//!
//! Pins the foundation surface ADR-057 commits:
//!
//! 1. `ConstraintRef::Recurse { shape_iri, descent_bound }` exists as a
//!    variant of `uor_foundation::pipeline::ConstraintRef` carrying a
//!    content-addressed shape IRI and a descent bound.
//! 2. `LeafConstraintRef::Recurse { shape_iri, descent_bound }` exists
//!    as the leaf-level parallel and round-trips through `as_leaf`/
//!    `into_constraint`.
//! 3. `shift_constraint` and `shift_leaf_constraint` pass Recurse through
//!    unchanged (no site references to shift).
//! 4. `pipeline::shape_iri_registry::RegisteredShape` carries the four
//!    fields the wiki spec names: iri, site_count, constraints, cycle_size.
//! 5. `pipeline::shape_iri_registry::lookup_shape` is publicly callable
//!    and returns `None` on an unregistered IRI.
//! 6. The wire-format discriminant byte for Recurse is 10 (one more than
//!    Conjunction = 9), per ADR-057's wire-format integration rule.

use uor_foundation::enforcement::Hasher;
use uor_foundation::pipeline::shape_iri_registry::RegisteredShape;
use uor_foundation::pipeline::{
    shape_iri_registry, shift_constraint, shift_leaf_constraint, ConstraintRef, LeafConstraintRef,
};

#[test]
fn constraint_ref_recurse_variant_carries_shape_iri_and_descent_bound() {
    let c = ConstraintRef::Recurse {
        shape_iri: "urn:test:json_value",
        descent_bound: 32,
    };
    match c {
        ConstraintRef::Recurse {
            shape_iri,
            descent_bound,
        } => {
            assert_eq!(shape_iri, "urn:test:json_value");
            assert_eq!(descent_bound, 32);
        }
        other => panic!("expected Recurse, got {other:?}"),
    }
}

#[test]
fn leaf_constraint_ref_recurse_round_trips_via_as_leaf_into_constraint() {
    let c = ConstraintRef::Recurse {
        shape_iri: "urn:test:xml_node",
        descent_bound: 16,
    };
    let leaf = c.as_leaf();
    match leaf {
        LeafConstraintRef::Recurse {
            shape_iri,
            descent_bound,
        } => {
            assert_eq!(shape_iri, "urn:test:xml_node");
            assert_eq!(descent_bound, 16);
        }
        other => panic!("expected LeafConstraintRef::Recurse, got {other:?}"),
    }
    let back = leaf.into_constraint();
    match back {
        ConstraintRef::Recurse {
            shape_iri,
            descent_bound,
        } => {
            assert_eq!(shape_iri, "urn:test:xml_node");
            assert_eq!(descent_bound, 16);
        }
        other => panic!("expected ConstraintRef::Recurse round-trip, got {other:?}"),
    }
}

#[test]
fn shift_constraint_passes_recurse_through_unchanged() {
    // Recurse references a shape by IRI — no site positions to shift.
    // The shift_constraint helper should pass Recurse through unchanged
    // regardless of offset.
    let c = ConstraintRef::Recurse {
        shape_iri: "urn:test:sexpr",
        descent_bound: 64,
    };
    let shifted = shift_constraint(c, 42);
    match shifted {
        ConstraintRef::Recurse {
            shape_iri,
            descent_bound,
        } => {
            assert_eq!(shape_iri, "urn:test:sexpr");
            assert_eq!(descent_bound, 64);
        }
        other => panic!("expected Recurse pass-through, got {other:?}"),
    }
}

#[test]
fn shift_leaf_constraint_passes_recurse_through_unchanged() {
    let c = LeafConstraintRef::Recurse {
        shape_iri: "urn:test:protobuf_msg",
        descent_bound: 8,
    };
    let shifted = shift_leaf_constraint(c, 100);
    match shifted {
        LeafConstraintRef::Recurse {
            shape_iri,
            descent_bound,
        } => {
            assert_eq!(shape_iri, "urn:test:protobuf_msg");
            assert_eq!(descent_bound, 8);
        }
        other => panic!("expected LeafConstraintRef::Recurse pass-through, got {other:?}"),
    }
}

#[test]
fn shape_iri_registry_registered_shape_carries_canonical_fields() {
    // Verify the RegisteredShape struct has the four fields the wiki
    // spec names. Construct one by hand and read each accessor.
    static EMPTY_CONSTRAINTS: &[ConstraintRef] = &[];
    let entry = RegisteredShape {
        iri: "urn:test:registered_shape_1",
        site_count: 7,
        constraints: EMPTY_CONSTRAINTS,
        cycle_size: u64::MAX,
    };
    assert_eq!(entry.iri, "urn:test:registered_shape_1");
    assert_eq!(entry.site_count, 7);
    assert!(entry.constraints.is_empty());
    assert_eq!(entry.cycle_size, u64::MAX);
}

#[test]
fn shape_iri_registry_lookup_returns_none_for_unregistered_iri() {
    // Foundation's built-in registry is empty by default (standard-library
    // Layer-3 sub-crates per ADR-031 publishing canonical shapes register
    // through this path in future foundation-curated additions; the trait-
    // based `ShapeRegistryProvider` path admits application registries
    // independently). An unregistered IRI returns `None`.
    assert!(shape_iri_registry::lookup_shape("urn:nonexistent:shape").is_none());
}

// ── Wire-format discriminant byte for Recurse ─────────────────────────

/// Minimal Hasher that records folded bytes so we can pin the discriminant.
#[derive(Debug, Clone, Copy, Default)]
struct ByteRecorder {
    first_byte: Option<u8>,
}
impl Hasher for ByteRecorder {
    const OUTPUT_BYTES: usize = 1;
    fn initial() -> Self {
        Self::default()
    }
    fn fold_byte(self, b: u8) -> Self {
        if self.first_byte.is_none() {
            Self {
                first_byte: Some(b),
            }
        } else {
            self
        }
    }
    fn finalize(self) -> [u8; 32] {
        let mut out = [0u8; 32];
        if let Some(b) = self.first_byte {
            out[0] = b;
        }
        out
    }
}

#[test]
fn fold_constraint_ref_emits_discriminant_byte_10_for_recurse() {
    // ADR-057 wire-format: Recurse is discriminant byte 10 (one more
    // than Conjunction = 9).
    let c = ConstraintRef::Recurse {
        shape_iri: "urn:test:ast",
        descent_bound: 4,
    };
    let hasher = ByteRecorder::default();
    let folded = uor_foundation::enforcement::fold_constraint_ref(hasher, &c);
    assert_eq!(
        folded.first_byte,
        Some(10),
        "ADR-057 Recurse must emit discriminant byte 10"
    );
}

// ── End-to-end: register_shape! + lookup_shape_in ──────────────────────
//
// ADR-057 commits the const-aggregated `ShapeRegistryProvider` surface.
// `register_shape!(MyRegistry, Shape1, Shape2, …)` from `uor-foundation-sdk`
// emits a marker type implementing `ShapeRegistryProvider`; foundation's
// `lookup_shape_in::<MyRegistry>(iri)` walks the const-aggregated registry.
// This test exercises the registration path WITHOUT going through the SDK
// macro (since this test file lives in the foundation crate and can't
// invoke proc-macros from `uor-foundation-sdk`). The end-to-end path is
// validated in `uor-foundation-sdk/tests/smoke.rs`'s
// `register_shape_macro_*` tests.

use uor_foundation::pipeline::shape_iri_registry::{EmptyShapeRegistry, ShapeRegistryProvider};
use uor_foundation::pipeline::{__sdk_seal, shape_iri_registry::lookup_shape_in};

/// Hand-rolled marker type mirroring what `register_shape!(TestRegistry, …)`
/// would emit from the SDK macro. The implementation is identical at the
/// trait surface; foundation's lookup_shape_in operates on it the same way.
struct TestRegistry;
impl __sdk_seal::Sealed for TestRegistry {}
impl ShapeRegistryProvider for TestRegistry {
    const REGISTRY: &'static [RegisteredShape] = &[
        RegisteredShape {
            iri: "urn:test:json_value",
            site_count: 4,
            constraints: &[],
            cycle_size: u64::MAX,
        },
        RegisteredShape {
            iri: "urn:test:xml_element",
            site_count: 8,
            constraints: &[],
            cycle_size: u64::MAX,
        },
    ];
}

#[test]
fn empty_shape_registry_default_provides_empty_registry_slice() {
    assert_eq!(
        <EmptyShapeRegistry as ShapeRegistryProvider>::REGISTRY.len(),
        0
    );
}

#[test]
fn lookup_shape_in_finds_application_registered_shape() {
    let entry = lookup_shape_in::<TestRegistry>("urn:test:json_value")
        .expect("registered JSON shape is found");
    assert_eq!(entry.iri, "urn:test:json_value");
    assert_eq!(entry.site_count, 4);
    assert_eq!(entry.cycle_size, u64::MAX);
}

#[test]
fn lookup_shape_in_falls_back_to_foundation_registry_when_app_misses() {
    // The application registry only has json_value and xml_element. A
    // missing IRI returns None (foundation's built-in registry is also
    // empty in v0.4.14; future stdlib-canonical shapes go there).
    assert!(lookup_shape_in::<TestRegistry>("urn:test:nonexistent").is_none());
}

#[test]
fn lookup_shape_in_finds_second_entry() {
    let entry = lookup_shape_in::<TestRegistry>("urn:test:xml_element")
        .expect("registered XML shape is found");
    assert_eq!(entry.iri, "urn:test:xml_element");
    assert_eq!(entry.site_count, 8);
}

#[test]
fn empty_registry_provider_returns_none_via_lookup_shape_in() {
    assert!(lookup_shape_in::<EmptyShapeRegistry>("any-iri").is_none());
}

// ── ADR-057 step 3: Recurse expansion at ψ_1 (structural reading) ───────
//
// These tests pin the wire-faithful behavior the wiki's ADR-057 step 3
// commits: at ψ_1 evaluation, `ConstraintRef::Recurse { shape_iri, descent_bound }`
// expands through the shape-IRI registry — the structural reading
// reflects the **expanded** constraint geometry, not the opaque Recurse
// node. Foundation's `primitive_simplicial_nerve_betti_in<T, R>` is the
// computation; this test compares N(C) before and after expansion.

use uor_foundation::enforcement::primitive_simplicial_nerve_betti_in;
use uor_foundation::pipeline::ConstrainedTypeShape;

/// Outer shape whose only constraint is `Recurse` pointing at
/// `urn:test:recursive_inner` (the inner shape's CONSTRAINTS array is
/// inlined in `RecursiveTestRegistry` below). Before ADR-057 expansion:
/// nerve sees one opaque Site-like constraint with all-sites support.
/// After expansion: nerve sees the inner's two non-overlapping Site
/// constraints.
struct OuterRecShape;
impl ConstrainedTypeShape for OuterRecShape {
    const IRI: &'static str = "urn:test:recursive_outer";
    const SITE_COUNT: usize = 2;
    const CYCLE_SIZE: u64 = u64::MAX; // saturated per ADR-057 cycle-size rule
    const CONSTRAINTS: &'static [uor_foundation::pipeline::ConstraintRef] =
        &[uor_foundation::pipeline::ConstraintRef::Recurse {
            shape_iri: "urn:test:recursive_inner",
            descent_bound: 4,
        }];
}

/// Registry that knows about `urn:test:recursive_inner` with the inner
/// shape's CONSTRAINTS array. The expansion at OuterRecShape's
/// Recurse references this registry's entry.
struct RecursiveTestRegistry;
impl __sdk_seal::Sealed for RecursiveTestRegistry {}
impl ShapeRegistryProvider for RecursiveTestRegistry {
    const REGISTRY: &'static [RegisteredShape] = &[RegisteredShape {
        iri: "urn:test:recursive_inner",
        site_count: 2,
        constraints: &[
            uor_foundation::pipeline::ConstraintRef::Site { position: 0 },
            uor_foundation::pipeline::ConstraintRef::Site { position: 1 },
        ],
        cycle_size: 4,
    }];
}

#[test]
fn primitive_simplicial_nerve_betti_in_returns_unregistered_without_registry() {
    // OuterRecShape's only constraint is Recurse pointing at
    // urn:test:recursive_inner. EmptyShapeRegistry's REGISTRY is empty
    // and foundation's FOUNDATION_REGISTRY is also empty in v0.4.15,
    // so lookup fails. ADR-057: the expansion errors with
    // RECURSE_SHAPE_UNREGISTERED rather than silently treating Recurse
    // as an anonymous Site.
    let result = primitive_simplicial_nerve_betti_in::<OuterRecShape, EmptyShapeRegistry>();
    assert!(
        result.is_err(),
        "ADR-057: unregistered shape must error, not silently fall through",
    );
}

#[test]
fn primitive_simplicial_nerve_betti_in_expands_via_registered_shape() {
    // RecursiveTestRegistry contains urn:test:recursive_inner with two
    // non-overlapping Site constraints (positions 0, 1). After
    // expansion the nerve sees two vertices, no overlapping pairs ⇒
    // b_0 = 2 (two disconnected components), b_1 = 0, b_2 = 0.
    let betti = primitive_simplicial_nerve_betti_in::<OuterRecShape, RecursiveTestRegistry>()
        .expect("recursive shape expands through the registered inner");
    assert_eq!(
        betti[0], 2,
        "two disconnected Site constraints ⇒ b_0 = 2 after expansion",
    );
    assert_eq!(betti[1], 0, "no 1-simplices ⇒ b_1 = 0");
    assert_eq!(betti[2], 0, "no 2-simplices ⇒ b_2 = 0");
}

#[test]
fn primitive_simplicial_nerve_betti_default_path_consults_foundation_registry_only() {
    // The default-path primitive (no R parameter) delegates to
    // primitive_simplicial_nerve_betti_in::<T, EmptyShapeRegistry>().
    // Since FOUNDATION_REGISTRY is empty for v0.4.15, an OuterRecShape
    // call must error with RECURSE_SHAPE_UNREGISTERED — proving the
    // default path is registry-aware, not Recurse-opaque.
    let result = uor_foundation::enforcement::primitive_simplicial_nerve_betti::<OuterRecShape>();
    assert!(
        result.is_err(),
        "default-path: unregistered Recurse must propagate the registry error",
    );
}

#[test]
fn primitive_simplicial_nerve_betti_in_bottoms_out_at_zero_descent_bound() {
    // Shape with descent_bound = 0 — the Recurse contributes no
    // further constraints. The expanded constraint set is empty;
    // nerve of empty complex returns b_0 = 1 by convention.
    struct ZeroBoundRecShape;
    impl ConstrainedTypeShape for ZeroBoundRecShape {
        const IRI: &'static str = "urn:test:zero_bound_outer";
        const SITE_COUNT: usize = 2;
        const CYCLE_SIZE: u64 = u64::MAX;
        const CONSTRAINTS: &'static [uor_foundation::pipeline::ConstraintRef] =
            &[uor_foundation::pipeline::ConstraintRef::Recurse {
                shape_iri: "urn:test:recursive_inner",
                descent_bound: 0,
            }];
    }
    let betti = primitive_simplicial_nerve_betti_in::<ZeroBoundRecShape, RecursiveTestRegistry>()
        .expect("zero-bound Recurse bottoms out — empty expansion");
    assert_eq!(
        betti[0], 1,
        "zero-bound Recurse ⇒ empty expansion ⇒ b_0 = 1 (empty-complex convention)",
    );
}

#[test]
fn null_resolver_tuple_carries_empty_shape_registry_by_default() {
    // ADR-057: ResolverTuple gains `type ShapeRegistry` associated
    // type. NullResolverTuple defaults to EmptyShapeRegistry —
    // applications declaring `resolver!` blocks without a
    // `shape_registry:` clause inherit the same default. The associated
    // type's REGISTRY const is observably the empty slice, so we can
    // assert on its observable behavior (rather than just naming the
    // type).
    use uor_foundation::pipeline::{NullResolverTuple, ResolverTuple};
    let registry =
        <<NullResolverTuple as ResolverTuple>::ShapeRegistry as ShapeRegistryProvider>::REGISTRY;
    assert!(
        registry.is_empty(),
        "NullResolverTuple's default ShapeRegistry must have an empty REGISTRY",
    );
}
