//! Integer execution bridge for the retained full-context recurrent model.
//!
//! Parameters retain their signed codes and dyadic scales. Numerical execution
//! uses add/subtract, shifts, comparisons, integer table reads, binary products,
//! division and square roots implemented in `crate::math`. No floating
//! model operation or hardware product is requested by these numerical kernels.
//! This remains a dense-parameter, allocating, finite-context prototype. Loading
//! validates legacy metadata; offline table construction and host evaluation
//! are separate from this numerical path. Approximate integer execution does
//! not imply bitwise equivalence to the original F32 accumulation order.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::config::{JointConfig, QuantizedTrainingState, ReadMode, Transport};
use crate::format::{self as joint_quantization, ParameterQuantization};
use crate::math::{self, MathResult};
use crate::tables::{Tables, TOTAL};
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
    keys: Vec<[i32; KEY_DIM]>,
    values: Vec<[i32; VAL_DIM]>,
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

pub const KEY_DIM: usize = 64;
pub const VAL_DIM: usize = 256;
pub const PERSISTENT_CAPACITY: usize = 32;
pub const DIALOGUE_CAPACITY: usize = 224;
pub const TOTAL_MEMORY_CAPACITY: usize = 256;
pub const AGE_HORIZON_CLAMP: usize = 63;

pub use crate::math::{
    atan2_q30, HopfFiberPointQ30, T8ZetaState, UnitS3Q30, CORDIC_ATAN_TABLE_Q30,
    ZETA_FREQUENCIES_Q30,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlotTarget {
    Persistent,
    Dialogue,
}

/// Partitioned conversational memory state for zero-matmul serving.
///
/// Power-of-two aligned memory structures (key stride 256 B = 2^8, value stride 1024 B = 2^10)
/// eliminate all hardware address multiplier (`madd`) instructions during serving.
#[derive(Clone, Debug)]
pub struct SessionState {
    pub identity: String,
    /// Recurrent state vector in Q11 format (dimension = 256).
    pub state: Vec<i32>,

    // --- Persistent Session Partition (Slots 0..32) ---
    pub persistent_keys: Vec<[i32; KEY_DIM]>,
    pub persistent_values: Vec<[i32; VAL_DIM]>,
    pub persistent_tokens: Vec<u32>,
    pub persistent_capacity: usize,
    pub persistent_sealed: bool,

    // --- Rolling Dialogue Partition (Slots 32..256 = 224 slots) ---
    pub dialogue_keys: Box<[[i32; KEY_DIM]; DIALOGUE_CAPACITY]>,
    pub dialogue_values: Box<[[i32; VAL_DIM]; DIALOGUE_CAPACITY]>,
    pub dialogue_tokens: Vec<u32>,
    pub dialogue_sequences: Vec<u64>,
    pub dialogue_turn_ids: Vec<u32>,
    pub dialogue_capacity: usize,
    pub dialogue_cursor: usize,
    pub dialogue_len: usize,
    pub dialogue_seen: u64,
    pub current_turn_id: u32,

    // --- Geometric & Phase State ---
    pub zeta_state: T8ZetaState,
    pub hopf_state: HopfFiberPointQ30,
    pub cumulative_holonomy_q30: i64,

    pub age_horizon_clamp: usize,
}

impl SessionState {
    pub fn persistent_len(&self) -> usize {
        self.persistent_tokens.len()
    }

    pub fn dialogue_len(&self) -> usize {
        self.dialogue_len
    }

    pub fn total_len(&self) -> usize {
        self.persistent_len() + self.dialogue_len()
    }

    pub fn is_empty(&self) -> bool {
        self.total_len() == 0
    }

    pub fn seal_persistent(&mut self) {
        self.persistent_sealed = true;
    }

    pub fn is_persistent_sealed(&self) -> bool {
        self.persistent_sealed
    }

    pub fn start_turn(&mut self) {
        self.current_turn_id += 1;
    }

    pub fn current_turn(&self) -> u32 {
        self.current_turn_id
    }

    pub fn zeta_phases(&self) -> [i32; 8] {
        self.zeta_state.phases
    }

    pub fn holonomy_accumulator(&self) -> i64 {
        self.cumulative_holonomy_q30
    }
}

#[derive(Clone, Debug)]
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

/// Build each input coordinate's signed4 multiples once for reuse across rows.
/// Slots use the coefficient's low nibble: 0..7 and -7..-1; reserved -8 is
/// unreachable after import validation and its slot is zero. i64 intermediates
/// keep these shifts/additions exact even for the full i32 input range.
#[inline(never)]
fn low_bit_products(input: &[i32]) -> Vec<[i64; 16]> {
    input
        .iter()
        .map(|&x| {
            let x = i64::from(x);
            let twice = x << 1;
            let four = x << 2;
            let three = x + twice;
            let five = x + four;
            let six = twice + four;
            let seven = (x << 3) - x;
            [
                0, x, twice, three, four, five, six, seven, 0, -seven, -six, -five, -four, -three,
                -twice, -x,
            ]
        })
        .collect()
}

/// Coefficients are signed4, validated at import. The retained shape bounds
/// (at most512 input coordinates) keep even full-i32 products/sums inside i64.
/// Coordinate order is unchanged; replacing recomputation with table reads adds
/// no rounding. This hot path is separate from model-driver shape arithmetic.
#[inline(never)]
fn low_bit_dot(products: &[[i64; 16]], weights: &[i16]) -> i64 {
    let mut total = 0i64;
    for (multiples, &weight) in products.iter().zip(weights) {
        total += multiples[usize::from((weight as u16) & 15)];
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
        if manifest["schema"] != "uor-r4.joint-recurrent-packed-emulator/1"
            || manifest
                .get("admission")
                .is_some_and(|value| value != "full")
            || manifest["numerical_contract"] != crate::config::quantized_numerical_contract()?
            || manifest["parameter_manifest_sha256"]
                != crate::sha256_file(
                    &directory.join(joint_quantization::HARD_PARAMETERS_MANIFEST_FILE),
                )?
        {
            return Err(invalid(
                "integer import requires the retained full-access numerical contract",
            ));
        }
        if fs::metadata(directory.join(joint_quantization::HARD_PARAMETERS_MANIFEST_FILE))?.len()
            > 4 * 1024 * 1024
        {
            return Err(invalid("integer parameter descriptor too large"));
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
            || !crate::config::valid_quantization_clock(
                state.start_step,
                state.ramp_steps,
                state.completed_step,
                state.preparation,
            )
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

    /// Create a new partitioned conversational memory session.
    pub fn new_conversational_session(&self) -> SessionState {
        let dialogue_keys = vec![[0i32; KEY_DIM]; DIALOGUE_CAPACITY]
            .into_boxed_slice()
            .try_into()
            .unwrap_or_else(|_| panic!("dialogue_keys size mismatch"));
        let dialogue_values = vec![[0i32; VAL_DIM]; DIALOGUE_CAPACITY]
            .into_boxed_slice()
            .try_into()
            .unwrap_or_else(|_| panic!("dialogue_values size mismatch"));
        SessionState {
            identity: self.identity.clone(),
            state: vec![0; self.config.width],
            persistent_keys: Vec::with_capacity(PERSISTENT_CAPACITY),
            persistent_values: Vec::with_capacity(PERSISTENT_CAPACITY),
            persistent_tokens: Vec::with_capacity(PERSISTENT_CAPACITY),
            persistent_capacity: PERSISTENT_CAPACITY,
            persistent_sealed: false,
            dialogue_keys,
            dialogue_values,
            dialogue_tokens: vec![0; DIALOGUE_CAPACITY],
            dialogue_sequences: vec![0; DIALOGUE_CAPACITY],
            dialogue_turn_ids: vec![0; DIALOGUE_CAPACITY],
            dialogue_capacity: DIALOGUE_CAPACITY,
            dialogue_cursor: 0,
            dialogue_len: 0,
            dialogue_seen: 0,
            current_turn_id: 0,
            zeta_state: T8ZetaState::new(),
            hopf_state: HopfFiberPointQ30::default(),
            cumulative_holonomy_q30: 0,
            age_horizon_clamp: AGE_HORIZON_CLAMP,
        }
    }

    /// Look up signed 4-bit token embedding codes from embedding.weight.
    /// Row stride uses exact power-of-two shift (`<< 8` for width 256).
    /// Strictly 0 hardware multipliers.
    #[inline(never)]
    pub fn embed(&self, token: u32) -> Result<Vec<i32>> {
        let token_index = token as usize;
        if token_index >= self.config.vocab_size {
            return Err(invalid("token out of vocabulary bounds"));
        }
        let width = self.config.width;
        let embedding = self.parameter("embedding.weight")?;
        let start = match width {
            256 => token_index << 8,
            128 => token_index << 7,
            _ => return Err(invalid("unsupported embedding width")),
        };
        let end = start + width;
        Ok(embedding.codes[start..end]
            .iter()
            .map(|&x| i32::from(x))
            .collect())
    }

    /// Project normalized hidden state to vocabulary logits using pure signed-4
    /// shift-and-add dot products over embedding.weight.
    /// Row stride uses exact power-of-two shift (`<< 8` for width 256).
    /// Strictly 0 hardware multipliers, 0 floating point.
    #[inline(never)]
    pub fn project_vocab(&self, hidden: &[i32]) -> Result<Vec<i32>> {
        if hidden.len() != self.config.width {
            return Err(invalid("project_vocab input width mismatch"));
        }
        let embedding = self.parameter("embedding.weight")?;
        let output_bias = self.vector_work("output.bias")?;
        let products = low_bit_products(hidden);

        let width = self.config.width;
        let mut logits = Vec::with_capacity(self.config.vocab_size);
        if width == 256 {
            for (row_idx, &bias) in output_bias.iter().enumerate().take(self.config.vocab_size) {
                let start = row_idx << 8;
                let end = start + 256;
                let row = &embedding.codes[start..end];
                let dot = low_bit_dot(&products, row);
                let scaled_val = scaled(
                    i128::from(dot),
                    WORK_BITS - 10 + i32::from(embedding.spec.row_exponents[row_idx]),
                )?;
                logits.push(quantize(scaled_val + bias, WORK_BITS, 8)?);
            }
        } else if width == 128 {
            for (row_idx, &bias) in output_bias.iter().enumerate().take(self.config.vocab_size) {
                let start = row_idx << 7;
                let end = start + 128;
                let row = &embedding.codes[start..end];
                let dot = low_bit_dot(&products, row);
                let scaled_val = scaled(
                    i128::from(dot),
                    WORK_BITS - 10 + i32::from(embedding.spec.row_exponents[row_idx]),
                )?;
                logits.push(quantize(scaled_val + bias, WORK_BITS, 8)?);
            }
        } else {
            return Err(invalid("unsupported width for project_vocab"));
        }
        Ok(logits)
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
        let products = low_bit_products(input);
        match input.len() {
            256 => p
                .codes
                .chunks_exact(256)
                .zip(&p.spec.row_exponents)
                .map(|(row, &exponent)| {
                    scaled(
                        i128::from(low_bit_dot(&products, row)),
                        WORK_BITS + input_exponent + i32::from(exponent),
                    )
                })
                .collect(),
            512 => p
                .codes
                .chunks_exact(512)
                .zip(&p.spec.row_exponents)
                .map(|(row, &exponent)| {
                    scaled(
                        i128::from(low_bit_dot(&products, row)),
                        WORK_BITS + input_exponent + i32::from(exponent),
                    )
                })
                .collect(),
            128 => p
                .codes
                .chunks_exact(128)
                .zip(&p.spec.row_exponents)
                .map(|(row, &exponent)| {
                    scaled(
                        i128::from(low_bit_dot(&products, row)),
                        WORK_BITS + input_exponent + i32::from(exponent),
                    )
                })
                .collect(),
            other => Err(invalid(format!("unsupported matrix input width {other}"))),
        }
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
        let embedding_codes = self.embed(token)?;
        let embedding = self.parameter("embedding.weight")?;
        let token_affine = self.matrix_work(
            &embedding_codes,
            i32::from(embedding.spec.row_exponents[token as usize]),
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
            let mut sum_coords = [0i128; VAL_DIM];
            for (mass, value) in masses[1..].iter().zip(&session.values) {
                let m = i128::from(*mass);
                for coord in 0..width {
                    sum_coords[coord] += product(m, i128::from(value[coord]))?;
                }
            }
            let mut read = vec![0i32; width];
            for (coordinate, out) in read.iter_mut().enumerate() {
                *out = quantize(sum_coords[coordinate], 48 + 14, STATE_BITS)?;
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
        let logits = self.project_vocab(&hidden)?;
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
        let mut key_arr = [0i32; KEY_DIM];
        let key_len = key.len().min(KEY_DIM);
        key_arr[..key_len].copy_from_slice(&key[..key_len]);
        session.keys.push(key_arr);
        let mut val_arr = [0i32; VAL_DIM];
        let val_len = value.len().min(VAL_DIM);
        val_arr[..val_len].copy_from_slice(&value[..val_len]);
        session.values.push(val_arr);
        session.tokens.push(token);
        Ok(IntegerStep {
            probabilities,
            state,
            no_read_mass: no_read,
            read_masses,
            copy_gate: gate,
        })
    }

    /// Step conversational session with partitioned memory, zero age decay for persona,
    /// cyclic FIFO dialogue ring buffer, T8 zeta phase coordinates, and Hopf fiber holonomy tracking.
    pub fn step_conversational(
        &self,
        session: &mut SessionState,
        token: u32,
        target: SlotTarget,
        mode: ReadMode,
    ) -> Result<IntegerStep> {
        if session.identity != self.identity {
            return Err(invalid("conversational step identity mismatch"));
        }
        if token as usize >= self.config.vocab_size {
            return Err(invalid("conversational step token out of bounds"));
        }
        if target == SlotTarget::Persistent {
            if session.persistent_sealed {
                return Err(invalid(
                    "cannot write to sealed persistent session partition",
                ));
            }
            if session.persistent_keys.len() >= session.persistent_capacity {
                return Err(invalid(
                    "persistent session partition capacity (32) exceeded",
                ));
            }
        }

        let width = self.config.width;
        let embedding_codes = self.embed(token)?;
        let embedding = self.parameter("embedding.weight")?;
        let token_affine = self.matrix_work(
            &embedding_codes,
            i32::from(embedding.spec.row_exponents[token as usize]),
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

        let n_sys = session.persistent_keys.len();
        let n_dial = session.dialogue_len;
        let total_slots = n_sys + n_dial;

        let (no_read, read_masses, read) = if total_slots == 0 || mode == ReadMode::NoRead {
            (TOTAL, vec![0; total_slots], vec![0; width])
        } else {
            let query = self.affine(&normalized, 10, "read.query")?;
            let null = self.affine(&normalized, 10, "read.no_read")?[0];
            let age = self.vector_work("read.age")?;
            let mut scores = Vec::with_capacity(1 + total_slots);
            scores.push(null);

            // A. Persistent session slots: zero age decay (fixed to age[0])
            for key in &session.persistent_keys {
                let mut dot = 0i128;
                for (&q, &k) in query.iter().zip(key) {
                    dot += product(i128::from(q), i128::from(k))?;
                }
                let score = scaled(dot, WORK_BITS - 19)? + age[0];
                scores.push(quantize(score, WORK_BITS, 8)?);
            }

            // B. Dialogue slots: relative age with horizon clamp
            for (key, &seq) in session.dialogue_keys[..n_dial]
                .iter()
                .zip(&session.dialogue_sequences[..n_dial])
            {
                let mut dot = 0i128;
                for (&q, &k) in query.iter().zip(key) {
                    dot += product(i128::from(q), i128::from(k))?;
                }
                let delta_t = session.dialogue_seen.saturating_sub(1 + seq);
                let age_idx = (delta_t as usize)
                    .min(age.len().saturating_sub(1))
                    .min(session.age_horizon_clamp);
                let score = scaled(dot, WORK_BITS - 19)? + age[age_idx];
                scores.push(quantize(score, WORK_BITS, 8)?);
            }

            let masses = softmax(&scores, &self.tables)?;
            let m_sys = &masses[1..1 + n_sys];
            let m_dial = &masses[1 + n_sys..];

            let mut sum_coords = [0i128; VAL_DIM];
            for (&mass, val) in m_sys.iter().zip(&session.persistent_values) {
                let m = i128::from(mass);
                for (acc, &v) in sum_coords.iter_mut().zip(val) {
                    *acc += product(m, i128::from(v))?;
                }
            }
            for (&mass, val) in m_dial.iter().zip(&session.dialogue_values[..n_dial]) {
                let m = i128::from(mass);
                for (acc, &v) in sum_coords.iter_mut().zip(val) {
                    *acc += product(m, i128::from(v))?;
                }
            }

            let mut read = vec![0i32; width];
            for (out, &acc) in read.iter_mut().zip(&sum_coords) {
                *out = quantize(acc, 48 + 14, STATE_BITS)?;
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
                let val = product(i128::from(x), i128::from(w))?;
                Ok(scaled(val, i32::from(norm.spec.row_exponents[0]))?.clamp(-32767, 32767) as i32)
            })
            .collect::<Result<Vec<_>>>()?;
        let logits = self.project_vocab(&hidden)?;
        let vocabulary = softmax(&logits, &self.tables)?;

        let mut copy = vec![0u64; self.config.vocab_size];
        let m_sys = &read_masses[..n_sys];
        let m_dial = &read_masses[n_sys..];
        for (&id, &mass) in session.persistent_tokens.iter().zip(m_sys) {
            if (id as usize) < self.config.vocab_size {
                copy[id as usize] += mass;
            }
        }
        for (&id, &mass) in session.dialogue_tokens[..n_dial].iter().zip(m_dial) {
            if (id as usize) < self.config.vocab_size {
                copy[id as usize] += mass;
            }
        }

        let fraction = i128::from(TOTAL)
            - scaled(product(i128::from(gate), i128::from(TOTAL - no_read))?, -15)?;
        let mut probabilities = Vec::with_capacity(self.config.vocab_size);
        for (v, c) in vocabulary.into_iter().zip(copy) {
            let raw = scaled(product(i128::from(v), fraction)?, -48)?
                + scaled(product(i128::from(c), i128::from(gate))?, -15)?;
            let uniform = divide(
                product(raw, 99_999_999)? + i128::from(TOTAL >> 12),
                100_000_000,
            )?;
            probabilities
                .push(u64::try_from(uniform).map_err(|_| invalid("integer mixture range"))?);
        }
        normalize_residual(&mut probabilities)?;

        session.state = state.clone();
        match target {
            SlotTarget::Persistent => {
                let mut key_arr = [0i32; KEY_DIM];
                key_arr.copy_from_slice(&key[..KEY_DIM]);
                let mut val_arr = [0i32; VAL_DIM];
                val_arr.copy_from_slice(&value[..VAL_DIM]);

                session.persistent_keys.push(key_arr);
                session.persistent_values.push(val_arr);
                session.persistent_tokens.push(token);
            }
            SlotTarget::Dialogue => {
                let idx = session.dialogue_cursor;
                session.dialogue_keys[idx].copy_from_slice(&key[..KEY_DIM]);
                session.dialogue_values[idx].copy_from_slice(&value[..VAL_DIM]);
                session.dialogue_tokens[idx] = token;
                session.dialogue_sequences[idx] = session.dialogue_seen;
                session.dialogue_turn_ids[idx] = session.current_turn_id;
                let next_cursor = session.dialogue_cursor + 1;
                session.dialogue_cursor = if next_cursor >= session.dialogue_capacity {
                    0
                } else {
                    next_cursor
                };
                session.dialogue_len = (session.dialogue_len + 1).min(session.dialogue_capacity);
                session.dialogue_seen += 1;
            }
        }

        session.zeta_state.step(token);

        let s3_coords = [state[0], state[1], state[2], state[3]];
        let next_s3 = math::UnitS3Q30::from_i32_coords(&s3_coords);
        let next_pt = next_s3.hopf_fiber_project();
        let mut d_phase = (next_pt.fiber_phase as i64) - (session.hopf_state.fiber_phase as i64);
        if d_phase > (1i64 << 30) {
            d_phase -= 2i64 << 30;
        } else if d_phase < -(1i64 << 30) {
            d_phase += 2i64 << 30;
        }
        if d_phase == 0 {
            d_phase = (math::ZETA_FREQUENCIES_Q30[0] as i64) >> 10;
        }
        session.cumulative_holonomy_q30 += d_phase;
        session.hopf_state = next_pt;

        Ok(IntegerStep {
            probabilities,
            state,
            no_read_mass: no_read,
            read_masses,
            copy_gate: gate,
        })
    }

    pub fn synthetic_for_test() -> Self {
        let config = JointConfig::default();
        let mut parameters = BTreeMap::new();
        let shapes = config.shapes();
        for (name, shape) in shapes {
            let total_elements: usize = shape.iter().product();
            let bits = if shape.len() == 2 { 4 } else { 16 };
            let codes = vec![1i16; total_elements];
            let row_count = if shape.len() == 2 { shape[0] } else { 1 };
            let spec = ParameterQuantization {
                bits,
                shape: shape.clone(),
                row_exponents: vec![0i16; row_count],
            };
            parameters.insert(name, Parameter { codes, spec });
        }
        let tables = Tables::synthetic_for_test();
        Self {
            config,
            parameters,
            tables,
            identity: "synthetic_model".to_owned(),
        }
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
    let mut sum: u64 = 0;
    for &w in &weights {
        sum = sum.wrapping_add(core::hint::black_box(w));
    }
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
        let input = [i32::MIN, -32767, -3, 0, 1, 16384, 32767, i32::MAX];
        let products = low_bit_products(&input);
        for code in -7i16..=7 {
            for (&x, multiples) in input.iter().zip(&products) {
                assert_eq!(
                    multiples[usize::from((code as u16) & 15)],
                    i64::from(x) * i64::from(code)
                );
            }
            let weights = vec![code; input.len()];
            let expected: i64 = input.iter().map(|&x| i64::from(x) * i64::from(code)).sum();
            assert_eq!(low_bit_dot(&products, &weights), expected);
        }
        for x in [i32::MIN, i32::MAX] {
            let products = low_bit_products(&[x; 512]);
            for code in [-7, 7] {
                assert_eq!(
                    low_bit_dot(&products, &[code; 512]),
                    i64::from(x) * i64::from(code) * 512
                );
            }
        }
        let mixed_codes: Vec<i16> = (-7..=7).cycle().take(512).collect();
        let mixed_input: Vec<i32> = input.into_iter().cycle().take(512).collect();
        let expected: i64 = mixed_input
            .iter()
            .zip(&mixed_codes)
            .map(|(&x, &code)| i64::from(x) * i64::from(code))
            .sum();
        assert_eq!(
            low_bit_dot(&low_bit_products(&mixed_input), &mixed_codes),
            expected
        );
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
