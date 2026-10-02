//! Fixed answer-only strict-q4 value continuation through the frozen learned bank.
use candle_core::{Device, Tensor, Var};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
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
#[path = "attention-geometric-value-q4-fit/fit.rs"]
mod fit;
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
    projection: PathBuf,
    mode: fit::Mode,
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
        .ok_or_else(|| invalid("usage: attention-geometric-value-q4-fit ARGS.json"))?;
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
    for root in [&a.parent, &a.no_read_fit, &a.composition_fit, &a.projection] {
        report_output::verify(root)?;
    }
    let parent_bytes = fs::read(a.parent.join("report.json"))?;
    let parent: Value = serde_json::from_slice(&parent_bytes)?;
    let scalar_bytes = fs::read(a.no_read_fit.join("report.json"))?;
    let scalar: Value = serde_json::from_slice(&scalar_bytes)?;
    let fit_bytes = fs::read(a.composition_fit.join("report.json"))?;
    let bank_fit: Value = serde_json::from_slice(&fit_bytes)?;
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
        || bank_fit["complete"] != true
        || bank_fit["mode"] != "fit"
        || bank_fit["seed"] != a.seed
        || bank_fit["fit"]["completed_updates"] != 640
        || bank_fit["parent"]["query_parent_report_sha256"] != sha256_bytes(&parent_bytes)
        || bank_fit["parent"]["no_read_fit_report_sha256"] != sha256_bytes(&scalar_bytes)
    {
        return Err(invalid("accepted query/scalar/composition lineage differs"));
    }
    let initial_rng = bank_fit["fit"]["next_generator_state"]
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
        ("projection", a.projection.as_path()),
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
    if bank_fit["compiled_metadata"] != serde_json::to_value(original_bank.metadata())? {
        return Err(invalid("composition parent metadata differs"));
    }
    original_bank.validate_for(&bank)?;
    let projection_bytes = fs::read(a.projection.join("report.json"))?;
    let projection: Value = serde_json::from_slice(&projection_bytes)?;
    let expected_projection = if a.seed == 1 {
        "70a3f1a16dee81c005d7bd67c073fa6a32fbb6427a5b9df1847a6835b7016351"
    } else {
        "d2d05603501524526ac657bbe332b0c87703cc9799c50472b2c4f45794bf895f"
    };
    if sha256_bytes(&projection_bytes) != expected_projection
        || projection["complete"] != true
        || projection["seed"] != a.seed
        || projection["optimizer_updates"] != 0
        || projection["hard_forward_policy"] != Q4_NATIVE_CHOICE_SURROGATE
        || projection["initial_training_rng"] != initial_rng
        || projection["next_training_rng"] != initial_rng
    {
        return Err(invalid(
            "accepted repaired projection identity/policy/RNG differs",
        ));
    }
    let initial_source = a.projection.join("q4-value-source");
    let initial_native = a.projection.join("q4-value-native");
    let initial_null = a.projection.join("rebound-no-read-native");
    let initial_bank = a.projection.join("rebound-composition-native");
    let mut weights = ValueProducerWeights::load_source(&initial_source)?;
    let source_hashes = fit::parameter_hashes(&weights)?;
    if !weights.is_q4()
        || weights.packed_coefficients()?.len() != 64448
        || projection["packed_sha256"] != sha256_bytes(&weights.packed_coefficients()?)
    {
        return Err(invalid("starting q4 source inventory differs"));
    }
    let qvp = ValueProducerSourcePaths {
        value_source: &initial_source,
        context_source: &cs,
        context_dependencies: a.dependencies(),
    };
    let mut projected = CompiledValueProducer::load(&initial_native, qvp, &tokenizer)?;
    projected.validate_for(
        &weights,
        &ContextWeights::load_source(&cs, a.dependencies(), &tokenizer)?,
    )?;
    projected.validate_native_context(&context)?;
    if projection["compiled_metadata"] != serde_json::to_value(projected.metadata())? {
        return Err(invalid("projection native metadata differs"));
    }
    let qnp = NoReadSourcePaths {
        no_read_source: &ns,
        value: qvp,
        context_native: &cn,
        value_native: &initial_native,
        reducer_native: &rn,
    };
    let mut qnull = CompiledNoRead::load(&initial_null, qnp, &tokenizer)?;
    let qbp = CompositionSourcePaths {
        composition_source: &bs,
        no_read: qnp,
        no_read_native: &initial_null,
    };
    let mut qbank = CompiledComposition::load(&initial_bank, qbp, &tokenizer)?;
    same_numerical_files(&nn, &initial_null)?;
    same_numerical_files(&bn, &initial_bank)?;
    qbank.validate_for(&bank)?;
    let identity = json!({"parent_report_sha256":sha256_bytes(&parent_bytes),
        "no_read_report_sha256":sha256_bytes(&scalar_bytes),"composition_report_sha256":sha256_bytes(&fit_bytes),
        "projection_report_sha256":sha256_bytes(&projection_bytes),"starting_source_parameters":source_hashes,
        "frozen_files":frozen_files,"frozen_base_variables":frozen_base,"frozen_bank_parameters":frozen_bank,
        "data_helper_sha256":sha256_bytes(include_bytes!("attention-geometric-value-learned/data.rs")),
        "initial_training_rng":initial_rng,"initial_absolute_generator_step":fit::PARENT_UPDATES});
    write(&a.out.join("input-identities.json"), &identity)?;
    let preparation_seconds = started.elapsed().as_secs_f64();
    struct Panel {
        name: &'static str,
        episodes: Vec<data::Episode>,
        parent_rows: Vec<Value>,
        projection_rows: Vec<Value>,
        input_hash: String,
        parent_hash: String,
        projection_hash: String,
    }
    let mut panels = Vec::new();
    for name in ["original", "stress"] {
        let inputs = fs::read(a.parent.join(if name == "original" {
            "evaluation.json"
        } else {
            "stress.json"
        }))?;
        let episodes: Vec<data::Episode> = serde_json::from_slice(&inputs)?;
        let parent_rows = fs::read(a.composition_fit.join(format!("{name}-rows.json")))?;
        let projection_rows = fs::read(a.projection.join(format!("{name}-rows.json")))?;
        let p: Vec<Value> = serde_json::from_slice(&parent_rows)?;
        let q: Vec<Value> = serde_json::from_slice(&projection_rows)?;
        if episodes.len() != 128 || p.len() != 128 || q.len() != 128 {
            return Err(invalid("complete retained128-row panels required"));
        }
        fs::write(a.out.join(format!("{name}-episodes.json")), &inputs)?;
        panels.push(Panel {
            name,
            episodes,
            parent_rows: p,
            projection_rows: q,
            input_hash: sha256_bytes(&inputs),
            parent_hash: sha256_bytes(&parent_rows),
            projection_hash: sha256_bytes(&projection_rows),
        });
    }
    let evaluate = |prefit: bool,
                    weights: &ValueProducerWeights,
                    projected: &CompiledValueProducer,
                    qnull: &CompiledNoRead,
                    qbank: &CompiledComposition|
     -> Result<(Vec<Value>, bool)> {
        let mut summaries = Vec::new();
        let mut complete = true;
        for panel in &panels {
            let filename = if prefit {
                format!("prefit-{}-rows.json", panel.name)
            } else {
                format!("{}-rows.json", panel.name)
            };
            let mut rows = Vec::new();
            let mut counts = [0usize; 4];
            let mut means = [0f64; 4];
            let mut gains = [0usize; 2];
            let mut losses = [0usize; 2];
            let mut answer_changes = 0;
            let mut packet_changes = 0;
            let mut max_delta = 0f32;
            for (index, e) in panel.episodes.iter().enumerate() {
                let limit = if prefit {
                    a.maximum_seconds - 300
                } else {
                    a.maximum_seconds
                };
                if started.elapsed().as_secs() >= limit {
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
                let pr = &panel.parent_rows[index];
                let qr = &panel.projection_rows[index];
                if pr["episode"] != serde_json::to_value(e)?
                    || pr["native_prediction"] != pp
                    || pr["answer_logits"]["native"] != serde_json::to_value(&pol)?
                    || qr["episode"] != serde_json::to_value(e)?
                    || qr["index"] != index
                {
                    return Err(invalid("frozen actual parent/episode replay differs"));
                }
                let qp = qr["native_prediction"]
                    .as_u64()
                    .ok_or_else(|| invalid("projection prediction missing"))?
                    as usize;
                let qlogits: Vec<f32> =
                    serde_json::from_value(qr["answer_logits"]["native"].clone())?;
                let (sl, svalues) = model.forward_geometric_q4_values(
                    &e.ids, 1, t, &context, &events, &span, &potential, &reducer, projected, qnull,
                    weights, &bank, false,
                )?;
                let (nl, ntrace, nvalues) = model.forward_geometric_composition_native_with_trace(
                    &e.ids, 1, t, &context, &events, &span, &potential, &reducer, projected, qnull,
                    qbank, false,
                )?;
                check_logits(&sl, t)?;
                check_logits(&nl, t)?;
                check_trace(&svalues.trace, t)?;
                check_trace(&nvalues, t)?;
                check_read(&ntrace, t)?;
                let (sp, sol) = answer(&sl, e.query)?;
                let (np, nol) = answer(&nl, e.query)?;
                same_read_weights(&ptrace, &ntrace)?;
                let query_heads = (0..2)
                    .map(|h| ntrace.rows[h * t + e.query].clone())
                    .collect::<Vec<_>>();
                let query_null = (0..2)
                    .map(|h| ntrace.no_read_q24[h * t + e.query])
                    .collect::<Vec<_>>();
                if prefit
                    && (qr["source_prediction"] != sp
                        || qp != np
                        || qr["answer_logits"]["source"] != serde_json::to_value(&sol)?
                        || qlogits != nol
                        || qr["query_heads"] != serde_json::to_value(&query_heads)?
                        || qr["query_no_read_q24"] != serde_json::to_value(&query_null)?
                        || svalues.trace != nvalues)
                {
                    return Err(invalid("exact projection2 prefit outputs/trace differ"));
                }
                let packet_mismatches = svalues
                    .trace
                    .packets
                    .iter()
                    .zip(&nvalues.packets)
                    .enumerate()
                    .filter(|(_, (s, n))| s != n)
                    .map(|(i, (s, n))| json!({"lane_index_b_h_t_4":i,"source":s,"native":n}))
                    .collect::<Vec<_>>();
                let coordinate_mismatches = svalues
                    .trace
                    .values_q16
                    .iter()
                    .zip(&nvalues.values_q16)
                    .enumerate()
                    .filter(|(_, (s, n))| s != n)
                    .map(|(i, (s, n))| json!({"coordinate_index_b_h_t_16":i,"source":s,"native":n}))
                    .collect::<Vec<_>>();
                let delta = sl
                    .to_vec2::<f32>()?
                    .iter()
                    .zip(nl.to_vec2::<f32>()?)
                    .flat_map(|(s, n)| s.iter().zip(n).map(|(s, n)| (s - n).abs()))
                    .fold(0f32, f32::max);
                max_delta = max_delta.max(delta);
                answer_changes += usize::from(sp != np);
                packet_changes += usize::from(svalues.trace != nvalues);
                let changed_atoms = pvalues
                    .packets
                    .iter()
                    .zip(&nvalues.packets)
                    .map(|(p, n)| p.iter().zip(n).filter(|(a, b)| a != b).count())
                    .sum::<usize>();
                for (i, p) in [pp, qp, sp, np].into_iter().enumerate() {
                    counts[i] += usize::from(p == e.answer as usize);
                }
                for (i, l) in [&pol, &qlogits, &sol, &nol].into_iter().enumerate() {
                    means[i] += ce(l, e.answer as usize)?;
                }
                for (i, p) in [pp, qp].into_iter().enumerate() {
                    gains[i] += usize::from(p != e.answer as usize && np == e.answer as usize);
                    losses[i] += usize::from(p == e.answer as usize && np != e.answer as usize);
                }
                rows.push(json!({"index":index,"episode":e,"parent_prediction":pp,"projection_prediction":qp,
                    "source_prediction":sp,"native_prediction":np,"answer_logits":{"parent":pol,"projection":qlogits,"source":sol,"native":nol},
                    "source_native_answer_match":sp==np,"source_native_packet_trace_equal":svalues.trace==nvalues,
                    "packet_mismatches":packet_mismatches,"coordinate_mismatches":coordinate_mismatches,
                    "changed_primitive_atoms_from_unrestricted":changed_atoms,"maximum_actual_position_logit_delta":delta,
                    "query_heads":query_heads,"query_no_read_q24":query_null,"all_frozen_reader_weights_identical":true}));
                if (index + 1) % 32 == 0 {
                    write(&a.out.join(&filename), &rows)?;
                }
            }
            write(&a.out.join(&filename), &rows)?;
            let d = rows.len().max(1) as f64;
            summaries.push(json!({"panel":panel.name,"rows":rows.len(),"requested_rows":128,
                "answers":{"parent":counts[0],"projection":counts[1],"source":counts[2],"native":counts[3]},
                "mean_answer_ce":{"parent":means[0]/d,"projection":means[1]/d,"source":means[2]/d,"native":means[3]/d},
                "gains_from_parent":gains[0],"losses_from_parent":losses[0],"gains_from_projection":gains[1],"losses_from_projection":losses[1],
                "source_native_answer_changes":answer_changes,"source_native_packet_trace_mismatches":packet_changes,
                "maximum_actual_position_logit_delta":max_delta,"input_sha256":panel.input_hash,"parent_rows_sha256":panel.parent_hash,
                "projection_rows_sha256":panel.projection_hash}));
            if !complete {
                break;
            }
        }
        Ok((summaries, complete))
    };
    let prefit_at = Instant::now();
    let (prefit, prefit_complete) = evaluate(true, &weights, &projected, &qnull, &qbank)?;
    let prefit_seconds = prefit_at.elapsed().as_secs_f64();
    write(&a.out.join("prefit-summary.json"), &prefit)?;
    if !prefit_complete {
        return Err(invalid("deadline during prefit; no updates performed"));
    }
    let progress = fit::run(
        &a.out,
        &weights,
        initial_rng,
        a.mode,
        started + Duration::from_secs(a.maximum_seconds - 300),
        &identity,
        |ids, batch, time, w| {
            model.forward_geometric_q4_values(
                ids, batch, time, &context, &events, &span, &potential, &reducer, &projected,
                &qnull, w, &bank, false,
            )
        },
    )?;
    let compile_at = Instant::now();
    let final_source = a.out.join("q4-value-source");
    let final_native = a.out.join("q4-value-native");
    let final_null = a.out.join("rebound-no-read-native");
    let final_bank = a.out.join("rebound-composition-native");
    let fitted_hashes = fit::parameter_hashes(&weights)?;
    weights.save_source(&final_source)?;
    weights = ValueProducerWeights::load_source(&final_source)?;
    if fit::parameter_hashes(&weights)? != fitted_hashes {
        return Err(invalid("fitted source reload changed bits"));
    }
    let qvp = ValueProducerSourcePaths {
        value_source: &final_source,
        context_source: &cs,
        context_dependencies: a.dependencies(),
    };
    CompiledValueProducer::compile(qvp, &tokenizer)?.save(&final_native)?;
    projected = CompiledValueProducer::load(&final_native, qvp, &tokenizer)?;
    projected.validate_for(
        &weights,
        &ContextWeights::load_source(&cs, a.dependencies(), &tokenizer)?,
    )?;
    projected.validate_native_context(&context)?;
    let qnp = NoReadSourcePaths {
        no_read_source: &ns,
        value: qvp,
        context_native: &cn,
        value_native: &final_native,
        reducer_native: &rn,
    };
    CompiledNoRead::compile(&NoReadWeights::load(&ns)?, qnp, &tokenizer)?.save(&final_null)?;
    qnull = CompiledNoRead::load(&final_null, qnp, &tokenizer)?;
    let null_files = same_numerical_files(&initial_null, &final_null)?;
    let qbp = CompositionSourcePaths {
        composition_source: &bs,
        no_read: qnp,
        no_read_native: &final_null,
    };
    CompiledComposition::compile(&bank, qbp, &tokenizer)?.save(&final_bank)?;
    qbank = CompiledComposition::load(&final_bank, qbp, &tokenizer)?;
    qbank.validate_for(&bank)?;
    let bank_files = same_numerical_files(&initial_bank, &final_bank)?;
    let compilation_seconds = compile_at.elapsed().as_secs_f64();
    let evaluation_at = Instant::now();
    let (summaries, eval_complete) = evaluate(false, &weights, &projected, &qnull, &qbank)?;
    let evaluation_seconds = evaluation_at.elapsed().as_secs_f64();
    if variable_hashes(&model)? != frozen_base
        || parameter_hashes(bank.parameters())? != frozen_bank
        || file_snapshot(&frozen_roots)? != frozen_files
    {
        return Err(invalid("frozen dependency/source changed during fit"));
    }
    let complete = progress.completed && eval_complete;
    write(
        &a.out.join("report.json"),
        &json!({"schema":"uor-r4.geometric-value-q4-answer-fit/1",
        "complete":complete,"mode":a.mode,"seed":a.seed,"fit":progress,"parent":identity,
        "prior_unrestricted_producer_updates":1280,"frozen_no_read_updates":640,"frozen_composition_updates":640,
        "trainable_scalar_count":128896,"trainable_inventory":weights.parameters().iter().map(|(n,v)|(n.clone(),v.dims().to_vec())).collect::<BTreeMap<_,_>>(),
        "objective":"mean actual answer CE; one target/episode; denominator8; no auxiliary",
        "optimizer":{"rate":fit::RATE,"beta1":0.9,"beta2":0.999,"epsilon":1e-8,"weight_decay":0,"selected_global_norm_clip":1},
        "source_policy":weights.config(),"hard_forward_policy":Q4_NATIVE_CHOICE_SURROGATE,
        "packed_sha256":sha256_bytes(&weights.packed_coefficients()?),"compiled_metadata":projected.metadata(),
        "rebound_no_read_numerical_files_unchanged":null_files,"rebound_composition_numerical_files_unchanged":bank_files,
        "prefit":prefit,"panels":summaries,"preparation_seconds":preparation_seconds,"prefit_evaluation_seconds":prefit_seconds,
        "compilation_reload_seconds":compilation_seconds,"evaluation_seconds":evaluation_seconds,"elapsed_seconds":started.elapsed().as_secs_f64(),
        "decision":if !complete{"PARTIAL_RETAIN_NO_QUALITY_VERDICT"}else if a.mode==fit::Mode::Check{"ACTUAL_TWO_B8_Q4_BACKWARDS_ZERO_UPDATES"}else{"FIXED_Q4_VALUE_ANSWER_FIT_RETAIN_ALL_ROWS"},
        "scope":"All ten strict q4 value families train through frozen learned geometric composition. Native current-coefficient hard choices; explicit biased finite-choice backward. No auxiliary, scale selection or perfect recovery condition. Upstream/bank/NoRead/tail frozen; source real-softmax versus native integer reducer remains distinct. Development panels exposed, no general language or full-native/energy qualification. Adam moments not serialized."}),
    )?;
    println!(
        "q4 fit complete={complete}; updates={}; {}",
        progress.completed_updates,
        a.out.display()
    );
    Ok(())
}
fn same_read_weights(
    a: &uor_r4_training::geometric_composition_native::ComposedReadTrace,
    b: &uor_r4_training::geometric_composition_native::ComposedReadTrace,
) -> Result<()> {
    if a.no_read_q24 != b.no_read_q24 || a.rows.len() != b.rows.len() {
        return Err(invalid("frozen null/reader layout differs"));
    }
    for (a, b) in a.rows.iter().zip(&b.rows) {
        if a.occurrence_weights_q31 != b.occurrence_weights_q31
            || a.no_read_weight_q31 != b.no_read_weight_q31
            || a.total_weight_q31 != b.total_weight_q31
            || a.max_score_q24 != b.max_score_q24
        {
            return Err(invalid("value fit changed frozen address/age/null weights"));
        }
    }
    Ok(())
}
