//! Observed-text memory answers through one reusable session boundary.
//!
//! The serving input is **observed tokens plus declared system metadata only**: a tokenized document
//! of statements, an observed question clause, and declared clause boundaries. An entity registry
//! is supplied only to the explicitly named membership control. No gold semantic role,
//! subject/object/answer extent, follow pointer, depth or target is available to the primary path.
//!
//! What is learned from declared development text, under **one declared structured objective**
//! `S(x, y) = sum of the named span potentials for the roles assigned by y + the learned singleton
//! background potentials of the tokens y leaves uncovered`. The hypothesis is
//! `y = (subject span, cue span, cue role, optional object span)`.
//!
//! * `feature_weights`: signed categorical role weights over the cue span's ordered local context;
//!   the cue **role is part of the decoded hypothesis**, so span selection and role choice are fitted
//!   and served by the same score rather than by a separate staged objective.
//! * `segment_weights`: signed categorical potentials for a segment's endpoints, neighbours, ordered
//!   interior positions (`G_POS`), adjacent pairs (`G_BIGRAM`) and length, per segment role. Argument
//!   roles are **sided** relative to the cue (`SEG_SUBJECT_LEFT/RIGHT`, `SEG_OBJECT_LEFT/RIGHT`), so
//!   "the subject precedes the relation and the object follows it" is learnable structure rather than
//!   a tie between symmetric token features. The background role is learned too, as one-token
//!   segments, so a trailing adjunct competes with extending an argument instead of being free.
//! * `token_elements` plus optional ordered finite-group interval features (`use_h4`) add an exact
//!   ordered `2I` interval summary on top of the order-aware categorical arm, i.e. a **hybrid**.
//! * `action_per_role` / `goal_per_role`: role semantics fitted from declared gold goals/actions.
//!
//! Candidate support is bounded disjoint subject/cue/optional-object spans in any order; the cue
//! remains at most four tokens. Interval potentials (including the singleton background potentials)
//! are precomputed once per clause before the bounded decode. General readable-language performance
//! remains empirical.
//!
//! What is exact and typed: entity/answer spans keep full multiword token identity together with
//! their occurrence `(segment, start, len)`. Joins use tagged exact byte keys for adapter-validated
//! aligned text, or explicitly legacy token keys. The source-bound document is immutable here.
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
    if tokens.iter().any(|token| *token as usize >= OB_MAX_VOCAB) {
        return f;
    }
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
    /// The original observed text this clause was tokenized from.
    #[serde(default)]
    pub text: String,
    /// Exact byte length of each token inside `text`, so byte offsets and lexical keys are exact
    /// rather than recovered by lossy per-token decoding. May be empty for legacy fixtures.
    #[serde(default)]
    pub byte_lengths: Vec<u32>,
}

impl Clause {
    /// A clause observed without byte alignment (legacy fixtures fall back to token identity).
    pub fn of(seg: u32, tokens: Vec<u32>) -> Self {
        Clause {
            seg,
            tokens,
            text: String::new(),
            byte_lengths: Vec::new(),
        }
    }
}

/// Declared offline supervision for one development clause. Never available at serving.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
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
/// Segment roles for the bounded structured span model. Arguments are **sided**: an argument placed
/// before the cue has its own role column, so "the subject precedes the relation and the object
/// follows it" is learnable structure instead of a tie between symmetric token features.
pub const SEG_BACKGROUND: usize = 0;
pub const SEG_SUBJECT_LEFT: usize = 1;
pub const SEG_SUBJECT_RIGHT: usize = 2;
pub const SEG_CUE: usize = 3;
pub const SEG_OBJECT_LEFT: usize = 4;
pub const SEG_OBJECT_RIGHT: usize = 5;
pub const OB_SEG_ROLES: usize = 6;
/// Declared maximum length of one contiguous segment.
pub const OB_MAX_SEG: usize = OB_MAX_CLAUSE;

/// Feature kinds for the segment potential.
pub const G_FIRST: u64 = 16;
pub const G_LAST: u64 = 17;
pub const G_LEFT: u64 = 18;
pub const G_RIGHT: u64 = 19;
pub const G_LEN: u64 = 20;
pub const G_INITIAL: u64 = 21;
pub const G_FINAL: u64 = 22;
/// Every interior token of the segment, which the previous endpoint-only features omitted.
pub const G_INTERIOR: u64 = 23;
/// Ordered interval element of the segment under the learned per-token group elements.
pub const G_H4_INTERVAL: u64 = 24;
/// Relative element between the segment interval and its left/right neighbour intervals.
pub const G_H4_LEFT: u64 = 25;
pub const G_H4_RIGHT: u64 = 26;
/// Ordered relative-position token inside a segment: `(offset << 16) | token`. This gives the
/// categorical arm bounded **order** information, so comparing it against the geometric arm does not
/// confound geometry with the geometric arm alone seeing order.
pub const G_POS: u64 = 27;
/// Ordered adjacent token pair inside a segment: `(left << 16) | right`.
pub const G_BIGRAM: u64 = 28;
/// Shift used by the ordered categorical feature payloads.
pub const G_ORDER_SHIFT: u64 = 16;
/// The feature packing uses two disjoint 16-bit token fields. This is an explicit model domain,
/// not a truncation: larger vocabularies require a new packing/format before they can be served.
pub const OB_MAX_VOCAB: usize = 1usize << G_ORDER_SHIFT;

/// Exact lexical key of a selected span: the span's original bytes with only exterior ASCII
/// whitespace removed. Case and all interior bytes are preserved, so `Cedar` and `Cedar Annex` stay
/// different and the same name keeps one key across BPE boundary differences at sentence positions.
/// The surface span remains the provenance and copy source; this key never replaces it.
pub fn lexical_key(text: &str, byte_lengths: &[u32], start: usize, len: usize) -> Option<Vec<u8>> {
    let end = start
        .checked_add(len)
        .filter(|end| len > 0 && *end <= byte_lengths.len())?;
    if byte_lengths.contains(&0) {
        return None;
    }
    let sum = |lengths: &[u32]| {
        lengths
            .iter()
            .try_fold(0usize, |n, b| n.checked_add(*b as usize))
    };
    if sum(byte_lengths)? != text.len() {
        return None;
    }
    let offset = sum(&byte_lengths[..start])?;
    let span = sum(&byte_lengths[start..end])?;
    let slice = text.as_bytes().get(offset..offset.checked_add(span)?)?;
    let lo = slice
        .iter()
        .position(|b| !b.is_ascii_whitespace())
        .unwrap_or(slice.len());
    let hi = slice
        .iter()
        .rposition(|b| !b.is_ascii_whitespace())
        .map(|i| i + 1)
        .unwrap_or(lo);
    (lo < hi).then(|| slice[lo..hi].to_vec())
}

/// The adapter must additionally verify decoded token bytes against the original input. Lengths
/// alone establish extents, not authenticity of a tokenizer/text pairing. Empty lengths explicitly
/// select the legacy token-only regime; malformed supplied alignment is never silently downgraded.
fn validate_alignment(clause: &Clause) -> Result<(), String> {
    if clause.byte_lengths.is_empty() {
        return Ok(());
    }
    if clause.byte_lengths.len() != clause.tokens.len()
        || clause.byte_lengths.contains(&0)
        || clause
            .byte_lengths
            .iter()
            .try_fold(0usize, |n, b| n.checked_add(*b as usize))
            != Some(clause.text.len())
    {
        return Err("supplied byte alignment does not tile the observed text".into());
    }
    Ok(())
}

fn identity_key(clause: &Clause, start: usize, len: usize) -> Result<Vec<u8>, String> {
    let mut key;
    if clause.byte_lengths.is_empty() {
        key = vec![0]; // distinct from exact text bytes, even when payload bytes coincide
        let end = start.checked_add(len).ok_or("token key extent overflow")?;
        for token in clause
            .tokens
            .get(start..end)
            .ok_or("token key outside clause")?
        {
            key.extend_from_slice(&token.to_le_bytes());
        }
    } else {
        key = vec![1];
        key.extend(
            lexical_key(&clause.text, &clause.byte_lengths, start, len)
                .ok_or("selected span has no non-whitespace lexical identity")?,
        );
    }
    Ok(key)
}

fn valid_identity_key(key: &[u8]) -> bool {
    match key.split_first() {
        Some((0, rest)) => !rest.is_empty() && rest.len() % 4 == 0,
        Some((1, rest)) => {
            !rest.is_empty()
                && !rest[0].is_ascii_whitespace()
                && !rest[rest.len() - 1].is_ascii_whitespace()
        }
        _ => false,
    }
}

/// Recover a segment's interval product from ordered group prefixes: `inverse(P[start]) * P[end]`.
fn interval_product(prefix: &[usize], start: usize, end: usize) -> Option<usize> {
    let t = super::group_table::group_table();
    let a = *prefix.get(start)?;
    let b = *prefix.get(end)?;
    if start > end || a >= super::group_table::GROUP_ORDER || b >= super::group_table::GROUP_ORDER {
        return None;
    }
    Some(t.product[t.inverse[a] as usize * super::group_table::ROW_STRIDE + b] as usize)
}

/// The relative group element between two interval products, `inverse(a) * b`.
fn relative_element(a: usize, b: usize) -> usize {
    let t = super::group_table::group_table();
    t.product[t.inverse[a.min(super::group_table::GROUP_ORDER - 1)] as usize
        * super::group_table::ROW_STRIDE
        + b.min(super::group_table::GROUP_ORDER - 1)] as usize
}

/// One decoded hypothesis: exact token extents for subject, cue and optional object, **and the cue
/// role**, all chosen under the single declared score.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Segmentation {
    pub subject: (usize, usize),
    pub cue: (usize, usize),
    /// The semantic cue role (0..OB_N_ROLES) selected jointly with the spans.
    pub role: usize,
    pub object: Option<(usize, usize)>,
    pub score: i64,
}

impl Segmentation {
    /// The exact hypothesis compared for fitting and evaluation: spans plus role.
    pub fn same_hypothesis(&self, other: &Segmentation) -> bool {
        self.subject == other.subject
            && self.cue == other.cue
            && self.object == other.object
            && self.role == other.role
    }
}

/// Per-clause potentials prepared once before the bounded decode. `seg[start * stride + end]` is the
/// span potential for a segment role; `role[start * stride + end]` is the cue role potential.
/// Indexing is by half-open token interval `[start, end)`.
struct Potentials {
    stride: usize,
    seg: Vec<[i64; OB_SEG_ROLES]>,
    role: Vec<[i64; OB_N_ROLES]>,
}

impl Potentials {
    #[inline]
    fn seg(&self, start: usize, end: usize, role: usize) -> i64 {
        self.seg[start * self.stride + end][role]
    }
    #[inline]
    fn role(&self, start: usize, end: usize, role: usize) -> i64 {
        self.role[start * self.stride + end][role]
    }
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
    /// Learned segment potentials: `(feature key, weight per segment role)`.
    pub segment_weights: Vec<(u64, [i32; OB_SEG_ROLES])>,
    /// Learned per-token group elements used by the ordered interval features when `use_h4`.
    pub token_elements: Vec<(u32, u8)>,
    /// Whether the ordered H4 interval features are part of this model's feature family.
    pub use_h4: bool,
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
            version: 6,
            feature_weights: Vec::new(),
            segment_weights: Vec::new(),
            token_elements: Vec::new(),
            use_h4: false,
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
        if self.version != 6 {
            return Err("unsupported observed-text model version".into());
        }
        if max_vocab == 0 || max_vocab > OB_MAX_VOCAB {
            return Err("vocabulary must fit the declared 16-bit token feature domain".into());
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
        if self.segment_weights.windows(2).any(|w| w[0].0 >= w[1].0)
            || self.token_elements.windows(2).any(|w| w[0].0 >= w[1].0)
        {
            return Err("segment weights and token elements must have unique sorted keys".into());
        }
        for (key, _weights) in &self.segment_weights {
            let kind = key >> 40;
            let value = key & ((1u64 << 40) - 1);
            if !(G_FIRST..=G_BIGRAM).contains(&kind) {
                return Err("unknown segment feature".into());
            }
            match kind {
                G_FIRST..=G_RIGHT | G_INTERIOR
                    if value > u32::MAX as u64 || value >= max_vocab as u64 =>
                {
                    return Err("segment token outside vocabulary".into())
                }
                G_POS => {
                    let token = value & ((1u64 << G_ORDER_SHIFT) - 1);
                    if (value >> G_ORDER_SHIFT) >= OB_MAX_SEG as u64 || token >= max_vocab as u64 {
                        return Err("ordered position feature outside its bound".into());
                    }
                }
                G_BIGRAM => {
                    if (value >> G_ORDER_SHIFT) >= max_vocab as u64
                        || (value & ((1u64 << G_ORDER_SHIFT) - 1)) >= max_vocab as u64
                    {
                        return Err("ordered pair feature outside the vocabulary".into());
                    }
                }
                G_LEN if value == 0 || value > OB_MAX_SEG as u64 => {
                    return Err("segment length outside bound".into())
                }
                G_INITIAL | G_FINAL if value != 0 => {
                    return Err("segment edge feature has a payload".into())
                }
                G_H4_INTERVAL..=G_H4_RIGHT
                    if !self.use_h4 || value >= super::group_table::GROUP_ORDER as u64 =>
                {
                    return Err("invalid or disabled group feature".into())
                }
                _ => {}
            }
        }
        if self.token_elements.iter().any(|(token, element)| {
            *token as usize >= max_vocab || *element as usize >= super::group_table::GROUP_ORDER
        }) || (!self.use_h4 && !self.token_elements.is_empty())
        {
            return Err("invalid token/group map".into());
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

    /// Historical narrow score view. The joint decoder uses the exact wide scores below; it never
    /// clips role potentials before choosing the winning hypothesis.
    pub fn candidate_scores(&self, tokens: &[u32], start: usize, len: usize) -> [i32; OB_N_ROLES] {
        self.cue_scores_wide(tokens, start, len)
            .map(|score| score.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32)
    }

    fn cue_scores_wide(&self, tokens: &[u32], start: usize, len: usize) -> [i64; OB_N_ROLES] {
        let mut out = [0i64; OB_N_ROLES];
        for key in candidate_features(tokens, start, len) {
            if let Ok(i) = self.feature_weights.binary_search_by_key(&key, |(k, _)| *k) {
                let row = self.feature_weights[i].1;
                for r in 0..OB_N_ROLES {
                    out[r] += i64::from(row[r]);
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

    /// Learned per-token group element for the ordered interval features.
    pub fn token_element(&self, token: u32) -> usize {
        self.token_elements
            .binary_search_by_key(&token, |(t, _)| *t)
            .ok()
            .map(|i| self.token_elements[i].1 as usize)
            .unwrap_or_else(|| super::group_table::group_table().identity as usize)
    }

    /// Ordered group prefixes `P[j] = g(x_0) ... g(x_{j-1})` for one clause.
    fn prefix_products(&self, tokens: &[u32]) -> Vec<usize> {
        let t = super::group_table::group_table();
        let mut p = Vec::with_capacity(tokens.len() + 1);
        let mut acc = t.identity as usize;
        p.push(acc);
        for tok in tokens {
            acc = t.product[acc * super::group_table::ROW_STRIDE
                + self
                    .token_element(*tok)
                    .min(super::group_table::GROUP_ORDER - 1)] as usize;
            p.push(acc);
        }
        p
    }

    /// The one segment observation extractor, used by fitting and serving. It adds every interior
    /// token and, when the arm enables them, the exact ordered interval element and its relative
    /// relations to the neighbouring intervals.
    pub fn segment_features(
        &self,
        tokens: &[u32],
        start: usize,
        len: usize,
        prefix: &[usize],
    ) -> Vec<u64> {
        let n = tokens.len();
        let key = |kind: u64, value: u64| (kind << 40) | value;
        let mut f = Vec::with_capacity(12);
        if tokens.len() > OB_MAX_CLAUSE
            || tokens.iter().any(|token| *token as usize >= OB_MAX_VOCAB)
        {
            return f;
        }
        let Some(end) = start
            .checked_add(len)
            .filter(|end| len > 0 && len <= OB_MAX_SEG && *end <= n)
        else {
            return f;
        };
        if self.use_h4
            && (prefix.len() != n + 1
                || prefix.iter().any(|p| *p >= super::group_table::GROUP_ORDER))
        {
            return f;
        }
        f.push(key(G_FIRST, u64::from(tokens[start])));
        f.push(key(G_LAST, u64::from(tokens[start + len - 1])));
        f.push(key(G_LEN, len as u64));
        if start > 0 {
            f.push(key(G_LEFT, u64::from(tokens[start - 1])));
        }
        if start + len < n {
            f.push(key(G_RIGHT, u64::from(tokens[start + len])));
        }
        for i in (start + 1)..(start + len).saturating_sub(1) {
            if i + 1 < start + len {
                f.push(key(G_INTERIOR, u64::from(tokens[i])));
            }
        }
        // Bounded *ordered* categorical features. The interior bag alone cannot carry order, so the
        // categorical comparator would otherwise lack information the geometric arm receives.
        for i in start..end {
            f.push(key(
                G_POS,
                (((i - start) as u64) << G_ORDER_SHIFT) | u64::from(tokens[i]),
            ));
        }
        for i in start..end.saturating_sub(1) {
            f.push(key(
                G_BIGRAM,
                (u64::from(tokens[i]) << G_ORDER_SHIFT) | u64::from(tokens[i + 1]),
            ));
        }
        if start == 0 {
            f.push(key(G_INITIAL, 0));
        }
        if start + len == n {
            f.push(key(G_FINAL, 0));
        }
        if self.use_h4 {
            let Some(interval) = interval_product(prefix, start, end) else {
                return Vec::new();
            };
            f.push(key(G_H4_INTERVAL, interval as u64));
            if start == 0 {
                f.push(key(G_H4_LEFT, interval as u64));
            } else {
                let Some(prev_interval) = interval_product(prefix, start - 1, start) else {
                    return Vec::new();
                };
                f.push(key(
                    G_H4_LEFT,
                    relative_element(prev_interval, interval) as u64,
                ));
            }
            if start + len == n {
                f.push(key(G_H4_RIGHT, interval as u64));
            } else {
                let Some(next_interval) = interval_product(prefix, end, end + 1) else {
                    return Vec::new();
                };
                f.push(key(
                    G_H4_RIGHT,
                    relative_element(interval, next_interval) as u64,
                ));
            }
        }
        f
    }

    pub fn segment_score(
        &self,
        tokens: &[u32],
        start: usize,
        len: usize,
        role: usize,
        prefix: &[usize],
    ) -> i64 {
        if role >= OB_SEG_ROLES {
            return 0;
        }
        let mut acc = 0i64;
        for key in self.segment_features(tokens, start, len, prefix) {
            if let Ok(i) = self.segment_weights.binary_search_by_key(&key, |(k, _)| *k) {
                acc += i64::from(self.segment_weights[i].1[role]);
            }
        }
        acc
    }

    /// Precompute every interval potential once per clause. The bounded decode then performs only
    /// table reads, instead of repeating interior scans and sparse lookups inside each
    /// position/mask/length/role loop.
    fn interval_potentials(&self, tokens: &[u32], prefix: &[usize]) -> Potentials {
        let n = tokens.len();
        let stride = n + 1;
        let mut seg = vec![[0i64; OB_SEG_ROLES]; stride * stride];
        let mut role = vec![[0i64; OB_N_ROLES]; stride * stride];
        for start in 0..n {
            for end in (start + 1)..=n {
                let len = end - start;
                let idx = start * stride + end;
                for r in 0..OB_SEG_ROLES {
                    seg[idx][r] = self.segment_score(tokens, start, len, r, prefix);
                }
                if len <= OB_MAX_MARKER {
                    role[idx] = self.cue_scores_wide(tokens, start, len);
                }
            }
        }
        Potentials { stride, seg, role }
    }

    /// **One declared structured objective** over the joint hypothesis
    /// `(subject span, cue span, cue role, optional object span)`. Subject/cue/object may occur in any
    /// order and each at most once; uncovered tokens contribute learned singleton background scores.
    /// Span selection and role choice are one argmax, so fitting and serving optimise the same score.
    pub fn decode_segmentation(&self, tokens: &[u32]) -> Option<Segmentation> {
        let n = tokens.len();
        if n == 0 || n > OB_MAX_CLAUSE || tokens.iter().any(|token| *token as usize >= OB_MAX_VOCAB)
        {
            return None;
        }
        let prefix = self.prefix_products(tokens);
        let potential = self.interval_potentials(tokens, &prefix);
        decode_segmentation_with(tokens, &potential)
    }

    /// The learned four-way cue role of an observed cue span, from the retained categorical scorer.
    pub fn cue_role(&self, tokens: &[u32], start: usize, len: usize) -> usize {
        let scores = self.cue_scores_wide(tokens, start, len);
        (0..OB_N_ROLES)
            .max_by_key(|r| (scores[*r], std::cmp::Reverse(*r)))
            .unwrap_or(0)
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

/// Slot indices for the backtracking array. Distinct from the segment-role constants, which now
/// distinguish the two sides of the cue.
const SLOT_SUBJECT: usize = 0;
const SLOT_CUE: usize = 1;
const SLOT_OBJECT: usize = 2;

/// The bounded joint decode over precomputed potentials. Split from [`ObservedTextModel`] so the
/// recurrence can be exercised directly against exhaustive legal assignments on tiny examples.
const DECODE_NEG: i64 = i64::MIN / 4;
const SEM_UNSET: usize = OB_N_ROLES;
const SEM_COUNT: usize = OB_N_ROLES + 1;

fn decode_segmentation_with(tokens: &[u32], pot: &Potentials) -> Option<Segmentation> {
    let n = tokens.len();
    let idx = |pos: usize, mask: usize, sem: usize| (pos * 8 + mask) * SEM_COUNT + sem;
    let states = (n + 1) * 8 * SEM_COUNT;
    let mut best = vec![DECODE_NEG; states];
    let mut choice = vec![None::<(u32, u8)>; states];
    best[idx(0, 0, SEM_UNSET)] = 0;
    for pos in 0..n {
        for mask in 0..8usize {
            for sem in 0..SEM_COUNT {
                let cur = best[idx(pos, mask, sem)];
                if cur == DECODE_NEG {
                    continue;
                }
                // Background is a single learned-potential unit step; longer background is a chain of
                // them, so no latent multi-token background segmentation is introduced.
                let bg = cur.saturating_add(pot.seg(pos, pos + 1, SEG_BACKGROUND));
                if bg > best[idx(pos + 1, mask, sem)] {
                    best[idx(pos + 1, mask, sem)] = bg;
                    choice[idx(pos + 1, mask, sem)] = Some((pos as u32, 0));
                }
                for len in 1..=(n - pos).min(OB_MAX_SEG) {
                    let end = pos + len;
                    if mask & 1 == 0 {
                        let role = if mask & 2 != 0 {
                            SEG_SUBJECT_RIGHT
                        } else {
                            SEG_SUBJECT_LEFT
                        };
                        let sc = cur.saturating_add(pot.seg(pos, end, role));
                        if sc > best[idx(end, mask | 1, sem)] {
                            best[idx(end, mask | 1, sem)] = sc;
                            choice[idx(end, mask | 1, sem)] = Some((pos as u32, 1));
                        }
                    }
                    if mask & 4 == 0 {
                        let role = if mask & 2 != 0 {
                            SEG_OBJECT_RIGHT
                        } else {
                            SEG_OBJECT_LEFT
                        };
                        let sc = cur.saturating_add(pot.seg(pos, end, role));
                        if sc > best[idx(end, mask | 4, sem)] {
                            best[idx(end, mask | 4, sem)] = sc;
                            choice[idx(end, mask | 4, sem)] = Some((pos as u32, 2));
                        }
                    }
                    if mask & 2 == 0 && len <= OB_MAX_MARKER {
                        let base = cur.saturating_add(pot.seg(pos, end, SEG_CUE));
                        for r in 0..OB_N_ROLES {
                            let sc = base.saturating_add(pot.role(pos, end, r));
                            if sc > best[idx(end, mask | 2, r)] {
                                best[idx(end, mask | 2, r)] = sc;
                                choice[idx(end, mask | 2, r)] = Some((pos as u32, (10 + r) as u8));
                            }
                        }
                    }
                }
            }
        }
    }
    // A question states subject and cue; a statement adds an object. A question wins a tie, so a
    // clause whose best object hypothesis adds nothing stays a question.
    let (mask_question, mask_statement) = (1usize | 2, 1usize | 2 | 4);
    let mut statement: Option<(i64, usize)> = None;
    let mut question: Option<(i64, usize)> = None;
    for sem in 0..OB_N_ROLES {
        let s = best[idx(n, mask_statement, sem)];
        if s != DECODE_NEG && statement.is_none_or(|(b, _)| s > b) {
            statement = Some((s, sem));
        }
        let q = best[idx(n, mask_question, sem)];
        if q != DECODE_NEG && question.is_none_or(|(b, _)| q > b) {
            question = Some((q, sem));
        }
    }
    let (mask, sem, score) = match (statement, question) {
        (Some((s, ss)), Some((q, qs))) => {
            if s > q {
                (mask_statement, ss, s)
            } else {
                (mask_question, qs, q)
            }
        }
        (Some((s, ss)), None) => (mask_statement, ss, s),
        (None, Some((q, qs))) => (mask_question, qs, q),
        (None, None) => return None,
    };
    let mut spans: [Option<(usize, usize)>; 3] = [None; 3];
    let mut role: Option<usize> = None;
    let mut pos = n;
    let mut m = mask;
    let mut s = sem;
    while pos > 0 {
        let (prev, kind) = choice[idx(pos, m, s)]?;
        let prev = prev as usize;
        let len = pos - prev;
        match kind {
            0 => {}
            1 => {
                spans[SLOT_SUBJECT] = Some((prev, len));
                m &= !1;
            }
            2 => {
                spans[SLOT_OBJECT] = Some((prev, len));
                m &= !4;
            }
            k => {
                spans[SLOT_CUE] = Some((prev, len));
                role = Some((k - 10) as usize);
                m &= !2;
                s = SEM_UNSET;
            }
        }
        pos = prev;
    }
    Some(Segmentation {
        subject: spans[SLOT_SUBJECT]?,
        cue: spans[SLOT_CUE]?,
        role: role?,
        object: spans[SLOT_OBJECT],
        score,
    })
}

/// The learned observation extractor: clause -> (subject, marker, object, role, question goal).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    pub seg: u32,
    /// Exact lexical key of the subject span; this is the identity used for joins.
    pub subject_key: Vec<u8>,
    /// Exact lexical key of the object span, when the clause states an object.
    pub object_key: Option<Vec<u8>>,
    pub subject: Vec<u32>,
    pub marker: Vec<u32>,
    pub object: Option<Vec<u32>>,
    pub subject_start: u32,
    pub marker_start: u32,
    pub object_start: Option<u32>,
    pub role: usize,
    /// The joint structured score of this clause's decoded hypothesis.
    pub score: i64,
    /// Present when the winning hypothesis has no object span, including interior-cue questions.
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
    validate_alignment(clause)?;
    let seg = model
        .decode_segmentation(&clause.tokens)
        .ok_or("no legal structured span assignment in clause")?;
    let subject = clause.tokens[seg.subject.0..seg.subject.0 + seg.subject.1].to_vec();
    let role = seg.role;
    let object = seg.object.map(|(s, l)| clause.tokens[s..s + l].to_vec());
    let question_goal = if object.is_none() {
        model.role_goal(role)
    } else {
        None
    };
    Ok(Observation {
        seg: clause.seg,
        subject_key: identity_key(clause, seg.subject.0, seg.subject.1)?,
        object_key: seg
            .object
            .map(|(s, l)| identity_key(clause, s, l))
            .transpose()?,
        subject_start: seg.subject.0 as u32,
        marker_start: seg.cue.0 as u32,
        object_start: seg.object.map(|(s, _)| s as u32),
        subject,
        marker: clause.tokens[seg.cue.0..seg.cue.0 + seg.cue.1].to_vec(),
        object,
        role,
        score: seg.score,
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
    /// Historical control with exact token-vector membership, retained at its original boundary.
    Membership {
        entities: Vec<Vec<u32>>,
    },
    /// Matched-identity control. Entries are exact original lexical bytes with exterior ASCII
    /// whitespace removed, without the internal identity-mode tag. Only aligned text is eligible.
    LexicalMembership {
        entities: Vec<Vec<u8>>,
    },
    ReadCap {
        max_reads: u8,
    },
}

/// Exact source provenance for an owned phrase, including document identity and full extent.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextCapture {
    /// Exact lexical key of the captured object span: the identity later joins are made on.
    pub key: Vec<u8>,
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
    /// Exact lexical key of the current query argument; joins use this, not the token form.
    pub key: Vec<u8>,
    /// Surface tokens of the current query argument, kept for provenance and emission.
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
    fn start(binding: TextBinding, goal: Goal, key: Vec<u8>, tokens: Vec<u32>) -> Self {
        Self {
            version: 3,
            eos: binding.eos,
            binding,
            goal,
            query: TextObjective { goal, key, tokens },
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
        if self.version != 3
            || &self.binding != expected
            || max_vocab != expected.max_vocab
            || self.eos != expected.eos
        {
            return Err("session version or runtime binding mismatch".into());
        }
        if self.goal != self.query.goal
            || self.query.tokens.is_empty()
            || self.query.tokens.len() > OB_MAX_CLAUSE
            || !valid_identity_key(&self.query.key)
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
                    || !valid_identity_key(&capture.key)
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
            TextControl::LexicalMembership { entities } => {
                if entities.iter().any(|e| {
                    e.is_empty()
                        || e.first().is_some_and(u8::is_ascii_whitespace)
                        || e.last().is_some_and(u8::is_ascii_whitespace)
                }) || self
                    .observations
                    .iter()
                    .filter_map(|o| o.object_key.as_ref())
                    .any(|key| key.first() != Some(&1))
                {
                    return Err(ObservedTextError::Observation(
                        "lexical membership requires nonempty canonical raw keys and aligned source objects".into(),
                    ));
                }
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
        let frame = TextSession::start(
            self.binding.clone(),
            goal,
            observed.subject_key.clone(),
            observed.subject.clone(),
        );
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
                || pair[0].object_key.as_ref() != Some(&pair[1].subject_key)
            {
                return Err("visited objects do not form a dependent continuation chain".into());
            }
        }
        if let Some(last) = history.last() {
            if session.captured.as_ref().map(|capture| &capture.key) != last.object_key.as_ref() {
                return Err("captured lexical identity disagrees with its bound source".into());
            }
            let action = self.action_after_read(last)?;
            let followed = session.pending == RelAction::Read
                || matches!(
                    session.terminal,
                    Some(RelAction::Unresolved | RelAction::Exhausted)
                );
            if followed {
                if action != RelAction::Continue
                    || last.object_key.as_ref() != Some(&session.query.key)
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
                if action != expected_action || last.subject_key != session.query.key {
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
            TextControl::LexicalMembership { entities } => {
                let raw_key = observed
                    .object_key
                    .as_deref()
                    .and_then(|key| key.strip_prefix(&[1]))
                    .ok_or("lexical membership requires an aligned source object's exact key")?;
                if entities.iter().any(|key| key.as_slice() == raw_key) {
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
                    let mut best: Option<(i64, usize)> = None;
                    for (i, observed) in self.observations.iter().enumerate() {
                        if observed.subject_key != session.query.key
                            || self.model.role_goal(observed.role) != Some(session.goal)
                            || observed.object.is_none()
                        {
                            continue;
                        }
                        // Rank admitted statements by their own joint structured score: the same
                        // declared objective the decoder maximises, not a separate auxiliary score.
                        let score = observed.score;
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
                                key: observed.object_key.clone().unwrap_or_default(),
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
                    key: session
                        .captured
                        .as_ref()
                        .map(|c| c.key.clone())
                        .unwrap_or_default(),
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
    /// Learned joint span potentials (`segment_weights`).
    pub segment_potentials: usize,
    /// Learned cue-role weights (`feature_weights`).
    pub role_weights: usize,
    /// Exact cue extent **and** cue-role matches on the same clauses.
    pub cue_role_correct: usize,
    pub h4_enabled: bool,
    /// Accepted strictly-improving code-map moves and the code evaluations they cost.
    pub h4_moves: usize,
    pub h4_evaluations: usize,
    pub candidate_updates: usize,
    pub epochs_run: usize,
    /// Same-unit exact subject/cue/object extents (role ignored) at the declared uninformed start and
    /// after fitting. Reported separately so a joint-hypothesis score cannot hide a wrong role.
    pub span_initial_correct: usize,
    pub span_final_correct: usize,
    /// Same-unit exact **joint** hypotheses (spans and cue role).
    pub role_initial_correct: usize,
    pub role_final_correct: usize,
    /// Role-action table accounting on the same gold statement-action units as before.
    pub policy_initial_correct: usize,
    pub policy_final_correct: usize,
    pub policy_examples: usize,
    pub roles_observed: Vec<usize>,
}

/// The declared gold hypothesis for one development clause.
fn gold_segmentation(label: &ClauseLabel) -> Segmentation {
    Segmentation {
        subject: (label.subject.0 as usize, label.subject.1),
        cue: (label.marker.0 as usize, label.marker.1),
        role: label.role,
        object: label.object.map(|(s, l)| (s as usize, l)),
        score: 0,
    }
}

fn add_seg_weight(
    weights: &mut Vec<(u64, [i32; OB_SEG_ROLES])>,
    key: u64,
    role: usize,
    delta: i32,
) {
    match weights.binary_search_by_key(&key, |(k, _)| *k) {
        Ok(i) => weights[i].1[role] = weights[i].1[role].saturating_add(delta),
        Err(i) => {
            let mut w = [0i32; OB_SEG_ROLES];
            w[role] = delta;
            weights.insert(i, (key, w));
        }
    }
}

fn add_role_weight(weights: &mut Vec<(u64, [i32; OB_N_ROLES])>, key: u64, role: usize, delta: i32) {
    match weights.binary_search_by_key(&key, |(k, _)| *k) {
        Ok(i) => weights[i].1[role] = weights[i].1[role].saturating_add(delta),
        Err(i) => {
            let mut w = [0i32; OB_N_ROLES];
            w[role] = delta;
            weights.insert(i, (key, w));
        }
    }
}

/// Tokens covered by the assigned spans of `seg`; every other token is background.
fn covered_tokens(n: usize, seg: &Segmentation) -> Vec<bool> {
    let mut covered = vec![false; n];
    for span in [Some(seg.subject), Some(seg.cue), seg.object]
        .into_iter()
        .flatten()
    {
        for i in span.0..(span.0 + span.1).min(n) {
            covered[i] = true;
        }
    }
    covered
}

/// Apply `delta` for every feature of `seg`. The `+1`/`-1` pair on gold and prediction is exactly the
/// structured feature difference the decode scores, so span potentials and cue-role weights are
/// fitted against the **same** objective that inference maximises.
fn apply_segmentation(
    model: &mut ObservedTextModel,
    clause: &Clause,
    seg: &Segmentation,
    prefix: &[usize],
    delta: i32,
) {
    let cue_end = seg.cue.0 + seg.cue.1;
    let right_of_cue = |span: (usize, usize)| span.0 >= cue_end;
    let subject_role = if right_of_cue(seg.subject) {
        SEG_SUBJECT_RIGHT
    } else {
        SEG_SUBJECT_LEFT
    };
    {
        let keys = model.segment_features(&clause.tokens, seg.subject.0, seg.subject.1, prefix);
        for key in keys {
            add_seg_weight(&mut model.segment_weights, key, subject_role, delta);
        }
    }
    {
        let keys = model.segment_features(&clause.tokens, seg.cue.0, seg.cue.1, prefix);
        for key in keys {
            add_seg_weight(&mut model.segment_weights, key, SEG_CUE, delta);
        }
    }
    if let Some(object) = seg.object {
        let role = if right_of_cue(object) {
            SEG_OBJECT_RIGHT
        } else {
            SEG_OBJECT_LEFT
        };
        let keys = model.segment_features(&clause.tokens, object.0, object.1, prefix);
        for key in keys {
            add_seg_weight(&mut model.segment_weights, key, role, delta);
        }
    }
    for (i, is_covered) in covered_tokens(clause.tokens.len(), seg).iter().enumerate() {
        if *is_covered {
            continue;
        }
        let keys = model.segment_features(&clause.tokens, i, 1, prefix);
        for key in keys {
            add_seg_weight(&mut model.segment_weights, key, SEG_BACKGROUND, delta);
        }
    }
    let keys = candidate_features(&clause.tokens, seg.cue.0, seg.cue.1);
    for key in keys {
        add_role_weight(&mut model.feature_weights, key, seg.role, delta);
    }
}

fn clause_correct(model: &ObservedTextModel, clause: &Clause, label: &ClauseLabel) -> bool {
    model
        .decode_segmentation(&clause.tokens)
        .is_some_and(|p| p.same_hypothesis(&gold_segmentation(label)))
}

fn span_correct(model: &ObservedTextModel, clause: &Clause, label: &ClauseLabel) -> bool {
    let gold = gold_segmentation(label);
    model
        .decode_segmentation(&clause.tokens)
        .is_some_and(|p| p.subject == gold.subject && p.cue == gold.cue && p.object == gold.object)
}

fn exact_hypotheses(
    model: &ObservedTextModel,
    clauses: &[Clause],
    labels: &[ClauseLabel],
) -> usize {
    labels
        .iter()
        .filter(|label| {
            clauses
                .iter()
                .find(|c| c.seg == label.seg)
                .is_some_and(|clause| clause_correct(model, clause, label))
        })
        .count()
}

fn exact_spans(model: &ObservedTextModel, clauses: &[Clause], labels: &[ClauseLabel]) -> usize {
    labels
        .iter()
        .filter(|label| {
            clauses
                .iter()
                .find(|c| c.seg == label.seg)
                .is_some_and(|clause| span_correct(model, clause, label))
        })
        .count()
}

/// The one declared-objective fit: a structured perceptron over the joint hypothesis. The update is
/// the scored feature difference. Model selection retains the best exact joint-hypothesis count on
/// the fitting clauses; this discrete selection metric differs from the linear inference score.
fn fit_joint_model(
    model: &mut ObservedTextModel,
    clauses: &[Clause],
    labels: &[ClauseLabel],
) -> (usize, usize) {
    let mut updates = 0usize;
    let mut epochs_run = 0usize;
    let mut best = exact_hypotheses(model, clauses, labels);
    let mut best_model = model.clone();
    for _epoch in 0..24 {
        epochs_run += 1;
        let mut changed = false;
        for label in labels {
            let Some(clause) = clauses.iter().find(|c| c.seg == label.seg) else {
                continue;
            };
            let gold = gold_segmentation(label);
            let pred = model.decode_segmentation(&clause.tokens);
            if pred.as_ref().is_some_and(|p| p.same_hypothesis(&gold)) {
                continue;
            }
            let prefix = model.prefix_products(&clause.tokens);
            apply_segmentation(model, clause, &gold, &prefix, 1);
            if let Some(p) = pred.as_ref() {
                apply_segmentation(model, clause, p, &prefix, -1);
            }
            updates += 1;
            changed = true;
        }
        let value = exact_hypotheses(model, clauses, labels);
        if value > best {
            best = value;
            best_model = model.clone();
        }
        if !changed {
            break;
        }
    }
    *model = best_model;
    (updates, epochs_run)
}

/// Bounded coordinate search over the ordered group code map, selected by exact fitting-clause
/// joint-hypothesis count. Only affected clauses are re-decoded. At perfect fitting accuracy no
/// strictly improving move exists; zero moves there do not diagnose geometric expressiveness.
fn coordinate_search_codes(
    model: &mut ObservedTextModel,
    clauses: &[Clause],
    labels: &[ClauseLabel],
) -> (usize, usize) {
    let clause_of: Vec<Option<&Clause>> = labels
        .iter()
        .map(|label| clauses.iter().find(|c| c.seg == label.seg))
        .collect();
    let correct = |model: &ObservedTextModel, i: usize| -> bool {
        clause_of[i].is_some_and(|clause| clause_correct(model, clause, &labels[i]))
    };
    let mut hits: Vec<bool> = (0..labels.len()).map(|i| correct(model, i)).collect();
    let mut moves = 0usize;
    let mut evaluations = 0usize;
    for _pass in 0..2 {
        let mut improved = false;
        let tokens: Vec<u32> = model.token_elements.iter().map(|(t, _)| *t).collect();
        for (idx, token) in tokens.iter().enumerate() {
            let affected: Vec<usize> = (0..labels.len())
                .filter(|i| clause_of[*i].is_some_and(|c| c.tokens.contains(token)))
                .collect();
            if affected.is_empty() {
                continue;
            }
            let base = affected.iter().filter(|i| hits[**i]).count();
            let saved = model.token_elements[idx].1;
            let mut best = base;
            let mut best_val = saved;
            for cand in 0..super::group_table::GROUP_ORDER {
                model.token_elements[idx].1 = cand as u8;
                let count = affected.iter().filter(|i| correct(model, **i)).count();
                evaluations += 1;
                if count > best {
                    best = count;
                    best_val = cand as u8;
                }
            }
            model.token_elements[idx].1 = best_val;
            if best > base {
                for &i in &affected {
                    hits[i] = correct(model, i);
                }
                moves += 1;
                improved = true;
            }
        }
        if !improved {
            break;
        }
    }
    (moves, evaluations)
}

/// Fit the observation model from declared development clauses and their gold labels.
///
/// Supervision: the declared subject/cue/object extents and cue role are the gold hypothesis.
/// Features come from the **same** extractors serving uses, so no gold field can become an input
/// feature. The learner is a bounded structured perceptron over the joint hypothesis; the declared
/// uninformed start has no weights at all.
pub fn fit_observed_text_model(
    clauses: &[Clause],
    labels: &[ClauseLabel],
    use_h4: bool,
) -> Result<(ObservedTextModel, ObFitReport), ObservedTextError> {
    fit_observed_text_model_inner(clauses, labels, use_h4).map_err(ObservedTextError::Supervision)
}

fn fit_observed_text_model_inner(
    clauses: &[Clause],
    labels: &[ClauseLabel],
    use_h4: bool,
) -> Result<(ObservedTextModel, ObFitReport), String> {
    let mut segments = BTreeSet::new();
    for clause in clauses {
        if !segments.insert(clause.seg)
            || clause.tokens.is_empty()
            || clause.tokens.len() > OB_MAX_CLAUSE
            || clause
                .tokens
                .iter()
                .any(|token| *token as usize >= OB_MAX_VOCAB)
        {
            return Err("duplicate segment or invalid clause/token bound in fitting".into());
        }
        validate_alignment(clause)?;
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
        if !within(label.subject) || !within(label.marker) {
            return Err("declared span is outside its clause".into());
        }
        if let Some(object) = label.object {
            if !within(object) {
                return Err("declared object is outside its clause".into());
            }
        }
        let mut extents = vec![label.subject, label.marker];
        extents.extend(label.object);
        extents.sort_by_key(|span| span.0);
        if label.marker.1 > OB_MAX_MARKER
            || extents
                .windows(2)
                .any(|pair| pair[0].0 as usize + pair[0].1 > pair[1].0 as usize)
        {
            return Err("gold spans overlap or cue exceeds its feature bound".into());
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
        if label.object.is_some()
            && semantics
                .insert(label.role, label.action)
                .is_some_and(|prior| prior != label.action)
        {
            return Err("one observed role has conflicting supervised actions".into());
        }
    }
    let mut model = ObservedTextModel::uninformed();
    model.use_h4 = use_h4;
    if use_h4 {
        let vocab: BTreeSet<u32> = clauses
            .iter()
            .flat_map(|c| c.tokens.iter().copied())
            .collect();
        // Deterministic initial codes are installed before any weight is fitted. They carry no
        // presumed semantics; accepted coordinate moves must improve the same decoded objective.
        model.token_elements = vocab
            .into_iter()
            .map(|t| (t, (t as usize % super::group_table::GROUP_ORDER) as u8))
            .collect();
    }
    let span_initial_correct = exact_spans(&model, clauses, labels);
    let initial_correct = exact_hypotheses(&model, clauses, labels);
    // ---- one declared objective: joint spans + cue role ----
    let (mut updates, mut epochs_run) = fit_joint_model(&mut model, clauses, labels);
    // ---- ordered H4 interval elements: coherent alternating code/weight fit ----
    let mut h4_moves = 0usize;
    let mut h4_evaluations = 0usize;
    if use_h4 {
        let mut best = exact_hypotheses(&model, clauses, labels);
        let mut best_model = model.clone();
        for _round in 0..3 {
            let (moves, evaluations) = coordinate_search_codes(&mut model, clauses, labels);
            h4_moves += moves;
            h4_evaluations += evaluations;
            if moves == 0 {
                break;
            }
            let (u, e) = fit_joint_model(&mut model, clauses, labels);
            updates += u;
            epochs_run += e;
            let value = exact_hypotheses(&model, clauses, labels);
            if value > best {
                best = value;
                best_model = model.clone();
            } else {
                break;
            }
        }
        model = best_model;
    }
    let cue_hits = labels
        .iter()
        .filter(|label| {
            clauses
                .iter()
                .find(|c| c.seg == label.seg)
                .is_some_and(|clause| {
                    model.decode_segmentation(&clause.tokens).is_some_and(|s| {
                        s.cue == (label.marker.0 as usize, label.marker.1) && s.role == label.role
                    })
                })
        })
        .count();
    // ---- role semantics from the declared labels ----
    for (&role, &goal) in &role_goals {
        model.goal_per_role[role] = goal.index() as u8;
    }
    for (&role, &action) in &semantics {
        model.action_per_role[role] = action_tag(action);
        model.redirect_per_role[role] = action == RelAction::Continue;
    }
    model.validate(OB_MAX_VOCAB).map_err(|e| e.to_string())?;
    let span_final_correct = exact_spans(&model, clauses, labels);
    let final_correct = exact_hypotheses(&model, clauses, labels);
    let mut initial_action = 0usize;
    let mut policy_final_correct = 0usize;
    let mut examples = 0usize;
    for label in labels {
        if label.object.is_none() {
            continue;
        }
        examples += 1;
        let want = label.action;
        if want == RelAction::Emit {
            initial_action += 1;
        }
        if model.role_action(label.role) == want {
            policy_final_correct += 1;
        }
    }
    let report = ObFitReport {
        clauses: clauses.len(),
        segment_potentials: model.segment_weights.len(),
        role_weights: model.feature_weights.len(),
        candidate_updates: updates,
        epochs_run,
        span_initial_correct,
        span_final_correct,
        role_initial_correct: initial_correct,
        role_final_correct: final_correct,
        policy_initial_correct: initial_action,
        policy_final_correct,
        policy_examples: examples,
        roles_observed: role_goals.keys().copied().collect(),
        cue_role_correct: cue_hits,
        h4_enabled: use_h4,
        h4_moves,
        h4_evaluations,
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
            Clause::of(0, vec![10, 20, 21, 30]), // Mara works-in Cedar
            Clause::of(1, vec![10, 22, 23, 11]), // Mara office-follows Ivo
            Clause::of(2, vec![10, 24, 25, 31]), // Mara is-on-project Atlas
            Clause::of(3, vec![10, 26, 27, 12]), // Mara project-follows Nila
            Clause::of(4, vec![10, 20, 21]),     // Mara works-in ?
            Clause::of(5, vec![10, 24, 25]),     // Mara is-on-project ?
            Clause::of(6, vec![11, 20, 21, 32]), // Ivo works-in Fen
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
        let (model, report) = fit_observed_text_model(&clauses, &labels, false).unwrap();
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
        let (_, report) = fit_observed_text_model(&clauses, &labels, false).unwrap();
        // Baseline is computed on the new structured decoder, rather than borrowed from the old
        // marker extractor. An all-zero decoder can correctly place some unlabeled spans by tie.
        let zero = ObservedTextModel::uninformed();
        let initial = clauses
            .iter()
            .zip(&labels)
            .filter(|(clause, label)| {
                zero.decode_segmentation(&clause.tokens).is_some_and(|seg| {
                    seg.subject == (label.subject.0 as usize, label.subject.1)
                        && seg.cue == (label.marker.0 as usize, label.marker.1)
                        && seg.object == label.object.map(|(s, l)| (s as usize, l))
                })
            })
            .count();
        assert_eq!(report.span_initial_correct, initial);
        assert_eq!(report.span_final_correct, labels.len());
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
            let clause = Clause::of(1, vec![10, 20]);
            assert_eq!(loaded.candidate_scores(&clause.tokens, 1, 1), weights);
            assert_eq!(loaded.locate_marker(&clause.tokens), Some((1, 1, weights)));
            // The retained categorical role scorer still resolves the span's role on its own; the
            // decode now chooses that role jointly through the same weights.
            assert_eq!(loaded.cue_role(&clause.tokens, 1, 1), 1);
        }
        let mut model = ObservedTextModel::uninformed();
        model.feature_weights = vec![((F_FIRST << 40) | 20, [50_000, 50_000, 0, 0])];
        let clause = Clause::of(1, vec![10, 20, 20]);
        assert_eq!(
            model.locate_marker(&clause.tokens).map(|(s, l, _)| (s, l)),
            Some((1, 1))
        );
        assert_eq!(model.cue_role(&clause.tokens, 1, 1), 0);
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
    fn supervision_rejects_conflicts_overlaps_and_unrepresentable_cues() {
        let (clauses, labels) = dev();
        for mutation in 0..6 {
            let mut bad = labels.clone();
            match mutation {
                0 => bad[0].subject = (1, 1),
                1 => bad[0].object = Some((2, 1)),
                2 => bad[0].subject = (u32::MAX, 1),
                3 => bad[6].goal = Goal::Project,
                4 => bad[6].action = RelAction::Continue,
                _ => bad[0].action = RelAction::Read,
            }
            assert!(
                matches!(
                    fit_observed_text_model(&clauses, &bad, false),
                    Err(ObservedTextError::Supervision(_))
                ),
                "mutation {mutation}"
            );
        }
        // A valid in-clause span can still be outside the candidate admission bound.
        let clause = Clause::of(1, vec![10, 20, 21, 22, 23, 24, 30]);
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
            fit_observed_text_model(&[clause], &[label], false),
            Err(ObservedTextError::Supervision(_))
        ));
    }

    /// These tests qualify source ownership, continuation and emission, not length generalization
    /// from the one-token training fixture. Supply transparent boundary potentials for their known
    /// prefix/cue/suffix inputs; separate tests above exercise actual learned span/role fitting.
    fn runtime(clauses: Vec<Clause>, eos: Option<u32>) -> ObservedTextRuntime {
        let (development, labels) = dev();
        let (mut model, _) = fit_observed_text_model(&development, &labels, false).unwrap();
        let mut potentials = BTreeMap::<u64, [i32; OB_SEG_ROLES]>::new();
        potentials.entry(G_INITIAL << 40).or_default()[SEG_SUBJECT_LEFT] = 10;
        potentials.entry(G_FINAL << 40).or_default()[SEG_OBJECT_RIGHT] = 10;
        potentials.entry((G_LEN << 40) | 2).or_default()[SEG_CUE] = 2;
        for token in [20, 22, 24, 26] {
            {
                let row = potentials.entry((G_RIGHT << 40) | token).or_default();
                row[SEG_SUBJECT_LEFT] = 20;
                row[SEG_SUBJECT_RIGHT] = 20;
            }
            potentials.entry((G_FIRST << 40) | token).or_default()[SEG_CUE] = 20;
        }
        for token in [21, 23, 25, 27] {
            {
                let row = potentials.entry((G_LEFT << 40) | token).or_default();
                row[SEG_OBJECT_LEFT] = 20;
                row[SEG_OBJECT_RIGHT] = 20;
            }
            potentials.entry((G_LAST << 40) | token).or_default()[SEG_CUE] = 20;
        }
        model.segment_weights = potentials.into_iter().collect();
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
        Clause::of(999, tokens)
    }

    #[test]
    fn loaded_runtime_preserves_goal_and_exact_multiword_source() {
        let rt = runtime(
            vec![
                Clause::of(10, vec![10, 11, 22, 23, 12, 11]),
                Clause::of(11, vec![12, 11, 20, 21, 32, 33]),
                // Shared suffix is not an exact entity match.
                Clause::of(12, vec![13, 11, 20, 21, 34]),
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
            vec![Clause::of(9, vec![10, 22, 23, 10])],
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
            vec![Clause::of(9, vec![10, 20, 21, 30])],
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
            vec![Clause::of(10, vec![10, 20, 21, 30, 31, 32])],
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
        let clauses = vec![Clause::of(10, vec![10, 20, 21, 30, 31])];
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
            vec![Clause::of(10, vec![10, 20, 21, 32, 31])],
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
        let rt = runtime(vec![Clause::of(1, tokens)], Some(u32::MAX - 1));
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
            Clause::of(10, vec![10, 22, 23, 11]),
            Clause::of(11, vec![11, 20, 21, 32]),
            Clause::of(12, vec![12, 20, 21, 33]),
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
        let (model, _) = fit_observed_text_model(&clauses, &labels, false).unwrap();
        assert_eq!(
            ObservedTextModel::from_bytes(&model.to_bytes().unwrap(), 4096).unwrap(),
            model
        );
        assert!(matches!(
            ObservedTextModel::from_bytes(b"{", 4096),
            Err(ObservedTextError::Serialization(_))
        ));
        assert!(matches!(
            observe_clause(&model, &Clause::of(999, Vec::new())),
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
        assert!(fit_observed_text_model(&clauses, &bad_labels, false).is_err());
        bad_labels = labels.clone();
        bad_labels[0].role = OB_N_ROLES;
        assert!(matches!(
            fit_observed_text_model(&clauses, &bad_labels, false),
            Err(ObservedTextError::Supervision(_))
        ));
    }

    #[test]
    fn structured_decoder_admits_reordering_and_background_with_exact_scores() {
        let mut model = ObservedTextModel::uninformed();
        // Authored component test, not a learned-language result: object precedes subject and cue;
        // a trailing token receives the fixed zero background score.
        model.segment_weights = vec![
            ((G_FIRST << 40) | 10, [0, i32::MAX, i32::MAX, 0, 0, 0]),
            ((G_FIRST << 40) | 20, [0, 0, 0, i32::MAX, 0, 0]),
            ((G_FIRST << 40) | 30, [0, 0, 0, 0, i32::MAX, i32::MAX]),
            ((G_LEN << 40) | 1, [0, 1, 1, 1, 1, 1]),
        ];
        let loaded = ObservedTextModel::from_bytes(&model.to_bytes().unwrap(), 4096).unwrap();
        let decoded = loaded.decode_segmentation(&[30, 10, 20, 99]).unwrap();
        assert_eq!(decoded.subject, (1, 1));
        assert_eq!(decoded.cue, (2, 1));
        assert_eq!(decoded.object, Some((0, 1)));
        assert_eq!(decoded.score, 3 * (i64::from(i32::MAX) + 1));
        for (start, len) in [
            (usize::MAX, 1),
            (1, usize::MAX),
            (0, 0),
            (0, OB_MAX_SEG + 1),
        ] {
            assert!(model
                .segment_features(&[10, 20], start, len, &[])
                .is_empty());
        }
    }

    #[test]
    fn segment_model_loader_rejects_invalid_fields_and_learns_background() {
        for key in [
            G_LEN << 40,
            (G_LEN << 40) | 25,
            (G_FIRST << 40) | 4096,
            (G_INITIAL << 40) | 1,
            (G_FINAL << 40) | 1,
            (G_BIGRAM + 1) << 40,
            (G_H4_INTERVAL << 40) | 120,
            (G_POS << 40) | (OB_MAX_SEG as u64) << G_ORDER_SHIFT,
            (G_POS << 40) | 4096,
            (G_BIGRAM << 40) | (4096u64 << G_ORDER_SHIFT),
            (G_BIGRAM << 40) | 4096,
        ] {
            let mut bad = ObservedTextModel::uninformed();
            bad.segment_weights = vec![(key, [0; OB_SEG_ROLES])];
            assert!(ObservedTextModel::from_bytes(&bad.to_bytes().unwrap(), 4096).is_err());
        }
        // The background role is learned now, so a nonzero background weight is accepted.
        let mut ok = ObservedTextModel::uninformed();
        ok.segment_weights = vec![(G_INITIAL << 40, [1, 0, 0, 0, 0, 0])];
        assert!(ok.validate(4096).is_ok());
        let mut bad = ObservedTextModel::uninformed();
        bad.segment_weights = vec![(G_INITIAL << 40, [0; OB_SEG_ROLES]); 2];
        assert!(bad.validate(4096).is_err());
        for elements in [
            vec![(1, 120)],
            vec![(4096, 0)],
            vec![(1, 0), (1, 0)],
            vec![(2, 0), (1, 0)],
        ] {
            bad = ObservedTextModel::uninformed();
            bad.use_h4 = true;
            bad.token_elements = elements;
            assert!(bad.validate(4096).is_err());
        }
        bad = ObservedTextModel::uninformed();
        bad.token_elements = vec![(1, 0)];
        assert!(bad.validate(4096).is_err());
    }

    #[test]
    fn lexical_keys_are_exact_and_identity_regimes_are_disjoint() {
        assert_eq!(lexical_key(" Ivo", &[2, 2], 0, 2), Some(b"Ivo".to_vec()));
        assert_eq!(lexical_key("Ivo", &[3], 0, 1), Some(b"Ivo".to_vec()));
        assert_ne!(
            lexical_key("Ivo", &[3], 0, 1),
            lexical_key("ivo", &[3], 0, 1)
        );
        assert_ne!(
            lexical_key("Cedar Annex", &[5, 6], 0, 2),
            lexical_key("CedarAnnex", &[5, 5], 0, 2)
        );
        assert_eq!(lexical_key("Ivo", &[1, 1], 0, 1), None);
        assert_eq!(lexical_key("Ivo", &[3], usize::MAX, 2), None);
        assert_eq!(lexical_key("Ivo", &[3], 0, usize::MAX), None);
        let raw = Clause {
            seg: 0,
            tokens: vec![1],
            text: "abcd".into(),
            byte_lengths: vec![4],
        };
        let legacy = Clause::of(1, vec![u32::from_le_bytes(*b"abcd")]);
        assert_ne!(
            identity_key(&raw, 0, 1).unwrap(),
            identity_key(&legacy, 0, 1).unwrap()
        );
        let rt = runtime(vec![Clause::of(1, vec![10, 20, 21, 30])], None);
        let mut invalid = Clause {
            seg: 0,
            tokens: vec![10, 20, 21],
            text: "Mara works".into(),
            byte_lengths: vec![1, 1, 1],
        };
        assert!(rt.start(&invalid).is_err());
        invalid.byte_lengths.clear(); // explicitly legacy token-only is still supported
        assert!(rt.start(&invalid).is_ok());
    }

    #[test]
    fn byte_identity_joins_distinct_token_forms_and_restore_binds_capture_key() {
        // Token IDs deliberately differ across the first redirect object and the next subject.
        // This component test supplies verified extents; the real tokenizer adapter owns the
        // token-byte authenticity check and is exercised by the runner, not assumed here.
        let clauses = vec![
            Clause {
                seg: 1,
                tokens: vec![10, 22, 23, 11],
                text: "Mara office follows Ivo".into(),
                byte_lengths: vec![4, 7, 8, 4],
            },
            Clause {
                seg: 2,
                tokens: vec![12, 20, 21, 30],
                text: "Ivo works in Fen".into(),
                byte_lengths: vec![3, 6, 3, 4],
            },
        ];
        let rt = runtime(clauses, Some(u32::MAX - 1));
        let q = Clause {
            seg: 999,
            tokens: vec![13, 20, 21],
            text: "Mara works in".into(),
            byte_lengths: vec![4, 6, 3],
        };
        let mut session = rt.start(&q).unwrap();
        rt.step(&mut session).unwrap();
        assert_eq!(session.pending, RelAction::Continue);
        let mut forged = session.clone();
        forged.captured.as_mut().unwrap().key = [vec![1], b"Fen".to_vec()].concat();
        assert!(rt.restore(&serde_json::to_vec(&forged).unwrap()).is_err());
        loop {
            let bytes = rt.snapshot(&session).unwrap();
            let mut loaded = rt.restore(&bytes).unwrap();
            let mut same = session.clone();
            assert_eq!(rt.run(&mut loaded).unwrap(), rt.run(&mut same).unwrap());
            assert_eq!(loaded, same);
            if session.terminal.is_some() {
                break;
            }
            rt.step(&mut session).unwrap();
        }
        assert_eq!(session.reads, 2);
        assert_eq!(session.emitted, vec![30, u32::MAX - 1]);
    }

    #[test]
    fn ordered_interval_products_equal_direct_noncommutative_composition() {
        let t = super::super::group_table::group_table();
        let mut pair = None;
        for a in 0..super::super::group_table::GROUP_ORDER {
            for b in 0..super::super::group_table::GROUP_ORDER {
                if t.product[a * super::super::group_table::ROW_STRIDE + b]
                    != t.product[b * super::super::group_table::ROW_STRIDE + a]
                {
                    pair = Some((a, b));
                    break;
                }
            }
            if pair.is_some() {
                break;
            }
        }
        let (a, b) = pair.unwrap();
        let mut model = ObservedTextModel::uninformed();
        model.use_h4 = true;
        model.token_elements = vec![(10, a as u8), (20, b as u8)];
        let tokens = [10, 20, 10, 20];
        let prefix = model.prefix_products(&tokens);
        for start in 0..=tokens.len() {
            let mut direct = t.identity as usize;
            for end in start..=tokens.len() {
                assert_eq!(interval_product(&prefix, start, end), Some(direct));
                if end < tokens.len() {
                    direct = t.product[direct * super::super::group_table::ROW_STRIDE
                        + model.token_element(tokens[end])] as usize;
                }
            }
        }
        let reversed = model.prefix_products(&[20, 10]);
        assert_ne!(
            interval_product(&prefix, 0, 2),
            interval_product(&reversed, 0, 2)
        );
        assert_eq!(interval_product(&[], 0, 0), None);
        assert!(model.segment_features(&tokens, 0, 2, &[]).is_empty());
        assert_eq!(model.token_element(999), t.identity as usize);
    }

    /// Every legal hypothesis of the declared space, scored through the *same* precomputed
    /// potentials the decoder uses. Independent of the recurrence under test.
    fn all_legal(model: &ObservedTextModel, tokens: &[u32]) -> Vec<Segmentation> {
        let n = tokens.len();
        let prefix = model.prefix_products(tokens);
        let pot = model.interval_potentials(tokens, &prefix);
        let intervals: Vec<(usize, usize)> = (0..n)
            .flat_map(|a| ((a + 1)..=n).map(move |b| (a, b)))
            .collect();
        let disjoint = |a: (usize, usize), b: (usize, usize)| a.1 <= b.0 || b.1 <= a.0;
        // Uncovered tokens contribute their learned singleton background potentials.
        let background = |spans: &[(usize, usize)]| -> i64 {
            (0..n)
                .filter(|i| !spans.iter().any(|s| *i >= s.0 && *i < s.1))
                .map(|i| pot.seg(i, i + 1, SEG_BACKGROUND))
                .sum()
        };
        let mut out = Vec::new();
        for &sub in &intervals {
            for &cue in &intervals {
                if cue.1 - cue.0 > OB_MAX_MARKER || !disjoint(sub, cue) {
                    continue;
                }
                for r in 0..OB_N_ROLES {
                    let subject_role = if sub.0 >= cue.1 {
                        SEG_SUBJECT_RIGHT
                    } else {
                        SEG_SUBJECT_LEFT
                    };
                    let base = pot.seg(sub.0, sub.1, subject_role)
                        + pot.seg(cue.0, cue.1, SEG_CUE)
                        + pot.role(cue.0, cue.1, r)
                        + background(&[sub, cue]);
                    out.push(Segmentation {
                        subject: (sub.0, sub.1 - sub.0),
                        cue: (cue.0, cue.1 - cue.0),
                        role: r,
                        object: None,
                        score: base,
                    });
                    for &obj in &intervals {
                        if !disjoint(sub, obj) || !disjoint(cue, obj) {
                            continue;
                        }
                        let object_role = if obj.0 >= cue.1 {
                            SEG_OBJECT_RIGHT
                        } else {
                            SEG_OBJECT_LEFT
                        };
                        out.push(Segmentation {
                            subject: (sub.0, sub.1 - sub.0),
                            cue: (cue.0, cue.1 - cue.0),
                            role: r,
                            object: Some((obj.0, obj.1 - obj.0)),
                            score: base + pot.seg(obj.0, obj.1, object_role)
                                - background(&[sub, cue])
                                + background(&[sub, cue, obj]),
                        });
                    }
                }
            }
        }
        out
    }

    #[test]
    fn joint_decode_matches_exhaustive_enumeration_of_the_declared_space() {
        // Enumerate several tiny score landscapes with nonzero semantic-role and background
        // potentials. The oracle enumerates assignments, not the recurrence's states/transitions.
        for variant in 0..4i32 {
            let mut model = ObservedTextModel::uninformed();
            let mut weights = Vec::new();
            let mut role_weights = Vec::new();
            for i in 0..6u64 {
                let token = 10 + i;
                weights.push((
                    (G_FIRST << 40) | token,
                    [1 + i as i32 - variant, -2 * i as i32, i as i32, -1, 2, -2],
                ));
                weights.push((
                    (G_LAST << 40) | token,
                    [-1 - variant, 1 + i as i32, -1 - i as i32, 1, i as i32, -1],
                ));
                weights.push((
                    (G_POS << 40) | ((i % 3) << G_ORDER_SHIFT) | token,
                    [i as i32, 1, 1, 1, 1, 1],
                ));
                let mut roles = [0; OB_N_ROLES];
                roles[((i as usize) + variant as usize) % OB_N_ROLES] = 11 + i as i32;
                role_weights.push(((F_FIRST << 40) | token, roles));
            }
            weights.sort_by_key(|(k, _)| *k);
            model.segment_weights = weights;
            model.feature_weights = role_weights;
            for tokens in [&[10u32, 11, 12][..], &[14u32, 12, 10, 13, 11][..]] {
                let dp = model.decode_segmentation(tokens).unwrap();
                let all = all_legal(&model, tokens);
                let best = all.iter().map(|s| s.score).max().unwrap();
                assert_eq!(dp.score, best, "variant {variant}, {tokens:?}");
                assert!(all
                    .iter()
                    .any(|s| s.same_hypothesis(&dp) && s.score == dp.score));
            }
        }
        // Explicit tie policy and an i32-overflow witness for cue score accumulation.
        let mut zero = ObservedTextModel::uninformed();
        let tied = zero.decode_segmentation(&[10, 11, 12]).unwrap();
        assert_eq!(tied.object, None);
        assert_eq!(tied.role, 0);
        zero.feature_weights = vec![
            ((F_FIRST << 40) | 11, [0, i32::MAX, i32::MAX, 0]),
            ((F_LAST << 40) | 11, [0, 1, 2, 0]),
        ];
        let decoded = zero.decode_segmentation(&[10, 11]).unwrap();
        assert_eq!(decoded.role, 2);
        assert_eq!(decoded.score, i64::from(i32::MAX) + 2);
    }

    #[test]
    fn joint_update_equals_the_declared_scored_feature_difference() {
        let mut model = ObservedTextModel::uninformed();
        let clause = Clause::of(0, vec![10, 11, 12, 13, 14, 15]);
        let gold = Segmentation {
            subject: (0, 1),
            cue: (2, 1),
            role: 2,
            object: Some((4, 1)),
            score: 0,
        };
        let pred = Segmentation {
            subject: (4, 1),
            cue: (2, 1),
            role: 1,
            object: None,
            score: 0,
        };
        let prefix = model.prefix_products(&clause.tokens);
        // Independently collect the declared feature multiset of each complete assignment. The
        // shared extractor is intentional; scoring/update multiplicity and role attribution are
        // what this check proves, including both cue sides and uncovered singleton tokens.
        let features = |seg: &Segmentation| {
            let mut counts = BTreeMap::<(bool, u64, usize), i32>::new();
            let mut spans = vec![
                (
                    seg.subject,
                    if seg.subject.0 < seg.cue.0 {
                        SEG_SUBJECT_LEFT
                    } else {
                        SEG_SUBJECT_RIGHT
                    },
                ),
                (seg.cue, SEG_CUE),
            ];
            if let Some(obj) = seg.object {
                spans.push((
                    obj,
                    if obj.0 < seg.cue.0 {
                        SEG_OBJECT_LEFT
                    } else {
                        SEG_OBJECT_RIGHT
                    },
                ));
            }
            for (span, role) in &spans {
                for feature in model.segment_features(&clause.tokens, span.0, span.1, &prefix) {
                    *counts.entry((false, feature, *role)).or_default() += 1;
                }
            }
            for i in 0..clause.tokens.len() {
                if spans
                    .iter()
                    .any(|(span, _)| i >= span.0 && i < span.0 + span.1)
                {
                    continue;
                }
                for feature in model.segment_features(&clause.tokens, i, 1, &prefix) {
                    *counts.entry((false, feature, SEG_BACKGROUND)).or_default() += 1;
                }
            }
            for feature in candidate_features(&clause.tokens, seg.cue.0, seg.cue.1) {
                *counts.entry((true, feature, seg.role)).or_default() += 1;
            }
            counts
        };
        let mut expected = features(&gold);
        for (key, value) in features(&pred) {
            *expected.entry(key).or_default() -= value;
        }
        let norm_squared: i64 = expected.values().map(|v| i64::from(*v).pow(2)).sum();
        apply_segmentation(&mut model, &clause, &gold, &prefix, 1);
        apply_segmentation(&mut model, &clause, &pred, &prefix, -1);
        for ((is_role, key, role), value) in &expected {
            let actual = if *is_role {
                model
                    .feature_weights
                    .iter()
                    .find(|(k, _)| k == key)
                    .unwrap()
                    .1[*role]
            } else {
                model
                    .segment_weights
                    .iter()
                    .find(|(k, _)| k == key)
                    .unwrap()
                    .1[*role]
            };
            assert_eq!(actual, *value);
        }
        let assignments = all_legal(&model, &clause.tokens);
        let gold_score = assignments
            .iter()
            .find(|x| x.same_hypothesis(&gold))
            .unwrap()
            .score;
        let pred_score = assignments
            .iter()
            .find(|x| x.same_hypothesis(&pred))
            .unwrap()
            .score;
        assert_eq!(gold_score - pred_score, norm_squared);
    }

    #[test]
    fn ordered_feature_token_domain_is_explicit_at_every_boundary() {
        let model = ObservedTextModel::uninformed();
        assert!(model.validate(OB_MAX_VOCAB).is_ok());
        assert!(model.validate(OB_MAX_VOCAB + 1).is_err());
        let bad = [0, OB_MAX_VOCAB as u32, 2];
        assert!(candidate_features(&bad, 1, 1).is_empty());
        assert!(model.segment_features(&bad, 0, 3, &[]).is_empty());
        assert!(model.decode_segmentation(&bad).is_none());
        assert!(observe_clause(&model, &Clause::of(0, bad.to_vec())).is_err());
        let label = ClauseLabel {
            seg: 0,
            subject: (0, 1),
            marker: (1, 1),
            object: Some((2, 1)),
            role: 0,
            goal: Goal::Office,
            action: RelAction::Emit,
        };
        assert!(fit_observed_text_model(&[Clause::of(0, bad.to_vec())], &[label], false).is_err());
        // This pair used to alias via the upper token bits entering the adjacent packed field.
        assert_eq!((1u64 << G_ORDER_SHIFT) | 65536, (1u64 << G_ORDER_SHIFT) | 0);
        let valid = [u16::MAX as u32, 0];
        assert!(!model.segment_features(&valid, 0, 2, &[]).is_empty());
        let mut malformed = model.clone();
        malformed.segment_weights = vec![(
            (G_BIGRAM << 40) | ((OB_MAX_VOCAB as u64) << G_ORDER_SHIFT),
            [0; OB_SEG_ROLES],
        )];
        assert!(malformed.validate(OB_MAX_VOCAB).is_err());
        let rt = runtime(vec![Clause::of(1, vec![10, 20, 21, 30])], None);
        assert!(rt.start(&Clause::of(999, bad.to_vec())).is_err());
    }

    #[test]
    fn ordered_segment_features_distinguish_interior_permutations() {
        let model = ObservedTextModel::uninformed();
        let fwd = model.segment_features(&[10, 20, 30, 40], 1, 2, &[]);
        let rev = model.segment_features(&[10, 30, 20, 40], 1, 2, &[]);
        assert_ne!(fwd, rev);
        // A four-token interior permutation that endpoint-only features cannot see.
        let p = model.segment_features(&[10, 20, 30, 40, 50, 60], 1, 4, &[]);
        let q = model.segment_features(&[10, 20, 40, 30, 50, 60], 1, 4, &[]);
        assert_ne!(p, q);
        assert!(p.contains(&((G_BIGRAM << 40) | (20 << G_ORDER_SHIFT) | 30)));
        assert!(p.contains(&((G_POS << 40) | 20)));
        assert!(!p.contains(&((G_BIGRAM << 40) | (20 << G_ORDER_SHIFT) | 40)));
    }

    #[test]
    fn lexical_membership_matches_identity_but_does_not_replace_role_semantics() {
        let clauses = vec![
            Clause {
                seg: 1,
                tokens: vec![10, 22, 23, 11],
                text: "Mara office follows Ivo".into(),
                byte_lengths: vec![4, 7, 8, 4],
            },
            Clause {
                seg: 2,
                tokens: vec![12, 20, 21, 30],
                text: "Ivo works in Fen".into(),
                byte_lengths: vec![3, 6, 3, 4],
            },
        ];
        let q = Clause {
            seg: 999,
            tokens: vec![13, 20, 21],
            text: "Mara works in".into(),
            byte_lengths: vec![4, 6, 3],
        };
        let primary = runtime(clauses.clone(), None);
        let old = runtime(clauses.clone(), None)
            .with_control(TextControl::Membership {
                entities: vec![vec![12]],
            })
            .unwrap();
        let matched = runtime(clauses.clone(), None)
            .with_control(TextControl::LexicalMembership {
                entities: vec![b"Ivo".to_vec()],
            })
            .unwrap();
        let mut old_state = old.start(&q).unwrap();
        old.run(&mut old_state).unwrap();
        assert_eq!(old_state.emitted, vec![11]);
        assert_eq!(old_state.reads, 1); // different token form defeats this historical control
        let mut state = matched.start(&q).unwrap();
        assert!(primary.restore(&matched.snapshot(&state).unwrap()).is_err());
        loop {
            let saved = matched.snapshot(&state).unwrap();
            let mut resumed = matched.restore(&saved).unwrap();
            let mut direct = state.clone();
            assert_eq!(
                matched.run(&mut resumed).unwrap(),
                matched.run(&mut direct).unwrap()
            );
            assert_eq!(resumed, direct);
            if state.terminal.is_some() {
                break;
            }
            matched.step(&mut state).unwrap();
        }
        assert_eq!(state.emitted, vec![30]);
        assert_eq!(state.reads, 2);
        let mut normal = primary.start(&q).unwrap();
        primary.run(&mut normal).unwrap();
        assert_eq!(normal.emitted, state.emitted);
        assert_eq!(normal.reads, state.reads);

        // Same lexical value Ivo, but now the first record is an assertion. A matched membership
        // policy still follows it incorrectly: identity normalization cannot learn Emit/Continue.
        let mut terminal_world = clauses.clone();
        terminal_world[0] = Clause {
            seg: 1,
            tokens: vec![10, 20, 21, 11],
            text: "Mara works in Ivo".into(),
            byte_lengths: vec![4, 6, 3, 4],
        };
        let terminal_primary = runtime(terminal_world.clone(), None);
        let terminal_control = runtime(terminal_world, None)
            .with_control(TextControl::LexicalMembership {
                entities: vec![b"Ivo".to_vec()],
            })
            .unwrap();
        let mut correct = terminal_primary.start(&q).unwrap();
        terminal_primary.run(&mut correct).unwrap();
        assert_eq!(correct.emitted, vec![11]);
        assert_eq!(correct.reads, 1);
        let mut overfollowed = terminal_control.start(&q).unwrap();
        terminal_control.run(&mut overfollowed).unwrap();
        assert_eq!(overfollowed.emitted, vec![30]);
        assert_eq!(overfollowed.reads, 2);
        for invalid_key in [Vec::new(), b" Ivo".to_vec(), b"Ivo ".to_vec()] {
            assert!(runtime(clauses.clone(), None)
                .with_control(TextControl::LexicalMembership {
                    entities: vec![invalid_key]
                })
                .is_err());
        }
        assert!(runtime(vec![Clause::of(1, vec![10, 20, 21, 30])], None)
            .with_control(TextControl::LexicalMembership {
                entities: Vec::new()
            })
            .is_err());
    }
}
