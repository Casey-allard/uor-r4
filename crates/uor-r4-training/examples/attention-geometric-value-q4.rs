//! Strict-q4 value projection and connected answer backwards; zero optimizer updates.
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
    geometric_read_native::{CompiledGeometricRead, ReadSourceBinding},
    geometric_span_native::{CompiledSpanActions, SpanSourceBinding},
    geometric_stack::{logits_cross_entropy, StackModel},
    geometric_value_producer::{ValueProducerWeights, Q4_NATIVE_CHOICE_SURROGATE},
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
    composition_fit: PathBuf,
    out: PathBuf,
    seed: u64,
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
fn main() -> Result<()> {
    let mut cli = std::env::args().skip(1);
    let path = cli
        .next()
        .ok_or_else(|| invalid("usage: attention-geometric-value-q4 ARGS.json"))?;
    if cli.next().is_some() {
        return Err(invalid("one argument file required"));
    }
    let bytes = fs::read(path)?;
    let a: Args = serde_json::from_slice(&bytes)?;
    if ![1, 2].contains(&a.seed) || !(301..=840).contains(&a.maximum_seconds) {
        return Err(invalid("seed1/2 and explicit301..840s required"));
    }
    report_output::claim(&a.out)?;
    let started = Instant::now();
    fs::write(a.out.join("arguments.json"), bytes)?;
    let result = run(&a, started);
    if let Err(e) = &result {
        write(
            &a.out.join("failure.json"),
            &json!({"error":e.to_string(),"elapsed_seconds":started.elapsed().as_secs_f64(),"decision":"INCOMPLETE_RETAIN_NO_QUALITY_VERDICT"}),
        )?;
    }
    report_output::seal(&a.out)?;
    report_output::verify(&a.out)?;
    result
}
fn run(a: &Args, started: Instant) -> Result<()> {
    for root in [&a.parent, &a.no_read_fit, &a.composition_fit] {
        report_output::verify(root)?;
    }
    let parent_bytes = fs::read(a.parent.join("report.json"))?;
    let parent: Value = serde_json::from_slice(&parent_bytes)?;
    let scalar_bytes = fs::read(a.no_read_fit.join("report.json"))?;
    let scalar: Value = serde_json::from_slice(&scalar_bytes)?;
    let fit_bytes = fs::read(a.composition_fit.join("report.json"))?;
    let fit: Value = serde_json::from_slice(&fit_bytes)?;
    let expected_fit = if a.seed == 1 {
        "f805ea43fc8250f3dd5b549a459bdf40159361dc8e3715ec736d2b46db65d034"
    } else {
        "81616b9a950f31de32b6ecdf57480435eb15d0617a075d9eb75d2d07dfd3c0e8"
    };
    if sha256_bytes(&fit_bytes) != expected_fit {
        return Err(invalid("retained composition fit identity differs"));
    }
    if parent["complete"] != true
        || parent["cumulative_updates"] != 1280
        || parent["auxiliary_credit"] != "query_read"
        || scalar["complete"] != true
        || scalar["fit"]["completed_updates"] != 640
        || fit["complete"] != true
        || fit["mode"] != "fit"
        || fit["seed"] != a.seed
        || fit["fit"]["completed_updates"] != 640
        || fit["parent"]["query_parent_report_sha256"] != sha256_bytes(&parent_bytes)
        || fit["parent"]["no_read_fit_report_sha256"] != sha256_bytes(&scalar_bytes)
    {
        return Err(invalid("accepted query/scalar/composition lineage differs"));
    }
    let initial_rng = fit["fit"]["next_generator_state"]
        .as_u64()
        .ok_or_else(|| invalid("exact composition-final RNG missing"))?;
    let expected_rng = if a.seed == 1 {
        461503785751815875u64
    } else {
        1220542080958524937u64
    };
    if initial_rng != expected_rng {
        return Err(invalid("retained composition-final RNG differs"));
    }
    let model = StackModel::load(&a.model, &Device::Cpu)?;
    if model.config.width != 32
        || model.config.heads != 2
        || model.config.context != 128
        || model.config.vocab_size != 40
    {
        return Err(invalid("actual H2/width32/context128/V40 parent required"));
    }
    let tokenizer = fs::read(a.span_native.join("tokenizer-identity.bin"))?;
    let cs = a.parent.join("trained/context-source");
    let cn = a.parent.join("trained/native-context");
    let vs = a.parent.join("trained/value-source");
    let vn = a.parent.join("trained/native-values");
    let rn = a.parent.join("native-read");
    let ns = a.no_read_fit.join("no-read-source");
    let nn = a.no_read_fit.join("no-read-native");
    let bs = a.composition_fit.join("composition-source");
    let bn = a.composition_fit.join("composition-native");
    let frozen_roots = [
        ("base", a.model.as_path()),
        ("context-source", cs.as_path()),
        ("context-native", cn.as_path()),
        ("value-source", vs.as_path()),
        ("value-native", vn.as_path()),
        ("event-source", a.event_source.as_path()),
        ("event-native", a.event_native.as_path()),
        ("span-native", a.span_native.as_path()),
        ("potential-native", a.potential_native.as_path()),
        ("reducer-native", rn.as_path()),
        ("no-read-source", ns.as_path()),
        ("no-read-native", nn.as_path()),
        ("composition-source", bs.as_path()),
        ("composition-native", bn.as_path()),
    ];
    let frozen_files = file_snapshot(&frozen_roots)?;
    let frozen_base = variable_hashes(&model)?;
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
            .ok_or_else(|| invalid("spans missing"))?,
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
    let original = CompiledValueProducer::load(&vn, vp, &tokenizer)?;
    let np = NoReadSourcePaths {
        no_read_source: &ns,
        value: vp,
        context_native: &cn,
        value_native: &vn,
        reducer_native: &rn,
    };
    let null = CompiledNoRead::load(&nn, np, &tokenizer)?;
    let bank = CompositionWeights::load(&bs)?;
    let frozen_bank = parameter_hashes(bank.parameters())?;
    let bp = CompositionSourcePaths {
        composition_source: &bs,
        no_read: np,
        no_read_native: &nn,
    };
    let original_bank = CompiledComposition::load(&bn, bp, &tokenizer)?;
    if fit["compiled_metadata"] != serde_json::to_value(original_bank.metadata())? {
        return Err(invalid("composition parent metadata differs"));
    }
    original_bank.validate_for(&bank)?;
    let before = ValueProducerWeights::load_source(&vs)?;
    let mut quantization = Vec::new();
    for (name, v) in before.parameters() {
        let values = v.flatten_all()?.to_vec1::<f32>()?;
        let mut clipped = 0;
        let mut zero = 0;
        let mut rounded = 0;
        let mut max_error = 0f64;
        let mut squared = 0f64;
        for &x in &values {
            if !x.is_finite() {
                return Err(invalid("nonfinite retained coefficient"));
            }
            let code = (x * 4.).round().clamp(-7., 7.);
            clipped += usize::from(x.abs() > 1.75);
            rounded += usize::from(x != (x * 4.).round() / 4.);
            zero += usize::from(code == 0.);
            let error = (f64::from(x) - f64::from(code) / 4.).abs();
            max_error = max_error.max(error);
            squared += error * error;
        }
        quantization.push(json!({"name":name,"shape":v.dims(),"coefficients":values.len(),"clipped_shadows":clipped,"rounded_coefficients":rounded,"zero_codes":zero,"maximum_coefficient_error":max_error,"rms_error":(squared/values.len() as f64).sqrt()}));
    }
    let weights = before.into_q4()?;
    let qs = a.out.join("q4-value-source");
    weights.save_source(&qs)?;
    let weights = ValueProducerWeights::load_source(&qs)?;
    let source_hashes = parameter_hashes(weights.parameters())?;
    let packed = weights.packed_coefficients()?;
    if packed.len() != 64448 || !weights.is_q4() {
        return Err(invalid("strict source coefficient inventory differs"));
    }
    let qn = a.out.join("q4-value-native");
    let qvp = ValueProducerSourcePaths {
        value_source: &qs,
        context_source: &cs,
        context_dependencies: a.dependencies(),
    };
    CompiledValueProducer::compile(qvp, &tokenizer)?.save(&qn)?;
    let projected = CompiledValueProducer::load(&qn, qvp, &tokenizer)?;
    projected.validate_for(
        &weights,
        &ContextWeights::load_source(&cs, a.dependencies(), &tokenizer)?,
    )?;
    projected.validate_native_context(&context)?;
    let qnp = NoReadSourcePaths {
        no_read_source: &ns,
        value: qvp,
        context_native: &cn,
        value_native: &qn,
        reducer_native: &rn,
    };
    let qnn = a.out.join("rebound-no-read-native");
    CompiledNoRead::compile(&NoReadWeights::load(&ns)?, qnp, &tokenizer)?.save(&qnn)?;
    let qnull = CompiledNoRead::load(&qnn, qnp, &tokenizer)?;
    let null_files = same_numerical_files(&nn, &qnn)?;
    let qbp = CompositionSourcePaths {
        composition_source: &bs,
        no_read: qnp,
        no_read_native: &qnn,
    };
    let qbn = a.out.join("rebound-composition-native");
    CompiledComposition::compile(&bank, qbp, &tokenizer)?.save(&qbn)?;
    let qbank = CompiledComposition::load(&qbn, qbp, &tokenizer)?;
    let bank_files = same_numerical_files(&bn, &qbn)?;
    let preparation_seconds = started.elapsed().as_secs_f64();
    write(
        &a.out.join("input-identities.json"),
        &json!({"parent_report_sha256":sha256_bytes(&parent_bytes),"no_read_report_sha256":sha256_bytes(&scalar_bytes),"composition_report_sha256":sha256_bytes(&fit_bytes),"frozen_files":frozen_files,"frozen_base_variables":frozen_base,"frozen_bank_parameters":frozen_bank,"data_helper_sha256":sha256_bytes(include_bytes!("attention-geometric-value-learned/data.rs")),"initial_training_rng":initial_rng}),
    )?;
    let mut summaries = Vec::new();
    let evaluation_started = Instant::now();
    let mut complete = true;
    for panel in ["original", "stress"] {
        let inputs = fs::read(a.parent.join(if panel == "original" {
            "evaluation.json"
        } else {
            "stress.json"
        }))?;
        let episodes: Vec<data::Episode> = serde_json::from_slice(&inputs)?;
        let old_bytes = fs::read(a.composition_fit.join(format!("{panel}-rows.json")))?;
        let old_rows: Vec<Value> = serde_json::from_slice(&old_bytes)?;
        if episodes.len() != 128 || old_rows.len() != 128 {
            return Err(invalid("full retained128-row panels required"));
        }
        fs::write(a.out.join(format!("{panel}-episodes.json")), &inputs)?;
        let mut rows = Vec::new();
        let mut count = [0usize; 3];
        let mut gains = 0;
        let mut losses = 0;
        let mut source_native_changes = 0;
        let mut changed_atoms = 0;
        let mut max_delta = 0f32;
        let mut means = [0f64; 3];
        let mut max_value_delta = 0i64;
        for (index, e) in episodes.iter().enumerate() {
            if started.elapsed().as_secs() >= a.maximum_seconds - 60 {
                complete = false;
                break;
            }
            let t = e.ids.len();
            let (pl, ptrace, pvalues) = model.forward_geometric_composition_native_with_trace(
                &e.ids,
                1,
                t,
                &context,
                &events,
                &span,
                &potential,
                &reducer,
                &original,
                &null,
                &original_bank,
                false,
            )?;
            check_logits(&pl, t)?;
            check_trace(&pvalues, t)?;
            check_read(&ptrace, t)?;
            let (pp, pol) = answer(&pl, e.query)?;
            if old_rows[index]["episode"] != serde_json::to_value(e)?
                || old_rows[index]["native_prediction"] != pp
                || old_rows[index]["answer_logits"]["native"] != serde_json::to_value(&pol)?
            {
                return Err(invalid("actual composition parent row replay differs"));
            }
            let (sl, svalues) = model.forward_geometric_q4_values(
                &e.ids, 1, t, &context, &events, &span, &potential, &reducer, &projected, &qnull,
                &weights, &bank, false,
            )?;
            check_logits(&sl, t)?;
            check_trace(&svalues.trace, t)?;
            let (sp, sol) = answer(&sl, e.query)?;
            let (nl, ntrace, nvalues) = model.forward_geometric_composition_native_with_trace(
                &e.ids, 1, t, &context, &events, &span, &potential, &reducer, &projected, &qnull,
                &qbank, false,
            )?;
            check_logits(&nl, t)?;
            check_trace(&nvalues, t)?;
            check_read(&ntrace, t)?;
            let (np, nol) = answer(&nl, e.query)?;
            if ptrace.no_read_q24 != ntrace.no_read_q24 || ptrace.rows.len() != ntrace.rows.len() {
                return Err(invalid("q4 projection changes frozen null/reader layout"));
            }
            for (p, n) in ptrace.rows.iter().zip(&ntrace.rows) {
                if p.occurrence_weights_q31 != n.occurrence_weights_q31
                    || p.no_read_weight_q31 != n.no_read_weight_q31
                    || p.total_weight_q31 != n.total_weight_q31
                    || p.max_score_q24 != n.max_score_q24
                {
                    return Err(invalid("q4 projection changes address/age/NoRead weights"));
                }
            }
            let atoms = pvalues
                .packets
                .iter()
                .zip(&nvalues.packets)
                .map(|(p, n)| p.iter().zip(n).filter(|(x, y)| x != y).count())
                .sum::<usize>();
            changed_atoms += atoms;
            let delta = pvalues
                .values_q16
                .iter()
                .zip(&nvalues.values_q16)
                .map(|(p, n)| (i64::from(*p) - i64::from(*n)).abs())
                .max()
                .unwrap_or(0);
            max_value_delta = max_value_delta.max(delta);
            let numerical = sol
                .iter()
                .zip(&nol)
                .map(|(x, y)| (x - y).abs())
                .fold(0f32, f32::max);
            max_delta = max_delta.max(numerical);
            source_native_changes += usize::from(sp != np);
            let pc = pp == e.answer as usize;
            let nc = np == e.answer as usize;
            count[0] += usize::from(pc);
            count[1] += usize::from(sp == e.answer as usize);
            count[2] += usize::from(nc);
            gains += usize::from(!pc && nc);
            losses += usize::from(pc && !nc);
            for (m, l) in means.iter_mut().zip([&pol, &sol, &nol]) {
                *m += ce(l, e.answer as usize)?;
            }
            let packet_mismatches = svalues
                .trace
                .packets
                .iter()
                .zip(&nvalues.packets)
                .enumerate()
                .filter(|(_, (source, native))| source != native)
                .map(|(lane_index, (source, native))| {
                    json!({
                        "lane_index_b_h_t_4": lane_index, "source": source, "native": native
                    })
                })
                .collect::<Vec<_>>();
            let coordinate_mismatches = svalues
                .trace
                .values_q16
                .iter()
                .zip(&nvalues.values_q16)
                .enumerate()
                .filter(|(_, (source, native))| source != native)
                .map(|(coordinate_index, (source, native))| {
                    json!({
                        "coordinate_index_b_h_t_16": coordinate_index,
                        "source_q16": source, "native_q16": native
                    })
                })
                .collect::<Vec<_>>();
            rows.push(json!({"packet_mismatches":packet_mismatches,"coordinate_mismatches":coordinate_mismatches,"index":index,"episode":e,"parent_prediction":pp,"source_prediction":sp,"native_prediction":np,"answer_logits":{"parent":pol,"source":sol,"native":nol},"changed_primitive_atoms":atoms,"maximum_value_q16_delta":delta,"source_native_packet_trace_equal":svalues.trace==nvalues,"maximum_answer_logit_delta":numerical,"source_native_answer_match":sp==np,"query_heads":(0..2).map(|h|ntrace.rows[h*t+e.query].clone()).collect::<Vec<_>>(),"query_no_read_q24":(0..2).map(|h|ntrace.no_read_q24[h*t+e.query]).collect::<Vec<_>>(),"all_frozen_reader_weights_identical":true}));
            if (index + 1) % 32 == 0 {
                write(&a.out.join(format!("{panel}-rows.json")), &rows)?;
            }
        }
        write(&a.out.join(format!("{panel}-rows.json")), &rows)?;
        let denominator = rows.len().max(1) as f64;
        summaries.push(json!({"panel":panel,"rows":rows.len(),"source_native_packet_trace_mismatches":rows.iter().filter(|r|r["source_native_packet_trace_equal"]!=true).count(),"answers":{"parent":count[0],"source":count[1],"native":count[2]},"gains":gains,"losses":losses,"source_native_answer_changes":source_native_changes,"changed_primitive_atoms":changed_atoms,"maximum_value_q16_delta":max_value_delta,"maximum_answer_logit_delta":max_delta,"mean_answer_ce":{"parent":means[0]/denominator,"source":means[1]/denominator,"native":means[2]/denominator},"input_sha256":sha256_bytes(&inputs),"parent_rows_sha256":sha256_bytes(&old_bytes)}));
        if !complete {
            break;
        }
    }
    let evaluation_seconds = evaluation_started.elapsed().as_secs_f64();
    let mut rng = data::Rng(initial_rng);
    let mut batches = Vec::new();
    fs::create_dir(a.out.join("diagnostic-inputs"))?;
    for batch in 0..2 {
        if started.elapsed().as_secs() >= a.maximum_seconds - 300 {
            complete = false;
            break;
        }
        let rng_before = rng.0;
        let absolute_step = 2560 + batch;
        let n = [2, 4][absolute_step % 2];
        let episodes = (0..8)
            .map(|_| {
                let facts = data::draw(&mut rng, n);
                let query = rng.next(n);
                let noise = (0..n).map(|_| data::noise(&mut rng, 4)).collect::<Vec<_>>();
                let qnoise = data::noise(&mut rng, 4);
                data::episode(
                    &facts,
                    &noise,
                    &facts[query].0,
                    &qnoise,
                    absolute_step,
                    "joint_train",
                )
            })
            .collect::<Vec<_>>();
        let bytes = serde_json::to_vec(&episodes)?;
        fs::write(
            a.out.join(format!("diagnostic-inputs/batch-{batch}.json")),
            &bytes,
        )?;
        let (ids, targets, mask, time) = data::batch(&episodes);
        if mask.iter().sum::<f32>() != 8.
            || episodes.iter().enumerate().any(|(b, e)| {
                e.query + 1 != e.ids.len()
                    || e.ids[e.query] != 34
                    || (0..time).any(|t| {
                        mask[b * time + t] != if t == e.query { 1. } else { 0. }
                            || (t == e.query && targets[b * time + t] != e.answer)
                    })
            })
        {
            return Err(invalid("diagnostic actual answer mask differs"));
        }
        let calc = Instant::now();
        let (logits, output) = model.forward_geometric_q4_values(
            &ids, 8, time, &context, &events, &span, &potential, &reducer, &projected, &qnull,
            &weights, &bank, false,
        )?;
        let loss = logits_cross_entropy(&logits, &targets, Some(&mask))?;
        let value = loss.to_scalar::<f32>()?;
        if !value.is_finite() {
            return Err(invalid("nonfinite ordinary answer loss"));
        }
        let gradients = loss.backward()?;
        let mut families = serde_json::Map::new();
        for (name, var) in weights.parameters() {
            let g = gradients
                .get(var.as_tensor())
                .ok_or_else(|| invalid(format!("q4 answer gradient disconnected:{name}")))?
                .flatten_all()?
                .to_vec1::<f32>()?;
            if g.iter().any(|x| !x.is_finite()) {
                return Err(invalid("nonfinite value gradient"));
            }
            families.insert(name.clone(),json!({"elements":g.len(),"nonzero":g.iter().filter(|x|**x!=0.).count(),"l2":g.iter().map(|x|f64::from(*x).powi(2)).sum::<f64>().sqrt()}));
        }
        if bank
            .parameters()
            .values()
            .any(|v| gradients.get(v.as_tensor()).is_some())
        {
            return Err(invalid("frozen bank received gradient"));
        }
        let zero_atoms = output
            .trace
            .packets
            .iter()
            .flat_map(|p| p.iter())
            .filter(|p| {
                p.status == uor_r4_training::geometric_value_native::ValuePacketStatus::PresentZero
            })
            .count();
        let mut actual_zero_atoms = 0;
        for (b, e) in episodes.iter().enumerate() {
            for h in 0..2 {
                for t in 0..e.ids.len() {
                    for l in 0..4 {
                        let index = ((b * 2 + h) * time + t) * 4 + l;
                        let pair =
                            output.trace.packets.get(index).ok_or_else(|| {
                                invalid("diagnostic primitive trace shape differs")
                            })?;
                        actual_zero_atoms+=pair.iter().filter(|p|p.status==uor_r4_training::geometric_value_native::ValuePacketStatus::PresentZero).count();
                    }
                }
            }
        }
        batches.push(json!({"batch":batch,"absolute_generator_step":absolute_step,"rng_before":rng_before,"rng_after":rng.0,"episode_sha256":sha256_bytes(&bytes),"time":time,"actual_positions":episodes.iter().map(|e|e.ids.len()).sum::<usize>(),"padded_positions":ids.len(),"answer_denominator":8,"answer_mean_ce":value,"calculation_seconds":calc.elapsed().as_secs_f64(),"ordinary_answer_family_gradients":families,"present_zero_atoms_all_padded_positions":zero_atoms,"present_zero_atoms_actual_tokens":actual_zero_atoms,"frozen_bank_gradient_absent":true,"optimizer_updates":0}));
    }
    if parameter_hashes(weights.parameters())? != source_hashes
        || parameter_hashes(bank.parameters())? != frozen_bank
        || variable_hashes(&model)? != frozen_base
        || file_snapshot(&frozen_roots)? != frozen_files
    {
        return Err(invalid(
            "construction/backwards mutated source or frozen parent",
        ));
    }
    complete = complete && batches.len() == 2;
    write(&a.out.join("diagnostic-batches.json"), &batches)?;
    write(
        &a.out.join("report.json"),
        &json!({"schema":"uor-r4.geometric-value-q4-construction/1","complete":complete,"seed":a.seed,"optimizer_updates":0,"source_policy":weights.config(),"hard_forward_policy":Q4_NATIVE_CHOICE_SURROGATE,"packed_coefficient_bytes":packed.len(),"packed_sha256":sha256_bytes(&packed),"coefficient_quantization":quantization,"compiled_metadata":projected.metadata(),"rebound_no_read_numerical_files_unchanged":null_files,"rebound_composition_numerical_files_unchanged":bank_files,"panels":summaries,"diagnostic_batches":batches,"initial_training_rng":initial_rng,"next_training_rng":initial_rng,"observed_diagnostic_rng_after":rng.0,"preparation_seconds":preparation_seconds,"evaluation_seconds":evaluation_seconds,"elapsed_seconds":started.elapsed().as_secs_f64(),"decision":if complete{"CONNECTED_Q4_PROJECTION_AND_ANSWER_BACKWARDS_RETAIN_ALL_ROWS"}else{"PARTIAL_RETAIN_NO_QUALITY_VERDICT"},"scope":"Strict q4 source, derived fixed-geometry tables and frozen-bank value adjoint. Hard choices use current packed coefficients and native integer codes; backward probabilities and source real-softmax/F32 versus native Q24/Q16/LUT drift remain explicit; no full attention/language/energy or geometry advantage. Other wide learned coefficients and float tail remain. No fit performed."}),
    )?;
    println!(
        "q4 value construction complete={complete}; {}",
        a.out.display()
    );
    Ok(())
}
