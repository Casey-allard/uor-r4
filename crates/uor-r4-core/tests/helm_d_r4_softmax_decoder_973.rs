//! Decision-bearing full-decoder HELM-D-R4 softmax parity probe for #973.
//!
//! This ignored test uses the pinned local SmolLM2 donor on one deterministic
//! held-out SimpleWiki article. It changes only the coordinate frame around
//! every causal attention head, then compares full-vocabulary logits, loss,
//! top-1, decoded continuation, causal work, a byte-deterministic replay, and a
//! source-frame-permuted destructive control.

use std::env;
use std::error::Error;
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uor_r4_core::helm_d_r4_attention::{
    R4SpinCausalAttentionTransport, R4SpinTransportAudit, R4SpinTransportEvidence,
    R4SpinTransportIntervention, HELM_D_R4_GAUGE_SOFTMAX_POLICY, HELM_D_UPSTREAM_COMMIT,
};
use uor_r4_core::source_free_table::d3_is_held_out;
use uor_r4_core::transformerless::scenarios::Tokenizer;
use uor_r4_model_source::attention::{
    CausalAttentionLayerSelection, CausalAttentionTransportAudit,
};
use uor_r4_model_source::{HuggingFaceLlamaOracle, TeacherExecutionConfig};

const MODEL_ENV: &str = "UOR_R4_973_HELM_D_R4_MODEL";
const TOKENIZER_ENV: &str = "UOR_R4_973_HELM_D_R4_TOKENIZER";
const CORPUS_ENV: &str = "UOR_R4_973_HELM_D_R4_CORPUS";
const OUTPUT_ENV: &str = "UOR_R4_973_HELM_D_R4_OUTPUT";
const EVALUATED_TOKENS_ENV: &str = "UOR_R4_973_HELM_D_R4_EVALUATED_TOKENS";
const GENERATED_TOKENS_ENV: &str = "UOR_R4_973_HELM_D_R4_GENERATED_TOKENS";
const WORKERS_ENV: &str = "UOR_R4_973_HELM_D_R4_WORKERS";

const DEFAULT_MODEL: &str = "/Users/casey.allard/uor-r4/.uor-models/sources/smollm2-135m-instruct";
const DEFAULT_TOKENIZER: &str =
    "/Users/casey.allard/uor-r4/.uor-models/compiled/smollm2-135m-instruct/tokenizer.bin";
const DEFAULT_CORPUS: &str =
    "/Users/casey.allard/uor-r4/.uor-models/corpora/simple-wiki-20231101/articles.jsonl";
const DEFAULT_OUTPUT: &str =
    "/Users/casey.allard/uor-r4/.uor-models/research/issue-973-helm-d-r4-softmax/result.json";

const FROZEN_CORPUS_CID: &str =
    "blake3:194db0eebf2d49823ece01ee935447a0cc9edeaf018454ceea480ce7590132cf";
const FROZEN_DOCUMENTS: usize = 3_000;
const SCHEMA: &str = "uor-r4.helm-d-r4-softmax-decoder/1";
const DEFAULT_EVALUATED_TOKENS: usize = 3;
const DEFAULT_GENERATED_TOKENS: usize = 2;

// Predeclared before the first live donor run. The gauge lift is algebraically
// identical, but each four-lane action crosses f64/f32 at every decoder layer.
const LOGIT_ABSOLUTE_TOLERANCE: f64 = 2.0e-2;
const LOGIT_RELATIVE_TOLERANCE: f64 = 1.0e-3;
const MEAN_ABSOLUTE_LOGIT_TOLERANCE: f64 = 2.0e-3;
const NEXT_TOKEN_LOSS_TOLERANCE: f64 = 2.0e-3;
const CONTROL_MINIMUM_MAX_LOGIT_DELTA: f64 = 2.0e-2;

const PASS_TERMINAL: &str =
    "PASS_HELM_D_R4_GAUGE_SOFTMAX_FULL_DECODER_PARITY_ADVANCE_TO_INTRINSIC_R4";
const FAIL_TERMINAL: &str =
    "FAIL_HELM_D_R4_GAUGE_SOFTMAX_FULL_DECODER_PARITY_STOP_BEFORE_INTRINSIC_R4";

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

#[derive(Debug, Deserialize)]
struct Article {
    id: String,
    title: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct CorpusManifest {
    article_count: usize,
    corpus_cid: String,
}

#[derive(Debug, Clone, Serialize)]
struct PositionMetric {
    position: usize,
    observed_token: u32,
    target_token: u32,
    top1_token: u32,
    target_loss_nats: f64,
}

#[derive(Debug)]
struct ArmRun {
    logits: Vec<Vec<f32>>,
    position_metrics: Vec<PositionMetric>,
    generated_tokens: Vec<u32>,
    decoded: String,
    logits_cid: String,
    state_cid: String,
    audit: Option<CausalAttentionTransportAudit>,
    policy_identity: Option<String>,
    implementation_evidence: Option<R4SpinTransportEvidence>,
}

#[derive(Debug, Clone, Copy, Serialize)]
struct DifferenceSummary {
    maximum_absolute: f64,
    mean_absolute: f64,
    compared_logits: u64,
    every_logit_within_absolute_plus_relative_tolerance: bool,
}

#[derive(Debug, Serialize)]
struct ArmReport {
    position_metrics: Vec<PositionMetric>,
    generated_tokens: Vec<u32>,
    decoded: String,
    logits_cid: String,
    state_cid: String,
    audit: Option<AuditReport>,
    policy_identity: Option<String>,
    implementation_evidence: Option<R4SpinTransportEvidence>,
}

#[derive(Debug, Serialize)]
struct AuditReport {
    positions: u64,
    layers: u64,
    heads: u64,
    query_transforms: u64,
    key_transports: u64,
    value_transports: u64,
    output_transforms: u64,
    future_reads: u64,
    maximum_query_position: Option<usize>,
    maximum_source_position: Option<usize>,
}

impl From<CausalAttentionTransportAudit> for AuditReport {
    fn from(audit: CausalAttentionTransportAudit) -> Self {
        Self {
            positions: audit.positions,
            layers: audit.layers,
            heads: audit.heads,
            query_transforms: audit.query_transforms,
            key_transports: audit.key_transports,
            value_transports: audit.value_transports,
            output_transforms: audit.output_transforms,
            future_reads: audit.future_reads,
            maximum_query_position: audit.maximum_query_position,
            maximum_source_position: audit.maximum_source_position,
        }
    }
}

#[derive(Debug, Serialize)]
struct ResultPayload {
    schema: &'static str,
    issue: u32,
    terminal: &'static str,
    helm_d_upstream_commit: &'static str,
    helm_d_role: &'static str,
    donor_source_cid: String,
    corpus_cid: &'static str,
    held_out_document_id: String,
    held_out_document_title: String,
    evaluated_next_token_positions: usize,
    generated_tokens: usize,
    worker_policy: String,
    model_shape: ModelShape,
    tolerances: Tolerances,
    ordinary_donor: ArmReport,
    ordinary_donor_replay: ArmReport,
    coherent_r4_spin: ArmReport,
    coherent_replay: ArmReport,
    source_frame_permuted: ArmReport,
    coherent_vs_donor: DifferenceSummary,
    donor_replay_exact: bool,
    replay_exact: bool,
    all_teacher_forced_top1_equal: bool,
    all_teacher_forced_loss_within_tolerance: bool,
    decoded_continuation_equal: bool,
    control_maximum_absolute_logit_delta: f64,
    control_changed_attention_function: bool,
    causal_audit_exact: bool,
    r4_implementation_evidence_exact: bool,
    nonclaims: [&'static str; 5],
}

#[derive(Debug, Serialize)]
struct ModelShape {
    dimension: usize,
    layers: usize,
    query_heads: usize,
    kv_heads: usize,
    head_size: usize,
    vocabulary: usize,
}

#[derive(Debug, Serialize)]
struct Tolerances {
    logit_absolute: f64,
    logit_relative: f64,
    mean_absolute_logit: f64,
    next_token_loss: f64,
    control_minimum_max_logit_delta: f64,
}

#[derive(Debug, Serialize)]
struct ResultEnvelope {
    result_cid: String,
    result: ResultPayload,
}

#[test]
#[ignore = "requires the pinned 257 MiB SmolLM2 source, tokenizer, and SimpleWiki corpus"]
fn held_out_full_decoder_r4_spin_softmax_parity() -> TestResult {
    let model_path = path_from_env(MODEL_ENV, DEFAULT_MODEL);
    let tokenizer_path = path_from_env(TOKENIZER_ENV, DEFAULT_TOKENIZER);
    let corpus_path = path_from_env(CORPUS_ENV, DEFAULT_CORPUS);
    let output_path = path_from_env(OUTPUT_ENV, DEFAULT_OUTPUT);
    let evaluated_tokens = positive_usize_env(EVALUATED_TOKENS_ENV, DEFAULT_EVALUATED_TOKENS)?;
    let generated_tokens = positive_usize_env(GENERATED_TOKENS_ENV, DEFAULT_GENERATED_TOKENS)?;
    let workers = match env::var(WORKERS_ENV) {
        Ok(value) => Some(
            NonZeroUsize::new(value.parse::<usize>()?).ok_or("worker count must be positive")?,
        ),
        Err(env::VarError::NotPresent) => None,
        Err(error) => return Err(error.into()),
    };
    let execution = match workers {
        Some(workers) => TeacherExecutionConfig::fixed_workers(workers),
        None => TeacherExecutionConfig::available_parallelism(),
    };
    let worker_policy = workers.map_or_else(
        || {
            let resolved = std::thread::available_parallelism()
                .map(NonZeroUsize::get)
                .unwrap_or(1);
            format!("available_parallelism:{resolved}")
        },
        |workers| format!("fixed:{}", workers.get()),
    );

    require_file(&tokenizer_path)?;
    require_file(&corpus_path)?;
    verify_corpus_manifest(&corpus_path)?;
    if !model_path.is_dir() {
        return Err(format!("model directory is unavailable: {}", model_path.display()).into());
    }

    let tokenizer = Tokenizer::try_load(&tokenizer_path)?;
    let article = first_held_out_article(&corpus_path)?;
    let natural_text = format!("{}\n\n{}", article.title, article.text);
    let tokens = tokenizer.encode(&natural_text);
    let required_natural_tokens = evaluated_tokens
        .checked_add(1)
        .ok_or("natural-token bound overflow")?;
    if tokens.len() < required_natural_tokens {
        return Err(format!(
            "held-out article {} has {} tokens; {required_natural_tokens} required",
            article.id,
            tokens.len()
        )
        .into());
    }
    let natural_tokens = &tokens[..required_natural_tokens];
    let sequence_capacity = evaluated_tokens
        .checked_add(generated_tokens.saturating_sub(1))
        .ok_or("sequence-capacity overflow")?;

    let oracle = HuggingFaceLlamaOracle::load_with_execution(&model_path, execution)?;
    let config = oracle.cfg();
    if config.r4_attention {
        return Err(
            "ordinary donor unexpectedly enabled the historical r4_attention switch".into(),
        );
    }
    let head_size = config.dim / config.n_heads;
    if head_size % 4 != 0 {
        return Err(format!("donor head width {head_size} is not divisible by four").into());
    }
    if sequence_capacity > config.seq_len {
        return Err(format!(
            "requested sequence capacity {sequence_capacity} exceeds donor maximum {}",
            config.seq_len
        )
        .into());
    }

    let donor = run_donor(
        &oracle,
        &tokenizer,
        natural_tokens,
        evaluated_tokens,
        generated_tokens,
        sequence_capacity,
    )?;
    let donor_replay = run_donor(
        &oracle,
        &tokenizer,
        natural_tokens,
        evaluated_tokens,
        generated_tokens,
        sequence_capacity,
    )?;
    let coherent = run_transport(
        &oracle,
        &tokenizer,
        natural_tokens,
        evaluated_tokens,
        generated_tokens,
        sequence_capacity,
        R4SpinTransportIntervention::Coherent,
    )?;
    let replay = run_transport(
        &oracle,
        &tokenizer,
        natural_tokens,
        evaluated_tokens,
        generated_tokens,
        sequence_capacity,
        R4SpinTransportIntervention::Coherent,
    )?;
    let permuted = run_transport(
        &oracle,
        &tokenizer,
        natural_tokens,
        evaluated_tokens,
        generated_tokens,
        sequence_capacity,
        R4SpinTransportIntervention::SourceFramePermuted,
    )?;

    let coherent_vs_donor = compare_logits(&donor.logits, &coherent.logits)?;
    let control_difference = compare_logits(
        &coherent.logits[..evaluated_tokens],
        &permuted.logits[..evaluated_tokens],
    )?;
    let donor_replay_exact = exact_arm_replay(&donor, &donor_replay);
    let replay_exact = exact_arm_replay(&coherent, &replay);
    let all_teacher_forced_top1_equal = donor
        .position_metrics
        .iter()
        .zip(&coherent.position_metrics)
        .all(|(donor, coherent)| donor.top1_token == coherent.top1_token);
    let all_teacher_forced_loss_within_tolerance = donor
        .position_metrics
        .iter()
        .zip(&coherent.position_metrics)
        .all(|(donor, coherent)| {
            (donor.target_loss_nats - coherent.target_loss_nats).abs() <= NEXT_TOKEN_LOSS_TOLERANCE
        });
    let decoded_continuation_equal =
        donor.generated_tokens == coherent.generated_tokens && donor.decoded == coherent.decoded;
    let expected_audit = expected_audit(sequence_capacity, config.n_layers, config.n_heads)?;
    let causal_audit_exact = coherent.audit == Some(expected_audit)
        && replay.audit == Some(expected_audit)
        && permuted.audit == Some(expected_audit);
    let coherent_r4_audit = expected_r4_audit(
        sequence_capacity,
        config.n_layers,
        config.n_heads,
        head_size,
        R4SpinTransportIntervention::Coherent,
    )?;
    let permuted_r4_audit = expected_r4_audit(
        sequence_capacity,
        config.n_layers,
        config.n_heads,
        head_size,
        R4SpinTransportIntervention::SourceFramePermuted,
    )?;
    let r4_implementation_evidence_exact = r4_evidence_matches(
        &coherent,
        R4SpinTransportIntervention::Coherent,
        coherent_r4_audit,
        sequence_capacity,
    ) && r4_evidence_matches(
        &replay,
        R4SpinTransportIntervention::Coherent,
        coherent_r4_audit,
        sequence_capacity,
    ) && r4_evidence_matches(
        &permuted,
        R4SpinTransportIntervention::SourceFramePermuted,
        permuted_r4_audit,
        sequence_capacity,
    ) && coherent.implementation_evidence
        == replay.implementation_evidence
        && coherent
            .implementation_evidence
            .as_ref()
            .map(|evidence| &evidence.frame_table_offsets[..evaluated_tokens])
            == permuted
                .implementation_evidence
                .as_ref()
                .map(|evidence| &evidence.frame_table_offsets[..evaluated_tokens]);
    let control_changed_attention_function =
        control_difference.maximum_absolute >= CONTROL_MINIMUM_MAX_LOGIT_DELTA;
    let pass = coherent_vs_donor.every_logit_within_absolute_plus_relative_tolerance
        && coherent_vs_donor.mean_absolute <= MEAN_ABSOLUTE_LOGIT_TOLERANCE
        && all_teacher_forced_top1_equal
        && all_teacher_forced_loss_within_tolerance
        && decoded_continuation_equal
        && donor_replay_exact
        && replay_exact
        && causal_audit_exact
        && r4_implementation_evidence_exact
        && control_changed_attention_function;
    let terminal = if pass { PASS_TERMINAL } else { FAIL_TERMINAL };

    let payload = ResultPayload {
        schema: SCHEMA,
        issue: 973,
        terminal,
        helm_d_upstream_commit: HELM_D_UPSTREAM_COMMIT,
        helm_d_role: "pinned architectural and causal-geometric-attention source; no checkpoint parity claimed",
        donor_source_cid: oracle.source_cid().to_owned(),
        corpus_cid: FROZEN_CORPUS_CID,
        held_out_document_id: article.id,
        held_out_document_title: article.title,
        evaluated_next_token_positions: evaluated_tokens,
        generated_tokens,
        worker_policy,
        model_shape: ModelShape {
            dimension: config.dim,
            layers: config.n_layers,
            query_heads: config.n_heads,
            kv_heads: config.n_kv_heads,
            head_size,
            vocabulary: config.vocab,
        },
        tolerances: Tolerances {
            logit_absolute: LOGIT_ABSOLUTE_TOLERANCE,
            logit_relative: LOGIT_RELATIVE_TOLERANCE,
            mean_absolute_logit: MEAN_ABSOLUTE_LOGIT_TOLERANCE,
            next_token_loss: NEXT_TOKEN_LOSS_TOLERANCE,
            control_minimum_max_logit_delta: CONTROL_MINIMUM_MAX_LOGIT_DELTA,
        },
        ordinary_donor: arm_report(&donor),
        ordinary_donor_replay: arm_report(&donor_replay),
        coherent_r4_spin: arm_report(&coherent),
        coherent_replay: arm_report(&replay),
        source_frame_permuted: arm_report(&permuted),
        coherent_vs_donor,
        donor_replay_exact,
        replay_exact,
        all_teacher_forced_top1_equal,
        all_teacher_forced_loss_within_tolerance,
        decoded_continuation_equal,
        control_maximum_absolute_logit_delta: control_difference.maximum_absolute,
        control_changed_attention_function,
        causal_audit_exact,
        r4_implementation_evidence_exact,
        nonclaims: [
            "not an R4 predictive advantage",
            "not intrinsic R4 distance or centroid attention",
            "not softmax removal",
            "not transformerless or source-free serving",
            "not a correctness, reasoning, scale, or release claim",
        ],
    };
    write_report(&output_path, payload)?;

    assert!(pass, "{terminal}; report={}", output_path.display());
    Ok(())
}

fn path_from_env(name: &str, default: &str) -> PathBuf {
    env::var_os(name).map_or_else(|| PathBuf::from(default), PathBuf::from)
}

fn positive_usize_env(name: &str, default: usize) -> TestResult<usize> {
    let value = env::var(name)
        .ok()
        .map(|value| value.parse::<usize>())
        .transpose()?
        .unwrap_or(default);
    if value == 0 {
        return Err(format!("{name} must be positive").into());
    }
    Ok(value)
}

fn require_file(path: &Path) -> TestResult {
    if path.is_file() {
        Ok(())
    } else {
        Err(format!("required file is unavailable: {}", path.display()).into())
    }
}

fn first_held_out_article(path: &Path) -> TestResult<Article> {
    let reader = BufReader::new(fs::File::open(path)?);
    for line in reader.lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }
        let article: Article = serde_json::from_str(&line)?;
        if d3_is_held_out(&article.id) {
            return Ok(article);
        }
    }
    Err("no deterministic held-out article is available".into())
}

fn verify_corpus_manifest(corpus_path: &Path) -> TestResult {
    let manifest_path = corpus_path.with_file_name("manifest.json");
    require_file(&manifest_path)?;
    let manifest: CorpusManifest = serde_json::from_slice(&fs::read(&manifest_path)?)?;
    if manifest.article_count != FROZEN_DOCUMENTS || manifest.corpus_cid != FROZEN_CORPUS_CID {
        return Err(format!(
            "corpus manifest mismatch: articles={} cid={}",
            manifest.article_count, manifest.corpus_cid
        )
        .into());
    }
    let mut corpus = fs::File::open(corpus_path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = corpus.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let observed_cid = format!("blake3:{}", hasher.finalize().to_hex());
    if observed_cid != FROZEN_CORPUS_CID {
        return Err(format!(
            "corpus byte CID mismatch: expected {FROZEN_CORPUS_CID}, observed {observed_cid}"
        )
        .into());
    }
    Ok(())
}

fn run_donor(
    oracle: &HuggingFaceLlamaOracle,
    tokenizer: &Tokenizer,
    natural_tokens: &[u32],
    evaluated_tokens: usize,
    generated_tokens: usize,
    sequence_capacity: usize,
) -> TestResult<ArmRun> {
    let mut state = oracle.new_state_bounded(sequence_capacity)?;
    let mut logits = vec![0.0; oracle.cfg().vocab];
    let mut all_logits = Vec::with_capacity(sequence_capacity);
    let mut metrics = Vec::with_capacity(evaluated_tokens);
    for position in 0..evaluated_tokens {
        oracle.step_state(
            &mut state,
            natural_tokens[position] as usize,
            position,
            &mut logits,
        )?;
        metrics.push(position_metric(
            position,
            natural_tokens[position],
            natural_tokens[position + 1],
            &logits,
        ));
        all_logits.push(logits.clone());
    }
    let generated = continue_donor(
        oracle,
        &mut state,
        &mut logits,
        &mut all_logits,
        evaluated_tokens,
        generated_tokens,
    )?;
    Ok(ArmRun {
        logits_cid: logits_cid(&all_logits),
        state_cid: state.persistent_state_cid(),
        decoded: tokenizer.decode(&generated),
        generated_tokens: generated,
        position_metrics: metrics,
        logits: all_logits,
        audit: None,
        policy_identity: None,
        implementation_evidence: None,
    })
}

fn run_transport(
    oracle: &HuggingFaceLlamaOracle,
    tokenizer: &Tokenizer,
    natural_tokens: &[u32],
    evaluated_tokens: usize,
    generated_tokens: usize,
    sequence_capacity: usize,
    intervention: R4SpinTransportIntervention,
) -> TestResult<ArmRun> {
    let transport = R4SpinCausalAttentionTransport::new(
        u32::try_from(oracle.cfg().vocab - 1)?,
        sequence_capacity,
        intervention,
    )?;
    let mut session = oracle.new_causal_attention_transport_session(
        Box::new(transport),
        CausalAttentionLayerSelection::All,
        sequence_capacity,
    )?;
    let mut logits = vec![0.0; oracle.cfg().vocab];
    let mut all_logits = Vec::with_capacity(sequence_capacity);
    let mut metrics = Vec::with_capacity(evaluated_tokens);
    for position in 0..evaluated_tokens {
        oracle.step_causal_attention_transport(
            &mut session,
            natural_tokens[position] as usize,
            position,
            &mut logits,
        )?;
        metrics.push(position_metric(
            position,
            natural_tokens[position],
            natural_tokens[position + 1],
            &logits,
        ));
        all_logits.push(logits.clone());
    }
    let generated = continue_transport(
        oracle,
        &mut session,
        &mut logits,
        &mut all_logits,
        evaluated_tokens,
        generated_tokens,
    )?;
    let policy_identity = session.policy_identity().to_owned();
    let implementation_evidence = session
        .transport_implementation_evidence()?
        .ok_or("R4 transport did not emit implementation evidence")?;
    let implementation_evidence = serde_json::from_str(&implementation_evidence)?;
    Ok(ArmRun {
        logits_cid: logits_cid(&all_logits),
        state_cid: session.persistent_state_cid(),
        decoded: tokenizer.decode(&generated),
        generated_tokens: generated,
        position_metrics: metrics,
        logits: all_logits,
        audit: Some(session.audit()),
        policy_identity: Some(policy_identity),
        implementation_evidence: Some(implementation_evidence),
    })
}

fn continue_donor(
    oracle: &HuggingFaceLlamaOracle,
    state: &mut uor_r4_model_source::State,
    logits: &mut [f32],
    all_logits: &mut Vec<Vec<f32>>,
    next_position: usize,
    generated_tokens: usize,
) -> TestResult<Vec<u32>> {
    let mut generated = Vec::with_capacity(generated_tokens);
    generated.push(u32::try_from(argmax(logits))?);
    for offset in 1..generated_tokens {
        let token = generated[offset - 1];
        oracle.step_state(state, token as usize, next_position + offset - 1, logits)?;
        all_logits.push(logits.to_vec());
        generated.push(u32::try_from(argmax(logits))?);
    }
    Ok(generated)
}

fn continue_transport(
    oracle: &HuggingFaceLlamaOracle,
    session: &mut uor_r4_model_source::CausalAttentionTransportSession,
    logits: &mut [f32],
    all_logits: &mut Vec<Vec<f32>>,
    next_position: usize,
    generated_tokens: usize,
) -> TestResult<Vec<u32>> {
    let mut generated = Vec::with_capacity(generated_tokens);
    generated.push(u32::try_from(argmax(logits))?);
    for offset in 1..generated_tokens {
        let token = generated[offset - 1];
        oracle.step_causal_attention_transport(
            session,
            token as usize,
            next_position + offset - 1,
            logits,
        )?;
        all_logits.push(logits.to_vec());
        generated.push(u32::try_from(argmax(logits))?);
    }
    Ok(generated)
}

fn position_metric(
    position: usize,
    observed_token: u32,
    target_token: u32,
    logits: &[f32],
) -> PositionMetric {
    PositionMetric {
        position,
        observed_token,
        target_token,
        top1_token: u32::try_from(argmax(logits)).unwrap_or(u32::MAX),
        target_loss_nats: cross_entropy(logits, target_token as usize),
    }
}

fn argmax(values: &[f32]) -> usize {
    let mut winner = 0;
    for index in 1..values.len() {
        if values[index] > values[winner] {
            winner = index;
        }
    }
    winner
}

fn cross_entropy(logits: &[f32], target: usize) -> f64 {
    let maximum = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max) as f64;
    let normalizer = logits
        .iter()
        .map(|logit| (f64::from(*logit) - maximum).exp())
        .sum::<f64>()
        .ln()
        + maximum;
    normalizer - f64::from(logits[target])
}

fn compare_logits(left: &[Vec<f32>], right: &[Vec<f32>]) -> TestResult<DifferenceSummary> {
    if left.len() != right.len() || left.is_empty() {
        return Err("logit traces are empty or have different position counts".into());
    }
    let mut maximum_absolute = 0.0f64;
    let mut absolute_sum = 0.0f64;
    let mut compared = 0u64;
    let mut every_within = true;
    for (left_row, right_row) in left.iter().zip(right) {
        if left_row.len() != right_row.len() || left_row.is_empty() {
            return Err("logit rows are empty or have different widths".into());
        }
        for (left_value, right_value) in left_row.iter().zip(right_row) {
            let left_value = f64::from(*left_value);
            let right_value = f64::from(*right_value);
            let difference = (left_value - right_value).abs();
            let allowance = LOGIT_ABSOLUTE_TOLERANCE
                + LOGIT_RELATIVE_TOLERANCE * left_value.abs().max(right_value.abs());
            maximum_absolute = maximum_absolute.max(difference);
            absolute_sum += difference;
            compared = compared.saturating_add(1);
            every_within &= difference <= allowance;
        }
    }
    Ok(DifferenceSummary {
        maximum_absolute,
        mean_absolute: absolute_sum / compared as f64,
        compared_logits: compared,
        every_logit_within_absolute_plus_relative_tolerance: every_within,
    })
}

fn exact_arm_replay(left: &ArmRun, right: &ArmRun) -> bool {
    left.logits_cid == right.logits_cid
        && left.state_cid == right.state_cid
        && left.generated_tokens == right.generated_tokens
        && left.decoded == right.decoded
        && left.audit == right.audit
        && left.policy_identity == right.policy_identity
        && left.implementation_evidence == right.implementation_evidence
        && left
            .logits
            .iter()
            .flatten()
            .zip(right.logits.iter().flatten())
            .all(|(left, right)| left.to_bits() == right.to_bits())
}

fn r4_evidence_matches(
    run: &ArmRun,
    intervention: R4SpinTransportIntervention,
    expected_audit: R4SpinTransportAudit,
    expected_positions: usize,
) -> bool {
    run.policy_identity.as_deref() == Some(HELM_D_R4_GAUGE_SOFTMAX_POLICY)
        && run
            .implementation_evidence
            .as_ref()
            .is_some_and(|evidence| {
                evidence.schema == "uor-r4.r4-spin-transport-evidence/1"
                    && evidence.policy_identity == HELM_D_R4_GAUGE_SOFTMAX_POLICY
                    && evidence.intervention == intervention
                    && evidence.frame_table_offsets.len() == expected_positions
                    && evidence.audit == expected_audit
            })
}

fn expected_audit(
    positions: usize,
    layers: usize,
    heads: usize,
) -> TestResult<CausalAttentionTransportAudit> {
    let positions_u64 = u64::try_from(positions)?;
    let layers_u64 = u64::try_from(layers)?;
    let heads_u64 = u64::try_from(heads)?;
    let layer_calls = positions_u64
        .checked_mul(layers_u64)
        .ok_or("audit layer count overflow")?;
    let head_calls = layer_calls
        .checked_mul(heads_u64)
        .ok_or("audit head count overflow")?;
    let prefix_sources = positions_u64
        .checked_mul(positions_u64 + 1)
        .and_then(|value| value.checked_div(2))
        .ok_or("audit prefix count overflow")?;
    let source_calls = prefix_sources
        .checked_mul(layers_u64)
        .and_then(|value| value.checked_mul(heads_u64))
        .ok_or("audit source count overflow")?;
    Ok(CausalAttentionTransportAudit {
        positions: positions_u64,
        layers: layer_calls,
        heads: head_calls,
        query_transforms: head_calls,
        key_transports: source_calls,
        value_transports: source_calls,
        output_transforms: head_calls,
        future_reads: 0,
        maximum_query_position: Some(positions - 1),
        maximum_source_position: Some(positions - 1),
    })
}

fn expected_r4_audit(
    positions: usize,
    layers: usize,
    heads: usize,
    head_size: usize,
    intervention: R4SpinTransportIntervention,
) -> TestResult<R4SpinTransportAudit> {
    let positions = u64::try_from(positions)?;
    let layers = u64::try_from(layers)?;
    let heads = u64::try_from(heads)?;
    let blocks = u64::try_from(head_size / 4)?;
    let query_blocks = positions
        .checked_mul(layers)
        .and_then(|value| value.checked_mul(heads))
        .and_then(|value| value.checked_mul(blocks))
        .ok_or("R4 query-block audit overflow")?;
    let prefix_sources = positions
        .checked_mul(positions + 1)
        .and_then(|value| value.checked_div(2))
        .ok_or("R4 prefix audit overflow")?;
    let source_blocks = prefix_sources
        .checked_mul(layers)
        .and_then(|value| value.checked_mul(heads))
        .and_then(|value| value.checked_mul(blocks))
        .ok_or("R4 source-block audit overflow")?;
    let encoded = query_blocks
        .checked_add(
            source_blocks
                .checked_mul(2)
                .ok_or("R4 encoding audit overflow")?,
        )
        .ok_or("R4 encoding audit overflow")?;
    let source_frame_permutations = match intervention {
        R4SpinTransportIntervention::Coherent => 0,
        R4SpinTransportIntervention::SourceFramePermuted => prefix_sources
            .saturating_sub(1)
            .checked_mul(layers)
            .and_then(|value| value.checked_mul(heads))
            .and_then(|value| value.checked_mul(blocks))
            .and_then(|value| value.checked_mul(2))
            .ok_or("R4 permutation audit overflow")?,
    };
    Ok(R4SpinTransportAudit {
        positions_prepared: positions,
        r4_blocks_encoded: encoded,
        key_blocks_transported: source_blocks,
        value_blocks_transported: source_blocks,
        output_blocks_decoded: query_blocks,
        future_position_reads: 0,
        source_frame_permutations,
    })
}

fn logits_cid(logits: &[Vec<f32>]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"uor-r4.helm-d-r4-softmax-logits/1");
    for row in logits {
        hasher.update(&(row.len() as u64).to_le_bytes());
        for value in row {
            hasher.update(&value.to_bits().to_le_bytes());
        }
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn arm_report(run: &ArmRun) -> ArmReport {
    ArmReport {
        position_metrics: run.position_metrics.clone(),
        generated_tokens: run.generated_tokens.clone(),
        decoded: run.decoded.clone(),
        logits_cid: run.logits_cid.clone(),
        state_cid: run.state_cid.clone(),
        audit: run.audit.map(AuditReport::from),
        policy_identity: run.policy_identity.clone(),
        implementation_evidence: run.implementation_evidence.clone(),
    }
}

fn write_report(path: &Path, result: ResultPayload) -> TestResult {
    let canonical = serde_json::to_vec(&result)?;
    let result_cid = format!("blake3:{}", blake3::hash(&canonical).to_hex());
    let envelope = ResultEnvelope { result_cid, result };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(&envelope)?)?;
    Ok(())
}
