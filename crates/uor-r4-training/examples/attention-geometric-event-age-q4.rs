//! Source-bound strict event/age admission on retained parents; zero updates.
//! A rejected fixed grid is a construction decision, not a geometry verdict.
use candle_core::{Device, Tensor};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{fs, path::PathBuf, time::Instant};
use uor_r4_core::report_output;
use uor_r4_training::{
    geometric_attention_native::CompiledGeometricAttention,
    geometric_composition::CompositionWeights,
    geometric_composition_native::{CompiledComposition, CompositionSourcePaths},
    geometric_context::{CompiledContext, ContextSourcePaths, ContextWeights},
    geometric_event::{self, CompiledEvents, EventWeights},
    geometric_no_read::NoReadWeights,
    geometric_no_read_native::{CompiledNoRead, NoReadSourcePaths},
    geometric_potential_native::{CompiledGeometricPotentials, PotentialSourceBinding},
    geometric_read_native::{CompiledGeometricRead, ReadSourceBinding},
    geometric_span_native::{CompiledSpanActions, SpanSourceBinding},
    geometric_stack::StackModel,
    geometric_value_producer_native::{CompiledValueProducer, ValueProducerSourcePaths},
    sha256_bytes, Result, TrainingError,
};
#[derive(Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ConversionMode {
    #[default]
    UnchangedQuarter,
    FixedGaugePriorResidual,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Args {
    parent_arguments: PathBuf,
    context_attempt: PathBuf,
    out: PathBuf,
    maximum_seconds: u64,
    #[serde(default)]
    conversion_mode: ConversionMode,
}
fn invalid(s: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(s.into())
}
fn path(a: &Value, key: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(a[key].as_str().ok_or_else(|| {
        invalid(format!("missing parent path:{key}"))
    })?))
}
fn write(p: PathBuf, v: &impl serde::Serialize) -> Result<()> {
    fs::write(p, serde_json::to_vec_pretty(v)?)?;
    Ok(())
}
fn trace_record(t: &geometric_event::NativeEventTrace) -> Value {
    json!({"batch":t.batch,"time":t.time,"lanes":t.lanes,
        "actions":t.actions.iter().map(|a|*a as u8).collect::<Vec<_>>(),
        "transition_actions":t.transition_actions,"states":t.states,
        "event_scores":t.event_scores,"coefficient_reads":t.coefficient_reads})
}
fn main() -> Result<()> {
    let mut cli = std::env::args().skip(1);
    let input = cli
        .next()
        .ok_or_else(|| invalid("usage: attention-geometric-event-age-q4 ARGS.json"))?;
    if cli.next().is_some() {
        return Err(invalid("one arguments file required"));
    }
    let bytes = fs::read(input)?;
    let a: Args = serde_json::from_slice(&bytes)?;
    if !(60..=840).contains(&a.maximum_seconds) {
        return Err(invalid("explicit60..840 second bound required"));
    }
    report_output::claim(&a.out)?;
    let at = Instant::now();
    fs::write(a.out.join("arguments.json"), bytes)?;
    let result = run(&a, at);
    if let Err(e) = &result {
        write(
            a.out.join("failure.json"),
            &json!({"error":e.to_string(),"elapsed_seconds":at.elapsed().as_secs_f64(),"decision":"INCOMPLETE_NO_QUALITY_VERDICT"}),
        )?;
    }
    report_output::seal(&a.out)?;
    report_output::verify(&a.out)?;
    result
}
fn run(a: &Args, at: Instant) -> Result<()> {
    report_output::verify(&a.context_attempt)?;
    let parent_bytes = fs::read(&a.parent_arguments)?;
    if parent_bytes != fs::read(a.context_attempt.join("arguments.json"))? {
        return Err(invalid(
            "parent argument bytes differ from sealed context attempt",
        ));
    }
    let parent: Value = serde_json::from_slice(&parent_bytes)?;
    let parent_report_bytes = fs::read(a.context_attempt.join("report.json"))?;
    let parent_report: Value = serde_json::from_slice(&parent_report_bytes)?;
    let expected_parent_sha = match parent["seed"].as_u64() {
        Some(1) => "c0d4ca673dea64f9177360e9d96f0ff67685b7c12741cd30653c3ac117823c25",
        Some(2) => "8c70ee949778d2e2f85604dc518bef21961388e72061fd9039b9cae0fdc55eeb",
        _ => return Err(invalid("retained seed1/2 required")),
    };
    if sha256_bytes(&parent_report_bytes) != expected_parent_sha {
        return Err(invalid("retained context parent identity differs"));
    }
    if parent_report["complete"] != true || parent_report["optimizer_updates"] != 0 {
        return Err(invalid("complete zero-update parent required"));
    }
    let model_path = path(&parent, "model")?;
    let event_source = path(&parent, "event_source")?;
    let event_native = path(&parent, "event_native")?;
    let span_native = path(&parent, "span_native")?;
    let potential_parent = path(&parent, "potential_parent")?;
    report_output::verify(&potential_parent)?;
    let model = StackModel::load(&model_path, &Device::Cpu)?;
    if model.config.vocab_size != 40
        || model.config.heads != 2
        || model.config.width != 32
        || model.config.context != 128
    {
        return Err(invalid(
            "retained V40 H2 width32 context128 source required",
        ));
    }
    let tokenizer = fs::read(span_native.join("tokenizer-identity.bin"))?;
    let original_inputs = [
        ("model", model_path.join("model.safetensors")),
        ("config", model_path.join("config.json")),
        (
            "event_parameters",
            event_source.join("event-parameters.safetensors"),
        ),
        ("event_metadata", event_source.join("metadata.json")),
    ]
    .into_iter()
    .map(|(name, path)| Ok((name.to_owned(), sha256_bytes(&fs::read(path)?))))
    .collect::<Result<std::collections::BTreeMap<_, _>>>()?;
    let old_events = CompiledEvents::load(&event_native, &event_source, &model_path, &tokenizer)?;
    let raw_events = EventWeights::load_source(&event_source, &model_path, &tokenizer)?;
    let transform = if a.conversion_mode == ConversionMode::FixedGaugePriorResidual {
        for (name, v) in raw_events.parameters() {
            let scale = match name.as_str() {
                "token_transition" | "self_transition" | "neighbor_transition" => 0.5,
                "token_event" | "state_event" => 0.25,
                _ => return Err(invalid("unrecognized event source family")),
            };
            v.set(&v.as_tensor().affine(scale, 0.)?)?;
        }
        "offline-TT/ST/NT-times1/2;TE/SE-times1/4;hard-positive-gauge-before-quantization;no-runtime-inverse;T1-surrogate-changed/1"
    } else {
        "unchanged-source-quarter-grid/1"
    };
    let event_result = raw_events.into_q4();
    let (event_weights, events, event_admission) = match event_result {
        Ok(w) => {
            let source = a.out.join("event-q4-source");
            let native = a.out.join("event-q4-native");
            w.save_source(&source, &model_path, &tokenizer)?;
            let w = EventWeights::load_source(&source, &model_path, &tokenizer)?;
            CompiledEvents::compile(&w, &source, &model_path, &tokenizer)?.save(&native)?;
            let e = CompiledEvents::load(&native, &source, &model_path, &tokenizer)?;
            e.validate_q4_training(&w)?;
            let record = json!({"admitted":true,"metadata":e.metadata(),"packed_sha256":sha256_bytes(&w.packed_coefficients()?)});
            (Some(w), Some(e), record)
        }
        Err(e) => (
            None,
            None,
            json!({"admitted":false,"refusal":e.to_string(),"decision":"FIXED_SOURCE_GRID_NOT_ADMITTED_NO_CLIPPING"}),
        ),
    };
    let potential = CompiledGeometricPotentials::load(
        &potential_parent.join("potential-q4-native"),
        &PotentialSourceBinding::from_directory(&model_path, &tokenizer)?,
    )?;
    let binding = ReadSourceBinding::from_directory(&model_path, &tokenizer, &potential, 2)?;
    let old_read =
        CompiledGeometricRead::load(&a.context_attempt.join("rebound-native-read"), &binding)?;
    let age = model
        .variables()
        .get("layers.02.read.age")
        .ok_or_else(|| invalid("saved age missing"))?
        .as_tensor();
    let age_result = match a.conversion_mode {
        ConversionMode::UnchangedQuarter => {
            CompiledGeometricRead::compile_q4_age(age, &potential, &binding)
        }
        ConversionMode::FixedGaugePriorResidual => {
            CompiledGeometricRead::compile_q4_age_residual(age, &potential, &binding)
        }
    };
    let mut new_reducer = None;
    let age_admission = match age_result {
        Ok(read) => {
            let native = a.out.join("age-q4-native");
            read.save(&native)?;
            let read = CompiledGeometricRead::load(&native, &binding)?;
            if old_read.exp_q31() != read.exp_q31() {
                return Err(invalid("strict age changed exp table"));
            }
            let record =
                json!({"admitted":true,"metadata":read.metadata(),"exp_table_unchanged":true});
            new_reducer = Some(read);
            record
        }
        Err(e) => {
            json!({"admitted":false,"refusal":e.to_string(),"decision":"FIXED_SOURCE_GRID_NOT_ADMITTED_NO_CLIPPING"})
        }
    };
    write(
        a.out.join("boundary-admission.json"),
        &json!({"event":event_admission,"age":age_admission}),
    )?;
    let mut panels = Vec::new();
    if let (Some(weights), Some(events)) = (&event_weights, &events) {
        for panel in ["original", "stress"] {
            let input = fs::read(a.context_attempt.join(format!("{panel}-episodes.json")))?;
            let episodes: Vec<Value> = serde_json::from_slice(&input)?;
            if episodes.len() != 128 {
                return Err(invalid("128 retained rows required"));
            }
            let mut rows = Vec::new();
            let mut changed = 0usize;
            for (index, episode) in episodes.iter().enumerate() {
                if at.elapsed().as_secs() >= a.maximum_seconds {
                    return Err(invalid("configured admission comparison limit reached"));
                }
                let ids: Vec<u32> = serde_json::from_value(episode["ids"].clone())?;
                let source = weights.forward_q4(&ids, 1, ids.len(), false)?;
                let native = geometric_event::trace_native(&ids, 1, ids.len(), events, false)?;
                let old = geometric_event::trace_native(&ids, 1, ids.len(), &old_events, false)?;
                if source.trace != native {
                    return Err(invalid("strict event source/native trace differs"));
                }
                let changes = old
                    .actions
                    .iter()
                    .zip(&native.actions)
                    .filter(|(a, b)| a != b)
                    .count();
                changed += changes;
                rows.push(json!({"index":index,"episode":episode,"old_trace":trace_record(&old),"q4_trace":trace_record(&native),"changed_actions":changes,"source_native_exact":true}));
            }
            write(a.out.join(format!("{panel}-event-rows.json")), &rows)?;
            panels.push(json!({"panel":panel,"rows":rows.len(),"input_sha256":sha256_bytes(&input),"changed_actions":changed}));
        }
    }
    let integrated = if let (Some(events), Some(reducer)) = (&events, &new_reducer) {
        Some(run_integrated(
            a,
            &parent,
            &model,
            &tokenizer,
            &potential,
            &old_read,
            reducer,
            &old_events,
            events,
            at,
        )?)
    } else {
        None
    };
    write(
        a.out.join("report.json"),
        &json!({
            "schema":"uor-r4.geometric-event-age-q4-admission/1","complete":true,"optimizer_updates":0,
            "parent_arguments_sha256":sha256_bytes(&parent_bytes),"original_inputs":original_inputs,"parent_report_sha256":sha256_bytes(&parent_report_bytes),
            "event":event_admission,"age":age_admission,"event_panels":panels,"integrated":integrated,"offline_event_transform":transform,
            "elapsed_seconds":at.elapsed().as_secs_f64(),
            "decision":if event_admission["admitted"]==true && age_admission["admitted"]==true {"INTEGRATED_FIXED_CONVERSION_COMPLETE_RETAIN_ALL_ROWS"} else {"PRESERVE_FIXED_GRID_REFUSAL_REQUIRES_CAUSAL_LEARNING_DESIGN"},
            "scope":"Actual source/native event fidelity and explicit age admission; admitted modes receive retained-panel integrated persistent/whole-prefix comparison. No optimizer fit, natural-language qualification, full serving or geometric advantage claim"
        }),
    )?;
    Ok(())
}

fn same_payloads(old: &std::path::Path, new: &std::path::Path) -> Result<Vec<String>> {
    let mut names = Vec::new();
    for e in fs::read_dir(old)? {
        let e = e?;
        let name = e.file_name();
        if name == "metadata.json" {
            continue;
        }
        if !e.file_type()?.is_file() || fs::read(e.path())? != fs::read(new.join(&name))? {
            return Err(invalid(format!(
                "rebinding changed frozen payload:{}",
                name.to_string_lossy()
            )));
        }
        names.push(name.to_string_lossy().into_owned());
    }
    names.sort();
    Ok(names)
}
fn bits(t: &Tensor) -> Result<Vec<u32>> {
    Ok(t.flatten_all()?
        .to_vec1::<f32>()?
        .iter()
        .map(|x| x.to_bits())
        .collect())
}
fn answer(t: &Tensor, q: usize, target: usize) -> Result<(usize, Vec<f32>, f64)> {
    let rows = t.to_vec2::<f32>()?;
    let row = rows
        .get(q)
        .ok_or_else(|| invalid("answer position missing"))?;
    if row.is_empty() || row.iter().any(|x| !x.is_finite()) || target >= row.len() {
        return Err(invalid("invalid answer logits/target"));
    }
    let mut best = 0;
    for i in 1..row.len() {
        if row[i] > row[best] {
            best = i;
        }
    }
    let maximum = row.iter().copied().fold(f32::NEG_INFINITY, f32::max) as f64;
    let ce = maximum
        + row
            .iter()
            .map(|&x| (f64::from(x) - maximum).exp())
            .sum::<f64>()
            .ln()
        - f64::from(row[target]);
    Ok((best, row.clone(), ce))
}
#[allow(clippy::too_many_arguments)]
fn run_integrated(
    a: &Args,
    parent: &Value,
    model: &StackModel,
    tokenizer: &[u8],
    potential: &CompiledGeometricPotentials,
    old_reducer: &CompiledGeometricRead,
    reducer: &CompiledGeometricRead,
    old_events: &CompiledEvents,
    events: &CompiledEvents,
    at: Instant,
) -> Result<Value> {
    let model_path = path(parent, "model")?;
    let old_es = path(parent, "event_source")?;
    let old_en = path(parent, "event_native")?;
    let sn = path(parent, "span_native")?;
    let pp = path(parent, "potential_parent")?;
    let pn = pp.join("potential-q4-native");
    let cs = a.context_attempt.join("rebound-context-source");
    let cn = a.context_attempt.join("rebound-context-native");
    let rn = a.context_attempt.join("rebound-native-read");
    let vn = a.context_attempt.join("rebound-value-native");
    let nn = a.context_attempt.join("rebound-no-read-native");
    let bn = a.context_attempt.join("rebound-composition-native");
    let vf = path(parent, "value_fit")?;
    let nf = path(parent, "no_read_fit")?;
    let bf = path(parent, "composition_fit")?;
    for root in [&vf, &nf, &bf] {
        report_output::verify(root)?;
    }
    let vs = vf.join("q4-value-source");
    let ns = nf.join("no-read-source");
    let bs = bf.join("composition-source");
    let oldpaths = ContextSourcePaths {
        base: &model_path,
        event_source: &old_es,
        event_native: &old_en,
        span_native: &sn,
        potential_native: &pn,
    };
    let oldcontext = CompiledContext::load(&cn, &cs, oldpaths, tokenizer)?;
    let cw = ContextWeights::load_source(&cs, oldpaths, tokenizer)?;
    if !cw.is_q4() {
        return Err(invalid("retained q4 context required"));
    }
    let span = CompiledSpanActions::load(
        &sn,
        &SpanSourceBinding::from_files(
            &model_path.join("model.safetensors"),
            &model_path.join("config.json"),
            tokenizer,
        )?,
        model
            .geometric_span()
            .ok_or_else(|| invalid("span config missing"))?,
    )?;
    let ovp = ValueProducerSourcePaths {
        value_source: &vs,
        context_source: &cs,
        context_dependencies: oldpaths,
    };
    let oldvalues = CompiledValueProducer::load(&vn, ovp, tokenizer)?;
    let onp = NoReadSourcePaths {
        no_read_source: &ns,
        value: ovp,
        context_native: &cn,
        value_native: &vn,
        reducer_native: &rn,
    };
    let oldnull = CompiledNoRead::load(&nn, onp, tokenizer)?;
    let oldbank = CompiledComposition::load(
        &bn,
        CompositionSourcePaths {
            composition_source: &bs,
            no_read: onp,
            no_read_native: &nn,
        },
        tokenizer,
    )?;
    let es = a.out.join("event-q4-source");
    let en = a.out.join("event-q4-native");
    let paths = ContextSourcePaths {
        base: &model_path,
        event_source: &es,
        event_native: &en,
        span_native: &sn,
        potential_native: &pn,
    };
    let ncs = a.out.join("rebound-context-source");
    let ncn = a.out.join("rebound-context-native");
    cw.save_source(&ncs, paths, tokenizer)?;
    let cw = ContextWeights::load_source(&ncs, paths, tokenizer)?;
    CompiledContext::compile(&cw, &ncs, paths, tokenizer)?.save(&ncn)?;
    let context = CompiledContext::load(&ncn, &ncs, paths, tokenizer)?;
    let context_source_payload = same_payloads(&cs, &ncs)?;
    let context_native_payload = same_payloads(&cn, &ncn)?;
    let nrn = a.out.join("age-q4-native");
    let nvn = a.out.join("rebound-value-native");
    let vp = ValueProducerSourcePaths {
        value_source: &vs,
        context_source: &ncs,
        context_dependencies: paths,
    };
    CompiledValueProducer::compile(vp, tokenizer)?.save(&nvn)?;
    let values = CompiledValueProducer::load(&nvn, vp, tokenizer)?;
    let value_payload = same_payloads(&vn, &nvn)?;
    let nnn = a.out.join("rebound-no-read-native");
    let np = NoReadSourcePaths {
        no_read_source: &ns,
        value: vp,
        context_native: &ncn,
        value_native: &nvn,
        reducer_native: &nrn,
    };
    CompiledNoRead::compile(&NoReadWeights::load(&ns)?, np, tokenizer)?.save(&nnn)?;
    let null = CompiledNoRead::load(&nnn, np, tokenizer)?;
    let null_payload = same_payloads(&nn, &nnn)?;
    let nbn = a.out.join("rebound-composition-native");
    let bp = CompositionSourcePaths {
        composition_source: &bs,
        no_read: np,
        no_read_native: &nnn,
    };
    CompiledComposition::compile(&CompositionWeights::load(&bs)?, bp, tokenizer)?.save(&nbn)?;
    let bank = CompiledComposition::load(&nbn, bp, tokenizer)?;
    let bank_payload = same_payloads(&bn, &nbn)?;
    let admitted = CompiledGeometricAttention::new(
        &context, events, &span, potential, reducer, &values, &null, &bank,
    )?;
    admitted.validate_stack(model)?;
    if !admitted.metadata().wider_source_components.is_empty() {
        return Err(invalid(
            "strict candidate retains a wider learned attention source",
        ));
    }
    if CompiledGeometricAttention::new(
        &context, events, &span, potential, reducer, &oldvalues, &null, &bank,
    )
    .is_ok()
    {
        return Err(invalid(
            "old valid value/context dependency silently admitted",
        ));
    }
    let mut summaries = Vec::new();
    for panel in ["original", "stress"] {
        let input = fs::read(a.context_attempt.join(format!("{panel}-episodes.json")))?;
        let saved_bytes = fs::read(a.context_attempt.join(format!("{panel}-rows.json")))?;
        let episodes: Vec<Value> = serde_json::from_slice(&input)?;
        let saved: Vec<Value> = serde_json::from_slice(&saved_bytes)?;
        if episodes.len() != 128 || saved.len() != 128 {
            return Err(invalid("complete128-row retained panel required"));
        }
        let mut rows = Vec::new();
        let mut correct = [0usize; 2];
        let mut total_ce = [0.; 2];
        let mut gains = 0;
        let mut losses = 0;
        for (index, e) in episodes.iter().enumerate() {
            if at.elapsed().as_secs() >= a.maximum_seconds {
                return Err(invalid("configured integrated comparison limit reached"));
            }
            let ids: Vec<u32> = serde_json::from_value(e["ids"].clone())?;
            let q = e["query"]
                .as_u64()
                .ok_or_else(|| invalid("query missing"))? as usize;
            let target = e["answer"]
                .as_u64()
                .ok_or_else(|| invalid("target missing"))? as usize;
            if ids.is_empty() || ids.len() > 128 || q >= ids.len() {
                return Err(invalid("retained row bounds differ"));
            }
            let (old, old_read, old_value) = model
                .forward_geometric_composition_native_with_trace(
                    &ids,
                    1,
                    ids.len(),
                    &oldcontext,
                    old_events,
                    &span,
                    potential,
                    old_reducer,
                    &oldvalues,
                    &oldnull,
                    &oldbank,
                    false,
                )?;
            let (new, new_read, new_value) = model
                .forward_geometric_composition_native_with_trace(
                    &ids,
                    1,
                    ids.len(),
                    &context,
                    events,
                    &span,
                    potential,
                    reducer,
                    &values,
                    &null,
                    &bank,
                    false,
                )?;
            let incremental =
                model.forward_geometric_attention_session(&ids, 1, ids.len(), &admitted)?;
            if bits(&new)? != bits(&incremental)? {
                return Err(invalid(format!(
                    "{panel} row{index} incremental/whole-prefix bits differ"
                )));
            }
            let (op, ol, oc) = answer(&old, q, target)?;
            let (np, nl, nc) = answer(&new, q, target)?;
            if saved[index]["episode"] != *e
                || saved[index]["native_prediction"] != op
                || serde_json::from_value::<Vec<f32>>(
                    saved[index]["answer_logits"]["native"].clone(),
                )?
                .iter()
                .map(|x| x.to_bits())
                .ne(ol.iter().map(|x| x.to_bits()))
            {
                return Err(invalid("retained parent answer replay differs"));
            }
            correct[0] += usize::from(op == target);
            correct[1] += usize::from(np == target);
            total_ce[0] += oc;
            total_ce[1] += nc;
            gains += usize::from(op != target && np == target);
            losses += usize::from(op == target && np != target);
            if index == 0 {
                let short = ids.len() / 2;
                let (prefix_native, prefix_read, _) = model
                    .forward_geometric_composition_native_with_trace(
                        &ids[..short],
                        1,
                        short,
                        &context,
                        events,
                        &span,
                        potential,
                        reducer,
                        &values,
                        &null,
                        &bank,
                        false,
                    )?;
                let prefix_session = model.forward_geometric_attention_session(
                    &ids[..short],
                    1,
                    short,
                    &admitted,
                )?;
                if bits(&prefix_native)? != bits(&prefix_session)? {
                    return Err(invalid("same-shape candidate prefix bridge differs"));
                }
                // Integer rows compare across prefix lengths. The floating tail
                // compares only against a reference with the same GEMM shape.
                for h in 0..2 {
                    for t in 0..short {
                        if prefix_read.rows[h * short + t] != new_read.rows[h * ids.len() + t] {
                            return Err(invalid("future prefix changes integer reader row"));
                        }
                    }
                }
                let other: Vec<u32> =
                    serde_json::from_value(episodes[(index + 1) % episodes.len()]["ids"].clone())?;
                let bt = ids.len().min(other.len());
                let b2ids = ids[..bt]
                    .iter()
                    .chain(&other[..bt])
                    .copied()
                    .collect::<Vec<_>>();
                let (b2native, _, _) = model.forward_geometric_composition_native_with_trace(
                    &b2ids, 2, bt, &context, events, &span, potential, reducer, &values, &null,
                    &bank, false,
                )?;
                let b2session =
                    model.forward_geometric_attention_session(&b2ids, 2, bt, &admitted)?;
                if bits(&b2native)? != bits(&b2session)? {
                    return Err(invalid("same-shape candidate B2 reset/provenance differs"));
                }
            }
            rows.push(json!({"index":index,"episode":e,"parent_prediction":op,"native_prediction":np,"answer_logits":{"parent":ol,"native":nl},"answer_ce":{"parent":oc,"native":nc},"old_read":old_read,"new_read":new_read,"old_value":old_value,"new_value":new_value,"all_actual_position_session_bits_equal":true}));
        }
        write(a.out.join(format!("{panel}-integrated-rows.json")), &rows)?;
        summaries.push(json!({"panel":panel,"rows":rows.len(),"input_sha256":sha256_bytes(&input),"saved_rows_sha256":sha256_bytes(&saved_bytes),"correct":{"parent":correct[0],"native":correct[1]},"mean_answer_ce":{"parent":total_ce[0]/rows.len() as f64,"native":total_ce[1]/rows.len() as f64},"gains":gains,"losses":losses,"all_actual_position_session_bits_equal":true}));
    }
    Ok(
        json!({"admission":admitted.metadata(),"panels":summaries,"unchanged_payloads":{"context_source":context_source_payload,"context_native":context_native_payload,"value":value_payload,"no_read":null_payload,"bank":bank_payload},"exp_table_unchanged":old_reducer.exp_q31()==reducer.exp_q31(),"mismatched_value_context_refused":true,"B2_prefix_checked":true,"scope":"Exposed retained authored panels; exact integer attention with unchanged float Stack tail; zero updates, no natural-language or geometry-advantage qualification"}),
    )
}
