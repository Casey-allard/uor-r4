//! Query-independent serialization of complete raw and adjusted order tables.

use super::{invalid, RawTables};
use crate::{
    kn_score::KnTables,
    ngram_count::{
        context_bytes, triple_bytes, NgramContext, NgramOrderDigest, NgramTriple, CONTEXT_BYTES,
        NGRAM_BOS, NGRAM_MAX_ORDER, NGRAM_VOCABULARY, TRIPLE_BYTES,
    },
    Result,
};
use sha2::{Digest, Sha256};

const MAGIC: &[u8; 8] = b"UORKNC01";
type Order = (NgramOrderDigest, Vec<NgramTriple>, Vec<NgramContext>);

/// Complete model-independent lookups for one pinned original-token window.
pub struct CompleteCountLookup {
    raw: RawTables,
    kn: KnTables,
}

impl CompleteCountLookup {
    /// Validate the artifact identity before allocating any order table.
    pub fn from_bytes(
        bytes: &[u8],
        expected_training: [u8; 32],
        expected_artifact: [u8; 32],
    ) -> Result<Self> {
        if Sha256::digest(bytes)[..] != expected_artifact {
            return Err(invalid("count lookup artifact digest mismatch"));
        }
        let mut wire = Wire { remaining: bytes };
        if &wire.take::<8>()? != MAGIC {
            return Err(invalid("wrong count lookup format"));
        }
        let highest = u32::from_le_bytes(wire.take()?);
        if highest > NGRAM_MAX_ORDER || wire.take::<32>()? != expected_training {
            return Err(invalid("invalid order or count lookup training binding"));
        }
        let raw = read_orders(&mut wire, highest)?;
        let kn = read_orders(&mut wire, highest)?;
        if !wire.remaining.is_empty()
            || raw.last().map(|row| (&row.1, &row.2)) != kn.last().map(|row| (&row.1, &row.2))
        {
            return Err(invalid("trailing bytes or unequal order-N count rows"));
        }
        Ok(Self {
            raw: RawTables::new(highest, raw)?,
            kn: KnTables::new(highest, kn)?,
        })
    }

    /// Fill the per-order ML lane inputs and return the original KN Q48 word.
    /// `tokens` contains the caller's causal history and the target at `at`.
    pub fn lookup(
        &self,
        tokens: &[u16],
        at: usize,
        counts: &mut [u32],
        totals: &mut [u64],
    ) -> Result<u64> {
        self.raw.lookup_target(tokens, at, counts, totals)?;
        self.kn.score_target(tokens, at)
    }
}

/// Encode full count assembler outputs, with raw first and adjusted second.
/// Below the highest order, the adjusted rows count distinct left contexts.
/// Both highest orders must carry the same raw state.
pub fn encode_complete_lookup(
    highest: u32,
    training: [u8; 32],
    raw: &[Order],
    adjusted: &[Order],
) -> Result<Vec<u8>> {
    if highest > NGRAM_MAX_ORDER
        || raw.len() != highest as usize + 1
        || raw.len() != adjusted.len()
        || raw.last().map(|row| (&row.1, &row.2)) != adjusted.last().map(|row| (&row.1, &row.2))
    {
        return Err(invalid("invalid complete lookup order layout"));
    }
    let mut bytes = Vec::new();
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&highest.to_le_bytes());
    bytes.extend_from_slice(&training);
    for orders in [raw, adjusted] {
        for (order, (digest, triples, contexts)) in orders.iter().enumerate() {
            if digest.order as usize != order
                || digest.grams != triples.len() as u64
                || digest.contexts != contexts.len() as u64
            {
                return Err(invalid("incomplete or reordered count assembly"));
            }
            validate_records(triples, contexts, digest.count_of_counts)?;
            bytes.extend_from_slice(&digest.grams.to_le_bytes());
            bytes.extend_from_slice(&digest.contexts.to_le_bytes());
            for value in digest.count_of_counts {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
            for triple in triples {
                bytes.extend_from_slice(&triple_bytes(triple));
            }
            for context in contexts {
                bytes.extend_from_slice(&context_bytes(context));
            }
        }
    }
    Ok(bytes)
}

struct Wire<'a> {
    remaining: &'a [u8],
}

impl Wire<'_> {
    fn take<const N: usize>(&mut self) -> Result<[u8; N]> {
        let (word, last) = self
            .remaining
            .split_at_checked(N)
            .ok_or_else(|| invalid("truncated count lookup"))?;
        self.remaining = last;
        word.try_into()
            .map_err(|_| invalid("wrong count word size"))
    }

    fn count(&mut self) -> Result<usize> {
        u64::from_le_bytes(self.take()?)
            .try_into()
            .map_err(|_| invalid("count exceeds usize"))
    }
}

fn read_orders(wire: &mut Wire<'_>, highest: u32) -> Result<Vec<Order>> {
    let mut output = Vec::new();
    for order in 0..=highest {
        let grams = wire.count()?;
        let census = wire.count()?;
        let mut spectrum = [0u64; 4];
        for value in &mut spectrum {
            *value = u64::from_le_bytes(wire.take()?);
        }
        let size = grams
            .checked_mul(TRIPLE_BYTES)
            .zip(census.checked_mul(CONTEXT_BYTES))
            .and_then(|(g, c)| g.checked_add(c))
            .ok_or_else(|| invalid("count table length overflow"))?;
        if size > wire.remaining.len() {
            return Err(invalid("count table exceeds artifact bounds"));
        }
        let mut triples = Vec::with_capacity(grams);
        let mut contexts = Vec::with_capacity(census);
        for _ in 0..grams {
            triples.push(NgramTriple {
                key: u128::from_le_bytes(wire.take()?),
                next: u16::from_le_bytes(wire.take()?),
                count: u32::from_le_bytes(wire.take()?),
            });
        }
        for _ in 0..census {
            contexts.push(NgramContext {
                key: u128::from_le_bytes(wire.take()?),
                total: u64::from_le_bytes(wire.take()?),
                followers: u32::from_le_bytes(wire.take()?),
                n1: u32::from_le_bytes(wire.take()?),
                n2: u32::from_le_bytes(wire.take()?),
                n3_plus: u32::from_le_bytes(wire.take()?),
            });
        }
        validate_records(&triples, &contexts, spectrum)?;
        let digest = NgramOrderDigest {
            order,
            grams: grams as u64,
            contexts: census as u64,
            count_of_counts: spectrum,
            // These receipt fields are not consumed by the predictive tables.
            targets: 0,
            triple_sha256: String::new(),
            context_sha256: String::new(),
        };
        output.push((digest, triples, contexts));
    }
    Ok(output)
}

fn validate_records(
    triples: &[NgramTriple],
    contexts: &[NgramContext],
    expected_spectrum: [u64; 4],
) -> Result<()> {
    if !triples
        .windows(2)
        .all(|r| (r[0].key, r[0].next) < (r[1].key, r[1].next))
    {
        return Err(invalid("count triples are not canonically sorted"));
    }
    let mut spectrum = [0; 4];
    let mut groups = triples.chunk_by(|a, b| a.key == b.key);
    for context in contexts {
        let rows = groups
            .next()
            .ok_or_else(|| invalid("context has no triples"))?;
        let mut classes = [0u32; 3];
        let mut total = 0u64;
        if rows[0].key != context.key {
            return Err(invalid("context and triple key differ"));
        }
        for row in rows {
            if row.next == NGRAM_BOS || u32::from(row.next) >= NGRAM_VOCABULARY || row.count == 0 {
                return Err(invalid("invalid original count triple"));
            }
            if row.count <= 4 {
                spectrum[row.count as usize - 1] += 1;
            }
            classes[row.count.min(3) as usize - 1] += 1;
            total += u64::from(row.count);
        }
        if (classes, total, rows.len())
            != (
                [context.n1, context.n2, context.n3_plus],
                context.total,
                context.followers as usize,
            )
        {
            return Err(invalid("context census differs from complete triples"));
        }
    }
    if groups.next().is_some() || spectrum != expected_spectrum {
        return Err(invalid("incomplete context or count spectrum differs"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ngram_count_oracle::{oracle_adjusted, oracle_count};

    #[test]
    fn complete_lookup_roundtrips_raw_and_adjusted_words() -> Result<()> {
        let train = [0, 2, 3, 5, 2, 3, 1, 0, 4, 3, 5, 2, 3, 1, 0, 4, 3, 5, 6, 1];
        // BOS, repeat, changed left context, unseen token, and record restart.
        let probe = [0, 2, 3, 5, 2, 3, 1, 0, 4, 3, 5, 7, 1];
        let highest = 3;
        let raw = (0..=highest)
            .map(|order| oracle_count(&train, order))
            .collect::<Result<Vec<_>>>()?;
        let adjusted = (0..=highest)
            .map(|order| oracle_adjusted(&train, order, highest))
            .collect::<Result<Vec<_>>>()?;
        let (_, raw_triples, _) = &raw[1];
        let (_, adjusted_triples, _) = &adjusted[1];
        assert_ne!(raw_triples, adjusted_triples);
        let bytes = encode_complete_lookup(highest, [11; 32], &raw, &adjusted)?;
        let digest = Sha256::digest(&bytes).into();
        let loaded = CompleteCountLookup::from_bytes(&bytes, [11; 32], digest)?;
        assert!(CompleteCountLookup::from_bytes(&bytes, [10; 32], digest).is_err());
        let truncated = &bytes[..bytes.len() - 1];
        assert!(CompleteCountLookup::from_bytes(truncated, [11; 32], digest).is_err());
        assert!(CompleteCountLookup::from_bytes(
            truncated,
            [11; 32],
            Sha256::digest(truncated).into()
        )
        .is_err());
        let raw = RawTables::new(highest, raw)?;
        let adjusted = KnTables::new(highest, adjusted)?;
        let width = highest as usize + 1;
        let (want_counts, want_totals) = raw.lookups(&probe)?;
        let (mut counts, mut totals) = ([0; 4], [0; 4]);
        for (row, (at, _)) in probe
            .iter()
            .enumerate()
            .filter(|(_, &id)| id != 0)
            .enumerate()
        {
            let q48 = loaded.lookup(&probe, at, &mut counts, &mut totals)?;
            assert_eq!(q48, adjusted.score_target(&probe, at)?);
            assert_eq!(counts, want_counts[row * width..(row + 1) * width]);
            assert_eq!(totals, want_totals[row * width..(row + 1) * width]);
        }
        assert!(loaded.lookup(&probe, 0, &mut counts, &mut totals).is_err());
        eprintln!(
            "complete codec fixture: {} bytes, {} target rows, {} raw words in each tape",
            bytes.len(),
            want_counts.len() / width,
            want_counts.len()
        );
        Ok(())
    }

    #[test]
    fn rejects_query_filtered_or_damaged_tables() -> Result<()> {
        let train = [0, 2, 3, 4, 2, 3, 1];
        let raw = vec![oracle_count(&train, 0)?];
        let mut wrong = raw.clone();
        let (_, triples, _) = &mut wrong[0];
        triples.pop();
        assert!(encode_complete_lookup(0, [11; 32], &wrong, &wrong).is_err());
        let mut wrong = raw.clone();
        let (_, _, contexts) = &mut wrong[0];
        contexts[0].total += 1;
        assert!(encode_complete_lookup(0, [11; 32], &wrong, &wrong).is_err());
        Ok(())
    }
}
