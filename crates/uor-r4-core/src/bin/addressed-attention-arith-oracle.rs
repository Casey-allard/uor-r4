//! Measurement-only probe: when the addressed-attention read selects canonical
//! decimal operand spans, does the engine produce the correct answer?
//!
//! The frozen 8-example development pilot split is prose. It contains no bare
//! canonical decimal operand, so its own scored surface never offers `AddAB` or
//! `SubAB` (`objects.rs:233-253`, `objects.rs:761-790`) and cannot ask this
//! question at all. This binary supplies a new authored split of short
//! canonical-decimal prompts and drives the *unmodified* engine on it.
//!
//! It adds no production code path, changes no default behaviour, and never
//! mutates frozen parameters, geometry, pilot data or any committed artifact.
//! Every root it writes is exclusively claimed and sealed. Subcommands:
//!
//!   criteria <abs-base>   frozen acceptance criteria; written before any run
//!   diagnose <abs-base>   offers-only structural diagnostic over prompt shapes
//!   run      <abs-base>   the pre-registered scored run and mechanism arms
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde_json::{json, Value};
use uor_r4_core::native_geometric::addressed_attention::artifact::{
    BoundGeometry, Model, Provenance,
};
use uor_r4_core::native_geometric::addressed_attention::circuit::PrimitiveExport;
use uor_r4_core::native_geometric::addressed_attention::engine::{
    self, Head, Policy, RuntimeSession, Work,
};
use uor_r4_core::native_geometric::addressed_attention::inputs::Phase;
use uor_r4_core::native_geometric::addressed_attention::objects::{self, Action, Symbol};
use uor_r4_core::native_geometric::addressed_attention::pilot::{self, Control, Controlled};
use uor_r4_core::native_geometric::addressed_attention::pilot_data::{self, Example};
use uor_r4_core::native_geometric::addressed_attention::policy::{
    implementation_digest, InterpretedPolicy, Parameters,
};
use uor_r4_core::report_output;

type AnyResult<T> = Result<T, Box<dyn std::error::Error>>;

const PARAMETER_SEED: u64 = 7341;
const MODEL_DATA: &[u8] = b"addressed-attention-arith-oracle/1";
const CONFIG_DIGEST: &[u8] = b"addressed-attention-arith-oracle-config/1";
/// Offer rate required by criterion A1.
const A1_MIN_OFFER_FRACTION: f64 = 0.05;

/// Frozen before any engine call in this session. `criteria` mode writes it.
const CRITERIA: &str = r#"{
  "schema": "uor-r4.addressed-attention-arith-oracle-criteria/1",
  "frozen": "written before any engine run in this session; no threshold below was adjusted after seeing a result",
  "question": "When the addressed-attention read selects the right operand spans, does the engine produce the correct answer? The frozen 8-example development split is prose and contains no bare canonical decimal, so its scored surface offers AddAB/SubAB zero times and cannot ask this question.",
  "instrument": {
    "bin": "addressed-attention-arith-oracle (new, measurement-only; no production change)",
    "policy": "deterministic CompiledPolicy over Parameters::seeded(7341); BoundGeometry::canonical()",
    "split": "arith-canonical-decimal/1, authored in this bin, 32 short prompts whose window contains bare canonical decimal operands; expected answers constructed by exact i64 arithmetic in this instrument (NOT a frozen external fixture)",
    "prompt_shape": "chosen by the `diagnose` subcommand, which measures only whether arithmetic is OFFERED and whether the intended operand pair is selected. No expected answer, emitted symbol or correctness signal selects the shape."
  },
  "primary_criteria": {
    "A1_reachability": "under Control::Full, AddAB or SubAB is OFFERED (legal mask index 3 or 4 true) on >= 5% of control-head calls",
    "A2_selection": "summed RuntimeSession::work().selected_operations over the Full arm > 0",
    "A3_reporting": "correct_symbols/positions, exact_responses, generated_eos and mean_response_ce reported for Full and for all three destructive controls (ReadDisabled, ExactPayloadMasked, StateTransportDisabled)",
    "A4_decision_rule": "POSITIVE iff Full has non-zero correct_symbols AND for every one of the three controls Full mean_response_ce is strictly lower AND at least one example is exactly right (answer+EOS) under Full and not under that control. Otherwise the run is recorded as a NULL with its numbers."
  },
  "secondary_measurements": {
    "S1_computed_value": "for every step whose control head chose AddAB/SubAB, compare the produced active-lease bytes with the exact canonical encoding of the chosen operation applied to the two payloads the read actually selected; count exact matches",
    "S2_conditional_emission": "for those same steps, count how often the emitted symbol equals the byte at the produced lease cursor",
    "S3_forced_read": "harness-only instrument: query roots chosen by exhaustive search over the 120x120 root pairs against the intended span keys, null roots chosen to minimise the null score, extent forced to the intended length. Arm C1 leaves the control head to CompiledPolicy; arm C2 additionally forces the intended operator when legal. Reported: forcing success rate, offers, selected_operations, exact answers.",
    "S4_interpreted": "the same split driven with InterpretedPolicy (uncompiled argmax over the same parameters) for comparison"
  },
  "branch_rule": {
    "B1": "Full beats all three controls with non-zero correctness -> POSITIVE: the mechanism computes when given the right data; the frozen split was the blocker",
    "B2": "Full does not beat all three controls -> NULL with numbers: the deficit is in the mechanism, not only in the data",
    "B3": "arithmetic is never even offered -> the split failed to place canonical decimals where the read can select them; report what was observed instead of forcing a conclusion"
  },
  "honesty": [
    "The new split's expected answers are CONSTRUCTED here by i64 arithmetic, not taken from a frozen external fixture; that is a weaker instrument than a frozen corpus.",
    "Control calls include prefill positions, exactly as pilot::evaluate counts them.",
    "No production source file is modified by this probe."
  ]
}"#;

// ------------------------------------------------------------------- split

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Shape {
    /// "8 3\n"
    Space,
    /// "83\n"
    Concat,
    /// "8\n3\n"
    Newline,
    /// "8+3\n"
    Plus,
}

impl Shape {
    const ALL: [Shape; 4] = [Shape::Space, Shape::Concat, Shape::Newline, Shape::Plus];
    fn name(self) -> &'static str {
        match self {
            Shape::Space => "space",
            Shape::Concat => "concat",
            Shape::Newline => "newline",
            Shape::Plus => "plus",
        }
    }
    fn render(self, a: &str, b: &str) -> String {
        match self {
            Shape::Space => format!("{a} {b}\n"),
            Shape::Concat => format!("{a}{b}\n"),
            Shape::Newline => format!("{a}\n{b}\n"),
            Shape::Plus => format!("{a}+{b}\n"),
        }
    }
    /// Intended operand spans `(start, len)` inside the rendered prompt.
    fn spans(self, a: &str, b: &str) -> [(u64, u8); 2] {
        let la = a.len() as u64;
        let lb = b.len() as u8;
        match self {
            Shape::Concat => [(0, la as u8), (la, lb)],
            Shape::Space | Shape::Newline | Shape::Plus => [(0, la as u8), (la + 1, lb)],
        }
    }
}

/// (a, b, op) with op 0 = Add, 1 = Sub. Answers are computed exactly below.
const PROBLEMS: [(i64, i64, u8); 32] = [
    // addition, one-digit operands, one-digit result
    (1, 2, 0),
    (2, 3, 0),
    (3, 4, 0),
    (4, 5, 0),
    (1, 7, 0),
    (2, 6, 0),
    (3, 5, 0),
    (4, 4, 0),
    // addition, one-digit operands, two-digit result
    (5, 7, 0),
    (6, 8, 0),
    (8, 9, 0),
    (9, 9, 0),
    // subtraction, one-digit operands
    (9, 3, 1),
    (8, 2, 1),
    (7, 5, 1),
    (6, 1, 1),
    (5, 4, 1),
    (9, 9, 1),
    (8, 6, 1),
    (7, 2, 1),
    // addition, two-digit operands
    (12, 34, 0),
    (21, 45, 0),
    (33, 26, 0),
    (40, 19, 0),
    (11, 88, 0),
    (55, 27, 0),
    // subtraction, two-digit operands
    (56, 23, 1),
    (48, 19, 1),
    (90, 45, 1),
    (73, 28, 1),
    (64, 37, 1),
    (81, 16, 1),
];

const OP_ADD: usize = 3;
const OP_SUB: usize = 4;

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
fn payload_string(lease: &objects::Lease) -> String {
    String::from_utf8_lossy(lease.payload()).to_string()
}
fn span_string(prompt: &[u8], span: (u64, u8)) -> String {
    let start = span.0 as usize;
    let end = start + usize::from(span.1);
    prompt
        .get(start..end)
        .map(|b| String::from_utf8_lossy(b).to_string())
        .unwrap_or_default()
}
fn exact_value(action: Action, a: i64, b: i64) -> Option<i64> {
    match action {
        Action::AddAB => a.checked_add(b),
        Action::SubAB => a.checked_sub(b),
        _ => None,
    }
}
fn canonical(value: i64) -> Vec<u8> {
    objects::encode_i64(value).0[..usize::from(objects::encode_i64(value).1)].to_vec()
}

fn split(shape: Shape) -> Vec<Example> {
    PROBLEMS
        .iter()
        .enumerate()
        .map(|(index, &(a, b, op))| {
            let (as_, bs) = (a.to_string(), b.to_string());
            let scalar = if op == 0 { a + b } else { a - b };
            Example {
                id: format!("arith/{}/{index}", if op == 0 { "add" } else { "sub" }),
                family: if op == 0 {
                    "arith_canonical_decimal_add".to_string()
                } else {
                    "arith_canonical_decimal_sub".to_string()
                },
                prompt: shape.render(&as_, &bs).into_bytes(),
                answer: scalar.to_string().into_bytes(),
                expected_scalar: Some(scalar),
            }
        })
        .collect()
}

fn op_of(index: usize) -> usize {
    if PROBLEMS[index].2 == 0 {
        OP_ADD
    } else {
        OP_SUB
    }
}
fn intended(shape: Shape, index: usize) -> [(u64, u8); 2] {
    let (a, b, _) = PROBLEMS[index];
    shape.spans(&a.to_string(), &b.to_string())
}

// -------------------------------------------------------------- statistics

#[derive(Clone, Copy, Debug, Default)]
struct Stats {
    control_calls: u64,
    add_offered: u64,
    sub_offered: u64,
    arith_offered_steps: u64,
    chosen_add: u64,
    chosen_sub: u64,
    legal_sizes: [u64; 8],
    forced_root: u64,
    forced_null: u64,
    forced_extent: u64,
    forced_extent_rejected: u64,
    forced_op: u64,
    forced_op_rejected: u64,
}
impl Stats {
    fn record(&mut self, legal: &[bool], choice: usize) {
        self.control_calls += 1;
        let add = legal.get(OP_ADD).copied().unwrap_or(false);
        let sub = legal.get(OP_SUB).copied().unwrap_or(false);
        self.add_offered += u64::from(add);
        self.sub_offered += u64::from(sub);
        self.arith_offered_steps += u64::from(add || sub);
        self.chosen_add += u64::from(choice == OP_ADD);
        self.chosen_sub += u64::from(choice == OP_SUB);
        let k = legal.iter().filter(|&&b| b).count();
        if let Some(slot) = self.legal_sizes.get_mut(k) {
            *slot += 1;
        }
    }
    fn merge(&mut self, other: &Stats) {
        self.control_calls += other.control_calls;
        self.add_offered += other.add_offered;
        self.sub_offered += other.sub_offered;
        self.arith_offered_steps += other.arith_offered_steps;
        self.chosen_add += other.chosen_add;
        self.chosen_sub += other.chosen_sub;
        for (dst, src) in self.legal_sizes.iter_mut().zip(other.legal_sizes) {
            *dst += src;
        }
        self.forced_root += other.forced_root;
        self.forced_null += other.forced_null;
        self.forced_extent += other.forced_extent;
        self.forced_extent_rejected += other.forced_extent_rejected;
        self.forced_op += other.forced_op;
        self.forced_op_rejected += other.forced_op_rejected;
    }
    fn json(&self) -> Value {
        json!({
            "control_calls": self.control_calls,
            "add_offered": self.add_offered,
            "sub_offered": self.sub_offered,
            "arithmetic_offered_steps": self.arith_offered_steps,
            "chosen_add": self.chosen_add,
            "chosen_sub": self.chosen_sub,
            "chosen_arithmetic": self.chosen_add + self.chosen_sub,
            "legal_size_histogram": self.legal_sizes,
            "forced_root": self.forced_root,
            "forced_null": self.forced_null,
            "forced_extent": self.forced_extent,
            "forced_extent_rejected": self.forced_extent_rejected,
            "forced_op": self.forced_op,
            "forced_op_rejected": self.forced_op_rejected,
        })
    }
}

// ------------------------------------------------------------ oracle policy

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    Production,
    ForcedRead,
    ForcedReadOp,
}

#[derive(Clone, Copy, Debug, Default)]
struct Plan {
    q: [[u16; 2]; 2],
    n: [[u16; 2]; 2],
    len: [u8; 2],
}

#[derive(Clone, Copy, Debug)]
enum Winner {
    Occurrence(u64),
    Result(u64),
    None,
}

struct Oracle<P: Policy> {
    inner: P,
    arm: Arm,
    phase: Phase,
    force: bool,
    plan: Plan,
    intended_op: usize,
    stats: Stats,
}

impl<P: Policy> Oracle<P> {
    fn new(inner: P, arm: Arm, intended_op: usize) -> Self {
        Self {
            inner,
            arm,
            phase: Phase::QueryA,
            force: false,
            plan: Plan::default(),
            intended_op,
            stats: Stats::default(),
        }
    }
}

impl<P: Policy> Policy for Oracle<P> {
    fn context(&mut self, phase: Phase, input: &[bool; 1024]) -> engine::Result<u16> {
        self.phase = phase;
        self.inner.context(phase, input)
    }
    fn choice(&mut self, head: Head, context: u16, legal: &[bool]) -> engine::Result<usize> {
        let mut choice = self.inner.choice(head, context, legal)?;
        if self.arm != Arm::Production && self.force {
            let read = match self.phase {
                Phase::QueryA => Some(0usize),
                Phase::QueryB => Some(1usize),
                _ => None,
            };
            if let (Some(which), Head::Root(h)) = (read, head) {
                if h < 2 {
                    choice = usize::from(self.plan.q[which][usize::from(h)]);
                    self.stats.forced_root += 1;
                }
            }
            if let (Some(which), Head::Null(h)) = (read, head) {
                if h < 2 {
                    choice = usize::from(self.plan.n[which][usize::from(h)]);
                    self.stats.forced_null += 1;
                }
            }
            if let Head::Extent = head {
                let which = match self.phase {
                    Phase::ExtentA => Some(0usize),
                    Phase::ExtentB => Some(1usize),
                    _ => None,
                };
                if let Some(which) = which {
                    let want = usize::from(self.plan.len[which].saturating_sub(1));
                    if legal.get(want).copied().unwrap_or(false) {
                        choice = want;
                        self.stats.forced_extent += 1;
                    } else {
                        self.stats.forced_extent_rejected += 1;
                    }
                }
            }
            if self.arm == Arm::ForcedReadOp {
                if let Head::Control = head {
                    if legal.get(self.intended_op).copied().unwrap_or(false) {
                        choice = self.intended_op;
                        self.stats.forced_op += 1;
                    } else {
                        self.stats.forced_op_rejected += 1;
                    }
                }
            }
        }
        if matches!(head, Head::Control) {
            self.stats.record(legal, choice);
        }
        Ok(choice)
    }
}

// -------------------------------------------------------------- read plan

/// Best harness-side query/null/extent plan for the intended operand spans, plus
/// a faithful replay of the engine's own selection loop to say whether the
/// intended occurrence would actually win.
fn compute_plan(
    g: &BoundGeometry,
    runtime: &RuntimeSession,
    epoch: u64,
    spans: [(u64, u8); 2],
) -> AnyResult<(Plan, Value)> {
    let frontier = runtime.objects().frontier();
    let mut plan = Plan::default();
    let mut detail = Vec::new();
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
        let (q_score, q) = best.ok_or("no query pair")?;
        let mut worst: Option<(i16, [u16; 2])> = None;
        for n0 in 0..120u16 {
            for n1 in 0..120u16 {
                let s = g.score(q, [n0, n1])?;
                if worst.is_none_or(|(b, _)| s < b) {
                    worst = Some((s, [n0, n1]));
                }
            }
        }
        let (null_score, n) = worst.ok_or("no null pair")?;
        let mut score = null_score;
        let mut winner = Winner::None;
        for s in frontier.saturating_sub(256)..frontier {
            let rec = runtime.objects().occurrence(epoch, s)?;
            let c = g.score(q, rec.keys)?;
            if c > score {
                score = c;
                winner = Winner::Occurrence(s);
            }
        }
        for id in runtime
            .objects()
            .result_frontier()
            .saturating_sub(8)
            .max(1)..runtime.objects().result_frontier()
        {
            let rec = runtime.objects().result(epoch, id)?;
            let c = g.score(q, rec.lease.keys())?;
            if c > score {
                score = c;
                winner = Winner::Result(id);
            }
        }
        let maximum = runtime.objects().maximum_extent(epoch, start)?;
        plan.q[which] = q;
        plan.n[which] = n;
        plan.len[which] = len;
        detail.push(json!({
            "read": which,
            "intended_start": start,
            "intended_len": len,
            "query": q,
            "query_score": q_score,
            "null": n,
            "null_score": null_score,
            "winning_score": score,
            "winner": match winner {
                Winner::Occurrence(s) => json!({"kind": "occurrence", "start": s}),
                Winner::Result(id) => json!({"kind": "result", "id": id}),
                Winner::None => json!({"kind": "none"}),
            },
            "intended_wins": matches!(winner, Winner::Occurrence(s) if s == start),
            "maximum_extent": maximum,
            "extent_forcible": maximum >= len,
        }));
    }
    Ok((plan, Value::Array(detail)))
}

// ------------------------------------------------------------------ driver

#[allow(clippy::too_many_arguments)]
fn drive(
    g: &BoundGeometry,
    params: &Parameters,
    compiled: &PrimitiveExport,
    example: &Example,
    index: usize,
    control: Control,
    arm: Arm,
    shape: Shape,
    record: bool,
) -> AnyResult<Value> {
    let epoch = index as u64 + 1;
    let intended_op = op_of(index);
    let mut runtime = RuntimeSession::new(params.digest(), g, epoch)?;
    let mut policy = Oracle::new(
        Controlled::new(compiled, control, g.identity()),
        arm,
        intended_op,
    );
    for &byte in &example.prompt {
        let offer = runtime.predict(g, &mut policy)?;
        runtime.observe(Symbol::Byte(byte), offer.offer.id, g, &mut policy)?;
    }
    let spans = intended(shape, index);
    let mut steps: Vec<Value> = Vec::new();
    let mut correct_symbols = 0usize;
    let mut positions = 0usize;
    let mut generated: Vec<u16> = Vec::new();
    let mut hash = blake3::Hasher::new();
    let mut correct_value_steps = 0u64;
    let mut wrong_value_steps = 0u64;
    let mut copy_steps = 0u64;
    let mut noncopy_steps = 0u64;
    let mut intended_pair_steps = 0u64;
    let mut arithmetic_steps = 0u64;
    let mut read_plan = Value::Null;
    let mut first_generated_state_digest = String::new();
    let mut state_digest_taken = false;

    for target in example
        .answer
        .iter()
        .map(|&b| u16::from(b))
        .chain([256])
    {
        if arm != Arm::Production {
            let (plan, detail) =
                compute_plan(g, &runtime, epoch, spans).map_err(|e| format!("plan: {e}"))?;
            policy.plan = plan;
            policy.force = true;
            read_plan = detail;
        }
        if !state_digest_taken {
            first_generated_state_digest = blake3::hash(&runtime.snapshot()?).to_hex().to_string();
            state_digest_taken = true;
        }
        let offered = runtime.predict(g, &mut policy)?;
        let emitted = symbol_code(offered.offer.symbol);
        positions += 1;
        if emitted == target {
            correct_symbols += 1;
        }
        let operands = offered.offer.operands();
        let payloads: Vec<Option<String>> =
            operands.iter().map(|o| o.as_ref().map(payload_string)).collect();
        let values: [Option<i64>; 2] = operands.map(|o| o.and_then(|l| objects::decode_i64(l.payload()).ok()));
        let action = offered.offer.action;
        let produced = offered.offer.active().and_then(|a| {
            a.provisional()
                .map(|_| String::from_utf8_lossy(a.lease().payload()).to_string())
        });
        let produced_bytes = offered
            .offer
            .active()
            .and_then(|a| a.provisional().map(|_| a.lease().payload().to_vec()));
        let active_byte = offered.offer.active().map(objects::ActiveLease::byte);
        let arithmetic = matches!(action, Action::AddAB | Action::SubAB);
        let mut value_correct: Option<bool> = None;
        if arithmetic {
            arithmetic_steps += 1;
            if let (Some(a), Some(b)) = (values[0], values[1]) {
                if let Some(expected) = exact_value(action, a, b) {
                    let want = canonical(expected);
                    value_correct = Some(produced_bytes.as_deref() == Some(want.as_slice()));
                    if value_correct == Some(true) {
                        correct_value_steps += 1;
                    } else {
                        wrong_value_steps += 1;
                    }
                }
            }
            if let Some(byte) = active_byte {
                if emitted == u16::from(byte) {
                    copy_steps += 1;
                } else {
                    noncopy_steps += 1;
                }
            }
        }
        let intended_pair = payloads[0].as_deref() == Some(span_string(&example.prompt, spans[0]).as_str())
            && payloads[1].as_deref() == Some(span_string(&example.prompt, spans[1]).as_str());
        intended_pair_steps += u64::from(intended_pair);
        if record {
            steps.push(json!({
                "target": target,
                "emitted": emitted,
                "match": emitted == target,
                "action": action_name(action),
                "action_index": action_index(action),
                "selected": offered.trace.selected.iter().map(|r| match r {
                    None => Value::Null,
                    Some(objects::Reference::Occurrence { start, end, .. }) =>
                        json!({"kind": "occurrence", "start": start, "end": end}),
                    Some(objects::Reference::Result { id, .. }) => json!({"kind": "result", "id": id}),
                }).collect::<Vec<_>>(),
                "payloads": payloads,
                "numeric_valid": offered.trace.numeric_valid,
                "scalar_decodes": offered.trace.scalar_decodes,
                "selected_operations": offered.trace.selected_operations,
                "produced": produced,
                "value_correct": value_correct,
                "produced_cursor_byte": active_byte,
                "emitted_is_produced_cursor_byte": active_byte.map(|b| emitted == u16::from(b)),
                "intended_pair_selected": intended_pair,
                "cursor": offered.trace.cursor,
                "contexts": offered.trace.contexts.iter().map(|c| c.map(u64::from)).collect::<Vec<_>>(),
            }));
        }
        let actual = symbol_of(target)?;
        let trace = runtime.observe(actual, offered.offer.id, g, &mut policy)?;
        hash.update(format!("{trace:?}").as_bytes());
        generated.push(emitted);
        if emitted == 256 {
            break;
        }
    }
    let expected: Vec<u16> = example
        .answer
        .iter()
        .map(|&b| u16::from(b))
        .chain([256])
        .collect();
    let work = runtime.work();
    let exact = generated == expected;
    Ok(json!({
        "example_index": index,
        "id": example.id,
        "family": example.family,
        "prompt": String::from_utf8_lossy(&example.prompt).to_string(),
        "expected": String::from_utf8_lossy(&example.answer).to_string(),
        "expected_scalar": example.expected_scalar,
        "control": format!("{control:?}"),
        "arm": format!("{arm:?}"),
        "correct_symbols": correct_symbols,
        "positions": positions,
        "teacher_forced_exact": exact,
        "generated": generated,
        "arithmetic_steps": arithmetic_steps,
        "intended_pair_steps": intended_pair_steps,
        "correct_value_steps": correct_value_steps,
        "wrong_value_steps": wrong_value_steps,
        "emitted_equals_produced_byte_steps": copy_steps,
        "emitted_differs_from_produced_byte_steps": noncopy_steps,
        "work": work_json(work),
        "stats": policy.stats.json(),
        "read_plan": read_plan,
        "trace_digest": hash.finalize().to_hex().to_string(),
        "first_generated_state_digest": first_generated_state_digest,
        "steps": steps,
    }))
}

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

#[derive(Default)]
struct Aggregate {
    correct_symbols: u64,
    positions: u64,
    exact: u64,
    arithmetic_steps: u64,
    intended_pair_steps: u64,
    correct_value_steps: u64,
    wrong_value_steps: u64,
    copy_steps: u64,
    noncopy_steps: u64,
    selected_operations: u64,
    scalar_decodes: u64,
    circuit_calls: u64,
    candidate_scores: u64,
    stats: Stats,
    forced_read_ok: u64,
    forced_read_total: u64,
}
impl Aggregate {
    fn absorb(&mut self, row: &Value) {
        let u = |v: &Value| v.as_u64().unwrap_or(0);
        self.correct_symbols += u(&row["correct_symbols"]);
        self.positions += u(&row["positions"]);
        self.exact += u64::from(row["teacher_forced_exact"].as_bool().unwrap_or(false));
        self.arithmetic_steps += u(&row["arithmetic_steps"]);
        self.intended_pair_steps += u(&row["intended_pair_steps"]);
        self.correct_value_steps += u(&row["correct_value_steps"]);
        self.wrong_value_steps += u(&row["wrong_value_steps"]);
        self.copy_steps += u(&row["emitted_equals_produced_byte_steps"]);
        self.noncopy_steps += u(&row["emitted_differs_from_produced_byte_steps"]);
        self.selected_operations += u(&row["work"]["selected_operations"]);
        self.scalar_decodes += u(&row["work"]["scalar_decodes"]);
        self.circuit_calls += u(&row["work"]["circuit_calls"]);
        self.candidate_scores += u(&row["work"]["candidate_scores"]);
        let s = &row["stats"];
        let mut st = Stats {
            control_calls: u(&s["control_calls"]),
            add_offered: u(&s["add_offered"]),
            sub_offered: u(&s["sub_offered"]),
            arith_offered_steps: u(&s["arithmetic_offered_steps"]),
            chosen_add: u(&s["chosen_add"]),
            chosen_sub: u(&s["chosen_sub"]),
            ..Stats::default()
        };
        for (i, slot) in st.legal_sizes.iter_mut().enumerate() {
            *slot = s["legal_size_histogram"][i].as_u64().unwrap_or(0);
        }
        st.forced_root = u(&s["forced_root"]);
        st.forced_null = u(&s["forced_null"]);
        st.forced_extent = u(&s["forced_extent"]);
        st.forced_extent_rejected = u(&s["forced_extent_rejected"]);
        st.forced_op = u(&s["forced_op"]);
        st.forced_op_rejected = u(&s["forced_op_rejected"]);
        self.stats.merge(&st);
        if let Some(plan) = row["read_plan"].as_array() {
            for read in plan {
                self.forced_read_total += 1;
                self.forced_read_ok += u64::from(read["intended_wins"].as_bool().unwrap_or(false));
            }
        }
    }
    fn json(&self) -> Value {
        json!({
            "correct_symbols": self.correct_symbols,
            "positions": self.positions,
            "teacher_forced_exact_responses": self.exact,
            "arithmetic_steps": self.arithmetic_steps,
            "intended_pair_steps": self.intended_pair_steps,
            "correct_value_steps": self.correct_value_steps,
            "wrong_value_steps": self.wrong_value_steps,
            "emitted_equals_produced_byte_steps": self.copy_steps,
            "emitted_differs_from_produced_byte_steps": self.noncopy_steps,
            "selected_operations": self.selected_operations,
            "scalar_decodes": self.scalar_decodes,
            "circuit_calls": self.circuit_calls,
            "candidate_scores": self.candidate_scores,
            "forced_read_ok": self.forced_read_ok,
            "forced_read_total": self.forced_read_total,
            "stats": self.stats.json(),
        })
    }
}

// -------------------------------------------------------------------- io

fn emit(base: &Path, name: &str, value: &Value) -> AnyResult<PathBuf> {
    let root = base.join(name);
    report_output::claim(&root)?;
    std::fs::write(root.join("report.json"), serde_json::to_vec_pretty(value)?)?;
    report_output::seal(&root)?;
    let files = report_output::verify(&root)?;
    println!("[oracle] sealed {} files={}", root.display(), files.len());
    Ok(root)
}

fn prepare_base(base: &str) -> AnyResult<PathBuf> {
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
    std::fs::create_dir_all(&base)?;
    Ok(base)
}

// ------------------------------------------------------------------ modes

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

fn mode_criteria(base: &Path) -> AnyResult<()> {
    let start = Instant::now();
    let value: Value = serde_json::from_str(CRITERIA)?;
    let shape_note = json!({
        "criteria": value,
        "criteria_digest": blake3::hash(CRITERIA.as_bytes()).to_hex().to_string(),
        "bin_source_digest": blake3::hash(include_bytes!("addressed-attention-arith-oracle.rs"))
            .to_hex()
            .to_string(),
        "split_size": PROBLEMS.len(),
        "problem_table": PROBLEMS.iter().enumerate().map(|(i, &(a, b, op))| json!({
            "index": i,
            "a": a,
            "b": b,
            "op": if op == 0 { "Add" } else { "Sub" },
            "expected_scalar": if op == 0 { a + b } else { a - b },
        })).collect::<Vec<_>>(),
        "candidate_shapes": Shape::ALL.iter().map(|s| s.name()).collect::<Vec<_>>(),
        "elapsed_ms": start.elapsed().as_millis(),
    });
    emit(base, "criteria", &shape_note)?;
    Ok(())
}

fn mode_diagnose(base: &Path, params: &Parameters, model: &Model) -> AnyResult<()> {
    let start = Instant::now();
    let g = model.geometry();
    let mut rows: Vec<Value> = Vec::new();
    for shape in Shape::ALL {
        let examples = split(shape);
        let mut aggregate = Aggregate::default();
        let mut offers_any = false;
        for (index, example) in examples.iter().enumerate() {
            let row = drive(
                g,
                params,
                model.compiled(),
                example,
                index,
                Control::Full,
                Arm::Production,
                shape,
                false,
            )?;
            aggregate.absorb(&row);
            if row["stats"]["arithmetic_offered_steps"].as_u64().unwrap_or(0) > 0 {
                offers_any = true;
            }
        }
        let agg = aggregate.json();
        let calls = agg["stats"]["control_calls"].as_u64().unwrap_or(0);
        let offers = agg["stats"]["arithmetic_offered_steps"].as_u64().unwrap_or(0);
        rows.push(json!({
            "shape": shape.name(),
            "prompt_example": String::from_utf8_lossy(&examples[0].prompt).to_string(),
            "any_example_offered": offers_any,
            "offer_fraction": if calls == 0 { 0.0 } else { offers as f64 / calls as f64 },
            "aggregate": agg,
        }));
        println!(
            "[oracle] diagnose shape={} calls={} offered={} chosen={} intended_pair_steps={}",
            shape.name(),
            calls,
            offers,
            agg["stats"]["chosen_arithmetic"],
            agg["intended_pair_steps"],
        );
    }
    let value = json!({
        "schema": "uor-r4.addressed-attention-arith-oracle-diagnose/1",
        "purpose": "offers-only structural diagnostic: which prompt shape places bare canonical decimals where the read can select them. No expected answer, emitted symbol or correctness signal is used to choose the shape.",
        "param_seed": params.seed(),
        "criteria_digest_a1": A1_MIN_OFFER_FRACTION,
        "shapes": rows,
        "elapsed_ms": start.elapsed().as_millis(),
    });
    emit(base, "diagnose", &value)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn arm_rows(
    g: &BoundGeometry,
    params: &Parameters,
    model: &Model,
    shape: Shape,
    control: Control,
    arm: Arm,
    record: bool,
) -> AnyResult<(Aggregate, Vec<Value>)> {
    let examples = split(shape);
    let mut aggregate = Aggregate::default();
    let mut rows = Vec::new();
    for (index, example) in examples.iter().enumerate() {
        let row = drive(
            g,
            params,
            model.compiled(),
            example,
            index,
            control,
            arm,
            shape,
            record,
        )?;
        aggregate.absorb(&row);
        rows.push(row);
    }
    Ok((aggregate, rows))
}

fn mode_run(base: &Path, params: &Parameters, model: &Model, shape: Shape) -> AnyResult<()> {
    let start = Instant::now();
    let g = model.geometry();
    let examples = split(shape);
    let digest = pilot_data::corpus_digest(&examples);

    // --- authoritative crate metrics: pilot::evaluate for Full and 3 controls
    let mut official = serde_json::Map::new();
    let mut official_rows: serde_json::Map<String, Value> = serde_json::Map::new();
    for control in pilot::CONTROLS {
        let evaluated = pilot::evaluate(params, model.compiled(), g, &examples, control)?;
        let metrics = pilot::metrics(&evaluated);
        official.insert(format!("{control:?}"), serde_json::to_value(&metrics)?);
        official_rows.insert(format!("{control:?}"), serde_json::to_value(&evaluated)?);
        println!(
            "[oracle] pilot::evaluate {control:?} correct={}/{} exact={}/{} ce={:.6}",
            metrics.correct_symbols,
            metrics.positions,
            metrics.exact_responses,
            metrics.examples,
            metrics.mean_response_ce
        );
    }

    // --- instrumented arms: same recipe, plus traces and counters
    let mut arms = serde_json::Map::new();
    let mut arm_aggregates: serde_json::Map<String, Value> = serde_json::Map::new();
    for control in pilot::CONTROLS {
        let (aggregate, rows) = arm_rows(g, params, model, shape, control, Arm::Production, true)?;
        let agg = aggregate.json();
        println!(
            "[oracle] instrumented {:?} calls={} offered={} chosen={} selected_operations={} correct={}/{} exact={}",
            control,
            agg["stats"]["control_calls"],
            agg["stats"]["arithmetic_offered_steps"],
            agg["stats"]["chosen_arithmetic"],
            agg["selected_operations"],
            agg["correct_symbols"],
            agg["positions"],
            agg["exact_responses"],
        );
        arm_aggregates.insert(format!("{control:?}"), agg);
        arms.insert(format!("{control:?}"), Value::Array(rows));
    }

    // --- secondary arm S4: InterpretedPolicy (uncompiled argmax)
    let interpreted = {
        let mut aggregate = Aggregate::default();
        let mut rows = Vec::new();
        for (index, example) in examples.iter().enumerate() {
            let epoch = index as u64 + 1;
            let mut runtime = RuntimeSession::new(params.digest(), g, epoch)?;
            let mut policy = Oracle::new(
                InterpretedPolicy { parameters: params },
                Arm::Production,
                0,
            );
            for &byte in &example.prompt {
                let offer = runtime.predict(g, &mut policy)?;
                runtime.observe(Symbol::Byte(byte), offer.offer.id, g, &mut policy)?;
            }
            let spans = intended(shape, index);
            let mut correct_symbols = 0u64;
            let mut positions = 0u64;
            let mut generated: Vec<u16> = Vec::new();
            let mut arithmetic_steps = 0u64;
            let mut intended_pair_steps = 0u64;
            let mut correct_value_steps = 0u64;
            let mut wrong_value_steps = 0u64;
            let mut copy_steps = 0u64;
            let mut noncopy_steps = 0u64;
            for target in example
                .answer
                .iter()
                .map(|&b| u16::from(b))
                .chain([256])
            {
                let offered = runtime.predict(g, &mut policy)?;
                let emitted = symbol_code(offered.offer.symbol);
                positions += 1;
                if emitted == target {
                    correct_symbols += 1;
                }
                let operands = offered.offer.operands();
                let values: [Option<i64>; 2] =
                    operands.map(|o| o.and_then(|l| objects::decode_i64(l.payload()).ok()));
                let action = offered.offer.action;
                if matches!(action, Action::AddAB | Action::SubAB) {
                    arithmetic_steps += 1;
                    if let (Some(a), Some(b)) = (values[0], values[1]) {
                        if let Some(expected) = exact_value(action, a, b) {
                            let want = canonical(expected);
                            let produced = offered
                                .offer
                                .active()
                                .and_then(|x| x.provisional().map(|_| x.lease().payload().to_vec()));
                            if produced.as_deref() == Some(want.as_slice()) {
                                correct_value_steps += 1;
                            } else {
                                wrong_value_steps += 1;
                            }
                        }
                    }
                    if let Some(byte) = offered.offer.active().map(objects::ActiveLease::byte) {
                        if emitted == u16::from(byte) {
                            copy_steps += 1;
                        } else {
                            noncopy_steps += 1;
                        }
                    }
                }
                let payloads: Vec<Option<String>> =
                    operands.iter().map(|o| o.as_ref().map(payload_string)).collect();
                let intended_pair =
                    payloads[0].as_deref() == Some(span_string(&example.prompt, spans[0]).as_str())
                        && payloads[1].as_deref()
                            == Some(span_string(&example.prompt, spans[1]).as_str());
                intended_pair_steps += u64::from(intended_pair);
                let actual = symbol_of(target)?;
                runtime.observe(actual, offered.offer.id, g, &mut policy)?;
                generated.push(emitted);
                if emitted == 256 {
                    break;
                }
            }
            let expected: Vec<u16> = example
                .answer
                .iter()
                .map(|&b| u16::from(b))
                .chain([256])
                .collect();
            let work = runtime.work();
            let row = json!({
                "example_index": index,
                "control": "InterpretedPolicy",
                "arm": "Production",
                "correct_symbols": correct_symbols,
                "positions": positions,
                "teacher_forced_exact": generated == expected,
                "arithmetic_steps": arithmetic_steps,
                "intended_pair_steps": intended_pair_steps,
                "correct_value_steps": correct_value_steps,
                "wrong_value_steps": wrong_value_steps,
                "emitted_equals_produced_byte_steps": copy_steps,
                "emitted_differs_from_produced_byte_steps": noncopy_steps,
                "work": work_json(work),
                "stats": policy.stats.json(),
                "read_plan": Value::Null,
            });
            aggregate.absorb(&row);
            rows.push(row);
        }
        (aggregate.json(), Value::Array(rows))
    };
    println!(
        "[oracle] instrumented InterpretedPolicy calls={} offered={} chosen={} selected_operations={} correct={}/{} exact={}",
        interpreted.0["stats"]["control_calls"],
        interpreted.0["stats"]["arithmetic_offered_steps"],
        interpreted.0["stats"]["chosen_arithmetic"],
        interpreted.0["selected_operations"],
        interpreted.0["correct_symbols"],
        interpreted.0["positions"],
        interpreted.0["exact_responses"],
    );

    // --- secondary arm S3: forced read, C1 (compiled control) and C2 (forced op)
    let mut forced = serde_json::Map::new();
    for (name, arm) in [("C1_forced_read", Arm::ForcedRead), ("C2_forced_read_and_op", Arm::ForcedReadOp)] {
        let (aggregate, rows) = arm_rows(g, params, model, shape, Control::Full, arm, true)?;
        let agg = aggregate.json();
        println!(
            "[oracle] {name} calls={} forced_read_ok={}/{} offered={} chosen={} selected_operations={} correct={}/{} exact={} value_ok={} copy={}",
            agg["stats"]["control_calls"],
            agg["forced_read_ok"],
            agg["forced_read_total"],
            agg["stats"]["arithmetic_offered_steps"],
            agg["stats"]["chosen_arithmetic"],
            agg["selected_operations"],
            agg["correct_symbols"],
            agg["positions"],
            agg["exact_responses"],
            agg["correct_value_steps"],
            agg["emitted_equals_produced_byte_steps"],
        );
        forced.insert(format!("{name}_aggregate"), agg);
        forced.insert(format!("{name}_rows"), Value::Array(rows));
    }

    // --- pre-registered branch rule
    let full = &arm_aggregates["Full"];
    let full_ce = official["Full"]["mean_response_ce"].as_f64().unwrap_or(f64::NAN);
    let controls = ["ReadDisabled", "ExactPayloadMasked", "StateTransportDisabled"];
    let mut control_verdicts = serde_json::Map::new();
    let mut beats_all = true;
    for control in controls {
        let arm = &arm_aggregates[control];
        let ce = official[control]["mean_response_ce"].as_f64().unwrap_or(f64::NAN);
        let strict_ce = full_ce < ce;
        let mut unique_exact = false;
        if let (Some(f), Some(c)) = (
            official_rows["Full"].as_array(),
            official_rows[control].as_array(),
        ) {
            unique_exact = f.iter().zip(c).any(|(fr, cr)| {
                fr["exact_response_and_eos"].as_bool().unwrap_or(false)
                    && !cr["exact_response_and_eos"].as_bool().unwrap_or(false)
            });
        }
        if !(strict_ce && unique_exact) {
            beats_all = false;
        }
        control_verdicts.insert(
            control.to_string(),
            json!({
                "full_mean_ce": full_ce,
                "control_mean_ce": ce,
                "full_ce_strictly_lower": strict_ce,
                "full_has_unique_exact_example": unique_exact,
                "full_correct_symbols": full["correct_symbols"],
                "control_correct_symbols": arm["correct_symbols"],
                "full_exact_responses": full["exact_responses"],
                "control_exact_responses": arm["exact_responses"],
            }),
        );
    }
    let full_nonzero = full["correct_symbols"].as_u64().unwrap_or(0) > 0;
    let offered = full["stats"]["arithmetic_offered_steps"].as_u64().unwrap_or(0);
    let calls = full["stats"]["control_calls"].as_u64().unwrap_or(0);
    let offer_fraction = if calls == 0 { 0.0 } else { offered as f64 / calls as f64 };
    let a1 = offer_fraction >= A1_MIN_OFFER_FRACTION;
    let a2 = full["selected_operations"].as_u64().unwrap_or(0) > 0;
    let a3 = official.len() == 4;
    let branch = if !a1 {
        "B3_SPLIT_DID_NOT_REACH_ARITHMETIC"
    } else if a2 && full_nonzero && beats_all {
        "B1_POSITIVE_MECHANISM_COMPUTES_WITH_RIGHT_DATA"
    } else {
        "B2_NULL_DEFICIT_IS_IN_THE_MECHANISM"
    };

    let value = json!({
        "schema": "uor-r4.addressed-attention-arith-oracle-run/1",
        "param_seed": params.seed(),
        "parameter_digest": params.digest().iter().map(|b| format!("{b:02x}")).collect::<String>(),
        "source_digest": implementation_digest().iter().map(|b| format!("{b:02x}")).collect::<String>(),
        "shape": shape.name(),
        "split": {
            "name": "arith-canonical-decimal/1",
            "examples": examples.len(),
            "digest": digest.iter().map(|b| format!("{b:02x}")).collect::<String>(),
            "answers": "CONSTRUCTED by this instrument: exact i64 a+b / a-b rendered by objects::encode_i64; not a frozen external fixture",
            "rows": examples.iter().map(|e| json!({
                "id": e.id,
                "family": e.family,
                "prompt": String::from_utf8_lossy(&e.prompt).to_string(),
                "answer": String::from_utf8_lossy(&e.answer).to_string(),
                "expected_scalar": e.expected_scalar,
            })).collect::<Vec<_>>(),
        },
        "criteria": serde_json::from_str::<Value>(CRITERIA)?,
        "criteria_digest": blake3::hash(CRITERIA.as_bytes()).to_hex().to_string(),
        "official_pilot_evaluate": official,
        "instrumented_aggregates": arm_aggregates,
        "interpreted_policy": {"aggregate": interpreted.0, "rows": interpreted.1},
        "forced_read": forced,
        "verdict": {
            "A1_offer_fraction": offer_fraction,
            "A1_threshold": A1_MIN_OFFER_FRACTION,
            "A1_reachability": a1,
            "A2_selection": a2,
            "A3_reporting": a3,
            "full_correct_symbols_nonzero": full_nonzero,
            "full_beats_all_three_controls": beats_all,
            "controls": control_verdicts,
            "branch": branch,
        },
        "rows_by_control": arms,
        "official_rows_by_control": official_rows,
        "elapsed_ms": start.elapsed().as_millis(),
        "caveats": [
            "All arms use unfitted Parameters::seeded(7341); there is no fit and no checkpoint.",
            "The split is authored here, not a frozen external corpus; its answers are constructed by the instrument.",
            "Forced-read arms are harness instruments (Policy overrides only). The engine, objects, geometry and compiled heads are unmodified.",
            "pilot::evaluate metrics are the authoritative scored surface; the instrumented arms replicate its recipe and add Work/trace counters."
        ],
    });
    emit(base, "run", &value)?;
    std::fs::write(
        base.join("run-summary.json"),
        serde_json::to_vec_pretty(&json!({
            "branch": branch,
            "A1": a1, "A2": a2, "A3": a3,
            "offer_fraction": offer_fraction,
            "full_correct_symbols": full["correct_symbols"],
            "full_positions": full["positions"],
            "full_exact": full["exact_responses"],
            "full_ce": full_ce,
            "beats_all_three_controls": beats_all,
            "official": official,
        }))?,
    )?;
    println!("[oracle] branch={branch} elapsed_ms={}", start.elapsed().as_millis());
    Ok(())
}

fn main() -> AnyResult<()> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build_global()
        .map_err(|e| format!("thread pool: {e}"))?;
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).ok_or(
        "usage: addressed-attention-arith-oracle <criteria|diagnose|run> <abs-base-dir> [shape]",
    )?;
    let base = prepare_base(args.get(2).ok_or("missing absolute base directory")?)?;
    let started = Instant::now();
    match mode.as_str() {
        "criteria" => {
            mode_criteria(&base)?;
            println!("[oracle] criteria sealed base={}", base.display());
            return Ok(());
        }
        "diagnose" => {
            let params = Parameters::seeded(PARAMETER_SEED)?;
            let model = build_model(&params)?;
            mode_diagnose(&base, &params, &model)?;
        }
        "run" => {
            let shape_name = args.get(3).map(String::as_str).ok_or(
                "usage: addressed-attention-arith-oracle run <abs-base-dir> <space|concat|newline|plus>",
            )?;
            let shape = *Shape::ALL
                .iter()
                .find(|s| s.name() == shape_name)
                .ok_or("unknown shape")?;
            let params = Parameters::seeded(PARAMETER_SEED)?;
            let model = build_model(&params)?;
            mode_run(&base, &params, &model, shape)?;
        }
        other => return Err(format!("unknown mode {other}").into()),
    }
    println!("[oracle] done elapsed_ms={} base={}", started.elapsed().as_millis(), base.display());
    Ok(())
}
