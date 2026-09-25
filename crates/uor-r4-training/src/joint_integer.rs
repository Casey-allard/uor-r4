//! Integer execution bridge for the retained full-context recurrent model.
//!
//! Parameters retain their signed codes and dyadic scales. Numerical execution
//! uses add/subtract, shifts, comparisons, integer table reads, binary products,
//! division and square roots implemented in `joint_integer_math`. No floating
//! model operation or hardware product is requested by these numerical kernels.
//! This remains a dense-parameter, allocating, finite-context prototype. Loading
//! validates legacy metadata; offline table construction and host evaluation
//! are separate from this numerical path. Approximate integer execution does
//! not imply bitwise equivalence to the original F32 accumulation order.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::joint_admission::AdmissionPolicy;
use crate::joint_integer_math::{self as math, MathResult};
use crate::joint_integer_tables::{Tables, TOTAL};
use crate::joint_model::{JointConfig, QuantizedTrainingState, ReadMode, Transport};
use crate::joint_quantization::{self, ParameterQuantization};
use crate::{invalid, Result};

pub const PROBABILITY_TOTAL: u64 = TOTAL;
const WORK_BITS: i32 = 40;
const STATE_BITS: i32 = 11;

struct Parameter {
    codes: Vec<i16>,
    spec: ParameterQuantization,
}

pub struct IntegerModel {
    config: JointConfig,
    parameters: BTreeMap<String, Parameter>,
    tables: Tables,
    identity: String,
}

pub struct IntegerSession {
    identity: String,
    state: Vec<i32>,
    keys: Vec<Vec<i32>>,
    values: Vec<Vec<i32>>,
    tokens: Vec<u32>,
}

impl IntegerSession {
    pub fn len(&self) -> usize {
        self.tokens.len()
    }
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
}

pub struct IntegerStep {
    pub probabilities: Vec<u64>,
    pub state: Vec<i32>,
    pub no_read_mass: u64,
    pub read_masses: Vec<u64>,
    pub copy_gate: i32,
}

fn arithmetic<T>(value: MathResult<T>) -> Result<T> {
    value.map_err(|e| invalid(format!("integer model arithmetic: {e}")))
}

fn scaled(value: i128, shift: i32) -> Result<i128> {
    arithmetic(math::scale_pow2(value, shift))
}

fn product(a: i128, b: i128) -> Result<i128> {
    arithmetic(math::checked_mul(a, b))
}

fn divide(a: i128, b: i128) -> Result<i128> {
    arithmetic(math::div_round(a, b))
}

fn quantize(value: i128, input_bits: i32, output_bits: i32) -> Result<i32> {
    Ok(scaled(value, output_bits - input_bits)?.clamp(-32767, 32767) as i32)
}

/// Input codes are signed16 and coefficients signed4, validated at import.
/// This hot path is separate from index/shape arithmetic in the model driver.
#[inline(never)]
fn low_bit_dot(input: &[i32], weights: &[i16]) -> i64 {
    let mut total = 0i64;
    for (&x, &w) in input.iter().zip(weights) {
        let x = i64::from(x);
        let magnitude = match w.unsigned_abs() {
            0 => 0,
            1 => x,
            2 => x << 1,
            3 => x + (x << 1),
            4 => x << 2,
            5 => x + (x << 2),
            6 => (x << 1) + (x << 2),
            7 => (x << 3) - x,
            _ => unreachable!("validated signed4 coefficient"),
        };
        total += if w < 0 { -magnitude } else { magnitude };
    }
    total
}

impl IntegerModel {
    pub fn load_with_tables(directory: &Path, tables: &Path) -> Result<Self> {
        let manifest_path = directory.join("hard-model.json");
        if fs::metadata(&manifest_path)?.len() > 8 * 1024 * 1024 {
            return Err(invalid("integer model manifest too large"));
        }
        let manifest: Value = serde_json::from_slice(&fs::read(&manifest_path)?)?;
        let admission: AdmissionPolicy = manifest
            .get("admission")
            .map(|v| serde_json::from_value(v.clone()))
            .transpose()?
            .unwrap_or_default();
        if manifest["schema"] != "uor-r4.joint-recurrent-packed-emulator/1"
            || admission != AdmissionPolicy::Full
            || manifest["numerical_contract"] != crate::joint_model::quantized_numerical_contract()
            || manifest["parameter_manifest_sha256"]
                != crate::sha256_file(
                    &directory.join(joint_quantization::HARD_PARAMETERS_MANIFEST_FILE),
                )?
        {
            return Err(invalid(
                "integer import requires the retained full-access numerical contract",
            ));
        }
        let descriptor: Value = serde_json::from_slice(&fs::read(
            directory.join(joint_quantization::HARD_PARAMETERS_MANIFEST_FILE),
        )?)?;
        if descriptor != manifest["parameter_manifest"] {
            return Err(invalid("integer parameter manifest binding differs"));
        }
        let config: JointConfig = serde_json::from_value(manifest["model"].clone())?;
        config.validate()?;
        if config.context != 256 || config.width != 256 || config.read_width != 64 {
            return Err(invalid("integer bridge fixes context256/state256/read64"));
        }
        let state: QuantizedTrainingState =
            serde_json::from_value(manifest["quantization"].clone())?;
        let (spec, codes) = joint_quantization::load_hard_codes(directory)?;
        if state.spec != spec
            || state.ramp_steps == 0
            || state.completed_step < state.start_step
            || spec
                .parameters
                .iter()
                .map(|(k, v)| (k.clone(), v.shape.clone()))
                .collect::<BTreeMap<_, _>>()
                != config.shapes()
        {
            return Err(invalid("integer model shapes/scales/clock differ"));
        }
        let parameters = codes
            .into_iter()
            .map(|(name, codes)| {
                let parameter = Parameter {
                    codes,
                    spec: spec.parameters[&name].clone(),
                };
                (name, parameter)
            })
            .collect();
        let tables = Tables::load(tables)?;
        let identity = format!("{}:{}", crate::sha256_file(&manifest_path)?, tables.sha256);
        Ok(Self {
            config,
            parameters,
            tables,
            identity,
        })
    }

    pub fn config(&self) -> &JointConfig {
        &self.config
    }

    pub fn new_session(&self) -> IntegerSession {
        IntegerSession {
            identity: self.identity.clone(),
            state: vec![0; self.config.width],
            keys: Vec::with_capacity(self.config.context),
            values: Vec::with_capacity(self.config.context),
            tokens: Vec::with_capacity(self.config.context),
        }
    }

    fn parameter(&self, name: &str) -> Result<&Parameter> {
        self.parameters
            .get(name)
            .ok_or_else(|| invalid(format!("missing integer parameter {name}")))
    }

    fn vector_work(&self, name: &str) -> Result<Vec<i128>> {
        let p = self.parameter(name)?;
        p.codes
            .iter()
            .map(|&v| {
                scaled(
                    i128::from(v),
                    WORK_BITS + i32::from(p.spec.row_exponents[0]),
                )
            })
            .collect()
    }

    /// Return Q40 affine sums. The highest permitted exponent and shape fit
    /// i128. Scales finer than Q40 round once here (at most2^-41 absolute);
    /// the original Q8 output interface subsequently rounds and saturates.
    fn matrix_work(&self, input: &[i32], input_exponent: i32, name: &str) -> Result<Vec<i128>> {
        let p = self.parameter(name)?;
        if p.spec.bits != 4 || p.spec.shape.len() != 2 || p.spec.shape[1] != input.len() {
            return Err(invalid("integer affine input shape or bit width"));
        }
        p.codes
            .chunks_exact(input.len())
            .zip(&p.spec.row_exponents)
            .map(|(row, &exponent)| {
                scaled(
                    i128::from(low_bit_dot(input, row)),
                    WORK_BITS + input_exponent + i32::from(exponent),
                )
            })
            .collect()
    }

    fn affine(&self, input: &[i32], input_bits: i32, prefix: &str) -> Result<Vec<i32>> {
        let values = self.matrix_work(input, -input_bits, &format!("{prefix}.weight"))?;
        let bias = self.vector_work(&format!("{prefix}.bias"))?;
        values
            .into_iter()
            .zip(bias)
            .map(|(v, b)| quantize(v + b, WORK_BITS, 8))
            .collect()
    }

    fn tanh(&self, input: &[i32]) -> Vec<i32> {
        input
            .iter()
            .map(|&v| self.tables.tanh[(v + 32767) as usize])
            .collect()
    }

    fn sigmoid(&self, input: &[i32]) -> Vec<i32> {
        input
            .iter()
            .map(|&v| self.tables.sigmoid[(v + 32767) as usize])
            .collect()
    }

    pub fn step(
        &self,
        session: &mut IntegerSession,
        token: u32,
        mode: ReadMode,
    ) -> Result<IntegerStep> {
        if session.identity != self.identity
            || token as usize >= self.config.vocab_size
            || session.len() >= self.config.context
        {
            return Err(invalid("integer session artifact/context/token mismatch"));
        }
        let width = self.config.width;
        let embedding = self.parameter("embedding.weight")?;
        let token_index = token as usize;
        let embedding_codes: Vec<i32> = embedding.codes
            [token_index * width..(token_index + 1) * width]
            .iter()
            .map(|&x| i32::from(x))
            .collect();
        let token_affine = self.matrix_work(
            &embedding_codes,
            i32::from(embedding.spec.row_exponents[token_index]),
            "recurrent.input.weight",
        )?;
        let previous_normalized = normalize_state(&session.state)?;
        let recurrent = self.matrix_work(&previous_normalized, -10, "recurrent.state.weight")?;
        let bias = self.vector_work("recurrent.bias")?;
        let fused = token_affine
            .into_iter()
            .zip(recurrent)
            .zip(bias)
            .map(|((a, b), c)| quantize(a + b + c, WORK_BITS, 8))
            .collect::<Result<Vec<_>>>()?;
        let candidate = self.tanh(&fused[..width]);
        let z = self.sigmoid(&fused[width..width + width]);
        let transported = transport(
            &session.state,
            &fused[width + width..],
            self.config.transport,
        )?;
        let provisional = blend(&transported, &candidate, &z)?;
        let normalized = normalize_state(&provisional)?;
        let previous = session.len();
        let (no_read, read_masses, read) = if previous == 0 || mode == ReadMode::NoRead {
            (TOTAL, vec![0; previous], vec![0; width])
        } else {
            let query = self.affine(&normalized, 10, "read.query")?;
            let null = self.affine(&normalized, 10, "read.no_read")?[0];
            let age = self.vector_work("read.age")?;
            let mut scores = Vec::with_capacity(previous + 1);
            scores.push(null);
            for (index, key) in session.keys.iter().enumerate() {
                let mut dot = 0i128;
                for (&q, &k) in query.iter().zip(key) {
                    dot += product(i128::from(q), i128::from(k))?;
                }
                // Q8 dot Q8, divided by sqrt(read_width64)=8: exponent -19.
                let score = scaled(dot, WORK_BITS - 19)? + age[previous - 1 - index];
                scores.push(quantize(score, WORK_BITS, 8)?);
            }
            let masses = softmax(&scores, &self.tables)?;
            let mut read = vec![0i32; width];
            for (coordinate, out) in read.iter_mut().enumerate() {
                let mut sum = 0i128;
                for (mass, value) in masses[1..].iter().zip(&session.values) {
                    sum += product(i128::from(*mass), i128::from(value[coordinate]))?;
                }
                *out = quantize(sum, 48 + 14, STATE_BITS)?;
            }
            (masses[0], masses[1..].to_vec(), read)
        };
        let mut update_input = provisional.clone();
        update_input.extend_from_slice(&read);
        let update = self.tanh(&self.affine(&update_input, STATE_BITS, "update")?);
        let rho = self.sigmoid(&self.affine(&update_input, STATE_BITS, "update.gate")?)[0];
        let state = blend(&provisional, &update, &vec![rho; width])?;
        let mut copy_input = state.clone();
        copy_input.extend_from_slice(&read);
        let gate = self.sigmoid(&self.affine(&copy_input, STATE_BITS, "copy.gate")?)[0];
        let write_normalized = normalize_state(&state)?;
        let key = self.affine(&write_normalized, 10, "read.key")?;
        let value = self.tanh(&self.affine(&write_normalized, 10, "read.value")?);
        let norm = self.parameter("output.norm.weight")?;
        let hidden = write_normalized
            .iter()
            .zip(&norm.codes)
            .map(|(&x, &w)| {
                let value = product(i128::from(x), i128::from(w))?;
                Ok(
                    scaled(value, i32::from(norm.spec.row_exponents[0]))?.clamp(-32767, 32767)
                        as i32,
                )
            })
            .collect::<Result<Vec<_>>>()?;
        let logits = self.matrix_work(&hidden, -10, "embedding.weight")?;
        let output_bias = self.vector_work("output.bias")?;
        let logits = logits
            .into_iter()
            .zip(output_bias)
            .map(|(v, b)| quantize(v + b, WORK_BITS, 8))
            .collect::<Result<Vec<_>>>()?;
        let vocabulary = softmax(&logits, &self.tables)?;
        let mut copy = vec![0u64; self.config.vocab_size];
        for (&id, &mass) in session.tokens.iter().zip(&read_masses) {
            copy[id as usize] += mass;
        }
        let fraction = i128::from(TOTAL)
            - scaled(product(i128::from(gate), i128::from(TOTAL - no_read))?, -15)?;
        let mut probabilities = Vec::with_capacity(self.config.vocab_size);
        for (v, c) in vocabulary.into_iter().zip(copy) {
            let raw = scaled(product(i128::from(v), fraction)?, -48)?
                + scaled(product(i128::from(c), i128::from(gate))?, -15)?;
            // The retained uniform mixture is exactly 1/100000000 in this bridge.
            let uniform = divide(
                product(raw, 99_999_999)? + i128::from(TOTAL >> 12),
                100_000_000,
            )?;
            probabilities
                .push(u64::try_from(uniform).map_err(|_| invalid("integer mixture range"))?);
        }
        normalize_residual(&mut probabilities)?;
        // Commit after the complete prediction succeeds. Current input is
        // written here and cannot be a source for this step's contextual copy.
        session.state = state.clone();
        session.keys.push(key);
        session.values.push(value);
        session.tokens.push(token);
        Ok(IntegerStep {
            probabilities,
            state,
            no_read_mass: no_read,
            read_masses,
            copy_gate: gate,
        })
    }
}

/// State Q11 -> RMS-normalized Q10. Variance uses Q64 guard precision in code
/// units: sum(x_code^2)/256 + 2^22/100000. The square root has 32 guard bits.
#[inline(never)]
fn normalize_state(input: &[i32]) -> Result<Vec<i32>> {
    if input.len() != 256 {
        return Err(invalid("integer normalization width must be256"));
    }
    let mut sum = 0u128;
    for &x in input {
        sum += product(i128::from(x), i128::from(x))? as u128;
    }
    let epsilon = divide(1i128 << 86, 100_000)? as u128;
    let variance = (sum << 56) + epsilon;
    let denominator = math::isqrt(variance) as i128;
    input
        .iter()
        .map(|&x| Ok(divide(i128::from(x) << 42, denominator)?.clamp(-32767, 32767) as i32))
        .collect()
}

#[inline(never)]
fn blend(state: &[i32], candidate: &[i32], gates: &[i32]) -> Result<Vec<i32>> {
    state
        .iter()
        .zip(candidate)
        .zip(gates)
        .map(|((&x, &y), &g)| {
            let a = product(i128::from(32768 - g), i128::from(x) << 3)?;
            let b = product(i128::from(g), i128::from(y))?;
            quantize(a + b, 29, STATE_BITS)
        })
        .collect()
}

#[inline(never)]
fn transport(input: &[i32], raw: &[i32], kind: Transport) -> Result<Vec<i32>> {
    let mut output = Vec::with_capacity(input.len());
    for (x, r) in input.chunks_exact(4).zip(raw.chunks_exact(4)) {
        // Common center denominator cancels during normalization. Q32 keeps
        // sqrt2 for the matched ordinary arm without a served transcendental.
        let mut centered = [0i128; 4];
        for j in 0..4 {
            centered[j] = i128::from(r[j]) << 32;
        }
        centered[0] += match kind {
            Transport::Quaternion => 2560i128 << 32,
            // nearest Q32 representation of 2560*sqrt(2), offline constant
            Transport::HouseholderPair => 15_549_442_559_877,
        };
        let mut sum = 0u128;
        for &v in &centered {
            sum += product(v, v)? as u128;
        }
        let denominator = math::isqrt(sum) as i128;
        let mut unit = [0i128; 4];
        // Minimum nonzero quantized center is well above original1e-6,
        // except a possible cancellation in the real-coordinate component.
        let threshold = match kind {
            Transport::Quaternion => 10_995_116i128,
            Transport::HouseholderPair => 15_549_443i128,
        };
        if denominator <= threshold {
            unit[0] = 16384;
        } else {
            for j in 0..4 {
                unit[j] = divide(centered[j] << 14, denominator)?.clamp(-32767, 32767);
            }
        }
        let x: [i128; 4] = [
            i128::from(x[0]),
            i128::from(x[1]),
            i128::from(x[2]),
            i128::from(x[3]),
        ];
        match kind {
            Transport::Quaternion => {
                let [w, a, b, c] = unit;
                let y = [
                    product(w, x[0])? - product(a, x[1])? - product(b, x[2])? - product(c, x[3])?,
                    product(w, x[1])? + product(a, x[0])? + product(b, x[3])? - product(c, x[2])?,
                    product(w, x[2])? - product(a, x[3])? + product(b, x[0])? + product(c, x[1])?,
                    product(w, x[3])? + product(a, x[2])? - product(b, x[1])? + product(c, x[0])?,
                ];
                for v in y {
                    output.push(quantize(v, 25, STATE_BITS)?);
                }
            }
            Transport::HouseholderPair => {
                let reflected = [-x[0], x[1], x[2], x[3]];
                let mut dot = 0i128;
                for j in 0..4 {
                    dot += product(unit[j], reflected[j])?;
                }
                for j in 0..4 {
                    let value = (reflected[j] << 28) - (product(unit[j], dot)? << 1);
                    output.push(quantize(value, 39, STATE_BITS)?);
                }
            }
        }
    }
    Ok(output)
}

#[inline(never)]
fn softmax(scores: &[i32], tables: &Tables) -> Result<Vec<u64>> {
    let max = *scores
        .iter()
        .max()
        .ok_or_else(|| invalid("empty integer softmax"))?;
    let weights: Vec<u64> = scores
        .iter()
        .map(|&s| tables.exp[(max - s) as usize])
        .collect();
    let sum: u64 = weights.iter().sum();
    let mut masses = weights
        .into_iter()
        .map(|w| {
            u64::try_from(divide(i128::from(w) << 48, i128::from(sum))?)
                .map_err(|_| invalid("integer softmax mass range"))
        })
        .collect::<Result<Vec<_>>>()?;
    normalize_residual(&mut masses)?;
    Ok(masses)
}

/// Correct at most half a unit per rounded entry on the largest probability.
/// This supplies an exactly normalized finite integer distribution.
fn normalize_residual(values: &mut [u64]) -> Result<()> {
    let mut total = 0u64;
    let mut largest = 0usize;
    for (i, &v) in values.iter().enumerate() {
        total = total
            .checked_add(v)
            .ok_or_else(|| invalid("probability total overflow"))?;
        if v > values[largest] {
            largest = i;
        }
    }
    if values.is_empty() {
        return Err(invalid("empty probability distribution"));
    }
    if total <= TOTAL {
        values[largest] += TOTAL - total;
    } else {
        values[largest] = values[largest]
            .checked_sub(total - TOTAL)
            .ok_or_else(|| invalid("probability residual exceeds maximum"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed4_affine_accumulation() {
        let input = [-32767, -3, 0, 1, 16384, 32767];
        for code in -7i16..=7 {
            let weights = vec![code; input.len()];
            let expected: i64 = input.iter().map(|&x| i64::from(x) * i64::from(code)).sum();
            assert_eq!(low_bit_dot(&input, &weights), expected);
        }
    }

    #[test]
    fn normalization_zero_scale_and_transport_identity() -> Result<()> {
        assert_eq!(normalize_state(&vec![0; 256])?, vec![0; 256]);
        let normalized = normalize_state(&vec![2048; 256])?;
        assert!(normalized.iter().all(|&x| x == 1024));
        let state = [100, -400, 800, -1200];
        for kind in [Transport::Quaternion, Transport::HouseholderPair] {
            assert_eq!(transport(&state, &[0; 4], kind)?, state);
        }
        // Exact-denominator centering must preserve the degenerate raw=-10
        // quaternion branch; q and -q are not interchangeable in this model.
        assert_eq!(
            transport(&state, &[-2560, 0, 0, 0], Transport::Quaternion)?,
            state
        );
        assert_eq!(blend(&state, &[0; 4], &[0; 4])?, state);
        assert_eq!(blend(&state, &[16384; 4], &[32768; 4])?, vec![2048; 4]);
        Ok(())
    }
}
