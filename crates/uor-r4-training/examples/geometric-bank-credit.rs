//! Native bank full-answer hard-forward marginal CE and offline gradient admission.
//! Zero updates; source-only runtime packets and separately frozen typed answers.
use candle_core::{Tensor, Var};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
    time::Instant,
};
use uor_r4_core::{
    answer_oracle::{FrozenAnswers, RecordedValueIntent},
    report_output,
};
use uor_r4_integer::{
    geometric_occurrence_read::{FrameMetadata, FrameStatus, SelectedRecordFrame, SourceIdentity},
    geometric_source_realizer::{
        NativeArtifactBinding, NativeSourceRealizer as IntegerRealizer, SourceBankSegment,
    },
};
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::{
    geometric_occurrence_consumer::{
        source_realizer::{NativeSourceRealizer, SourceRealizerWeights},
        ConsumerIdentity,
    },
    sha256_bytes, sha256_file,
};
#[path = "../../uor-r4-integer/examples/support/source_probe.rs"]
mod output_support;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const REPORT_CAP: usize = 64 * 1024 * 1024;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Args {
    source_weights: PathBuf,
    trusted_native_binding: PathBuf,
    native_artifact: PathBuf,
    bank_inputs: PathBuf,
    bank_labels: PathBuf,
    out: PathBuf,
    maximum_seconds: u64,
    maximum_context_tokens: usize,
    maximum_generation_tokens: usize,
    maximum_report_bytes: usize,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Inputs {
    schema: String,
    cases: Vec<Packet>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Packet {
    id: String,
    segments: Vec<Segment>,
    query_ids: Vec<u32>,
    actual_prefix_ids: Vec<u32>,
}
#[derive(Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum Segment {
    Source {
        event: u64,
        record: u64,
        commit: u64,
        scope: String,
        entity: Vec<u32>,
        relation: u32,
        view: u32,
        original_source_ids: Vec<u32>,
    },
    Context {
        event: u64,
        role: u32,
        token_ids: Vec<u32>,
    },
}
impl Segment {
    fn frame(&self) -> Option<SelectedRecordFrame<'_>> {
        match self {
            Self::Source {
                record,
                commit,
                scope,
                entity,
                relation,
                view,
                original_source_ids,
                ..
            } => Some(SelectedRecordFrame {
                identity: SourceIdentity {
                    record: *record,
                    commit: *commit,
                },
                metadata: FrameMetadata {
                    scope: scope.as_bytes(),
                    entity,
                    relation: *relation,
                    view: *view,
                    status: FrameStatus::Found,
                },
                token_ids: original_source_ids,
            }),
            Self::Context { .. } => None,
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Labels {
    schema: String,
    protocol: String,
    membership_only: bool,
    cases: Vec<Label>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Label {
    id: String,
    answers: FrozenAnswers,
}
fn invalid(s: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, s.into())
}
fn read_json(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
fn write_json(root: &Path, name: &str, value: &Value) -> Result<()> {
    let bytes = serde_json::to_vec(value)?;
    let destination = root.join(name);
    let existing = fs::metadata(&destination).map_or(0, |m| m.len() as usize);
    let current = fs::read_dir(root)?.try_fold(0usize, |sum, entry| -> io::Result<usize> {
        let metadata = entry?.metadata()?;
        Ok(sum.saturating_add(metadata.len() as usize))
    })?;
    if current.saturating_sub(existing).saturating_add(bytes.len()) > REPORT_CAP - 1024 * 1024 {
        return Err(invalid("report byte cap reached").into());
    }
    fs::write(destination, bytes)?;
    Ok(())
}
fn parameter_receipts(parameters: &BTreeMap<String, Var>) -> Result<Value> {
    let mut receipts = BTreeMap::new();
    for (name, var) in parameters {
        let values = var.flatten_all()?.to_vec1::<f32>()?;
        let bytes = values
            .iter()
            .flat_map(|v| v.to_bits().to_le_bytes())
            .collect::<Vec<_>>();
        receipts.insert(name, json!({"shape":var.dims(),"elements":values.len(),"f32_le_bits_sha256":sha256_bytes(&bytes)}));
    }
    Ok(serde_json::to_value(receipts)?)
}
fn peak_rss_kib() -> Option<u64> {
    fs::read_to_string("/proc/self/status")
        .ok()?
        .lines()
        .find_map(|s| {
            s.strip_prefix("VmHWM:")?
                .split_whitespace()
                .next()?
                .parse()
                .ok()
        })
}
fn executable() -> Result<(PathBuf, &'static str)> {
    match std::env::current_exe() {
        Ok(path) => Ok((path, "current_exe")),
        Err(error) => {
            let path = std::env::args().next().map(PathBuf::from).ok_or(error)?;
            if !path.is_absolute() {
                return Err(invalid("current_exe unavailable; argv0 must be absolute").into());
            }
            Ok((path, "absolute_argv0_fallback"))
        }
    }
}

fn capped(path: &Path) -> Result<Vec<u8>> {
    use std::io::Read;
    let f = fs::File::open(path)?;
    if f.metadata()?.len() > 4 * 1024 * 1024 {
        return Err(invalid("input exceeds4MiB").into());
    }
    let mut b = Vec::new();
    f.take(4 * 1024 * 1024 + 1).read_to_end(&mut b)?;
    if b.len() > 4 * 1024 * 1024 {
        return Err(invalid("input grew beyond4MiB").into());
    }
    Ok(b)
}
fn checked_args() -> Result<Args> {
    let mut argv = std::env::args().skip(1);
    let file = argv
        .next()
        .ok_or_else(|| invalid("one config path required"))?;
    if argv.next().is_some() {
        return Err(invalid("only one config path").into());
    }
    let a: Args = serde_json::from_slice(&capped(Path::new(&file))?)?;
    if a.maximum_seconds == 0
        || a.maximum_seconds > 300
        || a.maximum_context_tokens != 128
        || a.maximum_generation_tokens > 32
        || a.maximum_report_bytes != REPORT_CAP
    {
        return Err(invalid("declared bank admission bounds differ").into());
    }
    let output = output_support::prospective_output(&a.out)?;
    for p in [
        &a.source_weights,
        &a.trusted_native_binding,
        &a.native_artifact,
        &a.bank_inputs,
        &a.bank_labels,
    ] {
        let path = fs::canonicalize(p)?;
        if output.starts_with(&path) {
            return Err(invalid("output beneath input").into());
        }
        for ancestor in path.ancestors() {
            if ancestor.join(report_output::MANIFEST_FILE).is_file() && output.starts_with(ancestor)
            {
                return Err(invalid("output beneath sealed input ancestor").into());
            }
        }
    }
    Ok(a)
}
fn check_limit(a: &Args, start: Instant) -> Result<()> {
    if start.elapsed().as_secs() >= a.maximum_seconds
        || peak_rss_kib().is_some_and(|v| v > 8 * 1024 * 1024)
    {
        return Err(invalid("wall/RSS admission cap reached").into());
    }
    Ok(())
}
fn nearest_seal(path: &Path) -> Result<PathBuf> {
    for ancestor in path.ancestors() {
        if ancestor.join(report_output::MANIFEST_FILE).is_file() {
            return Ok(ancestor.to_path_buf());
        }
    }
    Err(invalid("required source/native/panel sealed ancestor absent").into())
}
fn validate_labels(inputs: &Inputs, labels: &Labels) -> Result<()> {
    if inputs.schema != "uor-r4.native-source-bank-probe-input/1"
        || labels.schema != "uor-r4.native-source-bank-labels/1"
        || labels.protocol != "uor-r4.literal-role-dialogue/2"
        || !labels.membership_only
        || inputs.cases.is_empty()
        || inputs.cases.len() > 64
        || inputs.cases.len() != labels.cases.len()
    {
        return Err(invalid("bank input/label contract differs").into());
    }
    let mut ids = BTreeSet::new();
    for (p, l) in inputs.cases.iter().zip(&labels.cases) {
        l.answers.validate()?;
        if p.id.is_empty()
            || !ids.insert(&p.id)
            || p.id != l.id
            || l.answers.intent != RecordedValueIntent::Current
            || l.answers.accepted.len() != 1
            || p.segments.is_empty()
            || p.segments.len() > 128
            || p.query_ids.is_empty()
            || !p.actual_prefix_ids.is_empty()
        {
            return Err(invalid("case binding differs; nonempty actual prefix unsupported for full-answer admission").into());
        }
    }
    Ok(())
}
fn gradients(
    loss: &Tensor,
    parameters: &BTreeMap<String, Var>,
    sum: &mut BTreeMap<String, Vec<f64>>,
    weight: f64,
) -> Result<Value> {
    let g = loss.backward()?;
    let mut receipt = BTreeMap::new();
    for (name, var) in parameters {
        if let Some(gradient) = g.get(var) {
            let values = gradient.flatten_all()?.to_vec1::<f32>()?;
            if values.iter().any(|v| !v.is_finite()) {
                return Err(invalid(format!("nonfinite gradient {name}")).into());
            }
            let aggregate = sum
                .entry(name.clone())
                .or_insert_with(|| vec![0.; values.len()]);
            if aggregate.len() != values.len() {
                return Err(invalid("gradient shape changed").into());
            }
            for (v, x) in aggregate.iter_mut().zip(&values) {
                *v += f64::from(*x) * weight;
            }
            receipt.insert(name.clone(),json!({"connected":true,"finite":true,"elements":values.len(),"nonzero":values.iter().filter(|v|**v!=0.).count(),"l1":values.iter().map(|v|f64::from(v.abs())).sum::<f64>()}));
        } else {
            receipt.insert(name.clone(),json!({"connected":false,"finite":"NOT_RUN disconnected","elements":var.elem_count(),"nonzero":0}));
        }
    }
    Ok(serde_json::to_value(receipt)?)
}
fn run(a: &Args, start: Instant) -> Result<Value> {
    let mut hashes = BTreeMap::new();
    let mut sealed_roots = BTreeSet::new();
    for p in [
        &a.source_weights,
        &a.native_artifact,
        &a.bank_inputs,
        &a.bank_labels,
    ] {
        let sealed = nearest_seal(p)?;
        if sealed_roots.insert(sealed.clone()) {
            report_output::verify(&sealed)?;
        }
        let manifest = sealed.join(report_output::MANIFEST_FILE);
        hashes.insert(
            manifest.to_string_lossy().into_owned(),
            sha256_file(&manifest)?,
        );
    }
    for p in [
        &a.trusted_native_binding,
        &a.bank_inputs,
        &a.bank_labels,
        &a.native_artifact.join("metadata.json"),
        &a.native_artifact.join("tokenizer.json"),
    ] {
        hashes.insert(p.to_string_lossy().into_owned(), sha256_file(p)?);
    }
    let input: Inputs = serde_json::from_slice(&capped(&a.bank_inputs)?)?;
    let labels: Labels = serde_json::from_slice(&capped(&a.bank_labels)?)?;
    validate_labels(&input, &labels)?;
    let expected: NativeArtifactBinding =
        serde_json::from_slice(&capped(&a.trusted_native_binding)?)?;
    let integer = IntegerRealizer::load_native(&a.native_artifact, &expected)?;
    let bytes = fs::read(a.native_artifact.join("tokenizer.json"))?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes)
        .ok_or_else(|| invalid("invalid bound ByteBPE"))?;
    let source = SourceRealizerWeights::load_source(&a.source_weights, &bytes)?;
    let metadata = read_json(&a.native_artifact.join("metadata.json"))?;
    let identity: ConsumerIdentity = serde_json::from_value(metadata["identity"].clone())?;
    let native = NativeSourceRealizer::load(&a.native_artifact, &source, &identity)?;
    if native.artifact_binding()? != expected {
        return Err(invalid("source/native/trusted identity differs").into());
    }
    let prepared = source.prepare(&native)?;
    let params = source.parameters();
    let frozen = parameter_receipts(&params)?;
    let mut rows = Vec::new();
    let mut full_sum = BTreeMap::new();
    let mut first_sum = BTreeMap::new();
    let mut any_zero = false;
    let mut any_first_zero = false;
    let mut total_positions = 0;
    let mut replay_positions = 0;
    for (index, (packet, label)) in input.cases.iter().zip(&labels.cases).enumerate() {
        check_limit(a, start)?;
        let mut target = tokenizer.encode(&format!(" {}", label.answers.accepted[0]));
        target.push(integer.binding().eos_token_id());
        if target.is_empty() || target.len() > 32 {
            return Err(invalid("full answer/EOS exceeds32").into());
        }
        let decoded = String::from_utf8(tokenizer.decode_bytes(&target[..target.len() - 1]))?;
        if decoded != format!(" {}", label.answers.accepted[0]) {
            return Err(invalid("answer byte/token roundtrip differs").into());
        }
        let mut views = Vec::new();
        for segment in &packet.segments {
            views.push(match segment.frame() {
                Some(frame) => Some(integer.compile_view(frame.token_ids)?),
                None => None,
            });
        }
        let mut segments = Vec::new();
        for (i, segment) in packet.segments.iter().enumerate() {
            segments.push(match segment {
                Segment::Source { event, .. } => SourceBankSegment::Source {
                    frame: segment.frame().ok_or_else(|| invalid("frame absent"))?,
                    view: views[i].as_ref().ok_or_else(|| invalid("view absent"))?,
                    event: *event,
                },
                Segment::Context {
                    token_ids,
                    event,
                    role,
                } => SourceBankSegment::Context {
                    token_ids,
                    event: *event,
                    role: *role,
                },
            });
        }
        let mut losses = Vec::new();
        let mut tokens = Vec::new();
        let mut zero = false;
        let mut ce_total = 0.;
        for (step, &target_label) in target.iter().enumerate() {
            check_limit(a, start)?;
            let prefix = &target[..step];
            let hard = integer.read_bank(&segments, &packet.query_ids, prefix)?;
            let equivalent = native.read_bank(&segments, &packet.query_ids, prefix)?;
            if hard != equivalent {
                return Err(invalid("training/independent native bank trace differs").into());
            }
            let mass = hard
                .actions
                .token_masses
                .iter()
                .filter(|m| m.token_id == target_label)
                .map(|m| m.weight_q31)
                .sum::<u64>();
            let total = hard.actions.total_weight_q31;
            if total == 0 {
                return Err(invalid("native normalizer zero").into());
            }
            let probability = mass as f64 / total as f64;
            let ce = if mass > 0 {
                Some(-probability.ln())
            } else {
                None
            };
            if mass == 0 {
                zero = true;
                any_zero = true;
                if step == 0 {
                    any_first_zero = true;
                }
            } else {
                let measured =
                    prepared.loss_bank(&segments, &packet.query_ids, prefix, target_label)?;
                if measured.trace != hard || measured.target_probability != probability {
                    return Err(invalid("bank hard-forward/loss native parity differs").into());
                }
                let offline = measured.loss.to_scalar::<f32>()?;
                if !offline.is_finite() || (f64::from(offline) + probability.ln()).abs() > 1e-5 {
                    return Err(invalid("offline loss not native marginal CE").into());
                }
                losses.push(measured.loss);
                ce_total += ce.ok_or_else(|| invalid("finite CE absent"))?;
            }
            let query_start = hard.context.tokens.len() - prefix.len() - packet.query_ids.len();
            let source_positions = hard
                .candidates
                .iter()
                .map(|c| c.context_position)
                .collect::<BTreeSet<_>>();
            let role_positions = json!({"source":source_positions,"noncandidate_history":(0..query_start).filter(|p|!source_positions.contains(p)).collect::<Vec<_>>(),"query":(query_start..query_start+packet.query_ids.len()).collect::<Vec<_>>(),"teacherforced_prefix":(query_start+packet.query_ids.len()..hard.context.tokens.len()).collect::<Vec<_>>(),"gradient_attribution":"shared context Vars; positions are causal provenance, not separate role gradients"});
            tokens.push(json!({"step":step,"teacherforced_prefix_ids_labels_only":prefix,"target_label_only_after_native_read":target_label,"target_mass_q31":mass,"total_weight_q31":total,"native_ce":ce,"zero_support":mass==0,"causal_position_roles":role_positions,"trace":hard}));
            total_positions += 1;
        }
        let first_gradient = if tokens[0]["zero_support"] == false {
            Some(gradients(
                &losses[0],
                &params,
                &mut first_sum,
                1. / input.cases.len() as f64,
            )?)
        } else {
            None
        };
        let full_gradient = if !zero {
            let mean = Tensor::stack(&losses, 0)?.mean_all()?;
            Some(gradients(
                &mean,
                &params,
                &mut full_sum,
                1. / input.cases.len() as f64,
            )?)
        } else {
            None
        };
        let mut generated = Vec::new();
        let mut ownprefix_traces = Vec::new();
        let mut eos = false;
        for step in 0..a.maximum_generation_tokens {
            check_limit(a, start)?;
            let hard = integer.read_bank(&segments, &packet.query_ids, &generated)?;
            let equivalent = native.read_bank(&segments, &packet.query_ids, &generated)?;
            if hard != equivalent {
                return Err(invalid("ownprefix native parity differs").into());
            }
            let token = hard.actions.chosen_token_id;
            ownprefix_traces.push(json!({"step":step,"actual_prefix_ids":generated,"trace":hard}));
            generated.push(token);
            replay_positions += 1;
            if token == integer.binding().eos_token_id() {
                eos = true;
                break;
            }
        }
        let non_eos = if eos {
            &generated[..generated.len() - 1]
        } else {
            &generated[..]
        };
        let raw = String::from_utf8(tokenizer.decode_bytes(non_eos))?;
        let reply = raw.strip_prefix(' ').unwrap_or(&raw);
        let row = json!({"id":packet.id,"native_mean_answer_ce":if zero {None}else{Some(ce_total/target.len() as f64)},"zero_support":zero,"first_token_gradient":first_gradient,"full_answer_gradient":full_gradient,"gradient_status":if zero {"full-answer NOT_RUN: infinite native CE, no support floor or positive-only surrogate"}else{"ordinary full-answer episode-mean backward"},"canonical_tokens":tokens,"generated_ids_including_eos":generated,"terminated_with_eos":eos,"reply_text":reply,"accepted_complete_answer":eos && label.answers.accepts(reply),"ownprefix_traces":ownprefix_traces});
        let filename = format!("row-{index:04}.json");
        write_json(&a.out, &filename, &row)?;
        rows.push(json!({"id":packet.id,"path":filename,"sha256":sha256_file(&a.out.join(&filename))?,"zero_support":zero}));
        if parameter_receipts(&params)? != frozen {
            return Err(invalid("zero-update source/context/readouts changed").into());
        }
    }
    let aggregate = |sum: &BTreeMap<String, Vec<f64>>| -> Value {
        let mut out = BTreeMap::new();
        for (n, g) in sum {
            out.insert(n,json!({"elements":g.len(),"finite":g.iter().all(|x|x.is_finite()),"nonzero":g.iter().filter(|x|**x!=0.).count(),"l1":g.iter().map(|x|x.abs()).sum::<f64>(),"f64_le_bits_sha256":sha256_bytes(&g.iter().flat_map(|x|x.to_bits().to_le_bytes()).collect::<Vec<_>>())}));
        }
        json!(out)
    };
    for (path, hash) in &hashes {
        if sha256_file(Path::new(path))? != *hash {
            return Err(invalid("input changed").into());
        }
    }
    for sealed in sealed_roots {
        report_output::verify(&sealed)?;
    }
    let (exe, lookup) = executable()?;
    Ok(
        json!({"schema":"uor-r4.geometric-bank-credit/1","status":"completed","cases":input.cases.len(),"target_positions":total_positions,"native_ownprefix_replays":replay_positions,"optimizer_updates":0,"rows":rows,"first_token_equalepisode_gradient":if any_first_zero {None}else{Some(aggregate(&first_sum))},"aggregate_episode_denominator":input.cases.len(),"first_token_complete_batch_available":!any_first_zero,"full_answer_complete_batch_available":!any_zero,"first_token_gradient_status":if any_first_zero {"NOT_RUN complete first-token objective; zero support row"}else{"ordinary first-token CE; episode equal weights"},"full_answer_equalepisode_gradient":if any_zero {None}else{Some(aggregate(&full_sum))},"full_batch_gradient_status":if any_zero {"NOT_RUN infinite native objective; finite rows are disclosed separately"}else{"finite full-answer ordinary marginal CE; episode equal weights"},"source_parameters_before_after_equal":true,"source_parameters":frozen,"trusted_native_binding":expected,"input_files_sha256":hashes,"elapsed_seconds":start.elapsed().as_secs_f64(),"peak_rss_kib_linux":peak_rss_kib(),"host_arch":std::env::consts::ARCH,"host_os":std::env::consts::OS,"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"executable_sha256":sha256_file(&exe)?,"executable_lookup":lookup,"scope":"development bank native gradient admission only; shared context coefficients include source/query/cues/prefix, not independent source/query parameters; biased context/quantized STE, no optimizer descent guarantee or trained chat"}),
    )
}
fn main() {
    let result = (|| -> Result<()> {
        let a = checked_args()?;
        report_output::claim(&a.out)?;
        let start = Instant::now();
        let measured = run(&a, start);
        match &measured {
            Ok(v) => write_json(&a.out, "report.json", v)?,
            Err(e) => write_json(
                &a.out,
                "failure.json",
                &json!({"status":"failed","error":e.to_string(),"optimizer_updates":0,"elapsed_seconds":start.elapsed().as_secs_f64()}),
            )?,
        }
        report_output::seal(&a.out)?;
        report_output::verify(&a.out)?;
        measured.map(|_| ())
    })();
    if let Err(e) = result {
        eprintln!("bank credit: {e}");
        std::process::exit(1);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn runtime_packet_rejects_target_fields_and_labels_bind_unique_case() -> Result<()> {
        let raw = r#"{"schema":"uor-r4.native-source-bank-probe-input/1","cases":[{"id":"x","segments":[],"query_ids":[1],"actual_prefix_ids":[],"target_ids":[2]}]}"#;
        assert!(serde_json::from_str::<Inputs>(raw).is_err());
        let input = Inputs {
            schema: "uor-r4.native-source-bank-probe-input/1".into(),
            cases: vec![Packet {
                id: "x".into(),
                segments: vec![Segment::Context {
                    event: 3,
                    role: 0,
                    token_ids: vec![1],
                }],
                query_ids: vec![1],
                actual_prefix_ids: vec![],
            }],
        };
        let mut labels = Labels {
            schema: "uor-r4.native-source-bank-labels/1".into(),
            protocol: "uor-r4.literal-role-dialogue/2".into(),
            membership_only: true,
            cases: vec![Label {
                id: "x".into(),
                answers: FrozenAnswers {
                    intent: RecordedValueIntent::Current,
                    accepted: vec!["singer.".into()],
                },
            }],
        };
        validate_labels(&input, &labels)?;
        labels.cases[0].id = "y".into();
        assert!(validate_labels(&input, &labels).is_err());
        Ok(())
    }
}
