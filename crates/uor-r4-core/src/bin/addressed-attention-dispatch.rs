//! Bounded dispatch probe for the addressed_attention runtime.
//!
//! Purpose: decide whether `native_geometric::addressed_attention` is inert by
//! omission or inert by mathematics. This binary is measurement-only. It adds
//! no production code path, changes no default behaviour, and never mutates
//! the frozen parameters, geometry, pilot data or any committed artifact.
//!
//! It (a) constructs the canonical `BoundGeometry` and a `Model` exactly as the
//! in-crate tests do, (b) drives a `RuntimeSession` for a bounded number of
//! steps with the crate's own `SampledPolicy`, (c) reports the `Work` counters,
//! `ForwardTrace`s and every accuracy signal the crate already exposes
//! (`pilot::evaluate` metrics and `learner::batch` mean CE), and (d) writes the
//! whole report into an exclusively claimed report root.
use std::io::Write as _;
use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;

use serde_json::{json, Value};
use uor_r4_core::native_geometric::addressed_attention::artifact::{BoundGeometry, Model, Provenance};
use uor_r4_core::native_geometric::addressed_attention::engine::{ForwardTrace, RuntimeSession, Work};
use uor_r4_core::native_geometric::addressed_attention::learner;
use uor_r4_core::native_geometric::addressed_attention::objects::{Action, Reference, Symbol};
use uor_r4_core::native_geometric::addressed_attention::pilot::{self, Control};
use uor_r4_core::native_geometric::addressed_attention::pilot_data;
use uor_r4_core::native_geometric::addressed_attention::policy::{
    implementation_digest, Parameters, SampledPolicy,
};
use uor_r4_core::report_output;

type AnyResult<T> = Result<T, Box<dyn std::error::Error>>;

const PARAMETER_SEED: u64 = 7341;
const EVENT_SEED: u64 = 973;
const STEPS: usize = 1000;
const LAST_TRACES: usize = 8;
const MODEL_DATA: &[u8] = b"addressed-attention-dispatch-probe/1";
/// Bounded input sentence used for the scored path and the learner bound check.
const SCORED_SENTENCE: &[u8] = b"Ada is in Lima. Ben is in Oslo.\nAda?\n";

/// Mirror every report line to stdout as well as the report file, so the raw
/// output is durable both in the claimed root and in the run log.
struct Tee<'a>(&'a Mutex<std::fs::File>);
impl std::io::Write for Tee<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        std::io::stdout().write_all(buf)?;
        let mut file = self
            .0
            .lock()
            .map_err(|_| std::io::Error::other("report log lock poisoned"))?;
        file.write_all(buf)?;
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        std::io::stdout().flush()?;
        let mut file = self
            .0
            .lock()
            .map_err(|_| std::io::Error::other("report log lock poisoned"))?;
        file.flush()
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

fn reference_code(r: Option<Reference>) -> Value {
    match r {
        None => Value::Null,
        Some(Reference::Occurrence {
            epoch,
            turn,
            start,
            end,
        }) => json!({"kind": "occurrence", "epoch": epoch, "turn": turn, "start": start, "end": end}),
        Some(Reference::Result { epoch, id }) => json!({"kind": "result", "epoch": epoch, "id": id}),
    }
}

fn trace_json(t: &ForwardTrace) -> Value {
    json!({
        "phase_contexts": t.contexts.iter().map(|c| c.map(u64::from)).collect::<Vec<_>>(),
        "context_calls": t.circuit_calls(),
        "selected": [reference_code(t.selected[0]), reference_code(t.selected[1])],
        "roots_before": t.roots_before,
        "roots_after": t.roots_after,
        "offer_id": t.offer_id,
        "action": action_name(t.action),
        "output": symbol_code(t.output),
        "output_is_eos": t.output == Symbol::Eos,
        "actual": t.actual.map(symbol_code),
        "match": t.actual.map(|a| a == t.output),
        "cursor": t.cursor,
        "candidate_scores": t.candidate_scores,
        "numeric_valid": t.numeric_valid,
        "scalar_decodes": t.scalar_decodes,
        "selected_operations": t.selected_operations,
    })
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

fn main() -> AnyResult<()> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build_global()
        .map_err(|e| format!("thread pool: {e}"))?;
    let root = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("UOR_ADDRESSED_DISPATCH_REPORT").ok())
        .ok_or("usage: addressed-attention-dispatch <new-absolute-report-root>")?;
    let root = Path::new(&root).to_path_buf();
    if !root.is_absolute() {
        return Err("report root must be an absolute path".into());
    }
    for ancestor in root.parent().into_iter().flat_map(Path::ancestors) {
        if ancestor.join("manifest.json").exists() {
            return Err(format!(
                "report root {} is beneath an already sealed root",
                root.display()
            )
            .into());
        }
    }
    report_output::claim(&root)?;
    let log = Mutex::new(std::fs::File::create_new(root.join("run.log"))?);
    let started = Instant::now();

    let result = run(&root, &log, started);
    match &result {
        Ok(()) => {
            writeln!(
                Tee(&log),
                "[aa-dispatch] status=PASS_COMPLETE elapsed_ms={}",
                started.elapsed().as_millis()
            )?;
        }
        Err(error) => {
            let value = json!({
                "status": "INCOMPLETE_ADDRESSED_ATTENTION_DISPATCH_PROBE",
                "error": error.to_string(),
                "no_automatic_retry": true,
            });
            std::fs::write(
                root.join("incomplete.json"),
                serde_json::to_vec_pretty(&value)?,
            )?;
            writeln!(Tee(&log), "[aa-dispatch] INCOMPLETE: {error}")?;
        }
    }
    report_output::seal(&root)?;
    report_output::verify(&root)?;
    result
}

fn run(root: &Path, log: &Mutex<std::fs::File>, started: Instant) -> AnyResult<()> {
    // ---- (a) canonical construction, copied from engine_tests.rs / pilot_tests.rs
    let t0 = Instant::now();
    let geometry_source = BoundGeometry::canonical()?;
    let geometry_digest = geometry_source.identity_digest();
    let parameters = Parameters::seeded(PARAMETER_SEED)?;
    let parameter_digest = parameters.digest();
    let data_digest = *blake3::hash(MODEL_DATA).as_bytes();
    let config_digest = *blake3::hash(b"addressed-attention-dispatch-config/1").as_bytes();
    let compiled = parameters.compile()?;
    let model = Model::new(
        compiled,
        Provenance {
            seed: parameters.seed(),
            training_data_digest: data_digest,
            training_config_digest: config_digest,
            parameter_digest,
            source_digest: implementation_digest(),
            parent: None,
        },
    )?;
    let constructed_ms = t0.elapsed().as_millis();
    let geometry = model.geometry();

    // ---- (b) bounded drive with the crate's own SampledPolicy. The whole
    // target stream is the terminal-EOS pilot document; the frozen parameters
    // are NOT fitted, so this measures mechanics, not language quality.
    let document: Vec<u16> = SCORED_SENTENCE.iter().map(|&b| u16::from(b)).chain([256]).collect();
    writeln!(
        Tee(log),
        "[aa-dispatch] constructed geometry={} identity={} parameters={} model_bytes={} document={} positions",
        hex::encode(geometry_digest),
        geometry.identity(),
        hex::encode(parameter_digest),
        model.encode().len(),
        document.len()
    )?;

    let t0 = Instant::now();
    let mut session = RuntimeSession::new(model.id(), geometry, 41)?;
    let mut policy = SampledPolicy::new(&parameters, EVENT_SEED, 0, 0);
    let mut outputs: Vec<u16> = Vec::with_capacity(STEPS);
    let mut actuals: Vec<u16> = Vec::with_capacity(STEPS);
    let mut actions: Vec<Action> = Vec::with_capacity(STEPS);
    let mut reads: Vec<u8> = Vec::with_capacity(STEPS);
    let mut cursor_reads = 0usize;
    let mut exact_matches = 0usize;
    let mut eos_outputs = 0usize;
    let mut last_traces: Vec<Value> = Vec::with_capacity(LAST_TRACES);
    let mut turns = 1u64;
    let mut previous_actual: Option<Symbol> = None;
    for step in 0..STEPS {
        // The pilot document ends in terminal EOS. The engine ends the response
        // at EOS (engine.rs:476), so the next document starts with begin_turn()
        // exactly as forward_tests.rs:43 does mid-stream.
        if previous_actual == Some(Symbol::Eos) {
            session.begin_turn()?;
            turns += 1;
        }
        let target = document[step % document.len()];
        policy.set_target(usize::from(target), document.len())?;
        let offered = session.predict(geometry, &mut policy)?;
        if policy.target_pending() {
            return Err(format!("target not consumed at EMIT on step {step}").into());
        }
        outputs.push(symbol_code(offered.offer.symbol));
        actions.push(offered.trace.action);
        reads.push(offered.trace.selected.iter().flatten().count() as u8);
        cursor_reads += usize::from(offered.trace.cursor.is_some());
        eos_outputs += usize::from(offered.offer.symbol == Symbol::Eos);
        let actual = symbol_of(target)?;
        let completed = session.observe(actual, offered.offer.id, geometry, &mut policy)?;
        actuals.push(symbol_code(actual));
        exact_matches += usize::from(completed.actual == Some(completed.output));
        previous_actual = Some(actual);
        if step + LAST_TRACES >= STEPS {
            last_traces.push(trace_json(&completed));
        }
    }
    let drive_us = t0.elapsed().as_micros();
    let work = session.work();
    let counts = policy.counts();
    let steps_done = STEPS as u64;
    let counters = json!({
        "work": work_json(work),
        "advanced": {
            "circuit_calls": work.circuit_calls > 0,
            "candidate_scores": work.candidate_scores > 0,
            "predictions": work.predictions > 0,
            "observations": work.observations > 0,
            "scalar_decodes": work.scalar_decodes > 0,
            "selected_operations": work.selected_operations > 0,
        },
        "sampled_policy": {
            "circuit_calls": counts.circuit_calls,
            "gate_events": counts.gate_events,
            "head_events": counts.head_events,
            "scored_positions": counts.scored_positions,
            "mean_conditional_ce": policy.mean_loss() / steps_done as f64,
            "policy_counters_equal_runtime": counts.circuit_calls == work.circuit_calls,
        },
        "steps": steps_done,
        "turns": turns,
        "elapsed_us": drive_us,
        "us_per_step": drive_us as f64 / steps_done as f64,
        "domain": {
            "distinct_offered_symbols": outputs.iter().collect::<std::collections::BTreeSet<_>>().len(),
            "distinct_actual_symbols": actuals.iter().collect::<std::collections::BTreeSet<_>>().len(),
            "offer_equals_actual_rate": exact_matches as f64 / steps_done as f64,
            "eos_offered": eos_outputs,
            "steps_with_one_selected_read": reads.iter().filter(|&&r| r == 1).count(),
            "steps_with_two_selected_reads": reads.iter().filter(|&&r| r == 2).count(),
            "steps_with_no_selected_read": reads.iter().filter(|&&r| r == 0).count(),
            "steps_with_active_cursor": cursor_reads,
            "offered_symbol_histogram": counted(&outputs),
            "action_histogram": counted(&actions),
        },
        "last_traces": last_traces,
    });

    // ---- (c) the engine's own evaluation surface: pilot::evaluate on the
    // frozen development split, Full control plus the three declared controls.
    let development = pilot_data::development();
    let mut pilot_json = serde_json::Map::new();
    let t0 = Instant::now();
    for control in pilot::CONTROLS {
        let rows = pilot::evaluate(&parameters, model.compiled(), geometry, &development, control)?;
        let metrics = pilot::metrics(&rows);
        pilot_json.insert(
            format!("{control:?}"),
            json!({
                "metrics": metrics,
                "per_row": rows.iter().map(|r| json!({
                    "id": r.id,
                    "family": r.family,
                    "expected": r.expected,
                    "predictions": r.predictions,
                    "generated": r.generated,
                    "correct_symbols": r.correct_symbols,
                    "positions": r.positions,
                    "response_ce": r.response_ce,
                    "exact_response_and_eos": r.exact_response_and_eos,
                    "generated_eos": r.generated_eos,
                    "selected_reads": r.selected_reads,
                    "candidate_scores": r.candidate_scores,
                    "circuit_calls": r.circuit_calls,
                })).collect::<Vec<_>>(),
            }),
        );
    }
    let pilot_us = t0.elapsed().as_micros();
    let full = pilot::metrics(&pilot::evaluate(
        &parameters,
        model.compiled(),
        geometry,
        &development,
        Control::Full,
    )?);

    // ---- (d) the learner's own hard path bound, run once on one document.
    let learner_document: Vec<u16> = pilot::document(&development[0]);
    let t0 = Instant::now();
    let batch = learner::batch(&parameters, geometry, &learner_document, EVENT_SEED, 0);
    let learner_us = t0.elapsed().as_micros();
    let learner_json = match &batch {
        Ok(batch) => json!({
            "status": "OK",
            "positions": learner_document.len(),
            "mean_ce": batch.mean_ce,
            "counts": batch.counts,
            "gradient_len": batch.gradient.len(),
            "gradient_finite": batch.gradient.iter().all(|g| g.is_finite()),
            "gradient_nonzero": batch.gradient.iter().filter(|g| **g != 0.0).count(),
            "elapsed_us": batch.elapsed_us,
            "bound_checks": "learner::batch enforces circuit_calls<=8, candidate_scores<=530, scalar_decodes<=2, selected_operations<=1 per position",
        }),
        Err(error) => json!({"status": "ERROR", "error": error.to_string()}),
    };

    let report = json!({
        "schema": "uor-r4.addressed-attention-dispatch-probe/1",
        "elapsed_ms_at_report": started.elapsed().as_millis(),
        "construction": {
            "path": [
                "BoundGeometry::canonical() -> artifact.rs:56",
                "Parameters::seeded(7341) -> policy.rs:40",
                "parameters.compile() -> policy.rs:78",
                "Model::new(compiled, Provenance{..}) -> artifact.rs:250",
                "RuntimeSession::new(model.id(), model.geometry(), 41) -> engine.rs:149",
                "SampledPolicy::new(&parameters, 973, 0, 0) -> policy.rs:214",
                "session.predict(geometry, &mut policy) -> engine.rs:348",
                "session.observe(actual, offer.id, geometry, &mut policy) -> engine.rs:441",
            ],
            "geometry_identity": geometry.identity(),
            "geometry_digest": hex::encode(geometry_digest),
            "parameter_seed": parameters.seed(),
            "parameter_count": parameters.values().len(),
            "parameter_digest": hex::encode(parameter_digest),
            "model_bytes": model.encode().len(),
            "model_id": hex::encode(model.id()),
            "elapsed_ms": constructed_ms,
            "fitted": false,
            "fitted_note": "Parameters::seeded is the documented unfitted uniform[-.25,.25) initialization (policy.rs:39), so symbol accuracy here is a mechanics signal, not a quality claim",
        },
        "drive": counters,
        "pilot_evaluation": {
            "development_examples": development.len(),
            "elapsed_us": pilot_us,
            "arms": pilot_json,
            "full_metrics": full,
        },
        "learner_bound_check": learner_json,
        "learner_elapsed_us": learner_us,
        "elapsed_ms_total": started.elapsed().as_millis(),
        "caveats": [
            "Parameter values are the unfitted seed-7341 initialization; no fit, no SGD and no checkpoint were loaded.",
            "The pilot rows are the authored development split, not an independent holdout.",
            "This probe reads no trained artifact and changes no production path.",
        ],
    });
    std::fs::write(root.join("dispatch.json"), serde_json::to_vec_pretty(&report)?)?;

    writeln!(Tee(log), "[aa-dispatch] construction_ms={constructed_ms}")?;
    writeln!(Tee(log), "[aa-dispatch] work={}", serde_json::to_string(&work_json(work))?)?;
    writeln!(
        Tee(log),
        "[aa-dispatch] sampled_policy circuit_calls={} gate_events={} head_events={} scored_positions={} mean_ce={}",
        counts.circuit_calls,
        counts.gate_events,
        counts.head_events,
        counts.scored_positions,
        policy.mean_loss() / steps_done as f64
    )?;
    writeln!(
        Tee(log),
        "[aa-dispatch] domain distinct_offered={} steps_with_read={} two_reads={} cursor={} eos_offered={} offer_equals_actual={:.4}",
        outputs.iter().collect::<std::collections::BTreeSet<_>>().len(),
        reads.iter().filter(|&&r| r > 0).count(),
        reads.iter().filter(|&&r| r == 2).count(),
        cursor_reads,
        eos_outputs,
        exact_matches as f64 / steps_done as f64
    )?;
    writeln!(
        Tee(log),
        "[aa-dispatch] pilot Full: examples={} positions={} correct_symbols={} exact_responses={} mean_response_ce={:.6} generated_eos={}",
        full.examples, full.positions, full.correct_symbols, full.exact_responses, full.mean_response_ce, full.generated_eos
    )?;
    for control in pilot::CONTROLS {
        if let Some(arm) = report["pilot_evaluation"]["arms"].get(format!("{control:?}")) {
            let m = &arm["metrics"];
            writeln!(
                Tee(log),
                "[aa-dispatch] pilot {control:?}: correct_symbols={} exact_responses={} mean_response_ce={:.6}",
                m["correct_symbols"], m["exact_responses"], m["mean_response_ce"]
            )?;
        }
    }
    writeln!(
        Tee(log),
        "[aa-dispatch] learner_bound_check={}",
        serde_json::to_string(&learner_json)?
    )?;
    writeln!(
        Tee(log),
        "[aa-dispatch] drive_us={drive_us} pilot_us={pilot_us} learner_us={learner_us} total_ms={}",
        started.elapsed().as_millis()
    )?;
    Ok(())
}
