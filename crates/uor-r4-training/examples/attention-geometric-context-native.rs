//! Native latent geometric context compilation in the existing reader.
//! Authored grammar and frozen base; no natural-language or full serving claim.
use candle_core::{Device, Tensor};
use candle_nn::{AdamW, Optimizer, ParamsAdamW};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fs, path::Path, time::Instant};
use uor_r4_core::report_output;
use uor_r4_training::geometric_address::AddressCode;
use uor_r4_training::geometric_context::{
    self, CompiledContext, ContextSourcePaths, ContextWeights,
};
use uor_r4_training::geometric_event::CompiledEvents;
use uor_r4_training::geometric_potential_native::classify_inputs;
use uor_r4_training::geometric_potential_native::{
    CompiledGeometricPotentials, PotentialSourceBinding,
};
use uor_r4_training::geometric_span_native::{CompiledSpanActions, SpanSourceBinding};
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

fn gradient_norm(events: &ContextWeights, grads: &candle_core::backprop::GradStore) -> Result<f64> {
    let mut sum = 0.;
    for v in events.parameters().values() {
        if let Some(g) = grads.get(v.as_tensor()) {
            sum += f64::from(g.sqr()?.sum_all()?.to_scalar::<f32>()?);
        }
    }
    let norm = sum.sqrt();
    if !norm.is_finite() {
        return Err(invalid("nonfinite context gradient"));
    }
    Ok(norm)
}
fn clip(
    events: &ContextWeights,
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
fn teacher(model: &StackModel, ids: &[u32], b: usize, t: usize) -> Result<Vec<AddressCode>> {
    let u = model.geometric_context_teacher(ids, b, t)?;
    let config = model
        .geometric_address()
        .ok_or_else(|| invalid("address absent"))?;
    Ok(classify_inputs(&u, &u, config)?.current)
}
fn credit(
    model: &StackModel,
    context: &ContextWeights,
    ids: &[u32],
    es: &[Episode],
    time: usize,
) -> Result<(Tensor, Tensor)> {
    let target = teacher(model, ids, es.len(), time)?;
    let output = context.forward(ids, es.len(), time, false)?;
    let roots: Vec<u32> = target.iter().map(|c| u32::from(c.root)).collect();
    let categories: Vec<u32> = target
        .iter()
        .map(|c| {
            if c.present {
                1 + u32::from(c.radius_bin)
            } else {
                0
            }
        })
        .collect();
    let mut weights = vec![0.; target.len()];
    for (b, e) in es.iter().enumerate() {
        for t in 0..e.ids.len() {
            weights[(b * time + t) * 8..(b * time + t + 1) * 8].fill(1.);
        }
    }
    Ok((
        logits_cross_entropy(
            &output.root_logits.reshape((target.len(), 120))?,
            &roots,
            Some(&weights),
        )?,
        logits_cross_entropy(
            &output.category_logits.reshape((target.len(), 33))?,
            &categories,
            Some(&weights),
        )?,
    ))
}
fn validate_trace(
    states: &[Vec<u8>],
    actions: &[Vec<u8>],
    roots: &[u8],
    categories: &[u8],
    positions: usize,
) -> Result<()> {
    if states.len() != positions
        || actions.len() != positions
        || roots.len() != positions * 8
        || categories.len() != positions * 8
        || states
            .iter()
            .chain(actions)
            .any(|r| r.len() != 8 || r.iter().any(|&x| x >= 120))
        || roots.iter().any(|&x| x >= 120)
        || categories.iter().any(|&x| x >= 33)
    {
        return Err(invalid("context trace dimensions/codes differ"));
    }
    Ok(())
}
fn validate_output(rows: &[Vec<f32>], positions: usize) -> Result<()> {
    if rows.len() != positions
        || rows
            .iter()
            .any(|r| r.len() != 40 || r.iter().any(|v| !v.is_finite()))
    {
        return Err(invalid("context full output dimensions/nonfinite"));
    }
    Ok(())
}
fn native_json(trace: &geometric_context::NativeContextTrace) -> Value {
    json!({"batch":trace.batch,"time":trace.time,"heads":trace.heads,"lanes_per_head":trace.lanes_per_head,
        "states":trace.states,"actions":trace.actions,"emitted_roots":trace.emitted_roots,"categories":trace.categories,
        "codes":trace.codes.iter().map(|c|json!({"root":c.root(),"radius_bin":c.radius_bin(),"present":c.present()})).collect::<Vec<_>>(),"coefficient_reads":trace.coefficient_reads})
}
fn score(
    model: &StackModel,
    context: &ContextWeights,
    compiled: &CompiledContext,
    events: &CompiledEvents,
    span: &CompiledSpanActions,
    potential: &CompiledGeometricPotentials,
    es: &[Episode],
    out: &Path,
    deadline: Instant,
) -> Result<Value> {
    fs::create_dir(out)?;
    let mut rows = Vec::new();
    let (
        mut actual,
        mut padded,
        mut teacher_roots,
        mut teacher_categories,
        mut teacher_full,
        mut teacher_absent,
    ) = (0usize, 0usize, 0usize, 0usize, 0usize, 0usize);
    let (mut action_diff, mut state_diff, mut root_diff, mut category_diff, mut code_diff) =
        (0usize, 0usize, 0usize, 0usize, 0usize);
    for (chunk, group) in es.chunks(32).enumerate() {
        if Instant::now() >= deadline {
            break;
        }
        let (ids, _, _, time) = batch(group);
        let target = teacher(model, &ids, group.len(), time)?;
        let float = context.float_trace(&ids, group.len(), time, false)?;
        let hard = float.address_codes()?;
        let native = geometric_context::trace_native(&ids, group.len(), time, compiled, false)?;
        let reset = geometric_context::trace_native(&ids, group.len(), time, compiled, true)?;
        if hard.len() != ids.len() * 8
            || native.codes.len() != hard.len()
            || reset.codes.len() != hard.len()
            || target.len() != hard.len()
        {
            return Err(invalid("context trace shape"));
        }
        validate_trace(
            &float.states,
            &float.actions,
            &float.emitted_roots,
            &float.categories,
            ids.len(),
        )?;
        validate_trace(
            &native.states,
            &native.actions,
            &native.emitted_roots,
            &native.categories,
            ids.len(),
        )?;
        validate_trace(
            &reset.states,
            &reset.actions,
            &reset.emitted_roots,
            &reset.categories,
            ids.len(),
        )?;
        action_diff += float
            .actions
            .iter()
            .zip(&native.actions)
            .filter(|(a, b)| a != b)
            .count();
        state_diff += float
            .states
            .iter()
            .zip(&native.states)
            .filter(|(a, b)| a != b)
            .count();
        root_diff += float
            .emitted_roots
            .iter()
            .zip(&native.emitted_roots)
            .filter(|(a, b)| a != b)
            .count();
        category_diff += float
            .categories
            .iter()
            .zip(&native.categories)
            .filter(|(a, b)| a != b)
            .count();
        code_diff += hard
            .iter()
            .zip(&native.codes)
            .filter(|(a, b)| a != b)
            .count();
        padded += ids.len();
        actual += group.iter().map(|e| e.ids.len()).sum::<usize>();
        let baseline = model
            .forward_geometric_event_native(
                &ids,
                group.len(),
                time,
                events,
                span,
                potential,
                false,
            )?
            .to_vec2::<f32>()?;
        let reference = model
            .forward_geometric_context(&ids, group.len(), time, context, events, span, false)?
            .to_vec2::<f32>()?;
        let replay = model
            .forward_geometric_context_replay(
                &ids,
                group.len(),
                time,
                context,
                events,
                span,
                potential,
                false,
            )?
            .to_vec2::<f32>()?;
        let logits = model
            .forward_geometric_context_native(
                &ids,
                group.len(),
                time,
                compiled,
                events,
                span,
                potential,
                false,
            )?
            .to_vec2::<f32>()?;
        let reset_logits = model
            .forward_geometric_context_native(
                &ids,
                group.len(),
                time,
                compiled,
                events,
                span,
                potential,
                true,
            )?
            .to_vec2::<f32>()?;
        for output in [&baseline, &reference, &replay, &logits, &reset_logits] {
            validate_output(output, ids.len())?;
        }
        let (mut bm, mut fm, mut nm, mut rm) = (
            [Vec::new(), Vec::new()],
            [Vec::new(), Vec::new()],
            [Vec::new(), Vec::new()],
            [Vec::new(), Vec::new()],
        );
        for h in 0..2 {
            let target = binding(group, h);
            bm[h] = model
                .read_binding_masses_geometric_event_native(
                    &ids,
                    group.len(),
                    time,
                    &target,
                    events,
                    span,
                    potential,
                    false,
                )?
                .to_vec1::<f32>()?;
            fm[h] = model
                .read_binding_masses_geometric_context_replay(
                    &ids,
                    group.len(),
                    time,
                    &target,
                    context,
                    events,
                    span,
                    potential,
                    false,
                )?
                .to_vec1::<f32>()?;
            nm[h] = model
                .read_binding_masses_geometric_context_native(
                    &ids,
                    group.len(),
                    time,
                    &target,
                    compiled,
                    events,
                    span,
                    potential,
                    false,
                )?
                .to_vec1::<f32>()?;
            rm[h] = model
                .read_binding_masses_geometric_context_native(
                    &ids,
                    group.len(),
                    time,
                    &target,
                    compiled,
                    events,
                    span,
                    potential,
                    true,
                )?
                .to_vec1::<f32>()?;
            for v in [&bm[h], &fm[h], &nm[h], &rm[h]] {
                valid_masses(v, group.len())?;
            }
        }
        let mut chunk_rows = Vec::new();
        for (b, e) in group.iter().enumerate() {
            for t in 0..e.ids.len() {
                for l in 0..8 {
                    let at = (b * time + t) * 8 + l;
                    let a = target[at];
                    let n = native.codes[at];
                    teacher_roots += usize::from(a.present == n.present() && a.root == n.root());
                    teacher_categories +=
                        usize::from(a.present == n.present() && a.radius_bin == n.radius_bin());
                    teacher_full += usize::from(
                        a.present == n.present()
                            && a.root == n.root()
                            && a.radius_bin == n.radius_bin(),
                    );
                    teacher_absent += usize::from(!a.present);
                }
            }
            let at = b * time + e.query;
            let row = json!({"pair":e.pair,"condition":e.condition,"query":e.query,"source":e.source,"answer":e.answer,
                "baseline_prediction":argmax(&baseline[at])?,"reference_prediction":argmax(&reference[at])?,"replay_prediction":argmax(&replay[at])?,"native_prediction":argmax(&logits[at])?,"reset_prediction":argmax(&reset_logits[at])?,
                "baseline_answer_logits":baseline[at],"reference_answer_logits":reference[at],"replay_answer_logits":replay[at],"native_answer_logits":logits[at],"reset_answer_logits":reset_logits[at],
                "baseline_source_masses":[bm[0][b],bm[1][b]],"replay_source_masses":[fm[0][b],fm[1][b]],"native_source_masses":[nm[0][b],nm[1][b]],"reset_source_masses":[rm[0][b],rm[1][b]]});
            rows.push(row.clone());
            chunk_rows.push(row);
        }
        fs::write(
            out.join(format!("chunk-{chunk}.json")),
            serde_json::to_vec_pretty(
                &json!({"batch":group.len(),"time":time,"ids":ids,"teacher_codes":target,"float_trace":float,"native_trace":native_json(&native),"reset_trace":native_json(&reset),"rows":chunk_rows}),
            )?,
        )?;
    }
    let summary = json!({"complete":rows.len()==es.len(),"rows":rows.len(),"actual_positions":actual,"padded_positions":padded,
        "actual_lanes":actual*8,"teacher_root_agreement":teacher_roots,"teacher_category_agreement":teacher_categories,"teacher_full_agreement":teacher_full,"teacher_absent_lanes":teacher_absent,
        "compilation_action_position_disagreements":action_diff,"compilation_state_position_disagreements":state_diff,"compilation_root_disagreements":root_diff,"compilation_category_disagreements":category_diff,"compilation_code_disagreements":code_diff,
        "baseline_answers":rows.iter().filter(|r|r["baseline_prediction"]==r["answer"]).count(),"reference_answers":rows.iter().filter(|r|r["reference_prediction"]==r["answer"]).count(),"replay_answers":rows.iter().filter(|r|r["replay_prediction"]==r["answer"]).count(),"native_answers":rows.iter().filter(|r|r["native_prediction"]==r["answer"]).count(),"reset_answers":rows.iter().filter(|r|r["reset_prediction"]==r["answer"]).count(),
        "compilation_prediction_changes":rows.iter().filter(|r|r["native_prediction"]!=r["replay_prediction"]).count(),
        "native_head0_majorities":rows.iter().filter(|r|r["native_source_masses"][0].as_f64().is_some_and(|v|v>0.5)).count(),"native_head1_majorities":rows.iter().filter(|r|r["native_source_masses"][1].as_f64().is_some_and(|v|v>0.5)).count(),"rows_detail":rows});
    fs::write(
        out.join("summary.json"),
        serde_json::to_vec_pretty(&summary)?,
    )?;
    Ok(summary)
}
fn run(
    parent: &Path,
    potential_root: &Path,
    event_root: &Path,
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
    for (path, hash) in [
        (
            "evaluation.json",
            "dabdaa1a2a8dcf23e1cfe5164ef00b97b923c3b74dc09dcd1d4a307d1ff64915",
        ),
        (
            "stress.json",
            "e3c81984b7b41e0fcb329fde026318d067b6397e93832508c3ee60180a5a9d63",
        ),
    ] {
        if sha256_file(&parent.join("run-1").join(path))? != hash {
            return Err(invalid("pinned panel hash"));
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
        let expected = if seed == 1 {
            "235614f2e1861dd79e2e4ea8dac3cdaaec13243f145c04d9a77fbda3e9bb47fa"
        } else {
            "3909b1129e4cf2a2aa72c3cc7ad3172b8eb1c847b881ce054418e31f44c21873"
        };
        if sha256_file(&base.join("model.safetensors"))? != expected {
            return Err(invalid("base model hash"));
        }
        let span_dir = potential_root
            .join("run-1")
            .join(&name)
            .join("native-actions");
        let potential_dir = potential_root
            .join("run-1")
            .join(&name)
            .join("native-potentials");
        let event_source = event_root.join("run-1").join(&name).join("event-source");
        let event_native = event_root.join("run-1").join(&name).join("native-events");
        let tokenizer = fs::read(span_dir.join("tokenizer-identity.bin"))?;
        let span_source = SpanSourceBinding::from_files(
            &base.join("model.safetensors"),
            &base.join("config.json"),
            &tokenizer,
        )?;
        let span = CompiledSpanActions::load(
            &span_dir,
            &span_source,
            model
                .geometric_span()
                .ok_or_else(|| invalid("span absent"))?,
        )?;
        let potential = CompiledGeometricPotentials::load(
            &potential_dir,
            &PotentialSourceBinding::from_directory(&base, &tokenizer)?,
        )?;
        let events = CompiledEvents::load(&event_native, &event_source, &base, &tokenizer)?;
        let paths = ContextSourcePaths {
            base: &base,
            event_source: &event_source,
            event_native: &event_native,
            span_native: &span_dir,
            potential_native: &potential_dir,
        };
        let context = ContextWeights::new(40, 32, 2, 830100 + seed)?;
        let mut optimizer = AdamW::new(
            context.parameters().values().cloned().collect(),
            ParamsAdamW {
                lr: 0.03,
                beta1: 0.9,
                beta2: 0.95,
                eps: 1e-8,
                weight_decay: 0.,
            },
        )?;
        let mut rng = Rng(830019 + seed);
        let mut history = Vec::new();
        let (mut completed, mut actual, mut padded) = (0, 0, 0);
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
            let (root_loss, category_loss) = credit(&model, &context, &ids, &es, time)?;
            let logits =
                model.forward_geometric_context(&ids, 8, time, &context, &events, &span, false)?;
            let language = logits_cross_entropy(&logits, &targets, Some(&weights))?;
            let loss = ((&root_loss + &category_loss)? + language.affine(0.1, 0.)?)?;
            let mut grads = loss.backward()?;
            let norm = gradient_norm(&context, &grads)?;
            let record = step % 40 == 0 || step + 1 == steps;
            let mut language_norms = std::collections::BTreeMap::new();
            if record {
                let g = language.backward()?;
                for (name, v) in context.parameters() {
                    language_norms.insert(
                        name.clone(),
                        g.get(v.as_tensor())
                            .map(|x| -> Result<f32> { Ok(x.abs()?.max_all()?.to_scalar::<f32>()?) })
                            .transpose()?,
                    );
                }
            }
            clip(&context, &mut grads, norm)?;
            optimizer.step(&grads)?;
            completed = step + 1;
            if record {
                history.push(json!({"step":completed,"time":time,"root_nll":root_loss.to_scalar::<f32>()?,"category_nll":category_loss.to_scalar::<f32>()?,"answer_nll":language.to_scalar::<f32>()?,"gradient_norm":norm,"pure_answer_max_gradients":language_norms,"elapsed_seconds":start.elapsed().as_secs_f64()}));
                fs::write(
                    root.join("history.json"),
                    serde_json::to_vec_pretty(&history)?,
                )?;
            }
        }
        context.save_source(&root.join("context-source"), paths, &tokenizer)?;
        let reload = ContextWeights::load_source(&root.join("context-source"), paths, &tokenizer)?;
        let compiled =
            CompiledContext::compile(&reload, &root.join("context-source"), paths, &tokenizer)?;
        compiled.save(&root.join("native-context"))?;
        let loaded = CompiledContext::load(
            &root.join("native-context"),
            &root.join("context-source"),
            paths,
            &tokenizer,
        )?;
        let original = score(
            &model,
            &reload,
            &loaded,
            &events,
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
            &events,
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
                p["compilation_code_disagreements"] != 0
                    || p["compilation_state_position_disagreements"] != 0
                    || p["compilation_action_position_disagreements"] != 0
            }) {
                "COMPILE_DIFFERENCE_RETAIN_TRACES"
            } else {
                "MEASURED_CONTEXT_PRODUCER_RETAIN_ALL_OUTCOMES"
            };
        reports.push(json!({"name":name,"seed":seed,"steps":completed,"requested_steps":steps,"base_model_sha256":expected,"context_config":reload.config(),"training_actual_positions":actual,"training_padded_positions":padded,"history":history,"decision":decision,"original":original,"stress":longer}));
        fs::write(
            out.join("progress.json"),
            serde_json::to_vec_pretty(&reports)?,
        )?;
    }
    Ok(
        json!({"schema":"uor-r4.geometric-context-native-learning/1","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNAVAILABLE"),"results":reports,"requested_steps":steps,"maximum_seconds":max_seconds,"elapsed_seconds":start.elapsed().as_secs_f64(),"scope":"authored development donor-context compilation; latent8roots independent emitted8addresses; frozen nativeevents/span/potential/base; rootCE+categoryCE+.1answerCE; ordinaryforward accepts IDs/learnedweights only; fullcausal128ceiling width32heads2B8; no exactteacher-codegate, natural-language/reasoningtransfer/geometryadvantage/energy/fullinteger-serving claim; originalu supplies floatvalues/NoRead/trunk/output"}),
    )
}
fn main() -> Result<()> {
    let (mut parent, mut potentials, mut events, mut out) = (None, None, None, None);
    let (mut steps, mut seconds) = (640usize, 7200u64);
    for arg in std::env::args().skip(1) {
        let (k, v) = arg.split_once('=').ok_or_else(|| invalid("key=value"))?;
        match k {
            "parent" => parent = Some(std::path::PathBuf::from(v)),
            "potentials" => potentials = Some(std::path::PathBuf::from(v)),
            "events" => events = Some(std::path::PathBuf::from(v)),
            "out" => out = Some(std::path::PathBuf::from(v)),
            "steps" => steps = v.parse().map_err(|_| invalid("steps"))?,
            "max_seconds" => seconds = v.parse().map_err(|_| invalid("seconds"))?,
            _ => return Err(invalid("unknown option")),
        }
    }
    if steps == 0 || steps > 640 || seconds == 0 || seconds > 7200 {
        return Err(invalid("steps1..640,seconds1..7200"));
    }
    let parent = parent.ok_or_else(|| invalid("parent required"))?;
    let potentials = potentials.ok_or_else(|| invalid("potentials required"))?;
    let events = events.ok_or_else(|| invalid("events required"))?;
    let out = out.ok_or_else(|| invalid("out required"))?;
    report_output::claim(&out)?;
    let result = run(&parent, &potentials, &events, &out, steps, seconds);
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
    fn native_context_evaluator_masks_padding_and_has_no_labels_in_forward() -> Result<()> {
        let panel = evaluation(2110173, 4);
        let (ids, _, _, time) = batch(&panel[..8]);
        let saved = ids.clone();
        let context = ContextWeights::new(40, 32, 2, 19)?;
        let _ = context.forward(&ids, 8, time, false)?;
        assert_eq!(ids, saved);
        assert!(panel
            .iter()
            .all(|e| e.ids.len() == e.roles.len() && e.ids.len() <= 128));
        valid_masses(&[0., 1.], 2)?;
        assert!(valid_masses(&[f32::NAN], 1).is_err());
        assert_eq!(argmax(&[1., 1.])?, 0);
        Ok(())
    }
}
