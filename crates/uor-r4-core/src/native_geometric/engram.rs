//! Flat Integer Engram Table 2.0 (Milestone 11).
//!
//! Provides $O(1)$ constant-time lookup for frequent bigram, trigram, and conditioned skip-bigram
//! collocations with zero runtime matrix multiplications, zero runtime floats, and zero dynamic
//! heap allocations on serving hot paths.
//!
//! Key: 32-bit integer hash of context token tuple:
//! - Bigram: (w_{t-1}, w_t)
//! - Trigram: (w_{t-2}, w_{t-1}, w_t)
//! - Skip-Bigram: (w_{t-k}, w_t, condition) for k in {2, 4, 8}
//! Value: Up to 4 candidate tokens with fixed-point Q1.15 scores: `[(u32, i16); 4]`.
//! Memory footprint: 32,768 slots * 12 bytes = ~393 KB (resident in M1 L2 cache).
//! Serving probe: flat open-addressing linear probe (< 15 ns, zero allocations).

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Maximum number of candidate continuations per collocation context.
pub const MAX_ENGRAM_CANDIDATES: usize = 4;

/// Default static capacity (slots) of the flat engram hash table (32,768 slots = ~393 KB).
pub const DEFAULT_ENGRAM_CAPACITY: usize = 32768;

/// Maximum number of stored collocation contexts (load factor ~0.31 for fast O(1) probing and <= 500 KB footprint).
pub const MAX_ENGRAM_COLLOCATIONS: usize = 10240;

/// A single entry in the flat engram hash table (exactly 12 bytes).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngramEntry {
    /// 32-bit integer hash key of context (0 denotes unoccupied slot).
    pub key: u32,
    /// Offset in candidate storage pool.
    pub candidate_offset: u32,
    /// Number of valid continuation candidates (0..=MAX_ENGRAM_CANDIDATES).
    pub count: u8,
    /// Syntactic condition register / parity bits.
    pub condition: u8,
    /// Reserved alignment bytes to guarantee exact 12-byte struct layout.
    pub _reserved: [u8; 2],
}

impl Default for EngramEntry {
    fn default() -> Self {
        Self {
            key: 0,
            candidate_offset: 0,
            count: 0,
            condition: 0,
            _reserved: [0; 2],
        }
    }
}

/// Compute 32-bit integer hash for bigram context (w_{t-1}, w_t).
///
/// Guaranteed to return non-zero so 0 remains the unoccupied slot sentinel.
#[inline]
pub fn hash_bigram(w_prev: u32, w_curr: u32) -> u32 {
    let mut h = 0x811c9dc5_u32 ^ 2;
    h = (h ^ w_prev).wrapping_mul(0x01000193);
    h = (h ^ w_curr).wrapping_mul(0x01000193);
    h ^= h >> 16;
    h = h.wrapping_mul(0x45d9f3b);
    h ^= h >> 16;
    if h == 0 {
        1
    } else {
        h
    }
}

/// Compute 32-bit integer hash for trigram context (w_{t-2}, w_{t-1}, w_t).
///
/// Guaranteed to return non-zero so 0 remains the unoccupied slot sentinel.
#[inline]
pub fn hash_trigram(w_prev2: u32, w_prev: u32, w_curr: u32) -> u32 {
    let mut h = 0x811c9dc5_u32 ^ 3;
    h = (h ^ w_prev2).wrapping_mul(0x01000193);
    h = (h ^ w_prev).wrapping_mul(0x01000193);
    h = (h ^ w_curr).wrapping_mul(0x01000193);
    h ^= h >> 16;
    h = h.wrapping_mul(0x45d9f3b);
    h ^= h >> 16;
    if h == 0 {
        1
    } else {
        h
    }
}

/// Compute 32-bit integer hash for conditioned skip-bigram context (w_{t-k}, w_t, condition).
///
/// Guaranteed to return non-zero so 0 remains the unoccupied slot sentinel.
#[inline]
pub fn hash_skip(w_skip: u32, w_curr: u32, condition: u8) -> u32 {
    let mut h = 0x811c9dc5_u32 ^ 0x53; // 'S' domain separation for skip-bigram
    h = (h ^ (condition as u32)).wrapping_mul(0x01000193);
    h = (h ^ w_skip).wrapping_mul(0x01000193);
    h = (h ^ w_curr).wrapping_mul(0x01000193);
    h ^= h >> 16;
    h = h.wrapping_mul(0x45d9f3b);
    h ^= h >> 16;
    if h == 0 {
        1
    } else {
        h
    }
}

/// Compute 32-bit integer hash for 4-gram context (w_{t-3}, w_{t-2}, w_{t-1}, w_t).
///
/// Guaranteed to return non-zero so 0 remains the unoccupied slot sentinel.
#[inline]
pub fn hash_4gram(w_prev3: u32, w_prev2: u32, w_prev: u32, w_curr: u32) -> u32 {
    let mut h = 0x811c9dc5_u32 ^ 4;
    h = (h ^ w_prev3).wrapping_mul(0x01000193);
    h = (h ^ w_prev2).wrapping_mul(0x01000193);
    h = (h ^ w_prev).wrapping_mul(0x01000193);
    h = (h ^ w_curr).wrapping_mul(0x01000193);
    h ^= h >> 16;
    h = h.wrapping_mul(0x45d9f3b);
    h ^= h >> 16;
    if h == 0 {
        1
    } else {
        h
    }
}

/// Compute 32-bit integer hash for 5-gram context (w_{t-4}, w_{t-3}, w_{t-2}, w_{t-1}, w_t).
///
/// Guaranteed to return non-zero so 0 remains the unoccupied slot sentinel.
#[inline]
pub fn hash_5gram(w_prev4: u32, w_prev3: u32, w_prev2: u32, w_prev: u32, w_curr: u32) -> u32 {
    let mut h = 0x811c9dc5_u32 ^ 5;
    h = (h ^ w_prev4).wrapping_mul(0x01000193);
    h = (h ^ w_prev3).wrapping_mul(0x01000193);
    h = (h ^ w_prev2).wrapping_mul(0x01000193);
    h = (h ^ w_prev).wrapping_mul(0x01000193);
    h = (h ^ w_curr).wrapping_mul(0x01000193);
    h ^= h >> 16;
    h = h.wrapping_mul(0x45d9f3b);
    h ^= h >> 16;
    if h == 0 {
        1
    } else {
        h
    }
}

/// Flat integer engram table for zero-allocation O(1) collocation retrieval.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngramTable {
    pub entries: Vec<EngramEntry>,
    #[serde(default)]
    pub candidates: Vec<(u32, i16)>,
    #[serde(default)]
    pub mask: usize,
    #[serde(default)]
    pub count: usize,
}

impl Default for EngramTable {
    fn default() -> Self {
        Self::with_capacity(DEFAULT_ENGRAM_CAPACITY)
    }
}

impl EngramTable {
    /// Create an empty table with given slot capacity (rounded up to power of 2).
    pub fn with_capacity(capacity: usize) -> Self {
        let cap = capacity.next_power_of_two().max(16);
        Self {
            entries: vec![EngramEntry::default(); cap],
            candidates: Vec::new(),
            mask: cap - 1,
            count: 0,
        }
    }

    /// Total memory footprint of the engram table in bytes.
    #[inline]
    pub fn memory_footprint(&self) -> usize {
        self.entries.len() * std::mem::size_of::<EngramEntry>()
            + self.candidates.len() * std::mem::size_of::<(u32, i16)>()
            + std::mem::size_of::<Self>()
    }

    /// Insert or update an engram entry with condition and up to 4 candidates.
    pub fn insert_conditioned(&mut self, key: u32, condition: u8, cands: &[(u32, i16)]) -> bool {
        if key == 0 || cands.is_empty() {
            return false;
        }
        if self.entries.is_empty() {
            *self = Self::with_capacity(DEFAULT_ENGRAM_CAPACITY);
        }
        let cap = self.entries.len();
        if self.count >= cap / 2 {
            return false;
        }
        let mask = if self.mask != 0 { self.mask } else { cap - 1 };
        let mut slot = (key as usize) & mask;
        let c_len = cands.len().min(MAX_ENGRAM_CANDIDATES);
        for _ in 0..16 {
            let entry = &mut self.entries[slot];
            if entry.key == 0 {
                let offset = self.candidates.len() as u32;
                self.candidates.extend_from_slice(&cands[..c_len]);
                entry.key = key;
                entry.candidate_offset = offset;
                entry.count = c_len as u8;
                entry.condition = condition;
                self.count += 1;
                return true;
            } else if entry.key == key {
                let old_off = entry.candidate_offset as usize;
                let old_cnt = entry.count as usize;
                if old_cnt == c_len && old_off + old_cnt <= self.candidates.len() {
                    self.candidates[old_off..old_off + c_len].copy_from_slice(&cands[..c_len]);
                } else {
                    let offset = self.candidates.len() as u32;
                    self.candidates.extend_from_slice(&cands[..c_len]);
                    entry.candidate_offset = offset;
                    entry.count = c_len as u8;
                }
                entry.condition = condition;
                return true;
            }
            slot = (slot + 1) & mask;
        }
        false
    }

    /// Insert or update an engram entry with up to 4 candidates (default condition 0).
    pub fn insert(&mut self, key: u32, candidates: &[(u32, i16)]) -> bool {
        self.insert_conditioned(key, 0, candidates)
    }

    /// O(1) constant-time lookup by 32-bit hash key.
    ///
    /// Executes via a bounded flat array probe (< 10 ns):
    /// zero runtime multiplications, zero runtime floats, zero dynamic heap allocations.
    #[inline]
    pub fn lookup(&self, key: u32) -> Option<&[(u32, i16)]> {
        if key == 0 || self.entries.is_empty() {
            return None;
        }
        let mask = if self.mask != 0 {
            self.mask
        } else {
            self.entries.len().saturating_sub(1)
        };
        let mut slot = (key as usize) & mask;
        for _ in 0..16 {
            let entry = &self.entries[slot];
            if entry.key == key {
                let start = entry.candidate_offset as usize;
                let end = start + (entry.count as usize);
                if end <= self.candidates.len() {
                    return Some(&self.candidates[start..end]);
                }
                return None;
            }
            if entry.key == 0 {
                return None;
            }
            slot = (slot + 1) & mask;
        }
        None
    }

    /// Lookup candidates for a bigram context (w_{t-1}, w_t).
    #[inline]
    pub fn lookup_bigram(&self, w_prev: u32, w_curr: u32) -> Option<&[(u32, i16)]> {
        let key = hash_bigram(w_prev, w_curr);
        self.lookup(key)
    }

    /// Lookup candidates for a trigram context (w_{t-2}, w_{t-1}, w_t).
    #[inline]
    pub fn lookup_trigram(&self, w_prev2: u32, w_prev: u32, w_curr: u32) -> Option<&[(u32, i16)]> {
        let key = hash_trigram(w_prev2, w_prev, w_curr);
        self.lookup(key)
    }

    /// Lookup candidates for a 4-gram context (w_{t-3}, w_{t-2}, w_{t-1}, w_t).
    #[inline]
    pub fn lookup_4gram(
        &self,
        w_prev3: u32,
        w_prev2: u32,
        w_prev: u32,
        w_curr: u32,
    ) -> Option<&[(u32, i16)]> {
        let key = hash_4gram(w_prev3, w_prev2, w_prev, w_curr);
        self.lookup(key)
    }

    /// Lookup candidates for a 5-gram context (w_{t-4}, w_{t-3}, w_{t-2}, w_{t-1}, w_t).
    #[inline]
    pub fn lookup_5gram(
        &self,
        w_prev4: u32,
        w_prev3: u32,
        w_prev2: u32,
        w_prev: u32,
        w_curr: u32,
    ) -> Option<&[(u32, i16)]> {
        let key = hash_5gram(w_prev4, w_prev3, w_prev2, w_prev, w_curr);
        self.lookup(key)
    }

    /// Lookup candidates for a conditioned skip-bigram context (w_{t-k}, w_t, condition).
    #[inline]
    pub fn lookup_skip(&self, w_skip: u32, w_curr: u32, condition: u8) -> Option<&[(u32, i16)]> {
        let key = hash_skip(w_skip, w_curr, condition);
        self.lookup(key)
    }

    /// Number of occupied collocation entries.
    #[inline]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Whether the table is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Build an EngramTable from frequency maps of bigrams and trigrams.
    pub fn from_frequencies(
        bigrams: &HashMap<(u32, u32), HashMap<u32, u32>>,
        trigrams: &HashMap<(u32, u32, u32), HashMap<u32, u32>>,
        max_collocations: usize,
    ) -> Self {
        let empty_skips = HashMap::new();
        Self::from_frequencies_conditioned(bigrams, trigrams, &empty_skips, max_collocations)
    }

    /// Build an EngramTable from frequency maps of bigrams, trigrams, and conditioned skip-bigrams.
    pub fn from_frequencies_conditioned(
        bigrams: &HashMap<(u32, u32), HashMap<u32, u32>>,
        trigrams: &HashMap<(u32, u32, u32), HashMap<u32, u32>>,
        skip_bigrams: &HashMap<(u32, u32, u8), HashMap<u32, u32>>,
        max_collocations: usize,
    ) -> Self {
        Self::from_frequencies_conditioned_min_freq(
            bigrams,
            trigrams,
            skip_bigrams,
            max_collocations,
            2,
        )
    }

    /// Build an EngramTable from frequency maps of bigrams, trigrams, and conditioned skip-bigrams with a configurable minimum frequency threshold.
    pub fn from_frequencies_conditioned_min_freq(
        bigrams: &HashMap<(u32, u32), HashMap<u32, u32>>,
        trigrams: &HashMap<(u32, u32, u32), HashMap<u32, u32>>,
        skip_bigrams: &HashMap<(u32, u32, u8), HashMap<u32, u32>>,
        max_collocations: usize,
        min_freq: u32,
    ) -> Self {
        let empty_5 = HashMap::new();
        let empty_4 = HashMap::new();
        Self::from_frequencies_full(
            &empty_5,
            &empty_4,
            trigrams,
            bigrams,
            skip_bigrams,
            max_collocations,
            min_freq,
        )
    }

    /// Build an EngramTable from frequency maps of 5-grams, 4-grams, trigrams, bigrams, and conditioned skip-bigrams.
    pub fn from_frequencies_full(
        fivegrams: &HashMap<[u32; 5], HashMap<u32, u32>>,
        fourgrams: &HashMap<[u32; 4], HashMap<u32, u32>>,
        trigrams: &HashMap<(u32, u32, u32), HashMap<u32, u32>>,
        bigrams: &HashMap<(u32, u32), HashMap<u32, u32>>,
        skip_bigrams: &HashMap<(u32, u32, u8), HashMap<u32, u32>>,
        max_collocations: usize,
        min_freq: u32,
    ) -> Self {
        let mut table = Self::with_capacity(DEFAULT_ENGRAM_CAPACITY);

        struct CandidateEntry {
            key: u32,
            condition: u8,
            total_freq: u32,
            candidates: [(u32, i16); MAX_ENGRAM_CANDIDATES],
            candidate_count: usize,
        }

        let mut all_entries = Vec::new();

        // 1. Process 5-grams (weight x16)
        for (ctx, next_map) in fivegrams {
            let total_freq: u32 = next_map.values().sum();
            if total_freq < min_freq {
                continue;
            }
            let mut sorted_cands: Vec<(u32, u32)> =
                next_map.iter().map(|(&t, &c)| (t, c)).collect();
            sorted_cands.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
            let mut cands = [(0u32, 0i16); MAX_ENGRAM_CANDIDATES];
            let mut c_count = 0;
            for &(tok, count) in &sorted_cands {
                if c_count >= MAX_ENGRAM_CANDIDATES {
                    break;
                }
                if c_count == 0 || (count as u64 * 100) >= (total_freq as u64 * 15) {
                    let score_q15 = ((count as u64 * 32767) / total_freq as u64).min(32767) as i16;
                    cands[c_count] = (tok, score_q15);
                    c_count += 1;
                }
            }
            let key = hash_5gram(ctx[0], ctx[1], ctx[2], ctx[3], ctx[4]);
            all_entries.push(CandidateEntry {
                key,
                condition: 0,
                total_freq: total_freq.saturating_mul(16),
                candidates: cands,
                candidate_count: c_count,
            });
        }

        // 2. Process 4-grams (weight x8)
        for (ctx, next_map) in fourgrams {
            let total_freq: u32 = next_map.values().sum();
            if total_freq < min_freq {
                continue;
            }
            let mut sorted_cands: Vec<(u32, u32)> =
                next_map.iter().map(|(&t, &c)| (t, c)).collect();
            sorted_cands.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
            let mut cands = [(0u32, 0i16); MAX_ENGRAM_CANDIDATES];
            let mut c_count = 0;
            for &(tok, count) in &sorted_cands {
                if c_count >= MAX_ENGRAM_CANDIDATES {
                    break;
                }
                if c_count == 0 || (count as u64 * 100) >= (total_freq as u64 * 15) {
                    let score_q15 = ((count as u64 * 32767) / total_freq as u64).min(32767) as i16;
                    cands[c_count] = (tok, score_q15);
                    c_count += 1;
                }
            }
            let key = hash_4gram(ctx[0], ctx[1], ctx[2], ctx[3]);
            all_entries.push(CandidateEntry {
                key,
                condition: 0,
                total_freq: total_freq.saturating_mul(8),
                candidates: cands,
                candidate_count: c_count,
            });
        }

        // 3. Process trigrams (with 4x specificity weight)
        for (&(w0, w1, w2), next_map) in trigrams {
            let total_freq: u32 = next_map.values().sum();
            if total_freq < min_freq {
                continue;
            }
            let mut sorted_cands: Vec<(u32, u32)> =
                next_map.iter().map(|(&t, &c)| (t, c)).collect();
            sorted_cands.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
            let mut cands = [(0u32, 0i16); MAX_ENGRAM_CANDIDATES];
            let mut c_count = 0;
            for &(tok, count) in &sorted_cands {
                if c_count >= MAX_ENGRAM_CANDIDATES {
                    break;
                }
                if c_count == 0 || (count as u64 * 100) >= (total_freq as u64 * 15) {
                    let score_q15 = ((count as u64 * 32767) / total_freq as u64).min(32767) as i16;
                    cands[c_count] = (tok, score_q15);
                    c_count += 1;
                }
            }
            let key = hash_trigram(w0, w1, w2);
            all_entries.push(CandidateEntry {
                key,
                condition: 0,
                total_freq: total_freq.saturating_mul(4),
                candidates: cands,
                candidate_count: c_count,
            });
        }

        // 4. Process bigrams
        for (&(w1, w2), next_map) in bigrams {
            let total_freq: u32 = next_map.values().sum();
            if total_freq < min_freq {
                continue;
            }
            let mut sorted_cands: Vec<(u32, u32)> =
                next_map.iter().map(|(&t, &c)| (t, c)).collect();
            sorted_cands.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
            let mut cands = [(0u32, 0i16); MAX_ENGRAM_CANDIDATES];
            let mut c_count = 0;
            for &(tok, count) in &sorted_cands {
                if c_count >= MAX_ENGRAM_CANDIDATES {
                    break;
                }
                if c_count == 0 || (count as u64 * 100) >= (total_freq as u64 * 15) {
                    let score_q15 = ((count as u64 * 32767) / total_freq as u64).min(32767) as i16;
                    cands[c_count] = (tok, score_q15);
                    c_count += 1;
                }
            }
            let key = hash_bigram(w1, w2);
            all_entries.push(CandidateEntry {
                key,
                condition: 0,
                total_freq,
                candidates: cands,
                candidate_count: c_count,
            });
        }

        // 5. Process conditioned skip-bigrams (weight x2, and x3 for quotation closure)
        for (&(w_skip, w_curr, cond), next_map) in skip_bigrams {
            let total_freq: u32 = next_map.values().sum();
            if total_freq < min_freq {
                continue;
            }
            let mut sorted_cands: Vec<(u32, u32)> =
                next_map.iter().map(|(&t, &c)| (t, c)).collect();
            sorted_cands.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
            let mut cands = [(0u32, 0i16); MAX_ENGRAM_CANDIDATES];
            let mut c_count = 0;
            for &(tok, count) in &sorted_cands {
                if c_count >= MAX_ENGRAM_CANDIDATES {
                    break;
                }
                if c_count == 0 || (count as u64 * 100) >= (total_freq as u64 * 15) {
                    let score_q15 = ((count as u64 * 32767) / total_freq as u64).min(32767) as i16;
                    cands[c_count] = (tok, score_q15);
                    c_count += 1;
                }
            }
            let key = hash_skip(w_skip, w_curr, cond);
            let weight = if cond & 1 != 0 {
                total_freq.saturating_mul(3)
            } else {
                total_freq.saturating_mul(2)
            };
            all_entries.push(CandidateEntry {
                key,
                condition: cond,
                total_freq: weight,
                candidates: cands,
                candidate_count: c_count,
            });
        }

        all_entries.sort_by(|a, b| {
            b.total_freq
                .cmp(&a.total_freq)
                .then_with(|| a.key.cmp(&b.key))
        });

        let limit = max_collocations.min(MAX_ENGRAM_COLLOCATIONS);
        for entry in all_entries.into_iter().take(limit) {
            if table.candidates.len() + entry.candidate_count > 12800 {
                break;
            }
            table.insert_conditioned(
                entry.key,
                entry.condition,
                &entry.candidates[..entry.candidate_count],
            );
        }

        table
    }

    /// Build EngramTable directly from token sequences and lexical pieces.
    pub fn from_sequences_with_pieces(
        sequences: &[Vec<usize>],
        lexical_pieces: &[Vec<u8>],
        max_collocations: usize,
    ) -> Self {
        let mut bigrams: HashMap<(u32, u32), HashMap<u32, u32>> = HashMap::new();
        let mut trigrams: HashMap<(u32, u32, u32), HashMap<u32, u32>> = HashMap::new();
        let mut skip_bigrams: HashMap<(u32, u32, u8), HashMap<u32, u32>> = HashMap::new();

        for seq in sequences {
            let mut q = 0u8;
            let mut d = 0u8;

            for t in 0..seq.len() {
                let curr_tok = seq[t] as u32;

                let scratch;
                let bytes: &[u8] = if curr_tok < 258 {
                    if (2..=257).contains(&curr_tok) {
                        scratch = [(curr_tok - 2) as u8];
                        &scratch[..]
                    } else {
                        &[]
                    }
                } else {
                    lexical_pieces
                        .get((curr_tok - 258) as usize)
                        .map(|p| &p[..])
                        .unwrap_or(&[])
                };

                for &b in bytes {
                    match b {
                        b'"' => {
                            q ^= 1;
                        }
                        b'(' | b'[' | b'{' => {
                            d = (d + 1).min(3);
                        }
                        b')' | b']' | b'}' => {
                            d = d.saturating_sub(1);
                        }
                        b',' | b';' | b':' => {
                            if d == 0 {
                                d = 1;
                            }
                        }
                        b'.' | b'?' | b'!' => {
                            d = 0;
                        }
                        _ => {}
                    }
                }
                let cond = (q & 1) | ((d & 3) << 1);

                if t + 1 < seq.len() {
                    let next = seq[t + 1] as u32;

                    // Bigram (w_{t-1}, w_t) -> w_{t+1}
                    if t >= 1 {
                        let w_prev = seq[t - 1] as u32;
                        *bigrams
                            .entry((w_prev, curr_tok))
                            .or_default()
                            .entry(next)
                            .or_insert(0) += 1;
                    }

                    // Trigram (w_{t-2}, w_{t-1}, w_t) -> w_{t+1}
                    if t >= 2 {
                        let w_prev2 = seq[t - 2] as u32;
                        let w_prev = seq[t - 1] as u32;
                        *trigrams
                            .entry((w_prev2, w_prev, curr_tok))
                            .or_default()
                            .entry(next)
                            .or_insert(0) += 1;
                    }

                    // Conditioned skip-bigrams for k in {2, 4, 8}
                    for &k in &[2, 4, 8] {
                        if t >= k {
                            let w_skip = seq[t - k] as u32;
                            *skip_bigrams
                                .entry((w_skip, curr_tok, cond))
                                .or_default()
                                .entry(next)
                                .or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        Self::from_frequencies_conditioned_min_freq(
            &bigrams,
            &trigrams,
            &skip_bigrams,
            max_collocations,
            1,
        )
    }

    /// Build EngramTable directly from token sequences.
    pub fn from_sequences(sequences: &[Vec<usize>], max_collocations: usize) -> Self {
        Self::from_sequences_with_pieces(sequences, &[], max_collocations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engram_entry_size_and_memory_footprint() {
        assert_eq!(
            std::mem::size_of::<EngramEntry>(),
            12,
            "EngramEntry must be exactly 12 bytes for 32,768-entry cache footprint"
        );
        let table = EngramTable::with_capacity(32768);
        assert_eq!(table.entries.len(), 32768);
        let fp = table.memory_footprint();
        assert!(
            fp <= 500 * 1024,
            "EngramTable footprint {} exceeds 500 KB ceiling",
            fp
        );
        assert!(
            fp >= 380 * 1024,
            "EngramTable footprint {} is below ~393 KB expected size",
            fp
        );
    }

    #[test]
    fn test_engram_hashing() {
        let h_bi1 = hash_bigram(10, 20);
        let h_bi2 = hash_bigram(10, 21);
        assert_ne!(h_bi1, 0);
        assert_ne!(h_bi1, h_bi2);

        let h_tri1 = hash_trigram(5, 10, 20);
        let h_tri2 = hash_trigram(6, 10, 20);
        assert_ne!(h_tri1, 0);
        assert_ne!(h_tri1, h_tri2);

        let h_skip1 = hash_skip(5, 20, 0);
        let h_skip2 = hash_skip(5, 20, 1);
        assert_ne!(h_skip1, 0);
        assert_ne!(h_skip1, h_skip2);

        // Domain separation between bigram, trigram, and skip-bigram
        assert_ne!(h_bi1, h_tri1);
        assert_ne!(h_bi1, h_skip1);
        assert_ne!(h_tri1, h_skip1);
    }

    #[test]
    fn test_engram_table_insert_and_lookup() {
        let mut table = EngramTable::with_capacity(128);
        assert!(table.is_empty());
        assert_eq!(table.len(), 0);

        let cands = [(100u32, 16384i16), (101u32, 8192i16)];
        assert!(table.insert(hash_bigram(10, 20), &cands));
        assert_eq!(table.len(), 1);

        let res = table.lookup_bigram(10, 20);
        assert!(res.is_some());
        let found = res.unwrap();
        assert_eq!(found.len(), 2);
        assert_eq!(found[0], (100, 16384));
        assert_eq!(found[1], (101, 8192));

        // Skip-bigram conditioned lookup
        let skip_cands = [(200u32, 24576i16)];
        assert!(table.insert_conditioned(hash_skip(5, 20, 1), 1, &skip_cands));
        let skip_res = table.lookup_skip(5, 20, 1);
        assert!(skip_res.is_some());
        assert_eq!(skip_res.unwrap()[0], (200, 24576));

        // Miss with wrong condition
        assert!(table.lookup_skip(5, 20, 0).is_none());

        // Miss
        assert!(table.lookup_bigram(10, 21).is_none());
        assert!(table.lookup_trigram(5, 10, 20).is_none());
    }

    #[test]
    fn test_engram_from_sequences() {
        let seq1 = vec![1, 2, 3, 4];
        let seq2 = vec![1, 2, 3, 5];
        let seq3 = vec![1, 2, 3, 4];

        let table = EngramTable::from_sequences(&[seq1, seq2, seq3], 100);
        assert!(!table.is_empty());

        // Context (2, 3) followed by 4 (twice) and 5 (once)
        let res = table.lookup_bigram(2, 3);
        assert!(res.is_some());
        let cands = res.unwrap();
        assert_eq!(cands[0].0, 4); // Most frequent first
    }

    #[test]
    fn test_engram_deep_linear_probe_retrieval() {
        // Table with capacity 64 (mask = 63).
        // Insert keys that map to the exact same initial slot, testing probe depth >= 5.
        let mut table = EngramTable::with_capacity(64);
        let base_slot = 10_u32;
        let cands = [(42u32, 1000i16)];

        // Create 8 keys that all have (key as usize) & 63 == base_slot
        let mut keys = Vec::new();
        for i in 0..8 {
            let k = base_slot + (i as u32) * 64;
            keys.push(k);
            let inserted = table.insert(k, &cands);
            assert!(inserted, "Key {} should be inserted", k);
        }

        // All 8 keys must be successfully retrieved, including those at probe depth >= 4
        for (i, &k) in keys.iter().enumerate() {
            let res = table.lookup(k);
            assert!(
                res.is_some(),
                "Key {} at probe depth {} must be found by lookup",
                k,
                i
            );
            assert_eq!(res.unwrap()[0].0, 42);
        }
    }

    #[test]
    fn test_engram_deterministic_tie_breaking() {
        let mut bigrams = HashMap::new();
        let trigrams = HashMap::new();

        // Add 10 bigrams with identical total_freq = 5
        for i in 1..=10 {
            let mut next = HashMap::new();
            next.insert(100 + i, 5);
            bigrams.insert((i, i + 1), next);
        }

        let table1 = EngramTable::from_frequencies(&bigrams, &trigrams, 5);
        let table2 = EngramTable::from_frequencies(&bigrams, &trigrams, 5);

        assert_eq!(
            table1, table2,
            "EngramTable compilation must be bit-exact deterministic"
        );
    }
}
