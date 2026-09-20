//! Versioned, executable hard artifact for the older-prefix group-state channel.
//!
//! The earlier pilot wrote floating `CPXS` checkpoints plus a JSON descriptor holding only hashes:
//! that is not a serving artifact, and its group table was built lazily by floating nearest-root
//! classification. This module supplies what the principal review requires:
//!
//! * a **bound exact `2I` product table** derived from the exact twice-scaled `Z[phi]` root
//!   construction in `canonical_lexical_ingestion`, with an explicit bijection between the
//!   historical `learner::embedding` enumeration and the sorted exact order - no runtime floating
//!   nearest-root construction;
//! * packed reader/output coefficients **with their fixed shifts**, nibble-packed action codes, the
//!   palette, arm semantics, dimensions and parent/tokenizer identities in one `CPX2` file; and
//! * a validated loader that rejects malformed dimensions, codes, shifts, table contents, index
//!   mappings and parent identity.
//!
//! The learned function is unchanged: token action codes are palette-slot IDs, and the exact table
//! is stored in the **historical** state order, so reader rows and action IDs keep their meaning.
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use super::embedding::{canonical_h4_roots_q30, H4_ROOT_COUNT};
use super::group_table::{group_table, GROUP_ORDER, ROW_STRIDE};
use super::lowbit::TernaryLinear;
use super::prefix_state::{Arm, Palette, PALETTE_SIZE};
use super::prior_learning::PriorCore;
use crate::canonical_lexical_ingestion::validate_h4_binary_icosahedral_closure;

pub const PREFIX_ARTIFACT_MAGIC: &[u8; 4] = b"CPX2";
pub const PREFIX_ARTIFACT_VERSION: u32 = 2;
/// Reader width and fixed shifts for the recovered channel.
pub const READER_WIDTH: usize = 16;
pub const READER_SHIFT: u32 = 5;
pub const OUTPUT_SHIFT: u32 = 3;
/// Context bound; the maximum older-prefix length is `CONTEXT - 2`.
pub const CONTEXT: usize = 64;
pub const MAX_OLDER: usize = 62;

type Zp = [i64; 2];
type Root = [Zp; 4];

// ---------------------------------------------------------------------------
// Exact Z[phi] arithmetic and the canonical root order
// ---------------------------------------------------------------------------

/// `(a + b*phi) * (c + d*phi)` in `Z[phi]`, using `phi^2 = phi + 1`.
fn zp_mul(x: Zp, y: Zp) -> Zp {
    [
        x[0] * y[0] + x[1] * y[1],
        x[0] * y[1] + x[1] * y[0] + x[1] * y[1],
    ]
}
fn zp_neg(x: Zp) -> Zp {
    [-x[0], -x[1]]
}
fn zp_sub(x: Zp, y: Zp) -> Zp {
    [x[0] - y[0], x[1] - y[1]]
}
fn zp_add(x: Zp, y: Zp) -> Zp {
    [x[0] + y[0], x[1] + y[1]]
}
fn zp_half(x: Zp) -> Option<Zp> {
    if x[0] % 2 != 0 || x[1] % 2 != 0 {
        return None;
    }
    Some([x[0] / 2, x[1] / 2])
}

/// Hamilton product in the same order as `group_table::hamilton`.
fn quat_mul(a: Root, b: Root) -> Root {
    let c0 = zp_sub(
        zp_sub(
            zp_sub(zp_mul(a[0], b[0]), zp_mul(a[1], b[1])),
            zp_mul(a[2], b[2]),
        ),
        zp_mul(a[3], b[3]),
    );
    let c1 = zp_add(
        zp_add(zp_mul(a[0], b[1]), zp_mul(a[1], b[0])),
        zp_sub(zp_mul(a[2], b[3]), zp_mul(a[3], b[2])),
    );
    let c2 = zp_add(
        zp_sub(zp_mul(a[0], b[2]), zp_mul(a[1], b[3])),
        zp_add(zp_mul(a[2], b[0]), zp_mul(a[3], b[1])),
    );
    let c3 = zp_add(
        zp_add(zp_mul(a[0], b[3]), zp_mul(a[1], b[2])),
        zp_sub(zp_mul(a[3], b[0]), zp_mul(a[2], b[1])),
    );
    [c0, c1, c2, c3]
}

/// The exact sorted canonical root table: 120 roots as twice-scaled `Z[phi]` tuples.
pub fn exact_sorted_roots() -> Vec<Root> {
    let zero: Zp = [0, 0];
    let mut set: BTreeSet<Root> = BTreeSet::new();
    for axis in 0..4 {
        for sign in [-1i64, 1] {
            let mut r = [zero; 4];
            r[axis] = [2 * sign, 0];
            set.insert(r);
        }
    }
    for signs in 0u8..16 {
        let mut r = [zero; 4];
        for (axis, slot) in r.iter_mut().enumerate() {
            *slot = [if signs & (1 << axis) == 0 { -1 } else { 1 }, 0];
        }
        set.insert(r);
    }
    let base: Root = [zero, [1, 0], [0, 1], [-1, 1]];
    for first in 0..4usize {
        for second in 0..4usize {
            for third in 0..4usize {
                for fourth in 0..4usize {
                    let perm = [first, second, third, fourth];
                    if perm.iter().copied().collect::<BTreeSet<_>>().len() != 4 {
                        continue;
                    }
                    let inversions = (0..4)
                        .flat_map(|l| ((l + 1)..4).map(move |r| (l, r)))
                        .filter(|(l, r)| perm[*l] > perm[*r])
                        .count();
                    if inversions % 2 != 0 {
                        continue;
                    }
                    for signs in 0u8..8 {
                        let mut signed = base;
                        for source in 1..4 {
                            if signs & (1 << (source - 1)) == 0 {
                                signed[source] = zp_neg(signed[source]);
                            }
                        }
                        set.insert(perm.map(|s| signed[s]));
                    }
                }
            }
        }
    }
    set.into_iter().collect()
}

/// The historical `learner::embedding` enumeration order, as exact twice-scaled tuples.
pub fn historical_roots() -> Vec<Root> {
    let zero: Zp = [0, 0];
    let mut v: Vec<Root> = Vec::with_capacity(H4_ROOT_COUNT);
    for axis in 0..4 {
        for sign in [-1i64, 1] {
            let mut r = [zero; 4];
            r[axis] = [2 * sign, 0];
            v.push(r);
        }
    }
    for i in 0..16u8 {
        let bit = |b: u8| -> i64 {
            if i & b != 0 {
                1
            } else {
                -1
            }
        };
        v.push([[bit(1), 0], [bit(2), 0], [bit(4), 0], [bit(8), 0]]);
    }
    const EVEN_PERMS: [[usize; 4]; 12] = [
        [0, 1, 2, 3],
        [0, 2, 3, 1],
        [0, 3, 1, 2],
        [1, 0, 3, 2],
        [1, 2, 0, 3],
        [1, 3, 2, 0],
        [2, 0, 1, 3],
        [2, 1, 3, 0],
        [2, 3, 0, 1],
        [3, 0, 2, 1],
        [3, 1, 0, 2],
        [3, 2, 1, 0],
    ];
    let base: Root = [zero, [1, 0], [0, 1], [-1, 1]];
    for perm in EVEN_PERMS {
        for s1 in [-1i64, 1] {
            for s2 in [-1i64, 1] {
                for s3 in [-1i64, 1] {
                    let mut signed = base;
                    if s1 < 0 {
                        signed[1] = zp_neg(signed[1]);
                    }
                    if s2 < 0 {
                        signed[2] = zp_neg(signed[2]);
                    }
                    if s3 < 0 {
                        signed[3] = zp_neg(signed[3]);
                    }
                    v.push([
                        signed[perm[0]],
                        signed[perm[1]],
                        signed[perm[2]],
                        signed[perm[3]],
                    ]);
                }
            }
        }
    }
    v
}

/// A bound exact `2I` table in historical state order, with its provenance mapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExactGroupTable {
    /// Row-major products in historical state order, stride `ROW_STRIDE` (padding zero).
    pub product: Box<[u8]>,
    pub identity: u8,
    pub inverse: Box<[u8]>,
    /// Historical index -> sorted exact index, for provenance.
    pub historical_to_sorted: Box<[u8]>,
    /// Mismatches against the historical floating table under the bijection (must be 0).
    pub max_mismatches_vs_floating: usize,
}

impl ExactGroupTable {
    pub fn compose(&self, state: u8, gamma: u8) -> u8 {
        self.product[state as usize * ROW_STRIDE + gamma as usize]
    }

    /// Build and cross-verify the exact table. All 14,400 products are checked against BOTH the
    /// independent exact `Z[phi]` multiplication and the canonical closure before returning.
    pub fn build() -> Result<Self, String> {
        let sorted = exact_sorted_roots();
        if sorted.len() != H4_ROOT_COUNT {
            return Err(format!(
                "exact sorted roots: {} != {H4_ROOT_COUNT}",
                sorted.len()
            ));
        }
        let hist = historical_roots();
        if hist.len() != H4_ROOT_COUNT {
            return Err(format!(
                "historical roots: {} != {H4_ROOT_COUNT}",
                hist.len()
            ));
        }
        if hist.iter().copied().collect::<BTreeSet<_>>().len() != H4_ROOT_COUNT {
            return Err("historical enumeration repeats a root".into());
        }
        let mut map: Vec<u8> = Vec::with_capacity(H4_ROOT_COUNT);
        for r in hist.iter() {
            let idx = sorted
                .binary_search(r)
                .map_err(|_| "historical root absent from the exact sorted table".to_string())?;
            map.push(idx as u8);
        }
        if map.iter().copied().collect::<BTreeSet<_>>().len() != H4_ROOT_COUNT {
            return Err("historical enumeration is not a bijection onto the exact roots".into());
        }
        let closure = validate_h4_binary_icosahedral_closure()
            .map_err(|e| format!("canonical closure: {e}"))?;
        if closure.root_count != H4_ROOT_COUNT {
            return Err("canonical closure root count mismatch".into());
        }
        let mut inv_map: Vec<u8> = vec![0; H4_ROOT_COUNT];
        for (h, &s) in map.iter().enumerate() {
            inv_map[s as usize] = h as u8;
        }
        let identity_hist = inv_map[closure.identity_index as usize];
        let mut product = vec![0u8; H4_ROOT_COUNT * ROW_STRIDE];
        let floating = group_table();
        let mut mismatch = 0usize;
        for hi in 0..H4_ROOT_COUNT {
            for hj in 0..H4_ROOT_COUNT {
                let prod = half(quat_mul(hist[hi], hist[hj]))?;
                let si = sorted
                    .binary_search(&prod)
                    .map_err(|_| format!("exact product ({hi},{hj}) is not a root"))?;
                let ci = closure
                    .product_index(map[hi] as u16, map[hj] as u16)
                    .ok_or("canonical closure product out of range")?
                    as usize;
                if si != ci {
                    return Err(format!(
                        "independent exact product ({hi},{hj}) disagrees with the canonical closure: {si} vs {ci}"
                    ));
                }
                let hprod = inv_map[si];
                product[hi * ROW_STRIDE + hj] = hprod;
                if floating.product[hi * ROW_STRIDE + hj] != hprod {
                    mismatch += 1;
                }
            }
        }
        let mut inverse = vec![0u8; H4_ROOT_COUNT];
        for a in 0..H4_ROOT_COUNT {
            inverse[a] = (0..H4_ROOT_COUNT)
                .find(|&b| product[a * ROW_STRIDE + b] == identity_hist)
                .ok_or_else(|| format!("no exact inverse for {a}"))? as u8;
        }
        Ok(Self {
            product: product.into_boxed_slice(),
            identity: identity_hist,
            inverse: inverse.into_boxed_slice(),
            historical_to_sorted: map.into_boxed_slice(),
            max_mismatches_vs_floating: mismatch,
        })
    }
}

fn half(q: Root) -> Result<Root, String> {
    let mut out = [[0i64; 2]; 4];
    for (dst, src) in out.iter_mut().zip(q.iter()) {
        *dst = zp_half(*src).ok_or("scaled H4 product is not exactly divisible by two")?;
    }
    Ok(out)
}

/// The legacy `parent_sha256` convention: SHA256 of the hex-encoded parent artifact digest string.
/// Returns `(raw file digest, legacy convention digest)`.
pub fn parent_hash_convention(artifact_bytes: &[u8]) -> ([u8; 32], [u8; 32]) {
    use sha2::{Digest, Sha256};
    let file_digest: [u8; 32] = Sha256::digest(artifact_bytes).into();
    let string_digest: [u8; 32] = Sha256::digest(hex_lower(&file_digest).as_bytes()).into();
    (file_digest, string_digest)
}

pub fn hex_lower(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

// ---------------------------------------------------------------------------
// The executable artifact
// ---------------------------------------------------------------------------

/// An executable hard prefix artifact: frozen parent plus the learned residual channel and the
/// bound exact group table.
#[derive(Debug, Clone)]
pub struct PrefixHard {
    pub parent: PriorCore,
    pub reader: TernaryLinear,
    pub wg: TernaryLinear,
    /// Unpacked palette-slot IDs, one byte per token.
    pub action_codes: Vec<u8>,
    pub palette: Palette,
    pub arm: Arm,
    pub table: ExactGroupTable,
    pub parent_file_digest: [u8; 32],
    pub tokenizer_digest: [u8; 32],
}

impl PrefixHard {
    pub fn residual_scores(&self, q: usize) -> Vec<i32> {
        let mut h = vec![0i32; READER_WIDTH];
        for (j, slot) in h.iter_mut().enumerate() {
            *slot = self.reader.weight(q, j) << self.reader.shift(q);
        }
        self.wg.forward_i32(&h)
    }

    pub fn int_logits(&self, prev: usize, cur: usize, q: Option<usize>) -> Vec<i32> {
        let mut z = self.parent.int_logits(prev, cur, true);
        if let Some(q) = q {
            for (a, b) in z.iter_mut().zip(self.residual_scores(q).iter()) {
                *a += *b;
            }
        }
        z
    }

    #[inline]
    pub fn action(&self, token: u32) -> usize {
        let v = self.parent.cfg.vocab;
        self.action_codes[(token as usize).min(v - 1)] as usize
    }

    pub fn state_older(&self, tokens: &[u32], i: usize) -> Option<usize> {
        if i < 2 {
            return None;
        }
        let start = i.saturating_sub(CONTEXT - 1);
        let end = i - 1;
        if end <= start {
            return None;
        }
        let mut q = self.table.identity;
        for &tok in &tokens[start..end] {
            q = self
                .table
                .compose(q, self.palette.elements[self.action(tok)]);
        }
        Some(q as usize)
    }

    /// The local-tail state: the product of the two local tokens' actions.
    pub fn state_tail(&self, tokens: &[u32], i: usize) -> Option<usize> {
        if i < 2 {
            return None;
        }
        let mut q = self.table.identity;
        for &tok in &tokens[i - 1..=i] {
            q = self
                .table
                .compose(q, self.palette.elements[self.action(tok)]);
        }
        Some(q as usize)
    }

    /// **Arm-aware state selection** shared by training, evaluation and generation.
    pub fn state_for_arm(&self, tokens: &[u32], i: usize) -> Option<usize> {
        match self.arm {
            Arm::LearnedTail => self.state_tail(tokens, i),
            _ => self.state_older(tokens, i),
        }
    }

    /// Greedy continuation with arm-aware state and no decoder change.
    pub fn generate(&self, prompt: &[u32], n_new: usize) -> Vec<u32> {
        let mut toks = prompt.to_vec();
        let v = self.parent.cfg.vocab;
        for _ in 0..n_new {
            let i = toks.len() - 1;
            let prev = if i == 0 {
                self.parent.cfg.pad_row()
            } else {
                (toks[i - 1] as usize).min(v - 1)
            };
            let cur = (toks[i] as usize).min(v - 1);
            let q = self.state_for_arm(&toks, i);
            let z = self.int_logits(prev, cur, q);
            let mut best = 0usize;
            for r in 1..z.len() {
                if z[r] > z[best] {
                    best = r;
                }
            }
            toks.push(best as u32);
        }
        toks[prompt.len()..].to_vec()
    }

    pub fn weight_bytes(&self) -> usize {
        self.reader.weight_bytes() + self.wg.weight_bytes() + self.action_codes.len().div_ceil(2)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut o = Vec::new();
        o.extend_from_slice(PREFIX_ARTIFACT_MAGIC);
        o.extend_from_slice(&PREFIX_ARTIFACT_VERSION.to_le_bytes());
        o.extend_from_slice(&(self.parent.cfg.vocab as u32).to_le_bytes());
        o.extend_from_slice(&(self.parent.cfg.dv as u32).to_le_bytes());
        o.extend_from_slice(&(READER_WIDTH as u32).to_le_bytes());
        o.extend_from_slice(&READER_SHIFT.to_le_bytes());
        o.extend_from_slice(&OUTPUT_SHIFT.to_le_bytes());
        o.extend_from_slice(&(CONTEXT as u32).to_le_bytes());
        o.extend_from_slice(&self.parent.cfg.f_bits.to_le_bytes());
        o.extend_from_slice(&self.parent.cfg.bias_scale_bits.to_le_bytes());
        o.push(self.arm as u8);
        o.extend_from_slice(&self.parent_file_digest);
        o.extend_from_slice(&self.tokenizer_digest);
        o.extend_from_slice(&self.palette.elements);
        o.push(self.palette.identity);
        o.push(self.table.identity);
        o.extend_from_slice(&self.table.product);
        o.extend_from_slice(&self.table.inverse);
        o.extend_from_slice(&self.table.historical_to_sorted);
        o.extend_from_slice(&(self.reader.rows as u32).to_le_bytes());
        o.extend_from_slice(&(self.reader.cols as u32).to_le_bytes());
        for r in 0..self.reader.rows {
            o.extend_from_slice(&self.reader.shift(r).to_le_bytes());
        }
        o.extend_from_slice(self.reader.packed());
        o.extend_from_slice(&(self.wg.rows as u32).to_le_bytes());
        o.extend_from_slice(&(self.wg.cols as u32).to_le_bytes());
        for r in 0..self.wg.rows {
            o.extend_from_slice(&self.wg.shift(r).to_le_bytes());
        }
        o.extend_from_slice(self.wg.packed());
        let v = self.parent.cfg.vocab;
        let mut packed = vec![0u8; v.div_ceil(2)];
        for (t, &code) in self.action_codes.iter().enumerate() {
            packed[t >> 1] |= (code & 0xF) << (4 * (t & 1) as u32);
        }
        o.extend_from_slice(&packed);
        o
    }

    /// Validated load. Rejects malformed dimensions, codes, shifts, table content, mapping and
    /// parent identity before returning.
    pub fn from_bytes(
        bytes: &[u8],
        parent: &PriorCore,
        expected_parent_digest: &[u8; 32],
    ) -> Result<Self, String> {
        let mut c = 0usize;
        let take = |c: &mut usize, n: usize| -> Result<&[u8], String> {
            let end = c.checked_add(n).ok_or("size overflow")?;
            if end > bytes.len() {
                return Err("truncated prefix artifact".into());
            }
            let s = &bytes[*c..end];
            *c = end;
            Ok(s)
        };
        if take(&mut c, 4)? != PREFIX_ARTIFACT_MAGIC {
            return Err("bad prefix artifact magic".into());
        }
        let u32_at = |c: &mut usize| -> Result<u32, String> {
            Ok(u32::from_le_bytes(take(c, 4)?.try_into().unwrap()))
        };
        if u32_at(&mut c)? != PREFIX_ARTIFACT_VERSION {
            return Err("unsupported prefix artifact version".into());
        }
        let vocab = u32_at(&mut c)? as usize;
        let dv = u32_at(&mut c)? as usize;
        let reader_width = u32_at(&mut c)? as usize;
        let reader_shift = u32_at(&mut c)?;
        let output_shift = u32_at(&mut c)?;
        let context = u32_at(&mut c)? as usize;
        let f_bits = u32_at(&mut c)?;
        let bias_scale_bits = u32_at(&mut c)?;
        let arm = Arm::from_u8(take(&mut c, 1)?[0])?;
        let mut parent_file_digest = [0u8; 32];
        parent_file_digest.copy_from_slice(take(&mut c, 32)?);
        let mut tokenizer_digest = [0u8; 32];
        tokenizer_digest.copy_from_slice(take(&mut c, 32)?);
        let mut elements = [0u8; PALETTE_SIZE];
        elements.copy_from_slice(take(&mut c, PALETTE_SIZE)?);
        let palette_identity = take(&mut c, 1)?[0];
        let table_identity = take(&mut c, 1)?[0];
        let product = take(&mut c, H4_ROOT_COUNT * ROW_STRIDE)?.to_vec();
        let inverse = take(&mut c, H4_ROOT_COUNT)?.to_vec();
        let mapping = take(&mut c, H4_ROOT_COUNT)?.to_vec();

        if vocab != parent.cfg.vocab
            || dv != parent.cfg.dv
            || reader_width != READER_WIDTH
            || reader_shift != READER_SHIFT
            || output_shift != OUTPUT_SHIFT
            || context != CONTEXT
            || f_bits != parent.cfg.f_bits
            || bias_scale_bits != parent.cfg.bias_scale_bits
        {
            return Err("prefix artifact configuration differs from the supplied parent".into());
        }
        if &parent_file_digest != expected_parent_digest {
            return Err("prefix artifact parent digest differs".into());
        }
        for a in 0..H4_ROOT_COUNT {
            let mut row = [false; H4_ROOT_COUNT];
            for b in 0..H4_ROOT_COUNT {
                let p = product[a * ROW_STRIDE + b] as usize;
                if p >= H4_ROOT_COUNT || row[p] {
                    return Err(format!("exact product table row {a} is not a bijection"));
                }
                row[p] = true;
            }
        }
        let e = table_identity as usize;
        if e >= H4_ROOT_COUNT {
            return Err("bound table identity out of range".into());
        }
        for a in 0..H4_ROOT_COUNT {
            if product[a * ROW_STRIDE + e] as usize != a
                || product[e * ROW_STRIDE + a] as usize != a
            {
                return Err("identity law failed in the bound table".into());
            }
            let inv = inverse[a] as usize;
            if inv >= H4_ROOT_COUNT || product[a * ROW_STRIDE + inv] as usize != e {
                return Err("inverse law failed in the bound table".into());
            }
        }
        let mut seen = [false; H4_ROOT_COUNT];
        for &m in mapping.iter() {
            if (m as usize) >= H4_ROOT_COUNT || seen[m as usize] {
                return Err("root mapping is not a bijection".into());
            }
            seen[m as usize] = true;
        }

        let reader_rows = u32_at(&mut c)? as usize;
        let reader_cols = u32_at(&mut c)? as usize;
        if reader_rows != H4_ROOT_COUNT || reader_cols != READER_WIDTH {
            return Err("reader shape mismatch".into());
        }
        let mut r_shift = Vec::with_capacity(reader_rows);
        for _ in 0..reader_rows {
            let s = u32_at(&mut c)?;
            if s != READER_SHIFT {
                return Err("reader shift differs from the declared fixed scale".into());
            }
            r_shift.push(s);
        }
        let r_packed = take(&mut c, (reader_rows * reader_cols).div_ceil(4))?.to_vec();
        let reader = TernaryLinear::from_packed(r_packed, r_shift, reader_rows, reader_cols)?;

        let out_rows = u32_at(&mut c)? as usize;
        let out_cols = u32_at(&mut c)? as usize;
        if out_rows != vocab || out_cols != READER_WIDTH {
            return Err("output head shape mismatch".into());
        }
        let mut o_shift = Vec::with_capacity(out_rows);
        for _ in 0..out_rows {
            o_shift.push(u32_at(&mut c)?);
        }
        let o_packed = take(&mut c, (out_rows * out_cols).div_ceil(4))?.to_vec();
        let wg = TernaryLinear::from_packed(o_packed, o_shift, out_rows, out_cols)?;

        let action_packed = take(&mut c, vocab.div_ceil(2))?;
        let mut action_codes = Vec::with_capacity(vocab);
        for t in 0..vocab {
            let code = (action_packed[t >> 1] >> (4 * (t & 1) as u32)) & 0xF;
            if (code as usize) >= PALETTE_SIZE {
                return Err(format!(
                    "action code {code} for token {t} is outside the palette"
                ));
            }
            action_codes.push(code);
        }
        if c != bytes.len() {
            return Err(format!(
                "{} trailing bytes in the prefix artifact",
                bytes.len() - c
            ));
        }
        for &element in elements.iter() {
            if (element as usize) >= H4_ROOT_COUNT {
                return Err("palette element outside the group".into());
            }
        }
        if palette_identity != table_identity {
            return Err("palette identity must match the bound table identity".into());
        }
        Ok(Self {
            parent: parent.clone(),
            reader,
            wg,
            action_codes,
            palette: Palette {
                elements,
                identity: palette_identity,
            },
            arm,
            table: ExactGroupTable {
                product: product.into_boxed_slice(),
                identity: table_identity,
                inverse: inverse.into_boxed_slice(),
                historical_to_sorted: mapping.into_boxed_slice(),
                max_mismatches_vs_floating: 0,
            },
            parent_file_digest,
            tokenizer_digest,
        })
    }
}

pub fn canonical_group_order() -> usize {
    GROUP_ORDER
}

pub fn floating_root_count() -> usize {
    canonical_h4_roots_q30().len()
}

/// Full-vocabulary argmax with lowest-ID tie handling.
pub fn argmax_full(z: &[i32]) -> usize {
    let mut best = 0usize;
    for r in 1..z.len() {
        if z[r] > z[best] {
            best = r;
        }
    }
    best
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_geometric::learner::prefix_state::palette;
    use crate::native_geometric::learner::prior_learning::{bias_codes_from_counts, Config};

    fn tiny_parent(vocab: usize, dv: usize) -> PriorCore {
        let counts = vec![1u64; vocab];
        let bias = bias_codes_from_counts(&counts, vocab as u64, vocab);
        let cfg = Config::new(vocab, dv);
        let mut st = 0x1234_5678u64;
        let mut fill = |n: usize| -> Vec<f32> {
            (0..n)
                .map(|_| {
                    st ^= st << 13;
                    st ^= st >> 7;
                    st ^= st << 17;
                    let u = (st as f64 / u64::MAX as f64) as f32;
                    (u * 2.0 - 1.0) * 1.5
                })
                .collect()
        };
        let e_old = fill((vocab + 1) * dv);
        let e_new = fill(vocab * dv);
        let wo = fill(vocab * dv);
        PriorCore {
            cfg,
            elements: (0..vocab).map(|t| (t % H4_ROOT_COUNT) as u16).collect(),
            e_old: TernaryLinear::quantize(&e_old, vocab + 1, dv),
            e_new: TernaryLinear::quantize(&e_new, vocab, dv),
            w_o: TernaryLinear::quantize(&wo, vocab, dv),
            bias_codes: bias,
        }
    }

    #[test]
    fn exact_table_matches_the_floating_table_under_the_bijection() {
        let t = ExactGroupTable::build().expect("exact table");
        assert_eq!(t.product.len(), H4_ROOT_COUNT * ROW_STRIDE);
        assert_eq!(
            t.max_mismatches_vs_floating, 0,
            "the exact table must agree with the historical floating table under the bijection"
        );
        for a in 0..H4_ROOT_COUNT {
            assert_eq!(t.product[a * ROW_STRIDE + t.identity as usize] as usize, a);
            assert_eq!(t.product[t.identity as usize * ROW_STRIDE + a] as usize, a);
            assert_eq!(
                t.product[a * ROW_STRIDE + t.inverse[a] as usize],
                t.identity
            );
        }
    }

    #[test]
    fn historical_mapping_is_a_bijection_and_preserves_negation() {
        let t = ExactGroupTable::build().expect("exact table");
        let sorted = exact_sorted_roots();
        let hist = historical_roots();
        let mut seen = [false; H4_ROOT_COUNT];
        for (h, &s) in t.historical_to_sorted.iter().enumerate() {
            assert!(!seen[s as usize]);
            seen[s as usize] = true;
            assert_eq!(sorted[s as usize], hist[h]);
        }
        for (h, r) in hist.iter().enumerate() {
            let neg = r.map(zp_neg);
            let n = sorted.binary_search(&neg).expect("negation must be a root");
            let nh = hist
                .iter()
                .position(|x| *x == neg)
                .expect("historical negation");
            assert_eq!(t.historical_to_sorted[nh] as usize, n);
            assert_ne!(h, nh, "q and -q stay distinct");
        }
    }

    #[test]
    fn sorted_table_matches_the_canonical_closure_identity() {
        let sorted = exact_sorted_roots();
        assert_eq!(sorted.len(), H4_ROOT_COUNT);
        let closure = validate_h4_binary_icosahedral_closure().expect("closure");
        assert_eq!(closure.root_count, sorted.len());
        let identity = [[2, 0], [0, 0], [0, 0], [0, 0]];
        assert_eq!(
            closure.identity_index as usize,
            sorted
                .iter()
                .position(|r| *r == identity)
                .expect("identity")
        );
    }

    #[test]
    fn palette_identity_matches_the_bound_table_identity() {
        let t = ExactGroupTable::build().expect("exact table");
        assert_eq!(palette().identity, t.identity);
    }

    #[test]
    fn artifact_round_trips_and_rejects_malformed_input() {
        let parent = tiny_parent(64, 32);
        let t = ExactGroupTable::build().expect("exact table");
        let (file_digest, _) = parent_hash_convention(b"fake");
        let hard = PrefixHard {
            parent: parent.clone(),
            reader: TernaryLinear::from_packed(
                vec![0u8; (H4_ROOT_COUNT * READER_WIDTH).div_ceil(4)],
                vec![READER_SHIFT; H4_ROOT_COUNT],
                H4_ROOT_COUNT,
                READER_WIDTH,
            )
            .unwrap(),
            wg: TernaryLinear::from_packed(
                vec![0u8; (64 * READER_WIDTH).div_ceil(4)],
                vec![OUTPUT_SHIFT; 64],
                64,
                READER_WIDTH,
            )
            .unwrap(),
            action_codes: vec![0u8; 64],
            palette: Palette {
                elements: palette().elements,
                identity: t.identity,
            },
            arm: Arm::LearnedOlder,
            table: t,
            parent_file_digest: file_digest,
            tokenizer_digest: [3u8; 32],
        };
        let bytes = hard.to_bytes();
        let reload = PrefixHard::from_bytes(&bytes, &parent, &file_digest).expect("round trip");
        assert_eq!(reload.action_codes, hard.action_codes);
        assert_eq!(reload.table.product, hard.table.product);
        assert_eq!(reload.arm, Arm::LearnedOlder);
        assert!(PrefixHard::from_bytes(&bytes, &parent, &[9u8; 32]).is_err());
        assert!(PrefixHard::from_bytes(&bytes[..bytes.len() - 1], &parent, &file_digest).is_err());
        let mut bad = bytes.clone();
        let n = bad.len();
        bad[n - 1] = 0xFF;
        assert!(PrefixHard::from_bytes(&bad, &parent, &file_digest).is_err());
    }
}
