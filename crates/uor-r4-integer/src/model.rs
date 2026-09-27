//! Integer execution bridge for the retained full-context recurrent model.
//!
//! Parameters retain their signed codes and dyadic scales. Numerical execution
//! uses add/subtract, shifts, comparisons, integer table reads, binary products,
//! division and square roots implemented in `crate::math`. No floating
//! model operation or hardware product is requested by these numerical kernels.
//! The read scores keys by the retained dot product or, for a Lorentz model, by
//! the hyperbolic distance kernel in `crate::lorentz` with a sealed arcosh table.
//! This remains a dense-parameter, allocating, finite-context prototype. Loading
//! validates legacy metadata; offline table construction and host evaluation
//! are separate from this numerical path. Approximate integer execution does
//! not imply bitwise equivalence to the original F32 accumulation order.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::config::{
    JointConfig, QuantizedTrainingState, ReadGeometry, ReadMode, ServingProfile, Transport,
    LORENTZ_LOG_BETA, LORENTZ_OFFSET,
};
use crate::format::{self as joint_quantization, ParameterQuantization};
use crate::lorentz::{self, LorentzRead};
use crate::math::{self, MathResult};
use crate::packed_rows::{
    low_bit_dot, low_bit_dot_4_contiguous_512, low_bit_dot_8_contiguous, CoefficientCodes,
};
use crate::tables::{Tables, TOTAL};
use crate::{invalid, Result};

pub const PROBABILITY_TOTAL: u64 = TOTAL;
const WORK_BITS: i32 = 40;
const STATE_BITS: i32 = 11;

#[derive(Clone)]
struct Parameter {
    codes: CoefficientCodes,
    spec: ParameterQuantization,
}

/// Unique shared coefficient payload retained by a loaded model. This excludes
/// scales, caches, Arc/allocator metadata, loading temporaries and session state;
/// it is not a process RSS or physical memory-traffic measurement.
#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Serialize)]
pub struct CoefficientStorage {
    pub signed4_coefficients: usize,
    pub packed_signed4_bytes: usize,
    pub signed16_coefficients: usize,
    pub signed16_bytes: usize,
}

pub struct IntegerModel {
    config: JointConfig,
    serving_profile: ServingProfile,
    parameters: BTreeMap<String, Parameter>,
    tables: Tables,
    /// Load-time constants of a Lorentz read; `None` for the dot read.
    lorentz: Option<LorentzRead>,
    identity: String,
    output_bias: Vec<i128>,
    output_shifts: Vec<i32>,
    recurrent_bias: Vec<i128>,
    read_age: Vec<i128>,
    update_bias: Vec<i128>,
    update_gate_bias: Vec<i128>,
    copy_gate_bias: Vec<i128>,
    read_key_bias: Vec<i128>,
    read_value_bias: Vec<i128>,
    read_query_bias: Vec<i128>,
    read_no_read_bias: Vec<i128>,
    p_embedding: Parameter,
    p_recurrent_input: Parameter,
    p_recurrent_state: Parameter,
    p_read_query: Parameter,
    p_read_no_read: Parameter,
    p_update: Parameter,
    p_update_gate: Parameter,
    p_copy_gate: Parameter,
    p_read_key: Parameter,
    p_read_value: Parameter,
    p_output_norm: Parameter,
}

pub struct IntegerSession {
    identity: String,
    state: Vec<i32>,
    keys: Vec<[i32; KEY_DIM]>,
    /// Squared key norms in code units, kept for a Lorentz read only.
    key_norms: Vec<i128>,
    values: SessionValues,
    tokens: Vec<u32>,
}

// Ordinary sessions retain their original 256-coordinate allocation; only the
// explicit wide profile allocates 576-coordinate rows. Google's SessionState
// and its fixed VAL_DIM storage are independent of this enum.
enum SessionValues {
    Retained(Vec<[i32; VAL_DIM]>),
    Dialogue576(Vec<[i32; 576]>),
}

impl SessionValues {
    fn read(&self, masses: &[u64], width: usize) -> Result<Vec<i32>> {
        let mut read = vec![0i32; width];
        match self {
            Self::Retained(values) => {
                if !matches!(width, 128 | 256) || masses.len() != values.len() {
                    return Err(invalid("retained value memory shape differs"));
                }
                let mut sum_coords = [0i64; VAL_DIM];
                for (&mass, value) in masses.iter().zip(values) {
                    accumulate_slot_value_1x(&mut sum_coords, mass, value);
                }
                for (coordinate, out) in read.iter_mut().enumerate() {
                    *out = quantize(sum_coords[coordinate] as i128, 48 + 14, STATE_BITS)?;
                }
            }
            Self::Dialogue576(values) => {
                if width != 576 || masses.len() != values.len() {
                    return Err(invalid("dialogue576 value memory shape differs"));
                }
                let mut sum_coords = [0i64; 576];
                for (&mass, value) in masses.iter().zip(values) {
                    accumulate_slot_value_576(&mut sum_coords, mass, value);
                }
                for (coordinate, out) in read.iter_mut().enumerate() {
                    *out = quantize(sum_coords[coordinate] as i128, 48 + 14, STATE_BITS)?;
                }
            }
        }
        Ok(read)
    }

    fn push(&mut self, value: &[i32]) -> Result<()> {
        match self {
            Self::Retained(values) => {
                if !matches!(value.len(), 128 | 256) {
                    return Err(invalid("retained value write width differs"));
                }
                let mut row = [0; VAL_DIM];
                row[..value.len()].copy_from_slice(value);
                values.push(row);
            }
            Self::Dialogue576(values) => {
                let row: [i32; 576] = value
                    .try_into()
                    .map_err(|_| invalid("dialogue576 value write width differs"))?;
                values.push(row);
            }
        }
        Ok(())
    }
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
pub const L2_PAGE_CAPACITY: usize = 64;
pub const TOTAL_MEMORY_CANDIDATES: usize =
    PERSISTENT_CAPACITY + DIALOGUE_CAPACITY + L2_PAGE_CAPACITY; // 320
pub const MAX_SCORES_CAPACITY: usize = TOTAL_MEMORY_CANDIDATES + 1; // 321
pub const AGE_HORIZON_CLAMP: usize = 63;

/// Canonical sequential primes for the 224 L1 dialogue slots (p_0 = 2 through p_223 = 1423).
pub const SLOT_PRIMES_224: [u32; 224] = [
    2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97,
    101, 103, 107, 109, 113, 127, 131, 137, 139, 149, 151, 157, 163, 167, 173, 179, 181, 191, 193,
    197, 199, 211, 223, 227, 229, 233, 239, 241, 251, 257, 263, 269, 271, 277, 281, 283, 293, 307,
    311, 313, 317, 331, 337, 347, 349, 353, 359, 367, 373, 379, 383, 389, 397, 401, 409, 419, 421,
    431, 433, 439, 443, 449, 457, 461, 463, 467, 479, 487, 491, 499, 503, 509, 521, 523, 541, 547,
    557, 563, 569, 571, 577, 587, 593, 599, 601, 607, 613, 617, 619, 631, 641, 643, 647, 653, 659,
    661, 673, 677, 683, 691, 701, 709, 719, 727, 733, 739, 743, 751, 757, 761, 769, 773, 787, 797,
    809, 811, 821, 823, 827, 829, 839, 853, 857, 859, 863, 877, 881, 883, 887, 907, 911, 919, 929,
    937, 941, 947, 953, 967, 971, 977, 983, 991, 997, 1009, 1013, 1019, 1021, 1031, 1033, 1039,
    1049, 1051, 1061, 1063, 1069, 1087, 1091, 1093, 1097, 1103, 1109, 1117, 1123, 1129, 1151, 1153,
    1163, 1171, 1181, 1187, 1193, 1201, 1213, 1217, 1223, 1229, 1231, 1237, 1249, 1259, 1277, 1279,
    1283, 1289, 1291, 1297, 1301, 1303, 1307, 1319, 1321, 1327, 1361, 1367, 1373, 1381, 1399, 1409,
    1423,
];

/// Fibonacci numbers F_1 through F_24 for turn lengths up to 24 slots.
pub const FIBONACCI_WEIGHTS: [u64; 24] = [
    1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610, 987, 1597, 2584, 4181, 6765, 10946,
    17711, 28657, 46368,
];

/// Precomputed prefix sums of FIBONACCI_WEIGHTS where FIBONACCI_PREFIX_SUMS[n] = sum_{i=0}^{n-1} F_{i+1}.
/// Precomputing prefix sums eliminates dynamic summation loops, preventing LLVM auto-vectorization
/// and eliminating Class III `fmov` register transfer instructions.
pub const FIBONACCI_PREFIX_SUMS: [u64; 25] = [
    0, 1, 2, 4, 7, 12, 20, 33, 54, 88, 143, 232, 376, 609, 986, 1596, 2583, 4180, 6764, 10945,
    17710, 28656, 46367, 75024, 121392,
];

pub const GALOIS_POLY_64: u64 = 0xC96C_5795_D787_0F42;
pub const GOLDEN_RATIO_IV_64: u64 = 0x9E37_79B9_7F4A_7C15;

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

/// Compressed L2 Galois prime memory page for evicted dialogue turns ($K = 512..1024$).
///
/// Power-of-two padded (2048 bytes = 2^11) with 64-byte alignment to guarantee
/// strictly zero multiplier (`madd`/`smull`) instructions during array indexing.
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct L2PrimePage {
    pub key: [i32; KEY_DIM],
    pub value: [i32; VAL_DIM],
    pub prime_signature: u64,
    pub salient_tokens: [u32; 4],
    pub start_seq: usize,
    pub end_seq: usize,
    pub turn_id: u32,
    pub _pad: [u8; 720],
}

impl Default for L2PrimePage {
    fn default() -> Self {
        Self {
            key: [0i32; KEY_DIM],
            value: [0i32; VAL_DIM],
            prime_signature: 0,
            salient_tokens: [0u32; 4],
            start_seq: 0,
            end_seq: 0,
            turn_id: 0,
            _pad: [0u8; 720],
        }
    }
}

impl serde::Serialize for L2PrimePage {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("L2PrimePage", 7)?;
        state.serialize_field("key", &self.key[..])?;
        state.serialize_field("value", &self.value[..])?;
        state.serialize_field("prime_signature", &self.prime_signature)?;
        state.serialize_field("salient_tokens", &self.salient_tokens)?;
        state.serialize_field("start_seq", &self.start_seq)?;
        state.serialize_field("end_seq", &self.end_seq)?;
        state.serialize_field("turn_id", &self.turn_id)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for L2PrimePage {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct L2PrimePageHelper {
            key: Vec<i32>,
            value: Vec<i32>,
            prime_signature: u64,
            salient_tokens: [u32; 4],
            start_seq: usize,
            end_seq: usize,
            turn_id: u32,
        }

        let helper = L2PrimePageHelper::deserialize(deserializer)?;
        if helper.key.len() != KEY_DIM {
            return Err(serde::de::Error::invalid_length(
                helper.key.len(),
                &format!("{KEY_DIM} elements").as_str(),
            ));
        }
        if helper.value.len() != VAL_DIM {
            return Err(serde::de::Error::invalid_length(
                helper.value.len(),
                &format!("{VAL_DIM} elements").as_str(),
            ));
        }

        let mut page = L2PrimePage::default();
        page.key.copy_from_slice(&helper.key);
        page.value.copy_from_slice(&helper.value);
        page.prime_signature = helper.prime_signature;
        page.salient_tokens = helper.salient_tokens;
        page.start_seq = helper.start_seq;
        page.end_seq = helper.end_seq;
        page.turn_id = helper.turn_id;
        Ok(page)
    }
}

/// Partitioned conversational memory state for zero-matmul serving.
///
/// Power-of-two aligned memory structures (key stride 256 B = 2^8, value stride 1024 B = 2^10,
/// L2 page stride 2048 B = 2^11) eliminate all hardware address multiplier (`madd`) instructions during serving.
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

    // --- L2 Prime Paging Store (64 pages) ---
    pub l2_pages: Box<[L2PrimePage; L2_PAGE_CAPACITY]>,
    pub l2_cursor: usize,
    pub l2_len: usize,
    pub l2_seen: u64,
    pub last_compressed_turn_id: u32,

    // --- Geometric & Phase State ---
    pub zeta_state: T8ZetaState,
    pub hopf_state: HopfFiberPointQ30,
    pub cumulative_holonomy_q30: i64,

    pub age_horizon_clamp: usize,

    // --- Scratch Buffers (Reused across steps to eliminate stack/heap allocations) ---
    pub scratch_products: Vec<[i64; 16]>,
    pub copy_scratch: Vec<u64>,

    // --- Step Output Buffers (Zero heap allocation per step) ---
    pub last_probabilities: Box<[u64; 4096]>,
    pub last_read_masses: Vec<u64>,
    pub last_no_read_mass: u64,
    pub last_copy_gate: i32,
    pub has_step: bool,
}

impl SessionState {
    pub fn persistent_len(&self) -> usize {
        self.persistent_tokens.len()
    }

    pub fn dialogue_len(&self) -> usize {
        self.dialogue_len
    }

    pub fn l2_len(&self) -> usize {
        self.l2_len
    }

    pub fn l2_pages(&self) -> &[L2PrimePage] {
        &self.l2_pages[..self.l2_len]
    }

    pub fn total_len(&self) -> usize {
        self.persistent_len() + self.dialogue_len() + self.l2_len()
    }

    pub fn active_slots_count(&self) -> usize {
        self.total_len()
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

    #[inline(always)]
    pub fn last_probabilities(&self) -> &[u64] {
        &self.last_probabilities[..]
    }

    #[inline(always)]
    pub fn last_read_masses(&self) -> &[u64] {
        &self.last_read_masses
    }

    #[inline(always)]
    pub fn last_no_read_mass(&self) -> u64 {
        self.last_no_read_mass
    }

    #[inline(always)]
    pub fn last_copy_gate(&self) -> i32 {
        self.last_copy_gate
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

#[inline(always)]
fn arithmetic<T>(value: MathResult<T>) -> Result<T> {
    value.map_err(|e| invalid(format!("integer model arithmetic: {e}")))
}

#[inline(always)]
fn scaled(value: i128, shift: i32) -> Result<i128> {
    arithmetic(math::scale_pow2(value, shift))
}

#[inline(always)]
fn product(a: i128, b: i128) -> Result<i128> {
    arithmetic(math::checked_mul(a, b))
}

#[inline(always)]
fn divide(a: i128, b: i128) -> Result<i128> {
    arithmetic(math::div_round(a, b))
}

#[inline(always)]
fn quantize(value: i128, input_bits: i32, output_bits: i32) -> Result<i32> {
    Ok(scaled(value, output_bits - input_bits)?.clamp(-32767, 32767) as i32)
}

#[inline(always)]
fn mul_shift_add_i64(a: i64, b: i64) -> i64 {
    if a == 0 || b == 0 {
        return 0;
    }
    let neg = (a < 0) ^ (b < 0);
    let mut u = a.unsigned_abs();
    let mut v = b.unsigned_abs();
    if u < v {
        core::mem::swap(&mut u, &mut v);
    }
    let mut res = 0u64;
    while v != 0 {
        let tz = v.trailing_zeros();
        if tz > 0 {
            u <<= tz;
            v >>= tz;
        }
        res += u;
        v >>= 1;
        if v != 0 {
            u <<= 1;
        }
    }
    if neg {
        -(res as i64)
    } else {
        res as i64
    }
}

#[inline(always)]
fn build_i64_nibble_table(val: i64) -> [i64; 16] {
    let mut t = [0i64; 16];
    for d in 1..16 {
        t[d] = core::hint::black_box(t[d - 1] + val);
    }
    t
}

#[inline(always)]
fn build_i64_nibble_table_4x(
    u0: i64,
    u1: i64,
    u2: i64,
    u3: i64,
) -> ([i64; 16], [i64; 16], [i64; 16], [i64; 16]) {
    let mut t0 = [0i64; 16];
    let mut t1 = [0i64; 16];
    let mut t2 = [0i64; 16];
    let mut t3 = [0i64; 16];
    for d in 1..16 {
        t0[d] = core::hint::black_box(t0[d - 1] + u0);
        t1[d] = core::hint::black_box(t1[d - 1] + u1);
        t2[d] = core::hint::black_box(t2[d - 1] + u2);
        t3[d] = core::hint::black_box(t3[d - 1] + u3);
    }
    (t0, t1, t2, t3)
}

#[inline(always)]
fn mul_nibble_table_i64(table: &[i64; 16], x: i64) -> i64 {
    if x == 0 {
        return 0;
    }
    let neg = x < 0;
    let u = x.unsigned_abs() as usize;
    let p = table[u & 0x0f]
        + (table[(u >> 4) & 0x0f] << 4)
        + (table[(u >> 8) & 0x0f] << 8)
        + (table[(u >> 12) & 0x0f] << 12);
    if neg {
        -p
    } else {
        p
    }
}

#[inline(always)]
fn mul_nibble_table_4x(
    t0: &[i64; 16],
    t1: &[i64; 16],
    t2: &[i64; 16],
    t3: &[i64; 16],
    x: i64,
) -> (i64, i64, i64, i64) {
    let neg = x < 0;
    let u = x.unsigned_abs() as usize;
    let k0 = u & 0x0f;
    let k1 = (u >> 4) & 0x0f;
    let k2 = (u >> 8) & 0x0f;
    let k3 = (u >> 12) & 0x0f;

    let p0 = t0[k0] + (t0[k1] << 4) + (t0[k2] << 8) + (t0[k3] << 12);
    let p1 = t1[k0] + (t1[k1] << 4) + (t1[k2] << 8) + (t1[k3] << 12);
    let p2 = t2[k0] + (t2[k1] << 4) + (t2[k2] << 8) + (t2[k3] << 12);
    let p3 = t3[k0] + (t3[k1] << 4) + (t3[k2] << 8) + (t3[k3] << 12);

    if neg {
        (-p0, -p1, -p2, -p3)
    } else {
        (p0, p1, p2, p3)
    }
}

#[inline(always)]
fn mul_code_i64(x: i64, code: i16) -> i64 {
    let neg = code < 0;
    let u = code.unsigned_abs() as usize;
    let p = match u {
        0 => 0,
        1 => x,
        2 => x << 1,
        3 => (x << 1) + x,
        4 => x << 2,
        5 => (x << 2) + x,
        6 => (x << 2) + (x << 1),
        7 => (x << 3) - x,
        _ => {
            let mut acc = 0i64;
            let mut val = x;
            let mut bits = u;
            while bits != 0 {
                if bits & 1 != 0 {
                    acc += val;
                }
                val <<= 1;
                bits >>= 1;
            }
            acc
        }
    };
    if neg {
        -p
    } else {
        p
    }
}

#[inline(always)]
fn div_rem_u64(numerator: u64, denominator: u64) -> (u64, u64) {
    if numerator < denominator {
        return (0, numerator);
    }
    let shift = denominator.leading_zeros() - numerator.leading_zeros();
    let mut divisor = denominator << shift;
    let mut place = 1u64 << shift;
    let mut remainder = numerator;
    let mut quotient = 0u64;
    while place >= 8 {
        if remainder >= divisor {
            remainder -= divisor;
            quotient |= place;
        }
        divisor >>= 1;
        place >>= 1;
        if remainder >= divisor {
            remainder -= divisor;
            quotient |= place;
        }
        divisor >>= 1;
        place >>= 1;
        if remainder >= divisor {
            remainder -= divisor;
            quotient |= place;
        }
        divisor >>= 1;
        place >>= 1;
        if remainder >= divisor {
            remainder -= divisor;
            quotient |= place;
        }
        divisor >>= 1;
        place >>= 1;
    }
    while place != 0 {
        if remainder >= divisor {
            remainder -= divisor;
            quotient |= place;
        }
        divisor >>= 1;
        place >>= 1;
    }
    (quotient, remainder)
}

#[inline(always)]
fn div_round_100m_raw(raw: u64, u_const: u64) -> Result<u64> {
    const DENOM: u64 = 100_000_000;
    if raw <= u_const {
        let diff = u_const - raw;
        let (q, r) = div_rem_u64(diff, DENOM);
        let rounded = q + if r >= DENOM - r { 1 } else { 0 };
        Ok(raw + rounded)
    } else {
        let pos_diff = raw - u_const;
        let (q_sub, r_sub) = div_rem_u64(pos_diff, DENOM);
        let (q, r) = if r_sub == 0 {
            (raw - q_sub, 0u64)
        } else {
            (raw - q_sub - 1, DENOM - r_sub)
        };
        let rounded = q + if r >= DENOM - r { 1 } else { 0 };
        Ok(rounded)
    }
}

#[inline(always)]
fn scale_and_quantize_logit(dot: i64, shift: i32, bias: i128) -> i32 {
    let val = if shift >= 0 {
        i128::from(dot) << (shift as u32)
    } else {
        let right = shift.unsigned_abs();
        let half = 1u128 << (right - 1);
        let neg = dot < 0;
        let mag = (dot.unsigned_abs() as u128 + half) >> right;
        if neg {
            -(mag as i128)
        } else {
            mag as i128
        }
    };
    let sum = val + bias;
    let neg = sum < 0;
    let mag = sum.unsigned_abs();
    let rounded = (mag + (1u128 << 31)) >> 32;
    let res = if neg {
        -(rounded as i64)
    } else {
        rounded as i64
    };
    res.clamp(-32767, 32767) as i32
}

#[inline(always)]
fn scale_and_quantize_score(dot: i64, age_val: i128) -> i32 {
    let s = (i128::from(dot) << 21) + age_val;
    let neg = s < 0;
    let mag = s.unsigned_abs();
    let rounded = (mag + (1u128 << 31)) >> 32;
    let res = if neg {
        -(rounded as i64)
    } else {
        rounded as i64
    };
    res.clamp(-32767, 32767) as i32
}

#[inline(always)]
fn div_round_u64(numerator: u64, denominator: u64, neg: bool) -> i64 {
    if denominator == 0 {
        return 0;
    }
    let (q, r) = div_rem_u64(numerator, denominator);
    let rounded = q + if r >= denominator - r { 1 } else { 0 };
    if neg {
        -(rounded as i64)
    } else {
        rounded as i64
    }
}

/// Compute Galois weighted integer barycenter key across turn slots using the 2-accumulator reverse Fibonacci recurrence.
/// Strictly 0 hardware multipliers, 0 hardware dividers, 0 floating-point registers.
#[inline(never)]
pub fn compress_barycenter_key_fibonacci(
    dialogue_keys: &[[i32; KEY_DIM]; DIALOGUE_CAPACITY],
    slot_indices: &[usize],
    out_key: &mut [i32; KEY_DIM],
) {
    let slots = if slot_indices.len() > 24 {
        &slot_indices[slot_indices.len() - 24..]
    } else {
        slot_indices
    };
    let n = slots.len();
    if n == 0 {
        out_key.fill(0);
        return;
    }
    if n == 1 {
        out_key.copy_from_slice(&dialogue_keys[slots[0]]);
        return;
    }

    // Direct constant table lookup of prefix sum (strictly 0 loop, 0 SIMD, 0 fmov)
    let total_weight = FIBONACCI_PREFIX_SUMS[n.min(24)];

    if total_weight == 0 {
        out_key.fill(0);
        return;
    }

    // Coordinate-wise 2-accumulator reverse Fibonacci sweep (strictly zero mul/div)
    for d in 0..KEY_DIM {
        let mut a = 0i64;
        let mut b = 0i64;
        for &slot_idx in slots.iter().rev() {
            let val = dialogue_keys[slot_idx][d] as i64;
            let next_a = a + b + val;
            b = a;
            a = next_a;
        }

        // Restoring binary division rounded to nearest (0 sdiv/udiv)
        let neg = a < 0;
        let mag = a.unsigned_abs();
        let (q, r) = div_rem_u64(mag, total_weight);
        let rounded = q + if r >= total_weight - r { 1 } else { 0 };
        let res = if neg {
            -(rounded as i64)
        } else {
            rounded as i64
        };
        out_key[d] = res.clamp(-32767, 32767) as i32;
    }
}

/// Branchless step of 64-bit Galois LFSR with primitive polynomial.
#[inline(always)]
pub fn galois_lfsr_step(state: u64) -> u64 {
    let lsb = state & 1;
    let mask = (0u64).wrapping_sub(lsb);
    (state >> 1) ^ (GALOIS_POLY_64 & mask)
}

/// Absorb a 64-bit word into the Galois LFSR with 8 rounds of diffusion.
#[inline(always)]
pub fn galois_lfsr_absorb(mut state: u64, word: u64) -> u64 {
    state ^= word;
    for _ in 0..8 {
        let mask = (0u64).wrapping_sub(state & 1);
        state = (state >> 1) ^ (GALOIS_POLY_64 & mask);
    }
    state
}

/// Compute exact prime signature for a dialogue turn from tokens and slot prime addresses.
/// Strictly 0 hardware multipliers, 0 hardware dividers, 0 floats.
#[inline(never)]
pub fn compute_turn_prime_signature(
    dialogue_tokens: &[u32],
    slot_indices: &[usize],
    turn_id: u32,
    start_seq: u64,
    end_seq: u64,
) -> u64 {
    let mut state = GOLDEN_RATIO_IV_64 ^ ((turn_id as u64) << 32) ^ start_seq;
    if state == 0 {
        state = GOLDEN_RATIO_IV_64;
    }

    for &slot_idx in slot_indices {
        let token = if slot_idx < dialogue_tokens.len() {
            dialogue_tokens[slot_idx]
        } else {
            0
        };
        let prime = if slot_idx < SLOT_PRIMES_224.len() {
            SLOT_PRIMES_224[slot_idx]
        } else {
            let (_, rem) = div_rem_u64(slot_idx as u64, SLOT_PRIMES_224.len() as u64);
            SLOT_PRIMES_224[if (rem as usize) < SLOT_PRIMES_224.len() {
                rem as usize
            } else {
                0
            }]
        };
        let word = ((prime as u64) << 32) | (token as u64);
        state = galois_lfsr_absorb(state, word);
    }

    let final_word = (end_seq << 16) ^ (slot_indices.len() as u64);
    state = galois_lfsr_absorb(state, final_word);

    if state == 0 {
        GOLDEN_RATIO_IV_64
    } else {
        state
    }
}

/// Heuristic token salience scoring for L2 compression.
/// Zero multipliers, zero dividers, zero floats.
#[inline(always)]
pub fn score_token_salience(token: u32, key: &[i32; KEY_DIM]) -> i32 {
    let mut score = 0i32;

    // Special & role tokens (<|system|>, <|user|>, <|assistant|>, <|turn_end|>, BOS, EOS, UNK)
    if token <= 6 {
        return -1_000_000;
    }

    // ASCII whitespace and common punctuation penalties
    if token <= 32
        || (33..=47).contains(&token)
        || (58..=64).contains(&token)
        || (91..=96).contains(&token)
        || (123..=126).contains(&token)
    {
        score -= 500_000;
    } else if (token as u8).is_ascii_digit() {
        // Numeric tokens (+60k)
        score += 60_000;
    } else if (token as u8).is_ascii_uppercase() {
        // Uppercase / Proper nouns (+50k)
        score += 50_000;
    } else if token >= 256 {
        // Multi-byte BPE subwords (entity subwords)
        score += 10_000 + (token as i32 & 0x0FFF);
    }

    // Geometric Key L1 Energy: sum(|k[d]|) >> 6
    let mut l1_norm = 0i32;
    for &k in key {
        l1_norm = l1_norm.saturating_add((k.unsigned_abs().min(32767)) as i32);
    }
    score = score.saturating_add(l1_norm >> 6);

    score
}

/// Extract up to 4 salient entity tokens from turn tokens using zero-matmul scoring.
#[inline(never)]
pub fn extract_salient_tokens(
    dialogue_tokens: &[u32],
    dialogue_keys: &[[i32; KEY_DIM]; DIALOGUE_CAPACITY],
    slot_indices: &[usize],
    out_tokens: &mut [u32; 4],
) {
    if slot_indices.is_empty() {
        out_tokens.fill(0);
        return;
    }

    // Collect up to 224 candidate tokens and scores on stack (< 2 KB stack space)
    let n = slot_indices.len().min(DIALOGUE_CAPACITY);
    let mut candidates: [(u32, i32); DIALOGUE_CAPACITY] = [(0, i32::MIN); DIALOGUE_CAPACITY];
    let mut num_candidates = 0;

    for &slot_idx in &slot_indices[..n] {
        let tok = if slot_idx < dialogue_tokens.len() {
            dialogue_tokens[slot_idx]
        } else {
            0
        };
        let key = &dialogue_keys[slot_idx];
        let score = score_token_salience(tok, key);

        // Deduplicate: if token already in candidates, update score with max
        let mut found = false;
        for c in &mut candidates[..num_candidates] {
            if c.0 == tok {
                if score > c.1 {
                    c.1 = score;
                }
                found = true;
                break;
            }
        }
        if !found && num_candidates < DIALOGUE_CAPACITY {
            candidates[num_candidates] = (tok, score);
            num_candidates += 1;
        }
    }

    // Sort descending by score using insertion sort (0 heap allocations)
    for i in 1..num_candidates {
        let mut j = i;
        while j > 0 && candidates[j].1 > candidates[j - 1].1 {
            candidates.swap(j, j - 1);
            j -= 1;
        }
    }

    // Populate top 4 salient tokens
    let mut count = 0;
    for c in &candidates[..num_candidates] {
        if c.1 > -500_000 {
            out_tokens[count] = c.0;
            count += 1;
            if count == 4 {
                break;
            }
        }
    }

    // If fewer than 4 candidates had positive/acceptable scores, pad
    if count == 0 {
        let first_tok = if slot_indices[0] < dialogue_tokens.len() {
            dialogue_tokens[slot_indices[0]]
        } else {
            0
        };
        out_tokens.fill(first_tok);
    } else {
        let pad_tok = out_tokens[0];
        out_tokens[count..4].fill(pad_tok);
    }
}

#[inline(always)]
fn square_u64_to_u128(x: u64) -> u128 {
    let mut u = x as u128;
    let mut v = x;
    let mut res = 0u128;
    while v != 0 {
        let tz = v.trailing_zeros();
        if tz > 0 {
            u <<= tz;
            v >>= tz;
        }
        res += u;
        v >>= 1;
        if v != 0 {
            u <<= 1;
        }
    }
    res
}

#[inline(always)]
fn quantize_q25_to_q11(v: i64) -> i32 {
    let neg = v < 0;
    let mag = v.unsigned_abs();
    let rounded = (mag + 8192) >> 14;
    let res = if neg {
        -(rounded as i64)
    } else {
        rounded as i64
    };
    res.clamp(-32767, 32767) as i32
}

#[inline(always)]
fn quantize_q39_to_q11(v: i64) -> i32 {
    let neg = v < 0;
    let mag = v.unsigned_abs();
    let rounded = (mag + (1u64 << 27)) >> 28;
    let res = if neg {
        -(rounded as i64)
    } else {
        rounded as i64
    };
    res.clamp(-32767, 32767) as i32
}

#[inline(always)]
fn quantize_q29_to_q11(v: i64) -> i32 {
    let neg = v < 0;
    let mag = v.unsigned_abs();
    let rounded = (mag + (1u64 << 17)) >> 18;
    let res = if neg {
        -(rounded as i64)
    } else {
        rounded as i64
    };
    res.clamp(-32767, 32767) as i32
}

#[allow(dead_code)]
const MUL99_TABLE: [u64; 16] = [
    0,
    99_999_999,
    199_999_998,
    299_999_997,
    399_999_996,
    499_999_995,
    599_999_994,
    699_999_993,
    799_999_992,
    899_999_991,
    999_999_990,
    1_099_999_989,
    1_199_999_988,
    1_299_999_987,
    1_399_999_986,
    1_499_999_985,
];

#[inline(always)]
fn build_fraction_table(fraction: u128) -> [u128; 16] {
    let mut table = [0u128; 16];
    for d in 1..16 {
        table[d] = core::hint::black_box(table[d - 1] + fraction);
    }
    table
}

#[inline(always)]
fn mul_fraction_radix16(v: u64, table: &[u128; 16]) -> u128 {
    let v_u = v as u128;
    let mut acc = table[(v_u & 0xF) as usize]
        + (table[((v_u >> 4) & 0xF) as usize] << 4)
        + (table[((v_u >> 8) & 0xF) as usize] << 8)
        + (table[((v_u >> 12) & 0xF) as usize] << 12);
    if v >= (1 << 16) {
        acc += (table[((v_u >> 16) & 0xF) as usize] << 16)
            + (table[((v_u >> 20) & 0xF) as usize] << 20)
            + (table[((v_u >> 24) & 0xF) as usize] << 24)
            + (table[((v_u >> 28) & 0xF) as usize] << 28);
        if v >= (1 << 32) {
            acc += (table[((v_u >> 32) & 0xF) as usize] << 32)
                + (table[((v_u >> 36) & 0xF) as usize] << 36)
                + (table[((v_u >> 40) & 0xF) as usize] << 40)
                + (table[((v_u >> 44) & 0xF) as usize] << 44)
                + (table[((v_u >> 48) & 0xF) as usize] << 48);
        }
    }
    acc
}

#[allow(dead_code)]
#[inline(always)]
fn mul99_radix16(raw: u64) -> u128 {
    let r_u = raw as u128;
    let m0 = MUL99_TABLE[(r_u & 0xF) as usize] as u128;
    let m1 = MUL99_TABLE[((r_u >> 4) & 0xF) as usize] as u128;
    let m2 = MUL99_TABLE[((r_u >> 8) & 0xF) as usize] as u128;
    let m3 = MUL99_TABLE[((r_u >> 12) & 0xF) as usize] as u128;
    let m4 = MUL99_TABLE[((r_u >> 16) & 0xF) as usize] as u128;
    let m5 = MUL99_TABLE[((r_u >> 20) & 0xF) as usize] as u128;
    let m6 = MUL99_TABLE[((r_u >> 24) & 0xF) as usize] as u128;
    let m7 = MUL99_TABLE[((r_u >> 28) & 0xF) as usize] as u128;
    let m8 = MUL99_TABLE[((r_u >> 32) & 0xF) as usize] as u128;
    let m9 = MUL99_TABLE[((r_u >> 36) & 0xF) as usize] as u128;
    let m10 = MUL99_TABLE[((r_u >> 40) & 0xF) as usize] as u128;
    let m11 = MUL99_TABLE[((r_u >> 44) & 0xF) as usize] as u128;
    let m12 = MUL99_TABLE[((r_u >> 48) & 0xF) as usize] as u128;

    m0 + (m1 << 4)
        + (m2 << 8)
        + (m3 << 12)
        + (m4 << 16)
        + (m5 << 20)
        + (m6 << 24)
        + (m7 << 28)
        + (m8 << 32)
        + (m9 << 36)
        + (m10 << 40)
        + (m11 << 44)
        + (m12 << 48)
}

#[inline(always)]
fn build_div_table(d: u128) -> [u128; 16] {
    let d2 = d << 1;
    let d4 = d << 2;
    let d8 = d << 3;
    let d3 = d2 + d;
    let d5 = d4 + d;
    let d6 = d4 + d2;
    let d7 = d8 - d;
    [
        0,
        d,
        d2,
        d3,
        d4,
        d5,
        d6,
        d7,
        d8,
        d8 + d,
        d8 + d2,
        d8 + d3,
        d8 + d4,
        d8 + d5,
        d8 + d6,
        (d << 4) - d,
    ]
}

#[inline(always)]
fn fast_div_round_radix16(numerator: u128, t: &[u128; 16], denominator: u128) -> u64 {
    if denominator == 0 {
        return 0;
    }
    if numerator < denominator {
        return if numerator >= denominator - numerator {
            1
        } else {
            0
        };
    }
    let shift = denominator.leading_zeros() - numerator.leading_zeros();
    let mut s = (((shift + 3) >> 2) << 2).min(124);
    let mut remainder = numerator;
    let mut quotient = 0u128;

    loop {
        let rem_slice = remainder >> s;
        let k = if rem_slice >= t[8] {
            if rem_slice >= t[12] {
                if rem_slice >= t[14] {
                    if rem_slice >= t[15] {
                        15
                    } else {
                        14
                    }
                } else if rem_slice >= t[13] {
                    13
                } else {
                    12
                }
            } else if rem_slice >= t[10] {
                if rem_slice >= t[11] {
                    11
                } else {
                    10
                }
            } else if rem_slice >= t[9] {
                9
            } else {
                8
            }
        } else if rem_slice >= t[4] {
            if rem_slice >= t[6] {
                if rem_slice >= t[7] {
                    7
                } else {
                    6
                }
            } else if rem_slice >= t[5] {
                5
            } else {
                4
            }
        } else if rem_slice >= t[2] {
            if rem_slice >= t[3] {
                3
            } else {
                2
            }
        } else if rem_slice >= t[1] {
            1
        } else {
            0
        };

        if k != 0 {
            remainder -= t[k] << s;
            quotient |= (k as u128) << s;
        }

        if s == 0 {
            break;
        }
        s -= 4;
    }

    if remainder >= denominator - remainder {
        quotient += 1;
    }
    quotient as u64
}

#[inline(always)]
fn build_div_table_u64(d: u64) -> [u64; 16] {
    let d2 = d << 1;
    let d4 = d << 2;
    let d8 = d << 3;
    let d3 = d2 + d;
    let d5 = d4 + d;
    let d6 = d4 + d2;
    let d7 = d8 - d;
    [
        0,
        d,
        d2,
        d3,
        d4,
        d5,
        d6,
        d7,
        d8,
        d8 + d,
        d8 + d2,
        d8 + d3,
        d8 + d4,
        d8 + d5,
        d8 + d6,
        (d << 4) - d,
    ]
}

#[inline(always)]
fn fast_div_round_radix16_u64(numerator: u128, t: &[u64; 16], denominator: u64) -> u64 {
    if denominator == 0 {
        return 0;
    }
    if numerator < denominator as u128 {
        return if numerator >= (denominator as u128 - numerator) {
            1
        } else {
            0
        };
    }
    let num_bits = 128 - numerator.leading_zeros();
    let num_nibbles = (num_bits + 3) >> 2;
    let mut rem = 0u64;
    let mut quotient = 0u64;

    let d8 = t[8];
    let d4 = t[4];
    let d2 = t[2];
    let d1 = t[1];

    for i in (0..num_nibbles).rev() {
        let nibble = ((numerator >> (i * 4)) & 0xF) as u64;
        rem = (rem << 4) | nibble;
        let mut k = 0u64;
        if rem >= d8 {
            k |= 8;
            rem -= d8;
        }
        if rem >= d4 {
            k |= 4;
            rem -= d4;
        }
        if rem >= d2 {
            k |= 2;
            rem -= d2;
        }
        if rem >= d1 {
            k |= 1;
            rem -= d1;
        }
        quotient = (quotient << 4) | k;
    }

    if rem >= denominator - rem {
        quotient += 1;
    }
    quotient
}

#[allow(dead_code)]
const DIV100M_TABLE_U64: [u64; 16] = [
    0,
    100_000_000,
    200_000_000,
    300_000_000,
    400_000_000,
    500_000_000,
    600_000_000,
    700_000_000,
    800_000_000,
    900_000_000,
    1_000_000_000,
    1_100_000_000,
    1_200_000_000,
    1_300_000_000,
    1_400_000_000,
    1_500_000_000,
];

#[allow(dead_code)]
#[inline(always)]
fn fast_div_round(numerator: u128, denominator: u128) -> u64 {
    if denominator <= (1u128 << 60) {
        let t = build_div_table_u64(denominator as u64);
        fast_div_round_radix16_u64(numerator, &t, denominator as u64)
    } else {
        let t = build_div_table(denominator);
        fast_div_round_radix16(numerator, &t, denominator)
    }
}

#[allow(dead_code)]
#[inline(always)]
fn fast_div_round_100m(numerator: u128) -> u64 {
    fast_div_round_radix16_u64(numerator, &DIV100M_TABLE_U64, 100_000_000)
}

#[inline(always)]
fn fill_low_bit_products(input: &[i32], out: &mut [[i64; 16]]) {
    let len = input.len().min(out.len());
    for i in 0..len {
        let x = i64::from(input[i]);
        let twice = x << 1;
        let four = x << 2;
        let three = x + twice;
        let five = x + four;
        let six = twice + four;
        let seven = (x << 3) - x;
        out[i] = [
            0, x, twice, three, four, five, six, seven, 0, -seven, -six, -five, -four, -three,
            -twice, -x,
        ];
    }
}

#[inline(always)]
fn fill_packed_low_bit_products(input: &[u8], out: &mut [[i64; 16]]) {
    for (&byte, pair) in input.iter().zip(out.chunks_exact_mut(2)) {
        for (nibble, table) in [byte & 15, byte >> 4].into_iter().zip(pair) {
            let x = i64::from((nibble as i8) << 4 >> 4);
            let twice = x << 1;
            let four = x << 2;
            let three = x + twice;
            let five = x + four;
            let six = twice + four;
            let seven = (x << 3) - x;
            *table = [
                0, x, twice, three, four, five, six, seven, 0, -seven, -six, -five, -four, -three,
                -twice, -x,
            ];
        }
    }
}

/// Build each input coordinate's signed4 multiples once for reuse across rows.
/// Slots use the coefficient's low nibble: 0..7 and -7..-1; reserved -8 is
/// unreachable after import validation and its slot is zero. i64 intermediates
/// keep these shifts/additions exact even for the full i32 input range.
#[inline(never)]
fn low_bit_products(input: &[i32]) -> Vec<[i64; 16]> {
    let mut products = vec![[0i64; 16]; input.len()];
    fill_low_bit_products(input, &mut products);
    products
}

/// Precompute 16 multiples (0..15) of coordinate `x` using only shifts, adds, and subtracts.
/// Strictly 0 hardware multipliers, dividers, or floats in ARM64 disassembly.
#[inline(always)]
fn build_coord_products(x: i32) -> [i64; 16] {
    let q = x as i64;
    let mut table = [0i64; 16];
    for d in 1..16 {
        table[d] = core::hint::black_box(table[d - 1] + q);
    }
    table
}

/// Compute exact inner product of coordinate table `p` (16 multiples of query coordinate)
/// with key coordinate `v` using radix-16 table lookup and shift-add.
/// Strictly 0 hardware multipliers, dividers, or floats in ARM64 disassembly.
#[inline(always)]
fn dot_coord(p: &[i64; 16], v: i32) -> i64 {
    let u = v.unsigned_abs() as usize;
    let m = p[u & 0x0f]
        + (p[(u >> 4) & 0x0f] << 4)
        + (p[(u >> 8) & 0x0f] << 8)
        + (p[(u >> 12) & 0x0f] << 12);
    if v < 0 {
        -m
    } else {
        m
    }
}

/// 4-slot tiled query-key dot product across KEY_DIM=64 coordinates.
/// Reuses precomputed query coordinate product tables across 4 active slots in registers.
/// Strictly 0 hardware multipliers, dividers, or floats in ARM64 disassembly.
#[inline(always)]
fn memory_dot_product_4x(
    query_products: &[[i64; 16]; KEY_DIM],
    k0: &[i32; KEY_DIM],
    k1: &[i32; KEY_DIM],
    k2: &[i32; KEY_DIM],
    k3: &[i32; KEY_DIM],
) -> (i64, i64, i64, i64) {
    let mut acc0 = 0i64;
    let mut acc1 = 0i64;
    let mut acc2 = 0i64;
    let mut acc3 = 0i64;

    for d in (0..KEY_DIM).step_by(4) {
        let p0 = &query_products[d];
        let p1 = &query_products[d + 1];
        let p2 = &query_products[d + 2];
        let p3 = &query_products[d + 3];

        acc0 += dot_coord(p0, k0[d])
            + dot_coord(p1, k0[d + 1])
            + dot_coord(p2, k0[d + 2])
            + dot_coord(p3, k0[d + 3]);
        acc1 += dot_coord(p0, k1[d])
            + dot_coord(p1, k1[d + 1])
            + dot_coord(p2, k1[d + 2])
            + dot_coord(p3, k1[d + 3]);
        acc2 += dot_coord(p0, k2[d])
            + dot_coord(p1, k2[d + 1])
            + dot_coord(p2, k2[d + 2])
            + dot_coord(p3, k2[d + 3]);
        acc3 += dot_coord(p0, k3[d])
            + dot_coord(p1, k3[d + 1])
            + dot_coord(p2, k3[d + 2])
            + dot_coord(p3, k3[d + 3]);
    }

    (acc0, acc1, acc2, acc3)
}

/// 1-slot query-key dot product across KEY_DIM=64 coordinates.
/// Strictly 0 hardware multipliers, dividers, or floats in ARM64 disassembly.
#[inline(always)]
fn memory_dot_product_1x(query_products: &[[i64; 16]; KEY_DIM], k: &[i32; KEY_DIM]) -> i64 {
    let mut acc = 0i64;
    for d in 0..KEY_DIM {
        acc += dot_coord(&query_products[d], k[d]);
    }
    acc
}

#[inline(always)]
fn build_mass_table_i64(m_u64: u64) -> [i64; 16] {
    let m = m_u64 as i64;
    let mut table = [0i64; 16];
    for d in 1..16 {
        table[d] = core::hint::black_box(table[d - 1] + m);
    }
    table
}

#[inline(always)]
fn mul_mass_coord(t: &[i64; 16], v: i32) -> i64 {
    let u = v.unsigned_abs() as usize;
    let p0 = t[u & 0x0f];
    let p1 = t[(u >> 4) & 0x0f].wrapping_shl(4);
    let p2 = t[(u >> 8) & 0x0f].wrapping_shl(8);
    let p3 = t[(u >> 12) & 0x0f].wrapping_shl(12);
    let p = p0.wrapping_add(p1).wrapping_add(p2).wrapping_add(p3);
    if v < 0 {
        p.wrapping_neg()
    } else {
        p
    }
}

/// 4-slot fused value accumulation across VAL_DIM=256 coordinates.
/// Fuses up to 4 active slots into a single coordinate sweep, reducing memory traffic by 75%.
/// Strictly 0 hardware multipliers, dividers, or floats in ARM64 disassembly.
#[inline(always)]
fn accumulate_slot_values_4x(
    sum_coords: &mut [i64; VAL_DIM],
    m: &[u64],
    v0: &[i32; VAL_DIM],
    v1: &[i32; VAL_DIM],
    v2: &[i32; VAL_DIM],
    v3: &[i32; VAL_DIM],
) {
    let mask = m[0] | m[1] | m[2] | m[3];
    if mask == 0 {
        return;
    }

    let active_count =
        (m[0] != 0) as u8 + (m[1] != 0) as u8 + (m[2] != 0) as u8 + (m[3] != 0) as u8;
    if active_count == 1 {
        if m[0] != 0 {
            accumulate_slot_value_1x(sum_coords, m[0], v0);
        } else if m[1] != 0 {
            accumulate_slot_value_1x(sum_coords, m[1], v1);
        } else if m[2] != 0 {
            accumulate_slot_value_1x(sum_coords, m[2], v2);
        } else {
            accumulate_slot_value_1x(sum_coords, m[3], v3);
        }
        return;
    }

    let t0 = if m[0] != 0 {
        Some(build_mass_table_i64(m[0]))
    } else {
        None
    };
    let t1 = if m[1] != 0 {
        Some(build_mass_table_i64(m[1]))
    } else {
        None
    };
    let t2 = if m[2] != 0 {
        Some(build_mass_table_i64(m[2]))
    } else {
        None
    };
    let t3 = if m[3] != 0 {
        Some(build_mass_table_i64(m[3]))
    } else {
        None
    };

    for i in (0..VAL_DIM).step_by(4) {
        for offset in 0..4 {
            let idx = i + offset;
            let mut acc = 0i64;
            if let Some(ref t) = t0 {
                acc = acc.wrapping_add(mul_mass_coord(t, v0[idx]));
            }
            if let Some(ref t) = t1 {
                acc = acc.wrapping_add(mul_mass_coord(t, v1[idx]));
            }
            if let Some(ref t) = t2 {
                acc = acc.wrapping_add(mul_mass_coord(t, v2[idx]));
            }
            if let Some(ref t) = t3 {
                acc = acc.wrapping_add(mul_mass_coord(t, v3[idx]));
            }
            sum_coords[idx] = sum_coords[idx].wrapping_add(acc);
        }
    }
}

/// 1-slot value accumulation unrolled 4x across VAL_DIM=256 coordinates.
/// Strictly 0 hardware multipliers, dividers, or floats in ARM64 disassembly.
#[inline(always)]
fn accumulate_slot_value_1x(sum_coords: &mut [i64; VAL_DIM], mass: u64, val: &[i32; VAL_DIM]) {
    if mass == 0 {
        return;
    }
    let t = build_mass_table_i64(mass);
    for i in (0..VAL_DIM).step_by(4) {
        sum_coords[i] = sum_coords[i].wrapping_add(mul_mass_coord(&t, val[i]));
        sum_coords[i + 1] = sum_coords[i + 1].wrapping_add(mul_mass_coord(&t, val[i + 1]));
        sum_coords[i + 2] = sum_coords[i + 2].wrapping_add(mul_mass_coord(&t, val[i + 2]));
        sum_coords[i + 3] = sum_coords[i + 3].wrapping_add(mul_mass_coord(&t, val[i + 3]));
    }
}

/// Wide values retain the same Q14 range and total Q48 attention mass as the
/// retained path. Each coordinate's magnitude is at most 2^62; increasing the
/// number of coordinates does not increase a coordinate's accumulation bound.
#[inline(never)]
fn accumulate_slot_value_576(sum_coords: &mut [i64; 576], mass: u64, val: &[i32; 576]) {
    if mass == 0 {
        return;
    }
    let t = build_mass_table_i64(mass);
    for i in (0..576).step_by(4) {
        sum_coords[i] = sum_coords[i].wrapping_add(mul_mass_coord(&t, val[i]));
        sum_coords[i + 1] = sum_coords[i + 1].wrapping_add(mul_mass_coord(&t, val[i + 1]));
        sum_coords[i + 2] = sum_coords[i + 2].wrapping_add(mul_mass_coord(&t, val[i + 2]));
        sum_coords[i + 3] = sum_coords[i + 3].wrapping_add(mul_mass_coord(&t, val[i + 3]));
    }
}

impl IntegerModel {
    pub fn load_with_tables(directory: &Path, tables: &Path) -> Result<Self> {
        Self::load_with_tables_profile(directory, tables, ServingProfile::Retained)
    }

    /// Load an explicitly requested serving profile. Shape alone never opts an
    /// artifact into the wider runtime or changes the canonical import scope.
    pub fn load_with_tables_profile(
        directory: &Path,
        tables: &Path,
        profile: ServingProfile,
    ) -> Result<Self> {
        let manifest_path = directory.join("hard-model.json");
        if fs::metadata(&manifest_path)?.len() > 8 * 1024 * 1024 {
            return Err(invalid("integer model manifest too large"));
        }
        let manifest: Value = serde_json::from_slice(&fs::read(&manifest_path)?)?;
        let stored_profile: ServingProfile = manifest
            .get("serving_profile")
            .cloned()
            .map(serde_json::from_value)
            .transpose()?
            .unwrap_or_default();
        if stored_profile != profile {
            return Err(invalid(
                "integer serving profile differs from requested profile",
            ));
        }
        // The read geometry selects the contract: the retained legacy contract
        // for the dot read, with the quantized Lorentz declaration otherwise.
        let config: JointConfig = serde_json::from_value(manifest["model"].clone())?;
        config.validate_for_profile(profile)?;
        if config.read_geometry == ReadGeometry::LorentzAffine {
            return Err(crate::IntegerError::UnsupportedReadGeometry(
                config.read_geometry,
            ));
        }
        if manifest["schema"] != "uor-r4.joint-recurrent-packed-emulator/1"
            || manifest
                .get("admission")
                .is_some_and(|value| value != "full")
            || (profile == ServingProfile::Dialogue576 && manifest["admission"] != "full")
            || manifest["numerical_contract"]
                != crate::config::packed_numerical_contract_for_profile(
                    config.read_geometry,
                    profile,
                )?
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
        // Profile validation above bounds every shape and keeps read width64.
        if config.read_width != 64 {
            return Err(invalid("integer bridge fixes read64"));
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
        let parameters: BTreeMap<String, Parameter> = codes
            .into_iter()
            .map(|(name, codes)| {
                let parameter_spec = spec.parameters[&name].clone();
                let parameter = Parameter {
                    codes: CoefficientCodes::from_decoded(codes, parameter_spec.bits)?,
                    spec: parameter_spec,
                };
                Ok((name, parameter))
            })
            .collect::<Result<_>>()?;
        let output_bias = match parameters.get("output.bias") {
            Some(p) => p
                .codes
                .iter()
                .map(|v| {
                    scaled(
                        i128::from(v),
                        WORK_BITS + i32::from(p.spec.row_exponents[0]),
                    )
                })
                .collect::<Result<Vec<_>>>()?,
            None => Vec::new(),
        };
        let output_shifts = match parameters.get("embedding.weight") {
            Some(emb) => emb
                .spec
                .row_exponents
                .iter()
                .map(|&exp| WORK_BITS - 10 + i32::from(exp))
                .collect(),
            None => Vec::new(),
        };
        let recurrent_bias = match parameters.get("recurrent.bias") {
            Some(p) => p
                .codes
                .iter()
                .map(|v| {
                    scaled(
                        i128::from(v),
                        WORK_BITS + i32::from(p.spec.row_exponents[0]),
                    )
                })
                .collect::<Result<Vec<_>>>()?,
            None => Vec::new(),
        };
        let read_age = match parameters.get("read.age") {
            Some(p) => p
                .codes
                .iter()
                .map(|v| {
                    scaled(
                        i128::from(v),
                        WORK_BITS + i32::from(p.spec.row_exponents[0]),
                    )
                })
                .collect::<Result<Vec<_>>>()?,
            None => Vec::new(),
        };
        let cache_vec = |name: &str| -> Result<Vec<i128>> {
            match parameters.get(name) {
                Some(p) => p
                    .codes
                    .iter()
                    .map(|v| {
                        scaled(
                            i128::from(v),
                            WORK_BITS + i32::from(p.spec.row_exponents[0]),
                        )
                    })
                    .collect::<Result<Vec<_>>>(),
                None => Ok(Vec::new()),
            }
        };
        let update_bias = cache_vec("update.bias")?;
        let update_gate_bias = cache_vec("update.gate.bias")?;
        let copy_gate_bias = cache_vec("copy.gate.bias")?;
        let read_key_bias = cache_vec("read.key.bias")?;
        let read_value_bias = cache_vec("read.value.bias")?;
        let read_query_bias = cache_vec("read.query.bias")?;
        let read_no_read_bias = cache_vec("read.no_read.bias")?;

        let get_param = |name: &str| -> Parameter {
            parameters.get(name).cloned().unwrap_or_else(|| Parameter {
                codes: CoefficientCodes::ones(0, 4),
                spec: ParameterQuantization {
                    bits: 4,
                    shape: Vec::new(),
                    row_exponents: Vec::new(),
                },
            })
        };
        let p_embedding = get_param("embedding.weight");
        let p_recurrent_input = get_param("recurrent.input.weight");
        let p_recurrent_state = get_param("recurrent.state.weight");
        let p_read_query = get_param("read.query.weight");
        let p_read_no_read = get_param("read.no_read.weight");
        let p_update = get_param("update.weight");
        let p_update_gate = get_param("update.gate.weight");
        let p_copy_gate = get_param("copy.gate.weight");
        let p_read_key = get_param("read.key.weight");
        let p_read_value = get_param("read.value.weight");
        let p_output_norm = get_param("output.norm.weight");

        let tables = Tables::load(tables)?;
        let mut identity = format!("{}:{}", crate::sha256_file(&manifest_path)?, tables.sha256);
        let lorentz = match config.read_geometry {
            ReadGeometry::Dot => None,
            ReadGeometry::LorentzAffine => {
                return Err(crate::IntegerError::UnsupportedReadGeometry(
                    config.read_geometry,
                ));
            }
            ReadGeometry::Lorentz => {
                let Some(arcosh) = &tables.arcosh_sha256 else {
                    return Err(invalid(
                        "a Lorentz read requires a table root exported with the sealed arcosh table",
                    ));
                };
                identity = format!("{identity}:{arcosh}");
                let scalar = |name: &str| -> Result<(i16, i16)> {
                    let p = parameters
                        .get(name)
                        .ok_or_else(|| invalid(format!("missing integer parameter {name}")))?;
                    Ok((p.codes.code(0)?, p.spec.row_exponents[0]))
                };
                Some(LorentzRead::new(
                    scalar(LORENTZ_LOG_BETA)?,
                    scalar(LORENTZ_OFFSET)?,
                )?)
            }
        };
        Ok(Self {
            config,
            serving_profile: profile,
            parameters,
            tables,
            lorentz,
            identity,
            output_bias,
            output_shifts,
            recurrent_bias,
            read_age,
            update_bias,
            update_gate_bias,
            copy_gate_bias,
            read_key_bias,
            read_value_bias,
            read_query_bias,
            read_no_read_bias,
            p_embedding,
            p_recurrent_input,
            p_recurrent_state,
            p_read_query,
            p_read_no_read,
            p_update,
            p_update_gate,
            p_copy_gate,
            p_read_key,
            p_read_value,
            p_output_norm,
        })
    }

    pub fn config(&self) -> &JointConfig {
        &self.config
    }

    pub fn serving_profile(&self) -> ServingProfile {
        self.serving_profile
    }

    /// Count each parameter's shared code allocation once, including arrays
    /// also referenced by cached hot-parameter fields.
    pub fn coefficient_storage(&self) -> CoefficientStorage {
        let mut result = CoefficientStorage::default();
        for parameter in self.parameters.values() {
            if parameter.codes.is_signed4() {
                result.signed4_coefficients += parameter.codes.len();
                result.packed_signed4_bytes += parameter.codes.payload_bytes();
            } else {
                result.signed16_coefficients += parameter.codes.len();
                result.signed16_bytes += parameter.codes.payload_bytes();
            }
        }
        result
    }

    pub fn new_session(&self) -> IntegerSession {
        IntegerSession {
            identity: self.identity.clone(),
            state: vec![0; self.config.width],
            keys: Vec::with_capacity(self.config.context),
            key_norms: Vec::with_capacity(self.config.context),
            values: match self.serving_profile {
                ServingProfile::Retained => {
                    SessionValues::Retained(Vec::with_capacity(self.config.context))
                }
                ServingProfile::Dialogue576 => {
                    SessionValues::Dialogue576(Vec::with_capacity(self.config.context))
                }
            },
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
        let l2_pages = vec![L2PrimePage::default(); L2_PAGE_CAPACITY]
            .into_boxed_slice()
            .try_into()
            .unwrap_or_else(|_| panic!("l2_pages size mismatch"));
        let last_probabilities = vec![0u64; 4096]
            .into_boxed_slice()
            .try_into()
            .unwrap_or_else(|_| panic!("probabilities size mismatch"));
        let last_read_masses = Vec::with_capacity(TOTAL_MEMORY_CANDIDATES);
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
            l2_pages,
            l2_cursor: 0,
            l2_len: 0,
            l2_seen: 0,
            last_compressed_turn_id: u32::MAX,
            zeta_state: T8ZetaState::new(),
            hopf_state: HopfFiberPointQ30::default(),
            cumulative_holonomy_q30: 0,
            age_horizon_clamp: AGE_HORIZON_CLAMP,
            scratch_products: vec![[0i64; 16]; 512],
            copy_scratch: vec![0u64; 4096],
            last_probabilities,
            last_read_masses,
            last_no_read_mass: 0,
            last_copy_gate: 0,
            has_step: false,
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
            576 => (token_index << 9) + (token_index << 6),
            _ => return Err(invalid("unsupported embedding width")),
        };
        let end = start + width;
        (start..end)
            .map(|index| embedding.codes.code(index).map(i32::from))
            .collect()
    }

    /// Project normalized hidden state to vocabulary logits using pure signed-4
    /// shift-and-add dot products over embedding.weight.
    /// Project normalized hidden state to vocabulary logits using pure signed-4
    /// shift-and-add dot products over embedding.weight with precomputed products.
    /// Row stride uses exact power-of-two shift (`<< 8` for width 256).
    /// Strictly 0 hardware multipliers, 0 floating point.
    #[inline(never)]
    pub fn project_vocab_with_products_into(
        &self,
        products: &[[i64; 16]],
        logits: &mut [i32],
    ) -> Result<()> {
        let embedding = &self.p_embedding;
        let fallback_bias;
        let output_bias: &[i128] = if !self.output_bias.is_empty() {
            &self.output_bias
        } else {
            fallback_bias = self.vector_work("output.bias")?;
            &fallback_bias
        };
        let fallback_shifts;
        let output_shifts: &[i32] = if !self.output_shifts.is_empty() {
            &self.output_shifts
        } else {
            fallback_shifts = embedding
                .spec
                .row_exponents
                .iter()
                .map(|&exp| WORK_BITS - 10 + i32::from(exp))
                .collect::<Vec<_>>();
            &fallback_shifts
        };

        let width = self.config.width;
        let vocab_size = self.config.vocab_size;
        if logits.len() < vocab_size || products.len() < width {
            return Err(invalid("logits or product buffer too small"));
        }
        if width == 256 {
            let p_256: &[[i64; 16]; 256] = products[..256]
                .try_into()
                .map_err(|_| invalid("products width must be 256"))?;
            let chunks_8_count = vocab_size / 8;
            for c in 0..chunks_8_count {
                let r0 = c * 8;
                let s0 = r0 << 8;
                let chunk_arr: &[u8; 1024] =
                    match embedding.codes.packed_range(s0, 2048)?.try_into() {
                        Ok(arr) => arr,
                        Err(_) => continue,
                    };

                let (dot0, dot1, dot2, dot3, dot4, dot5, dot6, dot7) =
                    low_bit_dot_8_contiguous(p_256, chunk_arr);

                logits[r0] = scale_and_quantize_logit(dot0, output_shifts[r0], output_bias[r0]);
                logits[r0 + 1] =
                    scale_and_quantize_logit(dot1, output_shifts[r0 + 1], output_bias[r0 + 1]);
                logits[r0 + 2] =
                    scale_and_quantize_logit(dot2, output_shifts[r0 + 2], output_bias[r0 + 2]);
                logits[r0 + 3] =
                    scale_and_quantize_logit(dot3, output_shifts[r0 + 3], output_bias[r0 + 3]);
                logits[r0 + 4] =
                    scale_and_quantize_logit(dot4, output_shifts[r0 + 4], output_bias[r0 + 4]);
                logits[r0 + 5] =
                    scale_and_quantize_logit(dot5, output_shifts[r0 + 5], output_bias[r0 + 5]);
                logits[r0 + 6] =
                    scale_and_quantize_logit(dot6, output_shifts[r0 + 6], output_bias[r0 + 6]);
                logits[r0 + 7] =
                    scale_and_quantize_logit(dot7, output_shifts[r0 + 7], output_bias[r0 + 7]);
            }
            for row_idx in (chunks_8_count * 8)..vocab_size {
                let start = row_idx << 8;
                let end = start + 256;
                let row = embedding.codes.packed_range(start, end - start)?;
                let dot = low_bit_dot(&products[..width], row);
                logits[row_idx] =
                    scale_and_quantize_logit(dot, output_shifts[row_idx], output_bias[row_idx]);
            }
        } else if width == 128 {
            for (row_idx, &bias) in output_bias.iter().enumerate().take(vocab_size) {
                let start = row_idx << 7;
                let end = start + 128;
                let row = embedding.codes.packed_range(start, end - start)?;
                let dot = low_bit_dot(&products[..width], row);
                let scaled_val = scaled(i128::from(dot), output_shifts[row_idx])?;
                logits[row_idx] = quantize(scaled_val + bias, WORK_BITS, 8)?;
            }
        } else if width == 576 {
            for (row_idx, &bias) in output_bias.iter().enumerate().take(vocab_size) {
                let start = (row_idx << 9) + (row_idx << 6);
                let row = embedding.codes.packed_range(start, 576)?;
                let dot = low_bit_dot(&products[..576], row);
                let scaled_val = scaled(i128::from(dot), output_shifts[row_idx])?;
                logits[row_idx] = quantize(scaled_val + bias, WORK_BITS, 8)?;
            }
        } else {
            return Err(invalid("unsupported width for project_vocab"));
        }
        Ok(())
    }

    #[inline(never)]
    pub fn project_vocab_with_products(&self, products: &[[i64; 16]]) -> Result<Vec<i32>> {
        let mut logits = vec![0i32; self.config.vocab_size];
        self.project_vocab_with_products_into(products, &mut logits)?;
        Ok(logits)
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
        let products = low_bit_products(hidden);
        self.project_vocab_with_products(&products)
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
            .map(|v| {
                scaled(
                    i128::from(v),
                    WORK_BITS + i32::from(p.spec.row_exponents[0]),
                )
            })
            .collect()
    }

    fn matrix_work(&self, input: &[i32], input_exponent: i32, name: &str) -> Result<Vec<i128>> {
        let p = self.parameter(name)?;
        let mut products = vec![[0i64; 16]; input.len()];
        fill_low_bit_products(input, &mut products);
        self.matrix_work_direct(&products, p, input.len(), input_exponent)
    }

    /// Return Q40 affine sums using precomputed low_bit products.
    #[allow(dead_code)]
    fn matrix_work_with_products(
        &self,
        products: &[[i64; 16]],
        input_len: usize,
        input_exponent: i32,
        name: &str,
    ) -> Result<Vec<i128>> {
        let p = self.parameter(name)?;
        self.matrix_work_direct(products, p, input_len, input_exponent)
    }

    fn matrix_work_direct_into(
        &self,
        products: &[[i64; 16]],
        p: &Parameter,
        input_len: usize,
        input_exponent: i32,
        out: &mut [i128],
    ) -> Result<()> {
        if p.spec.bits != 4
            || p.spec.shape.len() != 2
            || p.spec.shape[1] != input_len
            || products.len() < input_len
        {
            return Err(invalid("integer affine input shape or bit width"));
        }
        let num_rows = p.spec.row_exponents.len();
        if out.len() < num_rows {
            return Err(invalid("output buffer too small for matrix_work_direct"));
        }
        match input_len {
            256 => {
                let p_256: &[[i64; 16]; 256] = products[..256]
                    .try_into()
                    .map_err(|_| invalid("products width must be 256"))?;
                let chunks_8 = num_rows / 8;
                for c in 0..chunks_8 {
                    let r0 = c * 8;
                    let s0 = r0 << 8;
                    let chunk: &[u8; 1024] = p
                        .codes
                        .packed_range(s0, 2048)?
                        .try_into()
                        .map_err(|_| invalid("parameter code slice"))?;

                    let (d0, d1, d2, d3, d4, d5, d6, d7) = low_bit_dot_8_contiguous(p_256, chunk);

                    out[r0] = scaled(
                        i128::from(d0),
                        WORK_BITS + input_exponent + i32::from(p.spec.row_exponents[r0]),
                    )?;
                    out[r0 + 1] = scaled(
                        i128::from(d1),
                        WORK_BITS + input_exponent + i32::from(p.spec.row_exponents[r0 + 1]),
                    )?;
                    out[r0 + 2] = scaled(
                        i128::from(d2),
                        WORK_BITS + input_exponent + i32::from(p.spec.row_exponents[r0 + 2]),
                    )?;
                    out[r0 + 3] = scaled(
                        i128::from(d3),
                        WORK_BITS + input_exponent + i32::from(p.spec.row_exponents[r0 + 3]),
                    )?;
                    out[r0 + 4] = scaled(
                        i128::from(d4),
                        WORK_BITS + input_exponent + i32::from(p.spec.row_exponents[r0 + 4]),
                    )?;
                    out[r0 + 5] = scaled(
                        i128::from(d5),
                        WORK_BITS + input_exponent + i32::from(p.spec.row_exponents[r0 + 5]),
                    )?;
                    out[r0 + 6] = scaled(
                        i128::from(d6),
                        WORK_BITS + input_exponent + i32::from(p.spec.row_exponents[r0 + 6]),
                    )?;
                    out[r0 + 7] = scaled(
                        i128::from(d7),
                        WORK_BITS + input_exponent + i32::from(p.spec.row_exponents[r0 + 7]),
                    )?;
                }
                for (r, out_r) in out.iter_mut().enumerate().take(num_rows).skip(chunks_8 * 8) {
                    let s = r << 8;
                    let row = p.codes.packed_range(s, 256)?;
                    let d = low_bit_dot(&products[..input_len], row);
                    *out_r = scaled(
                        i128::from(d),
                        WORK_BITS + input_exponent + i32::from(p.spec.row_exponents[r]),
                    )?;
                }
                Ok(())
            }
            512 => {
                let p_512: &[[i64; 16]; 512] = products[..512]
                    .try_into()
                    .map_err(|_| invalid("products width must be 512"))?;
                let chunks_4 = num_rows / 4;
                for c in 0..chunks_4 {
                    let r0 = c * 4;
                    let r1 = r0 + 1;
                    let r2 = r0 + 2;
                    let r3 = r0 + 3;

                    let s0 = r0 << 9;
                    let chunk: &[u8; 1024] = p
                        .codes
                        .packed_range(s0, 2048)?
                        .try_into()
                        .map_err(|_| invalid("parameter code slice"))?;

                    let (d0, d1, d2, d3) = low_bit_dot_4_contiguous_512(p_512, chunk);

                    out[r0] = scaled(
                        i128::from(d0),
                        WORK_BITS + input_exponent + i32::from(p.spec.row_exponents[r0]),
                    )?;
                    out[r1] = scaled(
                        i128::from(d1),
                        WORK_BITS + input_exponent + i32::from(p.spec.row_exponents[r1]),
                    )?;
                    out[r2] = scaled(
                        i128::from(d2),
                        WORK_BITS + input_exponent + i32::from(p.spec.row_exponents[r2]),
                    )?;
                    out[r3] = scaled(
                        i128::from(d3),
                        WORK_BITS + input_exponent + i32::from(p.spec.row_exponents[r3]),
                    )?;
                }
                for (r, out_r) in out.iter_mut().enumerate().take(num_rows).skip(chunks_4 * 4) {
                    let s = r << 9;
                    let row = p.codes.packed_range(s, 512)?;
                    let d = low_bit_dot(&products[..input_len], row);
                    *out_r = scaled(
                        i128::from(d),
                        WORK_BITS + input_exponent + i32::from(p.spec.row_exponents[r]),
                    )?;
                }
                Ok(())
            }
            128 => {
                for (r, &exponent) in p.spec.row_exponents.iter().enumerate() {
                    let row = p.codes.packed_range(r << 7, 128)?;
                    out[r] = scaled(
                        i128::from(low_bit_dot(&products[..128], row)),
                        WORK_BITS + input_exponent + i32::from(exponent),
                    )?;
                }
                Ok(())
            }
            576 | 1152 => {
                // Both non-power-of-two strides use a monotonic address, so
                // row indexing does not request a variable hardware product.
                let mut start = 0;
                for (r, &exponent) in p.spec.row_exponents.iter().enumerate() {
                    let row = p.codes.packed_range(start, input_len)?;
                    out[r] = scaled(
                        i128::from(low_bit_dot(&products[..input_len], row)),
                        WORK_BITS + input_exponent + i32::from(exponent),
                    )?;
                    start += input_len;
                }
                Ok(())
            }
            other => Err(invalid(format!("unsupported matrix input width {other}"))),
        }
    }

    fn matrix_work_direct(
        &self,
        products: &[[i64; 16]],
        p: &Parameter,
        input_len: usize,
        input_exponent: i32,
    ) -> Result<Vec<i128>> {
        let mut results = vec![0i128; p.spec.row_exponents.len()];
        self.matrix_work_direct_into(products, p, input_len, input_exponent, &mut results)?;
        Ok(results)
    }

    fn affine_direct_into(
        &self,
        products: &[[i64; 16]],
        p: &Parameter,
        bias: &[i128],
        input_len: usize,
        input_bits: i32,
        out: &mut [i32],
    ) -> Result<()> {
        if p.spec.bits != 4
            || p.spec.shape.len() != 2
            || p.spec.shape[1] != input_len
            || products.len() < input_len
        {
            return Err(invalid("integer affine input shape or bit width"));
        }
        let num_rows = p.spec.row_exponents.len();
        if out.len() < num_rows {
            return Err(invalid("output buffer too small for affine"));
        }
        let input_exponent = -input_bits;
        match input_len {
            256 => {
                let p_256: &[[i64; 16]; 256] = products[..256]
                    .try_into()
                    .map_err(|_| invalid("products width must be 256"))?;
                let chunks_8 = num_rows / 8;
                for c in 0..chunks_8 {
                    let r0 = c * 8;
                    let s0 = r0 << 8;
                    let chunk: &[u8; 1024] = p
                        .codes
                        .packed_range(s0, 2048)?
                        .try_into()
                        .map_err(|_| invalid("parameter code slice"))?;

                    let (d0, d1, d2, d3, d4, d5, d6, d7) = low_bit_dot_8_contiguous(p_256, chunk);

                    let dots = [d0, d1, d2, d3, d4, d5, d6, d7];
                    for (k, &dot) in dots.iter().enumerate() {
                        let r = r0 + k;
                        let s = WORK_BITS + input_exponent + i32::from(p.spec.row_exponents[r]);
                        let b = if bias.is_empty() { 0 } else { bias[r] };
                        out[r] = scale_and_quantize_logit(dot, s, b);
                    }
                }
                for (r, out_r) in out.iter_mut().enumerate().take(num_rows).skip(chunks_8 * 8) {
                    let s_row = r << 8;
                    let row = p.codes.packed_range(s_row, 256)?;
                    let d = low_bit_dot(&products[..input_len], row);
                    let shift = WORK_BITS + input_exponent + i32::from(p.spec.row_exponents[r]);
                    let b = if bias.is_empty() { 0 } else { bias[r] };
                    *out_r = scale_and_quantize_logit(d, shift, b);
                }
                Ok(())
            }
            512 => {
                let p_512: &[[i64; 16]; 512] = products[..512]
                    .try_into()
                    .map_err(|_| invalid("products width must be 512"))?;
                let chunks_4 = num_rows / 4;
                for c in 0..chunks_4 {
                    let r0 = c * 4;
                    let r1 = r0 + 1;
                    let r2 = r0 + 2;
                    let r3 = r0 + 3;

                    let s0 = r0 << 9;
                    let chunk: &[u8; 1024] = p
                        .codes
                        .packed_range(s0, 2048)?
                        .try_into()
                        .map_err(|_| invalid("parameter code slice"))?;

                    let (d0, d1, d2, d3) = low_bit_dot_4_contiguous_512(p_512, chunk);

                    let shift0 = WORK_BITS + input_exponent + i32::from(p.spec.row_exponents[r0]);
                    let shift1 = WORK_BITS + input_exponent + i32::from(p.spec.row_exponents[r1]);
                    let shift2 = WORK_BITS + input_exponent + i32::from(p.spec.row_exponents[r2]);
                    let shift3 = WORK_BITS + input_exponent + i32::from(p.spec.row_exponents[r3]);

                    let b0 = if bias.is_empty() { 0 } else { bias[r0] };
                    let b1 = if bias.is_empty() { 0 } else { bias[r1] };
                    let b2 = if bias.is_empty() { 0 } else { bias[r2] };
                    let b3 = if bias.is_empty() { 0 } else { bias[r3] };

                    out[r0] = scale_and_quantize_logit(d0, shift0, b0);
                    out[r1] = scale_and_quantize_logit(d1, shift1, b1);
                    out[r2] = scale_and_quantize_logit(d2, shift2, b2);
                    out[r3] = scale_and_quantize_logit(d3, shift3, b3);
                }
                for r in (chunks_4 * 4)..num_rows {
                    let s_row = r << 9;
                    let row = p.codes.packed_range(s_row, 512)?;
                    let d = low_bit_dot(&products[..input_len], row);
                    let shift = WORK_BITS + input_exponent + i32::from(p.spec.row_exponents[r]);
                    let b = if bias.is_empty() { 0 } else { bias[r] };
                    out[r] = scale_and_quantize_logit(d, shift, b);
                }
                Ok(())
            }
            _ => {
                let values = self.matrix_work_direct(products, p, input_len, input_exponent)?;
                if bias.is_empty() {
                    for (i, v) in values.into_iter().enumerate() {
                        out[i] = quantize(v, WORK_BITS, 8)?;
                    }
                } else {
                    for (i, (v, &b)) in values.into_iter().zip(bias).enumerate() {
                        out[i] = quantize(v + b, WORK_BITS, 8)?;
                    }
                }
                Ok(())
            }
        }
    }

    #[allow(dead_code)]
    fn affine_direct(
        &self,
        products: &[[i64; 16]],
        p: &Parameter,
        bias: &[i128],
        input_len: usize,
        input_bits: i32,
    ) -> Result<Vec<i32>> {
        let mut out = vec![0i32; p.spec.row_exponents.len()];
        self.affine_direct_into(products, p, bias, input_len, input_bits, &mut out)?;
        Ok(out)
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

    fn arcosh(&self) -> Result<&[u32]> {
        self.tables
            .arcosh
            .as_deref()
            .ok_or_else(|| invalid("missing arcosh table"))
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
        let fallback_bias;
        let bias: &[i128] = if !self.recurrent_bias.is_empty() {
            &self.recurrent_bias
        } else {
            fallback_bias = self.vector_work("recurrent.bias")?;
            &fallback_bias
        };
        let fused = token_affine
            .into_iter()
            .zip(recurrent)
            .zip(bias)
            .map(|((a, b), &c)| quantize(a + b + c, WORK_BITS, 8))
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
            let fallback_age;
            let age: &[i128] = if !self.read_age.is_empty() {
                &self.read_age
            } else {
                fallback_age = self.vector_work("read.age")?;
                &fallback_age
            };
            let hyperbolic = match &self.lorentz {
                Some(read) => Some((read, lorentz::squared_norm(&query)?, self.arcosh()?)),
                None => None,
            };
            let mut scores = Vec::with_capacity(previous + 1);
            scores.push(null);

            if let Some((read, query_norm, table)) = hyperbolic {
                for (index, key) in session.keys.iter().enumerate() {
                    let mut dot = 0i128;
                    for (&q, &k) in query.iter().zip(key) {
                        dot += product(i128::from(q), i128::from(k))?;
                    }
                    let key_norm = *session
                        .key_norms
                        .get(index)
                        .ok_or_else(|| invalid("missing Lorentz key norm"))?;
                    let raw = read.score(query_norm, key_norm, dot, table, WORK_BITS)?;
                    let score = raw + age[previous - 1 - index];
                    scores.push(quantize(score, WORK_BITS, 8)?);
                }
            } else {
                let mut query_products = [[0i64; 16]; KEY_DIM];
                let q_len = query.len().min(KEY_DIM);
                for d in 0..q_len {
                    query_products[d] = build_coord_products(query[d]);
                }

                let n_chunks = session.keys.len() >> 2;
                let mut key_idx = 0;
                for c in 0..n_chunks {
                    let base = c << 2;
                    let (d0, d1, d2, d3) = memory_dot_product_4x(
                        &query_products,
                        &session.keys[base],
                        &session.keys[base + 1],
                        &session.keys[base + 2],
                        &session.keys[base + 3],
                    );
                    let s0 = scaled(i128::from(d0), WORK_BITS - 19)? + age[previous - 1 - key_idx];
                    scores.push(quantize(s0, WORK_BITS, 8)?);
                    let s1 =
                        scaled(i128::from(d1), WORK_BITS - 19)? + age[previous - 1 - (key_idx + 1)];
                    scores.push(quantize(s1, WORK_BITS, 8)?);
                    let s2 =
                        scaled(i128::from(d2), WORK_BITS - 19)? + age[previous - 1 - (key_idx + 2)];
                    scores.push(quantize(s2, WORK_BITS, 8)?);
                    let s3 =
                        scaled(i128::from(d3), WORK_BITS - 19)? + age[previous - 1 - (key_idx + 3)];
                    scores.push(quantize(s3, WORK_BITS, 8)?);
                    key_idx += 4;
                }
                for key in &session.keys[(n_chunks << 2)..] {
                    let d = memory_dot_product_1x(&query_products, key);
                    let s = scaled(i128::from(d), WORK_BITS - 19)? + age[previous - 1 - key_idx];
                    scores.push(quantize(s, WORK_BITS, 8)?);
                    key_idx += 1;
                }
            }

            let masses = softmax(&scores, &self.tables)?;
            let read = session.values.read(&masses[1..], width)?;
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
        let key_norm = match self.lorentz {
            Some(_) => Some(lorentz::squared_norm(&key)?),
            None => None,
        };
        let value = self.tanh(&self.affine(&write_normalized, 10, "read.value")?);
        let norm = self.parameter("output.norm.weight")?;
        let hidden = write_normalized
            .iter()
            .zip(norm.codes.iter())
            .map(|(&x, w)| {
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
        let zero_uniform = u64::try_from(divide(i128::from(TOTAL >> 12), 100_000_000)?)
            .map_err(|_| invalid("integer mixture range"))?;
        let mut probabilities = Vec::with_capacity(self.config.vocab_size);
        for (v, c) in vocabulary.into_iter().zip(copy) {
            if v == 0 && c == 0 {
                probabilities.push(zero_uniform);
                continue;
            }
            let copy_term = if c != 0 {
                scaled(product(i128::from(c), i128::from(gate))?, -15)?
            } else {
                0
            };
            let raw = scaled(product(i128::from(v), fraction)?, -48)? + copy_term;
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
        // The value shape check precedes all mutation; after its successful
        // push, the remaining commits below are infallible.
        session.values.push(&value)?;
        session.state = state.clone();
        let mut key_arr = [0i32; KEY_DIM];
        let key_len = key.len().min(KEY_DIM);
        key_arr[..key_len].copy_from_slice(&key[..key_len]);
        session.keys.push(key_arr);
        session.key_norms.extend(key_norm);
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
    pub fn step_conversational_into(
        &self,
        session: &mut SessionState,
        token: u32,
        target: SlotTarget,
        mode: ReadMode,
    ) -> Result<()> {
        if session.identity != self.identity {
            return Err(invalid("conversational step identity mismatch"));
        }
        // Partitioned conversational memory stores full-width values and scores
        // keys by the dot read; a Lorentz or width-128 model uses `step`.
        if self.lorentz.is_some() || self.config.width != VAL_DIM {
            return Err(invalid(
                "conversational sessions serve only the width-256 dot-read shape",
            ));
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
        let mut scratch_products = [[0i64; 16]; 512];

        let token_index = token as usize;
        let start = match width {
            256 => token_index << 8,
            128 => token_index << 7,
            _ => return Err(invalid("unsupported embedding width")),
        };
        let end = start + width;
        fill_packed_low_bit_products(
            self.p_embedding.codes.packed_range(start, end - start)?,
            &mut scratch_products[..width],
        );

        let mut token_affine = [0i128; 768];
        self.matrix_work_direct_into(
            &scratch_products[..width],
            &self.p_recurrent_input,
            width,
            i32::from(self.p_embedding.spec.row_exponents[token_index]),
            &mut token_affine[..3 * width],
        )?;

        let mut previous_normalized = [0i32; 256];
        normalize_state_into(&session.state, &mut previous_normalized)?;
        fill_low_bit_products(
            &previous_normalized[..width],
            &mut scratch_products[..width],
        );

        let mut recurrent = [0i128; 768];
        self.matrix_work_direct_into(
            &scratch_products[..width],
            &self.p_recurrent_state,
            width,
            -10,
            &mut recurrent[..3 * width],
        )?;

        let fallback_bias;
        let bias: &[i128] = if !self.recurrent_bias.is_empty() {
            &self.recurrent_bias
        } else {
            fallback_bias = self.vector_work("recurrent.bias")?;
            &fallback_bias
        };

        let mut fused = [0i32; 768];
        for i in 0..3 * width {
            fused[i] = quantize(token_affine[i] + recurrent[i] + bias[i], WORK_BITS, 8)?;
        }

        let mut candidate = [0i32; 256];
        for i in 0..width {
            candidate[i] = self.tables.tanh[(fused[i] + 32767) as usize];
        }
        let mut z = [0i32; 256];
        for i in 0..width {
            z[i] = self.tables.sigmoid[(fused[width + i] + 32767) as usize];
        }

        let mut transported = [0i32; 256];
        transport_into(
            &session.state,
            &fused[width + width..3 * width],
            self.config.transport,
            &mut transported[..width],
        )?;

        let mut provisional = [0i32; 256];
        blend_into(
            &transported[..width],
            &candidate[..width],
            &z[..width],
            &mut provisional[..width],
        )?;

        let mut normalized = [0i32; 256];
        normalize_state_into(&provisional[..width], &mut normalized)?;

        let n_sys = session.persistent_keys.len();
        let n_dial = session.dialogue_len;
        let n_page = session.l2_len;
        let total_slots = n_sys + n_dial + n_page;

        let mut all_masses = [0u64; MAX_SCORES_CAPACITY];
        let mut read = [0i32; 256];
        let no_read = if total_slots == 0 || mode == ReadMode::NoRead {
            TOTAL
        } else {
            fill_low_bit_products(&normalized[..width], &mut scratch_products[..width]);

            let mut query = [0i32; KEY_DIM];
            self.affine_direct_into(
                &scratch_products[..width],
                &self.p_read_query,
                &self.read_query_bias,
                width,
                10,
                &mut query,
            )?;

            let mut null_buf = [0i32; 1];
            self.affine_direct_into(
                &scratch_products[..width],
                &self.p_read_no_read,
                &self.read_no_read_bias,
                width,
                10,
                &mut null_buf,
            )?;
            let null = null_buf[0];

            let fallback_age;
            let age: &[i128] = if !self.read_age.is_empty() {
                &self.read_age
            } else {
                fallback_age = self.vector_work("read.age")?;
                &fallback_age
            };

            // Precompute query coordinate product tables on stack (64 x 16 i64 = 8 KB).
            let mut query_products = [[0i64; 16]; KEY_DIM];
            for d in 0..KEY_DIM {
                query_products[d] = build_coord_products(query[d]);
            }

            let mut scores = [0i32; MAX_SCORES_CAPACITY];
            scores[0] = null;
            let mut score_count = 1;

            // A. Persistent session slots: zero age decay (fixed to age[0]), tiled by 4
            let n_sys_chunks = n_sys >> 2;
            for c in 0..n_sys_chunks {
                let base = c << 2;
                let (d0, d1, d2, d3) = memory_dot_product_4x(
                    &query_products,
                    &session.persistent_keys[base],
                    &session.persistent_keys[base + 1],
                    &session.persistent_keys[base + 2],
                    &session.persistent_keys[base + 3],
                );
                scores[score_count] = scale_and_quantize_score(d0, age[0]);
                scores[score_count + 1] = scale_and_quantize_score(d1, age[0]);
                scores[score_count + 2] = scale_and_quantize_score(d2, age[0]);
                scores[score_count + 3] = scale_and_quantize_score(d3, age[0]);
                score_count += 4;
            }
            for key in &session.persistent_keys[(n_sys_chunks << 2)..n_sys] {
                let d = memory_dot_product_1x(&query_products, key);
                scores[score_count] = scale_and_quantize_score(d, age[0]);
                score_count += 1;
            }

            // B. Dialogue slots: relative age with horizon clamp, tiled by 4 across active slots only
            let n_dial_chunks = n_dial >> 2;
            for c in 0..n_dial_chunks {
                let base = c << 2;
                let (d0, d1, d2, d3) = memory_dot_product_4x(
                    &query_products,
                    &session.dialogue_keys[base],
                    &session.dialogue_keys[base + 1],
                    &session.dialogue_keys[base + 2],
                    &session.dialogue_keys[base + 3],
                );

                let delta0 = session
                    .dialogue_seen
                    .saturating_sub(1 + session.dialogue_sequences[base]);
                let age_idx0 = (delta0 as usize)
                    .min(age.len().saturating_sub(1))
                    .min(session.age_horizon_clamp);
                scores[score_count] = scale_and_quantize_score(d0, age[age_idx0]);

                let delta1 = session
                    .dialogue_seen
                    .saturating_sub(1 + session.dialogue_sequences[base + 1]);
                let age_idx1 = (delta1 as usize)
                    .min(age.len().saturating_sub(1))
                    .min(session.age_horizon_clamp);
                scores[score_count + 1] = scale_and_quantize_score(d1, age[age_idx1]);

                let delta2 = session
                    .dialogue_seen
                    .saturating_sub(1 + session.dialogue_sequences[base + 2]);
                let age_idx2 = (delta2 as usize)
                    .min(age.len().saturating_sub(1))
                    .min(session.age_horizon_clamp);
                scores[score_count + 2] = scale_and_quantize_score(d2, age[age_idx2]);

                let delta3 = session
                    .dialogue_seen
                    .saturating_sub(1 + session.dialogue_sequences[base + 3]);
                let age_idx3 = (delta3 as usize)
                    .min(age.len().saturating_sub(1))
                    .min(session.age_horizon_clamp);
                scores[score_count + 3] = scale_and_quantize_score(d3, age[age_idx3]);
                score_count += 4;
            }

            for idx in (n_dial_chunks << 2)..n_dial {
                let d = memory_dot_product_1x(&query_products, &session.dialogue_keys[idx]);
                let delta = session
                    .dialogue_seen
                    .saturating_sub(1 + session.dialogue_sequences[idx]);
                let age_idx = (delta as usize)
                    .min(age.len().saturating_sub(1))
                    .min(session.age_horizon_clamp);
                scores[score_count] = scale_and_quantize_score(d, age[age_idx]);
                score_count += 1;
            }

            // C. L2 Prime Pages: relative age with horizon clamp, tiled by 4 across active pages only
            let n_page_chunks = n_page >> 2;
            for c in 0..n_page_chunks {
                let base = c << 2;
                let (d0, d1, d2, d3) = memory_dot_product_4x(
                    &query_products,
                    &session.l2_pages[base].key,
                    &session.l2_pages[base + 1].key,
                    &session.l2_pages[base + 2].key,
                    &session.l2_pages[base + 3].key,
                );

                let delta0 = session
                    .dialogue_seen
                    .saturating_sub(1 + session.l2_pages[base].end_seq as u64);
                let age_idx0 = (delta0 as usize)
                    .min(age.len().saturating_sub(1))
                    .min(session.age_horizon_clamp);
                scores[score_count] = scale_and_quantize_score(d0, age[age_idx0]);

                let delta1 = session
                    .dialogue_seen
                    .saturating_sub(1 + session.l2_pages[base + 1].end_seq as u64);
                let age_idx1 = (delta1 as usize)
                    .min(age.len().saturating_sub(1))
                    .min(session.age_horizon_clamp);
                scores[score_count + 1] = scale_and_quantize_score(d1, age[age_idx1]);

                let delta2 = session
                    .dialogue_seen
                    .saturating_sub(1 + session.l2_pages[base + 2].end_seq as u64);
                let age_idx2 = (delta2 as usize)
                    .min(age.len().saturating_sub(1))
                    .min(session.age_horizon_clamp);
                scores[score_count + 2] = scale_and_quantize_score(d2, age[age_idx2]);

                let delta3 = session
                    .dialogue_seen
                    .saturating_sub(1 + session.l2_pages[base + 3].end_seq as u64);
                let age_idx3 = (delta3 as usize)
                    .min(age.len().saturating_sub(1))
                    .min(session.age_horizon_clamp);
                scores[score_count + 3] = scale_and_quantize_score(d3, age[age_idx3]);
                score_count += 4;
            }

            for idx in (n_page_chunks << 2)..n_page {
                let d = memory_dot_product_1x(&query_products, &session.l2_pages[idx].key);
                let delta = session
                    .dialogue_seen
                    .saturating_sub(1 + session.l2_pages[idx].end_seq as u64);
                let age_idx = (delta as usize)
                    .min(age.len().saturating_sub(1))
                    .min(session.age_horizon_clamp);
                scores[score_count] = scale_and_quantize_score(d, age[age_idx]);
                score_count += 1;
            }

            all_masses.fill(0);
            softmax_into(
                &scores[..score_count],
                &self.tables,
                &mut all_masses[..score_count],
            )?;
            let no_read = all_masses[0];
            let m_sys = &all_masses[1..1 + n_sys];
            let m_dial = &all_masses[1 + n_sys..1 + n_sys + n_dial];
            let m_page = &all_masses[1 + n_sys + n_dial..score_count];

            let mut sum_coords = [0i64; VAL_DIM];
            let n_sys_vchunks = n_sys >> 2;
            for c in 0..n_sys_vchunks {
                let base = c << 2;
                let m_chunk: &[u64; 4] = m_sys[base..base + 4].try_into().unwrap();
                if (m_chunk[0] | m_chunk[1] | m_chunk[2] | m_chunk[3]) == 0 {
                    continue;
                }
                accumulate_slot_values_4x(
                    &mut sum_coords,
                    m_chunk,
                    &session.persistent_values[base],
                    &session.persistent_values[base + 1],
                    &session.persistent_values[base + 2],
                    &session.persistent_values[base + 3],
                );
            }
            let sys_rem_start = n_sys_vchunks << 2;
            for (&mass, val) in m_sys[sys_rem_start..n_sys]
                .iter()
                .zip(&session.persistent_values[sys_rem_start..n_sys])
            {
                accumulate_slot_value_1x(&mut sum_coords, mass, val);
            }

            let n_dial_vchunks = n_dial >> 2;
            for c in 0..n_dial_vchunks {
                let base = c << 2;
                let m_chunk: &[u64; 4] = m_dial[base..base + 4].try_into().unwrap();
                if (m_chunk[0] | m_chunk[1] | m_chunk[2] | m_chunk[3]) == 0 {
                    continue;
                }
                accumulate_slot_values_4x(
                    &mut sum_coords,
                    m_chunk,
                    &session.dialogue_values[base],
                    &session.dialogue_values[base + 1],
                    &session.dialogue_values[base + 2],
                    &session.dialogue_values[base + 3],
                );
            }
            let dial_rem_start = n_dial_vchunks << 2;
            for (&mass, val) in m_dial[dial_rem_start..n_dial]
                .iter()
                .zip(&session.dialogue_values[dial_rem_start..n_dial])
            {
                accumulate_slot_value_1x(&mut sum_coords, mass, val);
            }

            let n_page_vchunks = n_page >> 2;
            for c in 0..n_page_vchunks {
                let base = c << 2;
                let m_chunk: &[u64; 4] = m_page[base..base + 4].try_into().unwrap();
                if (m_chunk[0] | m_chunk[1] | m_chunk[2] | m_chunk[3]) == 0 {
                    continue;
                }
                accumulate_slot_values_4x(
                    &mut sum_coords,
                    m_chunk,
                    &session.l2_pages[base].value,
                    &session.l2_pages[base + 1].value,
                    &session.l2_pages[base + 2].value,
                    &session.l2_pages[base + 3].value,
                );
            }
            let page_rem_start = n_page_vchunks << 2;
            for (&mass, page) in m_page[page_rem_start..n_page]
                .iter()
                .zip(&session.l2_pages[page_rem_start..n_page])
            {
                accumulate_slot_value_1x(&mut sum_coords, mass, &page.value);
            }

            for (out, &acc) in read[..width].iter_mut().zip(&sum_coords[..width]) {
                *out = quantize(acc as i128, 48 + 14, STATE_BITS)?;
            }
            no_read
        };

        let mut concat_buf = [0i32; 512];
        concat_buf[..width].copy_from_slice(&provisional[..width]);
        concat_buf[width..width + width].copy_from_slice(&read[..width]);
        fill_low_bit_products(&concat_buf, &mut scratch_products[..512]);

        let mut update_raw = [0i32; 256];
        self.affine_direct_into(
            &scratch_products[..512],
            &self.p_update,
            &self.update_bias,
            512,
            STATE_BITS,
            &mut update_raw[..width],
        )?;
        let mut update = [0i32; 256];
        for i in 0..width {
            update[i] = self.tables.tanh[(update_raw[i] + 32767) as usize];
        }

        let mut update_gate_raw = [0i32; 1];
        self.affine_direct_into(
            &scratch_products[..512],
            &self.p_update_gate,
            &self.update_gate_bias,
            512,
            STATE_BITS,
            &mut update_gate_raw,
        )?;
        let rho = self.tables.sigmoid[(update_gate_raw[0] + 32767) as usize];

        let mut state = [0i32; 256];
        blend_scalar_into(
            &provisional[..width],
            &update[..width],
            rho,
            &mut state[..width],
        )?;

        fill_low_bit_products(&state[..width], &mut scratch_products[..width]);

        let mut copy_gate_raw = [0i32; 1];
        self.affine_direct_into(
            &scratch_products[..512],
            &self.p_copy_gate,
            &self.copy_gate_bias,
            512,
            STATE_BITS,
            &mut copy_gate_raw,
        )?;
        let gate = self.tables.sigmoid[(copy_gate_raw[0] + 32767) as usize];

        let mut write_normalized = [0i32; 256];
        normalize_state_into(&state[..width], &mut write_normalized)?;
        fill_low_bit_products(&write_normalized[..width], &mut scratch_products[..width]);

        let mut key = [0i32; KEY_DIM];
        self.affine_direct_into(
            &scratch_products[..width],
            &self.p_read_key,
            &self.read_key_bias,
            width,
            10,
            &mut key,
        )?;

        let mut value_raw = [0i32; VAL_DIM];
        self.affine_direct_into(
            &scratch_products[..width],
            &self.p_read_value,
            &self.read_value_bias,
            width,
            10,
            &mut value_raw,
        )?;
        let mut value = [0i32; VAL_DIM];
        for i in 0..VAL_DIM {
            value[i] = self.tables.tanh[(value_raw[i] + 32767) as usize];
        }

        let mut hidden = [0i32; 256];
        let row_exp = i32::from(self.p_output_norm.spec.row_exponents[0]);
        for d in 0..width {
            let x = write_normalized[d] as i64;
            let w = self.p_output_norm.codes.code(d)?;
            let val = i128::from(mul_code_i64(x, w));
            hidden[d] = scaled(val, row_exp)?.clamp(-32767, 32767) as i32;
        }
        fill_low_bit_products(&hidden[..width], &mut scratch_products[..width]);

        let mut logits = [0i32; 4096];
        self.project_vocab_with_products_into(
            &scratch_products[..width],
            &mut logits[..self.config.vocab_size],
        )?;

        softmax_into(
            &logits[..self.config.vocab_size],
            &self.tables,
            &mut session.last_probabilities[..self.config.vocab_size],
        )?;

        session.copy_scratch.fill(0);
        let copy = &mut session.copy_scratch;
        if mode != ReadMode::NoRead && total_slots > 0 {
            let m_sys = &all_masses[1..1 + n_sys];
            let m_dial = &all_masses[1 + n_sys..1 + n_sys + n_dial];
            let m_page = &all_masses[1 + n_sys + n_dial..1 + total_slots];
            for (&id, &mass) in session.persistent_tokens.iter().zip(m_sys) {
                if mass != 0 && (id as usize) < self.config.vocab_size {
                    copy[id as usize] += mass;
                }
            }
            for (&id, &mass) in session.dialogue_tokens[..n_dial].iter().zip(m_dial) {
                if mass != 0 && (id as usize) < self.config.vocab_size {
                    copy[id as usize] += mass;
                }
            }
            for (page, &mass) in session.l2_pages[..n_page].iter().zip(m_page) {
                if mass != 0 {
                    let m1 = mass >> 2;
                    let m2 = mass >> 3;
                    let m3 = m2;
                    let allocated = (mass >> 1) + m1 + m2 + m3;
                    let m0 = (mass >> 1) + (mass - allocated);

                    let page_masses = [m0, m1, m2, m3];
                    for (&tok, &m) in page.salient_tokens.iter().zip(&page_masses) {
                        if m != 0 && (tok as usize) < self.config.vocab_size {
                            copy[tok as usize] += m;
                        }
                    }
                }
            }
        }

        let fraction = i128::from(TOTAL)
            - scaled(product(i128::from(gate), i128::from(TOTAL - no_read))?, -15)?;
        let fraction_table = build_fraction_table(fraction as u128);
        let u_const = TOTAL >> 12;
        let zero_uniform = div_round_100m_raw(0, u_const)?;

        for (i, &c) in copy[..self.config.vocab_size].iter().enumerate() {
            let v = session.last_probabilities[i];
            if v == 0 && c == 0 {
                session.last_probabilities[i] = zero_uniform;
                continue;
            }
            let copy_term = if c != 0 {
                scaled(i128::from(mul_shift_add_i64(c as i64, gate as i64)), -15)?
            } else {
                0
            };
            let raw = if v != 0 {
                let prod_v = mul_fraction_radix16(v, &fraction_table);
                let quotient = (prod_v >> 48) as i128;
                let rem = prod_v & ((1u128 << 48) - 1);
                let raw_v = if rem >= (1u128 << 47) {
                    quotient + 1
                } else {
                    quotient
                };
                raw_v + copy_term
            } else {
                copy_term
            };
            let uniform = div_round_100m_raw(raw as u64, u_const)?;
            session.last_probabilities[i] = uniform;
        }
        normalize_residual(&mut session.last_probabilities[..self.config.vocab_size])?;

        session.state[..width].copy_from_slice(&state[..width]);
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
                if session.dialogue_len >= session.dialogue_capacity {
                    let evict_idx = session.dialogue_cursor;
                    let evict_turn_id = session.dialogue_turn_ids[evict_idx];
                    if evict_turn_id != session.last_compressed_turn_id {
                        let mut slot_indices = [0usize; DIALOGUE_CAPACITY];
                        let mut count = 0;
                        let mut curr = evict_idx;
                        while count < session.dialogue_len {
                            if session.dialogue_turn_ids[curr] == evict_turn_id {
                                slot_indices[count] = curr;
                                count += 1;
                                curr = if curr + 1 >= session.dialogue_capacity {
                                    0
                                } else {
                                    curr + 1
                                };
                            } else {
                                break;
                            }
                        }

                        if count > 0 {
                            let mut page = L2PrimePage::default();
                            compress_barycenter_key_fibonacci(
                                &session.dialogue_keys,
                                &slot_indices[..count],
                                &mut page.key,
                            );
                            let terminal_idx = slot_indices[count - 1];
                            page.value
                                .copy_from_slice(&session.dialogue_values[terminal_idx]);
                            let start_seq = session.dialogue_sequences[slot_indices[0]];
                            let end_seq = session.dialogue_sequences[terminal_idx];
                            page.prime_signature = compute_turn_prime_signature(
                                &session.dialogue_tokens,
                                &slot_indices[..count],
                                evict_turn_id,
                                start_seq,
                                end_seq,
                            );
                            extract_salient_tokens(
                                &session.dialogue_tokens,
                                &session.dialogue_keys,
                                &slot_indices[..count],
                                &mut page.salient_tokens,
                            );
                            page.start_seq = start_seq as usize;
                            page.end_seq = end_seq as usize;
                            page.turn_id = evict_turn_id;

                            let p_idx = session.l2_cursor;
                            session.l2_pages[p_idx] = page;
                            session.l2_cursor = if p_idx + 1 >= L2_PAGE_CAPACITY {
                                0
                            } else {
                                p_idx + 1
                            };
                            session.l2_len = (session.l2_len + 1).min(L2_PAGE_CAPACITY);
                            session.l2_seen += 1;
                            session.last_compressed_turn_id = evict_turn_id;
                        }
                    }
                }

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

        session.last_read_masses.clear();
        if total_slots > 0 {
            if mode == ReadMode::NoRead {
                session.last_read_masses.resize(total_slots, 0u64);
            } else {
                session
                    .last_read_masses
                    .extend_from_slice(&all_masses[1..1 + total_slots]);
            }
        }
        session.last_no_read_mass = no_read;
        session.last_copy_gate = gate;
        session.has_step = true;

        Ok(())
    }

    /// Step conversational session with partitioned memory, zero age decay for persona,
    /// cyclic FIFO dialogue ring buffer, T8 zeta phase coordinates, and Hopf fiber holonomy tracking.
    #[inline(never)]
    pub fn step_conversational(
        &self,
        session: &mut SessionState,
        token: u32,
        target: SlotTarget,
        mode: ReadMode,
    ) -> Result<IntegerStep> {
        self.step_conversational_into(session, token, target, mode)?;
        Ok(IntegerStep {
            probabilities: session.last_probabilities[..self.config.vocab_size].to_vec(),
            state: session.state[..self.config.width].to_vec(),
            no_read_mass: session.last_no_read_mass,
            read_masses: session.last_read_masses.clone(),
            copy_gate: session.last_copy_gate,
        })
    }

    pub fn synthetic_for_test() -> Self {
        Self::synthetic_with_config(JointConfig::default(), ServingProfile::Retained)
    }

    fn synthetic_with_config(config: JointConfig, serving_profile: ServingProfile) -> Self {
        let mut parameters = BTreeMap::new();
        let shapes = config.shapes();
        for (name, shape) in shapes {
            let total_elements: usize = shape.iter().product();
            let bits = if shape.len() == 2 { 4 } else { 16 };
            let codes = CoefficientCodes::ones(total_elements, bits);
            let row_count = if shape.len() == 2 { shape[0] } else { 1 };
            let spec = ParameterQuantization {
                bits,
                shape: shape.clone(),
                row_exponents: vec![0i16; row_count],
            };
            parameters.insert(name, Parameter { codes, spec });
        }
        let output_bias = match parameters.get("output.bias") {
            Some(p) => p
                .codes
                .iter()
                .map(|v| {
                    scaled(
                        i128::from(v),
                        WORK_BITS + i32::from(p.spec.row_exponents[0]),
                    )
                })
                .collect::<Result<Vec<_>>>()
                .unwrap_or_default(),
            None => Vec::new(),
        };
        let output_shifts = match parameters.get("embedding.weight") {
            Some(emb) => emb
                .spec
                .row_exponents
                .iter()
                .map(|&exp| WORK_BITS - 10 + i32::from(exp))
                .collect(),
            None => Vec::new(),
        };
        let recurrent_bias = match parameters.get("recurrent.bias") {
            Some(p) => p
                .codes
                .iter()
                .map(|v| {
                    scaled(
                        i128::from(v),
                        WORK_BITS + i32::from(p.spec.row_exponents[0]),
                    )
                })
                .collect::<Result<Vec<_>>>()
                .unwrap_or_default(),
            None => Vec::new(),
        };
        let read_age = match parameters.get("read.age") {
            Some(p) => p
                .codes
                .iter()
                .map(|v| {
                    scaled(
                        i128::from(v),
                        WORK_BITS + i32::from(p.spec.row_exponents[0]),
                    )
                })
                .collect::<Result<Vec<_>>>()
                .unwrap_or_default(),
            None => Vec::new(),
        };
        let cache_vec = |name: &str| -> Vec<i128> {
            match parameters.get(name) {
                Some(p) => p
                    .codes
                    .iter()
                    .map(|v| {
                        scaled(
                            i128::from(v),
                            WORK_BITS + i32::from(p.spec.row_exponents[0]),
                        )
                    })
                    .collect::<Result<Vec<_>>>()
                    .unwrap_or_default(),
                None => Vec::new(),
            }
        };
        let update_bias = cache_vec("update.bias");
        let update_gate_bias = cache_vec("update.gate.bias");
        let copy_gate_bias = cache_vec("copy.gate.bias");
        let read_key_bias = cache_vec("read.key.bias");
        let read_value_bias = cache_vec("read.value.bias");
        let read_query_bias = cache_vec("read.query.bias");
        let read_no_read_bias = cache_vec("read.no_read.bias");

        let get_param = |name: &str| -> Parameter {
            parameters.get(name).cloned().unwrap_or_else(|| Parameter {
                codes: CoefficientCodes::ones(0, 4),
                spec: ParameterQuantization {
                    bits: 4,
                    shape: Vec::new(),
                    row_exponents: Vec::new(),
                },
            })
        };
        let p_embedding = get_param("embedding.weight");
        let p_recurrent_input = get_param("recurrent.input.weight");
        let p_recurrent_state = get_param("recurrent.state.weight");
        let p_read_query = get_param("read.query.weight");
        let p_read_no_read = get_param("read.no_read.weight");
        let p_update = get_param("update.weight");
        let p_update_gate = get_param("update.gate.weight");
        let p_copy_gate = get_param("copy.gate.weight");
        let p_read_key = get_param("read.key.weight");
        let p_read_value = get_param("read.value.weight");
        let p_output_norm = get_param("output.norm.weight");

        let tables = Tables::synthetic_for_test();
        Self {
            config,
            serving_profile,
            parameters,
            tables,
            lorentz: None,
            identity: match serving_profile {
                ServingProfile::Retained => "synthetic_model",
                ServingProfile::Dialogue576 => "synthetic_dialogue576",
            }
            .to_owned(),
            output_bias,
            output_shifts,
            recurrent_bias,
            read_age,
            update_bias,
            update_gate_bias,
            copy_gate_bias,
            read_key_bias,
            read_value_bias,
            read_query_bias,
            read_no_read_bias,
            p_embedding,
            p_recurrent_input,
            p_recurrent_state,
            p_read_query,
            p_read_no_read,
            p_update,
            p_update_gate,
            p_copy_gate,
            p_read_key,
            p_read_value,
            p_output_norm,
        }
    }
}

/// State Q11 -> RMS-normalized Q10. Variance uses Q64 guard precision in code
/// units: sum(x_code^2)/width + 2^22/100000, width 128 or 256. The square root
/// has 32 guard bits.
#[inline(always)]
fn normalize_state_into(input: &[i32], out: &mut [i32; 256]) -> Result<()> {
    if input.len() != 256 {
        return Err(invalid("integer normalization width must be 256"));
    }
    let mut sum = 0u64;
    for &x in input {
        let x64 = x as i64;
        sum += mul_shift_add_i64(x64, x64) as u64;
    }
    const EPSILON: u128 = (1u128 << 86) / 100_000 + 1;
    let variance = (u128::from(sum) << 56) + EPSILON;
    let denominator = math::isqrt(variance);
    let d_u64 = denominator as u64;
    let table = build_div_table_u64(d_u64);
    for i in 0..256 {
        let x = input[i];
        if x == 0 {
            out[i] = 0;
        } else {
            let neg = x < 0;
            let abs_x = x.unsigned_abs() as u128;
            let q = fast_div_round_radix16_u64(abs_x << 42, &table, d_u64);
            let val = if neg { -(q as i32) } else { q as i32 };
            out[i] = val.clamp(-32767, 32767);
        }
    }
    Ok(())
}

#[inline(never)]
fn normalize_state(input: &[i32]) -> Result<Vec<i32>> {
    if input.len() == 576 {
        return normalize_state_576(input);
    }
    let shift = match input.len() {
        128 => 57,
        256 => 56,
        _ => return Err(invalid("integer normalization width must be 128 or 256")),
    };
    if input.len() == 256 {
        let mut output = [0i32; 256];
        normalize_state_into(input, &mut output)?;
        return Ok(output.to_vec());
    }
    let mut sum = 0u64;
    for &x in input {
        let x64 = x as i64;
        sum += mul_shift_add_i64(x64, x64) as u64;
    }
    const EPSILON: u128 = (1u128 << 86) / 100_000 + 1;
    let variance = (u128::from(sum) << shift) + EPSILON;
    let denominator = math::isqrt(variance);
    let d_u64 = denominator as u64;
    let table = build_div_table_u64(d_u64);
    let mut out = vec![0i32; input.len()];
    for i in 0..input.len() {
        let x = input[i];
        if x == 0 {
            out[i] = 0;
        } else {
            let neg = x < 0;
            let abs_x = x.unsigned_abs() as u128;
            let q = fast_div_round_radix16_u64(abs_x << 42, &table, d_u64);
            let val = if neg { -(q as i32) } else { q as i32 };
            out[i] = val.clamp(-32767, 32767);
        }
    }
    Ok(out)
}

/// Q11 -> Q10 with the explicit width576 rational variance. The signed16
/// interface bound gives S<9*2^36, (S<<58)<2^98, D<2^47 and a radix16 divisor
/// table below 2^51. Reject a wider private-helper domain before accumulating.
/// The division by9 floors BEFORE epsilon and square root; rounding it or
/// first dividing S by576 would define a different numerical contract.
#[inline(never)]
fn normalize_state_576(input: &[i32]) -> Result<Vec<i32>> {
    if input.len() != 576 || input.iter().any(|&x| !(-32767..=32767).contains(&x)) {
        return Err(invalid(
            "dialogue576 normalization requires 576 signed16 interface codes",
        ));
    }
    let mut sum = 0u64;
    for &x in input {
        let x64 = i64::from(x);
        sum += mul_shift_add_i64(x64, x64) as u64;
    }
    const EPSILON: u128 = (1u128 << 86) / 100_000 + 1;
    let variance = arithmetic(math::div_rem_unsigned(u128::from(sum) << 58, 9))?.0 + EPSILON;
    let denominator = math::isqrt(variance) as u64;
    let table = build_div_table_u64(denominator);
    let mut out = vec![0i32; 576];
    for (value, &x) in out.iter_mut().zip(input) {
        if x != 0 {
            let q =
                fast_div_round_radix16_u64(u128::from(x.unsigned_abs()) << 42, &table, denominator);
            let signed = if x < 0 { -(q as i32) } else { q as i32 };
            *value = signed.clamp(-32767, 32767);
        }
    }
    Ok(out)
}

#[inline(always)]
fn blend_into(state: &[i32], candidate: &[i32], gates: &[i32], out: &mut [i32]) -> Result<()> {
    for i in 0..state.len() {
        let x = state[i] as i64;
        let y = candidate[i] as i64;
        let g = gates[i] as i64;
        let a = mul_shift_add_i64(32768 - g, x << 3);
        let b = mul_shift_add_i64(g, y);
        out[i] = quantize_q29_to_q11(a + b);
    }
    Ok(())
}

#[inline(never)]
fn blend(state: &[i32], candidate: &[i32], gates: &[i32]) -> Result<Vec<i32>> {
    let mut out = vec![0i32; state.len()];
    blend_into(state, candidate, gates, &mut out)?;
    Ok(out)
}

#[inline(always)]
fn blend_scalar_into(state: &[i32], candidate: &[i32], g: i32, out: &mut [i32]) -> Result<()> {
    let one_minus_g = 32768 - g;
    let t_one_minus_g = build_coord_products(one_minus_g);
    let t_g = build_coord_products(g);
    for i in 0..state.len() {
        let x = state[i];
        let y = candidate[i];
        let a = dot_coord(&t_one_minus_g, x << 3);
        let b = dot_coord(&t_g, y);
        out[i] = quantize_q29_to_q11(a + b);
    }
    Ok(())
}

#[allow(dead_code)]
#[inline(always)]
fn blend_scalar(state: &[i32], candidate: &[i32], g: i32) -> Result<Vec<i32>> {
    let mut out = vec![0i32; state.len()];
    blend_scalar_into(state, candidate, g, &mut out)?;
    Ok(out)
}

#[inline(always)]
fn transport_into(input: &[i32], raw: &[i32], kind: Transport, out: &mut [i32]) -> Result<()> {
    let mut out_idx = 0;
    let n = input.len().min(raw.len()) >> 2;
    for i in 0..n {
        let base = i << 2;
        let x = &input[base..base + 4];
        let r = &raw[base..base + 4];
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
            let u = v.unsigned_abs() as u64;
            sum += square_u64_to_u128(u);
        }
        let denominator = math::isqrt(sum) as i128;
        let mut unit = [0i64; 4];
        // Minimum nonzero quantized center is well above original1e-6,
        // except a possible cancellation in the real-coordinate component.
        let threshold = match kind {
            Transport::Quaternion => 10_995_116i128,
            Transport::HouseholderPair => 15_549_443i128,
        };
        if denominator <= threshold {
            unit[0] = 16384;
        } else {
            let den_u64 = denominator as u64;
            for j in 0..4 {
                let num = centered[j] << 14;
                let num_neg = num < 0;
                let num_u64 = num.unsigned_abs() as u64;
                let value = div_round_u64(num_u64, den_u64, num_neg);
                unit[j] = value.clamp(-32767, 32767);
            }
        }
        let x0 = i64::from(x[0]);
        let x1 = i64::from(x[1]);
        let x2 = i64::from(x[2]);
        let x3 = i64::from(x[3]);
        match kind {
            Transport::Quaternion => {
                let (t0, t1, t2, t3) =
                    build_i64_nibble_table_4x(unit[0], unit[1], unit[2], unit[3]);

                let (p00, p10, p20, p30) = mul_nibble_table_4x(&t0, &t1, &t2, &t3, x0);
                let (p01, p11, p21, p31) = mul_nibble_table_4x(&t0, &t1, &t2, &t3, x1);
                let (p02, p12, p22, p32) = mul_nibble_table_4x(&t0, &t1, &t2, &t3, x2);
                let (p03, p13, p23, p33) = mul_nibble_table_4x(&t0, &t1, &t2, &t3, x3);

                let y0 = p00 - p11 - p22 - p33;
                let y1 = p01 + p10 + p23 - p32;
                let y2 = p02 - p13 + p20 + p31;
                let y3 = p03 + p12 - p21 + p30;

                out[out_idx] = quantize_q25_to_q11(y0);
                out[out_idx + 1] = quantize_q25_to_q11(y1);
                out[out_idx + 2] = quantize_q25_to_q11(y2);
                out[out_idx + 3] = quantize_q25_to_q11(y3);
                out_idx += 4;
            }
            Transport::HouseholderPair => {
                let reflected = [-x0, x1, x2, x3];
                let (t0, t1, t2, t3) =
                    build_i64_nibble_table_4x(unit[0], unit[1], unit[2], unit[3]);

                let dot = mul_nibble_table_i64(&t0, reflected[0])
                    + mul_nibble_table_i64(&t1, reflected[1])
                    + mul_nibble_table_i64(&t2, reflected[2])
                    + mul_nibble_table_i64(&t3, reflected[3]);

                let t_dot = build_i64_nibble_table(dot);
                let d0 = mul_nibble_table_i64(&t_dot, unit[0]);
                let d1 = mul_nibble_table_i64(&t_dot, unit[1]);
                let d2 = mul_nibble_table_i64(&t_dot, unit[2]);
                let d3 = mul_nibble_table_i64(&t_dot, unit[3]);

                let y0 = (reflected[0] << 28) - (d0 << 1);
                let y1 = (reflected[1] << 28) - (d1 << 1);
                let y2 = (reflected[2] << 28) - (d2 << 1);
                let y3 = (reflected[3] << 28) - (d3 << 1);

                out[out_idx] = quantize_q39_to_q11(y0);
                out[out_idx + 1] = quantize_q39_to_q11(y1);
                out[out_idx + 2] = quantize_q39_to_q11(y2);
                out[out_idx + 3] = quantize_q39_to_q11(y3);
                out_idx += 4;
            }
        }
    }
    Ok(())
}

#[inline(never)]
fn transport(input: &[i32], raw: &[i32], kind: Transport) -> Result<Vec<i32>> {
    let mut output = vec![0i32; input.len()];
    transport_into(input, raw, kind, &mut output)?;
    Ok(output)
}

#[inline(always)]
fn softmax_into(scores: &[i32], tables: &Tables, masses_out: &mut [u64]) -> Result<()> {
    if scores.is_empty() || masses_out.len() < scores.len() {
        return Err(invalid("empty integer softmax or output buffer too small"));
    }
    let max = *scores
        .iter()
        .max()
        .ok_or_else(|| invalid("empty integer softmax"))?;
    let mut sum: u64 = 0;
    for (i, &s) in scores.iter().enumerate() {
        let diff = (max - s) as usize;
        let w = if diff < tables.exp.len() {
            tables.exp[diff]
        } else {
            0
        };
        sum = sum.wrapping_add(w);
        masses_out[i] = w;
    }
    let sum_u = u128::from(sum);
    for w in masses_out[..scores.len()].iter_mut() {
        if *w != 0 {
            let num = u128::from(*w) << 48;
            let (q, r) =
                math::div_rem_unsigned(num, sum_u).map_err(|e| invalid(format!("{e:?}")))?;
            let rounded = q + u128::from(r >= sum_u - r);
            *w = rounded as u64;
        }
    }
    normalize_residual(&mut masses_out[..scores.len()])?;
    Ok(())
}

#[inline(never)]
fn softmax(scores: &[i32], tables: &Tables) -> Result<Vec<u64>> {
    let mut masses = vec![0u64; scores.len()];
    softmax_into(scores, tables, &mut masses)?;
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
    fn packed_signed4_affine_accumulation() -> Result<()> {
        let input = [i32::MIN, -32767, -3, 0, 1, 16384, 32767, i32::MAX];
        let products = low_bit_products(&input);
        for code in -7i16..=7 {
            for (&x, multiples) in input.iter().zip(&products) {
                assert_eq!(
                    multiples[usize::from((code as u16) & 15)],
                    i64::from(x) * i64::from(code)
                );
            }
            let weights = CoefficientCodes::from_decoded(vec![code; input.len()], 4)?;
            let expected: i64 = input.iter().map(|&x| i64::from(x) * i64::from(code)).sum();
            assert_eq!(
                low_bit_dot(&products, weights.packed_range(0, input.len())?),
                expected
            );
        }
        for x in [i32::MIN, i32::MAX] {
            let products = low_bit_products(&[x; 512]);
            for code in [-7, 7] {
                let weights = CoefficientCodes::from_decoded(vec![code; 512], 4)?;
                assert_eq!(
                    low_bit_dot(&products, weights.packed_range(0, 512)?),
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
        let weights = CoefficientCodes::from_decoded(mixed_codes, 4)?;
        assert_eq!(
            low_bit_dot(
                &low_bit_products(&mixed_input),
                weights.packed_range(0, 512)?
            ),
            expected
        );
        Ok(())
    }

    #[test]
    fn packed_signed4_matrix_dispatch_rows_scales_and_embedding() -> Result<()> {
        let mut model = IntegerModel::synthetic_for_test();
        for width in [128, 256, 512, 576, 1152] {
            // Mixed signs, zero, and non-block-aligned row counts exercise all
            // dispatches, including the tail after a blocked group and the
            // non-power-of-two wide rows.
            let rows = 9;
            let input: Vec<i32> = (0..width)
                .map(|i| [i32::MIN, -32767, -1, 0, 1, 32767, i32::MAX][i % 7])
                .collect();
            let products = low_bit_products(&input);
            let decoded: Vec<i16> = (0..rows * width)
                .map(|i| ((i * 11 + 3) % 15) as i16 - 7)
                .collect();
            let parameter = Parameter {
                codes: CoefficientCodes::from_decoded(decoded.clone(), 4)?,
                spec: ParameterQuantization {
                    bits: 4,
                    shape: vec![rows, width],
                    row_exponents: (0..rows).map(|r| r as i16 - 5).collect(),
                },
            };
            let mut sums = vec![i128::MIN; rows];
            model.matrix_work_direct_into(&products, &parameter, width, -10, &mut sums)?;
            let biases: Vec<i128> = (0..rows).map(|i| (i as i128 - 4) << 30).collect();
            let mut affine = vec![0; rows];
            model.affine_direct_into(&products, &parameter, &biases, width, 10, &mut affine)?;
            for row in 0..rows {
                let dot: i64 = input
                    .iter()
                    .zip(&decoded[row * width..(row + 1) * width])
                    .map(|(&x, &w)| i64::from(x) * i64::from(w))
                    .sum();
                let expected = scaled(
                    i128::from(dot),
                    30 + i32::from(parameter.spec.row_exponents[row]),
                )?;
                assert_eq!(sums[row], expected);
                assert_eq!(affine[row], quantize(expected + biases[row], WORK_BITS, 8)?);
            }
            let mut untouched = vec![123i128; rows];
            assert!(model
                .matrix_work_direct_into(
                    &products[..width - 1],
                    &parameter,
                    width,
                    -10,
                    &mut untouched
                )
                .is_err());
            assert_eq!(untouched, vec![123; rows]);
        }
        for width in [128, 256, 576] {
            let decoded: Vec<i16> = (0..4096 * width)
                .map(|i| ((i * 7 + 2) % 15) as i16 - 7)
                .collect();
            let embedding = Parameter {
                codes: CoefficientCodes::from_decoded(decoded.clone(), 4)?,
                spec: ParameterQuantization {
                    bits: 4,
                    shape: vec![4096, width],
                    row_exponents: vec![-3; 4096],
                },
            };
            model.config.width = width;
            model
                .parameters
                .insert("embedding.weight".into(), embedding.clone());
            model.p_embedding = embedding;
            model.output_shifts.fill(27);
            for token in [0, 1, 4095] {
                assert_eq!(
                    model.embed(token as u32)?,
                    decoded[token * width..(token + 1) * width]
                        .iter()
                        .map(|&x| i32::from(x))
                        .collect::<Vec<_>>()
                );
            }
            let input: Vec<i32> = (0..width).map(|i| i as i32 - 83).collect();
            let products = low_bit_products(&input);
            let actual = model.project_vocab_with_products(&products)?;
            for row in 0..4096 {
                let dot: i64 = input
                    .iter()
                    .zip(&decoded[row * width..(row + 1) * width])
                    .map(|(&x, &w)| i64::from(x) * i64::from(w))
                    .sum();
                assert_eq!(
                    actual[row],
                    quantize(
                        scaled(i128::from(dot), 27)? + model.output_bias[row],
                        WORK_BITS,
                        8
                    )?
                );
            }
            let mut from_packed = vec![[0; 16]; width];
            fill_packed_low_bit_products(
                model.p_embedding.codes.packed_range(width, width)?,
                &mut from_packed,
            );
            assert_eq!(from_packed, low_bit_products(&model.embed(1)?));
        }
        Ok(())
    }

    #[test]
    fn normalization_zero_scale_and_transport_identity() -> Result<()> {
        assert_eq!(normalize_state(&vec![0; 256])?, vec![0; 256]);
        let normalized = normalize_state(&vec![2048; 256])?;
        assert!(normalized.iter().all(|&x| x == 1024));
        // Width 128 divides by its own count: the same unit-RMS result.
        assert!(normalize_state(&vec![2048; 128])?
            .iter()
            .all(|&x| x == 1024));
        let alternating: Vec<i32> = (0..256)
            .map(|i| if i % 2 == 0 { 2048 } else { -2048 })
            .collect();
        assert_eq!(
            normalize_state(&alternating[..128])?,
            normalize_state(&alternating)?[..128]
        );
        assert!(normalize_state(&vec![1; 64]).is_err());
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

    #[test]
    fn dialogue576_normalization_matches_rational_reference() -> Result<()> {
        let mut sparse = vec![0; 576];
        sparse[575] = -32767;
        let mixed: Vec<i32> = (0..576)
            .map(|i| [-32767, -2048, -1, 0, 1, 2, 16384, 32767][i % 8])
            .collect();
        for input in [
            vec![0; 576],
            vec![2048; 576],
            vec![32767; 576],
            sparse,
            mixed,
        ] {
            // Test-only ordinary arithmetic independently evaluates the new
            // rational variance and the existing ties-away quotient rule.
            let sum: u128 = input
                .iter()
                .map(|&x| (i128::from(x) * i128::from(x)) as u128)
                .sum();
            let variance = (sum << 58) / 9 + (1u128 << 86) / 100_000 + 1;
            let denominator = math::isqrt(variance);
            let expected: Vec<i32> = input
                .iter()
                .map(|&x| {
                    let numerator = u128::from(x.unsigned_abs()) << 42;
                    let rounded = (numerator / denominator)
                        + u128::from(
                            numerator % denominator >= denominator - numerator % denominator,
                        );
                    let signed = if x < 0 {
                        -(rounded as i32)
                    } else {
                        rounded as i32
                    };
                    signed.clamp(-32767, 32767)
                })
                .collect();
            assert_eq!(normalize_state(&input)?, expected);
        }
        assert_eq!(normalize_state(&vec![2048; 576])?, vec![1024; 576]);
        assert!(normalize_state_576(&vec![1; 575]).is_err());
        for bad in [i32::MIN, -32768, 32768, i32::MAX] {
            let mut input = vec![0; 576];
            input[575] = bad;
            assert!(normalize_state(&input).is_err());
        }
        Ok(())
    }

    #[test]
    fn dialogue576_full_value_memory_and_step() -> Result<()> {
        // All 256 slots and coordinates beyond the old limit participate.
        let mut memory = SessionValues::Dialogue576(Vec::with_capacity(256));
        let mut row = [0; 576];
        row[0] = 8192;
        row[255] = -8192;
        row[256] = 16384;
        row[575] = -16384;
        for _ in 0..256 {
            memory.push(&row)?;
        }
        let read = memory.read(&[TOTAL >> 8; 256], 576)?;
        assert_eq!(
            (read[0], read[255], read[256], read[575]),
            (1024, -1024, 2048, -2048)
        );
        assert!(memory.push(&row[..256]).is_err());
        assert_eq!(memory.read(&[TOTAL >> 8; 256], 576)?, read);
        assert!(memory.read(&[TOTAL >> 8; 255], 576).is_err());

        let config = JointConfig {
            width: 576,
            ..JointConfig::default()
        };
        config.validate_for_profile(ServingProfile::Dialogue576)?;
        let model = IntegerModel::synthetic_with_config(config, ServingProfile::Dialogue576);
        assert_eq!(model.serving_profile(), ServingProfile::Dialogue576);
        for mode in [ReadMode::Enabled, ReadMode::NoRead] {
            let mut session = model.new_session();
            let first = model.step(&mut session, 0, mode)?;
            let second = model.step(&mut session, 10, mode)?;
            assert_eq!(first.state.len(), 576);
            assert_eq!(second.state.len(), 576);
            assert_eq!(second.probabilities.iter().sum::<u64>(), TOTAL);
            assert_eq!(second.read_masses.len(), 1);
            if mode == ReadMode::NoRead {
                assert_eq!(second.read_masses, [0]);
            }
            let SessionValues::Dialogue576(values) = &session.values else {
                return Err(invalid("wide session allocated retained values"));
            };
            assert_eq!(values.len(), 2);
            assert!(values.iter().all(|value| value[575] != 0));
            let old_state = session.state.clone();
            session.tokens.resize(256, 0);
            assert!(model.step(&mut session, 0, mode).is_err());
            assert_eq!(session.state, old_state);
        }
        // The separate, fixed256 Google API remains outside this profile.
        let mut conversational = model.new_conversational_session();
        assert!(model
            .step_conversational(
                &mut conversational,
                0,
                SlotTarget::Dialogue,
                ReadMode::Enabled
            )
            .is_err());
        let retained = IntegerModel::synthetic_for_test().new_session();
        assert!(matches!(retained.values, SessionValues::Retained(_)));
        Ok(())
    }

    #[test]
    fn dialogue576_profile_manifest_is_explicit_and_separate() -> Result<()> {
        use crate::config::packed_numerical_contract_for_profile;
        use crate::IntegerError;
        let config = JointConfig {
            width: 576,
            ..JointConfig::default()
        };
        assert!(config.validate().is_err());
        config.validate_for_profile(ServingProfile::Dialogue576)?;
        assert!(JointConfig::default()
            .validate_for_profile(ServingProfile::Dialogue576)
            .is_err());
        let value = serde_json::to_value(&config)?;
        for (field, bad) in [
            ("vocab_size", serde_json::json!(2048)),
            ("width", serde_json::json!(256)),
            ("read_width", serde_json::json!(128)),
            ("context", serde_json::json!(128)),
            ("transport", serde_json::json!("householder_pair")),
            ("read_geometry", serde_json::json!("lorentz")),
        ] {
            let mut changed = value.clone();
            changed[field] = bad;
            assert!(serde_json::from_value::<JointConfig>(changed)?
                .validate_for_profile(ServingProfile::Dialogue576)
                .is_err());
        }
        let retained = crate::config::packed_numerical_contract(ReadGeometry::Dot)?;
        assert_eq!(
            serde_json::to_vec(&packed_numerical_contract_for_profile(
                ReadGeometry::Dot,
                ServingProfile::Retained
            )?)?,
            serde_json::to_vec(&retained)?
        );
        let wide =
            packed_numerical_contract_for_profile(ReadGeometry::Dot, ServingProfile::Dialogue576)?;
        assert_eq!(wide["native_integer_profile"]["width"], 576);
        assert!(packed_numerical_contract_for_profile(
            ReadGeometry::Lorentz,
            ServingProfile::Dialogue576
        )
        .is_err());
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| invalid("test clock before epoch"))?
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "uor-integer-dialogue576-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&directory)?;
        let manifest = serde_json::json!({
            "schema": "uor-r4.joint-recurrent-packed-emulator/1",
            "serving_profile": "dialogue576", "admission": "full",
            "model": config, "numerical_contract": wide,
        });
        for (field, replacement) in [
            ("serving_profile", None),
            ("serving_profile", Some(serde_json::json!("retained"))),
            ("admission", None),
            ("admission", Some(serde_json::json!("recent64"))),
            ("numerical_contract", Some(retained)),
        ] {
            let mut changed = manifest.clone();
            if let Some(replacement) = replacement {
                changed[field] = replacement;
            } else if let Some(object) = changed.as_object_mut() {
                object.remove(field);
            }
            fs::write(
                directory.join("hard-model.json"),
                serde_json::to_vec(&changed)?,
            )?;
            assert!(matches!(
                IntegerModel::load_with_tables_profile(
                    &directory,
                    &directory,
                    ServingProfile::Dialogue576
                ),
                Err(IntegerError::Invalid(_))
            ));
        }
        fs::write(
            directory.join("hard-model.json"),
            serde_json::to_vec(&manifest)?,
        )?;
        assert!(matches!(
            IntegerModel::load_with_tables(&directory, &directory),
            Err(IntegerError::Invalid(_))
        ));
        // Correct metadata reaches the absent parameter descriptor; this is
        // profile/contract evidence, not a loaded learned artifact replay.
        assert!(matches!(
            IntegerModel::load_with_tables_profile(
                &directory,
                &directory,
                ServingProfile::Dialogue576
            ),
            Err(IntegerError::Io(_))
        ));
        fs::remove_dir_all(directory)?;
        Ok(())
    }

    /// Each read geometry binds its own contract. The dot contract is the
    /// retained legacy contract; the Lorentz contract changes only the read
    /// declaration. A manifest pairing one geometry with the other's contract,
    /// or with none, is refused before any parameter or table access, and a
    /// matching Lorentz manifest passes on to the parameter binding.
    #[test]
    fn loader_binds_each_read_geometry_to_its_contract() -> Result<()> {
        use crate::config::{packed_numerical_contract, quantized_numerical_contract};
        use crate::IntegerError;
        let dot_contract = packed_numerical_contract(ReadGeometry::Dot)?;
        let lorentz_contract = packed_numerical_contract(ReadGeometry::Lorentz)?;
        assert_eq!(dot_contract, quantized_numerical_contract()?);
        let (Some(dot_fields), Some(lorentz_fields)) =
            (dot_contract.as_object(), lorentz_contract.as_object())
        else {
            return Err(invalid("contracts are JSON objects"));
        };
        let changed: Vec<&str> = lorentz_fields
            .iter()
            .filter(|(key, value)| dot_fields.get(*key) != Some(*value))
            .map(|(key, _)| key.as_str())
            .collect();
        assert_eq!(changed, ["read", "read_geometry"]);
        assert!(dot_fields
            .keys()
            .all(|key| lorentz_fields.contains_key(key)));

        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| invalid("test clock before epoch"))?
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "uor-integer-read-geometry-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&directory)?;
        let write = |model: Value, contract: Option<&Value>| -> Result<()> {
            let mut manifest = serde_json::json!({
                "schema": "uor-r4.joint-recurrent-packed-emulator/1",
                "model": model,
            });
            if let Some(contract) = contract {
                manifest["numerical_contract"] = contract.clone();
            }
            fs::write(
                directory.join("hard-model.json"),
                serde_json::to_vec(&manifest)?,
            )?;
            Ok(())
        };
        let lorentz = serde_json::to_value(JointConfig {
            read_geometry: ReadGeometry::Lorentz,
            ..JointConfig::default()
        })?;
        let dot = serde_json::to_value(JointConfig::default())?;
        let mut explicit_dot = dot.clone();
        explicit_dot["read_geometry"] = serde_json::json!("dot");
        for (model, contract) in [
            (&lorentz, Some(&dot_contract)),
            (&lorentz, None),
            (&dot, Some(&lorentz_contract)),
            (&dot, None),
            (&explicit_dot, None),
        ] {
            write(model.clone(), contract)?;
            let Err(error) = IntegerModel::load_with_tables(&directory, &directory) else {
                return Err(invalid("mismatched manifest was accepted"));
            };
            assert!(
                matches!(&error, IntegerError::Invalid(message) if message.contains("numerical contract")),
                "{error}"
            );
        }
        // A matching Lorentz manifest passes the contract and then needs its
        // parameter descriptor, which this directory lacks.
        write(lorentz, Some(&lorentz_contract))?;
        let Err(error) = IntegerModel::load_with_tables(&directory, &directory) else {
            return Err(invalid("parameter-free manifest was accepted"));
        };
        assert!(matches!(error, IntegerError::Io(_)), "{error}");
        fs::remove_dir_all(&directory)?;
        Ok(())
    }

    #[test]
    fn fast_mixture_radix16_matches_exact_arithmetic() -> Result<()> {
        let fractions = [0u128, 1, 1000, 123456789, TOTAL as u128];
        let values: [u64; 6] = [0, 1, 42, 65535, 123456789, TOTAL];

        for &fraction in &fractions {
            let table = build_fraction_table(fraction);
            for &v in &values {
                let prod = mul_fraction_radix16(v, &table);
                let expected = (u128::from(v)) * fraction;
                assert_eq!(
                    prod, expected,
                    "mul_fraction_radix16 mismatch for v={v}, f={fraction}"
                );
            }
        }

        for &raw in &[0u64, 1, 100, 12345, 99999999, TOTAL, TOTAL * 2] {
            let prod99 = mul99_radix16(raw);
            let expected = u128::from(raw) * 99_999_999u128;
            assert_eq!(prod99, expected, "mul99_radix16 mismatch for raw={raw}");
        }

        let test_numerators = [
            0u128,
            1,
            50_000_000,
            100_000_000,
            150_000_000,
            100_000_000 * 42 + 37,
            (TOTAL as u128) * 99_999_999 + (TOTAL as u128 >> 12),
        ];
        for &num in &test_numerators {
            let fast = fast_div_round_100m(num);
            let expected = math::div_round(num as i128, 100_000_000)
                .map_err(|e| invalid(format!("{e:?}")))? as u64;
            assert_eq!(fast, expected, "fast_div_round_100m mismatch for num={num}");
        }

        let test_denominators = [
            1u128,
            7,
            100,
            12345,
            65536,
            1_000_000,
            (1u128 << 48) - 1,
            (1u128 << 48) + 1234567,
        ];
        for &den in &test_denominators {
            for &num in &[
                0u128,
                1,
                den / 2,
                den - 1,
                den,
                den + 1,
                den * 3 + den / 2,
                den * 17 + 5,
            ] {
                let fast = fast_div_round(num, den);
                let expected = math::div_round(num as i128, den as i128)
                    .map_err(|e| invalid(format!("{e:?}")))? as u64;
                assert_eq!(
                    fast, expected,
                    "fast_div_round mismatch for num={num}, den={den}"
                );
            }
        }
        Ok(())
    }

    #[test]
    fn mul_shift_add_i64_matches_scalar() {
        let values = [-32767i64, -2560, -100, -1, 0, 1, 100, 2560, 32767];
        for &a in &values {
            for &b in &values {
                assert_eq!(mul_shift_add_i64(a, b), a * b, "mismatch for a={a}, b={b}");
            }
        }
    }

    #[test]
    fn div_round_100m_raw_matches_exact() -> Result<()> {
        let u_const = TOTAL >> 12;
        let test_raws = [
            0u64,
            1,
            2,
            42,
            100,
            1000,
            100_000_000,
            u_const.saturating_sub(1),
            u_const,
            u_const + 1,
            u_const + 100_000_000,
            TOTAL,
            TOTAL * 2,
        ];
        for &raw in &test_raws {
            let res = div_round_100m_raw(raw, u_const)?;
            let num = (raw as u128) * 99_999_999 + (u_const as u128);
            let expected = math::div_round(num as i128, 100_000_000)
                .map_err(|e| invalid(format!("{e:?}")))? as u64;
            assert_eq!(res, expected, "div_round_100m_raw mismatch for raw={raw}");
        }
        Ok(())
    }

    #[test]
    fn packed_signed4_eight_rows_match_scalar() -> Result<()> {
        let input: Vec<i32> = (0..256i32).map(|i| i * 19 - 2468).collect();
        let products = low_bit_products(&input);
        let p_256: &[[i64; 16]; 256] = (&products[..256]).try_into().unwrap();

        let mut weights_2048 = [0i16; 2048];
        for (i, w) in weights_2048.iter_mut().enumerate() {
            *w = (((i * 11 + 3) % 15) as i16) - 7;
        }

        let packed = CoefficientCodes::from_decoded(weights_2048.to_vec(), 4)?;
        let bytes: &[u8; 1024] = packed
            .packed_range(0, 2048)?
            .try_into()
            .map_err(|_| invalid("test packed tile"))?;
        let (d0, d1, d2, d3, d4, d5, d6, d7) = low_bit_dot_8_contiguous(p_256, bytes);

        let scalar_dot = |weights: &[i16]| -> i64 {
            input
                .iter()
                .zip(weights)
                .map(|(&x, &w)| i64::from(x) * i64::from(w))
                .sum()
        };

        assert_eq!(d0, scalar_dot(&weights_2048[0..256]));
        assert_eq!(d1, scalar_dot(&weights_2048[256..512]));
        assert_eq!(d2, scalar_dot(&weights_2048[512..768]));
        assert_eq!(d3, scalar_dot(&weights_2048[768..1024]));
        assert_eq!(d4, scalar_dot(&weights_2048[1024..1280]));
        assert_eq!(d5, scalar_dot(&weights_2048[1280..1536]));
        assert_eq!(d6, scalar_dot(&weights_2048[1536..1792]));
        assert_eq!(d7, scalar_dot(&weights_2048[1792..2048]));
        Ok(())
    }

    #[test]
    fn scale_and_quantize_logit_matches_reference() -> Result<()> {
        let dots = [
            -50_000_000i64,
            -1_000_000,
            -100,
            0,
            100,
            1_000_000,
            50_000_000,
        ];
        let shifts = [10, 14, 15, 16, 20, 25];
        let biases = [-100_000_000i128, -1000, 0, 1000, 100_000_000];

        for &dot in &dots {
            for &shift in &shifts {
                for &bias in &biases {
                    let fast = scale_and_quantize_logit(dot, shift, bias);
                    let val = scaled(i128::from(dot), shift)?;
                    let expected = quantize(val + bias, WORK_BITS, 8)?;
                    assert_eq!(
                        fast, expected,
                        "mismatch for dot={dot}, shift={shift}, bias={bias}"
                    );
                }
            }
        }
        Ok(())
    }

    #[test]
    fn mul_nibble_table_matches_scalar() {
        let vals = [
            -32767i64, -16384, -2560, -100, -1, 0, 1, 100, 2560, 16384, 32767,
        ];
        let xs = [-32767i64, -16384, -500, -1, 0, 1, 500, 16384, 32767];
        for &val in &vals {
            let table = build_i64_nibble_table(val);
            for &x in &xs {
                let fast = mul_nibble_table_i64(&table, x);
                let expected = val * x;
                assert_eq!(fast, expected, "mismatch for val={val}, x={x}");
            }
        }
    }

    #[test]
    fn mul_nibble_table_4x_matches_mul_nibble_table_i64() {
        let vals = [
            -32767i64, -16384, -2560, -100, -1, 0, 1, 100, 2560, 16384, 32767,
        ];
        let xs = [-32767i64, -16384, -500, -1, 0, 1, 500, 16384, 32767];
        for i in 0..vals.len().saturating_sub(3) {
            let (t0, t1, t2, t3) =
                build_i64_nibble_table_4x(vals[i], vals[i + 1], vals[i + 2], vals[i + 3]);
            let s0 = build_i64_nibble_table(vals[i]);
            let s1 = build_i64_nibble_table(vals[i + 1]);
            let s2 = build_i64_nibble_table(vals[i + 2]);
            let s3 = build_i64_nibble_table(vals[i + 3]);
            assert_eq!(t0, s0);
            assert_eq!(t1, s1);
            assert_eq!(t2, s2);
            assert_eq!(t3, s3);
            for &x in &xs {
                let (p0, p1, p2, p3) = mul_nibble_table_4x(&t0, &t1, &t2, &t3, x);
                assert_eq!(p0, mul_nibble_table_i64(&s0, x));
                assert_eq!(p1, mul_nibble_table_i64(&s1, x));
                assert_eq!(p2, mul_nibble_table_i64(&s2, x));
                assert_eq!(p3, mul_nibble_table_i64(&s3, x));
            }
        }
    }

    #[test]
    fn mul_code_i64_matches_scalar() {
        let xs = [-32767i64, -1000, -1, 0, 1, 1000, 32767];
        for code in -7..=7i16 {
            for &x in &xs {
                let fast = mul_code_i64(x, code);
                let expected = x * i64::from(code);
                assert_eq!(fast, expected, "mismatch for x={x}, code={code}");
            }
        }
    }
}
