//! Issue #259: stored geometric coordinates carry verifiable UOR provenance.

use uor_r4_router::UorR4Router;

const ID: &str = "user:provenance";

#[test]
fn indexed_coordinates_are_populated_and_verifiable() {
    let mut router = UorR4Router::new(0.5);
    let sentence = "telescopes gather light from distant ancient stars";
    router.index_sentence(sentence, ID);

    let items = router.corpus_items_for(ID);
    assert_eq!(items.len(), 1);
    let provenance = items[0]
        .provenance
        .as_ref()
        .expect("newly indexed coordinates carry provenance")
        .clone();
    assert!(provenance.source_kappa.starts_with("sha256:"));
    assert!(provenance.projection_kappa.starts_with("sha256:"));
    assert!(provenance.vocabulary_kappa.starts_with("sha256:"));
    assert_eq!(router.verify_corpus_provenance(ID), Ok(1));

    let retrieved = router.get_top_resonances_native("distant stars", ID, 1);
    assert_eq!(retrieved.len(), 1);
    assert_eq!(retrieved[0].sentence, sentence);
    assert_eq!(retrieved[0].provenance.as_ref(), Some(&provenance));
}

#[test]
fn content_free_measurement_arm_still_records_provenance() {
    let mut router = UorR4Router::new(0.5);
    router.index_sentence_content_free("a legacy coordinate without evidence", ID);

    // Content-free indexing is still an intentionally supported measurement
    // arm, but it now records provenance as well; verification must not be
    // silently skipped for that path.
    assert_eq!(router.verify_corpus_provenance(ID), Ok(1));
}
