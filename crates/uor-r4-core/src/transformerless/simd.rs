use super::compiler::{D, K, SIG_BYTES};

#[inline(always)]
pub fn hamming_distance_36_scalar(a: &[u8; SIG_BYTES], b: &[u8; SIG_BYTES]) -> u32 {
    let mut dist = 0u32;
    for (chunk_a, chunk_b) in a.chunks(8).zip(b.chunks(8)) {
        let mut ba = [0u8; 8];
        ba[..chunk_a.len()].copy_from_slice(chunk_a);
        let mut bb = [0u8; 8];
        bb[..chunk_b.len()].copy_from_slice(chunk_b);
        dist += (u64::from_le_bytes(ba) ^ u64::from_le_bytes(bb)).count_ones();
    }
    dist
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn hamming_distance_36_avx2(a: &[u8; SIG_BYTES], b: &[u8; SIG_BYTES]) -> u32 {
    use std::arch::x86_64::*;

    // Load first 32 bytes
    let va = _mm256_loadu_si256(a.as_ptr() as *const __m256i);
    let vb = _mm256_loadu_si256(b.as_ptr() as *const __m256i);
    let vxor = _mm256_xor_si256(va, vb);

    // popcount lookup table for lower nibble
    let lookup = _mm256_setr_epi8(
        0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4, 0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3,
        3, 4,
    );
    let low_mask = _mm256_set1_epi8(0x0f);

    let lo = _mm256_and_si256(vxor, low_mask);
    let hi = _mm256_and_si256(_mm256_srli_epi16(vxor, 4), low_mask);

    let pop_lo = _mm256_shuffle_epi8(lookup, lo);
    let pop_hi = _mm256_shuffle_epi8(lookup, hi);

    let pop = _mm256_add_epi8(pop_lo, pop_hi);

    // Sum absolute differences against 0 to sum bytes in 64-bit blocks
    let zero = _mm256_setzero_si256();
    let sum_64 = _mm256_sad_epu8(pop, zero);

    // sum_64 has 4 64-bit values containing sums. Extract them.
    let mut sum = [0u64; 4];
    _mm256_storeu_si256(sum.as_mut_ptr() as *mut __m256i, sum_64);
    let simd_pop = (sum[0] + sum[1] + sum[2] + sum[3]) as u32;

    // Add the remaining 4 bytes
    let rem_a = u32::from_le_bytes(a[32..36].try_into().unwrap());
    let rem_b = u32::from_le_bytes(b[32..36].try_into().unwrap());
    let rem_pop = (rem_a ^ rem_b).count_ones();

    simd_pop + rem_pop
}

#[cfg(target_arch = "aarch64")]
unsafe fn hamming_distance_36_neon(a: &[u8; SIG_BYTES], b: &[u8; SIG_BYTES]) -> u32 {
    use std::arch::aarch64::*;

    // 32 bytes = 2x 16-byte vectors
    let va1 = vld1q_u8(a.as_ptr());
    let vb1 = vld1q_u8(b.as_ptr());
    let va2 = vld1q_u8(a.as_ptr().add(16));
    let vb2 = vld1q_u8(b.as_ptr().add(16));

    let vxor1 = veorq_u8(va1, vb1);
    let vxor2 = veorq_u8(va2, vb2);

    let vcnt1 = vcntq_u8(vxor1);
    let vcnt2 = vcntq_u8(vxor2);

    let simd_pop = vaddvq_u8(vcnt1) as u32 + vaddvq_u8(vcnt2) as u32;

    // Add remaining 4 bytes
    let rem_a = u32::from_le_bytes(a[32..36].try_into().unwrap());
    let rem_b = u32::from_le_bytes(b[32..36].try_into().unwrap());
    let rem_pop = (rem_a ^ rem_b).count_ones();

    simd_pop + rem_pop
}

/// Dispatches to the fastest available Hamming distance implementation for 36-byte arrays.
#[inline(always)]
pub fn hamming_distance_36(a: &[u8; SIG_BYTES], b: &[u8; SIG_BYTES]) -> u32 {
    #[cfg(target_arch = "x86_64")]
    {
        if std::arch::is_x86_feature_detected!("avx2") {
            return unsafe { hamming_distance_36_avx2(a, b) };
        }
        hamming_distance_36_scalar(a, b)
    }

    #[cfg(target_arch = "aarch64")]
    {
        // NEON is baseline for aarch64 in Rust
        unsafe { hamming_distance_36_neon(a, b) }
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        hamming_distance_36_scalar(a, b)
    }
}

const DOT_LANES: usize = 4;
const DOT_GROUPS: usize = K / DOT_LANES;

/// One dimension's decoded shift/add terms for four adjacent classes.
///
/// This is a load-time expansion of the packed u16 table. It keeps the
/// deployed artifact unchanged while giving SIMD adapters contiguous shift
/// and sign metadata. `active` and `negative` use all-zero/all-one i64 masks.
#[derive(Clone, Copy)]
pub(crate) struct DotTermVector {
    shifts: [i64; DOT_LANES],
    active: [i64; DOT_LANES],
    negative: [i64; DOT_LANES],
}

#[derive(Clone, Copy)]
pub(crate) struct DotVector {
    /// Terms are ordered high byte then low byte, matching the scalar path.
    terms: [DotTermVector; 2],
}

/// Compact one-term representation used by current TLA6/TLA7 artifacts.
///
/// The packed ABI still reserves two terms per entry, but the Phase C
/// emission contains only one active term. Keeping the inactive term and
/// its masks in the load-time working set tripled the bytes fetched by the
/// hot scan. This representation preserves the same decoded shifts/signs
/// while making the common path cache-line dense.
#[derive(Clone, Copy)]
pub(crate) struct DotSingleVector {
    shifts: [i64; DOT_LANES],
    negative: [i64; DOT_LANES],
}

pub(crate) enum DotStageData {
    Single(Vec<DotSingleVector>),
    Two(Vec<DotVector>),
}

pub(crate) struct DotStage {
    /// Dimension-major: D vectors, each containing K/4 class groups.
    pub(crate) data: DotStageData,
}

impl DotStage {
    pub(crate) fn vector_count(&self) -> usize {
        match &self.data {
            DotStageData::Single(vectors) => vectors.len(),
            DotStageData::Two(vectors) => vectors.len(),
        }
    }
}

pub(crate) struct DotTables {
    pub(crate) stages: Vec<DotStage>,
}

fn decode_dot_term(term: u8) -> (i64, i64, i64) {
    if term & 0x40 == 0 {
        // The SIMD one-term path treats 64 as an out-of-range left shift,
        // which produces zero on both AVX2 and NEON. Keep `active == 0` for
        // the scalar and legacy two-term paths, where the sentinel is never
        // applied without its mask.
        return (64, 0, 0);
    }
    let exponent = i64::from(term & 0x3F) - 32;
    let negative = if term & 0x80 != 0 { -1 } else { 0 };
    (exponent, -1, negative)
}

/// True when no entry activates both packed bytes, i.e. the compiler emitted
/// at most one active term per dot entry (every current Phase C artifact).
fn is_single_term(table: &[u16]) -> bool {
    table.iter().all(|entry| {
        let [lo, hi] = entry.to_le_bytes();
        !(lo & 0x40 != 0 && hi & 0x40 != 0)
    })
}

fn build_single_term(table: &[u16]) -> Vec<DotSingleVector> {
    let mut vectors = Vec::with_capacity(D * DOT_GROUPS);
    for dimension in 0..D {
        for group in 0..DOT_GROUPS {
            let mut shifts = [0; DOT_LANES];
            let mut negative = [0; DOT_LANES];
            for lane in 0..DOT_LANES {
                let class = group * DOT_LANES + lane;
                let entry = table[class * D + dimension].to_le_bytes();
                let term = if entry[1] & 0x40 != 0 {
                    entry[1]
                } else {
                    entry[0]
                };
                let (shift, _, sign) = decode_dot_term(term);
                shifts[lane] = shift;
                negative[lane] = sign;
            }
            vectors.push(DotSingleVector { shifts, negative });
        }
    }
    vectors
}

fn build_two_term(table: &[u16]) -> Vec<DotVector> {
    let mut vectors = Vec::with_capacity(D * DOT_GROUPS);
    for dimension in 0..D {
        for group in 0..DOT_GROUPS {
            let mut terms = [DotTermVector {
                shifts: [0; DOT_LANES],
                active: [0; DOT_LANES],
                negative: [0; DOT_LANES],
            }; 2];
            for lane in 0..DOT_LANES {
                let class = group * DOT_LANES + lane;
                let entry = table[class * D + dimension].to_le_bytes();
                let (hi_shift, hi_active, hi_negative) = decode_dot_term(entry[1]);
                let (lo_shift, lo_active, lo_negative) = decode_dot_term(entry[0]);
                terms[0].shifts[lane] = hi_shift;
                terms[0].active[lane] = hi_active;
                terms[0].negative[lane] = hi_negative;
                terms[1].shifts[lane] = lo_shift;
                terms[1].active[lane] = lo_active;
                terms[1].negative[lane] = lo_negative;
            }
            vectors.push(DotVector { terms });
        }
    }
    vectors
}

impl DotTables {
    pub(crate) fn from_packed(packed: &[Vec<u16>]) -> Option<Self> {
        Self::build(packed, false)
    }

    /// Build every stage in the legacy two-term representation regardless of
    /// what the artifact would select, so the #330 benchmark can compare both
    /// layouts in one process under one protocol instead of across two
    /// commits — and so the tests can witness that the two representations
    /// decode the same packed ABI to the same class.
    ///
    /// Not reachable from the deployed path: `from_packed` is what the runtime
    /// calls, and this is gated to tests and the measurement feature.
    #[cfg(any(test, feature = "bench-internals"))]
    pub(crate) fn from_packed_forced_two_term(packed: &[Vec<u16>]) -> Option<Self> {
        Self::build(packed, true)
    }

    fn build(packed: &[Vec<u16>], force_two_term: bool) -> Option<Self> {
        if packed.is_empty() {
            return None;
        }
        let mut stages = Vec::with_capacity(packed.len());
        for table in packed {
            if table.len() != D * K {
                return None;
            }
            let data = if !force_two_term && is_single_term(table) {
                DotStageData::Single(build_single_term(table))
            } else {
                DotStageData::Two(build_two_term(table))
            };
            stages.push(DotStage { data });
        }
        Some(Self { stages })
    }
}

#[cfg(not(target_arch = "aarch64"))]
#[inline]
fn dot_term_scalar_decoded(work: i64, shift: i64, negative: i64) -> i64 {
    if shift == 64 {
        return 0;
    }
    let shifted = if shift >= 0 {
        work << shift
    } else {
        work >> -shift
    };
    if negative != 0 {
        -shifted
    } else {
        shifted
    }
}

#[cfg(not(target_arch = "aarch64"))]
#[inline]
fn dot_term_scalar(work: i64, term: &DotTermVector, lane: usize) -> i64 {
    if term.active[lane] == 0 {
        return 0;
    }
    dot_term_scalar_decoded(work, term.shifts[lane], term.negative[lane])
}

#[cfg(not(target_arch = "aarch64"))]
pub(crate) fn dot_argmax_scalar(stage: &DotStage, work: &[i64; D]) -> u8 {
    let mut scores = [0i64; K];
    match &stage.data {
        DotStageData::Single(vectors) => {
            for (dimension, groups) in vectors.chunks_exact(DOT_GROUPS).enumerate() {
                for (group, vector) in groups.iter().enumerate() {
                    for lane in 0..DOT_LANES {
                        let class = group * DOT_LANES + lane;
                        scores[class] += dot_term_scalar_decoded(
                            work[dimension],
                            vector.shifts[lane],
                            vector.negative[lane],
                        );
                    }
                }
            }
        }
        DotStageData::Two(vectors) => {
            for (dimension, groups) in vectors.chunks_exact(DOT_GROUPS).enumerate() {
                for (group, vector) in groups.iter().enumerate() {
                    for lane in 0..DOT_LANES {
                        let class = group * DOT_LANES + lane;
                        scores[class] += dot_term_scalar(work[dimension], &vector.terms[0], lane);
                        scores[class] += dot_term_scalar(work[dimension], &vector.terms[1], lane);
                    }
                }
            }
        }
    }
    let mut best_class = 0u8;
    let mut best_score = i64::MIN;
    for (class, &score) in scores.iter().enumerate() {
        if score > best_score {
            best_score = score;
            best_class = class as u8;
        }
    }
    best_class
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn dot_argmax_avx2_terms(vectors: &[DotVector], work: &[i64; D]) -> u8 {
    use std::arch::x86_64::*;

    let zero = _mm256_setzero_si256();
    let all_ones = _mm256_set1_epi64x(-1);
    let sixty_four = _mm256_set1_epi64x(64);
    let mut accumulators = [zero; DOT_GROUPS];
    for (dimension, groups) in vectors.chunks_exact(DOT_GROUPS).enumerate() {
        let work_vector = _mm256_set1_epi64x(work[dimension]);
        for (group, vector) in groups.iter().enumerate() {
            for term in &vector.terms {
                let shifts = _mm256_loadu_si256(term.shifts.as_ptr() as *const __m256i);
                let active = _mm256_loadu_si256(term.active.as_ptr() as *const __m256i);
                let negative = _mm256_loadu_si256(term.negative.as_ptr() as *const __m256i);
                let is_right = _mm256_cmpgt_epi64(zero, shifts);
                let absolute_shift = _mm256_sub_epi64(zero, shifts);
                let left = _mm256_sllv_epi64(work_vector, shifts);
                let logical_right = _mm256_srlv_epi64(work_vector, absolute_shift);
                let sign = _mm256_cmpgt_epi64(zero, work_vector);
                let fill_shift = _mm256_sub_epi64(sixty_four, absolute_shift);
                let fill = _mm256_sllv_epi64(all_ones, fill_shift);
                let arithmetic_right = _mm256_or_si256(logical_right, _mm256_and_si256(sign, fill));
                let shifted = _mm256_blendv_epi8(left, arithmetic_right, is_right);
                let negated = _mm256_sub_epi64(zero, shifted);
                let signed = _mm256_blendv_epi8(shifted, negated, negative);
                let active_value = _mm256_blendv_epi8(zero, signed, active);
                accumulators[group] = _mm256_add_epi64(accumulators[group], active_value);
            }
        }
    }

    let mut scores = [0i64; K];
    for (group, accumulator) in accumulators.iter().enumerate() {
        let mut lanes = [0i64; DOT_LANES];
        _mm256_storeu_si256(lanes.as_mut_ptr() as *mut __m256i, *accumulator);
        for (lane, &score) in lanes.iter().enumerate() {
            scores[group * DOT_LANES + lane] = score;
        }
    }
    let mut best_class = 0u8;
    let mut best_score = i64::MIN;
    for (class, &score) in scores.iter().enumerate() {
        if score > best_score {
            best_score = score;
            best_class = class as u8;
        }
    }
    best_class
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn dot_argmax_avx2_single_term(vectors: &[DotSingleVector], work: &[i64; D]) -> u8 {
    use std::arch::x86_64::*;

    let zero = _mm256_setzero_si256();
    let all_ones = _mm256_set1_epi64x(-1);
    let sixty_four = _mm256_set1_epi64x(64);
    let mut accumulators = [zero; DOT_GROUPS];
    for (dimension, groups) in vectors.chunks_exact(DOT_GROUPS).enumerate() {
        let work_vector = _mm256_set1_epi64x(work[dimension]);
        for (group, vector) in groups.iter().enumerate() {
            let shifts = _mm256_loadu_si256(vector.shifts.as_ptr() as *const __m256i);
            let negative = _mm256_loadu_si256(vector.negative.as_ptr() as *const __m256i);
            let is_right = _mm256_cmpgt_epi64(zero, shifts);
            let absolute_shift = _mm256_sub_epi64(zero, shifts);
            let left = _mm256_sllv_epi64(work_vector, shifts);
            let logical_right = _mm256_srlv_epi64(work_vector, absolute_shift);
            let sign = _mm256_cmpgt_epi64(zero, work_vector);
            let fill_shift = _mm256_sub_epi64(sixty_four, absolute_shift);
            let fill = _mm256_sllv_epi64(all_ones, fill_shift);
            let arithmetic_right = _mm256_or_si256(logical_right, _mm256_and_si256(sign, fill));
            let shifted = _mm256_blendv_epi8(left, arithmetic_right, is_right);
            let negated = _mm256_sub_epi64(zero, shifted);
            let signed = _mm256_blendv_epi8(shifted, negated, negative);
            accumulators[group] = _mm256_add_epi64(accumulators[group], signed);
        }
    }

    let mut best_class = 0u8;
    let mut best_score = i64::MIN;
    for (group, accumulator) in accumulators.iter().enumerate() {
        let mut lanes = [0i64; DOT_LANES];
        _mm256_storeu_si256(lanes.as_mut_ptr() as *mut __m256i, *accumulator);
        for (lane, &score) in lanes.iter().enumerate() {
            let class = group * DOT_LANES + lane;
            if score > best_score {
                best_score = score;
                best_class = class as u8;
            }
        }
    }
    best_class
}

#[cfg(target_arch = "aarch64")]
unsafe fn dot_argmax_neon_terms(vectors: &[DotVector], work: &[i64; D]) -> u8 {
    use std::arch::aarch64::*;

    let zero = vdupq_n_s64(0);
    let mut accumulators_a = [zero; DOT_GROUPS];
    let mut accumulators_b = [zero; DOT_GROUPS];
    for (dimension, groups) in vectors.chunks_exact(DOT_GROUPS).enumerate() {
        let work_vector = vdupq_n_s64(work[dimension]);
        for (group, vector) in groups.iter().enumerate() {
            for term in &vector.terms {
                let shifts_a = vld1q_s64(term.shifts.as_ptr());
                let shifts_b = vld1q_s64(term.shifts.as_ptr().add(2));
                let active_a = vld1q_s64(term.active.as_ptr());
                let active_b = vld1q_s64(term.active.as_ptr().add(2));
                let negative_a = vld1q_s64(term.negative.as_ptr());
                let negative_b = vld1q_s64(term.negative.as_ptr().add(2));
                let shifted_a = vshlq_s64(work_vector, shifts_a);
                let shifted_b = vshlq_s64(work_vector, shifts_b);
                let signed_a = vbslq_s64(
                    vreinterpretq_u64_s64(negative_a),
                    vnegq_s64(shifted_a),
                    shifted_a,
                );
                let signed_b = vbslq_s64(
                    vreinterpretq_u64_s64(negative_b),
                    vnegq_s64(shifted_b),
                    shifted_b,
                );
                accumulators_a[group] = vaddq_s64(
                    accumulators_a[group],
                    vbslq_s64(vreinterpretq_u64_s64(active_a), signed_a, zero),
                );
                accumulators_b[group] = vaddq_s64(
                    accumulators_b[group],
                    vbslq_s64(vreinterpretq_u64_s64(active_b), signed_b, zero),
                );
            }
        }
    }

    let mut scores = [0i64; K];
    for group in 0..DOT_GROUPS {
        scores[group * DOT_LANES] = vget_lane_s64(vget_low_s64(accumulators_a[group]), 0);
        scores[group * DOT_LANES + 1] = vget_lane_s64(vget_high_s64(accumulators_a[group]), 0);
        scores[group * DOT_LANES + 2] = vget_lane_s64(vget_low_s64(accumulators_b[group]), 0);
        scores[group * DOT_LANES + 3] = vget_lane_s64(vget_high_s64(accumulators_b[group]), 0);
    }
    let mut best_class = 0u8;
    let mut best_score = i64::MIN;
    for (class, &score) in scores.iter().enumerate() {
        if score > best_score {
            best_score = score;
            best_class = class as u8;
        }
    }
    best_class
}

#[cfg(target_arch = "aarch64")]
unsafe fn dot_argmax_neon_single_term(vectors: &[DotSingleVector], work: &[i64; D]) -> u8 {
    use std::arch::aarch64::*;

    let zero = vdupq_n_s64(0);
    let mut accumulators_a = [zero; DOT_GROUPS];
    let mut accumulators_b = [zero; DOT_GROUPS];
    for (dimension, groups) in vectors.chunks_exact(DOT_GROUPS).enumerate() {
        let work_vector = vdupq_n_s64(work[dimension]);
        for (group, vector) in groups.iter().enumerate() {
            let shifts_a = vld1q_s64(vector.shifts.as_ptr());
            let shifts_b = vld1q_s64(vector.shifts.as_ptr().add(2));
            let negative_a = vld1q_s64(vector.negative.as_ptr());
            let negative_b = vld1q_s64(vector.negative.as_ptr().add(2));
            let shifted_a = vshlq_s64(work_vector, shifts_a);
            let shifted_b = vshlq_s64(work_vector, shifts_b);
            let signed_a = vbslq_s64(
                vreinterpretq_u64_s64(negative_a),
                vnegq_s64(shifted_a),
                shifted_a,
            );
            let signed_b = vbslq_s64(
                vreinterpretq_u64_s64(negative_b),
                vnegq_s64(shifted_b),
                shifted_b,
            );
            accumulators_a[group] = vaddq_s64(accumulators_a[group], signed_a);
            accumulators_b[group] = vaddq_s64(accumulators_b[group], signed_b);
        }
    }

    let mut best_class = 0u8;
    let mut best_score = i64::MIN;
    for group in 0..DOT_GROUPS {
        let lanes = [
            vget_lane_s64(vget_low_s64(accumulators_a[group]), 0),
            vget_lane_s64(vget_high_s64(accumulators_a[group]), 0),
            vget_lane_s64(vget_low_s64(accumulators_b[group]), 0),
            vget_lane_s64(vget_high_s64(accumulators_b[group]), 0),
        ];
        for (lane, &score) in lanes.iter().enumerate() {
            let class = group * DOT_LANES + lane;
            if score > best_score {
                best_score = score;
                best_class = class as u8;
            }
        }
    }
    best_class
}

#[cfg(target_arch = "x86_64")]
pub(crate) fn dot_argmax(stage: &DotStage, work: &[i64; D]) -> u8 {
    if std::arch::is_x86_feature_detected!("avx2") {
        return unsafe {
            match &stage.data {
                DotStageData::Single(vectors) => dot_argmax_avx2_single_term(vectors, work),
                DotStageData::Two(vectors) => dot_argmax_avx2_terms(vectors, work),
            }
        };
    }
    dot_argmax_scalar(stage, work)
}

#[cfg(target_arch = "aarch64")]
pub(crate) fn dot_argmax(stage: &DotStage, work: &[i64; D]) -> u8 {
    unsafe {
        match &stage.data {
            DotStageData::Single(vectors) => dot_argmax_neon_single_term(vectors, work),
            DotStageData::Two(vectors) => dot_argmax_neon_terms(vectors, work),
        }
    }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub(crate) fn dot_argmax(stage: &DotStage, work: &[i64; D]) -> u8 {
    dot_argmax_scalar(stage, work)
}

/// Measurement-only handle onto the built dot tables (#330).
///
/// The isolated microbenchmark criterion needs to time `dot_argmax` without
/// the surrounding bundle/threshold work, and the scalar fallback inside
/// `Runtime::dot_argmax` is not a usable wall-clock baseline (it routes every
/// table entry through the op-census kernel). This module exposes the scan
/// itself plus the layout actually selected at load time, so the benchmark can
/// witness which representation it measured rather than assuming.
///
/// Nothing here participates in the deployed prediction path.
#[cfg(feature = "bench-internals")]
pub mod bench {
    use super::{dot_argmax, DotStageData, DotTables, D};

    /// Which load-time representation `from_packed` selected.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum DotLayout {
        /// Compact 64-byte one-term vectors (current TLA6/TLA7 artifacts).
        Compact,
        /// Legacy 192-byte two-term vectors.
        TwoTerm,
    }

    pub struct DotScan(DotTables);

    impl DotScan {
        pub fn from_packed(packed: &[Vec<u16>]) -> Option<Self> {
            DotTables::from_packed(packed).map(Self)
        }

        /// Force the legacy two-term layout for a same-process,
        /// same-protocol comparison against the compact one.
        pub fn from_packed_forced_two_term(packed: &[Vec<u16>]) -> Option<Self> {
            DotTables::from_packed_forced_two_term(packed).map(Self)
        }

        pub fn stage_count(&self) -> usize {
            self.0.stages.len()
        }

        /// The representation chosen for `stage`, so a benchmark can assert it
        /// exercised the compact path instead of silently timing the legacy one.
        pub fn layout(&self, stage: usize) -> DotLayout {
            match &self.0.stages[stage].data {
                DotStageData::Single(_) => DotLayout::Compact,
                DotStageData::Two(_) => DotLayout::TwoTerm,
            }
        }

        #[inline]
        pub fn argmax(&self, stage: usize, work: &[i64; D]) -> u8 {
            dot_argmax(&self.0.stages[stage], work)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transformerless::compiler;
    use proptest::prelude::*;

    #[test]
    fn one_term_tables_use_compact_load_layout() {
        let table = vec![compiler::pack_dot_entry(0.5); D * K];
        let packed = vec![table];
        let tables = DotTables::from_packed(&packed).expect("valid dot table");
        assert!(matches!(&tables.stages[0].data, DotStageData::Single(_)));
        assert_eq!(tables.stages[0].vector_count(), D * DOT_GROUPS);
        assert_eq!(std::mem::size_of::<DotSingleVector>(), 64);
        assert_eq!(std::mem::size_of::<DotVector>(), 192);
    }

    #[test]
    fn two_term_tables_keep_legacy_load_layout() {
        let high = compiler::pack_dot_entry(0.5).to_le_bytes()[1];
        let low = compiler::pack_dot_entry(0.25).to_le_bytes()[1];
        let entry = u16::from_le_bytes([low, high]);
        let table = vec![entry; D * K];
        let packed = vec![table];
        let tables = DotTables::from_packed(&packed).expect("valid dot table");
        assert!(matches!(&tables.stages[0].data, DotStageData::Two(_)));
        assert_eq!(tables.stages[0].vector_count(), D * DOT_GROUPS);
    }

    proptest! {
        /// The compact one-term layout is only a valid substitution if it
        /// decodes the same packed ABI to the same argmax as the legacy
        /// two-term layout. #330 swaps representations on the hot path, so
        /// this equality is the load-layout counterpart to the existing
        /// scalar/SIMD output witnesses.
        #[test]
        fn compact_and_legacy_layouts_agree(
            weights in proptest::collection::vec(-1.0f32..1.0, K),
            work_seed in proptest::collection::vec(-4096i64..4096, D),
        ) {
            let mut table = vec![0u16; D * K];
            for class in 0..K {
                for dimension in 0..D {
                    table[class * D + dimension] =
                        compiler::pack_dot_entry(weights[class] * ((dimension % 7) as f32 - 3.0));
                }
            }
            let packed = vec![table];

            let compact = DotTables::from_packed(&packed).expect("valid dot table");
            let legacy =
                DotTables::from_packed_forced_two_term(&packed).expect("valid dot table");
            prop_assert!(matches!(&compact.stages[0].data, DotStageData::Single(_)));
            prop_assert!(matches!(&legacy.stages[0].data, DotStageData::Two(_)));

            let mut work = [0i64; D];
            work.copy_from_slice(&work_seed);

            prop_assert_eq!(
                dot_argmax(&compact.stages[0], &work),
                dot_argmax(&legacy.stages[0], &work)
            );
        }

        #[test]
        fn test_simd_hamming_equivalence(
            a in proptest::collection::vec(any::<u8>(), SIG_BYTES).prop_map(|v| {
                let mut arr = [0u8; SIG_BYTES];
                arr.copy_from_slice(&v);
                arr
            }),
            b in proptest::collection::vec(any::<u8>(), SIG_BYTES).prop_map(|v| {
                let mut arr = [0u8; SIG_BYTES];
                arr.copy_from_slice(&v);
                arr
            })
        ) {
            let scalar = hamming_distance_36_scalar(&a, &b);
            let dispatched = hamming_distance_36(&a, &b);

            #[cfg(target_arch = "x86_64")]
            if std::arch::is_x86_feature_detected!("avx2") {
                let avx2 = unsafe { hamming_distance_36_avx2(&a, &b) };
                assert_eq!(scalar, avx2, "AVX2 implementation differs from scalar");
            }

            #[cfg(target_arch = "aarch64")]
            {
                let neon = unsafe { hamming_distance_36_neon(&a, &b) };
                assert_eq!(scalar, neon, "NEON implementation differs from scalar");
            }

            assert_eq!(scalar, dispatched, "Dispatched implementation differs from scalar");
        }
    }
}
