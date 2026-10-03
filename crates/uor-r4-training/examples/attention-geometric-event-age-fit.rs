//! Fixed event/standalone-age continuation of the actual retained native reader.
//! A completed same-parent credit check is mandatory. This executable exposes
//! the separately admitted fixed640-update fit and an optional zero-update crossover.
use candle_core::{Device, Tensor, Var};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use uor_r4_core::report_output;
use uor_r4_training::{
    geometric_age_source::AgeSource,
    geometric_composition::CompositionWeights,
    geometric_composition_native::{CompiledComposition, CompositionSourcePaths},
    geometric_context::{CompiledContext, ContextSourcePaths, ContextWeights},
    geometric_event::{CompiledEvents, EventWeights},
    geometric_event_credit::{EventAgeCreditOutput, POLICY},
    geometric_no_read::NoReadWeights,
    geometric_no_read_native::{CompiledNoRead, NoReadSourcePaths},
    geometric_potential_native::{CompiledGeometricPotentials, PotentialSourceBinding},
    geometric_potential_q4::PotentialQ4Weights,
    geometric_read_native::{CompiledGeometricRead, ReadSourceBinding},
    geometric_span_native::{CompiledSpanActions, SpanSourceBinding},
    geometric_stack::StackModel,
    geometric_value_producer::ValueProducerWeights,
    geometric_value_producer_native::{CompiledValueProducer, ValueProducerSourcePaths},
    sha256_bytes, Result,
};
#[path = "attention-geometric-event-age-fit/cross.rs"]
mod cross;
#[path = "attention-geometric-value-learned/data.rs"]
mod data;
#[path = "attention-geometric-event-age-fit/fit.rs"]
mod fit;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Args {
    credit_check: PathBuf,
    out: PathBuf,
    maximum_seconds: u64,
    #[serde(default)]
    counterfactual_fit: Option<PathBuf>,
}
fn invalid(x: impl Into<String>) -> uor_r4_training::TrainingError {
    uor_r4_training::TrainingError::Invalid(x.into())
}
fn write(p: &Path, x: &impl Serialize) -> Result<()> {
    fs::write(p, serde_json::to_vec_pretty(x)?)?;
    Ok(())
}
fn path(v: &Value, k: &str) -> Result<PathBuf> {
    v[k].as_str()
        .map(PathBuf::from)
        .ok_or_else(|| invalid(format!("missing path:{k}")))
}
fn trace_record(t: &uor_r4_training::geometric_event::NativeEventTrace) -> Value {
    json!({"batch":t.batch,"time":t.time,"lanes":t.lanes,
        "actions":t.actions.iter().map(|a|*a as u8).collect::<Vec<_>>(),
        "transition_actions":t.transition_actions,"states":t.states,
        "event_scores":t.event_scores,"coefficient_reads":t.coefficient_reads})
}
fn bits(t: &Tensor) -> Result<Vec<u32>> {
    Ok(t.flatten_all()?
        .to_vec1::<f32>()?
        .into_iter()
        .map(f32::to_bits)
        .collect())
}
fn hashes(vs: &BTreeMap<String, Var>) -> Result<BTreeMap<String, String>> {
    vs.iter()
        .map(|(n, v)| {
            Ok((
                n.clone(),
                sha256_bytes(
                    &bits(v.as_tensor())?
                        .iter()
                        .flat_map(|x| x.to_le_bytes())
                        .collect::<Vec<_>>(),
                ),
            ))
        })
        .collect()
}
fn snapshot(roots: &[(&str, &Path)]) -> Result<BTreeMap<String, String>> {
    fn visit(root: &Path, p: &Path, label: &str, out: &mut BTreeMap<String, String>) -> Result<()> {
        for e in fs::read_dir(p)? {
            let e = e?;
            let ty = e.file_type()?;
            if ty.is_dir() {
                visit(root, &e.path(), label, out)?;
            } else if ty.is_file() {
                let p = e.path();
                let rel = p.strip_prefix(root).map_err(|_| invalid("snapshot path"))?;
                out.insert(
                    format!("{label}/{}", rel.display()),
                    sha256_bytes(&fs::read(p)?),
                );
            } else {
                return Err(invalid("nonregular frozen artifact"));
            }
        }
        Ok(())
    }
    let mut out = BTreeMap::new();
    for (label, p) in roots {
        visit(p, p, label, &mut out)?;
    }
    Ok(out)
}
fn same_payloads(old: &Path, new: &Path) -> Result<Vec<String>> {
    let mut out = Vec::new();
    for e in fs::read_dir(old)? {
        let e = e?;
        let n = e.file_name();
        if n == "metadata.json" {
            continue;
        }
        if !e.file_type()?.is_file() || fs::read(e.path())? != fs::read(new.join(&n))? {
            return Err(invalid(format!(
                "frozen numerical payload changed:{}",
                n.to_string_lossy()
            )));
        }
        out.push(n.to_string_lossy().into_owned());
    }
    out.sort();
    Ok(out)
}
#[derive(Deserialize)]
struct PriorLogits {
    parent: Vec<f32>,
    native: Vec<f32>,
}
#[derive(Deserialize)]
struct PriorRow {
    index: usize,
    episode: data::Episode,
    parent_prediction: usize,
    native_prediction: usize,
    answer_logits: PriorLogits,
}
struct Panel {
    name: &'static str,
    rows: Vec<PriorRow>,
    input_sha256: String,
    saved_rows_sha256: String,
}
fn panels(conversion: &Path, context: &Path) -> Result<Vec<Panel>> {
    let mut result = Vec::new();
    for name in ["original", "stress"] {
        let raw = fs::read(conversion.join(format!("{name}-integrated-rows.json")))?;
        // Typed deserialization skips the large unneeded historical read/value fields.
        let rows: Vec<PriorRow> = serde_json::from_slice(&raw)?;
        let digest = sha256_bytes(&raw);
        drop(raw);
        let input = fs::read(context.join(format!("{name}-episodes.json")))?;
        let episodes: Vec<data::Episode> = serde_json::from_slice(&input)?;
        if rows.len() != 128
            || episodes.len() != 128
            || rows
                .iter()
                .zip(&episodes)
                .enumerate()
                .any(|(i, (r, e))| r.index != i || r.episode != *e)
        {
            return Err(invalid("retained panel identity differs"));
        }
        result.push(Panel {
            name,
            rows,
            input_sha256: sha256_bytes(&input),
            saved_rows_sha256: digest,
        });
    }
    Ok(result)
}
struct Bundle<'a> {
    context: &'a CompiledContext,
    events: &'a CompiledEvents,
    span: &'a CompiledSpanActions,
    potential: &'a CompiledGeometricPotentials,
    reducer: &'a CompiledGeometricRead,
    values: &'a CompiledValueProducer,
    null: &'a CompiledNoRead,
    bank: &'a CompiledComposition,
}
fn answer(t: &Tensor, q: usize, target: usize) -> Result<(usize, Vec<f32>, f64)> {
    let rows = t.to_vec2::<f32>()?;
    let x = rows.get(q).ok_or_else(|| invalid("query missing"))?;
    if x.len() != 40 || target >= x.len() || x.iter().any(|x| !x.is_finite()) {
        return Err(invalid("invalid answer logits"));
    }
    let mut best = 0;
    for i in 1..x.len() {
        if x[i] > x[best] {
            best = i;
        }
    }
    let max = x.iter().copied().fold(f32::NEG_INFINITY, f32::max) as f64;
    let ce = max
        + x.iter()
            .map(|x| (f64::from(*x) - max).exp())
            .sum::<f64>()
            .ln()
        - f64::from(x[target]);
    Ok((best, x.clone(), ce))
}
#[derive(Serialize)]
struct Evaluation {
    complete: bool,
    rows: usize,
    panels: Vec<Value>,
}
#[allow(clippy::too_many_arguments)]
fn evaluate(
    root: &Path,
    model: &StackModel,
    bundle: Bundle<'_>,
    panels: &[Panel],
    baseline: bool,
    deadline: Instant,
    live: Option<&dyn Fn(&[u32]) -> Result<(Tensor, EventAgeCreditOutput)>>,
) -> Result<Evaluation> {
    report_output::claim(root)?;
    let mut done = Evaluation {
        complete: false,
        rows: 0,
        panels: Vec::new(),
    };
    for panel in panels {
        let dir = root.join(panel.name);
        fs::create_dir(&dir)?;
        let (mut count, mut correct, mut oldcorrect, mut rawcorrect) = (0, 0, 0, 0);
        let (mut gain, mut loss, mut rawgain, mut rawloss) = (0, 0, 0, 0);
        let (mut ce_sum, mut old_ce, mut raw_ce) = (0., 0., 0.);
        for r in &panel.rows {
            if Instant::now() >= deadline {
                break;
            }
            let e = &r.episode;
            let time = e.ids.len();
            if time == 0 || time > 128 || e.query >= time {
                return Err(invalid("panel row bounds differ"));
            }
            let (native, read, values) = model.forward_geometric_composition_native_with_trace(
                &e.ids,
                1,
                time,
                bundle.context,
                bundle.events,
                bundle.span,
                bundle.potential,
                bundle.reducer,
                bundle.values,
                bundle.null,
                bundle.bank,
                false,
            )?;
            let (np, nl, nc) = answer(&native, e.query, e.answer as usize)?;
            if baseline
                && (np != r.native_prediction
                    || nl.iter().map(|x| x.to_bits()).ne(r
                        .answer_logits
                        .native
                        .iter()
                        .map(|x| x.to_bits())))
            {
                return Err(invalid(
                    "prefit replay differs from accepted conversion row",
                ));
            }
            let mut trace = Value::Null;
            if let Some(forward) = live {
                let (source, o) = forward(&e.ids)?;
                if bits(&source)? != bits(&native)?
                    || serde_json::to_vec(&read)? != serde_json::to_vec(&o.read)?
                    || serde_json::to_vec(&values)? != serde_json::to_vec(&o.values.trace)?
                {
                    return Err(invalid(
                        "final independent native reload differs from current training hard path",
                    ));
                }
                trace = json!({"event":trace_record(&o.event.trace),"span":o.span,"potential_q24":o.potential.scores_q24,"age_q24":o.age_q24,"policy":o.policy});
            }
            let ce = |x: &[f32]| -> Result<f64> {
                if x.len() != 40 || x.iter().any(|x| !x.is_finite()) {
                    return Err(invalid("prior answer logits invalid"));
                }
                let m = x.iter().copied().fold(f32::NEG_INFINITY, f32::max) as f64;
                Ok(m + x
                    .iter()
                    .map(|x| (f64::from(*x) - m).exp())
                    .sum::<f64>()
                    .ln()
                    - f64::from(x[e.answer as usize]))
            };
            let (ok, old, raw) = (
                np == e.answer as usize,
                r.native_prediction == e.answer as usize,
                r.parent_prediction == e.answer as usize,
            );
            correct += usize::from(ok);
            oldcorrect += usize::from(old);
            rawcorrect += usize::from(raw);
            gain += usize::from(ok && !old);
            loss += usize::from(!ok && old);
            rawgain += usize::from(ok && !raw);
            rawloss += usize::from(!ok && raw);
            ce_sum += nc;
            old_ce += ce(&r.answer_logits.native)?;
            raw_ce += ce(&r.answer_logits.parent)?;
            count += 1;
            done.rows += 1;
            write(
                &dir.join(format!("row-{:03}.json", r.index)),
                &json!({"index":r.index,"episode":e,
    "native_prediction":np,"conversion_prediction":r.native_prediction,"unconverted_prediction":r.parent_prediction,
    "answer_logits":{"native":nl,"conversion":r.answer_logits.native,"unconverted":r.answer_logits.parent},
    "answer_ce":nc,"source_native_all_logit_bits_equal":live.is_some(),"read":read,"values":values,"current":trace}),
            )?;
        }
        done.panels.push(json!({"panel":panel.name,"rows":count,"input_sha256":panel.input_sha256,"conversion_rows_sha256":panel.saved_rows_sha256,
   "correct":{"native":correct,"conversion":oldcorrect,"unconverted":rawcorrect},"ce_sum":{"native":ce_sum,"conversion":old_ce,"unconverted":raw_ce},
   "gains_vs_conversion":gain,"losses_vs_conversion":loss,"gains_vs_unconverted":rawgain,"losses_vs_unconverted":rawloss}));
        write(&root.join("progress.json"), &done)?;
        if count != 128 {
            break;
        }
    }
    done.complete = done.rows == 256;
    write(&root.join("report.json"), &done)?;
    report_output::seal(root)?;
    report_output::verify(root)?;
    Ok(done)
}
fn main() -> Result<()> {
    let mut cli = std::env::args().skip(1);
    let file = cli
        .next()
        .ok_or_else(|| invalid("usage: attention-geometric-event-age-fit ARGS.json"))?;
    if cli.next().is_some() {
        return Err(invalid("one args file required"));
    }
    let bytes = fs::read(file)?;
    let a: Args = serde_json::from_slice(&bytes)?;
    let time_ok = if a.counterfactual_fit.is_some() {
        (61..=840).contains(&a.maximum_seconds)
    } else {
        (361..=86400).contains(&a.maximum_seconds)
    };
    if !time_ok {
        return Err(invalid(
            "explicit61..840 seconds for crossover or361..86400 for fit required; each needs separate admission",
        ));
    }
    report_output::claim(&a.out)?;
    let started = Instant::now();
    fs::write(a.out.join("arguments.json"), bytes)?;
    let result = run(&a, started);
    if let Err(e) = &result {
        write(
            &a.out.join("failure.json"),
            &json!({"error":e.to_string(),"seconds":started.elapsed().as_secs_f64(),"decision":"INCOMPLETE_NO_QUALITY_VERDICT"}),
        )?;
    }
    report_output::seal(&a.out)?;
    report_output::verify(&a.out)?;
    result
}
fn run(a: &Args, started: Instant) -> Result<()> {
    let deadline = started + Duration::from_secs(a.maximum_seconds);
    let fit_deadline = if a.counterfactual_fit.is_some() {
        deadline
    } else {
        deadline - Duration::from_secs(300)
    };
    report_output::verify(&a.credit_check)?;
    let checkbytes = fs::read(a.credit_check.join("report.json"))?;
    let check: Value = serde_json::from_slice(&checkbytes)?;
    let checkargs: Value =
        serde_json::from_slice(&fs::read(a.credit_check.join("arguments.json"))?)?;
    if check["schema"] != "uor-r4.retained-reader-event-age-check/1"
        || check["complete"] != true
        || check["optimizer_updates"] != 0
        || check["selected_shadows"] != 11968
        || check["decision"]
            != "ACTUAL_RETAINED_READER_CREDIT_CHECK_COMPLETE_FIT_REQUIRES_COST_ADMISSION"
        || check["original_base_parameters_unchanged"] != true
        || check["event_source_parameters_unchanged"] != true
        || check["frozen_consumer_parameters_unchanged"] != true
    {
        return Err(invalid(
            "completed actual retained-reader credit check required",
        ));
    }
    let checks = check["checks"]
        .as_array()
        .ok_or_else(|| invalid("check batches missing"))?;
    if checks.len() != 2
        || checks.iter().enumerate().any(|(i, x)| {
            x["absolute_step"] != 3200 + i
                || x["episodes"] != 8
                || x["optimizer_updates"] != 0
                || x["all_position_native_forward_bits_equal"] != true
                || x["all_routes_cut_disconnected"] != true
                || x["frozen_parameter_gradients_absent"] != true
        })
    {
        return Err(invalid("short/long actual check evidence differs"));
    }
    let conversion_attempt = path(&checkargs, "conversion_attempt")?;
    report_output::verify(&conversion_attempt)?;
    let conversion_bytes = fs::read(conversion_attempt.join("report.json"))?;
    let conversion: Value = serde_json::from_slice(&conversion_bytes)?;
    let construction: Value =
        serde_json::from_slice(&fs::read(conversion_attempt.join("arguments.json"))?)?;
    if conversion["complete"] != true
        || conversion["optimizer_updates"] != 0
        || conversion["event"]["admitted"] != true
        || conversion["age"]["admitted"] != true
    {
        return Err(invalid(
            "completed zero-update admitted conversion required",
        ));
    }
    let context_parent = path(&construction, "context_attempt")?;
    report_output::verify(&context_parent)?;
    let parent_bytes = fs::read(path(&construction, "parent_arguments")?)?;
    if parent_bytes != fs::read(context_parent.join("arguments.json"))? {
        return Err(invalid("parent arguments not equal sealed context attempt"));
    }
    let parent: Value = serde_json::from_slice(&parent_bytes)?;
    let seed = parent["seed"]
        .as_u64()
        .ok_or_else(|| invalid("seed missing"))?;
    if ![1, 2].contains(&seed) {
        return Err(invalid("retained seed1/2 required"));
    }
    let model_path = path(&parent, "model")?;
    let model = StackModel::load(&model_path, &Device::Cpu)?;
    if model.config.width != 32
        || model.config.heads != 2
        || model.config.context != 128
        || model.config.vocab_size != 40
    {
        return Err(invalid("V40/H2/width32/context128 required"));
    }
    let base_before = hashes(model.variables())?;
    let span_path = path(&parent, "span_native")?;
    let tokenizer = fs::read(span_path.join("tokenizer-identity.bin"))?;
    let pp = path(&parent, "potential_parent")?;
    let vf = path(&parent, "value_fit")?;
    let nf = path(&parent, "no_read_fit")?;
    let bf = path(&parent, "composition_fit")?;
    for p in [&pp, &vf, &nf, &bf] {
        report_output::verify(p)?;
    }
    let pn = pp.join("potential-q4-native");
    let ps = pp.join("potential-q4-source");
    let es = conversion_attempt.join("event-q4-source");
    let en = conversion_attempt.join("event-q4-native");
    let cs = conversion_attempt.join("rebound-context-source");
    let cn = conversion_attempt.join("rebound-context-native");
    let rn = conversion_attempt.join("age-q4-native");
    let vn = conversion_attempt.join("rebound-value-native");
    let nn = conversion_attempt.join("rebound-no-read-native");
    let bn = conversion_attempt.join("rebound-composition-native");
    let vs = vf.join("q4-value-source");
    let ns = nf.join("no-read-source");
    let bs = bf.join("composition-source");
    let deps = ContextSourcePaths {
        base: &model_path,
        event_source: &es,
        event_native: &en,
        span_native: &span_path,
        potential_native: &pn,
    };
    let context = CompiledContext::load(&cn, &cs, deps, &tokenizer)?;
    let events = CompiledEvents::load(&en, &es, &model_path, &tokenizer)?;
    let event_weights = EventWeights::load_source(&es, &model_path, &tokenizer)?;
    if !event_weights.is_q4() {
        return Err(invalid(
            "saved transformed q4 event source required; no new rescaling",
        ));
    }
    let span = CompiledSpanActions::load(
        &span_path,
        &SpanSourceBinding::from_files(
            &model_path.join("model.safetensors"),
            &model_path.join("config.json"),
            &tokenizer,
        )?,
        model
            .geometric_span()
            .ok_or_else(|| invalid("span configuration absent"))?,
    )?;
    let potential = CompiledGeometricPotentials::load(
        &pn,
        &PotentialSourceBinding::from_directory(&model_path, &tokenizer)?,
    )?;
    let binding = ReadSourceBinding::from_directory(&model_path, &tokenizer, &potential, 2)?;
    let reducer = CompiledGeometricRead::load(&rn, &binding)?;
    let vp = ValueProducerSourcePaths {
        value_source: &vs,
        context_source: &cs,
        context_dependencies: deps,
    };
    let values = CompiledValueProducer::load(&vn, vp, &tokenizer)?;
    let np = NoReadSourcePaths {
        no_read_source: &ns,
        value: vp,
        context_native: &cn,
        value_native: &vn,
        reducer_native: &rn,
    };
    let null = CompiledNoRead::load(&nn, np, &tokenizer)?;
    let composition = CompiledComposition::load(
        &bn,
        CompositionSourcePaths {
            composition_source: &bs,
            no_read: np,
            no_read_native: &nn,
        },
        &tokenizer,
    )?;
    let vw = ValueProducerWeights::load_source(&vs)?;
    let nw = NoReadWeights::load(&ns)?;
    let pw = PotentialQ4Weights::load(&ps)?;
    let bw = CompositionWeights::load(&bs)?;
    let frozen_before = [
        hashes(vw.parameters())?,
        hashes(nw.parameters())?,
        hashes(pw.parameters())?,
        hashes(bw.parameters())?,
    ];
    let age = Var::from_tensor(
        &model
            .variables()
            .get("layers.02.read.age")
            .ok_or_else(|| invalid("base age absent"))?
            .as_tensor()
            .detach(),
    )?;
    let age_before = bits(age.as_tensor())?;
    let event_before = hashes(event_weights.parameters())?;
    if event_weights
        .parameters()
        .values()
        .map(|v| v.elem_count())
        .sum::<usize>()
        + age.elem_count()
        != 11968
    {
        return Err(invalid("event/age selected inventory differs"));
    }
    if check["seed"] != seed
        || check["conversion_report_sha256"] != sha256_bytes(&conversion_bytes)
        || check["standalone_age_baseline_unchanged"] != true
    {
        return Err(invalid(
            "credit check is not bound to this accepted conversion",
        ));
    }
    if !reducer.is_q4_age_residual()
        || AgeSource::new(age.as_tensor(), &binding)?.age_q24() != reducer.age_q24()
    {
        return Err(invalid(
            "initial absolute age does not reproduce accepted residual conversion",
        ));
    }
    let value_report: Value = serde_json::from_slice(&fs::read(vf.join("report.json"))?)?;
    let initial_rng = value_report["fit"]["next_generator_state"]
        .as_u64()
        .ok_or_else(|| invalid("saved post3200 RNG missing"))?;
    let pinned_rng = if seed == 1 {
        17847168693577953940u64
    } else {
        8892160473256393877u64
    };
    if value_report["fit"]["completed_updates"] != 640
        || initial_rng != pinned_rng
        || check["initial_generator_state"].as_u64() != Some(initial_rng)
        || check["next_training_generator_state"].as_u64() != Some(initial_rng)
    {
        return Err(invalid("saved sampler/check continuation identity differs"));
    }
    let cw = ContextWeights::load_source(&cs, deps, &tokenizer)?;
    let context_before = hashes(cw.parameters())?;
    let roots = [
        ("model", model_path.as_path()),
        ("context_source", cs.as_path()),
        ("context_native", cn.as_path()),
        ("event_source", es.as_path()),
        ("event_native", en.as_path()),
        ("span", span_path.as_path()),
        ("potential_source", ps.as_path()),
        ("potential_native", pn.as_path()),
        ("value_source", vs.as_path()),
        ("value_native", vn.as_path()),
        ("null_source", ns.as_path()),
        ("null_native", nn.as_path()),
        ("bank_source", bs.as_path()),
        ("bank_native", bn.as_path()),
        ("reducer", rn.as_path()),
    ];
    let files_before = snapshot(&roots)?;
    let parent_binding = json!({"seed":seed,"credit_check":a.credit_check,"credit_check_report_sha256":sha256_bytes(&checkbytes),
        "conversion_attempt":conversion_attempt,"conversion_report_sha256":sha256_bytes(&conversion_bytes),
        "initial_generator_state":initial_rng,"initial_parameters":fit::parameter_hashes(&event_weights,&age)?,
        "frozen_artifact_files":files_before,"hard_backward_policy":POLICY});
    write(&a.out.join("inputs.json"), &parent_binding)?;
    if let Some(fit_root) = &a.counterfactual_fit {
        let panels = panels(&conversion_attempt, &context_parent)?;
        let forward = |ids: &[u32], ew: &EventWeights, ag: &Tensor| {
            model.forward_geometric_event_age_native_credit(
                ids,
                1,
                ids.len(),
                &context,
                &events,
                &span,
                &potential,
                &reducer,
                &values,
                &null,
                &composition,
                ew,
                ag,
                &vw,
                &nw,
                &pw,
                &bw,
                [false; 4],
            )
        };
        let mut report = cross::run(
            &a.out,
            fit_root,
            &model_path,
            &tokenizer,
            &binding,
            &parent_binding,
            &event_weights,
            age.as_tensor(),
            &panels,
            deadline,
            forward,
        )?;
        if snapshot(&roots)? != files_before
            || hashes(model.variables())? != base_before
            || hashes(cw.parameters())? != context_before
            || [
                hashes(vw.parameters())?,
                hashes(nw.parameters())?,
                hashes(pw.parameters())?,
                hashes(bw.parameters())?,
            ] != frozen_before
        {
            return Err(invalid("crossover changed a frozen file or parameter"));
        }
        report["frozen_files_and_parameters_unchanged"] = json!(true);
        report["elapsed_seconds"] = json!(started.elapsed().as_secs_f64());
        write(&a.out.join("report.json"), &report)?;
        return Ok(());
    }
    fs::create_dir(a.out.join("checkpoints"))?;
    fit::checkpoint(
        &a.out.join("checkpoints/step-0000"),
        0,
        initial_rng,
        &event_weights,
        &age,
        &model_path,
        &tokenizer,
        &binding,
        &parent_binding,
    )?;
    let panels = panels(&conversion_attempt, &context_parent)?;
    let preparation_seconds = started.elapsed().as_secs_f64();
    let baseline = evaluate(
        &a.out.join("baseline"),
        &model,
        Bundle {
            context: &context,
            events: &events,
            span: &span,
            potential: &potential,
            reducer: &reducer,
            values: &values,
            null: &null,
            bank: &composition,
        },
        &panels,
        true,
        fit_deadline,
        None,
    )?;
    if !baseline.complete {
        return Err(invalid("prefit panel incomplete; no optimizer started"));
    }
    let frozen = model
        .variables()
        .values()
        .chain(cw.parameters().values())
        .chain(vw.parameters().values())
        .chain(nw.parameters().values())
        .chain(pw.parameters().values())
        .chain(bw.parameters().values())
        .cloned()
        .collect::<Vec<_>>();
    let forward = |ids: &[u32], batch: usize, time: usize, event: &EventWeights, age: &Tensor| {
        model.forward_geometric_event_age_native_credit(
            ids,
            batch,
            time,
            &context,
            &events,
            &span,
            &potential,
            &reducer,
            &values,
            &null,
            &composition,
            event,
            age,
            &vw,
            &nw,
            &pw,
            &bw,
            [true; 4],
        )
    };
    let fitted = fit::run(
        &a.out,
        &event_weights,
        &age,
        initial_rng,
        fit_deadline,
        &model_path,
        &tokenizer,
        &binding,
        &parent_binding,
        &frozen,
        forward,
    )?;
    // Independently saved/reloaded current sources. Only dependency envelopes change for frozen consumers.
    let export_at = Instant::now();
    let nes = a.out.join("event-source");
    let nen = a.out.join("event-native");
    event_weights.save_source(&nes, &model_path, &tokenizer)?;
    let ew = EventWeights::load_source(&nes, &model_path, &tokenizer)?;
    CompiledEvents::compile(&ew, &nes, &model_path, &tokenizer)?.save(&nen)?;
    let nevents = CompiledEvents::load(&nen, &nes, &model_path, &tokenizer)?;
    let nas = a.out.join("age-source");
    let nrn = a.out.join("age-native");
    AgeSource::new(age.as_tensor(), &binding)?.save(&nas)?;
    let age_source = AgeSource::load(&nas, &binding)?;
    CompiledGeometricRead::compile_learned_age(age.as_tensor(), &potential, &binding, &age_source)?
        .save(&nrn)?;
    let nreducer = CompiledGeometricRead::load(&nrn, &binding)?;
    nreducer.validate_age_source(&age_source)?;
    if bits(&age_source.tensor()?)? != bits(age.as_tensor())?
        || hashes(ew.parameters())? != hashes(event_weights.parameters())?
    {
        return Err(invalid(
            "independent event/age reload differs from fitted source",
        ));
    }
    let npaths = ContextSourcePaths {
        base: &model_path,
        event_source: &nes,
        event_native: &nen,
        span_native: &span_path,
        potential_native: &pn,
    };
    let ncs = a.out.join("rebound-context-source");
    let ncn = a.out.join("rebound-context-native");
    cw.save_source(&ncs, npaths, &tokenizer)?;
    let ncw = ContextWeights::load_source(&ncs, npaths, &tokenizer)?;
    CompiledContext::compile(&ncw, &ncs, npaths, &tokenizer)?.save(&ncn)?;
    let ncontext = CompiledContext::load(&ncn, &ncs, npaths, &tokenizer)?;
    let context_source_payload = same_payloads(&cs, &ncs)?;
    let context_native_payload = same_payloads(&cn, &ncn)?;
    let nvn = a.out.join("rebound-value-native");
    let nvp = ValueProducerSourcePaths {
        value_source: &vs,
        context_source: &ncs,
        context_dependencies: npaths,
    };
    CompiledValueProducer::compile(nvp, &tokenizer)?.save(&nvn)?;
    let nvalues = CompiledValueProducer::load(&nvn, nvp, &tokenizer)?;
    let value_payload = same_payloads(&vn, &nvn)?;
    let nnn = a.out.join("rebound-no-read-native");
    let nnp = NoReadSourcePaths {
        no_read_source: &ns,
        value: nvp,
        context_native: &ncn,
        value_native: &nvn,
        reducer_native: &nrn,
    };
    CompiledNoRead::compile(&nw, nnp, &tokenizer)?.save(&nnn)?;
    let nnull = CompiledNoRead::load(&nnn, nnp, &tokenizer)?;
    let null_payload = same_payloads(&nn, &nnn)?;
    let nbn = a.out.join("rebound-composition-native");
    let nbp = CompositionSourcePaths {
        composition_source: &bs,
        no_read: nnp,
        no_read_native: &nnn,
    };
    CompiledComposition::compile(&bw, nbp, &tokenizer)?.save(&nbn)?;
    let nbank = CompiledComposition::load(&nbn, nbp, &tokenizer)?;
    let bank_payload = same_payloads(&bn, &nbn)?;
    if reducer.exp_q31() != nreducer.exp_q31() || hashes(ncw.parameters())? != context_before {
        return Err(invalid("frozen context parameters or exp table changed"));
    }
    let admitted = uor_r4_training::geometric_attention_native::CompiledGeometricAttention::new(
        &ncontext, &nevents, &span, &potential, &nreducer, &nvalues, &nnull, &nbank,
    )?;
    admitted.validate_stack(&model)?;
    if !admitted.metadata().wider_source_components.is_empty() {
        return Err(invalid("export retains a wider learned attention source"));
    }
    let export_seconds = export_at.elapsed().as_secs_f64();
    // The live path uses current shadows with immutable enrolled parent provenance;
    // the independent native path uses the newly rebound/exported bundle.
    let live = |ids: &[u32]| {
        model.forward_geometric_event_age_native_credit(
            ids,
            1,
            ids.len(),
            &context,
            &events,
            &span,
            &potential,
            &reducer,
            &values,
            &null,
            &composition,
            &event_weights,
            age.as_tensor(),
            &vw,
            &nw,
            &pw,
            &bw,
            [true; 4],
        )
    };
    let final_panel = evaluate(
        &a.out.join("final"),
        &model,
        Bundle {
            context: &ncontext,
            events: &nevents,
            span: &span,
            potential: &potential,
            reducer: &nreducer,
            values: &nvalues,
            null: &nnull,
            bank: &nbank,
        },
        &panels,
        false,
        deadline,
        Some(&live),
    )?;
    if snapshot(&roots)? != files_before
        || hashes(model.variables())? != base_before
        || hashes(cw.parameters())? != context_before
        || [
            hashes(vw.parameters())?,
            hashes(nw.parameters())?,
            hashes(pw.parameters())?,
            hashes(bw.parameters())?,
        ] != frozen_before
    {
        return Err(invalid("frozen parent file or parameter changed"));
    }
    let complete = fitted.completed && final_panel.complete;
    write(
        &a.out.join("report.json"),
        &json!({"schema":"uor-r4.retained-reader-event-age-fit/1","complete":complete,
        "seed":seed,"fit":fitted,"baseline":baseline,"final":final_panel,
        "event_initial_parameters":event_before,"age_initial_bits_sha256":sha256_bytes(&age_before.iter().flat_map(|x|x.to_le_bytes()).collect::<Vec<_>>()),
        "selected_parameters":11968,"optimizer":{"name":"AdamW","lr":0.003,"beta1":0.9,"beta2":0.999,"eps":1e-8,"decay":0.,"selected_global_clip":1.,"moment_restart":true},
        "objective":"ordinary same-native-hard-model mean answer CE; one answer per episode; no auxiliary",
        "event_shadow_range":[-1.75,1.75],"age_shadow_range":"fixed prior(h0=-lag/16,h1=-lag/256) +/-7/8",
        "checkpoint_semantics":"source weights and completed-update RNG; AdamW moments NOT_SERIALIZED",
        "hard_backward_policy":POLICY,"frozen_files_unchanged":true,"frozen_parameters_unchanged":true,
        "independent_event_age_reload_exact":true,"unchanged_payloads":{"context_source":context_source_payload,
        "context_native":context_native_payload,"value":value_payload,"no_read":null_payload,"bank":bank_payload},
        "exp_table_unchanged":true,"native_admission":admitted.metadata(),"preparation_seconds":preparation_seconds,
        "export_seconds":export_seconds,"elapsed_seconds":started.elapsed().as_secs_f64(),
        "decision":if complete{"FIXED_DOSE_COMPLETE_RETAIN_ALL_ROWS_NO_AUTOMATIC_FOLLOWUP"}else{"PARTIAL_NO_QUALITY_VERDICT"},
        "scope":"Open development authored panels, fixed native attention and explicit biased capture/normalization adjoint. Float trunk and vocabulary tail retained; no language, geometric advantage or full model serving qualification."}),
    )?;
    Ok(())
}
