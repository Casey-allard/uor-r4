//! Deterministic corpus/coordinate driver. Corpus commits require an unissued capability.
//! Receipt/cursor arithmetic is metadata; every model evaluation and proposal is CUDA.
#![forbid(unsafe_code)]
use crate::{
    corpus::{
        self, BosIdentity, CorpusBatch, CorpusBinding, CorpusSource, FilePin, PreparedCorpus, Split,
    },
    metadata::BinderPlan,
    Error, ReadSpec, Result, Spec,
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
    corpus::sha256(&bytes)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchedulePolicy {
    CanonicalVectorThenReadV1,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DriverConfig {
    pub schema: String,
    pub spec: Spec,
    pub read_spec: ReadSpec,
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
#[cfg(feature = "cuda")]
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
}
impl Cursor {
    pub fn initial() -> Self {
        Self {
            completed_batches: 0,
            accepted_updates: 0,
            corpus_pass: 0,
            next_target: 0,
            next_coordinate: 0,
        }
    }
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
        let coordinates = config
            .spec
            .blocks
            .checked_mul(
                config
                    .spec
                    .vocabulary
                    .checked_add(stages)
                    .ok_or_else(|| invalid("coordinate inventory overflow"))?,
            )
            .ok_or_else(|| invalid("coordinate inventory overflow"))?;
        let lineage = Lineage {
            schema: DRIVER_SCHEMA.into(),
            driver_source_sha256: driver_source_sha256(),
            config: config.clone(),
            train: train.binding().clone(),
            dev: dev.binding().clone(),
        };
        Ok(Self {
            config,
            train,
            dev,
            lineage,
            coordinates,
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
                    ReadSpec::Flock { .. } => CoordinateKind::Gate,
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
                != (cursor.completed_batches % u64::from(self.coordinates)) as u32
        {
            return Err(invalid(
                "resume cursor differs from canonical corpus/coordinate cycle",
            ));
        }
        Ok(())
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
            next_coordinate: (completed % u64::from(self.coordinates)) as u32,
        })
    }
    pub fn plan(&self) -> Result<Value> {
        let plan = BinderPlan::new(self.config.spec.context as usize)?;
        Ok(
            json!({"schema":DRIVER_SCHEMA,"lineage":self.lineage,"lineage_sha256":self.lineage_sha256()?,"seed":self.config.seed,"schedule":{"policy":self.config.schedule,"parameters":self.coordinates,"vector_parameters":self.config.spec.vocabulary*self.config.spec.blocks,"read_parameters":self.config.spec.blocks*plan.stages() as u32,"one_coordinate_per_batch":true,"corpus_order":"ascending response-masked target positions; wraps after complete coverage; common across seeds","initial_cursor":Cursor::initial()},"train_batches_per_pass":(self.train.eligible_len() as u64).div_ceil(u64::from(self.config.batch_rows)),"dev_batches":(self.dev.eligible_len() as u64).div_ceil(u64::from(self.config.batch_rows)),"training":"HELD","applied_updates":0,"throughput":"UNMEASURED"}),
        )
    }
    pub fn validate_checkpoint(&self, checkpoint: &DriverCheckpoint) -> Result<()> {
        if checkpoint.schema != CHECKPOINT_SCHEMA || checkpoint.lineage != self.lineage {
            return Err(invalid(
                "checkpoint source/config/data/mask/tokenizer/role-policy lineage mismatch",
            ));
        }
        self.validate_cursor(&checkpoint.cursor)?;
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
        if spec != self.config.spec
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

#[cfg(test)]
mod tests {
    use super::*;
    fn metadata_fixture(seed: u32, read_spec: ReadSpec) -> Result<PreparedDriver> {
        fn corpus(split: Split, ids: &[u16], mask: &[u8], eligible: u64) -> Result<PreparedCorpus> {
            let mut bytes = vec![0; 64];
            bytes[..4].copy_from_slice(b"UORT");
            bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
            bytes[8..16].copy_from_slice(&(ids.len() as u64).to_le_bytes());
            bytes[16..20].copy_from_slice(&10u32.to_le_bytes());
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
                10,
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
        )?;
        let dev = corpus(Split::Dev, &[0, 3, 4], &[0, 1, 1], 2)?;
        let config = DriverConfig {
            schema: DRIVER_SCHEMA.into(),
            spec: Spec {
                vocabulary: 10,
                blocks: 2,
                context: 3,
                kept_cells: 2,
            },
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
            schedule: SchedulePolicy::CanonicalVectorThenReadV1,
        };
        // Inline, already admitted metadata fixture; no file/model initialization.
        config.validate()?;
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
            coordinates: 24,
        })
    }
    #[test]
    fn complete_coordinate_cycle_and_resume_are_metadata() -> Result<()> {
        for mode in [ReadSpec::Route, ReadSpec::Flock { kept: 2 }] {
            let driver = metadata_fixture(1, mode)?;
            let paired = metadata_fixture(2, mode)?;
            let mut cursor = Cursor::initial();
            let mut coordinates = Vec::new();
            for step in 0..24 {
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
                (0..4)
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
                    completed_batches: 24,
                    accepted_updates: 0,
                    corpus_pass: 12,
                    next_target: 0,
                    next_coordinate: 0
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
            next_target: (u64::MAX % 2) * 3,
            next_coordinate: (u64::MAX % 24) as u32,
        };
        driver.validate_cursor(&boundary)?;
        assert!(driver.advance(&boundary, false).is_err());
        assert!(driver.advance(&boundary, true).is_err());
        Ok(())
    }
}

/// An owner-release capability cannot be deserialized, copied from configuration,
/// or constructed by callers. Issuance remains deliberately held, not unimplemented.
pub struct CorpusUpdateAdmission {
    lineage_sha256: String,
}
pub fn request_corpus_update_admission(_driver: &PreparedDriver) -> Result<CorpusUpdateAdmission> {
    Err(Error::TrainingHeld)
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
}

#[cfg(feature = "cuda")]
impl PreparedDriver {
    fn admit_model(&self, model: &crate::device::Device, cursor: &Cursor) -> Result<()> {
        self.validate_cursor(cursor)?;
        if model.spec() != self.config.spec
            || model.read_spec() != self.config.read_spec
            || model.binding().seed != self.config.seed
            || model.epoch()? != cursor.accepted_updates
        {
            return Err(invalid(
                "model spec/read/seed/epoch differs from driver cursor",
            ));
        }
        Ok(())
    }
    pub fn new_model(&self, ordinal: usize) -> Result<crate::device::Device> {
        crate::device::Device::new_with_read(
            self.config.spec,
            self.config.read_spec,
            self.config.seed,
            ordinal,
        )
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
        let coordinate = self.coordinate(cursor.next_coordinate)?;
        let observation = model
            .propose(&carried, &batch.targets, coordinate.parameter())?
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
        self.admit_model(model, cursor)?;
        // Refuse cursor overflow before apply can change the model epoch/state.
        self.advance(cursor, false)?;
        let batch = self.batch(cursor)?;
        let carried = model.carry_windows(&batch.windows)?;
        let coordinate = self.coordinate(cursor.next_coordinate)?;
        let proposal = model.propose(&carried, &batch.targets, coordinate.parameter())?;
        let observation = proposal.observation()?;
        // GPU chooses and commits; host consumes only the observed commit/epoch metadata.
        let applied = model.apply(&proposal)?;
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
        })
    }
    pub fn run_transitions(
        &self,
        model: &mut crate::device::Device,
        mut cursor: Cursor,
        admission: &CorpusUpdateAdmission,
        batches: std::num::NonZeroU64,
    ) -> Result<(Cursor, Vec<StepReceipt>)> {
        let mut receipts = Vec::new();
        for _ in 0..batches.get() {
            let receipt = self.transition(model, &cursor, admission)?;
            cursor = receipt.cursor_after;
            receipts.push(receipt);
        }
        Ok((cursor, receipts))
    }
    pub fn evaluate(&self, model: &crate::device::Device) -> Result<Value> {
        if model.spec() != self.config.spec
            || model.read_spec() != self.config.read_spec
            || model.binding().seed != self.config.seed
        {
            return Err(invalid(
                "evaluation model config/seed differs from prepared lineage",
            ));
        }
        let epoch = model.epoch()?;
        let snapshot = model.snapshot()?;
        let snapshot_sha256 =
            corpus::sha256(&serde_json::to_vec(&snapshot).map_err(|e| invalid(e.to_string()))?);
        let mut first = 0;
        let mut receipts = Vec::new();
        while first < self.dev.eligible_len() as u64 {
            let batch = self
                .dev
                .batch(first, self.config.spec.context, self.config.batch_rows)?;
            let carried = model.carry_windows(&batch.windows)?;
            receipts.push(model.evaluate(&carried, &batch.targets)?);
            first = batch.next_target;
        }
        let aggregate = model.aggregate_evaluations(&receipts)?;
        let reference = crate::device::Device::count_reference(
            self.config.spec.vocabulary,
            &self.train.eligible_targets(),
            &self.dev.eligible_targets(),
            model.ordinal(),
        )?;
        if model.epoch()? != epoch {
            return Err(invalid("evaluation changed model epoch"));
        }
        Ok(
            json!({"schema":DRIVER_SCHEMA,"evaluation_lineage":self.lineage,"evaluation_lineage_sha256":self.lineage_sha256()?,"model_epoch":epoch,"model_snapshot_sha256":snapshot_sha256,"model_bindings":{"schema":snapshot.schema,"spec":snapshot.spec,"read_spec":snapshot.read_spec,"seed":snapshot.seed,"source_sha256":snapshot.source_sha256,"kernel_sha256":snapshot.kernel_sha256,"geometry_sha256":snapshot.geometry_sha256,"operator_kappa":snapshot.operator_kappa,"composition_kappa":snapshot.composition_kappa},"training_provenance":"DriverCheckpoint restore required by eval-corpus CLI; this evaluator's lineage identifies its evaluation inputs","scored_set_policy":corpus::WINDOW_POLICY,"aggregate":aggregate,"batches":receipts,"count_reference":reference,"training":"HELD","applied_updates":0}),
        )
    }
}
