//! Content-bearing angular storage tests (issue #245).
//!
//! Before the fix, `index_sentence_internal` routed every sentence with
//! `state_vector = None`, so stored `CorpusItem.state_vector`s were slices
//! of whatever the session brain state happened to be at index time —
//! independent of the sentence's words. These tests pin the repaired
//! behavior: stored vectors derive from sentence content.

use uor_r4_router::UorR4Router;

const ID: &str = "user:test";

fn nonzero(v: &[f64]) -> bool {
    v.iter().any(|&x| x != 0.0)
}

fn cosine(a: &[f64], b: &[f64]) -> f64 {
    let dot: f64 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let na: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let nb: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    dot / (na * nb)
}

#[test]
fn different_sentences_store_different_state_vectors() {
    let mut router = UorR4Router::new(0.5);
    router.index_sentence("the galaxy rotates around a supermassive black hole", ID);
    router.index_sentence(
        "bread dough rises because yeast produces carbon dioxide",
        ID,
    );

    let items = router.corpus_items_for(ID);
    assert_eq!(items.len(), 2, "both sentences indexed");
    let a = &items[0].state_vector;
    let b = &items[1].state_vector;
    assert!(nonzero(a) && nonzero(b), "stored vectors must be populated");
    assert_ne!(
        a, b,
        "different sentences must not store identical state vectors"
    );
}

#[test]
fn stored_vector_is_session_independent_within_a_router() {
    // The same sentence indexed before and after unrelated session
    // activity must store identical vectors: index-time session state must
    // not leak into storage. (Note: vectors are NOT comparable across
    // routers with different vocabulary histories — word→prime assignment
    // is arrival-ordered, so the address space is vocabulary-relative.)
    let sentence = "prime numbers thin out along the number line";

    let mut router = UorR4Router::new(0.5);
    router.index_sentence(sentence, "user:one");

    // Perturb session/vocabulary state with unrelated indexing.
    router.index_sentence(
        "volcanic eruptions reshape coastlines over centuries",
        "user:noise",
    );
    router.index_sentence("glaciers carve valleys as they advance", "user:noise");

    router.index_sentence(sentence, "user:two");

    let v1 = router.corpus_items_for("user:one")[0].state_vector.clone();
    let items2 = router.corpus_items_for("user:two");
    assert_eq!(items2.len(), 1);
    let v2 = items2[0].state_vector.clone();
    assert_eq!(
        v1, v2,
        "same sentence must store the same content-derived vector regardless of session activity"
    );
}

#[test]
fn matching_query_scores_closer_than_unrelated_sentence() {
    // DoD retrieval fixture: for a query sharing content with sentence A,
    // the stored vector of A must be closer (cosine) than the stored
    // vector of unrelated sentence B.
    let mut router = UorR4Router::new(0.5);
    let a = "planets orbit the sun in elliptical paths";
    let b = "the chef seasoned the soup with fresh basil";
    router.index_sentence(a, ID);
    router.index_sentence(b, ID);

    // Build the query vector through the same public surface: index the
    // exact text of A under a scratch identity and read its stored vector —
    // content-determinism (previous test) guarantees it equals A's stored
    // vector, so cos(q, A) = 1 while B differs.
    router.index_sentence(a, "user:scratch");
    let q = router.corpus_items_for("user:scratch")[0]
        .state_vector
        .clone();

    let items = router.corpus_items_for(ID);
    let (va, vb) = if items[0].sentence == a {
        (&items[0].state_vector, &items[1].state_vector)
    } else {
        (&items[1].state_vector, &items[0].state_vector)
    };

    let ca = cosine(&q, va);
    let cb = cosine(&q, vb);
    assert!(
        ca > cb,
        "content-matching sentence must score closer: cos(A)={ca} vs cos(B)={cb}"
    );
}
