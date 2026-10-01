//! Checked, scale-aware integer component for a learned identity latch.
//!
//! A vector coordinate represents `mantissa * 2^exponent`. Current and previous
//! inputs retain separate exponents, as do the two signed four-bit gate halves.
//! Each dot is exact shift/add arithmetic; the three nonzero dyadic terms are
//! aligned to their finest exponent and summed in checked i128. There is **no
//! rounding or saturation in this gate**: zero captures, negative values do not.
//! The offline weight/input quantizer owns and must record its rounding policy.
//!
//! Read [`IdentityLatch::identity`] before [`IdentityLatch::step`]: the current
//! row reads the old state, then the gate changes the state for the next row.
//! Capture copies both the input mantissas and their exponent, with no norm or
//! codebook conversion. Previous input advances even when capture is false.
//!
//! Construction allocates. Gate evaluation and latch steps allocate nothing,
//! and any returned error leaves latch state unchanged. This is a component,
//! not an integer port of the surrounding stack, q/k projections or reader.
//! Source arithmetic uses no floating point, multiplication or division; the
//! instruction mix of a compiled binary still requires its own inspection.

use core::hint::black_box;
use std::fmt;

use crate::math::scale_pow2;

/// Explicit component bounds, independent of any full-stack admission claim.
pub const MAX_LATCH_WIDTH: usize = 4096;
pub const MIN_LATCH_EXPONENT: i32 = -64;
pub const MAX_LATCH_EXPONENT: i32 = 64;

/// A borrowed vector with one retained exponent for this occurrence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DyadicVector<'a> {
    pub mantissas: &'a [i16],
    pub exponent: i32,
}

/// The fixed gate bias has an independent dyadic scale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DyadicBias {
    pub mantissa: i64,
    pub exponent: i32,
}

/// Exact accumulated gate value. Zero uses the canonical exponent zero.
/// The exponent is a product exponent and may exceed the input exponent range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DyadicLogit {
    pub mantissa: i128,
    pub exponent: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GateDecision {
    pub capture: bool,
    pub logit: DyadicLogit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GateHalf {
    Current,
    Previous,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LatchMode {
    Held,
    Local,
}

/// Fixed-size errors require no message allocation in the numerical step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdentityLatchError {
    InvalidWidth {
        width: usize,
    },
    WidthMismatch {
        expected: usize,
        actual: usize,
    },
    InvalidCoefficient {
        half: GateHalf,
        index: usize,
        value: i8,
    },
    InvalidExponent {
        exponent: i32,
    },
    /// Includes an unrepresentable exact alignment, even if an alternative
    /// evaluation order could cancel it into a representable final sum.
    ArithmeticOverflow,
}

impl fmt::Display for IdentityLatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidWidth { width } => write!(f, "identity latch width {width} is outside 1..={MAX_LATCH_WIDTH}"),
            Self::WidthMismatch { expected, actual } => write!(f, "identity latch needs width {expected}, got {actual}"),
            Self::InvalidCoefficient { half, index, value } => write!(f, "identity latch {half:?} coefficient {index} is {value}, outside -8..=7"),
            Self::InvalidExponent { exponent } => write!(f, "identity latch exponent {exponent} is outside {MIN_LATCH_EXPONENT}..={MAX_LATCH_EXPONENT}"),
            Self::ArithmeticOverflow => f.write_str("identity latch exact dyadic accumulation overflow"),
        }
    }
}

impl std::error::Error for IdentityLatchError {}

pub type LatchResult<T> = Result<T, IdentityLatchError>;

fn validate_exponent(exponent: i32) -> LatchResult<()> {
    if !(MIN_LATCH_EXPONENT..=MAX_LATCH_EXPONENT).contains(&exponent) {
        return Err(IdentityLatchError::InvalidExponent { exponent });
    }
    Ok(())
}

fn validate_vector(vector: DyadicVector<'_>, width: usize) -> LatchResult<()> {
    if vector.mantissas.len() != width {
        return Err(IdentityLatchError::WidthMismatch {
            expected: width,
            actual: vector.mantissas.len(),
        });
    }
    validate_exponent(vector.exponent)
}

/// Artifact-supplied coefficients. The constructor validates all codes and
/// scales once; no mutable parameter access is exposed during inference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdentityGate {
    current: Vec<i8>,
    current_exponent: i32,
    previous: Vec<i8>,
    previous_exponent: i32,
    bias: DyadicBias,
}

impl IdentityGate {
    pub fn new(
        current: Vec<i8>,
        current_exponent: i32,
        previous: Vec<i8>,
        previous_exponent: i32,
        bias: DyadicBias,
    ) -> LatchResult<Self> {
        let width = current.len();
        if width == 0 || width > MAX_LATCH_WIDTH {
            return Err(IdentityLatchError::InvalidWidth { width });
        }
        if previous.len() != width {
            return Err(IdentityLatchError::WidthMismatch {
                expected: width,
                actual: previous.len(),
            });
        }
        for (half, codes) in [
            (GateHalf::Current, &current),
            (GateHalf::Previous, &previous),
        ] {
            for (index, &value) in codes.iter().enumerate() {
                if !(-8..=7).contains(&value) {
                    return Err(IdentityLatchError::InvalidCoefficient { half, index, value });
                }
            }
        }
        validate_exponent(current_exponent)?;
        validate_exponent(previous_exponent)?;
        validate_exponent(bias.exponent)?;
        Ok(Self {
            current,
            current_exponent,
            previous,
            previous_exponent,
            bias,
        })
    }

    pub fn width(&self) -> usize {
        self.current.len()
    }

    /// Exact sign of `W_current current + W_previous previous + bias`.
    /// The input halves are not reinterpreted at one another's scales.
    #[inline(never)]
    pub fn evaluate(
        &self,
        current: DyadicVector<'_>,
        previous: DyadicVector<'_>,
    ) -> LatchResult<GateDecision> {
        validate_vector(current, self.width())?;
        validate_vector(previous, self.width())?;
        let terms = [
            DyadicLogit {
                mantissa: identity_gate_dot(&self.current, current.mantissas)?,
                exponent: self.current_exponent + current.exponent,
            },
            DyadicLogit {
                mantissa: identity_gate_dot(&self.previous, previous.mantissas)?,
                exponent: self.previous_exponent + previous.exponent,
            },
            DyadicLogit {
                mantissa: i128::from(self.bias.mantissa),
                exponent: self.bias.exponent,
            },
        ];
        let logit = identity_gate_sum(&terms)?;
        Ok(GateDecision {
            capture: logit.mantissa >= 0,
            logit,
        })
    }
}

/// Four-bit magnitude times one signed input, by at most four shifted adds.
/// The opaque zero follows the stack kernel convention to keep these additive
/// terms visible rather than inviting replacement by a multiply instruction.
#[inline(never)]
fn identity_gate_dot(weights: &[i8], inputs: &[i16]) -> LatchResult<i128> {
    let opaque_zero = black_box(0i128);
    let mut sum = 0i128;
    for (&weight, &input) in weights.iter().zip(inputs) {
        let mut code = weight.unsigned_abs();
        let mut value = i128::from(input);
        let mut product = 0i128;
        while code != 0 {
            if code & 1 != 0 {
                product = product
                    .checked_add(value ^ opaque_zero)
                    .ok_or(IdentityLatchError::ArithmeticOverflow)?;
            }
            code >>= 1;
            // |input| <= 2^15 and this loop makes at most four shifts.
            value <<= 1;
        }
        sum = if weight < 0 {
            sum.checked_sub(product)
        } else {
            sum.checked_add(product)
        }
        .ok_or(IdentityLatchError::ArithmeticOverflow)?;
    }
    Ok(sum)
}

#[inline(never)]
fn identity_gate_sum(terms: &[DyadicLogit; 3]) -> LatchResult<DyadicLogit> {
    let Some(exponent) = terms
        .iter()
        .filter(|term| term.mantissa != 0)
        .map(|term| term.exponent)
        .min()
    else {
        return Ok(DyadicLogit {
            mantissa: 0,
            exponent: 0,
        });
    };
    let mut mantissa = 0i128;
    for term in terms {
        if term.mantissa == 0 {
            continue;
        }
        // The difference is nonnegative: scale_pow2 therefore shifts exactly,
        // never entering its documented right-shift rounding branch.
        let aligned = scale_pow2(term.mantissa, term.exponent - exponent)
            .map_err(|_| IdentityLatchError::ArithmeticOverflow)?;
        mantissa = mantissa
            .checked_add(aligned)
            .ok_or(IdentityLatchError::ArithmeticOverflow)?;
    }
    Ok(DyadicLogit {
        mantissa,
        exponent: if mantissa == 0 { 0 } else { exponent },
    })
}

/// Reusable state with two preallocated vectors. No token labels, source masks
/// or semantic role rules enter the gate or state update.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdentityLatch {
    gate: IdentityGate,
    mode: LatchMode,
    identity: Vec<i16>,
    identity_exponent: i32,
    previous: Vec<i16>,
    previous_exponent: i32,
}

impl IdentityLatch {
    pub fn new(gate: IdentityGate, mode: LatchMode) -> LatchResult<Self> {
        let width = gate.width();
        Ok(Self {
            gate,
            mode,
            identity: vec![0; width],
            identity_exponent: 0,
            previous: vec![0; width],
            previous_exponent: 0,
        })
    }

    /// Observe this before stepping: it is the identity supplied to q/k for
    /// the current row, before that row's capture decision is applied.
    pub fn identity(&self) -> DyadicVector<'_> {
        DyadicVector {
            mantissas: &self.identity,
            exponent: self.identity_exponent,
        }
    }

    pub fn previous_input(&self) -> DyadicVector<'_> {
        DyadicVector {
            mantissas: &self.previous,
            exponent: self.previous_exponent,
        }
    }

    pub fn mode(&self) -> LatchMode {
        self.mode
    }

    /// Evaluate first, then commit both updates. A failed gate evaluation
    /// changes neither identity nor previous input. HOLD preserves the exact
    /// Held codes and exponent; Local no-capture produces canonical zero.
    #[inline(never)]
    pub fn step(&mut self, input: DyadicVector<'_>) -> LatchResult<GateDecision> {
        let decision = self.gate.evaluate(input, self.previous_input())?;
        if decision.capture {
            self.identity.copy_from_slice(input.mantissas);
            self.identity_exponent = input.exponent;
        } else if self.mode == LatchMode::Local {
            self.identity.fill(0);
            self.identity_exponent = 0;
        }
        self.previous.copy_from_slice(input.mantissas);
        self.previous_exponent = input.exponent;
        Ok(decision)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_four_bit_dot_matches_exact_integer_reference() -> LatchResult<()> {
        let inputs = [i16::MIN, -8193, -1, 0, 1, 4097, i16::MAX];
        for weight in -8i8..=7 {
            for input in inputs {
                assert_eq!(
                    identity_gate_dot(&[weight], &[input])?,
                    i128::from(weight) * i128::from(input)
                );
            }
        }
        let weights: Vec<i8> = (0..MAX_LATCH_WIDTH).map(|i| ((i & 15) as i8) - 8).collect();
        let values: Vec<i16> = (0..MAX_LATCH_WIDTH)
            .map(|i| inputs[i % inputs.len()])
            .collect();
        let expected: i128 = weights
            .iter()
            .zip(&values)
            .map(|(&w, &x)| i128::from(w) * i128::from(x))
            .sum();
        assert_eq!(identity_gate_dot(&weights, &values)?, expected);
        Ok(())
    }

    #[test]
    fn different_scales_align_exactly_and_zero_tie_captures() -> LatchResult<()> {
        let gate = IdentityGate::new(
            vec![3, -8],
            -2,
            vec![-1, 7],
            1,
            DyadicBias {
                mantissa: -3,
                exponent: -3,
            },
        )?;
        let current = DyadicVector {
            mantissas: &[4, -2],
            exponent: -3,
        };
        let previous = DyadicVector {
            mantissas: &[5, 1],
            exponent: -4,
        };
        let decision = gate.evaluate(current, previous)?;
        // 28 * 2^-5 + 2 * 2^-3 - 3 * 2^-3 = 24 * 2^-5.
        assert_eq!(
            decision,
            GateDecision {
                capture: true,
                logit: DyadicLogit {
                    mantissa: 24,
                    exponent: -5
                }
            }
        );
        // Equal real vectors at different input scales preserve the decision
        // and exact real sum, without merging the occurrence's stored scales.
        let rescaled = gate.evaluate(
            DyadicVector {
                mantissas: &[8, -4],
                exponent: -4,
            },
            previous,
        )?;
        assert_eq!(rescaled.logit.mantissa, 48);
        assert_eq!(rescaled.logit.exponent, -6);
        let cancel = IdentityGate::new(
            vec![1],
            -1,
            vec![-1],
            0,
            DyadicBias {
                mantissa: 0,
                exponent: -64,
            },
        )?;
        let zero = cancel.evaluate(
            DyadicVector {
                mantissas: &[2],
                exponent: 0,
            },
            DyadicVector {
                mantissas: &[1],
                exponent: 0,
            },
        )?;
        assert_eq!(
            zero,
            GateDecision {
                capture: true,
                logit: DyadicLogit {
                    mantissa: 0,
                    exponent: 0
                }
            }
        );
        let negative = cancel.evaluate(
            DyadicVector {
                mantissas: &[1],
                exponent: -64,
            },
            DyadicVector {
                mantissas: &[1],
                exponent: -63,
            },
        )?;
        assert!(!negative.capture);
        assert_eq!(
            negative.logit,
            DyadicLogit {
                mantissa: -3,
                exponent: -65
            }
        );
        Ok(())
    }

    #[test]
    fn latch_reads_old_state_and_advances_previous_input_on_hold() -> LatchResult<()> {
        for mode in [LatchMode::Held, LatchMode::Local] {
            // Decision depends only on previous input. The second call must
            // capture its CURRENT contents; a HOLD still advances previous.
            let gate = IdentityGate::new(
                vec![0, 0],
                0,
                vec![1, 0],
                0,
                DyadicBias {
                    mantissa: -1,
                    exponent: 0,
                },
            )?;
            let mut latch = IdentityLatch::new(gate, mode)?;
            assert_eq!(
                latch.identity(),
                DyadicVector {
                    mantissas: &[0, 0],
                    exponent: 0
                }
            );
            let first = DyadicVector {
                mantissas: &[2, -7],
                exponent: 1,
            };
            assert!(!latch.step(first)?.capture);
            assert_eq!(latch.previous_input(), first);
            assert_eq!(latch.identity().mantissas, &[0, 0]);
            let second = DyadicVector {
                mantissas: &[-8, i16::MIN],
                exponent: -2,
            };
            assert!(latch.step(second)?.capture);
            assert_eq!(latch.identity(), second);
            let third = DyadicVector {
                mantissas: &[16, 3],
                exponent: -3,
            };
            assert!(!latch.step(third)?.capture);
            assert_eq!(latch.previous_input(), third);
            assert_eq!(
                latch.identity(),
                if mode == LatchMode::Held {
                    second
                } else {
                    DyadicVector {
                        mantissas: &[0, 0],
                        exponent: 0,
                    }
                }
            );
            let fourth = DyadicVector {
                mantissas: &[0, 19],
                exponent: 4,
            };
            assert!(latch.step(fourth)?.capture);
            assert_eq!(latch.identity(), fourth);
        }
        Ok(())
    }

    #[test]
    fn bounds_and_overflow_reject_without_mutating_latch() -> LatchResult<()> {
        let bias = DyadicBias {
            mantissa: 0,
            exponent: 0,
        };
        assert!(matches!(
            IdentityGate::new(vec![], 0, vec![], 0, bias),
            Err(IdentityLatchError::InvalidWidth { .. })
        ));
        assert!(matches!(
            IdentityGate::new(
                vec![0; MAX_LATCH_WIDTH + 1],
                0,
                vec![0; MAX_LATCH_WIDTH + 1],
                0,
                bias
            ),
            Err(IdentityLatchError::InvalidWidth { .. })
        ));
        assert!(matches!(
            IdentityGate::new(vec![0], 0, vec![], 0, bias),
            Err(IdentityLatchError::WidthMismatch { .. })
        ));
        for code in [-9, 8] {
            assert!(matches!(
                IdentityGate::new(vec![code], 0, vec![0], 0, bias),
                Err(IdentityLatchError::InvalidCoefficient { .. })
            ));
        }
        for exponent in [i32::MIN, -65, 65, i32::MAX] {
            assert!(matches!(
                IdentityGate::new(vec![0], exponent, vec![0], 0, bias),
                Err(IdentityLatchError::InvalidExponent { .. })
            ));
        }
        // Tiny bias and a high-exponent nonzero input cannot be exactly
        // aligned within i128. Refuse rather than lose the low bits or wrap.
        let gate = IdentityGate::new(
            vec![1],
            64,
            vec![0],
            0,
            DyadicBias {
                mantissa: 1,
                exponent: -64,
            },
        )?;
        let mut latch = IdentityLatch::new(gate, LatchMode::Held)?;
        latch.step(DyadicVector {
            mantissas: &[0],
            exponent: 64,
        })?;
        let before = latch.clone();
        for input in [
            DyadicVector {
                mantissas: &[1],
                exponent: 64,
            },
            DyadicVector {
                mantissas: &[],
                exponent: 0,
            },
            DyadicVector {
                mantissas: &[1],
                exponent: 65,
            },
        ] {
            assert!(latch.step(input).is_err());
            assert_eq!(latch, before);
        }
        let max = DyadicLogit {
            mantissa: i128::MAX,
            exponent: 0,
        };
        assert_eq!(
            identity_gate_sum(&[
                max,
                DyadicLogit {
                    mantissa: 1,
                    exponent: 0
                },
                DyadicLogit {
                    mantissa: 0,
                    exponent: 0
                }
            ]),
            Err(IdentityLatchError::ArithmeticOverflow)
        );
        Ok(())
    }
}
