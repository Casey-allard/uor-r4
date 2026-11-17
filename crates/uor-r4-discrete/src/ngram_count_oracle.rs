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
