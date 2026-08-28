//! Natural, source-free lexical table vertical slice for issue #989.
//!
//! Excerpts are from the pinned Simple English Wikipedia D3 corpus
//! (CC-BY-SA-4.0; Wikimedia contributors) and retain their corpus article ids.

use uor_r4_core::source_free_table::{
    d3_is_held_out, BackoffOrder, ContinuationStop, SourceDocument, SourceFreeTable,
};

fn construction_documents() -> Vec<SourceDocument> {
    vec![
        SourceDocument::new("14", b"She was born in Ottawa, Canada.".to_vec()),
        SourceDocument::new("657", b"He was born in Bombay, India.".to_vec()),
        SourceDocument::new(
            "4579",
            b"Alexander Graham Bell was born in Edinburgh, Scotland.".to_vec(),
        ),
        SourceDocument::new("5121", b"He was born in Shrewsbury, Shropshire.".to_vec()),
    ]
}

fn held_out_documents() -> Vec<SourceDocument> {
    vec![SourceDocument::new(
        "13",
        "Alan Mathison Turing OBE FRS (London, 23 June 1912 – Wilmslow, Cheshire, 7 June 1954) was an English mathematician and computer scientist. He was born in Maida Vale, London."
            .as_bytes()
            .to_vec(),
    )]
}

fn has_period_one_or_two_cycle(tokens: &[u32]) -> bool {
    tokens.windows(2).any(|window| window[0] == window[1])
        || tokens
            .windows(4)
            .any(|window| window[0] == window[2] && window[1] == window[3])
}

#[test]
fn natural_held_out_choice_beats_unigram_and_decodes_continuation() {
    let construction = construction_documents();
    let held_out = held_out_documents();
    assert!(construction
        .iter()
        .all(|document| !d3_is_held_out(&document.id)));
    assert!(held_out.iter().all(|document| d3_is_held_out(&document.id)));

    let table = SourceFreeTable::compile(&construction).expect("construction-only table compiles");
    let held_tokens = table
        .encode_text(&held_out[0].text)
        .expect("held-out text encodes with byte fallback");
    assert_eq!(
        table.decode_tokens(&held_tokens).unwrap(),
        held_out[0].text,
        "held-out payload must invert without fitting its vocabulary"
    );

    let evaluation = table
        .evaluate_held_out(&held_out)
        .expect("D3 held-out evaluation is disjoint");
    assert!(evaluation.changed_choices > 0);
    assert!(evaluation.changed_choice_correct > 0);
    assert!(evaluation.known_target_positions > 0);
    assert!(
        evaluation.table_correct > evaluation.unigram_correct,
        "context table must improve the natural held-out fixture: {evaluation:?}"
    );

    let seed = table.encode_text(b"He was born").unwrap();
    let prediction = table.predict(&seed);
    assert_eq!(prediction.order, BackoffOrder::Trigram);
    assert_eq!(table.decode_tokens(&[prediction.token]).unwrap(), b" in");

    let continuation = table.continue_text(b"He was born", 8).unwrap();
    assert!(continuation.tokens.len() >= 4, "{continuation:?}");
    assert!(!has_period_one_or_two_cycle(&continuation.tokens));
    assert_ne!(continuation.stop, ContinuationStop::PeriodOneCycle);
    assert_ne!(continuation.stop, ContinuationStop::PeriodTwoCycle);
    assert!(
        continuation.decoded.starts_with(b" in Bombay, India"),
        "decoded continuation was {:?}",
        String::from_utf8_lossy(&continuation.decoded)
    );

    let transcript = table.canonical_transcript_bytes(&evaluation, b"He was born", &continuation);
    assert_eq!(
        transcript,
        table.canonical_transcript_bytes(&evaluation, b"He was born", &continuation)
    );
    let transcript_text = std::str::from_utf8(&transcript).unwrap();
    println!("{transcript_text}");
    for zero_counter in [
        "teacher_calls=0",
        "provider_calls=0",
        "source_weight_reads=0",
        "geometry_calls=0",
    ] {
        assert!(transcript_text.contains(zero_counter), "{transcript_text}");
    }
}

#[test]
fn artifact_reloads_byte_identically_and_preserves_predictions() {
    let table = SourceFreeTable::compile(&construction_documents()).unwrap();
    let bytes = table.to_bytes();
    let reloaded = SourceFreeTable::from_bytes(&bytes).expect("canonical artifact reloads");
    assert_eq!(reloaded.to_bytes(), bytes);
    assert_eq!(reloaded.artifact_cid(), table.artifact_cid());

    let context = table.encode_text(b"He was born").unwrap();
    assert_eq!(reloaded.predict(&context), table.predict(&context));
    assert_eq!(
        reloaded.continue_text(b"He was born", 8).unwrap(),
        table.continue_text(b"He was born", 8).unwrap()
    );

    let held_out = held_out_documents();
    let evaluation = table.evaluate_held_out(&held_out).unwrap();
    let continuation = table.continue_text(b"He was born", 8).unwrap();
    assert_eq!(
        reloaded.canonical_transcript_bytes(&evaluation, b"He was born", &continuation,),
        table.canonical_transcript_bytes(&evaluation, b"He was born", &continuation)
    );
}

#[test]
fn held_out_overlap_is_rejected_before_scoring() {
    let table = SourceFreeTable::compile(&construction_documents()).unwrap();
    let held_out_id = held_out_documents()[0].id.clone();
    let construction_text = construction_documents()[0].text.clone();
    let overlap = SourceDocument::new(held_out_id, construction_text);
    let error = table.evaluate_held_out(&[overlap]).unwrap_err();
    assert!(error.to_string().contains("construction text CID"));

    let repeated_text = b"This held-out text is repeated.".to_vec();
    let repeated_held_out = [
        SourceDocument::new("12", repeated_text.clone()),
        SourceDocument::new("13", repeated_text),
    ];
    let evaluation = table
        .evaluate_held_out(&repeated_held_out)
        .expect("same-partition text repetition is allowed");
    assert_eq!(evaluation.documents, 2);
}
