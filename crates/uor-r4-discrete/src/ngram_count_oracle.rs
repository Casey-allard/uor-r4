//! Test-only CPU oracle for the CUDA n-gram counter (claude-412). An ordered
//! map over packed keys, independent of the GPU's sort-and-reduce. Nothing
//! here is a product path; GPU counts are the ones that qualify (CHARTER section 3, part 5).

use crate::ngram_count::{
    check_tokens, ngram_admitted, partition_from_triples, partition_offset, NgramAssembler,
    NgramContext, NgramOrderDigest, NgramPartition, NgramTriple, PrimeRouteKey, NGRAM_BOS,
    NGRAM_MAX_ORDER,
};
use crate::{Error, Result};
use std::collections::{BTreeMap, BTreeSet};

/// The adjusted counts of `order` under highest order `highest` (claude-418),
/// from the token stream alone: for each order-k gram admitted at j, the SET
/// of left tokens t[j-k-1] for which the order-(k+1) gram at j is admitted.
/// a(g) = c(g) when k = N or the gram's oldest token is BOS, else the set's
/// size. No triple is backed off. A zero adjusted count is an error.
pub fn oracle_adjusted(
    tokens: &[u16],
    order: u32,
    highest: u32,
) -> Result<(NgramOrderDigest, Vec<NgramTriple>, Vec<NgramContext>)> {
    let raw = oracle_partition(tokens, order, 0, 4096)?;
    let k = order as usize;
    let mut left: BTreeMap<(u128, u16), BTreeSet<u16>> = BTreeMap::new();
    if order < highest {
        for j in k + 1..tokens.len() {
            if ngram_admitted(tokens, j, k) && ngram_admitted(tokens, j, k + 1) {
                let key = PrimeRouteKey::from_context(&tokens[j - k..j])?.word();
                left.entry((key, tokens[j])).or_default().insert(tokens[j - k - 1]);
            }
        }
    }
    let mut triples = Vec::with_capacity(raw.triples.len());
    for triple in &raw.triples {
        let NgramTriple { key, next, count } = *triple;
        let document_start = PrimeRouteKey::from_word(key)?.context().first() == Some(&NGRAM_BOS);
        let adjusted = if order == highest || document_start {
            count
        } else {
            left.get(&(key, next)).map_or(0, |set| set.len() as u32)
        };
        if adjusted == 0 {
            return Err(Error::Invalid(format!(
                "oracle: an order-{order} gram has adjusted count 0"
            )));
        }
        triples.push(NgramTriple { key, next, count: adjusted });
    }
    let mut assembler = NgramAssembler::new_adjusted(order, true)?;
    assembler.push(&partition_from_triples(triples))?;
    Ok(assembler.finish())
}

/// The triples, contexts and count-of-counts of `order`, restricted to targets
/// whose partition token lies in lo..hi.
pub fn oracle_partition(tokens: &[u16], order: u32, lo: u32, hi: u32) -> Result<NgramPartition> {
    if order > NGRAM_MAX_ORDER {
        return Err(Error::Invalid(format!("order {order} exceeds {NGRAM_MAX_ORDER}")));
    }
    check_tokens(tokens)?;
    let k = order as usize;
    let offset = partition_offset(order);
    let mut grams: BTreeMap<(u128, u16), u32> = BTreeMap::new();
    for j in k..tokens.len() {
        if !ngram_admitted(tokens, j, k) {
            continue;
        }
        let digit = u32::from(tokens[j - offset]);
        if digit < lo || digit >= hi {
            continue;
        }
        let key = PrimeRouteKey::from_context(&tokens[j - k..j])?.word();
        *grams.entry((key, tokens[j])).or_insert(0) += 1;
    }
    let mut partition = NgramPartition::default();
    for (&(key, next), &count) in &grams {
        partition.triples.push(NgramTriple { key, next, count });
        if (1..=4).contains(&count) {
            partition.count_of_counts[count as usize - 1] += 1;
        }
        let new_context = partition.contexts.last().is_none_or(|h| h.key != key);
        if new_context {
            partition.contexts.push(NgramContext {
                key,
                total: 0,
                followers: 0,
                n1: 0,
                n2: 0,
                n3_plus: 0,
            });
        }
        let h = partition.contexts.last_mut().expect("context");
        h.total += u64::from(count);
        h.followers += 1;
        match count {
            1 => h.n1 += 1,
            2 => h.n2 += 1,
            _ => h.n3_plus += 1,
        }
    }
    Ok(partition)
}

/// The whole order, through the same assembler the device output goes through.
pub fn oracle_count(
    tokens: &[u16],
    order: u32,
) -> Result<(NgramOrderDigest, Vec<NgramTriple>, Vec<NgramContext>)> {
    let mut assembler = NgramAssembler::new(order, true)?;
    assembler.push(&oracle_partition(tokens, order, 0, 4096)?)?;
    Ok(assembler.finish())
}

/// Plant a +1 on triple `at` and adjust its context record and n1..n4 by hand
/// to match, so the partition stays self-consistent and the assembler admits
/// it. A test then shows the triple SHA covers the count field; this is not
/// a detection path (an inconsistent plant is refused by the assembler).
pub fn plant_one_count(partition: &mut NgramPartition, at: usize) {
    let NgramTriple { key, count, .. } = partition.triples[at];
    partition.triples[at].count = count + 1;
    if (1..=4).contains(&count) {
        partition.count_of_counts[count as usize - 1] -= 1;
    }
    if count < 4 {
        partition.count_of_counts[count as usize] += 1;
    }
    let h = partition.contexts.iter_mut().find(|h| h.key == key).expect("context");
    h.total += 1;
    match count {
        1 => {
            h.n1 -= 1;
            h.n2 += 1;
        }
        2 => {
            h.n2 -= 1;
            h.n3_plus += 1;
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ngram_count::adjust_partition;

    /// claude-429: the left digit the device sorts on last. L(j) is
    /// t[j-k-1] + 1 when order k+1 admits j (j > k, and t[j-k], the slot that
    /// becomes interior, is not BOS), and 0 otherwise.
    fn left_digit(tokens: &[u16], j: usize, k: usize) -> u16 {
        if j > k && tokens[j - k] != NGRAM_BOS {
            tokens[j - k - 1] + 1
        } else {
            0
        }
    }

    /// claude-429: the device's adjust rule on the host. Order k's positions
    /// in lo..hi, sorted by (key, next, L); a gram's N1+ is the number of
    /// positions in its run whose L is nonzero and differs from the previous
    /// position's. A gram keeps its raw count at k = N or when its oldest slot
    /// is BOS. Nothing here reads order k+1's partition. As on the device, a
    /// document-start gram with a nonzero N1+ is refused: that check is the only
    /// place the t[j-k] != BOS condition in L shows, since such a gram's count
    /// is raw.
    fn device_rule_partition(
        tokens: &[u16],
        order: u32,
        highest: u32,
        lo: u32,
        hi: u32,
    ) -> Result<NgramPartition> {
        let k = order as usize;
        let offset = partition_offset(order);
        let mut sorted = Vec::new();
        for j in k..tokens.len() {
            let digit = u32::from(tokens[j - offset]);
            if !ngram_admitted(tokens, j, k) || digit < lo || digit >= hi {
                continue;
            }
            let key = PrimeRouteKey::from_context(&tokens[j - k..j])?.word();
            sorted.push((key, tokens[j], left_digit(tokens, j, k)));
        }
        sorted.sort_unstable();
        let mut triples = Vec::new();
        let mut at = 0;
        while at < sorted.len() {
            let (key, next, _) = sorted[at];
            let mut end = at;
            let mut types = 0u32;
            let mut previous = None;
            while let Some(&(run_key, run_next, left)) = sorted.get(end) {
                if (run_key, run_next) != (key, next) {
                    break;
                }
                if left != 0 && previous != Some(left) {
                    types += 1;
                }
                previous = Some(left);
                end += 1;
            }
            let count = (end - at) as u32;
            let document_start = PrimeRouteKey::from_word(key)?.slot(order) == Some(NGRAM_BOS);
            if document_start && types != 0 {
                return Err(Error::Invalid(format!(
                    "model: an order-{order} gram whose oldest slot is BOS has a left extension"
                )));
            }
            let adjusted = if order == highest || document_start {
                count
            } else {
                types
            };
            if adjusted == 0 {
                return Err(Error::Invalid(format!(
                    "model: an order-{order} gram has no counted left extension (rule 4)"
                )));
            }
            triples.push(NgramTriple {
                key,
                next,
                count: adjusted,
            });
            at = end;
        }
        Ok(partition_from_triples(triples))
    }

    /// Ids 2..=12 with BOS every `bos_every` tokens, from a xorshift state.
    fn bos_stream(len: usize, seed: u64, bos_every: usize) -> Vec<u16> {
        let mut state = seed | 1;
        (0..len)
            .map(|i| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                match i % bos_every {
                    0 => NGRAM_BOS,
                    _ => 2 + ((state >> 40) % 11) as u16,
                }
            })
            .collect()
    }

    /// claude-429: the device rule gives `adjust_partition`'s partition, which
    /// reads order k+1's runs, for orders 1..=4 under N in {1, 3, 4}, over the
    /// whole range and three cuts of it, on a stream with BOS every 61 tokens
    /// and on one with BOS every 7.
    #[test]
    fn ngram_count_device_adjust_rule_equals_adjust_partition() {
        for (bos_every, seed) in [(61usize, 4291u64), (7, 4292)] {
            let tokens = bos_stream(20_000, seed, bos_every);
            let (mut changed, mut kept_at_start) = (0usize, 0usize);
            for highest in [1u32, 3, 4] {
                for order in 1..=highest {
                    for (lo, hi) in [(0u32, 4096u32), (0, 5), (5, 9), (9, 4096)] {
                        let at = format!(
                            "BOS every {bos_every}, N {highest}, order {order}, {lo}..{hi}"
                        );
                        let counted = oracle_partition(&tokens, order, lo, hi).expect("counted");
                        let extended = match order < highest {
                            true => Some(
                                oracle_partition(&tokens, order + 1, lo, hi).expect("extended"),
                            ),
                            false => None,
                        };
                        let want =
                            adjust_partition(order, highest, lo, hi, &counted, extended.as_ref())
                                .expect("host adjust");
                        let got = device_rule_partition(&tokens, order, highest, lo, hi)
                            .expect("device rule");
                        assert_eq!(got, want, "{at}");
                        for (adjusted, raw) in got.triples.iter().zip(&counted.triples) {
                            let start = PrimeRouteKey::from_word(raw.key).expect("key").slot(order);
                            changed += usize::from(adjusted.count != raw.count);
                            kept_at_start +=
                                usize::from(order < highest && start == Some(NGRAM_BOS));
                        }
                    }
                }
            }
            assert!(
                changed > 0,
                "BOS every {bos_every}: no adjusted count differs from its raw count"
            );
            assert!(
                kept_at_start > 0,
                "BOS every {bos_every}: no document-start gram below N"
            );
        }
    }
}
