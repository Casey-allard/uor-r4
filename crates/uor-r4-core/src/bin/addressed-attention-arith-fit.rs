//! Bounded fit probe: can the addressed-attention emission head learn to copy a
//! value the engine has already computed correctly?
//!
//! Predecessor (`addressed-attention-arith-oracle`, sealed root
//! `aa-arith-oracle-20261005-204407`): with unfitted `Parameters::seeded(7341)`
//! the read can be forced to the intended operand spans, the integer operator
//! then produces the exactly correct value 80/80, and the compiled 257-way
//! emission head emits that byte 0/80. That run could not separate "cannot" from
//! "untrained". This binary runs one bounded fit of the crate's own learner
//! (`learner::batch` four-particle serial LOO + `learner::sgd`, exactly the
//! recipe of the preserved 64-update pilot driver `pilot_tests.rs:114-236`) on a
//! new authored canonical-decimal split, then re-runs the same scored surface.
//!
//! Measurement-only: it adds no production code path, changes no default
//! behaviour and mutates no frozen parameter, geometry, pilot data or artifact.
//! Subcommands:
//!
//!   criteria <abs-base>   frozen acceptance criteria; written before any fit
//!   run      <abs-base>   the pre-registered fit and matched evaluation
use std::path::Path;
use std::time::Instant;

use serde_json::{json, Value};
use uor_r4_core::native_geometric::addressed_attention::artifact::BoundGeometry;
use uor_r4_core::native_geometric::addressed_attention::circuit::{PrimitiveExport, GATE_LOGITS};
use uor_r4_core::native_geometric::addressed_attention::engine::{
    self, Head, Policy, RuntimeSession, Work,
};
use uor_r4_core::native_geometric::addressed_attention::inputs::Phase;
use uor_r4_core::native_geometric::addressed_attention::learner;
use uor_r4_core::native_geometric::addressed_attention::objects::{self, Action, Symbol};
use uor_r4_core::native_geometric::addressed_attention::pilot::{self, Control, Controlled};
use uor_r4_core::native_geometric::addressed_attention::pilot_data::Example;
use uor_r4_core::native_geometric::addressed_attention::policy::Parameters;
use uor_r4_core::report_output;

type AnyResult<T> = Result<T, Box<dyn std::error::Error>>;

const PARAMETER_SEED: u64 = 7341;
const EVENT_SEED: u64 = 973;
/// One recipe, no sweep: 4x the historical pilot's update count, same 0.05 rate.
const UPDATES: u64 = 256;
const RATE: f64 = 0.05;
/// Historical scale, evaluated as a pre-stated milestone only.
const MILESTONE_UPDATE: u64 = 64;
/// Fit-loop wall-clock cap (update boundaries only; no partial update).
const FIT_CAP_MS: u128 = 300_000;
/// Whole-attempt cap charged across baseline, fit and evaluation.
const ATTEMPT_CAP_MS: u128 = 900_000;

/// Frozen before any engine call in this session. `criteria` mode writes it.
const CRITERIA: &str = r#"{
  "schema": "uor-r4.addressed-attention-arith-fit-criteria/1",
  "frozen": "written before any fit or engine evaluation in this session; no threshold below was adjusted after seeing a result",
  "question": "Can the addressed-attention emission head learn to emit a byte the engine has already computed correctly? The predecessor run (aa-arith-oracle-20261005-204407) established the antecedent with unfitted parameters: forced-read+forced-operator produced the exactly correct canonical value 80/80 and emitted it 0/80. This run fits parameters and asks the same question again.",
  "instrument": {
    "bin": "addressed-attention-arith-fit (new, measurement-only; no production change)",
    "split": "arith-canonical-decimal/1, the same 32 authored prompts and the same newline shape as the predecessor, re-authored here from the same PROBLEMS table; expected answers constructed by exact i64 arithmetic plus canonical decimal rendering in this instrument (NOT a frozen external fixture)",
    "split_rule": "train = even global indices (16), holdout = odd global indices (16), fixed before the fit; both splits contain add and sub families and one- and two-digit operands",
    "fit_path": "crate learner: learner::batch(4-particle serial LOO over the full document prompt+answer+EOS; learner.rs:29) then learner::sgd(rate 0.05; learner.rs:117), exactly the recipe of the preserved pilot driver pilot_tests.rs:114-236",
    "fit_recipe": "updates 256, document_id = update index, example = train[update % 16], event_seed 973, plain SGD, no clipping/momentum/decay, 4 particles, no held-out data in the fit",
    "evaluation": "pilot::evaluate (deterministic CompiledPolicy over the fitted Parameters::compile()) on train (16), holdout (16) and all 32, for Full and all three destructive controls; CE trajectory from learner::batch mean_ce per update",
    "forced_instrument": "harness-only Policy overrides copied from the predecessor: intended-span query/null by exhaustive 120x120 search, extent forced to the intended length, and (C2) the intended operator forced when legal. It forces no circuit context and no emission choice."
  },
  "primary_criteria": {
    "F1": "the fit runs to completion without error and reports update count and per-update CE trajectory",
    "F2": "on the 16 holdout examples, emitted-byte accuracy on the arithmetic answer (correct_symbols scored over answer+EOS positions) is strictly greater than the pre-fit baseline on the same holdout and non-zero",
    "F3": "conditional on F2: Full strictly beats all three destructive controls on the same holdout - for every control, Full mean_response_ce is strictly lower AND at least one example is exactly right (answer+EOS) under Full and not under that control",
    "F4": "the exactly-computed value is still correct after the fit: in the forced-read+forced-operator arm over all 32 examples, the produced active-lease bytes equal the exact canonical value on every arithmetic step, pre-fit and post-fit",
    "F5_decision": "POSITIVE iff F1 and F2 and F3 and F4; otherwise NULL with its numbers"
  },
  "secondary_measurements": {
    "S1_copy": "forced-read+forced-operator arm over all 32 examples: emitted symbol equals the produced cursor byte on arithmetic steps, pre-fit vs post-fit (predecessor: 0/80)",
    "S2_milestone": "CE and holdout accuracy at update 64 (the historical pilot scale) as well as at the pre-stated end of the fit",
    "S3_selection_drift": "natural Full arm instrumented counters over all 32 examples: control calls, arithmetic offered, chosen arithmetic, selected_operations, intended operand pair selected, pre-fit vs post-fit",
    "S4_parameters": "parameter delta L2, changed compiled bytes, initial/final digests, per-update gradient split (gate vs head) L2"
  },
  "budget": {
    "updates": 256,
    "fit_wall_clock_cap_ms": 300000,
    "attempt_wall_clock_cap_ms": 900000,
    "cpu": "local CPU only, no GPU, no pod",
    "storage": "new exclusively claimed report root under ~/uor-r4-worktrees/reports/"
  },
  "branch_rule": {
    "POSITIVE": "F1 and F2 and F3 and F4 - the mechanism computes and can be trained to say it",
    "NULL": "otherwise, recorded with numbers; a CE drop with zero or control-tying accuracy is recorded as the fit-to-noise signature",
    "STOP": "if learner::batch rejects the authored documents, or the fit path cannot be driven without changing production code, stop and report the exact blocker"
  },
  "honesty": [
    "The split's expected answers are CONSTRUCTED here by i64 arithmetic; that is a weaker instrument than a frozen external fixture.",
    "Fit and holdout both come from the same 32 authored prompts; this is a bounded probe, not an independent corpus.",
    "No production source file is modified by this probe."
  ]
}"#;

// ------------------------------------------------------------------- split

/// (a, b, op) with op 0 = Add, 1 = Sub. Copied in order from the predecessor bin.
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
const OP_ADD: usize = 3;
const OP_SUB: usize = 4;

fn scalar_of(index: usize) -> i64 {
    let (a, b, op) = PROBLEMS[index];
    if op == 0 {
        a + b
    } else {
        a - b
    }
}
fn example(index: usize) -> Example {
    let (a, b, op) = PROBLEMS[index];
    Example {
        id: format!("arith/{}/{index}", if op == 0 { "add" } else { "sub" }),
        family: if op == 0 {
            "arith_canonical_decimal_add".to_string()
        } else {
            "arith_canonical_decimal_sub".to_string()
        },
        // The predecessor's diagnosed shape: "<a>\n<b>\n".
        prompt: format!("{a}\n{b}\n").into_bytes(),
        answer: scalar_of(index).to_string().into_bytes(),
        expected_scalar: Some(scalar_of(index)),
    }
}
fn train_indices() -> Vec<usize> {
    (0..PROBLEMS.len()).filter(|i| i % 2 == 0).collect()
}
fn holdout_indices() -> Vec<usize> {
    (0..PROBLEMS.len()).filter(|i| i % 2 == 1).collect()
}
fn examples_at(indices: &[usize]) -> Vec<Example> {
    indices.iter().map(|&i| example(i)).collect()
}
/// Intended operand spans in the newline prompt, from the global index.
fn intended(index: usize) -> [(u64, u8); 2] {
    let (a, b, _) = PROBLEMS[index];
    let la = a.to_string().len() as u64;
    let lb = b.to_string().len() as u8;
    [(0, la as u8), (la + 1, lb)]
}
fn op_of(index: usize) -> usize {
    if PROBLEMS[index].2 == 0 {
        OP_ADD
    } else {
        OP_SUB
    }
}
fn exact_value(action: Action, a: i64, b: i64) -> Option<i64> {
    match action {
        Action::AddAB => a.checked_add(b),
        Action::SubAB => a.checked_sub(b),
        _ => None,
    }
}
fn canonical(value: i64) -> Vec<u8> {
    let (bytes, len) = objects::encode_i64(value);
    bytes[..usize::from(len)].to_vec()
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
    /// Whether the engine's own selection loop would have preferred the intended
    /// occurrence for that read (diagnostic only; the harness forces the plan).
    intended_wins: [bool; 2],
    extent_forcible: [bool; 2],
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
    add_offered: u64,
    sub_offered: u64,
    arith_offered_steps: u64,
    chosen_add: u64,
    chosen_sub: u64,
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
    }
    fn merge(&mut self, other: &Stats) {
        self.control_calls += other.control_calls;
        self.add_offered += other.add_offered;
        self.sub_offered += other.sub_offered;
        self.arith_offered_steps += other.arith_offered_steps;
        self.chosen_add += other.chosen_add;
        self.chosen_sub += other.chosen_sub;
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
        })
    }
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
        if matches!(head, Head::Control) {
            self.stats.record(legal, choice);
        }
        Ok(choice)
    }
}

/// Best harness-side query/null plan for the intended operand spans, copied from
/// the predecessor instrument. Geometry-only; independent of the parameters.
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
        plan.intended_wins[which] = matches!(winner, Winner::Occurrence(s) if s == start);
        plan.extent_forcible[which] = maximum >= len;
    }
    Ok(plan)
}

/// Drive one example with teacher forcing, optionally forcing the intended read
/// and operator. Returns a per-arm summary plus optional per-step rows.
fn drive_arm(
    g: &BoundGeometry,
    params: &Parameters,
    compiled: &PrimitiveExport,
    index: usize,
    arm: Arm,
    record: bool,
) -> AnyResult<(Value, Vec<Value>)> {
    let epoch = index as u64 + 1;
    let example = example(index);
    let spans = intended(index);
    let mut runtime = RuntimeSession::new(params.digest(), g, epoch)?;
    let mut policy = Oracle::new(
        Controlled::new(compiled, Control::Full, g.identity()),
        arm,
        op_of(index),
    );
    for &byte in &example.prompt {
        let offer = runtime.predict(g, &mut policy)?;
        runtime.observe(Symbol::Byte(byte), offer.offer.id, g, &mut policy)?;
    }
    let mut steps = Vec::new();
    let mut correct_symbols = 0usize;
    let mut positions = 0usize;
    let mut arithmetic_steps = 0u64;
    let mut correct_value_steps = 0u64;
    let mut wrong_value_steps = 0u64;
    let mut copy_steps = 0u64;
    let mut intended_pair_steps = 0u64;
    let mut intended_wins_reads = 0u64;
    let mut extent_forcible_reads = 0u64;
    for target in example.answer.iter().map(|&b| u16::from(b)).chain([256]) {
        if arm != Arm::Production {
            let plan = compute_plan(g, &runtime, epoch, spans)?;
            intended_wins_reads += plan.intended_wins.iter().filter(|b| **b).count() as u64;
            extent_forcible_reads += plan.extent_forcible.iter().filter(|b| **b).count() as u64;
            policy.plan = plan;
            policy.force = true;
        }
        let offered = runtime.predict(g, &mut policy)?;
        let emitted = symbol_code(offered.offer.symbol);
        positions += 1;
        correct_symbols += usize::from(emitted == target);
        let operands = offered.offer.operands();
        let payloads: Vec<Option<String>> = operands
            .iter()
            .map(|o| o.as_ref().map(payload_string))
            .collect();
        let values: [Option<i64>; 2] =
            operands.map(|o| o.and_then(|l| objects::decode_i64(l.payload()).ok()));
        let action = offered.offer.action;
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
                }
            }
        }
        let intended_pair = payloads[0].as_deref()
            == Some(span_string(&example.prompt, spans[0]).as_str())
            && payloads[1].as_deref() == Some(span_string(&example.prompt, spans[1]).as_str());
        intended_pair_steps += u64::from(intended_pair);
        if record {
            steps.push(json!({
                "index": index,
                "target": target,
                "emitted": emitted,
                "match": emitted == target,
                "action": action_name(action),
                "payloads": payloads,
                "produced_cursor_byte": active_byte,
                "emitted_is_produced_cursor_byte": active_byte.map(|b| emitted == u16::from(b)),
                "value_correct": value_correct,
                "intended_pair_selected": intended_pair,
            }));
        }
        let actual = symbol_of(target)?;
        runtime.observe(actual, offered.offer.id, g, &mut policy)?;
        if emitted == 256 {
            break;
        }
    }
    let mut stats = Stats::default();
    stats.merge(&policy.stats);
    Ok((
        json!({
            "index": index,
            "id": example.id,
            "positions": positions,
            "correct_symbols": correct_symbols,
            "arithmetic_steps": arithmetic_steps,
            "correct_value_steps": correct_value_steps,
            "wrong_value_steps": wrong_value_steps,
            "copy_steps": copy_steps,
            "intended_pair_steps": intended_pair_steps,
            "intended_wins_reads": intended_wins_reads,
            "extent_forcible_reads": extent_forcible_reads,
            "control_stats": stats.json(),
            "work": work_json(runtime.work()),
        }),
        steps,
    ))
}

fn drive_all(
    g: &BoundGeometry,
    params: &Parameters,
    compiled: &PrimitiveExport,
    indices: &[usize],
    arm: Arm,
    record: bool,
) -> AnyResult<(Value, Vec<Value>)> {
    let mut totals = json!({
        "examples": 0, "positions": 0, "correct_symbols": 0, "arithmetic_steps": 0,
        "correct_value_steps": 0, "wrong_value_steps": 0, "copy_steps": 0, "intended_pair_steps": 0,
        "intended_wins_reads": 0, "extent_forcible_reads": 0,
    });
    let mut stats = Stats::default();
    let mut work = json!({
        "circuit_calls": 0u64, "candidate_scores": 0u64, "predictions": 0u64,
        "observations": 0u64, "scalar_decodes": 0u64, "selected_operations": 0u64,
    });
    let mut steps = Vec::new();
    let mut rows = Vec::new();
    for &index in indices {
        let (row, mut arm_steps) = drive_arm(g, params, compiled, index, arm, record)?;
        let control = &row["control_stats"];
        let local = Stats {
            control_calls: control["control_calls"].as_u64().unwrap_or(0),
            add_offered: control["add_offered"].as_u64().unwrap_or(0),
            sub_offered: control["sub_offered"].as_u64().unwrap_or(0),
            arith_offered_steps: control["arithmetic_offered_steps"].as_u64().unwrap_or(0),
            chosen_add: control["chosen_add"].as_u64().unwrap_or(0),
            chosen_sub: control["chosen_sub"].as_u64().unwrap_or(0),
        };
        stats.merge(&local);
        for key in [
            "examples",
            "positions",
            "correct_symbols",
            "arithmetic_steps",
            "correct_value_steps",
            "wrong_value_steps",
            "copy_steps",
            "intended_pair_steps",
            "intended_wins_reads",
            "extent_forcible_reads",
        ] {
            let base = totals[key].as_u64().unwrap_or(0);
            totals[key] = json!(base + row[key].as_u64().unwrap_or(0));
        }
        for key in [
            "circuit_calls",
            "candidate_scores",
            "predictions",
            "observations",
            "scalar_decodes",
            "selected_operations",
        ] {
            let base = work[key].as_u64().unwrap_or(0);
            work[key] = json!(base + row["work"][key].as_u64().unwrap_or(0));
        }
        rows.push(row);
        steps.append(&mut arm_steps);
    }
    Ok((
        json!({ "totals": totals, "control_stats": stats.json(), "work": work, "rows": rows }),
        steps,
    ))
}

// ------------------------------------------------------------------ driver

fn evaluate_sets(
    params: &Parameters,
    compiled: &PrimitiveExport,
    g: &BoundGeometry,
    train: &[Example],
    holdout: &[Example],
    all: &[Example],
) -> AnyResult<Value> {
    let mut out = serde_json::Map::new();
    for (set_name, examples) in [("train", train), ("holdout", holdout), ("all", all)] {
        let mut arms = serde_json::Map::new();
        for control in pilot::CONTROLS {
            let rows = pilot::evaluate(params, compiled, g, examples, control)?;
            let metrics = pilot::metrics(&rows);
            arms.insert(
                format!("{control:?}"),
                json!({ "metrics": metrics, "rows": rows }),
            );
        }
        out.insert(set_name.to_string(), Value::Object(arms));
    }
    Ok(Value::Object(out))
}

fn arm_metrics<'a>(evaluated: &'a Value, set: &str, control: &str) -> &'a Value {
    &evaluated[set][control]["metrics"]
}

fn main() -> AnyResult<()> {
    let mode = std::env::args().nth(1).ok_or(
        "usage: addressed-attention-arith-fit {criteria|run} <new-absolute-report-base>",
    )?;
    let base = std::env::args()
        .nth(2)
        .ok_or("usage: addressed-attention-arith-fit {criteria|run} <new-absolute-report-base>")?;
    let base = std::path::PathBuf::from(base);
    if !base.is_absolute() {
        return Err("report base must be an absolute path".into());
    }
    match mode.as_str() {
        "criteria" => write_criteria(&base),
        "run" => run(&base),
        other => Err(format!("unknown subcommand {other}").into()),
    }
}

fn write_criteria(base: &Path) -> AnyResult<()> {
    let root = base.join("criteria");
    for ancestor in root.parent().into_iter().flat_map(Path::ancestors) {
        if ancestor.join("manifest.json").exists() {
            return Err(format!("{} is beneath a sealed root", root.display()).into());
        }
    }
    report_output::claim(&root)?;
    let criteria: Value = serde_json::from_str(CRITERIA)?;
    let report = json!({
        "schema": "uor-r4.addressed-attention-arith-fit-criteria-report/1",
        "written_unix_s": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
        "bin_source_blake3": hex::encode(blake3::hash(include_bytes!("addressed-attention-arith-fit.rs").as_slice()).as_bytes()),
        "criteria": criteria,
    });
    let bytes = serde_json::to_vec_pretty(&report)?;
    std::fs::write(root.join("report.json"), &bytes)?;
    println!(
        "criteria written root={} bytes={} blake3={}",
        root.display(),
        bytes.len(),
        blake3::hash(&bytes).to_hex()
    );
    report_output::seal(&root)?;
    report_output::verify(&root)?;
    Ok(())
}

fn run(base: &Path) -> AnyResult<()> {
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
    let attempt_started = Instant::now();
    let result = run_attempt(&root, base, attempt_started);
    match &result {
        Ok(()) => println!(
            "[aa-arith-fit] status=COMPLETE elapsed_ms={}",
            attempt_started.elapsed().as_millis()
        ),
        Err(error) => {
            println!("[aa-arith-fit] INCOMPLETE: {error}");
            let _ = std::fs::write(
                root.join("incomplete.json"),
                serde_json::to_vec_pretty(&json!({
                    "status": "INCOMPLETE_ADDRESSED_ATTENTION_ARITH_FIT",
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
    // ---- criteria: read back and verify the frozen payload before any model work
    let criteria_bytes = std::fs::read(base.join("criteria").join("report.json"))?;
    let criteria_file: Value = serde_json::from_slice(&criteria_bytes)?;
    let frozen: Value = serde_json::from_str(CRITERIA)?;
    if criteria_file["criteria"] != frozen {
        return Err("frozen criteria changed after they were written".into());
    }
    let criteria_digest = blake3::hash(&criteria_bytes).to_hex().to_string();
    std::fs::write(root.join("criteria-digest.txt"), format!("{criteria_digest}\n"))?;

    let train_indices = train_indices();
    let holdout_indices = holdout_indices();
    let all_indices: Vec<usize> = (0..PROBLEMS.len()).collect();
    let train = examples_at(&train_indices);
    let holdout = examples_at(&holdout_indices);
    let all = examples_at(&all_indices);
    for (name, set) in [("train", &train), ("holdout", &holdout)] {
        std::fs::write(
            root.join(format!("{name}.json")),
            serde_json::to_vec_pretty(set)?,
        )?;
    }
    let train_digest = blake3::hash(&std::fs::read(root.join("train.json"))?)
        .to_hex()
        .to_string();

    // ---- construction (unchanged engine path)
    let g = BoundGeometry::canonical()?;
    let initial = Parameters::seeded(PARAMETER_SEED)?;
    let initial_digest = hex::encode(initial.digest());
    let initial_compiled = initial.compile()?;
    let initial_compiled_bytes = initial_compiled.encode();
    std::fs::write(root.join("initial-compiled.bin"), &initial_compiled_bytes)?;

    // ---- pre-fit scored surface (reproduces the sealed predecessor numbers)
    let t0 = Instant::now();
    let pre = evaluate_sets(&initial, &initial_compiled, &g, &train, &holdout, &all)?;
    let pre_us = t0.elapsed().as_micros();
    let t0 = Instant::now();
    let (pre_forced, pre_forced_steps) =
        drive_all(&g, &initial, &initial_compiled, &all_indices, Arm::ForcedReadOp, true)?;
    let pre_forced_us = t0.elapsed().as_micros();
    let t0 = Instant::now();
    let (pre_natural, pre_natural_steps) =
        drive_all(&g, &initial, &initial_compiled, &all_indices, Arm::Production, false)?;
    let pre_natural_us = t0.elapsed().as_micros();

    // ---- the one pre-registered fit: plain SGD on the crate's own learner
    let mut params = initial.clone();
    let mut steps: Vec<Value> = Vec::new();
    let mut updates = 0u64;
    let mut loop_error: Option<String> = None;
    let mut stopped_at_cap = false;
    let fit_timer = Instant::now();
    for update in 0..UPDATES {
        if fit_timer.elapsed().as_millis() >= FIT_CAP_MS || attempt_started.elapsed().as_millis() >= ATTEMPT_CAP_MS
        {
            stopped_at_cap = true;
            break;
        }
        let index = train_indices[update as usize % train_indices.len()];
        let example = &train[update as usize % train.len()];
        let batch = match learner::batch(
            &params,
            &g,
            &pilot::document(example),
            EVENT_SEED,
            update,
        ) {
            Ok(v) => v,
            Err(e) => {
                loop_error = Some(e.to_string());
                break;
            }
        };
        let mut norms = [0.0f64; 2];
        for (i, &v) in batch.gradient.iter().enumerate() {
            norms[usize::from(i >= GATE_LOGITS)] += v * v;
        }
        let next = match learner::sgd(&params, &batch.gradient, RATE) {
            Ok(v) => v,
            Err(e) => {
                loop_error = Some(e.to_string());
                break;
            }
        };
        steps.push(json!({
            "update": update + 1,
            "example_index": index,
            "example_id": example.id,
            "mean_ce": batch.mean_ce,
            "batch_us": batch.elapsed_us,
            "counts": batch.counts,
            "gate_gradient_l2": libm::sqrt(norms[0]),
            "head_gradient_l2": libm::sqrt(norms[1]),
            "gradient_nonzero": batch.gradient.iter().filter(|g| **g != 0.0).count(),
            "before": hex::encode(params.digest()),
            "after": hex::encode(next.digest()),
        }));
        params = next;
        updates += 1;
    }
    let fit_us = fit_timer.elapsed().as_micros();
    let ce_first = steps
        .first()
        .and_then(|s| s["mean_ce"].as_f64())
        .unwrap_or(f64::NAN);
    let ce_last = steps
        .last()
        .and_then(|s| s["mean_ce"].as_f64())
        .unwrap_or(f64::NAN);
    let ce_at_milestone = steps
        .get(MILESTONE_UPDATE as usize - 1)
        .and_then(|s| s["mean_ce"].as_f64());
    if let Some(error) = &loop_error {
        std::fs::write(
            root.join("fit-error.json"),
            serde_json::to_vec_pretty(&json!({"error": error, "updates": updates}))?,
        )?;
        return Err(format!("learner loop error after {updates} updates: {error}").into());
    }

    // ---- retained fitted payload: engine artifact encoding + raw parameter bits
    let compiled = params.compile()?;
    let encoded = compiled.encode();
    std::fs::write(root.join("final-compiled.bin"), &encoded)?;
    if PrimitiveExport::decode(&std::fs::read(root.join("final-compiled.bin"))?)? != compiled {
        return Err("final actual export reload".into());
    }
    let mut raw = Vec::with_capacity(params.values().len() * 8);
    for v in params.values() {
        raw.extend_from_slice(&v.to_bits().to_le_bytes());
    }
    std::fs::write(root.join("final-parameters.f64le"), &raw)?;
    let deltas: Vec<f64> = params
        .values()
        .iter()
        .zip(initial.values())
        .map(|(a, b)| a - b)
        .collect();
    let delta_l2 = libm::sqrt(deltas.iter().map(|v| v * v).sum::<f64>());
    let changed_compiled_bytes = initial_compiled_bytes
        .iter()
        .zip(&encoded)
        .filter(|(a, b)| a != b)
        .count();

    // ---- post-fit scored surface and instruments
    let t0 = Instant::now();
    let post = evaluate_sets(&params, &compiled, &g, &train, &holdout, &all)?;
    let post_us = t0.elapsed().as_micros();
    let t0 = Instant::now();
    let (post_forced, post_forced_steps) =
        drive_all(&g, &params, &compiled, &all_indices, Arm::ForcedReadOp, true)?;
    let post_forced_us = t0.elapsed().as_micros();
    let t0 = Instant::now();
    let (post_natural, post_natural_steps) =
        drive_all(&g, &params, &compiled, &all_indices, Arm::Production, false)?;
    let post_natural_us = t0.elapsed().as_micros();

    // ---- pre-registered decision
    let h0 = arm_metrics(&pre, "holdout", "Full")["correct_symbols"]
        .as_u64()
        .unwrap_or(0);
    let h1 = arm_metrics(&post, "holdout", "Full")["correct_symbols"]
        .as_u64()
        .unwrap_or(0);
    let a0 = arm_metrics(&pre, "all", "Full")["correct_symbols"]
        .as_u64()
        .unwrap_or(0);
    let a1 = arm_metrics(&post, "all", "Full")["correct_symbols"]
        .as_u64()
        .unwrap_or(0);
    let f1 = updates > 0 && loop_error.is_none();
    let f2 = h1 > h0 && h1 > 0;
    let full_ce = arm_metrics(&post, "holdout", "Full")["mean_response_ce"]
        .as_f64()
        .unwrap_or(f64::NAN);
    let mut weaker = Vec::new();
    for control in ["ReadDisabled", "ExactPayloadMasked", "StateTransportDisabled"] {
        let ce = arm_metrics(&post, "holdout", control)["mean_response_ce"]
            .as_f64()
            .unwrap_or(f64::NAN);
        let exact_full = arm_metrics(&post, "holdout", "Full")["exact_responses"]
            .as_u64()
            .unwrap_or(0);
        let exact_control = arm_metrics(&post, "holdout", control)["exact_responses"]
            .as_u64()
            .unwrap_or(0);
        weaker.push(json!({
            "control": control,
            "control_mean_response_ce": ce,
            "full_mean_response_ce": full_ce,
            "strictly_lower_ce": ce > full_ce,
            "full_exact": exact_full,
            "control_exact": exact_control,
            "uniquely_exact_under_full": exact_full > exact_control,
            "beaten": ce > full_ce && exact_full > exact_control,
        }));
    }
    let f3 = f2 && weaker.iter().all(|w| w["beaten"] == json!(true));
    let f4_pre = pre_forced["totals"]["arithmetic_steps"].as_u64().unwrap_or(0) > 0
        && pre_forced["totals"]["correct_value_steps"] == pre_forced["totals"]["arithmetic_steps"];
    let f4_post = post_forced["totals"]["arithmetic_steps"].as_u64().unwrap_or(0) > 0
        && post_forced["totals"]["correct_value_steps"]
            == post_forced["totals"]["arithmetic_steps"];
    let f4 = f4_pre && f4_post;
    let status = if f1 && f2 && f3 && f4 {
        "POSITIVE_ADDRESSED_ATTENTION_EMISSION_COPY_LEARNED"
    } else {
        "NULL_ADDRESSED_ATTENTION_EMISSION_COPY_NOT_LEARNED"
    };

    let report = json!({
        "schema": "uor-r4.addressed-attention-arith-fit-run/1",
        "status": status,
        "question": "Can a fitted emission head emit a byte the engine has computed correctly?",
        "construction": {
            "geometry_identity": g.identity(),
            "geometry_digest": hex::encode(g.identity_digest()),
            "parameter_seed": PARAMETER_SEED,
            "parameter_count": params.values().len(),
            "initial_digest": initial_digest,
            "final_digest": hex::encode(params.digest()),
            "initial_compiled_bytes": initial_compiled_bytes.len(),
            "final_compiled_bytes": encoded.len(),
            "changed_compiled_bytes": changed_compiled_bytes,
            "parameter_delta_l2": delta_l2,
            "finite": params.values().iter().all(|v| v.is_finite()),
            "split_rule": "train = even global indices, holdout = odd global indices, fixed before the fit",
            "train_examples": train.len(),
            "holdout_examples": holdout.len(),
            "train_digest": train_digest,
        },
        "fit": {
            "updates": updates,
            "requested_updates": UPDATES,
            "stopped_at_cap": stopped_at_cap,
            "fit_us": fit_us,
            "rate": RATE,
            "particles": learner::PARTICLES,
            "event_seed": EVENT_SEED,
            "recipe": "learner::batch(serial 4-particle LOO full-document CE) then learner::sgd(0.05); mirror of pilot_tests.rs:114-236",
            "checkpoint_note": "learner::Checkpoint is not used: its codec bound is 64 updates (learner.rs:12,175), below this pre-stated 256; the fitted payload is retained as the engine's own PrimitiveExport encoding plus raw f64le parameters",
            "ce_first_update": ce_first,
            "ce_last_update": ce_last,
            "ce_at_update_64": ce_at_milestone,
            "ce_delta": ce_last - ce_first,
            "steps": steps,
        },
        "evaluation": {
            "pre_fit": pre,
            "post_fit": post,
            "pre_us": pre_us,
            "post_us": post_us,
            "baseline_note": "pre_fit/all/Full reproduces the sealed predecessor number 0/80 on the same 32-example split and the same deterministic CompiledPolicy",
        },
        "forced_instrument": {
            "pre_fit": pre_forced,
            "post_fit": post_forced,
            "pre_us": pre_forced_us,
            "post_us": post_forced_us,
            "pre_steps": pre_forced_steps,
            "post_steps": post_forced_steps,
            "description": "query/null chosen by exhaustive 120x120 search against the intended span keys, extent forced to the intended length, intended operator forced when legal; circuit context and emission choice untouched",
        },
        "natural_instrument": {
            "pre_fit": pre_natural,
            "post_fit": post_natural,
            "pre_us": pre_natural_us,
            "post_us": post_natural_us,
            "pre_steps": pre_natural_steps,
            "post_steps": post_natural_steps,
        },
        "decision": {
            "F1_fit_completed": f1,
            "F2_holdout_correct_symbols_strictly_above_baseline": f2,
            "F3_full_beats_all_three_controls_on_holdout": f3,
            "F4_operator_still_exact": f4,
            "F4_pre": f4_pre,
            "F4_post": f4_post,
            "holdout_correct_symbols_pre": h0,
            "holdout_correct_symbols_post": h1,
            "all_correct_symbols_pre": a0,
            "all_correct_symbols_post": a1,
            "holdout_positions": arm_metrics(&post, "holdout", "Full")["positions"],
            "controls": weaker,
            "status": status,
            "promotion": false,
        },
        "secondary": {
            "S1_copy_pre_fit": pre_forced["totals"]["copy_steps"],
            "S1_copy_post_fit": post_forced["totals"]["copy_steps"],
            "S1_arithmetic_steps_pre": pre_forced["totals"]["arithmetic_steps"],
            "S1_arithmetic_steps_post": post_forced["totals"]["arithmetic_steps"],
        },
        "cost": {
            "attempt_us": attempt_started.elapsed().as_micros(),
            "fit_us": fit_us,
            "evaluation_us": pre_us + post_us + pre_forced_us + post_forced_us + pre_natural_us + post_natural_us,
            "fit_wall_clock_cap_ms": FIT_CAP_MS,
            "attempt_wall_clock_cap_ms": ATTEMPT_CAP_MS,
            "cpu": "local CPU only; no GPU, no pod, no external cost",
        },
        "honesty": [
            "Expected answers are constructed in this instrument by i64 arithmetic; a weaker instrument than a frozen fixture.",
            "Fit and holdout are disjoint example sets but both authored from the same 32-problem table.",
            "No production source file is modified; the only new path is this measurement bin.",
        ],
        "criteria_digest_blake3": criteria_digest,
        "bin_source_blake3": hex::encode(blake3::hash(include_bytes!("addressed-attention-arith-fit.rs").as_slice()).as_bytes()),
    });
    std::fs::write(root.join("report.json"), serde_json::to_vec_pretty(&report)?)?;

    println!(
        "[aa-arith-fit] updates={updates} stopped_at_cap={stopped_at_cap} fit_us={fit_us} ce_first={ce_first:.6} ce_last={ce_last:.6} ce_at_64={}",
        ce_at_milestone.map(|v| format!("{v:.6}")).unwrap_or_else(|| "NA".into())
    );
    println!(
        "[aa-arith-fit] FULL all: pre {}/{} post {}/{} | holdout: pre {} post {} of {}",
        a0,
        arm_metrics(&post, "all", "Full")["positions"],
        a1,
        arm_metrics(&post, "all", "Full")["positions"],
        h0,
        h1,
        arm_metrics(&post, "holdout", "Full")["positions"]
    );
    println!(
        "[aa-arith-fit] forced C2: arithmetic pre {} post {}; value-exact pre {} post {}; emitted==produced-byte pre {} post {}",
        pre_forced["totals"]["arithmetic_steps"],
        post_forced["totals"]["arithmetic_steps"],
        pre_forced["totals"]["correct_value_steps"],
        post_forced["totals"]["correct_value_steps"],
        pre_forced["totals"]["copy_steps"],
        post_forced["totals"]["copy_steps"],
    );
    println!("[aa-arith-fit] status={status}");
    Ok(())
}
