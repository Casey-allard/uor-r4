//! Explicit historical R1d, nearest-child or learned-code child observation:
//! FF -> packed QF -> packed QQ -> integer.
//! No calibration, optimization, new data selection or quality acceptance gate.
//! Identical saved inputs localize numerical changes; actual dialogue retains
//! each form's own generated history. Only the existing open requests are used
//! for generation; historical data is also hashed for provenance.
#![forbid(unsafe_code)]

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
    time::Instant,
};

use candle_core::Device;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_core::{report_output, transformerless::hf_bpe::HfBpeTokenizer};
use uor_r4_integer::{
    config::{QuantizationPreparation, ServingProfile},
    generation::{ConversationRequest, Stop, TurnBoundary, TurnClosure},
    Bundle, IntegerStep, SamplePolicy, PROBABILITY_TOTAL,
};
use uor_r4_tokenizer::{dialogue::DialogueProtocol, ByteBpeTokenizer};
use uor_r4_training::{
    dialogue_artifact::LegacyDialogueArtifact,
    dialogue_child_artifact::DialogueChildArtifact,
    joint_admission::AdmissionPolicy,
    joint_evaluation::{self, JointGenerationStop, PROBABILITY_SUM_TOLERANCE},
    joint_model::{JointModel, JointStep, PrecisionMode, ReadMode},
    reference_eval::short_cycle_period,
    sha256_file,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const VOCAB: usize = 4096;
const WIDTH: usize = 576;
const CONTEXT: usize = 256;
const CAP: usize = 32;
const PARENT_SHA: &str = "95e3fbb06cb39b4354bd40c722873dac088d47554707777b291e588922a1a822";
const CORPUS_SHA: &str = "a66d52473cac24b28cc09a751ede7b680c40e7b56218e9074e246cf4fd1a99a5";
const TOKENIZER_SHA: &str = "d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89";
const REQUESTS_SHA: &str = "81268b51ef98d8d8e525a40e572538249ef845e321571ee55dec30fb8ef75484";
const REPLAY_SHA: &str = "4ea4df2614ebcf0eb31da944a3e7c63c8edec28effcee936984112f5c88455d8";
const CHILD_RESPONSES_SHA: &str =
    "bd3a6ce865ff750d66a5a2a10222ac8f3749a7c0482381cc6edd389b6080b31f";
const CHILD_RESULT_SHA: &str = "fb49d94c6d3a10e72457cfa12b81c3b3a06ff0986c3f111db04ecf01173facb6";
const CHILD_MODEL_SHA: &str = "98aca5ab14a9edab58dcd2d74d71e74d3904c27e1ce3c126fb66b670cc2baa1c";
const CHILD_FINGERPRINT: &str = "1eef006a29767f06fd07d29e03f4e9cdf254a6a8a0562c5c8a019b8c0e6bb1d6";
const PROTOCOL_ID: &str = "blake3:0099a613c8fcffc78210ed7b7387841917472d976f307a33526c8424ecf5d327";
const FORMS: [&str; 4] = ["FF", "QF", "QQ", "integer"];
const PAIRS: [&str; 3] = ["FF_QF", "QF_QQ", "QQ_integer"];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ObservationMode {
    HistoricalR1d,
    CompletePrefixChild,
    RoundedCompletePrefixChild,
}
impl ObservationMode {
    fn is_child(self) -> bool {
        self != Self::HistoricalR1d
    }
    fn conversion_schema(self) -> &'static str {
        match self {
            Self::HistoricalR1d => "uor-r4.native-dialogue576-conversion/1",
            Self::CompletePrefixChild => "uor-r4.native-dialogue576-child-conversion/1",
            Self::RoundedCompletePrefixChild => "uor-r4.native-dialogue576-child-rounding/1",
        }
    }
    fn preparation(self) -> QuantizationPreparation {
        match self {
            Self::RoundedCompletePrefixChild => QuantizationPreparation::CalibratedForRounding,
            _ => QuantizationPreparation::CalibratedForExport,
        }
    }
    fn reference_sha(self) -> &'static str {
        match self {
            Self::HistoricalR1d => REPLAY_SHA,
            Self::CompletePrefixChild | Self::RoundedCompletePrefixChild => CHILD_RESPONSES_SHA,
        }
    }
    /// (turns, prompt occurrences, selections, consumed trace positions).
    fn reference_counts(self) -> (usize, usize, usize, usize) {
        match self {
            Self::HistoricalR1d => (58, 2623, 1766, 4331),
            Self::CompletePrefixChild | Self::RoundedCompletePrefixChild => (58, 2464, 1508, 3914),
        }
    }
    fn model_step(self) -> usize {
        match self {
            Self::HistoricalR1d => 2237,
            Self::CompletePrefixChild | Self::RoundedCompletePrefixChild => 1024,
        }
    }
    fn input_scope(self) -> &'static str {
        match self {
            Self::HistoricalR1d => "Historical R1d only; no current-study child. Legacy import hashes train AND heldout token/mask files for provenance; no corpus or heldout scoring, panel selection, calibration or optimizer updates.",
            Self::CompletePrefixChild => "Selected continuous complete-prefix child and its own sealed saved responses, not the historical R1d output. Typed import binds immediate child checkpoint and historical ancestry separately. Data identities are provenance only; no corpus or heldout scoring, calibration, alpha learning or optimizer/model updates.",
            Self::RoundedCompletePrefixChild => "Selected continuous complete-prefix child remains FF and supplies the identical saved reference. QF/QQ/integer use separately bound learned legal codes on its frozen grids; prior alpha updates are distinct from the child model clock and from this forward-only observer. No corpus or heldout scoring, calibration, backward or optimizer updates occur here.",
        }
    }
}

fn parse_args(mut args: Vec<PathBuf>) -> Result<(ObservationMode, Vec<PathBuf>)> {
    let mode = if args.first().is_some_and(|p| p.as_os_str() == "--child") {
        args.remove(0);
        ObservationMode::CompletePrefixChild
    } else if args
        .first()
        .is_some_and(|p| p.as_os_str() == "--child-rounded")
    {
        args.remove(0);
        ObservationMode::RoundedCompletePrefixChild
    } else {
        ObservationMode::HistoricalR1d
    };
    ensure(
        args.len() == 8 && !args[0].as_os_str().to_string_lossy().starts_with("--"),
        "usage: dialogue-integer-observe [--child | --child-rounded] PARENT_REPORT_ROOT CORPUS_MANIFEST TOKENIZER PACKED_ROOT BUNDLE_ROOT REQUESTS RECORDED_RESPONSES NEW_REPORT_ROOT; omitted mode retains historical R1d",
    )?;
    Ok((mode, args))
}

enum ParentArtifact {
    Historical(LegacyDialogueArtifact),
    Child(DialogueChildArtifact),
}
impl ParentArtifact {
    fn model(&self) -> &JointModel {
        match self {
            Self::Historical(p) => p.model(),
            Self::Child(p) => p.model(),
        }
    }
    fn tokenizer(&self) -> &ByteBpeTokenizer {
        match self {
            Self::Historical(p) => p.tokenizer(),
            Self::Child(p) => p.tokenizer(),
        }
    }
    fn protocol(&self) -> &DialogueProtocol {
        match self {
            Self::Historical(p) => p.protocol(),
            Self::Child(p) => p.protocol(),
        }
    }
    fn provenance(&self) -> Result<Value> {
        Ok(match self {
            Self::Historical(p) => serde_json::to_value(p.provenance())?,
            Self::Child(p) => serde_json::to_value(p.provenance())?,
        })
    }
}

fn ensure(condition: bool, message: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(io::Error::other(message).into())
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Request {
    id: String,
    category: String,
    user_turns: Vec<String>,
}
#[derive(Deserialize)]
struct SavedReplay {
    artifact: Value,
    protocol: DialogueProtocol,
    protocol_identity: String,
    requests: usize,
    responses: usize,
    rows: Vec<SavedRequest>,
    generation_incremental_step_calls: usize,
    generated_selections: usize,
}
#[derive(Deserialize)]
struct ChildResponses {
    protocol: DialogueProtocol,
    requests_sha256: String,
    rows: Vec<SavedRequest>,
}
#[derive(Deserialize)]
struct SavedRequest {
    id: String,
    category: String,
    turns: Vec<SavedTurn>,
}
#[derive(Deserialize)]
struct SavedTurn {
    turn: usize,
    user: String,
    // Historical replay records the suffix separately. The child packet binds
    // the complete prompt instead; that exact ID sequence is checked below.
    appended_user_prefix_ids: Option<Vec<u32>>,
    generation: SavedGeneration,
    model_eos: bool,
    caller_eos_inserted_before_next_request: bool,
    retained_history_ids: Vec<u32>,
}
#[derive(Deserialize)]
struct SavedGeneration {
    prompt_token_ids: Vec<u32>,
    generated_token_ids: Vec<u32>,
    raw_decoded_bytes: Vec<u8>,
    stop: Stop,
    incremental_step_calls: usize,
    max_new_tokens: usize,
    context_capacity: usize,
    mode: ReadMode,
    seed: Option<u64>,
    decisions: Vec<SavedDecision>,
}
#[derive(Deserialize)]
struct SavedDecision {
    decision: usize,
    input_positions: usize,
    selected_token: u32,
}
#[derive(Serialize)]
struct Trace {
    id: String,
    category: String,
    turn: usize,
    prompt: Vec<u32>,
    generated: Vec<u32>,
    inputs: Vec<u32>,
    targets: Vec<u32>,
    assistant_start: usize,
}

/// The final generated ID (including EOS) is a target, never an input here.
fn trace_tokens(prompt: &[u32], generated: &[u32]) -> Result<(Vec<u32>, Vec<u32>, usize)> {
    ensure(
        prompt.first() == Some(&0)
            && !generated.is_empty()
            && generated.len() <= CAP
            && prompt.len() + CAP <= CONTEXT
            && prompt
                .iter()
                .chain(generated)
                .all(|&id| (id as usize) < VOCAB),
        "saved trace BOS/vocabulary/context/output length differs",
    )?;
    let mut inputs = prompt.to_vec();
    inputs.extend_from_slice(&generated[..generated.len() - 1]);
    let mut targets = prompt[1..].to_vec();
    targets.extend_from_slice(generated);
    Ok((inputs, targets, prompt.len() - 1))
}

fn validate_stop(ids: &[u32], stop: Stop) -> Result<()> {
    ensure(
        !ids.is_empty() && ids.len() <= CAP && ids.iter().all(|&id| (id as usize) < VOCAB),
        "invalid generated ID inventory",
    )?;
    for end in 1..=ids.len() {
        let expected = if ids[end - 1] == 1 {
            Some(Stop::Eos)
        } else if let Some(period) = short_cycle_period(&ids[..end]) {
            Some(Stop::ShortCycle { period })
        } else if end == CAP {
            Some(Stop::MaximumNewTokens)
        } else {
            None
        };
        if let Some(expected) = expected {
            return ensure(
                end == ids.len() && expected == stop,
                "generation stop/controller differs",
            );
        }
    }
    Err(io::Error::other("generation ends without the fixed controller stop").into())
}

fn close_history(history: &mut Vec<u32>, ids: &[u32], stop: Stop, has_next: bool) -> Result<bool> {
    validate_stop(ids, stop)?;
    history.extend_from_slice(ids);
    let caller_eos = has_next && stop != Stop::Eos;
    if caller_eos {
        history.push(1);
    }
    Ok(caller_eos)
}

fn prepare_traces(
    requests: &[Request],
    saved: &SavedReplay,
    codec: &ByteBpeTokenizer,
    protocol: &DialogueProtocol,
    mode: ObservationMode,
) -> Result<Vec<Trace>> {
    let unique: BTreeSet<_> = requests.iter().map(|r| &r.id).collect();
    ensure(
        requests.len() == 38
            && unique.len() == 38
            && saved.requests == 38
            && saved.responses == 58
            && saved.rows.len() == requests.len()
            && requests.iter().map(|r| r.user_turns.len()).sum::<usize>() == 58,
        "fixed request/turn population differs",
    )?;
    let encoder = protocol.bind(codec)?;
    let mut traces = Vec::new();
    let (mut prompts, mut selected, mut calls) = (0, 0, 0);
    for (request, row) in requests.iter().zip(&saved.rows) {
        ensure(
            request.id.starts_with("dev-")
                && [1, 3].contains(&request.user_turns.len())
                && request.id == row.id
                && request.category == row.category
                && request.user_turns.len() == row.turns.len(),
            "saved request identity/order differs",
        )?;
        let mut history = vec![0];
        let mut worst_history = 1usize;
        for (index, (user, turn)) in request.user_turns.iter().zip(&row.turns).enumerate() {
            let suffix = encoder.encode_user_prefix(user, index != 0);
            ensure(
                !user.trim().is_empty()
                    && suffix.emitted_turns == 1
                    && suffix.special_token_occurrences == 0
                    && turn.turn == index + 1
                    && turn.user == *user
                    && match &turn.appended_user_prefix_ids {
                        Some(recorded) => *recorded == suffix.tokens,
                        None => mode.is_child(),
                    },
                "saved user framing differs",
            )?;
            worst_history += suffix.tokens.len() + CAP;
            ensure(
                worst_history <= CONTEXT,
                "worst-case generated history exceeds context",
            )?;
            if index + 1 < row.turns.len() {
                worst_history += 1;
            }
            history.extend_from_slice(&suffix.tokens);
            let g = &turn.generation;
            let (inputs, targets, assistant_start) =
                trace_tokens(&history, &g.generated_token_ids)?;
            ensure(
                g.prompt_token_ids == history
                    && g.max_new_tokens == CAP
                    && g.context_capacity == CONTEXT
                    && g.mode == ReadMode::Enabled
                    && g.seed.is_none()
                    && g.incremental_step_calls == inputs.len()
                    && g.decisions.len() == g.generated_token_ids.len()
                    && g.decisions.iter().enumerate().all(|(i, d)| {
                        d.decision == i
                            && d.input_positions == history.len() + i
                            && d.selected_token == g.generated_token_ids[i]
                    })
                    && codec.decode_bytes(&g.generated_token_ids) == g.raw_decoded_bytes,
                "recorded generation trajectory/controller/decoded bytes differ",
            )?;
            prompts += history.len();
            selected += g.generated_token_ids.len();
            calls += inputs.len();
            traces.push(Trace {
                id: request.id.clone(),
                category: request.category.clone(),
                turn: index + 1,
                prompt: history.clone(),
                generated: g.generated_token_ids.clone(),
                inputs,
                targets,
                assistant_start,
            });
            let closed = close_history(
                &mut history,
                &g.generated_token_ids,
                g.stop,
                index + 1 < row.turns.len(),
            )?;
            ensure(
                closed == turn.caller_eos_inserted_before_next_request
                    && turn.model_eos == (g.stop == Stop::Eos)
                    && history == turn.retained_history_ids,
                "saved generated history or caller EOS differs",
            )?;
        }
    }
    ensure(
        (traces.len(), prompts, selected, calls) == mode.reference_counts()
            && saved.generation_incremental_step_calls == calls
            && saved.generated_selections == selected,
        "fixed common-trace denominators differ",
    )?;
    Ok(traces)
}

fn digest_bytes<I, B>(values: I) -> String
where
    I: IntoIterator<Item = B>,
    B: AsRef<[u8]>,
{
    let mut h = Sha256::new();
    for value in values {
        h.update(value);
    }
    hex::encode(h.finalize())
}
fn argmax(values: &[f64]) -> usize {
    values
        .iter()
        .enumerate()
        .fold((0, f64::NEG_INFINITY), |best, (i, &v)| {
            if v > best.1 {
                (i, v)
            } else {
                best
            }
        })
        .0
}

struct Snapshot {
    probabilities: Vec<f64>,
    state: Vec<f64>,
    no_read: f64,
    copy_gate: f64,
    probability_sha: String,
    state_sha: String,
    read_sha: String,
}
impl Snapshot {
    fn from_float(step: JointStep, position: usize) -> Result<Self> {
        ensure(
            step.probabilities.dims() == [1, VOCAB]
                && step.state.dims() == [1, WIDTH]
                && step.no_read_mass.dims() == [1]
                && step.copy_gate.dims() == [1]
                && step.read_masses.dims() == [1, position]
                && step.read_occurrences == (0..position).collect::<Vec<_>>()
                && step.written_occurrence == position
                && !step.probabilities.track_op()
                && !step.state.track_op(),
            "F32 trace shape/causal occurrence/gradient contract differs",
        )?;
        let p = step.probabilities.flatten_all()?.to_vec1::<f32>()?;
        let state = step.state.flatten_all()?.to_vec1::<f32>()?;
        let read = step.read_masses.flatten_all()?.to_vec1::<f32>()?;
        let no_read = f64::from(step.no_read_mass.to_vec1::<f32>()?[0]);
        let copy_gate = f64::from(step.copy_gate.to_vec1::<f32>()?[0]);
        let mass = |v: f64| v.is_finite() && (0.0..=1.0 + PROBABILITY_SUM_TOLERANCE).contains(&v);
        ensure(
            p.iter().all(|&v| mass(f64::from(v)))
                && (p.iter().map(|&v| f64::from(v)).sum::<f64>() - 1.0).abs()
                    <= PROBABILITY_SUM_TOLERANCE
                && state.iter().all(|v| v.is_finite())
                && mass(no_read)
                && mass(copy_gate)
                && read.iter().all(|&v| mass(f64::from(v)))
                && (read.iter().map(|&v| f64::from(v)).sum::<f64>() + no_read - 1.0).abs()
                    <= PROBABILITY_SUM_TOLERANCE,
            "invalid F32 trace probability/state/read values (existing evaluator tolerance)",
        )?;
        Ok(Self {
            probability_sha: digest_bytes(p.iter().map(|v| v.to_le_bytes())),
            state_sha: digest_bytes(state.iter().map(|v| v.to_le_bytes())),
            read_sha: digest_bytes(read.iter().map(|v| v.to_le_bytes())),
            probabilities: p.into_iter().map(f64::from).collect(),
            state: state.into_iter().map(f64::from).collect(),
            no_read,
            copy_gate,
        })
    }
    fn from_integer(step: IntegerStep, position: usize) -> Result<Self> {
        let total = u128::from(PROBABILITY_TOTAL);
        ensure(
            step.probabilities.len() == VOCAB
                && step.state.len() == WIDTH
                && step.read_masses.len() == position
                && step
                    .probabilities
                    .iter()
                    .map(|&x| u128::from(x))
                    .sum::<u128>()
                    == total
                && step.probabilities.iter().all(|&x| x <= PROBABILITY_TOTAL)
                && step
                    .read_masses
                    .iter()
                    .map(|&x| u128::from(x))
                    .sum::<u128>()
                    + u128::from(step.no_read_mass)
                    == total
                && step.read_masses.iter().all(|&x| x <= PROBABILITY_TOTAL)
                && step.no_read_mass <= PROBABILITY_TOTAL
                && (0..=32768).contains(&step.copy_gate)
                && step.state.iter().all(|v| (-32767..=32767).contains(v)),
            "integer trace dimension, exact Q48 total or code domain differs",
        )?;
        Ok(Self {
            probability_sha: digest_bytes(step.probabilities.iter().map(|v| v.to_le_bytes())),
            state_sha: digest_bytes(step.state.iter().map(|v| v.to_le_bytes())),
            read_sha: digest_bytes(step.read_masses.iter().map(|v| v.to_le_bytes())),
            probabilities: step
                .probabilities
                .into_iter()
                .map(|v| v as f64 / PROBABILITY_TOTAL as f64)
                .collect(),
            state: step
                .state
                .into_iter()
                .map(|v| f64::from(v) / 2048.0)
                .collect(),
            no_read: step.no_read_mass as f64 / PROBABILITY_TOTAL as f64,
            copy_gate: f64::from(step.copy_gate) / 32768.0,
        })
    }
    fn scalars(&self, target: usize) -> Value {
        let p = self.probabilities[target];
        let sum = self.probabilities.iter().sum::<f64>();
        json!({"greedy_token":argmax(&self.probabilities),"recorded_target_probability":p,
            "recorded_target_nll_nats":if p>0.0 {Some(-p.ln())} else {None},
            "zero_target_probability":p==0.0,"probability_sum":sum,"probability_sum_minus_one":sum-1.0,
            "no_read_mass":self.no_read,"copy_gate":self.copy_gate,
            "probabilities_sha256_native":self.probability_sha,"state_sha256_native":self.state_sha,
            "read_masses_sha256_native":self.read_sha})
    }
}

#[derive(Default, Serialize)]
struct Targets {
    count: usize,
    zero_probability_targets: usize,
    finite_nll_sum: f64,
}
impl Targets {
    fn add(&mut self, p: f64) {
        self.count += 1;
        if p == 0.0 {
            self.zero_probability_targets += 1;
        } else {
            self.finite_nll_sum -= p.ln();
        }
    }
    fn value(&self) -> Value {
        let finite_targets = self.count - self.zero_probability_targets;
        json!({"targets":self.count,"zero_probability_targets":self.zero_probability_targets,
            "finite_targets":finite_targets,"finite_nll_sum":self.finite_nll_sum,
            "finite_target_mean_nll_nats":if finite_targets>0 {Some(self.finite_nll_sum/finite_targets as f64)} else {None},
            "mean_nll_nats":if self.count>0 && self.zero_probability_targets==0 {Some(self.finite_nll_sum/self.count as f64)} else {None},
            "nll_status":if self.count==0 {"EMPTY"} else if self.zero_probability_targets>0 {"INFINITE_ZERO_PROBABILITY"} else {"FINITE"}})
    }
}
#[derive(Default)]
struct FormTotals {
    assistant: Targets,
    other: Targets,
    no_read_sum: f64,
    gate_sum: f64,
    calls: usize,
    model_ns: u128,
}
impl FormTotals {
    fn add(&mut self, s: &Snapshot, target: usize, assistant: bool, ns: u128) {
        if assistant {
            self.assistant.add(s.probabilities[target]);
        } else {
            self.other.add(s.probabilities[target]);
        }
        self.no_read_sum += s.no_read;
        self.gate_sum += s.copy_gate;
        self.calls += 1;
        self.model_ns += ns;
    }
    fn value(&self) -> Value {
        json!({"model_step_calls":self.calls,"model_step_nanoseconds":self.model_ns,
            "assistant_saved_targets":self.assistant.value(),"other_saved_targets":self.other.value(),
            "mean_no_read_mass":self.no_read_sum/self.calls.max(1) as f64,
            "mean_copy_gate":self.gate_sum/self.calls.max(1) as f64})
    }
}
#[derive(Serialize)]
struct Difference {
    total_variation: f64,
    max_probability_delta: f64,
    max_state_delta: f64,
    no_read_delta: f64,
    copy_gate_delta: f64,
    greedy_disagrees: bool,
}
fn difference(a: &Snapshot, b: &Snapshot) -> Difference {
    let mut d = Difference {
        total_variation: 0.0,
        max_probability_delta: 0.0,
        max_state_delta: 0.0,
        no_read_delta: b.no_read - a.no_read,
        copy_gate_delta: b.copy_gate - a.copy_gate,
        greedy_disagrees: argmax(&a.probabilities) != argmax(&b.probabilities),
    };
    for (&a, &b) in a.probabilities.iter().zip(&b.probabilities) {
        let delta = (a - b).abs();
        d.total_variation += delta / 2.0;
        d.max_probability_delta = d.max_probability_delta.max(delta);
    }
    for (&a, &b) in a.state.iter().zip(&b.state) {
        d.max_state_delta = d.max_state_delta.max((a - b).abs());
    }
    d
}
#[derive(Default)]
struct PairTotals {
    positions: usize,
    tv_sum: f64,
    max_tv: f64,
    max_probability_delta: f64,
    max_state_delta: f64,
    max_no_read_delta: f64,
    max_copy_gate_delta: f64,
    greedy_disagreements: usize,
    first_numerical_difference: Option<Value>,
    first_greedy_disagreement: Option<Value>,
}
impl PairTotals {
    fn add(&mut self, d: &Difference, trace: &Trace, position: usize) {
        self.positions += 1;
        self.tv_sum += d.total_variation;
        self.max_tv = self.max_tv.max(d.total_variation);
        self.max_probability_delta = self.max_probability_delta.max(d.max_probability_delta);
        self.max_state_delta = self.max_state_delta.max(d.max_state_delta);
        self.max_no_read_delta = self.max_no_read_delta.max(d.no_read_delta.abs());
        self.max_copy_gate_delta = self.max_copy_gate_delta.max(d.copy_gate_delta.abs());
        let at = || json!({"id":trace.id,"turn":trace.turn,"position":position});
        if self.first_numerical_difference.is_none()
            && (d.max_probability_delta != 0.0
                || d.max_state_delta != 0.0
                || d.no_read_delta != 0.0
                || d.copy_gate_delta != 0.0)
        {
            self.first_numerical_difference = Some(at());
        }
        if d.greedy_disagrees {
            self.greedy_disagreements += 1;
            if self.first_greedy_disagreement.is_none() {
                self.first_greedy_disagreement = Some(at());
            }
        }
    }
    fn value(&self) -> Value {
        json!({"positions":self.positions,"mean_total_variation":self.tv_sum/self.positions.max(1) as f64,
            "max_total_variation":self.max_tv,"max_probability_delta":self.max_probability_delta,
            "max_state_delta":self.max_state_delta,"max_absolute_no_read_delta":self.max_no_read_delta,
            "max_absolute_copy_gate_delta":self.max_copy_gate_delta,"greedy_disagreements":self.greedy_disagreements,
            "first_exact_numerical_difference":self.first_numerical_difference,"first_greedy_disagreement":self.first_greedy_disagreement})
    }
}

fn trace_all(
    models: [&JointModel; 3],
    bundle: &Bundle,
    traces: &[Trace],
    out: &Path,
) -> Result<Value> {
    let started = Instant::now();
    let mut stream = File::create_new(out.join("trace-positions.jsonl"))?;
    let mut totals: [FormTotals; 4] = std::array::from_fn(|_| FormTotals::default());
    let mut pairs: [PairTotals; 3] = std::array::from_fn(|_| PairTotals::default());
    let mut turns = Vec::new();
    for trace in traces {
        let mut sessions = [
            models[0].new_session(1)?,
            models[1].new_session(1)?,
            models[2].new_session(1)?,
        ];
        let mut integer = bundle.model().new_session();
        let mut local: [FormTotals; 4] = std::array::from_fn(|_| FormTotals::default());
        let mut local_pairs: [PairTotals; 3] = std::array::from_fn(|_| PairTotals::default());
        for (position, (&input, &target)) in trace.inputs.iter().zip(&trace.targets).enumerate() {
            let mut snapshots = Vec::with_capacity(4);
            let mut nanos = Vec::with_capacity(4);
            for (model, session) in models.iter().zip(&mut sessions) {
                let clock = Instant::now();
                let step = model.step(session, &[input], ReadMode::Enabled)?;
                nanos.push(clock.elapsed().as_nanos());
                snapshots.push(Snapshot::from_float(step, position)?);
            }
            let clock = Instant::now();
            let step = bundle
                .model()
                .step(&mut integer, input, ReadMode::Enabled)?;
            nanos.push(clock.elapsed().as_nanos());
            snapshots.push(Snapshot::from_integer(step, position)?);
            let assistant = position >= trace.assistant_start;
            for i in 0..4 {
                totals[i].add(&snapshots[i], target as usize, assistant, nanos[i]);
                local[i].add(&snapshots[i], target as usize, assistant, nanos[i]);
            }
            let mut differences = BTreeMap::new();
            for i in 0..3 {
                let d = difference(&snapshots[i], &snapshots[i + 1]);
                pairs[i].add(&d, trace, position);
                local_pairs[i].add(&d, trace, position);
                differences.insert(PAIRS[i], d);
            }
            let forms: BTreeMap<_, _> = FORMS
                .iter()
                .zip(&snapshots)
                .map(|(&name, s)| (name, s.scalars(target as usize)))
                .collect();
            write_row(
                &mut stream,
                &json!({"id":trace.id,"category":trace.category,"turn":trace.turn,
                "position":position,"input_token":input,"recorded_target_token":target,
                "assistant_saved_target":assistant,"forms":forms,"adjacent":differences}),
            )?;
        }
        turns.push(json!({"id":trace.id,"category":trace.category,"turn":trace.turn,
            "positions":trace.inputs.len(),"prompt_tokens":trace.prompt.len(),"saved_generated_targets":trace.generated.len(),
            "forms":FORMS.iter().zip(&local).map(|(&n,t)|(n,t.value())).collect::<BTreeMap<_,_>>(),
            "adjacent":PAIRS.iter().zip(&local_pairs).map(|(&n,t)|(n,t.value())).collect::<BTreeMap<_,_>>() }));
    }
    stream.sync_all()?;
    let positions: usize = traces.iter().map(|t| t.inputs.len()).sum();
    let assistant: usize = traces.iter().map(|t| t.generated.len()).sum();
    ensure(
        totals.iter().all(|t| {
            t.calls == positions
                && t.assistant.count == assistant
                && t.other.count == positions - assistant
        }),
        "executed trace denominator differs",
    )?;
    let result = json!({"forms":FORMS.iter().zip(&totals).map(|(&n,t)|(n,t.value())).collect::<BTreeMap<_,_>>(),
        "adjacent":PAIRS.iter().zip(&pairs).map(|(&n,t)|(n,t.value())).collect::<BTreeMap<_,_>>(),
        "turns":turns,"model_step_calls":4*positions,"saved_target_positions_per_form":positions,
        "elapsed_seconds":started.elapsed().as_secs_f64(),
        "scope":"Identical saved inputs with a fresh state each turn. Saved parent continuations are often weak, not correct answers or corpus NLL. Ordered interventions include recurrent interactions; no quality threshold or unique causal partition.",
        "hash_encoding":{"FF_QF_QQ":"little-endian F32 probabilities/state/read masses","integer":"little-endian U64 Q48 probabilities/read masses, I32 Q11 state"},
        "zero_probability":"Null mean NLL with INFINITE_ZERO_PROBABILITY status; no artificial floor.",
        "float_integrity_tolerance":PROBABILITY_SUM_TOLERANCE});
    write_json(&out.join("trace-summary.json"), &result)?;
    Ok(result)
}

fn float_stop(stop: &JointGenerationStop) -> Stop {
    match *stop {
        JointGenerationStop::Eos => Stop::Eos,
        JointGenerationStop::ShortCycle { period } => Stop::ShortCycle { period },
        JointGenerationStop::MaximumNewTokens => Stop::MaximumNewTokens,
        JointGenerationStop::FirstSentenceBoundary => Stop::FirstSentenceBoundary,
    }
}

fn generate_float(
    model: &JointModel,
    tokenizer: &HfBpeTokenizer,
    codec: &ByteBpeTokenizer,
    protocol: &DialogueProtocol,
    requests: &[Request],
    form: &str,
    out: &Path,
    reference: Option<&SavedReplay>,
) -> Result<Value> {
    let clock = Instant::now();
    let encoder = protocol.bind(codec)?;
    let mut stream = File::create_new(out.join(format!("responses-{form}.jsonl")))?;
    let (mut calls, mut selected, mut replies, mut native_seconds) = (0, 0, 0, 0.0);
    let mut rows = Vec::new();
    let mut reference_mismatches = Vec::new();
    for (request_index, request) in requests.iter().enumerate() {
        let mut history = vec![0];
        let mut turns = Vec::new();
        for (index, user) in request.user_turns.iter().enumerate() {
            let suffix = encoder.encode_user_prefix(user, index != 0);
            history.extend_from_slice(&suffix.tokens);
            let generation = joint_evaluation::generate_tokens(
                model,
                tokenizer,
                &history,
                ReadMode::Enabled,
                None,
                CAP,
            )?;
            ensure(
                generation.prompt_token_ids == history
                    && generation.incremental_step_calls
                        == history.len() + generation.generated_token_ids.len() - 1,
                "actual F32 input/call accounting differs",
            )?;
            calls += generation.incremental_step_calls;
            selected += generation.generated_token_ids.len();
            replies += 1;
            native_seconds += generation.elapsed_seconds;
            let stop = float_stop(&generation.stop);
            let reference_match = reference.map(|saved| {
                saved.rows.get(request_index).is_some_and(|row| {
                    row.id == request.id
                        && row.category == request.category
                        && row.turns.get(index).is_some_and(|turn| {
                            turn.user == *user
                                && turn.generation.prompt_token_ids == generation.prompt_token_ids
                                && turn.generation.generated_token_ids
                                    == generation.generated_token_ids
                                && turn.generation.raw_decoded_bytes == generation.raw_decoded_bytes
                                && turn.generation.stop == stop
                        })
                })
            });
            if reference_match == Some(false) {
                reference_mismatches.push(json!({"id":request.id,"turn":index+1}));
            }
            let caller = close_history(
                &mut history,
                &generation.generated_token_ids,
                stop,
                index + 1 < request.user_turns.len(),
            )?;
            let turn = json!({"turn":index+1,"user":user,"appended_user_prefix_ids":suffix.tokens,
                "model_eos":stop==Stop::Eos,"caller_eos_inserted_before_next_request":caller,
                "retained_history_ids":history,"generation":generation,
                "saved_reference_matches":reference_match,
                "execution":"fresh F32 session replays own exact full prefix each turn"});
            write_row(
                &mut stream,
                &json!({"id":request.id,"category":request.category,"turn":turn}),
            )?;
            turns.push(turn);
        }
        rows.push(json!({"id":request.id,"category":request.category,"turns":turns}));
    }
    stream.sync_all()?;
    ensure(
        replies == 58 && calls <= 4463 && selected <= 1856,
        "F32 fixed output inventory/call ceiling differs",
    )?;
    let result = json!({"form":form,"requests":38,"responses":replies,"generated_selections":selected,
        "incremental_step_calls":calls,"native_generation_seconds":native_seconds,
        "saved_reference_checked":reference.is_some(),"saved_reference_mismatches":reference_mismatches,
        "elapsed_seconds":clock.elapsed().as_secs_f64(),"rows":rows});
    write_json(&out.join(format!("responses-{form}.json")), &result)?;
    Ok(
        json!({"form":form,"responses":replies,"generated_selections":selected,"incremental_step_calls":calls,
        "saved_reference_checked":reference.is_some(),"saved_reference_mismatches":reference_mismatches,
        "native_generation_seconds":native_seconds,"elapsed_seconds":clock.elapsed().as_secs_f64()}),
    )
}

fn generate_integer(
    bundle: &Bundle,
    protocol: &DialogueProtocol,
    requests: &[Request],
    out: &Path,
) -> Result<Value> {
    let clock = Instant::now();
    let encoder = protocol.bind(bundle.tokenizer())?;
    let mut stream = File::create_new(out.join("responses-integer.jsonl"))?;
    let (mut calls, mut selected, mut replies, mut model_ns, mut native_ns) =
        (0, 0, 0, 0u128, 0u128);
    let mut rows = Vec::new();
    for request in requests {
        let mut conversation = bundle.dialogue_conversation(protocol, &[], 0, ReadMode::Enabled)?;
        let cursor = conversation.sampler_state()?;
        let mut history = Vec::new();
        let mut prior = None;
        let mut turns = Vec::new();
        for (index, user) in request.user_turns.iter().enumerate() {
            let before = conversation.step_calls();
            let native = conversation.respond(ConversationRequest {
                user,
                max_new_tokens: CAP,
                policy: SamplePolicy::Greedy,
                first_sentence: false,
                closure: TurnClosure::InterruptAssistant,
            })?;
            let suffix = encoder.encode_user_prefix(user, index != 0);
            let mut expected = if index == 0 { vec![0] } else { Vec::new() };
            if prior.is_some_and(|stop| stop != Stop::Eos) {
                expected.push(1);
            }
            expected.extend_from_slice(&suffix.tokens);
            ensure(
                native.appended_token_ids == expected,
                "integer new user/closure input differs",
            )?;
            ensure(
                match (prior, native.boundary) {
                    (None, TurnBoundary::InitialHistory) => true,
                    (Some(Stop::Eos), TurnBoundary::ModelEos { token: 1 }) => true,
                    (
                        Some(previous),
                        TurnBoundary::CallerEos {
                            token: 1,
                            interrupted_stop,
                        },
                    ) => previous == interrupted_stop && previous != Stop::Eos,
                    _ => false,
                },
                "integer explicit closure differs",
            )?;
            // Pending generated IDs already appear in history; append only the
            // wrapper's newly supplied IDs. It consumes pending tokens itself.
            history.extend_from_slice(&native.appended_token_ids);
            let g = &native.dialogue.generation;
            ensure(
                g.prompt_token_ids == history
                    && g.incremental_step_calls == conversation.step_calls() - before
                    && conversation.sampler_state()? == cursor
                    && g.sampler_state_after == cursor,
                "integer exact history, call accounting or inert greedy cursor differs",
            )?;
            validate_stop(&g.generated_token_ids, g.stop)?;
            calls += g.incremental_step_calls;
            selected += g.generated_token_ids.len();
            replies += 1;
            model_ns += g.model_step_nanoseconds;
            native_ns += g.whole_generation_nanoseconds;
            prior = Some(g.stop);
            history.extend_from_slice(&g.generated_token_ids);
            ensure(
                conversation.len() == history.len()
                    && conversation.step_calls() + 1 == history.len(),
                "integer pending-token accounting differs",
            )?;
            let turn = json!({"turn":index+1,"user":user,"exact_prompt_ids":g.prompt_token_ids,
                "appended_user_prefix_ids":suffix.tokens,"model_eos":g.stop==Stop::Eos,
                "retained_history_ids":history,"native_conversation_turn":native,
                "execution":"persistent integer conversation; final generated ID pending; caller closure is recorded on next turn"});
            write_row(
                &mut stream,
                &json!({"id":request.id,"category":request.category,"turn":turn}),
            )?;
            turns.push(turn);
        }
        rows.push(json!({"id":request.id,"category":request.category,"turns":turns}));
    }
    stream.sync_all()?;
    ensure(
        replies == 58 && calls <= 2960 && selected <= 1856,
        "integer fixed output inventory/call ceiling differs",
    )?;
    let result = json!({"form":"integer","requests":38,"responses":replies,"generated_selections":selected,
        "incremental_step_calls":calls,"model_step_nanoseconds":model_ns,"native_generation_nanoseconds":native_ns,
        "elapsed_seconds":clock.elapsed().as_secs_f64(),"rows":rows});
    write_json(&out.join("responses-integer.json"), &result)?;
    Ok(
        json!({"form":"integer","responses":replies,"generated_selections":selected,"incremental_step_calls":calls,
        "model_step_nanoseconds":model_ns,"native_generation_nanoseconds":native_ns,"elapsed_seconds":clock.elapsed().as_secs_f64()}),
    )
}

fn parameter_bindings(model: &JointModel) -> Result<Value> {
    let mut rows = Vec::new();
    for (name, var) in model.variables() {
        let values = var.flatten_all()?.to_vec1::<f32>()?;
        ensure(values.iter().all(|v| v.is_finite()), "nonfinite parameter")?;
        rows.push(json!({"name":name,"shape":var.dims(),"sha256_le_f32":digest_bytes(values.iter().map(|v|v.to_le_bytes()))}));
    }
    Ok(json!(rows))
}

fn load_saved_reference(
    args: &[PathBuf],
    mode: ObservationMode,
    provenance: &Value,
) -> Result<SavedReplay> {
    if mode == ObservationMode::HistoricalR1d {
        return Ok(serde_json::from_slice(&fs::read(&args[6])?)?);
    }
    ensure(
        sha256_file(&args[0].join("result.json"))? == CHILD_RESULT_SHA
            && fs::canonicalize(&args[6])? == fs::canonicalize(args[0].join("responses.json"))?
            && provenance["parameter_sha256"] == CHILD_MODEL_SHA
            && provenance["parameter_fingerprint"] == CHILD_FINGERPRINT,
        "child reference does not belong to the selected continuous checkpoint/report",
    )?;
    let result: Value = serde_json::from_slice(&fs::read(args[0].join("result.json"))?)?;
    ensure(
        result["schema"] == "uor-r4.dialogue-prefix-fit/1"
            && result["status"] == "TARGET_COMPLETE"
            && result["completed_step"] == 1024
            && result["final_parameter_fingerprint"] == CHILD_FINGERPRINT,
        "selected child result/clock/fingerprint differs",
    )?;
    let child: ChildResponses = serde_json::from_slice(&fs::read(&args[6])?)?;
    ensure(
        child.requests_sha256 == REQUESTS_SHA,
        "child reference request identity differs",
    )?;
    let responses = child.rows.iter().map(|r| r.turns.len()).sum();
    let calls = child
        .rows
        .iter()
        .flat_map(|r| &r.turns)
        .map(|t| t.generation.incremental_step_calls)
        .sum();
    let selected = child
        .rows
        .iter()
        .flat_map(|r| &r.turns)
        .map(|t| t.generation.generated_token_ids.len())
        .sum();
    // The child response file does not contain an artifact object. Its lineage
    // is established above by the sealed report/result/checkpoint, then adapted
    // into the shared trace validator; it is never relabeled as historical R1d.
    Ok(SavedReplay {
        artifact: provenance.clone(),
        protocol: child.protocol,
        protocol_identity: PROTOCOL_ID.into(),
        requests: child.rows.len(),
        responses,
        rows: child.rows,
        generation_incremental_step_calls: calls,
        generated_selections: selected,
    })
}

fn observe(args: &[PathBuf], source: &str, mode: ObservationMode) -> Result<()> {
    let clock = Instant::now();
    let out = &args[7];
    for (p, hash) in [
        (&args[1], CORPUS_SHA),
        (&args[2], TOKENIZER_SHA),
        (&args[5], REQUESTS_SHA),
        (&args[6], mode.reference_sha()),
    ] {
        ensure(
            sha256_file(p)? == hash,
            "fixed observation input identity differs",
        )?;
    }
    for root in [&args[0], &args[3], &args[4]] {
        report_output::verify(root)?;
    }
    for p in [&args[5], &args[6]] {
        report_output::verify(
            p.parent()
                .ok_or_else(|| io::Error::other("saved input has no sealed root"))?,
        )?;
    }
    report_output::verify(&args[4].join("tables"))?;
    let requests: Vec<Request> = serde_json::from_slice(&fs::read(&args[5])?)?;
    let hard: Value = serde_json::from_slice(&fs::read(args[3].join("hard-model.json"))?)?;
    let bundle_meta: Value = serde_json::from_slice(&fs::read(args[4].join("bundle.json"))?)?;
    for name in [
        "hard-model.json",
        "hard-parameters.json",
        "hard-parameters.bin",
    ] {
        ensure(
            sha256_file(&args[3].join(name))? == sha256_file(&args[4].join("model").join(name))?,
            "packed/bundle model bytes differ",
        )?;
    }
    ensure(
        bundle_meta["parent_sealed_manifest_sha256"]
            == sha256_file(&args[3].join("manifest.json"))?,
        "bundle does not bind supplied packed seal",
    )?;
    let artifact =
        match mode {
            ObservationMode::HistoricalR1d => ParentArtifact::Historical(
                LegacyDialogueArtifact::load(&args[0], &args[1], &args[2], &Device::Cpu)?,
            ),
            ObservationMode::CompletePrefixChild | ObservationMode::RoundedCompletePrefixChild => {
                ParentArtifact::Child(DialogueChildArtifact::load(
                    &args[0],
                    &args[1],
                    &args[2],
                    &Device::Cpu,
                )?)
            }
        };
    let provenance = artifact.provenance()?;
    let saved = load_saved_reference(args, mode, &provenance)?;
    let parent_sha = match mode {
        ObservationMode::HistoricalR1d => PARENT_SHA,
        ObservationMode::CompletePrefixChild | ObservationMode::RoundedCompletePrefixChild => {
            CHILD_MODEL_SHA
        }
    };
    ensure(
        provenance["parameter_sha256"] == parent_sha
            && provenance["steps_completed"] == mode.model_step()
            && provenance["protocol_identity"] == PROTOCOL_ID
            && saved.artifact == provenance
            && hard["conversion_provenance"]["parent"] == provenance
            && hard["conversion_provenance"]["schema"] == mode.conversion_schema()
            && saved.protocol_identity == PROTOCOL_ID
            && serde_json::to_value(&saved.protocol)? == serde_json::to_value(artifact.protocol())?,
        "selected parent/conversion/reference mode, identity or protocol differs",
    )?;
    let bundle = Bundle::load(&args[4])?;
    let observed_artifact_alpha_updates = if mode == ObservationMode::RoundedCompletePrefixChild {
        let updates = hard["conversion_provenance"]["rounding_stage"]["completed_updates"]
            .as_u64()
            .ok_or_else(|| io::Error::other("learned child alpha clock missing"))?;
        ensure(updates > 0, "learned child requires nonzero alpha updates")?;
        updates
    } else {
        0
    };
    ensure(
        bundle.model().serving_profile() == ServingProfile::Dialogue576
            && bundle.model().config() == &artifact.model().config
            && bundle.tokenizer().address() == artifact.tokenizer().address()
            && sha256_file(&args[4].join("tokenizer.json"))? == TOKENIZER_SHA,
        "bundle profile/model/tokenizer differs",
    )?;
    let qq =
        JointModel::load_hard_for_profile(&args[3], &Device::Cpu, ServingProfile::Dialogue576)?;
    let qf = qq.packed_parameter_precision_view()?;
    let ff = artifact.model();
    let quant = qq
        .quantization()
        .ok_or_else(|| io::Error::other("missing packed quantization"))?;
    ensure(
        ff.config == qq.config
            && ff.config == qf.config
            && ff.parameter_count() == 5_429_826
            && ff.quantization().is_none()
            && ff.admission_policy() == AdmissionPolicy::Full
            && qq.admission_policy() == AdmissionPolicy::Full
            && qf.admission_policy() == AdmissionPolicy::Full
            && qf.precision_mode() == Some(PrecisionMode::QF)
            && qf.quantization() == qq.quantization()
            && quant.start_step == mode.model_step()
            && quant.completed_step == mode.model_step()
            && quant.ramp_steps == 1
            && quant.preparation == Some(mode.preparation()),
        "fixed FF/QF/QQ configuration or honest calibration clock differs",
    )?;
    let ff_parameters = parameter_bindings(ff)?;
    let qq_parameters = parameter_bindings(&qq)?;
    ensure(
        parameter_bindings(&qf)? == qq_parameters,
        "QF/QQ decoded parameter arrays differ",
    )?;
    let tokenizer = HfBpeTokenizer::from_tokenizer_json_bytes(&fs::read(&args[2])?)
        .ok_or_else(|| io::Error::other("generation tokenizer parse"))?;
    ensure(
        tokenizer.address() == artifact.tokenizer().address(),
        "F32 generation tokenizer differs",
    )?;
    let traces = prepare_traces(
        &requests,
        &saved,
        artifact.tokenizer(),
        artifact.protocol(),
        mode,
    )?;
    let mut input_files = vec![
        args[1].clone(),
        args[2].clone(),
        args[5].clone(),
        args[6].clone(),
        args[0].join("manifest.json"),
        args[3].join("manifest.json"),
        args[3].join("hard-model.json"),
        args[3].join("hard-parameters.json"),
        args[3].join("hard-parameters.bin"),
        args[4].join("manifest.json"),
        args[4].join("bundle.json"),
        args[4].join("tables/tables.json"),
        args[4].join("tables/tables.bin"),
        args[4].join("tables/manifest.json"),
    ];
    if mode.is_child() {
        input_files.extend([
            args[0].join("result.json"),
            args[0].join("checkpoint-final/manifest.json"),
            args[0].join("checkpoint-final/checkpoint.json"),
            args[0].join("checkpoint-final/config.json"),
            args[0].join("checkpoint-final/model.safetensors"),
        ]);
    }
    let identities = input_files
        .iter()
        .map(|p| identity(p))
        .collect::<Result<Vec<_>>>()?;
    write_json(
        &out.join("input-binding.json"),
        &json!({"schema":"uor-r4.native-dialogue576-observation-inputs/1",
        "status":"VALIDATED_BEFORE_FIRST_FORWARD","source_commit":source,"executable_sha256":sha256_file(&std::env::current_exe()?)?,
        "source_sha256":source_hashes(),"cpu_accelerate_compiled":cfg!(feature="cpu-accelerate"),
        "thread_environment":(["RAYON_NUM_THREADS","VECLIB_MAXIMUM_THREADS","OMP_NUM_THREADS","GEMM_NUM_THREADS"].into_iter().map(|k|(k,std::env::var(k).ok())).collect::<BTreeMap<_,_>>()),
        "observation_mode":mode,"inputs":identities,"selected_parent":provenance,
        "historical_parent":if mode==ObservationMode::HistoricalR1d {provenance.clone()} else {Value::Null},
        "conversion_provenance":hard["conversion_provenance"],
        "packed_spec_and_scales":hard["quantization"],"bundle_identity":bundle.identity(),"protocol":artifact.protocol(),
        "FF_parameters":ff_parameters,"QF_QQ_parameters":qq_parameters,
        "numerical_contracts":{"FF":ff.numerical_contract(),"QF":qf.numerical_contract(),"QQ":qq.numerical_contract(),"integer":hard["numerical_contract"]},
        "reference_counts":{"turns":mode.reference_counts().0,"prompt_id_occurrences":mode.reference_counts().1,"generated_selections":mode.reference_counts().2,"trace_positions_per_form":mode.reference_counts().3},
        "scope":mode.input_scope()}),
    )?;
    write_json(
        &out.join("requests.json"),
        &serde_json::to_value(&requests)?,
    )?;
    write_json(
        &out.join("trace-inputs.json"),
        &serde_json::to_value(&traces)?,
    )?;
    let preparation_seconds = clock.elapsed().as_secs_f64();
    let trace_summary = trace_all([ff, &qf, &qq], &bundle, &traces, out)?;
    let ff_output = generate_float(
        ff,
        &tokenizer,
        artifact.tokenizer(),
        artifact.protocol(),
        &requests,
        "FF",
        out,
        mode.is_child().then_some(&saved),
    )?;
    let qq_output = generate_float(
        &qq,
        &tokenizer,
        artifact.tokenizer(),
        artifact.protocol(),
        &requests,
        "QQ",
        out,
        None,
    )?;
    let integer_output = generate_integer(&bundle, artifact.protocol(), &requests, out)?;
    ensure(
        parameter_bindings(ff)? == ff_parameters
            && parameter_bindings(&qq)? == qq_parameters
            && parameter_bindings(&qf)? == qq_parameters,
        "parameters changed during forward-only observation",
    )?;
    let generation_calls = [&ff_output, &qq_output, &integer_output]
        .iter()
        .map(|v| v["incremental_step_calls"].as_u64().unwrap_or(0))
        .sum::<u64>();
    let selections = [&ff_output, &qq_output, &integer_output]
        .iter()
        .map(|v| v["generated_selections"].as_u64().unwrap_or(0))
        .sum::<u64>();
    let reference_matches = mode == ObservationMode::HistoricalR1d
        || ff_output["saved_reference_mismatches"]
            .as_array()
            .is_some_and(Vec::is_empty);
    let trace_calls = trace_summary["model_step_calls"]
        .as_u64()
        .ok_or_else(|| io::Error::other("missing trace call count"))?;
    write_json(
        &out.join("result.json"),
        &json!({"schema":"uor-r4.native-dialogue576-observation/1","status":if reference_matches {"OBSERVATION_COMPLETE_REVIEW_PENDING"} else {"OBSERVATION_COMPLETE_REFERENCE_MISMATCH"},
        "observation_mode":mode,"child_FF_saved_reference_match":if mode.is_child() {json!(reference_matches)} else {Value::Null},
        "source_commit":source,"executable_sha256":sha256_file(&std::env::current_exe()?)?,
        "input_binding":identity(&out.join("input-binding.json"))?,"trace_summary":identity(&out.join("trace-summary.json"))?,
        "outputs":[ff_output,qq_output,integer_output],"requests":38,"responses":174,
        "trace_model_step_calls":trace_summary["model_step_calls"],"generation_model_step_calls":generation_calls,
        "total_model_step_calls":trace_calls+generation_calls,"generated_selections_including_eos":selections,
        "new_optimizer_updates":0,"new_backward_calls":0,"new_calibrations":0,"parameters_unchanged_after_observation":true,
        "observed_artifact_alpha_updates":observed_artifact_alpha_updates,
        "preparation_seconds":preparation_seconds,"elapsed_seconds_before_sealing":clock.elapsed().as_secs_f64(),
        "quality":"REVIEW_PENDING_NO_AUTOMATIC_PASS","energy_or_native_speed_advantage":"NOT_MEASURED",
        "scope":"Four-view numerical localization on identical saved weak-parent trajectories and actual fixed greedy dialogue outputs. Dense parameter access remains; whole-form histories may diverge. No model promotion, new dose/decoder/seed or categorical RNG parity claim."}),
    )?;
    ensure(
        reference_matches,
        "current child FF differs from its own saved reference; all generated outputs preserved",
    )?;
    Ok(())
}

fn write_row(file: &mut File, value: &Value) -> Result<()> {
    serde_json::to_writer(&mut *file, value)?;
    file.write_all(b"\n")?;
    file.flush()?;
    Ok(())
}
fn write_json(path: &Path, value: &Value) -> Result<()> {
    let mut file = File::create_new(path)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    Ok(())
}
fn identity(path: &Path) -> Result<Value> {
    Ok(json!({"path":path,"bytes":fs::metadata(path)?.len(),"sha256":sha256_file(path)?}))
}
fn source_hashes() -> BTreeMap<&'static str, String> {
    [
        (
            "example.rs",
            include_bytes!("dialogue-integer-observe.rs").as_slice(),
        ),
        (
            "dialogue_artifact.rs",
            include_bytes!("../src/dialogue_artifact.rs").as_slice(),
        ),
        (
            "dialogue_child_artifact.rs",
            include_bytes!("../src/dialogue_child_artifact.rs").as_slice(),
        ),
        (
            "dialogue_rounding_artifact.rs",
            include_bytes!("../src/dialogue_rounding_artifact.rs").as_slice(),
        ),
        (
            "joint_model.rs",
            include_bytes!("../src/joint_model.rs").as_slice(),
        ),
        (
            "joint_quantization.rs",
            include_bytes!("../src/joint_quantization.rs").as_slice(),
        ),
        (
            "joint_evaluation.rs",
            include_bytes!("../src/joint_evaluation.rs").as_slice(),
        ),
        (
            "reference_eval.rs",
            include_bytes!("../src/reference_eval.rs").as_slice(),
        ),
        (
            "integer/config.rs",
            include_bytes!("../../uor-r4-integer/src/config.rs").as_slice(),
        ),
        (
            "integer/model.rs",
            include_bytes!("../../uor-r4-integer/src/model.rs").as_slice(),
        ),
        (
            "integer/packed_rows.rs",
            include_bytes!("../../uor-r4-integer/src/packed_rows.rs").as_slice(),
        ),
        (
            "integer/math.rs",
            include_bytes!("../../uor-r4-integer/src/math.rs").as_slice(),
        ),
        (
            "integer/tables.rs",
            include_bytes!("../../uor-r4-integer/src/tables.rs").as_slice(),
        ),
        (
            "integer/bundle.rs",
            include_bytes!("../../uor-r4-integer/src/bundle.rs").as_slice(),
        ),
        (
            "integer/generation.rs",
            include_bytes!("../../uor-r4-integer/src/generation.rs").as_slice(),
        ),
        (
            "integer/conversation.rs",
            include_bytes!("../../uor-r4-integer/src/generation/conversation.rs").as_slice(),
        ),
        (
            "integer/sampling.rs",
            include_bytes!("../../uor-r4-integer/src/sampling.rs").as_slice(),
        ),
        (
            "tokenizer/dialogue.rs",
            include_bytes!("../../uor-r4-tokenizer/src/dialogue.rs").as_slice(),
        ),
        (
            "tokenizer/lib.rs",
            include_bytes!("../../uor-r4-tokenizer/src/lib.rs").as_slice(),
        ),
    ]
    .into_iter()
    .map(|(name, bytes)| (name, hex::encode(Sha256::digest(bytes))))
    .collect()
}

fn main() -> Result<()> {
    let (mode, args) = parse_args(std::env::args_os().skip(1).map(PathBuf::from).collect())?;
    let source = option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("");
    ensure(
        source.len() == 40 && source.bytes().all(|b| b.is_ascii_hexdigit()),
        "build with full UOR_BUILD_SOURCE_COMMIT",
    )?;
    let out = &args[7];
    report_output::claim(out)?;
    let result = observe(&args, source, mode);
    if let Err(error) = &result {
        write_json(
            &out.join("failed-attempt.json"),
            &json!({"status":"INCOMPLETE_OR_UNVERIFIED","error":error.to_string(),
            "source_commit":source,"observation_mode":mode,"input_paths":&args[..7],
            "scope":"Preserved execution/input failure; not a model-quality verdict. Flushed per-position/turn rows retain any completed observations. No automatic retry."}),
        )?;
    }
    report_output::seal(out)?;
    report_output::verify(out)?;
    result?;
    println!(
        "{}",
        json!({"status":"OBSERVATION_COMPLETE_REVIEW_PENDING","observation_mode":mode,"report_root":out})
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn child_mode_is_explicit_and_keeps_distinct_reference_identity_and_counts() -> Result<()> {
        let paths: Vec<PathBuf> = (0..8).map(|i| PathBuf::from(format!("path-{i}"))).collect();
        let (historical, unchanged) = parse_args(paths.clone())?;
        assert_eq!(historical, ObservationMode::HistoricalR1d);
        assert_eq!(unchanged, paths);
        let mut child_args = vec![PathBuf::from("--child")];
        child_args.extend(paths.clone());
        let (child, supplied) = parse_args(child_args)?;
        assert_eq!(child, ObservationMode::CompletePrefixChild);
        assert_eq!(supplied, paths);
        assert_eq!(historical.reference_counts(), (58, 2623, 1766, 4331));
        assert_eq!(child.reference_counts(), (58, 2464, 1508, 3914));
        assert_ne!(historical.reference_sha(), child.reference_sha());
        assert_eq!(child.model_step(), 1024);
        let mut rounded_args = vec![PathBuf::from("--child-rounded")];
        rounded_args.extend(paths.clone());
        let (rounded, supplied) = parse_args(rounded_args)?;
        assert_eq!(supplied, paths);
        assert_eq!(rounded, ObservationMode::RoundedCompletePrefixChild);
        assert_eq!(rounded.reference_sha(), child.reference_sha());
        assert_eq!(rounded.reference_counts(), child.reference_counts());
        assert_eq!(rounded.model_step(), child.model_step());
        assert_ne!(rounded.conversion_schema(), child.conversion_schema());
        assert_eq!(
            rounded.preparation(),
            QuantizationPreparation::CalibratedForRounding
        );
        assert_eq!(
            child.preparation(),
            QuantizationPreparation::CalibratedForExport
        );
        assert_eq!(historical.model_step(), 2237);
        assert!(parse_args(vec![PathBuf::from("--child")]).is_err());
        let mut unknown = paths;
        unknown[0] = PathBuf::from("--unknown");
        assert!(parse_args(unknown).is_err());
        Ok(())
    }

    #[test]
    fn common_trace_scores_final_eos_without_consuming_it() -> Result<()> {
        let (inputs, targets, boundary) = trace_tokens(&[0, 7, 8], &[9, 1])?;
        assert_eq!(inputs, [0, 7, 8, 9]);
        assert_eq!(targets, [7, 8, 9, 1]);
        assert_eq!(boundary, 2);
        assert!(trace_tokens(&[0], &[]).is_err());
        assert!(trace_tokens(&[7], &[1]).is_err());
        Ok(())
    }

    #[test]
    fn target_reduction_preserves_denominators_and_true_zero() {
        let mut x = Targets::default();
        x.add(0.5);
        x.add(0.25);
        assert_eq!(x.value()["targets"], 2);
        assert_eq!(x.value()["zero_probability_targets"], 0);
        assert!(
            (x.value()["mean_nll_nats"].as_f64().unwrap_or(f64::NAN) - 1.0397207708399179).abs()
                < 1e-12
        );
        x.add(0.0);
        assert!(x.value()["mean_nll_nats"].is_null());
        assert_eq!(x.value()["nll_status"], "INFINITE_ZERO_PROBABILITY");
        assert_eq!(x.value()["targets"], 3);
    }

    #[test]
    fn closure_adds_caller_eos_only_for_an_interrupted_previous_turn() -> Result<()> {
        let mut eos = vec![0, 8];
        assert!(!close_history(&mut eos, &[9, 1], Stop::Eos, true)?);
        assert_eq!(eos, [0, 8, 9, 1]);
        let mut cycle = vec![0, 8];
        assert!(close_history(
            &mut cycle,
            &[9, 9, 9],
            Stop::ShortCycle { period: 1 },
            true
        )?);
        assert_eq!(cycle, [0, 8, 9, 9, 9, 1]);
        assert!(validate_stop(&[9, 9, 9, 10], Stop::MaximumNewTokens).is_err());
        Ok(())
    }
}
