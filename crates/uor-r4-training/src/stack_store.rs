//! The product store bridge for the geometric stack (I1): a token-level API
//! over the scoped-memory contract's exact versioned store, [`Memory`]
//! (`uor_r4_core::native_geometric::learner::scoped_memory`).
//!
//! The semantics are that module's, reused and not reimplemented. Version
//! order, the predecessor chain, pinned views, conflict marking, same-value
//! reassertion, capacity eviction, lineage and serialization all run through
//! `Memory` itself. This module adds only three things:
//! - the token encoding of addresses and values;
//! - a fixed status vocabulary for re-entry into the model;
//! - load-time checks of that encoding.
//!
//! # The six typed distinctions (ROADMAP §2b, decided 2026-09-28)
//!
//! | Distinction | Request or outcome | `Memory` semantics |
//! |---|---|---|
//! | current | `HistoryView::Current` | the head of the chain at the pinned view |
//! | previous record | `HistoryView::PreviousAssertion` | the immediate predecessor, including a same-value reassertion |
//! | previous distinct value | `HistoryView::PreviousDistinctValue` | the nearest earlier record whose value differs from the head's |
//! | initial occurrence | `HistoryView::Initial` | the earliest record of the chain |
//! | absent | [`StoreRead::Absent`] | no record at the address is visible at the pinned view |
//! | evicted | [`StoreRead::Evicted`] | the required record exists, but declared capacity released its tokens |
//!
//! A seventh outcome, [`StoreRead::NoHistory`], says that the address exists
//! but the requested position does not; a one-record chain, for example, has
//! no previous record. Eviction is never reported as absence.
//!
//! Writes take `Update::Assert` or `Update::Correct`, with `Memory`'s rules:
//! - An assertion of the current value is a **same-value reassertion**. It
//!   appends a record, so history is never deduplicated.
//! - A bare assertion of a different value becomes current and is **marked
//!   `conflict`**.
//! - A correction supersedes and is never marked `conflict`. A correction
//!   that restates the current value still appends a revision.
//!
//! # Mapping onto `Memory`, and what does not map one to one
//!
//! 1. **Relation width.** `Memory` addresses `(scope bytes, entity bytes,
//!    relation byte)`. The stack's relation id is token-width (`u32`): M1's
//!    pointer head and D2's probe both address a relation by a token id,
//!    which one byte cannot hold. The store therefore carries the entity
//!    tokens and then the relation id as little-endian `u32` words inside
//!    `Memory`'s entity field, and writes the fixed [`TOKEN_RECORD_TAG`] into
//!    the relation byte. `encode_key` length-delimits the scope and the
//!    entity field, and the words have fixed width, so the key is injective in
//!    `(scope, entity token ids, relation id)`. `Memory` uses these fields only
//!    for key encoding, emptiness checks and its history digest, so none of
//!    its semantics change.
//!    A raw `Record`, however, shows the relation inside `entity` and the tag
//!    in `relation`; [`StackStore::addresses`] decodes them.
//! 2. **Value identity and owned tokens.** `Memory` keeps a record's exact
//!    lexical identity (`value`, used for equality) apart from its owned
//!    emitted tokens (`payload`). The store sets the payload to the value
//!    tokens and the identity to their little-endian bytes. Same-value
//!    reassertion, conflict and previous-distinct are therefore exact
//!    token-sequence equality. At capacity `Memory` releases the payload but
//!    keeps the identity bytes in a tombstone, as it declares ("metadata and
//!    lexical values remain available as tombstones"). A released record's
//!    tokens thus remain in its `value` field. A read never returns them; it
//!    reports `Evicted`. This is a storage fact, not an erasure guarantee.
//! 3. **Single-address reads.** Every record is written with
//!    `continues = false` and `parse_score = 0`. Dependent multi-hop reads and
//!    the parse-confidence control belong to the language runtime
//!    (`ScopedMemoryRuntime`) and are not exposed here. Load rejects a record
//!    that sets them. `source` is 0 unless [`StackStore::write_from`] supplies
//!    one, for example the token position of a write.
//! 4. **Capacity.** As `Memory` declares, capacity bounds the records whose
//!    tokens each `(scope, entity, relation)` chain retains. The number of
//!    addresses is not bounded, and tombstones persist.
//! 5. **Previous distinct keeps `Memory`'s eviction barrier.** A
//!    previous-distinct read does not traverse a released predecessor, even
//!    one whose value equals the head's; it reports `Evicted`.
//! 6. **Serialization.** [`StackStore::to_bytes`] is `Memory`'s own JSON
//!    (store version 2). [`StackStore::from_bytes`] decodes and validates it
//!    with `Memory::from_bytes`. It then rejects a foreign lineage and any
//!    record that is not in this token encoding.
//!
//! # Scope
//!
//! Exact host-side infrastructure. Reads and writes are binary search, byte
//! comparison and copies, with no floating point. A lookup allocates, and
//! persistence is JSON, so this is not a steady-state serving kernel and is
//! not audited against D11. It is not a learned read or write. Nothing here
//! wires the store into the stack's forward pass (M1, G).

use std::fmt;

use serde::{Deserialize, Serialize};
use uor_r4_core::native_geometric::learner::realtext_support::sha256_hex;
use uor_r4_core::native_geometric::learner::scoped_memory::{
    encode_key, Lookup, Memory, Record, ScopedMemoryError,
};

pub use uor_r4_core::native_geometric::learner::scoped_memory::{HistoryView, Update, Written};

/// The byte every stack record carries in `Memory`'s one-byte relation field.
/// The relation id itself travels in the entity field (module docs, mapping
/// 1); load checks the tag.
pub const TOKEN_RECORD_TAG: u8 = b'T';

/// Bytes per token id in the store's encodings: little-endian `u32`.
const TOKEN_BYTES: usize = 4;

/// Statuses a read re-enters the model with.
pub const STATUS_COUNT: usize = 4;
const VIEW_BITS: u32 = 2;
/// Requestable views.
pub const VIEW_COUNT: usize = 1 << VIEW_BITS;
/// Symbols of the status-and-view vocabulary ([`StoreSignal`]).
pub const SIGNAL_COUNT: usize = STATUS_COUNT << VIEW_BITS;

/// The requestable views, in code order.
pub const VIEWS: [HistoryView; VIEW_COUNT] = [
    HistoryView::Current,
    HistoryView::PreviousAssertion,
    HistoryView::PreviousDistinctValue,
    HistoryView::Initial,
];

/// The fixed status vocabulary a read re-enters the model with.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum StoreStatus {
    /// A record was selected, and its tokens are owned.
    Found,
    /// No record at the address is visible at the pinned view.
    Absent,
    /// The address exists, but the requested historical position does not.
    NoHistory,
    /// The required record exists, but declared capacity released its tokens.
    Evicted,
}

impl StoreStatus {
    /// Every status, in code order.
    pub const ALL: [Self; STATUS_COUNT] =
        [Self::Found, Self::Absent, Self::NoHistory, Self::Evicted];

    /// The fixed code, in `0..STATUS_COUNT`.
    pub const fn code(self) -> u32 {
        match self {
            Self::Found => 0,
            Self::Absent => 1,
            Self::NoHistory => 2,
            Self::Evicted => 3,
        }
    }

    pub const fn from_code(code: u32) -> Option<Self> {
        match code {
            0 => Some(Self::Found),
            1 => Some(Self::Absent),
            2 => Some(Self::NoHistory),
            3 => Some(Self::Evicted),
            _ => None,
        }
    }
}

/// The fixed code of a requested view, in `0..VIEW_COUNT` (the order of
/// [`VIEWS`]).
pub const fn view_code(view: HistoryView) -> u32 {
    match view {
        HistoryView::Current => 0,
        HistoryView::PreviousAssertion => 1,
        HistoryView::PreviousDistinctValue => 2,
        HistoryView::Initial => 3,
    }
}

pub const fn view_from_code(code: u32) -> Option<HistoryView> {
    match code {
        0 => Some(HistoryView::Current),
        1 => Some(HistoryView::PreviousAssertion),
        2 => Some(HistoryView::PreviousDistinctValue),
        3 => Some(HistoryView::Initial),
        _ => None,
    }
}

/// A read's status with the view it answered. The pair is one symbol of a
/// fixed `SIGNAL_COUNT`-symbol vocabulary, status-major, for an embedding
/// table. Codes use shifts and masks only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoreSignal {
    pub status: StoreStatus,
    pub view: HistoryView,
}

impl StoreSignal {
    /// The fixed code, in `0..SIGNAL_COUNT`.
    pub const fn code(self) -> u32 {
        (self.status.code() << VIEW_BITS) | view_code(self.view)
    }

    pub const fn from_code(code: u32) -> Option<Self> {
        match (
            StoreStatus::from_code(code >> VIEW_BITS),
            view_from_code(code & (VIEW_COUNT as u32 - 1)),
        ) {
            (Some(status), Some(view)) => Some(Self { status, view }),
            _ => None,
        }
    }
}

/// The record a successful read selected.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoreValue {
    /// The owned value tokens.
    pub tokens: Vec<u32>,
    /// The record's exact id.
    pub record: u64,
    /// The commit that wrote the record.
    pub commit: u64,
    /// How the record was written.
    pub update: Update,
    /// A bare assertion contradicted the then-current value.
    pub conflict: bool,
}

/// The typed outcome of one read.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StoreRead {
    Found(StoreValue),
    /// No record at the address is visible at the pinned view.
    Absent,
    /// The address exists, but the requested historical position does not.
    NoHistory,
    /// The required record exists, but declared capacity released its tokens.
    Evicted,
}

impl StoreRead {
    pub fn status(&self) -> StoreStatus {
        match self {
            Self::Found(_) => StoreStatus::Found,
            Self::Absent => StoreStatus::Absent,
            Self::NoHistory => StoreStatus::NoHistory,
            Self::Evicted => StoreStatus::Evicted,
        }
    }

    /// The re-entry symbol of this outcome as the answer to `view`.
    pub fn signal(&self, view: HistoryView) -> StoreSignal {
        StoreSignal {
            status: self.status(),
            view,
        }
    }

    pub fn value(&self) -> Option<&StoreValue> {
        match self {
            Self::Found(value) => Some(value),
            _ => None,
        }
    }
}

/// One address of the store, decoded from its records.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct StoreAddress {
    pub scope: Vec<u8>,
    pub entity: Vec<u32>,
    pub relation: u32,
}

/// Classified failures at the stack store's boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StackStoreError {
    /// The scoped-memory store refused a write, or failed to encode, decode or
    /// validate.
    Memory(ScopedMemoryError),
    /// A scope must be at least one byte.
    EmptyScope,
    /// An entity must be at least one token.
    EmptyEntity,
    /// A value must be at least one token.
    EmptyValue,
    /// Declared chain capacity must be positive.
    ZeroCapacity,
    /// A pinned view later than the committed history.
    ViewAfterCommit { view: u64, commit: u64 },
    /// Loaded bytes belong to another store lineage (session).
    ForeignLineage { expected: u64, found: u64 },
    /// A loaded record is not in the stack store's token encoding.
    Encoding { record: u64, reason: &'static str },
    /// A stored token id is outside the model's vocabulary.
    TokenOutOfVocabulary {
        record: u64,
        token: u32,
        vocab_size: usize,
    },
}

impl fmt::Display for StackStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Memory(error) => write!(f, "stack store: {error}"),
            Self::EmptyScope => write!(f, "stack store: a scope must be at least one byte"),
            Self::EmptyEntity => write!(f, "stack store: an entity must be at least one token"),
            Self::EmptyValue => write!(f, "stack store: a value must be at least one token"),
            Self::ZeroCapacity => write!(f, "stack store: chain capacity must be positive"),
            Self::ViewAfterCommit { view, commit } => write!(
                f,
                "stack store: view {view} is later than the committed history ({commit})"
            ),
            Self::ForeignLineage { expected, found } => write!(
                f,
                "stack store: lineage {found} is foreign to the expected lineage {expected}"
            ),
            Self::Encoding { record, reason } => {
                write!(f, "stack store: record {record} is not token-encoded: {reason}")
            }
            Self::TokenOutOfVocabulary {
                record,
                token,
                vocab_size,
            } => write!(
                f,
                "stack store: record {record} holds token {token}, outside the vocabulary of {vocab_size}"
            ),
        }
    }
}

impl std::error::Error for StackStoreError {}

impl From<ScopedMemoryError> for StackStoreError {
    fn from(error: ScopedMemoryError) -> Self {
        Self::Memory(error)
    }
}

/// The stack's session store: [`Memory`] addressed and valued by token ids.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StackStore {
    memory: Memory,
}

impl StackStore {
    /// An empty store. `lineage` identifies the session across save and
    /// reload. `capacity` is the declared number of records whose tokens each
    /// `(scope, entity, relation)` chain retains.
    pub fn new(lineage: u64, capacity: usize) -> Result<Self, StackStoreError> {
        if capacity == 0 {
            return Err(StackStoreError::ZeroCapacity);
        }
        Ok(Self {
            memory: Memory::new(lineage, capacity),
        })
    }

    pub fn lineage(&self) -> u64 {
        self.memory.lineage
    }

    pub fn capacity(&self) -> usize {
        self.memory.capacity
    }

    /// The latest commit. A read pinned here cannot see later writes.
    pub fn commit(&self) -> u64 {
        self.memory.commit
    }

    /// Records written, released ones included.
    pub fn records(&self) -> usize {
        self.memory.records.len()
    }

    /// The underlying scoped-memory store, read-only.
    pub fn memory(&self) -> &Memory {
        &self.memory
    }

    /// The `Memory` key of an address (module docs, mapping 1).
    pub fn key(scope: &[u8], entity: &[u32], relation: u32) -> Result<Vec<u8>, StackStoreError> {
        check_address(scope, entity)?;
        Ok(encode_key(
            scope,
            &entity_field(entity, relation),
            TOKEN_RECORD_TAG,
        ))
    }

    /// Commit `value` at `(scope, entity, relation)` with source 0.
    pub fn write(
        &mut self,
        scope: &[u8],
        entity: &[u32],
        relation: u32,
        value: &[u32],
        update: Update,
    ) -> Result<Written, StackStoreError> {
        self.write_from(scope, entity, relation, value, update, 0)
    }

    /// [`write`](Self::write), recording `source`, for example the token
    /// position of the write, in the record.
    pub fn write_from(
        &mut self,
        scope: &[u8],
        entity: &[u32],
        relation: u32,
        value: &[u32],
        update: Update,
        source: u64,
    ) -> Result<Written, StackStoreError> {
        check_address(scope, entity)?;
        if value.is_empty() {
            return Err(StackStoreError::EmptyValue);
        }
        Ok(self.memory.write(
            scope,
            &entity_field(entity, relation),
            TOKEN_RECORD_TAG,
            &token_bytes(value),
            value,
            source,
            update,
            false,
            0,
        )?)
    }

    /// Read `view` of an address at the latest commit.
    pub fn read(
        &self,
        scope: &[u8],
        entity: &[u32],
        relation: u32,
        view: HistoryView,
    ) -> Result<StoreRead, StackStoreError> {
        self.read_at(scope, entity, relation, view, self.memory.commit)
    }

    /// Read `view` of an address at the pinned commit `pinned`, through
    /// `Memory::lookup`. Records committed after the pin are invisible. A
    /// record visible at the pin but released since reads as `Evicted`.
    pub fn read_at(
        &self,
        scope: &[u8],
        entity: &[u32],
        relation: u32,
        view: HistoryView,
        pinned: u64,
    ) -> Result<StoreRead, StackStoreError> {
        if pinned > self.memory.commit {
            return Err(StackStoreError::ViewAfterCommit {
                view: pinned,
                commit: self.memory.commit,
            });
        }
        let key = Self::key(scope, entity, relation)?;
        Ok(match self.memory.lookup(&key, pinned, view) {
            Lookup::Found(record) => StoreRead::Found(StoreValue {
                tokens: record.payload.clone(),
                record: record.id,
                commit: record.commit,
                update: record.action,
                conflict: record.conflict,
            }),
            Lookup::Absent => StoreRead::Absent,
            Lookup::NoHistory => StoreRead::NoHistory,
            Lookup::Evicted => StoreRead::Evicted,
        })
    }

    /// Every address with at least one record, in `Memory`'s key order.
    pub fn addresses(&self) -> Result<Vec<StoreAddress>, StackStoreError> {
        self.memory
            .chains
            .iter()
            .map(|(_, ids)| {
                let record = ids
                    .first()
                    .and_then(|id| self.memory.record_ref(*id))
                    .ok_or(StackStoreError::Encoding {
                        record: 0,
                        reason: "a chain has no record",
                    })?;
                let (entity, relation) = decode_entity_field(record)?;
                Ok(StoreAddress {
                    scope: record.scope.clone(),
                    entity,
                    relation,
                })
            })
            .collect()
    }

    /// `Memory`'s immutable-history digest at the latest commit.
    pub fn history_sha256(&self) -> Result<String, StackStoreError> {
        Ok(self.memory.history_sha256(self.memory.commit)?)
    }

    /// Check that every entity and value token is below `vocab_size`. This
    /// includes the value identity a released record keeps. The relation id
    /// is an opaque id and is not checked.
    pub fn check_vocabulary(&self, vocab_size: usize) -> Result<(), StackStoreError> {
        for record in &self.memory.records {
            let (entity, _) = decode_entity_field(record)?;
            let value = decode_tokens(&record.value).ok_or(StackStoreError::Encoding {
                record: record.id,
                reason: "the value is not whole token ids",
            })?;
            if let Some(token) = entity
                .into_iter()
                .chain(value)
                .find(|token| *token as usize >= vocab_size)
            {
                return Err(StackStoreError::TokenOutOfVocabulary {
                    record: record.id,
                    token,
                    vocab_size,
                });
            }
        }
        Ok(())
    }

    /// `Memory`'s own serialization.
    pub fn to_bytes(&self) -> Result<Vec<u8>, StackStoreError> {
        Ok(self.memory.to_bytes()?)
    }

    /// Decode and validate with `Memory::from_bytes`, then reject a lineage
    /// other than `lineage` and any record not in this token encoding.
    pub fn from_bytes(bytes: &[u8], lineage: u64) -> Result<Self, StackStoreError> {
        let memory = Memory::from_bytes(bytes)?;
        if memory.lineage != lineage {
            return Err(StackStoreError::ForeignLineage {
                expected: lineage,
                found: memory.lineage,
            });
        }
        for record in &memory.records {
            check_record(record)?;
        }
        Ok(Self { memory })
    }
}

fn check_address(scope: &[u8], entity: &[u32]) -> Result<(), StackStoreError> {
    if scope.is_empty() {
        return Err(StackStoreError::EmptyScope);
    }
    if entity.is_empty() {
        return Err(StackStoreError::EmptyEntity);
    }
    Ok(())
}

/// Little-endian bytes of `tokens`. The width is fixed, so the map is
/// injective.
fn token_bytes(tokens: &[u32]) -> Vec<u8> {
    tokens
        .iter()
        .flat_map(|token| token.to_le_bytes())
        .collect()
}

/// The tokens of a little-endian byte string whose length is a whole number
/// of tokens.
fn decode_tokens(bytes: &[u8]) -> Option<Vec<u32>> {
    let chunks = bytes.chunks_exact(TOKEN_BYTES);
    if !chunks.remainder().is_empty() {
        return None;
    }
    chunks
        .map(|chunk| chunk.try_into().ok().map(u32::from_le_bytes))
        .collect()
}

/// `Memory`'s entity field for an address: the entity tokens, then the
/// relation id.
fn entity_field(entity: &[u32], relation: u32) -> Vec<u8> {
    let mut field = token_bytes(entity);
    field.extend_from_slice(&relation.to_le_bytes());
    field
}

fn decode_entity_field(record: &Record) -> Result<(Vec<u32>, u32), StackStoreError> {
    let bad = |reason: &'static str| StackStoreError::Encoding {
        record: record.id,
        reason,
    };
    let mut words =
        decode_tokens(&record.entity).ok_or_else(|| bad("the entity field is not whole words"))?;
    let relation = words
        .pop()
        .ok_or_else(|| bad("the entity field has no relation id"))?;
    if words.is_empty() {
        return Err(bad("the entity field has no entity token"));
    }
    Ok((words, relation))
}

/// The token-encoding invariants `write_from` establishes, which
/// `Memory::validate` does not know.
fn check_record(record: &Record) -> Result<(), StackStoreError> {
    let bad = |reason: &'static str| StackStoreError::Encoding {
        record: record.id,
        reason,
    };
    if record.relation != TOKEN_RECORD_TAG {
        return Err(bad("the relation byte is not the token-record tag"));
    }
    decode_entity_field(record)?;
    let value = decode_tokens(&record.value).ok_or_else(|| bad("the value is not whole tokens"))?;
    if !record.evicted && record.payload != value {
        return Err(bad("the value identity differs from the owned tokens"));
    }
    // The witness binds a released record's identity to the tokens it owned.
    if record.payload_sha256 != sha256_hex(&record.value) {
        return Err(bad("the payload witness differs from the value identity"));
    }
    if record.continues || record.parse_score != 0 {
        return Err(bad("language-runtime continuation or parse score is set"));
    }
    Ok(())
}

/// A store that exercises every distinction across several scopes, entities
/// and relations. It holds a reassertion, conflicts, corrections, released
/// records and a single-record chain. Every token is below 32. Tests only.
#[cfg(test)]
pub(crate) fn test_store(lineage: u64) -> StackStore {
    let writes: [(&[u8], &[u32], u32, &[u32], Update); 9] = [
        (b"alice", &[3, 4], 1, &[10, 11], Update::Assert),
        // A bare contradiction: conflict.
        (b"alice", &[3, 4], 1, &[12], Update::Assert),
        // A same-value reassertion; capacity 2 releases the first record.
        (b"alice", &[3, 4], 1, &[12], Update::Assert),
        // The same entity under another relation.
        (b"alice", &[3, 4], 2, &[13], Update::Assert),
        // A single-record chain.
        (b"alice", &[5], 1, &[14, 15, 16], Update::Assert),
        // The same entity and relation in another scope.
        (b"bob", &[3, 4], 1, &[17], Update::Assert),
        (b"bob", &[3, 4], 1, &[18], Update::Correct),
        (b"bob", &[3, 4], 1, &[17], Update::Assert),
        // A correction that restates the current value.
        (b"alice", &[3, 4], 2, &[13], Update::Correct),
    ];
    let mut store = StackStore::new(lineage, 2).expect("test store");
    for (scope, entity, relation, value, update) in writes {
        store
            .write(scope, entity, relation, value, update)
            .expect("test store write");
    }
    store
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCOPE: &[u8] = b"session";
    const ENTITY: &[u32] = &[11, 12];
    const RELATION: u32 = 7;
    const A: &[u32] = &[21, 22];
    const B: &[u32] = &[31];
    const C: &[u32] = &[41, 42, 43];
    const D: &[u32] = &[51];
    const E: &[u32] = &[52];

    /// The expected outcome of one read of a written address: a value and its
    /// record id, or a typed missing position. Absence is checked directly
    /// against `StoreRead::Absent`.
    #[derive(Debug)]
    enum Want {
        Found(&'static [u32], u64),
        NoHistory,
        Evicted,
    }

    fn check(read: &StoreRead, want: &Want, context: &str) {
        match (read, want) {
            (StoreRead::Found(value), Want::Found(tokens, record)) => {
                assert_eq!(value.tokens.as_slice(), *tokens, "{context}");
                assert_eq!(value.record, *record, "{context}");
            }
            (StoreRead::NoHistory, Want::NoHistory) | (StoreRead::Evicted, Want::Evicted) => {}
            _ => panic!("{context}: read {read:?}, wanted {want:?}"),
        }
    }

    fn read(store: &StackStore, view: HistoryView) -> StoreRead {
        store.read(SCOPE, ENTITY, RELATION, view).expect("read")
    }

    fn write(store: &mut StackStore, value: &[u32], update: Update) -> Written {
        store
            .write(SCOPE, ENTITY, RELATION, value, update)
            .expect("write")
    }

    /// Every write history against every view: current, previous record,
    /// previous distinct and initial, with and without eviction. Each write
    /// also carries its expected conflict mark, and the absent neighbouring
    /// addresses are checked beside each history.
    #[test]
    fn views_answer_the_declared_distinction_for_every_history() {
        use Update::{Assert, Correct};
        use Want::{Evicted, Found, NoHistory};
        // (value, update, expected conflict mark)
        type History = Vec<(&'static [u32], Update, bool)>;
        // Wanted reads in VIEWS order: current, previous record, previous
        // distinct, initial.
        let cases: Vec<(&str, usize, History, [Want; VIEW_COUNT])> = vec![
            (
                "first assertion",
                8,
                vec![(A, Assert, false)],
                [Found(A, 1), NoHistory, NoHistory, Found(A, 1)],
            ),
            (
                "update to a new value",
                8,
                vec![(A, Assert, false), (B, Assert, true)],
                [Found(B, 2), Found(A, 1), Found(A, 1), Found(A, 1)],
            ),
            (
                "same-value reassertion",
                8,
                vec![(A, Assert, false), (A, Assert, false)],
                [Found(A, 2), Found(A, 1), NoHistory, Found(A, 1)],
            ),
            (
                "reassertion after an update",
                8,
                vec![(A, Assert, false), (B, Assert, true), (B, Assert, false)],
                [Found(B, 3), Found(B, 2), Found(A, 1), Found(A, 1)],
            ),
            (
                "reassertion after a correction",
                8,
                vec![(A, Assert, false), (B, Correct, false), (B, Assert, false)],
                [Found(B, 3), Found(B, 2), Found(A, 1), Found(A, 1)],
            ),
            (
                "correction restating the current value",
                8,
                vec![(A, Assert, false), (A, Correct, false)],
                [Found(A, 2), Found(A, 1), NoHistory, Found(A, 1)],
            ),
            (
                "initial after several updates",
                8,
                vec![
                    (A, Assert, false),
                    (B, Correct, false),
                    (C, Assert, true),
                    (A, Correct, false),
                ],
                [Found(A, 4), Found(C, 3), Found(C, 3), Found(A, 1)],
            ),
            (
                "capacity releases the initial record",
                2,
                vec![(A, Assert, false), (B, Assert, true), (C, Assert, true)],
                [Found(C, 3), Found(B, 2), Found(B, 2), Evicted],
            ),
            (
                "capacity releases every expired record",
                2,
                vec![
                    (A, Assert, false),
                    (B, Correct, false),
                    (C, Correct, false),
                    (D, Correct, false),
                    (E, Correct, false),
                ],
                [Found(E, 5), Found(D, 4), Found(D, 4), Evicted],
            ),
            (
                "released distinct value behind a reassertion",
                2,
                vec![(A, Assert, false), (B, Assert, true), (B, Assert, false)],
                [Found(B, 3), Found(B, 2), Evicted, Evicted],
            ),
            (
                "capacity one keeps only the head",
                1,
                vec![(A, Assert, false), (A, Assert, false)],
                [Found(A, 2), Evicted, Evicted, Evicted],
            ),
            (
                "capacity one still compares against the retained head",
                1,
                vec![(A, Assert, false), (B, Assert, true), (B, Assert, false)],
                [Found(B, 3), Evicted, Evicted, Evicted],
            ),
        ];
        for (name, capacity, history, wants) in cases {
            let mut store = StackStore::new(1, capacity).expect("store");
            for (position, (value, update, conflict)) in history.into_iter().enumerate() {
                let written = write(&mut store, value, update);
                assert_eq!(
                    (written.revision, written.conflict),
                    (position as u64 + 1, conflict),
                    "{name}: write {position}"
                );
            }
            for (view, want) in VIEWS.iter().zip(&wants) {
                let got = read(&store, *view);
                check(&got, want, &format!("{name}, {view:?}"));
                assert_ne!(
                    got,
                    StoreRead::Absent,
                    "{name}: a written key is never absent"
                );
            }
            // Neighbouring addresses are absent at every view.
            let neighbours: [(&[u8], &[u32], u32); 3] = [
                (b"other", ENTITY, RELATION),
                (SCOPE, &[11], RELATION),
                (SCOPE, ENTITY, RELATION + 1),
            ];
            for view in VIEWS {
                for (scope, entity, relation) in neighbours {
                    assert_eq!(
                        store.read(scope, entity, relation, view).expect("read"),
                        StoreRead::Absent,
                        "{name}: {view:?} of a neighbour"
                    );
                }
            }
        }
    }

    #[test]
    fn previous_record_and_previous_distinct_differ_after_a_reassertion() {
        let mut store = StackStore::new(2, 8).expect("store");
        for value in [A, B, B] {
            write(&mut store, value, Update::Assert);
        }
        let previous = read(&store, HistoryView::PreviousAssertion);
        let distinct = read(&store, HistoryView::PreviousDistinctValue);
        assert_ne!(previous, distinct);
        assert_eq!(
            previous.value().map(|v| (v.tokens.as_slice(), v.record)),
            Some((B, 2))
        );
        assert_eq!(
            distinct.value().map(|v| (v.tokens.as_slice(), v.record)),
            Some((A, 1))
        );
    }

    #[test]
    fn a_correction_and_a_bare_contradiction_differ_in_the_conflict_mark() {
        let mut store = StackStore::new(3, 8).expect("store");
        let first = write(&mut store, A, Update::Assert);
        assert_eq!(
            (first.revision, first.conflict, first.superseded),
            (1, false, 0)
        );
        let reassert = write(&mut store, A, Update::Assert);
        assert_eq!(
            (reassert.revision, reassert.conflict, reassert.superseded),
            (2, false, first.id)
        );
        let bare = write(&mut store, B, Update::Assert);
        assert_eq!((bare.revision, bare.conflict), (3, true));
        let current = read(&store, HistoryView::Current);
        assert_eq!(
            current
                .value()
                .map(|v| (v.tokens.as_slice(), v.update, v.conflict)),
            Some((B, Update::Assert, true))
        );
        let corrected = write(&mut store, C, Update::Correct);
        assert_eq!(
            (corrected.revision, corrected.conflict, corrected.superseded),
            (4, false, bare.id)
        );
        let current = read(&store, HistoryView::Current);
        assert_eq!(
            current
                .value()
                .map(|v| (v.tokens.as_slice(), v.update, v.conflict)),
            Some((C, Update::Correct, false))
        );
        // The superseded bare assertion keeps its conflict mark in history.
        let previous = read(&store, HistoryView::PreviousAssertion);
        assert_eq!(
            previous.value().map(|v| (v.record, v.conflict)),
            Some((bare.id, true))
        );
        // A first write that is a correction is not a conflict either.
        let mut fresh = StackStore::new(3, 8).expect("store");
        let written = write(&mut fresh, B, Update::Correct);
        assert_eq!((written.revision, written.conflict), (1, false));
    }

    #[test]
    fn absent_differs_from_no_history_and_pinned_views_hide_later_writes() {
        let mut store = StackStore::new(4, 8).expect("store");
        for view in VIEWS {
            assert_eq!(
                read(&store, view),
                StoreRead::Absent,
                "{view:?} before a write"
            );
        }
        write(&mut store, A, Update::Assert);
        let pin = store.commit();
        assert_eq!(
            read(&store, HistoryView::PreviousAssertion),
            StoreRead::NoHistory
        );
        assert_eq!(
            read(&store, HistoryView::PreviousDistinctValue),
            StoreRead::NoHistory
        );
        write(&mut store, B, Update::Correct);
        let at = |view, pinned| {
            store
                .read_at(SCOPE, ENTITY, RELATION, view, pinned)
                .expect("read")
        };
        check(
            &at(HistoryView::Current, pin),
            &Want::Found(A, 1),
            "pinned current",
        );
        check(
            &at(HistoryView::Current, store.commit()),
            &Want::Found(B, 2),
            "latest current",
        );
        assert_eq!(
            at(HistoryView::PreviousAssertion, pin),
            StoreRead::NoHistory
        );
        // Before the first commit nothing is visible at the address.
        assert_eq!(at(HistoryView::Current, 0), StoreRead::Absent);
        assert_eq!(
            store.read_at(SCOPE, ENTITY, RELATION, HistoryView::Current, pin + 2),
            Err(StackStoreError::ViewAfterCommit { view: 3, commit: 2 })
        );

        // A record visible at an earlier pin but released since reads as
        // Evicted, never as Absent.
        let mut small = StackStore::new(4, 2).expect("store");
        write(&mut small, A, Update::Assert);
        let early = small.commit();
        write(&mut small, B, Update::Correct);
        write(&mut small, C, Update::Correct);
        for view in [HistoryView::Current, HistoryView::Initial] {
            assert_eq!(
                small
                    .read_at(SCOPE, ENTITY, RELATION, view, early)
                    .expect("read"),
                StoreRead::Evicted,
                "{view:?} at the early pin"
            );
        }
        assert_eq!(
            small
                .read_at(SCOPE, ENTITY, RELATION, HistoryView::Current, 0)
                .expect("read"),
            StoreRead::Absent
        );
    }

    #[test]
    fn scopes_entities_and_relations_are_independent_injective_addresses() {
        let addresses: [(&[u8], &[u32], u32); 7] = [
            (b"alice", &[5], 1),
            (b"bob", &[5], 1),
            (b"alice", &[5, 6], 1),
            (b"alice", &[5], 6),
            (b"alice", &[6], 1),
            // Naive concatenation gives these two the same bytes:
            // "a", 05 00 00 00, 06 00 00 00, 01 00 00 00.
            (b"a\x05\x00\x00\x00", &[6], 1),
            (b"a", &[5, 6], 1),
        ];
        let keys: Vec<Vec<u8>> = addresses
            .iter()
            .map(|(scope, entity, relation)| {
                StackStore::key(scope, entity, *relation).expect("key")
            })
            .collect();
        for (i, a) in keys.iter().enumerate() {
            for b in &keys[i + 1..] {
                assert_ne!(a, b, "two addresses share a key");
            }
        }
        // Capacity 1: a second write releases only its own chain's record.
        let mut store = StackStore::new(5, 1).expect("store");
        for (index, (scope, entity, relation)) in addresses.iter().enumerate() {
            store
                .write(
                    scope,
                    entity,
                    *relation,
                    &[100 + index as u32],
                    Update::Assert,
                )
                .expect("write");
        }
        store
            .write(b"alice", &[5], 1, &[200], Update::Correct)
            .expect("write");
        for (index, (scope, entity, relation)) in addresses.iter().enumerate() {
            let current = store
                .read(scope, entity, *relation, HistoryView::Current)
                .expect("read");
            let initial = store
                .read(scope, entity, *relation, HistoryView::Initial)
                .expect("read");
            if index == 0 {
                assert_eq!(current.value().map(|v| v.tokens.clone()), Some(vec![200]));
                assert_eq!(initial, StoreRead::Evicted);
            } else {
                let own = Some(vec![100 + index as u32]);
                assert_eq!(current.value().map(|v| v.tokens.clone()), own);
                assert_eq!(initial.value().map(|v| v.tokens.clone()), own);
            }
        }
        let mut decoded = store.addresses().expect("addresses");
        decoded.sort();
        let mut declared: Vec<StoreAddress> = addresses
            .iter()
            .map(|(scope, entity, relation)| StoreAddress {
                scope: scope.to_vec(),
                entity: entity.to_vec(),
                relation: *relation,
            })
            .collect();
        declared.sort();
        assert_eq!(decoded, declared);
        // Full-width token and relation ids round-trip.
        let mut wide = StackStore::new(5, 2).expect("store");
        let entity = [u32::MAX, 0, 256];
        wide.write(b"s", &entity, u32::MAX, &[65_536, u32::MAX], Update::Assert)
            .expect("write");
        assert_eq!(
            wide.addresses().expect("addresses"),
            vec![StoreAddress {
                scope: b"s".to_vec(),
                entity: entity.to_vec(),
                relation: u32::MAX,
            }]
        );
        assert_eq!(
            wide.read(b"s", &entity, u32::MAX, HistoryView::Current)
                .expect("read")
                .value()
                .map(|v| v.tokens.clone()),
            Some(vec![65_536, u32::MAX])
        );
    }

    #[test]
    fn save_then_load_gives_identical_reads_at_every_pin_and_identical_later_behaviour() {
        let store = test_store(21);
        let bytes = store.to_bytes().expect("to bytes");
        let loaded = StackStore::from_bytes(&bytes, 21).expect("from bytes");
        assert_eq!(loaded, store);
        assert_eq!(
            loaded.history_sha256().expect("history"),
            store.history_sha256().expect("history")
        );
        let addresses = store.addresses().expect("addresses");
        assert_eq!(loaded.addresses().expect("addresses"), addresses);
        assert_eq!(addresses.len(), 4);
        let mut statuses = std::collections::BTreeSet::new();
        for address in &addresses {
            for pin in 0..=store.commit() {
                for view in VIEWS {
                    let want = store
                        .read_at(&address.scope, &address.entity, address.relation, view, pin)
                        .expect("read");
                    let got = loaded
                        .read_at(&address.scope, &address.entity, address.relation, view, pin)
                        .expect("read");
                    assert_eq!(got, want, "{address:?} {view:?} at {pin}");
                    statuses.insert(want.status());
                }
            }
        }
        // The fixture reaches every status.
        assert_eq!(statuses.len(), STATUS_COUNT);
        // Later writes behave identically, down to the serialized bytes.
        let (mut before, mut after) = (store.clone(), loaded);
        let writes: [(&[u8], &[u32], u32, &[u32], Update); 4] = [
            (b"alice", &[3, 4], 1, &[12], Update::Assert),
            (b"alice", &[3, 4], 1, &[19], Update::Assert),
            (b"bob", &[3, 4], 1, &[20], Update::Correct),
            (b"carol", &[7], 3, &[21], Update::Assert),
        ];
        for (scope, entity, relation, value, update) in writes {
            assert_eq!(
                before.write(scope, entity, relation, value, update),
                after.write(scope, entity, relation, value, update)
            );
        }
        assert_eq!(
            before.to_bytes().expect("to bytes"),
            after.to_bytes().expect("to bytes")
        );
    }

    #[test]
    fn load_rejects_foreign_lineage_malformed_bytes_and_records_not_token_encoded() {
        let store = test_store(9);
        let bytes = store.to_bytes().expect("to bytes");
        assert_eq!(
            StackStore::from_bytes(&bytes, 10),
            Err(StackStoreError::ForeignLineage {
                expected: 10,
                found: 9
            })
        );
        assert!(matches!(
            StackStore::from_bytes(b"{", 9),
            Err(StackStoreError::Memory(_))
        ));
        // Structural corruption is Memory's own validation.
        let mut memory = store.memory().clone();
        memory.records[0].commit += 7;
        assert!(matches!(
            StackStore::from_bytes(&memory.to_bytes().expect("bytes"), 9),
            Err(StackStoreError::Memory(_))
        ));
        // A value identity that disagrees with its owned tokens passes
        // Memory's validation and fails the token encoding. The single-record
        // chain has no successor whose conflict mark depends on it.
        let mut memory = store.memory().clone();
        let single = memory
            .chains
            .iter()
            .find(|(_, ids)| ids.len() == 1)
            .map(|(_, ids)| ids[0])
            .expect("a single-record chain");
        let index = memory
            .records
            .iter()
            .position(|record| record.id == single)
            .expect("record");
        memory.records[index].value = token_bytes(&[14, 15, 17]);
        assert!(memory.validate().is_ok(), "Memory alone accepts it");
        assert!(matches!(
            StackStore::from_bytes(&memory.to_bytes().expect("bytes"), 9),
            Err(StackStoreError::Encoding { .. })
        ));
        // A language-runtime memory is valid Memory but not a stack store.
        for relation in [0, TOKEN_RECORD_TAG] {
            let mut language = Memory::new(9, 8);
            language
                .write(
                    b"alpha",
                    b"Ova",
                    relation,
                    b"Bramble",
                    &[1, 2, 3],
                    7,
                    Update::Assert,
                    false,
                    0,
                )
                .expect("language write");
            assert!(matches!(
                StackStore::from_bytes(&language.to_bytes().expect("bytes"), 9),
                Err(StackStoreError::Encoding { .. })
            ));
        }
    }

    #[test]
    fn the_status_vocabulary_is_fixed_and_round_trips() {
        assert_eq!(StoreStatus::ALL.map(StoreStatus::code), [0, 1, 2, 3]);
        assert_eq!(VIEWS.map(view_code), [0, 1, 2, 3]);
        let mut seen = [false; SIGNAL_COUNT];
        for status in StoreStatus::ALL {
            assert_eq!(StoreStatus::from_code(status.code()), Some(status));
            for view in VIEWS {
                assert_eq!(view_from_code(view_code(view)), Some(view));
                let signal = StoreSignal { status, view };
                let code = signal.code() as usize;
                assert!(code < SIGNAL_COUNT && !seen[code]);
                seen[code] = true;
                assert_eq!(StoreSignal::from_code(signal.code()), Some(signal));
            }
        }
        assert!(seen.iter().all(|s| *s));
        assert_eq!(StoreStatus::from_code(4), None);
        assert_eq!(view_from_code(4), None);
        assert_eq!(StoreSignal::from_code(SIGNAL_COUNT as u32), None);
        // Reads map onto the vocabulary.
        let mut store = StackStore::new(6, 1).expect("store");
        let signal = |store: &StackStore, view| read(store, view).signal(view);
        assert_eq!(
            signal(&store, HistoryView::Initial),
            StoreSignal {
                status: StoreStatus::Absent,
                view: HistoryView::Initial
            }
        );
        write(&mut store, A, Update::Assert);
        assert_eq!(
            signal(&store, HistoryView::PreviousAssertion).status,
            StoreStatus::NoHistory
        );
        write(&mut store, B, Update::Assert);
        assert_eq!(
            signal(&store, HistoryView::Current).status,
            StoreStatus::Found
        );
        assert_eq!(
            signal(&store, HistoryView::Initial).status,
            StoreStatus::Evicted
        );
    }

    #[test]
    fn rejects_empty_addresses_zero_capacity_and_out_of_vocabulary_tokens() {
        assert_eq!(StackStore::new(1, 0), Err(StackStoreError::ZeroCapacity));
        let mut store = StackStore::new(1, 1).expect("store");
        assert_eq!(
            store.write(b"", ENTITY, RELATION, A, Update::Assert),
            Err(StackStoreError::EmptyScope)
        );
        assert_eq!(
            store.write(SCOPE, &[], RELATION, A, Update::Assert),
            Err(StackStoreError::EmptyEntity)
        );
        assert_eq!(
            store.write(SCOPE, ENTITY, RELATION, &[], Update::Assert),
            Err(StackStoreError::EmptyValue)
        );
        assert_eq!(
            store.read(SCOPE, &[], RELATION, HistoryView::Current),
            Err(StackStoreError::EmptyEntity)
        );
        assert_eq!((store.records(), store.commit()), (0, 0));
        write(&mut store, &[36, 40], Update::Assert);
        // Capacity 1 releases the record holding token 40; its value identity
        // is still checked.
        write(&mut store, &[1], Update::Correct);
        assert_eq!(read(&store, HistoryView::Initial), StoreRead::Evicted);
        assert_eq!(store.check_vocabulary(41), Ok(()));
        assert_eq!(
            store.check_vocabulary(37),
            Err(StackStoreError::TokenOutOfVocabulary {
                record: 1,
                token: 40,
                vocab_size: 37
            })
        );
        assert_eq!(
            store.check_vocabulary(12),
            Err(StackStoreError::TokenOutOfVocabulary {
                record: 1,
                token: 12,
                vocab_size: 12
            })
        );
    }
}
