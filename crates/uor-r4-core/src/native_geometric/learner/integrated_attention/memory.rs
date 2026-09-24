//! Bounded exact event history and learned-code occurrence index for integrated attention.
//!
//! Observing a token always appends its exact identity, regardless of whether the model decides
//! to derive a semantic record. Product codes only propose candidates; they never establish exact
//! identity, absence, scope authority, or a version. A bounded search can be incomplete and says so.

use std::collections::HashMap;

pub type EventId = u64;
pub type RecordId = u64;

pub const MAX_CODE_LANES: usize = 16;
pub const MAX_CANDIDATES: usize = 64;
pub const GROUP_ORDER: u8 = 120;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MemoryLimits {
    pub event_capacity: usize,
    pub record_capacity: usize,
    pub max_token_bytes: usize,
    pub max_record_bytes: usize,
    pub max_record_tokens: usize,
    pub max_key_bytes: usize,
    pub max_index_pages: usize,
    pub max_postings_per_page: usize,
    pub max_pages_per_query: usize,
    pub max_examined_entries: usize,
    pub max_candidates: usize,
}

impl MemoryLimits {
    pub fn pilot() -> Self {
        Self {
            event_capacity: 8_192,
            record_capacity: 4_096,
            max_token_bytes: 128,
            max_record_bytes: 512,
            max_record_tokens: 64,
            max_key_bytes: 128,
            max_index_pages: 16_384,
            max_postings_per_page: 64,
            max_pages_per_query: 4,
            max_examined_entries: 256,
            max_candidates: 64,
        }
    }

    pub fn validate(self) -> Result<(), MemoryError> {
        if self.event_capacity == 0 || self.record_capacity == 0 {
            return Err(MemoryError::InvalidLimits(
                "event and record capacity must be positive",
            ));
        }
        if self.max_token_bytes == 0
            || self.max_record_bytes == 0
            || self.max_record_tokens == 0
            || self.max_key_bytes == 0
        {
            return Err(MemoryError::InvalidLimits(
                "byte and token limits must be positive",
            ));
        }
        if self.max_index_pages == 0
            || self.max_postings_per_page == 0
            || self.max_pages_per_query == 0
            || self.max_examined_entries == 0
            || self.max_candidates == 0
            || self.max_candidates > MAX_CANDIDATES
        {
            return Err(MemoryError::InvalidLimits(
                "index and query limits are invalid",
            ));
        }
        // Snapshots deserialize these limits before a model-bound runtime is available. Keep
        // standalone restore bounded as well as the model configuration's tighter checks.
        if self.event_capacity > 65_536
            || self.record_capacity > 65_536
            || self.max_token_bytes > 4_096
            || self.max_record_bytes > 4_096
            || self.max_record_tokens > 4_096
            || self.max_key_bytes > 4_096
            || self.max_index_pages > 262_144
            || self.max_postings_per_page > 4_096
            || self.max_pages_per_query > MAX_CODE_LANES
            || self.max_examined_entries > 65_536
            || self
                .event_capacity
                .checked_mul(self.max_token_bytes)
                .is_none_or(|bytes| bytes > (256usize << 20))
        {
            return Err(MemoryError::InvalidLimits(
                "snapshot allocation ceiling exceeded",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MemoryError {
    InvalidLimits(&'static str),
    TokenTooLong,
    KeyTooLong,
    PayloadTooLong,
    TooManyPayloadTokens,
    InvalidCode,
    EventEvicted(EventId),
    RecordEvicted(RecordId),
    UnknownEvent(EventId),
    UnknownRecord(RecordId),
    FutureCutoff(EventId),
    IdExhausted,
    InvalidSnapshot(&'static str),
}

impl std::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "integrated memory: {self:?}")
    }
}

impl std::error::Error for MemoryError {}

/// Opaque exact bytes supplied by the learner. The geometry code is stored separately.
/// `from_parts` is available where a caller has genuinely observed typed fields; it encodes
/// length delimiters so `(scope, entity, relation)` cannot collide by concatenation.
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExactKey(Vec<u8>);

impl ExactKey {
    pub fn new(bytes: &[u8], max_bytes: usize) -> Result<Self, MemoryError> {
        if bytes.len() > max_bytes {
            return Err(MemoryError::KeyTooLong);
        }
        Ok(Self(bytes.to_vec()))
    }

    pub fn from_parts(
        scope: &[u8],
        entity: &[u8],
        relation: &[u8],
        max_bytes: usize,
    ) -> Result<Self, MemoryError> {
        let total = 12usize
            .checked_add(scope.len())
            .and_then(|v| v.checked_add(entity.len()))
            .and_then(|v| v.checked_add(relation.len()))
            .ok_or(MemoryError::KeyTooLong)?;
        if total > max_bytes
            || scope.len() > u32::MAX as usize
            || entity.len() > u32::MAX as usize
            || relation.len() > u32::MAX as usize
        {
            return Err(MemoryError::KeyTooLong);
        }
        let mut bytes = Vec::with_capacity(total);
        for part in [scope, entity, relation] {
            bytes.extend_from_slice(&(part.len() as u32).to_le_bytes());
            bytes.extend_from_slice(part);
        }
        Ok(Self(bytes))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

#[derive(
    Clone, Copy, Debug, Default, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
pub struct ProductCode {
    len: u8,
    lanes: [u8; MAX_CODE_LANES],
}

impl ProductCode {
    pub fn new(lanes: &[u8]) -> Result<Self, MemoryError> {
        if lanes.is_empty()
            || lanes.len() > MAX_CODE_LANES
            || lanes.iter().any(|&v| v >= GROUP_ORDER)
        {
            return Err(MemoryError::InvalidCode);
        }
        let mut code = Self::default();
        code.len = lanes.len() as u8;
        code.lanes[..lanes.len()].copy_from_slice(lanes);
        Ok(code)
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.lanes[..usize::from(self.len)]
    }

    fn prefix(self, len: usize) -> PrefixKey {
        PrefixKey {
            len: len as u8,
            lanes: self.lanes,
        }
    }

    fn is_valid(self) -> bool {
        self.len > 0
            && usize::from(self.len) <= MAX_CODE_LANES
            && self.as_slice().iter().all(|&v| v < GROUP_ORDER)
            && self.lanes[usize::from(self.len)..].iter().all(|&v| v == 0)
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct PrefixKey {
    len: u8,
    lanes: [u8; MAX_CODE_LANES],
}

impl PrefixKey {
    fn from_code(code: ProductCode, len: usize) -> Self {
        let mut prefix = code.prefix(len);
        prefix.lanes[len..].fill(0);
        prefix
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RawEvent {
    pub id: EventId,
    pub source_id: u64,
    pub turn_id: u64,
    pub token_id: u32,
    pub byte_start: u64,
    bytes: Vec<u8>,
}

impl RawEvent {
    fn empty(max_token_bytes: usize) -> Self {
        Self {
            id: 0,
            source_id: 0,
            turn_id: 0,
            token_id: 0,
            byte_start: 0,
            bytes: Vec::with_capacity(max_token_bytes),
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn byte_end(&self) -> Option<u64> {
        self.byte_start.checked_add(self.bytes.len() as u64)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StoredRecord {
    pub id: RecordId,
    pub source_event_id: EventId,
    pub source_id: u64,
    pub commit_event_id: EventId,
    key: ExactKey,
    payload: Vec<u8>,
    token_ids: Vec<u32>,
    code: ProductCode,
    indexed_prefixes: u8,
}

impl StoredRecord {
    fn empty() -> Self {
        Self {
            id: 0,
            source_event_id: 0,
            source_id: 0,
            commit_event_id: 0,
            key: ExactKey::default(),
            payload: Vec::new(),
            token_ids: Vec::new(),
            code: ProductCode::default(),
            indexed_prefixes: 0,
        }
    }

    pub fn key(&self) -> &ExactKey {
        &self.key
    }
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
    pub fn token_ids(&self) -> &[u32] {
        &self.token_ids
    }
    pub fn code(&self) -> ProductCode {
        self.code
    }
    pub fn index_complete(&self) -> bool {
        self.indexed_prefixes == self.code.len
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WriteReceipt {
    pub record_id: RecordId,
    pub evicted_record: Option<RecordId>,
    pub indexed_prefixes: u8,
    pub index_complete: bool,
}

/// A code search is always approximate: even an untruncated prefix search cannot prove absence.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SearchStatus {
    pub incomplete_index: bool,
    pub page_limit: bool,
    pub examined_limit: bool,
    pub candidate_limit: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchResult {
    candidates: [RecordId; MAX_CANDIDATES],
    len: usize,
    pub pages_visited: usize,
    pub entries_examined: usize,
    pub status: SearchStatus,
}

impl SearchResult {
    fn empty(incomplete_index: bool) -> Self {
        Self {
            candidates: [0; MAX_CANDIDATES],
            len: 0,
            pages_visited: 0,
            entries_examined: 0,
            status: SearchStatus {
                incomplete_index,
                ..SearchStatus::default()
            },
        }
    }

    pub fn candidates(&self) -> &[RecordId] {
        &self.candidates[..self.len]
    }
}

/// Off-path snapshot of retained events/records. Index pages are deliberately reconstructed and
/// validated on restore rather than trusted as serialized authority.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MemorySnapshot {
    limits: MemoryLimits,
    events: Vec<RawEvent>,
    records: Vec<StoredRecord>,
    next_event_id: EventId,
    next_record_id: RecordId,
    event_len: usize,
    record_len: usize,
}

pub struct ExactMemory {
    limits: MemoryLimits,
    events: Vec<RawEvent>,
    records: Vec<StoredRecord>,
    next_event_id: EventId,
    next_record_id: RecordId,
    event_len: usize,
    record_len: usize,
    pages: HashMap<PrefixKey, Vec<RecordId>>,
    incomplete_records: usize,
}

impl ExactMemory {
    pub fn new(limits: MemoryLimits) -> Result<Self, MemoryError> {
        limits.validate()?;
        let mut events = Vec::with_capacity(limits.event_capacity);
        for _ in 0..limits.event_capacity {
            events.push(RawEvent::empty(limits.max_token_bytes));
        }
        let mut records = Vec::with_capacity(limits.record_capacity);
        for _ in 0..limits.record_capacity {
            records.push(StoredRecord::empty());
        }
        Ok(Self {
            limits,
            events,
            records,
            next_event_id: 1,
            next_record_id: 1,
            event_len: 0,
            record_len: 0,
            pages: HashMap::with_capacity(limits.max_index_pages),
            incomplete_records: 0,
        })
    }

    pub fn limits(&self) -> MemoryLimits {
        self.limits
    }
    pub fn latest_event_id(&self) -> Option<EventId> {
        self.next_event_id.checked_sub(1).filter(|&v| v > 0)
    }
    pub fn retained_event_count(&self) -> usize {
        self.event_len
    }
    pub fn retained_record_count(&self) -> usize {
        self.record_len
    }
    pub fn incomplete_record_count(&self) -> usize {
        self.incomplete_records
    }

    /// Raw observation is independent of every learned semantic write decision. The fixed event
    /// ring and per-slot byte buffers are initialized at construction and reused without growth.
    pub fn observe(
        &mut self,
        source_id: u64,
        turn_id: u64,
        token_id: u32,
        byte_start: u64,
        bytes: &[u8],
    ) -> Result<EventId, MemoryError> {
        if bytes.len() > self.limits.max_token_bytes
            || byte_start.checked_add(bytes.len() as u64).is_none()
        {
            return Err(MemoryError::TokenTooLong);
        }
        let id = self.next_event_id;
        let next = id.checked_add(1).ok_or(MemoryError::IdExhausted)?;
        let slot = ring_slot(id, self.limits.event_capacity);
        let event = &mut self.events[slot];
        event.id = id;
        event.source_id = source_id;
        event.turn_id = turn_id;
        event.token_id = token_id;
        event.byte_start = byte_start;
        event.bytes.clear();
        event.bytes.extend_from_slice(bytes);
        self.next_event_id = next;
        self.event_len = (self.event_len + 1).min(self.limits.event_capacity);
        Ok(id)
    }

    pub fn raw_event(&self, id: EventId) -> Result<&RawEvent, MemoryError> {
        if id == 0 || id >= self.next_event_id {
            return Err(MemoryError::UnknownEvent(id));
        }
        let event = &self.events[ring_slot(id, self.limits.event_capacity)];
        if event.id != id {
            return Err(MemoryError::EventEvicted(id));
        }
        Ok(event)
    }

    pub fn record(&self, id: RecordId) -> Result<&StoredRecord, MemoryError> {
        if id == 0 || id >= self.next_record_id {
            return Err(MemoryError::UnknownRecord(id));
        }
        let record = &self.records[ring_slot(id, self.limits.record_capacity)];
        if record.id != id {
            return Err(MemoryError::RecordEvicted(id));
        }
        Ok(record)
    }

    /// The learner supplies the exact key and product code after observing the responsible event.
    /// Owned payload bytes/token IDs survive raw-event ring eviction until this record is evicted.
    pub fn write(
        &mut self,
        source_event_id: EventId,
        key: &ExactKey,
        payload: &[u8],
        token_ids: &[u32],
        code: ProductCode,
    ) -> Result<WriteReceipt, MemoryError> {
        if key.as_bytes().len() > self.limits.max_key_bytes {
            return Err(MemoryError::KeyTooLong);
        }
        if payload.len() > self.limits.max_record_bytes {
            return Err(MemoryError::PayloadTooLong);
        }
        if token_ids.len() > self.limits.max_record_tokens {
            return Err(MemoryError::TooManyPayloadTokens);
        }
        if !code.is_valid() {
            return Err(MemoryError::InvalidCode);
        }
        let source_id = self.raw_event(source_event_id)?.source_id;
        let commit_event_id = self
            .latest_event_id()
            .ok_or(MemoryError::UnknownEvent(source_event_id))?;
        let id = self.next_record_id;
        let next = id.checked_add(1).ok_or(MemoryError::IdExhausted)?;
        let slot = ring_slot(id, self.limits.record_capacity);
        let evicted_record = (self.records[slot].id != 0).then_some(self.records[slot].id);
        if evicted_record.is_some() {
            self.remove_postings(slot);
        }
        let record = &mut self.records[slot];
        record.id = id;
        record.source_event_id = source_event_id;
        record.source_id = source_id;
        record.commit_event_id = commit_event_id;
        record.key.0.clear();
        record.key.0.extend_from_slice(key.as_bytes());
        record.payload.clear();
        record.payload.extend_from_slice(payload);
        record.token_ids.clear();
        record.token_ids.extend_from_slice(token_ids);
        record.code = code;
        record.indexed_prefixes = 0;
        self.next_record_id = next;
        self.record_len = (self.record_len + 1).min(self.limits.record_capacity);
        self.add_postings(slot);
        let indexed_prefixes = self.records[slot].indexed_prefixes;
        Ok(WriteReceipt {
            record_id: id,
            evicted_record,
            indexed_prefixes,
            index_complete: indexed_prefixes == code.len,
        })
    }

    fn remove_postings(&mut self, slot: usize) {
        let old = &self.records[slot];
        if !old.index_complete() {
            self.incomplete_records = self.incomplete_records.saturating_sub(1);
        }
        for len in 1..=usize::from(old.code.len) {
            let prefix = PrefixKey::from_code(old.code, len);
            let empty = if let Some(ids) = self.pages.get_mut(&prefix) {
                ids.retain(|&id| id != old.id);
                ids.is_empty()
            } else {
                false
            };
            if empty {
                self.pages.remove(&prefix);
            }
        }
    }

    fn add_postings(&mut self, slot: usize) {
        let id = self.records[slot].id;
        let code = self.records[slot].code;
        let mut inserted = 0u8;
        for len in 1..=usize::from(code.len) {
            let prefix = PrefixKey::from_code(code, len);
            if let Some(ids) = self.pages.get_mut(&prefix) {
                if ids.len() < self.limits.max_postings_per_page {
                    ids.push(id);
                    inserted += 1;
                }
            } else if self.pages.len() < self.limits.max_index_pages {
                let mut ids = Vec::with_capacity(self.limits.max_postings_per_page);
                ids.push(id);
                self.pages.insert(prefix, ids);
                inserted += 1;
            }
        }
        self.records[slot].indexed_prefixes = inserted;
        if inserted != code.len {
            self.incomplete_records += 1;
        }
    }

    /// Newest (most specific) matching prefixes are examined first. Every source/commit is checked
    /// against `causal_cutoff`; no future semantic write can leak through an older source event.
    /// An optional exact key is a filter, never inferred from the geometric code.
    pub fn query(
        &self,
        code: ProductCode,
        exact_key: Option<&ExactKey>,
        source_filter: Option<u64>,
        causal_cutoff: EventId,
    ) -> Result<SearchResult, MemoryError> {
        if !code.is_valid() {
            return Err(MemoryError::InvalidCode);
        }
        if causal_cutoff >= self.next_event_id {
            return Err(MemoryError::FutureCutoff(causal_cutoff));
        }
        let mut result = SearchResult::empty(self.incomplete_records > 0);
        for len in (1..=usize::from(code.len)).rev() {
            if result.pages_visited >= self.limits.max_pages_per_query {
                result.status.page_limit = true;
                break;
            }
            result.pages_visited += 1;
            let prefix = PrefixKey::from_code(code, len);
            let Some(ids) = self.pages.get(&prefix) else {
                continue;
            };
            for &id in ids.iter().rev() {
                if result.entries_examined >= self.limits.max_examined_entries {
                    result.status.examined_limit = true;
                    break;
                }
                result.entries_examined += 1;
                let Ok(record) = self.record(id) else {
                    continue;
                };
                if record.commit_event_id > causal_cutoff
                    || record.source_event_id > causal_cutoff
                    || source_filter.is_some_and(|source| source != record.source_id)
                    || exact_key.is_some_and(|key| key != &record.key)
                    || result.candidates().contains(&id)
                {
                    continue;
                }
                if result.len >= self.limits.max_candidates {
                    result.status.candidate_limit = true;
                    break;
                }
                result.candidates[result.len] = id;
                result.len += 1;
            }
            if result.status.examined_limit || result.status.candidate_limit {
                break;
            }
        }
        Ok(result)
    }

    pub fn snapshot(&self) -> MemorySnapshot {
        MemorySnapshot {
            limits: self.limits,
            events: self.events.clone(),
            records: self.records.clone(),
            next_event_id: self.next_event_id,
            next_record_id: self.next_record_id,
            event_len: self.event_len,
            record_len: self.record_len,
        }
    }

    pub fn restore(snapshot: MemorySnapshot) -> Result<Self, MemoryError> {
        snapshot.limits.validate()?;
        if snapshot.events.len() != snapshot.limits.event_capacity
            || snapshot.records.len() != snapshot.limits.record_capacity
            || snapshot.event_len > snapshot.limits.event_capacity
            || snapshot.record_len > snapshot.limits.record_capacity
            || snapshot.next_event_id == 0
            || snapshot.next_record_id == 0
            || snapshot.event_len
                != (snapshot.next_event_id - 1).min(snapshot.limits.event_capacity as u64) as usize
            || snapshot.record_len
                != (snapshot.next_record_id - 1).min(snapshot.limits.record_capacity as u64)
                    as usize
        {
            return Err(MemoryError::InvalidSnapshot("shape or cursor"));
        }
        let mut memory = Self::new(snapshot.limits)?;
        memory.events = snapshot.events;
        memory.records = snapshot.records;
        memory.next_event_id = snapshot.next_event_id;
        memory.next_record_id = snapshot.next_record_id;
        memory.event_len = snapshot.event_len;
        memory.record_len = snapshot.record_len;
        let earliest_event = memory.next_event_id.saturating_sub(memory.event_len as u64);
        let earliest_record = memory
            .next_record_id
            .saturating_sub(memory.record_len as u64);
        let mut live_events = 0usize;
        for (slot, event) in memory.events.iter_mut().enumerate() {
            if event.bytes.capacity() < memory.limits.max_token_bytes {
                event.bytes.reserve_exact(
                    memory
                        .limits
                        .max_token_bytes
                        .saturating_sub(event.bytes.len()),
                );
            }
            if event.id == 0 {
                continue;
            }
            live_events += 1;
            if event.id >= memory.next_event_id
                || event.id < earliest_event
                || event.bytes.len() > memory.limits.max_token_bytes
                || event.byte_end().is_none()
                || ring_slot(event.id, memory.limits.event_capacity) != slot
            {
                return Err(MemoryError::InvalidSnapshot("event identity or bytes"));
            }
        }
        if live_events != memory.event_len {
            return Err(MemoryError::InvalidSnapshot("event count"));
        }
        let mut live_records = 0usize;
        for slot in 0..memory.records.len() {
            let record = &memory.records[slot];
            if record.id == 0 {
                continue;
            }
            live_records += 1;
            if record.id >= memory.next_record_id
                || record.id < earliest_record
                || ring_slot(record.id, memory.limits.record_capacity) != slot
                || record.source_event_id == 0
                || record.source_event_id > record.commit_event_id
                || record.commit_event_id >= memory.next_event_id
                || record.key.as_bytes().len() > memory.limits.max_key_bytes
                || record.payload.len() > memory.limits.max_record_bytes
                || record.token_ids.len() > memory.limits.max_record_tokens
                || !record.code.is_valid()
            {
                return Err(MemoryError::InvalidSnapshot("record identity or causality"));
            }
            if let Ok(event) = memory.raw_event(record.source_event_id) {
                if event.source_id != record.source_id {
                    return Err(MemoryError::InvalidSnapshot("record source identity"));
                }
            }
        }
        if live_records != memory.record_len {
            return Err(MemoryError::InvalidSnapshot("record count"));
        }
        for id in earliest_event..memory.next_event_id {
            if id > 0 && memory.raw_event(id).is_err() {
                return Err(MemoryError::InvalidSnapshot("event gap"));
            }
        }
        for id in earliest_record..memory.next_record_id {
            if id > 0 && memory.record(id).is_err() {
                return Err(MemoryError::InvalidSnapshot("record gap"));
            }
        }
        for id in earliest_record..memory.next_record_id {
            if id > 0 {
                let slot = ring_slot(id, memory.limits.record_capacity);
                memory.add_postings(slot);
            }
        }
        Ok(memory)
    }

    /// Check the loaded model's declared limits before allocating a restored ring. A snapshot's
    /// serialized limits are not authority for request memory use.
    pub fn restore_with_limits(
        snapshot: MemorySnapshot,
        expected: MemoryLimits,
    ) -> Result<Self, MemoryError> {
        expected.validate()?;
        if snapshot.limits != expected {
            return Err(MemoryError::InvalidSnapshot("foreign memory limits"));
        }
        Self::restore(snapshot)
    }
}

fn ring_slot(id: u64, capacity: usize) -> usize {
    ((id - 1) % capacity as u64) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits() -> MemoryLimits {
        MemoryLimits {
            event_capacity: 3,
            record_capacity: 2,
            max_token_bytes: 8,
            max_record_bytes: 16,
            max_record_tokens: 4,
            max_key_bytes: 16,
            max_index_pages: 8,
            max_postings_per_page: 2,
            max_pages_per_query: 2,
            max_examined_entries: 4,
            max_candidates: 2,
        }
    }

    #[test]
    fn raw_history_is_independent_of_semantic_writes_and_eviction_is_explicit() {
        let mut memory = ExactMemory::new(limits()).unwrap();
        for value in 1..=4 {
            memory
                .observe(7, 1, value, value as u64, &[value as u8])
                .unwrap();
        }
        assert_eq!(memory.retained_event_count(), 3);
        assert_eq!(memory.retained_record_count(), 0);
        assert!(matches!(
            memory.raw_event(1),
            Err(MemoryError::EventEvicted(1))
        ));
        assert_eq!(memory.raw_event(4).unwrap().bytes(), &[4]);
    }

    #[test]
    fn query_respects_commit_cutoff_source_and_owned_payload() {
        let mut memory = ExactMemory::new(limits()).unwrap();
        let key = ExactKey::new(b"entity", 16).unwrap();
        let code = ProductCode::new(&[3, 7]).unwrap();
        let first = memory.observe(11, 1, 10, 0, b"a").unwrap();
        let r1 = memory
            .write(first, &key, b"alpha", &[10], code)
            .unwrap()
            .record_id;
        let second = memory.observe(11, 1, 20, 1, b"b").unwrap();
        let r2 = memory
            .write(first, &key, b"beta", &[20], code)
            .unwrap()
            .record_id;
        let before = memory.query(code, None, Some(11), first).unwrap();
        assert_eq!(before.candidates(), &[r1]);
        let after = memory.query(code, None, Some(11), second).unwrap();
        assert_eq!(after.candidates(), &[r2, r1]);
        assert!(memory
            .query(code, None, Some(12), second)
            .unwrap()
            .candidates()
            .is_empty());
        assert_eq!(memory.record(r2).unwrap().payload(), b"beta");
        assert_eq!(memory.record(r2).unwrap().token_ids(), &[20]);
    }

    #[test]
    fn record_eviction_and_index_overflow_are_visible() {
        let mut l = limits();
        l.max_postings_per_page = 1;
        let mut memory = ExactMemory::new(l).unwrap();
        let key = ExactKey::new(b"k", 16).unwrap();
        let code = ProductCode::new(&[1]).unwrap();
        let e1 = memory.observe(1, 1, 1, 0, b"a").unwrap();
        let r1 = memory.write(e1, &key, b"a", &[1], code).unwrap();
        assert!(r1.index_complete);
        let e2 = memory.observe(1, 1, 2, 1, b"b").unwrap();
        let r2 = memory.write(e2, &key, b"b", &[2], code).unwrap();
        assert!(!r2.index_complete);
        assert!(
            memory
                .query(code, None, None, e2)
                .unwrap()
                .status
                .incomplete_index
        );
        let e3 = memory.observe(1, 1, 3, 2, b"c").unwrap();
        let r3 = memory.write(e3, &key, b"c", &[3], code).unwrap();
        assert_eq!(r3.evicted_record, Some(r1.record_id));
        assert!(r3.index_complete);
        assert!(matches!(
            memory.record(r1.record_id),
            Err(MemoryError::RecordEvicted(_))
        ));
        assert_eq!(
            memory.query(code, None, None, e3).unwrap().candidates(),
            &[r3.record_id]
        );
    }

    #[test]
    fn search_budgets_report_truncation_without_claiming_absence() {
        let mut l = limits();
        l.record_capacity = 3;
        l.max_postings_per_page = 3;
        l.max_pages_per_query = 1;
        l.max_examined_entries = 1;
        let mut memory = ExactMemory::new(l).unwrap();
        let key = ExactKey::new(b"k", 16).unwrap();
        let code = ProductCode::new(&[2, 3]).unwrap();
        for token in 1..=3 {
            let event = memory
                .observe(4, 1, token, token as u64, &[token as u8])
                .unwrap();
            memory
                .write(event, &key, &[token as u8], &[token], code)
                .unwrap();
        }
        let result = memory.query(code, None, Some(4), 3).unwrap();
        assert_eq!(result.candidates().len(), 1);
        assert_eq!(result.entries_examined, 1);
        assert!(result.status.examined_limit);
        assert!(!result.status.incomplete_index);
    }

    #[test]
    fn restore_rebuilds_index_and_rejects_future_commit() {
        let mut memory = ExactMemory::new(limits()).unwrap();
        let event = memory.observe(4, 1, 9, 0, b"x").unwrap();
        let key = ExactKey::new(b"x", 16).unwrap();
        let code = ProductCode::new(&[2, 5]).unwrap();
        let record = memory
            .write(event, &key, b"exact", &[9], code)
            .unwrap()
            .record_id;
        let mut snapshot = memory.snapshot();
        let restored = ExactMemory::restore(snapshot.clone()).unwrap();
        assert_eq!(
            restored
                .query(code, Some(&key), Some(4), event)
                .unwrap()
                .candidates(),
            &[record]
        );
        snapshot.records[0].commit_event_id = event + 1;
        assert!(matches!(
            ExactMemory::restore(snapshot),
            Err(MemoryError::InvalidSnapshot(_))
        ));
    }

    #[test]
    fn restore_rejects_foreign_limits_before_allocating_session_ring() {
        let memory = ExactMemory::new(limits()).unwrap();
        let mut other = limits();
        other.event_capacity += 1;
        assert!(matches!(
            ExactMemory::restore_with_limits(memory.snapshot(), other),
            Err(MemoryError::InvalidSnapshot("foreign memory limits"))
        ));
    }
}
