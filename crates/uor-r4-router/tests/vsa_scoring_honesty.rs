//! #493 (disposition of #434 item 2): characterize what VSA retrieval actually
//! does after corpus ingestion, and guard the corrected record.
//!
//! #434 recorded the VSA arm's `0.0000` as a *wiring gap* — "index_corpus
//! populates `corpus_index_by_identity` and never touches `facet_store`". That
//! is no longer true (and may never have been on current code): `index_corpus`
//! calls `index_sentence_internal` → `index_sentence_routed`, which grounds each
//! sentence via `VsaGeometry` and calls `index_semantic_object`, so the facet
//! store IS populated. The zero has a different cause: the scorer compares the
//! query's 1024-dim VSA hypervector against the stored 512-dim SPECTRAL content
//! vector, and `cosine_similarity` returns exactly 0.0 on a length mismatch, so
//! every candidate scores 0.0 and the ranking is dead.
//!
//! This test pins both facts so the record cannot silently drift back, and so a
//! future change to VSA scoring is forced to update it (and confront that even a
//! commensurable comparison ranks at chance — the grounding is content-hash
//! derived, not semantic; see #493 and `docs/geometry_ablation_434.md`).

use uor_r4_router::UorR4Router;

const CORPUS: &str = "The quick brown fox jumps over the lazy dog near the river. \
Telescopes gather light from distant ancient stars across the night sky. \
Bread dough rises slowly when the yeast produces carbon dioxide gas. \
Glaciers carve deep valleys as they advance slowly over many centuries. \
The chef seasoned the tomato soup with fresh basil and cracked pepper.";

#[test]
fn vsa_after_index_corpus_populates_facets_but_scores_degenerate() {
    let mut router = UorR4Router::new(0.5);
    router.clear_corpus();
    router.set_geometry_type("vsa");

    let indexed = router.index_corpus(CORPUS, "shared");
    assert_eq!(
        indexed, 5,
        "all five sentences pass the index_corpus filter"
    );

    // Corrected fact #1: index_corpus DOES populate the facet store (contra the
    // #434 record). If this regresses to empty, the VSA candidate set dies for a
    // different reason and the record needs revisiting.
    let facet_keys = router.facet_store.type_index.len();
    assert!(
        facet_keys > 0,
        "index_corpus must populate facet_store.type_index (it did not: {facet_keys} keys) \
         — the #434 'never touches facet_store' claim is what this guards against"
    );

    // Corrected fact #2: retrieval returns the candidate SET, but every
    // relevance is exactly 0.0 — the 1024-vs-512 length mismatch in the scorer.
    // This is the honest signature of the dead ranking. A change that makes the
    // scoring commensurable will break this assertion ON PURPOSE: update it only
    // together with the disposition (a real VSA encoder), because a re-grounded
    // hypervector cosine still ranks at chance (grounding is content-hash based).
    let res = router.get_top_resonances_native("fox jumps over the lazy dog", "shared", 5);
    assert!(
        !res.is_empty(),
        "facet intersection should still return the candidate set"
    );
    assert!(
        res.iter().all(|r| r.relevance == 0.0),
        "VSA scoring is degenerate on a spectral store (every relevance must be exactly 0.0); \
         got {:?} — if this changed, see #493 before updating",
        res.iter().map(|r| r.relevance).collect::<Vec<_>>()
    );
}
