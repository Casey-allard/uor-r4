//! Runnable issue #950 control/treatment spike.
//!
//! This is intentionally one bounded product-facing command, not a benchmark
//! or training framework.  It loads the exact pinned local source snapshot,
//! applies its recorded chat template and source tokenizer, executes the five
//! greedy 32-token source controls plus one bounded treatment rollout, and
//! retains the structural and persistence evidence needed by G0.

use std::collections::HashSet;
use std::io;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uor_r4_core::transformerless::hf_bpe::{HfBpeTokenizer, TokenizerAdapter};
use uor_r4_model_source::geometric_decoder::{
    GeometricDecoderSession, GeometricMixer, GeometricOperatorTrace, GeometryContext,
    GeometryIntervention, GeometryMemorySpan, GeometryProvenance, DEFAULT_SUPPORT_BUDGET,
    MAX_MEMORY_SPANS, MAX_MEMORY_TOKENS,
};
use uor_r4_model_source::{
    BatchedTeacher, ExactBackendReport, HuggingFaceLlamaOracle, TeacherExecutionConfig,
};
use uor_r4_router::decoder_memory::TokenizerBoundMemorySpan;
use uor_r4_router::UorR4Router;

pub const REPORT_SCHEMA: &str = "uor-r4.geometric-decoder-spike/1";
pub const SOURCE_REPOSITORY: &str = "HuggingFaceTB/SmolLM2-135M-Instruct";
pub const PINNED_SOURCE_REVISION: &str = "7e27bd9f95328f0f3b08261d1252705110c806f8";
pub const PINNED_SOURCE_CID: &str =
    "blake3:12d2cd8a877ef2cdcf785b3d4d1f373e0419074cc884aeaff06fc059686a5ba5";
pub const PINNED_TOKENIZER_CID: &str =
    "blake3:944d1262d516abd56a8156dd3058a73a1bf3dc19419527592d854d162f288073";
pub const GENERATED_TOKENS: usize = 32;
/// G0 keeps the coherent substrate from the first 29 source layers and proves
/// one attention-free geometric seam at the final layer. Earlier-layer fitting
/// and comparative quality are deliberately deferred to G1 (#951).
pub const TARGET_LAYER: usize = 29;
pub const OTHER_IDENTITY: &str = "issue-950-isolation-probe";

/// Exact Jinja source carried by the pinned tokenizer snapshot.  The command
/// verifies this literal before rendering the one-user-turn specialization.
pub const PINNED_CHAT_TEMPLATE: &str = "{% for message in messages %}{% if loop.first and messages[0]['role'] != 'system' %}{{ '<|im_start|>system\nYou are a helpful AI assistant named SmolLM, trained by Hugging Face<|im_end|>\n' }}{% endif %}{{'<|im_start|>' + message['role'] + '\n' + message['content'] + '<|im_end|>' + '\n'}}{% endfor %}{% if add_generation_prompt %}{{ '<|im_start|>assistant\n' }}{% endif %}";

pub const FROZEN_PROMPTS: [&str; 5] = [
    "Explain in three short sentences why plants need sunlight.",
    "Give three practical tips for staying organized at work, with one brief explanation for each.",
    "Describe a rainy city morning in three vivid sentences.",
    "Explain in simple terms how a bicycle stays balanced while moving.",
    "Write a friendly three-sentence welcome message for a new teammate and mention collaboration.",
];

#[derive(Clone, Debug)]
pub struct GeometricSpikeConfig {
    pub source: PathBuf,
    pub source_revision: String,
    pub output: PathBuf,
    pub router_state_output: PathBuf,
    pub identity: String,
    pub workers: NonZeroUsize,
    /// Optional retained G0 control report. When present, its source,
    /// tokenizer, template, backend, decode contract, and all five transcripts
    /// are revalidated before only the disabled/treatment repair is rerun.
    pub control_report: Option<PathBuf>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceBinding {
    pub repository: String,
    pub revision: String,
    pub weights_cid: String,
    pub tokenizer: TokenizerAdapter,
    pub chat_template: String,
    pub chat_template_cid: String,
    pub backend: ExactBackendReport,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecodeContract {
    pub selection: String,
    pub generated_tokens: usize,
    pub eos_policy: String,
    pub cycle_periods_rejected: Vec<usize>,
    pub target_layer: usize,
    pub mixer_scope: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrozenRubric {
    pub grammatical: String,
    pub prompt_responsive: String,
    pub truncation: String,
    pub required_passing_prompts: usize,
    pub review_status: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RolloutTranscript {
    pub prompt_id: String,
    pub prompt: String,
    pub input_tokens: usize,
    pub generated_token_ids: Vec<u32>,
    pub raw_decoded: String,
    pub response_text: String,
    pub first_eos_offset: Option<usize>,
    pub utf8_decodable: bool,
    pub short_cycle_period: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TreatmentTranscript {
    pub rollout: RolloutTranscript,
    pub memory_spans: usize,
    pub memory_tokens: usize,
    pub router_state_cid: String,
    pub geometry_position_states: usize,
    pub mixer_checkpoint_identity: String,
    pub decision_trace: Vec<GeometricOperatorTrace>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DisabledControlEvidence {
    pub positions_compared: usize,
    pub logits_bit_exact: bool,
    pub generated_tokens_equal: bool,
    pub disabled_trace_entries: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReachabilityEvidence {
    pub prompt_id: String,
    pub compared_position: usize,
    pub real_support_cid: String,
    pub permuted_support_cid: String,
    pub support_changed: bool,
    pub changed_logits: usize,
    pub logit_linf_delta: f32,
    pub verdict: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersistenceEvidence {
    pub committed_turns: usize,
    pub exported_state_cid: String,
    pub restored_state_cid: String,
    pub same_identity_turns: usize,
    pub same_identity_exact: bool,
    pub other_identity_turns: usize,
    pub tokenizer_cid_preserved: bool,
    pub adapter_identity_preserved: bool,
    pub ordered_tokens_preserved: bool,
    pub verdict: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuntimeGates {
    pub five_control_rollouts: bool,
    pub all_control_rollouts_exactly_32_tokens: bool,
    pub no_short_cycles: bool,
    pub distinct_control_sequences: usize,
    pub one_treatment_rollout: bool,
    pub every_treatment_decision_executed_mixer: bool,
    pub all_treatments_decodable: bool,
    pub disabled_reproduces_control: bool,
    pub structural_reachability: bool,
    pub persistence_round_trip: bool,
    pub verdict: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ControlProvenance {
    pub execution: String,
    pub reused_report_cid: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeometricSpikeReport {
    pub schema: String,
    pub issue: u32,
    pub claim_scope: String,
    pub source: SourceBinding,
    pub decode: DecodeContract,
    pub rubric: FrozenRubric,
    pub identity: String,
    pub memory_adapter_identity: String,
    #[serde(default)]
    pub control_provenance: ControlProvenance,
    pub control: Vec<RolloutTranscript>,
    pub treatment: Vec<TreatmentTranscript>,
    pub disabled_control: DisabledControlEvidence,
    pub reachability: ReachabilityEvidence,
    pub persistence: PersistenceEvidence,
    pub gates: RuntimeGates,
}

#[derive(Debug)]
pub enum GeometricSpikeError {
    Io(io::Error),
    InvalidSource(String),
    Source(String),
    Tokenizer(String),
    Decoder(String),
    Router(String),
    Gate(String),
}

impl std::fmt::Display for GeometricSpikeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => error.fmt(formatter),
            Self::InvalidSource(error) => write!(formatter, "invalid pinned source: {error}"),
            Self::Source(error) => write!(formatter, "source control unavailable: {error}"),
            Self::Tokenizer(error) => write!(formatter, "source tokenizer unavailable: {error}"),
            Self::Decoder(error) => write!(formatter, "geometric treatment unavailable: {error}"),
            Self::Router(error) => write!(
                formatter,
                "tokenizer-bound router memory unavailable: {error}"
            ),
            Self::Gate(error) => write!(formatter, "issue #950 runtime gate failed: {error}"),
        }
    }
}

impl std::error::Error for GeometricSpikeError {}

impl From<io::Error> for GeometricSpikeError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// Execute the bounded spike, retain the JSON report and reloadable router
/// state, then fail the command if any structural/runtime gate is negative.
pub fn run_geometric_spike(
    config: &GeometricSpikeConfig,
) -> Result<GeometricSpikeReport, GeometricSpikeError> {
    validate_source(&config.source, &config.source_revision)?;
    let tokenizer = HfBpeTokenizer::from_dir(&config.source)
        .map_err(|error| GeometricSpikeError::Tokenizer(error.to_string()))?;
    let tokenizer_adapter = tokenizer.adapter();
    let tokenizer_cid = tokenizer.address();
    if tokenizer_cid != PINNED_TOKENIZER_CID {
        return Err(GeometricSpikeError::InvalidSource(format!(
            "tokenizer CID {tokenizer_cid} != pinned {PINNED_TOKENIZER_CID}"
        )));
    }
    let chat_template = read_chat_template(&config.source)?;
    if chat_template != PINNED_CHAT_TEMPLATE {
        return Err(GeometricSpikeError::InvalidSource(
            "tokenizer_config.json chat_template does not match the pinned SmolLM2 template"
                .to_owned(),
        ));
    }
    let rendered_prompts = FROZEN_PROMPTS
        .iter()
        .map(|prompt| render_chat_prompt(prompt))
        .collect::<Vec<_>>();
    let encoded_prompts = rendered_prompts
        .iter()
        .map(|prompt| tokenizer.encode(prompt))
        .collect::<Vec<_>>();
    let horizon = encoded_prompts
        .iter()
        .map(Vec::len)
        .max()
        .unwrap_or(0)
        .checked_add(GENERATED_TOKENS)
        .ok_or_else(|| GeometricSpikeError::Source("sequence horizon overflow".to_owned()))?;
    if horizon == 0 {
        return Err(GeometricSpikeError::Source(
            "frozen prompts produced an empty horizon".to_owned(),
        ));
    }
    let execution = TeacherExecutionConfig::fixed_workers(config.workers);
    let oracle = HuggingFaceLlamaOracle::load_with_sequence_length_and_execution(
        &config.source,
        horizon,
        execution,
    )
    .map_err(|error| GeometricSpikeError::Source(error.to_string()))?;
    if oracle.cfg().vocab != tokenizer.vocab_size() {
        return Err(GeometricSpikeError::InvalidSource(format!(
            "model vocab {} != tokenizer vocab {}",
            oracle.cfg().vocab,
            tokenizer.vocab_size()
        )));
    }
    if oracle.source_cid() != PINNED_SOURCE_CID {
        return Err(GeometricSpikeError::InvalidSource(format!(
            "weights CID {} != pinned {PINNED_SOURCE_CID}",
            oracle.source_cid()
        )));
    }

    let mut mixer_seed = Vec::new();
    mixer_seed.extend_from_slice(oracle.source_cid().as_bytes());
    mixer_seed.extend_from_slice(tokenizer_cid.as_bytes());
    mixer_seed.extend_from_slice(b"issue-950-one-layer-r4-spike");
    let mixer = GeometricMixer::deterministic(TARGET_LAYER, oracle.cfg().dim, &mixer_seed)
        .map_err(|error| GeometricSpikeError::Decoder(error.to_string()))?;
    let adapter_identity = mixer.memory_adapter_identity(oracle.source_cid(), &tokenizer_cid);

    let empty_context = GeometryContext::new(
        config.identity.clone(),
        tokenizer_cid.clone(),
        adapter_identity.clone(),
        [0.0; 4],
        Vec::new(),
        GeometryProvenance {
            source_cid: oracle.source_cid().to_owned(),
            router_state_cid: format!("blake3:{}", blake3::hash(b"empty-router-state").to_hex()),
            memory_source: "disabled-control-empty-context".to_owned(),
        },
    )
    .map_err(|error| GeometricSpikeError::Decoder(error.to_string()))?;

    // The source runtime's exact multi-stream implementation advances five
    // independent private states through shared immutable weights.  This is
    // the same source model and arithmetic as a serial control, with the
    // otherwise dominant weight traversal amortized across the frozen smoke.
    let (control, control_provenance) = if let Some(control_report) = &config.control_report {
        load_reused_controls(
            control_report,
            &oracle,
            &tokenizer,
            &tokenizer_adapter,
            &chat_template,
            &config.source_revision,
            &encoded_prompts,
        )?
    } else {
        (
            source_rollouts_batched(&oracle, &tokenizer, &encoded_prompts)?,
            ControlProvenance {
                execution: "exact five-lane shared-weight source batch in this command".to_owned(),
                reused_report_cid: None,
            },
        )
    };

    // Independently replay G0-P1 through the ordinary source path and the
    // disabled experimental seam.  Bit equality establishes that installing
    // the seam cannot perturb the retained source control when disabled.
    let disabled = oracle
        .new_geometric_session(
            mixer.clone(),
            empty_context,
            GeometryIntervention::Disabled,
            encoded_prompts[0].len() + GENERATED_TOKENS,
        )
        .map_err(|error| GeometricSpikeError::Decoder(error.to_string()))?;
    let (serial_control, disabled_evidence) = source_rollout(
        &oracle,
        &tokenizer,
        "G0-P1",
        FROZEN_PROMPTS[0],
        &rendered_prompts[0],
        &encoded_prompts[0],
        Some(disabled),
    )?;
    if serial_control.generated_token_ids != control[0].generated_token_ids {
        return Err(GeometricSpikeError::Gate(
            "batched G0-P1 control diverged from the serial source path".to_owned(),
        ));
    }
    let mut disabled_control = disabled_evidence.ok_or_else(|| {
        GeometricSpikeError::Gate("disabled control comparison was not executed".to_owned())
    })?;
    disabled_control.generated_tokens_equal =
        serial_control.generated_token_ids == control[0].generated_token_ids;

    let mut router = UorR4Router::new(0.5);
    let mut treatment = Vec::with_capacity(1);
    let mut reachability = None;
    for (index, ((prompt, rendered), input)) in FROZEN_PROMPTS
        .iter()
        .zip(rendered_prompts.iter())
        .zip(encoded_prompts.iter())
        .enumerate()
    {
        let prompt_id = format!("G0-P{}", index + 1);
        let prompt_tokens = tokenizer.encode(prompt);
        router
            .commit_tokenizer_bound_turn(
                &config.identity,
                "user",
                prompt,
                &prompt_tokens,
                &tokenizer_cid,
                &adapter_identity,
                oracle.source_cid(),
            )
            .map_err(|error| GeometricSpikeError::Router(error.to_string()))?;
        let context = geometry_context_from_router(
            &router,
            &config.identity,
            &tokenizer_cid,
            &adapter_identity,
            oracle.source_cid(),
        )?;
        let (transcript, probe) = treatment_rollout(
            &oracle,
            &tokenizer,
            &mixer,
            context,
            &prompt_id,
            prompt,
            rendered,
            input,
            index == 0,
        )?;
        let answer_tokens = tokenizer.encode(&transcript.rollout.response_text);
        let answer_text = if transcript.rollout.response_text.trim().is_empty() {
            transcript.rollout.raw_decoded.as_str()
        } else {
            transcript.rollout.response_text.as_str()
        };
        router
            .commit_tokenizer_bound_turn(
                &config.identity,
                "assistant",
                answer_text,
                if answer_tokens.is_empty() {
                    &transcript.rollout.generated_token_ids
                } else {
                    &answer_tokens
                },
                &tokenizer_cid,
                &adapter_identity,
                oracle.source_cid(),
            )
            .map_err(|error| GeometricSpikeError::Router(error.to_string()))?;
        if probe.is_some() {
            reachability = probe;
        }
        treatment.push(transcript);
        // G0 establishes an active, causal, bounded layer seam and one real
        // product rollout.  Fitted five-prompt treatment quality belongs to
        // G1 (#951), after this structural gate has promoted.
        break;
    }
    let reachability = reachability.ok_or_else(|| {
        GeometricSpikeError::Gate("controlled coordinate probe was not executed".to_owned())
    })?;

    let persistence = persist_and_reload(
        &router,
        &config.router_state_output,
        &config.identity,
        &tokenizer_cid,
        &adapter_identity,
        oracle.source_cid(),
    )?;
    let gates = evaluate_runtime_gates(
        &control,
        &treatment,
        &disabled_control,
        &reachability,
        &persistence,
    );
    let report = GeometricSpikeReport {
        schema: REPORT_SCHEMA.to_owned(),
        issue: 950,
        claim_scope: "experimental off-serving one-layer control viability and structural reachability; no training, geometric quality advantage, all-layer transformerless, production, performance, or multiplication-free claim".to_owned(),
        source: SourceBinding {
            repository: SOURCE_REPOSITORY.to_owned(),
            revision: config.source_revision.clone(),
            weights_cid: oracle.source_cid().to_owned(),
            tokenizer: tokenizer_adapter,
            chat_template,
            chat_template_cid: format!(
                "blake3:{}",
                blake3::hash(PINNED_CHAT_TEMPLATE.as_bytes()).to_hex()
            ),
            backend: oracle.exact_backend_report(),
        },
        decode: DecodeContract {
            selection: "deterministic greedy argmax; lower token id wins an exact tie".to_owned(),
            generated_tokens: GENERATED_TOKENS,
            eos_policy: "retain exactly 32 local token decisions; render response through the first EOS while retaining the complete token transcript".to_owned(),
            cycle_periods_rejected: vec![1, 2, 3, 4],
            target_layer: TARGET_LAYER,
            mixer_scope: "one experimental layer; all other source-attention layers retained".to_owned(),
        },
        rubric: FrozenRubric {
            grammatical: "PASS when the rendered response contains an intelligible English clause or structured list item without corruption or a repetition loop; a cap-truncated final clause is reviewed separately".to_owned(),
            prompt_responsive: "PASS when the response directly performs the requested explanation, tips, description, or welcome rather than changing topic or emitting only boilerplate".to_owned(),
            truncation: "The 32-token cap may truncate the last clause; earlier complete responsive material remains reviewable, but an otherwise fragmentary response fails".to_owned(),
            required_passing_prompts: 4,
            review_status: "PENDING_OPERATOR_REVIEW_IN_ISSUE_RECORD".to_owned(),
        },
        identity: config.identity.clone(),
        memory_adapter_identity: adapter_identity,
        control_provenance,
        control,
        treatment,
        disabled_control,
        reachability,
        persistence,
        gates,
    };
    write_json(&config.output, &report)?;
    if report.gates.verdict != "PASS" {
        return Err(GeometricSpikeError::Gate(format!(
            "runtime report retained at {} with verdict {}",
            config.output.display(),
            report.gates.verdict
        )));
    }
    Ok(report)
}

fn source_rollout(
    oracle: &HuggingFaceLlamaOracle,
    tokenizer: &HfBpeTokenizer,
    prompt_id: &str,
    prompt: &str,
    _rendered: &str,
    input: &[u32],
    mut disabled: Option<GeometricDecoderSession>,
) -> Result<(RolloutTranscript, Option<DisabledControlEvidence>), GeometricSpikeError> {
    let capacity = input.len() + GENERATED_TOKENS;
    let mut state = oracle
        .new_state_bounded(capacity)
        .map_err(|error| GeometricSpikeError::Source(error.to_string()))?;
    let mut logits = vec![0.0; oracle.cfg().vocab];
    let mut disabled_logits = vec![0.0; oracle.cfg().vocab];
    let mut positions_compared = 0usize;
    let mut logits_bit_exact = true;
    for (position, &token) in input.iter().enumerate() {
        oracle
            .step_state(&mut state, token as usize, position, &mut logits)
            .map_err(|error| GeometricSpikeError::Source(error.to_string()))?;
        if let Some(session) = disabled.as_mut() {
            oracle
                .step_geometric(session, token as usize, position, &mut disabled_logits)
                .map_err(|error| GeometricSpikeError::Decoder(error.to_string()))?;
            positions_compared += 1;
            logits_bit_exact &= same_f32_bits(&logits, &disabled_logits);
        }
    }
    let mut generated = Vec::with_capacity(GENERATED_TOKENS);
    generated.push(greedy_token(&logits)?);
    for offset in 1..GENERATED_TOKENS {
        let position = input.len() - 1 + offset;
        let token = generated[offset - 1];
        oracle
            .step_state(&mut state, token as usize, position, &mut logits)
            .map_err(|error| GeometricSpikeError::Source(error.to_string()))?;
        if let Some(session) = disabled.as_mut() {
            oracle
                .step_geometric(session, token as usize, position, &mut disabled_logits)
                .map_err(|error| GeometricSpikeError::Decoder(error.to_string()))?;
            positions_compared += 1;
            logits_bit_exact &= same_f32_bits(&logits, &disabled_logits);
        }
        generated.push(greedy_token(&logits)?);
    }
    let transcript = transcript(tokenizer, prompt_id, prompt, input.len(), generated);
    let comparison = disabled.map(|session| DisabledControlEvidence {
        positions_compared,
        logits_bit_exact,
        generated_tokens_equal: logits_bit_exact,
        disabled_trace_entries: session.traces().len(),
    });
    Ok((transcript, comparison))
}

fn source_rollouts_batched(
    oracle: &HuggingFaceLlamaOracle,
    tokenizer: &HfBpeTokenizer,
    inputs: &[Vec<u32>],
) -> Result<Vec<RolloutTranscript>, GeometricSpikeError> {
    if inputs.len() != FROZEN_PROMPTS.len() || inputs.iter().any(Vec::is_empty) {
        return Err(GeometricSpikeError::Source(
            "the frozen batch must contain five non-empty source prompts".to_owned(),
        ));
    }
    let sequence_capacity = inputs
        .iter()
        .map(|input| input.len() + GENERATED_TOKENS)
        .max()
        .ok_or_else(|| GeometricSpikeError::Source("empty source batch".to_owned()))?;
    let measured_steps = sequence_capacity - 1;
    let mut states = (0..inputs.len())
        .map(|_| oracle.new_state_bounded(sequence_capacity))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| GeometricSpikeError::Source(error.to_string()))?;
    let mut generated = vec![Vec::with_capacity(GENERATED_TOKENS); inputs.len()];
    let mut tokens = vec![0usize; inputs.len()];
    let mut positions = vec![0usize; inputs.len()];

    for position in 0..measured_steps {
        for (lane, input) in inputs.iter().enumerate() {
            tokens[lane] = if position < input.len() {
                input[position] as usize
            } else if generated[lane].len() < GENERATED_TOKENS {
                generated[lane].last().copied().ok_or_else(|| {
                    GeometricSpikeError::Source(format!(
                        "lane {lane} reached generation before its first source decision"
                    ))
                })? as usize
            } else {
                // The lane is already complete; keep its private state valid
                // while longer lanes finish. This token is never retained.
                2
            };
            positions[lane] = position;
        }
        oracle.forward_batch_into(&mut states, &tokens, &positions);
        for (lane, input) in inputs.iter().enumerate() {
            if position + 1 >= input.len() && generated[lane].len() < GENERATED_TOKENS {
                let logits = <HuggingFaceLlamaOracle as BatchedTeacher>::logits_mut(
                    oracle,
                    &mut states[lane],
                );
                generated[lane].push(greedy_token(logits)?);
            }
        }
    }

    if generated
        .iter()
        .any(|tokens| tokens.len() != GENERATED_TOKENS)
    {
        return Err(GeometricSpikeError::Source(
            "batched source smoke did not retain exactly 32 decisions per lane".to_owned(),
        ));
    }
    Ok(generated
        .into_iter()
        .enumerate()
        .map(|(index, tokens)| {
            transcript(
                tokenizer,
                &format!("G0-P{}", index + 1),
                FROZEN_PROMPTS[index],
                inputs[index].len(),
                tokens,
            )
        })
        .collect())
}

fn load_reused_controls(
    path: &Path,
    oracle: &HuggingFaceLlamaOracle,
    tokenizer: &HfBpeTokenizer,
    tokenizer_adapter: &TokenizerAdapter,
    chat_template: &str,
    source_revision: &str,
    inputs: &[Vec<u32>],
) -> Result<(Vec<RolloutTranscript>, ControlProvenance), GeometricSpikeError> {
    let bytes = std::fs::read(path)?;
    let report: GeometricSpikeReport = serde_json::from_slice(&bytes).map_err(|error| {
        GeometricSpikeError::Gate(format!(
            "retained control report {} is invalid: {error}",
            path.display()
        ))
    })?;
    let binding_matches = report.schema == REPORT_SCHEMA
        && report.issue == 950
        && report.source.repository == SOURCE_REPOSITORY
        && report.source.revision == source_revision
        && report.source.weights_cid == oracle.source_cid()
        && report.source.tokenizer == *tokenizer_adapter
        && report.source.chat_template == chat_template
        && report.source.backend == oracle.exact_backend_report()
        && report.decode.generated_tokens == GENERATED_TOKENS
        && report.decode.selection
            == "deterministic greedy argmax; lower token id wins an exact tie";
    let transcripts_match = report.control.len() == FROZEN_PROMPTS.len()
        && report
            .control
            .iter()
            .zip(FROZEN_PROMPTS)
            .zip(inputs)
            .enumerate()
            .all(|(index, ((rollout, prompt), input))| {
                let regenerated = transcript(
                    tokenizer,
                    &format!("G0-P{}", index + 1),
                    prompt,
                    input.len(),
                    rollout.generated_token_ids.clone(),
                );
                *rollout == regenerated
                    && rollout.generated_token_ids.len() == GENERATED_TOKENS
                    && rollout.utf8_decodable
                    && rollout.short_cycle_period.is_none()
            });
    let distinct_sequences = report
        .control
        .iter()
        .map(|rollout| rollout.generated_token_ids.as_slice())
        .collect::<HashSet<_>>()
        .len();
    if !binding_matches || !transcripts_match || distinct_sequences < 2 {
        return Err(GeometricSpikeError::Gate(format!(
            "retained control report {} does not satisfy the exact G0 source/decode binding",
            path.display()
        )));
    }
    Ok((
        report.control,
        ControlProvenance {
            execution:
                "exact controls revalidated from the retained negative-treatment repair input"
                    .to_owned(),
            reused_report_cid: Some(format!("blake3:{}", blake3::hash(&bytes).to_hex())),
        },
    ))
}

#[allow(clippy::too_many_arguments)]
fn treatment_rollout(
    oracle: &HuggingFaceLlamaOracle,
    tokenizer: &HfBpeTokenizer,
    mixer: &GeometricMixer,
    context: GeometryContext,
    prompt_id: &str,
    prompt: &str,
    _rendered: &str,
    input: &[u32],
    probe: bool,
) -> Result<(TreatmentTranscript, Option<ReachabilityEvidence>), GeometricSpikeError> {
    let capacity = input.len() + GENERATED_TOKENS;
    let memory_spans = context.memory_spans.len();
    let memory_tokens = context
        .memory_spans
        .iter()
        .map(|span| span.token_ids.len())
        .sum();
    let router_state_cid = context.provenance.router_state_cid.clone();
    let mut session = oracle
        .new_geometric_session(mixer.clone(), context, GeometryIntervention::Real, capacity)
        .map_err(|error| GeometricSpikeError::Decoder(error.to_string()))?;
    let mut logits = vec![0.0; oracle.cfg().vocab];
    for (position, &token) in input[..input.len() - 1].iter().enumerate() {
        oracle
            .step_geometric(&mut session, token as usize, position, &mut logits)
            .map_err(|error| GeometricSpikeError::Decoder(error.to_string()))?;
    }
    session.clear_traces();
    let mut permuted = probe.then(|| {
        let mut arm = session.clone();
        arm.set_intervention(GeometryIntervention::PermutedCoordinates);
        arm
    });
    let decision_position = input.len() - 1;
    let decision_token = input[decision_position];
    oracle
        .step_geometric(
            &mut session,
            decision_token as usize,
            decision_position,
            &mut logits,
        )
        .map_err(|error| GeometricSpikeError::Decoder(error.to_string()))?;
    let reachability = if let Some(permuted) = permuted.as_mut() {
        let mut permuted_logits = vec![0.0; oracle.cfg().vocab];
        oracle
            .step_geometric(
                permuted,
                decision_token as usize,
                decision_position,
                &mut permuted_logits,
            )
            .map_err(|error| GeometricSpikeError::Decoder(error.to_string()))?;
        Some(reachability_evidence(
            prompt_id,
            decision_position,
            session
                .traces()
                .last()
                .ok_or_else(|| GeometricSpikeError::Gate("real trace missing".to_owned()))?,
            permuted
                .traces()
                .last()
                .ok_or_else(|| GeometricSpikeError::Gate("permuted trace missing".to_owned()))?,
            &logits,
            &permuted_logits,
        ))
    } else {
        None
    };

    let mut generated = Vec::with_capacity(GENERATED_TOKENS);
    generated.push(greedy_token(&logits)?);
    for offset in 1..GENERATED_TOKENS {
        let position = input.len() - 1 + offset;
        let token = generated[offset - 1];
        oracle
            .step_geometric(&mut session, token as usize, position, &mut logits)
            .map_err(|error| GeometricSpikeError::Decoder(error.to_string()))?;
        generated.push(greedy_token(&logits)?);
    }
    let decision_trace = session.traces().to_vec();
    let rollout = transcript(tokenizer, prompt_id, prompt, input.len(), generated);
    Ok((
        TreatmentTranscript {
            rollout,
            memory_spans,
            memory_tokens,
            router_state_cid,
            geometry_position_states: session.context().position_states.len(),
            mixer_checkpoint_identity: session.checkpoint_identity(),
            decision_trace,
        },
        reachability,
    ))
}

fn geometry_context_from_router(
    router: &UorR4Router,
    identity: &str,
    tokenizer_cid: &str,
    adapter_identity: &str,
    source_cid: &str,
) -> Result<GeometryContext, GeometricSpikeError> {
    let memories = router
        .latest_tokenizer_bound_turns(
            identity,
            tokenizer_cid,
            adapter_identity,
            source_cid,
            MAX_MEMORY_SPANS,
            MAX_MEMORY_TOKENS,
        )
        .map_err(|error| GeometricSpikeError::Router(error.to_string()))?;
    let session_route_state = memories
        .last()
        .map(|memory| memory.r4_coordinates.map(|value| value as f32))
        .unwrap_or([0.0; 4]);
    let router_state_cid = router
        .tokenizer_bound_state_cid(identity)
        .ok_or_else(|| GeometricSpikeError::Router("bound state CID missing".to_owned()))?;
    let memory_spans = memories.into_iter().map(memory_span).collect::<Vec<_>>();
    GeometryContext::new(
        identity,
        tokenizer_cid,
        adapter_identity,
        session_route_state,
        memory_spans,
        GeometryProvenance {
            source_cid: source_cid.to_owned(),
            router_state_cid,
            memory_source: "uor-r4-router tokenizer-bound persistent turns".to_owned(),
        },
    )
    .map_err(|error| GeometricSpikeError::Decoder(error.to_string()))
}

fn memory_span(memory: TokenizerBoundMemorySpan) -> GeometryMemorySpan {
    GeometryMemorySpan {
        sequence: memory.sequence,
        role: memory.role,
        text: memory.text,
        token_ids: memory.token_ids,
        tokenizer_cid: memory.tokenizer_cid,
        adapter_identity: memory.adapter_identity,
        r4_coordinates: memory.r4_coordinates.map(|value| value as f32),
        provenance: memory.provenance,
    }
}

fn persist_and_reload(
    router: &UorR4Router,
    output: &Path,
    identity: &str,
    tokenizer_cid: &str,
    adapter_identity: &str,
    source_cid: &str,
) -> Result<PersistenceEvidence, GeometricSpikeError> {
    let before = router
        .tokenizer_bound_turns(identity, tokenizer_cid, adapter_identity, source_cid)
        .map_err(|error| GeometricSpikeError::Router(error.to_string()))?;
    let state = router.export_state();
    write_bytes(output, state.as_bytes())?;
    let exported_state_cid = format!("blake3:{}", blake3::hash(state.as_bytes()).to_hex());
    let mut restored = UorR4Router::new(0.5);
    if !restored.import_state_native(&state) {
        return Err(GeometricSpikeError::Router(
            "exported router state did not import".to_owned(),
        ));
    }
    let restored_state = restored.export_state();
    let restored_state_cid = format!(
        "blake3:{}",
        blake3::hash(restored_state.as_bytes()).to_hex()
    );
    let after = restored
        .tokenizer_bound_turns(identity, tokenizer_cid, adapter_identity, source_cid)
        .map_err(|error| GeometricSpikeError::Router(error.to_string()))?;
    let other = restored
        .tokenizer_bound_turns(OTHER_IDENTITY, tokenizer_cid, adapter_identity, source_cid)
        .map_err(|error| GeometricSpikeError::Router(error.to_string()))?;
    let tokenizer_cid_preserved = after
        .iter()
        .all(|memory| memory.tokenizer_cid == tokenizer_cid);
    let adapter_identity_preserved = after
        .iter()
        .all(|memory| memory.adapter_identity == adapter_identity);
    let ordered_tokens_preserved = before
        .iter()
        .map(|memory| (memory.sequence, memory.token_ids.as_slice()))
        .eq(after
            .iter()
            .map(|memory| (memory.sequence, memory.token_ids.as_slice())));
    let same_identity_exact = before == after;
    let pass = same_identity_exact
        && other.is_empty()
        && tokenizer_cid_preserved
        && adapter_identity_preserved
        && ordered_tokens_preserved;
    Ok(PersistenceEvidence {
        committed_turns: before.len(),
        exported_state_cid,
        restored_state_cid,
        same_identity_turns: after.len(),
        same_identity_exact,
        other_identity_turns: other.len(),
        tokenizer_cid_preserved,
        adapter_identity_preserved,
        ordered_tokens_preserved,
        verdict: if pass { "PASS" } else { "FAIL" }.to_owned(),
    })
}

fn evaluate_runtime_gates(
    control: &[RolloutTranscript],
    treatment: &[TreatmentTranscript],
    disabled: &DisabledControlEvidence,
    reachability: &ReachabilityEvidence,
    persistence: &PersistenceEvidence,
) -> RuntimeGates {
    let five_control_rollouts = control.len() == FROZEN_PROMPTS.len();
    let all_control_rollouts_exactly_32_tokens = control
        .iter()
        .all(|rollout| rollout.generated_token_ids.len() == GENERATED_TOKENS);
    let no_short_cycles = control
        .iter()
        .all(|rollout| rollout.short_cycle_period.is_none())
        && treatment
            .iter()
            .all(|rollout| rollout.rollout.short_cycle_period.is_none());
    let distinct_control_sequences = control
        .iter()
        .map(|rollout| rollout.generated_token_ids.clone())
        .collect::<HashSet<_>>()
        .len();
    let one_treatment_rollout = treatment.len() == 1;
    let every_treatment_decision_executed_mixer = treatment.iter().all(|rollout| {
        rollout.decision_trace.len() == GENERATED_TOKENS
            && rollout.geometry_position_states
                == rollout.rollout.input_tokens + GENERATED_TOKENS - 1
            && rollout.memory_spans <= MAX_MEMORY_SPANS
            && rollout.memory_tokens <= MAX_MEMORY_TOKENS
            && rollout
                .decision_trace
                .iter()
                .enumerate()
                .all(|(offset, trace)| {
                    trace.layer == TARGET_LAYER
                        && trace.position == rollout.rollout.input_tokens - 1 + offset
                        && trace.intervention == GeometryIntervention::Real
                        && trace.source_attention_calls == 0
                        && !trace.dense_full_prefix_qk
                        && trace.prefix_candidates == trace.position + 1
                        && trace.memory_candidates == rollout.memory_spans
                        && trace.support_budget == DEFAULT_SUPPORT_BUDGET
                        && !trace.selected_support.is_empty()
                        && trace.selected_support.len() <= trace.support_budget
                })
    });
    let all_treatments_decodable = treatment
        .iter()
        .all(|rollout| rollout.rollout.utf8_decodable && !rollout.rollout.raw_decoded.is_empty());
    let disabled_reproduces_control = disabled.logits_bit_exact
        && disabled.generated_tokens_equal
        && disabled.disabled_trace_entries == 0;
    let structural_reachability = reachability.verdict == "PASS";
    let persistence_round_trip = persistence.verdict == "PASS";
    let pass = five_control_rollouts
        && all_control_rollouts_exactly_32_tokens
        && no_short_cycles
        && distinct_control_sequences >= 2
        && one_treatment_rollout
        && every_treatment_decision_executed_mixer
        && all_treatments_decodable
        && disabled_reproduces_control
        && structural_reachability
        && persistence_round_trip;
    RuntimeGates {
        five_control_rollouts,
        all_control_rollouts_exactly_32_tokens,
        no_short_cycles,
        distinct_control_sequences,
        one_treatment_rollout,
        every_treatment_decision_executed_mixer,
        all_treatments_decodable,
        disabled_reproduces_control,
        structural_reachability,
        persistence_round_trip,
        verdict: if pass { "PASS" } else { "FAIL" }.to_owned(),
    }
}

fn reachability_evidence(
    prompt_id: &str,
    position: usize,
    real: &GeometricOperatorTrace,
    permuted: &GeometricOperatorTrace,
    real_logits: &[f32],
    permuted_logits: &[f32],
) -> ReachabilityEvidence {
    let support_changed = real.support_cid != permuted.support_cid;
    let mut changed_logits = 0usize;
    let mut logit_linf_delta = 0.0f32;
    for (&left, &right) in real_logits.iter().zip(permuted_logits) {
        if left.to_bits() != right.to_bits() {
            changed_logits += 1;
        }
        logit_linf_delta = logit_linf_delta.max((left - right).abs());
    }
    ReachabilityEvidence {
        prompt_id: prompt_id.to_owned(),
        compared_position: position,
        real_support_cid: real.support_cid.clone(),
        permuted_support_cid: permuted.support_cid.clone(),
        support_changed,
        changed_logits,
        logit_linf_delta,
        verdict: if support_changed || changed_logits > 0 {
            "PASS"
        } else {
            "FAIL"
        }
        .to_owned(),
    }
}

fn transcript(
    tokenizer: &HfBpeTokenizer,
    prompt_id: &str,
    prompt: &str,
    input_tokens: usize,
    generated_token_ids: Vec<u32>,
) -> RolloutTranscript {
    let eos = 2u32;
    let first_eos_offset = generated_token_ids.iter().position(|&token| token == eos);
    let response_end = first_eos_offset.unwrap_or(generated_token_ids.len());
    let response_ids = &generated_token_ids[..response_end];
    let raw_bytes = tokenizer.decode_bytes(&generated_token_ids);
    let response_bytes = tokenizer.decode_bytes(response_ids);
    RolloutTranscript {
        prompt_id: prompt_id.to_owned(),
        prompt: prompt.to_owned(),
        input_tokens,
        raw_decoded: String::from_utf8_lossy(&raw_bytes).into_owned(),
        response_text: String::from_utf8_lossy(&response_bytes).trim().to_owned(),
        first_eos_offset,
        utf8_decodable: String::from_utf8(raw_bytes).is_ok()
            && String::from_utf8(response_bytes).is_ok(),
        short_cycle_period: short_cycle_period(&generated_token_ids),
        generated_token_ids,
    }
}

pub fn short_cycle_period(tokens: &[u32]) -> Option<usize> {
    for period in 1..=4 {
        let span = period * 3;
        if tokens.len() < span {
            continue;
        }
        let tail = &tokens[tokens.len() - span..];
        if tail[..period] == tail[period..period * 2] && tail[..period] == tail[period * 2..] {
            return Some(period);
        }
    }
    None
}

fn greedy_token(logits: &[f32]) -> Result<u32, GeometricSpikeError> {
    let Some((&first, rest)) = logits.split_first() else {
        return Err(GeometricSpikeError::Source("empty logits".to_owned()));
    };
    if !first.is_finite() || rest.iter().any(|value| !value.is_finite()) {
        return Err(GeometricSpikeError::Source(
            "non-finite source logits".to_owned(),
        ));
    }
    let mut best_token = 0usize;
    let mut best_logit = first;
    for (offset, &logit) in rest.iter().enumerate() {
        let token = offset + 1;
        if logit > best_logit {
            best_logit = logit;
            best_token = token;
        }
    }
    u32::try_from(best_token)
        .map_err(|_| GeometricSpikeError::Source("token id exceeds u32".to_owned()))
}

fn same_f32_bits(left: &[f32], right: &[f32]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| left.to_bits() == right.to_bits())
}

fn render_chat_prompt(prompt: &str) -> String {
    format!(
        "<|im_start|>system\nYou are a helpful AI assistant named SmolLM, trained by Hugging Face<|im_end|>\n<|im_start|>user\n{prompt}<|im_end|>\n<|im_start|>assistant\n"
    )
}

fn validate_source(source: &Path, revision: &str) -> Result<(), GeometricSpikeError> {
    if revision != PINNED_SOURCE_REVISION {
        return Err(GeometricSpikeError::InvalidSource(format!(
            "source revision {revision} != pinned {PINNED_SOURCE_REVISION}"
        )));
    }
    if revision.len() != 40
        || !revision
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err(GeometricSpikeError::InvalidSource(
            "source revision must be a full 40-character hexadecimal commit".to_owned(),
        ));
    }
    for file in ["config.json", "tokenizer.json", "tokenizer_config.json"] {
        if !source.join(file).is_file() {
            return Err(GeometricSpikeError::InvalidSource(format!(
                "{} is missing {file}",
                source.display()
            )));
        }
    }
    let has_weights = source.join("model.safetensors").is_file()
        || source.join("model.safetensors.index.json").is_file();
    if !has_weights {
        return Err(GeometricSpikeError::InvalidSource(format!(
            "{} has no Safetensors weights",
            source.display()
        )));
    }
    let tree = source
        .join(".cache/huggingface/trees")
        .join(format!("{revision}.json"));
    if !tree.is_file() {
        return Err(GeometricSpikeError::InvalidSource(format!(
            "{} does not bind revision {revision}",
            source.display()
        )));
    }
    Ok(())
}

fn read_chat_template(source: &Path) -> Result<String, GeometricSpikeError> {
    let bytes = std::fs::read(source.join("tokenizer_config.json"))?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|error| {
        GeometricSpikeError::Tokenizer(format!("tokenizer_config.json: {error}"))
    })?;
    value
        .get("chat_template")
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            GeometricSpikeError::Tokenizer(
                "tokenizer_config.json has no string chat_template".to_owned(),
            )
        })
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<(), GeometricSpikeError> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| GeometricSpikeError::Io(io::Error::other(error)))?;
    write_bytes(path, &bytes)
}

fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), GeometricSpikeError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_chat_rendering_matches_the_template_specialization() {
        let rendered = render_chat_prompt("Hello");
        assert_eq!(
            rendered,
            "<|im_start|>system\nYou are a helpful AI assistant named SmolLM, trained by Hugging Face<|im_end|>\n<|im_start|>user\nHello<|im_end|>\n<|im_start|>assistant\n"
        );
        assert!(PINNED_CHAT_TEMPLATE.contains("add_generation_prompt"));
    }

    #[test]
    fn cycle_detector_rejects_only_repeated_period_one_through_four_tails() {
        assert_eq!(short_cycle_period(&[1, 2, 3, 4, 4, 4]), Some(1));
        assert_eq!(short_cycle_period(&[9, 1, 2, 1, 2, 1, 2]), Some(2));
        assert_eq!(short_cycle_period(&[1, 2, 3, 4, 5, 6]), None);
    }
}
