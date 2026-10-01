//! Scale-aware current-input plus retained-identity projection.
//!
//! Both maps use the existing weights-only `Grouped4BitRow::encode_f32(_, 32)`
//! policy: signed codes -7..7 and dyadic group exponents -24..16. This is not
//! the canonical stack export's `(16 + m)` scale grid. No quantizer, rotation,
//! norm or codebook operation is performed here.
//!
//! Group dots use shift/add arithmetic. All nonzero groups from BOTH maps are
//! added exactly as checked i128 dyadics before ONE final nearest/ties-away
//! rounding to signed Q16. No clipping or per-map/per-group rounding occurs.
//! `Grouped4BitRow::dot_integer` is deliberately not used: that method rounds
//! negative-exponent groups before their sum.
//!
//! Construction allocates and validates the immutable maps. Projection uses
//! preallocated scratch, then commits output only if every row succeeds. Read
//! the latch's prior identity before its update; callers must finish both q/k
//! projections before committing either cache or advancing that latch.
//! This numerical component does not establish full-stack integer serving or
//! the instruction mix of a compiled binary.

use core::hint::black_box;
use std::fmt;

use crate::codec::Grouped4BitRow;
use crate::identity_latch::{DyadicVector, MAX_LATCH_EXPONENT, MIN_LATCH_EXPONENT};
use crate::math::scale_pow2;

pub const PROJECTION_OUTPUT_EXPONENT: i32 = -16;
pub const MAX_PROJECTION_WIDTH: usize = 4096;
const GROUP_SIZE: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectionSide {
    Current,
    Identity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowProblem {
    Width { expected: usize, actual: usize },
    GroupSize { actual: usize },
    GroupCount { expected: usize, actual: usize },
    PackedLength { expected: usize, actual: usize },
    Exponent { group: usize, exponent: i16 },
    Code { column: usize, code: i8 },
    NonzeroPadding,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectionError {
    InvalidDimension {
        value: usize,
    },
    RowCountMismatch {
        current: usize,
        identity: usize,
    },
    InvalidRow {
        side: ProjectionSide,
        row: usize,
        problem: RowProblem,
    },
    InputWidth {
        side: ProjectionSide,
        expected: usize,
        actual: usize,
    },
    InputExponent {
        side: ProjectionSide,
        exponent: i32,
    },
    OutputWidth {
        expected: usize,
        actual: usize,
    },
    /// Exact alignment or addition failed, even if a different evaluation
    /// order might cancel an unrepresentable intermediate into a small result.
    ArithmeticOverflow {
        row: usize,
    },
    OutputOverflow {
        row: usize,
    },
}

impl fmt::Display for ProjectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimension { value } => write!(f, "paired projection dimension {value} is outside 1..={MAX_PROJECTION_WIDTH}"),
            Self::RowCountMismatch { current, identity } => write!(f, "paired projection row counts differ: current {current}, identity {identity}"),
            Self::InvalidRow { side, row, problem } => write!(f, "paired projection {side:?} row {row}: {problem:?}"),
            Self::InputWidth { side, expected, actual } => write!(f, "paired projection {side:?} input needs width {expected}, got {actual}"),
            Self::InputExponent { side, exponent } => write!(f, "paired projection {side:?} input exponent {exponent} is outside {MIN_LATCH_EXPONENT}..={MAX_LATCH_EXPONENT}"),
            Self::OutputWidth { expected, actual } => write!(f, "paired projection output needs width {expected}, got {actual}"),
            Self::ArithmeticOverflow { row } => write!(f, "paired projection exact accumulation overflow at row {row}"),
            Self::OutputOverflow { row } => write!(f, "paired projection Q16 output exceeds i32 at row {row}"),
        }
    }
}

impl std::error::Error for ProjectionError {}

/// One paired map; instantiate separately for query and key. Rows and their
/// scales are owned and immutable, while staging is reused by each call.
#[derive(Clone, Debug)]
pub struct PairedProjection {
    current: Vec<Grouped4BitRow>,
    identity: Vec<Grouped4BitRow>,
    input_width: usize,
    staging: Vec<i32>,
}

impl PairedProjection {
    pub fn new(
        current_rows: Vec<Grouped4BitRow>,
        identity_rows: Vec<Grouped4BitRow>,
    ) -> Result<Self, ProjectionError> {
        let rows = current_rows.len();
        validate_dimension(rows)?;
        if identity_rows.len() != rows {
            return Err(ProjectionError::RowCountMismatch {
                current: rows,
                identity: identity_rows.len(),
            });
        }
        let input_width = current_rows[0].elements;
        validate_dimension(input_width)?;
        for (side, map) in [
            (ProjectionSide::Current, &current_rows),
            (ProjectionSide::Identity, &identity_rows),
        ] {
            for (index, row) in map.iter().enumerate() {
                validate_row(row, input_width).map_err(|problem| ProjectionError::InvalidRow {
                    side,
                    row: index,
                    problem,
                })?;
            }
        }
        Ok(Self {
            current: current_rows,
            identity: identity_rows,
            input_width,
            staging: vec![0; rows],
        })
    }

    pub fn input_width(&self) -> usize {
        self.input_width
    }

    pub fn output_width(&self) -> usize {
        self.staging.len()
    }

    /// Compute both contributions before one final Q16 rounding. Output is
    /// unchanged on every error. No normalization, saturation or allocation is
    /// performed. Scratch may change on failure and is completely overwritten
    /// before a subsequent successful call commits.
    #[inline(never)]
    pub fn project(
        &mut self,
        current: DyadicVector<'_>,
        prior_identity: DyadicVector<'_>,
        output: &mut [i32],
    ) -> Result<(), ProjectionError> {
        validate_input(current, self.input_width, ProjectionSide::Current)?;
        validate_input(prior_identity, self.input_width, ProjectionSide::Identity)?;
        if output.len() != self.output_width() {
            return Err(ProjectionError::OutputWidth {
                expected: self.output_width(),
                actual: output.len(),
            });
        }
        for (row, ((current_row, identity_row), slot)) in self
            .current
            .iter()
            .zip(&self.identity)
            .zip(&mut self.staging)
            .enumerate()
        {
            let mut sum = ExactSum::default();
            sum.add_map(current_row, current)
                .and_then(|()| sum.add_map(identity_row, prior_identity))
                .map_err(|()| ProjectionError::ArithmeticOverflow { row })?;
            // This is the only right-shift rounding in the entire paired map.
            let rounded = scale_pow2(sum.mantissa, sum.exponent - PROJECTION_OUTPUT_EXPONENT)
                .map_err(|_| ProjectionError::ArithmeticOverflow { row })?;
            *slot = i32::try_from(rounded).map_err(|_| ProjectionError::OutputOverflow { row })?;
        }
        output.copy_from_slice(&self.staging);
        Ok(())
    }
}

fn validate_dimension(value: usize) -> Result<(), ProjectionError> {
    if value == 0 || value > MAX_PROJECTION_WIDTH {
        return Err(ProjectionError::InvalidDimension { value });
    }
    Ok(())
}

fn validate_input(
    input: DyadicVector<'_>,
    width: usize,
    side: ProjectionSide,
) -> Result<(), ProjectionError> {
    if input.mantissas.len() != width {
        return Err(ProjectionError::InputWidth {
            side,
            expected: width,
            actual: input.mantissas.len(),
        });
    }
    if !(MIN_LATCH_EXPONENT..=MAX_LATCH_EXPONENT).contains(&input.exponent) {
        return Err(ProjectionError::InputExponent {
            side,
            exponent: input.exponent,
        });
    }
    Ok(())
}

fn validate_row(row: &Grouped4BitRow, width: usize) -> Result<(), RowProblem> {
    if row.elements != width {
        return Err(RowProblem::Width {
            expected: width,
            actual: row.elements,
        });
    }
    if row.group_size != GROUP_SIZE {
        return Err(RowProblem::GroupSize {
            actual: row.group_size,
        });
    }
    let groups = (width + 31) >> 5;
    if row.group_exponents.len() != groups {
        return Err(RowProblem::GroupCount {
            expected: groups,
            actual: row.group_exponents.len(),
        });
    }
    let bytes = (width + 1) >> 1;
    if row.packed_codes.len() != bytes {
        return Err(RowProblem::PackedLength {
            expected: bytes,
            actual: row.packed_codes.len(),
        });
    }
    for (group, &exponent) in row.group_exponents.iter().enumerate() {
        if !(-24..=16).contains(&exponent) {
            return Err(RowProblem::Exponent { group, exponent });
        }
    }
    for column in 0..width {
        let code = row_code(row, column);
        if !(-7..=7).contains(&code) {
            return Err(RowProblem::Code { column, code });
        }
    }
    if width & 1 != 0 && row.packed_codes[bytes - 1] >> 4 != 0 {
        return Err(RowProblem::NonzeroPadding);
    }
    Ok(())
}

#[inline(always)]
fn row_code(row: &Grouped4BitRow, column: usize) -> i8 {
    let nibble = (row.packed_codes[column >> 1] >> ((column & 1) << 2)) & 15;
    (nibble as i8) << 4 >> 4
}

/// A nonzero contribution changes the common exponent only by exact checked
/// alignment. Cancellations to zero release the old alignment scale.
#[derive(Default)]
struct ExactSum {
    mantissa: i128,
    exponent: i32,
}

impl ExactSum {
    fn add_term(&mut self, mantissa: i128, exponent: i32) -> Result<(), ()> {
        if mantissa == 0 {
            return Ok(());
        }
        if self.mantissa == 0 {
            self.mantissa = mantissa;
            self.exponent = exponent;
            return Ok(());
        }
        let common = self.exponent.min(exponent);
        // Both shifts are nonnegative, so scale_pow2 is exact here.
        let left = scale_pow2(self.mantissa, self.exponent - common).map_err(|_| ())?;
        let right = scale_pow2(mantissa, exponent - common).map_err(|_| ())?;
        self.mantissa = left.checked_add(right).ok_or(())?;
        self.exponent = if self.mantissa == 0 { 0 } else { common };
        Ok(())
    }

    #[inline(never)]
    fn add_map(&mut self, row: &Grouped4BitRow, input: DyadicVector<'_>) -> Result<(), ()> {
        let opaque_zero = black_box(0i64);
        let mut start = 0usize;
        for &weight_exponent in &row.group_exponents {
            let end = (start + GROUP_SIZE).min(row.elements);
            let mut group_dot = 0i64;
            for column in start..end {
                let code = row_code(row, column);
                let mut magnitude = code.unsigned_abs();
                let mut value = i64::from(input.mantissas[column]);
                let mut product = 0i64;
                while magnitude != 0 {
                    if magnitude & 1 != 0 {
                        product = product.checked_add(value ^ opaque_zero).ok_or(())?;
                    }
                    magnitude >>= 1;
                    // Validated |code| <= 7: at most three small shifts.
                    value <<= 1;
                }
                group_dot = if code < 0 {
                    group_dot.checked_sub(product)
                } else {
                    group_dot.checked_add(product)
                }
                .ok_or(())?;
            }
            self.add_term(
                i128::from(group_dot),
                input.exponent + i32::from(weight_exponent),
            )?;
            start = end;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(codes: &[i8], exponents: &[i16]) -> Grouped4BitRow {
        let packed: Vec<u8> = codes
            .chunks(2)
            .map(|pair| {
                (pair[0] as u8 & 15) | ((pair.get(1).copied().unwrap_or(0) as u8 & 15) << 4)
            })
            .collect();
        Grouped4BitRow {
            group_size: 32,
            group_exponents: exponents.to_vec(),
            packed_codes: packed.into(),
            elements: codes.len(),
        }
    }

    #[test]
    fn paired_projection_mixed_scales_tail_and_orientation() -> Result<(), ProjectionError> {
        let mut current_codes = vec![0; 33];
        current_codes[..3].copy_from_slice(&[1, -2, 3]);
        current_codes[32] = 4;
        let mut identity_codes = vec![0; 33];
        identity_codes[..3].copy_from_slice(&[-3, 2, 1]);
        identity_codes[32] = -1;
        let negative_current: Vec<i8> = current_codes.iter().map(|&x| -x).collect();
        let negative_identity: Vec<i8> = identity_codes.iter().map(|&x| -x).collect();
        let mut projection = PairedProjection::new(
            vec![
                row(&current_codes, &[-16, -15]),
                row(&negative_current, &[-16, -15]),
            ],
            vec![
                row(&identity_codes, &[-17, -18]),
                row(&negative_identity, &[-17, -18]),
            ],
        )?;
        let mut current = vec![0; 33];
        current[..3].copy_from_slice(&[4, 6, -2]);
        current[32] = 7;
        let mut identity = vec![0; 33];
        identity[..3].copy_from_slice(&[2, -4, 8]);
        identity[32] = 3;
        let mut out = [99, 99];
        projection.project(
            DyadicVector {
                mantissas: &current,
                exponent: 0,
            },
            DyadicVector {
                mantissas: &identity,
                exponent: 2,
            },
            &mut out,
        )?;
        // (-14 + 2*28) + (2*(-6) - 3), at Q16. The negative
        // row retains the opposite orientation, with no vector normalization.
        assert_eq!(out, [27, -27]);
        assert_eq!(projection.input_width(), 33);
        assert_eq!(projection.output_width(), 2);
        Ok(())
    }

    #[test]
    fn paired_projection_rounds_only_after_both_maps_and_groups() -> Result<(), ProjectionError> {
        let input = DyadicVector {
            mantissas: &[1],
            exponent: 0,
        };
        for sign in [-1, 1] {
            let mut projection =
                PairedProjection::new(vec![row(&[sign], &[-17])], vec![row(&[sign], &[-17])])?;
            let mut out = [0];
            projection.project(input, input, &mut out)?;
            // Two half-units become one unit; separate rounding makes two.
            assert_eq!(out, [i32::from(sign)]);
            let zero = DyadicVector {
                mantissas: &[0],
                exponent: -64,
            };
            projection.project(input, zero, &mut out)?;
            assert_eq!(out, [i32::from(sign)], "half ties away from zero");
        }
        let mut codes = vec![0; 33];
        codes[0] = 1;
        codes[32] = 1;
        let mut values = vec![0; 33];
        values[0] = 1;
        values[32] = 1;
        let zeros = vec![0i8; 33];
        let zero_inputs = vec![0i16; 33];
        let mut grouped =
            PairedProjection::new(vec![row(&codes, &[-17, -17])], vec![row(&zeros, &[0, 0])])?;
        let mut out = [0];
        grouped.project(
            DyadicVector {
                mantissas: &values,
                exponent: 0,
            },
            DyadicVector {
                mantissas: &zero_inputs,
                exponent: 64,
            },
            &mut out,
        )?;
        assert_eq!(out, [1], "no per-group rounding");
        Ok(())
    }

    #[test]
    fn paired_projection_cancellation_zero_and_signed_endpoints() -> Result<(), ProjectionError> {
        let mut cancellation =
            PairedProjection::new(vec![row(&[7], &[16])], vec![row(&[-7], &[16])])?;
        for mantissa in [i16::MIN, -1, 0, 1, i16::MAX] {
            let input = DyadicVector {
                mantissas: &[mantissa],
                exponent: 0,
            };
            let mut out = [17];
            cancellation.project(input, input, &mut out)?;
            assert_eq!(out, [0], "large contributions cancel before Q16 conversion");
        }
        let mut direct = PairedProjection::new(vec![row(&[7], &[-16])], vec![row(&[-7], &[16])])?;
        for mantissa in [i16::MIN, i16::MAX] {
            let mut out = [0];
            direct.project(
                DyadicVector {
                    mantissas: &[mantissa],
                    exponent: 0,
                },
                DyadicVector {
                    mantissas: &[0],
                    exponent: -64,
                },
                &mut out,
            )?;
            assert_eq!(out, [7 * i32::from(mantissa)]);
        }
        // Same real inputs under different dyadic representations agree.
        let mut out = [0];
        direct.project(
            DyadicVector {
                mantissas: &[6],
                exponent: -1,
            },
            DyadicVector {
                mantissas: &[0],
                exponent: 64,
            },
            &mut out,
        )?;
        assert_eq!(out, [21]);
        Ok(())
    }

    #[test]
    fn paired_projection_errors_leave_output_unchanged_and_retry_cleanly(
    ) -> Result<(), ProjectionError> {
        let mut projection = PairedProjection::new(
            vec![row(&[1], &[-16]), row(&[1], &[16])],
            vec![row(&[0], &[0]), row(&[0], &[0])],
        )?;
        let current = DyadicVector {
            mantissas: &[1],
            exponent: 0,
        };
        let zero = DyadicVector {
            mantissas: &[0],
            exponent: 0,
        };
        let mut out = [23, -29];
        assert_eq!(
            projection.project(current, zero, &mut out),
            Err(ProjectionError::OutputOverflow { row: 1 })
        );
        assert_eq!(out, [23, -29], "completed first row must not leak");
        for bad in [
            DyadicVector {
                mantissas: &[],
                exponent: 0,
            },
            DyadicVector {
                mantissas: &[0],
                exponent: 65,
            },
        ] {
            assert!(projection.project(bad, zero, &mut out).is_err());
            assert!(projection.project(zero, bad, &mut out).is_err());
            assert_eq!(out, [23, -29]);
        }
        let mut short = [31];
        assert!(projection.project(zero, zero, &mut short).is_err());
        assert_eq!(short, [31]);
        projection.project(zero, zero, &mut out)?;
        assert_eq!(out, [0, 0]);
        let mut extreme = PairedProjection::new(vec![row(&[1], &[16])], vec![row(&[1], &[-24])])?;
        let mut out = [37];
        assert_eq!(
            extreme.project(
                DyadicVector {
                    mantissas: &[1],
                    exponent: 64
                },
                DyadicVector {
                    mantissas: &[1],
                    exponent: -64
                },
                &mut out
            ),
            Err(ProjectionError::ArithmeticOverflow { row: 0 })
        );
        assert_eq!(out, [37]);
        Ok(())
    }

    #[test]
    fn paired_projection_constructor_rejects_malformed_or_other_codec_rows() {
        let good = row(&[1], &[-16]);
        assert!(PairedProjection::new(vec![], vec![]).is_err());
        assert!(PairedProjection::new(vec![good.clone()], vec![]).is_err());
        let mut bad_rows = vec![
            row(&[-8], &[-16]),
            row(&[1], &[-25]),
            row(&[1], &[17]),
            row(&[1, 0], &[-16]),
        ];
        let mut bad = good.clone();
        bad.group_size = 16;
        bad_rows.push(bad);
        let mut bad = good.clone();
        bad.group_exponents.clear();
        bad_rows.push(bad);
        let mut bad = good.clone();
        bad.packed_codes = Vec::<u8>::new().into();
        bad_rows.push(bad);
        let mut bad = good.clone();
        bad.packed_codes = vec![0x11].into();
        bad_rows.push(bad);
        for bad in bad_rows {
            // Put a valid row first so wrong element counts are checked as a
            // row mismatch, rather than redefining the map's input width.
            assert!(PairedProjection::new(
                vec![good.clone(), bad.clone()],
                vec![good.clone(), good.clone()]
            )
            .is_err());
            assert!(PairedProjection::new(vec![good.clone()], vec![bad]).is_err());
        }
    }
}
