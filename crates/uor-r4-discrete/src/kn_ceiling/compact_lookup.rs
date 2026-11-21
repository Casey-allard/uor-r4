//! Resident raw and KN lookup over a common, query-independent key index.
//! Each order has fixed-width context and follower rows so serving can borrow
//! the artifact without a second expanded copy.

use super::{
    complete_lookup::{validate_records, Order},
    invalid, FrozenPickMixture, PickTarget,
};
use crate::{
    kn_score::{interpolate_order, KnDiscounts, KN_BASE_MASS},
    ngram_count::{
        partition_from_triples, NgramAssembler, NgramContext, NgramOrderDigest, NgramTriple,
        PrimeRouteKey, NGRAM_BOS, NGRAM_MAX_ORDER, NGRAM_VOCABULARY,
    },
    Result,
};
use sha2::{Digest, Sha256};

include!("runtime/compact.rs");
include!("runtime/rank.rs");

/// Pack full canonical raw and adjusted tables from the same count stream.
/// Their key/next support is the same because the adjusted assembler refuses
/// zero-mass continuations. The highest order also shares count values.
pub fn encode_compact_lookup(
    highest: u32,
    training: [u8; 32],
    raw: &[Order],
    adjusted: &[Order],
) -> Result<Vec<u8>> {
    if highest > NGRAM_MAX_ORDER || raw.len() != highest as usize + 1 || raw.len() != adjusted.len()
    {
        return Err(invalid("invalid compact lookup order layout"));
    }
    let mut bytes = Vec::new();
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&highest.to_le_bytes());
    bytes.extend_from_slice(&training);
    for (order, (raw_order, adjusted_order)) in (0..=highest).zip(raw.iter().zip(adjusted)) {
        let (raw_digest, raw_rows, raw_contexts) = raw_order;
        let (adjusted_digest, adjusted_rows, adjusted_contexts) = adjusted_order;
        let shared = order == highest;
        for (digest, rows, contexts) in [raw_order, adjusted_order] {
            if digest.order != order
                || digest.grams != rows.len() as u64
                || digest.contexts != contexts.len() as u64
            {
                return Err(invalid("incomplete or reordered compact count assembly"));
            }
            validate_records(rows, contexts, digest.count_of_counts)?;
        }
        if raw_contexts.len() != adjusted_contexts.len()
            || raw_rows.len() != adjusted_rows.len()
            || raw_rows.iter().zip(adjusted_rows).any(|(raw, kn)| {
                (raw.key, raw.next) != (kn.key, kn.next) || (shared && raw.count != kn.count)
            })
        {
            return Err(invalid(
                "raw and adjusted count support or shared values differ",
            ));
        }
        bytes.extend_from_slice(&as_u32(raw_rows.len())?.to_le_bytes());
        bytes.extend_from_slice(&as_u32(raw_contexts.len())?.to_le_bytes());
        for spectrum in [raw_digest.count_of_counts, adjusted_digest.count_of_counts] {
            for word in spectrum {
                bytes.extend_from_slice(&word.to_le_bytes());
            }
        }
        let mut row_start = 0u32;
        for context in raw_contexts {
            if PrimeRouteKey::from_word(context.key)?.order() != order {
                return Err(invalid("compact context key has the wrong order"));
            }
            bytes.extend_from_slice(&key_body(context.key, order)[16 - key_width(order)..]);
            bytes.extend_from_slice(&row_start.to_le_bytes());
            row_start += context.followers;
        }
        for (raw, kn) in raw_rows.iter().zip(adjusted_rows) {
            bytes.extend_from_slice(&raw.next.to_le_bytes());
            bytes.extend_from_slice(&raw.count.to_le_bytes());
            if !shared {
                bytes.extend_from_slice(&kn.count.to_le_bytes());
            }
        }
    }
    Ok(bytes)
}

fn as_u32(len: usize) -> Result<u32> {
    len.try_into()
        .map_err(|_| invalid("compact table exceeds u32 rows"))
}

impl CompactCountLookup<'_> {
    /// Audit the reloaded payload against the original raw and adjusted
    /// canonical count streams, reconstructing complete contexts in bounded
    /// batches. This is a build-time check and is outside the lookup path.
    pub fn canonical_digests(&self) -> Result<Vec<[NgramOrderDigest; 2]>> {
        const BATCH: usize = 64 * 1024;
        let mut digests = Vec::with_capacity(self.orders.len());
        for (k, table) in self.orders.iter().enumerate() {
            let order = k as u32;
            let [mut raw, mut adjusted] = [
                NgramAssembler::new(order, false)?,
                NgramAssembler::new_adjusted(order, false)?,
            ];
            let [mut raw_rows, mut adjusted_rows] = [Vec::new(), Vec::new()];
            for context in 0..table.contexts.len() / table.stride() {
                let body = table.context_key(context);
                let mut bytes = [0; 16];
                bytes[16 - body.len()..].copy_from_slice(body);
                let key =
                    ((order as u128) << 120) | (u128::from_be_bytes(bytes) << (120 - order * 12));
                let rows = table
                    .follower_rows(context)
                    .ok_or_else(|| invalid("invalid compact follower span during audit"))?;
                for row in rows.chunks_exact(follower_width(table.shared)) {
                    let (next, raw_count, adjusted_count) = table.read_follower(row);
                    raw_rows.push(NgramTriple {
                        key,
                        next,
                        count: raw_count,
                    });
                    adjusted_rows.push(NgramTriple {
                        key,
                        next,
                        count: adjusted_count,
                    });
                }
                if raw_rows.len() >= BATCH {
                    raw.push_owned(partition_from_triples(std::mem::take(&mut raw_rows)))?;
                    adjusted
                        .push_owned(partition_from_triples(std::mem::take(&mut adjusted_rows)))?;
                }
            }
            raw.push_owned(partition_from_triples(raw_rows))?;
            adjusted.push_owned(partition_from_triples(adjusted_rows))?;
            let (raw, _, _) = raw.finish();
            let (adjusted, _, _) = adjusted.finish();
            digests.push([raw, adjusted]);
        }
        Ok(digests)
    }
}

#[cfg(test)]
mod ranking_tests;

#[path = "compact_lookup/replay.rs"]
mod replay;
