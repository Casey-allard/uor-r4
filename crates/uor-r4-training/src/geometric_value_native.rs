//! Source-bound K1 geometric value codec and OFFLINE donor projection.
//!
//! This is a representation comparison, not a learned value producer. For
//! each actual donor four-vector, it exhaustively minimizes squared distance
//! to the decoded Q16 alphabet: PRESENT_ZERO first, then historical signed
//! root order, then increasing radius. Exact ties keep the first candidate.
//! No answers, source labels, ranking weights, or NoRead scores enter that
//! choice. NoRead and occurrence identities remain the caller's responsibility.
//!
//! A projected lane always has donor_valid=true. A numerical vector cannot
//! establish semantic ABSENT, and cancellation is not an absence flag.
//! Independent four-coordinate lanes preserve exactly 16 coordinates/head.
//!
//! Canonical table construction and admission reuse the integer codec's exact
//! golden-coefficient rounding, independently regenerated on reload. The raw
//! projected_q16 trace is authoritative. The returned F32 Tensor is a debug
//! reconstruction: large Q16 coordinates need not roundtrip through F32, so it
//! MUST NOT be requantized for an exact native reducer comparison.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::Path;

use candle_core::{DType, Device, Tensor};
use safetensors::{Dtype as SafeDtype, SafeTensors};
use serde::{Deserialize, Serialize};
use uor_r4_integer::geometric_value::{
    NativeGeometricValues, ValuePacket, ValueState, COORDINATES, FRACTIONAL_BITS,
    MAX_RADIUS_EXPONENT, MIN_RADIUS_EXPONENT, RADIUS_COUNT, RUNTIME_COORDINATE_BYTES, TABLE_BYTES,
    TABLE_ENTRIES,
};
use uor_r4_integer::h4_tables::{
    coefficients_sha256, mathematical_sha256, HistoricalH4Tables, ROOT_COUNT,
};

use crate::geometric_potential_native::CompiledGeometricPotentials;
use crate::geometric_read_native::{
    quantize_value_q16, CompiledGeometricRead, ReadBoundFile, ReadSourceBinding,
};
use crate::geometric_stack::StackConfig;
use crate::{invalid, sha256_bytes, Result};

pub const SCHEMA: &str = "uor-r4.geometric-value-native/1";
pub const VALUE_WIDTH: usize = 16;
pub const LANES_PER_HEAD: usize = VALUE_WIDTH / COORDINATES;
pub const CANDIDATES: usize = 1 + ROOT_COUNT * RADIUS_COUNT;
const PROJECTION: &str = "offline-oracle-Q16-input;exhaustive-decoded-alphabet-squared-L2-i128;strict-less;ties-PRESENT_ZERO-first-then-root-ascending-then-radius-ascending;no-answer-source-ranking-labels/1";
const DECODE: &str = "pinned-historical-signed-H4;(a+b*phi)/2;dyadic-radius[-16,14];exact-integer-interval-nearest-ties-away-Q16;integer-module-canonical-table/1";
const STATUS: &str = "ABSENT!=PRESENT_ZERO!=PRESENT_NONZERO;all-projected-donors-valid;coordinates-never-infer-ABSENT;nonzero-packet-not-reclassified-after-rounding;metadata-not-numerical-channels/1";
const PRODUCER: &str = "oracle-donor-projection-not-learned-serving-producer;independent-four-coordinate-fixed-basis-lanes;identity-transport;occurrence-identity-caller-owned;NoRead-and-support-unchanged/1";
const FILES: [&str; 4] = [
    "metadata.json",
    "coordinates-i32le.bin",
    "h4-tables.bin",
    "tokenizer-identity.bin",
];
const ALGEBRA: &[u8] = include_bytes!("../../uor-r4-integer/fixtures/historical-h4-tables-v1.bin");

fn bound(bytes: &[u8]) -> ReadBoundFile {
    ReadBoundFile {
        bytes: bytes.len(),
        sha256: sha256_bytes(bytes),
    }
}
fn f32_bytes(values: &[f32]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|v| v.to_bits().to_le_bytes())
        .collect()
}
fn tensor_values(tensor: &Tensor, shape: &[usize]) -> Result<Vec<f32>> {
    if !tensor.device().is_cpu() || tensor.dtype() != DType::F32 || tensor.dims() != shape {
        return Err(invalid("value codec requires matching CPU F32 tensor"));
    }
    let values = tensor.flatten_all()?.to_vec1::<f32>()?;
    if values.iter().any(|v| !v.is_finite()) {
        return Err(invalid("nonfinite value codec source coefficients"));
    }
    Ok(values)
}

/// Actual saved-source admission. Private fields prevent a caller's asserted
/// hashes from replacing the model, tokenizer, potential or reducer identity.
pub struct ValueSourceBinding {
    files: BTreeMap<String, ReadBoundFile>,
    layer: usize,
    heads: usize,
    context: usize,
    width: usize,
    value_parameter: String,
    value_weight: Vec<f32>,
    tokenizer: Vec<u8>,
    potential_metadata: Vec<u8>,
    read_metadata: Vec<u8>,
}
impl ValueSourceBinding {
    pub fn from_directory(
        directory: &Path,
        tokenizer: &[u8],
        potential: &CompiledGeometricPotentials,
        read: &CompiledGeometricRead,
        layer: usize,
    ) -> Result<Self> {
        let read_source =
            ReadSourceBinding::from_directory(directory, tokenizer, potential, layer)?;
        let mut files = BTreeMap::new();
        for (name, expected) in &read.metadata().source_files {
            let bytes = fs::read(directory.join(name))?;
            if bound(&bytes) != *expected {
                return Err(invalid(
                    "value codec reducer differs from actual saved source",
                ));
            }
            files.insert(name.clone(), bytes);
        }
        let config_bytes = files
            .get("config.json")
            .ok_or_else(|| invalid("value codec config absent"))?;
        let config: StackConfig = serde_json::from_slice(config_bytes)?;
        config.validate()?;
        if config.width / config.heads != VALUE_WIDTH || read.metadata().value_width != VALUE_WIDTH
        {
            return Err(invalid(
                "K1 value comparison preserves exactly 16 coordinates per head",
            ));
        }
        let model = files
            .get("model.safetensors")
            .ok_or_else(|| invalid("value codec model absent"))?;
        let archive = SafeTensors::deserialize(model)?;
        let age_name = format!("layers.{layer:02}.read.age");
        let age = archive.tensor(&age_name)?;
        if age.dtype() != SafeDtype::F32 || age.shape() != [config.heads, config.context] {
            return Err(invalid("value codec saved age shape/type differs"));
        }
        let age_values: Vec<f32> = age
            .data()
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect();
        let age = Tensor::from_vec(age_values, (config.heads, config.context), &Device::Cpu)?;
        let expected_read = CompiledGeometricRead::compile(&age, potential, &read_source)?;
        if expected_read.metadata() != read.metadata() {
            return Err(invalid(
                "value codec reducer differs from source recompilation",
            ));
        }
        let value_parameter = format!("layers.{layer:02}.read.value.weight");
        let value = archive.tensor(&value_parameter)?;
        if value.dtype() != SafeDtype::F32 || value.shape() != [config.width, config.width] {
            return Err(invalid("value codec saved value map shape/type differs"));
        }
        let value_weight: Vec<f32> = value
            .data()
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect();
        if value_weight.len() != config.width * config.width
            || value_weight.iter().any(|v| !v.is_finite())
        {
            return Err(invalid(
                "value codec saved value map is nonfinite or malformed",
            ));
        }
        for (name, bytes) in &files {
            if *bytes != fs::read(directory.join(name))? {
                return Err(invalid("value codec source changed during admission"));
            }
        }
        Ok(Self {
            files: files.iter().map(|(n, v)| (n.clone(), bound(v))).collect(),
            layer,
            heads: config.heads,
            context: config.context,
            width: config.width,
            value_parameter,
            value_weight,
            tokenizer: tokenizer.to_vec(),
            potential_metadata: serde_json::to_vec_pretty(potential.metadata())?,
            read_metadata: serde_json::to_vec_pretty(read.metadata())?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeometricValueMetadata {
    pub schema: String,
    pub codec_schema: String,
    pub source_files: BTreeMap<String, ReadBoundFile>,
    pub layer: usize,
    pub heads: usize,
    pub context: usize,
    pub width: usize,
    pub value_width: usize,
    pub lanes_per_head: usize,
    pub value_parameter: String,
    pub value_weight_f32: ReadBoundFile,
    pub potential_metadata: ReadBoundFile,
    pub read_metadata: ReadBoundFile,
    pub tokenizer: ReadBoundFile,
    pub geometry: ReadBoundFile,
    pub geometry_mathematical_sha256: String,
    pub root_coefficients_sha256: String,
    pub coordinates: ReadBoundFile,
    pub coordinate_entries: usize,
    pub runtime_coordinate_bytes: usize,
    pub root_count: usize,
    pub min_radius_exponent: i8,
    pub max_radius_exponent: i8,
    pub radius_count: usize,
    pub fractional_bits: u32,
    pub projection_candidates: usize,
    pub decode_policy: String,
    pub projection_policy: String,
    pub status_policy: String,
    pub producer_policy: String,
}
fn metadata(source: &ValueSourceBinding, bytes: &[u8]) -> Result<GeometricValueMetadata> {
    HistoricalH4Tables::from_bytes(ALGEBRA).map_err(|e| invalid(e.to_string()))?;
    Ok(GeometricValueMetadata {
        schema: SCHEMA.into(),
        codec_schema: uor_r4_integer::geometric_value::SCHEMA.into(),
        source_files: source.files.clone(),
        layer: source.layer,
        heads: source.heads,
        context: source.context,
        width: source.width,
        value_width: VALUE_WIDTH,
        lanes_per_head: LANES_PER_HEAD,
        value_parameter: source.value_parameter.clone(),
        value_weight_f32: bound(&f32_bytes(&source.value_weight)),
        potential_metadata: bound(&source.potential_metadata),
        read_metadata: bound(&source.read_metadata),
        tokenizer: bound(&source.tokenizer),
        geometry: bound(ALGEBRA),
        geometry_mathematical_sha256: mathematical_sha256(ALGEBRA)
            .map_err(|e| invalid(e.to_string()))?,
        root_coefficients_sha256: coefficients_sha256(),
        coordinates: bound(bytes),
        coordinate_entries: TABLE_ENTRIES,
        runtime_coordinate_bytes: RUNTIME_COORDINATE_BYTES,
        root_count: ROOT_COUNT,
        min_radius_exponent: MIN_RADIUS_EXPONENT,
        max_radius_exponent: MAX_RADIUS_EXPONENT,
        radius_count: RADIUS_COUNT,
        fractional_bits: FRACTIONAL_BITS,
        projection_candidates: CANDIDATES,
        decode_policy: DECODE.into(),
        projection_policy: PROJECTION.into(),
        status_policy: STATUS.into(),
        producer_policy: PRODUCER.into(),
    })
}

struct Candidate {
    packet: ValuePacket,
    coordinates: [i32; 4],
}
pub struct CompiledGeometricValues {
    metadata: GeometricValueMetadata,
    native: NativeGeometricValues,
    bytes: Vec<u8>,
    tokenizer: Vec<u8>,
    candidates: Vec<Candidate>,
}
impl CompiledGeometricValues {
    pub fn compile(source: &ValueSourceBinding) -> Result<Self> {
        let native = NativeGeometricValues::canonical().map_err(|e| invalid(e.to_string()))?;
        let bytes = native.to_bytes();
        if bytes.len() != TABLE_BYTES {
            return Err(invalid("canonical value table length differs"));
        }
        let metadata = metadata(source, &bytes)?;
        let zero = ValuePacket::present_zero();
        let mut candidates = vec![Candidate {
            packet: zero,
            coordinates: native.decode(zero),
        }];
        for root in 0..ROOT_COUNT {
            for radius in 0..RADIUS_COUNT {
                let packet = ValuePacket::present_nonzero(root as u8, radius as u8)
                    .map_err(|e| invalid(e.to_string()))?;
                candidates.push(Candidate {
                    packet,
                    coordinates: native.decode(packet),
                });
            }
        }
        Ok(Self {
            metadata,
            native,
            bytes,
            tokenizer: source.tokenizer.clone(),
            candidates,
        })
    }
    pub fn metadata(&self) -> &GeometricValueMetadata {
        &self.metadata
    }
    pub fn decode(&self, packet: ValuePacket) -> [i32; 4] {
        self.native.decode(packet)
    }
    pub fn validate_for(
        &self,
        value_weight: &Tensor,
        potential: &CompiledGeometricPotentials,
        read: &CompiledGeometricRead,
    ) -> Result<()> {
        let shape = [self.metadata.width, self.metadata.width];
        if bound(&f32_bytes(&tensor_values(value_weight, &shape)?))
            != self.metadata.value_weight_f32
            || bound(&serde_json::to_vec_pretty(potential.metadata())?)
                != self.metadata.potential_metadata
            || bound(&serde_json::to_vec_pretty(read.metadata())?) != self.metadata.read_metadata
        {
            return Err(invalid(
                "value codec is stale for live value map/potential/reducer",
            ));
        }
        Ok(())
    }
    pub fn save(&self, directory: &Path) -> Result<()> {
        fs::create_dir(directory)?;
        for (name, bytes) in [
            (FILES[0], serde_json::to_vec_pretty(&self.metadata)?),
            (FILES[1], self.bytes.clone()),
            (FILES[2], ALGEBRA.to_vec()),
            (FILES[3], self.tokenizer.clone()),
        ] {
            fs::File::create_new(directory.join(name))?.write_all(&bytes)?;
        }
        Ok(())
    }
    pub fn load(directory: &Path, source: &ValueSourceBinding) -> Result<Self> {
        let mut names = BTreeSet::new();
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                return Err(invalid("value codec artifact has nonregular file"));
            }
            names.insert(
                entry
                    .file_name()
                    .into_string()
                    .map_err(|_| invalid("value codec non-UTF8 filename"))?,
            );
        }
        if names != FILES.iter().map(|s| s.to_string()).collect() {
            return Err(invalid("value codec artifact file set differs"));
        }
        let expected = Self::compile(source)?;
        for (name, length) in [
            (FILES[1], TABLE_BYTES),
            (FILES[2], ALGEBRA.len()),
            (FILES[3], source.tokenizer.len()),
        ] {
            if fs::metadata(directory.join(name))?.len() != length as u64 {
                return Err(invalid("value codec artifact payload length differs"));
            }
        }
        let saved: GeometricValueMetadata =
            serde_json::from_slice(&fs::read(directory.join(FILES[0]))?)?;
        let bytes = fs::read(directory.join(FILES[1]))?;
        NativeGeometricValues::from_bytes(&bytes).map_err(|e| invalid(e.to_string()))?;
        if saved != expected.metadata
            || bytes != expected.bytes
            || fs::read(directory.join(FILES[2]))? != ALGEBRA
            || fs::read(directory.join(FILES[3]))? != expected.tokenizer
        {
            return Err(invalid(
                "value codec differs from independently regenerated source binding",
            ));
        }
        Ok(expected)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ValuePacketStatus {
    Absent,
    PresentZero,
    PresentNonzero,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValuePacketRecord {
    pub status: ValuePacketStatus,
    pub root: u8,
    pub radius_bin: u8,
}
impl ValuePacketRecord {
    fn from_packet(packet: ValuePacket) -> Self {
        let status = match packet.state() {
            ValueState::Absent => ValuePacketStatus::Absent,
            ValueState::PresentZero => ValuePacketStatus::PresentZero,
            ValueState::PresentNonzero => ValuePacketStatus::PresentNonzero,
        };
        Self {
            status,
            root: packet.root().index(),
            radius_bin: packet.radius_bin(),
        }
    }
    pub fn packet(&self) -> Result<ValuePacket> {
        let state = match self.status {
            ValuePacketStatus::Absent => ValueState::Absent,
            ValuePacketStatus::PresentZero => ValueState::PresentZero,
            ValuePacketStatus::PresentNonzero => ValueState::PresentNonzero,
        };
        ValuePacket::new(state, self.root, self.radius_bin).map_err(|e| invalid(e.to_string()))
    }
}

// Squared Q16 errors can exceed u64. Serialize decimal strings rather than
// allowing serde_json::Value or a JSON consumer to round exact u128 integers.
mod decimal_u128 {
    use serde::{Deserialize, Deserializer, Serializer};
    pub fn serialize<S: Serializer>(value: &u128, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&value.to_string())
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u128, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValueProjectionLane {
    pub donor_valid: bool,
    pub packet: ValuePacketRecord,
    pub error_q16: [i64; 4],
    #[serde(with = "decimal_u128")]
    pub squared_error_q32: u128,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValueProjectionTrace {
    pub batch: usize,
    pub heads: usize,
    pub time: usize,
    pub value_width: usize,
    /// [B,H,T,16] numerical values, before and after oracle projection.
    pub donor_q16: Vec<i32>,
    pub projected_q16: Vec<i32>,
    /// [B,H,T,4] lanes, with no semantic source/answer labels.
    pub lanes: Vec<ValueProjectionLane>,
    pub unique_vectors: usize,
    pub candidate_distances: usize,
    pub max_abs_error_q16: u64,
    #[serde(with = "decimal_u128")]
    pub sum_squared_error_q32: u128,
}
pub struct ValueProjectionOutput {
    /// F32 debug reconstruction only; feed trace.projected_q16 to reduction.
    pub projected: Tensor,
    pub trace: ValueProjectionTrace,
}

fn squared_error(donor: [i32; 4], candidate: [i32; 4]) -> u128 {
    donor
        .into_iter()
        .zip(candidate)
        .map(|(a, b)| {
            let d = i128::from(a) - i128::from(b);
            (d * d) as u128
        })
        .sum()
}
fn project_lane(
    donor: [i32; 4],
    compiled: &CompiledGeometricValues,
) -> (ValueProjectionLane, [i32; 4]) {
    // Both integer inputs are i32, so each squared difference is <2^64 and
    // four coordinates sum to <2^66. No saturating arithmetic or float metric.
    let mut best = &compiled.candidates[0];
    let mut error = squared_error(donor, best.coordinates);
    for candidate in compiled.candidates.iter().skip(1) {
        let proposed = squared_error(donor, candidate.coordinates);
        if proposed < error {
            best = candidate;
            error = proposed;
        }
    }
    (
        ValueProjectionLane {
            donor_valid: true,
            packet: ValuePacketRecord::from_packet(best.packet),
            error_q16: std::array::from_fn(|i| {
                i64::from(best.coordinates[i]) - i64::from(donor[i])
            }),
            squared_error_q32: error,
        },
        best.coordinates,
    )
}

/// Project actual Q16 donor values. Caching uses the exact four i32 values;
/// it changes neither candidates nor ties and introduces no extra quantizer.
pub fn project_q16(
    values: &[i32],
    batch: usize,
    heads: usize,
    time: usize,
    compiled: &CompiledGeometricValues,
) -> Result<ValueProjectionOutput> {
    let count = batch
        .checked_mul(heads)
        .and_then(|n| n.checked_mul(time))
        .and_then(|n| n.checked_mul(VALUE_WIDTH));
    if batch == 0
        || time == 0
        || time > compiled.metadata.context
        || heads != compiled.metadata.heads
        || count != Some(values.len())
    {
        return Err(invalid(
            "value projection requires [B,H,T,16] within bound source context/head layout",
        ));
    }
    let mut cache = BTreeMap::new();
    let mut projected_q16 = Vec::with_capacity(values.len());
    let mut lanes = Vec::with_capacity(values.len() / COORDINATES);
    let mut total = 0u128;
    let mut maximum = 0u64;
    for input in values.chunks_exact(COORDINATES) {
        let donor = [input[0], input[1], input[2], input[3]];
        let (record, coords) = cache
            .entry(donor)
            .or_insert_with(|| project_lane(donor, compiled));
        total = total
            .checked_add(record.squared_error_q32)
            .ok_or_else(|| invalid("value projection total squared error overflows u128"))?;
        maximum = maximum.max(
            record
                .error_q16
                .iter()
                .map(|e| e.unsigned_abs())
                .max()
                .unwrap_or(0),
        );
        lanes.push(record.clone());
        projected_q16.extend_from_slice(coords);
    }
    let candidate_distances = cache
        .len()
        .checked_mul(CANDIDATES)
        .ok_or_else(|| invalid("value projection candidate counter overflow"))?;
    let reconstructed: Vec<f32> = projected_q16
        .iter()
        .map(|&x| (f64::from(x) / 65_536.) as f32)
        .collect();
    Ok(ValueProjectionOutput {
        projected: Tensor::from_vec(
            reconstructed,
            (batch, heads, time, VALUE_WIDTH),
            &Device::Cpu,
        )?,
        trace: ValueProjectionTrace {
            batch,
            heads,
            time,
            value_width: VALUE_WIDTH,
            donor_q16: values.to_vec(),
            projected_q16,
            lanes,
            unique_vectors: cache.len(),
            candidate_distances,
            max_abs_error_q16: maximum,
            sum_squared_error_q32: total,
        },
    })
}
/// Explicit offline F32 donor boundary. Quantization is the same nearest /
/// ties-away Q16 rule as the retained reducer, before exact packet projection.
pub fn project_native(
    values: &Tensor,
    compiled: &CompiledGeometricValues,
) -> Result<ValueProjectionOutput> {
    let (batch, heads, time, width) = values.dims4()?;
    if width != VALUE_WIDTH || values.dtype() != DType::F32 || !values.device().is_cpu() {
        return Err(invalid("value projection needs CPU F32 donor[B,H,T,16]"));
    }
    let q16 = values
        .flatten_all()?
        .to_vec1::<f32>()?
        .into_iter()
        .map(quantize_value_q16)
        .collect::<Result<Vec<_>>>()?;
    project_q16(&q16, batch, heads, time, compiled)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_address::GeometricAddressConfig;
    use crate::geometric_potential_native::PotentialSourceBinding;
    use crate::geometric_span::GeometricSpanConfig;
    use crate::geometric_stack::{ReadIdentityLatch, ReadScore, StackArch, StackModel};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    const REGISTRY: &[u8] = b"K1 codec fixture token registry 0..3/v1";
    const HEADS: usize = 2;
    const CONTEXT: usize = 8;
    struct Fixture {
        directory: PathBuf,
        model: StackModel,
        potential: CompiledGeometricPotentials,
        read: CompiledGeometricRead,
        source: ValueSourceBinding,
    }
    fn fixture() -> Result<Fixture> {
        static SERIAL: AtomicU64 = AtomicU64::new(0);
        let clock = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| invalid(e.to_string()))?
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "uor-native-value-{}-{clock}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory)?;
        let base = directory.join("base");
        let config = StackConfig {
            arch: StackArch::Geometric,
            vocab_size: 4,
            width: HEADS * VALUE_WIDTH,
            heads: HEADS,
            mlp_hidden: 32,
            context: CONTEXT,
            pattern: "rra".into(),
            read: ReadScore::Lorentz,
            rotation: true,
            seed: 19,
            memory: None,
            select: None,
            pointer: None,
        };
        let mut model = StackModel::new(config, &Device::Cpu)?;
        model.set_read_identity_latch(ReadIdentityLatch::Held)?;
        let address = GeometricAddressConfig::new(HEADS * VALUE_WIDTH, HEADS)?;
        model.set_geometric_address(address.clone())?;
        model.set_geometric_span(GeometricSpanConfig::new(HEADS * VALUE_WIDTH)?)?;
        model.save(&base)?;
        let psource = PotentialSourceBinding::from_directory(&base, REGISTRY)?;
        let potential = CompiledGeometricPotentials::compile(
            &model.geometric_address_potentials()?,
            &address,
            &psource,
        )?;
        let rsource = ReadSourceBinding::from_directory(&base, REGISTRY, &potential, 2)?;
        let read = CompiledGeometricRead::compile(
            model.variables()["layers.02.read.age"].as_tensor(),
            &potential,
            &rsource,
        )?;
        let source = ValueSourceBinding::from_directory(&base, REGISTRY, &potential, &read, 2)?;
        Ok(Fixture {
            directory,
            model,
            potential,
            read,
            source,
        })
    }

    #[test]
    fn native_value_source_reload_resealed_tamper_and_stale_coefficients() -> Result<()> {
        let fixture = fixture()?;
        let compiled = CompiledGeometricValues::compile(&fixture.source)?;
        let artifact = fixture.directory.join("compiled");
        compiled.save(&artifact)?;
        assert!(compiled.save(&artifact).is_err());
        let loaded = CompiledGeometricValues::load(&artifact, &fixture.source)?;
        assert_eq!(compiled.metadata(), loaded.metadata());
        assert_eq!(compiled.bytes, loaded.bytes);
        assert_eq!(loaded.metadata().coordinates.bytes, 59_520);
        assert_eq!(loaded.metadata().runtime_coordinate_bytes, 61_440);
        let zero = vec![0; HEADS * VALUE_WIDTH];
        assert_eq!(
            project_q16(&zero, 1, HEADS, 1, &compiled)?.trace,
            project_q16(&zero, 1, HEADS, 1, &loaded)?.trace
        );
        let base = fixture.directory.join("base");
        assert!(ValueSourceBinding::from_directory(
            &base,
            b"other registry",
            &fixture.potential,
            &fixture.read,
            2
        )
        .is_err());
        assert!(ValueSourceBinding::from_directory(
            &base,
            REGISTRY,
            &fixture.potential,
            &fixture.read,
            0
        )
        .is_err());
        let original_metadata = fs::read(artifact.join(FILES[0]))?;
        for index in [1, 2] {
            let original = fs::read(artifact.join(FILES[index]))?;
            let mut changed = original.clone();
            changed[4] ^= 1;
            let mut lied = compiled.metadata.clone();
            if index == 1 {
                lied.coordinates = bound(&changed);
            } else {
                lied.geometry = bound(&changed);
            }
            fs::write(artifact.join(FILES[index]), &changed)?;
            fs::write(artifact.join(FILES[0]), serde_json::to_vec(&lied)?)?;
            assert!(CompiledGeometricValues::load(&artifact, &fixture.source).is_err());
            fs::write(artifact.join(FILES[index]), original)?;
            fs::write(artifact.join(FILES[0]), &original_metadata)?;
        }
        fs::write(artifact.join("extra"), b"unbound")?;
        assert!(CompiledGeometricValues::load(&artifact, &fixture.source).is_err());
        fs::remove_file(artifact.join("extra"))?;
        let value = fixture.model.variables()["layers.02.read.value.weight"].as_tensor();
        compiled.validate_for(value, &fixture.potential, &fixture.read)?;
        let mut changed = value.flatten_all()?.to_vec1::<f32>()?;
        changed[0] = f32::from_bits(changed[0].to_bits() ^ 1);
        let changed = Tensor::from_vec(changed, value.shape(), &Device::Cpu)?;
        assert!(compiled
            .validate_for(&changed, &fixture.potential, &fixture.read)
            .is_err());
        assert!(CompiledGeometricValues::load(&artifact, &fixture.source).is_ok());
        fs::remove_dir_all(fixture.directory)?;
        Ok(())
    }

    #[test]
    fn native_value_projection_fixed_basis_ties_capacity_and_status() -> Result<()> {
        let fixture = fixture()?;
        let compiled = CompiledGeometricValues::compile(&fixture.source)?;
        let (zero, coordinates) = project_lane([0; 4], &compiled);
        assert!(zero.donor_valid);
        assert_eq!(zero.packet.status, ValuePacketStatus::PresentZero);
        assert_eq!(coordinates, [0; 4]);
        // A dyadic midpoint is a real K1 representation loss, not Q16 error.
        // Ties choose the smaller radius after the same earliest root.
        let (midpoint, coordinates) = project_lane([98_304, 0, 0, 0], &compiled);
        assert_eq!(midpoint.packet.root, 1);
        assert_eq!(midpoint.packet.radius_bin, 16);
        assert_eq!(coordinates, [65_536, 0, 0, 0]);
        assert_eq!(midpoint.error_q16, [-32_768, 0, 0, 0]);
        assert_eq!(midpoint.squared_error_q32, 1u128 << 30);
        let (negative, coordinates) = project_lane([-98_304, 0, 0, 0], &compiled);
        assert_eq!(negative.packet.root, 0);
        assert_eq!(negative.packet.radius_bin, 16);
        assert_eq!(coordinates, [-65_536, 0, 0, 0]);
        assert_eq!(negative.squared_error_q32, midpoint.squared_error_q32);
        // The metadata states remain distinct even when coordinates agree.
        let absent = ValuePacketRecord::from_packet(ValuePacket::absent());
        assert_ne!(absent, zero.packet);
        assert_eq!(compiled.decode(absent.packet()?), coordinates.map(|_| 0));
        assert!(ValuePacketRecord {
            status: ValuePacketStatus::Absent,
            root: 0,
            radius_bin: 0
        }
        .packet()
        .is_err());
        let packet = ValuePacket::present_nonzero(1, 0).map_err(|e| invalid(e.to_string()))?;
        assert_eq!(compiled.decode(packet), [1, 0, 0, 0]);
        assert_eq!(
            ValuePacketRecord::from_packet(packet).status,
            ValuePacketStatus::PresentNonzero
        );
        fs::remove_dir_all(fixture.directory)?;
        Ok(())
    }

    #[test]
    fn native_value_projection_raw_q16_and_large_error_serialization() -> Result<()> {
        let fixture = fixture()?;
        let compiled = CompiledGeometricValues::compile(&fixture.source)?;
        // These admitted high-radius golden coordinates are not all exactly
        // representable as F32. Projection must preserve their raw integers.
        let golden = compiled
            .decode(ValuePacket::present_nonzero(24, 30).map_err(|e| invalid(e.to_string()))?);
        let values = golden.repeat(HEADS * LANES_PER_HEAD);
        let projected = project_q16(&values, 1, HEADS, 1, &compiled)?;
        assert_eq!(projected.trace.projected_q16, values);
        assert_eq!(projected.trace.sum_squared_error_q32, 0);
        assert_eq!(projected.trace.unique_vectors, 1);
        assert_eq!(projected.trace.candidate_distances, CANDIDATES);
        let debug = projected.projected.flatten_all()?.to_vec1::<f32>()?;
        let roundtrip = debug
            .into_iter()
            .map(quantize_value_q16)
            .collect::<Result<Vec<_>>>()?;
        assert_ne!(roundtrip, projected.trace.projected_q16);
        let extreme = project_q16(&vec![i32::MIN; HEADS * VALUE_WIDTH], 1, HEADS, 1, &compiled)?;
        assert!(extreme.trace.sum_squared_error_q32 > u128::from(u64::MAX));
        let bytes = serde_json::to_vec(&extreme.trace)?;
        let restored: ValueProjectionTrace = serde_json::from_slice(&bytes)?;
        assert_eq!(restored, extreme.trace);
        let as_value = serde_json::to_value(&extreme.trace)?;
        assert_eq!(
            as_value["sum_squared_error_q32"].as_str(),
            Some(extreme.trace.sum_squared_error_q32.to_string().as_str())
        );
        assert!(extreme
            .trace
            .lanes
            .iter()
            .all(|lane| lane.donor_valid && lane.packet.status != ValuePacketStatus::Absent));
        fs::remove_dir_all(fixture.directory)?;
        Ok(())
    }

    #[test]
    fn native_value_projection_preserves_layout_locality_and_input_domain() -> Result<()> {
        let fixture = fixture()?;
        let compiled = CompiledGeometricValues::compile(&fixture.source)?;
        let (batch, time) = (2, 3);
        let mut values = vec![0i32; batch * HEADS * time * VALUE_WIDTH];
        for position in 0..batch * HEADS * time {
            for lane in 0..LANES_PER_HEAD {
                let root = (position % 8) as u8;
                let bin = (lane + 10) as u8;
                let packet =
                    ValuePacket::present_nonzero(root, bin).map_err(|e| invalid(e.to_string()))?;
                let at = position * VALUE_WIDTH + lane * COORDINATES;
                values[at..at + COORDINATES].copy_from_slice(&compiled.decode(packet));
            }
        }
        let before = project_q16(&values, batch, HEADS, time, &compiled)?;
        assert_eq!(before.projected.dims(), [batch, HEADS, time, VALUE_WIDTH]);
        assert_eq!(before.trace.projected_q16, values);
        assert_eq!(
            before.trace.lanes.len(),
            batch * HEADS * time * LANES_PER_HEAD
        );
        let floating = Tensor::from_vec(
            values
                .iter()
                .map(|&v| v as f32 / 65_536.)
                .collect::<Vec<_>>(),
            (batch, HEADS, time, VALUE_WIDTH),
            &Device::Cpu,
        )?;
        assert_eq!(project_native(&floating, &compiled)?.trace, before.trace);
        for bh in 0..batch * HEADS {
            let at = (bh * time + 2) * VALUE_WIDTH;
            values[at..at + VALUE_WIDTH].fill(98_304);
        }
        let after = project_q16(&values, batch, HEADS, time, &compiled)?;
        for bh in 0..batch * HEADS {
            let at = bh * time * LANES_PER_HEAD;
            assert_eq!(
                before.trace.lanes[at..at + 2 * LANES_PER_HEAD],
                after.trace.lanes[at..at + 2 * LANES_PER_HEAD]
            );
        }
        assert!(project_q16(&values[..values.len() - 1], batch, HEADS, time, &compiled).is_err());
        assert!(project_q16(&values, batch, 1, time, &compiled).is_err());
        assert!(project_q16(&[], 1, HEADS, CONTEXT + 1, &compiled).is_err());
        for bad in [f32::NAN, 32_768.] {
            let bad = Tensor::from_vec(
                vec![bad; HEADS * VALUE_WIDTH],
                (1, HEADS, 1, VALUE_WIDTH),
                &Device::Cpu,
            )?;
            assert!(project_native(&bad, &compiled).is_err());
        }
        fs::remove_dir_all(fixture.directory)?;
        Ok(())
    }
}
