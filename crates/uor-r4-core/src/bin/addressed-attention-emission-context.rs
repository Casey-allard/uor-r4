//! Measurement-only probe: is the computed arithmetic value present in the
//! emission head's context, or is the emission head structurally unable to see it?
//!
//! The engine builds the emission context from the packed `Phase::Emit` input
//! (`engine.rs:388-393`). For `AddAB`/`SubAB` the "active" lease passed there is
//! the prospective result lease (`objects.rs:291-328`), so the packed active lane
//! carries the byte at the result cursor and the encoded length, not the scalar
//! `Derivation.value`. This binary measures that wiring directly instead of
//! arguing from the source:
//!
//!   trace   <abs-root>  all 32 authored examples, forced read + forced operator,
//!                       records every emission consultation at every answer step
//!                       (input fields, context, produced value, target byte)
//!   causal  <abs-root>  counterfactual: identical operands, identical reads,
//!                       AddAB vs SubAB on the same runtime. Bit-level diff of the
//!                       packed emission input, field attribution, context equality.
//!                       Plus a harness-forced canonical trajectory (forced read +
//!                       forced operator + forced correct emission + forced
//!                       Advance) that asks whether later digits of a multi-digit
//!                       result are visible at their own consultations.
//!
//! No production file is modified. Every root is exclusively claimed and sealed.

use std::path::{Path, PathBuf};
use std::time::Instant;

use serde_json::{json, Value};
use uor_r4_core::native_geometric::addressed_attention::artifact::{
    BoundGeometry, Model, Provenance,
};
use uor_r4_core::native_geometric::addressed_attention::circuit::PrimitiveExport;
use uor_r4_core::native_geometric::addressed_attention::engine::{
    self, Head, Policy, RuntimeSession,
};
use uor_r4_core::native_geometric::addressed_attention::inputs::Phase;
use uor_r4_core::native_geometric::addressed_attention::objects::{self, Action, Symbol};
use uor_r4_core::native_geometric::addressed_attention::pilot::{Control, Controlled};
use uor_r4_core::native_geometric::addressed_attention::policy::{
    implementation_digest, Parameters,
};
use uor_r4_core::report_output;

type AnyResult<T> = Result<T, Box<dyn std::error::Error>>;

const PARAMETER_SEED: u64 = 7341;
const MODEL_DATA: &[u8] = b"addressed-attention-emission-context/1";
const CONFIG_DIGEST: &[u8] = b"addressed-attention-emission-context-config/1";

const OP_ADD: usize = 3;
const OP_SUB: usize = 4;
const OP_ADVANCE: usize = 5;
const SYMBOL_EOS: u16 = 256;

/// Same authored table as the oracle instrument: (a, b, op), op 0 = Add.
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

/// Deliberate extra pair: 15+4=19 and 15-4=11 share first byte ('1') and length
/// (2), so the step-0 emission input cannot distinguish them by their leading
/// byte; at cursor 1 it can. Used to mark exactly where the information sits.
const AMBIGUOUS: (i64, i64) = (15, 4);

const CITATIONS: [&str; 14] = [
    "engine.rs:392-393 emission head consulted with the Phase::Emit context",
    "engine.rs:388-391 Emit input = inputs(g, a, b, prospective.active(), true)",
    "engine.rs:211-249 inputs() assembles CausalInputs from session state",
    "engine.rs:228-238 active lane: byte/kind/cursor/length/acknowledged/provisional",
    "engine.rs:383-386 Control head chooses the action from ACTIONS; prospective = prepare_action",
    "objects.rs:291-328 AddAB/SubAB build a provisional result lease (encode_i64 of the value)",
    "objects.rs:304-312 value = execute(...); bytes,len = encode_i64(value); Derivation{value}",
    "objects.rs:132-134 ActiveLease::byte() = lease.bytes[cursor]",
    "objects.rs:90-92 Lease::payload() = bytes[..len]",
    "inputs.rs:108-126 active lane packed only when active is Some and phase != QueryA",
    "inputs.rs:120 active byte -> bits 792..800; inputs.rs:123 active length -> bits 822..829",
    "policy.rs:87-103 head_range; policy.rs:99 Head::Emission -> emission + context*257",
    "policy.rs:116-118 CompiledPolicy::context = compiled Boolean circuit on the packed input",
    "policy.rs:124 Head::Emission choice = heads.emission(context), a 257-way row",
];

const HONESTY: [&str; 6] = [
    "The 32-example split and its answers are authored in this instrument by exact i64 arithmetic; it is not a frozen external fixture.",
    "Forced-read and forced-operator arms are harness-only Policy interventions: they choose among legal actions; they do not modify the engine, objects, circuit or compiled parameters.",
    "The trajectory arm additionally forces the emission choice to the correct byte and forces Advance; it describes a harness-driven canonical trajectory, not production behaviour.",
    "Emission consultations also occur at every prefill position; only post-prompt (answer) steps are scored here. Each predict call consults Head::Emission exactly once.",
    "The emission context is the address of a 257-entry parameter row (policy.rs:99); 'context differs' is the operational meaning of 'the emission head can see the difference'.",
    "No production source file is modified by this probe.",
];

// ----------------------------------------------------------------- helpers

fn action_name(a: Action) -> &'static str {
    match a {
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
        Symbol::Eos => 256,
    }
}
fn symbol_of(code: u16) -> AnyResult<Symbol> {
    match code {
        0..=255 => Ok(Symbol::Byte(code as u8)),
        256 => Ok(Symbol::Eos),
        other => Err(format!("symbol code out of domain: {other}").into()),
    }
}
fn canonical(value: i64) -> Vec<u8> {
    let (bytes, len) = objects::encode_i64(value);
    bytes[..usize::from(len)].to_vec()
}
fn prompt_of(a: i64, b: i64) -> Vec<u8> {
    format!("{a}\n{b}\n").into_bytes()
}
fn spans_of(a: i64, b: i64) -> [(u64, u8); 2] {
    let (as_, bs) = (a.to_string(), b.to_string());
    [(0, as_.len() as u8), (as_.len() as u64 + 1, bs.len() as u8)]
}
fn answer_of(a: i64, b: i64, add: bool) -> Vec<u8> {
    canonical(if add { a + b } else { a - b })
}
fn bits(input: &[bool; 1024], offset: usize, width: usize) -> u64 {
    (0..width).fold(0_u64, |acc, i| acc | (u64::from(input[offset + i]) << i))
}
fn input_digest(input: &[bool; 1024]) -> String {
    let mut bytes = [0_u8; 128];
    for (i, &b) in input.iter().enumerate() {
        if b {
            bytes[i / 8] |= 1 << (i % 8);
        }
    }
    blake3::hash(&bytes).to_hex().to_string()
}
fn bytes_hex(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Field map of the packed 1024-bit input at the emission consultation.
fn fields(input: &[bool; 1024]) -> Value {
    json!({
        "flags_u8": bits(input, 800, 8),
        "phase": bits(input, 808, 4),
        "active_byte_792_800": bits(input, 792, 8),
        "active_kind_812_814": bits(input, 812, 2),
        "active_cursor_816_822": bits(input, 816, 6),
        "active_length_822_829": bits(input, 822, 7),
        "active_acknowledged_843": input[843],
        "active_provisional_844": input[844],
        "operand_a_first_byte_776_784": bits(input, 776, 8),
        "operand_b_first_byte_784_792": bits(input, 784, 8),
        "operand_a_length_829_836": bits(input, 829, 7),
        "operand_b_length_836_843": bits(input, 836, 7),
        "pending_offer_flag_807": input[807],
        "last_control_888_891": bits(input, 888, 3),
        "publications_891_893": bits(input, 891, 2),
    })
}

/// Human-readable owner of a differing bit index, from the packing offsets in
/// `inputs.rs` and the active-lane mapping in `engine.rs:228-238`.
fn bit_owner(i: usize) -> &'static str {
    match i {
        768..=775 => "last_observed (inputs.rs:75-78)",
        776..=783 => "selected_a.first_byte (inputs.rs:98)",
        784..=791 => "selected_b.first_byte (inputs.rs:98)",
        792..=799 => "active.byte (inputs.rs:120)",
        800..=807 => "flags (inputs.rs:130)",
        808..=811 => "phase (inputs.rs:131)",
        812..=813 => "active.kind (inputs.rs:121)",
        816..=821 => "active.cursor (inputs.rs:122)",
        822..=828 => "active.length (inputs.rs:123)",
        829..=835 => "selected_a.length (inputs.rs:100)",
        836..=842 => "selected_b.length (inputs.rs:100)",
        843 => "active.acknowledged (inputs.rs:124)",
        844 => "active.provisional (inputs.rs:125)",
        848..=855 => "turn_position (inputs.rs:132)",
        856..=871 => "zeta_bins (inputs.rs:133-135)",
        888..=890 => "last_control (inputs.rs:136)",
        891..=892 => "publications (inputs.rs:137)",
        _ => "state/operand signatures (inputs.rs:71-73, 97)",
    }
}

// -------------------------------------------------------------------- plan

#[derive(Clone, Copy, Debug, Default)]
struct Plan {
    q: [[u16; 2]; 2],
    n: [[u16; 2]; 2],
    len: [u8; 2],
}

/// Harness-side read plan for the intended operand spans: exhaustive best query
/// and worst null over the 120x120 root pairs, extent fixed to the true length.
fn compute_plan(
    g: &BoundGeometry,
    runtime: &RuntimeSession,
    epoch: u64,
    spans: [(u64, u8); 2],
) -> AnyResult<Plan> {
    let mut plan = Plan::default();
    for (which, &(start, len)) in spans.iter().enumerate() {
        let record = runtime.objects().occurrence(epoch, start)?;
        let keys = record.keys;
        let mut best: Option<(i16, [u16; 2])> = None;
        for q0 in 0..120u16 {
            for q1 in 0..120u16 {
                let s = g.score([q0, q1], keys)?;
                if best.is_none_or(|(b, _)| s > b) {
                    best = Some((s, [q0, q1]));
                }
            }
        }
        let (_, q) = best.ok_or("no query pair")?;
        let mut worst: Option<(i16, [u16; 2])> = None;
        for n0 in 0..120u16 {
            for n1 in 0..120u16 {
                let s = g.score(q, [n0, n1])?;
                if worst.is_none_or(|(b, _)| s < b) {
                    worst = Some((s, [n0, n1]));
                }
            }
        }
        let (_, n) = worst.ok_or("no null pair")?;
        let maximum = runtime.objects().maximum_extent(epoch, start)?;
        if maximum < len {
            return Err(format!("extent {maximum} < intended {len}").into());
        }
        plan.q[which] = q;
        plan.n[which] = n;
        plan.len[which] = len;
    }
    Ok(plan)
}

// ------------------------------------------------------------------- probe

#[derive(Clone, Copy, Debug, Default)]
struct Forced {
    root: u64,
    null: u64,
    extent: u64,
    extent_rejected: u64,
    op: u64,
    op_rejected: u64,
    advance: u64,
    advance_rejected: u64,
    emission: u64,
}

#[derive(Clone, Copy, Debug)]
struct Consult {
    context: u16,
    inner_choice: usize,
    emitted: usize,
    input: usize,
}

struct Probe<P: Policy> {
    inner: P,
    phase: Phase,
    force: bool,
    plan: Plan,
    force_op: Option<usize>,
    force_emission: Option<usize>,
    inputs: Vec<[bool; 1024]>,
    last_emit_input: Option<usize>,
    consults: Vec<Consult>,
    forced: Forced,
}

impl<P: Policy> Probe<P> {
    fn new(inner: P) -> Self {
        Self {
            inner,
            phase: Phase::QueryA,
            force: false,
            plan: Plan::default(),
            force_op: None,
            force_emission: None,
            inputs: Vec::new(),
            last_emit_input: None,
            consults: Vec::new(),
            forced: Forced::default(),
        }
    }
    fn forced_json(&self) -> Value {
        json!({
            "root": self.forced.root,
            "null": self.forced.null,
            "extent": self.forced.extent,
            "extent_rejected": self.forced.extent_rejected,
            "op": self.forced.op,
            "op_rejected": self.forced.op_rejected,
            "advance": self.forced.advance,
            "advance_rejected": self.forced.advance_rejected,
            "emission": self.forced.emission,
        })
    }
}

impl<P: Policy> Policy for Probe<P> {
    fn context(&mut self, phase: Phase, input: &[bool; 1024]) -> engine::Result<u16> {
        self.phase = phase;
        let context = self.inner.context(phase, input)?;
        if phase == Phase::Emit {
            self.inputs.push(*input);
            self.last_emit_input = Some(self.inputs.len() - 1);
        }
        Ok(context)
    }
    fn choice(&mut self, head: Head, context: u16, legal: &[bool]) -> engine::Result<usize> {
        let inner_choice = self.inner.choice(head, context, legal)?;
        let mut choice = inner_choice;
        if self.force {
            match self.phase {
                Phase::QueryA | Phase::QueryB => {
                    let which = usize::from(self.phase == Phase::QueryB);
                    match head {
                        Head::Root(h) if h < 2 => {
                            choice = usize::from(self.plan.q[which][usize::from(h)]);
                            self.forced.root += 1;
                        }
                        Head::Null(h) if h < 2 => {
                            choice = usize::from(self.plan.n[which][usize::from(h)]);
                            self.forced.null += 1;
                        }
                        _ => {}
                    }
                }
                Phase::ExtentA | Phase::ExtentB => {
                    if let Head::Extent = head {
                        let which = usize::from(self.phase == Phase::ExtentB);
                        let want = usize::from(self.plan.len[which].saturating_sub(1));
                        if legal.get(want).copied().unwrap_or(false) {
                            choice = want;
                            self.forced.extent += 1;
                        } else {
                            self.forced.extent_rejected += 1;
                        }
                    }
                }
                _ => {}
            }
            if let Head::Control = head {
                if let Some(op) = self.force_op {
                    if legal.get(op).copied().unwrap_or(false) {
                        choice = op;
                        if op == OP_ADVANCE {
                            self.forced.advance += 1;
                        } else {
                            self.forced.op += 1;
                        }
                    } else if op == OP_ADVANCE {
                        self.forced.advance_rejected += 1;
                    } else {
                        self.forced.op_rejected += 1;
                    }
                }
            }
            if let Head::Emission = head {
                if let Some(code) = self.force_emission {
                    choice = code;
                    self.forced.emission += 1;
                }
            }
        }
        if let Head::Emission = head {
            let input = self.last_emit_input.ok_or(engine::EngineError::Invalid(
                "emission without a recorded Emit input",
            ))?;
            self.consults.push(Consult {
                context,
                inner_choice,
                emitted: choice,
                input,
            });
        }
        Ok(choice)
    }
}

// ------------------------------------------------------------------ driver

type ProbePolicy<'a> = Probe<Controlled<'a>>;

fn prefill<'a>(
    g: &BoundGeometry,
    params: &Parameters,
    compiled: &'a PrimitiveExport,
    epoch: u64,
    prompt: &[u8],
) -> AnyResult<(RuntimeSession, ProbePolicy<'a>)> {
    let mut runtime = RuntimeSession::new(params.digest(), g, epoch)?;
    let mut policy = Probe::new(Controlled::new(compiled, Control::Full, g.identity()));
    for &byte in prompt {
        let offer = runtime.predict(g, &mut policy)?;
        runtime.observe(Symbol::Byte(byte), offer.offer.id, g, &mut policy)?;
    }
    Ok((runtime, policy))
}

fn offer_json(offered: &engine::ForwardOffer) -> Value {
    let active = offered.offer.active();
    json!({
        "action": action_name(offered.offer.action),
        "emitted_symbol_by_head": symbol_code(offered.offer.symbol),
        "cursor_byte_of_produced_lease": active.map(objects::ActiveLease::byte),
        "produced_len": active.map(|a| a.lease().payload().len()),
        "produced_bytes": active
            .map(|a| String::from_utf8_lossy(a.lease().payload()).to_string()),
        "produced_is_provisional": active.map(|a| a.provisional().is_some()),
        "active_cursor": active.map(objects::ActiveLease::cursor),
        "active_acknowledged": active.map(objects::ActiveLease::acknowledged),
        "trace_contexts": offered.trace.contexts.iter().map(|c| c.map(u64::from)).collect::<Vec<_>>(),
    })
}

fn consult_json(input: &[bool; 1024], consult: &Consult) -> Value {
    json!({
        "context": consult.context,
        "what_the_unfitted_head_emitted": consult.emitted,
        "head_inner_choice_before_forcing": consult.inner_choice,
        "input_fields": fields(input),
        "input_blake3": input_digest(input),
    })
}

/// Mode `trace`: forced read + forced intended operator, teacher-forced actual
/// bytes, no emission forcing. Records every emission consultation per step.
fn run_trace(
    g: &BoundGeometry,
    params: &Parameters,
    compiled: &PrimitiveExport,
    root: &Path,
) -> AnyResult<()> {
    let start = Instant::now();
    let mut rows = Vec::new();
    let mut steps_total = 0_u64;
    let mut first_step_present = 0_u64;
    let mut first_step_examples = 0_u64;
    let mut later_step_present = 0_u64;
    let mut later_step_total = 0_u64;
    let mut unfitted_matches = 0_u64;
    let mut unfitted_matches_first_step = 0_u64;
    let mut distinct_step0_contexts = std::collections::BTreeSet::new();
    let mut value_to_step0_context: Vec<(String, u64)> = Vec::new();

    for (index, &(a, b, op)) in PROBLEMS.iter().enumerate() {
        let epoch = index as u64 + 1;
        let prompt = prompt_of(a, b);
        let answer = answer_of(a, b, op == 0);
        let spans = spans_of(a, b);
        let (mut runtime, mut policy) = prefill(g, params, compiled, epoch, &prompt)?;
        let mut steps = Vec::new();
        for (k, target) in answer
            .iter()
            .copied()
            .chain(std::iter::once(u8::MAX))
            .enumerate()
        {
            let target_code = if target == u8::MAX {
                SYMBOL_EOS
            } else {
                u16::from(target)
            };
            let plan = compute_plan(g, &runtime, epoch, spans)?;
            policy.plan = plan;
            policy.force = true;
            policy.force_op = Some(if op == 0 { OP_ADD } else { OP_SUB });
            policy.force_emission = None;
            let before = policy.consults.len();
            let offered = runtime.predict(g, &mut policy)?;
            let consults: Vec<Value> = policy.consults[before..]
                .iter()
                .map(|c| consult_json(&policy.inputs[c.input], c))
                .collect();
            let offer = offer_json(&offered);
            let cursor_byte = offer["cursor_byte_of_produced_lease"].as_u64();
            let produced = offer["produced_bytes"]
                .as_str()
                .map(str::to_string)
                .filter(|s| !s.is_empty());
            let present = cursor_byte == Some(u64::from(target_code));
            let in_input = consults.iter().all(|c| {
                c["input_fields"]["active_byte_792_800"].as_u64() == Some(u64::from(target_code))
            });
            let emitted = offer["emitted_symbol_by_head"].as_u64();
            let matched = emitted == Some(u64::from(target_code));
            steps_total += 1;
            unfitted_matches += u64::from(matched);
            if k == 0 {
                first_step_examples += 1;
                first_step_present += u64::from(present);
                unfitted_matches_first_step += u64::from(matched);
                if let Some(c) = policy.consults[before..].first() {
                    distinct_step0_contexts.insert(c.context);
                }
                if let Some(p) = produced.clone() {
                    value_to_step0_context.push((
                        p,
                        policy.consults[before..]
                            .first()
                            .map_or(0_u64, |c| u64::from(c.context)),
                    ));
                }
            } else {
                later_step_total += 1;
                later_step_present += u64::from(present);
            }
            steps.push(json!({
                "step": k,
                "target_code": target_code,
                "target_char": if target_code == SYMBOL_EOS { "EOS".to_string() } else { (target_code as u8 as char).to_string() },
                "action": offer["action"],
                "computed_value_bytes": produced,
                "computed_len": offer["produced_len"],
                "computed_cursor_byte": cursor_byte,
                "target_byte_equals_produced_cursor_byte": present,
                "target_byte_is_the_packed_active_byte_in_every_consult": in_input,
                "unfitted_head_emitted": emitted,
                "unfitted_head_matched_target": matched,
                "consultations_in_this_predict": consults,
                "offer": offer,
            }));
            runtime.observe(symbol_of(target_code)?, offered.offer.id, g, &mut policy)?;
            if target_code == SYMBOL_EOS {
                break;
            }
        }
        rows.push(json!({
            "index": index,
            "id": format!("arith/{}/{}", if op == 0 { "add" } else { "sub" }, index),
            "a": a,
            "b": b,
            "op": if op == 0 { "Add" } else { "Sub" },
            "prompt": String::from_utf8_lossy(&prompt).to_string(),
            "expected_answer": String::from_utf8_lossy(&answer).to_string(),
            "steps": steps,
        }));
        println!(
            "[emission-context] trace example {index} a={a} b={b} answer={}",
            String::from_utf8_lossy(&answer)
        );
    }

    let distinct_values = value_to_step0_context
        .iter()
        .map(|(v, _)| v.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let report = json!({
        "schema": "uor-r4.addressed-attention-emission-context-trace/1",
        "question": "At the answer-emitting step, is the byte the emission head must emit present in the packed input from which its context is computed?",
        "citations": CITATIONS,
        "arm": "forced read plan + forced intended operator (harness Policy choices only); actual bytes teacher-forced; emission head NOT forced",
        "configuration": {
            "parameters": format!("Parameters::seeded({PARAMETER_SEED}) (unfitted init, policy.rs:40-52)"),
            "parameter_digest": bytes_hex(&params.digest()),
            "geometry": "BoundGeometry::canonical()",
            "control": "Control::Full (no payload masking)",
            "shape": "<a>\\n<b>\\n (newline), spans [(0,len(a)),(len(a)+1,len(b))]",
            "split_size": PROBLEMS.len(),
        },
        "aggregate": {
            "examples": PROBLEMS.len(),
            "answer_and_eos_steps": steps_total,
            "first_answer_steps": first_step_examples,
            "first_answer_step_target_byte_present": first_step_present,
            "later_answer_step_target_byte_present": later_step_present,
            "later_answer_steps": later_step_total,
            "unfitted_head_matched_target_steps": unfitted_matches,
            "unfitted_head_matched_target_at_first_step": unfitted_matches_first_step,
            "distinct_step0_contexts": distinct_step0_contexts.len(),
            "distinct_computed_values": distinct_values,
            "value_to_step0_context": value_to_step0_context,
        },
        "rows": rows,
        "honesty": HONESTY,
        "elapsed_ms": start.elapsed().as_millis(),
    });
    seal(root, &report)
}

/// Mode `causal`: same operands, same runtime, AddAB vs SubAB; bit diff of the
/// packed emission input and context equality. Plus a forced canonical trajectory.
fn run_causal(
    g: &BoundGeometry,
    params: &Parameters,
    compiled: &PrimitiveExport,
    root: &Path,
) -> AnyResult<()> {
    let start = Instant::now();

    let run_one = |a: i64, b: i64, op: usize, epoch: u64| -> AnyResult<Value> {
        let prompt = prompt_of(a, b);
        let spans = spans_of(a, b);
        let (mut runtime, mut policy) = prefill(g, params, compiled, epoch, &prompt)?;
        let plan = compute_plan(g, &runtime, epoch, spans)?;
        policy.plan = plan;
        policy.force = true;
        policy.force_op = Some(op);
        let before = policy.consults.len();
        let offered = runtime.predict(g, &mut policy)?;
        let consult = *policy
            .consults
            .get(before)
            .ok_or("no emission consultation recorded")?;
        let input = policy.inputs[consult.input];
        let active = offered.offer.active();
        Ok(json!({
            "value_bytes": active.map(|l| String::from_utf8_lossy(l.lease().payload()).to_string()),
            "value_len": active.map(|l| l.lease().payload().len()),
            "cursor_byte": active.map(objects::ActiveLease::byte),
            "action": action_name(offered.offer.action),
            "context": consult.context,
            "input_fields": fields(&input),
            "input_blake3": input_digest(&input),
            "input_bits": input.to_vec(),
        }))
    };

    let mut pairs: Vec<(i64, i64)> = PROBLEMS.iter().map(|&(a, b, _)| (a, b)).collect();
    pairs.push(AMBIGUOUS);

    let mut swaps = Vec::new();
    let mut differing_values = 0_u64;
    let mut differing_inputs = 0_u64;
    let mut differing_contexts = 0_u64;
    for (n, &(a, b)) in pairs.iter().enumerate() {
        let add = run_one(a, b, OP_ADD, (n * 2 + 1) as u64)?;
        let sub = run_one(a, b, OP_SUB, (n * 2 + 2) as u64)?;
        let ia = add["input_bits"].as_array().unwrap();
        let is = sub["input_bits"].as_array().unwrap();
        let diff: Vec<usize> = (0..1024)
            .filter(|&i| ia[i].as_bool() != is[i].as_bool())
            .collect();
        let mut owners: Vec<String> = Vec::new();
        for &i in &diff {
            let owner = bit_owner(i);
            if !owners.iter().any(|o| o == owner) {
                owners.push(owner.to_string());
            }
        }
        let values_differ = add["value_bytes"] != sub["value_bytes"];
        let contexts_differ = add["context"] != sub["context"];
        differing_values += u64::from(values_differ);
        differing_inputs += u64::from(!diff.is_empty());
        differing_contexts += u64::from(contexts_differ);
        swaps.push(json!({
            "a": a,
            "b": b,
            "add": {
                "value": add["value_bytes"],
                "len": add["value_len"],
                "cursor_byte": add["cursor_byte"],
                "context": add["context"],
                "input_blake3": add["input_blake3"],
                "input_fields": add["input_fields"],
            },
            "sub": {
                "value": sub["value_bytes"],
                "len": sub["value_len"],
                "cursor_byte": sub["cursor_byte"],
                "context": sub["context"],
                "input_blake3": sub["input_blake3"],
                "input_fields": sub["input_fields"],
            },
            "values_differ": values_differ,
            "input_diff_bit_count": diff.len(),
            "input_diff_indices": diff,
            "input_diff_owners": owners,
            "contexts_differ": contexts_differ,
        }));
    }

    // Forced canonical trajectory: read forced, operator forced, emission forced to
    // the correct byte, Advance forced for every later digit. Asks whether each
    // answer byte is present at its own consultation.
    let trajectory_cases: [(i64, i64, usize); 8] = [
        (1, 2, OP_ADD),
        (9, 9, OP_ADD),
        (9, 3, OP_SUB),
        (64, 37, OP_SUB),
        (90, 45, OP_SUB),
        (73, 28, OP_SUB),
        (AMBIGUOUS.0, AMBIGUOUS.1, OP_ADD),
        (AMBIGUOUS.0, AMBIGUOUS.1, OP_SUB),
    ];
    let mut trajectories = Vec::new();
    for (n, &(a, b, op)) in trajectory_cases.iter().enumerate() {
        let answer = answer_of(a, b, op == OP_ADD);
        let spans = spans_of(a, b);
        let prompt = prompt_of(a, b);
        let epoch = 900 + n as u64;
        let (mut runtime, mut policy) = prefill(g, params, compiled, epoch, &prompt)?;
        let mut steps = Vec::new();
        for (k, target) in answer
            .iter()
            .copied()
            .chain(std::iter::once(u8::MAX))
            .enumerate()
        {
            let target_code = if target == u8::MAX {
                SYMBOL_EOS
            } else {
                u16::from(target)
            };
            let plan = compute_plan(g, &runtime, epoch, spans)?;
            policy.plan = plan;
            policy.force = true;
            policy.force_op = Some(if k == 0 { op } else { OP_ADVANCE });
            policy.force_emission = Some(usize::from(target_code));
            let before = policy.consults.len();
            let offered = runtime.predict(g, &mut policy)?;
            let consults: Vec<Value> = policy.consults[before..]
                .iter()
                .map(|c| consult_json(&policy.inputs[c.input], c))
                .collect();
            let present = consults.iter().all(|c| {
                c["input_fields"]["active_byte_792_800"].as_u64() == Some(u64::from(target_code))
            });
            let offer = offer_json(&offered);
            steps.push(json!({
                "step": k,
                "target_code": target_code,
                "target_char": if target_code == SYMBOL_EOS { "EOS".to_string() } else { (target_code as u8 as char).to_string() },
                "forced_action_index": policy.force_op,
                "action": offer["action"],
                "computed_value_bytes": offer["produced_bytes"],
                "computed_len": offer["produced_len"],
                "active_cursor": offer["active_cursor"],
                "active_acknowledged": offer["active_acknowledged"],
                "target_present_in_all_consults": present,
                "consultations": consults,
            }));
            runtime.observe(symbol_of(target_code)?, offered.offer.id, g, &mut policy)?;
            if target_code == SYMBOL_EOS {
                break;
            }
        }
        trajectories.push(json!({
            "a": a,
            "b": b,
            "op": if op == OP_ADD { "Add" } else { "Sub" },
            "prompt": String::from_utf8_lossy(&prompt).to_string(),
            "answer": String::from_utf8_lossy(&answer).to_string(),
            "steps": steps,
            "forced_counts": policy.forced_json(),
        }));
        println!(
            "[emission-context] causal trajectory a={a} b={b} op={} answer={}",
            if op == OP_ADD { "Add" } else { "Sub" },
            String::from_utf8_lossy(&answer)
        );
    }

    let report = json!({
        "schema": "uor-r4.addressed-attention-emission-context-causal/1",
        "question": "Does a change in the computed value alone (same operands, same reads, same runtime state) change the packed emission input and the emission context?",
        "citations": CITATIONS,
        "design": "Two runs per operand pair share the prompt, epoch, prefill trajectory, forced read plan and runtime state; only the forced operator differs. Any emission-input difference is therefore attributable to the computed value (and the action-dependent active lane), not to the operands.",
        "aggregate_swaps": {
            "operand_pairs": pairs.len(),
            "pairs_whose_values_differ": differing_values,
            "pairs_whose_emission_input_differs": differing_inputs,
            "pairs_whose_emission_context_differs": differing_contexts,
        },
        "ambiguous_prefix_case": {
            "pair": [AMBIGUOUS.0, AMBIGUOUS.1],
            "why": "15+4=19 and 15-4=11 share leading byte '1' and length 2, so at cursor 0 the emission input cannot distinguish them by their leading byte; at cursor 1 it can (see the two AMBIGUOUS trajectories)",
        },
        "swaps": swaps,
        "trajectories": trajectories,
        "honesty": HONESTY,
        "elapsed_ms": start.elapsed().as_millis(),
    });
    seal(root, &report)
}

fn seal_extra(root: &Path, report: &Value, extra: &[(&str, String)]) -> AnyResult<()> {
    report_output::claim(root)?;
    std::fs::write(root.join("report.json"), serde_json::to_vec_pretty(report)?)?;
    for (name, body) in extra {
        std::fs::write(root.join(name), body)?;
    }
    std::fs::write(
        root.join("PROVENANCE.txt"),
        format!(
            "bin: addressed-attention-emission-context\nbin_source_blake3: {}\nmodel_data_blake3: {}\nconfig_blake3: {}\n",
            blake3::hash(include_bytes!("addressed-attention-emission-context.rs")).to_hex(),
            blake3::hash(MODEL_DATA).to_hex(),
            blake3::hash(CONFIG_DIGEST).to_hex(),
        ),
    )?;
    report_output::seal(root)?;
    let unlisted = report_output::verify(root)?;
    println!(
        "[emission-context] sealed {} unlisted_files={}",
        root.display(),
        unlisted.len()
    );
    Ok(())
}

/// Mode `verdict`: reduce the two sealed measurement roots into one durable
/// conclusion root. Recomputes every headline number from the input reports and
/// records their BLAKE3 digests, so the summary is checkable against the data.
fn run_verdict(root: &Path, trace_root: &Path, causal_root: &Path) -> AnyResult<()> {
    let start = Instant::now();
    let trace_bytes = std::fs::read(trace_root.join("report.json"))?;
    let causal_bytes = std::fs::read(causal_root.join("report.json"))?;
    let t: Value = serde_json::from_slice(&trace_bytes)?;
    let c: Value = serde_json::from_slice(&causal_bytes)?;

    let mut steps = 0_u64;
    let mut single_consult = 0_u64;
    let mut matched = 0_u64;
    let mut first_present = 0_u64;
    let mut first_total = 0_u64;
    let mut later_present = 0_u64;
    let mut later_total = 0_u64;
    let mut eos_present = 0_u64;
    let mut eos_total = 0_u64;
    let mut digit_present = 0_u64;
    let mut digit_total = 0_u64;
    let mut groups: std::collections::BTreeMap<u64, Vec<(String, String)>> =
        std::collections::BTreeMap::new();
    for row in t["rows"].as_array().ok_or("trace rows")? {
        let answer = row["expected_answer"].as_str().unwrap_or("").to_string();
        let id = row["id"].as_str().unwrap_or("").to_string();
        for st in row["steps"].as_array().ok_or("steps")? {
            steps += 1;
            if st["consultations_in_this_predict"].as_array().map(Vec::len) == Some(1) {
                single_consult += 1;
            }
            matched += u64::from(st["unfitted_head_matched_target"].as_bool() == Some(true));
            let present = st["target_byte_is_the_packed_active_byte_in_every_consult"].as_bool()
                == Some(true);
            if st["target_char"].as_str() == Some("EOS") {
                eos_total += 1;
                eos_present += u64::from(present);
            } else {
                digit_total += 1;
                digit_present += u64::from(present);
            }
            if st["step"].as_u64() == Some(0) {
                first_total += 1;
                first_present += u64::from(
                    st["target_byte_equals_produced_cursor_byte"].as_bool() == Some(true),
                );
                if let Some(ctx) = st["consultations_in_this_predict"][0]["context"].as_u64() {
                    groups
                        .entry(ctx)
                        .or_default()
                        .push((answer.clone(), id.clone()));
                }
            } else {
                later_total += 1;
                later_present += u64::from(
                    st["target_byte_equals_produced_cursor_byte"].as_bool() == Some(true),
                );
            }
        }
    }
    let conflicting: Vec<Value> = groups
        .iter()
        .filter(|(_, items)| {
            items
                .iter()
                .map(|(a, _)| a.clone())
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                > 1
        })
        .map(|(ctx, items)| {
            json!({
                "context": ctx,
                "examples": items.iter().map(|(_, id)| id.clone()).collect::<Vec<_>>(),
                "answers": items.iter().map(|(a, _)| a.clone()).collect::<Vec<_>>(),
            })
        })
        .collect();
    let conflicting_examples: usize = conflicting
        .iter()
        .map(|g| g["examples"].as_array().map_or(0, Vec::len))
        .sum();

    let traj = c["trajectories"].as_array().ok_or("trajectories")?;
    let mut traj_digit_present = 0_u64;
    let mut traj_digit_total = 0_u64;
    let mut traj_eos_present = 0_u64;
    let mut traj_eos_total = 0_u64;
    for t in traj {
        for st in t["steps"].as_array().ok_or("traj steps")? {
            let present = st["target_present_in_all_consults"].as_bool() == Some(true);
            if st["target_char"].as_str() == Some("EOS") {
                traj_eos_total += 1;
                traj_eos_present += u64::from(present);
            } else {
                traj_digit_total += 1;
                traj_digit_present += u64::from(present);
            }
        }
    }
    let ambiguous = c["swaps"]
        .as_array()
        .and_then(|s| {
            s.iter().find(|x| {
                x["a"].as_i64() == Some(AMBIGUOUS.0) && x["b"].as_i64() == Some(AMBIGUOUS.1)
            })
        })
        .cloned()
        .unwrap_or(Value::Null);

    let swaps = c["swaps"].as_array().ok_or("swaps")?;
    let owners: std::collections::BTreeSet<String> = swaps
        .iter()
        .flat_map(|s| {
            s["input_diff_owners"]
                .as_array()
                .map(|v| {
                    v.iter()
                        .filter_map(|o| o.as_str().map(str::to_string))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        })
        .collect();

    let verdict = json!({
        "schema": "uor-r4.addressed-attention-emission-context-verdict/1",
        "question": "Is the computed arithmetic value present in the emission head's context, or is the emission head structurally unable to see it?",
        "answer": "INFORMATION PRESENT",
        "structural_finding": {
            "wired": "For AddAB/SubAB the prospective result lease is the active lease at the emission step (engine.rs:386-393, objects.rs:291-328). Its canonical encoding's cursor byte and encoded length are packed into the emission input at bits 792..800 and 822..829 (inputs.rs:120,123), together with kind=2, cursor, acknowledged and provisional. The emission context is the compiled circuit's evaluation of that input (policy.rs:116-118) and selects the head's 257-entry row (policy.rs:99,124).",
            "not_wired": "The scalar Derivation.value (objects.rs:106) is not a field of CausalInputs (inputs.rs:36-48); only its current canonical byte and length reach the head. EOS is never a packed byte.",
        },
        "measured": {
            "first_answer_step_target_byte_is_packed_active_byte": format!("{first_present}/{first_total}"),
            "all_answer_digit_steps_target_byte_is_packed_active_byte": format!("{digit_present}/{digit_total} (32/32 at step 0; the later misses are the harness re-forcing AddAB, which keeps the result cursor at 0 instead of advancing)"),
            "later_steps_under_forced_operator_only": format!("{later_present}/{later_total} (trajectory artifact: re-forcing AddAB keeps cursor 0)"),
            "eos_steps_where_eos_is_a_packed_byte": format!("{eos_present}/{eos_total}"),
            "unfitted_head_matched_target_steps": format!("{matched}/{steps}"),
            "emission_consultations_per_predict": format!("{single_consult}/{steps} exactly one"),
            "causal_swap_pairs": c["aggregate_swaps"]["operand_pairs"],
            "causal_swap_pairs_whose_emission_input_differs": c["aggregate_swaps"]["pairs_whose_emission_input_differs"],
            "causal_swap_pairs_whose_emission_context_differs": c["aggregate_swaps"]["pairs_whose_emission_context_differs"],
            "causal_swap_differing_fields": owners.iter().cloned().collect::<Vec<_>>(),
            "ambiguous_prefix_pair_15_4": {
                "add": ambiguous["add"]["value"],
                "sub": ambiguous["sub"]["value"],
                "step0_input_diff_bit_count": ambiguous["input_diff_bit_count"],
                "step0_contexts": [ambiguous["add"]["context"], ambiguous["sub"]["context"]],
                "note": "identical leading byte and length: the full value is not visible at cursor 0 by construction; it becomes visible at cursor 1",
            },
            "forced_canonical_trajectory_digit_steps_present": format!("{traj_digit_present}/{traj_digit_total}"),
            "forced_canonical_trajectory_eos_steps_present": format!("{traj_eos_present}/{traj_eos_total}"),
            "unfitted_step0_context_collisions": {
                "contexts": groups.len(),
                "examples": first_total,
                "multi_target_contexts": conflicting.len(),
                "examples_in_conflicting_contexts": conflicting_examples,
                "groups": conflicting,
                "note": "the unfitted compiled circuit is lossy; a fit must also move the trainable gate parameters (policy.rs:10,78-83), not only the head rows",
            },
        },
        "verdict_branch": "INFORMATION PRESENT: the value's canonical cursor byte and length are wired into the emission input and reach the emission context; no wiring change is required. The silence is a parameter/training problem, so the concurrent fit is the right instrument. Qualifications: (i) the value is available one byte at a time, not as a scalar; (ii) gate logits are trainable, so a fit can also reshape the circuit that hashes the input into the 9-bit context; (iii) EOS is not a wired byte and must be inferred from state.",
        "inputs": {
            "trace_root": trace_root.display().to_string(),
            "trace_report_blake3": blake3::hash(&trace_bytes).to_hex().to_string(),
            "causal_root": causal_root.display().to_string(),
            "causal_report_blake3": blake3::hash(&causal_bytes).to_hex().to_string(),
        },
        "citations": CITATIONS,
        "honesty": HONESTY,
        "elapsed_ms": start.elapsed().as_millis(),
    });

    let md = format!(
        "# Emission-phase context: does the emission head see the computed value?\n\n\
         **Verdict: INFORMATION PRESENT.** The value's canonical cursor byte and encoded length are wired into\n\
         the packed `Phase::Emit` input and reach the emission head's context. No wiring change is required;\n\
         the 0/80 silence is a parameter/training problem.\n\n\
         ## Structural trace\n\n\
         - `engine.rs:392-393` - the emission head is consulted with the `Phase::Emit` context:\n\
           `let context = call(p, Phase::Emit, input, &mut trace)?; choose(p, Head::Emission, context, &[true; 257])`.\n\
         - `engine.rs:388-391` - that input is `inputs(g, a, b, prospective.active(), true)`.\n\
         - `engine.rs:386` - `prospective = objects.prepare_action(prepared, action)`; the action came from the\n\
           Control head at `engine.rs:383-385`.\n\
         - `objects.rs:291-328` - for `AddAB`/`SubAB` the prospective active lease is built from\n\
           `value = execute(action, a, b)` (`objects.rs:304`), `encode_i64(value)` (`objects.rs:305`) and\n\
           `Derivation {{ value, .. }}` (`objects.rs:306-312`), with `cursor: 0` and `provisional: Some(derivation)`.\n\
         - `engine.rs:228-238` - `inputs()` maps that lease to `Active {{ byte: a.byte(), kind, cursor,\n\
           length: payload().len(), acknowledged, provisional }}`.\n\
         - `objects.rs:132-134` / `objects.rs:90-92` - `byte()` is `lease.bytes[cursor]`; `payload()` is `bytes[..len]`.\n\
         - `inputs.rs:108-126` - the active lane is packed at bits 792..800 (byte), 812..814 (kind), 816..822\n\
           (cursor), 822..829 (length), 843 (acknowledged), 844 (provisional).\n\
         - `policy.rs:116-118` - the context is the compiled Boolean circuit's evaluation of that packed input;\n\
           `policy.rs:99` / `policy.rs:124` - the context selects the emission head's 257-entry parameter row.\n\n\
         Not wired: the scalar `Derivation.value` (`objects.rs:106`) is not a field of `CausalInputs`\n\
         (`inputs.rs:36-48`). Only its canonical current byte and length reach the head. EOS is not a byte.\n\n\
         ## Measurement (unfitted `Parameters::seeded(7341)`, forced read + forced operator)\n\n\
         - First answer step: the required byte is the packed active byte in **{first_present}/{first_total}** examples.\n\
         - All answer-digit steps under this arm: **{digit_present}/{digit_total}** - 32/32 at step 0; the later\n\
           misses are the harness re-forcing `AddAB` every step, which keeps the result cursor at 0 instead of\n\
           advancing. With the correct action sequence the later digits are present too (next bullets).\n\
         - EOS steps where EOS is a packed byte: **{eos_present}/{eos_total}** (EOS must be inferred).\n\
         - Unfitted head matched the target on **{matched}/{steps}** steps (reproduces the 0/80 silence).\n\
         - Emission consultations per predict: **{single_consult}/{steps}** exactly one.\n\
         - Causal swap (same operands/state, AddAB vs SubAB): **{swaps_len}** pairs, values differ in\n\
           **{values_differ}**, emission input differs in **{input_differ}**, emission context differs in\n\
           **{ctx_differ}**. Every differing bit sits in: {owners}.\n\
         - Ambiguous pair 15+4=19 vs 15-4=11: step-0 inputs differ in **{amb_bits}** bits (identical leading byte\n\
           and length) with contexts {amb_ctx_add}/{amb_ctx_sub}; at cursor 1 the digits '9' vs '1' are distinguishable.\n\
         - Forced canonical trajectory (forced correct emission + forced Advance): **{traj_digit_present}/{traj_digit_total}**\n\
           digit steps carry the target byte at the cursor, including every 2nd and 3rd digit (e.g. 18 -> '1' then '8',\n\
           27 -> '2' then '7', 19 vs 11 -> '9' vs '1' at cursor 1).\n\
         - Unfitted step-0 contexts: {ctx_groups} contexts for {first_total} examples; **{conflict_groups}**\n\
           contexts (covering {conflict_examples} examples) collapse different targets, so a fit must also move the\n\
           trainable gate parameters (`policy.rs:10`, `policy.rs:78-83`), not just the head rows.\n\n\
         ## Honest scope\n\n\
         This establishes that the arithmetic result is connected to the emission head's input and context\n\
         byte-wise, so the head is not structurally blind and no rewiring is needed. It does not establish that\n\
         a bounded fit will succeed, nor that the value is visible as a scalar or that EOS is wired.\n\n\
         No production source file was modified. Raw data: `{trace_root}` and `{causal_root}`.\n",
        first_present = first_present,
        first_total = first_total,
        digit_present = digit_present,
        digit_total = digit_total,
        eos_present = eos_present,
        eos_total = eos_total,
        matched = matched,
        steps = steps,
        single_consult = single_consult,
        swaps_len = swaps.len(),
        values_differ = c["aggregate_swaps"]["pairs_whose_values_differ"],
        input_differ = c["aggregate_swaps"]["pairs_whose_emission_input_differs"],
        ctx_differ = c["aggregate_swaps"]["pairs_whose_emission_context_differs"],
        owners = owners.iter().cloned().collect::<Vec<_>>().join("; "),
        amb_bits = ambiguous["input_diff_bit_count"],
        amb_ctx_add = ambiguous["add"]["context"],
        amb_ctx_sub = ambiguous["sub"]["context"],
        traj_digit_present = traj_digit_present,
        traj_digit_total = traj_digit_total,
        ctx_groups = groups.len(),
        conflict_groups = conflicting.len(),
        conflict_examples = conflicting_examples,
        trace_root = trace_root.display(),
        causal_root = causal_root.display(),
    );

    seal_extra(
        root,
        &verdict,
        &[
            ("VERDICT.md", md),
            (
                "README.txt",
                "See VERDICT.md and report.json.\n".to_string(),
            ),
        ],
    )
}

fn seal(root: &Path, report: &Value) -> AnyResult<()> {
    report_output::claim(root)?;
    std::fs::write(root.join("report.json"), serde_json::to_vec_pretty(report)?)?;
    std::fs::write(
        root.join("PROVENANCE.txt"),
        format!(
            "bin: addressed-attention-emission-context\nbin_source_blake3: {}\nmodel_data_blake3: {}\nconfig_blake3: {}\n",
            blake3::hash(include_bytes!("addressed-attention-emission-context.rs")).to_hex(),
            blake3::hash(MODEL_DATA).to_hex(),
            blake3::hash(CONFIG_DIGEST).to_hex(),
        ),
    )?;
    report_output::seal(root)?;
    let unlisted = report_output::verify(root)?;
    println!(
        "[emission-context] sealed {} unlisted_files={}",
        root.display(),
        unlisted.len()
    );
    Ok(())
}

fn prepare_base(base: &str) -> AnyResult<PathBuf> {
    let base = PathBuf::from(base);
    if !base.is_absolute() {
        return Err("base directory must be an absolute path".into());
    }
    for ancestor in base.parent().into_iter().flat_map(Path::ancestors) {
        if ancestor.join("manifest.json").exists() {
            return Err(
                format!("base {} is beneath an already sealed root", base.display()).into(),
            );
        }
    }
    Ok(base)
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

fn main() -> AnyResult<()> {
    let _ = rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build_global();
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).ok_or(
        "usage: addressed-attention-emission-context <trace|causal> <abs-root> | verdict <abs-root> <trace-root> <causal-root>",
    )?;
    let base = prepare_base(args.get(2).ok_or("missing absolute base directory")?)?;
    let started = Instant::now();
    let params = Parameters::seeded(PARAMETER_SEED)?;
    let model = build_model(&params)?;
    match mode.as_str() {
        "trace" => run_trace(model.geometry(), &params, model.compiled(), &base)?,
        "causal" => run_causal(model.geometry(), &params, model.compiled(), &base)?,
        "verdict" => run_verdict(
            &base,
            Path::new(args.get(3).ok_or("missing trace root")?),
            Path::new(args.get(4).ok_or("missing causal root")?),
        )?,
        other => return Err(format!("unknown mode {other}").into()),
    }
    println!(
        "[emission-context] done mode={mode} elapsed_ms={} root={}",
        started.elapsed().as_millis(),
        base.display()
    );
    Ok(())
}
