//! Strict packed-q4 source for the existing native geometric context engine.
//!
//! Coefficients q in [-7,7] have fixed quarter-nat units. Pinned source order is
//! transition(token,self,neighbor), root(token,self,neighbor), then
//! category(token,self,neighbor); L1 omits every neighbor family. Token shapes
//! are [V,H,L,C], basis shapes [H,L,C,4], C=120/120/33. Expanded tables preserve
//! [V,H,L,stride] and [H,L,128,stride], with strides128/128/64 and zero padding.
//!
//! Token Q24 is q<<22. Basis Q24 is round_ties_away(sum(q_i*B_i)/8), once per
//! factor. B is the existing canonical-F32 Q25 observation, distinct from the
//! exact signed H4 algebra and the legacy F64 observation. Three selected terms
//! are bounded by8.75nat (5.25 atL1); wide entries are regenerated precision,
//! not freely learned wide weights. No centering, learned scale or clipping.
//!
//! Admission allocates and regenerates tables; recurrence/readout delegate to
//! unchanged NativeContextTables/NativeContextState. This module supplies no
//! autodiff bridge and makes no new opcode, capability or whole-serving claim.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::geometric_context::{
    ContextError, ContextTableSlices, NativeContextTables, CATEGORY_COUNT, CATEGORY_STRIDE,
    MAX_HEADS, MAX_LANES_PER_HEAD, MAX_VOCAB, ROOT_STRIDE,
};
use crate::geometric_no_read::CANONICAL_BASIS_Q25;

pub const SCHEMA: &str = "uor-r4.geometric-context-q4/1";
pub const POLICY: &str = "signed-q4[-7,7];reserved-minus8;fixed-quarter-nat;canonical-F32-Q25-observation;token-shift22/basis-div8;per-factor-Q24-nearest-ties-away;transition-root-category/token-self-neighbor;L1-neighbor-omitted;root128-category64-zero-padding;no-centering/1";
pub const FRACTIONAL_BITS: u32 = 24;
pub const COEFFICIENT_SHIFT: u32 = 22;
pub const ROOT_COUNT: usize = 120;
pub const FAMILY_NAMES: [&str; 9] = [
    "token_transition",
    "self_transition",
    "neighbor_transition",
    "token_root",
    "self_root",
    "neighbor_root",
    "token_category",
    "self_category",
    "neighbor_category",
];
pub const MAX_ABS_SCORE_Q24: i64 = 35i64 << 22;
pub const MAX_ABS_SINGLE_LANE_SCORE_Q24: i64 = 21i64 << 22;
const CLASSES: [usize; 3] = [ROOT_COUNT, ROOT_COUNT, CATEGORY_COUNT];
const STRIDES: [usize; 3] = [ROOT_STRIDE, ROOT_STRIDE, CATEGORY_STRIDE];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextQ4Config {
    pub vocab_size: usize,
    pub heads: usize,
    pub lanes_per_head: usize,
}
impl ContextQ4Config {
    pub fn validate(self) -> ContextQ4Result<()> {
        if !(1..=MAX_VOCAB).contains(&self.vocab_size)
            || !(1..=MAX_HEADS).contains(&self.heads)
            || !(1..=MAX_LANES_PER_HEAD).contains(&self.lanes_per_head)
        {
            return Err(ContextQ4Error::Configuration);
        }
        Ok(())
    }
    /// Order defines the packed source; do not substitute map ordering.
    pub fn coefficient_shapes(self) -> ContextQ4Result<Vec<(String, Vec<usize>)>> {
        self.validate()?;
        let mut result = Vec::with_capacity(9);
        for (f, classes) in CLASSES.into_iter().enumerate() {
            result.push((
                FAMILY_NAMES[f * 3].to_owned(),
                vec![self.vocab_size, self.heads, self.lanes_per_head, classes],
            ));
            result.push((
                FAMILY_NAMES[f * 3 + 1].to_owned(),
                vec![self.heads, self.lanes_per_head, classes, 4],
            ));
            if self.lanes_per_head > 1 {
                result.push((
                    FAMILY_NAMES[f * 3 + 2].to_owned(),
                    vec![self.heads, self.lanes_per_head, classes, 4],
                ));
            }
        }
        Ok(result)
    }
    pub fn coefficient_count(self) -> ContextQ4Result<usize> {
        self.coefficient_shapes()?
            .iter()
            .try_fold(0usize, |total, (_, shape)| {
                let n = shape
                    .iter()
                    .try_fold(1usize, |p, &d| p.checked_mul(d))
                    .ok_or(ContextQ4Error::ArithmeticOverflow)?;
                total
                    .checked_add(n)
                    .ok_or(ContextQ4Error::ArithmeticOverflow)
            })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContextQ4Error {
    Configuration,
    PackedLength { expected: usize, actual: usize },
    InvalidCoefficient { index: usize, value: i8 },
    NonzeroNibblePadding,
    BasisBounds,
    ArithmeticOverflow,
    TableMismatch,
    Native(ContextError),
}
impl fmt::Display for ContextQ4Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "geometric context q4: {self:?}")
    }
}
impl std::error::Error for ContextQ4Error {}
pub type ContextQ4Result<T> = Result<T, ContextQ4Error>;

pub fn canonical_basis_q25() -> [[i32; 4]; ROOT_COUNT] {
    CANONICAL_BASIS_Q25
}
fn validate_basis() -> ContextQ4Result<()> {
    if CANONICAL_BASIS_Q25[0] != [-(1 << 25), 0, 0, 0]
        || CANONICAL_BASIS_Q25[1] != [1 << 25, 0, 0, 0]
    {
        return Err(ContextQ4Error::BasisBounds);
    }
    for root in CANONICAL_BASIS_Q25 {
        let mut l1 = 0i64;
        for x in root {
            let a = i64::from(x).abs();
            if a > 1 << 25 {
                return Err(ContextQ4Error::BasisBounds);
            }
            l1 = l1
                .checked_add(a)
                .ok_or(ContextQ4Error::ArithmeticOverflow)?;
        }
        if l1 > 2 << 25 {
            return Err(ContextQ4Error::BasisBounds);
        }
    }
    Ok(())
}

pub fn pack_coefficients(values: &[i8]) -> ContextQ4Result<Vec<u8>> {
    let mut packed = vec![0; values.len().div_ceil(2)];
    for (index, &value) in values.iter().enumerate() {
        if !(-7..=7).contains(&value) {
            return Err(ContextQ4Error::InvalidCoefficient { index, value });
        }
        packed[index >> 1] |= ((value as u8) & 15) << ((index & 1) << 2);
    }
    Ok(packed)
}
pub fn unpack_coefficients(count: usize, packed: &[u8]) -> ContextQ4Result<Vec<i8>> {
    let expected = count.div_ceil(2);
    if packed.len() != expected {
        return Err(ContextQ4Error::PackedLength {
            expected,
            actual: packed.len(),
        });
    }
    if count & 1 != 0 && packed.last().copied().unwrap_or(0) & 0xf0 != 0 {
        return Err(ContextQ4Error::NonzeroNibblePadding);
    }
    (0..count)
        .map(|index| {
            let nibble = (packed[index >> 1] >> ((index & 1) << 2)) & 15;
            let value = if nibble & 8 == 0 {
                nibble as i8
            } else {
                nibble as i8 - 16
            };
            if value == -8 {
                Err(ContextQ4Error::InvalidCoefficient { index, value })
            } else {
                Ok(value)
            }
        })
        .collect()
}

fn coefficient_product(q: i8, value: i64) -> ContextQ4Result<i64> {
    let mut magnitude = q.unsigned_abs();
    let mut term = value;
    let mut result = 0i64;
    while magnitude != 0 {
        if magnitude & 1 != 0 {
            result = result
                .checked_add(term)
                .ok_or(ContextQ4Error::ArithmeticOverflow)?;
        }
        magnitude >>= 1;
        if magnitude != 0 {
            term = term
                .checked_add(term)
                .ok_or(ContextQ4Error::ArithmeticOverflow)?;
        }
    }
    if q < 0 {
        result
            .checked_neg()
            .ok_or(ContextQ4Error::ArithmeticOverflow)
    } else {
        Ok(result)
    }
}
fn round_div8(sum: i64) -> ContextQ4Result<i32> {
    let magnitude = sum
        .unsigned_abs()
        .checked_add(4)
        .ok_or(ContextQ4Error::ArithmeticOverflow)?
        >> 3;
    let value = i64::try_from(magnitude).map_err(|_| ContextQ4Error::ArithmeticOverflow)?;
    let value = if sum < 0 {
        value
            .checked_neg()
            .ok_or(ContextQ4Error::ArithmeticOverflow)?
    } else {
        value
    };
    i32::try_from(value).map_err(|_| ContextQ4Error::ArithmeticOverflow)
}
fn basis_score(q: &[i8], root: &[i32; 4]) -> ContextQ4Result<i32> {
    let mut sum = 0i64;
    for (&q, &b) in q.iter().zip(root) {
        sum = sum
            .checked_add(coefficient_product(q, i64::from(b))?)
            .ok_or(ContextQ4Error::ArithmeticOverflow)?;
    }
    round_div8(sum)
}

/// Exact offline single-row compiler score at one canonical Q25 H4 root.
/// Shares the compiler arithmetic, including per-factor ties-away rounding.
pub fn basis_score_q24(q: [i8; 4], root: u8) -> ContextQ4Result<i32> {
    for (index, &value) in q.iter().enumerate() {
        if !(-7..=7).contains(&value) {
            return Err(ContextQ4Error::InvalidCoefficient { index, value });
        }
    }
    let root = CANONICAL_BASIS_Q25
        .get(usize::from(root))
        .ok_or(ContextQ4Error::Configuration)?;
    basis_score(&q, root)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextQ4Stats {
    pub learned_coefficients: usize,
    pub packed_bytes: usize,
    /// Payload only; excludes admission copies, geometry and metadata.
    pub expanded_table_entries: usize,
    pub expanded_table_bytes: usize,
}
pub struct NativeContextQ4 {
    config: ContextQ4Config,
    packed: Box<[u8]>,
    tables: [Vec<i32>; 9],
    native: NativeContextTables,
    coefficient_count: usize,
}
fn table_slices(t: &[Vec<i32>; 9], neighbor: bool) -> ContextTableSlices<'_> {
    ContextTableSlices {
        token_transition: &t[0],
        self_transition: &t[1],
        neighbor_transition: neighbor.then_some(t[2].as_slice()),
        token_root: &t[3],
        self_root: &t[4],
        neighbor_root: neighbor.then_some(t[5].as_slice()),
        token_category: &t[6],
        self_category: &t[7],
        neighbor_category: neighbor.then_some(t[8].as_slice()),
    }
}
impl NativeContextQ4 {
    pub fn new(config: ContextQ4Config, packed: &[u8]) -> ContextQ4Result<Self> {
        let coefficient_count = config.coefficient_count()?;
        let coefficients = unpack_coefficients(coefficient_count, packed)?;
        validate_basis()?;
        let lanes = config.heads * config.lanes_per_head;
        let mut tables: [Vec<i32>; 9] = std::array::from_fn(|_| Vec::new());
        let mut at = 0usize;
        for f in 0..3 {
            let classes = CLASSES[f];
            let stride = STRIDES[f];
            let count = config
                .vocab_size
                .checked_mul(lanes)
                .and_then(|n| n.checked_mul(stride))
                .ok_or(ContextQ4Error::ArithmeticOverflow)?;
            tables[f * 3] = vec![0; count];
            for row in tables[f * 3].chunks_exact_mut(stride) {
                for v in &mut row[..classes] {
                    *v = i32::from(coefficients[at]) << COEFFICIENT_SHIFT;
                    at += 1;
                }
            }
            for factor in 1..=2 {
                if factor == 2 && config.lanes_per_head == 1 {
                    continue;
                }
                let count = lanes
                    .checked_mul(ROOT_STRIDE)
                    .and_then(|n| n.checked_mul(stride))
                    .ok_or(ContextQ4Error::ArithmeticOverflow)?;
                let table = &mut tables[f * 3 + factor];
                *table = vec![0; count];
                for lane in 0..lanes {
                    for (state, root) in CANONICAL_BASIS_Q25.iter().enumerate() {
                        for choice in 0..classes {
                            let start = at + (lane * classes + choice) * 4;
                            table[(lane * ROOT_STRIDE + state) * stride + choice] =
                                basis_score(&coefficients[start..start + 4], root)?;
                        }
                    }
                }
                at = at
                    .checked_add(lanes * classes * 4)
                    .ok_or(ContextQ4Error::ArithmeticOverflow)?;
            }
        }
        if at != coefficient_count {
            return Err(ContextQ4Error::ArithmeticOverflow);
        }
        let native = NativeContextTables::new(
            config.vocab_size,
            config.heads,
            config.lanes_per_head,
            table_slices(&tables, config.lanes_per_head > 1),
        )
        .map_err(ContextQ4Error::Native)?;
        Ok(Self {
            config,
            packed: packed.into(),
            tables,
            native,
            coefficient_count,
        })
    }
    /// Reject any wide table not exactly regenerated from the authoritative q4
    /// source, including padded bytes or a different omitted-neighbor layout.
    pub fn from_parts(
        config: ContextQ4Config,
        packed: &[u8],
        expanded: &[i32],
    ) -> ContextQ4Result<Self> {
        let admitted = Self::new(config, packed)?;
        if admitted.tables.iter().map(Vec::len).sum::<usize>() != expanded.len()
            || !admitted.tables.iter().flatten().eq(expanded.iter())
        {
            return Err(ContextQ4Error::TableMismatch);
        }
        Ok(admitted)
    }
    pub fn config(&self) -> ContextQ4Config {
        self.config
    }
    pub fn packed_coefficients(&self) -> &[u8] {
        &self.packed
    }
    /// Always nine entries; omitted L1 neighbor lengths are zero.
    pub fn expanded_lengths(&self) -> [usize; 9] {
        std::array::from_fn(|i| self.tables[i].len())
    }
    pub fn expanded_q24(&self) -> Vec<i32> {
        self.tables.iter().flatten().copied().collect()
    }
    pub fn table_bytes(&self) -> Vec<u8> {
        self.tables
            .iter()
            .flatten()
            .flat_map(|x| x.to_le_bytes())
            .collect()
    }
    pub fn table_slices(&self) -> ContextTableSlices<'_> {
        table_slices(&self.tables, self.config.lanes_per_head > 1)
    }
    pub fn native(&self) -> &NativeContextTables {
        &self.native
    }
    pub fn into_native(self) -> ContextQ4Result<NativeContextTables> {
        Ok(self.native)
    }
    pub fn stats(&self) -> ContextQ4Stats {
        let entries = self.tables.iter().map(Vec::len).sum::<usize>();
        ContextQ4Stats {
            learned_coefficients: self.coefficient_count,
            packed_bytes: self.packed.len(),
            expanded_table_entries: entries,
            expanded_table_bytes: entries * 4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_context::NativeContextState;
    use crate::h4_tables::{H4Code, HistoricalH4Tables};
    type TestResult = Result<(), Box<dyn std::error::Error>>;
    fn config(vocab_size: usize, heads: usize, lanes_per_head: usize) -> ContextQ4Config {
        ContextQ4Config {
            vocab_size,
            heads,
            lanes_per_head,
        }
    }
    fn oracle(q: &[i8], root: &[i32; 4]) -> i32 {
        let n: i128 = q
            .iter()
            .zip(root)
            .map(|(&q, &b)| i128::from(q) * i128::from(b))
            .sum();
        let a = n.unsigned_abs();
        let rounded = a / 8 + u128::from(a % 8 >= 4);
        if n < 0 {
            -(rounded as i32)
        } else {
            rounded as i32
        }
    }
    #[test]
    fn context_q4_public_basis_score_matches_compiler_and_rejects_invalid() -> TestResult {
        for (code, root) in canonical_basis_q25().iter().enumerate() {
            for q in [[0; 4], [7, -7, 3, -2], [-1, 1, -1, 1]] {
                assert_eq!(basis_score_q24(q, code as u8)?, basis_score(&q, root)?);
            }
        }
        assert!(basis_score_q24([-8, 0, 0, 0], 0).is_err());
        assert!(basis_score_q24([8, 0, 0, 0], 0).is_err());
        assert!(basis_score_q24([0; 4], 120).is_err());
        Ok(())
    }

    #[test]
    fn context_q4_packing_dimensions_and_single_lane_padding() -> TestResult {
        let values = [-7, -1, 0, 1, 7];
        let packed = pack_coefficients(&values)?;
        assert_eq!(packed, [0xf9, 0x10, 7]);
        assert_eq!(unpack_coefficients(5, &packed)?, values);
        assert!(pack_coefficients(&[-8]).is_err());
        assert!(pack_coefficients(&[8]).is_err());
        assert!(matches!(
            unpack_coefficients(2, &[0x80]),
            Err(ContextQ4Error::InvalidCoefficient {
                index: 1,
                value: -8
            })
        ));
        assert_eq!(
            unpack_coefficients(1, &[0x10]),
            Err(ContextQ4Error::NonzeroNibblePadding)
        );
        assert!(unpack_coefficients(0, &[])?.is_empty());
        assert!(unpack_coefficients(2, &[]).is_err());
        assert!(unpack_coefficients(2, &[0, 0]).is_err());
        for c in [
            config(0, 1, 1),
            config(MAX_VOCAB + 1, 1, 1),
            config(1, 0, 1),
            config(1, MAX_HEADS + 1, 1),
            config(1, 1, 0),
            config(1, 1, MAX_LANES_PER_HEAD + 1),
        ] {
            assert!(matches!(
                NativeContextQ4::new(c, &[]),
                Err(ContextQ4Error::Configuration)
            ));
        }
        assert_eq!(config(40, 2, 4).coefficient_count()?, 104832);
        assert_eq!(config(4096, 2, 4).coefficient_count()?, 8963136);
        let c = config(1, 1, 1);
        assert_eq!(c.coefficient_count()?, 1365);
        let shapes = c.coefficient_shapes()?;
        assert_eq!(shapes.len(), 6);
        assert!(shapes.iter().all(|(name, _)| !name.starts_with("neighbor")));
        let mut bytes = pack_coefficients(&vec![0; c.coefficient_count()?])?;
        let end = bytes.len() - 1;
        bytes[end] |= 0x10;
        assert!(matches!(
            NativeContextQ4::new(c, &bytes),
            Err(ContextQ4Error::NonzeroNibblePadding)
        ));
        Ok(())
    }
    #[test]
    fn context_q4_basis_signed_ties_bounds_and_i128_reference() -> TestResult {
        validate_basis()?;
        let patterns = [[7, 7, 7, 7], [-7, -7, -7, -7], [1, -2, 3, -4], [2, 0, 0, 0]];
        let mut positive_half = false;
        let mut negative_half = false;
        for root in canonical_basis_q25() {
            for q in patterns {
                let n: i128 = q
                    .iter()
                    .zip(root)
                    .map(|(&a, b)| i128::from(a) * i128::from(b))
                    .sum();
                let value = basis_score(&q, &root)?;
                assert_eq!(value, oracle(&q, &root));
                assert!(i64::from(value).abs() <= 7i64 << 23);
                assert_eq!(basis_score(&q, &root.map(|x| -x))?, -value);
                if n.unsigned_abs() % 8 == 4 {
                    positive_half |= n > 0;
                    negative_half |= n < 0;
                }
            }
        }
        assert!(positive_half && negative_half);
        assert_eq!(basis_score(&[7; 4], &[1 << 24; 4])?, 7 << 23);
        assert_eq!(
            i64::from(7 << 22) + 2 * i64::from(7 << 23),
            MAX_ABS_SCORE_Q24
        );
        assert_eq!(
            i64::from(7 << 22) + i64::from(7 << 23),
            MAX_ABS_SINGLE_LANE_SCORE_Q24
        );
        assert!(coefficient_product(7, i64::MAX).is_err());
        assert!(round_div8(i64::MAX).is_err());
        Ok(())
    }
    #[test]
    fn context_q4_all_family_layout_padding_and_regeneration() -> TestResult {
        for lanes in [1, 2] {
            let c = config(2, 2, lanes);
            let q = (0..c.coefficient_count()?)
                .map(|i| (i % 15) as i8 - 7)
                .collect::<Vec<_>>();
            let packed = pack_coefficients(&q)?;
            let admitted = NativeContextQ4::new(c, &packed)?;
            let lengths = admitted.expanded_lengths();
            let mut source_at = 0;
            let n = c.heads * c.lanes_per_head;
            for f in 0..3 {
                let classes = CLASSES[f];
                let stride = STRIDES[f];
                let token = &admitted.tables[f * 3];
                for row in token.chunks_exact(stride) {
                    for &entry in &row[..classes] {
                        assert_eq!(entry, i32::from(q[source_at]) << 22);
                        source_at += 1;
                    }
                    assert!(row[classes..].iter().all(|&v| v == 0));
                }
                for factor in 1..=2 {
                    if lanes == 1 && factor == 2 {
                        assert_eq!(lengths[f * 3 + factor], 0);
                        continue;
                    }
                    let table = &admitted.tables[f * 3 + factor];
                    for lane in 0..n {
                        for state in 0..ROOT_STRIDE {
                            for choice in 0..stride {
                                let value = table[(lane * ROOT_STRIDE + state) * stride + choice];
                                if state >= ROOT_COUNT || choice >= classes {
                                    assert_eq!(value, 0);
                                } else {
                                    let start = source_at + (lane * classes + choice) * 4;
                                    assert_eq!(
                                        value,
                                        oracle(&q[start..start + 4], &canonical_basis_q25()[state])
                                    );
                                }
                            }
                        }
                    }
                    source_at += n * classes * 4;
                }
            }
            assert_eq!(source_at, q.len());
            let expanded = admitted.expanded_q24();
            assert_eq!(admitted.stats().expanded_table_bytes, expanded.len() * 4);
            assert_eq!(admitted.table_bytes().len(), expanded.len() * 4);
            let restored = NativeContextQ4::from_parts(c, &packed, &expanded)?;
            assert_eq!(restored.packed_coefficients(), packed);
            assert_eq!(restored.native().stats().stored_entries, expanded.len());
            assert_eq!(restored.table_slices().neighbor_root.is_none(), lanes == 1);
            let mut changed = expanded.clone();
            changed[120] = 1;
            assert!(matches!(
                NativeContextQ4::from_parts(c, &packed, &changed),
                Err(ContextQ4Error::TableMismatch)
            ));
            let mut changed = expanded.clone();
            changed[0] ^= 1;
            assert!(matches!(
                NativeContextQ4::from_parts(c, &packed, &changed),
                Err(ContextQ4Error::TableMismatch)
            ));
            assert!(
                NativeContextQ4::from_parts(c, &packed, &expanded[..expanded.len() - 1]).is_err()
            );
            let native = restored.into_native()?;
            assert_eq!(
                (native.vocab_size(), native.heads(), native.lanes_per_head()),
                (2, 2, lanes)
            );
        }
        Ok(())
    }
    #[test]
    fn context_q4_existing_state_keeps_signed_order_and_absent_history() -> TestResult {
        let c = config(2, 1, 1);
        let shapes = c.coefficient_shapes()?;
        let mut offsets = std::collections::BTreeMap::new();
        let mut count = 0;
        for (name, shape) in shapes {
            offsets.insert(name, count);
            count += shape.iter().product::<usize>();
        }
        let basis = canonical_basis_q25();
        let i = basis
            .iter()
            .position(|&r| r == [0, 1 << 25, 0, 0])
            .ok_or("missing i")?;
        let j = basis
            .iter()
            .position(|&r| r == [0, 0, 1 << 25, 0])
            .ok_or("missing j")?;
        let mut q = vec![0; count];
        q[offsets["token_transition"] + i] = 7;
        q[offsets["token_transition"] + ROOT_COUNT + j] = 7;
        // Root readout observes the NEW i coordinate. Category0 ties17 and
        // wins earliest on +i; it does not erase that latent state.
        q[offsets["self_root"] + j * 4 + 1] = 7;
        q[offsets["token_category"] + 17] = 7;
        q[offsets["token_category"] + CATEGORY_COUNT + 17] = 7;
        q[offsets["self_category"] + 1] = 7;
        let admitted = NativeContextQ4::new(c, &pack_coefficients(&q)?)?;
        let algebra = HistoricalH4Tables::from_bytes(include_bytes!(
            "../fixtures/historical-h4-tables-v1.bin"
        ))?;
        let mut state = NativeContextState::new(1, 1)?;
        let first = state.step(0, admitted.native(), &algebra)?;
        assert_eq!(first.actions[0].index() as usize, i);
        assert_eq!(first.states[0].index() as usize, i);
        assert_eq!(first.readout_roots[0].index() as usize, j);
        assert_eq!(first.categories[0], 0);
        assert!(!first.output[0].present());
        assert_eq!(first.output[0].root(), 1);
        let second = state.step(0, admitted.native(), &algebra)?;
        assert_eq!(second.states[0].index(), 0);
        assert_eq!(second.categories[0], 17);
        assert!(second.output[0].present());
        state.reset();
        state.step(0, admitted.native(), &algebra)?;
        let ij = state.step(1, admitted.native(), &algebra)?.states[0];
        state.reset();
        state.step(1, admitted.native(), &algebra)?;
        let ji = state.step(0, admitted.native(), &algebra)?.states[0];
        assert_eq!(
            ij,
            algebra.compose(H4Code::try_from(i as u8)?, H4Code::try_from(j as u8)?)
        );
        assert_ne!(ij, ji);
        let before = state.states().to_vec();
        assert!(state.step(2, admitted.native(), &algebra).is_err());
        assert_eq!(state.states(), before);
        Ok(())
    }
}
