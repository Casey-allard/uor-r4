//! claude-146's induction-keyed Hamming reads in the integer artifact (claude-188): their
//! parameters as a `uor-r4.lut-stack/6` container stores them, one read per depth.
//!
//! The FBS session serves the whole read ([`ServedHamming`]): the copy heads (claude-194) and the
//! type heads (claude-195), at the stream read after its depth and the move on the readout. The
//! type heads keep each position's payload in the session, which a saved session does not carry,
//! so `restore_state` refuses a session with them rather than continue differently.

use crate::bitcode::popcount_swar;
use crate::fbs_snapshot::{CodeRole, ForcedCode, HammingTraceHead, HammingTraceRead, HammingTraceWindow};

use super::format::{Container, StackHamming, HAMMING_WEIGHT_EXP};
use super::kernels::{stack_div_u64, stack_hamilton, NORM_FINE_EXP, RESIDUAL_EXP};
use super::session::{stack_rotation, GATE_EPSILON};
use super::StackError;

/// One Hamming read's integers, at the exponents its [`StackHamming`] record names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HammingReadImage {
    /// After this many layers, or on the readout at the layer count.
    pub depth: usize,
    /// `width / 4` raw quaternions `[w, x, y, z]` at `2^rotation_exp`: the type heads' query.
    pub query_rotation: Vec<i32>,
    /// `width / 4` raw quaternions: the key, which the copy heads' queries share.
    pub key_rotation: Vec<i32>,
    /// `width / 4` raw quaternions: the type heads' value.
    pub value_rotation: Vec<i32>,
    /// `heads` margins in bits at `2^margin_exp`, each positive.
    pub margin: Vec<i32>,
    /// `2 * width` at `2^weight_exp`: row 0 the copy heads' weights, row 1 the type heads'.
    pub weight: Vec<i32>,
    /// The read's ask heads and, on the readout read, its question carry (claude-336), or `None`
    /// for a read whose type heads ask from the hook's input.
    pub ask: Option<AskImage>,
}

/// A stack's Hamming reads: the header record and one image per depth, in depth order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HammingImage {
    pub record: StackHamming,
    pub reads: Vec<HammingReadImage>,
}

/// The Hamming reads of a `UORLUT01` stack artifact, with every section at the length its shape
/// and record imply, or `None` for a stack without them. A record with a copy key per head
/// (`copy_keys`) is returned as it is: each caller reads every copy head by
/// [`StackHamming::head_copy_key`] (claude-269).
pub fn read_hamming_uorlut01(bytes: &[u8]) -> Result<Option<HammingImage>, StackError> {
    hamming_image(&Container::parse(bytes)?)
}

/// The Hamming reads of a parsed container, as [`read_hamming_uorlut01`].
pub(crate) fn hamming_image(artifact: &Container<'_>) -> Result<Option<HammingImage>, StackError> {
    let Some(record) = artifact.hamming.clone() else {
        return Ok(None);
    };
    let width = artifact.shape.width;
    let mut reads = Vec::with_capacity(record.depths.len());
    for &depth in &record.depths {
        let section = |part: &str, len: usize| -> Result<Vec<i32>, StackError> {
            let name = format!("hamming.d{depth}.{part}");
            let values = artifact.table_i32(&name)?;
            if values.len() != len {
                return Err(StackError::TableLength {
                    name,
                    len: values.len(),
                    expected: len,
                });
            }
            Ok(values)
        };
        let margin = section("margin", record.heads)?;
        if margin.iter().any(|&m| m <= 0) {
            return Err(StackError::Numerics(format!(
                "hamming.d{depth}.margin: a margin is not positive"
            )));
        }
        // claude-336: a read's ask sections are present exactly when its record says it asks, and
        // its carry exactly when it is the readout read with `carry_rows`.
        let r = reads.len();
        fn present<T>(found: Result<Vec<T>, StackError>, name: String, wanted: bool, why: &str) -> Result<Option<Vec<T>>, StackError> {
            match found {
                Ok(values) if wanted => Ok(Some(values)),
                Ok(_) => Err(StackError::Numerics(format!("{name}: present, but {why}"))),
                Err(StackError::MissingSection(_)) if !wanted => Ok(None),
                Err(e) => Err(e),
            }
        }
        let ask_name = format!("hamming.d{depth}.ask");
        let carry_name = format!("hamming.d{depth}.carry");
        let ask_table = present(artifact.table_i16(&ask_name), ask_name, record.read_asks(r), "the record says the read does not ask")?;
        let carry_gains = present(artifact.table_i32(&carry_name), carry_name, record.read_carry(r).is_some(), "the record names no carry at this read")?;
        let carry = record.read_carry(r).zip(carry_gains);
        reads.push(HammingReadImage {
            depth,
            query_rotation: section("query_rotation", width)?,
            key_rotation: section("key_rotation", width)?,
            value_rotation: section("value_rotation", width)?,
            margin,
            weight: section("weight", 2 * width)?,
            ask: AskImage::checked(ask_table, carry, artifact.shape.vocab, width)?,
        });
    }
    Ok(Some(HammingImage { record, reads }))
}

/// What the copy-code check (claude-237) takes from a stack artifact: its Hamming reads, its Q12
/// writer table (`vocab` rows of `width`), and each read's copy-head codes as the session serves
/// them ([`CopyHeads`]): entry `v`'s code for copy head `h` at `v * copy + h`. Under copy key 2
/// (claude-261) these are the codes each query and key joins two of, by halves.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServedCopyCodes {
    pub image: HammingImage,
    pub table: Vec<i16>,
    pub vocab: usize,
    pub width: usize,
    /// One list per read, in depth order.
    pub codes: Vec<Vec<u64>>,
}

/// The [`ServedCopyCodes`] of a `UORLUT01` stack artifact, or `None` for a stack without Hamming
/// reads. The session's view of the trunk (`fbs_image`) leaves out an artifact with a read, so
/// this reads the writer table itself, at the length the session checks. The codes do not depend
/// on a head's copy key, which only pairs them.
pub fn served_copy_codes_uorlut01(bytes: &[u8]) -> Result<Option<ServedCopyCodes>, StackError> {
    let artifact = Container::parse(bytes)?;
    let Some(image) = hamming_image(&artifact)? else {
        return Ok(None);
    };
    let (vocab, width) = (artifact.shape.vocab, artifact.shape.width);
    let table = artifact.table_i16("writer.table")?;
    if table.len() != vocab * width {
        return Err(StackError::TableLength {
            name: "writer.table".to_owned(),
            len: table.len(),
            expected: vocab * width,
        });
    }
    let copy = image.record.heads / 2;
    let codes = image
        .reads
        .iter()
        .map(|read| {
            let heads = CopyHeads::new(&image.record, read, &table, vocab, width);
            (0..vocab as u32)
                .flat_map(|v| (0..copy).map(move |h| (v, h)))
                .map(|(v, h)| heads.code(v, h))
                .collect()
        })
        .collect();
    Ok(Some(ServedCopyCodes { image, table, vocab, width, codes }))
}

/// The exponent of a copy mean: a Q12 table point times a Q31 weight.
pub const COPY_MEAN_EXP: i32 = -43;

/// Which earlier positions a copy head reads (claude-345). The artifact's own rule is
/// [`CopyRule::Mean`], the rule the float read trains with; the others are served only when a
/// caller sets them ([`CopyHeads::set_rule`]), to read one artifact under another rule.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CopyRule {
    /// The mean over the `kept` nearest positions inside the margin, ties to the earlier position.
    #[default]
    Mean,
    /// The latest position whose key equals the query exactly; a head with none reads nothing.
    LatestExact,
    /// The latest of the positions at the smallest distance, when that distance is inside the
    /// margin: hard top-1 (the 26 Sep retrieval head), with ties to the later position.
    LatestNearest,
}

impl CopyRule {
    /// The rule `name` names (`mean`, `latest-exact`, `latest-nearest`), or `None`.
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "mean" => Some(Self::Mean),
            "latest-exact" => Some(Self::LatestExact),
            "latest-nearest" => Some(Self::LatestNearest),
            _ => None,
        }
    }

    /// The name [`Self::parse`] reads back.
    pub fn name(self) -> &'static str {
        match self {
            Self::Mean => "mean",
            Self::LatestExact => "latest-exact",
            Self::LatestNearest => "latest-nearest",
        }
    }
}

/// The weight of a candidate at `distance` bits under a head's margin `margin_q16` (bits at
/// Q16), at Q31: `max(0, m − d) / max(m, 1)`, as the float read's
/// `margin.broadcast_sub(bits).relu().broadcast_div(margin.maximum(1.0))` (claude-189).
pub fn copy_weight_q31(margin_q16: i32, distance: u32) -> u64 {
    let margin = i64::from(margin_q16);
    let over = margin - (i64::from(distance) << 16);
    if over <= 0 {
        return 0;
    }
    // `over <= margin < 2^31`, so the shifted numerator stays below 2^62.
    stack_div_u64((over as u64) << 31, margin.max(1 << 16) as u64)
}

/// One read's copy heads, built for serving (claude-189): every vocabulary entry's sign code
/// for each copy head, and the copy heads' margins.
///
/// A copy head's query at `t` is the code of `x_t`'s table point through the key rotation, and
/// its key at `s` is the code of `x_{s−1}`'s point through the same rotation, so every code is
/// a vocabulary entry's. The rotation is the unit `g / |g|` of each raw key quaternion, as
/// `stack_rotation` normalizes the unsnapped gates, applied by `stack_hamilton` on the left,
/// as the float `hamilton_gate`. A bit is 1 at or above 0, 64 coordinates to a word in
/// coordinate order, as the float `pack_signs`.
///
/// Under copy key 2 (claude-261), each code joins two vocabulary entries' codes: the query is
/// `x_t`'s first half and `x_{t−1}`'s second, and the key `x_{s−1}`'s first half and `x_{s−2}`'s
/// second, as the float read's `pair_halves` joins the rotated points before their signs.
pub struct CopyHeads {
    copy: usize,
    kept: usize,
    width: usize,
    /// `vocab * copy` codes: entry `v`'s code for copy head `h` is at `v * copy + h`.
    codes: Vec<u64>,
    /// The copy heads' margins, in bits at Q16.
    margins: Vec<i32>,
    /// Per copy head: under copy key 2, the first half's bits, the low `head_width / 2`; `None`
    /// under copy key 1 (claude-268: each head by its own key).
    pair: Vec<Option<u64>>,
    /// Which positions [`Self::mean`] reads; [`CopyRule::Mean`] unless a caller sets another.
    rule: CopyRule,
}

impl CopyHeads {
    /// The copy heads of `read`, a read of a stack with `record`, over the Q12 `table` of
    /// `vocab` rows of `width`.
    pub fn new(record: &StackHamming, read: &HammingReadImage, table: &[i16], vocab: usize, width: usize) -> Self {
        let copy = record.heads / 2;
        Self {
            copy,
            kept: record.kept,
            width,
            codes: vocab_codes(&units(&read.key_rotation), table, vocab, width, record.heads, 0..copy),
            margins: read.margin[..copy].to_vec(),
            pair: (0..copy)
                .map(|h| {
                    (record.head_copy_key(h) == 2).then(|| (1u64 << (width / record.heads / 2)) - 1)
                })
                .collect(),
            // The artifact's own rule (claude-345): latest-exact when its record says so.
            rule: if record.copy_latest { CopyRule::LatestExact } else { CopyRule::Mean },
        }
    }

    /// Read under `rule` from here on: [`Self::mean`], and with it the forced replay and the
    /// trace, which pick their positions by the rule in force.
    pub fn set_rule(&mut self, rule: CopyRule) {
        self.rule = rule;
    }

    /// The rule [`Self::mean`] reads under.
    pub fn rule(&self) -> CopyRule {
        self.rule
    }

    /// Copy head `h`'s one position at the last row `t` of `tokens` under a latest rule: over
    /// every `1 ≤ s < t` (not only the `kept` nearest, whose ties go to the earlier position), the
    /// latest `s` at distance 0 when `exact`, else the latest `s` at the smallest distance. Empty
    /// when there is no such position.
    fn latest(&self, h: usize, tokens: &[u32], exact: bool) -> Vec<(usize, u32)> {
        let Some(t) = tokens.len().checked_sub(1) else {
            return Vec::new();
        };
        latest_of(self.query_at(tokens, t, h), t, exact, |s| self.key_at(tokens, s, h))
    }

    /// The positions [`Self::mean`] reads for head `h`, by the head's [`CopyRule`].
    fn choose_ruled(&self, h: usize, tokens: &[u32]) -> Vec<(usize, u32)> {
        match self.rule {
            CopyRule::Mean => self.choose(h, tokens),
            CopyRule::LatestExact => self.latest(h, tokens, true),
            CopyRule::LatestNearest => self.latest(h, tokens, false),
        }
    }

    /// Copy head `h`'s code for vocabulary entry `token`.
    pub fn code(&self, token: u32, h: usize) -> u64 {
        self.codes[token as usize * self.copy + h]
    }

    /// Head `h`'s code of `near`, or, when head `h` is on copy key 2, `near`'s first half joined to
    /// `far`'s second.
    fn joined(&self, near: u32, far: u32, h: usize) -> u64 {
        let code = self.code(near, h);
        match self.pair[h] {
            None => code,
            Some(first) => (code & first) | (self.code(far, h) & !first),
        }
    }

    /// Copy head `h`'s query at row `t` of `tokens`: `x_t`'s code, joined under copy key 2 to
    /// `x_{t−1}`'s (`x_0`'s own at row 0, as the float read's previous token there).
    fn query_at(&self, tokens: &[u32], t: usize, h: usize) -> u64 {
        self.joined(tokens[t], tokens[t.saturating_sub(1)], h)
    }

    /// Copy head `h`'s key at position `1 ≤ s` of `tokens`: `x_{s−1}`'s code, joined under copy
    /// key 2 to `x_{s−2}`'s (`x_0`'s at `s = 1`, as the float read's `two_back`).
    fn key_at(&self, tokens: &[u32], s: usize, h: usize) -> u64 {
        self.joined(tokens[s - 1], tokens[s.saturating_sub(2)], h)
    }

    /// Copy head `h`'s kept positions at the last row `t` of `tokens` (`x_0..=x_t`): the
    /// `min(kept, t − 1)` positions `1 ≤ s < t` nearest by `popcount(q XOR k_s)`, with their
    /// distances, in ascending distance and then ascending `s` (the design's tie rule; the
    /// float `choose_sorted` leaves the order of ties open). Position 0 has no previous token, and
    /// an empty `tokens` keeps nothing.
    pub fn choose(&self, h: usize, tokens: &[u32]) -> Vec<(usize, u32)> {
        let Some(t) = tokens.len().checked_sub(1) else {
            return Vec::new();
        };
        nearest(self.query_at(tokens, t, h), t, self.kept, |s| self.key_at(tokens, s, h))
    }

    /// The positions [`Self::mean`] reads for head `h`, with the codes `force` names in place of
    /// the computed ones (Q5's forced replay, claude-249): the row's query, and each candidate
    /// `s`'s key. The positions are picked by the head's [`CopyRule`], as the served read picks
    /// them, so a replay that forces nothing is the served read (claude-345).
    fn choose_forced(&self, h: usize, tokens: &[u32], force: &HeadForce) -> Vec<(usize, u32)> {
        let Some(t) = tokens.len().checked_sub(1) else {
            return Vec::new();
        };
        let query = force.query(t).unwrap_or_else(|| self.query_at(tokens, t, h));
        let key = |s: usize| force.key(s).unwrap_or_else(|| self.key_at(tokens, s, h));
        match self.rule {
            CopyRule::Mean => nearest(query, t, self.kept, key),
            CopyRule::LatestExact => latest_of(query, t, true, key),
            CopyRule::LatestNearest => latest_of(query, t, false, key),
        }
    }

    /// The copy heads' mean at the last row of `tokens`, at [`COPY_MEAN_EXP`], or `None` when no
    /// copy head kept a candidate inside its margin.
    ///
    /// Each head's mean is `Σ w_s · table[x_s] / max(1, #{w_s > 0})` over its kept positions:
    /// the position after the earlier match brings back what followed it, as the very point the
    /// nearest readout names it by. The copy mean is the mean of the head means over the heads
    /// that found a candidate, so a head that found nothing does not dilute one that did. Both
    /// quotients truncate toward zero.
    pub fn mean(&self, tokens: &[u32], table: &[i16]) -> Option<Vec<i128>> {
        self.mean_by(tokens, table, |h| self.choose_ruled(h, tokens))
    }

    /// [`Self::mean`] over the kept positions `choose` gives each head.
    fn mean_by(&self, tokens: &[u32], table: &[i16], choose: impl Fn(usize) -> Vec<(usize, u32)>) -> Option<Vec<i128>> {
        let width = self.width;
        let mut sum = vec![0i128; width];
        let mut found = 0i128;
        for h in 0..self.copy {
            let mut head = vec![0i128; width];
            let mut count = 0i128;
            for (s, d) in choose(h) {
                let weight = copy_weight_q31(self.margins[h], d);
                if weight == 0 {
                    continue;
                }
                count += 1;
                let row = &table[tokens[s] as usize * width..][..width];
                for (acc, &p) in head.iter_mut().zip(row) {
                    // Each point behind `black_box`: loaded eight at a time into a NEON register,
                    // the points end in SIMD-to-general transfers (`fmov`), which the serving
                    // audit refuses (claude-194).
                    *acc += i128::from(core::hint::black_box(p)) * i128::from(weight);
                }
            }
            if count == 0 {
                continue;
            }
            found += 1;
            for (acc, value) in sum.iter_mut().zip(head) {
                *acc += value / count;
            }
        }
        (found > 0).then(|| sum.into_iter().map(|value| value / found).collect())
    }
}

/// The unit rotations `g / |g|` of a read's raw quaternions at Q31, as `stack_rotation` normalizes
/// the unsnapped gates.
fn units(raw: &[i32]) -> Vec<[i64; 4]> {
    raw.chunks_exact(4)
        .map(|g| stack_rotation([g[0], g[1], g[2], g[3]], 1u64 << 31, GATE_EPSILON))
        .collect()
}

/// Head `h`'s sign code of `point` (`width` coordinates at any exponent) through `units`, applied
/// by `stack_hamilton` on the left as the float `hamilton_gate`: a bit is 1 at or above 0, 64
/// coordinates to a word in coordinate order, as the float `pack_signs`.
fn head_code(units: &[[i64; 4]], point: &[i64], h: usize, head_width: usize) -> u64 {
    let mut word = 0u64;
    for c in (0..head_width).step_by(4) {
        let at = h * head_width + c;
        let rotated = stack_hamilton(units[at / 4], [point[at], point[at + 1], point[at + 2], point[at + 3]]);
        for (i, &value) in rotated.iter().enumerate() {
            if value >= 0 {
                word |= 1u64 << (c + i);
            }
        }
    }
    word
}

/// Every vocabulary entry's code for each head of `heads`, through the key rotation `units`:
/// entry `v`'s code for the `j`-th head of the range is at `v * heads.len() + j`.
fn vocab_codes(units: &[[i64; 4]], table: &[i16], vocab: usize, width: usize, all: usize, heads: core::ops::Range<usize>) -> Vec<u64> {
    let head_width = width / all;
    let mut codes = Vec::with_capacity(vocab * heads.len());
    let mut point = vec![0i64; width];
    for row in table.chunks_exact(width).take(vocab) {
        for (p, &q) in point.iter_mut().zip(row) {
            *p = i64::from(q);
        }
        for h in heads.clone() {
            codes.push(head_code(units, &point, h, head_width));
        }
    }
    codes
}

/// The one position a latest rule reads at row `t`: over every `1 ≤ s < t`, the latest `s` whose
/// key is at distance 0 from `query` when `exact`, else the latest `s` at the smallest distance.
/// Empty when there is no such position.
fn latest_of(query: u64, t: usize, exact: bool, key: impl Fn(usize) -> u64) -> Vec<(usize, u32)> {
    let mut best: Option<(usize, u32)> = None;
    for s in 1..t {
        let d = popcount_swar(query ^ key(s));
        if exact && d != 0 {
            continue;
        }
        if best.is_none_or(|(_, least)| d <= least) {
            best = Some((s, d));
        }
    }
    best.into_iter().collect()
}

/// The `min(kept, t − 1)` positions `1 ≤ s < t` nearest to `query` by `popcount(query XOR key(s))`,
/// with their distances, in ascending distance and then ascending `s` (the design's tie rule).
fn nearest(query: u64, t: usize, kept: usize, key: impl Fn(usize) -> u64) -> Vec<(usize, u32)> {
    let mut candidates: Vec<(u32, usize)> = (1..t)
        // The SWAR popcount, not `count_ones`, which lowers to a NEON `cnt` and `fmov` on arm64
        // (claude-194: the step's disassembly check refuses them).
        .map(|s| (popcount_swar(query ^ key(s)), s))
        .collect();
    candidates.sort_unstable();
    candidates.truncate(kept);
    candidates.into_iter().map(|(d, s)| (s, d)).collect()
}

#[cfg(test)]
mod candidate_range_tests {
    use super::nearest;

    #[test]
    fn candidates_include_the_immediate_predecessor_and_exclude_the_current_row() {
        // Five stream tokens put the current row at 4. Valid payload positions are 1..4:
        // 3 is the immediate predecessor, 0 has no predecessor, and 4 is the current row.
        let kept = nearest(0, 4, 16, |_| 0);
        assert_eq!(kept, vec![(1, 0), (2, 0), (3, 0)]);
    }
}

#[cfg(test)]
mod copy_rule_tests {
    use super::{CopyHeads, CopyRule, HeadForce};

    /// One copy head on copy key 1 over a vocabulary of 4, with 4-bit codes and a 4-bit margin,
    /// and the table whose row `v` is `[10 * v + 1; 4]`.
    fn heads(kept: usize, rule: CopyRule) -> (CopyHeads, Vec<i16>) {
        let heads = CopyHeads {
            copy: 1,
            kept,
            width: 4,
            codes: vec![0b0000, 0b1111, 0b0011, 0b0001],
            margins: vec![4 << 16],
            pair: vec![None],
            rule,
        };
        (heads, (0..4i16).flat_map(|v| [10 * v + 1; 4]).collect())
    }

    /// Row `v` of the table at full weight, and at `weight_q31`.
    fn row(v: i128, weight_q31: i128) -> Option<Vec<i128>> {
        Some(vec![(10 * v + 1) * weight_q31; 4])
    }

    const FULL: i128 = 1 << 31;

    #[test]
    fn the_names_round_trip() {
        for rule in [CopyRule::Mean, CopyRule::LatestExact, CopyRule::LatestNearest] {
            assert_eq!(CopyRule::parse(rule.name()), Some(rule));
        }
        assert_eq!(CopyRule::parse("latest"), None);
        assert_eq!(CopyRule::default(), CopyRule::Mean);
    }

    /// Token 1 was followed by 2 and later by 3. The mean blends both with the near matches; the
    /// latest rules bring back 3, what followed the last copy.
    #[test]
    fn two_exact_matches_blend_under_the_mean_and_the_latest_rules_take_the_last() {
        let tokens = [1, 2, 0, 1, 3, 0, 1];
        let (mean, table) = heads(16, CopyRule::Mean);
        // Exact at s = 1 (row 2) and s = 4 (row 3); 2 bits off at s = 2 and 3 bits off at s = 5
        // (row 0 at a half and a quarter); 4 bits off at s = 3, outside the margin.
        assert_eq!(mean.mean(&tokens, &table), Some(vec![211 * (1 << 27); 4]));
        for rule in [CopyRule::LatestExact, CopyRule::LatestNearest] {
            let (latest, table) = heads(16, rule);
            assert_eq!(latest.mean(&tokens, &table), row(3, FULL), "{}", rule.name());
        }
    }

    /// With no exact match the exact rule reads nothing, and the nearest rule takes the later of
    /// the two positions at the smallest distance, at that distance's weight.
    #[test]
    fn without_an_exact_match_only_the_nearest_rule_reads() {
        let tokens = [2, 0, 2, 3, 1];
        let (mean, table) = heads(16, CopyRule::Mean);
        assert_eq!(mean.mean(&tokens, &table), Some(vec![16 * (1 << 30); 4]));
        let (exact, _) = heads(16, CopyRule::LatestExact);
        assert_eq!(exact.mean(&tokens, &table), None);
        let (nearest, _) = heads(16, CopyRule::LatestNearest);
        assert_eq!(nearest.mean(&tokens, &table), row(3, 1 << 30));
    }

    /// With more exact matches than the read keeps, the kept ones are the earliest, so the mean
    /// never sees the last; the latest rules scan every earlier position.
    #[test]
    fn the_latest_match_is_found_past_the_kept_count() {
        let tokens = [1, 0, 1, 2, 1, 3, 1];
        let (mean, table) = heads(2, CopyRule::Mean);
        assert_eq!(mean.choose(0, &tokens), vec![(1, 0), (3, 0)]);
        assert_eq!(mean.mean(&tokens, &table), row(1, FULL));
        for rule in [CopyRule::LatestExact, CopyRule::LatestNearest] {
            let (latest, table) = heads(2, rule);
            assert_eq!(latest.mean(&tokens, &table), row(3, FULL), "{}", rule.name());
        }
    }

    /// A replay that forces no code is the served read under every rule, and a forced query is
    /// read by the rule in force: under the exact rule a query no key equals reads nothing.
    #[test]
    fn a_forced_replay_picks_its_positions_by_the_rule_in_force() {
        let tokens = [1, 2, 0, 1, 3, 0, 1];
        for rule in [CopyRule::Mean, CopyRule::LatestExact, CopyRule::LatestNearest] {
            let (heads, table) = heads(16, rule);
            let free = HeadForce::default();
            let replay = heads.mean_by(&tokens, &table, |h| heads.choose_forced(h, &tokens, &free));
            assert_eq!(replay, heads.mean(&tokens, &table), "{}", rule.name());
        }
        let at_row_6 = |query: u64| HeadForce { query: [vec![None; 6], vec![Some(query)]].concat(), key: Vec::new() };
        let (exact, table) = heads(16, CopyRule::LatestExact);
        let (near, _) = heads(16, CopyRule::LatestNearest);
        // The keys at s = 1..=5 are 0b1111, 0b0011, 0b0000, 0b1111 and 0b0001. A query forced to
        // 0b0110 is 2 bits from the first four and 3 from the last: the exact rule reads nothing,
        // the nearest rule the latest at 2 bits, s = 4 (row 3), at that distance's half weight.
        let off = at_row_6(0b0110);
        assert_eq!(exact.mean_by(&tokens, &table, |h| exact.choose_forced(h, &tokens, &off)), None);
        assert_eq!(near.mean_by(&tokens, &table, |h| near.choose_forced(h, &tokens, &off)), row(3, 1 << 30));
        // A query forced to 0b0001 equals only the key at s = 5 (row 0), not the unforced s = 4.
        let moved = at_row_6(0b0001);
        assert_eq!(exact.mean_by(&tokens, &table, |h| exact.choose_forced(h, &tokens, &moved)), row(0, FULL));
    }

    #[test]
    fn a_row_with_no_earlier_position_reads_nothing() {
        for rule in [CopyRule::LatestExact, CopyRule::LatestNearest] {
            let (latest, table) = heads(16, rule);
            assert_eq!(latest.mean(&[1], &table), None);
            assert_eq!(latest.mean(&[1, 1], &table), None);
            assert_eq!(latest.mean(&[], &table), None);
        }
    }
}

/// The exponent of a carry gain `c`: Q16.
pub const ASK_CARRY_GAIN_EXP: i32 = crate::fbs_snapshot::HAMMING_CARRY_GAIN_EXP;

/// The longest carry reach served: with `|table + a| ≤ 2^16` at Q12 and `|c| < 2^31` at Q16, each
/// carried term is below `2^47` at Q28, so `2^12` of them and the row's own point stay below
/// `2^60` and the asked point is exact in `i64`.
pub const ASK_CARRY_MAX: usize = 1 << 12;

/// A read's ask heads (claude-330) and, on the readout read, its question carry (claude-335), as
/// the integer engine serves them (claude-336).
///
/// The question table `a` is stored at the token table's own scale, Q12 in `i16`, so `table + a`
/// is an exact Q12 sum; a gain `c` is Q16 in `i32`, so a carried term `c · (table + a)` is an
/// exact Q28 product, and the asked point is summed at Q28 with nothing rounded before the sign.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AskImage {
    /// `vocab * width` question entries at Q12.
    pub table: Vec<i16>,
    /// The carry's reach `h` (`1 ≤ h ≤ ASK_CARRY_MAX`) and its `vocab` gains at Q16, or `None`.
    pub carry: Option<(usize, Vec<i32>)>,
}

impl AskImage {
    /// The ask heads of a read over `vocab` rows of `width`, refusing a question table or a carry
    /// at the wrong length, a carry with no reach or past [`ASK_CARRY_MAX`], and a carry with no
    /// question table: a gain multiplies `table + a`, which a read without `a` does not ask with.
    pub fn checked(
        table: Option<Vec<i16>>,
        carry: Option<(usize, Vec<i32>)>,
        vocab: usize,
        width: usize,
    ) -> Result<Option<Self>, StackError> {
        let Some(table) = table else {
            return match carry {
                None => Ok(None),
                Some(_) => Err(StackError::Numerics("hamming.ask.carry: a carry gain needs the question table".into())),
            };
        };
        if table.len() != vocab * width {
            return Err(StackError::TableLength { name: "hamming.ask.table".into(), len: table.len(), expected: vocab * width });
        }
        if let Some((reach, gains)) = &carry {
            if *reach == 0 || *reach > ASK_CARRY_MAX {
                return Err(StackError::Numerics(format!("hamming.ask.carry: reach {reach} is outside 1..={ASK_CARRY_MAX}")));
            }
            if gains.len() != vocab {
                return Err(StackError::TableLength { name: "hamming.ask.carry".into(), len: gains.len(), expected: vocab });
            }
        }
        Ok(Some(Self { table, carry }))
    }
}

/// One read's type heads, built for serving (claude-195): the float read's heads `h ≥ heads / 2`.
///
/// A type head asks with the code of the hook's input `p_t` through the query rotation, and its
/// key at `s` is the code of `x_{s−1}`'s table point through the key rotation, as a copy head's
/// is, so the keys are built once per vocabulary entry. What a kept position brings back is the
/// full-width `p_s` through the value rotation: its payload, which the session keeps for every
/// position ([`Self::payload`]).
pub struct TypeHeads {
    copy: usize,
    heads: usize,
    kept: usize,
    width: usize,
    /// `vocab * (heads − copy)` key codes: entry `v`'s for type head `j` at `v * (heads − copy) + j`.
    keys: Vec<u64>,
    /// The type heads' margins, in bits at Q16.
    margins: Vec<i32>,
    query: Vec<[i64; 4]>,
    value: Vec<[i64; 4]>,
    /// The key rotation, through which an ask head asks (claude-336).
    key: Vec<[i64; 4]>,
    /// The ask heads' question table and carry, or `None` for a read that asks from `p_t`.
    ask: Option<AskImage>,
}

impl TypeHeads {
    /// The type heads of `read`, a read of a stack with `record`, over the Q12 `table` of `vocab`
    /// rows of `width`.
    pub fn new(record: &StackHamming, read: &HammingReadImage, table: &[i16], vocab: usize, width: usize) -> Self {
        let (copy, heads) = (record.heads / 2, record.heads);
        Self {
            copy,
            heads,
            kept: record.kept,
            width,
            keys: vocab_codes(&units(&read.key_rotation), table, vocab, width, heads, copy..heads),
            margins: read.margin[copy..].to_vec(),
            query: units(&read.query_rotation),
            value: units(&read.value_rotation),
            key: units(&read.key_rotation),
            ask: None,
        }
    }

    /// These type heads asking with `ask`'s question table and, if it has one, its carry.
    pub fn with_ask(mut self, ask: AskImage) -> Self {
        self.ask = Some(ask);
        self
    }

    /// The ask heads' point at row `t` of `tokens`, whose window starts at row `start`, at Q28:
    /// `(table + a)[x_t]`, plus `Σ_{s=1..min(h, t − start)} c[x_{t−s}] · (table + a)[x_{t−s}]`
    /// with the carry. Exact: [`ASK_CARRY_MAX`] bounds it below `2^60`. `None` without ask heads.
    pub fn asked_point(&self, table: &[i16], tokens: &[u32], t: usize, start: usize) -> Option<Vec<i64>> {
        let ask = self.ask.as_ref()?;
        let width = self.width;
        let asked = |v: u32, i: usize| -> i64 {
            let at = v as usize * width + i;
            i64::from(table[at]) + i64::from(ask.table[at])
        };
        let mut point: Vec<i64> = (0..width).map(|i| asked(tokens[t], i) << 16).collect();
        if let Some((reach, gains)) = &ask.carry {
            for s in 1..=(*reach).min(t - start) {
                let v = tokens[t - s];
                let gain = i64::from(gains[v as usize]);
                for (i, p) in point.iter_mut().enumerate() {
                    *p += gain * asked(v, i);
                }
            }
        }
        Some(point)
    }

    /// Each ask head's question code at row `t` (window from `start`): the sign code of
    /// [`Self::asked_point`] through the key rotation, or `None` without ask heads.
    pub fn ask_queries(&self, table: &[i16], tokens: &[u32], t: usize, start: usize) -> Option<Vec<u64>> {
        let point = self.asked_point(table, tokens, t, start)?;
        let head_width = self.width / self.heads;
        Some((self.copy..self.heads).map(|h| head_code(&self.key, &point, h, head_width)).collect())
    }

    /// Type head `j`'s key code for vocabulary entry `token`.
    pub fn key(&self, token: u32, j: usize) -> u64 {
        self.keys[token as usize * (self.heads - self.copy) + j]
    }

    /// Each type head's query code for the hook's input `point` (`width` coordinates).
    pub fn queries(&self, point: &[i64]) -> Vec<u64> {
        let head_width = self.width / self.heads;
        (self.copy..self.heads)
            .map(|h| head_code(&self.query, point, h, head_width))
            .collect()
    }

    /// `point` through the value rotation into `out`, each coordinate rounded half away from zero
    /// back to `point`'s exponent: the payload a later row brings back from this position.
    pub fn payload(&self, point: &[i64], out: &mut [i64]) {
        for ((lane, unit), out) in point.chunks_exact(4).zip(&self.value).zip(out.chunks_exact_mut(4)) {
            let rotated = stack_hamilton(*unit, [lane[0], lane[1], lane[2], lane[3]]);
            for (o, r) in out.iter_mut().zip(rotated) {
                // A rotation keeps each lane's norm, so a coordinate is at most twice the input's
                // largest: the clamp can act only on a readout input past 2^62, which a normed,
                // gained output does not reach.
                *o = round_shift(r, 31).clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64;
            }
        }
    }

    /// Type head `j`'s kept positions at the last row `t` of `tokens`, for its query code `query`.
    pub fn choose(&self, j: usize, query: u64, tokens: &[u32]) -> Vec<(usize, u32)> {
        if tokens.is_empty() {
            return Vec::new();
        }
        nearest(query, tokens.len() - 1, self.kept, |s| self.key(tokens[s - 1], j))
    }

    /// [`Self::choose`] with each candidate `s`'s key code that `force` names in place of the
    /// computed one (claude-249). A forced query is the caller's `query`.
    fn choose_forced(&self, j: usize, query: u64, tokens: &[u32], force: &HeadForce) -> Vec<(usize, u32)> {
        if tokens.is_empty() {
            return Vec::new();
        }
        nearest(query, tokens.len() - 1, self.kept, |s| force.key(s).unwrap_or_else(|| self.key(tokens[s - 1], j)))
    }

    /// The type heads' mean at the last row of `tokens`, at the payloads' exponent minus 31, or
    /// `None` when no type head kept a candidate inside its margin. `queries` are the row's codes
    /// ([`Self::queries`]) and `payloads` holds `width` values for each position up to the row,
    /// as the copy mean: each head's `Σ w_s · v_s / max(1, #{w_s > 0})`, then the mean over the
    /// heads that found a candidate, both truncating toward zero.
    pub fn mean(&self, queries: &[u64], tokens: &[u32], payloads: &[i64]) -> Option<Vec<i128>> {
        self.mean_by(queries, tokens, payloads, None)
    }

    /// [`Self::mean`], with each head `j`'s kept positions from [`Self::choose_forced`] under
    /// `force[j]` when `force` is given, else from [`Self::choose`]. It takes no `choose` closure:
    /// its captures were copied through a SIMD register (`fmov`), which the audit refuses (claude-255).
    fn mean_by(&self, queries: &[u64], tokens: &[u32], payloads: &[i64], force: Option<&[HeadForce]>) -> Option<Vec<i128>> {
        let width = self.width;
        let mut sum = vec![0i128; width];
        let mut found = 0i128;
        for (j, &query) in queries.iter().enumerate() {
            let mut head = vec![0i128; width];
            let mut count = 0i128;
            let kept = match force {
                Some(heads) => self.choose_forced(j, query, tokens, &heads[j]),
                None => self.choose(j, query, tokens),
            };
            for (s, d) in kept {
                let weight = copy_weight_q31(self.margins[j], d);
                if weight == 0 {
                    continue;
                }
                count += 1;
                for (acc, &v) in head.iter_mut().zip(&payloads[s * width..][..width]) {
                    // Behind `black_box`, as the copy mean's points (claude-194).
                    *acc += i128::from(core::hint::black_box(v)) * i128::from(weight);
                }
            }
            if count == 0 {
                continue;
            }
            found += 1;
            for (acc, value) in sum.iter_mut().zip(head) {
                *acc += value / count;
            }
        }
        (found > 0).then(|| sum.into_iter().map(|value| value / found).collect())
    }
}

/// From `w_c ⊙ c`'s exponent (`2^HAMMING_WEIGHT_EXP` times `2^COPY_MEAN_EXP`) to the residual's Q16.
const STREAM_SHIFT: u32 = (RESIDUAL_EXP - HAMMING_WEIGHT_EXP - COPY_MEAN_EXP) as u32;
/// The final norm's output raised to the copy mean's exponent.
const NORM_RAISE: u32 = (NORM_FINE_EXP - COPY_MEAN_EXP) as u32;
/// From `w_c ⊙ (c − n)`'s exponent to the final norm's.
const READOUT_SHIFT: u32 = (NORM_FINE_EXP - HAMMING_WEIGHT_EXP - COPY_MEAN_EXP) as u32;

/// `value / 2^shift`, rounded half away from zero, as the float's `round()`.
pub(super) fn round_shift(value: i128, shift: u32) -> i128 {
    let half = 1i128 << (shift - 1);
    if value >= 0 {
        (value + half) >> shift
    } else {
        -((half - value) >> shift)
    }
}

/// The type mean's extra exponent over its payloads': a Q31 weight.
const TYPE_MEAN_SHIFT: u32 = 31;
/// The copy term `w_c ⊙ c` raised to the stream's common exponent, `w_y ⊙ y`'s:
/// `2^HAMMING_WEIGHT_EXP · 2^(RESIDUAL_EXP − 31)`.
const STREAM_COPY_RAISE: u32 = (COPY_MEAN_EXP - RESIDUAL_EXP + TYPE_MEAN_SHIFT as i32) as u32;
/// The readout's type term `w_y ⊙ y` (at `2^HAMMING_WEIGHT_EXP · 2^(NORM_FINE_EXP − 31)`) down to the
/// copy move's exponent.
const READOUT_TYPE_DROP: u32 = (COPY_MEAN_EXP - NORM_FINE_EXP + TYPE_MEAN_SHIFT as i32) as u32;

/// The largest payload cache a session may hold (claude-195), the session read cache's bound.
const MAX_PAYLOAD_BYTES: u64 = super::format::MAX_READ_CACHE_BYTES;

/// One read's choices at one session step, recorded while the session's trace is on
/// (claude-235): its depth (the layer count for the readout read) and each served head's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HammingStepRead {
    pub depth: usize,
    /// The copy heads, then the type heads where the read serves them, in head order.
    pub heads: Vec<HammingStepHead>,
}

/// One head's choice at one step's row `t`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HammingStepHead {
    /// The query code at `t`.
    pub query: u64,
    /// The key code at `t`, the code of `x_{t−1}`'s point (a copy head's under copy key 2 joins
    /// `x_{t−2}`'s); 0 at row 0, which names no candidate.
    pub key: u64,
    /// The kept positions at `t`, in the order `nearest` keeps them.
    pub kept: Vec<u32>,
}

/// One head's forced codes in one window, by step (Q5's forced replay, claude-249): `query[t]`
/// replaces row `t`'s query, and `key[s]` candidate `s`'s key at every later row, since a key
/// names only tokens before `s` (`x_{s−1}`, and `x_{s−2}` under copy key 2), decided once at step `s`.
#[derive(Clone, Debug, Default)]
pub(crate) struct HeadForce {
    query: Vec<Option<u64>>,
    key: Vec<Option<u64>>,
}

impl HeadForce {
    fn query(&self, t: usize) -> Option<u64> {
        self.query.get(t).copied().flatten()
    }

    fn key(&self, s: usize) -> Option<u64> {
        self.key.get(s).copied().flatten()
    }
}

/// One read's forced codes: a [`HeadForce`] for each head it serves, the copy heads first.
#[derive(Clone, Debug, Default)]
pub(crate) struct ReadForce {
    heads: Vec<HeadForce>,
}

/// The trace of one window `ids` from its session's recorded reads `steps`, each step's reads in
/// the order the step runs them (claude-235): the record `fbs-toks --hamming-trace` writes, as the
/// float `fbs-snapshot --hamming-trace` writes it. Refused unless every step recorded the same
/// reads with the same heads.
pub fn hamming_trace_window(ids: &[u32], steps: &[HammingStepRead]) -> Result<HammingTraceWindow, StackError> {
    let per_step = if ids.is_empty() { 0 } else { steps.len() / ids.len() };
    if per_step == 0 || per_step * ids.len() != steps.len() {
        return Err(StackError::Shape("the Hamming trace does not hold the same reads at every step"));
    }
    let first = &steps[..per_step];
    let mut reads: Vec<HammingTraceRead> = first
        .iter()
        .map(|r| HammingTraceRead {
            depth: r.depth,
            heads: vec![
                HammingTraceHead { query: Vec::with_capacity(ids.len()), key: Vec::with_capacity(ids.len()), kept: Vec::with_capacity(ids.len()), query_f32: Vec::new(), key_f32: Vec::new() };
                r.heads.len()
            ],
        })
        .collect();
    for step in steps.chunks_exact(per_step) {
        for (read, at) in reads.iter_mut().zip(step) {
            if at.depth != read.depth || at.heads.len() != read.heads.len() {
                return Err(StackError::Shape("the Hamming trace does not hold the same reads at every step"));
            }
            for (head, choice) in read.heads.iter_mut().zip(&at.heads) {
                head.query.push(choice.query);
                head.key.push(choice.key);
                head.kept.push(choice.kept.clone());
            }
        }
    }
    Ok(HammingTraceWindow { ids: ids.to_vec(), reads })
}

/// One read on the served path: its copy heads and `w_c`, weight row 0; and its type heads and
/// `w_y`, weight row 1, when any `w_y` is not zero.
struct ServedRead {
    depth: usize,
    copy: CopyHeads,
    /// `width` values at `2^HAMMING_WEIGHT_EXP`.
    copy_weight: Vec<i32>,
    types: Option<TypeHeads>,
    type_weight: Vec<i32>,
}

/// A stack's Hamming reads as the FBS session serves them (claude-194, claude-195): the copy and
/// type heads of `flock_read_hamming.rs`'s `read`, at the two hooks of the float trunk
/// (`geometric_stack_native.rs`).
pub(crate) struct ServedHamming {
    reads: Vec<ServedRead>,
    layers: usize,
    width: usize,
}

/// Refuse a payload cache of `typed` reads over `context` positions of `width` past
/// `MAX_PAYLOAD_BYTES`: a session allocates it whole when it is created.
pub(super) fn check_payload_cache(typed: usize, context: usize, width: usize) -> Result<(), StackError> {
    let bytes = (typed as u64)
        .checked_mul(context as u64)
        .and_then(|b| b.checked_mul(width as u64))
        .and_then(|b| b.checked_mul(8));
    match bytes {
        Some(b) if b <= MAX_PAYLOAD_BYTES => Ok(()),
        _ => Err(StackError::Shape(
            "the Hamming type heads' payload cache exceeds the session read cache's 4 GiB",
        )),
    }
}

impl ServedHamming {
    /// The rule the copy heads are served under: the artifact's own unless
    /// [`Self::set_copy_rule`] changed it (claude-345).
    pub(crate) fn copy_rule(&self) -> CopyRule {
        self.reads.first().map_or(CopyRule::Mean, |read| read.copy.rule())
    }

    /// Serve every read's copy heads under `rule` ([`CopyHeads::set_rule`], claude-345).
    pub(crate) fn set_copy_rule(&mut self, rule: CopyRule) {
        for read in &mut self.reads {
            read.copy.set_rule(rule);
        }
    }

    /// The reads of `image` over the Q12 `table` of `vocab` rows of `width`, in a trunk of `layers`
    /// layers and `context` positions. A read's type heads are built only when one of its type
    /// weights is not zero; at `w_y = 0` they add nothing, as in the float read.
    pub(crate) fn new(image: &HammingImage, table: &[i16], vocab: usize, width: usize, layers: usize, context: usize) -> Result<Self, StackError> {
        let mut reads = Vec::with_capacity(image.reads.len());
        for read in &image.reads {
            let (copy_weight, type_weight) = read.weight.split_at(width);
            let typed = type_weight.iter().any(|&w| w != 0);
            reads.push(ServedRead {
                depth: read.depth,
                copy: CopyHeads::new(&image.record, read, table, vocab, width),
                copy_weight: copy_weight.to_vec(),
                types: typed.then(|| {
                    let heads = TypeHeads::new(&image.record, read, table, vocab, width);
                    match &read.ask {
                        Some(ask) => heads.with_ask(ask.clone()),
                        None => heads,
                    }
                }),
                type_weight: type_weight.to_vec(),
            });
        }
        let typed = reads.iter().filter(|r| r.types.is_some()).count();
        check_payload_cache(typed, context, width)?;
        Ok(Self { reads, layers, width })
    }

    /// Whether any read serves type heads, whose per-position payloads a session keeps.
    pub(crate) fn has_type_heads(&self) -> bool {
        self.reads.iter().any(|r| r.types.is_some())
    }

    /// A session's payload caches: for each read, `context * width` zeros if it has type heads,
    /// and nothing otherwise.
    pub(crate) fn payload_caches(&self, context: usize) -> Vec<Vec<i64>> {
        self.reads
            .iter()
            .map(|r| if r.types.is_some() { vec![0; context * self.width] } else { Vec::new() })
            .collect()
    }

    /// Each read's forced codes in one window (claude-249), a [`ReadForce`] per read in the
    /// served order, from `codes` over a window of at most `context` steps. Refused: a depth no
    /// read runs at (0, past the layer count, or a second read at a depth), a head the read does
    /// not serve, a step past the context, a key at step 0 (it names no candidate), and a code
    /// forced twice.
    pub(crate) fn forcing(&self, codes: &[ForcedCode], context: usize) -> Result<Vec<ReadForce>, StackError> {
        let mut reads: Vec<ReadForce> = self
            .reads
            .iter()
            .map(|r| ReadForce {
                heads: vec![HeadForce::default(); r.copy.copy + r.types.as_ref().map_or(0, |t| t.heads - t.copy)],
            })
            .collect();
        for code in codes {
            let i = self
                .reads
                .iter()
                .position(|r| r.depth == code.depth)
                .filter(|_| (1..=self.layers).contains(&code.depth))
                .ok_or(StackError::Shape("a forced code names a depth no read runs at"))?;
            let head = reads[i]
                .heads
                .get_mut(code.head)
                .ok_or(StackError::Shape("a forced code names a head its read does not serve"))?;
            if code.step >= context {
                return Err(StackError::Shape("a forced code names a step past the context"));
            }
            let slots = match code.role {
                CodeRole::Query => &mut head.query,
                CodeRole::Key if code.step == 0 => {
                    return Err(StackError::Shape("a forced key at step 0 names no candidate"));
                }
                CodeRole::Key => &mut head.key,
            };
            if slots.len() <= code.step {
                slots.resize(code.step + 1, None);
            }
            if slots[code.step].replace(code.code).is_some() {
                return Err(StackError::Shape("a code is forced twice"));
            }
        }
        Ok(reads)
    }

    /// The type heads' query codes at row `t` of `tokens`: the ask heads' question codes
    /// ([`TypeHeads::ask_queries`]) where the read asks (claude-336), else the codes of the hook's
    /// input `point`. The session's `tokens` are its whole window, from row 0, so the carry sums
    /// back to row 0 at most.
    fn type_queries(types: &TypeHeads, point: &[i64], table: &[i16], tokens: &[u32], t: usize) -> Vec<u64> {
        types.ask_queries(table, tokens, t, 0).unwrap_or_else(|| types.queries(point))
    }

    /// The type mean at the last row `t` of `tokens` for the read `i`, with the hook's input
    /// `point`: `point`'s payload is kept at row `t` first, for the rows after it. A `force`
    /// replaces the codes it names (claude-249).
    fn type_mean(&self, i: usize, point: &[i64], tokens: &[u32], table: &[i16], payloads: &mut [Vec<i64>], force: Option<&ReadForce>) -> Option<Vec<i128>> {
        let types = self.reads[i].types.as_ref()?;
        let t = tokens.len().checked_sub(1)?;
        let cache = &mut payloads[i];
        types.payload(point, &mut cache[t * self.width..][..self.width]);
        let payloads = &cache[..(t + 1) * self.width];
        let Some(force) = force else {
            return types.mean(&Self::type_queries(types, point, table, tokens, t), tokens, payloads);
        };
        let heads = &force.heads[self.reads[i].copy.copy..];
        let mut queries = Self::type_queries(types, point, table, tokens, t);
        for (query, head) in queries.iter_mut().zip(heads) {
            if let Some(forced) = head.query(t) {
                *query = forced;
            }
        }
        types.mean_by(&queries, tokens, payloads, Some(heads))
    }

    /// Read `i`'s choices at the last row `t` of `tokens`, its type heads asking with the hook's
    /// input `point` (claude-235's trace): the codes as used, `force`'s where it names one
    /// (claude-249). It recomputes what the read's means choose, so the served sums are
    /// untouched by it.
    #[inline(never)]
    fn record(&self, i: usize, point: &[i64], tokens: &[u32], table: &[i16], force: Option<&ReadForce>) -> HammingStepRead {
        let read = &self.reads[i];
        let (Some(_), t) = (tokens.last(), tokens.len().saturating_sub(1)) else {
            return HammingStepRead { depth: read.depth, heads: Vec::new() };
        };
        let none = HeadForce::default();
        let forced = |h: usize| force.map_or(&none, |f| &f.heads[h]);
        let kept = |chosen: Vec<(usize, u32)>| chosen.into_iter().map(|(s, _)| s as u32).collect();
        let mut heads: Vec<HammingStepHead> = (0..read.copy.copy)
            .map(|h| HammingStepHead {
                query: forced(h).query(t).unwrap_or_else(|| read.copy.query_at(tokens, t, h)),
                key: if t == 0 { 0 } else { forced(h).key(t).unwrap_or_else(|| read.copy.key_at(tokens, t, h)) },
                kept: kept(read.copy.choose_forced(h, tokens, forced(h))),
            })
            .collect();
        if let Some(types) = &read.types {
            for (j, query) in Self::type_queries(types, point, table, tokens, t).into_iter().enumerate() {
                let head = forced(read.copy.copy + j);
                let query = head.query(t).unwrap_or(query);
                heads.push(HammingStepHead {
                    query,
                    key: if t == 0 { 0 } else { head.key(t).unwrap_or_else(|| types.key(tokens[t - 1], j)) },
                    kept: kept(types.choose_forced(j, query, tokens, head)),
                });
            }
        }
        HammingStepRead { depth: read.depth, heads }
    }

    /// The read inside the stream after `depth` layers, as the float trunk's
    /// `x = q16(read.apply(&x, ids, &table))`: `x + w_c ⊙ c + w_y ⊙ y` at the last row of
    /// `tokens`, each sum formed exactly and rounded once to the residual's Q16, half away from
    /// zero, as `q16` rounds the sum. A kind of head that found no candidate adds nothing, as the
    /// float's `sums / max(found, 1)` is 0 there. A depth with no read, or the layer count (the
    /// readout's), is left alone. A `trace` receives the read's choices (claude-235), and
    /// `force`, each read's from [`Self::forcing`], replaces the codes it names (claude-249).
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn stream(&self, depth: usize, tokens: &[u32], table: &[i16], payloads: &mut [Vec<i64>], x: &mut [i32], trace: Option<&mut Vec<HammingStepRead>>, force: Option<&[ReadForce]>) {
        if depth == self.layers {
            return;
        }
        let Some(i) = self.reads.iter().position(|r| r.depth == depth) else {
            return;
        };
        let force = force.and_then(|f| f.get(i));
        if let Some(trace) = trace {
            let point: Vec<i64> = x.iter().map(|&v| i64::from(v)).collect();
            trace.push(self.record(i, &point, tokens, table, force));
        }
        let read = &self.reads[i];
        let copy = match force {
            None => read.copy.mean(tokens, table),
            Some(f) => read.copy.mean_by(tokens, table, |h| read.copy.choose_forced(h, tokens, &f.heads[h])),
        };
        let typed = match read.types {
            Some(_) => {
                let point: Vec<i64> = x.iter().map(|&v| i64::from(v)).collect();
                self.type_mean(i, &point, tokens, table, payloads, force)
            }
            None => None,
        };
        if copy.is_none() && typed.is_none() {
            return;
        }
        for (k, x) in x.iter_mut().enumerate() {
            let c = copy.as_ref().map_or(0, |c| c[k]);
            let y = typed.as_ref().map_or(0, |y| y[k]);
            *x = stream_sum(*x, read.copy_weight[k], c, read.type_weight[k], y);
        }
    }

    /// The read on the final norm's output `n` (at `NORM_FINE_EXP`), as the float's
    /// `apply_readout`: `n + w_c ⊙ (c − n)` where a copy head found a candidate at the last row
    /// of `tokens`, plus `w_y ⊙ y` where a type head did. The type term is pre-rounded 43 bits
    /// below `n`'s grid ([`readout_sum`]), and the sum then rounded once to `n`'s exponent. Not
    /// rounded to Q16, as the float trunk does not round the readout read. A `trace` receives the
    /// read's choices (claude-235), and `force` replaces the codes it names (claude-249).
    #[inline(never)]
    pub(crate) fn readout(&self, tokens: &[u32], table: &[i16], payloads: &mut [Vec<i64>], n: &mut [i64], trace: Option<&mut Vec<HammingStepRead>>, force: Option<&[ReadForce]>) {
        let Some(i) = self.reads.iter().position(|r| r.depth == self.layers) else {
            return;
        };
        let force = force.and_then(|f| f.get(i));
        if let Some(trace) = trace {
            trace.push(self.record(i, n, tokens, table, force));
        }
        let read = &self.reads[i];
        let copy = match force {
            None => read.copy.mean(tokens, table),
            Some(f) => read.copy.mean_by(tokens, table, |h| read.copy.choose_forced(h, tokens, &f.heads[h])),
        };
        let typed = self.type_mean(i, n, tokens, table, payloads, force);
        if copy.is_none() && typed.is_none() {
            return;
        }
        for (k, n) in n.iter_mut().enumerate() {
            let c = copy.as_ref().map(|c| c[k]);
            let y = typed.as_ref().map_or(0, |y| y[k]);
            *n = readout_sum(*n, read.copy_weight[k], c, read.type_weight[k], y);
        }
    }
}

/// One coordinate of the stream read: `x + w_c · c + w_y · y` (`x` at Q16, the weights at
/// `2^HAMMING_WEIGHT_EXP`, `c` at `2^COPY_MEAN_EXP`, `y` at `2^(RESIDUAL_EXP − 31)`), the sum formed
/// exactly at `w_y · y`'s exponent and rounded once to Q16, half away from zero.
pub(super) fn stream_sum(x: i32, w_c: i32, c: i128, w_y: i32, y: i128) -> i32 {
    // `|x · 2^55| < 2^86`; `|w_c · c · 2^4| < 2^81`; `|y| < 2^63` (a mean of Q31-weighted payloads
    // of at most 2^32), so `|w_y · y| < 2^94`: the sum fits an i128.
    let shift = STREAM_SHIFT + STREAM_COPY_RAISE;
    let sum = (i128::from(x) << shift) + ((i128::from(w_c) * c) << STREAM_COPY_RAISE) + i128::from(w_y) * y;
    round_shift(sum, shift).clamp(i128::from(i32::MIN), i128::from(i32::MAX)) as i32
}

/// One coordinate of the readout move: `n + w_c · (c − n) + w_y · y` (`n` at `NORM_FINE_EXP`,
/// `c` present where a copy head matched, `y` at `2^(NORM_FINE_EXP − 31)`), rounded once to `n`'s
/// exponent, half away from zero. The type term is first rounded to the copy move's
/// `2^(HAMMING_WEIGHT_EXP + COPY_MEAN_EXP)`, 43 bits below `n`'s grid, so the sum stays inside an
/// i128 for any `n`.
pub(super) fn readout_sum(n: i64, w_c: i32, c: Option<i128>, w_y: i32, y: i128) -> i64 {
    // `|n| < 2^63`, so `n · 2^43 < 2^106`; `|c − n · 2^19| < 2^83`, so the move `< 2^114`;
    // `|y| < 2^95`, so `|w_y · y| < 2^126` and its rounded term `< 2^114`: the sum fits an i128.
    let moved = c.map_or(0, |c| i128::from(w_c) * (c - (i128::from(n) << NORM_RAISE)));
    let typed = round_shift(i128::from(w_y) * y, READOUT_TYPE_DROP);
    let sum = round_shift((i128::from(n) << READOUT_SHIFT) + moved + typed, READOUT_SHIFT);
    sum.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64
}

#[cfg(test)]
thread_local! {
    /// While `Some`, the residual after every layer of each FBS step, its read applied, in order
    /// (claude-194's hook tests). Present in test builds only, and off until a test turns it on.
    pub(super) static LAYER_TRACE: std::cell::RefCell<Option<Vec<Vec<i32>>>> = const { std::cell::RefCell::new(None) };
}
