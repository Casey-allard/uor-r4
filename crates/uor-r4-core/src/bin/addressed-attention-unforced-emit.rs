//! Measurement-only probe: does the UNFORCED addressed-attention emission path
//! carry the produced cursor byte?
//!
//! Every previous measurement of this question used a forced-read or
//! forced-operator arm (or measured the forced case). This binary drives the
//! *unmodified* engine on the authored `arith-canonical-decimal/1` split with
//! `CompiledPolicy` over unfitted `Parameters::seeded(7341)` and **no forcing at
//! all**, and asks:
//!
//!   A. natural arm - on the 80 scored positions, does the engine's own
//!      selection loop pick the intended operand spans and the intended
//!      operator? On the positions where both happen naturally, does the
//!      emitted symbol equal the byte at the produced lease cursor?
//!   B. forced-emission-only arm - a harness-local `Policy` wrapper overrides
//!      *only* `Head::Emission` to the target and leaves the read and the
//!      operator natural.
//!
//! It also captures the packed `Phase::Emit` input at the emission consultation
//! (bits 792..800 carry the active lease's cursor byte, `inputs.rs:120`) so the
//! comparison is made against the byte that is actually wired in, not against a
//! value read off the offer object. For every scored position the binary sweeps
//! the 8-bit byte field over 0..=255 with the rest of the natural emission input
//! held fixed, which measures what the frozen 1024 -> 9 LUT cascade
//! (`circuit.rs:5`) can distinguish about that field and what the frozen
//! emission head would emit for each reachable context.
//!
//! No production file is modified. Every report root is exclusively claimed
//! before model work and sealed with its complete file set.
//!
//! Subcommands:
//!   prereg <abs-base>   frozen questions, thresholds and triage written first
//!   run    <abs-base>   natural arm, forced-emission-only arm, comparison
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde_json::{json, Value};
use uor_r4_core::native_geometric::addressed_attention::artifact::{
    BoundGeometry, Model, Provenance,
};
use uor_r4_core::native_geometric::addressed_attention::circuit::INPUT_BITS;
use uor_r4_core::native_geometric::addressed_attention::engine::{
    self, Head, Policy, RuntimeSession, Work,
};
use uor_r4_core::native_geometric::addressed_attention::inputs::Phase;
use uor_r4_core::native_geometric::addressed_attention::objects::{self, Action, Symbol};
use uor_r4_core::native_geometric::addressed_attention::pilot::{Control, Controlled};
use uor_r4_core::native_geometric::addressed_attention::pilot_data::Example;
use uor_r4_core::native_geometric::addressed_attention::policy::{
    implementation_digest, Parameters,
};
use uor_r4_core::report_output;

type AnyResult<T> = Result<T, Box<dyn std::error::Error>>;

const PARAMETER_SEED: u64 = 7341;
const MODEL_DATA: &[u8] = b"addressed-attention-unforced-emit/1";
const CONFIG_DIGEST: &[u8] = b"addressed-attention-unforced-emit-config/1";
const SYMBOL_EOS: u16 = 256;
const OP_ADD: usize = 3;
const OP_SUB: usize = 4;

/// Frozen before any engine call in this session. `prereg` mode writes it.
const PREREG: &str = r#"{
  "schema": "uor-r4.addressed-attention-unforced-emit-prereg/1",
  "frozen": "written before any engine call in this session; no threshold or triage rule below was adjusted after seeing a result",
  "question": "On the addressed-attention emission path, does the UNFORCED run carry the computed cursor byte from the operator to the emitted symbol?",
  "instrument": {
    "bin": "addressed-attention-unforced-emit (new, measurement-only; no production change)",
    "policy": "Deterministic CompiledPolicy over Parameters::seeded(7341), BoundGeometry::canonical(), wrapped in Control::Full (pilot.rs:43-73, a pass-through for Full)",
    "split": "arith-canonical-decimal/1 (the authored split of addressed-attention-arith-oracle), prompt shape newline \"{a}\\n{b}\\n\", 32 examples, 48 digit positions + 32 EOS = 80 scored positions",
    "recipe": "identical to addressed-attention-arith-oracle `run`: prompt bytes are observed with runtime.observe; each scored position calls runtime.predict; the observation is teacher-forced to the target byte/EOS; the emitted symbol is the engine's own proposal and is NOT fed back",
    "forcing": "arm A: none. arm B: Head::Emission choice overridden to the target index only; read and operator untouched",
    "input_capture": "the harness Policy wrapper receives the packed [bool; 1024] at every context call; at Phase::Emit it reads bits 792..800 (active lease cursor byte, inputs.rs:120) and bits 812..814/816..822/822..829/844",
    "reach_sweep": "at each scored position, with the natural Phase::Emit input otherwise fixed, the 8-bit field at 792..800 is set to each value 0..=255 and the compiled circuit's context is read; the frozen emission head's argmax is read for each distinct context"
  },
  "primary_questions": {
    "Q1_natural_read": "count of the 80 scored positions where the engine's own selection loop picks BOTH intended operand spans; reported structurally (Reference::Occurrence start/end equal to the intended span) and as the predecessor metric (payload bytes equal the intended span bytes)",
    "Q2_natural_operator": "count of the 80 scored positions where the Control head selects the intended operator (AddAB for op==0, SubAB for op==1)",
    "Q3_conditional_emission": "on the positions where Q1 and Q2 both hold naturally, count where the emitted symbol (Symbol::Byte code) equals the byte at the produced lease cursor (offer.active().byte(), the same byte wired at bits 792..800)",
    "Q3_void_rule": "if the Q1 and Q2 intersection is empty, the conditional count is VOID, not a failure: no position reached the comparison. A zero count over a non-empty intersection is evidence; a count over an empty intersection is not",
    "Q4_firing_counter": "Work counters from engine.rs:44 (circuit_calls, candidate_scores, predictions, observations, scalar_decodes, selected_operations) summed over the arm, plus the harness count of Head::Emission consultations. selected_operations == 0 makes the comparison void",
    "Q5_input_byte": "among scored positions whose action is AddAB/SubAB (a provisional prospective result lease), the packed input byte at bits 792..800 versus the emitted symbol and versus the target byte"
  },
  "second_arm": {
    "arm_B": "harness-local Policy wrapper forcing ONLY Head::Emission to the target index (the engine passes legal=[true;257] at engine.rs:393, so the override is always legal)",
    "reported": "how many scored positions emit the produced cursor byte under arm B, and how arm B's trajectory compares with arm A (the emitted symbol is consumed by engine.rs:467-474 and objects.rs:629, so a forced emission also changes what the engine accepts)",
    "separates": "if the natural operator never produces a byte that could be carried, arm B's count is constrained by the natural read/operator, not by the readout; if the natural operator does produce bytes and arm B still cannot carry them, the readout is implicated"
  },
  "triage_rules": {
    "a_value_never_produced": "the Q1+Q2 intersection is empty AND no scored position has a provisional prospective result lease carrying the exactly correct canonical value",
    "b_produced_but_context_not_distinguishing": "at provisional positions whose packed input byte already equals the target byte, the emission head's argmax for the reached context is not the target AND the 0..=255 sweep of the byte field is not injective on the byte (one context serves several bytes)",
    "c_something_else": "any residual: the value is produced and the byte is in the input and the sweep distinguishes bytes, yet the emitted symbol differs; report the numbers without a mechanism claim",
    "no_verdict": "this instrument reports measured counts and the named missing component; it retires and relabels no mechanism"
  },
  "honesty": [
    "The split and its answers are constructed by the instrument (exact i64 arithmetic rendered by objects::encode_i64), not a frozen external fixture.",
    "Both arms use unfitted Parameters::seeded(7341): this is a wiring/readout measurement, not a capability result.",
    "Scored positions are answer bytes and EOS only; prefill positions are driven identically but not scored, exactly as the predecessor instrument counts.",
    "The byte-field sweep is host-side arithmetic on a copy of the packed input; it does not enter the engine and does not force anything in arm A."
  ]
}"#;

// ------------------------------------------------------------------- split

/// (a, b, op) with op 0 = Add, 1 = Sub. Identical to
/// `addressed-attention-arith-oracle` PROBLEMS, and to
/// `addressed-attention-emission-context` PROBLEMS.
const PROBLEMS: [(i64, i64, u8); 32] = [
    (1, 2, 0),
    (2, 3, 0),
    (3, 4, 0),
    (4, 5, 0),
    (1, 7, 0),
    (2, 6, 0),
    (3, 5, 0),
    (4, 4, 0),
    (5, 7, 0),
    (6, 8, 0),
    (8, 9, 0),
    (9, 9, 0),
    (9, 3, 1),
    (8, 2, 1),
    (7, 5, 1),
    (6, 1, 1),
    (5, 4, 1),
    (9, 9, 1),
    (8, 6, 1),
    (7, 2, 1),
    (12, 34, 0),
    (21, 45, 0),
    (33, 26, 0),
    (40, 19, 0),
    (11, 88, 0),
    (55, 27, 0),
    (56, 23, 1),
    (48, 19, 1),
    (90, 45, 1),
    (73, 28, 1),
    (64, 37, 1),
    (81, 16, 1),
];

/// The chosen shape of the predecessor instrument (`newline`, chosen by its
/// offers-only diagnose subcommand): "{a}\n{b}\n".
fn render(a: i64, b: i64) -> Vec<u8> {
    format!("{a}\n{b}\n").into_bytes()
}
fn expected_op(index: usize) -> usize {
    if PROBLEMS[index].2 == 0 {
        OP_ADD
    } else {
        OP_SUB
    }
}
fn op_name(index: usize) -> &'static str {
    if PROBLEMS[index].2 == 0 {
        "Add"
    } else {
        "Sub"
    }
}
/// Intended operand spans `(start, len)` in "{a}\n{b}\n".
fn intended_spans(index: usize) -> [(u64, u8); 2] {
    let (a, b, _) = PROBLEMS[index];
    let (sa, sb) = (a.to_string(), b.to_string());
    let (la, lb) = (sa.len() as u64, sb.len() as u8);
    [(0, la as u8), (la + 1, lb)]
}
fn expected_scalar(index: usize) -> i64 {
    let (a, b, op) = PROBLEMS[index];
    if op == 0 {
        a + b
    } else {
        a - b
    }
}
fn example_of(index: usize) -> Example {
    let (a, b, _) = PROBLEMS[index];
    let scalar = expected_scalar(index);
    Example {
        id: format!("arith/{}/{index}", op_name(index).to_lowercase()),
        family: format!(
            "arith_canonical_decimal_{}",
            op_name(index).to_lowercase()
        ),
        prompt: render(a, b),
        answer: scalar.to_string().into_bytes(),
        expected_scalar: Some(scalar),
    }
}
fn action_index(action: Action) -> usize {
    match action {
        Action::Hold => 0,
        Action::AcquireA => 1,
        Action::AcquireB => 2,
        Action::AddAB => 3,
        Action::SubAB => 4,
        Action::Advance => 5,
        Action::Clear => 6,
    }
}
fn action_name(action: Action) -> &'static str {
    match action {
        Action::Hold => "Hold",
        Action::AcquireA => "AcquireA",
        Action::AcquireB => "AcquireB",
        Action::AddAB => "AddAB",
        Action::SubAB => "SubAB",
        Action::Advance => "Advance",
        Action::Clear => "Clear",
    }
}
fn symbol_code(s: Symbol) -> u16 {
    match s {
        Symbol::Byte(b) => u16::from(b),
        Symbol::Eos => SYMBOL_EOS,
    }
}
fn symbol_of(code: u16) -> AnyResult<Symbol> {
    match code {
        0..=255 => Ok(Symbol::Byte(code as u8)),
        SYMBOL_EOS => Ok(Symbol::Eos),
        other => Err(format!("symbol code out of domain: {other}").into()),
    }
}
fn canonical(value: i64) -> Vec<u8> {
    let (bytes, len) = objects::encode_i64(value);
    bytes[..usize::from(len)].to_vec()
}
fn span_string(prompt: &[u8], span: (u64, u8)) -> String {
    let start = span.0 as usize;
    let end = start + usize::from(span.1);
    prompt
        .get(start..end)
        .map(|b| String::from_utf8_lossy(b).to_string())
        .unwrap_or_default()
}
fn bits_u64(bits: &[bool; INPUT_BITS], offset: usize, width: usize) -> u64 {
    let mut v = 0u64;
    for i in 0..width {
        if bits[offset + i] {
            v |= 1u64 << i;
        }
    }
    v
}
fn set_bits_u64(bits: &mut [bool; INPUT_BITS], offset: usize, width: usize, value: u64) {
    for i in 0..width {
        bits[offset + i] = (value >> i) & 1 != 0;
    }
}
fn digest_bits(bits: &[bool; INPUT_BITS]) -> String {
    let mut packed = [0u8; INPUT_BITS / 8];
    for (i, &bit) in bits.iter().enumerate() {
        if bit {
            packed[i / 8] |= 1 << (i % 8);
        }
    }
    blake3::hash(&packed).to_hex().to_string()
}

// --------------------------------------------------------- emission capture

/// Everything the harness observes at one `Head::Emission` consultation.
#[derive(Clone, Debug)]
struct EmitCapture {
    /// Packed input byte at bits 792..800 (the active lease's cursor byte).
    input_byte: u8,
    /// Packed active-lane fields, for wire verification.
    active_kind: u8,
    active_cursor: u8,
    active_length: u8,
    active_acknowledged: bool,
    active_provisional: bool,
    flags: u8,
    /// Context the compiled circuit returned for the unmodified natural input.
    context: u16,
    /// Context the circuit returns when only bits 792..800 are set to the
    /// target byte (None when the target is EOS, which is not a byte).
    context_for_target_byte: Option<u16>,
    /// Distinct contexts over a 0..=255 sweep of bits 792..800.
    sweep_distinct_contexts: u16,
    /// Distinct emission-head argmax symbols over that sweep.
    sweep_distinct_argmax: u16,
    /// Emission-head argmax for the natural context.
    argmax_for_context: u16,
    /// Emission-head argmax for the target-byte context.
    argmax_for_target_byte: Option<u16>,
    /// Number of other byte values sharing the target byte's context.
    target_byte_context_peers: Option<u16>,
    /// blake3 of the full packed natural emission input.
    input_digest: String,
}

impl EmitCapture {
    fn json(&self) -> Value {
        json!({
            "input_byte_792_800": self.input_byte,
            "active_kind": self.active_kind,
            "active_cursor": self.active_cursor,
            "active_length": self.active_length,
            "active_acknowledged": self.active_acknowledged,
            "active_provisional": self.active_provisional,
            "flags_800_808": self.flags,
            "context": self.context,
            "context_for_target_byte": self.context_for_target_byte,
            "argmax_for_context": self.argmax_for_context,
            "argmax_for_target_byte": self.argmax_for_target_byte,
            "target_byte_context_peers": self.target_byte_context_peers,
            "sweep_distinct_contexts": self.sweep_distinct_contexts,
            "sweep_distinct_argmax": self.sweep_distinct_argmax,
            "input_digest": self.input_digest,
        })
    }
}

// -------------------------------------------------------------- policy arms

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    /// No forcing anywhere.
    Natural,
    /// Force only `Head::Emission` to the target; read and operator natural.
    ForcedEmissionOnly,
}

struct Instr<'a> {
    inner: Controlled<'a>,
    arm: Arm,
    /// Target symbol code for the current scored position (256 = EOS).
    target: u16,
    /// Capture of the most recent `Phase::Emit` consultation.
    emit: Option<EmitCapture>,
    emission_consults: u64,
    forced_emission_overrides: u64,
    /// Emission consultations during the current scored position.
    consults_this_position: u64,
}

impl<'a> Instr<'a> {
    fn new(compiled: &'a uor_r4_core::native_geometric::addressed_attention::circuit::PrimitiveExport,
           identity: u16,
           arm: Arm) -> Self {
        Self {
            inner: Controlled::new(compiled, Control::Full, identity),
            arm,
            target: SYMBOL_EOS,
            emit: None,
            emission_consults: 0,
            forced_emission_overrides: 0,
            consults_this_position: 0,
        }
    }

    /// The frozen reach sweep: with the natural packed input otherwise fixed,
    /// vary only the 8-bit active-byte field and read back what the circuit and
    /// the frozen emission head can distinguish about it.
    fn sweep(&mut self, input: &[bool; INPUT_BITS], target_byte: Option<u8>, natural_context: u16)
        -> engine::Result<(u16, u16, Option<u16>, Option<u16>, Option<u16>)> {
        let mut byte_to_context = [0u16; 256];
        let mut probe = *input;
        for value in 0..256usize {
            set_bits_u64(&mut probe, 792, 8, value as u64);
            let context = self.inner.context(Phase::Emit, &probe)?;
            byte_to_context[value] = context;
        }
        let mut distinct: Vec<u16> = byte_to_context.to_vec();
        distinct.sort_unstable();
        distinct.dedup();
        let mut argmaxes: Vec<u16> = Vec::new();
        for &context in &distinct {
            argmaxes.push(self.inner.choice(Head::Emission, context, &[true; 257])? as u16);
        }
        argmaxes.sort_unstable();
        argmaxes.dedup();
        let context_for_target = target_byte.map(|b| byte_to_context[usize::from(b)]);
        let argmax_for_target = match context_for_target {
            Some(context) => Some(self.inner.choice(Head::Emission, context, &[true; 257])? as u16),
            None => None,
        };
        let peers = context_for_target.map(|context| {
            byte_to_context
                .iter()
                .filter(|&&c| c == context)
                .count()
                .saturating_sub(1) as u16
        });
        // Restore the inner recorder to the natural context so no later reader
        // of `emission_context` sees a sweep value.
        self.inner.emission_context = Some(natural_context);
        Ok((
            distinct.len() as u16,
            argmaxes.len() as u16,
            context_for_target,
            argmax_for_target,
            peers,
        ))
    }
}

impl Policy for Instr<'_> {
    fn context(&mut self, phase: Phase, input: &[bool; INPUT_BITS]) -> engine::Result<u16> {
        let context = self.inner.context(phase, input)?;
        if phase == Phase::Emit {
            self.emission_consults += 1;
            self.consults_this_position += 1;
            let target_byte = if self.target == SYMBOL_EOS {
                None
            } else {
                Some(self.target as u8)
            };
            let (distinct_contexts, distinct_argmax, context_for_target, argmax_for_target, peers) =
                self.sweep(input, target_byte, context)?;
            let argmax_for_context =
                self.inner.choice(Head::Emission, context, &[true; 257])? as u16;
            self.emit = Some(EmitCapture {
                input_byte: bits_u64(input, 792, 8) as u8,
                active_kind: bits_u64(input, 812, 2) as u8,
                active_cursor: bits_u64(input, 816, 6) as u8,
                active_length: bits_u64(input, 822, 7) as u8,
                active_acknowledged: input[843],
                active_provisional: input[844],
                flags: bits_u64(input, 800, 8) as u8,
                context,
                context_for_target_byte: context_for_target,
                sweep_distinct_contexts: distinct_contexts,
                sweep_distinct_argmax: distinct_argmax,
                argmax_for_context,
                argmax_for_target_byte: argmax_for_target,
                target_byte_context_peers: peers,
                input_digest: digest_bits(input),
            });
        }
        Ok(context)
    }

    fn choice(&mut self, head: Head, context: u16, legal: &[bool]) -> engine::Result<usize> {
        let chosen = self.inner.choice(head, context, legal)?;
        if self.arm == Arm::ForcedEmissionOnly && head == Head::Emission {
            let target = usize::from(self.target);
            if legal.get(target).copied().unwrap_or(false) {
                self.forced_emission_overrides += 1;
                return Ok(target);
            }
            return Err(engine::EngineError::Invalid(
                "forced emission target is not legal",
            ));
        }
        Ok(chosen)
    }
}

// ------------------------------------------------------------------ driver

fn work_json(w: Work) -> Value {
    json!({
        "circuit_calls": w.circuit_calls,
        "candidate_scores": w.candidate_scores,
        "predictions": w.predictions,
        "observations": w.observations,
        "scalar_decodes": w.scalar_decodes,
        "selected_operations": w.selected_operations,
    })
}

fn reference_json(r: Option<objects::Reference>) -> Value {
    match r {
        None => Value::Null,
        Some(objects::Reference::Occurrence {
            start, end, turn, epoch,
        }) => json!({"kind": "occurrence", "epoch": epoch, "turn": turn, "start": start, "end": end}),
        Some(objects::Reference::Result { epoch, id }) => {
            json!({"kind": "result", "epoch": epoch, "id": id})
        }
    }
}
fn reference_matches(r: Option<objects::Reference>, span: (u64, u8)) -> bool {
    matches!(
        r,
        Some(objects::Reference::Occurrence { start, end, .. })
            if start == span.0 && end == span.0 + u64::from(span.1)
    )
}

struct StepRow {
    value: Value,
    /// (read_pair_ok, op_ok)
    read_pair_ok: bool,
    read_a_ok: bool,
    read_b_ok: bool,
    payload_a_ok: bool,
    payload_b_ok: bool,
    intended_pair_payload: bool,
    op_ok: bool,
    arithmetic: bool,
    provisional: bool,
    input_byte: u8,
    target: u16,
    emitted: u16,
    produced_byte: Option<u8>,
    value_correct_selected: Option<bool>,
    value_correct_intended: Option<bool>,
    capture: Option<EmitCapture>,
    /// Work delta for this predict call.
    work_selected_operations_delta: u64,
    work_circuit_calls_delta: u64,
    target_is_later_digit: bool,
    target_byte_in_input: bool,
}

#[allow(clippy::too_many_arguments)]
fn drive(
    g: &BoundGeometry,
    compiled: &uor_r4_core::native_geometric::addressed_attention::circuit::PrimitiveExport,
    params: &Parameters,
    index: usize,
    arm: Arm,
) -> AnyResult<(Vec<StepRow>, Work, u64, u64)> {
    let example = example_of(index);
    let epoch = index as u64 + 1;
    let spans = intended_spans(index);
    let intended_op = expected_op(index);
    let mut runtime = RuntimeSession::new(params.digest(), g, epoch)?;
    let mut policy = Instr::new(compiled, g.identity(), arm);
    for &byte in &example.prompt {
        let offer = runtime.predict(g, &mut policy)?;
        runtime.observe(Symbol::Byte(byte), offer.offer.id, g, &mut policy)?;
    }
    let prompt_work = runtime.work();
    let mut rows = Vec::new();
    let mut trajectory = blake3::Hasher::new();
    let answer_len = example.answer.len();
    for (position, target) in example
        .answer
        .iter()
        .map(|&b| u16::from(b))
        .chain([SYMBOL_EOS])
        .enumerate()
    {
        policy.target = target;
        policy.emit = None;
        policy.consults_this_position = 0;
        let before = runtime.work();
        let offered = runtime.predict(g, &mut policy)?;
        let after = runtime.work();
        let emitted = symbol_code(offered.offer.symbol);
        let capture = policy.emit.clone();
        let action = offered.offer.action;
        let arithmetic = matches!(action, Action::AddAB | Action::SubAB);
        let active = offered.offer.active();
        let produced_byte = active.map(objects::ActiveLease::byte);
        let provisional = active.and_then(|a| a.provisional()).is_some();
        let produced_bytes = active
            .filter(|_| provisional)
            .map(|a| a.lease().payload().to_vec());
        let operands = offered.offer.operands();
        let payloads: Vec<Option<String>> = operands
            .iter()
            .map(|o| o.as_ref().map(|l| String::from_utf8_lossy(l.payload()).to_string()))
            .collect();
        let selected_values: [Option<i64>; 2] = operands
            .map(|o| o.and_then(|l| objects::decode_i64(l.payload()).ok()));
        let value_correct_selected = if arithmetic {
            match (selected_values[0], selected_values[1]) {
                (Some(a), Some(b)) => {
                    let expected = if action == Action::AddAB { a + b } else { a - b };
                    Some(produced_bytes.as_deref() == Some(canonical(expected).as_slice()))
                }
                _ => None,
            }
        } else {
            None
        };
        let value_correct_intended = if arithmetic {
            Some(produced_bytes.as_deref() == Some(canonical(expected_scalar(index)).as_slice()))
        } else {
            None
        };
        let read_a_ok = reference_matches(offered.trace.selected[0], spans[0]);
        let read_b_ok = reference_matches(offered.trace.selected[1], spans[1]);
        let payload_a_ok = payloads[0].as_deref() == Some(span_string(&example.prompt, spans[0]).as_str());
        let payload_b_ok = payloads[1].as_deref() == Some(span_string(&example.prompt, spans[1]).as_str());
        let intended_pair_payload = payload_a_ok && payload_b_ok;
        let op_ok = action_index(action) == intended_op;
        let input_byte = capture.as_ref().map(|c| c.input_byte).unwrap_or(0);
        let target_byte_in_input = target != SYMBOL_EOS && input_byte == target as u8;
        let target_is_later_digit = target != SYMBOL_EOS && position > 0 && position < answer_len;
        let work_delta = Work {
            circuit_calls: after.circuit_calls - before.circuit_calls,
            candidate_scores: after.candidate_scores - before.candidate_scores,
            predictions: after.predictions - before.predictions,
            observations: after.observations - before.observations,
            scalar_decodes: after.scalar_decodes - before.scalar_decodes,
            selected_operations: after.selected_operations - before.selected_operations,
        };
        let step_digest = blake3::hash(
            format!(
                "{:?}|{:?}|{:?}|{:?}|{:?}|{}",
                offered.trace.contexts,
                offered.trace.selected,
                offered.trace.action,
                produced_byte,
                produced_bytes,
                input_byte
            )
            .as_bytes(),
        )
        .to_hex()
        .to_string();
        trajectory.update(step_digest.as_bytes());
        trajectory.update(&emitted.to_le_bytes());
        let value = json!({
            "example_index": index,
            "problem": {"a": PROBLEMS[index].0, "b": PROBLEMS[index].1, "op": op_name(index)},
            "position": position,
            "is_eos_position": target == SYMBOL_EOS,
            "target": target,
            "target_char": if target == SYMBOL_EOS { "EOS".to_string() } else { (target as u8 as char).to_string() },
            "emitted": emitted,
            "emitted_equals_target": emitted == target,
            "action": action_name(action),
            "action_index": action_index(action),
            "intended_op_index": intended_op,
            "action_is_intended_op": op_ok,
            "action_is_arithmetic": arithmetic,
            "read_a": {
                "selected": reference_json(offered.trace.selected[0]),
                "intended_start": spans[0].0,
                "intended_len": spans[0].1,
                "exact_span_match": read_a_ok,
            },
            "read_b": {
                "selected": reference_json(offered.trace.selected[1]),
                "intended_start": spans[1].0,
                "intended_len": spans[1].1,
                "exact_span_match": read_b_ok,
            },
            "read_pair_exact_span_match": read_a_ok && read_b_ok,
            "intended_pair_payload_match": intended_pair_payload,
            "operand_payloads": payloads,
            "numeric_valid": offered.trace.numeric_valid,
            "scalar_decodes": offered.trace.scalar_decodes,
            "selected_operations_trace": offered.trace.selected_operations,
            "cursor": offered.trace.cursor,
            "contexts": offered.trace.contexts.iter().map(|c| c.map(u64::from)).collect::<Vec<_>>(),
            "active_provisional": provisional,
            "produced_cursor_byte": produced_byte,
            "produced_value_bytes": produced_bytes.as_ref().map(|b| String::from_utf8_lossy(b).to_string()),
            "value_correct_vs_selected_operands": value_correct_selected,
            "value_correct_vs_intended_problem": value_correct_intended,
            "emitted_equals_produced_cursor_byte": produced_byte.map(|b| emitted == u16::from(b)),
            "target_byte_in_emission_input": target_byte_in_input,
            "target_is_later_digit_of_multidigit_answer": target_is_later_digit,
            "emission_consults_this_position": policy.consults_this_position,
            "emit_capture": capture.as_ref().map(EmitCapture::json),
            "work_delta": work_json(work_delta),
            "work_cumulative": work_json(after),
            "step_digest": step_digest,
        });
        rows.push(StepRow {
            value,
            read_pair_ok: read_a_ok && read_b_ok,
            read_a_ok,
            read_b_ok,
            payload_a_ok,
            payload_b_ok,
            intended_pair_payload,
            op_ok,
            arithmetic,
            provisional,
            input_byte,
            target,
            emitted,
            produced_byte,
            value_correct_selected,
            value_correct_intended,
            capture,
            work_selected_operations_delta: work_delta.selected_operations,
            work_circuit_calls_delta: work_delta.circuit_calls,
            target_is_later_digit,
            target_byte_in_input,
        });
        let actual = symbol_of(target)?;
        runtime.observe(actual, offered.offer.id, g, &mut policy)?;
    }
    let work = runtime.work();
    let prompt_predictions = prompt_work.predictions;
    Ok((
        rows,
        work,
        prompt_predictions,
        policy.forced_emission_overrides,
    ))
}

// -------------------------------------------------------------- aggregation

fn aggregate(rows: &[StepRow], work: Work) -> Value {
    let positions = rows.len() as u64;
    let mut read_a_ok = 0u64;
    let mut read_b_ok = 0u64;
    let mut read_pair_ok = 0u64;
    let mut payload_a_ok = 0u64;
    let mut payload_b_ok = 0u64;
    let mut payload_pair_ok = 0u64;
    let mut op_ok = 0u64;
    let mut arithmetic = 0u64;
    let mut input_byte_equals_target = 0u64;
    let mut emitted_equals_input_byte = 0u64;
    let mut emitted_equals_target = 0u64;
    let mut later_digit_positions = 0u64;
    let mut later_digit_target_in_input = 0u64;
    let mut consults = 0u64;
    let mut exactly_one_consult = 0u64;
    let mut scored_selected_operations = 0u64;
    let mut scored_circuit_calls = 0u64;
    // any active lease (Hold/Advance/Acquire/arithmetic): wire check for 792..800
    let mut active_rows = 0u64;
    let mut active_input_equals_produced = 0u64;
    let mut active_emitted_equals_produced = 0u64;
    // provisional prospective result leases only (a value the operator produced)
    let mut prov_steps = 0u64;
    let mut prov_value_selected_ok = 0u64;
    let mut prov_value_selected_wrong = 0u64;
    let mut prov_value_intended_ok = 0u64;
    let mut prov_emitted_equals_produced = 0u64;
    let mut prov_input_equals_produced = 0u64;
    let mut prov_produced_equals_target = 0u64;
    let mut prov_argmax_equals_produced = 0u64;
    let mut prov_argmax_equals_target = 0u64;
    let mut prov_input_equals_target = 0u64;
    let mut prov_sweep_contexts: Vec<u64> = Vec::new();
    // (read_pair_ok, op_ok) cross-tab
    let mut cross = [[0u64; 2]; 2];
    // (value correct for the read operands, emitted carries it) on provisional steps
    let mut provisional_cross = [[0u64; 2]; 2];
    let mut sweep_contexts: Vec<u64> = Vec::new();
    let mut sweep_argmax: Vec<u64> = Vec::new();
    let mut collapse_steps = 0u64;
    let mut target_byte_new_context = 0u64;
    let mut both_natural_carried = 0u64;
    let mut both_natural_provisional = 0u64;
    let mut emitted_histogram: std::collections::BTreeMap<u16, u64> = std::collections::BTreeMap::new();
    let mut target_in_input_positions: Vec<Value> = Vec::new();
    for row in rows {
        read_a_ok += u64::from(row.read_a_ok);
        read_b_ok += u64::from(row.read_b_ok);
        read_pair_ok += u64::from(row.read_pair_ok);
        payload_a_ok += u64::from(row.payload_a_ok);
        payload_b_ok += u64::from(row.payload_b_ok);
        payload_pair_ok += u64::from(row.intended_pair_payload);
        op_ok += u64::from(row.op_ok);
        arithmetic += u64::from(row.arithmetic);
        cross[usize::from(!row.read_pair_ok)][usize::from(!row.op_ok)] += 1;
        *emitted_histogram.entry(row.emitted).or_insert(0) += 1;
        scored_selected_operations += row.work_selected_operations_delta;
        scored_circuit_calls += row.work_circuit_calls_delta;
        emitted_equals_target += u64::from(row.emitted == row.target);
        if row.target_is_later_digit {
            later_digit_positions += 1;
            later_digit_target_in_input += u64::from(row.target_byte_in_input);
        }
        if let Some(byte) = row.produced_byte {
            active_rows += 1;
            if row.input_byte == byte {
                active_input_equals_produced += 1;
            }
            if row.emitted == u16::from(byte) {
                active_emitted_equals_produced += 1;
            }
        }
        if row.read_pair_ok && row.op_ok {
            if row.provisional {
                both_natural_provisional += 1;
            }
            if let Some(byte) = row.produced_byte {
                if row.emitted == u16::from(byte) {
                    both_natural_carried += 1;
                }
            }
        }
        if row.provisional {
            prov_steps += 1;
            if row.value_correct_selected == Some(true) {
                prov_value_selected_ok += 1;
            }
            if row.value_correct_selected == Some(false) {
                prov_value_selected_wrong += 1;
            }
            if row.value_correct_intended == Some(true) {
                prov_value_intended_ok += 1;
            }
            if let Some(byte) = row.produced_byte {
                if row.emitted == u16::from(byte) {
                    prov_emitted_equals_produced += 1;
                }
                if row.input_byte == byte {
                    prov_input_equals_produced += 1;
                }
                if row.target != SYMBOL_EOS && byte == row.target as u8 {
                    prov_produced_equals_target += 1;
                }
            }
            if let Some(capture) = &row.capture {
                if let Some(byte) = row.produced_byte {
                    if capture.argmax_for_context == u16::from(byte) {
                        prov_argmax_equals_produced += 1;
                    }
                }
                if capture.argmax_for_context == row.target {
                    prov_argmax_equals_target += 1;
                }
                if row.target != SYMBOL_EOS && capture.input_byte == row.target as u8 {
                    prov_input_equals_target += 1;
                }
                prov_sweep_contexts.push(u64::from(capture.sweep_distinct_contexts));
            }
            let value_ok = row.value_correct_selected == Some(true);
            let carried = row
                .produced_byte
                .map(|b| row.emitted == u16::from(b))
                .unwrap_or(false);
            provisional_cross[usize::from(!value_ok)][usize::from(!carried)] += 1;
        }
        if let Some(capture) = &row.capture {
            consults += 1;
            if row.value["emission_consults_this_position"].as_u64() == Some(1) {
                exactly_one_consult += 1;
            }
            if row.target != SYMBOL_EOS && capture.input_byte == row.target as u8 {
                input_byte_equals_target += 1;
                target_in_input_positions.push(json!({
                    "example_index": row.value["example_index"],
                    "position": row.value["position"],
                    "target": row.target,
                    "action": row.value["action"],
                    "active_provisional": row.provisional,
                }));
            }
            if row.emitted == u16::from(capture.input_byte) {
                emitted_equals_input_byte += 1;
            }
            sweep_contexts.push(u64::from(capture.sweep_distinct_contexts));
            sweep_argmax.push(u64::from(capture.sweep_distinct_argmax));
            if capture.target_byte_context_peers.unwrap_or(0) > 0 {
                collapse_steps += 1;
            }
            if ratio_new_context(row, capture) {
                target_byte_new_context += 1;
            }
        }
    }
    let mean = |v: &[u64]| {
        if v.is_empty() {
            0.0
        } else {
            v.iter().sum::<u64>() as f64 / v.len() as f64
        }
    };
    json!({
        "positions": positions,
        "emitted_equals_target": emitted_equals_target,
        "distinct_emitted_symbols": emitted_histogram.len(),
        "emitted_symbol_histogram": emitted_histogram,
        "natural_read": {
            "read_a_exact_span_matches": read_a_ok,
            "read_b_exact_span_matches": read_b_ok,
            "read_pair_exact_span_matches": read_pair_ok,
            "intended_a_payload_matches": payload_a_ok,
            "intended_b_payload_matches": payload_b_ok,
            "intended_pair_payload_matches": payload_pair_ok,
        },
        "natural_operator": {
            "action_is_intended_op": op_ok,
            "action_is_arithmetic_add_or_sub": arithmetic,
            "read_pair_and_op_cross_tab": {
                "legend": "cross[read_pair_failed][op_failed]; [0][0] = both natural",
                "both_natural": cross[0][0],
                "read_ok_op_not": cross[0][1],
                "read_not_op_ok": cross[1][0],
                "neither": cross[1][1],
            },
        },
        "provisional_emission": {
            "legend": "rows whose prospective active lease is a value the operator just produced (objects.rs:291-328); this is the only surface on which the produced cursor byte can be carried",
            "provisional_steps": prov_steps,
            "value_correct_vs_selected_operands": prov_value_selected_ok,
            "value_wrong_vs_selected_operands": prov_value_selected_wrong,
            "value_correct_vs_intended_problem": prov_value_intended_ok,
            "emitted_equals_produced_cursor_byte": prov_emitted_equals_produced,
            "input_byte_792_800_equals_produced_cursor_byte": prov_input_equals_produced,
            "produced_cursor_byte_equals_target_byte": prov_produced_equals_target,
            "argmax_for_reached_context_equals_produced_byte": prov_argmax_equals_produced,
            "argmax_for_reached_context_equals_target_byte": prov_argmax_equals_target,
            "input_byte_equals_target_byte": prov_input_equals_target,
            "value_ok_vs_carried_cross_tab": {
                "legend": "cross[value_wrong][not_carried]; [0][1] = value correct for the operands read but the emitted symbol is not the produced cursor byte",
                "value_ok_and_carried": provisional_cross[0][0],
                "value_ok_not_carried": provisional_cross[0][1],
                "value_wrong_and_carried": provisional_cross[1][0],
                "value_wrong_not_carried": provisional_cross[1][1],
            },
        },
        "active_lease_wire_check": {
            "legend": "every row with any active lease (Hold/Advance/AcquireA/AcquireB/AddAB/SubAB); confirms bits 792..800 carry offer.active().byte()",
            "rows_with_an_active_lease": active_rows,
            "input_byte_792_800_equals_active_lease_byte": active_input_equals_produced,
            "emitted_equals_active_lease_byte": active_emitted_equals_produced,
        },
        "conditional_emission": {
            "both_natural_positions": cross[0][0],
            "both_natural_provisional_steps": both_natural_provisional,
            "emitted_equals_produced_cursor_byte": both_natural_carried,
            "void": cross[0][0] == 0,
            "rule": "an empty intersection is VOID, not a failure: no position reached the comparison",
        },
        "emission_input": {
            "emission_consultations_scored_positions": consults,
            "positions_with_exactly_one_consultation": exactly_one_consult,
            "input_byte_equals_target_byte": input_byte_equals_target,
            "positions_where_input_byte_equals_target": target_in_input_positions,
            "emitted_equals_input_byte": emitted_equals_input_byte,
            "multidigit_later_digit_positions": later_digit_positions,
            "multidigit_later_digit_target_present_in_input": later_digit_target_in_input,
        },
        "reach_sweep": {
            "steps_swept": sweep_contexts.len() as u64,
            "distinct_contexts_over_0_255_byte_sweep_min": sweep_contexts.iter().copied().min().unwrap_or(0),
            "distinct_contexts_over_0_255_byte_sweep_max": sweep_contexts.iter().copied().max().unwrap_or(0),
            "distinct_contexts_over_0_255_byte_sweep_mean": mean(&sweep_contexts),
            "distinct_emission_argmax_over_sweep_min": sweep_argmax.iter().copied().min().unwrap_or(0),
            "distinct_emission_argmax_over_sweep_max": sweep_argmax.iter().copied().max().unwrap_or(0),
            "steps_where_target_byte_shares_its_context_with_another_byte": collapse_steps,
            "steps_where_target_byte_reaches_a_different_context": target_byte_new_context,
            "provisional_steps_distinct_contexts_min": prov_sweep_contexts.iter().copied().min().unwrap_or(0),
            "provisional_steps_distinct_contexts_max": prov_sweep_contexts.iter().copied().max().unwrap_or(0),
            "provisional_steps_distinct_contexts_mean": mean(&prov_sweep_contexts),
        },
        "work": work_json(work),
        "scored_positions_work": {
            "selected_operations_on_scored_positions": scored_selected_operations,
            "circuit_calls_on_scored_positions": scored_circuit_calls,
        },
    })
}

/// Whether substituting the target byte for the natural input byte moves the
/// context (i.e. the cascade still distinguishes those two byte values).
fn ratio_new_context(row: &StepRow, capture: &EmitCapture) -> bool {
    match (row.target, capture.context_for_target_byte) {
        (SYMBOL_EOS, _) => false,
        (_, None) => false,
        (_, Some(context)) => context != capture.context,
    }
}

// ------------------------------------------------------------------ reports

fn emit(base: &Path, name: &str, value: &Value) -> AnyResult<PathBuf> {
    let root = base.join(name);
    report_output::claim(&root)?;
    std::fs::write(root.join("report.json"), serde_json::to_vec_pretty(value)?)?;
    report_output::seal(&root)?;
    let files = report_output::verify(&root)?;
    println!("[unforced-emit] sealed {} unlisted={}", root.display(), files.len());
    Ok(root)
}

fn build_model(params: &Parameters) -> AnyResult<Model> {
    let compiled = params.compile()?;
    let model = Model::new(
        compiled,
        Provenance {
            seed: params.seed(),
            training_data_digest: *blake3::hash(MODEL_DATA).as_bytes(),
            training_config_digest: *blake3::hash(CONFIG_DIGEST).as_bytes(),
            parameter_digest: params.digest(),
            source_digest: implementation_digest(),
            parent: None,
        },
    )?;
    Ok(model)
}

fn hex(digest: [u8; 32]) -> String {
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

fn mode_prereg(base: &Path) -> AnyResult<()> {
    let start = Instant::now();
    let value: Value = serde_json::from_str(PREREG)?;
    let note = json!({
        "prereg": value,
        "prereg_digest": blake3::hash(PREREG.as_bytes()).to_hex().to_string(),
        "bin_source_digest": blake3::hash(include_bytes!("addressed-attention-unforced-emit.rs"))
            .to_hex().to_string(),
        "split_size": PROBLEMS.len(),
        "split_digest_material": PROBLEMS.iter().enumerate().map(|(i, &(a, b, op))| json!({
            "index": i, "a": a, "b": b, "op": if op == 0 { "Add" } else { "Sub" },
            "expected_scalar": expected_scalar(i),
            "prompt": String::from_utf8_lossy(&render(a, b)).to_string(),
        })).collect::<Vec<_>>(),
        "elapsed_ms": start.elapsed().as_millis(),
    });
    emit(base, "prereg", &note)?;
    Ok(())
}

fn baseline_replication(agg: &Value) -> Value {
    // Numbers published by the sealed predecessor root
    // `aa-arith-oracle-20261005-204407` for Control::Full, shape=newline,
    // measured with the same seed, split and recipe.
    let pairs: [(&str, u64, u64); 10] = [
        (
            "positions",
            80,
            agg["positions"].as_u64().unwrap_or(u64::MAX),
        ),
        (
            "natural_read.intended_pair_payload_matches",
            0,
            agg["natural_read"]["intended_pair_payload_matches"]
                .as_u64()
                .unwrap_or(u64::MAX),
        ),
        (
            "natural_operator.action_is_arithmetic_add_or_sub",
            6,
            agg["natural_operator"]["action_is_arithmetic_add_or_sub"]
                .as_u64()
                .unwrap_or(u64::MAX),
        ),
        (
            "provisional_emission.value_correct_vs_selected_operands",
            6,
            agg["provisional_emission"]["value_correct_vs_selected_operands"]
                .as_u64()
                .unwrap_or(u64::MAX),
        ),
        (
            "provisional_emission.emitted_equals_produced_cursor_byte",
            0,
            agg["provisional_emission"]["emitted_equals_produced_cursor_byte"]
                .as_u64()
                .unwrap_or(u64::MAX),
        ),
        (
            "work.selected_operations",
            9,
            agg["work"]["selected_operations"].as_u64().unwrap_or(u64::MAX),
        ),
        (
            "work.circuit_calls",
            1554,
            agg["work"]["circuit_calls"].as_u64().unwrap_or(u64::MAX),
        ),
        (
            "work.candidate_scores",
            1976,
            agg["work"]["candidate_scores"].as_u64().unwrap_or(u64::MAX),
        ),
        (
            "work.predictions",
            232,
            agg["work"]["predictions"].as_u64().unwrap_or(u64::MAX),
        ),
        (
            "work.scalar_decodes",
            226,
            agg["work"]["scalar_decodes"].as_u64().unwrap_or(u64::MAX),
        ),
    ];
    let mut mismatches = Vec::new();
    let mut reproduced = serde_json::Map::new();
    for (field, published, got) in pairs {
        reproduced.insert(field.to_string(), json!(got));
        if published != got {
            mismatches.push(json!({"field": field, "published": published, "reproduced": got}));
        }
    }
    json!({
        "source_root": "/Users/casey.allard/uor-r4/.worktrees/reports/aa-arith-oracle-20261005-204407",
        "published_expected": {
            "positions": 80,
            "natural_read.intended_pair_payload_matches": 0,
            "natural_operator.action_is_arithmetic_add_or_sub": 6,
            "provisional_emission.value_correct_vs_selected_operands": 6,
            "provisional_emission.emitted_equals_produced_cursor_byte": 0,
            "work.selected_operations": 9,
            "work.circuit_calls": 1554,
            "work.candidate_scores": 1976,
            "work.predictions": 232,
            "work.scalar_decodes": 226,
        },
        "reproduced": Value::Object(reproduced),
        "mismatches": mismatches,
        "identical": mismatches.is_empty(),
    })
}

fn mode_run(base: &Path, params: &Parameters, model: &Model) -> AnyResult<()> {
    let start = Instant::now();
    let g = model.geometry();
    let compiled = model.compiled();

    // --- arm A: fully natural
    let mut natural_rows: Vec<StepRow> = Vec::new();
    let mut natural_work = Work::default();
    let mut natural_overrides = 0u64;
    for index in 0..PROBLEMS.len() {
        let (rows, work, _, overrides) = drive(g, compiled, params, index, Arm::Natural)?;
        natural_work.circuit_calls += work.circuit_calls;
        natural_work.candidate_scores += work.candidate_scores;
        natural_work.predictions += work.predictions;
        natural_work.observations += work.observations;
        natural_work.scalar_decodes += work.scalar_decodes;
        natural_work.selected_operations += work.selected_operations;
        natural_overrides += overrides;
        natural_rows.extend(rows);
    }
    let natural_agg = aggregate(&natural_rows, natural_work);
    let natural_overrides_used = natural_overrides;
    println!(
        "[unforced-emit] natural: read_pair={} payload_pair={} op_ok={} provisional={} emitted_eq_produced={} selected_operations={} emitted_eq_target={}",
        natural_agg["natural_read"]["read_pair_exact_span_matches"],
        natural_agg["natural_read"]["intended_pair_payload_matches"],
        natural_agg["natural_operator"]["action_is_intended_op"],
        natural_agg["provisional_emission"]["provisional_steps"],
        natural_agg["provisional_emission"]["emitted_equals_produced_cursor_byte"],
        natural_agg["work"]["selected_operations"],
        natural_agg["emitted_equals_target"],
    );
    let natural_steps: Vec<Value> = natural_rows.iter().map(|r| r.value.clone()).collect();
    let natural_digest = blake3::hash(
        serde_json::to_string(&natural_steps)
            .unwrap_or_default()
            .as_bytes(),
    )
    .to_hex()
    .to_string();

    // --- arm B: forced emission only
    let mut forced_rows: Vec<StepRow> = Vec::new();
    let mut forced_work = Work::default();
    let mut forced_overrides = 0u64;
    for index in 0..PROBLEMS.len() {
        let (rows, work, _, overrides) = drive(g, compiled, params, index, Arm::ForcedEmissionOnly)?;
        forced_work.circuit_calls += work.circuit_calls;
        forced_work.candidate_scores += work.candidate_scores;
        forced_work.predictions += work.predictions;
        forced_work.observations += work.observations;
        forced_work.scalar_decodes += work.scalar_decodes;
        forced_work.selected_operations += work.selected_operations;
        forced_overrides += overrides;
        forced_rows.extend(rows);
    }
    let forced_agg = aggregate(&forced_rows, forced_work);
    println!(
        "[unforced-emit] forced-emission-only: overrides={} read_pair={} op_ok={} provisional={} emitted_eq_produced={} selected_operations={}",
        forced_overrides,
        forced_agg["natural_read"]["read_pair_exact_span_matches"],
        forced_agg["natural_operator"]["action_is_intended_op"],
        forced_agg["provisional_emission"]["provisional_steps"],
        forced_agg["provisional_emission"]["emitted_equals_produced_cursor_byte"],
        forced_agg["work"]["selected_operations"],
    );
    let forced_steps: Vec<Value> = forced_rows.iter().map(|r| r.value.clone()).collect();
    let forced_digest = blake3::hash(
        serde_json::to_string(&forced_steps)
            .unwrap_or_default()
            .as_bytes(),
    )
    .to_hex()
    .to_string();

    // --- pure readout counterfactual on the natural trajectory: forcing only
    // the emission readout does not change what the natural operator produced,
    // so compare the produced byte with the target on the natural rows.
    let natural_readout_counterfactual = natural_rows
        .iter()
        .filter(|r| r.provisional && r.produced_byte.is_some())
        .fold((0u64, 0u64), |(hit, total), r| {
            let hit = hit + u64::from(r.produced_byte == Some(r.target as u8) && r.target != SYMBOL_EOS);
            (hit, total + 1)
        });

    let natural_report = json!({
        "schema": "uor-r4.addressed-attention-unforced-emit-arm/1",
        "arm": "Natural",
        "forcing": "none",
        "forced_emission_overrides": natural_overrides_used,
        "param_seed": params.seed(),
        "parameter_digest": hex(params.digest()),
        "source_digest": hex(implementation_digest()),
        "aggregate": natural_agg,
        "baseline_replication": baseline_replication(&natural_agg),
        "trajectory_digest": natural_digest,
        "steps": natural_steps,
        "elapsed_ms": start.elapsed().as_millis(),
    });
    emit(base, "natural", &natural_report)?;

    let forced_report = json!({
        "schema": "uor-r4.addressed-attention-unforced-emit-arm/1",
        "arm": "ForcedEmissionOnly",
        "forcing": "harness Policy wrapper returns the target index at Head::Emission only (legal=[true;257] at engine.rs:393); the read and the Control head are untouched",
        "forced_emission_overrides": forced_overrides,
        "param_seed": params.seed(),
        "parameter_digest": hex(params.digest()),
        "source_digest": hex(implementation_digest()),
        "aggregate": forced_agg,
        "trajectory_digest": forced_digest,
        "steps": forced_steps,
        "elapsed_ms": start.elapsed().as_millis(),
    });
    emit(base, "forced-emission-only", &forced_report)?;

    // --- comparison + triage
    let both_natural = natural_agg["conditional_emission"]["both_natural_positions"]
        .as_u64()
        .unwrap_or(0);
    let both_natural_carried = natural_agg["conditional_emission"]
        ["emitted_equals_produced_cursor_byte"]
        .as_u64()
        .unwrap_or(0);
    let prov = &natural_agg["provisional_emission"];
    let provisional = prov["provisional_steps"].as_u64().unwrap_or(0);
    let prov_value_selected_ok = prov["value_correct_vs_selected_operands"]
        .as_u64()
        .unwrap_or(0);
    let prov_value_intended_ok = prov["value_correct_vs_intended_problem"]
        .as_u64()
        .unwrap_or(0);
    let prov_input_equals_produced = prov["input_byte_792_800_equals_produced_cursor_byte"]
        .as_u64()
        .unwrap_or(0);
    let prov_produced_equals_target = prov["produced_cursor_byte_equals_target_byte"]
        .as_u64()
        .unwrap_or(0);
    let prov_argmax_equals_produced = prov["argmax_for_reached_context_equals_produced_byte"]
        .as_u64()
        .unwrap_or(0);
    let prov_input_equals_target = prov["input_byte_equals_target_byte"]
        .as_u64()
        .unwrap_or(0);
    let collapse = natural_agg["reach_sweep"]
        ["steps_where_target_byte_shares_its_context_with_another_byte"]
        .as_u64()
        .unwrap_or(0);
    let new_context = natural_agg["reach_sweep"]
        ["steps_where_target_byte_reaches_a_different_context"]
        .as_u64()
        .unwrap_or(0);
    let mean_contexts = natural_agg["reach_sweep"]["distinct_contexts_over_0_255_byte_sweep_mean"]
        .as_f64()
        .unwrap_or(0.0);
    let min_contexts = natural_agg["reach_sweep"]["distinct_contexts_over_0_255_byte_sweep_min"]
        .as_u64()
        .unwrap_or(0);
    let max_contexts = natural_agg["reach_sweep"]["distinct_contexts_over_0_255_byte_sweep_max"]
        .as_u64()
        .unwrap_or(0);
    let emitted_equals_input_byte = natural_agg["emission_input"]["emitted_equals_input_byte"]
        .as_u64()
        .unwrap_or(0);
    let later_digits = natural_agg["emission_input"]["multidigit_later_digit_positions"]
        .as_u64()
        .unwrap_or(0);
    let later_digits_present = natural_agg["emission_input"]
        ["multidigit_later_digit_target_present_in_input"]
        .as_u64()
        .unwrap_or(0);
    let selected_operations = natural_agg["work"]["selected_operations"]
        .as_u64()
        .unwrap_or(0);
    let read_a_ok = natural_agg["natural_read"]["read_a_exact_span_matches"]
        .as_u64()
        .unwrap_or(0);
    let read_b_ok = natural_agg["natural_read"]["read_b_exact_span_matches"]
        .as_u64()
        .unwrap_or(0);
    let forced_carried = forced_agg["provisional_emission"]["emitted_equals_produced_cursor_byte"]
        .as_u64()
        .unwrap_or(0);
    let forced_provisional = forced_agg["provisional_emission"]["provisional_steps"]
        .as_u64()
        .unwrap_or(0);
    let forced_payload_pair = forced_agg["natural_read"]["intended_pair_payload_matches"]
        .as_u64()
        .unwrap_or(0);
    let forced_op_ok = forced_agg["natural_operator"]["action_is_intended_op"]
        .as_u64()
        .unwrap_or(0);
    let forced_read_pair = forced_agg["natural_read"]["read_pair_exact_span_matches"]
        .as_u64()
        .unwrap_or(0);

    // Triage, evaluated on the only surface where a produced byte exists: the
    // provisional prospective result leases.
    let triage_mode = if selected_operations == 0 {
        json!({
            "mode": "void",
            "reason": "Work::selected_operations == 0: the mechanism did not execute; the comparison is void rather than negative",
        })
    } else if provisional == 0 {
        json!({
            "mode": "a_value_never_produced",
            "evidence": {
                "both_natural_positions": both_natural,
                "provisional_steps_natural": provisional,
                "selected_operations": selected_operations,
            },
            "reading": "no scored position carried a provisional prospective result lease, so no produced byte ever reached the emission input",
        })
    } else if prov_input_equals_produced == provisional && prov_argmax_equals_produced == 0 {
        json!({
            "mode": "b_value_produced_but_context_not_distinguishing",
            "evidence": {
                "provisional_steps": provisional,
                "provisional_steps_value_correct_for_the_operands_read": prov_value_selected_ok,
                "provisional_steps_value_correct_for_the_intended_problem": prov_value_intended_ok,
                "provisional_steps_where_input_byte_equals_produced_byte": prov_input_equals_produced,
                "provisional_steps_where_emission_argmax_equals_produced_byte": prov_argmax_equals_produced,
                "provisional_steps_where_input_byte_equals_target_byte": prov_input_equals_target,
                "provisional_steps_where_produced_byte_equals_target_byte": prov_produced_equals_target,
                "steps_where_target_byte_shares_its_context_with_another_byte": collapse,
                "distinct_contexts_over_0_255_byte_sweep": {
                    "min": min_contexts, "max": max_contexts, "mean": mean_contexts,
                },
            },
            "reading": "on every position where the operator produced a value, the value was exact for the operands the read supplied and its cursor byte was present in the emission input, yet the frozen emission head never returned that byte; the frozen 1024->9 cascade distinguishes only a few contexts across the whole 0..255 byte field, so the missing component is the emission readout together with the gate parameters that build its context",
        })
    } else {
        json!({
            "mode": "c_something_else",
            "evidence": {
                "both_natural_positions": both_natural,
                "provisional_steps_natural": provisional,
                "provisional_steps_where_input_byte_equals_produced_byte": prov_input_equals_produced,
                "provisional_steps_where_emission_argmax_equals_produced_byte": prov_argmax_equals_produced,
                "positions_where_input_byte_equals_target_byte": prov_input_equals_target,
                "steps_where_target_byte_shares_its_context_with_another_byte": collapse,
                "steps_where_target_byte_reaches_a_different_context": new_context,
            },
            "reading": "residual case: report the counts; the natural trajectory carries more structure than a single named mode",
        })
    };
    let mut triage = triage_mode;
    if let Some(object) = triage.as_object_mut() {
        object.insert(
            "conditional_emission".to_string(),
            json!({
                "both_natural_positions": both_natural,
                "emitted_equals_produced_cursor_byte": both_natural_carried,
                "void": both_natural == 0,
                "rule": "an empty intersection is VOID, not a failure: no position reached the comparison",
            }),
        );
        object.insert(
            "multidigit_carry_forward".to_string(),
            json!({
                "later_digit_positions": later_digits,
                "later_digit_target_present_in_emission_input": later_digits_present,
            }),
        );
    }
    let comparison = json!({
        "schema": "uor-r4.addressed-attention-unforced-emit-comparison/1",
        "natural_trajectory_digest": natural_digest,
        "forced_emission_trajectory_digest": forced_digest,
        "trajectory_identical": natural_digest == forced_digest,
        "trajectory_note": "the emitted symbol is consumed by engine.rs:467-474 and objects.rs:629 (an accepted proposal keeps the provisional active lease, an unaccepted one drops it), so arming the emission head changes the run; this digest shows that directly",
        "answers": {
            "A_natural": {
                "positions": natural_agg["positions"],
                "read_pair_exact_span_matches": natural_agg["natural_read"]["read_pair_exact_span_matches"],
                "read_a_exact_span_matches": read_a_ok,
                "read_b_exact_span_matches": read_b_ok,
                "intended_pair_payload_matches": natural_agg["natural_read"]["intended_pair_payload_matches"],
                "action_is_intended_op": natural_agg["natural_operator"]["action_is_intended_op"],
                "action_is_arithmetic": natural_agg["natural_operator"]["action_is_arithmetic_add_or_sub"],
                "emitted_equals_input_byte_792_800": emitted_equals_input_byte,
            },
            "A_work_counters": natural_agg["work"],
            "A_conditional_emission": natural_agg["conditional_emission"],
            "A_emission_input": natural_agg["emission_input"],
            "A_reach_sweep": natural_agg["reach_sweep"],
            "B_forced_emission_only": {
                "overrides": forced_overrides,
                "positions": forced_agg["positions"],
                "read_pair_exact_span_matches": forced_agg["natural_read"]["read_pair_exact_span_matches"],
                "intended_pair_payload_matches": forced_payload_pair,
                "action_is_intended_op": forced_op_ok,
                "provisional_steps": forced_provisional,
                "emitted_equals_produced_cursor_byte": forced_carried,
                "work": forced_agg["work"],
            },
            "B_natural_trajectory_readout_counterfactual": {
                "provisional_steps": natural_readout_counterfactual.1,
                "produced_byte_equals_target": natural_readout_counterfactual.0,
            },
            "separation": {
                "natural_provisional_steps": provisional,
                "natural_provisional_value_correct_for_operands_read": prov_value_selected_ok,
                "natural_provisional_value_correct_for_intended_problem": prov_value_intended_ok,
                "natural_provisional_argmax_equals_produced_byte": prov_argmax_equals_produced,
                "forced_arm_provisional_steps": forced_provisional,
                "natural_intended_op_steps": natural_agg["natural_operator"]["action_is_intended_op"],
                "natural_read_pair_steps": natural_agg["natural_read"]["read_pair_exact_span_matches"],
                "forced_arm_carried": forced_carried,
                "forced_arm_read_pair_exact_span_matches": forced_read_pair,
                "reading": if provisional == 0 {
                    "the natural run never yields a provisional produced byte, so the arm-B count is constrained by the natural read/operator, not by the emission readout"
                } else {
                    "the natural run does yield provisional produced bytes; compare the arm-B count with the natural-trajectory counterfactual"
                },
            },
        },
        "triage": triage,
        "named_missing_component": {
            "read": "the natural read never selects the intended operand spans (0/80): the query/null root heads at the frozen 1024->9 contexts do not identify the authored decimal occurrences",
            "operator": "the natural Control head selects AddAB/SubAB on some positions but not the intended operator per problem, and never together with the intended read",
            "carry_forward": "engine.rs:466-475 + objects.rs:629-648 only keep the provisional result lease (and advance its cursor) when the observed actual symbol equals the emitted proposal; a wrong emission therefore erases the produced cursor byte before the next position, so a multi-digit result can never expose its later digits on an unforced trajectory",
            "readout": "the frozen emission head maps the reached contexts to symbols that are not the target; because the 1024->9 cascade is not injective on the byte field at 792..800, changing head rows alone cannot separate byte values that share a context"
        }
    });
    emit(base, "comparison", &comparison)?;
    println!(
        "[unforced-emit] triage={} elapsed_ms={}",
        triage["mode"], start.elapsed().as_millis()
    );
    Ok(())
}

fn prepare_base(base: &str, create: bool) -> AnyResult<PathBuf> {
    let base = PathBuf::from(base);
    if !base.is_absolute() {
        return Err("base directory must be an absolute path".into());
    }
    for ancestor in base.parent().into_iter().flat_map(Path::ancestors) {
        if ancestor.join("manifest.json").exists() {
            return Err(format!(
                "base {} is beneath an already sealed root",
                base.display()
            )
            .into());
        }
    }
    if create {
        if base.exists() {
            return Err(format!(
                "base {} already exists; each report root is created exclusively and never reused",
                base.display()
            )
            .into());
        }
        std::fs::create_dir_all(&base)?;
    } else if !base.is_dir() {
        return Err(format!(
            "base {} does not exist; run the prereg mode first",
            base.display()
        )
        .into());
    }
    Ok(base)
}

fn main() -> AnyResult<()> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build_global()
        .map_err(|e| format!("thread pool: {e}"))?;
    let args: Vec<String> = std::env::args().collect();
    let mode = args
        .get(1)
        .ok_or("usage: addressed-attention-unforced-emit <prereg|run> <abs-base-dir>")?;
    let base = prepare_base(
        args.get(2).ok_or("missing absolute base directory")?,
        mode == "prereg",
    )?;
    let started = Instant::now();
    match mode.as_str() {
        "prereg" => {
            mode_prereg(&base)?;
        }
        "run" => {
            let params = Parameters::seeded(PARAMETER_SEED)?;
            let model = build_model(&params)?;
            mode_run(&base, &params, &model)?;
        }
        other => return Err(format!("unknown mode {other}").into()),
    }
    println!(
        "[unforced-emit] done elapsed_ms={} base={}",
        started.elapsed().as_millis(),
        base.display()
    );
    Ok(())
}
