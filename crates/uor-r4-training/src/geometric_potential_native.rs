//! Source-bound Q24 finite geometric potentials and an offline replay boundary.
//!
//! The compiler evaluates the two coordinate-unary and one bilinear potential
//! on all signed canonical roots. It preserves both radial tables and both
//! presence tables, with the original conditional masks and directed relatives.
//! Each family is rounded once to signed Q24, nearest/ties-away, with overflow
//! rejected. These are i32 table entries, NOT a qualified four-bit weight codec.
//! Integer lookup/addition is isolated; input classification still calls the
//! existing F32-to-F64 training encoder, and final F32 score reconstruction is a
//! labelled replay boundary. Controller, age, NoRead, softmax and value/output
//! computation are outside this component. No backward graph is provided.
//!
//! Error contract, per head with L lanes: roots are the same F64 constants in
//! compiler and reference, each coordinate bounded by one. S bounds the sum of
//! absolute underlying monomials: both unary L1 norms, bilinear coefficient L1
//! norm, and one maximum absolute entry from each radial/presence family, summed
//! over lanes. It does NOT use cancellation-prone compiled family sums.
//! Let u64=2^-53, gamma(n)=n*u64/(1-n*u64), gc=gamma(64), and
//! gr=gamma(128*L+128). Sixty-four operations dominate each compiled angular
//! family's evaluation; 128*L+128 dominates the reference's monomial products,
//! lane assembly and lane accumulation. Q=7*L/(2*2^24) bounds table quantization.
//! The integer sum is below 2^39, hence conversion to F64 and dyadic rescaling
//! are exact. Before F32 casts, error is at most Q+(gc+gr)*S. Add BOTH casts:
//! u32*((2+gc+gr)*S+Q)+2^-149, where u32=2^-24 and the last term covers the two
//! half-subnormal absolute rounding allowances. Positive bound arithmetic is
//! rounded upward, including S. Gamma's denominator is exact for these small
//! integer operation counts; its quotient is rounded upward. Nonfinite/unsafe
//! bounds and any table overflow fail admission. This bounds scores, not margins
//! or downstream answer changes. Masks can select fewer than seven terms but
//! the bound always retains all seven. Higher-precision reinterpretations of
//! irrational roots are not the reference addressed by this contract.

use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::Path;

use candle_core::{DType, Device, Tensor};
use safetensors::{Dtype as SafeDtype, SafeTensors};
use serde::{Deserialize, Serialize};
use uor_r4_core::native_geometric::learner::{
    embedding::canonical_h4_roots,
    group_table::{group_table, GROUP_ORDER, ROW_STRIDE},
    prefix_artifact::historical_roots,
};
use uor_r4_integer::geometric_potential::{
    AddressLane, NativePotentialTables, CONTENT_PRESENCE_OFFSET, CONTENT_RADIUS_OFFSET,
    CONTENT_UNARY_OFFSET, CONTEXT_PRESENCE_OFFSET, CONTEXT_RADIUS_OFFSET, CONTEXT_UNARY_OFFSET,
    ENTRIES_PER_LANE, FRACTIONAL_BITS, PAIR_OFFSET,
};
use uor_r4_integer::h4_classifier::H4_ROOT_COEFFICIENTS;
use uor_r4_integer::h4_tables::{
    coefficients_sha256, mathematical_sha256, H4Code, HistoricalH4Tables, PAYLOAD_BYTES,
    TRUSTED_MATHEMATICAL_SHA256,
};

use crate::geometric_address::{
    self, encode_lane, geometry_digest, AddressWeights, GeometricAddressConfig,
};
use crate::geometric_stack::{StackModel, GEOMETRIC_ADDRESS_RECORD};
use crate::{invalid, sha256_bytes, Result};

pub const SCHEMA: &str = "uor-r4.native-geometric-potentials/1";
const POLICY:&str="signed-i32-Q24;nearest-ties-away;one-round-per-family;no-centering;overflow-reject;head-lane-seven-families/1";
const BOUND_RULE: &str =
    "S-raw-monomial-absolute;gamma64+gamma128L128;seven-Q24;both-F32-casts;positive-upward/1";
const FILES: [&str; 4] = [
    "metadata.json",
    "potential-i32le.bin",
    "h4-tables.bin",
    "tokenizer-identity.bin",
];
const PAYLOAD: &[u8] = include_bytes!("../../uor-r4-integer/fixtures/historical-h4-tables-v1.bin");
const SUFFIXES: [&str; 7] = [
    "content_unary",
    "context_unary",
    "pair",
    "content_radius",
    "context_radius",
    "content_presence",
    "context_presence",
];
const COUNTS: [usize; 7] = [4, 4, 16, 1024, 1024, 4, 4];
type Values = [Vec<f32>; 7];

fn shapes(config: &GeometricAddressConfig) -> [Vec<usize>; 7] {
    let (h, l) = (config.heads, config.lanes_per_head);
    [
        vec![h, l, 4],
        vec![h, l, 4],
        vec![h, l, 4, 4],
        vec![h, l, 32, 32],
        vec![h, l, 32, 32],
        vec![h, l, 4],
        vec![h, l, 4],
    ]
}
fn width(config: &GeometricAddressConfig) -> Result<usize> {
    let width = config
        .heads
        .checked_mul(config.lanes_per_head)
        .and_then(|v| v.checked_mul(4))
        .ok_or_else(|| invalid("potential configuration width overflow"))?;
    config.validate(width, config.heads)?;
    Ok(width)
}
fn names() -> [String; 7] {
    std::array::from_fn(|i| format!("layers.02.read.address.{}", SUFFIXES[i]))
}
fn bytes(values: &[f32]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|v| v.to_bits().to_le_bytes())
        .collect()
}
fn table_bytes(values: &[i32]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_le_bytes()).collect()
}
fn tensors(weights: &AddressWeights) -> [&Tensor; 7] {
    [
        &weights.content_unary,
        &weights.context_unary,
        &weights.pair,
        &weights.content_radius,
        &weights.context_radius,
        &weights.content_presence,
        &weights.context_presence,
    ]
}
fn values(weights: &AddressWeights, config: &GeometricAddressConfig) -> Result<Values> {
    width(config)?;
    let expected = shapes(config);
    let tensors = tensors(weights);
    let mut out: Values = std::array::from_fn(|_| Vec::new());
    for i in 0..7 {
        if tensors[i].dims() != expected[i]
            || tensors[i].dtype() != DType::F32
            || !tensors[i].device().is_cpu()
        {
            return Err(invalid(
                "native potential requires seven correctly shaped CPU F32 coefficient tensors",
            ));
        }
        out[i] = tensors[i].flatten_all()?.to_vec1::<f32>()?;
        if out[i].iter().any(|v| !v.is_finite()) {
            return Err(invalid("nonfinite native potential coefficient"));
        }
    }
    Ok(out)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoefficientIdentity {
    pub name: String,
    pub shape: Vec<usize>,
    pub bytes: usize,
    pub sha256: String,
}
fn identities(values: &Values, config: &GeometricAddressConfig) -> Vec<CoefficientIdentity> {
    let shape = shapes(config);
    let names = names();
    (0..7)
        .map(|i| {
            let bytes = bytes(&values[i]);
            CoefficientIdentity {
                name: names[i].clone(),
                shape: shape[i].clone(),
                bytes: bytes.len(),
                sha256: sha256_bytes(&bytes),
            }
        })
        .collect()
}

/// Actual-byte source provenance, not self-reported hashes from a candidate.
pub struct PotentialSourceBinding {
    model_sha256: String,
    config_sha256: String,
    address_sidecar_sha256: String,
    address: GeometricAddressConfig,
    values: Values,
    identities: Vec<CoefficientIdentity>,
    tokenizer: Vec<u8>,
}
impl PotentialSourceBinding {
    pub fn from_directory(directory: &Path, tokenizer_identity: &[u8]) -> Result<Self> {
        if tokenizer_identity.is_empty() {
            return Err(invalid(
                "native potential requires explicit tokenizer identity bytes",
            ));
        }
        let model = fs::read(directory.join("model.safetensors"))?;
        let config = fs::read(directory.join("config.json"))?;
        let sidecar = fs::read(directory.join(GEOMETRIC_ADDRESS_RECORD))?;
        let address = StackModel::saved_geometric_address(directory)?
            .ok_or_else(|| invalid("saved model has no verified geometric address mode"))?;
        width(&address)?;
        if model != fs::read(directory.join("model.safetensors"))?
            || config != fs::read(directory.join("config.json"))?
            || sidecar != fs::read(directory.join(GEOMETRIC_ADDRESS_RECORD))?
        {
            return Err(invalid("saved potential source changed during admission"));
        }
        let safetensors = SafeTensors::deserialize(&model)?;
        let expected = shapes(&address);
        let names = names();
        let mut values: Values = std::array::from_fn(|_| Vec::new());
        for i in 0..7 {
            let view = safetensors.tensor(&names[i])?;
            if view.dtype() != SafeDtype::F32 || view.shape() != expected[i] {
                return Err(invalid("saved potential tensor shape/type differs"));
            }
            values[i] = view
                .data()
                .chunks_exact(4)
                .map(|v| f32::from_le_bytes([v[0], v[1], v[2], v[3]]))
                .collect();
            if values[i].len() != expected[i].iter().product::<usize>()
                || values[i].iter().any(|v| !v.is_finite())
            {
                return Err(invalid("saved potential coefficient bytes are invalid"));
            }
        }
        Ok(Self {
            model_sha256: sha256_bytes(&model),
            config_sha256: sha256_bytes(&config),
            address_sidecar_sha256: sha256_bytes(&sidecar),
            identities: identities(&values, &address),
            address,
            values,
            tokenizer: tokenizer_identity.to_vec(),
        })
    }
    pub fn config(&self) -> &GeometricAddressConfig {
        &self.address
    }
}

fn up(value: f64) -> Result<f64> {
    if !value.is_finite() || value < 0. {
        return Err(invalid("unsafe nonfinite/negative potential error bound"));
    }
    let next = f64::from_bits(value.to_bits() + 1);
    if !next.is_finite() {
        return Err(invalid("potential error bound overflow"));
    }
    Ok(next)
}
fn add_up(a: f64, b: f64) -> Result<f64> {
    up(a + b)
}
fn mul_up(a: f64, b: f64) -> Result<f64> {
    up(a * b)
}
fn gamma(n: usize) -> Result<f64> {
    // This range makes n*u and 1-n*u exact binary64 numbers, and covers every
    // admitted operation count. Never round the denominator upward.
    if n == 0 || n > 1_000_000 {
        return Err(invalid("unsupported potential gamma operation count"));
    }
    let nu = n as f64 * (1.0 / 9_007_199_254_740_992.0);
    up(nu / (1.0 - nu))
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeadScoreErrorBound {
    pub head: usize,
    pub lanes: usize,
    pub absolute_term_sum: f64,
    pub compiled_gamma: f64,
    pub reference_gamma: f64,
    pub quantization: f64,
    pub f64_evaluation: f64,
    pub reference_f32_rounding: f64,
    pub native_f32_rounding: f64,
    pub total: f64,
}
fn bounds(values: &Values, config: &GeometricAddressConfig) -> Result<Vec<HeadScoreErrorBound>> {
    let l = config.lanes_per_head;
    let gc = gamma(64)?;
    let gr = gamma(128 * l + 128)?;
    let q = (7 * l) as f64 / (2.0 * (1u64 << FRACTIONAL_BITS) as f64);
    let u32 = 1.0 / 16_777_216.0;
    let half_subnormal = f64::from(f32::from_bits(1)) / 2.0;
    let mut result = Vec::with_capacity(config.heads);
    for head in 0..config.heads {
        let mut s = 0.;
        for lane in 0..l {
            let index = head * l + lane;
            for family in 0..7 {
                let chunk = &values[family][index * COUNTS[family]..(index + 1) * COUNTS[family]];
                if family < 3 {
                    for &coefficient in chunk {
                        s = add_up(s, f64::from(coefficient).abs())?;
                    }
                } else {
                    let maximum = chunk.iter().map(|v| f64::from(*v).abs()).fold(0., f64::max);
                    s = add_up(s, maximum)?;
                }
            }
        }
        let reference_magnitude = mul_up(add_up(1., gr)?, s)?;
        let native_magnitude = add_up(mul_up(add_up(1., gc)?, s)?, q)?;
        if reference_magnitude > f64::from(f32::MAX) || native_magnitude > f64::from(f32::MAX) {
            return Err(invalid(
                "potential bound cannot guarantee finite F32 replay scores",
            ));
        }
        let f64_evaluation = mul_up(add_up(gc, gr)?, s)?;
        let reference_f32_rounding = add_up(mul_up(u32, reference_magnitude)?, half_subnormal)?;
        let native_f32_rounding = add_up(mul_up(u32, native_magnitude)?, half_subnormal)?;
        let total = add_up(
            add_up(q, f64_evaluation)?,
            add_up(reference_f32_rounding, native_f32_rounding)?,
        )?;
        result.push(HeadScoreErrorBound {
            head,
            lanes: l,
            absolute_term_sum: s,
            compiled_gamma: gc,
            reference_gamma: gr,
            quantization: q,
            f64_evaluation,
            reference_f32_rounding,
            native_f32_rounding,
            total,
        });
    }
    Ok(result)
}

fn quantize(value: f64) -> Result<i32> {
    let rounded = (value * (1u64 << FRACTIONAL_BITS) as f64).round();
    if !rounded.is_finite() || rounded < f64::from(i32::MIN) || rounded > f64::from(i32::MAX) {
        return Err(invalid(
            "finite geometric potential overflows fixed signed Q24",
        ));
    }
    Ok(rounded as i32)
}
fn expand(values: &Values, config: &GeometricAddressConfig) -> Result<Vec<i32>> {
    width(config)?;
    let roots = canonical_h4_roots()
        .iter()
        .map(|q| q.to_array())
        .collect::<Vec<_>>();
    if roots.len() != 120
        || roots
            .iter()
            .flatten()
            .any(|v| !v.is_finite() || v.abs() > 1.)
    {
        return Err(invalid(
            "potential bound requires canonical coordinates bounded by one",
        ));
    }
    let count = config.heads * config.lanes_per_head;
    let mut output = vec![0; count * ENTRIES_PER_LANE];
    for lane in 0..count {
        let row = &mut output[lane * ENTRIES_PER_LANE..(lane + 1) * ENTRIES_PER_LANE];
        for q in 0..120 {
            row[CONTENT_UNARY_OFFSET + q] = quantize(
                (0..4)
                    .map(|i| f64::from(values[0][lane * 4 + i]) * roots[q][i])
                    .sum(),
            )?;
            row[CONTEXT_UNARY_OFFSET + q] = quantize(
                (0..4)
                    .map(|i| f64::from(values[1][lane * 4 + i]) * roots[q][i])
                    .sum(),
            )?;
            for r in 0..120 {
                let mut pair = 0.;
                for i in 0..4 {
                    for j in 0..4 {
                        pair +=
                            f64::from(values[2][lane * 16 + i * 4 + j]) * roots[q][i] * roots[r][j];
                    }
                }
                row[PAIR_OFFSET + q * 128 + r] = quantize(pair)?;
            }
        }
        for (family, offset) in [
            (3, CONTENT_RADIUS_OFFSET),
            (4, CONTEXT_RADIUS_OFFSET),
            (5, CONTENT_PRESENCE_OFFSET),
            (6, CONTEXT_PRESENCE_OFFSET),
        ] {
            for i in 0..COUNTS[family] {
                row[offset + i] = quantize(f64::from(values[family][lane * COUNTS[family] + i]))?;
            }
        }
    }
    Ok(output)
}

fn admit_tables(payload: &[u8]) -> Result<HistoricalH4Tables> {
    let table = HistoricalH4Tables::from_bytes(payload).map_err(|e| invalid(e.to_string()))?;
    let exact = historical_roots();
    let roots = canonical_h4_roots();
    if exact.len() != 120 || roots.len() != 120 {
        return Err(invalid("potential historical root count differs"));
    }
    let phi = (1. + 5f64.sqrt()) / 2.;
    for i in 0..120 {
        let root = roots[i].to_array();
        for j in 0..4 {
            for k in 0..2 {
                if exact[i][j][k] != i64::from(H4_ROOT_COEFFICIENTS[i][j][k]) {
                    return Err(invalid("potential signed coefficient order differs"));
                }
            }
            let [a, b] = H4_ROOT_COEFFICIENTS[i][j];
            if (root[j] - (f64::from(a) + f64::from(b) * phi) / 2.).abs() > 1e-14 {
                return Err(invalid("potential canonical coordinate order differs"));
            }
        }
        if encode_lane(root.map(|v| v as f32))?.root != i as u8 {
            return Err(invalid("potential signed classification order differs"));
        }
    }
    let training = group_table();
    if table.identity().index() != training.identity {
        return Err(invalid("potential exact/training identity differs"));
    }
    for i in 0..GROUP_ORDER {
        let a = H4Code::try_from(i as u8).map_err(|e| invalid(e.to_string()))?;
        if table.inverse(a).index() != training.inverse[i] {
            return Err(invalid("potential exact/training inverse differs"));
        }
        for j in 0..GROUP_ORDER {
            let b = H4Code::try_from(j as u8).map_err(|e| invalid(e.to_string()))?;
            if table.compose(a, b).index() != training.product[i * ROW_STRIDE + j] {
                return Err(invalid("potential exact/training product differs"));
            }
        }
    }
    Ok(table)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PotentialMetadata {
    pub schema: String,
    pub model_sha256: String,
    pub config_sha256: String,
    pub address_sidecar_sha256: String,
    pub address: GeometricAddressConfig,
    pub coefficients: Vec<CoefficientIdentity>,
    pub tokenizer_sha256: String,
    pub tokenizer_bytes: usize,
    pub table_entries: usize,
    pub table_bytes: usize,
    pub table_sha256: String,
    pub algebra_bytes: usize,
    pub algebra_payload_sha256: String,
    pub algebra_mathematical_sha256: String,
    pub coefficient_geometry_sha256: String,
    pub training_geometry_sha256: String,
    pub fractional_bits: u32,
    pub policy: String,
    pub bound_rule: String,
    pub error_bounds: Vec<HeadScoreErrorBound>,
}
fn metadata(
    source: &PotentialSourceBinding,
    flat: &[i32],
    payload: &[u8],
    error_bounds: Vec<HeadScoreErrorBound>,
) -> Result<PotentialMetadata> {
    let math = mathematical_sha256(payload).map_err(|e| invalid(e.to_string()))?;
    if Some(math.as_str()) != TRUSTED_MATHEMATICAL_SHA256 {
        return Err(invalid("potential mathematical trust anchor differs"));
    }
    let binary = table_bytes(flat);
    Ok(PotentialMetadata {
        schema: SCHEMA.into(),
        model_sha256: source.model_sha256.clone(),
        config_sha256: source.config_sha256.clone(),
        address_sidecar_sha256: source.address_sidecar_sha256.clone(),
        address: source.address.clone(),
        coefficients: source.identities.clone(),
        tokenizer_sha256: sha256_bytes(&source.tokenizer),
        tokenizer_bytes: source.tokenizer.len(),
        table_entries: flat.len(),
        table_bytes: binary.len(),
        table_sha256: sha256_bytes(&binary),
        algebra_bytes: payload.len(),
        algebra_payload_sha256: sha256_bytes(payload),
        algebra_mathematical_sha256: math,
        coefficient_geometry_sha256: coefficients_sha256(),
        training_geometry_sha256: geometry_digest().into(),
        fractional_bits: FRACTIONAL_BITS as u32,
        policy: POLICY.into(),
        bound_rule: BOUND_RULE.into(),
        error_bounds,
    })
}

pub struct CompiledGeometricPotentials {
    metadata: PotentialMetadata,
    native: NativePotentialTables,
    algebra: HistoricalH4Tables,
    flat: Vec<i32>,
    algebra_bytes: Vec<u8>,
    tokenizer: Vec<u8>,
}
impl CompiledGeometricPotentials {
    pub fn compile(
        weights: &AddressWeights,
        config: &GeometricAddressConfig,
        source: &PotentialSourceBinding,
    ) -> Result<Self> {
        let live = values(weights, config)?;
        if config != &source.address || identities(&live, config) != source.identities {
            return Err(invalid(
                "live potentials differ from actual saved source coefficients",
            ));
        }
        let flat = expand(&live, config)?;
        let error_bounds = bounds(&live, config)?;
        let algebra = admit_tables(PAYLOAD)?;
        let native = NativePotentialTables::new(config.heads, config.lanes_per_head, &flat)
            .map_err(|e| invalid(e.to_string()))?;
        Ok(Self {
            metadata: metadata(source, &flat, PAYLOAD, error_bounds)?,
            native,
            algebra,
            flat,
            algebra_bytes: PAYLOAD.to_vec(),
            tokenizer: source.tokenizer.clone(),
        })
    }
    pub fn metadata(&self) -> &PotentialMetadata {
        &self.metadata
    }
    pub fn error_bounds(&self) -> &[HeadScoreErrorBound] {
        &self.metadata.error_bounds
    }
    pub fn validate_for(
        &self,
        weights: &AddressWeights,
        config: &GeometricAddressConfig,
    ) -> Result<()> {
        let live = values(weights, config)?;
        if config != &self.metadata.address
            || identities(&live, config) != self.metadata.coefficients
        {
            return Err(invalid(
                "compiled potential tables are stale for live coefficients/configuration",
            ));
        }
        Ok(())
    }
    pub fn save(&self, directory: &Path) -> Result<()> {
        fs::create_dir(directory)?;
        let write = |name: &str, data: &[u8]| -> Result<()> {
            fs::File::create_new(directory.join(name))?.write_all(data)?;
            Ok(())
        };
        write(FILES[1], &table_bytes(&self.flat))?;
        write(FILES[2], &self.algebra_bytes)?;
        write(FILES[3], &self.tokenizer)?;
        write(FILES[0], &serde_json::to_vec_pretty(&self.metadata)?)?;
        Ok(())
    }
    pub fn load(directory: &Path, source: &PotentialSourceBinding) -> Result<Self> {
        let mut names = BTreeSet::new();
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                return Err(invalid("potential artifact contains non-regular file"));
            }
            names.insert(
                entry
                    .file_name()
                    .into_string()
                    .map_err(|_| invalid("potential filename is not UTF-8"))?,
            );
        }
        if names != FILES.iter().map(|s| s.to_string()).collect() {
            return Err(invalid("potential artifact file set differs"));
        }
        let flat = expand(&source.values, &source.address)?;
        let expected_bytes = table_bytes(&flat);
        if fs::metadata(directory.join(FILES[1]))?.len() != expected_bytes.len() as u64
            || fs::metadata(directory.join(FILES[2]))?.len() != PAYLOAD_BYTES as u64
            || fs::metadata(directory.join(FILES[3]))?.len() != source.tokenizer.len() as u64
        {
            return Err(invalid("potential payload length differs"));
        }
        if fs::read(directory.join(FILES[1]))? != expected_bytes
            || fs::read(directory.join(FILES[3]))? != source.tokenizer
        {
            return Err(invalid(
                "potential payload/tokenizer differs from actual source recompilation",
            ));
        }
        let algebra_bytes = fs::read(directory.join(FILES[2]))?;
        let algebra = admit_tables(&algebra_bytes)?;
        let saved: PotentialMetadata =
            serde_json::from_slice(&fs::read(directory.join(FILES[0]))?)?;
        let expected = metadata(
            source,
            &flat,
            &algebra_bytes,
            bounds(&source.values, &source.address)?,
        )?;
        if saved != expected {
            return Err(invalid(
                "potential metadata/source/bound/geometry binding differs",
            ));
        }
        let native =
            NativePotentialTables::new(source.address.heads, source.address.lanes_per_head, &flat)
                .map_err(|e| invalid(e.to_string()))?;
        Ok(Self {
            metadata: saved,
            native,
            algebra,
            flat,
            algebra_bytes,
            tokenizer: source.tokenizer.clone(),
        })
    }
}

/// Current classifier codes are observational output, never source labels.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PotentialCodeTrace {
    pub batch: usize,
    pub time: usize,
    pub heads: usize,
    pub lanes: usize,
    pub current: Vec<geometric_address::AddressCode>,
    pub prior: Vec<geometric_address::AddressCode>,
}
pub fn classify_inputs(
    current: &Tensor,
    prior: &Tensor,
    config: &GeometricAddressConfig,
) -> Result<PotentialCodeTrace> {
    let (batch, time, w) = current.dims3()?;
    if batch == 0
        || time == 0
        || w != width(config)?
        || prior.shape() != current.shape()
        || current.dtype() != DType::F32
        || prior.dtype() != DType::F32
        || !current.device().is_cpu()
        || !prior.device().is_cpu()
    {
        return Err(invalid(
            "potential replay needs matching nonempty CPU F32 [B,T,W] inputs",
        ));
    }
    let classify = |tensor: &Tensor| -> Result<Vec<geometric_address::AddressCode>> {
        tensor
            .flatten_all()?
            .to_vec1::<f32>()?
            .chunks_exact(4)
            .map(|x| encode_lane([x[0], x[1], x[2], x[3]]))
            .collect()
    };
    Ok(PotentialCodeTrace {
        batch,
        time,
        heads: config.heads,
        lanes: config.lanes_per_head,
        current: classify(current)?,
        prior: classify(prior)?,
    })
}
fn native_codes(codes: &[geometric_address::AddressCode]) -> Result<Vec<AddressLane>> {
    codes
        .iter()
        .map(|c| {
            AddressLane::new(c.root, c.radius_bin, c.present).map_err(|e| invalid(e.to_string()))
        })
        .collect()
}

/// Full geometric score only, before age/causal mask/NoRead. Every pair is
/// computed; the caller retains the existing single masking/normalization path.
pub fn score_native(
    current: &Tensor,
    prior: &Tensor,
    config: &GeometricAddressConfig,
    weights: &AddressWeights,
    compiled: &CompiledGeometricPotentials,
) -> Result<Tensor> {
    compiled.validate_for(weights, config)?;
    let trace = classify_inputs(current, prior, config)?;
    let (r, c) = (native_codes(&trace.current)?, native_codes(&trace.prior)?);
    score_native_codes(&r, &c, trace.batch, trace.time, config, weights, compiled)
}

/// Offline tensor bridge around direct typed integer scoring. Neither content
/// nor context addresses are reconstructed or classified from floating point.
/// The returned F32 scores feed the still-floating surrounding reader; this is
/// not a complete native serving entry point.
pub fn score_native_codes(
    current: &[AddressLane],
    prior: &[AddressLane],
    batch: usize,
    time: usize,
    config: &GeometricAddressConfig,
    weights: &AddressWeights,
    compiled: &CompiledGeometricPotentials,
) -> Result<Tensor> {
    compiled.validate_for(weights, config)?;
    let (b, t, h, l) = (batch, time, config.heads, config.lanes_per_head);
    let count = b
        .checked_mul(t)
        .and_then(|n| n.checked_mul(h))
        .and_then(|n| n.checked_mul(l))
        .ok_or_else(|| invalid("typed geometric input size overflow"))?;
    if b == 0 || t == 0 || current.len() != count || prior.len() != count {
        return Err(invalid(
            "typed geometric inputs differ from nonempty [B,T,H,L]",
        ));
    }
    let (r, c) = (current, prior);
    let size = b
        .checked_mul(h)
        .and_then(|n| n.checked_mul(t))
        .and_then(|n| n.checked_mul(t))
        .ok_or_else(|| invalid("potential score output size overflow"))?;
    let mut output = vec![0f32; size];
    for batch in 0..b {
        for head in 0..h {
            for query in 0..t {
                for key in 0..t {
                    let q = ((batch * t + query) * h + head) * l;
                    let k = ((batch * t + key) * h + head) * l;
                    let fixed = compiled
                        .native
                        .score(
                            head,
                            &c[q..q + l],
                            &c[k..k + l],
                            &r[q..q + l],
                            &r[k..k + l],
                            &compiled.algebra,
                        )
                        .map_err(|e| invalid(e.to_string()))?;
                    output[((batch * h + head) * t + query) * t + key] =
                        (fixed as f64 / (1u64 << FRACTIONAL_BITS) as f64) as f32;
                }
            }
        }
    }
    Ok(Tensor::from_vec(output, (b, h, t, t), &Device::Cpu)?)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeadScoreComparison {
    pub head: usize,
    pub pairs: usize,
    pub max_absolute_error: f64,
    pub bound: f64,
    pub bound_violations: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScoreComparison {
    pub batch: usize,
    pub time: usize,
    pub heads: Vec<HeadScoreComparison>,
}
pub fn compare_reference(
    current: &Tensor,
    prior: &Tensor,
    config: &GeometricAddressConfig,
    weights: &AddressWeights,
    compiled: &CompiledGeometricPotentials,
) -> Result<ScoreComparison> {
    let native = score_native(current, prior, config, weights, compiled)?
        .flatten_all()?
        .to_vec1::<f32>()?;
    let reference = geometric_address::score(current, prior, config, weights)?
        .flatten_all()?
        .to_vec1::<f32>()?;
    let (batch, time, _) = current.dims3()?;
    let mut heads = compiled
        .error_bounds()
        .iter()
        .map(|b| HeadScoreComparison {
            head: b.head,
            pairs: 0,
            max_absolute_error: 0.,
            bound: b.total,
            bound_violations: 0,
        })
        .collect::<Vec<_>>();
    for b in 0..batch {
        for h in 0..config.heads {
            for q in 0..time {
                for k in 0..time {
                    let at = ((b * config.heads + h) * time + q) * time + k;
                    if !reference[at].is_finite() || !native[at].is_finite() {
                        return Err(invalid("nonfinite potential comparison score"));
                    }
                    let error = (f64::from(native[at]) - f64::from(reference[at])).abs();
                    heads[h].pairs += 1;
                    heads[h].max_absolute_error = heads[h].max_absolute_error.max(error);
                    heads[h].bound_violations += usize::from(error > heads[h].bound);
                }
            }
        }
    }
    Ok(ScoreComparison { batch, time, heads })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_stack::{ReadIdentityLatch, ReadScore, StackArch, StackConfig};
    use candle_core::Device;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn weights(model: &StackModel) -> AddressWeights {
        let names = names();
        let get = |i: usize| model.variables()[&names[i]].as_tensor().clone();
        AddressWeights {
            content_unary: get(0),
            context_unary: get(1),
            pair: get(2),
            content_radius: get(3),
            context_radius: get(4),
            content_presence: get(5),
            context_presence: get(6),
        }
    }

    fn fixture() -> Result<(PathBuf, StackModel, PotentialSourceBinding)> {
        static SERIAL: AtomicU64 = AtomicU64::new(0);
        let clock = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| invalid(e.to_string()))?
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "uor-native-potential-{}-{clock}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory)?;
        let config = StackConfig {
            arch: StackArch::Geometric,
            vocab_size: 8,
            width: 8,
            heads: 1,
            mlp_hidden: 16,
            context: 16,
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
        model.set_geometric_address(GeometricAddressConfig::new(8, 1)?)?;
        for (family, name) in names().iter().enumerate() {
            let variable = &model.variables()[name];
            let values = (0..variable.elem_count())
                .map(|i| (((i * 13 + family * 7) % 31) as f32 - 15.) * 0.013 + 0.0013)
                .collect::<Vec<_>>();
            variable.set(&Tensor::from_vec(values, variable.shape(), &Device::Cpu)?)?;
        }
        model.save(&directory)?;
        let source = PotentialSourceBinding::from_directory(
            &directory,
            b"native-potential fixture token registry 0..7",
        )?;
        Ok((directory, model, source))
    }

    #[test]
    fn native_potential_source_reload_resealed_tamper_and_stale_weights() -> Result<()> {
        let (directory, model, source) = fixture()?;
        let config = source.config();
        let w = weights(&model);
        let compiled = CompiledGeometricPotentials::compile(&w, config, &source)?;
        let artifact = directory.join("compiled");
        compiled.save(&artifact)?;
        assert!(compiled.save(&artifact).is_err());
        let loaded = CompiledGeometricPotentials::load(&artifact, &source)?;
        assert_eq!(loaded.metadata(), compiled.metadata());
        let metadata_bytes = fs::read(artifact.join(FILES[0]))?;
        let original = fs::read(artifact.join(FILES[1]))?;
        let mut changed = original.clone();
        changed[0] ^= 1;
        fs::write(artifact.join(FILES[1]), &changed)?;
        let mut lied = compiled.metadata.clone();
        lied.table_sha256 = sha256_bytes(&changed);
        fs::write(artifact.join(FILES[0]), serde_json::to_vec(&lied)?)?;
        assert!(CompiledGeometricPotentials::load(&artifact, &source).is_err());
        fs::write(artifact.join(FILES[1]), original)?;
        fs::write(artifact.join(FILES[0]), &metadata_bytes)?;
        let other = PotentialSourceBinding::from_directory(
            &directory,
            b"same numerical IDs but a different token registry",
        )?;
        assert!(CompiledGeometricPotentials::load(&artifact, &other).is_err());
        let mut malformed: serde_json::Value = serde_json::from_slice(&metadata_bytes)?;
        malformed["ignored"] = true.into();
        fs::write(artifact.join(FILES[0]), serde_json::to_vec(&malformed)?)?;
        assert!(CompiledGeometricPotentials::load(&artifact, &source).is_err());
        fs::write(artifact.join(FILES[0]), metadata_bytes)?;
        fs::write(artifact.join("extra"), b"unbound")?;
        assert!(CompiledGeometricPotentials::load(&artifact, &source).is_err());
        fs::remove_file(artifact.join("extra"))?;
        // One bit of live coefficient mutation invalidates the artifact even
        // if the fixed Q24 table would happen to be unchanged.
        let name = names()[0].clone();
        let variable = &model.variables()[&name];
        let mut v = variable.flatten_all()?.to_vec1::<f32>()?;
        v[0] = f32::from_bits(v[0].to_bits() ^ 1);
        variable.set(&Tensor::from_vec(v, variable.shape(), &Device::Cpu)?)?;
        assert!(compiled.validate_for(&weights(&model), config).is_err());
        assert!(CompiledGeometricPotentials::compile(&weights(&model), config, &source).is_err());
        // The saved geometric-address verifier is the source-admission boundary.
        fs::remove_file(directory.join(GEOMETRIC_ADDRESS_RECORD))?;
        assert!(PotentialSourceBinding::from_directory(&directory, b"registry").is_err());
        fs::remove_dir_all(directory)?;
        Ok(())
    }

    #[test]
    fn native_potential_all_signed_finite_angular_entries_match_compiler_bound() -> Result<()> {
        let (directory, model, source) = fixture()?;
        let config = source.config();
        let w = weights(&model);
        let compiled = CompiledGeometricPotentials::compile(&w, config, &source)?;
        let values = values(&w, config)?;
        let roots = canonical_h4_roots();
        let scale = (1u64 << FRACTIONAL_BITS) as f64;
        for lane in 0..config.lanes_per_head {
            let row = &compiled.flat[lane * ENTRIES_PER_LANE..(lane + 1) * ENTRIES_PER_LANE];
            for q in 0..120 {
                let qc = roots[q].to_array();
                for (family, offset) in [(0, CONTENT_UNARY_OFFSET), (1, CONTEXT_UNARY_OFFSET)] {
                    let expected = (0..4)
                        .map(|i| f64::from(values[family][lane * 4 + i]) * qc[i])
                        .sum::<f64>();
                    assert!((f64::from(row[offset + q]) / scale - expected).abs() <= 0.5 / scale);
                }
                for r in 0..120 {
                    let rc = roots[r].to_array();
                    let mut expected = 0.;
                    for i in 0..4 {
                        for j in 0..4 {
                            expected += f64::from(values[2][lane * 16 + i * 4 + j]) * qc[i] * rc[j];
                        }
                    }
                    assert!(
                        (f64::from(row[PAIR_OFFSET + q * 128 + r]) / scale - expected).abs()
                            <= 0.5 / scale
                    );
                }
            }
            for q in 0..128 {
                for r in 0..128 {
                    if q >= 120 || r >= 120 {
                        assert_eq!(row[PAIR_OFFSET + q * 128 + r], 0);
                    }
                }
            }
            for (family, offset) in [
                (3, CONTENT_RADIUS_OFFSET),
                (4, CONTEXT_RADIUS_OFFSET),
                (5, CONTENT_PRESENCE_OFFSET),
                (6, CONTEXT_PRESENCE_OFFSET),
            ] {
                for i in 0..COUNTS[family] {
                    assert!(
                        (f64::from(row[offset + i]) / scale
                            - f64::from(values[family][lane * COUNTS[family] + i]))
                        .abs()
                            <= 0.5 / scale
                    );
                }
            }
        }
        assert_ne!(
            compiled.metadata.training_geometry_sha256,
            compiled.metadata.algebra_mathematical_sha256
        );
        fs::remove_dir_all(directory)?;
        Ok(())
    }

    #[test]
    fn native_potential_actual_scores_cover_seven_families_zero_radius_and_presence() -> Result<()>
    {
        let (directory, model, source) = fixture()?;
        let config = source.config();
        let w = weights(&model);
        let compiled = CompiledGeometricPotentials::compile(&w, config, &source)?;
        let (batch, time) = (2, 48);
        let roots = canonical_h4_roots();
        let mut current = Vec::new();
        let mut prior = Vec::new();
        for row in 0..batch * time {
            for lane in 0..config.lanes_per_head {
                let radius = 2f32.powi(((row + lane * 7) % 32) as i32 - 16);
                let r = roots[(row * 7 + lane * 13) % 120].to_array();
                let c = roots[(row * 11 + lane * 17 + 31) % 120].to_array();
                current.extend(r.map(|v| if row % 5 == 0 { 0. } else { v as f32 * radius }));
                prior.extend(c.map(|v| if row % 7 == 0 { 0. } else { v as f32 / radius }));
            }
        }
        let current = Tensor::from_vec(current, (batch, time, 8), &Device::Cpu)?;
        let prior = Tensor::from_vec(prior, (batch, time, 8), &Device::Cpu)?;
        let result = compare_reference(&current, &prior, config, &w, &compiled)?;
        assert_eq!(result.heads[0].pairs, batch * time * time);
        assert_eq!(result.heads[0].bound_violations, 0);
        let trace = classify_inputs(&current, &prior, config)?;
        assert!(trace.current.iter().any(|c| !c.present));
        assert!(trace.current.iter().any(|c| c.present && c.radius_bin == 0));
        assert!(trace
            .current
            .iter()
            .any(|c| c.present && c.radius_bin == 31));
        let r = native_codes(&trace.current)?;
        let c = native_codes(&trace.prior)?;
        let l = config.lanes_per_head;
        let mut seen = [false; 4];
        for q in 0..time {
            for k in 0..time {
                seen[2 * usize::from(trace.prior[q * l].present)
                    + usize::from(trace.prior[k * l].present)] = true;
                let fixed = compiled
                    .native
                    .score(
                        0,
                        &c[q * l..q * l + l],
                        &c[k * l..k * l + l],
                        &r[q * l..q * l + l],
                        &r[k * l..k * l + l],
                        &compiled.algebra,
                    )
                    .map_err(|e| invalid(e.to_string()))?;
                assert!(fixed.unsigned_abs() < (1u64 << 39));
            }
        }
        assert_eq!(seen, [true; 4]);
        // Signed unary imaginary coordinates distinguish inverse relatives.
        let a = encode_lane([0., 1., 0., 0.])?.root;
        let b = group_table().inverse[usize::from(a)];
        assert_ne!(
            compiled.flat[CONTENT_UNARY_OFFSET + usize::from(a)],
            compiled.flat[CONTENT_UNARY_OFFSET + usize::from(b)]
        );
        fs::remove_dir_all(directory)?;
        Ok(())
    }

    #[test]
    fn native_potential_quantization_gamma_and_input_refusals_are_fixed() -> Result<()> {
        let scale = (1u64 << FRACTIONAL_BITS) as f64;
        assert_eq!(quantize(0.5 / scale)?, 1);
        assert_eq!(quantize(-0.5 / scale)?, -1);
        assert_eq!(quantize(f64::from(i32::MIN) / scale)?, i32::MIN);
        assert_eq!(quantize(f64::from(i32::MAX) / scale)?, i32::MAX);
        assert!(quantize(128.).is_err());
        assert!(quantize(-128. - 1. / scale).is_err());
        assert!(quantize(f64::NAN).is_err());
        assert!(gamma(0).is_err());
        assert!(gamma(1_000_001).is_err());
        assert!(up(f64::MAX).is_err());
        assert!(up(-1.).is_err());
        for lanes in [1, 2, 32] {
            let n = 128 * lanes + 128;
            let u = 1. / 9_007_199_254_740_992.;
            assert!(gamma(n)? >= (n as f64 * u) / (1. - n as f64 * u));
        }
        let (directory, model, source) = fixture()?;
        let config = source.config();
        let w = weights(&model);
        let compiled = CompiledGeometricPotentials::compile(&w, config, &source)?;
        let mut cancellation = source.values.clone();
        cancellation[0][0] = 1e20;
        cancellation[0][1] = -1e20;
        let large = bounds(&cancellation, config)?;
        assert!(large[0].absolute_term_sum >= 2e20);
        assert!(expand(&cancellation, config).is_err());
        let mut overflow = source.values.clone();
        overflow[0].fill(f32::MAX);
        assert!(bounds(&overflow, config).is_err());
        let zero = Tensor::zeros((1, 1, 8), DType::F32, &Device::Cpu)?;
        let bad = Tensor::from_vec(vec![f32::NAN; 8], (1, 1, 8), &Device::Cpu)?;
        assert!(score_native(&bad, &zero, config, &w, &compiled).is_err());
        assert!(score_native(&zero.to_dtype(DType::F64)?, &zero, config, &w, &compiled).is_err());
        let other = Tensor::zeros((1, 2, 8), DType::F32, &Device::Cpu)?;
        assert!(score_native(&zero, &other, config, &w, &compiled).is_err());
        assert!(PotentialSourceBinding::from_directory(&directory, b"").is_err());
        fs::remove_dir_all(directory)?;
        Ok(())
    }
}
