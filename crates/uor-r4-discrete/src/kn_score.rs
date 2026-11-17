//! Integer interpolated modified Kneser–Ney scorer over the exact n-gram
//! counter's adjusted counts (claude-419). Host only; the GPU scorer of
//! claude-420 must equal it. Every quantity is an integer: masses are Q48
//! (`PROBABILITY_ONE = 2^48`), discounts are quantized once per order, and the
//! only operations are add, subtract, multiply, floor-divide and binary search.
//! No exponential, no normalizer, no squashing and no matrix product.
//!
//! The rule, for order k (k context tokens) and highest order N:
//! - `Dq_i = floor(M (i n_i S - (i+1) n1 n_{i+1}) / (n_i S))`, `S = n1 + 2 n2`;
//!   an order with a zero n_i or a `Dq_i` outside `[0, i M]` takes KenLM's
//!   fallback `(M/2, M, 3M/2)` for all three (claude-421);
//! - `P_k = floor((a M - D(a)) / A) + floor(gamma P_{k-1} / M)`, with
//!   `gamma = floor((Dq1 n1(h) + Dq2 n2(h) + Dq3 n3+(h)) / A)`;
//! - a context with no record (A = 0) gives `P_k = P_{k-1}`;
//! - `P_{-1} = M / 4096`;
//! - the result is at least 1 (claude-422): a gamma chain whose floors reach
//!   0 is floored at one Q48 unit, 2^-48. Real KN gives such a target a mass
//!   below 2^-48, so the floor can only lower R\*'s bits; it adds at most 4,096
//!   units (2^-36) to a history's sum.
//!
//! The BOS rule of claude-415 needs no separate test here. The counter never
//! writes a context with BOS outside its oldest slot, so such a context has no
//! record and backs off. A BOS target has a = 0 at every order and gets the
//! gamma chain's mass, so the 4,096 masses of a history sum to M up to the
//! floors. A scored stream never scores a BOS target: `score_target` refuses one.
//!
//! Bits are not taken here. `uor-r4-integer`'s `lawful_likelihood::bits_q12`
//! is compiled out of this crate's test build (the `uor-r4-cpu-train`
//! dev-dependency turns on `serving-wasm`), so the scorer hands exact Q48
//! masses to the caller, and claude-420's instrument takes `bits_q12` of them.

use crate::ngram_count::{
    NgramContext, NgramOrderDigest, NgramTriple, PrimeRouteKey, NGRAM_BOS, NGRAM_MAX_ORDER,
    NGRAM_VOCABULARY,
};
use crate::{Error, Result};

/// `2^48`, the Q48 unit; equal to `lawful_likelihood::PROBABILITY_ONE`.
pub const PROBABILITY_ONE: u64 = 1 << 48;

/// The uniform base mass `P_{-1}`: 2^48 / 4096 = 2^36.
pub const KN_BASE_MASS: u64 = PROBABILITY_ONE / NGRAM_VOCABULARY as u64;

fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(message.into())
}

/// One order's three discounts `Dq_1, Dq_2, Dq_3+` in Q48.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnDiscounts {
    pub dq: [u64; 3],
}

impl KnDiscounts {
    /// KenLM's `--discount_fallback` default (1/2, 1, 3/2) in Q48: an order
    /// whose spectrum gives no in-range Chen–Goodman estimate takes these
    /// three discounts whole (claude-421; the order-0 continuation spectrum of
    /// the whole D87 TRAIN stream is n1..n4 = [7, 6, 11, 11], so D2 < 0).
    pub const FALLBACK: Self = Self {
        dq: [
            PROBABILITY_ONE / 2,
            PROBABILITY_ONE,
            PROBABILITY_ONE / 2 * 3,
        ],
    };

    /// Chen–Goodman's estimates from one order's count-of-counts n1..n4,
    /// floored once to Q48. Refuses a zero n_i and a discount outside
    /// `[0, i M]`.
    pub fn from_count_of_counts(count_of_counts: [u64; 4]) -> Result<Self> {
        Self::chen_goodman(count_of_counts)?
            .map(|dq| Self { dq })
            .map_err(invalid)
    }

    /// The order's discounts and whether they are `FALLBACK`: Chen–Goodman
    /// when every n_i > 0 and every estimate lies in `[0, i M]`, else the
    /// fallback for the whole order, as KenLM does. Overflow is refused, but
    /// only in the estimates computed before the first degenerate one (as in
    /// KenLM's order of checks); that needs counts near 2^60.
    pub fn with_fallback(count_of_counts: [u64; 4]) -> Result<(Self, bool)> {
        Ok(match Self::chen_goodman(count_of_counts)? {
            Ok(dq) => (Self { dq }, false),
            Err(_) => (Self::FALLBACK, true),
        })
    }

    /// The outer error is overflow; the inner one is a degenerate spectrum.
    fn chen_goodman(count_of_counts: [u64; 4]) -> Result<std::result::Result<[u64; 3], String>> {
        if count_of_counts.contains(&0) {
            return Ok(Err(format!(
                "modified-KN discounts need n1..n4 > 0, got {count_of_counts:?}"
            )));
        }
        let n: [i128; 4] = count_of_counts.map(i128::from);
        let overflow = || invalid("modified-KN discount arithmetic overflowed");
        let s = n[0]
            .checked_add(n[1].checked_mul(2).ok_or_else(overflow)?)
            .ok_or_else(overflow)?;
        let m = i128::from(PROBABILITY_ONE);
        let mut dq = [0u64; 3];
        for i in 1..=3usize {
            let ii = i as i128;
            let kept = ii
                .checked_mul(n[i - 1])
                .and_then(|x| x.checked_mul(s))
                .ok_or_else(overflow)?;
            let taken = (ii + 1)
                .checked_mul(n[0])
                .and_then(|x| x.checked_mul(n[i]))
                .ok_or_else(overflow)?;
            let numerator = m.checked_mul(kept - taken).ok_or_else(overflow)?;
            let denominator = n[i - 1].checked_mul(s).ok_or_else(overflow)?;
            let discount = numerator.div_euclid(denominator);
            if discount < 0 || discount > ii * m {
                return Ok(Err(format!(
                    "modified-KN discount D{i} = {discount} lies outside [0, {i} * 2^48] for n1..n4 {count_of_counts:?}"
                )));
            }
            dq[i - 1] = discount as u64;
        }
        Ok(Ok(dq))
    }

    /// `D(a)`: 0 for an unseen gram, then `Dq_min(a, 3)`.
    pub fn discount(&self, a: u32) -> u64 {
        match a {
            0 => 0,
            1 => self.dq[0],
            2 => self.dq[1],
            _ => self.dq[2],
        }
    }
}

struct KnOrder {
    discounts: KnDiscounts,
    /// True when `discounts` is `KnDiscounts::FALLBACK`.
    fallback: bool,
    triples: Vec<NgramTriple>,
    contexts: Vec<NgramContext>,
}

/// One table per order 0..=N, read by binary search in the counter's
/// canonical order (key, then next).
pub struct KnTables {
    highest: u32,
    orders: Vec<KnOrder>,
}

impl KnTables {
    /// Takes the assembler outputs of orders 0..=highest, in order: adjusted
    /// below `highest`, raw at `highest`.
    pub fn new(
        highest: u32,
        orders: Vec<(NgramOrderDigest, Vec<NgramTriple>, Vec<NgramContext>)>,
    ) -> Result<Self> {
        if highest > NGRAM_MAX_ORDER || orders.len() != highest as usize + 1 {
            return Err(invalid(format!(
                "KN tables need orders 0..={highest} (at most {NGRAM_MAX_ORDER}), got {} tables",
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
            let wrong_order = triples
                .iter()
                .map(|t| t.key)
                .chain(contexts.iter().map(|c| c.key));
            for key in wrong_order {
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
            let (discounts, fallback) = KnDiscounts::with_fallback(digest.count_of_counts)?;
            tables.push(KnOrder {
                discounts,
                fallback,
                triples,
                contexts,
            });
        }
        Ok(Self {
            highest,
            orders: tables,
        })
    }

    pub fn highest(&self) -> u32 {
        self.highest
    }

    /// Order k's discounts, triples and contexts, in canonical order, for the
    /// GPU scorer's upload (claude-421).
    pub fn order_tables(&self, k: u32) -> Option<(KnDiscounts, &[NgramTriple], &[NgramContext])> {
        let order = self.orders.get(k as usize)?;
        Some((order.discounts, &order.triples, &order.contexts))
    }

    /// Whether order k took `KnDiscounts::FALLBACK`.
    pub fn discount_fallback(&self, k: u32) -> Option<bool> {
        self.orders.get(k as usize).map(|order| order.fallback)
    }

    /// The Q48 mass of the target `tokens[j]` after its history, at least 1.
    /// Refuses a BOS target, which a stream never scores.
    pub fn score_target(&self, tokens: &[u16], j: usize) -> Result<u64> {
        let target = *tokens
            .get(j)
            .ok_or_else(|| invalid(format!("target {j} is past the stream")))?;
        if target == NGRAM_BOS {
            return Err(invalid(format!("target {j} is BOS, which is never scored")));
        }
        self.mass(tokens, j)
    }

    /// The interpolation itself, for any target id including BOS.
    fn mass(&self, tokens: &[u16], j: usize) -> Result<u64> {
        let target = tokens[j];
        if u32::from(target) >= NGRAM_VOCABULARY {
            return Err(invalid(format!(
                "token id {target} is outside the 12-bit vocabulary"
            )));
        }
        let m = u128::from(PROBABILITY_ONE);
        let mut mass = u128::from(KN_BASE_MASS);
        for k in 0..=(self.highest as usize).min(j) {
            let order = &self.orders[k];
            let key = PrimeRouteKey::from_context(&tokens[j - k..j])?.word();
            let Ok(at) = order.contexts.binary_search_by(|c| c.key.cmp(&key)) else {
                continue;
            };
            let NgramContext {
                total,
                n1,
                n2,
                n3_plus,
                ..
            } = order.contexts[at];
            let a = order
                .triples
                .binary_search_by(|t| (t.key, t.next).cmp(&(key, target)))
                .map_or(0, |i| order.triples[i].count);
            let [d1, d2, d3] = order.discounts.dq.map(u128::from);
            let total = u128::from(total);
            let gamma =
                (d1 * u128::from(n1) + d2 * u128::from(n2) + d3 * u128::from(n3_plus)) / total;
            let kept = u128::from(a) * m - u128::from(order.discounts.discount(a));
            mass = kept / total + gamma * mass / m;
        }
        u64::try_from(mass.max(1)).map_err(|_| invalid("a KN mass exceeded 2^64"))
    }

    /// The Q48 mass of every non-BOS target of `tokens`, in stream order.
    pub fn scored_masses(&self, tokens: &[u16]) -> Result<Vec<u64>> {
        (0..tokens.len())
            .filter(|&j| tokens[j] != NGRAM_BOS)
            .map(|j| self.score_target(tokens, j))
            .collect()
    }
}

/// claude-422: N = 0 tables whose gamma chain floors an unseen id to 0. One
/// order-0 context of total 2^40 has a single follower, id 5 with count 2^20.
/// With n1..n4 = [2, 1, 1, 1], Dq = [M/2, M/2, M], so gamma = Dq3 / 2^40 = 2^8
/// and an unseen id gets floor(2^8 * 2^36 / 2^48) = 0 before the floor.
/// Shared with the CUDA tests.
#[cfg(test)]
pub(crate) fn kn_underflow_tables() -> KnTables {
    let key = PrimeRouteKey::from_context(&[])
        .expect("empty context")
        .word();
    let digest = NgramOrderDigest {
        order: 0,
        targets: 1 << 20,
        grams: 1,
        contexts: 1,
        count_of_counts: [2, 1, 1, 1],
        triple_sha256: String::new(),
        context_sha256: String::new(),
    };
    let triple = NgramTriple {
        key,
        next: 5,
        count: 1 << 20,
    };
    let context = NgramContext {
        key,
        total: 1 << 40,
        followers: 1,
        n1: 0,
        n2: 0,
        n3_plus: 1,
    };
    KnTables::new(0, vec![(digest, vec![triple], vec![context])]).expect("underflow tables")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kn_score_oracle::KnOracle;
    use crate::ngram_count_oracle::oracle_adjusted;

    const M: u128 = 1 << 48;

    /// A text-like stream: ids with a heavy head and a long tail
    /// (5 + a product of four uniforms on 0..16, over 16), one draw in 61 a
    /// BOS, and one draw in 9 a copy of 2..9 tokens from an earlier position,
    /// as text repeats phrases. Copied spans get copied again, so long grams
    /// repeat with skewed counts and every order 0..4 has a falling spectrum
    /// n1 > n2 > n3 > n4 > 0.
    fn kn_score_stream(seed: u64, len: usize) -> Vec<u16> {
        let mut state = seed | 1;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 16) as u32
        };
        let mut stream: Vec<u16> = vec![NGRAM_BOS];
        while stream.len() < len {
            let r = next();
            if (r >> 16) % 9 == 0 && stream.len() > 16 {
                let from = next() as usize % (stream.len() - 9);
                let run = 2 + (r >> 20) as usize % 8;
                for at in from..from + run {
                    stream.push(stream[at]);
                }
                continue;
            }
            let product = (r % 16) * ((r >> 4) % 16) * ((r >> 8) % 16) * ((r >> 12) % 16);
            stream.push(if (r >> 24) % 61 == 0 {
                NGRAM_BOS
            } else {
                (5 + product / 16) as u16
            });
        }
        stream.truncate(len);
        stream[0] = NGRAM_BOS;
        stream
    }

    fn tables_from_oracle(tokens: &[u16], highest: u32) -> KnTables {
        let orders = (0..=highest)
            .map(|k| oracle_adjusted(tokens, k, highest).expect("adjusted counts"))
            .collect::<Vec<_>>();
        for o in &orders {
            let (digest, _, _) = o;
            eprintln!(
                "spectrum N {highest} order {}: n1..n4 {:?}",
                digest.order, digest.count_of_counts
            );
        }
        KnTables::new(highest, orders).expect("tables")
    }

    #[test]
    fn kn_score_discounts_refuse_zero_and_out_of_range_counts() {
        assert!(KnDiscounts::from_count_of_counts([0, 1, 1, 1]).is_err());
        assert!(KnDiscounts::from_count_of_counts([1, 1, 1, 0]).is_err());
        // S = 3; D2's numerator is 2*1*3 - 3*1*100 < 0.
        assert!(KnDiscounts::from_count_of_counts([1, 1, 100, 1]).is_err());
        assert!(KnDiscounts::from_count_of_counts([u64::MAX; 4]).is_err());
        // n = [2,1,1,1]: S = 4, D1 = M*4/8, D2 = M*2/4, D3 = M*4/4.
        let d = KnDiscounts::from_count_of_counts([2, 1, 1, 1]).expect("discounts");
        assert_eq!(d.dq, [(M / 2) as u64, (M / 2) as u64, M as u64]);
        // Chen–Goodman on a typical spectrum stays inside [0, i].
        let d =
            KnDiscounts::from_count_of_counts([4_132_730, 413_614, 122_848, 56_120]).expect("real");
        assert!(d.dq[0] < M as u64 && d.dq[1] < 2 * M as u64 && d.dq[2] < 3 * M as u64);
    }

    #[test]
    fn kn_score_tables_refuse_bad_shapes() {
        let tokens = kn_score_stream(0x4190, 3_000);
        let mut orders: Vec<_> = (0..=2u32)
            .map(|k| oracle_adjusted(&tokens, k, 2).expect("adjusted"))
            .collect();
        assert!(KnTables::new(3, orders.clone()).is_err(), "too few tables");
        orders.swap(0, 1);
        assert!(
            KnTables::new(2, orders.clone()).is_err(),
            "orders out of place"
        );
        orders.swap(0, 1);
        let (_, order_one, _) = &mut orders[1];
        order_one.swap(0, 1);
        assert!(KnTables::new(2, orders).is_err(), "unsorted triples");
        let tables = tables_from_oracle(&tokens, 2);
        let bos = tokens.iter().position(|&t| t == NGRAM_BOS).expect("bos");
        assert!(tables.score_target(&tokens, bos).is_err());
        assert!(tables.score_target(&tokens, tokens.len()).is_err());
    }

    /// claude-422: an unseen id whose chain floors to 0 scores one Q48 unit;
    /// the seen id scores (2^20 M - M) / 2^40 = (2^20 - 1) 2^8, its gamma
    /// share 2^8 * 2^36 / 2^48 flooring to 0.
    #[test]
    fn kn_score_floors_an_underflowed_chain_at_one_unit() {
        let tables = kn_underflow_tables();
        let masses = tables.scored_masses(&[5, 7, 5]).expect("masses");
        assert_eq!(masses, vec![((1 << 20) - 1) << 8, 1, ((1 << 20) - 1) << 8]);
    }

    /// N = 2 on 12 tokens, every mass derived by hand. The count-of-counts of
    /// every order are set to [2,1,1,1] so that Dq = [M/2, M/2, M]; the
    /// counts themselves come from the counter's oracle and are checked first.
    #[test]
    fn kn_score_hand_fixture() {
        //            j: 0  1  2  3  4  5  6  7  8  9 10 11
        let t: [u16; 12] = [0, 5, 6, 5, 6, 5, 7, 0, 6, 5, 6, 8];
        let mut orders: Vec<_> = (0..=2u32)
            .map(|k| oracle_adjusted(&t, k, 2).expect("adjusted"))
            .collect();
        let a = |k: usize, ctx: &[u16], w: u16| {
            let key = PrimeRouteKey::from_context(ctx).expect("key").word();
            let (_, triples, _) = &orders[k];
            triples
                .iter()
                .find(|x| x.key == key && x.next == w)
                .map_or(0, |x| x.count)
        };
        // Order 0, continuation counts: 5 has left {0,6}, 6 has {5,0}, 7 {5}, 8 {6}.
        assert_eq!(
            [a(0, &[], 5), a(0, &[], 6), a(0, &[], 7), a(0, &[], 8)],
            [2, 2, 1, 1]
        );
        // Order 1: ctx 0 is a document start (raw); 5 -> {6: left {0,6}, 7: {6}}; 6 -> {5: {5,0}, 8: {5}}.
        assert_eq!([a(1, &[0], 5), a(1, &[0], 6)], [1, 1]);
        assert_eq!(
            [a(1, &[5], 6), a(1, &[5], 7), a(1, &[6], 5), a(1, &[6], 8)],
            [2, 1, 2, 1]
        );
        // Order 2 = N, raw; (7,0) is not admitted, so j = 8 has no order-2 gram.
        assert_eq!(
            [a(2, &[0, 5], 6), a(2, &[5, 6], 5), a(2, &[5, 6], 8)],
            [1, 2, 1]
        );
        assert_eq!(
            [
                a(2, &[6, 5], 6),
                a(2, &[6, 5], 7),
                a(2, &[0, 6], 5),
                a(2, &[7, 0], 6)
            ],
            [2, 1, 1, 0]
        );
        for order in &mut orders {
            order.0.count_of_counts = [2, 1, 1, 1];
        }
        let tables = KnTables::new(2, orders).expect("tables");

        let h = M / 2;
        let base = M / 4096;
        // Order 0: A = 6, n1(h) = 2 (7, 8), n2(h) = 2 (5, 6): gamma = (h*2 + h*2) / 6.
        let g0 = (2 * h + 2 * h) / 6;
        let q0 = g0 * base / M;
        let p0_56 = (2 * M - h) / 6 + q0;
        let p0_78 = (M - h) / 6 + q0;
        let p0_unseen = q0;
        // Order 1. ctx 0: A = 2, n1 = 2, gamma = 2h / 2 = h.
        let p1 = |a: u128, total: u128, gamma: u128, lower: u128| {
            (a * M - if a == 0 { 0 } else { h }) / total + gamma * lower / M
        };
        let g_0 = 2 * h / 2;
        let p1_5_0 = p1(1, 2, g_0, p0_56);
        let p1_6_0 = p1(1, 2, g_0, p0_56);
        // ctx 5: A = 3, {6:2, 7:1}, n1 = n2 = 1, gamma = 2h / 3; ctx 6 likewise with {5:2, 8:1}.
        let g_3 = (h + h) / 3;
        let p1_6_5 = p1(2, 3, g_3, p0_56);
        let p1_7_5 = p1(1, 3, g_3, p0_78);
        let p1_5_6 = p1(2, 3, g_3, p0_56);
        let p1_8_6 = p1(1, 3, g_3, p0_78);
        let p1_9_6 = p1(0, 3, g_3, p0_unseen);
        // Order 2. (0,5): A = 1 {6:1}, gamma = h; (0,6): A = 1 {5:1}, gamma = h;
        // (5,6): A = 3 {5:2, 8:1}; (6,5): A = 3 {6:2, 7:1}; both gamma = 2h / 3.
        let g_1 = h;
        let want: [(usize, u128); 10] = [
            (1, p1_5_0), // j = 1 < 2: no order-2 context, order 1 after BOS
            (2, p1(1, 1, g_1, p1_6_5)),
            (3, p1(2, 3, g_3, p1_5_6)),
            (4, p1(2, 3, g_3, p1_6_5)),
            (5, p1(2, 3, g_3, p1_5_6)),
            (6, p1(1, 3, g_3, p1_7_5)),
            (8, p1_6_0), // (7,0) holds BOS in slot 1: no record, back off to ctx 0
            (9, p1(1, 1, g_1, p1_5_6)),
            (10, p1(2, 3, g_3, p1_6_5)),
            (11, p1(1, 3, g_3, p1_8_6)),
        ];
        for (j, mass) in want {
            assert_eq!(
                u128::from(tables.score_target(&t, j).expect("mass")),
                mass,
                "target {j}"
            );
        }
        // Unseen contexts back off: (6,6) -> ctx 6; (0,9) and (9) -> order 0.
        assert_eq!(
            u128::from(tables.score_target(&[0, 6, 6, 5], 3).expect("mass")),
            p1_5_6
        );
        assert_eq!(
            u128::from(tables.score_target(&[0, 9, 5], 2).expect("mass")),
            p0_56
        );
        // An unseen word after a seen context gets the gamma chain only.
        assert_eq!(
            u128::from(tables.score_target(&[0, 5, 6, 9], 3).expect("mass")),
            g_3 * p1_9_6 / M
        );
        let scored: Vec<u128> = tables
            .scored_masses(&t)
            .expect("masses")
            .into_iter()
            .map(u128::from)
            .collect();
        let wanted: Vec<u128> = want.iter().map(|&(_, mass)| mass).collect();
        assert_eq!(scored, wanted);
    }

    /// N = 0: order-0 KN against the uniform base alone, by hand. On the
    /// fixture's raw order-0 counts (5:4, 6:4, 7:1, 8:1; A = 10; n1(h) = 2,
    /// n2(h) = 0, n3+(h) = 2) with Dq = [M/2, M/2, M], the a = 4 targets take
    /// the Dq3 branch.
    #[test]
    fn kn_score_order_zero_against_uniform() {
        let t: [u16; 12] = [0, 5, 6, 5, 6, 5, 7, 0, 6, 5, 6, 8];
        let (mut digest, triples, contexts) = oracle_adjusted(&t, 0, 0).expect("order 0");
        let counts: Vec<(u16, u32)> = triples.iter().map(|x| (x.next, x.count)).collect();
        assert_eq!(counts, [(5, 4), (6, 4), (7, 1), (8, 1)]);
        digest.count_of_counts = [2, 1, 1, 1];
        let tables = KnTables::new(0, vec![(digest, triples, contexts)]).expect("tables");
        let h = M / 2;
        // gamma = (h*2 + h*0 + M*2) / 10; the base share of every id is gamma * (M/4096) / M.
        let gamma = (2 * h + 2 * M) / 10;
        let share = gamma * (M / 4096) / M;
        let p_frequent = (4 * M - M) / 10 + share;
        let p_once = (M - h) / 10 + share;
        for (j, want) in [(1, p_frequent), (2, p_frequent), (6, p_once), (11, p_once)] {
            assert_eq!(
                u128::from(tables.score_target(&t, j).expect("mass")),
                want,
                "target {j}"
            );
        }
        assert_eq!(
            u128::from(tables.score_target(&[0, 9], 1).expect("mass")),
            share
        );
        let sum: u128 = (0..4096u16)
            .map(|w| u128::from(tables.mass(&[w], 0).expect("mass")))
            .sum();
        assert!((M - 3 * 4096..=M).contains(&sum), "sum {sum}");
    }

    /// Every history's 4,096 masses sum to M up to the floors: less than three
    /// units per order per target.
    #[test]
    fn kn_score_masses_sum_to_one_up_to_the_floors() {
        for highest in 1..=4u32 {
            let tokens = kn_score_stream(0x4191 + u64::from(highest), 6_000);
            let tables = tables_from_oracle(&tokens, highest);
            let n = highest as usize;
            let mut histories: Vec<Vec<u16>> = (0..tokens.len())
                .map(|j| tokens[j.saturating_sub(n)..j].to_vec())
                .collect();
            histories.extend([
                vec![],
                vec![9, 9, 9, 9][..n].to_vec(),
                vec![0, 300, 5, 300][..n].to_vec(),
            ]);
            histories.sort();
            histories.dedup();
            let floor = M - 3 * (u128::from(highest) + 1) * 4096;
            let (mut checked, mut worst) = (0, 0u128);
            for history in &histories {
                let mut slice = history.clone();
                slice.push(0);
                let j = history.len();
                let mut sum: u128 = 0;
                for w in 0..4096u16 {
                    slice[j] = w;
                    sum += u128::from(tables.mass(&slice, j).expect("mass"));
                }
                assert!(
                    sum <= M && sum >= floor,
                    "N {highest} history {history:?}: sum {sum}"
                );
                checked += 1;
                worst = worst.max(M - sum);
            }
            assert!(checked > 100, "only {checked} histories");
            eprintln!(
                "N {highest}: {checked} histories, largest deficit {worst} units, bound {}",
                M - floor
            );
        }
    }

    /// The table scorer equals the independent oracle on every target, one
    /// by one and through `scored_masses`.
    #[test]
    fn kn_score_matches_the_oracle_on_seeded_streams() {
        for seed in [0x419a_u64, 0x419b, 0x419c] {
            let tokens = kn_score_stream(seed, 6_000);
            for highest in 1..=4u32 {
                let orders: Vec<_> = (0..=highest)
                    .map(|k| oracle_adjusted(&tokens, k, highest).expect("adjusted"))
                    .collect();
                let tables = KnTables::new(highest, orders).expect("tables");
                let oracle = KnOracle::new(&tokens, highest as usize);
                let mut wanted = Vec::new();
                for j in 0..tokens.len() {
                    if tokens[j] == NGRAM_BOS {
                        continue;
                    }
                    let want = oracle.mass(&tokens, j);
                    let got = tables.score_target(&tokens, j).expect("mass");
                    assert_eq!(
                        u128::from(got),
                        want,
                        "seed {seed:x} N {highest} target {j}"
                    );
                    wanted.push(want as u64);
                }
                assert_eq!(tables.scored_masses(&tokens).expect("masses"), wanted);
            }
        }
    }

    /// 40 ids drawn uniformly, one draw in 300 a BOS: every unigram and
    /// bigram follows many distinct tokens, so orders 0 and 1 have no
    /// continuation counts 1..=4, while the raw top order's spectrum falls.
    fn kn_score_small_vocabulary_stream(seed: u64, len: usize) -> Vec<u16> {
        let mut state = seed | 1;
        let mut stream = vec![NGRAM_BOS];
        while stream.len() < len {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let r = (state >> 16) as u32;
            stream.push(if r.is_multiple_of(300) {
                NGRAM_BOS
            } else {
                (5 + (r >> 9) % 40) as u16
            });
        }
        stream
    }

    /// claude-421: an order with a degenerate spectrum takes KenLM's
    /// fallback discounts whole, the oracle does the same on its own counts,
    /// and the two agree on every target.
    #[test]
    fn kn_score_falls_back_on_degenerate_spectra_as_the_oracle_does() {
        let (fallback, fell_back) =
            KnDiscounts::with_fallback([7, 6, 11, 11]).expect("D87 order 0");
        assert!(fell_back);
        assert_eq!(fallback.dq, [M as u64 / 2, M as u64, 3 * M as u64 / 2]);
        let (_, zero_fell_back) = KnDiscounts::with_fallback([0, 1, 1, 1]).expect("zero");
        assert!(zero_fell_back);
        let (strict, fell_back) = KnDiscounts::with_fallback([2, 1, 1, 1]).expect("strict");
        assert!(!fell_back);
        assert_eq!(
            strict,
            KnDiscounts::from_count_of_counts([2, 1, 1, 1]).unwrap()
        );
        assert!(
            KnDiscounts::with_fallback([u64::MAX; 4]).is_err(),
            "overflow"
        );
        // The oracle's own rule agrees on each branch: in range, a zero n_i,
        // D2 < 0 with every n_i > 0 (D87's order 0), and D3 < 0 alone.
        let spectra = [
            [4_132_730, 413_614, 122_848, 56_120],
            [2, 1, 1, 1],
            [0, 1, 1, 1],
            [7, 6, 11, 11],
            [1, 1, 100, 1],
            [100, 50, 20, 1000],
        ];
        for n in spectra {
            let (host, _) = KnDiscounts::with_fallback(n).expect("discounts");
            let oracle = crate::kn_score_oracle::oracle_discounts(n.map(i128::from));
            assert_eq!(host.dq.map(i128::from), oracle, "n1..n4 {n:?}");
        }

        let tokens = kn_score_small_vocabulary_stream(0x421f, 30_000);
        const N: u32 = 2;
        let orders: Vec<_> = (0..=N)
            .map(|k| oracle_adjusted(&tokens, k, N).expect("adjusted"))
            .collect();
        let tables = KnTables::new(N, orders).expect("tables");
        assert_eq!(tables.discount_fallback(0), Some(true));
        assert_eq!(tables.discount_fallback(1), Some(true));
        assert_eq!(tables.discount_fallback(N), Some(false));
        let oracle = KnOracle::new(&tokens, N as usize);
        let wanted: Vec<u64> = (0..tokens.len())
            .filter(|&j| tokens[j] != NGRAM_BOS)
            .map(|j| u64::try_from(oracle.mass(&tokens, j)).expect("fits"))
            .collect();
        assert_eq!(tables.scored_masses(&tokens).expect("masses"), wanted);
    }
}
