//! Fixed-dose answer-only geometric composition learning; separate opt-in driver.
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
use uor_r4_training::{
    geometric_composition::CompositionWeights,
    geometric_composition_native::{CompiledComposition, CompositionSourcePaths},
    geometric_context::{CompiledContext, ContextSourcePaths},
    geometric_event::CompiledEvents,
    geometric_no_read_native::{CompiledNoRead, NoReadSourcePaths},
    geometric_potential_native::{CompiledGeometricPotentials, PotentialSourceBinding},
    geometric_read_native::{CompiledGeometricRead, ReadSourceBinding},
    geometric_span_native::{CompiledSpanActions, SpanSourceBinding},
    geometric_stack::StackModel,
    geometric_value_producer_native::{CompiledValueProducer, ValueProducerSourcePaths},
    sha256_bytes, Result,
};
#[path = "attention-geometric-value-learned/data.rs"]
mod data;
#[path = "attention-geometric-composition-fit/fit.rs"]
mod fit;
fn invalid(m: impl Into<String>) -> uor_r4_training::TrainingError {
    uor_r4_training::TrainingError::Invalid(m.into())
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
    no_read_fit: PathBuf,
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
fn write(p: &Path, v: &impl serde::Serialize) -> Result<()> {
    fs::write(p, serde_json::to_vec_pretty(v)?)?;
    Ok(())
}
fn answer(t: &Tensor, q: usize) -> Result<(usize, Vec<f32>)> {
    let rows = t.to_vec2::<f32>()?;
    let row = rows
        .get(q)
        .ok_or_else(|| invalid("answer position missing"))?;
    if row.is_empty() || row.iter().any(|v| !v.is_finite()) {
        return Err(invalid("nonfinite composition answer logits"));
    }
    let mut best = 0;
    for i in 1..row.len() {
        if row[i] > row[best] {
            best = i;
        }
    }
    Ok((best, row.clone()))
}
fn checked_logits(t: &Tensor, time: usize, vocabulary: usize) -> Result<Vec<Vec<f32>>> {
    if t.dims() != [time, vocabulary] {
        return Err(invalid("composition logits shape differs"));
    }
    let rows = t.to_vec2::<f32>()?;
    if rows.iter().flatten().any(|v| !v.is_finite()) {
        return Err(invalid("nonfinite composition prefix logits"));
    }
    Ok(rows)
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

fn main() -> Result<()> {
    let mut cli = std::env::args().skip(1);
    let path = cli
        .next()
        .ok_or_else(|| invalid("usage: attention-geometric-composition-fit ARGS.json"))?;
    if cli.next().is_some() {
        return Err(invalid("one argument file required"));
    }
    let bytes = fs::read(path)?;
    let a: Args = serde_json::from_slice(&bytes)?;
    // The root's prospective card must admit the explicit requested limit.
    // This format ceiling does not authorize a fit or an allowance extension.
    if ![1, 2].contains(&a.seed)
        || !(301..=7200).contains(&a.maximum_seconds)
        || !(1..=128).contains(&a.rows_per_panel)
        || (a.mode == fit::Mode::Fit && a.rows_per_panel != 128)
    {
        return Err(invalid(
            "seed1/2, explicit301..7200s, rows1..128; fit requires128 rows per panel",
        ));
    }
    report_output::claim(&a.out)?;
    let started = Instant::now();
    fs::write(a.out.join("arguments.json"), bytes)?;
    let result = run(&a, started);
    if let Err(error) = &result {
        write(
            &a.out.join("failure.json"),
            &json!({
                "error":error.to_string(),"elapsed_seconds":started.elapsed().as_secs_f64(),
                "decision":"INCOMPLETE_ATTEMPT_RETAIN_CHECKPOINTS_AND_ROWS_NO_QUALITY_VERDICT",
            }),
        )?;
    }
    report_output::seal(&a.out)?;
    report_output::verify(&a.out)?;
    result
}
fn run(a: &Args, started: Instant) -> Result<()> {
    for root in [&a.parent, &a.no_read_fit, &a.initial_comparison] {
        report_output::verify(root)?;
    }
    let parent_bytes = fs::read(a.parent.join("report.json"))?;
    let parent: Value = serde_json::from_slice(&parent_bytes)?;
    let scalar_bytes = fs::read(a.no_read_fit.join("report.json"))?;
    let scalar: Value = serde_json::from_slice(&scalar_bytes)?;
    let initial_bytes = fs::read(a.initial_comparison.join("report.json"))?;
    let initial: Value = serde_json::from_slice(&initial_bytes)?;
    let scalar_hash = sha256_bytes(&scalar_bytes);
    let expected_scalar = if a.seed == 1 {
        "bfa6475445461ac4d783df5a14c8ed3d75bb27ce555970b489a2245eed21598d"
    } else {
        "642c030a274b298d46e5b936b7d476f901292bd98975678e79d733af5136c62b"
    };
    if parent["complete"] != true
        || parent["completed_updates"] != 640
        || parent["cumulative_updates"] != 1280
        || parent["auxiliary_credit"] != "query_read"
        || scalar["complete"] != true
        || scalar["mode"] != "fit"
        || scalar["seed"] != a.seed
        || scalar["fit"]["completed_updates"] != 640
        || scalar["frozen_producer_updates"] != 1280
        || scalar_hash != expected_scalar
        || scalar["parent"]["parent_report_sha256"] != sha256_bytes(&parent_bytes)
        || initial["schema"] != "uor-r4.geometric-composition-construction/1"
        || initial["complete"] != true
        || initial["optimizer_updates"] != 0
        || initial["query_parent_report_sha256"] != sha256_bytes(&parent_bytes)
        || initial["no_read_fit_report_sha256"] != scalar_hash
    {
        return Err(invalid(
            "actual accepted query1280/scalar640/construction lineage differs",
        ));
    }
    let checkpoint_bytes = fs::read(a.no_read_fit.join("checkpoints/step-0640/checkpoint.json"))?;
    let checkpoint: Value = serde_json::from_slice(&checkpoint_bytes)?;
    let rng = checkpoint["next_generator_state"]
        .as_u64()
        .ok_or_else(|| invalid("exact scalar-final u64 RNG missing"))?;
    let expected_rng = if a.seed == 1 {
        9347350181718662555u64
    } else {
        10697677911788535647u64
    };
    if checkpoint["no_read_completed_updates"] != 640
        || checkpoint["next_generator_absolute_step"] != 1920
        || checkpoint["frozen_producer_updates"] != 1280
        || scalar["fit"]["next_generator_state"].as_u64() != Some(rng)
        || rng != expected_rng
    {
        return Err(invalid("scalar-final RNG/update identity differs"));
    }
    for name in uor_r4_training::geometric_no_read::SOURCE_FILES {
        if fs::read(a.no_read_fit.join("no-read-source").join(name))?
            != fs::read(
                a.no_read_fit
                    .join("checkpoints/step-0640/no-read-source")
                    .join(name),
            )?
        {
            return Err(invalid("scalar checkpoint and final source differ"));
        }
    }
    let model = StackModel::load(&a.model, &Device::Cpu)?;
    if model.config.width != 32
        || model.config.heads != 2
        || model.config.context != 128
        || model.config.vocab_size != 40
    {
        return Err(invalid(
            "accepted H2/width32/context128/vocabulary40 required",
        ));
    }
    let tokenizer = fs::read(a.span_native.join("tokenizer-identity.bin"))?;
    let cs = a.parent.join("trained/context-source");
    let cn = a.parent.join("trained/native-context");
    let vs = a.parent.join("trained/value-source");
    let vn = a.parent.join("trained/native-values");
    let rn = a.parent.join("native-read");
    let ns = a.no_read_fit.join("no-read-source");
    let nn = a.no_read_fit.join("no-read-native");
    let context = CompiledContext::load(&cn, &cs, a.dependencies(), &tokenizer)?;
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
            .ok_or_else(|| invalid("parent spans missing"))?,
    )?;
    let potential = CompiledGeometricPotentials::load(
        &a.potential_native,
        &PotentialSourceBinding::from_directory(&a.model, &tokenizer)?,
    )?;
    let reducer = CompiledGeometricRead::load(
        &rn,
        &ReadSourceBinding::from_directory(&a.model, &tokenizer, &potential, 2)?,
    )?;
    let vp = ValueProducerSourcePaths {
        value_source: &vs,
        context_source: &cs,
        context_dependencies: a.dependencies(),
    };
    let values = CompiledValueProducer::load(&vn, vp, &tokenizer)?;
    let np = NoReadSourcePaths {
        no_read_source: &ns,
        value: vp,
        context_native: &cn,
        value_native: &vn,
        reducer_native: &rn,
    };
    let no_read = CompiledNoRead::load(&nn, np, &tokenizer)?;
    if scalar["no_read_metadata"] != serde_json::to_value(no_read.metadata())? {
        return Err(invalid(
            "scalar retained report differs from actual artifact",
        ));
    }
    macro_rules! composition_paths {
        ($source:expr) => {
            CompositionSourcePaths {
                composition_source: $source,
                no_read: np,
                no_read_native: &nn,
            }
        };
    }
    let original_source = a.initial_comparison.join("composition-source");
    let weights = CompositionWeights::load(&original_source)?;
    if fit::parameter_hashes(&weights)? != fit::parameter_hashes(&CompositionWeights::new()?)? {
        return Err(invalid(
            "composition initialization differs from fixed asymmetric source",
        ));
    }
    let initial_compiled = CompiledComposition::load(
        &a.initial_comparison.join("composition-native"),
        composition_paths!(&original_source),
        &tokenizer,
    )?;
    initial_compiled.validate_for(&weights)?;
    if initial["compiled_metadata"] != serde_json::to_value(initial_compiled.metadata())? {
        return Err(invalid(
            "initial construction metadata differs from loaded bank",
        ));
    }
    weights.save(&a.out.join("initial-source"))?;
    initial_compiled.save(&a.out.join("initial-native"))?;
    let frozen_roots = [
        ("base", a.model.as_path()),
        ("context-source", cs.as_path()),
        ("context-native", cn.as_path()),
        ("value-source", vs.as_path()),
        ("value-native", vn.as_path()),
        ("event-source", a.event_source.as_path()),
        ("event-native", a.event_native.as_path()),
        ("span-native", a.span_native.as_path()),
        ("potential-native", a.potential_native.as_path()),
        ("reducer-native", rn.as_path()),
        ("no-read-source", ns.as_path()),
        ("no-read-native", nn.as_path()),
        ("original-composition", original_source.as_path()),
    ];
    let frozen_files = file_snapshot(&frozen_roots)?;
    let frozen_variables = variable_hashes(&model)?;
    let identity = json!({"query_parent_report_sha256":sha256_bytes(&parent_bytes),"no_read_fit_report_sha256":scalar_hash,
        "scalar_checkpoint_sha256":sha256_bytes(&checkpoint_bytes),"construction_report_sha256":sha256_bytes(&initial_bytes),
        "initial_composition_metadata":initial_compiled.metadata(),"frozen_files":frozen_files,"frozen_base_variables":frozen_variables,
        "seed":a.seed,"initial_rng":rng,"data_helper_sha256":sha256_bytes(include_bytes!("attention-geometric-value-learned/data.rs"))});
    write(&a.out.join("input-identities.json"), &identity)?;
    struct Panel {
        name: &'static str,
        episodes: Vec<data::Episode>,
        parents: Vec<Value>,
        prefit: Vec<Value>,
        input_hash: String,
        parent_rows_hash: String,
    }
    let mut panels = Vec::new();
    let prefit_started = Instant::now();
    for name in ["original", "stress"] {
        let input = fs::read(a.parent.join(if name == "original" {
            "evaluation.json"
        } else {
            "stress.json"
        }))?;
        let episodes: Vec<data::Episode> = serde_json::from_slice(&input)?;
        let old_bytes = fs::read(a.no_read_fit.join(format!("{name}-rows.json")))?;
        let old_rows: Vec<Value> = serde_json::from_slice(&old_bytes)?;
        let construction_bytes = fs::read(a.initial_comparison.join(format!("{name}-rows.json")))?;
        let construction: Vec<Value> = serde_json::from_slice(&construction_bytes)?;
        let parent_panel = scalar["panels"]
            .as_array()
            .ok_or_else(|| invalid("scalar panels missing"))?
            .iter()
            .find(|p| p["panel"] == name)
            .ok_or_else(|| invalid("scalar panel absent"))?;
        if episodes.len() != 128
            || old_rows.len() != 128
            || construction.is_empty()
            || construction.len() > 128
            || parent_panel["input_sha256"] != sha256_bytes(&input)
        {
            return Err(invalid("parent evaluation panel identity differs"));
        }
        fs::write(a.out.join(format!("{name}-episodes.json")), &input)?;
        fs::write(a.out.join(format!("parent-{name}-rows.json")), &old_bytes)?;
        fs::write(
            a.out.join(format!("construction-{name}-rows.json")),
            &construction_bytes,
        )?;
        let mut prefit = Vec::new();
        for (index, e) in episodes.iter().take(a.rows_per_panel).enumerate() {
            if started.elapsed().as_secs() >= a.maximum_seconds - 300 {
                write(&a.out.join(format!("prefit-{name}-rows.json")), &prefit)?;
                return Err(invalid(
                    "deadline during prefit: no updates performed; retained partial rows",
                ));
            }
            let t = e.ids.len();
            if t == 0 || t > 128 || e.query + 1 != t {
                return Err(invalid("episode shape differs"));
            }
            if old_rows[index]["episode"] != serde_json::to_value(e)? {
                return Err(invalid("parent episode identity differs"));
            }
            let (old, old_read, old_packets) = model.forward_geometric_no_read_native_with_trace(
                &e.ids, 1, t, &context, &events, &span, &potential, &reducer, &values, &no_read,
                false,
            )?;
            let (initial_logits, initial_trace, initial_packets) = model
                .forward_geometric_composition_native_with_trace(
                    &e.ids,
                    1,
                    t,
                    &context,
                    &events,
                    &span,
                    &potential,
                    &reducer,
                    &values,
                    &no_read,
                    &initial_compiled,
                    false,
                )?;
            same_read_inputs(&old_read, &initial_trace, &old_packets, &initial_packets)?;
            let (op, ol) = answer(&old, e.query)?;
            let (ip, il) = answer(&initial_logits, e.query)?;
            if old_rows[index]["native_prediction"] != op
                || old_rows[index]["answer_logits"]["native"] != serde_json::to_value(&ol)?
            {
                return Err(invalid("actual frozen parent prediction/logits changed"));
            }
            if let Some(previous) = construction.get(index) {
                if previous["index"] != index
                    || previous["episode"] != serde_json::to_value(e)?
                    || previous["legacy_answer"] != op
                    || previous["legacy_logits"] != serde_json::to_value(&ol)?
                    || previous["native_answer"] != ip
                    || previous["native_logits"] != serde_json::to_value(&il)?
                {
                    return Err(invalid(
                        "actual initial bank differs from admitted construction row",
                    ));
                }
            }
            prefit.push(json!({"index":index,"episode":e,"parent_prediction":op,"parent_logits":ol,
                "initial_prediction":ip,"initial_logits":il,"query_heads":(0..2).map(|h|initial_trace.rows[h*t+e.query].clone()).collect::<Vec<_>>(),
                "query_no_read_q24":(0..2).map(|h|initial_trace.no_read_q24[h*t+e.query]).collect::<Vec<_>>() }));
            if (index + 1) % 32 == 0 {
                write(&a.out.join(format!("prefit-{name}-rows.json")), &prefit)?;
            }
        }
        write(&a.out.join(format!("prefit-{name}-rows.json")), &prefit)?;
        panels.push(Panel {
            name,
            episodes,
            parents: old_rows,
            prefit,
            input_hash: sha256_bytes(&input),
            parent_rows_hash: sha256_bytes(&old_bytes),
        });
    }
    let prefit_seconds = prefit_started.elapsed().as_secs_f64();
    let preparation_seconds = started.elapsed().as_secs_f64();
    let progress = fit::run(
        &a.out,
        &weights,
        rng,
        a.mode,
        started + Duration::from_secs(a.maximum_seconds - 300),
        &identity,
        |ids, batch, time, w| {
            model.forward_geometric_composition(
                ids, batch, time, &context, &events, &span, &potential, &reducer, &values,
                &no_read, w, false,
            )
        },
    )?;
    let compile_start = Instant::now();
    let saved_source = a.out.join("composition-source");
    let trained_hashes = fit::parameter_hashes(&weights)?;
    weights.save(&saved_source)?;
    let weights = CompositionWeights::load(&saved_source)?;
    if fit::parameter_hashes(&weights)? != trained_hashes {
        return Err(invalid(
            "independent composition source reload changed parameters",
        ));
    }
    let native_dir = a.out.join("composition-native");
    CompiledComposition::compile(&weights, composition_paths!(&saved_source), &tokenizer)?
        .save(&native_dir)?;
    let compiled =
        CompiledComposition::load(&native_dir, composition_paths!(&saved_source), &tokenizer)?;
    compiled.validate_for(&weights)?;
    compiled.validate_dependencies(
        &context, &potential, &reducer, &values, &events, &span, &no_read,
    )?;
    let compilation_seconds = compile_start.elapsed().as_secs_f64();
    let evaluation_start = Instant::now();
    let mut summaries = Vec::new();
    let mut complete = true;
    for panel in panels {
        let mut rows = Vec::new();
        let mut count = [0usize; 4];
        let mut changes = 0;
        let mut max_delta = 0f32;
        let mut initial_gains = 0;
        let mut initial_losses = 0;
        let mut parent_gains = 0;
        let mut parent_losses = 0;
        for (index, e) in panel.episodes.iter().take(a.rows_per_panel).enumerate() {
            if started.elapsed().as_secs() >= a.maximum_seconds {
                complete = false;
                break;
            }
            let t = e.ids.len();
            let source = model.forward_geometric_composition(
                &e.ids, 1, t, &context, &events, &span, &potential, &reducer, &values, &no_read,
                &weights, false,
            )?;
            let (native, trace, packets) = model.forward_geometric_composition_native_with_trace(
                &e.ids, 1, t, &context, &events, &span, &potential, &reducer, &values, &no_read,
                &compiled, false,
            )?;
            let (old, old_read, old_packets) = model.forward_geometric_no_read_native_with_trace(
                &e.ids, 1, t, &context, &events, &span, &potential, &reducer, &values, &no_read,
                false,
            )?;
            same_read_inputs(&old_read, &trace, &old_packets, &packets)?;
            let (op, ol) = answer(&old, e.query)?;
            let (sp, sl) = answer(&source, e.query)?;
            let (np, nl) = answer(&native, e.query)?;
            let initial = &panel.prefit[index];
            let ip = initial["initial_prediction"]
                .as_u64()
                .ok_or_else(|| invalid("initial prediction missing"))?
                as usize;
            if initial["parent_prediction"] != op
                || initial["parent_logits"] != serde_json::to_value(&ol)?
            {
                return Err(invalid("frozen parent changed during fit"));
            }
            let sv = checked_logits(&source, t, model.config.vocab_size)?;
            let nv = checked_logits(&native, t, model.config.vocab_size)?;
            let delta = sv
                .iter()
                .zip(&nv)
                .flat_map(|(a, b)| a.iter().zip(b))
                .map(|(a, b)| (a - b).abs())
                .fold(0f32, f32::max);
            max_delta = max_delta.max(delta);
            changes += usize::from(sp != np);
            for (n, p) in count.iter_mut().zip([op, ip, sp, np]) {
                *n += usize::from(p == e.answer as usize);
            }
            initial_gains += usize::from(np == e.answer as usize && ip != e.answer as usize);
            initial_losses += usize::from(np != e.answer as usize && ip == e.answer as usize);
            parent_gains += usize::from(np == e.answer as usize && op != e.answer as usize);
            parent_losses += usize::from(np != e.answer as usize && op == e.answer as usize);
            rows.push(json!({"index":index,"episode":e,"parent_row":panel.parents[index],"prefit_row":initial,
                "parent_prediction":op,"initial_prediction":ip,"source_prediction":sp,"native_prediction":np,
                "answer_logits":{"parent":ol,"source":sl,"native":nl},"source_native_answer_match":sp==np,
                "maximum_actual_position_logit_delta":delta,"query_heads":(0..2).map(|h|trace.rows[h*t+e.query].clone()).collect::<Vec<_>>(),
                "query_no_read_q24":(0..2).map(|h|trace.no_read_q24[h*t+e.query]).collect::<Vec<_>>(),
                "all_actual_packets_null_weights_identical_to_parent":true}));
            if (index + 1) % 32 == 0 {
                write(&a.out.join(format!("{}-rows.json", panel.name)), &rows)?;
            }
        }
        write(&a.out.join(format!("{}-rows.json", panel.name)), &rows)?;
        summaries.push(json!({"panel":panel.name,"rows":rows.len(),"requested_rows":a.rows_per_panel,
            "answers":{"parent":count[0],"initial_bank":count[1],"source":count[2],"native":count[3]},
            "source_native_answer_changes":changes,"maximum_actual_position_logit_delta":max_delta,
            "gains_from_initial":initial_gains,"losses_from_initial":initial_losses,
            "gains_from_parent":parent_gains,"losses_from_parent":parent_losses,
            "input_sha256":panel.input_hash,"parent_rows_sha256":panel.parent_rows_hash}));
        if !complete {
            break;
        }
    }
    let evaluation_seconds = evaluation_start.elapsed().as_secs_f64();
    if variable_hashes(&model)? != frozen_variables || file_snapshot(&frozen_roots)? != frozen_files
    {
        return Err(invalid("frozen base or producer dependency changed"));
    }
    let complete = complete && progress.completed;
    write(
        &a.out.join("report.json"),
        &json!({
            "schema":"uor-r4.geometric-composition-answer-fit/1","complete":complete,"mode":a.mode,"seed":a.seed,
            "fit":progress,"frozen_producer_updates":1280,"frozen_no_read_updates":640,
            "trainable_inventory":["composition.gains[128]","composition.left_logits[128,120]","composition.right_logits[128,120]"],
            "trainable_scalar_count":30848,"primary_objective":"mean answer CE; one target/episode; no auxiliary",
            "optimizer":{"rate":fit::RATE,"beta1":0.9,"beta2":0.95,"epsilon":1e-8,"weight_decay":0,"clip_selected_global_norm":1},
            "compiled_metadata":compiled.metadata(),"parent":identity,"panels":summaries,
            "preparation_seconds":preparation_seconds,"prefit_evaluation_seconds":prefit_seconds,
            "compilation_reload_seconds":compilation_seconds,"evaluation_seconds":evaluation_seconds,"elapsed_seconds":started.elapsed().as_secs_f64(),
            "decision":if !complete{"PARTIAL_RETAIN_NO_QUALITY_VERDICT"}else if a.mode==fit::Mode::Check{"ACTUAL_TWO_B8_ANSWER_BACKWARDS_ONLY_ZERO_UPDATES"}else{"FIXED_BANK_ONLY_ANSWER_FIT_RETAIN_ALL_ROWS"},
            "scope":"Fixed K2 transport family learned from admitted asymmetric initialization; no perfect donor-reconstruction gate, no general expressivity theorem. Selector roots categorical and gains q4/4; source real-softmax versus native wide-Q16/LUT reduction. All other producer/base/scalar parameters frozen. Float tail and wide learned coefficients elsewhere remain. Check-mode two-draw RNG stream is diagnostic only; checkpoint AdamW moments unavailable.",
        }),
    )?;
    println!(
        "composition fit complete={complete}; updates={}; {}",
        progress.completed_updates,
        a.out.display()
    );
    Ok(())
}

fn same_read_inputs(
    old: &uor_r4_training::geometric_read_native::NativeReadTrace,
    new: &uor_r4_training::geometric_composition_native::ComposedReadTrace,
    old_values: &uor_r4_training::geometric_value_producer::ValueProducerTrace,
    new_values: &uor_r4_training::geometric_value_producer::ValueProducerTrace,
) -> Result<()> {
    if old_values != new_values
        || old.no_read_q24 != new.no_read_q24
        || old.rows.len() != new.rows.len()
    {
        return Err(invalid(
            "composition changes actual packet/null/occurrence inputs",
        ));
    }
    for (a, b) in old.rows.iter().zip(&new.rows) {
        if a.occurrence_weights_q31 != b.occurrence_weights_q31
            || a.no_read_weight_q31 != b.no_read_weight_q31
            || a.total_weight_q31 != b.total_weight_q31
            || a.max_score_q24 != b.max_score_q24
        {
            return Err(invalid(
                "composition changes head-specific source/age/null normalization",
            ));
        }
    }
    Ok(())
}
