//! Source-bound offline bridge to the native geometric weighted reducer.
//!
//! Geometric potential scores enter as signed Q24 integers, without an F32
//! round trip. Saved learned age coefficients and caller-provided floating
//! NoRead scores are rounded to Q24; floating values are rounded to Q16. All
//! conversions use nearest/ties-away and reject overflow/nonfinite values.
//! The numerical kernel includes NoRead's zero payload, keeps every causal
//! occurrence and normalizes only after accumulating integer numerators.
//!
//! The exp table is compiled here with Rust F64 exp at step 2^-8, rounded to
//! Q31. Reload independently regenerates its bytes and the saved age table.
//! Its identity binds this compiler's arithmetic, not cross-libm bit parity.
//! The returned Tensor reconstructs F32 only AFTER integer reduction. This
//! is not a gradient operation or a full native model: donor NoRead/value
//! production, read.out and the surrounding stack remain outside the kernel.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::Path;

use candle_core::{DType, Device, Tensor};
use safetensors::{Dtype as SafeDtype, SafeTensors};
use serde::{Deserialize, Serialize};
use uor_r4_integer::geometric_age_q4::{
    AgeQ4Config, AgeResidualQ4Config, NativeAgeQ4, NativeAgeResidualQ4,
};
use uor_r4_integer::geometric_no_read::pack_coefficients;
use uor_r4_integer::geometric_read::{
    NativeGeometricRead, EXP_STEP_LOG2, EXP_TABLE_LEN, MAX_CONTEXT, MAX_VALUE_WIDTH, WEIGHT_ONE,
};

use crate::geometric_potential_native::{CompiledGeometricPotentials, PotentialSourceBinding};
use crate::geometric_stack::{
    StackArch, StackConfig, StackModel, GEOMETRIC_ADDRESS_RECORD, GEOMETRIC_SPAN_RECORD,
};
use crate::{invalid, sha256_bytes, Result};

pub const SCHEMA: &str = "uor-r4.geometric-read-reducer/1";
pub const Q4_AGE_SCHEMA: &str = "uor-r4.geometric-read-reducer/2";
pub const Q4_AGE_RESIDUAL_SCHEMA: &str = "uor-r4.geometric-read-reducer/3";
pub const Q4_AGE_FILE: &str = "age-coefficients-q4.bin";
pub const Q4_AGE_RESIDUAL_FILE: &str = "age-residual-coefficients-q4.bin";
pub const Q4_AGE_TRAINING_POLICY: &str = "CPU-F32-nat-shadows;fixed-quarter-grid-nearest-ties-away;reject-outside[-1.75,1.75];live-shadow-minus-detached-shadow-plus-detached-current-grid;identity-STE;no-reducer-backward/1";
pub const Q4_AGE_RESIDUAL_TRAINING_POLICY: &str = "CPU-F32-absolute-nat-shadows;F64-subtract-fixed-initialization-prior;residual-eighth-grid-nearest-ties-away;reject-residual-outside[-0.875,0.875];live-raw-shadow-minus-detached-raw-shadow-plus-detached-current-prior-and-grid;identity-STE;prior-not-learned;no-reducer-backward/1";
const Q4_AGE_QUANTIZATION: &str = "age-packed-signed-q4[-7,7]-quarter-nat-exact-Q24;NoRead-signed-i64-Q24;value-and-output-signed-i32-Q16;nearest-ties-away;nonfinite-overflow-reject/1";
const Q4_AGE_RESIDUAL_QUANTIZATION: &str = "age-fixed-initialization-prior-plus-packed-signed-q4[-7,7]-eighth-nat-residual-exact-Q24;NoRead-signed-i64-Q24;value-and-output-signed-i32-Q16;nearest-ties-away;nonfinite-overflow-reject/1";
const QUANTIZATION: &str = "age-and-NoRead-signed-i64-Q24;value-and-output-signed-i32-Q16;nearest-ties-away;nonfinite-overflow-reject/1";
const EXP_POLICY: &str = "Rust-F64-exp(-index/256)*2^31;nearest-ties-away;8194-u32-entries;step-8;interpolation-a-minus-floor-decrement;bias-less-than-one-Q31-unit;outside-table-zero/1";
const REDUCTION: &str = "full-causal-prefix-current-included;age=query-source;NoRead-zero-payload;max-subtract;Q31-exponentials;i128-weighted-sum;exact-integer-division-nearest-ties-away;no-support-pruning/1";
const PRODUCER: &str = "admitted-native-geometric-potential-Q24;caller-CPU-F32-NoRead-and-values;no-source-label-input;F32-reconstruction-after-native-reduction/1";
const FILES: [&str; 4] = [
    "metadata.json",
    "age-i64le.bin",
    "exp-u32le.bin",
    "tokenizer-identity.bin",
];

/// Inventory dispatch for dependency snapshots. This does not replace `load`,
/// which independently regenerates and admits all numerical payloads.
pub fn native_file_names(directory: &Path) -> Result<Vec<&'static str>> {
    let metadata: GeometricReadMetadata =
        serde_json::from_slice(&fs::read(directory.join(FILES[0]))?)?;
    let mut files = FILES.to_vec();
    if let Some(name) = metadata.age_mode()?.packed_file() {
        files.push(name);
    }
    Ok(files)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadBoundFile {
    pub bytes: usize,
    pub sha256: String,
}
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
fn age_bytes(values: &[i64]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_le_bytes()).collect()
}
fn exp_bytes(values: &[u32]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_le_bytes()).collect()
}

/// Actual saved source bytes, constructed by reading and validating the model.
/// The caller cannot provide replacement self-reported hashes.
pub struct ReadSourceBinding {
    files: BTreeMap<String, ReadBoundFile>,
    layer: usize,
    heads: usize,
    context: usize,
    value_width: usize,
    age_parameter: String,
    age: Vec<f32>,
    tokenizer: Vec<u8>,
    potential_metadata: Vec<u8>,
}
impl ReadSourceBinding {
    /// Bind the explicitly chosen read site. The current saved geometric
    /// address operation is supported only at layer 2 of an `rra` stack.
    pub fn from_directory(
        directory: &Path,
        tokenizer_identity: &[u8],
        potential: &CompiledGeometricPotentials,
        layer: usize,
    ) -> Result<Self> {
        if tokenizer_identity.is_empty() {
            return Err(invalid(
                "native read requires explicit tokenizer identity bytes",
            ));
        }
        let mut source = BTreeMap::new();
        for name in [
            "config.json",
            "model.safetensors",
            GEOMETRIC_ADDRESS_RECORD,
            GEOMETRIC_SPAN_RECORD,
        ] {
            source.insert(name.to_owned(), fs::read(directory.join(name))?);
        }
        let config: StackConfig = serde_json::from_slice(&source["config.json"])?;
        config.validate()?;
        if layer != 2
            || config.arch != StackArch::Geometric
            || config.pattern != "rra"
            || config.context == 0
            || config.context > MAX_CONTEXT
            || config.width / config.heads > MAX_VALUE_WIDTH
        {
            return Err(invalid(
                "native read requires explicit rra layer2, supported context/head width",
            ));
        }
        let address = StackModel::saved_geometric_address(directory)?
            .ok_or_else(|| invalid("native read requires saved geometric addressing"))?;
        StackModel::saved_geometric_span(directory)?
            .ok_or_else(|| invalid("native read requires saved geometric span producer"))?;
        // Either admitted potential schema keeps the immutable donor/base
        // identity separate from its numerical coefficient source. Rebinding
        // changes only the metadata envelope, never age or exponent tables.
        let potential_parent =
            PotentialSourceBinding::from_directory(directory, tokenizer_identity)?;
        potential.validate_base_source(&potential_parent)?;
        let meta = potential.metadata();
        if meta.model_sha256 != sha256_bytes(&source["model.safetensors"])
            || meta.config_sha256 != sha256_bytes(&source["config.json"])
            || meta.address_sidecar_sha256 != sha256_bytes(&source[GEOMETRIC_ADDRESS_RECORD])
            || meta.address != address
            || address.heads != config.heads
            || meta.tokenizer_bytes != tokenizer_identity.len()
            || meta.tokenizer_sha256 != sha256_bytes(tokenizer_identity)
        {
            return Err(invalid(
                "native read potential object differs from actual saved source/tokenizer",
            ));
        }
        let age_parameter = format!("layers.{layer:02}.read.age");
        let archive = SafeTensors::deserialize(&source["model.safetensors"])?;
        let view = archive.tensor(&age_parameter)?;
        if view.dtype() != SafeDtype::F32
            || view.shape() != [config.heads, config.context]
            || view.data().len() != config.heads * config.context * 4
        {
            return Err(invalid("saved native read age shape/type/length differs"));
        }
        let age = view
            .data()
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect::<Vec<_>>();
        for &value in &age {
            quantize_score_q24(value)?;
        }
        for (name, bytes) in &source {
            if *bytes != fs::read(directory.join(name))? {
                return Err(invalid("native read source changed during admission"));
            }
        }
        Ok(Self {
            files: source
                .iter()
                .map(|(name, bytes)| (name.clone(), bound(bytes)))
                .collect(),
            layer,
            heads: config.heads,
            context: config.context,
            value_width: config.width / config.heads,
            age_parameter,
            age,
            tokenizer: tokenizer_identity.to_vec(),
            potential_metadata: serde_json::to_vec_pretty(meta)?,
        })
    }
}

/// This bound uses exact powers of two; F64's rounded representation of
/// i64::MAX is NOT a safe inclusive upper range check.
pub fn quantize_score_q24(value: f32) -> Result<i64> {
    let scaled = (f64::from(value) * 16_777_216.).round();
    if !scaled.is_finite()
        || !(-9_223_372_036_854_775_808.0..9_223_372_036_854_775_808.0).contains(&scaled)
    {
        return Err(invalid(
            "native read Q24 score is nonfinite or overflows i64",
        ));
    }
    Ok(scaled as i64)
}
pub fn quantize_value_q16(value: f32) -> Result<i32> {
    let scaled = (f64::from(value) * 65_536.).round();
    if !scaled.is_finite() || !(-2_147_483_648.0..2_147_483_648.0).contains(&scaled) {
        return Err(invalid(
            "native read Q16 value is nonfinite or overflows i32",
        ));
    }
    Ok(scaled as i32)
}
fn canonical_exp() -> Vec<u32> {
    (0..EXP_TABLE_LEN)
        .map(|i| ((-(i as f64) / 256.).exp() * WEIGHT_ONE as f64).round() as u32)
        .collect()
}
fn tensor_age(age: &Tensor, heads: usize, context: usize) -> Result<Vec<f32>> {
    if age.dtype() != DType::F32 || !age.device().is_cpu() || age.dims() != [heads, context] {
        return Err(invalid("native read needs CPU F32 age [heads,context]"));
    }
    let values = age.flatten_all()?.to_vec1::<f32>()?;
    for &value in &values {
        quantize_score_q24(value)?;
    }
    Ok(values)
}

fn q4_age(config: AgeQ4Config, shadow: &[f32]) -> Result<NativeAgeQ4> {
    let count = config
        .coefficient_count()
        .map_err(|e| invalid(e.to_string()))?;
    if shadow.len() != count {
        return Err(invalid("q4 age shadow coefficient count differs"));
    }
    let coefficients = shadow
        .iter()
        .map(|&value| {
            if !value.is_finite() || !(-1.75..=1.75).contains(&value) {
                return Err(invalid(
                    "q4 age shadow outside fixed [-1.75,1.75] nat range",
                ));
            }
            Ok((f64::from(value) * 4.).round() as i8)
        })
        .collect::<Result<Vec<_>>>()?;
    NativeAgeQ4::new(
        config,
        &pack_coefficients(&coefficients).map_err(|e| invalid(e.to_string()))?,
    )
    .map_err(|e| invalid(e.to_string()))
}

/// Current hard quarter-nat age with an explicitly chosen identity surrogate.
/// This does not attach gradients to the integer reducer. The caller must use
/// this graph in its actual normalized score path. Every call re-quantizes the
/// live shadows; no compiled snapshot is reused as a live hard value.
pub fn q4_age_training_view(age: &Tensor, heads: usize, context: usize) -> Result<Tensor> {
    let codec = q4_age(
        AgeQ4Config { heads, context },
        &tensor_age(age, heads, context)?,
    )?;
    let grid = Tensor::from_vec(
        codec
            .age_q24()
            .iter()
            .map(|&x| (x as f64 / 16_777_216.) as f32)
            .collect::<Vec<_>>(),
        (heads, context),
        age.device(),
    )?;
    Ok((age - &age.detach())?.add(&grid)?)
}

fn q4_age_residual(config: AgeResidualQ4Config, raw: &[f32]) -> Result<NativeAgeResidualQ4> {
    if raw.len()
        != config
            .coefficient_count()
            .map_err(|e| invalid(e.to_string()))?
    {
        return Err(invalid("q4 residual age raw coefficient count differs"));
    }
    let prior = config.prior_q24().map_err(|e| invalid(e.to_string()))?;
    let coefficients = raw
        .iter()
        .zip(prior)
        .map(|(&raw, prior)| {
            // F64 subtraction is part of the source contract: a rounded F32
            // intermediate could change which side of a half tie is selected.
            let residual = f64::from(raw) - prior as f64 / 16_777_216.;
            if !residual.is_finite() || !(-0.875..=0.875).contains(&residual) {
                return Err(invalid(
                    "q4 age residual outside fixed [-0.875,0.875] nat range",
                ));
            }
            Ok((residual * 8.).round() as i8)
        })
        .collect::<Result<Vec<_>>>()?;
    NativeAgeResidualQ4::new(
        config,
        &pack_coefficients(&coefficients).map_err(|e| invalid(e.to_string()))?,
    )
    .map_err(|e| invalid(e.to_string()))
}

/// Absolute live age shadows, with a fixed initialization prior and eighth-nat
/// residual choices. The prior is not learned. Every forward rebuilds current
/// residual choices and attaches a unit surrogate adjoint to the raw shadow;
/// the integer reducer itself remains detached.
pub fn q4_age_residual_training_view(age: &Tensor, heads: usize, context: usize) -> Result<Tensor> {
    let codec = q4_age_residual(
        AgeResidualQ4Config { heads, context },
        &tensor_age(age, heads, context)?,
    )?;
    let hard = Tensor::from_vec(
        codec
            .age_q24()
            .iter()
            .map(|&x| (x as f64 / 16_777_216.) as f32)
            .collect::<Vec<_>>(),
        (heads, context),
        age.device(),
    )?;
    Ok((age - &age.detach())?.add(&hard)?)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgeQ4Metadata {
    pub schema: String,
    pub policy: String,
    pub config: AgeQ4Config,
    pub coefficient_count: usize,
    pub packed_coefficients: ReadBoundFile,
    pub training_policy: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgeResidualQ4Metadata {
    pub schema: String,
    pub policy: String,
    pub config: AgeResidualQ4Config,
    pub prior_identity: String,
    pub prior_phase: String,
    pub prior_head_shifts: Vec<u32>,
    pub prior_q24: ReadBoundFile,
    pub residual_exponent: i32,
    pub coefficient_count: usize,
    pub packed_coefficients: ReadBoundFile,
    pub training_policy: String,
}

#[derive(Clone, Copy)]
enum AgeMode {
    Raw,
    Quarter,
    PriorResidual,
}
impl AgeMode {
    fn packed_file(self) -> Option<&'static str> {
        match self {
            Self::Raw => None,
            Self::Quarter => Some(Q4_AGE_FILE),
            Self::PriorResidual => Some(Q4_AGE_RESIDUAL_FILE),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeometricReadMetadata {
    pub schema: String,
    pub source_files: BTreeMap<String, ReadBoundFile>,
    pub layer: usize,
    pub heads: usize,
    pub context: usize,
    pub value_width: usize,
    pub kernel_max_context: usize,
    pub age_parameter: String,
    pub age_f32: ReadBoundFile,
    pub age_q24: ReadBoundFile,
    pub exp_q31: ReadBoundFile,
    pub potential_metadata: ReadBoundFile,
    pub tokenizer: ReadBoundFile,
    pub exp_entries: usize,
    pub exp_step_log2: i32,
    pub score_fractional_bits: u32,
    pub value_fractional_bits: u32,
    pub weight_fractional_bits: u32,
    pub quantization: String,
    pub exp_policy: String,
    pub reduction: String,
    pub producer_policy: String,
    /// Omitted entirely in schema /1, retaining its serialized interpretation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub age_q4: Option<AgeQ4Metadata>,
    /// Exclusive schema /3 branch; omitted in both historical schemas.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub age_residual_q4: Option<AgeResidualQ4Metadata>,
}
impl GeometricReadMetadata {
    fn age_mode(&self) -> Result<AgeMode> {
        match (
            self.schema.as_str(),
            self.age_q4.as_ref(),
            self.age_residual_q4.as_ref(),
        ) {
            (SCHEMA, None, None) => Ok(AgeMode::Raw),
            (Q4_AGE_SCHEMA, Some(_), None) => Ok(AgeMode::Quarter),
            (Q4_AGE_RESIDUAL_SCHEMA, None, Some(_)) => Ok(AgeMode::PriorResidual),
            _ => Err(invalid(
                "native read schema/age policy combination is unsupported",
            )),
        }
    }
}
fn metadata(source: &ReadSourceBinding, age: &[i64], exp: &[u32]) -> GeometricReadMetadata {
    GeometricReadMetadata {
        schema: SCHEMA.into(),
        source_files: source.files.clone(),
        layer: source.layer,
        heads: source.heads,
        context: source.context,
        value_width: source.value_width,
        kernel_max_context: MAX_CONTEXT,
        age_parameter: source.age_parameter.clone(),
        age_f32: bound(&f32_bytes(&source.age)),
        age_q24: bound(&age_bytes(age)),
        exp_q31: bound(&exp_bytes(exp)),
        potential_metadata: bound(&source.potential_metadata),
        tokenizer: bound(&source.tokenizer),
        exp_entries: EXP_TABLE_LEN,
        exp_step_log2: EXP_STEP_LOG2,
        score_fractional_bits: 24,
        value_fractional_bits: 16,
        weight_fractional_bits: 31,
        quantization: QUANTIZATION.into(),
        exp_policy: EXP_POLICY.into(),
        reduction: REDUCTION.into(),
        producer_policy: PRODUCER.into(),
        age_q4: None,
        age_residual_q4: None,
    }
}

pub struct CompiledGeometricRead {
    metadata: GeometricReadMetadata,
    age: Vec<i64>,
    exp: Vec<u32>,
    tokenizer: Vec<u8>,
    age_coefficients_q4: Option<Vec<u8>>,
}
impl CompiledGeometricRead {
    pub fn compile(
        age: &Tensor,
        potential: &CompiledGeometricPotentials,
        source: &ReadSourceBinding,
    ) -> Result<Self> {
        if f32_bytes(&tensor_age(age, source.heads, source.context)?) != f32_bytes(&source.age)
            || serde_json::to_vec_pretty(potential.metadata())? != source.potential_metadata
        {
            return Err(invalid(
                "live native read age/potential differs from saved source",
            ));
        }
        Self::from_source(source)
    }
    /// Explicit strict conversion. Raw saved F32 age remains the immutable
    /// source identity; neither range clipping nor source mutation is allowed.
    pub fn compile_q4_age(
        age: &Tensor,
        potential: &CompiledGeometricPotentials,
        source: &ReadSourceBinding,
    ) -> Result<Self> {
        if f32_bytes(&tensor_age(age, source.heads, source.context)?) != f32_bytes(&source.age)
            || serde_json::to_vec_pretty(potential.metadata())? != source.potential_metadata
        {
            return Err(invalid(
                "live q4 read age/potential differs from saved raw source",
            ));
        }
        Self::from_source_q4(source)
    }
    /// Explicit /3 conversion; a failed /2 conversion never selects this mode.
    pub fn compile_q4_age_residual(
        age: &Tensor,
        potential: &CompiledGeometricPotentials,
        source: &ReadSourceBinding,
    ) -> Result<Self> {
        if f32_bytes(&tensor_age(age, source.heads, source.context)?) != f32_bytes(&source.age)
            || serde_json::to_vec_pretty(potential.metadata())? != source.potential_metadata
        {
            return Err(invalid(
                "live residual age/potential differs from saved raw source",
            ));
        }
        Self::from_source_q4_residual(source)
    }
    fn from_source(source: &ReadSourceBinding) -> Result<Self> {
        let age = source
            .age
            .iter()
            .copied()
            .map(quantize_score_q24)
            .collect::<Result<Vec<_>>>()?;
        let exp = canonical_exp();
        NativeGeometricRead::new(source.context, source.value_width, &exp)
            .map_err(|e| invalid(e.to_string()))?;
        Ok(Self {
            metadata: metadata(source, &age, &exp),
            age,
            exp,
            tokenizer: source.tokenizer.clone(),
            age_coefficients_q4: None,
        })
    }
    fn from_source_q4(source: &ReadSourceBinding) -> Result<Self> {
        let config = AgeQ4Config {
            heads: source.heads,
            context: source.context,
        };
        let codec = q4_age(config, &source.age)?;
        let age = codec.age_q24().to_vec();
        let exp = canonical_exp();
        NativeGeometricRead::new(source.context, source.value_width, &exp)
            .map_err(|e| invalid(e.to_string()))?;
        let mut metadata = metadata(source, &age, &exp);
        metadata.schema = Q4_AGE_SCHEMA.into();
        metadata.quantization = Q4_AGE_QUANTIZATION.into();
        metadata.age_q4 = Some(AgeQ4Metadata {
            schema: uor_r4_integer::geometric_age_q4::SCHEMA.into(),
            policy: uor_r4_integer::geometric_age_q4::POLICY.into(),
            config,
            coefficient_count: source.age.len(),
            packed_coefficients: bound(codec.packed()),
            training_policy: Q4_AGE_TRAINING_POLICY.into(),
        });
        Ok(Self {
            metadata,
            age,
            exp,
            tokenizer: source.tokenizer.clone(),
            age_coefficients_q4: Some(codec.packed().to_vec()),
        })
    }
    fn from_source_q4_residual(source: &ReadSourceBinding) -> Result<Self> {
        let config = AgeResidualQ4Config {
            heads: source.heads,
            context: source.context,
        };
        let codec = q4_age_residual(config, &source.age)?;
        let age = codec.age_q24().to_vec();
        let exp = canonical_exp();
        NativeGeometricRead::new(source.context, source.value_width, &exp)
            .map_err(|e| invalid(e.to_string()))?;
        let mut metadata = metadata(source, &age, &exp);
        metadata.schema = Q4_AGE_RESIDUAL_SCHEMA.into();
        metadata.quantization = Q4_AGE_RESIDUAL_QUANTIZATION.into();
        metadata.age_residual_q4 = Some(AgeResidualQ4Metadata {
            schema: uor_r4_integer::geometric_age_q4::RESIDUAL_SCHEMA.into(),
            policy: uor_r4_integer::geometric_age_q4::RESIDUAL_POLICY.into(),
            config,
            prior_identity: uor_r4_integer::geometric_age_q4::PRIOR_IDENTITY.into(),
            prior_phase: uor_r4_integer::geometric_age_q4::PRIOR_PHASE.into(),
            prior_head_shifts: codec
                .prior_head_shifts()
                .map_err(|e| invalid(e.to_string()))?,
            prior_q24: bound(&age_bytes(codec.prior_q24())),
            residual_exponent: uor_r4_integer::geometric_age_q4::RESIDUAL_EXPONENT,
            coefficient_count: source.age.len(),
            packed_coefficients: bound(codec.packed()),
            training_policy: Q4_AGE_RESIDUAL_TRAINING_POLICY.into(),
        });
        Ok(Self {
            metadata,
            age,
            exp,
            tokenizer: source.tokenizer.clone(),
            age_coefficients_q4: Some(codec.packed().to_vec()),
        })
    }
    pub fn metadata(&self) -> &GeometricReadMetadata {
        &self.metadata
    }
    /// Admitted immutable exp table for the separately versioned wide reducer.
    pub fn exp_q31(&self) -> &[u32] {
        &self.exp
    }

    /// Admitted immutable score units; zero age is lag zero.
    pub fn age_q24(&self) -> &[i64] {
        &self.age
    }
    pub fn is_q4_age(&self) -> bool {
        self.age_coefficients_q4.is_some()
    }
    pub fn is_q4_age_residual(&self) -> bool {
        self.metadata.age_residual_q4.is_some()
    }
    pub fn packed_age_coefficients(&self) -> Option<&[u8]> {
        self.age_coefficients_q4.as_deref()
    }
    pub fn age_training_view(&self, age: &Tensor) -> Result<Tensor> {
        if !self.is_q4_age() || age.dims() != [self.metadata.heads, self.metadata.context] {
            return Err(invalid(
                "q4 age training view requires strict artifact and matching shape",
            ));
        }
        if self.is_q4_age_residual() {
            q4_age_residual_training_view(age, self.metadata.heads, self.metadata.context)
        } else {
            q4_age_training_view(age, self.metadata.heads, self.metadata.context)
        }
    }
    /// A training-lineage check, not equality to the immutable compiled ages.
    /// The caller must consume `age_training_view` for current hard score values.
    pub fn validate_q4_training(
        &self,
        age: &Tensor,
        potential: &CompiledGeometricPotentials,
    ) -> Result<()> {
        if !self.is_q4_age()
            || bound(&serde_json::to_vec_pretty(potential.metadata())?)
                != self.metadata.potential_metadata
        {
            return Err(invalid("q4 age training artifact/potential differs"));
        }
        let values = tensor_age(age, self.metadata.heads, self.metadata.context)?;
        if self.is_q4_age_residual() {
            q4_age_residual(
                AgeResidualQ4Config {
                    heads: self.metadata.heads,
                    context: self.metadata.context,
                },
                &values,
            )?;
        } else {
            q4_age(
                AgeQ4Config {
                    heads: self.metadata.heads,
                    context: self.metadata.context,
                },
                &values,
            )?;
        }
        Ok(())
    }

    pub fn validate_for(
        &self,
        age: &Tensor,
        potential: &CompiledGeometricPotentials,
    ) -> Result<()> {
        if bound(&f32_bytes(&tensor_age(
            age,
            self.metadata.heads,
            self.metadata.context,
        )?)) != self.metadata.age_f32
            || bound(&serde_json::to_vec_pretty(potential.metadata())?)
                != self.metadata.potential_metadata
        {
            return Err(invalid(
                "native read is stale for live age/potential coefficients",
            ));
        }
        Ok(())
    }
    pub fn save(&self, directory: &Path) -> Result<()> {
        fs::create_dir(directory)?;
        for (name, bytes) in [
            (FILES[0], serde_json::to_vec_pretty(&self.metadata)?),
            (FILES[1], age_bytes(&self.age)),
            (FILES[2], exp_bytes(&self.exp)),
            (FILES[3], self.tokenizer.clone()),
        ] {
            fs::File::create_new(directory.join(name))?.write_all(&bytes)?;
        }
        if let Some(packed) = &self.age_coefficients_q4 {
            let filename = self
                .metadata
                .age_mode()?
                .packed_file()
                .ok_or_else(|| invalid("native read packed age has no declared file"))?;
            fs::File::create_new(directory.join(filename))?.write_all(packed)?;
        }
        Ok(())
    }
    pub fn load(directory: &Path, source: &ReadSourceBinding) -> Result<Self> {
        let mut names = BTreeSet::new();
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                return Err(invalid("native read artifact has nonregular file"));
            }
            names.insert(
                entry
                    .file_name()
                    .into_string()
                    .map_err(|_| invalid("native read artifact non-UTF8 name"))?,
            );
        }
        let saved: GeometricReadMetadata =
            serde_json::from_slice(&fs::read(directory.join(FILES[0]))?)?;
        let mode = saved.age_mode()?;
        let mut expected_names: BTreeSet<String> = FILES.iter().map(|s| s.to_string()).collect();
        if let Some(filename) = mode.packed_file() {
            expected_names.insert(filename.into());
        }
        if names != expected_names {
            return Err(invalid("native read artifact file set differs"));
        }
        let expected = match mode {
            AgeMode::Raw => Self::from_source(source)?,
            AgeMode::Quarter => Self::from_source_q4(source)?,
            AgeMode::PriorResidual => Self::from_source_q4_residual(source)?,
        };
        let age = age_bytes(&expected.age);
        let exp = exp_bytes(&expected.exp);
        if fs::metadata(directory.join(FILES[1]))?.len() != age.len() as u64
            || fs::metadata(directory.join(FILES[2]))?.len() != exp.len() as u64
            || fs::metadata(directory.join(FILES[3]))?.len() != expected.tokenizer.len() as u64
        {
            return Err(invalid("native read artifact payload length differs"));
        }
        if let Some(packed) = &expected.age_coefficients_q4 {
            let filename = mode
                .packed_file()
                .ok_or_else(|| invalid("native read packed age has no declared file"))?;
            if fs::metadata(directory.join(filename))?.len() != packed.len() as u64
                || fs::read(directory.join(filename))? != *packed
            {
                return Err(invalid(
                    "native read packed age differs from raw-source recompilation",
                ));
            }
        }
        if saved != expected.metadata
            || fs::read(directory.join(FILES[1]))? != age
            || fs::read(directory.join(FILES[2]))? != exp
            || fs::read(directory.join(FILES[3]))? != expected.tokenizer
        {
            return Err(invalid(
                "native read artifact differs from actual source recompilation",
            ));
        }
        Ok(expected)
    }
}

/// One causal row. Equal payloads at distinct positions retain separate
/// numerator entries. A zero output does not imply absence or NoRead.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeReadRow {
    pub output_q16: Vec<i32>,
    pub occurrence_weights_q31: Vec<u64>,
    pub no_read_weight_q31: u64,
    pub total_weight_q31: u64,
    pub max_score_q24: i64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeReadTrace {
    pub batch: usize,
    pub heads: usize,
    pub time: usize,
    pub value_width: usize,
    /// Quantized actual caller inputs, [B,H,T] and [B,H,T,V].
    pub no_read_q24: Vec<i64>,
    pub values_q16: Vec<i32>,
    /// [B,H,T] row order; row q contains exactly q+1 occurrence weights.
    pub rows: Vec<NativeReadRow>,
}
impl NativeReadTrace {
    fn row(&self, batch: usize, head: usize, query: usize) -> Result<&NativeReadRow> {
        if batch >= self.batch || head >= self.heads || query >= self.time {
            return Err(invalid("native read observer coordinate outside trace"));
        }
        let at = batch
            .checked_mul(self.heads)
            .and_then(|x| x.checked_add(head))
            .and_then(|x| x.checked_mul(self.time))
            .and_then(|x| x.checked_add(query))
            .ok_or_else(|| invalid("native read observer index overflow"))?;
        let row = self
            .rows
            .get(at)
            .ok_or_else(|| invalid("native read observer row absent"))?;
        let total = row
            .occurrence_weights_q31
            .iter()
            .try_fold(row.no_read_weight_q31, |sum, &w| sum.checked_add(w));
        if row.occurrence_weights_q31.len() != query + 1
            || row.total_weight_q31 == 0
            || total != Some(row.total_weight_q31)
        {
            return Err(invalid("native read observer weights/total inconsistent"));
        }
        Ok(row)
    }
    /// Exact summed occurrence numerator and normalization denominator. Source
    /// labels are used only here, after the native numerical result exists.
    pub fn source_fraction(
        &self,
        batch: usize,
        head: usize,
        query: usize,
        sources: &[usize],
    ) -> Result<(u64, u64)> {
        let row = self.row(batch, head, query)?;
        let mut seen = BTreeSet::new();
        let mut numerator = 0u64;
        for &source in sources {
            if source > query || !seen.insert(source) {
                return Err(invalid(
                    "native read source observer has future or duplicate occurrence",
                ));
            }
            numerator = numerator
                .checked_add(row.occurrence_weights_q31[source])
                .ok_or_else(|| invalid("native read source numerator overflow"))?;
        }
        Ok((numerator, row.total_weight_q31))
    }
    pub fn source_mass(
        &self,
        batch: usize,
        head: usize,
        query: usize,
        sources: &[usize],
    ) -> Result<f32> {
        let (n, d) = self.source_fraction(batch, head, query, sources)?;
        Ok((n as f64 / d as f64) as f32)
    }
    pub fn no_read_mass(&self, batch: usize, head: usize, query: usize) -> Result<f32> {
        let row = self.row(batch, head, query)?;
        Ok((row.no_read_weight_q31 as f64 / row.total_weight_q31 as f64) as f32)
    }
}
pub struct NativeReadOutput {
    pub read: Tensor,
    pub trace: NativeReadTrace,
}

/// Offline bridge. Scores use [B,H,T,T] order and remain Q24. Future score
/// cells are never consumed. No source/answer masks are accepted by this API.
pub fn reduce_native(
    potential_q24: &[i64],
    no_read: &Tensor,
    values: &Tensor,
    age: &Tensor,
    potential: &CompiledGeometricPotentials,
    compiled: &CompiledGeometricRead,
) -> Result<NativeReadOutput> {
    compiled.validate_for(age, potential)?;
    let (batch, heads, time, value_width) = values.dims4()?;
    let positions = batch
        .checked_mul(heads)
        .and_then(|n| n.checked_mul(time))
        .ok_or_else(|| invalid("native read input size overflow"))?;
    if batch == 0
        || time == 0
        || time > compiled.metadata.context
        || heads != compiled.metadata.heads
        || value_width != compiled.metadata.value_width
        || no_read.dims() != [batch, heads, time]
        || values.dtype() != DType::F32
        || no_read.dtype() != DType::F32
        || !values.device().is_cpu()
        || !no_read.device().is_cpu()
        || positions.checked_mul(time) != Some(potential_q24.len())
    {
        return Err(invalid("native read requires matching CPU F32 NoRead[B,H,T],values[B,H,T,V],Q24scores[B,H,T,T]"));
    }
    let values_q16 = values
        .flatten_all()?
        .to_vec1::<f32>()?
        .into_iter()
        .map(quantize_value_q16)
        .collect::<Result<Vec<_>>>()?;
    reduce_native_q16(
        potential_q24,
        no_read,
        &values_q16,
        value_width,
        age,
        potential,
        compiled,
    )
}

/// Offline bridge for already decoded integer geometric payloads. It avoids
/// a Q16 -> F32 -> Q16 round trip, which loses low bits at large magnitudes.
/// The actual Q16 inputs remain in the trace; tensor reconstruction occurs
/// only after the native numerical result for the unfinished float decoder.
pub fn reduce_native_q16(
    potential_q24: &[i64],
    no_read: &Tensor,
    values_q16: &[i32],
    value_width: usize,
    age: &Tensor,
    potential: &CompiledGeometricPotentials,
    compiled: &CompiledGeometricRead,
) -> Result<NativeReadOutput> {
    compiled.validate_for(age, potential)?;
    let (batch, heads, time) = no_read.dims3()?;
    let positions = batch
        .checked_mul(heads)
        .and_then(|n| n.checked_mul(time))
        .ok_or_else(|| invalid("native Q16 read input size overflow"))?;
    if batch == 0
        || time == 0
        || time > compiled.metadata.context
        || heads != compiled.metadata.heads
        || value_width != compiled.metadata.value_width
        || no_read.dtype() != DType::F32
        || !no_read.device().is_cpu()
        || positions.checked_mul(time) != Some(potential_q24.len())
        || positions.checked_mul(value_width) != Some(values_q16.len())
    {
        return Err(invalid("native Q16 read requires matching CPU F32 NoRead[B,H,T],Q16values[B,H,T,V],Q24scores[B,H,T,T]"));
    }
    let no_read_q24 = no_read
        .flatten_all()?
        .to_vec1::<f32>()?
        .into_iter()
        .map(quantize_score_q24)
        .collect::<Result<Vec<_>>>()?;
    reduce_native_raw_q24(
        potential_q24,
        &no_read_q24,
        values_q16,
        batch,
        heads,
        time,
        value_width,
        potential,
        compiled,
    )
}

/// Offline composition bridge for fully integer inputs. This preserves raw
/// Q24 NoRead and Q16 payload bits; F32 is reconstructed only AFTER reduction.
/// The caller binds the live producer/age identities before entering this API.
/// Existing reducer metadata still means its original compiled age/exp source.
pub fn reduce_native_raw_q24(
    potential_q24: &[i64],
    no_read_q24: &[i64],
    values_q16: &[i32],
    batch: usize,
    heads: usize,
    time: usize,
    value_width: usize,
    potential: &CompiledGeometricPotentials,
    compiled: &CompiledGeometricRead,
) -> Result<NativeReadOutput> {
    let positions = batch
        .checked_mul(heads)
        .and_then(|x| x.checked_mul(time))
        .ok_or_else(|| invalid("native raw read layout overflow"))?;
    if batch == 0
        || time == 0
        || time > compiled.metadata.context
        || heads != compiled.metadata.heads
        || value_width != compiled.metadata.value_width
        || positions != no_read_q24.len()
        || positions.checked_mul(time) != Some(potential_q24.len())
        || positions.checked_mul(value_width) != Some(values_q16.len())
        || bound(&serde_json::to_vec_pretty(potential.metadata())?)
            != compiled.metadata.potential_metadata
    {
        return Err(invalid(
            "native raw Q24 read shape/potential identity differs",
        ));
    }
    let mut kernel =
        NativeGeometricRead::new(compiled.metadata.context, value_width, &compiled.exp)
            .map_err(|e| invalid(e.to_string()))?;
    let mut output = Vec::with_capacity(values_q16.len());
    let mut rows = Vec::with_capacity(positions);
    let mut causal_age = Vec::with_capacity(time);
    for b in 0..batch {
        for h in 0..heads {
            let first = (b * heads + h) * time;
            for q in 0..time {
                causal_age.clear();
                causal_age
                    .extend((0..=q).map(|j| compiled.age[h * compiled.metadata.context + q - j]));
                let score_at = (first + q) * time;
                let row = kernel
                    .reduce(
                        &potential_q24[score_at..score_at + q + 1],
                        &causal_age,
                        no_read_q24[first + q],
                        &values_q16[first * value_width..(first + q + 1) * value_width],
                    )
                    .map_err(|e| invalid(e.to_string()))?;
                output.extend(
                    row.output_q16
                        .iter()
                        .map(|&x| (f64::from(x) / 65_536.) as f32),
                );
                rows.push(NativeReadRow {
                    output_q16: row.output_q16.to_vec(),
                    occurrence_weights_q31: row.occurrence_weights_q31.to_vec(),
                    no_read_weight_q31: row.no_read_weight_q31,
                    total_weight_q31: row.total_weight_q31,
                    max_score_q24: row.max_score_q24,
                });
            }
        }
    }
    Ok(NativeReadOutput {
        read: Tensor::from_vec(output, (batch, heads, time, value_width), &Device::Cpu)?,
        trace: NativeReadTrace {
            batch,
            heads,
            time,
            value_width,
            no_read_q24: no_read_q24.to_vec(),
            values_q16: values_q16.to_vec(),
            rows,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_address::GeometricAddressConfig;
    use crate::geometric_potential_native::PotentialSourceBinding;
    use crate::geometric_span::GeometricSpanConfig;
    use crate::geometric_stack::{ReadIdentityLatch, ReadScore};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    const REGISTRY: &[u8] = b"native reducer fixture token registry 0..3/v1";
    const HEADS: usize = 2;
    const CONTEXT: usize = 8;
    const VALUE_WIDTH: usize = 8;

    struct Fixture {
        directory: PathBuf,
        model: StackModel,
        potential: CompiledGeometricPotentials,
        source: ReadSourceBinding,
    }
    impl Fixture {
        fn age(&self) -> &Tensor {
            self.model.variables()["layers.02.read.age"].as_tensor()
        }
        fn compile(&self) -> Result<CompiledGeometricRead> {
            CompiledGeometricRead::compile(self.age(), &self.potential, &self.source)
        }
    }
    fn fixture(ages: Vec<f32>) -> Result<Fixture> {
        static SERIAL: AtomicU64 = AtomicU64::new(0);
        let clock = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| invalid(e.to_string()))?
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "uor-native-read-{}-{clock}-{}",
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
            rotation_group: Default::default(),
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
        model.variables()["layers.02.read.age"].set(&Tensor::from_vec(
            ages,
            (HEADS, CONTEXT),
            &Device::Cpu,
        )?)?;
        model.save(&base)?;
        let potential_source = PotentialSourceBinding::from_directory(&base, REGISTRY)?;
        let potential = CompiledGeometricPotentials::compile(
            &model.geometric_address_potentials()?,
            &address,
            &potential_source,
        )?;
        let source = ReadSourceBinding::from_directory(&base, REGISTRY, &potential, 2)?;
        Ok(Fixture {
            directory,
            model,
            potential,
            source,
        })
    }

    #[test]
    fn native_read_q4_age_live_grid_units_and_identity_gradient() -> Result<()> {
        use candle_core::Var;
        let shadow = Var::from_vec(
            vec![-1.75f32, -0.125, 0., 0.125, 0.249, 1.75],
            (2, 3),
            &Device::Cpu,
        )?;
        let expected = vec![-1.75f32, -0.25, 0., 0.25, 0.25, 1.75];
        let view = q4_age_training_view(shadow.as_tensor(), 2, 3)?;
        assert_eq!(view.flatten_all()?.to_vec1::<f32>()?, expected);
        let coefficients = Tensor::from_vec(vec![1f32, 2., 3., 4., 5., 6.], (2, 3), &Device::Cpu)?;
        let gradient = (&view * &coefficients)?.sum_all()?.backward()?;
        assert_eq!(
            gradient
                .get(shadow.as_tensor())
                .ok_or_else(|| invalid("q4 age adjoint missing"))?
                .flatten_all()?
                .to_vec1::<f32>()?,
            vec![1f32, 2., 3., 4., 5., 6.]
        );
        let mut changed = vec![-1.75f32, -0.125, 0., 0.125, 0.249, 1.75];
        changed[0] = 0.4;
        shadow.set(&Tensor::from_vec(changed, (2, 3), &Device::Cpu)?)?;
        assert_eq!(
            q4_age_training_view(shadow.as_tensor(), 2, 3)?
                .flatten_all()?
                .to_vec1::<f32>()?[0],
            0.5
        );
        assert!(q4_age_training_view(shadow.as_tensor(), 1, 6).is_err());
        assert!(q4_age_training_view(&shadow.to_dtype(DType::F64)?, 2, 3).is_err());
        for invalid_value in [
            f32::NAN,
            f32::INFINITY,
            f32::NEG_INFINITY,
            1.750001,
            -1.750001,
        ] {
            let tensor = Tensor::from_vec(vec![invalid_value], (1, 1), &Device::Cpu)?;
            assert!(q4_age_training_view(&tensor, 1, 1).is_err());
        }
        Ok(())
    }

    #[test]
    fn native_read_q4_age_artifact_recompile_inventory_and_resealed_tamper() -> Result<()> {
        let fixture = fixture(
            (0..HEADS * CONTEXT)
                .map(|i| i as f32 * 0.021 - 0.126)
                .collect(),
        )?;
        let legacy = fixture.compile()?;
        let strict = CompiledGeometricRead::compile_q4_age(
            fixture.age(),
            &fixture.potential,
            &fixture.source,
        )?;
        assert!(!legacy.is_q4_age());
        assert!(strict.is_q4_age());
        assert_eq!(strict.exp_q31(), legacy.exp_q31());
        assert_eq!(strict.metadata.age_f32, legacy.metadata.age_f32);
        assert_eq!(strict.metadata.source_files, legacy.metadata.source_files);
        assert_ne!(strict.age_q24(), legacy.age_q24());
        let legacy_json = serde_json::to_value(legacy.metadata())?;
        assert!(legacy_json.get("age_q4").is_none());
        // A missing optional field retains the schema /1 metadata bytes.
        let reloaded_metadata: GeometricReadMetadata = serde_json::from_value(legacy_json)?;
        assert_eq!(
            serde_json::to_vec_pretty(&reloaded_metadata)?,
            serde_json::to_vec_pretty(legacy.metadata())?
        );
        let legacy_directory = fixture.directory.join("legacy-age");
        legacy.save(&legacy_directory)?;
        assert_eq!(native_file_names(&legacy_directory)?, FILES);
        let artifact = fixture.directory.join("q4-age");
        strict.save(&artifact)?;
        assert_eq!(native_file_names(&artifact)?.len(), 5);
        assert_eq!(
            CompiledGeometricRead::load(&artifact, &fixture.source)?.metadata(),
            strict.metadata()
        );
        assert!(legacy.age_training_view(fixture.age()).is_err());
        let grid = strict
            .age_training_view(fixture.age())?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert_eq!(
            grid.iter()
                .map(|&v| quantize_score_q24(v))
                .collect::<Result<Vec<_>>>()?,
            strict.age_q24()
        );
        let original_metadata = fs::read(artifact.join(FILES[0]))?;
        for filename in [Q4_AGE_FILE, FILES[1], FILES[2]] {
            let original = fs::read(artifact.join(filename))?;
            let mut changed = original.clone();
            changed[0] ^= 1;
            fs::write(artifact.join(filename), &changed)?;
            let mut lied = strict.metadata.clone();
            if filename == Q4_AGE_FILE {
                lied.age_q4
                    .as_mut()
                    .ok_or_else(|| invalid("missing test q4 metadata"))?
                    .packed_coefficients = bound(&changed);
            } else if filename == FILES[1] {
                lied.age_q24 = bound(&changed);
            } else {
                lied.exp_q31 = bound(&changed);
            }
            fs::write(artifact.join(FILES[0]), serde_json::to_vec_pretty(&lied)?)?;
            assert!(CompiledGeometricRead::load(&artifact, &fixture.source).is_err());
            fs::write(artifact.join(filename), original)?;
            fs::write(artifact.join(FILES[0]), &original_metadata)?;
        }
        let packed = fs::read(artifact.join(Q4_AGE_FILE))?;
        fs::remove_file(artifact.join(Q4_AGE_FILE))?;
        assert!(CompiledGeometricRead::load(&artifact, &fixture.source).is_err());
        fs::write(artifact.join(Q4_AGE_FILE), packed)?;
        fs::write(artifact.join("unbound"), b"extra")?;
        assert!(CompiledGeometricRead::load(&artifact, &fixture.source).is_err());
        fs::remove_file(artifact.join("unbound"))?;
        for schema in [SCHEMA, "uor-r4.geometric-read-reducer/unknown"] {
            let mut lied = strict.metadata.clone();
            lied.schema = schema.into();
            fs::write(artifact.join(FILES[0]), serde_json::to_vec_pretty(&lied)?)?;
            assert!(CompiledGeometricRead::load(&artifact, &fixture.source).is_err());
            assert!(native_file_names(&artifact).is_err());
        }
        fs::write(artifact.join(FILES[0]), original_metadata)?;
        assert!(CompiledGeometricRead::load(&artifact, &fixture.source).is_ok());
        fs::remove_dir_all(fixture.directory)?;
        Ok(())
    }

    #[test]
    fn native_read_q4_age_raw_source_identity_and_live_training_are_distinct() -> Result<()> {
        let fixture = fixture(vec![0.; HEADS * CONTEXT])?;
        let strict = CompiledGeometricRead::compile_q4_age(
            fixture.age(),
            &fixture.potential,
            &fixture.source,
        )?;
        let artifact = fixture.directory.join("q4-age");
        strict.save(&artifact)?;
        let mut changed = vec![0f32; HEADS * CONTEXT];
        changed[0] = f32::from_bits(1); // Same grid, different raw source identity.
        let same_grid = Tensor::from_vec(changed.clone(), (HEADS, CONTEXT), &Device::Cpu)?;
        assert!(strict.validate_for(&same_grid, &fixture.potential).is_err());
        assert!(CompiledGeometricRead::compile_q4_age(
            &same_grid,
            &fixture.potential,
            &fixture.source
        )
        .is_err());
        strict.validate_q4_training(&same_grid, &fixture.potential)?;
        let mut changed_source = fixture.source;
        changed_source.age[0] = f32::from_bits(1);
        assert!(CompiledGeometricRead::load(&artifact, &changed_source).is_err());
        changed[0] = 0.5;
        let live = Tensor::from_vec(changed, (HEADS, CONTEXT), &Device::Cpu)?;
        strict.validate_q4_training(&live, &fixture.potential)?;
        assert_eq!(strict.age_q24()[0], 0); // The saved artifact is still immutable.
        assert_eq!(
            strict
                .age_training_view(&live)?
                .flatten_all()?
                .to_vec1::<f32>()?[0],
            0.5
        );
        changed_source.age[0] = 1.8;
        assert!(CompiledGeometricRead::from_source_q4(&changed_source).is_err());
        fs::remove_dir_all(fixture.directory)?;
        Ok(())
    }

    #[test]
    fn native_read_age_residual_live_prior_grid_f64_ties_and_gradient() -> Result<()> {
        use candle_core::Var;
        let cfg = AgeResidualQ4Config {
            heads: 2,
            context: 128,
        };
        let prior = cfg.prior_q24().map_err(|e| invalid(e.to_string()))?;
        let mut raw = prior
            .iter()
            .map(|&v| (v as f64 / 16_777_216.) as f32)
            .collect::<Vec<_>>();
        raw[0] = -0.0625; // Negative exact half tie at lag0.
        raw[1] = -1e-10; // Residual just BELOW +1/16, not an F32-rounded tie.
        raw[128] = 0.875;
        raw[255] -= 0.875;
        let shadow = Var::from_vec(raw.clone(), (2, 128), &Device::Cpu)?;
        let view = q4_age_residual_training_view(shadow.as_tensor(), 2, 128)?;
        let values = view.flatten_all()?.to_vec1::<f32>()?;
        assert_eq!(values[0], -0.125);
        assert_eq!(values[1], -0.0625);
        assert_eq!(values[40], -2.5); // Raw age outside the /2 coefficient range.
        assert_eq!(values[128], 0.875);
        assert_eq!(values[255], -127. / 256. - 0.875);
        assert!(q4_age_training_view(shadow.as_tensor(), 2, 128).is_err());
        let mut weights = vec![0f32; 256];
        weights[0] = 2.;
        weights[1] = -3.;
        weights[40] = 4.;
        weights[128] = 5.;
        let loss =
            (&view * &Tensor::from_vec(weights.clone(), (2, 128), &Device::Cpu)?)?.sum_all()?;
        let gradient = loss.backward()?;
        assert_eq!(
            gradient
                .get(shadow.as_tensor())
                .ok_or_else(|| invalid("age residual gradient missing"))?
                .flatten_all()?
                .to_vec1::<f32>()?,
            weights
        );
        raw[1] = 0.; // Now residual is exactly +1/16: ties away selects +1/8.
        shadow.set(&Tensor::from_vec(raw.clone(), (2, 128), &Device::Cpu)?)?;
        assert_eq!(
            q4_age_residual_training_view(shadow.as_tensor(), 2, 128)?
                .flatten_all()?
                .to_vec1::<f32>()?[1],
            0.0625
        );
        assert!(q4_age_residual_training_view(shadow.as_tensor(), 1, 128).is_err());
        for bad in [
            f32::NAN,
            f32::INFINITY,
            f32::NEG_INFINITY,
            0.875001,
            -0.875001,
        ] {
            raw[0] = bad;
            let age = Tensor::from_vec(raw.clone(), (2, 128), &Device::Cpu)?;
            assert!(q4_age_residual_training_view(&age, 2, 128).is_err());
        }
        Ok(())
    }

    #[test]
    fn native_read_age_residual_schema_roundtrip_and_prior_payload_tamper() -> Result<()> {
        let cfg = AgeResidualQ4Config {
            heads: HEADS,
            context: CONTEXT,
        };
        let prior = cfg.prior_q24().map_err(|e| invalid(e.to_string()))?;
        let ages = prior
            .iter()
            .enumerate()
            .map(|(i, &v)| (v as f64 / 16_777_216.) as f32 + if i % 2 == 0 { 0.2 } else { -0.2 })
            .collect();
        let fixture = fixture(ages)?;
        let legacy = fixture.compile()?;
        let quarter = CompiledGeometricRead::compile_q4_age(
            fixture.age(),
            &fixture.potential,
            &fixture.source,
        )?;
        let residual = CompiledGeometricRead::compile_q4_age_residual(
            fixture.age(),
            &fixture.potential,
            &fixture.source,
        )?;
        assert!(!legacy.is_q4_age());
        assert!(quarter.is_q4_age() && !quarter.is_q4_age_residual());
        assert!(residual.is_q4_age() && residual.is_q4_age_residual());
        assert_eq!(residual.exp_q31(), legacy.exp_q31());
        assert_eq!(residual.exp_q31(), quarter.exp_q31());
        assert_eq!(residual.metadata.age_f32, legacy.metadata.age_f32);
        assert_eq!(residual.metadata.source_files, legacy.metadata.source_files);
        let residual_meta = residual
            .metadata
            .age_residual_q4
            .as_ref()
            .ok_or_else(|| invalid("missing residual metadata"))?;
        assert_eq!(residual_meta.prior_head_shifts, [4, 8]);
        assert_eq!(residual_meta.prior_q24, bound(&age_bytes(&prior)));
        assert_eq!(residual_meta.residual_exponent, -3);
        for (i, compiled) in [&legacy, &quarter, &residual].into_iter().enumerate() {
            let path = fixture.directory.join(format!("age-mode-{i}"));
            compiled.save(&path)?;
            assert_eq!(
                CompiledGeometricRead::load(&path, &fixture.source)?.metadata(),
                compiled.metadata()
            );
            let names = native_file_names(&path)?;
            assert_eq!(names.len(), if i == 0 { 4 } else { 5 });
            if i < 2 {
                assert!(serde_json::to_value(compiled.metadata())?
                    .get("age_residual_q4")
                    .is_none());
            } else {
                assert!(names.contains(&Q4_AGE_RESIDUAL_FILE));
                assert!(!names.contains(&Q4_AGE_FILE));
                assert!(serde_json::to_value(compiled.metadata())?
                    .get("age_q4")
                    .is_none());
            }
        }
        let artifact = fixture.directory.join("age-mode-2");
        let original_metadata = fs::read(artifact.join(FILES[0]))?;
        // Every fixed-prior identifier must be regenerated, never trusted just
        // because the altered metadata and numerical payload are self-consistent.
        for field in [
            "prior_identity",
            "prior_phase",
            "prior_head_shifts",
            "prior_q24",
            "residual_exponent",
        ] {
            let mut lied = residual.metadata.clone();
            let inner = lied
                .age_residual_q4
                .as_mut()
                .ok_or_else(|| invalid("missing residual metadata"))?;
            match field {
                "prior_identity" => inner.prior_identity.push_str("changed"),
                "prior_phase" => inner.prior_phase.push_str("lag1"),
                "prior_head_shifts" => inner.prior_head_shifts.swap(0, 1),
                "prior_q24" => inner.prior_q24 = bound(&age_bytes(&vec![0; prior.len()])),
                _ => inner.residual_exponent = -2,
            }
            fs::write(artifact.join(FILES[0]), serde_json::to_vec_pretty(&lied)?)?;
            assert!(CompiledGeometricRead::load(&artifact, &fixture.source).is_err());
        }
        fs::write(artifact.join(FILES[0]), &original_metadata)?;
        for filename in [Q4_AGE_RESIDUAL_FILE, FILES[1]] {
            let original = fs::read(artifact.join(filename))?;
            let mut changed = original.clone();
            changed[0] ^= 1;
            fs::write(artifact.join(filename), &changed)?;
            let mut lied = residual.metadata.clone();
            if filename == Q4_AGE_RESIDUAL_FILE {
                lied.age_residual_q4
                    .as_mut()
                    .ok_or_else(|| invalid("missing residual metadata"))?
                    .packed_coefficients = bound(&changed);
            } else {
                lied.age_q24 = bound(&changed);
            }
            fs::write(artifact.join(FILES[0]), serde_json::to_vec_pretty(&lied)?)?;
            assert!(CompiledGeometricRead::load(&artifact, &fixture.source).is_err());
            fs::write(artifact.join(filename), original)?;
            fs::write(artifact.join(FILES[0]), &original_metadata)?;
        }
        let mut ambiguous = residual.metadata.clone();
        ambiguous.age_q4 = quarter.metadata.age_q4.clone();
        fs::write(
            artifact.join(FILES[0]),
            serde_json::to_vec_pretty(&ambiguous)?,
        )?;
        assert!(native_file_names(&artifact).is_err());
        assert!(CompiledGeometricRead::load(&artifact, &fixture.source).is_err());
        fs::write(artifact.join(FILES[0]), &original_metadata)?;
        fs::write(artifact.join(Q4_AGE_FILE), [0])?; // No aliases or extra /2 payloads.
        assert!(CompiledGeometricRead::load(&artifact, &fixture.source).is_err());
        fs::remove_file(artifact.join(Q4_AGE_FILE))?;
        assert!(CompiledGeometricRead::load(&artifact, &fixture.source).is_ok());
        fs::remove_dir_all(fixture.directory)?;
        Ok(())
    }

    #[test]
    fn native_read_age_residual_raw_identity_and_live_mode_dispatch() -> Result<()> {
        let cfg = AgeResidualQ4Config {
            heads: HEADS,
            context: CONTEXT,
        };
        let ages = cfg
            .prior_q24()
            .map_err(|e| invalid(e.to_string()))?
            .iter()
            .map(|&x| (x as f64 / 16_777_216.) as f32)
            .collect();
        let fixture = fixture(ages)?;
        let residual = CompiledGeometricRead::compile_q4_age_residual(
            fixture.age(),
            &fixture.potential,
            &fixture.source,
        )?;
        let artifact = fixture.directory.join("residual-source");
        residual.save(&artifact)?;
        let mut current = fixture.source.age.clone();
        current[0] = f32::from_bits(1); // Same packed residual, different source bits.
        let within_cell = Tensor::from_vec(current.clone(), (HEADS, CONTEXT), &Device::Cpu)?;
        assert!(residual
            .validate_for(&within_cell, &fixture.potential)
            .is_err());
        assert!(CompiledGeometricRead::compile_q4_age_residual(
            &within_cell,
            &fixture.potential,
            &fixture.source
        )
        .is_err());
        residual.validate_q4_training(&within_cell, &fixture.potential)?;
        assert_eq!(
            residual
                .age_training_view(&within_cell)?
                .flatten_all()?
                .to_vec1::<f32>()?[0],
            0.
        );
        current[0] = 0.2;
        let moved = Tensor::from_vec(current, (HEADS, CONTEXT), &Device::Cpu)?;
        residual.validate_q4_training(&moved, &fixture.potential)?;
        assert_eq!(
            residual
                .age_training_view(&moved)?
                .flatten_all()?
                .to_vec1::<f32>()?[0],
            0.25
        );
        assert_eq!(residual.age_q24()[0], 0); // Saved expansion remains immutable.
        let mut changed_source = fixture.source;
        changed_source.age[0] = f32::from_bits(1);
        assert!(CompiledGeometricRead::load(&artifact, &changed_source).is_err());
        changed_source.age[0] = 0.875001;
        assert!(CompiledGeometricRead::from_source_q4_residual(&changed_source).is_err());
        fs::remove_dir_all(fixture.directory)?;
        Ok(())
    }

    #[test]
    fn native_read_q4_potential_rebind_preserves_age_and_exponential_payloads() -> Result<()> {
        let fixture = fixture(
            (0..HEADS * CONTEXT)
                .map(|i| i as f32 * 0.013 - 0.07)
                .collect(),
        )?;
        let before = fixture.compile()?;
        let base = fixture.directory.join("base");
        let source_directory = fixture.directory.join("strict-potential-source");
        let strict = crate::geometric_potential_q4::PotentialQ4Weights::from_base(&base, REGISTRY)?;
        strict.save(&source_directory)?;
        let potential_source = PotentialSourceBinding::from_directory(&base, REGISTRY)?;
        let potential =
            CompiledGeometricPotentials::compile_q4(&strict, &source_directory, &potential_source)?;
        let rebound = ReadSourceBinding::from_directory(&base, REGISTRY, &potential, 2)?;
        assert!(before.validate_for(fixture.age(), &potential).is_err());
        let after = CompiledGeometricRead::compile(fixture.age(), &potential, &rebound)?;
        assert_eq!(before.age, after.age);
        assert_eq!(before.exp, after.exp);
        assert_eq!(before.metadata.age_f32, after.metadata.age_f32);
        assert_ne!(
            before.metadata.potential_metadata,
            after.metadata.potential_metadata
        );
        let artifact = fixture.directory.join("rebound-read");
        after.save(&artifact)?;
        let reload = CompiledGeometricRead::load(&artifact, &rebound)?;
        assert_eq!(reload.metadata, after.metadata);
        assert!(CompiledGeometricRead::load(&artifact, &fixture.source).is_err());
        fs::remove_dir_all(fixture.directory)?;
        Ok(())
    }

    #[test]
    fn native_read_quantization_endpoints_and_fixed_exp_policy() -> Result<()> {
        assert_eq!(quantize_score_q24(0.5 / 16_777_216.)?, 1);
        assert_eq!(quantize_score_q24(-0.5 / 16_777_216.)?, -1);
        assert_eq!(quantize_value_q16(0.5 / 65_536.)?, 1);
        assert_eq!(quantize_value_q16(-0.5 / 65_536.)?, -1);
        assert_eq!(quantize_score_q24(-549_755_813_888.)?, i64::MIN);
        assert!(quantize_score_q24(549_755_813_888.).is_err());
        assert_eq!(quantize_value_q16(-32_768.)?, i32::MIN);
        assert!(quantize_value_q16(32_768.).is_err());
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(quantize_score_q24(value).is_err());
            assert!(quantize_value_q16(value).is_err());
        }
        let table = canonical_exp();
        assert_eq!(table.len(), 8194);
        assert_eq!(u64::from(table[0]), WEIGHT_ONE);
        assert_eq!(table[8193], 0);
        assert!(table.windows(2).all(|pair| pair[0] >= pair[1]));
        NativeGeometricRead::new(CONTEXT, VALUE_WIDTH, &table)
            .map_err(|e| invalid(e.to_string()))?;
        Ok(())
    }

    #[test]
    fn native_read_saved_source_recompile_resealed_tamper_and_stale_age() -> Result<()> {
        let fixture = fixture(vec![0.; HEADS * CONTEXT])?;
        let compiled = fixture.compile()?;
        let artifact = fixture.directory.join("compiled");
        compiled.save(&artifact)?;
        assert!(compiled.save(&artifact).is_err());
        let loaded = CompiledGeometricRead::load(&artifact, &fixture.source)?;
        assert_eq!(compiled.metadata(), loaded.metadata());
        assert_eq!(compiled.age, loaded.age);
        assert_eq!(compiled.exp, loaded.exp);
        assert_eq!(loaded.metadata().layer, 2);
        assert_eq!(loaded.metadata().age_parameter, "layers.02.read.age");
        let base = fixture.directory.join("base");
        assert!(ReadSourceBinding::from_directory(&base, REGISTRY, &fixture.potential, 0).is_err());
        assert!(ReadSourceBinding::from_directory(
            &base,
            b"changed registry",
            &fixture.potential,
            2
        )
        .is_err());

        // Re-sealing a modified payload's self-reported digest cannot override
        // independently reconstructed saved age or canonical exp bytes.
        let original_metadata = fs::read(artifact.join(FILES[0]))?;
        for (index, is_age) in [(1, true), (2, false)] {
            let original = fs::read(artifact.join(FILES[index]))?;
            let mut changed = original.clone();
            changed[4] ^= 1;
            fs::write(artifact.join(FILES[index]), &changed)?;
            let mut lied = compiled.metadata.clone();
            if is_age {
                lied.age_q24 = bound(&changed);
            } else {
                lied.exp_q31 = bound(&changed);
            }
            fs::write(artifact.join(FILES[0]), serde_json::to_vec(&lied)?)?;
            assert!(CompiledGeometricRead::load(&artifact, &fixture.source).is_err());
            fs::write(artifact.join(FILES[index]), original)?;
            fs::write(artifact.join(FILES[0]), &original_metadata)?;
        }
        fs::write(artifact.join("unbound"), b"extra")?;
        assert!(CompiledGeometricRead::load(&artifact, &fixture.source).is_err());
        fs::remove_file(artifact.join("unbound"))?;
        let mut stale = vec![0f32; HEADS * CONTEXT];
        stale[0] = f32::from_bits(1); // Same Q24 value, different saved F32 bits.
        let stale = Tensor::from_vec(stale, (HEADS, CONTEXT), &Device::Cpu)?;
        assert!(compiled.validate_for(&stale, &fixture.potential).is_err());
        assert!(
            CompiledGeometricRead::compile(&stale, &fixture.potential, &fixture.source).is_err()
        );
        assert!(CompiledGeometricRead::load(&artifact, &fixture.source).is_ok());
        fs::remove_dir_all(fixture.directory)?;
        Ok(())
    }

    #[test]
    fn native_read_direct_q16_retains_bits_lost_by_float_payload_roundtrip() -> Result<()> {
        let fixture = fixture(vec![0.; HEADS * CONTEXT])?;
        let compiled = fixture.compile()?;
        let high = (1_i32 << 30) + 1;
        assert_ne!(
            quantize_value_q16((f64::from(high) / 65_536.) as f32)?,
            high
        );
        let null = Tensor::from_vec(vec![-40f32; HEADS], (1, HEADS, 1), &Device::Cpu)?;
        let values = vec![high; HEADS * VALUE_WIDTH];
        let output = reduce_native_q16(
            &vec![0; HEADS],
            &null,
            &values,
            VALUE_WIDTH,
            fixture.age(),
            &fixture.potential,
            &compiled,
        )?;
        assert_eq!(output.trace.values_q16, values);
        for row in output.trace.rows {
            assert_eq!(row.no_read_weight_q31, 0);
            assert_eq!(row.output_q16, vec![high; VALUE_WIDTH]);
        }
        assert!(reduce_native_q16(
            &vec![0; HEADS],
            &null,
            &values[..values.len() - 1],
            VALUE_WIDTH,
            fixture.age(),
            &fixture.potential,
            &compiled,
        )
        .is_err());
        fs::remove_dir_all(fixture.directory)?;
        Ok(())
    }

    #[test]
    fn native_read_raw_null_preserves_q24_bits() -> Result<()> {
        let fixture = fixture(vec![0.; HEADS * CONTEXT])?;
        let compiled = fixture.compile()?;
        let high = (1i64 << 30) + 1;
        assert_ne!(quantize_score_q24((high as f64 / 16777216.) as f32)?, high);
        let null = vec![high; HEADS];
        let payload = vec![65536; HEADS * VALUE_WIDTH];
        let scores = vec![high; HEADS];
        let raw = reduce_native_raw_q24(
            &scores,
            &null,
            &payload,
            1,
            HEADS,
            1,
            VALUE_WIDTH,
            &fixture.potential,
            &compiled,
        )?;
        assert_eq!(raw.trace.no_read_q24, null);
        for row in raw.trace.rows {
            assert_eq!(row.max_score_q24, high);
            assert_eq!(row.no_read_weight_q31, WEIGHT_ONE);
            assert_eq!(row.total_weight_q31, 2 * WEIGHT_ONE);
            assert_eq!(row.output_q16, vec![32768; VALUE_WIDTH]);
        }
        assert!(reduce_native_raw_q24(
            &scores,
            &null[..1],
            &payload,
            1,
            HEADS,
            1,
            VALUE_WIDTH,
            &fixture.potential,
            &compiled
        )
        .is_err());
        fs::remove_dir_all(fixture.directory)?;
        Ok(())
    }

    #[test]
    fn native_read_causal_layout_cancellation_and_exact_source_numerators() -> Result<()> {
        let fixture = fixture(vec![0.; HEADS * CONTEXT])?;
        let compiled = fixture.compile()?;
        let (batch, time) = (2, 3);
        let null = Tensor::zeros((batch, HEADS, time), DType::F32, &Device::Cpu)?;
        let mut values = vec![0f32; batch * HEADS * time * VALUE_WIDTH];
        for b in 0..batch {
            for h in 0..HEADS {
                // At query 1 the two real occurrences cancel, but retain positive
                // source mass and a separately represented NoRead probability.
                let unit = (1 + b * HEADS + h) as f32;
                for (q, value) in [unit, -unit, unit].into_iter().enumerate() {
                    let at = ((b * HEADS + h) * time + q) * VALUE_WIDTH;
                    values[at..at + VALUE_WIDTH].fill(value);
                }
            }
        }
        let input = Tensor::from_vec(
            values.clone(),
            (batch, HEADS, time, VALUE_WIDTH),
            &Device::Cpu,
        )?;
        let mut scores = vec![0i64; batch * HEADS * time * time];
        for row in 0..batch * HEADS * time {
            let q = row % time;
            // Future cells are not admitted to reduction, even when their
            // consumption would cause an i64 maximum-difference overflow.
            scores[row * time + q + 1..(row + 1) * time].fill(i64::MIN);
        }
        let actual = reduce_native(
            &scores,
            &null,
            &input,
            fixture.age(),
            &fixture.potential,
            &compiled,
        )?;
        assert_eq!(actual.read.dims(), [batch, HEADS, time, VALUE_WIDTH]);
        for b in 0..batch {
            for h in 0..HEADS {
                let unit = (1 + b * HEADS + h) as i32;
                let q0 = actual.trace.row(b, h, 0)?;
                assert_eq!(q0.output_q16, vec![unit * 32_768; VALUE_WIDTH]);
                let q1 = actual.trace.row(b, h, 1)?;
                assert_eq!(q1.output_q16, vec![0; VALUE_WIDTH]);
                assert_eq!(
                    actual.trace.source_fraction(b, h, 1, &[0, 1])?,
                    (2 * WEIGHT_ONE, 3 * WEIGHT_ONE)
                );
                assert!(actual.trace.no_read_mass(b, h, 1)? > 0.);
                assert_eq!(actual.trace.source_mass(b, h, 2, &[0, 2])?, 0.5);
                assert_eq!(
                    actual.trace.source_fraction(b, h, 2, &[])?,
                    (0, 4 * WEIGHT_ONE)
                );
                assert!(actual.trace.source_mass(b, h, 2, &[0, 0]).is_err());
                assert!(actual.trace.source_mass(b, h, 1, &[2]).is_err());
            }
        }
        assert!(actual.trace.source_mass(batch, 0, 0, &[0]).is_err());
        // Changing only the last payload leaves every earlier reduction and
        // its weights unchanged; distinct equal payloads remain occurrences.
        for bh in 0..batch * HEADS {
            let at = (bh * time + 2) * VALUE_WIDTH;
            values[at..at + VALUE_WIDTH].fill(27.);
        }
        let changed = Tensor::from_vec(values, input.shape(), &Device::Cpu)?;
        let later = reduce_native(
            &scores,
            &null,
            &changed,
            fixture.age(),
            &fixture.potential,
            &compiled,
        )?;
        for b in 0..batch {
            for h in 0..HEADS {
                for q in 0..2 {
                    assert_eq!(actual.trace.row(b, h, q)?, later.trace.row(b, h, q)?);
                }
            }
        }
        fs::remove_dir_all(fixture.directory)?;
        Ok(())
    }

    #[test]
    fn native_read_age_orientation_and_input_refusals() -> Result<()> {
        let mut ages = vec![0.; HEADS * CONTEXT];
        for age in 0..CONTEXT {
            ages[age] = age as f32;
            ages[CONTEXT + age] = -(age as f32);
        }
        let fixture = fixture(ages)?;
        let compiled = fixture.compile()?;
        let time = 3;
        let null = Tensor::zeros((1, HEADS, time), DType::F32, &Device::Cpu)?;
        let values = Tensor::ones((1, HEADS, time, VALUE_WIDTH), DType::F32, &Device::Cpu)?;
        let scores = vec![0; HEADS * time * time];
        let actual = reduce_native(
            &scores,
            &null,
            &values,
            fixture.age(),
            &fixture.potential,
            &compiled,
        )?;
        let increasing = &actual.trace.row(0, 0, 2)?.occurrence_weights_q31;
        assert!(increasing[0] > increasing[1] && increasing[1] > increasing[2]);
        let decreasing = &actual.trace.row(0, 1, 2)?.occurrence_weights_q31;
        assert!(decreasing[0] < decreasing[1] && decreasing[1] < decreasing[2]);
        assert!(reduce_native(
            &scores[..scores.len() - 1],
            &null,
            &values,
            fixture.age(),
            &fixture.potential,
            &compiled
        )
        .is_err());
        let wrong = Tensor::zeros((1, HEADS, time + 1), DType::F32, &Device::Cpu)?;
        assert!(reduce_native(
            &scores,
            &wrong,
            &values,
            fixture.age(),
            &fixture.potential,
            &compiled
        )
        .is_err());
        for bad in [f32::NAN, 32_768.] {
            let mut payload = vec![1.; HEADS * time * VALUE_WIDTH];
            payload[0] = bad;
            let payload = Tensor::from_vec(payload, values.shape(), &Device::Cpu)?;
            assert!(reduce_native(
                &scores,
                &null,
                &payload,
                fixture.age(),
                &fixture.potential,
                &compiled
            )
            .is_err());
        }
        let bad_null = Tensor::from_vec(
            vec![f32::INFINITY; HEADS * time],
            null.shape(),
            &Device::Cpu,
        )?;
        assert!(reduce_native(
            &scores,
            &bad_null,
            &values,
            fixture.age(),
            &fixture.potential,
            &compiled
        )
        .is_err());
        let mut bad_scores = scores;
        bad_scores[time * 2] = i64::MAX; // positive age at query2/source0 overflows.
        assert!(reduce_native(
            &bad_scores,
            &null,
            &values,
            fixture.age(),
            &fixture.potential,
            &compiled
        )
        .is_err());
        fs::remove_dir_all(fixture.directory)?;
        Ok(())
    }
}
