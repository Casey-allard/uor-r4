//! Scoped, versioned conversation memory with learned ingestion intent.
//!
//! Three responsibilities are deliberately separated, because conflating them is what lets a
//! syntactically confident stale sentence decide what is true.
//!
//! * **Learned observation and intent.** The retained joint ordinary-form decoder extracts the
//!   subject span, cue span, relation-bearing cue role and optional object span from observed text.
//!   A separately fitted intent table reads the cue span for statement intent
//!   (`Assert`, `Correct`, `NonAsserting`) or question intent (`Current`, `Previous`, `Initial`).
//!   No gold field, host dictionary, supplied correction flag or preselected fact reaches this path.
//! * **Exact versioned storage.** Records are addressed by an injective length-delimited
//!   `(scope, entity lexical key, relation)` key and carry an exact id, predecessor, global commit,
//!   source id, owned payload and learned update action. Version order, equality, serialization and
//!   predecessor traversal are deterministic infrastructure and are never approximated.
//! * **Causal read and complete emission.** One `ScopedMemoryRuntime::step` selects an eligible
//!   record under a single pinned commit view, then reuses the retained continuation vocabulary and
//!   exact owned emission.
//!
//! # Declared semantics, fixed before fitting
//!
//! * **Assert** introduces a record for `(scope, entity, relation)`. If the value equals the current
//!   head's value this is a **same-value reassertion**: a new record is appended with the same value,
//!   so history is preserved and never silently deduplicated. If the value differs from the current
//!   head, the bare assertion becomes the new current record **and is marked `conflict`**, because a
//!   bare present assertion claims the present state of the world while still being distinguishable
//!   from an explicit revision.
//! * **Correct** (explicit correction language) supersedes with the stated value and is never marked
//!   `conflict`. A correction that restates the current value still appends a revision.
//! * **`previous`** means the previous *assertion*: the immediate predecessor in the chain, including
//!   a same-value reassertion. The previous *distinct value* is a separate exact query
//!   (`HistoryView::PreviousDistinctValue`) and is never substituted for it, because deduplicating
//!   reassertions would destroy the history the distinction exists to preserve.
//! * **`initial`** is the earliest record in the chain. `previous` and `initial` are exact chain
//!   operations; they are never obtained by widening a word list.
//! * A **historical dependent request** selects the first hop with the requested `HistoryView`;
//!   every later hop resolves as `HistoryView::Current` **at the same pinned view**. `previous` is
//!   deliberately not re-applied independently at every hop.
//! * The **read view** is pinned by `ScopedMemoryRuntime::ask`. A correction committed after the
//!   session started is invisible to it, so an answer can never splice two committed versions.
//! * At declared chain **capacity** the oldest record is marked evicted and its owned payload is
//!   released. Traversal that requires it returns the typed `ScopedTerminal::Evicted`, and an
//!   unfollowed link is never reported as absence.
#![forbid(unsafe_code)]

use super::observed_text_session::{
    candidate_features, lexical_key, observe_clause, Clause, Observation, ObservedTextModel,
};
use super::realtext_support::sha256_hex;
use super::relational_session::RelAction;

/// Classified failures at the scoped-memory boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScopedMemoryError {
    Model(String),
    Intent(String),
    Observation(String),
    Store(String),
    Session(String),
    Serialization(String),
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

/// The intent format is unchanged; store/session v2 adds immutable payload/history witnesses.
pub const SCOPED_VERSION: u8 = 1;
pub const SCOPED_STORE_VERSION: u8 = 2;
pub const SCOPED_SESSION_VERSION: u8 = 2;
/// Statement intents: assert, explicit correction, declared nonasserting.
pub const N_STMT_INTENT: usize = 3;
/// Learned statement intent indices.
pub const STMT_ASSERT: usize = 0;
pub const STMT_CORRECT: usize = 1;
pub const STMT_NONASSERTING: usize = 2;
/// Learned question intents: current, previous assertion, initial.
pub const N_ASK_INTENT: usize = 3;
pub const ASK_CURRENT: usize = 0;
pub const ASK_PREVIOUS: usize = 1;
pub const ASK_INITIAL: usize = 2;
/// Declared default retained chain length per `(scope, entity, relation)`.
pub const DEFAULT_CHAIN_CAPACITY: usize = 8;
/// Canonical address the unscoped control collapses every scope onto.
pub const UNSCOPED_MARKER: &[u8] = b"*";

/// Declared hop bound for one dependent answer.
pub const SCOPED_MAX_HOPS: u8 = 6;
/// Declared answer-token bound.
pub const SCOPED_MAX_ANSWER: usize = 64;

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

/// The learned intent model. Both tables score the cue span's ordered local context through the same
/// retained extractor serving uses, so no gold field can become an input feature.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntentModel {
    pub version: u8,
    pub statement: Vec<(u64, [i32; N_STMT_INTENT])>,
    pub question: Vec<(u64, [i32; N_ASK_INTENT])>,
}

impl IntentModel {
    pub fn uninformed() -> Self {
        IntentModel {
            version: SCOPED_VERSION,
            statement: Vec::new(),
            question: Vec::new(),
        }
    }

    fn wide_statement(&self, tokens: &[u32], start: usize, len: usize) -> [i64; N_STMT_INTENT] {
        let mut out = [0i64; N_STMT_INTENT];
        for key in candidate_features(tokens, start, len) {
            if let Ok(i) = self.statement.binary_search_by_key(&key, |(k, _)| *k) {
                let row = self.statement[i].1;
                for r in 0..N_STMT_INTENT {
                    out[r] += i64::from(row[r]);
                }
            }
        }
        out
    }

    fn wide_question(&self, tokens: &[u32], start: usize, len: usize) -> [i64; N_ASK_INTENT] {
        let mut out = [0i64; N_ASK_INTENT];
        for key in candidate_features(tokens, start, len) {
            if let Ok(i) = self.question.binary_search_by_key(&key, |(k, _)| *k) {
                let row = self.question[i].1;
                for r in 0..N_ASK_INTENT {
                    out[r] += i64::from(row[r]);
                }
            }
        }
        out
    }

    /// The learned statement intent of an observed cue span.
    pub fn statement_intent(&self, tokens: &[u32], start: usize, len: usize) -> usize {
        let scores = self.wide_statement(tokens, start, len);
        (0..N_STMT_INTENT)
            .max_by_key(|r| (scores[*r], std::cmp::Reverse(*r)))
            .unwrap_or(0)
    }

    /// The learned question intent of an observed cue span.
    pub fn question_intent(&self, tokens: &[u32], start: usize, len: usize) -> usize {
        let scores = self.wide_question(tokens, start, len);
        (0..N_ASK_INTENT)
            .max_by_key(|r| (scores[*r], std::cmp::Reverse(*r)))
            .unwrap_or(0)
    }

    pub fn validate(&self, max_vocab: usize) -> Result<(), ScopedMemoryError> {
        if self.version != SCOPED_VERSION {
            return Err(ScopedMemoryError::Intent(
                "unsupported intent model version".into(),
            ));
        }
        if max_vocab == 0 || max_vocab > super::observed_text_session::OB_MAX_VOCAB {
            return Err(ScopedMemoryError::Intent(
                "vocabulary exceeds the 16-bit feature domain".into(),
            ));
        }
        for table in [&self.statement, &self.question] {
            if table.windows(2).any(|w| w[0].0 >= w[1].0) {
                return Err(ScopedMemoryError::Intent(
                    "intent weights must have unique sorted keys".into(),
                ));
            }
        }
        for (key, _) in self.statement.iter().chain(self.question.iter()) {
            let kind = key >> 40;
            let value = key & ((1u64 << 40) - 1);
            let ok = match kind {
                1..=6 => value < max_vocab as u64,
                7 => value > 0 && value <= super::observed_text_session::OB_MAX_MARKER as u64,
                8 | 9 => value == 0,
                _ => false,
            };
            if !ok {
                return Err(ScopedMemoryError::Intent(
                    "intent feature is outside the retained extractor domain".into(),
                ));
            }
        }
        Ok(())
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, ScopedMemoryError> {
        serde_json::to_vec(self).map_err(|e| ScopedMemoryError::Serialization(e.to_string()))
    }

    pub fn from_bytes(bytes: &[u8], max_vocab: usize) -> Result<Self, ScopedMemoryError> {
        let model: Self = serde_json::from_slice(bytes)
            .map_err(|e| ScopedMemoryError::Serialization(e.to_string()))?;
        model.validate(max_vocab)?;
        Ok(model)
    }
}

/// Declared supervision for one development cue span.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IntentExample {
    pub question: bool,
    pub intent: usize,
}

/// Fit report for the learned intent tables.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct IntentFitReport {
    pub examples: usize,
    pub statement_weights: usize,
    pub question_weights: usize,
    pub statement_initial_correct: usize,
    pub statement_final_correct: usize,
    pub question_initial_correct: usize,
    pub question_final_correct: usize,
    pub epochs_run: usize,
    pub updates: usize,
}

/// Fit both intent tables from declared development clauses and their supervision. The learner is the
/// same bounded candidate perceptron used for the retained role scorer, with i64 accumulation and no
/// premature clipping; the declared uninformed start has no weights at all.
pub fn fit_intent_model(
    clauses: &[Clause],
    cue_spans: &[(usize, usize)],
    examples: &[IntentExample],
    max_vocab: usize,
) -> Result<(IntentModel, IntentFitReport), ScopedMemoryError> {
    IntentModel::uninformed().validate(max_vocab)?;
    if clauses.len() != examples.len() || clauses.len() != cue_spans.len() {
        return Err(ScopedMemoryError::Intent(
            "intent supervision does not align with its clauses".into(),
        ));
    }
    for (clause, span) in clauses.iter().zip(cue_spans.iter()) {
        if clause.tokens.iter().any(|t| *t as usize >= max_vocab) {
            return Err(ScopedMemoryError::Intent(
                "intent clause token is outside the declared feature domain".into(),
            ));
        }
        if span.1 == 0
            || span.1 > super::observed_text_session::OB_MAX_MARKER
            || span
                .0
                .checked_add(span.1)
                .is_none_or(|end| end > clause.tokens.len())
        {
            return Err(ScopedMemoryError::Intent(
                "intent cue span is outside the declared candidate bound".into(),
            ));
        }
    }
    for example in examples {
        let bound = if example.question {
            N_ASK_INTENT
        } else {
            N_STMT_INTENT
        };
        if example.intent >= bound {
            return Err(ScopedMemoryError::Intent(
                "intent supervision is out of range".into(),
            ));
        }
    }
    let mut model = IntentModel::uninformed();
    let exact = |model: &IntentModel| -> (usize, usize) {
        let mut statement = 0usize;
        let mut question = 0usize;
        for ((clause, span), example) in clauses.iter().zip(cue_spans.iter()).zip(examples.iter()) {
            let predicted = if example.question {
                model.question_intent(&clause.tokens, span.0, span.1)
            } else {
                model.statement_intent(&clause.tokens, span.0, span.1)
            };
            if predicted == example.intent {
                if example.question {
                    question += 1;
                } else {
                    statement += 1;
                }
            }
        }
        (statement, question)
    };
    let (statement_initial_correct, question_initial_correct) = exact(&model);
    let mut updates = 0usize;
    let mut epochs_run = 0usize;
    let mut best = (statement_initial_correct, question_initial_correct);
    let mut best_model = model.clone();
    for _epoch in 0..24 {
        epochs_run += 1;
        let mut changed = false;
        for ((clause, span), example) in clauses.iter().zip(cue_spans.iter()).zip(examples.iter()) {
            let predicted = if example.question {
                model.question_intent(&clause.tokens, span.0, span.1)
            } else {
                model.statement_intent(&clause.tokens, span.0, span.1)
            };
            if predicted == example.intent {
                continue;
            }
            let keys = candidate_features(&clause.tokens, span.0, span.1);
            for key in keys {
                if example.question {
                    add_intent_row(&mut model.question, key, predicted, example.intent);
                } else {
                    add_intent_row(&mut model.statement, key, predicted, example.intent);
                }
            }
            updates += 1;
            changed = true;
        }
        let value = exact(&model);
        if value.0 + value.1 > best.0 + best.1 {
            best = value;
            best_model = model.clone();
        }
        if !changed {
            break;
        }
    }
    model = best_model;
    let (statement_final_correct, question_final_correct) = exact(&model);
    let report = IntentFitReport {
        examples: examples.len(),
        statement_weights: model.statement.len(),
        question_weights: model.question.len(),
        statement_initial_correct,
        statement_final_correct,
        question_initial_correct,
        question_final_correct,
        epochs_run,
        updates,
    };
    Ok((model, report))
}

fn add_intent_row<const N: usize>(
    table: &mut Vec<(u64, [i32; N])>,
    key: u64,
    predicted: usize,
    gold: usize,
) {
    match table.binary_search_by_key(&key, |(k, _)| *k) {
        Ok(i) => {
            table[i].1[gold] = table[i].1[gold].saturating_add(1);
            table[i].1[predicted] = table[i].1[predicted].saturating_sub(1);
        }
        Err(i) => {
            let mut row = [0i32; N];
            row[gold] = row[gold].saturating_add(1);
            row[predicted] = row[predicted].saturating_sub(1);
            table.insert(i, (key, row));
        }
    }
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

/// Declared experimental controls. They are immutable runtime inputs and part of snapshot identity.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MemoryControl {
    #[default]
    Normal,
    /// Reads disabled: the first read returns a typed no-read terminal.
    NoRead,
    /// Writes rejected: ingestion adds no record.
    UpdateDisabled,
    /// Ignore the pinned view and use the latest committed record (splicing control).
    Unpinned,
    /// Drop the scope from the address (cross-scope contamination control).
    Unscoped,
    /// Choose the highest source parse score instead of the committed head.
    ParseScoreAuthority,
}

/// Immutable identities prepared once from the loaded bytes and store.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemoryBinding {
    pub model_sha256: String,
    pub intent_sha256: String,
    pub lineage: u64,
    pub control_sha256: String,
    pub capacity: usize,
    pub max_vocab: usize,
    pub eos: Option<u32>,
}

/// Exact owned payload captured for one record.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemoryCapture {
    pub key: Vec<u8>,
    pub record_id: u64,
    pub commit: u64,
    pub value: Vec<u8>,
    pub payload: Vec<u32>,
    pub continues: bool,
}

/// Typed terminal reason for one answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ScopedTerminal {
    /// A complete owned answer was emitted.
    Complete,
    /// No record is eligible for a required step at the pinned view.
    Unresolved,
    /// The declared hop or answer-token bound was reached.
    Exhausted,
    /// The requested historical position is not present in the retained chain.
    NoHistory,
    /// The required record exists but its owned payload was released under declared capacity.
    Evicted,
    /// Reads are disabled by the bound control.
    NoRead,
    /// A dependent read revisited an address it had already followed.
    Cycle,
}

/// One answer session, pinned to one committed view.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScopedSession {
    pub version: u8,
    pub binding: MemoryBinding,
    pub view: u64,
    /// Immutable committed-history witness, independent of physical order and later appends.
    pub view_sha256: String,
    pub scope: Vec<u8>,
    pub relation: u8,
    pub history: HistoryView,
    pub hop: u8,
    pub query_entity: Vec<u8>,
    pub captured: Option<MemoryCapture>,
    pub pending: RelAction,
    pub emitted: Vec<u32>,
    pub cursor: usize,
    pub visited: Vec<Vec<u8>>,
    pub terminal: Option<ScopedTerminal>,
    pub eos: Option<u32>,
}

/// Performed action and its next phase, with the exact selected record.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ScopedStepEffect {
    pub action: RelAction,
    pub next_action: Option<RelAction>,
    pub selected_record: Option<u64>,
    pub selected_commit: Option<u64>,
    pub selected_value: Option<Vec<u8>>,
    pub hop: u8,
    pub emitted: Option<u32>,
    pub terminal: Option<ScopedTerminal>,
}

impl ScopedSession {
    /// The captured payload survives replacement; liveness of its origin is a separate question.
    pub fn origin_is_live(&self, memory: &Memory) -> bool {
        let Some(capture) = &self.captured else {
            return false;
        };
        if memory.lineage != self.binding.lineage
            || memory.history_sha256(self.view).ok().as_deref() != Some(self.view_sha256.as_str())
        {
            return false;
        }
        // IDs are scoped to committed history; a foreign store can reuse the same integer ID.
        match memory.head(&capture.key) {
            Lookup::Found(record) => {
                record.id == capture.record_id
                    && record.commit == capture.commit
                    && encode_key(&record.scope, &record.entity, record.relation) == capture.key
                    && record.value == capture.value
                    && record.continues == capture.continues
                    && record.payload == capture.payload
                    && record.payload_sha256 == payload_sha256(&capture.payload)
            }
            _ => false,
        }
    }

    fn captured_payload_len(&self) -> usize {
        self.captured
            .as_ref()
            .map(|capture| capture.payload.len())
            .unwrap_or(0)
    }
}

/// Learned ingestion outcome for one observed clause.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum IngestOutcome {
    /// A record was committed.
    Wrote(Written),
    /// A declared nonasserting input wrote nothing.
    NonAsserting { reason: String },
    /// The bound control rejected the write.
    Rejected { reason: String },
}

/// The learned reading of one observed clause, without any write.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct IngestObservation {
    pub is_question: bool,
    pub intent: usize,
    pub relation: u8,
    pub role: usize,
    pub parse_score: i64,
    pub entity_key: Vec<u8>,
    pub value_key: Option<Vec<u8>>,
    pub continues: bool,
}

/// A loaded scoped-memory runtime: learned binder, learned intent, exact store and bound control.
pub struct ScopedMemoryRuntime {
    model: ObservedTextModel,
    intent: IntentModel,
    memory: Memory,
    binding: MemoryBinding,
    control: MemoryControl,
    max_vocab: usize,
}

fn raw_span_key(clause: &Clause, start: u32, len: usize) -> Result<Vec<u8>, ScopedMemoryError> {
    if len == 0 {
        return Err(ScopedMemoryError::Observation(
            "declared span is empty".into(),
        ));
    }
    lexical_key(&clause.text, &clause.byte_lengths, start as usize, len).ok_or_else(|| {
        ScopedMemoryError::Observation(
            "clause has no exact byte alignment for a lexical key".into(),
        )
    })
}

impl ScopedMemoryRuntime {
    #[allow(clippy::too_many_arguments)]
    pub fn load(
        model_bytes: &[u8],
        intent_bytes: &[u8],
        memory: Memory,
        lineage: u64,
        control: MemoryControl,
        max_vocab: usize,
        eos: Option<u32>,
    ) -> Result<Self, ScopedMemoryError> {
        if memory.lineage != lineage {
            return Err(ScopedMemoryError::Store(
                "store lineage disagrees with the declared runtime lineage".into(),
            ));
        }
        let model = ObservedTextModel::from_bytes(model_bytes, max_vocab)
            .map_err(|e| ScopedMemoryError::Model(e.to_string()))?;
        let intent = IntentModel::from_bytes(intent_bytes, max_vocab)?;
        memory.validate()?;
        // EOS is an explicitly bound protocol terminator and may be outside the lexical vocabulary.
        // Owned source payloads must still consist entirely of valid lexical token IDs.
        if memory
            .records
            .iter()
            .any(|r| r.payload.iter().any(|t| *t as usize >= max_vocab))
        {
            return Err(ScopedMemoryError::Store(
                "stored token outside vocabulary".into(),
            ));
        }
        let binding = MemoryBinding {
            model_sha256: sha256_hex(model_bytes),
            intent_sha256: sha256_hex(intent_bytes),
            lineage,
            control_sha256: sha256_hex(
                &serde_json::to_vec(&control)
                    .map_err(|e| ScopedMemoryError::Serialization(e.to_string()))?,
            ),
            capacity: memory.capacity,
            max_vocab,
            eos,
        };
        Ok(Self {
            model,
            intent,
            memory,
            binding,
            control,
            max_vocab,
        })
    }

    pub fn model(&self) -> &ObservedTextModel {
        &self.model
    }
    pub fn intent(&self) -> &IntentModel {
        &self.intent
    }
    pub fn memory(&self) -> &Memory {
        &self.memory
    }
    pub fn binding(&self) -> &MemoryBinding {
        &self.binding
    }
    pub fn control(&self) -> &MemoryControl {
        &self.control
    }
    pub fn max_vocab(&self) -> usize {
        self.max_vocab
    }
    /// The latest committed revision.
    pub fn revision(&self) -> u64 {
        self.memory.commit
    }
    /// The exact store bytes, for save/reload.
    pub fn store_bytes(&self) -> Result<Vec<u8>, ScopedMemoryError> {
        self.memory.to_bytes()
    }
    pub fn intent_bytes(&self) -> Result<Vec<u8>, ScopedMemoryError> {
        self.intent.to_bytes()
    }
    /// Rebuild a runtime over a different store/control, preserving the loaded model and intent.
    pub fn with_memory(
        &self,
        memory: Memory,
        control: MemoryControl,
    ) -> Result<Self, ScopedMemoryError> {
        let model_bytes = self
            .model
            .to_bytes()
            .map_err(|e| ScopedMemoryError::Model(e.to_string()))?;
        let intent_bytes = self.intent.to_bytes()?;
        Self::load(
            &model_bytes,
            &intent_bytes,
            memory,
            self.binding.lineage,
            control,
            self.max_vocab,
            self.binding.eos,
        )
    }

    fn address(&self, scope: &[u8], entity: &[u8], relation: u8) -> Vec<u8> {
        match self.control {
            MemoryControl::Unscoped => encode_key(UNSCOPED_MARKER, entity, relation),
            _ => encode_key(scope, entity, relation),
        }
    }

    /// Read one clause through the loaded binder and intent model without writing anything.
    pub fn observe(&self, clause: &Clause) -> Result<IngestObservation, ScopedMemoryError> {
        let observation: Observation = observe_clause(&self.model, clause)
            .map_err(|e| ScopedMemoryError::Observation(e.to_string()))?;
        let relation = self
            .model
            .role_goal(observation.role)
            .ok_or_else(|| {
                ScopedMemoryError::Observation("cue role has no learned relation".into())
            })?
            .index() as u8;
        let entity_key =
            raw_span_key(clause, observation.subject_start, observation.subject.len())?;
        let is_question = observation.object.is_none();
        let intent = if is_question {
            self.intent.question_intent(
                &clause.tokens,
                observation.marker_start as usize,
                observation.marker.len(),
            )
        } else {
            self.intent.statement_intent(
                &clause.tokens,
                observation.marker_start as usize,
                observation.marker.len(),
            )
        };
        let value_key = match &observation.object {
            Some(object) => Some(raw_span_key(
                clause,
                observation.object_start.ok_or_else(|| {
                    ScopedMemoryError::Observation("object span has no start".into())
                })?,
                object.len(),
            )?),
            None => None,
        };
        Ok(IngestObservation {
            is_question,
            intent,
            relation,
            role: observation.role,
            parse_score: observation.score,
            entity_key,
            value_key,
            continues: self.model.role_is_redirect(observation.role),
        })
    }

    /// Learn intent from an observed clause and, only for an asserting intent, commit one record.
    pub fn ingest(
        &mut self,
        clause: &Clause,
        scope: &[u8],
        source: u64,
    ) -> Result<IngestOutcome, ScopedMemoryError> {
        let payload = observe_clause(&self.model, clause)
            .map_err(|e| ScopedMemoryError::Observation(e.to_string()))?
            .object;
        let learned = self.observe(clause)?;
        if learned.is_question {
            return Ok(IngestOutcome::NonAsserting {
                reason: "observed clause is a question".into(),
            });
        }
        let action = match learned.intent {
            STMT_ASSERT => Update::Assert,
            STMT_CORRECT => Update::Correct,
            STMT_NONASSERTING => {
                return Ok(IngestOutcome::NonAsserting {
                    reason: "learned nonasserting statement intent".into(),
                })
            }
            _ => {
                return Err(ScopedMemoryError::Intent(
                    "statement intent is out of range".into(),
                ))
            }
        };
        if matches!(self.control, MemoryControl::UpdateDisabled) {
            return Ok(IngestOutcome::Rejected {
                reason: "writes are disabled by the bound control".into(),
            });
        }
        let value = learned.value_key.ok_or_else(|| {
            ScopedMemoryError::Observation("asserting clause has no observed value".into())
        })?;
        let payload = payload.ok_or_else(|| {
            ScopedMemoryError::Observation("asserting clause has no payload".into())
        })?;
        // The unscoped control collapses writes and reads onto one canonical marker address, so two
        // scopes really do alias; a read-only collapse would only produce a uniform absence, and an
        // empty address would be rejected rather than silently aliasing every scope.
        let address_scope: &[u8] = match self.control {
            MemoryControl::Unscoped => UNSCOPED_MARKER,
            _ => scope,
        };
        let written = self.memory.write(
            address_scope,
            &learned.entity_key,
            learned.relation,
            &value,
            &payload,
            source,
            action,
            learned.continues,
            learned.parse_score,
        )?;
        Ok(IngestOutcome::Wrote(written))
    }

    /// Start an answer. The read view is pinned to the commit current at this moment.
    pub fn ask(&self, clause: &Clause, scope: &[u8]) -> Result<ScopedSession, ScopedMemoryError> {
        let observed = self.observe(clause)?;
        if !observed.is_question {
            return Err(ScopedMemoryError::Observation(
                "observed clause is not a question".into(),
            ));
        }
        let history = HistoryView::from_question_intent(observed.intent).ok_or_else(|| {
            ScopedMemoryError::Intent("learned question intent is out of range".into())
        })?;
        self.ask_view(scope, observed.relation, &observed.entity_key, history)
    }

    /// Start an answer for an explicit exact view. Used by the harness for the historical semantics
    /// that are not reachable from learned question intent.
    pub fn ask_view(
        &self,
        scope: &[u8],
        relation: u8,
        entity: &[u8],
        history: HistoryView,
    ) -> Result<ScopedSession, ScopedMemoryError> {
        if scope.is_empty() || entity.is_empty() {
            return Err(ScopedMemoryError::Session(
                "scope and query entity must be nonempty".into(),
            ));
        }
        let session = ScopedSession {
            version: SCOPED_SESSION_VERSION,
            binding: self.binding.clone(),
            view: self.memory.commit,
            view_sha256: self.memory.history_sha256(self.memory.commit)?,
            scope: scope.to_vec(),
            relation,
            history,
            hop: 0,
            query_entity: entity.to_vec(),
            captured: None,
            pending: RelAction::Read,
            emitted: Vec::new(),
            cursor: 0,
            visited: Vec::new(),
            terminal: None,
            eos: self.binding.eos,
        };
        self.validate(&session)?;
        Ok(session)
    }

    pub fn validate(&self, session: &ScopedSession) -> Result<(), ScopedMemoryError> {
        if session.version != SCOPED_SESSION_VERSION || session.binding != self.binding {
            return Err(ScopedMemoryError::Session(
                "session version or runtime binding mismatch".into(),
            ));
        }
        if session.view > self.memory.commit
            || session.eos != self.binding.eos
            || session.view_sha256 != self.memory.history_sha256(session.view)?
        {
            return Err(ScopedMemoryError::Session(
                "session view is outside the committed history".into(),
            ));
        }
        if session.scope.is_empty() || session.query_entity.is_empty() {
            return Err(ScopedMemoryError::Session("empty session address".into()));
        }
        if session.hop > SCOPED_MAX_HOPS
            || session.cursor > session.captured_payload_len()
            || session.emitted.len() > SCOPED_MAX_ANSWER
            || session.visited.len() != session.hop as usize
            || session
                .visited
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != session.visited.len()
        {
            return Err(ScopedMemoryError::Session(
                "invalid hop, cursor or visited history".into(),
            ));
        }
        match &session.captured {
            Some(capture) => {
                let record = self.memory.record_ref(capture.record_id).ok_or_else(|| {
                    ScopedMemoryError::Session("capture references a missing record".into())
                })?;
                let beyond_pin = capture.commit > session.view
                    && !matches!(self.control, MemoryControl::Unpinned);
                let failure = session.terminal.is_some()
                    && session.terminal != Some(ScopedTerminal::Complete);
                let follows_capture = session.pending == RelAction::Read || failure;
                let (captured_entity, capture_hop) = if follows_capture {
                    let entity = session.visited.last().ok_or_else(|| {
                        ScopedMemoryError::Session(
                            "continued capture lacks a visited source".into(),
                        )
                    })?;
                    if !capture.continues || session.query_entity != capture.value {
                        return Err(ScopedMemoryError::Session(
                            "continued capture/value disagree".into(),
                        ));
                    }
                    (
                        entity,
                        session.hop.checked_sub(1).ok_or_else(|| {
                            ScopedMemoryError::Session("continued capture has no hop".into())
                        })?,
                    )
                } else {
                    (&session.query_entity, session.hop)
                };
                if capture.payload.is_empty()
                    || capture.payload.len() + usize::from(session.eos.is_some())
                        > SCOPED_MAX_ANSWER
                    || capture
                        .payload
                        .iter()
                        .any(|token| *token as usize >= self.max_vocab)
                    || capture.key
                        != self.address(&session.scope, captured_entity, session.relation)
                    || capture.key != encode_key(&record.scope, &record.entity, record.relation)
                    || capture.commit != record.commit
                    || beyond_pin
                    || capture.value != record.value
                    || capture.continues != record.continues
                    || payload_sha256(&capture.payload) != record.payload_sha256
                    || (!record.evicted && capture.payload != record.payload)
                {
                    return Err(ScopedMemoryError::Session(
                        "capture disagrees with its exact owned record".into(),
                    ));
                }
                // Pinned exact selection remains stable even when a payload was later evicted.
                if matches!(
                    self.control,
                    MemoryControl::Normal | MemoryControl::UpdateDisabled | MemoryControl::Unscoped
                ) {
                    let history = if capture_hop == 0 {
                        session.history
                    } else {
                        HistoryView::Current
                    };
                    if !matches!(self.memory.lookup_identity(&capture.key, session.view, history),
                        Lookup::Found(selected) if selected.id == capture.record_id)
                    {
                        return Err(ScopedMemoryError::Session(
                            "capture is not eligible at its pinned view".into(),
                        ));
                    }
                }
                let mut expected = capture.payload[..session.cursor].to_vec();
                if session.terminal == Some(ScopedTerminal::Complete) {
                    if let Some(eos) = session.eos {
                        expected.push(eos);
                    }
                }
                if session.emitted != expected {
                    return Err(ScopedMemoryError::Session(
                        "emissions disagree with the owned payload prefix/EOS".into(),
                    ));
                }
            }
            None => {
                if session.cursor != 0 || !session.emitted.is_empty() || session.hop != 0 {
                    return Err(ScopedMemoryError::Session(
                        "session has progress without an owned capture".into(),
                    ));
                }
            }
        }
        let terminators = usize::from(
            session.terminal == Some(ScopedTerminal::Complete) && session.eos.is_some(),
        );
        match session.terminal {
            Some(ScopedTerminal::Complete) => {
                if session.pending != RelAction::Stop
                    || !session.captured.as_ref().is_some_and(|c| !c.continues)
                    || session.cursor == 0
                    || session.cursor != session.captured_payload_len()
                    || session.emitted.len() != session.cursor + terminators
                {
                    return Err(ScopedMemoryError::Session(
                        "complete terminal requires the full owned payload".into(),
                    ));
                }
            }
            Some(_) => {
                if session.pending != RelAction::Stop || !session.emitted.is_empty() {
                    return Err(ScopedMemoryError::Session(
                        "typed failure terminal must be silent".into(),
                    ));
                }
            }
            None => match session.pending {
                RelAction::Read if session.cursor == 0 && session.emitted.is_empty() => {}
                RelAction::Continue
                    if session.captured.as_ref().is_some_and(|c| c.continues)
                        && session.cursor == 0
                        && session.emitted.is_empty() => {}
                RelAction::Emit
                    if session.captured.as_ref().is_some_and(|c| !c.continues)
                        && session.cursor < session.captured_payload_len() => {}
                RelAction::Stop
                    if session.captured.as_ref().is_some_and(|c| !c.continues)
                        && session.cursor > 0
                        && session.cursor == session.captured_payload_len() => {}
                _ => {
                    return Err(ScopedMemoryError::Session(
                        "invalid active session phase".into(),
                    ))
                }
            },
        }
        Ok(())
    }

    fn terminate(&self, session: &mut ScopedSession, reason: ScopedTerminal) {
        session.terminal = Some(reason);
        session.pending = RelAction::Stop;
    }

    /// One causal step. Generation, resumption and the controls all use this single implementation.
    pub fn step(&self, session: &mut ScopedSession) -> Result<ScopedStepEffect, ScopedMemoryError> {
        self.validate(session)?;
        if session.terminal.is_some() {
            return Err(ScopedMemoryError::Session(
                "session already terminal".into(),
            ));
        }
        let performed = session.pending;
        let mut effect = ScopedStepEffect {
            action: performed,
            next_action: None,
            selected_record: None,
            selected_commit: None,
            selected_value: None,
            hop: session.hop,
            emitted: None,
            terminal: None,
        };
        match performed {
            RelAction::Read => {
                if matches!(self.control, MemoryControl::NoRead) {
                    self.terminate(session, ScopedTerminal::NoRead);
                } else {
                    let view = match self.control {
                        MemoryControl::Unpinned => self.memory.commit,
                        _ => session.view,
                    };
                    let history = if session.hop == 0 {
                        session.history
                    } else {
                        HistoryView::Current
                    };
                    let key = self.address(&session.scope, &session.query_entity, session.relation);
                    if session.visited.iter().any(|v| v == &session.query_entity) {
                        self.terminate(session, ScopedTerminal::Cycle);
                    } else {
                        match self.select(&key, view, history) {
                            Lookup::Found(record) => {
                                if session.hop >= SCOPED_MAX_HOPS {
                                    self.terminate(session, ScopedTerminal::Exhausted);
                                } else if record.payload.is_empty() {
                                    self.terminate(session, ScopedTerminal::Evicted);
                                } else if record.payload.len() + usize::from(session.eos.is_some())
                                    > SCOPED_MAX_ANSWER
                                {
                                    // Reject before any token escapes; typed failure terminals are silent.
                                    self.terminate(session, ScopedTerminal::Exhausted);
                                } else {
                                    effect.selected_record = Some(record.id);
                                    effect.selected_commit = Some(record.commit);
                                    effect.selected_value = Some(record.value.clone());
                                    let capture = MemoryCapture {
                                        key,
                                        record_id: record.id,
                                        commit: record.commit,
                                        value: record.value.clone(),
                                        payload: record.payload.clone(),
                                        continues: record.continues,
                                    };
                                    let continues = capture.continues;
                                    session.captured = Some(capture);
                                    session.pending = if continues {
                                        RelAction::Continue
                                    } else {
                                        RelAction::Emit
                                    };
                                }
                            }
                            Lookup::Absent => self.terminate(session, ScopedTerminal::Unresolved),
                            Lookup::NoHistory => self.terminate(session, ScopedTerminal::NoHistory),
                            Lookup::Evicted => self.terminate(session, ScopedTerminal::Evicted),
                        }
                    }
                }
            }
            RelAction::Continue => {
                let (value, continues) = match &session.captured {
                    Some(capture) => (capture.value.clone(), capture.continues),
                    None => {
                        return Err(ScopedMemoryError::Session(
                            "continue without a capture".into(),
                        ))
                    }
                };
                if !continues {
                    return Err(ScopedMemoryError::Session(
                        "continue from a terminal value".into(),
                    ));
                }
                // A followed address is recorded once per hop, so the visited count equals the hop
                // count and a repeated address is detectable at the next read.
                session.visited.push(session.query_entity.clone());
                session.query_entity = value;
                session.hop += 1;
                session.pending = RelAction::Read;
            }
            RelAction::Emit => {
                let (token, done) = match &session.captured {
                    Some(capture) => {
                        let token = *capture.payload.get(session.cursor).ok_or_else(|| {
                            ScopedMemoryError::Session("emission past the payload".into())
                        })?;
                        (token, session.cursor + 1 == capture.payload.len())
                    }
                    None => {
                        return Err(ScopedMemoryError::Session("emit without a capture".into()))
                    }
                };
                if session.emitted.len() >= SCOPED_MAX_ANSWER {
                    self.terminate(session, ScopedTerminal::Exhausted);
                } else {
                    session.emitted.push(token);
                    session.cursor += 1;
                    effect.emitted = Some(token);
                    if done {
                        session.pending = RelAction::Stop;
                    }
                }
            }
            RelAction::Stop => {
                let mut exhausted = false;
                if let Some(eos) = session.eos {
                    if session.emitted.len() >= SCOPED_MAX_ANSWER {
                        exhausted = true;
                    } else {
                        session.emitted.push(eos);
                        effect.emitted = Some(eos);
                    }
                }
                if exhausted {
                    self.terminate(session, ScopedTerminal::Exhausted);
                } else {
                    self.terminate(session, ScopedTerminal::Complete);
                }
            }
            _ => {
                return Err(ScopedMemoryError::Session(
                    "unsupported active scoped phase".into(),
                ))
            }
        }
        self.validate(session)?;
        effect.next_action = if session.terminal.is_none() {
            Some(session.pending)
        } else {
            None
        };
        effect.terminal = session.terminal;
        effect.hop = session.hop;
        Ok(effect)
    }

    /// Selection under one view and control. The committed version decides; a parse score decides only
    /// in the named control.
    fn select<'a>(&'a self, key: &[u8], view: u64, history: HistoryView) -> Lookup<'a> {
        if matches!(self.control, MemoryControl::ParseScoreAuthority) {
            let Some(index) = self.memory.chain_index(key) else {
                return Lookup::Absent;
            };
            let mut best: Option<&Record> = None;
            for id in &self.memory.chains[index].1 {
                let Some(record) = self.memory.record_ref(*id) else {
                    continue;
                };
                if record.commit > view || record.evicted {
                    continue;
                }
                if best.is_none_or(|b| record.parse_score > b.parse_score) {
                    best = Some(record);
                }
            }
            return best.map_or(Lookup::Absent, Lookup::Found);
        }
        self.memory.lookup(key, view, history)
    }

    /// Run to a terminal under the declared bounds.
    pub fn run(
        &self,
        session: &mut ScopedSession,
    ) -> Result<Vec<ScopedStepEffect>, ScopedMemoryError> {
        self.validate(session)?;
        let bound = SCOPED_MAX_HOPS as usize * 2 + SCOPED_MAX_ANSWER + 2;
        let mut effects = Vec::new();
        for _ in 0..bound {
            if session.terminal.is_some() {
                return Ok(effects);
            }
            effects.push(self.step(session)?);
        }
        if session.terminal.is_none() {
            self.terminate(session, ScopedTerminal::Exhausted);
            self.validate(session)?;
            effects.push(ScopedStepEffect {
                action: RelAction::Exhausted,
                next_action: None,
                selected_record: None,
                selected_commit: None,
                selected_value: None,
                hop: session.hop,
                emitted: None,
                terminal: session.terminal,
            });
        }
        Ok(effects)
    }

    pub fn snapshot(&self, session: &ScopedSession) -> Result<Vec<u8>, ScopedMemoryError> {
        self.validate(session)?;
        serde_json::to_vec(session).map_err(|e| ScopedMemoryError::Serialization(e.to_string()))
    }

    pub fn restore(&self, bytes: &[u8]) -> Result<ScopedSession, ScopedMemoryError> {
        let session: ScopedSession = serde_json::from_slice(bytes)
            .map_err(|e| ScopedMemoryError::Serialization(e.to_string()))?;
        self.validate(&session)?;
        Ok(session)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(memory: &mut Memory, entity: &str, value: &str, action: Update) -> Written {
        memory
            .write(
                b"alpha",
                entity.as_bytes(),
                0,
                value.as_bytes(),
                &[1, 2, 3],
                7,
                action,
                false,
                0,
            )
            .expect("store write")
    }

    fn value_of(lookup: Lookup<'_>) -> Option<String> {
        match lookup {
            Lookup::Found(record) => Some(String::from_utf8_lossy(&record.value).into_owned()),
            _ => None,
        }
    }

    #[test]
    fn key_encoding_is_injective_where_delimiter_concatenation_collides() {
        // A naive `scope/entity` join gives both of these the same string.
        let a = encode_key(b"a", b"b/c", 1);
        let b = encode_key(b"a/b", b"c", 1);
        assert_ne!(a, b);
        assert_eq!(encode_key(b"a", b"b/c", 1), encode_key(b"a", b"b/c", 1));
        assert_ne!(encode_key(b"x", b"y", 0), encode_key(b"x", b"y", 1));
        assert_ne!(
            encode_key(b"x", b"y", 0),
            encode_key(b"x", b"y", 0).len().to_le_bytes().to_vec()
        );
    }

    #[test]
    fn assert_correction_conflict_and_same_value_reassertion_are_distinct() {
        let mut memory = Memory::new(1, 8);
        let first = write(&mut memory, "Ova", "Bramble", Update::Assert);
        assert_eq!(first.revision, 1);
        assert!(!first.conflict);
        assert_eq!(first.superseded, 0);

        // Same-value reassertion: a new revision, no conflict, history preserved.
        let reassert = write(&mut memory, "Ova", "Bramble", Update::Assert);
        assert_eq!(reassert.revision, 2);
        assert!(!reassert.conflict);
        assert_eq!(reassert.superseded, first.id);

        // Contradictory bare assertion becomes current and is marked as a conflict.
        let conflict = write(&mut memory, "Ova", "Quarry", Update::Assert);
        assert!(conflict.conflict);
        assert_eq!(conflict.superseded, reassert.id);

        // An explicit correction is never marked as a conflict.
        let corrected = write(&mut memory, "Ova", "Vale", Update::Correct);
        assert!(!corrected.conflict);

        let key = encode_key(b"alpha", b"Ova", 0);
        assert_eq!(
            value_of(memory.lookup(&key, u64::MAX, HistoryView::Current)).as_deref(),
            Some("Vale")
        );
        // `previous` is the previous assertion: the explicit correction's predecessor, not a distinct value.
        assert_eq!(
            value_of(memory.lookup(&key, u64::MAX, HistoryView::PreviousAssertion)).as_deref(),
            Some("Quarry")
        );
        // The previous *distinct* value skips nothing here because "Quarry" already differs.
        assert_eq!(
            value_of(memory.lookup(&key, u64::MAX, HistoryView::PreviousDistinctValue)).as_deref(),
            Some("Quarry")
        );
        assert_eq!(
            value_of(memory.lookup(&key, u64::MAX, HistoryView::Initial)).as_deref(),
            Some("Bramble")
        );
        memory.validate().expect("store validates");
    }

    #[test]
    fn previous_distinct_skips_reassertions_while_previous_assertion_does_not() {
        let mut memory = Memory::new(1, 8);
        write(&mut memory, "Ova", "Bramble", Update::Assert);
        write(&mut memory, "Ova", "Quarry", Update::Assert);
        // Two same-value reassertions of the current value.
        write(&mut memory, "Ova", "Quarry", Update::Assert);
        write(&mut memory, "Ova", "Quarry", Update::Correct);
        let key = encode_key(b"alpha", b"Ova", 0);
        assert_eq!(
            value_of(memory.lookup(&key, u64::MAX, HistoryView::PreviousAssertion)).as_deref(),
            Some("Quarry")
        );
        assert_eq!(
            value_of(memory.lookup(&key, u64::MAX, HistoryView::PreviousDistinctValue)).as_deref(),
            Some("Bramble")
        );
    }

    #[test]
    fn a_pinned_view_hides_later_commits_and_missing_history_is_typed() {
        let mut memory = Memory::new(1, 8);
        write(&mut memory, "Ova", "Bramble", Update::Assert);
        let view = memory.commit;
        write(&mut memory, "Ova", "Vale", Update::Correct);
        let key = encode_key(b"alpha", b"Ova", 0);
        assert_eq!(
            value_of(memory.lookup(&key, view, HistoryView::Current)).as_deref(),
            Some("Bramble")
        );
        assert_eq!(
            value_of(memory.lookup(&key, u64::MAX, HistoryView::Current)).as_deref(),
            Some("Vale")
        );
        // The pinned view precedes the first commit, so nothing is eligible there.
        assert!(matches!(
            memory.lookup(&key, 0, HistoryView::Current),
            Lookup::Absent
        ));
        // A single-record chain has no previous assertion.
        assert!(matches!(
            memory.lookup(&key, view, HistoryView::PreviousAssertion),
            Lookup::NoHistory
        ));
        // An unknown address is absent, not a history result.
        let unknown = encode_key(b"alpha", b"Unknown", 0);
        assert!(matches!(
            memory.lookup(&unknown, u64::MAX, HistoryView::Current),
            Lookup::Absent
        ));
    }

    #[test]
    fn scopes_and_relations_are_independent_addresses() {
        let mut memory = Memory::new(1, 8);
        memory
            .write(
                b"alpha",
                b"Ova",
                0,
                b"Bramble",
                &[9],
                1,
                Update::Assert,
                false,
                0,
            )
            .unwrap();
        memory
            .write(
                b"beta",
                b"Ova",
                0,
                b"Vale",
                &[8],
                1,
                Update::Assert,
                false,
                0,
            )
            .unwrap();
        memory
            .write(
                b"alpha",
                b"Ova",
                1,
                b"Atlas",
                &[7],
                1,
                Update::Assert,
                false,
                0,
            )
            .unwrap();
        assert_eq!(
            value_of(memory.lookup(
                &encode_key(b"alpha", b"Ova", 0),
                u64::MAX,
                HistoryView::Current
            ))
            .as_deref(),
            Some("Bramble")
        );
        assert_eq!(
            value_of(memory.lookup(
                &encode_key(b"beta", b"Ova", 0),
                u64::MAX,
                HistoryView::Current
            ))
            .as_deref(),
            Some("Vale")
        );
        assert_eq!(
            value_of(memory.lookup(
                &encode_key(b"alpha", b"Ova", 1),
                u64::MAX,
                HistoryView::Current
            ))
            .as_deref(),
            Some("Atlas")
        );
    }

    #[test]
    fn declared_capacity_releases_the_oldest_payload_as_a_visible_tombstone() {
        let mut memory = Memory::new(1, 2);
        write(&mut memory, "Ova", "Bramble", Update::Assert);
        write(&mut memory, "Ova", "Quarry", Update::Assert);
        write(&mut memory, "Ova", "Vale", Update::Assert);
        let key = encode_key(b"alpha", b"Ova", 0);
        // The head is never the evicted record.
        assert_eq!(
            value_of(memory.lookup(&key, u64::MAX, HistoryView::Current)).as_deref(),
            Some("Vale")
        );
        // The released record is reported as a typed release, never as absence.
        assert!(matches!(
            memory.lookup(&key, u64::MAX, HistoryView::Initial),
            Lookup::Evicted
        ));
        let records = memory.records.clone();
        assert!(records[0].evicted && records[0].payload.is_empty());
        memory.validate().expect("evicted store validates");
    }

    #[test]
    fn store_round_trips_and_rejects_tampering() {
        let mut memory = Memory::new(5, 4);
        write(&mut memory, "Ova", "Bramble", Update::Assert);
        write(&mut memory, "Ova", "Vale", Update::Correct);
        let bytes = memory.to_bytes().unwrap();
        let reloaded = Memory::from_bytes(&bytes).unwrap();
        assert_eq!(reloaded, memory);
        assert!(Memory::from_bytes(b"{").is_err());

        let mut tampered = memory.clone();
        tampered.records[0].commit = 99;
        assert!(tampered.validate().is_err());
        let mut bad_key = memory.clone();
        bad_key.chains[0].0 = encode_key(b"other", b"Ova", 0);
        assert!(bad_key.validate().is_err());
        let mut early = memory.clone();
        early.records[1].commit = early.records[0].commit;
        assert!(early.validate().is_err());
    }

    fn runtime(memory: Memory, eos: Option<u32>) -> ScopedMemoryRuntime {
        ScopedMemoryRuntime::load(
            &ObservedTextModel::uninformed().to_bytes().unwrap(),
            &IntentModel::uninformed().to_bytes().unwrap(),
            memory.clone(),
            memory.lineage,
            MemoryControl::Normal,
            4096,
            eos,
        )
        .unwrap()
    }

    #[test]
    fn capacity_releases_each_expired_payload_after_multiple_overflows() {
        let mut memory = Memory::new(11, 2);
        for turn in 0..8 {
            write(
                &mut memory,
                "Ova",
                &format!("value-{turn}"),
                Update::Correct,
            );
            memory.validate().unwrap();
            assert_eq!(
                memory.records.iter().filter(|r| !r.evicted).count(),
                (turn + 1).min(2)
            );
            for record in memory.records.iter().filter(|r| r.evicted) {
                assert!(record.payload.is_empty());
                assert_eq!(record.payload.capacity(), 0);
                assert_eq!(record.payload_sha256, payload_sha256(&[1, 2, 3]));
            }
        }
        let key = encode_key(b"alpha", b"Ova", 0);
        assert!(matches!(
            memory.lookup(&key, 2, HistoryView::Current),
            Lookup::Evicted
        ));
        assert_eq!(
            value_of(memory.lookup(&key, 8, HistoryView::PreviousAssertion)).as_deref(),
            Some("value-6")
        );
    }

    #[test]
    fn reload_rejects_id_predecessor_coverage_capacity_and_payload_corruption() {
        let mut valid = Memory::new(12, 2);
        write(&mut valid, "Ova", "A", Update::Assert);
        write(&mut valid, "Ova", "B", Update::Correct);
        write(&mut valid, "Ova", "C", Update::Correct);
        let mut broken = Vec::new();
        let mut m = valid.clone();
        m.records[1].id = 1;
        broken.push(m);
        let mut m = valid.clone();
        m.records[1].predecessor = 0;
        broken.push(m);
        let mut m = valid.clone();
        m.chains[0].1.remove(1);
        broken.push(m);
        let mut m = valid.clone();
        m.commit += 1;
        broken.push(m);
        let mut m = valid.clone();
        m.records[1].commit = 1;
        broken.push(m);
        let mut m = valid.clone();
        m.records[1].payload[0] = 99;
        broken.push(m);
        let mut m = valid.clone();
        m.records[1].evicted = true;
        m.records[1].payload.clear();
        broken.push(m);
        let mut m = valid.clone();
        m.records[0].payload_sha256.clear();
        broken.push(m);
        let mut m = valid.clone();
        m.records[1].conflict = true;
        broken.push(m);
        let mut m = valid.clone();
        m.version = 1;
        broken.push(m);
        for m in broken {
            assert!(Memory::from_bytes(&m.to_bytes().unwrap()).is_err());
        }
        valid.records.reverse();
        assert!(Memory::from_bytes(&valid.to_bytes().unwrap()).is_ok());
    }

    #[test]
    fn snapshot_enforces_owned_prefix_eos_capture_and_phase_at_each_step() {
        let mut memory = Memory::new(13, 2);
        write(&mut memory, "Ova", "A", Update::Assert);
        let runtime = runtime(memory, Some(4095));
        let mut session = runtime
            .ask_view(b"alpha", 0, b"Ova", HistoryView::Current)
            .unwrap();
        assert_eq!(
            runtime
                .restore(&runtime.snapshot(&session).unwrap())
                .unwrap(),
            session
        );
        runtime.step(&mut session).unwrap();
        let mut tampered = session.clone();
        tampered.captured.as_mut().unwrap().payload[0] = 9;
        assert!(runtime
            .restore(&serde_json::to_vec(&tampered).unwrap())
            .is_err());
        let mut tampered = session.clone();
        tampered.captured.as_mut().unwrap().record_id = 999;
        assert!(runtime.validate(&tampered).is_err());
        let mut tampered = session.clone();
        tampered.captured.as_mut().unwrap().value = b"other".to_vec();
        assert!(runtime.validate(&tampered).is_err());
        let mut tampered = session.clone();
        tampered.captured.as_mut().unwrap().key = encode_key(b"beta", b"Ova", 0);
        assert!(runtime.validate(&tampered).is_err());
        let mut tampered = session.clone();
        tampered.pending = RelAction::Continue;
        assert!(runtime.validate(&tampered).is_err());
        let mut old = session.clone();
        old.version = 1;
        assert!(runtime.restore(&serde_json::to_vec(&old).unwrap()).is_err());
        while session.terminal.is_none() {
            runtime.step(&mut session).unwrap();
            assert_eq!(
                runtime
                    .restore(&runtime.snapshot(&session).unwrap())
                    .unwrap(),
                session
            );
            if !session.emitted.is_empty() {
                let mut tampered = session.clone();
                tampered.emitted[0] = 99;
                assert!(runtime.validate(&tampered).is_err());
            }
        }
        assert_eq!(session.emitted, vec![1, 2, 3, 4095]);
        let mut bad_eos = session.clone();
        *bad_eos.emitted.last_mut().unwrap() = 4094;
        assert!(runtime.validate(&bad_eos).is_err());
    }

    #[test]
    fn pinned_history_rejects_a_same_lineage_fork_but_capture_survives_eviction() {
        let mut memory = Memory::new(14, 1);
        write(&mut memory, "Ova", "A", Update::Assert);
        let mut original = runtime(memory.clone(), None);
        let mut session = original
            .ask_view(b"alpha", 0, b"Ova", HistoryView::Current)
            .unwrap();
        original.step(&mut session).unwrap();
        let snapshot = original.snapshot(&session).unwrap();
        let mut fork = memory;
        fork.records[0].value = b"foreign".to_vec();
        let foreign = runtime(fork, None);
        assert_eq!(foreign.binding(), original.binding());
        assert!(foreign.restore(&snapshot).is_err());
        write(&mut original.memory, "Ova", "B", Update::Correct);
        assert!(original.memory.records[0].evicted);
        assert!(!session.origin_is_live(original.memory()));
        let mut restored = original.restore(&snapshot).unwrap();
        original.run(&mut restored).unwrap();
        assert_eq!(restored.emitted, vec![1, 2, 3]);
        assert_eq!(restored.terminal, Some(ScopedTerminal::Complete));
    }

    #[test]
    fn dependent_snapshot_preserves_one_history_and_rejects_capture_retargeting() {
        let mut memory = Memory::new(15, 4);
        memory
            .write(
                b"alpha",
                b"Ova",
                0,
                b"Rin",
                &[1],
                1,
                Update::Assert,
                true,
                0,
            )
            .unwrap();
        memory
            .write(
                b"alpha",
                b"Rin",
                0,
                b"answer",
                &[2, 3],
                2,
                Update::Assert,
                false,
                0,
            )
            .unwrap();
        let mut runtime = runtime(memory, None);
        let mut session = runtime
            .ask_view(b"alpha", 0, b"Ova", HistoryView::Current)
            .unwrap();
        for _ in 0..2 {
            runtime.step(&mut session).unwrap();
            session = runtime
                .restore(&runtime.snapshot(&session).unwrap())
                .unwrap();
        }
        assert_eq!(session.pending, RelAction::Read);
        assert_eq!(session.query_entity, b"Rin");
        runtime
            .memory
            .write(
                b"alpha",
                b"Rin",
                0,
                b"new",
                &[4],
                3,
                Update::Correct,
                false,
                0,
            )
            .unwrap();
        runtime.run(&mut session).unwrap();
        assert_eq!(session.emitted, vec![2, 3]);
        let mut wrong = session.clone();
        wrong.captured.as_mut().unwrap().commit = 3;
        assert!(runtime.validate(&wrong).is_err());
    }

    #[test]
    fn oversized_answer_is_rejected_before_emission_and_counters_do_not_wrap() {
        let mut memory = Memory::new(16, 2);
        memory
            .write(
                b"alpha",
                b"Ova",
                0,
                b"long",
                &[1; SCOPED_MAX_ANSWER],
                1,
                Update::Assert,
                false,
                0,
            )
            .unwrap();
        let runtime = runtime(memory, Some(4095));
        let mut session = runtime
            .ask_view(b"alpha", 0, b"Ova", HistoryView::Current)
            .unwrap();
        runtime.run(&mut session).unwrap();
        assert_eq!(session.terminal, Some(ScopedTerminal::Exhausted));
        assert!(session.emitted.is_empty());
        let mut saturated = Memory::new(17, 2);
        saturated.next_id = u64::MAX;
        let before = saturated.clone();
        assert!(saturated
            .write(b"alpha", b"Ova", 0, b"A", &[1], 1, Update::Assert, false, 0)
            .is_err());
        assert_eq!(saturated, before);
    }

    #[test]
    fn public_previous_distinct_keeps_eviction_barrier_for_equal_tombstone() {
        let mut memory = Memory::new(18, 1);
        write(&mut memory, "Ova", "A", Update::Assert);
        write(&mut memory, "Ova", "A", Update::Assert);
        let key = encode_key(b"alpha", b"Ova", 0);
        assert!(matches!(
            memory.lookup(&key, memory.commit, HistoryView::PreviousDistinctValue),
            Lookup::Evicted
        ));
        assert!(matches!(
            memory.lookup_identity(&key, memory.commit, HistoryView::PreviousDistinctValue),
            Lookup::NoHistory
        ));
    }

    #[test]
    fn owned_previous_distinct_capture_survives_new_public_eviction_barrier() {
        let mut memory = Memory::new(19, 2);
        write(&mut memory, "Ova", "A", Update::Assert);
        write(&mut memory, "Ova", "B", Update::Correct);
        let mut runtime = runtime(memory, None);
        let mut session = runtime
            .ask_view(b"alpha", 0, b"Ova", HistoryView::PreviousDistinctValue)
            .unwrap();
        runtime.step(&mut session).unwrap();
        assert_eq!(session.captured.as_ref().unwrap().record_id, 1);
        let snapshot = runtime.snapshot(&session).unwrap();
        write(&mut runtime.memory, "Ova", "B", Update::Correct);
        assert!(matches!(
            runtime.memory.lookup(
                &encode_key(b"alpha", b"Ova", 0),
                session.view,
                HistoryView::PreviousDistinctValue
            ),
            Lookup::Evicted
        ));
        let mut restored = runtime.restore(&snapshot).unwrap();
        runtime.run(&mut restored).unwrap();
        assert_eq!(restored.terminal, Some(ScopedTerminal::Complete));
        assert_eq!(restored.emitted, vec![1, 2, 3]);
    }

    #[test]
    fn origin_liveness_checks_lineage_history_and_owned_contents() {
        let mut memory = Memory::new(20, 2);
        write(&mut memory, "Ova", "A", Update::Assert);
        write(&mut memory, "Rin", "B", Update::Assert);
        let runtime = runtime(memory.clone(), None);
        let mut session = runtime
            .ask_view(b"alpha", 0, b"Ova", HistoryView::Current)
            .unwrap();
        runtime.step(&mut session).unwrap();
        assert!(session.origin_is_live(&memory));

        let mut foreign_lineage = memory.clone();
        foreign_lineage.lineage += 1;
        assert!(!session.origin_is_live(&foreign_lineage));
        let mut foreign_history = memory.clone();
        foreign_history.records[1].value = b"different".to_vec();
        assert!(!session.origin_is_live(&foreign_history));
        let mut changed_payload = memory.clone();
        changed_payload.records[0].payload[0] = 9;
        assert!(!session.origin_is_live(&changed_payload));
        let mut changed_capture = session.clone();
        changed_capture.captured.as_mut().unwrap().value = b"different".to_vec();
        assert!(!changed_capture.origin_is_live(&memory));

        memory.records.reverse();
        assert!(session.origin_is_live(&memory));
        write(&mut memory, "Rin", "C", Update::Correct);
        assert!(session.origin_is_live(&memory));
        write(&mut memory, "Ova", "C", Update::Correct);
        assert!(!session.origin_is_live(&memory));
    }

    #[test]
    fn bound_out_of_band_eos_is_valid_but_out_of_vocabulary_payload_is_not() {
        let mut memory = Memory::new(21, 2);
        write(&mut memory, "Ova", "A", Update::Assert);
        let eos = u32::MAX - 1;
        let runtime = runtime(memory.clone(), Some(eos));
        let mut session = runtime
            .ask_view(b"alpha", 0, b"Ova", HistoryView::Current)
            .unwrap();
        runtime.run(&mut session).unwrap();
        assert_eq!(session.emitted, vec![1, 2, 3, eos]);
        assert_eq!(
            runtime
                .restore(&runtime.snapshot(&session).unwrap())
                .unwrap(),
            session
        );
        let mut wrong_eos = session.clone();
        *wrong_eos.emitted.last_mut().unwrap() = eos - 1;
        assert!(runtime.validate(&wrong_eos).is_err());

        memory.records[0].payload = vec![4096];
        memory.records[0].payload_sha256 = payload_sha256(&memory.records[0].payload);
        assert!(ScopedMemoryRuntime::load(
            &ObservedTextModel::uninformed().to_bytes().unwrap(),
            &IntentModel::uninformed().to_bytes().unwrap(),
            memory,
            21,
            MemoryControl::Normal,
            4096,
            Some(eos),
        )
        .is_err());
    }
}
