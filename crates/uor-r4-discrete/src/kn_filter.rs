//! The query filter of the reference scorer R* (claude-421). The whole-TRAIN
//! modified-KN tables do not fit in memory, but [`KnTables::scored_masses`]
//! reads, for each scored target j and order k <= min(N, j), only the context
//! record of `query[j-k..j]` and the triple of (that context, `query[j]`).
//! [`KnQuery`] lists those keys once per order; a [`KnFilter`] keeps, while
//! the counter streams one order's partitions, exactly the records listed.
//! Tables built from the kept records and the full order's digest score every
//! query target as the unfiltered tables do: an absent context backs off and
//! an absent triple reads a = 0 in both. Integer keys, sorted vectors and
//! binary search only.
//!
//! [`KnTables::scored_masses`]: crate::kn_score::KnTables::scored_masses

use crate::ngram_count::{
    check_tokens, NgramContext, NgramPartition, NgramTriple, PrimeRouteKey, NGRAM_BOS,
    NGRAM_MAX_ORDER,
};
use crate::{Error, Result};

fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(message.into())
}

/// One order's looked-up keys, sorted and deduplicated.
struct QueryOrder {
    contexts: Vec<u128>,
    pairs: Vec<(u128, u16)>,
}

/// The keys a query stream looks up at orders 0..=highest.
pub struct KnQuery {
    highest: u32,
    orders: Vec<QueryOrder>,
}

impl KnQuery {
    /// For each order k, the context key of every query position j >= k whose
    /// target is not BOS, and its (key, target) pair. Refuses an order above
    /// [`NGRAM_MAX_ORDER`] and an id outside the vocabulary.
    pub fn from_stream(query: &[u16], highest: u32) -> Result<Self> {
        if highest > NGRAM_MAX_ORDER {
            return Err(invalid(format!(
                "a KN query reads at most order {NGRAM_MAX_ORDER}, got {highest}"
            )));
        }
        check_tokens(query)?;
        let mut orders = Vec::with_capacity(highest as usize + 1);
        for k in 0..=highest as usize {
            let mut contexts = Vec::new();
            let mut pairs = Vec::new();
            for j in k..query.len() {
                if query[j] == NGRAM_BOS {
                    continue;
                }
                let key = PrimeRouteKey::from_context(&query[j - k..j])?.word();
                contexts.push(key);
                pairs.push((key, query[j]));
            }
            contexts.sort_unstable();
            contexts.dedup();
            pairs.sort_unstable();
            pairs.dedup();
            orders.push(QueryOrder { contexts, pairs });
        }
        Ok(Self { highest, orders })
    }

    pub fn highest(&self) -> u32 {
        self.highest
    }

    /// Order k's looked-up (contexts, pairs) counts.
    pub fn lookups(&self, k: u32) -> Option<(usize, usize)> {
        let QueryOrder { contexts, pairs } = self.orders.get(k as usize)?;
        Some((contexts.len(), pairs.len()))
    }

    /// The filter of order k, fed that order's partitions in canonical order.
    pub fn filter(&self, k: u32) -> Result<KnFilter<'_>> {
        let order = self.orders.get(k as usize).ok_or_else(|| {
            invalid(format!(
                "the KN query holds orders 0..={}, not {k}",
                self.highest
            ))
        })?;
        Ok(KnFilter {
            order,
            last_triple: None,
            last_context: None,
            triples: Vec::new(),
            contexts: Vec::new(),
        })
    }
}

/// Keeps one order's records that a query looks up. A refused partition
/// leaves the filter unusable; the caller drops the whole count.
pub struct KnFilter<'q> {
    order: &'q QueryOrder,
    last_triple: Option<(u128, u16)>,
    last_context: Option<u128>,
    triples: Vec<NgramTriple>,
    contexts: Vec<NgramContext>,
}

impl KnFilter<'_> {
    /// Append the partition's looked-up triples and contexts. Refuses triples
    /// that do not strictly ascend in (key, next) and contexts whose keys fall,
    /// across partitions as within one. An equal context key (the order-0
    /// straddle of a seam) is added field by field into the last kept record.
    pub fn keep(&mut self, partition: &NgramPartition) -> Result<()> {
        for triple in &partition.triples {
            let at = (triple.key, triple.next);
            if self.last_triple.is_some_and(|before| before >= at) {
                return Err(invalid(
                    "KN filter: triples are not in strictly ascending order",
                ));
            }
            self.last_triple = Some(at);
            if self.order.pairs.binary_search(&at).is_ok() {
                self.triples.push(*triple);
            }
        }
        for context in &partition.contexts {
            if self.last_context.is_some_and(|before| before > context.key) {
                return Err(invalid("KN filter: contexts are not in ascending order"));
            }
            self.last_context = Some(context.key);
            if self.order.contexts.binary_search(&context.key).is_err() {
                continue;
            }
            match self.contexts.last_mut() {
                Some(kept) if kept.key == context.key => add_context(kept, context)?,
                _ => self.contexts.push(*context),
            }
        }
        Ok(())
    }

    pub fn finish(self) -> (Vec<NgramTriple>, Vec<NgramContext>) {
        (self.triples, self.contexts)
    }
}

/// Every field of a context record is additive over disjoint follower sets.
fn add_context(kept: &mut NgramContext, part: &NgramContext) -> Result<()> {
    let overflow = || invalid("KN filter: a straddling context's sum overflowed");
    kept.total = kept.total.checked_add(part.total).ok_or_else(overflow)?;
    for (field, add) in [
        (&mut kept.followers, part.followers),
        (&mut kept.n1, part.n1),
        (&mut kept.n2, part.n2),
        (&mut kept.n3_plus, part.n3_plus),
    ] {
        *field = field.checked_add(add).ok_or_else(overflow)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kn_score::KnTables;
    use crate::ngram_count::{partition_from_triples, NgramAssembler};
    use crate::ngram_count_oracle::oracle_adjusted;

    /// The text-like stream of `kn_score`'s tests: a heavy-headed id
    /// distribution, one draw in 61 a BOS, one in 9 a copied earlier span.
    fn kn_filter_stream(seed: u64, len: usize) -> Vec<u16> {
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

    /// Four partitions of one order's canonical triples, as the device emits
    /// them: at order >= 1 the cuts fall on key boundaries; at order 0 every
    /// triple has the empty key, so each part carries a share of its record.
    fn partitions(order: u32, triples: &[NgramTriple]) -> Vec<NgramPartition> {
        let mut cuts = vec![0];
        for quarter in 1..4 {
            let mut at = (triples.len() * quarter / 4).max(cuts[cuts.len() - 1]);
            while order >= 1
                && at > 0
                && at < triples.len()
                && triples[at - 1].key == triples[at].key
            {
                at += 1;
            }
            cuts.push(at);
        }
        cuts.push(triples.len());
        cuts.windows(2)
            .map(|w| partition_from_triples(triples[w[0]..w[1]].to_vec()))
            .collect()
    }

    #[test]
    fn kn_filter_filtered_tables_score_as_unfiltered() {
        let train = kn_filter_stream(0x421a, 6_000);
        let query = kn_filter_stream(0x421b, 2_500);
        assert!(query.iter().filter(|&&t| t == NGRAM_BOS).count() > 5);
        let mut by_order: Vec<Vec<u64>> = Vec::new();
        for highest in 0..=4u32 {
            let lookups = KnQuery::from_stream(&query, highest).expect("query");
            let (mut full, mut kept) = (Vec::new(), Vec::new());
            for k in 0..=highest {
                let oracle = oracle_adjusted(&train, k, highest).expect("adjusted");
                let (_, triples, _) = &oracle;
                let parts = partitions(k, triples);
                assert!(parts.iter().filter(|p| !p.triples.is_empty()).count() >= 3);
                let mut assembler = NgramAssembler::new_adjusted(k, true).expect("assembler");
                let mut filter = lookups.filter(k).expect("filter");
                for part in &parts {
                    assembler.push(part).expect("canonical partition");
                    filter.keep(part).expect("kept");
                }
                let (digest, _, assembled) = assembler.finish();
                let (_, _, contexts) = &oracle;
                assert_eq!(&assembled, contexts, "the partitions reassemble order {k}");
                if k == 0 {
                    let shares = parts.iter().filter(|p| !p.contexts.is_empty()).count();
                    assert!(shares >= 2, "the empty context straddles a seam");
                }
                let (filtered_triples, filtered_contexts) = filter.finish();
                if k == highest {
                    assert!(
                        filtered_triples.len() < triples.len(),
                        "N {highest}: the filter kept all {} triples",
                        triples.len()
                    );
                }
                if k == 0 {
                    assert_eq!(
                        filtered_contexts, *contexts,
                        "the order-0 record merges whole"
                    );
                }
                kept.push((digest, filtered_triples, filtered_contexts));
                full.push(oracle);
            }
            let full = KnTables::new(highest, full).expect("full tables");
            let kept = KnTables::new(highest, kept).expect("filtered tables");
            let want = full.scored_masses(&query).expect("full masses");
            assert_eq!(
                kept.scored_masses(&query).expect("filtered masses"),
                want,
                "N {highest}"
            );
            by_order.push(want);
        }
        assert_ne!(by_order[2], by_order[0], "orders above 0 change the masses");
    }

    #[test]
    fn kn_filter_refuses_falling_keys_and_bad_queries() {
        assert!(KnQuery::from_stream(&[0, 5, 6], NGRAM_MAX_ORDER + 1).is_err());
        assert!(KnQuery::from_stream(&[0, 5, 4096], 1).is_err());
        let query = [0u16, 5, 6, 5, 7];
        let lookups = KnQuery::from_stream(&query, 1).expect("query");
        assert!(lookups.filter(2).is_err());
        let key = |ctx: &[u16]| PrimeRouteKey::from_context(ctx).expect("key").word();
        let triple = |ctx: &[u16], next: u16| NgramTriple {
            key: key(ctx),
            next,
            count: 1,
        };
        let high = partition_from_triples(vec![triple(&[6], 5)]);
        let low = partition_from_triples(vec![triple(&[5], 6)]);
        let mut filter = lookups.filter(1).expect("filter");
        filter.keep(&high).expect("first partition");
        assert!(
            filter.keep(&low).is_err(),
            "a partition below the kept keys"
        );
        let mut filter = lookups.filter(1).expect("filter");
        let inside = partition_from_triples(vec![triple(&[6], 5), triple(&[5], 6)]);
        assert!(
            filter.keep(&inside).is_err(),
            "falling keys inside one partition"
        );
        let mut filter = lookups.filter(1).expect("filter");
        filter.keep(&low).expect("low");
        filter.keep(&high).expect("high");
        let (triples, contexts) = filter.finish();
        assert_eq!(triples.len(), 2);
        assert_eq!(contexts.len(), 2);
    }
}
