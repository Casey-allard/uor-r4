//! Frozen natural preflight for #953 `MultiscaleCountRadiusR4V1`.
//!
//! The real 3,000-document held-out result is intentionally absent here. This
//! fixture checks the predeclared frame, support, matched work, decode, and
//! artifact binding before that decision-bearing run is allowed.

use uor_r4_core::source_free_table::{
    d3_is_held_out, BackoffOrder, ContinuationStop, MultiscaleCountRadiusR4V1, SourceDocument,
    SourceFreeTable,
};

fn construction_documents() -> Vec<SourceDocument> {
    vec![
        SourceDocument::new("14", b"The red fox rests.".to_vec()),
        SourceDocument::new("657", b"The red fox runs.".to_vec()),
        SourceDocument::new("4579", b"The team runs.".to_vec()),
        SourceDocument::new("5121", b"The athlete runs.".to_vec()),
    ]
}

fn held_out_document() -> SourceDocument {
    SourceDocument::new("13", b"At dusk the red fox runs.".to_vec())
}

fn decoded(table: &SourceFreeTable, token: u32) -> Vec<u8> {
    table.decode_tokens(&[token]).unwrap()
}

#[test]
fn natural_tie_has_identical_support_and_load_bearing_r4_radius() {
    let construction = construction_documents();
    let held_out = held_out_document();
    assert!(construction
        .iter()
        .all(|document| !d3_is_held_out(&document.id)));
    assert!(d3_is_held_out(&held_out.id));

    let table = SourceFreeTable::compile(&construction).unwrap();
    let overlay = MultiscaleCountRadiusR4V1::compile(&table).unwrap();
    let context = table.encode_text(b"At dusk the red fox").unwrap();
    let prediction = table
        .predict_multiscale_count_radius(&context, &overlay)
        .unwrap();

    assert_eq!(prediction.order, BackoffOrder::Trigram);
    assert_eq!(prediction.max_count, 1);
    assert!(prediction.geometry_reachable);
    assert_eq!(
        prediction
            .baseline_support_tokens
            .iter()
            .map(|token| decoded(&table, *token))
            .collect::<Vec<_>>(),
        vec![b" rests".to_vec(), b" runs".to_vec()]
    );
    assert_eq!(
        prediction.baseline_support_tokens,
        prediction.geometric_support_tokens
    );
    assert_eq!(
        prediction
            .max_count_tie_tokens
            .iter()
            .map(|token| decoded(&table, *token))
            .collect::<Vec<_>>(),
        vec![b" rests".to_vec(), b" runs".to_vec()]
    );
    assert_eq!(decoded(&table, prediction.baseline_token), b" rests");
    assert_eq!(decoded(&table, prediction.geometric_token), b" runs");
    assert_eq!(prediction.baseline_work, prediction.geometric_work);

    let rests_radius = prediction
        .tie_evidence
        .iter()
        .find(|candidate| decoded(&table, candidate.token) == b" rests")
        .unwrap()
        .radius;
    let runs_radius = prediction
        .tie_evidence
        .iter()
        .find(|candidate| decoded(&table, candidate.token) == b" runs")
        .unwrap()
        .radius;
    assert!(runs_radius > rests_radius);

    let evaluation = table
        .evaluate_held_out_multiscale_count_radius(&overlay, &[held_out])
        .unwrap();
    assert_eq!(evaluation.support_mismatches, 0);
    assert_eq!(evaluation.work_mismatches, 0);
    assert!(evaluation.reachable_tie_positions > 0);
    assert!(evaluation.changed_choices > 0);
    assert!(evaluation.geometric_correct > evaluation.baseline_correct);
    assert!(evaluation.geometric_changed_correct > evaluation.baseline_changed_correct);
    assert_eq!(evaluation.teacher_calls, 0);
    assert_eq!(evaluation.provider_calls, 0);
    assert_eq!(evaluation.source_weight_reads, 0);
}

#[test]
fn matched_continuation_records_only_the_first_shared_frame() {
    let table = SourceFreeTable::compile(&construction_documents()).unwrap();
    let overlay = MultiscaleCountRadiusR4V1::compile(&table).unwrap();
    let continuation = table
        .continue_text_multiscale_count_radius(&overlay, b"At dusk the red fox", 8)
        .unwrap();

    assert_eq!(continuation.baseline.decoded, b" rests.");
    assert_eq!(continuation.geometric.decoded, b" runs.");
    assert_eq!(continuation.baseline.stop, ContinuationStop::EndOfDocument);
    assert_eq!(continuation.geometric.stop, ContinuationStop::EndOfDocument);
    let divergence = continuation.first_divergence.unwrap();
    assert_eq!(divergence.unit_index, 0);
    assert!(divergence.support_matched);
    assert!(divergence.work_matched);
    assert_eq!(decoded(&table, divergence.baseline_token), b" rests");
    assert_eq!(decoded(&table, divergence.geometric_token), b" runs");
}

#[test]
fn overlay_reloads_canonically_and_binds_unchanged_sftbl001_bytes() {
    let table = SourceFreeTable::compile(&construction_documents()).unwrap();
    let table_bytes = table.to_bytes();
    let table_cid = table.artifact_cid();
    let overlay = MultiscaleCountRadiusR4V1::compile(&table).unwrap();
    let overlay_bytes = overlay.to_bytes();

    assert_eq!(table.to_bytes(), table_bytes);
    assert_eq!(table.artifact_cid(), table_cid);
    assert_eq!(overlay.table_artifact_cid(), table_cid);
    assert_eq!(
        MultiscaleCountRadiusR4V1::compile(&table)
            .unwrap()
            .to_bytes(),
        overlay_bytes
    );
    let reloaded = MultiscaleCountRadiusR4V1::from_bytes(&table, &overlay_bytes).unwrap();
    assert_eq!(reloaded.to_bytes(), overlay_bytes);
    assert_eq!(reloaded.artifact_cid(), overlay.artifact_cid());
    assert!(reloaded.stats().eligible_trigram_rows > 0);
    assert!(reloaded.stats().geometry_changed_rows > 0);

    let reloaded_table = SourceFreeTable::from_bytes(&table_bytes).unwrap();
    let reloaded_overlay =
        MultiscaleCountRadiusR4V1::from_bytes(&reloaded_table, &overlay_bytes).unwrap();
    assert_eq!(reloaded_overlay.to_bytes(), overlay_bytes);

    let other_table = SourceFreeTable::compile(&[
        SourceDocument::new("14", b"A different construction sentence.".to_vec()),
        SourceDocument::new("657", b"Another different sentence.".to_vec()),
    ])
    .unwrap();
    let context = other_table.encode_text(b"Another different").unwrap();
    assert!(other_table
        .predict_multiscale_count_radius(&context, &overlay)
        .unwrap_err()
        .to_string()
        .contains("binding mismatches"));

    let mut tampered = overlay_bytes;
    let final_index = tampered.len() - 1;
    tampered[final_index] ^= 1;
    assert!(MultiscaleCountRadiusR4V1::from_bytes(&table, &tampered).is_err());
}
