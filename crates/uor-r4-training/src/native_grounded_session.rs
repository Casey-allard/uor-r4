//! Predicted compiler -> exact versioned store -> the existing native bank.
//! No gold relation, answer record, authored reply or float stack is used.
//! Original statements and value spans stay separate: the store owns exact
//! values; the reader receives the original value slice beside its actual
//! assertion Context, rather than copying compiler cue words as the payload.
//! Host admission/snapshots allocate; this is not a complete-path D11 claim.

use serde::{Deserialize, Serialize};
use uor_r4_core::native_geometric::learner::native_bank_generate::{
    AdmittedBank, BankPin, GenerationLimits, NativeBankGeneration, NativeBankGenerator,
    OwnedBankSegment, OwnedBankSource, PinnedBankSnapshot, SnapshotSourceStatus,
};
use uor_r4_integer::geometric_source_realizer::NativeArtifactBinding;
use uor_r4_tokenizer::ByteBpeTokenizer;

use crate::stack_grounded_session::{
    CompiledAction, CompilerIdentity, EncoderIdentity, MemoryEffect, SessionScope, SourceSpan,
    TurnCompiler,
};
use crate::stack_store::{HistoryView, StackStore, StoreRead, Update};
use crate::{invalid, sha256_bytes, Result};

pub const SCHEMA: &str = "uor-r4.native-grounded-session/1";
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeSessionLimits {
    pub max_turns: usize,
    pub max_source_bytes: usize,
    pub max_store_records: usize,
    pub max_snapshot_bytes: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Turn {
    event: u64,
    source: String,
    action: CompiledAction,
    user_ids: Vec<u32>,
    reply_ids: Vec<u32>,
    record: Option<u64>,
}

/// Store-read status is kept beside the geometric bank, not converted into an
/// invented Source token or an absence string. Every address is reported.
pub struct NativePreparedTurn {
    bank: AdmittedBank,
    effect: MemoryEffect,
    reads: Vec<(u32, StoreRead)>,
    staged: StackStore,
    turn: Turn,
    compiler_sha256: String,
    session_sha256: String,
}
impl NativePreparedTurn {
    pub(crate) fn with_status(mut self, code: u8) -> Result<Self> {
        self.bank = self
            .bank
            .with_status(code)
            .map_err(|e| invalid(e.to_string()))?;
        Ok(self)
    }
    pub fn bank(&self) -> &AdmittedBank {
        &self.bank
    }
    pub fn effect(&self) -> &MemoryEffect {
        &self.effect
    }
    pub fn reads(&self) -> &[(u32, StoreRead)] {
        &self.reads
    }
}

pub struct NativeTurnOutcome {
    pub action: CompiledAction,
    pub effect: MemoryEffect,
    pub reads: Vec<(u32, StoreRead)>,
    pub generation: NativeBankGeneration,
}
#[derive(Clone, Debug)]
pub struct NativeRecordSelection {
    pub relation: u32,
    pub view: HistoryView,
    pub record: u64,
}

fn selected_packet(
    store: &StackStore,
    scope: &SessionScope,
    turns: &[Turn],
    pending: &Turn,
    selected: &[NativeRecordSelection],
) -> Result<(PinnedBankSnapshot, Vec<(u32, StoreRead)>)> {
    let mut sources = std::collections::BTreeMap::new();
    let mut reads = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    if selected.len() > 128 {
        return Err(invalid("selected record admission exceeds128"));
    }
    for s in selected {
        if !seen.insert(s.record) {
            return Err(invalid("selected record duplicated"));
        }
        let read = store
            .read_at(
                &scope.scope,
                &scope.entity,
                s.relation,
                s.view,
                store.commit(),
            )
            .map_err(|e| invalid(e.to_string()))?;
        let value = read
            .value()
            .filter(|v| v.record == s.record && !v.conflict)
            .ok_or_else(|| invalid("selected record is not an eligible exact pinned version"))?;
        let origin = turns
            .iter()
            .chain(std::iter::once(pending))
            .find(|t| t.record == Some(s.record))
            .ok_or_else(|| invalid("selected record has no original source witness"))?;
        sources.insert(
            origin.event,
            OwnedBankSource {
                event: origin.event,
                record: s.record,
                commit: value.commit,
                scope: scope.scope.clone(),
                entity: scope.entity.clone(),
                relation: s.relation,
                view: s.view as u32,
                status: SnapshotSourceStatus::Found,
                original_token_ids: value.tokens.clone(),
            },
        );
        reads.push((s.relation, read));
    }
    let mut segments = Vec::new();
    for t in turns.iter().chain(std::iter::once(pending)) {
        if sources.contains_key(&t.event) {
            segments.push(OwnedBankSegment::Context {
                event: t.event,
                role: 0,
                token_ids: t.user_ids.clone(),
            });
        }
        if let Some(s) = sources.remove(&t.event) {
            segments.push(OwnedBankSegment::Source(s));
        }
    }
    Ok((
        PinnedBankSnapshot {
            pin: BankPin {
                lineage: store.lineage(),
                commit: store.commit(),
                scope: scope.scope.clone(),
            },
            segments,
            query_ids: pending.user_ids.clone(),
        },
        reads,
    ))
}

/// A coarse Claude/SpiralCore E8 route beside exact committed content identity.
/// Routing collisions never merge records. This is not an H4->E8 cast, a metric
/// on digest bits, or adoption of a floating value-table slot as episodic memory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExactSpiralRecordRoute {
    lineage: u64,
    scope: SessionScope,
    record: u64,
    commit: u64,
    payload_sha256: String,
    root: u16,
}
impl ExactSpiralRecordRoute {
    pub fn root(&self) -> u16 {
        self.root
    }
    pub fn record(&self) -> u64 {
        self.record
    }
    pub fn commit(&self) -> u64 {
        self.commit
    }
    pub fn transport(&self, action: u8) -> Result<Self> {
        let mut next = self.clone();
        next.root = crate::e8_memory::e8_tables()?.transport_root(self.root, action)?;
        Ok(next)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    schema: String,
    compiler: CompilerIdentity,
    emitter: NativeArtifactBinding,
    execution_sha256: String,
    scope: SessionScope,
    limits: NativeSessionLimits,
    store: Vec<u8>,
    turns: Vec<Turn>,
}

/// One coherent owner for compiler, store and native reader/emitter. Preparation
/// is immutable; a generation failure cannot commit a write or partial history.
pub struct NativeGroundedSession<'a, C: TurnCompiler> {
    compiler: C,
    compiler_identity: CompilerIdentity,
    emitter: NativeBankGenerator,
    tokenizer: &'a ByteBpeTokenizer,
    scope: SessionScope,
    limits: NativeSessionLimits,
    store: StackStore,
    turns: Vec<Turn>,
}

fn validate_compiler(
    compiler: &impl TurnCompiler,
    emitter: &NativeArtifactBinding,
    tokenizer_bytes: &[u8],
) -> Result<()> {
    let id = compiler.identity();
    let expected = EncoderIdentity {
        config_sha256: emitter.identity.parent_config_sha256.clone(),
        model_sha256: emitter.metadata_sha256.clone(),
        transport_sha256: Some(
            uor_r4_integer::h4_tables::TRUSTED_MATHEMATICAL_SHA256
                .ok_or_else(|| invalid("unbound H4 algebra"))?
                .into(),
        ),
    };
    if sha256_bytes(compiler.artifact_bytes()) != id.artifact_sha256
        || sha256_bytes(tokenizer_bytes) != id.tokenizer_sha256
        || id.tokenizer_sha256 != emitter.identity.tokenizer_sha256
        || id.encoder.as_ref() != Some(&expected)
        || ![
            crate::geometric_turn_compiler::SCHEMA,
            crate::geometric_turn_compiler::JOINT_SCHEMA,
        ]
        .contains(&id.schema.as_str())
        || id.relations.is_empty()
        || id
            .relations
            .iter()
            .enumerate()
            .any(|(i, r)| r.id != (i + 1) as u32 || r.name.is_empty())
    {
        return Err(invalid(
            "native compiler/tokenizer/encoder/label binding differs",
        ));
    }
    Ok(())
}

fn value_ids(tokenizer: &ByteBpeTokenizer, source: &str, span: SourceSpan) -> Result<Vec<u32>> {
    let value = source
        .get(span.start..span.end)
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| invalid("compiler span empty/out-of-range/non-UTF8-boundary"))?;
    let ids = tokenizer.encode(value);
    if ids.is_empty() || tokenizer.decode_bytes(&ids) != value.as_bytes() {
        return Err(invalid(
            "compiler span is not a lossless original-text slice",
        ));
    }
    Ok(ids)
}

fn apply(
    store: &mut StackStore,
    tokenizer: &ByteBpeTokenizer,
    scope: &SessionScope,
    relations: &[crate::stack_grounded_session::RelationLabel],
    source: &str,
    action: &CompiledAction,
    event: u64,
) -> Result<MemoryEffect> {
    let (relation, span, update) = match action {
        CompiledAction::Assert { relation, span } => (*relation, Some(*span), Update::Assert),
        CompiledAction::Correct { relation, span } => (*relation, Some(*span), Update::Correct),
        CompiledAction::QueryCurrent { relation } | CompiledAction::Query { relation, .. } => {
            (*relation, None, Update::Assert)
        }
        CompiledAction::Unresolved { reason } => {
            if reason.trim().is_empty() {
                return Err(invalid("unresolved compiler action has no reason"));
            }
            return Ok(MemoryEffect::Unresolved);
        }
    };
    if !relations.iter().any(|r| r.id == relation) {
        return Err(invalid("compiler relation outside saved schema"));
    }
    if let Some(span) = span {
        let ids = value_ids(tokenizer, source, span)?;
        let written = store
            .write_from(&scope.scope, &scope.entity, relation, &ids, update, event)
            .map_err(|e| invalid(e.to_string()))?;
        Ok(MemoryEffect::Write {
            written,
            value_tokens: ids,
        })
    } else {
        let view = match action {
            CompiledAction::Query { view, .. } => *view,
            _ => HistoryView::Current,
        };
        Ok(MemoryEffect::Read {
            read: store
                .read(&scope.scope, &scope.entity, relation, view)
                .map_err(|e| invalid(e.to_string()))?,
        })
    }
}

/// Admit all current addresses, not a compiler-selected answer record. A typed
/// historical query changes only that address's version view, never ranking.
fn packet(
    store: &StackStore,
    scope: &SessionScope,
    turns: &[Turn],
    pending: &Turn,
) -> Result<(PinnedBankSnapshot, Vec<(u32, StoreRead)>)> {
    let mut sources = std::collections::BTreeMap::new();
    let mut reads = Vec::new();
    for address in store.addresses().map_err(|e| invalid(e.to_string()))? {
        if address.scope != scope.scope || address.entity != scope.entity {
            continue;
        }
        let view = match pending.action {
            CompiledAction::Query { relation, view } if relation == address.relation => view,
            _ => HistoryView::Current,
        };
        let read = store
            .read_at(
                &scope.scope,
                &scope.entity,
                address.relation,
                view,
                store.commit(),
            )
            .map_err(|e| invalid(e.to_string()))?;
        if let StoreRead::Found(value) = &read {
            if !value.conflict {
                let origin = turns
                    .iter()
                    .chain(std::iter::once(pending))
                    .find(|t| t.record == Some(value.record))
                    .ok_or_else(|| {
                        invalid("stored record has no original compiler/source witness")
                    })?;
                sources.insert(
                    origin.event,
                    OwnedBankSource {
                        event: origin.event,
                        record: value.record,
                        commit: value.commit,
                        scope: scope.scope.clone(),
                        entity: scope.entity.clone(),
                        relation: address.relation,
                        view: view as u32,
                        status: SnapshotSourceStatus::Found,
                        original_token_ids: value.tokens.clone(),
                    },
                );
            }
        }
        reads.push((address.relation, read));
    }
    let mut segments = Vec::new();
    for t in turns.iter().chain(std::iter::once(pending)) {
        // The current turn is the query, except a freshly committed statement
        // also supplies its own preceding factual Context and original Source.
        if t.event != pending.event || sources.contains_key(&t.event) {
            segments.push(OwnedBankSegment::Context {
                event: t.event,
                role: 0,
                token_ids: t.user_ids.clone(),
            });
        }
        if let Some(source) = sources.remove(&t.event) {
            segments.push(OwnedBankSegment::Source(source));
        }
        if !t.reply_ids.is_empty() {
            segments.push(OwnedBankSegment::Context {
                event: t.event,
                role: 1,
                token_ids: t.reply_ids.clone(),
            });
        }
    }
    if !sources.is_empty() {
        return Err(invalid("unconsumed exact source witnesses"));
    }
    Ok((
        PinnedBankSnapshot {
            pin: BankPin {
                lineage: store.lineage(),
                commit: store.commit(),
                scope: scope.scope.clone(),
            },
            segments,
            query_ids: pending.user_ids.clone(),
        },
        reads,
    ))
}

impl<'a, C: TurnCompiler> NativeGroundedSession<'a, C> {
    pub fn new(
        compiler: C,
        emitter: NativeBankGenerator,
        tokenizer: &'a ByteBpeTokenizer,
        tokenizer_bytes: &[u8],
        scope: SessionScope,
        limits: NativeSessionLimits,
        lineage: u64,
        capacity: usize,
    ) -> Result<Self> {
        validate_compiler(&compiler, emitter.source_binding(), tokenizer_bytes)?;
        let decoded = ByteBpeTokenizer::from_tokenizer_json_bytes(tokenizer_bytes)
            .ok_or_else(|| invalid("invalid bound native tokenizer bytes"))?;
        if decoded.address() != tokenizer.address() {
            return Err(invalid("live tokenizer differs from bound bytes"));
        }
        StackStore::key(&scope.scope, &scope.entity, 1).map_err(|e| invalid(e.to_string()))?;
        if limits.max_turns == 0
            || limits.max_turns > 128
            || limits.max_source_bytes == 0
            || limits.max_source_bytes > 1024 * 1024
            || limits.max_store_records == 0
            || limits.max_snapshot_bytes == 0
            || limits.max_snapshot_bytes > 16 * 1024 * 1024
        {
            return Err(invalid("native session admission limits invalid"));
        }
        let compiler_identity = compiler.identity().clone();
        let store = StackStore::new(lineage, capacity).map_err(|e| invalid(e.to_string()))?;
        Ok(Self {
            compiler,
            compiler_identity,
            emitter,
            tokenizer,
            scope,
            limits,
            store,
            turns: Vec::new(),
        })
    }
    pub fn store(&self) -> &StackStore {
        &self.store
    }
    pub(crate) fn record_source(&self, record: u64) -> Result<&str> {
        self.turns
            .iter()
            .find(|t| t.record == Some(record))
            .map(|t| t.source.as_str())
            .ok_or_else(|| invalid("exact record lacks original compiler source"))
    }
    pub(crate) fn teacher_step(
        &mut self,
        prepared: &NativePreparedTurn,
        prefix: &[u32],
    ) -> Result<uor_r4_core::native_geometric::learner::native_bank_generate::NativeBankGenerateStep>
    {
        self.check_compiler()?;
        if prepared.session_sha256 != self.session_sha256()?
            || prepared.compiler_sha256 != self.compiler_identity.artifact_sha256
        {
            return Err(invalid("teacher packet no longer binds this session"));
        }
        self.emitter
            .step(&prepared.bank, prefix)
            .map_err(|e| invalid(e.to_string()))
    }
    /// The route is supplied by the existing geometric reader, not derived from
    /// hash bits. Bind only an exact committed record in this session's scope.
    pub fn bind_spiral_route(&self, record: u64, root: u16) -> Result<ExactSpiralRecordRoute> {
        let tables = crate::e8_memory::e8_tables()?;
        tables.transport_root(root, 0)?;
        let r = self
            .store
            .memory()
            .record_ref(record)
            .filter(|r| !r.evicted && r.scope == self.scope.scope)
            .ok_or_else(|| invalid("Spiral route record unavailable/evicted/foreign"))?;
        let owned = self.turns.iter().any(|t| t.record == Some(record));
        if !owned {
            return Err(invalid("Spiral route lacks compiler content binding"));
        }
        Ok(ExactSpiralRecordRoute {
            lineage: self.store.lineage(),
            scope: self.scope.clone(),
            record,
            commit: r.commit,
            payload_sha256: r.payload_sha256.clone(),
            root,
        })
    }
    pub fn read_spiral_route(&self, route: &ExactSpiralRecordRoute) -> Result<&[u32]> {
        if route.lineage != self.store.lineage() || route.scope != self.scope {
            return Err(invalid("Spiral route store lineage/scope/entity differs"));
        }
        crate::e8_memory::e8_tables()?.transport_root(route.root, 0)?;
        let r = self
            .store
            .memory()
            .record_ref(route.record)
            .filter(|r| {
                !r.evicted
                    && r.scope == self.scope.scope
                    && r.commit == route.commit
                    && r.payload_sha256 == route.payload_sha256
            })
            .ok_or_else(|| invalid("Spiral route exact content unavailable/evicted/foreign"))?;
        Ok(&r.payload)
    }
    fn session_sha256(&self) -> Result<String> {
        Ok(sha256_bytes(&serde_json::to_vec(&(
            &self.scope,
            &self.limits,
            self.store.capacity(),
            &self.turns,
            self.store
                .history_sha256()
                .map_err(|e| invalid(e.to_string()))?,
            &self.compiler_identity,
            self.emitter.source_binding(),
            self.emitter
                .execution_sha256()
                .map_err(|e| invalid(e.to_string()))?,
        ))?))
    }
    fn check_compiler(&self) -> Result<()> {
        if self.compiler.identity() != &self.compiler_identity
            || sha256_bytes(self.compiler.artifact_bytes())
                != self.compiler_identity.artifact_sha256
        {
            return Err(invalid("compiler changed during native session"));
        }
        Ok(())
    }
    pub fn prepare_turn(&self, source: &str) -> Result<NativePreparedTurn> {
        self.prepare_with(
            source,
            None::<
                fn(
                    &StackStore,
                    &CompiledAction,
                    &MemoryEffect,
                ) -> Result<Vec<NativeRecordSelection>>,
            >,
        )
    }
    pub fn prepare_selected_turn(
        &self,
        source: &str,
        select: impl FnOnce(
            &StackStore,
            &CompiledAction,
            &MemoryEffect,
        ) -> Result<Vec<NativeRecordSelection>>,
    ) -> Result<NativePreparedTurn> {
        self.prepare_with(source, Some(select))
    }
    fn prepare_with(
        &self,
        source: &str,
        select: Option<
            impl FnOnce(
                &StackStore,
                &CompiledAction,
                &MemoryEffect,
            ) -> Result<Vec<NativeRecordSelection>>,
        >,
    ) -> Result<NativePreparedTurn> {
        self.check_compiler()?;
        if self.turns.len() >= self.limits.max_turns
            || source.len() > self.limits.max_source_bytes
            || source.trim().is_empty()
        {
            return Err(invalid("native turn source/turn admission exceeded"));
        }
        let user_ids = self.tokenizer.encode(source);
        if self.tokenizer.decode_bytes(&user_ids) != source.as_bytes()
            || user_ids
                .iter()
                .any(|&id| !self.emitter.token_binding().admits_token(id))
        {
            return Err(invalid("native user text is not admitted/lossless"));
        }
        let action = self
            .compiler
            .compile(source)
            .map_err(|e| invalid(e.to_string()))?;
        let event = self.turns.len() as u64 + 1;
        let mut staged = self.store.clone();
        let effect = apply(
            &mut staged,
            self.tokenizer,
            &self.scope,
            &self.compiler_identity.relations,
            source,
            &action,
            event,
        )?;
        if staged.records() > self.limits.max_store_records {
            return Err(invalid("native store record admission exceeded"));
        }
        let record = match effect {
            MemoryEffect::Write { written, .. } => Some(written.id),
            _ => None,
        };
        let turn = Turn {
            event,
            source: source.into(),
            action,
            user_ids,
            reply_ids: Vec::new(),
            record,
        };
        let selected_path = select.is_some();
        let (mut input, reads) = match select {
            Some(f) => selected_packet(
                &staged,
                &self.scope,
                &self.turns,
                &turn,
                &f(&staged, &turn.action, &effect)?,
            )?,
            None => packet(&staged, &self.scope, &self.turns, &turn)?,
        };
        // The selected path admits source witnesses, not all historical text.
        // One most-recent unselected exchange is optional within the remaining
        // native budget. Legacy all-address packet behavior stays unchanged.
        let mut bank = self
            .emitter
            .admit_bank(PinnedBankSnapshot {
                pin: input.pin.clone(),
                segments: input.segments.clone(),
                query_ids: input.query_ids.clone(),
            })
            .map_err(|e| invalid(e.to_string()))?;
        if selected_path {
            if let Some(previous) = self.turns.last() {
                let already_included = input.segments.iter().any(|s| match s {
                    OwnedBankSegment::Source(s) => s.event == previous.event,
                    OwnedBankSegment::Context { event, .. } => *event == previous.event,
                });
                if !already_included
                    && bank.base_tokens() + previous.user_ids.len() + previous.reply_ids.len()
                        <= 120
                {
                    input.segments.push(OwnedBankSegment::Context {
                        event: previous.event,
                        role: 0,
                        token_ids: previous.user_ids.clone(),
                    });
                    if !previous.reply_ids.is_empty() {
                        input.segments.push(OwnedBankSegment::Context {
                            event: previous.event,
                            role: 1,
                            token_ids: previous.reply_ids.clone(),
                        });
                    }
                    input.segments.sort_by_key(|s| match s {
                        OwnedBankSegment::Source(s) => (s.event, 1),
                        OwnedBankSegment::Context { event, role, .. } => {
                            (*event, if *role == 0 { 0 } else { 2 })
                        }
                    });
                    bank = self
                        .emitter
                        .admit_bank(input)
                        .map_err(|e| invalid(e.to_string()))?;
                }
            }
        }
        Ok(NativePreparedTurn {
            bank,
            effect,
            reads,
            staged,
            turn,
            compiler_sha256: self.compiler_identity.artifact_sha256.clone(),
            session_sha256: self.session_sha256()?,
        })
    }
    pub fn turn(&mut self, source: &str, limits: GenerationLimits) -> Result<NativeTurnOutcome> {
        let prepared = self.prepare_turn(source)?;
        self.emit_prepared(prepared, limits)
    }
    pub fn emit_prepared(
        &mut self,
        prepared: NativePreparedTurn,
        limits: GenerationLimits,
    ) -> Result<NativeTurnOutcome> {
        self.emit_prepared_with(prepared, limits, |generator, bank, limits| {
            generator
                .generate(bank, limits)
                .map_err(|e| invalid(e.to_string()))
        })
    }
    pub(crate) fn emit_prepared_with(
        &mut self,
        mut prepared: NativePreparedTurn,
        limits: GenerationLimits,
        generate: impl FnOnce(
            &mut NativeBankGenerator,
            &AdmittedBank,
            GenerationLimits,
        ) -> Result<NativeBankGeneration>,
    ) -> Result<NativeTurnOutcome> {
        self.check_compiler()?;
        if prepared.compiler_sha256 != self.compiler_identity.artifact_sha256
            || prepared.session_sha256 != self.session_sha256()?
            || prepared.turn.event != self.turns.len() as u64 + 1
        {
            return Err(invalid("prepared native turn is stale/foreign"));
        }
        if limits.maximum_tokens == 0 {
            return Err(invalid("native reply token admission is zero"));
        }
        let generation = generate(&mut self.emitter, &prepared.bank, limits)?;
        if generation.pin != *prepared.bank.pin()
            || generation.generated_ids.len() > limits.maximum_tokens
            || generation
                .generated_ids
                .iter()
                .any(|&id| !self.emitter.token_binding().admits_token(id))
            || generation.executed_steps != generation.generated_ids.len()
        {
            return Err(invalid(
                "offline generation receipt differs from admitted bank/limits",
            ));
        }
        if generation.executed_steps == 0 {
            return Err(invalid("native bank leaves no reply context budget"));
        }
        prepared.turn.reply_ids = generation.generated_ids.clone();
        self.store = prepared.staged;
        self.turns.push(prepared.turn);
        Ok(NativeTurnOutcome {
            action: self
                .turns
                .last()
                .ok_or_else(|| invalid("native turn commit absent"))?
                .action
                .clone(),
            effect: prepared.effect,
            reads: prepared.reads,
            generation,
        })
    }
    pub fn snapshot(&self) -> Result<Vec<u8>> {
        self.check_compiler()?;
        let bytes = serde_json::to_vec(&Snapshot {
            schema: SCHEMA.into(),
            compiler: self.compiler_identity.clone(),
            emitter: self.emitter.source_binding().clone(),
            execution_sha256: self
                .emitter
                .execution_sha256()
                .map_err(|e| invalid(e.to_string()))?,
            scope: self.scope.clone(),
            limits: self.limits.clone(),
            store: self.store.to_bytes().map_err(|e| invalid(e.to_string()))?,
            turns: self.turns.clone(),
        })?;
        if bytes.len() > self.limits.max_snapshot_bytes {
            return Err(invalid("native snapshot admission exceeded"));
        }
        Ok(bytes)
    }
    /// The expected digest comes from separately trusted custody, not a field
    /// inside this snapshot. Inner replay checks do not authenticate reply IDs.
    pub fn restore(&mut self, bytes: &[u8], trusted_sha256: &str) -> Result<()> {
        self.restore_checked(bytes, trusted_sha256, |_, _| Ok(()))
    }
    pub(crate) fn restore_checked(
        &mut self,
        bytes: &[u8],
        trusted_sha256: &str,
        check: impl FnOnce(&StackStore, &[(u64, String)]) -> Result<()>,
    ) -> Result<()> {
        self.check_compiler()?;
        if bytes.len() > self.limits.max_snapshot_bytes || sha256_bytes(bytes) != trusted_sha256 {
            return Err(invalid("native snapshot admission exceeded"));
        }
        let s: Snapshot = serde_json::from_slice(bytes)?;
        if s.schema != SCHEMA
            || s.compiler != self.compiler_identity
            || s.emitter != *self.emitter.source_binding()
            || s.execution_sha256
                != self
                    .emitter
                    .execution_sha256()
                    .map_err(|e| invalid(e.to_string()))?
            || s.scope != self.scope
            || s.limits != self.limits
            || s.turns.len() > self.limits.max_turns
        {
            return Err(invalid(
                "native snapshot compiler/model/scope/limits differ",
            ));
        }
        let restored = StackStore::from_bytes(&s.store, self.store.lineage())
            .map_err(|e| invalid(e.to_string()))?;
        if restored.capacity() != self.store.capacity()
            || restored.records() > self.limits.max_store_records
        {
            return Err(invalid("native snapshot store admission differs"));
        }
        // Replay recorded predictions, not training labels. This binds original
        // spans, occurrence/version identity and exact values across reload.
        let mut replay = StackStore::new(restored.lineage(), restored.capacity())
            .map_err(|e| invalid(e.to_string()))?;
        for (i, t) in s.turns.iter().enumerate() {
            if t.event != i as u64 + 1
                || t.source.trim().is_empty()
                || t.source.len() > self.limits.max_source_bytes
                || t.user_ids != self.tokenizer.encode(&t.source)
                || t.action
                    != self
                        .compiler
                        .compile(&t.source)
                        .map_err(|e| invalid(e.to_string()))?
                || t.reply_ids
                    .iter()
                    .any(|&id| !self.emitter.token_binding().admits_token(id))
            {
                return Err(invalid(
                    "native snapshot chronological compiler/history witness differs",
                ));
            }
            let effect = apply(
                &mut replay,
                self.tokenizer,
                &self.scope,
                &self.compiler_identity.relations,
                &t.source,
                &t.action,
                t.event,
            )?;
            let record = match effect {
                MemoryEffect::Write { written, .. } => Some(written.id),
                _ => None,
            };
            if record != t.record {
                return Err(invalid("native snapshot source record differs"));
            }
        }
        if replay.to_bytes().map_err(|e| invalid(e.to_string()))? != s.store {
            return Err(invalid(
                "native snapshot store not explained by compiler history",
            ));
        }
        let witnesses = s
            .turns
            .iter()
            .filter_map(|t| t.record.map(|r| (r, t.source.clone())))
            .collect::<Vec<_>>();
        check(&restored, &witnesses)?;
        self.store = restored;
        self.turns = s.turns;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stack_grounded_session::RelationLabel;
    const TOK: &str = r#"{"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":{"<|bos|>":0,"<|eos|>":1,"<|unk|>":2,".":3,"a":4,"b":5,"\u0120":6},"merges":[]},"added_tokens":[{"id":0,"content":"<|bos|>"},{"id":1,"content":"<|eos|>"},{"id":2,"content":"<|unk|>"}]}"#;
    fn tokenizer() -> Result<ByteBpeTokenizer> {
        ByteBpeTokenizer::from_tokenizer_json_bytes(TOK.as_bytes())
            .ok_or_else(|| invalid("fixture tokenizer"))
    }
    fn scope() -> SessionScope {
        SessionScope {
            scope: b"owner".to_vec(),
            entity: vec![4],
        }
    }
    fn labels() -> Vec<RelationLabel> {
        vec![
            RelationLabel {
                id: 1,
                name: "job".into(),
            },
            RelationLabel {
                id: 2,
                name: "home".into(),
            },
        ]
    }
    fn write(
        store: &mut StackStore,
        turns: &mut Vec<Turn>,
        source: &str,
        relation: u32,
        update: bool,
    ) -> Result<()> {
        let tokenizer = tokenizer()?;
        let event = turns.len() as u64 + 1;
        let span = SourceSpan { start: 2, end: 3 };
        let action = if update {
            CompiledAction::Correct { relation, span }
        } else {
            CompiledAction::Assert { relation, span }
        };
        let effect = apply(
            store,
            &tokenizer,
            &scope(),
            &labels(),
            source,
            &action,
            event,
        )?;
        let MemoryEffect::Write { written, .. } = effect else {
            return Err(invalid("fixture write absent"));
        };
        turns.push(Turn {
            event,
            source: source.into(),
            action,
            user_ids: tokenizer.encode(source),
            reply_ids: vec![5, 1],
            record: Some(written.id),
        });
        Ok(())
    }
    fn query(turns: &[Turn], action: CompiledAction) -> Turn {
        Turn {
            event: turns.len() as u64 + 1,
            source: "a".into(),
            action,
            user_ids: vec![4],
            reply_ids: vec![],
            record: None,
        }
    }
    #[test]
    fn native_memory_packet_retains_all_addresses_and_original_value_not_cues() -> Result<()> {
        let mut store = StackStore::new(7, 4).map_err(|e| invalid(e.to_string()))?;
        let mut turns = vec![];
        write(&mut store, &mut turns, "a a", 1, false)?;
        write(&mut store, &mut turns, "b b", 2, false)?;
        let q = query(&turns, CompiledAction::QueryCurrent { relation: 1 });
        let (bank, reads) = packet(&store, &scope(), &turns, &q)?;
        assert_eq!(reads.len(), 2);
        assert_eq!(bank.pin.commit, 2);
        let sources: Vec<_> = bank
            .segments
            .iter()
            .filter_map(|s| match s {
                OwnedBankSegment::Source(s) => Some(s),
                _ => None,
            })
            .collect();
        assert_eq!(sources.len(), 2); // not a compiler/gold selected answer
        assert_eq!(sources[0].original_token_ids, [4]);
        assert_eq!(sources[1].original_token_ids, [5]);
        assert_eq!((sources[0].record, sources[1].record), (1, 2));
        let contexts: Vec<_> = bank
            .segments
            .iter()
            .filter_map(|s| match s {
                OwnedBankSegment::Context { token_ids, .. } => Some(token_ids.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(
            contexts,
            [vec![4, 6, 4], vec![5, 1], vec![5, 6, 5], vec![5, 1]]
        );
        let foreign = SessionScope {
            scope: b"other".to_vec(),
            entity: vec![4],
        };
        let (bank, reads) = packet(&store, &foreign, &turns, &q)?;
        assert!(reads.is_empty());
        assert!(!bank
            .segments
            .iter()
            .any(|s| matches!(s, OwnedBankSegment::Source(_))));
        Ok(())
    }
    #[test]
    fn native_memory_packet_reassertion_history_eviction_and_conflict_stay_distinct() -> Result<()>
    {
        let mut store = StackStore::new(7, 2).map_err(|e| invalid(e.to_string()))?;
        let mut turns = vec![];
        write(&mut store, &mut turns, "a a", 1, false)?;
        write(&mut store, &mut turns, "b a", 1, false)?;
        let previous = query(
            &turns,
            CompiledAction::Query {
                relation: 1,
                view: HistoryView::PreviousAssertion,
            },
        );
        let (bank, _) = packet(&store, &scope(), &turns, &previous)?;
        assert!(bank
            .segments
            .iter()
            .any(|s| matches!(s,OwnedBankSegment::Source(s) if s.record==1)));
        write(&mut store, &mut turns, "a b", 1, true)?;
        let initial = query(
            &turns,
            CompiledAction::Query {
                relation: 1,
                view: HistoryView::Initial,
            },
        );
        let (bank, reads) = packet(&store, &scope(), &turns, &initial)?;
        assert!(matches!(reads[0].1, StoreRead::Evicted));
        assert!(!bank
            .segments
            .iter()
            .any(|s| matches!(s, OwnedBankSegment::Source(_))));
        write(&mut store, &mut turns, "a a", 1, false)?;
        let current = query(&turns, CompiledAction::QueryCurrent { relation: 1 });
        let (bank, reads) = packet(&store, &scope(), &turns, &current)?;
        assert!(matches!(&reads[0].1,StoreRead::Found(v) if v.conflict));
        assert!(!bank
            .segments
            .iter()
            .any(|s| matches!(s, OwnedBankSegment::Source(_))));
        Ok(())
    }
    #[test]
    fn native_memory_adapter_rejects_bad_spans_labels_and_unwitnessed_content() -> Result<()> {
        let tokenizer = tokenizer()?;
        let mut store = StackStore::new(7, 2).map_err(|e| invalid(e.to_string()))?;
        for action in [
            CompiledAction::Assert {
                relation: 3,
                span: SourceSpan { start: 0, end: 1 },
            },
            CompiledAction::Assert {
                relation: 1,
                span: SourceSpan { start: 0, end: 9 },
            },
            CompiledAction::Unresolved { reason: "".into() },
        ] {
            assert!(apply(&mut store, &tokenizer, &scope(), &labels(), "a", &action, 1).is_err());
            assert_eq!(store.commit(), 0);
        }
        assert!(value_ids(&tokenizer, "\u{e9}", SourceSpan { start: 1, end: 2 }).is_err());
        store
            .write(&scope().scope, &scope().entity, 1, &[4], Update::Assert)
            .map_err(|e| invalid(e.to_string()))?;
        let q = query(&[], CompiledAction::QueryCurrent { relation: 1 });
        assert!(packet(&store, &scope(), &[], &q).is_err());
        Ok(())
    }
}
