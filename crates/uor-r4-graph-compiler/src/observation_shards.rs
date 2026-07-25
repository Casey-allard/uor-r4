//! Parallel Observation, Trace, and Evaluation Processing Over Deterministic Shards (#170).
//!
//! Provides content-addressed shard partitioning, coarse-grained parallel execution over
//! Rayon, and ordered deterministic reductions guaranteeing 100% bit-identical byte digests.

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

/// Deterministic 64-bit FNV-1a hash over byte slice.
pub fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3u64);
    }
    hash
}

/// A content-addressed observation / trace / evaluation shard.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservationShard {
    /// Deterministic 64-bit content-addressed shard ID.
    pub shard_id: u64,
    /// 32-byte content hash representation.
    pub content_hash: [u8; 32],
    /// Observation text/payload items in this shard.
    pub items: Vec<String>,
}

impl ObservationShard {
    /// Create a new content-addressed shard from observation items.
    pub fn new(items: Vec<String>) -> Self {
        let mut combined = Vec::new();
        for item in &items {
            let bytes = item.as_bytes();
            combined.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
            combined.extend_from_slice(bytes);
        }
        let shard_id = fnv1a_64(&combined);
        let mut content_hash = [0u8; 32];
        content_hash[..8].copy_from_slice(&shard_id.to_le_bytes());

        ObservationShard {
            shard_id,
            content_hash,
            items,
        }
    }
}

/// Configuration for observation shard partitioning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShardProcessingConfig {
    /// Maximum items per coarse shard.
    pub chunk_size: usize,
}

impl Default for ShardProcessingConfig {
    fn default() -> Self {
        ShardProcessingConfig { chunk_size: 64 }
    }
}

/// Output item wrapper carrying the originating shard ID for ordered reduction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShardReductionResult<T> {
    pub shard_id: u64,
    pub data: T,
}

/// Parallel observation shard processing engine.
pub struct ParallelShardEngine;

impl ParallelShardEngine {
    /// Partition input observation strings into content-addressed coarse shards.
    pub fn partition_items(
        items: &[String],
        config: &ShardProcessingConfig,
    ) -> Vec<ObservationShard> {
        let chunk_size = config.chunk_size.max(1);
        items
            .chunks(chunk_size)
            .map(|chunk| ObservationShard::new(chunk.to_vec()))
            .collect()
    }

    /// Process shards in parallel using Rayon and return un-ordered partial results.
    pub fn process_shards_parallel<F, R>(
        shards: &[ObservationShard],
        processor: F,
    ) -> Vec<ShardReductionResult<R>>
    where
        F: Fn(&ObservationShard) -> R + Sync + Send,
        R: Send,
    {
        shards
            .par_iter()
            .map(|shard| ShardReductionResult {
                shard_id: shard.shard_id,
                data: processor(shard),
            })
            .collect()
    }

    /// Perform deterministic ordered reduction combining shard results in ascending shard-ID order.
    pub fn ordered_shard_reduce<R>(mut results: Vec<ShardReductionResult<R>>) -> Vec<R> {
        results.sort_by_key(|res| res.shard_id);
        results.into_iter().map(|res| res.data).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shard_content_addressing_determinism() {
        let items = vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()];
        let shard1 = ObservationShard::new(items.clone());
        let shard2 = ObservationShard::new(items);
        assert_eq!(shard1.shard_id, shard2.shard_id);
        assert_eq!(shard1.content_hash, shard2.content_hash);
    }

    #[test]
    fn test_parallel_shard_processing_and_ordered_reduction() {
        let items: Vec<String> = (0..100).map(|i| format!("doc_{i}")).collect();
        let config = ShardProcessingConfig { chunk_size: 10 };
        let shards = ParallelShardEngine::partition_items(&items, &config);
        assert_eq!(shards.len(), 10);

        let par_results =
            ParallelShardEngine::process_shards_parallel(&shards, |shard| shard.items.len());

        let reduced = ParallelShardEngine::ordered_shard_reduce(par_results);
        assert_eq!(reduced.len(), 10);
        assert!(reduced.iter().all(|&len| len == 10));
    }

    #[test]
    fn test_shard_content_addressing_distinguishes_item_boundaries() {
        let shard1 = ObservationShard::new(vec!["ab".to_string(), "c".to_string()]);
        let shard2 = ObservationShard::new(vec!["a".to_string(), "bc".to_string()]);
        assert_ne!(shard1.shard_id, shard2.shard_id);
        assert_ne!(shard1.content_hash, shard2.content_hash);
    }
}
