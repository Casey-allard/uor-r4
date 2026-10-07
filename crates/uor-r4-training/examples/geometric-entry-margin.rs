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
    geometric_cue_carrier::{CueAngularConfig, CueAngularQ4, CueJointMetadata, CueJointQ4},
    geometric_occurrence_read::{FrameMetadata, FrameStatus, SelectedRecordFrame, SourceIdentity},
    geometric_prefix_transport::{PrefixAngularConfig, PrefixAngularQ4},
    geometric_source_realizer::{
        NativeArtifactBinding, NativeSourceRealizer as IntegerRealizer, SourceBankSegment,
    },
};
use uor_r4_integer::{geometric_vocabulary_actions::NativeVocabularyActions, h4_tables::H4Code};
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
    checkpoint: PathBuf,
    inputs: PathBuf,
    labels: PathBuf,
    out: PathBuf,
    row_indices: Vec<usize>,
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
    let source_commit = option_env!("UOR_BUILD_SOURCE_COMMIT")
        .ok_or_else(|| bad("compiled source identity UNBOUND"))?;
    if source_commit.len() != 40 || !source_commit.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(bad("compiled source identity must be full commit SHA"));
    }
    let output = output_support::prospective_output(&a.out)?;
    for input in [&a.checkpoint, &a.inputs, &a.labels] {
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
