//! Exact versioned storage extracted unchanged from scoped_memory.rs.
//! Model intent fitting, numerical decoders and serving runtime remain excluded.
#![forbid(unsafe_code)]
use super::realtext_support::sha256_hex;
/// Classified failures at the scoped-memory boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScopedMemoryError {
    Model(String),
    Intent(String),
    Observation(String),
    Store(String),
    Session(String),
    Serialization(String),
    Computation(String),
}

impl std::fmt::Display for ScopedMemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (category, detail) = match self {
            Self::Model(detail) => ("model", detail),
            Self::Intent(detail) => ("intent", detail),
            Self::Observation(detail) => ("observation", detail),
            Self::Store(detail) => ("store", detail),
            Self::Session(detail) => ("session", detail),
            Self::Serialization(detail) => ("serialization", detail),
            Self::Computation(detail) => ("computation", detail),
        };
        write!(f, "scoped-memory {category}: {detail}")
    }
}

impl std::error::Error for ScopedMemoryError {}

impl From<ScopedMemoryError> for String {
    fn from(error: ScopedMemoryError) -> Self {
        error.to_string()
    }
}

pub const SCOPED_STORE_VERSION: u8 = 2;
const ASK_CURRENT: usize = 0;
const ASK_PREVIOUS: usize = 1;
const ASK_INITIAL: usize = 2;
/// Which retained historical position an answer requests. Only the first three are reachable from
/// learned question intent; the fourth is exact infrastructure used by the harness.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum HistoryView {
    Current,
    PreviousAssertion,
    PreviousDistinctValue,
    Initial,
}

impl HistoryView {
    fn from_question_intent(intent: usize) -> Option<Self> {
        match intent {
            ASK_CURRENT => Some(Self::Current),
            ASK_PREVIOUS => Some(Self::PreviousAssertion),
            ASK_INITIAL => Some(Self::Initial),
            _ => None,
        }
    }
}

/// Learned update action stored with a record.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum Update {
    Assert,
    Correct,
}

fn payload_sha256(payload: &[u32]) -> String {
    let bytes: Vec<u8> = payload
        .iter()
        .flat_map(|token| token.to_le_bytes())
        .collect();
    sha256_hex(&bytes)
}

/// Injective, length-delimited key encoding. Delimiter concatenation can collide; this cannot.
pub fn encode_key(scope: &[u8], entity: &[u8], relation: u8) -> Vec<u8> {
    let mut out = Vec::with_capacity(scope.len() + entity.len() + 9);
    out.extend_from_slice(&(scope.len() as u32).to_le_bytes());
    out.extend_from_slice(scope);
    out.extend_from_slice(&(entity.len() as u32).to_le_bytes());
    out.extend_from_slice(entity);
    out.push(relation);
    out
}

/// One exact versioned record. `payload` is the owned surface used for emission; `value` is the exact
/// lexical identity used for equality and dependent continuation.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub id: u64,
    pub scope: Vec<u8>,
    pub entity: Vec<u8>,
    pub relation: u8,
    pub value: Vec<u8>,
    pub payload: Vec<u32>,
    /// Immutable witness retained when capacity releases the owned token buffer.
    pub payload_sha256: String,
    /// 0 for the first record of a chain.
    pub predecessor: u64,
    pub commit: u64,
    pub source: u64,
    pub action: Update,
    /// A bare assertion contradicted the then-current value.
    pub conflict: bool,
    /// Learned: the value is itself an entity key, so a dependent read continues.
    pub continues: bool,
    /// The joint parse score of the source clause. Stored for the parse-confidence control; it never
    /// participates in version selection.
    pub parse_score: i64,
    /// Declared capacity released this record's owned payload.
    pub evicted: bool,
}

/// The exact versioned store.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Memory {
    pub version: u8,
    /// Stable lineage identity, preserved across save/reload, so a foreign session is rejected.
    pub lineage: u64,
    pub records: Vec<Record>,
    /// Chain per encoded key, sorted by key and ordered by ascending commit.
    pub chains: Vec<(Vec<u8>, Vec<u64>)>,
    pub next_id: u64,
    pub commit: u64,
    pub capacity: usize,
}

/// Typed outcome of an exact historical lookup. An unfollowed link is never reported as absence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lookup<'a> {
    Found(&'a Record),
    /// No record is eligible at this view for this address.
    Absent,
    /// The address exists at this view but the requested historical position does not.
    NoHistory,
    /// The required record exists but its owned payload was released under declared capacity.
    Evicted,
}

/// Outcome of one committed write.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Written {
    pub id: u64,
    pub commit: u64,
    pub action: Update,
    pub conflict: bool,
    pub superseded: u64,
    pub revision: u64,
}

impl Memory {
    pub fn new(lineage: u64, capacity: usize) -> Self {
        Memory {
            version: SCOPED_STORE_VERSION,
            lineage,
            records: Vec::new(),
            chains: Vec::new(),
            next_id: 1,
            commit: 0,
            capacity: capacity.max(1),
        }
    }

    /// Exact chain index for an encoded key.
    pub fn chain_index(&self, key: &[u8]) -> Option<usize> {
        self.chains
            .binary_search_by(|(k, _)| k.as_slice().cmp(key))
            .ok()
    }

    /// Exact record by id.
    pub fn record_ref(&self, id: u64) -> Option<&Record> {
        self.records.iter().find(|r| r.id == id)
    }

    /// The retained chain for an encoded key, in ascending commit order.
    pub fn chain(&self, key: &[u8]) -> &[u64] {
        self.chain_index(key)
            .map(|index| self.chains[index].1.as_slice())
            .unwrap_or(&[])
    }

    /// Exact lookup under one pinned view. Version order alone decides eligibility; a parse score or a
    /// physical position never does.
    pub fn lookup(&self, key: &[u8], view: u64, history: HistoryView) -> Lookup<'_> {
        // Preserve the declared conservative public policy: a previous-distinct query does not
        // traverse a released predecessor. Internal identity verification still uses its retained
        // lexical metadata so an already-owned historical capture survives later eviction.
        if history == HistoryView::PreviousDistinctValue {
            let mut visible = self
                .chain(key)
                .iter()
                .rev()
                .filter_map(|id| self.record_ref(*id))
                .filter(|record| record.commit <= view);
            if let Some(head) = visible.next() {
                for previous in visible {
                    if previous.evicted {
                        return Lookup::Evicted;
                    }
                    if previous.value != head.value {
                        break;
                    }
                }
            }
        }
        match self.lookup_identity(key, view, history) {
            Lookup::Found(record) if record.evicted => Lookup::Evicted,
            result => result,
        }
    }

    // Metadata and lexical values remain available as tombstones. Selection identity therefore
    // survives payload release, while the public lookup reports Evicted for a required payload.
    fn lookup_identity(&self, key: &[u8], view: u64, history: HistoryView) -> Lookup<'_> {
        let Some(index) = self.chain_index(key) else {
            return Lookup::Absent;
        };
        let visible: Vec<&Record> = self.chains[index]
            .1
            .iter()
            .filter_map(|id| self.record_ref(*id))
            .filter(|record| record.commit <= view)
            .collect();
        let Some(head) = visible.last() else {
            return Lookup::Absent;
        };
        match history {
            HistoryView::Current => Lookup::Found(head),
            HistoryView::PreviousAssertion => match visible.len().checked_sub(2) {
                Some(position) => {
                    let record = visible[position];
                    Lookup::Found(record)
                }
                None => Lookup::NoHistory,
            },
            HistoryView::PreviousDistinctValue => {
                let mut position = visible.len();
                while let Some(previous) = position.checked_sub(1).and_then(|p| visible.get(p)) {
                    if previous.id == head.id {
                        position -= 1;
                        continue;
                    }
                    if previous.value != head.value {
                        return Lookup::Found(previous);
                    }
                    position -= 1;
                }
                Lookup::NoHistory
            }
            HistoryView::Initial => {
                let record = visible[0];
                Lookup::Found(record)
            }
        }
    }

    /// The record currently authoritative for `key` at the latest commit.
    pub fn head(&self, key: &[u8]) -> Lookup<'_> {
        self.lookup(key, u64::MAX, HistoryView::Current)
    }

    /// Commit one record. Version order, the predecessor link and the capacity release are exact.
    #[allow(clippy::too_many_arguments)]
    pub fn write(
        &mut self,
        scope: &[u8],
        entity: &[u8],
        relation: u8,
        value: &[u8],
        payload: &[u32],
        source: u64,
        action: Update,
        continues: bool,
        parse_score: i64,
    ) -> Result<Written, ScopedMemoryError> {
        if scope.is_empty() || entity.is_empty() || value.is_empty() {
            return Err(ScopedMemoryError::Store(format!(
                "scope/entity/value must be nonempty (scope={} entity={} value={})",
                scope.len(),
                entity.len(),
                value.len()
            )));
        }
        if payload.is_empty() {
            return Err(ScopedMemoryError::Store(
                "an owned payload must have at least one token".into(),
            ));
        }
        let next_id = self
            .next_id
            .checked_add(1)
            .ok_or_else(|| ScopedMemoryError::Store("record id exhausted".into()))?;
        let next_commit = self
            .commit
            .checked_add(1)
            .ok_or_else(|| ScopedMemoryError::Store("commit counter exhausted".into()))?;
        let key = encode_key(scope, entity, relation);
        if self.chain_index(&key).is_none() {
            self.chains.push((key.clone(), Vec::new()));
            self.chains.sort_by(|a, b| a.0.cmp(&b.0));
        }
        let index = self
            .chain_index(&key)
            .ok_or_else(|| ScopedMemoryError::Store("inserted chain is not addressable".into()))?;
        let head_id = self.chains[index].1.last().copied().unwrap_or(0);
        let previous = self
            .record_ref(head_id)
            .map(|r| (r.value.clone(), r.evicted));
        let conflict = match (&previous, action) {
            (Some((value_before, false)), Update::Assert) => value_before.as_slice() != value,
            _ => false,
        };
        let revision = self.chains[index].1.len() as u64 + 1;
        let id = self.next_id;
        self.next_id = next_id;
        self.commit = next_commit;
        self.records.push(Record {
            id,
            scope: scope.to_vec(),
            entity: entity.to_vec(),
            relation,
            value: value.to_vec(),
            payload: payload.to_vec(),
            payload_sha256: payload_sha256(payload),
            predecessor: head_id,
            commit: self.commit,
            source,
            action,
            conflict,
            continues,
            parse_score,
            evicted: false,
        });
        self.chains[index].1.push(id);
        // Declared capacity releases the oldest owned payload as a visible tombstone.
        if self.chains[index].1.len() > self.capacity {
            let expired = self.chains[index].1.len() - self.capacity - 1;
            let oldest = self.chains[index].1[expired];
            if let Some(record) = self.records.iter_mut().find(|r| r.id == oldest) {
                record.evicted = true;
                record.payload = Vec::new();
            }
        }
        Ok(Written {
            id,
            commit: self.commit,
            action,
            conflict,
            superseded: head_id,
            revision,
        })
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, ScopedMemoryError> {
        serde_json::to_vec(self).map_err(|e| ScopedMemoryError::Serialization(e.to_string()))
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ScopedMemoryError> {
        let memory: Self = serde_json::from_slice(bytes)
            .map_err(|e| ScopedMemoryError::Serialization(e.to_string()))?;
        memory.validate()?;
        Ok(memory)
    }

    /// Bind an answer to immutable history rather than a caller-reused lineage/counter alone.
    /// Payload eviction is excluded, so an owned capture remains valid after capacity release.
    pub fn history_sha256(&self, view: u64) -> Result<String, ScopedMemoryError> {
        if view > self.commit {
            return Err(ScopedMemoryError::Store(
                "view exceeds committed history".into(),
            ));
        }
        let mut records: Vec<_> = self.records.iter().filter(|r| r.commit <= view).collect();
        records.sort_by_key(|r| r.id);
        let evidence: Vec<_> = records
            .iter()
            .map(|r| {
                (
                    r.id,
                    &r.scope,
                    &r.entity,
                    r.relation,
                    &r.value,
                    &r.payload_sha256,
                    r.predecessor,
                    r.commit,
                    r.source,
                    r.action,
                    r.conflict,
                    r.continues,
                    r.parse_score,
                )
            })
            .collect();
        let bytes = serde_json::to_vec(&("scoped-history-v2", self.lineage, view, evidence))
            .map_err(|e| ScopedMemoryError::Serialization(e.to_string()))?;
        Ok(sha256_hex(&bytes))
    }

    pub fn validate(&self) -> Result<(), ScopedMemoryError> {
        let bad = |message: &str| ScopedMemoryError::Store(message.into());
        if self.version != SCOPED_STORE_VERSION {
            return Err(bad("unsupported store version"));
        }
        if self.capacity == 0 {
            return Err(bad("capacity must be positive"));
        }
        let count = u64::try_from(self.records.len()).map_err(|_| bad("too many records"))?;
        if count.checked_add(1) != Some(self.next_id) || self.commit != count {
            return Err(bad("record ids and commits must be a dense exact sequence"));
        }
        let mut by_id = std::collections::BTreeMap::new();
        for record in &self.records {
            if record.id == 0
                || record.id > count
                || record.commit != record.id
                || by_id.insert(record.id, record).is_some()
            {
                return Err(bad(
                    "record id/commit is duplicate or outside the exact sequence",
                ));
            }
            if record.scope.is_empty() || record.entity.is_empty() || record.value.is_empty() {
                return Err(bad("record has an empty address or value"));
            }
            if record.payload_sha256.len() != 64
                || !record.payload_sha256.bytes().all(|b| b.is_ascii_hexdigit())
                || (record.evicted && !record.payload.is_empty())
                || (!record.evicted
                    && (record.payload.is_empty()
                        || record.payload_sha256 != payload_sha256(&record.payload)))
            {
                return Err(bad("record payload does not match its immutable witness"));
            }
        }
        if self.chains.windows(2).any(|w| w[0].0 >= w[1].0) {
            return Err(bad("chains must have unique sorted keys"));
        }
        let mut covered = std::collections::BTreeSet::new();
        for (key, ids) in &self.chains {
            if ids.is_empty() {
                return Err(bad("empty chain"));
            }
            let mut previous: Option<&Record> = None;
            for (position, id) in ids.iter().enumerate() {
                let record = *by_id
                    .get(id)
                    .ok_or_else(|| bad("chain references a missing id"))?;
                if !covered.insert(*id) {
                    return Err(bad("record occurs more than once in chains"));
                }
                if &encode_key(&record.scope, &record.entity, record.relation) != key
                    || record.predecessor != previous.map_or(0, |r| r.id)
                    || previous.is_some_and(|r| record.commit <= r.commit)
                {
                    return Err(bad(
                        "record address/predecessor/order disagrees with its chain",
                    ));
                }
                let expected_conflict = record.action == Update::Assert
                    && previous.is_some_and(|r| r.value != record.value);
                if record.conflict != expected_conflict {
                    return Err(bad(
                        "record conflict flag disagrees with its predecessor/action",
                    ));
                }
                if record.evicted != (position < ids.len().saturating_sub(self.capacity)) {
                    return Err(bad("retained payloads violate the declared chain capacity"));
                }
                previous = Some(record);
            }
        }
        if covered.len() != self.records.len() {
            return Err(bad("store contains an orphan record"));
        }
        Ok(())
    }
}
