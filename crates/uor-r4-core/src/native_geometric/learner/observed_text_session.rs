//! Observed-text memory answers through one reusable session boundary.
//!
//! The serving input is **observed tokens plus declared system metadata only**: a tokenized document
//! of statements, an observed question clause, and declared clause boundaries. An entity registry
//! is supplied only to the explicitly named membership control. No gold semantic role,
//! subject/object/answer extent, follow pointer, depth or target is available to the primary path.
//!
//! What is learned from declared development text:
//!
//! * `feature_weights`: signed categorical weights for a candidate marker's endpoints, nearby
//!   tokens, length and clause-edge flags, fitted by a structured perceptron;
//! * `action_per_role`: the action semantics of each role, fitted from declared gold actions starting
//!   from a declared uninformed policy.
//! * `goal_per_role`: the declared goal associated with each supervised role.
//!
//! Candidate admission is fixed: a nonempty subject prefix, a contiguous marker of at most four
//! tokens, and the complete remaining suffix as the optional object. Arbitrary clause layouts and
//! learned subject/object boundaries are not implemented. Local features distinguish some token
//! permutations, but do not encode the order of a four-token marker's two interior tokens.
//!
//! What is exact and typed: entity/answer spans keep full multiword token identity together with
//! their occurrence `(segment, start, len)`, so a repeated word or a shared suffix cannot collapse two
//! different spans, and a captured phrase stays usable after its origin is overwritten.
//!
//! The loaded session boundary [`ObservedTextRuntime::step`] is the single causal implementation used by teacher
//! forcing, generation, interventions, resumption and timing. `RelAction` is reused from the corrected predecessor; complete owned spans retain source provenance.
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use super::realtext_support::sha256_hex;
use super::relational_session::RelAction;

/// Classified failures at the public observation/model/session boundaries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObservedTextError {
    Model(String),
    Observation(String),
    Session(String),
    Supervision(String),
    Serialization(String),
}

impl std::fmt::Display for ObservedTextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (category, detail) = match self {
            Self::Model(detail) => ("model", detail),
            Self::Observation(detail) => ("observation", detail),
            Self::Session(detail) => ("session", detail),
            Self::Supervision(detail) => ("supervision", detail),
            Self::Serialization(detail) => ("serialization", detail),
        };
        write!(f, "observed-text {category}: {detail}")
    }
}

impl std::error::Error for ObservedTextError {}

/// Experiment/CLI adapters may retain their existing textual error boundary.
impl From<ObservedTextError> for String {
    fn from(error: ObservedTextError) -> Self {
        error.to_string()
    }
}

/// Feature kinds for the shared candidate extractor.
pub const F_FIRST: u64 = 1;
pub const F_LAST: u64 = 2;
pub const F_LEFT: u64 = 3;
pub const F_RIGHT: u64 = 4;
pub const F_LEFT2: u64 = 5;
pub const F_RIGHT2: u64 = 6;
pub const F_LEN: u64 = 7;
pub const F_INITIAL: u64 = 8;
pub const F_FINAL: u64 = 9;
/// Declared maximum marker length considered by the candidate enumerator.
pub const OB_MAX_MARKER: usize = 4;

/// **The one candidate observation extractor**, used identically by fitting and serving. It encodes
/// the candidate's ordered local context, never the candidate's role or the expected answer.
pub fn candidate_features(tokens: &[u32], start: usize, len: usize) -> Vec<u64> {
    let n = tokens.len();
    let key = |kind: u64, value: u64| (kind << 40) | value;
    let mut f = Vec::with_capacity(9);
    let Some(end) = start
        .checked_add(len)
        .filter(|end| len > 0 && len <= OB_MAX_MARKER && *end <= n)
    else {
        return f;
    };
    f.push(key(F_FIRST, u64::from(tokens[start])));
    f.push(key(F_LAST, u64::from(tokens[end - 1])));
    f.push(key(F_LEN, len as u64));
    if start > 0 {
        f.push(key(F_LEFT, u64::from(tokens[start - 1])));
    }
    if end < n {
        f.push(key(F_RIGHT, u64::from(tokens[end])));
    }
    if start >= 2 {
        f.push(key(F_LEFT2, u64::from(tokens[start - 2])));
    }
    if let Some(right2) = end.checked_add(1).filter(|right2| *right2 < n) {
        f.push(key(F_RIGHT2, u64::from(tokens[right2])));
    }
    if start == 0 {
        f.push(key(F_INITIAL, 0));
    }
    if end == n {
        f.push(key(F_FINAL, 0));
    }
    f
}

/// Number of learned statement roles: office assertion/redirect and project assertion/redirect.
pub const OB_N_ROLES: usize = 4;
/// Declared execution cap; reaching it is reported as an exhausted/cycle outcome, never a success.
pub const OB_MAX_READS: u8 = 6;
/// Declared maximum clause length considered by the extractor.
pub const OB_MAX_CLAUSE: usize = 24;

/// A declared goal: which fact a request asks for.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum Goal {
    Office,
    Project,
}

impl Goal {
    pub fn index(self) -> usize {
        match self {
            Goal::Office => 0,
            Goal::Project => 1,
        }
    }
    pub fn from_index(i: usize) -> Option<Goal> {
        match i {
            0 => Some(Goal::Office),
            1 => Some(Goal::Project),
            _ => None,
        }
    }
}

/// One exact observed span: its tokens plus its occurrence identity inside the document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedSpan {
    pub seg: u32,
    pub start: u32,
    pub tokens: Vec<u32>,
}

impl ObservedSpan {
    pub fn len(&self) -> usize {
        self.tokens.len()
    }
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
    fn position(&self) -> (u32, u32, usize) {
        (self.seg, self.start, self.tokens.len())
    }
}

/// One statement clause as the *extractor* sees it, with no gold fields attached.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Clause {
    pub seg: u32,
    pub tokens: Vec<u32>,
}

/// Declared offline supervision for one development clause. Never available at serving.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ClauseLabel {
    pub seg: u32,
    pub subject: (u32, usize),
    pub marker: (u32, usize),
    /// The observed object extent, or `None` for a question clause.
    pub object: Option<(u32, usize)>,
    pub role: usize,
    /// Declared offline goal for this role; supervised, never served.
    pub goal: Goal,
    /// Declared offline action semantics for this role; supervised, never served.
    pub action: RelAction,
}

/// The learned observation model.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedTextModel {
    pub version: u8,
    /// Sparse learned candidate-feature weights, sorted by feature key. A candidate is scored from
    /// its **ordered local context** (its own first/last tokens, the tokens immediately before and
    /// after it, its length and its clause-relative position). These observations can distinguish
    /// some permutations and contextual roles; four-token marker interior order is not represented.
    pub feature_weights: Vec<(u64, [i32; OB_N_ROLES])>,
    /// Learned action semantics per role, fitted from declared gold actions.
    pub action_per_role: [u8; OB_N_ROLES],
    /// Learned goal per role.
    pub goal_per_role: [u8; OB_N_ROLES],
    /// Whether that role's action is a redirect (continue) rather than a terminal assertion.
    pub redirect_per_role: [bool; OB_N_ROLES],
}

fn action_tag(a: RelAction) -> u8 {
    match a {
        RelAction::Read => 1,
        RelAction::Continue => 2,
        RelAction::Emit => 3,
        RelAction::Stop => 4,
        RelAction::Unresolved => 5,
        RelAction::Exhausted => 6,
    }
}

fn action_from_tag(t: u8) -> RelAction {
    match t {
        1 => RelAction::Read,
        2 => RelAction::Continue,
        3 => RelAction::Emit,
        4 => RelAction::Stop,
        5 => RelAction::Unresolved,
        _ => RelAction::Exhausted,
    }
}

impl ObservedTextModel {
    /// A declared uninformed start: no candidate weights, every role terminal office.
    pub fn uninformed() -> Self {
        ObservedTextModel {
            version: 3,
            feature_weights: Vec::new(),
            action_per_role: [action_tag(RelAction::Emit); OB_N_ROLES],
            goal_per_role: [0; OB_N_ROLES],
            redirect_per_role: [false; OB_N_ROLES],
        }
    }

    pub fn validate(&self, max_vocab: usize) -> Result<(), ObservedTextError> {
        self.validate_model(max_vocab)
            .map_err(ObservedTextError::Model)
    }

    fn validate_model(&self, max_vocab: usize) -> Result<(), String> {
        if self.version != 3 {
            return Err("unsupported observed-text model version".into());
        }
        if max_vocab == 0 {
            return Err("empty vocabulary".into());
        }
        if self.feature_weights.windows(2).any(|w| w[0].0 >= w[1].0) {
            return Err("observed-text feature weights must have unique sorted keys".into());
        }
        for (key, _) in self.feature_weights.iter() {
            let kind = key >> 40;
            let value = key & ((1u64 << 40) - 1);
            if !(F_FIRST..=F_FINAL).contains(&kind) {
                return Err("unknown observed-text feature kind".into());
            }
            match kind {
                F_LEN if value == 0 || value > OB_MAX_MARKER as u64 => {
                    return Err(
                        "observed-text feature length is outside the candidate bound".into(),
                    );
                }
                F_INITIAL | F_FINAL if value != 0 => {
                    return Err("observed-text edge feature has a nonzero payload".into());
                }
                F_FIRST..=F_RIGHT2 if value > u32::MAX as u64 || value >= max_vocab as u64 => {
                    return Err("observed-text feature token is outside the vocabulary".into());
                }
                _ => {}
            }
        }
        for role in 0..OB_N_ROLES {
            let action = self.action_per_role[role];
            if ![action_tag(RelAction::Emit), action_tag(RelAction::Continue)].contains(&action)
                || Goal::from_index(self.goal_per_role[role] as usize).is_none()
                || self.redirect_per_role[role] != (action == action_tag(RelAction::Continue))
            {
                return Err("invalid observed-text action/goal/redirect table".into());
            }
        }
        Ok(())
    }

    pub fn from_bytes(bytes: &[u8], max_vocab: usize) -> Result<Self, ObservedTextError> {
        let model: Self = serde_json::from_slice(bytes)
            .map_err(|e| ObservedTextError::Serialization(e.to_string()))?;
        model.validate(max_vocab)?;
        Ok(model)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, ObservedTextError> {
        serde_json::to_vec(self).map_err(|e| ObservedTextError::Serialization(e.to_string()))
    }

    /// The one candidate scorer. Fitting and serving both call it through [`candidate_features`].
    pub fn candidate_scores(&self, tokens: &[u32], start: usize, len: usize) -> [i32; OB_N_ROLES] {
        let mut out = [0i32; OB_N_ROLES];
        for key in candidate_features(tokens, start, len) {
            if let Ok(i) = self.feature_weights.binary_search_by_key(&key, |(k, _)| *k) {
                let row = self.feature_weights[i].1;
                for r in 0..OB_N_ROLES {
                    out[r] = out[r].saturating_add(row[r]);
                }
            }
        }
        out
    }

    pub fn role_action(&self, role: usize) -> RelAction {
        self.action_per_role
            .get(role)
            .copied()
            .map(action_from_tag)
            .unwrap_or(RelAction::Unresolved)
    }

    pub fn role_goal(&self, role: usize) -> Option<Goal> {
        self.goal_per_role
            .get(role)
            .and_then(|g| Goal::from_index(*g as usize))
    }

    pub fn role_is_redirect(&self, role: usize) -> bool {
        self.redirect_per_role.get(role).copied().unwrap_or(false)
    }

    /// Locate the marker candidate with the best contextual score. Bounded contiguous candidates
    /// must leave a nonempty observed prefix, which is declared candidate scaffolding.
    pub fn locate_marker(&self, tokens: &[u32]) -> Option<(usize, usize, [i32; OB_N_ROLES])> {
        if tokens.len() > OB_MAX_CLAUSE || tokens.len() < 2 {
            return None;
        }
        let n = tokens.len();
        let mut best: Option<(i32, usize, usize, [i32; OB_N_ROLES])> = None;
        for start in 1..n {
            for len in 1..=(n - start).min(OB_MAX_MARKER) {
                let scores = self.candidate_scores(tokens, start, len);
                let score = *scores.iter().max().unwrap_or(&0);
                let better = match best {
                    None => true,
                    Some((bs, _, blen, _)) => score > bs || (score == bs && len < blen),
                };
                if better {
                    best = Some((score, start, len, scores));
                }
            }
        }
        best.map(|(_, start, len, v)| (start, len, v))
    }
}

/// The learned observation extractor: clause -> (subject, marker, object, role, question goal).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    pub seg: u32,
    pub subject: Vec<u32>,
    pub marker: Vec<u32>,
    pub object: Option<Vec<u32>>,
    pub subject_start: u32,
    pub marker_start: u32,
    pub object_start: Option<u32>,
    pub role: usize,
    /// Present when the clause is a question clause (the marker ends the clause).
    pub question_goal: Option<Goal>,
}

/// Observe one clause through the learned model only.
pub fn observe_clause(
    model: &ObservedTextModel,
    clause: &Clause,
) -> Result<Observation, ObservedTextError> {
    observe_clause_inner(model, clause).map_err(ObservedTextError::Observation)
}

fn observe_clause_inner(model: &ObservedTextModel, clause: &Clause) -> Result<Observation, String> {
    if clause.tokens.is_empty() || clause.tokens.len() > OB_MAX_CLAUSE {
        return Err("clause outside the declared token bound".into());
    }
    let (mstart, mlen, votes) = model
        .locate_marker(&clause.tokens)
        .ok_or("no learned marker candidate in clause")?;
    let role = (0..OB_N_ROLES)
        .max_by_key(|r| (votes[*r], std::cmp::Reverse(*r)))
        .unwrap_or(0);
    let subject = clause.tokens[..mstart].to_vec();
    if subject.is_empty() {
        return Err("clause has no observed subject".into());
    }
    let object = clause.tokens[mstart + mlen..].to_vec();
    let question_goal = if object.is_empty() {
        model.role_goal(role)
    } else {
        None
    };
    Ok(Observation {
        seg: clause.seg,
        subject_start: 0,
        marker_start: mstart as u32,
        object_start: if object.is_empty() {
            None
        } else {
            Some((mstart + mlen) as u32)
        },
        subject,
        marker: clause.tokens[mstart..mstart + mlen].to_vec(),
        object: if object.is_empty() {
            None
        } else {
            Some(object)
        },
        role,
        question_goal,
    })
}

/// Immutable identities prepared from the actual loaded bytes and observed document once.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextBinding {
    pub model_sha256: String,
    pub tokenizer_sha256: String,
    pub world_id: u32,
    pub world_version: u32,
    pub doc_sha256: String,
    pub control_sha256: String,
    pub max_vocab: usize,
    pub eos: Option<u32>,
}

/// Declared experimental controls are immutable runtime inputs and part of snapshot identity.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TextControl {
    #[default]
    Normal,
    ReadsDisabled,
    Membership {
        entities: Vec<Vec<u32>>,
    },
    ReadCap {
        max_reads: u8,
    },
}

/// Exact source provenance for an owned phrase, including document identity and full extent.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextCapture {
    pub world_id: u32,
    pub world_version: u32,
    pub doc_sha256: String,
    pub segment: u32,
    pub start: u32,
    pub len: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextObjective {
    pub goal: Goal,
    pub tokens: Vec<u32>,
}

/// A versioned owned session, containing every control and emission position needed to resume.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextSession {
    pub version: u8,
    pub binding: TextBinding,
    pub goal: Goal,
    pub query: TextObjective,
    pub captured: Option<TextCapture>,
    pub captured_span: Option<(u32, u32, usize)>,
    pub captured_tokens: Vec<u32>,
    pub pending: RelAction,
    pub reads: u8,
    pub emitted: Vec<u32>,
    pub cursor: usize,
    pub visited: Vec<(u32, u32, usize)>,
    pub terminal: Option<RelAction>,
    /// This typed terminator is separate from ordinary vocabulary tokens.
    pub eos: Option<u32>,
}

/// Performed action and its next phase are separate fields; a Read must be recorded as a Read.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StepEffect {
    pub action: RelAction,
    pub next_action: Option<RelAction>,
    pub selected_segment: Option<u32>,
    pub selected_role: Option<usize>,
    pub emitted: Option<u32>,
    pub terminal: Option<RelAction>,
}

impl TextSession {
    fn start(binding: TextBinding, goal: Goal, tokens: Vec<u32>) -> Self {
        Self {
            version: 2,
            eos: binding.eos,
            binding,
            goal,
            query: TextObjective { goal, tokens },
            captured: None,
            captured_span: None,
            captured_tokens: Vec::new(),
            pending: RelAction::Read,
            reads: 0,
            emitted: Vec::new(),
            cursor: 0,
            visited: Vec::new(),
            terminal: None,
        }
    }

    /// Structural checks complement the runtime's verification against the bound observed source.
    pub fn validate(
        &self,
        expected: &TextBinding,
        max_vocab: usize,
    ) -> Result<(), ObservedTextError> {
        self.validate_structure(expected, max_vocab)
            .map_err(ObservedTextError::Session)
    }

    fn validate_structure(&self, expected: &TextBinding, max_vocab: usize) -> Result<(), String> {
        if self.version != 2
            || &self.binding != expected
            || max_vocab != expected.max_vocab
            || self.eos != expected.eos
        {
            return Err("session version or runtime binding mismatch".into());
        }
        if self.goal != self.query.goal
            || self.query.tokens.is_empty()
            || self.query.tokens.len() > OB_MAX_CLAUSE
        {
            return Err("inconsistent observed request objective".into());
        }
        if self
            .query
            .tokens
            .iter()
            .chain(self.captured_tokens.iter())
            .any(|t| *t as usize >= max_vocab)
        {
            return Err("ordinary session token outside vocabulary".into());
        }
        if self.captured_tokens.len() > OB_MAX_CLAUSE
            || self.cursor > self.captured_tokens.len()
            || self.reads > OB_MAX_READS
            || self.visited.len() != self.reads as usize
            || self.visited.iter().copied().collect::<BTreeSet<_>>().len() != self.visited.len()
        {
            return Err("invalid session span, cursor or read history".into());
        }
        match &self.captured {
            Some(capture) => {
                if capture.world_id != expected.world_id
                    || capture.world_version != expected.world_version
                    || capture.doc_sha256 != expected.doc_sha256
                    || capture.len == 0
                    || capture.len != self.captured_tokens.len()
                    || self.captured_span != Some((capture.segment, capture.start, capture.len))
                    || self.visited.last().copied() != self.captured_span
                {
                    return Err("inconsistent captured phrase provenance".into());
                }
            }
            None => {
                if self.captured_span.is_some()
                    || !self.captured_tokens.is_empty()
                    || self.reads != 0
                    || self.cursor != 0
                {
                    return Err("session has content without an owned capture".into());
                }
            }
        }
        let successful_stop = self.terminal == Some(RelAction::Stop);
        let terminators = usize::from(successful_stop && self.eos.is_some());
        if self.emitted.len() != self.cursor + terminators
            || self.emitted.get(..self.cursor) != self.captured_tokens.get(..self.cursor)
            || (terminators == 1 && self.emitted.last().copied() != self.eos)
        {
            return Err(
                "emissions are not the exact captured prefix and declared terminator".into(),
            );
        }
        if self.terminal.is_some() {
            if self.pending != RelAction::Stop
                || !matches!(
                    self.terminal,
                    Some(RelAction::Stop | RelAction::Unresolved | RelAction::Exhausted)
                )
            {
                return Err("invalid terminal session phase".into());
            }
            if successful_stop
                && (self.captured.is_none()
                    || self.cursor != self.captured_tokens.len()
                    || self.cursor == 0)
            {
                return Err("successful Stop requires a complete nonempty owned answer".into());
            }
        } else {
            match self.pending {
                RelAction::Read if self.cursor == 0 && self.emitted.is_empty() => {}
                RelAction::Continue
                    if self.captured.is_some() && self.cursor == 0 && self.emitted.is_empty() => {}
                RelAction::Emit
                    if self.captured.is_some() && self.cursor < self.captured_tokens.len() => {}
                RelAction::Stop
                    if self.captured.is_some()
                        && self.cursor > 0
                        && self.cursor == self.captured_tokens.len() => {}
                _ => return Err("invalid active session phase".into()),
            }
        }
        Ok(())
    }

    fn terminate(&mut self, reason: RelAction) {
        self.terminal = Some(reason);
        self.pending = RelAction::Stop;
    }
}

/// A loaded, source-bound runtime. The primary receives no entity registry or gold goal.
/// Documents are immutable here: an explicit new document/version creates a new runtime identity.
pub struct ObservedTextRuntime {
    model: ObservedTextModel,
    clauses: Vec<Clause>,
    observations: Vec<Observation>,
    binding: TextBinding,
    control: TextControl,
}

impl ObservedTextRuntime {
    pub fn load(
        model_bytes: &[u8],
        tokenizer_sha256: &str,
        clauses: Vec<Clause>,
        world_id: u32,
        world_version: u32,
        max_vocab: usize,
        eos: Option<u32>,
    ) -> Result<Self, ObservedTextError> {
        let model = ObservedTextModel::from_bytes(model_bytes, max_vocab)?;
        if tokenizer_sha256.len() != 64 || !tokenizer_sha256.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(ObservedTextError::Session(
                "tokenizer identity must be an actual SHA-256 digest".into(),
            ));
        }
        let mut seen = BTreeSet::new();
        for clause in &clauses {
            if !seen.insert(clause.seg)
                || clause.tokens.is_empty()
                || clause.tokens.len() > OB_MAX_CLAUSE
                || clause.tokens.iter().any(|t| *t as usize >= max_vocab)
            {
                return Err(ObservedTextError::Observation(
                    "document has duplicate segments or invalid observed tokens".into(),
                ));
            }
        }
        let observations = clauses
            .iter()
            .map(|clause| observe_clause(&model, clause))
            .collect::<Result<Vec<_>, _>>()?;
        let binding = TextBinding {
            model_sha256: sha256_hex(model_bytes),
            tokenizer_sha256: tokenizer_sha256.to_owned(),
            world_id,
            world_version,
            doc_sha256: sha256_hex(
                &serde_json::to_vec(&clauses)
                    .map_err(|e| ObservedTextError::Serialization(e.to_string()))?,
            ),
            control_sha256: sha256_hex(
                &serde_json::to_vec(&TextControl::Normal)
                    .map_err(|e| ObservedTextError::Serialization(e.to_string()))?,
            ),
            max_vocab,
            eos,
        };
        Ok(Self {
            model,
            clauses,
            observations,
            binding,
            control: TextControl::Normal,
        })
    }

    pub fn with_control(mut self, control: TextControl) -> Result<Self, ObservedTextError> {
        match &control {
            TextControl::Membership { entities }
                if entities.iter().any(|e| {
                    e.is_empty()
                        || e.len() > OB_MAX_CLAUSE
                        || e.iter().any(|t| *t as usize >= self.binding.max_vocab)
                }) =>
            {
                return Err(ObservedTextError::Observation(
                    "invalid membership-control input".into(),
                ))
            }
            TextControl::ReadCap { max_reads } if *max_reads == 0 || *max_reads > OB_MAX_READS => {
                return Err(ObservedTextError::Session(
                    "invalid read-cap control".into(),
                ))
            }
            _ => {}
        }
        self.binding.control_sha256 = sha256_hex(
            &serde_json::to_vec(&control)
                .map_err(|e| ObservedTextError::Serialization(e.to_string()))?,
        );
        self.control = control;
        Ok(self)
    }

    pub fn model(&self) -> &ObservedTextModel {
        &self.model
    }
    pub fn binding(&self) -> &TextBinding {
        &self.binding
    }
    pub fn observations(&self) -> &[Observation] {
        &self.observations
    }
    pub fn clauses(&self) -> &[Clause] {
        &self.clauses
    }

    /// Parse only the actually observed question; desired goal/answer remains outside this API.
    pub fn start(&self, question: &Clause) -> Result<TextSession, ObservedTextError> {
        if question
            .tokens
            .iter()
            .any(|t| *t as usize >= self.binding.max_vocab)
        {
            return Err(ObservedTextError::Observation(
                "question contains an out-of-vocabulary token".into(),
            ));
        }
        let observed = observe_clause(&self.model, question)?;
        let goal = observed.question_goal.ok_or_else(|| {
            ObservedTextError::Observation("observed clause is not a recognized question".into())
        })?;
        let frame = TextSession::start(self.binding.clone(), goal, observed.subject);
        self.validate(&frame)?;
        Ok(frame)
    }

    pub fn origin_is_live(&self, session: &TextSession) -> bool {
        let Some(c) = &session.captured else {
            return false;
        };
        if c.world_id != self.binding.world_id
            || c.world_version != self.binding.world_version
            || c.doc_sha256 != self.binding.doc_sha256
        {
            return false;
        }
        let Some(clause) = self.clauses.iter().find(|clause| clause.seg == c.segment) else {
            return false;
        };
        (c.start as usize)
            .checked_add(c.len)
            .and_then(|end| clause.tokens.get(c.start as usize..end))
            == Some(session.captured_tokens.as_slice())
    }

    pub fn validate(&self, session: &TextSession) -> Result<(), ObservedTextError> {
        self.validate_session(session)
            .map_err(ObservedTextError::Session)
    }

    fn validate_session(&self, session: &TextSession) -> Result<(), String> {
        session.validate(&self.binding, self.binding.max_vocab)?;
        // Every retained source reference denotes an actually parsed object for the active goal.
        // A substring that happens to match raw document bytes is not sufficient provenance.
        let mut history = Vec::with_capacity(session.visited.len());
        for &(segment, start, len) in &session.visited {
            let observed = self
                .observations
                .iter()
                .find(|observed| {
                    observed.seg == segment
                        && observed.object_start == Some(start)
                        && observed
                            .object
                            .as_ref()
                            .is_some_and(|tokens| tokens.len() == len)
                        && self.model.role_goal(observed.role) == Some(session.goal)
                })
                .ok_or("visited reference is not an observed object for the active goal")?;
            history.push(observed);
        }
        if session.captured.is_some() && !self.origin_is_live(session) {
            return Err("captured phrase disagrees with its bound source occurrence".into());
        }
        if matches!(self.control, TextControl::ReadsDisabled) && !history.is_empty() {
            return Err("read-disabled session contains a read history".into());
        }
        if let TextControl::ReadCap { max_reads } = self.control {
            if session.reads > max_reads {
                return Err("session history exceeds its bound read control".into());
            }
        }
        // The original question is not retained: these checks establish internal chain and phase
        // consistency, not authentication of an external request or proof of ranking optimality.
        for pair in history.windows(2) {
            if self.action_after_read(pair[0])? != RelAction::Continue
                || pair[0].object.as_ref() != Some(&pair[1].subject)
            {
                return Err("visited objects do not form a dependent continuation chain".into());
            }
        }
        if let Some(last) = history.last() {
            let action = self.action_after_read(last)?;
            let followed = session.pending == RelAction::Read
                || matches!(
                    session.terminal,
                    Some(RelAction::Unresolved | RelAction::Exhausted)
                );
            if followed {
                if action != RelAction::Continue
                    || last.object.as_ref() != Some(&session.query.tokens)
                {
                    return Err("followed session does not preserve its dependent query".into());
                }
            } else {
                let expected_action = if session.pending == RelAction::Continue {
                    RelAction::Continue
                } else {
                    RelAction::Emit
                };
                if action != expected_action || last.subject != session.query.tokens {
                    return Err(
                        "session phase disagrees with its captured source role/control".into(),
                    );
                }
            }
        }
        Ok(())
    }

    /// The fixed control and the learned role table determine the next phase in both step and
    /// restoration validation. The membership registry is available only to that named control.
    fn action_after_read(&self, observed: &Observation) -> Result<RelAction, String> {
        let object = observed
            .object
            .as_ref()
            .ok_or("selected fact lacks an object")?;
        Ok(match &self.control {
            TextControl::Membership { entities } => {
                if entities.contains(object) {
                    RelAction::Continue
                } else {
                    RelAction::Emit
                }
            }
            _ => self.model.role_action(observed.role),
        })
    }

    pub fn snapshot(&self, session: &TextSession) -> Result<Vec<u8>, ObservedTextError> {
        self.validate(session)?;
        serde_json::to_vec(session).map_err(|e| ObservedTextError::Serialization(e.to_string()))
    }

    pub fn restore(&self, bytes: &[u8]) -> Result<TextSession, ObservedTextError> {
        let session: TextSession = serde_json::from_slice(bytes)
            .map_err(|e| ObservedTextError::Serialization(e.to_string()))?;
        self.validate(&session)?;
        Ok(session)
    }

    pub fn step(&self, session: &mut TextSession) -> Result<StepEffect, ObservedTextError> {
        self.step_inner(session).map_err(ObservedTextError::Session)
    }

    fn step_inner(&self, session: &mut TextSession) -> Result<StepEffect, String> {
        self.validate(session)?;
        if session.terminal.is_some() {
            return Err("session already terminal".into());
        }
        let performed = session.pending;
        let mut effect = StepEffect {
            action: performed,
            next_action: None,
            selected_segment: None,
            selected_role: None,
            emitted: None,
            terminal: None,
        };
        match performed {
            RelAction::Read => {
                let cap = match self.control {
                    TextControl::ReadCap { max_reads } => max_reads,
                    _ => OB_MAX_READS,
                };
                if matches!(self.control, TextControl::ReadsDisabled) {
                    effect.action = RelAction::Unresolved;
                    session.terminate(RelAction::Unresolved);
                } else if session.reads >= cap {
                    effect.action = RelAction::Exhausted;
                    session.terminate(RelAction::Exhausted);
                } else {
                    let mut best: Option<(i32, usize)> = None;
                    for (i, observed) in self.observations.iter().enumerate() {
                        if observed.subject != session.query.tokens
                            || self.model.role_goal(observed.role) != Some(session.goal)
                            || observed.object.is_none()
                        {
                            continue;
                        }
                        // Rank admitted statements by the same contextual candidate score that
                        // selected their marker span, so observation and ranking agree.
                        let clause = self
                            .clauses
                            .iter()
                            .find(|c| c.seg == observed.seg)
                            .ok_or("observed statement lost its clause")?;
                        let scores = self.model.candidate_scores(
                            &clause.tokens,
                            observed.marker_start as usize,
                            observed.marker.len(),
                        );
                        let score = scores[observed.role.min(OB_N_ROLES - 1)];
                        if best.is_none_or(|(old, _)| score > old) {
                            best = Some((score, i));
                        }
                    }
                    if let Some((_, idx)) = best {
                        let observed = &self.observations[idx];
                        let object = observed
                            .object
                            .as_ref()
                            .ok_or("selected fact lacks an object")?;
                        let start = observed
                            .object_start
                            .ok_or("selected fact lacks an extent")?;
                        let span = (observed.seg, start, object.len());
                        effect.selected_segment = Some(observed.seg);
                        effect.selected_role = Some(observed.role);
                        if session.visited.contains(&span) {
                            session.terminate(RelAction::Exhausted);
                        } else {
                            session.visited.push(span);
                            session.reads += 1;
                            session.captured = Some(TextCapture {
                                world_id: self.binding.world_id,
                                world_version: self.binding.world_version,
                                doc_sha256: self.binding.doc_sha256.clone(),
                                segment: observed.seg,
                                start,
                                len: object.len(),
                            });
                            session.captured_span = Some(span);
                            session.captured_tokens = object.clone();
                            session.pending = self.action_after_read(observed)?;
                        }
                    } else {
                        session.terminate(RelAction::Unresolved);
                    }
                }
            }
            RelAction::Continue => {
                session.query = TextObjective {
                    goal: session.goal,
                    tokens: session.captured_tokens.clone(),
                };
                session.pending = RelAction::Read;
            }
            RelAction::Emit => {
                let token = *session
                    .captured_tokens
                    .get(session.cursor)
                    .ok_or("emission cursor past captured phrase")?;
                session.emitted.push(token);
                session.cursor += 1;
                effect.emitted = Some(token);
                if session.cursor == session.captured_tokens.len() {
                    session.pending = RelAction::Stop;
                }
            }
            RelAction::Stop => {
                // Stop is legal only after the full phrase; the typed EOS is appended exactly once.
                if let Some(eos) = session.eos {
                    session.emitted.push(eos);
                    effect.emitted = Some(eos);
                }
                session.terminate(RelAction::Stop);
            }
            _ => return Err("unsupported active observed-text phase".into()),
        }
        self.validate(session)?;
        effect.next_action = if session.terminal.is_none() {
            Some(session.pending)
        } else {
            None
        };
        effect.terminal = session.terminal;
        Ok(effect)
    }

    /// At most six reads, five continuation steps, 24 answer tokens and one Stop are possible.
    pub fn run(&self, session: &mut TextSession) -> Result<Vec<StepEffect>, ObservedTextError> {
        self.run_inner(session).map_err(ObservedTextError::Session)
    }

    fn run_inner(&self, session: &mut TextSession) -> Result<Vec<StepEffect>, String> {
        self.validate(session)?;
        let mut effects = Vec::new();
        for _ in 0..(2 * OB_MAX_READS as usize + OB_MAX_CLAUSE + 2) {
            if session.terminal.is_some() {
                return Ok(effects);
            }
            effects.push(self.step(session)?);
        }
        if session.terminal.is_none() {
            session.terminate(RelAction::Exhausted);
            self.validate(session)?;
            effects.push(StepEffect {
                action: RelAction::Exhausted,
                next_action: None,
                selected_segment: None,
                selected_role: None,
                emitted: None,
                terminal: session.terminal,
            });
        }
        Ok(effects)
    }
}

/// Receipt for the observation-model fit.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ObFitReport {
    pub clauses: usize,
    pub feature_weights: usize,
    pub candidate_updates: usize,
    pub epochs_run: usize,
    /// Same-unit whole-clause (marker, role) accuracy with the declared uninformed start and after
    /// fitting, on exactly the same development clauses.
    pub role_initial_correct: usize,
    pub role_final_correct: usize,
    /// Role-action table accounting on the same gold statement-action units as before.
    pub policy_initial_correct: usize,
    pub policy_final_correct: usize,
    pub policy_examples: usize,
    pub roles_observed: Vec<usize>,
}

/// Whole-clause (marker extent, role) accuracy of a fitted model on declared clauses.
fn role_accuracy(model: &ObservedTextModel, clauses: &[Clause], labels: &[ClauseLabel]) -> usize {
    let mut hits = 0usize;
    for label in labels {
        let Some(clause) = clauses.iter().find(|c| c.seg == label.seg) else {
            continue;
        };
        let Some((start, len, votes)) = model.locate_marker(&clause.tokens) else {
            continue;
        };
        let role = (0..OB_N_ROLES)
            .max_by_key(|r| (votes[*r], std::cmp::Reverse(*r)))
            .unwrap_or(0);
        if start == label.marker.0 as usize && len == label.marker.1 && role == label.role {
            hits += 1;
        }
    }
    hits
}

/// Fit the observation model from declared development clauses and their gold labels.
///
/// Supervision: the declared marker extent and role are the gold candidate. Candidate features come
/// from the **same** extractor serving uses, so no gold field can become an input feature. The learner
/// is a bounded candidate perceptron; the declared uninformed start has no weights at all.
pub fn fit_observed_text_model(
    clauses: &[Clause],
    labels: &[ClauseLabel],
) -> Result<(ObservedTextModel, ObFitReport), ObservedTextError> {
    fit_observed_text_model_inner(clauses, labels).map_err(ObservedTextError::Supervision)
}

fn fit_observed_text_model_inner(
    clauses: &[Clause],
    labels: &[ClauseLabel],
) -> Result<(ObservedTextModel, ObFitReport), String> {
    let mut segments = BTreeSet::new();
    for clause in clauses {
        if !segments.insert(clause.seg)
            || clause.tokens.is_empty()
            || clause.tokens.len() > OB_MAX_CLAUSE
        {
            return Err("duplicate segment or invalid clause bound in fitting".into());
        }
    }
    let mut labeled = BTreeSet::new();
    let mut semantics: BTreeMap<usize, RelAction> = BTreeMap::new();
    let mut role_goals: BTreeMap<usize, Goal> = BTreeMap::new();
    for label in labels {
        if !labeled.insert(label.seg) || label.role >= OB_N_ROLES {
            return Err("duplicate supervision or out-of-range role".into());
        }
        let clause = clauses
            .iter()
            .find(|c| c.seg == label.seg)
            .ok_or("label has no observed clause")?;
        let within = |span: (u32, usize)| {
            span.1 > 0
                && (span.0 as usize)
                    .checked_add(span.1)
                    .is_some_and(|end| end <= clause.tokens.len())
        };
        if !within(label.subject) || !within(label.marker) || label.marker.0 == 0 {
            return Err("declared span is outside its clause".into());
        }
        let marker_start = label.marker.0 as usize;
        let marker_end = marker_start + label.marker.1; // checked by within above
        if label.subject.0 != 0 || label.subject.1 != marker_start || label.marker.1 > OB_MAX_MARKER
        {
            return Err(
                "supervision is outside the subject-prefix/marker candidate grammar".into(),
            );
        }
        if let Some(object) = label.object {
            if !within(object)
                || object.0 as usize != marker_end
                || object.1 != clause.tokens.len() - marker_end
            {
                return Err("declared object must be the complete suffix after its marker".into());
            }
        } else if marker_end != clause.tokens.len() {
            return Err("question marker must end its observed clause".into());
        }
        if !matches!(label.action, RelAction::Emit | RelAction::Continue) {
            return Err("supervised role action must be Emit or Continue".into());
        }
        if role_goals
            .insert(label.role, label.goal)
            .is_some_and(|prior| prior != label.goal)
        {
            return Err("one observed role has conflicting supervised goals".into());
        }
        if label.object.is_some() {
            if semantics
                .insert(label.role, label.action)
                .is_some_and(|prior| prior != label.action)
            {
                return Err("one observed role has conflicting supervised actions".into());
            }
        }
    }
    let mut model = ObservedTextModel::uninformed();
    let role_initial_correct = role_accuracy(&model, clauses, labels);
    let mut weights: BTreeMap<u64, [i32; OB_N_ROLES]> = BTreeMap::new();
    let predictions = |tokens: &[u32], weights: &BTreeMap<u64, [i32; OB_N_ROLES]>| {
        let n = tokens.len();
        let mut best: Option<(i32, usize, usize, usize)> = None;
        for start in 1..n {
            for len in 1..=(n - start).min(OB_MAX_MARKER) {
                let mut sc = [0i32; OB_N_ROLES];
                for key in candidate_features(tokens, start, len) {
                    let row = weights.get(&key).copied().unwrap_or([0; OB_N_ROLES]);
                    for r in 0..OB_N_ROLES {
                        sc[r] = sc[r].saturating_add(row[r]);
                    }
                }
                let role = (0..OB_N_ROLES)
                    .max_by_key(|r| (sc[*r], std::cmp::Reverse(*r)))
                    .unwrap_or(0);
                let score = sc[role];
                let better = match best {
                    None => true,
                    Some((bs, _, blen, _)) => score > bs || (score == bs && len < blen),
                };
                if better {
                    best = Some((score, start, len, role));
                }
            }
        }
        best.map(|(_, s, l, r)| (s, l, r))
    };
    let epochs = 8usize;
    let mut updates = 0usize;
    let mut epochs_run = 0usize;
    for _epoch in 0..epochs {
        epochs_run += 1;
        let mut changed = false;
        for label in labels {
            let clause = clauses
                .iter()
                .find(|c| c.seg == label.seg)
                .ok_or("label lost its clause")?;
            let gold = (label.marker.0 as usize, label.marker.1, label.role);
            // The prediction must be taken *before* any update: recomputing it after the positive
            // update would subtract the gold candidate's own features and cancel the step.
            let pred = predictions(&clause.tokens, &weights);
            if pred == Some(gold) {
                continue;
            }
            for key in candidate_features(&clause.tokens, gold.0, gold.1) {
                let row = weights.entry(key).or_insert([0; OB_N_ROLES]);
                row[gold.2] = row[gold.2].saturating_add(1);
            }
            if let Some((ps, pl, pr)) = pred {
                for key in candidate_features(&clause.tokens, ps, pl) {
                    let row = weights.entry(key).or_insert([0; OB_N_ROLES]);
                    row[pr] = row[pr].saturating_sub(1);
                }
            }
            updates += 1;
            changed = true;
        }
        if !changed {
            break;
        }
    }
    model.feature_weights = weights.into_iter().collect();
    let final_correct = role_accuracy(&model, clauses, labels);
    // Role semantics: goal and action per role come from the declared labels, never from serving.
    for (&role, &goal) in &role_goals {
        model.goal_per_role[role] = goal.index() as u8;
    }
    for (&role, &action) in &semantics {
        model.action_per_role[role] = action_tag(action);
        model.redirect_per_role[role] = action == RelAction::Continue;
    }
    model.validate(usize::MAX).map_err(|e| e.to_string())?;
    // Same-unit accounting for the role-action table (statement units only).
    let mut policy_initial_correct = 0usize;
    let mut policy_final_correct = 0usize;
    let mut examples = 0usize;
    for label in labels {
        if label.object.is_none() {
            continue;
        }
        examples += 1;
        let want = label.action;
        if want == RelAction::Emit {
            policy_initial_correct += 1;
        }
        if model.role_action(label.role) == want {
            policy_final_correct += 1;
        }
    }
    let report = ObFitReport {
        clauses: clauses.len(),
        feature_weights: model.feature_weights.len(),
        candidate_updates: updates,
        epochs_run,
        role_initial_correct,
        role_final_correct: final_correct,
        policy_initial_correct,
        policy_final_correct,
        policy_examples: examples,
        roles_observed: role_goals.keys().copied().collect(),
    };
    Ok((model, report))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Declared development text for the test: two goals, an assertion and a redirect each.
    fn dev() -> (Vec<Clause>, Vec<ClauseLabel>) {
        // tokens: subject marker object
        let clauses = vec![
            Clause {
                seg: 0,
                tokens: vec![10, 20, 21, 30],
            }, // Mara works-in Cedar
            Clause {
                seg: 1,
                tokens: vec![10, 22, 23, 11],
            }, // Mara office-follows Ivo
            Clause {
                seg: 2,
                tokens: vec![10, 24, 25, 31],
            }, // Mara is-on-project Atlas
            Clause {
                seg: 3,
                tokens: vec![10, 26, 27, 12],
            }, // Mara project-follows Nila
            Clause {
                seg: 4,
                tokens: vec![10, 20, 21],
            }, // Mara works-in ?
            Clause {
                seg: 5,
                tokens: vec![10, 24, 25],
            }, // Mara is-on-project ?
            Clause {
                seg: 6,
                tokens: vec![11, 20, 21, 32],
            }, // Ivo works-in Fen
        ];
        let labels = vec![
            ClauseLabel {
                seg: 0,
                subject: (0, 1),
                marker: (1, 2),
                object: Some((3, 1)),
                role: 0,
                goal: Goal::Office,
                action: RelAction::Emit,
            },
            ClauseLabel {
                seg: 1,
                subject: (0, 1),
                marker: (1, 2),
                object: Some((3, 1)),
                role: 1,
                goal: Goal::Office,
                action: RelAction::Continue,
            },
            ClauseLabel {
                seg: 2,
                subject: (0, 1),
                marker: (1, 2),
                object: Some((3, 1)),
                role: 2,
                goal: Goal::Project,
                action: RelAction::Emit,
            },
            ClauseLabel {
                seg: 3,
                subject: (0, 1),
                marker: (1, 2),
                object: Some((3, 1)),
                role: 3,
                goal: Goal::Project,
                action: RelAction::Continue,
            },
            ClauseLabel {
                seg: 4,
                subject: (0, 1),
                marker: (1, 2),
                object: None,
                role: 0,
                goal: Goal::Office,
                action: RelAction::Emit,
            },
            ClauseLabel {
                seg: 5,
                subject: (0, 1),
                marker: (1, 2),
                object: None,
                role: 2,
                goal: Goal::Project,
                action: RelAction::Emit,
            },
            ClauseLabel {
                seg: 6,
                subject: (0, 1),
                marker: (1, 2),
                object: Some((3, 1)),
                role: 0,
                goal: Goal::Office,
                action: RelAction::Emit,
            },
        ];
        (clauses, labels)
    }

    #[test]
    fn the_extractor_locates_roles_and_goals_without_gold_fields() {
        let (clauses, labels) = dev();
        let (model, report) = fit_observed_text_model(&clauses, &labels).unwrap();
        assert_eq!(report.policy_final_correct, report.policy_examples);
        assert!(report.policy_final_correct > report.policy_initial_correct);
        for (clause, label) in clauses.iter().zip(labels.iter()) {
            let obs = observe_clause(&model, clause).expect("clause must be observable");
            assert_eq!(obs.role, label.role, "seg {}", clause.seg);
            let (ss, sl) = label.subject;
            let expected_subject = clause.tokens[ss as usize..(ss as usize + sl)].to_vec();
            assert_eq!(obs.subject, expected_subject, "seg {}", clause.seg);
            if label.object.is_some() {
                assert!(obs.object.is_some());
                assert!(obs.question_goal.is_none());
            } else {
                assert!(obs.object.is_none());
                assert_eq!(obs.question_goal, Some(label.goal));
            }
        }
    }

    #[test]
    fn fit_reports_extractor_and_action_baselines_on_their_own_units() {
        let (clauses, labels) = dev();
        let (_, report) = fit_observed_text_model(&clauses, &labels).unwrap();
        // With zero weights, the shortest/first marker candidate is one token; all gold markers
        // have two tokens. This baseline is distinct from the three initially-correct Emit actions.
        assert_eq!(report.role_initial_correct, 0);
        assert_eq!(report.role_final_correct, labels.len());
        assert_eq!(report.policy_initial_correct, 3);
        assert_eq!(report.policy_examples, 5);
        assert_eq!(report.policy_final_correct, 5);
    }

    #[test]
    fn full_width_candidate_scores_preserve_the_winning_role_and_ties() {
        for weights in [[40_000, 50_000, 0, 0], [-40_000, -35_000, -50_000, -60_000]] {
            let mut model = ObservedTextModel::uninformed();
            model.feature_weights = vec![((F_FIRST << 40) | 20, weights)];
            let loaded = ObservedTextModel::from_bytes(&model.to_bytes().unwrap(), 4096).unwrap();
            let clause = Clause {
                seg: 1,
                tokens: vec![10, 20],
            };
            assert_eq!(loaded.candidate_scores(&clause.tokens, 1, 1), weights);
            assert_eq!(loaded.locate_marker(&clause.tokens), Some((1, 1, weights)));
            assert_eq!(observe_clause(&loaded, &clause).unwrap().role, 1);
        }
        let mut model = ObservedTextModel::uninformed();
        model.feature_weights = vec![((F_FIRST << 40) | 20, [50_000, 50_000, 0, 0])];
        let clause = Clause {
            seg: 1,
            tokens: vec![10, 20, 20],
        };
        assert_eq!(
            model.locate_marker(&clause.tokens).map(|(s, l, _)| (s, l)),
            Some((1, 1))
        );
        assert_eq!(observe_clause(&model, &clause).unwrap().role, 0);
    }

    #[test]
    fn candidate_and_feature_bounds_reject_unrepresentable_inputs() {
        let tokens = [10, 20, 30, 40, 50, 60];
        for (start, len) in [(usize::MAX, 1), (1, usize::MAX), (0, 0), (6, 1), (0, 5)] {
            assert!(candidate_features(&tokens, start, len).is_empty());
        }
        for key in [
            F_LEN << 40,
            (F_LEN << 40) | (OB_MAX_MARKER as u64 + 1),
            (F_INITIAL << 40) | 1,
            (F_FINAL << 40) | 1,
            (F_FIRST << 40) | 4096,
            (F_FIRST << 40) | (u32::MAX as u64 + 1),
            (F_FINAL + 1) << 40,
        ] {
            let mut model = ObservedTextModel::uninformed();
            model.feature_weights = vec![(key, [0; OB_N_ROLES])];
            assert!(matches!(
                ObservedTextModel::from_bytes(&model.to_bytes().unwrap(), 4096),
                Err(ObservedTextError::Model(_))
            ));
        }
    }

    #[test]
    fn supervision_rejects_conflicts_and_spans_outside_the_candidate_grammar() {
        let (clauses, labels) = dev();
        for mutation in 0..6 {
            let mut bad = labels.clone();
            match mutation {
                0 => bad[0].subject = (1, 1),
                1 => bad[0].object = Some((2, 1)),
                2 => bad[0].object = None,
                3 => bad[6].goal = Goal::Project,
                4 => bad[6].action = RelAction::Continue,
                _ => bad[0].action = RelAction::Read,
            }
            assert!(
                matches!(
                    fit_observed_text_model(&clauses, &bad),
                    Err(ObservedTextError::Supervision(_))
                ),
                "mutation {mutation}"
            );
        }
        // A valid in-clause span can still be outside the candidate admission bound.
        let clause = Clause {
            seg: 1,
            tokens: vec![10, 20, 21, 22, 23, 24, 30],
        };
        let label = ClauseLabel {
            seg: 1,
            subject: (0, 1),
            marker: (1, 5),
            object: Some((6, 1)),
            role: 0,
            goal: Goal::Office,
            action: RelAction::Emit,
        };
        assert!(matches!(
            fit_observed_text_model(&[clause], &[label]),
            Err(ObservedTextError::Supervision(_))
        ));
    }

    fn runtime(clauses: Vec<Clause>, eos: Option<u32>) -> ObservedTextRuntime {
        let (development, labels) = dev();
        let (model, _) = fit_observed_text_model(&development, &labels).unwrap();
        ObservedTextRuntime::load(
            &model.to_bytes().unwrap(),
            &"ab".repeat(32),
            clauses,
            1,
            7,
            4096,
            eos,
        )
        .unwrap()
    }

    fn question(subject: Vec<u32>, marker: &[u32]) -> Clause {
        let mut tokens = subject;
        tokens.extend_from_slice(marker);
        Clause { seg: 999, tokens }
    }

    #[test]
    fn loaded_runtime_preserves_goal_and_exact_multiword_source() {
        let rt = runtime(
            vec![
                Clause {
                    seg: 10,
                    tokens: vec![10, 11, 22, 23, 12, 11],
                },
                Clause {
                    seg: 11,
                    tokens: vec![12, 11, 20, 21, 32, 33],
                },
                // Shared suffix is not an exact entity match.
                Clause {
                    seg: 12,
                    tokens: vec![13, 11, 20, 21, 34],
                },
            ],
            Some(u32::MAX - 1),
        );
        let mut session = rt.start(&question(vec![10, 11], &[20, 21])).unwrap();
        let effects = rt.run(&mut session).unwrap();
        assert_eq!(session.terminal, Some(RelAction::Stop));
        assert_eq!(session.goal, Goal::Office);
        assert_eq!(session.query.goal, Goal::Office);
        assert_eq!(session.emitted, vec![32, 33, u32::MAX - 1]);
        assert_eq!(session.reads, 2);
        assert_eq!(session.captured_span, Some((11, 4, 2)));
        assert_eq!(
            effects
                .iter()
                .filter(|e| e.action == RelAction::Read)
                .count(),
            2
        );
        assert!(rt.origin_is_live(&session));
        rt.validate(&session).unwrap();
    }

    #[test]
    fn missing_fact_cycle_and_read_controls_have_explicit_terminal_reasons() {
        let cycle = runtime(
            vec![Clause {
                seg: 9,
                tokens: vec![10, 22, 23, 10],
            }],
            Some(u32::MAX - 1),
        );
        let q = question(vec![10], &[20, 21]);
        let mut s = cycle.start(&q).unwrap();
        cycle.run(&mut s).unwrap();
        assert_eq!(s.terminal, Some(RelAction::Exhausted));
        assert!(s.emitted.is_empty());
        let mut missing = cycle.start(&question(vec![99], &[20, 21])).unwrap();
        cycle.run(&mut missing).unwrap();
        assert_eq!(missing.terminal, Some(RelAction::Unresolved));
        assert!(missing.emitted.is_empty());
        let disabled = runtime(
            vec![Clause {
                seg: 9,
                tokens: vec![10, 20, 21, 30],
            }],
            Some(u32::MAX - 1),
        )
        .with_control(TextControl::ReadsDisabled)
        .unwrap();
        let mut no_read = disabled.start(&q).unwrap();
        disabled.run(&mut no_read).unwrap();
        assert_eq!(no_read.terminal, Some(RelAction::Unresolved));
        assert!(no_read.emitted.is_empty());
    }

    #[test]
    fn every_emission_phase_and_final_eos_resume_independently() {
        let rt = runtime(
            vec![Clause {
                seg: 10,
                tokens: vec![10, 20, 21, 30, 31, 32],
            }],
            Some(u32::MAX - 1),
        );
        let mut original = rt.start(&question(vec![10], &[20, 21])).unwrap();
        loop {
            let bytes = rt.snapshot(&original).unwrap();
            let mut restored = rt.restore(&bytes).unwrap();
            let mut continued = original.clone();
            assert_eq!(
                rt.run(&mut restored).unwrap(),
                rt.run(&mut continued).unwrap()
            );
            assert_eq!(restored, continued);
            if original.terminal.is_some() {
                break;
            }
            rt.step(&mut original).unwrap();
        }
        assert_eq!(original.emitted, vec![30, 31, 32, u32::MAX - 1]);
        assert!(rt.step(&mut original).is_err());
    }

    #[test]
    fn malformed_or_foreign_snapshots_cannot_emit_unowned_content() {
        let clauses = vec![Clause {
            seg: 10,
            tokens: vec![10, 20, 21, 30, 31],
        }];
        let rt = runtime(clauses.clone(), Some(u32::MAX - 1));
        let mut session = rt.start(&question(vec![10], &[20, 21])).unwrap();
        let mut empty_stop = session.clone();
        empty_stop.pending = RelAction::Stop;
        assert!(rt
            .restore(&serde_json::to_vec(&empty_stop).unwrap())
            .is_err());
        empty_stop.terminal = Some(RelAction::Stop);
        assert!(rt
            .restore(&serde_json::to_vec(&empty_stop).unwrap())
            .is_err());
        let mut unowned = session.clone();
        unowned.pending = RelAction::Emit;
        unowned.captured_tokens = vec![100];
        assert!(rt.restore(&serde_json::to_vec(&unowned).unwrap()).is_err());
        rt.step(&mut session).unwrap();
        let valid = rt.snapshot(&session).unwrap();
        // A fake earlier source must not pass merely because counts and the final capture agree.
        let mut bad_history = session.clone();
        bad_history.visited.insert(0, (10, 1, 2));
        bad_history.reads = 2;
        assert!(rt
            .restore(&serde_json::to_vec(&bad_history).unwrap())
            .is_err());
        // These bytes really occur in the document, but they are the marker, not an object.
        let mut marker_capture = session.clone();
        marker_capture.captured.as_mut().unwrap().start = 1;
        marker_capture.captured_span = Some((10, 1, 2));
        marker_capture.visited = vec![(10, 1, 2)];
        marker_capture.captured_tokens = vec![20, 21];
        assert!(rt
            .restore(&serde_json::to_vec(&marker_capture).unwrap())
            .is_err());
        let mut bad_cursor = session.clone();
        bad_cursor.cursor = 3;
        assert!(rt
            .restore(&serde_json::to_vec(&bad_cursor).unwrap())
            .is_err());
        let mut wrong_prefix = session.clone();
        wrong_prefix.cursor = 1;
        wrong_prefix.emitted = vec![99];
        assert!(rt
            .restore(&serde_json::to_vec(&wrong_prefix).unwrap())
            .is_err());
        let mut wrong_source = session.clone();
        wrong_source.captured_tokens[0] = 99;
        assert!(rt
            .restore(&serde_json::to_vec(&wrong_source).unwrap())
            .is_err());
        let mut wrong_goal = session.clone();
        wrong_goal.query.goal = Goal::Project;
        assert!(rt
            .restore(&serde_json::to_vec(&wrong_goal).unwrap())
            .is_err());
        let mut wrong_version = session.clone();
        wrong_version.version = 99;
        assert!(rt
            .restore(&serde_json::to_vec(&wrong_version).unwrap())
            .is_err());
        let other = runtime(
            vec![Clause {
                seg: 10,
                tokens: vec![10, 20, 21, 32, 31],
            }],
            Some(u32::MAX - 1),
        );
        assert!(other.restore(&valid).is_err());
        assert!(!other.origin_is_live(&session));
        assert_eq!(session.captured_tokens, vec![30, 31]); // immutable owned copy was not changed
        let control = runtime(clauses.clone(), Some(u32::MAX - 1))
            .with_control(TextControl::ReadsDisabled)
            .unwrap();
        assert!(control.restore(&valid).is_err());
        let model_bytes = rt.model().to_bytes().unwrap();
        let other_namespace = ObservedTextRuntime::load(
            &model_bytes,
            &"ab".repeat(32),
            clauses.clone(),
            2,
            7,
            4096,
            Some(u32::MAX - 1),
        )
        .unwrap();
        assert!(other_namespace.restore(&valid).is_err());
        let other_tokenizer = ObservedTextRuntime::load(
            &model_bytes,
            &"cd".repeat(32),
            clauses.clone(),
            1,
            7,
            4096,
            Some(u32::MAX - 1),
        )
        .unwrap();
        assert!(other_tokenizer.restore(&valid).is_err());
        let mut changed_model = rt.model().clone();
        changed_model.feature_weights[0].1[0] += 1;
        let other_model = ObservedTextRuntime::load(
            &changed_model.to_bytes().unwrap(),
            &"ab".repeat(32),
            clauses,
            1,
            7,
            4096,
            Some(u32::MAX - 1),
        )
        .unwrap();
        assert!(other_model.restore(&valid).is_err());
    }

    #[test]
    fn long_legal_phrase_completes_instead_of_being_silently_truncated() {
        let mut tokens = vec![10, 20, 21];
        let answer: Vec<u32> = (100..121).collect();
        tokens.extend_from_slice(&answer);
        assert_eq!(tokens.len(), OB_MAX_CLAUSE);
        let rt = runtime(vec![Clause { seg: 1, tokens }], Some(u32::MAX - 1));
        let mut session = rt.start(&question(vec![10], &[20, 21])).unwrap();
        rt.run(&mut session).unwrap();
        let mut expected = answer;
        expected.push(u32::MAX - 1);
        assert_eq!(session.emitted, expected);
        assert_eq!(session.terminal, Some(RelAction::Stop));
    }

    #[test]
    fn restore_rejects_forged_role_phases_and_unrelated_valid_source_history() {
        let clauses = vec![
            Clause {
                seg: 10,
                tokens: vec![10, 22, 23, 11],
            },
            Clause {
                seg: 11,
                tokens: vec![11, 20, 21, 32],
            },
            Clause {
                seg: 12,
                tokens: vec![12, 20, 21, 33],
            },
        ];
        let rt = runtime(clauses.clone(), Some(u32::MAX - 1));
        let q = question(vec![10], &[20, 21]);
        let mut session = rt.start(&q).unwrap();
        rt.step(&mut session).unwrap();
        assert_eq!(session.pending, RelAction::Continue);
        let mut forged = session.clone();
        forged.pending = RelAction::Emit;
        assert!(rt.restore(&serde_json::to_vec(&forged).unwrap()).is_err());
        rt.step(&mut session).unwrap();
        rt.step(&mut session).unwrap();
        assert_eq!(session.pending, RelAction::Emit);
        forged = session.clone();
        forged.pending = RelAction::Continue;
        assert!(rt.restore(&serde_json::to_vec(&forged).unwrap()).is_err());
        forged = session.clone();
        // The unrelated source is a genuine same-goal object, but not a predecessor of the capture.
        forged.visited[0] = (12, 3, 1);
        assert!(rt.restore(&serde_json::to_vec(&forged).unwrap()).is_err());
        rt.run(&mut session).unwrap();
        assert_eq!(session.emitted, vec![32, u32::MAX - 1]);

        // Explicit controls may legitimately override the role table; restoration must agree with
        // the bound control instead of hard-coding the primary's role decision.
        let membership = runtime(clauses, Some(u32::MAX - 1))
            .with_control(TextControl::Membership {
                entities: Vec::new(),
            })
            .unwrap();
        let mut controlled = membership.start(&q).unwrap();
        membership.step(&mut controlled).unwrap();
        assert_eq!(controlled.pending, RelAction::Emit);
        let mut restored = membership
            .restore(&membership.snapshot(&controlled).unwrap())
            .unwrap();
        membership.run(&mut restored).unwrap();
        assert_eq!(restored.emitted, vec![11, u32::MAX - 1]);
    }

    #[test]
    fn candidate_features_distinguish_local_order_but_not_four_token_interiors() {
        // The token is identical, while its observed neighbors differ.
        let a = candidate_features(&[10, 20, 30, 20, 40], 1, 1);
        let b = candidate_features(&[10, 20, 30, 20, 40], 3, 1);
        assert_ne!(a, b);
        // This is an actual permutation: token identity and bag are preserved.
        let fwd = candidate_features(&[10, 20, 30, 40], 1, 2);
        let rev = candidate_features(&[10, 30, 20, 40], 1, 2);
        assert_ne!(fwd, rev);
        // Endpoints and external context cannot distinguish this interior permutation. Record the
        // representation limit instead of claiming every permutation changes the model's score.
        assert_eq!(
            candidate_features(&[10, 20, 30, 40, 50, 60], 1, 4),
            candidate_features(&[10, 20, 40, 30, 50, 60], 1, 4)
        );
        let inner = candidate_features(&[10, 20, 30, 40], 1, 1);
        let outer = candidate_features(&[10, 20, 30, 40], 0, 1);
        assert!(outer.contains(&((F_INITIAL << 40) | 0)));
        assert!(!inner.contains(&((F_INITIAL << 40) | 0)));
    }

    #[test]
    fn model_loader_and_fit_reject_invalid_semantics_and_source_labels() {
        let (clauses, labels) = dev();
        let (model, _) = fit_observed_text_model(&clauses, &labels).unwrap();
        assert_eq!(
            ObservedTextModel::from_bytes(&model.to_bytes().unwrap(), 4096).unwrap(),
            model
        );
        assert!(matches!(
            ObservedTextModel::from_bytes(b"{", 4096),
            Err(ObservedTextError::Serialization(_))
        ));
        assert!(matches!(
            observe_clause(
                &model,
                &Clause {
                    seg: 999,
                    tokens: Vec::new()
                }
            ),
            Err(ObservedTextError::Observation(_))
        ));
        let mut invalid = model.clone();
        invalid.version = 1;
        assert!(matches!(
            ObservedTextModel::from_bytes(&invalid.to_bytes().unwrap(), 4096),
            Err(ObservedTextError::Model(_))
        ));
        invalid = model.clone();
        invalid.action_per_role[0] = 99;
        assert!(ObservedTextModel::from_bytes(&invalid.to_bytes().unwrap(), 4096).is_err());
        invalid = model.clone();
        invalid.feature_weights.push(invalid.feature_weights[0]);
        assert!(ObservedTextModel::from_bytes(&invalid.to_bytes().unwrap(), 4096).is_err());
        invalid = model.clone();
        invalid.goal_per_role[0] = 99;
        assert!(ObservedTextModel::from_bytes(&invalid.to_bytes().unwrap(), 4096).is_err());
        let mut bad_labels = labels.clone();
        bad_labels[0].marker.1 = 100;
        assert!(fit_observed_text_model(&clauses, &bad_labels).is_err());
        bad_labels = labels.clone();
        bad_labels[0].role = OB_N_ROLES;
        assert!(matches!(
            fit_observed_text_model(&clauses, &bad_labels),
            Err(ObservedTextError::Supervision(_))
        ));
    }
}
