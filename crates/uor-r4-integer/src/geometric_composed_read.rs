//! Versioned wide-Q16 reduction for the fixed geometric composition bank.
//!
//! This component reuses the unchanged legacy reducer at width one with a
//! preallocated zero payload to obtain EXACTLY its score+age/exp/NoRead weights.
//! The zero scratch is not a semantic value or a new occurrence. The admitted
//! i64 Q16 payload is then mixed using checked i128 shift/add products and the
//! existing restoring unsigned division, with nearest/ties-away output rounding.
//!
//! Up to128 occurrences, |payload|<=28*2^30 and each weight<=2^31 imply
//! |numerator|<=28*2^68<2^73. The denominator is positive and at most129*2^31.
//! Each result is a convex combination of admitted payloads and NoRead's zero;
//! it fits the same bound. Normalize heads independently, then use sum_heads.
//! No probability rounding, support truncation, clipping or i32 cast is added.
//!
//! Construction allocates; reduce has no allocation, float or source-level
//! multiplication/division. Scoped compiled-instruction evidence is separate.

use std::fmt;

use crate::geometric_composition::{MAX_COMPOSED_Q16, OUTPUT_WIDTH};
use crate::geometric_read::{NativeGeometricRead, ReadError};
use crate::stack::kernels::stack_div_u128;

pub const SCHEMA: &str = "uor-r4.geometric-composed-read-i64-q16/1";
pub const MAX_CONTEXT: usize = 128;
pub const VALUE_FRACTIONAL_BITS: u32 = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComposedReadError {
    InvalidContext(usize),
    TooManyOccurrences { actual: usize, maximum: usize },
    ValueLength { expected: usize, actual: usize },
    ValueBound { coordinate: usize, value: i64 },
    Legacy(ReadError),
    ArithmeticOverflow,
}
impl fmt::Display for ComposedReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "geometric composed read: {self:?}")
    }
}
impl std::error::Error for ComposedReadError {}
pub type ComposedReadResult<T> = Result<T, ComposedReadError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComposedReadReduction<'a> {
    pub output_q16: &'a [i64],
    pub occurrence_weights_q31: &'a [u64],
    pub no_read_weight_q31: u64,
    pub total_weight_q31: u64,
    pub max_score_q24: i64,
}
#[derive(Clone, Copy, Debug)]
struct Summary {
    occurrences: usize,
    no_read_weight_q31: u64,
    total_weight_q31: u64,
    max_score_q24: i64,
}

/// Public output and trace are committed together only after a successful
/// reduction. A rejected call leaves last() unchanged, including its weights.
#[derive(Debug)]
pub struct NativeGeometricComposedRead {
    legacy: NativeGeometricRead,
    zeros: [i32; MAX_CONTEXT],
    weights: [u64; MAX_CONTEXT],
    output: [i64; OUTPUT_WIDTH],
    last: Option<Summary>,
}

impl NativeGeometricComposedRead {
    pub fn new(max_context: usize, exp_table: &[u32]) -> ComposedReadResult<Self> {
        if !(1..=MAX_CONTEXT).contains(&max_context) {
            return Err(ComposedReadError::InvalidContext(max_context));
        }
        Ok(Self {
            legacy: NativeGeometricRead::new(max_context, 1, exp_table)
                .map_err(ComposedReadError::Legacy)?,
            zeros: [0; MAX_CONTEXT],
            weights: [0; MAX_CONTEXT],
            output: [0; OUTPUT_WIDTH],
            last: None,
        })
    }
    pub fn max_context(&self) -> usize {
        self.legacy.max_context()
    }
    pub fn value_width(&self) -> usize {
        OUTPUT_WIDTH
    }
    pub fn last(&self) -> Option<ComposedReadReduction<'_>> {
        self.last.map(|last| ComposedReadReduction {
            output_q16: &self.output,
            occurrence_weights_q31: &self.weights[..last.occurrences],
            no_read_weight_q31: last.no_read_weight_q31,
            total_weight_q31: last.total_weight_q31,
            max_score_q24: last.max_score_q24,
        })
    }

    /// Occurrence-major [N,32] i64 Q16 values, with only the actual causal
    /// prefix supplied. Equal payload/address occurrences remain separate.
    #[inline(never)]
    pub fn reduce(
        &mut self,
        potential_q24: &[i64],
        age_q24: &[i64],
        no_read_q24: i64,
        values_q16: &[i64],
    ) -> ComposedReadResult<ComposedReadReduction<'_>> {
        let count = potential_q24.len();
        if count > self.max_context() {
            return Err(ComposedReadError::TooManyOccurrences {
                actual: count,
                maximum: self.max_context(),
            });
        }
        let expected = count << 5;
        if values_q16.len() != expected {
            return Err(ComposedReadError::ValueLength {
                expected,
                actual: values_q16.len(),
            });
        }
        for (coordinate, &value) in values_q16.iter().enumerate() {
            if !(-MAX_COMPOSED_Q16..=MAX_COMPOSED_Q16).contains(&value) {
                return Err(ComposedReadError::ValueBound { coordinate, value });
            }
        }
        let legacy = self
            .legacy
            .reduce(potential_q24, age_q24, no_read_q24, &self.zeros[..count])
            .map_err(ComposedReadError::Legacy)?;
        let mut mix = [0_i128; OUTPUT_WIDTH];
        for (&weight, row) in legacy
            .occurrence_weights_q31
            .iter()
            .zip(values_q16.chunks_exact(OUTPUT_WIDTH))
        {
            if weight != 0 {
                for (sum, &value) in mix.iter_mut().zip(row) {
                    *sum = weighted_add(*sum, weight, value)?;
                }
            }
        }
        let mut output = [0_i64; OUTPUT_WIDTH];
        let denominator = u128::from(legacy.total_weight_q31);
        for (coordinate, (&numerator, out)) in mix.iter().zip(output.iter_mut()).enumerate() {
            let rounded_numerator = numerator
                .unsigned_abs()
                .checked_add(denominator >> 1)
                .ok_or(ComposedReadError::ArithmeticOverflow)?;
            let magnitude = stack_div_u128(rounded_numerator, denominator);
            let signed_magnitude =
                i64::try_from(magnitude).map_err(|_| ComposedReadError::ArithmeticOverflow)?;
            *out = if numerator < 0 {
                -signed_magnitude
            } else {
                signed_magnitude
            };
            if !(-MAX_COMPOSED_Q16..=MAX_COMPOSED_Q16).contains(out) {
                return Err(ComposedReadError::ValueBound {
                    coordinate,
                    value: *out,
                });
            }
        }
        // Staged output and independent trace protect last() even if an
        // internal checked-arithmetic invariant were violated above.
        self.output = output;
        self.weights[..count].copy_from_slice(legacy.occurrence_weights_q31);
        let last = Summary {
            occurrences: count,
            no_read_weight_q31: legacy.no_read_weight_q31,
            total_weight_q31: legacy.total_weight_q31,
            max_score_q24: legacy.max_score_q24,
        };
        self.last = Some(last);
        Ok(ComposedReadReduction {
            output_q16: &self.output,
            occurrence_weights_q31: &self.weights[..count],
            no_read_weight_q31: last.no_read_weight_q31,
            total_weight_q31: last.total_weight_q31,
            max_score_q24: last.max_score_q24,
        })
    }
}

/// Crucially the product is i128 FROM THE FIRST ADD. The legacy i32 payload
/// mix helper forms an i64 product and is not suitable for this wider range.
fn weighted_add(mut accumulator: i128, mut weight: u64, value: i64) -> ComposedReadResult<i128> {
    let mut term = i128::from(value);
    while weight != 0 {
        if weight & 1 != 0 {
            accumulator = accumulator
                .checked_add(term)
                .ok_or(ComposedReadError::ArithmeticOverflow)?;
        }
        weight >>= 1;
        if weight != 0 {
            term = term
                .checked_add(term)
                .ok_or(ComposedReadError::ArithmeticOverflow)?;
        }
    }
    Ok(accumulator)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_read::{EXP_TABLE_LEN, WEIGHT_ONE};
    type TestResult = Result<(), Box<dyn std::error::Error>>;
    fn table() -> Vec<u32> {
        let mut t = vec![0; EXP_TABLE_LEN];
        t[0] = WEIGHT_ONE as u32;
        t[1] = (WEIGHT_ONE >> 1) as u32;
        t
    }
    fn owned(x: ComposedReadReduction<'_>) -> (Vec<i64>, Vec<u64>, u64, u64, i64) {
        (
            x.output_q16.to_vec(),
            x.occurrence_weights_q31.to_vec(),
            x.no_read_weight_q31,
            x.total_weight_q31,
            x.max_score_q24,
        )
    }

    #[test]
    fn native_geometric_composed_read_exact_legacy_weights_and_narrow_outputs() -> TestResult {
        let t = table();
        let mut wide = NativeGeometricComposedRead::new(8, &t)?;
        let mut old = NativeGeometricRead::new(8, OUTPUT_WIDTH, &t)?;
        let step = 1_i64 << 16;
        let scores = [0, -step, -step, -(1_i64 << 48)];
        let ages = [0, 0, step >> 1, 0];
        let values: Vec<i32> = (0..4 * OUTPUT_WIDTH)
            .map(|i| {
                if i % 3 == 0 {
                    i32::MIN
                } else {
                    (i as i32) * 37 - 129
                }
            })
            .collect();
        let larger: Vec<i64> = values.iter().copied().map(i64::from).collect();
        let a = wide.reduce(&scores, &ages, -step, &larger)?;
        let b = old.reduce(&scores, &ages, -step, &values)?;
        assert_eq!(a.occurrence_weights_q31, b.occurrence_weights_q31);
        assert_eq!(a.no_read_weight_q31, b.no_read_weight_q31);
        assert_eq!(a.total_weight_q31, b.total_weight_q31);
        assert_eq!(a.max_score_q24, b.max_score_q24);
        assert!(a
            .output_q16
            .iter()
            .zip(b.output_q16)
            .all(|(&x, &y)| x == i64::from(y)));
        Ok(())
    }

    #[test]
    fn native_geometric_composed_read_upper_radius_128_and_exact_host_reference() -> TestResult {
        let mut wide = NativeGeometricComposedRead::new(MAX_CONTEXT, &table())?;
        let scores = [0; MAX_CONTEXT];
        let values: Vec<i64> = (0..MAX_CONTEXT * OUTPUT_WIDTH)
            .map(|i| {
                if i % 2 == 0 {
                    MAX_COMPOSED_Q16
                } else {
                    -MAX_COMPOSED_Q16
                }
            })
            .collect();
        let result = wide.reduce(&scores, &scores, 0, &values)?;
        assert_eq!(result.total_weight_q31, 129 * WEIGHT_ONE);
        assert_eq!(result.no_read_weight_q31, WEIGHT_ONE);
        assert_eq!(result.occurrence_weights_q31, &[WEIGHT_ONE; MAX_CONTEXT]);
        for (column, &actual) in result.output_q16.iter().enumerate() {
            // Host multiplication/division are an independent test oracle,
            // never part of the served weighted-product implementation.
            let numerator: i128 = (0..MAX_CONTEXT)
                .map(|row| i128::from(WEIGHT_ONE) * i128::from(values[row * OUTPUT_WIDTH + column]))
                .sum();
            assert!(numerator.unsigned_abs() < (1_u128 << 73));
            assert!(numerator.unsigned_abs() > u128::from(u64::MAX));
            let denominator = u128::from(result.total_weight_q31);
            let magnitude = (numerator.unsigned_abs() + denominator / 2) / denominator;
            let expected = if numerator < 0 {
                -(magnitude as i64)
            } else {
                magnitude as i64
            };
            assert_eq!(actual, expected);
        }
        let short = wide.reduce(&[0], &[0], -(1_i64 << 48), &values[..OUTPUT_WIDTH])?;
        assert_eq!(short.no_read_weight_q31, 0);
        assert_eq!(short.output_q16, &values[..OUTPUT_WIDTH]);
        assert_eq!(short.occurrence_weights_q31.len(), 1);
        Ok(())
    }

    #[test]
    fn native_geometric_composed_read_null_cancellation_ties_and_distinct_denominators(
    ) -> TestResult {
        let mut wide = NativeGeometricComposedRead::new(2, &table())?;
        let empty = wide.reduce(&[], &[], i64::MIN, &[])?;
        assert_eq!(empty.output_q16, &[0; OUTPUT_WIDTH]);
        assert_eq!(empty.no_read_weight_q31, WEIGHT_ONE);
        let alternating: Vec<i64> = (0..OUTPUT_WIDTH)
            .map(|i| if i % 2 == 0 { 1 } else { -1 })
            .collect();
        assert_eq!(
            wide.reduce(&[0], &[0], 0, &alternating)?.output_q16,
            alternating
        );
        let cancel: Vec<i64> = [
            vec![MAX_COMPOSED_Q16; OUTPUT_WIDTH],
            vec![-MAX_COMPOSED_Q16; OUTPUT_WIDTH],
        ]
        .concat();
        let result = wide.reduce(&[0, 0], &[0, 0], 0, &cancel)?;
        assert_eq!(result.output_q16, &[0; OUTPUT_WIDTH]);
        assert_eq!(result.total_weight_q31, 3 * WEIGHT_ONE);
        assert_eq!(result.occurrence_weights_q31.len(), 2);
        let first: [i64; OUTPUT_WIDTH] = wide
            .reduce(&[0], &[0], 0, &[9; OUTPUT_WIDTH])?
            .output_q16
            .try_into()?;
        let second: [i64; OUTPUT_WIDTH] = wide
            .reduce(&[0, 0], &[0, 0], 0, &[9; OUTPUT_WIDTH << 1])?
            .output_q16
            .try_into()?;
        assert_eq!(
            crate::geometric_composition::sum_heads(&first, &second)?,
            [11; OUTPUT_WIDTH]
        ); // round(9/2)+round(18/3)
        Ok(())
    }

    #[test]
    fn native_geometric_composed_read_errors_preserve_public_output_and_trace() -> TestResult {
        let t = table();
        for context in [0, MAX_CONTEXT + 1, usize::MAX] {
            assert!(NativeGeometricComposedRead::new(context, &t).is_err());
        }
        assert!(NativeGeometricComposedRead::new(1, &t[..EXP_TABLE_LEN - 1]).is_err());
        let mut wide = NativeGeometricComposedRead::new(2, &t)?;
        let before = owned(wide.reduce(&[0], &[0], 0, &[MAX_COMPOSED_Q16; OUTPUT_WIDTH])?);
        let cases: Vec<(Vec<i64>, Vec<i64>, i64, Vec<i64>)> = vec![
            (vec![0; 3], vec![0; 3], 0, vec![0; 3 * OUTPUT_WIDTH]),
            (vec![0], vec![], 0, vec![0; OUTPUT_WIDTH]),
            (vec![0], vec![0], 0, vec![]),
            (
                vec![0],
                vec![0],
                0,
                vec![MAX_COMPOSED_Q16 + 1; OUTPUT_WIDTH],
            ),
            (vec![0], vec![0], 0, vec![i64::MIN; OUTPUT_WIDTH]),
            (vec![i64::MAX], vec![1], 0, vec![0; OUTPUT_WIDTH]),
            (vec![i64::MIN], vec![0], 0, vec![0; OUTPUT_WIDTH]),
            (vec![0], vec![0], i64::MIN, vec![0; OUTPUT_WIDTH]),
        ];
        for (scores, age, null, values) in cases {
            assert!(wide.reduce(&scores, &age, null, &values).is_err());
            assert_eq!(owned(wide.last().ok_or("missing retained output")?), before);
        }
        Ok(())
    }
}
