//! Object-store consumer for M3 durable conversation memory (`UORMEM01` + `UR4SESS1`).
//!
//! Ports the scoped durable-session contract (`IdentityScope`, `export_scope`,
//! `import_scope`, `reset_scope`, `retract_fact`) onto [`Store`], [`Manifest`],
//! and [`Bindings`]:
//! - Every asserted or revised fact ingests its source turn through a
//!   [`PinnedTokenizer`] and stores verified byte spans and token-occurrence
//!   indices in [`Store`].
//! - Reading an active fact verifies the underlying records and token bindings
//!   via [`Store::range_from`] and [`Bindings::read_occurrence`].
//! - Unstated facts return [`FactReadOutcome::RefusedUnstated`] with zero
//!   backend fetches; retracted facts return
//!   [`FactReadOutcome::RefusedRetracted`] rather than guessing.
//! - Non-memory prose turns fold into [`Store`] without mutating or overwriting
//!   any fact slot (`#1718`).
//! - `export_scope` folds the `UORMEM01` binary envelope (including the embedded
//!   `UR4SESS1` session snapshot bytes) into [`Store`] and returns a pinned
//!   [`Manifest`]; `import_scope` verifies the whole-object SHA-256 via
//!   [`Store::rebuild`] and restores the scope byte-for-byte.
//! - Pure integer execution: no floating-point types or operations, no
//!   exponential or squashing functions, and no dense matrix products.

use crate::{
    bindings::{ingest, Bindings, PinnedTokenizer},
    reserve, Error, Limits, Manifest, ObjectVersion, RecordBackend, Result, Store,
};
use std::{collections::BTreeMap, ops::Range};

/// Binary magic for a scoped durable-memory envelope stored in [`Store`].
pub const DURABLE_CONSUMER_MAGIC: &[u8; 8] = b"UORMEM01";

/// Format version of the `UORMEM01` binary envelope.
pub const DURABLE_CONSUMER_VERSION: u32 = 1;

/// Binary magic of the integer stack session codec (`crates/uor-r4-integer/src/session_codec.rs`).
pub const UR4SESS1_MAGIC: &[u8; 8] = b"UR4SESS1";

/// Maximum identifier byte length for each component of [`IdentityScope`].
const MAX_SCOPE_FIELD_BYTES: usize = 256;

/// Maximum byte length of a fact owner or relation key.
const MAX_FACT_KEY_BYTES: usize = 256;

/// Isolated identity scope partitioning persistent conversational memory.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IdentityScope {
    pub user_id: String,
    pub project_id: String,
    pub session_id: String,
}

impl IdentityScope {
    /// Construct and validate an [`IdentityScope`]. Rejects empty or oversized fields.
    pub fn new(
        user_id: impl Into<String>,
        project_id: impl Into<String>,
        session_id: impl Into<String>,
    ) -> Result<Self> {
        let user = user_id.into().trim().to_string();
        let project = project_id.into().trim().to_string();
        let session = session_id.into().trim().to_string();
        if user.is_empty()
            || project.is_empty()
            || session.is_empty()
            || user.len() > MAX_SCOPE_FIELD_BYTES
            || project.len() > MAX_SCOPE_FIELD_BYTES
            || session.len() > MAX_SCOPE_FIELD_BYTES
        {
            return Err(Error::SchemaMismatch);
        }
        Ok(Self {
            user_id: user,
            project_id: project,
            session_id: session,
        })
    }

    /// Canonical slash-delimited key for diagnostics.
    pub fn key(&self) -> String {
        format!("{}/{}/{}", self.user_id, self.project_id, self.session_id)
    }
}

/// Versioned mutation action recorded on a fact slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FactAction {
    /// Initial assertion or unrevised overwrite (`Action 1`).
    Assert = 1,
    /// Explicit revision superseding the prior value (`Action 2`).
    Revise = 2,
    /// Explicit retraction erasing the active value (`Action 3`).
    Retract = 3,
}

impl FactAction {
    fn from_u8(v: u8) -> Result<Self> {
        match v {
            1 => Ok(Self::Assert),
            2 => Ok(Self::Revise),
            3 => Ok(Self::Retract),
            _ => Err(Error::SchemaMismatch),
        }
    }
}

/// Content-addressed provenance tying a stored fact value to its source turn
/// [`Manifest`], token [`Bindings`], byte span, and covered token occurrences.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FactProvenance {
    pub manifest: Manifest,
    pub bindings: Bindings,
    pub value_range: Range<u64>,
    pub occurrence_indices: Vec<u64>,
}

/// Versioned state of a single `(owner, relation)` slot inside a [`IdentityScope`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FactVersionRecord {
    pub owner: String,
    pub relation: String,
    pub version: u64,
    pub action: FactAction,
    pub conflict: bool,
    pub current: Option<FactProvenance>,
    pub previous: Option<FactProvenance>,
}

/// Outcome of reading a fact from [`DurableObjectMemoryStore`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FactReadOutcome {
    /// Active fact verified against [`Store`] and [`Bindings`].
    Quoted {
        owner: String,
        relation: String,
        value_bytes: Vec<u8>,
        token_ids: Vec<u32>,
        version: u64,
        action: FactAction,
        conflict: bool,
        object_version: ObjectVersion,
        previous_value_bytes: Option<Vec<u8>>,
    },
    /// Fact was never stated in this scope; zero backend reads performed.
    RefusedUnstated,
    /// Fact was explicitly retracted in this scope; zero backend reads performed.
    RefusedRetracted {
        owner: String,
        relation: String,
        retracted_at_version: u64,
    },
}

/// Validated structural view of a `UR4SESS1` binary session snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ur4Sess1HeaderView {
    pub schema: String,
    pub version: u32,
    pub artifact_sha256: String,
    pub position: u64,
    pub cache_at: u64,
    pub lift_at: u64,
    pub copy_scale_q16: i32,
    pub logits: Vec<i32>,
    pub tokens: Vec<u32>,
    pub layer_count: usize,
}

impl Ur4Sess1HeaderView {
    /// Validate a complete `UR4SESS1` byte stream without floating point or division.
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let mut r = SliceReader::new(bytes);
        if r.take(8)? != UR4SESS1_MAGIC {
            return Err(Error::SchemaMismatch);
        }
        let schema = r.string(256)?;
        let version = r.u32()?;
        let artifact_sha256 = r.string(256)?;
        let position = r.u64()?;
        let cache_at = r.u64()?;
        let lift_at = r.u64()?;
        let copy_scale_q16 = r.i32()?;
        let logits = r.i32_vec()?;
        let tokens = r.u32_vec()?;
        match r.u8()? {
            0 => {}
            1 => {
                let _snap = r.u32_vec()?;
            }
            _ => return Err(Error::SchemaMismatch),
        }
        let _pointer_keys = r.i32_vec()?;
        let _pointer_lifts = r.u64_vec()?;
        let _mixture = r.i32_vec()?;
        let layer_count = r.checked_count(0)?;
        for _ in 0..layer_count {
            match r.u8()? {
                0 => {
                    let _state = r.i64_vec()?;
                    let _history = r.i32_vec()?;
                }
                1 => {
                    let _keys = r.i32_vec()?;
                    let _values = r.i32_vec()?;
                    let _lifts = r.u64_vec()?;
                }
                _ => return Err(Error::SchemaMismatch),
            }
        }
        if !r.is_eof() {
            return Err(Error::SchemaMismatch);
        }
        Ok(Self {
            schema,
            version,
            artifact_sha256,
            position,
            cache_at,
            lift_at,
            copy_scale_q16,
            logits,
            tokens,
            layer_count,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScopedMemoryState {
    scope: IdentityScope,
    last_entity: Option<String>,
    ur4sess1_bytes: Option<Vec<u8>>,
    facts: BTreeMap<(String, String), FactVersionRecord>,
    non_memory_turns: Vec<(Manifest, Bindings)>,
}

impl ScopedMemoryState {
    fn new(scope: IdentityScope) -> Self {
        Self {
            scope,
            last_entity: None,
            ur4sess1_bytes: None,
            facts: BTreeMap::new(),
            non_memory_turns: Vec::new(),
        }
    }

    fn resolve_pronoun<'a>(&'a self, owner_or_pronoun: &'a str) -> &'a str {
        if matches!(
            owner_or_pronoun.to_ascii_lowercase().as_str(),
            "she" | "he" | "it" | "they" | "this" | "that"
        ) {
            if let Some(ref entity) = self.last_entity {
                return entity.as_str();
            }
        }
        owner_or_pronoun
    }
}

/// Multi-tenant durable conversation memory store backed by [`Store`].
#[derive(Debug, Clone, Default)]
pub struct DurableObjectMemoryStore {
    scopes: BTreeMap<IdentityScope, ScopedMemoryState>,
}

impl DurableObjectMemoryStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Return all active [`IdentityScope`] keys.
    pub fn scopes(&self) -> Vec<IdentityScope> {
        self.scopes.keys().cloned().collect()
    }

    /// Count of non-memory prose turns ingested under `scope`.
    pub fn non_memory_turn_count(&self, scope: &IdentityScope) -> usize {
        self.scopes
            .get(scope)
            .map_or(0, |s| s.non_memory_turns.len())
    }

    /// Attach or replace the validated `UR4SESS1` binary snapshot for `scope`.
    pub fn set_ur4sess1_snapshot(&mut self, scope: &IdentityScope, bytes: &[u8]) -> Result<()> {
        Ur4Sess1HeaderView::parse(bytes)?;
        let state = self
            .scopes
            .entry(scope.clone())
            .or_insert_with(|| ScopedMemoryState::new(scope.clone()));
        state.ur4sess1_bytes = Some(bytes.to_vec());
        Ok(())
    }

    /// Borrow the active `UR4SESS1` binary snapshot for `scope`, if present.
    pub fn ur4sess1_snapshot(&self, scope: &IdentityScope) -> Option<&[u8]> {
        self.scopes
            .get(scope)
            .and_then(|s| s.ur4sess1_bytes.as_deref())
    }

    /// Assert a fact (`FactAction::Assert`) for `(owner, relation)` from `turn_bytes[value_range]`.
    pub fn assert_fact(
        &mut self,
        store: &mut Store,
        tokenizer: &PinnedTokenizer,
        scope: &IdentityScope,
        owner: &str,
        relation: &str,
        turn_bytes: &[u8],
        value_range: Range<u64>,
    ) -> Result<u64> {
        self.commit_fact(
            store,
            tokenizer,
            scope,
            owner,
            relation,
            turn_bytes,
            value_range,
            FactAction::Assert,
        )
    }

    /// Explicitly revise a fact (`FactAction::Revise`) for `(owner, relation)`.
    pub fn revise_fact(
        &mut self,
        store: &mut Store,
        tokenizer: &PinnedTokenizer,
        scope: &IdentityScope,
        owner: &str,
        relation: &str,
        turn_bytes: &[u8],
        value_range: Range<u64>,
    ) -> Result<u64> {
        self.commit_fact(
            store,
            tokenizer,
            scope,
            owner,
            relation,
            turn_bytes,
            value_range,
            FactAction::Revise,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn commit_fact(
        &mut self,
        store: &mut Store,
        tokenizer: &PinnedTokenizer,
        scope: &IdentityScope,
        owner: &str,
        relation: &str,
        turn_bytes: &[u8],
        value_range: Range<u64>,
        action: FactAction,
    ) -> Result<u64> {
        validate_fact_key(owner)?;
        validate_fact_key(relation)?;
        if value_range.start >= value_range.end || value_range.end > turn_bytes.len() as u64 {
            return Err(Error::InvalidRange);
        }
        let ingested = ingest(store, tokenizer, turn_bytes)?;
        let mut occurrence_indices = Vec::new();
        for occ in ingested.bindings.occurrences() {
            let r = occ.byte_range();
            if r.end > value_range.start && r.start < value_range.end {
                if r.start < value_range.start || r.end > value_range.end {
                    return Err(Error::BindingMismatch);
                }
                occurrence_indices.push(occ.index());
            }
        }
        if occurrence_indices.is_empty() {
            return Err(Error::BindingMismatch);
        }
        let provenance = FactProvenance {
            manifest: ingested.manifest,
            bindings: ingested.bindings,
            value_range: value_range.clone(),
            occurrence_indices,
        };
        let new_slice = &turn_bytes[value_range.start as usize..value_range.end as usize];

        let state = self
            .scopes
            .entry(scope.clone())
            .or_insert_with(|| ScopedMemoryState::new(scope.clone()));
        let key = (owner.to_string(), relation.to_string());

        let next_version = match state.facts.get_mut(&key) {
            None => {
                state.facts.insert(
                    key,
                    FactVersionRecord {
                        owner: owner.to_string(),
                        relation: relation.to_string(),
                        version: 1,
                        action,
                        conflict: false,
                        current: Some(provenance),
                        previous: None,
                    },
                );
                1
            }
            Some(existing) => {
                let same_bytes = if let Some(ref curr) = existing.current {
                    store
                        .range(&curr.manifest, curr.value_range.clone())
                        .map(|v| v.bytes() == new_slice)
                        .unwrap_or(false)
                } else {
                    false
                };
                if same_bytes && existing.action != FactAction::Retract {
                    if action == FactAction::Revise {
                        existing.conflict = false;
                        existing.action = FactAction::Revise;
                    }
                    existing.version
                } else {
                    let v = existing
                        .version
                        .checked_add(1)
                        .ok_or(Error::LimitExceeded("fact version"))?;
                    let conflict = action == FactAction::Assert && existing.current.is_some();
                    existing.previous = existing.current.take();
                    existing.current = Some(provenance);
                    existing.version = v;
                    existing.action = action;
                    existing.conflict = conflict;
                    v
                }
            }
        };
        state.last_entity = Some(owner.to_string());
        Ok(next_version)
    }

    /// Explicitly retract (`FactAction::Retract`) a stored fact in `scope`.
    pub fn retract_fact(
        &mut self,
        scope: &IdentityScope,
        owner: &str,
        relation: &str,
    ) -> Result<u64> {
        validate_fact_key(owner)?;
        validate_fact_key(relation)?;
        let state = self.scopes.get_mut(scope).ok_or(Error::SchemaMismatch)?;
        let resolved = state.resolve_pronoun(owner).to_string();
        let key = (resolved.clone(), relation.to_string());
        let record = state.facts.get_mut(&key).ok_or(Error::SchemaMismatch)?;
        let v = record
            .version
            .checked_add(1)
            .ok_or(Error::LimitExceeded("fact version"))?;
        record.previous = record.current.take();
        record.version = v;
        record.action = FactAction::Retract;
        record.conflict = false;
        if state.last_entity.as_deref() == Some(resolved.as_str()) {
            state.last_entity = None;
        }
        Ok(v)
    }

    /// Ingest a non-memory prose turn into [`Store`] with [`Bindings`] without
    /// modifying or overwriting any stored fact slot (`#1718`).
    pub fn ingest_non_memory_prose(
        &mut self,
        store: &mut Store,
        tokenizer: &PinnedTokenizer,
        scope: &IdentityScope,
        turn_bytes: &[u8],
    ) -> Result<Manifest> {
        let ingested = ingest(store, tokenizer, turn_bytes)?;
        let state = self
            .scopes
            .entry(scope.clone())
            .or_insert_with(|| ScopedMemoryState::new(scope.clone()));
        state
            .non_memory_turns
            .push((ingested.manifest.clone(), ingested.bindings));
        Ok(ingested.manifest)
    }

    /// Read and verify a fact from `store` for `scope`.
    pub fn read_fact(
        &self,
        store: &Store,
        tokenizer: &PinnedTokenizer,
        scope: &IdentityScope,
        owner_or_pronoun: &str,
        relation: &str,
    ) -> Result<FactReadOutcome> {
        self.read_fact_from(
            store,
            store.limits(),
            tokenizer,
            scope,
            owner_or_pronoun,
            relation,
        )
    }

    /// Read and verify a fact from any [`RecordBackend`].
    /// Performs zero backend fetches when the fact is unstated or retracted.
    pub fn read_fact_from<B: RecordBackend>(
        &self,
        backend: &B,
        limits: &Limits,
        tokenizer: &PinnedTokenizer,
        scope: &IdentityScope,
        owner_or_pronoun: &str,
        relation: &str,
    ) -> Result<FactReadOutcome> {
        validate_fact_key(owner_or_pronoun)?;
        validate_fact_key(relation)?;
        let Some(state) = self.scopes.get(scope) else {
            return Ok(FactReadOutcome::RefusedUnstated);
        };
        let resolved = state.resolve_pronoun(owner_or_pronoun);
        let key = (resolved.to_string(), relation.to_string());
        let Some(record) = state.facts.get(&key) else {
            return Ok(FactReadOutcome::RefusedUnstated);
        };
        if record.action == FactAction::Retract || record.current.is_none() {
            return Ok(FactReadOutcome::RefusedRetracted {
                owner: record.owner.clone(),
                relation: record.relation.clone(),
                retracted_at_version: record.version,
            });
        }
        let curr = record.current.as_ref().ok_or(Error::SchemaMismatch)?;
        let (value_bytes, token_ids, object_version) =
            verify_provenance(backend, limits, tokenizer, curr)?;
        let previous_value_bytes = match &record.previous {
            Some(prev) => {
                let (prev_bytes, _, _) = verify_provenance(backend, limits, tokenizer, prev)?;
                Some(prev_bytes)
            }
            None => None,
        };
        Ok(FactReadOutcome::Quoted {
            owner: record.owner.clone(),
            relation: record.relation.clone(),
            value_bytes,
            token_ids,
            version: record.version,
            action: record.action,
            conflict: record.conflict,
            object_version,
            previous_value_bytes,
        })
    }

    /// Explicitly reset `scope`, clearing all facts, non-memory turns, pronoun
    /// antecedent, and `UR4SESS1` snapshot bytes while preserving other scopes.
    pub fn reset_scope(&mut self, scope: &IdentityScope) -> Result<()> {
        let state = self.scopes.get_mut(scope).ok_or(Error::SchemaMismatch)?;
        *state = ScopedMemoryState::new(scope.clone());
        Ok(())
    }

    /// Fold the canonical `UORMEM01` binary envelope for `scope` into `store`
    /// and return its content-addressed [`Manifest`].
    pub fn export_scope(&self, store: &mut Store, scope: &IdentityScope) -> Result<Manifest> {
        let state = self.scopes.get(scope).ok_or(Error::SchemaMismatch)?;
        let bytes = encode_scope_state(state, store.limits())?;
        Ok(store.fold(&bytes)?.manifest)
    }

    /// Rebuild and verify a `UORMEM01` envelope from `store` and restore its
    /// [`IdentityScope`] state into `self`.
    pub fn import_scope(&mut self, store: &Store, manifest: &Manifest) -> Result<IdentityScope> {
        let verified = store.rebuild(manifest)?;
        let state = decode_scope_state(verified.bytes(), store.limits())?;
        let scope = state.scope.clone();
        self.scopes.insert(scope.clone(), state);
        Ok(scope)
    }

    /// Rebuild, verify, and restore a `UORMEM01` envelope only if its embedded
    /// [`IdentityScope`] matches `expected_scope`.
    pub fn import_scope_expecting(
        &mut self,
        store: &Store,
        manifest: &Manifest,
        expected_scope: &IdentityScope,
    ) -> Result<IdentityScope> {
        let verified = store.rebuild(manifest)?;
        let state = decode_scope_state(verified.bytes(), store.limits())?;
        if &state.scope != expected_scope {
            return Err(Error::BindingMismatch);
        }
        let scope = state.scope.clone();
        self.scopes.insert(scope.clone(), state);
        Ok(scope)
    }
}

fn validate_fact_key(s: &str) -> Result<()> {
    if s.is_empty() || s.len() > MAX_FACT_KEY_BYTES {
        return Err(Error::SchemaMismatch);
    }
    Ok(())
}

fn verify_provenance<B: RecordBackend>(
    backend: &B,
    limits: &Limits,
    tokenizer: &PinnedTokenizer,
    prov: &FactProvenance,
) -> Result<(Vec<u8>, Vec<u32>, ObjectVersion)> {
    if prov.bindings.object_version() != prov.manifest.version()
        || prov.bindings.tokenizer_identity() != tokenizer.identity()
    {
        return Err(Error::BindingMismatch);
    }
    let span_verified =
        Store::range_from(backend, &prov.manifest, prov.value_range.clone(), limits)?;
    let mut assembled = Vec::new();
    let mut token_ids = Vec::new();
    reserve(&mut token_ids, prov.occurrence_indices.len())?;
    for &occ_idx in &prov.occurrence_indices {
        let idx = usize::try_from(occ_idx).map_err(|_| Error::InvalidRange)?;
        let occ = prov
            .bindings
            .occurrences()
            .get(idx)
            .ok_or(Error::InvalidRange)?;
        let tok_bytes = tokenizer.token_bytes(occ.token_id(), limits.max_returned_bytes)?;
        let occ_verified = Store::range_from(backend, &prov.manifest, occ.byte_range(), limits)?;
        if occ_verified.bytes() != tok_bytes.as_slice() {
            return Err(Error::BindingMismatch);
        }
        assembled.extend_from_slice(occ_verified.bytes());
        token_ids.push(occ.token_id());
    }
    if assembled != span_verified.bytes() {
        return Err(Error::BindingMismatch);
    }
    Ok((
        span_verified.bytes().to_vec(),
        token_ids,
        span_verified.version(),
    ))
}

fn encode_provenance(out: &mut Vec<u8>, prov: &FactProvenance, limits: &Limits) -> Result<()> {
    let manifest_bytes = prov.manifest.encode();
    let bindings_bytes = prov.bindings.encode(limits)?;
    put_bytes(out, &manifest_bytes)?;
    put_bytes(out, &bindings_bytes)?;
    out.extend_from_slice(&prov.value_range.start.to_le_bytes());
    out.extend_from_slice(&prov.value_range.end.to_le_bytes());
    let count = u32::try_from(prov.occurrence_indices.len()).map_err(|_| Error::SchemaMismatch)?;
    out.extend_from_slice(&count.to_le_bytes());
    for &idx in &prov.occurrence_indices {
        out.extend_from_slice(&idx.to_le_bytes());
    }
    Ok(())
}

fn decode_provenance(r: &mut SliceReader<'_>, limits: &Limits) -> Result<FactProvenance> {
    let manifest_bytes = r.bytes(limits.max_record_bytes)?;
    let manifest = Manifest::decode(manifest_bytes, limits)?;
    let bindings_bytes = r.bytes(limits.max_record_bytes)?;
    let bindings = Bindings::decode(bindings_bytes, limits)?;
    if bindings.object_version() != manifest.version() {
        return Err(Error::BindingMismatch);
    }
    let start = r.u64()?;
    let end = r.u64()?;
    if start >= end || end > manifest.byte_len() {
        return Err(Error::InvalidRange);
    }
    let count = r.checked_u32_count(3)?;
    if count == 0 || count > limits.max_occurrences {
        return Err(Error::SchemaMismatch);
    }
    let mut occurrence_indices = Vec::new();
    reserve(&mut occurrence_indices, count)?;
    for _ in 0..count {
        occurrence_indices.push(r.u64()?);
    }
    Ok(FactProvenance {
        manifest,
        bindings,
        value_range: start..end,
        occurrence_indices,
    })
}

fn encode_scope_state(state: &ScopedMemoryState, limits: &Limits) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    out.extend_from_slice(DURABLE_CONSUMER_MAGIC);
    out.extend_from_slice(&DURABLE_CONSUMER_VERSION.to_le_bytes());
    put_str(&mut out, &state.scope.user_id)?;
    put_str(&mut out, &state.scope.project_id)?;
    put_str(&mut out, &state.scope.session_id)?;

    match &state.last_entity {
        None => out.push(0),
        Some(entity) => {
            out.push(1);
            put_str(&mut out, entity)?;
        }
    }

    match &state.ur4sess1_bytes {
        None => out.push(0),
        Some(bytes) => {
            Ur4Sess1HeaderView::parse(bytes)?;
            out.push(1);
            put_bytes(&mut out, bytes)?;
        }
    }

    let fact_count = u32::try_from(state.facts.len()).map_err(|_| Error::SchemaMismatch)?;
    out.extend_from_slice(&fact_count.to_le_bytes());
    for record in state.facts.values() {
        put_str(&mut out, &record.owner)?;
        put_str(&mut out, &record.relation)?;
        out.extend_from_slice(&record.version.to_le_bytes());
        out.push(record.action as u8);
        out.push(u8::from(record.conflict));
        match &record.current {
            None => out.push(0),
            Some(curr) => {
                out.push(1);
                encode_provenance(&mut out, curr, limits)?;
            }
        }
        match &record.previous {
            None => out.push(0),
            Some(prev) => {
                out.push(1);
                encode_provenance(&mut out, prev, limits)?;
            }
        }
    }

    let prose_count =
        u32::try_from(state.non_memory_turns.len()).map_err(|_| Error::SchemaMismatch)?;
    out.extend_from_slice(&prose_count.to_le_bytes());
    for (manifest, bindings) in &state.non_memory_turns {
        put_bytes(&mut out, &manifest.encode())?;
        put_bytes(&mut out, &bindings.encode(limits)?)?;
    }

    Ok(out)
}

fn decode_scope_state(bytes: &[u8], limits: &Limits) -> Result<ScopedMemoryState> {
    let mut r = SliceReader::new(bytes);
    if r.take(8)? != DURABLE_CONSUMER_MAGIC {
        return Err(Error::SchemaMismatch);
    }
    let version = r.u32()?;
    if version != DURABLE_CONSUMER_VERSION {
        return Err(Error::SchemaMismatch);
    }
    let user_id = r.string(MAX_SCOPE_FIELD_BYTES)?;
    let project_id = r.string(MAX_SCOPE_FIELD_BYTES)?;
    let session_id = r.string(MAX_SCOPE_FIELD_BYTES)?;
    let scope = IdentityScope::new(user_id, project_id, session_id)?;

    let last_entity = match r.u8()? {
        0 => None,
        1 => {
            let e = r.string(MAX_FACT_KEY_BYTES)?;
            validate_fact_key(&e)?;
            Some(e)
        }
        _ => return Err(Error::SchemaMismatch),
    };

    let ur4sess1_bytes = match r.u8()? {
        0 => None,
        1 => {
            let raw = r.bytes(limits.max_object_bytes as usize)?;
            Ur4Sess1HeaderView::parse(raw)?;
            Some(raw.to_vec())
        }
        _ => return Err(Error::SchemaMismatch),
    };

    let fact_count = r.checked_u32_count(4)?;
    let mut facts = BTreeMap::new();
    for _ in 0..fact_count {
        let owner = r.string(MAX_FACT_KEY_BYTES)?;
        let relation = r.string(MAX_FACT_KEY_BYTES)?;
        validate_fact_key(&owner)?;
        validate_fact_key(&relation)?;
        let f_version = r.u64()?;
        let action = FactAction::from_u8(r.u8()?)?;
        let conflict = match r.u8()? {
            0 => false,
            1 => true,
            _ => return Err(Error::SchemaMismatch),
        };
        let current = match r.u8()? {
            0 => None,
            1 => Some(decode_provenance(&mut r, limits)?),
            _ => return Err(Error::SchemaMismatch),
        };
        let previous = match r.u8()? {
            0 => None,
            1 => Some(decode_provenance(&mut r, limits)?),
            _ => return Err(Error::SchemaMismatch),
        };
        if action != FactAction::Retract && current.is_none() {
            return Err(Error::SchemaMismatch);
        }
        let key = (owner.clone(), relation.clone());
        if facts
            .insert(
                key,
                FactVersionRecord {
                    owner,
                    relation,
                    version: f_version,
                    action,
                    conflict,
                    current,
                    previous,
                },
            )
            .is_some()
        {
            return Err(Error::SchemaMismatch);
        }
    }

    let prose_count = r.checked_u32_count(4)?;
    let mut non_memory_turns = Vec::new();
    reserve(&mut non_memory_turns, prose_count)?;
    for _ in 0..prose_count {
        let m_bytes = r.bytes(limits.max_record_bytes)?;
        let manifest = Manifest::decode(m_bytes, limits)?;
        let b_bytes = r.bytes(limits.max_record_bytes)?;
        let bindings = Bindings::decode(b_bytes, limits)?;
        if bindings.object_version() != manifest.version() {
            return Err(Error::BindingMismatch);
        }
        non_memory_turns.push((manifest, bindings));
    }

    if !r.is_eof() {
        return Err(Error::SchemaMismatch);
    }

    Ok(ScopedMemoryState {
        scope,
        last_entity,
        ur4sess1_bytes,
        facts,
        non_memory_turns,
    })
}

fn put_str(out: &mut Vec<u8>, s: &str) -> Result<()> {
    put_bytes(out, s.as_bytes())
}

fn put_bytes(out: &mut Vec<u8>, b: &[u8]) -> Result<()> {
    let len = u32::try_from(b.len()).map_err(|_| Error::LimitExceeded("envelope field"))?;
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(b);
    Ok(())
}

struct SliceReader<'a> {
    buf: &'a [u8],
    at: usize,
}

impl<'a> SliceReader<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Self { buf, at: 0 }
    }

    fn is_eof(&self) -> bool {
        self.at == self.buf.len()
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self.at.checked_add(n).ok_or(Error::SchemaMismatch)?;
        if end > self.buf.len() {
            return Err(Error::SchemaMismatch);
        }
        let s = &self.buf[self.at..end];
        self.at = end;
        Ok(s)
    }

    fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    fn u32(&mut self) -> Result<u32> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn i32(&mut self) -> Result<i32> {
        let b = self.take(4)?;
        Ok(i32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn u64(&mut self) -> Result<u64> {
        let b = self.take(8)?;
        Ok(u64::from_le_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }

    fn i64(&mut self) -> Result<i64> {
        let b = self.take(8)?;
        Ok(i64::from_le_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }

    fn v64(&mut self) -> Result<u64> {
        let mut acc = 0u64;
        let mut shift = 0u32;
        loop {
            let b = self.u8()?;
            acc |= u64::from(b & 0x7f) << shift;
            if b & 0x80 == 0 {
                return Ok(acc);
            }
            shift += 7;
            if shift >= 64 {
                return Err(Error::SchemaMismatch);
            }
        }
    }

    fn checked_count(&mut self, elem_shift: u32) -> Result<usize> {
        let raw = self.v64()?;
        let n = usize::try_from(raw).map_err(|_| Error::SchemaMismatch)?;
        if n > ((self.buf.len() - self.at) >> elem_shift) {
            return Err(Error::SchemaMismatch);
        }
        Ok(n)
    }

    fn checked_u32_count(&mut self, elem_shift: u32) -> Result<usize> {
        let raw = self.u32()? as usize;
        if raw > ((self.buf.len() - self.at) >> elem_shift) {
            return Err(Error::SchemaMismatch);
        }
        Ok(raw)
    }

    fn bytes(&mut self, max_len: usize) -> Result<&'a [u8]> {
        let len = self.u32()? as usize;
        if len > max_len {
            return Err(Error::LimitExceeded("envelope field"));
        }
        self.take(len)
    }

    fn string(&mut self, max_len: usize) -> Result<String> {
        let b = self.bytes(max_len)?;
        std::str::from_utf8(b)
            .map(str::to_owned)
            .map_err(|_| Error::Utf8)
    }

    fn i32_vec(&mut self) -> Result<Vec<i32>> {
        let n = self.checked_count(2)?;
        let mut v = Vec::new();
        reserve(&mut v, n)?;
        for _ in 0..n {
            v.push(self.i32()?);
        }
        Ok(v)
    }

    fn u32_vec(&mut self) -> Result<Vec<u32>> {
        let n = self.checked_count(2)?;
        let mut v = Vec::new();
        reserve(&mut v, n)?;
        for _ in 0..n {
            v.push(self.u32()?);
        }
        Ok(v)
    }

    fn i64_vec(&mut self) -> Result<Vec<i64>> {
        let n = self.checked_count(3)?;
        let mut v = Vec::new();
        reserve(&mut v, n)?;
        for _ in 0..n {
            v.push(self.i64()?);
        }
        Ok(v)
    }

    fn u64_vec(&mut self) -> Result<Vec<u64>> {
        let n = self.checked_count(3)?;
        let mut v = Vec::new();
        reserve(&mut v, n)?;
        for _ in 0..n {
            v.push(self.u64()?);
        }
        Ok(v)
    }
}
