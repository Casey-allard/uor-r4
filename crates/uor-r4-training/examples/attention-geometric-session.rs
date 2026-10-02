//! Exact persistent reader comparison on both retained strict-q4 context lineages.
//! The unchanged Stack tail remains floating and outside the reader contract.
use candle_core::Device;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{fs, path::PathBuf, time::Instant};
use uor_r4_core::report_output;
use uor_r4_training::{
    geometric_attention_native::CompiledGeometricAttention,
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Args {
    parent_arguments: PathBuf,
    context_attempt: PathBuf,
    out: PathBuf,
    maximum_seconds: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ParentArgs {
    model: PathBuf,
    parent: PathBuf,
    event_source: PathBuf,
    event_native: PathBuf,
    span_native: PathBuf,
    potential_native: PathBuf,
    no_read_fit: PathBuf,
    composition_fit: PathBuf,
    value_fit: PathBuf,
    potential_parent: PathBuf,
    out: PathBuf,
    seed: u64,
    maximum_seconds: u64,
}
fn invalid(message: impl Into<String>) -> uor_r4_training::TrainingError {
    uor_r4_training::TrainingError::Invalid(message.into())
}
fn write(path: &std::path::Path, value: &impl serde::Serialize) -> Result<()> {
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
fn best(row: &[f32]) -> Result<usize> {
    if row.is_empty() || row.iter().any(|v| !v.is_finite()) {
        return Err(invalid("finite nonempty logit row required"));
    }
    let mut selected = 0;
    for i in 1..row.len() {
        if row[i] > row[selected] {
            selected = i;
        }
    }
    Ok(selected)
}
fn main() -> Result<()> {
    let arg = std::env::args()
        .nth(1)
        .ok_or_else(|| invalid("argument JSON required"))?;
    let bytes = fs::read(arg)?;
    let args: Args = serde_json::from_slice(&bytes)?;
    if !(61..=840).contains(&args.maximum_seconds) {
        return Err(invalid("explicit61..840-second comparison limit required"));
    }
    report_output::claim(&args.out)?;
    let at = Instant::now();
    fs::write(args.out.join("arguments.json"), bytes)?;
    let result = run(&args, at);
    if let Err(error) = &result {
        write(
            &args.out.join("failure.json"),
            &json!({"error":error.to_string(),
            "elapsed_seconds":at.elapsed().as_secs_f64(),"decision":"INCOMPLETE_RETAIN_NO_QUALITY_VERDICT"}),
        )?;
    }
    report_output::seal(&args.out)?;
    report_output::verify(&args.out)?;
    result
}
fn run(a: &Args, at: Instant) -> Result<()> {
    report_output::verify(&a.context_attempt)?;
    let parent_bytes = fs::read(&a.parent_arguments)?;
    if parent_bytes != fs::read(a.context_attempt.join("arguments.json"))? {
        return Err(invalid(
            "comparison parent arguments differ from sealed construction",
        ));
    }
    let parent: ParentArgs = serde_json::from_slice(&parent_bytes)?;
    let context_report = fs::read(a.context_attempt.join("report.json"))?;
    let report: Value = serde_json::from_slice(&context_report)?;
    let expected = match parent.seed {
        1 => "c0d4ca673dea64f9177360e9d96f0ff67685b7c12741cd30653c3ac117823c25",
        2 => "8c70ee949778d2e2f85604dc518bef21961388e72061fd9039b9cae0fdc55eeb",
        _ => return Err(invalid("retained seed1/2 required")),
    };
    if sha256_bytes(&context_report) != expected
        || report["complete"] != true
        || report["optimizer_updates"] != 0
        || report["seed"] != parent.seed
    {
        return Err(invalid("completed strict-context parent identity differs"));
    }
    // These unused historical argument fields remain part of the pinned packet;
    // no artifact is compiled or rebound during this comparison.
    let _historical = (
        &parent.parent,
        &parent.potential_native,
        &parent.out,
        parent.maximum_seconds,
    );
    let model = StackModel::load(&parent.model, &Device::Cpu)?;
    let tokenizer = fs::read(parent.span_native.join("tokenizer-identity.bin"))?;
    let event = CompiledEvents::load(
        &parent.event_native,
        &parent.event_source,
        &parent.model,
        &tokenizer,
    )?;
    let span = CompiledSpanActions::load(
        &parent.span_native,
        &SpanSourceBinding::from_files(
            &parent.model.join("model.safetensors"),
            &parent.model.join("config.json"),
            &tokenizer,
        )?,
        model
            .geometric_span()
            .ok_or_else(|| invalid("saved span configuration missing"))?,
    )?;
    let pn = parent.potential_parent.join("potential-q4-native");
    let potential = CompiledGeometricPotentials::load(
        &pn,
        &PotentialSourceBinding::from_directory(&parent.model, &tokenizer)?,
    )?;
    let cs = a.context_attempt.join("rebound-context-source");
    let cn = a.context_attempt.join("rebound-context-native");
    let rn = a.context_attempt.join("rebound-native-read");
    let vn = a.context_attempt.join("rebound-value-native");
    let nn = a.context_attempt.join("rebound-no-read-native");
    let bn = a.context_attempt.join("rebound-composition-native");
    let vs = parent.value_fit.join("q4-value-source");
    let ns = parent.no_read_fit.join("no-read-source");
    let bs = parent.composition_fit.join("composition-source");
    let dependencies = ContextSourcePaths {
        base: &parent.model,
        event_source: &parent.event_source,
        event_native: &parent.event_native,
        span_native: &parent.span_native,
        potential_native: &pn,
    };
    let context = CompiledContext::load(&cn, &cs, dependencies, &tokenizer)?;
    if serde_json::to_value(context.metadata())? != report["compiled_metadata"] {
        return Err(invalid(
            "loaded context identity differs from the retained measured parent",
        ));
    }
    let reducer = CompiledGeometricRead::load(
        &rn,
        &ReadSourceBinding::from_directory(&parent.model, &tokenizer, &potential, 2)?,
    )?;
    let vp = ValueProducerSourcePaths {
        value_source: &vs,
        context_source: &cs,
        context_dependencies: dependencies,
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
    let admitted = CompiledGeometricAttention::new(
        &context,
        &event,
        &span,
        &potential,
        &reducer,
        &values,
        &null,
        &composition,
    )?;
    admitted.validate_stack(&model)?;
    // A real valid q4 value artifact from the preceding context lineage must
    // not be silently admitted against this changed context metadata.
    let previous_cs = parent.potential_parent.join("rebound-context-source");
    let previous_values = CompiledValueProducer::load(
        &parent.potential_parent.join("rebound-value-native"),
        ValueProducerSourcePaths {
            value_source: &vs,
            context_source: &previous_cs,
            context_dependencies: dependencies,
        },
        &tokenizer,
    )?;
    if CompiledGeometricAttention::new(
        &context,
        &event,
        &span,
        &potential,
        &reducer,
        &previous_values,
        &null,
        &composition,
    )
    .is_ok()
    {
        return Err(invalid(
            "different valid value/context lineage was admitted",
        ));
    }
    let mut panels = Vec::new();
    let mut interface_checks = None;
    for (panel_index, name) in ["original", "stress"].into_iter().enumerate() {
        let bytes = fs::read(a.context_attempt.join(format!("{name}-episodes.json")))?;
        let episodes: Vec<data::Episode> = serde_json::from_slice(&bytes)?;
        let saved: Vec<Value> = serde_json::from_slice(&fs::read(
            a.context_attempt.join(format!("{name}-rows.json")),
        )?)?;
        if episodes.len() != 128
            || saved.len() != 128
            || report["panels"][panel_index]["panel"] != name
            || report["panels"][panel_index]["input_sha256"] != sha256_bytes(&bytes)
        {
            return Err(invalid("all128 retained episodes required"));
        }
        fs::write(a.out.join(format!("{name}-episodes.json")), &bytes)?;
        let mut rows = Vec::new();
        let mut correct = 0;
        for (index, episode) in episodes.iter().enumerate() {
            if at.elapsed().as_secs() >= a.maximum_seconds {
                return Err(invalid(
                    "declared comparison deadline reached; retain incomplete attempt",
                ));
            }
            let time = episode.ids.len();
            if time == 0
                || time > 128
                || episode.query >= time
                || episode.answer as usize >= model.config.vocab_size
                || saved[index]["episode"] != serde_json::to_value(episode)?
            {
                return Err(invalid("retained episode bounds/identity differ"));
            }
            let (baseline, baseline_read, baseline_values) = model
                .forward_geometric_composition_native_with_trace(
                    &episode.ids,
                    1,
                    time,
                    &context,
                    &event,
                    &span,
                    &potential,
                    &reducer,
                    &values,
                    &null,
                    &composition,
                    false,
                )?;
            let exact = compare_rows(&episode.ids, &admitted, &baseline_read, &baseline_values)?;
            let session =
                model.forward_geometric_attention_session(&episode.ids, 1, time, &admitted)?;
            let baseline_logits = baseline.to_vec2::<f32>()?;
            let session_logits = session.to_vec2::<f32>()?;
            if baseline_logits.len() != session_logits.len()
                || baseline_logits.iter().zip(&session_logits).any(|(x, y)| {
                    x.len() != y.len() || x.iter().zip(y).any(|(a, b)| a.to_bits() != b.to_bits())
                })
            {
                return Err(invalid(format!(
                    "{name} row{index} actual-position logit bits differ"
                )));
            }
            let answer = best(&session_logits[episode.query])?;
            if saved[index]["native_prediction"].as_u64() != Some(answer as u64) {
                return Err(invalid(
                    "session answer differs from retained context parent",
                ));
            }
            let saved_answer: Vec<f32> =
                serde_json::from_value(saved[index]["answer_logits"]["native"].clone())?;
            if saved_answer.len() != session_logits[episode.query].len()
                || saved_answer
                    .iter()
                    .zip(&session_logits[episode.query])
                    .any(|(x, y)| x.to_bits() != y.to_bits())
            {
                return Err(invalid(
                    "session answer-logit bits differ from retained context parent",
                ));
            }
            if interface_checks.is_none() {
                let short = &episode.ids[..time / 2];
                let (prefix, _, _) = model.forward_geometric_composition_native_with_trace(
                    short,
                    1,
                    short.len(),
                    &context,
                    &event,
                    &span,
                    &potential,
                    &reducer,
                    &values,
                    &null,
                    &composition,
                    false,
                )?;
                let prefix_session =
                    model.forward_geometric_attention_session(short, 1, short.len(), &admitted)?;
                require_logit_bits(&prefix.to_vec2::<f32>()?, &prefix_session.to_vec2::<f32>()?)?;
                let other = &episodes[(index + 1) % episodes.len()].ids;
                let batch_time = time.min(other.len());
                let duplicate = episode.ids[..batch_time]
                    .iter()
                    .chain(&other[..batch_time])
                    .copied()
                    .collect::<Vec<_>>();
                let (batched, _, _) = model.forward_geometric_composition_native_with_trace(
                    &duplicate,
                    2,
                    batch_time,
                    &context,
                    &event,
                    &span,
                    &potential,
                    &reducer,
                    &values,
                    &null,
                    &composition,
                    false,
                )?;
                let batched_session = model
                    .forward_geometric_attention_session(&duplicate, 2, batch_time, &admitted)?;
                require_logit_bits(
                    &batched.to_vec2::<f32>()?,
                    &batched_session.to_vec2::<f32>()?,
                )?;
                interface_checks = Some(
                    json!({"short_prefix_tokens":short.len(),"batch":2,"batch_time":batch_time,
                    "same_shape_native_reference_logits_bits_equal":true}),
                );
            }
            correct += usize::from(answer == episode.answer as usize);
            rows.push(json!({"index":index,"episode":episode,"prediction":answer,
                "all_actual_position_logits_bits_equal":true,"answer_logits":session_logits[episode.query],
                "exact_reader_comparison":exact}));
        }
        write(&a.out.join(format!("{name}-rows.json")), &rows)?;
        panels.push(json!({"panel":name,"rows":rows.len(),"correct":correct,
            "episodes_sha256":sha256_bytes(&bytes),"all_actual_position_logits_bits_equal":true}));
    }
    write(
        &a.out.join("report.json"),
        &json!({"schema":"uor-r4.geometric-attention-session-comparison/1",
        "complete":true,"seed":parent.seed,"optimizer_updates":0,"parent_report_sha256":expected,
        "parent_arguments_sha256":sha256_bytes(&parent_bytes),
        "admission":admitted.metadata(),"panels":panels,"elapsed_seconds":at.elapsed().as_secs_f64(),
        "session_storage":admitted.session(128)?.storage_bytes(),"stack_interface_checks":interface_checks,
        "mismatched_valid_value_context_dependency_refused":true,
        "scope":"Exact persistent integer reader and unchanged float Stack-tail comparison on exposed retained panels; no model-quality improvement, all-q4 coefficient, whole-serving, sparse-access or energy qualification"}),
    )?;
    Ok(())
}

fn require_logit_bits(left: &[Vec<f32>], right: &[Vec<f32>]) -> Result<()> {
    if left.len() != right.len()
        || left.iter().zip(right).any(|(x, y)| {
            x.len() != y.len() || x.iter().zip(y).any(|(a, b)| a.to_bits() != b.to_bits())
        })
    {
        return Err(invalid(
            "same-shape native/persistent Stack interface logits differ",
        ));
    }
    Ok(())
}

fn compare_rows(
    ids: &[u32],
    admitted: &CompiledGeometricAttention<'_>,
    baseline: &uor_r4_training::geometric_composition_native::ComposedReadTrace,
    values: &uor_r4_training::geometric_value_producer::ValueProducerTrace,
) -> Result<Value> {
    use uor_r4_integer::{geometric_potential::AddressLane, geometric_value::ValueState};
    let time = ids.len();
    let events =
        uor_r4_training::geometric_event::trace_native(ids, 1, time, admitted.events(), false)?;
    let context =
        uor_r4_training::geometric_context::trace_native(ids, 1, time, admitted.context(), false)?;
    let held = uor_r4_training::geometric_span_native::trace_native_events(
        ids,
        1,
        time,
        &events.actions,
        admitted.span(),
    )?;
    let addresses = &context.codes;
    let mut content = Vec::with_capacity(time * 8);
    for occurrence in &held.prior_codes {
        for lane in 0..8 {
            let address = match occurrence {
                Some(codes) => AddressLane::new(codes[lane], 16, true),
                None => AddressLane::new(1, 0, false),
            }
            .map_err(|e| invalid(e.to_string()))?;
            content.push(address);
        }
    }
    let mut session = admitted.session(time)?;
    let mut trace = Vec::new();
    let mut score_pairs = 0;
    for (q, &token) in ids.iter().enumerate() {
        let step = session
            .push(token as usize)
            .map_err(|e| invalid(e.to_string()))?;
        let event_states = step.event.states[..events.lanes]
            .iter()
            .map(|c| c.index())
            .collect::<Vec<_>>();
        let event_actions = step.event.actions[..events.lanes]
            .iter()
            .map(|c| c.index())
            .collect::<Vec<_>>();
        let ctx_states = step.context.states[..8]
            .iter()
            .map(|c| c.index())
            .collect::<Vec<_>>();
        let ctx_actions = step.context.actions[..8]
            .iter()
            .map(|c| c.index())
            .collect::<Vec<_>>();
        let roots = step.context.readout_roots[..8]
            .iter()
            .map(|c| c.index())
            .collect::<Vec<_>>();
        let old_held = step
            .held_before
            .map(|codes| codes.iter().map(|c| c.index()).collect::<Vec<_>>());
        let independently_produced = admitted.values().produce(
            token as usize,
            &step.context.states,
            step.held_before.map(|codes| codes.as_slice()),
            true,
        )?;
        if step.values.root_choices != independently_produced.root_choices
            || step.values.categories != independently_produced.categories
        {
            return Err(invalid("session raw value root/category choices differ"));
        }
        if step.position != q
            || event_states != events.states[q]
            || event_actions != events.transition_actions[q]
            || step.event.event != events.actions[q]
            || step.event.event_scores != events.event_scores[q]
            || ctx_states != context.states[q]
            || ctx_actions != context.actions[q]
            || roots != context.emitted_roots[q * 8..(q + 1) * 8]
            || step.context.categories[..8] != context.categories[q * 8..(q + 1) * 8]
            || old_held != held.prior_codes[q]
            || step.content_codes[..] != content[q * 8..(q + 1) * 8]
            || step.context_codes[..] != addresses[q * 8..(q + 1) * 8]
            || !step.values.occurrence_valid
            || !values.occurrence_valid[q]
        {
            return Err(invalid(format!(
                "session row{q} state/observation/old-held/occurrence differs"
            )));
        }
        let mut heads = Vec::new();
        for h in 0..2 {
            let row = &baseline.rows[h * time + q];
            let current = &step.heads[h];
            if current.output_q16[..] != row.output_q16
                || current.occurrence_weights_q31 != row.occurrence_weights_q31
                || current.no_read_q24 != baseline.no_read_q24[h * time + q]
                || current.no_read_weight_q31 != row.no_read_weight_q31
                || current.total_weight_q31 != row.total_weight_q31
                || current.max_score_q24 != row.max_score_q24
                || step.composed_q16[h][..]
                    != baseline.values_q16[(h * time + q) * 32..(h * time + q + 1) * 32]
                || step.values.values_q16[h * 16..(h + 1) * 16]
                    != values.values_q16[(h * time + q) * 16..(h * time + q + 1) * 16]
            {
                return Err(invalid(format!(
                    "session row{q} head{h} payload/reduction differs"
                )));
            }
            for lane in 0..4 {
                for atom in 0..2 {
                    let actual = step.values.packets[h * 4 + lane][atom];
                    let prior = &values.packets[(h * time + q) * 4 + lane][atom];
                    let status = match actual.state() {
                        ValueState::Absent => uor_r4_training::geometric_value_native::ValuePacketStatus::Absent,
                        ValueState::PresentZero => uor_r4_training::geometric_value_native::ValuePacketStatus::PresentZero,
                        ValueState::PresentNonzero => uor_r4_training::geometric_value_native::ValuePacketStatus::PresentNonzero,
                    };
                    if status != prior.status
                        || actual.root().index() != prior.root
                        || actual.radius_bin() != prior.radius_bin
                    {
                        return Err(invalid(format!("session row{q} head{h} packet differs")));
                    }
                }
            }
            for k in 0..=q {
                let qo = q * 8 + h * 4;
                let ko = k * 8 + h * 4;
                let score = admitted.potential().score_pair_codes(
                    h,
                    &content[qo..qo + 4],
                    &content[ko..ko + 4],
                    &addresses[qo..qo + 4],
                    &addresses[ko..ko + 4],
                )?;
                let age =
                    admitted.reducer().age_q24()[h * admitted.reducer().metadata().context + q - k];
                if current.potential_q24[k] != score || current.age_q24[k] != age {
                    return Err(invalid(format!(
                        "session row{q} head{h} occurrence{k} score/age differs"
                    )));
                }
                score_pairs += 1;
            }
            heads.push(json!({"potential_q24":current.potential_q24,"age_q24":current.age_q24,
                "no_read_q24":current.no_read_q24,"occurrence_weights_q31":current.occurrence_weights_q31,
                "no_read_weight_q31":current.no_read_weight_q31,"total_weight_q31":current.total_weight_q31,
                "max_score_q24":current.max_score_q24,"output_q16":current.output_q16,
                "composed_payload_q16":step.composed_q16[h]}));
        }
        for c in 0..32 {
            let sum = baseline.rows[q].output_q16[c]
                .checked_add(baseline.rows[time + q].output_q16[c])
                .ok_or_else(|| invalid("baseline integer sum overflow"))?;
            if step.output_q16[c] != sum {
                return Err(invalid("session checked head sum differs"));
            }
        }
        trace.push(json!({"position":q,"event_states":event_states,"event_actions":event_actions,
            "event":step.event.event as u8,"event_scores_q24":step.event.event_scores,
            "context_states":ctx_states,"context_actions":ctx_actions,"raw_observation_roots":roots,
            "observation_categories":&step.context.categories[..8],"old_held":old_held,"heads":heads,
            "value_raw_roots":step.values.root_choices.as_slice(),
            "value_categories":step.values.categories.as_slice(),
            "output_q16":step.output_q16}));
    }
    // Rejecting a full session or invalid token must retain the public output;
    // reset then replays the exact same first occurrence.
    let last = session
        .last()
        .ok_or_else(|| invalid("successful session has no publication"))?
        .output_q16
        .to_vec();
    if session.push(ids[0] as usize).is_ok()
        || session.len() != time
        || session
            .last()
            .ok_or_else(|| invalid("capacity error cleared last"))?
            .output_q16[..]
            != last
    {
        return Err(invalid("capacity rejection changed committed session"));
    }
    session.reset();
    if session.len() != 0 || session.last().is_some() {
        return Err(invalid("reset retained publication/history"));
    }
    if session.push(usize::MAX).is_ok() || session.len() != 0 || session.last().is_some() {
        return Err(invalid("invalid token changed reset session"));
    }
    let replay = session
        .push(ids[0] as usize)
        .map_err(|e| invalid(e.to_string()))?;
    if replay.output_q16[..]
        != trace[0]["output_q16"]
            .as_array()
            .ok_or_else(|| invalid("saved reset reference"))?
            .iter()
            .map(|v| v.as_i64().ok_or_else(|| invalid("saved reset coordinate")))
            .collect::<Result<Vec<_>>>()?
    {
        return Err(invalid("reset replay differs"));
    }
    Ok(
        json!({"tokens":time,"causal_score_pairs":score_pairs,"compared_reader_fields_exact":true,
        "capacity_invalid_token_reset_preserved":true,"rows":trace}),
    )
}
