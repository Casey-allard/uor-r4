//! Exact geometric prime/semiprime address resolution over `uor-r4-object-store`.
//!
//! Connects a canonical prime-factor address (`PrimeAddress`) to content-addressed
//! payloads stored in [`Store`] (`ByteSpan`, `WholeObject`, or `TokenOccurrence`),
//! with versioned overwrite tracking (`Current` and `Previous`), bounded capacity
//! eviction (`Evicted`), and division-free sorted-merge GCD sieve admission.
//! Uses an exact integer consumer with no floating point, no exponential or
//! squashing form, and no dense matrix product (CHARTER §3.1–§3.3, D11, D23, D30).

use crate::{
    bindings::{Bindings, PinnedTokenizer},
    reserve, Error, Limits, Manifest, ObjectVersion, RecordBackend, Result, Store,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, ops::Range};

/// Selects whether an exact read targets the current or the previous distinct version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionSelect {
    Current,
    Previous,
}

/// Canonical prime-factor address ($p_1 < p_2 < \dots < p_k$) with optional
/// ordered neighbour pairs ($(p_a, p_b)$) that preserve relation direction.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PrimeAddress {
    primes: Vec<u32>,
    ordered_pairs: Vec<(u32, u32)>,
}

impl PrimeAddress {
    /// Construct a square-free canonical prime address from one or more prime atoms.
    /// Rejects empty inputs, `0`, `1`, or composite integers with `Error::SchemaMismatch`.
    pub fn new(primes: &[u32]) -> Result<Self> {
        Self::with_pairs(primes, &[])
    }

    /// Construct a square-free canonical prime address together with ordered prime
    /// pairs `(p_a, p_b)` whose endpoints must belong to the prime set.
    pub fn with_pairs(primes: &[u32], pairs: &[(u32, u32)]) -> Result<Self> {
        if primes.is_empty() {
            return Err(Error::SchemaMismatch);
        }
        let mut canonical = Vec::new();
        reserve(&mut canonical, primes.len())?;
        for &p in primes {
            if !is_prime_u32(p) {
                return Err(Error::SchemaMismatch);
            }
            canonical.push(p);
        }
        canonical.sort_unstable();
        canonical.dedup();

        let mut canonical_pairs = Vec::new();
        reserve(&mut canonical_pairs, pairs.len())?;
        for &(a, b) in pairs {
            if canonical.binary_search(&a).is_err() || canonical.binary_search(&b).is_err() {
                return Err(Error::SchemaMismatch);
            }
            canonical_pairs.push((a, b));
        }
        canonical_pairs.sort_unstable();
        canonical_pairs.dedup();

        Ok(Self {
            primes: canonical,
            ordered_pairs: canonical_pairs,
        })
    }

    /// Canonical ascending list of distinct prime factors.
    pub fn primes(&self) -> &[u32] {
        &self.primes
    }

    /// Canonical sorted list of ordered prime pairs `(p_a, p_b)`.
    pub fn ordered_pairs(&self) -> &[(u32, u32)] {
        &self.ordered_pairs
    }

    /// Exact semiprime product $k = p_a p_b$ when the address has exactly two prime factors.
    pub fn semiprime(&self) -> Option<u64> {
        if self.primes.len() == 2 {
            Some(u64::from(self.primes[0]) * u64::from(self.primes[1]))
        } else {
            None
        }
    }

    /// Count shared prime factors between `self` and `other` via a division-free
    /// two-pointer sorted merge ($\gcd(q, A_c) > 1 \iff \text{shared} \ge 1$).
    pub fn gcd_shared_primes(&self, other: &Self) -> usize {
        let mut i = 0usize;
        let mut j = 0usize;
        let mut shared = 0usize;
        while i < self.primes.len() && j < other.primes.len() {
            let a = self.primes[i];
            let b = other.primes[j];
            if a == b {
                shared += 1;
                i += 1;
                j += 1;
            } else if a < b {
                i += 1;
            } else {
                j += 1;
            }
        }
        shared
    }

    /// Count matching ordered prime pairs `(p_a, p_b)` via a two-pointer sorted merge.
    pub fn ordered_pair_overlap(&self, other: &Self) -> usize {
        let mut i = 0usize;
        let mut j = 0usize;
        let mut matches = 0usize;
        while i < self.ordered_pairs.len() && j < other.ordered_pairs.len() {
            let a = self.ordered_pairs[i];
            let b = other.ordered_pairs[j];
            if a == b {
                matches += 1;
                i += 1;
                j += 1;
            } else if a < b {
                i += 1;
            } else {
                j += 1;
            }
        }
        matches
    }
}

/// Stored object-store target bound to a [`PrimeAddress`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectTarget {
    /// Verified byte slice `[range.start..range.end]` inside a pinned `Manifest`.
    ByteSpan {
        manifest: Manifest,
        range: Range<u64>,
    },
    /// Whole object verified against the `Manifest`'s full SHA-256 digest.
    WholeObject { manifest: Manifest },
    /// Single token occurrence verified via [`Bindings::read_occurrence`].
    TokenOccurrence {
        manifest: Manifest,
        bindings: Bindings,
        index: u64,
    },
}

/// Typed result of resolving an address against [`AddressDirectory`] and [`Store`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddressReadOutcome {
    /// Active current slot resolved and verified against the object store.
    Hit {
        address: PrimeAddress,
        version: u64,
        sequence: u64,
        object_version: ObjectVersion,
        bytes: Vec<u8>,
        whole_object_verified: bool,
    },
    /// Active slot's previous distinct version resolved and verified against the object store.
    Previous {
        address: PrimeAddress,
        version: u64,
        sequence: u64,
        object_version: ObjectVersion,
        bytes: Vec<u8>,
        whole_object_verified: bool,
    },
    /// Address was never written (or has no previous version); zero backend reads performed.
    Absent,
    /// Address was written earlier and later evicted when the bounded directory reached capacity.
    Evicted {
        address: PrimeAddress,
        evicted_at_sequence: u64,
    },
}

#[derive(Debug, Clone)]
struct VersionedEntry {
    target: ObjectTarget,
    version: u64,
    sequence: u64,
}

#[derive(Debug, Clone)]
struct SlotRecord {
    current: VersionedEntry,
    previous: Option<VersionedEntry>,
}

/// Exact candidate counts for one sieve query, before any backend fetch.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SieveCandidateCounts {
    pub active_examined: usize,
    pub active_admitted: usize,
    pub evicted_examined: usize,
    pub evicted_admitted: usize,
}

/// Bounded versioned directory mapping [`PrimeAddress`] keys to [`ObjectTarget`]s.
#[derive(Debug, Clone)]
pub struct AddressDirectory {
    max_slots: usize,
    next_sequence: u64,
    slots: BTreeMap<PrimeAddress, SlotRecord>,
    evicted: BTreeMap<PrimeAddress, u64>,
}

impl AddressDirectory {
    /// Create a bounded directory holding at most `max_slots` active addresses.
    pub fn new(max_slots: usize) -> Result<Self> {
        if max_slots == 0 {
            return Err(Error::SchemaMismatch);
        }
        Ok(Self {
            max_slots,
            next_sequence: 1,
            slots: BTreeMap::new(),
            evicted: BTreeMap::new(),
        })
    }

    /// Number of currently active slots.
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    /// Whether the directory has no active slots.
    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    /// Bind `address` to `target`.
    /// - Reasserting the identical target refreshes `sequence` without overwriting `previous`
    ///   or incrementing `version`.
    /// - Writing a new distinct target increments `version` and moves the prior `current`
    ///   entry to `previous`.
    /// - Inserting a new address when `slots.len() == max_slots` evicts the oldest active
    ///   slot by `sequence` and records its eviction sequence.
    pub fn write(&mut self, address: PrimeAddress, target: ObjectTarget) -> Result<u64> {
        let seq = self.next_sequence;
        self.next_sequence = self
            .next_sequence
            .checked_add(1)
            .ok_or(Error::LimitExceeded("address sequence"))?;

        self.evicted.remove(&address);

        if let Some(slot) = self.slots.get_mut(&address) {
            if slot.current.target == target {
                slot.current.sequence = seq;
                return Ok(slot.current.version);
            }
            let new_version = slot
                .current
                .version
                .checked_add(1)
                .ok_or(Error::LimitExceeded("slot version"))?;
            let old_current = slot.current.clone();
            slot.previous = Some(old_current);
            slot.current = VersionedEntry {
                target,
                version: new_version,
                sequence: seq,
            };
            return Ok(new_version);
        }

        if self.slots.len() >= self.max_slots {
            if let Some(oldest_addr) = self
                .slots
                .iter()
                .min_by_key(|(_, rec)| rec.current.sequence)
                .map(|(k, _)| k.clone())
            {
                self.slots.remove(&oldest_addr);
                self.evicted.insert(oldest_addr, seq);
            }
        }

        self.slots.insert(
            address,
            SlotRecord {
                current: VersionedEntry {
                    target,
                    version: 1,
                    sequence: seq,
                },
                previous: None,
            },
        );
        Ok(1)
    }

    /// Resolve `query` by exact address match against `store`.
    pub fn read_exact(
        &self,
        store: &Store,
        tokenizer: Option<&PinnedTokenizer>,
        query: &PrimeAddress,
        select: VersionSelect,
    ) -> Result<AddressReadOutcome> {
        self.read_exact_from(store, store.limits(), tokenizer, query, select)
    }

    /// Resolve `query` by exact address match against any [`RecordBackend`].
    /// Performs zero backend fetches when the address is `Absent` or `Evicted`.
    pub fn read_exact_from<B: RecordBackend>(
        &self,
        backend: &B,
        limits: &Limits,
        tokenizer: Option<&PinnedTokenizer>,
        query: &PrimeAddress,
        select: VersionSelect,
    ) -> Result<AddressReadOutcome> {
        if let Some(slot) = self.slots.get(query) {
            let entry = match select {
                VersionSelect::Current => &slot.current,
                VersionSelect::Previous => match &slot.previous {
                    Some(prev) => prev,
                    None => return Ok(AddressReadOutcome::Absent),
                },
            };
            let (bytes, object_version, whole_object_verified) =
                resolve_target(backend, limits, tokenizer, &entry.target)?;
            return Ok(match select {
                VersionSelect::Current => AddressReadOutcome::Hit {
                    address: query.clone(),
                    version: entry.version,
                    sequence: entry.sequence,
                    object_version,
                    bytes,
                    whole_object_verified,
                },
                VersionSelect::Previous => AddressReadOutcome::Previous {
                    address: query.clone(),
                    version: entry.version,
                    sequence: entry.sequence,
                    object_version,
                    bytes,
                    whole_object_verified,
                },
            });
        }
        if let Some(&evicted_at_sequence) = self.evicted.get(query) {
            return Ok(AddressReadOutcome::Evicted {
                address: query.clone(),
                evicted_at_sequence,
            });
        }
        Ok(AddressReadOutcome::Absent)
    }

    /// Resolve `query` by sorted-merge GCD sieve ($\gcd(q, A_c) > 1$) against `store`.
    /// Admits active slots sharing at least one prime factor and ranks them
    /// deterministically by `(shared_primes, ordered_pair_matches, sequence)`.
    pub fn read_sieve(
        &self,
        store: &Store,
        tokenizer: Option<&PinnedTokenizer>,
        query: &PrimeAddress,
    ) -> Result<AddressReadOutcome> {
        self.read_sieve_from(store, store.limits(), tokenizer, query)
    }

    /// Resolve `query` by sorted-merge GCD sieve against any [`RecordBackend`].
    pub fn read_sieve_from<B: RecordBackend>(
        &self,
        backend: &B,
        limits: &Limits,
        tokenizer: Option<&PinnedTokenizer>,
        query: &PrimeAddress,
    ) -> Result<AddressReadOutcome> {
        let mut best: Option<((usize, usize, u64), &PrimeAddress, &VersionedEntry)> = None;
        for (addr, slot) in &self.slots {
            let shared = query.gcd_shared_primes(addr);
            if shared == 0 {
                continue;
            }
            let pairs = query.ordered_pair_overlap(addr);
            let rank = (shared, pairs, slot.current.sequence);
            if best.is_none_or(|(best_rank, _, _)| rank > best_rank) {
                best = Some((rank, addr, &slot.current));
            }
        }
        if let Some((_, addr, entry)) = best {
            let (bytes, object_version, whole_object_verified) =
                resolve_target(backend, limits, tokenizer, &entry.target)?;
            return Ok(AddressReadOutcome::Hit {
                address: addr.clone(),
                version: entry.version,
                sequence: entry.sequence,
                object_version,
                bytes,
                whole_object_verified,
            });
        }
        let mut best_evicted: Option<((usize, usize, u64), &PrimeAddress, u64)> = None;
        for (addr, &evicted_seq) in &self.evicted {
            let shared = query.gcd_shared_primes(addr);
            if shared == 0 {
                continue;
            }
            let pairs = query.ordered_pair_overlap(addr);
            let rank = (shared, pairs, evicted_seq);
            if best_evicted.is_none_or(|(best_rank, _, _)| rank > best_rank) {
                best_evicted = Some((rank, addr, evicted_seq));
            }
        }
        if let Some((_, addr, evicted_at_sequence)) = best_evicted {
            return Ok(AddressReadOutcome::Evicted {
                address: addr.clone(),
                evicted_at_sequence,
            });
        }
        Ok(AddressReadOutcome::Absent)
    }

    /// Count active and evicted candidates examined and admitted by the sieve.
    pub fn sieve_candidate_counts(&self, query: &PrimeAddress) -> SieveCandidateCounts {
        let mut counts = SieveCandidateCounts::default();
        for address in self.slots.keys() {
            counts.active_examined += 1;
            if query.gcd_shared_primes(address) > 0 {
                counts.active_admitted += 1;
            }
        }
        if counts.active_admitted == 0 {
            for address in self.evicted.keys() {
                counts.evicted_examined += 1;
                if query.gcd_shared_primes(address) > 0 {
                    counts.evicted_admitted += 1;
                }
            }
        }
        counts
    }
}

fn resolve_target<B: RecordBackend>(
    backend: &B,
    limits: &Limits,
    tokenizer: Option<&PinnedTokenizer>,
    target: &ObjectTarget,
) -> Result<(Vec<u8>, ObjectVersion, bool)> {
    match target {
        ObjectTarget::ByteSpan { manifest, range } => {
            let verified = Store::range_from(backend, manifest, range.clone(), limits)?;
            let version = verified.version();
            Ok((verified.bytes().to_vec(), version, false))
        }
        ObjectTarget::WholeObject { manifest } => {
            let verified = Store::range_from(backend, manifest, 0..manifest.byte_len(), limits)?;
            let digest: [u8; 32] = Sha256::digest(verified.bytes()).into();
            if digest != manifest.whole_sha256() {
                return Err(Error::IntegrityFailure);
            }
            let version = verified.version();
            Ok((verified.bytes().to_vec(), version, true))
        }
        ObjectTarget::TokenOccurrence {
            manifest,
            bindings,
            index,
        } => {
            let tok = tokenizer.ok_or(Error::BindingMismatch)?;
            if bindings.object_version() != manifest.version()
                || bindings.tokenizer_identity() != tok.identity()
                || manifest.byte_len()
                    != bindings
                        .occurrences()
                        .last()
                        .map_or(0, |occ| occ.byte_range().end)
            {
                return Err(Error::BindingMismatch);
            }
            let idx = usize::try_from(*index).map_err(|_| Error::InvalidRange)?;
            let occ = bindings.occurrences().get(idx).ok_or(Error::InvalidRange)?;
            let verified = Store::range_from(backend, manifest, occ.byte_range(), limits)?;
            let tok_bytes = tok.token_bytes(occ.token_id(), limits.max_returned_bytes)?;
            if verified.bytes() != tok_bytes.as_slice() {
                return Err(Error::BindingMismatch);
            }
            let version = verified.version();
            Ok((verified.bytes().to_vec(), version, false))
        }
    }
}

/// Deterministic integer primality test without floating point.
fn is_prime_u32(n: u32) -> bool {
    if n < 2 {
        return false;
    }
    if n == 2 || n == 3 {
        return true;
    }
    if n.is_multiple_of(2) || n.is_multiple_of(3) {
        return false;
    }
    let mut d = 5u64;
    let limit = u64::from(n);
    while d * d <= limit {
        if limit.is_multiple_of(d) || limit.is_multiple_of(d + 2) {
            return false;
        }
        d += 6;
    }
    true
}
