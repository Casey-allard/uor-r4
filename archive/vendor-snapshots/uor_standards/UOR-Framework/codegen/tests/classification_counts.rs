//! Phase 0 test: per-variant classification counts match the constants in
//! `spec/src/counts.rs`. Drift between the ontology and the classification
//! fails this test.

use uor_codegen::classification::{classify_all, count};
use uor_ontology::counts as C;
use uor_ontology::Ontology;

#[test]
fn counts_match_constants() {
    let entries = classify_all(Ontology::full());
    let c = count(&entries);

    assert_eq!(c.skip, C::CLASSIFICATION_SKIP, "CLASSIFICATION_SKIP drift");
    assert_eq!(
        c.already_implemented,
        C::CLASSIFICATION_ALREADY_IMPLEMENTED,
        "CLASSIFICATION_ALREADY_IMPLEMENTED drift"
    );
    assert_eq!(
        c.path1,
        C::CLASSIFICATION_PATH1,
        "CLASSIFICATION_PATH1 drift"
    );
    assert_eq!(
        c.path2,
        C::CLASSIFICATION_PATH2,
        "CLASSIFICATION_PATH2 drift"
    );
    assert_eq!(
        c.path3,
        C::CLASSIFICATION_PATH3,
        "CLASSIFICATION_PATH3 drift"
    );
    assert_eq!(
        c.path4,
        C::CLASSIFICATION_PATH4,
        "CLASSIFICATION_PATH4 drift"
    );
}

#[test]
fn counts_sum_to_class_total() {
    // Defense-in-depth: total must equal CLASSES even if per-variant
    // counts are individually drifted.
    let sum = C::CLASSIFICATION_SKIP
        + C::CLASSIFICATION_ALREADY_IMPLEMENTED
        + C::CLASSIFICATION_PATH1
        + C::CLASSIFICATION_PATH2
        + C::CLASSIFICATION_PATH3
        + C::CLASSIFICATION_PATH4;
    assert_eq!(
        sum,
        C::CLASSES,
        "sum of CLASSIFICATION_* constants ({sum}) must equal CLASSES ({})",
        C::CLASSES,
    );
}

#[test]
fn skip_count_matches_enum_class_count() {
    assert_eq!(
        C::CLASSIFICATION_SKIP,
        Ontology::enum_class_names().len(),
        "CLASSIFICATION_SKIP should equal enum_class_names().len()"
    );
}
