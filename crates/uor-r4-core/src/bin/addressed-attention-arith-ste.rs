//! Opt-in relaxed/straight-through fit instrument for the addressed-attention
//! emission context, plus the pre-registered S1-S5 readouts.
//!
//! Modes:
//!   criteria <new-absolute-report-base>   seal the frozen criteria first
//!   run      <new-absolute-report-base>   the one pre-registered fit + evaluation
//!
//! This binary changes no production default path: it drives the unmodified engine
//! through `CompiledPolicy`/`Controlled`, and the only new production code it calls
//! is the explicitly selected `addressed_attention::ste` training entry point.
use std::path::Path;
use std::time::Instant;

use serde_json::{json, Value};
use uor_r4_core::native_geometric::addressed_attention::artifact::{
    BoundGeometry, Model, Provenance,
};
use uor_r4_core::native_geometric::addressed_attention::circuit::{PrimitiveExport, GATE_LOGITS};
use uor_r4_core::native_geometric::addressed_attention::engine::{
    self, Head, Policy, RuntimeSession, Work,
};
use uor_r4_core::native_geometric::addressed_attention::inputs::Phase;
use uor_r4_core::native_geometric::addressed_attention::objects::{self, Action, Lease, Symbol};
use uor_r4_core::native_geometric::addressed_attention::pilot::{Control, Controlled};
use uor_r4_core::native_geometric::addressed_attention::policy::{
    head_range, implementation_digest, Parameters,
};
use uor_r4_core::native_geometric::addressed_attention::ste::{
    self, SteConfig, StePair, SCORE_FUNCTION_GATE_RMS,
};
use uor_r4_core::report_output;

type AnyResult<T> = Result<T, Box<dyn std::error::Error>>;

const PARAMETER_SEED: u64 = 7341;
const MODEL_DATA: &[u8] = b"addressed-attention-arith-ste/1";
const CONFIG_DIGEST: &[u8] = b"addressed-attention-arith-ste-config/1";

/// Byte-channel offsets in the packed 1024-bit input (inputs.rs).
const BIT_A_FIRST_BYTE: usize = 776;
const BIT_B_FIRST_BYTE: usize = 784;
const BIT_ACTIVE_BYTE: usize = 792;

const OP_ADD: usize = 3;
const OP_SUB: usize = 4;
const SYMBOL_EOS: u16 = 256;

/// Fit-loop wall-clock cap, checked at update boundaries only.
const FIT_CAP_MS: u128 = 300_000;
/// Whole-run cap, charged across capture, fit and every evaluation arm.
const ATTEMPT_CAP_MS: u128 = 1_800_000;
/// Diagnostic cadence for the in-fit trajectory.
const DIAGNOSTIC_EVERY: u64 = 250;

/// (a, b, op) with op 0 = Add, 1 = Sub. Copied in order from the predecessor bins.
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

/// The nine examples of the sealed emission-reach probe, in the same order and with
/// the same space-separated shape. Used only for the S2 sweep readout.
const REACH: [(i64, i64); 9] = [
    (4, 5),
    (1, 2),
    (1, 14),
    (2, 3),
    (3, 4),
    (4, 4),
    (5, 7),
    (6, 8),
    (9, 9),
];

const CRITERIA: &str = r#"{
  "schema": "uor-r4.addressed-attention-arith-ste-criteria/1",
  "frozen": "written and sealed before any fit was executed in this session; no threshold, rate, step count or schedule below was adjusted after seeing a fit result",
  "question": "Does an opt-in straight-through / relaxed estimator over the gate LUT cascade put usable gradient on the discrete gate truth tables, so the cascade learns to route the computed digit carried at input bits 792..800 into the 9-bit emission context? The located defect: at Parameters::seeded(7341) the hash-like cascade erases that digit - sweeping its byte field over 0..255 with the other 1023 bits fixed yields 2-4 distinct contexts and is constant in 3 of 9 examples, and the preserved score-function fit (aa-arith-emit-fit-20261005-210113) moved only 331 of 49,030 compiled bytes with copy rate 0/80 before and after.",
  "baseline_measured_this_session": {
    "worktree": "~/uor-r4-worktrees/aa-ste at origin/main 12f5caa5, unmodified; these are frozen-parameter measurements, no training was run before them",
    "b1_emission_reach_sealed_root": "aa-ste-before-emission-reach-20261006-013232",
    "b1_active_byte_sweep_distinct_contexts": [2, 1, 2, 2, 1, 2, 4, 2, 1],
    "b1_mean_distinct_contexts": 1.8888888888888888,
    "b1_constant_context_examples": 3,
    "b2_emission_context_trace_root": "aa-ste-before-emission-context-trace-20261006-013235",
    "b2_distinct_step0_contexts": 23,
    "b2_conflicting_target_contexts": 5,
    "b2_examples_in_conflicting_contexts": 13,
    "b3_predecessor_score_function_fit": {
      "sealed_root": "aa-arith-emit-fit-20261005-210113",
      "updates": 78,
      "changed_compiled_bytes": 331,
      "compiled_bytes": 49030,
      "gate_gradient_l2_at_update_1": 2.004,
      "head_gradient_l2_at_update_1": 0.379,
      "parameter_delta_l2": 0.9404917143011023,
      "copy_steps_pre": 0,
      "copy_steps_post": 0,
      "arithmetic_steps": 80
    }
  },
  "primary_criteria": {
    "S1_fit_runs_and_moves_the_circuit": "the relaxed fit runs to completion without error and changes at least 3x the 331 baseline = 993 of the 49,030 compiled export bytes, with parameter_delta_l2 > 0. This is a mechanism-activity threshold, not a quality threshold.",
    "S2_digit_reaches_context": "primary readout. (a) constant-context examples in the 256-value sweep of byte field 792..800 fall from 3/9 to at most 1/9; (b) the mean distinct-context count over the nine examples strictly exceeds the 1.889 baseline and at least two examples exceed the baseline maximum of 4.",
    "S3_collision_structure": "over the same 32-example forced-read+forced-operator trace used by the sealed emission-context bin, the distinct step-0 context count is below the 23 baseline AND the number of examples sharing a context with a conflicting target is below the 13 baseline.",
    "S4_copy_rate": "in the forced-read+forced-operator arm over the arithmetic split, copy_steps (emitted symbol equals the produced cursor byte) leaves 0/80, i.e. is at least 1 of 80. Reported for the fit split (even global indices) and the holdout split (odd) separately as well as the total.",
    "S5_operator_stays_exact": "in the same forced arm, correct_value_steps equals arithmetic_steps (80/80) both pre-fit and post-fit, and the addressed_attention module test suite still passes.",
    "decision": "POSITIVE iff S1 and S2 and (S3 or S4); otherwise NULL recorded with its numbers"
  },
  "recipe_one_only": {
    "capture": "the forced-read+forced-operator arm over all 32 examples (newline shape, same PROBLEMS table as the predecessor fit); for every Emit consult the packed 1024-bit input and the teacher-forced target are recorded. EOS steps are excluded from the fit by design: the defect is the digit routing and one context must not be asked to emit both a digit and EOS.",
    "fit_split": "even global indices only, the same split rule as the predecessor score-function fit",
    "trained_blocks": "gate logits [0, GATE_LOGITS) through the relaxed estimator, and the 512 emission rows through the crate's hard-context cross_entropy. All other blocks (roots, extent, control, null) are frozen, so the fitted artifact differs from the frozen parent only where the discrete routing is learned.",
    "optimizer": "the crate's own learner::sgd, theta' = theta - rate*gradient, rate 0.05 for both blocks, no momentum, clipping, decay or restarts",
    "gate_scale": "the relaxed gate block is normalised per update to RMS 0.026973 (= 2.004/sqrt(5520), the predecessor fit's measured first-update gate-gradient L2 over its 5520 gate logits), so every update has the predecessor's own step size and any difference is attributable to the direction, not the magnitude",
    "head_scale": "raw conditional cross-entropy of the target at the hard context, the crate's training::cross_entropy with normalizer 1",
    "updates": 6000,
    "pair_order": "single pair per update, cycling the fit pairs in index order",
    "temperature": "geometric anneal 1.0 -> 0.1 over the 6000 updates; 1.0 is the crate's existing Bernoulli sampling probability and 0.1 keeps a non-degenerate sigmoid derivative in f64 (exact saturation needs |logit| >= 3.7) while making each relaxed bit agree with its hard bit to 1e-5 for |logit| >= 0.5",
    "instrumented_diagnostics": "every 250 updates the fit records the S2 sweep, the distinct hard contexts over the fit pairs and the fit-pair emission accuracy; diagnostics do not alter the update",
    "no_sweep": "one recipe, one run; the first complete run is the result, including any early stop at a wall-clock cap"
  },
  "budget": {
    "fit_wall_clock_cap_ms": 300000,
    "whole_run_cap_ms": 1800000,
    "cpu": "local CPU only; no GPU, no pod, no external cost",
    "storage": "one newly and exclusively claimed report root under ~/uor-r4-worktrees/reports/"
  },
  "branch_rule": {
    "POSITIVE": "S1 and S2 and (S3 or S4)",
    "NULL": "otherwise, recorded with numbers; a null is the useful statement that this estimator is still too weak or that the cascade cannot represent the routing",
    "STOP": "if the engine, the fit or the crate's own tests fail, stop and report the exact blocker rather than shipping a partial mechanism"
  },
  "honesty": [
    "The fit consumes Emit inputs captured with the frozen parent parameters; the final S2-S5 numbers are measured by driving the unmodified engine end-to-end with the fitted parameters, but the training signal itself is not re-captured as the parameters move.",
    "The 32-example split is authored in this instrument by i64 arithmetic; it is a bounded mechanism probe, not an independent corpus and not a language result.",
    "S1 is an activity threshold: moving many compiled bytes is necessary for the mechanism to be alive, not evidence of quality; S2-S4 carry the quality claim.",
    "The S2 sweep examples are a different nine-problem set from the 32-example fit split; the sweep is a structural readout of the circuit's response to byte field 792..800.",
    "The default hard cascade is untouched: the relaxed estimator is reachable only through the explicitly named ste module, and the before/after comparison of the existing bins is the evidence."
  ]
}"#;

// ------------------------------------------------------------------- split

fn scalar_of(index: usize) -> i64 {
    let (a, b, op) = PROBLEMS[index];
    if op == 0 {
        a + b
    } else {
        a - b
    }
}
fn canonical(value: i64) -> Vec<u8> {
    value.to_string().into_bytes()
}
fn exact_value(action: Action, a: i64, b: i64) -> Option<i64> {
    match action {
        Action::AddAB => a.checked_add(b),
        Action::SubAB => a.checked_sub(b),
        _ => None,
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
        other => Err(format!("no symbol for code {other}").into()),
    }
}
fn op_of(index: usize) -> usize {
    if PROBLEMS[index].2 == 0 {
        OP_ADD
    } else {
        OP_SUB
    }
}
fn intended(index: usize) -> [(u64, u8); 2] {
    let (a, b, _) = PROBLEMS[index];
    let la = a.to_string().len() as u64;
    let lb = b.to_string().len() as u8;
    [(0, la as u8), (la + 1, lb)]
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
fn payload_string(lease: &Lease) -> String {
    String::from_utf8_lossy(lease.payload()).to_string()
}
fn span_string(prompt: &[u8], span: (u64, u8)) -> String {
    let start = span.0 as usize;
    let end = start + usize::from(span.1);
    String::from_utf8_lossy(&prompt[start..end]).to_string()
}

// ------------------------------------------------- forced-read instrument

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    Production,
    ForcedReadOp,
}

#[derive(Clone, Copy, Debug, Default)]
struct Plan {
    q: [[u16; 2]; 2],
    n: [[u16; 2]; 2],
    len: [u8; 2],
}
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
enum Winner {
    Occurrence(u64),
    Result(u64),
    None,
}
#[derive(Clone, Copy, Debug, Default)]
struct Stats {
    control_calls: u64,
    arith_offered_steps: u64,
    chosen_add: u64,
    chosen_sub: u64,
}
impl Stats {
    fn record(&mut self, legal: &[bool], choice: usize) {
        self.control_calls += 1;
        let add = legal.get(OP_ADD).copied().unwrap_or(false);
        let sub = legal.get(OP_SUB).copied().unwrap_or(false);
        self.arith_offered_steps += u64::from(add || sub);
        self.chosen_add += u64::from(choice == OP_ADD);
        self.chosen_sub += u64::from(choice == OP_SUB);
    }
    fn merge(&mut self, other: &Stats) {
        self.control_calls += other.control_calls;
        self.arith_offered_steps += other.arith_offered_steps;
        self.chosen_add += other.chosen_add;
        self.chosen_sub += other.chosen_sub;
    }
    fn json(&self) -> Value {
        json!({
            "control_calls": self.control_calls,
            "arithmetic_offered_steps": self.arith_offered_steps,
            "chosen_add": self.chosen_add,
            "chosen_sub": self.chosen_sub,
            "chosen_arithmetic": self.chosen_add + self.chosen_sub,
        })
    }
}

/// One captured Emit consult.
#[derive(Clone, Copy, Debug)]
struct Consult {
    input: [bool; 1024],
    target: u16,
    context: u16,
    emitted: u16,
    arithmetic: bool,
    produced_cursor_byte: Option<u8>,
    value_correct: Option<bool>,
}

struct Capture<P: Policy> {
    inner: P,
    arm: Arm,
    phase: Phase,
    force: bool,
    plan: Plan,
    intended_op: usize,
    /// Target the driver is currently teacher-forcing at this position.
    current_target: u16,
    /// Packed input seen by the most recent Emit context call.
    emit_input: Option<[bool; 1024]>,
    consults: Vec<Consult>,
    stats: Stats,
}
impl<P: Policy> Capture<P> {
    fn new(inner: P, arm: Arm, intended_op: usize) -> Self {
        Self {
            inner,
            arm,
            phase: Phase::QueryA,
            force: false,
            plan: Plan::default(),
            intended_op,
            current_target: SYMBOL_EOS,
            emit_input: None,
            consults: Vec::new(),
            stats: Stats::default(),
        }
    }
}
impl<P: Policy> Policy for Capture<P> {
    fn context(&mut self, phase: Phase, input: &[bool; 1024]) -> engine::Result<u16> {
        self.phase = phase;
        if phase == Phase::Emit {
            self.emit_input = Some(*input);
        }
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
                }
            }
            if let (Some(which), Head::Null(h)) = (read, head) {
                if h < 2 {
                    choice = usize::from(self.plan.n[which][usize::from(h)]);
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
                    }
                }
            }
            if self.arm == Arm::ForcedReadOp {
                if let Head::Control = head {
                    if legal.get(self.intended_op).copied().unwrap_or(false) {
                        choice = self.intended_op;
                    }
                }
            }
        }
        if let Head::Control = head {
            self.stats.record(legal, choice);
        }
        if let Head::Emission = head {
            if let Some(input) = self.emit_input {
                self.consults.push(Consult {
                    input,
                    target: self.current_target,
                    context,
                    emitted: choice as u16,
                    arithmetic: false,
                    produced_cursor_byte: None,
                    value_correct: None,
                });
            }
        }
        Ok(choice)
    }
}

fn compute_plan(
    g: &BoundGeometry,
    runtime: &RuntimeSession,
    epoch: u64,
    spans: [(u64, u8); 2],
) -> AnyResult<Plan> {
    let frontier = runtime.objects().frontier();
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
        for id in runtime.objects().result_frontier().saturating_sub(8).max(1)
            ..runtime.objects().result_frontier()
        {
            let rec = runtime.objects().result(epoch, id)?;
            let c = g.score(q, rec.lease.keys())?;
            if c > score {
                score = c;
                winner = Winner::Result(id);
            }
        }
        let maximum = runtime.objects().maximum_extent(epoch, start)?;
        let _ = winner;
        plan.q[which] = q;
        plan.n[which] = n;
        plan.len[which] = len;
        if maximum < len {
            return Err(format!("extent {maximum} < intended {len}").into());
        }
    }
    Ok(plan)
}

/// Per-step trace of one arithmetic example under the forced arm.
#[derive(Clone, Debug)]
struct StepTrace {
    step: usize,
    target: u16,
    context: u16,
    emitted: u16,
    produced_cursor_byte: Option<u8>,
    value_correct: Option<bool>,
    arithmetic: bool,
    input: [bool; 1024],
}

#[derive(Clone, Debug)]
struct ExampleTrace {
    index: usize,
    steps: Vec<StepTrace>,
    stats: Stats,
    work: Work,
    correct_value_steps: u64,
    wrong_value_steps: u64,
}

/// Drive one example with teacher forcing, forcing the intended read and operator.
/// Contexts and packed Emit inputs come from the unmodified engine; nothing here
/// chooses a context or an emission target.
fn drive_example(
    g: &BoundGeometry,
    params: &Parameters,
    compiled: &PrimitiveExport,
    index: usize,
    arm: Arm,
) -> AnyResult<ExampleTrace> {
    let epoch = index as u64 + 1;
    let (a, b, op) = PROBLEMS[index];
    let prompt = format!("{a}\n{b}\n").into_bytes();
    let answer = canonical(scalar_of(index));
    let spans = intended(index);
    let mut runtime = RuntimeSession::new(params.digest(), g, epoch)?;
    let mut policy = Capture::new(
        Controlled::new(compiled, Control::Full, g.identity()),
        arm,
        op_of(index),
    );
    for &byte in &prompt {
        let offer = runtime.predict(g, &mut policy)?;
        runtime.observe(Symbol::Byte(byte), offer.offer.id, g, &mut policy)?;
    }
    let mut steps = Vec::new();
    let mut correct_value_steps = 0u64;
    let mut wrong_value_steps = 0u64;
    for (step, target) in answer
        .iter()
        .map(|&byte| u16::from(byte))
        .chain([SYMBOL_EOS])
        .enumerate()
    {
        if arm != Arm::Production {
            let plan = compute_plan(g, &runtime, epoch, spans)?;
            policy.plan = plan;
            policy.force = true;
        }
        policy.current_target = target;
        let before = policy.consults.len();
        let offered = runtime.predict(g, &mut policy)?;
        let emitted = symbol_code(offered.offer.symbol);
        let action = offered.offer.action;
        let arithmetic = matches!(action, Action::AddAB | Action::SubAB);
        let produced = offered.offer.active().and_then(|active| {
            active
                .provisional()
                .map(|_| active.lease().payload().to_vec())
        });
        let cursor_byte = offered.offer.active().map(objects::ActiveLease::byte);
        let operands = offered.offer.operands();
        let values: [Option<i64>; 2] =
            operands.map(|o| o.and_then(|lease| objects::decode_i64(lease.payload()).ok()));
        let mut value_correct = None;
        if arithmetic {
            if let (Some(x), Some(y)) = (values[0], values[1]) {
                if let Some(expected) = exact_value(action, x, y) {
                    let want = canonical(expected);
                    let correct = produced.as_deref() == Some(want.as_slice());
                    value_correct = Some(correct);
                    if correct {
                        correct_value_steps += 1;
                    } else {
                        wrong_value_steps += 1;
                    }
                }
            }
        }
        if let Some(consult) = policy.consults[before..].last_mut() {
            consult.arithmetic = arithmetic;
            consult.produced_cursor_byte = cursor_byte;
            consult.value_correct = value_correct;
        }
        let (context, input) = policy.consults[before..]
            .last()
            .map(|c| (c.context, c.input))
            .ok_or("no emission consult on this step")?;
        steps.push(StepTrace {
            step,
            target,
            context,
            emitted,
            produced_cursor_byte: cursor_byte,
            value_correct,
            arithmetic,
            input,
        });
        let _ = op;
        runtime.observe(symbol_of(target)?, offered.offer.id, g, &mut policy)?;
        if emitted == SYMBOL_EOS {
            break;
        }
    }
    Ok(ExampleTrace {
        index,
        steps,
        stats: policy.stats,
        work: runtime.work(),
        correct_value_steps,
        wrong_value_steps,
    })
}

/// The forced-arm arithmetic totals for a set of examples: S4 and S5.
fn forced_totals(
    g: &BoundGeometry,
    params: &Parameters,
    compiled: &PrimitiveExport,
    indices: &[usize],
) -> AnyResult<Value> {
    let mut arithmetic_steps = 0u64;
    let mut correct_value_steps = 0u64;
    let mut copy_steps = 0u64;
    let mut rows = Vec::new();
    let mut stats = Stats::default();
    for &index in indices {
        let trace = drive_example(g, params, compiled, index, Arm::ForcedReadOp)?;
        let mut local_copy = 0u64;
        let mut local_arith = 0u64;
        for step in &trace.steps {
            if step.arithmetic {
                local_arith += 1;
                if let Some(byte) = step.produced_cursor_byte {
                    if step.emitted == u16::from(byte) {
                        local_copy += 1;
                    }
                }
            }
        }
        arithmetic_steps += local_arith;
        correct_value_steps += trace.correct_value_steps;
        copy_steps += local_copy;
        stats.merge(&trace.stats);
        rows.push(json!({
            "index": index,
            "arithmetic_steps": local_arith,
            "correct_value_steps": trace.correct_value_steps,
            "wrong_value_steps": trace.wrong_value_steps,
            "copy_steps": local_copy,
            "control_stats": trace.stats.json(),
        }));
    }
    Ok(json!({
        "examples": indices.len(),
        "arithmetic_steps": arithmetic_steps,
        "correct_value_steps": correct_value_steps,
        "copy_steps": copy_steps,
        "control_stats": stats.json(),
        "rows": rows,
    }))
}

/// The S3 readout: step-0 context and target of every example, with the collision
/// structure the sealed emission-context bin reports.
fn collision_structure(
    g: &BoundGeometry,
    params: &Parameters,
    compiled: &PrimitiveExport,
    indices: &[usize],
) -> AnyResult<Value> {
    let mut groups: Vec<(u16, Vec<(usize, u16)>)> = Vec::new();
    let mut values = Vec::new();
    for &index in indices {
        let trace = drive_example(g, params, compiled, index, Arm::ForcedReadOp)?;
        let first = trace.steps.first().ok_or("no first answer step")?;
        match groups.iter_mut().find(|(c, _)| *c == first.context) {
            Some((_, members)) => members.push((index, first.target)),
            None => groups.push((first.context, vec![(index, first.target)])),
        }
        values.push(json!({
            "index": index,
            "context": first.context,
            "target": first.target,
            "step0_input_active_byte": byte_from_bits(&first.input, BIT_ACTIVE_BYTE),
        }));
    }
    let conflicts: Vec<Value> = groups
        .iter()
        .filter(|(_, members)| {
            members
                .iter()
                .map(|(_, t)| *t)
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                > 1
        })
        .map(|(context, members)| {
            json!({
                "context": context,
                "size": members.len(),
                "members": members.iter().map(|(i, t)| json!([i, t])).collect::<Vec<_>>(),
                "distinct_targets": members.iter().map(|(_, t)| *t).collect::<std::collections::BTreeSet<_>>().len(),
            })
        })
        .collect();
    let conflicting_examples: usize = conflicts
        .iter()
        .map(|c| c["size"].as_u64().unwrap_or(0) as usize)
        .sum();
    Ok(json!({
        "examples": indices.len(),
        "distinct_step0_contexts": groups.len(),
        "conflicting_target_contexts": conflicts.len(),
        "examples_in_conflicting_contexts": conflicting_examples,
        "groups": groups
            .iter()
            .map(|(c, members)| json!({
                "context": c,
                "size": members.len(),
                "targets": members.iter().map(|(_, t)| *t).collect::<Vec<_>>(),
            }))
            .collect::<Vec<_>>(),
        "conflicts": conflicts,
        "rows": values,
    }))
}

// ------------------------------------------------------------- S2 sweeps

fn set_byte_bits(input: &[bool; 1024], offset: usize, value: u8) -> [bool; 1024] {
    let mut out = *input;
    for i in 0..8 {
        out[offset + i] = (value >> i) & 1 != 0;
    }
    out
}
fn byte_from_bits(input: &[bool; 1024], offset: usize) -> u8 {
    (0..8).fold(0u8, |acc, i| acc | (u8::from(input[offset + i]) << i))
}
/// Distinct contexts over the 256 values of one byte channel, holding every other
/// bit fixed. Same method as the sealed emission-reach bin.
fn sweep_channel(
    compiled: &PrimitiveExport,
    input: &[bool; 1024],
    offset: usize,
) -> AnyResult<(usize, Vec<[u64; 2]>)> {
    let mut seen: std::collections::BTreeMap<u16, u8> = std::collections::BTreeMap::new();
    for value in 0..=255u8 {
        let context = compiled
            .circuit
            .evaluate(&set_byte_bits(input, offset, value))?;
        seen.entry(context).or_insert(value);
    }
    let mut sample: Vec<[u64; 2]> = seen
        .iter()
        .map(|(&context, &value)| [u64::from(context), u64::from(value)])
        .collect();
    sample.sort_unstable();
    Ok((seen.len(), sample))
}

/// Capture the single arithmetic Emit consult of one space-shaped reach example,
/// exactly as the sealed emission-reach bin drives it.
fn capture_reach(
    g: &BoundGeometry,
    params: &Parameters,
    compiled: &PrimitiveExport,
    index: usize,
    a: i64,
    b: i64,
) -> AnyResult<([bool; 1024], u16)> {
    let epoch = index as u64 + 1;
    let prompt = format!("{a} {b}\n").into_bytes();
    let spans: [(u64, u8); 2] = [
        (0, a.to_string().len() as u8),
        (a.to_string().len() as u64 + 1, b.to_string().len() as u8),
    ];
    let mut runtime = RuntimeSession::new(params.digest(), g, epoch)?;
    let mut policy = Capture::new(
        Controlled::new(compiled, Control::Full, g.identity()),
        Arm::ForcedReadOp,
        OP_ADD,
    );
    for &byte in &prompt {
        let offer = runtime.predict(g, &mut policy)?;
        runtime.observe(Symbol::Byte(byte), offer.offer.id, g, &mut policy)?;
    }
    let plan = compute_plan(g, &runtime, epoch, spans)?;
    policy.plan = plan;
    policy.force = true;
    policy.current_target = 0;
    let before = policy.consults.len();
    let _ = runtime.predict(g, &mut policy)?;
    let consult = policy.consults[before..]
        .last()
        .ok_or("no emission consult on reach answer step")?;
    Ok((consult.input, consult.context))
}

/// S2 readout over the nine reach examples. `captured` holds the Emit inputs the
/// engine produced under whatever parameters were used to capture them, so the same
/// method can isolate the circuit change (frozen-parent inputs) or measure it
/// end-to-end (inputs re-captured with the fitted parameters).
fn reach_sweeps(
    g: &BoundGeometry,
    params: &Parameters,
    compiled: &PrimitiveExport,
    captured: &[([bool; 1024], u16)],
) -> AnyResult<Value> {
    let mut distinct = Vec::new();
    let mut rows = Vec::new();
    for (i, &(a, b)) in REACH.iter().enumerate() {
        let (input, context) = captured[i];
        let (count, sample) = sweep_channel(compiled, &input, BIT_ACTIVE_BYTE)?;
        distinct.push(count);
        rows.push(json!({
            "index": i,
            "a": a,
            "b": b,
            "captured_context": context,
            "active_byte_distinct_contexts": count,
            "active_byte_sample": sample,
        }));
    }
    let mean = distinct.iter().sum::<usize>() as f64 / distinct.len() as f64;
    let constant = distinct.iter().filter(|&&c| c == 1).count();
    let _ = (g, params);
    Ok(json!({
        "examples": REACH.len(),
        "distinct_contexts": distinct,
        "mean_distinct_contexts": mean,
        "constant_context_examples": constant,
        "max_distinct_contexts": distinct.iter().copied().max().unwrap_or(0),
        "rows": rows,
    }))
}

fn capture_reach_all(
    g: &BoundGeometry,
    params: &Parameters,
    compiled: &PrimitiveExport,
) -> AnyResult<Vec<([bool; 1024], u16)>> {
    let mut out = Vec::with_capacity(REACH.len());
    for (i, &(a, b)) in REACH.iter().enumerate() {
        out.push(capture_reach(g, params, compiled, i, a, b)?);
    }
    Ok(out)
}

// -------------------------------------------------------------- diagnostics

/// Cheap in-fit readout over the fit pairs plus the frozen-parent reach inputs.
fn fit_diagnostics(
    g: &BoundGeometry,
    params: &Parameters,
    compiled: &PrimitiveExport,
    pairs: &[(usize, StePair)],
    frozen_reach: &[([bool; 1024], u16)],
) -> AnyResult<Value> {
    let topology = ste::topology_for(params)?;
    let mut contexts = std::collections::BTreeSet::new();
    let mut correct = 0u64;
    for (_, pair) in pairs {
        let context = ste::hard_context(&topology, params, &pair.input)?;
        contexts.insert(context);
        if ste::emission_argmax(params, context)? == pair.target {
            correct += 1;
        }
    }
    let sweep = reach_sweeps(g, params, compiled, frozen_reach)?;
    Ok(json!({
        "fit_pairs": pairs.len(),
        "distinct_fit_contexts": contexts.len(),
        "fit_pair_argmax_correct": correct,
        "dictated_sweep_mean_distinct": sweep["mean_distinct_contexts"],
        "dictated_sweep_constant_examples": sweep["constant_context_examples"],
    }))
}

// ------------------------------------------------------------------- driver

fn prepare_base(base: &str) -> AnyResult<std::path::PathBuf> {
    let base = std::path::PathBuf::from(base);
    if !base.is_absolute() {
        return Err("report base must be an absolute path".into());
    }
    for ancestor in base.parent().into_iter().flat_map(Path::ancestors) {
        if ancestor.join("manifest.json").exists() {
            return Err(format!("{} is beneath a sealed root", base.display()).into());
        }
    }
    std::fs::create_dir_all(&base)?;
    Ok(base)
}

fn write_criteria(base: &Path) -> AnyResult<()> {
    let root = base.join("criteria");
    for ancestor in root.parent().into_iter().flat_map(Path::ancestors) {
        if ancestor.join("manifest.json").exists() {
            return Err(format!("{} is beneath a sealed root", root.display()).into());
        }
    }
    report_output::claim(&root)?;
    let frozen: Value = serde_json::from_str(CRITERIA)?;
    let report = json!({
        "schema": "uor-r4.addressed-attention-arith-ste-criteria-report/1",
        "status": "CRITERIA_SEALED_BEFORE_ANY_FIT",
        "criteria": frozen,
        "bin_source_blake3": hex::encode(blake3::hash(include_bytes!("addressed-attention-arith-ste.rs").as_slice()).as_bytes()),
    });
    std::fs::write(
        root.join("report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    report_output::seal(&root)?;
    report_output::verify(&root)?;
    println!("[aa-ste] criteria sealed root={}", root.display());
    Ok(())
}

fn main() -> AnyResult<()> {
    let _ = rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build_global();
    let mode = std::env::args()
        .nth(1)
        .ok_or("usage: addressed-attention-arith-ste {criteria|run} <new-absolute-report-base>")?;
    let base = prepare_base(&std::env::args().nth(2).ok_or(
        "usage: addressed-attention-arith-ste {criteria|run} <new-absolute-report-base>",
    )?)?;
    let started = Instant::now();
    match mode.as_str() {
        "criteria" => write_criteria(&base)?,
        "run" => run(&base, started)?,
        other => return Err(format!("unknown mode {other}").into()),
    }
    println!(
        "[aa-ste] done mode={mode} elapsed_ms={} base={}",
        started.elapsed().as_millis(),
        base.display()
    );
    Ok(())
}

fn run(base: &Path, attempt_started: Instant) -> AnyResult<()> {
    let root = base.join("run");
    for ancestor in root.parent().into_iter().flat_map(Path::ancestors) {
        if ancestor.join("manifest.json").exists() {
            return Err(format!(
                "{} is beneath a sealed root; use a new base",
                root.display()
            )
            .into());
        }
    }
    report_output::claim(&root)?;
    let result = run_attempt(&root, base, attempt_started);
    match &result {
        Ok(()) => println!(
            "[aa-ste] status=COMPLETE elapsed_ms={}",
            attempt_started.elapsed().as_millis()
        ),
        Err(error) => {
            println!("[aa-ste] INCOMPLETE: {error}");
            let _ = std::fs::write(
                root.join("incomplete.json"),
                serde_json::to_vec_pretty(&json!({
                    "status": "INCOMPLETE_ADDRESSED_ATTENTION_ARITH_STE",
                    "error": error.to_string(),
                    "no_automatic_retry": true,
                }))?,
            );
        }
    }
    report_output::seal(&root)?;
    report_output::verify(&root)?;
    result
}

fn run_attempt(root: &Path, base: &Path, attempt_started: Instant) -> AnyResult<()> {
    // ---- criteria read-back before any model work
    let criteria_bytes = std::fs::read(base.join("criteria").join("report.json"))?;
    let criteria_file: Value = serde_json::from_slice(&criteria_bytes)?;
    let frozen: Value = serde_json::from_str(CRITERIA)?;
    if criteria_file["criteria"] != frozen {
        return Err("frozen criteria changed after they were written".into());
    }
    std::fs::write(
        root.join("criteria-digest.txt"),
        format!("{}\n", blake3::hash(&criteria_bytes).to_hex()),
    )?;

    let all_indices: Vec<usize> = (0..PROBLEMS.len()).collect();
    let fit_indices: Vec<usize> = all_indices.iter().copied().filter(|i| i % 2 == 0).collect();
    let holdout_indices: Vec<usize> = all_indices.iter().copied().filter(|i| i % 2 == 1).collect();

    let g = BoundGeometry::canonical()?;
    let initial = Parameters::seeded(PARAMETER_SEED)?;
    let initial_compiled = initial.compile()?;
    let initial_bytes = initial_compiled.encode();
    std::fs::write(root.join("initial-compiled.bin"), &initial_bytes)?;

    // ---- construction identity: the model wrapper is the same one the existing
    // bins use, so `Controlled` behaves identically here.
    let model = Model::new(
        initial_compiled.clone(),
        Provenance {
            seed: initial.seed(),
            training_data_digest: *blake3::hash(MODEL_DATA).as_bytes(),
            training_config_digest: *blake3::hash(CONFIG_DIGEST).as_bytes(),
            parameter_digest: initial.digest(),
            source_digest: implementation_digest(),
            parent: None,
        },
    )?;

    // ---- phase 0: capture. Pre-fit instruments plus the fit pairs.
    let pre_forced = forced_totals(&g, &initial, &initial_compiled, &all_indices)?;
    let pre_fit_forced = forced_totals(&g, &initial, &initial_compiled, &fit_indices)?;
    let pre_holdout_forced = forced_totals(&g, &initial, &initial_compiled, &holdout_indices)?;
    let pre_collisions = collision_structure(&g, &initial, &initial_compiled, &all_indices)?;
    let frozen_reach = capture_reach_all(&g, &initial, &initial_compiled)?;
    let pre_sweep = reach_sweeps(&g, &initial, &initial_compiled, &frozen_reach)?;

    let mut pairs: Vec<(usize, StePair)> = Vec::new();
    let mut eos_steps = 0u64;
    let mut digit_steps = 0u64;
    for &index in &all_indices {
        let trace = drive_example(&g, &initial, &initial_compiled, index, Arm::ForcedReadOp)?;
        for step in &trace.steps {
            if step.target == SYMBOL_EOS {
                eos_steps += 1;
                continue;
            }
            digit_steps += 1;
            if fit_indices.contains(&index) {
                pairs.push((
                    index,
                    StePair {
                        input: step.input,
                        target: step.target,
                    },
                ));
            }
        }
    }
    if pairs.is_empty() {
        return Err("no fit pairs captured".into());
    }
    let fit_pairs: Vec<StePair> = pairs.iter().map(|(_, p)| *p).collect();

    // ---- phase 1: the single pre-registered relaxed fit
    let config = SteConfig {
        updates: 6000,
        rate: 0.05,
        temperature_start: 1.0,
        temperature_end: 0.1,
        gate_rms: SCORE_FUNCTION_GATE_RMS,
    };
    let mut diagnostics: Vec<Value> = Vec::new();
    let mut trace_head: Vec<Value> = Vec::new();
    let fit_started = Instant::now();
    let (fitted, updates_done, stopped_early, stop_reason) = {
        let mut observer = |current: &Parameters, record: &ste::SteUpdate| -> bool {
            if record.update == 1
                || record.update % DIAGNOSTIC_EVERY == 0
                || record.update == config.updates
            {
                trace_head.push(json!({
                    "update": record.update,
                    "pair": record.pair,
                    "temperature": record.temperature,
                    "hard_context": record.hard_context,
                    "soft_loss": record.soft_loss,
                    "marginal": record.marginal,
                    "dominant": record.dominant,
                    "hard_ce": record.hard_ce,
                    "gate_gradient_rms": record.gate_gradient_rms,
                    "gate_gradient_raw_rms": record.gate_gradient_raw_rms,
                    "head_gradient_l2": record.head_gradient_l2,
                    "gradient_nonzero": record.gradient_nonzero,
                    "parameters_digest": hex::encode(record.parameters_digest),
                }));
                let diagnostic = match current.compile() {
                    Ok(compiled) => fit_diagnostics(&g, current, &compiled, &pairs, &frozen_reach)
                        .unwrap_or(Value::Null),
                    Err(_) => Value::Null,
                };
                diagnostics.push(json!({
                    "update": record.update,
                    "elapsed_ms": fit_started.elapsed().as_millis(),
                    "diagnostic": diagnostic,
                }));
                println!(
                    "[aa-ste] update {}/{} digest={} elapsed_ms={}",
                    record.update,
                    config.updates,
                    hex::encode(record.parameters_digest),
                    fit_started.elapsed().as_millis()
                );
            }
            fit_started.elapsed().as_millis() < FIT_CAP_MS
                && attempt_started.elapsed().as_millis() < ATTEMPT_CAP_MS
        };
        let outcome = ste::fit(&initial, &fit_pairs, &config, &mut observer)?;
        (
            outcome.parameters,
            outcome.updates_completed,
            outcome.stopped_early,
            outcome.stop_reason,
        )
    };
    let fit_us = fit_started.elapsed().as_micros();

    // ---- phase 2: post-fit evaluation with the unmodified engine
    let compiled = fitted.compile()?;
    let encoded = compiled.encode();
    std::fs::write(root.join("fitted-compiled.bin"), &encoded)?;
    if PrimitiveExport::decode(&encoded)? != compiled {
        return Err("fitted export reload".into());
    }
    let mut raw = Vec::with_capacity(fitted.values().len() * 8);
    for value in fitted.values() {
        raw.extend_from_slice(&value.to_bits().to_le_bytes());
    }
    std::fs::write(root.join("fitted-parameters.f64le"), &raw)?;

    let post_forced = forced_totals(&g, &fitted, &compiled, &all_indices)?;
    let post_fit_forced = forced_totals(&g, &fitted, &compiled, &fit_indices)?;
    let post_holdout_forced = forced_totals(&g, &fitted, &compiled, &holdout_indices)?;
    let post_collisions = collision_structure(&g, &fitted, &compiled, &all_indices)?;
    let post_reach = capture_reach_all(&g, &fitted, &compiled)?;
    let post_sweep_frozen_input = reach_sweeps(&g, &fitted, &compiled, &frozen_reach)?;
    let post_sweep_end_to_end = reach_sweeps(&g, &fitted, &compiled, &post_reach)?;

    // ---- S1..S5 decision
    let changed = initial_bytes
        .iter()
        .zip(&encoded)
        .filter(|(a, b)| a != b)
        .count();
    let deltas: Vec<f64> = fitted
        .values()
        .iter()
        .zip(initial.values())
        .map(|(a, b)| a - b)
        .collect();
    let delta_l2 = libm::sqrt(deltas.iter().map(|v| v * v).sum::<f64>());
    let gate_delta = libm::sqrt(deltas[..GATE_LOGITS].iter().map(|v| v * v).sum::<f64>());
    let (emission_offset, _) = head_range(Head::Emission, 0)?;
    let head_delta = libm::sqrt(deltas[emission_offset..].iter().map(|v| v * v).sum::<f64>());
    let other_delta = libm::sqrt(
        deltas[GATE_LOGITS..emission_offset]
            .iter()
            .map(|v| v * v)
            .sum::<f64>(),
    );

    let s1 = updates_done > 0 && changed >= 993 && delta_l2 > 0.0;
    let sweep_mean = post_sweep_frozen_input["mean_distinct_contexts"]
        .as_f64()
        .unwrap_or(f64::NAN);
    let sweep_constant = post_sweep_frozen_input["constant_context_examples"]
        .as_u64()
        .unwrap_or(u64::MAX);
    let sweep_max = post_sweep_frozen_input["max_distinct_contexts"]
        .as_u64()
        .unwrap_or(0);
    let sweep_over_4 = post_sweep_frozen_input["distinct_contexts"]
        .as_array()
        .map(|values| {
            values
                .iter()
                .filter(|v| v.as_u64().unwrap_or(0) > 4)
                .count()
        })
        .unwrap_or(0);
    let s2a = sweep_constant <= 1;
    let s2b = sweep_mean > 1.8888888888888888 && sweep_over_4 >= 2;
    let s2 = s2a && s2b;
    let post_contexts = post_collisions["distinct_step0_contexts"]
        .as_u64()
        .unwrap_or(u64::MAX);
    let post_conflicting = post_collisions["examples_in_conflicting_contexts"]
        .as_u64()
        .unwrap_or(u64::MAX);
    let s3 = post_contexts < 23 && post_conflicting < 13;
    let post_copy = post_forced["copy_steps"].as_u64().unwrap_or(0);
    let s4 = post_copy >= 1;
    let pre_arith = pre_forced["arithmetic_steps"].as_u64().unwrap_or(0);
    let pre_correct = pre_forced["correct_value_steps"].as_u64().unwrap_or(0);
    let post_arith = post_forced["arithmetic_steps"].as_u64().unwrap_or(0);
    let post_correct = post_forced["correct_value_steps"].as_u64().unwrap_or(0);
    let s5 = pre_arith > 0 && pre_correct == pre_arith && post_correct == post_arith;
    let status = if s1 && s2 && (s3 || s4) {
        "POSITIVE_ADDRESSED_ATTENTION_STE_ROUTING_LEARNED"
    } else {
        "NULL_ADDRESSED_ATTENTION_STE_ROUTING_NOT_LEARNED"
    };

    let compiled_bytes_len = encoded.len();
    let report = json!({
        "schema": "uor-r4.addressed-attention-arith-ste-run/1",
        "status": status,
        "question": "Does the opt-in relaxed/straight-through gate estimator move the discrete cascade enough to route the computed digit at bits 792..800 into the 9-bit emission context?",
        "construction": {
            "geometry_identity": g.identity(),
            "geometry_digest": hex::encode(g.identity_digest()),
            "model_bytes": model.encode().len(),
            "parameter_seed": PARAMETER_SEED,
            "parameter_count": fitted.values().len(),
            "initial_digest": hex::encode(initial.digest()),
            "fitted_digest": hex::encode(fitted.digest()),
            "compiled_bytes": encoded.len(),
            "changed_compiled_bytes": changed,
            "baseline_changed_compiled_bytes": 331,
            "parameter_delta_l2": delta_l2,
            "gate_block_delta_l2": gate_delta,
            "emission_block_delta_l2": head_delta,
            "other_blocks_delta_l2": other_delta,
            "finite": fitted.values().iter().all(|v| v.is_finite()),
            "fit_split_rule": "fit = even global indices, holdout = odd global indices, identical to the predecessor score-function fit",
            "fit_examples": fit_indices.len(),
            "holdout_examples": holdout_indices.len(),
            "digit_steps_captured": digit_steps,
            "eos_steps_excluded": eos_steps,
            "fit_pairs": fit_pairs.len(),
        },
        "fit": {
            "updates_requested": config.updates,
            "updates_completed": updates_done,
            "stopped_early": stopped_early,
            "stop_reason": stop_reason,
            "rate": config.rate,
            "gate_rms_normalisation": config.gate_rms,
            "temperature_start": config.temperature_start,
            "temperature_end": config.temperature_end,
            "trained_blocks": "gate logits [0, GATE_LOGITS) via the relaxed estimator; the 512 emission rows via hard-context cross-entropy; every other block frozen",
            "optimizer": "learner::sgd (crate), theta' = theta - rate*gradient",
            "fit_us": fit_us,
            "trace_head": trace_head,
            "diagnostics": diagnostics,
            "capture_note": "the fit consumes Emit inputs captured with the frozen parent parameters; the evaluation below drives the unmodified engine end-to-end with the fitted parameters",
        },
        "sweep": {
            "method": "same 256-value single-byte-channel sweep as the sealed emission-reach bin, byte field 792..800, all other 1023 bits fixed",
            "baseline": {
                "distinct_contexts": [2, 1, 2, 2, 1, 2, 4, 2, 1],
                "mean_distinct_contexts": 1.8888888888888888,
                "constant_context_examples": 3,
                "max_distinct_contexts": 4,
                "sealed_root": "aa-ste-before-emission-reach-20261006-013232"
            },
            "pre_fit": pre_sweep,
            "post_fit_frozen_parent_inputs": post_sweep_frozen_input,
            "post_fit_end_to_end_inputs": post_sweep_end_to_end,
        },
        "collisions": {
            "method": "step-0 context and target of the same 32-example forced-read+forced-operator trace the sealed emission-context bin reports",
            "baseline": {
                "distinct_step0_contexts": 23,
                "conflicting_target_contexts": 5,
                "examples_in_conflicting_contexts": 13,
                "sealed_root": "aa-ste-before-emission-context-trace-20261006-013235"
            },
            "pre_fit": pre_collisions,
            "post_fit": post_collisions,
        },
        "copy_and_operator": {
            "method": "forced-read+forced-operator arm over the arithmetic split; copy_steps counts emitted symbol == produced cursor byte on arithmetic steps; correct_value_steps counts produced bytes == exact canonical i64 value",
            "pre_fit": {
                "all": pre_forced,
                "fit_split": pre_fit_forced,
                "holdout_split": pre_holdout_forced,
            },
            "post_fit": {
                "all": post_forced,
                "fit_split": post_fit_forced,
                "holdout_split": post_holdout_forced,
            },
        },
        "decision": {
            "S1_fit_moves_the_circuit": s1,
            "S1_threshold_changed_bytes": 993,
            "S1_changed_compiled_bytes": changed,
            "S1_parameter_delta_l2": delta_l2,
            "S2_digit_reaches_context": s2,
            "S2a_constant_examples_at_most_1": s2a,
            "S2a_constant_examples": sweep_constant,
            "S2b_mean_above_baseline_and_two_examples_above_4": s2b,
            "S2b_mean_distinct": sweep_mean,
            "S2b_max_distinct": sweep_max,
            "S2b_examples_above_4": sweep_over_4,
            "S3_collision_structure": s3,
            "S3_distinct_step0_contexts": post_contexts,
            "S3_examples_in_conflicting_contexts": post_conflicting,
            "S4_copy_rate_leaves_zero": s4,
            "S4_copy_steps_all": post_copy,
            "S4_arithmetic_steps_all": post_arith,
            "S4_copy_steps_fit_split": post_fit_forced["copy_steps"],
            "S4_copy_steps_holdout_split": post_holdout_forced["copy_steps"],
            "S5_operator_stays_exact": s5,
            "S5_pre": format!("{pre_correct}/{pre_arith}"),
            "S5_post": format!("{post_correct}/{post_arith}"),
            "status": status,
            "promotion": false,
        },
        "cost": {
            "attempt_us": attempt_started.elapsed().as_micros(),
            "fit_us": fit_us,
            "fit_wall_clock_cap_ms": FIT_CAP_MS,
            "attempt_wall_clock_cap_ms": ATTEMPT_CAP_MS,
            "cpu": "local CPU only; no GPU, no pod, no external cost",
        },
        "honesty": [
            "The fit consumes Emit inputs captured with the frozen parent parameters; S2-S5 are then measured by driving the unmodified engine with the fitted parameters.",
            "The 32-example split is authored in this instrument by i64 arithmetic; this is a bounded mechanism probe, not an independent corpus or a language result.",
            "S1 is an activity threshold, not a quality threshold; S2-S4 carry the quality claim.",
            "The default hard cascade is unchanged: this run only calls the explicitly named ste training entry point.",
            "The nine S2 sweep examples are a different problem set from the 32-example fit split.",
        ],
        "criteria_digest_blake3": blake3::hash(&criteria_bytes).to_hex().to_string(),
        "bin_source_blake3": hex::encode(blake3::hash(include_bytes!("addressed-attention-arith-ste.rs").as_slice()).as_bytes()),
    });
    std::fs::write(
        root.join("report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!("[aa-ste] S1={s1} S2={s2} S3={s3} S4={s4} S5={s5} status={status}");
    println!(
        "[aa-ste] changed_bytes={changed}/{compiled_bytes_len} delta_l2={delta_l2:.6} copies={post_copy}/{post_arith} contexts={post_contexts} conflicts={post_conflicting}"
    );
    Ok(())
}
