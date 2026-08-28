//! Frozen Gate 0 probe for #973 `PriorSentenceCountRadiusR4V1`.
//!
//! The English text is synthetic. D3 hashes enforce construction/held-out
//! partition separation for this bounded causal probe; they do not make the
//! fixture corpus-derived or establish natural-distribution transfer.

use serde::Serialize;
use uor_r4_core::higher_scope_geometric_attention::{
    MatchedPriorSentenceCountRadiusContinuation, MatchedPriorSentenceCountRadiusPrediction,
    PriorSentenceCountRadiusAbstention, PriorSentenceCountRadiusR4V1,
};
use uor_r4_core::source_free_table::{
    d3_is_held_out, BackoffOrder, ContinuationStop, MultiscaleCountRadiusR4V1, SourceDocument,
    SourceFreeTable, BOS_TOKEN,
};

const TEA_PROMPT: &[u8] = b"Mara chose tea before lunch. When the server arrived, Mara asked for";
const COFFEE_PROMPT: &[u8] =
    b"Mara chose coffee before lunch. When the server arrived, Mara asked for";

fn construction_documents() -> Vec<SourceDocument> {
    vec![
        SourceDocument::new(
            "14",
            b"Nora chose tea before breakfast. Later Nora asked for tea.".to_vec(),
        ),
        SourceDocument::new(
            "657",
            b"Owen chose coffee before breakfast. Later Owen asked for coffee.".to_vec(),
        ),
    ]
}

fn context(table: &SourceFreeTable, prompt: &[u8]) -> Vec<u32> {
    let mut context = vec![BOS_TOKEN];
    context.extend(table.encode_text(prompt).unwrap());
    context
}

fn decoded(table: &SourceFreeTable, token: u32) -> Vec<u8> {
    table.decode_tokens(&[token]).unwrap()
}

fn decoded_tokens(table: &SourceFreeTable, tokens: &[u32]) -> Vec<Vec<u8>> {
    tokens.iter().map(|token| decoded(table, *token)).collect()
}

#[derive(Serialize)]
struct LabelBlindCensus<'a> {
    schema: u32,
    domain: &'static str,
    table_cid: String,
    base_overlay_cid: String,
    operator_cid: String,
    cases: Vec<LabelBlindCase<'a>>,
    teacher_calls: u64,
    provider_calls: u64,
    source_weight_reads: u64,
    future_unit_reads: u64,
}

#[derive(Serialize)]
struct LabelBlindCase<'a> {
    partition_id: &'static str,
    prompt_cid: String,
    prediction: &'a MatchedPriorSentenceCountRadiusPrediction,
}

#[derive(Serialize)]
struct DecodedSmoke<'a> {
    schema: u32,
    domain: &'static str,
    operator_cid: String,
    cases: Vec<DecodedCase<'a>>,
    real_correct: u32,
    disabled_correct: u32,
    permuted_correct: u32,
    support_mismatches: u32,
    work_mismatches: u32,
    terminal: &'static str,
}

#[derive(Serialize)]
struct DecodedCase<'a> {
    partition_id: &'static str,
    target_hex: String,
    continuation: &'a MatchedPriorSentenceCountRadiusContinuation,
}

#[test]
fn label_blind_gate0_then_decoded_controls_make_prior_scope_load_bearing() {
    let construction = construction_documents();
    assert!(construction
        .iter()
        .all(|document| !d3_is_held_out(&document.id)));
    assert!(d3_is_held_out("12"));
    assert!(d3_is_held_out("13"));

    let prompt_documents = [
        SourceDocument::new("12", TEA_PROMPT.to_vec()),
        SourceDocument::new("13", COFFEE_PROMPT.to_vec()),
    ];
    for prompt in &prompt_documents {
        assert!(construction
            .iter()
            .all(|document| document.id != prompt.id && document.text_cid() != prompt.text_cid()));
    }

    let table = SourceFreeTable::compile(&construction).unwrap();
    let base_overlay = MultiscaleCountRadiusR4V1::compile(&table).unwrap();
    let operator = PriorSentenceCountRadiusR4V1::compile(&table, &base_overlay).unwrap();
    let operator_bytes = operator.to_bytes();
    let operator_cid = operator.artifact_cid();
    let reloaded =
        PriorSentenceCountRadiusR4V1::from_bytes(&table, &base_overlay, &operator_bytes).unwrap();
    assert_eq!(reloaded.to_bytes(), operator_bytes);
    assert_eq!(reloaded.artifact_cid(), operator_cid);
    assert_eq!(reloaded.table_artifact_cid(), table.artifact_cid());
    assert_eq!(
        reloaded.base_overlay_artifact_cid(),
        base_overlay.artifact_cid()
    );

    // Gate 0 sees prompts only. No continuation target is constructed above
    // this point.
    let tea_context = context(&table, TEA_PROMPT);
    let coffee_context = context(&table, COFFEE_PROMPT);
    assert_eq!(
        &tea_context[tea_context.len() - 2..],
        &coffee_context[coffee_context.len() - 2..]
    );
    assert_eq!(
        decoded_tokens(&table, &tea_context[tea_context.len() - 2..]),
        vec![b" asked".to_vec(), b" for".to_vec()]
    );

    let tea = reloaded
        .predict_matched(&table, &base_overlay, &tea_context)
        .unwrap();
    let coffee = reloaded
        .predict_matched(&table, &base_overlay, &coffee_context)
        .unwrap();
    for prediction in [&tea, &coffee] {
        assert_eq!(prediction.local.order, BackoffOrder::Trigram);
        assert_eq!(prediction.local.max_count, 1);
        assert!(prediction.local.geometry_reachable);
        assert_eq!(
            decoded_tokens(&table, &prediction.local.baseline_support_tokens),
            vec![b" coffee".to_vec(), b" tea".to_vec()]
        );
        assert_eq!(
            prediction.local.baseline_support_tokens,
            prediction.local.geometric_support_tokens
        );
        assert_eq!(
            prediction.local.max_count_tie_tokens,
            prediction.local.baseline_support_tokens
        );
        assert_eq!(
            prediction.local.baseline_work,
            prediction.local.geometric_work
        );
        assert_eq!(decoded(&table, prediction.local.baseline_token), b" coffee");
        assert_eq!(
            decoded(&table, prediction.local.geometric_token),
            b" coffee"
        );
        assert_eq!(prediction.prior_prefix_units, 13);
        assert_eq!(prediction.prior_candidate_occurrences, 1);
        assert_eq!(prediction.real.work.candidate_membership_checks, 26);
        assert!(prediction.sentence_boundary_index.is_some());
        assert!(prediction.support_matched);
        assert!(prediction.work_matched);
        assert_eq!(prediction.operator_abstention, None);
        assert_eq!(prediction.teacher_calls, 0);
        assert_eq!(prediction.provider_calls, 0);
        assert_eq!(prediction.source_weight_reads, 0);
        assert_eq!(prediction.future_unit_reads, 0);
        assert_eq!(prediction.candidate_evidence.len(), 2);
        for candidate in &prediction.candidate_evidence {
            assert_eq!(candidate.count, 1);
            assert_eq!(candidate.local_coordinates.trigram_q32, 1_u64 << 31);
            assert_eq!(candidate.local_coordinates.bigram_q32, 1_u64 << 31);
            assert_eq!(candidate.local_coordinates.unigram_q32, 330_382_099);
            assert_eq!(candidate.local_coordinates.depth_q32, 3_u64 << 30);
            assert_eq!(candidate.local_radius, 19_708_817_909_656_044_393);
            assert_eq!(candidate.disabled_prior_q32, 0);
            assert_eq!(candidate.disabled_radius, 9_332_524_368_194_421_609);
        }
    }

    assert_eq!(decoded(&table, tea.real.token), b" tea");
    assert_eq!(decoded(&table, coffee.real.token), b" coffee");
    assert_eq!(decoded(&table, tea.scope_disabled.token), b" coffee");
    assert_eq!(decoded(&table, coffee.scope_disabled.token), b" coffee");
    assert_eq!(decoded(&table, tea.candidate_permuted.token), b" coffee");
    assert_eq!(decoded(&table, coffee.candidate_permuted.token), b" tea");
    assert_ne!(tea.real.token, coffee.real.token);
    assert_eq!(tea.scope_disabled.token, coffee.scope_disabled.token);

    for prediction in [&tea, &coffee] {
        let present = prediction
            .candidate_evidence
            .iter()
            .find(|candidate| candidate.prior_count == 1)
            .unwrap();
        let absent = prediction
            .candidate_evidence
            .iter()
            .find(|candidate| candidate.prior_count == 0)
            .unwrap();
        assert_eq!(present.real_prior_q32, 1_u64 << 32);
        assert_eq!(present.real_radius, 27_779_268_441_903_973_225);
        assert_eq!(absent.real_prior_q32, 0);
        assert_eq!(absent.real_radius, 9_332_524_368_194_421_609);
    }

    // Candidate-relative transformed classes are frozen before labels. The
    // same class may not demand incompatible actions.
    let tea_class = tea
        .candidate_evidence
        .iter()
        .map(|candidate| {
            (
                candidate.token,
                candidate.prior_count,
                candidate.real_radius,
            )
        })
        .collect::<Vec<_>>();
    let coffee_class = coffee
        .candidate_evidence
        .iter()
        .map(|candidate| {
            (
                candidate.token,
                candidate.prior_count,
                candidate.real_radius,
            )
        })
        .collect::<Vec<_>>();
    assert_ne!(tea_class, coffee_class);
    let mut class_actions = std::collections::BTreeMap::new();
    assert_eq!(class_actions.insert(tea_class, tea.real.token), None);
    assert_eq!(class_actions.insert(coffee_class, coffee.real.token), None);

    let label_blind_census = LabelBlindCensus {
        schema: 1,
        domain: "uor-r4.prior-sentence-count-radius-gate0-census/1",
        table_cid: table.artifact_cid(),
        base_overlay_cid: base_overlay.artifact_cid(),
        operator_cid: operator_cid.clone(),
        cases: vec![
            LabelBlindCase {
                partition_id: "12",
                prompt_cid: format!("blake3:{}", blake3::hash(TEA_PROMPT).to_hex()),
                prediction: &tea,
            },
            LabelBlindCase {
                partition_id: "13",
                prompt_cid: format!("blake3:{}", blake3::hash(COFFEE_PROMPT).to_hex()),
                prediction: &coffee,
            },
        ],
        teacher_calls: 0,
        provider_calls: 0,
        source_weight_reads: 0,
        future_unit_reads: 0,
    };
    let frozen_census_bytes = serde_json::to_vec(&label_blind_census).unwrap();
    let frozen_census_cid = format!("blake3:{}", blake3::hash(&frozen_census_bytes).to_hex());
    assert_eq!(
        serde_json::to_vec(&label_blind_census).unwrap(),
        frozen_census_bytes
    );

    // The sealed continuations are attached only after the target-free census
    // and bound operator bytes have been frozen.
    let sealed = [
        ("12", TEA_PROMPT, b" tea.".as_slice()),
        ("13", COFFEE_PROMPT, b" coffee.".as_slice()),
    ];
    let tea_continuation = reloaded
        .continue_matched(&table, &base_overlay, sealed[0].1, 3)
        .unwrap();
    let coffee_continuation = reloaded
        .continue_matched(&table, &base_overlay, sealed[1].1, 3)
        .unwrap();
    let continuations = [&tea_continuation, &coffee_continuation];
    let mut real_correct = 0_u32;
    let mut disabled_correct = 0_u32;
    let mut permuted_correct = 0_u32;
    for (index, continuation) in continuations.iter().enumerate() {
        let target = sealed[index].2;
        real_correct += u32::from(continuation.real.decoded == target);
        disabled_correct += u32::from(continuation.scope_disabled.decoded == target);
        permuted_correct += u32::from(continuation.candidate_permuted.decoded == target);
        assert_eq!(continuation.real.stop, ContinuationStop::EndOfDocument);
        assert_eq!(
            continuation.scope_disabled.stop,
            ContinuationStop::EndOfDocument
        );
        assert_eq!(
            continuation.candidate_permuted.stop,
            ContinuationStop::EndOfDocument
        );
        assert_eq!(continuation.real.tokens.len(), 2);
        assert_eq!(continuation.scope_disabled.tokens.len(), 2);
        assert_eq!(continuation.candidate_permuted.tokens.len(), 2);
        assert!(continuation.first_decision.support_matched);
        assert!(continuation.first_decision.work_matched);
    }
    assert_eq!(tea_continuation.real.decoded, b" tea.");
    assert_eq!(coffee_continuation.real.decoded, b" coffee.");
    assert_eq!(tea_continuation.scope_disabled.decoded, b" coffee.");
    assert_eq!(coffee_continuation.scope_disabled.decoded, b" coffee.");
    assert_eq!(tea_continuation.candidate_permuted.decoded, b" coffee.");
    assert_eq!(coffee_continuation.candidate_permuted.decoded, b" tea.");
    assert_eq!(real_correct, 2);
    assert_eq!(disabled_correct, 1);
    assert_eq!(permuted_correct, 0);

    let decoded_smoke = DecodedSmoke {
        schema: 1,
        domain: "uor-r4.prior-sentence-count-radius-decoded-smoke/1",
        operator_cid: operator_cid.clone(),
        cases: vec![
            DecodedCase {
                partition_id: sealed[0].0,
                target_hex: hex::encode(sealed[0].2),
                continuation: &tea_continuation,
            },
            DecodedCase {
                partition_id: sealed[1].0,
                target_hex: hex::encode(sealed[1].2),
                continuation: &coffee_continuation,
            },
        ],
        real_correct,
        disabled_correct,
        permuted_correct,
        support_mismatches: 0,
        work_mismatches: 0,
        terminal: "RETAIN_GATE0_PRIOR_SENTENCE_ATTENTION_CONTINUE_PARAGRAPH_CONVERSATION",
    };
    let smoke_bytes = serde_json::to_vec(&decoded_smoke).unwrap();
    let smoke_cid = format!("blake3:{}", blake3::hash(&smoke_bytes).to_hex());

    let tea_replay = reloaded
        .continue_matched(&table, &base_overlay, sealed[0].1, 3)
        .unwrap();
    let coffee_replay = reloaded
        .continue_matched(&table, &base_overlay, sealed[1].1, 3)
        .unwrap();
    assert_eq!(tea_replay, tea_continuation);
    assert_eq!(coffee_replay, coffee_continuation);
    let replay_smoke = DecodedSmoke {
        schema: 1,
        domain: "uor-r4.prior-sentence-count-radius-decoded-smoke/1",
        operator_cid: operator_cid.clone(),
        cases: vec![
            DecodedCase {
                partition_id: sealed[0].0,
                target_hex: hex::encode(sealed[0].2),
                continuation: &tea_replay,
            },
            DecodedCase {
                partition_id: sealed[1].0,
                target_hex: hex::encode(sealed[1].2),
                continuation: &coffee_replay,
            },
        ],
        real_correct,
        disabled_correct,
        permuted_correct,
        support_mismatches: 0,
        work_mismatches: 0,
        terminal: "RETAIN_GATE0_PRIOR_SENTENCE_ATTENTION_CONTINUE_PARAGRAPH_CONVERSATION",
    };
    assert_eq!(serde_json::to_vec(&replay_smoke).unwrap(), smoke_bytes);
    assert_eq!(reloaded.to_bytes(), operator_bytes);

    println!(
        "table_cid={}\nbase_overlay_cid={}\noperator_bytes={}\noperator_cid={operator_cid}\nlabel_blind_census_cid={frozen_census_cid}\ndecoded_smoke_cid={smoke_cid}\nreal=2/2\nscope_disabled=1/2\ncandidate_permuted=0/2\nsupport_mismatches=0\nwork_mismatches=0\nterminal=RETAIN_GATE0_PRIOR_SENTENCE_ATTENTION_CONTINUE_PARAGRAPH_CONVERSATION",
        table.artifact_cid(),
        base_overlay.artifact_cid(),
        operator_bytes.len(),
    );
}

#[test]
fn operator_rejects_binding_drift_tamper_and_excess_scope() {
    let table = SourceFreeTable::compile(&construction_documents()).unwrap();
    let base_overlay = MultiscaleCountRadiusR4V1::compile(&table).unwrap();
    let operator = PriorSentenceCountRadiusR4V1::compile(&table, &base_overlay).unwrap();
    let bytes = operator.to_bytes();

    let other_table = SourceFreeTable::compile(&[
        SourceDocument::new("14", b"A separate alpha construction.".to_vec()),
        SourceDocument::new("657", b"A separate beta construction.".to_vec()),
    ])
    .unwrap();
    let other_overlay = MultiscaleCountRadiusR4V1::compile(&other_table).unwrap();
    assert!(
        PriorSentenceCountRadiusR4V1::from_bytes(&other_table, &other_overlay, &bytes).is_err()
    );
    assert!(PriorSentenceCountRadiusR4V1::from_bytes(&table, &other_overlay, &bytes).is_err());

    let mut tampered = bytes;
    let final_index = tampered.len() - 1;
    tampered[final_index] ^= 1;
    assert!(PriorSentenceCountRadiusR4V1::from_bytes(&table, &base_overlay, &tampered).is_err());

    let mut excessive = b" tea".repeat(65);
    excessive.extend_from_slice(b". When the server arrived, Mara asked for");
    let excessive_context = context(&table, &excessive);
    let error = operator
        .predict_matched(&table, &base_overlay, &excessive_context)
        .unwrap_err();
    assert!(error.to_string().contains("64-unit operator bound"));

    let mut excessive_ineligible = b" tea".repeat(65);
    excessive_ineligible.extend_from_slice(b". Later Nora asked");
    let excessive_ineligible_context = context(&table, &excessive_ineligible);
    let error = operator
        .predict_matched(&table, &base_overlay, &excessive_ineligible_context)
        .unwrap_err();
    assert!(error.to_string().contains("64-unit operator bound"));

    let missing_boundary = context(&table, b"Mara asked for");
    let missing = operator
        .predict_matched(&table, &base_overlay, &missing_boundary)
        .unwrap();
    assert_eq!(
        missing.operator_abstention,
        Some(PriorSentenceCountRadiusAbstention::MissingSentenceBoundary)
    );
    assert_eq!(missing.real.token, missing.local.geometric_token);
    assert_eq!(missing.real.unique_radius_winner, None);

    let zero_signal = context(
        &table,
        b"Mara chose cocoa before lunch. When the server arrived, Mara asked for",
    );
    let zero = operator
        .predict_matched(&table, &base_overlay, &zero_signal)
        .unwrap();
    assert_eq!(
        zero.operator_abstention,
        Some(PriorSentenceCountRadiusAbstention::NoPriorCandidateOccurrence)
    );
    assert_eq!(zero.real.token, zero.local.geometric_token);
    assert_eq!(zero.real.unique_radius_winner, None);
    assert!(zero.work_matched);
}

#[test]
fn decision_support_is_exactly_the_max_count_tie_not_the_full_row() {
    let wide_construction = vec![
        SourceDocument::new(
            "14",
            b"Nora asked for tea. Later Nora asked for tea.".to_vec(),
        ),
        SourceDocument::new(
            "657",
            b"Owen asked for coffee. Later Owen asked for coffee.".to_vec(),
        ),
        SourceDocument::new("4579", b"Iris asked for cocoa.".to_vec()),
    ];
    assert!(wide_construction
        .iter()
        .all(|document| !d3_is_held_out(&document.id)));
    let wide_table = SourceFreeTable::compile(&wide_construction).unwrap();
    let wide_overlay = MultiscaleCountRadiusR4V1::compile(&wide_table).unwrap();
    let wide_operator = PriorSentenceCountRadiusR4V1::compile(&wide_table, &wide_overlay).unwrap();
    let wide_context = context(
        &wide_table,
        b"Mara chose tea before lunch. When the server arrived, Mara asked for",
    );
    let prediction = wide_operator
        .predict_matched(&wide_table, &wide_overlay, &wide_context)
        .unwrap();

    assert_eq!(
        decoded_tokens(&wide_table, &prediction.local.baseline_support_tokens),
        vec![b" cocoa".to_vec(), b" coffee".to_vec(), b" tea".to_vec()]
    );
    assert_eq!(
        decoded_tokens(&wide_table, &prediction.local.max_count_tie_tokens),
        vec![b" coffee".to_vec(), b" tea".to_vec()]
    );
    assert_eq!(
        prediction.real.support_tokens,
        prediction.local.max_count_tie_tokens
    );
    assert_eq!(
        prediction.scope_disabled.support_tokens,
        prediction.local.max_count_tie_tokens
    );
    assert_eq!(
        prediction.candidate_permuted.support_tokens,
        prediction.local.max_count_tie_tokens
    );
    assert!(prediction.support_matched);
}
