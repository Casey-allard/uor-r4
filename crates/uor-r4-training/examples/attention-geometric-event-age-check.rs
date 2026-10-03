//! Two real-parent B8 backward checks, without optimizer or sampler advancement.
use candle_core::{backprop::GradStore, Device, Tensor, Var};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::Instant,
};
use uor_r4_core::report_output;
use uor_r4_training::{
    geometric_age_source::AgeSource,
    geometric_composition::CompositionWeights,
    geometric_composition_native::{CompiledComposition, CompositionSourcePaths},
    geometric_context::{CompiledContext, ContextSourcePaths},
    geometric_event::{CompiledEvents, EventWeights},
    geometric_no_read::NoReadWeights,
    geometric_no_read_native::{CompiledNoRead, NoReadSourcePaths},
    geometric_potential_native::{CompiledGeometricPotentials, PotentialSourceBinding},
    geometric_potential_q4::PotentialQ4Weights,
    geometric_read_native::{CompiledGeometricRead, ReadSourceBinding},
    geometric_span_native::{CompiledSpanActions, SpanSourceBinding},
    geometric_stack::{logits_cross_entropy, StackModel},
    geometric_value_producer::ValueProducerWeights,
    geometric_value_producer_native::{CompiledValueProducer, ValueProducerSourcePaths},
    sha256_bytes, Result,
};
#[path = "attention-geometric-value-learned/data.rs"]
mod data;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Args {
    conversion_attempt: PathBuf,
    out: PathBuf,
    maximum_seconds: u64,
}
fn invalid(m: impl Into<String>) -> uor_r4_training::TrainingError {
    uor_r4_training::TrainingError::Invalid(m.into())
}
fn write(p: &Path, v: &impl serde::Serialize) -> Result<()> {
    fs::write(p, serde_json::to_vec_pretty(v)?)?;
    Ok(())
}
fn path(v: &Value, name: &str) -> Result<PathBuf> {
    v[name]
        .as_str()
        .map(PathBuf::from)
        .ok_or_else(|| invalid(format!("missing path:{name}")))
}
fn bits(t: &Tensor) -> Result<Vec<u32>> {
    Ok(t.flatten_all()?
        .to_vec1::<f32>()?
        .into_iter()
        .map(f32::to_bits)
        .collect())
}
fn hashes(vars: &BTreeMap<String, Var>) -> Result<BTreeMap<String, String>> {
    vars.iter()
        .map(|(name, v)| {
            Ok((
                name.clone(),
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
fn gradient(g: &GradStore, v: &Var) -> Result<Value> {
    let Some(t) = g.get(v.as_tensor()) else {
        return Ok(json!({"connected":false,"l2":0.,"nonzero":0}));
    };
    let values = t.flatten_all()?.to_vec1::<f32>()?;
    if values.iter().any(|x| !x.is_finite()) {
        return Err(invalid("nonfinite answer gradient"));
    }
    let l2 = values
        .iter()
        .map(|x| f64::from(*x).powi(2))
        .sum::<f64>()
        .sqrt();
    Ok(
        json!({"connected":true,"l2":l2,"nonzero":values.iter().filter(|x|**x!=0.).count(),"coordinates":values.len()}),
    )
}
// Exact unchanged sampler logic from value-q4-fit/fit.rs::training_batch.
// Uses only a diagnostic copy of the saved post-update3200 RNG state.
fn training_batch(rng: &mut data::Rng, absolute_step: usize) -> Vec<data::Episode> {
    let n = [2, 4][absolute_step % 2];
    (0..8)
        .map(|_| {
            let facts = data::draw(rng, n);
            let query = rng.next(n);
            let noise = (0..n).map(|_| data::noise(rng, 4)).collect::<Vec<_>>();
            let query_noise = data::noise(rng, 4);
            data::episode(
                &facts,
                &noise,
                &facts[query].0,
                &query_noise,
                absolute_step,
                "joint_train",
            )
        })
        .collect()
}
fn main() -> Result<()> {
    let mut cli = std::env::args().skip(1);
    let p = cli
        .next()
        .ok_or_else(|| invalid("usage: attention-geometric-event-age-check ARGS.json"))?;
    if cli.next().is_some() {
        return Err(invalid("one argument file required"));
    }
    let bytes = fs::read(p)?;
    let a: Args = serde_json::from_slice(&bytes)?;
    if !(61..=840).contains(&a.maximum_seconds) {
        return Err(invalid("explicit61..840 second check bound required"));
    }
    report_output::claim(&a.out)?;
    let at = Instant::now();
    fs::write(a.out.join("arguments.json"), bytes)?;
    let result = run(&a, at);
    if let Err(e) = &result {
        write(
            &a.out.join("failure.json"),
            &json!({"error":e.to_string(),"elapsed_seconds":at.elapsed().as_secs_f64(),"optimizer_updates":0,"decision":"INCOMPLETE_NO_FIT_ADMISSION"}),
        )?;
    }
    report_output::seal(&a.out)?;
    report_output::verify(&a.out)?;
    result
}
fn run(a: &Args, at: Instant) -> Result<()> {
    report_output::verify(&a.conversion_attempt)?;
    let conversion_bytes = fs::read(a.conversion_attempt.join("report.json"))?;
    let conversion: Value = serde_json::from_slice(&conversion_bytes)?;
    let construction: Value =
        serde_json::from_slice(&fs::read(a.conversion_attempt.join("arguments.json"))?)?;
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
    let es = a.conversion_attempt.join("event-q4-source");
    let en = a.conversion_attempt.join("event-q4-native");
    let cs = a.conversion_attempt.join("rebound-context-source");
    let cn = a.conversion_attempt.join("rebound-context-native");
    let rn = a.conversion_attempt.join("age-q4-native");
    let vn = a.conversion_attempt.join("rebound-value-native");
    let nn = a.conversion_attempt.join("rebound-no-read-native");
    let bn = a.conversion_attempt.join("rebound-composition-native");
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
    let source = AgeSource::new(age.as_tensor(), &binding)?;
    if source.age_q24() != reducer.age_q24() {
        return Err(invalid(
            "current age does not regenerate accepted residual parent",
        ));
    }
    let age_source_path = a.out.join("initial-age-source");
    source.save(&age_source_path)?;
    let source = AgeSource::load(&age_source_path, &binding)?;
    let age_native_path = a.out.join("initial-age-native");
    CompiledGeometricRead::compile_learned_age(age.as_tensor(), &potential, &binding, &source)?
        .save(&age_native_path)?;
    let exported = CompiledGeometricRead::load(&age_native_path, &binding)?;
    exported.validate_age_source(&source)?;
    if exported.age_q24() != reducer.age_q24() {
        return Err(invalid(
            "standalone age export changes baseline numeric table",
        ));
    }
    // Export one declared parameter intervention without touching the training Var.
    let mut changed = age
        .as_tensor()
        .to_vec2::<f32>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    changed[0] = if reducer.age_q24()[0] == (1 << 21) {
        0.
    } else {
        0.125
    };
    let changed = Tensor::from_vec(changed, (2, 128), &Device::Cpu)?;
    let changed_source = AgeSource::new(&changed, &binding)?;
    changed_source.save(&a.out.join("changed-age-source"))?;
    let changed_source = AgeSource::load(&a.out.join("changed-age-source"), &binding)?;
    let changed_native = a.out.join("changed-age-native");
    CompiledGeometricRead::compile_learned_age(&changed, &potential, &binding, &changed_source)?
        .save(&changed_native)?;
    let changed_native = CompiledGeometricRead::load(&changed_native, &binding)?;
    changed_native.validate_age_source(&changed_source)?;
    if changed_native.age_q24() == exported.age_q24()
        || changed_native.validate_age_source(&source).is_ok()
    {
        return Err(invalid(
            "changed age not independently bound or stale source accepted",
        ));
    }
    let value_fit_report: Value = serde_json::from_slice(&fs::read(vf.join("report.json"))?)?;
    if value_fit_report["fit"]["completed_updates"] != 640 {
        return Err(invalid("completed640 value parent required"));
    }
    let initial_rng = value_fit_report["fit"]["next_generator_state"]
        .as_u64()
        .ok_or_else(|| invalid("saved post-value RNG absent"))?;
    let mut rng = data::Rng(initial_rng);
    let mut checks = Vec::new();
    for check in 0..2 {
        if at.elapsed().as_secs() >= a.maximum_seconds {
            return Err(invalid("configured whole check bound reached"));
        }
        let absolute_step = 3200 + check;
        let rng_before = rng.0;
        let episodes = training_batch(&mut rng, absolute_step);
        write(&a.out.join(format!("batch-{check}.json")), &episodes)?;
        let (ids, targets, mask, time) = data::batch(&episodes);
        if episodes.len() != 8
            || time > 128
            || episodes
                .iter()
                .any(|e| e.query + 1 != e.ids.len() || e.ids[e.query] != 34)
            || mask.iter().filter(|x| **x == 1.).count() != 8
        {
            return Err(invalid("actual answer-only B8 contract differs"));
        }
        let prepare = Instant::now();
        let native = model
            .forward_geometric_composition_native_with_trace(
                &ids,
                8,
                time,
                &context,
                &events,
                &span,
                &potential,
                &reducer,
                &values,
                &null,
                &composition,
                false,
            )?
            .0;
        let native_bits = bits(&native)?;
        let native_ce = logits_cross_entropy(&native, &targets, Some(&mask))?.to_scalar::<f32>()?;
        drop(native);
        let native_seconds = prepare.elapsed().as_secs_f64();
        let calc = Instant::now();
        let (logits, _) = model.forward_geometric_event_age_native_credit(
            &ids,
            8,
            time,
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
        )?;
        if bits(&logits)? != native_bits {
            return Err(invalid(
                "connected actual-reader forward differs from same-shape native baseline",
            ));
        }
        let loss = logits_cross_entropy(&logits, &targets, Some(&mask))?;
        let ce = loss.to_scalar::<f32>()?;
        if !ce.is_finite() || ce.to_bits() != native_ce.to_bits() {
            return Err(invalid("actual native answer CE differs or nonfinite"));
        }
        let g = loss.backward()?;
        let event_gradients = event_weights
            .parameters()
            .iter()
            .map(|(n, v)| Ok((n.clone(), gradient(&g, v)?)))
            .collect::<Result<BTreeMap<_, _>>>()?;
        let age_gradient = gradient(&g, &age)?;
        if model
            .variables()
            .values()
            .chain(vw.parameters().values())
            .chain(nw.parameters().values())
            .chain(pw.parameters().values())
            .chain(bw.parameters().values())
            .any(|v| g.get(v.as_tensor()).is_some())
        {
            return Err(invalid("frozen surrounding parameter has answer gradient"));
        }
        let connected_seconds = calc.elapsed().as_secs_f64();
        drop(g);
        drop(loss);
        drop(logits);
        let cut_at = Instant::now();
        let (cut, _) = model.forward_geometric_event_age_native_credit(
            &ids,
            8,
            time,
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
            [false; 4],
        )?;
        if bits(&cut)? != native_bits {
            return Err(invalid("all-route cut changes native hard output"));
        }
        let cut_grad = logits_cross_entropy(&cut, &targets, Some(&mask))?.backward()?;
        if event_weights
            .parameters()
            .values()
            .chain(std::iter::once(&age))
            .any(|v| cut_grad.get(v.as_tensor()).is_some())
        {
            return Err(invalid("all-route cut still connects event or age"));
        }
        checks.push(json!({"absolute_step":absolute_step,"facts":([2,4][check]),"episodes":8,"actual_positions":episodes.iter().map(|e|e.ids.len()).sum::<usize>(),"padded_positions":ids.len(),"time":time,"rng_before":rng_before,"diagnostic_rng_after":rng.0,"answer_denominator":8,"answer_ce":ce,"native_baseline_seconds":native_seconds,"connected_forward_backward_seconds":connected_seconds,"cut_seconds":cut_at.elapsed().as_secs_f64(),"event_gradients":event_gradients,"age_gradient":age_gradient,"all_position_native_forward_bits_equal":true,"all_routes_cut_disconnected":true,"frozen_parameter_gradients_absent":true,"optimizer_updates":0}));
        write(&a.out.join("partial-checks.json"), &checks)?;
    }
    if hashes(model.variables())? != base_before
        || hashes(event_weights.parameters())? != event_before
        || bits(age.as_tensor())? != age_before
        || [
            hashes(vw.parameters())?,
            hashes(nw.parameters())?,
            hashes(pw.parameters())?,
            hashes(bw.parameters())?,
        ] != frozen_before
    {
        return Err(invalid("zero-update check changed a source parameter"));
    }
    let event_useful = checks.iter().any(|x| {
        x["event_gradients"]
            .as_object()
            .is_some_and(|f| f.values().any(|g| g["l2"].as_f64().is_some_and(|n| n > 0.)))
    });
    write(
        &a.out.join("report.json"),
        &json!({"schema":"uor-r4.retained-reader-event-age-check/1","complete":true,"seed":seed,"conversion_report_sha256":sha256_bytes(&conversion_bytes),"initial_generator_state":initial_rng,"next_training_generator_state":initial_rng,"observed_diagnostic_generator_state":rng.0,"optimizer_updates":0,"selected_shadows":11968,"standalone_age_baseline_unchanged":true,"changed_age_export_reload_and_stale_source_rejection":true,"original_base_parameters_unchanged":true,"event_source_parameters_unchanged":true,"frozen_consumer_parameters_unchanged":true,"checks":checks,"elapsed_seconds":at.elapsed().as_secs_f64(),"decision":if event_useful{"ACTUAL_RETAINED_READER_CREDIT_CHECK_COMPLETE_FIT_REQUIRES_COST_ADMISSION"}else{"ACTUAL_EVENT_CREDIT_ZERO_NO_FIT_ADMISSION"},"scope":"Two exposed sampler continuation diagnostics, zero updates. Native integer hard attention with explicitly biased capture/real-softmax backward and frozen F32 trunk/head; no language, full serving, geometric advantage or optimizer fit claim."}),
    )?;
    Ok(())
}
