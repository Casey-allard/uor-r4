//! Offline static-action compilation and a labelled native span replay boundary.
//!
//! Admission binds actual saved model/configuration bytes, original F32 embedding
//! bytes, span semantics, and caller-provided tokenizer identity bytes. Those
//! bytes may describe an explicit toy token registry; equal numerical token IDs
//! are not evidence that two general tokenizers have the same meaning.
//!
//! The token dictionary uses precisely the training `encode_lane` classification
//! of each original F32 embedding lane (its implementation promotes to F64).
//! Zero is the identity action. The existing pinned exact historical H4 payload
//! is reused, with signed coefficient ordering and ALL training products,
//! inverses and identity checked at admission. No new algebra is constructed.
//!
//! The executed dictionary lookup and SpanRegister update use integer state.
//! Controller argmax still reads FLOAT model logits. Canonical F32 reconstruction
//! in `produce_native` is explicitly a replay boundary for the existing reader;
//! it is not whole-model integer serving. There is no autodiff through this path.

use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::Path;

use candle_core::{DType, Tensor};
use safetensors::{Dtype as SafeDtype, SafeTensors};
use serde::{Deserialize, Serialize};
use uor_r4_core::native_geometric::learner::{
    embedding::canonical_h4_roots,
    group_table::{group_table, GROUP_ORDER, ROW_STRIDE},
    prefix_artifact::historical_roots,
};
use uor_r4_integer::geometric_span::{SpanAction, SpanRegister, TokenActionDictionary};
use uor_r4_integer::h4_classifier::H4_ROOT_COEFFICIENTS;
use uor_r4_integer::h4_tables::{
    coefficients_sha256, mathematical_sha256, H4Code, HistoricalH4Tables, PAYLOAD_BYTES,
    TRUSTED_MATHEMATICAL_SHA256,
};

use crate::geometric_address::{encode_lane, geometry_digest};
use crate::geometric_span::{self, GeometricSpanConfig, SpanPolicy};
use crate::{invalid, sha256_bytes, Result};

pub const SCHEMA: &str = "uor-r4.native-span-actions/1";
const CLASSIFICATION: &str = "training-encode_lane-original-f32-promoted-f64;signed-nearest-root;lowest-index-tie;zero-identity/1";
const REPLAY: &str = "old-held;earliest-argmax-f32-controller;integer-dictionary-register;canonical-f32-reconstruction-only/1";
const FILES: [&str; 4] = [
    "metadata.json",
    "token-actions.bin",
    "h4-tables.bin",
    "tokenizer-identity.bin",
];
const PINNED_PAYLOAD: &[u8] =
    include_bytes!("../../uor-r4-integer/fixtures/historical-h4-tables-v1.bin");

/// A source identity obtained from actual files, never caller-asserted digests.
/// Fields are private so load cannot be authorized by self-reported metadata.
pub struct SpanSourceBinding {
    model_sha256: String,
    config_sha256: String,
    embedding_sha256: String,
    vocab_size: usize,
    width: usize,
    embedding: Vec<f32>,
    tokenizer_identity: Vec<u8>,
}

fn embedding_bytes(values: &[f32], vocab_size: usize, width: usize) -> Vec<u8> {
    let mut bytes = b"uor-r4.native-span-embedding-f32/1\0".to_vec();
    bytes.extend_from_slice(&(vocab_size as u64).to_le_bytes());
    bytes.extend_from_slice(&(width as u64).to_le_bytes());
    for value in values {
        bytes.extend_from_slice(&value.to_bits().to_le_bytes());
    }
    bytes
}

fn tensor_embedding(embedding: &Tensor) -> Result<(usize, usize, Vec<f32>)> {
    let (vocab_size, width) = embedding.dims2()?;
    if vocab_size == 0 || embedding.dtype() != DType::F32 || !embedding.device().is_cpu() {
        return Err(invalid(
            "native span compilation requires nonempty CPU F32 embedding[V,W]",
        ));
    }
    GeometricSpanConfig::new(width)?;
    let values = embedding.flatten_all()?.to_vec1::<f32>()?;
    if values.iter().any(|v| !v.is_finite()) {
        return Err(invalid("nonfinite native span embedding"));
    }
    Ok((vocab_size, width, values))
}

impl SpanSourceBinding {
    pub fn from_files(
        model_path: &Path,
        config_path: &Path,
        tokenizer_identity: &[u8],
    ) -> Result<Self> {
        if tokenizer_identity.is_empty() {
            return Err(invalid(
                "native span source requires explicit tokenizer identity bytes",
            ));
        }
        let model = fs::read(model_path)?;
        let config = fs::read(config_path)?;
        let tensors = SafeTensors::deserialize(&model)?;
        let view = tensors.tensor("embedding.weight")?;
        if view.dtype() != SafeDtype::F32 || view.shape().len() != 2 || view.shape()[0] == 0 {
            return Err(invalid(
                "saved native span embedding must be nonempty F32[V,W]",
            ));
        }
        let (vocab_size, width) = (view.shape()[0], view.shape()[1]);
        GeometricSpanConfig::new(width)?;
        let count = vocab_size
            .checked_mul(width)
            .and_then(|n| n.checked_mul(4))
            .ok_or_else(|| invalid("saved native span embedding size overflow"))?;
        if view.data().len() != count {
            return Err(invalid("saved native span embedding byte count differs"));
        }
        let json: serde_json::Value = serde_json::from_slice(&config)?;
        if json.get("width").and_then(|v| v.as_u64()) != Some(width as u64)
            || json.get("vocab_size").and_then(|v| v.as_u64()) != Some(vocab_size as u64)
        {
            return Err(invalid(
                "saved native span model/configuration dimensions differ",
            ));
        }
        let embedding = view
            .data()
            .chunks_exact(4)
            .map(|v| f32::from_le_bytes([v[0], v[1], v[2], v[3]]))
            .collect::<Vec<_>>();
        if embedding.iter().any(|v| !v.is_finite()) {
            return Err(invalid("saved native span embedding is nonfinite"));
        }
        Ok(Self {
            model_sha256: sha256_bytes(&model),
            config_sha256: sha256_bytes(&config),
            embedding_sha256: sha256_bytes(&embedding_bytes(&embedding, vocab_size, width)),
            vocab_size,
            width,
            embedding,
            tokenizer_identity: tokenizer_identity.to_vec(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeSpanMetadata {
    pub schema: String,
    pub model_sha256: String,
    pub config_sha256: String,
    pub embedding_sha256: String,
    pub tokenizer_sha256: String,
    pub tokenizer_bytes: usize,
    pub span: GeometricSpanConfig,
    pub span_config_sha256: String,
    pub vocab_size: usize,
    pub width: usize,
    pub lanes: usize,
    pub dictionary_bytes: usize,
    pub dictionary_sha256: String,
    pub table_bytes: usize,
    pub table_payload_sha256: String,
    pub table_mathematical_sha256: String,
    pub coefficient_sha256: String,
    pub training_geometry_sha256: String,
    pub classification: String,
    pub replay: String,
}

/// Admitted immutable dictionary and exact algebra. Fields cannot be replaced
/// independently of the validation that created this object.
pub struct CompiledSpanActions {
    metadata: NativeSpanMetadata,
    dictionary: TokenActionDictionary,
    tables: HistoricalH4Tables,
    dictionary_bytes: Vec<u8>,
    table_bytes: Vec<u8>,
    tokenizer_identity: Vec<u8>,
}

fn classify(values: &[f32]) -> Result<Vec<u8>> {
    if values.len() % 4 != 0 {
        return Err(invalid("native span embedding lane shape differs"));
    }
    values
        .chunks_exact(4)
        .map(|v| {
            let code = encode_lane([v[0], v[1], v[2], v[3]])?;
            Ok(if code.present {
                code.root
            } else {
                H4Code::IDENTITY.index()
            })
        })
        .collect()
}

fn admit_tables(payload: &[u8]) -> Result<HistoricalH4Tables> {
    let tables = HistoricalH4Tables::from_bytes(payload).map_err(|e| invalid(e.to_string()))?;
    let historical = historical_roots();
    let floating = canonical_h4_roots();
    if historical.len() != GROUP_ORDER || floating.len() != GROUP_ORDER {
        return Err(invalid("native span canonical root count differs"));
    }
    let phi = (1.0 + 5.0f64.sqrt()) / 2.0;
    for i in 0..GROUP_ORDER {
        let root = floating[i].to_array();
        for j in 0..4 {
            for k in 0..2 {
                if historical[i][j][k] != i64::from(H4_ROOT_COEFFICIENTS[i][j][k]) {
                    return Err(invalid("native span historical coefficient order differs"));
                }
            }
            let [a, b] = H4_ROOT_COEFFICIENTS[i][j];
            if (root[j] - (f64::from(a) + f64::from(b) * phi) / 2.0).abs() > 1e-14 {
                return Err(invalid(
                    "native span training root coordinates differ from exact coefficients",
                ));
            }
        }
        if usize::from(encode_lane(root.map(|v| v as f32))?.root) != i {
            return Err(invalid(
                "native span training signed root placement order differs",
            ));
        }
    }
    let training = group_table();
    if tables.identity().index() != training.identity {
        return Err(invalid("native span exact/training group identity differs"));
    }
    for i in 0..GROUP_ORDER {
        let a = H4Code::try_from(i as u8).map_err(|e| invalid(e.to_string()))?;
        if tables.inverse(a).index() != training.inverse[i] {
            return Err(invalid("native span exact/training inverse differs"));
        }
        for j in 0..GROUP_ORDER {
            let b = H4Code::try_from(j as u8).map_err(|e| invalid(e.to_string()))?;
            if tables.compose(a, b).index() != training.product[i * ROW_STRIDE + j] {
                return Err(invalid("native span exact/training product differs"));
            }
        }
    }
    Ok(tables)
}

fn metadata(
    source: &SpanSourceBinding,
    span: &GeometricSpanConfig,
    dictionary: &[u8],
    table: &[u8],
) -> Result<NativeSpanMetadata> {
    span.validate(source.width)?;
    let mathematical = mathematical_sha256(table).map_err(|e| invalid(e.to_string()))?;
    if Some(mathematical.as_str()) != TRUSTED_MATHEMATICAL_SHA256 {
        return Err(invalid(
            "native span exact mathematical trust anchor differs",
        ));
    }
    Ok(NativeSpanMetadata {
        schema: SCHEMA.into(),
        model_sha256: source.model_sha256.clone(),
        config_sha256: source.config_sha256.clone(),
        embedding_sha256: source.embedding_sha256.clone(),
        tokenizer_sha256: sha256_bytes(&source.tokenizer_identity),
        tokenizer_bytes: source.tokenizer_identity.len(),
        span: span.clone(),
        span_config_sha256: sha256_bytes(&serde_json::to_vec(span)?),
        vocab_size: source.vocab_size,
        width: source.width,
        lanes: source.width / 4,
        dictionary_bytes: dictionary.len(),
        dictionary_sha256: sha256_bytes(dictionary),
        table_bytes: table.len(),
        table_payload_sha256: sha256_bytes(table),
        table_mathematical_sha256: mathematical,
        coefficient_sha256: coefficients_sha256(),
        training_geometry_sha256: geometry_digest().to_owned(),
        classification: CLASSIFICATION.into(),
        replay: REPLAY.into(),
    })
}

impl CompiledSpanActions {
    pub fn compile(
        embedding: &Tensor,
        span: &GeometricSpanConfig,
        source: &SpanSourceBinding,
    ) -> Result<Self> {
        let (vocab, width, values) = tensor_embedding(embedding)?;
        span.validate(width)?;
        if vocab != source.vocab_size
            || width != source.width
            || sha256_bytes(&embedding_bytes(&values, vocab, width)) != source.embedding_sha256
        {
            return Err(invalid(
                "native span compile input differs from actual saved embedding",
            ));
        }
        let dictionary_bytes = classify(&values)?;
        let tables = admit_tables(PINNED_PAYLOAD)?;
        let dictionary = TokenActionDictionary::new(width / 4, &dictionary_bytes)
            .map_err(|e| invalid(e.to_string()))?;
        let metadata = metadata(source, span, &dictionary_bytes, PINNED_PAYLOAD)?;
        Ok(Self {
            metadata,
            dictionary,
            tables,
            dictionary_bytes,
            table_bytes: PINNED_PAYLOAD.to_vec(),
            tokenizer_identity: source.tokenizer_identity.clone(),
        })
    }

    pub fn metadata(&self) -> &NativeSpanMetadata {
        &self.metadata
    }
    /// Immutable admitted token actions; no token classification at serving.
    pub fn dictionary(&self) -> &TokenActionDictionary {
        &self.dictionary
    }
    pub fn geometry(&self) -> &HistoricalH4Tables {
        &self.tables
    }
    pub fn vocab_size(&self) -> usize {
        self.dictionary.vocab_size()
    }
    pub fn lanes(&self) -> usize {
        self.dictionary.lanes()
    }

    /// Refuse a stale dictionary after any embedding mutation, even when the
    /// old and new embeddings happen to quantize to the same codes.
    pub fn validate_for(&self, embedding: &Tensor, span: &GeometricSpanConfig) -> Result<()> {
        let (vocab, width, values) = tensor_embedding(embedding)?;
        span.validate(width)?;
        if span != &self.metadata.span
            || vocab != self.metadata.vocab_size
            || width != self.metadata.width
            || sha256_bytes(&embedding_bytes(&values, vocab, width))
                != self.metadata.embedding_sha256
        {
            return Err(invalid(
                "native span dictionary no longer matches live embedding/span",
            ));
        }
        Ok(())
    }

    /// Create a new artifact directory exclusively. The enclosing experiment
    /// owns its report claim/seal; this method never overwrites an old artifact.
    pub fn save(&self, directory: &Path) -> Result<()> {
        fs::create_dir(directory)?;
        let write = |name: &str, bytes: &[u8]| -> Result<()> {
            fs::File::create_new(directory.join(name))?.write_all(bytes)?;
            Ok(())
        };
        write(FILES[1], &self.dictionary_bytes)?;
        write(FILES[2], &self.table_bytes)?;
        write(FILES[3], &self.tokenizer_identity)?;
        write(FILES[0], &serde_json::to_vec_pretty(&self.metadata)?)?;
        Ok(())
    }

    pub fn load(
        directory: &Path,
        source: &SpanSourceBinding,
        span: &GeometricSpanConfig,
    ) -> Result<Self> {
        span.validate(source.width)?;
        let mut names = BTreeSet::new();
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                return Err(invalid("native span artifact contains a non-regular file"));
            }
            names.insert(
                entry
                    .file_name()
                    .into_string()
                    .map_err(|_| invalid("native span artifact filename is not UTF-8"))?,
            );
        }
        if names != FILES.iter().map(|s| s.to_string()).collect() {
            return Err(invalid("native span artifact file set differs"));
        }
        let saved: NativeSpanMetadata =
            serde_json::from_slice(&fs::read(directory.join(FILES[0]))?)?;
        let expected_len = source
            .vocab_size
            .checked_mul(source.width / 4)
            .ok_or_else(|| invalid("native span dictionary size overflow"))?;
        if fs::metadata(directory.join(FILES[1]))?.len() != expected_len as u64
            || fs::metadata(directory.join(FILES[2]))?.len() != PAYLOAD_BYTES as u64
            || fs::metadata(directory.join(FILES[3]))?.len()
                != source.tokenizer_identity.len() as u64
        {
            return Err(invalid("native span artifact payload length differs"));
        }
        let dictionary_bytes = fs::read(directory.join(FILES[1]))?;
        let table_bytes = fs::read(directory.join(FILES[2]))?;
        let tokenizer_identity = fs::read(directory.join(FILES[3]))?;
        if tokenizer_identity != source.tokenizer_identity
            || dictionary_bytes != classify(&source.embedding)?
        {
            return Err(invalid(
                "native span dictionary/tokenizer differs from actual source bytes",
            ));
        }
        let tables = admit_tables(&table_bytes)?;
        let expected = metadata(source, span, &dictionary_bytes, &table_bytes)?;
        if saved != expected {
            return Err(invalid(
                "native span metadata/model/configuration/geometry binding differs",
            ));
        }
        let dictionary = TokenActionDictionary::new(source.width / 4, &dictionary_bytes)
            .map_err(|e| invalid(e.to_string()))?;
        Ok(Self {
            metadata: saved,
            dictionary,
            tables,
            dictionary_bytes,
            table_bytes,
            tokenizer_identity,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeSpanTrace {
    pub batch: usize,
    pub time: usize,
    pub lanes: usize,
    pub actions: Vec<u8>,
    pub prior_codes: Vec<Option<Vec<u8>>>,
}

fn actions(
    ids: &[u32],
    batch: usize,
    time: usize,
    logits: &Tensor,
    compiled: &CompiledSpanActions,
) -> Result<Vec<SpanAction>> {
    let count = batch
        .checked_mul(time)
        .ok_or_else(|| invalid("native span replay shape overflow"))?;
    if batch == 0
        || time == 0
        || ids.len() != count
        || logits.dims() != [batch, time, 4]
        || logits.dtype() != DType::F32
        || !logits.device().is_cpu()
    {
        return Err(invalid(
            "native span replay requires nonempty ids[B*T] and CPU F32 logits[B,T,4]",
        ));
    }
    if ids.iter().any(|&id| id as usize >= compiled.vocab_size()) {
        return Err(invalid(
            "native span replay token ID outside admitted dictionary",
        ));
    }
    let values = logits.flatten_all()?.to_vec1::<f32>()?;
    if values.iter().any(|v| !v.is_finite()) {
        return Err(invalid(
            "native span replay controller logits are nonfinite",
        ));
    }
    values
        .chunks_exact(4)
        .map(|row| {
            let mut selected = 0;
            for i in 1..4 {
                if row[i] > row[selected] {
                    selected = i;
                }
            }
            SpanAction::try_from(selected as u8).map_err(|e| invalid(e.to_string()))
        })
        .collect()
}

/// Observe the integer dictionary/register path. All IDs, shapes and logits
/// are validated before a register is constructed or any state is mutated.
pub fn trace_native(
    ids: &[u32],
    batch: usize,
    time: usize,
    control_logits: &Tensor,
    compiled: &CompiledSpanActions,
) -> Result<NativeSpanTrace> {
    let decisions = actions(ids, batch, time, control_logits, compiled)?;
    trace_native_events(ids, batch, time, &decisions, compiled)
}

/// Direct typed integer events enter the register without a float argmax.
pub fn trace_native_events(
    ids: &[u32],
    batch: usize,
    time: usize,
    decisions: &[SpanAction],
    compiled: &CompiledSpanActions,
) -> Result<NativeSpanTrace> {
    let count = batch
        .checked_mul(time)
        .ok_or_else(|| invalid("native event/span shape overflow"))?;
    if batch == 0
        || time == 0
        || ids.len() != count
        || decisions.len() != count
        || ids.iter().any(|&id| id as usize >= compiled.vocab_size())
    {
        return Err(invalid("native event/span input shape or token differs"));
    }

    let mut register = SpanRegister::new(compiled.lanes()).map_err(|e| invalid(e.to_string()))?;
    let mut trace = NativeSpanTrace {
        batch,
        time,
        lanes: compiled.lanes(),
        actions: Vec::with_capacity(ids.len()),
        prior_codes: Vec::with_capacity(ids.len()),
    };
    for b in 0..batch {
        register.reset();
        for t in 0..time {
            let row = b * time + t;
            trace.prior_codes.push(
                register
                    .held()
                    .map(|codes| codes.iter().map(|code| code.index()).collect()),
            );
            let selected = decisions[row];
            trace.actions.push(selected as u8);
            let token = compiled
                .dictionary
                .row(ids[row] as usize)
                .map_err(|e| invalid(e.to_string()))?;
            register
                .step(selected, token, &compiled.tables)
                .map_err(|e| invalid(e.to_string()))?;
        }
    }
    Ok(trace)
}

/// F32 reconstruction is outside the integer numerical producer. This tensor
/// carries no backward graph and must not be described as full integer serving.
pub fn produce_native(
    ids: &[u32],
    batch: usize,
    time: usize,
    control_logits: &Tensor,
    compiled: &CompiledSpanActions,
) -> Result<Tensor> {
    let trace = trace_native(ids, batch, time, control_logits, compiled)?;
    reconstruct_trace(ids, batch, time, &trace, compiled, control_logits.device())
}

/// Reconstruction remains outside the native event and register kernels.
pub fn produce_native_events(
    ids: &[u32],
    batch: usize,
    time: usize,
    decisions: &[SpanAction],
    compiled: &CompiledSpanActions,
) -> Result<Tensor> {
    let trace = trace_native_events(ids, batch, time, decisions, compiled)?;
    reconstruct_trace(
        ids,
        batch,
        time,
        &trace,
        compiled,
        &candle_core::Device::Cpu,
    )
}

fn reconstruct_trace(
    ids: &[u32],
    batch: usize,
    time: usize,
    trace: &NativeSpanTrace,
    compiled: &CompiledSpanActions,
    device: &candle_core::Device,
) -> Result<Tensor> {
    let roots = canonical_h4_roots();
    let count = ids
        .len()
        .checked_mul(compiled.metadata.width)
        .ok_or_else(|| invalid("native span reconstructed output size overflow"))?;
    let mut output = vec![0f32; count];
    for (row, codes) in trace.prior_codes.iter().enumerate() {
        if let Some(codes) = codes {
            for (lane, &code) in codes.iter().enumerate() {
                let coordinates = roots[usize::from(code)].to_array();
                for i in 0..4 {
                    output[row * compiled.metadata.width + lane * 4 + i] = coordinates[i] as f32;
                }
            }
        }
    }
    Ok(Tensor::from_vec(
        output,
        (batch, time, compiled.metadata.width),
        device,
    )?)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpanReplayComparison {
    pub positions: usize,
    pub lanes_per_position: usize,
    pub reference_present_positions: usize,
    pub native_present_positions: usize,
    pub presence_mismatches: usize,
    pub present_lane_code_mismatches: usize,
    pub max_absolute_reconstruction_error: f32,
}

/// Independent reference uses the existing hard training producer; the native
/// producer never consumes this reference or its codes as execution inputs.
pub fn compare_reference(
    ids: &[u32],
    batch: usize,
    time: usize,
    logits: &Tensor,
    embedding: &Tensor,
    span: &GeometricSpanConfig,
    compiled: &CompiledSpanActions,
) -> Result<SpanReplayComparison> {
    compiled.validate_for(embedding, span)?;
    let native = trace_native(ids, batch, time, logits, compiled)?;
    let indices = Tensor::from_vec(ids.to_vec(), ids.len(), embedding.device())?;
    let selected = embedding
        .index_select(&indices, 0)?
        .reshape((batch, time, span.width))?;
    let reference = geometric_span::produce(&selected, logits, span, SpanPolicy::Ordered)?
        .flatten_all()?
        .to_vec1::<f32>()?;
    let reconstructed = produce_native(ids, batch, time, logits, compiled)?
        .flatten_all()?
        .to_vec1::<f32>()?;
    let mut result = SpanReplayComparison {
        positions: ids.len(),
        lanes_per_position: compiled.lanes(),
        reference_present_positions: 0,
        native_present_positions: 0,
        presence_mismatches: 0,
        present_lane_code_mismatches: 0,
        max_absolute_reconstruction_error: 0.,
    };
    for row in 0..ids.len() {
        let mut reference_codes = Vec::with_capacity(compiled.lanes());
        let mut presence = None;
        for lane in 0..compiled.lanes() {
            let at = row * span.width + lane * 4;
            let code = encode_lane([
                reference[at],
                reference[at + 1],
                reference[at + 2],
                reference[at + 3],
            ])?;
            if presence.is_some_and(|value| value != code.present) {
                return Err(invalid(
                    "hard span reference has inconsistent lane presence",
                ));
            }
            presence = Some(code.present);
            reference_codes.push(code.root);
        }
        let present = presence == Some(true);
        result.reference_present_positions += usize::from(present);
        result.native_present_positions += usize::from(native.prior_codes[row].is_some());
        if present != native.prior_codes[row].is_some() {
            result.presence_mismatches += 1;
        }
        if present {
            if let Some(codes) = &native.prior_codes[row] {
                result.present_lane_code_mismatches += reference_codes
                    .iter()
                    .zip(codes)
                    .filter(|(a, b)| a != b)
                    .count();
            }
        }
    }
    for (a, b) in reference.iter().zip(&reconstructed) {
        result.max_absolute_reconstruction_error =
            result.max_absolute_reconstruction_error.max((a - b).abs());
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::Device;
    use safetensors::tensor::TensorView;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn fixture() -> Result<(PathBuf, Tensor, SpanSourceBinding)> {
        static SERIAL: AtomicU64 = AtomicU64::new(0);
        let clock = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| invalid(e.to_string()))?
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "uor-native-span-{}-{clock}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory)?;
        let mut values = canonical_h4_roots()
            .iter()
            .flat_map(|q| q.to_array().map(|v| v as f32))
            .collect::<Vec<_>>();
        values.extend_from_slice(&[0.; 4]);
        let bytes = values
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect::<Vec<_>>();
        let view = TensorView::new(SafeDtype::F32, vec![121, 4], &bytes)?;
        let saved = safetensors::serialize([("embedding.weight", view)], None)?;
        fs::write(directory.join("model.safetensors"), saved)?;
        fs::write(
            directory.join("config.json"),
            br#"{"vocab_size":121,"width":4,"fixture":"native-span"}"#,
        )?;
        let source = SpanSourceBinding::from_files(
            &directory.join("model.safetensors"),
            &directory.join("config.json"),
            b"test token registry: signed historical roots 0..119; zero row 120",
        )?;
        Ok((
            directory,
            Tensor::from_vec(values, (121, 4), &Device::Cpu)?,
            source,
        ))
    }

    fn logits(actions: &[u8], batch: usize, time: usize) -> Result<Tensor> {
        let values = actions
            .iter()
            .flat_map(|&action| {
                (0..4).map(move |i| {
                    if i == usize::from(action) {
                        1f32
                    } else {
                        -1f32
                    }
                })
            })
            .collect::<Vec<_>>();
        Ok(Tensor::from_vec(values, (batch, time, 4), &Device::Cpu)?)
    }

    #[test]
    fn native_span_export_admits_pinned_algebra_and_every_signed_token_root() -> Result<()> {
        let (directory, embedding, source) = fixture()?;
        let span = GeometricSpanConfig::new(4)?;
        let compiled = CompiledSpanActions::compile(&embedding, &span, &source)?;
        for i in 0..120 {
            assert_eq!(
                compiled
                    .dictionary
                    .row(i)
                    .map_err(|e| invalid(e.to_string()))?[0]
                    .index(),
                i as u8
            );
        }
        assert_eq!(
            compiled
                .dictionary
                .row(120)
                .map_err(|e| invalid(e.to_string()))?[0],
            H4Code::IDENTITY
        );
        assert_eq!(
            compiled.metadata.table_mathematical_sha256,
            TRUSTED_MATHEMATICAL_SHA256.unwrap()
        );
        assert_eq!(
            compiled.metadata.training_geometry_sha256,
            geometry_digest()
        );
        assert_ne!(
            compiled.metadata.training_geometry_sha256,
            compiled.metadata.table_mathematical_sha256
        );
        assert_eq!(compiled.metadata.dictionary_bytes, 121);
        assert_eq!(compiled.metadata.table_bytes, PAYLOAD_BYTES);
        compiled.validate_for(&embedding, &span)?;
        let mut changed = embedding.flatten_all()?.to_vec1::<f32>()?;
        changed[0] += 1e-5; // Small enough to retain its code; still stale bytes.
        let changed = Tensor::from_vec(changed, (121, 4), &Device::Cpu)?;
        assert!(compiled.validate_for(&changed, &span).is_err());
        assert!(CompiledSpanActions::compile(&changed, &span, &source).is_err());
        fs::remove_dir_all(directory)?;
        Ok(())
    }

    #[test]
    fn native_span_replay_matches_order_empty_identity_and_every_old_held_row() -> Result<()> {
        let (directory, embedding, source) = fixture()?;
        let span = GeometricSpanConfig::new(4)?;
        let compiled = CompiledSpanActions::compile(&embedding, &span, &source)?;
        let a = u32::from(encode_lane([0., 1., 0., 0.])?.root);
        let b = u32::from(encode_lane([0., 0., 1., 0.])?.root);
        let anchor = u32::from(encode_lane([0.5, 0.5, 0.5, 0.5])?.root);
        let tail = u32::from(encode_lane([0., 0., 0., 1.])?.root);
        let id = u32::from(H4Code::IDENTITY.index());
        let ids = [
            id, anchor, a, b, tail, id, id, id, anchor, b, a, tail, id, id,
        ];
        let z = logits(&[1, 2, 2, 2, 2, 3, 0, 1, 2, 2, 2, 2, 3, 0], 2, 7)?;
        let trace = trace_native(&ids, 2, 7, &z, &compiled)?;
        assert_ne!(trace.prior_codes[6], trace.prior_codes[13]);
        assert_eq!(trace.prior_codes[7], None); // Batch reset.
        let measured = compare_reference(&ids, 2, 7, &z, &embedding, &span, &compiled)?;
        assert_eq!(measured.presence_mismatches, 0);
        assert_eq!(measured.present_lane_code_mismatches, 0);
        assert_eq!(measured.max_absolute_reconstruction_error, 0.);
        assert_eq!(measured.native_present_positions, 2);
        let inverse = u32::from(group_table().inverse[a as usize]);
        let ids = [id, id, id, a, inverse, id, id, id, id, id, 120, id, id];
        let z = logits(&[1, 3, 1, 2, 2, 3, 0, 1, 3, 1, 2, 3, 0], 1, 13)?;
        let trace = trace_native(&ids, 1, 13, &z, &compiled)?;
        assert!(trace.prior_codes[..6].iter().all(Option::is_none));
        for row in 6..13 {
            assert_eq!(trace.prior_codes[row], Some(vec![id as u8]));
        }
        let measured = compare_reference(&ids, 1, 13, &z, &embedding, &span, &compiled)?;
        assert_eq!(measured.max_absolute_reconstruction_error, 0.);
        assert_eq!(measured.presence_mismatches, 0);
        fs::remove_dir_all(directory)?;
        Ok(())
    }

    #[test]
    fn native_span_reload_refuses_rehashed_dictionary_and_changed_bindings() -> Result<()> {
        let (directory, embedding, source) = fixture()?;
        let span = GeometricSpanConfig::new(4)?;
        let compiled = CompiledSpanActions::compile(&embedding, &span, &source)?;
        let artifact = directory.join("compiled");
        compiled.save(&artifact)?;
        assert!(compiled.save(&artifact).is_err());
        let loaded = CompiledSpanActions::load(&artifact, &source, &span)?;
        assert_eq!(loaded.metadata(), compiled.metadata());
        let original_meta = fs::read(artifact.join(FILES[0]))?;
        let original_dictionary = fs::read(artifact.join(FILES[1]))?;
        let mut changed = original_dictionary.clone();
        changed[0] = H4Code::IDENTITY.index();
        fs::write(artifact.join(FILES[1]), &changed)?;
        let mut lied = compiled.metadata.clone();
        lied.dictionary_sha256 = sha256_bytes(&changed);
        fs::write(artifact.join(FILES[0]), serde_json::to_vec(&lied)?)?;
        assert!(CompiledSpanActions::load(&artifact, &source, &span).is_err());
        fs::write(artifact.join(FILES[1]), &original_dictionary)?;
        fs::write(artifact.join(FILES[0]), &original_meta)?;
        let other_tokenizer = SpanSourceBinding::from_files(
            &directory.join("model.safetensors"),
            &directory.join("config.json"),
            b"another token registry with the same numerical IDs",
        )?;
        assert!(CompiledSpanActions::load(&artifact, &other_tokenizer, &span).is_err());
        let config = fs::read(directory.join("config.json"))?;
        fs::write(
            directory.join("config.json"),
            br#"{"vocab_size":121,"width":4,"fixture":"changed"}"#,
        )?;
        let other_config = SpanSourceBinding::from_files(
            &directory.join("model.safetensors"),
            &directory.join("config.json"),
            &source.tokenizer_identity,
        )?;
        assert!(CompiledSpanActions::load(&artifact, &other_config, &span).is_err());
        fs::write(directory.join("config.json"), &config)?;
        let mut unknown: serde_json::Value = serde_json::from_slice(&original_meta)?;
        unknown["ignored_field"] = true.into();
        fs::write(artifact.join(FILES[0]), serde_json::to_vec(&unknown)?)?;
        assert!(CompiledSpanActions::load(&artifact, &source, &span).is_err());
        fs::write(artifact.join(FILES[0]), &original_meta)?;
        fs::write(artifact.join("unbound.bin"), b"extra")?;
        assert!(CompiledSpanActions::load(&artifact, &source, &span).is_err());
        fs::remove_file(artifact.join("unbound.bin"))?;
        let mut table = fs::read(artifact.join(FILES[2]))?;
        table[2] ^= 1;
        fs::write(artifact.join(FILES[2]), table)?;
        assert!(CompiledSpanActions::load(&artifact, &source, &span).is_err());
        fs::remove_dir_all(directory)?;
        Ok(())
    }

    #[test]
    fn native_span_replay_validates_complete_inputs_and_earliest_argmax() -> Result<()> {
        let (directory, embedding, source) = fixture()?;
        let span = GeometricSpanConfig::new(4)?;
        let compiled = CompiledSpanActions::compile(&embedding, &span, &source)?;
        let tied = Tensor::from_vec(
            vec![0f32, 2., 2., 1., 0., 0., 0., 0.],
            (1, 2, 4),
            &Device::Cpu,
        )?;
        let trace = trace_native(&[0, 1], 1, 2, &tied, &compiled)?;
        assert_eq!(trace.actions, vec![1, 0]);
        assert!(trace_native(&[0, 121], 1, 2, &tied, &compiled).is_err());
        assert!(trace_native(&[0], 1, 2, &tied, &compiled).is_err());
        let bad = Tensor::from_vec(
            vec![0f32, 2., 1., 0., 0., 0., f32::NAN, 0.],
            (1, 2, 4),
            &Device::Cpu,
        )?;
        assert!(trace_native(&[0, 1], 1, 2, &bad, &compiled).is_err());
        let bad_type = tied.to_dtype(DType::F64)?;
        assert!(produce_native(&[0, 1], 1, 2, &bad_type, &compiled).is_err());
        assert!(SpanSourceBinding::from_files(
            &directory.join("model.safetensors"),
            &directory.join("config.json"),
            b""
        )
        .is_err());
        fs::remove_dir_all(directory)?;
        Ok(())
    }
}
