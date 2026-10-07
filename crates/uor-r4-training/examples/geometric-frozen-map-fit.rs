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
    geometric_read_state_bridge::BridgeLearningWeights,
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
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Args {
    mode: Mode,
    credit: Credit,
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
    #[serde(default)]
    baseline: Option<PathBuf>,
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
    if ![1001, 1002, 1003].contains(&a.seed)
        || a.maximum_seconds == 0
        || a.maximum_report_bytes < (64 << 20)
        || a.maximum_report_bytes > (2 << 30)
        || (a.baseline.is_some() && a.mode != Mode::Fit)
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
fn loss_weight_policy(enabled: bool) -> &'static str {
    if enabled {
        "equal-nonempty-phases-per-episode/1;entry-position0;later-exact-allsource-token-membership;empty-phase-zero;remaining-phases-equal;no-runtime-gate"
    } else {
        "legacy-equal-episode-token-mean/1"
    }
}
struct EpisodeLossWeights {
    counts: [usize; 3],
    phases: Vec<usize>,
    weights: Vec<f64>,
}
/// Offline label weighting only, called after actual target-free action admission.
/// Candidate membership does not imply correct-source identity or force Copy.
fn episode_loss_weights(
    target: &[u32],
    admitted_copy_ids: &BTreeSet<u32>,
    batch_episodes: usize,
    balanced: bool,
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
            if balanced {
                1. / nonempty as f64 / counts[phase] as f64 / batch_episodes as f64
            } else {
                // Keep exact prior floating arithmetic and operation order.
                1. / target.len() as f64 / batch_episodes as f64
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
    let mut learner = PreparedBankGenerate::new(&prepared, &l.generate, &gs, &l.exp)?
        .with_prefix_temporal_utility(false)
        .with_read_state_bridge(&l.marker, &bs)?
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
        "credit":a.credit.name(),"token_backward_chunk":1,"frozen_bridge_excluded_from_gradient_accumulation":true}),
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
        "categorical_receipt":l.categorical_receipt,"credit":a.credit.name(),"order_seed":a.seed,
        "native_independently_reloaded":true,"masters_independently_reloaded":true,
        "fresh_adam":"moments zero-initialized; not historical optimizer continuation",
        "frozen_bridge_training":"marker parameters excluded from Adam and clip; original masters unchanged"});
    fs::write(
        root.join("receipt.json"),
        serde_json::to_vec_pretty(&receipt)?,
    )?;
    Ok((integer, generate, bridge, receipt))
}

fn pairs(eps: &[Episode]) -> Result<Vec<(usize, usize, usize)>> {
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
    if pairs.len() != 256 || covered.len() != eps.len() {
        return Err(bad("complete 256 source-swap pairs required"));
    }
    Ok(pairs)
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
    for (i, j, pos) in pairs(eps)? {
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
    // Credit, data-order seed, host and CUDA device are not initialized integer
    // evaluation inputs. Each fit still executes fresh seed-specific graph parity.
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
    let frozen_original = identities(&l.original_bridge.parameters())?;
    let frozen_marker = identities(&l.marker.parameters())?;
    let initial_masters = identities(&active(&l.source, &l.generate)?)?;
    let schedule = order(a.seed, train.len());
    write(
        a,
        "order.json",
        &json!({"seed":a.seed,"order":schedule,
        "policy":"SplitMix64/Fisher-Yates full512 once; fixed cyclic batches of8, 128updates; order only, no model reinitialization"}),
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
        "phase_policy":loss_weight_policy(true),"credit":a.credit.name(),"CUDA_VISIBLE_DEVICES":std::env::var("CUDA_VISIBLE_DEVICES").ok()}),
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
            "updates":0,"credit":a.credit.name(),"order_seed":a.seed,"admission":admission,
            "initial_metrics":initial_metrics,"initial_evaluation_provenance":baseline_provenance,
            "elapsed_seconds":start.elapsed().as_secs_f64(),
            "scope":"zero-update authenticated categorical restart admission; no learning verdict"}),
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
    for update in 0..UPDATES {
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
            json!({"step":update+1,"before_update":receipt,"global_active_gradient_norm":norm}),
        );
        write(a, "updates.json", &json!(updates))?;
        // Recoverable exact checkpoints, not acceptance/reselection gates.
        if (update + 1) % 32 == 0 {
            checkpoint(a, update + 1, &l)?;
        }
    }
    let (native, generate, bridge, final_receipt) = {
        // Final checkpoint was already written at step128; reload all native paths.
        let cp = a.out.join("checkpoint-0128");
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
        "development-0128",
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
    write(a, "metrics-0128.json", &final_metrics)?;
    Ok(
        json!({"schema":"uor-r4.geometric-frozen-map-fit/1","status":"COMPLETED","mode":"fit",
        "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"device":"cuda:0","credit":a.credit.name(),
        "order_seed":a.seed,"updates":UPDATES,"batch":BATCH,"token_backward_chunk":1,
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
            json!({"schema":"uor-r4.geometric-frozen-map-fit/1","status":"FAILED","error":e.to_string(),
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
    fn phase_weights_conserve_equal_nonempty_phase_mass() -> Result<()> {
        let target = [10, 20, 20, 30, 31];
        let copy = BTreeSet::from([20]);
        let p = episode_loss_weights(&target, &copy, 8, true)?;
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
