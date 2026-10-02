//! Four-bit learned geometric NoRead scalar. Wider Q24 entries are derived
//! outputs, regenerated from packed q in [-7,7] at the fixed quarter-nat scale.
//! The Q25 observation basis reproduces canonical F32 root coordinates; exact
//! H4 identities remain separate. Admission allocates; score only selects rows
//! and adds integers. Release-opcode/full-model qualification is separate.
use crate::geometric_potential::AddressLane;
use crate::h4_classifier::H4_ROOT_COEFFICIENTS;
use crate::h4_tables::H4Code;
use serde::{Deserialize, Serialize};
use std::fmt;

pub const FRACTIONAL_BITS: u32 = 24;
pub const COEFFICIENT_SHIFT: u32 = 22;
pub const ROOT_STRIDE: usize = 128;
pub const CATEGORY_COUNT: usize = 33;
pub const CATEGORY_STRIDE: usize = 64;
pub const POLICY: &str = "signed-q4[-7,7];reserved-minus8;fixed-quarter-nat;canonical-F32-Q25-basis;per-root-Q24-nearest-ties-away;head-major-bias-token-latent-category-held-valid/1";
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NoReadConfig {
    pub vocabulary: usize,
    pub heads: usize,
    pub latent_lanes_per_head: usize,
}
impl NoReadConfig {
    pub fn validate(self) -> Result<(), NoReadError> {
        if !(1..=4096).contains(&self.vocabulary)
            || !(1..=2).contains(&self.heads)
            || !(1..=4).contains(&self.latent_lanes_per_head)
        {
            return Err(NoReadError::Configuration);
        }
        Ok(())
    }
    pub fn lanes(self) -> usize {
        self.heads * self.latent_lanes_per_head
    }
    pub fn coefficients_per_head(self) -> usize {
        1 + self.vocabulary + self.lanes() * 42
    }
    pub fn coefficient_count(self) -> usize {
        self.heads * self.coefficients_per_head()
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NoReadError {
    Configuration,
    CoefficientLength { expected: usize, actual: usize },
    InvalidCoefficient { index: usize, value: i8 },
    NonzeroNibblePadding,
    TableMismatch,
    Token { token: usize, vocabulary: usize },
    Shape { expected: usize, actual: usize },
}
impl fmt::Display for NoReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "geometric NoRead: {self:?}")
    }
}
impl std::error::Error for NoReadError {}

const fn make_basis() -> [[i32; 4]; 120] {
    let mut out = [[0; 4]; 120];
    let mut root = 0;
    while root < 120 {
        let mut axis = 0;
        while axis < 4 {
            let [a, b] = H4_ROOT_COEFFICIENTS[root][axis];
            // Canonical F32 phi/2 is exactly 27146106 / 2^25.
            let golden = if b < 0 {
                -27146106
            } else if b > 0 {
                27146106
            } else {
                0
            };
            out[root][axis] = ((a as i32) << 24) + golden;
            axis += 1;
        }
        root += 1;
    }
    out
}
pub const CANONICAL_BASIS_Q25: [[i32; 4]; 120] = make_basis();
pub fn canonical_basis_q25() -> [[i32; 4]; 120] {
    CANONICAL_BASIS_Q25
}
pub fn pack_coefficients(values: &[i8]) -> Result<Vec<u8>, NoReadError> {
    let mut packed = vec![0; values.len().div_ceil(2)];
    for (i, &value) in values.iter().enumerate() {
        if !(-7..=7).contains(&value) {
            return Err(NoReadError::InvalidCoefficient { index: i, value });
        }
        packed[i >> 1] |= ((value as u8) & 15) << ((i & 1) << 2);
    }
    Ok(packed)
}
fn unpack(config: NoReadConfig, packed: &[u8]) -> Result<Vec<i8>, NoReadError> {
    let n = config.coefficient_count();
    let expected = n.div_ceil(2);
    if packed.len() != expected {
        return Err(NoReadError::CoefficientLength {
            expected,
            actual: packed.len(),
        });
    }
    if n & 1 != 0 && packed.last().copied().unwrap_or(0) & 0xf0 != 0 {
        return Err(NoReadError::NonzeroNibblePadding);
    }
    (0..n)
        .map(|i| {
            let nibble = (packed[i >> 1] >> ((i & 1) << 2)) & 15;
            let q = if nibble & 8 != 0 {
                nibble as i8 - 16
            } else {
                nibble as i8
            };
            if q == -8 {
                Err(NoReadError::InvalidCoefficient { index: i, value: q })
            } else {
                Ok(q)
            }
        })
        .collect()
}
fn dot_q24(coefficients: &[i8], root: &[i32; 4]) -> i32 {
    let mut sum = 0i64;
    for (&q, &x) in coefficients.iter().zip(root) {
        let mut magnitude = q.unsigned_abs();
        let mut term = i64::from(x);
        let mut product = 0;
        while magnitude != 0 {
            if magnitude & 1 != 0 {
                product += term;
            }
            magnitude >>= 1;
            term <<= 1;
        }
        sum += if q < 0 { -product } else { product };
    }
    if sum < 0 {
        -(((-sum + 4) >> 3) as i32)
    } else {
        ((sum + 4) >> 3) as i32
    }
}
#[repr(align(128))]
#[derive(Debug)]
struct Head {
    bias: i32,
    token: Box<[i32]>,
    latent: Box<[[i32; 128]]>,
    category: Box<[[i32; 64]]>,
    held: Box<[[i32; 128]]>,
    valid: Box<[i32]>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoReadStats {
    pub learned_coefficients: usize,
    pub packed_bytes: usize,
    pub expanded_table_bytes: usize,
    pub without_span_reads: usize,
    pub with_span_reads: usize,
}
#[derive(Debug)]
pub struct NativeGeometricNoRead {
    config: NoReadConfig,
    packed: Box<[u8]>,
    heads: Box<[Head]>,
}
impl NativeGeometricNoRead {
    pub fn new(config: NoReadConfig, packed: &[u8]) -> Result<Self, NoReadError> {
        config.validate()?;
        let q = unpack(config, packed)?;
        let lanes = config.lanes();
        let mut heads = Vec::with_capacity(config.heads);
        for row in q.chunks_exact(config.coefficients_per_head()) {
            let mut at = 0;
            let bias = i32::from(row[at]) << COEFFICIENT_SHIFT;
            at += 1;
            let token = row[at..at + config.vocabulary]
                .iter()
                .map(|&x| i32::from(x) << COEFFICIENT_SHIFT)
                .collect::<Vec<_>>()
                .into_boxed_slice();
            at += config.vocabulary;
            let mut latent = Vec::with_capacity(lanes);
            for coefficients in row[at..at + lanes * 4].chunks_exact(4) {
                let mut table = [0; 128];
                for (dst, root) in table.iter_mut().zip(CANONICAL_BASIS_Q25.iter()) {
                    *dst = dot_q24(coefficients, root);
                }
                latent.push(table);
            }
            at += lanes * 4;
            let mut category = Vec::with_capacity(lanes);
            for coefficients in row[at..at + lanes * 33].chunks_exact(33) {
                let mut table = [0; 64];
                for (dst, &x) in table.iter_mut().zip(coefficients) {
                    *dst = i32::from(x) << COEFFICIENT_SHIFT;
                }
                category.push(table);
            }
            at += lanes * 33;
            let mut held = Vec::with_capacity(lanes);
            for coefficients in row[at..at + lanes * 4].chunks_exact(4) {
                let mut table = [0; 128];
                for (dst, root) in table.iter_mut().zip(CANONICAL_BASIS_Q25.iter()) {
                    *dst = dot_q24(coefficients, root);
                }
                held.push(table);
            }
            at += lanes * 4;
            let valid = row[at..]
                .iter()
                .map(|&x| i32::from(x) << COEFFICIENT_SHIFT)
                .collect::<Vec<_>>()
                .into_boxed_slice();
            heads.push(Head {
                bias,
                token,
                latent: latent.into_boxed_slice(),
                category: category.into_boxed_slice(),
                held: held.into_boxed_slice(),
                valid,
            });
        }
        Ok(Self {
            config,
            packed: packed.into(),
            heads: heads.into_boxed_slice(),
        })
    }
    pub fn from_parts(
        config: NoReadConfig,
        packed: &[u8],
        expanded: &[i32],
    ) -> Result<Self, NoReadError> {
        let native = Self::new(config, packed)?;
        if native.expanded_q24() != expanded {
            return Err(NoReadError::TableMismatch);
        }
        Ok(native)
    }
    pub fn config(&self) -> NoReadConfig {
        self.config
    }
    pub fn packed_coefficients(&self) -> &[u8] {
        &self.packed
    }
    pub fn expanded_q24(&self) -> Vec<i32> {
        let mut out = Vec::new();
        for h in &self.heads {
            out.push(h.bias);
            out.extend_from_slice(&h.token);
            for row in &h.latent {
                out.extend_from_slice(row);
            }
            for row in &h.category {
                out.extend_from_slice(row);
            }
            for row in &h.held {
                out.extend_from_slice(row);
            }
            out.extend_from_slice(&h.valid);
        }
        out
    }
    pub fn stats(&self) -> NoReadStats {
        NoReadStats {
            learned_coefficients: self.config.coefficient_count(),
            packed_bytes: self.packed.len(),
            expanded_table_bytes: self
                .heads
                .iter()
                .map(|h| {
                    1 + h.token.len()
                        + h.latent.len() * 128
                        + h.category.len() * 64
                        + h.held.len() * 128
                        + h.valid.len()
                })
                .sum::<usize>()
                * 4,
            without_span_reads: self.config.heads * (2 + self.config.lanes() * 2),
            with_span_reads: self.config.heads * (2 + self.config.lanes() * 4),
        }
    }
    #[inline(never)]
    pub fn score(
        &self,
        token: usize,
        latent: &[H4Code],
        observed: &[AddressLane],
        old_held: Option<&[H4Code]>,
    ) -> Result<[i64; 2], NoReadError> {
        if token >= self.config.vocabulary {
            return Err(NoReadError::Token {
                token,
                vocabulary: self.config.vocabulary,
            });
        }
        let expected = self.heads[0].latent.len();
        for actual in [
            latent.len(),
            observed.len(),
            old_held.map_or(expected, |x| x.len()),
        ] {
            if actual != expected {
                return Err(NoReadError::Shape { expected, actual });
            }
        }
        let mut output = [0i64; 2];
        for (head, dst) in self.heads.iter().zip(output.iter_mut()) {
            let mut sum = i64::from(head.bias) + i64::from(head.token[token]);
            for (((root, address), table), categories) in latent
                .iter()
                .zip(observed)
                .zip(head.latent.iter())
                .zip(head.category.iter())
            {
                let category = if address.present() {
                    usize::from(address.radius_bin()) + 1
                } else {
                    0
                };
                sum +=
                    i64::from(table[usize::from(root.index())]) + i64::from(categories[category]);
            }
            if let Some(held) = old_held {
                for ((root, table), &valid) in
                    held.iter().zip(head.held.iter()).zip(head.valid.iter())
                {
                    sum += i64::from(table[usize::from(root.index())]) + i64::from(valid);
                }
            }
            *dst = sum;
        }
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn c() -> NoReadConfig {
        NoReadConfig {
            vocabulary: 3,
            heads: 1,
            latent_lanes_per_head: 1,
        }
    }
    fn root(v: u8) -> H4Code {
        H4Code::try_from(v).unwrap()
    }
    fn absent() -> AddressLane {
        AddressLane::new(1, 0, false).unwrap()
    }
    #[test]
    fn no_read_q4_tables_regenerate_and_reject_malformed_source() {
        let c = c();
        let q = vec![1; c.coefficient_count()];
        let packed = pack_coefficients(&q).unwrap();
        let n = NativeGeometricNoRead::new(c, &packed).unwrap();
        let tables = n.expanded_q24();
        assert_eq!(
            NativeGeometricNoRead::from_parts(c, &packed, &tables)
                .unwrap()
                .expanded_q24(),
            tables
        );
        let mut corrupt = tables.clone();
        corrupt[4 + 120] = 1;
        assert!(matches!(
            NativeGeometricNoRead::from_parts(c, &packed, &corrupt),
            Err(NoReadError::TableMismatch)
        ));
        let mut bad = packed.clone();
        bad[0] = 8;
        assert!(matches!(
            NativeGeometricNoRead::new(c, &bad),
            Err(NoReadError::InvalidCoefficient { .. })
        ));
        assert!(pack_coefficients(&[-8]).is_err());
        assert!(NativeGeometricNoRead::new(c, &packed[..packed.len() - 1]).is_err());
    }
    #[test]
    fn no_read_signed_roots_categories_and_old_held_are_distinct() {
        let c = c();
        let mut q = vec![0; c.coefficient_count()];
        q[4] = 4;
        q[8] = 2;
        q[8 + 32] = 3;
        q[41] = 4;
        q[45] = 1;
        let n = NativeGeometricNoRead::new(c, &pack_coefficients(&q).unwrap()).unwrap();
        let absent = n.score(0, &[root(1)], &[absent()], None).unwrap()[0];
        assert_eq!(absent, 3i64 << 23);
        let minus = n.score(0, &[root(0)], &[self::absent()], None).unwrap()[0];
        assert_eq!(absent - minus, 2i64 << 24);
        let present = n
            .score(
                0,
                &[root(1)],
                &[AddressLane::new(1, 31, true).unwrap()],
                None,
            )
            .unwrap()[0];
        assert_eq!(present - absent, 1i64 << 22);
        let held = n
            .score(0, &[root(1)], &[self::absent()], Some(&[root(1)]))
            .unwrap()[0];
        assert_eq!(held - absent, 5i64 << 22);
    }
    #[test]
    fn no_read_maximum_terms_and_errors_are_bounded() {
        let c = NoReadConfig {
            vocabulary: 4096,
            heads: 2,
            latent_lanes_per_head: 4,
        };
        let n = NativeGeometricNoRead::new(
            c,
            &pack_coefficients(&vec![7; c.coefficient_count()]).unwrap(),
        )
        .unwrap();
        let roots = [root(23); 8];
        let observed = [AddressLane::new(1, 31, true).unwrap(); 8];
        let out = n.score(4095, &roots, &observed, Some(&roots)).unwrap();
        assert_eq!(out, [1468006400; 2]);
        assert_eq!(n.stats().expanded_table_bytes, 53320);
        assert_eq!(n.stats().with_span_reads, 68);
        assert!(n.score(4096, &roots, &observed, None).is_err());
        assert!(n.score(0, &roots[..7], &observed, None).is_err());
        assert!(n.score(0, &roots, &observed, Some(&roots[..7])).is_err());
        assert_eq!(out, n.score(4095, &roots, &observed, Some(&roots)).unwrap());
    }
    #[test]
    fn no_read_basis_signed_dyadics_and_rounding() {
        assert_eq!(CANONICAL_BASIS_Q25[0], [-33554432, 0, 0, 0]);
        assert_eq!(CANONICAL_BASIS_Q25[1], [33554432, 0, 0, 0]);
        assert_eq!(dot_q24(&[1, 0, 0, 0], &[4, 0, 0, 0]), 1);
        assert_eq!(dot_q24(&[-1, 0, 0, 0], &[4, 0, 0, 0]), -1);
        for row in CANONICAL_BASIS_Q25 {
            assert!(row.iter().map(|x| i64::from(*x).abs()).sum::<i64>() <= 2i64 << 25);
        }
    }
}
