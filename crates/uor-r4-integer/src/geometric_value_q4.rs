//! Strict four-bit source admission for the existing geometric K2 value producer.
//!
//! Every free coefficient is a signed nibble q in [-7,7] at the single fixed
//! quarter-logit scale. Wider Q24 table entries are derived values, independently
//! regenerated from that packed source and the existing canonical-F32 Q25 basis.
//! The exact signed H4 group and this rounded observation basis remain distinct.
//! This module allocates only during admission/serialization. Numerical value
//! production delegates to the existing `NativeValueProducer`; its scoped
//! opcode evidence is not a claim about this compiler or the complete model.
//!
//! Source order is root then category, and within each family token, own,
//! neighbor, span, span-valid. A single latent lane omits neighbor coefficients
//! and tables. Token shapes are [V,H,4,2,C], three state bases [H,4,2,C,4],
//! validity [H,4,2,C], with C=120 or 32. Expanded state axes use stride128;
//! root choices use stride128 and category choices stride32. All padding is zero.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::geometric_no_read::CANONICAL_BASIS_Q25;
use crate::geometric_value_producer::{
    NativeValueProducer, ValueProducerError, ValueProducerTableSlices, ATOMS, CATEGORY_COUNT,
    CATEGORY_STRIDE, MAX_HEADS, MAX_LATENT_LANES_PER_HEAD, MAX_VOCAB, ROOT_STRIDE,
    VALUE_LANES_PER_HEAD,
};

pub const SCHEMA: &str = "uor-r4.geometric-value-q4/1";
pub const POLICY: &str = "signed-q4[-7,7];reserved-minus8;fixed-quarter-logit;canonical-F32-Q25-basis;per-root-Q24-nearest-ties-away;root-then-category/token-own-neighbor-span-valid;neighbor-omitted-if-L1/1";
pub const FRACTIONAL_BITS: u32 = 24;
pub const COEFFICIENT_SHIFT: u32 = 22;
pub const ROOT_COUNT: usize = 120;
pub const FAMILY_NAMES: [&str; 10] = [
    "token_root",
    "own_root",
    "neighbor_root",
    "span_root",
    "span_valid_root",
    "token_category",
    "own_category",
    "neighbor_category",
    "span_category",
    "span_valid_category",
];
/// Canonical fixed-basis l1 norm <=2: token+three bases+validity <=14.
pub const MAX_ABS_SCORE_Q24: i64 = 14i64 << FRACTIONAL_BITS;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValueQ4Config {
    pub vocab_size: usize,
    pub heads: usize,
    pub latent_lanes_per_head: usize,
}

impl ValueQ4Config {
    pub fn validate(self) -> ValueQ4Result<()> {
        if !(1..=MAX_VOCAB).contains(&self.vocab_size)
            || !(1..=MAX_HEADS).contains(&self.heads)
            || !(1..=MAX_LATENT_LANES_PER_HEAD).contains(&self.latent_lanes_per_head)
        {
            return Err(ValueQ4Error::Configuration);
        }
        Ok(())
    }

    /// Pinned packing order; never substitute alphabetical parameter ordering.
    pub fn coefficient_shapes(self) -> ValueQ4Result<Vec<(String, Vec<usize>)>> {
        self.validate()?;
        let mut shapes = Vec::with_capacity(10);
        for (f, classes) in [ROOT_COUNT, CATEGORY_COUNT].into_iter().enumerate() {
            let names = &FAMILY_NAMES[f * 5..f * 5 + 5];
            shapes.push((
                names[0].to_owned(),
                vec![
                    self.vocab_size,
                    self.heads,
                    VALUE_LANES_PER_HEAD,
                    ATOMS,
                    classes,
                ],
            ));
            for (factor, &name) in names.iter().enumerate().take(4).skip(1) {
                if factor == 2 && self.latent_lanes_per_head == 1 {
                    continue;
                }
                shapes.push((
                    name.to_owned(),
                    vec![self.heads, VALUE_LANES_PER_HEAD, ATOMS, classes, 4],
                ));
            }
            shapes.push((
                names[4].to_owned(),
                vec![self.heads, VALUE_LANES_PER_HEAD, ATOMS, classes],
            ));
        }
        Ok(shapes)
    }

    pub fn coefficient_count(self) -> ValueQ4Result<usize> {
        // Validated limits fit usize on both 32-bit and 64-bit targets.
        Ok(self
            .coefficient_shapes()?
            .iter()
            .map(|(_, shape)| shape.iter().product::<usize>())
            .sum())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValueQ4Error {
    Configuration,
    PackedLength { expected: usize, actual: usize },
    InvalidCoefficient { index: usize, value: i8 },
    NonzeroNibblePadding,
    TableMismatch,
    Native(ValueProducerError),
}
impl fmt::Display for ValueQ4Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "geometric value q4: {self:?}")
    }
}
impl std::error::Error for ValueQ4Error {}
pub type ValueQ4Result<T> = Result<T, ValueQ4Error>;

/// Existing pinned observation basis. Its LE-i32 bytes can be bound by an
/// enclosing artifact without constructing a second geometry policy.
pub fn canonical_basis_q25() -> [[i32; 4]; ROOT_COUNT] {
    CANONICAL_BASIS_Q25
}

/// Signed two's-complement nibble, low nibble first; unused high nibble zero.
pub fn pack_coefficients(values: &[i8]) -> ValueQ4Result<Vec<u8>> {
    let mut packed = vec![0; values.len().div_ceil(2)];
    for (index, &value) in values.iter().enumerate() {
        if !(-7..=7).contains(&value) {
            return Err(ValueQ4Error::InvalidCoefficient { index, value });
        }
        packed[index >> 1] |= ((value as u8) & 15) << ((index & 1) << 2);
    }
    Ok(packed)
}

pub fn unpack_coefficients(expected_count: usize, packed: &[u8]) -> ValueQ4Result<Vec<i8>> {
    let expected = expected_count.div_ceil(2);
    if packed.len() != expected {
        return Err(ValueQ4Error::PackedLength {
            expected,
            actual: packed.len(),
        });
    }
    if expected_count & 1 != 0 && packed.last().copied().unwrap_or(0) & 0xf0 != 0 {
        return Err(ValueQ4Error::NonzeroNibblePadding);
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
                Err(ValueQ4Error::InvalidCoefficient { index, value })
            } else {
                Ok(value)
            }
        })
        .collect()
}

// q4 * Q25 /4 expressed as Q24: sum(q*B)/8. All operands are bounded;
// sum of absolute products <=14*2^25, and output magnitude <=3.5*2^24.
// This construction uses integer shifts/adds and has no floating-point policy.
fn root_score_q24(coefficients: &[i8], root: &[i32; 4]) -> i32 {
    let mut sum = 0i64;
    for (&coefficient, &coordinate) in coefficients.iter().zip(root) {
        let mut magnitude = coefficient.unsigned_abs();
        let mut shifted = i64::from(coordinate);
        let mut product = 0i64;
        while magnitude != 0 {
            if magnitude & 1 != 0 {
                product += shifted;
            }
            magnitude >>= 1;
            shifted <<= 1;
        }
        sum += if coefficient < 0 { -product } else { product };
    }
    if sum < 0 {
        -(((-sum + 4) >> 3) as i32)
    } else {
        ((sum + 4) >> 3) as i32
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValueQ4Stats {
    pub learned_coefficients: usize,
    pub packed_bytes: usize,
    /// Derived i32 table payload only, excluding metadata/native copies/codec.
    pub expanded_table_bytes: usize,
    pub expanded_table_entries: usize,
}

/// Admission object retaining the authoritative packed source and regenerated
/// tables. Numerical execution uses `into_native`, not a new value engine.
pub struct NativeValueQ4 {
    config: ValueQ4Config,
    packed: Box<[u8]>,
    tables: [Vec<i32>; 10],
    coefficient_count: usize,
}
impl NativeValueQ4 {
    pub fn new(config: ValueQ4Config, packed: &[u8]) -> ValueQ4Result<Self> {
        let coefficient_count = config.coefficient_count()?;
        let coefficients = unpack_coefficients(coefficient_count, packed)?;
        let slots = config.heads * VALUE_LANES_PER_HEAD * ATOMS;
        let mut tables = std::array::from_fn(|_| Vec::new());
        let mut at = 0;
        for (family, (classes, stride)) in
            [(ROOT_COUNT, ROOT_STRIDE), (CATEGORY_COUNT, CATEGORY_STRIDE)]
                .into_iter()
                .enumerate()
        {
            let base = family * 5;
            let token = &mut tables[base];
            *token = vec![0; config.vocab_size * slots * stride];
            for row in token.chunks_exact_mut(stride) {
                for value in &mut row[..classes] {
                    *value = i32::from(coefficients[at]) << COEFFICIENT_SHIFT;
                    at += 1;
                }
            }
            for factor in 1..=3 {
                if factor == 2 && config.latent_lanes_per_head == 1 {
                    continue;
                }
                let table = &mut tables[base + factor];
                *table = vec![0; slots * ROOT_STRIDE * stride];
                for slot in 0..slots {
                    for (state, root) in CANONICAL_BASIS_Q25.iter().enumerate() {
                        for choice in 0..classes {
                            let start = at + (slot * classes + choice) * 4;
                            table[(slot * ROOT_STRIDE + state) * stride + choice] =
                                root_score_q24(&coefficients[start..start + 4], root);
                        }
                    }
                }
                at += slots * classes * 4;
            }
            let valid = &mut tables[base + 4];
            *valid = vec![0; slots * stride];
            for row in valid.chunks_exact_mut(stride) {
                for value in &mut row[..classes] {
                    *value = i32::from(coefficients[at]) << COEFFICIENT_SHIFT;
                    at += 1;
                }
            }
        }
        Ok(Self {
            config,
            packed: packed.into(),
            tables,
            coefficient_count,
        })
    }

    /// Reject arbitrary or resealed wide learned tables: every entry must equal
    /// the independent regeneration of the supplied packed source.
    pub fn from_parts(
        config: ValueQ4Config,
        packed: &[u8],
        expanded: &[i32],
    ) -> ValueQ4Result<Self> {
        let native = Self::new(config, packed)?;
        if native.tables.iter().map(Vec::len).sum::<usize>() != expanded.len()
            || !native.tables.iter().flatten().eq(expanded.iter())
        {
            return Err(ValueQ4Error::TableMismatch);
        }
        Ok(native)
    }

    pub fn config(&self) -> ValueQ4Config {
        self.config
    }
    pub fn packed_coefficients(&self) -> &[u8] {
        &self.packed
    }
    pub fn expanded_lengths(&self) -> [usize; 10] {
        std::array::from_fn(|i| self.tables[i].len())
    }
    pub fn expanded_q24(&self) -> Vec<i32> {
        self.tables.iter().flatten().copied().collect()
    }
    pub fn table_bytes(&self) -> Vec<u8> {
        self.tables
            .iter()
            .flatten()
            .flat_map(|value| value.to_le_bytes())
            .collect()
    }
    pub fn stats(&self) -> ValueQ4Stats {
        let entries = self.tables.iter().map(Vec::len).sum::<usize>();
        ValueQ4Stats {
            learned_coefficients: self.coefficient_count,
            packed_bytes: self.packed.len(),
            expanded_table_entries: entries,
            expanded_table_bytes: entries * 4,
        }
    }
    pub fn table_slices(&self) -> ValueProducerTableSlices<'_> {
        ValueProducerTableSlices {
            token_root: &self.tables[0],
            own_root: &self.tables[1],
            neighbor_root: (self.config.latent_lanes_per_head > 1)
                .then_some(self.tables[2].as_slice()),
            span_root: &self.tables[3],
            span_valid_root: &self.tables[4],
            token_category: &self.tables[5],
            own_category: &self.tables[6],
            neighbor_category: (self.config.latent_lanes_per_head > 1)
                .then_some(self.tables[7].as_slice()),
            span_category: &self.tables[8],
            span_valid_category: &self.tables[9],
        }
    }
    pub fn into_native(self) -> ValueQ4Result<NativeValueProducer> {
        NativeValueProducer::new(
            self.config.vocab_size,
            self.config.heads,
            self.config.latent_lanes_per_head,
            self.table_slices(),
        )
        .map_err(ValueQ4Error::Native)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small_config(lanes: usize) -> ValueQ4Config {
        ValueQ4Config {
            vocab_size: 2,
            heads: 1,
            latent_lanes_per_head: lanes,
        }
    }

    #[test]
    fn value_q4_packing_rejects_reserved_and_noncanonical_bytes() {
        let q = [-7, -1, 0, 1, 7];
        let packed = pack_coefficients(&q).unwrap();
        assert_eq!(packed, vec![0xf9, 0x10, 0x07]);
        assert_eq!(unpack_coefficients(q.len(), &packed).unwrap(), q);
        assert!(matches!(
            pack_coefficients(&[-8]),
            Err(ValueQ4Error::InvalidCoefficient {
                index: 0,
                value: -8
            })
        ));
        assert!(pack_coefficients(&[8]).is_err());
        assert!(matches!(
            unpack_coefficients(2, &[0x80]),
            Err(ValueQ4Error::InvalidCoefficient {
                index: 1,
                value: -8
            })
        ));
        assert_eq!(
            unpack_coefficients(1, &[0x10]),
            Err(ValueQ4Error::NonzeroNibblePadding)
        );
        assert!(unpack_coefficients(2, &[]).is_err());
        assert!(unpack_coefficients(2, &[0, 0]).is_err());
        assert!(unpack_coefficients(0, &[]).unwrap().is_empty());
        for config in [
            ValueQ4Config {
                vocab_size: 0,
                ..small_config(1)
            },
            ValueQ4Config {
                vocab_size: MAX_VOCAB + 1,
                ..small_config(1)
            },
            ValueQ4Config {
                heads: 0,
                ..small_config(1)
            },
            ValueQ4Config {
                heads: MAX_HEADS + 1,
                ..small_config(1)
            },
            small_config(0),
            small_config(MAX_LATENT_LANES_PER_HEAD + 1),
        ] {
            assert!(matches!(
                NativeValueQ4::new(config, &[]),
                Err(ValueQ4Error::Configuration)
            ));
        }
    }

    #[test]
    fn value_q4_integer_basis_expansion_has_signed_half_ties_and_bounds() {
        // Independent test-only multiply/divide oracle. These products are
        // small dyadic integers; no float reference masks a rounding error.
        let patterns = [[7, 7, 7, 7], [-7, -7, -7, -7], [1, -2, 3, -4], [2, 0, 0, 0]];
        let mut saw_positive_half = false;
        let mut saw_negative_half = false;
        for root in canonical_basis_q25() {
            assert!(root.iter().map(|x| i64::from(*x).abs()).sum::<i64>() <= 2i64 << 25);
            for q in patterns {
                let numerator = q
                    .iter()
                    .zip(root)
                    .map(|(&a, b)| i64::from(a) * i64::from(b))
                    .sum::<i64>();
                let abs = numerator.unsigned_abs();
                let rounded = abs / 8 + u64::from(abs % 8 >= 4);
                let expected = if numerator < 0 {
                    -(rounded as i32)
                } else {
                    rounded as i32
                };
                assert_eq!(root_score_q24(&q, &root), expected);
                assert!(i64::from(expected).abs() <= 7i64 << 23);
                if abs % 8 == 4 {
                    saw_positive_half |= numerator > 0;
                    saw_negative_half |= numerator < 0;
                }
                let opposite = root.map(|x| -x);
                assert_eq!(root_score_q24(&q, &opposite), -expected);
            }
        }
        assert!(saw_positive_half && saw_negative_half);
    }

    #[test]
    fn value_q4_layout_regeneration_padding_and_existing_native_admission() {
        for lanes in [1, 4] {
            let config = small_config(lanes);
            let shapes = config.coefficient_shapes().unwrap();
            let count = config.coefficient_count().unwrap();
            let q = (0..count).map(|i| (i % 15) as i8 - 7).collect::<Vec<_>>();
            let packed = pack_coefficients(&q).unwrap();
            let admitted = NativeValueQ4::new(config, &packed).unwrap();
            let lengths = admitted.expanded_lengths();
            assert_eq!(lengths[2] == 0, lanes == 1);
            assert_eq!(lengths[7] == 0, lanes == 1);
            let tables = admitted.table_slices();
            let all = [
                tables.token_root,
                tables.own_root,
                tables.neighbor_root.unwrap_or(&[]),
                tables.span_root,
                tables.span_valid_root,
                tables.token_category,
                tables.own_category,
                tables.neighbor_category.unwrap_or(&[]),
                tables.span_category,
                tables.span_valid_category,
            ];
            let mut source_offset = 0;
            for (name, shape) in shapes {
                let family_index = FAMILY_NAMES.iter().position(|x| *x == name).unwrap();
                let classes = if family_index < 5 { 120 } else { 32 };
                let stride = if family_index < 5 { 128 } else { 32 };
                let count = shape.iter().product::<usize>();
                let source = &q[source_offset..source_offset + count];
                let state_factor = matches!(family_index % 5, 1..=3);
                for (row, values) in all[family_index].chunks_exact(stride).enumerate() {
                    for (choice, &value) in values.iter().enumerate() {
                        let expected = if choice >= classes || (state_factor && row % 128 >= 120) {
                            0
                        } else if state_factor {
                            let at = ((row / 128) * classes + choice) * 4;
                            root_score_q24(&source[at..at + 4], &CANONICAL_BASIS_Q25[row % 128])
                        } else {
                            i32::from(source[row * classes + choice]) << 22
                        };
                        assert_eq!(value, expected, "{name} row{row} choice{choice}");
                    }
                }
                source_offset += count;
            }
            assert_eq!(source_offset, count);
            let expanded = admitted.expanded_q24();
            let reload = NativeValueQ4::from_parts(config, &packed, &expanded).unwrap();
            assert_eq!(reload.table_bytes(), admitted.table_bytes());
            assert_eq!(reload.stats().expanded_table_bytes, expanded.len() * 4);
            reload.into_native().unwrap();
            let mut tampered = expanded;
            tampered[0] ^= 1;
            assert!(matches!(
                NativeValueQ4::from_parts(config, &packed, &tampered),
                Err(ValueQ4Error::TableMismatch)
            ));
            let mut changed = q;
            changed[0] = 7;
            let changed = pack_coefficients(&changed).unwrap();
            assert!(matches!(
                NativeValueQ4::from_parts(config, &changed, &admitted.expanded_q24()),
                Err(ValueQ4Error::TableMismatch)
            ));
        }
    }

    #[test]
    fn value_q4_current_capacity_and_extreme_scores_are_explicit() {
        let config = ValueQ4Config {
            vocab_size: 40,
            heads: 2,
            latent_lanes_per_head: 4,
        };
        assert_eq!(config.coefficient_count().unwrap(), 128_896);
        let maximum = ValueQ4Config {
            vocab_size: MAX_VOCAB,
            heads: MAX_HEADS,
            latent_lanes_per_head: MAX_LATENT_LANES_PER_HEAD,
        };
        assert_eq!(maximum.coefficient_count().unwrap(), 9_993_088);
        let small = small_config(4);
        let admitted = NativeValueQ4::new(
            small,
            &pack_coefficients(&vec![7; small.coefficient_count().unwrap()]).unwrap(),
        )
        .unwrap();
        let t = admitted.table_slices();
        let mut maximum = 0i64;
        for state in 0..120 {
            let v = i64::from(t.token_root[0])
                + i64::from(t.own_root[state * 128])
                + i64::from(t.neighbor_root.unwrap()[state * 128])
                + i64::from(t.span_root[state * 128])
                + i64::from(t.span_valid_root[0]);
            maximum = maximum.max(v);
            assert!(v.abs() <= MAX_ABS_SCORE_Q24);
        }
        assert_eq!(maximum, MAX_ABS_SCORE_Q24);
        assert_eq!(
            admitted.stats().packed_bytes,
            small.coefficient_count().unwrap() / 2
        );
    }
}
