//! Test-only oracle for the integer modified-KN scorer (claude-419). It counts
//! straight from the token stream into ordered maps keyed by the context
//! tokens themselves: raw counts, sets of distinct left tokens, the adjusted
//! counts, per-context totals and n1/n2/n3+, and its own count-of-counts and
//! discounts. It shares no code with `KnTables` or with the counter's
//! assembler; only the admission rule `ngram_admitted` is common.

use crate::ngram_count::{ngram_admitted, NGRAM_BOS};
use std::collections::{BTreeMap, BTreeSet};

const M: i128 = 1 << 48;

struct OracleOrder {
    adjusted: BTreeMap<(Vec<u16>, u16), i128>,
    /// A, n1(h), n2(h), n3+(h).
    contexts: BTreeMap<Vec<u16>, [i128; 4]>,
    discounts: [i128; 3],
}

/// Chen–Goodman from n1..n4, or KenLM's fallback (1/2, 1, 3/2) for the whole
/// order when an n_i is zero or an estimate leaves [0, i]. Small counts only.
pub fn oracle_discounts(spectrum: [i128; 4]) -> [i128; 3] {
    let fallback = [M / 2, M, 3 * M / 2];
    if spectrum.contains(&0) {
        return fallback;
    }
    let [n1, n2, n3, n4] = spectrum;
    let s = n1 + 2 * n2;
    let pairs = [(n1, n2), (n2, n3), (n3, n4)];
    let estimates: Vec<i128> = pairs
        .into_iter()
        .zip(1i128..)
        .map(|((ni, next), r)| (M * (r * ni * s - (r + 1) * n1 * next)).div_euclid(ni * s))
        .collect();
    if estimates
        .iter()
        .zip(1i128..)
        .all(|(&d, r)| (0..=r * M).contains(&d))
    {
        [estimates[0], estimates[1], estimates[2]]
    } else {
        fallback
    }
}

pub struct KnOracle {
    highest: usize,
    orders: Vec<OracleOrder>,
}

impl KnOracle {
    /// Counts every order 0..=highest and takes its discounts.
    pub fn new(tokens: &[u16], highest: usize) -> Self {
        let mut orders = Vec::new();
        for k in 0..=highest {
            let mut raw: BTreeMap<(Vec<u16>, u16), i128> = BTreeMap::new();
            let mut left: BTreeMap<(Vec<u16>, u16), BTreeSet<u16>> = BTreeMap::new();
            for j in k..tokens.len() {
                if !ngram_admitted(tokens, j, k) {
                    continue;
                }
                let gram = (tokens[j - k..j].to_vec(), tokens[j]);
                *raw.entry(gram.clone()).or_insert(0) += 1;
                if k < highest && j > k && ngram_admitted(tokens, j, k + 1) {
                    left.entry(gram).or_default().insert(tokens[j - k - 1]);
                }
            }
            let mut adjusted = BTreeMap::new();
            let mut contexts: BTreeMap<Vec<u16>, [i128; 4]> = BTreeMap::new();
            let mut spectrum = [0i128; 4];
            for (gram, count) in raw {
                let document_start = gram.0.first() == Some(&NGRAM_BOS);
                let a = if k == highest || document_start {
                    count
                } else {
                    left.get(&gram).map_or(0, |set| set.len() as i128)
                };
                assert!(
                    a > 0,
                    "oracle: order-{k} gram {gram:?} has adjusted count 0"
                );
                let record = contexts.entry(gram.0.clone()).or_insert([0; 4]);
                record[0] += a;
                record[(a.min(3)) as usize] += 1;
                if a <= 4 {
                    spectrum[(a - 1) as usize] += 1;
                }
                adjusted.insert(gram, a);
            }
            let discounts = oracle_discounts(spectrum);
            orders.push(OracleOrder {
                adjusted,
                contexts,
                discounts,
            });
        }
        Self { highest, orders }
    }

    /// The Q48 mass of `tokens[j]` after its history, by the claude-419 rule.
    pub fn mass(&self, tokens: &[u16], j: usize) -> u128 {
        let mut p: i128 = M / 4096;
        for k in 0..=self.highest.min(j) {
            let order = &self.orders[k];
            let context = tokens[j - k..j].to_vec();
            let Some(&[total, n1, n2, n3]) = order.contexts.get(&context) else {
                continue;
            };
            let a = order
                .adjusted
                .get(&(context, tokens[j]))
                .copied()
                .unwrap_or(0);
            let [d1, d2, d3] = order.discounts;
            let d = match a {
                0 => 0,
                1 => d1,
                2 => d2,
                _ => d3,
            };
            let gamma = (d1 * n1 + d2 * n2 + d3 * n3) / total;
            p = (a * M - d) / total + gamma * p / M;
        }
        p as u128
    }
}
