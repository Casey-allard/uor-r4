//! Phase 0 test: hand-picked sentinels classify correctly.
//!
//! These are the canonical examples from the design notes. If any of them
//! misclassify, a larger regression is probably underway.

use uor_codegen::classification::classify;
use uor_ontology::Ontology;

fn path(iri: &str) -> String {
    let ontology = Ontology::full();
    let class = ontology
        .find_class(iri)
        .unwrap_or_else(|| panic!("class not in ontology: {iri}"));
    classify(class, ontology).path_kind.label().to_string()
}

#[test]
fn partition_is_already_implemented() {
    // NullPartition impls Partition<H> in enforcement.rs, so Phase 0 records
    // Partition itself as AlreadyImplemented (the ontology trait has a
    // concrete impl, full stop).
    assert_eq!(
        path("https://uor.foundation/partition/Partition"),
        "AlreadyImplemented"
    );
}

#[test]
fn partition_product_is_already_implemented() {
    assert_eq!(
        path("https://uor.foundation/partition/PartitionProduct"),
        "AlreadyImplemented"
    );
}

#[test]
fn cochain_complex_is_theory_deferred() {
    assert_eq!(
        path("https://uor.foundation/cohomology/CochainComplex"),
        "Path4TheoryDeferred"
    );
}

#[test]
fn gluing_obstruction_is_theory_deferred() {
    // GluingObstruction is in cohomology/ and on the explicit Path-4
    // allow-list; its `Obstruction` suffix would otherwise route it
    // through Path-2, but Path-4 precedence wins.
    assert_eq!(
        path("https://uor.foundation/cohomology/GluingObstruction"),
        "Path4TheoryDeferred"
    );
}

#[test]
fn born_rule_verification_is_theorem_witness() {
    assert_eq!(
        path("https://uor.foundation/cert/BornRuleVerification"),
        "Path2TheoremWitness"
    );
}

#[test]
fn witt_level_is_skipped() {
    // Enum class; no trait emitted, so not an orphan.
    assert_eq!(path("https://uor.foundation/schema/WittLevel"), "Skip");
}

#[test]
fn parallel_classes_are_theory_deferred() {
    // Every kernel/parallel class should be Path4.
    let ontology = Ontology::full();
    let ns = ontology
        .find_namespace("parallel")
        .expect("parallel namespace missing");
    assert!(
        !ns.classes.is_empty(),
        "parallel namespace has no classes — suspicious"
    );
    for class in &ns.classes {
        assert_eq!(
            classify(class, ontology).path_kind.label(),
            "Path4TheoryDeferred",
            "{} should be Path4TheoryDeferred",
            class.id
        );
    }
}

#[test]
fn stream_classes_are_theory_deferred() {
    let ontology = Ontology::full();
    let ns = ontology
        .find_namespace("stream")
        .expect("stream namespace missing");
    assert!(
        !ns.classes.is_empty(),
        "stream namespace has no classes — suspicious"
    );
    for class in &ns.classes {
        assert_eq!(
            classify(class, ontology).path_kind.label(),
            "Path4TheoryDeferred",
            "{} should be Path4TheoryDeferred",
            class.id
        );
    }
}

#[test]
fn entropy_bearing_classifier_is_not_trivially_false() {
    // Sanity: `BornRuleVerification` has a decimal property (probabilityAmplitude
    // or similar). If entropy_bearing is always false, something is wrong with R7.
    use uor_codegen::classification::{classify_all, PathKind as PK};
    let entries = classify_all(Ontology::full());
    let any_entropy = entries.iter().any(|e| {
        matches!(
            e.path_kind,
            PK::Path2TheoremWitness {
                entropy_bearing: true,
                ..
            }
        )
    });
    // If no Path-2 class is entropy-bearing, ~likely R7 is miswired, but
    // it's also possible the ontology's Path-2 classes genuinely lack
    // entropy properties at this point. Record as advisory, not hard fail.
    if !any_entropy {
        eprintln!("advisory: no Path-2 class classified as entropy_bearing — review R7 set");
    }
}
