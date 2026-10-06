//! Independent measurement: does the arithmetic RESULT reach the emission head's
//! CONTEXT, or is the emission head structurally unable to see it?
//!
//! This is a measurement-only binary. It adds no production code path and never
//! mutates frozen parameters, geometry, pilot data or any committed artifact.
//!
//! Method:
//!   (1) drive the unmodified engine with a harness policy that forces the read
//!       to the intended operand spans and forces the AddAB operator (same
//!       technique as addressed-attention-arith-oracle.rs), over examples whose
//!       computed values differ;
//!   (2) on each arithmetic answer step, capture (a) the computed value, (b) the
//!       packed 1024-bit input handed to `Policy::context` for the Emit phase,
//!       (c) the resulting context passed to `Head::Emission`;
//!   (3) surgically re-evaluate the frozen circuit while varying ONLY the byte
//!       channels (active.byte @792, selected_a.first_byte @776,
//!       selected_b.first_byte @784), holding every other bit fixed, to decide
//!       whether the RESULT channel reaches the 9-bit context.
use std::path::Path;

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
use uor_r4_core::native_geometric::addressed_attention::policy::{
    implementation_digest, CompiledPolicy, Parameters,
};
use uor_r4_core::report_output;

type AnyResult<T> = Result<T, Box<dyn std::error::Error>>;

const PARAMETER_SEED: u64 = 7341;
const MODEL_DATA: &[u8] = b"addressed-attention-emission-reach/1";
const CONFIG_DIGEST: &[u8] = b"addressed-attention-emission-reach-config/1";

/// Byte-channel bit offsets in the packed 1024-bit input (inputs.rs).
const BIT_A_FIRST_BYTE: usize = 776;
const BIT_B_FIRST_BYTE: usize = 784;
const BIT_ACTIVE_BYTE: usize = 792;

/// (a, b) addition problems whose computed values differ. Includes the three
/// examples named in the question.
const PROBLEMS: [(i64, i64); 9] = [
    (4, 5),  // -> 9   (question example)
    (1, 2),  // -> 3   (question example)
    (1, 14), // -> 15  (question example)
    (2, 3),  // -> 5
    (3, 4),  // -> 7
    (4, 4),  // -> 8
    (5, 7),  // -> 12 (two-digit)
    (6, 8),  // -> 14 (two-digit)
    (9, 9),  // -> 18 (two-digit)
];

fn symbol_code(s: Symbol) -> u16 {
    match s {
        Symbol::Byte(b) => u16::from(b),
        Symbol::Eos => 256,
    }
}

// ------------------------------------------------------------ oracle policy

/// Harness policy: forces the read to the intended operand spans (roots, nulls,
/// extent) and forces the AddAB operator, while capturing the packed input and
/// context the engine actually computes for each phase.
struct CapturePolicy<'a> {
    inner: CompiledPolicy<'a>,
    phase: Phase,
    force: bool,
    plan: Plan,
    /// Packed input captured for the Emit phase of the current tick.
    emit_input: Option<[bool; 1024]>,
    /// Packed input captured for the Control phase of the current tick.
    control_input: Option<[bool; 1024]>,
}

#[derive(Clone, Copy, Debug, Default)]
struct Plan {
    q: [[u16; 2]; 2],
    n: [[u16; 2]; 2],
    len: [u8; 2],
}

impl<'a> Policy for CapturePolicy<'a> {
    fn context(&mut self, phase: Phase, input: &[bool; 1024]) -> engine::Result<u16> {
        self.phase = phase;
        match phase {
            Phase::Emit => self.emit_input = Some(*input),
            Phase::Control => self.control_input = Some(*input),
            _ => {}
        }
        self.inner.context(phase, input)
    }
    fn choice(&mut self, head: Head, context: u16, legal: &[bool]) -> engine::Result<usize> {
        let mut choice = self.inner.choice(head, context, legal)?;
        if self.force {
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
            if let Head::Control = head {
                // Force AddAB (index 3) whenever legal.
                if legal.get(3).copied().unwrap_or(false) {
                    choice = 3;
                }
            }
        }
        Ok(choice)
    }
}

/// Harness-side query/null/extent plan for the intended operand spans. Faithful
/// replay of the engine's own selection loop. Copied from arith-oracle.
fn compute_plan(
    g: &BoundGeometry,
    runtime: &RuntimeSession,
    epoch: u64,
    spans: [(u64, u8); 2],
) -> AnyResult<(Plan, bool)> {
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
        let (_, n) = worst.ok_or("no null pair")?;
        plan.q[which] = q;
        plan.n[which] = n;
        plan.len[which] = len;
        // Replay the engine's selection to confirm the intended occurrence wins.
        let mut score = g.score(q, n)?;
        let mut winner_is_intended = false;
        for s in frontier.saturating_sub(256)..frontier {
            let rec = runtime.objects().occurrence(epoch, s)?;
            let c = g.score(q, rec.keys)?;
            if c > score {
                score = c;
                winner_is_intended = s == start;
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
                winner_is_intended = false;
            }
        }
        if !winner_is_intended {
            return Ok((plan, false));
        }
    }
    Ok((plan, true))
}

// ------------------------------------------------------------------ helpers

/// Back-propagate the fixed wiring from the 9 output bits to count which input
/// bits can structurally reach the context, independent of any LUT truth table.
/// Returns (reachable_input_bit_count, whether byte fields 776..800 reachable).
fn topology_reachability(circuit: &PrimitiveExport) -> (usize, bool) {
    let wires = circuit.circuit.topology().wires();
    // Output gates 0..9 are all "reachable" (they are the 9 context bits).
    let mut layer3 = [false; 16];
    for g in 0..9usize {
        for p in 0..4 {
            layer3[wires[336 + g][p] as usize] = true;
        }
    }
    let mut layer2 = [false; 64];
    for g in 0..16usize {
        if layer3[g] {
            for p in 0..4 {
                layer2[wires[320 + g][p] as usize] = true;
            }
        }
    }
    let mut layer1 = [false; 256];
    for g in 0..64usize {
        if layer2[g] {
            for p in 0..4 {
                layer1[wires[256 + g][p] as usize] = true;
            }
        }
    }
    let mut input = [false; 1024];
    for g in 0..256usize {
        if layer1[g] {
            for p in 0..4 {
                input[wires[g][p] as usize] = true;
            }
        }
    }
    let total = input.iter().filter(|&&b| b).count();
    let bytes_reachable = (BIT_A_FIRST_BYTE..BIT_ACTIVE_BYTE + 8).all(|i| input[i]);
    (total, bytes_reachable)
}

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

/// Re-evaluate the frozen circuit while varying exactly one byte channel.
/// Returns (distinct_context_count, sample_map) over all 256 values.
fn sweep_channel(
    circuit: &PrimitiveExport,
    input: &[bool; 1024],
    offset: usize,
) -> AnyResult<(usize, Vec<[u64; 2]>)> {
    let mut seen: std::collections::BTreeMap<u16, u8> = std::collections::BTreeMap::new();
    for v in 0..=255u8 {
        let ctx = circuit.circuit.evaluate(&set_byte_bits(input, offset, v))?;
        seen.entry(ctx).or_insert(v);
    }
    // Preserve a representative value for each distinct context.
    let mut sample: Vec<[u64; 2]> = seen
        .iter()
        .map(|(&ctx, &v)| [u64::from(ctx), u64::from(v)])
        .collect();
    sample.sort_unstable();
    Ok((seen.len(), sample))
}

// ------------------------------------------------------------------ driver

#[allow(clippy::too_many_arguments)]
fn drive_one(
    g: &BoundGeometry,
    params: &Parameters,
    compiled: &PrimitiveExport,
    a: i64,
    b: i64,
    index: usize,
) -> AnyResult<Value> {
    let epoch = index as u64 + 1;
    let prompt = format!("{a} {b}\n");
    let spans: [(u64, u8); 2] = [
        (0, a.to_string().len() as u8),
        (a.to_string().len() as u64 + 1, b.to_string().len() as u8),
    ];
    let mut runtime = RuntimeSession::new(params.digest(), g, epoch)?;
    let mut policy = CapturePolicy {
        inner: CompiledPolicy { compiled },
        phase: Phase::QueryA,
        force: false,
        plan: Plan::default(),
        emit_input: None,
        control_input: None,
    };
    // Prefill: observe the prompt bytes through the unforced engine.
    for &byte in prompt.as_bytes() {
        let offer = runtime.predict(g, &mut policy)?;
        runtime.observe(Symbol::Byte(byte), offer.offer.id, g, &mut policy)?;
    }
    // Answer step: force read + AddAB, capture Emit input and context.
    let (plan, read_ok) = compute_plan(g, &runtime, epoch, spans)?;
    policy.plan = plan;
    policy.force = true;
    let offered = runtime.predict(g, &mut policy)?;
    let emit_input = policy
        .emit_input
        .ok_or("no Emit input captured on answer step")?;
    let emit_context = offered.trace.contexts[Phase::Emit as usize];
    let control_context = offered.trace.contexts[Phase::Control as usize];

    let operands = offered.offer.operands();
    let values: [Option<i64>; 2] =
        operands.map(|o| o.and_then(|l| objects::decode_i64(l.payload()).ok()));
    let action = offered.offer.action;
    let produced = offered
        .offer
        .active()
        .and_then(|x| x.provisional().map(|_| x.lease().payload().to_vec()));
    let active_byte = offered.offer.active().map(objects::ActiveLease::byte);
    let active_len = offered.offer.active().map(|x| x.lease().payload().len());
    let emitted = symbol_code(offered.offer.symbol);

    // Confirm the packed Emit input actually carries the result digit at 792.
    let packed_active_byte = byte_from_bits(&emit_input, BIT_ACTIVE_BYTE);
    let packed_a_byte = byte_from_bits(&emit_input, BIT_A_FIRST_BYTE);
    let packed_b_byte = byte_from_bits(&emit_input, BIT_B_FIRST_BYTE);

    let is_arithmetic = matches!(action, Action::AddAB | Action::SubAB);
    if !is_arithmetic {
        return Err(format!("example {index}: action is {action:?}, not arithmetic").into());
    }

    // Surgical sweep of each byte channel on the captured Emit input.
    let sweep_active = sweep_channel(compiled, &emit_input, BIT_ACTIVE_BYTE)?;
    let sweep_a = sweep_channel(compiled, &emit_input, BIT_A_FIRST_BYTE)?;
    let sweep_b = sweep_channel(compiled, &emit_input, BIT_B_FIRST_BYTE)?;

    Ok(json!({
        "index": index,
        "a": a,
        "b": b,
        "prompt": prompt.trim_end().to_string(),
        "read_ok": read_ok,
        "operand_a": values[0],
        "operand_b": values[1],
        "computed_value": values[0].zip(values[1]).and_then(|(x, y)| x.checked_add(y)),
        "produced_payload": produced.as_ref().map(|p| String::from_utf8_lossy(p).to_string()),
        "active_byte": active_byte,
        "active_length": active_len,
        "emit_context": emit_context.map(u64::from),
        "control_context": control_context.map(u64::from),
        "emitted_symbol": emitted,
        "packed_input": {
            "active_byte_field": packed_active_byte,
            "a_first_byte_field": packed_a_byte,
            "b_first_byte_field": packed_b_byte,
            "active_byte_field_matches_result_digit": active_byte.map(|x| x == packed_active_byte),
        },
        "sweep": {
            "active_byte_distinct_contexts": sweep_active.0,
            "active_byte_sample": sweep_active.1,
            "a_first_byte_distinct_contexts": sweep_a.0,
            "a_first_byte_sample": sweep_a.1,
            "b_first_byte_distinct_contexts": sweep_b.0,
            "b_first_byte_sample": sweep_b.1,
        },
    }))
}

fn main() -> AnyResult<()> {
    let root = std::env::args()
        .nth(1)
        .ok_or("usage: addressed-attention-emission-reach <new-absolute-report-root>")?;
    let root = Path::new(&root);
    if !root.is_absolute() {
        return Err("report root must be an absolute path".into());
    }
    report_output::claim(root)?;

    let params = Parameters::seeded(PARAMETER_SEED)?;
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
    let g = model.geometry();

    let mut rows = Vec::new();
    for (index, &(a, b)) in PROBLEMS.iter().enumerate() {
        let row = drive_one(g, &params, model.compiled(), a, b, index)?;
        println!(
            "[reach] {a}+{b} emit_context={} computed={} active_byte={} emitted={} active_sweep_distinct={}",
            row["emit_context"],
            row["computed_value"],
            row["active_byte"],
            row["emitted_symbol"],
            row["sweep"]["active_byte_distinct_contexts"],
        );
        rows.push(row);
    }

    let any_active_varies = rows.iter().any(|r| {
        r["sweep"]["active_byte_distinct_contexts"].as_u64().unwrap_or(0) > 1
    });
    let all_active_varies = rows.iter().all(|r| {
        r["sweep"]["active_byte_distinct_contexts"].as_u64().unwrap_or(0) > 1
    });
    let distinct_emit_contexts: std::collections::BTreeSet<u64> = rows
        .iter()
        .filter_map(|r| r["emit_context"].as_u64())
        .collect();
    let topo = topology_reachability(model.compiled());

    let report = json!({
        "schema": "uor-r4.addressed-attention-emission-reach/1",
        "purpose": "Does the arithmetic RESULT reach the emission head's context?",
        "parameter_seed": params.seed(),
        "source_digest": hex::encode(implementation_digest()),
        "topology": {
            "reachable_input_bits_of_1024": topo.0,
            "byte_fields_776_to_800_reachable": topo.1,
            "note": "Back-propagation of the fixed WIRING_SEED=0x973 permutation topology from the 9 output bits; independent of LUT truth tables (parameters)."
        },
        "rows": rows,
        "summary": {
            "examples": PROBLEMS.len(),
            "distinct_emit_contexts_across_examples": distinct_emit_contexts.len(),
            "any_example_active_byte_reaches_context": any_active_varies,
            "all_examples_active_byte_reaches_context": all_active_varies,
        },
        "structural_trace": {
            "value_computed": "objects.rs prepare_action AddAB/SubAB -> execute() -> encode_i64() -> ActiveLease.bytes (cursor 0)",
            "value_placed_in_input": "engine.rs inputs() maps prospective.active() -> Active.byte (objects.rs ActiveLease::byte), packed at bits 792..800 by inputs.rs pack()",
            "context_derived": "engine.rs call() -> policy.context(Phase::Emit, input.pack(Emit)) -> circuit.evaluate -> 9-bit context",
            "emission_head_sees": "policy.rs head_range(Head::Emission, context) -> emission row argmax; the ONLY per-context information is the 9-bit context",
            "bit_offsets": {
                "selected_a_first_byte": BIT_A_FIRST_BYTE,
                "selected_b_first_byte": BIT_B_FIRST_BYTE,
                "active_byte": BIT_ACTIVE_BYTE,
            },
        },
        "caveats": [
            "Parameters are the unfitted seed-7341 init; the emission head argmax is untrained.",
            "This probe changes no production code; the engine, objects, geometry and compiled heads are unmodified.",
            "The sweep varies ONLY one 8-bit byte field while holding all 1023 other bits fixed, so it isolates that channel's reachability into the 9-bit context.",
        ],
    });
    std::fs::write(root.join("report.json"), serde_json::to_vec_pretty(&report)?)?;
    report_output::seal(root)?;
    report_output::verify(root)?;
    println!("[reach] sealed {}", root.display());
    println!(
        "[reach] any_active_varies={any_active_varies} all_active_varies={all_active_varies} distinct_emit_contexts={}",
        distinct_emit_contexts.len()
    );
    Ok(())
}
