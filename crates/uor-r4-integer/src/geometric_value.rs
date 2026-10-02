//! K1 fixed-basis signed-H4 values with a dyadic radius and explicit presence.
//!
//! Admission regenerates Q16 coordinates using exact integer bounds for sqrt(5).
//! It uses no floating approximation to decide rounding. The immutable numerical
//! decoder performs only selected table reads; it does not fit or choose packets.
//! Signed root IDs remain authoritative even when rounded coordinates collide.

use std::fmt;

use crate::h4_classifier::H4_ROOT_COEFFICIENTS;
use crate::h4_tables::{H4Code, ROOT_COUNT};
use crate::math::isqrt;

pub const SCHEMA: &str = "uor-r4.geometric-value-k1/1";
pub const FRACTIONAL_BITS: u32 = 16;
pub const MIN_RADIUS_EXPONENT: i8 = -16;
pub const MAX_RADIUS_EXPONENT: i8 = 14;
pub const RADIUS_COUNT: usize = 31;
pub const COORDINATES: usize = 4;
pub const TABLE_ENTRIES: usize = ROOT_COUNT * RADIUS_COUNT * COORDINATES;
pub const TABLE_BYTES: usize = TABLE_ENTRIES * 4;
/// One extra zero row per root makes the admitted numerical stride a power of 2.
pub const RADIUS_STRIDE: usize = 32;
pub const RUNTIME_COORDINATE_BYTES: usize = ROOT_COUNT * RADIUS_STRIDE * COORDINATES * 4;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GeometricValueError {
    InvalidRoot(u8),
    InvalidRadiusBin(u8),
    NoncanonicalEmptyPacket,
    InvalidTableLength(usize),
    NoncanonicalTable {
        entry: usize,
    },
    InvalidCoefficient {
        a: i8,
        b: i8,
    },
    InvalidExponent(i8),
    ArithmeticOverflow,
    PairCoordinateOverflow {
        coordinate: usize,
    },
    AmbiguousCoordinate {
        root: usize,
        radius_bin: usize,
        coordinate: usize,
    },
}

impl fmt::Display for GeometricValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRoot(root) => write!(f, "geometric value root {root} is outside 0..120"),
            Self::InvalidRadiusBin(bin) => write!(f, "geometric value radius bin {bin} is outside 0..31"),
            Self::NoncanonicalEmptyPacket => write!(f, "absent and present-zero values require root 1 and radius bin 0"),
            Self::InvalidTableLength(length) => write!(f, "geometric value table has {length} bytes, expected {TABLE_BYTES}"),
            Self::NoncanonicalTable { entry } => write!(f, "geometric value coordinate entry {entry} differs from exact canonical rounding"),
            Self::InvalidCoefficient { a, b } => write!(f, "geometric value coefficient ({a},{b}) exceeds the pinned H4 bounds"),
            Self::InvalidExponent(exponent) => write!(f, "geometric value exponent {exponent} is outside -16..=14"),
            Self::ArithmeticOverflow => write!(f, "geometric value table arithmetic overflow"),
            Self::PairCoordinateOverflow { coordinate } => write!(f, "geometric value pair coordinate {coordinate} is outside signed Q16 i32"),
            Self::AmbiguousCoordinate { root, radius_bin, coordinate } => write!(f, "geometric value interval has ambiguous rounding at root {root}, radius {radius_bin}, coordinate {coordinate}"),
        }
    }
}

impl std::error::Error for GeometricValueError {}

pub type ValueResult<T> = Result<T, GeometricValueError>;

/// Presence is separate from the decoder's approximate coordinate vector.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueState {
    Absent,
    PresentZero,
    PresentNonzero,
}

/// A single 4-coordinate value lane. Construction validates all codes, including
/// canonical placeholders for states without a nonzero geometric value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValuePacket {
    state: ValueState,
    root: H4Code,
    radius_bin: u8,
}

impl ValuePacket {
    pub fn new(state: ValueState, root: u8, radius_bin: u8) -> ValueResult<Self> {
        let root = H4Code::try_from(root).map_err(|_| GeometricValueError::InvalidRoot(root))?;
        if usize::from(radius_bin) >= RADIUS_COUNT {
            return Err(GeometricValueError::InvalidRadiusBin(radius_bin));
        }
        if state != ValueState::PresentNonzero && (root != H4Code::IDENTITY || radius_bin != 0) {
            return Err(GeometricValueError::NoncanonicalEmptyPacket);
        }
        Ok(Self {
            state,
            root,
            radius_bin,
        })
    }

    pub const fn absent() -> Self {
        Self {
            state: ValueState::Absent,
            root: H4Code::IDENTITY,
            radius_bin: 0,
        }
    }

    pub const fn present_zero() -> Self {
        Self {
            state: ValueState::PresentZero,
            root: H4Code::IDENTITY,
            radius_bin: 0,
        }
    }

    pub fn present_nonzero(root: u8, radius_bin: u8) -> ValueResult<Self> {
        Self::new(ValueState::PresentNonzero, root, radius_bin)
    }

    pub const fn state(self) -> ValueState {
        self.state
    }
    pub const fn root(self) -> H4Code {
        self.root
    }
    pub const fn radius_bin(self) -> u8 {
        self.radius_bin
    }
    /// Only meaningful for PresentNonzero; other states use the canonical bin 0.
    pub const fn radius_exponent(self) -> i8 {
        MIN_RADIUS_EXPONENT + self.radius_bin as i8
    }
}

/// Immutable admitted coordinates. Serialized order is root/radius/coordinate,
/// little-endian i32, with no padding. Runtime rows have a zero bin-31 pad for
/// shift/add indexing; packets can never name that pad. Allocation and exact
/// table construction are admission work, outside the numerical decoder.
pub struct NativeGeometricValues {
    rows: Box<[[i32; COORDINATES]]>,
}

impl NativeGeometricValues {
    pub fn canonical() -> ValueResult<Self> {
        let sqrt5_lower = isqrt(5u128 << 124);
        let sqrt5_lower =
            i128::try_from(sqrt5_lower).map_err(|_| GeometricValueError::ArithmeticOverflow)?;
        let mut rows = vec![[0; COORDINATES]; ROOT_COUNT * RADIUS_STRIDE];
        for (root, coefficients) in H4_ROOT_COEFFICIENTS.iter().enumerate() {
            for radius_bin in 0..RADIUS_COUNT {
                let exponent = MIN_RADIUS_EXPONENT + radius_bin as i8;
                for (coordinate, &[a, b]) in coefficients.iter().enumerate() {
                    let (lower, upper) = rounded_coordinate_bounds(a, b, exponent, sqrt5_lower)?;
                    if lower != upper {
                        return Err(GeometricValueError::AmbiguousCoordinate {
                            root,
                            radius_bin,
                            coordinate,
                        });
                    }
                    rows[(root << 5) + radius_bin][coordinate] = lower;
                }
            }
        }
        Ok(Self {
            rows: rows.into_boxed_slice(),
        })
    }

    /// Regeneration binds the bytes to the existing historical signed H4
    /// coefficient order and this declared rounding rule, not a caller digest.
    /// The enclosing model loader owns source/geometry provenance and file seals.
    pub fn from_bytes(bytes: &[u8]) -> ValueResult<Self> {
        if bytes.len() != TABLE_BYTES {
            return Err(GeometricValueError::InvalidTableLength(bytes.len()));
        }
        let canonical = Self::canonical()?;
        let mut entries = bytes.chunks_exact(4);
        let mut entry = 0;
        for root in 0..ROOT_COUNT {
            for radius_bin in 0..RADIUS_COUNT {
                for &expected in &canonical.rows[(root << 5) + radius_bin] {
                    let raw = entries
                        .next()
                        .ok_or(GeometricValueError::InvalidTableLength(bytes.len()))?;
                    let actual = i32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]);
                    if actual != expected {
                        return Err(GeometricValueError::NoncanonicalTable { entry });
                    }
                    entry += 1;
                }
            }
        }
        Ok(canonical)
    }

    /// Offline artifact construction. The numerical decoder never serializes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(TABLE_BYTES);
        for root in 0..ROOT_COUNT {
            for radius_bin in 0..RADIUS_COUNT {
                for value in self.rows[(root << 5) + radius_bin] {
                    bytes.extend_from_slice(&value.to_le_bytes());
                }
            }
        }
        bytes
    }

    /// Successful numerical path: no allocation, floating point, multiply or
    /// divide. Never infer presence/root identity again from rounded output.
    #[inline(never)]
    pub fn decode(&self, packet: ValuePacket) -> [i32; COORDINATES] {
        if packet.state != ValueState::PresentNonzero {
            return [0; COORDINATES];
        }
        self.rows[(usize::from(packet.root.index()) << 5) + usize::from(packet.radius_bin)]
    }

    /// Decode one base atom plus one residual atom in the same fixed basis.
    /// The sum of two already-rounded Q16 coordinates is exact in i64; there
    /// is no second rounding, saturation or normalization. All coordinates
    /// must fit i32 before a result is exposed. In particular, -2^31 is valid
    /// while +2^31 is rejected. This numerical path allocates nothing.
    ///
    /// Both primitive packet identities remain authoritative. Cancellation
    /// does not turn their states into PresentZero or Absent. The enclosing
    /// occurrence owns aggregate validity separately from these coordinates;
    /// this decoder never infers it from a zero sum. Absent atoms contribute
    /// numerical zero, and a PresentZero residual preserves the K1 result.
    #[inline(never)]
    pub fn decode_pair(
        &self,
        base: ValuePacket,
        residual: ValuePacket,
    ) -> ValueResult<[i32; COORDINATES]> {
        let base_coordinates = self.decode(base);
        let residual_coordinates = self.decode(residual);
        let mut result = [0; COORDINATES];
        for coordinate in 0..COORDINATES {
            let sum = i64::from(base_coordinates[coordinate])
                + i64::from(residual_coordinates[coordinate]);
            result[coordinate] = i32::try_from(sum)
                .map_err(|_| GeometricValueError::PairCoordinateOverflow { coordinate })?;
        }
        Ok(result)
    }
}

/// Exact rational nearest/ties-away rounding. Table construction bounds shift
/// to 34..64, numerator magnitude below 2^65 and output magnitude at most 2^30.
fn round_dyadic(numerator: i128, shift: u32) -> ValueResult<i32> {
    let magnitude = numerator.unsigned_abs();
    let half = 1u128
        .checked_shl(shift - 1)
        .ok_or(GeometricValueError::ArithmeticOverflow)?;
    let rounded = magnitude
        .checked_add(half)
        .ok_or(GeometricValueError::ArithmeticOverflow)?
        >> shift;
    let signed = i128::try_from(rounded).map_err(|_| GeometricValueError::ArithmeticOverflow)?;
    i32::try_from(if numerator < 0 { -signed } else { signed })
        .map_err(|_| GeometricValueError::ArithmeticOverflow)
}

fn rounded_coordinate_bounds(
    a: i8,
    b: i8,
    exponent: i8,
    sqrt5_lower: i128,
) -> ValueResult<(i32, i32)> {
    if !(-2..=2).contains(&a) || !(-1..=1).contains(&b) {
        return Err(GeometricValueError::InvalidCoefficient { a, b });
    }
    if !(MIN_RADIUS_EXPONENT..=MAX_RADIUS_EXPONENT).contains(&exponent) {
        return Err(GeometricValueError::InvalidExponent(exponent));
    }
    // (a+b*phi)/2 * 2^e * 2^16 = (2a+b+b*sqrt(5))*2^(e+14).
    // S=floor(sqrt(5*2^124)) brackets sqrt(5) strictly between S/2^62
    // and (S+1)/2^62. Ordered signed endpoints divided by 2^(48-e)
    // therefore contain the exact coordinate. Monotone ties-away rounding of
    // equal endpoint results proves the rounded integer without evaluating phi.
    let base = (i128::from(a) + i128::from(a) + i128::from(b)) << 62;
    let first = base
        .checked_add(
            i128::from(b)
                .checked_mul(sqrt5_lower)
                .ok_or(GeometricValueError::ArithmeticOverflow)?,
        )
        .ok_or(GeometricValueError::ArithmeticOverflow)?;
    let second = if b == 0 {
        first
    } else {
        first
            .checked_add(i128::from(b))
            .ok_or(GeometricValueError::ArithmeticOverflow)?
    };
    let shift = (48i16 - i16::from(exponent)) as u32;
    Ok((
        round_dyadic(first.min(second), shift)?,
        round_dyadic(first.max(second), shift)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometric_value_exact_endpoints_ties_and_golden_coordinates() {
        let values = NativeGeometricValues::canonical().unwrap();
        assert_eq!(
            values.decode(ValuePacket::present_nonzero(1, 0).unwrap()),
            [1, 0, 0, 0]
        );
        assert_eq!(
            values.decode(ValuePacket::present_nonzero(0, 30).unwrap()),
            [-(1 << 30), 0, 0, 0]
        );
        // Every all-halves root is at the exact rational ±1/2 tie at e=-16.
        for root in 8..24 {
            let decoded = values.decode(ValuePacket::present_nonzero(root, 0).unwrap());
            for (coordinate, code) in decoded.iter().enumerate() {
                assert_eq!(
                    *code,
                    i32::from(H4_ROOT_COEFFICIENTS[usize::from(root)][coordinate][0])
                );
            }
        }
        let lower = isqrt(5u128 << 124) as i128;
        assert_eq!(
            rounded_coordinate_bounds(0, 1, 0, lower).unwrap(),
            (53020, 53020)
        );
        assert_eq!(
            rounded_coordinate_bounds(-1, 1, 0, lower).unwrap(),
            (20252, 20252)
        );
        assert_eq!(
            rounded_coordinate_bounds(0, -1, 0, lower).unwrap(),
            (-53020, -53020)
        );
        assert_eq!(round_dyadic(1, 1).unwrap(), 1);
        assert_eq!(round_dyadic(-1, 1).unwrap(), -1);
        assert_eq!(round_dyadic(1, 2).unwrap(), 0);
        assert_eq!(round_dyadic(-1, 2).unwrap(), 0);
    }

    #[test]
    fn geometric_value_all_radii_preserve_signed_antipodes_and_bounds() {
        let values = NativeGeometricValues::canonical().unwrap();
        let radicand = 5u128 << 124;
        let lower = isqrt(radicand);
        assert!(lower * lower < radicand);
        assert!((lower + 1) * (lower + 1) > radicand);
        for (root, coefficients) in H4_ROOT_COEFFICIENTS.iter().enumerate() {
            let opposite = coefficients.map(|coordinate| coordinate.map(|x| -x));
            let antipode = H4_ROOT_COEFFICIENTS
                .iter()
                .position(|candidate| *candidate == opposite)
                .unwrap();
            for bin in 0..RADIUS_COUNT {
                let packet = ValuePacket::present_nonzero(root as u8, bin as u8).unwrap();
                assert_eq!(packet.radius_exponent(), MIN_RADIUS_EXPONENT + bin as i8);
                let decoded = values.decode(packet);
                assert!(decoded.iter().all(|&value| value.unsigned_abs() <= 1 << 30));
                assert_eq!(
                    values.decode(ValuePacket::present_nonzero(antipode as u8, bin as u8).unwrap()),
                    decoded.map(|x| -x)
                );
            }
        }
    }

    #[test]
    fn geometric_value_admission_regenerates_every_coordinate_and_rejects_padding() {
        let values = NativeGeometricValues::canonical().unwrap();
        let bytes = values.to_bytes();
        assert_eq!(bytes.len(), TABLE_BYTES);
        assert_eq!(
            NativeGeometricValues::from_bytes(&bytes)
                .unwrap()
                .to_bytes(),
            bytes
        );
        let mut corrupt = bytes.clone();
        corrupt[TABLE_BYTES - 1] ^= 1;
        assert!(
            matches!(NativeGeometricValues::from_bytes(&corrupt), Err(GeometricValueError::NoncanonicalTable { entry }) if entry == TABLE_ENTRIES - 1)
        );
        assert!(matches!(
            NativeGeometricValues::from_bytes(&bytes[..TABLE_BYTES - 1]),
            Err(GeometricValueError::InvalidTableLength(_))
        ));
        assert!(matches!(
            ValuePacket::present_nonzero(0, 31),
            Err(GeometricValueError::InvalidRadiusBin(31))
        ));
        assert!(matches!(
            ValuePacket::present_nonzero(120, 0),
            Err(GeometricValueError::InvalidRoot(120))
        ));
    }

    #[test]
    fn geometric_value_presence_and_root_identity_do_not_come_from_rounded_coordinates() {
        let values = NativeGeometricValues::canonical().unwrap();
        let absent = ValuePacket::absent();
        let zero = ValuePacket::present_zero();
        let identity = ValuePacket::present_nonzero(1, 0).unwrap();
        assert_ne!(absent, zero);
        assert_ne!(zero, identity);
        assert_eq!(values.decode(absent), values.decode(zero));
        assert_eq!(identity.state(), ValueState::PresentNonzero);
        assert_eq!(identity.root(), H4Code::IDENTITY);
        assert_eq!(values.decode(identity), [1, 0, 0, 0]);
        assert!(matches!(
            ValuePacket::new(ValueState::Absent, 0, 0),
            Err(GeometricValueError::NoncanonicalEmptyPacket)
        ));
        assert!(matches!(
            ValuePacket::new(ValueState::PresentZero, 1, 1),
            Err(GeometricValueError::NoncanonicalEmptyPacket)
        ));
        // At the smallest radius some nonzero exact coordinates round to zero.
        let root = H4_ROOT_COEFFICIENTS
            .iter()
            .position(|row| row.contains(&[-1, 1]))
            .unwrap();
        let coordinate = H4_ROOT_COEFFICIENTS[root]
            .iter()
            .position(|&entry| entry == [-1, 1])
            .unwrap();
        let packet = ValuePacket::present_nonzero(root as u8, 0).unwrap();
        assert_eq!(values.decode(packet)[coordinate], 0);
        assert_eq!(packet.state(), ValueState::PresentNonzero);
        assert_eq!(packet.root().index(), root as u8);
    }

    #[test]
    fn geometric_value_pair_checks_positive_overflow_and_includes_negative_endpoint() {
        let values = NativeGeometricValues::canonical().unwrap();
        for coordinate in 0..COORDINATES {
            let positive = ValuePacket::present_nonzero((2 * coordinate + 1) as u8, 30).unwrap();
            let negative = ValuePacket::present_nonzero((2 * coordinate) as u8, 30).unwrap();
            let before = values.decode(positive);
            assert_eq!(
                values.decode_pair(positive, positive),
                Err(GeometricValueError::PairCoordinateOverflow { coordinate })
            );
            assert_eq!(values.decode(positive), before);
            let mut expected = [0; COORDINATES];
            expected[coordinate] = i32::MIN;
            assert_eq!(values.decode_pair(negative, negative).unwrap(), expected);
        }
    }

    #[test]
    fn geometric_value_pair_zero_residual_preserves_k1_and_explicit_states() {
        let values = NativeGeometricValues::canonical().unwrap();
        let zero = ValuePacket::present_zero();
        let absent = ValuePacket::absent();
        for base in [
            absent,
            zero,
            ValuePacket::present_nonzero(1, 0).unwrap(),
            ValuePacket::present_nonzero(0, 30).unwrap(),
            ValuePacket::present_nonzero(24, 0).unwrap(),
            ValuePacket::present_nonzero(119, 30).unwrap(),
        ] {
            assert_eq!(values.decode_pair(base, zero).unwrap(), values.decode(base));
            assert_eq!(values.decode_pair(zero, base).unwrap(), values.decode(base));
            assert_eq!(
                values.decode_pair(base, absent).unwrap(),
                values.decode(base)
            );
        }
        assert_eq!(
            values.decode_pair(absent, absent).unwrap(),
            [0; COORDINATES]
        );
        assert_eq!(values.decode_pair(zero, zero).unwrap(), [0; COORDINATES]);
        assert_eq!(absent.state(), ValueState::Absent);
        assert_eq!(zero.state(), ValueState::PresentZero);
    }

    #[test]
    fn geometric_value_pair_antipodal_cancellation_preserves_nonzero_atoms() {
        let values = NativeGeometricValues::canonical().unwrap();
        for root in [1usize, 3, 24, 53, 119] {
            let opposite = H4_ROOT_COEFFICIENTS[root].map(|c| c.map(|x| -x));
            let opposite_root = H4_ROOT_COEFFICIENTS
                .iter()
                .position(|candidate| *candidate == opposite)
                .unwrap();
            for radius_bin in [0, 16, 30] {
                let base = ValuePacket::present_nonzero(root as u8, radius_bin).unwrap();
                let residual =
                    ValuePacket::present_nonzero(opposite_root as u8, radius_bin).unwrap();
                assert_eq!(
                    values.decode_pair(base, residual).unwrap(),
                    [0; COORDINATES]
                );
                assert_eq!(base.state(), ValueState::PresentNonzero);
                assert_eq!(residual.state(), ValueState::PresentNonzero);
                assert_ne!(base.root(), residual.root());
                assert_eq!(base.radius_bin(), radius_bin);
                assert_eq!(residual.radius_bin(), radius_bin);
            }
        }
    }
}
