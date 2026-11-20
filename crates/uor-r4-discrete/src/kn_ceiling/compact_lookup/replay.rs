use super::*;
use crate::{kn_ceiling::RawTables, kn_score::KnTables};

impl CompactCountLookup<'_> {
    /// Expand the complete follower spans of exactly one causal prefix for
    /// independent CUDA lookup. Global digests supply the original spectra;
    /// holding only matched contexts does not change their discounts.
    pub fn device_replay_tables(
        &self,
        prefix: &[u16],
        global: &[[NgramOrderDigest; 2]],
    ) -> Result<(RawTables, KnTables)> {
        if global.len() != self.orders.len() || global.is_empty() {
            return Err(invalid("device replay census has the wrong order width"));
        }
        let mut raw = Vec::with_capacity(global.len());
        let mut adjusted = Vec::with_capacity(global.len());
        for (k, (table, [raw_digest, adjusted_digest])) in
            self.orders.iter().zip(global).enumerate()
        {
            let order = k as u32;
            let (discounts, _) = KnDiscounts::with_fallback(adjusted_digest.count_of_counts)?;
            if raw_digest.order != order
                || adjusted_digest.order != order
                || discounts != table.discounts
            {
                return Err(invalid(
                    "device replay census changes a compact order or its discounts",
                ));
            }
            let [mut raw_rows, mut adjusted_rows] = [Vec::new(), Vec::new()];
            if prefix.len() >= k {
                let key = PrimeRouteKey::from_context(&prefix[prefix.len() - k..])?.word();
                let bytes = key_body(key, order);
                let needle = &bytes[16 - table.key_width..];
                let (mut lo, mut hi) = (0, table.contexts.len() / table.stride());
                while lo < hi {
                    let mid = lo + (hi - lo) / 2;
                    if table.context_key(mid) < needle {
                        lo = mid + 1;
                    } else {
                        hi = mid;
                    }
                }
                if lo < table.contexts.len() / table.stride() && table.context_key(lo) == needle {
                    let rows = table
                        .follower_rows(lo)
                        .ok_or_else(|| invalid("invalid CUDA replay follower span"))?;
                    for word in rows.chunks_exact(follower_width(table.shared)) {
                        let (next, raw, adjusted) = table.read_follower(word);
                        raw_rows.push(NgramTriple {
                            key,
                            next,
                            count: raw,
                        });
                        adjusted_rows.push(NgramTriple {
                            key,
                            next,
                            count: adjusted,
                        });
                    }
                }
            }
            let r = partition_from_triples(raw_rows);
            let a = partition_from_triples(adjusted_rows);
            raw.push((raw_digest.clone(), r.triples, r.contexts));
            adjusted.push((adjusted_digest.clone(), a.triples, a.contexts));
        }
        let highest = global.len() as u32 - 1;
        Ok((
            RawTables::new(highest, raw)?,
            KnTables::new(highest, adjusted)?,
        ))
    }
}
