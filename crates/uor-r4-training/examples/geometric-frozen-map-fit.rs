//! Matched exact-checkpoint restart with a frozen categorical transport map.
//! Targets enter only the token-alias objective; native source selection stays target-free.
#![recursion_limit = "256"]
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
    answer_oracle::FrozenAnswers,
    native_geometric::learner::{
        geometric_generate::{GenerateReadCounts, NativeGeometricGenerate},
        geometric_read_state_bridge::{BridgeReadCounts, NativeGeometricReadStateBridge},
    },
    report_output,
};
use uor_r4_integer::{
    geometric_context::NativeContextState,
    geometric_cue_carrier::{CueAngularConfig, CueAngularQ4, CueJointMetadata, CueJointQ4},
    geometric_occurrence_read::{FrameMetadata, FrameStatus, SelectedRecordFrame, SourceIdentity},
    geometric_prefix_transport::{PrefixAngularConfig, PrefixAngularQ4},
    geometric_source_emission_view::SourceEmissionView,
    geometric_source_realizer::{
        NativeArtifactBinding, NativeSourceRealizer as IntegerRealizer, SourceBankSegment,
    },
    geometric_vocabulary_actions::NativeVocabularyActions,
};
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::{
    geometric_bank_generate::PreparedBankGenerate,
    geometric_generate_learning::{GenerateLearningWeights, VocabularyScoreAdjoint},
    geometric_occurrence_consumer::{
        source_realizer::{NativeSourceRealizer, SourceRealizerWeights},
        ConsumerIdentity,
    },
    geometric_read_state_bridge::{BridgeLearningWeights, PreparedCategoricalBridge},
    sha256_bytes, sha256_file,
};
#[path = "../../uor-r4-integer/examples/support/source_probe.rs"]
mod output_support;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Mode {
    Admission,
    Fit,
    PredictionControl,
    EntryCeiling,
}
#[derive(Clone, Copy, Debug, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct ControlRates {
    generate: f64,
    prototype: f64,
    context: f64,
    potential: f64,
}
impl Default for ControlRates {
    fn default() -> Self {
        Self {
            generate: 0.003,
            prototype: 0.01,
            context: 0.002,
            potential: 0.003,
        }
    }
}
fn control_settings(a: &Args) -> Result<(usize, ControlRates)> {
    if a.mode != Mode::PredictionControl
        && (a.prediction_control_updates.is_some()
            || a.prediction_control_rates.is_some()
            || a.prediction_control_resume.is_some()
            || a.prediction_control_resume_source_step.is_some()
            || a.prediction_control_resume_generate_step.is_some()
            || a.prediction_control_trainable != ControlTrainable::Joint)
    {
        return Err(bad(
            "prediction control settings cannot change fixed fit/admission",
        ));
    }
    if (a.prediction_control_trainable != ControlTrainable::Joint)
        != a.prediction_control_resume.is_some()
    {
        return Err(bad("Generate-field control requires explicit resume; resume requires Generate-field control"));
    }
    if a.prediction_control_resume.is_none()
        && (a.prediction_control_resume_source_step.is_some()
            || a.prediction_control_resume_generate_step.is_some())
    {
        return Err(bad("component steps require authenticated resume"));
    }
    let n = a.prediction_control_updates.unwrap_or(32);
    let r = a.prediction_control_rates.unwrap_or_default();
    if ![0, 16, 32, 64].contains(&n)
        || [r.generate, r.prototype, r.context, r.potential]
            .iter()
            .any(|x| !x.is_finite() || *x <= 0.)
    {
        return Err(bad(
            "control requires explicit0/16/32/64 updates and finite positive rates",
        ));
    }
    Ok((n, r))
}
#[derive(Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Credit {
    Clipped,
    RawIdentity,
}
impl Credit {
    fn policy(self) -> VocabularyScoreAdjoint {
        match self {
            Self::Clipped => VocabularyScoreAdjoint::Clipped,
            Self::RawIdentity => VocabularyScoreAdjoint::RawIdentity,
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::Clipped => "clipped",
            Self::RawIdentity => "raw_identity",
        }
    }
}
/// Offline Context transport pullback only. Both variants use the same hard
/// frozen categorical marker artifact and target-free physical Source route.
#[derive(Clone, Copy, Debug, Default, Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ReadStatePullback {
    #[default]
    Legacy,
    Categorical,
}
impl ReadStatePullback {
    fn name(self) -> &'static str {
        match self {
            Self::Legacy => "legacy",
            Self::Categorical => "categorical",
        }
    }
}
/// Which supervised phases may carry credit. `all` is the declared
/// phase-balanced objective and stays byte-identical to the retained fits.
/// `entry_only` zeroes every later-position weight and leaves the entry
/// position's own balanced weight untouched, so the entry decision is learned
/// with no later-position credit in the four shared parameter groups. It
/// isolates one variable: the later-phase gradient, not the entry weight.
#[derive(Clone, Copy, Debug, Default, Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum LossScope {
    #[default]
    All,
    EntryOnly,
}
impl LossScope {
    fn name(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::EntryOnly => "entry_only",
        }
    }
}
#[derive(Clone, Copy, Debug, Default, Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ControlTrainable {
    #[default]
    Joint,
    GenerateField,
    PotentialGenerateField,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Args {
    mode: Mode,
    credit: Credit,
    #[serde(default)]
    read_state_pullback: ReadStatePullback,
    seed: u64,
    checkpoint: PathBuf,
    saved_fit: PathBuf,
    categorical: PathBuf,
    parent_config: PathBuf,
    training_inputs: PathBuf,
    training_labels: PathBuf,
    development_inputs: PathBuf,
    development_labels: PathBuf,
    maximum_seconds: u64,
    maximum_report_bytes: u64,
    out: PathBuf,
    #[serde(default = "default_updates")]
    updates: usize,
    #[serde(default)]
    loss_scope: LossScope,
    /// Fit a query-keyed entry scorer on one panel half and apply it at the entry
    /// position through the native forward, reporting the held-out half.
    #[serde(default)]
    ceiling_scorer: bool,
    #[serde(default)]
    baseline: Option<PathBuf>,
    #[serde(default)]
    prediction_control_updates: Option<usize>,
    #[serde(default)]
    prediction_control_rates: Option<ControlRates>,
    #[serde(default)]
    prediction_control_resume: Option<PathBuf>,
    #[serde(default)]
    prediction_control_resume_source_step: Option<usize>,
    #[serde(default)]
    prediction_control_resume_generate_step: Option<usize>,
    #[serde(default)]
    prediction_control_trainable: ControlTrainable,
}
const CONTROL_INDICES: [usize; 8] = [0, 1, 4, 5, 8, 9, 12, 13];
fn default_updates() -> usize {
    UPDATES
}
const INPUT_SHA: &str = "b9661606b280884217a64e0a5b643f8324a90390e47ade7241da0889a5f7c86a";
const LABEL_SHA: &str = "84991e0657b5697c0e061eaa3fe86e4a0ec7ce6bc2be8371b62698c6b8126155";
const CP_RECEIPT_SHA: &str = "9196520209cb8ed172fea65a60be0a1f2fa788e40d5857c65bf93eccb658b816";
const CAT_SHA: &str = "af5039e74c6cd92e2fa0bf4c72c315ef34f912d77b09fc607215a18f86cd1d0e";
const QUARTER_SHA: &str = "d7e56ed124ad80619dad5b6bfb90dec5f78fe855f43ef4bfd9011e2e31f5fccf";
const GENERATE_SHA: &str = "41c7beae25e98fdc658ff667959a7c02a6b916b04298a425f7f7f48285d21511";
const UPDATES: usize = 128;
const BATCH: usize = 8;
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
            _ => None,
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
struct Episode {
    packet: Packet,
    answers: FrozenAnswers,
    target: Vec<u32>,
    views: Vec<Option<SourceEmissionView>>,
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
                        frame: s.frame().ok_or_else(|| bad("source frame missing"))?,
                        view: self.views[i]
                            .as_ref()
                            .ok_or_else(|| bad("source view missing"))?,
                        event: *event,
                    },
                    Segment::Context {
                        event,
                        role,
                        token_ids,
                    } => SourceBankSegment::Context {
                        event: *event,
                        role: *role,
                        token_ids,
                    },
                })
            })
            .collect()
    }
    fn base_len(&self) -> usize {
        self.packet.query_ids.len()
            + self
                .packet
                .segments
                .iter()
                .zip(&self.views)
                .map(|(s, v)| match (s, v) {
                    (Segment::Source { .. }, Some(v)) => v.emitted_token_ids().len(),
                    (Segment::Context { token_ids, .. }, _) => token_ids.len(),
                    _ => 0,
                })
                .sum::<usize>()
    }
    fn has_source(&self) -> bool {
        self.packet
            .segments
            .iter()
            .any(|s| matches!(s, Segment::Source { .. }))
    }
    fn causal_no_source(&self, prefix: &[u32]) -> Result<Vec<u32>> {
        let mut ids = Vec::new();
        for s in &self.packet.segments {
            match s {
                Segment::Context { token_ids, .. } => ids.extend_from_slice(token_ids),
                _ => return Err(bad("no-source branch contains Source")),
            }
        }
        ids.extend_from_slice(&self.packet.query_ids);
        ids.extend_from_slice(prefix);
        if ids.is_empty() || ids.len() > 128 {
            return Err(bad("no-source causal context outside1..128"));
        }
        Ok(ids)
    }
}
fn bad(s: &str) -> Box<dyn std::error::Error> {
    io::Error::new(io::ErrorKind::InvalidData, s).into()
}
fn read(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
fn size(p: &Path) -> Result<u64> {
    let mut total = 0;
    for e in fs::read_dir(p)? {
        let p = e?.path();
        total += if p.is_dir() {
            size(&p)?
        } else {
            fs::metadata(p)?.len()
        };
    }
    Ok(total)
}
fn write(a: &Args, name: &str, v: &Value) -> Result<()> {
    let bytes = serde_json::to_vec(v)?;
    let p = a.out.join(name);
    let previous = fs::metadata(&p).map_or(0, |m| m.len());
    if size(&a.out)? - previous + bytes.len() as u64 > a.maximum_report_bytes - 1_048_576 {
        return Err(bad("report cap"));
    }
    fs::write(p, bytes)?;
    Ok(())
}
fn deadline(a: &Args, start: Instant) -> Result<()> {
    let elapsed = start.elapsed().as_secs_f64();
    if elapsed >= a.maximum_seconds as f64 && !a.out.join("wall-estimate-overrun.json").exists() {
        // Owner rule (6 October): a self-set estimate must not terminate a
        // progressing run. Resource/lease extensions still need ledger visibility
        // from the controlling lab; this notice does not renew a GPU lease.
        write(
            a,
            "wall-estimate-overrun.json",
            &json!({
                "schema":"uor-r4.wall-estimate-overrun/1",
                "declared_estimate_seconds":a.maximum_seconds,
                "first_observed_elapsed_seconds":elapsed,
                "action":"continue; self-set estimate is not a cancellation rule",
                "operator_obligation":"record extension on owning issue and renew active lease; owner spending caps, disk floor and failed/diverged work remain boundaries",
                "preflight_projection_admission":"unchanged"
            }),
        )?;
        eprintln!("Wall estimate exceeded after {elapsed:.3}s; continuing under owner rule. Record extension and renew the active lease.");
    }
    Ok(())
}
fn seal_for(path: &Path) -> Result<PathBuf> {
    for p in path.ancestors() {
        if p.join("manifest.json").is_file() {
            return Ok(p.to_path_buf());
        }
    }
    Err(bad("input has no enclosing seal"))
}

fn args() -> Result<Args> {
    let mut it = std::env::args().skip(1);
    let path = it.next().ok_or_else(|| bad("one JSON config required"))?;
    if it.next().is_some() {
        return Err(bad("one JSON config only"));
    }
    let a: Args = serde_json::from_slice(&fs::read(path)?)?;
    control_settings(&a)?;
    if ![1001, 1002, 1003].contains(&a.seed)
        || a.maximum_seconds == 0
        || a.maximum_report_bytes < (64 << 20)
        || a.maximum_report_bytes > (2 << 30)
        || (a.baseline.is_some() && a.mode != Mode::Fit)
        // Declared doses stay schedule-comparable with the retained campaign
        // and land on a written checkpoint, because recovery checkpoints are
        // written every 32 updates and the final reload reads the last one.
        || a.updates == 0
        || a.updates > UPDATES
        || a.updates % 32 != 0
    {
        return Err(bad("fixed order seeds/resource admission"));
    }
    let output = output_support::prospective_output(&a.out)?;
    for path in [
        &a.checkpoint,
        &a.saved_fit,
        &a.categorical,
        &a.parent_config,
        &a.training_inputs,
        &a.training_labels,
        &a.development_inputs,
        &a.development_labels,
    ]
    .into_iter()
    .chain(a.baseline.iter())
    .chain(a.prediction_control_resume.iter())
    {
        let input = fs::canonicalize(path)?;
        if output.starts_with(&input) || input.starts_with(&output) {
            return Err(bad("output/input overlap"));
        }
        for ancestor in input.ancestors() {
            if ancestor.join("manifest.json").exists() && output.starts_with(ancestor) {
                return Err(bad("output beneath input seal"));
            }
        }
    }
    Ok(a)
}
#[cfg(feature = "cuda")]
fn cuda() -> Result<Device> {
    Ok(Device::new_cuda(0)?)
}
#[cfg(not(feature = "cuda"))]
fn cuda() -> Result<Device> {
    Err(bad("CUDA feature/device required; no CPU fallback"))
}
fn cue_payload(root: &Path) -> Result<CueAngularQ4> {
    let meta = read(&root.join("native-metadata.json"))?;
    let config: CueAngularConfig = serde_json::from_value(meta["potential"].clone())?;
    let packed = fs::read(root.join("cue-q4.bin"))?;
    let mut q = CueAngularQ4::new(config, &packed)?;
    if let Some(j) = meta.get("joint") {
        let m: CueJointMetadata = serde_json::from_value(j.clone())?;
        let joint = CueJointQ4::new(m.config, fs::read(root.join("cue-joint-q4.bin"))?)?;
        if joint.metadata() != m {
            return Err(bad("cue joint payload receipt"));
        }
        q = q.with_joint(joint)?;
    } else if root.join("cue-joint-q4.bin").exists() {
        return Err(bad("unreceipted cue joint"));
    }
    Ok(q)
}
fn prefix_payload(root: &Path) -> Result<PrefixAngularQ4> {
    let m = read(&root.join("native-metadata.json"))?;
    let c: PrefixAngularConfig = serde_json::from_value(m["potential"].clone())?;
    Ok(PrefixAngularQ4::new(
        c,
        &fs::read(root.join("prefix-q4.bin"))?,
    )?)
}
fn load_panel(
    inputs: &Path,
    labels: &Path,
    integer: &IntegerRealizer,
    tok: &ByteBpeTokenizer,
    legal: &BTreeSet<u32>,
    max: usize,
) -> Result<Vec<Episode>> {
    let i: Inputs = serde_json::from_slice(&fs::read(inputs)?)?;
    let l: Labels = serde_json::from_slice(&fs::read(labels)?)?;
    if i.schema != "uor-r4.native-source-bank-probe-input/1"
        || l.schema != "uor-r4.native-source-bank-labels/1"
        || l.protocol != "uor-r4.literal-role-dialogue/2"
        || !l.membership_only
        || i.cases.is_empty()
        || i.cases.len() > max
        || i.cases.len() != l.cases.len()
    {
        return Err(bad("complete panel/schema/count contract"));
    }
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for (p, l) in i.cases.into_iter().zip(l.cases) {
        l.answers.validate()?;
        if p.id.is_empty()
            || p.id != l.id
            || !seen.insert(p.id.clone())
            || !p.actual_prefix_ids.is_empty()
            || p.query_ids.is_empty()
            || p.segments.len() > 128
            || l.answers.accepted.is_empty()
        {
            return Err(bad("case identity or complete-answer prefix contract"));
        }
        let mut target = tok.encode(&l.answers.accepted[0]);
        if tok.decode_bytes(&target) != l.answers.accepted[0].as_bytes() {
            return Err(bad("canonical answer roundtrip"));
        }
        target.push(integer.binding().eos_token_id());
        if target.len() > 32 {
            return Err(bad("answer exceeds32 token budget"));
        }
        let views = p
            .segments
            .iter()
            .map(|s| {
                s.frame()
                    .map(|f| integer.compile_view(f.token_ids))
                    .transpose()
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let e = Episode {
            packet: p,
            answers: l.answers,
            target,
            views,
        };
        let mut base = e.packet.query_ids.len();
        for (s, v) in e.packet.segments.iter().zip(&e.views) {
            base += match (s, v) {
                (Segment::Source { .. }, Some(v)) => v.emitted_token_ids().len(),
                (Segment::Context { token_ids, .. }, _) => token_ids.len(),
                _ => return Err(bad("view/source mismatch")),
            };
        }
        if base + e.target.len() > 128 {
            return Err(bad("complete teacher-prefix context exceeds128"));
        } // Eligibility is public token membership, never model-quality filtering.
        let mut ids = e.packet.query_ids.clone();
        ids.extend_from_slice(&e.target);
        for (s, v) in e.packet.segments.iter().zip(&e.views) {
            match s {
                Segment::Source {
                    original_source_ids,
                    ..
                } => {
                    ids.extend_from_slice(original_source_ids);
                    ids.extend_from_slice(
                        v.as_ref().ok_or_else(|| bad("view"))?.emitted_token_ids(),
                    );
                }
                Segment::Context { token_ids, .. } => ids.extend_from_slice(token_ids),
            }
        }
        if ids.iter().any(|id| !legal.contains(id)) {
            return Err(bad("panel token outside actual public action binding"));
        }
        out.push(e);
    }
    Ok(out)
}

fn prefix_clone(q: &PrefixAngularQ4) -> Result<PrefixAngularQ4> {
    Ok(PrefixAngularQ4::new(q.config(), q.packed_coefficients())?)
}
fn restore(
    root: &Path,
    inventory: &Value,
    vars: &BTreeMap<String, Var>,
    device: &Device,
) -> Result<()> {
    let map = inventory
        .as_object()
        .ok_or_else(|| bad("master inventory absent"))?;
    if map.keys().collect::<BTreeSet<_>>() != vars.keys().collect::<BTreeSet<_>>() {
        return Err(bad("master parameter inventory differs"));
    }
    for (name, var) in vars {
        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_')
        {
            return Err(bad("unsafe master name"));
        }
        let m = &map[name];
        let shape: Vec<usize> = serde_json::from_value(m["shape"].clone())?;
        let bytes = fs::read(root.join(format!("{name}.f32le")))?;
        if shape != var.dims()
            || bytes.len()
                != var
                    .elem_count()
                    .checked_mul(4)
                    .ok_or_else(|| bad("master size overflow"))?
            || m["bytes"].as_u64() != Some(bytes.len() as u64)
            || m["sha256"].as_str() != Some(sha256_bytes(&bytes).as_str())
        {
            return Err(bad("master shape/hash/bytes differ"));
        }
        let values = bytes
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect::<Vec<_>>();
        if values
            .iter()
            .any(|x| !x.is_finite() || (!name.ends_with("prototype_choices") && x.abs() > 1.75))
        {
            return Err(bad("master finite/range admission"));
        }
        var.set(&Tensor::from_vec(values, shape.as_slice(), device)?)?;
    }
    Ok(())
}
fn identities(vars: &BTreeMap<String, Var>) -> Result<BTreeMap<String, String>> {
    vars.iter()
        .map(|(n, v)| {
            let b = v
                .flatten_all()?
                .to_vec1::<f32>()?
                .into_iter()
                .flat_map(f32::to_le_bytes)
                .collect::<Vec<_>>();
            Ok((n.clone(), sha256_bytes(&b)))
        })
        .collect()
}
fn earliest_raw_copy_max(scores: &[i64]) -> Result<usize> {
    let mut selected = 0;
    let mut best = *scores
        .first()
        .ok_or_else(|| bad("bridge has no Copy candidates"))?;
    for (i, &score) in scores.iter().enumerate().skip(1) {
        if score > best {
            selected = i;
            best = score;
        }
    }
    Ok(selected)
}
fn native_step(
    model: &IntegerRealizer,
    g: &NativeGeometricGenerate,
    bridge: Option<&NativeGeometricReadStateBridge>,
    pool: &mut NativeVocabularyActions,
    e: &Episode,
    actual: &[u32],
    cueq: &CueAngularQ4,
    prefixq: &PrefixAngularQ4,
) -> Result<Value> {
    use uor_r4_integer::h4_tables::H4Code;
    let (codes, copy_ids, copy_scores, provenance) = if e.has_source() {
        let cue = model.compile_cue_carrier(cueq.clone())?;
        let prefix = model.compile_prefix_transport(&cue, prefix_clone(prefixq)?)?;
        let trace = model.read_bank_with_prefix_transport(
            &e.segments()?,
            &e.packet.query_ids,
            actual,
            &cue,
            &prefix,
        )?;
        let b = &trace.cue_bank.bank;
        let mut codes = b
            .context
            .states
            .last()
            .ok_or_else(|| bad("bank final state absent"))?
            .iter()
            .copied()
            .map(H4Code::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let ids = b
            .candidates
            .iter()
            .map(|c| c.occurrence.token_id)
            .collect::<Vec<_>>();
        let mut scores = vec![0i64; ids.len()];
        for h in &b.heads {
            for (i, &s) in h.scores_q24.iter().enumerate() {
                scores[i] = scores[i]
                    .checked_add(s)
                    .ok_or_else(|| bad("Copy score overflow"))?;
            }
        }
        let bridge_trace = if let Some(bridge) = bridge {
            let selected = earliest_raw_copy_max(&scores)?;
            let candidate = &b.candidates[selected];
            let source = b
                .context
                .states
                .get(candidate.context_position)
                .ok_or_else(|| bad("bridge selected source state missing"))?
                .iter()
                .copied()
                .map(H4Code::try_from)
                .collect::<std::result::Result<Vec<_>, _>>()?;
            let mut distinct_source_states = BTreeSet::new();
            for candidate in &b.candidates {
                let state = b
                    .context
                    .states
                    .get(candidate.context_position)
                    .ok_or_else(|| bad("alternative source state missing"))?;
                distinct_source_states.insert(state.clone());
            }
            let query = codes.clone();
            let mut actions = vec![H4Code::IDENTITY; codes.len()];
            let mut action_scores = vec![0i64; codes.len() * 120];
            let mut counts = BridgeReadCounts::default();
            bridge.apply_into(
                &query,
                &source,
                &mut codes,
                &mut actions,
                &mut action_scores,
                &mut counts,
            )?;
            json!({"distinct_candidate_source_states":distinct_source_states.len(),"selected_ordinal":selected,"selected_candidate":candidate,"query_state_codes":query.iter().map(|c|c.index()).collect::<Vec<_>>(),"selected_source_state_codes":source.iter().map(|c|c.index()).collect::<Vec<_>>(),"action_codes":actions.iter().map(|c|c.index()).collect::<Vec<_>>(),"action_scores_q24":action_scores,"native_costs":counts,"payload_sha256":bridge.metadata().payload_sha256})
        } else {
            Value::Null
        };
        let mut components = BTreeMap::new();
        for (name, source) in [
            ("cue", &trace.cue_bank.carrier.copy_q24),
            ("prefix", &trace.prefix.copy_q24),
        ] {
            let mut totals = vec![0i64; ids.len()];
            if source.len() != b.heads.len() {
                return Err(bad("Copy component head count differs"));
            }
            for head in source {
                if head.len() != ids.len() {
                    return Err(bad("Copy component candidate count differs"));
                }
                for (j, &score) in head.iter().enumerate() {
                    totals[j] = totals[j]
                        .checked_add(score)
                        .ok_or_else(|| bad("Copy component overflow"))?;
                }
            }
            components.insert(name, totals);
        }
        let contextual = scores
            .iter()
            .enumerate()
            .map(|(j, &score)| {
                score
                    .checked_sub(components["cue"][j])
                    .and_then(|s| s.checked_sub(components["prefix"][j]))
                    .ok_or_else(|| bad("Copy contextual attribution overflow"))
            })
            .collect::<Result<Vec<_>>>()?;
        components.insert("contextual", contextual);
        let mut provenance = json!({"copy_components_q24":components,"candidates":b.candidates,"causal_tokens":b.context.tokens,"legacy_terminal_actions_discarded":true});
        if bridge.is_some() {
            provenance["read_state_bridge"] = bridge_trace;
        }
        (codes, ids, scores, provenance)
    } else {
        let ids = e.causal_no_source(actual)?;
        let cfg = model.context_config();
        let (tables, geometry) = model.context_encoder_parts();
        let mut st = NativeContextState::new(cfg.heads, cfg.lanes_per_head)?;
        for &id in &ids {
            st.step(id as usize, tables, geometry)?;
        }
        (
            st.states().to_vec(),
            Vec::new(),
            Vec::new(),
            json!({"causal_tokens":ids,"no_source":true}),
        )
    };
    let mut gs = vec![0i64; g.vocab_size()];
    let mut counts = GenerateReadCounts::default();
    g.score_into(&codes, &mut gs, &mut counts)?;
    let actions = pool.reduce_trace(&gs, &copy_ids, &copy_scores)?;
    Ok(
        json!({"actual_prefix_ids":actual,"retained_state_codes":codes.iter().map(|s|s.index()).collect::<Vec<_>>(),"copy_token_ids":copy_ids,"copy_raw_scores_q24":copy_scores,"generate_raw_scores_sha256":sha256_bytes(&serde_json::to_vec(&gs)?),"pool":{"summary":actions.summary,"token_masses":actions.token_masses.iter().map(|m|[m.token_id as u64,m.weight_q31,m.generate_weight_q31,m.copy_weight_q31]).collect::<Vec<_>>()},"source_provenance":provenance,"generate_costs":counts}),
    )
}
fn evaluate(
    a: &Args,
    name: &str,
    model: &IntegerRealizer,
    g: &NativeGeometricGenerate,
    bridge: Option<&NativeGeometricReadStateBridge>,
    exp: &[u8],
    eps: &[Episode],
    tok: &ByteBpeTokenizer,
    cue: &CueAngularQ4,
    prefix: &PrefixAngularQ4,
    start: Instant,
) -> Result<Value> {
    let mut pool = NativeVocabularyActions::new(model.binding().clone(), exp)?;
    let mut rows = Vec::new();
    let mut complete = 0;
    let mut ce = 0.;
    let mut tokens = 0;
    for e in eps {
        deadline(a, start)?;
        let mut canonical = Vec::new();
        let mut rowce = 0.;
        for (t, &target) in e.target.iter().enumerate() {
            let mut step =
                native_step(model, g, bridge, &mut pool, e, &e.target[..t], cue, prefix)?;
            let total = step["pool"]["summary"]["total_weight_q31"]
                .as_u64()
                .ok_or_else(|| bad("native denominator missing"))?;
            let mass = step["pool"]["token_masses"]
                .as_array()
                .ok_or_else(|| bad("native token masses"))?
                .iter()
                .find(|m| m[0].as_u64() == Some(target as u64))
                .and_then(|m| m[1].as_u64())
                .ok_or_else(|| bad("full Generate target mass missing"))?;
            if total == 0 || mass == 0 {
                return Err(bad("positive native support violated"));
            }
            step["pool"]
                .as_object_mut()
                .ok_or_else(|| bad("pool object"))?
                .remove("token_masses");
            let loss = -(mass as f64 / total as f64).ln();
            rowce += loss;
            canonical.push(json!({"target_label_only":target,"native_ce":loss,"native_target_mass":mass,"native_denominator":total,"native":step}));
            tokens += 1;
        }
        ce += rowce / e.target.len() as f64;
        let mut ids = Vec::new();
        let mut generated = Vec::new();
        let mut eos = false;
        let generation_allowance = 32usize.min(128usize.saturating_sub(e.base_len()));
        for _ in 0..generation_allowance {
            deadline(a, start)?;
            let mut step = native_step(model, g, bridge, &mut pool, e, &ids, cue, prefix)?;
            let id = step["pool"]["summary"]["chosen_token_id"]
                .as_u64()
                .ok_or_else(|| bad("native chosen ID"))? as u32;
            step["pool"]
                .as_object_mut()
                .ok_or_else(|| bad("pool object"))?
                .remove("token_masses");
            generated.push(step);
            ids.push(id);
            if id == model.binding().eos_token_id() {
                eos = true;
                break;
            }
        }
        let text = tok.decode(&ids[..ids.len() - usize::from(eos)]);
        let accepted = eos && e.answers.accepts(&text);
        complete += usize::from(accepted);
        let row = json!({"id":e.packet.id,"canonical_target_ids_labels_only":e.target,"native_equal_episode_ce":rowce/e.target.len()as f64,"canonical":canonical,"generation":generated,"generated_ids":ids,"decoded":text,"eos":eos,"generation_allowance":generation_allowance,"cutoff":if eos{None}else if generation_allowance<32{Some("context-cap")}else{Some("generation-cap")},"complete":accepted});
        let filename = format!("{name}-row-{:04}.json", rows.len());
        write(a, &filename, &row)?;
        rows.push(json!({"id":e.packet.id,"native_equal_episode_ce":rowce/e.target.len()as f64,"complete":accepted,"eos":eos,"generated_ids":ids,"row_file":filename,"row_sha256":sha256_file(&a.out.join(filename))?}));
    }
    let result = json!({"cases":eps.len(),"target_positions":tokens,"complete":complete,"native_equal_episode_ce":ce/eps.len()as f64,"rows":rows,"runtime":"integer context/Copy/Generate; one full-vocabulary pool; old terminals excluded; actual emitted feedback","selection_primary":"complete own-prefix answers","selection_tie":"native equal-episode CE; earliest checkpoint"});
    write(a, &format!("{name}.json"), &result)?;
    Ok(result)
}
const LOSS_PHASE_NAMES: [&str; 3] = ["entry", "later_copy_covered", "later_generate_only"];
fn loss_weight_policy(enabled: bool, scope: LossScope) -> &'static str {
    if !enabled {
        "legacy-equal-episode-token-mean/1"
    } else if scope == LossScope::EntryOnly {
        "entry-phase-only/1;entry-position0;later-phases-zero-weight;entry-weight-unchanged;empty-phase-zero;no-runtime-gate"
    } else {
        "equal-nonempty-phases-per-episode/1;entry-position0;later-exact-allsource-token-membership;empty-phase-zero;remaining-phases-equal;no-runtime-gate"
    }
}
struct EpisodeLossWeights {
    counts: [usize; 3],
    phases: Vec<usize>,
    weights: Vec<f64>,
}
/// Offline label weighting only, called after actual target-free action admission.
/// Candidate membership does not imply correct-source identity or force Copy.
/// Under `balanced` each nonempty phase receives the same total weight per
/// episode; `LossScope::EntryOnly` then zeroes every non-entry phase, leaving
/// the entry position's own balanced weight untouched.
fn episode_loss_weights(
    target: &[u32],
    admitted_copy_ids: &BTreeSet<u32>,
    batch_episodes: usize,
    balanced: bool,
    scope: LossScope,
) -> Result<EpisodeLossWeights> {
    if target.is_empty() || batch_episodes == 0 {
        return Err(bad("loss weighting requires nonempty episode and batch"));
    }
    let phases = target
        .iter()
        .enumerate()
        .map(|(t, id)| {
            if t == 0 {
                0
            } else if admitted_copy_ids.contains(id) {
                1
            } else {
                2
            }
        })
        .collect::<Vec<_>>();
    let mut counts = [0usize; 3];
    for &phase in &phases {
        counts[phase] += 1;
    }
    let nonempty = counts.iter().filter(|&&n| n != 0).count();
    let weights = phases
        .iter()
        .map(|&phase| {
            if !balanced {
                // Keep exact prior floating arithmetic and operation order.
                1. / target.len() as f64 / batch_episodes as f64
            } else if scope == LossScope::EntryOnly && phase != 0 {
                0.
            } else {
                1. / nonempty as f64 / counts[phase] as f64 / batch_episodes as f64
            }
        })
        .collect();
    Ok(EpisodeLossWeights {
        counts,
        phases,
        weights,
    })
}

fn apply(
    optimizer: &mut AdamW,
    params: &BTreeMap<String, Var>,
    gradients: &BTreeMap<String, Tensor>,
    denominator: &Tensor,
) -> Result<()> {
    // A zero scalar graph creates an empty GradStore on the same device. Only the
    // admitted parameter group is inserted; separate optimizers share one clip.
    let first = params
        .values()
        .next()
        .ok_or_else(|| bad("empty optimizer group"))?;
    let mut store = first.as_tensor().sum_all()?.affine(0., 0.)?.backward()?;
    for (name, var) in params {
        if let Some(g) = gradients.get(name) {
            store.insert(var.as_tensor(), g.broadcast_div(denominator)?.detach());
        }
    }
    optimizer.step(&store)?;
    Ok(())
}

struct Loaded {
    source: SourceRealizerWeights,
    frozen: NativeSourceRealizer,
    integer: IntegerRealizer,
    tokenizer: ByteBpeTokenizer,
    generate: GenerateLearningWeights,
    original_bridge: BridgeLearningWeights,
    marker: BridgeLearningWeights,
    cue: CueAngularQ4,
    prefix: PrefixAngularQ4,
    exp: Vec<u8>,
    receipt: Value,
    categorical_receipt: Value,
}
fn authenticate(a: &Args) -> Result<()> {
    report_output::verify(&a.saved_fit)?;
    report_output::verify(&a.categorical)?;
    for p in [
        &a.training_inputs,
        &a.training_labels,
        &a.development_inputs,
        &a.development_labels,
    ] {
        report_output::verify(&seal_for(p)?)?;
    }
    let fit = read(&a.saved_fit.join("report.json"))?;
    let receipt = read(&a.checkpoint.join("receipt.json"))?;
    if fit["schema"] != "uor-r4.geometric-bank-generate-fit/1"
        || fit["status"] != "COMPLETED"
        || fit["mode"] != "fit"
        || fit["arm"] != "joint-potential"
        || fit["updates"] != 128
        || fs::canonicalize(a.saved_fit.join("checkpoint-0128"))?
            != fs::canonicalize(&a.checkpoint)?
        || sha256_file(&a.checkpoint.join("receipt.json"))? != CP_RECEIPT_SHA
        || fit["stages"]
            .as_array()
            .and_then(|v| v.last())
            .map(|v| &v["checkpoint"])
            != Some(&receipt)
    {
        return Err(bad("exact retained final checkpoint admission"));
    }
    for (p, expected) in [
        (&a.training_inputs, INPUT_SHA),
        (&a.development_inputs, INPUT_SHA),
        (&a.training_labels, LABEL_SHA),
        (&a.development_labels, LABEL_SHA),
    ] {
        if sha256_file(p)? != expected {
            return Err(bad("fixed 512-row panel hash differs"));
        }
    }
    let parent = read(&a.parent_config)?;
    for (key, expected) in [
        ("updates", json!(128)),
        ("learning_rate", json!(0.003)),
        ("prototype_learning_rate", json!(0.01)),
        ("context_learning_rate", json!(0.002)),
        ("potential_learning_rate", json!(0.003)),
        ("token_backward_chunk", json!(1)),
        ("phase_balanced_token_loss", json!(true)),
        ("prefix_temporal_utility", json!(false)),
        ("arm", json!("joint-potential")),
    ] {
        if parent[key] != expected {
            return Err(bad("retained fit configuration differs"));
        }
    }
    // Parent-config absolute paths are historical. Data/artifact identities above,
    // not guessed replacement ancestry, determine admission.
    Ok(())
}
fn load(a: &Args, d: &Device) -> Result<Loaded> {
    authenticate(a)?;
    let cp = &a.checkpoint;
    let receipt = read(&cp.join("receipt.json"))?;
    let binding: NativeArtifactBinding = serde_json::from_value(receipt["parent"].clone())?;
    let integer = IntegerRealizer::load_native(&cp.join("native"), &binding)?;
    let tokbytes = fs::read(cp.join("native/tokenizer.json"))?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokbytes)
        .ok_or_else(|| bad("ByteBPE unavailable"))?;
    let source =
        SourceRealizerWeights::load_context_potential_on_device(&cp.join("source"), &tokbytes, d)?;
    let meta = read(&cp.join("native/metadata.json"))?;
    let identity: ConsumerIdentity = serde_json::from_value(meta["identity"].clone())?;
    let frozen = NativeSourceRealizer::load(&cp.join("native"), &source, &identity)?;
    if frozen.artifact_binding()? != binding {
        return Err(bad("source/native binding differs"));
    }
    let genbytes = fs::read(cp.join("generate.bin"))?;
    if sha256_bytes(&genbytes) != GENERATE_SHA {
        return Err(bad("Generate donor identity"));
    }
    let gn = NativeGeometricGenerate::from_bytes(&genbytes, integer.binding())?;
    let generate = GenerateLearningWeights::from_native(integer.binding().clone(), &gn, d)?;
    let gm = read(&cp.join("generate-source/metadata.json"))?;
    if gm["tokenizer_sha256"] != sha256_bytes(&tokbytes)
        || gm["protocol"] != serde_json::to_value(integer.binding().protocol())?
        || gm["lanes"] != json!(generate.lanes())
    {
        return Err(bad("Generate masters binding"));
    }
    restore(
        &cp.join("generate-source"),
        &gm["parameters"],
        &generate.parameters(),
        d,
    )?;
    if generate.export_native()?.to_bytes()? != genbytes {
        return Err(bad("restored Generate export differs"));
    }
    let bbytes = fs::read(cp.join("read-state-bridge.bin"))?;
    if sha256_bytes(&bbytes) != QUARTER_SHA {
        return Err(bad("quarter bridge donor identity"));
    }
    let bn = NativeGeometricReadStateBridge::from_bytes(&bbytes, integer.binding())?;
    let original_bridge = BridgeLearningWeights::from_native(&bn, integer.binding(), d)?;
    let bmbytes = fs::read(cp.join("read-state-bridge-source/metadata.json"))?;
    let bm: Value = serde_json::from_slice(&bmbytes)?;
    if bm["metadata"] != serde_json::to_value(bn.metadata())? || bm != receipt["read_state_bridge"]
    {
        return Err(bad("bridge donor master receipt differs"));
    }
    restore(
        &cp.join("read-state-bridge-source"),
        &bm["source_parameters"],
        &original_bridge.parameters(),
        d,
    )?;
    if original_bridge.export_native()?.to_bytes()? != bbytes {
        return Err(bad("original quarter export differs"));
    }
    let rebuilt = original_bridge.export_categorical_actions()?;
    let cb = fs::read(a.categorical.join("read-state-bridge-categorical.bin"))?;
    let probe = read(&a.categorical.join("probe.json"))?;
    if sha256_bytes(&cb) != CAT_SHA
        || rebuilt.native.to_bytes()? != cb
        || probe["status"] != "COMPLETED"
        || probe["mode"] != "categorical"
        || probe["schema"] != "uor-r4.read-state-export-probe/1"
        || probe["checkpoint_receipt_sha256"] != CP_RECEIPT_SHA
        || probe["parent_manifest_sha256"] != sha256_file(&a.saved_fit.join("manifest.json"))?
        || probe["master_metadata_sha256"] != sha256_bytes(&bmbytes)
        || probe["master_identities"] != bm["source_parameters"]
        || probe["original_native_sha256"] != QUARTER_SHA
        || probe["derived_native_sha256"] != CAT_SHA
        || probe["derived_payload_sha256"] != rebuilt.native.metadata().payload_sha256
        || probe["original_coarse_reproduction"] != "BYTE_IDENTICAL"
        || probe["parent_binding"] != receipt["parent"]
        || probe["all_query_frame_mismatches"] != 0
        || probe["all_query_frame_checks"] != json!(generate.lanes() * 120 * 120)
    {
        return Err(bad(
            "categorical artifact does not bind exact retained masters",
        ));
    }
    // This separate marker object supplies a truthful quarter graph for its
    // categorical native energies. Its Vars are never in an optimizer or clip.
    let marker = BridgeLearningWeights::from_native(&rebuilt.native, integer.binding(), d)?;
    if marker.prepare_native()?.native.to_bytes()? != cb {
        return Err(bad("frozen marker/native energy snapshot differs"));
    }
    let cue = cue_payload(&cp.join("cue"))?;
    let prefix = prefix_payload(&cp.join("prefix"))?;
    let cc = frozen.compile_cue_carrier(cue.clone())?;
    let pp = frozen.compile_prefix_transport(&cc, prefix_clone(&prefix)?)?;
    if serde_json::to_value(cc.metadata())? != read(&cp.join("cue/native-metadata.json"))?
        || serde_json::to_value(pp.metadata())? != read(&cp.join("prefix/native-metadata.json"))?
    {
        return Err(bad("donor cue/prefix receipts differ"));
    }
    let exp = fs::read(cp.join("native/consumer/exp-q31.bin"))?;
    Ok(Loaded {
        source,
        frozen,
        integer,
        tokenizer,
        generate,
        original_bridge,
        marker,
        cue,
        prefix,
        exp,
        receipt,
        categorical_receipt: serde_json::to_value(rebuilt.receipt)?,
    })
}
fn active(
    source: &SourceRealizerWeights,
    g: &GenerateLearningWeights,
) -> Result<BTreeMap<String, Var>> {
    let mut vars = g.parameters();
    vars.extend(source.context_state_parameters());
    vars.extend(source.potential_parameters());
    if vars.keys().any(|n| n.starts_with("read_state_bridge.")) {
        return Err(bad("frozen bridge admitted to optimizer"));
    }
    Ok(vars)
}
fn order(seed: u64, n: usize) -> Vec<usize> {
    // Fully specified SplitMix64/Fisher-Yates order only; no model reseeding.
    let mut state = seed;
    let mut out = (0..n).collect::<Vec<_>>();
    for i in (1..n).rev() {
        state = state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^= z >> 31;
        out.swap(i, (z % (i as u64 + 1)) as usize);
    }
    out
}
fn batch(
    a: &Args,
    l: &Loaded,
    eps: &[Episode],
    indices: &[usize],
    d: &Device,
    start: Instant,
    independent: Option<&IntegerRealizer>,
) -> Result<(BTreeMap<String, Tensor>, Value)> {
    let current = l.source.compile_context_potential_rebound(&l.frozen)?;
    let prepared = l.source.prepare_context_potential_on_device(&current, d)?;
    let cue = current.compile_cue_carrier(l.cue.clone())?;
    let prefix = current.compile_prefix_transport(&cue, prefix_clone(&l.prefix)?)?;
    let gs = l.generate.prepare_native()?;
    let bs = l.marker.prepare_native()?;
    if sha256_bytes(&bs.native.to_bytes()?) != CAT_SHA {
        return Err(bad("frozen categorical map changed"));
    }
    // Re-authenticate the same marker snapshot at every batch preparation.
    // Context/Potential/Generate may have changed; the frozen map may not.
    let categorical = if a.read_state_pullback == ReadStatePullback::Categorical {
        Some(PreparedCategoricalBridge::from_bytes(
            &bs.native.to_bytes()?,
            l.generate.binding(),
            CAT_SHA,
            d,
        )?)
    } else {
        None
    };
    let learner = PreparedBankGenerate::new(&prepared, &l.generate, &gs, &l.exp)?
        .with_prefix_temporal_utility(false);
    let mut learner = match a.read_state_pullback {
        ReadStatePullback::Legacy => learner.with_read_state_bridge(&l.marker, &bs)?,
        ReadStatePullback::Categorical => learner.with_categorical_read_state_bridge(
            categorical
                .as_ref()
                .ok_or_else(|| bad("categorical pullback snapshot absent"))?,
        )?,
    }
    .with_read_selector_credit(true);
    let params = active(&l.source, &l.generate)?;
    let mut sums = BTreeMap::<String, Tensor>::new();
    let mut rows = Vec::new();
    let mut total = 0.;
    let mut positions = 0;
    let mut phase_positions = [0usize; 3];
    let mut phase_losses = [0.; 3];
    let mut pool = NativeVocabularyActions::new(l.generate.binding().clone(), &l.exp)?;
    for &index in indices {
        let e = eps.get(index).ok_or_else(|| bad("batch index absent"))?;
        let expected_union = e
            .views
            .iter()
            .flatten()
            .flat_map(|v| v.emitted_token_ids().iter().copied())
            .collect::<BTreeSet<_>>();
        let mut plan = None;
        let mut ep_loss = 0.;
        for (t, &target) in e.target.iter().enumerate() {
            deadline(a, start)?;
            let out = if e.has_source() {
                learner.forward_bank(
                    &e.segments()?,
                    &e.packet.query_ids,
                    &e.target[..t],
                    &cue,
                    &prefix,
                )?
            } else {
                learner.forward_no_source(&e.causal_no_source(&e.target[..t])?)?
            };
            let union = out.copy_token_ids.iter().copied().collect::<BTreeSet<_>>();
            if union != expected_union {
                return Err(bad("actual all-source Copy union changed"));
            }
            if let Some(model) = independent {
                let expected = native_step(
                    model,
                    &gs.native,
                    Some(&bs.native),
                    &mut pool,
                    e,
                    &e.target[..t],
                    &l.cue,
                    &l.prefix,
                )?;
                let masses = out
                    .actions
                    .token_masses
                    .iter()
                    .map(|m| {
                        [
                            m.token_id as u64,
                            m.weight_q31,
                            m.generate_weight_q31,
                            m.copy_weight_q31,
                        ]
                    })
                    .collect::<Vec<_>>();
                if expected["pool"]["summary"] != serde_json::to_value(&out.actions.summary)?
                    || expected["pool"]["token_masses"] != json!(masses)
                    || expected["copy_token_ids"] != json!(out.copy_token_ids)
                    || expected["copy_raw_scores_q24"] != json!(out.copy_scores_q24)
                    || expected["retained_state_codes"]
                        != json!(out
                            .final_state_codes
                            .iter()
                            .map(|c| c.index())
                            .collect::<Vec<_>>())
                    || expected["generate_raw_scores_sha256"]
                        != sha256_bytes(&serde_json::to_vec(&out.generate.scores_q24)?)
                {
                    return Err(bad(
                        "independent native categorical bank/Generate/pool parity differs",
                    ));
                }
                let old = out
                    .loss_with_credit(target, VocabularyScoreAdjoint::Clipped)?
                    .to_scalar::<f32>()?;
                let raw = out
                    .loss_with_credit(target, VocabularyScoreAdjoint::RawIdentity)?
                    .to_scalar::<f32>()?;
                if old.to_bits() != raw.to_bits() || !old.is_finite() {
                    return Err(bad("matched credits have different forward losses"));
                }
                let (_, bridge) = out
                    .read_state_bridge
                    .as_ref()
                    .ok_or_else(|| bad("categorical bridge absent"))?;
                let nb = &expected["source_provenance"]["read_state_bridge"];
                if nb["selected_ordinal"] != json!(out.read_state_bridge.as_ref().map(|v| v.0))
                    || nb["action_codes"]
                        != json!(bridge
                            .action_codes
                            .iter()
                            .map(|c| c.index())
                            .collect::<Vec<_>>())
                    || nb["action_scores_q24"] != json!(bridge.action_scores_q24)
                {
                    return Err(bad(
                        "independent categorical selected/action parity differs",
                    ));
                }
            }
            // Labels enter only after the complete target-free native pool.
            if plan.is_none() {
                plan = Some(episode_loss_weights(
                    &e.target,
                    &union,
                    indices.len(),
                    true,
                    a.loss_scope,
                )?);
            }
            let plan = plan.as_ref().ok_or_else(|| bad("loss phases absent"))?;
            let weight = plan.weights[t];
            let phase = plan.phases[t];
            let mass = out
                .actions
                .token_masses
                .iter()
                .find(|m| m.token_id == target)
                .ok_or_else(|| bad("target support absent"))?
                .weight_q31;
            let nll = -(mass as f64 / out.actions.summary.total_weight_q31 as f64).ln();
            if !nll.is_finite() {
                return Err(bad("nonfinite native token objective"));
            }
            total += nll * weight;
            ep_loss += nll * weight;
            positions += 1;
            phase_positions[phase] += 1;
            phase_losses[phase] += nll * weight;
            let grads = out
                .loss_with_credit(target, a.credit.policy())?
                .affine(weight, 0.)?
                .backward()?;
            // One token graph at a time; no detached context carrier is inserted.
            for (name, var) in &params {
                if let Some(g) = grads.get(var.as_tensor()) {
                    if !g.device().same_device(d)
                        || !g
                            .abs()?
                            .flatten_all()?
                            .max(0)?
                            .to_scalar::<f32>()?
                            .is_finite()
                    {
                        return Err(bad("gradient device/finite admission"));
                    }
                    let sum = if let Some(old) = sums.get(name) {
                        old.add(g)?.detach()
                    } else {
                        g.detach()
                    };
                    sums.insert(name.clone(), sum);
                }
            }
        }
        rows.push(
            json!({"id":e.packet.id,"index":index,"weighted_native_loss":ep_loss,
            "phase_counts":plan.map(|p|p.counts)}),
        );
    }
    Ok((
        sums,
        json!({"indices":indices,"rows":rows,"positions":positions,"weighted_native_loss":total,
        "phase_positions":phase_positions,"phase_losses":phase_losses,
        "independent_native_parity":independent.is_some(),
        "matched_forward_loss_bit_equal":independent.is_some(),
        "credit":a.credit.name(),"read_state_pullback":a.read_state_pullback.name(),"token_backward_chunk":1,"frozen_bridge_excluded_from_gradient_accumulation":true}),
    ))
}
fn save_masters(root: &Path, vars: &BTreeMap<String, Var>) -> Result<Value> {
    fs::create_dir(root)?;
    let mut inventory = BTreeMap::new();
    for (name, var) in vars {
        let values = var.flatten_all()?.to_vec1::<f32>()?;
        if values.iter().any(|v| !v.is_finite()) {
            return Err(bad("nonfinite checkpoint master"));
        }
        let bytes = values
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect::<Vec<_>>();
        fs::write(root.join(format!("{name}.f32le")), &bytes)?;
        inventory.insert(
            name,
            json!({"shape":var.dims(),"bytes":bytes.len(),"sha256":sha256_bytes(&bytes)}),
        );
    }
    Ok(json!(inventory))
}
fn copy_directory(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            return Err(bad("unexpected donor master subdirectory"));
        }
        fs::copy(entry.path(), to.join(entry.file_name()))?;
    }
    Ok(())
}
fn checkpoint(
    a: &Args,
    step: usize,
    l: &Loaded,
) -> Result<(
    IntegerRealizer,
    NativeGeometricGenerate,
    NativeGeometricReadStateBridge,
    Value,
)> {
    let estimated = size(&a.checkpoint.join("source"))?
        + size(&a.checkpoint.join("native"))?
        + l.generate
            .parameters()
            .values()
            .map(|v| v.elem_count() as u64 * 4)
            .sum::<u64>()
        + 4_194_304;
    if size(&a.out)? + estimated > a.maximum_report_bytes - 1_048_576 {
        return Err(bad("checkpoint storage admission"));
    }
    let root = a.out.join(format!("checkpoint-{step:04}"));
    fs::create_dir(&root)?;
    let native = l.source.compile_context_potential_rebound(&l.frozen)?;
    l.source.save_source(&root.join("source"))?;
    native.save(&root.join("native"))?;
    let binding = native.execution_binding()?;
    let integer = IntegerRealizer::load_native(&root.join("native"), &binding)?;
    let tokbytes = fs::read(root.join("native/tokenizer.json"))?;
    let disk_source = SourceRealizerWeights::load_context_potential_on_device(
        &root.join("source"),
        &tokbytes,
        &Device::Cpu,
    )?;
    let disk_native = NativeSourceRealizer::load(
        &root.join("native"),
        &disk_source,
        &serde_json::from_value::<ConsumerIdentity>(
            read(&root.join("native/metadata.json"))?["identity"].clone(),
        )?,
    )?;
    if disk_native.artifact_binding()? != binding {
        return Err(bad("independent source/native master reload differs"));
    }
    let gm = save_masters(&root.join("generate-source"), &l.generate.parameters())?;
    fs::write(
        root.join("generate-source/metadata.json"),
        serde_json::to_vec_pretty(&json!({
        "parameters":gm,"tokenizer_sha256":integer.binding().tokenizer_sha256(),
        "protocol":integer.binding().protocol(),"lanes":l.generate.lanes(),"order_seed":a.seed}))?,
    )?;
    let genbytes = l.generate.export_native()?.to_bytes()?;
    fs::write(root.join("generate.bin"), &genbytes)?;
    let generate = NativeGeometricGenerate::from_bytes(
        &fs::read(root.join("generate.bin"))?,
        integer.binding(),
    )?;
    let disk_gen =
        GenerateLearningWeights::from_native(integer.binding().clone(), &generate, &Device::Cpu)?;
    restore(
        &root.join("generate-source"),
        &gm,
        &disk_gen.parameters(),
        &Device::Cpu,
    )?;
    if disk_gen.export_native()?.to_bytes()? != genbytes {
        return Err(bad("saved Generate masters/native reload differs"));
    }
    let quarter = l.original_bridge.export_native()?.to_bytes()?;
    if sha256_bytes(&quarter) != QUARTER_SHA {
        return Err(bad("original quarter bridge changed"));
    }
    fs::write(root.join("read-state-bridge.bin"), &quarter)?;
    copy_directory(
        &a.checkpoint.join("read-state-bridge-source"),
        &root.join("read-state-bridge-source"),
    )?;
    let cat = l.marker.export_native()?.to_bytes()?;
    if sha256_bytes(&cat) != CAT_SHA {
        return Err(bad("categorical frozen map changed"));
    }
    fs::write(root.join("read-state-bridge-categorical.bin"), &cat)?;
    let bridge = NativeGeometricReadStateBridge::from_bytes(
        &fs::read(root.join("read-state-bridge-categorical.bin"))?,
        integer.binding(),
    )?;
    let rebind = NativeGeometricReadStateBridge::compile(
        integer.binding(),
        bridge.lanes(),
        bridge.packed_bias(),
        bridge.packed_relative(),
    )?;
    if rebind.to_bytes()? != cat {
        return Err(bad(
            "source action binding would change frozen categorical artifact",
        ));
    }
    let original_reload = NativeGeometricReadStateBridge::from_bytes(&quarter, integer.binding())?;
    let original_masters =
        BridgeLearningWeights::from_native(&original_reload, integer.binding(), &Device::Cpu)?;
    let bm = read(&root.join("read-state-bridge-source/metadata.json"))?;
    restore(
        &root.join("read-state-bridge-source"),
        &bm["source_parameters"],
        &original_masters.parameters(),
        &Device::Cpu,
    )?;
    if original_masters.export_native()?.to_bytes()? != quarter
        || original_masters
            .export_categorical_actions()?
            .native
            .to_bytes()?
            != cat
    {
        return Err(bad("independently restored frozen original/map differs"));
    }
    let cue = integer.compile_cue_carrier(l.cue.clone())?;
    let prefix = integer.compile_prefix_transport(&cue, prefix_clone(&l.prefix)?)?;
    fs::create_dir(root.join("cue"))?;
    fs::create_dir(root.join("prefix"))?;
    fs::write(
        root.join("cue/native-metadata.json"),
        serde_json::to_vec_pretty(cue.metadata())?,
    )?;
    fs::write(root.join("cue/cue-q4.bin"), l.cue.packed_coefficients())?;
    if let Some(j) = l.cue.joint() {
        fs::write(root.join("cue/cue-joint-q4.bin"), j.packed_coefficients())?;
    }
    fs::write(
        root.join("prefix/native-metadata.json"),
        serde_json::to_vec_pretty(prefix.metadata())?,
    )?;
    fs::write(
        root.join("prefix/prefix-q4.bin"),
        l.prefix.packed_coefficients(),
    )?;
    let dc = integer.compile_cue_carrier(cue_payload(&root.join("cue"))?)?;
    let dp = integer.compile_prefix_transport(&dc, prefix_payload(&root.join("prefix"))?)?;
    if serde_json::to_value(dc.metadata())? != read(&root.join("cue/native-metadata.json"))?
        || serde_json::to_value(dp.metadata())? != read(&root.join("prefix/native-metadata.json"))?
        || fs::read(root.join("native/consumer/exp-q31.bin"))? != l.exp
    {
        return Err(bad("frozen sidecar/exp payload reload differs"));
    }
    let old_binding = &l.receipt["parent"];
    let receipt = json!({"step":step,"parent":binding,"original_parent":old_binding,
        "source_metadata_rebound":serde_json::to_value(&binding)?!=*old_binding,
        "binding_scope":"source context/potential metadata honestly rebound; typed action bindings checked against current artifact; unchanged bridge coefficients, original masters and map bytes independently verified",
        "generate_sha256":sha256_bytes(&genbytes),"original_quarter_sha256":QUARTER_SHA,"categorical_sha256":CAT_SHA,
        "categorical_receipt":l.categorical_receipt,"credit":a.credit.name(),"read_state_pullback":a.read_state_pullback.name(),"order_seed":a.seed,
        "native_independently_reloaded":true,"masters_independently_reloaded":true,
        "fresh_adam":"moments zero-initialized; not historical optimizer continuation",
        "frozen_bridge_training":"marker parameters excluded from Adam and clip; original masters unchanged"});
    fs::write(
        root.join("receipt.json"),
        serde_json::to_vec_pretty(&receipt)?,
    )?;
    Ok((integer, generate, bridge, receipt))
}

fn scoped_pairs(eps: &[Episode]) -> Result<Vec<(usize, usize, usize)>> {
    let ids = eps
        .iter()
        .enumerate()
        .map(|(i, e)| (e.packet.id.clone(), i))
        .collect::<BTreeMap<_, _>>();
    let mut covered = BTreeSet::new();
    let mut pairs = Vec::new();
    for (i, e) in eps.iter().enumerate() {
        if e.packet.id.contains("-swap0-") {
            let other = e.packet.id.replace("-swap0-", "-swap1-");
            let j = *ids
                .get(&other)
                .ok_or_else(|| bad("paired source-swap row absent"))?;
            let f = &eps[j];
            let pos = e
                .target
                .iter()
                .zip(&f.target)
                .position(|(a, b)| a != b)
                .ok_or_else(|| bad("source-swap targets do not diverge"))?;
            if pos == 0
                || e.packet.query_ids != f.packet.query_ids
                || !covered.insert(i)
                || !covered.insert(j)
            {
                return Err(bad("source-swap pairing/query/prefix admission"));
            }
            pairs.push((i, j, pos));
        }
    }
    if pairs.len() * 2 != eps.len() || covered.len() != eps.len() {
        return Err(bad("complete source-swap pair coverage required"));
    }
    Ok(pairs)
}
fn pairs(eps: &[Episode]) -> Result<Vec<(usize, usize, usize)>> {
    let p = scoped_pairs(eps)?;
    if p.len() != 256 {
        return Err(bad("complete256 source-swap pairs required"));
    }
    Ok(p)
}
fn metrics(a: &Args, eval: &Value, eps: &[Episode]) -> Result<Value> {
    let refs = eval["rows"]
        .as_array()
        .ok_or_else(|| bad("evaluation rows missing"))?;
    if refs.len() != eps.len() {
        return Err(bad("metric row coverage differs"));
    }
    let mut rows = Vec::new();
    let mut entry_teacher = 0;
    let mut entry_own = 0;
    let mut copy_total = 0;
    let mut copy_correct = 0;
    for (r, e) in refs.iter().zip(eps) {
        if r["id"] != e.packet.id {
            return Err(bad("metric row IDs differ"));
        }
        let name = r["row_file"]
            .as_str()
            .ok_or_else(|| bad("metric row filename absent"))?;
        let path = a.out.join(name);
        if sha256_file(&path)? != r["row_sha256"] {
            return Err(bad("metric row hash differs"));
        }
        let row = read(&path)?;
        let canonical = row["canonical"]
            .as_array()
            .ok_or_else(|| bad("canonical steps absent"))?;
        let generated: Vec<u32> = serde_json::from_value(row["generated_ids"].clone())?;
        if canonical.len() != e.target.len() {
            return Err(bad("complete canonical metric coverage"));
        }
        let chosen = |t: usize| -> Result<u32> {
            Ok(canonical[t]["native"]["pool"]["summary"]["chosen_token_id"]
                .as_u64()
                .ok_or_else(|| bad("canonical chosen ID absent"))? as u32)
        };
        entry_teacher += usize::from(chosen(0)? == e.target[0]);
        entry_own += usize::from(generated.first() == e.target.first());
        for (t, &target) in e.target.iter().enumerate().skip(1) {
            let copy: Vec<u32> =
                serde_json::from_value(canonical[t]["native"]["copy_token_ids"].clone())?;
            if copy.contains(&target) {
                copy_total += 1;
                copy_correct += usize::from(chosen(t)? == target);
            }
        }
        rows.push((canonical.clone(), generated));
    }
    let mut teacher_count = 0;
    let mut own_count = 0;
    let mut reached_count = 0;
    let mut teacher_both = 0;
    let mut own_both = 0;
    let mut receipts = Vec::new();
    let paired = if a.mode == Mode::PredictionControl {
        scoped_pairs(eps)?
    } else {
        pairs(eps)?
    };
    for (i, j, pos) in paired {
        let mut arms = Vec::new();
        for k in [i, j] {
            let target = eps[k].target[pos];
            let canonical = &rows[k].0;
            let generated = &rows[k].1;
            let chosen = canonical[pos]["native"]["pool"]["summary"]["chosen_token_id"]
                .as_u64()
                .ok_or_else(|| bad("source-word chosen ID absent"))?
                as u32;
            let teacher = chosen == target;
            let own = generated.get(pos) == Some(&target);
            let reached = generated.len() > pos && generated[..pos] == eps[k].target[..pos];
            teacher_count += usize::from(teacher);
            own_count += usize::from(own);
            reached_count += usize::from(reached);
            arms.push(
                json!({"index":k,"id":eps[k].packet.id,"position":pos,"target":target,
                "teacher_prefix_correct":teacher,"own_position_correct":own,
                "own_prefix_reached_correctly":reached,
                "target_mass":canonical[pos]["native_target_mass"],
                "denominator":canonical[pos]["native_denominator"],
                "chosen_token":chosen}),
            );
        }
        teacher_both += usize::from(arms.iter().all(|v| v["teacher_prefix_correct"] == true));
        own_both += usize::from(arms.iter().all(|v| v["own_position_correct"] == true));
        receipts.push(json!({"indices":[i,j],"first_divergent_position":pos,"arms":arms}));
    }
    Ok(
        json!({"rows":eps.len(),"missing_rows":0,"entry_teacher_correct":entry_teacher,"entry_own_correct":entry_own,
        "first_divergent_teacher_correct":teacher_count,"first_divergent_own_position_correct":own_count,
        "first_divergent_own_prefix_reached":reached_count,"paired_teacher_both_correct":teacher_both,
        "paired_own_position_both_correct":own_both,"pairs":receipts,
        "later_copy_covered_positions":copy_total,"later_copy_covered_teacher_correct":copy_correct,
        "scope":"complete native own-prefix outputs and separately canonical teacher-prefix metrics; own position matches without a correct preceding prefix are not successful source-dependent continuation"}),
    )
}

// f63's native_step/evaluate semantics are byte-identical here. This explicit
// compatibility allowance must be revisited if initialized native evaluation
// semantics change; a matching parameter count alone never permits reuse.
const BASELINE_PRODUCER: &str = "f63ed81fd4f74d2501ba3d9888420662492416df";
fn baseline_identity(base: &Value, current: &Value, before: &Value, now: &Value) -> Result<()> {
    let producer = base["source_commit"]
        .as_str()
        .ok_or_else(|| bad("baseline producer absent"))?;
    if producer != BASELINE_PRODUCER && Some(producer) != option_env!("UOR_BUILD_SOURCE_COMMIT") {
        return Err(bad(
            "baseline producer has no explicit native-evaluation compatibility",
        ));
    }
    for field in [
        "checkpoint_receipt_sha256",
        "parent_config_sha256",
        "training_input_sha256",
        "training_labels_sha256",
        "original_bridge_masters",
        "marker_masters",
        "categorical",
        "initial_active_masters",
        "active_parameter_names",
        "fresh_adam",
        "rates",
        "phase_policy",
    ] {
        if base.get(field).is_none() || base[field] != current[field] {
            return Err(bad(&format!("baseline initial identity differs: {field}")));
        }
    }
    for field in [
        "step",
        "parent",
        "original_parent",
        "source_metadata_rebound",
        "generate_sha256",
        "original_quarter_sha256",
        "categorical_sha256",
        "categorical_receipt",
        "native_independently_reloaded",
        "masters_independently_reloaded",
    ] {
        if before.get(field).is_none() || before[field] != now[field] {
            return Err(bad(&format!(
                "baseline checkpoint identity differs: {field}"
            )));
        }
    }
    if before["step"] != 0
        || before["native_independently_reloaded"] != true
        || before["masters_independently_reloaded"] != true
        || base["fresh_adam"] != true
    {
        return Err(bad("baseline initialization/reload admission"));
    }
    // Credit, read-state pullback, data-order seed, host and CUDA device are
    // not initialized integer evaluation inputs. Do not require the new
    // non-forward pullback field from an older authenticated baseline.
    // Each fit still executes fresh seed/arm-specific graph/native parity.
    Ok(())
}
fn tree_identity(root: &Path) -> Result<String> {
    fn walk(root: &Path, path: &Path, rows: &mut Vec<(String, u64, String)>) -> Result<()> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            if kind.is_dir() {
                walk(root, &entry.path(), rows)?;
            } else if kind.is_file() {
                rows.push((
                    entry.path().strip_prefix(root)?.display().to_string(),
                    entry.metadata()?.len(),
                    sha256_file(&entry.path())?,
                ));
            } else {
                return Err(bad("baseline artifact has a nonregular entry"));
            }
        }
        Ok(())
    }
    let mut rows = Vec::new();
    walk(root, root, &mut rows)?;
    rows.sort();
    Ok(sha256_bytes(&serde_json::to_vec(&rows)?))
}
fn reuse_baseline(
    a: &Args,
    root: &Path,
    dev: &[Episode],
    current_receipt: &Value,
) -> Result<(Value, Value)> {
    report_output::verify(root)?;
    let report = read(&root.join("report.json"))?;
    let admission = read(&root.join("admission.json"))?;
    let zero = read(&root.join("zero-update-admission.json"))?;
    if report["schema"] != "uor-r4.geometric-frozen-map-fit/1"
        || report["status"] != "COMPLETED"
        || report["mode"] != "admission"
        || report["updates"] != 0
        || report["admission"] != zero
        || zero["independent_native_parity"] != true
        || zero["matched_forward_loss_bit_equal"] != true
    {
        return Err(bad(
            "baseline must be a completed sealed zero-update native admission",
        ));
    }
    let current = read(&a.out.join("admission.json"))?;
    let previous_receipt = read(&root.join("checkpoint-0000/receipt.json"))?;
    baseline_identity(&admission, &current, &previous_receipt, current_receipt)?;
    let mut artifact_receipts = BTreeMap::new();
    for name in [
        "source",
        "native",
        "read-state-bridge-source",
        "cue",
        "prefix",
    ] {
        let expected = tree_identity(&root.join("checkpoint-0000").join(name))?;
        if expected != tree_identity(&a.out.join("checkpoint-0000").join(name))? {
            return Err(bad(
                "baseline initialized source/native/frozen master bytes differ",
            ));
        }
        artifact_receipts.insert(name, expected);
    }
    for name in [
        "generate.bin",
        "read-state-bridge.bin",
        "read-state-bridge-categorical.bin",
    ] {
        if sha256_file(&root.join("checkpoint-0000").join(name))?
            != sha256_file(&a.out.join("checkpoint-0000").join(name))?
        {
            return Err(bad("baseline initialized native decoder/map bytes differ"));
        }
    }
    let evaluation = read(&root.join("development-0000.json"))?;
    let metric = read(&root.join("metrics-0000.json"))?;
    if report["initial_metrics"] != metric
        || evaluation["cases"] != 512
        || evaluation["target_positions"] != 6664
        || dev.len() != 512
    {
        return Err(bad("baseline complete evaluation/metric scope differs"));
    }
    let refs = evaluation["rows"]
        .as_array()
        .ok_or_else(|| bad("baseline rows absent"))?;
    if refs.len() != dev.len() {
        return Err(bad("baseline missing row references"));
    }
    for (index, (reference, e)) in refs.iter().zip(dev).enumerate() {
        let name = format!("development-0000-row-{index:04}.json");
        if reference["id"] != e.packet.id || reference["row_file"] != name {
            return Err(bad("baseline ordered row ID/file differs"));
        }
        let path = root.join(&name);
        let bytes = fs::read(&path)?;
        if reference["row_sha256"] != sha256_bytes(&bytes) {
            return Err(bad("baseline row hash differs"));
        }
        let row: Value = serde_json::from_slice(&bytes)?;
        if row["id"] != e.packet.id
            || row["canonical_target_ids_labels_only"] != json!(e.target)
            || row["generated_ids"] != reference["generated_ids"]
            || row["complete"] != reference["complete"]
        {
            return Err(bad("baseline row label/output reference differs"));
        }
        let canonical = row["canonical"]
            .as_array()
            .ok_or_else(|| bad("baseline canonical rows absent"))?;
        if canonical.len() != e.target.len()
            || canonical
                .iter()
                .zip(&e.target)
                .any(|(step, target)| step["target_label_only"] != *target)
        {
            return Err(bad("baseline canonical target coverage differs"));
        }
        if size(&a.out)? + bytes.len() as u64 > a.maximum_report_bytes - 1_048_576 {
            return Err(bad("baseline evidence copy exceeds report cap"));
        }
        // Keep the fit report self-contained. Never write into the sealed source.
        use std::io::Write;
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(a.out.join(name))?
            .write_all(&bytes)?;
    }
    let reproduced = metrics(a, &evaluation, dev)?;
    if reproduced != metric {
        return Err(bad(
            "baseline metrics do not reproduce from complete copied rows",
        ));
    }
    write(a, "development-0000.json", &evaluation)?;
    let provenance = json!({"reused":true,"execution":"NOT_RUN in this fit; copied authenticated initial native evaluation",
        "producer_source_commit":admission["source_commit"],"compatible_consumer_source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),
        "baseline_root":root,"baseline_manifest_sha256":sha256_file(&root.join("manifest.json"))?,
        "baseline_report_sha256":sha256_file(&root.join("report.json"))?,
        "baseline_evaluation_sha256":sha256_file(&root.join("development-0000.json"))?,
        "baseline_metrics_sha256":sha256_file(&root.join("metrics-0000.json"))?,
        "initialized_artifact_tree_identities":artifact_receipts,"rows_verified_and_copied":refs.len(),
        "producer_elapsed_seconds":report["elapsed_seconds"],
        "compatibility_scope":"same initialized integer source/native/decoder/categorical map, full input/labels and master identities; native_step/evaluate unchanged; order/credit do not enter native inference; each fit executes its own graph/native admission"});
    write(a, "baseline-reuse.json", &provenance)?;
    Ok((evaluation, provenance))
}

fn optimizer(vars: &BTreeMap<String, Var>, lr: f64) -> Result<AdamW> {
    Ok(AdamW::new(
        vars.values().cloned().collect(),
        ParamsAdamW {
            lr,
            beta1: 0.9,
            beta2: 0.999,
            eps: 1e-8,
            weight_decay: 0.,
        },
    )?)
}
fn clip_denominator(grads: &BTreeMap<String, Tensor>, d: &Device) -> Result<(Tensor, f64)> {
    if grads.is_empty() {
        return Err(bad("no active parameter gradients"));
    }
    let mut scale = 0f64;
    for g in grads.values() {
        let value = g.abs()?.flatten_all()?.max(0)?.to_scalar::<f32>()? as f64;
        if !value.is_finite() {
            return Err(bad("nonfinite accumulated gradient"));
        }
        scale = scale.max(value);
    }
    let norm = if scale == 0. {
        0.
    } else {
        let terms = grads
            .values()
            .map(|g| g.affine(1. / scale, 0.)?.sqr()?.sum_all())
            .collect::<candle_core::Result<Vec<_>>>()?;
        scale * (Tensor::stack(&terms, 0)?.sum_all()?.to_scalar::<f32>()? as f64).sqrt()
    };
    if !norm.is_finite() {
        return Err(bad("nonfinite global gradient norm"));
    }
    let denominator = norm.max(1.) as f32;
    if !denominator.is_finite() {
        return Err(bad("gradient clip denominator exceeds F32"));
    }
    Ok((Tensor::new(denominator, d)?, norm))
}
fn disk_floor(a: &Args) -> Result<()> {
    let output = std::process::Command::new("df")
        .arg("-Pk")
        .arg(&a.out)
        .output()?;
    if !output.status.success() {
        return Err(bad("disk-floor observation unavailable"));
    }
    let text = std::str::from_utf8(&output.stdout)?;
    let available = text
        .lines()
        .last()
        .and_then(|l| l.split_whitespace().nth(3))
        .and_then(|n| n.parse::<u64>().ok())
        .ok_or_else(|| bad("disk-floor parse"))?;
    if available < 128 * 1024 {
        return Err(bad("128MiB disk stop margin reached"));
    }
    Ok(())
}
fn control_panel_admit(eps: &[Episode]) -> Result<Value> {
    if eps.len() != 8 {
        return Err(bad("prediction control requires predetermined eight cases"));
    }
    let p = scoped_pairs(eps)?;
    if p != vec![(0, 4, 3), (1, 5, 3), (2, 6, 3), (3, 7, 3)] {
        return Err(bad("control source-swap pairs/common prefix differ"));
    }
    let mut entries = BTreeSet::new();
    let mut phases = [0usize; 3];
    let mut rows = Vec::new();
    for (local, e) in eps.iter().enumerate() {
        if !e.has_source() || !e.packet.actual_prefix_ids.is_empty() {
            return Err(bad(
                "control requires authentic full Source bank and empty entry prefix",
            ));
        }
        entries.insert(e.target[0]);
        let ids = e
            .views
            .iter()
            .flatten()
            .flat_map(|v| v.emitted_token_ids().iter().copied())
            .collect::<Vec<_>>();
        if ids.len() != 11 || ids.contains(&e.target[0]) {
            return Err(bad(
                "control full eleven occurrences/Generate-only entry differ",
            ));
        }
        // Only the phase classification is consumed here, and the loss scope
        // changes weights rather than counts, so this admission receipt is
        // scope-independent by construction.
        let plan = episode_loss_weights(
            &e.target,
            &ids.iter().copied().collect(),
            8,
            true,
            LossScope::All,
        )?;
        for k in 0..3 {
            phases[k] += plan.counts[k];
        }
        rows.push(json!({"local_index":local,"original_index":CONTROL_INDICES[local],"id":e.packet.id,
            "target_ids_labels_only":e.target,"physical_copy_occurrences":ids.len(),"phase_counts":plan.counts}));
    }
    if entries != BTreeSet::from([617, 2997]) || phases != [8, 44, 32] {
        return Err(bad("control entry roles/phase coverage differ"));
    }
    Ok(
        json!({"original_indices":CONTROL_INDICES,"local_pairs":p,"phase_counts":phases,"rows":rows,
        "scope":"eight construction cases; four swapped-source pairs, two entry roles; no held-out or temporal-update claim"}),
    )
}

fn control_entry_diagnostics(
    a: &Args,
    l: &Loaded,
    model: &IntegerRealizer,
    g: &NativeGeometricGenerate,
    bridge: &NativeGeometricReadStateBridge,
    eps: &[Episode],
    step: usize,
) -> Result<Value> {
    use uor_r4_integer::h4_tables::H4Code;
    let mut pool = NativeVocabularyActions::new(model.binding().clone(), &l.exp)?;
    let mut only = NativeVocabularyActions::new(model.binding().clone(), &l.exp)?;
    let params = l.generate.parameters();
    let biases = params
        .get("generate.bias")
        .ok_or_else(|| bad("control bias masters absent"))?;
    let mut rows = Vec::new();
    for (local, e) in eps.iter().enumerate() {
        // Complete target-free native bank selection first. Labels cannot choose
        // a source occurrence, frame, route, state or vocabulary candidate.
        let native = native_step(model, g, Some(bridge), &mut pool, e, &[], &l.cue, &l.prefix)?;
        let state: Vec<u8> = serde_json::from_value(native["retained_state_codes"].clone())?;
        let state = state
            .into_iter()
            .map(H4Code::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let mut scores = vec![0i64; g.vocab_size()];
        g.score_into(&state, &mut scores, &mut GenerateReadCounts::default())?;
        if native["generate_raw_scores_sha256"] != sha256_bytes(&serde_json::to_vec(&scores)?) {
            return Err(bad("entry diagnostic native score digest differs"));
        }
        let ids: Vec<u32> = serde_json::from_value(native["copy_token_ids"].clone())?;
        let copies: Vec<i64> = serde_json::from_value(native["copy_raw_scores_q24"].clone())?;
        let full = pool.reduce_trace(&scores, &ids, &copies)?;
        if native["pool"]["summary"] != serde_json::to_value(&full.summary)? {
            return Err(bad("entry diagnostic full pool differs"));
        }
        let generate_only = only.reduce_trace(&scores, &[], &[])?;
        let gold = e.target[0];
        let gold_score = *scores
            .get(gold as usize)
            .ok_or_else(|| bad("entry gold outside vocabulary"))?;
        let gm = full
            .token_masses
            .iter()
            .find(|m| m.token_id == gold)
            .ok_or_else(|| bad("entry native gold mass absent"))?;
        let wrong = full
            .actions
            .iter()
            .filter(|x| {
                matches!(
                    x.action,
                    uor_r4_integer::geometric_vocabulary_actions::VocabularyAction::Copy { .. }
                ) && x.token_id != gold
            })
            .max_by(|x, y| {
                x.raw_score_q24
                    .cmp(&y.raw_score_q24)
                    .then_with(|| y.action_offset.cmp(&x.action_offset))
            })
            .ok_or_else(|| bad("entry wrong Copy absent"))?;
        let wrong_alias = full
            .token_masses
            .iter()
            .filter(|m| m.token_id != gold && m.copy_weight_q31 > 0)
            .max_by(|x, y| {
                x.copy_weight_q31
                    .cmp(&y.copy_weight_q31)
                    .then_with(|| y.token_id.cmp(&x.token_id))
            })
            .ok_or_else(|| bad("entry wrong Copy alias absent"))?;
        let bias = biases
            .as_tensor()
            .narrow(0, gold as usize, 1)?
            .reshape(())?
            .to_scalar::<f32>()?;
        let code = g.token_bias(gold as usize)?;
        let next_up = if code < 7 {
            Some((f64::from(code) + 0.5) / 4.)
        } else {
            None
        };
        let next_down = if code > -7 {
            Some((f64::from(code) - 0.5) / 4.)
        } else {
            None
        };
        rows.push(json!({"local_index":local,"original_index":CONTROL_INDICES[local],"id":e.packet.id,
            "gold_entry_label_only":gold,"generate_only_summary":generate_only.summary,"full_summary":full.summary,
            "gold_generate_raw_q24":gold_score,"gold_generate_clipped_q24":gold_score.clamp(-(8<<24),8<<24),
            "gold_generate_weight_q31":gm.generate_weight_q31,"gold_copy_weight_q31":gm.copy_weight_q31,"gold_total_weight_q31":gm.weight_q31,
            "highest_raw_wrong_copy_action":wrong,"highest_copy_component_wrong_token_alias":wrong_alias,
            "raw_gold_minus_wrong_copy_q24":gold_score.checked_sub(wrong.raw_score_q24),
            "clipped_gold_minus_wrong_copy_q24":gold_score.clamp(-(8<<24),8<<24)-wrong.score_q24,
            "score_clip_q24":8i64<<24,"gold_bias_master":bias,"gold_bias_native_code":code,
            "bias_positive_cell_boundary":next_up,"bias_positive_boundary_distance":next_up.map(|x|x-f64::from(bias)),
            "bias_negative_cell_boundary":next_down,"bias_negative_boundary_distance":next_down.map(|x|f64::from(bias)-x),
            "native":native}));
    }
    let v = json!({"step":step,"rows":rows,"generate_sha256":sha256_bytes(&g.to_bytes()?),
        "categorical_sha256":sha256_bytes(&bridge.to_bytes()?),"scope":"native entry replay; all4096 Generate and all physical Copy; labels read only after forward; Generate-only pool is a diagnostic, not serving"});
    write(a, &format!("prediction-entry-{step:04}.json"), &v)?;
    Ok(v)
}

/// Zero-update entry-position ceiling over every development row.
///
/// The same target-free native forward as the control diagnostic, run over the
/// whole panel and reported twice: once with the physical Copy channel present
/// (the live mixed pool) and once with it removed (the Generate-only pool).
/// Labels are read only after each forward, so this is a capability bound on
/// the frozen artifact: not a fit, not a selection, not a serving change.
/// Stable, label-independent half of the panel. Any positional split on this
/// panel is a split on the answer -- the two answers alternate with row parity --
/// so the split hashes the case id.
fn panel_split(id: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in id.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash % 2
}

/// The boundary feature a dedicated entry scorer keys on: the query the model
/// was given. This is the plumbing proof -- it asks whether a learned table,
/// fitted on one half of the panel and applied at the entry position through the
/// same native forward and the same pool, moves the entry decision, and whether
/// it survives export to integer scores. The feature is deliberately the weakest
/// interesting one: it cannot generalise to an unseen wording, which the
/// feasibility read on the same panel already established, and that limit is
/// reported with the number rather than hidden behind it.
fn entry_scorer_read(
    a: &Args,
    l: &Loaded,
    model: &IntegerRealizer,
    g: &NativeGeometricGenerate,
    bridge: &NativeGeometricReadStateBridge,
    eps: &[Episode],
) -> Result<Value> {
    use uor_r4_integer::h4_tables::H4Code;
    let mut pool = NativeVocabularyActions::new(model.binding().clone(), &l.exp)?;
    let mut only = NativeVocabularyActions::new(model.binding().clone(), &l.exp)?;
    let feature = |e: &Episode| format!("q{:?}", e.packet.query_ids);
    let mut table = BTreeMap::<String, BTreeMap<u32, u64>>::new();
    let mut fit_rows = 0usize;
    for e in eps {
        if panel_split(&e.packet.id) != 0 {
            continue;
        }
        fit_rows += 1;
        *table
            .entry(feature(e))
            .or_default()
            .entry(e.target[0])
            .or_default() += 1;
    }
    let (mut test_rows, mut covered) = (0usize, 0usize);
    let (mut base_correct, mut full_correct, mut scored_correct, mut scored_full_correct) =
        (0usize, 0usize, 0usize, 0usize);
    let mut records = Vec::new();
    for e in eps {
        if panel_split(&e.packet.id) != 1 {
            continue;
        }
        test_rows += 1;
        let native = native_step(model, g, Some(bridge), &mut pool, e, &[], &l.cue, &l.prefix)?;
        let state: Vec<u8> = serde_json::from_value(native["retained_state_codes"].clone())?;
        let state = state
            .into_iter()
            .map(H4Code::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let mut scores = vec![0i64; g.vocab_size()];
        g.score_into(&state, &mut scores, &mut GenerateReadCounts::default())?;
        let ids: Vec<u32> = serde_json::from_value(native["copy_token_ids"].clone())?;
        let copies: Vec<i64> = serde_json::from_value(native["copy_raw_scores_q24"].clone())?;
        let gold = e.target[0];
        let base = only.reduce_trace(&scores, &[], &[])?;
        let base_full = pool.reduce_trace(&scores, &ids, &copies)?;
        base_correct += usize::from(base.summary.chosen_token_id == gold);
        full_correct += usize::from(base_full.summary.chosen_token_id == gold);
        // Apply the fitted table the way an exported scorer would: a bounded
        // integer bonus on the learned token, inside the same score clip the
        // native pool already enforces.
        let mut scored = scores.clone();
        let mut bonus = None;
        if let Some(counts) = table.get(&feature(e)) {
            covered += 1;
            let total = counts.values().sum::<u64>().max(1);
            if let Some((token, count)) = counts
                .iter()
                .max_by(|x, y| x.1.cmp(y.1).then_with(|| y.0.cmp(x.0)))
            {
                let scaled = (*count as f64 / total as f64) * (6.0 * f64::from(1 << 24));
                scored[*token as usize] =
                    scored[*token as usize].saturating_add(scaled.round() as i64);
                bonus = Some((*token, scaled.round() as i64));
            }
        }
        let with = pool.reduce_trace(&scored, &ids, &copies)?;
        let with_only = only.reduce_trace(&scored, &[], &[])?;
        scored_correct += usize::from(with_only.summary.chosen_token_id == gold);
        scored_full_correct += usize::from(with.summary.chosen_token_id == gold);
        records.push(json!({"id":e.packet.id,"gold_entry_label_only":gold,
            "base_generate_only":base.summary.chosen_token_id,
            "base_mixed":base_full.summary.chosen_token_id,
            "scored_generate_only":with_only.summary.chosen_token_id,
            "scored_mixed":with.summary.chosen_token_id,
            "scorer_bonus":bonus}));
    }
    let v = json!({"fit_rows":fit_rows,"test_rows":test_rows,"test_rows_covered_by_table":covered,
        "distinct_features":table.len(),
        "base_generate_only_correct":base_correct,"base_mixed_correct":full_correct,
        "scored_generate_only_correct":scored_correct,"scored_mixed_correct":scored_full_correct,
        "row_records":records,
        "scope":"plumbing proof on the native path: query-keyed table fitted on one panel half and applied as a bounded integer bonus at the entry position before the pool reduction; the feature cannot generalise to an unseen wording and the covered-row count says how much of the number is memorisation; no checkpoint export yet"});
    write(a, "entry-scorer-read.json", &v)?;
    Ok(v)
}

fn entry_ceiling_panel(
    a: &Args,
    l: &Loaded,
    model: &IntegerRealizer,
    g: &NativeGeometricGenerate,
    bridge: &NativeGeometricReadStateBridge,
    eps: &[Episode],
) -> Result<Value> {
    use uor_r4_integer::h4_tables::H4Code;
    let mut pool = NativeVocabularyActions::new(model.binding().clone(), &l.exp)?;
    let mut only = NativeVocabularyActions::new(model.binding().clone(), &l.exp)?;
    let mut rows = Vec::new();
    let (mut full_correct, mut generate_only_correct) = (0usize, 0usize);
    let (mut full_copy_dominated, mut gold_in_copy) = (0usize, 0usize);
    let mut gold_ranks: Vec<u64> = Vec::new();
    let mut next_gold_ranks: Vec<u64> = Vec::new();
    let mut gold_mass_fraction: Vec<f64> = Vec::new();
    for (index, e) in eps.iter().enumerate() {
        // Complete target-free native bank selection first. Labels cannot choose
        // a source occurrence, frame, route, state or vocabulary candidate.
        let native = native_step(model, g, Some(bridge), &mut pool, e, &[], &l.cue, &l.prefix)?;
        let state: Vec<u8> = serde_json::from_value(native["retained_state_codes"].clone())?;
        let state = state
            .into_iter()
            .map(H4Code::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let mut scores = vec![0i64; g.vocab_size()];
        g.score_into(&state, &mut scores, &mut GenerateReadCounts::default())?;
        let ids: Vec<u32> = serde_json::from_value(native["copy_token_ids"].clone())?;
        let copies: Vec<i64> = serde_json::from_value(native["copy_raw_scores_q24"].clone())?;
        let full = pool.reduce_trace(&scores, &ids, &copies)?;
        let generate_only = only.reduce_trace(&scores, &[], &[])?;
        let gold = e.target[0];
        let full_chosen = full.summary.chosen_token_id;
        let only_chosen = generate_only.summary.chosen_token_id;
        // Where the answer sits in the fully ordered emission, so "the emission
        // cannot do it" and "the emission nearly does it" are distinguishable.
        let mut ordered = generate_only
            .token_masses
            .iter()
            .map(|m| (m.token_id, m.weight_q31))
            .collect::<Vec<_>>();
        ordered.sort_by(|x, y| y.1.cmp(&x.1).then_with(|| x.0.cmp(&y.0)));
        let gold_rank = ordered
            .iter()
            .position(|m| m.0 == gold)
            .map(|position| position as u64 + 1);
        gold_ranks.push(gold_rank.unwrap_or(ordered.len() as u64 + 1));
        // The same read one token later, after observing the correct first
        // token. This separates "the boundary state carries nothing" from "the
        // emission cannot rank at all": the continuation is given its prefix,
        // while the entry position has to select the answer from the query.
        if e.target.len() > 1 {
            let native_next = native_step(
                model,
                g,
                Some(bridge),
                &mut pool,
                e,
                &[gold],
                &l.cue,
                &l.prefix,
            )?;
            let state_next: Vec<u8> =
                serde_json::from_value(native_next["retained_state_codes"].clone())?;
            let state_next = state_next
                .into_iter()
                .map(H4Code::try_from)
                .collect::<std::result::Result<Vec<_>, _>>()?;
            let mut scores_next = vec![0i64; g.vocab_size()];
            g.score_into(
                &state_next,
                &mut scores_next,
                &mut GenerateReadCounts::default(),
            )?;
            let only_next = only.reduce_trace(&scores_next, &[], &[])?;
            let gold_next = e.target[1];
            let mut ordered_next = only_next
                .token_masses
                .iter()
                .map(|m| (m.token_id, m.weight_q31))
                .collect::<Vec<_>>();
            ordered_next.sort_by(|x, y| y.1.cmp(&x.1).then_with(|| x.0.cmp(&y.0)));
            next_gold_ranks.push(
                ordered_next
                    .iter()
                    .position(|m| m.0 == gold_next)
                    .map(|position| position as u64 + 1)
                    .unwrap_or(ordered_next.len() as u64 + 1),
            );
        }
        gold_mass_fraction.push(
            ordered
                .iter()
                .find(|m| m.0 == gold)
                .map(|m| m.1 as f64 / generate_only.summary.total_weight_q31.max(1) as f64)
                .unwrap_or(0.),
        );
        full_correct += usize::from(full_chosen == gold);
        generate_only_correct += usize::from(only_chosen == gold);
        full_copy_dominated += usize::from(
            full.summary.chosen_copy_weight_q31 > full.summary.chosen_generate_weight_q31,
        );
        gold_in_copy += usize::from(ids.contains(&gold));
        rows.push(
            json!({"index":index,"id":e.packet.id,"gold_entry_label_only":gold,
            "full_chosen":full_chosen,"generate_only_chosen":only_chosen,
            "full_correct":full_chosen==gold,"generate_only_correct":only_chosen==gold,
            "gold_in_copy_candidates":ids.contains(&gold),"copy_candidates":ids.len(),
            "gold_generate_rank":gold_rank,
            "full_summary":full.summary,"generate_only_summary":generate_only.summary}),
        );
    }
    let mut ranks = gold_ranks.clone();
    let mut next_ranks = next_gold_ranks.clone();
    let mut fractions = gold_mass_fraction.clone();
    ranks.sort_unstable();
    next_ranks.sort_unstable();
    fractions.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
    let pick = |v: &[u64], q: f64| -> u64 {
        if v.is_empty() {
            0
        } else {
            v[((v.len() as f64 - 1.) * q).round() as usize]
        }
    };
    let pick_fraction = |v: &[f64], q: f64| -> f64 {
        if v.is_empty() {
            0.
        } else {
            v[((v.len() as f64 - 1.) * q).round() as usize]
        }
    };
    let v = json!({"rows":eps.len(),
        "full_pool_entry_correct":full_correct,
        "generate_only_entry_correct":generate_only_correct,
        "full_pool_winners_copy_dominated":full_copy_dominated,
        "rows_with_gold_in_copy_candidates":gold_in_copy,
        "generate_only_gold_rank_min":ranks.first().copied().unwrap_or(0),
        "generate_only_gold_rank_median":pick(&ranks, 0.5),
        "generate_only_gold_rank_p90":pick(&ranks, 0.9),
        "generate_only_gold_rank_max":ranks.last().copied().unwrap_or(0),
        "generate_only_gold_mass_fraction_median":pick_fraction(&fractions, 0.5),
        "next_token_gold_rank_median":pick(&next_ranks, 0.5),
        "next_token_gold_rank_p90":pick(&next_ranks, 0.9),
        "next_token_gold_rank_max":next_ranks.last().copied().unwrap_or(0),
        "next_token_rows":next_ranks.len(),
        "generate_actions":eps
            .first()
            .map_or(0, |_| only.legal_token_ids().len()),
        "row_records":rows,
        "scope":"zero-update entry ceiling: target-free native forward, mixed pool versus Generate-only pool, labels read after forward; gold rank is its position in the fully ordered emission; a capability bound on one artifact, not a fit and not held-out language evidence"});
    write(a, "entry-ceiling.json", &v)?;
    Ok(v)
}

fn control_crossings(
    a: &Args,
    initial: &NativeGeometricGenerate,
    g: &NativeGeometricGenerate,
    step: usize,
) -> Result<Value> {
    let prototypes = initial
        .prototypes()
        .iter()
        .zip(g.prototypes())
        .filter(|(x, y)| x != y)
        .count();
    let mut bias = 0;
    let mut unary = 0;
    let mut pair = 0;
    for id in 0..g.vocab_size() {
        bias += usize::from(initial.token_bias(id)? != g.token_bias(id)?);
    }
    for lane in 0..g.lanes() {
        for r in 0..120 {
            unary += usize::from(
                initial.energy().get_unary(lane as u8, r)?
                    != g.energy().get_unary(lane as u8, r)?,
            );
        }
    }
    for e in 0..g.energy().edges().len() {
        for x in 0..120 {
            for y in 0..120 {
                pair += usize::from(
                    initial.energy().get_pair(e, x, y)? != g.energy().get_pair(e, x, y)?,
                );
            }
        }
    }
    let old = tree_identity(&a.out.join("checkpoint-0000/native/consumer"))?;
    let current = tree_identity(&a.out.join(format!("checkpoint-{step:04}/native/consumer")))?;
    Ok(
        json!({"step":step,"generate_prototype_lane_code_crossings":prototypes,"generate_bias_code_crossings":bias,
        "generate_unary_code_crossings":unary,"generate_pair_code_crossings":pair,
        "native_consumer_tree_changed":old!=current,"initial_native_consumer_tree_sha256":old,"current_native_consumer_tree_sha256":current,
        "consumer_identity_scope":"tree identity changes include native Context/Potential metadata; not a count of categorical state crossings"}),
    )
}

const CONTROL_RESUME_PRODUCER: &str = "3fc63bec66e03a3148c83c48c795667582d258c2";
fn restore_control_resume(a: &Args, l: &Loaded, d: &Device) -> Result<Option<Value>> {
    let Some(root) = &a.prediction_control_resume else {
        return Ok(None);
    };
    report_output::verify(root)?;
    let r = read(&root.join("report.json"))?;
    let admission = read(&root.join("admission.json"))?;
    let source_step = a.prediction_control_resume_source_step.unwrap_or(32);
    let generate_step = a.prediction_control_resume_generate_step.unwrap_or(32);
    let cross = source_step == 48
        && generate_step == 64
        && r["source_commit"] == "b451571aa638d1f93e3ccf25f718f27ed2ea731f";
    let original =
        source_step == 32 && generate_step == 32 && r["source_commit"] == CONTROL_RESUME_PRODUCER;
    if !original && !cross {
        return Err(bad("unreviewed component-resume producer/step combination"));
    }
    if cross && control_settings(a)?.0 != 0 {
        return Err(bad("component recomposition is zero-update only"));
    }
    let final_step = if cross { 64 } else { 32 };
    let cp = root.join(format!("checkpoint-{source_step:04}"));
    let gp = root.join(format!("checkpoint-{generate_step:04}"));
    let final_receipt = read(&root.join(format!("checkpoint-{final_step:04}/receipt.json")))?;
    let generate_receipt = read(&gp.join("receipt.json"))?;
    let receipt = read(&cp.join("receipt.json"))?;
    if r["schema"] != "uor-r4.geometric-prediction-control/1"
        || r["status"] != "COMPLETED"
        || r["mode"] != "prediction_control"
        || r["updates"] != final_step
        || r["original_indices"] != json!(CONTROL_INDICES)
        || r["final_receipt"] != final_receipt
        || final_receipt["step"] != final_step
        || receipt["step"] != source_step
        || generate_receipt["step"] != generate_step
        || generate_receipt["generate_sha256"] != sha256_file(&gp.join("generate.bin"))?
        || generate_receipt["credit"] != a.credit.name()
        || generate_receipt["read_state_pullback"] != a.read_state_pullback.name()
        || !r["blocks"]
            .as_array()
            .is_some_and(|rows| rows.iter().any(|v| v["step"] == source_step))
        || receipt["credit"] != a.credit.name()
        || receipt["read_state_pullback"] != a.read_state_pullback.name()
        || admission["input_sha256"] != INPUT_SHA
        || admission["labels_sha256"] != LABEL_SHA
        || admission["checkpoint_receipt_sha256"] != CP_RECEIPT_SHA
        || admission["credit"] != a.credit.name()
        || admission["read_state_pullback"] != a.read_state_pullback.name()
        || admission["panel"]["original_indices"] != json!(CONTROL_INDICES)
    {
        return Err(bad("resume must be an authenticated completed supported control with matching components/data/geometry/credit"));
    }
    if tree_identity(&cp.join("read-state-bridge-source"))?
        != tree_identity(&a.checkpoint.join("read-state-bridge-source"))?
    {
        return Err(bad("resume changed original frozen bridge masters"));
    }
    let cue = cue_payload(&cp.join("cue"))?;
    let prefix = prefix_payload(&cp.join("prefix"))?;
    if cue.config() != l.cue.config()
        || cue.packed_coefficients() != l.cue.packed_coefficients()
        || cue.joint().map(|v| v.packed_coefficients())
            != l.cue.joint().map(|v| v.packed_coefficients())
        || prefix.config() != l.prefix.config()
        || prefix.packed_coefficients() != l.prefix.packed_coefficients()
    {
        return Err(bad(
            "resume changed fixed cue/prefix coefficient/configuration payloads",
        ));
    }
    if sha256_file(&cp.join("read-state-bridge.bin"))? != QUARTER_SHA
        || sha256_file(&cp.join("read-state-bridge-categorical.bin"))? != CAT_SHA
    {
        return Err(bad("resume changed frozen bridge"));
    }
    let tokbytes = fs::read(cp.join("native/tokenizer.json"))?;
    let binding: NativeArtifactBinding = serde_json::from_value(receipt["parent"].clone())?;
    let resumed_native = IntegerRealizer::load_native(&cp.join("native"), &binding)?;
    if resumed_native.binding() != l.integer.binding() {
        return Err(bad("resume tokenizer/protocol binding differs"));
    }
    let restored =
        SourceRealizerWeights::load_context_potential_on_device(&cp.join("source"), &tokbytes, d)?;
    let mut source = l.source.context_state_parameters();
    source.extend(l.source.potential_parameters());
    let mut donor = restored.context_state_parameters();
    donor.extend(restored.potential_parameters());
    if source.keys().collect::<Vec<_>>() != donor.keys().collect::<Vec<_>>() {
        return Err(bad("resume source master families differ"));
    }
    for (name, var) in &source {
        var.set(donor[name].as_tensor())?;
    }
    let gm = read(&gp.join("generate-source/metadata.json"))?;
    if gm["tokenizer_sha256"] != sha256_bytes(&tokbytes)
        || gm["protocol"] != serde_json::to_value(l.integer.binding().protocol())?
        || gm["lanes"] != json!(l.generate.lanes())
    {
        return Err(bad("resume Generate metadata binding differs"));
    }
    restore(
        &gp.join("generate-source"),
        &gm["parameters"],
        &l.generate.parameters(),
        d,
    )?;
    if l.generate.export_native()?.to_bytes()? != fs::read(gp.join("generate.bin"))? {
        return Err(bad("resumed Generate masters do not export saved artifact"));
    }
    if cross {
        let other_binding: NativeArtifactBinding =
            serde_json::from_value(generate_receipt["parent"].clone())?;
        let other_native = IntegerRealizer::load_native(&gp.join("native"), &other_binding)?;
        if other_native.binding() != resumed_native.binding() {
            return Err(bad("component tokenizer/protocol binding differs"));
        }
        let other_source = SourceRealizerWeights::load_context_potential_on_device(
            &gp.join("source"),
            &tokbytes,
            &Device::Cpu,
        )?;
        if identities(&other_source.context_state_parameters())?
            != identities(&restored.context_state_parameters())?
        {
            return Err(bad("component Context masters differ"));
        }
        let source_gm = read(&cp.join("generate-source/metadata.json"))?;
        if source_gm["parameters"]["generate.prototype_choices"]
            != gm["parameters"]["generate.prototype_choices"]
        {
            return Err(bad("component prototype masters differ"));
        }
        for name in ["read-state-bridge-source"] {
            if tree_identity(&cp.join(name))? != tree_identity(&gp.join(name))? {
                return Err(bad("component frozen bridge masters differ"));
            }
        }
        let gc = cue_payload(&gp.join("cue"))?;
        let pc = prefix_payload(&gp.join("prefix"))?;
        if gc.config() != l.cue.config()
            || gc.packed_coefficients() != l.cue.packed_coefficients()
            || gc.joint().map(|v| v.packed_coefficients())
                != l.cue.joint().map(|v| v.packed_coefficients())
            || pc.config() != l.prefix.config()
            || pc.packed_coefficients() != l.prefix.packed_coefficients()
            || sha256_file(&gp.join("read-state-bridge.bin"))? != QUARTER_SHA
            || sha256_file(&gp.join("read-state-bridge-categorical.bin"))? != CAT_SHA
        {
            return Err(bad("component frozen sidecars differ"));
        }
    }
    let n = l.source.compile_context_potential_rebound(&l.frozen)?;
    // Provenance belongs to the independently saved/reloaded parent; the
    // freshly compiled object supplies only a would-export execution identity.
    let identity: ConsumerIdentity =
        serde_json::from_value(read(&cp.join("native/metadata.json"))?["identity"].clone())?;
    let reloaded = NativeSourceRealizer::load(&cp.join("native"), &l.source, &identity)?;
    let loaded_binding = reloaded.artifact_binding()?;
    if serde_json::to_value(&loaded_binding)? != receipt["parent"]
        || n.execution_binding()? != loaded_binding
    {
        return Err(bad(
            "resumed source masters do not export saved native identity",
        ));
    }
    Ok(Some(
        json!({"root":fs::canonicalize(root)?,"producer_source_commit":r["source_commit"],"source_step":source_step,"generate_step":generate_step,"component_recomposition":cross,
        "report_sha256":sha256_file(&root.join("report.json"))?,"manifest_sha256":sha256_file(&root.join("manifest.json"))?,
        "checkpoint_receipt_sha256":sha256_file(&cp.join("receipt.json"))?,"generate_checkpoint_receipt_sha256":sha256_file(&gp.join("receipt.json"))?,"final_checkpoint_receipt_sha256":sha256_file(&root.join(format!("checkpoint-{final_step:04}/receipt.json")))?,"producer_completed_block_updates":final_step,"continued_parent_steps":if cross {None} else {Some(32)},"optimizer":if cross {"zero-update component recomposition; no optimizer steps"} else {"fresh field Adam, prior moments not resumed"}}),
    ))
}
fn run_prediction_control(
    a: &Args,
    start: Instant,
    d: &Device,
    l: &Loaded,
    train: &[Episode],
    dev: Vec<Episode>,
) -> Result<Value> {
    let (limit, rates) = control_settings(a)?;
    let resume = restore_control_resume(a, l, d)?;
    let eps = dev
        .into_iter()
        .enumerate()
        .filter_map(|(i, e)| CONTROL_INDICES.contains(&i).then_some(e))
        .collect::<Vec<_>>();
    let panel = control_panel_admit(&eps)?;
    for (local, e) in eps.iter().enumerate() {
        if train[CONTROL_INDICES[local]].packet.id != e.packet.id
            || train[CONTROL_INDICES[local]].target != e.target
        {
            return Err(bad("control train/development cases differ"));
        }
    }
    let original = identities(&l.original_bridge.parameters())?;
    let marker = identities(&l.marker.parameters())?;
    write(
        a,
        "admission.json",
        &json!({"mode":"prediction_control","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),
        "checkpoint_receipt_sha256":CP_RECEIPT_SHA,"input_sha256":INPUT_SHA,"labels_sha256":LABEL_SHA,
        "panel":panel,"updates":limit,"rates":rates,"resume":resume,"trainable":a.prediction_control_trainable,"credit":a.credit.name(),"read_state_pullback":a.read_state_pullback.name(),
        "native_pool_backend":"host","fresh_adam":true,"initial_active_masters":identities(&active(&l.source,&l.generate)?)?,
        "original_bridge_masters":original,"marker_masters":marker,"loss_scope":a.loss_scope.name(),"phase_policy":loss_weight_policy(true, a.loss_scope)}),
    )?;
    write(
        a,
        "order.json",
        &json!({"order":CONTROL_INDICES,"policy":"same predetermined eight cases every update; no resampling or model reseeding"}),
    )?;
    let (model, g, b, initial_receipt) = checkpoint(a, 0, l)?;
    if g.vocab_size() != 4096 {
        return Err(bad("control requires the full pinned 4096 vocabulary"));
    }
    let initial_parent = if let Some(root) = &a.prediction_control_resume {
        root.join(format!(
            "checkpoint-{:04}",
            a.prediction_control_resume_source_step.unwrap_or(32)
        ))
    } else {
        a.checkpoint.clone()
    };
    let expected_receipt = read(&initial_parent.join("receipt.json"))?;
    let initial_generate = if let Some(root) = &a.prediction_control_resume {
        root.join(format!(
            "checkpoint-{:04}",
            a.prediction_control_resume_generate_step.unwrap_or(32)
        ))
    } else {
        a.checkpoint.clone()
    };
    if g.to_bytes()? != fs::read(initial_generate.join("generate.bin"))?
        || initial_receipt["parent"] != expected_receipt["parent"]
    {
        return Err(bad("control zero-update donor/resume identity differs"));
    }
    if a.prediction_control_resume.is_some() {
        for name in ["source", "native", "cue", "prefix"] {
            if tree_identity(&a.out.join("checkpoint-0000").join(name))?
                != tree_identity(&initial_parent.join(name))?
            {
                return Err(bad(
                    "initial resumed checkpoint is not exact saved native/master input",
                ));
            }
        }
    }
    let (_, admission) = batch(a, l, train, &CONTROL_INDICES, d, start, Some(&model))?;
    write(a, "zero-update-admission.json", &admission)?;
    // Initial entry replay precedes any optimizer construction or update.
    control_entry_diagnostics(a, l, &model, &g, &b, &eps, 0)?;
    // The eight control cases are a selected subset. Measure the same artifact's
    // entry decision on the whole panel it was drawn from, mixed pool versus
    // Generate-only pool, so "works on the eight" and "works on the panel" are
    // separate numbers rather than one number and an assumption. At the declared
    // zero-update recomposition the artifact measured here is the recomposed one.
    let panel_ceiling = {
        let public = NativeVocabularyActions::new(l.integer.binding().clone(), &l.exp)?;
        let legal = public
            .legal_token_ids()
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let panel = load_panel(
            &a.development_inputs,
            &a.development_labels,
            &l.integer,
            &l.tokenizer,
            &legal,
            512,
        )?;
        entry_ceiling_panel(a, l, &model, &g, &b, &panel)?
    };
    let initial_eval = evaluate(
        a,
        "prediction-0000",
        &model,
        &g,
        Some(&b),
        &l.exp,
        &eps,
        &l.tokenizer,
        &l.cue,
        &l.prefix,
        start,
    )?;
    let initial_metrics = metrics(a, &initial_eval, &eps)?;
    write(a, "prediction-metrics-0000.json", &initial_metrics)?;
    let mut blocks = vec![
        json!({"step":0,"evaluation":initial_eval,"metrics":initial_metrics,"native_crossings":control_crossings(a,&g,&g,0)?}),
    ];
    let gp = l.generate.parameters();
    let (prototype, coefficients): (BTreeMap<_, _>, BTreeMap<_, _>) = gp
        .into_iter()
        .partition(|(name, _)| name == "generate.prototype_choices");
    let context = l.source.context_state_parameters();
    let potential = l.source.potential_parameters();
    let field_only = a.prediction_control_trainable != ControlTrainable::Joint;
    let potential_active = a.prediction_control_trainable != ControlTrainable::GenerateField;
    let mut frozen_groups = prototype.clone();
    frozen_groups.extend(context.clone());
    if !potential_active {
        frozen_groups.extend(potential.clone());
    }
    let frozen_groups_identity = identities(&frozen_groups)?;
    let mut go = optimizer(&coefficients, rates.generate)?;
    let mut po = optimizer(&prototype, rates.prototype)?;
    let mut co = optimizer(&context, rates.context)?;
    let mut vo = optimizer(&potential, rates.potential)?;
    let mut updates = Vec::new();
    let mut final_receipt = initial_receipt.clone();
    for update in 0..limit {
        disk_floor(a)?;
        deadline(a, start)?;
        let (grads, receipt) = batch(a, l, train, &CONTROL_INDICES, d, start, None)?;
        let grads = if field_only {
            grads
                .into_iter()
                .filter(|(name, _)| {
                    coefficients.contains_key(name)
                        || (potential_active && potential.contains_key(name))
                })
                .collect()
        } else {
            grads
        };
        let (denominator, norm) = clip_denominator(&grads, d)?;
        apply(&mut go, &coefficients, &grads, &denominator)?;
        if !field_only {
            apply(&mut po, &prototype, &grads, &denominator)?;
            apply(&mut co, &context, &grads, &denominator)?;
            for var in context.values() {
                var.set(&var.as_tensor().clamp(-1.75, 1.75)?)?;
            }
        }
        if potential_active {
            apply(&mut vo, &potential, &grads, &denominator)?;
            l.source.project_potential_range()?;
        }
        l.generate.project_shadow_range()?;
        if field_only && identities(&frozen_groups)? != frozen_groups_identity {
            return Err(bad(
                "field control changed frozen Context/prototype or inactive Potential masters",
            ));
        }
        if identities(&l.original_bridge.parameters())? != original
            || identities(&l.marker.parameters())? != marker
        {
            return Err(bad("control mutated frozen bridge masters"));
        }
        d.synchronize()?;
        updates.push(json!({"step":update+1,"before_update":receipt,"rates":rates,"global_active_gradient_norm":norm}));
        write(a, "updates.json", &json!(updates))?;
        if (update + 1) % 8 == 0 {
            let step = update + 1;
            let (model, current, bridge, rec) = checkpoint(a, step, l)?;
            control_entry_diagnostics(a, l, &model, &current, &bridge, &eps, step)?;
            let eval = evaluate(
                a,
                &format!("prediction-{step:04}"),
                &model,
                &current,
                Some(&bridge),
                &l.exp,
                &eps,
                &l.tokenizer,
                &l.cue,
                &l.prefix,
                start,
            )?;
            let m = metrics(a, &eval, &eps)?;
            write(a, &format!("prediction-metrics-{step:04}.json"), &m)?;
            blocks.push(json!({"step":step,"evaluation":eval,"metrics":m,"native_crossings":control_crossings(a,&g,&current,step)?}));
            final_receipt = rec;
        }
        write(a, "prediction-blocks.json", &json!(blocks))?;
    }
    let last = blocks
        .last()
        .ok_or_else(|| bad("control final evaluation absent"))?;
    let win = last["evaluation"]["complete"] == 8
        && last["metrics"]["entry_own_correct"] == 8
        && last["metrics"]["paired_own_position_both_correct"] == 4
        && last["metrics"]["first_divergent_own_prefix_reached"] == 8;
    write(a, "prediction-blocks.json", &json!(blocks))?;
    Ok(
        json!({"schema":"uor-r4.geometric-prediction-control/1","status":"COMPLETED","mode":"prediction_control",
        "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"updates":limit,"rates":rates,"native_pool_backend":"host",
        "resume":resume,"trainable":a.prediction_control_trainable,"source_routing_scope":"Potential updates may change selected physical Source and transported state; no fixed-coordinate claim","original_indices":CONTROL_INDICES,"training_row_draws":limit*8,"target_position_draws":limit*84,
        "initial_receipt":initial_receipt,"final_receipt":final_receipt,"blocks":blocks,"panel_ceiling_step0":panel_ceiling,
        "native_prediction_control_win":win,"elapsed_seconds":start.elapsed().as_secs_f64(),
        "scope":"construction learning control on8 retained cases; complete accepted own-prefix replies+EOS and four source-swap pairs required; no generalization/chat/attention qualification; initial diagnostic0 can be run before choosing prospective control rates"}),
    )
}

fn run(a: &Args, start: Instant) -> Result<Value> {
    let d = cuda()?;
    let l = load(a, &d)?;
    let public = NativeVocabularyActions::new(l.integer.binding().clone(), &l.exp)?;
    let legal = public
        .legal_token_ids()
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let train = load_panel(
        &a.training_inputs,
        &a.training_labels,
        &l.integer,
        &l.tokenizer,
        &legal,
        512,
    )?;
    let dev = load_panel(
        &a.development_inputs,
        &a.development_labels,
        &l.integer,
        &l.tokenizer,
        &legal,
        512,
    )?;
    if train.len() != 512 || dev.len() != 512 {
        return Err(bad("full fixed panel required"));
    }
    pairs(&train)?;
    pairs(&dev)?;
    if a.mode == Mode::PredictionControl {
        return run_prediction_control(a, start, &d, &l, &train, dev);
    }
    let frozen_original = identities(&l.original_bridge.parameters())?;
    let frozen_marker = identities(&l.marker.parameters())?;
    let initial_masters = identities(&active(&l.source, &l.generate)?)?;
    let schedule = order(a.seed, train.len());
    write(
        a,
        "order.json",
        &json!({"seed":a.seed,"order":schedule,
        "policy":format!("SplitMix64/Fisher-Yates full512 once; fixed cyclic batches of8, {}updates; order only, no model reinitialization", a.updates)}),
    )?;
    write(
        a,
        "admission.json",
        &json!({"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),
        "checkpoint_receipt_sha256":CP_RECEIPT_SHA,"parent_config_sha256":sha256_file(&a.parent_config)?,
        "training_input_sha256":INPUT_SHA,"training_labels_sha256":LABEL_SHA,
        "construction_panel_overlap":"training and open development are the identical retained512panel; not generalization",
        "original_bridge_masters":frozen_original,"marker_masters":frozen_marker,
        "categorical":l.categorical_receipt,"initial_active_masters":initial_masters,
        "active_parameter_names":active(&l.source,&l.generate)?.keys().collect::<Vec<_>>(),
        "fresh_adam":true,"rates":{"generate":0.003,"prototype":0.01,"context":0.002,"potential":0.003},
        "phase_policy":loss_weight_policy(true, a.loss_scope),"loss_scope":a.loss_scope.name(),"declared_updates":a.updates,"credit":a.credit.name(),"read_state_pullback":a.read_state_pullback.name(),"CUDA_VISIBLE_DEVICES":std::env::var("CUDA_VISIBLE_DEVICES").ok()}),
    )?;
    let (initial_native, initial_generate, initial_bridge, initial_receipt) = checkpoint(a, 0, &l)?;
    if initial_native.binding().tokenizer_sha256() != l.integer.binding().tokenizer_sha256()
        || initial_generate.to_bytes()? != fs::read(a.checkpoint.join("generate.bin"))?
        || initial_receipt["parent"] != l.receipt["parent"]
    {
        return Err(bad("zero-update restored native identity differs"));
    }
    let indices = schedule[..BATCH].to_vec();
    let (_, admission) = batch(a, &l, &train, &indices, &d, start, Some(&initial_native))?;
    write(a, "zero-update-admission.json", &admission)?;
    let (baseline, baseline_provenance) = if let Some(root) = &a.baseline {
        reuse_baseline(a, root, &dev, &initial_receipt)?
    } else {
        let evaluation = evaluate(
            a,
            "development-0000",
            &initial_native,
            &initial_generate,
            Some(&initial_bridge),
            &l.exp,
            &dev,
            &l.tokenizer,
            &l.cue,
            &l.prefix,
            start,
        )?;
        (
            evaluation,
            json!({"execution":"new native CPU integer evaluation on this pod",
            "reused":false,"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT")}),
        )
    };
    let initial_metrics = metrics(a, &baseline, &dev)?;
    write(a, "metrics-0000.json", &initial_metrics)?;
    if a.mode == Mode::Admission {
        return Ok(
            json!({"schema":"uor-r4.geometric-frozen-map-fit/1","status":"COMPLETED","mode":"admission",
            "updates":0,"credit":a.credit.name(),"read_state_pullback":a.read_state_pullback.name(),"order_seed":a.seed,"admission":admission,
            "initial_metrics":initial_metrics,"initial_evaluation_provenance":baseline_provenance,
            "elapsed_seconds":start.elapsed().as_secs_f64(),
            "scope":"zero-update authenticated categorical restart admission; no learning verdict"}),
        );
    }
    if a.mode == Mode::EntryCeiling {
        let ceiling = entry_ceiling_panel(
            a,
            &l,
            &initial_native,
            &initial_generate,
            &initial_bridge,
            &dev,
        )?;
        let scorer = if a.ceiling_scorer {
            Some(entry_scorer_read(
                a,
                &l,
                &initial_native,
                &initial_generate,
                &initial_bridge,
                &dev,
            )?)
        } else {
            None
        };
        return Ok(
            json!({"schema":"uor-r4.geometric-frozen-map-fit/1","status":"COMPLETED","mode":"entry_ceiling",
            "updates":0,"credit":a.credit.name(),"read_state_pullback":a.read_state_pullback.name(),"order_seed":a.seed,
            "checkpoint_receipt_sha256":CP_RECEIPT_SHA,"ceiling":ceiling,"entry_scorer":scorer,"initial_metrics":initial_metrics,
            "elapsed_seconds":start.elapsed().as_secs_f64(),
            "scope":"zero-update entry-position ceiling with the physical Copy channel removed; labels read only after the target-free forward; no fit and no serving change"}),
        );
    }
    let gp = l.generate.parameters();
    let (prototype, coefficients): (BTreeMap<_, _>, BTreeMap<_, _>) = gp
        .into_iter()
        .partition(|(name, _)| name == "generate.prototype_choices");
    let context = l.source.context_state_parameters();
    let potential = l.source.potential_parameters();
    let mut go = optimizer(&coefficients, 0.003)?;
    let mut po = optimizer(&prototype, 0.01)?;
    let mut co = optimizer(&context, 0.002)?;
    let mut vo = optimizer(&potential, 0.003)?;
    let mut updates = Vec::new();
    for update in 0..a.updates {
        disk_floor(a)?;
        deadline(a, start)?;
        let indices = (0..BATCH)
            .map(|i| schedule[(update * BATCH + i) % schedule.len()])
            .collect::<Vec<_>>();
        let (grads, receipt) = batch(a, &l, &train, &indices, &d, start, None)?;
        let (denominator, norm) = clip_denominator(&grads, &d)?;
        apply(&mut go, &coefficients, &grads, &denominator)?;
        apply(&mut po, &prototype, &grads, &denominator)?;
        apply(&mut co, &context, &grads, &denominator)?;
        apply(&mut vo, &potential, &grads, &denominator)?;
        l.generate.project_shadow_range()?;
        for var in context.values() {
            var.set(&var.as_tensor().clamp(-1.75, 1.75)?)?;
        }
        l.source.project_potential_range()?;
        if identities(&l.original_bridge.parameters())? != frozen_original
            || identities(&l.marker.parameters())? != frozen_marker
        {
            return Err(bad("frozen bridge master mutation"));
        }
        d.synchronize()?;
        updates.push(
            json!({"step":update+1,"read_state_pullback":a.read_state_pullback.name(),"before_update":receipt,"global_active_gradient_norm":norm}),
        );
        write(a, "updates.json", &json!(updates))?;
        // Recoverable exact checkpoints, not acceptance/reselection gates.
        if (update + 1) % 32 == 0 {
            checkpoint(a, update + 1, &l)?;
        }
    }
    let (native, generate, bridge, final_receipt) = {
        // Final checkpoint was already written at the declared dose; reload all
        // native paths.
        let cp = a.out.join(format!("checkpoint-{:04}", a.updates));
        let rec = read(&cp.join("receipt.json"))?;
        let binding: NativeArtifactBinding = serde_json::from_value(rec["parent"].clone())?;
        (
            IntegerRealizer::load_native(&cp.join("native"), &binding)?,
            NativeGeometricGenerate::from_bytes(
                &fs::read(cp.join("generate.bin"))?,
                l.generate.binding(),
            )?,
            NativeGeometricReadStateBridge::from_bytes(
                &fs::read(cp.join("read-state-bridge-categorical.bin"))?,
                l.generate.binding(),
            )?,
            rec,
        )
    };
    let final_eval = evaluate(
        a,
        &format!("development-{:04}", a.updates),
        &native,
        &generate,
        Some(&bridge),
        &l.exp,
        &dev,
        &l.tokenizer,
        &l.cue,
        &l.prefix,
        start,
    )?;
    let final_metrics = metrics(a, &final_eval, &dev)?;
    write(a, &format!("metrics-{:04}.json", a.updates), &final_metrics)?;
    Ok(
        json!({"schema":"uor-r4.geometric-frozen-map-fit/1","status":"COMPLETED","mode":"fit",
        "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"device":"cuda:0","credit":a.credit.name(),"read_state_pullback":a.read_state_pullback.name(),
        "order_seed":a.seed,"updates":a.updates,"batch":BATCH,"token_backward_chunk":1,"loss_scope":a.loss_scope.name(),
        "initial_receipt":initial_receipt,"final_receipt":final_receipt,
        "initial_metrics":initial_metrics,"final_metrics":final_metrics,
        "initial_evaluation_provenance":baseline_provenance,
        "final_active_masters":identities(&active(&l.source,&l.generate)?)?,
        "frozen_original_masters":frozen_original,"frozen_marker_masters":frozen_marker,
        "elapsed_seconds":start.elapsed().as_secs_f64(),
        "scope":"matched fixed categorical-map checkpoint restart; context/potential/Generate learned from actual native alias objective; train=open-development512; first-source-word teacher metrics separate from own-prefix outputs; no held-out transfer, attention or chat qualification"}),
    )
}
fn main() -> Result<()> {
    let a = args()?;
    report_output::claim(&a.out)?;
    let start = Instant::now();
    let result = run(&a, start);
    let report = match &result {
        Ok(value) => value.clone(),
        Err(e) => {
            json!({"schema":"uor-r4.geometric-frozen-map-fit/1","status":"FAILED","error":e.to_string(),"credit":a.credit.name(),"read_state_pullback":a.read_state_pullback.name(),
            "elapsed_seconds":start.elapsed().as_secs_f64(),
            "model_verdict":"UNQUALIFIED; preserve partial checkpoints and completed rows; execution failure is not model failure"})
        }
    };
    // Failure receipts use a reserved margin even when report storage admission failed.
    fs::write(
        a.out.join("report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    report_output::seal(&a.out)?;
    report_output::verify(&a.out)?;
    result.map(|_| ())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn read_state_pullback_config_defaults_serializes_and_rejects_unknown() -> Result<()> {
        let original = json!({"mode":"fit","credit":"raw_identity","seed":1001,
            "checkpoint":"cp","saved_fit":"fit","categorical":"cat","parent_config":"parent",
            "training_inputs":"input","training_labels":"labels","development_inputs":"input",
            "development_labels":"labels","maximum_seconds":3600,"maximum_report_bytes":1073741824,"out":"new"});
        let legacy: Args = serde_json::from_value(original.clone())?;
        assert_eq!(legacy.read_state_pullback, ReadStatePullback::Legacy);
        assert_eq!(
            serde_json::to_value(legacy.read_state_pullback)?,
            json!("legacy")
        );
        let mut categorical = original.clone();
        categorical["read_state_pullback"] = json!("categorical");
        let parsed: Args = serde_json::from_value(categorical.clone())?;
        assert_eq!(parsed.read_state_pullback, ReadStatePullback::Categorical);
        assert_eq!(
            serde_json::to_value(parsed.read_state_pullback)?,
            json!("categorical")
        );
        assert!(parsed.credit == Credit::RawIdentity);
        categorical["read_state_pullback"] = json!("query_keep");
        assert!(serde_json::from_value::<Args>(categorical).is_err());
        Ok(())
    }
    #[test]
    fn prediction_control_config_is_explicit_and_cannot_change_fixed_fit() -> Result<()> {
        let base = json!({"mode":"fit","credit":"raw_identity","seed":1001,
            "checkpoint":"cp","saved_fit":"fit","categorical":"cat","parent_config":"parent",
            "training_inputs":"input","training_labels":"labels","development_inputs":"input",
            "development_labels":"labels","maximum_seconds":3600,"maximum_report_bytes":1073741824,"out":"new"});
        let fixed: Args = serde_json::from_value(base.clone())?;
        let (_, defaults) = control_settings(&fixed)?;
        assert_eq!(defaults.generate, 0.003);
        let mut config = base.clone();
        config["prediction_control_updates"] = json!(16);
        assert!(control_settings(&serde_json::from_value::<Args>(config.clone())?).is_err());
        config["mode"] = json!("prediction_control");
        assert_eq!(
            control_settings(&serde_json::from_value::<Args>(config.clone())?)?.0,
            16
        );
        config["prediction_control_updates"] = json!(0);
        assert_eq!(
            control_settings(&serde_json::from_value::<Args>(config.clone())?)?.0,
            0
        );
        config
            .as_object_mut()
            .ok_or_else(|| bad("test config object"))?
            .remove("prediction_control_updates");
        assert_eq!(
            control_settings(&serde_json::from_value::<Args>(config.clone())?)?.0,
            32
        );
        config["prediction_control_updates"] = json!(64);
        assert_eq!(
            control_settings(&serde_json::from_value::<Args>(config.clone())?)?.0,
            64
        );
        config["prediction_control_updates"] = json!(65);
        assert!(control_settings(&serde_json::from_value::<Args>(config.clone())?).is_err());
        config["prediction_control_updates"] = json!(8);
        assert!(control_settings(&serde_json::from_value::<Args>(config.clone())?).is_err());
        config["prediction_control_updates"] = json!(32);
        config["prediction_control_rates"] =
            json!({"generate":0.04,"prototype":0.01,"context":0.002,"potential":0.003});
        let mut parsed: Args = serde_json::from_value(config.clone())?;
        assert_eq!(control_settings(&parsed)?.1.generate, 0.04);
        parsed
            .prediction_control_rates
            .as_mut()
            .ok_or_else(|| bad("test rates absent"))?
            .generate = f64::NAN;
        assert!(control_settings(&parsed).is_err());
        config["mode"] = json!("admission");
        assert!(control_settings(&serde_json::from_value::<Args>(config.clone())?).is_err());
        config["mode"] = json!("prediction_control");
        config["prediction_control_trainable"] = json!("generate_field");
        assert!(control_settings(&serde_json::from_value::<Args>(config.clone())?).is_err());
        config["prediction_control_resume"] = json!("sealed-control32");
        assert!(control_settings(&serde_json::from_value::<Args>(config.clone())?).is_ok());
        config["prediction_control_trainable"] = json!("potential_generate_field");
        assert!(control_settings(&serde_json::from_value::<Args>(config.clone())?).is_ok());
        config["prediction_control_updates"] = json!(0);
        config["prediction_control_resume_source_step"] = json!(48);
        config["prediction_control_resume_generate_step"] = json!(64);
        let mixed: Args = serde_json::from_value(config.clone())?;
        assert_eq!(mixed.prediction_control_resume_source_step, Some(48));
        assert_eq!(mixed.prediction_control_resume_generate_step, Some(64));
        assert_eq!(control_settings(&mixed)?.0, 0);
        config
            .as_object_mut()
            .ok_or_else(|| bad("test object"))?
            .remove("prediction_control_resume");
        assert!(control_settings(&serde_json::from_value::<Args>(config.clone())?).is_err());
        config["prediction_control_resume"] = json!("sealed-control32");
        config["prediction_control_trainable"] = json!("joint");
        assert!(control_settings(&serde_json::from_value::<Args>(config.clone())?).is_err());
        config["prediction_control_trainable"] = json!("unknown");
        assert!(serde_json::from_value::<Args>(config.clone()).is_err());
        config["mode"] = json!("unknown_control");
        assert!(serde_json::from_value::<Args>(config).is_err());
        Ok(())
    }
    #[test]
    fn phase_weights_conserve_equal_nonempty_phase_mass() -> Result<()> {
        let target = [10, 20, 20, 30, 31];
        let copy = BTreeSet::from([20]);
        let p = episode_loss_weights(&target, &copy, 8, true, LossScope::All)?;
        assert_eq!(p.counts, [1, 2, 2]);
        for phase in 0..3 {
            let sum = p
                .weights
                .iter()
                .zip(&p.phases)
                .filter(|(_, n)| **n == phase)
                .map(|(w, _)| w)
                .sum::<f64>();
            assert!((sum - 1. / 24.).abs() < 1e-14);
        }
        assert!((p.weights.iter().sum::<f64>() - 1. / 8.).abs() < 1e-14);
        Ok(())
    }
    /// The isolation arm removes exactly one thing: later-position credit. The
    /// entry position keeps its own balanced weight, so any difference between
    /// the arms is the later-phase gradient and not an entry reweighting.
    #[test]
    fn entry_only_scope_zeroes_later_credit_and_preserves_entry_weight() -> Result<()> {
        let target = [10, 20, 20, 30, 31];
        let copy = BTreeSet::from([20]);
        let all = episode_loss_weights(&target, &copy, 8, true, LossScope::All)?;
        let only = episode_loss_weights(&target, &copy, 8, true, LossScope::EntryOnly)?;
        assert_eq!(only.phases, all.phases);
        assert_eq!(only.counts, all.counts);
        assert_eq!(only.weights[0], all.weights[0]);
        assert!(only.weights[1..].iter().all(|w| *w == 0.));
        assert!((only.weights.iter().sum::<f64>() - 1. / 24.).abs() < 1e-14);
        // The legacy position-mean policy is untouched by the scope.
        let legacy = episode_loss_weights(&target, &copy, 8, false, LossScope::EntryOnly)?;
        assert!(legacy.weights.iter().all(|w| *w > 0.));
        Ok(())
    }
    #[test]
    fn entry_ceiling_mode_is_a_distinct_zero_update_configuration() -> Result<()> {
        let config = json!({"mode":"entry_ceiling","credit":"clipped","seed":1001,
            "checkpoint":"cp","saved_fit":"fit","categorical":"cat","parent_config":"parent",
            "training_inputs":"input","training_labels":"labels","development_inputs":"input",
            "development_labels":"labels","maximum_seconds":3600,"maximum_report_bytes":1073741824,"out":"new"});
        let a: Args = serde_json::from_value(config.clone())?;
        assert!(a.mode == Mode::EntryCeiling);
        // The ceiling is a zero-update bound, so it must not be reachable through
        // the fit or control modes, and it must not imply a declared dose.
        assert!(a.mode != Mode::Fit && a.mode != Mode::Admission);
        assert!(serde_json::from_value::<Args>(config).is_ok());
        Ok(())
    }
    #[test]
    fn loss_scope_and_dose_default_to_the_retained_campaign() -> Result<()> {
        let config = json!({"mode":"fit","credit":"clipped","seed":1001,
            "checkpoint":"cp","saved_fit":"fit","categorical":"cat","parent_config":"parent",
            "training_inputs":"input","training_labels":"labels","development_inputs":"input",
            "development_labels":"labels","maximum_seconds":3600,"maximum_report_bytes":1073741824,"out":"new"});
        let a: Args = serde_json::from_value(config.clone())?;
        assert_eq!(a.updates, UPDATES);
        assert_eq!(a.loss_scope, LossScope::All);
        assert_eq!(a.loss_scope.name(), "all");
        assert_eq!(
            serde_json::to_value(a.loss_scope)?,
            json!("all"),
            "the default scope must serialize as the declared phase-balanced policy"
        );
        let mut isolated = config;
        isolated["loss_scope"] = json!("entry_only");
        isolated["updates"] = json!(64);
        let a: Args = serde_json::from_value(isolated)?;
        assert_eq!(a.loss_scope, LossScope::EntryOnly);
        assert_eq!(a.updates, 64);
        Ok(())
    }
    #[test]
    fn order_is_seeded_complete_and_changes_only_schedule() {
        let a = order(1001, 512);
        assert_eq!(a, order(1001, 512));
        assert_eq!(
            a.iter().copied().collect::<BTreeSet<_>>(),
            (0..512).collect()
        );
        assert_ne!(a, order(1002, 512));
        assert_ne!(order(1002, 512), order(1003, 512));
    }
    #[test]
    fn clip_preserves_tiny_gradient_norm_without_bridge_groups() -> Result<()> {
        let d = Device::Cpu;
        let gradients = BTreeMap::from([(
            "consumer.potential.context_unary".into(),
            Tensor::from_vec(vec![1e-25f32, 2e-25], 2, &d)?,
        )]);
        let (denominator, norm) = clip_denominator(&gradients, &d)?;
        assert!(norm > 2e-25 && norm < 3e-25);
        assert_eq!(denominator.to_scalar::<f32>()?, 1.);
        Ok(())
    }
    fn baseline_fixture() -> (Value, Value) {
        let mut base =
            json!({"source_commit":BASELINE_PRODUCER,"credit":"clipped","order_seed":1001});
        for field in [
            "checkpoint_receipt_sha256",
            "parent_config_sha256",
            "training_input_sha256",
            "training_labels_sha256",
            "original_bridge_masters",
            "marker_masters",
            "categorical",
            "initial_active_masters",
            "active_parameter_names",
            "rates",
            "phase_policy",
        ] {
            base[field] = json!({"identity":field});
        }
        base["fresh_adam"] = json!(true);
        let mut checkpoint = json!({"step":0,"native_independently_reloaded":true,"masters_independently_reloaded":true});
        for field in [
            "parent",
            "original_parent",
            "source_metadata_rebound",
            "generate_sha256",
            "original_quarter_sha256",
            "categorical_sha256",
            "categorical_receipt",
        ] {
            checkpoint[field] = json!({"identity":field});
        }
        (base, checkpoint)
    }
    #[test]
    fn baseline_accepts_only_same_initialized_model_independent_of_order_and_credit() -> Result<()>
    {
        let (base, checkpoint) = baseline_fixture();
        let mut current = base.clone();
        current["credit"] = json!("raw_identity");
        current["order_seed"] = json!(1003);
        current["read_state_pullback"] = json!("categorical");
        baseline_identity(&base, &current, &checkpoint, &checkpoint)?;
        current["initial_active_masters"] = json!({"different_context_master":true});
        assert!(baseline_identity(&base, &current, &checkpoint, &checkpoint).is_err());
        let mut changed = checkpoint.clone();
        changed["generate_sha256"] = json!("different");
        assert!(baseline_identity(&base, &base, &checkpoint, &changed).is_err());
        Ok(())
    }
    #[test]
    fn baseline_rejects_unreviewed_producer_or_incomplete_initial_identity() {
        let (mut base, checkpoint) = baseline_fixture();
        base["source_commit"] = json!("unreviewed-producer");
        assert!(baseline_identity(&base, &base, &checkpoint, &checkpoint).is_err());
        base["source_commit"] = json!(BASELINE_PRODUCER);
        base.as_object_mut().map(|m| m.remove("marker_masters"));
        assert!(baseline_identity(&base, &base, &checkpoint, &checkpoint).is_err());
        let (base, mut checkpoint) = baseline_fixture();
        checkpoint["step"] = json!(1);
        assert!(baseline_identity(&base, &base, &checkpoint, &checkpoint).is_err());
    }
}
