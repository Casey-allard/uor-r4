//! Exact n-gram counts over u16 token streams (claude-412): the shared primitive
//! of the sealed reference R* and P1's count base.
//!
//! This file holds the host side only: the ordered key [`PrimeRouteKey`], the
//! canonical record formats, the per-order SHA-256 (the D85 replay identity),
//! the key-range partition plan and the assembly of partition outputs. The
//! counting itself runs on CUDA (`ngram_count.cu`, `ngram_count_device.rs`);
//! the CPU counter in `ngram_count_oracle.rs` is a test oracle only.
//!
//! Order k (0..=10) counts each target j, k <= j < n, under the context
//! t[j-k..j]. The canonical triple stream of an order is every distinct
//! (key, next) in ascending (key word, next) order with its count.

use crate::{Error, Result};
use sha2::{Digest, Sha256};

/// Token ids are 12-bit: V = 4,096.
pub const NGRAM_VOCABULARY: u32 = 4096;
/// Up to ten context tokens: 12 x 10 = 120 bits, the order in the top byte.
pub const NGRAM_MAX_ORDER: u32 = 10;
/// Bytes per canonical triple: key u128 LE, next u16 LE, count u32 LE.
pub const TRIPLE_BYTES: usize = 22;
/// Bytes per canonical context record: key u128, total u64, N1+, N1, N2, N3+ (u32 each).
pub const CONTEXT_BYTES: usize = 40;
const TRIPLE_DOMAIN: &[u8] = b"uor-r4/ngram-triples/v1\0";
const CONTEXT_DOMAIN: &[u8] = b"uor-r4/ngram-contexts/v1\0";

fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(message.into())
}

/// The ordered context key of the DECISION, section 3, part 2, in its packed-id
/// form. No prime is computed here: the key is an ordered 12-bit packing of
/// (order, t_{-1}..t_{-k}), a bijection on (order, ordered context), and so
/// carries exactly the information of the decision's ordered positional prime
/// key. Slot 1 (the latest token) occupies bits 108..120, slot s bits
/// 120-12s..132-12s, and the order k occupies bits 120..128. Unused slots are 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PrimeRouteKey(u128);

impl PrimeRouteKey {
    /// The key of `context`, given in stream order (oldest first, latest last).
    pub fn from_context(context: &[u16]) -> Result<Self> {
        let order = context.len() as u32;
        if order > NGRAM_MAX_ORDER {
            return Err(invalid(format!(
                "n-gram key holds at most {NGRAM_MAX_ORDER} context tokens, got {order}"
            )));
        }
        let mut word = (order as u128) << 120;
        for (s, &token) in context.iter().rev().enumerate() {
            if u32::from(token) >= NGRAM_VOCABULARY {
                return Err(invalid(format!(
                    "token id {token} is outside the 12-bit vocabulary"
                )));
            }
            word |= (token as u128) << (108 - 12 * s as u32);
        }
        Ok(Self(word))
    }

    /// A stored word, checked: a known order, 12-bit slots and zero unused slots.
    pub fn from_word(word: u128) -> Result<Self> {
        let order = (word >> 120) as u32;
        if order > NGRAM_MAX_ORDER {
            return Err(invalid(format!("n-gram key order {order} exceeds {NGRAM_MAX_ORDER}")));
        }
        let used_bits = 12 * order;
        let unused_mask = ((1u128 << 120) - 1) >> used_bits;
        if word & unused_mask != 0 {
            return Err(invalid("n-gram key has nonzero unused slots"));
        }
        Ok(Self(word))
    }

    pub fn word(self) -> u128 {
        self.0
    }

    pub fn order(self) -> u32 {
        (self.0 >> 120) as u32
    }

    /// Slot s (1 = latest); None unless 1 <= s <= order (and so s <= 10).
    pub fn slot(self, s: u32) -> Option<u16> {
        if s == 0 || s > self.order() || s > NGRAM_MAX_ORDER {
            return None;
        }
        Some(((self.0 >> (120 - 12 * s)) & 0xfff) as u16)
    }

    /// The context in stream order (oldest first).
    pub fn context(self) -> Vec<u16> {
        (1..=self.order()).rev().filter_map(|s| self.slot(s)).collect()
    }

    /// Back-off to order k-1: drop the oldest slot.
    pub fn back_off(self) -> Option<Self> {
        let order = self.order();
        if order == 0 {
            return None;
        }
        let kept = self.0 & (((1u128 << 120) - 1) & !(((1u128 << 120) - 1) >> (12 * (order - 1))));
        Some(Self(kept | (((order - 1) as u128) << 120)))
    }
}

/// One canonical (context key, next token, count) triple.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NgramTriple {
    pub key: u128,
    pub next: u16,
    pub count: u32,
}

/// One context record: total count c(h), N1+(h.) distinct followers, and the
/// followers seen exactly once, exactly twice and three or more times.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NgramContext {
    pub key: u128,
    pub total: u64,
    pub followers: u32,
    pub n1: u32,
    pub n2: u32,
    pub n3_plus: u32,
}

/// Canonical 22-byte encoding of a triple.
pub fn triple_bytes(triple: &NgramTriple) -> [u8; TRIPLE_BYTES] {
    let mut bytes = [0u8; TRIPLE_BYTES];
    bytes[..16].copy_from_slice(&triple.key.to_le_bytes());
    bytes[16..18].copy_from_slice(&triple.next.to_le_bytes());
    bytes[18..].copy_from_slice(&triple.count.to_le_bytes());
    bytes
}

/// Canonical 40-byte encoding of a context record. N3+ is implied
/// (followers - n1 - n2) but carried so a reader needs no rule.
pub fn context_bytes(context: &NgramContext) -> [u8; CONTEXT_BYTES] {
    let mut bytes = [0u8; CONTEXT_BYTES];
    bytes[..16].copy_from_slice(&context.key.to_le_bytes());
    bytes[16..24].copy_from_slice(&context.total.to_le_bytes());
    bytes[24..28].copy_from_slice(&context.followers.to_le_bytes());
    bytes[28..32].copy_from_slice(&context.n1.to_le_bytes());
    bytes[32..36].copy_from_slice(&context.n2.to_le_bytes());
    bytes[36..40].copy_from_slice(&context.n3_plus.to_le_bytes());
    bytes
}

/// What one order's count produced: the replay identity and the census.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NgramOrderDigest {
    pub order: u32,
    pub targets: u64,
    pub grams: u64,
    pub contexts: u64,
    /// n1..n4: grams whose count is exactly 1, 2, 3, 4.
    pub count_of_counts: [u64; 4],
    pub triple_sha256: String,
    pub context_sha256: String,
}

/// The output of one key-range partition, as the device emits it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NgramPartition {
    pub triples: Vec<NgramTriple>,
    pub contexts: Vec<NgramContext>,
    pub count_of_counts: [u64; 4],
}

/// Assembles one order's partitions, in canonical order, into the canonical
/// streams and their SHA-256. Partitions cover disjoint ascending key ranges,
/// so the triple stream is their concatenation. A context may straddle a
/// boundary only at order 0 (the empty context); its records are added, since
/// every field is additive over disjoint follower sets. The context records
/// and n1..n4 are pure functions of the triples: the assembler derives them
/// from each partition's triples and refuses a partition whose own differ.
pub struct NgramAssembler {
    order: u32,
    keep: bool,
    triple_hash: Sha256,
    context_hash: Sha256,
    last_triple: Option<(u128, u16)>,
    pending_context: Option<NgramContext>,
    targets: u64,
    grams: u64,
    contexts: u64,
    count_of_counts: [u64; 4],
    pub triples: Vec<NgramTriple>,
    pub context_records: Vec<NgramContext>,
}

impl NgramAssembler {
    /// `keep` retains the streams in memory (tests); otherwise only hashes.
    pub fn new(order: u32, keep: bool) -> Result<Self> {
        if order > NGRAM_MAX_ORDER {
            return Err(invalid(format!("n-gram order {order} exceeds {NGRAM_MAX_ORDER}")));
        }
        let mut triple_hash = Sha256::new();
        triple_hash.update(TRIPLE_DOMAIN);
        triple_hash.update([order as u8]);
        let mut context_hash = Sha256::new();
        context_hash.update(CONTEXT_DOMAIN);
        context_hash.update([order as u8]);
        Ok(Self {
            order,
            keep,
            triple_hash,
            context_hash,
            last_triple: None,
            pending_context: None,
            targets: 0,
            grams: 0,
            contexts: 0,
            count_of_counts: [0; 4],
            triples: Vec::new(),
            context_records: Vec::new(),
        })
    }

    /// Append the next partition. Refuses any break of canonical order and any
    /// context record or n1..n4 that differs from the one its triples imply.
    pub fn push(&mut self, partition: &NgramPartition) -> Result<()> {
        let mut last = self.last_triple;
        for triple in &partition.triples {
            if (triple.key >> 120) as u32 != self.order {
                return Err(invalid("n-gram triple carries another order's key"));
            }
            if u32::from(triple.next) >= NGRAM_VOCABULARY {
                return Err(invalid("n-gram triple next token is outside the 12-bit vocabulary"));
            }
            let at = (triple.key, triple.next);
            if last.is_some_and(|before| before >= at) {
                return Err(invalid("n-gram triples are not in strictly ascending order"));
            }
            if triple.count == 0 {
                return Err(invalid("n-gram triple has a zero count"));
            }
            last = Some(at);
        }
        for context in &partition.contexts {
            if (context.key >> 120) as u32 != self.order {
                return Err(invalid("n-gram context carries another order's key"));
            }
            let parts = u64::from(context.n1) + u64::from(context.n2) + u64::from(context.n3_plus);
            if u64::from(context.followers) != parts {
                return Err(invalid("n-gram context followers differ from n1 + n2 + n3+"));
            }
        }
        let derived_count_of_counts = check_census(&partition.triples, &partition.contexts)?;
        if partition.count_of_counts != derived_count_of_counts {
            return Err(invalid("n-gram count-of-counts differ from the ones the triples imply"));
        }
        if self.order >= 1 {
            let seam = self.pending_context.zip(partition.contexts.first());
            if seam.is_some_and(|(pending, first)| pending.key == first.key) {
                return Err(invalid("an order >= 1 n-gram context straddles a partition seam"));
            }
        }
        self.last_triple = last;
        for triple in &partition.triples {
            self.triple_hash.update(triple_bytes(triple));
            self.targets += u64::from(triple.count);
            self.grams += 1;
        }
        for (slot, value) in self.count_of_counts.iter_mut().zip(partition.count_of_counts) {
            *slot += value;
        }
        for context in &partition.contexts {
            match self.pending_context.take() {
                Some(mut pending) if pending.key == context.key => {
                    pending.total += context.total;
                    pending.followers += context.followers;
                    pending.n1 += context.n1;
                    pending.n2 += context.n2;
                    pending.n3_plus += context.n3_plus;
                    self.pending_context = Some(pending);
                }
                Some(pending) => {
                    if pending.key > context.key {
                        return Err(invalid("n-gram contexts are not in ascending order"));
                    }
                    self.emit_context(pending);
                    self.pending_context = Some(*context);
                }
                None => self.pending_context = Some(*context),
            }
        }
        if self.keep {
            self.triples.extend_from_slice(&partition.triples);
        }
        Ok(())
    }

    fn emit_context(&mut self, context: NgramContext) {
        self.context_hash.update(context_bytes(&context));
        self.contexts += 1;
        if self.keep {
            self.context_records.push(context);
        }
    }

    pub fn finish(mut self) -> (NgramOrderDigest, Vec<NgramTriple>, Vec<NgramContext>) {
        if let Some(pending) = self.pending_context.take() {
            self.emit_context(pending);
        }
        let digest = NgramOrderDigest {
            order: self.order,
            targets: self.targets,
            grams: self.grams,
            contexts: self.contexts,
            count_of_counts: self.count_of_counts,
            triple_sha256: format!("{:x}", self.triple_hash.finalize()),
            context_sha256: format!("{:x}", self.context_hash.finalize()),
        };
        (digest, self.triples, self.context_records)
    }
}

/// Checks, in one pass and without a second vector, that `contexts` are
/// exactly the records a run of canonical triples implies (one record per run
/// of equal keys), and returns the n1..n4 the triples imply.
fn check_census(triples: &[NgramTriple], contexts: &[NgramContext]) -> Result<[u64; 4]> {
    let differ = || invalid("n-gram context records differ from the ones the triples imply");
    let mut count_of_counts = [0u64; 4];
    let mut running: Option<NgramContext> = None;
    let mut next = 0usize;
    for triple in triples {
        if (1..=4).contains(&triple.count) {
            count_of_counts[triple.count as usize - 1] += 1;
        }
        let h = match running.as_mut() {
            Some(h) if h.key == triple.key => h,
            _ => {
                if let Some(done) = running.take() {
                    if contexts.get(next) != Some(&done) {
                        return Err(differ());
                    }
                    next += 1;
                }
                running.insert(NgramContext {
                    key: triple.key,
                    total: 0,
                    followers: 0,
                    n1: 0,
                    n2: 0,
                    n3_plus: 0,
                })
            }
        };
        h.total += u64::from(triple.count);
        h.followers += 1;
        match triple.count {
            1 => h.n1 += 1,
            2 => h.n2 += 1,
            _ => h.n3_plus += 1,
        }
    }
    if let Some(done) = running {
        if contexts.get(next) != Some(&done) {
            return Err(differ());
        }
        next += 1;
    }
    if next != contexts.len() {
        return Err(differ());
    }
    Ok(count_of_counts)
}

/// The kernels' u32 index products (2g for keys, 4c + 3 for context records)
/// stay below 2^32 while a partition holds at most 2^30 targets.
pub const NGRAM_MAX_PARTITION_TARGETS: u64 = 1 << 30;
/// The kernels' grid-stride and tile arithmetic wraps above 2^32 - 65,536
/// positions; the resident stream is capped well below, at 2^31 tokens.
pub const NGRAM_MAX_STREAM_TOKENS: u64 = 1 << 31;

/// The target count of a partition the device may count: at most 2^30.
pub fn partition_target_bound(range: &PartitionRange) -> Result<u32> {
    if range.targets > NGRAM_MAX_PARTITION_TARGETS {
        return Err(invalid(format!(
            "n-gram partition {}..{} holds {} targets, above the 2^30 device bound",
            range.lo, range.hi, range.targets
        )));
    }
    Ok(range.targets as u32)
}

/// The length of a stream the device may hold: 1..=2^31 tokens.
pub fn stream_length_bound(len: usize) -> Result<u32> {
    if len == 0 || len as u64 > NGRAM_MAX_STREAM_TOKENS {
        return Err(invalid(format!("n-gram stream needs 1..=2^31 tokens, got {len}")));
    }
    Ok(len as u32)
}

/// One key-range partition: the partition token takes values lo..hi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PartitionRange {
    pub lo: u32,
    pub hi: u32,
    pub targets: u64,
}

/// Greedy plan over the partition token's histogram: each partition is the
/// longest run of token values whose targets fit `budget`, or a single token
/// value that alone exceeds it. The plan covers 0..4096 in order.
pub fn plan_partitions(histogram: &[u64], budget: u64) -> Result<Vec<PartitionRange>> {
    if histogram.len() != NGRAM_VOCABULARY as usize || budget == 0 {
        return Err(invalid("partition plan needs a 4,096-bin histogram and a positive budget"));
    }
    let mut plan = Vec::new();
    let mut lo = 0u32;
    let mut sum = 0u64;
    for (token, &count) in histogram.iter().enumerate() {
        let token = token as u32;
        if token > lo && sum + count > budget {
            plan.push(PartitionRange { lo, hi: token, targets: sum });
            lo = token;
            sum = 0;
        }
        sum += count;
    }
    plan.push(PartitionRange { lo, hi: NGRAM_VOCABULARY, targets: sum });
    if let Some(p) = plan.iter().find(|p| p.targets > NGRAM_MAX_PARTITION_TARGETS) {
        return Err(invalid(format!(
            "partition plan: token values {}..{} hold {} targets, above the 2^30 partition bound",
            p.lo, p.hi, p.targets
        )));
    }
    Ok(plan)
}

/// The partition token of target j under order k: slot 1 (t[j-1]) for k >= 1,
/// the next token t[j] for k = 0. Its value is the most significant sort digit.
pub fn partition_offset(order: u32) -> usize {
    usize::from(order >= 1)
}

/// The 64-byte UORT header, then little-endian u16 tokens.
pub fn read_uort_tokens(bytes: &[u8]) -> Result<Vec<u16>> {
    if bytes.len() < 64 || &bytes[..4] != b"UORT" {
        return Err(invalid("token stream lacks the 64-byte UORT header"));
    }
    let count = u64::from_le_bytes(bytes[8..16].try_into().expect("8 bytes"));
    let payload = &bytes[64..];
    let want = count
        .checked_mul(2)
        .ok_or_else(|| invalid(format!("UORT header names {count} tokens, past the u64 byte bound")))?;
    if payload.len() as u64 != want {
        return Err(invalid(format!(
            "UORT header names {count} tokens but the payload holds {} bytes",
            payload.len()
        )));
    }
    Ok(payload
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect())
}

/// The document-start token. Streams frame each document with BOS = 0 at its
/// start and EOS = 1 at its end.
pub const NGRAM_BOS: u16 = 0;

/// Whether order `k` counts target `j` (k <= j): the target is not a BOS, and
/// no BOS lies inside the context except in its oldest slot `tokens[j - k]`,
/// so no gram reaches back across a document start.
pub fn ngram_admitted(tokens: &[u16], j: usize, k: usize) -> bool {
    debug_assert!(k <= j, "target {j} has no full order-{k} context");
    tokens[j] != NGRAM_BOS && (1..k).all(|s| tokens[j - s] != NGRAM_BOS)
}

/// Refuse any id outside the 12-bit vocabulary before it reaches a kernel.
pub fn check_tokens(tokens: &[u16]) -> Result<()> {
    match tokens.iter().position(|&t| u32::from(t) >= NGRAM_VOCABULARY) {
        Some(at) => Err(invalid(format!(
            "token {} at position {at} is outside the 12-bit vocabulary",
            tokens[at]
        ))),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ngram_count_oracle::{oracle_count, oracle_partition, plant_one_count};

    #[test]
    fn ngram_count_key_round_trip_and_back_off() {
        for context in [&[][..], &[7], &[0, 4095], &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]] {
            let key = PrimeRouteKey::from_context(context).unwrap();
            assert_eq!(key.order() as usize, context.len());
            assert_eq!(key.context(), context);
            assert_eq!(PrimeRouteKey::from_word(key.word()).unwrap(), key);
            match key.back_off() {
                None => assert!(context.is_empty()),
                Some(shorter) => {
                    assert_eq!(shorter.context(), &context[1..]);
                    assert_eq!(shorter, PrimeRouteKey::from_context(&context[1..]).unwrap());
                }
            }
        }
        let key = PrimeRouteKey::from_context(&[11, 22, 33]).unwrap();
        assert_eq!((key.slot(1), key.slot(2), key.slot(3)), (Some(33), Some(22), Some(11)));
        assert_eq!((key.slot(0), key.slot(4), key.slot(11)), (None, None, None));
        let full = PrimeRouteKey::from_context(&[1; 10]).unwrap();
        assert_eq!((full.slot(10), full.slot(11)), (Some(1), None));
        assert!(PrimeRouteKey::from_context(&[4096]).is_err());
        assert!(PrimeRouteKey::from_context(&[0; 11]).is_err());
        assert!(PrimeRouteKey::from_word(key.word() | 1).is_err());
        assert!(PrimeRouteKey::from_word(11u128 << 120).is_err());
    }

    #[test]
    fn ngram_count_key_preserves_order() {
        let (a, b, c) = (17u16, 300u16, 4001u16);
        let abc = PrimeRouteKey::from_context(&[a, b, c]).unwrap();
        let cba = PrimeRouteKey::from_context(&[c, b, a]).unwrap();
        assert_ne!(abc, cba, "\"a b c\" and \"c b a\" must give different keys");
        assert_ne!(
            PrimeRouteKey::from_context(&[a, b]).unwrap(),
            PrimeRouteKey::from_context(&[b, a]).unwrap()
        );
        let the = 5u16;
        let x = 9u16;
        assert_ne!(
            PrimeRouteKey::from_context(&[the, x, the]).unwrap(),
            PrimeRouteKey::from_context(&[x, the]).unwrap()
        );
        // Counted: the stream "a b c | c b a" has order-3 context a b c once
        // and c b a once, as two distinct contexts (EOS = 1 separates them:
        // a BOS target would not be counted).
        let stream = [a, b, c, 1, c, b, a, 1];
        let (digest, _, contexts) = oracle_count(&stream, 3).unwrap();
        assert!(contexts.iter().any(|h| h.key == abc.word()));
        assert!(contexts.iter().any(|h| h.key == cba.word()));
        assert_eq!(digest.contexts, 5);
    }

    /// Hand fixture: "1 2 1 2 1 3 1 2", n = 8.
    #[test]
    fn ngram_count_oracle_hand_fixture() {
        let stream = [1u16, 2, 1, 2, 1, 3, 1, 2];
        let key = |c: &[u16]| PrimeRouteKey::from_context(c).unwrap().word();
        // Order 0: 1 x4, 2 x3, 3 x1, one context.
        let (d0, t0, h0) = oracle_count(&stream, 0).unwrap();
        assert_eq!(
            t0,
            vec![
                NgramTriple { key: key(&[]), next: 1, count: 4 },
                NgramTriple { key: key(&[]), next: 2, count: 3 },
                NgramTriple { key: key(&[]), next: 3, count: 1 },
            ]
        );
        assert_eq!(
            h0,
            vec![NgramContext { key: key(&[]), total: 8, followers: 3, n1: 1, n2: 0, n3_plus: 2 }]
        );
        assert_eq!((d0.targets, d0.count_of_counts), (8, [1, 0, 1, 1]));
        // Order 1: targets j = 1..8. Pairs (1,2) x3, (2,1) x2, (1,3) x1, (3,1) x1.
        let (d1, t1, h1) = oracle_count(&stream, 1).unwrap();
        assert_eq!(
            t1,
            vec![
                NgramTriple { key: key(&[1]), next: 2, count: 3 },
                NgramTriple { key: key(&[1]), next: 3, count: 1 },
                NgramTriple { key: key(&[2]), next: 1, count: 2 },
                NgramTriple { key: key(&[3]), next: 1, count: 1 },
            ]
        );
        assert_eq!(
            h1,
            vec![
                NgramContext { key: key(&[1]), total: 4, followers: 2, n1: 1, n2: 0, n3_plus: 1 },
                NgramContext { key: key(&[2]), total: 2, followers: 1, n1: 0, n2: 1, n3_plus: 0 },
                NgramContext { key: key(&[3]), total: 1, followers: 1, n1: 1, n2: 0, n3_plus: 0 },
            ]
        );
        assert_eq!((d1.targets, d1.count_of_counts), (7, [2, 1, 1, 0]));
        // Order 2: (1 2 -> 1) x2, (2 1 -> 2) x1, (2 1 -> 3) x1, (1 3 -> 1), (3 1 -> 2).
        let (d2, t2, _) = oracle_count(&stream, 2).unwrap();
        assert_eq!(d2.targets, 6);
        assert_eq!(d2.count_of_counts, [4, 1, 0, 0]);
        assert_eq!(t2[0], NgramTriple { key: key(&[2, 1]), next: 2, count: 1 });
        assert_eq!(t2.iter().map(|t| t.count).sum::<u32>(), 6);
        // Order 8 needs j >= 8, and n = 8: no target.
        let (d8, t8, _) = oracle_count(&stream, 8).unwrap();
        assert_eq!((d8.targets, t8.len()), (0, 0));
    }

    #[test]
    fn ngram_count_oracle_respects_document_boundaries() {
        let stream = [0u16, 5, 6, 7, 1, 0, 5, 6, 1];
        let (_, triples, _) = oracle_count(&stream, 2).unwrap();
        let key = |a: u16, b: u16| PrimeRouteKey::from_context(&[a, b]).unwrap().word();
        let count = |a: u16, b: u16, next: u16| {
            triples
                .iter()
                .find(|t| t.key == key(a, b) && t.next == next)
                .map(|t| t.count)
        };
        assert_eq!(count(0, 5, 6), Some(2));
        assert_eq!(count(7, 1, 0), None);
        assert_eq!(count(1, 0, 5), None);
        assert!(triples.iter().all(|t| t.next != NGRAM_BOS));
        let total: u32 = triples.iter().map(|t| t.count).sum();
        assert_eq!((triples.len(), total), (4, 5));
        // Order 0 drops only the BOS targets.
        let (_, unigrams, _) = oracle_count(&stream, 0).unwrap();
        let total: u32 = unigrams.iter().map(|t| t.count).sum();
        assert_eq!(total, 7);
    }

    #[test]
    fn ngram_count_partitions_assemble_to_the_whole() {
        let mut state = 0x412u64;
        let stream: Vec<u16> = (0..20_000)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                // Skewed: most mass on a few ids so high orders repeat.
                let r = (state >> 20) as u32;
                (if r % 4 == 0 { r % 4096 } else { r % 13 }) as u16
            })
            .collect();
        // r % 13 plants BOS = 0 at about one position in eighteen, so the
        // partitions and the assembler see document boundaries.
        assert!(stream.iter().filter(|&&t| t == NGRAM_BOS).count() > 500);
        for order in [0u32, 1, 2, 5] {
            let (whole, triples, contexts) = oracle_count(&stream, order).unwrap();
            let offset = partition_offset(order);
            let mut histogram = vec![0u64; 4096];
            for j in order as usize..stream.len() {
                if ngram_admitted(&stream, j, order as usize) {
                    histogram[stream[j - offset] as usize] += 1;
                }
            }
            for budget in [u64::MAX, 3_000, 500] {
                let plan = plan_partitions(&histogram, budget).unwrap();
                assert_eq!(plan.first().unwrap().lo, 0);
                assert_eq!(plan.last().unwrap().hi, 4096);
                let mut assembler = NgramAssembler::new(order, true).unwrap();
                for range in &plan {
                    assert!(range.targets <= budget || range.hi == range.lo + 1);
                    assembler
                        .push(&oracle_partition(&stream, order, range.lo, range.hi).unwrap())
                        .unwrap();
                }
                let (digest, t, h) = assembler.finish();
                assert_eq!(digest, whole, "order {order} budget {budget}");
                assert_eq!((t, h), (triples.clone(), contexts.clone()));
            }
        }
    }

    /// A single partition token value cannot be split, so the plan refuses one
    /// that alone holds more than the device's 2^30 targets, at plan time.
    #[test]
    fn ngram_count_plan_refuses_a_partition_past_the_target_limit() {
        let mut histogram = vec![0u64; 4096];
        histogram[7] = NGRAM_MAX_PARTITION_TARGETS;
        histogram[8] = 5;
        let plan = plan_partitions(&histogram, NGRAM_MAX_PARTITION_TARGETS).expect("2^30 admitted");
        assert!(plan.iter().any(|p| p.targets == NGRAM_MAX_PARTITION_TARGETS));
        assert!(plan.iter().all(|p| p.targets <= NGRAM_MAX_PARTITION_TARGETS));
        histogram[7] = NGRAM_MAX_PARTITION_TARGETS + 1;
        let refusal = plan_partitions(&histogram, NGRAM_MAX_PARTITION_TARGETS).unwrap_err();
        assert!(refusal.to_string().contains("above the 2^30 partition bound"), "{refusal}");
    }

    /// A consistent plant (count, context record and n1..n4 adjusted together)
    /// moves the triple SHA: the SHA covers the count field.
    #[test]
    fn ngram_count_assembler_refuses_disorder_and_the_sha_covers_a_planted_count() {
        let stream = [1u16, 2, 1, 2, 1, 3, 1, 2];
        let (digest, triples, _) = oracle_count(&stream, 1).unwrap();
        let mut planted = oracle_partition(&stream, 1, 0, 4096).unwrap();
        plant_one_count(&mut planted, 1);
        let mut assembler = NgramAssembler::new(1, false).unwrap();
        assembler.push(&planted).unwrap();
        let (planted_digest, ..) = assembler.finish();
        assert_ne!(planted_digest.triple_sha256, digest.triple_sha256);
        let mut swapped = oracle_partition(&stream, 1, 0, 4096).unwrap();
        swapped.triples.swap(0, 1);
        assert!(NgramAssembler::new(1, false).unwrap().push(&swapped).is_err());
        assert_eq!(triples.len(), 4);
        assert!(check_tokens(&[0, 4095]).is_ok());
        assert!(check_tokens(&[0, 4096]).is_err());
    }

    fn refusal(order: u32, partitions: &[NgramPartition]) -> String {
        let mut assembler = NgramAssembler::new(order, false).unwrap();
        for partition in partitions {
            if let Err(error) = assembler.push(partition) {
                return error.to_string();
            }
        }
        panic!("the assembler admitted every partition");
    }

    const HAND: [u16; 8] = [1, 2, 1, 2, 1, 3, 1, 2];

    #[test]
    fn ngram_count_assembler_refuses_a_planted_count_of_counts() {
        let mut partition = oracle_partition(&HAND, 1, 0, 4096).unwrap();
        partition.count_of_counts[0] += 1;
        assert!(refusal(1, &[partition]).contains("count-of-counts differ"));
    }

    #[test]
    fn ngram_count_assembler_refuses_a_corrupted_context_n1() {
        let mut partition = oracle_partition(&HAND, 1, 0, 4096).unwrap();
        // Context [1]: followers 2 = n1 1 + n3+ 1. Move one follower from n3+
        // to n1, so followers = n1 + n2 + n3+ still holds and only the
        // derivation from the triples can see it.
        assert_eq!((partition.contexts[0].n1, partition.contexts[0].n3_plus), (1, 1));
        partition.contexts[0].n1 += 1;
        partition.contexts[0].n3_plus -= 1;
        assert!(refusal(1, &[partition]).contains("context records differ"));
    }

    /// A partition with one context record more, or one fewer, than its
    /// triples imply is refused by the streamed comparison.
    #[test]
    fn ngram_count_assembler_refuses_an_extra_or_missing_context() {
        let mut extra = oracle_partition(&HAND, 1, 0, 4096).unwrap();
        let mut last = *extra.contexts.last().unwrap();
        last.key += 1;
        extra.contexts.push(last);
        assert!(refusal(1, &[extra]).contains("context records differ"));
        let mut missing = oracle_partition(&HAND, 1, 0, 4096).unwrap();
        missing.contexts.pop();
        assert!(refusal(1, &[missing]).contains("context records differ"));
    }

    #[test]
    fn ngram_count_assembler_admits_unmodified_oracle_partitions() {
        for order in 0..=3 {
            for (lo, hi) in [(0, 4096), (0, 2), (2, 4096)] {
                let partition = oracle_partition(&HAND, order, lo, hi).unwrap();
                NgramAssembler::new(order, false).unwrap().push(&partition).unwrap();
            }
        }
    }

    /// Two partitions whose triples ascend, each self-consistent, that share an
    /// order-1 context across their seam.
    #[test]
    fn ngram_count_assembler_refuses_an_order_one_context_across_a_seam() {
        let key = PrimeRouteKey::from_context(&[7]).unwrap().word();
        let half = |next: u16| NgramPartition {
            triples: vec![NgramTriple { key, next, count: 1 }],
            contexts: vec![NgramContext { key, total: 1, followers: 1, n1: 1, n2: 0, n3_plus: 0 }],
            count_of_counts: [1, 0, 0, 0],
        };
        assert!(refusal(1, &[half(1), half(2)]).contains("straddles a partition seam"));
        // Order 0's empty context may straddle.
        let empty = PrimeRouteKey::from_context(&[]).unwrap().word();
        let mut assembler = NgramAssembler::new(0, true).unwrap();
        for next in [1u16, 2] {
            let mut part = half(next);
            part.triples[0].key = empty;
            part.contexts[0].key = empty;
            assembler.push(&part).unwrap();
        }
        let (digest, _, contexts) = assembler.finish();
        assert_eq!((digest.contexts, contexts[0].followers, contexts[0].total), (1, 2, 2));
    }

    #[test]
    fn ngram_count_assembler_refuses_a_next_token_outside_the_vocabulary() {
        let mut partition = oracle_partition(&HAND, 1, 0, 4096).unwrap();
        let last = partition.triples.len() - 1;
        partition.triples[last].next = 4096;
        assert!(refusal(1, &[partition]).contains("next token is outside"));
    }

    #[test]
    fn ngram_count_assembler_refuses_a_context_of_another_order() {
        let mut partition = oracle_partition(&HAND, 1, 0, 4096).unwrap();
        partition.contexts[0].key ^= 3u128 << 120;
        assert!(refusal(1, &[partition]).contains("context carries another order"));
    }

    #[test]
    fn ngram_count_assembler_refuses_followers_unequal_to_their_parts() {
        let mut partition = oracle_partition(&HAND, 1, 0, 4096).unwrap();
        partition.contexts[0].followers += 1;
        assert!(refusal(1, &[partition]).contains("followers differ from n1 + n2 + n3+"));
    }

    #[test]
    fn ngram_count_uort_header_overflow_is_an_error() {
        let mut bytes = vec![0u8; 64];
        bytes[..4].copy_from_slice(b"UORT");
        bytes[8..16].copy_from_slice(&(1u64 << 63).to_le_bytes());
        assert!(read_uort_tokens(&bytes).is_err());
        bytes[8..16].copy_from_slice(&1u64.to_le_bytes());
        bytes.extend_from_slice(&7u16.to_le_bytes());
        assert_eq!(read_uort_tokens(&bytes).unwrap(), vec![7]);
    }

    #[test]
    fn ngram_count_device_bounds_refuse_past_the_u32_index_range() {
        let range = |targets| PartitionRange { lo: 0, hi: 1, targets };
        assert_eq!(partition_target_bound(&range(1 << 30)).unwrap(), 1 << 30);
        assert!(partition_target_bound(&range((1 << 30) + 1)).is_err());
        assert_eq!(stream_length_bound(1 << 31).unwrap(), 1 << 31);
        assert!(stream_length_bound((1 << 31) + 1).is_err());
        assert!(stream_length_bound(0).is_err());
    }
}
