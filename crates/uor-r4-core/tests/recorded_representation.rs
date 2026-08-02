use uor_r4_core::transformerless::compiler::{Corpus, RecordedRepresentation, D};
use uor_r4_model_source::RepresentationSource;

fn corpus() -> Corpus {
    Corpus {
        n: 2,
        stories: 1,
        story: vec![0, 0],
        input: vec![1, 2],
        next: vec![2, 3],
        t_argmax: vec![2, 3],
        top_tokens: vec![[2, 0, 0, 0, 0, 0, 0, 0], [3, 1, 0, 0, 0, 0, 0, 0]],
        top_weights: vec![[100, 0, 0, 0, 0, 0, 0, 0], [70, 30, 0, 0, 0, 0, 0, 0]],
        span_start: vec![0, 1],
        span_end: vec![1, 2],
        byte_start: vec![u32::MAX; 2],
        byte_end: vec![u32::MAX; 2],
        hidden: None,
    }
}

#[test]
fn recorded_representation_uses_records_without_hidden_sidecar() {
    let first = RecordedRepresentation::from_corpus(&corpus(), 4).expect("records are enough");
    let second = RecordedRepresentation::from_corpus(&corpus(), 4).expect("deterministic");
    assert_eq!(first.kappa(), second.kappa());
    assert_eq!(first.vocab_size(), 4);
    assert_eq!(first.source_dimension(), D);

    let mut rows = vec![0.0; 4 * D];
    first
        .read_embedding_rows(0..4, &mut rows)
        .expect("rows fit");
    assert!(rows.iter().any(|value| *value != 0.0));
}
