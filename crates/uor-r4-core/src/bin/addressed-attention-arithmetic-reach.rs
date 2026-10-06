//! Bounded reachability probe for `AddAB`/`SubAB` at the addressed-attention
//! control head.
//!
//! Measurement-only. It adds no production code path, changes no default
//! behaviour, and never mutates frozen parameters, geometry, pilot data or any
//! committed artifact. It answers one question: can ANY objective make the
//! control head select `AddAB`/`SubAB`, and if not, is the exclusion a property
//! of the legal mask (`objects.rs::prepare_operands`) rather than of the
//! sampled objective?
//!
//! Three sections, each written to its own exclusively claimed and sealed root:
//!   1. `mask-probe`   direct `ObjectSession::prepare_operands` masks for numeric
//!                     and non-numeric payloads (no policy, no sampling).
//!   2. `drive-*`      full `RuntimeSession::predict`/`observe` drives under
//!                     several control-head objectives (sampled, argmax,
//!                     arithmetic-first, hard-add), recording per step whether
//!                     the mask OFFERED indices 3/4 and whether the policy CHOSE
//!                     them. This separates "offered but not chosen" (objective)
//!                     from "never offered" (structural).
//!   3. `pilot-path`   the frozen 8-example development split driven through the
//!                     crate's own `pilot::Controlled` policy with a recording
//!                     wrapper, plus `pilot::evaluate` metrics for Full and the
//!                     three destructive controls.
use std::path::Path;
use std::time::Instant;

use serde_json::{json, Value};
use uor_r4_core::native_geometric::addressed_attention::artifact::{
    BoundGeometry, Model, Provenance,
};
use uor_r4_core::native_geometric::addressed_attention::engine::{self, Head, Policy, RuntimeSession};
use uor_r4_core::native_geometric::addressed_attention::inputs::Phase;
use uor_r4_core::native_geometric::addressed_attention::objects::{Action, ObjectSession, Symbol};
use uor_r4_core::native_geometric::addressed_attention::pilot::{self, Control, Controlled};
use uor_r4_core::native_geometric::addressed_attention::pilot_data;
use uor_r4_core::native_geometric::addressed_attention::policy::{
    head_range, implementation_digest, Parameters, SampledPolicy,
};
use uor_r4_core::report_output;

type AnyResult<T> = Result<T, Box<dyn std::error::Error>>;

const MODEL_DATA: &[u8] = b"addressed-attention-arithmetic-reach/1";
const CONFIG_DIGEST: &[u8] = b"addressed-attention-arithmetic-reach-config/1";

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

fn counted<T: Copy + std::fmt::Debug>(values: &[T]) -> Value {
    let mut out: Vec<(String, u64)> = Vec::new();
    for v in values {
        let key = format!("{v:?}");
        match out.iter_mut().find(|(k, _)| *k == key) {
            Some((_, n)) => *n += 1,
            None => out.push((key, 1)),
        }
    }
    out.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    serde_json::to_value(out).unwrap_or(Value::Null)
}

fn emit(base: &Path, name: &str, value: &Value) -> AnyResult<()> {
    let root = base.join(name);
    report_output::claim(&root)?;
    std::fs::write(root.join("report.json"), serde_json::to_vec_pretty(value)?)?;
    report_output::seal(&root)?;
    report_output::verify(&root)?;
    Ok(())
}

// ---------------------------------------------------------------- head stats

/// Per-head accounting for the control head. `offered_*` counts steps where the
/// engine's legal mask admitted index 3 / 4 (AddAB / SubAB); the engine passes
/// this mask verbatim from `ObjectSession::prepare_operands` (engine.rs:384).
#[derive(Clone, Copy, Debug, Default)]
struct Stats {
    control_calls: u64,
    add_offered: u64,
    sub_offered: u64,
    arith_offered_steps: u64,
    chosen_add: u64,
    chosen_sub: u64,
    legal_sizes: [u64; 8],
}

impl Stats {
    fn record(&mut self, legal: &[bool], choice: usize) {
        self.control_calls += 1;
        let add = legal.get(3).copied().unwrap_or(false);
        let sub = legal.get(4).copied().unwrap_or(false);
        if add {
            self.add_offered += 1;
        }
        if sub {
            self.sub_offered += 1;
        }
        if add || sub {
            self.arith_offered_steps += 1;
        }
        match choice {
            3 => self.chosen_add += 1,
            4 => self.chosen_sub += 1,
            _ => {}
        }
        let k = legal.iter().filter(|&&b| b).count();
        if let Some(slot) = self.legal_sizes.get_mut(k) {
            *slot += 1;
        }
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
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Objective {
    /// The crate's own `SampledPolicy` draw (softmax over the legal mask).
    Sampled,
    /// Deterministic argmax over the legal mask using the same 7 logits.
    ControlArgmax,
    /// Return AddAB if legal, else SubAB if legal, else the sampled draw.
    ArithFirst,
    /// Always return AddAB, legal or not (the engine must reject it).
    HardAdd,
}

struct Probe<'a> {
    inner: SampledPolicy<'a>,
    params: &'a Parameters,
    objective: Objective,
    stats: Stats,
}

impl<'a> Probe<'a> {
    fn new(params: &'a Parameters, objective: Objective, event_seed: u64) -> Self {
        Self {
            inner: SampledPolicy::new(params, event_seed, 0, 0),
            params,
            objective,
            stats: Stats::default(),
        }
    }
    fn argmax_legal(&self, head: Head, context: u16, legal: &[bool]) -> engine::Result<usize> {
        let (offset, n) = head_range(head, context)?;
        let logits = &self.params.values()[offset..offset + n];
        let mut best: Option<usize> = None;
        for (i, &allowed) in legal.iter().enumerate() {
            if allowed && best.is_none_or(|b| logits[i] > logits[b]) {
                best = Some(i);
            }
        }
        best.ok_or(engine::EngineError::Invalid("empty legal support"))
    }
}

impl Policy for Probe<'_> {
    fn context(&mut self, phase: Phase, input: &[bool; 1024]) -> engine::Result<u16> {
        self.inner.context(phase, input)
    }
    fn choice(&mut self, head: Head, context: u16, legal: &[bool]) -> engine::Result<usize> {
        let choice = match (head, self.objective) {
            (Head::Control, Objective::ControlArgmax) => self.argmax_legal(head, context, legal)?,
            (Head::Control, Objective::ArithFirst) => {
                if legal.get(3).copied().unwrap_or(false) {
                    3
                } else if legal.get(4).copied().unwrap_or(false) {
                    4
                } else {
                    self.inner.choice(head, context, legal)?
                }
            }
            (Head::Control, Objective::HardAdd) => 3,
            _ => self.inner.choice(head, context, legal)?,
        };
        if matches!(head, Head::Control) {
            self.stats.record(legal, choice);
        }
        Ok(choice)
    }
}

// ------------------------------------------------------------- mask section

fn mask_case(label: &str, doc: &[u8], spans: [(u64, u8); 2]) -> AnyResult<Value> {
    let mut session = ObjectSession::new([0u8; 32], [0u8; 32], 1);
    for &byte in doc {
        session.observe_input(byte, [0, 0], [0, 0, 0, 0])?;
    }
    let a = session.acquire_occurrence(1, spans[0].0, spans[0].1)?;
    let b = session.acquire_occurrence(1, spans[1].0, spans[1].1)?;
    let payloads = [
        String::from_utf8_lossy(a.payload()).to_string(),
        String::from_utf8_lossy(b.payload()).to_string(),
    ];
    let prepared = session.prepare_operands(Some(a), Some(b))?;
    Ok(json!({
        "label": label,
        "document": String::from_utf8_lossy(doc),
        "spans": [{"start": spans[0].0, "len": spans[0].1}, {"start": spans[1].0, "len": spans[1].1}],
        "payloads": payloads,
        "numeric_valid": prepared.numeric_valid(),
        "legal_mask": prepared.legal(),
        "add_offered": prepared.legal()[3],
        "sub_offered": prepared.legal()[4],
    }))
}

// ------------------------------------------------------------ drive section

#[derive(Clone, Copy)]
struct DriveConfig {
    name: &'static str,
    doc: &'static [u8],
    append_eos: bool,
    param_seed: u64,
    event_seed: u64,
    steps: usize,
    objective: Objective,
}

fn drive(config: &DriveConfig, params: &Parameters, geometry: &BoundGeometry, model: &Model) -> AnyResult<Value> {
    let mut document: Vec<u16> = config.doc.iter().map(|&b| u16::from(b)).collect();
    if config.append_eos {
        document.push(256);
    }
    let mut session = RuntimeSession::new(model.id(), geometry, 41)?;
    let mut policy = Probe::new(params, config.objective, config.event_seed);
    let mut actions: Vec<Action> = Vec::with_capacity(config.steps);
    let mut outputs: Vec<u16> = Vec::with_capacity(config.steps);
    let mut reads: Vec<u8> = Vec::with_capacity(config.steps);
    let mut turns = 1u64;
    let mut previous_actual: Option<Symbol> = None;
    let mut error: Option<Value> = None;
    let mut completed = 0usize;
    let started = Instant::now();
    for step in 0..config.steps {
        if previous_actual == Some(Symbol::Eos) {
            session.begin_turn()?;
            turns += 1;
        }
        let target = document[step % document.len()];
        policy.inner.set_target(usize::from(target), document.len())?;
        let offered = match session.predict(geometry, &mut policy) {
            Ok(offered) => offered,
            Err(e) => {
                error = Some(json!({
                    "status": "PREDICT_ERROR",
                    "at_step": step,
                    "error": e.to_string(),
                    "target_left_pending": policy.inner.target_pending(),
                    "note": "the engine rejects the objective's choice as illegal before any offer is cached",
                }));
                break;
            }
        };
        actions.push(offered.trace.action);
        outputs.push(match offered.offer.symbol {
            Symbol::Byte(b) => u16::from(b),
            Symbol::Eos => 256,
        });
        reads.push(offered.trace.selected.iter().flatten().count() as u8);
        let actual = if target == 256 {
            Symbol::Eos
        } else {
            Symbol::Byte(target as u8)
        };
        session.observe(actual, offered.offer.id, geometry, &mut policy)?;
        previous_actual = Some(actual);
        completed += 1;
        if previous_actual == Some(Symbol::Eos) && completed == config.steps {
            break;
        }
    }
    let elapsed_us = started.elapsed().as_micros();
    let work = session.work();
    let counts = policy.inner.counts();
    Ok(json!({
        "name": config.name,
        "document": String::from_utf8_lossy(config.doc),
        "document_len": document.len(),
        "append_eos": config.append_eos,
        "param_seed": config.param_seed,
        "event_seed": config.event_seed,
        "objective": format!("{:?}", config.objective),
        "requested_steps": config.steps,
        "steps_completed": completed,
        "turns": turns,
        "elapsed_us": elapsed_us,
        "stats": policy.stats.json(),
        "work": {
            "circuit_calls": work.circuit_calls,
            "candidate_scores": work.candidate_scores,
            "predictions": work.predictions,
            "observations": work.observations,
            "scalar_decodes": work.scalar_decodes,
            "selected_operations": work.selected_operations,
        },
        "action_histogram": counted(&actions),
        "add_plus_sub": policy.stats.chosen_add + policy.stats.chosen_sub,
        "selected_operations": work.selected_operations,
        "steps_with_two_reads": reads.iter().filter(|&&r| r == 2).count(),
        "distinct_offered_symbols": outputs.iter().collect::<std::collections::BTreeSet<_>>().len(),
        "policy_circuit_calls": counts.circuit_calls,
        "error": error,
    }))
}

// ------------------------------------------------------------ pilot section

struct PilotProbe<'a> {
    inner: Controlled<'a>,
    stats: Stats,
}
impl Policy for PilotProbe<'_> {
    fn context(&mut self, phase: Phase, input: &[bool; 1024]) -> engine::Result<u16> {
        self.inner.context(phase, input)
    }
    fn choice(&mut self, head: Head, context: u16, legal: &[bool]) -> engine::Result<usize> {
        let choice = self.inner.choice(head, context, legal)?;
        if matches!(head, Head::Control) {
            self.stats.record(legal, choice);
        }
        Ok(choice)
    }
}

/// Replicates `pilot::evaluate`'s recipe with a recording wrapper; the metrics
/// themselves are taken from the crate's own `pilot::evaluate` unchanged.
fn pilot_path(
    params: &Parameters,
    compiled: &uor_r4_core::native_geometric::addressed_attention::circuit::PrimitiveExport,
    geometry: &BoundGeometry,
    examples: &[pilot_data::Example],
) -> AnyResult<Value> {
    let mut stats = Stats::default();
    let mut positions = 0u64;
    for (i, example) in examples.iter().enumerate() {
        let mut runtime = RuntimeSession::new(params.digest(), geometry, i as u64 + 1)?;
        let mut policy = PilotProbe {
            inner: Controlled::new(compiled, Control::Full, geometry.identity()),
            stats: Stats::default(),
        };
        for &byte in &example.prompt {
            let offer = runtime.predict(geometry, &mut policy)?;
            runtime.observe(Symbol::Byte(byte), offer.offer.id, geometry, &mut policy)?;
        }
        for target in example
            .answer
            .iter()
            .map(|&b| u16::from(b))
            .chain([256])
        {
            let offer = runtime.predict(geometry, &mut policy)?;
            let actual = if target == 256 {
                Symbol::Eos
            } else {
                Symbol::Byte(target as u8)
            };
            runtime.observe(actual, offer.offer.id, geometry, &mut policy)?;
            positions += 1;
        }
        let s = policy.stats;
        stats.control_calls += s.control_calls;
        stats.add_offered += s.add_offered;
        stats.sub_offered += s.sub_offered;
        stats.arith_offered_steps += s.arith_offered_steps;
        stats.chosen_add += s.chosen_add;
        stats.chosen_sub += s.chosen_sub;
        for (dst, src) in stats.legal_sizes.iter_mut().zip(s.legal_sizes) {
            *dst += src;
        }
    }
    Ok(json!({
        "examples": examples.len(),
        "scored_positions": positions,
        "stats": stats.json(),
    }))
}

// --------------------------------------------------------------------- main

fn main() -> AnyResult<()> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build_global()
        .map_err(|e| format!("thread pool: {e}"))?;
    let base = std::env::args()
        .nth(1)
        .ok_or("usage: addressed-attention-arithmetic-reach <new-absolute-base-dir>")?;
    let base = std::path::PathBuf::from(base);
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
    let started = Instant::now();

    // ---- section 1: raw legal masks from prepare_operands, no policy involved
    let masks = json!({
        "schema": "uor-r4.addressed-attention-mask-probe/1",
        "source": "ObjectSession::prepare_operands -> PreparedOperands::legal() (objects.rs:227-263)",
        "cases": [
            mask_case("non_numeric_words", b"Ada is in Lima.", [(0, 3), (11, 4)])?,
            mask_case("numeric_single_digits", b"3 5", [(0, 1), (2, 1)])?,
            mask_case("numeric_multi_digits", b"12 345", [(0, 2), (3, 3)])?,
            mask_case("leading_zero_invalid", b"007 5", [(0, 3), (4, 1)])?,
            mask_case("overflow_add_only", b"9223372036854775807 1", [(0, 19), (20, 1)])?,
            mask_case("mixed_word_and_number", b"Ada 42", [(0, 3), (4, 2)])?,
        ],
    });
    emit(&base, "mask-probe", &masks)?;

    // ---- section 2: drives under several control-head objectives
    let base_doc: &[u8] = b"Ada is in Lima. Ben is in Oslo.\nAda?\n";
    let digits_space: &[u8] = b"1 2 3 4 5 6 7 8 9 0\n";
    let digits_run: &[u8] = b"123456789123456789123456789\n";
    let program: &[u8] = b"Rust: print a+b; a=3,b=5.\nfn main(){println!(\"{}\",8);}\n";
    let configs: [DriveConfig; 17] = [
        DriveConfig { name: "00-harness-baseline-eos-1000", doc: base_doc, append_eos: true, param_seed: 7341, event_seed: 973, steps: 1000, objective: Objective::Sampled },
        DriveConfig { name: "01-base-sampled-1000", doc: base_doc, append_eos: false, param_seed: 7341, event_seed: 973, steps: 1000, objective: Objective::Sampled },
        DriveConfig { name: "02-base-argmax-1000", doc: base_doc, append_eos: false, param_seed: 7341, event_seed: 973, steps: 1000, objective: Objective::ControlArgmax },
        DriveConfig { name: "03-base-arithfirst-1000", doc: base_doc, append_eos: false, param_seed: 7341, event_seed: 973, steps: 1000, objective: Objective::ArithFirst },
        DriveConfig { name: "04-base-hardadd", doc: base_doc, append_eos: false, param_seed: 7341, event_seed: 973, steps: 1000, objective: Objective::HardAdd },
        DriveConfig { name: "05-digits-space-sampled-1000", doc: digits_space, append_eos: false, param_seed: 7341, event_seed: 973, steps: 1000, objective: Objective::Sampled },
        DriveConfig { name: "06-digits-run-sampled-1000", doc: digits_run, append_eos: false, param_seed: 7341, event_seed: 973, steps: 1000, objective: Objective::Sampled },
        DriveConfig { name: "07-program-sampled-1000", doc: program, append_eos: false, param_seed: 7341, event_seed: 973, steps: 1000, objective: Objective::Sampled },
        DriveConfig { name: "08-digits-run-argmax-1000", doc: digits_run, append_eos: false, param_seed: 7341, event_seed: 973, steps: 1000, objective: Objective::ControlArgmax },
        DriveConfig { name: "09-digits-run-seed973", doc: digits_run, append_eos: false, param_seed: 973, event_seed: 973, steps: 1000, objective: Objective::Sampled },
        DriveConfig { name: "10-digits-run-seed41", doc: digits_run, append_eos: false, param_seed: 41, event_seed: 973, steps: 1000, objective: Objective::Sampled },
        DriveConfig { name: "11-digits-run-seed12345", doc: digits_run, append_eos: false, param_seed: 12345, event_seed: 973, steps: 1000, objective: Objective::Sampled },
        DriveConfig { name: "12-digits-run-event1", doc: digits_run, append_eos: false, param_seed: 7341, event_seed: 1, steps: 1000, objective: Objective::Sampled },
        DriveConfig { name: "13-digits-run-event7", doc: digits_run, append_eos: false, param_seed: 7341, event_seed: 7, steps: 1000, objective: Objective::Sampled },
        DriveConfig { name: "14-digits-run-sampled-4000", doc: digits_run, append_eos: false, param_seed: 7341, event_seed: 973, steps: 4000, objective: Objective::Sampled },
        DriveConfig { name: "15-base-sampled-4000", doc: base_doc, append_eos: false, param_seed: 7341, event_seed: 973, steps: 4000, objective: Objective::Sampled },
        DriveConfig { name: "16-digits-run-eos-1000", doc: digits_run, append_eos: true, param_seed: 7341, event_seed: 973, steps: 1000, objective: Objective::Sampled },
    ];

    let development = pilot_data::development();
    let mut rows: Vec<Value> = Vec::new();
    let mut pilot_by_seed: std::collections::BTreeMap<u64, Value> = std::collections::BTreeMap::new();

    let data_digest = *blake3::hash(MODEL_DATA).as_bytes();
    let config_digest = *blake3::hash(CONFIG_DIGEST).as_bytes();

    for config in &configs {
        let params = Parameters::seeded(config.param_seed)?;
        let compiled = params.compile()?;
        let model = Model::new(
            compiled,
            Provenance {
                seed: params.seed(),
                training_data_digest: data_digest,
                training_config_digest: config_digest,
                parameter_digest: params.digest(),
                source_digest: implementation_digest(),
                parent: None,
            },
        )?;
        let geometry = model.geometry();

        let drive_json = drive(config, &params, geometry, &model)?;
        emit(&base, config.name, &drive_json)?;
        rows.push(drive_json.clone());

        if !pilot_by_seed.contains_key(&config.param_seed) {
            let mut arms = serde_json::Map::new();
            for control in pilot::CONTROLS {
                let eval = pilot::evaluate(&params, model.compiled(), geometry, &development, control)?;
                arms.insert(format!("{control:?}"), serde_json::to_value(pilot::metrics(&eval))?);
            }
            let path = pilot_path(&params, model.compiled(), geometry, &development)?;
            let value = json!({"param_seed": config.param_seed, "arms": arms, "pilot_path": path});
            emit(&base, &format!("pilot-seed-{}", config.param_seed), &value)?;
            pilot_by_seed.insert(config.param_seed, value);
        }
        println!(
            "[reach] {} steps={} objective={:?} offered_steps={} add={} sub={} selected_operations={} error={}",
            config.name,
            drive_json["steps_completed"],
            config.objective,
            drive_json["stats"]["arithmetic_offered_steps"],
            drive_json["stats"]["chosen_add"],
            drive_json["stats"]["chosen_sub"],
            drive_json["work"]["selected_operations"],
            !drive_json["error"].is_null(),
        );
    }

    // ---- section 3: summary table
    let table: Vec<Value> = rows
        .iter()
        .map(|r| {
            let seed = r["param_seed"].as_u64().unwrap_or(0);
            let pilot = pilot_by_seed.get(&seed);
            json!({
                "name": r["name"],
                "document_kind": r["document"],
                "param_seed": seed,
                "event_seed": r["event_seed"],
                "objective": r["objective"],
                "steps_completed": r["steps_completed"],
                "control_calls": r["stats"]["control_calls"],
                "arithmetic_offered_steps": r["stats"]["arithmetic_offered_steps"],
                "add_offered": r["stats"]["add_offered"],
                "sub_offered": r["stats"]["sub_offered"],
                "add_chosen": r["stats"]["chosen_add"],
                "sub_chosen": r["stats"]["chosen_sub"],
                "add_plus_sub": r["add_plus_sub"],
                "selected_operations": r["work"]["selected_operations"],
                "predict_error": r["error"].clone(),
                "pilot_full": pilot.map(|p| p["arms"]["Full"].clone()),
                "pilot_read_disabled": pilot.map(|p| p["arms"]["ReadDisabled"].clone()),
                "pilot_exact_payload_masked": pilot.map(|p| p["arms"]["ExactPayloadMasked"].clone()),
                "pilot_state_transport_disabled": pilot.map(|p| p["arms"]["StateTransportDisabled"].clone()),
                "pilot_path": pilot.map(|p| p["pilot_path"].clone()),
            })
        })
        .collect();
    let any_chosen = rows.iter().any(|r| {
        r["stats"]["chosen_add"].as_u64().unwrap_or(0) + r["stats"]["chosen_sub"].as_u64().unwrap_or(0) > 0
    });
    let any_offered = rows
        .iter()
        .any(|r| r["stats"]["arithmetic_offered_steps"].as_u64().unwrap_or(0) > 0);
    let summary = json!({
        "schema": "uor-r4.addressed-attention-arithmetic-reach/1",
        "mechanism": [
            "engine.rs:384 `let legal = prepared.legal();` then engine.rs:385 `ACTIONS[choose(p, Head::Control, context, &legal)?]`: the control head's candidate set is exactly the 7-bit mask.",
            "objects.rs:233 `let mut legal = [true, a.is_some(), b.is_some(), false, false, false, true];` hard-false for AddAB(3)/SubAB(4).",
            "objects.rs:234-253 sets legal[3]/legal[4] only when `publications < 2` and `values` (objects.rs:232 `decode_i64(payload)`) are both Some, plus a sign-bounded overflow test.",
            "objects.rs:761-790 decode_i64 accepts only a canonical decimal (optional '-', no leading zeros, <=20 bytes).",
            "policy.rs:307 `categorical(logits, legal, key)` samples over the legal mask only; policy.rs:98 places the 7 control logits at head_range(Head::Control, context).",
            "engine.rs:125-133 `choose` rejects any returned index whose mask bit is false with EngineError::Invalid(\"policy selected illegal outcome\")."
        ],
        "any_offered": any_offered,
        "any_chosen": any_chosen,
        "elapsed_ms": started.elapsed().as_millis(),
        "configs": table,
        "caveats": [
            "All drives use the crate's unfitted Parameters::seeded initialization; no fit, no checkpoint.",
            "The document is the harness target stream (drive input), not engine semantics; the object session, geometry and engine are unchanged.",
            "This probe adds a new bin only; production behaviour is unchanged."
        ],
    });
    emit(&base, "summary", &summary)?;
    std::fs::write(base.join("sweep.json"), serde_json::to_vec_pretty(&summary)?)?;
    println!(
        "[reach] any_offered={any_offered} any_chosen={any_chosen} elapsed_ms={} base={}",
        started.elapsed().as_millis(),
        base.display()
    );
    Ok(())
}
