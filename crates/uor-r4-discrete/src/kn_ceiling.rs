//! claude-433: the count-mixer ceiling. Raw count tables at every order give
//! each query target several count experts (KN, the maximum-likelihood mass
//! at each seen order, and the uniform base), and a bucketed linear mix of
//! them is fit by integer EM. All masses are Q48 and all weights Q32.
//!
//! The lookup reads raw tables with exactly `KnTables`'s key derivation,
//! binary searches and order range, so a CUDA kernel can later produce the
//! same flat (count, total) layout. Bits are taken by a caller-passed
//! function, as `bits_q12` lives in `uor-r4-integer`.

use crate::kn_score::KN_BASE_MASS;
use crate::ngram_count::{
    NgramContext, NgramOrderDigest, NgramTriple, PrimeRouteKey, NGRAM_BOS, NGRAM_MAX_ORDER,
    NGRAM_VOCABULARY,
};
use crate::{Error, Result};

mod frozen_pick;
pub use frozen_pick::FrozenPickMixture;

/// Q32 unit of a mixing weight.
pub const WEIGHT_ONE: u64 = 1 << 32;
/// Buckets per seen order: min(15, floor(log2 total)).
pub const BUCKETS_PER_ORDER: usize = 16;

fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(message.into())
}

/// Raw count tables of orders 0..=highest, read by binary search in the
/// counter's canonical order (key, then next).
pub struct RawTables {
    highest: u32,
    orders: Vec<(Vec<NgramTriple>, Vec<NgramContext>)>,
}

impl RawTables {
    /// Takes the raw counter outputs of orders 0..=highest, in order, and
    /// validates them as `KnTables::new` does.
    pub fn new(
        highest: u32,
        orders: Vec<(NgramOrderDigest, Vec<NgramTriple>, Vec<NgramContext>)>,
    ) -> Result<Self> {
        if highest > NGRAM_MAX_ORDER || orders.len() != highest as usize + 1 {
            return Err(invalid(format!(
                "raw tables need orders 0..={highest} (at most {NGRAM_MAX_ORDER}), got {} tables",
                orders.len()
            )));
        }
        let mut tables = Vec::with_capacity(orders.len());
        for (k, (digest, triples, contexts)) in orders.into_iter().enumerate() {
            if digest.order != k as u32 {
                return Err(invalid(format!("table {k} holds order {}", digest.order)));
            }
            let triples_sorted = triples.windows(2).all(|w| {
                let (x, y) = (&w[0], &w[1]);
                (x.key, x.next) < (y.key, y.next)
            });
            let contexts_sorted = contexts.windows(2).all(|w| w[0].key < w[1].key);
            if !triples_sorted || !contexts_sorted {
                return Err(invalid(format!(
                    "order-{k} table is not in canonical order"
                )));
            }
            let keys = triples
                .iter()
                .map(|t| t.key)
                .chain(contexts.iter().map(|c| c.key));
            for key in keys {
                if PrimeRouteKey::from_word(key)?.order() != k as u32 {
                    return Err(invalid(format!(
                        "order-{k} table holds a key of another order"
                    )));
                }
            }
            if contexts.iter().any(|c| c.total == 0) {
                return Err(invalid(format!(
                    "order-{k} table holds a context with total 0"
                )));
            }
            tables.push((triples, contexts));
        }
        Ok(Self {
            highest,
            orders: tables,
        })
    }

    pub fn highest(&self) -> u32 {
        self.highest
    }

    /// Order k's triples and contexts, in canonical order, for a device upload.
    pub fn order_tables(&self, k: u32) -> Option<(&[NgramTriple], &[NgramContext])> {
        let (triples, contexts) = self.orders.get(k as usize)?;
        Some((triples, contexts))
    }

    /// For every non-BOS target j in stream order, highest+1 pairs
    /// (count a, context total), flattened target-major at t*(highest+1)+k.
    /// An unseen context, or k > j, gives (0, 0); a seen context without the
    /// target gives (0, total).
    pub fn lookups(&self, tokens: &[u16]) -> Result<(Vec<u32>, Vec<u64>)> {
        let width = self.highest as usize + 1;
        let mut counts = Vec::new();
        let mut totals = Vec::new();
        for (j, &target) in tokens.iter().enumerate() {
            if target == NGRAM_BOS {
                continue;
            }
            if u32::from(target) >= NGRAM_VOCABULARY {
                return Err(invalid(format!(
                    "token id {target} is outside the 12-bit vocabulary"
                )));
            }
            let start = counts.len();
            counts.resize(start + width, 0);
            totals.resize(start + width, 0);
            for k in 0..=(self.highest as usize).min(j) {
                let (triples, contexts) = &self.orders[k];
                let key = PrimeRouteKey::from_context(&tokens[j - k..j])?.word();
                let Ok(at) = contexts.binary_search_by(|c| c.key.cmp(&key)) else {
                    continue;
                };
                totals[start + k] = contexts[at].total;
                counts[start + k] = triples
                    .binary_search_by(|t| (t.key, t.next).cmp(&(key, target)))
                    .map_or(0, |i| triples[i].count);
            }
        }
        Ok((counts, totals))
    }
}

/// Experts per target: KN, ML_0..=ML_highest, U.
pub fn expert_count(highest: u32) -> usize {
    highest as usize + 3
}

/// Buckets: 16 per highest seen order.
pub fn bucket_count(highest: u32) -> usize {
    (highest as usize + 1) * BUCKETS_PER_ORDER
}

/// The highest seen order h of one target's totals row and its bucket
/// h*16 + min(15, floor(log2 total_h)). Refuses a target with no seen order.
pub fn ceiling_bucket(totals: &[u64]) -> Result<(usize, u16)> {
    let h = totals
        .iter()
        .rposition(|&total| total > 0)
        .ok_or_else(|| invalid("a target has no seen order"))?;
    let log2 = 63 - totals[h].leading_zeros() as usize;
    let bucket = h * BUCKETS_PER_ORDER + log2.min(BUCKETS_PER_ORDER - 1);
    let bucket = u16::try_from(bucket).map_err(|_| invalid("a bucket exceeded u16"))?;
    Ok((h, bucket))
}

/// Convert one raw lookup row into its bucket and Q48 expert masses.
/// The caller supplies KN, one count/total pair per order, and a row with
/// one slot per expert. Orders beyond the last seen context are zero.
pub fn ceiling_expert_row(
    kn: u64,
    counts: &[u32],
    totals: &[u64],
    masses: &mut [u64],
) -> Result<u16> {
    if counts.len() != totals.len() || masses.len() != totals.len() + 2 {
        return Err(invalid("invalid ceiling expert row shape"));
    }
    let (_, bucket) = ceiling_bucket(totals)?;
    masses[0] = kn;
    for (i, (&count, &total)) in counts.iter().zip(totals).enumerate() {
        if total != 0 && u64::from(count) > total {
            return Err(invalid(format!(
                "order {i} count {count} exceeds its total {total}"
            )));
        }
        masses[i + 1] = if total == 0 {
            0
        } else {
            // count <= total, so this quotient is at most 2^48.
            ((u128::from(count) << 48) / u128::from(total)) as u64
        };
    }
    masses[totals.len() + 1] = KN_BASE_MASS;
    Ok(bucket)
}

/// Whether expert e is present in bucket b: KN and U always, ML_k iff k <= h.
fn present(highest: u32, bucket: usize, e: usize) -> bool {
    let h = bucket / BUCKETS_PER_ORDER;
    e == 0 || e == expert_count(highest) - 1 || e - 1 <= h
}

/// The fit: Q32 weights (buckets x experts) and the summed bits over the
/// fit targets before the first update and after each update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CeilingFit {
    pub weights: Vec<u64>,
    pub fit_bits: Vec<u128>,
}

/// Where the integer EM starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmStart {
    /// Uniform over each bucket's present experts (claude-433).
    Uniform,
    /// claude-438: 2^32 - (L - 1) on KN and 1 on each of the other L - 1
    /// present experts, so the start's mix is KN's own mass up to floors and
    /// every present expert can still gain weight.
    Kn,
}

/// claude-439: the Pick class of a target that did not fire.
pub const PICK_NOT_FIRED: u8 = 0;
/// claude-439: Pick fired at pair distance 0.
pub const PICK_FIRED_EXACT: u8 = 1;
/// claude-439: Pick fired at pair distance above 0.
pub const PICK_FIRED_NEAR: u8 = 2;
/// claude-439: Pick classes; each holds one copy of the base buckets.
pub const PICK_CLASSES: u8 = 3;

/// claude-439: one target's Pick reading: its class and whether the copied
/// token is the target. `hit` needs a fired class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PickTarget {
    pub class: u8,
    pub hit: bool,
}

fn ceiling_pick_row(
    highest: u32,
    kn: u64,
    counts: &[u32],
    totals: &[u64],
    pick: PickTarget,
    row: &mut [u64],
) -> Result<u16> {
    if pick.class >= PICK_CLASSES {
        return Err(invalid(format!(
            "Pick class {} is above {}",
            pick.class,
            PICK_CLASSES - 1
        )));
    }
    if pick.hit && pick.class == PICK_NOT_FIRED {
        return Err(invalid("a Pick hit that did not fire"));
    }
    let base = expert_count(highest);
    let plain = ceiling_expert_row(kn, counts, totals, &mut row[..base])?;
    row[base] = if pick.hit { 1 << 48 } else { 0 };
    let bucket = usize::from(pick.class) * bucket_count(highest) + usize::from(plain);
    u16::try_from(bucket).map_err(|_| invalid("a bucket exceeded u16"))
}

fn mix_sum(weights: &[u64], masses: &[u64]) -> u128 {
    weights
        .iter()
        .zip(masses)
        .map(|(&weight, &mass)| u128::from(weight) * u128::from(mass))
        .sum()
}

fn mixed_q48(sum: u128) -> u64 {
    u64::try_from(sum >> 32).unwrap_or(u64::MAX).max(1)
}

/// Every target's bucket and its expert masses.
pub struct CeilingTable {
    highest: u32,
    /// claude-439: whether the table holds the Pick column after U.
    pick: bool,
    buckets: Vec<u16>,
    masses: Vec<u64>,
}

impl CeilingTable {
    /// claude-439: `from_lookups` plus the Pick expert at index
    /// `expert_count(highest)`, with mass 2^48 on a hit and 0 otherwise.
    /// A target's bucket is class * `bucket_count(highest)` plus its base
    /// bucket, and Pick is present only in the buckets of a fired class.
    /// Refuses a Pick list of another length, a class above 2 and a hit
    /// that did not fire.
    pub fn from_lookups_with_pick(
        highest: u32,
        kn_masses: &[u64],
        counts: &[u32],
        totals: &[u64],
        pick: &[PickTarget],
    ) -> Result<Self> {
        if pick.len() != kn_masses.len() {
            return Err(invalid(format!(
                "{} KN masses need as many Pick targets, got {}",
                kn_masses.len(),
                pick.len()
            )));
        }
        let width = highest as usize + 1;
        let targets = kn_masses.len();
        if counts.len() != targets * width || totals.len() != targets * width {
            return Err(invalid(format!(
                "{targets} KN masses need {} counts and totals, got {} and {}",
                targets * width,
                counts.len(),
                totals.len()
            )));
        }
        // Rows are written in place, so no Pick-free copy of the table is held.
        let base = expert_count(highest);
        let mut buckets = Vec::with_capacity(targets);
        let mut masses = vec![0u64; targets * (base + 1)];
        for (t, &p) in pick.iter().enumerate() {
            let row = &mut masses[t * (base + 1)..(t + 1) * (base + 1)];
            let bucket = ceiling_pick_row(
                highest,
                kn_masses[t],
                &counts[t * width..(t + 1) * width],
                &totals[t * width..(t + 1) * width],
                p,
                row,
            )
            .map_err(|error| invalid(format!("target {t}: {error}")))?;
            buckets.push(bucket);
        }
        Ok(Self {
            highest,
            pick: true,
            buckets,
            masses,
        })
    }

    /// claude-439: buckets of this table: `bucket_count(highest)`, times
    /// `PICK_CLASSES` with the Pick column.
    pub fn bucket_total(&self) -> usize {
        let classes = if self.pick { usize::from(PICK_CLASSES) } else { 1 };
        bucket_count(self.highest) * classes
    }

    /// claude-439: the Pick expert's index, if the table holds the column.
    pub fn pick_expert(&self) -> Option<usize> {
        self.pick.then(|| expert_count(self.highest))
    }

    /// Whether expert e is present in bucket b of this table: Pick in the
    /// buckets of a fired class, every other expert by its base bucket.
    fn holds(&self, b: usize, e: usize) -> bool {
        let stride = bucket_count(self.highest);
        if self.pick_expert() == Some(e) {
            return b / stride != usize::from(PICK_NOT_FIRED);
        }
        present(self.highest, b % stride, e)
    }

    /// Builds the experts from KN masses and the lookup layout of
    /// `RawTables::lookups`.
    pub fn from_lookups(
        highest: u32,
        kn_masses: &[u64],
        counts: &[u32],
        totals: &[u64],
    ) -> Result<Self> {
        let width = highest as usize + 1;
        let targets = kn_masses.len();
        if counts.len() != targets * width || totals.len() != targets * width {
            return Err(invalid(format!(
                "{targets} KN masses need {} counts and totals, got {} and {}",
                targets * width,
                counts.len(),
                totals.len()
            )));
        }
        let experts = expert_count(highest);
        let mut buckets = Vec::with_capacity(targets);
        let mut masses = vec![0u64; targets * experts];
        for (t, &kn) in kn_masses.iter().enumerate() {
            let row_counts = &counts[t * width..(t + 1) * width];
            let row_totals = &totals[t * width..(t + 1) * width];
            let row = &mut masses[t * experts..(t + 1) * experts];
            let bucket = ceiling_expert_row(kn, row_counts, row_totals, row)
                .map_err(|error| invalid(format!("target {t}: {error}")))?;
            buckets.push(bucket);
        }
        Ok(Self {
            highest,
            pick: false,
            buckets,
            masses,
        })
    }

    pub fn experts(&self) -> usize {
        expert_count(self.highest) + usize::from(self.pick)
    }

    pub fn targets(&self) -> usize {
        self.buckets.len()
    }

    pub fn buckets(&self) -> &[u16] {
        &self.buckets
    }

    pub fn masses(&self) -> &[u64] {
        &self.masses
    }

    /// Uniform Q32 weights over each bucket's present experts, the
    /// remainder on the lowest present index.
    pub fn uniform_weights(&self) -> Vec<u64> {
        self.uniform_weights_with(&vec![true; self.experts()])
    }

    /// Uniform Q32 weights over each bucket's present experts that `on`
    /// keeps; an expert `on` drops holds 0 in every bucket.
    fn uniform_weights_with(&self, on: &[bool]) -> Vec<u64> {
        let experts = self.experts();
        let mut weights = vec![0u64; self.bucket_total() * experts];
        for b in 0..self.bucket_total() {
            let row = &mut weights[b * experts..(b + 1) * experts];
            let live: Vec<usize> = (0..experts)
                .filter(|&e| on[e] && self.holds(b, e))
                .collect();
            let share = WEIGHT_ONE / live.len() as u64;
            for &e in &live {
                row[e] = share;
            }
            row[live[0]] += WEIGHT_ONE - share * live.len() as u64;
        }
        weights
    }

    /// claude-438: the KN start over each bucket's present experts that `on`
    /// keeps: each of them at 1 but KN, which holds the rest of 2^32. An
    /// expert `on` drops holds 0. Refuses a mask that drops KN.
    fn kn_weights_with(&self, on: &[bool]) -> Result<Vec<u64>> {
        if !on[0] {
            return Err(invalid("the KN start needs the KN expert"));
        }
        let experts = self.experts();
        let mut weights = vec![0u64; self.bucket_total() * experts];
        for b in 0..self.bucket_total() {
            let row = &mut weights[b * experts..(b + 1) * experts];
            let others = (1..experts).filter(|&e| on[e] && self.holds(b, e));
            let mut rest = WEIGHT_ONE;
            for e in others {
                row[e] = 1;
                rest -= 1;
            }
            row[0] = rest;
        }
        Ok(weights)
    }

    /// Target t's unshifted mix sum of w_e * p_e.
    fn mix_sum(&self, weights: &[u64], t: usize) -> u128 {
        let experts = self.experts();
        let b = usize::from(self.buckets[t]);
        let w = &weights[b * experts..(b + 1) * experts];
        let p = &self.masses[t * experts..(t + 1) * experts];
        mix_sum(w, p)
    }

    fn mixed(&self, weights: &[u64], t: usize) -> u64 {
        mixed_q48(self.mix_sum(weights, t))
    }

    /// Every target's mixed Q48 mass, at least 1.
    pub fn mixed_masses(&self, weights: &[u64]) -> Result<Vec<u64>> {
        self.check_weights(weights)?;
        Ok((0..self.targets())
            .map(|t| self.mixed(weights, t))
            .collect())
    }

    fn check_weights(&self, weights: &[u64]) -> Result<()> {
        let want = self.bucket_total() * self.experts();
        if weights.len() != want {
            return Err(invalid(format!(
                "weights need {want} entries, got {}",
                weights.len()
            )));
        }
        Ok(())
    }

    fn fit_bits_sum<E: std::fmt::Display>(
        &self,
        weights: &[u64],
        fit: &[bool],
        bits: &impl Fn(u64) -> std::result::Result<u64, E>,
    ) -> Result<u128> {
        let mut sum = 0u128;
        for t in (0..self.targets()).filter(|&t| fit[t]) {
            let b = bits(self.mixed(weights, t)).map_err(|e| invalid(e.to_string()))?;
            sum += u128::from(b);
        }
        Ok(sum)
    }

    /// Integer EM over the fit targets, from uniform weights.
    pub fn fit_em<E: std::fmt::Display>(
        &self,
        fit: &[bool],
        iterations: u32,
        bits: impl Fn(u64) -> std::result::Result<u64, E>,
    ) -> Result<CeilingFit> {
        self.fit_em_with(fit, &vec![true; self.experts()], iterations, bits)
    }

    /// Integer EM over the fit targets with only the experts `experts` keeps:
    /// an expert it drops is absent in every bucket, at weight 0 from the
    /// start and never lifted to 1. All experts on is `fit_em`. Refuses a
    /// mask of the wrong length and one that leaves a bucket no expert.
    pub fn fit_em_with<E: std::fmt::Display>(
        &self,
        fit: &[bool],
        experts: &[bool],
        iterations: u32,
        bits: impl Fn(u64) -> std::result::Result<u64, E>,
    ) -> Result<CeilingFit> {
        self.fit_em_from(fit, experts, EmStart::Uniform, iterations, bits)
    }

    /// claude-438: `fit_em_with` from the chosen start.
    pub fn fit_em_from<E: std::fmt::Display>(
        &self,
        fit: &[bool],
        experts: &[bool],
        start: EmStart,
        iterations: u32,
        bits: impl Fn(u64) -> std::result::Result<u64, E>,
    ) -> Result<CeilingFit> {
        let on = self.check_fit_inputs(fit, experts)?;
        let experts = self.experts();
        let buckets = self.bucket_total();
        let mut weights = match start {
            EmStart::Uniform => self.uniform_weights_with(on),
            EmStart::Kn => self.kn_weights_with(on)?,
        };
        let mut fit_bits = vec![self.fit_bits_sum(&weights, fit, &bits)?];
        for _ in 0..iterations {
            let mut resp = vec![0u128; buckets * experts];
            let mut seen = vec![false; buckets];
            for t in (0..self.targets()).filter(|&t| fit[t]) {
                let b = usize::from(self.buckets[t]);
                let m = self.mix_sum(&weights, t);
                if m == 0 {
                    continue;
                }
                seen[b] = true;
                let w = &weights[b * experts..(b + 1) * experts];
                let p = &self.masses[t * experts..(t + 1) * experts];
                for e in 0..experts {
                    resp[b * experts + e] += ((u128::from(w[e]) * u128::from(p[e])) << 32) / m;
                }
            }
            for b in (0..buckets).filter(|&b| seen[b]) {
                let r = &resp[b * experts..(b + 1) * experts];
                let row = &mut weights[b * experts..(b + 1) * experts];
                self.update_bucket(b, on, r, row);
            }
            fit_bits.push(self.fit_bits_sum(&weights, fit, &bits)?);
        }
        Ok(CeilingFit { weights, fit_bits })
    }

    /// claude-438: an upper bound, in Q12 bits, on how far the fit targets'
    /// scored bits at `weights` (the sum `fit_em` records) can be above the
    /// bits of the best weights that keep each bucket's present experts
    /// `experts` keeps, scored with real-valued masses. Any weights' scored
    /// bits are at least that best, so scored bits minus this bound is a
    /// lower bound on every such fit's score.
    ///
    /// Two parts, both rounded up:
    /// - the real-valued distance. For a bucket b with N_b fit targets and
    ///   mixes m_t at `weights` (read as reals), let g_e =
    ///   (1/N_b) sum_t p_te / m_t. Then sum_e w_e g_e = 1, and for any weights
    ///   w* of the bucket Jensen's inequality gives sum_t log2(m*_t / m_t) <=
    ///   N_b log2(sum_e w*_e g_e) <= N_b log2(max_e g_e). G = max_e g_e is
    ///   taken in Q32 with floors (under 2^-31 relative low), and its log2 is
    ///   `bits` of the Q48 mass floor(2^80 / G), plus one Q12 unit per target;
    /// - the scoring floors. A scored mass is floor(m / 2^32), which adds at
    ///   most log2(1 + 1 / mass) < 2 / mass bits, and `bits` sits at most one
    ///   Q12 unit (plus 2^-48 bit) above the exact bits: 2 + ceil(8192 / mass)
    ///   Q12 units per target.
    pub fn gap_bound_q12<E: std::fmt::Display>(
        &self,
        weights: &[u64],
        fit: &[bool],
        experts: &[bool],
        bits: impl Fn(u64) -> std::result::Result<u64, E>,
    ) -> Result<u128> {
        let on = self.check_fit_inputs(fit, experts)?;
        self.check_weights(weights)?;
        let experts = self.experts();
        let buckets = self.bucket_total();
        let mut ratio = vec![0u128; buckets * experts];
        let mut n = vec![0u128; buckets];
        let mut floors = 0u128;
        for t in (0..self.targets()).filter(|&t| fit[t]) {
            let b = usize::from(self.buckets[t]);
            let m = self.mix_sum(weights, t);
            if m == 0 {
                return Err(invalid(format!("target {t} has a zero mix")));
            }
            n[b] += 1;
            floors += 2 + u128::from(8192u64.div_ceil(self.mixed(weights, t)));
            let p = &self.masses[t * experts..(t + 1) * experts];
            for e in (0..experts).filter(|&e| on[e] && self.holds(b, e)) {
                let term = (u128::from(p[e]) << 64) / m;
                let sum = &mut ratio[b * experts + e];
                *sum = sum
                    .checked_add(term)
                    .ok_or_else(|| invalid("a ratio sum exceeded 2^128"))?;
            }
        }
        let mut gap = 0u128;
        for b in (0..buckets).filter(|&b| n[b] > 0) {
            let g = ratio[b * experts..(b + 1) * experts]
                .iter()
                .map(|&r| r / n[b])
                .max()
                .unwrap_or(0);
            let one = u128::from(WEIGHT_ONE);
            let log2 = if g <= one {
                0
            } else {
                let mass = u64::try_from((1u128 << 80) / g)
                    .map_err(|_| invalid("a bound mass exceeded 2^64"))?;
                if mass == 0 {
                    return Err(invalid(format!(
                        "bucket {b}: the bound's ratio is too large"
                    )));
                }
                bits(mass).map_err(|e| invalid(e.to_string()))?
            };
            gap += n[b] * (u128::from(log2) + 1);
        }
        Ok(gap + floors)
    }

    /// The expert mask and fit selection of a fit, checked: the mask's length,
    /// a bucket the mask leaves no expert, and the fit selection's length.
    fn check_fit_inputs<'a>(&self, fit: &[bool], experts: &'a [bool]) -> Result<&'a [bool]> {
        let on = experts;
        if on.len() != self.experts() {
            return Err(invalid(format!(
                "the expert mask holds {} entries, not {}",
                on.len(),
                self.experts()
            )));
        }
        if let Some(b) = (0..self.bucket_total())
            .find(|&b| !(0..on.len()).any(|e| on[e] && self.holds(b, e)))
        {
            return Err(invalid(format!(
                "the expert mask leaves bucket {b} no expert"
            )));
        }
        if fit.len() != self.targets() {
            return Err(invalid(format!(
                "fit selects {} targets of {}",
                fit.len(),
                self.targets()
            )));
        }
        Ok(on)
    }

    /// One bucket's M step: w_e = floor(R_e * 2^32 / sum R), the remainder
    /// to the largest weight, then every present expert at 0 lifted to 1
    /// from the largest weight. A bucket with sum R = 0 keeps its weights.
    fn update_bucket(&self, b: usize, on: &[bool], r: &[u128], row: &mut [u64]) {
        let total: u128 = r.iter().sum();
        if total == 0 {
            return;
        }
        for (w, &re) in row.iter_mut().zip(r) {
            *w = u64::try_from((re << 32) / total).unwrap_or(u64::MAX);
        }
        let assigned: u64 = row.iter().sum();
        let top = largest(row);
        row[top] += WEIGHT_ONE - assigned;
        for e in 0..row.len() {
            if on[e] && self.holds(b, e) && row[e] == 0 {
                let top = largest(row);
                row[top] -= 1;
                row[e] = 1;
            }
        }
    }
}

/// Index of the largest weight, lowest index on ties.
fn largest(row: &[u64]) -> usize {
    let mut best = 0;
    for (e, &w) in row.iter().enumerate() {
        if w > row[best] {
            best = e;
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ngram_count::ngram_admitted;
    use std::collections::BTreeMap;

    /// Exact -log2(mass / 2^48) in Q12 by repeated squaring, integer only.
    fn test_bits(mass: u64) -> std::result::Result<u64, String> {
        if mass == 0 {
            return Err("mass 0".into());
        }
        let n = 63 - u64::from(mass.leading_zeros());
        let mut y = u128::from(mass) << (62 - n);
        let mut frac = 0u64;
        for _ in 0..12 {
            y = (y * y) >> 62;
            frac <<= 1;
            if y >= 2 << 62 {
                frac |= 1;
                y >>= 1;
            }
        }
        Ok((48u64 << 12).saturating_sub((n << 12) | frac))
    }

    fn digest(order: u32) -> NgramOrderDigest {
        NgramOrderDigest {
            order,
            targets: 0,
            grams: 0,
            contexts: 0,
            count_of_counts: [0; 4],
            triple_sha256: String::new(),
            context_sha256: String::new(),
        }
    }

    /// Raw tables counted from `train` with the admission rule.
    fn raw_tables(train: &[u16], highest: u32) -> RawTables {
        let mut orders = Vec::new();
        for k in 0..=highest as usize {
            let mut grams: BTreeMap<(u128, u16), u32> = BTreeMap::new();
            let mut ctx: BTreeMap<u128, u64> = BTreeMap::new();
            for j in k..train.len() {
                if !ngram_admitted(train, j, k) {
                    continue;
                }
                let key = PrimeRouteKey::from_context(&train[j - k..j])
                    .unwrap()
                    .word();
                *grams.entry((key, train[j])).or_default() += 1;
                *ctx.entry(key).or_default() += 1;
            }
            let triples = grams
                .into_iter()
                .map(|((key, next), count)| NgramTriple { key, next, count })
                .collect();
            let contexts = ctx
                .into_iter()
                .map(|(key, total)| NgramContext {
                    key,
                    total,
                    followers: 0,
                    n1: 0,
                    n2: 0,
                    n3_plus: 0,
                })
                .collect();
            orders.push((digest(k as u32), triples, contexts));
        }
        RawTables::new(highest, orders).unwrap()
    }

    /// Brute force over token slices: (a, total) per non-BOS target and order.
    fn brute(train: &[u16], query: &[u16], highest: usize) -> (Vec<u32>, Vec<u64>) {
        let mut grams: BTreeMap<Vec<u16>, u32> = BTreeMap::new();
        let mut ctx: BTreeMap<Vec<u16>, u64> = BTreeMap::new();
        for j in 0..train.len() {
            for k in 0..=highest.min(j) {
                if ngram_admitted(train, j, k) {
                    *grams.entry(train[j - k..=j].to_vec()).or_default() += 1;
                    *ctx.entry(train[j - k..j].to_vec()).or_default() += 1;
                }
            }
        }
        let (mut counts, mut totals) = (Vec::new(), Vec::new());
        for j in (0..query.len()).filter(|&j| query[j] != NGRAM_BOS) {
            for k in 0..=highest {
                if k > j {
                    counts.push(0);
                    totals.push(0);
                    continue;
                }
                totals.push(ctx.get(&query[j - k..j]).copied().unwrap_or(0));
                counts.push(grams.get(&query[j - k..=j]).copied().unwrap_or(0));
            }
        }
        (counts, totals)
    }

    /// claude-439: lookups never cross a BOS, so a BOS-framed stream looked up in
    /// BOS-aligned pieces gives the whole stream's lookups. The Pick arms' host check
    /// compares one such piece at a time and relies on this.
    #[test]
    fn lookups_in_bos_aligned_pieces_equal_the_whole() {
        let train: Vec<u16> = vec![
            0, 5, 6, 7, 5, 6, 8, 1, 0, 5, 6, 7, 9, 1, 0, 6, 7, 5, 6, 7, 1, 0, 9, 9, 9, 1,
        ];
        let query: Vec<u16> = vec![
            0, 5, 6, 7, 1, 0, 6, 7, 5, 6, 8, 1, 0, 9, 9, 9, 6, 1, 0, 7, 5, 6, 7, 1,
        ];
        let starts: Vec<usize> = (0..query.len()).filter(|&i| query[i] == 0).collect();
        for highest in [2u32, 3] {
            let tables = raw_tables(&train, highest);
            let whole = tables.lookups(&query).unwrap();
            for cut in 1..starts.len() {
                let (a, b) = query.split_at(starts[cut]);
                let (mut counts, mut totals) = tables.lookups(a).unwrap();
                let (c, t) = tables.lookups(b).unwrap();
                counts.extend(c);
                totals.extend(t);
                assert_eq!((counts, totals), whole, "highest {highest}, cut {cut}");
            }
            // A cut inside a record is not the same: the pieces must start at a BOS.
            let (a, b) = query.split_at(starts[1] + 3);
            let (mut counts, mut totals) = tables.lookups(a).unwrap();
            let (c, t) = tables.lookups(b).unwrap();
            counts.extend(c);
            totals.extend(t);
            assert_ne!((counts, totals), whole, "highest {highest}, mid-record cut");
        }
    }

    #[test]
    fn lookups_match_brute_force() {
        let train: Vec<u16> = vec![
            0, 5, 6, 7, 5, 6, 8, 0, 5, 6, 7, 9, 0, 6, 7, 5, 6, 7, 0, 9, 9, 9,
        ];
        let query: Vec<u16> = vec![5, 6, 7, 0, 5, 6, 8, 11, 0, 9, 9, 9, 6, 7, 5, 12, 6];
        for highest in [2u32, 3] {
            let tables = raw_tables(&train, highest);
            let got = tables.lookups(&query).unwrap();
            let want = brute(&train, &query, highest as usize);
            assert_eq!(got, want, "highest {highest}");
            let (counts, totals) = got;
            let width = highest as usize + 1;
            assert_eq!(totals.len(), 15 * width);
            // k > j: the first target (j = 0) sees only order 0.
            assert!(totals[1..width].iter().all(|&t| t == 0));
            // An unseen context: 11 then 0 is never a context; 12 is unseen.
            assert!(totals.iter().zip(&counts).any(|(&t, &a)| t > 0 && a == 0));
            assert!(totals.iter().filter(|&&t| t == 0).count() > width);
        }
    }

    #[test]
    fn raw_tables_refuse_bad_input() {
        let tables = raw_tables(&[0, 5, 6, 7], 1);
        assert_eq!(tables.highest(), 1);
        let (triples, contexts) = tables.order_tables(1).unwrap();
        let (triples, contexts) = (triples.to_vec(), contexts.to_vec());
        let (t0, c0) = tables.order_tables(0).unwrap();
        let (t0, c0) = (t0.to_vec(), c0.to_vec());
        assert!(RawTables::new(1, vec![(digest(0), t0.clone(), c0.clone())]).is_err());
        let swapped = vec![
            (digest(0), triples.clone(), contexts.clone()),
            (digest(1), t0.clone(), c0.clone()),
        ];
        assert!(RawTables::new(1, swapped).is_err());
        let mut zero = c0.clone();
        zero[0].total = 0;
        let bad = vec![
            (digest(0), t0.clone(), zero),
            (digest(1), triples.clone(), contexts.clone()),
        ];
        assert!(RawTables::new(1, bad).is_err());
        let mut rev = triples.clone();
        rev.reverse();
        let bad = vec![(digest(0), t0, c0), (digest(1), rev, contexts)];
        assert!(RawTables::new(1, bad).is_err());
    }

    #[test]
    fn bucket_rule_edges() {
        assert_eq!(ceiling_bucket(&[1, 0, 0]).unwrap(), (0, 0));
        assert_eq!(ceiling_bucket(&[1 << 15, 0, 0]).unwrap(), (0, 15));
        assert_eq!(ceiling_bucket(&[9, (1 << 16) - 1, 0]).unwrap(), (1, 31));
        assert_eq!(ceiling_bucket(&[9, 3, 1 << 40]).unwrap(), (2, 47));
        assert_eq!(ceiling_bucket(&[9, 2, 0]).unwrap(), (1, 17));
        assert_eq!(ceiling_bucket(&[7, 0, 0]).unwrap(), (0, 2));
        assert!(ceiling_bucket(&[0, 0, 0]).is_err());
        assert_eq!(bucket_count(2), 48);
        assert_eq!(expert_count(2), 5);
    }

    /// A deterministic fixture at highest = 2 over buckets of h = 0, 1, 2.
    fn fixture() -> CeilingTable {
        let (kn, counts, totals) = fixture_inputs();
        CeilingTable::from_lookups(2, &kn, &counts, &totals).unwrap()
    }

    /// The fixture's KN masses, counts and totals.
    fn fixture_inputs() -> (Vec<u64>, Vec<u32>, Vec<u64>) {
        let (mut kn, mut counts, mut totals) = (Vec::new(), Vec::new(), Vec::new());
        let mut s = 12345u64;
        for t in 0..240u64 {
            s = s
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let h = (t % 3) as usize;
            let mut row_t = [0u64; 3];
            let mut row_a = [0u32; 3];
            for k in 0..=h {
                let total = 1 + ((s >> (8 * k + 3)) % 200);
                row_t[k] = total;
                row_a[k] = ((s >> (8 * k + 20)) % (total + 1)) as u32;
            }
            kn.push(1 + (s >> 20) % (1 << 44));
            counts.extend_from_slice(&row_a);
            totals.extend_from_slice(&row_t);
        }
        (kn, counts, totals)
    }

    fn assert_sums(table: &CeilingTable, weights: &[u64]) {
        let experts = table.experts();
        for b in 0..bucket_count(2) {
            let row = &weights[b * experts..(b + 1) * experts];
            let mut sum = 0u64;
            for (e, &w) in row.iter().enumerate() {
                if present(2, b, e) {
                    assert!(w >= 1, "bucket {b} expert {e} at 0");
                    sum += w;
                } else {
                    assert_eq!(w, 0, "absent expert {e} of bucket {b}");
                }
            }
            assert_eq!(sum, WEIGHT_ONE, "bucket {b}");
        }
    }

    #[test]
    fn weights_sum_to_one() {
        let table = fixture();
        let used: std::collections::BTreeSet<u16> = table.buckets().iter().copied().collect();
        assert!(
            used.iter()
                .map(|&b| b / 16)
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                >= 3
        );
        let fit: Vec<bool> = (0..table.targets()).map(|t| t % 5 != 0).collect();
        for iterations in 0..=8 {
            let result = table.fit_em(&fit, iterations, test_bits).unwrap();
            assert_eq!(result.fit_bits.len(), iterations as usize + 1);
            assert_sums(&table, &result.weights);
        }
    }

    #[test]
    fn fit_bits_never_rise() {
        let table = fixture();
        let fit = vec![true; table.targets()];
        let result = table.fit_em(&fit, 12, test_bits).unwrap();
        for w in result.fit_bits.windows(2) {
            assert!(w[1] <= w[0], "fit bits rose: {:?}", result.fit_bits);
        }
        assert!(result.fit_bits[12] < result.fit_bits[0]);
        let mixed = table.mixed_masses(&result.weights).unwrap();
        let again: u128 = mixed
            .iter()
            .map(|&m| u128::from(test_bits(m).unwrap()))
            .sum();
        assert_eq!(again, result.fit_bits[12]);
    }

    #[test]
    fn dominant_expert_takes_the_bucket() {
        // Bucket h = 2: ML_2 always sees the target (a = total); the others
        // give it little mass.
        let (mut kn, mut counts, mut totals) = (Vec::new(), Vec::new(), Vec::new());
        for t in 0..64u32 {
            kn.push(1 << 30);
            counts.extend_from_slice(&[1, 1, 40 + t % 3]);
            totals.extend_from_slice(&[1000, 900, u64::from(40 + t % 3)]);
        }
        let table = CeilingTable::from_lookups(2, &kn, &counts, &totals).unwrap();
        let b = usize::from(table.buckets()[0]);
        assert_eq!(b / 16, 2);
        let fit = vec![true; table.targets()];
        let result = table.fit_em(&fit, 10, test_bits).unwrap();
        assert!(result.weights[b * table.experts() + 3] > 1 << 31);
        assert_sums(&table, &result.weights);
    }

    #[test]
    fn mixed_masses_at_least_one() {
        // Target 0: KN 0 and every ML 0, so only U carries mass.
        let kn = [0u64, 1 << 40];
        let counts = [0u32, 0, 3, 2];
        let totals = [5u64, 0, 4, 2];
        let table = CeilingTable::from_lookups(1, &kn, &counts, &totals).unwrap();
        let fit = vec![true; 2];
        for iterations in [0, 1, 6] {
            let result = table.fit_em(&fit, iterations, test_bits).unwrap();
            let mixed = table.mixed_masses(&result.weights).unwrap();
            assert!(mixed.iter().all(|&m| m >= 1), "{mixed:?}");
        }
        // U at weight 1 alone gives KN_BASE_MASS >> 32 = 16, never 0.
        let mut weights = vec![0u64; bucket_count(1) * 4];
        let b = usize::from(table.buckets()[0]);
        weights[b * 4] = WEIGHT_ONE - 2;
        weights[b * 4 + 1] = 1;
        weights[b * 4 + 3] = 1;
        assert_eq!(table.mixed_masses(&weights).unwrap()[0], KN_BASE_MASS >> 32);
    }

    #[test]
    fn an_expert_mask_keeps_dropped_experts_at_zero() {
        let table = fixture();
        let experts = table.experts();
        let fit: Vec<bool> = (0..table.targets()).map(|t| t % 5 != 0).collect();
        // All experts on is fit_em.
        let all = vec![true; experts];
        assert_eq!(
            table.fit_em_with(&fit, &all, 6, test_bits).unwrap(),
            table.fit_em(&fit, 6, test_bits).unwrap()
        );
        // KN and U only: every ML weight stays 0, KN + U sum to one.
        let mut kn_u = vec![false; experts];
        kn_u[0] = true;
        kn_u[experts - 1] = true;
        for iterations in [0, 1, 6] {
            let result = table
                .fit_em_with(&fit, &kn_u, iterations, test_bits)
                .unwrap();
            for b in 0..bucket_count(2) {
                let row = &result.weights[b * experts..(b + 1) * experts];
                assert!(
                    row[1..experts - 1].iter().all(|&w| w == 0),
                    "bucket {b}: {row:?}"
                );
                assert!(row[0] >= 1 && row[experts - 1] >= 1, "bucket {b}: {row:?}");
                assert_eq!(row[0] + row[experts - 1], WEIGHT_ONE, "bucket {b}");
            }
        }
        // A wrong length and a mask that leaves a bucket nothing are refused.
        assert!(table.fit_em_with(&fit, &all[1..], 1, test_bits).is_err());
        let mut ml_2 = vec![false; experts];
        ml_2[3] = true;
        assert!(table.fit_em_with(&fit, &ml_2, 1, test_bits).is_err());
    }

    #[test]
    fn length_mismatches_are_refused() {
        assert!(CeilingTable::from_lookups(2, &[1, 2], &[0; 5], &[1; 6]).is_err());
        assert!(CeilingTable::from_lookups(2, &[1, 2], &[0; 6], &[1; 7]).is_err());
        let table = CeilingTable::from_lookups(2, &[1, 2], &[0; 6], &[1; 6]).unwrap();
        assert!(table.fit_em(&[true], 1, test_bits).is_err());
        assert!(table.mixed_masses(&[0; 3]).is_err());
        assert!(CeilingTable::from_lookups(1, &[1], &[0, 0], &[0, 0]).is_err());
    }

    /// claude-438: the KN start's weights over `on`, per bucket.
    fn assert_kn_start(table: &CeilingTable, on: &[bool], weights: &[u64]) {
        let experts = table.experts();
        for b in 0..bucket_count(2) {
            let row = &weights[b * experts..(b + 1) * experts];
            let live = (0..experts).filter(|&e| on[e] && present(2, b, e)).count() as u64;
            assert_eq!(row[0], WEIGHT_ONE - (live - 1), "bucket {b}: {row:?}");
            for (e, &w) in row.iter().enumerate().skip(1) {
                let want = u64::from(on[e] && present(2, b, e));
                assert_eq!(w, want, "bucket {b} expert {e}: {row:?}");
            }
        }
    }

    #[test]
    fn the_kn_start_puts_the_rest_on_kn() {
        let table = fixture();
        let experts = table.experts();
        let fit = vec![true; table.targets()];
        let all = vec![true; experts];
        let start = table
            .fit_em_from(&fit, &all, EmStart::Kn, 0, test_bits)
            .unwrap();
        assert_kn_start(&table, &all, &start.weights);
        let mut kn_u = vec![false; experts];
        kn_u[0] = true;
        kn_u[experts - 1] = true;
        let start = table
            .fit_em_from(&fit, &kn_u, EmStart::Kn, 0, test_bits)
            .unwrap();
        assert_kn_start(&table, &kn_u, &start.weights);
        // A mask without KN is refused; the uniform start is fit_em_with's.
        let mut no_kn = all.clone();
        no_kn[0] = false;
        assert!(table
            .fit_em_from(&fit, &no_kn, EmStart::Kn, 1, test_bits)
            .is_err());
        assert_eq!(
            table
                .fit_em_from(&fit, &all, EmStart::Uniform, 5, test_bits)
                .unwrap(),
            table.fit_em_with(&fit, &all, 5, test_bits).unwrap()
        );
    }

    #[test]
    fn the_kn_start_scores_kn_up_to_floors() {
        let table = fixture();
        let experts = table.experts();
        let fit: Vec<bool> = (0..table.targets()).map(|t| t % 7 != 3).collect();
        let fitted = fit.iter().filter(|&&f| f).count() as u128;
        let kn_bits: u128 = (0..table.targets())
            .filter(|&t| fit[t])
            .map(|t| u128::from(test_bits(table.masses()[t * experts]).unwrap()))
            .sum();
        let result = table
            .fit_em_from(&fit, &vec![true; experts], EmStart::Kn, 12, test_bits)
            .unwrap();
        assert!(
            result.fit_bits[0] <= kn_bits + fitted,
            "start {} against KN {kn_bits}",
            result.fit_bits[0]
        );
        for w in result.fit_bits.windows(2) {
            assert!(w[1] <= w[0], "fit bits rose: {:?}", result.fit_bits);
        }
        assert!(result.fit_bits[12] < result.fit_bits[0]);
        assert_sums(&table, &result.weights);
    }

    #[test]
    fn the_gap_bound_covers_what_em_still_saves() {
        let table = fixture();
        let experts = table.experts();
        let fit: Vec<bool> = (0..table.targets()).map(|t| t % 5 != 1).collect();
        let fitted = fit.iter().filter(|&&f| f).count() as u128;
        let mut kn_u = vec![false; experts];
        kn_u[0] = true;
        kn_u[experts - 1] = true;
        for on in [vec![true; experts], kn_u] {
            for start in [EmStart::Uniform, EmStart::Kn] {
                let long = table.fit_em_from(&fit, &on, start, 400, test_bits).unwrap();
                let best = *long.fit_bits.last().unwrap();
                for passes in [0u32, 1, 3, 10, 40] {
                    let at = table
                        .fit_em_from(&fit, &on, start, passes, test_bits)
                        .unwrap();
                    let gap = table
                        .gap_bound_q12(&at.weights, &fit, &on, test_bits)
                        .unwrap();
                    let scored = *at.fit_bits.last().unwrap();
                    assert!(
                        scored - best <= gap,
                        "{start:?} pass {passes}: saves {} beyond the bound {gap}",
                        scored - best
                    );
                }
                // Near the optimum the bound shrinks to its floor slack.
                let gap = table
                    .gap_bound_q12(&long.weights, &fit, &on, test_bits)
                    .unwrap();
                assert!(gap >= 3 * fitted && gap <= 6 * fitted, "{start:?}: {gap}");
            }
        }
        // From the uniform start, EM saves far more than the slack alone.
        let all = vec![true; experts];
        let first = table
            .fit_em_from(&fit, &all, EmStart::Uniform, 0, test_bits)
            .unwrap();
        let long = table
            .fit_em_from(&fit, &all, EmStart::Uniform, 400, test_bits)
            .unwrap();
        assert!(first.fit_bits[0] - long.fit_bits[400] > 20 * fitted);
        // Wrong lengths are refused.
        assert!(table
            .gap_bound_q12(&first.weights[1..], &fit, &all, test_bits)
            .is_err());
        assert!(table
            .gap_bound_q12(&first.weights, &fit[1..], &all, test_bits)
            .is_err());
    }

    /// claude-439: the fixture with a Pick column.
    fn pick_table(pick: &[PickTarget]) -> CeilingTable {
        let (kn, counts, totals) = fixture_inputs();
        CeilingTable::from_lookups_with_pick(2, &kn, &counts, &totals, pick).unwrap()
    }

    /// Classes 0, 1, 2 in runs of three, so every class meets every h; a
    /// fired target hits where `hit` says.
    fn pick_targets(hit: impl Fn(usize) -> bool) -> Vec<PickTarget> {
        (0..240)
            .map(|t| {
                let class = ((t / 3) % 3) as u8;
                PickTarget {
                    class,
                    hit: class != PICK_NOT_FIRED && hit(t),
                }
            })
            .collect()
    }

    #[test]
    fn a_pick_table_extends_the_plain_rows() {
        let plain = fixture();
        let pick = pick_targets(|t| t % 7 != 0);
        let table = pick_table(&pick);
        let (base, stride) = (plain.experts(), bucket_count(2));
        assert_eq!(table.experts(), base + 1);
        for (t, p) in pick.iter().enumerate() {
            let row = &table.masses()[t * (base + 1)..(t + 1) * (base + 1)];
            assert_eq!(&row[..base], &plain.masses()[t * base..(t + 1) * base], "target {t}");
            assert_eq!(row[base], if p.hit { 1 << 48 } else { 0 }, "target {t}");
            let want = usize::from(p.class) * stride + usize::from(plain.buckets()[t]);
            assert_eq!(usize::from(table.buckets()[t]), want, "target {t}");
        }
    }

    /// Sum of every weight times its index.
    fn weight_digest(weights: &[u64]) -> u128 {
        weights
            .iter()
            .enumerate()
            .map(|(i, &w)| u128::from(w) * i as u128)
            .sum()
    }

    #[test]
    fn a_table_without_pick_is_unchanged() {
        let plain = fixture();
        assert_eq!(plain.pick_expert(), None);
        assert_eq!(plain.bucket_total(), bucket_count(2));
        let fit = vec![true; plain.targets()];
        let all = vec![true; plain.experts()];
        let run = plain
            .fit_em_from(&fit, &all, EmStart::Kn, 24, test_bits)
            .unwrap();
        // Recorded on 9ae71b00a, before the Pick column.
        let want: [u128; 25] = [
            5386025, 5386018, 5384475, 5299439, 4907364, 3901407, 2436743, 1675294, 1375956,
            1235687, 1171471, 1155153, 1150895, 1148938, 1147701, 1146762, 1146027, 1145471,
            1145079, 1144825, 1144625, 1144497, 1144403, 1144323, 1144262,
        ];
        assert_eq!(run.fit_bits, want);
        assert_eq!(weight_digest(&run.weights), 24357983333462);
        // A Pick table whose targets all stay in class 0 fits the same.
        let table = pick_table(&vec![PickTarget { class: 0, hit: false }; 240]);
        let on = vec![true; table.experts()];
        let got = table
            .fit_em_from(&fit, &on, EmStart::Kn, 24, test_bits)
            .unwrap();
        assert_eq!(got.fit_bits, run.fit_bits);
        let (pe, xe) = (plain.experts(), table.experts());
        for b in 0..bucket_count(2) {
            assert_eq!(got.weights[b * xe + pe], 0, "bucket {b} holds Pick");
            assert_eq!(
                &got.weights[b * xe..b * xe + pe],
                &run.weights[b * pe..(b + 1) * pe],
                "bucket {b}"
            );
        }
    }

    #[test]
    fn the_pick_bucket_rule() {
        let plain = fixture();
        let pick = pick_targets(|t| t % 2 == 0);
        let table = pick_table(&pick);
        let stride = bucket_count(2);
        assert_eq!(table.bucket_total(), 3 * stride);
        assert_eq!(table.experts(), plain.experts() + 1);
        assert_eq!(table.pick_expert(), Some(expert_count(2)));
        let xe = table.experts();
        for (t, p) in pick.iter().enumerate() {
            let base = usize::from(plain.buckets()[t]);
            let want = usize::from(p.class) * stride + base;
            assert_eq!(usize::from(table.buckets()[t]), want, "target {t}");
            let mass = table.masses()[t * xe + xe - 1];
            assert_eq!(mass, if p.hit { 1 << 48 } else { 0 }, "target {t}");
        }
        assert!(pick.iter().any(|p| p.class == PICK_FIRED_EXACT));
        assert!(pick.iter().any(|p| p.class == PICK_FIRED_NEAR));
        for b in 0..table.bucket_total() {
            assert_eq!(table.holds(b, xe - 1), b >= stride, "bucket {b}");
            for e in 0..xe - 1 {
                assert_eq!(table.holds(b, e), present(2, b % stride, e));
            }
        }
        // At highest = 10: 3 * 176 buckets.
        let mut totals = vec![0u64; 11];
        totals[0] = 1;
        let one = [PickTarget { class: 2, hit: true }];
        let wide = CeilingTable::from_lookups_with_pick(10, &[1], &[0; 11], &totals, &one)
            .unwrap();
        assert_eq!(wide.bucket_total(), 528);
        assert_eq!(wide.buckets(), &[352]);
        // Bad Pick lists are refused.
        let (kn, counts, totals) = fixture_inputs();
        let build = |p: &[PickTarget]| {
            CeilingTable::from_lookups_with_pick(2, &kn, &counts, &totals, p).is_err()
        };
        assert!(build(&pick[1..]));
        let mut bad = pick.clone();
        bad[4].class = 3;
        assert!(build(&bad));
        let mut bad = pick.clone();
        bad[0] = PickTarget { class: 0, hit: true };
        assert!(build(&bad));
        assert!(!build(&pick));
    }

    #[test]
    fn a_right_copy_lowers_the_bits() {
        let plain = fixture();
        let fit = vec![true; plain.targets()];
        let all = vec![true; plain.experts()];
        let base = plain
            .fit_em_from(&fit, &all, EmStart::Kn, 24, test_bits)
            .unwrap();
        let table = pick_table(&pick_targets(|t| t % 7 != 0));
        let on = vec![true; table.experts()];
        let got = table
            .fit_em_from(&fit, &on, EmStart::Kn, 24, test_bits)
            .unwrap();
        assert!(got.fit_bits[24] < base.fit_bits[24], "{} vs {}", got.fit_bits[24], base.fit_bits[24]);
    }

    #[test]
    fn a_wrong_copy_falls_to_the_floor() {
        let table = pick_table(&pick_targets(|_| false));
        let fit = vec![true; table.targets()];
        let on = vec![true; table.experts()];
        let got = table
            .fit_em_from(&fit, &on, EmStart::Kn, 24, test_bits)
            .unwrap();
        let xe = table.experts();
        let fired: std::collections::BTreeSet<usize> = table
            .buckets()
            .iter()
            .map(|&b| usize::from(b))
            .filter(|&b| b >= bucket_count(2))
            .collect();
        assert!(!fired.is_empty());
        for b in fired {
            assert_eq!(got.weights[b * xe + xe - 1], 1, "bucket {b}");
        }
    }

    #[test]
    fn the_read_disabled_mask_matches_a_split_with_no_copy() {
        let plain = fixture();
        let pick = pick_targets(|t| t % 7 != 0);
        let table = pick_table(&pick);
        let xe = table.experts();
        let pe = plain.experts();
        let mut off = vec![true; xe];
        off[table.pick_expert().unwrap()] = false;
        let fit = vec![true; table.targets()];
        let got = table
            .fit_em_from(&fit, &off, EmStart::Kn, 24, test_bits)
            .unwrap();
        for b in 0..table.bucket_total() {
            assert_eq!(got.weights[b * xe + xe - 1], 0, "bucket {b}");
        }
        // Each class's buckets fit as the plain table fits that class alone.
        let stride = bucket_count(2);
        for class in 0..PICK_CLASSES {
            let only: Vec<bool> = pick.iter().map(|p| p.class == class).collect();
            let split = plain
                .fit_em_from(&only, &vec![true; pe], EmStart::Kn, 24, test_bits)
                .unwrap();
            for b in 0..stride {
                let at = usize::from(class) * stride + b;
                assert_eq!(
                    &got.weights[at * xe..at * xe + pe],
                    &split.weights[b * pe..(b + 1) * pe],
                    "class {class} bucket {b}"
                );
            }
        }
    }

    #[test]
    fn pick_rows_sum_to_one() {
        let table = pick_table(&pick_targets(|t| t % 3 != 0));
        let fit = vec![true; table.targets()];
        let on = vec![true; table.experts()];
        let xe = table.experts();
        for start in [EmStart::Uniform, EmStart::Kn] {
            for passes in [0, 6] {
                let got = table
                    .fit_em_from(&fit, &on, start, passes, test_bits)
                    .unwrap();
                assert_eq!(got.weights.len(), table.bucket_total() * xe);
                for b in 0..table.bucket_total() {
                    let row = &got.weights[b * xe..(b + 1) * xe];
                    for (e, &w) in row.iter().enumerate() {
                        if table.holds(b, e) {
                            assert!(w >= 1, "{start:?} bucket {b} expert {e} at 0");
                        } else {
                            assert_eq!(w, 0, "{start:?} absent expert {e} of bucket {b}");
                        }
                    }
                    assert_eq!(row.iter().sum::<u64>(), WEIGHT_ONE, "{start:?} bucket {b}");
                }
                assert!(table.mixed_masses(&got.weights).is_ok());
                assert!(table
                    .gap_bound_q12(&got.weights, &fit, &on, test_bits)
                    .is_ok());
            }
        }
    }
}
