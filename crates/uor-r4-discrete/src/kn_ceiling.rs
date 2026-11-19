//! claude-433: the count-mixer ceiling. Raw count tables at every order give
//! each query target several count experts (KN, the maximum-likelihood mass
//! at each seen order, and the uniform base), and a bucketed linear mix of
//! them is fit by integer EM. All masses are Q48 and all weights Q32.
//!
//! The lookup reads raw tables with exactly `KnTables`'s key derivation,
//! binary searches and order range, so a CUDA kernel can later produce the
//! same flat (count, total) layout. Bits are taken by a caller-passed
//! function, as `bits_q12` lives in `uor-r4-integer`.

use crate::kn_score::{KN_BASE_MASS, PROBABILITY_ONE};
use crate::ngram_count::{
    NgramContext, NgramOrderDigest, NgramTriple, PrimeRouteKey, NGRAM_BOS, NGRAM_MAX_ORDER,
    NGRAM_VOCABULARY,
};
use crate::{Error, Result};

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

/// Every target's bucket and its expert masses.
pub struct CeilingTable {
    highest: u32,
    buckets: Vec<u16>,
    masses: Vec<u64>,
}

impl CeilingTable {
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
            let (h, bucket) = ceiling_bucket(row_totals)?;
            let row = &mut masses[t * experts..(t + 1) * experts];
            row[0] = kn;
            for k in 0..=h {
                let (a, total) = (row_counts[k], row_totals[k]);
                if total == 0 {
                    continue;
                }
                if u64::from(a) > total {
                    return Err(invalid(format!(
                        "target {t} order {k} count {a} exceeds its total {total}"
                    )));
                }
                let ml = (u128::from(a) << 48) / u128::from(total);
                row[1 + k] = u64::try_from(ml).map_err(|_| invalid("an ML mass exceeded 2^64"))?;
            }
            row[experts - 1] = KN_BASE_MASS;
            buckets.push(bucket);
        }
        debug_assert!(KN_BASE_MASS < PROBABILITY_ONE);
        Ok(Self {
            highest,
            buckets,
            masses,
        })
    }

    pub fn experts(&self) -> usize {
        expert_count(self.highest)
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
        let mut weights = vec![0u64; bucket_count(self.highest) * experts];
        for b in 0..bucket_count(self.highest) {
            let row = &mut weights[b * experts..(b + 1) * experts];
            let live: Vec<usize> = (0..experts)
                .filter(|&e| on[e] && present(self.highest, b, e))
                .collect();
            let share = WEIGHT_ONE / live.len() as u64;
            for &e in &live {
                row[e] = share;
            }
            row[live[0]] += WEIGHT_ONE - share * live.len() as u64;
        }
        weights
    }

    /// Target t's unshifted mix sum of w_e * p_e.
    fn mix_sum(&self, weights: &[u64], t: usize) -> u128 {
        let experts = self.experts();
        let b = usize::from(self.buckets[t]);
        let w = &weights[b * experts..(b + 1) * experts];
        let p = &self.masses[t * experts..(t + 1) * experts];
        w.iter()
            .zip(p)
            .map(|(&w, &p)| u128::from(w) * u128::from(p))
            .sum()
    }

    fn mixed(&self, weights: &[u64], t: usize) -> u64 {
        let m = self.mix_sum(weights, t) >> 32;
        u64::try_from(m).unwrap_or(u64::MAX).max(1)
    }

    /// Every target's mixed Q48 mass, at least 1.
    pub fn mixed_masses(&self, weights: &[u64]) -> Result<Vec<u64>> {
        self.check_weights(weights)?;
        Ok((0..self.targets())
            .map(|t| self.mixed(weights, t))
            .collect())
    }

    fn check_weights(&self, weights: &[u64]) -> Result<()> {
        let want = bucket_count(self.highest) * self.experts();
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
        let on = experts;
        if on.len() != self.experts() {
            return Err(invalid(format!(
                "the expert mask holds {} entries, not {}",
                on.len(),
                self.experts()
            )));
        }
        if let Some(b) = (0..bucket_count(self.highest))
            .find(|&b| !(0..on.len()).any(|e| on[e] && present(self.highest, b, e)))
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
        let experts = self.experts();
        let buckets = bucket_count(self.highest);
        let mut weights = self.uniform_weights_with(on);
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
            if on[e] && present(self.highest, b, e) && row[e] == 0 {
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
        CeilingTable::from_lookups(2, &kn, &counts, &totals).unwrap()
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
}
