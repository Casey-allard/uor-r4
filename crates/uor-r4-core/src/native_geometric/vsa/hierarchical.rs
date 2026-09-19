//! Hierarchical Lattice Codebook and Bounded Shortlist Routing.
//!
//! Replaces unconstrained full-vocabulary scanning on serving hot paths with
//! a two-tier geometric index:
//! 1. Coarse routing: 120 canonical H4 roots of the 600-cell on S3, or
//!    hyperdimensional VSA sector anchors.
//! 2. Fine routing: Leaf buckets (<= 32 tokens each) partitioned within active sectors.
//! 3. Bounded candidate output: Zero-allocation stack buffer `Shortlist<MAX>`.
//!
//! Invariants: Zero Host-LLM fallbacks, Zero runtime matrix multiplications,
//! Zero runtime floats, Zero steady-state heap allocations.

#![forbid(unsafe_code)]

use super::codebook::Codebook;
use super::hypervector::Hypervector;
use crate::native_geometric::hopf_metric::{UnitS3, UnitS3Q30};
use crate::native_geometric::learner::embedding::{canonical_h4_roots_q30, H4_ROOT_COUNT};
use serde::{Deserialize, Serialize};

/// Maximum number of discrete tokens contained within a single leaf bucket.
pub const LEAF_BUCKET_CAPACITY: usize = 32;

/// Maximum candidate extraction cap from any single H4 sector in multi-sector routing.
pub const MAX_PER_SECTOR: usize = 8;

/// Fine-grained leaf bucket partitioning tokens within a coarse H4 root sector.
///
/// Contains at most `LEAF_BUCKET_CAPACITY` (32) tokens and a bundled centroid hypervector.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeafBucket<const WORDS: usize> {
    pub centroid: Hypervector<WORDS>,
    pub tokens: Vec<u32>,
}

impl<const WORDS: usize> LeafBucket<WORDS> {
    pub const MAX_TOKENS: usize = LEAF_BUCKET_CAPACITY;

    /// Construct a new leaf bucket with centroid and token list.
    pub fn new(centroid: Hypervector<WORDS>, tokens: Vec<u32>) -> Self {
        debug_assert!(
            tokens.len() <= Self::MAX_TOKENS,
            "LeafBucket token count {} exceeds maximum {}",
            tokens.len(),
            Self::MAX_TOKENS
        );
        Self { centroid, tokens }
    }

    /// Number of tokens in this bucket.
    #[inline]
    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    /// Whether this bucket contains no tokens.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
}

/// Stack-allocated candidate token shortlist for zero-allocation candidate retrieval.
///
/// Holds up to `MAX` candidate token IDs without dynamic heap allocations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shortlist<const MAX: usize> {
    pub items: [u32; MAX],
    pub len: usize,
}

impl<const MAX: usize> Shortlist<MAX> {
    /// Construct an empty shortlist with zero elements.
    #[inline]
    pub const fn empty() -> Self {
        Self {
            items: [0u32; MAX],
            len: 0,
        }
    }

    /// View active candidates as a contiguous slice.
    #[inline]
    pub fn as_slice(&self) -> &[u32] {
        &self.items[..self.len]
    }

    /// Current number of candidate tokens.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether the shortlist is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Maximum capacity of this shortlist.
    #[inline]
    pub const fn capacity(&self) -> usize {
        MAX
    }

    /// Whether the shortlist has reached capacity.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.len >= MAX
    }

    /// Push a candidate token if capacity remains and token is not already present.
    ///
    /// Returns `true` if the token was newly added.
    #[inline]
    pub fn push(&mut self, token: u32) -> bool {
        if self.len < MAX {
            let mut i = 0;
            while i < self.len {
                if self.items[i] == token {
                    return false;
                }
                i += 1;
            }
            self.items[self.len] = token;
            self.len += 1;
            true
        } else {
            false
        }
    }

    /// Push a candidate token without checking for duplicates.
    #[inline]
    pub fn push_unchecked(&mut self, token: u32) -> bool {
        if self.len < MAX {
            self.items[self.len] = token;
            self.len += 1;
            true
        } else {
            false
        }
    }

    /// Clear all candidate tokens.
    #[inline]
    pub fn clear(&mut self) {
        self.len = 0;
    }
}

impl<const MAX: usize> Default for Shortlist<MAX> {
    #[inline]
    fn default() -> Self {
        Self::empty()
    }
}

impl<const MAX: usize> core::ops::Deref for Shortlist<MAX> {
    type Target = [u32];

    #[inline]
    fn deref(&self) -> &[u32] {
        self.as_slice()
    }
}

impl<'a, const MAX: usize> IntoIterator for &'a Shortlist<MAX> {
    type Item = &'a u32;
    type IntoIter = core::slice::Iter<'a, u32>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

/// Serde helper for fixed-size 120-element array of leaf bucket vectors.
mod serde_sectors_120 {
    use super::*;
    use serde::{de::SeqAccess, de::Visitor, Deserializer, Serializer};

    pub fn serialize<S, const W: usize>(
        arr: &[Vec<LeafBucket<W>>; 120],
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(120))?;
        for item in arr {
            seq.serialize_element(item)?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D, const W: usize>(
        deserializer: D,
    ) -> Result<[Vec<LeafBucket<W>>; 120], D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SectorsVisitor<const W: usize>;
        impl<'de, const W: usize> Visitor<'de> for SectorsVisitor<W> {
            type Value = [Vec<LeafBucket<W>>; 120];

            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                formatter.write_str("an array of 120 sector bucket vectors")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut arr: [Vec<LeafBucket<W>>; 120] = core::array::from_fn(|_| Vec::new());
                for (i, slot) in arr.iter_mut().enumerate() {
                    *slot = seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(i, &self))?;
                }
                Ok(arr)
            }
        }
        deserializer.deserialize_seq(SectorsVisitor::<W>)
    }
}

/// Serde helper for fixed-size 120-element array of anchor hypervectors.
mod serde_arr_120 {
    use super::*;
    use serde::{de::SeqAccess, de::Visitor, Deserializer, Serializer};

    pub fn serialize<S, const W: usize>(
        arr: &Box<[Hypervector<W>; 120]>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(120))?;
        for item in arr.as_ref() {
            seq.serialize_element(item)?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D, const W: usize>(
        deserializer: D,
    ) -> Result<Box<[Hypervector<W>; 120]>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ArrVisitor<const W: usize>;
        impl<'de, const W: usize> Visitor<'de> for ArrVisitor<W> {
            type Value = Box<[Hypervector<W>; 120]>;

            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                formatter.write_str("an array of 120 hypervectors")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut arr = vec![Hypervector::<W>::zero(); 120];
                for (i, slot) in arr.iter_mut().enumerate() {
                    *slot = seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(i, &self))?;
                }
                let boxed_slice = arr.into_boxed_slice();
                let boxed_arr: Box<[Hypervector<W>; 120]> = match boxed_slice.try_into() {
                    Ok(b) => b,
                    Err(_) => return Err(serde::de::Error::custom("expected 120 hypervectors")),
                };
                Ok(boxed_arr)
            }
        }
        deserializer.deserialize_seq(ArrVisitor::<W>)
    }
}

/// Hierarchical Lattice Codebook with 120 coarse H4 root sectors and fine leaf buckets.
///
/// Enables bounded-shortlist candidate retrieval with zero runtime matrix multiplications,
/// zero runtime floats, and zero steady-state heap allocations.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HierarchicalCodebook<const WORDS: usize> {
    pub vocab_size: usize,
    #[serde(with = "serde_sectors_120")]
    pub sectors: [Vec<LeafBucket<WORDS>>; 120],
    #[serde(with = "serde_arr_120")]
    pub root_anchors: Box<[Hypervector<WORDS>; 120]>,
}

impl<const WORDS: usize> HierarchicalCodebook<WORDS> {
    /// Construct a new hierarchical codebook by partitioning vocabulary tokens into coarse H4 root
    /// sectors and chunking each sector into leaf buckets of size <= 32.
    pub fn new(vocab_size: usize, token_to_root: &[u8], base_codebook: &Codebook<WORDS>) -> Self {
        let mut root_tokens: [Vec<u32>; H4_ROOT_COUNT] = core::array::from_fn(|_| Vec::new());
        for token_id in 0..vocab_size {
            let root = if token_id < token_to_root.len() {
                token_to_root[token_id] as usize % H4_ROOT_COUNT
            } else {
                0
            };
            root_tokens[root].push(token_id as u32);
        }

        let mut root_anchors_vec = vec![Hypervector::<WORDS>::zero(); H4_ROOT_COUNT];
        let mut sectors: [Vec<LeafBucket<WORDS>>; H4_ROOT_COUNT] =
            core::array::from_fn(|_| Vec::new());

        for r in 0..H4_ROOT_COUNT {
            if root_tokens[r].is_empty() {
                root_anchors_vec[r] = base_codebook.get_atom(&format!("h4_root_{}", r));
            } else {
                let token_vecs: Vec<Hypervector<WORDS>> = root_tokens[r]
                    .iter()
                    .map(|&t| base_codebook.get(t))
                    .collect();
                root_anchors_vec[r] = Hypervector::bundle(&token_vecs);

                for chunk in root_tokens[r].chunks(LEAF_BUCKET_CAPACITY) {
                    let chunk_vecs: Vec<Hypervector<WORDS>> =
                        chunk.iter().map(|&t| base_codebook.get(t)).collect();
                    let centroid = Hypervector::bundle(&chunk_vecs);
                    sectors[r].push(LeafBucket {
                        centroid,
                        tokens: chunk.to_vec(),
                    });
                }
            }
        }

        let root_anchors: Box<[Hypervector<WORDS>; H4_ROOT_COUNT]> =
            match root_anchors_vec.into_boxed_slice().try_into() {
                Ok(b) => b,
                Err(_) => unreachable!("exact length H4_ROOT_COUNT"),
            };

        Self {
            vocab_size,
            sectors,
            root_anchors,
        }
    }

    /// Route continuous S3 context and/or discrete VSA context into a bounded token shortlist,
    /// integrating active memory candidates and coarse/fine sector partitioning.
    ///
    /// Distributes candidate slots across the top 6 active H4 sectors (capped at `MAX_PER_SECTOR = 8`,
    /// totaling up to 48 slots) plus up to 16 exact memory / engram priority slots.
    pub fn route_shortlist_q30_with_memory<const MAX: usize>(
        &self,
        query_s3: Option<UnitS3Q30>,
        query_vsa: Option<&Hypervector<WORDS>>,
        memory_tokens: &[u32],
    ) -> Shortlist<MAX> {
        let mut shortlist = Shortlist::<MAX>::empty();
        if self.vocab_size == 0 || MAX == 0 {
            return shortlist;
        }

        // 1. Inject active memory / engram priority candidates first (up to 16 slots)
        let max_memory = 16.min(MAX);
        for &token in memory_tokens {
            if shortlist.len >= max_memory {
                break;
            }
            if (token as usize) < self.vocab_size {
                shortlist.push(token);
            }
        }
        if shortlist.len >= MAX {
            return shortlist;
        }

        // 2. Coarse routing: find top 6 active H4 sectors
        let mut active_sectors: [Option<usize>; 6] = [None; 6];

        if let Some(s3) = query_s3 {
            let canonical = canonical_h4_roots_q30();
            let mut top = [(0usize, i64::MIN); 6];
            let mut count = 0usize;

            for r in 0..H4_ROOT_COUNT {
                if self.sectors[r].is_empty() {
                    continue;
                }
                let root_q30 = canonical[r];
                let score = (s3.0[0] as i64 * root_q30.0[0] as i64
                    + s3.0[1] as i64 * root_q30.0[1] as i64
                    + s3.0[2] as i64 * root_q30.0[2] as i64
                    + s3.0[3] as i64 * root_q30.0[3] as i64)
                    >> 30;

                if count < 6 {
                    let mut pos = count;
                    while pos > 0 && top[pos - 1].1 < score {
                        top[pos] = top[pos - 1];
                        pos -= 1;
                    }
                    top[pos] = (r, score);
                    count += 1;
                } else if score > top[5].1 {
                    let mut pos = 5;
                    while pos > 0 && top[pos - 1].1 < score {
                        top[pos] = top[pos - 1];
                        pos -= 1;
                    }
                    top[pos] = (r, score);
                }
            }
            for i in 0..count {
                active_sectors[i] = Some(top[i].0);
            }
        } else if let Some(vsa) = query_vsa {
            let mut top = [(0usize, i16::MIN); 6];
            let mut count = 0usize;

            for r in 0..H4_ROOT_COUNT {
                if self.sectors[r].is_empty() {
                    continue;
                }
                let score = vsa.similarity_q15(&self.root_anchors[r]);
                if count < 6 {
                    let mut pos = count;
                    while pos > 0 && top[pos - 1].1 < score {
                        top[pos] = top[pos - 1];
                        pos -= 1;
                    }
                    top[pos] = (r, score);
                    count += 1;
                } else if score > top[5].1 {
                    let mut pos = 5;
                    while pos > 0 && top[pos - 1].1 < score {
                        top[pos] = top[pos - 1];
                        pos -= 1;
                    }
                    top[pos] = (r, score);
                }
            }
            for i in 0..count {
                active_sectors[i] = Some(top[i].0);
            }
        } else {
            // Default: pick first 6 non-empty sectors
            let mut count = 0usize;
            for r in 0..H4_ROOT_COUNT {
                if !self.sectors[r].is_empty() {
                    active_sectors[count] = Some(r);
                    count += 1;
                    if count >= 6 {
                        break;
                    }
                }
            }
        }

        // 3. Fine routing: extract up to MAX_PER_SECTOR (8) candidates per sector across active sectors
        for &maybe_sec in &active_sectors {
            let Some(sec) = maybe_sec else { continue };
            let buckets = &self.sectors[sec];
            if buckets.is_empty() {
                continue;
            }

            let mut sector_count = 0usize;

            if buckets.len() == 1 {
                for &token in &buckets[0].tokens {
                    if shortlist.len >= MAX || sector_count >= MAX_PER_SECTOR {
                        break;
                    }
                    if shortlist.push(token) {
                        sector_count += 1;
                    }
                }
            } else if let Some(vsa) = query_vsa {
                // Find closest bucket by similarity_q15
                let mut best_bucket = 0;
                let mut best_score = i16::MIN;
                for (b_idx, bucket) in buckets.iter().enumerate() {
                    let sim = vsa.similarity_q15(&bucket.centroid);
                    if sim > best_score {
                        best_score = sim;
                        best_bucket = b_idx;
                    }
                }

                // Push tokens from best bucket
                for &token in &buckets[best_bucket].tokens {
                    if shortlist.len >= MAX || sector_count >= MAX_PER_SECTOR {
                        break;
                    }
                    if shortlist.push(token) {
                        sector_count += 1;
                    }
                }

                // If space remains within MAX_PER_SECTOR, check second closest bucket
                if shortlist.len < MAX && sector_count < MAX_PER_SECTOR && buckets.len() > 1 {
                    let mut second_best = None;
                    let mut second_score = i16::MIN;
                    for (b_idx, bucket) in buckets.iter().enumerate() {
                        if b_idx == best_bucket {
                            continue;
                        }
                        let sim = vsa.similarity_q15(&bucket.centroid);
                        if sim > second_score {
                            second_score = sim;
                            second_best = Some(b_idx);
                        }
                    }
                    if let Some(b_idx) = second_best {
                        for &token in &buckets[b_idx].tokens {
                            if shortlist.len >= MAX || sector_count >= MAX_PER_SECTOR {
                                break;
                            }
                            if shortlist.push(token) {
                                sector_count += 1;
                            }
                        }
                    }
                }
            } else {
                // No query_vsa: take buckets in order
                for bucket in buckets {
                    for &token in &bucket.tokens {
                        if shortlist.len >= MAX || sector_count >= MAX_PER_SECTOR {
                            break;
                        }
                        if shortlist.push(token) {
                            sector_count += 1;
                        }
                    }
                    if shortlist.len >= MAX || sector_count >= MAX_PER_SECTOR {
                        break;
                    }
                }
            }

            if shortlist.len >= MAX {
                break;
            }
        }

        // 4. Overflow fill: if shortlist still has capacity, fill remaining slots from active sectors
        if shortlist.len < MAX {
            for &maybe_sec in &active_sectors {
                let Some(sec) = maybe_sec else { continue };
                for bucket in &self.sectors[sec] {
                    for &token in &bucket.tokens {
                        if shortlist.len >= MAX {
                            break;
                        }
                        shortlist.push(token);
                    }
                    if shortlist.len >= MAX {
                        break;
                    }
                }
                if shortlist.len >= MAX {
                    break;
                }
            }
        }

        // 5. Fallback: if shortlist is still empty, populate from non-empty sectors
        if shortlist.len == 0 && self.vocab_size > 0 {
            for sec in 0..H4_ROOT_COUNT {
                for bucket in &self.sectors[sec] {
                    for &token in &bucket.tokens {
                        if shortlist.push(token) && shortlist.len >= MAX {
                            return shortlist;
                        }
                    }
                }
            }
        }

        shortlist
    }

    /// Route candidate shortlist with active memory tokens and continuous S3 query.
    #[inline]
    pub fn route_shortlist_with_memory<const MAX: usize>(
        &self,
        query_s3: Option<UnitS3>,
        query_vsa: Option<&Hypervector<WORDS>>,
        memory_tokens: &[u32],
    ) -> Shortlist<MAX> {
        self.route_shortlist_q30_with_memory(query_s3.map(|s| s.to_q30()), query_vsa, memory_tokens)
    }

    /// Route continuous S3 context and/or discrete VSA context into a bounded token shortlist.
    ///
    /// - Coarse routing: finds top 1 or 2 H4 sectors by S3 dot-product with canonical roots or
    ///   VSA similarity with sector anchors.
    /// - Fine routing: selects closest leaf bucket(s) in active sector(s) using Hamming distance /
    ///   Q1.15 similarity between query_vsa and bucket.centroid.
    /// - Copies candidate tokens into `Shortlist<MAX>` with zero dynamic heap allocations.
    #[inline]
    pub fn route_shortlist<const MAX: usize>(
        &self,
        query_s3: Option<UnitS3>,
        query_vsa: Option<&Hypervector<WORDS>>,
    ) -> Shortlist<MAX> {
        self.route_shortlist_with_memory(query_s3, query_vsa, &[])
    }

    /// Total number of leaf buckets across all 120 sectors.
    pub fn leaf_bucket_count(&self) -> usize {
        self.sectors.iter().map(|s| s.len()).sum()
    }

    /// Total number of tokens stored across all leaf buckets.
    pub fn total_token_count(&self) -> usize {
        self.sectors
            .iter()
            .flat_map(|s| s.iter())
            .map(|b| b.tokens.len())
            .sum()
    }

    /// Build token-to-cluster mapping from hierarchical leaf buckets.
    ///
    /// Maps each discrete token to its unique leaf bucket index across all 120 sectors.
    /// Clamps maximum cluster count to `MAX_FINE_CLUSTERS` (480).
    /// Returns `(token_to_cluster, num_clusters)`.
    pub fn build_token_to_cluster(&self) -> (Vec<u16>, usize) {
        if self.vocab_size == 0 {
            return (Vec::new(), 0);
        }
        let mut token_to_cluster = vec![0u16; self.vocab_size];
        let mut cluster_idx = 0usize;
        for sector in &self.sectors {
            for bucket in sector {
                let c = (cluster_idx % 480) as u16;
                for &token in &bucket.tokens {
                    if (token as usize) < self.vocab_size {
                        token_to_cluster[token as usize] = c;
                    }
                }
                cluster_idx += 1;
            }
        }
        let num_clusters = cluster_idx.min(480).max(1);
        (token_to_cluster, num_clusters)
    }
}

impl<const WORDS: usize> Default for HierarchicalCodebook<WORDS> {
    fn default() -> Self {
        let root_anchors = match vec![Hypervector::<WORDS>::zero(); 120]
            .into_boxed_slice()
            .try_into()
        {
            Ok(b) => b,
            Err(_) => unreachable!("exact length 120"),
        };
        Self {
            vocab_size: 0,
            sectors: core::array::from_fn(|_| Vec::new()),
            root_anchors,
        }
    }
}
