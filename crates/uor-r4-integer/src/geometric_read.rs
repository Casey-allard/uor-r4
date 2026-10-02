//! Bounded native reduction of geometric read scores and occurrence payloads.
//!
//! Scores and age are Q24; payloads and output are signed Q16. NoRead is an
//! explicit score with a zero payload and participates in the denominator.
//! The caller supplies only the admitted causal prefix, in occurrence order;
//! equal addresses or equal payloads do not merge occurrences. This component
//! does not produce addresses, payloads, NoRead logits or output projections.
//!
//! The fixed exp-table grid matches the existing stack exporter: 8,194 Q31
//! entries, spacing 2^-8, covering 32 nats plus an interpolation endpoint.
//! The existing interpolation subtracts a truncated nonnegative decrement:
//! it rounds the interpolated Q31 value upward by less than one integer unit.
//! Values beyond the table return zero. The enclosing artifact loader
//! binds the exact table bytes; shape/order checks do not authenticate them or
//! establish that arbitrary admitted entries approximate exp.
//!
//! Reduction uses the maximum score, unnormalized Q31 weights and an exact
//! i128 weighted sum. One exact restoring division per output coordinate rounds
//! to nearest, ties away from zero. No reciprocal approximation or probability
//! rounding occurs before payload mixing. The maximum-scoring slot has weight
//! 2^31, so the denominator is positive without a fallback.
//!
//! Admission allocates all storage. `reduce` performs no floating arithmetic,
//! multiplication, division instruction or allocation in source: runtime
//! products and division reuse shift/add/table and restoring-division kernels.
//! Compiled instruction evidence and complete-model qualification are separate.

use std::fmt;

use crate::stack::kernels::{stack_div_u128, stack_exp_neg, stack_mix_row};

pub const MAX_CONTEXT: usize = 256;
pub const MAX_VALUE_WIDTH: usize = 64;
pub const SCORE_FRACTIONAL_BITS: u32 = 24;
pub const VALUE_FRACTIONAL_BITS: u32 = 16;
pub const WEIGHT_FRACTIONAL_BITS: u32 = 31;
pub const WEIGHT_ONE: u64 = 1_u64 << WEIGHT_FRACTIONAL_BITS;
pub const EXP_STEP_LOG2: i32 = -8;
pub const EXP_TABLE_LEN: usize = (32 << 8) + 2;
/// The first Q24 distance without a complete interpolation interval. Rejecting
/// it before the helper's usize cast also preserves tail semantics on wasm32.
pub const EXP_TAIL_Q24: i64 = ((EXP_TABLE_LEN - 1) as i64) << 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadError {
    InvalidContext(usize),
    InvalidValueWidth(usize),
    TableLength {
        actual: usize,
    },
    TableEndpoint {
        index: usize,
        value: u32,
    },
    TableBound {
        index: usize,
        value: u32,
    },
    TableOrder {
        index: usize,
    },
    TooManyOccurrences {
        actual: usize,
        maximum: usize,
    },
    AgeLength {
        expected: usize,
        actual: usize,
    },
    ValueLength {
        expected: usize,
        actual: usize,
    },
    ScoreAddition {
        occurrence: usize,
    },
    /// `None` identifies NoRead; otherwise this is a causal occurrence index.
    ScoreDifference {
        occurrence: Option<usize>,
    },
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidContext(n) => {
                write!(f, "geometric read context {n} is outside 1..={MAX_CONTEXT}")
            }
            Self::InvalidValueWidth(n) => write!(
                f,
                "geometric read value width {n} is outside 1..={MAX_VALUE_WIDTH}"
            ),
            Self::TableLength { actual } => write!(
                f,
                "geometric read exp table has {actual} entries, expected {EXP_TABLE_LEN}"
            ),
            Self::TableEndpoint { index, value } => write!(
                f,
                "geometric read exp table endpoint {index} has invalid value {value}"
            ),
            Self::TableBound { index, value } => write!(
                f,
                "geometric read exp table entry {index} exceeds Q31 one: {value}"
            ),
            Self::TableOrder { index } => {
                write!(f, "geometric read exp table increases at {index}")
            }
            Self::TooManyOccurrences { actual, maximum } => write!(
                f,
                "geometric read has {actual} occurrences, maximum {maximum}"
            ),
            Self::AgeLength { expected, actual } => {
                write!(f, "geometric read has {actual} ages, expected {expected}")
            }
            Self::ValueLength { expected, actual } => write!(
                f,
                "geometric read has {actual} payload coordinates, expected {expected}"
            ),
            Self::ScoreAddition { occurrence } => write!(
                f,
                "geometric score plus age overflows at occurrence {occurrence}"
            ),
            Self::ScoreDifference { occurrence } => write!(
                f,
                "geometric read maximum difference exceeds i64 at {occurrence:?}"
            ),
        }
    }
}

impl std::error::Error for ReadError {}
pub type ReadResult<T> = Result<T, ReadError>;

/// Borrowed numerical output of the most recent successful reduction. Weights
/// are exp weights sharing `total_weight_q31`, not independently rounded
/// probabilities. Zero output may mean cancellation; inspect weights/NoRead.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReadReduction<'a> {
    pub output_q16: &'a [i32],
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

/// An admitted table plus bounded reusable numerical scratch. Invalid input
/// leaves [`Self::last`] unchanged. Private score scratch may be overwritten
/// during validation, but no exposed output or weights change on an error.
#[derive(Debug)]
pub struct NativeGeometricRead {
    max_context: usize,
    value_width: usize,
    exp_table: Box<[u32]>,
    value_lengths: Box<[usize]>,
    scores: Box<[i64]>,
    weights: Box<[u64]>,
    mix: Box<[i128]>,
    output: Box<[i32]>,
    last: Option<Summary>,
}

impl NativeGeometricRead {
    pub fn new(max_context: usize, value_width: usize, exp_table: &[u32]) -> ReadResult<Self> {
        if !(1..=MAX_CONTEXT).contains(&max_context) {
            return Err(ReadError::InvalidContext(max_context));
        }
        if !(1..=MAX_VALUE_WIDTH).contains(&value_width) {
            return Err(ReadError::InvalidValueWidth(value_width));
        }
        if exp_table.len() != EXP_TABLE_LEN {
            return Err(ReadError::TableLength {
                actual: exp_table.len(),
            });
        }
        if u64::from(exp_table[0]) != WEIGHT_ONE {
            return Err(ReadError::TableEndpoint {
                index: 0,
                value: exp_table[0],
            });
        }
        if exp_table[EXP_TABLE_LEN - 1] != 0 {
            return Err(ReadError::TableEndpoint {
                index: EXP_TABLE_LEN - 1,
                value: exp_table[EXP_TABLE_LEN - 1],
            });
        }
        for (index, &value) in exp_table.iter().enumerate() {
            if u64::from(value) > WEIGHT_ONE {
                return Err(ReadError::TableBound { index, value });
            }
            if index != 0 && value > exp_table[index - 1] {
                return Err(ReadError::TableOrder { index });
            }
        }
        // Prefix payload lengths are admitted once, avoiding runtime products
        // or length division when validating/visiting occurrence rows.
        let mut value_lengths = Vec::with_capacity(max_context + 1);
        let mut length = 0;
        for _ in 0..=max_context {
            value_lengths.push(length);
            length += value_width;
        }
        Ok(Self {
            max_context,
            value_width,
            exp_table: exp_table.into(),
            value_lengths: value_lengths.into_boxed_slice(),
            scores: vec![0; max_context].into_boxed_slice(),
            weights: vec![0; max_context].into_boxed_slice(),
            mix: vec![0; value_width].into_boxed_slice(),
            output: vec![0; value_width].into_boxed_slice(),
            last: None,
        })
    }

    pub fn max_context(&self) -> usize {
        self.max_context
    }
    pub fn value_width(&self) -> usize {
        self.value_width
    }

    fn exp_weight(&self, difference: i64) -> u64 {
        if difference >= EXP_TAIL_Q24 {
            0
        } else {
            stack_exp_neg(difference, -24, &self.exp_table, EXP_STEP_LOG2)
        }
    }

    pub fn last(&self) -> Option<ReadReduction<'_>> {
        self.last.map(|last| ReadReduction {
            output_q16: &self.output,
            occurrence_weights_q31: &self.weights[..last.occurrences],
            no_read_weight_q31: last.no_read_weight_q31,
            total_weight_q31: last.total_weight_q31,
            max_score_q24: last.max_score_q24,
        })
    }

    /// Reduce one head/query's causal prefix. `age_q24` is already aligned
    /// with occurrence order (the caller selects query-age entries). Payloads
    /// are contiguous occurrence-major rows of the admitted value width.
    ///
    /// Checked score+age and maximum subtraction reject an unrepresentable
    /// i64 difference even when it would lie beyond the exp-table tail. This
    /// is an explicit numerical input boundary, never wrapping or saturation.
    /// NoRead-only input is supported: all three occurrence slices are empty.
    pub fn reduce(
        &mut self,
        potential_q24: &[i64],
        age_q24: &[i64],
        no_read_q24: i64,
        values_q16: &[i32],
    ) -> ReadResult<ReadReduction<'_>> {
        let count = potential_q24.len();
        if count > self.max_context {
            return Err(ReadError::TooManyOccurrences {
                actual: count,
                maximum: self.max_context,
            });
        }
        if age_q24.len() != count {
            return Err(ReadError::AgeLength {
                expected: count,
                actual: age_q24.len(),
            });
        }
        let expected = self.value_lengths[count];
        if values_q16.len() != expected {
            return Err(ReadError::ValueLength {
                expected,
                actual: values_q16.len(),
            });
        }
        let mut maximum = no_read_q24;
        for (index, (&potential, &age)) in potential_q24.iter().zip(age_q24).enumerate() {
            let score = potential
                .checked_add(age)
                .ok_or(ReadError::ScoreAddition { occurrence: index })?;
            maximum = maximum.max(score);
            self.scores[index] = score;
        }
        let null_difference = maximum
            .checked_sub(no_read_q24)
            .ok_or(ReadError::ScoreDifference { occurrence: None })?;
        for (index, score) in self.scores[..count].iter().enumerate() {
            maximum
                .checked_sub(*score)
                .ok_or(ReadError::ScoreDifference {
                    occurrence: Some(index),
                })?;
        }
        // Every recoverable failure precedes this point. The bounds below
        // also establish that the reused wrapping product/sum helpers are
        // exact here rather than relying on their general wrapping behavior.
        let null_weight = self.exp_weight(null_difference);
        let mut total = null_weight;
        for index in 0..count {
            let weight = self.exp_weight(maximum - self.scores[index]);
            self.weights[index] = weight;
            total += weight;
        }
        // 1 <= total/2^31 <=257. Each signed coordinate sum has magnitude
        // <=256*2^31*2^31=2^70, well inside i128. NoRead contributes no value.
        self.mix.fill(0);
        let mut from = 0;
        for &weight in &self.weights[..count] {
            let to = from + self.value_width;
            if weight != 0 {
                stack_mix_row(weight, &values_q16[from..to], &mut self.mix);
            }
            from = to;
        }
        for (output, &numerator) in self.output.iter_mut().zip(self.mix.iter()) {
            // floor((|n| + floor(d/2))/d) is nearest, ties-away for either
            // parity of d. The quotient is a convex combination of admitted
            // i32 payloads and zero, hence fits the signed i32 output range.
            let magnitude = stack_div_u128(
                numerator.unsigned_abs() + u128::from(total >> 1),
                u128::from(total),
            );
            let signed = if numerator < 0 {
                -(magnitude as i64)
            } else {
                magnitude as i64
            };
            *output = signed as i32;
        }
        let last = Summary {
            occurrences: count,
            no_read_weight_q31: null_weight,
            total_weight_q31: total,
            max_score_q24: maximum,
        };
        self.last = Some(last);
        Ok(ReadReduction {
            output_q16: &self.output,
            occurrence_weights_q31: &self.weights[..count],
            no_read_weight_q31: null_weight,
            total_weight_q31: total,
            max_score_q24: maximum,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    type TestResult = Result<(), Box<dyn std::error::Error>>;

    // A precisely known admitted interpolation table, not an exp-fidelity
    // claim. The enclosing compiler binds the actual exporter table bytes.
    fn table() -> Vec<u32> {
        let mut values = vec![0; EXP_TABLE_LEN];
        values[0] = WEIGHT_ONE as u32;
        values[1] = (WEIGHT_ONE >> 1) as u32;
        values
    }

    fn owned(value: ReadReduction<'_>) -> (Vec<i32>, Vec<u64>, u64, u64, i64) {
        (
            value.output_q16.to_vec(),
            value.occurrence_weights_q31.to_vec(),
            value.no_read_weight_q31,
            value.total_weight_q31,
            value.max_score_q24,
        )
    }

    #[test]
    fn native_geometric_read_noread_occurrences_and_cancellation() -> TestResult {
        let mut read = NativeGeometricRead::new(4, 2, &table())?;
        assert!(read.last().is_none());
        let empty = read.reduce(&[], &[], i64::MIN, &[])?;
        assert_eq!(empty.output_q16, &[0, 0]);
        assert!(empty.occurrence_weights_q31.is_empty());
        assert_eq!(empty.no_read_weight_q31, WEIGHT_ONE);
        assert_eq!(empty.total_weight_q31, WEIGHT_ONE);
        // Equal occurrences remain two separate denominator terms, even
        // where their payloads cancel exactly in one coordinate.
        let cancel = read.reduce(&[0, 0], &[0, 0], 0, &[9, 12, -9, 12])?;
        assert_eq!(cancel.occurrence_weights_q31, &[WEIGHT_ONE, WEIGHT_ONE]);
        assert_eq!(cancel.output_q16, &[0, 8]);
        assert_eq!(cancel.total_weight_q31, 3 * WEIGHT_ONE);
        assert_eq!(cancel.no_read_weight_q31, WEIGHT_ONE);
        // One occurrence and NoRead tie: final signed half is ties-away.
        assert_eq!(read.reduce(&[0], &[0], 0, &[1, -1])?.output_q16, &[1, -1]);
        Ok(())
    }

    #[test]
    fn native_geometric_read_age_interpolation_tail_and_exact_reference() -> TestResult {
        let mut read = NativeGeometricRead::new(8, 3, &table())?;
        let step = 1_i64 << 16; // 2^-8 nats represented in Q24.
        let potential = [0, -step, -step, i64::MIN + 1];
        let ages = [0, 0, step >> 1, 0];
        let payload = [3, -7, 11, -5, 13, 17, 19, -23, 29, i32::MIN, i32::MAX, 31];
        let result = read.reduce(&potential, &ages, -step, &payload)?;
        let weights = [
            WEIGHT_ONE,
            WEIGHT_ONE >> 1,
            (WEIGHT_ONE >> 2) + (WEIGHT_ONE >> 1),
            0,
        ];
        assert_eq!(result.occurrence_weights_q31, &weights);
        assert_eq!(result.no_read_weight_q31, WEIGHT_ONE >> 1);
        let total = (WEIGHT_ONE >> 1) + weights.iter().sum::<u64>();
        assert_eq!(result.total_weight_q31, total);
        for column in 0..3 {
            // Independent host integer multiplication/division reference is
            // test-only; the numerical component reuses shift/add kernels.
            let numerator: i128 = weights
                .iter()
                .enumerate()
                .map(|(row, &weight)| i128::from(weight) * i128::from(payload[row * 3 + column]))
                .sum();
            let magnitude = (numerator.unsigned_abs() + u128::from(total / 2)) / u128::from(total);
            let expected = if numerator < 0 {
                -(magnitude as i64)
            } else {
                magnitude as i64
            };
            assert_eq!(i64::from(result.output_q16[column]), expected);
        }
        // Exact maximum can be either NoRead or a source; huge representable
        // differences go to the explicit table tail without index overflow.
        let null = read.reduce(&[i64::MIN + 1], &[0], 0, &[1, 2, 3])?;
        assert_eq!(null.occurrence_weights_q31, &[0]);
        assert_eq!(null.output_q16, &[0, 0, 0]);
        let narrow_index_tail = read.reduce(&[-(1_i64 << 48)], &[0], 0, &[1, 2, 3])?;
        assert_eq!(narrow_index_tail.occurrence_weights_q31, &[0]);
        let source = read.reduce(&[i64::MAX], &[0], 0, &[i32::MIN, i32::MAX, -1])?;
        assert_eq!(source.no_read_weight_q31, 0);
        assert_eq!(source.output_q16, &[i32::MIN, i32::MAX, -1]);
        Ok(())
    }

    #[test]
    fn native_geometric_read_extrema_and_full_capacity() -> TestResult {
        let mut read = NativeGeometricRead::new(MAX_CONTEXT, MAX_VALUE_WIDTH, &table())?;
        let scores = vec![0; MAX_CONTEXT];
        let values: Vec<i32> = (0..MAX_CONTEXT * MAX_VALUE_WIDTH)
            .map(|i| if i % 2 == 0 { i32::MIN } else { i32::MAX })
            .collect();
        let result = read.reduce(&scores, &scores, 0, &values)?;
        assert_eq!(result.total_weight_q31, 257 * WEIGHT_ONE);
        assert_eq!(result.occurrence_weights_q31.len(), MAX_CONTEXT);
        assert!(result
            .occurrence_weights_q31
            .iter()
            .all(|&w| w == WEIGHT_ONE));
        for (column, &value) in result.output_q16.iter().enumerate() {
            let raw = if column % 2 == 0 { i32::MIN } else { i32::MAX };
            let numerator = i128::from(raw) * 256;
            let magnitude = (numerator.unsigned_abs() + 128) / 257;
            let expected = if raw < 0 {
                -(magnitude as i64)
            } else {
                magnitude as i64
            };
            assert_eq!(i64::from(value), expected);
        }
        // A shorter prefix must not expose or mix stale previous weights.
        let short = read.reduce(&[0], &[0], 0, &values[..MAX_VALUE_WIDTH])?;
        assert_eq!(short.occurrence_weights_q31.len(), 1);
        assert_eq!(short.total_weight_q31, 2 * WEIGHT_ONE);
        Ok(())
    }

    #[test]
    fn native_geometric_read_admission_and_errors_are_transactional() -> TestResult {
        let valid = table();
        for context in [0, MAX_CONTEXT + 1, usize::MAX] {
            assert!(NativeGeometricRead::new(context, 1, &valid).is_err());
        }
        for width in [0, MAX_VALUE_WIDTH + 1, usize::MAX] {
            assert!(NativeGeometricRead::new(1, width, &valid).is_err());
        }
        assert!(matches!(
            NativeGeometricRead::new(1, 1, &valid[..EXP_TABLE_LEN - 1]),
            Err(ReadError::TableLength { .. })
        ));
        for (index, value) in [(0, 0), (EXP_TABLE_LEN - 1, 1), (1, u32::MAX), (3, 1)] {
            let mut bad = valid.clone();
            bad[index] = value;
            assert!(NativeGeometricRead::new(1, 1, &bad).is_err());
        }
        let mut read = NativeGeometricRead::new(2, 1, &valid)?;
        let before = owned(read.reduce(&[0], &[0], 0, &[11])?);
        assert!(matches!(
            read.reduce(&[0, 0, 0], &[0, 0, 0], 0, &[0, 0, 0]),
            Err(ReadError::TooManyOccurrences { .. })
        ));
        assert_eq!(
            owned(read.last().ok_or("missing previous reduction")?),
            before
        );
        assert!(matches!(
            read.reduce(&[0], &[], 0, &[0]),
            Err(ReadError::AgeLength { .. })
        ));
        assert_eq!(
            owned(read.last().ok_or("missing previous reduction")?),
            before
        );
        assert!(matches!(
            read.reduce(&[0], &[0], 0, &[]),
            Err(ReadError::ValueLength { .. })
        ));
        assert_eq!(
            owned(read.last().ok_or("missing previous reduction")?),
            before
        );
        assert!(matches!(
            read.reduce(&[0, i64::MAX], &[0, 1], 0, &[0, 0]),
            Err(ReadError::ScoreAddition { occurrence: 1 })
        ));
        assert_eq!(
            owned(read.last().ok_or("missing previous reduction")?),
            before
        );
        assert!(matches!(
            read.reduce(&[i64::MIN], &[0], 0, &[0]),
            Err(ReadError::ScoreDifference {
                occurrence: Some(0)
            })
        ));
        assert_eq!(
            owned(read.last().ok_or("missing previous reduction")?),
            before
        );
        assert!(matches!(
            read.reduce(&[0], &[0], i64::MIN, &[0]),
            Err(ReadError::ScoreDifference { occurrence: None })
        ));
        assert_eq!(
            owned(read.last().ok_or("missing previous reduction")?),
            before
        );
        Ok(())
    }
}
