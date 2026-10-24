//! Deterministic corpus/coordinate driver. Corpus commits require explicit pilot admission.
//! Receipt/cursor arithmetic is metadata; every model evaluation and proposal is CUDA.
#![forbid(unsafe_code)]
use crate::{
    corpus::{
        self, BosIdentity, CorpusBatch, CorpusBinding, CorpusSource, FilePin, PreparedCorpus, Split,
    },
    metadata::BinderPlan,
    normalization_mechanism::NormalizationMode,
    position_mechanism::PositionConfig,
    Error, ObjectivePolicy, ReadInitialization, ReadSpec, Result, Spec,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const DRIVER_SCHEMA: &str = "uor-r4.discrete-corpus-driver/1";
pub const CHECKPOINT_SCHEMA: &str = "uor-r4.discrete-driver-checkpoint/1";
fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(message.into())
}
pub fn driver_source_sha256() -> String {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(include_bytes!("corpus.rs"));
    bytes.extend_from_slice(include_bytes!("training.rs"));
    bytes.extend_from_slice(include_bytes!("controller.rs"));
    bytes.extend_from_slice(include_bytes!("completion.rs"));
    bytes.extend_from_slice(include_bytes!("completion_baseline.rs"));
    bytes.extend_from_slice(include_bytes!("redirect_admission.rs"));
    bytes.extend_from_slice(include_bytes!("attention_learning.rs"));
    bytes.extend_from_slice(include_bytes!("attention_evidence.rs"));
    bytes.extend_from_slice(include_bytes!("attention_transfer.rs"));
    bytes.extend_from_slice(include_bytes!("attention_preflight.rs"));
    bytes.extend_from_slice(include_bytes!("attention_anchor.rs"));
    bytes.extend_from_slice(include_bytes!("attention_audit.rs"));
    bytes.extend_from_slice(include_bytes!("attention_anchor_device.rs"));
    bytes.extend_from_slice(include_bytes!("attention_audited.rs"));
    bytes.extend_from_slice(include_bytes!("attention_audit_qualification.rs"));
    bytes.extend_from_slice(include_bytes!("driver_qualification.rs"));
    bytes.extend_from_slice(include_bytes!("relation_qualification.rs"));
    bytes.extend_from_slice(include_bytes!("snapshot.rs"));
    bytes.extend_from_slice(include_bytes!("position_mechanism.rs"));
    bytes.extend_from_slice(include_bytes!("main.rs"));
    corpus::sha256(&bytes)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchedulePolicy {
    CanonicalVectorThenReadV1,
    InterleavedReadEvery8V1,
}
/// Whether genuine Relation proposals may commit their relation-tail coordinate.
/// This does not change the read operator, proposal inventory or schedule.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationLearningPolicy {
    #[default]
    LearnV1,
    FrozenZeroV1,
}
impl RelationLearningPolicy {
    pub fn is_learn(&self) -> bool {
        *self == Self::LearnV1
    }
}
/// Which exact counterfactual kernel scores a proposal. Both produce identical
/// losses, chooser input and statuses (row-parallel GPU parity, #2630); only the
/// launch shape differs. The default keeps existing configurations byte-identical.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CounterfactualPath {
    #[default]
    LanesV1,
    RowsV1,
}
impl CounterfactualPath {
    pub fn is_lanes(&self) -> bool {
        *self == Self::LanesV1
    }
}
/// BVG-K Stage C2: coordinates committed per batch (`K`). The default `K = 1`
/// is never serialized, so existing config and lineage bytes are unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CoordinateBlock(pub u32);
impl Default for CoordinateBlock {
    fn default() -> Self {
        Self::ONE
    }
}
impl CoordinateBlock {
    pub const ONE: Self = Self(1);
    pub fn is_one(&self) -> bool {
        *self == Self::ONE
    }
    pub fn get(self) -> u32 {
        self.0
    }
}
/// Initial codebook of a fresh model (D39 item 3). The default is the seeded
/// device initializer, and it keeps existing configurations byte-identical. A
/// pinned table is an exact integer codebook (e.g. the qualified PrimeZeta
/// constructor's output), bound into the lineage by its file SHA-256 and
/// admitted by the same GPU snapshot validation as any reload.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CodebookInitialization {
    #[default]
    SeededDeviceV1,
    PinnedTableV1 {
        table: FilePin,
    },
}
impl CodebookInitialization {
    pub fn is_seeded_device(&self) -> bool {
        *self == Self::SeededDeviceV1
    }
}
pub const CODEBOOK_TABLE_SCHEMA: &str = "uor-r4.codebook-table/1";
/// The integer codebook file a pinned initialization names. `table_sha256` is
/// SHA-256 over the little-endian bytes of every word, in table order (the
/// PrimeZeta constructor's own table digest).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CodebookTable {
    pub schema: String,
    pub spec: Spec,
    pub mode: String,
    pub table: Vec<i32>,
    pub table_sha256: String,
    /// Declared number of distinct full token vectors (4,096 means injective at
    /// V4096). The loader recounts it, so a claimed property is a checked one.
    pub distinct_vectors: usize,
}
impl CodebookTable {
    pub fn words_sha256(table: &[i32]) -> String {
        let bytes: Vec<u8> = table.iter().flat_map(|w| w.to_le_bytes()).collect();
        corpus::sha256(&bytes)
    }
    /// Distinct full token vectors (all blocks) in a table of `blocks` blocks.
    pub fn count_distinct(table: &[i32], blocks: u32) -> usize {
        let width = blocks as usize * 8;
        if width == 0 {
            return 0;
        }
        table
            .chunks_exact(width)
            .collect::<std::collections::HashSet<_>>()
            .len()
    }
    /// Read and check a pinned table for `spec`. Host metadata only: root
    /// membership is admitted on the GPU when the table is loaded.
    pub fn load(pin: &FilePin, spec: Spec) -> Result<Vec<i32>> {
        let file: Self =
            serde_json::from_slice(&pin.read()?).map_err(|e| invalid(e.to_string()))?;
        if file.schema != CODEBOOK_TABLE_SCHEMA {
            return Err(invalid(format!("codebook table schema {}", file.schema)));
        }
        if file.spec.vocabulary != spec.vocabulary || file.spec.blocks != spec.blocks {
            return Err(invalid(
                "codebook table vocabulary/blocks differ from the driver spec",
            ));
        }
        if file.table.len() != spec.vocabulary as usize * spec.blocks as usize * 8 {
            return Err(invalid(
                "codebook table length differs from vocabulary x blocks x 8",
            ));
        }
        if Self::words_sha256(&file.table) != file.table_sha256 {
            return Err(invalid(
                "codebook table words differ from their declared SHA-256",
            ));
        }
        if Self::count_distinct(&file.table, spec.blocks) != file.distinct_vectors {
            return Err(invalid(
                "codebook table distinct-vector count differs from its declaration",
            ));
        }
        Ok(file.table)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DriverConfig {
    pub schema: String,
    pub spec: Spec,
    pub read_spec: ReadSpec,
    #[serde(
        default,
        skip_serializing_if = "ObjectivePolicy::is_nearest_address_margin"
    )]
    pub objective_policy: ObjectivePolicy,
    #[serde(default, skip_serializing_if = "ReadInitialization::is_zero_gates")]
    pub read_initialization: ReadInitialization,
    #[serde(
        default,
        skip_serializing_if = "NormalizationMode::is_scale_matched_snap"
    )]
    pub normalization_mode: NormalizationMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<PositionConfig>,
    #[serde(default, skip_serializing_if = "RelationLearningPolicy::is_learn")]
    pub relation_learning_policy: RelationLearningPolicy,
    #[serde(default, skip_serializing_if = "CounterfactualPath::is_lanes")]
    pub counterfactual_path: CounterfactualPath,
    #[serde(
        default,
        skip_serializing_if = "CodebookInitialization::is_seeded_device"
    )]
    pub codebook_initialization: CodebookInitialization,
    #[serde(default, skip_serializing_if = "CoordinateBlock::is_one")]
    pub coordinate_block: CoordinateBlock,
    pub seed: u32,
    pub batch_rows: u32,
    pub train: CorpusSource,
    pub dev: CorpusSource,
    pub tokenizer: FilePin,
    pub bos: BosIdentity,
    pub schedule: SchedulePolicy,
}
impl DriverConfig {
    pub fn validate(&self) -> Result<()> {
        self.read_spec.validate(self.spec)?;
        if let CodebookInitialization::PinnedTableV1 { table } = &self.codebook_initialization {
            corpus::validate_sha256(&table.sha256)?;
        }
        self.read_initialization.validate(self.read_spec)?;
        if self.read_initialization == ReadInitialization::ActiveLeafV1
            && (self.read_spec == ReadSpec::Route
                || self.schedule != SchedulePolicy::InterleavedReadEvery8V1)
        {
            return Err(invalid(
                "active-leaf initialization requires an explicitly interleaved Flock or Relation driver",
            ));
        }
        if matches!(
            self.read_initialization,
            ReadInitialization::RouteBypassV1 | ReadInitialization::RoutePermutedActionsV1
        ) && (self.read_spec != ReadSpec::Route
            || self.schedule != SchedulePolicy::CanonicalVectorThenReadV1)
        {
            return Err(invalid(
                "route read-off and permuted controls require canonical Route schedule",
            ));
        }
        if self.normalization_mode.device_word()? != 0 && self.read_spec == ReadSpec::Route {
            return Err(invalid("snap-scale control needs a Flock or Relation read"));
        }
        if let Some(position) = &self.position {
            position.validate(self.spec, self.read_spec)?;
        }
        if self.relation_learning_policy == RelationLearningPolicy::FrozenZeroV1
            && (!matches!(self.read_spec, ReadSpec::Relation { .. })
                || self.schedule != SchedulePolicy::InterleavedReadEvery8V1
                || self.read_initialization != ReadInitialization::ActiveLeafV1)
        {
            return Err(invalid(
                "frozen-zero relation learning requires interleaved Relation with active-leaf initialization",
            ));
        }
        if !(1..=64).contains(&self.coordinate_block.get()) {
            return Err(invalid("coordinate_block must be 1..=64"));
        }
        if !self.coordinate_block.is_one() && self.counterfactual_path != CounterfactualPath::RowsV1
        {
            return Err(invalid(
                "coordinate_block > 1 requires the rows_v1 counterfactual path",
            ));
        }
        if self.schema != DRIVER_SCHEMA
            || !(1..=64).contains(&self.batch_rows)
            || self.train.split != Split::Train
            || self.dev.split != Split::Dev
        {
            return Err(invalid(
                "driver needs current schema, rows1..64 and separate declared TRAIN/DEV roles",
            ));
        }
        for pin in [
            &self.train.tokens,
            &self.train.mask,
            &self.dev.tokens,
            &self.dev.mask,
            &self.tokenizer,
        ] {
            corpus::validate_sha256(&pin.sha256)?;
        }
        if self.train.tokens.sha256 == self.dev.tokens.sha256 {
            return Err(invalid("DEV content cannot be passed as TRAIN reference"));
        }
        if self.bos.token.is_empty() || self.bos.id >= self.spec.vocabulary {
            return Err(invalid("driver BOS metadata invalid"));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lineage {
    pub schema: String,
    pub driver_source_sha256: String,
    pub config: DriverConfig,
    pub train: CorpusBinding,
    pub dev: CorpusBinding,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoordinateKind {
    VectorBlock,
    Route,
    Gate,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Coordinate {
    pub kind: CoordinateKind,
    pub index: u32,
}
impl Coordinate {
    fn parameter(self) -> crate::Parameter {
        match self.kind {
            CoordinateKind::VectorBlock => crate::Parameter::VectorBlock(self.index),
            CoordinateKind::Route => crate::Parameter::Route(self.index),
            CoordinateKind::Gate => crate::Parameter::Gate(self.index),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cursor {
    pub completed_batches: u64,
    pub accepted_updates: u64,
    pub corpus_pass: u64,
    pub next_target: u64,
    pub next_coordinate: u32,
    /// Present iff `coordinate_block > 1`; absent (bytes unchanged) at `K = 1`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block: Option<BlockCursor>,
}
impl Cursor {
    pub fn initial() -> Self {
        Self {
            completed_batches: 0,
            accepted_updates: 0,
            corpus_pass: 0,
            next_target: 0,
            next_coordinate: 0,
            block: None,
        }
    }
}
/// Per-family coordinate ordinals of a `K > 1` schedule (absolute visit
/// counts; the family index is the ordinal modulo the family size).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockCursor {
    pub o_vec: u64,
    pub o_read: u64,
    pub committed_moves: u64,
}
/// The coordinates one batch visits: one family, in today's order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub kind: CoordinateKind,
    pub coordinates: Vec<crate::Parameter>,
}
/// The first cursor of a config's schedule (`block` present iff `K > 1`):
/// the single definition behind `PreparedDriver::initial_cursor`.
pub fn initial_cursor_for(config: &DriverConfig) -> Cursor {
    Cursor {
        block: (!config.coordinate_block.is_one()).then_some(BlockCursor {
            o_vec: 0,
            o_read: 0,
            committed_moves: 0,
        }),
        ..Cursor::initial()
    }
}
/// Per-family cursors `(o_vec, o_read)` before batch `completed_batches`: a
/// pure function of the batch ordinal, `K`, the schedule and family sizes.
/// A vector batch takes `min(K, vectors left in its pass)`, a read batch
/// `min(K, reads left in its pass)`, so no block straddles a family pass.
pub fn family_cursors(
    completed_batches: u64,
    block: u32,
    schedule: SchedulePolicy,
    vectors: u64,
    reads: u64,
) -> Result<(u64, u64)> {
    let k = u64::from(block);
    if k == 0 || vectors == 0 || reads == 0 {
        return Err(invalid("block schedule needs K and both families nonzero"));
    }
    let vector_blocks = vectors.div_ceil(k);
    let read_blocks = reads.div_ceil(k);
    let (vector_batches, read_batches) = match schedule {
        SchedulePolicy::InterleavedReadEvery8V1 => (
            completed_batches - completed_batches / 8,
            completed_batches / 8,
        ),
        SchedulePolicy::CanonicalVectorThenReadV1 => {
            let cycle = vector_blocks + read_blocks;
            let (cycles, phase) = (completed_batches / cycle, completed_batches % cycle);
            let overflow = || invalid("block schedule overflow");
            (
                cycles
                    .checked_mul(vector_blocks)
                    .and_then(|v| v.checked_add(phase.min(vector_blocks)))
                    .ok_or_else(overflow)?,
                cycles
                    .checked_mul(read_blocks)
                    .and_then(|v| v.checked_add(phase.saturating_sub(vector_blocks)))
                    .ok_or_else(overflow)?,
            )
        }
    };
    let ordinal = |batches: u64, blocks: u64, size: u64| {
        (batches / blocks)
            .checked_mul(size)
            .and_then(|v| v.checked_add((batches % blocks) * k))
            .ok_or_else(|| invalid("block schedule overflow"))
    };
    Ok((
        ordinal(vector_batches, vector_blocks, vectors)?,
        ordinal(read_batches, read_blocks, reads)?,
    ))
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DriverCheckpoint {
    pub schema: String,
    pub lineage: Lineage,
    pub cursor: Cursor,
    pub model_sha256: String,
    /// Serialized device Snapshot, admitted before reload; no host model operation.
    pub model: Value,
}
#[derive(Debug)]
pub struct PreparedDriver {
    config: DriverConfig,
    train: PreparedCorpus,
    dev: PreparedCorpus,
    lineage: Lineage,
    coordinates: u32,
    schedule_cycle: u32,
    /// The pinned initial codebook, when the config names one.
    initial_table: Option<Vec<i32>>,
}

impl PreparedDriver {
    pub fn load(config: DriverConfig) -> Result<Self> {
        config.validate()?;
        let train = PreparedCorpus::load(&config.train, config.spec.vocabulary, &config.bos)?;
        let dev = PreparedCorpus::load(&config.dev, config.spec.vocabulary, &config.bos)?;
        Self::from_prepared(config, train, dev)
    }
    /// Construct from already SHA-admitted corpus metadata; checks full provenance.
    pub fn from_prepared(
        config: DriverConfig,
        train: PreparedCorpus,
        dev: PreparedCorpus,
    ) -> Result<Self> {
        config.validate()?;
        config
            .bos
            .validate_tokenizer(&config.tokenizer.read()?, config.spec.vocabulary)?;
        if train.binding().source != config.train
            || dev.binding().source != config.dev
            || train.binding().bos != config.bos
            || dev.binding().bos != config.bos
            || train.binding().vocabulary != config.spec.vocabulary
            || dev.binding().vocabulary != config.spec.vocabulary
        {
            return Err(invalid(
                "prepared corpus provenance differs from driver configuration",
            ));
        }
        let stages = BinderPlan::new(config.spec.context as usize)?.stages() as u32;
        let vectors = config
            .spec
            .blocks
            .checked_mul(config.spec.vocabulary)
            .ok_or_else(|| invalid("coordinate inventory overflow"))?;
        let read_parameters = config.read_spec.gate_count(config.spec.blocks, stages)?;
        let coordinates = vectors
            .checked_add(read_parameters)
            .ok_or_else(|| invalid("coordinate inventory overflow"))?;
        let schedule_cycle = match config.schedule {
            SchedulePolicy::CanonicalVectorThenReadV1 => coordinates,
            SchedulePolicy::InterleavedReadEvery8V1 => vectors
                .checked_add(config.spec.blocks * stages)
                .ok_or_else(|| invalid("schedule cycle overflow"))?,
        };
        let lineage = Lineage {
            schema: DRIVER_SCHEMA.into(),
            driver_source_sha256: driver_source_sha256(),
            config: config.clone(),
            train: train.binding().clone(),
            dev: dev.binding().clone(),
        };
        let initial_table = match &config.codebook_initialization {
            CodebookInitialization::SeededDeviceV1 => None,
            CodebookInitialization::PinnedTableV1 { table } => {
                Some(CodebookTable::load(table, config.spec)?)
            }
        };
        Ok(Self {
            config,
            train,
            dev,
            lineage,
            coordinates,
            schedule_cycle,
            initial_table,
        })
    }
    pub fn config(&self) -> &DriverConfig {
        &self.config
    }
    pub fn train(&self) -> &PreparedCorpus {
        &self.train
    }
    pub fn dev(&self) -> &PreparedCorpus {
        &self.dev
    }
    pub fn lineage(&self) -> &Lineage {
        &self.lineage
    }
    pub fn lineage_sha256(&self) -> Result<String> {
        Ok(corpus::sha256(
            &serde_json::to_vec(&self.lineage).map_err(|e| invalid(e.to_string()))?,
        ))
    }
    pub fn coordinate_count(&self) -> u32 {
        self.coordinates
    }
    pub fn schedule_cycle(&self) -> u32 {
        self.schedule_cycle
    }
    fn combine_gate_count(&self) -> Result<usize> {
        let stages = BinderPlan::new(self.config.spec.context as usize)?.stages();
        Ok(self.config.spec.blocks as usize * stages)
    }
    /// A declared policy refusal, independent of the genuine CUDA candidate costs.
    /// Combine gates and vector coordinates remain learnable in the frozen control.
    pub fn apply_refused_by_policy(&self, coordinate: Coordinate) -> Result<bool> {
        if self.config.relation_learning_policy.is_learn()
            || coordinate.kind != CoordinateKind::Gate
        {
            return Ok(false);
        }
        let stages = BinderPlan::new(self.config.spec.context as usize)?.stages() as u32;
        let reads = self
            .config
            .read_spec
            .gate_count(self.config.spec.blocks, stages)?;
        if coordinate.index >= reads {
            return Err(invalid("policy coordinate outside declared read inventory"));
        }
        Ok(coordinate.index as usize >= self.combine_gate_count()?)
    }
    fn validate_frozen_relation_gates(&self, gates: &[i32]) -> Result<()> {
        if self.config.relation_learning_policy.is_learn() {
            return Ok(());
        }
        let stages = BinderPlan::new(self.config.spec.context as usize)?.stages() as u32;
        let expected = self
            .config
            .read_spec
            .gate_count(self.config.spec.blocks, stages)? as usize;
        if gates.len() != expected
            || gates.iter().any(|trit| !(-1..=1).contains(trit))
            || gates[self.combine_gate_count()?..]
                .iter()
                .any(|trit| *trit != 0)
        {
            return Err(invalid(
                "frozen-zero Relation requires the complete gate inventory and every relation trit zero",
            ));
        }
        Ok(())
    }
    /// Schedule from the absolute batch ordinal. A cursor's wrapped phase alone
    /// is insufficient: a 16,404-batch interleaved cycle is not a multiple of 8.
    pub fn coordinate_at(&self, completed_batches: u64) -> Result<Coordinate> {
        if !self.config.coordinate_block.is_one() {
            return Err(invalid(
                "coordinate_block > 1 schedule is addressed through block_at",
            ));
        }
        if self.config.schedule == SchedulePolicy::CanonicalVectorThenReadV1 {
            return self.coordinate((completed_batches % u64::from(self.coordinates)) as u32);
        }
        let vectors = self.config.spec.vocabulary * self.config.spec.blocks;
        if completed_batches % 8 == 7 {
            let reads = self.coordinates - vectors;
            Ok(Coordinate {
                kind: if self.config.read_spec == ReadSpec::Route {
                    CoordinateKind::Route
                } else {
                    CoordinateKind::Gate
                },
                index: ((completed_batches / 8) % u64::from(reads)) as u32,
            })
        } else {
            Ok(Coordinate {
                kind: CoordinateKind::VectorBlock,
                index: ((completed_batches - completed_batches / 8) % u64::from(vectors)) as u32,
            })
        }
    }
    pub fn coordinate(&self, index: u32) -> Result<Coordinate> {
        if index >= self.coordinates {
            return Err(invalid("schedule coordinate outside declared inventory"));
        }
        let vectors = self.config.spec.vocabulary * self.config.spec.blocks;
        Ok(if index < vectors {
            Coordinate {
                kind: CoordinateKind::VectorBlock,
                index,
            }
        } else {
            Coordinate {
                kind: match self.config.read_spec {
                    ReadSpec::Route => CoordinateKind::Route,
                    ReadSpec::Flock { .. } | ReadSpec::Relation { .. } => CoordinateKind::Gate,
                },
                index: index - vectors,
            }
        })
    }
    pub fn validate_cursor(&self, cursor: &Cursor) -> Result<()> {
        let batches =
            (self.train.eligible_len() as u64).div_ceil(u64::from(self.config.batch_rows));
        if cursor.accepted_updates > cursor.completed_batches
            || cursor.corpus_pass != cursor.completed_batches / batches
            || cursor.next_target
                != (cursor.completed_batches % batches) * u64::from(self.config.batch_rows)
            || cursor.next_coordinate
                != (cursor.completed_batches % u64::from(self.schedule_cycle)) as u32
        {
            return Err(invalid(
                "resume cursor differs from declared corpus/schedule cycle",
            ));
        }
        let k = self.config.coordinate_block.get();
        match (&cursor.block, k > 1) {
            (None, false) => {}
            (Some(_), false) => {
                return Err(invalid("coordinate_block 1 cursor carries no block fields"));
            }
            (None, true) => {
                return Err(invalid("coordinate_block > 1 cursor requires block fields"));
            }
            (Some(block), true) => {
                if (block.o_vec, block.o_read) != self.family_cursors(cursor.completed_batches)? {
                    return Err(invalid(
                        "resume o_vec/o_read differ from the per-family block schedule",
                    ));
                }
                if block.committed_moves < cursor.accepted_updates
                    || cursor
                        .accepted_updates
                        .checked_mul(u64::from(k))
                        .is_some_and(|bound| block.committed_moves > bound)
                {
                    return Err(invalid(
                        "resume committed_moves outside accepted_updates..=K*accepted_updates",
                    ));
                }
            }
        }
        Ok(())
    }
    fn families(&self) -> (u64, u64) {
        let vectors = self.config.spec.vocabulary * self.config.spec.blocks;
        (u64::from(vectors), u64::from(self.coordinates - vectors))
    }
    fn family_cursors(&self, completed_batches: u64) -> Result<(u64, u64)> {
        let (vectors, reads) = self.families();
        family_cursors(
            completed_batches,
            self.config.coordinate_block.get(),
            self.config.schedule,
            vectors,
            reads,
        )
    }
    fn is_read_batch(&self, completed_batches: u64) -> bool {
        match self.config.schedule {
            SchedulePolicy::InterleavedReadEvery8V1 => completed_batches % 8 == 7,
            SchedulePolicy::CanonicalVectorThenReadV1 => {
                let (vectors, reads) = self.families();
                let k = u64::from(self.config.coordinate_block.get());
                let vector_blocks = vectors.div_ceil(k);
                completed_batches % (vector_blocks + reads.div_ceil(k)) >= vector_blocks
            }
        }
    }
    /// The block batch `cursor.completed_batches` visits. At `K = 1` it is
    /// exactly `[coordinate_at(completed_batches)]`.
    pub fn block_at(&self, cursor: &Cursor) -> Result<Block> {
        self.validate_cursor(cursor)?;
        let (vectors, reads) = self.families();
        let (o_vec, o_read) = self.family_cursors(cursor.completed_batches)?;
        let (kind, ordinal, size) = if self.is_read_batch(cursor.completed_batches) {
            let kind = if self.config.read_spec == ReadSpec::Route {
                CoordinateKind::Route
            } else {
                CoordinateKind::Gate
            };
            (kind, o_read, reads)
        } else {
            (CoordinateKind::VectorBlock, o_vec, vectors)
        };
        let start = ordinal % size;
        let end = size.min(start + u64::from(self.config.coordinate_block.get()));
        Ok(Block {
            kind,
            coordinates: (start..end)
                .map(|index| {
                    Coordinate {
                        kind,
                        index: index as u32,
                    }
                    .parameter()
                })
                .collect(),
        })
    }
    /// Advance one block batch: `completed_batches` +1, the batch's family
    /// cursor by its block length, `accepted_updates` by the commit and
    /// `committed_moves` by `moves` (0 without a commit, 1..=length with one).
    pub fn advance_block(&self, cursor: &Cursor, committed: bool, moves: u64) -> Result<Cursor> {
        let length = self.block_at(cursor)?.coordinates.len() as u64;
        if committed != (moves > 0) || moves > length {
            return Err(invalid(
                "block commit needs 1..=block-length moves, and no commit none",
            ));
        }
        let mut next = self.advance_unchecked(cursor, committed)?;
        if let Some(block) = cursor.block {
            let (o_vec, o_read) = self.family_cursors(next.completed_batches)?;
            next.block = Some(BlockCursor {
                o_vec,
                o_read,
                committed_moves: block
                    .committed_moves
                    .checked_add(moves)
                    .ok_or_else(|| invalid("move cursor overflow"))?,
            });
        }
        Ok(next)
    }
    /// The first cursor of this driver's schedule (`block` present iff `K > 1`).
    pub fn initial_cursor(&self) -> Cursor {
        initial_cursor_for(&self.config)
    }
    pub fn batch(&self, cursor: &Cursor) -> Result<CorpusBatch> {
        self.validate_cursor(cursor)?;
        self.train.batch(
            cursor.next_target,
            self.config.spec.context,
            self.config.batch_rows,
        )
    }
    pub fn advance(&self, cursor: &Cursor, applied: bool) -> Result<Cursor> {
        self.validate_cursor(cursor)?;
        if !self.config.coordinate_block.is_one() {
            return Err(invalid(
                "coordinate_block > 1 advances through advance_block",
            ));
        }
        self.advance_unchecked(cursor, applied)
    }
    fn advance_unchecked(&self, cursor: &Cursor, applied: bool) -> Result<Cursor> {
        let completed = cursor
            .completed_batches
            .checked_add(1)
            .ok_or_else(|| invalid("batch cursor overflow"))?;
        let batches =
            (self.train.eligible_len() as u64).div_ceil(u64::from(self.config.batch_rows));
        Ok(Cursor {
            completed_batches: completed,
            accepted_updates: cursor
                .accepted_updates
                .checked_add(u64::from(applied))
                .ok_or_else(|| invalid("update cursor overflow"))?,
            corpus_pass: completed / batches,
            next_target: (completed % batches) * u64::from(self.config.batch_rows),
            next_coordinate: (completed % u64::from(self.schedule_cycle)) as u32,
            block: None,
        })
    }
    pub fn plan(&self) -> Result<Value> {
        let plan = BinderPlan::new(self.config.spec.context as usize)?;
        let read_parameters = self
            .config
            .read_spec
            .gate_count(self.config.spec.blocks, plan.stages() as u32)?;
        let mut record = json!({"schema":DRIVER_SCHEMA,"lineage":self.lineage,"lineage_sha256":self.lineage_sha256()?,"seed":self.config.seed,"schedule":{"policy":self.config.schedule,"parameters":self.coordinates,"vector_parameters":self.config.spec.vocabulary*self.config.spec.blocks,"read_parameters":read_parameters,"one_coordinate_per_batch":true,"corpus_order":"ascending response-masked target positions; wraps after complete coverage; common across seeds","initial_cursor":Cursor::initial()},"train_batches_per_pass":(self.train.eligible_len() as u64).div_ceil(u64::from(self.config.batch_rows)),"dev_batches":(self.dev.eligible_len() as u64).div_ceil(u64::from(self.config.batch_rows)),"training":"HELD","applied_updates":0,"throughput":"UNMEASURED"});
        if self.config.schedule == SchedulePolicy::InterleavedReadEvery8V1 {
            record["schedule"]["cycle"] = json!(self.schedule_cycle);
            record["schedule"]["coordinate_basis"] = json!("absolute completed_batches");
        }
        Ok(record)
    }
    pub fn validate_checkpoint(&self, checkpoint: &DriverCheckpoint) -> Result<()> {
        if checkpoint.schema != CHECKPOINT_SCHEMA || checkpoint.lineage != self.lineage {
            return Err(invalid(
                "checkpoint source/config/data/mask/tokenizer/role-policy lineage mismatch",
            ));
        }
        self.validate_cursor(&checkpoint.cursor)?;
        // G-Init: before any accepted update, a pinned lineage carries exactly
        // its pinned codebook (no substituted or seeded table under its lineage).
        if let Some(table) = &self.initial_table {
            if checkpoint.cursor.accepted_updates == 0
                && checkpoint.model.get("table") != Some(&serde_json::json!(table))
            {
                return Err(invalid(
                    "pinned-codebook checkpoint before any update differs from its pinned table",
                ));
            }
        }
        corpus::validate_sha256(&checkpoint.model_sha256)?;
        if corpus::sha256(
            &serde_json::to_vec(&checkpoint.model).map_err(|e| invalid(e.to_string()))?,
        ) != checkpoint.model_sha256
        {
            return Err(invalid("checkpoint model storage hash mismatch"));
        }
        let spec: Spec = serde_json::from_value(
            checkpoint
                .model
                .get("spec")
                .cloned()
                .ok_or_else(|| invalid("checkpoint missing model spec"))?,
        )
        .map_err(|e| invalid(e.to_string()))?;
        let mode: ReadSpec = serde_json::from_value(
            checkpoint
                .model
                .get("read_spec")
                .cloned()
                .ok_or_else(|| invalid("checkpoint missing read mode"))?,
        )
        .map_err(|e| invalid(e.to_string()))?;
        let objective: ObjectivePolicy = serde_json::from_value(
            checkpoint
                .model
                .get("objective_policy")
                .cloned()
                .unwrap_or_else(|| json!(ObjectivePolicy::default())),
        )
        .map_err(|error| invalid(error.to_string()))?;
        let initialization: ReadInitialization = serde_json::from_value(
            checkpoint
                .model
                .get("read_initialization")
                .cloned()
                .unwrap_or_else(|| json!(ReadInitialization::default())),
        )
        .map_err(|error| invalid(error.to_string()))?;
        let normalization: NormalizationMode = serde_json::from_value(
            checkpoint
                .model
                .get("normalization_mode")
                .cloned()
                .unwrap_or_else(|| json!(NormalizationMode::default())),
        )
        .map_err(|error| invalid(error.to_string()))?;
        let position: Option<PositionConfig> = serde_json::from_value(
            checkpoint
                .model
                .get("position")
                .cloned()
                .unwrap_or(Value::Null),
        )
        .map_err(|error| invalid(error.to_string()))?;
        if spec != self.config.spec
            || objective != self.config.objective_policy
            || initialization != self.config.read_initialization
            || normalization != self.config.normalization_mode
            || position != self.config.position
            || mode != self.config.read_spec
            || checkpoint.model.get("seed").and_then(Value::as_u64)
                != Some(u64::from(self.config.seed))
            || checkpoint.model.get("epoch").and_then(Value::as_u64)
                != Some(checkpoint.cursor.accepted_updates)
            || checkpoint.model.get("schema").and_then(Value::as_str) != Some(crate::SCHEMA)
        {
            return Err(invalid(
                "checkpoint model spec/read/seed/epoch differs from declared driver state",
            ));
        }
        if !self.config.relation_learning_policy.is_learn() {
            let gates: Vec<i32> = serde_json::from_value(
                checkpoint
                    .model
                    .get("gates")
                    .cloned()
                    .ok_or_else(|| invalid("frozen-zero checkpoint missing gates"))?,
            )
            .map_err(|error| invalid(error.to_string()))?;
            self.validate_frozen_relation_gates(&gates)?;
        }
        for name in ["source_sha256", "geometry_sha256", "kernel_sha256"] {
            corpus::validate_sha256(
                checkpoint
                    .model
                    .get(name)
                    .and_then(Value::as_str)
                    .ok_or_else(|| invalid(format!("checkpoint missing {name}")))?,
            )?;
        }
        Ok(())
    }
}

/// Bind two already serialized snapshot observations to the exact stored Value.
/// The caller obtains both typed hashes with the real Snapshot serializer;
/// this helper handles storage integrity only, without evaluating a model.
#[cfg(any(feature = "cuda", test))]
fn bind_evaluation_storage_domain(
    stored: &Value,
    legacy_storage: &str,
    stored_snapshot: &str,
    actual_snapshot: &str,
) -> Result<String> {
    for hash in [legacy_storage, stored_snapshot, actual_snapshot] {
        corpus::validate_sha256(hash)?;
    }
    let storage = corpus::sha256(&serde_json::to_vec(stored).map_err(|e| invalid(e.to_string()))?);
    if storage != legacy_storage || stored_snapshot != actual_snapshot {
        return Err(invalid(
            "evaluation stored checkpoint or typed snapshot differs from model",
        ));
    }
    Ok(storage)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluation_storage_binding_preserves_explicit_default_without_changing_typed_domain(
    ) -> Result<()> {
        // Metadata serializer fixture, not the CUDA-gated Snapshot type or a
        // model. Both forms decode identically and omit the default in B.
        #[derive(serde::Serialize, serde::Deserialize)]
        struct Metadata {
            epoch: u64,
            #[serde(default, skip_serializing_if = "is_false")]
            default_flag: bool,
        }
        fn is_false(flag: &bool) -> bool {
            !*flag
        }
        let canonical = json!({"epoch":0});
        let explicit = json!({"epoch":0,"default_flag":false});
        let typed_hash = |stored: &Value| -> Result<String> {
            let typed: Metadata =
                serde_json::from_value(stored.clone()).map_err(|e| invalid(e.to_string()))?;
            Ok(corpus::sha256(
                &serde_json::to_vec(&typed).map_err(|e| invalid(e.to_string()))?,
            ))
        };
        let canonical_a =
            corpus::sha256(&serde_json::to_vec(&canonical).map_err(|e| invalid(e.to_string()))?);
        let explicit_a =
            corpus::sha256(&serde_json::to_vec(&explicit).map_err(|e| invalid(e.to_string()))?);
        let b = typed_hash(&canonical)?;
        assert_ne!(canonical_a, explicit_a);
        assert_eq!(typed_hash(&explicit)?, b);
        assert_eq!(
            bind_evaluation_storage_domain(&explicit, &explicit_a, &b, &b)?,
            explicit_a
        );
        assert!(bind_evaluation_storage_domain(&explicit, &canonical_a, &b, &b).is_err());
        assert!(
            bind_evaluation_storage_domain(&explicit, &explicit_a, &b, &"f".repeat(64)).is_err()
        );
        assert!(bind_evaluation_storage_domain(&explicit, &explicit_a, "invalid", &b).is_err());
        Ok(())
    }
    #[test]
    fn pilot_slice_reports_masked_reference_share_without_reordering() -> Result<()> {
        let mut counts = vec![0; 4096];
        counts[0] = 2;
        counts[409] = 3;
        counts[410] = 100;
        let report = pilot_slice_metadata(&counts, 105)?;
        assert_eq!(report["masked_train_target_sum"], 5);
        assert_eq!(report["zero_count_tokens"], 408);
        assert_eq!(
            report["combined_share"],
            json!({"numerator":5,"denominator":105})
        );
        assert_eq!(report["target_counts_by_token_id"][409], 3);
        assert!(pilot_slice_metadata(&counts, 5).is_err());
        assert!(pilot_slice_metadata(&[u64::MAX, 1], 0).is_err());
        assert_eq!(pilot_slice_metadata(&[0; 410], 0)?["status"], "UNAVAILABLE");
        Ok(())
    }
    #[test]
    fn snapshot_metadata_distinguishes_interior_patterns_from_writer_cells() -> Result<()> {
        // Literal storage fixture, not model initialization or a host forward.
        let a = [2, 2, 0, 0, 0, 0, 0, 0];
        let b = [-2, -2, 0, 0, 0, 0, 0, 0];
        let table: Vec<i32> = [a, a, a, a, a, b, a, a, b, b, a, a]
            .into_iter()
            .flatten()
            .collect();
        let mut snapshot =
            json!({"spec":{"vocabulary":3,"blocks":4,"context":1,"kept_cells":2},"table":table});
        let report = snapshot_metadata(&snapshot)?;
        assert_eq!(report["distinct_full_vector_patterns"], 3);
        assert_eq!(report["occupied_writer_cells"], 2);
        snapshot["table"]
            .as_array_mut()
            .ok_or_else(|| invalid("fixture table"))?
            .pop();
        assert!(snapshot_metadata(&snapshot).is_err());
        Ok(())
    }
    fn metadata_fixture(seed: u32, read_spec: ReadSpec) -> Result<PreparedDriver> {
        metadata_schedule_fixture(
            seed,
            read_spec,
            Spec {
                vocabulary: 10,
                blocks: 2,
                context: 3,
                kept_cells: 2,
            },
            SchedulePolicy::CanonicalVectorThenReadV1,
        )
    }
    fn metadata_schedule_fixture(
        seed: u32,
        read_spec: ReadSpec,
        spec: Spec,
        schedule: SchedulePolicy,
    ) -> Result<PreparedDriver> {
        fn corpus(
            split: Split,
            ids: &[u16],
            mask: &[u8],
            eligible: u64,
            vocabulary: u32,
        ) -> Result<PreparedCorpus> {
            let mut bytes = vec![0; 64];
            bytes[..4].copy_from_slice(b"UORT");
            bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
            bytes[8..16].copy_from_slice(&(ids.len() as u64).to_le_bytes());
            bytes[16..20].copy_from_slice(&vocabulary.to_le_bytes());
            for id in ids {
                bytes.extend_from_slice(&id.to_le_bytes());
            }
            let source = CorpusSource {
                split,
                tokens: FilePin {
                    path: format!("{split:?}.uort").into(),
                    sha256: corpus::sha256(&bytes),
                },
                mask: FilePin {
                    path: format!("{split:?}.mask").into(),
                    sha256: corpus::sha256(mask),
                },
                expected_payload_tokens: ids.len() as u64,
                expected_eligible_targets: eligible,
            };
            PreparedCorpus::from_bytes(
                source,
                &bytes,
                mask,
                vocabulary,
                BosIdentity {
                    token: "<|bos|>".into(),
                    id: 0,
                },
            )
        }
        let train = corpus(
            Split::Train,
            &[0, 9, 9, 3, 3, 4, 0, 9, 4],
            &[0, 0, 0, 1, 1, 1, 0, 0, 1],
            4,
            spec.vocabulary,
        )?;
        let dev = corpus(Split::Dev, &[0, 3, 4], &[0, 1, 1], 2, spec.vocabulary)?;
        let config = DriverConfig {
            objective_policy: ObjectivePolicy::NearestAddressMarginV1,
            read_initialization: ReadInitialization::ZeroGatesV1,
            normalization_mode: NormalizationMode::ScaleMatchedSnapV1,
            position: None,
            relation_learning_policy: RelationLearningPolicy::LearnV1,
            counterfactual_path: CounterfactualPath::LanesV1,
            codebook_initialization: CodebookInitialization::SeededDeviceV1,
            coordinate_block: CoordinateBlock::ONE,
            schema: DRIVER_SCHEMA.into(),
            spec,
            read_spec,
            seed,
            batch_rows: 3,
            train: train.binding().source.clone(),
            dev: dev.binding().source.clone(),
            tokenizer: FilePin {
                path: "inline-tokenizer-metadata.json".into(),
                sha256: corpus::sha256(br#"{"model":{"vocab":{"<|bos|>":0}}}"#),
            },
            bos: BosIdentity {
                token: "<|bos|>".into(),
                id: 0,
            },
            schedule,
        };
        // Inline, already admitted metadata fixture; no file/model initialization.
        config.validate()?;
        let stages = BinderPlan::new(spec.context as usize)?.stages() as u32;
        let vectors = spec.vocabulary * spec.blocks;
        let coordinates = vectors + read_spec.gate_count(spec.blocks, stages)?;
        let schedule_cycle = match schedule {
            SchedulePolicy::CanonicalVectorThenReadV1 => coordinates,
            SchedulePolicy::InterleavedReadEvery8V1 => vectors + spec.blocks * stages,
        };
        let lineage = Lineage {
            schema: DRIVER_SCHEMA.into(),
            driver_source_sha256: driver_source_sha256(),
            config: config.clone(),
            train: train.binding().clone(),
            dev: dev.binding().clone(),
        };
        Ok(PreparedDriver {
            config,
            train,
            dev,
            lineage,
            coordinates,
            schedule_cycle,
            initial_table: None,
        })
    }
    #[test]
    fn complete_coordinate_cycle_and_resume_are_metadata() -> Result<()> {
        for mode in [
            ReadSpec::Route,
            ReadSpec::Flock { kept: 2 },
            ReadSpec::Relation { kept: 2, sweeps: 1 },
        ] {
            let driver = metadata_fixture(1, mode)?;
            let paired = metadata_fixture(2, mode)?;
            let read_count = mode.gate_count(2, 2)?;
            let total = 20 + read_count;
            assert_eq!(driver.coordinate_count(), total);
            let mut cursor = Cursor::initial();
            let mut coordinates = Vec::new();
            for step in 0..total {
                let coordinate = driver.coordinate(cursor.next_coordinate)?;
                coordinates.push(coordinate);
                let batch = driver.batch(&cursor)?;
                assert_eq!(
                    batch.target_positions,
                    if step % 2 == 0 {
                        vec![3, 4, 5]
                    } else {
                        vec![8]
                    }
                );
                assert_eq!(paired.batch(&cursor)?, batch); // seed affects initialization, not corpus order.
                let bytes = serde_json::to_vec(&cursor).map_err(|e| invalid(e.to_string()))?;
                let resumed: Cursor =
                    serde_json::from_slice(&bytes).map_err(|e| invalid(e.to_string()))?;
                assert_eq!(driver.batch(&resumed)?, batch);
                assert_eq!(
                    driver.advance(&resumed, false)?,
                    driver.advance(&cursor, false)?
                );
                cursor = driver.advance(&cursor, false)?;
            }
            assert_eq!(
                coordinates[..20],
                (0..20)
                    .map(|index| Coordinate {
                        kind: CoordinateKind::VectorBlock,
                        index
                    })
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                coordinates[20..],
                (0..read_count)
                    .map(|index| Coordinate {
                        kind: if mode == ReadSpec::Route {
                            CoordinateKind::Route
                        } else {
                            CoordinateKind::Gate
                        },
                        index
                    })
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                cursor,
                Cursor {
                    completed_batches: u64::from(total),
                    accepted_updates: 0,
                    corpus_pass: u64::from(total) / 2,
                    next_target: 0,
                    next_coordinate: 0,
                    block: None
                }
            );
            let first = driver.batch(&Cursor::initial())?;
            assert_eq!(first.targets, [3, 3, 4]);
            assert_eq!(
                first
                    .windows
                    .iter()
                    .map(|w| w.iter().map(|id| id.token).collect::<Vec<_>>())
                    .collect::<Vec<_>>(),
                [vec![0, 9, 9], vec![9, 9, 3], vec![9, 3, 3]]
            );
            let tail = driver.batch(&driver.advance(&Cursor::initial(), false)?)?;
            assert_eq!(tail.targets, [4]);
            assert_eq!(
                tail.windows[0]
                    .iter()
                    .map(|id| id.token)
                    .collect::<Vec<_>>(),
                [0, 9]
            );
            assert!(matches!(
                request_corpus_update_admission(&driver),
                Err(Error::TrainingHeld)
            ));
        }
        Ok(())
    }
    #[test]
    fn pinned_lineage_refuses_a_substituted_table_before_any_update() -> Result<()> {
        let mut driver = metadata_fixture(1, ReadSpec::Route)?;
        let pinned: Vec<i32> = vec![7; 16];
        driver.initial_table = Some(pinned.clone());
        // Storage-header fixture only; no GPU model or numeric reference is constructed.
        let header = |table: &[i32]| json!({"schema":crate::SCHEMA,"spec":driver.config.spec,"read_spec":driver.config.read_spec,"seed":1,"epoch":0,"table":table,"source_sha256":"0".repeat(64),"kernel_sha256":"1".repeat(64),"geometry_sha256":"2".repeat(64)});
        let checkpoint = |model: Value, cursor: Cursor| -> Result<DriverCheckpoint> {
            Ok(DriverCheckpoint {
                schema: CHECKPOINT_SCHEMA.into(),
                lineage: driver.lineage.clone(),
                cursor,
                model_sha256: corpus::sha256(
                    &serde_json::to_vec(&model).map_err(|e| invalid(e.to_string()))?,
                ),
                model,
            })
        };
        driver.validate_checkpoint(&checkpoint(header(&pinned), Cursor::initial())?)?;
        let mut swapped = pinned.clone();
        swapped[0] = 8;
        let refused = driver.validate_checkpoint(&checkpoint(header(&swapped), Cursor::initial())?);
        assert!(refused.is_err_and(|e| e.to_string().contains("differs from its pinned table")));
        // A missing table is no substitute either.
        let mut absent = header(&pinned);
        absent.as_object_mut().map(|m| m.remove("table"));
        assert!(driver
            .validate_checkpoint(&checkpoint(absent, Cursor::initial())?)
            .is_err());
        // A malformed pin hash is refused at config validation.
        let mut config = driver.config.clone();
        config.codebook_initialization = CodebookInitialization::PinnedTableV1 {
            table: FilePin {
                path: std::path::PathBuf::from("/codebook.json"),
                sha256: "not-a-digest".into(),
            },
        };
        assert!(config.validate().is_err());
        Ok(())
    }
    #[test]
    fn checkpoint_metadata_refuses_swapped_config_data_and_cursor() -> Result<()> {
        let driver = metadata_fixture(1, ReadSpec::Route)?;
        // Storage-header fixture only; no GPU model or numeric reference is constructed.
        let model = json!({"schema":crate::SCHEMA,"spec":driver.config.spec,"read_spec":driver.config.read_spec,"seed":1,"epoch":0,"source_sha256":"0".repeat(64),"kernel_sha256":"1".repeat(64),"geometry_sha256":"2".repeat(64)});
        let checkpoint = DriverCheckpoint {
            schema: CHECKPOINT_SCHEMA.into(),
            lineage: driver.lineage.clone(),
            cursor: Cursor::initial(),
            model_sha256: corpus::sha256(
                &serde_json::to_vec(&model).map_err(|e| invalid(e.to_string()))?,
            ),
            model,
        };
        driver.validate_checkpoint(&checkpoint)?;
        let encoded_legacy =
            serde_json::to_value(&driver.config).map_err(|error| invalid(error.to_string()))?;
        assert!(encoded_legacy.get("objective_policy").is_none());
        let mut rank_config = driver.config.clone();
        rank_config.objective_policy = ObjectivePolicy::OccupiedCellRankHingeV1;
        let encoded_rank =
            serde_json::to_value(&rank_config).map_err(|error| invalid(error.to_string()))?;
        assert_eq!(
            encoded_rank["objective_policy"],
            "occupied_cell_rank_hinge_v1"
        );
        assert_ne!(encoded_legacy, encoded_rank);
        let mut wrong_objective = checkpoint.clone();
        wrong_objective.model["objective_policy"] = json!(ObjectivePolicy::OccupiedCellRankHingeV1);
        wrong_objective.model_sha256 = corpus::sha256(
            &serde_json::to_vec(&wrong_objective.model)
                .map_err(|error| invalid(error.to_string()))?,
        );
        assert!(driver.validate_checkpoint(&wrong_objective).is_err());
        let mut wrong = checkpoint.clone();
        wrong.cursor.next_coordinate = 1;
        assert!(driver.validate_checkpoint(&wrong).is_err());
        let mut wrong = checkpoint.clone();
        wrong.lineage.config.seed = 2;
        assert!(driver.validate_checkpoint(&wrong).is_err());
        let mut wrong = checkpoint.clone();
        wrong.lineage.train.source.mask.sha256 = "a".repeat(64);
        assert!(driver.validate_checkpoint(&wrong).is_err());
        let mut wrong = checkpoint.clone();
        wrong.model["seed"] = json!(2);
        wrong.model_sha256 =
            corpus::sha256(&serde_json::to_vec(&wrong.model).map_err(|e| invalid(e.to_string()))?);
        assert!(driver.validate_checkpoint(&wrong).is_err());
        let mut wrong = checkpoint.clone();
        wrong.model_sha256 = "f".repeat(64);
        assert!(driver.validate_checkpoint(&wrong).is_err());
        assert!(metadata_fixture(2, ReadSpec::Route)?
            .validate_checkpoint(&checkpoint)
            .is_err());
        let boundary = Cursor {
            completed_batches: u64::MAX,
            accepted_updates: 0,
            corpus_pass: u64::MAX / 2,
            next_target: 3,
            next_coordinate: (u64::MAX % 24) as u32,
            block: None,
        };
        driver.validate_cursor(&boundary)?;
        assert!(driver.advance(&boundary, false).is_err());
        assert!(driver.advance(&boundary, true).is_err());
        Ok(())
    }
    #[test]
    fn interleaved_schedule_24_step_fixture_and_resume() -> Result<()> {
        let spec = Spec {
            vocabulary: 10,
            blocks: 2,
            context: 3,
            kept_cells: 2,
        };
        for mode in [
            ReadSpec::Route,
            ReadSpec::Flock { kept: 2 },
            ReadSpec::Relation { kept: 2, sweeps: 1 },
        ] {
            let driver =
                metadata_schedule_fixture(1, mode, spec, SchedulePolicy::InterleavedReadEvery8V1)?;
            assert_eq!(driver.schedule_cycle(), 24);
            assert_eq!(
                driver.coordinate_count(),
                if matches!(mode, ReadSpec::Relation { .. }) {
                    34
                } else {
                    24
                }
            );
            let mut cursor = Cursor::initial();
            let mut vectors = Vec::new();
            let mut reads = Vec::new();
            for step in 0..48 {
                assert_eq!(cursor.completed_batches, step);
                let coordinate = driver.coordinate_at(cursor.completed_batches)?;
                if step < 24 && coordinate.kind == CoordinateKind::VectorBlock {
                    vectors.push(coordinate.index);
                }
                if coordinate.kind != CoordinateKind::VectorBlock {
                    reads.push(coordinate.index);
                }
                let resumed: Cursor = serde_json::from_slice(
                    &serde_json::to_vec(&cursor).map_err(|e| invalid(e.to_string()))?,
                )
                .map_err(|e| invalid(e.to_string()))?;
                driver.validate_cursor(&resumed)?;
                assert_eq!(driver.coordinate_at(resumed.completed_batches)?, coordinate);
                assert_eq!(
                    driver.advance(&resumed, false)?,
                    driver.advance(&cursor, false)?
                );
                cursor = driver.advance(&cursor, false)?;
                if step == 23 {
                    assert_eq!(cursor.next_coordinate, 0);
                    assert_eq!(cursor.corpus_pass, 12);
                    assert_eq!(cursor.next_target, 0);
                }
            }
            assert_eq!(
                vectors,
                (0..20).chain(std::iter::once(0)).collect::<Vec<_>>()
            );
            assert_eq!(
                reads,
                if matches!(mode, ReadSpec::Relation { .. }) {
                    vec![0, 1, 2, 3, 4, 5]
                } else {
                    vec![0, 1, 2, 3, 0, 1]
                }
            );
        }
        Ok(())
    }
    #[test]
    fn interleaved_schedule_c192_counts_frequencies_and_absolute_wrap() -> Result<()> {
        let spec = Spec {
            vocabulary: 4096,
            blocks: 4,
            context: 192,
            kept_cells: 8,
        };
        for mode in [
            ReadSpec::Route,
            ReadSpec::Flock { kept: 8 },
            ReadSpec::Relation { kept: 8, sweeps: 1 },
        ] {
            let driver =
                metadata_schedule_fixture(1, mode, spec, SchedulePolicy::InterleavedReadEvery8V1)?;
            assert_eq!(driver.schedule_cycle(), 16404);
            let read_count = if matches!(mode, ReadSpec::Relation { .. }) {
                45
            } else {
                20
            };
            assert_eq!(driver.coordinate_count(), 16384 + read_count);
            let mut read_frequencies = vec![0; read_count as usize];
            let mut vector_indices = Vec::new();
            let mut reads = 0;
            for step in 0..16404 {
                let coordinate = driver.coordinate_at(step)?;
                if coordinate.kind == CoordinateKind::VectorBlock {
                    vector_indices.push(coordinate.index);
                } else {
                    assert_eq!(
                        coordinate.kind,
                        if mode == ReadSpec::Route {
                            CoordinateKind::Route
                        } else {
                            CoordinateKind::Gate
                        }
                    );
                    read_frequencies[coordinate.index as usize] += 1;
                    reads += 1;
                }
                let expected = match step + 1 {
                    4101 => Some((3589, 512)),
                    8202 => Some((7177, 1025)),
                    12303 => Some((10766, 1537)),
                    16404 => Some((14354, 2050)),
                    _ => None,
                };
                if let Some(expected) = expected {
                    assert_eq!((vector_indices.len(), reads), expected);
                }
            }
            assert_eq!(vector_indices, (0..14354).collect::<Vec<_>>());
            for (index, count) in read_frequencies.iter().enumerate() {
                assert_eq!(
                    *count,
                    if read_count == 45 {
                        if index < 25 {
                            46
                        } else {
                            45
                        }
                    } else if index < 10 {
                        103
                    } else {
                        102
                    }
                );
            }
            if read_count == 45 {
                assert_eq!(read_frequencies[..20].iter().sum::<usize>(), 920);
                assert_eq!(read_frequencies[20..].iter().sum::<usize>(), 1130);
            }
            // The cycle wraps the cursor but MUST NOT reset the absolute interleaver.
            let cursor = Cursor {
                completed_batches: 16404,
                accepted_updates: 0,
                corpus_pass: 8202,
                next_target: 0,
                next_coordinate: 0,
                block: None,
            };
            driver.validate_cursor(&cursor)?;
            assert_eq!(
                driver.coordinate_at(cursor.completed_batches)?,
                Coordinate {
                    kind: CoordinateKind::VectorBlock,
                    index: 14354
                }
            );
            assert_ne!(
                driver.coordinate_at(cursor.completed_batches)?,
                driver.coordinate_at(u64::from(cursor.next_coordinate))?
            );
            assert_eq!(driver.coordinate_at(16407)?.index, 2050 % read_count);
            let mut wrong = cursor;
            wrong.next_coordinate = 1;
            assert!(driver.validate_cursor(&wrong).is_err());
        }
        Ok(())
    }
    #[test]
    fn initialization_defaults_and_cross_schedule_lineage_refuse() -> Result<()> {
        let canonical = metadata_fixture(1, ReadSpec::Relation { kept: 2, sweeps: 1 })?;
        let encoded =
            serde_json::to_value(&canonical.config).map_err(|e| invalid(e.to_string()))?;
        assert!(encoded.get("read_initialization").is_none());
        assert!(encoded.get("normalization_mode").is_none());
        let decoded: DriverConfig =
            serde_json::from_value(encoded.clone()).map_err(|e| invalid(e.to_string()))?;
        assert_eq!(decoded, canonical.config);
        let model = json!({"schema":crate::SCHEMA,"spec":canonical.config.spec,"read_spec":canonical.config.read_spec,"seed":1,"epoch":0,"source_sha256":"0".repeat(64),"kernel_sha256":"1".repeat(64),"geometry_sha256":"2".repeat(64)});
        let mut checkpoint = DriverCheckpoint {
            schema: CHECKPOINT_SCHEMA.into(),
            lineage: canonical.lineage.clone(),
            cursor: Cursor::initial(),
            model_sha256: corpus::sha256(
                &serde_json::to_vec(&model).map_err(|e| invalid(e.to_string()))?,
            ),
            model,
        };
        canonical.validate_checkpoint(&checkpoint)?;
        let mut interleaved = metadata_schedule_fixture(
            1,
            canonical.config.read_spec,
            canonical.config.spec,
            SchedulePolicy::InterleavedReadEvery8V1,
        )?;
        assert!(interleaved.validate_checkpoint(&checkpoint).is_err());
        checkpoint.lineage = interleaved.lineage.clone();
        interleaved.validate_checkpoint(&checkpoint)?;
        assert!(canonical.validate_checkpoint(&checkpoint).is_err());
        let mut active = canonical.config.clone();
        active.read_initialization = ReadInitialization::ActiveLeafV1;
        assert!(active.validate().is_err());
        active.schedule = SchedulePolicy::InterleavedReadEvery8V1;
        active.validate()?;
        active.read_spec = ReadSpec::Route;
        assert!(active.validate().is_err());
        active.read_spec = ReadSpec::Flock { kept: 2 };
        active.validate()?;
        interleaved.config.read_initialization = ReadInitialization::ActiveLeafV1;
        interleaved.lineage.config = interleaved.config.clone();
        checkpoint.lineage = interleaved.lineage.clone();
        assert!(
            interleaved.validate_checkpoint(&checkpoint).is_err(),
            "missing snapshot identity defaults to zero and cannot admit ActiveLeaf"
        );
        checkpoint.model["read_initialization"] = json!(ReadInitialization::ActiveLeafV1);
        checkpoint.model_sha256 = corpus::sha256(
            &serde_json::to_vec(&checkpoint.model).map_err(|e| invalid(e.to_string()))?,
        );
        interleaved.validate_checkpoint(&checkpoint)?;
        assert_eq!(
            serde_json::to_value(&interleaved.config).map_err(|e| invalid(e.to_string()))?
                ["read_initialization"],
            "active_leaf_v1"
        );
        let route_base = metadata_fixture(1, ReadSpec::Route)?;
        let base_lineage = route_base.lineage_sha256()?;
        for control in [
            ReadInitialization::RouteBypassV1,
            ReadInitialization::RoutePermutedActionsV1,
        ] {
            let mut route_control = metadata_fixture(1, ReadSpec::Route)?;
            route_control.config.read_initialization = control;
            route_control.config.validate()?;
            route_control.lineage.config = route_control.config.clone();
            assert_ne!(route_control.lineage_sha256()?, base_lineage);
            let mut bad_mode = route_control.config.clone();
            bad_mode.read_spec = ReadSpec::Flock { kept: 2 };
            assert!(bad_mode.validate().is_err());
            let mut bad_schedule = route_control.config.clone();
            bad_schedule.schedule = SchedulePolicy::InterleavedReadEvery8V1;
            assert!(bad_schedule.validate().is_err());
        }
        Ok(())
    }
    #[test]
    fn normalization_mode_round_trips_and_checkpoint_mismatch_refuses() -> Result<()> {
        let canonical = metadata_fixture(1, ReadSpec::Relation { kept: 2, sweeps: 1 })?;
        let model = json!({"schema":crate::SCHEMA,"spec":canonical.config.spec,"read_spec":canonical.config.read_spec,"seed":1,"epoch":0,"source_sha256":"0".repeat(64),"kernel_sha256":"1".repeat(64),"geometry_sha256":"2".repeat(64)});
        let mut checkpoint = DriverCheckpoint {
            schema: CHECKPOINT_SCHEMA.into(),
            lineage: canonical.lineage.clone(),
            cursor: Cursor::initial(),
            model_sha256: corpus::sha256(
                &serde_json::to_vec(&model).map_err(|e| invalid(e.to_string()))?,
            ),
            model,
        };
        canonical.validate_checkpoint(&checkpoint)?;
        let mut control = metadata_fixture(1, ReadSpec::Relation { kept: 2, sweeps: 1 })?;
        control.config.normalization_mode = NormalizationMode::UnitSnapControlV1;
        control.config.validate()?;
        control.lineage.config = control.config.clone();
        let encoded = serde_json::to_value(&control.config).map_err(|e| invalid(e.to_string()))?;
        assert_eq!(encoded["normalization_mode"], "unit_snap_control_v1");
        let decoded: DriverConfig =
            serde_json::from_value(encoded).map_err(|e| invalid(e.to_string()))?;
        assert_eq!(decoded, control.config);
        checkpoint.lineage = control.lineage.clone();
        assert!(
            control.validate_checkpoint(&checkpoint).is_err(),
            "a checkpoint without the field is scale-matched and cannot admit the control"
        );
        checkpoint.model["normalization_mode"] = json!(NormalizationMode::UnitSnapControlV1);
        checkpoint.model_sha256 = corpus::sha256(
            &serde_json::to_vec(&checkpoint.model).map_err(|e| invalid(e.to_string()))?,
        );
        control.validate_checkpoint(&checkpoint)?;
        checkpoint.lineage = canonical.lineage.clone();
        assert!(canonical.validate_checkpoint(&checkpoint).is_err());
        let mut route = control.config.clone();
        route.read_spec = ReadSpec::Route;
        assert!(route.validate().is_err());
        let mut unnormalized = canonical.config.clone();
        unnormalized.normalization_mode = NormalizationMode::UnnormalizedControlV1;
        unnormalized.validate()?;
        unnormalized.read_spec = ReadSpec::Route;
        assert!(unnormalized.validate().is_err());
        Ok(())
    }
    #[test]
    fn position_config_round_trips_and_checkpoint_mismatch_refuses() -> Result<()> {
        use crate::position_mechanism::{PositionConfig, PositionMode, PositionStepTable};

        let mut control = metadata_fixture(1, ReadSpec::Relation { kept: 2, sweeps: 1 })?;
        control.config.position = Some(PositionConfig::new(
            PositionMode::PrefixHolonomyV1,
            PositionStepTable::identity(control.config.spec.blocks)?,
        ));
        control.config.validate()?;
        control.lineage.config = control.config.clone();
        let encoded = serde_json::to_value(&control.config).map_err(|e| invalid(e.to_string()))?;
        assert_eq!(encoded["position"]["mode"], "prefix_holonomy_v1");
        let decoded: DriverConfig =
            serde_json::from_value(encoded).map_err(|e| invalid(e.to_string()))?;
        assert_eq!(decoded, control.config);

        let model = json!({"schema":crate::SCHEMA,"spec":control.config.spec,"read_spec":control.config.read_spec,"seed":1,"epoch":0,"source_sha256":"0".repeat(64),"kernel_sha256":"1".repeat(64),"geometry_sha256":"2".repeat(64)});
        let checkpoint = DriverCheckpoint {
            schema: CHECKPOINT_SCHEMA.into(),
            lineage: control.lineage.clone(),
            cursor: Cursor::initial(),
            model_sha256: corpus::sha256(
                &serde_json::to_vec(&model).map_err(|e| invalid(e.to_string()))?,
            ),
            model,
        };
        assert!(control.validate_checkpoint(&checkpoint).is_err());
        let mut matched = checkpoint;
        matched.model["position"] = serde_json::to_value(control.config.position.clone()).unwrap();
        matched.model_sha256 = corpus::sha256(
            &serde_json::to_vec(&matched.model).map_err(|e| invalid(e.to_string()))?,
        );
        control.validate_checkpoint(&matched)?;
        let mut legacy = metadata_fixture(1, ReadSpec::Relation { kept: 2, sweeps: 1 })?;
        legacy.lineage.config = legacy.config.clone();
        assert!(legacy.validate_checkpoint(&matched).is_err());
        assert_eq!(
            serde_json::to_value(&legacy.config)
                .unwrap()
                .get("position"),
            None
        );
        Ok(())
    }

    fn frozen_relation_metadata_fixture() -> Result<PreparedDriver> {
        let mut driver = metadata_schedule_fixture(
            1,
            ReadSpec::Relation { kept: 8, sweeps: 1 },
            Spec {
                vocabulary: 4096,
                blocks: 4,
                context: 192,
                kept_cells: 8,
            },
            SchedulePolicy::InterleavedReadEvery8V1,
        )?;
        driver.config.read_initialization = ReadInitialization::ActiveLeafV1;
        driver.config.relation_learning_policy = RelationLearningPolicy::FrozenZeroV1;
        driver.config.validate()?;
        driver.lineage.config = driver.config.clone();
        Ok(driver)
    }
    #[test]
    fn relation_learning_policy_defaults_and_explicit_scope() -> Result<()> {
        let legacy = metadata_fixture(1, ReadSpec::Route)?;
        let mut encoded =
            serde_json::to_value(&legacy.config).map_err(|e| invalid(e.to_string()))?;
        assert!(encoded.get("relation_learning_policy").is_none());
        let decoded: DriverConfig =
            serde_json::from_value(encoded.clone()).map_err(|e| invalid(e.to_string()))?;
        assert_eq!(decoded, legacy.config);
        encoded["relation_learning_policy"] = json!(RelationLearningPolicy::LearnV1);
        let explicit: DriverConfig =
            serde_json::from_value(encoded).map_err(|e| invalid(e.to_string()))?;
        assert_eq!(explicit, legacy.config);
        let frozen = frozen_relation_metadata_fixture()?;
        assert_eq!(
            serde_json::to_value(&frozen.config).map_err(|e| invalid(e.to_string()))?
                ["relation_learning_policy"],
            "frozen_zero_v1"
        );
        for mode in [ReadSpec::Route, ReadSpec::Flock { kept: 8 }] {
            let mut wrong = frozen.config.clone();
            wrong.read_spec = mode;
            assert!(wrong.validate().is_err());
        }
        let mut wrong = frozen.config.clone();
        wrong.schedule = SchedulePolicy::CanonicalVectorThenReadV1;
        assert!(wrong.validate().is_err());
        wrong = frozen.config.clone();
        wrong.read_initialization = ReadInitialization::ZeroGatesV1;
        assert!(wrong.validate().is_err());
        Ok(())
    }
    #[test]
    fn initial_cursor_for_is_k_aware_and_k1_is_legacy_initial() -> Result<()> {
        let driver = metadata_fixture(1, ReadSpec::Route)?;
        let mut config = driver.config().clone();
        assert_eq!(initial_cursor_for(&config), Cursor::initial());
        assert_eq!(driver.initial_cursor(), Cursor::initial());
        config.coordinate_block = CoordinateBlock(4);
        let k4 = initial_cursor_for(&config);
        assert_eq!(
            k4.block,
            Some(BlockCursor {
                o_vec: 0,
                o_read: 0,
                committed_moves: 0
            })
        );
        assert_ne!(k4, Cursor::initial());
        Ok(())
    }
    const SPEC24: Spec = Spec {
        vocabulary: 10,
        blocks: 2,
        context: 3,
        kept_cells: 2,
    };
    const SPEC192: Spec = Spec {
        vocabulary: 4096,
        blocks: 4,
        context: 192,
        kept_cells: 8,
    };
    fn block_fixture(
        k: u32,
        mode: ReadSpec,
        spec: Spec,
        schedule: SchedulePolicy,
    ) -> Result<PreparedDriver> {
        let mut driver = metadata_schedule_fixture(1, mode, spec, schedule)?;
        driver.config.coordinate_block = CoordinateBlock(k);
        driver.config.counterfactual_path = CounterfactualPath::RowsV1;
        Ok(driver)
    }
    fn refusal<T: std::fmt::Debug>(result: Result<T>) -> String {
        match result {
            Err(Error::Invalid(message)) => message,
            other => format!("not an Invalid refusal: {other:?}"),
        }
    }
    fn parameter_index(parameter: crate::Parameter) -> u32 {
        match parameter {
            crate::Parameter::VectorBlock(i)
            | crate::Parameter::Route(i)
            | crate::Parameter::Gate(i) => i,
        }
    }
    #[test]
    fn block_k1_reproduces_coordinate_at_on_24_step_and_c192_fixtures() -> Result<()> {
        for schedule in [
            SchedulePolicy::InterleavedReadEvery8V1,
            SchedulePolicy::CanonicalVectorThenReadV1,
        ] {
            for (spec, mode, steps) in [
                (SPEC24, ReadSpec::Route, 48),
                (SPEC24, ReadSpec::Flock { kept: 2 }, 48),
                (SPEC24, ReadSpec::Relation { kept: 2, sweeps: 1 }, 48),
                (SPEC192, ReadSpec::Route, 16404),
                (SPEC192, ReadSpec::Relation { kept: 8, sweeps: 1 }, 16404),
            ] {
                let driver = block_fixture(1, mode, spec, schedule)?;
                let mut cursor = driver.initial_cursor();
                assert_eq!(cursor, Cursor::initial());
                for step in 0..steps {
                    let block = driver.block_at(&cursor)?;
                    let expected = driver.coordinate_at(step)?;
                    assert_eq!(block.kind, expected.kind);
                    assert_eq!(block.coordinates, vec![expected.parameter()]);
                    let committed = step % 3 == 0;
                    let next = driver.advance_block(&cursor, committed, u64::from(committed))?;
                    // At K = 1 a block advance is today's advance, byte for byte.
                    assert_eq!(next, driver.advance(&cursor, committed)?);
                    assert!(next.block.is_none());
                    cursor = next;
                }
            }
        }
        Ok(())
    }
    #[test]
    fn block_visit_ratio_and_family_passes_match_k1() -> Result<()> {
        for schedule in [
            SchedulePolicy::InterleavedReadEvery8V1,
            SchedulePolicy::CanonicalVectorThenReadV1,
        ] {
            for mode in [ReadSpec::Route, ReadSpec::Relation { kept: 8, sweeps: 1 }] {
                for k in [4u32, 16] {
                    let driver = block_fixture(k, mode, SPEC192, schedule)?;
                    let (vectors, reads) = driver.families();
                    let mut cursor = driver.initial_cursor();
                    let (mut vector_batches, mut read_batches) = (0u64, 0u64);
                    let (mut vector_seq, mut read_seq) = (Vec::new(), Vec::new());
                    let mut moves = 0;
                    // Two full vector passes in either schedule.
                    let cycle = vectors.div_ceil(u64::from(k)) + reads.div_ceil(u64::from(k));
                    let steps = match schedule {
                        SchedulePolicy::InterleavedReadEvery8V1 => {
                            8 * (2 * vectors.div_ceil(u64::from(k))).div_ceil(7)
                        }
                        SchedulePolicy::CanonicalVectorThenReadV1 => 2 * cycle,
                    };
                    for step in 0..steps {
                        let block = driver.block_at(&cursor)?;
                        let indices: Vec<u32> = block
                            .coordinates
                            .iter()
                            .copied()
                            .map(parameter_index)
                            .collect();
                        assert!(!indices.is_empty() && indices.len() <= k as usize);
                        // Contiguous, ascending: no block wraps or straddles a pass.
                        assert!(indices.windows(2).all(|w| w[1] == w[0] + 1));
                        if block.kind == CoordinateKind::VectorBlock {
                            vector_batches += 1;
                            vector_seq.extend(indices.iter().copied());
                        } else {
                            read_batches += 1;
                            read_seq.extend(indices.iter().copied());
                        }
                        let take = (step % 2 == 0).then_some(indices.len() as u64);
                        moves += take.unwrap_or(0);
                        cursor =
                            driver.advance_block(&cursor, take.is_some(), take.unwrap_or(0))?;
                        assert_eq!(cursor.block.map(|b| b.committed_moves), Some(moves));
                    }
                    if schedule == SchedulePolicy::InterleavedReadEvery8V1 {
                        // Batch kinds follow today's rule, so 1 read : 7 vector per 8.
                        // Exactly every eighth batch is a read batch.
                        assert_eq!(read_batches, steps / 8);
                        assert_eq!(vector_batches, steps - steps / 8);
                    } else {
                        assert_eq!(vector_batches, 2 * vectors.div_ceil(u64::from(k)));
                        assert_eq!(vector_seq.len() as u64, 2 * vectors);
                        assert_eq!(read_batches, 2 * reads.div_ceil(u64::from(k)));
                        assert_eq!(read_seq.len() as u64, 2 * reads);
                    }
                    // Every coordinate once per family pass, in today's order.
                    for (seq, size) in [(&vector_seq, vectors), (&read_seq, reads)] {
                        // Exact visit counts: full passes plus the ascending partial.
                        let (passes, rest) = (seq.len() as u64 / size, seq.len() as u64 % size);
                        assert!(passes >= 1);
                        for index in 0..size {
                            let count = seq.iter().filter(|i| u64::from(**i) == index).count();
                            assert_eq!(count as u64, passes + u64::from(index < rest));
                        }
                        assert!(seq
                            .iter()
                            .enumerate()
                            .all(|(i, index)| u64::from(*index) == i as u64 % size));
                    }
                    assert!(vector_seq.len() as u64 >= 2 * vectors);
                }
            }
        }
        Ok(())
    }
    #[test]
    fn block_config_and_resume_refusals_are_focused() -> Result<()> {
        let config = metadata_fixture(1, ReadSpec::Route)?.config.clone();
        for k in [0, 65] {
            let mut bad = config.clone();
            bad.coordinate_block = CoordinateBlock(k);
            bad.counterfactual_path = CounterfactualPath::RowsV1;
            assert_eq!(refusal(bad.validate()), "coordinate_block must be 1..=64");
        }
        let mut lanes = config.clone();
        lanes.coordinate_block = CoordinateBlock(4);
        assert_eq!(
            refusal(lanes.validate()),
            "coordinate_block > 1 requires the rows_v1 counterfactual path"
        );
        let mut rows = lanes.clone();
        rows.counterfactual_path = CounterfactualPath::RowsV1;
        rows.validate()?;

        let driver = block_fixture(
            4,
            ReadSpec::Route,
            SPEC24,
            SchedulePolicy::InterleavedReadEvery8V1,
        )?;
        let mut cursor = driver.initial_cursor();
        for step in 0..11 {
            let length = driver.block_at(&cursor)?.coordinates.len() as u64;
            cursor =
                driver.advance_block(&cursor, step % 2 == 0, (step % 2 == 0) as u64 * length)?;
        }
        driver.validate_cursor(&cursor)?;
        let block = cursor
            .block
            .ok_or_else(|| invalid("K>1 cursor has block"))?;
        let with = |b: BlockCursor| Cursor {
            block: Some(b),
            ..cursor
        };
        let schedule_refusal = "resume o_vec/o_read differ from the per-family block schedule";
        assert_eq!(
            refusal(driver.validate_cursor(&with(BlockCursor {
                o_vec: block.o_vec + 1,
                ..block
            }))),
            schedule_refusal
        );
        assert_eq!(
            refusal(driver.validate_cursor(&with(BlockCursor {
                o_read: block.o_read + 1,
                ..block
            }))),
            schedule_refusal
        );
        let moves_refusal = "resume committed_moves outside accepted_updates..=K*accepted_updates";
        for committed_moves in [cursor.accepted_updates - 1, 4 * cursor.accepted_updates + 1] {
            assert_eq!(
                refusal(driver.validate_cursor(&with(BlockCursor {
                    committed_moves,
                    ..block
                }))),
                moves_refusal
            );
        }
        assert_eq!(
            refusal(driver.validate_cursor(&Cursor {
                block: None,
                ..cursor
            })),
            "coordinate_block > 1 cursor requires block fields"
        );
        assert_eq!(
            refusal(driver.advance(&cursor, true)),
            "coordinate_block > 1 advances through advance_block"
        );
        assert_eq!(
            refusal(driver.advance_block(&cursor, true, 0)),
            "block commit needs 1..=block-length moves, and no commit none"
        );
        let k1 = block_fixture(
            1,
            ReadSpec::Route,
            SPEC24,
            SchedulePolicy::InterleavedReadEvery8V1,
        )?;
        assert_eq!(
            refusal(k1.validate_cursor(&Cursor {
                block: Some(BlockCursor {
                    o_vec: 0,
                    o_read: 0,
                    committed_moves: 0
                }),
                ..Cursor::initial()
            })),
            "coordinate_block 1 cursor carries no block fields"
        );
        Ok(())
    }
    #[test]
    fn block_cursor_serialized_mid_pass_resumes_the_uninterrupted_sequence() -> Result<()> {
        for schedule in [
            SchedulePolicy::InterleavedReadEvery8V1,
            SchedulePolicy::CanonicalVectorThenReadV1,
        ] {
            let driver = block_fixture(4, ReadSpec::Flock { kept: 2 }, SPEC24, schedule)?;
            let run = |mut cursor: Cursor, from: u64, to: u64| -> Result<(Cursor, Vec<Block>)> {
                let mut blocks = Vec::new();
                for step in from..to {
                    let block = driver.block_at(&cursor)?;
                    let moves = if step % 3 == 0 {
                        0
                    } else {
                        block.coordinates.len().min(2) as u64
                    };
                    blocks.push(block);
                    cursor = driver.advance_block(&cursor, moves > 0, moves)?;
                }
                Ok((cursor, blocks))
            };
            let (whole_end, whole) = run(driver.initial_cursor(), 0, 100)?;
            let (middle, mut first) = run(driver.initial_cursor(), 0, 37)?;
            let encoded = serde_json::to_vec(&middle).map_err(|e| invalid(e.to_string()))?;
            let resumed: Cursor =
                serde_json::from_slice(&encoded).map_err(|e| invalid(e.to_string()))?;
            assert_eq!(resumed, middle);
            driver.validate_cursor(&resumed)?;
            let (end, rest) = run(resumed, 37, 100)?;
            first.extend(rest);
            assert_eq!(first, whole);
            assert_eq!(end, whole_end);
        }
        Ok(())
    }
    /// Block coordinate lists (vector blocks, read blocks) in visit order.
    fn k3_cycle(schedule: SchedulePolicy, steps: u64) -> Result<(Vec<Vec<u32>>, Vec<Vec<u32>>)> {
        let driver = block_fixture(3, ReadSpec::Flock { kept: 2 }, SPEC24, schedule)?;
        assert_eq!(driver.families(), (20, 4));
        let mut cursor = driver.initial_cursor();
        let (mut vector, mut read) = (Vec::new(), Vec::new());
        for _ in 0..steps {
            let block = driver.block_at(&cursor)?;
            let list: Vec<u32> = block
                .coordinates
                .iter()
                .copied()
                .map(parameter_index)
                .collect();
            if block.kind == CoordinateKind::VectorBlock {
                vector.push(list);
            } else {
                read.push(list);
            }
            cursor = driver.advance_block(&cursor, false, 0)?;
        }
        Ok((vector, read))
    }
    #[test]
    fn block_k3_truncates_at_pass_boundaries_under_both_schedules() -> Result<()> {
        let pass: Vec<Vec<u32>> = vec![
            vec![0, 1, 2],
            vec![3, 4, 5],
            vec![6, 7, 8],
            vec![9, 10, 11],
            vec![12, 13, 14],
            vec![15, 16, 17],
            vec![18, 19],
        ];
        let reads: Vec<Vec<u32>> = vec![vec![0, 1, 2], vec![3]];
        // Canonical: 7 vector blocks then 2 read blocks per cycle.
        let (vector, read) = k3_cycle(SchedulePolicy::CanonicalVectorThenReadV1, 9)?;
        assert_eq!((&vector, &read), (&pass, &reads));
        // Interleaved: every eighth batch is a read batch; 16 steps = 14 + 2.
        let (vector, read) = k3_cycle(SchedulePolicy::InterleavedReadEvery8V1, 16)?;
        assert_eq!(vector, [pass.clone(), pass.clone()].concat());
        assert_eq!(read, reads);
        // No block straddles a pass; every coordinate once per family pass.
        for list in vector.iter().chain(read.iter()) {
            assert!(list.windows(2).all(|w| w[1] == w[0] + 1));
        }
        assert_eq!(pass.concat(), (0..20).collect::<Vec<u32>>());
        assert_eq!(reads.concat(), (0..4).collect::<Vec<u32>>());
        Ok(())
    }
    #[test]
    fn block_k3_cursor_resumes_right_after_a_truncated_block() -> Result<()> {
        for schedule in [
            SchedulePolicy::InterleavedReadEvery8V1,
            SchedulePolicy::CanonicalVectorThenReadV1,
        ] {
            let driver = block_fixture(3, ReadSpec::Flock { kept: 2 }, SPEC24, schedule)?;
            let run = |mut cursor: Cursor, from: u64, to: u64| -> Result<(Cursor, Vec<Block>)> {
                let mut blocks = Vec::new();
                for step in from..to {
                    let block = driver.block_at(&cursor)?;
                    let moves = if step % 3 == 0 { 0 } else { 1 };
                    blocks.push(block);
                    cursor = driver.advance_block(&cursor, moves > 0, moves)?;
                }
                Ok((cursor, blocks))
            };
            // Step 6 is the truncated [18, 19] vector block; resume right after it.
            let (middle, mut first) = run(driver.initial_cursor(), 0, 7)?;
            assert_eq!(first[6].coordinates.len(), 2);
            let encoded = serde_json::to_vec(&middle).map_err(|e| invalid(e.to_string()))?;
            let resumed: Cursor =
                serde_json::from_slice(&encoded).map_err(|e| invalid(e.to_string()))?;
            driver.validate_cursor(&resumed)?;
            let (end, rest) = run(resumed, 7, 60)?;
            first.extend(rest);
            let (whole_end, whole) = run(driver.initial_cursor(), 0, 60)?;
            assert_eq!(first, whole);
            assert_eq!(end, whole_end);
        }
        Ok(())
    }
    #[test]
    fn block_k4_initial_checkpoint_validates_and_coordinate_at_refuses() -> Result<()> {
        let driver = block_fixture(
            4,
            ReadSpec::Flock { kept: 2 },
            SPEC24,
            SchedulePolicy::InterleavedReadEvery8V1,
        )?;
        let initial = driver.initial_cursor();
        assert_ne!(initial, Cursor::initial());
        driver.validate_cursor(&initial)?;
        let model = json!({"schema":crate::SCHEMA,"spec":driver.config.spec,"read_spec":driver.config.read_spec,"seed":1,"epoch":0,"source_sha256":"0".repeat(64),"kernel_sha256":"1".repeat(64),"geometry_sha256":"2".repeat(64)});
        let checkpoint = DriverCheckpoint {
            schema: CHECKPOINT_SCHEMA.into(),
            lineage: driver.lineage.clone(),
            cursor: initial,
            model_sha256: corpus::sha256(
                &serde_json::to_vec(&model).map_err(|e| invalid(e.to_string()))?,
            ),
            model,
        };
        driver.validate_checkpoint(&checkpoint)?;
        let mut legacy = checkpoint.clone();
        legacy.cursor = Cursor::initial();
        assert!(driver.validate_checkpoint(&legacy).is_err());
        assert_eq!(
            refusal(driver.coordinate_at(0)),
            "coordinate_block > 1 schedule is addressed through block_at"
        );
        Ok(())
    }
    #[test]
    fn coordinate_block_default_keeps_config_and_cursor_bytes() -> Result<()> {
        let legacy = metadata_fixture(1, ReadSpec::Route)?;
        let encoded = serde_json::to_vec(&legacy.config).map_err(|e| invalid(e.to_string()))?;
        let value: Value = serde_json::from_slice(&encoded).map_err(|e| invalid(e.to_string()))?;
        assert!(value.get("coordinate_block").is_none());
        let decoded: DriverConfig =
            serde_json::from_slice(&encoded).map_err(|e| invalid(e.to_string()))?;
        assert_eq!(decoded.coordinate_block, CoordinateBlock::ONE);
        assert_eq!(
            serde_json::to_vec(&decoded).map_err(|e| invalid(e.to_string()))?,
            encoded
        );
        // The opt-in adds exactly one key; removing it restores the K = 1 bytes.
        let mut blocked = legacy.config.clone();
        blocked.coordinate_block = CoordinateBlock(4);
        blocked.counterfactual_path = CounterfactualPath::RowsV1;
        let mut blocked_value =
            serde_json::to_value(&blocked).map_err(|e| invalid(e.to_string()))?;
        assert_eq!(blocked_value["coordinate_block"], 4);
        let mut rows_value = value.clone();
        rows_value["counterfactual_path"] = json!("rows_v1");
        blocked_value
            .as_object_mut()
            .ok_or_else(|| invalid("config is an object"))?
            .remove("coordinate_block");
        assert_eq!(blocked_value, rows_value);
        // Pre-change K = 1 cursor bytes, verbatim.
        let k1 = block_fixture(
            1,
            ReadSpec::Route,
            SPEC24,
            SchedulePolicy::InterleavedReadEvery8V1,
        )?;
        assert_eq!(
            serde_json::to_string(&k1.initial_cursor()).map_err(|e| invalid(e.to_string()))?,
            r#"{"completed_batches":0,"accepted_updates":0,"corpus_pass":0,"next_target":0,"next_coordinate":0}"#
        );
        let k4 = block_fixture(
            4,
            ReadSpec::Route,
            SPEC24,
            SchedulePolicy::InterleavedReadEvery8V1,
        )?;
        assert!(serde_json::to_string(&k4.initial_cursor())
            .map_err(|e| invalid(e.to_string()))?
            .ends_with(r#""block":{"o_vec":0,"o_read":0,"committed_moves":0}}"#));
        Ok(())
    }
    #[test]
    fn counterfactual_path_default_keeps_config_bytes_and_rows_is_explicit() -> Result<()> {
        let legacy = metadata_fixture(1, ReadSpec::Route)?;
        let encoded = serde_json::to_value(&legacy.config).map_err(|e| invalid(e.to_string()))?;
        // The default is never serialized, so existing config hashes are unchanged.
        assert!(encoded.get("counterfactual_path").is_none());
        assert_eq!(
            legacy.config.counterfactual_path,
            CounterfactualPath::LanesV1
        );
        let mut rows = legacy.config.clone();
        rows.counterfactual_path = CounterfactualPath::RowsV1;
        let rows_encoded = serde_json::to_value(&rows).map_err(|e| invalid(e.to_string()))?;
        assert_eq!(rows_encoded["counterfactual_path"], "rows_v1");
        let decoded: DriverConfig =
            serde_json::from_value(rows_encoded).map_err(|e| invalid(e.to_string()))?;
        assert_eq!(decoded, rows);
        // The selected kernel is part of the bound configuration identity.
        assert_ne!(
            serde_json::to_vec(&rows).map_err(|e| invalid(e.to_string()))?,
            serde_json::to_vec(&legacy.config).map_err(|e| invalid(e.to_string()))?
        );
        Ok(())
    }
    #[test]
    fn codebook_initialization_default_keeps_config_bytes_and_pin_is_bound() -> Result<()> {
        let legacy = metadata_fixture(1, ReadSpec::Route)?;
        let encoded = serde_json::to_value(&legacy.config).map_err(|e| invalid(e.to_string()))?;
        // The seeded default is never serialized: existing config hashes hold.
        assert!(encoded.get("codebook_initialization").is_none());
        assert!(legacy.initial_table.is_none());
        let mut pinned = legacy.config.clone();
        pinned.codebook_initialization = CodebookInitialization::PinnedTableV1 {
            table: FilePin {
                path: std::path::PathBuf::from("/codebook.json"),
                sha256: "a".repeat(64),
            },
        };
        let pinned_encoded = serde_json::to_value(&pinned).map_err(|e| invalid(e.to_string()))?;
        assert_eq!(
            pinned_encoded["codebook_initialization"]["kind"],
            "pinned_table_v1"
        );
        let decoded: DriverConfig =
            serde_json::from_value(pinned_encoded).map_err(|e| invalid(e.to_string()))?;
        assert_eq!(decoded, pinned);
        // The pinned table's file hash is part of the bound configuration.
        assert_ne!(
            serde_json::to_vec(&pinned).map_err(|e| invalid(e.to_string()))?,
            serde_json::to_vec(&legacy.config).map_err(|e| invalid(e.to_string()))?
        );
        Ok(())
    }
    #[test]
    fn pinned_codebook_table_is_checked_before_any_model() -> Result<()> {
        let spec = Spec {
            vocabulary: 2,
            blocks: 2,
            context: 4,
            kept_cells: 1,
        };
        let table: Vec<i32> = (0..32).collect();
        let good = CodebookTable {
            schema: CODEBOOK_TABLE_SCHEMA.into(),
            spec,
            mode: "prime_zeta".into(),
            table_sha256: CodebookTable::words_sha256(&table),
            distinct_vectors: CodebookTable::count_distinct(&table, spec.blocks),
            table: table.clone(),
        };
        // Gitignored scratch under the repository root (never /tmp).
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../local/test-scratch")
            .join(format!("codebook-table-{}", std::process::id()));
        std::fs::create_dir_all(&dir).map_err(|e| invalid(e.to_string()))?;
        let pin = |name: &str, file: &CodebookTable| -> Result<FilePin> {
            let path = dir.join(name);
            let bytes = serde_json::to_vec(file).map_err(|e| invalid(e.to_string()))?;
            std::fs::write(&path, &bytes).map_err(|e| invalid(e.to_string()))?;
            Ok(FilePin {
                path,
                sha256: corpus::sha256(&bytes),
            })
        };
        assert_eq!(CodebookTable::load(&pin("good.json", &good)?, spec)?, table);
        let mut wrong_schema = good.clone();
        wrong_schema.schema = "other".into();
        let mut short = good.clone();
        short.table.pop();
        short.table_sha256 = CodebookTable::words_sha256(&short.table);
        let mut wrong_digest = good.clone();
        wrong_digest.table[0] += 1;
        let mut other_spec = good.clone();
        other_spec.spec.vocabulary = 3;
        let mut other_blocks = good.clone();
        other_blocks.spec.blocks = 4;
        let mut false_distinct = good.clone();
        false_distinct.distinct_vectors += 1;
        for (name, file) in [
            ("schema.json", &wrong_schema),
            ("short.json", &short),
            ("digest.json", &wrong_digest),
            ("spec.json", &other_spec),
            ("blocks.json", &other_blocks),
            ("distinct.json", &false_distinct),
        ] {
            assert!(
                CodebookTable::load(&pin(name, file)?, spec).is_err(),
                "{name}"
            );
        }
        // A file whose bytes differ from its pin is refused before parsing.
        let mut stale = pin("good.json", &good)?;
        stale.sha256 = "b".repeat(64);
        assert!(CodebookTable::load(&stale, spec).is_err());
        std::fs::remove_dir_all(&dir).map_err(|e| invalid(e.to_string()))?;
        Ok(())
    }
    #[test]
    fn frozen_relation_preserves_all_scheduled_proposal_slots() -> Result<()> {
        let frozen = frozen_relation_metadata_fixture()?;
        let mut learned = frozen_relation_metadata_fixture()?;
        learned.config.relation_learning_policy = RelationLearningPolicy::LearnV1;
        learned.lineage.config = learned.config.clone();
        assert_eq!(frozen.coordinate_count(), 16384 + 45);
        assert_eq!(frozen.schedule_cycle(), 16404);
        let mut refused = 0;
        let mut cursor = Cursor::initial();
        for step in 0..16404 {
            let coordinate = frozen.coordinate_at(step)?;
            assert_eq!(coordinate, learned.coordinate_at(step)?);
            assert!(!learned.apply_refused_by_policy(coordinate)?);
            let expected = coordinate.kind == CoordinateKind::Gate && coordinate.index >= 20;
            assert_eq!(frozen.apply_refused_by_policy(coordinate)?, expected);
            refused += usize::from(expected);
            // Metadata only: a withheld commit advances the genuine-attempt
            // cursor while preserving accepted updates; no model is executed.
            cursor = frozen.advance(&cursor, false)?;
        }
        assert_eq!(refused, 1130);
        assert_eq!(cursor.completed_batches, 16404);
        assert_eq!(cursor.accepted_updates, 0);
        assert!(frozen
            .apply_refused_by_policy(Coordinate {
                kind: CoordinateKind::Gate,
                index: 45
            })
            .is_err());
        Ok(())
    }
    #[test]
    fn frozen_relation_checkpoint_requires_complete_zero_tail_and_policy_lineage() -> Result<()> {
        let frozen = frozen_relation_metadata_fixture()?;
        // Storage headers and trits only, never a CPU model/reference evaluator.
        let mut gates = vec![0i32; 45];
        gates[0] = -1;
        gates[19] = 1; // Combine gates remain learnable.
        let model = json!({"schema":crate::SCHEMA,"spec":frozen.config.spec,"read_spec":frozen.config.read_spec,"read_initialization":ReadInitialization::ActiveLeafV1,"seed":1,"epoch":0,"gates":gates,"source_sha256":"0".repeat(64),"kernel_sha256":"1".repeat(64),"geometry_sha256":"2".repeat(64)});
        let mut checkpoint = DriverCheckpoint {
            schema: CHECKPOINT_SCHEMA.into(),
            lineage: frozen.lineage.clone(),
            cursor: Cursor::initial(),
            model_sha256: corpus::sha256(
                &serde_json::to_vec(&model).map_err(|e| invalid(e.to_string()))?,
            ),
            model,
        };
        frozen.validate_checkpoint(&checkpoint)?;
        let rehash = |checkpoint: &mut DriverCheckpoint| -> Result<()> {
            checkpoint.model_sha256 = corpus::sha256(
                &serde_json::to_vec(&checkpoint.model).map_err(|e| invalid(e.to_string()))?,
            );
            Ok(())
        };
        for index in 20..45 {
            for trit in [-1, 1] {
                let mut wrong = checkpoint.clone();
                wrong.model["gates"][index] = json!(trit);
                rehash(&mut wrong)?;
                assert!(frozen.validate_checkpoint(&wrong).is_err());
            }
        }
        for bad in [
            Value::Null,
            json!(vec![0; 44]),
            json!(vec![0; 46]),
            json!(vec!["0"; 45]),
        ] {
            let mut wrong = checkpoint.clone();
            wrong.model["gates"] = bad;
            rehash(&mut wrong)?;
            assert!(frozen.validate_checkpoint(&wrong).is_err());
        }
        let mut wrong = checkpoint.clone();
        wrong
            .model
            .as_object_mut()
            .ok_or_else(|| invalid("fixture object"))?
            .remove("gates");
        rehash(&mut wrong)?;
        assert!(frozen.validate_checkpoint(&wrong).is_err());
        let mut learned = frozen_relation_metadata_fixture()?;
        learned.config.relation_learning_policy = RelationLearningPolicy::LearnV1;
        learned.lineage.config = learned.config.clone();
        assert!(learned.validate_checkpoint(&checkpoint).is_err());
        checkpoint.lineage.config.relation_learning_policy = RelationLearningPolicy::LearnV1;
        assert!(frozen.validate_checkpoint(&checkpoint).is_err());
        learned.validate_checkpoint(&checkpoint)?;
        Ok(())
    }
    #[test]
    fn receipt_policy_refusal_is_explicit_and_legacy_default_is_omitted() -> Result<()> {
        let value = json!({"lineage_sha256":"0".repeat(64),"cursor_before":Cursor::initial(),"cursor_after":Cursor::initial(),"coordinate":{"kind":"gate","index":20},"target_positions":[3],"candidate":1,"loss_before":10,"loss_after":9,"model_epoch":0,"applied":false,"any_candidate_cost_differs_from_identity":true});
        let mut receipt: StepReceipt =
            serde_json::from_value(value).map_err(|e| invalid(e.to_string()))?;
        assert!(!receipt.apply_refused_by_policy);
        assert!(serde_json::to_value(&receipt)
            .map_err(|e| invalid(e.to_string()))?
            .get("apply_refused_by_policy")
            .is_none());
        receipt.apply_refused_by_policy = true;
        let encoded = serde_json::to_value(&receipt).map_err(|e| invalid(e.to_string()))?;
        assert_eq!(encoded["apply_refused_by_policy"], true);
        assert_eq!(encoded["loss_after"], 9); // Hypothetical CUDA observation is retained.
        assert_eq!(encoded["applied"], false);
        Ok(())
    }
    #[test]
    fn k1_step_receipt_bytes_are_unchanged() -> Result<()> {
        let receipt = StepReceipt {
            lineage_sha256: "ab".into(),
            cursor_before: Cursor::initial(),
            cursor_after: Cursor::initial(),
            coordinate: Coordinate {
                kind: CoordinateKind::Gate,
                index: 20,
            },
            target_positions: vec![3],
            candidate: 1,
            loss_before: 10,
            loss_after: 9,
            model_epoch: 0,
            applied: false,
            apply_refused_by_policy: false,
            any_candidate_cost_differs_from_identity: true,
        };
        let cursor = r#"{"completed_batches":0,"accepted_updates":0,"corpus_pass":0,"next_target":0,"next_coordinate":0}"#;
        assert_eq!(
            serde_json::to_string(&receipt).map_err(|e| invalid(e.to_string()))?,
            format!(
                r#"{{"lineage_sha256":"ab","cursor_before":{cursor},"cursor_after":{cursor},"coordinate":{{"kind":"gate","index":20}},"target_positions":[3],"candidate":1,"loss_before":10,"loss_after":9,"model_epoch":0,"applied":false,"any_candidate_cost_differs_from_identity":true}}"#
            )
        );
        Ok(())
    }
    #[test]
    fn block_receipt_round_trips_and_omits_absent_joint_loss() -> Result<()> {
        let driver = block_fixture(
            4,
            ReadSpec::Flock { kept: 2 },
            SPEC24,
            SchedulePolicy::InterleavedReadEvery8V1,
        )?;
        let before = driver.initial_cursor();
        let after = driver.advance_block(&before, true, 3)?;
        let mut receipt = BlockReceipt {
            lineage_sha256: "ab".into(),
            cursor_before: before,
            cursor_after: after,
            kind: CoordinateKind::VectorBlock,
            k: 4,
            s_len: 3,
            joint: true,
            l_before: 10,
            l_joint: Some(7),
            l_best: 8,
            committed_loss: 7,
            commit_ordinals: vec![0, 2, 3],
            o_vec: 0,
            o_read: 0,
            completed_batches: 0,
            model_epoch: 1,
        };
        let encoded = serde_json::to_string(&receipt).map_err(|e| invalid(e.to_string()))?;
        let decoded: BlockReceipt =
            serde_json::from_str(&encoded).map_err(|e| invalid(e.to_string()))?;
        assert_eq!(decoded, receipt);
        assert_eq!(
            decoded.cursor_after.block,
            Some(BlockCursor {
                o_vec: 4,
                o_read: 0,
                committed_moves: 3
            })
        );
        receipt.l_joint = None;
        let value = serde_json::to_value(&receipt).map_err(|e| invalid(e.to_string()))?;
        assert!(value.get("l_joint").is_none());
        let decoded: BlockReceipt =
            serde_json::from_value(value).map_err(|e| invalid(e.to_string()))?;
        assert_eq!(decoded, receipt);
        Ok(())
    }
    #[test]
    fn block_checkpoint_refuses_inconsistent_moves_and_family_cursors() -> Result<()> {
        let driver = block_fixture(
            4,
            ReadSpec::Flock { kept: 2 },
            SPEC24,
            SchedulePolicy::InterleavedReadEvery8V1,
        )?;
        // Two committed batches (4 + 1 moves) and one no-op batch.
        let mut cursor = driver.initial_cursor();
        for (committed, moves) in [(true, 4), (false, 0), (true, 1)] {
            cursor = driver.advance_block(&cursor, committed, moves)?;
        }
        assert_eq!(cursor.accepted_updates, 2);
        let checkpoint_at = |cursor: Cursor| -> Result<DriverCheckpoint> {
            let model = json!({"schema":crate::SCHEMA,"spec":driver.config.spec,"read_spec":driver.config.read_spec,"seed":1,"epoch":cursor.accepted_updates,"source_sha256":"0".repeat(64),"kernel_sha256":"1".repeat(64),"geometry_sha256":"2".repeat(64)});
            Ok(DriverCheckpoint {
                schema: CHECKPOINT_SCHEMA.into(),
                lineage: driver.lineage.clone(),
                cursor,
                model_sha256: corpus::sha256(
                    &serde_json::to_vec(&model).map_err(|e| invalid(e.to_string()))?,
                ),
                model,
            })
        };
        driver.validate_checkpoint(&checkpoint_at(cursor)?)?;
        let block = cursor
            .block
            .ok_or_else(|| invalid("K = 4 cursor carries block fields"))?;
        assert_eq!(block.committed_moves, 5);
        for (moves, o_vec, o_read, message) in [
            (
                1,
                block.o_vec,
                block.o_read,
                "resume committed_moves outside accepted_updates..=K*accepted_updates",
            ),
            (
                9,
                block.o_vec,
                block.o_read,
                "resume committed_moves outside accepted_updates..=K*accepted_updates",
            ),
            (
                5,
                block.o_read,
                block.o_vec,
                "resume o_vec/o_read differ from the per-family block schedule",
            ),
            (
                5,
                block.o_vec + 1,
                block.o_read,
                "resume o_vec/o_read differ from the per-family block schedule",
            ),
        ] {
            let mut bad = cursor;
            bad.block = Some(BlockCursor {
                o_vec,
                o_read,
                committed_moves: moves,
            });
            assert_eq!(
                refusal(driver.validate_checkpoint(&checkpoint_at(bad)?)),
                message
            );
        }
        // epoch == accepted_updates still binds the model to the block cursor.
        let mut stale = checkpoint_at(cursor)?;
        stale.model["epoch"] = json!(5);
        stale.model_sha256 =
            corpus::sha256(&serde_json::to_vec(&stale.model).map_err(|e| invalid(e.to_string()))?);
        assert!(driver.validate_checkpoint(&stale).is_err());
        Ok(())
    }
}

/// A capability cannot be deserialized or constructed by external callers.
/// The default request remains held; the pilot controller validates explicit
/// release evidence and records its admission before requesting this capability.
pub struct CorpusUpdateAdmission {
    lineage_sha256: String,
    max_completed_batches: u64,
}
pub fn request_corpus_update_admission(_driver: &PreparedDriver) -> Result<CorpusUpdateAdmission> {
    Err(Error::TrainingHeld)
}

/// P1 is a summary of the completed GPU count table, not a host predictor.
/// The ascending pilot visits the first 410 tokens; no frequency reordering.
pub fn pilot_slice_metadata(counts: &[u64], source_targets: u64) -> Result<Value> {
    let total = counts.iter().try_fold(0u64, |sum, count| {
        sum.checked_add(*count)
            .ok_or_else(|| invalid("count receipt overflow"))
    })?;
    if total != source_targets {
        return Err(invalid(
            "GPU count receipt total differs from masked TRAIN targets",
        ));
    }
    let visited = &counts[..counts.len().min(410)];
    let sum = visited.iter().try_fold(0u64, |sum, count| {
        sum.checked_add(*count)
            .ok_or_else(|| invalid("slice count receipt overflow"))
    })?;
    let zero_ids: Vec<usize> = visited
        .iter()
        .enumerate()
        .filter_map(|(id, count)| (*count == 0).then_some(id))
        .collect();
    Ok(json!({
        "policy":"ascending VectorBlock 0..1639; token IDs 0..409 at blocks=4",
        "source":"completed GPU response-masked TRAIN count reference; DEV excluded",
        "visited_token_count":visited.len(), "target_counts_by_token_id":visited,
        "masked_train_target_sum":sum, "zero_count_tokens":zero_ids.len(),
        "zero_count_token_ids":zero_ids,
        "combined_share":{"numerator":sum,"denominator":source_targets},
        "status":if source_targets == 0 { "UNAVAILABLE" } else { "OBSERVED" }
    }))
}

/// P2 counts byte-identical storage patterns in a completed GPU snapshot.
/// Writer cells are exactly ordered (first root, last root) pairs. Equality of
/// those stored blocks counts occupied cells without routing/scoring on host.
/// The CUDA loader remains responsible for admitting geometric root values.
pub fn snapshot_metadata(snapshot: &Value) -> Result<Value> {
    use std::collections::BTreeSet;
    let spec: Spec = serde_json::from_value(
        snapshot
            .get("spec")
            .cloned()
            .ok_or_else(|| invalid("snapshot metadata missing spec"))?,
    )
    .map_err(|e| invalid(e.to_string()))?;
    spec.validate()?;
    let table: Vec<i32> = serde_json::from_value(
        snapshot
            .get("table")
            .cloned()
            .ok_or_else(|| invalid("snapshot metadata missing table"))?,
    )
    .map_err(|e| invalid(e.to_string()))?;
    let width = spec.blocks as usize * 8;
    if table.len() != spec.vocabulary as usize * width {
        return Err(invalid("snapshot metadata table shape mismatch"));
    }
    let mut patterns = BTreeSet::new();
    let mut cells = BTreeSet::new();
    for vector in table.chunks_exact(width) {
        patterns.insert(vector);
        cells.insert((&vector[..8], &vector[width - 8..]));
    }
    Ok(json!({"distinct_full_vector_patterns":patterns.len(),
        "occupied_writer_cells":cells.len(),"vocabulary":spec.vocabulary,
        "source":"exact stored full vectors and endpoint block pairs; host metadata only"}))
}

pub(crate) fn issue_corpus_update_admission(
    driver: &PreparedDriver,
    release: &crate::controller::ValidatedRelease,
) -> Result<CorpusUpdateAdmission> {
    let lineage_sha256 = driver.lineage_sha256()?;
    if release.lineage_sha256() != lineage_sha256 || !matches!(release.max_batches(), 1640 | 16404)
    {
        return Err(invalid(
            "release differs from the admitted pilot lineage/dose",
        ));
    }
    Ok(CorpusUpdateAdmission {
        lineage_sha256,
        max_completed_batches: release.max_batches(),
    })
}

/// Finite qualification has a separate opaque release; it cannot inherit a
/// full-dose capability or enlarge the approved source protocol's bound.
#[cfg(feature = "cuda")]
pub(crate) fn issue_audit_qualification_admission(
    driver: &PreparedDriver,
    release: &crate::controller::attention_audited::qualification::ValidatedQualificationRelease,
) -> Result<CorpusUpdateAdmission> {
    let max_completed_batches = release.finite_bound(driver)?;
    Ok(CorpusUpdateAdmission {
        lineage_sha256: driver.lineage_sha256()?,
        max_completed_batches,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StepReceipt {
    pub lineage_sha256: String,
    pub cursor_before: Cursor,
    pub cursor_after: Cursor,
    pub coordinate: Coordinate,
    pub target_positions: Vec<u64>,
    pub candidate: u64,
    pub loss_before: u64,
    pub loss_after: u64,
    pub model_epoch: u64,
    pub applied: bool,
    /// A genuine proposal was observed, but this transition's declared control
    /// forbade committing a relation-tail coordinate, regardless of its cost.
    #[serde(default, skip_serializing_if = "is_false")]
    pub apply_refused_by_policy: bool,
    /// Device comparison of every active candidate against CURRENT, including
    /// worse candidates. No accepted update is not the same as no sensitivity.
    pub any_candidate_cost_differs_from_identity: bool,
}
fn is_false(value: &bool) -> bool {
    !*value
}
/// One BVG-K block batch. `o_vec`/`o_read` and `completed_batches` are the
/// cursor before the batch; `cursor_after` carries the advanced block cursor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockReceipt {
    pub lineage_sha256: String,
    pub cursor_before: Cursor,
    pub cursor_after: Cursor,
    pub kind: CoordinateKind,
    pub k: u32,
    pub s_len: u32,
    pub joint: bool,
    pub l_before: u64,
    /// `L(S_t)`; present iff `|S_t| >= 2`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub l_joint: Option<u64>,
    pub l_best: u64,
    pub committed_loss: u64,
    pub commit_ordinals: Vec<u32>,
    pub o_vec: u64,
    pub o_read: u64,
    pub completed_batches: u64,
    pub model_epoch: u64,
}

#[cfg(feature = "cuda")]
impl PreparedDriver {
    fn admit_model(&self, model: &crate::device::Device, cursor: &Cursor) -> Result<()> {
        self.validate_cursor(cursor)?;
        if model.spec() != self.config.spec
            || model.read_spec() != self.config.read_spec
            || model.objective_policy() != self.config.objective_policy
            || model.read_initialization() != self.config.read_initialization
            || model.normalization_mode() != self.config.normalization_mode
            || model.position() != self.config.position.as_ref()
            || model.binding().seed != self.config.seed
            || model.epoch()? != cursor.accepted_updates
        {
            return Err(invalid(
                "model spec/read/seed/epoch differs from driver cursor",
            ));
        }
        if !self.config.relation_learning_policy.is_learn() {
            self.validate_frozen_relation_gates(&model.read_gates()?)?;
        }
        Ok(())
    }
    pub fn new_model(&self, ordinal: usize) -> Result<crate::device::Device> {
        let model = crate::device::Device::new_with_position(
            self.config.spec,
            self.config.read_spec,
            self.config.objective_policy,
            self.config.read_initialization,
            self.config.normalization_mode,
            self.config.position.clone(),
            self.config.seed,
            ordinal,
        )?;
        let Some(table) = &self.initial_table else {
            return Ok(model);
        };
        // Routes, gates, seed and epoch stay the device initializer's; only the
        // codebook is replaced. from_snapshot admits every table word as a root
        // on the GPU, exactly as for any reload.
        let mut snapshot = model.snapshot()?;
        snapshot.table = table.clone();
        let model = crate::device::Device::from_snapshot(snapshot, ordinal)?;
        if model.snapshot()?.table != *table {
            return Err(invalid("pinned initial codebook differs after device load"));
        }
        Ok(model)
    }
    pub fn checkpoint(
        &self,
        model: &crate::device::Device,
        cursor: Cursor,
    ) -> Result<DriverCheckpoint> {
        self.admit_model(model, &cursor)?;
        let model = serde_json::to_value(model.snapshot()?).map_err(|e| invalid(e.to_string()))?;
        let checkpoint = DriverCheckpoint {
            schema: CHECKPOINT_SCHEMA.into(),
            lineage: self.lineage.clone(),
            cursor,
            model_sha256: corpus::sha256(
                &serde_json::to_vec(&model).map_err(|e| invalid(e.to_string()))?,
            ),
            model,
        };
        self.validate_checkpoint(&checkpoint)?;
        Ok(checkpoint)
    }
    pub fn restore(
        &self,
        checkpoint: DriverCheckpoint,
        ordinal: usize,
    ) -> Result<(crate::device::Device, Cursor)> {
        self.validate_checkpoint(&checkpoint)?;
        let snapshot: crate::device::Snapshot =
            serde_json::from_value(checkpoint.model).map_err(|e| invalid(e.to_string()))?;
        let model = crate::device::Device::from_snapshot(snapshot, ordinal)?;
        self.admit_model(&model, &checkpoint.cursor)?;
        Ok((model, checkpoint.cursor))
    }
    /// Score one proposal with the configured exact counterfactual kernel. The
    /// row-parallel path returns the same losses and chooser input (#2630).
    fn propose_with_path(
        &self,
        model: &crate::device::Device,
        carried: &crate::device::CarriedBatch,
        targets: &[u32],
        parameter: crate::Parameter,
    ) -> Result<crate::device::Proposal> {
        match self.config.counterfactual_path {
            CounterfactualPath::LanesV1 => model.propose(carried, targets, parameter),
            CounterfactualPath::RowsV1 => model.propose_rows_cached(carried, targets, parameter),
        }
    }
    /// Deterministic virtual proposal. The returned next cursor changes no model state.
    pub fn propose_next(
        &self,
        model: &crate::device::Device,
        cursor: &Cursor,
    ) -> Result<StepReceipt> {
        self.admit_model(model, cursor)?;
        // Preflight all metadata overflow before any possible device mutation.
        // accepted_updates <= completed_batches makes the applied increment safe
        // whenever the next completed-batch increment is safe.
        self.advance(cursor, false)?;
        let batch = self.batch(cursor)?;
        let carried = model.carry_windows(&batch.windows)?;
        let coordinate = self.coordinate_at(cursor.completed_batches)?;
        let observation = self
            .propose_with_path(model, &carried, &batch.targets, coordinate.parameter())?
            .observation()?;
        Ok(StepReceipt {
            lineage_sha256: self.lineage_sha256()?,
            cursor_before: *cursor,
            cursor_after: self.advance(cursor, false)?,
            coordinate,
            target_positions: batch.target_positions,
            candidate: observation.candidate,
            loss_before: observation.before,
            loss_after: observation.after,
            model_epoch: observation.epoch,
            applied: false,
            apply_refused_by_policy: false,
            any_candidate_cost_differs_from_identity: observation
                .any_candidate_cost_differs_from_identity,
        })
    }
    /// Complete transition path, inaccessible without a genuinely issued owner capability.
    pub fn transition(
        &self,
        model: &mut crate::device::Device,
        cursor: &Cursor,
        admission: &CorpusUpdateAdmission,
    ) -> Result<StepReceipt> {
        if admission.lineage_sha256 != self.lineage_sha256()? {
            return Err(invalid("owner admission belongs to another driver lineage"));
        }
        if cursor.completed_batches >= admission.max_completed_batches {
            return Err(invalid("admitted diagnostic pilot dose exhausted"));
        }
        self.admit_model(model, cursor)?;
        // Refuse cursor overflow before apply can change the model epoch/state.
        self.advance(cursor, false)?;
        let batch = self.batch(cursor)?;
        let carried = model.carry_windows(&batch.windows)?;
        let coordinate = self.coordinate_at(cursor.completed_batches)?;
        let proposal =
            self.propose_with_path(model, &carried, &batch.targets, coordinate.parameter())?;
        let observation = proposal.observation()?;
        // CUDA still computes the complete counterfactual for frozen relation
        // slots. Policy only withholds the commit; its loss remains hypothetical.
        let apply_refused_by_policy = self.apply_refused_by_policy(coordinate)?;
        let applied = if apply_refused_by_policy {
            false
        } else {
            model.apply(&proposal)?
        };
        let after = self.advance(cursor, applied)?;
        self.admit_model(model, &after)?;
        Ok(StepReceipt {
            lineage_sha256: self.lineage_sha256()?,
            cursor_before: *cursor,
            cursor_after: after,
            coordinate,
            target_positions: batch.target_positions,
            candidate: observation.candidate,
            loss_before: observation.before,
            loss_after: observation.after,
            model_epoch: model.epoch()?,
            applied,
            apply_refused_by_policy,
            any_candidate_cost_differs_from_identity: observation
                .any_candidate_cost_differs_from_identity,
        })
    }
    pub fn run_transitions(
        &self,
        model: &mut crate::device::Device,
        mut cursor: Cursor,
        admission: &CorpusUpdateAdmission,
        batches: std::num::NonZeroU64,
    ) -> Result<(Cursor, Vec<StepReceipt>)> {
        if !self.config.coordinate_block.is_one() {
            return Err(invalid(
                "coordinate_block > 1 runs through run_block_transitions",
            ));
        }
        let mut receipts = Vec::new();
        for _ in 0..batches.get() {
            let receipt = self.transition(model, &cursor, admission)?;
            cursor = receipt.cursor_after;
            receipts.push(receipt);
        }
        Ok((cursor, receipts))
    }
    /// One BVG-K block batch under the same owner admission as `transition`:
    /// score the block on one carried batch, choose and decide on the device,
    /// commit the decided set (none, one move, or the joint `S_t`). FrozenZeroV1
    /// marks relation-tail gates ineligible, so they are never committed.
    /// Crate-private: drivers reach it only through `run_block_transitions`
    /// (K > 1); the K = 1 call exists solely for the H4 identity control V1.
    pub(crate) fn transition_block(
        &self,
        model: &mut crate::device::Device,
        cursor: &Cursor,
        admission: &CorpusUpdateAdmission,
        rowsk: &crate::device::RowsKScoring,
        bvgk: &crate::device::BvgkKernels,
    ) -> Result<BlockReceipt> {
        if admission.lineage_sha256 != self.lineage_sha256()? {
            return Err(invalid("owner admission belongs to another driver lineage"));
        }
        if cursor.completed_batches >= admission.max_completed_batches {
            return Err(invalid("admitted diagnostic pilot dose exhausted"));
        }
        self.admit_model(model, cursor)?;
        // Refuse cursor overflow before apply_block can change the model.
        self.advance_block(cursor, false, 0)?;
        let block = self.block_at(cursor)?;
        let eligible = block
            .coordinates
            .iter()
            .map(|parameter| {
                let index = match *parameter {
                    crate::Parameter::VectorBlock(i)
                    | crate::Parameter::Route(i)
                    | crate::Parameter::Gate(i) => i,
                };
                self.apply_refused_by_policy(Coordinate {
                    kind: block.kind,
                    index,
                })
                .map(|refused| !refused)
            })
            .collect::<Result<Vec<_>>>()?;
        let batch = self.batch(cursor)?;
        let carried = model.carry_windows(&batch.windows)?;
        let proposal = model.propose_block(
            &carried,
            &batch.targets,
            &block.coordinates,
            &eligible,
            rowsk,
            bvgk,
        )?;
        let record = proposal.record().clone();
        let commit = model.apply_block(&proposal)?;
        // The device commit must agree with its own record before the cursor
        // moves: a disagreement is a loud refusal, never a silent drift.
        if commit.moves as usize != record.commit_ordinals.len() || commit.joint != record.joint {
            return Err(Error::Invalid(format!(
                "block commit ({} moves, joint {}) disagrees with its record ({} ordinals, joint {})",
                commit.moves,
                commit.joint,
                record.commit_ordinals.len(),
                record.joint
            )));
        }
        let after = self.advance_block(cursor, commit.moves > 0, u64::from(commit.moves))?;
        self.admit_model(model, &after)?;
        let (o_vec, o_read) = self.family_cursors(cursor.completed_batches)?;
        Ok(BlockReceipt {
            lineage_sha256: self.lineage_sha256()?,
            cursor_before: *cursor,
            cursor_after: after,
            kind: block.kind,
            k: record.k,
            s_len: record.s_len,
            joint: record.joint,
            l_before: record.l_before,
            l_joint: record.l_joint,
            l_best: record.l_best,
            committed_loss: record.committed_loss,
            commit_ordinals: record.commit_ordinals,
            o_vec,
            o_read,
            completed_batches: cursor.completed_batches,
            model_epoch: model.epoch()?,
        })
    }
    /// The `coordinate_block > 1` loop; `K = 1` stays on `run_transitions`.
    pub fn run_block_transitions(
        &self,
        model: &mut crate::device::Device,
        mut cursor: Cursor,
        admission: &CorpusUpdateAdmission,
        batches: std::num::NonZeroU64,
    ) -> Result<(Cursor, Vec<BlockReceipt>)> {
        if self.config.coordinate_block.is_one() {
            return Err(invalid("coordinate_block 1 runs through run_transitions"));
        }
        let rowsk = model.rowsk_scoring()?;
        let bvgk = model.bvgk_kernels()?;
        let mut receipts = Vec::new();
        for _ in 0..batches.get() {
            let receipt = self.transition_block(model, &cursor, admission, &rowsk, &bvgk)?;
            cursor = receipt.cursor_after;
            receipts.push(receipt);
        }
        Ok((cursor, receipts))
    }
    pub fn evaluate(&self, model: &crate::device::Device) -> Result<Value> {
        self.evaluate_prepared(model, &self.dev, false, None)
    }
    /// Preserve the admitted checkpoint's exact stored Value domain A, even
    /// when typed decoding omits explicitly stored default fields on restore.
    pub fn evaluate_checkpoint(
        &self,
        model: &crate::device::Device,
        checkpoint: &DriverCheckpoint,
    ) -> Result<Value> {
        self.evaluate_prepared(model, &self.dev, false, Some(checkpoint))
    }
    /// A separately pinned evaluation-only corpus. Labels never select model
    /// state, initialization or the read; every numerical operation is CUDA.
    pub fn evaluate_transfer(
        &self,
        model: &crate::device::Device,
        transfer: &PreparedCorpus,
    ) -> Result<Value> {
        self.validate_transfer_content(transfer)?;
        self.evaluate_prepared(model, transfer, true, None)
    }
    pub fn evaluate_transfer_checkpoint(
        &self,
        model: &crate::device::Device,
        transfer: &PreparedCorpus,
        checkpoint: &DriverCheckpoint,
    ) -> Result<Value> {
        self.validate_transfer_content(transfer)?;
        self.evaluate_prepared(model, transfer, true, Some(checkpoint))
    }
    fn validate_transfer_content(&self, transfer: &PreparedCorpus) -> Result<()> {
        let binding = transfer.binding();
        if binding.source.split != Split::Dev
            || binding.vocabulary != self.config.spec.vocabulary
            || binding.bos != self.config.bos
            || binding.source.tokens.sha256 == self.config.train.tokens.sha256
            || binding.source.tokens.sha256 == self.config.dev.tokens.sha256
            || binding.eligible_targets < 200
        {
            return Err(invalid(
                "transfer needs separately pinned evaluation content, the same vocabulary/BOS and at least200 eligible rows",
            ));
        }
        Ok(())
    }
    fn evaluate_prepared(
        &self,
        model: &crate::device::Device,
        scored: &PreparedCorpus,
        transfer: bool,
        checkpoint: Option<&DriverCheckpoint>,
    ) -> Result<Value> {
        if let Some(checkpoint) = checkpoint {
            self.validate_checkpoint(checkpoint)?;
        }
        if model.spec() != self.config.spec
            || model.read_spec() != self.config.read_spec
            || model.objective_policy() != self.config.objective_policy
            || model.read_initialization() != self.config.read_initialization
            || model.normalization_mode() != self.config.normalization_mode
            || model.binding().seed != self.config.seed
        {
            return Err(invalid(
                "evaluation model config/seed differs from prepared lineage",
            ));
        }
        let epoch = model.epoch()?;
        let snapshot = model.snapshot()?;
        self.validate_frozen_relation_gates(&snapshot.gates)?;
        let mut snapshot_hashes = crate::device::snapshot_hash_domains(&snapshot)?;
        if let Some(checkpoint) = checkpoint {
            if epoch != checkpoint.cursor.accepted_updates {
                return Err(invalid("evaluation checkpoint epoch differs from model"));
            }
            let stored_hashes = crate::device::stored_snapshot_hash_domains(&checkpoint.model)?;
            snapshot_hashes.model_storage_sha256 = bind_evaluation_storage_domain(
                &checkpoint.model,
                &checkpoint.model_sha256,
                &stored_hashes.model_snapshot_sha256,
                &snapshot_hashes.model_snapshot_sha256,
            )?;
        }
        let mut first = 0;
        let mut receipts = Vec::new();
        while first < scored.eligible_len() as u64 {
            let batch = scored.batch(first, self.config.spec.context, self.config.batch_rows)?;
            let carried = model.carry_windows(&batch.windows)?;
            let mut evaluation = model.evaluate(&carried, &batch.targets)?;
            for ((row, position), target) in evaluation
                .rows
                .iter_mut()
                .zip(&batch.target_positions)
                .zip(&batch.targets)
            {
                row.target_position = Some(*position);
                row.target_token = Some(*target);
            }
            receipts.push(evaluation);
            first = batch.next_target;
        }
        let aggregate = model.aggregate_evaluations(&receipts)?;
        let reference = crate::device::Device::count_reference(
            self.config.spec.vocabulary,
            &self.train.eligible_targets(),
            &scored.eligible_targets(),
            model.ordinal(),
        )?;
        let pilot_slice_bias = pilot_slice_metadata(
            &reference.train_masked_reference.token_counts,
            reference.train_masked_reference.source_targets,
        )?;
        if model.epoch()? != epoch {
            return Err(invalid("evaluation changed model epoch"));
        }
        let mut record = json!({"schema":DRIVER_SCHEMA,"objective_evaluation_schema":crate::objective::EVALUATION_SCHEMA,"objective_policy":snapshot.objective_policy,"legacy_total_loss_numerator_semantics":"8T_plus_widthA","evaluation_lineage":self.lineage,"evaluation_lineage_sha256":self.lineage_sha256()?,"model_epoch":epoch,"model_storage_sha256":snapshot_hashes.model_storage_sha256,"model_snapshot_sha256":snapshot_hashes.model_snapshot_sha256,"model_bindings":{"schema":snapshot.schema,"spec":snapshot.spec,"read_spec":snapshot.read_spec,"objective_policy":snapshot.objective_policy,"seed":snapshot.seed,"source_sha256":snapshot.source_sha256,"kernel_sha256":snapshot.kernel_sha256,"geometry_sha256":snapshot.geometry_sha256,"operator_kappa":snapshot.operator_kappa,"composition_kappa":snapshot.composition_kappa},"training_provenance":"DriverCheckpoint restore required by eval-corpus CLI; this evaluator's lineage identifies its evaluation inputs","scored_set_policy":corpus::WINDOW_POLICY,"aggregate":aggregate,"batches":receipts,"count_reference":reference,"pilot_slice_bias":pilot_slice_bias,"evaluation_mutates_model":false,"accepted_updates_in_model":epoch,"updates_applied_by_evaluation":0});
        if transfer {
            record["transfer_corpus"] = json!(scored.binding());
            record["transfer_is_separate_evaluation_content"] = json!(true);
        }
        Ok(record)
    }
}
