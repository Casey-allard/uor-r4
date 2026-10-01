//! Paired occurrence-binding experiment on the existing native stack.
//! Synthetic token episodes test query/key association, not natural language.
//! Run: attention-binding out=NEW_ROOT [steps=320] [max_seconds=900] [carry=0|1]
use candle_core::{DType, Device, Tensor};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fs, path::Path, time::Instant};
use uor_r4_core::report_output;
use uor_r4_training::geometric_stack::{
    ReadBinding, ReadBindingTarget, ReadScore, StackAdamW, StackArch, StackConfig, StackModel,
    StackSite,
};
use uor_r4_training::{Result, TrainingError};

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
    source: usize,
    answer: u32,
    facts: usize,
    pair: usize,
    condition: String,
}
fn episode(facts: &[(u32, u32)], key: u32, pair: usize, condition: &str) -> Episode {
    let mut ids = vec![36];
    let mut source = 0;
    let mut answer = 0;
    for &(k, v) in facts {
        ids.extend([32, k, v]);
        if k == key {
            source = ids.len() - 1;
            answer = v;
        }
    }
    ids.extend([33, key, 34]);
    Episode {
        ids,
        source,
        answer,
        facts: facts.len(),
        pair,
        condition: condition.into(),
    }
}
fn draw(rng: &mut Rng, n: usize) -> Vec<(u32, u32)> {
    let mut keys: Vec<u32> = (0..16).collect();
    for i in (1..keys.len()).rev() {
        let j = rng.next(i + 1);
        keys.swap(i, j);
    }
    keys[..n]
        .iter()
        .map(|&k| (k, 16 + rng.next(16) as u32))
        .collect()
}
fn evaluation() -> Vec<Episode> {
    evaluation_draw(910173, &[2, 4, 8])
}
fn evaluation_draw(seed: u64, sizes: &[usize]) -> Vec<Episode> {
    let mut rng = Rng(seed);
    let mut out = Vec::new();
    for &n in sizes {
        for p in 0..16 {
            let pair = n * 100 + p;
            let facts = draw(&mut rng, n);
            let q = rng.next(n);
            out.push(episode(&facts, facts[q].0, pair, "base"));
            out.push(episode(&facts, facts[(q + 1) % n].0, pair, "changed_query"));
            let mut swapped = facts.clone();
            let v = swapped[q].1;
            swapped[q].1 = swapped[(q + 1) % n].1;
            swapped[(q + 1) % n].1 = v;
            out.push(episode(&swapped, facts[q].0, pair, "swapped_values"));
            let mut reversed = facts.clone();
            reversed.reverse();
            out.push(episode(&reversed, facts[q].0, pair, "reversed_order"));
        }
    }
    out
}
fn binding(episodes: &[Episode]) -> ReadBindingTarget {
    ReadBindingTarget {
        layer: 2,
        head: 0,
        rows: episodes
            .iter()
            .enumerate()
            .map(|(batch, e)| ReadBinding {
                batch,
                query: e.ids.len() - 1,
                sources: vec![e.source],
            })
            .collect(),
    }
}
fn batch(episodes: &[Episode]) -> (Vec<u32>, Vec<u32>, Vec<f32>) {
    let time = episodes[0].ids.len();
    let ids = episodes
        .iter()
        .flat_map(|e| e.ids.iter().copied())
        .collect();
    let mut targets = vec![0; time * episodes.len()];
    let mut weights = vec![0.; targets.len()];
    for (b, e) in episodes.iter().enumerate() {
        targets[b * time + time - 1] = e.answer;
        weights[b * time + time - 1] = 1.;
    }
    (ids, targets, weights)
}
fn score(model: &StackModel, episodes: &[Episode]) -> Result<Vec<Value>> {
    let mut out = Vec::new();
    for group in episodes.chunks(64) {
        // Evaluation groups have equal lengths within each fact-count cohort.
        let time = group[0].ids.len();
        let (ids, targets, weights) = batch(group);
        let logits = model.forward(&ids, group.len(), time)?;
        let predictions = logits.argmax(candle_core::D::Minus1)?.to_vec1::<u32>()?;
        let masses = model
            .read_binding_masses(&ids, group.len(), time, &binding(group))?
            .to_vec1::<f32>()?;
        let nll = model
            .weighted_loss(&ids, &targets, &weights, group.len(), time)?
            .to_scalar::<f32>()?;
        for (b, e) in group.iter().enumerate() {
            let latest = e.ids[3 * e.facts];
            out.push(json!({"pair":e.pair,"condition":e.condition,"facts":e.facts,
                "source":e.source,"answer":e.answer,"prediction":predictions[b*time+time-1],
                "expected_answer_differs_from_base":e.answer!=group[b-b%4].answer,
                "expected_source_differs_from_base":e.source!=group[b-b%4].source,
                "answer_correct":predictions[b*time+time-1]==e.answer,
                "correct_occurrence_mass":masses[b],
                "source_majority":masses[b]>0.5,
                "source_majority_scope":"sufficient for unique top source against the full prefix and NoRead; not a complete argmax count",
                "latest_answer_correct":latest==e.answer,"latest_source_correct":e.source==3*e.facts,
                "uniform_source_mass":1.0/(time as f64+1.0),"cohort_answer_nll":nll}));
        }
    }
    Ok(out)
}
fn run(
    out: &Path,
    steps: usize,
    max_seconds: u64,
    carry: bool,
    auxiliary_filter: Option<bool>,
) -> Result<Value> {
    let start = Instant::now();
    let eval = evaluation();
    fs::write(
        out.join("evaluation.json"),
        serde_json::to_vec_pretty(&eval)?,
    )?;
    let mut reports = Vec::new();
    for seed in [1, 2] {
        for read in [ReadScore::Dot, ReadScore::Lorentz] {
            for auxiliary in [false, true] {
                if carry && auxiliary != auxiliary_filter.unwrap_or(true) {
                    continue;
                }
                if start.elapsed().as_secs() >= max_seconds {
                    break;
                }
                let name = format!(
                    "{read:?}-{}-s{seed}",
                    if auxiliary { "binding" } else { "language" }
                );
                let root = out.join(&name);
                fs::create_dir(&root)?;
                let config = StackConfig {
                    arch: StackArch::Geometric,
                    vocab_size: 40,
                    width: 32,
                    heads: 2,
                    mlp_hidden: 64,
                    context: 64,
                    pattern: "rra".into(),
                    read,
                    rotation: true,
                    seed,
                    memory: None,
                    select: None,
                    pointer: None,
                };
                let mut model = StackModel::new(config, &Device::Cpu)?;
                model.set_read_identity_carry(carry)?;
                let mut optimizer = StackAdamW::new(&model, 0.0, 1.0)?;
                let mut rng = Rng(700019 + seed);
                let mut history = Vec::new();
                let mut done = 0;
                for step in 0..steps {
                    if start.elapsed().as_secs() >= max_seconds {
                        break;
                    }
                    let n = [2, 4, 8][step % 3];
                    let episodes: Vec<_> = (0..16)
                        .map(|_| {
                            let f = draw(&mut rng, n);
                            let q = rng.next(n);
                            episode(&f, f[q].0, 0, "train")
                        })
                        .collect();
                    let time = episodes[0].ids.len();
                    let (ids, targets, weights) = batch(&episodes);
                    let (language, aux) = if auxiliary {
                        let loss = model.loss_with_binding(
                            &ids,
                            &targets,
                            Some(&weights),
                            16,
                            time,
                            &binding(&episodes),
                        )?;
                        (loss.language, Some(loss.binding))
                    } else {
                        (
                            model.weighted_loss(&ids, &targets, &weights, 16, time)?,
                            None,
                        )
                    };
                    let language_value = language.to_scalar::<f32>()?;
                    let binding_value = aux.as_ref().map(|a| a.to_scalar::<f32>()).transpose()?;
                    let total = match aux {
                        Some(aux) => language.add(&aux)?,
                        None => language,
                    };
                    let norm = optimizer.update(&model, &total.backward()?, 0.003)?;
                    done = step + 1;
                    if step % 40 == 0 || done == steps {
                        history.push(json!({"step":done,"answer_nll":language_value,
                        "binding_nll":binding_value,"gradient_norm":norm,"elapsed_seconds":start.elapsed().as_secs_f64()}));
                    }
                }
                model.save(&root.join("model"))?;
                let rows = if start.elapsed().as_secs() < max_seconds {
                    Some(score(&model, &eval)?)
                } else {
                    None
                };
                let read_disabled = if rows.is_some() && start.elapsed().as_secs() < max_seconds {
                    let weight = model
                        .variables()
                        .get("layers.02.read.out.weight")
                        .ok_or_else(|| invalid("missing selected read output"))?;
                    let original = weight.as_tensor().copy()?;
                    weight.set(&Tensor::zeros(weight.shape(), DType::F32, model.device())?)?;
                    let disabled = score(&model, &eval);
                    weight.set(&original)?;
                    Some(disabled?)
                } else {
                    None
                };
                let reload = StackModel::load(&root.join("model"), &Device::Cpu)?;
                let ids = &eval[0].ids;
                let before = model
                    .forward(ids, 1, ids.len())?
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                let after = reload
                    .forward(ids, 1, ids.len())?
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                let report = json!({"name":name,"read_identity_carry":carry,"seed":seed,"read":read,"binding_weight":if auxiliary {1.0}else{0.0},
                    "steps":done,"requested_steps":steps,"config":model.config,"parameter_count":model.variables().values().map(|v|v.elem_count()).sum::<usize>(),
                    "history":history,"rows":rows,"read_output_disabled_rows":read_disabled,"reload_logits_identical":before==after,
                    "scope":"synthetic token association only; no natural language, quantized serving, geometric superiority or frontier qualification"});
                fs::write(
                    root.join("result.json"),
                    serde_json::to_vec_pretty(&report)?,
                )?;
                reports.push(report);
            }
        }
    }
    Ok(
        json!({"schema":"uor-r4.attention-binding-experiment/1","reports":reports,
        "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNAVAILABLE"),
        "read_identity_carry":carry,"auxiliary_filter":auxiliary_filter,"planned_arms":if carry {4}else{8},"steps":steps,"max_seconds":max_seconds,"elapsed_seconds":start.elapsed().as_secs_f64(),
        "data":"distinct shuffled keys, iid values with duplicate tokens; 2/4/8 facts; query, value and order interventions",
        "labels":"training-only exact value occurrence; no candidate pruning or runtime annotation",
        "sampling":"16 base episodes per fact count, four correlated interventions each; fresh deterministic draws, not a certified deduplicated final holdout",
        "mediation":"selected read output projection is temporarily zeroed for paired evaluation then restored; recurrent residual paths remain available",
        "limits":"cooperative deadline, checkpoints retained at each arm; planned_arms above; unfinished arms are NOT_RUN",
        "uniform_baseline":"includes all causal token positions and NoRead; empirical latest-answer/source baselines retained separately"}),
    )
}

fn difference(a: &[f32], b: &[f32]) -> f64 {
    let delta = a
        .iter()
        .zip(b)
        .map(|(&a, &b)| (f64::from(a) - f64::from(b)).powi(2))
        .sum::<f64>()
        .sqrt();
    let scale = ((a.iter().map(|&x| f64::from(x).powi(2)).sum::<f64>()
        + b.iter().map(|&x| f64::from(x).powi(2)).sum::<f64>())
        / 2.0)
        .sqrt();
    delta / scale.max(1e-12)
}
fn latent(
    model: &StackModel,
    ids: &[u32],
    batch: usize,
    time: usize,
) -> Result<(Vec<Vec<f32>>, Vec<Vec<f32>>, Vec<Vec<f32>>)> {
    let mut input = None;
    let _ = model.hidden_with_capture(ids, batch, time, &mut |site, tensor| {
        if site == StackSite::Read(2) {
            input = Some(tensor.clone());
        }
        Ok(())
    })?;
    let input = input.ok_or_else(|| invalid("missing read input"))?;
    let gain = model
        .variables()
        .get("layers.02.read_norm.weight")
        .ok_or_else(|| invalid("read gain"))?;
    let u = input.broadcast_mul(gain.as_tensor())?;
    let u = if model.read_identity_carry() {
        let shaped = u.reshape((batch, time, 32))?;
        let zero = Tensor::zeros((batch, 1, 32), DType::F32, model.device())?;
        Tensor::cat(&[&zero, &shaped.narrow(1, 0, time - 1)?], 1)?.reshape((batch * time, 32))?
    } else {
        u
    };
    let project = |name: &str| -> Result<Vec<Vec<f32>>> {
        let weight = model
            .variables()
            .get(name)
            .ok_or_else(|| invalid("read weight"))?;
        Ok(u.matmul(&weight.as_tensor().t()?)?
            .narrow(1, 0, 16)?
            .to_vec2::<f32>()?)
    };
    Ok((
        input.to_vec2::<f32>()?,
        project("layers.02.read.query.weight")?,
        project("layers.02.read.key.weight")?,
    ))
}
fn probe(models: &Path, out: &Path) -> Result<Value> {
    report_output::verify(models)?;
    let parent_report: Value = serde_json::from_slice(&fs::read(models.join("report.json"))?)?;
    let carry = parent_report["read_identity_carry"]
        .as_bool()
        .unwrap_or(false);
    let auxiliary_filter = parent_report["auxiliary_filter"].as_bool();
    let bytes = fs::read(models.join("evaluation.json"))?;
    let all: Vec<Episode> = serde_json::from_slice(&bytes)?;
    let episodes: Vec<_> = all
        .into_iter()
        .filter(|e| e.condition == "base" || e.condition == "changed_query")
        .collect();
    if episodes.len() != 96 {
        return Err(invalid("expected frozen 48 query pairs"));
    }
    fs::write(
        out.join("query-pairs.json"),
        serde_json::to_vec_pretty(&episodes)?,
    )?;
    let mut reports = Vec::new();
    for seed in [1, 2] {
        for read in [ReadScore::Dot, ReadScore::Lorentz] {
            for auxiliary in [false, true] {
                if carry && auxiliary != auxiliary_filter.unwrap_or(true) {
                    continue;
                }
                let name = format!(
                    "{read:?}-{}-s{seed}",
                    if auxiliary { "binding" } else { "language" }
                );
                let root = models.join(&name);
                let parent: Value = serde_json::from_slice(&fs::read(root.join("result.json"))?)?;
                let model = StackModel::load(&root.join("model"), &Device::Cpu)?;
                if model.config.width != 32
                    || model.config.heads != 2
                    || model.config.pattern != "rra"
                {
                    return Err(invalid("probe model shape"));
                }
                let mut rows = Vec::new();
                for group in episodes.chunks(32) {
                    let time = group[0].ids.len();
                    for pair in group.chunks(2) {
                        if pair.len() != 2
                            || pair[0].pair != pair[1].pair
                            || pair[0].source == pair[1].source
                        {
                            return Err(invalid("invalid query pair"));
                        }
                    }
                    let (ids, _, _) = batch(group);
                    let target = |which: usize| ReadBindingTarget {
                        layer: 2,
                        head: 0,
                        rows: group
                            .iter()
                            .enumerate()
                            .map(|(batch, e)| ReadBinding {
                                batch,
                                query: e.ids.len() - 1,
                                sources: vec![group[batch - batch % 2 + which].source],
                            })
                            .collect(),
                    };
                    let a = model
                        .read_binding_masses(&ids, group.len(), time, &target(0))?
                        .to_vec1::<f32>()?;
                    let b = model
                        .read_binding_masses(&ids, group.len(), time, &target(1))?
                        .to_vec1::<f32>()?;
                    let (input, query, key) = latent(&model, &ids, group.len(), time)?;
                    let mut swapped = group.to_vec();
                    for pair in swapped.chunks_mut(2) {
                        let i = pair[0].source - 1;
                        let j = pair[1].source - 1;
                        for e in pair {
                            e.ids.swap(i, j);
                        }
                    }
                    let (swap_ids, _, _) = batch(&swapped);
                    let (swap_input, _, swap_key) = latent(&model, &swap_ids, group.len(), time)?;
                    for p in (0..group.len()).step_by(2) {
                        let masses = [a[p], b[p], a[p + 1], b[p + 1]];
                        if masses.iter().any(|&m| m <= 0.0 || !m.is_finite()) {
                            return Err(invalid("probe mass invalid"));
                        }
                        let contrast = (f64::from(a[p]) / f64::from(b[p])).ln()
                            - (f64::from(a[p + 1]) / f64::from(b[p + 1])).ln();
                        let first = p * time;
                        let second = (p + 1) * time;
                        let source = group[p].source;
                        rows.push(json!({"pair":group[p].pair,"facts":group[p].facts,"masses_AA_AB_BA_BB":masses,
                    "query_specific_log_odds_contrast":contrast,
                    "query_key_state_change":difference(&input[first+time-2],&input[second+time-2]),
                    "query_answer_state_change":difference(&input[first+time-1],&input[second+time-1]),
                    "query_projection_change":difference(&query[first+time-1],&query[second+time-1]),
                    "swapped_source_key_state_change":difference(&input[first+source-1],&swap_input[first+source-1]),
                    "swapped_source_value_state_change":difference(&input[first+source],&swap_input[first+source]),
                    "swapped_source_value_key_projection_change":difference(&key[first+source],&swap_key[first+source]),
                    "swap_inputs":swapped[p].ids}));
                    }
                }
                reports.push(json!({"name":name,"steps":parent["steps"],"rows":rows,"model_sha256":uor_r4_training::sha256_file(&root.join("model/model.safetensors"))?}));
            }
        }
    }
    Ok(
        json!({"schema":"uor-r4.attention-query-probe/1","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNAVAILABLE"),
        "evaluation_sha256":uor_r4_training::sha256_file(&models.join("evaluation.json"))?,"models":models,"reports":reports,
        "contrast":"log(mAA/mAB)-log(mBA/mBB); normalizer, NoRead and fixed positional preferences cancel; positive is correct query-specific separation",
        "latent":"RMS-normalized current pre-gain read input; learned gain and optional causal predecessor shift then actual query/key projections, head0; relative RMS Euclidean differences",
        "scope":"saved synthetic models only, no fit, threshold promotion or geometry advantage"}),
    )
}
/// Fixed post-fit development challenge; no training or design selection.
fn stress(models: &Path, out: &Path) -> Result<Value> {
    report_output::verify(models)?;
    let parent: Value = serde_json::from_slice(&fs::read(models.join("report.json"))?)?;
    let carry = parent["read_identity_carry"].as_bool().unwrap_or(false);
    let auxiliary_filter = parent["auxiliary_filter"].as_bool();
    let eval = evaluation_draw(1920173, &[2, 8, 16]);
    fs::write(
        out.join("evaluation.json"),
        serde_json::to_vec_pretty(&eval)?,
    )?;
    let mut reports = Vec::new();
    for seed in [1, 2] {
        for read in [ReadScore::Dot, ReadScore::Lorentz] {
            for auxiliary in [false, true] {
                if carry && auxiliary != auxiliary_filter.unwrap_or(true) {
                    continue;
                }
                let name = format!(
                    "{read:?}-{}-s{seed}",
                    if auxiliary { "binding" } else { "language" }
                );
                let root = models.join(&name);
                let model = StackModel::load(&root.join("model"), &Device::Cpu)?;
                if model.read_identity_carry() != carry {
                    return Err(invalid("parent carry differs"));
                }
                let mut head1 = Vec::new();
                for group in eval.chunks(64) {
                    let time = group[0].ids.len();
                    let (ids, _, _) = batch(group);
                    let mut target = binding(group);
                    target.head = 1;
                    let masses = model
                        .read_binding_masses(&ids, group.len(), time, &target)?
                        .to_vec1::<f32>()?;
                    for (e, mass) in group.iter().zip(masses) {
                        head1.push(json!({"pair":e.pair,"condition":e.condition,"correct_occurrence_mass":mass,"source_majority":mass>0.5}));
                    }
                }
                reports.push(json!({"name":name,"read_identity_carry":carry,"rows":score(&model,&eval)?,"head1_occurrence_rows":head1,

                    "model_sha256":uor_r4_training::sha256_file(&root.join("model/model.safetensors"))?}));
            }
        }
    }
    Ok(
        json!({"schema":"uor-r4.attention-binding-stress/1","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNAVAILABLE"),
        "models":models,"seed":1920173,"facts":[2,8,16],"read_identity_carry":carry,"reports":reports,
        "scope":"post-fit fresh deterministic development draw; 16-fact length extrapolation beyond training 8; no certified final holdout or serving qualification"}),
    )
}

fn main() -> Result<()> {
    let mut out = None;
    let mut mode = "fit".to_string();
    let mut models = None;
    let mut steps = 320;
    let mut seconds = 900;
    let mut carry = false;
    let mut auxiliary_filter = None;
    for arg in std::env::args().skip(1) {
        let (k, v) = arg
            .split_once('=')
            .ok_or_else(|| invalid("expected key=value"))?;
        match k {
            "mode" => mode = v.into(),
            "auxiliary" => {
                auxiliary_filter = Some(match v {
                    "0" => false,
                    "1" => true,
                    _ => return Err(invalid("auxiliary 0|1")),
                })
            }
            "carry" => {
                carry = match v {
                    "0" => false,
                    "1" => true,
                    _ => return Err(invalid("carry 0|1")),
                }
            }
            "models" => models = Some(std::path::PathBuf::from(v)),
            "out" => out = Some(std::path::PathBuf::from(v)),
            "steps" => steps = v.parse().map_err(|_| invalid("steps"))?,
            "max_seconds" => seconds = v.parse().map_err(|_| invalid("max_seconds"))?,
            _ => return Err(invalid("unknown option")),
        }
    }
    if steps == 0 || steps > 2000 || seconds == 0 || seconds > 1800 {
        return Err(invalid("steps 1..2000, max_seconds 1..1800"));
    }
    if auxiliary_filter.is_some() && !carry {
        return Err(invalid("auxiliary filter requires carry=1"));
    }
    let out = out.ok_or_else(|| invalid("out required"))?;
    report_output::claim(&out)?;
    let result = match mode.as_str() {
        "fit" => run(&out, steps, seconds, carry, auxiliary_filter),
        "probe" => probe(&models.ok_or_else(|| invalid("models required"))?, &out),
        "stress" => stress(&models.ok_or_else(|| invalid("models required"))?, &out),
        _ => Err(invalid("mode fit|probe|stress")),
    };
    match &result {
        Ok(report) => fs::write(out.join("report.json"), serde_json::to_vec_pretty(report)?)?,
        Err(error) => fs::write(
            out.join("error.json"),
            serde_json::to_vec_pretty(&json!({"error":error.to_string()}))?,
        )?,
    }
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    result?;
    Ok(())
}
