//! Phase 0 test: every class in `Ontology::full()` receives a deterministic
//! classification; totals match `uor_ontology::counts::CLASSES`.

use uor_codegen::classification::{classify, classify_all, count, PathKind};
use uor_ontology::Ontology;

#[test]
fn every_class_has_a_classification() {
    let ontology = Ontology::full();
    let entries = classify_all(ontology);
    assert_eq!(
        entries.len(),
        ontology.class_count(),
        "classify_all should produce one entry per ontology class"
    );
}

#[test]
fn counts_total_matches_class_count() {
    let ontology = Ontology::full();
    let entries = classify_all(ontology);
    let c = count(&entries);
    assert_eq!(
        c.total(),
        uor_ontology::counts::CLASSES,
        "sum of per-variant counts must equal uor_ontology::counts::CLASSES"
    );
}

#[test]
fn classification_is_deterministic() {
    let ontology = Ontology::full();
    for module in &ontology.namespaces {
        for class in &module.classes {
            let a = classify(class, ontology);
            let b = classify(class, ontology);
            assert_eq!(
                a.path_kind.label(),
                b.path_kind.label(),
                "classify({}) non-deterministic",
                class.id
            );
        }
    }
}

#[test]
fn every_pathkind_variant_is_labeled() {
    // Sanity check — the label() function covers every variant.
    for label in [
        PathKind::Skip.label(),
        PathKind::AlreadyImplemented.label(),
        PathKind::Path1HandleResolver.label(),
        PathKind::Path4TheoryDeferred.label(),
        PathKind::Path2TheoremWitness {
            entropy_bearing: false,
            theorem_identity: String::new(),
        }
        .label(),
        PathKind::Path3PrimitiveBacked {
            primitive_name: "x".to_string(),
        }
        .label(),
    ] {
        assert!(!label.is_empty());
    }
}
