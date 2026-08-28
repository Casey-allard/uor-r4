//! Executable research contract for #973 corpus-induced document spin placement.
//!
//! The ordinary test is a small natural-language fixture that exercises the
//! immutable placement artifact, construction-only anti-recall index, target-
//! free census, four matched arms, and post-join report shape. The ignored D3
//! runner is the decision-bearing experiment. It validates the frozen corpus
//! identities and persists the target-free report before it is allowed to
//! attach a held-out next-route target.

use std::collections::BTreeSet;
use std::env;
use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use uor_r4_core::corpus_induced_spin_placement::{
    CorpusInducedDocumentSpinAntiRecallIndex, CorpusInducedDocumentSpinArm,
    CorpusInducedDocumentSpinArtifactStats, CorpusInducedDocumentSpinComparatorEvaluation,
    CorpusInducedDocumentSpinEvaluation, CorpusInducedDocumentSpinForbiddenReads,
    CorpusInducedDocumentSpinPlacementR4V1, CorpusInducedDocumentSpinTargetFreeCensus,
    MIN_OPERATIVE_ANTI_RECALL_POSITIONS, NEGATIVE_TERMINAL, POSITIVE_TERMINAL,
    UNAVAILABLE_TERMINAL,
};
use uor_r4_core::source_free_table::{
    d3_is_held_out, MultiscaleCountRadiusR4V1, SourceDocument, SourceFreeTable, BOS_TOKEN,
    EOS_TOKEN,
};

const CORPUS_PATH_ENV: &str = "UOR_R4_973_D3_CORPUS";
const OUTPUT_DIR_ENV: &str = "UOR_R4_973_OUTPUT_DIR";
const DEFAULT_CORPUS_PATH: &str =
    "/Users/casey.allard/uor-r4/.uor-models/corpora/simple-wiki-20231101/articles.jsonl";
const DEFAULT_OUTPUT_DIR: &str =
    "/Users/casey.allard/uor-r4/.uor-models/research/issue-973-corpus-spin-v1";

const FROZEN_CORPUS_CID: &str =
    "blake3:194db0eebf2d49823ece01ee935447a0cc9edeaf018454ceea480ce7590132cf";
const FROZEN_MANIFEST_CID: &str =
    "blake3:bb5f446ce92df60f7824ed5a1f04ede385386e7a47b9c198ae83a5d0f907bab3";
const FROZEN_TABLE_CID: &str =
    "blake3:ccdc399731cb866a329be478467a434cda4e445813421e5d17c21ccc87288297";
const FROZEN_OVERLAY_CID: &str =
    "blake3:914126a311c3984d1482258a8f0a7fa2e34896540d502d19f1d9076fbd4a9b76";
const FROZEN_CONSTRUCTION_SET_KAPPA: &str =
    "blake3:af2a2d7d49db55279e7ea40947a3259ac0a100aa56e8d920951e7c27eaf6df5c";
const FROZEN_HELD_OUT_SET_KAPPA: &str =
    "blake3:7a7558e96aa86aa2d8965972b69ddce02222c6eccc8ca560df2141fc0ac4170e";
const FROZEN_DOCUMENTS: usize = 3_000;
const FROZEN_CONSTRUCTION_DOCUMENTS: usize = 2_404;
const FROZEN_HELD_OUT_DOCUMENTS: usize = 596;
const FROZEN_TARGET_FREE_ADMISSIONS: u64 = 81_177;
const FROZEN_KNOWN_TARGET_ADMISSIONS: u64 = 76_641;

const TABLE_ARTIFACT_FILE: &str = "source_free_table.sftbl001";
const OVERLAY_ARTIFACT_FILE: &str = "multiscale_count_radius.sftr4o01";
const OPERATOR_ARTIFACT_FILE: &str = "corpus_induced_document_spin.cidsp001";
const TARGET_FREE_REPORT_FILE: &str = "target_free_preflight.json";
const EVALUATION_REPORT_FILE: &str = "evaluation_report.json";
const CANONICAL_RUN_FILE: &str = "canonical_run.json";

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

static TEMPORARY_REPORT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Deserialize)]
struct D3Manifest {
    schema: u32,
    article_count: usize,
    corpus_cid: String,
}

#[derive(Debug, Deserialize)]
struct D3Article {
    id: String,
    text: String,
}

#[derive(Debug, Serialize)]
struct DecodedDecision {
    token: u32,
    decoded_utf8: String,
}

#[derive(Debug, Serialize)]
struct DecodedWitness {
    document_id: String,
    target_index: u32,
    real: DecodedDecision,
    scope_disabled: DecodedDecision,
    order_shuffled: DecodedDecision,
    operator_permuted: DecodedDecision,
}

#[derive(Serialize, Clone, Copy)]
struct ReportIdentity<'a> {
    manifest_cid: &'a str,
    corpus_cid: &'a str,
    table_cid: &'a str,
    overlay_cid: &'a str,
}

const FROZEN_REPORT_IDENTITY: ReportIdentity<'static> = ReportIdentity {
    manifest_cid: FROZEN_MANIFEST_CID,
    corpus_cid: FROZEN_CORPUS_CID,
    table_cid: FROZEN_TABLE_CID,
    overlay_cid: FROZEN_OVERLAY_CID,
};

struct TemporaryReportDirectory {
    path: PathBuf,
}

impl TemporaryReportDirectory {
    fn create() -> io::Result<Self> {
        loop {
            let sequence = TEMPORARY_REPORT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let path = env::temp_dir().join(format!(
                "uor-r4-973-unavailable-report-{}-{sequence}",
                std::process::id()
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TemporaryReportDirectory {
    fn drop(&mut self) {
        for file in [
            TARGET_FREE_REPORT_FILE,
            EVALUATION_REPORT_FILE,
            CANONICAL_RUN_FILE,
        ] {
            let _ = fs::remove_file(self.path.join(file));
        }
        let _ = fs::remove_dir(&self.path);
    }
}

#[derive(Serialize)]
struct TargetFreePreflightReport<'a> {
    schema: u32,
    domain: &'static str,
    manifest_cid: &'a str,
    corpus_cid: &'a str,
    table_cid: &'a str,
    overlay_cid: &'a str,
    operator_cid: String,
    operator_stats: CorpusInducedDocumentSpinArtifactStats,
    anti_recall_index_kappa: &'a str,
    anti_recall_full_prefixes: usize,
    anti_recall_natural_states: usize,
    anti_recall_operative_signatures: usize,
    census_cid: String,
    decoded_witness: Option<&'a DecodedWitness>,
    census: &'a CorpusInducedDocumentSpinTargetFreeCensus,
}

#[derive(Serialize)]
struct CanonicalD3RunReport<'a> {
    schema: u32,
    domain: &'static str,
    manifest_cid: &'a str,
    corpus_cid: &'a str,
    table_cid: &'a str,
    overlay_cid: &'a str,
    operator_cid: String,
    census_cid: String,
    evaluation_report_cid: String,
    decoded_witness: Option<&'a DecodedWitness>,
    evaluation: &'a CorpusInducedDocumentSpinEvaluation,
}

fn natural_construction_fixture() -> Vec<SourceDocument> {
    vec![
        SourceDocument::new("14", b"The red fox rests.".to_vec()),
        SourceDocument::new("657", b"At noon the red fox rests.".to_vec()),
        SourceDocument::new("4579", b"The red fox runs.".to_vec()),
        SourceDocument::new("5121", b"At dusk the red fox runs.".to_vec()),
    ]
}

fn natural_held_out_fixture() -> Vec<SourceDocument> {
    vec![
        SourceDocument::new("12", b"Tonight the red fox rests.".to_vec()),
        SourceDocument::new("13", b"Tomorrow the red fox runs.".to_vec()),
    ]
}

fn cid(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

fn invalid_data(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

fn require(condition: bool, message: impl Into<String>) -> io::Result<()> {
    if condition {
        Ok(())
    } else {
        Err(invalid_data(message))
    }
}

fn report_stage(stage: &str, started: Instant) {
    eprintln!(
        "#973 stage={stage} elapsed={:.3}s",
        started.elapsed().as_secs_f64()
    );
}

fn decision_bytes(table: &SourceFreeTable, token: u32) -> TestResult<String> {
    Ok(String::from_utf8(table.decode_tokens(&[token])?)?)
}

fn decode_witness(
    table: &SourceFreeTable,
    census: &CorpusInducedDocumentSpinTargetFreeCensus,
) -> TestResult<DecodedWitness> {
    let witness = census
        .frozen_decoded_witness
        .as_ref()
        .ok_or_else(|| invalid_data("target-free census has no frozen decoded witness"))?;
    Ok(DecodedWitness {
        document_id: witness.document_id.clone(),
        target_index: witness.target_index,
        real: DecodedDecision {
            token: witness.real_token,
            decoded_utf8: decision_bytes(table, witness.real_token)?,
        },
        scope_disabled: DecodedDecision {
            token: witness.scope_disabled_token,
            decoded_utf8: decision_bytes(table, witness.scope_disabled_token)?,
        },
        order_shuffled: DecodedDecision {
            token: witness.order_shuffled_token,
            decoded_utf8: decision_bytes(table, witness.order_shuffled_token)?,
        },
        operator_permuted: DecodedDecision {
            token: witness.operator_permuted_token,
            decoded_utf8: decision_bytes(table, witness.operator_permuted_token)?,
        },
    })
}

fn assert_comparator_report(
    comparator: &CorpusInducedDocumentSpinComparatorEvaluation,
    expected_pairs: Option<u64>,
) {
    assert_eq!(
        comparator.wins + comparator.losses,
        comparator.discordant,
        "discordant count must be the paired win/loss denominator"
    );
    if let Some(expected_pairs) = expected_pairs {
        assert_eq!(
            comparator.wins + comparator.losses + comparator.ties,
            expected_pairs,
            "every declared pair must enter the exact comparator report"
        );
    }
    assert!(
        comparator
            .one_sided_exact_sign_test
            .starts_with("20*sum_{k=0}^{"),
        "exact sign-test expression must be explicit: {comparator:?}"
    );
    let exact_threshold_passes = comparator.one_sided_exact_sign_test.ends_with(": true");
    assert!(
        exact_threshold_passes || comparator.one_sided_exact_sign_test.ends_with(": false"),
        "exact integer sign-test decision is missing: {comparator:?}"
    );
    assert_eq!(
        comparator.passes,
        comparator.wins > comparator.losses && exact_threshold_passes
    );
    assert_eq!(
        comparator.terminal,
        if comparator.passes {
            "PASS_DIRECTIONAL_EXACT_SIGN_TEST"
        } else {
            "FAIL_DIRECTIONAL_EXACT_SIGN_TEST"
        }
    );
}

fn assert_evaluation_report(evaluation: &CorpusInducedDocumentSpinEvaluation) {
    for comparator in [
        &evaluation.versus_scope_disabled,
        &evaluation.versus_order_shuffled,
        &evaluation.versus_operator_permuted,
    ] {
        assert_comparator_report(
            comparator,
            Some(evaluation.operative_known_target_positions),
        );
    }
    let document_blocked = [
        &evaluation.document_blocked_versus_scope_disabled,
        &evaluation.document_blocked_versus_order_shuffled,
        &evaluation.document_blocked_versus_operator_permuted,
    ];
    let document_pairs =
        document_blocked[0].wins + document_blocked[0].losses + document_blocked[0].ties;
    assert!(document_pairs <= evaluation.held_out_documents);
    for comparator in document_blocked {
        assert_comparator_report(comparator, Some(document_pairs));
    }
    assert_eq!(
        evaluation.witness_continuation.is_some(),
        evaluation.witness_continuation_cid.is_some()
    );
    if let (Some(continuation), Some(continuation_cid)) = (
        &evaluation.witness_continuation,
        &evaluation.witness_continuation_cid,
    ) {
        assert_eq!(
            continuation_cid,
            &cid(&serde_json::to_vec(continuation).unwrap())
        );
    }
    let all_six_comparators_pass = [
        &evaluation.versus_scope_disabled,
        &evaluation.versus_order_shuffled,
        &evaluation.versus_operator_permuted,
        &evaluation.document_blocked_versus_scope_disabled,
        &evaluation.document_blocked_versus_order_shuffled,
        &evaluation.document_blocked_versus_operator_permuted,
    ]
    .into_iter()
    .all(|comparator| comparator.passes);
    assert_eq!(
        evaluation.decision == POSITIVE_TERMINAL,
        all_six_comparators_pass && evaluation.witness_continuation_contrast
    );
    assert_eq!(evaluation.forbidden_reads.total(), 0);
}

fn validate_unavailable_evaluation(
    evaluation: &CorpusInducedDocumentSpinEvaluation,
    census: &CorpusInducedDocumentSpinTargetFreeCensus,
) -> TestResult {
    require(
        !census.meets_frozen_preflight,
        "UNAVAILABLE evaluation requires a valid failed target-free preflight",
    )?;
    require(
        census.support_mismatches == 0
            && census.work_mismatches == 0
            && census.invalid_score_firewall_certificates == 0
            && census.forbidden_reads.total() == 0,
        "contract-invalid target-free evidence cannot terminate as scientific UNAVAILABLE",
    )?;
    require(
        evaluation.decision == UNAVAILABLE_TERMINAL,
        format!(
            "failed preflight produced an invalid terminal: {}",
            evaluation.decision
        ),
    )?;
    require(
        evaluation.census_cid == census.artifact_cid()?,
        "UNAVAILABLE evaluation is not bound to the persisted target-free census",
    )?;
    require(
        evaluation.held_out_set_kappa == census.held_out_set_kappa,
        "UNAVAILABLE evaluation held-out binding drifted",
    )?;
    require(
        evaluation.held_out_documents == census.held_out_documents,
        "UNAVAILABLE evaluation held-out document count drifted",
    )?;
    require(
        evaluation.operative_positions == census.operative_positions.len() as u64,
        "UNAVAILABLE evaluation operative count drifted",
    )?;
    require(
        evaluation.post_join_admission_opportunities == 0
            && evaluation.post_join_known_target_admission_opportunities == 0
            && evaluation.operative_known_target_positions == 0
            && evaluation.real_correct == 0
            && evaluation.scope_disabled_correct == 0
            && evaluation.order_shuffled_correct == 0
            && evaluation.operator_permuted_correct == 0,
        "UNAVAILABLE evaluation attached label-bearing held-out results",
    )?;
    require(
        !evaluation.witness_continuation_contrast
            && evaluation.witness_continuation.is_none()
            && evaluation.witness_continuation_cid.is_none(),
        "UNAVAILABLE evaluation emitted a post-join continuation witness",
    )?;
    require(
        evaluation.target_reads == 0,
        "failed preflight attempted to read a held-out target",
    )?;
    require(
        evaluation.forbidden_reads.total() == 0,
        "UNAVAILABLE evaluation recorded a forbidden read",
    )?;
    assert_evaluation_report(evaluation);
    Ok(())
}

fn source_slice<'a>(source: &'a str, start: &str, end: &str) -> Result<&'a str, String> {
    let start_index = source
        .find(start)
        .ok_or_else(|| format!("score-firewall start marker is absent: {start}"))?;
    let end_index = source[start_index..]
        .find(end)
        .map(|offset| start_index + offset)
        .ok_or_else(|| format!("score-firewall end marker is absent: {end}"))?;
    Ok(&source[start_index..end_index])
}

fn invocation_slices<'a>(source: &'a str, name: &str) -> Result<Vec<&'a str>, String> {
    let mut invocations = Vec::new();
    let mut cursor = 0;
    while let Some(relative_start) = source[cursor..].find(name) {
        let start = cursor + relative_start;
        let open = source[start..]
            .find('(')
            .map(|offset| start + offset)
            .ok_or_else(|| {
                format!("score-firewall invocation has no opening parenthesis: {name}")
            })?;
        let mut depth = 0_u32;
        let mut close = None;
        for (offset, byte) in source.as_bytes()[open..].iter().enumerate() {
            match byte {
                b'(' => depth += 1,
                b')' => {
                    depth = depth
                        .checked_sub(1)
                        .ok_or_else(|| "score-firewall parenthesis underflow".to_owned())?;
                    if depth == 0 {
                        close = Some(open + offset + 1);
                        break;
                    }
                }
                _ => {}
            }
        }
        let close = close.ok_or_else(|| {
            format!("score-firewall invocation has no closing parenthesis: {name}")
        })?;
        invocations.push(&source[start..close]);
        cursor = close;
    }
    Ok(invocations)
}

fn source_without_comments_or_strings(source: &str) -> String {
    source
        .lines()
        .filter_map(|line| {
            let code = line.split_once("//").map_or(line, |(code, _)| code);
            let mut outside = true;
            let mut escaped = false;
            let mut scrubbed = String::new();
            for character in code.chars() {
                if outside {
                    if character == '"' {
                        outside = false;
                    } else {
                        scrubbed.push(character);
                    }
                } else if escaped {
                    escaped = false;
                } else if character == '\\' {
                    escaped = true;
                } else if character == '"' {
                    outside = true;
                }
            }
            (!scrubbed.trim().is_empty()).then_some(scrubbed)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn enforce_exact_score_firewall(
    placement_source: &str,
    exact_kernel_source: &str,
    forbidden_reads: CorpusInducedDocumentSpinForbiddenReads,
) -> Result<(), String> {
    if forbidden_reads.total() != 0 {
        return Err(format!(
            "score-firewall rejected {} injected forbidden reads",
            forbidden_reads.total()
        ));
    }

    let executed_score_kernel = source_slice(
        placement_source,
        "fn execute_arm(",
        "fn issue_score_firewall_certificate(",
    )?;
    let relative_calls = invocation_slices(executed_score_kernel, "candidate_relative_exact_cost")?;
    if relative_calls.len() != 1 {
        return Err(format!(
            "score-firewall expected one shared executed-arm relative-cost seam, found {}",
            relative_calls.len()
        ));
    }
    let decision_kernel =
        source_slice(placement_source, "fn select_decision(", "fn blake3_label(")?;
    let selection_calls = invocation_slices(decision_kernel, "select_unique_minimum_exact_costs")?;
    if selection_calls.len() != 1 {
        return Err(format!(
            "score-firewall expected one exact unique-minimum call, found {}",
            selection_calls.len()
        ));
    }
    let exact_cost_kernel = source_slice(
        exact_kernel_source,
        "pub(crate) fn candidate_relative_exact_cost(",
        "#[derive(Debug, Clone, Copy)]",
    )?;
    let exact_selection_kernel = source_slice(
        exact_kernel_source,
        "pub(crate) fn select_unique_minimum_exact_costs(",
        "fn select_exact_costs(",
    )?;
    let prediction_kernel = placement_source
        .split_once("    fn predict_from_frame(")
        .map(|(_, suffix)| suffix)
        .ok_or_else(|| "predict_from_frame production certificate seam is absent".to_owned())?;
    if invocation_slices(prediction_kernel, "issue_score_firewall_certificate")?.len() != 1
        || invocation_slices(prediction_kernel, "forbidden_reads_from_mask")?.len() != 1
    {
        return Err(
            "score-firewall production certificate must issue and consume exactly once".to_owned(),
        );
    }
    for required_production_certificate_seam in [
        "score_firewall_certificate: CorpusInducedDocumentSpinScoreFirewallCertificate",
        "ScoreFrameOrigin::CausalPrefix => 0",
        "ScoreFrameOrigin::InjectedHeldOutTarget => 1",
    ] {
        if !placement_source.contains(required_production_certificate_seam) {
            return Err(format!(
                "score-firewall lost production certificate seam `{required_production_certificate_seam}`"
            ));
        }
    }

    let mut guarded_source = String::new();
    for invocation in relative_calls.iter().chain(selection_calls.iter()) {
        guarded_source.push_str(invocation);
        guarded_source.push('\n');
    }
    guarded_source.push_str(exact_cost_kernel);
    guarded_source.push_str(exact_selection_kernel);
    let guarded_source = source_without_comments_or_strings(&guarded_source);
    for forbidden in [
        "target",
        "future",
        "teacher",
        "provider",
        "source_weight",
        "payload",
        "prime",
        "rank",
        "digest",
        "support",
        "provenance",
    ] {
        if guarded_source.contains(forbidden) {
            return Err(format!(
                "score-firewall rejected forbidden numeric input `{forbidden}`"
            ));
        }
    }
    for required_type_seam in [
        "class_state: ExactSpinState",
        "global_state: ExactSpinState",
        "relative: ExactSpinState",
        "costs: &[BoundedGlobalExactSpinCost]",
        "select_unique_minimum_exact_costs(costs)",
    ] {
        if !guarded_source.contains(required_type_seam) {
            return Err(format!(
                "score-firewall lost required exact type seam `{required_type_seam}`"
            ));
        }
    }
    Ok(())
}

#[test]
fn exact_score_source_and_counter_firewall_rejects_injected_leaks() {
    const PLACEMENT_SOURCE: &str = include_str!("../src/corpus_induced_spin_placement.rs");
    const EXACT_KERNEL_SOURCE: &str = include_str!("../src/bounded_global_exact_spin_attention.rs");

    enforce_exact_score_firewall(
        PLACEMENT_SOURCE,
        EXACT_KERNEL_SOURCE,
        CorpusInducedDocumentSpinForbiddenReads::default(),
    )
    .expect("the live exact score seam must accept only exact geometry and exact costs");

    let executed_score_kernel = source_slice(
        PLACEMENT_SOURCE,
        "fn execute_arm(",
        "fn issue_score_firewall_certificate(",
    )
    .expect("live executed-arm score kernel must be source-addressable");
    let relative_invocation =
        invocation_slices(executed_score_kernel, "candidate_relative_exact_cost")
            .expect("live relative-cost call must parse")
            .into_iter()
            .next()
            .expect("live relative-cost call must exist");
    let injected_invocation =
        relative_invocation.replacen("query.state", "held_out_target_derived_state", 1);
    let injected_source = PLACEMENT_SOURCE.replacen(relative_invocation, &injected_invocation, 1);
    assert_ne!(
        injected_source, PLACEMENT_SOURCE,
        "target-derived-state injection must reach the live executed-arm score seam"
    );
    let injected_source_error = enforce_exact_score_firewall(
        &injected_source,
        EXACT_KERNEL_SOURCE,
        CorpusInducedDocumentSpinForbiddenReads::default(),
    )
    .expect_err("an injected target score input must fail the source firewall");
    assert!(injected_source_error.contains("target"));

    let mut injected_counter = CorpusInducedDocumentSpinForbiddenReads::default();
    injected_counter.teacher_calls = 1;
    let injected_counter_error =
        enforce_exact_score_firewall(PLACEMENT_SOURCE, EXACT_KERNEL_SOURCE, injected_counter)
            .expect_err("an injected forbidden-read counter must fail the typed firewall");
    assert!(injected_counter_error.contains("injected forbidden reads"));
}

#[test]
fn natural_fixture_replays_artifact_and_exercises_four_target_join_arms() -> TestResult {
    let construction = natural_construction_fixture();
    let held_out = natural_held_out_fixture();
    assert!(
        construction
            .iter()
            .all(|document| !d3_is_held_out(&document.id)),
        "fixture construction IDs must remain in the D3 construction partition"
    );
    assert!(
        held_out.iter().all(|document| d3_is_held_out(&document.id)),
        "fixture held-out IDs must remain in the D3 held-out partition"
    );

    let table = SourceFreeTable::compile(&construction)?;
    assert!(table.is_bound_to_construction_documents(&construction));
    assert!(table.is_disjoint_d3_held_out_documents(&held_out));
    let held_stream = table.encode_document_stream(&held_out[0])?;
    assert_eq!(held_stream.first(), Some(&BOS_TOKEN));
    assert_eq!(held_stream.last(), Some(&EOS_TOKEN));
    assert!(!table.is_fitted_lexical_token(BOS_TOKEN));
    assert!(!table.is_fitted_lexical_token(EOS_TOKEN));
    assert!(!table.is_fitted_lexical_token(table.maximum_token_id() + 1));

    let overlay = MultiscaleCountRadiusR4V1::compile(&table)?;
    let compiled =
        CorpusInducedDocumentSpinPlacementR4V1::compile(&table, &overlay, &construction)?;
    let artifact_bytes = compiled.to_bytes()?;
    assert_eq!(
        CorpusInducedDocumentSpinPlacementR4V1::compile(&table, &overlay, &construction)?
            .to_bytes()?,
        artifact_bytes,
        "construction-only compilation must be byte reproducible"
    );
    let expected_operator_cid = compiled.artifact_cid()?;
    let operator = CorpusInducedDocumentSpinPlacementR4V1::from_bytes(
        &table,
        &overlay,
        &expected_operator_cid,
        &artifact_bytes,
    )?;
    assert_eq!(operator.to_bytes()?, artifact_bytes);
    assert_eq!(operator.artifact_cid()?, cid(&artifact_bytes));
    assert!(operator.stats().usable_prototypes >= 2);

    let mut prefix = vec![BOS_TOKEN];
    prefix.extend(table.encode_text(b"Tonight the red fox")?);
    let prediction = operator.predict_matched(&table, &overlay, &prefix)?;
    assert!(
        prediction.score_firewall_certificate.validate(),
        "production score-firewall certificate must self-validate"
    );
    assert_eq!(
        prediction.score_firewall_certificate.operator_cid,
        operator.artifact_cid()?
    );
    assert_eq!(
        prediction.score_firewall_certificate.causal_prefix_units,
        prefix.len() as u64
    );
    assert_eq!(
        prediction.score_firewall_certificate.candidate_count,
        prediction.local.max_count_tie_tokens.len() as u64
    );
    assert_eq!(
        prediction
            .score_firewall_certificate
            .forbidden_dependency_mask,
        0
    );
    assert!(prediction.prototype_complete, "{prediction:#?}");
    assert!(prediction.natural_reverse_distinct, "{prediction:#?}");
    assert!(
        prediction.permutation_cost_vector_changed,
        "{prediction:#?}"
    );
    assert!(prediction.support_matched, "{prediction:#?}");
    assert!(prediction.work_matched, "{prediction:#?}");
    assert_eq!(prediction.forbidden_reads.total(), 0);
    assert_eq!(prediction.real.arm, CorpusInducedDocumentSpinArm::Real);
    assert_eq!(
        prediction.scope_disabled.arm,
        CorpusInducedDocumentSpinArm::ScopeDisabled
    );
    assert_eq!(
        prediction.order_shuffled.arm,
        CorpusInducedDocumentSpinArm::OrderShuffled
    );
    assert_eq!(
        prediction.operator_permuted.arm,
        CorpusInducedDocumentSpinArm::OperatorPermuted
    );
    for control in [
        &prediction.scope_disabled,
        &prediction.order_shuffled,
        &prediction.operator_permuted,
    ] {
        assert_eq!(control.support_tokens, prediction.real.support_tokens);
        assert_eq!(control.work, prediction.real.work);
    }
    let mut support = prediction
        .real
        .support_tokens
        .iter()
        .map(|&token| table.decode_tokens(&[token]))
        .collect::<Result<Vec<_>, _>>()?;
    support.sort();
    assert_eq!(support, vec![b" rests".to_vec(), b" runs".to_vec()]);

    let anti_recall = CorpusInducedDocumentSpinAntiRecallIndex::compile(
        &operator,
        &table,
        &overlay,
        &construction,
    )?;
    let census = operator.target_free_census(&table, &overlay, &anti_recall, &held_out)?;
    let census_bytes = census.canonical_bytes()?;
    assert_eq!(census.canonical_bytes()?, census_bytes);
    assert_eq!(census.forbidden_reads.total(), 0);
    assert_eq!(census.support_mismatches, 0);
    assert_eq!(census.work_mismatches, 0);
    assert_eq!(census.invalid_score_firewall_certificates, 0);
    assert_eq!(
        census.score_firewall_policy_kappa,
        prediction.score_firewall_certificate.policy_kappa
    );
    assert!(!census.meets_frozen_preflight);

    let fixture_manifest_cid = cid(b"uor-r4.issue-973-natural-fixture-manifest/1");
    let fixture_corpus_cid = cid(b"uor-r4.issue-973-natural-fixture-corpus/1");
    let fixture_table_cid = table.artifact_cid();
    let fixture_overlay_cid = overlay.artifact_cid();
    let fixture_identity = ReportIdentity {
        manifest_cid: &fixture_manifest_cid,
        corpus_cid: &fixture_corpus_cid,
        table_cid: &fixture_table_cid,
        overlay_cid: &fixture_overlay_cid,
    };
    let decoded_witness = census
        .frozen_decoded_witness
        .as_ref()
        .map(|_| decode_witness(&table, &census))
        .transpose()?;
    let census_cid = census.artifact_cid()?;
    let target_free_report = TargetFreePreflightReport {
        schema: 1,
        domain: "uor-r4.issue-973-target-free-preflight/1",
        manifest_cid: fixture_identity.manifest_cid,
        corpus_cid: fixture_identity.corpus_cid,
        table_cid: fixture_identity.table_cid,
        overlay_cid: fixture_identity.overlay_cid,
        operator_cid: operator.artifact_cid()?,
        operator_stats: operator.stats(),
        anti_recall_index_kappa: anti_recall.index_kappa(),
        anti_recall_full_prefixes: anti_recall.full_prefix_count(),
        anti_recall_natural_states: anti_recall.natural_state_count(),
        anti_recall_operative_signatures: anti_recall.operative_signature_count(),
        census_cid: census_cid.clone(),
        decoded_witness: decoded_witness.as_ref(),
        census: &census,
    };
    let target_free_bytes = serde_json::to_vec(&target_free_report)?;
    let temporary_output = TemporaryReportDirectory::create()?;
    persist_and_replay(
        &temporary_output.path().join(TARGET_FREE_REPORT_FILE),
        &target_free_bytes,
        "fixture target-free report",
    )?;

    // This non-frozen fixture deliberately exercises the unavailable report
    // shape. The decision-bearing full-D3 runner below may not join targets
    // unless its frozen preflight passes.
    let evaluation =
        operator.evaluate_held_out(&table, &overlay, &anti_recall, &census, &held_out)?;
    validate_unavailable_evaluation(&evaluation, &census)?;
    let report_bytes = evaluation.canonical_bytes()?;
    assert_eq!(evaluation.canonical_bytes()?, report_bytes);
    assert_eq!(evaluation.report_cid()?, cid(&report_bytes));
    let canonical_run_bytes = persist_terminal_bundle(
        temporary_output.path(),
        fixture_identity,
        &operator.artifact_cid()?,
        &census_cid,
        &target_free_bytes,
        decoded_witness.as_ref(),
        &evaluation,
    )?;
    let persisted_evaluation: serde_json::Value = serde_json::from_slice(&fs::read(
        temporary_output.path().join(EVALUATION_REPORT_FILE),
    )?)?;
    assert_eq!(persisted_evaluation["decision"], UNAVAILABLE_TERMINAL);
    assert_eq!(persisted_evaluation["target_reads"], 0);
    let persisted_run: serde_json::Value = serde_json::from_slice(&canonical_run_bytes)?;
    assert_eq!(
        persisted_run["evaluation"]["decision"],
        UNAVAILABLE_TERMINAL
    );
    assert_eq!(persisted_run["evaluation"]["target_reads"], 0);
    Ok(())
}

fn load_frozen_d3_corpus(
    corpus_path: &Path,
) -> TestResult<(Vec<SourceDocument>, Vec<SourceDocument>)> {
    let corpus_bytes = fs::read(corpus_path)?;
    require(
        cid(&corpus_bytes) == FROZEN_CORPUS_CID,
        format!(
            "D3 corpus CID mismatch at {}: got {}",
            corpus_path.display(),
            cid(&corpus_bytes)
        ),
    )?;

    let manifest_path = corpus_path.with_file_name("manifest.json");
    let manifest_bytes = fs::read(&manifest_path)?;
    require(
        cid(&manifest_bytes) == FROZEN_MANIFEST_CID,
        format!(
            "D3 manifest CID mismatch at {}: got {}",
            manifest_path.display(),
            cid(&manifest_bytes)
        ),
    )?;
    let manifest: D3Manifest = serde_json::from_slice(&manifest_bytes)?;
    require(manifest.schema == 1, "D3 manifest schema must be 1")?;
    require(
        manifest.article_count == FROZEN_DOCUMENTS,
        "D3 manifest article count drifted",
    )?;
    require(
        manifest.corpus_cid == FROZEN_CORPUS_CID,
        "D3 manifest no longer declares the frozen corpus CID",
    )?;

    let corpus_text = std::str::from_utf8(&corpus_bytes)?;
    let mut seen_ids = BTreeSet::new();
    let mut documents = Vec::with_capacity(FROZEN_DOCUMENTS);
    for (line_index, line) in corpus_text.lines().enumerate() {
        require(
            !line.trim().is_empty(),
            format!("D3 corpus contains an empty line at {}", line_index + 1),
        )?;
        let article: D3Article = serde_json::from_str(line).map_err(|error| {
            invalid_data(format!(
                "invalid D3 article JSON at line {}: {error}",
                line_index + 1
            ))
        })?;
        require(
            seen_ids.insert(article.id.clone()),
            format!("duplicate D3 article ID {}", article.id),
        )?;
        documents.push(SourceDocument::new(article.id, article.text.into_bytes()));
    }
    require(
        documents.len() == FROZEN_DOCUMENTS,
        format!(
            "D3 corpus has {} articles, expected {FROZEN_DOCUMENTS}",
            documents.len()
        ),
    )?;
    documents.sort_by(|left, right| left.id.cmp(&right.id));
    let (held_out, construction): (Vec<_>, Vec<_>) = documents
        .into_iter()
        .partition(|document| d3_is_held_out(&document.id));
    require(
        construction.len() == FROZEN_CONSTRUCTION_DOCUMENTS,
        format!(
            "D3 construction partition has {} documents, expected {FROZEN_CONSTRUCTION_DOCUMENTS}",
            construction.len()
        ),
    )?;
    require(
        held_out.len() == FROZEN_HELD_OUT_DOCUMENTS,
        format!(
            "D3 held-out partition has {} documents, expected {FROZEN_HELD_OUT_DOCUMENTS}",
            held_out.len()
        ),
    )?;
    Ok((construction, held_out))
}

fn write_atomically(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| invalid_data(format!("output path has no parent: {}", path.display())))?;
    fs::create_dir_all(parent)?;
    if path.exists() && fs::read(path)? == bytes {
        return Ok(());
    }
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| invalid_data(format!("output filename is invalid: {}", path.display())))?;
    let temporary = parent.join(format!(".{file_name}.tmp-{}", std::process::id()));
    let mut file = fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&temporary)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    fs::rename(temporary, path)?;
    #[cfg(unix)]
    fs::File::open(parent)?.sync_all()?;
    Ok(())
}

fn persist_and_replay(path: &Path, bytes: &[u8], label: &str) -> TestResult {
    write_atomically(path, bytes)?;
    require(
        fs::read(path)? == bytes,
        format!("{label} did not replay byte-identically from disk"),
    )?;
    Ok(())
}

fn persist_terminal_bundle(
    output_dir: &Path,
    identity: ReportIdentity<'_>,
    operator_cid: &str,
    census_cid: &str,
    target_free_bytes: &[u8],
    decoded_witness: Option<&DecodedWitness>,
    evaluation: &CorpusInducedDocumentSpinEvaluation,
) -> TestResult<Vec<u8>> {
    persist_and_replay(
        &output_dir.join(TARGET_FREE_REPORT_FILE),
        target_free_bytes,
        "target-free report",
    )?;

    let evaluation_bytes = evaluation.canonical_bytes()?;
    require(
        evaluation.canonical_bytes()? == evaluation_bytes,
        "evaluation report bytes are not reproducible",
    )?;
    require(
        evaluation.report_cid()? == cid(&evaluation_bytes),
        "evaluation report CID does not bind its canonical bytes",
    )?;
    persist_and_replay(
        &output_dir.join(EVALUATION_REPORT_FILE),
        &evaluation_bytes,
        "evaluation report",
    )?;

    let canonical_run = CanonicalD3RunReport {
        schema: 1,
        domain: "uor-r4.issue-973-corpus-induced-document-spin-run/1",
        manifest_cid: identity.manifest_cid,
        corpus_cid: identity.corpus_cid,
        table_cid: identity.table_cid,
        overlay_cid: identity.overlay_cid,
        operator_cid: operator_cid.to_owned(),
        census_cid: census_cid.to_owned(),
        evaluation_report_cid: evaluation.report_cid()?,
        decoded_witness,
        evaluation,
    };
    let canonical_run_bytes = serde_json::to_vec(&canonical_run)?;
    require(
        serde_json::to_vec(&canonical_run)? == canonical_run_bytes,
        "canonical run bytes are not reproducible",
    )?;
    persist_and_replay(
        &output_dir.join(CANONICAL_RUN_FILE),
        &canonical_run_bytes,
        "canonical run report",
    )?;
    Ok(canonical_run_bytes)
}

fn load_or_compile_table(
    path: &Path,
    construction: &[SourceDocument],
) -> TestResult<SourceFreeTable> {
    let bytes = if path.exists() {
        fs::read(path)?
    } else {
        let table = SourceFreeTable::compile(construction)?;
        let bytes = table.to_bytes();
        require(
            table.artifact_cid() == FROZEN_TABLE_CID,
            format!("fresh SFTBL001 CID drifted: {}", table.artifact_cid()),
        )?;
        write_atomically(path, &bytes)?;
        bytes
    };
    let table = SourceFreeTable::from_bytes(&bytes)?;
    require(
        table.to_bytes() == bytes,
        "cached SFTBL001 artifact is not canonical",
    )?;
    require(
        table.artifact_cid() == FROZEN_TABLE_CID,
        format!("cached SFTBL001 CID drifted: {}", table.artifact_cid()),
    )?;
    require(
        table.is_bound_to_construction_documents(construction),
        "cached SFTBL001 artifact is not bound to the frozen construction set",
    )?;
    Ok(table)
}

fn load_or_compile_overlay(
    path: &Path,
    table: &SourceFreeTable,
) -> TestResult<MultiscaleCountRadiusR4V1> {
    let bytes = if path.exists() {
        fs::read(path)?
    } else {
        let overlay = MultiscaleCountRadiusR4V1::compile(table)?;
        let bytes = overlay.to_bytes();
        require(
            overlay.artifact_cid() == FROZEN_OVERLAY_CID,
            format!("fresh #953 overlay CID drifted: {}", overlay.artifact_cid()),
        )?;
        write_atomically(path, &bytes)?;
        bytes
    };
    let overlay = MultiscaleCountRadiusR4V1::from_bytes(table, &bytes)?;
    require(
        overlay.to_bytes() == bytes,
        "cached #953 overlay artifact is not canonical",
    )?;
    require(
        overlay.table_artifact_cid() == FROZEN_TABLE_CID,
        "cached #953 overlay is not bound to the frozen table",
    )?;
    require(
        overlay.artifact_cid() == FROZEN_OVERLAY_CID,
        format!(
            "cached #953 overlay CID drifted: {}",
            overlay.artifact_cid()
        ),
    )?;
    Ok(overlay)
}

fn compile_and_replay_operator(
    path: &Path,
    table: &SourceFreeTable,
    overlay: &MultiscaleCountRadiusR4V1,
    construction: &[SourceDocument],
) -> TestResult<CorpusInducedDocumentSpinPlacementR4V1> {
    // Recompile from the exact construction set on every decision-bearing run
    // so the expected CID is trusted independently of the cached bytes. The
    // frozen table and #953 overlay remain the expensive reusable inputs.
    let compiled = CorpusInducedDocumentSpinPlacementR4V1::compile(table, overlay, construction)?;
    let expected_operator_cid = compiled.artifact_cid()?;
    let compiled_bytes = compiled.to_bytes()?;
    require(
        expected_operator_cid == cid(&compiled_bytes),
        "fresh #973 operator CID does not bind its canonical bytes",
    )?;
    let bytes = if path.exists() {
        let cached = fs::read(path)?;
        require(
            cached == compiled_bytes,
            "cached #973 operator differs from the fresh construction-only compile",
        )?;
        cached
    } else {
        write_atomically(path, &compiled_bytes)?;
        compiled_bytes
    };
    let operator = CorpusInducedDocumentSpinPlacementR4V1::from_bytes(
        table,
        overlay,
        &expected_operator_cid,
        &bytes,
    )?;
    require(
        operator.to_bytes()? == bytes,
        "cached #973 operator does not replay byte identically",
    )?;
    require(
        operator.artifact_cid()? == cid(&bytes),
        "cached #973 operator CID does not bind its canonical bytes",
    )?;
    require(
        operator.table_artifact_cid() == FROZEN_TABLE_CID,
        "cached #973 operator table binding drifted",
    )?;
    require(
        operator.base_overlay_artifact_cid() == FROZEN_OVERLAY_CID,
        "cached #973 operator overlay binding drifted",
    )?;
    Ok(operator)
}

/// Decision-bearing D3 run. It caches the frozen #989 table, accepted #953
/// overlay, and aggregate #973 operator in an explicit output directory. The
/// target-free report is durably written before any evaluation call.
#[test]
#[ignore = "full frozen D3 corpus-induction experiment; approximately 20-30 minutes on M1"]
fn full_d3_corpus_induced_document_spin_placement_973() -> TestResult {
    let run_started = Instant::now();
    eprintln!(
        "#973 stage=start native_workers={}",
        std::thread::available_parallelism()
            .map_or(1, usize::from)
            .min(8)
    );
    let report_identity = FROZEN_REPORT_IDENTITY;
    let corpus_path =
        PathBuf::from(env::var_os(CORPUS_PATH_ENV).unwrap_or_else(|| DEFAULT_CORPUS_PATH.into()));
    let output_dir =
        PathBuf::from(env::var_os(OUTPUT_DIR_ENV).unwrap_or_else(|| DEFAULT_OUTPUT_DIR.into()));
    let (construction, held_out) = load_frozen_d3_corpus(&corpus_path)?;
    report_stage("corpus_loaded", run_started);

    let table = load_or_compile_table(&output_dir.join(TABLE_ARTIFACT_FILE), &construction)?;
    report_stage("table_replayed", run_started);
    require(
        table.is_disjoint_d3_held_out_documents(&held_out),
        "frozen held-out population is not strictly disjoint from construction",
    )?;
    let overlay = load_or_compile_overlay(&output_dir.join(OVERLAY_ARTIFACT_FILE), &table)?;
    report_stage("overlay_replayed", run_started);
    let operator = compile_and_replay_operator(
        &output_dir.join(OPERATOR_ARTIFACT_FILE),
        &table,
        &overlay,
        &construction,
    )?;
    report_stage("operator_recompiled_and_replayed", run_started);
    require(
        operator.construction_set_kappa() == FROZEN_CONSTRUCTION_SET_KAPPA,
        format!(
            "#973 construction-set kappa drifted: {}",
            operator.construction_set_kappa()
        ),
    )?;
    let anti_recall = CorpusInducedDocumentSpinAntiRecallIndex::compile(
        &operator,
        &table,
        &overlay,
        &construction,
    )?;
    report_stage("anti_recall_compiled", run_started);

    let census = operator.target_free_census(&table, &overlay, &anti_recall, &held_out)?;
    report_stage("target_free_census_complete", run_started);
    require(
        census.held_out_documents == FROZEN_HELD_OUT_DOCUMENTS as u64,
        "target-free census held-out population drifted",
    )?;
    require(
        census.admission_opportunities == FROZEN_TARGET_FREE_ADMISSIONS,
        format!(
            "target-free census produced {} admissions, expected {FROZEN_TARGET_FREE_ADMISSIONS}",
            census.admission_opportunities
        ),
    )?;
    require(
        census.held_out_set_kappa == FROZEN_HELD_OUT_SET_KAPPA,
        format!(
            "target-free held-out-set kappa drifted: {}",
            census.held_out_set_kappa
        ),
    )?;
    require(
        census.support_mismatches == 0 && census.work_mismatches == 0,
        "target-free census violated the matched support/work contract",
    )?;
    require(
        census.invalid_score_firewall_certificates == 0,
        "target-free census contains an invalid production score-firewall certificate",
    )?;
    require(
        !census.score_firewall_policy_kappa.is_empty(),
        "target-free census omitted the production score-firewall policy binding",
    )?;
    require(
        census.forbidden_reads.total() == 0,
        "target-free census recorded a forbidden read",
    )?;
    let decoded_witness = census
        .frozen_decoded_witness
        .as_ref()
        .map(|_| decode_witness(&table, &census))
        .transpose()?;
    let census_cid = census.artifact_cid()?;
    let target_free_report = TargetFreePreflightReport {
        schema: 1,
        domain: "uor-r4.issue-973-target-free-preflight/1",
        manifest_cid: report_identity.manifest_cid,
        corpus_cid: report_identity.corpus_cid,
        table_cid: report_identity.table_cid,
        overlay_cid: report_identity.overlay_cid,
        operator_cid: operator.artifact_cid()?,
        operator_stats: operator.stats(),
        anti_recall_index_kappa: anti_recall.index_kappa(),
        anti_recall_full_prefixes: anti_recall.full_prefix_count(),
        anti_recall_natural_states: anti_recall.natural_state_count(),
        anti_recall_operative_signatures: anti_recall.operative_signature_count(),
        census_cid: census_cid.clone(),
        decoded_witness: decoded_witness.as_ref(),
        census: &census,
    };
    let target_free_bytes = serde_json::to_vec(&target_free_report)?;
    require(
        serde_json::to_vec(&target_free_report)? == target_free_bytes,
        "target-free report bytes are not reproducible",
    )?;
    persist_and_replay(
        &output_dir.join(TARGET_FREE_REPORT_FILE),
        &target_free_bytes,
        "target-free report",
    )?;
    report_stage("target_free_report_durable", run_started);

    if !census.meets_frozen_preflight {
        // A scientifically unavailable preflight is still a valid terminal.
        // Ask the evaluator to reproduce the census and emit its zero-target-
        // read report, then stop without entering the target-join branch.
        let evaluation =
            operator.evaluate_held_out(&table, &overlay, &anti_recall, &census, &held_out)?;
        validate_unavailable_evaluation(&evaluation, &census)?;
        let canonical_run_bytes = persist_terminal_bundle(
            &output_dir,
            report_identity,
            &operator.artifact_cid()?,
            &census_cid,
            &target_free_bytes,
            decoded_witness.as_ref(),
            &evaluation,
        )?;
        report_stage("unavailable_terminal_durable", run_started);
        println!("{}", std::str::from_utf8(&canonical_run_bytes)?);
        return Ok(());
    }

    require(
        census.operative_positions.len() as u64 >= MIN_OPERATIVE_ANTI_RECALL_POSITIONS,
        "passing preflight has too few operative anti-recall positions",
    )?;
    let decoded_witness = decoded_witness
        .as_ref()
        .ok_or_else(|| invalid_data("passing preflight has no all-control decoded witness"))?;
    require(
        decoded_witness.real.token != EOS_TOKEN,
        "frozen witness real arm may not select EOS",
    )?;
    for control in [
        &decoded_witness.scope_disabled,
        &decoded_witness.order_shuffled,
        &decoded_witness.operator_permuted,
    ] {
        require(
            decoded_witness.real.token != control.token,
            "frozen witness real arm must differ from every control",
        )?;
    }

    // The only authorized target join is below this point, after the frozen
    // target-free report is complete, validated, and durably persisted.
    let evaluation =
        operator.evaluate_held_out(&table, &overlay, &anti_recall, &census, &held_out)?;
    report_stage("authorized_target_join_complete", run_started);
    require(
        evaluation.post_join_admission_opportunities == FROZEN_TARGET_FREE_ADMISSIONS,
        "post-join structural population drifted",
    )?;
    require(
        evaluation.post_join_known_target_admission_opportunities == FROZEN_KNOWN_TARGET_ADMISSIONS,
        format!(
            "post-join known-target population produced {}, expected {FROZEN_KNOWN_TARGET_ADMISSIONS}",
            evaluation.post_join_known_target_admission_opportunities
        ),
    )?;
    require(
        evaluation.operative_positions == census.operative_positions.len() as u64,
        "post-join operative population drifted from the target-free census",
    )?;
    require(
        evaluation.target_reads > 0,
        "post-join evaluation did not record its held-out target reads",
    )?;
    require(
        matches!(
            evaluation.decision.as_str(),
            POSITIVE_TERMINAL | NEGATIVE_TERMINAL
        ),
        format!(
            "a passing preflight produced an invalid terminal: {}",
            evaluation.decision
        ),
    )?;
    assert_evaluation_report(&evaluation);
    let canonical_run_bytes = persist_terminal_bundle(
        &output_dir,
        report_identity,
        &operator.artifact_cid()?,
        &census_cid,
        &target_free_bytes,
        Some(decoded_witness),
        &evaluation,
    )?;
    report_stage("decision_terminal_durable", run_started);
    println!("{}", std::str::from_utf8(&canonical_run_bytes)?);
    Ok(())
}
