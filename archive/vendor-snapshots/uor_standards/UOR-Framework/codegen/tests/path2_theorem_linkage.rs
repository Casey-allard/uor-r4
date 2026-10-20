//! Phase 10a verification: every Path-2 classification carries a
//! `theorem_identity` that resolves to a real `op:Identity` individual
//! AND routes to a known primitive-module via Phase 10d's
//! `primitive_module_for_identity`.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::collections::HashSet;

use uor_codegen::classification::{
    classify_all, identity_to_snake, primitive_module_for_identity, PathKind,
};
use uor_ontology::Ontology;

#[test]
fn every_path2_class_resolves_to_real_identity() {
    let ontology = Ontology::full();
    let entries = classify_all(ontology);

    let identity_iris: HashSet<&'static str> = ontology
        .namespaces
        .iter()
        .flat_map(|m| m.individuals.iter())
        .filter(|i| i.type_ == "https://uor.foundation/op/Identity")
        .map(|i| i.id)
        .collect();

    let mut path2_count = 0usize;
    for e in &entries {
        if let PathKind::Path2TheoremWitness {
            theorem_identity, ..
        } = &e.path_kind
        {
            path2_count += 1;
            assert!(
                !theorem_identity.is_empty(),
                "Path-2 class `{}` has empty theorem_identity",
                e.class_iri
            );
            assert!(
                identity_iris.contains(theorem_identity.as_str()),
                "Path-2 class `{}` resolves to `{}` which is NOT a real op:Identity",
                e.class_iri,
                theorem_identity,
            );
        }
    }
    assert!(
        path2_count > 0,
        "expected at least one Path-2 classification"
    );
}

#[test]
fn every_path2_identity_routes_to_known_primitive_module() {
    let ontology = Ontology::full();
    let entries = classify_all(ontology);

    let known_modules: HashSet<&str> =
        ["pt", "st", "cpt", "ob", "ih", "lo", "oa", "br", "cc", "dp"]
            .into_iter()
            .collect();

    for e in &entries {
        if let PathKind::Path2TheoremWitness {
            theorem_identity, ..
        } = &e.path_kind
        {
            let module = primitive_module_for_identity(theorem_identity);
            assert!(
                known_modules.contains(module),
                "Path-2 class `{}` (identity `{}`) routes to unknown module `{}`",
                e.class_iri,
                theorem_identity,
                module,
            );
        }
    }
}

#[test]
fn identity_to_snake_known_cases() {
    assert_eq!(identity_to_snake("https://uor.foundation/op/PT_1"), "pt_1");
    assert_eq!(
        identity_to_snake("https://uor.foundation/op/PT_2a"),
        "pt_2a"
    );
    assert_eq!(
        identity_to_snake("https://uor.foundation/op/CPT_2a"),
        "cpt_2a"
    );
    assert_eq!(
        identity_to_snake("https://uor.foundation/op/OB_M1"),
        "ob_m1",
    );
    assert_eq!(
        identity_to_snake("https://uor.foundation/op/surfaceSymmetry"),
        "surface_symmetry",
    );
    assert_eq!(
        identity_to_snake("https://uor.foundation/op/WLS_2"),
        "wls_2"
    );
}
