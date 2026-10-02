//! Actual saved-parent construction, no optimization and no quality gate.
use candle_core::{Device, Tensor};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
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
fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let path = args
        .next()
        .ok_or_else(|| invalid("usage: attention-geometric-composition ARGS.json"))?;
    if args.next().is_some() {
        return Err(invalid("one arguments path required"));
    }
    let bytes = fs::read(path)?;
    let a: Args = serde_json::from_slice(&bytes)?;
    if !(1..=128).contains(&a.rows_per_panel) || !(1..=900).contains(&a.maximum_seconds) {
        return Err(invalid("rows1..128,seconds1..900 required"));
    }
    report_output::claim(&a.out)?;
    fs::write(a.out.join("arguments.json"), bytes)?;
    let result = run(&a);
    report_output::seal(&a.out)?;
    report_output::verify(&a.out)?;
    result
}
fn run(a: &Args) -> Result<()> {
    let start = Instant::now();
    report_output::verify(&a.parent)?;
    report_output::verify(&a.no_read_fit)?;
    let parent_report_bytes = fs::read(a.parent.join("report.json"))?;
    let parent_report: Value = serde_json::from_slice(&parent_report_bytes)?;
    let fit_report_bytes = fs::read(a.no_read_fit.join("report.json"))?;
    let fit_report: Value = serde_json::from_slice(&fit_report_bytes)?;
    if parent_report["complete"] != true
        || parent_report["completed_updates"] != 640
        || parent_report["cumulative_updates"] != 1280
        || parent_report["auxiliary_credit"] != "query_read"
        || fit_report["complete"] != true
        || fit_report["mode"] != "fit"
        || fit_report["fit"]["completed_updates"] != 640
        || fit_report["frozen_producer_updates"] != 1280
    {
        return Err(invalid("completed query and NoRead fit parents required"));
    }
    let model = StackModel::load(&a.model, &Device::Cpu)?;
    if model.config.width != 32 || model.config.heads != 2 || model.config.context != 128 {
        return Err(invalid("actual H2width32context128 parent required"));
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
            .ok_or_else(|| invalid("parent span config missing"))?,
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
    if fit_report["no_read_metadata"] != serde_json::to_value(no_read.metadata())? {
        return Err(invalid(
            "NoRead fit report differs from actual loaded artifact",
        ));
    }
    let source = a.out.join("composition-source");
    let weights = CompositionWeights::new()?;
    weights.save(&source)?;
    let weights = CompositionWeights::load(&source)?;
    let cp = CompositionSourcePaths {
        composition_source: &source,
        no_read: np,
        no_read_native: &nn,
    };
    let native_dir = a.out.join("composition-native");
    CompiledComposition::compile(&weights, cp, &tokenizer)?.save(&native_dir)?;
    let compiled = CompiledComposition::load(&native_dir, cp, &tokenizer)?;
    compiled.validate_dependencies(
        &context, &potential, &reducer, &values, &events, &span, &no_read,
    )?;
    let tampered = a.out.join("tampered-width-native");
    fs::create_dir(&tampered)?;
    for file in [
        "metadata.json",
        "left-roots.bin",
        "right-roots.bin",
        "gains-q4.bin",
        "tokenizer-identity.bin",
    ] {
        fs::copy(native_dir.join(file), tampered.join(file))?;
    }
    let mut meta: Value = serde_json::from_slice(&fs::read(tampered.join("metadata.json"))?)?;
    meta["payload_storage_bits"] = json!(32);
    write(&tampered.join("metadata.json"), &meta)?;
    let rejects_width = CompiledComposition::load(&tampered, cp, &tokenizer).is_err();
    if !rejects_width {
        return Err(invalid("composition admits wrong payload width"));
    }
    let mut panels = Vec::new();
    let mut complete = true;
    let mut gradient = Value::Null;
    let mut prefix = Value::Null;
    let mut poison = Value::Null;
    let mut maximum_delta = 0f32;
    for name in ["original", "stress"] {
        let input = fs::read(a.parent.join(if name == "original" {
            "evaluation.json"
        } else {
            "stress.json"
        }))?;
        let episodes: Vec<data::Episode> = serde_json::from_slice(&input)?;
        let retained_bytes = fs::read(a.no_read_fit.join(format!("{name}-rows.json")))?;
        let retained: Vec<Value> = serde_json::from_slice(&retained_bytes)?;
        let fit_panel = fit_report["panels"]
            .as_array()
            .ok_or_else(|| invalid("fit panels missing"))?
            .iter()
            .find(|p| p["panel"] == name)
            .ok_or_else(|| invalid("fit panel absent"))?;
        if fit_panel["input_sha256"] != sha256_bytes(&input) || retained.len() != 128 {
            return Err(invalid("changed saved-parent evaluation panel"));
        }
        if episodes.len() != 128 {
            return Err(invalid("fixed128 parent panel required"));
        }
        let mut rows = Vec::new();
        let mut counts = [0usize; 3];
        for (index, e) in episodes.iter().take(a.rows_per_panel).enumerate() {
            if start.elapsed().as_secs() >= a.maximum_seconds {
                complete = false;
                break;
            }
            let t = e.ids.len();
            if t == 0 || t > 128 || e.query + 1 != t {
                return Err(invalid("parent episode shape differs"));
            }
            let (old, old_read, old_values) = model.forward_geometric_no_read_native_with_trace(
                &e.ids, 1, t, &context, &events, &span, &potential, &reducer, &values, &no_read,
                false,
            )?;
            let source_logits = model.forward_geometric_composition(
                &e.ids, 1, t, &context, &events, &span, &potential, &reducer, &values, &no_read,
                &weights, false,
            )?;
            let (native_logits, trace, packets) = model
                .forward_geometric_composition_native_with_trace(
                    &e.ids, 1, t, &context, &events, &span, &potential, &reducer, &values,
                    &no_read, &compiled, false,
                )?;
            if packets != old_values
                || trace.no_read_q24 != old_read.no_read_q24
                || trace.rows.len() != old_read.rows.len()
            {
                return Err(invalid(
                    "composition altered actual packets/null/occurrences",
                ));
            }
            for (a, b) in trace.rows.iter().zip(&old_read.rows) {
                if a.occurrence_weights_q31 != b.occurrence_weights_q31
                    || a.no_read_weight_q31 != b.no_read_weight_q31
                    || a.total_weight_q31 != b.total_weight_q31
                    || a.max_score_q24 != b.max_score_q24
                {
                    return Err(invalid("composition changes occurrence/age/NoRead weights"));
                }
            }
            checked_logits(&old, t, model.config.vocab_size)?;
            checked_logits(&source_logits, t, model.config.vocab_size)?;
            checked_logits(&native_logits, t, model.config.vocab_size)?;
            let (oa, ol) = answer(&old, e.query)?;
            if retained[index]["episode"] != serde_json::to_value(e)?
                || retained[index]["native_prediction"] != oa
                || retained[index]["answer_logits"]["native"] != serde_json::to_value(&ol)?
            {
                return Err(invalid(
                    "loaded NoRead parent prediction/logits differ from retained row",
                ));
            }
            let (sa, sl) = answer(&source_logits, e.query)?;
            let (na, nl) = answer(&native_logits, e.query)?;
            let delta = sl
                .iter()
                .zip(&nl)
                .map(|(x, y)| (x - y).abs())
                .fold(0f32, f32::max);
            maximum_delta = maximum_delta.max(delta);
            for (c, p) in counts.iter_mut().zip([oa, sa, na]) {
                *c += usize::from(p == e.answer as usize);
            }
            if name == "original" && index == 0 {
                let loss = candle_nn::loss::cross_entropy(
                    &source_logits.narrow(0, e.query, 1)?,
                    &Tensor::from_vec(vec![e.answer], 1, &Device::Cpu)?,
                )?;
                let g = loss.backward()?;
                let mut grads = serde_json::Map::new();
                for (name, var) in weights.parameters() {
                    let v = g
                        .get(var.as_tensor())
                        .ok_or_else(|| invalid(format!("answer gradient disconnected:{name}")))?
                        .flatten_all()?
                        .to_vec1::<f32>()?;
                    if v.iter().any(|x| !x.is_finite()) {
                        return Err(invalid("nonfinite composition answer gradient"));
                    }
                    grads.insert(name.clone(),json!({"l1":v.iter().map(|x|f64::from(x.abs())).sum::<f64>(),"nonzero":v.iter().filter(|x|**x!=0.).count()}));
                }
                gradient = json!(grads);
                let variable = model
                    .variables()
                    .get("layers.02.read.out.weight")
                    .ok_or_else(|| invalid("legacy read.out missing"))?
                    .clone();
                let original = Tensor::from_vec(
                    variable.flatten_all()?.to_vec1::<f32>()?,
                    variable.shape(),
                    &Device::Cpu,
                )?;
                variable.set(&Tensor::full(f32::NAN, original.shape(), &Device::Cpu)?)?;
                let changed = model.forward_geometric_composition_native_with_trace(
                    &e.ids, 1, t, &context, &events, &span, &potential, &reducer, &values,
                    &no_read, &compiled, false,
                );
                variable.set(&original)?;
                let (changed, _, _) = changed?;
                let changed = answer(&changed, e.query)?.1;
                let equal = changed == nl;
                if !equal {
                    return Err(invalid("poisoned legacy read.out changes composition"));
                }
                poison = json!({"legacy_read_out_nan_bypassed":equal});
                if t < 128 {
                    let mut ids = e.ids.clone();
                    ids.push(1);
                    let (future, future_trace, _) = model
                        .forward_geometric_composition_native_with_trace(
                            &ids,
                            1,
                            t + 1,
                            &context,
                            &events,
                            &span,
                            &potential,
                            &reducer,
                            &values,
                            &no_read,
                            &compiled,
                            false,
                        )?;
                    let first = checked_logits(&native_logits, t, model.config.vocab_size)?;
                    let second = checked_logits(&future, t + 1, model.config.vocab_size)?;
                    for h in 0..2 {
                        for q in 0..t {
                            if trace.rows[h * t + q] != future_trace.rows[h * (t + 1) + q]
                                || trace.no_read_q24[h * t + q]
                                    != future_trace.no_read_q24[h * (t + 1) + q]
                            {
                                return Err(invalid(
                                    "future token changes exact integer composition prefix",
                                ));
                            }
                        }
                    }
                    let max = first
                        .iter()
                        .zip(&second[..t])
                        .flat_map(|(a, b)| a.iter().zip(b))
                        .map(|(a, b)| (a - b).abs())
                        .fold(0f32, f32::max);
                    prefix = json!({"exact_integer_prefix_equal":true,"maximum_float_tail_prefix_logit_delta":max,"prefix_length":t,"scope":"exact integer per-head values/weights/null;unfinished float-tail difference reported without bit-parity assertion"});
                }
            }
            rows.push(json!({"index":index,"episode":e,"legacy_answer":oa,"source_answer":sa,"native_answer":na,"legacy_logits":ol,"source_logits":sl,"native_logits":nl,"max_source_native_logit_delta":delta,"source_native_answer_match":sa==na,"query_heads":(0..2).map(|h|trace.rows[h*t+e.query].clone()).collect::<Vec<_>>()}));
        }
        write(&a.out.join(format!("{name}-rows.json")), &rows)?;
        panels.push(json!({"name":name,"input_sha256":sha256_bytes(&input),"rows":rows.len(),"correct":{"legacy":counts[0],"source":counts[1],"native":counts[2]},"source_native_answer_matches":rows.iter().filter(|r|r["source_native_answer_match"]==true).count()}));
    }
    write(
        &a.out.join("report.json"),
        &json!({"schema":"uor-r4.geometric-composition-construction/1","complete":complete,"optimizer_updates":0,"initialization":"fixed-asymmetric-geometry-bank;not-donor-projection-or-answer-selected","quality_admission_gate":false,"panels":panels,"ordinary_answer_gradients":gradient,"poison":poison,"causal_prefix":prefix,"wrong_width_rejected":rejects_width,"maximum_source_native_answer_logit_delta":maximum_delta,"query_parent_report_sha256":sha256_bytes(&parent_report_bytes),"no_read_fit_report_sha256":sha256_bytes(&fit_report_bytes),"legacy_parent_rows_replayed":true,"compiled_metadata":compiled.metadata(),"elapsed_seconds":start.elapsed().as_secs_f64(),"scope":"connected geometric read.out construction;float tail and wide producer coefficients remain;not learnedquality or whole-serving qualification"}),
    )?;
    println!(
        "composition construction complete={complete},elapsed={:.3}s",
        start.elapsed().as_secs_f64()
    );
    Ok(())
}
