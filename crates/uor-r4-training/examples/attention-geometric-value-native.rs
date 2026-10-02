//! No-training representation comparison of actual Q16 donor values and
//! reconstruction-only K1 signed-H4/dyadic packets. Source weights and NoRead
//! are unchanged. Oracle projection is not a learned native value producer.
use candle_core::Device;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use uor_r4_core::report_output;
use uor_r4_training::geometric_context::{CompiledContext, ContextSourcePaths};
use uor_r4_training::geometric_event::CompiledEvents;
use uor_r4_training::geometric_potential_native::{
    CompiledGeometricPotentials, PotentialSourceBinding,
};
use uor_r4_training::geometric_read_native::{CompiledGeometricRead, ReadSourceBinding};
use uor_r4_training::geometric_span_native::{CompiledSpanActions, SpanSourceBinding};
use uor_r4_training::geometric_stack::StackModel;
use uor_r4_training::{sha256_bytes, sha256_file, Result, TrainingError};

const EVALUATION_SHA256: &str = "dabdaa1a2a8dcf23e1cfe5164ef00b97b923c3b74dc09dcd1d4a307d1ff64915";
const STRESS_SHA256: &str = "e3c81984b7b41e0fcb329fde026318d067b6397e93832508c3ee60180a5a9d63";
const CHUNK: usize = 8;
fn invalid(message: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(message.into())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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
struct Args {
    model: PathBuf,
    context_source: PathBuf,
    context_native: PathBuf,
    event_source: PathBuf,
    event_native: PathBuf,
    span_native: PathBuf,
    potential_native: PathBuf,
    evaluation: PathBuf,
    stress: PathBuf,
    out: PathBuf,
    max_seconds: u64,
}
impl Args {
    fn parse() -> Result<Self> {
        let mut options = BTreeMap::new();
        for argument in std::env::args().skip(1) {
            let (key, value) = argument
                .split_once('=')
                .ok_or_else(|| invalid("use key=value arguments"))?;
            if value.is_empty() || options.insert(key.to_string(), value.to_string()).is_some() {
                return Err(invalid("empty or repeated argument"));
            }
        }
        let max_seconds = options
            .remove("max_seconds")
            .unwrap_or_else(|| "900".into())
            .parse::<u64>()
            .map_err(|_| invalid("invalid max_seconds"))?;
        if max_seconds == 0 || max_seconds > 900 {
            return Err(invalid("max_seconds must be 1..900"));
        }
        let mut path = |key: &str| {
            options
                .remove(key)
                .map(PathBuf::from)
                .ok_or_else(|| invalid(format!("missing {key}")))
        };
        let result = Self {
            model: path("model")?,
            context_source: path("context_source")?,
            context_native: path("context_native")?,
            event_source: path("event_source")?,
            event_native: path("event_native")?,
            span_native: path("span_native")?,
            potential_native: path("potential_native")?,
            evaluation: path("evaluation")?,
            stress: path("stress")?,
            out: path("out")?,
            max_seconds,
        };
        if !options.is_empty() {
            return Err(invalid("unknown argument"));
        }
        Ok(result)
    }
}
fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    fs::write(path, serde_json::to_vec(value)?)?;
    Ok(())
}
fn panel(path: &Path, expected: &str, out: &Path) -> Result<Vec<Episode>> {
    let bytes = fs::read(path)?;
    if sha256_bytes(&bytes) != expected {
        return Err(invalid("pinned development panel identity differs"));
    }
    let episodes: Vec<Episode> = serde_json::from_slice(&bytes)?;
    if episodes.len() != 128
        || episodes.iter().any(|e| {
            e.ids.is_empty()
                || e.ids.len() > 128
                || e.ids.len() != e.roles.len()
                || e.query + 1 != e.ids.len()
                || e.source >= e.query
                || e.ids.get(e.source) != Some(&e.answer)
                || e.ids[e.query] != 34
                || e.ids.iter().any(|&id| id >= 40)
        })
    {
        return Err(invalid(
            "saved panel occurrence/query/value contract differs",
        ));
    }
    fs::write(out, bytes)?;
    Ok(episodes)
}
fn batch(episodes: &[Episode]) -> Result<(Vec<u32>, usize)> {
    let time = episodes
        .iter()
        .map(|e| e.ids.len())
        .max()
        .ok_or_else(|| invalid("empty chunk"))?;
    let count = episodes
        .len()
        .checked_mul(time)
        .ok_or_else(|| invalid("chunk shape overflow"))?;
    let mut ids = vec![38; count];
    for (b, e) in episodes.iter().enumerate() {
        ids[b * time..b * time + e.ids.len()].copy_from_slice(&e.ids);
    }
    Ok((ids, time))
}
fn argmax(values: &[f32]) -> Result<u32> {
    if values.is_empty() || values.iter().any(|x| !x.is_finite()) {
        return Err(invalid("nonempty finite logits required"));
    }
    let mut best = 0;
    for i in 1..values.len() {
        if values[i] > values[best] {
            best = i;
        }
    }
    Ok(best as u32)
}
fn validate_logits(rows: &[Vec<f32>], positions: usize, vocabulary: usize) -> Result<()> {
    if rows.len() != positions
        || rows
            .iter()
            .any(|r| r.len() != vocabulary || r.iter().any(|x| !x.is_finite()))
    {
        return Err(invalid("logit shape or finite-value contract differs"));
    }
    Ok(())
}
fn finite_mass(values: &[f32], count: usize) -> Result<()> {
    if values.len() != count
        || values
            .iter()
            .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
    {
        return Err(invalid("source mass shape/range differs"));
    }
    Ok(())
}
fn equal_bits(a: &[Vec<f32>], b: &[Vec<f32>]) -> bool {
    a.len() == b.len()
        && a.iter().zip(b).all(|(a, b)| {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| a.to_bits() == b.to_bits())
        })
}
fn input_files(directory: &Path) -> Result<BTreeMap<String, Value>> {
    let mut files = BTreeMap::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            return Err(invalid("artifact directory contains nonregular file"));
        }
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| invalid("artifact filename is not UTF8"))?;
        files.insert(
            name,
            json!({"bytes":entry.metadata()?.len(),"sha256":sha256_file(&entry.path())?}),
        );
    }
    Ok(files)
}

#[derive(Serialize)]
struct Row {
    index: usize,
    episode: Episode,
    baseline_prediction: u32,
    projected_prediction: u32,
    baseline_answer_logits: Vec<f32>,
    projected_answer_logits: Vec<f32>,
    source_masses: [f32; 2],
    no_read_masses: [f32; 2],
    maximum_answer_logit_delta: f64,
    maximum_actual_position_logit_delta: f64,
    maximum_read_error_q16: u64,
}

struct Loaded {
    value_codec: uor_r4_training::geometric_value_native::CompiledGeometricValues,
    model: StackModel,
    context: CompiledContext,
    events: CompiledEvents,
    span: CompiledSpanActions,
    potential: CompiledGeometricPotentials,
    reducer: CompiledGeometricRead,
}
fn score(loaded: &Loaded, episodes: &[Episode], out: &Path, deadline: Instant) -> Result<Value> {
    fs::create_dir(out)?;
    let mut rows = Vec::new();
    let mut restored = 0usize;
    let mut checked_coordinates = 0usize;
    let mut maximum_read_error_q16 = 0u64;
    let mut maximum_projection_error_q16 = 0u64;
    let mut partial_chunk = None;
    let m = &loaded.model;
    for (chunk, group) in episodes.chunks(CHUNK).enumerate() {
        if Instant::now() >= deadline {
            break;
        }
        let (ids, time) = batch(group)?;
        let (baseline, baseline_trace) = m.forward_geometric_context_read_native_with_trace(
            &ids,
            group.len(),
            time,
            &loaded.context,
            &loaded.events,
            &loaded.span,
            &loaded.potential,
            &loaded.reducer,
            false,
        )?;
        let baseline = baseline.to_vec2::<f32>()?;
        validate_logits(&baseline, ids.len(), m.config.vocab_size)?;
        write_json(
            &out.join(format!("chunk-{chunk:03}-baseline.json")),
            &json!({"batch":group.len(),"time":time,"ids":ids,"logits":baseline,"trace":baseline_trace}),
        )?;
        if Instant::now() >= deadline {
            partial_chunk = Some(chunk);
            break;
        }
        let (projected, trace, projection) = m
            .forward_geometric_context_value_projection_native_with_trace(
                &ids,
                group.len(),
                time,
                &loaded.context,
                &loaded.events,
                &loaded.span,
                &loaded.potential,
                &loaded.reducer,
                &loaded.value_codec,
                false,
            )?;
        let projected = projected.to_vec2::<f32>()?;
        validate_logits(&projected, ids.len(), m.config.vocab_size)?;
        if trace.values_q16 != projection.projected_q16
            || baseline_trace.values_q16 != projection.donor_q16
            || baseline_trace.rows.len() != trace.rows.len()
        {
            return Err(invalid("actual projection/reducer payload trace mismatch"));
        }
        for (a, b) in baseline_trace.rows.iter().zip(&trace.rows) {
            if a.occurrence_weights_q31 != b.occurrence_weights_q31
                || a.no_read_weight_q31 != b.no_read_weight_q31
                || a.total_weight_q31 != b.total_weight_q31
                || a.max_score_q24 != b.max_score_q24
            {
                return Err(invalid(
                    "value projection changed source/NoRead/age weights",
                ));
            }
        }
        write_json(
            &out.join(format!("chunk-{chunk:03}-projected.json")),
            &json!({"batch":group.len(),"time":time,"logits":projected,"trace":trace,"projection":projection}),
        )?;
        if Instant::now() >= deadline {
            partial_chunk = Some(chunk);
            break;
        }
        let (after, _) = m.forward_geometric_context_read_native_with_trace(
            &ids,
            group.len(),
            time,
            &loaded.context,
            &loaded.events,
            &loaded.span,
            &loaded.potential,
            &loaded.reducer,
            false,
        )?;
        let after = after.to_vec2::<f32>()?;
        validate_logits(&after, ids.len(), m.config.vocab_size)?;
        if !equal_bits(&baseline, &after) {
            return Err(invalid(
                "native reference changed after oracle value projection",
            ));
        }
        restored += 1;
        for (b, e) in group.iter().enumerate() {
            let mut episode_read_error = 0u64;
            let mut logit_delta = 0f64;
            for q in 0..e.ids.len() {
                for (a, z) in baseline[b * time + q].iter().zip(&projected[b * time + q]) {
                    logit_delta = logit_delta.max((f64::from(*a) - f64::from(*z)).abs());
                }
                for head in 0..trace.heads {
                    let first = (b * trace.heads + head) * time;
                    let row = &trace.rows[first + q];
                    let old = &baseline_trace.rows[first + q];
                    for coordinate in 0..trace.value_width {
                        let mut error_numerator = 0u128;
                        for (j, &weight) in row.occurrence_weights_q31.iter().enumerate() {
                            let at = (first + j) * trace.value_width + coordinate;
                            let error = (i64::from(projection.projected_q16[at])
                                - i64::from(projection.donor_q16[at]))
                            .unsigned_abs();
                            maximum_projection_error_q16 = maximum_projection_error_q16.max(error);
                            error_numerator += u128::from(weight) * u128::from(error);
                        }
                        let error = (i64::from(row.output_q16[coordinate])
                            - i64::from(old.output_q16[coordinate]))
                        .unsigned_abs();
                        // Two nearest-rounded read outputs contribute at most
                        // one Q16 unit beyond the exact weighted input error.
                        if u128::from(error) * u128::from(row.total_weight_q31)
                            > error_numerator + u128::from(row.total_weight_q31)
                        {
                            return Err(invalid(
                                "actual projected read exceeds unchanged-weight error bound",
                            ));
                        }
                        checked_coordinates += 1;
                        episode_read_error = episode_read_error.max(error);
                    }
                }
            }
            maximum_read_error_q16 = maximum_read_error_q16.max(episode_read_error);
            let at = b * time + e.query;
            let answer_delta = baseline[at]
                .iter()
                .zip(&projected[at])
                .map(|(a, z)| (f64::from(*a) - f64::from(*z)).abs())
                .fold(0f64, f64::max);
            let source_masses = [
                trace.source_mass(b, 0, e.query, &[e.source])?,
                trace.source_mass(b, 1, e.query, &[e.source])?,
            ];
            let no_read_masses = [
                trace.no_read_mass(b, 0, e.query)?,
                trace.no_read_mass(b, 1, e.query)?,
            ];
            finite_mass(&source_masses, 2)?;
            finite_mass(&no_read_masses, 2)?;
            rows.push(Row {
                index: chunk * CHUNK + b,
                episode: e.clone(),
                baseline_prediction: argmax(&baseline[at])?,
                projected_prediction: argmax(&projected[at])?,
                baseline_answer_logits: baseline[at].clone(),
                projected_answer_logits: projected[at].clone(),
                source_masses,
                no_read_masses,
                maximum_answer_logit_delta: answer_delta,
                maximum_actual_position_logit_delta: logit_delta,
                maximum_read_error_q16: episode_read_error,
            });
        }
        write_json(&out.join("rows.json"), &rows)?;
    }
    let summary = json!({"complete":rows.len()==episodes.len(),"requested_rows":episodes.len(),"scored_rows":rows.len(),"reference_restored_chunks":restored,"partial_chunk":partial_chunk,
        "baseline_answers":rows.iter().filter(|r|r.baseline_prediction==r.episode.answer).count(),
        "projected_answers":rows.iter().filter(|r|r.projected_prediction==r.episode.answer).count(),
        "changed_predictions":rows.iter().filter(|r|r.baseline_prediction!=r.projected_prediction).count(),
        "maximum_answer_logit_delta":rows.iter().map(|r|r.maximum_answer_logit_delta).fold(0.,f64::max),
        "maximum_actual_position_logit_delta":rows.iter().map(|r|r.maximum_actual_position_logit_delta).fold(0.,f64::max),
        "maximum_read_error_q16":maximum_read_error_q16,"maximum_projected_coordinate_error_q16":maximum_projection_error_q16,
        "unchanged_weight_bound_checked_coordinates":checked_coordinates,"unchanged_weight_bound_violations":0,"rows":rows});
    write_json(&out.join("summary.json"), &summary)?;
    Ok(summary)
}
fn run(args: &Args, start: Instant) -> Result<Value> {
    let deadline = start + Duration::from_secs(args.max_seconds);
    let evaluation = panel(
        &args.evaluation,
        EVALUATION_SHA256,
        &args.out.join("evaluation.json"),
    )?;
    let stress = panel(&args.stress, STRESS_SHA256, &args.out.join("stress.json"))?;
    let mut inputs = BTreeMap::new();
    for (name, path) in [
        ("model", &args.model),
        ("context_source", &args.context_source),
        ("context_native", &args.context_native),
        ("event_source", &args.event_source),
        ("event_native", &args.event_native),
        ("span_native", &args.span_native),
        ("potential_native", &args.potential_native),
    ] {
        inputs.insert(name, json!({"path":path,"files":input_files(path)?}));
    }
    let exe = std::env::current_exe()?;
    write_json(
        &args.out.join("inputs.json"),
        &json!({"artifacts":inputs,"evaluation_sha256":EVALUATION_SHA256,"stress_sha256":STRESS_SHA256,"executable":exe,"executable_sha256":sha256_file(&exe)?,"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNAVAILABLE"),"maximum_seconds":args.max_seconds}),
    )?;
    if Instant::now() >= deadline {
        return Ok(
            json!({"complete":false,"decision":"PARTIAL_BEFORE_MODEL_LOAD","elapsed_seconds":start.elapsed().as_secs_f64()}),
        );
    }
    let model = StackModel::load(&args.model, &Device::Cpu)?;
    if model.config.width != 32
        || model.config.heads != 2
        || model.config.vocab_size != 40
        || model.config.context != 128
    {
        return Err(invalid(
            "retained width32/heads2/vocab40/context128 required",
        ));
    }
    let tokenizer = fs::read(args.span_native.join("tokenizer-identity.bin"))?;
    let span_source = SpanSourceBinding::from_files(
        &args.model.join("model.safetensors"),
        &args.model.join("config.json"),
        &tokenizer,
    )?;
    let span = CompiledSpanActions::load(
        &args.span_native,
        &span_source,
        model
            .geometric_span()
            .ok_or_else(|| invalid("saved span configuration absent"))?,
    )?;
    let potential = CompiledGeometricPotentials::load(
        &args.potential_native,
        &PotentialSourceBinding::from_directory(&args.model, &tokenizer)?,
    )?;
    let events = CompiledEvents::load(
        &args.event_native,
        &args.event_source,
        &args.model,
        &tokenizer,
    )?;
    let paths = ContextSourcePaths {
        base: &args.model,
        event_source: &args.event_source,
        event_native: &args.event_native,
        span_native: &args.span_native,
        potential_native: &args.potential_native,
    };
    let context = CompiledContext::load(
        &args.context_native,
        &args.context_source,
        paths,
        &tokenizer,
    )?;
    let source = ReadSourceBinding::from_directory(&args.model, &tokenizer, &potential, 2)?;
    let age = model
        .variables()
        .get("layers.02.read.age")
        .ok_or_else(|| invalid("saved age absent"))?
        .as_tensor();
    let reducer = CompiledGeometricRead::compile(age, &potential, &source)?;
    reducer.save(&args.out.join("native-read"))?;
    let reducer = CompiledGeometricRead::load(&args.out.join("native-read"), &source)?;
    let value_source = uor_r4_training::geometric_value_native::ValueSourceBinding::from_directory(
        &args.model,
        &tokenizer,
        &potential,
        &reducer,
        2,
    )?;
    let value_codec =
        uor_r4_training::geometric_value_native::CompiledGeometricValues::compile(&value_source)?;
    value_codec.save(&args.out.join("native-values"))?;
    let value_codec = uor_r4_training::geometric_value_native::CompiledGeometricValues::load(
        &args.out.join("native-values"),
        &value_source,
    )?;
    let metadata = value_codec.metadata().clone();
    let loaded = Loaded {
        value_codec,
        model,
        context,
        events,
        span,
        potential,
        reducer,
    };
    let original = score(&loaded, &evaluation, &args.out.join("original"), deadline)?;
    let longer = score(&loaded, &stress, &args.out.join("stress"), deadline)?;
    let complete = original["complete"] == true && longer["complete"] == true;
    let changes = original["changed_predictions"].as_u64().unwrap_or(0)
        + longer["changed_predictions"].as_u64().unwrap_or(0);
    Ok(
        json!({"schema":"uor-r4.geometric-value-native-projection/1","complete":complete,"decision":if !complete{"PARTIAL_BUDGET_NO_QUALITY_VERDICT"}else if changes!=0{"K1_REPRESENTATION_CHANGED_RETAIN_ROWS"}else{"K1_ORACLE_REPRESENTATION_RETENTION_NOT_LEARNED_PRODUCER"},"value_codec":metadata,"original":original,"stress":longer,"elapsed_seconds":start.elapsed().as_secs_f64(),"training_steps":0,"maximum_seconds":args.max_seconds,"scope":"same nativecontext/event/span/potential/scalarNoRead/age/occurrenceweights; actualdonorQ16values versus reconstruction-only nearest K1 signedH4/dyadicradius packets; oracleprojection receives donorvalues, NOT learnednativeproducer; fullcausalvalues and original/stress authoreddevelopment panels; no source/answer labels in projection; downstreamread.out/trunk/head remainfloat; no natural-language, geometryadvantage or fullserving claim"}),
    )
}
fn main() -> Result<()> {
    let args = Args::parse()?;
    report_output::claim(&args.out)?;
    let start = Instant::now();
    let result = run(&args, start);
    match &result {
        Ok(report) => write_json(&args.out.join("report.json"), report)?,
        Err(error) => write_json(
            &args.out.join("error.json"),
            &json!({"error":error.to_string(),"elapsed_seconds":start.elapsed().as_secs_f64()}),
        )?,
    }
    report_output::seal(&args.out)?;
    report_output::verify(&args.out)?;
    result?;
    Ok(())
}
