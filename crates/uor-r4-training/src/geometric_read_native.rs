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
use uor_r4_integer::geometric_read::{
    NativeGeometricRead, EXP_STEP_LOG2, EXP_TABLE_LEN, MAX_CONTEXT, MAX_VALUE_WIDTH, WEIGHT_ONE,
};

use crate::geometric_potential_native::{CompiledGeometricPotentials, PotentialSourceBinding};
use crate::geometric_stack::{
    StackArch, StackConfig, StackModel, GEOMETRIC_ADDRESS_RECORD, GEOMETRIC_SPAN_RECORD,
};
use crate::{invalid, sha256_bytes, Result};

pub const SCHEMA: &str = "uor-r4.geometric-read-reducer/1";
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
    }
}

pub struct CompiledGeometricRead {
    metadata: GeometricReadMetadata,
    age: Vec<i64>,
    exp: Vec<u32>,
    tokenizer: Vec<u8>,
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
        if names != FILES.iter().map(|s| s.to_string()).collect() {
            return Err(invalid("native read artifact file set differs"));
        }
        let expected = Self::from_source(source)?;
        let age = age_bytes(&expected.age);
        let exp = exp_bytes(&expected.exp);
        if fs::metadata(directory.join(FILES[1]))?.len() != age.len() as u64
            || fs::metadata(directory.join(FILES[2]))?.len() != exp.len() as u64
            || fs::metadata(directory.join(FILES[3]))?.len() != expected.tokenizer.len() as u64
        {
            return Err(invalid("native read artifact payload length differs"));
        }
        let saved: GeometricReadMetadata =
            serde_json::from_slice(&fs::read(directory.join(FILES[0]))?)?;
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
