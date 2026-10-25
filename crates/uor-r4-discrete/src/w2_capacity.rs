//! W2 read-code capacity: host counting, scoring and the band decision of the
//! registered sheet `docs/harness/prep/m1-read-code-capacity-2026-10-17.md`
//! (§4.2 hypotheses, §4.5 step 5, §4.6 evaluability, §4.7 decision rule).
//!
//! Pure host post-processing of GPU-produced read codes (D24 "CPU audits
//! stored GPU observations"). Counts and every decision are exact integers:
//! no floating point, no squashing, no matrix product. Each irrational floor
//! `2·sqrt(b+c)` is compared by squaring both sides in integers.

use std::collections::BTreeMap;

mod geometry {
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/geometry.rs"));
}

pub const VOCABULARY: usize = 4096;
pub const ROOT_COUNT: usize = 16;
pub const BLOCKS: usize = 4;
pub const BLOCK_WIDTH: usize = 8;
pub const TOP_K: usize = 8;
/// §4.6: rounded 2-SE floor at 149/14,287.
pub const FULL_PANEL_MIN_FLOOR: i64 = 24;
/// §4.6 multiplicity bar: distinct codes with ≥2 matched TRAIN counts.
pub const MIN_CODES_WITH_COUNT_GE_2: u64 = 1024;
/// §4.6 power rule: below this `b+c` the exact McNemar test must reject.
pub const MIN_POWERED_DISCORDANCE: u64 = 100;
/// §4.2: Q4 writer full-panel top-1.
pub const W_WRITER_TOP1: u64 = 149;
pub const MATCHED_TRAIN_ROWS: usize = 1_049_856;
pub const DEV_ROWS: usize = 14_287;
/// §4.5 fallback regime label the consumer verifier requires.
pub const FALLBACK_CAPTURE_MODE: &str = "production_forward_matched_1049856_fallback";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CapacityError {
    /// §4.5 step 5 / §4.7 precedence 0: a block matched no root. Instrument fault.
    AbsentRoot { block: [i32; BLOCK_WIDTH] },
    /// §4.8 item 4: no token ≥ 4,096.
    TokenOutOfRange { token: u16 },
}

impl std::fmt::Display for CapacityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AbsentRoot { block } => {
                write!(f, "INSTRUMENT_FAULT: ABSENT root for block {block:?}")
            }
            Self::TokenOutOfRange { token } => {
                write!(f, "INSTRUMENT_FAULT: token {token} >= {VOCABULARY}")
            }
        }
    }
}

impl std::error::Error for CapacityError {}

/// Exact-match root decode, mirroring `root_id` in `discrete.cu`: the first of
/// the 16 geometry roots equal to the block, else ABSENT.
pub fn decode_block(block: &[i32; BLOCK_WIDTH]) -> Result<u8, CapacityError> {
    geometry::ROOTS
        .iter()
        .position(|root| root.iter().zip(block).all(|(&r, &v)| i32::from(r) == v))
        .map(|r| r as u8)
        .ok_or(CapacityError::AbsentRoot { block: *block })
}

/// The 32-int read vector of one root per block.
pub fn root_vector(roots: [u8; BLOCKS]) -> [i32; BLOCKS * BLOCK_WIDTH] {
    let mut q = [0i32; BLOCKS * BLOCK_WIDTH];
    for (b, &r) in roots.iter().enumerate() {
        for d in 0..BLOCK_WIDTH {
            q[b * BLOCK_WIDTH + d] = i32::from(geometry::ROOTS[usize::from(r)][d]);
        }
    }
    q
}

/// `code = r0 | r1<<4 | r2<<8 | r3<<12` (consumer schema §5.2 nibble order).
pub fn pack_code(roots: [u8; BLOCKS]) -> u16 {
    roots
        .iter()
        .enumerate()
        .fold(0u16, |code, (b, &r)| code | (u16::from(r & 0xF) << (4 * b)))
}

pub fn unpack_code(code: u16) -> [u8; BLOCKS] {
    [0, 1, 2, 3].map(|b| ((code >> (4 * b)) & 0xF) as u8)
}

/// Decode a 32-int read vector (four 8-int blocks) to its packed code.
pub fn decode_read_vector(q: &[i32; BLOCKS * BLOCK_WIDTH]) -> Result<u16, CapacityError> {
    let mut roots = [0u8; BLOCKS];
    for (b, root) in roots.iter_mut().enumerate() {
        let block: [i32; BLOCK_WIDTH] = q[b * BLOCK_WIDTH..(b + 1) * BLOCK_WIDTH]
            .try_into()
            .expect("block width");
        *root = decode_block(&block)?;
    }
    Ok(pack_code(roots))
}

/// 8-bit `(first,last)` address key: nibbles 0 and 3.
pub fn address_key(code: u16) -> u16 {
    let r = unpack_code(code);
    u16::from(r[0]) | (u16::from(r[3]) << 4)
}

/// 8-bit `(root1,root2)` interior key: nibbles 1 and 2.
pub fn interior_key(code: u16) -> u16 {
    let r = unpack_code(code);
    u16::from(r[1]) | (u16::from(r[2]) << 4)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrainRow {
    pub code: u16,
    pub query_token: u16,
    pub target_token: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DevRow {
    pub code: u16,
    pub query_token: u16,
    pub target_token: u16,
    /// Writer hits from the production forward, re-verified against the sealed receipt.
    pub writer_hit1: bool,
    pub writer_hit8: bool,
}

/// Sheet tie rule: highest count, then lowest token id; at most `TOP_K` tokens.
fn rank<I: IntoIterator<Item = (u16, u64)>>(entries: I) -> Vec<u16> {
    let mut v: Vec<(u16, u64)> = entries.into_iter().filter(|&(_, c)| c > 0).collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    v.into_iter().take(TOP_K).map(|(t, _)| t).collect()
}

/// `count[key][token]`, sparse and deterministic.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KeyedTable {
    counts: BTreeMap<u16, BTreeMap<u16, u64>>,
    total: u64,
}

impl KeyedTable {
    fn add(&mut self, key: u16, token: u16) {
        *self
            .counts
            .entry(key)
            .or_default()
            .entry(token)
            .or_default() += 1;
        self.total += 1;
    }
    pub fn total(&self) -> u64 {
        self.total
    }
    /// Sum over every cell; equals `total()` when counts are conserved.
    pub fn cell_sum(&self) -> u64 {
        self.counts.values().flat_map(|m| m.values()).sum()
    }
    pub fn key_count(&self, key: u16) -> u64 {
        self.counts.get(&key).map_or(0, |m| m.values().sum())
    }
    pub fn distinct_keys(&self) -> u64 {
        self.counts.len() as u64
    }
    pub fn keys_with_count_ge(&self, n: u64) -> u64 {
        self.counts
            .values()
            .filter(|m| m.values().sum::<u64>() >= n)
            .count() as u64
    }
    /// Histogram `key count -> number of keys` (§4.6 count histogram).
    pub fn count_histogram(&self) -> BTreeMap<u64, u64> {
        let mut h = BTreeMap::new();
        for m in self.counts.values() {
            *h.entry(m.values().sum::<u64>()).or_default() += 1;
        }
        h
    }
    /// Majority count of the key's top token (numerator of `code_majority_fraction`).
    pub fn key_majority(&self, key: u16) -> u64 {
        self.counts
            .get(&key)
            .and_then(|m| m.values().max().copied())
            .unwrap_or(0)
    }
    fn ranked(&self, key: u16) -> Option<Vec<u16>> {
        self.counts
            .get(&key)
            .map(|m| rank(m.iter().map(|(&t, &c)| (t, c))))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unigram {
    counts: Vec<u64>,
    total: u64,
}

impl Default for Unigram {
    fn default() -> Self {
        Self {
            counts: vec![0; VOCABULARY],
            total: 0,
        }
    }
}

impl Unigram {
    pub fn total(&self) -> u64 {
        self.total
    }
    pub fn ranked(&self) -> Vec<u16> {
        rank(self.counts.iter().enumerate().map(|(t, &c)| (t as u16, c)))
    }
}

/// The model-matched tables, all built on the same TRAIN rows.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tables {
    /// Exact-code table T.
    pub code: KeyedTable,
    /// Query-token bigram G.
    pub bigram: KeyedTable,
    /// Secondary, descriptive: `(first,last)` and `(root1,root2)` tables.
    pub address_only: KeyedTable,
    pub interior_only: KeyedTable,
    /// Model-matched unigram U, also the fallback for every keyed table.
    pub unigram: Unigram,
}

pub fn build_tables(train: &[TrainRow]) -> Result<Tables, CapacityError> {
    let mut t = Tables::default();
    for row in train {
        for token in [row.query_token, row.target_token] {
            if usize::from(token) >= VOCABULARY {
                return Err(CapacityError::TokenOutOfRange { token });
            }
        }
        t.code.add(row.code, row.target_token);
        t.bigram.add(row.query_token, row.target_token);
        t.address_only.add(address_key(row.code), row.target_token);
        t.interior_only
            .add(interior_key(row.code), row.target_token);
        t.unigram.counts[usize::from(row.target_token)] += 1;
        t.unigram.total += 1;
    }
    Ok(t)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prediction {
    pub top8: Vec<u16>,
    pub fallback_used: bool,
}

impl Prediction {
    pub fn top1(&self) -> Option<u16> {
        self.top8.first().copied()
    }
    pub fn hit1(&self, target: u16) -> bool {
        self.top1() == Some(target)
    }
    pub fn hit8(&self, target: u16) -> bool {
        self.top8.contains(&target)
    }
}

/// Keyed prediction with the sheet's fallback: an unseen key uses the
/// model-matched unigram ranking. A seen key's top-8 is its own ranking only
/// (not padded), so top-8 is never inflated by the fallback.
pub fn predict(table: &KeyedTable, key: u16, unigram: &Unigram) -> Prediction {
    match table.ranked(key) {
        Some(top8) => Prediction {
            top8,
            fallback_used: false,
        },
        None => Prediction {
            top8: unigram.ranked(),
            fallback_used: true,
        },
    }
}

/// One DEV row's scored outcome (the host-computed part of the §4.3 per-row record).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DevScore {
    pub table: Prediction,
    pub seen_code: bool,
    pub code_train_count: u64,
    pub code_majority_count: u64,
    pub unigram_hit1: bool,
    pub unigram_hit8: bool,
    pub bigram: Prediction,
    pub address_only: Prediction,
    pub interior_only: Prediction,
    pub target_token: u16,
    pub writer_hit1: bool,
    pub writer_hit8: bool,
}

pub fn score_dev(tables: &Tables, dev: &[DevRow]) -> Result<Vec<DevScore>, CapacityError> {
    let uni = tables.unigram.ranked();
    dev.iter()
        .map(|row| {
            for token in [row.query_token, row.target_token] {
                if usize::from(token) >= VOCABULARY {
                    return Err(CapacityError::TokenOutOfRange { token });
                }
            }
            let target = row.target_token;
            let code_train_count = tables.code.key_count(row.code);
            Ok(DevScore {
                table: predict(&tables.code, row.code, &tables.unigram),
                seen_code: code_train_count >= 1,
                code_train_count,
                code_majority_count: tables.code.key_majority(row.code),
                unigram_hit1: uni.first() == Some(&target),
                unigram_hit8: uni.contains(&target),
                bigram: predict(&tables.bigram, row.query_token, &tables.unigram),
                address_only: predict(&tables.address_only, address_key(row.code), &tables.unigram),
                interior_only: predict(
                    &tables.interior_only,
                    interior_key(row.code),
                    &tables.unigram,
                ),
                target_token: target,
                writer_hit1: row.writer_hit1,
                writer_hit8: row.writer_hit8,
            })
        })
        .collect()
}

/// Paired discordance against comparator X: `b` = X hit & table miss, `c` = X miss & table hit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Discordance {
    pub b: u64,
    pub c: u64,
}

impl Discordance {
    pub fn n(&self) -> u64 {
        self.b + self.c
    }
    fn push(&mut self, comparator_hit: bool, table_hit: bool) {
        match (comparator_hit, table_hit) {
            (true, false) => self.b += 1,
            (false, true) => self.c += 1,
            _ => {}
        }
    }
}

/// Exact §4.6 aggregates over the DEV panel.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Aggregates {
    pub dev_rows: u64,
    pub t_top1: u64,
    pub t_top8: u64,
    pub u_top1: u64,
    pub u_top8: u64,
    pub g_top1: u64,
    pub g_top8: u64,
    pub w_top1: u64,
    pub w_top8: u64,
    pub address_only_top1: u64,
    pub interior_only_top1: u64,
    pub coverage_rows: u64,
    pub dev_rows_code_count_ge_2: u64,
    pub fallback_rows: u64,
    pub codes_with_count_ge_2: u64,
    pub distinct_codes: u64,
    pub matched_train_total: u64,
    pub bigram_train_total: u64,
    pub d_u: Discordance,
    pub d_g: Discordance,
    pub d_w: Discordance,
}

impl Aggregates {
    pub fn m_top1(&self) -> u64 {
        self.u_top1.max(self.g_top1)
    }
}

pub fn aggregate(tables: &Tables, scores: &[DevScore]) -> Aggregates {
    let mut a = Aggregates {
        dev_rows: scores.len() as u64,
        codes_with_count_ge_2: tables.code.keys_with_count_ge(2),
        distinct_codes: tables.code.distinct_keys(),
        matched_train_total: tables.code.cell_sum(),
        bigram_train_total: tables.bigram.cell_sum(),
        ..Aggregates::default()
    };
    for s in scores {
        let t = s.target_token;
        let th = s.table.hit1(t);
        let gh = s.bigram.hit1(t);
        a.t_top1 += u64::from(th);
        a.t_top8 += u64::from(s.table.hit8(t));
        a.u_top1 += u64::from(s.unigram_hit1);
        a.u_top8 += u64::from(s.unigram_hit8);
        a.g_top1 += u64::from(gh);
        a.g_top8 += u64::from(s.bigram.hit8(t));
        a.w_top1 += u64::from(s.writer_hit1);
        a.w_top8 += u64::from(s.writer_hit8);
        a.address_only_top1 += u64::from(s.address_only.hit1(t));
        a.interior_only_top1 += u64::from(s.interior_only.hit1(t));
        a.coverage_rows += u64::from(s.seen_code);
        a.dev_rows_code_count_ge_2 += u64::from(s.code_train_count >= 2);
        a.fallback_rows += u64::from(s.table.fallback_used);
        a.d_u.push(s.unigram_hit1, th);
        a.d_g.push(gh, th);
        a.d_w.push(s.writer_hit1, th);
    }
    a
}

/// Exact two-sided binomial (McNemar) test rejects at α = 0.05:
/// `2·Σ_{i≤min(b,c)} C(n,i) ≤ 0.05·2^n  ⇔  40·Σ C(n,i) ≤ 2^n`. Only used for `n < 100`.
pub fn exact_mcnemar_rejects_005(d: Discordance) -> bool {
    let n = d.n();
    if n == 0 {
        return false;
    }
    assert!(
        n < MIN_POWERED_DISCORDANCE,
        "exact test is only applied below the b+c=100 bar"
    );
    let k = d.b.min(d.c);
    let (mut tail, mut binom) = (0u128, 1u128);
    for i in 0..=k {
        if i > 0 {
            binom = binom * u128::from(n - i + 1) / u128::from(i);
        }
        tail += binom;
    }
    40u128.saturating_mul(tail) <= (1u128 << n)
}

/// §4.6 coverage bar: `ceil(0.9·n)` rows (12,859 of 14,287).
pub fn coverage_bar(dev_rows: u64) -> u64 {
    (9 * dev_rows).div_ceil(10)
}

/// `d > max(24, 2·sqrt(n))`, exact.
fn exceeds_floor(d: i64, n: u64) -> bool {
    d > FULL_PANEL_MIN_FLOOR && i128::from(d) * i128::from(d) > 4 * i128::from(n)
}

/// `d ≥ max(24, 2·sqrt(n))`, exact.
fn reaches_floor(d: i64, n: u64) -> bool {
    d >= FULL_PANEL_MIN_FLOOR && i128::from(d) * i128::from(d) >= 4 * i128::from(n)
}

/// `max(24, 2·sqrt(n)) + max(24, 2·sqrt(m)) < d`, exact in integers.
fn floor_sum_below(n: u64, m: u64, d: i64) -> bool {
    let d = i128::from(d);
    let rational = |x: u64| 4 * i128::from(x) <= 576; // floor is 24
    let (n4, m4) = (4 * i128::from(n), 4 * i128::from(m));
    match (rational(n), rational(m)) {
        (true, true) => 48 < d,
        (true, false) => d - 24 > 0 && (d - 24) * (d - 24) > m4,
        (false, true) => d - 24 > 0 && (d - 24) * (d - 24) > n4,
        // 2√n + 2√m < d ⇔ d > 0 ∧ r = d² − 4n − 4m > 0 ∧ r² > 64nm
        (false, false) => {
            let r = d * d - n4 - m4;
            d > 0 && r > 0 && r * r > 4 * n4 * m4
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Band {
    Underpowered,
    HLaw,
    HMixed,
    HRep,
    WriterDominates,
}

impl Band {
    /// Verdict label as the consumer verifier spells it.
    pub fn label(self) -> &'static str {
        match self {
            Self::Underpowered => "UNDERPOWERED",
            Self::HLaw => "H_law",
            Self::HMixed => "H_mixed",
            Self::HRep => "H_rep",
            Self::WriterDominates => "WRITER_DOMINATES",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decision {
    pub band: Band,
    /// §4.7 precedence row that fired (1..=5).
    pub precedence: u8,
    /// `M + F < W − H` (§4.2 guard); reported as an analysis note.
    pub pathological_ordering: bool,
    pub underpowered_reasons: Vec<String>,
}

/// §4.7 rows 2–5 (evaluability already passed). `n_ug = max(b_U+c_U, b_G+c_G)`
/// so `F = max(F_U, F_G) = max(24, 2·sqrt(n_ug))`; `H = max(24, 2·sqrt(n_w))`.
pub fn decide_band(t: u64, m: u64, w: u64, n_ug: u64, n_w: u64) -> (u8, Band, bool) {
    let (t, m, w) = (t as i64, m as i64, w as i64);
    // M + F < W − H  ⇔  F + H < W − M
    let pathological = floor_sum_below(n_ug, n_w, w - m);
    // T ≥ W − H  ⇔  W − T ≤ H
    let deficit = w - t;
    let t_ge_w_minus_h = deficit <= 0 || !exceeds_floor(deficit, n_w);
    if pathological && !t_ge_w_minus_h {
        (4, Band::HRep, true)
    } else if exceeds_floor(t - m, n_ug) {
        (2, Band::HLaw, pathological)
    } else if reaches_floor(t - w, n_w) {
        (3, Band::HMixed, pathological)
    } else if t_ge_w_minus_h {
        (4, Band::HRep, pathological)
    } else {
        (5, Band::WriterDominates, pathological)
    }
}

/// §4.6 evaluability; empty means every check passed.
pub fn underpowered_reasons(a: &Aggregates) -> Vec<String> {
    let mut r = Vec::new();
    let bar = coverage_bar(a.dev_rows);
    if a.coverage_rows < bar {
        r.push(format!(
            "coverage {}/{} < {bar}",
            a.coverage_rows, a.dev_rows
        ));
    }
    if a.codes_with_count_ge_2 < MIN_CODES_WITH_COUNT_GE_2 {
        r.push(format!(
            "codes_with_count_ge_2 {} < {MIN_CODES_WITH_COUNT_GE_2}",
            a.codes_with_count_ge_2
        ));
    }
    for (name, d) in [("U", a.d_u), ("G", a.d_g)] {
        if d.n() < MIN_POWERED_DISCORDANCE && !exact_mcnemar_rejects_005(d) {
            r.push(format!(
                "{name}: b+c={} < 100 and exact McNemar does not reject at 0.05",
                d.n()
            ));
        }
    }
    r
}

/// §4.7 in order: evaluability (row 1), then the band (rows 2–5).
pub fn decide(a: &Aggregates) -> Decision {
    let underpowered_reasons = underpowered_reasons(a);
    if !underpowered_reasons.is_empty() {
        return Decision {
            band: Band::Underpowered,
            precedence: 1,
            pathological_ordering: false,
            underpowered_reasons,
        };
    }
    let (precedence, band, pathological_ordering) = decide_band(
        a.t_top1,
        a.m_top1(),
        a.w_top1,
        a.d_u.n().max(a.d_g.n()),
        a.d_w.n(),
    );
    Decision {
        band,
        precedence,
        pathological_ordering,
        underpowered_reasons,
    }
}

/// Outputs of the kernel's signed-permutation `action()` (`discrete.cu`) over
/// every action × root that are not exactly one of the 16 roots. The root set
/// is closed under the 64 actions iff this is 0.
pub fn symmetry_action_non_roots() -> u64 {
    let mut misses = 0u64;
    for a in 0..geometry::AXES.len() / BLOCK_WIDTH {
        for x in &geometry::ROOTS {
            let mut y = [0i8; BLOCK_WIDTH];
            for (d, out) in y.iter_mut().enumerate() {
                let axis = geometry::AXES[a * BLOCK_WIDTH + d];
                let v = x[usize::from(axis & 7)];
                *out = if axis & 8 != 0 { -v } else { v };
            }
            misses += u64::from(!geometry::ROOTS.contains(&y));
        }
    }
    misses
}

/// `act[a][r]`: the root the kernel's action `a` maps root `r` to. Any action
/// output that is not an exact root is an instrument fault (sheet §4.3 item 6).
pub fn symmetry_root_actions() -> Result<Vec<[u8; ROOT_COUNT]>, String> {
    (0..geometry::AXES.len() / BLOCK_WIDTH)
        .map(|a| {
            let mut row = [0u8; ROOT_COUNT];
            for (r, x) in geometry::ROOTS.iter().enumerate() {
                let mut y = [0i8; BLOCK_WIDTH];
                for (d, out) in y.iter_mut().enumerate() {
                    let axis = geometry::AXES[a * BLOCK_WIDTH + d];
                    let v = x[usize::from(axis & 7)];
                    *out = if axis & 8 != 0 { -v } else { v };
                }
                row[r] = geometry::ROOTS
                    .iter()
                    .position(|root| *root == y)
                    .ok_or_else(|| format!("action {a} maps root {r} to a non-root"))?
                    as u8;
            }
            Ok(row)
        })
        .collect()
}

/// Canonical symmetry key of every code: the minimum packed code over the
/// orbit under the actions applied uniformly to all four root blocks.
pub fn symmetry_keys(actions: &[[u8; ROOT_COUNT]]) -> Vec<u16> {
    (0..=u16::MAX)
        .map(|code| {
            let roots = unpack_code(code);
            actions
                .iter()
                .map(|act| pack_code(roots.map(|r| act[usize::from(r)])))
                .min()
                .unwrap_or(code)
        })
        .collect()
}

/// `table_sym`: counts pooled over each code's symmetry orbit.
pub fn build_sym_table(train: &[TrainRow], keys: &[u16]) -> KeyedTable {
    let mut t = KeyedTable::default();
    for row in train {
        t.add(keys[usize::from(row.code)], row.target_token);
    }
    t
}

impl Unigram {
    /// Every seen token in the tie rule's order (no `TOP_K` cut).
    fn ranked_all(&self) -> Vec<u16> {
        let mut v: Vec<(u16, u64)> = (0..VOCABULARY as u16)
            .map(|t| (t, self.counts[usize::from(t)]))
            .filter(|&(_, c)| c > 0)
            .collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        v.into_iter().map(|(t, _)| t).collect()
    }
}

/// A row's admitted token set: `t` is admitted iff `token_cells[t]` is one of
/// the row's present `admitted_cell_slots` (the GPU export; no host recomputation).
pub struct Admission<'a> {
    pub token_cells: &'a [u16],
    pub slots: &'a [u16],
}

impl Admission<'_> {
    pub fn admits(&self, token: u16) -> bool {
        let cell = self.token_cells[usize::from(token)];
        cell != crate::w2_capture::ABSENT_SLOT && self.slots.contains(&cell)
    }
}

/// Admission-matched prediction (sheet §4.3 item 4): the key's admitted tokens
/// by (count desc, token asc), then the admitted unigram ranking, skipping
/// duplicates, to `TOP_K`. With no admitted token seen under the key, the
/// admitted unigram gives the whole prediction (`fallback_used`).
pub fn predict_admitted(
    table: &KeyedTable,
    key: u16,
    unigram_all: &[u16],
    admission: &Admission,
) -> Prediction {
    let own = table.counts.get(&key).map_or_else(Vec::new, |m| {
        rank(
            m.iter()
                .filter(|&(&t, _)| admission.admits(t))
                .map(|(&t, &c)| (t, c)),
        )
    });
    let fallback_used = own.is_empty();
    let mut top8 = own;
    for &t in unigram_all.iter().filter(|&&t| admission.admits(t)) {
        if top8.len() == TOP_K {
            break;
        }
        if !top8.contains(&t) {
            top8.push(t);
        }
    }
    Prediction {
        top8,
        fallback_used,
    }
}

/// The admission-matched arms of one DEV row, plus the full-panel `table_sym`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdmittedScore {
    pub table: Prediction,
    pub unigram: Prediction,
    pub bigram: Prediction,
    pub table_sym: Prediction,
    pub table_sym_full: Prediction,
    pub target_admitted: bool,
    pub writer_hit1: bool,
    pub writer_hit8: bool,
}

/// Score the admission-matched and symmetry-pooled arms; `slots[i]` is DEV row
/// `i`'s `admitted_cell_slots`. The writer arm is restricted by requiring the
/// target to be admitted. Descriptive only: it cannot move the verdict.
pub fn score_admitted(
    tables: &Tables,
    table_sym: &KeyedTable,
    sym_keys: &[u16],
    dev: &[DevRow],
    token_cells: &[u16],
    slots: &[Vec<u16>],
) -> Vec<AdmittedScore> {
    assert_eq!(
        slots.len(),
        dev.len(),
        "score_admitted: one slot list per DEV row"
    );
    let unigram_all = tables.unigram.ranked_all();
    let empty = KeyedTable::default();
    dev.iter()
        .zip(slots)
        .map(|(row, slots)| {
            let adm = Admission { token_cells, slots };
            let sym_key = sym_keys[usize::from(row.code)];
            let target_admitted = adm.admits(row.target_token);
            AdmittedScore {
                table: predict_admitted(&tables.code, row.code, &unigram_all, &adm),
                unigram: predict_admitted(&empty, 0, &unigram_all, &adm),
                bigram: predict_admitted(&tables.bigram, row.query_token, &unigram_all, &adm),
                table_sym: predict_admitted(table_sym, sym_key, &unigram_all, &adm),
                table_sym_full: predict(table_sym, sym_key, &tables.unigram),
                target_admitted,
                writer_hit1: row.writer_hit1 && target_admitted,
                writer_hit8: row.writer_hit8 && target_admitted,
            }
        })
        .collect()
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ArmCounts {
    pub hit1: u64,
    pub hit8: u64,
}

/// Restricted hit counts per arm (`table, unigram, bigram, table_sym, writer`),
/// over all rows and over the target-admitted rows, plus full-panel `table_sym` top-1.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AdmissionMatched {
    pub rows: u64,
    pub target_admitted_rows: u64,
    pub all: [ArmCounts; 5],
    pub target_admitted: [ArmCounts; 5],
    pub table_sym_top1_full: u64,
}

pub const ADMITTED_ARMS: [&str; 5] = ["table", "unigram", "bigram", "table_sym", "writer"];

pub fn admission_matched(scores: &[AdmittedScore], dev: &[DevRow]) -> AdmissionMatched {
    let mut m = AdmissionMatched::default();
    for (s, row) in scores.iter().zip(dev) {
        let t = row.target_token;
        let hits = [
            (s.table.hit1(t), s.table.hit8(t)),
            (s.unigram.hit1(t), s.unigram.hit8(t)),
            (s.bigram.hit1(t), s.bigram.hit8(t)),
            (s.table_sym.hit1(t), s.table_sym.hit8(t)),
            (s.writer_hit1, s.writer_hit8),
        ];
        m.rows += 1;
        m.target_admitted_rows += u64::from(s.target_admitted);
        m.table_sym_top1_full += u64::from(s.table_sym_full.hit1(t));
        for (k, (h1, h8)) in hits.into_iter().enumerate() {
            m.all[k].hit1 += u64::from(h1);
            m.all[k].hit8 += u64::from(h8);
            if s.target_admitted {
                m.target_admitted[k].hit1 += u64::from(h1);
                m.target_admitted[k].hit8 += u64::from(h8);
            }
        }
    }
    m
}

/// BOS token id; a document starts at each occurrence.
pub const BOS_ID: u16 = 0;
/// Causal window of the DEV panel (the sealed relation-subpanel context).
pub const DEV_CONTEXT: usize = 192;

/// Parse a UORT token file: 64-byte header (`UORT`, u32 version 1, u64 count,
/// u32 vocab 4096, zeros) then `count` u16 LE tokens.
pub fn parse_uort_tokens(bytes: &[u8]) -> Result<Vec<u16>, String> {
    if bytes.len() < 64 || &bytes[0..4] != b"UORT" {
        return Err("tokens: missing UORT header".into());
    }
    let version = u32::from_le_bytes(bytes[4..8].try_into().expect("4"));
    let count = u64::from_le_bytes(bytes[8..16].try_into().expect("8"));
    let vocab = u32::from_le_bytes(bytes[16..20].try_into().expect("4"));
    if version != 1 || vocab as usize != VOCABULARY || bytes[20..64].iter().any(|&b| b != 0) {
        return Err(format!("tokens: header version {version} vocab {vocab}"));
    }
    let body = &bytes[64..];
    if body.len() as u64 != 2 * count {
        return Err(format!(
            "tokens: {} body bytes for count {count}",
            body.len()
        ));
    }
    Ok(body
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect())
}

/// One eligible DEV row of the sealed relation subpanel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PanelRow {
    pub ordinal: usize,
    pub position: usize,
    pub document_id: usize,
    pub query_token: u16,
    pub target_token: u16,
    /// The most recent earlier `query` in the window is followed by `target`.
    pub copy_successor: bool,
    /// Distance bucket (0: 1..=8, 1: 9..=32, 2: 33..=96, 3: 97..=192), only
    /// when `copy_successor`.
    pub bucket: Option<u8>,
}

/// Eligible DEV rows (`p>0`, token≠BOS, mask==1) classified exactly as the
/// sealed relation-subpanel audit (`ee047cb88`) classifies them.
pub fn dev_panel(tokens: &[u16], mask: &[u8]) -> Result<Vec<PanelRow>, String> {
    if tokens.len() != mask.len() {
        return Err(format!("tokens {} != mask {}", tokens.len(), mask.len()));
    }
    let documents: Vec<usize> = (0..tokens.len()).filter(|&p| tokens[p] == BOS_ID).collect();
    let eligible = (1..tokens.len()).filter(|&p| tokens[p] != BOS_ID && mask[p] == 1);
    let mut rows = Vec::new();
    for (ordinal, p) in eligible.enumerate() {
        let doc = documents
            .partition_point(|&d| d <= p)
            .checked_sub(1)
            .ok_or_else(|| format!("row {ordinal} at {p} precedes the first BOS"))?;
        let start = p.saturating_sub(DEV_CONTEXT).max(documents[doc]);
        let (query, target) = (tokens[p - 1], tokens[p]);
        let most_recent = (start..p - 1).rev().find(|&j| tokens[j] == query);
        let copy_successor = most_recent.is_some_and(|j| tokens[j + 1] == target);
        let bucket = most_recent
            .filter(|_| copy_successor)
            .map(|j| match (p - 1) - j {
                0..=8 => 0,
                9..=32 => 1,
                33..=96 => 2,
                _ => 3,
            });
        rows.push(PanelRow {
            ordinal,
            position: p,
            document_id: doc,
            query_token: query,
            target_token: target,
            copy_successor,
            bucket,
        });
    }
    Ok(rows)
}

/// The sealed binary ordered-row binding: per row in order, ordinal and
/// position as u64 LE, query and target as u32 LE, the copy-successor byte and
/// the bucket byte (`bucket + 1`, or 0).
pub fn ordered_row_binding_sha256(rows: &[PanelRow]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    for r in rows {
        h.update((r.ordinal as u64).to_le_bytes());
        h.update((r.position as u64).to_le_bytes());
        h.update(u32::from(r.query_token).to_le_bytes());
        h.update(u32::from(r.target_token).to_le_bytes());
        h.update([u8::from(r.copy_successor)]);
        h.update([r.bucket.map(|b| b + 1).unwrap_or(0)]);
    }
    format!("{:x}", h.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn w2_panel_buckets_the_most_recent_copy() {
        // doc 0: BOS 5 6 5 6 | doc 1: BOS 5 7
        let tokens = [0u16, 5, 6, 5, 6, 0, 5, 7];
        let rows = dev_panel(&tokens, &[1; 8]).unwrap();
        assert_eq!(rows.len(), 6);
        assert_eq!(rows[3].position, 4);
        assert!(rows[3].copy_successor);
        assert_eq!(rows[3].bucket, Some(0));
        assert_eq!((rows[5].document_id, rows[5].copy_successor), (1, false));
    }

    /// Gated on `UOR_W2_Q4` (the W2 input folder); skipped when unset.
    #[test]
    fn w2_row_binding_matches_the_sealed_panel() {
        let Ok(root) = std::env::var("UOR_W2_Q4") else {
            eprintln!("UOR_W2_Q4 unset: row-binding check skipped");
            return;
        };
        let dir = std::path::Path::new(&root).join("driver-data/ft/dev");
        let tokens =
            parse_uort_tokens(&std::fs::read(dir.join("tokens.u16")).expect("tokens")).unwrap();
        let mask = std::fs::read(dir.join("response_mask.u8")).expect("mask");
        let rows = dev_panel(&tokens, &mask).unwrap();
        let sha = ordered_row_binding_sha256(&rows);
        eprintln!("rows {} binding {sha}", rows.len());
        assert_eq!(rows.len(), DEV_ROWS);
        assert_eq!(
            sha,
            "e4ef44fb81d6c3a833212e4857117db71cafb8259b81c286e5b1391129d74446"
        );
    }

    const W: u64 = 149;

    #[test]
    fn w2_symmetry_actions_keep_every_root_a_root() {
        assert_eq!(geometry::AXES.len() / BLOCK_WIDTH, 64);
        assert_eq!(symmetry_action_non_roots(), 0);
    }

    const NO: u16 = crate::w2_capture::ABSENT_SLOT;

    fn rows(spec: &[(u16, u16, u64)]) -> Vec<TrainRow> {
        let mut v = Vec::new();
        for &(code, target, n) in spec {
            for _ in 0..n {
                v.push(TrainRow {
                    code,
                    query_token: 1,
                    target_token: target,
                });
            }
        }
        v
    }

    fn cells(spec: &[(u16, u16)]) -> Vec<u16> {
        let mut c = vec![NO; VOCABULARY];
        for &(token, cell) in spec {
            c[usize::from(token)] = cell;
        }
        c
    }

    #[test]
    fn w2_admitted_prediction_differs_from_unrestricted() {
        let t = build_tables(&rows(&[(1, 10, 3), (1, 20, 1)])).unwrap();
        let token_cells = cells(&[(10, 5), (20, 6)]);
        let slots = [6, NO];
        let adm = Admission {
            token_cells: &token_cells,
            slots: &slots,
        };
        let full = predict(&t.code, 1, &t.unigram);
        let restricted = predict_admitted(&t.code, 1, &t.unigram.ranked_all(), &adm);
        assert_eq!(full.top1(), Some(10));
        assert_eq!(restricted.top1(), Some(20));
        assert_eq!(restricted.top8, vec![20]);
        assert!(!restricted.fallback_used);
    }

    #[test]
    fn w2_admitted_ties_break_to_lowest_token() {
        let t = build_tables(&rows(&[(1, 30, 2), (1, 25, 2), (1, 10, 5)])).unwrap();
        let token_cells = cells(&[(30, 1), (25, 2), (10, 3)]);
        let slots = [1, 2];
        let adm = Admission {
            token_cells: &token_cells,
            slots: &slots,
        };
        let restricted = predict_admitted(&t.code, 1, &t.unigram.ranked_all(), &adm);
        assert_eq!(restricted.top8, vec![25, 30]);
    }

    #[test]
    fn w2_admitted_falls_back_to_the_admitted_unigram() {
        let t = build_tables(&rows(&[(2, 10, 4), (3, 40, 1), (3, 41, 2)])).unwrap();
        let token_cells = cells(&[(10, 3), (40, 7), (41, 8)]);
        let slots = [7, NO, NO];
        let adm = Admission {
            token_cells: &token_cells,
            slots: &slots,
        };
        let p = predict_admitted(&t.code, 2, &t.unigram.ranked_all(), &adm);
        assert!(p.fallback_used);
        assert_eq!(p.top8, vec![40]);
        let none = Admission {
            token_cells: &token_cells,
            slots: &[NO],
        };
        let q = predict_admitted(&t.code, 2, &t.unigram.ranked_all(), &none);
        assert_eq!(q.top1(), None);
    }

    #[test]
    fn w2_admitted_hit1_is_bounded_by_target_admitted_rows() {
        let train = rows(&[(1, 10, 3), (1, 20, 1), (2, 30, 2), (2, 10, 1)]);
        let t = build_tables(&train).unwrap();
        let token_cells = cells(&[(10, 1), (20, 2), (30, 3)]);
        let dev_row = |code, target, w| DevRow {
            code,
            query_token: 1,
            target_token: target,
            writer_hit1: w,
            writer_hit8: w,
        };
        let dev = [
            dev_row(1, 10, true),
            dev_row(1, 20, true),
            dev_row(2, 30, false),
            dev_row(2, 10, true),
        ];
        let slots = vec![vec![2], vec![2], vec![1, 3], vec![3]];
        let actions = symmetry_root_actions().unwrap();
        let keys = symmetry_keys(&actions);
        let sym = build_sym_table(&train, &keys);
        let scores = score_admitted(&t, &sym, &keys, &dev, &token_cells, &slots);
        let m = admission_matched(&scores, &dev);
        assert_eq!((m.rows, m.target_admitted_rows), (4, 2));
        for k in 0..ADMITTED_ARMS.len() {
            assert!(m.all[k].hit1 <= m.target_admitted_rows);
            assert_eq!(m.all[k], m.target_admitted[k]);
        }
        // Row 0's target 10 is not admitted; row 1's 20 is, and the restricted table finds it.
        assert_eq!(scores[1].table.top1(), Some(20));
        assert_eq!(m.all[4].hit1, 1);
    }

    #[test]
    fn w2_symmetry_key_is_constant_on_an_orbit() {
        let actions = symmetry_root_actions().unwrap();
        assert_eq!(actions.len(), 64);
        let identity: [u8; ROOT_COUNT] = core::array::from_fn(|r| r as u8);
        assert!(actions.contains(&identity));
        let keys = symmetry_keys(&actions);
        for code in [pack_code([1, 5, 9, 14]), pack_code([0, 0, 3, 15])] {
            let roots = unpack_code(code);
            assert!(keys[usize::from(code)] <= code);
            for act in &actions {
                let image = pack_code(roots.map(|r| act[usize::from(r)]));
                assert_eq!(keys[usize::from(image)], keys[usize::from(code)]);
            }
        }
    }

    #[test]
    fn w2_symmetry_actions_are_closed_under_composition() {
        let actions = symmetry_root_actions().unwrap();
        for act_a in &actions {
            for act_b in &actions {
                let composed: [u8; ROOT_COUNT] =
                    core::array::from_fn(|r| act_a[usize::from(act_b[r])]);
                assert!(actions.contains(&composed));
            }
        }
    }

    #[test]
    fn every_root_round_trips_and_absent_is_an_error() {
        for r in 0..ROOT_COUNT as u8 {
            let roots = [r, (r + 1) % 16, (r + 7) % 16, 15 - r];
            let q = root_vector(roots);
            assert_eq!(decode_read_vector(&q), Ok(pack_code(roots)));
            assert_eq!(unpack_code(pack_code(roots)), roots);
        }
        let zero = [0i32; BLOCK_WIDTH];
        assert_eq!(
            decode_block(&zero),
            Err(CapacityError::AbsentRoot { block: zero })
        );
        let mut q = root_vector([1, 2, 3, 4]);
        q[17] += 1; // corrupt block 2
        assert!(matches!(
            decode_read_vector(&q),
            Err(CapacityError::AbsentRoot { .. })
        ));
    }

    #[test]
    fn code_nibble_order_and_secondary_keys() {
        let code = pack_code([0x1, 0x2, 0x3, 0x4]);
        assert_eq!(code, 0x4321);
        assert_eq!(address_key(code), 0x41);
        assert_eq!(interior_key(code), 0x32);
    }

    fn tr(code: u16, q: u16, t: u16) -> TrainRow {
        TrainRow {
            code,
            query_token: q,
            target_token: t,
        }
    }

    #[test]
    fn tie_rule_is_highest_count_then_lowest_token() {
        // code 7: tokens 9 and 5 tie at 2; token 3 has 1.
        let train = [
            tr(7, 0, 9),
            tr(7, 0, 9),
            tr(7, 0, 5),
            tr(7, 0, 5),
            tr(7, 0, 3),
        ];
        let t = build_tables(&train).unwrap();
        let p = predict(&t.code, 7, &t.unigram);
        assert_eq!(p.top8, vec![5, 9, 3]);
        assert!(!p.fallback_used);
    }

    #[test]
    fn unseen_key_falls_back_to_matched_unigram() {
        let train = [tr(1, 10, 40), tr(2, 11, 40), tr(2, 11, 41)];
        let t = build_tables(&train).unwrap();
        let p = predict(&t.code, 99, &t.unigram);
        assert!(p.fallback_used);
        assert_eq!(p.top8, vec![40, 41]);
        let g = predict(&t.bigram, 12, &t.unigram);
        assert!(g.fallback_used);
        assert_eq!(g.top1(), Some(40));
        let dev = [DevRow {
            code: 99,
            query_token: 11,
            target_token: 41,
            writer_hit1: false,
            writer_hit8: false,
        }];
        let s = score_dev(&t, &dev).unwrap();
        assert!(!s[0].seen_code && s[0].table.fallback_used);
        assert!(!s[0].bigram.fallback_used && s[0].bigram.top1() == Some(40));
    }

    #[test]
    fn counts_conserve_and_tokens_are_bounded() {
        let train: Vec<TrainRow> = (0..1000u16)
            .map(|i| tr(i % 37, i % 101, (i * 7) % 4096))
            .collect();
        let t = build_tables(&train).unwrap();
        for table in [&t.code, &t.bigram, &t.address_only, &t.interior_only] {
            assert_eq!(table.cell_sum(), train.len() as u64);
            assert_eq!(table.total(), train.len() as u64);
            assert_eq!(
                table
                    .count_histogram()
                    .iter()
                    .map(|(k, v)| k * v)
                    .sum::<u64>(),
                1000
            );
        }
        assert_eq!(t.unigram.total(), 1000);
        assert_eq!(
            build_tables(&[tr(0, 0, 4096)]),
            Err(CapacityError::TokenOutOfRange { token: 4096 })
        );
    }

    #[test]
    fn aggregates_count_paired_discordance() {
        // code 1 -> 50, code 2 -> 60; bigram query 7 -> 50 (3 vs 2); unigram top-1 = 50.
        let mut train = vec![tr(1, 7, 50); 3];
        train.push(tr(2, 7, 60));
        train.push(tr(3, 7, 60));
        let t = build_tables(&train).unwrap();
        let dev = [
            DevRow {
                code: 1,
                query_token: 7,
                target_token: 50,
                writer_hit1: true,
                writer_hit8: true,
            },
            DevRow {
                code: 2,
                query_token: 7,
                target_token: 60,
                writer_hit1: false,
                writer_hit8: true,
            },
            DevRow {
                code: 1,
                query_token: 7,
                target_token: 60,
                writer_hit1: true,
                writer_hit8: true,
            },
        ];
        let a = aggregate(&t, &score_dev(&t, &dev).unwrap());
        assert_eq!((a.t_top1, a.u_top1, a.g_top1, a.w_top1), (2, 1, 1, 2));
        assert_eq!(a.d_u, Discordance { b: 0, c: 1 });
        assert_eq!(a.d_g, Discordance { b: 0, c: 1 });
        assert_eq!(a.d_w, Discordance { b: 1, c: 1 });
        assert_eq!(
            (
                a.coverage_rows,
                a.codes_with_count_ge_2,
                a.matched_train_total
            ),
            (3, 1, 5)
        );
    }

    /// Normal ordering, rational floors: n ≤ 144 so F = H = 24.
    /// M + F = 1024, W + H = 173, W − H = 125.
    #[test]
    fn band_boundaries_are_strict_as_registered() {
        let band = |t| {
            let (_, band, _) = decide_band(t, 1000, W, 100, 100);
            band
        };
        assert_eq!(band(1025), Band::HLaw);
        assert_eq!(band(1024), Band::HMixed, "T = M + F is never H_law");
        assert_eq!(band(174), Band::HMixed);
        assert_eq!(band(173), Band::HMixed, "T = W + H is H_mixed");
        assert_eq!(band(172), Band::HRep);
        assert_eq!(band(149), Band::HRep);
        assert_eq!(band(126), Band::HRep);
        assert_eq!(band(125), Band::HRep, "T = W - H is H_rep");
        assert_eq!(band(124), Band::WriterDominates);
        assert_eq!(band(0), Band::WriterDominates);
    }

    /// Irrational floors: F = 2·sqrt(401) ≈ 40.05, H = 2·sqrt(401).
    #[test]
    fn band_boundaries_with_paired_sqrt_floors() {
        let band = |t| {
            let (_, band, _) = decide_band(t, 1000, W, 401, 401);
            band
        };
        assert_eq!(band(1041), Band::HLaw);
        assert_eq!(band(1040), Band::HMixed);
        assert_eq!(band(190), Band::HMixed);
        assert_eq!(band(189), Band::HRep);
        assert_eq!(band(109), Band::HRep);
        assert_eq!(band(108), Band::WriterDominates);
        // F = 40 exactly (n = 400): T = M + 40 is not H_law.
        let (_, mixed, _) = decide_band(1040, 1000, W, 400, 100);
        assert_eq!(mixed, Band::HMixed);
        let (_, law, _) = decide_band(1041, 1000, W, 400, 100);
        assert_eq!(law, Band::HLaw);
    }

    #[test]
    fn empty_mixed_interval_and_pathological_guard() {
        // M + F = 134 < W + H = 173: no H_mixed; T = 150 > M + F is H_law.
        assert_eq!(decide_band(150, 110, W, 100, 100), (2, Band::HLaw, false));
        // M + F = 74 < W − H = 125: pathological. T = 100 > M + F but T < W − H → H_rep.
        assert_eq!(decide_band(100, 50, W, 100, 100), (4, Band::HRep, true));
        assert_eq!(decide_band(124, 50, W, 100, 100), (4, Band::HRep, true));
        assert_eq!(decide_band(125, 50, W, 100, 100), (2, Band::HLaw, true));
        // Guard boundary: F + H = 48 against W − M.
        assert!(!floor_sum_below(100, 100, 48));
        assert!(floor_sum_below(100, 100, 49));
        // 2√401 + 2√401 = 4√401 ≈ 80.1
        assert!(!floor_sum_below(401, 401, 80));
        assert!(floor_sum_below(401, 401, 81));
        // 24 + 2√401 ≈ 64.05
        assert!(!floor_sum_below(100, 401, 64));
        assert!(floor_sum_below(401, 100, 65));
    }

    fn powered() -> Aggregates {
        Aggregates {
            dev_rows: DEV_ROWS as u64,
            t_top1: 1100,
            u_top1: 1000,
            g_top1: 900,
            w_top1: W,
            coverage_rows: 12_859,
            codes_with_count_ge_2: 1024,
            d_u: Discordance { b: 50, c: 50 },
            d_g: Discordance { b: 60, c: 60 },
            d_w: Discordance { b: 40, c: 60 },
            ..Aggregates::default()
        }
    }

    #[test]
    fn underpowered_takes_precedence() {
        let a = powered();
        assert_eq!(decide(&a).band, Band::HLaw);
        let mut c = a.clone();
        c.coverage_rows = 12_858;
        assert_eq!(coverage_bar(DEV_ROWS as u64), 12_859);
        assert_eq!(
            (decide(&c).band, decide(&c).precedence),
            (Band::Underpowered, 1)
        );
        let mut m = a.clone();
        m.codes_with_count_ge_2 = 1023;
        assert_eq!(decide(&m).band, Band::Underpowered);
        // b+c = 99, balanced: exact test cannot reject.
        let mut p = a.clone();
        p.d_g = Discordance { b: 49, c: 50 };
        assert_eq!(decide(&p).band, Band::Underpowered);
        // b+c = 20, lopsided: exact test rejects, so evaluable.
        let mut q = a.clone();
        q.d_u = Discordance { b: 0, c: 20 };
        assert!(underpowered_reasons(&q).is_empty());
    }

    #[test]
    fn exact_mcnemar_matches_known_cutoffs() {
        // n = 20: two-sided p for k = 5 is 0.0414, for k = 6 is 0.1153.
        assert!(exact_mcnemar_rejects_005(Discordance { b: 5, c: 15 }));
        assert!(!exact_mcnemar_rejects_005(Discordance { b: 6, c: 14 }));
        assert!(!exact_mcnemar_rejects_005(Discordance { b: 0, c: 0 }));
        assert!(!exact_mcnemar_rejects_005(Discordance { b: 0, c: 5 }));
        assert!(exact_mcnemar_rejects_005(Discordance { b: 0, c: 6 }));
    }

    #[test]
    #[should_panic(expected = "exact test is only applied below")]
    fn exact_mcnemar_refuses_the_powered_range() {
        // b+c = 100 is at the power bar, where the u128 binomial is no longer used.
        exact_mcnemar_rejects_005(Discordance { b: 50, c: 50 });
    }
}
