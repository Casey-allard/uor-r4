//! Test-only CPU oracle for the CUDA n-gram counter (claude-412). An ordered
//! map over packed keys, independent of the GPU's sort-and-reduce. Nothing
//! here is a product path; GPU counts are the ones that qualify (CHARTER section 3, part 5).

use crate::ngram_count::{
    check_tokens, partition_offset, NgramAssembler, NgramContext, NgramOrderDigest,
    NgramPartition, NgramTriple, PrimeRouteKey, NGRAM_MAX_ORDER,
};
use crate::{Error, Result};
use std::collections::BTreeMap;

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
