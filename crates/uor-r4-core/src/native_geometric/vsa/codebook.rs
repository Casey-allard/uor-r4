//! Deterministic Token and Atom Codebook for High-Dimensional VSA.
//!
//! Maps discrete token IDs and semantic atoms into mutually quasi-orthogonal
//! basis hypervectors with zero runtime matrix multiplications and zero runtime floats.

use super::hypervector::Hypervector;
use serde::{Deserialize, Serialize};

/// Deterministic codebook mapping tokens to quasi-orthogonal basis hypervectors.
///
/// Precomputes basis vectors for vocabulary tokens up to `vocab_size` for $O(1)$
/// array indexing on serving paths. Tokens beyond `vocab_size` are generated
/// on-the-fly deterministically from `(seed, token_id)` with zero heap allocations.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Codebook<const WORDS: usize> {
    pub seed: u64,
    pub vocab_size: usize,
    pub table: Vec<Hypervector<WORDS>>,
}

impl<const WORDS: usize> Codebook<WORDS> {
    /// Initialize a deterministic codebook with precomputed basis vectors for `0..vocab_size`.
    pub fn new(vocab_size: usize, seed: u64) -> Self {
        let mut table = Vec::with_capacity(vocab_size);
        for token_id in 0..vocab_size {
            table.push(Self::generate_basis_vector(seed, token_id as u64));
        }
        Self {
            seed,
            vocab_size,
            table,
        }
    }

    /// Pure on-demand codebook without precomputed table (zero heap allocations at construction).
    pub fn on_demand(vocab_size: usize, seed: u64) -> Self {
        Self {
            seed,
            vocab_size,
            table: Vec::new(),
        }
    }

    /// Retrieve the basis hypervector for a given token ID.
    ///
    /// If `token_id < table.len()`, returns by direct array lookup ($O(1)$).
    /// Otherwise, generates deterministically on-the-fly with zero heap allocations.
    #[inline]
    pub fn get(&self, token_id: u32) -> Hypervector<WORDS> {
        let idx = token_id as usize;
        if idx < self.table.len() {
            self.table[idx]
        } else {
            Self::generate_basis_vector(self.seed, token_id as u64)
        }
    }

    /// Retrieve a reference to the basis hypervector if already in the precomputed table.
    #[inline]
    pub fn get_ref(&self, token_id: u32) -> Option<&Hypervector<WORDS>> {
        self.table.get(token_id as usize)
    }

    /// Deterministically map a string atom (e.g. role, entity, special token) into a hypervector.
    pub fn get_atom(&self, atom: &str) -> Hypervector<WORDS> {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.seed.to_le_bytes());
        hasher.update(atom.as_bytes());
        let hash = hasher.finalize();
        let bytes = hash.as_bytes();

        let salt = u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        let secondary = u64::from_le_bytes([
            bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
        ]);

        Hypervector::from_seed(self.seed ^ salt, secondary)
    }

    /// Find the nearest token in the vocabulary to a query hypervector.
    ///
    /// Returns `(token_id, hamming_distance)`.
    pub fn nearest_token(&self, query: &Hypervector<WORDS>) -> (u32, usize) {
        let mut best_tok = 0u32;
        let mut min_dist = usize::MAX;

        let n = if !self.table.is_empty() {
            self.table.len()
        } else {
            self.vocab_size
        };

        for tok in 0..n {
            let vec = self.get(tok as u32);
            let dist = query.hamming_distance(&vec);
            if dist < min_dist {
                min_dist = dist;
                best_tok = tok as u32;
            }
        }

        (best_tok, min_dist)
    }

    /// Rank top-K tokens in vocabulary by fixed-point $Q1.15$ similarity.
    ///
    /// Serving hot-path friendly: Zero runtime floats.
    pub fn top_k_tokens_q15(&self, query: &Hypervector<WORDS>, k: usize) -> Vec<(u32, i16)> {
        let n = if !self.table.is_empty() {
            self.table.len()
        } else {
            self.vocab_size
        };

        let mut ranked = Vec::with_capacity(n);
        for tok in 0..n {
            let vec = self.get(tok as u32);
            let score = query.similarity_q15(&vec);
            ranked.push((tok as u32, score));
        }

        ranked.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        if ranked.len() > k {
            ranked.truncate(k);
        }
        ranked
    }

    /// Zero-allocation top-K candidate search on stack for serving hot paths.
    ///
    /// Maintains the top-K highest scoring token IDs without dynamic heap allocations.
    /// Returns an array of `(token_id, score_q15)` sorted in descending order of score.
    pub fn top_k_candidates_q15<const K: usize>(
        &self,
        query: &Hypervector<WORDS>,
    ) -> [(u32, i16); K] {
        let mut top = [(0u32, i16::MIN); K];
        if K == 0 {
            return top;
        }

        let n = if !self.table.is_empty() {
            self.table.len()
        } else {
            self.vocab_size
        };

        for tok in 0..n {
            let vec = self.get(tok as u32);
            let score = query.similarity_q15(&vec);
            let tok_id = tok as u32;

            // If score is greater than the smallest in top-K:
            if score > top[K - 1].1 || (score == top[K - 1].1 && tok_id < top[K - 1].0) {
                // Find insertion position
                let mut pos = K - 1;
                while pos > 0
                    && (score > top[pos - 1].1
                        || (score == top[pos - 1].1 && tok_id < top[pos - 1].0))
                {
                    top[pos] = top[pos - 1];
                    pos -= 1;
                }
                top[pos] = (tok_id, score);
            }
        }

        top
    }

    /// Internal deterministic pseudo-random generator for basis hypervectors.
    #[inline]
    fn generate_basis_vector(seed: u64, token_id: u64) -> Hypervector<WORDS> {
        // High-entropy mixing constant derived from the golden ratio phi and sqrt(2)
        let mixed_salt = token_id
            .wrapping_mul(0x517cc1b727220a95)
            .wrapping_add(0x9e3779b97f4a7c15);
        Hypervector::from_seed(seed, mixed_salt)
    }
}
