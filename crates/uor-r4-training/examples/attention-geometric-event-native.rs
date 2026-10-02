//! Native geometric context/event learning in the existing geometric reader.
//! Authored grammar and frozen base; no natural-language or full serving claim.
use candle_core::{DType, Device, Tensor};
use candle_nn::{AdamW, Optimizer, ParamsAdamW};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fs, path::Path, time::Instant};
use uor_r4_core::report_output;
use uor_r4_training::geometric_event::{self, CompiledEvents, EventWeights};
use uor_r4_training::geometric_potential_native::{
    CompiledGeometricPotentials, PotentialSourceBinding,
};
use uor_r4_training::geometric_span_native::{
    trace_native_events, CompiledSpanActions, SpanSourceBinding,
};
use uor_r4_training::geometric_stack::{
    logits_cross_entropy, ReadBinding, ReadBindingTarget, StackModel,
};
use uor_r4_training::{sha256_file, Result, TrainingError};
fn invalid(s: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(s.into())
}
struct Rng(u64);
impl Rng {
    fn next(&mut self, n: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % n as u64) as usize
    }
}
#[derive(Clone, Serialize, Deserialize)]
struct Episode {
    ids: Vec<u32>,
    roles: Vec<String>,
    source: usize,
    query: usize,
    answer: u32,
    facts: usize,
    pair: usize,
    condition: String,
    write_gaps: Vec<usize>,
    query_gap: usize,
    target_write_gap: usize,
    query_span: Vec<u32>,
}
fn push(ids: &mut Vec<u32>, roles: &mut Vec<String>, id: u32, role: &str) {
    ids.push(id);
    roles.push(role.into());
}
fn episode(
    facts: &[(Vec<u32>, u32)],
    write_noise: &[Vec<u32>],
    query_key: &[u32],
    query_noise: &[u32],
    pair: usize,
    condition: &str,
) -> Episode {
    let (mut ids, mut roles) = (Vec::new(), Vec::new());
    push(&mut ids, &mut roles, 36, "bos");
    let (mut source, mut answer) = (0, 0);
    let mut target_write_gap = 0;
    for (i, (key, value)) in facts.iter().enumerate() {
        push(&mut ids, &mut roles, 32, "write_open");
        for &word in key {
            push(&mut ids, &mut roles, word, "write_key");
        }
        push(&mut ids, &mut roles, 39, "write_commit");
        for &noise in &write_noise[i] {
            push(&mut ids, &mut roles, 37, "noise_marker");
            push(&mut ids, &mut roles, noise, "noise_key");
        }
        push(&mut ids, &mut roles, 35, "value_marker");
        push(&mut ids, &mut roles, *value, "value");
        if key.as_slice() == query_key {
            source = ids.len() - 1;
            answer = *value;
            target_write_gap = write_noise[i].len();
        }
    }
    push(&mut ids, &mut roles, 33, "query_open");
    for &word in query_key {
        push(&mut ids, &mut roles, word, "query_key");
    }
    push(&mut ids, &mut roles, 39, "query_commit");
    for &noise in query_noise {
        push(&mut ids, &mut roles, 37, "noise_marker");
        push(&mut ids, &mut roles, noise, "noise_key");
    }
    push(&mut ids, &mut roles, 34, "answer_marker");
    let query = ids.len() - 1;
    Episode {
        ids,
        roles,
        source,
        query,
        answer,
        facts: facts.len(),
        pair,
        condition: condition.into(),
        write_gaps: write_noise.iter().map(Vec::len).collect(),
        query_gap: query_noise.len(),
        target_write_gap,
        query_span: query_key.to_vec(),
    }
}
fn draw(rng: &mut Rng, n: usize) -> Vec<(Vec<u32>, u32)> {
    let mut words: Vec<u32> = (0..16).collect();
    for i in (1..16).rev() {
        let j = rng.next(i + 1);
        words.swap(i, j);
    }
    let mut facts = Vec::new();
    for pair in 0..n / 2 {
        let (a, b) = (words[2 + 2 * pair], words[3 + 2 * pair]);
        let first = 16 + rng.next(16) as u32;
        let second = 16 + ((first - 16 + 1 + rng.next(15) as u32) % 16);
        facts.push((vec![words[0], a, b, words[1]], first));
        facts.push((vec![words[0], b, a, words[1]], second));
    }
    facts
}
fn noise(rng: &mut Rng, max: usize) -> Vec<u32> {
    let n = rng.next(max + 1);
    (0..n).map(|_| rng.next(16) as u32).collect()
}
fn evaluation(seed: u64, max_gap: usize) -> Vec<Episode> {
    let mut rng = Rng(seed);
    let mut out = Vec::new();
    for n in [2, 4] {
        for p in 0..16 {
            let mut facts = draw(&mut rng, n);
            let q = rng.next(n);
            if p % 8 == 0 {
                facts[q ^ 1].1 = facts[q].1;
            }
            let wn: Vec<_> = (0..n).map(|_| noise(&mut rng, max_gap)).collect();
            let qn = noise(&mut rng, max_gap);
            let pair = n * 100 + p;
            out.push(episode(&facts, &wn, &facts[q].0, &qn, pair, "base"));
            out.push(episode(
                &facts,
                &wn,
                &facts[q ^ 1].0,
                &qn,
                pair,
                "changed_query",
            ));
            let mut changed = facts.clone();
            changed[q].1 = facts[q ^ 1].1;
            changed[q ^ 1].1 = facts[q].1;
            out.push(episode(
                &changed,
                &wn,
                &facts[q].0,
                &qn,
                pair,
                "swapped_values",
            ));
            let mut reversed = facts.clone();
            reversed.reverse();
            let mut rn = wn.clone();
            rn.reverse();
            out.push(episode(
                &reversed,
                &rn,
                &facts[q].0,
                &qn,
                pair,
                "reversed_order",
            ));
        }
    }
    out
}
fn batch(es: &[Episode]) -> (Vec<u32>, Vec<u32>, Vec<f32>, usize) {
    let time = es.iter().map(|e| e.ids.len()).max().unwrap_or(0);
    let (mut ids, mut targets, mut weights) = (
        vec![38; es.len() * time],
        vec![0; es.len() * time],
        vec![0.; es.len() * time],
    );
    for (b, e) in es.iter().enumerate() {
        ids[b * time..b * time + e.ids.len()].copy_from_slice(&e.ids);
        targets[b * time + e.query] = e.answer;
        weights[b * time + e.query] = 1.;
    }
    (ids, targets, weights, time)
}
fn binding(es: &[Episode], head: usize) -> ReadBindingTarget {
    ReadBindingTarget {
        layer: 2,
        head,
        rows: es
            .iter()
            .enumerate()
            .map(|(batch, e)| ReadBinding {
                batch,
                query: e.query,
                sources: vec![e.source],
            })
            .collect(),
    }
}

fn action_for(role: &str) -> u32 {
    match role {
        "write_open" | "query_open" => 1,
        "write_key" | "query_key" => 2,
        "write_commit" | "query_commit" => 3,
        _ => 0,
    }
}
fn event_credit(events: &EventWeights, ids: &[u32], es: &[Episode], time: usize) -> Result<Tensor> {
    let mut counts = [0usize; 4];
    for e in es {
        for role in &e.roles {
            counts[action_for(role) as usize] += 1;
        }
    }
    if counts.contains(&0) {
        return Err(invalid("all four event classes required"));
    }
    let mut targets = vec![0; ids.len()];
    let mut weights = vec![0.; ids.len()];
    for (b, e) in es.iter().enumerate() {
        for (t, role) in e.roles.iter().enumerate() {
            let a = action_for(role);
            targets[b * time + t] = a;
            weights[b * time + t] = 0.25 / counts[a as usize] as f32;
        }
    }
    let logits = events
        .event_logits(ids, es.len(), time)?
        .reshape((ids.len(), 4))?;
    logits_cross_entropy(&logits, &targets, Some(&weights))
}
fn gradient_norm(events: &EventWeights, grads: &candle_core::backprop::GradStore) -> Result<f64> {
    let mut sum = 0.;
    for v in events.parameters().values() {
        if let Some(g) = grads.get(v.as_tensor()) {
            sum += f64::from(g.sqr()?.sum_all()?.to_scalar::<f32>()?);
        }
    }
    let norm = sum.sqrt();
    if !norm.is_finite() {
        return Err(invalid("nonfinite event gradient"));
    }
    Ok(norm)
}
fn clip(
    events: &EventWeights,
    grads: &mut candle_core::backprop::GradStore,
    norm: f64,
) -> Result<()> {
    if norm > 1. {
        for v in events.parameters().values() {
            if let Some(g) = grads.get(v.as_tensor()) {
                let scaled = g.affine(1. / norm, 0.)?;
                grads.insert(v.as_tensor(), scaled);
            }
        }
    }
    Ok(())
}
fn argmax(row: &[f32]) -> Result<u32> {
    if row.is_empty() || row.iter().any(|v| !v.is_finite()) {
        return Err(invalid("finite nonempty logits required"));
    }
    let mut best = 0;
    for i in 1..row.len() {
        if row[i] > row[best] {
            best = i;
        }
    }
    Ok(best as u32)
}
fn valid_masses(values: &[f32], count: usize) -> Result<()> {
    if values.len() != count || values.iter().any(|x| !x.is_finite() || *x < 0. || *x > 1.) {
        return Err(invalid("source probability shape/range"));
    }
    Ok(())
}
fn native_trace_json(trace: &geometric_event::NativeEventTrace) -> Value {
    json!({
        "batch": trace.batch,
        "time": trace.time,
        "lanes": trace.lanes,
        "actions": trace.actions.iter().map(|action| *action as u8).collect::<Vec<_>>(),
        "transition_actions": trace.transition_actions,
        "states": trace.states,
        "event_scores": trace.event_scores,
        "coefficient_reads": trace.coefficient_reads,
    })
}
fn validate_code_trace(
    states: &[Vec<u8>],
    actions: &[Vec<u8>],
    count: usize,
    lanes: usize,
) -> Result<()> {
    if states.len() != count
        || actions.len() != count
        || states
            .iter()
            .chain(actions)
            .any(|row| row.len() != lanes || row.iter().any(|&c| c >= 120))
    {
        return Err(invalid("context trace shape/code mismatch"));
    }
    Ok(())
}
fn validate_output(rows: &[Vec<f32>], count: usize, vocab: usize) -> Result<()> {
    if rows.len() != count
        || rows
            .iter()
            .any(|row| row.len() != vocab || row.iter().any(|v| !v.is_finite()))
    {
        return Err(invalid("output shape/nonfinite logits"));
    }
    Ok(())
}
fn score(
    model: &StackModel,
    weights: &EventWeights,
    compiled: &CompiledEvents,
    span: &CompiledSpanActions,
    potential: &CompiledGeometricPotentials,
    es: &[Episode],
    out: &Path,
    deadline: Instant,
) -> Result<Value> {
    fs::create_dir(out)?;
    let mut rows = Vec::new();
    let mut event_rows = Vec::new();
    let mut role_counts = std::collections::BTreeMap::<String, [usize; 4]>::new();
    let mut action_disagreements = 0usize;
    let mut transition_disagreements = 0usize;
    let mut context_state_disagreements = 0usize;
    let mut old_span_code_mismatches = 0usize;
    let mut padded = 0;
    let mut actual = 0;
    for (chunk, group) in es.chunks(32).enumerate() {
        if Instant::now() >= deadline {
            break;
        }
        let (ids, _, _, time) = batch(group);
        padded += ids.len();
        actual += group.iter().map(|e| e.ids.len()).sum::<usize>();
        let float_events = weights
            .event_logits(&ids, group.len(), time)?
            .to_vec3::<f32>()?;
        let float_trace = weights.float_trace(&ids, group.len(), time, false)?;
        let native = geometric_event::trace_native(&ids, group.len(), time, compiled, false)?;
        let reset = geometric_event::trace_native(&ids, group.len(), time, compiled, true)?;
        let count = ids.len();
        let lanes = weights.config().lanes;
        validate_code_trace(&float_trace.states, &float_trace.actions, count, lanes)?;
        validate_code_trace(&native.states, &native.transition_actions, count, lanes)?;
        validate_code_trace(&reset.states, &reset.transition_actions, count, lanes)?;
        if native.actions.len() != count
            || reset.actions.len() != count
            || native.event_scores.len() != count
            || reset.event_scores.len() != count
            || float_trace.events.len() != count
            || float_trace.logits.len() != count * 4
            || float_trace.events.iter().any(|&x| x >= 4)
            || float_trace.logits.iter().any(|x| !x.is_finite())
        {
            return Err(invalid("event trace shape/finite mismatch"));
        }
        if float_events.len() != group.len()
            || float_events.iter().any(|row| {
                row.len() != time
                    || row
                        .iter()
                        .any(|x| x.len() != 4 || x.iter().any(|v| !v.is_finite()))
            })
        {
            return Err(invalid("float event tensor shape/finite mismatch"));
        }
        for (a, b) in float_trace.actions.iter().zip(&native.transition_actions) {
            transition_disagreements += a.iter().zip(b).filter(|(a, b)| a != b).count();
        }
        for (a, b) in float_trace.states.iter().zip(&native.states) {
            context_state_disagreements += a.iter().zip(b).filter(|(a, b)| a != b).count();
        }
        let prior = trace_native_events(&ids, group.len(), time, &native.actions, span)?;
        let diagnostic_logits = weights.event_logits(&ids, group.len(), time)?;
        let float_prior = uor_r4_training::geometric_span_native::trace_native(
            &ids,
            group.len(),
            time,
            &diagnostic_logits,
            span,
        )?;
        for trace in [&prior, &float_prior] {
            if trace.prior_codes.len() != count
                || trace.actions.len() != count
                || trace.prior_codes.iter().flatten().any(|codes| {
                    codes.len() != model.config.width / 4 || codes.iter().any(|&c| c >= 120)
                })
            {
                return Err(invalid("span trace shape/code mismatch"));
            }
        }
        for (a, b) in prior.prior_codes.iter().zip(&float_prior.prior_codes) {
            old_span_code_mismatches += usize::from(a != b);
        }
        let baseline = model.forward(&ids, group.len(), time)?.to_vec2::<f32>()?;
        let trained = model
            .forward_geometric_event(&ids, group.len(), time, weights, false)?
            .to_vec2::<f32>()?;
        let logits = model
            .forward_geometric_event_native(
                &ids,
                group.len(),
                time,
                compiled,
                span,
                potential,
                false,
            )?
            .to_vec2::<f32>()?;
        let reset_logits = model
            .forward_geometric_event_native(
                &ids,
                group.len(),
                time,
                compiled,
                span,
                potential,
                true,
            )?
            .to_vec2::<f32>()?;
        for output in [&baseline, &trained, &logits, &reset_logits] {
            validate_output(output, count, model.config.vocab_size)?;
        }
        let mut bm = [Vec::new(), Vec::new()];
        let mut nm = [Vec::new(), Vec::new()];
        let mut rm = [Vec::new(), Vec::new()];
        for h in 0..2 {
            let target = binding(group, h);
            bm[h] = model
                .read_binding_masses(&ids, group.len(), time, &target)?
                .to_vec1::<f32>()?;
            nm[h] = model
                .read_binding_masses_geometric_event_native(
                    &ids,
                    group.len(),
                    time,
                    &target,
                    compiled,
                    span,
                    potential,
                    false,
                )?
                .to_vec1::<f32>()?;
            rm[h] = model
                .read_binding_masses_geometric_event_native(
                    &ids,
                    group.len(),
                    time,
                    &target,
                    compiled,
                    span,
                    potential,
                    true,
                )?
                .to_vec1::<f32>()?;
            valid_masses(&bm[h], group.len())?;
            valid_masses(&nm[h], group.len())?;
            valid_masses(&rm[h], group.len())?;
        }
        let mut chunk_rows = Vec::new();
        let mut chunk_events = Vec::new();
        for (b, e) in group.iter().enumerate() {
            let at = b * time + e.query;
            let bp = argmax(&baseline[at])?;
            let np = argmax(&logits[at])?;
            let tp = argmax(&trained[at])?;
            let rp = argmax(&reset_logits[at])?;
            let row = json!({"pair":e.pair,"condition":e.condition,"query":e.query,"source":e.source,"answer":e.answer,"baseline_prediction":bp,"trained_prediction":tp,"native_prediction":np,"reset_prediction":rp,"baseline_answer_logits":baseline[at],"trained_answer_logits":trained[at],"native_answer_logits":logits[at],"reset_answer_logits":reset_logits[at],"baseline_source_masses":[bm[0][b],bm[1][b]],"native_source_masses":[nm[0][b],nm[1][b]],"reset_source_masses":[rm[0][b],rm[1][b]]});
            chunk_rows.push(row.clone());
            rows.push(row);
            for (t, role) in e.roles.iter().enumerate() {
                let at = b * time + t;
                let target = action_for(role);
                let f = argmax(&float_events[b][t])?;
                let n = native.actions[at] as u32;
                let r = reset.actions[at] as u32;
                action_disagreements += usize::from(f != n);
                let count = role_counts.entry(role.clone()).or_default();
                count[0] += 1;
                count[1] += usize::from(n == target);
                count[2] += usize::from(r == target);
                count[3] += usize::from(f == target);
                let observation = json!({"batch_row":b,"time":t,"token":e.ids[t],"role":role,"target_event":target,"float_event":f,"native_event":n,"reset_event":r,"native_event_scores":native.event_scores[at]});
                chunk_events.push(observation.clone());
                event_rows.push(observation);
            }
        }
        // Full padded integer traces are retained even though labels only cover real positions.
        fs::write(
            out.join(format!("chunk-{chunk}.json")),
            serde_json::to_vec_pretty(
                &json!({"batch":group.len(),"time":time,"ids":ids,"float_trace":float_trace,"native_trace":native_trace_json(&native),"reset_trace":native_trace_json(&reset),"span_trace":prior,"rows":chunk_rows,"events":chunk_events}),
            )?,
        )?;
    }
    let total_events = event_rows.len();
    let correct_events = event_rows
        .iter()
        .filter(|r| r["native_event"] == r["target_event"])
        .count();
    let reset_correct = event_rows
        .iter()
        .filter(|r| r["reset_event"] == r["target_event"])
        .count();
    let summary = json!({"rows":rows.len(),"expected_rows":es.len(),"complete":rows.len()==es.len(),"padded_positions":padded,"actual_positions":actual,"native_answers":rows.iter().filter(|r|r["native_prediction"]==r["answer"]).count(),"baseline_answers":rows.iter().filter(|r|r["baseline_prediction"]==r["answer"]).count(),"trained_answers":rows.iter().filter(|r|r["trained_prediction"]==r["answer"]).count(),"reset_answers":rows.iter().filter(|r|r["reset_prediction"]==r["answer"]).count(),"baseline_prediction_changes":rows.iter().filter(|r|r["native_prediction"]!=r["baseline_prediction"]).count(),"compilation_prediction_changes":rows.iter().filter(|r|r["native_prediction"]!=r["trained_prediction"]).count(),"native_head0_majorities":rows.iter().filter(|r|r["native_source_masses"][0].as_f64().is_some_and(|v|v>0.5)).count(),"native_head1_majorities":rows.iter().filter(|r|r["native_source_masses"][1].as_f64().is_some_and(|v|v>0.5)).count(),"reset_head0_majorities":rows.iter().filter(|r|r["reset_source_masses"][0].as_f64().is_some_and(|v|v>0.5)).count(),"events":total_events,"correct_events":correct_events,"reset_correct_events":reset_correct,"compilation_event_disagreements":action_disagreements,"compilation_transition_disagreements":transition_disagreements,"compilation_context_state_disagreements":context_state_disagreements,"span_code_mismatches":old_span_code_mismatches,"role_counts_total_native_reset_float":role_counts,"rows_detail":rows});
    fs::write(
        out.join("summary.json"),
        serde_json::to_vec_pretty(&summary)?,
    )?;
    Ok(summary)
}
fn run(
    parent: &Path,
    potential_root: &Path,
    out: &Path,
    steps: usize,
    max_seconds: u64,
) -> Result<Value> {
    let start = Instant::now();
    let deadline = start + std::time::Duration::from_secs(max_seconds);
    let eval: Vec<Episode> =
        serde_json::from_slice(&fs::read(parent.join("run-1/evaluation.json"))?)?;
    let stress: Vec<Episode> =
        serde_json::from_slice(&fs::read(parent.join("run-1/stress.json"))?)?;
    if eval.len() != 128
        || stress.len() != 128
        || eval
            .iter()
            .chain(&stress)
            .any(|e| e.ids.len() > 128 || e.ids.len() != e.roles.len())
    {
        return Err(invalid("pinned panel shape"));
    }
    let expected = [
        (
            "run-1/report.json",
            "60f78851dd86b0063cca537eabd50aa2a3c092bd25b29de4f6db7ae5d2ef3db5",
        ),
        (
            "run-1/evaluation.json",
            "dabdaa1a2a8dcf23e1cfe5164ef00b97b923c3b74dc09dcd1d4a307d1ff64915",
        ),
        (
            "run-1/stress.json",
            "e3c81984b7b41e0fcb329fde026318d067b6397e93832508c3ee60180a5a9d63",
        ),
    ];
    for (path, sha) in expected {
        if sha256_file(&parent.join(path))? != sha {
            return Err(invalid("parent panel/hash differs"));
        }
    }
    let mut reports = Vec::new();
    for seed in [1u64, 2] {
        if Instant::now() >= deadline {
            break;
        }
        let name = format!("GeometricSpan-Ordered-s{seed}");
        let root = out.join(&name);
        fs::create_dir(&root)?;
        let base = parent.join("run-1").join(&name).join("model");
        let model = StackModel::load(&base, &Device::Cpu)?;
        let expected_model = if seed == 1 {
            "235614f2e1861dd79e2e4ea8dac3cdaaec13243f145c04d9a77fbda3e9bb47fa"
        } else {
            "3909b1129e4cf2a2aa72c3cc7ad3172b8eb1c847b881ce054418e31f44c21873"
        };
        if sha256_file(&base.join("model.safetensors"))? != expected_model {
            return Err(invalid("base model differs"));
        }
        // Match the saved dictionary's exact explicit tokenizer bytes, not a newly invented registry.
        let tokenizer = fs::read(
            potential_root
                .join("run-1")
                .join(&name)
                .join("native-actions/tokenizer-identity.bin"),
        )?;
        let span_source = SpanSourceBinding::from_files(
            &base.join("model.safetensors"),
            &base.join("config.json"),
            &tokenizer,
        )?;
        let span = CompiledSpanActions::load(
            &potential_root
                .join("run-1")
                .join(&name)
                .join("native-actions"),
            &span_source,
            model
                .geometric_span()
                .ok_or_else(|| invalid("span absent"))?,
        )?;
        let potential_source = PotentialSourceBinding::from_directory(&base, &tokenizer)?;
        let potential = CompiledGeometricPotentials::load(
            &potential_root
                .join("run-1")
                .join(&name)
                .join("native-potentials"),
            &potential_source,
        )?;
        let events = EventWeights::new(40, 2, 820100 + seed)?;
        let mut optimizer = AdamW::new(
            events.parameters().values().cloned().collect(),
            ParamsAdamW {
                lr: 0.03,
                beta1: 0.9,
                beta2: 0.95,
                eps: 1e-8,
                weight_decay: 0.,
            },
        )?;
        let mut rng = Rng(820019 + seed);
        let mut history = Vec::new();
        let mut completed = 0;
        let mut actual = 0;
        let mut padded = 0;
        for step in 0..steps {
            if Instant::now() >= deadline {
                break;
            }
            let n = [2, 4][step % 2];
            let es: Vec<_> = (0..8)
                .map(|_| {
                    let facts = draw(&mut rng, n);
                    let q = rng.next(n);
                    let wn: Vec<_> = (0..n).map(|_| noise(&mut rng, 4)).collect();
                    let qn = noise(&mut rng, 4);
                    episode(&facts, &wn, &facts[q].0, &qn, 0, "train")
                })
                .collect();
            let (ids, targets, weights, time) = batch(&es);
            if time > 128 {
                return Err(invalid("training context ceiling"));
            }
            actual += es.iter().map(|e| e.ids.len()).sum::<usize>();
            padded += ids.len();
            let event_loss = event_credit(&events, &ids, &es, time)?;
            let logits = model.forward_geometric_event(&ids, 8, time, &events, false)?;
            let language = logits_cross_entropy(&logits, &targets, Some(&weights))?;
            let loss = (&event_loss + language.affine(0.1, 0.)?)?;
            let mut grads = loss.backward()?;
            let norm = gradient_norm(&events, &grads)?;
            let record = step % 40 == 0 || step + 1 == steps;
            let mut language_norms = std::collections::BTreeMap::new();
            if record {
                let g = language.backward()?;
                for (name, v) in events.parameters() {
                    language_norms.insert(
                        name.clone(),
                        g.get(v.as_tensor())
                            .map(|x| -> Result<f32> { Ok(x.abs()?.max_all()?.to_scalar::<f32>()?) })
                            .transpose()?,
                    );
                }
            }
            clip(&events, &mut grads, norm)?;
            optimizer.step(&grads)?;
            completed = step + 1;
            if record {
                history.push(json!({"step":completed,"time":time,"event_nll_balanced":event_loss.to_scalar::<f32>()?,"answer_nll":language.to_scalar::<f32>()?,"gradient_norm":norm,"pure_answer_max_gradients":language_norms,"elapsed_seconds":start.elapsed().as_secs_f64()}));
                fs::write(
                    root.join("history.json"),
                    serde_json::to_vec_pretty(&history)?,
                )?;
            }
        }
        events.save_source(&root.join("event-source"), &base, &tokenizer)?;
        let reload = EventWeights::load_source(&root.join("event-source"), &base, &tokenizer)?;
        let compiled =
            CompiledEvents::compile(&reload, &root.join("event-source"), &base, &tokenizer)?;
        compiled.save(&root.join("native-events"))?;
        let loaded = CompiledEvents::load(
            &root.join("native-events"),
            &root.join("event-source"),
            &base,
            &tokenizer,
        )?;
        let original = score(
            &model,
            &reload,
            &loaded,
            &span,
            &potential,
            &eval,
            &root.join("original"),
            deadline,
        )?;
        let longer = score(
            &model,
            &reload,
            &loaded,
            &span,
            &potential,
            &stress,
            &root.join("stress"),
            deadline,
        )?;
        let decision =
            if completed != steps || original["complete"] != true || longer["complete"] != true {
                "PARTIAL_BUDGET_NO_QUALITY_VERDICT"
            } else if [&original, &longer].iter().any(|p| {
                p["compilation_event_disagreements"] != 0
                    || p["compilation_transition_disagreements"] != 0
                    || p["compilation_context_state_disagreements"] != 0
            }) {
                "COMPILE_DIFFERENCE_RETAIN_TRACES"
            } else {
                "MEASURED_NATIVE_EVENT_LEARNING_RETAIN_ALL_OUTCOMES"
            };
        reports.push(json!({"name":name,"seed":seed,"steps":completed,"requested_steps":steps,"base_model_sha256":expected_model,"training_actual_positions":actual,"training_padded_positions":padded,"context_lanes":2,"event_config":reload.config(),"history":history,"decision":decision,"original":original,"stress":longer}));
        fs::write(
            out.join("progress.json"),
            serde_json::to_vec_pretty(&reports)?,
        )?;
    }
    Ok(
        json!({"schema":"uor-r4.geometric-event-native-learning/1","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNAVAILABLE"),"results":reports,"requested_steps":steps,"maximum_seconds":max_seconds,"elapsed_seconds":start.elapsed().as_secs_f64(),"training_scope":"new event factors only; frozen base/span/scorer; balanced event CE +0.1answerCE; labels outsideforward","context":"128fullcausal ceiling, width32,2heads,2newcontextlanes, trainingB8; actual lengths reported separately","scope":"authored contextualeventlearning in mixed offline stack; native event/span/scorer components only; float currentcodeproducer/trunk/age/NoRead/softmax/value/output remain; no language/geometrysuperiority/donorreasoning/energy qualification"}),
    )
}
fn main() -> Result<()> {
    let (mut parent, mut potentials, mut out) = (None, None, None);
    let (mut steps, mut seconds) = (320usize, 1800u64);
    for arg in std::env::args().skip(1) {
        let (k, v) = arg.split_once('=').ok_or_else(|| invalid("key=value"))?;
        match k {
            "parent" => parent = Some(std::path::PathBuf::from(v)),
            "potentials" => potentials = Some(std::path::PathBuf::from(v)),
            "out" => out = Some(std::path::PathBuf::from(v)),
            "steps" => steps = v.parse().map_err(|_| invalid("steps"))?,
            "max_seconds" => seconds = v.parse().map_err(|_| invalid("seconds"))?,
            _ => return Err(invalid("unknown option")),
        }
    }
    if steps == 0 || steps > 320 || seconds == 0 || seconds > 1800 {
        return Err(invalid("steps1..320 seconds1..1800"));
    }
    let parent = parent.ok_or_else(|| invalid("parent required"))?;
    let potentials = potentials.ok_or_else(|| invalid("potentials required"))?;
    let out = out.ok_or_else(|| invalid("out required"))?;
    report_output::claim(&out)?;
    let result = run(&parent, &potentials, &out, steps, seconds);
    match &result {
        Ok(r) => fs::write(out.join("report.json"), serde_json::to_vec_pretty(r)?)?,
        Err(e) => fs::write(
            out.join("error.json"),
            serde_json::to_vec_pretty(&json!({"error":e.to_string()}))?,
        )?,
    };
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    result?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_geometric_event_evaluator_keeps_labels_outside_forward_and_reports_ambiguous_tokens(
    ) -> Result<()> {
        let panel = evaluation(2110173, 4);
        assert_eq!(panel.len(), 128);
        let mut roles =
            std::collections::BTreeMap::<u32, std::collections::BTreeSet<String>>::new();
        for e in &panel {
            assert_eq!(e.roles.len(), e.ids.len());
            assert!(e.ids.len() <= 128);
            for (&token, role) in e.ids.iter().zip(&e.roles) {
                roles.entry(token).or_default().insert(role.clone());
            }
        }
        assert!(roles
            .iter()
            .any(|(&token, r)| token < 16 && r.contains("write_key") && r.contains("noise_key")));
        let (ids, _, _, time) = batch(&panel[..8]);
        let saved = ids.clone();
        let weights = EventWeights::new(40, 2, 19)?;
        let _ = weights.event_logits(&ids, 8, time)?;
        assert_eq!(ids, saved);
        valid_masses(&[0., 1.], 2)?;
        assert!(valid_masses(&[f32::NAN], 1).is_err());
        assert!(valid_masses(&[0.], 2).is_err());
        assert_eq!(argmax(&[1., 1.])?, 0);
        assert!(validate_code_trace(&[vec![120]], &[vec![1]], 1, 1).is_err());
        assert!(validate_code_trace(&[], &[], 1, 1).is_err());
        assert!(validate_output(&[vec![f32::NAN]], 1, 1).is_err());
        assert!(validate_output(&[], 1, 1).is_err());
        Ok(())
    }
}
