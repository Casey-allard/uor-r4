//! Fixed two-term signed-H4 transport from K2 value packets to residual lanes.
//!
//! For every head/output/input/term, transform each nonzero atom by
//! `left * root * inverse(right)`, preserving its radius. Accumulate signed
//! dimensionless q4 gains against canonical decoded Q16 coordinates, then
//! round ONCE to nearest, ties away, after the whole coordinate sum / 4.
//! Present zero, absence and cancellation are not relabelled by this operator;
//! the caller retains the original packets and occurrence validity.
//!
//! This is a restricted geometric operator family, not an arbitrary dense
//! donor matrix or an independently learned paired-H4/Galois companion.
//! Admission allocates; composition uses table reads and checked shift/add.
//! Opcode evidence and whole-model serving qualification are separate.

use std::fmt;

use crate::geometric_value::{GeometricValueError, NativeGeometricValues, ValuePacket, ValueState};
use crate::h4_tables::{H4Code, H4TableError, HistoricalH4Tables};

pub const SCHEMA: &str = "uor-r4.geometric-composition-two-sided-k2-i64-q16/1";
pub const HEADS: usize = 2;
pub const INPUT_LANES: usize = 4;
pub const OUTPUT_LANES: usize = 8;
pub const TERMS: usize = 2;
pub const OUTPUT_WIDTH: usize = 32;
pub const BANK_TERMS: usize = 128;
pub const PACKED_GAIN_BYTES: usize = 64;
pub const MAX_COMPOSED_Q16: i64 = 28_i64 << 30;
pub const MAX_HEAD_SUM_Q16: i64 = MAX_COMPOSED_Q16 << 1;
pub const GAIN_POLICY: &str = "signed-q4[-7,7];reserved-minus8;dimensionless-quarter;low-nibble-first;head-output-input-term/1";
pub const ROUNDING_POLICY: &str = "canonical-atom-Q16;sum-all-terms-atoms-inputs;one-quarter-nearest-ties-away;per-head-reduction-then-exact-head-sum/1";

#[derive(Debug)]
pub enum CompositionError {
    SelectorLength {
        side: &'static str,
        actual: usize,
    },
    InvalidSelector {
        side: &'static str,
        index: usize,
        code: u8,
    },
    GainLength {
        actual: usize,
    },
    InvalidGain {
        index: usize,
        value: i8,
    },
    Head(usize),
    Geometry(H4TableError),
    Decoder(GeometricValueError),
    ArithmeticOverflow,
    CoordinateBound {
        coordinate: usize,
        value: i64,
    },
}
impl fmt::Display for CompositionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "geometric composition: {self:?}")
    }
}
impl std::error::Error for CompositionError {}
pub type CompositionResult<T> = Result<T, CompositionError>;

/// Offline packing only. Selectors use one byte each; no seven-bit packing is
/// claimed. The 128 gains really occupy 64 bytes, with -8 reserved/rejected.
pub fn pack_gains(gains: &[i8]) -> CompositionResult<Vec<u8>> {
    if gains.len() != BANK_TERMS {
        return Err(CompositionError::GainLength {
            actual: gains.len(),
        });
    }
    let mut packed = vec![0; PACKED_GAIN_BYTES];
    for (index, &gain) in gains.iter().enumerate() {
        if !(-7..=7).contains(&gain) {
            return Err(CompositionError::InvalidGain { index, value: gain });
        }
        packed[index >> 1] |= ((gain as u8) & 15) << ((index & 1) << 2);
    }
    Ok(packed)
}

#[derive(Clone, Copy)]
struct Term {
    left: H4Code,
    right_inverse: H4Code,
    gain: i8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompositionStats {
    pub terms: usize,
    pub selector_bytes: usize,
    pub packed_gain_bytes: usize,
    pub artifact_bank_bytes: usize,
    pub runtime_bank_bytes: usize,
    pub geometry_payload_bytes: usize,
    pub coordinate_table_bytes: usize,
}

pub struct NativeGeometricComposition {
    terms: [Term; BANK_TERMS],
    tables: HistoricalH4Tables,
    codec: NativeGeometricValues,
}

impl NativeGeometricComposition {
    /// The exact signed group payload is admitted against its pinned canonical
    /// identity. The enclosing compiler binds these bytes and the selected
    /// bank to the model, tokenizer, producer and reader artifacts.
    pub fn new(
        left: &[u8],
        right: &[u8],
        packed_gains: &[u8],
        h4_payload: &[u8],
    ) -> CompositionResult<Self> {
        for (side, codes) in [("left", left), ("right", right)] {
            if codes.len() != BANK_TERMS {
                return Err(CompositionError::SelectorLength {
                    side,
                    actual: codes.len(),
                });
            }
        }
        if packed_gains.len() != PACKED_GAIN_BYTES {
            return Err(CompositionError::GainLength {
                actual: packed_gains.len(),
            });
        }
        let tables =
            HistoricalH4Tables::from_bytes(h4_payload).map_err(CompositionError::Geometry)?;
        let codec = NativeGeometricValues::canonical().map_err(CompositionError::Decoder)?;
        let mut terms = [Term {
            left: H4Code::IDENTITY,
            right_inverse: H4Code::IDENTITY,
            gain: 0,
        }; BANK_TERMS];
        for index in 0..BANK_TERMS {
            let l =
                H4Code::try_from(left[index]).map_err(|_| CompositionError::InvalidSelector {
                    side: "left",
                    index,
                    code: left[index],
                })?;
            let r =
                H4Code::try_from(right[index]).map_err(|_| CompositionError::InvalidSelector {
                    side: "right",
                    index,
                    code: right[index],
                })?;
            let nibble = (packed_gains[index >> 1] >> ((index & 1) << 2)) & 15;
            let gain = if nibble >= 8 {
                (nibble as i8) - 16
            } else {
                nibble as i8
            };
            if gain == -8 {
                return Err(CompositionError::InvalidGain { index, value: gain });
            }
            terms[index] = Term {
                left: l,
                right_inverse: tables.inverse(r),
                gain,
            };
        }
        Ok(Self {
            terms,
            tables,
            codec,
        })
    }

    pub fn stats(&self) -> CompositionStats {
        CompositionStats {
            terms: BANK_TERMS,
            selector_bytes: BANK_TERMS << 1,
            packed_gain_bytes: PACKED_GAIN_BYTES,
            artifact_bank_bytes: (BANK_TERMS << 1) + PACKED_GAIN_BYTES,
            runtime_bank_bytes: std::mem::size_of_val(&self.terms),
            geometry_payload_bytes: crate::h4_tables::PAYLOAD_BYTES,
            coordinate_table_bytes: crate::geometric_value::RUNTIME_COORDINATE_BYTES,
        }
    }

    /// Transform one occurrence's four K2 lanes for one head. All primitive
    /// radius bins remain admissible, even when the resulting coordinates do
    /// not fit the legacy i32 payload representation. No clipping or re-rooting.
    #[inline(never)]
    pub fn compose(
        &self,
        head: usize,
        packets: &[[ValuePacket; 2]; INPUT_LANES],
    ) -> CompositionResult<[i64; OUTPUT_WIDTH]> {
        if head >= HEADS {
            return Err(CompositionError::Head(head));
        }
        let mut output = [0; OUTPUT_WIDTH];
        let mut index = head << 6;
        for (output_lane, out) in output.chunks_exact_mut(4).enumerate() {
            let mut sum = [0_i64; 4];
            for atoms in packets {
                for _ in 0..TERMS {
                    let term = self.terms[index];
                    index += 1;
                    if term.gain == 0 {
                        continue;
                    }
                    for &atom in atoms {
                        if atom.state() != ValueState::PresentNonzero {
                            continue;
                        }
                        let root = self.tables.compose(
                            self.tables.compose(term.left, atom.root()),
                            term.right_inverse,
                        );
                        let moved = ValuePacket::present_nonzero(root.index(), atom.radius_bin())
                            .map_err(CompositionError::Decoder)?;
                        for (acc, value) in sum.iter_mut().zip(self.codec.decode(moved)) {
                            let contribution = gain_product(term.gain, i64::from(value))?;
                            *acc = acc
                                .checked_add(contribution)
                                .ok_or(CompositionError::ArithmeticOverflow)?;
                        }
                    }
                }
            }
            for (coordinate, (&raw, value)) in sum.iter().zip(out.iter_mut()).enumerate() {
                let magnitude = raw
                    .unsigned_abs()
                    .checked_add(2)
                    .ok_or(CompositionError::ArithmeticOverflow)?
                    >> 2;
                let rounded =
                    i64::try_from(magnitude).map_err(|_| CompositionError::ArithmeticOverflow)?;
                *value = if raw < 0 { -rounded } else { rounded };
                if !(-MAX_COMPOSED_Q16..=MAX_COMPOSED_Q16).contains(value) {
                    return Err(CompositionError::CoordinateBound {
                        coordinate: (output_lane << 2) + coordinate,
                        value: *value,
                    });
                }
            }
        }
        Ok(output)
    }
}

/// Shift/add signed q4 product; its admitted magnitude is at most 7*2^30.
fn gain_product(gain: i8, value: i64) -> CompositionResult<i64> {
    let mut bits = gain.unsigned_abs();
    let mut term = value;
    let mut sum = 0_i64;
    while bits != 0 {
        if bits & 1 != 0 {
            sum = sum
                .checked_add(term)
                .ok_or(CompositionError::ArithmeticOverflow)?;
        }
        bits >>= 1;
        if bits != 0 {
            term = term
                .checked_add(term)
                .ok_or(CompositionError::ArithmeticOverflow)?;
        }
    }
    if gain < 0 {
        sum.checked_neg()
            .ok_or(CompositionError::ArithmeticOverflow)
    } else {
        Ok(sum)
    }
}

/// Normalize each head separately BEFORE this exact sum. Neither a shared
/// denominator nor averaging heads implements this declared operation.
#[inline(never)]
pub fn sum_heads(
    first: &[i64; OUTPUT_WIDTH],
    second: &[i64; OUTPUT_WIDTH],
) -> CompositionResult<[i64; OUTPUT_WIDTH]> {
    let mut output = [0; OUTPUT_WIDTH];
    for (coordinate, ((&a, &b), out)) in first.iter().zip(second).zip(output.iter_mut()).enumerate()
    {
        for value in [a, b] {
            if !(-MAX_COMPOSED_Q16..=MAX_COMPOSED_Q16).contains(&value) {
                return Err(CompositionError::CoordinateBound { coordinate, value });
            }
        }
        *out = a
            .checked_add(b)
            .ok_or(CompositionError::ArithmeticOverflow)?;
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    type TestResult = Result<(), Box<dyn std::error::Error>>;
    fn payload() -> Result<Vec<u8>, std::io::Error> {
        std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/historical-h4-tables-v1.bin"
        ))
    }
    fn blank() -> [[ValuePacket; 2]; INPUT_LANES] {
        [[ValuePacket::present_zero(); 2]; INPUT_LANES]
    }

    #[test]
    fn native_geometric_composition_signed_order_inverse_and_head_layout() -> TestResult {
        let mut left = [1; BANK_TERMS];
        let mut right = [1; BANK_TERMS];
        let mut gains = [0; BANK_TERMS];
        // i * j * inverse(j) = i; omitting inverse gives -i.
        left[0] = 3;
        right[0] = 5;
        gains[0] = 4;
        // Head 1, last output lane, input lane 3, term 1: j*i=-k.
        left[127] = 5;
        gains[127] = 4;
        let bank =
            NativeGeometricComposition::new(&left, &right, &pack_gains(&gains)?, &payload()?)?;
        let mut atoms = blank();
        atoms[0][0] = ValuePacket::present_nonzero(5, 16)?;
        atoms[3][1] = ValuePacket::present_nonzero(3, 16)?;
        let a = bank.compose(0, &atoms)?;
        assert_eq!(&a[..4], &[0, 1 << 16, 0, 0]);
        assert!(a[4..].iter().all(|&x| x == 0));
        let b = bank.compose(1, &atoms)?;
        assert_eq!(&b[28..], &[0, 0, 0, -(1 << 16)]);
        assert!(b[..28].iter().all(|&x| x == 0));
        atoms[0][0] = ValuePacket::present_nonzero(4, 16)?; // -j
        assert_eq!(&bank.compose(0, &atoms)?[..4], &[0, -(1 << 16), 0, 0]);
        Ok(())
    }

    #[test]
    fn native_geometric_composition_one_round_zero_cancellation_and_full_radius() -> TestResult {
        let mut gains = [0; BANK_TERMS];
        gains[0] = 1;
        gains[1] = 1;
        let bank = NativeGeometricComposition::new(
            &[1; BANK_TERMS],
            &[1; BANK_TERMS],
            &pack_gains(&gains)?,
            &payload()?,
        )?;
        let mut atoms = blank();
        atoms[0][0] = ValuePacket::present_nonzero(1, 0)?;
        // Two quarters sum to one half ->1; rounding each quarter would give0.
        assert_eq!(bank.compose(0, &atoms)?[0], 1);
        atoms[0][0] = ValuePacket::present_nonzero(0, 0)?;
        assert_eq!(bank.compose(0, &atoms)?[0], -1);
        atoms[0][1] = ValuePacket::present_nonzero(1, 0)?;
        let original = atoms;
        assert_eq!(bank.compose(0, &atoms)?, [0; OUTPUT_WIDTH]);
        assert_eq!(atoms, original); // two nonzero atoms remain authoritative
        assert_eq!(
            bank.compose(0, &[[ValuePacket::absent(); 2]; INPUT_LANES])?,
            [0; OUTPUT_WIDTH]
        );
        let wide = NativeGeometricComposition::new(
            &[1; BANK_TERMS],
            &[1; BANK_TERMS],
            &pack_gains(&[7; BANK_TERMS])?,
            &payload()?,
        )?;
        let full = [[ValuePacket::present_nonzero(1, 30)?; 2]; INPUT_LANES];
        let out = wide.compose(0, &full)?;
        for lane in out.chunks_exact(4) {
            assert_eq!(lane, &[MAX_COMPOSED_Q16, 0, 0, 0]);
        }
        assert!(out[0] > i64::from(i32::MAX));
        assert_eq!(sum_heads(&out, &out)?[0], MAX_HEAD_SUM_Q16);
        let negative = NativeGeometricComposition::new(
            &[1; BANK_TERMS],
            &[1; BANK_TERMS],
            &pack_gains(&[-7; BANK_TERMS])?,
            &payload()?,
        )?;
        assert_eq!(negative.compose(0, &full)?[0], -MAX_COMPOSED_Q16);
        Ok(())
    }

    #[test]
    fn native_geometric_composition_admission_and_storage() -> TestResult {
        let raw = payload()?;
        assert!(pack_gains(&[0; 127]).is_err());
        for gain in [-8, 8, i8::MIN, i8::MAX] {
            assert!(pack_gains(&[gain; BANK_TERMS]).is_err());
        }
        assert!(NativeGeometricComposition::new(
            &[1; 127],
            &[1; BANK_TERMS],
            &[0; PACKED_GAIN_BYTES],
            &raw
        )
        .is_err());
        assert!(NativeGeometricComposition::new(
            &[120; BANK_TERMS],
            &[1; BANK_TERMS],
            &[0; PACKED_GAIN_BYTES],
            &raw
        )
        .is_err());
        assert!(NativeGeometricComposition::new(
            &[1; BANK_TERMS],
            &[1; BANK_TERMS],
            &[0x80; PACKED_GAIN_BYTES],
            &raw
        )
        .is_err());
        let mut bad = raw.clone();
        bad[0] ^= 1;
        assert!(NativeGeometricComposition::new(
            &[1; BANK_TERMS],
            &[1; BANK_TERMS],
            &[0; PACKED_GAIN_BYTES],
            &bad
        )
        .is_err());
        let bank = NativeGeometricComposition::new(
            &[1; BANK_TERMS],
            &[1; BANK_TERMS],
            &pack_gains(&[-7; BANK_TERMS])?,
            &raw,
        )?;
        assert!(bank.compose(HEADS, &blank()).is_err());
        assert_eq!(bank.stats().artifact_bank_bytes, 320);
        assert_eq!(bank.stats().runtime_bank_bytes, 384);
        assert!(sum_heads(&[MAX_COMPOSED_Q16 + 1; OUTPUT_WIDTH], &[0; OUTPUT_WIDTH]).is_err());
        Ok(())
    }
}
