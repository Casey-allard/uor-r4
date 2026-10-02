//! Source-bound geometric attention value-to-residual composition.
//! Numerical execution is a separately versioned i64-Q16 kernel; the legacy
//! i32 reader contract is unchanged. Floating tensors reconstruct offline model
//! inputs only after geometric composition and integer reduction.
use crate::geometric_composition::{CompositionConfig, CompositionWeights};
use crate::geometric_context::{BoundFile, CompiledContext};
use crate::geometric_event::CompiledEvents;
use crate::geometric_no_read_native::{CompiledNoRead, NoReadSourcePaths};
use crate::geometric_potential_native::CompiledGeometricPotentials;
use crate::geometric_read_native::CompiledGeometricRead;
use crate::geometric_span_native::CompiledSpanActions;
use crate::geometric_value_native::ValuePacketStatus;
use crate::geometric_value_producer::ValueProducerTrace;
use crate::geometric_value_producer_native::{CompiledValueProducer, ValueParameterIdentity};
use crate::{invalid, sha256_bytes, Result};
use candle_core::{Device, Tensor, Var};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
    path::Path,
};
use uor_r4_integer::{
    geometric_composed_read::NativeGeometricComposedRead,
    geometric_composition::{NativeGeometricComposition, OUTPUT_WIDTH},
    geometric_value::{ValuePacket, ValueState},
};
pub const SCHEMA: &str = "uor-r4.geometric-read-composition-native/1";
const POLICY:&str="H2-input4-output8-K2;left*root*inverse(right);byte-signed-H4-IDs;q4[-7,7]-reserved-minus8-reject;dimensionless-quarter-gain;one-ties-away-quarter-after-all-atoms;separate-i64-Q16;per-head-original-age-exp-NoRead;checked-i128-shift-add-longdivision;head-sum-after-normalization/1";
const FILES: [&str; 5] = [
    "metadata.json",
    "left-roots.bin",
    "right-roots.bin",
    "gains-q4.bin",
    "tokenizer-identity.bin",
];
#[derive(Clone, Copy)]
pub struct CompositionSourcePaths<'a> {
    pub composition_source: &'a Path,
    pub no_read: NoReadSourcePaths<'a>,
    pub no_read_native: &'a Path,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompositionMetadata {
    pub schema: String,
    pub config: CompositionConfig,
    pub policy: String,
    pub source_files: BTreeMap<String, BoundFile>,
    pub parameters: BTreeMap<String, ValueParameterIdentity>,
    pub no_read_metadata: BoundFile,
    pub reducer_metadata: BoundFile,
    pub left: BoundFile,
    pub right: BoundFile,
    pub gains: BoundFile,
    pub tokenizer: BoundFile,
    pub payload_fractional_bits: u32,
    pub payload_storage_bits: u32,
    pub output_width: usize,
    pub actual_bank_bytes: usize,
    pub numerical_composition_schema: String,
    pub numerical_reducer_schema: String,
}
fn bound(bytes: &[u8]) -> BoundFile {
    BoundFile {
        bytes: bytes.len(),
        sha256: sha256_bytes(bytes),
    }
}
fn identities(p: &BTreeMap<String, Var>) -> Result<BTreeMap<String, ValueParameterIdentity>> {
    p.iter()
        .map(|(name, v)| {
            let a = v.flatten_all()?.to_vec1::<f32>()?;
            if a.iter().any(|x| !x.is_finite()) {
                return Err(invalid("nonfinite composition source parameter"));
            }
            let bytes = a.into_iter().flat_map(f32::to_le_bytes).collect::<Vec<_>>();
            Ok((
                name.clone(),
                ValueParameterIdentity {
                    shape: v.dims().to_vec(),
                    f32_bytes_sha256: sha256_bytes(&bytes),
                },
            ))
        })
        .collect()
}
fn snapshot(p: CompositionSourcePaths<'_>) -> Result<BTreeMap<String, BoundFile>> {
    ["metadata.json", "composition-parameters.safetensors"]
        .into_iter()
        .map(|name| {
            Ok((
                name.to_string(),
                bound(&fs::read(p.composition_source.join(name))?),
            ))
        })
        .collect()
}
pub struct CompiledComposition {
    metadata: CompositionMetadata,
    native: NativeGeometricComposition,
    left: Vec<u8>,
    right: Vec<u8>,
    gains: Vec<u8>,
    tokenizer: Vec<u8>,
}
impl CompiledComposition {
    pub fn compile(
        w: &CompositionWeights,
        p: CompositionSourcePaths<'_>,
        tokenizer: &[u8],
    ) -> Result<Self> {
        if tokenizer.is_empty() {
            return Err(invalid("composition tokenizer identity required"));
        }
        let before = snapshot(p)?;
        let saved = CompositionWeights::load(p.composition_source)?;
        let parameters = identities(w.parameters())?;
        if w.config() != saved.config() || parameters != identities(saved.parameters())? {
            return Err(invalid("live composition differs from saved source"));
        }
        let no_read = CompiledNoRead::load(p.no_read_native, p.no_read, tokenizer)?;
        if no_read.config().heads != 2 {
            return Err(invalid("composition requires two actual NoRead heads"));
        }
        let (left, right) = saved.selected_roots()?;
        let gains = saved.packed_gains()?;
        let h4 = fs::read(p.no_read.value.context_source.join("h4-tables.bin"))?;
        if sha256_bytes(&h4) != saved.config().geometry_sha256 {
            return Err(invalid(
                "composition geometry differs from actual producer context",
            ));
        }
        let native = NativeGeometricComposition::new(&left, &right, &gains, &h4)
            .map_err(|e| invalid(e.to_string()))?;
        if snapshot(p)? != before {
            return Err(invalid("composition source changed during compilation"));
        }
        let metadata = CompositionMetadata {
            schema: SCHEMA.into(),
            config: saved.config().clone(),
            policy: POLICY.into(),
            source_files: before,
            parameters,
            no_read_metadata: bound(&serde_json::to_vec(no_read.metadata())?),
            reducer_metadata: no_read
                .metadata()
                .dependency_metadata
                .get("reducer")
                .ok_or_else(|| invalid("composition NoRead reducer identity absent"))?
                .clone(),
            left: bound(&left),
            right: bound(&right),
            gains: bound(&gains),
            tokenizer: bound(tokenizer),
            payload_fractional_bits: 16,
            payload_storage_bits: 64,
            output_width: OUTPUT_WIDTH,
            actual_bank_bytes: left.len() + right.len() + gains.len(),
            numerical_composition_schema: uor_r4_integer::geometric_composition::SCHEMA.into(),
            numerical_reducer_schema: uor_r4_integer::geometric_composed_read::SCHEMA.into(),
        };
        Ok(Self {
            metadata,
            native,
            left,
            right,
            gains,
            tokenizer: tokenizer.to_vec(),
        })
    }
    pub fn metadata(&self) -> &CompositionMetadata {
        &self.metadata
    }
    pub fn native_kernel(&self) -> &NativeGeometricComposition {
        &self.native
    }
    pub fn validate_for(&self, w: &CompositionWeights) -> Result<()> {
        if w.config() != &self.metadata.config
            || identities(w.parameters())? != self.metadata.parameters
            || w.selected_roots()? != (self.left.clone(), self.right.clone())
            || w.packed_gains()? != self.gains
        {
            return Err(invalid("composition stale for current source parameters"));
        }
        Ok(())
    }
    #[allow(clippy::too_many_arguments)]
    pub fn validate_dependencies(
        &self,
        context: &CompiledContext,
        potential: &CompiledGeometricPotentials,
        reducer: &CompiledGeometricRead,
        producer: &CompiledValueProducer,
        events: &CompiledEvents,
        span: &CompiledSpanActions,
        no_read: &CompiledNoRead,
    ) -> Result<()> {
        no_read.validate_dependencies(context, potential, reducer, producer, events, span)?;
        if bound(&serde_json::to_vec(no_read.metadata())?) != self.metadata.no_read_metadata
            || reducer.metadata().heads != 2
            || reducer.metadata().value_width != 16
            || reducer.metadata().context > 128
        {
            return Err(invalid(
                "composition actual producer/NoRead/reducer binding differs",
            ));
        }
        Ok(())
    }
    pub fn compose_trace(&self, trace: &ValueProducerTrace) -> Result<Vec<i64>> {
        compose_trace(&self.native, trace)
    }
    pub fn save(&self, dir: &Path) -> Result<()> {
        fs::create_dir(dir)?;
        for (name, bytes) in [
            (FILES[0], serde_json::to_vec_pretty(&self.metadata)?),
            (FILES[1], self.left.clone()),
            (FILES[2], self.right.clone()),
            (FILES[3], self.gains.clone()),
            (FILES[4], self.tokenizer.clone()),
        ] {
            fs::File::create_new(dir.join(name))?.write_all(&bytes)?;
        }
        Ok(())
    }
    pub fn load(dir: &Path, p: CompositionSourcePaths<'_>, tokenizer: &[u8]) -> Result<Self> {
        let names = fs::read_dir(dir)?
            .map(|e| {
                let e = e?;
                if !e.file_type()?.is_file() {
                    return Err(invalid("composition artifact nonregular entry"));
                }
                e.file_name()
                    .into_string()
                    .map_err(|_| invalid("composition artifact nonUTF8 entry"))
            })
            .collect::<Result<BTreeSet<_>>>()?;
        if names != FILES.iter().map(|s| s.to_string()).collect() {
            return Err(invalid("composition artifact file set differs"));
        }
        let expected = Self::compile(
            &CompositionWeights::load(p.composition_source)?,
            p,
            tokenizer,
        )?;
        let saved: CompositionMetadata = serde_json::from_slice(&fs::read(dir.join(FILES[0]))?)?;
        if saved != expected.metadata {
            return Err(invalid(
                "composition artifact metadata differs from independent source compile",
            ));
        }
        for (name, bytes) in [
            (FILES[1], &expected.left),
            (FILES[2], &expected.right),
            (FILES[3], &expected.gains),
            (FILES[4], &expected.tokenizer),
        ] {
            if fs::metadata(dir.join(name))?.len() != bytes.len() as u64
                || fs::read(dir.join(name))? != *bytes
            {
                return Err(invalid("composition artifact payload differs from source"));
            }
        }
        Ok(expected)
    }
}
pub fn compose_trace(
    bank: &NativeGeometricComposition,
    t: &ValueProducerTrace,
) -> Result<Vec<i64>> {
    let positions = t
        .batch
        .checked_mul(t.heads)
        .and_then(|x| x.checked_mul(t.time))
        .ok_or_else(|| invalid("composition trace size overflow"))?;
    if t.batch == 0
        || t.heads != 2
        || t.time == 0
        || t.time > 128
        || t.packets.len() != positions * 4
        || t.occurrence_valid.len() != t.batch * t.time
        || t.values_q16.len() != positions * 16
    {
        return Err(invalid("composition K2 trace layout differs"));
    }
    let codec = uor_r4_integer::geometric_value::NativeGeometricValues::canonical()
        .map_err(|e| invalid(e.to_string()))?;
    let mut result = Vec::with_capacity(positions * 32);
    for pos in 0..positions {
        let mut packets = [[ValuePacket::absent(); 2]; 4];
        for (lane, pair) in packets.iter_mut().enumerate() {
            for (atom, p) in pair.iter_mut().enumerate() {
                let r = &t.packets[pos * 4 + lane][atom];
                let state = match r.status {
                    ValuePacketStatus::Absent => ValueState::Absent,
                    ValuePacketStatus::PresentZero => ValueState::PresentZero,
                    ValuePacketStatus::PresentNonzero => ValueState::PresentNonzero,
                };
                *p = ValuePacket::new(state, r.root, r.radius_bin)
                    .map_err(|e| invalid(e.to_string()))?;
            }
        }
        let b = pos / (2 * t.time);
        let token = pos % t.time;
        if !t.occurrence_valid[b * t.time + token]
            && packets
                .iter()
                .flatten()
                .any(|p| p.state() != ValueState::Absent)
        {
            return Err(invalid(
                "invalid occurrence carries present composition packet",
            ));
        }
        for (lane, pair) in packets.iter().enumerate() {
            let decoded = codec
                .decode_pair(pair[0], pair[1])
                .map_err(|e| invalid(e.to_string()))?;
            if decoded.as_slice() != &t.values_q16[pos * 16 + lane * 4..pos * 16 + lane * 4 + 4] {
                return Err(invalid("composition trace packet/value identity differs"));
            }
        }
        let h = (pos / t.time) % 2;
        result.extend_from_slice(
            &bank
                .compose(h, &packets)
                .map_err(|e| invalid(e.to_string()))?,
        );
    }
    Ok(result)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComposedReadRow {
    pub output_q16: Vec<i64>,
    pub occurrence_weights_q31: Vec<u64>,
    pub no_read_weight_q31: u64,
    pub total_weight_q31: u64,
    pub max_score_q24: i64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComposedReadTrace {
    pub batch: usize,
    pub heads: usize,
    pub time: usize,
    pub value_width: usize,
    pub no_read_q24: Vec<i64>,
    pub values_q16: Vec<i64>,
    pub rows: Vec<ComposedReadRow>,
}
pub struct ComposedReadOutput {
    pub read: Tensor,
    pub trace: ComposedReadTrace,
}
#[allow(clippy::too_many_arguments)]
pub fn reduce_composed_raw_q24(
    scores: &[i64],
    null: &[i64],
    values: &[i64],
    batch: usize,
    heads: usize,
    time: usize,
    potential: &CompiledGeometricPotentials,
    reducer: &CompiledGeometricRead,
    composition: &CompiledComposition,
) -> Result<ComposedReadOutput> {
    let positions = batch
        .checked_mul(heads)
        .and_then(|x| x.checked_mul(time))
        .ok_or_else(|| invalid("composed read size overflow"))?;
    if batch == 0
        || heads != 2
        || time == 0
        || time > reducer.metadata().context
        || time > 128
        || null.len() != positions
        || scores.len() != positions * time
        || values.len() != positions * 32
        || bound(&serde_json::to_vec(reducer.metadata())?) != composition.metadata.reducer_metadata
        || composition.metadata.output_width != 32
        || composition.metadata.payload_storage_bits != 64
        || bound(&serde_json::to_vec_pretty(potential.metadata())?).sha256
            != reducer.metadata().potential_metadata.sha256
    {
        return Err(invalid("composed raw read shape/identity differs"));
    }
    let mut kernel =
        NativeGeometricComposedRead::new(reducer.metadata().context, reducer.exp_q31())
            .map_err(|e| invalid(e.to_string()))?;
    let mut outputs = Vec::with_capacity(positions * 32);
    let mut rows = Vec::with_capacity(positions);
    let mut age = Vec::with_capacity(time);
    for b in 0..batch {
        for h in 0..heads {
            let first = (b * heads + h) * time;
            for q in 0..time {
                age.clear();
                age.extend(
                    (0..=q).map(|j| reducer.age_q24()[h * reducer.metadata().context + q - j]),
                );
                let at = (first + q) * time;
                let row = kernel
                    .reduce(
                        &scores[at..at + q + 1],
                        &age,
                        null[first + q],
                        &values[first * 32..(first + q + 1) * 32],
                    )
                    .map_err(|e| invalid(e.to_string()))?;
                outputs.extend(row.output_q16.iter().map(|&x| (x as f64 / 65536.) as f32));
                rows.push(ComposedReadRow {
                    output_q16: row.output_q16.to_vec(),
                    occurrence_weights_q31: row.occurrence_weights_q31.to_vec(),
                    no_read_weight_q31: row.no_read_weight_q31,
                    total_weight_q31: row.total_weight_q31,
                    max_score_q24: row.max_score_q24,
                });
            }
        }
    }
    Ok(ComposedReadOutput {
        read: Tensor::from_vec(outputs, (batch, heads, time, 32), &Device::Cpu)?,
        trace: ComposedReadTrace {
            batch,
            heads,
            time,
            value_width: 32,
            no_read_q24: null.to_vec(),
            values_q16: values.to_vec(),
            rows,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_value_native::ValuePacketRecord;
    #[test]
    fn native_composition_trace_admission_matches_source_and_preserves_zero() -> Result<()> {
        let weights = CompositionWeights::new()?;
        let bank = weights.hard_bank()?;
        let empty = ValuePacketRecord {
            status: ValuePacketStatus::Absent,
            root: 1,
            radius_bin: 0,
        };
        let mut trace = ValueProducerTrace {
            batch: 1,
            heads: 2,
            time: 1,
            occurrence_valid: vec![false],
            packets: vec![[empty.clone(), empty]; 8],
            values_q16: vec![0; 32],
        };
        assert_eq!(compose_trace(&bank, &trace)?, vec![0; 64]);
        assert_eq!(weights.compose_trace(&trace)?, vec![0; 64]);
        trace.packets[0][0].status = ValuePacketStatus::PresentZero;
        assert!(compose_trace(&bank, &trace).is_err());
        assert!(weights.compose_trace(&trace).is_err());
        trace.occurrence_valid[0] = true;
        assert_eq!(compose_trace(&bank, &trace)?, vec![0; 64]);
        trace.values_q16[0] = 1;
        assert!(compose_trace(&bank, &trace).is_err());
        assert!(weights.compose_trace(&trace).is_err());
        Ok(())
    }
}
