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

use super::grounded_session::GroundedFactorization;
use super::group_table::{group_table, GROUP_ORDER, ROW_STRIDE};
use super::lexical_realization::{
    RealizationAction, RealizationContext, RealizationDecision, RealizationModel,
};
use super::observed_text_session::{
    candidate_features, lexical_key, observe_clause, Clause, Observation, ObservedTextModel,
};
use super::realtext_support::sha256_hex;
use super::shared_transition::{SharedTransitionModel, StExample};
use super::state_lexical::StateLexicalModel;

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

/// The intent format is unchanged; store v2 adds immutable payload/history witnesses; session v3
/// adds the typed computation phase and its owned result.
pub const SCOPED_VERSION: u8 = 1;
pub const SCOPED_STORE_VERSION: u8 = 2;
pub const SCOPED_SESSION_VERSION: u8 = 5;
pub const SCOPED_INTENT_VERSION: u8 = 2;
pub const SCOPED_MAX_OPERATIONS: usize = super::observed_text_session::OB_MAX_CLAUSE;

/// The scoped session's action vocabulary: the retained memory actions plus one typed computation
/// phase. `Apply` runs exactly one observed operation through the bound computation artifact.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SessionAction {
    Read,
    Apply,
    Continue,
    Emit,
    Stop,
}
/// Statement intents: assert, explicit correction, declared nonasserting, computation request.
pub const N_STMT_INTENT: usize = 4;
/// Learned statement intent indices.
pub const STMT_ASSERT: usize = 0;
pub const STMT_CORRECT: usize = 1;
pub const STMT_NONASSERTING: usize = 2;
/// A request that applies observed operations to a scoped source and consumes the result. It writes
/// nothing.
pub const STMT_COMPUTE: usize = 3;
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
/// Declared bound on learned realization vocabulary words per answer. Kept a power of two so the
/// realized emission test is a compare, not a division.
pub const SCOPED_MAX_VOCAB_WORDS: usize = 4;

/// The declared public output contract.
///
/// `LegacyWords` is the retained byte-for-byte plain answer: the exact owned payload followed by the
/// bound terminator. `RealizedV1` is a prospectively versioned richer contract in which the *same*
/// learned policy interleaves shared learned vocabulary words with the *same* exact owned span. The
/// distinction is a versioned output contract, not an evaluator-supplied task mode, and it lets
/// semantic retention be measured separately from byte-for-byte retention.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum OutputContract {
    #[default]
    LegacyWords,
    RealizedV1,
    /// A learned state-conditioned lexical decoder: the same exact owned span, interleaved with
    /// learned vocabulary words chosen by a bounded integer recurrent state that consumes executed
    /// insert-slot/Copy events and a content embedding of the selected evidence. Supersedes
    /// `RealizedV1` as the richer contract; `RealizedV1` remains retained and loadable.
    StateLexicalV1,
}

impl OutputContract {
    pub fn as_u8(self) -> u8 {
        match self {
            Self::LegacyWords => 0,
            Self::RealizedV1 => 1,
            Self::StateLexicalV1 => 2,
        }
    }
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::LegacyWords),
            1 => Some(Self::RealizedV1),
            2 => Some(Self::StateLexicalV1),
            _ => None,
        }
    }
    /// Whether the contract interleaves learned vocabulary with the exact owned span.
    pub fn is_lexical(self) -> bool {
        matches!(self, Self::RealizedV1 | Self::StateLexicalV1)
    }
}

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
    /// A migrated three-class artifact must never gain an unfitted fourth class at score zero.
    pub statement_classes: usize,
    pub statement: Vec<(u64, [i32; N_STMT_INTENT])>,
    pub question: Vec<(u64, [i32; N_ASK_INTENT])>,
}

impl IntentModel {
    pub fn uninformed() -> Self {
        IntentModel {
            version: SCOPED_INTENT_VERSION,
            statement_classes: N_STMT_INTENT,
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
        (0..self.statement_classes)
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
        if self.version != SCOPED_INTENT_VERSION || !matches!(self.statement_classes, 3 | 4) {
            return Err(ScopedMemoryError::Intent(
                "unsupported intent model version".into(),
            ));
        }
        if max_vocab == 0 || max_vocab > super::observed_text_session::OB_MAX_VOCAB {
            return Err(ScopedMemoryError::Intent(
                "vocabulary exceeds the 16-bit feature domain".into(),
            ));
        }
        if self.statement.windows(2).any(|w| w[0].0 >= w[1].0)
            || self.question.windows(2).any(|w| w[0].0 >= w[1].0)
        {
            return Err(ScopedMemoryError::Intent(
                "intent weights must have unique sorted keys".into(),
            ));
        }
        let in_domain = |key: u64| -> bool {
            let kind = key >> 40;
            let value = key & ((1u64 << 40) - 1);
            match kind {
                1..=6 => value < max_vocab as u64,
                7 => value > 0 && value <= super::observed_text_session::OB_MAX_MARKER as u64,
                8 | 9 => value == 0,
                _ => false,
            }
        };
        if self
            .statement
            .iter()
            .map(|(key, _)| *key)
            .chain(self.question.iter().map(|(key, _)| *key))
            .any(|key| !in_domain(key))
        {
            return Err(ScopedMemoryError::Intent(
                "intent feature is outside the retained extractor domain".into(),
            ));
        }
        Ok(())
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, ScopedMemoryError> {
        serde_json::to_vec(self).map_err(|e| ScopedMemoryError::Serialization(e.to_string()))
    }

    pub fn from_bytes(bytes: &[u8], max_vocab: usize) -> Result<Self, ScopedMemoryError> {
        let mut value: serde_json::Value = serde_json::from_slice(bytes)
            .map_err(|e| ScopedMemoryError::Serialization(e.to_string()))?;
        if value["version"] == serde_json::json!(1) {
            let rows = value["statement"].as_array_mut().ok_or_else(|| {
                ScopedMemoryError::Intent("legacy intent has no statement rows".into())
            })?;
            for row in rows {
                let weights = row
                    .get_mut(1)
                    .and_then(|v| v.as_array_mut())
                    .ok_or_else(|| {
                        ScopedMemoryError::Intent("invalid legacy statement row".into())
                    })?;
                if weights.len() != 3 {
                    return Err(ScopedMemoryError::Intent(
                        "v1 intent requires three classes; unversioned four-class artifact is unsupported".into(),
                    ));
                }
                weights.push(serde_json::json!(0));
            }
            value["version"] = serde_json::json!(SCOPED_INTENT_VERSION);
            value["statement_classes"] = serde_json::json!(3);
        }
        let model: Self = serde_json::from_value(value)
            .map_err(|e| ScopedMemoryError::Serialization(e.to_string()))?;
        model.validate(max_vocab)?;
        Ok(model)
    }
}

/// A learned, artifact-bound lexicon between exact lexical bytes and a computation artifact's
/// opaque domain identifiers.
///
/// The identifiers carry **no positional meaning**: which group element each one denotes is recovered
/// from observed development transitions by the constructive factorization, never from an index, a
/// token id, a hidden semantic label or a keyword table. Serving looks a key up by its exact bytes, so
/// a different BPE segmentation of the same label cannot silently become a different operand.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroundingLexicon {
    pub version: u8,
    /// Observed surface key -> opaque domain identifier.
    pub key_to_label: Vec<(Vec<u8>, u32)>,
    /// Opaque identifier -> the canonical exact key of that label's own memory entity.
    pub label_to_key: Vec<(u32, Vec<u8>)>,
}

impl GroundingLexicon {
    /// The declared empty lexicon: it grounds nothing. Only valid when no computation artifact is
    /// bound, so the ordinary memory lifecycle can load without one.
    pub fn empty() -> Self {
        GroundingLexicon {
            version: SCOPED_VERSION,
            key_to_label: Vec::new(),
            label_to_key: Vec::new(),
        }
    }

    /// Fit the lexicon from declared development surface observations. A surface key with two
    /// identifiers, an identifier without a canonical entity key, or an empty key is rejected.
    pub fn from_observations(
        surface: &[(Vec<u8>, u32)],
        canonical: &[(u32, Vec<u8>)],
    ) -> Result<Self, ScopedMemoryError> {
        let mut key_to_label = surface.to_vec();
        key_to_label.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        if key_to_label.iter().any(|(key, _)| key.is_empty()) {
            return Err(ScopedMemoryError::Computation(
                "lexicon surface key must be nonempty".into(),
            ));
        }
        if key_to_label
            .windows(2)
            .any(|w| w[0].0 == w[1].0 && w[0].1 != w[1].1)
        {
            return Err(ScopedMemoryError::Computation(
                "one surface key would denote two identifiers".into(),
            ));
        }
        key_to_label.dedup();
        let mut label_to_key = canonical.to_vec();
        label_to_key.sort_by(|a, b| a.0.cmp(&b.0));
        if label_to_key.iter().any(|(_, key)| key.is_empty())
            || label_to_key.windows(2).any(|w| w[0].0 == w[1].0)
        {
            return Err(ScopedMemoryError::Computation(
                "canonical label keys must be nonempty and unique per identifier".into(),
            ));
        }
        let lexicon = GroundingLexicon {
            version: SCOPED_VERSION,
            key_to_label,
            label_to_key,
        };
        lexicon.validate()?;
        Ok(lexicon)
    }

    pub fn label_for(&self, key: &[u8]) -> Option<u32> {
        self.key_to_label
            .binary_search_by(|(k, _)| k.as_slice().cmp(key))
            .ok()
            .map(|i| self.key_to_label[i].1)
    }

    pub fn key_for_label(&self, label: u32) -> Option<&[u8]> {
        self.label_to_key
            .binary_search_by(|(l, _)| l.cmp(&label))
            .ok()
            .map(|i| self.label_to_key[i].1.as_slice())
    }

    pub fn labels(&self) -> impl Iterator<Item = u32> + '_ {
        self.label_to_key.iter().map(|(label, _)| *label)
    }

    pub fn validate(&self) -> Result<(), ScopedMemoryError> {
        let bad = |message: &str| ScopedMemoryError::Computation(format!("lexicon {message}"));
        if self.version != SCOPED_VERSION {
            return Err(bad("unsupported version"));
        }
        if self.key_to_label.is_empty() != self.label_to_key.is_empty() {
            return Err(bad("half-empty"));
        }
        if self.key_to_label.is_empty() {
            return Ok(());
        }
        if self.key_to_label.windows(2).any(|w| w[0].0 >= w[1].0)
            || self.label_to_key.windows(2).any(|w| w[0].0 >= w[1].0)
            || self.key_to_label.iter().any(|(key, _)| key.is_empty())
        {
            return Err(bad("entries are not unique and sorted"));
        }
        // Operation identifiers need no memory entity of their own; every *outcome* identifier that
        // can address a derived read must ground back to its own canonical key.
        for (label, key) in &self.label_to_key {
            if key.is_empty() || self.label_for(key) != Some(*label) {
                return Err(bad("canonical key does not ground to its own identifier"));
            }
        }
        Ok(())
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, ScopedMemoryError> {
        serde_json::to_vec(self).map_err(|e| ScopedMemoryError::Serialization(e.to_string()))
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ScopedMemoryError> {
        let lexicon: Self = serde_json::from_slice(bytes)
            .map_err(|e| ScopedMemoryError::Serialization(e.to_string()))?;
        lexicon.validate()?;
        Ok(lexicon)
    }
}

/// The recovered action's central involution, used only by the topological-fold control.
///
/// `Q8/{+-1}` is commutative, so identifying `q` with `-q` cannot distinguish `ij` from `ji`. The
/// projection is *computed* from the recovered tables: the unique non-identity element that is
/// self-inverse and commutes with the whole recovered image.
#[derive(Clone, Debug)]
pub struct FoldedGroup {
    project: Vec<usize>,
}

impl FoldedGroup {
    pub fn recover(factorization: &GroundedFactorization) -> Result<Self, ScopedMemoryError> {
        validate_factorization(factorization)?;
        if factorization.cyclic {
            return Err(ScopedMemoryError::Computation(
                "the central-sign fold requires the exact group action".into(),
            ));
        }
        let t = group_table();
        let mut image: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();
        for state in factorization
            .payload_state
            .iter()
            .chain(factorization.outcome_state.iter())
        {
            image.insert(*state as usize);
        }
        let codes: Vec<usize> = factorization
            .action_code
            .iter()
            .map(|c| *c as usize)
            .collect();
        let seeds: Vec<usize> = image.iter().copied().collect();
        for a in seeds.iter() {
            for b in &codes {
                image.insert(t.product[*b * ROW_STRIDE + *a] as usize);
            }
        }
        let elements: Vec<usize> = image.iter().copied().collect();
        let identity = t.identity as usize;
        let involutions: Vec<_> = elements
            .iter()
            .copied()
            .filter(|candidate| {
                *candidate != identity
                    && t.product[*candidate * ROW_STRIDE + *candidate] as usize == identity
            })
            .collect();
        if elements.len() != 8
            || involutions.len() != 1
            || !elements.iter().all(|a| {
                elements
                    .iter()
                    .all(|b| image.contains(&(t.product[*a * ROW_STRIDE + *b] as usize)))
            })
            || elements.iter().all(|a| {
                elements
                    .iter()
                    .all(|b| t.product[*a * ROW_STRIDE + *b] == t.product[*b * ROW_STRIDE + *a])
            })
        {
            return Err(ScopedMemoryError::Computation(
                "central-sign projection requires a recovered Q8 image".into(),
            ));
        }
        let central = elements
            .iter()
            .copied()
            .find(|candidate| {
                *candidate != identity
                    && t.product[*candidate * ROW_STRIDE + *candidate] as usize == identity
                    && elements.iter().all(|x| {
                        t.product[*candidate * ROW_STRIDE + *x]
                            == t.product[*x * ROW_STRIDE + *candidate]
                    })
            })
            .ok_or_else(|| {
                ScopedMemoryError::Computation(
                    "recovered action has no central involution to project".into(),
                )
            })?;
        let mut project: Vec<usize> = (0..GROUP_ORDER).collect();
        for element in elements {
            let paired = t.product[central * ROW_STRIDE + element] as usize;
            let representative = element.min(paired);
            project[element] = representative;
            project[paired] = representative;
        }
        Ok(FoldedGroup { project })
    }
}

/// A directly tabulated finite control: one shared permutation per observed primitive plus one
/// initial-state table, both read off the same development observations. It reuses familiar
/// transitions across sequences, so it is a competent finite comparator rather than a whole-program
/// dictionary.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TabulatedControl {
    pub value_domain: Vec<u32>,
    pub value_state: Vec<u8>,
    pub action_domain: Vec<u32>,
    /// `action_perm[p][state]` is the successor state.
    pub action_perm: Vec<Vec<u8>>,
}

impl TabulatedControl {
    pub fn fit(examples: &[StExample]) -> Result<Self, ScopedMemoryError> {
        let mut labels: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
        let mut primitives: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
        for example in examples {
            labels.insert(example.payload);
            for target in &example.targets {
                labels.insert(*target);
            }
            for primitive in &example.primitives {
                primitives.insert(*primitive);
            }
        }
        let value_domain: Vec<u32> = labels.into_iter().collect();
        let action_domain: Vec<u32> = primitives.into_iter().collect();
        if value_domain.is_empty()
            || value_domain.len() > 256
            || action_domain.is_empty()
            || examples
                .iter()
                .any(|e| e.primitives.len() != e.targets.len())
        {
            return Err(ScopedMemoryError::Computation(
                "invalid finite observation dimensions".into(),
            ));
        }
        let index = |label: u32| {
            value_domain
                .iter()
                .position(|l| *l == label)
                .map(|i| i as u8)
        };
        let mut perms: Vec<Vec<Option<u8>>> =
            vec![vec![None; value_domain.len()]; action_domain.len()];
        for example in examples {
            let Some(state) = index(example.payload) else {
                continue;
            };
            let mut current = state;
            for (j, primitive) in example.primitives.iter().enumerate() {
                let Some(target) = example.targets.get(j).copied() else {
                    break;
                };
                let Some(next) = index(target) else { break };
                let p = action_domain
                    .iter()
                    .position(|q| q == primitive)
                    .ok_or_else(|| {
                        ScopedMemoryError::Computation("tabulated primitive is unknown".into())
                    })?;
                if let Some(existing) = perms[p][current as usize] {
                    if existing != next {
                        return Err(ScopedMemoryError::Computation(
                            "tabulated observations disagree on one transition".into(),
                        ));
                    }
                }
                perms[p][current as usize] = Some(next);
                current = next;
            }
        }
        // Each lexical outcome is a state; every observed transition above is checked regardless
        // of example ordering or which primitive happens to occur first in the observations.
        let value_state: Vec<u8> = (0..value_domain.len()).map(|i| i as u8).collect();
        let mut action_perm = Vec::with_capacity(action_domain.len());
        for perm in perms {
            let mut row = Vec::with_capacity(value_domain.len());
            for entry in &perm {
                row.push(entry.ok_or_else(|| {
                    ScopedMemoryError::Computation(
                        "a tabulated primitive does not ground every outcome".into(),
                    )
                })?);
            }
            let distinct: std::collections::BTreeSet<u8> = row.iter().copied().collect();
            if distinct.len() != row.len() {
                return Err(ScopedMemoryError::Computation(
                    "a tabulated primitive is not bijective".into(),
                ));
            }
            action_perm.push(row);
        }
        Ok(TabulatedControl {
            value_domain,
            value_state,
            action_domain,
            action_perm,
        })
    }

    pub fn initial_state(&self, label: u32) -> Result<usize, ScopedMemoryError> {
        self.value_domain
            .iter()
            .position(|l| *l == label)
            .and_then(|i| self.value_state.get(i).map(|s| *s as usize))
            .ok_or_else(|| ScopedMemoryError::Computation("unknown operand".into()))
    }

    pub fn apply(&self, state: usize, primitive: u32) -> Result<usize, ScopedMemoryError> {
        let p = self
            .action_domain
            .iter()
            .position(|q| *q == primitive)
            .ok_or_else(|| ScopedMemoryError::Computation("unknown operation".into()))?;
        self.action_perm
            .get(p)
            .and_then(|row| row.get(state))
            .map(|s| *s as usize)
            .ok_or_else(|| ScopedMemoryError::Computation("state outside the table".into()))
    }

    pub fn label_for_state(&self, state: usize) -> Option<u32> {
        self.value_state
            .iter()
            .position(|s| *s as usize == state)
            .and_then(|i| self.value_domain.get(i).copied())
    }
}

/// The bound computation artifact. Every variant is fitted on the *same* observed development
/// transitions; only the arithmetic differs, so a comparison is matched.
#[derive(Clone, Debug)]
pub enum ComputationBackend {
    /// The constructive exact factorization of the observed action: retained signed state.
    Signed(Box<GroundedFactorization>),
    /// The same recovered action with the central sign projected away (topological-fold control).
    Folded(Box<GroundedFactorization>, Box<FoldedGroup>),
    /// The project's fitted shared-transition model, reported at its measured fit.
    Finite(Box<SharedTransitionModel>),
    /// The directly tabulated finite control, provably implementing the observed action.
    Tabulated(Box<TabulatedControl>),
    /// No computation artifact bound.
    Absent,
}

fn valid_domain(domain: &[u32], states: usize) -> bool {
    !domain.is_empty() && domain.len() == states && domain.windows(2).all(|w| w[0] < w[1])
}

fn is_permutation(row: &[u8], n: usize) -> bool {
    row.len() == n
        && row.iter().all(|s| (*s as usize) < n)
        && row.iter().collect::<std::collections::BTreeSet<_>>().len() == n
}

fn checked_action(
    cyclic: bool,
    domain: &[u32],
    codes: &[u8],
    state: usize,
    label: u32,
) -> Option<usize> {
    let code = *codes.get(domain.iter().position(|op| *op == label)?)? as usize;
    if state >= GROUP_ORDER || code >= GROUP_ORDER {
        return None;
    }
    if cyclic {
        Some((state + code) % GROUP_ORDER)
    } else {
        group_table()
            .product
            .get(code * ROW_STRIDE + state)
            .map(|next| *next as usize)
    }
}

fn validate_factorization(f: &GroundedFactorization) -> Result<(), ScopedMemoryError> {
    if !valid_domain(&f.payload_domain, f.payload_state.len())
        || !valid_domain(&f.outcome_domain, f.outcome_state.len())
        || !valid_domain(&f.action_domain, f.action_code.len())
        || f.payload_state
            .iter()
            .chain(&f.outcome_state)
            .chain(&f.action_code)
            .any(|state| *state as usize >= GROUP_ORDER)
        || f.outcome_state
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != f.outcome_state.len()
        || !f.outcome_domain.contains(&f.reference_outcome)
    {
        return Err(ScopedMemoryError::Computation(
            "invalid factorization dimensions or domains".into(),
        ));
    }
    if f.payload_state.iter().chain(&f.outcome_state).any(|state| {
        f.action_domain.iter().any(|op| {
            f.apply(*state as usize, *op)
                .is_none_or(|next| f.outcome_for_state(next).is_none())
        })
    }) {
        return Err(ScopedMemoryError::Computation(
            "factorized action leaves its grounded outcomes".into(),
        ));
    }
    Ok(())
}

impl ComputationBackend {
    /// Canonical execution identity, including every field that changes the served arithmetic.
    pub fn identity_json(&self) -> serde_json::Value {
        backend_identity(self)
    }

    /// Validated independent reconstruction. Numeric narrowing, mismatched vectors and unsupported
    /// artifact versions return errors before any indexing or arithmetic occurs.
    pub fn from_identity_json(value: &serde_json::Value) -> Result<Self, ScopedMemoryError> {
        let bad = |detail: &str| ScopedMemoryError::Computation(detail.into());
        if value["version"] != serde_json::json!(1) {
            return Err(bad("unsupported computation identity version"));
        }
        let read_u32 = |key: &str| -> Result<Vec<u32>, ScopedMemoryError> {
            serde_json::from_value(value[key].clone()).map_err(|e| bad(&format!("{key}: {e}")))
        };
        let read_u8 = |key: &str| -> Result<Vec<u8>, ScopedMemoryError> {
            serde_json::from_value(value[key].clone()).map_err(|e| bad(&format!("{key}: {e}")))
        };
        let backend = match value["name"].as_str() {
            Some("signed_factorization" | "folded_central_sign") => {
                let f = GroundedFactorization {
                    cyclic: value["cyclic"]
                        .as_bool()
                        .ok_or_else(|| bad("missing arithmetic mode"))?,
                    action_domain: read_u32("action_domain")?,
                    action_code: read_u8("action_code")?,
                    outcome_domain: read_u32("outcome_domain")?,
                    outcome_state: read_u8("outcome_state")?,
                    payload_domain: read_u32("payload_domain")?,
                    payload_state: read_u8("payload_state")?,
                    reference_outcome: serde_json::from_value(value["reference_outcome"].clone())
                        .map_err(|e| bad(&format!("reference outcome: {e}")))?,
                };
                validate_factorization(&f)?;
                if value["name"] == "folded_central_sign" {
                    let fold = FoldedGroup::recover(&f)?;
                    if value["projection"] != serde_json::json!(fold.project) {
                        return Err(bad("fold projection disagrees with recovered action"));
                    }
                    Self::Folded(Box::new(f), Box::new(fold))
                } else {
                    Self::Signed(Box::new(f))
                }
            }
            Some("tabulated_finite") => Self::Tabulated(Box::new(TabulatedControl {
                value_domain: read_u32("value_domain")?,
                value_state: read_u8("value_state")?,
                action_domain: read_u32("action_domain")?,
                action_perm: serde_json::from_value(value["action_perm"].clone())
                    .map_err(|e| bad(&format!("permutation: {e}")))?,
            })),
            Some("finite_transition") => {
                let bytes: Vec<u8> = serde_json::from_value(value["model_bytes"].clone())
                    .map_err(|e| bad(&format!("finite model: {e}")))?;
                Self::Finite(Box::new(
                    SharedTransitionModel::from_bytes(&bytes, usize::MAX)
                        .map_err(|e| bad(&format!("finite model: {e:?}")))?,
                ))
            }
            Some("absent") => Self::Absent,
            _ => return Err(bad("unsupported computation backend")),
        };
        backend.validate()?;
        if backend.identity_json() != *value {
            return Err(bad(
                "computation identity has unknown or noncanonical fields",
            ));
        }
        Ok(backend)
    }

    pub fn validate(&self) -> Result<(), ScopedMemoryError> {
        let bad =
            || ScopedMemoryError::Computation("invalid computation dimensions or domains".into());
        match self {
            Self::Signed(f) => validate_factorization(f),
            Self::Folded(f, fold) => {
                if FoldedGroup::recover(f)?.project != fold.project {
                    return Err(bad());
                }
                Ok(())
            }
            Self::Finite(m) => {
                if !valid_domain(&m.value_domain, m.value_state.len())
                    || !valid_domain(&m.action_domain, m.action_code.len())
                    || m.value_state
                        .iter()
                        .chain(&m.action_code)
                        .any(|s| *s as usize >= GROUP_ORDER)
                {
                    return Err(bad());
                }
                Ok(())
            }
            Self::Tabulated(t) => {
                let n = t.value_domain.len();
                if !valid_domain(&t.value_domain, t.value_state.len())
                    || n > 256
                    || !valid_domain(&t.action_domain, t.action_perm.len())
                    || !is_permutation(&t.value_state, n)
                    || t.action_perm.iter().any(|r| !is_permutation(r, n))
                {
                    return Err(bad());
                }
                Ok(())
            }
            Self::Absent => Ok(()),
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Self::Signed(_) => "signed_factorization",
            Self::Folded(_, _) => "folded_central_sign",
            Self::Finite(_) => "finite_transition",
            Self::Tabulated(_) => "tabulated_finite",
            Self::Absent => "absent",
        }
    }

    /// The retained element a grounded operand label denotes.
    pub fn initial_state(&self, label: u32) -> Result<usize, ScopedMemoryError> {
        let initial = |domain: &[u32], states: &[u8]| {
            domain
                .iter()
                .position(|p| *p == label)
                .and_then(|i| states.get(i))
                .map(|s| *s as usize)
                .filter(|s| *s < GROUP_ORDER)
        };
        match self {
            Self::Signed(factorization) => initial(
                &factorization.payload_domain,
                &factorization.payload_state,
            )
            .ok_or_else(|| {
                ScopedMemoryError::Computation("operand has no grounded initial state".into())
            }),
            Self::Folded(f, fold) => initial(&f.payload_domain, &f.payload_state)
                .and_then(|s| fold.project.get(s).copied())
                .ok_or_else(|| {
                    ScopedMemoryError::Computation("operand has no projected initial state".into())
                }),
            Self::Finite(model) => {
                initial(&model.value_domain, &model.value_state).ok_or_else(|| {
                    ScopedMemoryError::Computation("unknown or invalid finite initial state".into())
                })
            }
            Self::Tabulated(table) => table.initial_state(label),
            Self::Absent => Err(ScopedMemoryError::Computation(
                "no computation artifact is bound".into(),
            )),
        }
    }

    /// Apply one grounded operation label. Left action: `next = A[op] * state`.
    pub fn apply(&self, state: usize, label: u32) -> Result<usize, ScopedMemoryError> {
        match self {
            Self::Signed(factorization) => checked_action(
                factorization.cyclic,
                &factorization.action_domain,
                &factorization.action_code,
                state,
                label,
            )
            .ok_or_else(|| {
                ScopedMemoryError::Computation("operation is not grounded by the artifact".into())
            }),
            Self::Folded(factorization, folded) => {
                let next = checked_action(
                    factorization.cyclic,
                    &factorization.action_domain,
                    &factorization.action_code,
                    state,
                    label,
                )
                .ok_or_else(|| {
                    ScopedMemoryError::Computation(
                        "operation is not grounded by the artifact".into(),
                    )
                })?;
                folded
                    .project
                    .get(next)
                    .copied()
                    .ok_or_else(|| ScopedMemoryError::Computation("invalid fold projection".into()))
            }
            Self::Finite(model) => checked_action(
                model.cyclic,
                &model.action_domain,
                &model.action_code,
                state,
                label,
            )
            .ok_or_else(|| {
                ScopedMemoryError::Computation("unknown or invalid finite action".into())
            }),
            Self::Tabulated(table) => table.apply(state, label),
            Self::Absent => Err(ScopedMemoryError::Computation(
                "no computation artifact is bound".into(),
            )),
        }
    }

    /// The grounded label of a retained element, if the artifact grounds it.
    pub fn label_for_state(&self, state: usize) -> Option<u32> {
        match self {
            Self::Signed(f) | Self::Folded(f, _) => f
                .outcome_state
                .iter()
                .zip(&f.outcome_domain)
                .find(|(s, _)| **s as usize == state)
                .map(|(_, label)| *label),
            Self::Finite(model) => model
                .value_state
                .iter()
                .zip(&model.value_domain)
                .find(|(s, _)| **s as usize == state)
                .map(|(_, label)| *label),
            Self::Tabulated(table) => table.label_for_state(state),
            Self::Absent => None,
        }
    }
}

/// The owned result of one consumed computation inside an answer frame. It is deliberately not a
/// stored user assertion: it records provenance, the artifact identity and the grounded key that the
/// next read consumed.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComputedState {
    pub source_record: u64,
    pub source_commit: u64,
    pub source_scope: Vec<u8>,
    pub source_entity: Vec<u8>,
    pub source_key: Vec<u8>,
    pub source_hop: u8,
    pub operand_key: Vec<u8>,
    pub operand_state: usize,
    /// The operand's exact owned payload tokens, retained so the lexical decoder can see the value
    /// the computation started from as content (not as a supplied `changed` flag).
    #[serde(default)]
    pub operand_payload: Vec<u32>,
    pub state: usize,
    pub applied: usize,
    pub artifact: String,
    pub derived_label: u32,
    pub derived_key: Vec<u8>,
    /// The derived key was actually used as the next read address.
    pub consumed: bool,
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
    /// Skip the computation phase: the derived key stays the operand's own label.
    ApplyDisabled,
    /// Compute the retained state but do not consume it: the next read uses the operand label.
    ConsumeDisabled,
    /// Hold the realization context's owned-evidence and derived-result flags at zero while keeping
    /// the learned table, the copy placement and the exact span. This isolates a contextual
    /// vocabulary effect from copy placement: the same artifact queried without its causal features.
    RealizationContextDisabled,
    /// Keep the state-lexical content and flags but hold the decoder state at its initial value, so
    /// no emitted symbol advances the recurrence. This isolates the sequence mechanism: if the
    /// vocabulary effect survives, it did not depend on the symbols the session actually emitted.
    RecurrenceDisabled,
}

/// Immutable identities prepared once from the loaded bytes and store.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemoryBinding {
    pub model_sha256: String,
    pub intent_sha256: String,
    pub lexicon_sha256: String,
    pub artifact_sha256: String,
    pub backend: String,
    pub lineage: u64,
    pub control_sha256: String,
    pub capacity: usize,
    pub max_vocab: usize,
    pub eos: Option<u32>,
    /// Identity of the bound realization artifact bytes, or the empty-byte digest when none is bound.
    pub realization_sha256: String,
    /// The declared output contract this runtime serves, as [`OutputContract::as_u8`].
    pub contract: u8,
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
    /// The capture came from a read of the consumed computation result.
    pub derived: bool,
    /// The successful Read's position in this answer, independent of later query mutation.
    pub read_hop: u8,
}

/// Immutable request interpretation retained alongside progress. A raw entry also retains the
/// observed clause, so restore can re-derive its selected entity/intent/operations using the bound
/// model. Exact API entries are explicitly distinguishable from language entries. This is causal
/// consistency, not authentication against replacing both a request and its entire history.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionRequest {
    pub entity: Vec<u8>,
    pub relation: u8,
    pub history: HistoryView,
    pub ops: Vec<Vec<u8>>,
    pub clause: Option<Clause>,
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
    /// The selected operand has no grounded initial state in the bound artifact.
    UnknownOperand,
    /// An observed operation has no grounded action in the bound artifact.
    UnknownOperation,
    /// The retained state has no grounded label in the bound artifact.
    UngroundedResult,
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
    pub request: SessionRequest,
    pub hop: u8,
    pub query_entity: Vec<u8>,
    /// The observed operations of a computation request, as exact lexical keys in request order.
    pub ops: Vec<Vec<u8>>,
    /// How many observed operations the retained state has consumed.
    pub op_cursor: usize,
    /// The owned computed result, present from the first Apply step.
    pub computation: Option<ComputedState>,
    /// The current query address came from the computation result rather than a stored value.
    pub derived: bool,
    pub captured: Option<MemoryCapture>,
    pub pending: SessionAction,
    pub emitted: Vec<u32>,
    pub cursor: usize,
    /// The output contract this session was started under.
    pub contract: u8,
    /// How many learned realization vocabulary words have been emitted so far.
    pub vocabulary_words: u8,
    /// How many of those words preceded the owned span.
    pub prelude_words: u8,
    pub visited: Vec<Vec<u8>>,
    pub terminal: Option<ScopedTerminal>,
    pub eos: Option<u32>,
    /// The bounded integer decoder state of the `StateLexicalV1` contract. Empty until the first
    /// realized emission initialises it; carried exactly across snapshot/restore.
    #[serde(default)]
    pub sl_state: Vec<i32>,
}

/// Performed action and its next phase, with the exact selected record.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ScopedStepEffect {
    pub action: SessionAction,
    pub next_action: Option<SessionAction>,
    pub selected_record: Option<u64>,
    pub selected_commit: Option<u64>,
    pub selected_value: Option<Vec<u8>>,
    /// The grounded action label consumed by one Apply step.
    pub op_label: Option<u32>,
    /// The retained state after one Apply step.
    pub computed_state: Option<usize>,
    pub hop: u8,
    pub emitted: Option<u32>,
    /// The realized emission decision performed by this step, when `RealizedV1` is bound.
    pub realization: Option<RealizationDecision>,
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
    lexicon: GroundingLexicon,
    backend: ComputationBackend,
    realization: Option<RealizationModel>,
    state_lexical: Option<StateLexicalModel>,
    memory: Memory,
    binding: MemoryBinding,
    control: MemoryControl,
    contract: OutputContract,
    max_vocab: usize,
}

/// The artifact identity actually consumed by a computation step.
fn backend_identity(backend: &ComputationBackend) -> serde_json::Value {
    match backend {
        ComputationBackend::Signed(f) | ComputationBackend::Folded(f, _) => serde_json::json!({
            "version": 1,
            "name": backend.name(),
            "cyclic": f.cyclic,
            "reference_outcome": f.reference_outcome,
            "projection": match backend { ComputationBackend::Folded(_, fold) => Some(&fold.project), _ => None },
            "action_domain": f.action_domain,
            "action_code": f.action_code,
            "outcome_domain": f.outcome_domain,
            "outcome_state": f.outcome_state,
            "payload_domain": f.payload_domain,
            "payload_state": f.payload_state,
        }),
        ComputationBackend::Finite(model) => serde_json::json!({
            "version": 1,
            "name": backend.name(),
            "model_bytes": model.to_bytes(),
            "value_domain": model.value_domain,
            "value_state": model.value_state,
            "action_domain": model.action_domain,
            "action_code": model.action_code,
        }),
        ComputationBackend::Tabulated(table) => serde_json::json!({
            "version": 1,
            "name": backend.name(),
            "value_domain": table.value_domain,
            "value_state": table.value_state,
            "action_domain": table.action_domain,
            "action_perm": table.action_perm,
        }),
        ComputationBackend::Absent => serde_json::json!({"version": 1, "name": "absent"}),
    }
}

/// The exact raw bytes of one observed span, computed from the verified byte alignment.
fn span_bytes(clause: &Clause, start: u32, len: usize) -> Option<&[u8]> {
    if len == 0 || clause.byte_lengths.len() != clause.tokens.len() {
        return None;
    }
    let end = start
        .checked_add(len as u32)
        .filter(|end| *end as usize <= clause.tokens.len())?;
    let sum = |lengths: &[u32]| {
        lengths
            .iter()
            .try_fold(0usize, |n, b| n.checked_add(*b as usize))
    };
    let offset = sum(&clause.byte_lengths[..start as usize])?;
    let span = sum(&clause.byte_lengths[start as usize..end as usize])?;
    clause
        .text
        .as_bytes()
        .get(offset..offset.checked_add(span)?)
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
    /// The retained memory-only constructor: no computation artifact is bound.
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
        Self::load_with_computation(
            model_bytes,
            intent_bytes,
            None,
            ComputationBackend::Absent,
            memory,
            lineage,
            control,
            max_vocab,
            eos,
        )
    }

    /// Load with a declared computation artifact and its learned grounding lexicon. A non-absent
    /// artifact requires a nonempty lexicon: an operation or operand the artifact cannot ground must
    /// stay typed rather than silently become an identity.
    #[allow(clippy::too_many_arguments)]
    pub fn load_with_computation(
        model_bytes: &[u8],
        intent_bytes: &[u8],
        lexicon_bytes: Option<&[u8]>,
        backend: ComputationBackend,
        memory: Memory,
        lineage: u64,
        control: MemoryControl,
        max_vocab: usize,
        eos: Option<u32>,
    ) -> Result<Self, ScopedMemoryError> {
        Self::load_grounded(
            model_bytes,
            intent_bytes,
            lexicon_bytes,
            backend,
            None,
            OutputContract::LegacyWords,
            memory,
            lineage,
            control,
            max_vocab,
            eos,
        )
    }

    /// Load with a declared computation artifact and an optional learned realization artifact.
    ///
    /// `RealizedV1` requires the realization bytes: a richer contract that silently degraded to the
    /// legacy copy path would make its measured behaviour uninterpretable. `LegacyWords` ignores a
    /// supplied realization artifact except for identity binding.
    #[allow(clippy::too_many_arguments)]
    pub fn load_grounded(
        model_bytes: &[u8],
        intent_bytes: &[u8],
        lexicon_bytes: Option<&[u8]>,
        backend: ComputationBackend,
        realization_bytes: Option<&[u8]>,
        contract: OutputContract,
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
        let lexicon = match lexicon_bytes {
            Some(bytes) => GroundingLexicon::from_bytes(bytes)?,
            None => GroundingLexicon::empty(),
        };
        backend.validate()?;
        if !matches!(backend, ComputationBackend::Absent) && lexicon.key_to_label.is_empty() {
            return Err(ScopedMemoryError::Computation(
                "a bound computation artifact requires a nonempty lexicon".into(),
            ));
        }
        memory.validate()?;
        // The lexical artifact bytes are interpreted by the declared contract: `StateLexicalV1`
        // binds a learned state-conditioned decoder, the older contracts bind the retained finite
        // realization table. Both are versioned artifacts; a contract never silently degrades.
        let (realization, state_lexical) = match contract {
            OutputContract::StateLexicalV1 => {
                let model = match realization_bytes {
                    Some(bytes) => Some(
                        StateLexicalModel::from_bytes(bytes, max_vocab)
                            .map_err(ScopedMemoryError::Model)?,
                    ),
                    None => None,
                };
                if model.is_none() {
                    return Err(ScopedMemoryError::Model(
                        "the state-lexical output contract requires a state-lexical artifact"
                            .into(),
                    ));
                }
                (None, model)
            }
            _ => {
                let model = match realization_bytes {
                    Some(bytes) => Some(
                        RealizationModel::from_bytes(bytes, max_vocab)
                            .map_err(ScopedMemoryError::Model)?,
                    ),
                    None => None,
                };
                if contract == OutputContract::RealizedV1 && model.is_none() {
                    return Err(ScopedMemoryError::Model(
                        "the realized output contract requires a realization artifact".into(),
                    ));
                }
                (model, None)
            }
        };
        let lexical_slots: Option<&[u32]> = state_lexical
            .as_ref()
            .map(|m| m.slots.as_slice())
            .or_else(|| realization.as_ref().map(|m| m.slots.as_slice()));
        if lexical_slots.is_some_and(|slots| slots.iter().any(|token| Some(*token) == eos)) {
            return Err(ScopedMemoryError::Model(
                "a lexical vocabulary slot cannot be the protocol terminator".into(),
            ));
        }
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
            lexicon_sha256: sha256_hex(lexicon_bytes.unwrap_or(&[])),
            artifact_sha256: sha256_hex(
                &serde_json::to_vec(&backend_identity(&backend))
                    .map_err(|e| ScopedMemoryError::Serialization(e.to_string()))?,
            ),
            backend: backend.name().to_string(),
            lineage,
            control_sha256: sha256_hex(
                &serde_json::to_vec(&control)
                    .map_err(|e| ScopedMemoryError::Serialization(e.to_string()))?,
            ),
            capacity: memory.capacity,
            max_vocab,
            eos,
            realization_sha256: sha256_hex(realization_bytes.unwrap_or(&[])),
            contract: contract.as_u8(),
        };
        Ok(Self {
            model,
            intent,
            lexicon,
            backend,
            realization,
            state_lexical,
            memory,
            binding,
            control,
            contract,
            max_vocab,
        })
    }

    /// The bound learned realization artifact, if any.
    pub fn realization(&self) -> Option<&RealizationModel> {
        self.realization.as_ref()
    }
    /// The bound learned state-conditioned decoder, if any.
    pub fn state_lexical(&self) -> Option<&StateLexicalModel> {
        self.state_lexical.as_ref()
    }
    /// The declared output contract this runtime serves.
    pub fn contract(&self) -> OutputContract {
        self.contract
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
    pub fn lexicon(&self) -> &GroundingLexicon {
        &self.lexicon
    }
    pub fn backend(&self) -> &ComputationBackend {
        &self.backend
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
        let lexicon_bytes = self.lexicon.to_bytes()?;
        let realization_bytes = match self.realization.as_ref() {
            Some(model) => Some(model.to_bytes().map_err(ScopedMemoryError::Model)?),
            None => None,
        };
        let state_lexical_bytes = match self.state_lexical.as_ref() {
            Some(model) => Some(model.to_bytes().map_err(ScopedMemoryError::Model)?),
            None => None,
        };
        let mut runtime = Self::load_grounded(
            &model_bytes,
            &intent_bytes,
            Some(&lexicon_bytes),
            self.backend.clone(),
            state_lexical_bytes
                .as_deref()
                .or(realization_bytes.as_deref()),
            self.contract,
            memory,
            self.binding.lineage,
            control,
            self.max_vocab,
            self.binding.eos,
        )?;
        // Reserializing an already-loaded artifact (including a losslessly migrated v1 intent)
        // does not change the immutable identity of the bytes originally supplied to this runtime.
        runtime.binding.model_sha256 = self.binding.model_sha256.clone();
        runtime.binding.intent_sha256 = self.binding.intent_sha256.clone();
        runtime.binding.lexicon_sha256 = self.binding.lexicon_sha256.clone();
        runtime.binding.realization_sha256 = self.binding.realization_sha256.clone();
        Ok(runtime)
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
            STMT_NONASSERTING | STMT_COMPUTE => {
                return Ok(IngestOutcome::NonAsserting {
                    reason: "learned nonasserting statement intent".into(),
                });
            }
            _ => {
                return Err(ScopedMemoryError::Intent(
                    "statement intent is out of range".into(),
                ));
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
        let request = self.interpret_request(clause)?;
        let mut session =
            self.ask_view(scope, request.relation, &request.entity, request.history)?;
        session.ops = request.ops.clone();
        session.request = request;
        self.validate(&session)?;
        Ok(session)
    }

    fn interpret_request(&self, clause: &Clause) -> Result<SessionRequest, ScopedMemoryError> {
        let observed = self.observe(clause)?;
        if observed.is_question {
            let history = HistoryView::from_question_intent(observed.intent).ok_or_else(|| {
                ScopedMemoryError::Intent("learned question intent is out of range".into())
            })?;
            return Ok(SessionRequest {
                entity: observed.entity_key,
                relation: observed.relation,
                history,
                ops: Vec::new(),
                clause: Some(clause.clone()),
            });
        }
        if observed.intent == STMT_COMPUTE {
            // A computation request: the observed object span carries the ordered operations. The
            // surface words are taken from the actual observed bytes, so a different BPE segmentation
            // of the same label cannot become a different operand.
            let observation = observe_clause(&self.model, clause)
                .map_err(|e| ScopedMemoryError::Observation(e.to_string()))?;
            let (start, len) = observation
                .object_start
                .zip(observation.object.as_ref().map(|o| o.len()))
                .ok_or_else(|| {
                    ScopedMemoryError::Observation(
                        "computation request has no operation span".into(),
                    )
                })?;
            let bytes = span_bytes(clause, start, len).ok_or_else(|| {
                ScopedMemoryError::Observation(
                    "computation request has no exact byte alignment".into(),
                )
            })?;
            let ops: Vec<Vec<u8>> = bytes
                .split(|b| b.is_ascii_whitespace())
                .filter(|word| !word.is_empty())
                .map(|word| word.to_vec())
                .collect();
            if ops.is_empty() {
                return Err(ScopedMemoryError::Observation(
                    "computation request observes no operation".into(),
                ));
            }
            return Ok(SessionRequest {
                entity: observed.entity_key,
                relation: observed.relation,
                history: HistoryView::Current,
                ops,
                clause: Some(clause.clone()),
            });
        }
        Err(ScopedMemoryError::Observation(
            "observed clause is not a request".into(),
        ))
    }

    /// Start a computation request: read the source operand, apply the observed operations, then
    /// consume the grounded result as the next exact read address, all at one pinned view.
    pub fn ask_compute(
        &self,
        scope: &[u8],
        relation: u8,
        entity: &[u8],
        ops: Vec<Vec<u8>>,
    ) -> Result<ScopedSession, ScopedMemoryError> {
        if matches!(self.backend, ComputationBackend::Absent) {
            return Err(ScopedMemoryError::Computation(
                "no computation artifact is bound".into(),
            ));
        }
        if ops.is_empty() || ops.len() > SCOPED_MAX_OPERATIONS {
            return Err(ScopedMemoryError::Computation(
                "operation count is outside the declared bound".into(),
            ));
        }
        for op in &ops {
            if op.is_empty() {
                return Err(ScopedMemoryError::Computation(
                    "observed operation is empty".into(),
                ));
            }
        }
        let mut session = self.ask_view(scope, relation, entity, HistoryView::Current)?;
        session.ops = ops.clone();
        session.request.ops = ops;
        self.validate(&session)?;
        Ok(session)
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
            request: SessionRequest {
                entity: entity.to_vec(),
                relation,
                history,
                ops: Vec::new(),
                clause: None,
            },
            hop: 0,
            query_entity: entity.to_vec(),
            ops: Vec::new(),
            op_cursor: 0,
            computation: None,
            derived: false,
            captured: None,
            pending: SessionAction::Read,
            emitted: Vec::new(),
            cursor: 0,
            contract: self.contract.as_u8(),
            vocabulary_words: 0,
            prelude_words: 0,
            visited: Vec::new(),
            terminal: None,
            eos: self.binding.eos,
            sl_state: Vec::new(),
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
        if OutputContract::from_u8(session.contract) != Some(self.contract) {
            return Err(ScopedMemoryError::Session(
                "session output contract disagrees with the runtime".into(),
            ));
        }
        if session.prelude_words > session.vocabulary_words
            || session.vocabulary_words as usize > self.lexical_word_bound()
        {
            return Err(ScopedMemoryError::Session(
                "realization word counters are out of range".into(),
            ));
        }
        if self.contract == OutputContract::StateLexicalV1 {
            let model = self.state_lexical.as_ref().ok_or_else(|| {
                ScopedMemoryError::Session(
                    "state-lexical contract without a state-lexical artifact".into(),
                )
            })?;
            if !session.sl_state.is_empty()
                && (session.sl_state.len() != model.h_dim
                    || session
                        .sl_state
                        .iter()
                        .any(|v| i64::from(*v).abs() > i64::from(model.h_clamp)))
            {
                return Err(ScopedMemoryError::Session(
                    "retained decoder state disagrees with the bound artifact width or clamp"
                        .into(),
                ));
            }
        } else if !session.sl_state.is_empty() {
            return Err(ScopedMemoryError::Session(
                "retained decoder state without a state-lexical contract".into(),
            ));
        }
        if session.request.entity.is_empty()
            || session.request.relation != session.relation
            || session.request.history != session.history
            || session.request.ops != session.ops
            || session.ops.len() > SCOPED_MAX_OPERATIONS
            || session.ops.iter().any(|op| op.is_empty())
            || (!session.ops.is_empty() && matches!(self.backend, ComputationBackend::Absent))
            || session.visited.first().unwrap_or(&session.query_entity) != &session.request.entity
        {
            return Err(ScopedMemoryError::Session(
                "progress disagrees with the retained request".into(),
            ));
        }
        if let Some(clause) = &session.request.clause {
            if self.interpret_request(clause)? != session.request {
                return Err(ScopedMemoryError::Session(
                    "request interpretation differs from its observed clause".into(),
                ));
            }
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
                let capture_hop = capture.read_hop;
                let captured_entity = if capture_hop == session.hop {
                    session.query_entity.as_slice()
                } else if capture_hop.checked_add(1) == Some(session.hop) {
                    session
                        .visited
                        .get(capture_hop as usize)
                        .ok_or_else(|| {
                            ScopedMemoryError::Session("capture has no visited origin".into())
                        })?
                        .as_slice()
                } else {
                    return Err(ScopedMemoryError::Session(
                        "capture read position disagrees with progress".into(),
                    ));
                };
                let mut capture_fault: Option<&str> = None;
                if capture.payload.is_empty() {
                    capture_fault = Some("empty payload");
                } else if capture.payload.len()
                    + usize::from(session.eos.is_some())
                    + if self.contract.is_lexical() {
                        self.lexical_word_bound()
                    } else {
                        0
                    }
                    > SCOPED_MAX_ANSWER
                {
                    capture_fault = Some("answer bound");
                } else if capture
                    .payload
                    .iter()
                    .any(|token| *token as usize >= self.max_vocab)
                {
                    capture_fault = Some("out-of-vocabulary payload");
                } else if capture.key
                    != self.address(&session.scope, captured_entity, session.relation)
                {
                    capture_fault = Some("address");
                } else if capture.key != encode_key(&record.scope, &record.entity, record.relation)
                {
                    capture_fault = Some("record address");
                } else if capture.commit != record.commit {
                    capture_fault = Some("commit");
                } else if beyond_pin {
                    capture_fault = Some("beyond pin");
                } else if capture.value != record.value {
                    capture_fault = Some("value");
                } else if capture.continues != record.continues {
                    capture_fault = Some("continuation flag");
                } else if payload_sha256(&capture.payload) != record.payload_sha256 {
                    capture_fault = Some("payload witness");
                } else if !record.evicted && capture.payload != record.payload {
                    capture_fault = Some("payload");
                }
                if let Some(fault) = capture_fault {
                    return Err(ScopedMemoryError::Session(format!(
                        "capture disagrees with its exact owned record: {fault} (entity {:?} hop {} derived {})",
                        String::from_utf8_lossy(captured_entity),
                        capture_hop,
                        session.derived
                    )));
                }
                // Pinned exact selection remains stable even when a payload was later evicted.
                if !matches!(
                    self.control,
                    MemoryControl::Unpinned | MemoryControl::ParseScoreAuthority
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
                let terminators = usize::from(
                    session.terminal == Some(ScopedTerminal::Complete) && session.eos.is_some(),
                );
                match self.contract {
                    OutputContract::LegacyWords => {
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
                        if session.vocabulary_words != 0 || session.prelude_words != 0 {
                            return Err(ScopedMemoryError::Session(
                                "the legacy contract must emit no realization words".into(),
                            ));
                        }
                    }
                    _ => {
                        let prelude = session.prelude_words as usize;
                        let words = session.vocabulary_words as usize;
                        if session.emitted.len() != session.cursor + words + terminators
                            || prelude > words
                            || session.emitted.len() < prelude + session.cursor
                        {
                            return Err(ScopedMemoryError::Session(
                                "realized emissions disagree with the owned span, word count or terminator"
                                    .into(),
                            ));
                        }
                        if session.emitted[prelude..prelude + session.cursor]
                            != capture.payload[..session.cursor]
                        {
                            return Err(ScopedMemoryError::Session(
                                "realized emission does not carry the exact owned payload prefix"
                                    .into(),
                            ));
                        }
                        let slots = self.lexical_slots().ok_or_else(|| {
                            ScopedMemoryError::Session(
                                "lexical contract without a bound lexical artifact".into(),
                            )
                        })?;
                        let before = &session.emitted[..prelude];
                        let after = &session.emitted[prelude + session.cursor..];
                        let mut counted = 0usize;
                        for token in before.iter().chain(after.iter()) {
                            if Some(*token) == session.eos {
                                continue;
                            }
                            counted += 1;
                            if !slots.contains(token) {
                                return Err(ScopedMemoryError::Session(
                                    "realized vocabulary word is not from the learned slot set"
                                        .into(),
                                ));
                            }
                        }
                        if counted != words {
                            return Err(ScopedMemoryError::Session(
                                "realized word count disagrees with the emitted words".into(),
                            ));
                        }
                    }
                }
            }
            None => {
                if session.cursor != 0
                    || !session.emitted.is_empty()
                    || session.hop != 0
                    || session.vocabulary_words != 0
                    || session.prelude_words != 0
                {
                    return Err(ScopedMemoryError::Session(
                        "session has progress without an owned capture".into(),
                    ));
                }
            }
        }
        if session.op_cursor > session.ops.len() {
            return Err(ScopedMemoryError::Session(
                "operation cursor exceeds the observed operations".into(),
            ));
        }
        if session.derived && session.computation.is_none() {
            return Err(ScopedMemoryError::Session(
                "derived address without a computation".into(),
            ));
        }
        if let Some(computed) = &session.computation {
            // The result is published by the finishing Apply step, which is one step after the last
            // operation is applied; the interval between those two steps is a legal state.
            let published = !computed.derived_key.is_empty();
            if computed.applied != session.op_cursor
                || computed.artifact != self.binding.artifact_sha256
                || computed.source_scope != session.scope
                || session.ops.is_empty()
                || computed.source_hop > session.hop
            {
                return Err(ScopedMemoryError::Session(
                    "computed result disagrees with its session, artifact or cursor".into(),
                ));
            }
            if let Some(capture) = &session.captured {
                // Once the derived read replaces the source capture, the source provenance lives in
                // the computation itself; only the un-replaced source capture is compared here.
                if session.hop == computed.source_hop
                    && !capture.derived
                    && (computed.source_record != capture.record_id
                        || computed.source_commit != capture.commit
                        || computed.source_key != capture.key
                        || computed.operand_key != capture.value)
                {
                    return Err(ScopedMemoryError::Session(
                        "computed operand disagrees with its captured source".into(),
                    ));
                }
            } else {
                return Err(ScopedMemoryError::Session(
                    "computation without a captured operand".into(),
                ));
            }
            let source = self
                .memory
                .record_ref(computed.source_record)
                .ok_or_else(|| {
                    ScopedMemoryError::Session("computed source record is missing".into())
                })?;
            // Older non-state-lexical snapshots did not retain this field. They do not consume
            // it as lexical input, so their default empty value remains compatible. Whenever an
            // operand payload is supplied, and always for StateLexicalV1, bind its exact tokens to
            // the source's retained witness even after the live payload has been evicted.
            if self.contract == OutputContract::StateLexicalV1
                || !computed.operand_payload.is_empty()
            {
                if computed.operand_payload.is_empty()
                    || computed.operand_payload.len() > SCOPED_MAX_ANSWER
                    || computed
                        .operand_payload
                        .iter()
                        .any(|token| *token as usize >= self.max_vocab)
                    || payload_sha256(&computed.operand_payload) != source.payload_sha256
                    || (!source.evicted && computed.operand_payload != source.payload)
                {
                    return Err(ScopedMemoryError::Session(
                        "computed operand payload disagrees with its exact source witness".into(),
                    ));
                }
            }
            if computed.source_commit != source.commit
                || computed.source_entity != source.entity
                || computed.source_key != encode_key(&source.scope, &source.entity, source.relation)
                || computed.source_key
                    != self.address(&session.scope, &computed.source_entity, session.relation)
                || computed.operand_key != source.value
                || source.continues
                || (source.commit > session.view
                    && !matches!(self.control, MemoryControl::Unpinned))
                || (computed.source_hop < session.hop
                    && session.visited.get(computed.source_hop as usize)
                        != Some(&computed.source_entity))
            {
                return Err(ScopedMemoryError::Session(
                    "computed provenance disagrees with its exact source".into(),
                ));
            }
            if !matches!(
                self.control,
                MemoryControl::Unpinned | MemoryControl::ParseScoreAuthority
            ) {
                let history = if computed.source_hop == 0 {
                    session.history
                } else {
                    HistoryView::Current
                };
                if !matches!(self.memory.lookup_identity(&computed.source_key, session.view, history),
                    Lookup::Found(record) if record.id == source.id)
                {
                    return Err(ScopedMemoryError::Session(
                        "computed source was not eligible at the pinned view".into(),
                    ));
                }
            }
            let operand_label = self
                .lexicon
                .label_for(&computed.operand_key)
                .ok_or_else(|| {
                    ScopedMemoryError::Session("computed operand is ungrounded".into())
                })?;
            if self.backend.initial_state(operand_label).ok() != Some(computed.operand_state) {
                return Err(ScopedMemoryError::Session(
                    "computed operand state disagrees with the bound artifact".into(),
                ));
            }
            let skip = matches!(self.control, MemoryControl::ApplyDisabled);
            if skip && computed.applied != 0 {
                return Err(ScopedMemoryError::Session(
                    "disabled Apply has operation progress".into(),
                ));
            }
            let mut replayed = computed.operand_state;
            for op in session.ops.iter().take(computed.applied) {
                let label = self.lexicon.label_for(op).ok_or_else(|| {
                    ScopedMemoryError::Session("applied operation is ungrounded".into())
                })?;
                replayed = self.backend.apply(replayed, label)?;
            }
            if computed.state != replayed {
                return Err(ScopedMemoryError::Session(
                    "retained state disagrees with replayed observed operations".into(),
                ));
            }
            if published {
                if (!skip && computed.applied != session.ops.len())
                    || computed.consumed == matches!(self.control, MemoryControl::ConsumeDisabled)
                    || session.pending == SessionAction::Apply
                    || session.hop <= computed.source_hop
                {
                    return Err(ScopedMemoryError::Session(
                        "published result disagrees with operation progress or control".into(),
                    ));
                }
                let label = if skip {
                    Some(operand_label)
                } else {
                    self.backend.label_for_state(computed.state)
                }
                .ok_or_else(|| ScopedMemoryError::Session("computed state is ungrounded".into()))?;
                let key = self.lexicon.key_for_label(label).ok_or_else(|| {
                    ScopedMemoryError::Session("computed label has no canonical key".into())
                })?;
                if computed.derived_label != label || computed.derived_key != key {
                    return Err(ScopedMemoryError::Session(format!(
                        "computed result disagrees with the published grounding: state {} label {} vs {} key {:?} vs {:?}",
                        computed.state,
                        computed.derived_label,
                        label,
                        String::from_utf8_lossy(&computed.derived_key),
                        String::from_utf8_lossy(key)
                    )));
                }
                // The consumed address is the derived key (or, when consumption is disabled, the
                // operand's own label). The derived capture must own that exact address; a later hop
                // may legitimately have moved the query on.
                let consumed = if computed.consumed {
                    computed.derived_key.as_slice()
                } else {
                    computed.operand_key.as_slice()
                };
                let consumed_address = self.address(&session.scope, consumed, session.relation);
                if !session.derived {
                    return Err(ScopedMemoryError::Session(
                        "published computation is not being consumed".into(),
                    ));
                }
                if session
                    .visited
                    .get(computed.source_hop as usize + 1)
                    .map(Vec::as_slice)
                    .unwrap_or(session.query_entity.as_slice())
                    != consumed
                {
                    return Err(ScopedMemoryError::Session(
                        "next query was not the consumed result".into(),
                    ));
                }
                // Before the derived read completes, the retained capture is still the source operand;
                // once it completes, it must own exactly the consumed address.
                if let Some(capture) = &session.captured {
                    if capture.derived != (capture.read_hop == computed.source_hop + 1)
                        || (capture.derived && capture.key != consumed_address)
                    {
                        return Err(ScopedMemoryError::Session(format!(
                            "derived read addressed {:?} instead of the consumed result {:?}",
                            String::from_utf8_lossy(&capture.key),
                            String::from_utf8_lossy(consumed)
                        )));
                    }
                }
            } else if computed.consumed
                || session.derived
                || computed.derived_label != 0
                || (session.terminal.is_none() && session.pending != SessionAction::Apply)
                || session.hop != computed.source_hop
            {
                return Err(ScopedMemoryError::Session(
                    "unpublished computation already claims a consumed result".into(),
                ));
            }
        } else if session.op_cursor != 0 || session.captured.as_ref().is_some_and(|c| c.derived) {
            return Err(ScopedMemoryError::Session(
                "operation cursor advanced without a computation".into(),
            ));
        }
        // Every followed address must come from its selected record, except the one explicitly
        // computed edge. Tombstones retain the immutable identity/value needed for this check.
        if !matches!(
            self.control,
            MemoryControl::Unpinned | MemoryControl::ParseScoreAuthority
        ) {
            for (hop, entity) in session.visited.iter().enumerate() {
                if session
                    .computation
                    .as_ref()
                    .is_some_and(|c| c.source_hop as usize == hop)
                {
                    continue;
                }
                let history = if hop == 0 {
                    session.history
                } else {
                    HistoryView::Current
                };
                let record = match self.memory.lookup_identity(
                    &self.address(&session.scope, entity, session.relation),
                    session.view,
                    history,
                ) {
                    Lookup::Found(record) => record,
                    _ => {
                        return Err(ScopedMemoryError::Session(
                            "followed record is not eligible".into(),
                        ));
                    }
                };
                let next = session
                    .visited
                    .get(hop + 1)
                    .unwrap_or(&session.query_entity);
                if !record.continues || &record.value != next {
                    return Err(ScopedMemoryError::Session(
                        "followed address disagrees with its source value".into(),
                    ));
                }
            }
        }
        let terminators = usize::from(
            session.terminal == Some(ScopedTerminal::Complete) && session.eos.is_some(),
        );
        let realized_words = if self.contract.is_lexical() {
            session.vocabulary_words as usize
        } else {
            0
        };
        match session.terminal {
            Some(ScopedTerminal::Complete) => {
                if session.pending != SessionAction::Stop
                    || !session.captured.as_ref().is_some_and(|c| !c.continues)
                    || session.cursor == 0
                    || session.cursor != session.captured_payload_len()
                    || session.emitted.len() != session.cursor + terminators + realized_words
                {
                    return Err(ScopedMemoryError::Session(
                        "complete terminal requires the full owned payload".into(),
                    ));
                }
            }
            Some(_) => {
                if session.pending != SessionAction::Stop || !session.emitted.is_empty() {
                    return Err(ScopedMemoryError::Session(
                        "typed failure terminal must be silent".into(),
                    ));
                }
            }
            None => match session.pending {
                SessionAction::Read if session.cursor == 0 && session.emitted.is_empty() => {}
                SessionAction::Apply
                    if session.computation.is_some() && session.captured.is_some() => {}
                SessionAction::Continue
                    if session.captured.as_ref().is_some_and(|c| c.continues)
                        && session.cursor == 0
                        && session.emitted.is_empty() => {}
                SessionAction::Emit
                    if session.captured.as_ref().is_some_and(|c| !c.continues)
                        && (session.cursor < session.captured_payload_len()
                            || self.contract.is_lexical()) => {}
                SessionAction::Stop
                    if session.captured.as_ref().is_some_and(|c| !c.continues)
                        && session.cursor > 0
                        && session.cursor == session.captured_payload_len() => {}
                _ => {
                    return Err(ScopedMemoryError::Session(
                        "invalid active session phase".into(),
                    ));
                }
            },
        }
        self.validate_realized_progress(session)?;
        Ok(())
    }

    /// Reconstruct the reachable lexical prefix with the same emission operation used by serving.
    /// Slot membership alone cannot establish that the bound policy chose a word or reached Stop.
    /// This is bounded causal consistency, not authentication of a caller-supplied conversation.
    fn validate_realized_progress(&self, session: &ScopedSession) -> Result<(), ScopedMemoryError> {
        if !self.contract.is_lexical() {
            return Ok(());
        }
        let emitting = session.terminal == Some(ScopedTerminal::Complete)
            || (session.terminal.is_none()
                && matches!(session.pending, SessionAction::Emit | SessionAction::Stop));
        if !emitting {
            if !session.emitted.is_empty()
                || session.cursor != 0
                || session.vocabulary_words != 0
                || session.prelude_words != 0
                || !session.sl_state.is_empty()
            {
                return Err(ScopedMemoryError::Session(
                    "lexical progress precedes an emission-capable capture".into(),
                ));
            }
            return Ok(());
        }
        let mut replay = session.clone();
        replay.cursor = 0;
        replay.emitted.clear();
        replay.vocabulary_words = 0;
        replay.prelude_words = 0;
        replay.pending = SessionAction::Emit;
        replay.terminal = None;
        // The retained decoder state is a function of the emitted prefix, so a reachability replay
        // must rebuild it from the same starting point rather than trust the stored vector.
        replay.sl_state.clear();
        // Each transition emits a token, reaches the learned Stop, or emits the terminator. There
        // are at most SCOPED_MAX_ANSWER tokens; one extra iteration checks the completed frame.
        for _ in 0..SCOPED_MAX_ANSWER + 3 {
            if replay.cursor == session.cursor
                && replay.emitted == session.emitted
                && replay.vocabulary_words == session.vocabulary_words
                && replay.prelude_words == session.prelude_words
                && replay.pending == session.pending
                && replay.terminal == session.terminal
                && replay.sl_state == session.sl_state
            {
                return Ok(());
            }
            match (replay.pending, replay.terminal) {
                (SessionAction::Emit, None) => {
                    let mut effect = ScopedStepEffect {
                        action: SessionAction::Emit,
                        next_action: None,
                        selected_record: None,
                        selected_commit: None,
                        selected_value: None,
                        op_label: None,
                        computed_state: None,
                        hop: replay.hop,
                        emitted: None,
                        realization: None,
                        terminal: None,
                    };
                    // Calling the shared operation directly avoids recursively entering validate.
                    if self.contract == OutputContract::StateLexicalV1 {
                        self.state_lexical_emit(&mut replay, &mut effect)?;
                    } else {
                        self.realized_emit(&mut replay, &mut effect)?;
                    }
                }
                (SessionAction::Stop, None) => {
                    if let Some(eos) = replay.eos {
                        replay.emitted.push(eos);
                    }
                    replay.terminal = Some(ScopedTerminal::Complete);
                }
                _ => break,
            }
        }
        Err(ScopedMemoryError::Session(
            "realized emissions or phase are not reachable under the bound policy".into(),
        ))
    }

    fn terminate(&self, session: &mut ScopedSession, reason: ScopedTerminal) {
        session.terminal = Some(reason);
        session.pending = SessionAction::Stop;
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
            op_label: None,
            computed_state: None,
            hop: session.hop,
            emitted: None,
            realization: None,
            terminal: None,
        };
        match performed {
            SessionAction::Read => {
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
                                } else if self.contract.is_lexical()
                                    && record.payload.len()
                                        + self.lexical_word_bound()
                                        + usize::from(session.eos.is_some())
                                        > SCOPED_MAX_ANSWER
                                {
                                    // The realized contract reserves room for the learned words, so a
                                    // payload that cannot fit them plus its terminator is refused
                                    // before any token escapes rather than mid-response.
                                    self.terminate(session, ScopedTerminal::Exhausted);
                                } else {
                                    effect.selected_record = Some(record.id);
                                    effect.selected_commit = Some(record.commit);
                                    effect.selected_value = Some(record.value.clone());
                                    // Only the read of the consumed result is a *derived* capture.
                                    // Later hops of the same answer are ordinary dependent reads,
                                    // so the flag cannot be inherited by them.
                                    let derived_capture = session.derived
                                        && session.computation.as_ref().is_some_and(|c| {
                                            let consumed = if c.consumed {
                                                c.derived_key.as_slice()
                                            } else {
                                                c.operand_key.as_slice()
                                            };
                                            key == self.address(
                                                &session.scope,
                                                consumed,
                                                session.relation,
                                            )
                                        });
                                    let capture = MemoryCapture {
                                        key: key.clone(),
                                        record_id: record.id,
                                        commit: record.commit,
                                        value: record.value.clone(),
                                        payload: record.payload.clone(),
                                        continues: record.continues,
                                        derived: derived_capture,
                                        read_hop: session.hop,
                                    };
                                    let continues = capture.continues;
                                    let operand_key = capture.value.clone();
                                    session.captured = Some(capture);
                                    if session.ops.is_empty()
                                        || session.computation.is_some()
                                        || continues
                                    {
                                        session.pending = if continues {
                                            SessionAction::Continue
                                        } else {
                                            SessionAction::Emit
                                        };
                                    } else {
                                        // The source operand of a computation request starts the
                                        // computation instead of a continuation or an emission.
                                        match self.lexicon.label_for(&operand_key) {
                                            Some(label) => {
                                                match self.backend.initial_state(label) {
                                                    Ok(state) => {
                                                        session.computation = Some(ComputedState {
                                                            source_record: record.id,
                                                            source_commit: record.commit,
                                                            source_scope: session.scope.clone(),
                                                            source_entity: session
                                                                .query_entity
                                                                .clone(),
                                                            source_key: key,
                                                            source_hop: session.hop,
                                                            operand_key,
                                                            operand_state: state,
                                                            operand_payload: record.payload.clone(),
                                                            state,
                                                            applied: 0,
                                                            artifact: self
                                                                .binding
                                                                .artifact_sha256
                                                                .clone(),
                                                            derived_label: 0,
                                                            derived_key: Vec::new(),
                                                            consumed: false,
                                                        });
                                                        session.op_cursor = 0;
                                                        session.pending = SessionAction::Apply;
                                                    }
                                                    Err(_) => self.terminate(
                                                        session,
                                                        ScopedTerminal::UnknownOperand,
                                                    ),
                                                }
                                            }
                                            None => self
                                                .terminate(session, ScopedTerminal::UnknownOperand),
                                        }
                                    }
                                }
                            }
                            Lookup::Absent => self.terminate(session, ScopedTerminal::Unresolved),
                            Lookup::NoHistory => self.terminate(session, ScopedTerminal::NoHistory),
                            Lookup::Evicted => self.terminate(session, ScopedTerminal::Evicted),
                        }
                    }
                }
            }
            SessionAction::Apply => {
                let Some(mut computed) = session.computation.clone() else {
                    return Err(ScopedMemoryError::Session(
                        "apply without a computation".into(),
                    ));
                };
                let skip = matches!(self.control, MemoryControl::ApplyDisabled);
                if computed.applied < session.ops.len() && !skip {
                    let op_key = session.ops[computed.applied].clone();
                    let applied = self
                        .lexicon
                        .label_for(&op_key)
                        .ok_or(ScopedTerminal::UnknownOperation)
                        .and_then(|label| {
                            self.backend
                                .apply(computed.state, label)
                                .map(|state| (label, state))
                                .map_err(|_| ScopedTerminal::UnknownOperation)
                        });
                    match applied {
                        Ok((label, state)) => {
                            computed.state = state;
                            computed.applied += 1;
                            session.op_cursor = computed.applied;
                            effect.op_label = Some(label);
                            effect.computed_state = Some(state);
                            session.computation = Some(computed);
                            session.pending = SessionAction::Apply;
                        }
                        Err(reason) => {
                            session.computation = Some(computed);
                            self.terminate(session, reason);
                        }
                    }
                } else {
                    // Either every observed operation was applied, or the apply control skipped them.
                    // Ground the retained state (or the operand label under the skip control) and
                    // consume it as the next exact read address.
                    let grounded = if skip {
                        self.lexicon.label_for(&computed.operand_key).map(|label| {
                            (label, computed.operand_state, computed.operand_key.clone())
                        })
                    } else {
                        self.backend
                            .label_for_state(computed.state)
                            .and_then(|label| {
                                self.lexicon
                                    .key_for_label(label)
                                    .map(|key| (label, computed.state, key.to_vec()))
                            })
                    };
                    match grounded {
                        Some((label, state, key)) => {
                            computed.derived_label = label;
                            computed.derived_key = key.clone();
                            computed.state = state;
                            let consumed = !matches!(self.control, MemoryControl::ConsumeDisabled);
                            computed.consumed = consumed;
                            let source_entity = computed.source_entity.clone();
                            session.query_entity = if consumed {
                                key
                            } else {
                                computed.operand_key.clone()
                            };
                            effect.computed_state = Some(state);
                            effect.selected_value = Some(computed.derived_key.clone());
                            session.computation = Some(computed);
                            session.visited.push(source_entity);
                            session.hop += 1;
                            session.derived = true;
                            session.pending = SessionAction::Read;
                        }
                        None => {
                            session.computation = Some(computed);
                            self.terminate(session, ScopedTerminal::UngroundedResult);
                        }
                    }
                }
            }
            SessionAction::Continue => {
                let (value, continues) = match &session.captured {
                    Some(capture) => (capture.value.clone(), capture.continues),
                    None => {
                        return Err(ScopedMemoryError::Session(
                            "continue without a capture".into(),
                        ));
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
                session.pending = SessionAction::Read;
            }
            SessionAction::Emit => match self.contract {
                OutputContract::StateLexicalV1 => self.state_lexical_emit(session, &mut effect)?,
                OutputContract::RealizedV1 => self.realized_emit(session, &mut effect)?,
                OutputContract::LegacyWords => {
                    let (token, done) = match &session.captured {
                        Some(capture) => {
                            let token = *capture.payload.get(session.cursor).ok_or_else(|| {
                                ScopedMemoryError::Session("emission past the payload".into())
                            })?;
                            (token, session.cursor + 1 == capture.payload.len())
                        }
                        None => {
                            return Err(ScopedMemoryError::Session(
                                "emit without a capture".into(),
                            ));
                        }
                    };
                    if session.emitted.len() >= SCOPED_MAX_ANSWER {
                        self.terminate(session, ScopedTerminal::Exhausted);
                    } else {
                        session.emitted.push(token);
                        session.cursor += 1;
                        effect.emitted = Some(token);
                        if done {
                            session.pending = SessionAction::Stop;
                        }
                    }
                }
            },
            SessionAction::Stop => {
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

    /// The causal, target-free context that indexes the realization table for the current capture.
    /// Offline fitting may observe this same context on an executed session. Targets must not
    /// manufacture provenance/history flags that differ from this serving observation.
    pub fn realization_context(&self, session: &ScopedSession) -> RealizationContext {
        let len = session.captured_payload_len();
        let cursor = session.cursor;
        let copy_stage = if cursor == 0 {
            0
        } else if cursor < len {
            1
        } else {
            2
        };
        let (payload, value, derived, key, commit) = match &session.captured {
            Some(capture) => (
                capture.payload.as_slice(),
                capture.value.as_slice(),
                capture.derived,
                capture.key.as_slice(),
                capture.commit,
            ),
            None => (&[][..], &[][..], false, &[][..], 0),
        };
        // A bounded class of the owned evidence: the composed H4 element of the captured payload
        // through the bound model's learned per-token elements. This is the same exact composition
        // table the retained geometric paths use, evaluated as table reads only.
        let table = group_table();
        let mut state = table.identity as usize;
        for token in payload {
            let root = self.model.token_element(*token).min(GROUP_ORDER - 1);
            state = table.product[state * ROW_STRIDE + root] as usize;
        }
        let buckets = self
            .realization
            .as_ref()
            .map(|m| m.evidence_buckets.max(1) as usize)
            .unwrap_or(1)
            .min(GROUP_ORDER);
        let evidence_class = (state & (buckets - 1)) as u16;
        // Older eligible evidence: a superseded committed value for this exact address. This is a
        // property of the owned store history, not of the request text or the evaluator.
        let prior_differs = if commit == 0 || key.is_empty() {
            false
        } else {
            matches!(
                // This is an immutable lexical-metadata comparison, not a request for a released
                // payload. Future capacity eviction must not change an already-pinned answer.
                self.memory.lookup_identity(key, commit, HistoryView::PreviousAssertion),
                Lookup::Found(previous) if previous.value != value
            )
        };
        let context_disabled = matches!(self.control, MemoryControl::RealizationContextDisabled);
        RealizationContext {
            relation: session.relation,
            history: match session.history {
                HistoryView::Current => 0,
                HistoryView::PreviousAssertion => 1,
                HistoryView::PreviousDistinctValue => 2,
                HistoryView::Initial => 3,
            },
            derived: derived && !context_disabled,
            prior_differs: prior_differs && !context_disabled,
            evidence_class,
            copy_stage,
            emitted_bucket: session.vocabulary_words.min(3),
        }
    }

    /// Declared bound on learned insert words per answer under the bound contract.
    fn lexical_word_bound(&self) -> usize {
        match self.contract {
            OutputContract::StateLexicalV1 => self
                .state_lexical
                .as_ref()
                .map(|m| m.max_insert_words as usize)
                .unwrap_or(SCOPED_MAX_VOCAB_WORDS),
            _ => SCOPED_MAX_VOCAB_WORDS,
        }
    }

    /// The bound learned vocabulary slot tokens, under either lexical contract.
    fn lexical_slots(&self) -> Option<&[u32]> {
        self.state_lexical
            .as_ref()
            .map(|m| m.slots.as_slice())
            .or_else(|| self.realization.as_ref().map(|m| m.slots.as_slice()))
    }

    /// The requested-history code the decoder consumes.
    fn history_code(session: &ScopedSession) -> u8 {
        match session.history {
            HistoryView::Current => 0,
            HistoryView::PreviousAssertion => 1,
            HistoryView::PreviousDistinctValue => 2,
            HistoryView::Initial => 3,
        }
    }

    /// Whether an older committed value for the same address was superseded before this one. This is
    /// an immutable lexical-metadata comparison, not a request for a released payload, so later
    /// capacity eviction cannot change an already-pinned answer.
    fn prior_differs(&self, session: &ScopedSession) -> bool {
        let Some(capture) = &session.captured else {
            return false;
        };
        if capture.commit == 0 || capture.key.is_empty() {
            return false;
        }
        matches!(
            self.memory.lookup_identity(
                &capture.key,
                capture.commit,
                HistoryView::PreviousAssertion
            ),
            Lookup::Found(previous) if previous.value != capture.value
        )
    }

    /// The copy stage of the owned span: 0 before it, 1 inside it, 2 once it is complete.
    fn copy_stage(session: &ScopedSession) -> u8 {
        let len = session.captured_payload_len();
        let cursor = session.cursor;
        if cursor == 0 {
            0
        } else if cursor < len {
            1
        } else {
            2
        }
    }

    /// The content features of the state-conditioned decoder: the selected owned payload tokens and,
    /// for a consumed computation, the operand payload the computation started from. The context
    /// control zeroes them, so the ablation isolates the causal evidence from the copy placement.
    fn state_lexical_content(&self, session: &ScopedSession) -> (Vec<u32>, Vec<u32>) {
        if matches!(self.control, MemoryControl::RealizationContextDisabled) {
            return (Vec::new(), Vec::new());
        }
        let sel = session
            .captured
            .as_ref()
            .map(|capture| capture.payload.clone())
            .unwrap_or_default();
        let res = if session.derived {
            session
                .computation
                .as_ref()
                .map(|computed| computed.operand_payload.clone())
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        (sel, res)
    }

    /// The causal flag block the decoder consumes, under the context control.
    fn state_lexical_flags(&self, session: &ScopedSession) -> Vec<i32> {
        let disabled = matches!(self.control, MemoryControl::RealizationContextDisabled);
        let history = if disabled {
            0
        } else {
            Self::history_code(session)
        };
        let derived = session.derived && !disabled;
        let prior = !disabled && self.prior_differs(session);
        StateLexicalModel::flags(history, derived, prior)
    }

    /// One realized emission step under the versioned `StateLexicalV1` contract.
    ///
    /// The learned state-conditioned decoder chooses, at every emission position, between completing
    /// the exact owned token (`Copy`), inserting one learned vocabulary word, and `Stop`. The same
    /// owned structural invariants as `RealizedV1` keep the span exact: `Copy` is forced inside the
    /// payload; a premature Stop is replaced with Copy. The decoder's state advances with the
    /// executed insert-slot/Copy event, so the reachability replay and a restored snapshot agree
    /// with serving. Copied-token identity is not part of this artifact's recurrent feedback.
    fn state_lexical_emit(
        &self,
        session: &mut ScopedSession,
        effect: &mut ScopedStepEffect,
    ) -> Result<(), ScopedMemoryError> {
        let model = self.state_lexical.as_ref().ok_or_else(|| {
            ScopedMemoryError::Session(
                "state-lexical contract without a state-lexical artifact".into(),
            )
        })?;
        let (sel, res) = self.state_lexical_content(session);
        let m = model.content_feature(&sel, &res);
        let f = self.state_lexical_flags(session);
        if session.sl_state.is_empty() {
            session.sl_state = model.init_state(&m, &f);
        }
        let stage = Self::copy_stage(session);
        let (chosen, _logits) = model.decide(&session.sl_state, &m, &f, stage);
        let room = session.vocabulary_words < model.max_insert_words;
        let permitted = match (stage, chosen) {
            (0, RealizationAction::Insert(_)) => room,
            (0, RealizationAction::Copy) => true,
            (1, RealizationAction::Copy) => true,
            (2, RealizationAction::Insert(_)) => room,
            (2, RealizationAction::Stop) => true,
            _ => false,
        };
        let action = if permitted {
            chosen
        } else {
            match stage {
                2 => RealizationAction::Stop,
                _ => RealizationAction::Copy,
            }
        };
        effect.realization = Some(RealizationDecision {
            action,
            from_table: permitted,
            key: StateLexicalModel::state_digest(&session.sl_state),
            evidence_class: 0,
        });
        match action {
            RealizationAction::Copy => {
                let token = *session
                    .captured
                    .as_ref()
                    .and_then(|capture| capture.payload.get(session.cursor))
                    .ok_or_else(|| {
                        ScopedMemoryError::Session("emission past the payload".into())
                    })?;
                session.emitted.push(token);
                session.cursor += 1;
                effect.emitted = Some(token);
            }
            RealizationAction::Insert(slot) => {
                let token = *model.slots.get(slot as usize).ok_or_else(|| {
                    ScopedMemoryError::Session("state-lexical slot is not declared".into())
                })?;
                session.emitted.push(token);
                session.vocabulary_words += 1;
                if session.cursor == 0 {
                    session.prelude_words += 1;
                }
                effect.emitted = Some(token);
            }
            RealizationAction::Stop => {
                session.pending = SessionAction::Stop;
            }
        }
        // Advance by the executed action event. Copy uses one shared symbol regardless of its token;
        // the recurrence control holds the initial state instead.
        if !matches!(action, RealizationAction::Stop)
            && !matches!(self.control, MemoryControl::RecurrenceDisabled)
        {
            session.sl_state =
                model.transition(&session.sl_state, StateLexicalModel::symbol_of(action));
        }
        Ok(())
    }

    /// One realized emission step under the versioned `RealizedV1` contract.
    ///
    /// Owned structural invariants keep the exact span: a learned vocabulary word may never
    /// interrupt the payload (`Copy` is forced while `0 < cursor < len`) and the payload may never be
    /// truncated (`Stop` is forced while `cursor < len`). The learned table therefore decides which
    /// words surround the span and where to end; the span itself stays exactly owned. A decision
    /// overridden by an invariant is reported with `from_table = false` so no learned credit is
    /// claimed for the owned guarantee.
    fn realized_emit(
        &self,
        session: &mut ScopedSession,
        effect: &mut ScopedStepEffect,
    ) -> Result<(), ScopedMemoryError> {
        let Some(model) = self.realization.as_ref() else {
            return Err(ScopedMemoryError::Session(
                "realized contract without a realization artifact".into(),
            ));
        };
        let ctx = self.realization_context(session);
        let decision = model.decide(&ctx);
        let room = session.vocabulary_words < SCOPED_MAX_VOCAB_WORDS as u8;
        // The owned structural invariants keep the span exact and complete. An action they override is
        // reported with `from_table = false`, so no learned credit is claimed for the guarantee.
        let permitted = match (ctx.copy_stage, decision.action) {
            (0, RealizationAction::Insert(_)) => room,
            (0, RealizationAction::Copy) => true,
            (1, RealizationAction::Copy) => true,
            (2, RealizationAction::Insert(_)) => room,
            (2, RealizationAction::Stop) => true,
            _ => false,
        };
        let action = if permitted {
            decision.action
        } else {
            match ctx.copy_stage {
                2 => RealizationAction::Stop,
                _ => RealizationAction::Copy,
            }
        };
        effect.realization = Some(RealizationDecision {
            action,
            from_table: decision.from_table && permitted,
            key: decision.key,
            evidence_class: ctx.evidence_class,
        });
        match action {
            RealizationAction::Copy => {
                let token = *session
                    .captured
                    .as_ref()
                    .and_then(|capture| capture.payload.get(session.cursor))
                    .ok_or_else(|| {
                        ScopedMemoryError::Session("emission past the payload".into())
                    })?;
                session.emitted.push(token);
                session.cursor += 1;
                effect.emitted = Some(token);
            }
            RealizationAction::Insert(slot) => {
                let token = *model.slots.get(slot as usize).ok_or_else(|| {
                    ScopedMemoryError::Session("realization slot is not declared".into())
                })?;
                session.emitted.push(token);
                session.vocabulary_words += 1;
                if session.cursor == 0 {
                    session.prelude_words += 1;
                }
                effect.emitted = Some(token);
            }
            RealizationAction::Stop => {
                session.pending = SessionAction::Stop;
            }
        }
        Ok(())
    }

    /// Run to a terminal under the declared bounds.
    pub fn run(
        &self,
        session: &mut ScopedSession,
    ) -> Result<Vec<ScopedStepEffect>, ScopedMemoryError> {
        self.validate(session)?;
        let bound = SCOPED_MAX_HOPS as usize * 2 + SCOPED_MAX_ANSWER + session.ops.len() + 4;
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
                action: SessionAction::Stop,
                next_action: None,
                selected_record: None,
                selected_commit: None,
                selected_value: None,
                op_label: None,
                computed_state: None,
                hop: session.hop,
                emitted: None,
                realization: None,
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
        let value: serde_json::Value = serde_json::from_slice(bytes)
            .map_err(|e| ScopedMemoryError::Serialization(e.to_string()))?;
        if value["version"] != serde_json::json!(SCOPED_SESSION_VERSION) {
            return Err(ScopedMemoryError::Session(
                "unsupported session snapshot version".into(),
            ));
        }
        let session: ScopedSession = serde_json::from_value(value)
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

    /// A minimal valid lexicon for tests that exercise the memory phases without computation.
    fn test_lexicon() -> GroundingLexicon {
        GroundingLexicon::from_observations(&[(b"A".to_vec(), 1)], &[(1, b"A".to_vec())]).unwrap()
    }

    fn runtime(memory: Memory, eos: Option<u32>) -> ScopedMemoryRuntime {
        ScopedMemoryRuntime::load_with_computation(
            &ObservedTextModel::uninformed().to_bytes().unwrap(),
            &IntentModel::uninformed().to_bytes().unwrap(),
            Some(&test_lexicon().to_bytes().unwrap()),
            ComputationBackend::Absent,
            memory.clone(),
            memory.lineage,
            MemoryControl::Normal,
            4096,
            eos,
        )
        .unwrap()
    }

    fn replay_test_realization() -> RealizationModel {
        use super::super::lexical_realization::{fit_realization, RealizationExample};
        let mut examples = Vec::new();
        for prior in [false, true] {
            for (stage, words, action) in [
                (0, 0, RealizationAction::Insert(u8::from(prior))),
                (0, 1, RealizationAction::Copy),
                (1, 1, RealizationAction::Copy),
                (2, 1, RealizationAction::Insert(2)),
                (2, 2, RealizationAction::Stop),
            ] {
                examples.push(RealizationExample {
                    context: RealizationContext {
                        relation: 0,
                        history: 0,
                        derived: false,
                        prior_differs: prior,
                        evidence_class: 0,
                        copy_stage: stage,
                        emitted_bucket: words,
                    },
                    action,
                });
            }
        }
        fit_realization(&examples, 4096, vec![11, 12, 13], 8).unwrap()
    }

    fn replay_test_runtime(memory: Memory, eos: Option<u32>) -> ScopedMemoryRuntime {
        ScopedMemoryRuntime::load_grounded(
            &ObservedTextModel::uninformed().to_bytes().unwrap(),
            &IntentModel::uninformed().to_bytes().unwrap(),
            None,
            ComputationBackend::Absent,
            Some(&replay_test_realization().to_bytes().unwrap()),
            OutputContract::RealizedV1,
            memory.clone(),
            memory.lineage,
            MemoryControl::Normal,
            4096,
            eos,
        )
        .unwrap()
    }

    #[test]
    fn realized_restore_replays_words_stop_and_terminator_at_every_boundary() {
        let mut memory = Memory::new(31, 2);
        write(&mut memory, "Ova", "A", Update::Assert);
        let runtime = replay_test_runtime(memory, Some(4095));
        let mut session = runtime
            .ask_view(b"alpha", 0, b"Ova", HistoryView::Current)
            .unwrap();
        let mut boundaries = Vec::new();
        loop {
            let bytes = runtime.snapshot(&session).unwrap();
            assert_eq!(runtime.restore(&bytes).unwrap(), session);
            boundaries.push(bytes);
            if session.terminal.is_some() {
                break;
            }
            runtime.step(&mut session).unwrap();
            if session.cursor == session.captured_payload_len()
                && session.cursor > 0
                && session.vocabulary_words == 1
            {
                let mut premature_stop = session.clone();
                premature_stop.pending = SessionAction::Stop;
                assert!(runtime
                    .restore(&serde_json::to_vec(&premature_stop).unwrap())
                    .is_err());
            }
            if session.cursor == 1 {
                let mut interrupted = session.clone();
                interrupted.emitted.push(13);
                interrupted.vocabulary_words += 1;
                assert!(runtime.validate(&interrupted).is_err());
            }
        }
        assert_eq!(session.emitted, vec![11, 1, 2, 3, 13, 4095]);
        for bytes in boundaries {
            let mut restored = runtime.restore(&bytes).unwrap();
            runtime.run(&mut restored).unwrap();
            assert_eq!(restored, session);
        }
        let mut wrong_slot = session.clone();
        wrong_slot.emitted[0] = 12; // Still a valid slot, but not the selected one.
        assert!(runtime
            .restore(&serde_json::to_vec(&wrong_slot).unwrap())
            .is_err());
        let mut misplaced_eos = session;
        let last = misplaced_eos.emitted.len() - 1;
        misplaced_eos.emitted.swap(0, last);
        assert!(runtime
            .restore(&serde_json::to_vec(&misplaced_eos).unwrap())
            .is_err());
    }

    #[test]
    fn realized_restore_rejects_unearned_word_counters_before_capture() {
        let mut memory = Memory::new(32, 2);
        write(&mut memory, "Ova", "A", Update::Assert);
        let runtime = replay_test_runtime(memory, None);
        let mut session = runtime
            .ask_view(b"alpha", 0, b"Ova", HistoryView::Current)
            .unwrap();
        session.vocabulary_words = 1;
        session.prelude_words = 1;
        assert!(runtime
            .restore(&serde_json::to_vec(&session).unwrap())
            .is_err());
    }

    /// Fit a tiny state-lexical decoder over the declared development sequences. Slot 0 opens every
    /// answer, slot 1 is the present connective and slot 2 the past connective; the historical
    /// wording is conditioned on the owned-evidence flags, exactly as the scoped session presents
    /// them.
    fn replay_test_state_lexical() -> StateLexicalModel {
        use super::super::state_lexical::{fit_state_lexical, SlFitConfig, SlSequence};
        let mut examples = Vec::new();
        for prior in [false, true] {
            for history in [0u8, 1, 2, 3] {
                for derived in [false, true] {
                    let second = if history > 0 { 2 } else { 1 };
                    let mut actions = vec![
                        RealizationAction::Insert(0),
                        RealizationAction::Insert(second),
                    ];
                    actions.extend(vec![RealizationAction::Copy; 3]);
                    actions.push(RealizationAction::Stop);
                    examples.push(SlSequence {
                        sel: vec![1, 2, 3],
                        res: vec![],
                        history,
                        derived,
                        prior_differs: prior,
                        actions,
                    });
                }
            }
        }
        let cfg = SlFitConfig {
            epochs: 900,
            h_dim: 20,
            e_dim: 16,
            ..Default::default()
        };
        fit_state_lexical(&examples, 4096, vec![11, 12, 13], 4, &cfg)
            .unwrap()
            .model
    }

    fn replay_test_sl_runtime(
        memory: Memory,
        control: MemoryControl,
        eos: Option<u32>,
    ) -> ScopedMemoryRuntime {
        ScopedMemoryRuntime::load_grounded(
            &ObservedTextModel::uninformed().to_bytes().unwrap(),
            &IntentModel::uninformed().to_bytes().unwrap(),
            None,
            ComputationBackend::Absent,
            Some(&replay_test_state_lexical().to_bytes().unwrap()),
            OutputContract::StateLexicalV1,
            memory.clone(),
            memory.lineage,
            control,
            4096,
            eos,
        )
        .unwrap()
    }

    /// A deterministic Copy/Stop policy used only to exercise snapshot authority. No fitted
    /// language or score claim is attached to this fixture.
    fn safety_state_lexical() -> StateLexicalModel {
        use super::super::state_lexical::{SlLinear, SL_VERSION};
        let zero = |rows: usize, cols: usize| SlLinear {
            rows,
            cols,
            packed: vec![0; (rows * cols).div_ceil(4)],
            shift: vec![0; rows],
        };
        let mut wo = zero(3, 14);
        // Stop beats Copy only when the complete-copy-stage feature is one.
        let stop_complete = 14 + 13;
        wo.packed[stop_complete >> 2] |= 1 << ((stop_complete & 3) * 2);
        StateLexicalModel {
            version: SL_VERSION,
            vocab: 4096,
            slots: vec![11],
            content_tokens: vec![11],
            h_dim: 1,
            e_dim: 2,
            f_dim: 6,
            c_dim: 3,
            h_clamp: 255,
            m_clamp: 255,
            e: vec![1],
            wi: zero(1, 10),
            wt: zero(1, 4),
            wo,
            bi: vec![0],
            bt: vec![0],
            bo: vec![0, 0, -1],
            max_insert_words: 1,
        }
    }

    fn safety_lexical_runtime() -> ScopedMemoryRuntime {
        let old = computation_runtime(MemoryControl::Normal);
        ScopedMemoryRuntime::load_grounded(
            &old.model.to_bytes().unwrap(),
            &old.intent.to_bytes().unwrap(),
            Some(&old.lexicon.to_bytes().unwrap()),
            old.backend.clone(),
            Some(&safety_state_lexical().to_bytes().unwrap()),
            OutputContract::StateLexicalV1,
            old.memory.clone(),
            old.memory.lineage,
            MemoryControl::Normal,
            4096,
            Some(4095),
        )
        .unwrap()
    }

    #[test]
    fn state_lexical_restore_binds_exact_recurrent_state_at_every_phase() {
        let rt = safety_lexical_runtime();
        let mut session = rt
            .ask_compute(b"alpha", 0, b"Mara", vec![b"i".to_vec()])
            .unwrap();
        loop {
            let saved = rt.snapshot(&session).unwrap();
            assert_eq!(rt.restore(&saved).unwrap(), session);
            let mut forged = session.clone();
            forged.sl_state = vec![1]; // Correct width and clamp, but unreachable from this prefix.
            assert!(rt.restore(&serde_json::to_vec(&forged).unwrap()).is_err());
            if !session.sl_state.is_empty() {
                let mut omitted = session.clone();
                omitted.sl_state.clear();
                assert!(rt.restore(&serde_json::to_vec(&omitted).unwrap()).is_err());
                let mut huge = session.clone();
                huge.sl_state[0] = i32::MAX;
                assert!(rt.restore(&serde_json::to_vec(&huge).unwrap()).is_err());
            }
            if session.terminal.is_some() {
                break;
            }
            rt.step(&mut session).unwrap();
        }
    }

    #[test]
    fn computed_operand_payload_is_bound_after_source_eviction_with_legacy_compatibility() {
        let rt = safety_lexical_runtime();
        let mut session = rt
            .ask_compute(b"alpha", 0, b"Mara", vec![b"i".to_vec()])
            .unwrap();
        while !session.derived || session.pending != SessionAction::Emit {
            rt.step(&mut session).unwrap();
        }
        let source_id = session.computation.as_ref().unwrap().source_record;
        let mut memory = rt.memory.clone();
        for i in 0..3 {
            memory
                .write(
                    b"alpha",
                    b"Mara",
                    0,
                    b"L2",
                    &[12],
                    100 + i,
                    Update::Correct,
                    false,
                    0,
                )
                .unwrap();
        }
        assert!(memory.record_ref(source_id).unwrap().evicted);
        let newer = rt.with_memory(memory, MemoryControl::Normal).unwrap();
        assert_eq!(
            newer.restore(&rt.snapshot(&session).unwrap()).unwrap(),
            session
        );
        for payload in [
            vec![],
            vec![12],
            vec![4096],
            vec![11; SCOPED_MAX_ANSWER + 1],
        ] {
            let mut forged = session.clone();
            forged.computation.as_mut().unwrap().operand_payload = payload;
            assert!(newer
                .restore(&serde_json::to_vec(&forged).unwrap())
                .is_err());
        }
        let mut retained = session.clone();
        newer.run(&mut retained).unwrap();
        assert_eq!(retained.terminal, Some(ScopedTerminal::Complete));

        let legacy = computation_runtime(MemoryControl::Normal);
        let mut old = legacy
            .ask_compute(b"alpha", 0, b"Mara", vec![b"i".to_vec()])
            .unwrap();
        legacy.step(&mut old).unwrap();
        legacy.step(&mut old).unwrap();
        let mut omitted = serde_json::to_value(&old).unwrap();
        omitted["computation"]
            .as_object_mut()
            .unwrap()
            .remove("operand_payload");
        assert!(legacy
            .restore(&serde_json::to_vec(&omitted).unwrap())
            .is_ok());
        old.computation.as_mut().unwrap().operand_payload = vec![12];
        assert!(legacy.restore(&serde_json::to_vec(&old).unwrap()).is_err());
    }

    #[test]
    fn state_lexical_contract_emits_learned_words_around_the_exact_span_and_restores() {
        // Direct: prior_differs is false, so the present connective is learned.
        let mut direct_memory = Memory::new(61, 2);
        write(&mut direct_memory, "Ova", "A", Update::Assert);
        let direct = replay_test_sl_runtime(direct_memory, MemoryControl::Normal, Some(4095));
        let mut session = direct
            .ask_view(b"alpha", 0, b"Ova", HistoryView::Current)
            .unwrap();
        let mut boundaries = Vec::new();
        loop {
            let bytes = direct.snapshot(&session).unwrap();
            assert_eq!(direct.restore(&bytes).unwrap(), session);
            boundaries.push(bytes);
            if session.terminal.is_some() {
                break;
            }
            direct.step(&mut session).unwrap();
        }
        assert_eq!(session.terminal, Some(ScopedTerminal::Complete));
        assert_eq!(session.emitted, vec![11, 12, 1, 2, 3, 4095]);
        // Every saved frame replays to the same terminal answer.
        for bytes in boundaries {
            let mut restored = direct.restore(&bytes).unwrap();
            direct.run(&mut restored).unwrap();
            assert_eq!(restored, session);
        }

        // A current value stays present even when a different predecessor exists. History intent,
        // not the prior-difference flag, controls tense in this declared fixture.
        let mut sup_memory = Memory::new(62, 2);
        write(&mut sup_memory, "Ova", "A", Update::Assert);
        write(&mut sup_memory, "Ova", "B", Update::Correct);
        let sup = replay_test_sl_runtime(sup_memory, MemoryControl::Normal, Some(4095));
        let mut session = sup
            .ask_view(b"alpha", 0, b"Ova", HistoryView::Current)
            .unwrap();
        let effects = sup.run(&mut session).unwrap();
        assert_eq!(session.emitted, vec![11, 12, 1, 2, 3, 4095]);
        // The retained decoder state is carried in the frames and is non-trivial at the last step.
        assert_eq!(session.sl_state.len(), 20);
        assert!(effects.iter().any(|e| e.realization.is_some()));
    }

    #[test]
    fn state_lexical_recurrence_control_changes_the_emitted_sequence() {
        let mut memory = Memory::new(63, 2);
        write(&mut memory, "Ova", "A", Update::Assert);
        let normal = replay_test_sl_runtime(memory.clone(), MemoryControl::Normal, Some(4095));
        let frozen = replay_test_sl_runtime(memory, MemoryControl::RecurrenceDisabled, Some(4095));
        let mut a = normal
            .ask_view(b"alpha", 0, b"Ova", HistoryView::Current)
            .unwrap();
        normal.run(&mut a).unwrap();
        let mut b = frozen
            .ask_view(b"alpha", 0, b"Ova", HistoryView::Current)
            .unwrap();
        frozen.run(&mut b).unwrap();
        assert_ne!(
            a.emitted, b.emitted,
            "holding the decoder state must change the emitted sequence"
        );
        // The frozen control still emits a structurally valid frame under the same invariants.
        frozen.validate(&b).unwrap();
    }

    #[test]
    fn realized_pinned_words_survive_predecessor_and_origin_payload_eviction() {
        let mut memory = Memory::new(33, 2);
        write(&mut memory, "Ova", "A", Update::Assert);
        write(&mut memory, "Ova", "B", Update::Correct);
        let mut runtime = replay_test_runtime(memory, Some(4095));
        let mut captured = runtime
            .ask_view(b"alpha", 0, b"Ova", HistoryView::Current)
            .unwrap();
        runtime.step(&mut captured).unwrap();
        let captured_bytes = runtime.snapshot(&captured).unwrap();
        let mut expected = captured.clone();
        runtime.run(&mut expected).unwrap();
        assert_eq!(expected.emitted, vec![12, 1, 2, 3, 13, 4095]);
        let mut mid_word = captured;
        runtime.step(&mut mid_word).unwrap();
        let mid_word_bytes = runtime.snapshot(&mid_word).unwrap();
        let complete_bytes = runtime.snapshot(&expected).unwrap();

        for value in ["C", "D"] {
            write(&mut runtime.memory, "Ova", value, Update::Correct);
            assert!(runtime.memory.records[0].evicted);
            for bytes in [&captured_bytes, &mid_word_bytes, &complete_bytes] {
                let mut restored = runtime.restore(bytes).unwrap();
                runtime.run(&mut restored).unwrap();
                assert_eq!(restored, expected);
            }
        }
        assert!(runtime.memory.records[1].evicted);
        assert!(runtime.memory.records[1].payload.is_empty());
        assert!(runtime.validate(&expected).is_ok());
    }

    #[test]
    fn realized_vocabulary_cannot_bind_the_protocol_terminator_as_a_word() {
        let model = ObservedTextModel::uninformed().to_bytes().unwrap();
        let intent = IntentModel::uninformed().to_bytes().unwrap();
        let realization = replay_test_realization().to_bytes().unwrap();
        assert!(matches!(
            ScopedMemoryRuntime::load_grounded(
                &model,
                &intent,
                None,
                ComputationBackend::Absent,
                Some(&realization),
                OutputContract::RealizedV1,
                Memory::new(34, 2),
                34,
                MemoryControl::Normal,
                4096,
                Some(11),
            ),
            Err(ScopedMemoryError::Model(_))
        ));
    }

    /// The realized contract: one learned vocabulary word selected from owned evidence, with the
    /// exact span retained, the legacy contract unchanged, and the context-disabled control
    /// collapsing the evidence-conditioned choice.
    #[test]
    fn realized_contract_selects_a_learned_word_from_owned_evidence() {
        use super::super::lexical_realization::{fit_realization, RealizationExample};
        let base = |prior: bool, copy_stage: u8, bucket: u8| RealizationContext {
            relation: 0,
            history: 0,
            derived: false,
            prior_differs: prior,
            evidence_class: 0,
            copy_stage,
            emitted_bucket: bucket,
        };
        let realization = fit_realization(
            &[
                RealizationExample {
                    context: base(false, 0, 0),
                    action: RealizationAction::Insert(0),
                },
                RealizationExample {
                    context: base(true, 0, 0),
                    action: RealizationAction::Insert(1),
                },
                RealizationExample {
                    context: base(false, 0, 1),
                    action: RealizationAction::Copy,
                },
                RealizationExample {
                    context: base(true, 0, 1),
                    action: RealizationAction::Copy,
                },
                RealizationExample {
                    context: base(false, 2, 1),
                    action: RealizationAction::Stop,
                },
                RealizationExample {
                    context: base(true, 2, 1),
                    action: RealizationAction::Stop,
                },
            ],
            4096,
            vec![11, 12, 13],
            8,
        )
        .unwrap();
        let realization_bytes = realization.to_bytes().unwrap();
        let model = ObservedTextModel::uninformed().to_bytes().unwrap();
        let intent = IntentModel::uninformed().to_bytes().unwrap();
        let lexicon = test_lexicon().to_bytes().unwrap();

        // One store has a superseded older value for the same address; the other does not.
        let store = |superseded: bool| {
            let mut memory = Memory::new(1, 4);
            if superseded {
                memory
                    .write(
                        b"alpha",
                        b"Ova",
                        0,
                        b"B",
                        &[7, 7],
                        1,
                        Update::Assert,
                        false,
                        0,
                    )
                    .unwrap();
            }
            memory
                .write(
                    b"alpha",
                    b"Ova",
                    0,
                    b"C",
                    &[7, 7],
                    2,
                    Update::Assert,
                    false,
                    0,
                )
                .unwrap();
            memory
        };
        let run = |superseded: bool, control: MemoryControl, contract: OutputContract| {
            let memory = store(superseded);
            let lineage = memory.lineage;
            let rt = ScopedMemoryRuntime::load_grounded(
                &model,
                &intent,
                Some(&lexicon),
                ComputationBackend::Absent,
                Some(&realization_bytes),
                contract,
                memory,
                lineage,
                control,
                4096,
                None,
            )
            .unwrap();
            let mut session = rt
                .ask_view(b"alpha", 0, b"Ova", HistoryView::Current)
                .unwrap();
            rt.run(&mut session).unwrap();
            assert_eq!(session.terminal, Some(ScopedTerminal::Complete));
            session.emitted
        };

        // The request and the copied payload are identical; only older owned evidence differs.
        let direct = run(false, MemoryControl::Normal, OutputContract::RealizedV1);
        let superseded = run(true, MemoryControl::Normal, OutputContract::RealizedV1);
        assert_eq!(direct, vec![11, 7, 7]);
        assert_eq!(superseded, vec![12, 7, 7]);
        assert_eq!(direct[1..], superseded[1..]);
        assert_ne!(direct[0], superseded[0]);
        // The retained legacy contract on the same artifact stays the exact owned payload.
        assert_eq!(
            run(true, MemoryControl::Normal, OutputContract::LegacyWords),
            vec![7, 7]
        );
        // Holding the contextual flags at zero collapses the evidence-conditioned pair.
        assert_eq!(
            run(
                true,
                MemoryControl::RealizationContextDisabled,
                OutputContract::RealizedV1
            ),
            direct
        );
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

    /// Q8 development observations over eight labels with two noncommuting generators. The
    /// factorization recovers the action; the label/primitive identifiers are opaque.
    fn q8_fixture() -> (GroundedFactorization, u32, u32, Vec<u32>) {
        let witness = super::super::shared_transition::q8_witness().unwrap();
        let t = group_table();
        let product = |a: usize, b: usize| t.product[a * ROW_STRIDE + b] as usize;
        let mut pair = None;
        for a in witness.iter() {
            for b in witness.iter() {
                if product(*a as usize, *b as usize) != product(*b as usize, *a as usize) {
                    pair = Some((*a as u32, *b as u32));
                    break;
                }
            }
            if pair.is_some() {
                break;
            }
        }
        let (gen_i, gen_j) = pair.unwrap();
        let labels: Vec<u32> = (1..=8u32).collect();
        let label_of = |element: usize| -> u32 {
            let index = witness.iter().position(|w| *w as usize == element).unwrap();
            labels[index]
        };
        let mut examples = Vec::new();
        for (k, label) in labels.iter().enumerate() {
            let state = witness[k] as usize;
            // Both orders, so each primitive is observed in both positions over every outcome. The
            // factorization needs that coverage; it is a property of the declared observations.
            let after_i = product(gen_i as usize, state);
            let after_ij = product(gen_j as usize, after_i);
            let after_j = product(gen_j as usize, state);
            let after_ji = product(gen_i as usize, after_j);
            examples.push(super::super::shared_transition::StExample {
                payload: *label,
                primitives: vec![100, 200],
                targets: vec![label_of(after_i), label_of(after_ij)],
            });
            examples.push(super::super::shared_transition::StExample {
                payload: *label,
                primitives: vec![200, 100],
                targets: vec![label_of(after_j), label_of(after_ji)],
            });
        }
        let (factorization, report) =
            super::super::grounded_session::factor_observed_graph(&examples, false).unwrap();
        assert_eq!(report.transitions_checked, report.transitions_consistent);
        (factorization, gen_i, gen_j, labels)
    }

    #[test]
    fn lexicon_rejects_ambiguous_and_ungrounded_entries() {
        assert!(GroundingLexicon::from_observations(
            &[(b"A".to_vec(), 1), (b"A".to_vec(), 2)],
            &[(1, b"A".to_vec()), (2, b"B".to_vec())]
        )
        .is_err());
        assert!(GroundingLexicon::from_observations(&[(b"A".to_vec(), 1)], &[]).is_err());
        assert!(GroundingLexicon::from_observations(&[], &[(1, b"A".to_vec())]).is_err());
        assert!(
            GroundingLexicon::from_observations(&[(b"A".to_vec(), 1)], &[(2, b"B".to_vec())])
                .is_err()
        );
        let ok = GroundingLexicon::from_observations(
            &[(b"A".to_vec(), 1), (b"B".to_vec(), 2)],
            &[(1, b"A".to_vec()), (2, b"B".to_vec())],
        )
        .unwrap();
        assert_eq!(ok.label_for(b"A"), Some(1));
        assert_eq!(ok.key_for_label(2), Some(&b"B"[..]));
        assert_eq!(ok.label_for(b"Z"), None);
        let mut forged = serde_json::to_value(&ok).unwrap();
        forged["key_to_label"] = serde_json::json!([[b"A", 1], [b"A", 2], [b"B", 2]]);
        assert!(GroundingLexicon::from_bytes(&serde_json::to_vec(&forged).unwrap()).is_err());
    }

    #[test]
    fn old_three_class_intent_migrates_without_selecting_an_unfitted_fourth_class() {
        let feature = candidate_features(&[11], 0, 1)[0];
        let bytes = serde_json::to_vec(&serde_json::json!({
            "version": 1, "statement": [[feature, [-3, -2, -1]]], "question": []
        }))
        .unwrap();
        let migrated = IntentModel::from_bytes(&bytes, 4096).unwrap();
        assert_eq!(migrated.statement_classes, 3);
        assert_eq!(migrated.statement_intent(&[11], 0, 1), STMT_NONASSERTING);
        assert_eq!(
            IntentModel::from_bytes(&migrated.to_bytes().unwrap(), 4096).unwrap(),
            migrated
        );
        let mut unversioned = serde_json::to_value(&migrated).unwrap();
        unversioned["version"] = serde_json::json!(1);
        assert!(IntentModel::from_bytes(&serde_json::to_vec(&unversioned).unwrap(), 4096).is_err());
    }

    #[test]
    fn older_session_versions_report_explicit_incompatibility_before_field_decoding() {
        let rt = runtime(Memory::new(71, 2), None);
        for version in [1, 2, 3] {
            let bytes = serde_json::to_vec(&serde_json::json!({"version": version})).unwrap();
            assert!(
                matches!(rt.restore(&bytes), Err(ScopedMemoryError::Session(message))
                if message == "unsupported session snapshot version")
            );
        }
    }

    #[test]
    fn computation_identity_validates_dimensions_numbers_modes_and_fold() {
        let (f, _, _, _) = q8_fixture();
        let signed = ComputationBackend::Signed(Box::new(f.clone()));
        let identity = signed.identity_json();
        assert_eq!(
            ComputationBackend::from_identity_json(&identity)
                .unwrap()
                .identity_json(),
            identity
        );
        for (field, bad) in [
            ("action_code", serde_json::json!([])),
            ("action_code", serde_json::json!([256, 0])),
            ("payload_domain", serde_json::json!([4294967296u64])),
            ("outcome_state", serde_json::json!([0, 0])),
        ] {
            let mut changed = identity.clone();
            changed[field] = bad;
            assert!(
                ComputationBackend::from_identity_json(&changed).is_err(),
                "{field}"
            );
        }
        let mut mode = f.clone();
        mode.cyclic = true;
        assert_ne!(
            signed.identity_json(),
            ComputationBackend::Signed(Box::new(mode)).identity_json()
        );
        let folded = ComputationBackend::Folded(
            Box::new(f.clone()),
            Box::new(FoldedGroup::recover(&f).unwrap()),
        );
        let mut fold = folded.identity_json();
        assert_eq!(
            ComputationBackend::from_identity_json(&fold)
                .unwrap()
                .identity_json(),
            fold
        );
        fold["projection"][0] = serde_json::json!(999);
        assert!(ComputationBackend::from_identity_json(&fold).is_err());
        let malformed = ComputationBackend::Signed(Box::new(GroundedFactorization {
            payload_state: Vec::new(),
            ..f
        }));
        assert!(malformed.validate().is_err());
    }

    fn computation_runtime(control: MemoryControl) -> ScopedMemoryRuntime {
        let (f, _, _, labels) = q8_fixture();
        let mut memory = Memory::new(71, 2);
        memory
            .write(
                b"alpha",
                b"Mara",
                0,
                b"L1",
                &[11],
                1,
                Update::Assert,
                false,
                0,
            )
            .unwrap();
        for label in &labels {
            memory
                .write(
                    b"alpha",
                    format!("L{label}").as_bytes(),
                    0,
                    format!("answer-{label}").as_bytes(),
                    &[*label + 40],
                    *label as u64 + 1,
                    Update::Assert,
                    false,
                    0,
                )
                .unwrap();
        }
        let mut surfaces = vec![(b"i".to_vec(), 100), (b"j".to_vec(), 200)];
        let canonical: Vec<_> = labels
            .iter()
            .map(|l| (*l, format!("L{l}").into_bytes()))
            .collect();
        surfaces.extend(canonical.iter().map(|(l, key)| (key.clone(), *l)));
        let lexicon = GroundingLexicon::from_observations(&surfaces, &canonical).unwrap();
        ScopedMemoryRuntime::load_with_computation(
            &ObservedTextModel::uninformed().to_bytes().unwrap(),
            &IntentModel::uninformed().to_bytes().unwrap(),
            Some(&lexicon.to_bytes().unwrap()),
            ComputationBackend::Signed(Box::new(f)),
            memory,
            71,
            control,
            4096,
            Some(4095),
        )
        .unwrap()
    }

    #[test]
    fn computation_restore_rejects_operation_state_source_control_and_query_tampering() {
        let rt = computation_runtime(MemoryControl::Normal);
        let mut session = rt
            .ask_compute(b"alpha", 0, b"Mara", vec![b"i".to_vec(), b"j".to_vec()])
            .unwrap();
        rt.step(&mut session).unwrap();
        rt.step(&mut session).unwrap();
        let mut changed = session.clone();
        changed.ops.reverse();
        assert!(rt.restore(&serde_json::to_vec(&changed).unwrap()).is_err());
        let mut changed = session.clone();
        changed.computation.as_mut().unwrap().state =
            changed.computation.as_ref().unwrap().operand_state;
        assert!(rt.validate(&changed).is_err());
        rt.step(&mut session).unwrap();
        rt.step(&mut session).unwrap(); // publish, before the dependent read
        assert_eq!(session.pending, SessionAction::Read);
        let mut changed = session.clone();
        changed.query_entity = b"L1".to_vec();
        if changed.query_entity == session.query_entity {
            changed.query_entity = b"L2".to_vec();
        }
        assert!(rt.validate(&changed).is_err());
        let mut changed = session.clone();
        changed.computation.as_mut().unwrap().consumed = false;
        assert!(rt.validate(&changed).is_err());
        let mut changed = session.clone();
        changed.pending = SessionAction::Apply;
        assert!(rt.validate(&changed).is_err());
        rt.step(&mut session).unwrap(); // result has replaced the source capture
        for field in ["source_record", "source_commit"] {
            let mut changed = serde_json::to_value(&session).unwrap();
            changed["computation"][field] = serde_json::json!(999);
            assert!(rt.restore(&serde_json::to_vec(&changed).unwrap()).is_err());
        }
        rt.run(&mut session).unwrap();
        assert_eq!(session.terminal, Some(ScopedTerminal::Complete));
    }

    #[test]
    fn raw_request_restore_rederives_observed_operations_and_compute_ingestion_writes_nothing() {
        use super::super::observed_text_session::{fit_observed_text_model, ClauseLabel, Goal};
        let clause = Clause {
            seg: 0,
            tokens: vec![11, 12, 13, 14],
            text: "Mara do i j".into(),
            byte_lengths: vec![4, 3, 2, 2],
        };
        let label = ClauseLabel {
            seg: 0,
            subject: (0, 1),
            marker: (1, 1),
            object: Some((2, 2)),
            role: 0,
            goal: Goal::Office,
            action: super::super::relational_session::RelAction::Emit,
        };
        let (model, _) =
            fit_observed_text_model(std::slice::from_ref(&clause), &[label], false).unwrap();
        let mut intent = IntentModel::uninformed();
        intent.statement = candidate_features(&clause.tokens, 1, 1)
            .into_iter()
            .map(|key| (key, [0, 0, 0, 5]))
            .collect();
        intent.statement.sort_by_key(|(key, _)| *key);
        intent.statement.dedup_by_key(|(key, _)| *key);
        let base = computation_runtime(MemoryControl::Normal);
        let mut rt = ScopedMemoryRuntime::load_with_computation(
            &model.to_bytes().unwrap(),
            &intent.to_bytes().unwrap(),
            Some(&base.lexicon.to_bytes().unwrap()),
            base.backend.clone(),
            base.memory.clone(),
            71,
            MemoryControl::Normal,
            4096,
            Some(4095),
        )
        .unwrap();
        let mut session = rt.ask(&clause, b"alpha").unwrap();
        assert_eq!(session.ops, vec![b"i".to_vec(), b"j".to_vec()]);
        rt.step(&mut session).unwrap();
        let mut changed = session.clone();
        changed.ops.reverse();
        changed.request.ops.reverse();
        assert!(rt.restore(&serde_json::to_vec(&changed).unwrap()).is_err());
        assert_eq!(
            rt.restore(&rt.snapshot(&session).unwrap()).unwrap(),
            session
        );
        let revision = rt.revision();
        assert!(matches!(
            rt.ingest(&clause, b"alpha", 99).unwrap(),
            IngestOutcome::NonAsserting { .. }
        ));
        assert_eq!(rt.revision(), revision);
    }

    #[test]
    fn tabulated_fit_is_independent_of_example_order_and_rejects_truncation() {
        let (f, _, _, labels) = q8_fixture();
        let factorization = &f;
        let examples: Vec<_> = labels
            .iter()
            .flat_map(|label| {
                [100, 200].into_iter().map(move |op| StExample {
                    payload: *label,
                    primitives: vec![op],
                    targets: vec![factorization
                        .outcome_for_state(
                            factorization
                                .apply(factorization.initial_state(*label).unwrap(), op)
                                .unwrap(),
                        )
                        .unwrap()],
                })
            })
            .collect();
        let forward = TabulatedControl::fit(&examples).unwrap();
        let mut reversed = examples.clone();
        reversed.reverse();
        assert_eq!(forward, TabulatedControl::fit(&reversed).unwrap());
        let mut malformed = examples;
        malformed[0].targets.clear();
        assert!(TabulatedControl::fit(&malformed).is_err());
        let backend = ComputationBackend::Tabulated(Box::new(forward));
        assert_eq!(
            ComputationBackend::from_identity_json(&backend.identity_json())
                .unwrap()
                .identity_json(),
            backend.identity_json()
        );
    }

    #[test]
    fn computed_source_can_be_followed_and_two_result_redirects_resume_after_source_eviction() {
        let mut rt = computation_runtime(MemoryControl::Normal);
        let s = rt.backend.initial_state(1).unwrap();
        let state = rt
            .backend
            .apply(rt.backend.apply(s, 100).unwrap(), 200)
            .unwrap();
        let result_key = rt
            .lexicon
            .key_for_label(rt.backend.label_for_state(state).unwrap())
            .unwrap()
            .to_vec();
        rt.memory
            .write(
                b"alpha",
                b"Origin",
                0,
                b"Mara",
                &[20],
                20,
                Update::Assert,
                true,
                0,
            )
            .unwrap();
        rt.memory
            .write(
                b"alpha",
                &result_key,
                0,
                b"Hop1",
                &[21],
                21,
                Update::Correct,
                true,
                0,
            )
            .unwrap();
        rt.memory
            .write(
                b"alpha",
                b"Hop1",
                0,
                b"Hop2",
                &[22],
                22,
                Update::Assert,
                true,
                0,
            )
            .unwrap();
        rt.memory
            .write(
                b"alpha",
                b"Hop2",
                0,
                b"Answer",
                &[23],
                23,
                Update::Assert,
                false,
                0,
            )
            .unwrap();
        let mut session = rt
            .ask_compute(b"alpha", 0, b"Origin", vec![b"i".to_vec(), b"j".to_vec()])
            .unwrap();
        while session.computation.as_ref().is_none_or(|c| c.applied != 1) {
            rt.step(&mut session).unwrap();
        }
        let snap = rt.snapshot(&session).unwrap();
        let source_id = session.computation.as_ref().unwrap().source_record;
        rt.memory
            .write(
                b"alpha",
                b"Mara",
                0,
                b"L2",
                &[31],
                30,
                Update::Correct,
                false,
                0,
            )
            .unwrap();
        rt.memory
            .write(
                b"alpha",
                b"Mara",
                0,
                b"L3",
                &[32],
                31,
                Update::Correct,
                false,
                0,
            )
            .unwrap();
        assert!(rt.memory.record_ref(source_id).unwrap().evicted);
        session = rt.restore(&snap).unwrap();
        while session.terminal.is_none() {
            rt.step(&mut session).unwrap();
            session = rt.restore(&rt.snapshot(&session).unwrap()).unwrap();
        }
        assert_eq!(session.emitted, vec![23, 4095]);
        assert_eq!(session.computation.as_ref().unwrap().source_hop, 1);
        assert_eq!(session.hop, 4);
    }

    #[test]
    fn central_sign_fold_identifies_opposites_that_the_signed_action_separates() {
        let (factorization, _, _, labels) = q8_fixture();
        let folded = FoldedGroup::recover(&factorization).unwrap();
        let signed = ComputationBackend::Signed(Box::new(factorization.clone()));
        let projected =
            ComputationBackend::Folded(Box::new(factorization.clone()), Box::new(folded));
        // 100 and 200 are the declared primitive identifiers; the recovered action domain grounds them.
        let mut separated = 0;
        for label in labels {
            let state = signed.initial_state(label).unwrap();
            let ij = signed
                .apply(signed.apply(state, 100).unwrap(), 200)
                .unwrap();
            let ji = signed
                .apply(signed.apply(state, 200).unwrap(), 100)
                .unwrap();
            if ij != ji {
                separated += 1;
                assert_ne!(
                    signed.label_for_state(ij),
                    signed.label_for_state(ji),
                    "retained sign must distinguish the reversed order"
                );
                let p_ij = projected
                    .apply(projected.apply(state, 100).unwrap(), 200)
                    .unwrap();
                let p_ji = projected
                    .apply(projected.apply(state, 200).unwrap(), 100)
                    .unwrap();
                assert_eq!(p_ij, p_ji, "the central-sign fold must identify the pair");
            }
        }
        assert!(
            separated > 0,
            "the fixture must contain an order-sensitive pair"
        );
    }

    #[test]
    fn computation_consumes_the_grounded_result_and_respects_its_controls() {
        let (factorization, gen_i, gen_j, labels) = q8_fixture();
        let mut memory = Memory::new(31, 8);
        memory
            .write(
                b"alpha",
                b"Mara",
                0,
                b"L1",
                &[11],
                1,
                Update::Assert,
                false,
                0,
            )
            .unwrap();
        // The derived read must find a record for the grounded result label's own entity.
        let s1 = factorization.initial_state(1).unwrap();
        let s_ij = factorization
            .apply(factorization.apply(s1, 100).unwrap(), 200)
            .unwrap();
        let derived = factorization.outcome_for_state(s_ij).unwrap();
        memory
            .write(
                b"alpha",
                format!("L{derived}").as_bytes(),
                0,
                b"TARGET",
                &[42, 43],
                2,
                Update::Assert,
                false,
                0,
            )
            .unwrap();
        let model_bytes = ObservedTextModel::uninformed().to_bytes().unwrap();
        let intent_bytes = IntentModel::uninformed().to_bytes().unwrap();
        // The lexicon grounds the observed operation words "i"/"j" to the artifact's declared
        // primitive identifiers 100/200, and each label word to its opaque domain identifier.
        let mut surface: Vec<(Vec<u8>, u32)> = vec![(b"i".to_vec(), 100), (b"j".to_vec(), 200)];
        surface.extend(labels.iter().map(|l| (format!("L{l}").into_bytes(), *l)));
        let lexicon = GroundingLexicon::from_observations(
            &surface,
            &labels
                .iter()
                .map(|l| (*l, format!("L{l}").into_bytes()))
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let _ = (gen_i, gen_j);
        let ops = vec![b"i".to_vec(), b"j".to_vec()];
        let backend = ComputationBackend::Signed(Box::new(factorization.clone()));
        let runtime = |control: MemoryControl| {
            ScopedMemoryRuntime::load_with_computation(
                &model_bytes,
                &intent_bytes,
                Some(&lexicon.to_bytes().unwrap()),
                backend.clone(),
                memory.clone(),
                31,
                control,
                4096,
                Some(4095),
            )
            .unwrap()
        };
        let run =
            |control: MemoryControl| -> (Vec<u32>, Option<ScopedTerminal>, Option<ComputedState>) {
                let rt = runtime(control.clone());
                let mut session = rt.ask_compute(b"alpha", 0, b"Mara", ops.clone()).unwrap();
                let effects = rt.run(&mut session).unwrap();
                assert!(effects.iter().any(|e| e.action == SessionAction::Apply));
                (
                    session.emitted.clone(),
                    session.terminal,
                    session.computation.clone(),
                )
            };
        let (emitted, terminal, computed) = run(MemoryControl::Normal);
        assert_eq!(terminal, Some(ScopedTerminal::Complete));
        assert_eq!(emitted, vec![42, 43, 4095]);
        let computed = computed.unwrap();
        assert!(computed.consumed && computed.applied == 2);
        assert_eq!(computed.derived_key, format!("L{derived}").into_bytes());

        // Consumption disabled: the state is computed and published, but the read uses the operand's
        // own label, which addresses a different (here absent) entity, so the answer changes.
        let (skip_emitted, terminal_skip, computed_skip) = run(MemoryControl::ConsumeDisabled);
        assert_eq!(terminal_skip, Some(ScopedTerminal::Unresolved));
        assert!(skip_emitted.is_empty());
        let computed_skip = computed_skip.unwrap();
        assert!(!computed_skip.consumed && computed_skip.applied == 2);
        assert_eq!(
            computed_skip.derived_key,
            format!("L{derived}").into_bytes()
        );

        // Apply disabled: no operation is applied, so the operand label addresses the read.
        let (_, terminal_off, computed_off) = run(MemoryControl::ApplyDisabled);
        assert_eq!(terminal_off, Some(ScopedTerminal::Unresolved));
        assert_eq!(computed_off.unwrap().applied, 0);

        // An ungrounded operation is typed rather than silently dropped.
        let rt = runtime(MemoryControl::Normal);
        let mut session = rt
            .ask_compute(b"alpha", 0, b"Mara", vec![b"nope".to_vec()])
            .unwrap();
        rt.run(&mut session).unwrap();
        assert_eq!(session.terminal, Some(ScopedTerminal::UnknownOperation));
        assert!(session.emitted.is_empty());
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
        tampered.pending = SessionAction::Continue;
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
        assert_eq!(session.pending, SessionAction::Read);
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
        assert!(ScopedMemoryRuntime::load_with_computation(
            &ObservedTextModel::uninformed().to_bytes().unwrap(),
            &IntentModel::uninformed().to_bytes().unwrap(),
            Some(&test_lexicon().to_bytes().unwrap()),
            ComputationBackend::Absent,
            memory,
            21,
            MemoryControl::Normal,
            4096,
            Some(eos),
        )
        .is_err());
    }
}
