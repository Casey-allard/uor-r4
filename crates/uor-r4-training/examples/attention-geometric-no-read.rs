//! Actual-parent, no-fit comparison of the connected geometric NoRead path.
//! Zero coefficients validate construction, not learned quality or abstention.
use candle_core::{Device, Tensor};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
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
    no_read_source: Option<PathBuf>,
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
fn main() -> Result<()> {
    let mut cli = std::env::args().skip(1);
    let path = cli
        .next()
        .ok_or_else(|| invalid("usage: attention-geometric-no-read ARGS.json"))?;
    if cli.next().is_some() {
        return Err(invalid("exactly one configuration path required"));
    }
    let argument_bytes = fs::read(path)?;
    let a: Args = serde_json::from_slice(&argument_bytes)?;
    if !(1..=128).contains(&a.rows_per_panel) || !(1..=1800).contains(&a.maximum_seconds) {
        return Err(invalid(
            "rows_per_panel1..128 and maximum_seconds1..1800 required",
        ));
    }
    report_output::claim(&a.out)?;
    let started = Instant::now();
    fs::write(a.out.join("arguments.json"), &argument_bytes)?;
    let parent_report_bytes = fs::read(a.parent.join("report.json"))?;
    let parent_report: Value = serde_json::from_slice(&parent_report_bytes)?;
    if parent_report["complete"] != true
        || parent_report["completed_updates"] != 640
        || parent_report["cumulative_updates"] != 1280
        || parent_report["auxiliary_credit"] != "query_read"
    {
        return Err(invalid("completed query-credit parent required"));
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
    let weights = match &a.no_read_source {
        Some(path) => NoReadWeights::load(path)?,
        None => NoReadWeights::new(40, 2, 4)?,
    };
    let saved_source = a.out.join("no-read-source");
    weights.save(&saved_source)?;
    let weights = NoReadWeights::load(&saved_source)?;
    let paths = NoReadSourcePaths {
        no_read_source: &saved_source,
        value: value_paths,
        context_native: &context_native,
        value_native: &value_native,
        reducer_native: &reducer_native,
    };
    let compiled_directory = a.out.join("no-read-native");
    CompiledNoRead::compile(&weights, paths, &tokenizer)?.save(&compiled_directory)?;
    let compiled = CompiledNoRead::load(&compiled_directory, paths, &tokenizer)?;
    let mut panels = Vec::new();
    let mut complete = true;
    let mut answer_gradient = Value::Null;
    for name in ["original", "stress"] {
        let input_path = a.parent.join(if name == "original" {
            "evaluation.json"
        } else {
            "stress.json"
        });
        let input_bytes = fs::read(&input_path)?;
        let episodes: Vec<Episode> = serde_json::from_slice(&input_bytes)?;
        let previous_bytes = fs::read(a.parent.join(name).join("rows.json"))?;
        let previous: Vec<Value> = serde_json::from_slice(&previous_bytes)?;
        if episodes.len() != 128 || previous.len() != 128 {
            return Err(invalid("fixed128-row parent panels required"));
        }
        let mut rows = Vec::new();
        let mut counts = [0usize; 3];
        let mut parent_changes = 0;
        let mut native_changes = 0;
        let mut maximum_actual_logit_delta = 0f32;
        for (index, episode) in episodes.iter().take(a.rows_per_panel).enumerate() {
            if started.elapsed().as_secs() >= a.maximum_seconds {
                complete = false;
                break;
            }
            if episode.ids.is_empty()
                || episode.ids.len() > 128
                || episode.query + 1 != episode.ids.len()
                || previous[index]["episode"] != serde_json::to_value(episode)?
            {
                return Err(invalid("actual parent episode identity/layout differs"));
            }
            let t = episode.ids.len();
            let (baseline, baseline_read, _) = model
                .forward_geometric_context_learned_values_native_with_trace(
                    &episode.ids,
                    1,
                    t,
                    &context,
                    &events,
                    &span,
                    &potential,
                    &reducer,
                    &values,
                    false,
                )?;
            let (source, source_null) = model.forward_geometric_no_read(
                &episode.ids,
                1,
                t,
                &context,
                &events,
                &span,
                &potential,
                &reducer,
                &values,
                &weights,
                false,
            )?;
            if name == "original" && index == 0 {
                let target = Tensor::from_vec(vec![episode.answer], 1, &Device::Cpu)?;
                let loss =
                    candle_nn::loss::cross_entropy(&source.narrow(0, episode.query, 1)?, &target)?;
                let gradients = loss.backward()?;
                let coefficient = weights
                    .parameters()
                    .get("coefficients")
                    .ok_or_else(|| invalid("NoRead coefficient variable absent"))?;
                let gradient = gradients
                    .get(coefficient.as_tensor())
                    .ok_or_else(|| invalid("actual ordinary-answer NoRead gradient disconnected"))?
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                if gradient.iter().any(|x| !x.is_finite()) {
                    return Err(invalid("nonfinite NoRead answer gradient"));
                }
                let cfg = *weights.config();
                let widths = [
                    ("bias", 1),
                    ("token", cfg.vocabulary),
                    ("retained_root", cfg.lanes() * 4),
                    ("category", cfg.lanes() * 33),
                    ("held_root", cfg.lanes() * 4),
                    ("held_valid", cfg.lanes()),
                ];
                let mut heads = Vec::new();
                for h in 0..cfg.heads {
                    let mut at = h * cfg.coefficients_per_head();
                    let mut families = serde_json::Map::new();
                    for (name, width) in widths {
                        let norm = gradient[at..at + width]
                            .iter()
                            .map(|x| f64::from(*x).powi(2))
                            .sum::<f64>()
                            .sqrt();
                        families.insert(name.into(), json!(norm));
                        at += width;
                    }
                    heads.push(families);
                }
                answer_gradient = json!({"episode_index":0,"answer_ce":loss.to_scalar::<f32>()?,
                    "per_head_family_l2":heads,"updates":0,
                    "scope":"one actual saved-parent answer backward; saturation can yield small credit; no fit or general gradient guarantee"});
            }
            let (native, native_read, _) = model.forward_geometric_no_read_native_with_trace(
                &episode.ids,
                1,
                t,
                &context,
                &events,
                &span,
                &potential,
                &reducer,
                &values,
                &compiled,
                false,
            )?;
            let (bp, bl) = answer(&baseline, episode.query)?;
            let (sp, sl) = answer(&source, episode.query)?;
            let (np, nl) = answer(&native, episode.query)?;
            let prior = previous[index]["native_prediction"]
                .as_u64()
                .ok_or_else(|| invalid("parent prediction absent"))? as u32;
            parent_changes += usize::from(bp != prior);
            native_changes += usize::from(sp != np);
            for (count, prediction) in counts.iter_mut().zip([bp, sp, np]) {
                *count += usize::from(prediction == episode.answer);
            }
            let s = source.flatten_all()?.to_vec1::<f32>()?;
            let n = native.flatten_all()?.to_vec1::<f32>()?;
            if s.len() != n.len() {
                return Err(invalid("source/native logit shape differs"));
            }
            for (&x, &y) in s.iter().zip(&n) {
                if !x.is_finite() || !y.is_finite() {
                    return Err(invalid("nonfinite comparator logit"));
                }
                maximum_actual_logit_delta = maximum_actual_logit_delta.max((x - y).abs());
            }
            rows.push(
                json!({"index":index,"episode":episode,"baseline_prediction":bp,
                "source_prediction":sp,"native_prediction":np,"parent_prediction":prior,
                "answer_logits":{"baseline":bl,"source":sl,"native":nl},
                "source_null_nats":source_null.to_vec3::<f32>()?,
                "baseline_null_q24_all_positions":baseline_read.no_read_q24,
                "native_null_q24_all_positions":native_read.no_read_q24,
                "baseline_query_read":query_trace(&baseline_read,episode.query)?,
                "native_query_read":query_trace(&native_read,episode.query)?}),
            );
        }
        write(&a.out.join(format!("{name}-rows.json")), &rows)?;
        panels.push(json!({"panel":name,"scored_rows":rows.len(),"requested_rows":a.rows_per_panel,
            "answers":{"legacy":counts[0],"source":counts[1],"native":counts[2]},
            "legacy_changed_from_saved_parent":parent_changes,"source_native_answer_changes":native_changes,
            "maximum_actual_position_logit_delta":maximum_actual_logit_delta,
            "input_sha256":sha256_bytes(&input_bytes),"parent_rows_sha256":sha256_bytes(&previous_bytes)}));
        if !complete {
            break;
        }
    }
    write(
        &a.out.join("report.json"),
        &json!({"schema":"uor-r4.geometric-no-read-construction/1",
        "complete":complete,"updates":0,"initialization":if a.no_read_source.is_some(){"saved-source"}else{"all-zero-q4"},
        "parent_report_sha256":sha256_bytes(&parent_report_bytes),"panels":panels,
        "actual_answer_gradient":answer_gradient,
        "no_read_metadata":compiled.metadata(),"elapsed_seconds":started.elapsed().as_secs_f64(),
        "scope":"No fit. Loaded source real-softmax versus native LUT/Q24/Q16 composition; surrounding trunk/output remain float. Zero coefficients and positive-record grammar do not qualify learned quality or semantic abstention."}),
    )?;
    report_output::seal(&a.out)?;
    report_output::verify(&a.out)?;
    println!(
        "NoRead no-fit comparison complete={complete}; sealed file set verified; {}",
        a.out.display()
    );
    Ok(())
}
