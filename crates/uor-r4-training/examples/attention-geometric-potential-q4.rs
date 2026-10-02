//! Fixed strict-q4 potential construction and ordinary answer-credit comparison.
use candle_core::{Device, Tensor, Var};
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
    geometric_composition::CompositionWeights,
    geometric_composition_native::{CompiledComposition, CompositionSourcePaths},
    geometric_context::{CompiledContext, ContextSourcePaths, ContextWeights},
    geometric_event::CompiledEvents,
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
    model: PathBuf,
    parent: PathBuf,
    event_source: PathBuf,
    event_native: PathBuf,
    span_native: PathBuf,
    potential_native: PathBuf,
    no_read_fit: PathBuf,
    composition_fit: PathBuf,
    value_fit: PathBuf,
    out: PathBuf,
    seed: u64,
    maximum_seconds: u64,
}
impl Args {
    fn dependencies<'a>(&'a self, potential: &'a Path) -> ContextSourcePaths<'a> {
        ContextSourcePaths {
            base: &self.model,
            event_source: &self.event_source,
            event_native: &self.event_native,
            span_native: &self.span_native,
            potential_native: potential,
        }
    }
}
fn invalid(m: impl Into<String>) -> uor_r4_training::TrainingError {
    uor_r4_training::TrainingError::Invalid(m.into())
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

fn parameter_hashes(vars: &BTreeMap<String, Var>) -> Result<BTreeMap<String, String>> {
    vars.iter()
        .map(|(name, v)| {
            Ok((
                name.clone(),
                sha256_bytes(
                    &v.flatten_all()?
                        .to_vec1::<f32>()?
                        .iter()
                        .flat_map(|x| x.to_bits().to_le_bytes())
                        .collect::<Vec<_>>(),
                ),
            ))
        })
        .collect()
}
fn same_numerical_files(old: &Path, new: &Path) -> Result<Vec<String>> {
    let mut names = Vec::new();
    for entry in fs::read_dir(old)? {
        let entry = entry?;
        let name = entry.file_name();
        if name == "metadata.json" {
            continue;
        }
        if !entry.file_type()?.is_file() || fs::read(entry.path())? != fs::read(new.join(&name))? {
            return Err(invalid("rebinding changed frozen numerical file"));
        }
        names.push(name.to_string_lossy().into_owned());
    }
    names.sort();
    Ok(names)
}
fn ce(logits: &[f32], target: usize) -> Result<f64> {
    if target >= logits.len() {
        return Err(invalid("answer target out of range"));
    }
    let m = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max) as f64;
    Ok(m + logits
        .iter()
        .map(|x| (f64::from(*x) - m).exp())
        .sum::<f64>()
        .ln()
        - f64::from(logits[target]))
}
fn check_trace(
    t: &uor_r4_training::geometric_value_producer::ValueProducerTrace,
    time: usize,
) -> Result<()> {
    if t.batch != 1
        || t.heads != 2
        || t.time != time
        || t.occurrence_valid.len() != time
        || t.packets.len() != 2 * time * 4
        || t.values_q16.len() != 2 * time * 16
    {
        return Err(invalid("projection value trace shape differs"));
    }
    Ok(())
}
fn check_logits(t: &Tensor, time: usize) -> Result<()> {
    if t.dims() != [time, 40] || t.to_vec2::<f32>()?.iter().flatten().any(|x| !x.is_finite()) {
        return Err(invalid("full projection logits shape/finiteness differs"));
    }
    Ok(())
}
fn check_read(
    t: &uor_r4_training::geometric_composition_native::ComposedReadTrace,
    time: usize,
) -> Result<()> {
    if t.batch != 1
        || t.heads != 2
        || t.time != time
        || t.value_width != 32
        || t.rows.len() != 2 * time
        || t.no_read_q24.len() != 2 * time
        || t.values_q16.len() != 2 * time * 32
        || t.rows.iter().enumerate().any(|(i, r)| {
            r.output_q16.len() != 32 || r.occurrence_weights_q31.len() != i % time + 1
        })
    {
        return Err(invalid("projection read trace shape differs"));
    }
    Ok(())
}

fn verify_current_scores(
    output: &uor_r4_training::geometric_potential_q4::PotentialQ4Output,
    potential: &CompiledGeometricPotentials,
    batch: usize,
    time: usize,
    lanes: usize,
) -> Result<usize> {
    let heads = 2;
    let length = batch * time * heads * lanes;
    if output.content_codes.len() != length
        || output.context_codes.len() != length
        || output.scores_q24.len() != batch * heads * time * time
    {
        return Err(invalid("actual potential score/code shape differs"));
    }
    let mut count = 0;
    for b in 0..batch {
        for h in 0..heads {
            for q in 0..time {
                for k in 0..time {
                    let index = ((b * heads + h) * time + q) * time + k;
                    if k > q {
                        if output.scores_q24[index] != 0 {
                            return Err(invalid(
                                "future raw potential score is not masked placeholder",
                            ));
                        }
                        continue;
                    }
                    let qi = ((b * time + q) * heads + h) * lanes;
                    let ki = ((b * time + k) * heads + h) * lanes;
                    let expected = potential.score_pair_codes(
                        h,
                        &output.content_codes[qi..qi + lanes],
                        &output.content_codes[ki..ki + lanes],
                        &output.context_codes[qi..qi + lanes],
                        &output.context_codes[ki..ki + lanes],
                    )?;
                    if output.scores_q24[index] != expected {
                        return Err(invalid(
                            "actual current-coefficient source/native Q24 score differs",
                        ));
                    }
                    count += 1;
                }
            }
        }
    }
    Ok(count)
}
fn main() -> Result<()> {
    let mut cli = std::env::args().skip(1);
    let path = cli
        .next()
        .ok_or_else(|| invalid("usage: attention-geometric-potential-q4 ARGS.json"))?;
    if cli.next().is_some() {
        return Err(invalid("one argument file required"));
    }
    let bytes = fs::read(path)?;
    let a: Args = serde_json::from_slice(&bytes)?;
    if ![1, 2].contains(&a.seed) || !(61..=240).contains(&a.maximum_seconds) {
        return Err(invalid("seed1/2 and explicit61..240 seconds required"));
    }
    report_output::claim(&a.out)?;
    let at = Instant::now();
    fs::write(a.out.join("arguments.json"), bytes)?;
    let result = run(&a, at);
    if let Err(e) = &result {
        write(
            &a.out.join("failure.json"),
            &json!({"error":e.to_string(),"elapsed_seconds":at.elapsed().as_secs_f64(),"decision":"INCOMPLETE_RETAIN_NO_QUALITY_VERDICT"}),
        )?;
    }
    report_output::seal(&a.out)?;
    report_output::verify(&a.out)?;
    result
}
fn run(a: &Args, at: Instant) -> Result<()> {
    for root in [&a.parent, &a.no_read_fit, &a.composition_fit, &a.value_fit] {
        report_output::verify(root)?;
    }
    let report_bytes = fs::read(a.value_fit.join("report.json"))?;
    let prior: Value = serde_json::from_slice(&report_bytes)?;
    let expected = if a.seed == 1 {
        "f9c18b70e48e21429a58965fbb2c461d6e97942f81c2ab8c2cc5a700ba3cee3c"
    } else {
        "a341dcfbf9bf87e131be7936e92edbe36ce823e4cce983f25b21a2ec8150e6e6"
    };
    if sha256_bytes(&report_bytes) != expected
        || prior["complete"] != true
        || prior["fit"]["completed_updates"] != 640
        || prior["seed"] != a.seed
    {
        return Err(invalid("completed q4 value parent identity differs"));
    }
    let model = StackModel::load(&a.model, &Device::Cpu)?;
    if model.config.width != 32
        || model.config.heads != 2
        || model.config.context != 128
        || model.config.vocab_size != 40
    {
        return Err(invalid("actual V40/H2/width32/context128 parent required"));
    }
    let tokenizer = fs::read(a.span_native.join("tokenizer-identity.bin"))?;
    let cs = a.parent.join("trained/context-source");
    let cn = a.parent.join("trained/native-context");
    let rn = a.parent.join("native-read");
    let ns = a.no_read_fit.join("no-read-source");
    let bs = a.composition_fit.join("composition-source");
    let vs = a.value_fit.join("q4-value-source");
    let vn = a.value_fit.join("q4-value-native");
    let nn = a.value_fit.join("rebound-no-read-native");
    let bn = a.value_fit.join("rebound-composition-native");
    let frozen_roots = [
        ("base", a.model.as_path()),
        ("context-source", cs.as_path()),
        ("context-native", cn.as_path()),
        ("event-source", a.event_source.as_path()),
        ("event-native", a.event_native.as_path()),
        ("span-native", a.span_native.as_path()),
        ("potential", a.potential_native.as_path()),
        ("reducer", rn.as_path()),
        ("value-source", vs.as_path()),
        ("value-native", vn.as_path()),
        ("null-source", ns.as_path()),
        ("null-native", nn.as_path()),
        ("bank-source", bs.as_path()),
        ("bank-native", bn.as_path()),
    ];
    let frozen_files = file_snapshot(&frozen_roots)?;
    let frozen_base = variable_hashes(&model)?;
    let oldpaths = a.dependencies(&a.potential_native);
    let cw = ContextWeights::load_source(&cs, oldpaths, &tokenizer)?;
    let oldcontext = CompiledContext::load(&cn, &cs, oldpaths, &tokenizer)?;
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
            .ok_or_else(|| invalid("span configuration missing"))?,
    )?;
    let binding = PotentialSourceBinding::from_directory(&a.model, &tokenizer)?;
    let oldpotential = CompiledGeometricPotentials::load(&a.potential_native, &binding)?;
    let oldreducer = CompiledGeometricRead::load(
        &rn,
        &ReadSourceBinding::from_directory(&a.model, &tokenizer, &oldpotential, 2)?,
    )?;
    let oldvp = ValueProducerSourcePaths {
        value_source: &vs,
        context_source: &cs,
        context_dependencies: oldpaths,
    };
    let oldvalues = CompiledValueProducer::load(&vn, oldvp, &tokenizer)?;
    let oldnp = NoReadSourcePaths {
        no_read_source: &ns,
        value: oldvp,
        context_native: &cn,
        value_native: &vn,
        reducer_native: &rn,
    };
    let oldnull = CompiledNoRead::load(&nn, oldnp, &tokenizer)?;
    let oldbank = CompiledComposition::load(
        &bn,
        CompositionSourcePaths {
            composition_source: &bs,
            no_read: oldnp,
            no_read_native: &nn,
        },
        &tokenizer,
    )?;
    let valueweights = ValueProducerWeights::load_source(&vs)?;
    let valuehash = parameter_hashes(valueweights.parameters())?;
    let bankweights = CompositionWeights::load(&bs)?;
    let bankhash = parameter_hashes(bankweights.parameters())?;
    let context_hash = parameter_hashes(cw.parameters())?;
    let weights = PotentialQ4Weights::from_base(&a.model, &tokenizer)?;
    let coefficient_hash = parameter_hashes(weights.parameters())?;
    let ps = a.out.join("potential-q4-source");
    let pn = a.out.join("potential-q4-native");
    weights.save(&ps)?;
    let weights = PotentialQ4Weights::load(&ps)?;
    if parameter_hashes(weights.parameters())? != coefficient_hash
        || weights.packed_coefficients()?.len() != 8320
    {
        return Err(invalid("strict potential source reload differs"));
    }
    CompiledGeometricPotentials::compile_q4(&weights, &ps, &binding)?.save(&pn)?;
    let potential = CompiledGeometricPotentials::load(&pn, &binding)?;
    potential.validate_q4_source(&weights)?;
    let paths = a.dependencies(&pn);
    let ncs = a.out.join("rebound-context-source");
    let ncn = a.out.join("rebound-context-native");
    cw.save_source(&ncs, paths, &tokenizer)?;
    let cw = ContextWeights::load_source(&ncs, paths, &tokenizer)?;
    if parameter_hashes(cw.parameters())? != context_hash {
        return Err(invalid("rebound context coefficients changed"));
    }
    CompiledContext::compile(&cw, &ncs, paths, &tokenizer)?.save(&ncn)?;
    let context = CompiledContext::load(&ncn, &ncs, paths, &tokenizer)?;
    let context_files = same_numerical_files(&cn, &ncn)?;
    let nrn = a.out.join("rebound-native-read");
    let readbinding = ReadSourceBinding::from_directory(&a.model, &tokenizer, &potential, 2)?;
    let age = model
        .variables()
        .get("layers.02.read.age")
        .ok_or_else(|| invalid("frozen age missing"))?
        .as_tensor();
    CompiledGeometricRead::compile(age, &potential, &readbinding)?.save(&nrn)?;
    let reducer = CompiledGeometricRead::load(&nrn, &readbinding)?;
    let reducer_files = same_numerical_files(&rn, &nrn)?;
    let nvn = a.out.join("rebound-value-native");
    let vp = ValueProducerSourcePaths {
        value_source: &vs,
        context_source: &ncs,
        context_dependencies: paths,
    };
    CompiledValueProducer::compile(vp, &tokenizer)?.save(&nvn)?;
    let values = CompiledValueProducer::load(&nvn, vp, &tokenizer)?;
    let value_files = same_numerical_files(&vn, &nvn)?;
    values.validate_for(&valueweights, &cw)?;
    values.validate_native_context(&context)?;
    let nnn = a.out.join("rebound-no-read-native");
    let np = NoReadSourcePaths {
        no_read_source: &ns,
        value: vp,
        context_native: &ncn,
        value_native: &nvn,
        reducer_native: &nrn,
    };
    CompiledNoRead::compile(&NoReadWeights::load(&ns)?, np, &tokenizer)?.save(&nnn)?;
    let null = CompiledNoRead::load(&nnn, np, &tokenizer)?;
    let null_files = same_numerical_files(&nn, &nnn)?;
    let nbn = a.out.join("rebound-composition-native");
    let bp = CompositionSourcePaths {
        composition_source: &bs,
        no_read: np,
        no_read_native: &nnn,
    };
    CompiledComposition::compile(&bankweights, bp, &tokenizer)?.save(&nbn)?;
    let bank = CompiledComposition::load(&nbn, bp, &tokenizer)?;
    let bank_files = same_numerical_files(&bn, &nbn)?;
    let construction_seconds = at.elapsed().as_secs_f64();
    // Exercise a loaded original row before diagnostics and complete-panel evaluation.
    let original: Vec<data::Episode> =
        serde_json::from_slice(&fs::read(a.value_fit.join("original-episodes.json"))?)?;
    let first = original
        .first()
        .ok_or_else(|| invalid("parent panel empty"))?;
    let (cheap, _, cheap_values) = model.forward_geometric_composition_native_with_trace(
        &first.ids,
        1,
        first.ids.len(),
        &context,
        &events,
        &span,
        &potential,
        &reducer,
        &values,
        &null,
        &bank,
        false,
    )?;
    check_logits(&cheap, first.ids.len())?;
    let prefix_time = first.ids.len() / 2;
    let (prefix, _, prefix_values) = model.forward_geometric_composition_native_with_trace(
        &first.ids[..prefix_time],
        1,
        prefix_time,
        &context,
        &events,
        &span,
        &potential,
        &reducer,
        &values,
        &null,
        &bank,
        false,
    )?;
    let prefix_logits = prefix.to_vec2::<f32>()?;
    let full_logits = cheap.to_vec2::<f32>()?;
    let prefix_delta = prefix_logits
        .iter()
        .zip(&full_logits)
        .flat_map(|(a, b)| a.iter().zip(b).map(|(a, b)| (a - b).abs()))
        .fold(0f32, f32::max);
    for h in 0..2 {
        for t in 0..prefix_time {
            for lane in 0..4 {
                if prefix_values.packets[(h * prefix_time + t) * 4 + lane]
                    != cheap_values.packets[(h * first.ids.len() + t) * 4 + lane]
                {
                    return Err(invalid("future prefix changes native packet"));
                }
            }
        }
    }
    if prefix_values.occurrence_valid != cheap_values.occurrence_valid[..prefix_time] {
        return Err(invalid("future prefix changes native occurrence admission"));
    }
    write(
        &a.out.join("cheap-loaded-prefix.json"),
        &json!({"prefix_time":prefix_time,"full_time":first.ids.len(),"native_prefix_packet_equality":true,"native_prefix_occurrence_equality":true,"maximum_prefix_logit_delta":prefix_delta,"native_float_tail_prefix_bits_equal":prefix_logits==full_logits[..prefix_time]}),
    )?;
    let mut diagnostic_rng = data::Rng(
        prior["fit"]["next_generator_state"]
            .as_u64()
            .ok_or_else(|| invalid("exact parent RNG missing"))?,
    );
    let initial_rng = diagnostic_rng.0;
    let mut gradients = Vec::new();
    for absolute in 3200..3202 {
        if at.elapsed().as_secs() >= a.maximum_seconds - 60 {
            return Err(invalid(
                "construction exhausted backward reserve; no quality verdict",
            ));
        }
        let facts_count = [2, 4][absolute % 2];
        let episodes = (0..8)
            .map(|_| {
                let facts = data::draw(&mut diagnostic_rng, facts_count);
                let query = diagnostic_rng.next(facts_count);
                let noise = (0..facts_count)
                    .map(|_| data::noise(&mut diagnostic_rng, 4))
                    .collect::<Vec<_>>();
                let qnoise = data::noise(&mut diagnostic_rng, 4);
                data::episode(
                    &facts,
                    &noise,
                    &facts[query].0,
                    &qnoise,
                    absolute,
                    "potential_construction_diagnostic",
                )
            })
            .collect::<Vec<_>>();
        let (ids, targets, mask, t) = data::batch(&episodes);
        if mask.iter().sum::<f32>() != 8.0 || t > 128 {
            return Err(invalid("diagnostic answer mask/context differs"));
        }
        write(
            &a.out.join(format!("diagnostic-{absolute}-episodes.json")),
            &episodes,
        )?;
        let grad_at = Instant::now();
        let (logits, score, _) = model.forward_geometric_q4_potential(
            &ids, 8, t, &context, &events, &span, &potential, &reducer, &values, &null, &bank,
            &weights, false,
        )?;
        let exact_scores = verify_current_scores(&score, &potential, 8, t, 4)?;
        let loss = logits_cross_entropy(&logits, &targets, Some(&mask))?;
        let g = loss.backward()?;
        let mut families = BTreeMap::new();
        for (name, var) in weights.parameters() {
            let grad = g
                .get(var.as_tensor())
                .ok_or_else(|| invalid(format!("potential answer gradient disconnected:{name}")))?
                .flatten_all()?
                .to_vec1::<f32>()?;
            if grad.iter().any(|x| !x.is_finite()) {
                return Err(invalid("nonfinite potential answer gradient"));
            }
            families.insert(name.clone(),json!({"l2":grad.iter().map(|x|f64::from(*x).powi(2)).sum::<f64>().sqrt(),"nonzero":grad.iter().filter(|x|**x!=0.).count(),"elements":grad.len()}));
        }
        gradients.push(json!({"absolute_draw":absolute,"rng_after_diagnostic_draw":diagnostic_rng.0,"batch_time":t,"actual_positions":episodes.iter().map(|e|e.ids.len()).sum::<usize>(),"padded_positions":8*t,"answer_denominator":8,"exact_causal_score_pairs":exact_scores,"answer_ce":loss.to_scalar::<f32>()?,"elapsed_seconds":grad_at.elapsed().as_secs_f64(),"families":families,"optimizer_updates":0}));
    }
    let mut panels = Vec::new();
    let mut complete = true;
    for name in ["original", "stress"] {
        let input = fs::read(a.value_fit.join(format!("{name}-episodes.json")))?;
        let episodes: Vec<data::Episode> = serde_json::from_slice(&input)?;
        let parent_bytes = fs::read(a.value_fit.join(format!("{name}-rows.json")))?;
        let parent: Vec<Value> = serde_json::from_slice(&parent_bytes)?;
        if episodes.len() != 128 || parent.len() != 128 {
            return Err(invalid("full retained panel required"));
        }
        fs::write(a.out.join(format!("{name}-episodes.json")), &input)?;
        let mut rows = Vec::new();
        let mut counts = [0; 3];
        let mut means = [0.; 3];
        let (mut gains, mut losses) = (0, 0);
        let (mut answer_changes, mut packet_changes) = (0, 0);
        let mut max_delta = 0f32;
        for (i, e) in episodes.iter().enumerate() {
            if at.elapsed().as_secs() >= a.maximum_seconds {
                complete = false;
                break;
            }
            let t = e.ids.len();
            let (pl, pr, pv) = model.forward_geometric_composition_native_with_trace(
                &e.ids,
                1,
                t,
                &oldcontext,
                &events,
                &span,
                &oldpotential,
                &oldreducer,
                &oldvalues,
                &oldnull,
                &oldbank,
                false,
            )?;
            let (sl, score, sv) = model.forward_geometric_q4_potential(
                &e.ids, 1, t, &context, &events, &span, &potential, &reducer, &values, &null,
                &bank, &weights, false,
            )?;
            let (nl, nr, nv) = model.forward_geometric_composition_native_with_trace(
                &e.ids, 1, t, &context, &events, &span, &potential, &reducer, &values, &null,
                &bank, false,
            )?;
            let exact_scores = verify_current_scores(&score, &potential, 1, t, 4)?;
            for l in [&pl, &sl, &nl] {
                check_logits(l, t)?;
            }
            for v in [&pv, &sv, &nv] {
                check_trace(v, t)?;
            }
            check_read(&pr, t)?;
            check_read(&nr, t)?;
            let (pp, pol) = answer(&pl, e.query)?;
            let (sp, sol) = answer(&sl, e.query)?;
            let (np, nol) = answer(&nl, e.query)?;
            if parent[i]["episode"] != serde_json::to_value(e)?
                || parent[i]["native_prediction"] != pp
                || parent[i]["answer_logits"]["native"] != serde_json::to_value(&pol)?
            {
                return Err(invalid("actual q4 value parent replay differs"));
            }
            if pv != nv || sv != nv || pr.no_read_q24 != nr.no_read_q24 {
                return Err(invalid("potential lowering changed frozen values/null"));
            }
            let delta = sl
                .to_vec2::<f32>()?
                .iter()
                .zip(nl.to_vec2::<f32>()?)
                .flat_map(|(s, n)| s.iter().zip(n).map(|(s, n)| (s - n).abs()))
                .fold(0f32, f32::max);
            max_delta = max_delta.max(delta);
            answer_changes += usize::from(sp != np);
            packet_changes += usize::from(sv != nv);
            for (j, pred) in [pp, sp, np].into_iter().enumerate() {
                counts[j] += usize::from(pred == e.answer as usize);
            }
            for (j, l) in [&pol, &sol, &nol].into_iter().enumerate() {
                means[j] += ce(l, e.answer as usize)?;
            }
            gains += usize::from(pp != e.answer as usize && np == e.answer as usize);
            losses += usize::from(pp == e.answer as usize && np != e.answer as usize);
            let oldheads = (0..2)
                .map(|h| pr.rows[h * t + e.query].clone())
                .collect::<Vec<_>>();
            let newheads = (0..2)
                .map(|h| nr.rows[h * t + e.query].clone())
                .collect::<Vec<_>>();
            rows.push(json!({"index":i,"episode":e,"parent_prediction":pp,"source_prediction":sp,"native_prediction":np,"answer_logits":{"parent":pol,"source":sol,"native":nol},"old_query_heads":oldheads,"new_query_heads":newheads,"source_scores_q24":score.scores_q24,"exact_causal_score_pairs":exact_scores,"source_native_packets_equal":sv==nv,"frozen_packets_equal":pv==nv,"frozen_no_read_equal":pr.no_read_q24==nr.no_read_q24,"maximum_actual_position_logit_delta":delta}));
        }
        if !rows.is_empty() {
            for m in &mut means {
                *m /= rows.len() as f64;
            }
        }
        write(&a.out.join(format!("{name}-rows.json")), &rows)?;
        panels.push(json!({"panel":name,"rows":rows.len(),"requested_rows":128,"input_sha256":sha256_bytes(&input),"parent_rows_sha256":sha256_bytes(&parent_bytes),"answers":{"parent":counts[0],"source":counts[1],"native":counts[2]},"mean_answer_ce":{"parent":means[0],"source":means[1],"native":means[2]},"gains":gains,"losses":losses,"source_native_answer_changes":answer_changes,"source_native_packet_mismatches":packet_changes,"maximum_actual_position_logit_delta":max_delta}));
    }
    if file_snapshot(&frozen_roots)? != frozen_files
        || variable_hashes(&model)? != frozen_base
        || parameter_hashes(weights.parameters())? != coefficient_hash
        || parameter_hashes(valueweights.parameters())? != valuehash
        || parameter_hashes(bankweights.parameters())? != bankhash
    {
        return Err(invalid(
            "construction changed frozen coefficient/source identity",
        ));
    }
    write(
        &a.out.join("report.json"),
        &json!({"schema":"uor-r4.geometric-potential-q4-construction/1","complete":complete,"seed":a.seed,"optimizer_updates":0,"parent_report_sha256":sha256_bytes(&report_bytes),"construction_seconds":construction_seconds,"elapsed_seconds":at.elapsed().as_secs_f64(),"source_policy":weights.config(),"packed_sha256":sha256_bytes(&weights.packed_coefficients()?),"compiled_metadata":potential.metadata(),"gradients":gradients,"initial_training_rng":initial_rng,"next_training_rng":initial_rng,"diagnostic_rng_after":diagnostic_rng.0,"panels":panels,"unchanged_numerical_files":{"context":context_files,"reducer":reducer_files,"value":value_files,"no_read":null_files,"bank":bank_files},"scope":"Fixed q4 coefficient and changed Q25 observation-basis construction together; no coefficient-only attribution. Exposed development, no optimizer fit or whole-language/native/energy qualification.","decision":if complete{"CONSTRUCTION_COMPLETE_RETAIN_ALL_ROWS"}else{"INCOMPLETE_RETAIN_NO_QUALITY_VERDICT"}}),
    )?;
    Ok(())
}
