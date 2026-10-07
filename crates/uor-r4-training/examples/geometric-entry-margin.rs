//! Zero-update actual-checkpoint entry-credit diagnostic. No fit or runtime change.
use candle_core::{Device, Tensor, Var};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
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
    geometric_cue_carrier::{
        CueAngularConfig, CueAngularQ4, CueJointMetadata, CueJointQ4, NativeCueCarrier,
    },
    geometric_occurrence_read::{FrameMetadata, FrameStatus, SelectedRecordFrame, SourceIdentity},
    geometric_prefix_transport::{NativePrefixTransport, PrefixAngularConfig, PrefixAngularQ4},
    geometric_source_realizer::{
        NativeArtifactBinding, NativeSourceRealizer as IntegerRealizer, SourceBankSegment,
    },
};
use uor_r4_integer::{
    geometric_source_actions::SourceActionBinding,
    geometric_vocabulary_actions::NativeVocabularyActions, h4_tables::H4Code,
};
#[path = "../../uor-r4-integer/examples/support/source_probe.rs"]
mod output_support;
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::{
    geometric_bank_generate::PreparedBankGenerate,
    geometric_generate_learning::{GenerateLearningWeights, VocabularyScoreAdjoint},
    geometric_occurrence_consumer::{
        source_realizer::{NativeSourceRealizer, SourceRealizerWeights},
        ConsumerIdentity,
    },
    geometric_read_state_bridge::BridgeLearningWeights,
    sha256_bytes,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn bad(s: &str) -> Box<dyn std::error::Error> {
    io::Error::new(io::ErrorKind::InvalidData, s).into()
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    #[serde(default)]
    mode: DiagnosticMode,
    #[serde(default)]
    interchange: Option<InterchangeConfig>,
    checkpoint: PathBuf,
    inputs: PathBuf,
    labels: PathBuf,
    out: PathBuf,
    row_indices: Vec<usize>,
}
#[derive(Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum DiagnosticMode {
    #[default]
    EntryCredit,
    SourceStateInterchange,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InterchangeConfig {
    saved_fit: PathBuf,
    categorical: PathBuf,
    pairs: Vec<[usize; 2]>,
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
fn read(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
#[cfg(feature = "cuda")]
fn device() -> Result<Device> {
    Ok(Device::new_cuda(0)?)
}
#[cfg(not(feature = "cuda"))]
fn device() -> Result<Device> {
    Err(bad("CUDA feature/device required; no CPU fallback"))
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
fn scalar_max_abs(t: &Tensor) -> Result<f64> {
    if t.elem_count() == 0 {
        return Ok(0.);
    }
    let maximum = t.flatten_all()?.abs()?.max(0)?.to_scalar::<f32>()? as f64;
    if !maximum.is_finite() {
        return Err(bad("nonfinite gradient maximum"));
    }
    Ok(maximum)
}
fn projected_direction(name: &str, var: &Var, g: &Tensor) -> Result<Tensor> {
    if name.ends_with("prototype_choices") {
        return Ok(g.neg()?);
    }
    let upper = var.ge(1.75f64)?.to_dtype(g.dtype())?;
    let lower = var.le(-1.75f64)?.to_dtype(g.dtype())?;
    let outward_upper = g.lt(0f64)?.to_dtype(g.dtype())?;
    let outward_lower = g.gt(0f64)?.to_dtype(g.dtype())?;
    let blocked = ((&upper * &outward_upper)? + (&lower * &outward_lower)?)?;
    Ok((g.neg()? * (blocked.neg()? + 1f64)?)?)
}
fn normalized(t: &Tensor, maximum: f64) -> Result<Tensor> {
    // Division by a representable observed f32 maximum avoids casting a huge
    // reciprocal into f32 for tiny gradients. Only reduced scalars cross device.
    Ok(t.broadcast_div(&Tensor::new(maximum as f32, t.device())?)?)
}
fn direction_comparison(
    grad: &candle_core::backprop::GradStore,
    margin: &candle_core::backprop::GradStore,
    vars: &BTreeMap<String, Var>,
) -> Result<Value> {
    let mut tensors = Vec::new();
    let (mut gmax, mut dmax, mut mmax) = (0f64, 0f64, 0f64);
    let mut connected = 0usize;
    for (name, var) in vars {
        let g = match grad.get(var.as_tensor()) {
            Some(g) => {
                connected += 1;
                g.clone()
            }
            None => var.zeros_like()?,
        };
        let m = margin
            .get(var.as_tensor())
            .cloned()
            .unwrap_or(var.zeros_like()?);
        let direction = projected_direction(name, var, &g)?;
        gmax = gmax.max(scalar_max_abs(&g)?);
        dmax = dmax.max(scalar_max_abs(&direction)?);
        mmax = mmax.max(scalar_max_abs(&m)?);
        tensors.push((g, direction, m));
    }
    let (mut gn, mut dn, mut mn, mut cross) = (0f64, 0f64, 0f64, 0f64);
    for (g, direction, m) in &tensors {
        if gmax > 0. {
            gn += normalized(g, gmax)?.sqr()?.sum_all()?.to_scalar::<f32>()? as f64;
        }
        if dmax > 0. {
            let nd = normalized(direction, dmax)?;
            dn += nd.sqr()?.sum_all()?.to_scalar::<f32>()? as f64;
            if mmax > 0. {
                cross += (normalized(m, mmax)? * &nd)?
                    .sum_all()?
                    .to_scalar::<f32>()? as f64;
            }
        }
        if mmax > 0. {
            mn += normalized(m, mmax)?.sqr()?.sum_all()?.to_scalar::<f32>()? as f64;
        }
    }
    if ![gn, dn, mn, cross].iter().all(|x| x.is_finite()) {
        return Err(bad("nonfinite normalized gradient comparison"));
    }
    let mut out = json!({
        "connected_parameter_tensors": connected,
        "parameter_tensors": vars.len(),
        "gradient_max_abs": gmax,
        "squared_norm": gmax * gmax * gn,
        "projected_direction_max_abs": dmax,
        "projected_squared_norm": dmax * dmax * dn,
        "margin_gradient_max_abs": mmax,
        "margin_squared_norm": mmax * mmax * mn,
        "scope": "local surrogate projected descent; normalized scalar reductions; no update/native improvement"
    });
    if dmax == 0. || dn == 0. {
        out["status"] = json!("INACTIVE_DIRECTION");
    } else if mmax == 0. || mn == 0. {
        out["status"] = json!("INACTIVE_MARGIN_REFERENCE");
    } else {
        out["status"] = json!("ACTIVE");
        out["margin_directional_derivative"] = json!(mmax * dmax * cross);
        out["margin_unit_direction_derivative"] = json!(mmax * cross / dn.sqrt());
        out["signed_cosine_against_margin_gradient"] = json!(cross / (mn * dn).sqrt());
    }
    Ok(out)
}
fn gradient_receipt(
    grad: &candle_core::backprop::GradStore,
    vars: &BTreeMap<String, Var>,
) -> Result<Value> {
    let mut out = BTreeMap::new();
    for (name, var) in vars {
        out.insert(
            name.clone(),
            direction_comparison(grad, grad, &BTreeMap::from([(name.clone(), var.clone())]))?,
        );
    }
    Ok(serde_json::to_value(out)?)
}
fn projected_alignment(
    left: &candle_core::backprop::GradStore,
    right: &candle_core::backprop::GradStore,
    vars: &BTreeMap<String, Var>,
) -> Result<Value> {
    let mut pairs = Vec::new();
    let (mut lmax, mut rmax) = (0f64, 0f64);
    for (name, var) in vars {
        let l = left
            .get(var.as_tensor())
            .cloned()
            .unwrap_or(var.zeros_like()?);
        let r = right
            .get(var.as_tensor())
            .cloned()
            .unwrap_or(var.zeros_like()?);
        let l = projected_direction(name, var, &l)?;
        let r = projected_direction(name, var, &r)?;
        lmax = lmax.max(scalar_max_abs(&l)?);
        rmax = rmax.max(scalar_max_abs(&r)?);
        pairs.push((l, r));
    }
    if lmax == 0. || rmax == 0. {
        return Ok(json!({"status":"INACTIVE_DIRECTION"}));
    }
    let (mut ln, mut rn, mut cross) = (0f64, 0f64, 0f64);
    for (l, r) in pairs {
        let l = normalized(&l, lmax)?;
        let r = normalized(&r, rmax)?;
        ln += l.sqr()?.sum_all()?.to_scalar::<f32>()? as f64;
        rn += r.sqr()?.sum_all()?.to_scalar::<f32>()? as f64;
        cross += (l * r)?.sum_all()?.to_scalar::<f32>()? as f64;
    }
    if ![ln, rn, cross].iter().all(|x| x.is_finite()) {
        return Err(bad("nonfinite projected direction alignment"));
    }
    if ln == 0. || rn == 0. {
        return Ok(json!({"status":"INACTIVE_DIRECTION"}));
    }
    Ok(json!({
        "status":"ACTIVE",
        "projected_old_raw_signed_cosine":cross/(ln*rn).sqrt(),
        "scope":"global group Euclidean projected-direction alignment; near-equality is not a quality gate"
    }))
}
fn credit_comparisons(
    old: &candle_core::backprop::GradStore,
    raw: &candle_core::backprop::GradStore,
    margin: &candle_core::backprop::GradStore,
    vars: &BTreeMap<String, Var>,
) -> Result<Value> {
    let mut groups = BTreeMap::new();
    groups.insert("joint.Generate_and_potential".to_string(), vars.clone());
    for (prefix, group) in [
        ("generate.", "family.Generate"),
        ("consumer.potential.", "family.potential"),
    ] {
        groups.insert(
            group.to_string(),
            vars.iter()
                .filter(|(name, _)| name.starts_with(prefix))
                .map(|(name, var)| (name.clone(), var.clone()))
                .collect(),
        );
    }
    for (name, var) in vars {
        groups.insert(
            format!("parameter.{name}"),
            BTreeMap::from([(name.clone(), var.clone())]),
        );
    }
    let mut out = BTreeMap::new();
    for (name, group) in groups {
        out.insert(
            name,
            json!({
                "clipped": direction_comparison(old, margin, &group)?,
                "raw_identity": direction_comparison(raw, margin, &group)?,
                "preclip_margin": direction_comparison(margin, margin, &group)?,
                "clipped_raw_projected_alignment": projected_alignment(old, raw, &group)?,
            }),
        );
    }
    Ok(serde_json::to_value(out)?)
}
fn cue(root: &Path) -> Result<CueAngularQ4> {
    let m = read(&root.join("native-metadata.json"))?;
    let c: CueAngularConfig = serde_json::from_value(m["potential"].clone())?;
    let mut q = CueAngularQ4::new(c, &fs::read(root.join("cue-q4.bin"))?)?;
    if let Some(j) = m.get("joint") {
        let jm: CueJointMetadata = serde_json::from_value(j.clone())?;
        let joint = CueJointQ4::new(jm.config, fs::read(root.join("cue-joint-q4.bin"))?)?;
        if joint.metadata() != jm {
            return Err(bad("cue joint identity differs"));
        }
        q = q.with_joint(joint)?;
    } else if root.join("cue-joint-q4.bin").exists() {
        return Err(bad("unreceipted cue joint"));
    }
    Ok(q)
}
fn source_copy_control(
    binding: &SourceActionBinding,
    lanes: usize,
) -> Result<NativeGeometricReadStateBridge> {
    let bias = vec![0u8; lanes * 60];
    let mut relative = vec![0u8; lanes * 120 * 64];
    for lane in 0..lanes {
        for action in 0..120 {
            let at = (lane * 120 + action) * 128 + action;
            relative[at >> 1] |= 7 << if at & 1 == 0 { 0 } else { 4 };
        }
    }
    Ok(NativeGeometricReadStateBridge::compile(
        binding, lanes, &bias, &relative,
    )?)
}
fn first_divergence(a: &[u32], b: &[u32]) -> Result<usize> {
    a.iter()
        .zip(b)
        .position(|(x, y)| x != y)
        .filter(|&p| p > 0)
        .ok_or_else(|| {
            bad("paired answers require nonempty common prefix and differing next tokens")
        })
}
fn canonical_answer(label: &Label, tok: &ByteBpeTokenizer, eos: u32) -> Result<Vec<u32>> {
    label.answers.validate()?;
    let answer = label
        .answers
        .accepted
        .first()
        .ok_or_else(|| bad("empty accepted answer"))?;
    let mut ids = tok.encode(answer);
    if ids.is_empty() || tok.decode_bytes(&ids) != answer.as_bytes() {
        return Err(bad("canonical answer roundtrip differs"));
    }
    ids.push(eos);
    Ok(ids)
}
fn saved_row(fit: &Path, refs: &[Value], index: usize, id: &str) -> Result<(Value, String)> {
    let entry = refs
        .get(index)
        .ok_or_else(|| bad("saved row index unavailable"))?;
    let name = entry["row_file"]
        .as_str()
        .ok_or_else(|| bad("saved row file absent"))?;
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || ".-_".contains(c))
    {
        return Err(bad("unsafe saved row filename"));
    }
    let bytes = fs::read(fit.join(name))?;
    let hash = sha256_bytes(&bytes);
    let row: Value = serde_json::from_slice(&bytes)?;
    if entry["id"] != id || row["id"] != id || entry["row_sha256"] != hash {
        return Err(bad("saved row hash/ID differs"));
    }
    Ok((row, hash))
}
fn candidate_identity(candidate: &Value) -> Result<(u64, u64, u64, u64)> {
    let get = |v: &Value, key: &str| {
        v[key]
            .as_u64()
            .ok_or_else(|| bad("candidate identity absent"))
    };
    Ok((
        get(candidate, "segment_index")?,
        get(candidate, "event")?,
        get(&candidate["occurrence"], "record")?,
        get(&candidate["occurrence"], "commit")?,
    ))
}
fn interchange_keys(
    candidates: &[Value],
    ids: &[u32],
    target: u32,
    factual: usize,
) -> Result<Value> {
    if candidates.len() != ids.len() || factual >= ids.len() {
        return Err(bad("interchange candidate shape/factual ordinal differs"));
    }
    for (candidate, &id) in candidates.iter().zip(ids) {
        if candidate["occurrence"]["token_id"].as_u64() != Some(id as u64) {
            return Err(bad("interchange candidate token inventory differs"));
        }
    }
    let matching = ids
        .iter()
        .enumerate()
        .filter_map(|(i, &t)| (t == target).then_some(i))
        .collect::<Vec<_>>();
    let records = matching
        .iter()
        .map(|&i| candidate_identity(&candidates[i]))
        .collect::<Result<BTreeSet<_>>>()?;
    let unavailable = if records.is_empty() {
        "UNAVAILABLE_TARGET_NOT_IN_COPY"
    } else {
        "AMBIGUOUS_TARGET_TOKEN_IN_MULTIPLE_PHYSICAL_RECORDS"
    };
    let mut keys = vec![
        json!({"name":"factual_raw_max","status":"AVAILABLE","ordinal":factual,
        "selection_scope":"target-free earliest raw Copy maximum"}),
    ];
    if records.len() == 1 {
        let record = *records
            .iter()
            .next()
            .ok_or_else(|| bad("target record absent"))?;
        let target_index = matching
            .first()
            .copied()
            .ok_or_else(|| bad("target occurrence absent"))?;
        let mut same = None;
        let mut other = None;
        for (i, (&id, candidate)) in ids.iter().zip(candidates).enumerate() {
            let identity = candidate_identity(candidate)?;
            if identity == record && id != target && same.is_none() {
                same = Some(i);
            }
            if identity != record && other.is_none() {
                other = Some(i);
            }
        }
        for (name, ordinal, missing) in [
            (
                "diagnostic_target_token",
                Some(target_index),
                "UNAVAILABLE_TARGET_NOT_IN_COPY",
            ),
            (
                "diagnostic_same_record_wrong_offset",
                same,
                "UNAVAILABLE_NO_DIFFERENT_TOKEN_IN_RECORD",
            ),
            (
                "diagnostic_distractor_record",
                other,
                "UNAVAILABLE_NO_DISTRACTOR_RECORD",
            ),
        ] {
            keys.push(match ordinal {
                Some(i) => json!({"name":name,"status":"AVAILABLE","ordinal":i,
                    "selection_scope":"label-assisted diagnostic intervention only; earliest physical occurrence"}),
                None => json!({"name":name,"status":missing}),
            });
        }
    } else {
        for name in [
            "diagnostic_target_token",
            "diagnostic_same_record_wrong_offset",
            "diagnostic_distractor_record",
        ] {
            keys.push(json!({"name":name,"status":unavailable}));
        }
    }
    Ok(json!({"all_matching_target_ordinals":matching,
        "target_physical_records":records.iter().map(|(segment,event,record,commit)|
            json!({"segment_index":segment,"event":event,"record":record,"commit":commit})).collect::<Vec<_>>(),
        "arms":keys,"source_truth_scope":"unique token-bearing physical record only; ambiguous membership is not resolved by a hidden oracle"}))
}
fn apply_interchange(
    bridge: Option<&NativeGeometricReadStateBridge>,
    query: &[H4Code],
    key: &[H4Code],
) -> Result<(Vec<H4Code>, Vec<H4Code>, BridgeReadCounts, Vec<i64>)> {
    let mut post = query.to_vec();
    let mut actions = vec![H4Code::IDENTITY; query.len()];
    let mut scores = vec![0i64; query.len() * 120];
    let mut counts = BridgeReadCounts::default();
    if let Some(bridge) = bridge {
        bridge.apply_into(
            query,
            key,
            &mut post,
            &mut actions,
            &mut scores,
            &mut counts,
        )?;
    }
    Ok((post, actions, counts, scores))
}
fn native_interchange_arm(
    bridge: Option<&NativeGeometricReadStateBridge>,
    source_copy: bool,
    query: &[H4Code],
    key: &[H4Code],
    generate: &NativeGeometricGenerate,
    pool: &mut NativeVocabularyActions,
    copy_ids: &[u32],
    copy: &[i64],
    target: u32,
) -> Result<Value> {
    let (post, actions, transport_counts, action_scores) = apply_interchange(bridge, query, key)?;
    if source_copy && post != key {
        return Err(bad(
            "source-copy positive control failed exact q*(inverse(q)*k)=k",
        ));
    }
    let mut scores = vec![0i64; generate.vocab_size()];
    let mut costs = GenerateReadCounts::default();
    generate.score_into(&post, &mut scores, &mut costs)?;
    let trace = pool.reduce_trace(&scores, copy_ids, copy)?;
    let mass = trace
        .token_masses
        .iter()
        .find(|m| m.token_id == target)
        .ok_or_else(|| bad("interchange target not admitted by vocabulary"))?;
    let score = *scores
        .get(target as usize)
        .ok_or_else(|| bad("target beyond Generate vocabulary"))?;
    Ok(json!({
        "query_state_codes":query.iter().map(|c| c.index()).collect::<Vec<_>>(),
        "source_state_codes":key.iter().map(|c| c.index()).collect::<Vec<_>>(),
        "action_codes":actions.iter().map(|c| c.index()).collect::<Vec<_>>(),
        "poststate_codes":post.iter().map(|c| c.index()).collect::<Vec<_>>(),
        "action_scores_q24_sha256":sha256_bytes(&serde_json::to_vec(&action_scores)?),
        "source_copy_exact_poststate":source_copy.then_some(true),
        "generate_raw_scores_sha256":sha256_bytes(&serde_json::to_vec(&scores)?),
        "target_generate_raw_q24":score,
        "target_generate_mass_q31":mass.generate_weight_q31,
        "target_copy_mass_q31":mass.copy_weight_q31,
        "target_mass_q31":mass.weight_q31,"denominator_q31":trace.summary.total_weight_q31,
        "target_probability_offline":mass.weight_q31 as f64/trace.summary.total_weight_q31 as f64,
        "pool_summary":trace.summary,"transport_counts":transport_counts,"generate_costs":costs,
        "scope":"frozen Copy/query/Generate; diagnostic source-key/bridge interchange; no learned routing or argmax admission gate"
    }))
}
#[allow(clippy::too_many_arguments)]
fn source_state_interchange(
    a: &Config,
    integer: &IntegerRealizer,
    tok: &ByteBpeTokenizer,
    generate: &NativeGeometricGenerate,
    original: &NativeGeometricReadStateBridge,
    bridge_masters: &BridgeLearningWeights,
    learner: &mut PreparedBankGenerate<'_, '_>,
    carrier: &NativeCueCarrier<'_>,
    prefix: &NativePrefixTransport<'_>,
    independent_cue: &NativeCueCarrier<'_>,
    independent_prefix: &NativePrefixTransport<'_>,
    exp: &[u8],
    inputs: &Inputs,
    labels: &Labels,
    inputbytes: &[u8],
    labelbytes: &[u8],
) -> Result<Value> {
    let spec = a
        .interchange
        .as_ref()
        .ok_or_else(|| bad("interchange configuration absent"))?;
    report_output::verify(&spec.saved_fit)?;
    report_output::verify(&spec.categorical)?;
    let fit_bytes = fs::read(spec.saved_fit.join("report.json"))?;
    let fit: Value = serde_json::from_slice(&fit_bytes)?;
    if fit["schema"] != "uor-r4.geometric-bank-generate-fit/1"
        || fit["status"] != "COMPLETED"
        || fit["arm"] != "joint-potential"
        || fit["mode"] != "fit"
    {
        return Err(bad(
            "interchange requires retained completed joint-potential fit",
        ));
    }
    let step = fit["updates"]
        .as_u64()
        .ok_or_else(|| bad("retained fit updates absent"))?;
    let expected_cp = spec.saved_fit.join(format!("checkpoint-{step:04}"));
    if fs::canonicalize(&expected_cp)? != fs::canonicalize(&a.checkpoint)? {
        return Err(bad(
            "interchange checkpoint is not retained final checkpoint",
        ));
    }
    let receipt_bytes = fs::read(a.checkpoint.join("receipt.json"))?;
    let receipt: Value = serde_json::from_slice(&receipt_bytes)?;
    let stages = fit["stages"]
        .as_array()
        .ok_or_else(|| bad("retained fit stages absent"))?;
    let stage = stages
        .last()
        .ok_or_else(|| bad("retained final stage absent"))?;
    if stage["step"] != step || stage["checkpoint"] != receipt {
        return Err(bad("retained final stage/receipt differs"));
    }
    let refs = stage["evaluation"]["rows"]
        .as_array()
        .ok_or_else(|| bad("saved row inventory absent"))?;
    let admission = read(&spec.saved_fit.join("input-admission.json"))?;
    let hashes = admission["input_sha256"]
        .as_object()
        .ok_or_else(|| bad("fit panel hashes absent"))?;
    for bytes in [inputbytes, labelbytes] {
        if !hashes
            .values()
            .any(|v| v.as_str() == Some(sha256_bytes(bytes).as_str()))
        {
            return Err(bad("interchange panel bytes not retained fit inputs"));
        }
    }
    let categorical_bytes = fs::read(spec.categorical.join("read-state-bridge-categorical.bin"))?;
    let categorical =
        NativeGeometricReadStateBridge::from_bytes(&categorical_bytes, integer.binding())?;
    let bm_bytes = fs::read(a.checkpoint.join("read-state-bridge-source/metadata.json"))?;
    let bm: Value = serde_json::from_slice(&bm_bytes)?;
    let probe_bytes = fs::read(spec.categorical.join("probe.json"))?;
    let probe: Value = serde_json::from_slice(&probe_bytes)?;
    let rebuilt = bridge_masters.export_categorical_actions()?;
    let original_bytes = original.to_bytes()?;
    if rebuilt.native.to_bytes()? != categorical_bytes
        || probe["status"] != "COMPLETED"
        || probe["mode"] != "categorical"
        || probe["schema"] != "uor-r4.read-state-export-probe/1"
        || probe["checkpoint_receipt_sha256"] != sha256_bytes(&receipt_bytes)
        || probe["parent_manifest_sha256"]
            != sha256_bytes(&fs::read(spec.saved_fit.join("manifest.json"))?)
        || probe["master_metadata_sha256"] != sha256_bytes(&bm_bytes)
        || probe["master_identities"] != bm["source_parameters"]
        || bm != receipt["read_state_bridge"]
        || probe["original_native_sha256"] != sha256_bytes(&original_bytes)
        || probe["derived_native_sha256"] != sha256_bytes(&categorical_bytes)
        || probe["derived_payload_sha256"] != categorical.metadata().payload_sha256
        || probe["original_coarse_reproduction"] != "BYTE_IDENTICAL"
        || probe["parent_binding"] != receipt["parent"]
        || probe["all_query_frame_mismatches"] != 0
        || probe["all_query_frame_checks"] != generate.lanes() * 120 * 120
    {
        return Err(bad(
            "categorical intervention does not bind original retained masters/export",
        ));
    }
    let source_copy = source_copy_control(integer.binding(), generate.lanes())?;
    let mut pool = NativeVocabularyActions::new(integer.binding().clone(), exp)?;
    let mut rows = Vec::new();
    let mut pair_receipts = Vec::new();
    for pair in &spec.pairs {
        let mut packets = Vec::new();
        let mut targets = Vec::new();
        let mut saved = Vec::new();
        for &index in pair {
            let p = inputs
                .cases
                .get(index)
                .ok_or_else(|| bad("paired input absent"))?;
            let l = labels
                .cases
                .get(index)
                .ok_or_else(|| bad("paired labels absent"))?;
            if p.id != l.id || !p.actual_prefix_ids.is_empty() || p.query_ids.is_empty() {
                return Err(bad("interchange row/prefix/label identity differs"));
            }
            let ids = canonical_answer(l, tok, integer.binding().eos_token_id())?;
            let row = saved_row(&spec.saved_fit, refs, index, &p.id)?;
            if row.0["canonical_target_ids_labels_only"] != json!(ids) {
                return Err(bad("saved canonical targets differ from frozen answer"));
            }
            packets.push(p);
            targets.push(ids);
            saved.push(row);
        }
        if packets[0].query_ids != packets[1].query_ids {
            return Err(bad("paired source interchange queries differ"));
        }
        let position = first_divergence(&targets[0], &targets[1])?;
        let common = &targets[0][..position];
        if &targets[1][..position] != common {
            return Err(bad("paired common prefix differs"));
        }
        pair_receipts.push(json!({"indices":pair,"ids":[packets[0].id,packets[1].id],
            "position":position,"common_prefix_ids":common,"targets_labels_only":[targets[0][position],targets[1][position]],
            "scope":"fixed source-swap construction pair, common canonical prefix, not independent transfer or own-prefix generation"}));
        for (which, &index) in pair.iter().enumerate() {
            let p = packets[which];
            let views = p
                .segments
                .iter()
                .map(|s| {
                    s.frame()
                        .map(|f| integer.compile_view(f.token_ids))
                        .transpose()
                })
                .collect::<std::result::Result<Vec<_>, _>>()?;
            let segments = p
                .segments
                .iter()
                .zip(&views)
                .map(|(s, v)| {
                    Ok(match s {
                        Segment::Source { event, .. } => SourceBankSegment::Source {
                            frame: s.frame().ok_or_else(|| bad("source frame absent"))?,
                            view: v.as_ref().ok_or_else(|| bad("source view absent"))?,
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
                .collect::<Result<Vec<_>>>()?;
            let mut starts = Vec::new();
            let mut base = 0usize;
            for segment in &segments {
                starts.push(base);
                let len = match segment {
                    SourceBankSegment::Source { view, .. } => view.emitted_token_ids().len(),
                    SourceBankSegment::Context { token_ids, .. } => token_ids.len(),
                };
                base = base
                    .checked_add(len)
                    .ok_or_else(|| bad("interchange causal length overflow"))?;
            }
            base = base
                .checked_add(p.query_ids.len())
                .and_then(|n| n.checked_add(common.len()))
                .ok_or_else(|| bad("interchange full causal length overflow"))?;
            if base == 0 || base > 128 {
                return Err(bad("interchange causal bank exceeds1..128"));
            }
            // Common-prefix context is declared teacher forcing. No next-token
            // label or diagnostic occurrence choice enters these factual forwards.
            let independent = integer.read_bank_with_prefix_transport(
                &segments,
                &p.query_ids,
                common,
                independent_cue,
                independent_prefix,
            )?;
            let bank = &independent.cue_bank.bank;
            let query = bank
                .context
                .states
                .last()
                .ok_or_else(|| bad("interchange query state absent"))?
                .iter()
                .copied()
                .map(H4Code::try_from)
                .collect::<std::result::Result<Vec<_>, _>>()?;
            let ids = bank
                .candidates
                .iter()
                .map(|c| c.occurrence.token_id)
                .collect::<Vec<_>>();
            let mut copy = vec![0i64; ids.len()];
            for head in &bank.heads {
                if head.scores_q24.len() != ids.len() {
                    return Err(bad("interchange Copy shape differs"));
                }
                for (v, s) in copy.iter_mut().zip(&head.scores_q24) {
                    *v = v
                        .checked_add(*s)
                        .ok_or_else(|| bad("interchange Copy sum overflow"))?;
                }
            }
            let mut factual = 0usize;
            let mut best = *copy
                .first()
                .ok_or_else(|| bad("interchange bank has no Copy"))?;
            for (i, &s) in copy.iter().enumerate().skip(1) {
                if s > best {
                    factual = i;
                    best = s;
                }
            }
            let mut keys = Vec::new();
            for (i, c) in bank.candidates.iter().enumerate() {
                let v = views
                    .get(c.segment_index)
                    .and_then(|v| v.as_ref())
                    .ok_or_else(|| bad("candidate Source view absent"))?;
                let offset = usize::try_from(c.occurrence.token_offset)?;
                if c.bank_index != i
                    || v.emitted_token_ids().get(offset) != Some(&c.occurrence.token_id)
                    || starts
                        .get(c.segment_index)
                        .and_then(|s| s.checked_add(offset))
                        != Some(c.context_position)
                {
                    return Err(bad("candidate occurrence/chronology differs"));
                }
                match p.segments.get(c.segment_index) {
                    Some(Segment::Source {
                        event,
                        record,
                        commit,
                        ..
                    }) if *event == c.event
                        && *record == c.occurrence.record
                        && *commit == c.occurrence.commit => {}
                    _ => return Err(bad("candidate physical Source identity differs")),
                }
                keys.push(
                    bank.context
                        .states
                        .get(c.context_position)
                        .ok_or_else(|| bad("candidate retained state absent"))?
                        .iter()
                        .copied()
                        .map(H4Code::try_from)
                        .collect::<std::result::Result<Vec<_>, _>>()?,
                );
            }
            let (post, actions, _, action_scores) =
                apply_interchange(Some(original), &query, &keys[factual])?;
            let mut original_scores = vec![0i64; generate.vocab_size()];
            generate.score_into(
                &post,
                &mut original_scores,
                &mut GenerateReadCounts::default(),
            )?;
            let original_pool = pool.reduce_trace(&original_scores, &ids, &copy)?;
            let learning =
                learner.forward_bank(&segments, &p.query_ids, common, carrier, prefix)?;
            let (read, trace) = learning
                .read_state_bridge
                .as_ref()
                .ok_or_else(|| bad("learning bridge absent"))?;
            if *read != factual
                || trace.action_codes != actions
                || trace.action_scores_q24 != action_scores
                || learning.final_state_codes != post
                || learning.copy_token_ids != ids
                || learning.copy_scores_q24 != copy
                || learning.generate.scores_q24 != original_scores
                || serde_json::to_vec(&learning.actions)? != serde_json::to_vec(&original_pool)?
            {
                return Err(bad(
                    "interchange factual independent native/learning parity differs",
                ));
            }
            let diagonal = &saved[which].0["canonical"][position];
            let n = &diagonal["native"];
            let candidates = serde_json::to_value(&bank.candidates)?;
            let source_trace = &n["source_provenance"]["read_state_bridge"];
            if n["actual_prefix_ids"] != json!(common)
                || n["retained_state_codes"]
                    != json!(post.iter().map(|c| c.index()).collect::<Vec<_>>())
                || n["copy_token_ids"] != json!(ids)
                || n["copy_raw_scores_q24"] != json!(copy)
                || n["generate_raw_scores_sha256"]
                    != sha256_bytes(&serde_json::to_vec(&original_scores)?)
                || n["pool"]["summary"] != serde_json::to_value(&original_pool.summary)?
                || n["source_provenance"]["candidates"] != candidates
                || n["source_provenance"]["causal_tokens"] != json!(bank.context.tokens)
                || source_trace["selected_ordinal"] != factual
                || source_trace["action_codes"]
                    != json!(actions.iter().map(|c| c.index()).collect::<Vec<_>>())
                || source_trace["action_scores_q24"] != json!(action_scores)
                || source_trace["query_state_codes"]
                    != json!(query.iter().map(|c| c.index()).collect::<Vec<_>>())
                || source_trace["selected_source_state_codes"]
                    != json!(keys[factual].iter().map(|c| c.index()).collect::<Vec<_>>())
            {
                return Err(bad("interchange available saved diagonal differs"));
            }
            // Factual native parity and saved diagonal are complete before any
            // target-assisted source or token-position intervention is selected.
            let target = targets[which][position];
            let target_mass = original_pool
                .token_masses
                .iter()
                .find(|m| m.token_id == target)
                .ok_or_else(|| bad("saved target mass absent"))?;
            if diagonal["target_label_only"] != target
                || diagonal["native_target_mass"] != target_mass.weight_q31
                || diagonal["native_denominator"] != original_pool.summary.total_weight_q31
            {
                return Err(bad("available saved target mass differs"));
            }
            let key_plan = interchange_keys(
                candidates
                    .as_array()
                    .ok_or_else(|| bad("candidate inventory shape"))?,
                &ids,
                target,
                factual,
            )?;
            let mut experiments = Vec::new();
            for key_arm in key_plan["arms"]
                .as_array()
                .ok_or_else(|| bad("key arms absent"))?
            {
                let Some(ordinal) = key_arm["ordinal"]
                    .as_u64()
                    .map(usize::try_from)
                    .transpose()?
                else {
                    experiments.push(json!({"key":key_arm,"status":key_arm["status"]}));
                    continue;
                };
                let key = keys
                    .get(ordinal)
                    .ok_or_else(|| bad("interchange ordinal out of range"))?;
                let mut controls = BTreeMap::new();
                for (name, control, is_source_copy) in [
                    ("original_quarter", Some(original), false),
                    ("identity_bypass", None, false),
                    ("categorical_master_map", Some(&categorical), false),
                    ("exact_source_copy", Some(&source_copy), true),
                ] {
                    let mut result = native_interchange_arm(
                        control,
                        is_source_copy,
                        &query,
                        key,
                        generate,
                        &mut pool,
                        &ids,
                        &copy,
                        target,
                    )?;
                    result["bridge_native_sha256"] = match control {
                        Some(c) => json!(sha256_bytes(&c.to_bytes()?)),
                        None => Value::Null,
                    };
                    controls.insert(name.to_string(), result);
                }
                experiments.push(json!({"key":key_arm,"candidate":candidates[ordinal],
                    "source_state_codes":key.iter().map(|c|c.index()).collect::<Vec<_>>(),"controls":controls}));
            }
            let mut contrasts = Vec::new();
            if let Some(reference) = experiments.first() {
                for experiment in experiments.iter().skip(1) {
                    if experiment["controls"].is_null() {
                        continue;
                    }
                    for name in [
                        "original_quarter",
                        "identity_bypass",
                        "categorical_master_map",
                        "exact_source_copy",
                    ] {
                        let now = &experiment["controls"][name];
                        let old = &reference["controls"][name];
                        let raw = now["target_generate_raw_q24"]
                            .as_i64()
                            .ok_or_else(|| bad("arm raw score absent"))?
                            .checked_sub(
                                old["target_generate_raw_q24"]
                                    .as_i64()
                                    .ok_or_else(|| bad("reference raw score absent"))?,
                            )
                            .ok_or_else(|| bad("raw contrast overflow"))?;
                        let delta = |field: &str| -> Result<String> {
                            let a =
                                now[field].as_u64().ok_or_else(|| bad("arm mass absent"))? as i128;
                            let b = old[field]
                                .as_u64()
                                .ok_or_else(|| bad("reference mass absent"))?
                                as i128;
                            Ok((a - b).to_string())
                        };
                        contrasts.push(json!({"control":name,"key":experiment["key"]["name"],
                            "reference_key":"factual_raw_max","target_generate_raw_delta_q24":raw,
                            "target_generate_mass_delta_q31_signed":delta("target_generate_mass_q31")?,
                            "target_mass_delta_q31_signed":delta("target_mass_q31")?,
                            "target_probability_delta_offline":now["target_probability_offline"].as_f64().ok_or_else(||bad("probability absent"))?
                                -old["target_probability_offline"].as_f64().ok_or_else(||bad("reference probability absent"))?,
                            "source_state_changed":now["source_state_codes"]!=old["source_state_codes"],
                            "physical_occurrence_changed":experiment["key"]["ordinal"]!=reference["key"]["ordinal"],
                            "action_changed":now["action_codes"]!=old["action_codes"],
                            "poststate_changed":now["poststate_codes"]!=old["poststate_codes"],
                            "generate_scores_changed":now["generate_raw_scores_sha256"]!=old["generate_raw_scores_sha256"]}));
                    }
                }
            }
            rows.push(json!({"index":index,"id":p.id,"position":position,"actual_common_prefix_ids":common,
                "query_ids":p.query_ids,"target_label_only":target,"causal_tokens":base,
                "independent_factual_native_parity":true,"available_saved_diagonal_parity":true,
                "saved_row_sha256":saved[which].1,"candidate_inventory":candidates,
                "candidate_inventory_sha256":sha256_bytes(&serde_json::to_vec(&bank.candidates)?),
                "copy_token_ids":ids,"frozen_copy_scores_q24":copy,"key_plan":key_plan,
                "distinct_all_candidate_source_states":keys.iter().collect::<BTreeSet<_>>().len(),
                "query_frame_sha256":sha256_bytes(&serde_json::to_vec(&query.iter().map(|c|c.index()).collect::<Vec<_>>())?),
                "source_key_scope":"cumulative retained H4 state AFTER this physical occurrence context_position; not a raw-token or isolated-word embedding; equal keys do not identify equal occurrences",
                "experiments":experiments,"contrasts_vs_factual":contrasts}));
        }
    }
    Ok(
        json!({"schema":"uor-r4.source-state-interchange/1","updates":0,
        "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"device":"cuda:0",
        "inputs_sha256":sha256_bytes(inputbytes),"labels_sha256":sha256_bytes(labelbytes),
        "retained_fit_report_sha256":sha256_bytes(&fit_bytes),
        "retained_fit_manifest_sha256":sha256_bytes(&fs::read(spec.saved_fit.join("manifest.json"))?),
        "checkpoint_receipt_sha256":sha256_bytes(&receipt_bytes),
        "generate_sha256":sha256_bytes(&generate.to_bytes()?),
        "original_quarter_sha256":sha256_bytes(&original_bytes),
        "categorical_sha256":sha256_bytes(&categorical_bytes),
        "categorical_probe_sha256":sha256_bytes(&probe_bytes),
        "categorical_rebuild_receipt":rebuilt.receipt,
        "categorical_scope":"explicit retained-master action-map intervention; original energy magnitudes discarded",
        "source_copy_control_sha256":sha256_bytes(&source_copy.to_bytes()?),
        "positive_control_scope":"exact retained geometric transport only; decoder was not trained for this intervention; no positive predictive control",
        "freeze_scope":"query H4 frame and full Copy inventory/scores fixed within each row; Generate parameters/prototypes frozen, Generate scores allowed to respond to poststate",
        "boundary_scope":"source-key distinction, bridge-action/poststate distinction, frozen-decoder consumption measured separately within same factual query; categorical map may be noninjective even when byte-correct; cross-pair frame differences are not source-key interchange evidence",
        "pairs":pair_receipts,"rows":rows,
        "scope":"fixed construction first-source-word interchange; common canonical prefix; label-assisted key interventions after factual native parity; frozen Copy/query/Generate; no fit, argmax gate, own-prefix output, transfer, attention qualification or serving-policy adoption"}),
    )
}
fn run(a: &Config, d: &Device) -> Result<Value> {
    let cp = &a.checkpoint;
    let receipt = read(&cp.join("receipt.json"))?;
    let binding: NativeArtifactBinding = serde_json::from_value(receipt["parent"].clone())?;
    let integer = IntegerRealizer::load_native(&cp.join("native"), &binding)?;
    let tokbytes = fs::read(cp.join("native/tokenizer.json"))?;
    let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokbytes)
        .ok_or_else(|| bad("ByteBPE unavailable"))?;
    let source =
        SourceRealizerWeights::load_context_potential_on_device(&cp.join("source"), &tokbytes, d)?;
    let native_meta = read(&cp.join("native/metadata.json"))?;
    let identity: ConsumerIdentity = serde_json::from_value(native_meta["identity"].clone())?;
    let native = NativeSourceRealizer::load(&cp.join("native"), &source, &identity)?;
    if native.artifact_binding()? != binding {
        return Err(bad("source/native binding differs"));
    }
    let generate_bytes = fs::read(cp.join("generate.bin"))?;
    let gen_native = NativeGeometricGenerate::from_bytes(&generate_bytes, integer.binding())?;
    let g = GenerateLearningWeights::from_native(integer.binding().clone(), &gen_native, d)?;
    let gm = read(&cp.join("generate-source/metadata.json"))?;
    if gm["tokenizer_sha256"].as_str() != Some(sha256_bytes(&tokbytes).as_str())
        || gm["protocol"] != serde_json::to_value(integer.binding().protocol())?
        || gm["lanes"].as_u64() != Some(g.lanes() as u64)
    {
        return Err(bad("Generate master binding differs"));
    }
    restore(
        &cp.join("generate-source"),
        &gm["parameters"],
        &g.parameters(),
        d,
    )?;
    let gs = g.prepare_native()?;
    if gs.native.to_bytes()? != generate_bytes {
        return Err(bad("Generate restored export differs"));
    }
    let bridge_bytes = fs::read(cp.join("read-state-bridge.bin"))?;
    let bn = NativeGeometricReadStateBridge::from_bytes(&bridge_bytes, integer.binding())?;
    let bridge = BridgeLearningWeights::from_native(&bn, integer.binding(), d)?;
    let bm = read(&cp.join("read-state-bridge-source/metadata.json"))?;
    if bm["metadata"] != serde_json::to_value(bn.metadata())?
        || bm["native_sha256"].as_str() != Some(sha256_bytes(&bridge_bytes).as_str())
    {
        return Err(bad("bridge master binding differs"));
    }
    restore(
        &cp.join("read-state-bridge-source"),
        &bm["source_parameters"],
        &bridge.parameters(),
        d,
    )?;
    let bs = bridge.prepare_native()?;
    if bs.native.to_bytes()? != bridge_bytes {
        return Err(bad("restored original quarter bridge differs"));
    }
    let prepared = source.prepare_context_potential_on_device(&native, d)?;
    let cq = cue(&cp.join("cue"))?;
    let carrier = native.compile_cue_carrier(cq)?;
    let pm = read(&cp.join("prefix/native-metadata.json"))?;
    let pc: PrefixAngularConfig = serde_json::from_value(pm["potential"].clone())?;
    let prefix = native.compile_prefix_transport(
        &carrier,
        PrefixAngularQ4::new(pc, &fs::read(cp.join("prefix/prefix-q4.bin"))?)?,
    )?;
    if serde_json::to_value(carrier.metadata())? != read(&cp.join("cue/native-metadata.json"))?
        || serde_json::to_value(prefix.metadata())? != pm
    {
        return Err(bad("frozen sidecar metadata differs"));
    }
    let exp = fs::read(cp.join("native/consumer/exp-q31.bin"))?;
    let mut learner = PreparedBankGenerate::new(&prepared, &g, &gs, &exp)?
        .with_prefix_temporal_utility(false)
        .with_read_state_bridge(&bridge, &bs)?
        .with_read_selector_credit(true);
    let independent_cue = integer.compile_cue_carrier(cue(&cp.join("cue"))?)?;
    let independent_prefix = integer.compile_prefix_transport(
        &independent_cue,
        PrefixAngularQ4::new(pc, &fs::read(cp.join("prefix/prefix-q4.bin"))?)?,
    )?;
    let mut independent_pool = NativeVocabularyActions::new(integer.binding().clone(), &exp)?;
    let inputbytes = fs::read(&a.inputs)?;
    let labelbytes = fs::read(&a.labels)?;
    let inputs: Inputs = serde_json::from_slice(&inputbytes)?;
    let labels: Labels = serde_json::from_slice(&labelbytes)?;
    if inputs.schema != "uor-r4.native-source-bank-probe-input/1"
        || labels.schema != "uor-r4.native-source-bank-labels/1"
        || labels.protocol != "uor-r4.literal-role-dialogue/2"
        || !labels.membership_only
        || inputs.cases.len() != labels.cases.len()
        || inputs.cases.is_empty()
    {
        return Err(bad("panel contract differs"));
    }
    let mut all = g.parameters();
    all.extend(bridge.parameters());
    all.extend(source.context_state_parameters());
    all.extend(source.potential_parameters());
    let before = identities(&all)?;
    if a.mode == DiagnosticMode::SourceStateInterchange {
        let report = source_state_interchange(
            a,
            &integer,
            &tok,
            &gen_native,
            &bn,
            &bridge,
            &mut learner,
            &carrier,
            &prefix,
            &independent_cue,
            &independent_prefix,
            &exp,
            &inputs,
            &labels,
            &inputbytes,
            &labelbytes,
        )?;
        let after = identities(&all)?;
        if before != after {
            return Err(bad("interchange parameter masters mutated"));
        }
        let mut report = report;
        report["parameters_before"] = serde_json::to_value(before)?;
        report["parameters_after"] = serde_json::to_value(after)?;
        return Ok(report);
    }
    let mut families = g.parameters();
    families.extend(source.potential_parameters());
    let mut rows = Vec::new();
    for &i in &a.row_indices {
        let p = inputs
            .cases
            .get(i)
            .ok_or_else(|| bad("row index outside panel"))?;
        let l = labels
            .cases
            .get(i)
            .ok_or_else(|| bad("label index outside panel"))?;
        if p.id != l.id
            || !p.actual_prefix_ids.is_empty()
            || p.query_ids.is_empty()
            || p.segments.len() > 128
        {
            return Err(bad("entry/panel identity differs"));
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
        let segments = p
            .segments
            .iter()
            .zip(&views)
            .map(|(s, v)| {
                Ok(match s {
                    Segment::Source { event, .. } => SourceBankSegment::Source {
                        frame: s.frame().ok_or_else(|| bad("frame"))?,
                        view: v.as_ref().ok_or_else(|| bad("view"))?,
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
            .collect::<Result<Vec<_>>>()?;
        let base = segments.iter().try_fold(p.query_ids.len(), |n, segment| {
            let add = match segment {
                SourceBankSegment::Source { view, .. } => view.emitted_token_ids().len(),
                SourceBankSegment::Context { token_ids, .. } => token_ids.len(),
            };
            n.checked_add(add)
                .ok_or_else(|| bad("causal token length overflow"))
        })?;
        if base == 0 || base > 128 {
            return Err(bad("entry causal bank exceeds1..128 tokens"));
        }
        // Independently loaded integer path, no labels and no learning score anchors.
        let independent = integer.read_bank_with_prefix_transport(
            &segments,
            &p.query_ids,
            &[],
            &independent_cue,
            &independent_prefix,
        )?;
        let bank = &independent.cue_bank.bank;
        let mut native_codes = bank
            .context
            .states
            .last()
            .ok_or_else(|| bad("integer final state absent"))?
            .iter()
            .copied()
            .map(H4Code::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let native_ids = bank
            .candidates
            .iter()
            .map(|c| c.occurrence.token_id)
            .collect::<Vec<_>>();
        let mut native_copy = vec![0i64; native_ids.len()];
        for head in &bank.heads {
            if head.scores_q24.len() != native_copy.len() {
                return Err(bad("integer Copy shape differs"));
            }
            for (value, score) in native_copy.iter_mut().zip(&head.scores_q24) {
                *value = value
                    .checked_add(*score)
                    .ok_or_else(|| bad("integer Copy overflow"))?;
            }
        }
        let mut selected = 0usize;
        let mut best = *native_copy
            .first()
            .ok_or_else(|| bad("integer bank has no Copy"))?;
        for (j, &score) in native_copy.iter().enumerate().skip(1) {
            if score > best {
                selected = j;
                best = score
            }
        }
        let candidate = bank
            .candidates
            .get(selected)
            .ok_or_else(|| bad("integer selected occurrence absent"))?;
        let source_codes = bank
            .context
            .states
            .get(candidate.context_position)
            .ok_or_else(|| bad("integer source state absent"))?
            .iter()
            .copied()
            .map(H4Code::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let native_query = native_codes.clone();
        let mut native_actions = vec![H4Code::IDENTITY; native_codes.len()];
        let mut bridge_scores = vec![0i64; native_codes.len() * 120];
        bn.apply_into(
            &native_query,
            &source_codes,
            &mut native_codes,
            &mut native_actions,
            &mut bridge_scores,
            &mut BridgeReadCounts::default(),
        )?;
        let mut native_generate = vec![0i64; gen_native.vocab_size()];
        gen_native.score_into(
            &native_codes,
            &mut native_generate,
            &mut GenerateReadCounts::default(),
        )?;
        let native_pool =
            independent_pool.reduce_trace(&native_generate, &native_ids, &native_copy)?;
        // Complete target-free actual bank forward precedes any answer encoding.
        let out = learner.forward_bank(&segments, &p.query_ids, &[], &carrier, &prefix)?;
        let (actual_selected, actual_bridge) = out
            .read_state_bridge
            .as_ref()
            .ok_or_else(|| bad("learning bridge missing"))?;
        if out.final_state_codes != native_codes
            || out.copy_token_ids != native_ids
            || out.copy_scores_q24 != native_copy
            || out.generate.scores_q24 != native_generate
            || *actual_selected != selected
            || actual_bridge.action_codes != native_actions
            || actual_bridge.action_scores_q24 != bridge_scores
            || serde_json::to_vec(&out.actions)? != serde_json::to_vec(&native_pool)?
        {
            return Err(bad(
                "independent integer bank/bridge/Generate/action parity differs",
            ));
        }

        let before_packet = sha256_bytes(&serde_json::to_vec(&(
            out.actions.clone(),
            out.final_state_codes
                .iter()
                .map(|c| c.index())
                .collect::<Vec<_>>(),
            out.copy_scores_q24.clone(),
        ))?);
        l.answers.validate()?;
        let answer = l
            .answers
            .accepted
            .first()
            .ok_or_else(|| bad("empty accepted answers"))?;
        let ids = tok.encode(answer);
        if tok.decode_bytes(&ids) != answer.as_bytes() {
            return Err(bad("answer roundtrip differs"));
        }
        let target = *ids.first().ok_or_else(|| bad("empty answer target"))?;
        let old = out.loss(target)?;
        let raw = out.loss_with_credit(target, VocabularyScoreAdjoint::RawIdentity)?;
        let diagnostic = out.preclip_entry_margin_diagnostic(target)?;
        let old_value = old.to_scalar::<f32>()?;
        let raw_value = raw.to_scalar::<f32>()?;
        if old_value != raw_value {
            return Err(bad("score-adjoint policies changed native scalar loss"));
        }
        let margin = diagnostic.loss.to_scalar::<f32>()?;
        let old_g = old.backward()?;
        let raw_g = raw.backward()?;
        let new_g = diagnostic.loss.backward()?;
        let after_packet = sha256_bytes(&serde_json::to_vec(&(
            out.actions.clone(),
            out.final_state_codes
                .iter()
                .map(|c| c.index())
                .collect::<Vec<_>>(),
            out.copy_scores_q24.clone(),
        ))?);
        if before_packet != after_packet {
            return Err(bad("native packet mutated by backward"));
        }
        rows.push(json!({"index":i,"id":p.id,"actual_prefix_empty":true,"causal_tokens":base,"independent_integer_parity":true,"target":target,"old_native_marginal_loss":old_value,"raw_identity_native_marginal_loss":raw_value,"preclip_atom_margin":margin,"raw_gap_q24":diagnostic.raw_gap_q24,"wrong_copy_source_offset":diagnostic.wrong_copy_source_offset,"wrong_copy_above_clip":diagnostic.wrong_copy_above_clip,"native_packet_sha256":before_packet,"old_gradients":gradient_receipt(&old_g,&families)?,"raw_identity_gradients":gradient_receipt(&raw_g,&families)?,"margin_gradients":gradient_receipt(&new_g,&families)?,"projected_credit_comparisons":credit_comparisons(&old_g,&raw_g,&new_g,&families)?,"scope":diagnostic.score_scope}));
    }
    let after = identities(&all)?;
    if before != after {
        return Err(bad("parameter masters mutated"));
    }
    Ok(
        json!({"schema":"uor-r4.actual-entry-alias-credit/2","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNBOUND"),"device":"cuda:0","cuda_visible_devices":std::env::var("CUDA_VISIBLE_DEVICES").ok(),"ld_library_path":std::env::var("LD_LIBRARY_PATH").ok(),"cuda_compute_cap":std::env::var("CUDA_COMPUTE_CAP").ok(),"updates":0,"inputs_sha256":sha256_bytes(&inputbytes),"labels_sha256":sha256_bytes(&labelbytes),"generate_sha256":sha256_bytes(&generate_bytes),"original_quarter_bridge_sha256":sha256_bytes(&bridge_bytes),"parameters_before":before,"parameters_after":after,"frozen_cue_prefix_parameter_credit":"excluded: no sidecar variables in trainable inventory; context-state credit can remain","rows":rows,"scope":"source-index-fixed construction diagnostic; no fit, prediction improvement, transfer or clipping-causation verdict"}),
    )
}
fn main() -> Result<()> {
    let arg = std::env::args_os()
        .nth(1)
        .ok_or_else(|| bad("usage: geometric-entry-margin CONFIG.json"))?;
    let mut a: Config = serde_json::from_slice(&fs::read(arg)?)?;
    if a.row_indices.is_empty()
        || a.row_indices.len() > 8
        || a.row_indices.iter().copied().collect::<BTreeSet<_>>().len() != a.row_indices.len()
    {
        return Err(bad("declare1..8 distinct source-only fixed row indices"));
    }
    match (a.mode, a.interchange.as_ref()) {
        (DiagnosticMode::EntryCredit, None) => {}
        (DiagnosticMode::SourceStateInterchange, Some(spec))
            if spec.pairs == [[0, 8], [4, 12]] && a.row_indices == [0, 8, 4, 12] => {}
        _ => return Err(bad("interchange requires fixed pairs0/8,4/12 and indices0,8,4,12; entry mode has no interchange config")),
    }
    let source_commit = option_env!("UOR_BUILD_SOURCE_COMMIT")
        .ok_or_else(|| bad("compiled source identity UNBOUND"))?;
    if source_commit.len() != 40 || !source_commit.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(bad("compiled source identity must be full commit SHA"));
    }
    let output = output_support::prospective_output(&a.out)?;
    let mut input_paths = vec![&a.checkpoint, &a.inputs, &a.labels];
    if let Some(spec) = &a.interchange {
        input_paths.extend([&spec.saved_fit, &spec.categorical]);
    }
    for input in input_paths {
        let canonical = fs::canonicalize(input)?;
        if output.starts_with(&canonical) || canonical.starts_with(&output) {
            return Err(bad("output overlaps checkpoint/panel input"));
        }
    }
    a.out = output;
    let d = device()?;
    report_output::claim(&a.out)?;
    fs::write(
        a.out.join("config.json"),
        serde_json::to_vec_pretty(
            &json!({"checkpoint":a.checkpoint,"inputs":a.inputs,"labels":a.labels,"row_indices":a.row_indices,"rule":"indices declared before model work; no success filtering"}),
        )?,
    )?;
    if let Some(spec) = &a.interchange {
        fs::write(
            a.out.join("interchange-config.json"),
            serde_json::to_vec_pretty(&json!({
                "mode":"source_state_interchange", "saved_fit":spec.saved_fit,
                "categorical":spec.categorical, "pairs":spec.pairs,
                "key_interventions":"factual raw max; diagnostic label-assisted earliest target occurrence, earliest other token in same unique physical record, earliest distractor record; no filtering",
                "prefix_scope":"paired first-divergent canonical prefix; teacher-forced diagnostic, not own-prefix output"
            }))?,
        )?;
    }
    let result = run(&a, &d);
    match result {
        Ok(report) => {
            fs::write(
                a.out.join("report.json"),
                serde_json::to_vec_pretty(&report)?,
            )?;
            report_output::seal(&a.out)?;
            report_output::verify(&a.out)?;
            println!("{}", a.out.display());
            Ok(())
        }
        Err(e) => {
            fs::write(
                a.out.join("failure.json"),
                serde_json::to_vec_pretty(
                    &json!({"error":e.to_string(),"status":"EXECUTION_FAILURE; not model evidence"}),
                )?,
            )?;
            report_output::seal(&a.out)?;
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn old_entry_config_decodes_without_new_mode_fields() -> Result<()> {
        let config: Config = serde_json::from_value(json!({
            "checkpoint":"cp","inputs":"inputs","labels":"labels","out":"out","row_indices":[0,1]
        }))?;
        assert!(config.mode == DiagnosticMode::EntryCredit && config.interchange.is_none());
        assert_eq!(first_divergence(&[1, 2, 3, 4], &[1, 2, 3, 5])?, 3);
        assert!(first_divergence(&[1, 2], &[1, 2]).is_err());
        assert!(first_divergence(&[1, 2], &[3, 2]).is_err());
        Ok(())
    }
    #[test]
    fn interchange_occurrence_rules_keep_duplicates_and_mark_ambiguity() -> Result<()> {
        let candidate = |segment, record, offset, token| {
            json!({
                "segment_index":segment,"event":segment,
                "occurrence":{"record":record,"commit":1,"token_offset":offset,"token_id":token}
            })
        };
        let candidates = vec![
            candidate(1, 10, 0, 7),
            candidate(1, 10, 1, 8),
            candidate(1, 10, 2, 7),
            candidate(3, 20, 0, 9),
            candidate(3, 20, 1, 11),
        ];
        let plan = interchange_keys(&candidates, &[7, 8, 7, 9, 11], 7, 3)?;
        assert_eq!(plan["all_matching_target_ordinals"], json!([0, 2]));
        assert_eq!(plan["arms"][0]["ordinal"], 3);
        assert_eq!(plan["arms"][1]["ordinal"], 0);
        assert_eq!(plan["arms"][2]["ordinal"], 1);
        assert_eq!(plan["arms"][3]["ordinal"], 3);
        let mut ambiguous_candidates = candidates.clone();
        ambiguous_candidates[4]["occurrence"]["token_id"] = json!(7);
        let ambiguous = interchange_keys(&ambiguous_candidates, &[7, 8, 7, 9, 7], 7, 3)?;
        assert_eq!(ambiguous["all_matching_target_ordinals"], json!([0, 2, 4]));
        assert_eq!(
            ambiguous["arms"][1]["status"],
            "AMBIGUOUS_TARGET_TOKEN_IN_MULTIPLE_PHYSICAL_RECORDS"
        );
        assert!(ambiguous["arms"][1]["ordinal"].is_null());
        let absent = interchange_keys(&candidates, &[7, 8, 7, 9, 11], 99, 3)?;
        assert_eq!(
            absent["arms"][1]["status"],
            "UNAVAILABLE_TARGET_NOT_IN_COPY"
        );
        Ok(())
    }
    #[test]
    fn source_copy_control_exactly_transports_signed_query_key_states() -> Result<()> {
        const TOK: &str = r#"{"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":{"<|bos|>":0,"<|eos|>":1,"<|unk|>":2,".":3,"a":4,"b":5},"merges":[]},"added_tokens":[{"id":0,"content":"<|bos|>"},{"id":1,"content":"<|eos|>"},{"id":2,"content":"<|unk|>"}]}"#;
        let binding = SourceActionBinding::new(TOK.as_bytes())?;
        let control = source_copy_control(&binding, 2)?;
        for q in [0u8, 1, 2, 29, 119] {
            for k in [0u8, 1, 2, 47, 119] {
                let query = [H4Code::try_from(q)?, H4Code::try_from((q + 17) % 120)?];
                let key = [H4Code::try_from(k)?, H4Code::try_from((k + 73) % 120)?];
                let (post, _, _, _) = apply_interchange(Some(&control), &query, &key)?;
                assert_eq!(post, key);
                let (identity, actions, _, _) = apply_interchange(None, &query, &key)?;
                assert_eq!(identity, query);
                assert_eq!(actions, vec![H4Code::IDENTITY; 2]);
            }
        }
        Ok(())
    }
    #[test]
    fn scaled_group_comparison_retains_tiny_gradients_and_metric_weights() -> Result<()> {
        let d = Device::Cpu;
        let a = Var::from_vec(vec![0f32], 1, &d)?;
        let b = Var::from_vec(vec![0f32], 1, &d)?;
        let vars = BTreeMap::from([
            ("generate.a".to_string(), a.clone()),
            ("consumer.potential.b".to_string(), b.clone()),
        ]);
        let direction = ((a.as_tensor() * 1e-25f64)? + (b.as_tensor() * 2e-25f64)?)?
            .sum_all()?
            .backward()?;
        let margin = ((a.as_tensor() * 2e-25f64)? + (b.as_tensor() * 1e-25f64)?)?
            .sum_all()?
            .backward()?;
        let result = direction_comparison(&direction, &margin, &vars)?;
        assert_eq!(result["status"], "ACTIVE");
        let cosine = result["signed_cosine_against_margin_gradient"]
            .as_f64()
            .ok_or_else(|| bad("missing cosine"))?;
        assert!((cosine + 0.8).abs() < 1e-6);
        assert!(
            result["squared_norm"]
                .as_f64()
                .ok_or_else(|| bad("missing norm"))?
                > 0.
        );
        let zero = (a.as_tensor() * 0f64)?.sum_all()?.backward()?;
        assert_eq!(
            direction_comparison(&zero, &margin, &vars)?["status"],
            "INACTIVE_DIRECTION"
        );
        Ok(())
    }
}
