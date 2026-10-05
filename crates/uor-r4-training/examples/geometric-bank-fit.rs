//! Native bank full-answer hard-forward marginal CE and offline gradient admission.
//! Zero updates; source-only runtime packets and separately frozen typed answers.
use candle_core::{Device, Tensor, Var};
use candle_nn::{AdamW, Optimizer, ParamsAdamW};
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
    geometric_cue_carrier::{CueAngularConfig, CueAngularQ4, CueScoreMode, NativeCueCarrier},
    geometric_occurrence_read::{FrameMetadata, FrameStatus, SelectedRecordFrame, SourceIdentity},
    geometric_source_realizer::{
        NativeArtifactBinding, NativeSourceRealizer as IntegerRealizer, SourceBankSegment,
    },
};
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::{
    geometric_occurrence_consumer::{
        source_realizer::{
            CueAngularWeights, NativeSourceRealizer, PrefixAngularWeights, SourceRealizerWeights,
        },
        ConsumerIdentity,
    },
    sha256_bytes, sha256_file,
};
#[path = "../../uor-r4-integer/examples/support/source_probe.rs"]
mod output_support;
use uor_r4_integer::geometric_prefix_transport::{
    NativePrefixTransport, PrefixAngularConfig, PrefixAngularQ4, PrefixScoreMode,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const REPORT_CAP: usize = 512 * 1024 * 1024;
const FACTOR_REPORT_CAP: usize = 256 * 1024 * 1024;
const READOUT_FIT_CAP: usize = 300 * 1024 * 1024;
const READOUT_BROAD_CAP: usize = 64 * 1024 * 1024;
const READOUT_FAMILIES: &str = "active-contextual-Copy4+Stop+Period/1";
fn readout_mode(a: &Args) -> bool {
    a.mode.starts_with("readout-")
}
fn broad_mode(a: &Args) -> bool {
    matches!(a.mode.as_str(), "broadbatch" | "readout-broadbatch")
}
const CUE_FIT_CAP: usize = 128 * 1024 * 1024;
const CUE_FAMILIES: &str = "cue-angular-HxLx120/1";
fn cue_mode(a: &Args) -> bool {
    a.mode.starts_with("cue-")
}
const PREFIX_FIT_CAP: usize = 512 * 1024 * 1024;
const PREFIX_FAMILIES: &str = "prefix-angular-HxLx120/1";
fn prefix_mode(a: &Args) -> bool {
    a.mode.starts_with("prefix-")
}
const UPDATES: usize = 64;
const BATCH: usize = 8;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Args {
    mode: String,
    cue_score_mode: Option<CueScoreMode>,
    prefix_score_mode: Option<PrefixScoreMode>,
    frozen_cue_bundle: Option<PathBuf>,
    frozen_cue_native_metadata_sha256: Option<String>,
    frozen_cue_packed_sha256: Option<String>,
    learned_source_weights: Option<PathBuf>,
    learned_native_artifact: Option<PathBuf>,
    learned_trusted_native_binding: Option<PathBuf>,
    source_weights: PathBuf,
    native_artifact: PathBuf,
    trusted_native_binding: PathBuf,
    development_panel: PathBuf,
    development_manifest_sha256: String,
    fresh_panel: PathBuf,
    fresh_manifest_sha256: String,
    admission: Option<PathBuf>,
    admission_manifest_sha256: Option<String>,
    fit_authorization: Option<PathBuf>,
    exposed_controls: Option<PathBuf>,
    out: PathBuf,
    maximum_seconds: u64,
    maximum_context_tokens: usize,
    maximum_generation_tokens: usize,
    maximum_report_bytes: usize,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Authorization {
    schema: String,
    fit_admitted: bool,
    admission_report_sha256: String,
    development_manifest_sha256: String,
    fresh_manifest_sha256: String,
    trusted_binding_sha256: String,
    updates: usize,
    batch_episodes: usize,
    maximum_fit_seconds: u64,
    #[serde(default)]
    context_frozen: bool,
    #[serde(default)]
    active_families: Option<String>,
}
#[derive(Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct Inputs {
    schema: String,
    cases: Vec<Packet>,
}
#[derive(Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct Packet {
    id: String,
    segments: Vec<Segment>,
    query_ids: Vec<u32>,
    actual_prefix_ids: Vec<u32>,
}
#[derive(Deserialize, serde::Serialize)]
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
#[derive(Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct Labels {
    schema: String,
    protocol: String,
    membership_only: bool,
    cases: Vec<Label>,
}
#[derive(Deserialize, serde::Serialize)]
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

fn directory_bytes(path: &Path) -> Result<usize> {
    let mut n = 0usize;
    for e in fs::read_dir(path)? {
        let e = e?;
        let m = e.file_type()?;
        if m.is_symlink() {
            return Err(invalid("symlink in owned artifact/report").into());
        }
        n = n
            .checked_add(if m.is_dir() {
                directory_bytes(&e.path())?
            } else {
                e.metadata()?.len() as usize
            })
            .ok_or_else(|| invalid("report size overflow"))?;
    }
    Ok(n)
}
fn write_json(root: &Path, name: &str, v: &Value) -> Result<()> {
    write_json_limited(root, name, v, REPORT_CAP)
}
fn write_json_limited(root: &Path, name: &str, v: &Value, cap: usize) -> Result<()> {
    let b = serde_json::to_vec(v)?;
    let p = root.join(name);
    let previous = fs::metadata(&p).map_or(0, |m| m.len() as usize);
    let mut total_root = root;
    while let Some(parent) = total_root.parent() {
        if parent.join("attempt.json").is_file() {
            total_root = parent;
        } else {
            break;
        }
    }
    let cap = if total_root.join("resource-cap.json").is_file() {
        cap.min(
            read_json(&total_root.join("resource-cap.json"))?["maximum_report_bytes"]
                .as_u64()
                .ok_or_else(|| invalid("report cap receipt absent"))? as usize,
        )
    } else {
        cap
    };
    if directory_bytes(total_root)?
        .saturating_sub(previous)
        .saturating_add(b.len())
        > cap - 1024 * 1024
    {
        return Err(invalid("report cap reached").into());
    }
    fs::write(p, b)?;
    Ok(())
}
fn read_capped(path: &Path) -> Result<Vec<u8>> {
    use std::io::Read;
    let f = fs::File::open(path)?;
    if f.metadata()?.len() > 16 * 1024 * 1024 {
        return Err(invalid("input cap16MiB").into());
    }
    let mut b = Vec::new();
    f.take(16 * 1024 * 1024 + 1).read_to_end(&mut b)?;
    if b.len() > 16 * 1024 * 1024 {
        return Err(invalid("input grew beyond16MiB").into());
    }
    Ok(b)
}
fn nearest_seal(p: &Path) -> Result<PathBuf> {
    for a in p.ancestors() {
        if a.join(report_output::MANIFEST_FILE).is_file() {
            return Ok(a.to_path_buf());
        }
    }
    Err(invalid("required sealed source ancestor absent").into())
}
fn deadline(a: &Args, start: Instant) -> Result<()> {
    if start.elapsed().as_secs() >= a.maximum_seconds
        || peak_rss_kib().is_some_and(|n| n > 8 * 1024 * 1024)
    {
        return Err(invalid("declared model wall/RSS cap reached").into());
    }
    Ok(())
}
fn checked_args() -> Result<Args> {
    let mut av = std::env::args().skip(1);
    let config = av
        .next()
        .ok_or_else(|| invalid("one config path required"))?;
    if av.next().is_some() {
        return Err(invalid("only one config path").into());
    }
    let a: Args = serde_json::from_slice(&read_capped(Path::new(&config))?)?;
    if ![
        "broadbatch",
        "fit",
        "factor-probe",
        "readout-broadbatch",
        "readout-fit",
        "cue-broadbatch",
        "cue-fit",
        "prefix-broadbatch",
        "prefix-fit",
    ]
    .contains(&a.mode.as_str())
        || a.maximum_seconds == 0
        || a.maximum_seconds
            > if a.mode == "fit" {
                3600
            } else if matches!(a.mode.as_str(), "readout-fit" | "cue-fit" | "prefix-fit") {
                1200
            } else {
                300
            }
        || a.maximum_context_tokens != 128
        || a.maximum_generation_tokens > 32
        || a.maximum_report_bytes
            != if a.mode == "factor-probe" {
                FACTOR_REPORT_CAP
            } else if a.mode == "prefix-fit" {
                PREFIX_FIT_CAP
            } else if a.mode == "prefix-broadbatch" {
                READOUT_BROAD_CAP
            } else if a.mode == "cue-fit" {
                CUE_FIT_CAP
            } else if a.mode == "cue-broadbatch" {
                READOUT_BROAD_CAP
            } else if a.mode == "readout-fit" {
                READOUT_FIT_CAP
            } else if a.mode == "readout-broadbatch" {
                READOUT_BROAD_CAP
            } else {
                REPORT_CAP
            }
    {
        return Err(invalid("mode/resource contract differs").into());
    }
    if matches!(
        a.mode.as_str(),
        "fit" | "readout-fit" | "cue-fit" | "prefix-fit"
    ) && (a.admission.is_none()
        || a.admission_manifest_sha256.is_none()
        || a.fit_authorization.is_none())
    {
        return Err(invalid(
            "fit requires separate sealed broadbatch and explicit resource authorization",
        )
        .into());
    }
    if (broad_mode(&a) || a.mode == "cue-broadbatch" || a.mode == "prefix-broadbatch")
        && (a.admission.is_some() || a.fit_authorization.is_some())
    {
        return Err(invalid("broadbatch cannot automatically fit").into());
    }
    if cue_mode(&a) != a.cue_score_mode.is_some() {
        return Err(invalid("cue_score_mode required only for explicit cue modes").into());
    }
    let prefix_inputs = [
        a.prefix_score_mode.is_some(),
        a.frozen_cue_bundle.is_some(),
        a.frozen_cue_native_metadata_sha256.is_some(),
        a.frozen_cue_packed_sha256.is_some(),
    ];
    if (prefix_mode(&a) && !prefix_inputs.iter().all(|v| *v))
        || (!prefix_mode(&a) && prefix_inputs.iter().any(|v| *v))
    {
        return Err(invalid(
            "prefix mode requires complete explicit frozen cue native binding and mode",
        )
        .into());
    }
    let donors = [
        a.learned_source_weights.is_some(),
        a.learned_native_artifact.is_some(),
        a.learned_trusted_native_binding.is_some(),
    ];
    if (a.mode == "factor-probe"
        && (!donors.iter().all(|v| *v)
            || a.admission.is_some()
            || a.admission_manifest_sha256.is_some()
            || a.fit_authorization.is_some()
            || a.maximum_generation_tokens != 32))
        || (a.mode != "factor-probe" && donors.iter().any(|v| *v))
    {
        return Err(invalid(
            "factor mode requires only complete learned donor paths, no optimization admission",
        )
        .into());
    }
    let out = output_support::prospective_output(&a.out)?;
    let mut paths = vec![
        &a.source_weights,
        &a.native_artifact,
        &a.trusted_native_binding,
        &a.development_panel,
        &a.fresh_panel,
    ];
    paths.extend(a.learned_source_weights.iter());
    paths.extend(a.learned_native_artifact.iter());
    paths.extend(a.learned_trusted_native_binding.iter());
    paths.extend(a.frozen_cue_bundle.iter());
    paths.extend(a.admission.iter());
    paths.extend(a.fit_authorization.iter());
    paths.extend(a.exposed_controls.iter());
    for p in paths {
        let p = fs::canonicalize(p)?;
        if out.starts_with(&p) {
            return Err(invalid("output beneath input").into());
        }
        for ancestor in p.ancestors() {
            if ancestor.join(report_output::MANIFEST_FILE).is_file() && out.starts_with(ancestor) {
                return Err(invalid("output beneath sealed input").into());
            }
        }
    }
    Ok(a)
}
struct Episode {
    packet: Packet,
    answers: FrozenAnswers,
    target: Vec<u32>,
    views: Vec<Option<uor_r4_integer::geometric_source_emission_view::SourceEmissionView>>,
}
impl Episode {
    fn segments(&self) -> Result<Vec<SourceBankSegment<'_>>> {
        self.packet
            .segments
            .iter()
            .enumerate()
            .map(|(i, s)| {
                Ok(match s {
                    Segment::Source { event, .. } => SourceBankSegment::Source {
                        frame: s.frame().ok_or_else(|| invalid("frame absent"))?,
                        view: self.views[i]
                            .as_ref()
                            .ok_or_else(|| invalid("view absent"))?,
                        event: *event,
                    },
                    Segment::Context {
                        event,
                        role,
                        token_ids,
                    } => SourceBankSegment::Context {
                        token_ids,
                        event: *event,
                        role: *role,
                    },
                })
            })
            .collect()
    }
}
fn validate_answers(answers: &FrozenAnswers, single_source: bool) -> Result<()> {
    answers.validate()?;
    if answers.intent != RecordedValueIntent::Current
        || answers.accepted.is_empty()
        || (!single_source && answers.accepted.len() != 1)
    {
        return Err(invalid("accepted answer policy differs").into());
    }
    Ok(())
}
fn load_panel(
    root: &Path,
    expected_count: usize,
    native: &IntegerRealizer,
    tok: &ByteBpeTokenizer,
    receipt: bool,
) -> Result<Vec<Episode>> {
    report_output::verify(root)?;
    let inputs: Inputs = serde_json::from_slice(&read_capped(&root.join("inputs.json"))?)?;
    let labels: Labels = serde_json::from_slice(&read_capped(&root.join("labels.json"))?)?;
    if inputs.schema != "uor-r4.native-source-bank-probe-input/1"
        || labels.schema != "uor-r4.native-source-bank-labels/1"
        || labels.protocol != "uor-r4.literal-role-dialogue/2"
        || !labels.membership_only
        || inputs.cases.len() != expected_count
        || labels.cases.len() != expected_count
    {
        return Err(invalid("panel schema/count differs").into());
    }
    let context = if receipt {
        Some(read_json(&root.join("context-data.json"))?)
    } else {
        None
    };
    let mut ids = BTreeSet::new();
    let mut result = Vec::new();
    for (i, (packet, label)) in inputs.cases.into_iter().zip(labels.cases).enumerate() {
        let single_source = context
            .as_ref()
            .map(|c| c["cases"][i]["kind"] == "single-source")
            .unwrap_or(true);
        validate_answers(&label.answers, single_source)?;
        if packet.id != label.id
            || packet.id.is_empty()
            || !ids.insert(packet.id.clone())
            || packet.segments.is_empty()
            || packet.query_ids.is_empty()
            || !packet.actual_prefix_ids.is_empty()
        {
            return Err(
                invalid("panel case/label binding differs; nonempty prefix unsupported").into(),
            );
        }
        let mut target = tok.encode(&format!(" {}", label.answers.accepted[0]));
        target.push(native.binding().eos_token_id());
        for accepted in &label.answers.accepted {
            let text = format!(" {accepted}");
            if String::from_utf8(tok.decode_bytes(&tok.encode(&text)))? != text {
                return Err(invalid("accepted alternative byte-BPE roundtrip differs").into());
            }
        }
        if target.len() > 32
            || String::from_utf8(tok.decode_bytes(&target[..target.len() - 1]))?
                != format!(" {}", label.answers.accepted[0])
        {
            return Err(invalid("answer/EOS encoding differs or exceeds32").into());
        }
        let mut views = Vec::new();
        for s in &packet.segments {
            views.push(match s.frame() {
                Some(f) => Some(native.compile_view(f.token_ids)?),
                None => None,
            });
        }
        if let Some(c) = &context {
            let expected_views = views
                .iter()
                .enumerate()
                .filter_map(|(j, v)| {
                    v.as_ref()
                        .map(|view| json!({"segment_index":j,"source_view":view}))
                })
                .collect::<Vec<_>>();
            if c["cases"][i]["source_views"] != json!(expected_views) {
                return Err(invalid("frozen context sourceview projection differs").into());
            }
            if c["schema"] != "uor-r4.geometric-bank-context-data/1"
                || c["cases"][i]["id"] != packet.id
                || c["cases"][i]["target_ids_labels_only"] != json!(target)
            {
                return Err(invalid("separate context receipt/target binding differs").into());
            }
        }
        result.push(Episode {
            packet,
            answers: label.answers,
            target,
            views,
        });
    }
    if let Some(c) = context {
        let rows = c["cases"]
            .as_array()
            .ok_or_else(|| invalid("context cases absent"))?;
        if rows.len() != expected_count {
            return Err(invalid("context count differs").into());
        }
        let mut pairs = BTreeMap::<String, Vec<usize>>::new();
        for (i, r) in rows.iter().enumerate() {
            if let Some(pair) = r["pair_id"].as_str() {
                pairs.entry(pair.into()).or_default().push(i);
            }
        }
        if (expected_count == 128
            && (pairs.len() != 32 || rows[..64].iter().any(|r| r["kind"] != "single-source")))
            || (expected_count == 32 && pairs.len() != 16)
        {
            return Err(invalid("declared preservation/pair counts differ").into());
        }
        for indices in pairs.values() {
            if indices.len() != 2
                || !matches!(
                    (
                        rows[indices[0]]["query_role"].as_str(),
                        rows[indices[1]]["query_role"].as_str()
                    ),
                    (Some("job"), Some("where")) | (Some("where"), Some("job"))
                )
                || (expected_count == 128
                    && (indices[0] < 64
                        || indices[1] != indices[0] + 1
                        || (indices[0] - 64) % 2 != 0))
                || serde_json::to_value(&result[indices[0]].packet.segments)?
                    != serde_json::to_value(&result[indices[1]].packet.segments)?
            {
                return Err(invalid("same-bank different-query pair differs").into());
            }
        }
    }
    Ok(result)
}
fn active(name: &str) -> bool {
    name.starts_with("consumer.context.")
        || name.starts_with("consumer.no_read.")
        || name.starts_with("period.")
        || matches!(
            name,
            "consumer.potential.context_unary"
                | "consumer.potential.context_radius"
                | "consumer.potential.context_presence"
                | "consumer.potential.content_presence"
        )
}
fn readout_active(name: &str) -> bool {
    active(name) && !name.starts_with("consumer.context.")
}
fn active_for(name: &str, a: &Args) -> bool {
    if readout_mode(a) {
        readout_active(name)
    } else {
        active(name)
    }
}
fn inactive_bits(source: &SourceRealizerWeights, a: &Args) -> Result<Value> {
    parameter_receipts(
        &source
            .parameters()
            .into_iter()
            .filter(|(n, _)| !active_for(n, a))
            .collect(),
    )
}
fn scale(episodes: usize, tokens: usize) -> Result<f64> {
    if episodes == 0 || tokens == 0 {
        return Err(invalid("empty objective").into());
    }
    Ok(1. / episodes as f64 / tokens as f64)
}
struct Batch {
    gradients: BTreeMap<String, Tensor>,
    report: Value,
}
fn batch(
    indices: &[usize],
    episodes: &[Episode],
    source: &SourceRealizerWeights,
    native: &NativeSourceRealizer,
    a: &Args,
    start: Instant,
    independent: Option<&IntegerRealizer>,
) -> Result<Batch> {
    let begun = Instant::now();
    let prepared = source.prepare(native)?;
    let params = source.parameters();
    let mut gradients = BTreeMap::<String, Tensor>::new();
    let mut rows = Vec::new();
    let mut mean = 0.;
    let mut positions = 0;
    for &i in indices {
        let e = episodes.get(i).ok_or_else(|| invalid("batch index"))?;
        let segments = e.segments()?;
        let mut ce = 0.;
        let mut first = None;
        for (step, &target) in e.target.iter().enumerate() {
            deadline(a, start)?;
            let out = if readout_mode(a) {
                prepared.loss_bank_readout(
                    &segments,
                    &e.packet.query_ids,
                    &e.target[..step],
                    target,
                )?
            } else {
                prepared.loss_bank(&segments, &e.packet.query_ids, &e.target[..step], target)?
            };
            if let Some(integer) = independent {
                let expected =
                    integer.read_bank(&segments, &e.packet.query_ids, &e.target[..step])?;
                if expected != out.trace {
                    return Err(invalid(
                        "broadbatch complete native trace parity differs before backward",
                    )
                    .into());
                }
            }
            let mass = out
                .trace
                .actions
                .token_masses
                .iter()
                .filter(|v| v.token_id == target)
                .map(|v| v.weight_q31)
                .sum::<u64>();
            let total = out.trace.actions.total_weight_q31;
            if mass == 0 || total == 0 {
                return Err(invalid(format!("infinite native objective id={} step={step}; no floor or positive-only gradient",e.packet.id)).into());
            }
            let expected = -(mass as f64 / total as f64).ln();
            let scalar = out.loss.to_scalar::<f32>()?;
            if !scalar.is_finite()
                || out.target_probability != mass as f64 / total as f64
                || (f64::from(scalar) - expected).abs() > 1e-4 + 1e-5 * expected.abs()
            {
                return Err(invalid("ordinary loss differs actual native marginal").into());
            }
            ce += expected;
            positions += 1;
            if step == 0 {
                first = Some(expected);
            }
            let loss = (&out.loss * scale(indices.len(), e.target.len())?)?;
            let store = loss.backward()?;
            if readout_mode(a)
                && params.iter().any(|(name, var)| {
                    name.starts_with("consumer.context.") && store.get(var.as_tensor()).is_some()
                })
            {
                return Err(
                    invalid("frozen readout loss unexpectedly has context gradient").into(),
                );
            }
            for (name, var) in &params {
                if active_for(name, a) {
                    if let Some(g) = store.get(var.as_tensor()) {
                        let g = g.detach();
                        let combined = match gradients.remove(name) {
                            Some(old) => (&old + &g)?.detach(),
                            None => g,
                        };
                        gradients.insert(name.clone(), combined);
                    }
                }
            }
        }
        mean += ce / e.target.len() as f64 / indices.len() as f64;
        rows.push(json!({"id":e.packet.id,"target_steps":e.target.len(),"native_mean_token_ce":ce/e.target.len() as f64,"first_token_ce":first}));
    }
    let mut stats = BTreeMap::new();
    let mut square = 0.;
    for (name, g) in &gradients {
        let v = g.flatten_all()?.to_vec1::<f32>()?;
        if v.iter().any(|x| !x.is_finite()) {
            return Err(invalid("gradient nonfinite").into());
        }
        square += v.iter().map(|x| f64::from(*x).powi(2)).sum::<f64>();
        stats.insert(name.clone(),json!({"elements":v.len(),"finite":true,"nonzero":v.iter().filter(|x|**x!=0.).count(),"l1":v.iter().map(|x|f64::from(x.abs())).sum::<f64>()}));
    }
    Ok(Batch {
        gradients,
        report: json!({"episodes":indices.len(),"episode_indices":indices,"target_positions":positions,"native_equal_episode_ce":mean,"gradient_global_norm":square.sqrt(),"gradient_families":stats,"rows":rows,"elapsed_seconds":begun.elapsed().as_secs_f64(),"independent_native_trace_parity":independent.is_some(),"context_gradient_graph_absent":readout_mode(a),"active_families":if readout_mode(a){Some(READOUT_FAMILIES)}else{None},"preparation_cost_scope":"source/native validation and prepared context packing remain; readout mode omits per-prefix context adjoint", "objective":"mean episodes(mean completeanswer+EOS marginal CE); pertoken backward detached F32 gradient accumulation; globalclip aftermean; no supportfloor"}),
    })
}
fn apply(
    source: &SourceRealizerWeights,
    opt: &mut AdamW,
    g: BTreeMap<String, Tensor>,
) -> Result<f64> {
    let params = source.parameters();
    let mut sq = 0.;
    for v in g.values() {
        for x in v.flatten_all()?.to_vec1::<f32>()? {
            sq += f64::from(x).powi(2);
        }
    }
    let norm = sq.sqrt();
    if !norm.is_finite() {
        return Err(invalid("nonfinite clipnorm").into());
    }
    let clip = if norm > 1. { 1. / norm } else { 1. };
    let mut store = Tensor::new(0f32, &Device::Cpu)?.backward()?;
    for (n, g) in g {
        let v = params
            .get(&n)
            .ok_or_else(|| invalid("gradient Var absent"))?;
        store.insert(v.as_tensor(), (&g * clip)?.detach());
    }
    opt.step(&store)?;
    source.project_quarter_range()?;
    Ok(clip)
}
fn compact(trace: &uor_r4_integer::geometric_source_realizer::BankRealizerTrace) -> Result<Value> {
    let end = trace.context.states.len() - 1;
    let lanes = trace.context.heads * trace.context.lanes_per_head;
    Ok(
        json!({"actions":trace.actions,"bank_binding_sha256":trace.bank_binding_sha256,"context_sha256":sha256_bytes(&serde_json::to_vec(&trace.context)?),"final_latent_states":trace.context.states[end],"final_observed_codes":&trace.context.codes[end*lanes..(end+1)*lanes],"candidate_mapping":trace.candidates}),
    )
}
fn canonical(
    native: &IntegerRealizer,
    episodes: &[Episode],
    a: &Args,
    start: Instant,
) -> Result<Value> {
    let mut rows = Vec::new();
    let mut total = 0.;
    let mut zeros = Vec::new();
    let mut count = 0;
    for e in episodes {
        let segments = e.segments()?;
        let mut tokens = Vec::new();
        let mut rowce = 0.;
        let mut rowzero = false;
        for (step, &label) in e.target.iter().enumerate() {
            deadline(a, start)?;
            let trace = native.read_bank(&segments, &e.packet.query_ids, &e.target[..step])?;
            let mass = trace
                .actions
                .token_masses
                .iter()
                .filter(|t| t.token_id == label)
                .map(|t| t.weight_q31)
                .sum::<u64>();
            let normalizer = trace.actions.total_weight_q31;
            if normalizer == 0 {
                return Err(invalid("zero normalizer").into());
            }
            let ce = if mass == 0 {
                rowzero = true;
                zeros.push(json!({"id":e.packet.id,"step":step}));
                None
            } else {
                let ce = -(mass as f64 / normalizer as f64).ln();
                rowce += ce;
                Some(ce)
            };
            tokens.push(json!({"step":step,"teacherforced_prefix_ids_labels_only":&e.target[..step],"target_label_only_after_read":label,"native_ce":ce,"target_mass_q31":mass,"total_weight_q31":normalizer,"native":compact(&trace)?}));
            count += 1;
        }
        let mean = if rowzero {
            None
        } else {
            Some(rowce / e.target.len() as f64)
        };
        if let Some(m) = mean {
            total += m / episodes.len() as f64;
        }
        rows.push(json!({"id":e.packet.id,"native_mean_token_ce":mean,"tokens":tokens}));
    }
    Ok(
        json!({"cases":episodes.len(),"target_positions":count,"native_equal_episode_ce":if zeros.is_empty(){Some(total)}else{None},"zero_support_positions":zeros,"rows":rows,"probability_floor":false,"infinite_native_objective_when_zero":true}),
    )
}
fn generation(
    native: &IntegerRealizer,
    episodes: &[Episode],
    tok: &ByteBpeTokenizer,
    a: &Args,
    start: Instant,
) -> Result<Value> {
    let mut rows = Vec::new();
    let mut complete = 0;
    for e in episodes {
        let segments = e.segments()?;
        let mut ids = Vec::new();
        let mut traces = Vec::new();
        let mut eos = false;
        for step in 0..a.maximum_generation_tokens {
            deadline(a, start)?;
            let t = native.read_bank(&segments, &e.packet.query_ids, &ids)?;
            let next = t.actions.chosen_token_id;
            traces.push(json!({"step":step,"actual_prefix_ids":ids,"native":compact(&t)?}));
            ids.push(next);
            if next == native.binding().eos_token_id() {
                eos = true;
                break;
            }
        }
        let bytes = tok.decode_bytes(if eos { &ids[..ids.len() - 1] } else { &ids });
        let raw = String::from_utf8_lossy(&bytes);
        let text = raw.strip_prefix(' ').unwrap_or(&raw);
        let accepted = eos && String::from_utf8(bytes.clone()).is_ok() && e.answers.accepts(text);
        complete += usize::from(accepted);
        rows.push(json!({"id":e.packet.id,"generated_ids_including_eos":ids,"eos":eos,"reply_text":text,"raw_decoded_bytes_hex":hex::encode(bytes),"accepted_complete_answer":accepted,"tokens":traces}));
    }
    Ok(
        json!({"cases":episodes.len(),"accepted_complete":complete,"maximum_generated_tokens":a.maximum_generation_tokens,"canonical_prefixes_used":false,"rows":rows}),
    )
}
fn compare(old: &Value, new: &Value) -> Result<Value> {
    let left = old["rows"]
        .as_array()
        .ok_or_else(|| invalid("old rows absent"))?;
    let right = new["rows"]
        .as_array()
        .ok_or_else(|| invalid("new rows absent"))?;
    if left.len() != right.len() {
        return Err(invalid("row comparison count").into());
    }
    let mut rows = Vec::new();
    for (a, b) in left.iter().zip(right) {
        if a["id"] != b["id"] {
            return Err(invalid("comparison identity").into());
        }
        rows.push(json!({"id":a["id"],"previous_native_ce":a["native_mean_token_ce"],"current_native_ce":b["native_mean_token_ce"],"tokens_changed":a["tokens"].as_array().zip(b["tokens"].as_array()).map(|(a,b)|a.iter().zip(b).filter(|(a,b)|a["native"]!=b["native"]).count())}));
    }
    Ok(json!({"rows":rows}))
}
fn checkpoint(
    source: &SourceRealizerWeights,
    identity: &ConsumerIdentity,
    tok_bytes: &[u8],
    episodes: &[Episode],
    step: usize,
    prior: &[Value],
    frozen: &Value,
    a: &Args,
    start: Instant,
) -> Result<Value> {
    let required = directory_bytes(&a.source_weights)?
        .checked_add(directory_bytes(&a.native_artifact)?)
        .and_then(|n| n.checked_add(32 * 1024 * 1024))
        .ok_or_else(|| invalid("checkpoint projection overflow"))?;
    if directory_bytes(&a.out)?.saturating_add(required) > a.maximum_report_bytes - 1024 * 1024 {
        return Err(invalid("checkpoint source/native+32MiB report reserve exceeds cap").into());
    }
    let root = a.out.join(format!("checkpoint-{step:04}"));
    report_output::claim(&root)?;
    let result = (|| -> Result<Value> {
        if inactive_bits(source, a)? != *frozen {
            return Err(invalid("frozen source bits changed before export").into());
        }
        source.save_source(&root.join("realizer-source"))?;
        let native = source.compile(identity.clone())?;
        native.save(&root.join("realizer-native"))?;
        let restored =
            SourceRealizerWeights::load_source(&root.join("realizer-source"), tok_bytes)?;
        let loaded =
            NativeSourceRealizer::load(&root.join("realizer-native"), &restored, identity)?;
        if parameter_receipts(&source.parameters())? != parameter_receipts(&restored.parameters())?
        {
            return Err(invalid("checkpoint source shadow replay differs").into());
        }
        let binding = loaded.artifact_binding()?;
        if inactive_bits(&restored, a)? != *frozen {
            return Err(invalid("frozen reloaded source bits differ").into());
        }
        let frozen_native = if readout_mode(a) {
            Some(verify_frozen_native(
                &a.native_artifact,
                &root.join("realizer-native"),
            )?)
        } else {
            None
        };
        let integer = IntegerRealizer::load_native(&root.join("realizer-native"), &binding)?;
        let measured = canonical(&integer, episodes, a, start)?;
        write_json(&root, "canonical.json", &measured)?;
        let mut comparisons = Vec::new();
        for old in prior {
            let oldstep = old["updates"]
                .as_u64()
                .ok_or_else(|| invalid("prior step absent"))?;
            let oldreport = read_json(
                &a.out
                    .join(format!("checkpoint-{oldstep:04}/canonical.json")),
            )?;
            comparisons
                .push(json!({"prior_updates":oldstep,"comparison":compare(&oldreport,&measured)?}));
        }
        write_json(&root, "comparisons.json", &json!(comparisons))?;
        let receipt = json!({"updates":step,"frozen_native_context_receipt":frozen_native,"frozen_source_bits_unchanged":true,"native_equal_episode_ce":measured["native_equal_episode_ce"],"zero_support_positions":measured["zero_support_positions"],"source_parameter_receipts":parameter_receipts(&source.parameters())?,"trusted_export_binding":binding,"canonical_sha256":sha256_file(&root.join("canonical.json"))?});
        write_json(&root, "receipt.json", &receipt)?;
        Ok(receipt)
    })();
    if let Err(e) = &result {
        write_json(
            &root,
            "failure.json",
            &json!({"error":e.to_string(),"updates":step}),
        )?;
    }
    report_output::seal(&root)?;
    report_output::verify(&root)?;
    result
}
fn select(stages: &[Value]) -> Result<usize> {
    let mut best = None;
    for (i, s) in stages.iter().enumerate() {
        if let Some(ce) = s["native_equal_episode_ce"]
            .as_f64()
            .filter(|x| x.is_finite())
        {
            if best.is_none_or(|(_, v)| ce < v) {
                best = Some((i, ce));
            }
        }
    }
    best.map(|(i, _)| i)
        .ok_or_else(|| invalid("no finite native checkpoint").into())
}
fn balanced_indices(update: usize) -> Vec<usize> {
    (0..4)
        .map(|i| (update * 4 + i) % 64)
        .chain((0..4).map(|i| 64 + (update * 4 + i) % 64))
        .collect()
}
fn readout_admission_matches(
    report: &Value,
    development: &str,
    fresh: &str,
    trusted: &str,
) -> bool {
    let mut legacy = report.clone();
    legacy["schema"] = json!("uor-r4.geometric-bank-fit/1");
    legacy["mode"] = json!("broadbatch");
    report["schema"] == "uor-r4.geometric-bank-readout-fit/1"
        && report["mode"] == "readout-broadbatch"
        && report["context_frozen"] == true
        && report["active_families"] == READOUT_FAMILIES
        && report["gradient_report"]["context_gradient_graph_absent"] == true
        && admission_matches(&legacy, development, fresh, trusted)
}
fn verify_frozen_native(parent: &Path, candidate: &Path) -> Result<Value> {
    let mut files = BTreeMap::new();
    for name in [
        "tokenizer.json",
        "consumer/context-q4.bin",
        "consumer/exp-q31.bin",
    ] {
        let old = fs::read(parent.join(name))?;
        if old != fs::read(candidate.join(name))? {
            return Err(invalid(format!("frozen native bytes differ: {name}")).into());
        }
        files.insert(name, sha256_bytes(&old));
    }
    for name in ["metadata.json", "consumer/metadata.json"] {
        if configuration_only(read_json(&parent.join(name))?)?
            != configuration_only(read_json(&candidate.join(name))?)?
        {
            return Err(invalid(format!("frozen native config/geometry differs: {name}")).into());
        }
    }
    Ok(
        json!({"parent_artifact":parent,"unchanged_payload_sha256":files,"configuration_geometry_tokenizer_equal":true}),
    )
}
fn admission_matches(report: &Value, development: &str, fresh: &str, trusted: &str) -> bool {
    report["schema"] == "uor-r4.geometric-bank-fit/1"
        && report["mode"] == "broadbatch"
        && report["status"] == "completed"
        && report["optimizer_updates"] == 0
        && report["cases"] == 128
        && report["complete_objective_finite"] == true
        && report["source_parameters_unchanged"] == true
        && report["development_manifest_sha256"] == development
        && report["fresh_manifest_sha256"] == fresh
        && report["trusted_binding_sha256"] == trusted
        && report["gradient_report"]["independent_native_trace_parity"] == true
}
fn factor_kind(name: &str) -> &'static str {
    if name.starts_with("consumer.context.") {
        "context"
    } else if active(name) {
        "readout"
    } else {
        "inactive"
    }
}
fn donor_compatibility(
    parent: &BTreeMap<String, Var>,
    learned: &BTreeMap<String, Var>,
) -> Result<()> {
    if parent.keys().ne(learned.keys()) {
        return Err(invalid("factor donor parameter inventory differs").into());
    }
    for (name, p) in parent {
        let l = &learned[name];
        if p.dims() != l.dims() || p.dtype() != candle_core::DType::F32 || l.dtype() != p.dtype() {
            return Err(invalid(format!("factor donor shape/dtype differs: {name}")).into());
        }
        if factor_kind(name) == "inactive"
            && p.flatten_all()?
                .to_vec1::<f32>()?
                .iter()
                .map(|x| x.to_bits())
                .ne(l
                    .flatten_all()?
                    .to_vec1::<f32>()?
                    .iter()
                    .map(|x| x.to_bits()))
        {
            return Err(invalid(format!("inactive factor donor bits differ: {name}")).into());
        }
    }
    Ok(())
}
fn assign_factor(
    destination: &BTreeMap<String, Var>,
    parent: &BTreeMap<String, Var>,
    learned: &BTreeMap<String, Var>,
    context48: bool,
    readout48: bool,
) -> Result<BTreeMap<String, &'static str>> {
    if destination.keys().ne(parent.keys()) {
        return Err(invalid("factor destination inventory differs").into());
    }
    let mut donors = BTreeMap::new();
    for (name, var) in destination {
        let take48 = match factor_kind(name) {
            "context" => context48,
            "readout" => readout48,
            _ => false,
        };
        let donor = if take48 {
            &learned[name]
        } else {
            &parent[name]
        };
        var.set(donor.as_tensor())?;
        donors.insert(
            name.clone(),
            if take48 { "checkpoint48" } else { "parent0" },
        );
    }
    Ok(donors)
}
fn configuration_only(mut metadata: Value) -> Result<Value> {
    let m = metadata
        .as_object_mut()
        .ok_or_else(|| invalid("metadata object absent"))?;
    for field in [
        "files",
        "source_parameters",
        "parameter_sha256",
        "packed_sha256",
    ] {
        m.remove(field);
    }
    Ok(metadata)
}
fn factor_pair_report(panel: &Path, canonical: &Value, generation: &Value) -> Result<Value> {
    let data = read_json(&panel.join("context-data.json"))?;
    let rows = data["cases"]
        .as_array()
        .ok_or_else(|| invalid("factor pair metadata absent"))?;
    let cr = canonical["rows"]
        .as_array()
        .ok_or_else(|| invalid("factor canonical rows absent"))?;
    let gr = generation["rows"]
        .as_array()
        .ok_or_else(|| invalid("factor generation rows absent"))?;
    if rows.len() != 128 || cr.len() != 128 || gr.len() != 128 {
        return Err(invalid("factor full128 row count differs").into());
    }
    let mut pairs = BTreeMap::<String, Vec<Value>>::new();
    for ((d, c), g) in rows.iter().zip(cr).zip(gr) {
        if d["id"] != c["id"] || d["id"] != g["id"] {
            return Err(invalid("factor pair row identity differs").into());
        }
        if let Some(pair) = d["pair_id"].as_str() {
            pairs.entry(pair.into()).or_default().push(json!({"id":d["id"],"query_role":d["query_role"],"first_canonical_position":c["tokens"][0],"native_mean_token_ce":c["native_mean_token_ce"],"generated_ids_including_eos":g["generated_ids_including_eos"],"eos":g["eos"],"accepted_complete_answer":g["accepted_complete_answer"]}));
        }
    }
    if pairs.len() != 32
        || pairs
            .values()
            .any(|p| p.len() != 2 || p[0]["query_role"] == p[1]["query_role"])
    {
        return Err(invalid("factor requires32 intact query pairs").into());
    }
    Ok(
        json!({"pairs":pairs,"scope":"labels only after native read; first canonical position has empty actual prefix"}),
    )
}
fn factor_probe(a: &Args, start: Instant) -> Result<Value> {
    let learned_source = a
        .learned_source_weights
        .as_ref()
        .ok_or_else(|| invalid("learned source absent"))?;
    let learned_native = a
        .learned_native_artifact
        .as_ref()
        .ok_or_else(|| invalid("learned native absent"))?;
    let learned_binding = a
        .learned_trusted_native_binding
        .as_ref()
        .ok_or_else(|| invalid("learned trusted binding absent"))?;
    let mut inputs = BTreeMap::new();
    let mut sealed = BTreeSet::new();
    for path in [
        &a.source_weights,
        &a.native_artifact,
        &a.development_panel,
        learned_source,
        learned_native,
    ] {
        let root = nearest_seal(path)?;
        if sealed.insert(root.clone()) {
            report_output::verify(&root)?;
        }
        let manifest = root.join(report_output::MANIFEST_FILE);
        inputs.insert(
            manifest,
            sha256_file(&root.join(report_output::MANIFEST_FILE))?,
        );
    }
    if sha256_file(&a.development_panel.join(report_output::MANIFEST_FILE))?
        != a.development_manifest_sha256
    {
        return Err(invalid("factor development manifest differs").into());
    }
    for path in [&a.trusted_native_binding, learned_binding] {
        inputs.insert(path.clone(), sha256_file(path)?);
    }
    let parent_canonical = a
        .native_artifact
        .parent()
        .ok_or_else(|| invalid("parent checkpoint root absent"))?
        .join("canonical.json");
    let learned_canonical = learned_native
        .parent()
        .ok_or_else(|| invalid("learned checkpoint root absent"))?
        .join("canonical.json");
    for path in [&parent_canonical, &learned_canonical] {
        let root = nearest_seal(path)?;
        report_output::verify(&root)?;
        sealed.insert(root);
        inputs.insert(path.clone(), sha256_file(path)?);
    }
    let retained_parent_canonical = read_json(&parent_canonical)?;
    let retained_learned_canonical = read_json(&learned_canonical)?;
    let parent_binding: NativeArtifactBinding =
        serde_json::from_slice(&read_capped(&a.trusted_native_binding)?)?;
    let learned_binding_value: NativeArtifactBinding =
        serde_json::from_slice(&read_capped(learned_binding)?)?;
    let parent_integer = IntegerRealizer::load_native(&a.native_artifact, &parent_binding)?;
    let _learned_integer = IntegerRealizer::load_native(learned_native, &learned_binding_value)?;
    let bytes = fs::read(a.native_artifact.join("tokenizer.json"))?;
    if fs::read(learned_native.join("tokenizer.json"))? != bytes {
        return Err(invalid("factor tokenizer differs").into());
    }
    let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes)
        .ok_or_else(|| invalid("factor tokenizer invalid"))?;
    let parent = SourceRealizerWeights::load_source(&a.source_weights, &bytes)?;
    let learned = SourceRealizerWeights::load_source(learned_source, &bytes)?;
    let identity: ConsumerIdentity = serde_json::from_value(
        read_json(&a.native_artifact.join("metadata.json"))?["identity"].clone(),
    )?;
    let original = NativeSourceRealizer::load(&a.native_artifact, &parent, &identity)?;
    let learned_original = NativeSourceRealizer::load(learned_native, &learned, &identity)?;
    if original.artifact_binding()? != parent_binding
        || learned_original.artifact_binding()? != learned_binding_value
    {
        return Err(invalid("factor trusted source/native binding differs").into());
    }
    for file in ["metadata.json", "consumer/metadata.json"] {
        if configuration_only(read_json(&a.native_artifact.join(file))?)?
            != configuration_only(read_json(&learned_native.join(file))?)?
        {
            return Err(invalid(format!("factor native config differs: {file}")).into());
        }
    }
    for file in [
        "metadata.json",
        "consumer/context-config.json",
        "consumer/potential/metadata.json",
        "consumer/no-read/metadata.json",
        "period/metadata.json",
    ] {
        if configuration_only(read_json(&a.source_weights.join(file))?)?
            != configuration_only(read_json(&learned_source.join(file))?)?
        {
            return Err(invalid(format!("factor source config differs: {file}")).into());
        }
    }
    for file in ["tokenizer.json", "consumer/exp-q31.bin"] {
        if fs::read(a.native_artifact.join(file))? != fs::read(learned_native.join(file))? {
            return Err(invalid(format!("factor fixed table differs: {file}")).into());
        }
    }
    let p = parent.parameters();
    let l = learned.parameters();
    donor_compatibility(&p, &l)?;
    let initial_parent = parameter_receipts(&p)?;
    let initial_learned = parameter_receipts(&l)?;
    let episodes = load_panel(&a.development_panel, 128, &parent_integer, &tok, true)?;
    let export_size = directory_bytes(&a.source_weights)?
        .checked_add(directory_bytes(&a.native_artifact)?)
        .ok_or_else(|| invalid("factor export projection overflow"))?;
    if export_size
        .checked_mul(4)
        .and_then(|n| n.checked_add(48 * 1024 * 1024))
        .is_none_or(|n| n > FACTOR_REPORT_CAP - 1024 * 1024)
    {
        return Err(invalid("factor four exports plus trace reserve exceed report cap").into());
    }
    let mut reports = Vec::new();
    let mut prior_canonical: Vec<(&str, Value)> = Vec::new();
    let mut prior_generation: Vec<(&str, Value)> = Vec::new();
    for (name, context48, readout48) in [
        ("parent0", false, false),
        ("full48", true, true),
        ("C0R48", false, true),
        ("C48R0", true, false),
    ] {
        deadline(a, start)?;
        let root = a.out.join(name);
        report_output::claim(&root)?;
        let result = (|| -> Result<Value> {
            let source = SourceRealizerWeights::load_source(&a.source_weights, &bytes)?;
            let vars = source.parameters();
            let donors = assign_factor(&vars, &p, &l, context48, readout48)?;
            let expected = parameter_receipts(&vars)?;
            source.save_source(&root.join("realizer-source"))?;
            source
                .compile(identity.clone())?
                .save(&root.join("realizer-native"))?;
            let restored =
                SourceRealizerWeights::load_source(&root.join("realizer-source"), &bytes)?;
            if parameter_receipts(&restored.parameters())? != expected {
                return Err(invalid("factor exported source bits differ").into());
            }
            let loaded =
                NativeSourceRealizer::load(&root.join("realizer-native"), &restored, &identity)?;
            let binding = loaded.artifact_binding()?;
            let native = IntegerRealizer::load_native(&root.join("realizer-native"), &binding)?;
            let mut packed = BTreeMap::new();
            for (file, from48) in [
                ("consumer/context-q4.bin", context48),
                ("consumer/potential-q4.bin", readout48),
                ("consumer/no-read-q4.bin", readout48),
                ("period-q4.bin", readout48),
                ("tokenizer.json", false),
                ("consumer/exp-q31.bin", false),
            ] {
                let donor = if from48 {
                    learned_native
                } else {
                    &a.native_artifact
                };
                if fs::read(root.join("realizer-native").join(file))? != fs::read(donor.join(file))?
                {
                    return Err(
                        invalid(format!("factor packed donor mismatch: {name}/{file}")).into(),
                    );
                }
                packed.insert(file, json!({"donor":if from48{"checkpoint48"}else{"parent0"},"sha256":sha256_file(&root.join("realizer-native").join(file))?}));
            }
            let canonical_report = canonical(&native, &episodes, a, start)?;
            let endpoint_replay = if name == "parent0" || name == "full48" {
                let retained = if name == "parent0" {
                    &retained_parent_canonical
                } else {
                    &retained_learned_canonical
                };
                if canonical_report != *retained {
                    return Err(invalid(format!(
                        "factor endpoint complete canonical replay differs: {name}"
                    ))
                    .into());
                }
                json!({"complete_saved_canonical_equal":true,"retained_sha256":sha256_file(if name=="parent0"{&parent_canonical}else{&learned_canonical})?,"scope":"all saved canonical fields, complete compact native actions/codes/latent/candidate mapping and full context digest; retained canonical does not store raw full context arrays"})
            } else {
                json!({"complete_saved_canonical_equal":null,"scope":"hybrid has no retained endpoint"})
            };

            write_json_limited(
                &root,
                "canonical.json",
                &canonical_report,
                FACTOR_REPORT_CAP,
            )?;
            let generated = generation(&native, &episodes, &tok, a, start)?;
            write_json_limited(&root, "generation.json", &generated, FACTOR_REPORT_CAP)?;
            write_json_limited(
                &root,
                "query-pairs.json",
                &factor_pair_report(&a.development_panel, &canonical_report, &generated)?,
                FACTOR_REPORT_CAP,
            )?;
            let mut comparisons = Vec::new();
            for ((oldname, oldcanonical), (_, oldgeneration)) in
                prior_canonical.iter().zip(&prior_generation)
            {
                let oldrows = oldgeneration["rows"]
                    .as_array()
                    .ok_or_else(|| invalid("prior generation rows absent"))?;
                let newrows = generated["rows"]
                    .as_array()
                    .ok_or_else(|| invalid("factor generation rows absent"))?;
                let rows = oldrows.iter().zip(newrows).map(|(old,new)| json!({"id":new["id"],"previous_output_ids":old["generated_ids_including_eos"],"current_output_ids":new["generated_ids_including_eos"],"previous_eos":old["eos"],"current_eos":new["eos"],"previous_accepted":old["accepted_complete_answer"],"current_accepted":new["accepted_complete_answer"]})).collect::<Vec<_>>();
                comparisons.push(json!({"prior_factor":oldname,"canonical":compare(oldcanonical,&canonical_report)?,"ownprefix":rows}));
            }
            write_json_limited(
                &root,
                "comparisons.json",
                &json!(comparisons),
                FACTOR_REPORT_CAP,
            )?;
            let receipt = json!({"factor":name,"context48":context48,"readout48":readout48,"source_parameter_donors":donors,"source_parameter_receipts":expected,"packed_payload_donors":packed,"endpoint_replay":endpoint_replay,"trusted_export_binding":binding,"canonical_sha256":sha256_file(&root.join("canonical.json"))?,"generation_sha256":sha256_file(&root.join("generation.json"))?,"native_equal_episode_ce":canonical_report["native_equal_episode_ce"],"accepted_complete":generated["accepted_complete"],"optimizer_updates":0});
            write_json_limited(&root, "receipt.json", &receipt, FACTOR_REPORT_CAP)?;
            prior_canonical.push((name, canonical_report));
            prior_generation.push((name, generated));
            Ok(receipt)
        })();
        if let Err(e) = &result {
            write_json_limited(
                &root,
                "failure.json",
                &json!({"factor":name,"error":e.to_string()}),
                FACTOR_REPORT_CAP,
            )?;
        }
        report_output::seal(&root)?;
        report_output::verify(&root)?;
        reports.push(result?);
        if directory_bytes(&a.out)? > FACTOR_REPORT_CAP - 1024 * 1024 {
            return Err(invalid("factor report cap reached").into());
        }
    }
    if parameter_receipts(&p)? != initial_parent || parameter_receipts(&l)? != initial_learned {
        return Err(invalid("factor donors changed").into());
    }
    for (path, hash) in &inputs {
        if sha256_file(path)? != *hash {
            return Err(invalid("factor input changed").into());
        }
    }
    for root in sealed {
        report_output::verify(&root)?;
    }
    Ok(
        json!({"schema":"uor-r4.geometric-bank-factor-probe/1","mode":"factor-probe","status":"completed","optimizer_updates":0,"development_cases":128,"query_pairs":32,"factors":reports,"input_manifests_sha256":inputs,"fresh_predictions":"NOT_RUN","checkpoint_reselection":false,"donor_parameters_unchanged":true,"elapsed_seconds":start.elapsed().as_secs_f64(),"peak_rss_kib_linux":peak_rss_kib(),"scope":"zero-update development context/readout causal intervention; no fresh qualification or replacement of selected parent0"}),
    )
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CueAuthorization {
    schema: String,
    fit_admitted: bool,
    admission_report_sha256: String,
    development_manifest_sha256: String,
    fresh_manifest_sha256: String,
    trusted_binding_sha256: String,
    parent_frozen: bool,
    active_families: String,
    cue_score_mode: CueScoreMode,
    zero_cue_native_metadata_sha256: String,
    updates: usize,
    batch_episodes: usize,
    maximum_fit_seconds: u64,
}
fn cue_native_load<'a>(root: &Path, parent: &'a IntegerRealizer) -> Result<NativeCueCarrier<'a>> {
    let metadata = read_json(&root.join("native-metadata.json"))?;
    let binding: NativeArtifactBinding =
        serde_json::from_value(metadata["parent_artifact"].clone())?;
    if &binding != parent.artifact_binding() {
        return Err(invalid("native-only cue parent binding differs").into());
    }
    let config: CueAngularConfig = serde_json::from_value(metadata["potential"].clone())?;
    let packed = fs::read(root.join("cue-q4.bin"))?;
    let angular = CueAngularQ4::new(config, &packed).map_err(|e| invalid(e.to_string()))?;
    let carrier = parent.compile_cue_carrier(angular)?;
    if serde_json::to_value(carrier.metadata())? != metadata {
        return Err(invalid("native-only cue metadata/context/geometry/q4 receipt differs").into());
    }
    Ok(carrier)
}
fn cue_canonical(
    native: &IntegerRealizer,
    carrier: &NativeCueCarrier<'_>,
    episodes: &[Episode],
    a: &Args,
    start: Instant,
) -> Result<Value> {
    let mut rows = Vec::new();
    let mut total = 0.;
    let mut zeros = Vec::new();
    let mut count = 0;
    for e in episodes {
        let segments = e.segments()?;
        let mut tokens = Vec::new();
        let mut rowce = 0.;
        let mut rowzero = false;
        for (step, &target) in e.target.iter().enumerate() {
            deadline(a, start)?;
            let out = native.read_bank_with_cue_carrier(
                &segments,
                &e.packet.query_ids,
                &e.target[..step],
                carrier,
            )?;
            let mass = out
                .bank
                .actions
                .token_masses
                .iter()
                .filter(|x| x.token_id == target)
                .map(|x| x.weight_q31)
                .sum::<u64>();
            let den = out.bank.actions.total_weight_q31;
            if den == 0 {
                return Err(invalid("cue native zero normalizer").into());
            }
            let ce = if mass == 0 {
                rowzero = true;
                zeros.push(json!({"id":e.packet.id,"step":step}));
                None
            } else {
                let v = -(mass as f64 / den as f64).ln();
                rowce += v;
                Some(v)
            };
            tokens.push(json!({"step":step,"teacherforced_prefix_ids_labels_only":&e.target[..step],"target_label_only_after_read":target,"native_ce":ce,"target_mass_q31":mass,"total_weight_q31":den,"native":compact(&out.bank)?,"cue_carrier":out.carrier}));
            count += 1;
        }
        let mean = if rowzero {
            None
        } else {
            Some(rowce / e.target.len() as f64)
        };
        if let Some(v) = mean {
            total += v / episodes.len() as f64;
        }
        rows.push(json!({"id":e.packet.id,"native_mean_token_ce":mean,"tokens":tokens}));
    }
    Ok(
        json!({"cases":episodes.len(),"target_positions":count,"native_equal_episode_ce":if zeros.is_empty(){Some(total)}else{None},"zero_support_positions":zeros,"rows":rows,"probability_floor":false,"infinite_native_objective_when_zero":true}),
    )
}
fn cue_generation(
    native: &IntegerRealizer,
    carrier: &NativeCueCarrier<'_>,
    episodes: &[Episode],
    tok: &ByteBpeTokenizer,
    a: &Args,
    start: Instant,
) -> Result<Value> {
    let mut rows = Vec::new();
    let mut complete = 0;
    for e in episodes {
        let segments = e.segments()?;
        let mut ids = Vec::new();
        let mut traces = Vec::new();
        for step in 0..a.maximum_generation_tokens {
            deadline(a, start)?;
            let out =
                native.read_bank_with_cue_carrier(&segments, &e.packet.query_ids, &ids, carrier)?;
            let chosen = out.bank.actions.chosen_token_id;
            traces.push(json!({"step":step,"actual_prefix_ids":ids,"native":compact(&out.bank)?,"cue_carrier":out.carrier}));
            ids.push(chosen);
            if chosen == native.binding().eos_token_id() {
                break;
            }
        }
        let eos = ids.last() == Some(&native.binding().eos_token_id());
        let plain = if eos { &ids[..ids.len() - 1] } else { &ids[..] };
        let bytes = tok.decode_bytes(plain);
        let raw = String::from_utf8_lossy(&bytes);
        let text = raw.strip_prefix(' ').unwrap_or(&raw);
        let accepted = eos && String::from_utf8(bytes.clone()).is_ok() && e.answers.accepts(text);
        complete += usize::from(accepted);
        rows.push(json!({"id":e.packet.id,"generated_ids_including_eos":ids,"eos":eos,"reply_text":text,"raw_decoded_bytes_hex":hex::encode(bytes),"accepted_complete_answer":accepted,"tokens":traces}));
    }
    Ok(
        json!({"cases":episodes.len(),"accepted_complete":complete,"maximum_generated_tokens":a.maximum_generation_tokens,"canonical_prefixes_used":false,"native_only_cue_load":true,"rows":rows}),
    )
}
fn cue_batch(
    indices: &[usize],
    episodes: &[Episode],
    source: &SourceRealizerWeights,
    parent: &NativeSourceRealizer,
    weights: &CueAngularWeights,
    integer: Option<&IntegerRealizer>,
    a: &Args,
    start: Instant,
) -> Result<Batch> {
    let began = Instant::now();
    let prepared = source.prepare(parent)?;
    let carrier = parent.compile_cue_carrier(weights.native()?)?;
    let independent = integer
        .map(|p| -> Result<_> { Ok(p.compile_cue_carrier(weights.native()?)?) })
        .transpose()?;
    let params = weights.parameters();
    let parentvars = source.parameters();
    let mut gradients = BTreeMap::<String, Tensor>::new();
    let mut rows = Vec::new();
    let mut mean = 0.;
    let mut positions = 0;
    let lanes = weights.config().heads * weights.config().lanes_per_head;
    let mut visited = vec![BTreeSet::<u8>::new(); lanes];
    let mut address_counts = vec![[0usize; 2]; lanes];
    for &idx in indices {
        let e = &episodes[idx];
        let segments = e.segments()?;
        let mut ce = 0.;
        let mut first = None;
        for (step, &target) in e.target.iter().enumerate() {
            deadline(a, start)?;
            let out = prepared.loss_bank_cue(
                &segments,
                &e.packet.query_ids,
                &e.target[..step],
                target,
                weights,
                &carrier,
            )?;
            if let (Some(native), Some(c)) = (integer, independent.as_ref()) {
                if out.trace
                    != native.read_bank_with_cue_carrier(
                        &segments,
                        &e.packet.query_ids,
                        &e.target[..step],
                        c,
                    )?
                {
                    return Err(invalid("cue loss independent native full trace differs").into());
                }
            }
            for (lane, row) in out.trace.carrier.angular_indices.iter().enumerate() {
                for index in row {
                    if let Some(i) = index {
                        visited[lane].insert(*i);
                        address_counts[lane][1] += 1;
                    } else {
                        address_counts[lane][0] += 1;
                    }
                }
            }
            let expected = -out.target_probability.ln();
            let scalar = out.loss.to_scalar::<f32>()?;
            if !scalar.is_finite()
                || (f64::from(scalar) - expected).abs() > 1e-4 + 1e-5 * expected.abs()
            {
                return Err(invalid("cue loss native marginal differs").into());
            }
            ce += expected;
            positions += 1;
            if step == 0 {
                first = Some(expected);
            }
            let store = (&out.loss * scale(indices.len(), e.target.len())?)?.backward()?;
            if parentvars
                .values()
                .any(|var| store.get(var.as_tensor()).is_some())
            {
                return Err(invalid("cue loss unexpectedly connects frozen parent Var").into());
            }
            for (name, var) in &params {
                if let Some(g) = store.get(var.as_tensor()) {
                    let g = g.detach();
                    let sum = match gradients.remove(name) {
                        Some(old) => (&old + &g)?.detach(),
                        None => g,
                    };
                    gradients.insert(name.clone(), sum);
                }
            }
        }
        mean += ce / e.target.len() as f64 / indices.len() as f64;
        rows.push(json!({"id":e.packet.id,"target_steps":e.target.len(),"native_mean_token_ce":ce/e.target.len() as f64,"first_token_ce":first}));
    }
    let mut stats = BTreeMap::new();
    let mut sq = 0.;
    for (name, g) in &gradients {
        let v = g.flatten_all()?.to_vec1::<f32>()?;
        if v.iter().any(|x| !x.is_finite()) {
            return Err(invalid("cue gradient nonfinite").into());
        }
        sq += v.iter().map(|x| f64::from(*x).powi(2)).sum::<f64>();
        stats.insert(name.clone(),json!({"elements":v.len(),"finite":true,"nonzero":v.iter().filter(|x|**x!=0.).count(),"l1":v.iter().map(|x|f64::from(x.abs())).sum::<f64>()}));
    }
    Ok(Batch {
        gradients,
        report: json!({"episodes":indices.len(),"episode_indices":indices,"target_positions":positions,"native_equal_episode_ce":mean,"gradient_global_norm":sq.sqrt(),"gradient_families":stats,"rows":rows,"angular_visited_bins":visited,"angular_address_counts_absent_present":address_counts,"parent_gradient_graph_absent":true,"independent_native_trace_parity":integer.is_some(),"cue_score_mode":weights.config().mode,"active_families":CUE_FAMILIES,"elapsed_seconds":began.elapsed().as_secs_f64(),"objective":"equal episode mean fullanswer+EOS ordinary native marginal CE; detached F32 token gradient accumulation, globalclip aftermean","cost_scope":"frozen parent source validation/prepared codec packing remains; only cue960Vars backward/Adam"}),
    })
}
fn cue_apply(
    weights: &CueAngularWeights,
    opt: &mut AdamW,
    grad: BTreeMap<String, Tensor>,
) -> Result<f64> {
    let sq = grad
        .values()
        .map(|g| {
            Ok(g.flatten_all()?
                .to_vec1::<f32>()?
                .iter()
                .map(|x| f64::from(*x).powi(2))
                .sum::<f64>())
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .sum::<f64>();
    let norm = sq.sqrt();
    if !norm.is_finite() {
        return Err(invalid("cue clip norm nonfinite").into());
    }
    let clip = if norm > 1. { 1. / norm } else { 1. };
    let mut store = Tensor::new(0f32, &Device::Cpu)?.backward()?;
    let params = weights.parameters();
    for (name, g) in grad {
        let var = params
            .get(&name)
            .ok_or_else(|| invalid("unexpected cue gradient family"))?;
        store.insert(var.as_tensor(), (&g * clip)?.detach());
    }
    opt.step(&store)?;
    weights.project_shadow_range()?;
    Ok(clip)
}
fn cue_checkpoint(
    weights: &CueAngularWeights,
    parent: &NativeSourceRealizer,
    integer: &IntegerRealizer,
    episodes: &[Episode],
    step: usize,
    prior: &[Value],
    a: &Args,
    start: Instant,
) -> Result<Value> {
    let root = a.out.join(format!("checkpoint-{step:04}"));
    report_output::claim(&root)?;
    let result = (|| -> Result<Value> {
        weights.save(&root.join("cue"))?;
        let restored = CueAngularWeights::load(&root.join("cue"), parent, &a.native_artifact)?;
        if parameter_receipts(&weights.parameters())? != parameter_receipts(&restored.parameters())?
        {
            return Err(invalid("cue checkpoint shadows differ").into());
        }
        let carrier = cue_native_load(&root.join("cue"), integer)?;
        let report = cue_canonical(integer, &carrier, episodes, a, start)?;
        write_json(&root, "canonical.json", &report)?;
        let mut comparisons = Vec::new();
        for old in prior {
            let previous = old["updates"]
                .as_u64()
                .ok_or_else(|| invalid("prior cue step absent"))?;
            let saved = read_json(
                &a.out
                    .join(format!("checkpoint-{previous:04}/canonical.json")),
            )?;
            comparisons
                .push(json!({"prior_updates":previous,"comparison":compare(&saved,&report)?}));
        }
        write_json(&root, "comparisons.json", &json!(comparisons))?;
        let receipt = json!({"updates":step,"native_equal_episode_ce":report["native_equal_episode_ce"],"zero_support_positions":report["zero_support_positions"],"cue_source_parameter_receipts":parameter_receipts(&weights.parameters())?,"cue_native_metadata_sha256":sha256_file(&root.join("cue/native-metadata.json"))?,"cue_packed_sha256":sha256_file(&root.join("cue/cue-q4.bin"))?,"canonical_sha256":sha256_file(&root.join("canonical.json"))?,"frozen_parent_native_binding":integer.artifact_binding(),"native_only_generation_contract":true});
        write_json(&root, "receipt.json", &receipt)?;
        Ok(receipt)
    })();
    if let Err(e) = &result {
        write_json(
            &root,
            "failure.json",
            &json!({"error":e.to_string(),"updates":step}),
        )?;
    }
    report_output::seal(&root)?;
    report_output::verify(&root)?;
    result
}
fn cue_admission_matches(
    report: &Value,
    mode: CueScoreMode,
    development: &str,
    fresh: &str,
    trusted: &str,
    zero_metadata: &str,
    frozen: &Value,
) -> bool {
    report["schema"] == "uor-r4.geometric-cue-fit/1"
        && report["mode"] == "cue-broadbatch"
        && report["status"] == "completed"
        && report["optimizer_updates"] == 0
        && report["cases"] == 128
        && report["cue_score_mode"] == json!(mode)
        && report["parent_frozen"] == true
        && report["active_families"] == CUE_FAMILIES
        && report["complete_objective_finite"] == true
        && report["gradient_report"]["parent_gradient_graph_absent"] == true
        && report["gradient_report"]["independent_native_trace_parity"] == true
        && report["development_manifest_sha256"] == development
        && report["fresh_manifest_sha256"] == fresh
        && report["trusted_binding_sha256"] == trusted
        && report["zero_cue_native_metadata_sha256"] == zero_metadata
        && report["frozen_parent_source_receipts"] == *frozen
}
fn cue_run(a: &Args, start: Instant) -> Result<Value> {
    let mut sealed = BTreeSet::new();
    let mut inputs = BTreeMap::new();
    if let Some(controls) = &a.exposed_controls {
        let root = nearest_seal(controls)?;
        report_output::verify(&root)?;
        inputs.insert(
            root.join("manifest.json").to_string_lossy().into_owned(),
            sha256_file(&root.join("manifest.json"))?,
        );
        sealed.insert(root);
    }
    for (path, expected) in [
        (&a.development_panel, &a.development_manifest_sha256),
        (&a.fresh_panel, &a.fresh_manifest_sha256),
    ] {
        report_output::verify(path)?;
        if sha256_file(&path.join("manifest.json"))? != *expected {
            return Err(invalid("cue panel manifest differs").into());
        }
    }
    for p in [
        &a.source_weights,
        &a.native_artifact,
        &a.development_panel,
        &a.fresh_panel,
    ] {
        let root = nearest_seal(p)?;
        report_output::verify(&root)?;
        inputs.insert(
            root.join("manifest.json").to_string_lossy().into_owned(),
            sha256_file(&root.join("manifest.json"))?,
        );
        sealed.insert(root);
    }
    let binding: NativeArtifactBinding =
        serde_json::from_slice(&read_capped(&a.trusted_native_binding)?)?;
    let trusted_sha = sha256_file(&a.trusted_native_binding)?;
    inputs.insert(
        a.trusted_native_binding.to_string_lossy().into_owned(),
        trusted_sha.clone(),
    );
    let integer = IntegerRealizer::load_native(&a.native_artifact, &binding)?;
    let bytes = fs::read(a.native_artifact.join("tokenizer.json"))?;
    let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes)
        .ok_or_else(|| invalid("cue ByteBPE absent"))?;
    let source = SourceRealizerWeights::load_source(&a.source_weights, &bytes)?;
    let identity: ConsumerIdentity = serde_json::from_value(
        read_json(&a.native_artifact.join("metadata.json"))?["identity"].clone(),
    )?;
    let parent = NativeSourceRealizer::load(&a.native_artifact, &source, &identity)?;
    if parent.artifact_binding()? != binding {
        return Err(invalid("cue loaded parent source/native binding differs").into());
    }
    let development = load_panel(&a.development_panel, 128, &integer, &tok, true)?;
    let fresh = load_panel(&a.fresh_panel, 32, &integer, &tok, true)?;
    let mode = a
        .cue_score_mode
        .ok_or_else(|| invalid("cue_score_mode absent"))?;
    let weights = CueAngularWeights::zero(&parent, &a.native_artifact, mode)?;
    let frozen = parameter_receipts(&source.parameters())?;
    weights.save(&a.out.join("initial-cue"))?;
    let initial_metadata_sha = sha256_file(&a.out.join("initial-cue/native-metadata.json"))?;
    let carrier = cue_native_load(&a.out.join("initial-cue"), &integer)?;
    let baseline = cue_canonical(&integer, &carrier, &development, a, start)?;
    write_json(&a.out, "initial-canonical.json", &baseline)?;
    if baseline["native_equal_episode_ce"].is_null() {
        return Err(
            invalid("cue parent full128 native objective infinite; no support filter").into(),
        );
    }
    let original = canonical(&integer, &development, a, start)?;
    for (old, new) in original["rows"]
        .as_array()
        .ok_or_else(|| invalid("parent rows absent"))?
        .iter()
        .zip(
            baseline["rows"]
                .as_array()
                .ok_or_else(|| invalid("cue rows absent"))?,
        )
    {
        for (x, y) in old["tokens"]
            .as_array()
            .ok_or_else(|| invalid("parent tokens absent"))?
            .iter()
            .zip(
                new["tokens"]
                    .as_array()
                    .ok_or_else(|| invalid("cue tokens absent"))?,
            )
        {
            if x["native"] != y["native"]
                || x["target_mass_q31"] != y["target_mass_q31"]
                || x["total_weight_q31"] != y["total_weight_q31"]
            {
                return Err(invalid("zero cue native parent fidelity differs").into());
            }
        }
    }
    let measured = if a.mode == "cue-broadbatch" {
        Some(cue_batch(
            &(0..128).collect::<Vec<_>>(),
            &development,
            &source,
            &parent,
            &weights,
            Some(&integer),
            a,
            start,
        )?)
    } else {
        None
    };
    if let Some(batch) = measured {
        write_json(&a.out, "broadbatch.json", &batch.report)?;
        let parent_generation = cue_generation(&integer, &carrier, &development, &tok, a, start)?;
        write_json(
            &a.out,
            "development-parent-generation.json",
            &parent_generation,
        )?;
        write_json(
            &a.out,
            "parent-query-pair-diagnostics.json",
            &factor_pair_report(&a.development_panel, &baseline, &parent_generation)?,
        )?;
        if parameter_receipts(&source.parameters())? != frozen {
            return Err(invalid("cue broadbatch changed frozen parent").into());
        }
        for (p, h) in &inputs {
            if sha256_file(Path::new(p))? != *h {
                return Err(invalid("cue broad input changed").into());
            }
        }
        for p in &sealed {
            report_output::verify(p)?;
        }
        return Ok(
            json!({"schema":"uor-r4.geometric-cue-fit/1","mode":a.mode,"status":"completed","optimizer_updates":0,"cases":128,"cue_score_mode":mode,"parent_frozen":true,"active_families":CUE_FAMILIES,"gradient_report":batch.report,"native_equal_episode_ce":baseline["native_equal_episode_ce"],"complete_objective_finite":true,"zero_cue_native_metadata_sha256":initial_metadata_sha,"development_manifest_sha256":a.development_manifest_sha256,"fresh_manifest_sha256":a.fresh_manifest_sha256,"trusted_binding_sha256":trusted_sha,"frozen_parent_source_receipts":frozen,"input_manifests_sha256":inputs,"fit_admitted":false,"fresh_predictions":"NOT_RUN","elapsed_seconds":start.elapsed().as_secs_f64(),"peak_rss_kib_linux":peak_rss_kib()}),
        );
    }
    let admission = a
        .admission
        .as_ref()
        .ok_or_else(|| invalid("cue admission absent"))?;
    report_output::verify(admission)?;
    if Some(sha256_file(&admission.join("manifest.json"))?) != a.admission_manifest_sha256 {
        return Err(invalid("cue admission manifest differs").into());
    }
    let report = read_json(&admission.join("report.json"))?;
    let authpath = a
        .fit_authorization
        .as_ref()
        .ok_or_else(|| invalid("cue authorization absent"))?;
    let auth: CueAuthorization = serde_json::from_slice(&read_capped(authpath)?)?;
    if !cue_admission_matches(
        &report,
        mode,
        &a.development_manifest_sha256,
        &a.fresh_manifest_sha256,
        &trusted_sha,
        &initial_metadata_sha,
        &frozen,
    ) || auth.schema != "uor-r4.cue-fit-authorization/1"
        || !auth.fit_admitted
        || !auth.parent_frozen
        || auth.active_families != CUE_FAMILIES
        || auth.cue_score_mode != mode
        || auth.zero_cue_native_metadata_sha256 != initial_metadata_sha
        || auth.admission_report_sha256 != sha256_file(&admission.join("report.json"))?
        || auth.development_manifest_sha256 != a.development_manifest_sha256
        || auth.fresh_manifest_sha256 != a.fresh_manifest_sha256
        || auth.trusted_binding_sha256 != trusted_sha
        || auth.updates != UPDATES
        || auth.batch_episodes != BATCH
        || auth.maximum_fit_seconds != a.maximum_seconds
    {
        return Err(invalid(
            "cue distinct broadbatch/resource/mode/zero-artifact admission differs",
        )
        .into());
    }
    inputs.insert(
        admission
            .join("manifest.json")
            .to_string_lossy()
            .into_owned(),
        sha256_file(&admission.join("manifest.json"))?,
    );
    sealed.insert(admission.clone());
    inputs.insert(
        authpath.to_string_lossy().into_owned(),
        sha256_file(authpath)?,
    );
    let mut stages = Vec::new();
    let zero = cue_checkpoint(
        &weights,
        &parent,
        &integer,
        &development,
        0,
        &stages,
        a,
        start,
    )?;
    stages.push(zero);
    if baseline != read_json(&a.out.join("checkpoint-0000/canonical.json"))? {
        return Err(invalid("cue zero export baseline differs").into());
    }
    let parent_gen = cue_generation(&integer, &carrier, &development, &tok, a, start)?;
    write_json(&a.out, "development-parent-generation.json", &parent_gen)?;
    write_json(
        &a.out,
        "parent-query-pair-diagnostics.json",
        &factor_pair_report(&a.development_panel, &baseline, &parent_gen)?,
    )?;
    let mut optimizer = AdamW::new(
        weights.parameters().into_values().collect(),
        ParamsAdamW {
            lr: 0.003,
            beta1: 0.9,
            beta2: 0.999,
            eps: 1e-8,
            weight_decay: 0.,
        },
    )?;
    let initial_packed = weights.packed_coefficients()?;
    let mut first_cross = None;
    let mut batches = Vec::new();
    for update in 0..UPDATES {
        deadline(a, start)?;
        if parameter_receipts(&source.parameters())? != frozen {
            return Err(invalid("frozen cue parent changed before update").into());
        }
        let batch = cue_batch(
            &balanced_indices(update),
            &development,
            &source,
            &parent,
            &weights,
            None,
            a,
            start,
        )?;
        let clip = cue_apply(&weights, &mut optimizer, batch.gradients)?;
        let packed = weights.packed_coefficients()?;
        if first_cross.is_none() && packed != initial_packed {
            first_cross = Some(update + 1);
        }
        batches.push(json!({"update":update+1,"clip_factor":clip,"cue_packed_sha256":sha256_bytes(&packed),"batch":batch.report}));
        write_json(
            &a.out,
            "progress.json",
            &json!({"optimizer_updates":update+1,"first_native_packed_crossing_update":first_cross,"batches":batches,"elapsed_seconds":start.elapsed().as_secs_f64()}),
        )?;
        if parameter_receipts(&source.parameters())? != frozen {
            return Err(invalid("frozen cue parent changed after update").into());
        }
        if (update + 1) % 16 == 0 {
            stages.push(cue_checkpoint(
                &weights,
                &parent,
                &integer,
                &development,
                update + 1,
                &stages,
                a,
                start,
            )?);
        }
    }
    let selected = select(&stages)?;
    let step = stages[selected]["updates"]
        .as_u64()
        .ok_or_else(|| invalid("cue selected step absent"))?;
    write_json(
        &a.out,
        "selection-before-fresh.json",
        &json!({"selected_updates":step,"criterion":"full128 native equalepisodeCE including parent0 earliestties","fresh_predictions_before_selection":0}),
    )?;
    let selected_carrier =
        cue_native_load(&a.out.join(format!("checkpoint-{step:04}/cue")), &integer)?;
    let selected_can = read_json(&a.out.join(format!("checkpoint-{step:04}/canonical.json")))?;
    let generated = cue_generation(&integer, &selected_carrier, &development, &tok, a, start)?;
    write_json(&a.out, "development-selected-generation.json", &generated)?;
    write_json(
        &a.out,
        "selected-query-pair-diagnostics.json",
        &factor_pair_report(&a.development_panel, &selected_can, &generated)?,
    )?;
    write_json(
        &a.out,
        "fresh-parent-generation.json",
        &cue_generation(&integer, &carrier, &fresh, &tok, a, start)?,
    )?;
    write_json(
        &a.out,
        "fresh-selected-generation.json",
        &cue_generation(&integer, &selected_carrier, &fresh, &tok, a, start)?,
    )?;
    if let Some(root) = &a.exposed_controls {
        let controls = load_panel(root, 6, &integer, &tok, false)?;
        write_json(
            &a.out,
            "controls-parent-generation.json",
            &cue_generation(&integer, &carrier, &controls, &tok, a, start)?,
        )?;
        write_json(
            &a.out,
            "controls-selected-generation.json",
            &cue_generation(&integer, &selected_carrier, &controls, &tok, a, start)?,
        )?;
    }
    if parameter_receipts(&source.parameters())? != frozen {
        return Err(invalid("cue final frozen parent source changed").into());
    }
    for (p, h) in &inputs {
        if sha256_file(Path::new(p))? != *h {
            return Err(invalid("cue immutable input changed").into());
        }
    }
    for root in sealed {
        report_output::verify(&root)?;
    }
    Ok(
        json!({"schema":"uor-r4.geometric-cue-fit/1","mode":a.mode,"status":"completed","cue_score_mode":mode,"parent_frozen":true,"active_families":CUE_FAMILIES,"optimizer_updates":UPDATES,"batch_episodes":BATCH,"episode_visits":4,"batch_schedule":"balanced4single+4bank intactpairs","checkpoints":stages,"selected_updates":step,"first_native_packed_crossing_update":first_cross,"frozen_parent_source_receipts":frozen,"input_manifests_sha256":inputs,"fresh_predictions_before_selection":0,"no_parent_artifact_copies":true,"native_only_cue_generation":true,"elapsed_seconds":start.elapsed().as_secs_f64(),"peak_rss_kib_linux":peak_rss_kib(),"scope":"bounded source-text cue carrier learning; observedH4roots only, presence-masked; radius and latentfiber retainedunusedv1; sharedaliasnormalization; no goldrecord filter or fullchat qualification"}),
    )
}

fn prefix_frozen_cue_unchanged(a: &Args) -> Result<()> {
    let root = a
        .frozen_cue_bundle
        .as_ref()
        .ok_or_else(|| invalid("frozen cue root absent"))?;
    if Some(sha256_file(&root.join("native-metadata.json"))?) != a.frozen_cue_native_metadata_sha256
        || Some(sha256_file(&root.join("cue-q4.bin"))?) != a.frozen_cue_packed_sha256
    {
        return Err(invalid("frozen cue metadata/packed bytes changed").into());
    }
    Ok(())
}
fn prefix_zero_fidelity(original: &Value, baseline: &Value) -> Result<()> {
    let old = original["rows"]
        .as_array()
        .ok_or_else(|| invalid("cue parent rows absent"))?;
    let new = baseline["rows"]
        .as_array()
        .ok_or_else(|| invalid("prefix zero rows absent"))?;
    if old.len() != new.len()
        || original["cases"] != baseline["cases"]
        || original["target_positions"] != baseline["target_positions"]
    {
        return Err(invalid("zero prefix row/target coverage differs").into());
    }
    for (a, b) in old.iter().zip(new) {
        let x = a["tokens"]
            .as_array()
            .ok_or_else(|| invalid("cue parent tokens absent"))?;
        let y = b["tokens"]
            .as_array()
            .ok_or_else(|| invalid("prefix zero tokens absent"))?;
        if a["id"] != b["id"] || x.len() != y.len() {
            return Err(invalid("zero prefix row identity/token coverage differs").into());
        }
        for (x, y) in x.iter().zip(y) {
            for key in [
                "step",
                "teacherforced_prefix_ids_labels_only",
                "target_label_only_after_read",
                "native",
                "target_mass_q31",
                "total_weight_q31",
                "cue_carrier",
            ] {
                if x[key] != y[key] {
                    return Err(
                        invalid(format!("zero prefix cue48 fidelity differs at {key}")).into(),
                    );
                }
            }
        }
    }
    Ok(())
}
fn prefix_training_cue_load<'a>(
    root: &Path,
    parent: &'a NativeSourceRealizer,
) -> Result<NativeCueCarrier<'a>> {
    let metadata = read_json(&root.join("native-metadata.json"))?;
    let config: CueAngularConfig = serde_json::from_value(metadata["potential"].clone())?;
    let carrier = parent.compile_cue_carrier(
        CueAngularQ4::new(config, &fs::read(root.join("cue-q4.bin"))?)
            .map_err(|e| invalid(e.to_string()))?,
    )?;
    if serde_json::to_value(carrier.metadata())? != metadata {
        return Err(invalid("training frozen native cue binding differs").into());
    }
    Ok(carrier)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PrefixAuthorization {
    schema: String,
    fit_admitted: bool,
    admission_report_sha256: String,
    development_manifest_sha256: String,
    fresh_manifest_sha256: String,
    trusted_binding_sha256: String,
    parent_frozen: bool,
    active_families: String,
    prefix_score_mode: PrefixScoreMode,
    zero_prefix_native_metadata_sha256: String,
    frozen_cue_native_metadata_sha256: String,
    frozen_cue_packed_sha256: String,
    updates: usize,
    batch_episodes: usize,
    maximum_fit_seconds: u64,
}
fn prefix_native_load<'a>(
    root: &Path,
    parent: &'a IntegerRealizer,
    cue: &NativeCueCarrier<'_>,
) -> Result<NativePrefixTransport<'a>> {
    let metadata = read_json(&root.join("native-metadata.json"))?;
    let binding: NativeArtifactBinding =
        serde_json::from_value(metadata["parent_artifact"].clone())?;
    if &binding != parent.artifact_binding() {
        return Err(invalid("native-only cue parent binding differs").into());
    }
    let config: PrefixAngularConfig = serde_json::from_value(metadata["potential"].clone())?;
    let packed = fs::read(root.join("prefix-q4.bin"))?;
    let angular = PrefixAngularQ4::new(config, &packed).map_err(|e| invalid(e.to_string()))?;
    let carrier = parent.compile_prefix_transport(cue, angular)?;
    if serde_json::to_value(carrier.metadata())? != metadata {
        return Err(invalid("native-only cue metadata/context/geometry/q4 receipt differs").into());
    }
    Ok(carrier)
}
fn prefix_canonical(
    native: &IntegerRealizer,
    cue: &NativeCueCarrier<'_>,
    carrier: &NativePrefixTransport<'_>,
    episodes: &[Episode],
    a: &Args,
    start: Instant,
) -> Result<Value> {
    let mut rows = Vec::new();
    let mut total = 0.;
    let mut zeros = Vec::new();
    let mut count = 0;
    for e in episodes {
        let segments = e.segments()?;
        let mut tokens = Vec::new();
        let mut rowce = 0.;
        let mut rowzero = false;
        for (step, &target) in e.target.iter().enumerate() {
            deadline(a, start)?;
            let out = native.read_bank_with_prefix_transport(
                &segments,
                &e.packet.query_ids,
                &e.target[..step],
                cue,
                carrier,
            )?;
            let mass = out
                .cue_bank
                .bank
                .actions
                .token_masses
                .iter()
                .filter(|x| x.token_id == target)
                .map(|x| x.weight_q31)
                .sum::<u64>();
            let den = out.cue_bank.bank.actions.total_weight_q31;
            if den == 0 {
                return Err(invalid("cue native zero normalizer").into());
            }
            let ce = if mass == 0 {
                rowzero = true;
                zeros.push(json!({"id":e.packet.id,"step":step}));
                None
            } else {
                let v = -(mass as f64 / den as f64).ln();
                rowce += v;
                Some(v)
            };
            tokens.push(json!({"step":step,"teacherforced_prefix_ids_labels_only":&e.target[..step],"target_label_only_after_read":target,"native_ce":ce,"target_mass_q31":mass,"total_weight_q31":den,"native":compact(&out.cue_bank.bank)?,"cue_carrier":out.cue_bank.carrier,"prefix_transport":out.prefix}));
            count += 1;
        }
        let mean = if rowzero {
            None
        } else {
            Some(rowce / e.target.len() as f64)
        };
        if let Some(v) = mean {
            total += v / episodes.len() as f64;
        }
        rows.push(json!({"id":e.packet.id,"native_mean_token_ce":mean,"tokens":tokens}));
    }
    Ok(
        json!({"cases":episodes.len(),"target_positions":count,"native_equal_episode_ce":if zeros.is_empty(){Some(total)}else{None},"zero_support_positions":zeros,"rows":rows,"probability_floor":false,"infinite_native_objective_when_zero":true}),
    )
}
fn prefix_generation(
    native: &IntegerRealizer,
    cue: &NativeCueCarrier<'_>,
    carrier: &NativePrefixTransport<'_>,
    episodes: &[Episode],
    tok: &ByteBpeTokenizer,
    a: &Args,
    start: Instant,
) -> Result<Value> {
    let mut rows = Vec::new();
    let mut complete = 0;
    for e in episodes {
        let segments = e.segments()?;
        let mut ids = Vec::new();
        let mut traces = Vec::new();
        for step in 0..a.maximum_generation_tokens {
            deadline(a, start)?;
            let out = native.read_bank_with_prefix_transport(
                &segments,
                &e.packet.query_ids,
                &ids,
                cue,
                carrier,
            )?;
            let chosen = out.cue_bank.bank.actions.chosen_token_id;
            traces.push(json!({"step":step,"actual_prefix_ids":ids,"native":compact(&out.cue_bank.bank)?,"cue_carrier":out.cue_bank.carrier,"prefix_transport":out.prefix}));
            ids.push(chosen);
            if chosen == native.binding().eos_token_id() {
                break;
            }
        }
        let eos = ids.last() == Some(&native.binding().eos_token_id());
        let plain = if eos { &ids[..ids.len() - 1] } else { &ids[..] };
        let bytes = tok.decode_bytes(plain);
        let raw = String::from_utf8_lossy(&bytes);
        let text = raw.strip_prefix(' ').unwrap_or(&raw);
        let accepted = eos && String::from_utf8(bytes.clone()).is_ok() && e.answers.accepts(text);
        complete += usize::from(accepted);
        rows.push(json!({"id":e.packet.id,"generated_ids_including_eos":ids,"eos":eos,"reply_text":text,"raw_decoded_bytes_hex":hex::encode(bytes),"accepted_complete_answer":accepted,"tokens":traces}));
    }
    Ok(
        json!({"cases":episodes.len(),"accepted_complete":complete,"maximum_generated_tokens":a.maximum_generation_tokens,"canonical_prefixes_used":false,"native_only_prefix_load":true,"rows":rows}),
    )
}
fn prefix_batch(
    indices: &[usize],
    episodes: &[Episode],
    source: &SourceRealizerWeights,
    parent: &NativeSourceRealizer,
    weights: &PrefixAngularWeights,
    integer: Option<&IntegerRealizer>,
    a: &Args,
    start: Instant,
) -> Result<Batch> {
    let began = Instant::now();
    let prepared = source.prepare(parent)?;
    let cue_root = a
        .frozen_cue_bundle
        .as_ref()
        .ok_or_else(|| invalid("frozen cue bundle absent"))?;
    let cue = prefix_training_cue_load(cue_root, parent)?;
    let carrier = parent.compile_prefix_transport(&cue, weights.native()?)?;
    let independent_cue = integer.map(|p| cue_native_load(cue_root, p)).transpose()?;
    let independent = integer
        .map(|p| -> Result<_> {
            Ok(p.compile_prefix_transport(
                independent_cue
                    .as_ref()
                    .ok_or_else(|| invalid("independent cue absent"))?,
                weights.native()?,
            )?)
        })
        .transpose()?;
    let params = weights.parameters();
    let parentvars = source.parameters();
    let mut gradients = BTreeMap::<String, Tensor>::new();
    let mut rows = Vec::new();
    let mut mean = 0.;
    let mut positions = 0;
    let lanes = weights.config().heads * weights.config().lanes_per_head;
    let mut visited = vec![BTreeSet::<u8>::new(); lanes];
    let mut address_counts = vec![[0usize; 2]; lanes];
    for &idx in indices {
        let e = &episodes[idx];
        let segments = e.segments()?;
        let mut ce = 0.;
        let mut first = None;
        for (step, &target) in e.target.iter().enumerate() {
            deadline(a, start)?;
            let out = prepared.loss_bank_prefix(
                &segments,
                &e.packet.query_ids,
                &e.target[..step],
                target,
                weights,
                &cue,
                &carrier,
            )?;
            if let (Some(native), Some(c)) = (integer, independent.as_ref()) {
                if out.trace
                    != native.read_bank_with_prefix_transport(
                        &segments,
                        &e.packet.query_ids,
                        &e.target[..step],
                        independent_cue
                            .as_ref()
                            .ok_or_else(|| invalid("independent cue absent"))?,
                        c,
                    )?
                {
                    return Err(invalid("cue loss independent native full trace differs").into());
                }
            }
            for (lane, row) in out.trace.prefix.angular_indices.iter().enumerate() {
                for index in row {
                    visited[lane].insert(*index);
                    address_counts[lane][1] += 1;
                }
            }
            let expected = -out.target_probability.ln();
            let scalar = out.loss.to_scalar::<f32>()?;
            if !scalar.is_finite()
                || (f64::from(scalar) - expected).abs() > 1e-4 + 1e-5 * expected.abs()
            {
                return Err(invalid("cue loss native marginal differs").into());
            }
            ce += expected;
            positions += 1;
            if step == 0 {
                first = Some(expected);
            }
            let store = (&out.loss * scale(indices.len(), e.target.len())?)?.backward()?;
            if parentvars
                .values()
                .any(|var| store.get(var.as_tensor()).is_some())
            {
                return Err(invalid("cue loss unexpectedly connects frozen parent Var").into());
            }
            for (name, var) in &params {
                if let Some(g) = store.get(var.as_tensor()) {
                    let g = g.detach();
                    let sum = match gradients.remove(name) {
                        Some(old) => (&old + &g)?.detach(),
                        None => g,
                    };
                    gradients.insert(name.clone(), sum);
                }
            }
        }
        mean += ce / e.target.len() as f64 / indices.len() as f64;
        rows.push(json!({"id":e.packet.id,"target_steps":e.target.len(),"native_mean_token_ce":ce/e.target.len() as f64,"first_token_ce":first}));
    }
    let mut stats = BTreeMap::new();
    let mut sq = 0.;
    for (name, g) in &gradients {
        let v = g.flatten_all()?.to_vec1::<f32>()?;
        if v.iter().any(|x| !x.is_finite()) {
            return Err(invalid("cue gradient nonfinite").into());
        }
        sq += v.iter().map(|x| f64::from(*x).powi(2)).sum::<f64>();
        stats.insert(name.clone(),json!({"elements":v.len(),"finite":true,"nonzero":v.iter().filter(|x|**x!=0.).count(),"l1":v.iter().map(|x|f64::from(x.abs())).sum::<f64>()}));
    }
    Ok(Batch {
        gradients,
        report: json!({"episodes":indices.len(),"episode_indices":indices,"target_positions":positions,"native_equal_episode_ce":mean,"gradient_global_norm":sq.sqrt(),"gradient_families":stats,"rows":rows,"angular_visited_bins":visited,"latent_angular_index_counts_unused_used":address_counts,"latent_state_semantics_no_observed_absence_mask":true,"parent_gradient_graph_absent":true,"independent_native_trace_parity":integer.is_some(),"prefix_score_mode":weights.config().mode,"active_families":PREFIX_FAMILIES,"elapsed_seconds":began.elapsed().as_secs_f64(),"objective":"equal episode mean fullanswer+EOS ordinary native marginal CE; detached F32 token gradient accumulation, globalclip aftermean","cost_scope":"frozen parent source validation/prepared codec packing remains; only prefix960Vars backward/Adam; cue48 native frozen"}),
    })
}
fn prefix_apply(
    weights: &PrefixAngularWeights,
    opt: &mut AdamW,
    grad: BTreeMap<String, Tensor>,
) -> Result<f64> {
    let sq = grad
        .values()
        .map(|g| {
            Ok(g.flatten_all()?
                .to_vec1::<f32>()?
                .iter()
                .map(|x| f64::from(*x).powi(2))
                .sum::<f64>())
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .sum::<f64>();
    let norm = sq.sqrt();
    if !norm.is_finite() {
        return Err(invalid("cue clip norm nonfinite").into());
    }
    let clip = if norm > 1. { 1. / norm } else { 1. };
    let mut store = Tensor::new(0f32, &Device::Cpu)?.backward()?;
    let params = weights.parameters();
    for (name, g) in grad {
        let var = params
            .get(&name)
            .ok_or_else(|| invalid("unexpected cue gradient family"))?;
        store.insert(var.as_tensor(), (&g * clip)?.detach());
    }
    opt.step(&store)?;
    weights.project_shadow_range()?;
    Ok(clip)
}
fn prefix_checkpoint(
    weights: &PrefixAngularWeights,
    parent: &NativeSourceRealizer,
    integer: &IntegerRealizer,
    episodes: &[Episode],
    step: usize,
    prior: &[Value],
    a: &Args,
    start: Instant,
) -> Result<Value> {
    let root = a.out.join(format!("checkpoint-{step:04}"));
    report_output::claim(&root)?;
    let result = (|| -> Result<Value> {
        weights.save(&root.join("prefix"))?;
        let cue_root = a
            .frozen_cue_bundle
            .as_ref()
            .ok_or_else(|| invalid("frozen cue absent"))?;
        let training_cue = prefix_training_cue_load(cue_root, parent)?;
        let restored = PrefixAngularWeights::load(
            &root.join("prefix"),
            parent,
            &a.native_artifact,
            cue_root,
            &training_cue,
        )?;
        if parameter_receipts(&weights.parameters())? != parameter_receipts(&restored.parameters())?
        {
            return Err(invalid("cue checkpoint shadows differ").into());
        }
        let cue = cue_native_load(cue_root, integer)?;
        let carrier = prefix_native_load(&root.join("prefix"), integer, &cue)?;
        let report = prefix_canonical(integer, &cue, &carrier, episodes, a, start)?;
        write_json(&root, "canonical.json", &report)?;
        let mut comparisons = Vec::new();
        for old in prior {
            let previous = old["updates"]
                .as_u64()
                .ok_or_else(|| invalid("prior cue step absent"))?;
            let saved = read_json(
                &a.out
                    .join(format!("checkpoint-{previous:04}/canonical.json")),
            )?;
            comparisons
                .push(json!({"prior_updates":previous,"comparison":compare(&saved,&report)?}));
        }
        write_json(&root, "comparisons.json", &json!(comparisons))?;
        let receipt = json!({"updates":step,"native_equal_episode_ce":report["native_equal_episode_ce"],"zero_support_positions":report["zero_support_positions"],"prefix_source_parameter_receipts":parameter_receipts(&weights.parameters())?,"prefix_native_metadata_sha256":sha256_file(&root.join("prefix/native-metadata.json"))?,"prefix_packed_sha256":sha256_file(&root.join("prefix/prefix-q4.bin"))?,"canonical_sha256":sha256_file(&root.join("canonical.json"))?,"frozen_parent_native_binding":integer.artifact_binding(),"native_only_generation_contract":true});
        write_json(&root, "receipt.json", &receipt)?;
        Ok(receipt)
    })();
    if let Err(e) = &result {
        write_json(
            &root,
            "failure.json",
            &json!({"error":e.to_string(),"updates":step}),
        )?;
    }
    report_output::seal(&root)?;
    report_output::verify(&root)?;
    result
}
fn prefix_admission_matches(
    report: &Value,
    mode: PrefixScoreMode,
    development: &str,
    fresh: &str,
    trusted: &str,
    zero_metadata: &str,
    frozen: &Value,
) -> bool {
    report["schema"] == "uor-r4.geometric-prefix-fit/1"
        && report["mode"] == "prefix-broadbatch"
        && report["status"] == "completed"
        && report["optimizer_updates"] == 0
        && report["cases"] == 128
        && report["prefix_score_mode"] == json!(mode)
        && report["parent_frozen"] == true
        && report["active_families"] == PREFIX_FAMILIES
        && report["complete_objective_finite"] == true
        && report["gradient_report"]["parent_gradient_graph_absent"] == true
        && report["gradient_report"]["independent_native_trace_parity"] == true
        && report["development_manifest_sha256"] == development
        && report["fresh_manifest_sha256"] == fresh
        && report["trusted_binding_sha256"] == trusted
        && report["zero_prefix_native_metadata_sha256"] == zero_metadata
        && report["frozen_parent_source_receipts"] == *frozen
}
fn prefix_run(a: &Args, start: Instant) -> Result<Value> {
    let cue_root = a
        .frozen_cue_bundle
        .as_ref()
        .ok_or_else(|| invalid("frozen cue absent"))?;
    if Some(sha256_file(&cue_root.join("native-metadata.json"))?)
        != a.frozen_cue_native_metadata_sha256
        || Some(sha256_file(&cue_root.join("cue-q4.bin"))?) != a.frozen_cue_packed_sha256
    {
        return Err(invalid("frozen cue48 native metadata/payload binding differs").into());
    }
    let mut sealed = BTreeSet::new();
    let mut inputs = BTreeMap::new();
    if let Some(controls) = &a.exposed_controls {
        let root = nearest_seal(controls)?;
        report_output::verify(&root)?;
        inputs.insert(
            root.join("manifest.json").to_string_lossy().into_owned(),
            sha256_file(&root.join("manifest.json"))?,
        );
        sealed.insert(root);
    }
    for (path, expected) in [
        (&a.development_panel, &a.development_manifest_sha256),
        (&a.fresh_panel, &a.fresh_manifest_sha256),
    ] {
        report_output::verify(path)?;
        if sha256_file(&path.join("manifest.json"))? != *expected {
            return Err(invalid("cue panel manifest differs").into());
        }
    }
    for p in [
        cue_root,
        &a.source_weights,
        &a.native_artifact,
        &a.development_panel,
        &a.fresh_panel,
    ] {
        let root = nearest_seal(p)?;
        report_output::verify(&root)?;
        inputs.insert(
            root.join("manifest.json").to_string_lossy().into_owned(),
            sha256_file(&root.join("manifest.json"))?,
        );
        sealed.insert(root);
    }
    for name in ["native-metadata.json", "cue-q4.bin"] {
        let p = cue_root.join(name);
        inputs.insert(p.to_string_lossy().into_owned(), sha256_file(&p)?);
    }
    let binding: NativeArtifactBinding =
        serde_json::from_slice(&read_capped(&a.trusted_native_binding)?)?;
    let trusted_sha = sha256_file(&a.trusted_native_binding)?;
    inputs.insert(
        a.trusted_native_binding.to_string_lossy().into_owned(),
        trusted_sha.clone(),
    );
    let integer = IntegerRealizer::load_native(&a.native_artifact, &binding)?;
    let bytes = fs::read(a.native_artifact.join("tokenizer.json"))?;
    let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes)
        .ok_or_else(|| invalid("cue ByteBPE absent"))?;
    let source = SourceRealizerWeights::load_source(&a.source_weights, &bytes)?;
    let identity: ConsumerIdentity = serde_json::from_value(
        read_json(&a.native_artifact.join("metadata.json"))?["identity"].clone(),
    )?;
    let parent = NativeSourceRealizer::load(&a.native_artifact, &source, &identity)?;
    if parent.artifact_binding()? != binding {
        return Err(invalid("cue loaded parent source/native binding differs").into());
    }
    let development = load_panel(&a.development_panel, 128, &integer, &tok, true)?;
    let fresh = load_panel(&a.fresh_panel, 32, &integer, &tok, true)?;
    let mode = a
        .prefix_score_mode
        .ok_or_else(|| invalid("prefix_score_mode absent"))?;
    let training_cue = prefix_training_cue_load(cue_root, &parent)?;
    let cue = cue_native_load(cue_root, &integer)?;
    let weights =
        PrefixAngularWeights::zero(&parent, &a.native_artifact, cue_root, &training_cue, mode)?;
    let frozen = parameter_receipts(&source.parameters())?;
    weights.save(&a.out.join("initial-prefix"))?;
    let initial_metadata_sha = sha256_file(&a.out.join("initial-prefix/native-metadata.json"))?;
    let carrier = prefix_native_load(&a.out.join("initial-prefix"), &integer, &cue)?;
    let baseline = prefix_canonical(&integer, &cue, &carrier, &development, a, start)?;
    write_json(&a.out, "initial-canonical.json", &baseline)?;
    if baseline["native_equal_episode_ce"].is_null() {
        return Err(
            invalid("cue parent full128 native objective infinite; no support filter").into(),
        );
    }
    let original = cue_canonical(&integer, &cue, &development, a, start)?;
    prefix_zero_fidelity(&original, &baseline)?;
    let measured = if a.mode == "prefix-broadbatch" {
        Some(prefix_batch(
            &(0..128).collect::<Vec<_>>(),
            &development,
            &source,
            &parent,
            &weights,
            Some(&integer),
            a,
            start,
        )?)
    } else {
        None
    };
    if let Some(batch) = measured {
        write_json(&a.out, "broadbatch.json", &batch.report)?;
        let parent_generation =
            prefix_generation(&integer, &cue, &carrier, &development, &tok, a, start)?;
        write_json(
            &a.out,
            "development-parent-generation.json",
            &parent_generation,
        )?;
        write_json(
            &a.out,
            "parent-query-pair-diagnostics.json",
            &factor_pair_report(&a.development_panel, &baseline, &parent_generation)?,
        )?;
        if parameter_receipts(&source.parameters())? != frozen {
            return Err(invalid("cue broadbatch changed frozen parent").into());
        }
        for (p, h) in &inputs {
            if sha256_file(Path::new(p))? != *h {
                return Err(invalid("cue broad input changed").into());
            }
        }
        for p in &sealed {
            report_output::verify(p)?;
        }
        return Ok(
            json!({"schema":"uor-r4.geometric-prefix-fit/1","mode":a.mode,"status":"completed","optimizer_updates":0,"cases":128,"prefix_score_mode":mode,"parent_frozen":true,"active_families":PREFIX_FAMILIES,"gradient_report":batch.report,"native_equal_episode_ce":baseline["native_equal_episode_ce"],"complete_objective_finite":true,"frozen_cue_native_metadata_sha256":a.frozen_cue_native_metadata_sha256,"frozen_cue_packed_sha256":a.frozen_cue_packed_sha256,"zero_prefix_native_metadata_sha256":initial_metadata_sha,"development_manifest_sha256":a.development_manifest_sha256,"fresh_manifest_sha256":a.fresh_manifest_sha256,"trusted_binding_sha256":trusted_sha,"frozen_parent_source_receipts":frozen,"input_manifests_sha256":inputs,"fit_admitted":false,"fresh_predictions":"NOT_RUN","elapsed_seconds":start.elapsed().as_secs_f64(),"peak_rss_kib_linux":peak_rss_kib()}),
        );
    }
    let admission = a
        .admission
        .as_ref()
        .ok_or_else(|| invalid("cue admission absent"))?;
    report_output::verify(admission)?;
    if Some(sha256_file(&admission.join("manifest.json"))?) != a.admission_manifest_sha256 {
        return Err(invalid("cue admission manifest differs").into());
    }
    let report = read_json(&admission.join("report.json"))?;
    let authpath = a
        .fit_authorization
        .as_ref()
        .ok_or_else(|| invalid("cue authorization absent"))?;
    let auth: PrefixAuthorization = serde_json::from_slice(&read_capped(authpath)?)?;
    if !prefix_admission_matches(
        &report,
        mode,
        &a.development_manifest_sha256,
        &a.fresh_manifest_sha256,
        &trusted_sha,
        &initial_metadata_sha,
        &frozen,
    ) || auth.schema != "uor-r4.prefix-fit-authorization/1"
        || !auth.fit_admitted
        || !auth.parent_frozen
        || auth.active_families != PREFIX_FAMILIES
        || auth.prefix_score_mode != mode
        || Some(auth.frozen_cue_native_metadata_sha256.clone())
            != a.frozen_cue_native_metadata_sha256
        || Some(auth.frozen_cue_packed_sha256.clone()) != a.frozen_cue_packed_sha256
        || auth.zero_prefix_native_metadata_sha256 != initial_metadata_sha
        || auth.admission_report_sha256 != sha256_file(&admission.join("report.json"))?
        || auth.development_manifest_sha256 != a.development_manifest_sha256
        || auth.fresh_manifest_sha256 != a.fresh_manifest_sha256
        || auth.trusted_binding_sha256 != trusted_sha
        || auth.updates != UPDATES
        || auth.batch_episodes != BATCH
        || auth.maximum_fit_seconds != a.maximum_seconds
    {
        return Err(invalid(
            "cue distinct broadbatch/resource/mode/zero-artifact admission differs",
        )
        .into());
    }
    inputs.insert(
        admission
            .join("manifest.json")
            .to_string_lossy()
            .into_owned(),
        sha256_file(&admission.join("manifest.json"))?,
    );
    sealed.insert(admission.clone());
    inputs.insert(
        authpath.to_string_lossy().into_owned(),
        sha256_file(authpath)?,
    );
    let mut stages = Vec::new();
    let zero = prefix_checkpoint(
        &weights,
        &parent,
        &integer,
        &development,
        0,
        &stages,
        a,
        start,
    )?;
    stages.push(zero);
    if baseline != read_json(&a.out.join("checkpoint-0000/canonical.json"))? {
        return Err(invalid("cue zero export baseline differs").into());
    }
    let parent_gen = prefix_generation(&integer, &cue, &carrier, &development, &tok, a, start)?;
    write_json(&a.out, "development-parent-generation.json", &parent_gen)?;
    write_json(
        &a.out,
        "parent-query-pair-diagnostics.json",
        &factor_pair_report(&a.development_panel, &baseline, &parent_gen)?,
    )?;
    let mut optimizer = AdamW::new(
        weights.parameters().into_values().collect(),
        ParamsAdamW {
            lr: 0.003,
            beta1: 0.9,
            beta2: 0.999,
            eps: 1e-8,
            weight_decay: 0.,
        },
    )?;
    let initial_packed = weights.packed_coefficients()?;
    let mut first_cross = None;
    let mut batches = Vec::new();
    for update in 0..UPDATES {
        deadline(a, start)?;
        prefix_frozen_cue_unchanged(a)?;
        if parameter_receipts(&source.parameters())? != frozen {
            return Err(invalid("frozen cue parent changed before update").into());
        }
        let batch = prefix_batch(
            &balanced_indices(update),
            &development,
            &source,
            &parent,
            &weights,
            None,
            a,
            start,
        )?;
        let clip = prefix_apply(&weights, &mut optimizer, batch.gradients)?;
        let packed = weights.packed_coefficients()?;
        if first_cross.is_none() && packed != initial_packed {
            first_cross = Some(update + 1);
        }
        batches.push(json!({"update":update+1,"clip_factor":clip,"prefix_packed_sha256":sha256_bytes(&packed),"batch":batch.report}));
        write_json(
            &a.out,
            "progress.json",
            &json!({"optimizer_updates":update+1,"first_native_packed_crossing_update":first_cross,"batches":batches,"elapsed_seconds":start.elapsed().as_secs_f64()}),
        )?;
        prefix_frozen_cue_unchanged(a)?;
        if parameter_receipts(&source.parameters())? != frozen {
            return Err(invalid("frozen cue parent changed after update").into());
        }
        if (update + 1) % 16 == 0 {
            stages.push(prefix_checkpoint(
                &weights,
                &parent,
                &integer,
                &development,
                update + 1,
                &stages,
                a,
                start,
            )?);
        }
    }
    let selected = select(&stages)?;
    let step = stages[selected]["updates"]
        .as_u64()
        .ok_or_else(|| invalid("cue selected step absent"))?;
    write_json(
        &a.out,
        "selection-before-fresh.json",
        &json!({"selected_updates":step,"criterion":"full128 native equalepisodeCE including parent0 earliestties","fresh_predictions_before_selection":0}),
    )?;
    let selected_carrier = prefix_native_load(
        &a.out.join(format!("checkpoint-{step:04}/prefix")),
        &integer,
        &cue,
    )?;
    let selected_can = read_json(&a.out.join(format!("checkpoint-{step:04}/canonical.json")))?;
    let generated = prefix_generation(
        &integer,
        &cue,
        &selected_carrier,
        &development,
        &tok,
        a,
        start,
    )?;
    write_json(&a.out, "development-selected-generation.json", &generated)?;
    write_json(
        &a.out,
        "selected-query-pair-diagnostics.json",
        &factor_pair_report(&a.development_panel, &selected_can, &generated)?,
    )?;
    write_json(
        &a.out,
        "fresh-parent-generation.json",
        &prefix_generation(&integer, &cue, &carrier, &fresh, &tok, a, start)?,
    )?;
    write_json(
        &a.out,
        "fresh-selected-generation.json",
        &prefix_generation(&integer, &cue, &selected_carrier, &fresh, &tok, a, start)?,
    )?;
    if let Some(root) = &a.exposed_controls {
        let controls = load_panel(root, 6, &integer, &tok, false)?;
        write_json(
            &a.out,
            "controls-parent-generation.json",
            &prefix_generation(&integer, &cue, &carrier, &controls, &tok, a, start)?,
        )?;
        write_json(
            &a.out,
            "controls-selected-generation.json",
            &prefix_generation(&integer, &cue, &selected_carrier, &controls, &tok, a, start)?,
        )?;
    }
    if parameter_receipts(&source.parameters())? != frozen {
        return Err(invalid("cue final frozen parent source changed").into());
    }
    for (p, h) in &inputs {
        if sha256_file(Path::new(p))? != *h {
            return Err(invalid("cue immutable input changed").into());
        }
    }
    for root in sealed {
        report_output::verify(&root)?;
    }
    Ok(
        json!({"schema":"uor-r4.geometric-prefix-fit/1","mode":a.mode,"status":"completed","prefix_score_mode":mode,"parent_frozen":true,"active_families":PREFIX_FAMILIES,"optimizer_updates":UPDATES,"batch_episodes":BATCH,"episode_visits":4,"batch_schedule":"balanced4single+4bank intactpairs","frozen_cue_native_metadata_sha256":a.frozen_cue_native_metadata_sha256,"frozen_cue_packed_sha256":a.frozen_cue_packed_sha256,"checkpoints":stages,"selected_updates":step,"first_native_packed_crossing_update":first_cross,"frozen_parent_source_receipts":frozen,"input_manifests_sha256":inputs,"fresh_predictions_before_selection":0,"no_parent_artifact_copies":true,"native_only_prefix_generation":true,"elapsed_seconds":start.elapsed().as_secs_f64(),"peak_rss_kib_linux":peak_rss_kib(),"scope":"bounded ordered prefix transport learning; latentH4source-before-token+ownprefix; no observedabsence substitution; cue48 frozen; sharedaliasnormalization; no goldrecord filter or fullchat qualification"}),
    )
}

fn run(a: &Args, start: Instant) -> Result<Value> {
    if prefix_mode(a) {
        return prefix_run(a, start);
    }
    if a.mode == "factor-probe" {
        return factor_probe(a, start);
    }
    if cue_mode(a) {
        return cue_run(a, start);
    }
    for (root, expected) in [
        (&a.development_panel, &a.development_manifest_sha256),
        (&a.fresh_panel, &a.fresh_manifest_sha256),
    ] {
        report_output::verify(root)?;
        if sha256_file(&root.join("manifest.json"))? != *expected {
            return Err(invalid("panel manifest binding differs").into());
        }
    }
    let mut inputs = BTreeMap::new();
    let mut sealed = BTreeSet::new();
    for p in [
        &a.source_weights,
        &a.native_artifact,
        &a.development_panel,
        &a.fresh_panel,
    ] {
        let root = nearest_seal(p)?;
        if sealed.insert(root.clone()) {
            report_output::verify(&root)?;
        }
        let manifest = root.join("manifest.json");
        inputs.insert(
            manifest.to_string_lossy().into_owned(),
            sha256_file(&manifest)?,
        );
    }
    let trusted: NativeArtifactBinding =
        serde_json::from_slice(&read_capped(&a.trusted_native_binding)?)?;
    inputs.insert(
        a.trusted_native_binding.to_string_lossy().into_owned(),
        sha256_file(&a.trusted_native_binding)?,
    );
    let integer = IntegerRealizer::load_native(&a.native_artifact, &trusted)?;
    let bytes = fs::read(a.native_artifact.join("tokenizer.json"))?;
    let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes)
        .ok_or_else(|| invalid("ByteBPE absent"))?;
    let source = SourceRealizerWeights::load_source(&a.source_weights, &bytes)?;
    let metadata = read_json(&a.native_artifact.join("metadata.json"))?;
    let identity: ConsumerIdentity = serde_json::from_value(metadata["identity"].clone())?;
    let original = NativeSourceRealizer::load(&a.native_artifact, &source, &identity)?;
    if original.artifact_binding()? != trusted {
        return Err(invalid("trusted original source binding differs").into());
    }
    let development = load_panel(&a.development_panel, 128, &integer, &tok, true)?;
    let fresh = load_panel(&a.fresh_panel, 32, &integer, &tok, true)?;
    let inactive = inactive_bits(&source, a)?;
    let initial = parameter_receipts(&source.parameters())?;
    let base = canonical(&integer, &development, a, start)?;
    write_json(&a.out, "initial-canonical.json", &base)?;
    if base["native_equal_episode_ce"].is_null() {
        return Err(invalid(
            "infinite full128 objective before any update; retained canonical zero-support report",
        )
        .into());
    }
    if broad_mode(a) {
        let indices = (0..128).collect::<Vec<_>>();
        let measured = batch(
            &indices,
            &development,
            &source,
            &original,
            a,
            start,
            Some(&integer),
        )?;
        write_json(&a.out, "broadbatch.json", &measured.report)?;
        if initial != parameter_receipts(&source.parameters())? {
            return Err(invalid("zero-update broadbatch changed source").into());
        }
        for (p, h) in &inputs {
            if sha256_file(Path::new(p))? != *h {
                return Err(invalid("immutable broadbatch input changed").into());
            }
        }
        for p in &sealed {
            report_output::verify(p)?;
        }
        return Ok(
            json!({"schema":if readout_mode(a){"uor-r4.geometric-bank-readout-fit/1"}else{"uor-r4.geometric-bank-fit/1"},"mode":a.mode,"context_frozen":readout_mode(a),"active_families":if readout_mode(a){Some(READOUT_FAMILIES)}else{None},"status":"completed","optimizer_updates":0,"cases":128,"native_equal_episode_ce":base["native_equal_episode_ce"],"complete_objective_finite":true,"gradient_report":measured.report,"source_parameters_unchanged":true,"input_manifests_sha256":inputs,"development_manifest_sha256":a.development_manifest_sha256,"fresh_manifest_sha256":a.fresh_manifest_sha256,"trusted_binding_sha256":sha256_file(&a.trusted_native_binding)?,"source_parameter_receipts":initial,"elapsed_seconds":start.elapsed().as_secs_f64(),"peak_rss_kib_linux":peak_rss_kib(),"fit_admitted":false,"scope":"broadbatch instrument only; no automatic optimization or fresh predictions"}),
        );
    }
    let admission = a
        .admission
        .as_ref()
        .ok_or_else(|| invalid("admission absent"))?;
    report_output::verify(admission)?;
    if Some(sha256_file(&admission.join("manifest.json"))?) != a.admission_manifest_sha256 {
        return Err(invalid("admission manifest differs").into());
    }
    let report = read_json(&admission.join("report.json"))?;
    let auth_path = a
        .fit_authorization
        .as_ref()
        .ok_or_else(|| invalid("authorization absent"))?;
    let auth: Authorization = serde_json::from_slice(&read_capped(auth_path)?)?;
    let matched = if readout_mode(a) {
        readout_admission_matches(
            &report,
            &a.development_manifest_sha256,
            &a.fresh_manifest_sha256,
            &sha256_file(&a.trusted_native_binding)?,
        )
    } else {
        admission_matches(
            &report,
            &a.development_manifest_sha256,
            &a.fresh_manifest_sha256,
            &sha256_file(&a.trusted_native_binding)?,
        )
    };
    if !matched
        || report["source_parameter_receipts"] != initial
        || auth.schema
            != if readout_mode(a) {
                "uor-r4.bank-readout-fit-authorization/1"
            } else {
                "uor-r4.bank-fit-authorization/1"
            }
        || (readout_mode(a)
            && (!auth.context_frozen || auth.active_families.as_deref() != Some(READOUT_FAMILIES)))
        || !auth.fit_admitted
        || auth.admission_report_sha256 != sha256_file(&admission.join("report.json"))?
        || auth.development_manifest_sha256 != a.development_manifest_sha256
        || auth.fresh_manifest_sha256 != a.fresh_manifest_sha256
        || auth.trusted_binding_sha256 != sha256_file(&a.trusted_native_binding)?
        || auth.updates != UPDATES
        || auth.batch_episodes != BATCH
        || auth.maximum_fit_seconds != a.maximum_seconds
    {
        return Err(
            invalid("separate complete broadbatch/resource admission does not bind fit").into(),
        );
    }
    inputs.insert(
        auth_path.to_string_lossy().into_owned(),
        sha256_file(auth_path)?,
    );
    inputs.insert(
        admission
            .join("manifest.json")
            .to_string_lossy()
            .into_owned(),
        sha256_file(&admission.join("manifest.json"))?,
    );
    let mut stages = Vec::new();
    let zero = checkpoint(
        &source,
        &identity,
        &bytes,
        &development,
        0,
        &stages,
        &inactive,
        a,
        start,
    )?;
    stages.push(zero);
    let baseline = read_json(&a.out.join("checkpoint-0000/canonical.json"))?;
    if base != baseline {
        // timings not included; exact allnative rows/objective must replay
        return Err(invalid("exported baseline128 differs original parent").into());
    }
    let parent_generation = generation(&integer, &development, &tok, a, start)?;
    write_json(
        &a.out,
        "development-parent-generation.json",
        &parent_generation,
    )?;
    if readout_mode(a) {
        write_json(
            &a.out,
            "parent-query-pair-diagnostics.json",
            &factor_pair_report(&a.development_panel, &base, &parent_generation)?,
        )?;
    }
    let mut optimizer = AdamW::new(
        source
            .parameters()
            .into_iter()
            .filter(|(n, _)| active_for(n, a))
            .map(|(_, v)| v)
            .collect(),
        ParamsAdamW {
            lr: 0.003,
            beta1: 0.9,
            beta2: 0.999,
            eps: 1e-8,
            weight_decay: 0.,
        },
    )?;
    let mut batches = Vec::new();
    for update in 0..UPDATES {
        deadline(a, start)?;
        let indices = balanced_indices(update);
        if inactive_bits(&source, a)? != inactive {
            return Err(invalid("frozen source bits changed before update").into());
        }
        let current = source.compile(identity.clone())?;
        let measured = batch(&indices, &development, &source, &current, a, start, None)?;
        let clip = apply(&source, &mut optimizer, measured.gradients)?;
        if inactive_bits(&source, a)? != inactive {
            return Err(invalid("inactive potential source bits changed").into());
        }
        batches.push(json!({"update":update+1,"clip_factor":clip,"batch":measured.report}));
        write_json(
            &a.out,
            "progress.json",
            &json!({"optimizer_updates":update+1,"batches":batches,"elapsed_seconds":start.elapsed().as_secs_f64()}),
        )?;
        if (update + 1) % 16 == 0 {
            let s = checkpoint(
                &source,
                &identity,
                &bytes,
                &development,
                update + 1,
                &stages,
                &inactive,
                a,
                start,
            )?;
            stages.push(s);
        }
    }
    let selected = select(&stages)?;
    let step = stages[selected]["updates"]
        .as_u64()
        .ok_or_else(|| invalid("selected step absent"))?;
    write_json(
        &a.out,
        "selection-before-fresh.json",
        &json!({"selected_updates":step,"criterion":"lowest finite full128 native equalepisodeCE including parent0, earliest exact ties","fresh_predictions_before_selection":0}),
    )?;
    let chosen_root = a.out.join(format!("checkpoint-{step:04}"));
    let receipt = read_json(&chosen_root.join("receipt.json"))?;
    let selected_binding: NativeArtifactBinding =
        serde_json::from_value(receipt["trusted_export_binding"].clone())?;
    let selected_model =
        IntegerRealizer::load_native(&chosen_root.join("realizer-native"), &selected_binding)?;
    if readout_mode(a) {
        verify_frozen_native(&a.native_artifact, &chosen_root.join("realizer-native"))?;
    }
    let selected_generation = generation(&selected_model, &development, &tok, a, start)?;
    write_json(
        &a.out,
        "development-selected-generation.json",
        &selected_generation,
    )?;
    if readout_mode(a) {
        let selected_canonical = read_json(&chosen_root.join("canonical.json"))?;
        write_json(
            &a.out,
            "selected-query-pair-diagnostics.json",
            &factor_pair_report(
                &a.development_panel,
                &selected_canonical,
                &selected_generation,
            )?,
        )?;
    }
    write_json(
        &a.out,
        "fresh-parent-generation.json",
        &generation(&integer, &fresh, &tok, a, start)?,
    )?;
    write_json(
        &a.out,
        "fresh-selected-generation.json",
        &generation(&selected_model, &fresh, &tok, a, start)?,
    )?;
    if let Some(root) = &a.exposed_controls {
        let controls = load_panel(root, 6, &integer, &tok, false)?;
        write_json(
            &a.out,
            "controls-parent-generation.json",
            &generation(&integer, &controls, &tok, a, start)?,
        )?;
        write_json(
            &a.out,
            "controls-selected-generation.json",
            &generation(&selected_model, &controls, &tok, a, start)?,
        )?;
    }
    for (p, h) in &inputs {
        if sha256_file(Path::new(p))? != *h {
            return Err(invalid("immutable input changed").into());
        }
    }
    for p in sealed {
        report_output::verify(&p)?;
    }
    Ok(
        json!({"schema":if readout_mode(a){"uor-r4.geometric-bank-readout-fit/1"}else{"uor-r4.geometric-bank-fit/1"},"mode":a.mode,"context_frozen":readout_mode(a),"active_families":if readout_mode(a){Some(READOUT_FAMILIES)}else{None},"status":"completed","optimizer_updates":64,"batch_episodes":BATCH,"batch_schedule":"balanced four original+four bank, cyclic contiguous pairs; each update (update*4+i)%64 and64+(update*4+i)%64","episode_visits":4,"checkpoints":stages,"selected_updates":step,"input_manifests_sha256":inputs,"inactive_source_bits_unchanged":true,"elapsed_seconds":start.elapsed().as_secs_f64(),"peak_rss_kib_linux":peak_rss_kib(),"scope":if readout_mode(a){"actual128 frozen-geometric-context readout coadaptation; quarter STE ordinaryAdamW, no descent guarantee or fullchat qualification"}else{"actual128 exposed causal bank jointlearning; fixed quarter STE ordinaryAdamW no descent guarantee; no pretrained float serving, hard preservation veto, hidden source selection or fullchat qualification"}}),
    )
}
fn main() {
    let outcome = (|| -> Result<()> {
        let a = checked_args()?;
        report_output::claim(&a.out)?;
        if readout_mode(&a) || cue_mode(&a) || prefix_mode(&a) {
            fs::write(
                a.out.join("resource-cap.json"),
                serde_json::to_vec(&json!({"maximum_report_bytes":a.maximum_report_bytes}))?,
            )?;
        }
        let start = Instant::now();
        let result = run(&a, start);
        match &result {
            Ok(v) => {
                let mut v = v.clone();
                let (exe, lookup) = executable()?;
                v["source_commit"] = json!(option_env!("UOR_BUILD_SOURCE_COMMIT"));
                v["executable_sha256"] = json!(sha256_file(&exe)?);
                v["executable_lookup"] = json!(lookup);
                v["host_os"] = json!(std::env::consts::OS);
                v["host_arch"] = json!(std::env::consts::ARCH);
                write_json_limited(&a.out, "report.json", &v, a.maximum_report_bytes)?;
            }
            Err(e) => {
                write_json_limited(
                    &a.out,
                    "failure.json",
                    &json!({"status":"failed_or_stopped","error":e.to_string(),"mode":a.mode,"elapsed_seconds":start.elapsed().as_secs_f64(),"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"completed_updates_in_progress":read_json(&a.out.join("progress.json")).ok()}),
                    a.maximum_report_bytes,
                )?;
            }
        }
        report_output::seal(&a.out)?;
        report_output::verify(&a.out)?;
        result.map(|_| ())
    })();
    if let Err(e) = outcome {
        eprintln!("bank fit: {e}");
        std::process::exit(1);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn prefix_zero_fidelity_checks_all_rows_tokens_and_cue_parent() -> Result<()> {
        let token = json!({"step":0,"teacherforced_prefix_ids_labels_only":[],"target_label_only_after_read":4,"native":{"actions":"actual"},"target_mass_q31":9,"total_weight_q31":10,"cue_carrier":{"actual":"frozen cue48"}});
        let parent = json!({"cases":1,"target_positions":1,"rows":[{"id":"row","tokens":[token]}]});
        prefix_zero_fidelity(&parent, &parent)?;
        let mut missing = parent.clone();
        missing["rows"][0]["tokens"] = json!([]);
        assert!(prefix_zero_fidelity(&parent, &missing).is_err());
        let mut extra = parent.clone();
        extra["rows"]
            .as_array_mut()
            .ok_or_else(|| invalid("fixture rows"))?
            .push(parent["rows"][0].clone());
        assert!(prefix_zero_fidelity(&parent, &extra).is_err());
        let mut changed = parent.clone();
        changed["rows"][0]["tokens"][0]["cue_carrier"]["actual"] = json!("foreign cue");
        assert!(prefix_zero_fidelity(&parent, &changed).is_err());
        Ok(())
    }
    #[test]
    fn prefix_admission_is_distinct_and_binds_zero_native_and_data() {
        let frozen = json!({"parent":"bits"});
        let report = json!({"schema":"uor-r4.geometric-prefix-fit/1","mode":"prefix-broadbatch","status":"completed","optimizer_updates":0,"cases":128,"prefix_score_mode":"DirectedRelative","parent_frozen":true,"active_families":PREFIX_FAMILIES,"complete_objective_finite":true,"gradient_report":{"parent_gradient_graph_absent":true,"independent_native_trace_parity":true},"development_manifest_sha256":"dev","fresh_manifest_sha256":"fresh","trusted_binding_sha256":"trusted","zero_prefix_native_metadata_sha256":"zero-bound-full-cue","frozen_parent_source_receipts":frozen});
        assert!(prefix_admission_matches(
            &report,
            PrefixScoreMode::DirectedRelative,
            "dev",
            "fresh",
            "trusted",
            "zero-bound-full-cue",
            &frozen
        ));
        assert!(!prefix_admission_matches(
            &report,
            PrefixScoreMode::SourcePrefixUnary,
            "dev",
            "fresh",
            "trusted",
            "zero-bound-full-cue",
            &frozen
        ));
        assert!(!prefix_admission_matches(
            &report,
            PrefixScoreMode::DirectedRelative,
            "other",
            "fresh",
            "trusted",
            "zero-bound-full-cue",
            &frozen
        ));
        assert!(!prefix_admission_matches(
            &report,
            PrefixScoreMode::DirectedRelative,
            "dev",
            "fresh",
            "trusted",
            "foreign-cue-bound-zero",
            &frozen
        ));
    }
    #[test]
    fn cue_admission_binds_mode_zero_artifact_and_all_frozen_inputs() {
        let frozen = json!({"context":"exact","readout":"exact"});
        let r = json!({"schema":"uor-r4.geometric-cue-fit/1","mode":"cue-broadbatch","status":"completed","optimizer_updates":0,"cases":128,"cue_score_mode":"DirectedRelative","parent_frozen":true,"active_families":CUE_FAMILIES,"complete_objective_finite":true,"gradient_report":{"parent_gradient_graph_absent":true,"independent_native_trace_parity":true},"development_manifest_sha256":"dev","fresh_manifest_sha256":"fresh","trusted_binding_sha256":"parent","zero_cue_native_metadata_sha256":"zero","frozen_parent_source_receipts":frozen});
        assert!(cue_admission_matches(
            &r,
            CueScoreMode::DirectedRelative,
            "dev",
            "fresh",
            "parent",
            "zero",
            &frozen
        ));
        assert!(!cue_admission_matches(
            &r,
            CueScoreMode::CueUnary,
            "dev",
            "fresh",
            "parent",
            "zero",
            &frozen
        ));
        for (field, value) in [
            ("development_manifest_sha256", "wrong"),
            ("fresh_manifest_sha256", "wrong"),
            ("trusted_binding_sha256", "wrong"),
            ("zero_cue_native_metadata_sha256", "wrong"),
            ("mode", "readout-broadbatch"),
        ] {
            let mut other = r.clone();
            other[field] = json!(value);
            assert!(!cue_admission_matches(
                &other,
                CueScoreMode::DirectedRelative,
                "dev",
                "fresh",
                "parent",
                "zero",
                &frozen
            ));
        }
        assert!(!cue_admission_matches(
            &r,
            CueScoreMode::DirectedRelative,
            "dev",
            "fresh",
            "parent",
            "zero",
            &json!({"context":"changed"})
        ));
    }
    #[test]
    fn readout_admission_rejects_joint_or_wrong_freeze_receipt() {
        let mut r = json!({"schema":"uor-r4.geometric-bank-readout-fit/1","mode":"readout-broadbatch","context_frozen":true,"active_families":READOUT_FAMILIES,"status":"completed","optimizer_updates":0,"cases":128,"complete_objective_finite":true,"source_parameters_unchanged":true,"development_manifest_sha256":"dev","fresh_manifest_sha256":"fresh","trusted_binding_sha256":"native","gradient_report":{"independent_native_trace_parity":true,"context_gradient_graph_absent":true}});
        assert!(readout_admission_matches(&r, "dev", "fresh", "native"));
        assert!(!admission_matches(&r, "dev", "fresh", "native"));
        r["mode"] = json!("broadbatch");
        assert!(!readout_admission_matches(&r, "dev", "fresh", "native"));
        r["mode"] = json!("readout-broadbatch");
        r["gradient_report"]["context_gradient_graph_absent"] = json!(false);
        assert!(!readout_admission_matches(&r, "dev", "fresh", "native"));
    }
    #[test]
    fn readout_mask_keeps_four_copy_and_terminal_families_only() {
        for name in [
            "consumer.potential.context_unary",
            "consumer.potential.context_radius",
            "consumer.potential.context_presence",
            "consumer.potential.content_presence",
            "consumer.no_read.coefficients",
            "period.coefficients",
        ] {
            assert!(readout_active(name));
        }
        for name in [
            "consumer.context.tokens",
            "consumer.context.basis",
            "consumer.potential.pair_unary",
            "consumer.potential.phase",
        ] {
            assert!(!readout_active(name));
        }
    }
    #[test]
    fn factor_donors_preserve_inactive_bits_and_reject_shape_inventory_changes() -> Result<()> {
        let make = |v: Vec<f32>| -> Result<Var> {
            Ok(Var::from_tensor(&Tensor::from_vec(
                v.clone(),
                v.len(),
                &Device::Cpu,
            )?)?)
        };
        let mut parent = BTreeMap::new();
        parent.insert("consumer.context.token_root".into(), make(vec![0., 1.])?);
        parent.insert(
            "consumer.potential.context_unary".into(),
            make(vec![2., 3.])?,
        );
        parent.insert("consumer.potential.content_unary".into(), make(vec![-0.])?);
        let mut learned = BTreeMap::new();
        learned.insert("consumer.context.token_root".into(), make(vec![3., 4.])?);
        learned.insert(
            "consumer.potential.context_unary".into(),
            make(vec![5., 6.])?,
        );
        learned.insert("consumer.potential.content_unary".into(), make(vec![-0.])?);
        donor_compatibility(&parent, &learned)?;
        for (c, r) in [(false, false), (true, true), (false, true), (true, false)] {
            let destination = parent
                .iter()
                .map(|(name, v)| Ok((name.clone(), Var::from_tensor(&v.as_tensor().copy()?)?)))
                .collect::<Result<BTreeMap<_, _>>>()?;
            let donors = assign_factor(&destination, &parent, &learned, c, r)?;
            for (name, var) in &destination {
                let chosen = match factor_kind(name) {
                    "context" => c,
                    "readout" => r,
                    _ => false,
                };
                let expected = if chosen {
                    &learned[name]
                } else {
                    &parent[name]
                };
                assert_eq!(
                    var.flatten_all()?.to_vec1::<f32>()?,
                    expected.flatten_all()?.to_vec1::<f32>()?
                );
                assert_eq!(
                    donors[name],
                    if chosen { "checkpoint48" } else { "parent0" }
                );
            }
        }

        learned.insert("consumer.potential.content_unary".into(), make(vec![0.])?);
        assert!(donor_compatibility(&parent, &learned).is_err()); // numeric equality is insufficient
        learned.insert("consumer.potential.content_unary".into(), make(vec![-0.])?);
        learned.insert("consumer.context.token_root".into(), make(vec![1.])?);
        assert!(donor_compatibility(&parent, &learned).is_err());
        learned.remove("consumer.context.token_root");
        assert!(donor_compatibility(&parent, &learned).is_err());
        assert_eq!(factor_kind("consumer.context.token_root"), "context");
        for n in [
            "consumer.potential.context_unary",
            "consumer.potential.context_radius",
            "consumer.potential.context_presence",
            "consumer.potential.content_presence",
            "consumer.no_read.bias",
            "period.bias",
        ] {
            assert_eq!(factor_kind(n), "readout");
        }
        assert_eq!(factor_kind("consumer.potential.pair_unary"), "inactive");
        Ok(())
    }
    #[test]
    fn factor_metadata_keeps_geometry_and_configuration_while_rebinding_inventory() -> Result<()> {
        let a = json!({"config":{"heads":2},"identity":{"algebra":"bound"},"files":{"x":"old"},"source_parameters":{"x":"old"}});
        let mut b = a.clone();
        b["files"]["x"] = json!("new");
        b["source_parameters"]["x"] = json!("new");
        assert_eq!(
            configuration_only(a.clone())?,
            configuration_only(b.clone())?
        );
        b["identity"]["algebra"] = json!("foreign");
        assert_ne!(configuration_only(a)?, configuration_only(b)?);
        Ok(())
    }
    #[test]
    fn preserved_five_alias_answers_keep_canonical_first_and_all_membership() -> Result<()> {
        let answers: FrozenAnswers = serde_json::from_value(
            json!({"intent":"current","accepted":["singer.","It's singer.","It is singer.","You work as a singer.","Your job is singer."]}),
        )?;
        validate_answers(&answers, true)?;
        assert_eq!(answers.accepted[0], "singer.");
        assert_eq!(answers.accepted.len(), 5);
        for text in &answers.accepted {
            assert!(answers.accepts(text));
        }
        assert!(!answers.accepts("dancer."));
        assert!(validate_answers(&answers, false).is_err());
        Ok(())
    }
    #[test]
    fn native_selection_infinity_ties_and_episode_scaling() -> Result<()> {
        let s = vec![
            json!({"updates":0,"native_equal_episode_ce":1.}),
            json!({"updates":16,"native_equal_episode_ce":null}),
            json!({"updates":32,"native_equal_episode_ce":1.}),
            json!({"updates":48,"native_equal_episode_ce":0.5}),
            json!({"updates":64,"native_equal_episode_ce":0.5}),
        ];
        assert_eq!(select(&s)?, 3);
        assert_eq!(scale(2, 2)? * 2., 0.5);
        assert_eq!(scale(2, 7)? * 7., 0.5);
        assert!(scale(0, 1).is_err());
        Ok(())
    }
    #[test]
    fn balanced_schedule_preserves_four_visits_and_intact_query_pairs() {
        let mut count = [0usize; 128];
        for update in 0..64 {
            let indices = balanced_indices(update);
            assert_eq!(indices.len(), 8);
            assert!(indices[..4].iter().all(|i| *i < 64));
            assert!(indices[4..].iter().all(|i| *i >= 64));
            for pair in indices[4..].chunks(2) {
                assert_eq!(pair[0] % 2, 0);
                assert_eq!(pair[1], pair[0] + 1);
            }
            for i in indices {
                count[i] += 1;
            }
        }
        assert!(count.iter().all(|n| *n == 4));
    }
    #[test]
    fn admission_must_bind_current_panel_and_native_receipts() {
        let mut r = json!({"schema":"uor-r4.geometric-bank-fit/1","mode":"broadbatch","status":"completed","optimizer_updates":0,"cases":128,"complete_objective_finite":true,"source_parameters_unchanged":true,"development_manifest_sha256":"dev","fresh_manifest_sha256":"fresh","trusted_binding_sha256":"native","gradient_report":{"independent_native_trace_parity":true}});
        assert!(admission_matches(&r, "dev", "fresh", "native"));
        assert!(!admission_matches(&r, "wrong", "fresh", "native"));
        assert!(!admission_matches(&r, "dev", "wrong", "native"));
        assert!(!admission_matches(&r, "dev", "fresh", "wrong"));
        r["gradient_report"]["independent_native_trace_parity"] = json!(false);
        assert!(!admission_matches(&r, "dev", "fresh", "native"));
    }
    #[test]
    fn target_fields_rejected_from_runtime_packets() {
        let s = r#"{"schema":"uor-r4.native-source-bank-probe-input/1","cases":[{"id":"x","segments":[],"query_ids":[1],"actual_prefix_ids":[],"target_ids":[3]}]}"#;
        assert!(serde_json::from_str::<Inputs>(s).is_err());
    }
}
