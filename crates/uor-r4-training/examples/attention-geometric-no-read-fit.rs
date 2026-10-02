//! Fixed-dose answer-only NoRead learning from admitted query-seed producers.
//! Separate executable: the existing no-fit comparator and its CLI are unchanged.
use candle_core::{Device, Tensor};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use uor_r4_core::report_output;
use uor_r4_training::geometric_context::{CompiledContext, ContextSourcePaths};
use uor_r4_training::geometric_event::CompiledEvents;
use uor_r4_training::geometric_no_read::NoReadWeights;
use uor_r4_training::geometric_no_read_native::{CompiledNoRead, NoReadSourcePaths};
use uor_r4_training::geometric_potential_native::{
    CompiledGeometricPotentials, PotentialSourceBinding,
};
use uor_r4_training::geometric_read_native::{
    CompiledGeometricRead, NativeReadTrace, ReadSourceBinding,
};
use uor_r4_training::geometric_span_native::{CompiledSpanActions, SpanSourceBinding};
use uor_r4_training::geometric_stack::StackModel;
use uor_r4_training::geometric_value_producer_native::{
    CompiledValueProducer, ValueProducerSourcePaths,
};
use uor_r4_training::{sha256_bytes, Result};
#[path = "attention-geometric-value-learned/data.rs"]
mod data;
use data::Episode;
#[path = "attention-geometric-no-read-fit/fit.rs"]
mod fit;

fn invalid(message: impl Into<String>) -> uor_r4_training::TrainingError {
    uor_r4_training::TrainingError::Invalid(message.into())
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Args {
    model: PathBuf,
    parent: PathBuf,
    event_source: PathBuf,
    event_native: PathBuf,
    span_native: PathBuf,
    potential_native: PathBuf,
    out: PathBuf,
    rows_per_panel: usize,
    maximum_seconds: u64,
    initial_comparison: PathBuf,
    seed: u64,
    mode: fit::Mode,
}
impl Args {
    fn dependencies(&self) -> ContextSourcePaths<'_> {
        ContextSourcePaths {
            base: &self.model,
            event_source: &self.event_source,
            event_native: &self.event_native,
            span_native: &self.span_native,
            potential_native: &self.potential_native,
        }
    }
}
fn write(path: &Path, value: &impl serde::Serialize) -> Result<()> {
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
fn answer(logits: &Tensor, query: usize) -> Result<(u32, Vec<f32>)> {
    let all = logits.to_vec2::<f32>()?;
    let row = all
        .get(query)
        .ok_or_else(|| invalid("NoRead comparator answer position absent"))?;
    if row.is_empty() || row.iter().any(|x| !x.is_finite()) {
        return Err(invalid("NoRead comparator nonfinite/empty logits"));
    }
    let mut best = 0;
    for i in 1..row.len() {
        if row[i] > row[best] {
            best = i;
        }
    }
    Ok((best as u32, row.clone()))
}
fn query_trace(trace: &NativeReadTrace, query: usize) -> Result<Value> {
    if trace.batch != 1 || query >= trace.time {
        return Err(invalid("query trace layout differs"));
    }
    let mut heads = Vec::new();
    for h in 0..trace.heads {
        let at = h * trace.time + query;
        let row = trace
            .rows
            .get(at)
            .ok_or_else(|| invalid("query trace row absent"))?;
        heads.push(json!({"no_read_q24":trace.no_read_q24[at],
            "no_read_weight_q31":row.no_read_weight_q31,
            "occurrence_weights_q31":row.occurrence_weights_q31,
            "total_weight_q31":row.total_weight_q31,"max_score_q24":row.max_score_q24,
            "output_q16":row.output_q16}));
    }
    Ok(json!(heads))
}

fn variable_hashes(model: &StackModel) -> Result<BTreeMap<String, String>> {
    model
        .variables()
        .iter()
        .map(|(name, var)| {
            let bytes = var
                .flatten_all()?
                .to_vec1::<f32>()?
                .iter()
                .flat_map(|x| x.to_bits().to_le_bytes())
                .collect::<Vec<_>>();
            Ok((name.clone(), sha256_bytes(&bytes)))
        })
        .collect()
}
fn file_snapshot(roots: &[(&str, &Path)]) -> Result<BTreeMap<String, String>> {
    fn visit(
        root: &Path,
        path: &Path,
        label: &str,
        out: &mut BTreeMap<String, String>,
    ) -> Result<()> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            if kind.is_dir() {
                visit(root, &entry.path(), label, out)?;
            } else if kind.is_file() {
                let path = entry.path();
                let relative = path
                    .strip_prefix(root)
                    .map_err(|_| invalid("dependency snapshot path"))?;
                out.insert(
                    format!("{label}/{}", relative.display()),
                    sha256_bytes(&fs::read(path)?),
                );
            } else {
                return Err(invalid(
                    "dependency snapshot requires regular files/directories",
                ));
            }
        }
        Ok(())
    }
    let mut out = BTreeMap::new();
    for (label, path) in roots {
        visit(path, path, label, &mut out)?;
    }
    Ok(out)
}
fn independent_geometry_ablation(source: &Path) -> Result<NoReadWeights> {
    let copy = NoReadWeights::load(source)?;
    let cfg = *copy.config();
    let mut values = fit::shadows(&copy)?;
    for h in 0..cfg.heads {
        values[h * cfg.coefficients_per_head() + 1 + cfg.vocabulary
            ..(h + 1) * cfg.coefficients_per_head()]
            .fill(0.);
    }
    copy.parameters()
        .get("coefficients")
        .ok_or_else(|| invalid("ablation coefficients missing"))?
        .set(&Tensor::from_vec(
            values,
            (cfg.heads, cfg.coefficients_per_head()),
            &Device::Cpu,
        )?)?;
    Ok(copy)
}
fn main() -> Result<()> {
    let mut cli = std::env::args().skip(1);
    let path = cli
        .next()
        .ok_or_else(|| invalid("usage: attention-geometric-no-read-fit ARGS.json"))?;
    if cli.next().is_some() {
        return Err(invalid("exactly one configuration path required"));
    }
    let argument_bytes = fs::read(path)?;
    let args: Args = serde_json::from_slice(&argument_bytes)?;
    if ![1, 2].contains(&args.seed)
        || !(301..=900).contains(&args.maximum_seconds)
        || !(1..=128).contains(&args.rows_per_panel)
        || (args.mode == fit::Mode::Fit && args.rows_per_panel != 128)
    {
        return Err(invalid(
            "seed1/2; worker301..900s; fit requires all128 rows per panel",
        ));
    }
    report_output::claim(&args.out)?;
    let started = Instant::now();
    fs::write(args.out.join("arguments.json"), &argument_bytes)?;
    let result = run(&args, started);
    if let Err(error) = &result {
        write(
            &args.out.join("failure.json"),
            &json!({"error":error.to_string(),"elapsed_seconds":started.elapsed().as_secs_f64(),
            "decision":"INCOMPLETE_ATTEMPT_RETAIN_CHECKPOINTS_NO_QUALITY_VERDICT"}),
        )?;
    }
    report_output::seal(&args.out)?;
    report_output::verify(&args.out)?;
    result
}
fn run(a: &Args, started: Instant) -> Result<()> {
    report_output::verify(&a.parent)?;
    report_output::verify(&a.initial_comparison)?;
    let parent_report_bytes = fs::read(a.parent.join("report.json"))?;
    let parent_report: Value = serde_json::from_slice(&parent_report_bytes)?;
    if parent_report["complete"] != true
        || parent_report["completed_updates"] != 640
        || parent_report["cumulative_updates"] != 1280
        || parent_report["auxiliary_credit"] != "query_read"
    {
        return Err(invalid("completed query-credit parent required"));
    }
    let checkpoint_bytes = fs::read(a.parent.join("trained/checkpoint.json"))?;
    let checkpoint: Value = serde_json::from_slice(&checkpoint_bytes)?;
    let rng = checkpoint["next_generator_state"]
        .as_u64()
        .ok_or_else(|| invalid("exact u64 generator state missing"))?;
    let expected_rng = if a.seed == 1 {
        12355479550239716776u64
    } else {
        11629321088715907971u64
    };
    if checkpoint["completed_updates"] != 1280
        || checkpoint["args"]["seed"] != a.seed
        || rng != expected_rng
    {
        return Err(invalid(
            "saved query-parent seed/update/RNG identity differs",
        ));
    }
    let initial_report_bytes = fs::read(a.initial_comparison.join("report.json"))?;
    let initial_report: Value = serde_json::from_slice(&initial_report_bytes)?;
    if initial_report["complete"] != true
        || initial_report["updates"] != 0
        || initial_report["initialization"] != "all-zero-q4"
        || initial_report["parent_report_sha256"] != sha256_bytes(&parent_report_bytes)
    {
        return Err(invalid(
            "complete zero-init construction from this parent required",
        ));
    }
    let model = StackModel::load(&a.model, &Device::Cpu)?;
    if model.config.width != 32
        || model.config.heads != 2
        || model.config.vocab_size != 40
        || model.config.context != 128
    {
        return Err(invalid(
            "accepted width32/H2/V40/context128 parent required",
        ));
    }
    let tokenizer = fs::read(a.span_native.join("tokenizer-identity.bin"))?;
    let context_source = a.parent.join("trained/context-source");
    let context_native = a.parent.join("trained/native-context");
    let value_source = a.parent.join("trained/value-source");
    let value_native = a.parent.join("trained/native-values");
    let reducer_native = a.parent.join("native-read");
    let context = CompiledContext::load(
        &context_native,
        &context_source,
        a.dependencies(),
        &tokenizer,
    )?;
    let events = CompiledEvents::load(&a.event_native, &a.event_source, &a.model, &tokenizer)?;
    let span = CompiledSpanActions::load(
        &a.span_native,
        &SpanSourceBinding::from_files(
            &a.model.join("model.safetensors"),
            &a.model.join("config.json"),
            &tokenizer,
        )?,
        model
            .geometric_span()
            .ok_or_else(|| invalid("geometric span missing"))?,
    )?;
    let potential = CompiledGeometricPotentials::load(
        &a.potential_native,
        &PotentialSourceBinding::from_directory(&a.model, &tokenizer)?,
    )?;
    let reducer = CompiledGeometricRead::load(
        &reducer_native,
        &ReadSourceBinding::from_directory(&a.model, &tokenizer, &potential, 2)?,
    )?;
    let value_paths = ValueProducerSourcePaths {
        value_source: &value_source,
        context_source: &context_source,
        context_dependencies: a.dependencies(),
    };
    let values = CompiledValueProducer::load(&value_native, value_paths, &tokenizer)?;
    let initial_source = a.initial_comparison.join("no-read-source");
    let weights = NoReadWeights::load(&initial_source)?;
    if weights.config().vocabulary != 40
        || weights.config().heads != 2
        || weights.config().latent_lanes_per_head != 4
        || fit::shadows(&weights)?.iter().any(|x| *x != 0.)
    {
        return Err(invalid(
            "fixed all-zero 754-coefficient NoRead initialization required",
        ));
    }
    macro_rules! no_read_paths {
        ($source:expr) => {
            NoReadSourcePaths {
                no_read_source: $source,
                value: value_paths,
                context_native: &context_native,
                value_native: &value_native,
                reducer_native: &reducer_native,
            }
        };
    }
    let initial_compiled = CompiledNoRead::load(
        &a.initial_comparison.join("no-read-native"),
        no_read_paths!(&initial_source),
        &tokenizer,
    )?;
    initial_compiled.validate_for(&weights)?;
    if serde_json::to_value(initial_compiled.metadata())? != initial_report["no_read_metadata"] {
        return Err(invalid(
            "initial construction metadata does not match actual admitted artifact",
        ));
    }
    weights.save(&a.out.join("initial-source"))?;
    struct Panel {
        name: &'static str,
        episodes: Vec<Episode>,
        parent_rows: Vec<Value>,
        initial_rows: Vec<Value>,
        input_hash: String,
        parent_rows_hash: String,
        initial_rows_hash: String,
    }
    let mut inputs = Vec::new();
    for name in ["original", "stress"] {
        let bytes = fs::read(a.parent.join(if name == "original" {
            "evaluation.json"
        } else {
            "stress.json"
        }))?;
        let episodes: Vec<Episode> = serde_json::from_slice(&bytes)?;
        let parent_bytes = fs::read(a.parent.join(name).join("rows.json"))?;
        let parent_rows: Vec<Value> = serde_json::from_slice(&parent_bytes)?;
        let initial_bytes = fs::read(a.initial_comparison.join(format!("{name}-rows.json")))?;
        let initial_rows: Vec<Value> = serde_json::from_slice(&initial_bytes)?;
        if episodes.len() != 128 || parent_rows.len() != 128 || initial_rows.len() != 128 {
            return Err(invalid(
                "fixed128-row parent and construction panels required",
            ));
        }
        for (index, e) in episodes.iter().enumerate() {
            let encoded = serde_json::to_value(e)?;
            if e.ids.is_empty()
                || e.ids.len() > 128
                || e.query + 1 != e.ids.len()
                || parent_rows[index]["episode"] != encoded
                || initial_rows[index]["episode"] != encoded
                || initial_rows[index]["parent_prediction"]
                    != parent_rows[index]["native_prediction"]
                || initial_rows[index]["baseline_prediction"]
                    != parent_rows[index]["native_prediction"]
            {
                return Err(invalid(
                    "rowwise initial/parent episode or legacy prediction identity differs",
                ));
            }
        }
        fs::write(a.out.join(format!("{name}-episodes.json")), &bytes)?;
        fs::write(
            a.out.join(format!("initial-{name}-rows.json")),
            &initial_bytes,
        )?;
        inputs.push(Panel {
            name,
            episodes,
            parent_rows,
            initial_rows,
            input_hash: sha256_bytes(&bytes),
            parent_rows_hash: sha256_bytes(&parent_bytes),
            initial_rows_hash: sha256_bytes(&initial_bytes),
        });
    }
    let frozen_roots = [
        ("base", a.model.as_path()),
        ("context-source", context_source.as_path()),
        ("context-native", context_native.as_path()),
        ("value-source", value_source.as_path()),
        ("value-native", value_native.as_path()),
        ("event-source", a.event_source.as_path()),
        ("event-native", a.event_native.as_path()),
        ("span-native", a.span_native.as_path()),
        ("potential-native", a.potential_native.as_path()),
        ("reducer-native", reducer_native.as_path()),
    ];
    let frozen_files = file_snapshot(&frozen_roots)?;
    let frozen_variables = variable_hashes(&model)?;
    let parent_identity = json!({"parent_report_sha256":sha256_bytes(&parent_report_bytes),
        "parent_checkpoint_sha256":sha256_bytes(&checkpoint_bytes),"initial_report_sha256":sha256_bytes(&initial_report_bytes),
        "initial_no_read_metadata":initial_compiled.metadata(),"frozen_files":frozen_files,
        "frozen_base_variables":frozen_variables,"seed":a.seed});
    write(&a.out.join("input-identities.json"), &parent_identity)?;
    let preparation_seconds = started.elapsed().as_secs_f64();
    let progress = fit::run(
        &a.out,
        &weights,
        rng,
        a.mode,
        started + Duration::from_secs(a.maximum_seconds - 300),
        &parent_identity,
        |ids, batch, time, scalar| {
            model.forward_geometric_no_read(
                ids, batch, time, &context, &events, &span, &potential, &reducer, &values, scalar,
                false,
            )
        },
    )?;
    let compile_started = Instant::now();
    let saved_source = a.out.join("no-read-source");
    let trained_shadows = fit::shadows(&weights)?;
    weights.save(&saved_source)?;
    let weights = NoReadWeights::load(&saved_source)?;
    if fit::shadows(&weights)? != trained_shadows {
        return Err(invalid("independent scalar reload changed parameters"));
    }
    let compiled_directory = a.out.join("no-read-native");
    CompiledNoRead::compile(&weights, no_read_paths!(&saved_source), &tokenizer)?
        .save(&compiled_directory)?;
    let compiled = CompiledNoRead::load(
        &compiled_directory,
        no_read_paths!(&saved_source),
        &tokenizer,
    )?;
    compiled.validate_for(&weights)?;
    // Independent source reload, not a shared Var clone. Keep bias/token terms.
    let ablated = independent_geometry_ablation(&saved_source)?;
    let ablated_source = a.out.join("geometry-ablated-source");
    ablated.save(&ablated_source)?;
    let ablated_directory = a.out.join("geometry-ablated-native");
    CompiledNoRead::compile(&ablated, no_read_paths!(&ablated_source), &tokenizer)?
        .save(&ablated_directory)?;
    let ablated_native = CompiledNoRead::load(
        &ablated_directory,
        no_read_paths!(&ablated_source),
        &tokenizer,
    )?;
    ablated_native.validate_for(&ablated)?;
    if fit::shadows(&weights)? != trained_shadows {
        return Err(invalid("ablation modified trained scalar"));
    }
    let compilation_seconds = compile_started.elapsed().as_secs_f64();
    let evaluate_started = Instant::now();
    let mut panels = Vec::new();
    let mut evaluation_complete = true;
    let mut final_gradient = Value::Null;
    for panel in inputs {
        let mut rows = Vec::new();
        let mut counts = [0usize; 5]; // legacy, initial, source, native, geometry-ablated
        let mut source_native_changes = 0;
        let mut legacy_changes = 0;
        let mut initial_gains = 0;
        let mut initial_losses = 0;
        let mut parent_gains = 0;
        let mut parent_losses = 0;
        let mut ablation_changes = 0;
        let mut maximum_logit_delta = 0f32;
        for (index, e) in panel.episodes.iter().take(a.rows_per_panel).enumerate() {
            if started.elapsed().as_secs() >= a.maximum_seconds {
                evaluation_complete = false;
                break;
            }
            let t = e.ids.len();
            let (legacy, legacy_read, _) = model
                .forward_geometric_context_learned_values_native_with_trace(
                    &e.ids, 1, t, &context, &events, &span, &potential, &reducer, &values, false,
                )?;
            let (source, source_null) = model.forward_geometric_no_read(
                &e.ids, 1, t, &context, &events, &span, &potential, &reducer, &values, &weights,
                false,
            )?;
            if panel.name == "original" && index == 0 {
                let target = Tensor::from_vec(vec![e.answer], 1, &Device::Cpu)?;
                let loss = candle_nn::loss::cross_entropy(&source.narrow(0, e.query, 1)?, &target)?;
                let gradients = loss.backward()?;
                let var = weights
                    .parameters()
                    .get("coefficients")
                    .ok_or_else(|| invalid("NoRead coefficient missing"))?;
                let grad = gradients
                    .get(var.as_tensor())
                    .ok_or_else(|| invalid("final NoRead answer gradient disconnected"))?
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                final_gradient = json!({"answer_mean_ce":loss.to_scalar::<f32>()?,"per_head_family_l2":fit::family_l2(&weights,&grad)?,
                    "scope":"one recorded actual row; no positivity threshold"});
            }
            let (native, native_read, _) = model.forward_geometric_no_read_native_with_trace(
                &e.ids, 1, t, &context, &events, &span, &potential, &reducer, &values, &compiled,
                false,
            )?;
            let (ablated_logits, ablated_read, _) = model
                .forward_geometric_no_read_native_with_trace(
                    &e.ids,
                    1,
                    t,
                    &context,
                    &events,
                    &span,
                    &potential,
                    &reducer,
                    &values,
                    &ablated_native,
                    false,
                )?;
            let (bp, bl) = answer(&legacy, e.query)?;
            let (sp, sl) = answer(&source, e.query)?;
            let (np, nl) = answer(&native, e.query)?;
            let (ap, al) = answer(&ablated_logits, e.query)?;
            let previous = &panel.parent_rows[index];
            let initial = &panel.initial_rows[index];
            let parent_prediction = previous["native_prediction"]
                .as_u64()
                .ok_or_else(|| invalid("parent prediction missing"))?
                as u32;
            let initial_prediction = initial["native_prediction"]
                .as_u64()
                .ok_or_else(|| invalid("initial prediction missing"))?
                as u32;
            // The actual legacy query vector and full normalized-read receipt are a
            // strict same-producer control, not merely an aggregate score check.
            let legacy_query = query_trace(&legacy_read, e.query)?;
            if json!(bl) != initial["answer_logits"]["baseline"]
                || legacy_query != initial["baseline_query_read"]
            {
                return Err(invalid(
                    "loaded frozen legacy logits/read do not match construction control",
                ));
            }
            legacy_changes += usize::from(bp != parent_prediction);
            source_native_changes += usize::from(sp != np);
            ablation_changes += usize::from(ap != np);
            initial_gains += usize::from(np == e.answer && initial_prediction != e.answer);
            initial_losses += usize::from(np != e.answer && initial_prediction == e.answer);
            parent_gains += usize::from(np == e.answer && parent_prediction != e.answer);
            parent_losses += usize::from(np != e.answer && parent_prediction == e.answer);
            for (count, prediction) in counts.iter_mut().zip([bp, initial_prediction, sp, np, ap]) {
                *count += usize::from(prediction == e.answer);
            }
            let sf = source.flatten_all()?.to_vec1::<f32>()?;
            let nf = native.flatten_all()?.to_vec1::<f32>()?;
            if sf.len() != nf.len() {
                return Err(invalid("source/native logit shape differs"));
            }
            for (x, y) in sf.iter().zip(&nf) {
                if !x.is_finite() || !y.is_finite() {
                    return Err(invalid("nonfinite evaluation logits"));
                }
                maximum_logit_delta = maximum_logit_delta.max((x - y).abs());
            }
            rows.push(json!({"index":index,"episode":e,"parent_row":previous,"initial_row":initial,
                "baseline_prediction":bp,"source_prediction":sp,"native_prediction":np,"geometry_ablated_prediction":ap,
                "answer_logits":{"baseline":bl,"source":sl,"native":nl,"geometry_ablated":al},
                "source_null_nats":source_null.to_vec3::<f32>()?,"native_null_q24_all_positions":native_read.no_read_q24,
                "baseline_query_read":legacy_query,"native_query_read":query_trace(&native_read,e.query)?,
                "geometry_ablated_query_read":query_trace(&ablated_read,e.query)?}));
        }
        write(&a.out.join(format!("{}-rows.json", panel.name)), &rows)?;
        panels.push(json!({"panel":panel.name,"scored_rows":rows.len(),"requested_rows":a.rows_per_panel,
            "answers":{"legacy":counts[0],"initial_zero":counts[1],"source":counts[2],"native":counts[3],"geometry_ablated":counts[4]},
            "source_native_answer_changes":source_native_changes,"legacy_changed_from_parent":legacy_changes,
            "native_gains_from_initial":initial_gains,"native_losses_from_initial":initial_losses,
            "native_gains_from_parent":parent_gains,"native_losses_from_parent":parent_losses,
            "geometry_ablation_prediction_changes":ablation_changes,"maximum_actual_position_logit_delta":maximum_logit_delta,
            "input_sha256":panel.input_hash,"parent_rows_sha256":panel.parent_rows_hash,"initial_rows_sha256":panel.initial_rows_hash}));
        if !evaluation_complete {
            break;
        }
    }
    let evaluation_seconds = evaluate_started.elapsed().as_secs_f64();
    if variable_hashes(&model)? != frozen_variables || file_snapshot(&frozen_roots)? != frozen_files
    {
        return Err(invalid("frozen model/producer/source identity changed"));
    }
    let complete = progress.completed && evaluation_complete;
    write(
        &a.out.join("report.json"),
        &json!({"schema":"uor-r4.geometric-no-read-answer-fit/1",
        "complete":complete,"mode":a.mode,"seed":a.seed,"fit":progress,"frozen_producer_updates":1280,
        "trainable_inventory":["no_read.coefficients[2,377]"],"primary_objective":"mean CE of one answer per episode; no auxiliary",
        "optimizer":{"rate":fit::RATE,"beta1":0.9,"beta2":0.95,"epsilon":1e-8,"weight_decay":0,"clip_selected_global_norm":1},
        "panels":panels,"actual_final_answer_gradient":final_gradient,"no_read_metadata":compiled.metadata(),
        "parent":parent_identity,"preparation_seconds":preparation_seconds,"compilation_reload_seconds":compilation_seconds,
        "evaluation_seconds":evaluation_seconds,"elapsed_seconds":started.elapsed().as_secs_f64(),
        "decision":if !complete {"PARTIAL_RETAIN_NO_QUALITY_VERDICT"} else if a.mode==fit::Mode::Check {"ACTUAL_PARENT_ANSWER_BACKWARD_ONLY_NO_UPDATES"} else {"FIXED_ANSWER_ONLY_SCALAR_FIT_RETAIN_ALL_ROWS"},
        "scope":"Frozen producer1280; NoRead0-to640 separately. Fixed q4 quarter-nat grid; source real softmax versus native Q24/LUT mix. Current grammar always has a matching record and common query token: no semantic-abstention or general history-use claim. Geometry ablation changes scalar features only and is diagnostic, not an adoption gate. Other learned wide tables and float trunk/output remain."}),
    )?;
    println!(
        "NoRead answer fit complete={complete}; updates={}; {}",
        progress.completed_updates,
        a.out.display()
    );
    Ok(())
}
