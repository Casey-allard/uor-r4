//! Strict signed-q4 source for the existing seven-family geometric potential.
//!
//! Free coefficients are q in [-7,7] in fixed quarter-nat units. Packing is
//! family-major, then head/lane/component. Expanded tables retain the existing
//! head/lane/CU120/RU120/pair128x128/CR1024/RR1024/CP4/RP4 layout. Every padded
//! pair entry is zero. Each angular family rounds once, nearest/ties-away:
//! unary Q24 = sum(q*B)/8; cross Q24 = sum(q*B*C)/2^28; scalar Q24 = q<<22.
//!
//! B is the pinned canonical-F32 Q25 observation basis, NOT exact Z[phi] and
//! NOT the legacy potential's F64 observation basis. Exact signed H4 identity,
//! inverse and multiplication stay in the independently admitted group tables.
//! No score centering, scaling, input classification or new read kernel occurs.
//!
//! Admission uses checked integer arithmetic and may allocate/multiply while
//! constructing fixed tables. Numerical scoring delegates to the existing
//! lookup/add kernel. No compiler opcode or whole-model serving claim follows.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::geometric_no_read::CANONICAL_BASIS_Q25;
use crate::geometric_potential::{
    AddressLane, NativePotentialTables, PotentialError, CONTENT_PRESENCE_OFFSET,
    CONTENT_RADIUS_OFFSET, CONTENT_UNARY_OFFSET, CONTEXT_PRESENCE_OFFSET, CONTEXT_RADIUS_OFFSET,
    CONTEXT_UNARY_OFFSET, ENTRIES_PER_LANE, MAX_HEADS, MAX_LANES_PER_HEAD, PAIR_OFFSET,
    PAIR_STRIDE,
};
use crate::h4_tables::HistoricalH4Tables;

pub const SCHEMA: &str = "uor-r4.geometric-potential-q4/1";
pub const POLICY: &str = "signed-q4[-7,7];reserved-minus8;fixed-quarter-nat;canonical-F32-Q25-observation;unary-div8/cross-div2^28/scalar-shift22;per-family-Q24-nearest-ties-away;no-centering;family-major-seven/1";
pub const FRACTIONAL_BITS: u32 = 24;
pub const COEFFICIENT_SHIFT: u32 = 22;
pub const ROOT_COUNT: usize = 120;
pub const FAMILY_NAMES: [&str; 7] = [
    "content_unary",
    "context_unary",
    "pair",
    "content_radius",
    "context_radius",
    "content_presence",
    "context_presence",
];
pub const FAMILY_COUNTS: [usize; 7] = [4, 4, 16, 1024, 1024, 4, 4];
pub const COEFFICIENTS_PER_LANE: usize = 2080;
/// Two unaries <=3.5 each, cross <=7, four scalars <=1.75 each.
pub const MAX_ABS_LANE_SCORE_Q24: i64 = 21i64 << FRACTIONAL_BITS;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PotentialQ4Config {
    pub heads: usize,
    pub lanes_per_head: usize,
}
impl PotentialQ4Config {
    pub fn validate(self) -> PotentialQ4Result<()> {
        if !(1..=MAX_HEADS).contains(&self.heads)
            || !(1..=MAX_LANES_PER_HEAD).contains(&self.lanes_per_head)
        {
            return Err(PotentialQ4Error::Configuration);
        }
        Ok(())
    }
    /// This order, not alphabetical map order, defines the source bytes.
    pub fn coefficient_shapes(self) -> PotentialQ4Result<Vec<(String, Vec<usize>)>> {
        self.validate()?;
        let (h, l) = (self.heads, self.lanes_per_head);
        let shapes = [
            vec![h, l, 4],
            vec![h, l, 4],
            vec![h, l, 4, 4],
            vec![h, l, 32, 32],
            vec![h, l, 32, 32],
            vec![h, l, 4],
            vec![h, l, 4],
        ];
        Ok(FAMILY_NAMES
            .iter()
            .zip(shapes)
            .map(|(&name, shape)| (name.to_owned(), shape))
            .collect())
    }
    pub fn coefficient_count(self) -> PotentialQ4Result<usize> {
        self.validate()?;
        self.heads
            .checked_mul(self.lanes_per_head)
            .and_then(|x| x.checked_mul(COEFFICIENTS_PER_LANE))
            .ok_or(PotentialQ4Error::ArithmeticOverflow)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PotentialQ4Error {
    Configuration,
    PackedLength { expected: usize, actual: usize },
    InvalidCoefficient { index: usize, value: i8 },
    NonzeroNibblePadding,
    BasisBounds,
    ArithmeticOverflow,
    TableMismatch,
    Native(PotentialError),
}
impl fmt::Display for PotentialQ4Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "geometric potential q4: {self:?}")
    }
}
impl std::error::Error for PotentialQ4Error {}
pub type PotentialQ4Result<T> = Result<T, PotentialQ4Error>;

/// Pin these LE-i32 bytes separately from the exact H4 algebra identity.
pub fn canonical_basis_q25() -> [[i32; 4]; ROOT_COUNT] {
    CANONICAL_BASIS_Q25
}
fn validate_basis() -> PotentialQ4Result<()> {
    if CANONICAL_BASIS_Q25[0] != [-(1 << 25), 0, 0, 0]
        || CANONICAL_BASIS_Q25[1] != [1 << 25, 0, 0, 0]
    {
        return Err(PotentialQ4Error::BasisBounds);
    }
    for root in CANONICAL_BASIS_Q25 {
        let mut l1 = 0i64;
        for coordinate in root {
            let magnitude = i64::from(coordinate).abs();
            if magnitude > 1 << 25 {
                return Err(PotentialQ4Error::BasisBounds);
            }
            l1 = l1
                .checked_add(magnitude)
                .ok_or(PotentialQ4Error::ArithmeticOverflow)?;
        }
        if l1 > 2 << 25 {
            return Err(PotentialQ4Error::BasisBounds);
        }
    }
    Ok(())
}

/// Signed two's-complement nibble; low nibble first, unused high nibble zero.
pub fn pack_coefficients(values: &[i8]) -> PotentialQ4Result<Vec<u8>> {
    let mut packed = vec![0; values.len().div_ceil(2)];
    for (index, &value) in values.iter().enumerate() {
        if !(-7..=7).contains(&value) {
            return Err(PotentialQ4Error::InvalidCoefficient { index, value });
        }
        packed[index >> 1] |= ((value as u8) & 15) << ((index & 1) << 2);
    }
    Ok(packed)
}
pub fn unpack_coefficients(expected_count: usize, packed: &[u8]) -> PotentialQ4Result<Vec<i8>> {
    let expected = expected_count.div_ceil(2);
    if packed.len() != expected {
        return Err(PotentialQ4Error::PackedLength {
            expected,
            actual: packed.len(),
        });
    }
    if expected_count & 1 != 0 && packed.last().copied().unwrap_or(0) & 0xf0 != 0 {
        return Err(PotentialQ4Error::NonzeroNibblePadding);
    }
    (0..expected_count)
        .map(|index| {
            let nibble = (packed[index >> 1] >> ((index & 1) << 2)) & 15;
            let value = if nibble & 8 == 0 {
                nibble as i8
            } else {
                nibble as i8 - 16
            };
            if value == -8 {
                Err(PotentialQ4Error::InvalidCoefficient { index, value })
            } else {
                Ok(value)
            }
        })
        .collect()
}

fn coefficient_product(q: i8, value: i64) -> PotentialQ4Result<i64> {
    let mut magnitude = q.unsigned_abs();
    let mut term = value;
    let mut result = 0i64;
    while magnitude != 0 {
        if magnitude & 1 != 0 {
            result = result
                .checked_add(term)
                .ok_or(PotentialQ4Error::ArithmeticOverflow)?;
        }
        magnitude >>= 1;
        if magnitude != 0 {
            term = term
                .checked_add(term)
                .ok_or(PotentialQ4Error::ArithmeticOverflow)?;
        }
    }
    if q < 0 {
        result
            .checked_neg()
            .ok_or(PotentialQ4Error::ArithmeticOverflow)
    } else {
        Ok(result)
    }
}
fn rounded_q24(numerator: i64, shift: u32) -> PotentialQ4Result<i32> {
    if shift == 0 || shift >= 64 {
        return Err(PotentialQ4Error::ArithmeticOverflow);
    }
    let magnitude = numerator
        .unsigned_abs()
        .checked_add(1u64 << (shift - 1))
        .ok_or(PotentialQ4Error::ArithmeticOverflow)?
        >> shift;
    let signed = i64::try_from(magnitude).map_err(|_| PotentialQ4Error::ArithmeticOverflow)?;
    let signed = if numerator < 0 {
        signed
            .checked_neg()
            .ok_or(PotentialQ4Error::ArithmeticOverflow)?
    } else {
        signed
    };
    i32::try_from(signed).map_err(|_| PotentialQ4Error::ArithmeticOverflow)
}
fn unary_q24(q: &[i8], root: &[i32; 4]) -> PotentialQ4Result<i32> {
    let mut sum = 0i64;
    for (&q, &b) in q.iter().zip(root) {
        sum = sum
            .checked_add(coefficient_product(q, i64::from(b))?)
            .ok_or(PotentialQ4Error::ArithmeticOverflow)?;
    }
    rounded_q24(sum, 3)
}
// Distribute the exact integer sum before rounding: four weighted columns per
// content root, then dot against each context root. No intermediate rescale.
fn weighted_columns(q: &[i8], root: &[i32; 4]) -> PotentialQ4Result<[i64; 4]> {
    let mut columns = [0i64; 4];
    for (i, &b) in root.iter().enumerate() {
        for (j, column) in columns.iter_mut().enumerate() {
            *column = column
                .checked_add(coefficient_product(q[i * 4 + j], i64::from(b))?)
                .ok_or(PotentialQ4Error::ArithmeticOverflow)?;
        }
    }
    Ok(columns)
}
fn cross_q24(columns: &[i64; 4], root: &[i32; 4]) -> PotentialQ4Result<i32> {
    let mut sum = 0i64;
    for (&a, &b) in columns.iter().zip(root) {
        sum = sum
            .checked_add(
                a.checked_mul(i64::from(b))
                    .ok_or(PotentialQ4Error::ArithmeticOverflow)?,
            )
            .ok_or(PotentialQ4Error::ArithmeticOverflow)?;
    }
    rounded_q24(sum, 28)
}

/// Admission-only ordered bilinear table for sixteen row-major Q4 coefficients.
/// Uses the same pinned Q25 basis, one Q24 rounding and zero padding as pair.
/// Scoring consumes the compiled table; it does not evaluate a runtime product.
pub fn compile_ordered_pair_q4(packed: &[u8]) -> PotentialQ4Result<Vec<i32>> {
    let q = unpack_coefficients(16, packed)?;
    validate_basis()?;
    let mut table = vec![0; PAIR_STRIDE * PAIR_STRIDE];
    for (left, root) in CANONICAL_BASIS_Q25.iter().enumerate() {
        let columns = weighted_columns(&q, root)?;
        for (right, root) in CANONICAL_BASIS_Q25.iter().enumerate() {
            table[(left << 7) + right] = cross_q24(&columns, root)?;
        }
    }
    Ok(table)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PotentialQ4Stats {
    pub learned_coefficients: usize,
    pub packed_bytes: usize,
    /// Derived payload, excluding admission copies, metadata and group tables.
    pub expanded_table_entries: usize,
    pub expanded_table_bytes: usize,
}

/// Strict source admission plus the unchanged existing native scoring engine.
pub struct NativePotentialQ4 {
    config: PotentialQ4Config,
    packed: Box<[u8]>,
    expanded: Vec<i32>,
    native: NativePotentialTables,
    coefficient_count: usize,
}
impl NativePotentialQ4 {
    pub fn new(config: PotentialQ4Config, packed: &[u8]) -> PotentialQ4Result<Self> {
        let coefficient_count = config.coefficient_count()?;
        let coefficients = unpack_coefficients(coefficient_count, packed)?;
        validate_basis()?;
        let lanes = config.heads * config.lanes_per_head;
        let mut offsets = [0usize; 7];
        let mut consumed = 0usize;
        for (offset, count) in offsets.iter_mut().zip(FAMILY_COUNTS) {
            *offset = consumed;
            consumed = consumed
                .checked_add(
                    lanes
                        .checked_mul(count)
                        .ok_or(PotentialQ4Error::ArithmeticOverflow)?,
                )
                .ok_or(PotentialQ4Error::ArithmeticOverflow)?;
        }
        if consumed != coefficients.len() {
            return Err(PotentialQ4Error::ArithmeticOverflow);
        }
        let entries = lanes
            .checked_mul(ENTRIES_PER_LANE)
            .ok_or(PotentialQ4Error::ArithmeticOverflow)?;
        let mut expanded = vec![0; entries];
        for (lane, output) in expanded.chunks_exact_mut(ENTRIES_PER_LANE).enumerate() {
            let family = |f: usize| {
                let start = offsets[f] + lane * FAMILY_COUNTS[f];
                &coefficients[start..start + FAMILY_COUNTS[f]]
            };
            for (c, content) in CANONICAL_BASIS_Q25.iter().enumerate() {
                output[CONTENT_UNARY_OFFSET + c] = unary_q24(family(0), content)?;
                output[CONTEXT_UNARY_OFFSET + c] = unary_q24(family(1), content)?;
                let columns = weighted_columns(family(2), content)?;
                for (r, context) in CANONICAL_BASIS_Q25.iter().enumerate() {
                    output[PAIR_OFFSET + c * PAIR_STRIDE + r] = cross_q24(&columns, context)?;
                }
            }
            for (f, offset) in [
                (3, CONTENT_RADIUS_OFFSET),
                (4, CONTEXT_RADIUS_OFFSET),
                (5, CONTENT_PRESENCE_OFFSET),
                (6, CONTEXT_PRESENCE_OFFSET),
            ] {
                for (dst, &q) in output[offset..offset + FAMILY_COUNTS[f]]
                    .iter_mut()
                    .zip(family(f))
                {
                    *dst = i32::from(q) << COEFFICIENT_SHIFT;
                }
            }
        }
        let native = NativePotentialTables::new(config.heads, config.lanes_per_head, &expanded)
            .map_err(PotentialQ4Error::Native)?;
        Ok(Self {
            config,
            packed: packed.into(),
            expanded,
            native,
            coefficient_count,
        })
    }
    /// Every saved entry must equal regeneration; a resealed table is not a
    /// replacement learned source. This comparison includes all zero padding.
    pub fn from_parts(
        config: PotentialQ4Config,
        packed: &[u8],
        expanded: &[i32],
    ) -> PotentialQ4Result<Self> {
        let admitted = Self::new(config, packed)?;
        if admitted.expanded != expanded {
            return Err(PotentialQ4Error::TableMismatch);
        }
        Ok(admitted)
    }
    pub fn config(&self) -> PotentialQ4Config {
        self.config
    }
    pub fn packed_coefficients(&self) -> &[u8] {
        &self.packed
    }
    pub fn expanded_q24(&self) -> &[i32] {
        &self.expanded
    }
    pub fn table_bytes(&self) -> Vec<u8> {
        self.expanded.iter().flat_map(|x| x.to_le_bytes()).collect()
    }
    pub fn stats(&self) -> PotentialQ4Stats {
        PotentialQ4Stats {
            learned_coefficients: self.coefficient_count,
            packed_bytes: self.packed.len(),
            expanded_table_entries: self.expanded.len(),
            expanded_table_bytes: self.expanded.len() * 4,
        }
    }
    pub fn score(
        &self,
        head: usize,
        cq: &[AddressLane],
        ck: &[AddressLane],
        rq: &[AddressLane],
        rk: &[AddressLane],
        algebra: &HistoricalH4Tables,
    ) -> PotentialQ4Result<i64> {
        self.native
            .score(head, cq, ck, rq, rk, algebra)
            .map_err(PotentialQ4Error::Native)
    }
    pub fn into_native(self) -> PotentialQ4Result<NativePotentialTables> {
        Ok(self.native)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    type TestResult = Result<(), Box<dyn std::error::Error>>;
    fn config(heads: usize, lanes: usize) -> PotentialQ4Config {
        PotentialQ4Config {
            heads,
            lanes_per_head: lanes,
        }
    }
    fn source_index(config: PotentialQ4Config, family: usize, lane: usize, cell: usize) -> usize {
        let lanes = config.heads * config.lanes_per_head;
        FAMILY_COUNTS[..family].iter().sum::<usize>() * lanes + lane * FAMILY_COUNTS[family] + cell
    }
    fn oracle_round(n: i128, shift: u32) -> i32 {
        let denominator = 1u128 << shift;
        let a = n.unsigned_abs();
        let rounded = a / denominator + u128::from(a % denominator >= denominator / 2);
        if n < 0 {
            -(rounded as i32)
        } else {
            rounded as i32
        }
    }

    #[test]
    fn potential_q4_packing_dimensions_and_reserved_codes() -> TestResult {
        let q = [-7, -1, 0, 1, 7];
        let bytes = pack_coefficients(&q)?;
        assert_eq!(bytes, [0xf9, 0x10, 7]);
        assert_eq!(unpack_coefficients(q.len(), &bytes)?, q);
        assert!(pack_coefficients(&[-8]).is_err());
        assert!(pack_coefficients(&[8]).is_err());
        assert!(matches!(
            unpack_coefficients(2, &[0x80]),
            Err(PotentialQ4Error::InvalidCoefficient {
                index: 1,
                value: -8
            })
        ));
        assert_eq!(
            unpack_coefficients(1, &[0x10]),
            Err(PotentialQ4Error::NonzeroNibblePadding)
        );
        assert!(unpack_coefficients(2, &[]).is_err());
        assert!(unpack_coefficients(2, &[0, 0]).is_err());
        assert!(unpack_coefficients(0, &[])?.is_empty());
        for invalid in [
            config(0, 1),
            config(MAX_HEADS + 1, 1),
            config(1, 0),
            config(1, MAX_LANES_PER_HEAD + 1),
        ] {
            assert!(matches!(
                NativePotentialQ4::new(invalid, &[]),
                Err(PotentialQ4Error::Configuration)
            ));
        }
        assert_eq!(config(2, 4).coefficient_count()?, 16640);
        assert_eq!(config(2, 4).coefficient_count()?.div_ceil(2), 8320);
        let shapes = config(2, 4).coefficient_shapes()?;
        assert_eq!(
            shapes.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>(),
            FAMILY_NAMES
        );
        assert_eq!(
            shapes
                .iter()
                .map(|(_, s)| s.iter().product::<usize>())
                .sum::<usize>(),
            16640
        );
        assert_eq!(
            config(MAX_HEADS, MAX_LANES_PER_HEAD).coefficient_count()?,
            4259840
        );
        Ok(())
    }

    #[test]
    fn potential_q4_exact_signed_expansion_and_rounding() -> TestResult {
        validate_basis()?;
        for shift in [3, 28] {
            let half = 1i64 << (shift - 1);
            for n in [
                half - 1,
                half,
                half + 1,
                3 * half,
                -half + 1,
                -half,
                -half - 1,
                -3 * half,
            ] {
                assert_eq!(rounded_q24(n, shift)?, oracle_round(i128::from(n), shift));
            }
        }
        let unary = [1, -2, 3, -4];
        let pair = [7, -6, 5, -4, -3, 2, -1, 0, 1, 2, -3, -4, 5, -6, 7, -7];
        for a in canonical_basis_q25() {
            let n = unary
                .iter()
                .zip(a)
                .map(|(&q, b)| i128::from(q) * i128::from(b))
                .sum();
            assert_eq!(unary_q24(&unary, &a)?, oracle_round(n, 3));
            assert_eq!(unary_q24(&unary, &a.map(|v| -v))?, -unary_q24(&unary, &a)?);
            let columns = weighted_columns(&pair, &a)?;
            for b in canonical_basis_q25() {
                let mut n = 0i128;
                for i in 0..4 {
                    for j in 0..4 {
                        n += i128::from(pair[i * 4 + j]) * i128::from(a[i]) * i128::from(b[j]);
                    }
                }
                assert!(n.abs() <= 7i128 << 52);
                let actual = cross_q24(&columns, &b)?;
                assert_eq!(actual, oracle_round(n, 28));
                assert_eq!(cross_q24(&columns, &b.map(|v| -v))?, -actual);
                assert!(i64::from(actual).abs() <= 7i64 << FRACTIONAL_BITS);
            }
        }
        assert!(rounded_q24(i64::MAX, 3).is_err());
        assert!(rounded_q24(0, 0).is_err());
        assert!(coefficient_product(7, i64::MAX).is_err());
        Ok(())
    }

    #[test]
    fn potential_q4_layout_padding_and_independent_regeneration() -> TestResult {
        let config = config(2, 2);
        let q = (0..config.coefficient_count()?)
            .map(|i| (i % 15) as i8 - 7)
            .collect::<Vec<_>>();
        let packed = pack_coefficients(&q)?;
        let admitted = NativePotentialQ4::new(config, &packed)?;
        let flat = admitted.expanded_q24();
        assert_eq!(
            admitted.stats().expanded_table_entries,
            4 * ENTRIES_PER_LANE
        );
        assert_eq!(admitted.table_bytes().len(), flat.len() * 4);
        for (lane, row) in flat.chunks_exact(ENTRIES_PER_LANE).enumerate() {
            for (family, offset) in [
                (3, CONTENT_RADIUS_OFFSET),
                (4, CONTEXT_RADIUS_OFFSET),
                (5, CONTENT_PRESENCE_OFFSET),
                (6, CONTEXT_PRESENCE_OFFSET),
            ] {
                for cell in 0..FAMILY_COUNTS[family] {
                    assert_eq!(
                        row[offset + cell],
                        i32::from(q[source_index(config, family, lane, cell)]) << 22
                    );
                }
            }
            for (family, offset) in [(0, CONTENT_UNARY_OFFSET), (1, CONTEXT_UNARY_OFFSET)] {
                let start = source_index(config, family, lane, 0);
                for (root, basis) in canonical_basis_q25().iter().enumerate() {
                    assert_eq!(row[offset + root], unary_q24(&q[start..start + 4], basis)?);
                }
            }
            for c in 0..PAIR_STRIDE {
                for r in 0..PAIR_STRIDE {
                    if c >= ROOT_COUNT || r >= ROOT_COUNT {
                        assert_eq!(row[PAIR_OFFSET + c * PAIR_STRIDE + r], 0);
                    }
                }
            }
            let start = source_index(config, 2, lane, 0);
            let columns = weighted_columns(&q[start..start + 16], &canonical_basis_q25()[17])?;
            assert_eq!(
                row[PAIR_OFFSET + 17 * PAIR_STRIDE + 83],
                cross_q24(&columns, &canonical_basis_q25()[83])?
            );
        }
        let restored = NativePotentialQ4::from_parts(config, &packed, flat)?;
        assert_eq!(restored.packed_coefficients(), packed);
        let mut altered = flat.to_vec();
        altered[PAIR_OFFSET + 127] = 1;
        assert!(matches!(
            NativePotentialQ4::from_parts(config, &packed, &altered),
            Err(PotentialQ4Error::TableMismatch)
        ));
        assert!(NativePotentialTables::new(2, 2, &altered).is_err());
        altered = flat.to_vec();
        altered[CONTENT_UNARY_OFFSET] += 1;
        assert!(matches!(
            NativePotentialQ4::from_parts(config, &packed, &altered),
            Err(PotentialQ4Error::TableMismatch)
        ));
        let native = restored.into_native()?;
        assert_eq!((native.heads(), native.lanes()), (2, 2));
        Ok(())
    }

    #[test]
    fn potential_q4_existing_scorer_preserves_order_presence_and_all_families() -> TestResult {
        let config = config(1, 1);
        let mut q = vec![0; config.coefficient_count()?];
        q[source_index(config, 0, 0, 1)] = 2;
        q[source_index(config, 1, 0, 2)] = -3;
        q[source_index(config, 2, 0, 1 * 4 + 2)] = 4;
        q[source_index(config, 3, 0, 1 * 32 + 2)] = 5;
        q[source_index(config, 4, 0, 3 * 32 + 4)] = -6;
        for (i, x) in [-4, -3, -2, 7].into_iter().enumerate() {
            q[source_index(config, 5, 0, i)] = x;
        }
        for (i, x) in [-1, 1, 2, -7].into_iter().enumerate() {
            q[source_index(config, 6, 0, i)] = x;
        }
        let admitted = NativePotentialQ4::new(config, &pack_coefficients(&q)?)?;
        let algebra = HistoricalH4Tables::from_bytes(include_bytes!(
            "../fixtures/historical-h4-tables-v1.bin"
        ))?;
        let basis = canonical_basis_q25();
        let i = basis
            .iter()
            .position(|&x| x == [0, 1 << 25, 0, 0])
            .ok_or("missing signed i")? as u8;
        let j = basis
            .iter()
            .position(|&x| x == [0, 0, 1 << 25, 0])
            .ok_or("missing signed j")? as u8;
        let cq = [AddressLane::new(1, 1, true)?];
        let ck = [AddressLane::new(i, 2, true)?];
        let rq = [AddressLane::new(1, 3, true)?];
        let rk = [AddressLane::new(j, 4, true)?];
        let absent = [AddressLane::new(1, 0, false)?];
        assert_eq!(admitted.score(0, &cq, &ck, &rq, &rk, &algebra)?, 1i64 << 23);
        // Reverse content: inverse(i)*1=-i; radius[2,1] is zero.
        assert_eq!(
            admitted.score(0, &ck, &cq, &rq, &rk, &algebra)?,
            -15i64 << 22
        );
        // No content unary/radius/cross; CP query-only=-2 and RP both=-7.
        assert_eq!(
            admitted.score(0, &cq, &absent, &rq, &rk, &algebra)?,
            -18i64 << 22
        );
        assert_eq!(
            admitted.score(0, &absent, &absent, &absent, &absent, &algebra)?,
            -5i64 << 22
        );
        assert!(admitted.score(1, &cq, &ck, &rq, &rk, &algebra).is_err());
        assert!(admitted.score(0, &[], &ck, &rq, &rk, &algebra).is_err());
        let half_root = basis
            .iter()
            .position(|&x| x == [1 << 24; 4])
            .ok_or("missing positive half root")? as u8;
        let half = [AddressLane::new(half_root, 2, true)?];
        let maximum = NativePotentialQ4::new(
            config,
            &pack_coefficients(&vec![7; config.coefficient_count()?])?,
        )?;
        assert_eq!(
            maximum.score(0, &cq, &half, &rq, &half, &algebra)?,
            MAX_ABS_LANE_SCORE_Q24
        );
        Ok(())
    }
}
