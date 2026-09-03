use super::{
    numerics::{Frames, Weights},
    sha256, valid_hex, LoadedResearchReference, NativeError, NativeErrorTag as E, CONTRACT,
    CONTRACT_SHA256, OPERATION, OPERATOR_PROFILE,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectedBinding {
    pub artifact_sha256: String,
    pub contract_sha256: String,
    pub accepted_binding: Value,
    pub operator_profile: String,
    pub export_release_sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceFile {
    pub repository: String,
    pub revision: String,
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportProvenance {
    pub source_revision: String,
    pub exporter_revision: String,
    pub exporter_sources: Vec<SourceFile>,
    pub exporter_runtime: String,
    pub exporter_lock_sha256: String,
    pub release_sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Component {
    pub name: String,
    pub kind: String,
    pub dtype: String,
    pub shape: Vec<u64>,
    pub offset: u64,
    pub bytes: u64,
    pub sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema: String,
    pub name: String,
    pub canonicalization: String,
    pub contract_sha256: String,
    pub operation: String,
    pub operator_profile: String,
    pub source_binding: Value,
    pub export_provenance: ExportProvenance,
    pub components: Vec<Component>,
    pub identity_index: u32,
    pub native_state_sha256: String,
    pub tied_aliases: Value,
}
pub(super) fn contract() -> Result<Value, NativeError> {
    serde_json::from_str(CONTRACT).map_err(|_| NativeError::new(E::UnsupportedManifest))
}
pub(super) fn exact_keys<'a>(v: &Value, keys: impl Iterator<Item = &'a str>) -> bool {
    let Some(o) = v.as_object() else { return false };
    let keys: Vec<_> = keys.collect();
    o.len() == keys.len() && keys.iter().all(|k| o.contains_key(*k))
}
/// ASCII JSON realization defined in #1086, with sorted object keys and
/// explicit lowercase six-byte escapes for all ASCII control characters.
pub(super) fn canonical(v: &Value) -> Result<Vec<u8>, NativeError> {
    fn emit(v: &Value, out: &mut Vec<u8>) -> Result<(), NativeError> {
        match v {
            Value::Null => out.extend(b"null"),
            Value::Bool(x) => out.extend(if *x {
                b"true".as_slice()
            } else {
                b"false".as_slice()
            }),
            Value::Number(n) => {
                let x = n
                    .as_u64()
                    .ok_or_else(|| NativeError::new(E::UnsupportedManifest))?;
                out.extend(x.to_string().bytes());
            }
            Value::String(s) => {
                if !s.is_ascii() {
                    return Err(NativeError::new(E::UnsupportedManifest));
                }
                out.push(b'"');
                for x in s.bytes() {
                    match x {
                        b'"' => out.extend(b"\\\""),
                        b'\\' => out.extend(b"\\\\"),
                        0..=31 => out.extend(format!("\\u{x:04x}").bytes()),
                        _ => out.push(x),
                    }
                }
                out.push(b'"');
            }
            Value::Array(a) => {
                out.push(b'[');
                for (i, x) in a.iter().enumerate() {
                    if i > 0 {
                        out.push(b',')
                    }
                    emit(x, out)?;
                }
                out.push(b']');
            }
            Value::Object(o) => {
                out.push(b'{');
                let sorted: BTreeMap<_, _> = o.iter().collect();
                for (i, (k, x)) in sorted.into_iter().enumerate() {
                    if i > 0 {
                        out.push(b',')
                    }
                    emit(&Value::String(k.clone()), out)?;
                    out.push(b':');
                    emit(x, out)?;
                }
                out.push(b'}');
            }
        };
        Ok(())
    }
    let mut out = Vec::new();
    emit(v, &mut out)?;
    Ok(out)
}
fn err(tag: E) -> NativeError {
    NativeError::new(tag)
}
fn cid(b: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(b).to_hex())
}
fn value_u64(v: &Value) -> Result<u64, NativeError> {
    v.as_u64().ok_or_else(|| err(E::InvalidFrameTable))
}
fn flattened(v: &Value, out: &mut Vec<u64>) -> Result<(), NativeError> {
    if let Some(a) = v.as_array() {
        for x in a {
            flattened(x, out)?;
        }
    } else {
        out.push(value_u64(v)?);
    }
    Ok(())
}
fn state_cid(components: &[Component], payload: &[u8], owner: &str) -> Result<String, NativeError> {
    let mut h = blake3::Hasher::new();
    for c in components.iter().filter(|c| c.name.starts_with(owner)) {
        let name = &c.name[owner.len()..];
        let meta = json!({"name":name,"shape":c.shape,"dtype":"torch.float32"});
        h.update(&canonical(&meta)?);
        h.update(b"\n");
        h.update(part(payload, c)?);
    }
    Ok(format!("blake3:{}", h.finalize().to_hex()))
}
fn part<'a>(payload: &'a [u8], c: &Component) -> Result<&'a [u8], NativeError> {
    let start = usize::try_from(c.offset).map_err(|_| err(E::InvalidComponent))?;
    let len = usize::try_from(c.bytes).map_err(|_| err(E::InvalidComponent))?;
    payload
        .get(
            start
                ..start
                    .checked_add(len)
                    .ok_or_else(|| err(E::InvalidComponent))?,
        )
        .ok_or_else(|| NativeError::at(E::InvalidComponent, &c.name, start))
}
fn f32s(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect()
}
fn f64s(bytes: &[u8]) -> Vec<f64> {
    bytes
        .chunks_exact(8)
        .map(|b| f64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]))
        .collect()
}
fn indices(bytes: &[u8], name: &str) -> Result<Vec<usize>, NativeError> {
    bytes
        .chunks_exact(8)
        .enumerate()
        .map(|(i, b)| {
            let x = i64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
            if !(0..120).contains(&x) {
                Err(NativeError::at(E::InvalidFrameTable, name, i))
            } else {
                Ok(x as usize)
            }
        })
        .collect()
}
fn native_state(
    components: &[Component],
    payload: &[u8],
    identity: u32,
) -> Result<String, NativeError> {
    fn l(h: &mut Sha256, s: &str) {
        h.update((s.len() as u32).to_le_bytes());
        h.update(s.as_bytes());
    }
    let mut h = Sha256::new();
    l(&mut h, "uor-r4.native-reference-state/1");
    h.update((components.len() as u32).to_le_bytes());
    for c in components {
        l(&mut h, &c.name);
        l(&mut h, &c.kind);
        l(&mut h, &c.dtype);
        h.update((c.shape.len() as u32).to_le_bytes());
        for d in &c.shape {
            h.update(d.to_le_bytes());
        }
        h.update(c.bytes.to_le_bytes());
        h.update(part(payload, c)?);
    }
    h.update(identity.to_le_bytes());
    l(&mut h, OPERATOR_PROFILE);
    Ok(hex::encode(h.finalize()))
}
#[derive(Debug, Default, Clone, Serialize)]
pub struct ValidationAudit {
    pub stages: Vec<&'static str>,
    pub component_checks: Vec<String>,
    pub partial_model_states: Vec<&'static str>,
}
fn descriptor(
    x: &Component,
    t: &Value,
    tag: E,
    audit: &mut ValidationAudit,
) -> Result<(), NativeError> {
    audit
        .component_checks
        .push(format!("descriptor:{}", x.name));
    if x.dtype != t["dtype"]
        || serde_json::to_value(&x.shape).map_err(|_| err(tag.clone()))? != t["shape"]
    {
        return Err(NativeError::at(tag, &x.name, 0));
    }
    Ok(())
}
pub(super) fn load(
    bytes: Vec<u8>,
    expected: &ExpectedBinding,
) -> Result<LoadedResearchReference, NativeError> {
    load_audited(bytes, expected).0
}
pub(super) fn load_audited(
    bytes: Vec<u8>,
    expected: &ExpectedBinding,
) -> (
    Result<LoadedResearchReference, NativeError>,
    ValidationAudit,
) {
    let mut audit = ValidationAudit::default();
    let result = load_impl(bytes, expected, &mut audit);
    (result, audit)
}
fn load_impl(
    bytes: Vec<u8>,
    expected: &ExpectedBinding,
    audit: &mut ValidationAudit,
) -> Result<LoadedResearchReference, NativeError> {
    audit.stages.push("CONTAINER_LIMIT");
    if bytes.len() > 16 * 1024 * 1024 {
        return Err(err(E::ContainerLimit));
    }
    audit.stages.push("INVALID_CONTAINER");
    if bytes.len() < 20 || &bytes[..8] != b"R4LR0001" {
        return Err(err(E::InvalidContainer));
    }
    let ml = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;
    if ml > 256 * 1024 {
        return Err(err(E::ContainerLimit));
    }
    let manifest_end = 12usize
        .checked_add(ml)
        .ok_or_else(|| err(E::InvalidContainer))?;
    let lenbytes = bytes
        .get(manifest_end..manifest_end + 8)
        .ok_or_else(|| err(E::InvalidContainer))?;
    let pl = u64::from_le_bytes([
        lenbytes[0],
        lenbytes[1],
        lenbytes[2],
        lenbytes[3],
        lenbytes[4],
        lenbytes[5],
        lenbytes[6],
        lenbytes[7],
    ]);
    let payload_start = manifest_end + 8;
    let plen = usize::try_from(pl).map_err(|_| err(E::InvalidContainer))?;
    if payload_start.checked_add(plen) != Some(bytes.len()) {
        return Err(err(E::InvalidContainer));
    }
    audit.stages.push("ARTIFACT_IDENTITY_MISMATCH");
    let artifact_sha256 = sha256(&bytes);
    if !valid_hex(&expected.artifact_sha256, 64) || expected.artifact_sha256 != artifact_sha256 {
        return Err(err(E::ArtifactIdentityMismatch));
    }
    audit.stages.push("UNSUPPORTED_MANIFEST");
    let mb = &bytes[12..manifest_end];
    let mv: Value = serde_json::from_slice(mb).map_err(|_| err(E::UnsupportedManifest))?;
    if canonical(&mv)? != mb {
        return Err(err(E::UnsupportedManifest));
    }
    let m: Manifest = serde_json::from_value(mv).map_err(|_| err(E::UnsupportedManifest))?;
    if m.schema != "uor-r4.native-reference-manifest/1"
        || m.name != "R4LearnedReferenceV1"
        || m.canonicalization != "ascii-json-1086/1"
        || m.operation != OPERATION
    {
        return Err(err(E::UnsupportedManifest));
    }
    audit.stages.push("UNSUPPORTED_PROFILE");
    if !cfg!(all(target_arch = "aarch64", target_os = "macos"))
        || m.operator_profile != OPERATOR_PROFILE
        || expected.operator_profile != OPERATOR_PROFILE
    {
        return Err(err(E::UnsupportedProfile));
    }
    audit.stages.push("SOURCE_BINDING_MISMATCH");
    let c = contract()?;
    if m.contract_sha256 != CONTRACT_SHA256
        || expected.contract_sha256 != CONTRACT_SHA256
        || m.source_binding != expected.accepted_binding
        || m.source_binding != c["accepted_binding"]
        || m.export_provenance.release_sha256 != expected.export_release_sha256
        || !valid_hex(&expected.export_release_sha256, 64)
    {
        return Err(err(E::SourceBindingMismatch));
    }
    let p = &m.export_provenance;
    if !valid_hex(&p.source_revision, 40)
        || !valid_hex(&p.exporter_revision, 40)
        || !valid_hex(&p.exporter_lock_sha256, 64)
        || p.exporter_runtime.is_empty()
        || p.exporter_sources.is_empty()
    {
        return Err(err(E::SourceBindingMismatch));
    }
    let mut previous = "";
    for s in &p.exporter_sources {
        if s.path.as_str() <= previous
            || s.path.starts_with('/')
            || s.path
                .split('/')
                .any(|x| x.is_empty() || x == ".." || x == ".")
            || s.repository.is_empty()
            || !valid_hex(&s.revision, 40)
            || !valid_hex(&s.sha256, 64)
        {
            return Err(err(E::SourceBindingMismatch));
        }
        previous = &s.path;
    }
    audit.stages.push("INVALID_COMPONENT");
    let templates = c["components"]
        .as_array()
        .ok_or_else(|| err(E::UnsupportedManifest))?;
    if m.components.len() != 21 || plen != 2_160_742 {
        return Err(err(E::InvalidComponent));
    }
    let payload = &bytes[payload_start..];
    for (x, t) in m.components.iter().zip(templates) {
        audit.component_checks.push(format!("layout:{}", t["name"]));
        if x.name != t["name"]
            || x.kind != t["kind"]
            || Some(x.offset) != t["offset"].as_u64()
            || Some(x.bytes) != t["bytes"].as_u64()
            || !valid_hex(&x.sha256, 64)
            || sha256(part(payload, x)?) != x.sha256
        {
            return Err(NativeError::at(
                E::InvalidComponent,
                t["name"].as_str().unwrap_or(""),
                x.offset as usize,
            ));
        }
    }
    audit.stages.push("INVALID_TENSOR");
    for (x, t) in m.components[..14].iter().zip(templates) {
        audit.component_checks.push(format!("tensor:{}", x.name));
        let owner = if x.name.starts_with("reader.") {
            "reader"
        } else {
            "core"
        };
        if !audit.partial_model_states.contains(&owner) {
            audit.partial_model_states.push(owner);
        }
        if x.dtype != "f32le"
            || serde_json::to_value(&x.shape).map_err(|_| err(E::InvalidTensor))? != t["shape"]
        {
            return Err(NativeError::at(E::InvalidTensor, &x.name, 0));
        }
        for (i, v) in f32s(part(payload, x)?).into_iter().enumerate() {
            if !v.is_finite() {
                return Err(NativeError::at(E::InvalidTensor, &x.name, i));
            }
        }
    }
    if m.tied_aliases != c["tied_aliases"] {
        return Err(err(E::InvalidTensor));
    }
    // Original JSON components have immutable trusted hashes; their declared
    // byte vectors still require the exact dtype/shape before interpretation.
    audit.stages.push("INVALID_CODEC_POLICY");
    descriptor(
        &m.components[14],
        &templates[14],
        E::InvalidCodecPolicy,
        audit,
    )?;
    let vocabulary_bytes = part(payload, &m.components[14])?;
    if sha256(vocabulary_bytes) != m.source_binding["assets"]["vocabulary"]["sha256"]
        || cid(vocabulary_bytes) != m.source_binding["assets"]["vocabulary"]["cid"]
    {
        return Err(err(E::InvalidCodecPolicy));
    }
    let voc: Value =
        serde_json::from_slice(vocabulary_bytes).map_err(|_| err(E::InvalidCodecPolicy))?;
    descriptor(
        &m.components[15],
        &templates[15],
        E::InvalidCodecPolicy,
        audit,
    )?;
    let policy_bytes = part(payload, &m.components[15])?;
    if sha256(policy_bytes) != m.source_binding["policy_sha256"] {
        return Err(err(E::InvalidCodecPolicy));
    }
    let policy: Value =
        serde_json::from_slice(policy_bytes).map_err(|_| err(E::InvalidCodecPolicy))?;
    let prefix = policy["lexical_artifact"]["reader_prefix_by_id"]
        .as_array()
        .ok_or_else(|| err(E::InvalidCodecPolicy))?;
    let vocabulary: Vec<String> = serde_json::from_value(voc["core_vocabulary"].clone())
        .map_err(|_| err(E::InvalidCodecPolicy))?;
    let reader: Vec<String> = serde_json::from_value(voc["vocabulary"].clone())
        .map_err(|_| err(E::InvalidCodecPolicy))?;
    if vocabulary.len() != 4096
        || reader.len() != 4096
        || prefix.len() != 58
        || voc["padding_id"] != 57
    {
        return Err(err(E::InvalidCodecPolicy));
    }
    for i in 0..4096 {
        let expected_core = if i < 52 {
            prefix[i]
                .as_str()
                .ok_or_else(|| err(E::InvalidCodecPolicy))?
                .to_owned()
        } else {
            format!("<unused-{i:04}>")
        };
        let expected_reader = if i < 58 {
            prefix[i]
                .as_str()
                .ok_or_else(|| err(E::InvalidCodecPolicy))?
                .to_owned()
        } else {
            expected_core.clone()
        };
        if vocabulary[i] != expected_core || reader[i] != expected_reader {
            return Err(NativeError::at(E::InvalidCodecPolicy, "vocabulary.json", i));
        }
    }
    audit.stages.push("INVALID_FRAME_TABLE");
    descriptor(
        &m.components[16],
        &templates[16],
        E::InvalidFrameTable,
        audit,
    )?;
    let hb = part(payload, &m.components[16])?;
    if sha256(hb) != m.source_binding["assets"]["h4_frames"]["sha256"]
        || cid(hb) != m.source_binding["assets"]["h4_frames"]["cid"]
    {
        return Err(err(E::InvalidFrameTable));
    }
    let h: Value = serde_json::from_slice(hb).map_err(|_| err(E::InvalidFrameTable))?;
    descriptor(
        &m.components[17],
        &templates[17],
        E::InvalidFrameTable,
        audit,
    )?;
    let tb = part(payload, &m.components[17])?;
    if sha256(tb) != m.source_binding["assets"]["token_frames"]["sha256"]
        || cid(tb) != m.source_binding["assets"]["token_frames"]["cid"]
    {
        return Err(err(E::InvalidFrameTable));
    }
    let t: Value = serde_json::from_slice(tb).map_err(|_| err(E::InvalidFrameTable))?;
    if m.identity_index >= 120
        || h["identity_index"] != m.identity_index
        || t["identity_index"] != m.identity_index
    {
        return Err(err(E::InvalidFrameTable));
    }
    descriptor(
        &m.components[18],
        &templates[18],
        E::InvalidFrameTable,
        audit,
    )?;
    let matrices = f64s(part(payload, &m.components[18])?);
    let mut matrix_bits = Vec::new();
    flattened(&h["frame_matrix_f64_bits"], &mut matrix_bits)?;
    if matrix_bits.len() != 1920 {
        return Err(err(E::InvalidFrameTable));
    }
    for (i, (v, b)) in matrices.iter().zip(matrix_bits).enumerate() {
        if !v.is_finite() || v.to_bits() != b {
            return Err(NativeError::at(E::InvalidFrameTable, "frames", i));
        }
    }
    audit.component_checks.push("numeric:frames".to_owned());
    descriptor(
        &m.components[19],
        &templates[19],
        E::InvalidFrameTable,
        audit,
    )?;
    let multiplication = indices(part(payload, &m.components[19])?, "multiplication")?;
    let mut table = Vec::new();
    flattened(&h["multiplication_indices"], &mut table)?;
    if table.len() != multiplication.len() {
        return Err(err(E::InvalidFrameTable));
    }
    for (i, (a, b)) in multiplication.iter().zip(table).enumerate() {
        if *a as u64 != b {
            return Err(NativeError::at(E::InvalidFrameTable, "multiplication", i));
        }
    }
    audit
        .component_checks
        .push("numeric:multiplication".to_owned());
    descriptor(
        &m.components[20],
        &templates[20],
        E::InvalidFrameTable,
        audit,
    )?;
    let leaves = indices(part(payload, &m.components[20])?, "token_leaves")?;
    let mut leaf_table = Vec::new();
    flattened(&t["token_leaf_indices"], &mut leaf_table)?;
    if leaf_table.len() != leaves.len() {
        return Err(err(E::InvalidFrameTable));
    }
    for (i, (a, b)) in leaves.iter().zip(leaf_table).enumerate() {
        if *a as u64 != b {
            return Err(NativeError::at(E::InvalidFrameTable, "token_leaves", i));
        }
    }
    audit
        .component_checks
        .push("numeric:token_leaves".to_owned());
    if leaves[0] != m.identity_index as usize {
        return Err(err(E::InvalidFrameTable));
    }
    let witnesses = t["prefix_witnesses"]
        .as_array()
        .ok_or_else(|| err(E::InvalidFrameTable))?;
    if witnesses.len() != 3 {
        return Err(err(E::InvalidFrameTable));
    }
    for w in witnesses {
        let tokens = w["tokens"]
            .as_array()
            .ok_or_else(|| err(E::InvalidFrameTable))?;
        let indices = w["frame_indices"]
            .as_array()
            .ok_or_else(|| err(E::InvalidFrameTable))?;
        if tokens.is_empty() || tokens.len() > 8 || tokens.len() != indices.len() {
            return Err(err(E::InvalidFrameTable));
        }
        let mut frame = m.identity_index as usize;
        for (token, index) in tokens.iter().zip(indices) {
            let tid = value_u64(token)?;
            let leaf = leaves
                .get(tid as usize)
                .ok_or_else(|| err(E::InvalidFrameTable))?;
            frame = multiplication[frame * 120 + leaf];
            if frame as u64 != value_u64(index)? {
                return Err(err(E::InvalidFrameTable));
            }
        }
    }
    // File CIDs were checked against immutable trusted originals above. This
    // reproduces the accepted frame directory's two-record identity recipe.
    let tree = json!([{"path":"h4-frames.json","bytes":hb.len(),"cid":cid(hb)},{"path":"token-frames.json","bytes":tb.len(),"cid":cid(tb)}]);
    let mut tree_bytes = canonical(&tree)?;
    tree_bytes.push(b'\n');
    if cid(&tree_bytes) != m.source_binding["frame_tree_cid"] {
        return Err(err(E::InvalidFrameTable));
    }
    audit.stages.push("STATE_IDENTITY_MISMATCH");
    if state_cid(&m.components, payload, "reader.")? != m.source_binding["reader_state_cid"]
        || state_cid(&m.components, payload, "core.")? != m.source_binding["core_state_cid"]
        || !valid_hex(&m.native_state_sha256, 64)
        || native_state(&m.components, payload, m.identity_index)? != m.native_state_sha256
    {
        return Err(err(E::StateIdentityMismatch));
    }
    let w =
        |i: usize| -> Result<Vec<f32>, NativeError> { Ok(f32s(part(payload, &m.components[i])?)) };
    let weights = Weights {
        reader_context_bias: w(0)?,
        reader_context_weight: w(1)?,
        reader_embedding_weight: w(2)?,
        reader_role_projection_bias: w(3)?,
        reader_role_projection_weight: w(4)?,
        core_embedding_weight: w(5)?,
        core_key_projection_weight: w(6)?,
        core_null_key: w(7)?,
        core_null_value: w(8)?,
        core_output_norm_bias: w(9)?,
        core_output_norm_weight: w(10)?,
        core_output_projection_weight: w(11)?,
        core_query_projection_weight: w(12)?,
        core_value_projection_weight: w(13)?,
    };
    let frames = Frames {
        matrices,
        multiplication,
        token_leaves: leaves,
        identity: m.identity_index as usize,
    };
    Ok(LoadedResearchReference {
        artifact: bytes,
        manifest: m,
        weights,
        frames,
        vocabulary,
        artifact_sha256,
        qualification: None,
        busy: AtomicBool::new(false),
    })
}
