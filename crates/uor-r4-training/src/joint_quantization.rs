//! Frozen dyadic parameter quantization for the offline recurrent learner.
//!
//! This is floating-point hard simulation and a parameter codec, not an integer
//! serving kernel. Calibration runs once on the declared parent. The scale is
//! not learned or recomputed during optimization. CPU ties round away from zero,
//! matching Candle 0.9.2's F32 `Round` implementation. Zero has one canonical code.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use candle_core::{DType, Device, Tensor, Var};
use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{invalid, Result};

pub const SPEC_SCHEMA: &str = "uor-r4.joint-dyadic-quantization/2";
pub const HARD_SCHEMA: &str = "uor-r4.joint-dyadic-hard-parameters/1";
pub const CALIBRATION_RULE: &str = "frozen-parent; signed4 minimum F64 sum of squared F32 reconstruction error over clamped/deduplicated [ceiling-2,ceiling-1,ceiling], ties larger exponent; signed16 ceiling only; ceiling=ceil(log2(maxabs/qmax)) clamped [-24,16]; exact-zero ceiling=0; ties round away from zero";
pub const MIN_EXPONENT: i16 = -24;
pub const MAX_EXPONENT: i16 = 16;
pub const HARD_PARAMETERS_FILE: &str = "hard-parameters.bin";
pub const HARD_PARAMETERS_MANIFEST_FILE: &str = "hard-parameters.json";
pub const HARD_DESCRIPTOR_FILE: &str = HARD_PARAMETERS_MANIFEST_FILE;
const ENCODING: &str = "signed4-low-nibble-first-reserved-minus8;signed16-le-reserved-minus32768";
const MAX_PARAMETERS: usize = 256;
const MAX_TOTAL_ELEMENTS: usize = 16 * 1024 * 1024;
const MAX_DESCRIPTOR_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuantizationSpec {
    pub schema: String,
    #[serde(deserialize_with = "unique_parameters")]
    pub parameters: BTreeMap<String, ParameterQuantization>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParameterQuantization {
    pub shape: Vec<usize>,
    pub bits: u8,
    /// One exponent per output row for matrices; one exponent for a vector.
    pub row_exponents: Vec<i16>,
}

fn unique_parameters<'de, D>(
    deserializer: D,
) -> std::result::Result<BTreeMap<String, ParameterQuantization>, D::Error>
where
    D: Deserializer<'de>,
{
    struct UniqueParameters;
    impl<'de> Visitor<'de> for UniqueParameters {
        type Value = BTreeMap<String, ParameterQuantization>;
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a map of unique parameter names")
        }
        fn visit_map<M>(self, mut access: M) -> std::result::Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut result = BTreeMap::new();
            while let Some((name, value)) = access.next_entry::<String, ParameterQuantization>()? {
                if result.len() >= MAX_PARAMETERS || result.insert(name.clone(), value).is_some() {
                    return Err(serde::de::Error::custom(format!(
                        "duplicate parameter or parameter-count limit: {name}"
                    )));
                }
            }
            Ok(result)
        }
    }
    deserializer.deserialize_map(UniqueParameters)
}

fn required_bits(name: &str) -> Result<u8> {
    if name.is_empty()
        || name.len() > 256
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_'))
    {
        return Err(invalid("invalid quantized parameter name"));
    }
    if name.ends_with(".weight") {
        Ok(4)
    } else if name.ends_with(".bias") || name == "read.age" {
        Ok(16)
    } else {
        Err(invalid(format!("unsupported quantized parameter {name}")))
    }
}

fn elements(shape: &[usize]) -> Result<usize> {
    if !matches!(shape.len(), 1 | 2) || shape.contains(&0) {
        return Err(invalid(
            "quantized parameters require nonempty vectors or matrices",
        ));
    }
    let count = shape.iter().try_fold(1usize, |count, &dimension| {
        count
            .checked_mul(dimension)
            .ok_or_else(|| invalid("parameter shape overflow"))
    })?;
    if count > MAX_TOTAL_ELEMENTS {
        return Err(invalid("quantized parameter element limit"));
    }
    Ok(count)
}

fn step(exponent: i16) -> Result<f32> {
    if !(MIN_EXPONENT..=MAX_EXPONENT).contains(&exponent) {
        return Err(invalid("dyadic exponent outside [-24,16]"));
    }
    Ok(2f32.powi(i32::from(exponent)))
}

fn limits(bits: u8) -> Result<(i32, i32)> {
    match bits {
        4 => Ok((-7, 7)),
        16 => Ok((-32767, 32767)),
        _ => Err(invalid("parameter codes must have 4 or 16 bits")),
    }
}

impl ParameterQuantization {
    fn validate(&self, name: &str) -> Result<usize> {
        let count = elements(&self.shape)?;
        if self.bits != required_bits(name)? || (self.bits == 16 && self.shape.len() != 1) {
            return Err(invalid(format!(
                "parameter quantization kind differs for {name}"
            )));
        }
        let rows = if self.shape.len() == 2 {
            self.shape[0]
        } else {
            1
        };
        if self.row_exponents.len() != rows {
            return Err(invalid(format!(
                "parameter scale-row count differs for {name}"
            )));
        }
        for &exponent in &self.row_exponents {
            step(exponent)?;
        }
        Ok(count)
    }

    fn row_width(&self) -> usize {
        if self.shape.len() == 2 {
            self.shape[1]
        } else {
            self.shape[0]
        }
    }

    fn scale_tensor(&self, device: &Device) -> Result<Tensor> {
        let values = self
            .row_exponents
            .iter()
            .map(|&value| step(value))
            .collect::<Result<Vec<_>>>()?;
        if self.shape.len() == 2 {
            Ok(Tensor::from_vec(values, (self.shape[0], 1), device)?)
        } else {
            Ok(Tensor::from_vec(values, (1,), device)?)
        }
    }
}

impl QuantizationSpec {
    /// Structural bounds also apply to decoded, untrusted descriptors.
    pub fn validate_structure(&self) -> Result<()> {
        if self.schema != SPEC_SCHEMA
            || self.parameters.is_empty()
            || self.parameters.len() > MAX_PARAMETERS
        {
            return Err(invalid(
                "invalid dyadic quantization schema or parameter inventory",
            ));
        }
        let mut total = 0usize;
        for (name, parameter) in &self.parameters {
            total = total
                .checked_add(parameter.validate(name)?)
                .ok_or_else(|| invalid("parameter total overflow"))?;
            if total > MAX_TOTAL_ELEMENTS {
                return Err(invalid("hard parameter total element limit"));
            }
        }
        Ok(())
    }

    pub fn validate(&self, variables: &BTreeMap<String, Var>) -> Result<()> {
        self.validate_structure()?;
        if !self.parameters.keys().eq(variables.keys()) {
            return Err(invalid("quantization/variable parameter inventory differs"));
        }
        for (name, variable) in variables {
            if variable.dtype() != DType::F32 || variable.dims() != self.parameters[name].shape {
                return Err(invalid(format!(
                    "quantization shape/dtype differs for {name}"
                )));
            }
        }
        Ok(())
    }

    /// Prepare once per parameter per forward, not once per recurrent token.
    /// Finite input values are the learner's numeric contract; calibration and
    /// serialization check them on the host. This hot path adds no host sync.
    pub fn parameter(
        &self,
        name: &str,
        input: &Tensor,
        strength: f64,
        training: bool,
    ) -> Result<Tensor> {
        if self.schema != SPEC_SCHEMA {
            return Err(invalid("quantization schema mismatch"));
        }
        let parameter = self
            .parameters
            .get(name)
            .ok_or_else(|| invalid(format!("missing quantized parameter {name}")))?;
        parameter.validate(name)?;
        if input.dims() != parameter.shape {
            return Err(invalid(format!(
                "quantized parameter shape differs for {name}"
            )));
        }
        let (qmin, qmax) = limits(parameter.bits)?;
        fake_quant_scaled(
            input,
            &parameter.scale_tensor(input.device())?,
            qmin,
            qmax,
            strength,
            training,
        )
    }
}

/// Calibrate on the declared parent, then retain the returned spec unchanged.
/// Signed4 rows minimize parent reconstruction error over three nearby dyadic
/// scales; signed16 vectors use the ceiling scale. Exact all-zero rows use
/// exponent 0. Clipping is reported rather than adapting scales during training.
pub fn calibrate(variables: &BTreeMap<String, Var>) -> Result<QuantizationSpec> {
    let mut parameters = BTreeMap::new();
    for (name, variable) in variables {
        if variable.dtype() != DType::F32 {
            return Err(invalid(format!("non-F32 calibration parameter {name}")));
        }
        let bits = required_bits(name)?;
        let shape = variable.dims().to_vec();
        elements(&shape)?;
        if bits == 16 && shape.len() != 1 {
            return Err(invalid("additive parameters require one vector scale"));
        }
        let values = variable
            .as_detached_tensor()
            .flatten_all()?
            .to_vec1::<f32>()?;
        if values.iter().any(|value| !value.is_finite()) {
            return Err(invalid(format!("nonfinite calibration parameter {name}")));
        }
        let width = if shape.len() == 2 { shape[1] } else { shape[0] };
        let (_, qmax) = limits(bits)?;
        let row_exponents = values
            .chunks_exact(width)
            .map(|row| -> Result<i16> {
                let maximum = row
                    .iter()
                    .fold(0f32, |maximum, value| maximum.max(value.abs()));
                let ceiling = if maximum == 0.0 {
                    0
                } else {
                    (f64::from(maximum) / f64::from(qmax))
                        .log2()
                        .ceil()
                        .clamp(f64::from(MIN_EXPONENT), f64::from(MAX_EXPONENT))
                        as i16
                };
                if bits == 16 {
                    return Ok(ceiling);
                }
                let candidates: BTreeSet<_> = [ceiling - 2, ceiling - 1, ceiling]
                    .into_iter()
                    .map(|exponent| exponent.clamp(MIN_EXPONENT, MAX_EXPONENT))
                    .collect();
                let mut chosen = ceiling;
                let mut best_error = f64::INFINITY;
                for candidate in candidates {
                    let scale = step(candidate)?;
                    let error: f64 = row
                        .iter()
                        .map(|&value| {
                            let code = (value / scale).clamp(-7.0, 7.0).round();
                            let difference = f64::from(code * scale) - f64::from(value);
                            difference * difference
                        })
                        .sum();
                    if error < best_error || (error == best_error && candidate > chosen) {
                        best_error = error;
                        chosen = candidate;
                    }
                }
                Ok(chosen)
            })
            .collect::<Result<Vec<_>>>()?;
        parameters.insert(
            name.clone(),
            ParameterQuantization {
                shape,
                bits,
                row_exponents,
            },
        );
    }
    let spec = QuantizationSpec {
        schema: SPEC_SCHEMA.into(),
        parameters,
    };
    spec.validate(variables)?;
    Ok(spec)
}

fn validate_strength(input: &Tensor, qmin: i32, qmax: i32, strength: f64) -> Result<()> {
    if input.dtype() != DType::F32 || input.elem_count() == 0 {
        return Err(invalid("fake quantization requires a nonempty F32 tensor"));
    }
    // Integers up to 2^24 are exact in F32. Parameter codecs use stricter limits.
    if qmin >= qmax || qmin < -16_777_216 || qmax > 16_777_216 {
        return Err(invalid(
            "fake quantization code range is invalid or inexact in F32",
        ));
    }
    if !strength.is_finite() || !(0.0..=1.0).contains(&strength) {
        return Err(invalid("quantization strength must be finite and in [0,1]"));
    }
    Ok(())
}

fn fake_quant_scaled(
    input: &Tensor,
    scales: &Tensor,
    qmin: i32,
    qmax: i32,
    strength: f64,
    training: bool,
) -> Result<Tensor> {
    validate_strength(input, qmin, qmax, strength)?;
    if strength == 0.0 {
        return Ok(if training {
            input.clone()
        } else {
            input.detach()
        });
    }
    let detached = input.detach();
    let scaled = detached.broadcast_div(scales)?;
    let hard = scaled
        .clamp(f64::from(qmin), f64::from(qmax))?
        .round()?
        .broadcast_mul(scales)?;
    // Packed signed integer zero has no sign bit. Canonicalize negative zero.
    let hard = hard.eq(0f64)?.where_cond(&hard.zeros_like()?, &hard)?;
    let quantized = if training {
        // Candle's min/max gradient is 0.5 at equality. WhereCond instead gives
        // derivative 1 at both clip endpoints and 0 strictly outside them.
        let inside = scaled
            .ge(f64::from(qmin))?
            .mul(&scaled.le(f64::from(qmax))?)?;
        let clipped = inside.where_cond(input, &hard)?;
        hard.add(&clipped.sub(&clipped.detach())?)?
    } else {
        hard
    };
    if strength == 1.0 {
        Ok(quantized)
    } else {
        let original = if training { input.clone() } else { detached };
        Ok((original * (1.0 - strength))?.add(&(quantized * strength)?)?)
    }
}

/// Scalar dyadic quantizer for parameters or explicitly configured activations.
/// At strength 1 the forward value is exactly on the hard grid. At intermediate
/// strength the derivative is 1 in range and 1-strength outside the clip range.
pub fn fake_quant(
    input: &Tensor,
    exponent: i16,
    qmin: i32,
    qmax: i32,
    strength: f64,
    training: bool,
) -> Result<Tensor> {
    let scales = Tensor::new(step(exponent)?, input.device())?;
    fake_quant_scaled(input, &scales, qmin, qmax, strength, training)
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CodeEntry {
    name: String,
    offset: u64,
    elements: usize,
    bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct HardDescriptor {
    schema: String,
    encoding: String,
    calibration_rule: String,
    specification: QuantizationSpec,
    specification_sha256: String,
    payload_bytes: u64,
    payload_sha256: String,
    binding_sha256: String,
    parameters: Vec<CodeEntry>,
    /// Diagnostics against the values at export, not required for decoding.
    parameter_statistics: BTreeMap<String, ParameterStatistics>,
    numerical_scope: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ParameterStatistics {
    elements: usize,
    clipped_values: usize,
    zero_codes: usize,
    distinct_codes: usize,
    max_absolute_error: f64,
    squared_error_sum: f64,
}

fn hash(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn binding_hash(specification: &[u8], payload: &[u8]) -> Result<String> {
    let mut hasher = Sha256::new();
    hasher.update(HARD_SCHEMA.as_bytes());
    hasher.update([0]);
    hasher.update(
        u64::try_from(specification.len())
            .map_err(|_| invalid("spec length overflow"))?
            .to_le_bytes(),
    );
    hasher.update(specification);
    hasher.update(
        u64::try_from(payload.len())
            .map_err(|_| invalid("payload length overflow"))?
            .to_le_bytes(),
    );
    hasher.update(payload);
    Ok(hex::encode(hasher.finalize()))
}

fn layout(spec: &QuantizationSpec) -> Result<(Vec<CodeEntry>, u64)> {
    spec.validate_structure()?;
    let mut entries = Vec::with_capacity(spec.parameters.len());
    let mut offset = 0u64;
    for (name, parameter) in &spec.parameters {
        let count = elements(&parameter.shape)?;
        let length = match parameter.bits {
            4 => count.checked_add(1).map(|value| value / 2),
            16 => count.checked_mul(2),
            _ => None,
        }
        .ok_or_else(|| invalid("hard parameter byte-count overflow"))?;
        let bytes =
            u64::try_from(length).map_err(|_| invalid("hard parameter byte-count overflow"))?;
        entries.push(CodeEntry {
            name: name.clone(),
            offset,
            elements: count,
            bytes,
        });
        offset = offset
            .checked_add(bytes)
            .ok_or_else(|| invalid("hard parameter offset overflow"))?;
    }
    Ok((entries, offset))
}

/// The caller owns/claims the directory. Both outputs are created exclusively.
/// The returned metadata includes distortion and code-use counts at this save.
pub fn save_hard_parameters(
    spec: &QuantizationSpec,
    variables: &BTreeMap<String, Var>,
    directory: &Path,
) -> Result<Value> {
    spec.validate(variables)?;
    let (entries, payload_bytes) = layout(spec)?;
    let mut payload = Vec::with_capacity(
        usize::try_from(payload_bytes).map_err(|_| invalid("payload length overflow"))?,
    );
    let mut statistics = BTreeMap::new();
    for entry in &entries {
        let parameter = &spec.parameters[&entry.name];
        let values = variables[&entry.name]
            .as_detached_tensor()
            .flatten_all()?
            .to_vec1::<f32>()?;
        let (qmin, qmax) = limits(parameter.bits)?;
        let mut stats = ParameterStatistics::default();
        let mut used = BTreeSet::new();
        let mut low_nibble = None;
        for (index, &value) in values.iter().enumerate() {
            if !value.is_finite() {
                return Err(invalid(format!("nonfinite hard parameter {}", entry.name)));
            }
            let scale = step(parameter.row_exponents[index / parameter.row_width()])?;
            // Match Candle F32 division, clamp and round, including half ties.
            let scaled = value / scale;
            let code = scaled.clamp(qmin as f32, qmax as f32).round() as i32;
            let reconstructed = code as f32 * scale;
            let error = f64::from(reconstructed) - f64::from(value);
            stats.elements += 1;
            stats.clipped_values += usize::from(scaled < qmin as f32 || scaled > qmax as f32);
            stats.zero_codes += usize::from(code == 0);
            stats.max_absolute_error = stats.max_absolute_error.max(error.abs());
            stats.squared_error_sum += error * error;
            used.insert(code);
            if parameter.bits == 4 {
                let nibble = (code as i8 as u8) & 0x0f;
                if let Some(low) = low_nibble.take() {
                    payload.push(low | (nibble << 4));
                } else {
                    low_nibble = Some(nibble);
                }
            } else {
                payload.extend_from_slice(&(code as i16).to_le_bytes());
            }
        }
        if let Some(low) = low_nibble {
            payload.push(low); // An odd tensor's unused high nibble is zero.
        }
        stats.distinct_codes = used.len();
        statistics.insert(entry.name.clone(), stats);
    }
    if payload.len() as u64 != payload_bytes {
        return Err(invalid("hard parameter payload length differs from layout"));
    }
    let specification = serde_json::to_vec(spec)?;
    let descriptor = HardDescriptor {
        schema: HARD_SCHEMA.into(), encoding: ENCODING.into(), specification: spec.clone(),
        calibration_rule: CALIBRATION_RULE.into(),
        specification_sha256: hash(&specification), payload_bytes, payload_sha256: hash(&payload),
        binding_sha256: binding_hash(&specification, &payload)?, parameters: entries,
        parameter_statistics: statistics,
        numerical_scope: "Frozen dyadic hard parameters decoded to F32 for offline simulation; no integer serving claim".into(),
    };
    let mut descriptor_bytes = serde_json::to_vec_pretty(&descriptor)?;
    descriptor_bytes.push(b'\n');
    if descriptor_bytes.len() as u64 > MAX_DESCRIPTOR_BYTES {
        return Err(invalid("hard parameter descriptor exceeds size limit"));
    }
    let mut data_file = File::create_new(directory.join(HARD_PARAMETERS_FILE))?;
    let mut descriptor_file = File::create_new(directory.join(HARD_DESCRIPTOR_FILE))?;
    data_file.write_all(&payload)?;
    data_file.sync_all()?;
    descriptor_file.write_all(&descriptor_bytes)?;
    descriptor_file.sync_all()?;
    Ok(serde_json::to_value(descriptor)?)
}

/// Read only the hard descriptor and integer payload. No F32 shadow checkpoint
/// is opened. The model caller additionally checks its expected name/shape set.
pub fn load_hard_parameters(
    directory: &Path,
    device: &Device,
) -> Result<(QuantizationSpec, BTreeMap<String, Var>)> {
    let descriptor_path = directory.join(HARD_DESCRIPTOR_FILE);
    if fs::metadata(&descriptor_path)?.len() > MAX_DESCRIPTOR_BYTES {
        return Err(invalid("hard parameter descriptor exceeds size limit"));
    }
    let descriptor: HardDescriptor = serde_json::from_slice(&fs::read(descriptor_path)?)?;
    if descriptor.schema != HARD_SCHEMA
        || descriptor.encoding != ENCODING
        || descriptor.calibration_rule != CALIBRATION_RULE
    {
        return Err(invalid("hard parameter format mismatch"));
    }
    let (expected_entries, expected_bytes) = layout(&descriptor.specification)?;
    if descriptor.parameters != expected_entries || descriptor.payload_bytes != expected_bytes {
        return Err(invalid(
            "hard parameter names/offsets/shapes/lengths differ",
        ));
    }
    if !descriptor
        .parameter_statistics
        .keys()
        .eq(descriptor.specification.parameters.keys())
    {
        return Err(invalid("hard parameter statistics inventory differs"));
    }
    for entry in &expected_entries {
        let stats = &descriptor.parameter_statistics[&entry.name];
        if stats.elements != entry.elements
            || stats.clipped_values > entry.elements
            || stats.zero_codes > entry.elements
            || stats.distinct_codes > entry.elements
            || !stats.max_absolute_error.is_finite()
            || stats.max_absolute_error < 0.0
            || !stats.squared_error_sum.is_finite()
            || stats.squared_error_sum < 0.0
        {
            return Err(invalid("invalid hard parameter diagnostic statistics"));
        }
    }
    let data_path = directory.join(HARD_PARAMETERS_FILE);
    if fs::metadata(&data_path)?.len() != expected_bytes {
        return Err(invalid(
            "hard parameter payload truncated or has trailing data",
        ));
    }
    let payload = fs::read(data_path)?;
    let specification = serde_json::to_vec(&descriptor.specification)?;
    if payload.len() as u64 != expected_bytes
        || hash(&specification) != descriptor.specification_sha256
        || hash(&payload) != descriptor.payload_sha256
        || binding_hash(&specification, &payload)? != descriptor.binding_sha256
    {
        return Err(invalid(
            "hard parameter payload/specification hash mismatch",
        ));
    }
    let mut variables = BTreeMap::new();
    for entry in &expected_entries {
        let parameter = &descriptor.specification.parameters[&entry.name];
        let start =
            usize::try_from(entry.offset).map_err(|_| invalid("hard parameter offset overflow"))?;
        let length =
            usize::try_from(entry.bytes).map_err(|_| invalid("hard parameter length overflow"))?;
        let end = start
            .checked_add(length)
            .ok_or_else(|| invalid("hard parameter end overflow"))?;
        let bytes = payload
            .get(start..end)
            .ok_or_else(|| invalid("hard parameter offset outside payload"))?;
        if parameter.bits == 4
            && entry.elements % 2 == 1
            && bytes.last().is_some_and(|byte| byte >> 4 != 0)
        {
            return Err(invalid("nonzero unused hard parameter padding nibble"));
        }
        let mut values = Vec::with_capacity(entry.elements);
        for index in 0..entry.elements {
            let code = if parameter.bits == 4 {
                let nibble = (bytes[index / 2] >> (4 * (index % 2))) & 0x0f;
                if nibble == 8 {
                    return Err(invalid("reserved signed4 code -8"));
                }
                if nibble >= 8 {
                    i32::from(nibble) - 16
                } else {
                    i32::from(nibble)
                }
            } else {
                let offset = index * 2;
                let code = i16::from_le_bytes([bytes[offset], bytes[offset + 1]]);
                if code == i16::MIN {
                    return Err(invalid("reserved signed16 code -32768"));
                }
                i32::from(code)
            };
            let value = code as f32 * step(parameter.row_exponents[index / parameter.row_width()])?;
            if !value.is_finite() {
                return Err(invalid("nonfinite decoded hard parameter"));
            }
            values.push(value);
        }
        variables.insert(
            entry.name.clone(),
            Var::from_vec(values, parameter.shape.as_slice(), device)?,
        );
    }
    descriptor.specification.validate(&variables)?;
    Ok((descriptor.specification, variables))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct TemporaryDirectory(PathBuf);
    impl TemporaryDirectory {
        fn new() -> Result<Self> {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "uor-joint-quantization-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path)?;
            Ok(Self(path))
        }
    }
    impl Drop for TemporaryDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn values(tensor: &Tensor) -> Result<Vec<f32>> {
        Ok(tensor.flatten_all()?.to_vec1::<f32>()?)
    }

    #[test]
    fn ste_has_exact_hard_forward_and_inclusive_clipping_gradient() -> Result<()> {
        let input = Var::from_vec(
            vec![-4f32, -3.5, -0.75, -0.25, -0.0, 0.0, 0.25, 0.75, 3.5, 4.0],
            (10,),
            &Device::Cpu,
        )?;
        let hard = fake_quant(input.as_tensor(), -1, -7, 7, 1.0, true)?;
        let expected = vec![-3.5f32, -3.5, -1.0, -0.5, 0.0, 0.0, 0.5, 1.0, 3.5, 3.5];
        assert_eq!(
            values(&hard)?
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            expected
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        );
        let gradients = hard.sum_all()?.backward()?;
        let gradient = gradients
            .get(input.as_tensor())
            .ok_or_else(|| invalid("missing STE gradient"))?;
        assert_eq!(
            values(gradient)?,
            vec![0f32, 1., 1., 1., 1., 1., 1., 1., 1., 0.]
        );
        let blend = fake_quant(input.as_tensor(), -1, -7, 7, 0.25, true)?;
        let gradients = blend.sum_all()?.backward()?;
        assert_eq!(
            values(
                gradients
                    .get(input.as_tensor())
                    .ok_or_else(|| invalid("missing blended STE gradient"))?
            )?,
            vec![0.75f32, 1., 1., 1., 1., 1., 1., 1., 1., 0.75]
        );
        let zero = fake_quant(input.as_tensor(), -1, -7, 7, 0.0, true)?;
        assert_eq!(zero.id(), input.id());
        assert_eq!(
            values(&fake_quant(input.as_tensor(), -1, -7, 7, 1.0, false)?)?,
            expected
        );
        assert!(fake_quant(input.as_tensor(), -25, -7, 7, 1.0, true).is_err());
        assert!(fake_quant(input.as_tensor(), 0, -7, 7, f64::NAN, true).is_err());
        Ok(())
    }

    #[test]
    fn calibration_uses_output_rows_and_frozen_vector_scales() -> Result<()> {
        let variables = BTreeMap::from([
            (
                "embedding.weight".into(),
                Var::from_vec(vec![7f32, -7., 0.875, -0.875], (2, 2), &Device::Cpu)?,
            ),
            (
                "output.norm.weight".into(),
                Var::from_vec(vec![1f32, 1.], (2,), &Device::Cpu)?,
            ),
            (
                "output.bias".into(),
                Var::from_vec(vec![32767f32, -32767.], (2,), &Device::Cpu)?,
            ),
            (
                "read.age".into(),
                Var::from_vec(vec![0f32, 0.], (2,), &Device::Cpu)?,
            ),
            (
                "read.query.weight".into(),
                Var::from_vec(vec![1f32, 0.125, 0.125, 0., 0., 0.], (2, 3), &Device::Cpu)?,
            ),
        ]);
        let spec = calibrate(&variables)?;
        assert_eq!(spec.parameters["embedding.weight"].row_exponents, [0, -3]);
        assert_eq!(spec.parameters["output.norm.weight"].row_exponents, [-2]);
        assert_eq!(spec.parameters["output.bias"].row_exponents, [0]);
        assert_eq!(spec.parameters["read.age"].row_exponents, [0]);
        // Ceiling -2 wastes precision here. Allowing the 1.0 outlier to clip
        // at 0.875 gives smaller row error with exponent -3; zero ties choose 0.
        assert_eq!(spec.parameters["read.query.weight"].row_exponents, [-3, 0]);
        let original = spec.clone();
        variables["embedding.weight"].set(&Tensor::zeros((2, 2), DType::F32, &Device::Cpu)?)?;
        spec.parameter(
            "embedding.weight",
            variables["embedding.weight"].as_tensor(),
            1.0,
            true,
        )?;
        assert_eq!(spec, original);
        Ok(())
    }

    fn fixture() -> Result<(QuantizationSpec, BTreeMap<String, Var>)> {
        let variables = BTreeMap::from([
            (
                "output.norm.weight".into(),
                Var::from_vec(vec![-0.5f32, -0.0, 0.5, 1.5, 3.5], (5,), &Device::Cpu)?,
            ),
            (
                "read.query.weight".into(),
                Var::from_vec(
                    vec![-3.5f32, -0.25, 0.25, -14., -1., 3.],
                    (2, 3),
                    &Device::Cpu,
                )?,
            ),
            (
                "output.bias".into(),
                Var::from_vec(vec![-32768f32, -0.5, 32768.], (3,), &Device::Cpu)?,
            ),
        ]);
        let parameters = BTreeMap::from([
            (
                "output.norm.weight".into(),
                ParameterQuantization {
                    shape: vec![5],
                    bits: 4,
                    row_exponents: vec![-1],
                },
            ),
            (
                "read.query.weight".into(),
                ParameterQuantization {
                    shape: vec![2, 3],
                    bits: 4,
                    row_exponents: vec![-1, 1],
                },
            ),
            (
                "output.bias".into(),
                ParameterQuantization {
                    shape: vec![3],
                    bits: 16,
                    row_exponents: vec![0],
                },
            ),
        ]);
        Ok((
            QuantizationSpec {
                schema: SPEC_SCHEMA.into(),
                parameters,
            },
            variables,
        ))
    }

    #[test]
    fn hard_codec_roundtrip_matches_fake_quant_without_shadow_parameters() -> Result<()> {
        let (spec, variables) = fixture()?;
        let directory = TemporaryDirectory::new()?;
        let metadata = save_hard_parameters(&spec, &variables, &directory.0)?;
        assert_eq!(
            metadata["parameter_statistics"]["output.bias"]["clipped_values"],
            2
        );
        fs::write(
            directory.0.join("model.safetensors"),
            b"not a readable F32 model",
        )?;
        let (loaded_spec, loaded) = load_hard_parameters(&directory.0, &Device::Cpu)?;
        assert_eq!(loaded_spec, spec);
        for (name, original) in &variables {
            let hard = spec.parameter(name, original.as_tensor(), 1.0, false)?;
            let expected = values(&hard)?
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>();
            let actual = values(loaded[name].as_tensor())?
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>();
            assert_eq!(actual, expected, "{name}");
        }
        assert!(save_hard_parameters(&spec, &variables, &directory.0).is_err());
        Ok(())
    }

    fn rewrite_descriptor(
        directory: &Path,
        descriptor: &mut HardDescriptor,
        payload: &[u8],
    ) -> Result<()> {
        let specification = serde_json::to_vec(&descriptor.specification)?;
        descriptor.specification_sha256 = hash(&specification);
        descriptor.payload_sha256 = hash(payload);
        descriptor.binding_sha256 = binding_hash(&specification, payload)?;
        fs::write(directory.join(HARD_PARAMETERS_FILE), payload)?;
        fs::write(
            directory.join(HARD_DESCRIPTOR_FILE),
            serde_json::to_vec(descriptor)?,
        )?;
        Ok(())
    }

    #[test]
    fn hard_codec_rejects_hash_length_layout_and_reserved_codes() -> Result<()> {
        let (spec, variables) = fixture()?;
        let directory = TemporaryDirectory::new()?;
        save_hard_parameters(&spec, &variables, &directory.0)?;
        let original = fs::read(directory.0.join(HARD_PARAMETERS_FILE))?;
        let descriptor: HardDescriptor =
            serde_json::from_slice(&fs::read(directory.0.join(HARD_DESCRIPTOR_FILE))?)?;
        let mut modified = original.clone();
        modified[0] ^= 1;
        fs::write(directory.0.join(HARD_PARAMETERS_FILE), &modified)?;
        assert!(load_hard_parameters(&directory.0, &Device::Cpu).is_err());
        modified = original.clone();
        modified[0..2].copy_from_slice(&i16::MIN.to_le_bytes());
        rewrite_descriptor(&directory.0, &mut descriptor.clone(), &modified)?;
        assert!(load_hard_parameters(&directory.0, &Device::Cpu).is_err());
        modified = original.clone();
        let four_bit = descriptor
            .parameters
            .iter()
            .find(|entry| entry.name == "output.norm.weight")
            .ok_or_else(|| invalid("missing fixture weight"))?;
        modified[four_bit.offset as usize] = (modified[four_bit.offset as usize] & 0xf0) | 8;
        rewrite_descriptor(&directory.0, &mut descriptor.clone(), &modified)?;
        assert!(load_hard_parameters(&directory.0, &Device::Cpu).is_err());
        modified = original.clone();
        modified.push(0);
        rewrite_descriptor(&directory.0, &mut descriptor.clone(), &modified)?;
        assert!(load_hard_parameters(&directory.0, &Device::Cpu).is_err());
        let mut wrong_layout = descriptor.clone();
        wrong_layout.parameters[0].offset = 1;
        rewrite_descriptor(&directory.0, &mut wrong_layout, &original)?;
        assert!(load_hard_parameters(&directory.0, &Device::Cpu).is_err());
        let mut wrong_spec = descriptor;
        wrong_spec
            .specification
            .parameters
            .get_mut("output.norm.weight")
            .ok_or_else(|| invalid("missing fixture weight"))?
            .row_exponents[0] = MAX_EXPONENT + 1;
        rewrite_descriptor(&directory.0, &mut wrong_spec, &original)?;
        assert!(load_hard_parameters(&directory.0, &Device::Cpu).is_err());
        Ok(())
    }
}
