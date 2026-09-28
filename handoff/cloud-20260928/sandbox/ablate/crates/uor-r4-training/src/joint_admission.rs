//! Per-sequence causal admission index for D8 rung 3 bounded access.
//!
//! This module provides the *actual* bounded candidate-access rule (not a
//! full-context scan followed by top-k) for one batch lane of the joint
//! recurrent learner. It is plain Rust: no Candle, no gradients, no learned
//! parameters, no floating-point arithmetic beyond the declared comparisons.
//! `#![forbid(unsafe_code)]` is crate-wide in `lib.rs`; this module adds none.
//!
//! Semantics and boundaries:
//!
//! - **Causal.** A query returns prior occurrence identities only, strictly
//!   lower than the current index length. The caller queries before inserting
//!   the event it is reading (the write happens after the read), so an event is
//!   never admitted into its own read.
//! - **Exact identity.** Occurrence ids are the exact monotone insert order.
//!   The index does not store keys at all: the caller's occurrence tape keeps
//!   the exact 64-D key identity that maps an admitted id back to a real event.
//!   Bucket equality is a non-learned address property, never a semantic/hash
//!   distance and never an H4/geometry claim.
//! - **Bounded work, counted.** Every query reports the work it actually did
//!   (`AdmissionWork`) and the structure reports its logical slots/bytes
//!   (`AdmissionFootprint`). `Full` is the only O(len) path and it is
//!   diagnostic-only; `history_visits` exists so it cannot be mistaken for a
//!   bounded serving path.
//! - **Eviction is local.** An occurrence missing from an `Orthant64` result is
//!   a posting-window outcome, not evidence of global absence: the ring slot
//!   overwrite is counted in `posting_evictions`, and `Full` still lists the
//!   occurrence. `ExactCache64` never evicts; only its traversal is bounded.
//! - **No-read is the caller's decision.** When the caller is in `NoRead` it
//!   must not call [`AdmissionIndex::query`] at all, which keeps the recorded
//!   work honestly zero. Querying and discarding the result would be visible in
//!   `posting_visits`/`candidate_pool_entries`.
//! - **Gradient boundary.** Admission is a hard, deterministic, set-valued
//!   function of the inputs: it removes candidates and renormalizes the caller's
//!   read distribution over the admitted set. Gradients still flow through the
//!   *selected* candidates' own Q/K/V energies, ages and the recurrent path,
//!   but there is no gradient with respect to *which* candidates were admitted.
//!   Nothing in this module is a learned gate.
//!
//! The two fixed sign tables read only query coordinates `0..4` and `4..8` of
//! the 64-D read space. Those coordinates of the dense `[64, 256]` read
//! projection carry no declared lane/R4 structure; calling them geometric
//! lanes would be an unmeasured naming claim, so they are called coordinates.

use serde::{Deserialize, Serialize};

use crate::{invalid, Result};

/// Exact token vocabulary of the joint model (`JointConfig::vocab_size`).
pub const VOCAB_SIZE: usize = 4096;
/// Width of the read key/query space (`JointConfig::read_width`).
pub const KEY_WIDTH: usize = 64;
/// Recency window served by the `Recent64` policy.
pub const RECENT_CAPACITY: usize = 64;
/// Recency window admitted by `Orthant64` and `ExactCache64`.
pub const INDEXED_RECENT_WINDOW: usize = 32;
/// Number of fixed partial sign tables.
pub const ORTHANT_TABLE_COUNT: usize = 2;
/// Coordinates consumed by one sign table (0..4 and 4..8).
pub const ORTHANT_TABLE_WIDTH: usize = 4;
/// Buckets per sign table (one per sign pattern of four coordinates).
pub const ORTHANT_BUCKETS: usize = 1 << ORTHANT_TABLE_WIDTH;
/// Postings retained per bucket (FIFO: the most recent writes).
pub const ORTHANT_DEPTH: usize = 8;
/// Probes per table: the query's own bucket and one neighbour.
pub const ORTHANT_PROBES: usize = 2;
/// Bounded follower depth of the `ExactCache64` traversal.
pub const CACHE_FOLLOWERS: usize = 32;
/// Total posting slots: tables x buckets x depth.
pub const POSTING_SLOTS: usize = ORTHANT_TABLE_COUNT * ORTHANT_BUCKETS * ORTHANT_DEPTH;
/// Maximum scored candidates for the bounded policies.
pub const MAX_CANDIDATES: usize = 64;

/// Buckets across all tables (the posting rings are addressed flat).
const POSTING_BUCKETS: usize = ORTHANT_TABLE_COUNT * ORTHANT_BUCKETS;
/// Empty-slot marker; occurrence `u32::MAX` is refused so this stays unambiguous.
const NONE: u32 = u32::MAX;

/// Which candidate-access rule a query uses.
///
/// `Full` is the retained full-context reference list and is diagnostic only.
/// `Recent64`, `Orthant64` and `ExactCache64` are the bounded policies: they
/// differ in storage and probe structure but all return at most
/// [`MAX_CANDIDATES`] scored candidates.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdmissionPolicy {
    /// Every prior occurrence. Diagnostic/reference only, O(len).
    Full,
    /// The most recent 64 occurrences.
    Recent64,
    /// Last 32 events; diagnostic removes the indexed complement.
    Recent32,
    /// Most recent 32 occurrences plus two fixed partial sign tables.
    Orthant64,
    /// Most recent 32 occurrences plus exact observed-token followers.
    ExactCache64,
}

impl AdmissionPolicy {
    /// Every declared policy, for matched-control reporting.
    pub const ALL: [Self; 5] = [
        Self::Full,
        Self::Recent64,
        Self::Recent32,
        Self::Orthant64,
        Self::ExactCache64,
    ];

    /// Strict parse of the serde `snake_case` names.
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "full" => Ok(Self::Full),
            "recent64" => Ok(Self::Recent64),
            "recent32" => Ok(Self::Recent32),
            "orthant64" => Ok(Self::Orthant64),
            "exact_cache64" => Ok(Self::ExactCache64),
            _ => Err(invalid(
                "admission policy must be full, recent64, orthant64 or exact_cache64",
            )),
        }
    }

    /// The serde name of this policy.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Recent64 => "recent64",
            Self::Recent32 => "recent32",
            Self::Orthant64 => "orthant64",
            Self::ExactCache64 => "exact_cache64",
        }
    }
}

impl Default for AdmissionPolicy {
    fn default() -> Self {
        Self::Full
    }
}

/// Work actually performed by one query.
///
/// The five contract counters are `query_coordinates_read`, `bucket_probes`,
/// `posting_visits`, `recent_visits` and `dedup_comparisons`. The remaining
/// counters are justified additions and are deliberately *not* folded into
/// those, so posting cost is never inflated by another structure's work:
///
/// - `cache_head_probes`/`cache_link_visits`: the `ExactCache64` head lookup
///   and follower links traversed.
/// - `history_visits`: occurrence slots read from the whole history. Only
///   `Full` is non-zero; it marks the diagnostic path as unbounded.
/// - `candidate_pool_entries`: the pre-dedup candidate pool, before duplicate removal; actual scored candidates
///   are the length of the returned occurrence list. Reporting only the unique
///   result size would understate the cost.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AdmissionWork {
    /// Query coordinates inspected (8 for `Orthant64`, 0 otherwise).
    pub query_coordinates_read: usize,
    pub query_coordinates_checked: usize,
    pub dedup_shifted_entries: usize,
    /// Posting buckets looked up (4 for `Orthant64`, 0 otherwise).
    pub bucket_probes: usize,
    /// Posting slots read out of probed buckets.
    pub posting_visits: usize,
    /// Recency-ring slots read out.
    pub recent_visits: usize,
    /// Comparisons spent ordering and de-duplicating the candidate pool.
    pub dedup_comparisons: usize,
    /// Cache head-array probes.
    pub cache_head_probes: usize,
    /// Follower links traversed.
    pub cache_link_visits: usize,
    /// Whole-history occurrence slots read (`Full` only).
    pub history_visits: usize,
    /// Candidate ids entering the pool before de-duplication.
    pub candidate_pool_entries: usize,
}

/// One query result: sorted unique prior occurrence ids plus the work done.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AdmissionSelection {
    /// Prior occurrence ids, ascending and unique, all strictly below the
    /// index length observed by this query.
    pub occurrences: Vec<usize>,
    /// Work performed by this query.
    pub work: AdmissionWork,
}

/// Cumulative work performed by all writes into one index.
///
/// Insertions are constant work per event: 64 coordinates validated, 8 indexed,
/// two bucket writes, one recency write and at most one cache link update.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AdmissionInsertWork {
    /// Key coordinates validated for length and finiteness.
    pub key_coordinates_checked: usize,
    /// Key coordinates actually indexed (tables x width).
    pub key_coordinates_indexed: usize,
    /// Sign-table bucket writes.
    pub bucket_writes: usize,
    /// Posting-ring slot writes.
    pub posting_slot_writes: usize,
    /// Recency-ring slot writes.
    pub recent_writes: usize,
    /// Cache head updates (zero only before the first observed token).
    pub cache_head_updates: usize,
    /// Cache link writes.
    pub cache_link_writes: usize,
}

/// Allocated logical index storage, for reporting.
///
/// These are logical slot counts and their `u32` payload bytes. Allocator
/// overhead and `Vec` capacity rounding are excluded; [`Self::logical_bytes`]
/// adds one `size_of::<AdmissionIndex>()` interior record (the count fields,
/// ring cursors and the three vector headers, so those headers appear once
/// here and once inside the per-array figures).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AdmissionFootprint {
    /// Occurrences written so far (identical to the index length).
    pub occurrences: usize,
    /// Recency-ring slots allocated.
    pub recent_slots: usize,
    /// Recency-ring slots currently holding an occurrence.
    pub recent_filled: usize,
    /// Recency-ring overwrites that discarded a still-recent occurrence.
    pub recent_evictions: usize,
    /// Fixed sign tables.
    pub posting_tables: usize,
    /// Buckets per table.
    pub posting_buckets_per_table: usize,
    /// Posting slots allocated.
    pub posting_slots: usize,
    /// Posting slot writes performed.
    pub posting_slots_written: usize,
    /// Posting writes that overwrote an occupied slot (depth [`ORTHANT_DEPTH`]).
    pub posting_evictions: usize,
    /// Cache head slots allocated (one per vocabulary token).
    pub cache_heads: usize,
    /// Cache link slots allocated (one per occurrence).
    pub cache_link_slots: usize,
    /// Occurrences filed under a predecessor token.
    pub cache_followers_filed: usize,
    /// Logical slots across all stored arrays.
    pub logical_slots: usize,
    /// Logical bytes of those slots plus one interior record.
    pub logical_bytes: usize,
}

/// One causal admission index per batch lane (per sequence).
///
/// All policies are maintained on every write, because the query policy
/// is chosen at read time; the policy selects the query path only.
#[derive(Clone, Debug)]
pub struct AdmissionIndex {
    occurrences: usize,
    recent: Vec<u32>,
    recent_next: usize,
    recent_filled: usize,
    recent_evictions: usize,
    postings: Vec<u32>,
    posting_next: [u32; POSTING_BUCKETS],
    posting_filled: [u32; POSTING_BUCKETS],
    posting_slots_written: usize,
    posting_evictions: usize,
    cache_heads: Vec<u32>,
    cache_links: Vec<u32>,
    cache_followers_filed: usize,
    last_observed_token: Option<u32>,
    insert_work: AdmissionInsertWork,
}

impl AdmissionIndex {
    /// An empty index for one batch lane.
    pub fn new() -> Self {
        Self {
            occurrences: 0,
            recent: vec![NONE; RECENT_CAPACITY],
            recent_next: 0,
            recent_filled: 0,
            recent_evictions: 0,
            postings: vec![NONE; POSTING_SLOTS],
            posting_next: [0; POSTING_BUCKETS],
            posting_filled: [0; POSTING_BUCKETS],
            posting_slots_written: 0,
            posting_evictions: 0,
            cache_heads: vec![NONE; VOCAB_SIZE],
            cache_links: Vec::new(),
            cache_followers_filed: 0,
            last_observed_token: None,
            insert_work: AdmissionInsertWork::default(),
        }
    }

    /// Exact occurrence count; also the identity the next write will use.
    pub fn len(&self) -> usize {
        self.occurrences
    }

    pub fn is_empty(&self) -> bool {
        self.occurrences == 0
    }

    /// The next exact monotone occurrence identity.
    pub fn next_occurrence(&self) -> usize {
        self.occurrences
    }

    /// Cumulative insertion work since construction.
    pub fn insert_work(&self) -> AdmissionInsertWork {
        self.insert_work
    }

    /// Allocated logical slots/bytes, for reporting and cost closeout.
    pub fn footprint(&self) -> AdmissionFootprint {
        let slot = std::mem::size_of::<u32>();
        let posting_bytes = self.postings.len() * slot;
        let recent_bytes = self.recent.len() * slot;
        let head_bytes = self.cache_heads.len() * slot;
        let link_bytes = self.cache_links.len() * slot;
        AdmissionFootprint {
            occurrences: self.occurrences,
            recent_slots: self.recent.len(),
            recent_filled: self.recent_filled,
            recent_evictions: self.recent_evictions,
            posting_tables: ORTHANT_TABLE_COUNT,
            posting_buckets_per_table: ORTHANT_BUCKETS,
            posting_slots: self.postings.len(),
            posting_slots_written: self.posting_slots_written,
            posting_evictions: self.posting_evictions,
            cache_heads: self.cache_heads.len(),
            cache_link_slots: self.cache_links.len(),
            cache_followers_filed: self.cache_followers_filed,
            logical_slots: self.postings.len()
                + self.recent.len()
                + self.cache_heads.len()
                + self.cache_links.len(),
            logical_bytes: posting_bytes
                + recent_bytes
                + head_bytes
                + link_bytes
                + std::mem::size_of::<Self>(),
        }
    }

    /// Query the fixed sign table bucket of a 64-D key (reporting/tests).
    pub fn bucket_of(key: &[f32], table: usize) -> Result<usize> {
        validate_key(key, "bucket")?;
        if table >= ORTHANT_TABLE_COUNT {
            return Err(invalid("admission sign table index must be 0 or 1"));
        }
        Ok(sign_bucket(key, table))
    }

    /// The four probed buckets: per table, own bucket then neighbour.
    pub fn probe_buckets(key: &[f32]) -> Result<[usize; ORTHANT_PROBES * ORTHANT_TABLE_COUNT]> {
        validate_key(key, "probe")?;
        let mut probes = [0usize; ORTHANT_PROBES * ORTHANT_TABLE_COUNT];
        for table in 0..ORTHANT_TABLE_COUNT {
            let own = sign_bucket(key, table);
            probes[table * ORTHANT_PROBES] = own;
            probes[table * ORTHANT_PROBES + 1] = neighbour_bucket(key, table, own);
        }
        Ok(probes)
    }

    /// Write the next exact monotone occurrence after the caller's read.
    ///
    /// `key` is the event's 64-D read key (the identity the caller's ranker
    /// scores), `observed_token` its exact observed input token. The occurrence
    /// is filed under the *preceding* observed token so that a later query with
    /// a current input token retrieves the historical followers of that token.
    pub fn insert(&mut self, key: &[f32], observed_token: u32) -> Result<()> {
        validate_key(key, "insert")?;
        validate_token(observed_token)?;
        if self.occurrences >= NONE as usize {
            return Err(invalid("admission index occurrence identity exhausted"));
        }
        let occurrence = self.occurrences as u32;

        // Recency window: FIFO, newest last when read.
        if self.recent_filled == RECENT_CAPACITY {
            self.recent_evictions += 1;
        }
        self.recent[self.recent_next] = occurrence;
        self.recent_next = (self.recent_next + 1) % RECENT_CAPACITY;
        if self.recent_filled < RECENT_CAPACITY {
            self.recent_filled += 1;
        }

        // Two fixed partial sign tables, depth-8 FIFO postings per bucket.
        for table in 0..ORTHANT_TABLE_COUNT {
            let bucket = sign_bucket(key, table);
            let index = table * ORTHANT_BUCKETS + bucket;
            let cursor = self.posting_next[index] as usize;
            if self.posting_filled[index] as usize == ORTHANT_DEPTH {
                self.posting_evictions += 1;
            }
            self.postings[index * ORTHANT_DEPTH + cursor] = occurrence;
            self.posting_next[index] = ((cursor + 1) % ORTHANT_DEPTH) as u32;
            if (self.posting_filled[index] as usize) < ORTHANT_DEPTH {
                self.posting_filled[index] += 1;
            }
            self.posting_slots_written += 1;
        }

        // Exact observed-token follower chain: one link per occurrence.
        let mut cache_head_updates = 0usize;
        let mut cache_link_writes = 0usize;
        match self.last_observed_token {
            Some(previous) => {
                let head = self.cache_heads[previous as usize];
                self.cache_links.push(head);
                self.cache_heads[previous as usize] = occurrence;
                self.cache_followers_filed += 1;
                cache_head_updates = 1;
                cache_link_writes = 1;
            }
            None => self.cache_links.push(NONE),
        }
        self.last_observed_token = Some(observed_token);

        self.occurrences += 1;
        self.insert_work.key_coordinates_checked += KEY_WIDTH;
        self.insert_work.key_coordinates_indexed += ORTHANT_TABLE_COUNT * ORTHANT_TABLE_WIDTH;
        self.insert_work.bucket_writes += ORTHANT_TABLE_COUNT;
        self.insert_work.posting_slot_writes += ORTHANT_TABLE_COUNT;
        self.insert_work.recent_writes += 1;
        self.insert_work.cache_head_updates += cache_head_updates;
        self.insert_work.cache_link_writes += cache_link_writes;
        Ok(())
    }

    /// Admit prior occurrences for one read. Never mutates the index.
    ///
    /// `input_token` is always the current observed input token (never a
    /// target). It is validated for every policy and used only by
    /// `ExactCache64`. `query_key` is validated and used only by `Orthant64`;
    /// the other policies need no query vector at all.
    pub fn query(
        &self,
        policy: AdmissionPolicy,
        query_key: &[f32],
        input_token: u32,
    ) -> Result<AdmissionSelection> {
        validate_token(input_token)?;
        let mut work = AdmissionWork::default();
        let mut pool: Vec<usize> = Vec::new();
        let mut already_ordered = false;
        match policy {
            AdmissionPolicy::Full => {
                work.history_visits = self.occurrences;
                pool.reserve(self.occurrences);
                for occurrence in 0..self.occurrences {
                    pool.push(occurrence);
                }
                already_ordered = true;
            }
            AdmissionPolicy::Recent32 => {
                self.read_recent(INDEXED_RECENT_WINDOW, &mut pool, &mut work);
                already_ordered = true;
            }
            AdmissionPolicy::Recent64 => {
                self.read_recent(RECENT_CAPACITY, &mut pool, &mut work);
                already_ordered = true;
            }
            AdmissionPolicy::Orthant64 => {
                work.query_coordinates_read = 2 * ORTHANT_TABLE_COUNT * ORTHANT_TABLE_WIDTH;
                work.query_coordinates_checked = KEY_WIDTH;
                let probes = Self::probe_buckets(query_key)?;
                work.bucket_probes = probes.len();
                for (probe, bucket) in probes.iter().enumerate() {
                    let table = probe / ORTHANT_PROBES;
                    self.read_bucket(table, *bucket, &mut pool, &mut work);
                }
                self.read_recent(INDEXED_RECENT_WINDOW, &mut pool, &mut work);
            }
            AdmissionPolicy::ExactCache64 => {
                self.read_recent(INDEXED_RECENT_WINDOW, &mut pool, &mut work);
                work.cache_head_probes = 1;
                self.collect_cache_followers(input_token, CACHE_FOLLOWERS, &mut pool, &mut work);
            }
        }
        work.candidate_pool_entries = pool.len();
        let occurrences = if already_ordered {
            pool
        } else {
            sort_unique(pool, &mut work)
        };
        Ok(AdmissionSelection { occurrences, work })
    }

    fn read_recent(&self, window: usize, pool: &mut Vec<usize>, work: &mut AdmissionWork) {
        let take = self.recent_filled.min(window);
        if take == 0 {
            return;
        }
        let start = (self.recent_next + RECENT_CAPACITY - take) % RECENT_CAPACITY;
        for offset in 0..take {
            let slot = (start + offset) % RECENT_CAPACITY;
            pool.push(self.recent[slot] as usize);
            work.recent_visits += 1;
        }
    }

    fn read_bucket(
        &self,
        table: usize,
        bucket: usize,
        pool: &mut Vec<usize>,
        work: &mut AdmissionWork,
    ) {
        let index = table * ORTHANT_BUCKETS + bucket;
        let filled = self.posting_filled[index] as usize;
        let base = index * ORTHANT_DEPTH;
        for offset in 0..filled {
            pool.push(self.postings[base + offset] as usize);
            work.posting_visits += 1;
        }
    }

    /// Newest-first exact followers of `token`, bounded by `limit` links.
    fn collect_cache_followers(
        &self,
        token: u32,
        limit: usize,
        pool: &mut Vec<usize>,
        work: &mut AdmissionWork,
    ) {
        let mut link = self.cache_heads[token as usize];
        let mut taken = 0usize;
        while link != NONE && taken < limit {
            pool.push(link as usize);
            work.cache_link_visits += 1;
            taken += 1;
            link = self.cache_links[link as usize];
        }
    }
}

impl Default for AdmissionIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Sign bit `i` of `table` is set when coordinate `i` is `>= 0`, so `-0.0`
/// counts as positive. Non-finite keys are refused before use, which keeps the
/// bucket assignment total and deterministic.
fn sign_bucket(key: &[f32], table: usize) -> usize {
    let base = table * ORTHANT_TABLE_WIDTH;
    let mut bucket = 0usize;
    for offset in 0..ORTHANT_TABLE_WIDTH {
        if key[base + offset] >= 0.0 {
            bucket |= 1 << offset;
        }
    }
    bucket
}

/// Neighbour bucket reached by flipping the smallest-magnitude coordinate's
/// sign bit; ties keep the lowest coordinate index. The result always differs
/// from `own`, so a probe never reads the same bucket twice.
fn neighbour_bucket(key: &[f32], table: usize, own: usize) -> usize {
    let base = table * ORTHANT_TABLE_WIDTH;
    let mut smallest = 0usize;
    let mut magnitude = key[base].abs();
    for offset in 1..ORTHANT_TABLE_WIDTH {
        let candidate = key[base + offset].abs();
        if candidate < magnitude {
            magnitude = candidate;
            smallest = offset;
        }
    }
    own ^ (1 << smallest)
}

/// Binary-insertion into a sorted, duplicate-free vector. Every comparison is
/// charged to `dedup_comparisons`, so the reported cost is the cost paid. The
/// pool is bounded by [`MAX_CANDIDATES`], so this never touches the history.
fn sort_unique(pool: Vec<usize>, work: &mut AdmissionWork) -> Vec<usize> {
    let mut sorted: Vec<usize> = Vec::with_capacity(pool.len());
    for candidate in pool {
        let found = sorted.binary_search_by(|probe| {
            work.dedup_comparisons += 1;
            probe.cmp(&candidate)
        });
        if let Err(position) = found {
            work.dedup_shifted_entries += sorted.len() - position;
            sorted.insert(position, candidate);
        }
    }
    sorted
}

fn validate_key(key: &[f32], context: &str) -> Result<()> {
    if key.len() != KEY_WIDTH {
        return Err(invalid(format!(
            "admission {context} key must be {KEY_WIDTH} wide"
        )));
    }
    if key.iter().any(|value| !value.is_finite()) {
        return Err(invalid(format!("admission {context} key must be finite")));
    }
    Ok(())
}

fn validate_token(token: u32) -> Result<()> {
    if token as usize >= VOCAB_SIZE {
        return Err(invalid("admission token must be within vocabulary4096"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILLER_COUNT: usize = 40;

    /// A 64-D key with explicit leading coordinates; the rest are unused by the
    /// two sign tables and are left positive and finite.
    fn key_from(coordinates: [f32; 8]) -> Vec<f32> {
        let mut key = vec![0.5f32; KEY_WIDTH];
        key[..8].copy_from_slice(&coordinates);
        key
    }

    /// A key whose first eight coordinates carry the given signs.
    fn signed_key(signs: [bool; 8], magnitude: f32) -> Vec<f32> {
        let mut coordinates = [magnitude; 8];
        for (offset, positive) in signs.iter().enumerate() {
            coordinates[offset] = if *positive { magnitude } else { -magnitude };
        }
        key_from(coordinates)
    }

    /// Deterministic key stream (no external dependency), values in [-1, 1).
    fn stream_key(seed: u64) -> Vec<f32> {
        let mut state = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let mut key = vec![0f32; KEY_WIDTH];
        for value in key.iter_mut() {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            *value = (state >> 40) as f32 / (1u64 << 23) as f32 - 1.0;
        }
        key
    }

    /// Fixture self-check: the filler must land outside every bucket the query
    /// probes, otherwise the fixture would admit it for the wrong reason.
    fn assert_filler_disjoint(query: &[f32], filler: &[f32]) -> Result<()> {
        let probes = AdmissionIndex::probe_buckets(query)?;
        for table in 0..ORTHANT_TABLE_COUNT {
            let filler_bucket = AdmissionIndex::bucket_of(filler, table)?;
            for probe in 0..ORTHANT_PROBES {
                assert_ne!(
                    filler_bucket,
                    probes[table * ORTHANT_PROBES + probe],
                    "filler bucket must not be probed (table {table})"
                );
            }
        }
        Ok(())
    }

    /// Plant one target occurrence, then fill the recent window with events the
    /// query cannot probe, and return the admitted set.
    fn planted_target_result(query: &[f32], target: &[f32], filler: &[f32]) -> Result<Vec<usize>> {
        let mut index = AdmissionIndex::new();
        index.insert(target, 11)?;
        for step in 0..FILLER_COUNT {
            index.insert(filler, 22 + step as u32)?;
        }
        Ok(index
            .query(AdmissionPolicy::Orthant64, query, 22)?
            .occurrences)
    }

    /// The newest 32 occurrences after `FILLER_COUNT + 1` writes.
    fn recent_expectation() -> Vec<usize> {
        let total = FILLER_COUNT + 1;
        (total - INDEXED_RECENT_WINDOW..total).collect()
    }

    #[test]
    fn empty_index_returns_no_occurrences_for_every_policy() -> Result<()> {
        let index = AdmissionIndex::new();
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
        assert_eq!(index.next_occurrence(), 0);
        let key = signed_key([true; 8], 0.5);
        for policy in AdmissionPolicy::ALL {
            let selection = index.query(policy, &key, 7)?;
            assert!(selection.occurrences.is_empty(), "{policy:?}");
            assert_eq!(selection.work.candidate_pool_entries, 0, "{policy:?}");
            assert_eq!(selection.work.posting_visits, 0, "{policy:?}");
            assert_eq!(selection.work.recent_visits, 0, "{policy:?}");
            assert_eq!(selection.work.cache_link_visits, 0, "{policy:?}");
            assert_eq!(selection.work.history_visits, 0, "{policy:?}");
            assert_eq!(selection.work.dedup_comparisons, 0, "{policy:?}");
        }
        // Only the paths that touch no other structure do exactly zero work.
        assert_eq!(
            index.query(AdmissionPolicy::Full, &key, 7)?.work,
            AdmissionWork::default()
        );
        assert_eq!(
            index.query(AdmissionPolicy::Recent64, &key, 7)?.work,
            AdmissionWork::default()
        );
        // Fixed probe cost is paid even against an empty index, and is reported.
        let orthant = index.query(AdmissionPolicy::Orthant64, &key, 7)?.work;
        assert_eq!(
            orthant.query_coordinates_read,
            2 * ORTHANT_TABLE_COUNT * ORTHANT_TABLE_WIDTH
        );
        assert_eq!(orthant.bucket_probes, ORTHANT_PROBES * ORTHANT_TABLE_COUNT);
        let cached = index.query(AdmissionPolicy::ExactCache64, &key, 7)?.work;
        assert_eq!(cached.cache_head_probes, 1);
        assert_eq!(cached.cache_link_visits, 0);
        Ok(())
    }

    #[test]
    fn insert_order_is_exact_and_monotone() -> Result<()> {
        let mut index = AdmissionIndex::new();
        for step in 0..5u32 {
            let before = index.query(AdmissionPolicy::Recent64, &[], step)?;
            assert_eq!(before.occurrences.len(), step as usize);
            assert!(before
                .occurrences
                .iter()
                .all(|&occurrence| occurrence < index.len()));
            index.insert(&stream_key(step as u64), 100 + step)?;
            assert_eq!(index.len(), step as usize + 1);
            assert_eq!(index.next_occurrence(), step as usize + 1);
        }
        let after = index.query(AdmissionPolicy::Recent64, &[], 7)?;
        assert_eq!(after.occurrences, vec![0, 1, 2, 3, 4]);
        Ok(())
    }

    #[test]
    fn same_observed_token_keeps_distinct_occurrence_ids() -> Result<()> {
        let mut index = AdmissionIndex::new();
        let key = signed_key([true, false, true, false, true, false, true, false], 0.25);
        for _ in 0..8 {
            index.insert(&key, 42)?;
        }
        let selection = index.query(AdmissionPolicy::Recent64, &[], 43)?;
        assert_eq!(selection.occurrences, (0..8).collect::<Vec<_>>());
        let mut followers = Vec::new();
        let mut work = AdmissionWork::default();
        index.collect_cache_followers(42, CACHE_FOLLOWERS, &mut followers, &mut work);
        assert_eq!(followers, vec![7, 6, 5, 4, 3, 2, 1]);
        assert_eq!(work.cache_link_visits, 7);
        let cached = index.query(AdmissionPolicy::ExactCache64, &[], 42)?;
        assert_eq!(cached.occurrences, (0..8).collect::<Vec<_>>());
        Ok(())
    }

    #[test]
    fn query_is_pure_and_repeated_queries_agree() -> Result<()> {
        let mut index = AdmissionIndex::new();
        for step in 0..40usize {
            index.insert(&stream_key(step as u64), (step % VOCAB_SIZE) as u32)?;
        }
        let key = stream_key(9);
        let footprint = index.footprint();
        let first = index.query(AdmissionPolicy::Orthant64, &key, 5)?;
        let second = index.query(AdmissionPolicy::Orthant64, &key, 5)?;
        assert_eq!(first, second);
        assert_eq!(index.footprint(), footprint);
        assert_eq!(index.len(), 40);
        Ok(())
    }

    #[test]
    fn orthant_rejects_wrong_length_and_nonfinite_queries() -> Result<()> {
        let index = AdmissionIndex::new();
        assert!(index
            .query(AdmissionPolicy::Orthant64, &[0.5f32; 8], 1)
            .is_err());
        let mut nan = signed_key([true; 8], 0.5);
        nan[3] = f32::NAN;
        assert!(index.query(AdmissionPolicy::Orthant64, &nan, 1).is_err());
        let mut infinite = signed_key([true; 8], 0.5);
        infinite[0] = f32::INFINITY;
        assert!(index
            .query(AdmissionPolicy::Orthant64, &infinite, 1)
            .is_err());
        assert!(index
            .query(AdmissionPolicy::Full, &[], VOCAB_SIZE as u32)
            .is_err());
        assert!(index
            .query(AdmissionPolicy::Recent64, &[], VOCAB_SIZE as u32)
            .is_err());
        let mut index = AdmissionIndex::new();
        assert!(index
            .insert(&signed_key([true; 8], 0.5), VOCAB_SIZE as u32)
            .is_err());
        let mut short = signed_key([true; 8], 0.5);
        short.truncate(KEY_WIDTH - 1);
        assert!(index.insert(&short, 0).is_err());
        let mut nan_insert = signed_key([true; 8], 0.5);
        nan_insert[10] = f32::NAN;
        assert!(index.insert(&nan_insert, 0).is_err());
        assert!(index.is_empty());
        assert!(AdmissionIndex::bucket_of(&signed_key([true; 8], 0.5), 2).is_err());
        Ok(())
    }

    #[test]
    fn signed_zero_counts_positive_and_tables_split_on_coordinates() -> Result<()> {
        let key = signed_key([true, false, true, true, true, false, true, true], 0.5);
        let mut negative_zero = key.clone();
        negative_zero[0] = -0.0;
        assert_eq!(
            AdmissionIndex::bucket_of(&key, 0)?,
            AdmissionIndex::bucket_of(&negative_zero, 0)?
        );
        assert_eq!(AdmissionIndex::bucket_of(&key, 0)? & 1, 1);
        // Table 1 reads coordinates 4..8, so identical sign patterns agree.
        assert_eq!(
            AdmissionIndex::bucket_of(&key, 0)?,
            AdmissionIndex::bucket_of(&key, 1)?
        );
        let probe = AdmissionIndex::probe_buckets(&key)?;
        assert_eq!(probe.len(), ORTHANT_PROBES * ORTHANT_TABLE_COUNT);
        for table in 0..ORTHANT_TABLE_COUNT {
            assert_ne!(
                probe[table * ORTHANT_PROBES],
                probe[table * ORTHANT_PROBES + 1]
            );
        }
        Ok(())
    }

    #[test]
    fn orthant_neighbour_probe_admits_only_the_correct_flip() -> Result<()> {
        // Unique smallest coordinate is index 0, so the neighbour flips bit 0.
        let query = key_from([0.1, 0.6, 0.7, 0.8, 0.2, 0.3, 0.4, 0.5]);
        assert_eq!(AdmissionIndex::probe_buckets(&query)?, [15, 14, 15, 14]);
        let filler = signed_key([false, false, true, true, true, true, false, false], 0.75);
        assert_filler_disjoint(&query, &filler)?;
        let correct = signed_key([false, true, true, true, true, true, false, false], 0.4);
        let wrong = signed_key([true, false, true, true, true, true, false, false], 0.4);
        assert_eq!(AdmissionIndex::bucket_of(&correct, 0)?, 14);
        assert_eq!(AdmissionIndex::bucket_of(&correct, 1)?, 3);
        assert_eq!(AdmissionIndex::bucket_of(&wrong, 0)?, 13);
        assert_eq!(AdmissionIndex::bucket_of(&wrong, 1)?, 3);

        let mut expected = recent_expectation();
        expected.insert(0, 0);
        assert_eq!(planted_target_result(&query, &correct, &filler)?, expected);
        // Negative control: flipping the wrong coordinate admits nothing extra.
        assert_eq!(
            planted_target_result(&query, &wrong, &filler)?,
            recent_expectation()
        );
        Ok(())
    }

    #[test]
    fn orthant_neighbour_flip_breaks_ties_by_lowest_coordinate() -> Result<()> {
        // |q0| == |q1|, so the flip must take coordinate 0 (lowest index).
        let query = key_from([0.5, -0.5, 0.7, 0.8, 0.2, 0.3, 0.4, 0.5]);
        assert_eq!(query[0].abs(), query[1].abs());
        assert_eq!(AdmissionIndex::probe_buckets(&query)?, [13, 12, 15, 14]);
        let filler = signed_key([false, true, true, true, true, true, false, false], 0.75);
        assert_filler_disjoint(&query, &filler)?;
        let correct = signed_key([false, false, true, true, true, true, false, false], 0.4);
        let wrong = signed_key([true, true, true, true, true, true, false, false], 0.4);
        assert_eq!(AdmissionIndex::bucket_of(&correct, 0)?, 12);
        assert_eq!(AdmissionIndex::bucket_of(&wrong, 0)?, 15);

        let mut expected = recent_expectation();
        expected.insert(0, 0);
        assert_eq!(planted_target_result(&query, &correct, &filler)?, expected);
        assert_eq!(
            planted_target_result(&query, &wrong, &filler)?,
            recent_expectation()
        );
        Ok(())
    }

    #[test]
    fn orthant_deduplicates_overlapping_sources() -> Result<()> {
        let mut index = AdmissionIndex::new();
        let key = signed_key([true, false, true, false, true, true, false, false], 0.5);
        for step in 0..5u32 {
            index.insert(&key, 30 + step)?;
        }
        let selection = index.query(AdmissionPolicy::Orthant64, &key, 30)?;
        assert_eq!(selection.occurrences, (0..5).collect::<Vec<_>>());
        assert_eq!(
            selection
                .occurrences
                .iter()
                .filter(|occurrence| **occurrence == 2)
                .count(),
            1
        );
        // Every occurrence is in both probe buckets and the recent window.
        assert_eq!(selection.work.posting_visits, 10);
        assert_eq!(selection.work.recent_visits, 5);
        assert_eq!(selection.work.candidate_pool_entries, 15);
        assert!(selection.work.candidate_pool_entries > selection.occurrences.len());
        assert!(selection.work.dedup_comparisons > 0);
        Ok(())
    }

    #[test]
    fn orthant_work_is_bounded_at_long_history() -> Result<()> {
        let mut index = AdmissionIndex::new();
        for step in 0..4096usize {
            index.insert(&stream_key(step as u64), (step % VOCAB_SIZE) as u32)?;
        }
        assert_eq!(index.len(), 4096);
        let selection = index.query(AdmissionPolicy::Orthant64, &stream_key(11), 7)?;
        assert_eq!(selection.work.bucket_probes, 4);
        assert_eq!(selection.work.query_coordinates_read, 16);
        assert_eq!(selection.work.history_visits, 0);
        assert!(selection.work.posting_visits <= 32);
        assert_eq!(selection.work.recent_visits, INDEXED_RECENT_WINDOW);
        assert!(selection.work.candidate_pool_entries <= MAX_CANDIDATES);
        assert!(selection.occurrences.len() <= MAX_CANDIDATES);
        assert!(selection.work.dedup_comparisons <= MAX_CANDIDATES * 7 + MAX_CANDIDATES);
        assert!(selection
            .occurrences
            .iter()
            .all(|&occurrence| occurrence < 4096));
        assert!(selection
            .occurrences
            .windows(2)
            .all(|pair| pair[0] < pair[1]));

        let exact = index.query(AdmissionPolicy::ExactCache64, &[], 3000)?;
        assert!(exact.occurrences.len() <= MAX_CANDIDATES);
        assert!(exact.work.cache_link_visits <= CACHE_FOLLOWERS);
        assert_eq!(exact.work.history_visits, 0);

        // The diagnostic reference path is the only unbounded one.
        let reference = index.query(AdmissionPolicy::Full, &[], 7)?;
        assert_eq!(reference.occurrences.len(), 4096);
        assert_eq!(reference.work.history_visits, 4096);
        assert_eq!(reference.work.candidate_pool_entries, 4096);
        assert_eq!(reference.work.posting_visits, 0);
        Ok(())
    }

    #[test]
    fn full_policy_is_diagnostic_and_reports_history_visits() -> Result<()> {
        let mut index = AdmissionIndex::new();
        for step in 0..300usize {
            index.insert(&stream_key(step as u64), (step % VOCAB_SIZE) as u32)?;
        }
        let selection = index.query(AdmissionPolicy::Full, &[], 7)?;
        assert_eq!(selection.occurrences, (0..300).collect::<Vec<_>>());
        assert_eq!(selection.work.history_visits, 300);
        assert_eq!(selection.work.candidate_pool_entries, 300);
        assert_eq!(selection.work.bucket_probes, 0);
        assert_eq!(selection.work.posting_visits, 0);
        assert_eq!(selection.work.recent_visits, 0);
        assert_eq!(selection.work.query_coordinates_read, 0);
        assert_eq!(selection.work.dedup_comparisons, 0);
        Ok(())
    }

    #[test]
    fn recent64_returns_only_the_last_64() -> Result<()> {
        let mut index = AdmissionIndex::new();
        for step in 0..200usize {
            index.insert(&stream_key(step as u64), (step % VOCAB_SIZE) as u32)?;
        }
        let selection = index.query(AdmissionPolicy::Recent64, &[], 3)?;
        assert_eq!(selection.occurrences, (136..200).collect::<Vec<_>>());
        assert_eq!(selection.work.recent_visits, RECENT_CAPACITY);
        assert_eq!(selection.work.posting_visits, 0);
        assert_eq!(selection.work.dedup_comparisons, 0);
        Ok(())
    }

    #[test]
    fn exact_cache_followers_match_predecessor_token() -> Result<()> {
        let mut index = AdmissionIndex::new();
        let key = stream_key(3);
        for (step, token) in [7u32, 11, 7, 13, 7, 11].iter().enumerate() {
            assert_eq!(index.next_occurrence(), step);
            index.insert(&key, *token)?;
        }
        let mut followers = Vec::new();
        let mut work = AdmissionWork::default();
        index.collect_cache_followers(7, CACHE_FOLLOWERS, &mut followers, &mut work);
        assert_eq!(followers, vec![5, 3, 1]);
        assert_eq!(work.cache_link_visits, 3);
        let mut other = Vec::new();
        let mut other_work = AdmissionWork::default();
        index.collect_cache_followers(11, CACHE_FOLLOWERS, &mut other, &mut other_work);
        assert_eq!(other, vec![2]);

        let selection = index.query(AdmissionPolicy::ExactCache64, &[], 7)?;
        assert_eq!(selection.occurrences, (0..6).collect::<Vec<_>>());
        assert_eq!(selection.work.cache_head_probes, 1);
        assert_eq!(selection.work.cache_link_visits, 3);
        assert_eq!(selection.work.recent_visits, 6);

        let unseen = index.query(AdmissionPolicy::ExactCache64, &[], 99)?;
        assert_eq!(unseen.occurrences, (0..6).collect::<Vec<_>>());
        assert_eq!(unseen.work.cache_link_visits, 0);
        Ok(())
    }

    #[test]
    fn exact_cache_traversal_is_bounded_and_keeps_latest() -> Result<()> {
        let mut index = AdmissionIndex::new();
        let key = stream_key(5);
        let events = 100usize;
        for step in 0..events {
            let token = if step % 2 == 0 { 7u32 } else { 11u32 };
            index.insert(&key, token)?;
        }
        // Odd ids are preceded by token 7, so they are filed under it.
        let mut followers = Vec::new();
        let mut work = AdmissionWork::default();
        index.collect_cache_followers(7, CACHE_FOLLOWERS, &mut followers, &mut work);
        assert_eq!(followers.len(), CACHE_FOLLOWERS);
        assert_eq!(work.cache_link_visits, CACHE_FOLLOWERS);
        let newest = 99usize;
        for (position, follower) in followers.iter().enumerate() {
            assert_eq!(*follower, newest - 2 * position);
        }

        let mut expected: Vec<usize> = (events - INDEXED_RECENT_WINDOW..events).collect();
        for follower in &followers {
            if !expected.contains(follower) {
                expected.push(*follower);
            }
        }
        expected.sort_unstable();
        let selection = index.query(AdmissionPolicy::ExactCache64, &[], 7)?;
        assert_eq!(selection.occurrences, expected);
        assert_eq!(selection.work.cache_link_visits, CACHE_FOLLOWERS);
        assert!(selection.work.candidate_pool_entries <= MAX_CANDIDATES);
        Ok(())
    }

    #[test]
    fn token_identity_never_aliases_across_equal_keys() -> Result<()> {
        let mut index = AdmissionIndex::new();
        let key = stream_key(17);
        index.insert(&key, 100)?;
        index.insert(&key, 200)?;
        index.insert(&key, 101)?;
        index.insert(&key, 300)?;
        // Identical keys: the sign-table channel cannot tell these events apart.
        assert_eq!(
            AdmissionIndex::bucket_of(&key, 0)?,
            AdmissionIndex::bucket_of(&stream_key(17), 0)?
        );
        let mut under_100 = Vec::new();
        let mut work = AdmissionWork::default();
        index.collect_cache_followers(100, CACHE_FOLLOWERS, &mut under_100, &mut work);
        let mut under_101 = Vec::new();
        index.collect_cache_followers(101, CACHE_FOLLOWERS, &mut under_101, &mut work);
        assert_eq!(under_100, vec![1]);
        assert_eq!(under_101, vec![3]);
        assert_ne!(under_100, under_101);
        Ok(())
    }

    #[test]
    fn posting_eviction_is_index_local_not_global_absence() -> Result<()> {
        let mut index = AdmissionIndex::new();
        let key = signed_key([true, false, true, false, false, true, false, true], 0.5);
        for _ in 0..40u32 {
            index.insert(&key, 5)?;
        }
        let footprint = index.footprint();
        assert_eq!(footprint.posting_slots_written, 2 * 40);
        assert!(footprint.posting_evictions >= 32);
        // 40 writes stay inside the 64-slot recency ring, so recency evicts
        // nothing here; only the depth-8 posting rings overwrite.
        assert_eq!(footprint.recent_filled, 40);
        assert_eq!(footprint.recent_evictions, 0);
        assert_eq!(footprint.occurrences, 40);

        let selection = index.query(AdmissionPolicy::Orthant64, &key, 5)?;
        assert_eq!(
            selection.occurrences,
            (8..40).collect::<Vec<_>>(),
            "postings keep the newest 8 per bucket; recency adds the newest 32"
        );
        assert!(selection
            .occurrences
            .iter()
            .all(|&occurrence| occurrence >= 8));

        // Evicted from the posting window is not absence: the diagnostic list
        // remains the reference, and only the ring overwrite proves eviction.
        let reference = index.query(AdmissionPolicy::Full, &[], 5)?;
        assert_eq!(reference.occurrences.len(), 40);
        assert_eq!(reference.occurrences.first(), Some(&0));
        Ok(())
    }

    #[test]
    fn insert_work_is_constant_per_event_and_footprint_is_explicit() -> Result<()> {
        let mut index = AdmissionIndex::new();
        let events = 1000usize;
        for step in 0..events {
            index.insert(&stream_key(step as u64), (step % VOCAB_SIZE) as u32)?;
        }
        let work = index.insert_work();
        assert_eq!(work.key_coordinates_checked, events * KEY_WIDTH);
        assert_eq!(
            work.key_coordinates_indexed,
            events * ORTHANT_TABLE_COUNT * ORTHANT_TABLE_WIDTH
        );
        assert_eq!(work.bucket_writes, events * ORTHANT_TABLE_COUNT);
        assert_eq!(work.posting_slot_writes, events * ORTHANT_TABLE_COUNT);
        assert_eq!(work.recent_writes, events);
        assert_eq!(work.cache_head_updates, events - 1);
        assert_eq!(work.cache_link_writes, events - 1);

        let footprint = index.footprint();
        assert_eq!(footprint.occurrences, events);
        assert_eq!(footprint.posting_slots, POSTING_SLOTS);
        assert_eq!(footprint.posting_tables, ORTHANT_TABLE_COUNT);
        assert_eq!(footprint.posting_buckets_per_table, ORTHANT_BUCKETS);
        assert_eq!(footprint.recent_slots, RECENT_CAPACITY);
        assert_eq!(footprint.cache_heads, VOCAB_SIZE);
        assert_eq!(footprint.cache_link_slots, events);
        assert_eq!(footprint.cache_followers_filed, events - 1);
        assert!(footprint.posting_slots_written > footprint.posting_slots);
        assert!(footprint.posting_evictions > 0);
        assert!(footprint.recent_evictions > 0);
        assert!(footprint.logical_slots > VOCAB_SIZE);
        assert!(footprint.logical_bytes > footprint.cache_heads * std::mem::size_of::<u32>());
        Ok(())
    }

    #[test]
    fn policy_names_parse_and_round_trip_serde() -> Result<()> {
        assert_eq!(AdmissionPolicy::default(), AdmissionPolicy::Full);
        assert_eq!(AdmissionPolicy::ALL.len(), 5);
        for policy in AdmissionPolicy::ALL {
            assert_eq!(AdmissionPolicy::parse(policy.name())?, policy);
            let json = serde_json::to_string(&policy)?;
            assert_eq!(json, format!("\"{}\"", policy.name()));
            assert_eq!(serde_json::from_str::<AdmissionPolicy>(&json)?, policy);
        }
        assert!(AdmissionPolicy::parse("Full").is_err());
        assert!(AdmissionPolicy::parse("recent_64").is_err());

        let work = AdmissionWork::default();
        assert_eq!(work.candidate_pool_entries, 0);
        assert_eq!(work.query_coordinates_read, 0);
        let json = serde_json::to_string(&work)?;
        assert_eq!(serde_json::from_str::<AdmissionWork>(&json)?, work);
        let selection = AdmissionSelection::default();
        assert!(selection.occurrences.is_empty());
        assert_eq!(selection.work, work);
        Ok(())
    }
}
