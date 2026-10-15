//! Stage M2 copy-successor sub-panel partitioner, interleaved schedule verifier,
//! and zero-trit attribution receipt auditor (`S1`–`S4`, `E1`–`E3`, `B1`–`B3`).
//! Metadata and stored GPU receipts only: zero model execution, zero CUDA, zero float.
#![forbid(unsafe_code)]
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs,
    io::{Read, Write},
    path::Path,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
type Counts = BTreeMap<String, i64>;

const SCHEMA: &str = "uor-r4.relation-subpanel-audit/1";
const DRIVER_SCHEMA: &str = "uor-r4.discrete-corpus-driver/1";
const CHECKPOINT_SCHEMA: &str = "uor-r4.discrete-driver-checkpoint/1";
const COMPLETION_CHECKPOINT_SCHEMA: &str = "uor-r4.completion-checkpoint/1";
const COMPLETION_EVAL_SCHEMA: &str = "uor-r4.completion-evaluation/1";
const MODEL_SCHEMA: &str = "uor-r4.discrete-spiralcore-fbs/2";
const WINDOW_POLICY: &str =
    "response-masked-targets/within-document-predecessor/sliding-prefix/ascending-once/v1";
const DEV_TOKENS_SHA256: &str = "0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b";
const DEV_MASK_SHA256: &str = "a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078";
const DEV_PAYLOAD_TOKENS: usize = 42_856;
const DEV_DOCUMENTS: usize = 400;
const DEV_ELIGIBLE_ROWS: usize = 14_287;
const DEV_QUERY_RECUR_ROWS: usize = 8_648;
const DEV_COPY_SUCCESSOR_ROWS: usize = 3_847;
const DEV_ANY_SUCCESSOR_ROWS: usize = 5_182;
const DEV_COPY_CEILING_ROWS: usize = 8_248;
const DEV_COMPLEMENT_ROWS: usize = 10_440;
const DEV_DISTANCE_BUCKETS: [usize; 4] = [50, 2_872, 900, 25];
const RELATION_EDGE_TYPES: u32 = 5;
const DECISION_CURSOR_BATCHES: u64 = 16_404;
// Receipt parser resource limit, checked before allocation; not a scientific bar.
const MAX_FILE_BYTES: u64 = 128 * 1024 * 1024;

fn require(ok: bool, message: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(message.into())
    }
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn read_bounded_bytes(path: &Path) -> Result<(Vec<u8>, String)> {
    let file = fs::File::open(path)?;
    require(
        file.metadata()?.len() <= MAX_FILE_BYTES,
        "input file exceeds 128 MiB limit",
    )?;
    let mut bytes = Vec::new();
    file.take(MAX_FILE_BYTES + 1).read_to_end(&mut bytes)?;
    require(
        bytes.len() as u64 <= MAX_FILE_BYTES,
        "input file grew beyond 128 MiB limit",
    )?;
    let digest = hash(&bytes);
    Ok((bytes, digest))
}

fn read_json(path: &Path) -> Result<(Value, String)> {
    let (bytes, digest) = read_bounded_bytes(path)?;
    Ok((serde_json::from_slice(&bytes)?, digest))
}

fn write_exclusive_json(path: &Path, value: &Value) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(value)?;
    let mut out = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    out.write_all(&bytes)?;
    out.write_all(b"\n")?;
    Ok(())
}

fn n_i64(v: &Value, key: &str) -> Result<i64> {
    v.get(key)
        .and_then(Value::as_i64)
        .filter(|x| *x >= 0)
        .ok_or_else(|| format!("missing/nonnegative integer field: {key}").into())
}

fn n_u64(v: &Value, key: &str) -> Result<u64> {
    v.get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("missing/non-integer field: {key}").into())
}

fn sha64_field<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    let s = v
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing string SHA256 field: {key}"))?;
    require(
        s.len() == 64
            && s.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        &format!("invalid 64-hex SHA256 field: {key}"),
    )?;
    Ok(s)
}

fn count_val(c: &Counts, key: &str) -> Result<i64> {
    c.get(key)
        .copied()
        .ok_or_else(|| format!("missing count key: {key}").into())
}

fn array<'a>(v: &'a Value, key: &str) -> Result<&'a Vec<Value>> {
    v.get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("missing array: {key}").into())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadMode {
    Route,
    Flock { kept: u32 },
    Relation { kept: u32, sweeps: u32 },
}

impl ReadMode {
    pub fn parse(v: &Value) -> Result<Self> {
        let obj = v.as_object().ok_or("read_spec must be an object")?;
        let kind = v
            .get("kind")
            .and_then(Value::as_str)
            .ok_or("read_spec missing kind")?;
        match kind {
            "route" => {
                require(obj.len() == 1, "route read_spec accepts no extra fields")?;
                Ok(Self::Route)
            }
            "flock" => {
                require(
                    obj.len() == 2,
                    "flock read_spec requires only kind and kept",
                )?;
                let kept = u32::try_from(n_u64(v, "kept")?)?;
                require((1..=8).contains(&kept), "flock kept count requires 1..=8")?;
                Ok(Self::Flock { kept })
            }
            "relation" => {
                require(
                    obj.len() == 3,
                    "relation read_spec requires only kind, kept, and sweeps",
                )?;
                let kept = u32::try_from(n_u64(v, "kept")?)?;
                let sweeps = u32::try_from(n_u64(v, "sweeps")?)?;
                require(
                    (1..=8).contains(&kept),
                    "relation kept count must be in 1..=8",
                )?;
                require(
                    (1..=5).contains(&sweeps),
                    "relation sweep count must be in 1..=5",
                )?;
                Ok(Self::Relation { kept, sweeps })
            }
            _ => Err(format!("unknown read_spec kind: {kind}").into()),
        }
    }

    pub fn gate_count(self, blocks: u32, stages: u32) -> Result<u32> {
        let base = blocks
            .checked_mul(stages)
            .ok_or("gate inventory overflow")?;
        match self {
            Self::Route | Self::Flock { .. } => Ok(base),
            Self::Relation { .. } => {
                let rel = RELATION_EDGE_TYPES
                    .checked_mul(stages)
                    .ok_or("gate inventory overflow")?;
                Ok(base.checked_add(rel).ok_or("gate inventory overflow")?)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleKind {
    CanonicalVectorThenReadV1,
    InterleavedReadEvery8V1,
}

impl ScheduleKind {
    pub fn parse(v: &Value) -> Result<Self> {
        match v.as_str().ok_or("schedule must be a string")? {
            "canonical_vector_then_read_v1" => Ok(Self::CanonicalVectorThenReadV1),
            "interleaved_read_every_8_v1" => Ok(Self::InterleavedReadEvery8V1),
            other => Err(format!("unknown schedule policy: {other}").into()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterFamily {
    VectorBlock,
    Route,
    GateCombine,
    GateRelation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScheduledSlot {
    pub family: ParameterFamily,
    pub parameter_index: u32,
}

pub fn binder_stages(context: u32) -> Result<u32> {
    require((1..=243).contains(&context), "context must be in 1..=243")?;
    let mut width = context;
    let mut stages = 1u32;
    while width >= 3 {
        width /= 3;
        stages += 1;
    }
    Ok(stages)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScheduleGeometry {
    pub vocabulary: u32,
    pub blocks: u32,
    pub context: u32,
    pub stages: u32,
    pub read_mode: ReadMode,
    pub schedule: ScheduleKind,
    pub vector_parameters: u32,
    pub combine_parameters: u32,
    pub relation_parameters: u32,
    pub read_parameters: u32,
    pub parameter_count: u32,
    pub schedule_cycle: u32,
}

impl ScheduleGeometry {
    pub fn new(
        vocabulary: u32,
        blocks: u32,
        context: u32,
        read_mode: ReadMode,
        schedule: ScheduleKind,
    ) -> Result<Self> {
        require(
            (2..=8192).contains(&vocabulary) && (2..=16).contains(&blocks),
            "invalid vocabulary/blocks shape",
        )?;
        let stages = binder_stages(context)?;
        let vector_parameters = vocabulary
            .checked_mul(blocks)
            .ok_or("vector inventory overflow")?;
        let combine_parameters = blocks
            .checked_mul(stages)
            .ok_or("combine inventory overflow")?;
        let read_parameters = read_mode.gate_count(blocks, stages)?;
        let relation_parameters = read_parameters
            .checked_sub(combine_parameters)
            .ok_or("relation inventory underflow")?;
        let parameter_count = vector_parameters
            .checked_add(read_parameters)
            .ok_or("parameter inventory overflow")?;
        let schedule_cycle = match schedule {
            ScheduleKind::CanonicalVectorThenReadV1 => parameter_count,
            ScheduleKind::InterleavedReadEvery8V1 => vector_parameters
                .checked_add(combine_parameters)
                .ok_or("schedule cycle overflow")?,
        };
        Ok(Self {
            vocabulary,
            blocks,
            context,
            stages,
            read_mode,
            schedule,
            vector_parameters,
            combine_parameters,
            relation_parameters,
            read_parameters,
            parameter_count,
            schedule_cycle,
        })
    }

    pub fn slot(&self, step: u32) -> Result<ScheduledSlot> {
        require(
            step < self.schedule_cycle,
            "schedule step outside declared schedule_cycle",
        )?;
        match self.schedule {
            ScheduleKind::CanonicalVectorThenReadV1 => {
                if step < self.vector_parameters {
                    Ok(ScheduledSlot {
                        family: ParameterFamily::VectorBlock,
                        parameter_index: step,
                    })
                } else {
                    let read_idx = step - self.vector_parameters;
                    Ok(self.read_slot(read_idx))
                }
            }
            ScheduleKind::InterleavedReadEvery8V1 => {
                if step % 8 == 7 {
                    let read_idx = (step / 8) % self.read_parameters;
                    Ok(self.read_slot(read_idx))
                } else {
                    let vec_idx = (step - step / 8) % self.vector_parameters;
                    Ok(ScheduledSlot {
                        family: ParameterFamily::VectorBlock,
                        parameter_index: vec_idx,
                    })
                }
            }
        }
    }

    fn read_slot(&self, read_idx: u32) -> ScheduledSlot {
        match self.read_mode {
            ReadMode::Route => ScheduledSlot {
                family: ParameterFamily::Route,
                parameter_index: read_idx,
            },
            ReadMode::Flock { .. } => ScheduledSlot {
                family: ParameterFamily::GateCombine,
                parameter_index: read_idx,
            },
            ReadMode::Relation { .. } => {
                if read_idx < self.combine_parameters {
                    ScheduledSlot {
                        family: ParameterFamily::GateCombine,
                        parameter_index: read_idx,
                    }
                } else {
                    ScheduledSlot {
                        family: ParameterFamily::GateRelation,
                        parameter_index: read_idx,
                    }
                }
            }
        }
    }

    pub fn validate_cursor(
        &self,
        cursor: CursorSpec,
        eligible_train_targets: u64,
        batch_rows: u32,
    ) -> Result<()> {
        require(
            (1..=64).contains(&batch_rows),
            "batch_rows must be in 1..=64",
        )?;
        require(
            eligible_train_targets > 0,
            "eligible_train_targets must be > 0",
        )?;
        let batches_per_pass = eligible_train_targets.div_ceil(u64::from(batch_rows));
        let expected_coord = (cursor.completed_batches % u64::from(self.schedule_cycle)) as u32;
        require(
            cursor.accepted_updates <= cursor.completed_batches
                && cursor.corpus_pass == cursor.completed_batches / batches_per_pass
                && cursor.next_target
                    == (cursor.completed_batches % batches_per_pass) * u64::from(batch_rows)
                && cursor.next_coordinate == expected_coord,
            "resume cursor differs from declared corpus/schedule cycle",
        )?;
        Ok(())
    }

    pub fn advance_cursor(
        &self,
        cursor: CursorSpec,
        accepted: bool,
        eligible_train_targets: u64,
        batch_rows: u32,
    ) -> Result<CursorSpec> {
        self.validate_cursor(cursor, eligible_train_targets, batch_rows)?;
        let batches_per_pass = eligible_train_targets.div_ceil(u64::from(batch_rows));
        let completed_batches = cursor
            .completed_batches
            .checked_add(1)
            .ok_or("completed_batches overflow")?;
        let accepted_updates = cursor
            .accepted_updates
            .checked_add(u64::from(accepted))
            .ok_or("accepted_updates overflow")?;
        let next = CursorSpec {
            completed_batches,
            accepted_updates,
            corpus_pass: completed_batches / batches_per_pass,
            next_target: (completed_batches % batches_per_pass) * u64::from(batch_rows),
            next_coordinate: (completed_batches % u64::from(self.schedule_cycle)) as u32,
        };
        self.validate_cursor(next, eligible_train_targets, batch_rows)?;
        Ok(next)
    }

    pub fn validate_checkpoint_resume(
        &self,
        checkpoint_doc: &Value,
        expected_seed: u64,
        eligible_train_targets: u64,
        batch_rows: u32,
    ) -> Result<CursorSpec> {
        let driver_ckpt = if checkpoint_doc.get("schema").and_then(Value::as_str)
            == Some(COMPLETION_CHECKPOINT_SCHEMA)
        {
            checkpoint_doc
                .get("checkpoint")
                .ok_or("completion checkpoint missing inner checkpoint")?
        } else {
            checkpoint_doc
        };
        require(
            driver_ckpt.get("schema").and_then(Value::as_str) == Some(CHECKPOINT_SCHEMA),
            "invalid DriverCheckpoint schema",
        )?;
        let model = driver_ckpt
            .get("model")
            .ok_or("DriverCheckpoint missing model")?;
        let model_sha = hash(&serde_json::to_vec(model)?);
        require(
            driver_ckpt.get("model_sha256").and_then(Value::as_str) == Some(model_sha.as_str()),
            "DriverCheckpoint model_sha256 mismatch",
        )?;
        require(
            model.get("schema").and_then(Value::as_str) == Some(MODEL_SCHEMA),
            "invalid model snapshot schema",
        )?;
        let spec = model.get("spec").ok_or("model snapshot missing spec")?;
        require(
            u32::try_from(n_u64(spec, "vocabulary")?)? == self.vocabulary
                && u32::try_from(n_u64(spec, "blocks")?)? == self.blocks
                && u32::try_from(n_u64(spec, "context")?)? == self.context,
            "checkpoint model spec differs from schedule geometry",
        )?;
        let model_read = ReadMode::parse(
            model
                .get("read_spec")
                .ok_or("model snapshot missing read_spec")?,
        )?;
        require(
            model_read == self.read_mode,
            "checkpoint model read_spec differs from declared schedule read_mode",
        )?;
        require(
            n_u64(model, "seed")? == expected_seed,
            "checkpoint model seed mismatch",
        )?;
        if let Some(lineage) = driver_ckpt.get("lineage") {
            let cfg = lineage
                .get("config")
                .ok_or("checkpoint lineage missing config")?;
            let ckpt_sched = ScheduleKind::parse(
                cfg.get("schedule")
                    .ok_or("checkpoint lineage.config missing schedule")?,
            )?;
            require(
                ckpt_sched == self.schedule,
                "checkpoint schedule policy differs from declared schedule",
            )?;
            let ckpt_read = ReadMode::parse(
                cfg.get("read_spec")
                    .ok_or("checkpoint lineage.config missing read_spec")?,
            )?;
            require(
                ckpt_read == self.read_mode,
                "checkpoint lineage read_spec differs from declared read_mode",
            )?;
            require(
                n_u64(cfg, "seed")? == expected_seed
                    && u32::try_from(n_u64(cfg, "batch_rows")?)? == batch_rows,
                "checkpoint lineage seed or batch_rows mismatch",
            )?;
            let l_spec = cfg
                .get("spec")
                .ok_or("checkpoint lineage.config missing spec")?;
            require(
                l_spec == spec,
                "checkpoint lineage.config.spec differs from model.spec",
            )?;
            if let Some(tc) = lineage.get("train_corpus") {
                require(
                    n_u64(tc, "eligible_targets")? == eligible_train_targets,
                    "checkpoint lineage train_corpus.eligible_targets mismatch",
                )?;
            }
        }
        let cursor = CursorSpec::parse(
            driver_ckpt
                .get("cursor")
                .ok_or("DriverCheckpoint missing cursor")?,
        )?;
        require(
            n_u64(model, "epoch")? == cursor.accepted_updates,
            "checkpoint model.epoch differs from cursor.accepted_updates",
        )?;
        self.validate_cursor(cursor, eligible_train_targets, batch_rows)?;
        Ok(cursor)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorSpec {
    pub completed_batches: u64,
    pub accepted_updates: u64,
    pub corpus_pass: u64,
    pub next_target: u64,
    pub next_coordinate: u32,
}

impl CursorSpec {
    pub fn parse(v: &Value) -> Result<Self> {
        Ok(Self {
            completed_batches: n_u64(v, "completed_batches")?,
            accepted_updates: n_u64(v, "accepted_updates")?,
            corpus_pass: n_u64(v, "corpus_pass")?,
            next_target: n_u64(v, "next_target")?,
            next_coordinate: u32::try_from(n_u64(v, "next_coordinate")?)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetRowClass {
    pub position: usize,
    pub query_token: u32,
    pub target_token: u32,
    pub query_recurs: bool,
    pub copy_successor: bool,
    pub distance_bucket: Option<usize>,
    pub most_recent_distance: Option<usize>,
    pub any_occurrence_successor: bool,
    pub copy_ceiling: bool,
}

const SEALED_2471_ROUTE_CANONICAL_EVAL_SHAS: [&str; 2] = [
    "e4bcf43859fa0645db2ca0d5fe620f18620990cbb7efb4f28e6873f5a57f90c3",
    "4fff1d3c04ff6107666b2b8281ef2bbedf3019eeff6bff682bbbb8687bf888c8",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubpanelPartition {
    pub vocabulary: u32,
    pub bos_id: u32,
    pub tokens_sha256: String,
    pub mask_sha256: String,
    pub ordered_row_binding_sha256: String,
    pub payload_tokens: usize,
    pub documents: usize,
    pub context: usize,
    pub eligible_rows: usize,
    pub query_recur_rows: usize,
    pub copy_successor_rows: usize,
    pub distance_bucket_rows: [usize; 4],
    pub distance_one_rows: usize,
    pub any_occurrence_successor_rows: usize,
    pub copy_ceiling_rows: usize,
    pub complement_rows: usize,
    pub rows: Vec<TargetRowClass>,
}

pub fn partition_corpus(
    token_bytes: &[u8],
    mask_bytes: &[u8],
    context: usize,
    bos: u32,
) -> Result<SubpanelPartition> {
    require(
        (1..=243).contains(&context),
        "partition context must be in 1..=243",
    )?;
    require(
        token_bytes.len() >= 64 && &token_bytes[..4] == b"UORT",
        "invalid UORT token header",
    )?;
    let version = u32::from_le_bytes(token_bytes[4..8].try_into().unwrap());
    require(version == 1, "unsupported UORT version")?;
    let count = u64::from_le_bytes(token_bytes[8..16].try_into().unwrap()) as usize;
    let vocab = u32::from_le_bytes(token_bytes[16..20].try_into().unwrap());
    require((2..=8192).contains(&vocab), "invalid UORT vocabulary")?;
    require(bos < vocab, "BOS id outside vocabulary")?;
    require(
        token_bytes.len() == 64 + 2 * count,
        "UORT payload byte length mismatch",
    )?;
    require(mask_bytes.len() == count, "response mask length mismatch")?;
    require(
        mask_bytes.iter().all(|&b| b <= 1),
        "response mask contains non-binary value",
    )?;
    let tokens_sha256 = hash(token_bytes);
    let mask_sha256 = hash(mask_bytes);
    let tokens: Vec<u32> = token_bytes[64..]
        .chunks_exact(2)
        .map(|w| u32::from(u16::from_le_bytes([w[0], w[1]])))
        .collect();
    require(
        !tokens.is_empty() && tokens[0] == bos,
        "corpus must begin with BOS token",
    )?;
    require(
        tokens.iter().all(|&t| t < vocab),
        "corpus token exceeds declared vocabulary",
    )?;
    let documents: Vec<usize> = tokens
        .iter()
        .enumerate()
        .filter(|(_, &t)| t == bos)
        .map(|(p, _)| p)
        .collect();
    let eligible: Vec<usize> = (0..count)
        .filter(|&p| p > 0 && tokens[p] != bos && mask_bytes[p] == 1)
        .collect();
    let mut query_recur_rows = 0usize;
    let mut copy_successor_rows = 0usize;
    let mut distance_bucket_rows = [0usize; 4];
    let mut distance_one_rows = 0usize;
    let mut any_occurrence_successor_rows = 0usize;
    let mut copy_ceiling_rows = 0usize;
    let mut rows = Vec::with_capacity(eligible.len());

    for &p in &eligible {
        let doc = documents.partition_point(|&s| s <= p) - 1;
        let start = p.saturating_sub(context).max(documents[doc]);
        require(start < p, "empty causal prefix for eligible target")?;
        let query = tokens[p - 1];
        let target = tokens[p];
        let mut most_recent: Option<usize> = None;
        let mut any = false;
        let mut present = false;
        for j in start..p - 1 {
            if tokens[j] == target {
                present = true;
            }
            if tokens[j] == query {
                most_recent = Some(j);
                if tokens[j + 1] == target {
                    any = true;
                }
            }
        }
        if tokens[p - 1] == target {
            present = true;
        }
        if present {
            copy_ceiling_rows += 1;
        }
        let mut copy_successor = false;
        let mut distance_bucket = None;
        let mut most_recent_distance = None;
        if let Some(j) = most_recent {
            query_recur_rows += 1;
            let d = (p - 1) - j;
            most_recent_distance = Some(d);
            if tokens[j + 1] == target {
                copy_successor = true;
                copy_successor_rows += 1;
                if d == 1 {
                    distance_one_rows += 1;
                }
                let b = if d <= 8 {
                    0
                } else if d <= 32 {
                    1
                } else if d <= 96 {
                    2
                } else {
                    3
                };
                distance_bucket_rows[b] += 1;
                distance_bucket = Some(b);
            }
        }
        if any {
            any_occurrence_successor_rows += 1;
        }
        rows.push(TargetRowClass {
            position: p,
            query_token: query,
            target_token: target,
            query_recurs: most_recent.is_some(),
            copy_successor,
            distance_bucket,
            most_recent_distance,
            any_occurrence_successor: any,
            copy_ceiling: present,
        });
    }
    let complement_rows = eligible
        .len()
        .checked_sub(copy_successor_rows)
        .ok_or("complement row count underflow")?;
    require(
        distance_bucket_rows.iter().sum::<usize>() == copy_successor_rows,
        "distance buckets do not sum to copy_successor_rows",
    )?;
    let mut row_binding_hasher = Sha256::new();
    for (ordinal, row) in rows.iter().enumerate() {
        row_binding_hasher.update((ordinal as u64).to_le_bytes());
        row_binding_hasher.update((row.position as u64).to_le_bytes());
        row_binding_hasher.update(row.query_token.to_le_bytes());
        row_binding_hasher.update(row.target_token.to_le_bytes());
        row_binding_hasher.update([u8::from(row.copy_successor)]);
        row_binding_hasher.update([row.distance_bucket.map(|b| (b + 1) as u8).unwrap_or(0)]);
    }
    let ordered_row_binding_sha256 = format!("{:x}", row_binding_hasher.finalize());
    Ok(SubpanelPartition {
        vocabulary: vocab,
        bos_id: bos,
        tokens_sha256,
        mask_sha256,
        ordered_row_binding_sha256,
        payload_tokens: count,
        documents: documents.len(),
        context,
        eligible_rows: eligible.len(),
        query_recur_rows,
        copy_successor_rows,
        distance_bucket_rows,
        distance_one_rows,
        any_occurrence_successor_rows,
        copy_ceiling_rows,
        complement_rows,
        rows,
    })
}

const SLICE_KEYS: [&str; 17] = [
    "scored_rows",
    "top1_correct",
    "top8_correct",
    "target_cell_admitted_rows",
    "admitted_strict_wins",
    "token_strict_win_eligible_rows",
    "token_hinge_sum",
    "address_hinge_sum",
    "total_loss_numerator_sum",
    "token_eligible_rows",
    "address_eligible_rows",
    "token_rival_count_sum",
    "address_rival_count_sum",
    "admitted_cell_count_sum",
    "eligible_any_rows",
    "eligible_unviolated_rows",
    "token_unviolated_eligible_rows",
];

fn zero_slice() -> Counts {
    let mut c = Counts::new();
    for k in SLICE_KEYS {
        c.insert(k.into(), 0);
    }
    c.insert("address_unviolated_eligible_rows".into(), 0);
    c
}

fn add_count(c: &mut Counts, key: &str, value: i64) -> Result<()> {
    let slot = c.entry(key.into()).or_default();
    *slot = slot.checked_add(value).ok_or("receipt counter overflow")?;
    Ok(())
}

fn add_slices(a: &Counts, b: &Counts) -> Result<Counts> {
    let mut out = zero_slice();
    for (k, v) in a {
        add_count(&mut out, k, *v)?;
    }
    for (k, v) in b {
        add_count(&mut out, k, *v)?;
    }
    Ok(out)
}

fn parse_row_metrics(row: &Value, vocabulary: u32) -> Result<Counts> {
    let t = n_i64(row, "token_hinge")?;
    let a = n_i64(row, "address_hinge")?;
    let total = n_i64(row, "total_loss_numerator")?;
    let tr = n_i64(row, "token_rivals")?;
    let max_rivals = i64::from(vocabulary) - 1;
    require(
        tr <= max_rivals,
        "token_rivals exceeds nonself vocabulary bound",
    )?;
    let ar = n_i64(row, "address_rivals")?;
    require(
        !(tr == 0 && t > 0) && !(ar == 0 && a > 0),
        "nonzero hinge with zero rivals",
    )?;
    let cells = n_i64(row, "admitted_cells")?;
    let h1 = n_i64(row, "top1_correct")?;
    let h8 = n_i64(row, "top8_correct")?;
    let target_admitted = cells.checked_sub(ar).ok_or("address_rivals underflow")?;
    require(
        (1..=8).contains(&cells) && (0..=1).contains(&target_admitted),
        "invalid admitted_cells or target_cell_admitted_rows",
    )?;
    require(
        h1 <= h8 && h8 <= target_admitted,
        "rank/admission nesting violated",
    )?;
    require(
        !(h1 == 1 && tr > 0 && t > 32),
        "top1_correct row cannot have token_hinge > 32",
    )?;
    let min_admitted_tokens = i64::from(vocabulary.min(8));
    require(
        (min_admitted_tokens..=i64::from(vocabulary)).contains(&(tr + target_admitted)),
        "admitted token count outside bounds",
    )?;
    let strict = tr > 0 && t < 32;
    let admitted_strict = strict && target_admitted == 1;
    require(
        !admitted_strict || h1 == 1,
        "admitted strict win must be top1_correct",
    )?;
    let weighted = t
        .checked_mul(8)
        .and_then(|v| a.checked_mul(32).and_then(|w| v.checked_add(w)))
        .ok_or("weighted row loss overflow")?;
    require(total == weighted, "weighted row loss mismatch")?;
    let top8 = array(row, "top8")?;
    let expected_top_len = (vocabulary as usize).min(8);
    require(top8.len() == expected_top_len, "wrong top8 array length")?;
    let mut seen = BTreeSet::new();
    for id in top8 {
        let id = id.as_u64().ok_or("non-integer token ID in top8")?;
        require(
            id < u64::from(vocabulary) && seen.insert(id),
            "out-of-range or duplicate token ID in top8",
        )?;
    }
    let mut out = zero_slice();
    for (k, v) in [
        ("scored_rows", 1),
        ("top1_correct", h1),
        ("top8_correct", h8),
        ("target_cell_admitted_rows", target_admitted),
        ("admitted_strict_wins", i64::from(admitted_strict)),
        ("token_strict_win_eligible_rows", i64::from(strict)),
        ("token_hinge_sum", t),
        ("address_hinge_sum", a),
        ("total_loss_numerator_sum", total),
        ("token_eligible_rows", i64::from(tr > 0)),
        ("address_eligible_rows", i64::from(ar > 0)),
        ("token_rival_count_sum", tr),
        ("address_rival_count_sum", ar),
        ("admitted_cell_count_sum", cells),
        ("eligible_any_rows", i64::from(tr > 0 || ar > 0)),
        (
            "eligible_unviolated_rows",
            i64::from(total == 0 && (tr > 0 || ar > 0)),
        ),
        (
            "token_unviolated_eligible_rows",
            i64::from(tr > 0 && t == 0),
        ),
        (
            "address_unviolated_eligible_rows",
            i64::from(ar > 0 && a == 0),
        ),
    ] {
        out.insert(k.into(), v);
    }
    Ok(out)
}

fn verify_stored_aggregate(computed: &Counts, stored: &Value) -> Result<()> {
    require(
        n_i64(stored, "loss_denominator_per_row")? == 256,
        "loss_denominator_per_row mismatch",
    )?;
    for key in [
        "scored_rows",
        "top1_correct",
        "top8_correct",
        "token_hinge_sum",
        "address_hinge_sum",
        "total_loss_numerator_sum",
        "token_eligible_rows",
        "address_eligible_rows",
        "token_rival_count_sum",
        "address_rival_count_sum",
        "admitted_cell_count_sum",
        "eligible_any_rows",
        "eligible_unviolated_rows",
        "token_unviolated_eligible_rows",
        "address_unviolated_eligible_rows",
        "token_strict_win_eligible_rows",
    ] {
        require(
            n_i64(stored, key)? == *computed.get(key).ok_or("missing computed key")?,
            &format!("stored aggregate mismatch on {key}"),
        )?;
    }
    let b3 = n_i64(stored, "admitted_cell_count_sum")? - n_i64(stored, "address_rival_count_sum")?;
    require(
        b3 == *computed
            .get("target_cell_admitted_rows")
            .ok_or("missing target_cell_admitted_rows")?,
        "B3 target_cell_admitted_rows mismatch",
    )?;
    Ok(())
}

fn unwrap_evaluation(doc: &Value) -> Result<&Value> {
    if doc.get("schema").and_then(Value::as_str) == Some(COMPLETION_EVAL_SCHEMA) {
        doc.get("evaluation")
            .ok_or_else(|| "completion evaluation wrapper missing inner evaluation".into())
    } else if doc
        .get("baseline_evaluation")
        .and_then(|e| e.get("schema"))
        .and_then(Value::as_str)
        == Some(DRIVER_SCHEMA)
    {
        doc.get("baseline_evaluation")
            .ok_or_else(|| "missing baseline_evaluation".into())
    } else {
        Ok(doc)
    }
}

fn slice_with_conditional(c: &Counts) -> Value {
    let mut v = serde_json::to_value(c).unwrap_or_else(|_| json!({}));
    let b3 = c.get("target_cell_admitted_rows").copied().unwrap_or(0);
    let top1 = c.get("top1_correct").copied().unwrap_or(0);
    let top8 = c.get("top8_correct").copied().unwrap_or(0);
    let adm_strict = c.get("admitted_strict_wins").copied().unwrap_or(0);
    v["conditional_in_admitted_b3"] = json!({
        "denominator_b3": b3,
        "top1_numerator": top1,
        "top8_numerator": top8,
        "admitted_strict_numerator": adm_strict
    });
    v
}

#[derive(Debug, Clone)]
pub struct SubpanelEvaluationAudit {
    pub full_panel: Counts,
    pub copy_successor: Counts,
    pub distance_buckets: [Counts; 4],
    pub any_occurrence_successor: Counts,
    pub complement: Counts,
}

impl SubpanelEvaluationAudit {
    pub fn to_report_json(&self) -> Value {
        json!({
            "full_panel": slice_with_conditional(&self.full_panel),
            "copy_successor_subpanel": slice_with_conditional(&self.copy_successor),
            "distance_buckets": {
                "d_1_8": slice_with_conditional(&self.distance_buckets[0]),
                "d_9_32": slice_with_conditional(&self.distance_buckets[1]),
                "d_33_96": slice_with_conditional(&self.distance_buckets[2]),
                "d_97_192": slice_with_conditional(&self.distance_buckets[3])
            },
            "any_occurrence_successor_subpanel": slice_with_conditional(&self.any_occurrence_successor),
            "complement_subpanel": slice_with_conditional(&self.complement)
        })
    }
}

pub fn validate_evaluation_population_binding(
    partition: &SubpanelPartition,
    eval: &Value,
    batch_rows: usize,
) -> Result<()> {
    let _ = sha64_field(eval, "evaluation_lineage_sha256")?;
    let lineage = eval
        .get("evaluation_lineage")
        .ok_or("evaluation receipt missing evaluation_lineage")?;
    let cfg = lineage
        .get("config")
        .ok_or("evaluation_lineage missing config")?;
    let spec = cfg
        .get("spec")
        .ok_or("evaluation_lineage.config missing spec")?;
    let vocab = u32::try_from(n_u64(spec, "vocabulary")?)?;
    let blocks = u32::try_from(n_u64(spec, "blocks")?)?;
    let context = usize::try_from(n_u64(spec, "context")?)?;
    let kept_cells = u32::try_from(n_u64(spec, "kept_cells")?)?;
    require(
        vocab == partition.vocabulary
            && context == partition.context
            && (2..=16).contains(&blocks)
            && (1..=16).contains(&kept_cells)
            && kept_cells <= vocab,
        "evaluation_lineage.config.spec mismatch against partition",
    )?;
    require(
        usize::try_from(n_u64(cfg, "batch_rows")?)? == batch_rows,
        "evaluation_lineage.config.batch_rows mismatch",
    )?;
    let _ = ReadMode::parse(
        cfg.get("read_spec")
            .ok_or("evaluation_lineage.config missing read_spec")?,
    )?;
    let _ = ScheduleKind::parse(
        cfg.get("schedule")
            .ok_or("evaluation_lineage.config missing schedule")?,
    )?;
    let _ = n_u64(cfg, "seed")?;
    let cfg_bos = cfg
        .get("bos")
        .ok_or("evaluation_lineage.config missing bos")?;
    require(
        u32::try_from(n_u64(cfg_bos, "id")?)? == partition.bos_id,
        "evaluation_lineage.config.bos.id mismatch against partition",
    )?;
    let cfg_dev = cfg
        .get("dev")
        .ok_or("evaluation_lineage.config missing dev")?;
    let cfg_tokens = cfg_dev
        .get("tokens")
        .ok_or("evaluation_lineage.config.dev missing tokens")?;
    let cfg_mask = cfg_dev
        .get("mask")
        .ok_or("evaluation_lineage.config.dev missing mask")?;
    require(
        sha64_field(cfg_tokens, "sha256")? == partition.tokens_sha256,
        "evaluation_lineage.config.dev.tokens.sha256 mismatch against partition tokens",
    )?;
    require(
        sha64_field(cfg_mask, "sha256")? == partition.mask_sha256,
        "evaluation_lineage.config.dev.mask.sha256 mismatch against partition mask",
    )?;
    let has_file_bytes = cfg_tokens.get("bytes").is_some() || cfg_mask.get("bytes").is_some();
    if has_file_bytes {
        require(
            usize::try_from(n_u64(cfg_tokens, "bytes")?)? == 64 + 2 * partition.payload_tokens
                && usize::try_from(n_u64(cfg_mask, "bytes")?)? == partition.payload_tokens,
            "evaluation_lineage.config.dev byte length mismatch against partition",
        )?;
    }
    let has_expected_counts = cfg_dev.get("expected_payload_tokens").is_some()
        || cfg_dev.get("expected_eligible_targets").is_some();
    if has_expected_counts {
        require(
            usize::try_from(n_u64(cfg_dev, "expected_payload_tokens")?)?
                == partition.payload_tokens
                && usize::try_from(n_u64(cfg_dev, "expected_eligible_targets")?)?
                    == partition.eligible_rows,
            "evaluation_lineage.config.dev expected payload/eligible counts mismatch against partition",
        )?;
    }
    require(
        has_file_bytes || has_expected_counts,
        "evaluation_lineage.config.dev missing byte length or expected count bindings",
    )?;

    let dev = lineage.get("dev").ok_or("evaluation_lineage missing dev")?;
    let dev_bos = dev.get("bos").ok_or("evaluation_lineage.dev missing bos")?;
    require(
        u32::try_from(n_u64(dev_bos, "id")?)? == partition.bos_id,
        "evaluation_lineage.dev.bos.id mismatch against partition",
    )?;
    require(
        u32::try_from(n_u64(dev, "vocabulary")?)? == partition.vocabulary
            && usize::try_from(n_u64(dev, "documents")?)? == partition.documents
            && usize::try_from(n_u64(dev, "payload_tokens")?)? == partition.payload_tokens
            && usize::try_from(n_u64(dev, "mask_bytes")?)? == partition.payload_tokens
            && usize::try_from(n_u64(dev, "mask_selected_positions")?)? == partition.eligible_rows
            && usize::try_from(n_u64(dev, "eligible_targets")?)? == partition.eligible_rows
            && dev.get("window_policy").and_then(Value::as_str) == Some(WINDOW_POLICY),
        "evaluation_lineage.dev population metadata mismatch against partition",
    )?;
    let dev_source = dev
        .get("source")
        .ok_or("evaluation_lineage.dev missing source")?;
    require(
        dev_source == cfg_dev,
        "evaluation_lineage.dev.source disagrees with evaluation_lineage.config.dev",
    )?;

    let mb = eval
        .get("model_bindings")
        .ok_or("evaluation receipt missing model_bindings")?;
    require(
        mb.get("schema").and_then(Value::as_str) == Some(MODEL_SCHEMA)
            && mb.get("spec") == Some(spec)
            && mb.get("read_spec") == cfg.get("read_spec")
            && mb.get("seed") == cfg.get("seed"),
        "evaluation model_bindings disagrees with evaluation_lineage.config",
    )?;
    if eval.get("ordered_row_binding_sha256").is_some() {
        require(
            sha64_field(eval, "ordered_row_binding_sha256")?
                == partition.ordered_row_binding_sha256,
            "evaluation ordered_row_binding_sha256 mismatch against partition",
        )?;
    }
    Ok(())
}

fn verify_row_correspondence(
    row: &Value,
    expected: &TargetRowClass,
    row_ordinal: usize,
    require_explicit_row_binding: bool,
) -> Result<()> {
    let top8 = array(row, "top8")?;
    let target_u64 = u64::from(expected.target_token);
    let top1 = n_i64(row, "top1_correct")?;
    let top8_hit = n_i64(row, "top8_correct")?;
    let top1_matches_target = top8.first().and_then(Value::as_u64) == Some(target_u64);
    let top8_contains_target = top8.iter().any(|v| v.as_u64() == Some(target_u64));
    require(
        (top1 == 1) == top1_matches_target,
        "row top1_correct disagrees with top8[0] == expected target_token",
    )?;
    require(
        (top8_hit == 1) == top8_contains_target,
        "row top8_correct disagrees with expected target_token membership in top8",
    )?;
    if row.get("row_ordinal").is_some() {
        require(
            usize::try_from(n_u64(row, "row_ordinal")?)? == row_ordinal,
            "row_ordinal mismatch against partition row order",
        )?;
    }
    if require_explicit_row_binding || row.get("target_position").is_some() {
        require(
            usize::try_from(n_u64(row, "target_position")?)? == expected.position,
            "row target_position does not match partition row position",
        )?;
    }
    if require_explicit_row_binding || row.get("target_token").is_some() {
        require(
            u32::try_from(n_u64(row, "target_token")?)? == expected.target_token,
            "row target_token does not match partition row target_token",
        )?;
    }
    if row.get("query_token").is_some() {
        require(
            u32::try_from(n_u64(row, "query_token")?)? == expected.query_token,
            "row query_token does not match partition row query_token",
        )?;
    }
    Ok(())
}

pub fn audit_evaluation_subpanels(
    partition: &SubpanelPartition,
    eval_doc: &Value,
    batch_rows: usize,
) -> Result<SubpanelEvaluationAudit> {
    audit_evaluation_subpanels_with_mode(partition, eval_doc, batch_rows, true)
}

fn audit_evaluation_subpanels_with_mode(
    partition: &SubpanelPartition,
    eval_doc: &Value,
    batch_rows: usize,
    require_explicit_row_binding: bool,
) -> Result<SubpanelEvaluationAudit> {
    require(
        (1..=64).contains(&batch_rows),
        "batch_rows must be in 1..=64",
    )?;
    let eval = unwrap_evaluation(eval_doc)?;
    require(
        eval.get("schema").and_then(Value::as_str) == Some(DRIVER_SCHEMA)
            && eval.get("scored_set_policy").and_then(Value::as_str) == Some(WINDOW_POLICY)
            && eval
                .get("evaluation_mutates_model")
                .and_then(Value::as_bool)
                == Some(false)
            && eval
                .get("updates_applied_by_evaluation")
                .and_then(Value::as_u64)
                == Some(0),
        "evaluation receipt header/immutability check failed",
    )?;
    validate_evaluation_population_binding(partition, eval, batch_rows)?;
    let batches = array(eval, "batches")?;
    let expected_batches = partition.eligible_rows.div_ceil(batch_rows);
    require(
        batches.len() == expected_batches,
        "evaluation batch count does not match partition eligible_rows",
    )?;

    let mut full_panel = zero_slice();
    let mut copy_successor = zero_slice();
    let mut distance_buckets = [zero_slice(), zero_slice(), zero_slice(), zero_slice()];
    let mut any_occurrence_successor = zero_slice();
    let mut complement = zero_slice();
    let mut row_cursor = 0usize;

    for (b_idx, batch) in batches.iter().enumerate() {
        if batch.get("batch_index").is_some() {
            require(
                usize::try_from(n_u64(batch, "batch_index")?)? == b_idx,
                "evaluation batch_index mismatch",
            )?;
        }
        let rows = array(batch, "rows")?;
        let expected_len = if b_idx + 1 == expected_batches {
            let rem = partition.eligible_rows % batch_rows;
            if rem == 0 {
                batch_rows
            } else {
                rem
            }
        } else {
            batch_rows
        };
        require(
            rows.len() == expected_len,
            "evaluation batch row length mismatch",
        )?;
        if batch.get("target_positions").is_some() {
            let target_positions = array(batch, "target_positions")?;
            require(
                target_positions.len() == expected_len,
                "batch target_positions length mismatch",
            )?;
            for (offset, pos_val) in target_positions.iter().enumerate() {
                require(
                    pos_val.as_u64() == Some(partition.rows[row_cursor + offset].position as u64),
                    "batch target_positions entry disagrees with partition row position",
                )?;
            }
        }
        let mut batch_sums = zero_slice();
        for row in rows {
            let class = &partition.rows[row_cursor];
            let row_metrics = parse_row_metrics(row, partition.vocabulary)?;
            verify_row_correspondence(row, class, row_cursor, require_explicit_row_binding)?;
            row_cursor += 1;
            for (k, v) in &row_metrics {
                add_count(&mut batch_sums, k, *v)?;
                add_count(&mut full_panel, k, *v)?;
                if class.copy_successor {
                    add_count(&mut copy_successor, k, *v)?;
                    let b = class.distance_bucket.ok_or("missing distance bucket")?;
                    add_count(&mut distance_buckets[b], k, *v)?;
                } else {
                    add_count(&mut complement, k, *v)?;
                }
                if class.any_occurrence_successor {
                    add_count(&mut any_occurrence_successor, k, *v)?;
                }
            }
        }
        verify_stored_aggregate(&batch_sums, &batch["aggregate"])?;
    }
    require(
        row_cursor == partition.eligible_rows,
        "scored row count mismatch against partition",
    )?;
    verify_stored_aggregate(&full_panel, &eval["aggregate"])?;

    let sum_succ_comp = add_slices(&copy_successor, &complement)?;
    require(
        sum_succ_comp == full_panel,
        "E2 failed: copy_successor + complement != full_panel",
    )?;
    let mut sum_buckets = zero_slice();
    for b in &distance_buckets {
        sum_buckets = add_slices(&sum_buckets, b)?;
    }
    require(
        sum_buckets == copy_successor,
        "E2 failed: distance buckets do not sum to copy_successor subpanel",
    )?;

    Ok(SubpanelEvaluationAudit {
        full_panel,
        copy_successor,
        distance_buckets,
        any_occurrence_successor,
        complement,
    })
}

fn zero_transitions() -> Counts {
    let mut c = Counts::new();
    for k in [
        "rows",
        "top1_gained",
        "top1_lost",
        "top1_retained",
        "top1_never",
        "top8_gained",
        "top8_lost",
        "top8_retained",
        "top8_never",
        "b3_gained",
        "b3_lost",
        "b3_retained",
        "b3_never",
        "admitted_strict_gained",
        "admitted_strict_lost",
        "loss_decreased_rows",
        "loss_increased_rows",
        "loss_unchanged_rows",
        "weighted_token_delta",
        "weighted_address_delta",
        "total_loss_numerator_delta",
    ] {
        c.insert(k.into(), 0);
    }
    c
}

fn record_transition(c: &mut Counts, before: &Counts, after: &Counts) -> Result<()> {
    add_count(c, "rows", 1)?;
    for (prefix, key) in [
        ("top1", "top1_correct"),
        ("top8", "top8_correct"),
        ("b3", "target_cell_admitted_rows"),
    ] {
        let b = count_val(before, key)?;
        let a = count_val(after, key)?;
        let bucket = match (b, a) {
            (0, 1) => format!("{prefix}_gained"),
            (1, 0) => format!("{prefix}_lost"),
            (1, 1) => format!("{prefix}_retained"),
            _ => format!("{prefix}_never"),
        };
        add_count(c, &bucket, 1)?;
    }
    let sb = count_val(before, "admitted_strict_wins")?;
    let sa = count_val(after, "admitted_strict_wins")?;
    if sb == 0 && sa == 1 {
        add_count(c, "admitted_strict_gained", 1)?;
    } else if sb == 1 && sa == 0 {
        add_count(c, "admitted_strict_lost", 1)?;
    }
    let td = 8 * (count_val(after, "token_hinge_sum")? - count_val(before, "token_hinge_sum")?);
    let ad =
        32 * (count_val(after, "address_hinge_sum")? - count_val(before, "address_hinge_sum")?);
    let ld = count_val(after, "total_loss_numerator_sum")?
        - count_val(before, "total_loss_numerator_sum")?;
    require(td + ad == ld, "row transition weighted loss delta mismatch")?;
    add_count(c, "weighted_token_delta", td)?;
    add_count(c, "weighted_address_delta", ad)?;
    add_count(c, "total_loss_numerator_delta", ld)?;
    let loss_bucket = if ld < 0 {
        "loss_decreased_rows"
    } else if ld > 0 {
        "loss_increased_rows"
    } else {
        "loss_unchanged_rows"
    };
    add_count(c, loss_bucket, 1)?;
    Ok(())
}

fn diff_slice(before: &Counts, after: &Counts) -> Result<Counts> {
    let mut out = zero_slice();
    for (k, av) in after {
        let bv = before.get(k).copied().ok_or("missing before slice key")?;
        out.insert(k.clone(), av.checked_sub(bv).ok_or("slice diff overflow")?);
    }
    Ok(out)
}

pub fn compare_evaluation_subpanels(
    partition: &SubpanelPartition,
    before_doc: &Value,
    after_doc: &Value,
    batch_rows: usize,
) -> Result<Value> {
    let before_audit = audit_evaluation_subpanels(partition, before_doc, batch_rows)?;
    let after_audit = audit_evaluation_subpanels(partition, after_doc, batch_rows)?;
    let before_eval = unwrap_evaluation(before_doc)?;
    let after_eval = unwrap_evaluation(after_doc)?;
    let before_batches = array(before_eval, "batches")?;
    let after_batches = array(after_eval, "batches")?;

    let mut full_tr = zero_transitions();
    let mut succ_tr = zero_transitions();
    let mut bucket_tr = [
        zero_transitions(),
        zero_transitions(),
        zero_transitions(),
        zero_transitions(),
    ];
    let mut any_tr = zero_transitions();
    let mut comp_tr = zero_transitions();
    let mut row_cursor = 0usize;

    for (b_batch, a_batch) in before_batches.iter().zip(after_batches.iter()) {
        let b_rows = array(b_batch, "rows")?;
        let a_rows = array(a_batch, "rows")?;
        for (b_row, a_row) in b_rows.iter().zip(a_rows.iter()) {
            let class = &partition.rows[row_cursor];
            let bm = parse_row_metrics(b_row, partition.vocabulary)?;
            let am = parse_row_metrics(a_row, partition.vocabulary)?;
            verify_row_correspondence(b_row, class, row_cursor, true)?;
            verify_row_correspondence(a_row, class, row_cursor, true)?;
            require(
                b_row.get("target_position") == a_row.get("target_position")
                    && b_row.get("target_token") == a_row.get("target_token"),
                "paired rows disagree on target_position or target_token",
            )?;
            row_cursor += 1;
            record_transition(&mut full_tr, &bm, &am)?;
            if class.copy_successor {
                record_transition(&mut succ_tr, &bm, &am)?;
                let b = class.distance_bucket.ok_or("missing distance bucket")?;
                record_transition(&mut bucket_tr[b], &bm, &am)?;
            } else {
                record_transition(&mut comp_tr, &bm, &am)?;
            }
            if class.any_occurrence_successor {
                record_transition(&mut any_tr, &bm, &am)?;
            }
        }
    }
    let full_delta = diff_slice(&before_audit.full_panel, &after_audit.full_panel)?;
    let succ_delta = diff_slice(&before_audit.copy_successor, &after_audit.copy_successor)?;
    let comp_delta = diff_slice(&before_audit.complement, &after_audit.complement)?;
    let any_delta = diff_slice(
        &before_audit.any_occurrence_successor,
        &after_audit.any_occurrence_successor,
    )?;
    let b0_delta = diff_slice(
        &before_audit.distance_buckets[0],
        &after_audit.distance_buckets[0],
    )?;
    let b1_delta = diff_slice(
        &before_audit.distance_buckets[1],
        &after_audit.distance_buckets[1],
    )?;
    let b2_delta = diff_slice(
        &before_audit.distance_buckets[2],
        &after_audit.distance_buckets[2],
    )?;
    let b3_delta = diff_slice(
        &before_audit.distance_buckets[3],
        &after_audit.distance_buckets[3],
    )?;

    require(
        count_val(&succ_tr, "top8_gained")? - count_val(&succ_tr, "top8_lost")?
            == count_val(&succ_delta, "top8_correct")?
            && count_val(&full_tr, "top8_gained")? - count_val(&full_tr, "top8_lost")?
                == count_val(&full_delta, "top8_correct")?,
        "row transition net top8 delta disagrees with aggregate delta",
    )?;

    Ok(json!({
        "aggregate_deltas_after_minus_before": {
            "full_panel": full_delta,
            "copy_successor_subpanel": succ_delta,
            "distance_buckets": {
                "d_1_8": b0_delta,
                "d_9_32": b1_delta,
                "d_33_96": b2_delta,
                "d_97_192": b3_delta
            },
            "any_occurrence_successor_subpanel": any_delta,
            "complement_subpanel": comp_delta
        },
        "row_transitions": {
            "full_panel": full_tr,
            "copy_successor_subpanel": succ_tr,
            "distance_buckets": {
                "d_1_8": bucket_tr[0],
                "d_9_32": bucket_tr[1],
                "d_33_96": bucket_tr[2],
                "d_97_192": bucket_tr[3]
            },
            "any_occurrence_successor_subpanel": any_tr,
            "complement_subpanel": comp_tr
        }
    }))
}

pub fn prepare_zero_trit_checkpoint(checkpoint_doc: &Value) -> Result<(Value, Value)> {
    let top_schema = checkpoint_doc
        .get("schema")
        .and_then(Value::as_str)
        .ok_or("checkpoint/snapshot missing schema")?;
    let mut outer = checkpoint_doc.clone();

    let (orig_model_sha, expected_epoch) = match top_schema {
        COMPLETION_CHECKPOINT_SCHEMA | CHECKPOINT_SCHEMA => {
            let driver_ckpt = if top_schema == COMPLETION_CHECKPOINT_SCHEMA {
                outer
                    .get("checkpoint")
                    .ok_or("completion checkpoint missing inner checkpoint")?
            } else {
                &outer
            };
            require(
                driver_ckpt.get("schema").and_then(Value::as_str) == Some(CHECKPOINT_SCHEMA),
                "invalid DriverCheckpoint schema",
            )?;
            let model_ref = driver_ckpt
                .get("model")
                .ok_or("DriverCheckpoint missing model")?;
            let orig_bytes = serde_json::to_vec(model_ref)?;
            let orig_sha = hash(&orig_bytes);
            require(
                driver_ckpt.get("model_sha256").and_then(Value::as_str) == Some(orig_sha.as_str()),
                "input checkpoint model_sha256 mismatch before zero-trit transform",
            )?;
            if let Some(lineage) = driver_ckpt.get("lineage") {
                let cfg = lineage
                    .get("config")
                    .ok_or("checkpoint lineage missing config")?;
                require(
                    cfg.get("spec") == model_ref.get("spec")
                        && cfg.get("read_spec") == model_ref.get("read_spec")
                        && cfg.get("seed") == model_ref.get("seed"),
                    "checkpoint lineage.config disagrees with model spec/read_spec/seed",
                )?;
            }
            let epoch = if let Some(cursor) = driver_ckpt.get("cursor") {
                Some(n_u64(cursor, "accepted_updates")?)
            } else {
                None
            };
            (orig_sha, epoch)
        }
        MODEL_SCHEMA => {
            let orig_bytes = serde_json::to_vec(&outer)?;
            (hash(&orig_bytes), None)
        }
        _ => return Err("unsupported schema for prepare_zero_trit_checkpoint".into()),
    };

    let model: &mut Value = match top_schema {
        COMPLETION_CHECKPOINT_SCHEMA => outer
            .get_mut("checkpoint")
            .and_then(|c| c.get_mut("model"))
            .ok_or("completion checkpoint missing inner model")?,
        CHECKPOINT_SCHEMA => outer
            .get_mut("model")
            .ok_or("DriverCheckpoint missing model")?,
        MODEL_SCHEMA => &mut outer,
        _ => unreachable!(),
    };
    require(
        model.get("schema").and_then(Value::as_str) == Some(MODEL_SCHEMA),
        "invalid model snapshot schema",
    )?;
    let _seed = n_u64(model, "seed")?;
    let epoch = n_u64(model, "epoch")?;
    if let Some(exp_epoch) = expected_epoch {
        require(
            epoch == exp_epoch,
            "checkpoint model.epoch differs from cursor.accepted_updates",
        )?;
    }
    let spec = model.get("spec").ok_or("model snapshot missing spec")?;
    let vocabulary = u32::try_from(n_u64(spec, "vocabulary")?)?;
    let blocks = u32::try_from(n_u64(spec, "blocks")?)?;
    let context = u32::try_from(n_u64(spec, "context")?)?;
    let kept_cells = u32::try_from(n_u64(spec, "kept_cells")?)?;
    require(
        (2..=8192).contains(&vocabulary)
            && (2..=16).contains(&blocks)
            && (1..=243).contains(&context)
            && (1..=16).contains(&kept_cells)
            && kept_cells <= vocabulary,
        "invalid model spec bounds",
    )?;
    let read_mode = ReadMode::parse(
        model
            .get("read_spec")
            .ok_or("model snapshot missing read_spec")?,
    )?;
    require(
        matches!(read_mode, ReadMode::Relation { .. }),
        "zero-trit attribution requires ReadSpec::Relation snapshot",
    )?;
    let stages = binder_stages(context)?;
    let combine_len = (blocks * stages) as usize;
    let relation_len = (RELATION_EDGE_TYPES * stages) as usize;
    let expected_gates = combine_len + relation_len;
    let expected_table = (vocabulary as usize) * (blocks as usize) * 8;
    let table_arr = array(model, "table")?;
    require(
        table_arr.len() == expected_table,
        "model table length mismatch",
    )?;
    for entry in table_arr {
        let v = entry.as_i64().ok_or("non-integer table entry")?;
        require(
            matches!(v, -2 | 0 | 2),
            "model table entry outside {-2, 0, 2}",
        )?;
    }
    // Per crates/uor-r4-discrete/src/device.rs:490-492, 632, Device::snapshot() always
    // allocates `routes` with `blocks * stages` entries (each in 0..64) across all ReadSpec modes.
    let routes_arr = array(model, "routes")?;
    require(
        routes_arr.len() == combine_len,
        "Relation snapshot routes length must equal blocks * stages",
    )?;
    for r in routes_arr {
        let act = r.as_u64().ok_or("non-integer route entry")?;
        require(act < 64, "route entry outside 0..64")?;
    }

    let before_table = model["table"].clone();
    let before_routes = model["routes"].clone();
    let before_spec = model["spec"].clone();
    let before_read_spec = model["read_spec"].clone();

    let gates_arr = model
        .get_mut("gates")
        .and_then(Value::as_array_mut)
        .ok_or("model snapshot missing gates array")?;
    require(
        gates_arr.len() == expected_gates,
        "Relation snapshot gates length mismatch",
    )?;
    let mut before_combine_gates = Vec::with_capacity(combine_len);
    let mut before_relation_trits = Vec::with_capacity(relation_len);
    let mut nonzero_zeroed = 0usize;
    for (idx, g) in gates_arr.iter_mut().enumerate() {
        let trit = g.as_i64().ok_or("non-integer gate trit")?;
        require((-1..=1).contains(&trit), "gate trit outside -1..=1")?;
        if idx < combine_len {
            before_combine_gates.push(trit);
        } else {
            before_relation_trits.push(trit);
            if trit != 0 {
                nonzero_zeroed += 1;
            }
            *g = json!(0);
        }
    }
    let after_combine_gates: Vec<i64> = model["gates"]
        .as_array()
        .ok_or("missing gates")?
        .iter()
        .take(combine_len)
        .map(|g| g.as_i64().unwrap())
        .collect();
    require(
        model["table"] == before_table
            && model["routes"] == before_routes
            && model["spec"] == before_spec
            && model["read_spec"] == before_read_spec
            && after_combine_gates == before_combine_gates,
        "E3 failed: zero-trit transform altered non-relation fields",
    )?;

    let edge_names = [
        "parent_child",
        "sibling",
        "sequential_successor",
        "same_token",
        "same_frame",
    ];
    let mut edge_table = serde_json::Map::new();
    for (e_idx, name) in edge_names.iter().enumerate() {
        let start = e_idx * (stages as usize);
        let end = start + (stages as usize);
        edge_table.insert((*name).into(), json!(&before_relation_trits[start..end]));
    }

    let new_model_bytes = serde_json::to_vec(model)?;
    let new_model_sha = hash(&new_model_bytes);
    match top_schema {
        COMPLETION_CHECKPOINT_SCHEMA => {
            outer["checkpoint"]["model_sha256"] = json!(new_model_sha);
        }
        CHECKPOINT_SCHEMA => {
            outer["model_sha256"] = json!(new_model_sha);
        }
        MODEL_SCHEMA => {}
        _ => unreachable!(),
    }

    if nonzero_zeroed == 0 {
        require(
            new_model_sha == orig_model_sha && &outer == checkpoint_doc,
            "E3 failed: zeroing already-zero relation trits altered checkpoint",
        )?;
    }

    let summary = json!({
        "schema": SCHEMA,
        "control": "E3_zero_trit_attribution_snapshot_invariance_host",
        "combine_gate_count": combine_len,
        "relation_trit_count": relation_len,
        "nonzero_relation_trits_zeroed": nonzero_zeroed,
        "relation_trits_before": before_relation_trits,
        "relation_trit_table_5x_stages": edge_table,
        "original_model_sha256": orig_model_sha,
        "zero_trit_model_sha256": new_model_sha,
        "byte_identical_when_already_zero": nonzero_zeroed == 0,
        "cuda_discrete_validate_and_eval_receipt": "OWED_ON_STAGE_M1_MERGE_AND_OWNER_RELEASE",
        "updates_applied": 0
    });
    Ok((outer, summary))
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FamilyCounter {
    pub attempted: u64,
    pub cost_differs: u64,
    pub accepted: u64,
}

impl FamilyCounter {
    pub fn parse(v: &Value) -> Result<Self> {
        let obj = v.as_object().ok_or("counter must be an object")?;
        require(obj.len() == 3, "unexpected fields in family counter")?;
        let attempted = n_u64(v, "attempted")?;
        let cost_differs = n_u64(v, "any_candidate_cost_differs_from_identity")?;
        let accepted = n_u64(v, "accepted_updates")?;
        require(
            accepted <= cost_differs && cost_differs <= attempted,
            "counter ordering violated (accepted <= cost_differs <= attempted)",
        )?;
        Ok(Self {
            attempted,
            cost_differs,
            accepted,
        })
    }

    pub fn to_json(self) -> Value {
        json!({
            "attempted": self.attempted,
            "any_candidate_cost_differs_from_identity": self.cost_differs,
            "accepted_updates": self.accepted
        })
    }
}

pub fn verify_split_family_counters(
    sensitivity: &Value,
    geom: &ScheduleGeometry,
    completed_batches: u64,
    expected_accepted_total: u64,
) -> Result<Value> {
    let obj = sensitivity
        .as_object()
        .ok_or("sensitivity must be a JSON object")?;
    let mut expected_vec = 0u64;
    let mut expected_route = 0u64;
    let mut expected_combine = 0u64;
    let mut expected_relation = 0u64;
    for s in 0..completed_batches {
        let slot = geom.slot((s % u64::from(geom.schedule_cycle)) as u32)?;
        match slot.family {
            ParameterFamily::VectorBlock => expected_vec += 1,
            ParameterFamily::Route => expected_route += 1,
            ParameterFamily::GateCombine => expected_combine += 1,
            ParameterFamily::GateRelation => expected_relation += 1,
        }
    }
    let vec_c = FamilyCounter::parse(
        sensitivity
            .get("vector_block")
            .ok_or("missing vector_block counter")?,
    )?;
    require(
        vec_c.attempted == expected_vec,
        "vector_block attempted count mismatch against schedule",
    )?;
    let (route_c, combine_c, relation_c) = match geom.read_mode {
        ReadMode::Route => {
            require(
                obj.len() == 2 && obj.contains_key("route"),
                "Route sensitivity must contain only vector_block and route",
            )?;
            let r = FamilyCounter::parse(&sensitivity["route"])?;
            require(
                r.attempted == expected_route,
                "route attempted count mismatch against schedule",
            )?;
            (r, FamilyCounter::default(), FamilyCounter::default())
        }
        ReadMode::Flock { .. } => {
            require(
                obj.len() == 2 && obj.contains_key("gate_combine"),
                "Flock sensitivity must contain vector_block and gate_combine",
            )?;
            let c = FamilyCounter::parse(&sensitivity["gate_combine"])?;
            require(
                c.attempted == expected_combine,
                "gate_combine attempted count mismatch against schedule",
            )?;
            (FamilyCounter::default(), c, FamilyCounter::default())
        }
        ReadMode::Relation { .. } => {
            require(
                obj.len() == 3
                    && obj.contains_key("gate_combine")
                    && obj.contains_key("gate_relation"),
                "Relation sensitivity must contain vector_block, gate_combine, and gate_relation",
            )?;
            let c = FamilyCounter::parse(&sensitivity["gate_combine"])?;
            let r = FamilyCounter::parse(&sensitivity["gate_relation"])?;
            require(
                c.attempted == expected_combine && r.attempted == expected_relation,
                "Relation gate_combine/gate_relation attempted count mismatch against schedule",
            )?;
            (FamilyCounter::default(), c, r)
        }
    };
    let total_attempted = vec_c
        .attempted
        .checked_add(route_c.attempted)
        .and_then(|x| x.checked_add(combine_c.attempted))
        .and_then(|x| x.checked_add(relation_c.attempted))
        .ok_or("attempted sum overflow")?;
    let total_cost_differs = vec_c
        .cost_differs
        .checked_add(route_c.cost_differs)
        .and_then(|x| x.checked_add(combine_c.cost_differs))
        .and_then(|x| x.checked_add(relation_c.cost_differs))
        .ok_or("cost_differs sum overflow")?;
    let total_accepted = vec_c
        .accepted
        .checked_add(route_c.accepted)
        .and_then(|x| x.checked_add(combine_c.accepted))
        .and_then(|x| x.checked_add(relation_c.accepted))
        .ok_or("accepted sum overflow")?;
    require(
        total_attempted == completed_batches && total_accepted == expected_accepted_total,
        "split family counter totals disagree with completed_batches or accepted_updates",
    )?;
    Ok(json!({
        "vector_block": vec_c.to_json(),
        "route": route_c.to_json(),
        "gate_combine": combine_c.to_json(),
        "gate_relation": relation_c.to_json(),
        "totals": {
            "attempted": total_attempted,
            "any_candidate_cost_differs_from_identity": total_cost_differs,
            "accepted_updates": total_accepted
        }
    }))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiscriminatorSpreads {
    pub succ_top8_spread: i64,
    pub succ_top1_spread: i64,
    pub succ_b3_spread: i64,
    pub full_top1_spread: i64,
    pub full_top8_spread: i64,
    pub full_b3_spread: i64,
    pub two_seed_measured: bool,
}

impl Default for DiscriminatorSpreads {
    fn default() -> Self {
        Self {
            succ_top8_spread: 39,
            succ_top1_spread: 0,
            succ_b3_spread: 0,
            full_top1_spread: 20,
            full_top8_spread: 63,
            full_b3_spread: 87,
            two_seed_measured: false,
        }
    }
}

impl DiscriminatorSpreads {
    pub fn from_seed_pair(
        seed1_route: &SubpanelEvaluationAudit,
        seed2_route: &SubpanelEvaluationAudit,
    ) -> Result<Self> {
        Ok(Self {
            succ_top8_spread: (count_val(&seed1_route.copy_successor, "top8_correct")?
                - count_val(&seed2_route.copy_successor, "top8_correct")?)
            .abs()
            .max(39),
            succ_top1_spread: (count_val(&seed1_route.copy_successor, "top1_correct")?
                - count_val(&seed2_route.copy_successor, "top1_correct")?)
            .abs(),
            succ_b3_spread: (count_val(&seed1_route.copy_successor, "target_cell_admitted_rows")?
                - count_val(&seed2_route.copy_successor, "target_cell_admitted_rows")?)
            .abs(),
            full_top1_spread: (count_val(&seed1_route.full_panel, "top1_correct")?
                - count_val(&seed2_route.full_panel, "top1_correct")?)
            .abs()
            .max(20),
            full_top8_spread: (count_val(&seed1_route.full_panel, "top8_correct")?
                - count_val(&seed2_route.full_panel, "top8_correct")?)
            .abs()
            .max(63),
            full_b3_spread: (count_val(&seed1_route.full_panel, "target_cell_admitted_rows")?
                - count_val(&seed2_route.full_panel, "target_cell_admitted_rows")?)
            .abs()
            .max(87),
            two_seed_measured: true,
        })
    }
}

pub fn evaluate_m2_discriminator_bars(
    route_interleaved: &SubpanelEvaluationAudit,
    relation_interleaved: &SubpanelEvaluationAudit,
    relation_zero_trits: &SubpanelEvaluationAudit,
    gate_relation_accepted_updates: u64,
    spreads: DiscriminatorSpreads,
) -> Result<Value> {
    let delta_seed_succ_top8 = spreads.succ_top8_spread.max(39);
    let delta_seed_succ_top1 = spreads.succ_top1_spread.max(0);
    let delta_seed_succ_b3 = spreads.succ_b3_spread.max(0);
    let delta_seed_full_top1 = spreads.full_top1_spread.max(20);
    let delta_seed_full_top8 = spreads.full_top8_spread.max(63);
    let delta_seed_full_b3 = spreads.full_b3_spread.max(87);

    let route_succ_top8 = count_val(&route_interleaved.copy_successor, "top8_correct")?;
    let rel_succ_top8 = count_val(&relation_interleaved.copy_successor, "top8_correct")?;
    let zero_succ_top8 = count_val(&relation_zero_trits.copy_successor, "top8_correct")?;
    let succ_top8_gain = rel_succ_top8 - route_succ_top8;

    let route_succ_top1 = count_val(&route_interleaved.copy_successor, "top1_correct")?;
    let rel_succ_top1 = count_val(&relation_interleaved.copy_successor, "top1_correct")?;
    let succ_top1_delta = rel_succ_top1 - route_succ_top1;
    let route_succ_b3 = count_val(
        &route_interleaved.copy_successor,
        "target_cell_admitted_rows",
    )?;
    let rel_succ_b3 = count_val(
        &relation_interleaved.copy_successor,
        "target_cell_admitted_rows",
    )?;
    let succ_b3_delta = rel_succ_b3 - route_succ_b3;

    let b1_top8_gain_pass = succ_top8_gain > delta_seed_succ_top8;
    let b1_top1_nondec = succ_top1_delta >= -delta_seed_succ_top1;
    let b1_b3_nondec = succ_b3_delta >= -delta_seed_succ_b3;
    let b1_pass = b1_top8_gain_pass && b1_top1_nondec && b1_b3_nondec;

    let full_top1_delta = count_val(&relation_interleaved.full_panel, "top1_correct")?
        - count_val(&route_interleaved.full_panel, "top1_correct")?;
    let full_top8_delta = count_val(&relation_interleaved.full_panel, "top8_correct")?
        - count_val(&route_interleaved.full_panel, "top8_correct")?;
    let full_b3_delta = count_val(
        &relation_interleaved.full_panel,
        "target_cell_admitted_rows",
    )? - count_val(&route_interleaved.full_panel, "target_cell_admitted_rows")?;
    let b2_pass = full_top1_delta >= -delta_seed_full_top1
        && full_top8_delta >= -delta_seed_full_top8
        && full_b3_delta >= -delta_seed_full_b3;

    let zero_trit_succ_top8_delta = rel_succ_top8 - zero_succ_top8;
    let b3_pass = zero_trit_succ_top8_delta > 0 && gate_relation_accepted_updates >= 1;
    let keep = b1_pass && b2_pass && b3_pass;
    let b1_spread_mode = if spreads.two_seed_measured {
        "two_seed_measured_with_39_floor_top8_and_0_floor_side_guards"
    } else {
        "floor_only_not_sheet_b1"
    };
    let b2_spread_mode = if spreads.two_seed_measured {
        "two_seed_measured_with_20_63_87_floors"
    } else {
        "floor_only_not_sheet_b2"
    };
    let overall_outcome = if keep {
        if spreads.two_seed_measured {
            "KEEP_CANDIDATE_REQUIRES_TWO_SEEDS_AND_FRESH_TRANSFER"
        } else {
            "SINGLE_SEED_FLOOR_ONLY_NOT_SHEET_B1_REQUIRES_TWO_SEEDS_AND_FRESH_TRANSFER"
        }
    } else {
        "NEGATIVE_CLOSES_THIS_CONFIGURATION_ONLY_D22"
    };

    Ok(json!({
        "b1_copy_successor_subpanel": {
            "spread_mode": b1_spread_mode,
            "route_interleaved_top8": route_succ_top8,
            "relation_interleaved_top8": rel_succ_top8,
            "top8_delta": succ_top8_gain,
            "required_strict_floor_delta_seed": delta_seed_succ_top8,
            "top8_exceeds_floor": b1_top8_gain_pass,
            "top1_delta": succ_top1_delta,
            "top1_floor": -delta_seed_succ_top1,
            "top1_non_decreasing": b1_top1_nondec,
            "target_cell_admitted_delta": succ_b3_delta,
            "target_cell_admitted_floor": -delta_seed_succ_b3,
            "target_cell_admitted_non_decreasing": b1_b3_nondec,
            "pass": b1_pass
        },
        "b2_full_panel_non_regression": {
            "spread_mode": b2_spread_mode,
            "top1_delta": full_top1_delta,
            "top1_floor": -delta_seed_full_top1,
            "top8_delta": full_top8_delta,
            "top8_floor": -delta_seed_full_top8,
            "target_cell_admitted_delta": full_b3_delta,
            "target_cell_admitted_floor": -delta_seed_full_b3,
            "pass": b2_pass
        },
        "b3_zero_trit_attribution": {
            "relation_interleaved_succ_top8": rel_succ_top8,
            "relation_zero_trits_succ_top8": zero_succ_top8,
            "zero_trit_succ_top8_delta": zero_trit_succ_top8_delta,
            "gate_relation_accepted_updates": gate_relation_accepted_updates,
            "pass": b3_pass
        },
        "pass": keep,
        "overall_outcome": overall_outcome,
        "authorizes_model_compute": false
    }))
}

fn validate_decision_cursor(doc: &Value, eval: &Value, label: &str) -> Result<u64> {
    let epoch = n_u64(eval, "model_epoch")?;
    let acc = n_u64(eval, "accepted_updates_in_model")?;
    require(
        epoch == acc,
        &format!("{label} evaluation receipt model_epoch != accepted_updates_in_model"),
    )?;
    let top_batches = if doc.get("completed_batches").is_some() {
        Some(n_u64(doc, "completed_batches")?)
    } else if eval.get("completed_batches").is_some() {
        Some(n_u64(eval, "completed_batches")?)
    } else {
        None
    };
    let cursor_obj = doc.get("cursor").or_else(|| eval.get("cursor"));
    let cursor_batches = if let Some(c) = cursor_obj {
        let cb = n_u64(c, "completed_batches")?;
        let ca = n_u64(c, "accepted_updates")?;
        let cp = n_u64(c, "corpus_pass")?;
        let nc = n_u64(c, "next_coordinate")?;
        require(
            ca == epoch && cp == 0 && nc == 0,
            &format!(
                "{label} receipt cursor accepted_updates/corpus_pass/next_coordinate mismatch at decision point"
            ),
        )?;
        Some(cb)
    } else {
        None
    };
    let batches = match (top_batches, cursor_batches) {
        (Some(tb), Some(cb)) => {
            require(
                tb == cb,
                &format!(
                    "{label} receipt completed_batches disagrees with cursor.completed_batches"
                ),
            )?;
            tb
        }
        (Some(tb), None) => tb,
        (None, Some(cb)) => cb,
        (None, None) => {
            return Err(format!(
                "{label} receipt missing completed_batches / cursor at decision point"
            )
            .into())
        }
    };
    require(
        batches == DECISION_CURSOR_BATCHES && epoch <= DECISION_CURSOR_BATCHES,
        &format!("{label} receipt completed_batches must equal registered decision cursor 16404"),
    )?;
    Ok(epoch)
}

fn validate_discriminator_seed_trio(
    partition: &SubpanelPartition,
    route_doc: &Value,
    rel_doc: &Value,
    zero_doc: &Value,
    batch_rows: usize,
) -> Result<(u64, u64)> {
    let route_eval = unwrap_evaluation(route_doc)?;
    let rel_eval = unwrap_evaluation(rel_doc)?;
    let zero_eval = unwrap_evaluation(zero_doc)?;

    validate_evaluation_population_binding(partition, route_eval, batch_rows)?;
    validate_evaluation_population_binding(partition, rel_eval, batch_rows)?;
    validate_evaluation_population_binding(partition, zero_eval, batch_rows)?;

    let rb = route_eval
        .get("model_bindings")
        .ok_or("Route evaluation receipt missing model_bindings")?;
    let relb = rel_eval
        .get("model_bindings")
        .ok_or("Relation evaluation receipt missing model_bindings")?;
    let zb = zero_eval
        .get("model_bindings")
        .ok_or("Zero-trit evaluation receipt missing model_bindings")?;
    let r_mode = ReadMode::parse(rb.get("read_spec").ok_or("missing Route read_spec")?)?;
    let rel_mode = ReadMode::parse(relb.get("read_spec").ok_or("missing Relation read_spec")?)?;
    let z_mode = ReadMode::parse(zb.get("read_spec").ok_or("missing zero-trit read_spec")?)?;
    require(
        r_mode == ReadMode::Route,
        "Route evaluation receipt model_bindings.read_spec must be Route",
    )?;
    require(
        matches!(rel_mode, ReadMode::Relation { .. }) && rel_mode == z_mode,
        "Relation and zero-trit receipts must share identical ReadSpec::Relation",
    )?;
    require(
        rb.get("spec").is_some()
            && rb.get("spec") == relb.get("spec")
            && relb.get("spec") == zb.get("spec"),
        "Route, Relation, and zero-trit receipts must share identical model_bindings.spec",
    )?;
    require(
        relb == zb,
        "Relation and zero-trit evaluation receipts must share identical model_bindings",
    )?;
    let r_seed = n_u64(rb, "seed")?;
    let rel_seed = n_u64(relb, "seed")?;
    let z_seed = n_u64(zb, "seed")?;
    require(
        r_seed == rel_seed && rel_seed == z_seed,
        "Route, Relation, and zero-trit receipts in a seed trio must share identical seed",
    )?;

    let route_lin_sha = sha64_field(route_eval, "evaluation_lineage_sha256")?;
    let rel_lin_sha = sha64_field(rel_eval, "evaluation_lineage_sha256")?;
    let zero_lin_sha = sha64_field(zero_eval, "evaluation_lineage_sha256")?;
    require(
        rel_lin_sha == zero_lin_sha
            && rel_eval.get("evaluation_lineage") == zero_eval.get("evaluation_lineage"),
        "Relation and zero-trit receipts must share identical evaluation_lineage",
    )?;
    require(
        route_lin_sha != rel_lin_sha,
        "Route and Relation evaluation_lineage_sha256 must differ because config.read_spec differs",
    )?;
    let mut route_lin_matched = route_eval["evaluation_lineage"].clone();
    let rel_lin = &rel_eval["evaluation_lineage"];
    let route_sched = ScheduleKind::parse(
        route_lin_matched["config"]
            .get("schedule")
            .ok_or("missing Route schedule")?,
    )?;
    let rel_sched = ScheduleKind::parse(
        rel_lin["config"]
            .get("schedule")
            .ok_or("missing Relation schedule")?,
    )?;
    require(
        route_sched == ScheduleKind::InterleavedReadEvery8V1
            && rel_sched == ScheduleKind::InterleavedReadEvery8V1,
        "Stage M2 discriminator requires interleaved_read_every_8_v1 schedule on all arms",
    )?;
    route_lin_matched["config"]["read_spec"] = rel_lin["config"]["read_spec"].clone();
    require(
        &route_lin_matched == rel_lin,
        "Route and Relation evaluation_lineage must match on all fields except config.read_spec",
    )?;

    let route_epoch = validate_decision_cursor(route_doc, route_eval, "Route")?;
    let rel_epoch = validate_decision_cursor(rel_doc, rel_eval, "Relation")?;
    let zero_epoch = validate_decision_cursor(zero_doc, zero_eval, "zero-trit")?;
    require(
        rel_epoch == zero_epoch,
        "Relation and zero-trit receipts must have matching model_epoch and accepted_updates_in_model (0 extra updates)",
    )?;

    let _route_snap_sha = sha64_field(route_eval, "model_snapshot_sha256")?;
    let rel_snap_sha = sha64_field(rel_eval, "model_snapshot_sha256")?;
    let _zero_snap_sha = sha64_field(zero_eval, "model_snapshot_sha256")?;
    let zero_source_model_sha = if zero_doc.get("source_relation_model_sha256").is_some() {
        sha64_field(zero_doc, "source_relation_model_sha256")?
    } else {
        sha64_field(zero_eval, "source_relation_model_sha256")?
    };
    require(
        zero_source_model_sha == rel_snap_sha,
        "zero-trit receipt source_relation_model_sha256 does not match Relation model_snapshot_sha256",
    )?;
    let rel_ckpt_sha = if rel_doc.get("driver_checkpoint_sha256").is_some() {
        sha64_field(rel_doc, "driver_checkpoint_sha256")?
    } else {
        sha64_field(rel_eval, "driver_checkpoint_sha256")?
    };
    let zero_ckpt_sha = if zero_doc.get("source_relation_checkpoint_sha256").is_some() {
        sha64_field(zero_doc, "source_relation_checkpoint_sha256")?
    } else {
        sha64_field(zero_eval, "source_relation_checkpoint_sha256")?
    };
    require(
        zero_ckpt_sha == rel_ckpt_sha,
        "zero-trit receipt source_relation_checkpoint_sha256 does not match Relation driver_checkpoint_sha256",
    )?;

    let spec = rb.get("spec").ok_or("missing model_bindings.spec")?;
    let vocabulary = u32::try_from(n_u64(spec, "vocabulary")?)?;
    let blocks = u32::try_from(n_u64(spec, "blocks")?)?;
    let context = u32::try_from(n_u64(spec, "context")?)?;
    let route_geom = ScheduleGeometry::new(
        vocabulary,
        blocks,
        context,
        r_mode,
        ScheduleKind::InterleavedReadEvery8V1,
    )?;
    let rel_geom = ScheduleGeometry::new(
        vocabulary,
        blocks,
        context,
        rel_mode,
        ScheduleKind::InterleavedReadEvery8V1,
    )?;
    let route_sens = route_doc
        .get("sensitivity")
        .or_else(|| route_eval.get("sensitivity"))
        .ok_or("Route evaluation receipt missing sensitivity")?;
    verify_split_family_counters(
        route_sens,
        &route_geom,
        DECISION_CURSOR_BATCHES,
        route_epoch,
    )?;
    let rel_sens = rel_doc
        .get("sensitivity")
        .or_else(|| rel_eval.get("sensitivity"))
        .ok_or("Relation evaluation receipt missing sensitivity")?;
    let verified_rel_sens =
        verify_split_family_counters(rel_sens, &rel_geom, DECISION_CURSOR_BATCHES, rel_epoch)?;
    if let Some(zero_sens) = zero_doc
        .get("sensitivity")
        .or_else(|| zero_eval.get("sensitivity"))
    {
        require(
            zero_sens == rel_sens,
            "zero-trit receipt sensitivity disagrees with source Relation sensitivity",
        )?;
    }
    let gate_rel_accepted = n_u64(&verified_rel_sens["gate_relation"], "accepted_updates")?;

    Ok((r_seed, gate_rel_accepted))
}

fn verify_dev_partition_if_full(
    partition: &SubpanelPartition,
    tokens_sha: &str,
    mask_sha: &str,
) -> Result<()> {
    if partition.eligible_rows == DEV_ELIGIBLE_ROWS {
        require(
            tokens_sha == DEV_TOKENS_SHA256 && mask_sha == DEV_MASK_SHA256,
            "full DEV corpus SHA256 mismatch against pinned ft/dev digests",
        )?;
        require(
            partition.payload_tokens == DEV_PAYLOAD_TOKENS
                && partition.documents == DEV_DOCUMENTS
                && partition.query_recur_rows == DEV_QUERY_RECUR_ROWS
                && partition.copy_successor_rows == DEV_COPY_SUCCESSOR_ROWS
                && partition.distance_bucket_rows == DEV_DISTANCE_BUCKETS
                && partition.distance_one_rows == 0
                && partition.any_occurrence_successor_rows == DEV_ANY_SUCCESSOR_ROWS
                && partition.copy_ceiling_rows == DEV_COPY_CEILING_ROWS
                && partition.complement_rows == DEV_COMPLEMENT_ROWS,
            "E1 failed: ft/dev copy-successor sub-panel counts do not match pinned 3,847 / 14,287 partition",
        )?;
    }
    Ok(())
}

fn run_subpanel_cli(
    tokens_path: &Path,
    mask_path: &Path,
    eval_path: &Path,
    output_path: &Path,
) -> Result<()> {
    let (token_bytes, tokens_sha) = read_bounded_bytes(tokens_path)?;
    let (mask_bytes, mask_sha) = read_bounded_bytes(mask_path)?;
    let (eval_doc, eval_sha) = read_json(eval_path)?;
    let partition = partition_corpus(&token_bytes, &mask_bytes, 192, 0)?;
    verify_dev_partition_if_full(&partition, &tokens_sha, &mask_sha)?;
    let require_explicit_row_binding =
        !SEALED_2471_ROUTE_CANONICAL_EVAL_SHAS.contains(&eval_sha.as_str());
    let subpanels = audit_evaluation_subpanels_with_mode(
        &partition,
        &eval_doc,
        64,
        require_explicit_row_binding,
    )?;
    let report = json!({
        "schema": SCHEMA,
        "mode": "subpanel",
        "inputs": {
            "dev_tokens_sha256": tokens_sha,
            "dev_mask_sha256": mask_sha,
            "evaluation_receipt_sha256": eval_sha,
            "ordered_row_binding_sha256": partition.ordered_row_binding_sha256
        },
        "partition": {
            "vocabulary": partition.vocabulary,
            "payload_tokens": partition.payload_tokens,
            "documents": partition.documents,
            "context": partition.context,
            "eligible_rows": partition.eligible_rows,
            "query_recur_rows": partition.query_recur_rows,
            "copy_successor_rows": partition.copy_successor_rows,
            "distance_bucket_rows_1_8_9_32_33_96_97_192": partition.distance_bucket_rows,
            "distance_one_rows": partition.distance_one_rows,
            "any_occurrence_successor_rows": partition.any_occurrence_successor_rows,
            "copy_ceiling_rows": partition.copy_ceiling_rows,
            "complement_rows": partition.complement_rows
        },
        "subpanels": subpanels.to_report_json(),
        "schedule_controls_s1_s4": "HOST_REFERENCE_ONLY_DRIVER_S1_S4_OWED_FOR_G3",
        "authorization": "none",
        "authorizes_model_compute": false,
        "stage_m1_construction": "GATED_ON_PEER_REVIEW_AND_OWNER_RELEASE",
        "stage_m2_discriminator": "HELD",
        "seed2": "HELD",
        "m2": "HELD"
    });
    write_exclusive_json(output_path, &report)?;
    println!(
        "Relation sub-panel receipt audit PASS; eligible_rows={}; copy_successor_rows={}; output {}",
        partition.eligible_rows,
        partition.copy_successor_rows,
        output_path.display()
    );
    Ok(())
}

fn run_zero_trits_cli(input_checkpoint: &Path, output_checkpoint: &Path) -> Result<()> {
    let (ckpt, _) = read_json(input_checkpoint)?;
    let (zeroed, summary) = prepare_zero_trit_checkpoint(&ckpt)?;
    write_exclusive_json(output_checkpoint, &zeroed)?;
    println!(
        "Relation zero-trit attribution checkpoint prepared PASS; zeroed={}; output {}",
        summary["nonzero_relation_trits_zeroed"],
        output_checkpoint.display()
    );
    Ok(())
}

fn run_discriminator_cli(
    tokens_path: &Path,
    mask_path: &Path,
    route_eval_path: &Path,
    rel_eval_path: &Path,
    zero_eval_path: &Path,
    output_path: &Path,
) -> Result<()> {
    let (token_bytes, tokens_sha) = read_bounded_bytes(tokens_path)?;
    let (mask_bytes, mask_sha) = read_bounded_bytes(mask_path)?;
    let (route_doc, route_sha) = read_json(route_eval_path)?;
    let (rel_doc, rel_sha) = read_json(rel_eval_path)?;
    let (zero_doc, zero_sha) = read_json(zero_eval_path)?;
    let partition = partition_corpus(&token_bytes, &mask_bytes, 192, 0)?;
    verify_dev_partition_if_full(&partition, &tokens_sha, &mask_sha)?;
    let (seed, gate_rel_accepted) =
        validate_discriminator_seed_trio(&partition, &route_doc, &rel_doc, &zero_doc, 64)?;
    let route_audit = audit_evaluation_subpanels(&partition, &route_doc, 64)?;
    let rel_audit = audit_evaluation_subpanels(&partition, &rel_doc, 64)?;
    let zero_audit = audit_evaluation_subpanels(&partition, &zero_doc, 64)?;
    let rel_vs_route = compare_evaluation_subpanels(&partition, &route_doc, &rel_doc, 64)?;
    let zero_trit_cmp = compare_evaluation_subpanels(&partition, &zero_doc, &rel_doc, 64)?;
    let bars = evaluate_m2_discriminator_bars(
        &route_audit,
        &rel_audit,
        &zero_audit,
        gate_rel_accepted,
        DiscriminatorSpreads::default(),
    )?;
    let report = json!({
        "schema": SCHEMA,
        "mode": "discriminator_single_seed",
        "seed": seed,
        "inputs": {
            "dev_tokens_sha256": tokens_sha,
            "dev_mask_sha256": mask_sha,
            "ordered_row_binding_sha256": partition.ordered_row_binding_sha256,
            "route_interleaved_eval_sha256": route_sha,
            "relation_interleaved_eval_sha256": rel_sha,
            "relation_zero_trits_eval_sha256": zero_sha
        },
        "subpanels": {
            "route_interleaved": route_audit.to_report_json(),
            "relation_interleaved": rel_audit.to_report_json(),
            "relation_zero_trits": zero_audit.to_report_json()
        },
        "comparisons": {
            "relation_interleaved_minus_route_interleaved": rel_vs_route,
            "relation_interleaved_minus_relation_zero_trits": zero_trit_cmp
        },
        "bars": bars,
        "schedule_controls_s1_s4": "HOST_REFERENCE_ONLY_DRIVER_S1_S4_OWED_FOR_G3",
        "authorization": "none",
        "authorizes_model_compute": false,
        "stage_m1_construction": "GATED_ON_PEER_REVIEW_AND_OWNER_RELEASE",
        "stage_m2_discriminator": "HELD",
        "seed2": "HELD",
        "m2": "HELD"
    });
    write_exclusive_json(output_path, &report)?;
    println!(
        "Relation Stage M2 discriminator receipt audit PASS; outcome={}; output {}",
        report["bars"]["overall_outcome"],
        output_path.display()
    );
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn run_discriminator_two_seed_cli(
    tokens_path: &Path,
    mask_path: &Path,
    route_s1_path: &Path,
    rel_s1_path: &Path,
    zero_s1_path: &Path,
    route_s2_path: &Path,
    rel_s2_path: &Path,
    zero_s2_path: &Path,
    output_path: &Path,
) -> Result<()> {
    let (token_bytes, tokens_sha) = read_bounded_bytes(tokens_path)?;
    let (mask_bytes, mask_sha) = read_bounded_bytes(mask_path)?;
    let (route_s1, route_s1_sha) = read_json(route_s1_path)?;
    let (rel_s1, rel_s1_sha) = read_json(rel_s1_path)?;
    let (zero_s1, zero_s1_sha) = read_json(zero_s1_path)?;
    let (route_s2, route_s2_sha) = read_json(route_s2_path)?;
    let (rel_s2, rel_s2_sha) = read_json(rel_s2_path)?;
    let (zero_s2, zero_s2_sha) = read_json(zero_s2_path)?;
    let partition = partition_corpus(&token_bytes, &mask_bytes, 192, 0)?;
    verify_dev_partition_if_full(&partition, &tokens_sha, &mask_sha)?;

    let (seed1, gate_rel_s1) =
        validate_discriminator_seed_trio(&partition, &route_s1, &rel_s1, &zero_s1, 64)?;
    let (seed2, gate_rel_s2) =
        validate_discriminator_seed_trio(&partition, &route_s2, &rel_s2, &zero_s2, 64)?;
    require(
        seed1 != seed2,
        "paired two-seed discriminator requires distinct seed IDs",
    )?;
    let mut s1_lin_matched = unwrap_evaluation(&route_s1)?["evaluation_lineage"].clone();
    let s2_lin = &unwrap_evaluation(&route_s2)?["evaluation_lineage"];
    s1_lin_matched["config"]["seed"] = s2_lin["config"]["seed"].clone();
    require(
        &s1_lin_matched == s2_lin,
        "seed1 and seed2 evaluation_lineage must match on all fields except config.seed",
    )?;

    let route_s1_audit = audit_evaluation_subpanels(&partition, &route_s1, 64)?;
    let rel_s1_audit = audit_evaluation_subpanels(&partition, &rel_s1, 64)?;
    let zero_s1_audit = audit_evaluation_subpanels(&partition, &zero_s1, 64)?;
    let route_s2_audit = audit_evaluation_subpanels(&partition, &route_s2, 64)?;
    let rel_s2_audit = audit_evaluation_subpanels(&partition, &rel_s2, 64)?;
    let zero_s2_audit = audit_evaluation_subpanels(&partition, &zero_s2, 64)?;

    let spreads = DiscriminatorSpreads::from_seed_pair(&route_s1_audit, &route_s2_audit)?;

    let bars_s1 = evaluate_m2_discriminator_bars(
        &route_s1_audit,
        &rel_s1_audit,
        &zero_s1_audit,
        gate_rel_s1,
        spreads,
    )?;
    let bars_s2 = evaluate_m2_discriminator_bars(
        &route_s2_audit,
        &rel_s2_audit,
        &zero_s2_audit,
        gate_rel_s2,
        spreads,
    )?;
    let both_pass =
        bars_s1["pass"].as_bool() == Some(true) && bars_s2["pass"].as_bool() == Some(true);

    let report = json!({
        "schema": SCHEMA,
        "mode": "discriminator_two_seed",
        "seeds": [seed1, seed2],
        "inputs": {
            "dev_tokens_sha256": tokens_sha,
            "dev_mask_sha256": mask_sha,
            "ordered_row_binding_sha256": partition.ordered_row_binding_sha256,
            "seed1": {
                "route_interleaved_eval_sha256": route_s1_sha,
                "relation_interleaved_eval_sha256": rel_s1_sha,
                "relation_zero_trits_eval_sha256": zero_s1_sha
            },
            "seed2": {
                "route_interleaved_eval_sha256": route_s2_sha,
                "relation_interleaved_eval_sha256": rel_s2_sha,
                "relation_zero_trits_eval_sha256": zero_s2_sha
            }
        },
        "measured_two_seed_spreads": {
            "succ_top8_spread_floored_39": spreads.succ_top8_spread,
            "succ_top1_spread": spreads.succ_top1_spread,
            "succ_b3_spread": spreads.succ_b3_spread,
            "full_top1_spread_floored_20": spreads.full_top1_spread,
            "full_top8_spread_floored_63": spreads.full_top8_spread,
            "full_b3_spread_floored_87": spreads.full_b3_spread
        },
        "seed1_bars": bars_s1,
        "seed2_bars": bars_s2,
        "overall_outcome": if both_pass {
            "TWO_SEED_KEEP_REQUIRES_FRESH_TRANSFER"
        } else {
            "NEGATIVE_CLOSES_THIS_CONFIGURATION_ONLY_D22"
        },
        "schedule_controls_s1_s4": "HOST_REFERENCE_ONLY_DRIVER_S1_S4_OWED_FOR_G3",
        "authorization": "none",
        "authorizes_model_compute": false,
        "stage_m1_construction": "GATED_ON_PEER_REVIEW_AND_OWNER_RELEASE",
        "stage_m2_discriminator": "HELD",
        "seed2": "HELD",
        "m2": "HELD"
    });
    write_exclusive_json(output_path, &report)?;
    println!(
        "Relation Stage M2 two-seed discriminator receipt audit PASS; outcome={}; output {}",
        report["overall_outcome"],
        output_path.display()
    );
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().collect();
    require(
        args.len() >= 2,
        "usage: relation-subpanel-audit <subpanel|prepare-zero-trits|discriminator> ...",
    )?;
    match args[1].to_str().ok_or("invalid subcommand")? {
        "subpanel" => {
            require(
                args.len() == 6,
                "usage: relation-subpanel-audit subpanel DEV_TOKENS_U16 DEV_MASK_U8 EVALUATION_JSON NEW_OUTPUT_JSON",
            )?;
            run_subpanel_cli(
                Path::new(&args[2]),
                Path::new(&args[3]),
                Path::new(&args[4]),
                Path::new(&args[5]),
            )
        }
        "prepare-zero-trits" => {
            require(
                args.len() == 4,
                "usage: relation-subpanel-audit prepare-zero-trits RELATION_CHECKPOINT_JSON NEW_ZERO_TRIT_CHECKPOINT_JSON",
            )?;
            run_zero_trits_cli(Path::new(&args[2]), Path::new(&args[3]))
        }
        "discriminator" => match args.len() {
            8 => run_discriminator_cli(
                Path::new(&args[2]),
                Path::new(&args[3]),
                Path::new(&args[4]),
                Path::new(&args[5]),
                Path::new(&args[6]),
                Path::new(&args[7]),
            ),
            11 => run_discriminator_two_seed_cli(
                Path::new(&args[2]),
                Path::new(&args[3]),
                Path::new(&args[4]),
                Path::new(&args[5]),
                Path::new(&args[6]),
                Path::new(&args[7]),
                Path::new(&args[8]),
                Path::new(&args[9]),
                Path::new(&args[10]),
            ),
            _ => Err(
                "usage: relation-subpanel-audit discriminator DEV_TOKENS_U16 DEV_MASK_U8 ROUTE_SEED1_EVAL_JSON RELATION_SEED1_EVAL_JSON RELATION_ZERO_TRITS_SEED1_EVAL_JSON [ROUTE_SEED2_EVAL_JSON RELATION_SEED2_EVAL_JSON RELATION_ZERO_TRITS_SEED2_EVAL_JSON] NEW_OUTPUT_JSON".into(),
            ),
        },
        _ => Err(
            "unknown subcommand; expected subpanel, prepare-zero-trits, or discriminator".into(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_uort(vocab: u32, tokens: &[u16]) -> Vec<u8> {
        let mut bytes = vec![0u8; 64];
        bytes[..4].copy_from_slice(b"UORT");
        bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
        bytes[8..16].copy_from_slice(&(tokens.len() as u64).to_le_bytes());
        bytes[16..20].copy_from_slice(&vocab.to_le_bytes());
        for &t in tokens {
            bytes.extend_from_slice(&t.to_le_bytes());
        }
        bytes
    }

    fn build_eval_doc(
        part: &SubpanelPartition,
        row_specs: &[(i64, i64, i64, i64, i64)],
        batch_rows: usize,
    ) -> Value {
        assert_eq!(row_specs.len(), part.eligible_rows);
        let expected_top_len = (part.vocabulary as usize).min(8);
        let mut batches = Vec::new();
        let mut total = zero_slice();
        let mut row_ordinal = 0usize;
        for (b_idx, chunk) in row_specs.chunks(batch_rows).enumerate() {
            let mut rows = Vec::new();
            let mut target_positions = Vec::new();
            let mut b_sums = zero_slice();
            for &(t, a, tr, ar, cells) in chunk {
                let class = &part.rows[row_ordinal];
                let target_adm = cells - ar;
                let h1 = i64::from(target_adm == 1 && t < 32);
                let h8 = target_adm;
                let loss = 8 * t + 32 * a;
                let target_u64 = u64::from(class.target_token);
                let nonself: Vec<u64> = (0..u64::from(part.vocabulary))
                    .filter(|&id| id != target_u64)
                    .take(expected_top_len)
                    .collect();
                let mut top8 = Vec::with_capacity(expected_top_len);
                if h1 == 1 {
                    top8.push(target_u64);
                    top8.extend_from_slice(&nonself[..expected_top_len - 1]);
                } else if h8 == 1 {
                    top8.push(nonself[0]);
                    top8.push(target_u64);
                    top8.extend_from_slice(&nonself[1..expected_top_len - 1]);
                } else {
                    top8.extend_from_slice(&nonself[..expected_top_len]);
                }
                let r = json!({
                    "row_ordinal": row_ordinal,
                    "target_position": class.position,
                    "query_token": class.query_token,
                    "target_token": class.target_token,
                    "token_hinge": t,
                    "address_hinge": a,
                    "total_loss_numerator": loss,
                    "token_rivals": tr,
                    "address_rivals": ar,
                    "admitted_cells": cells,
                    "top1_correct": h1,
                    "top8_correct": h8,
                    "top8": top8
                });
                let rm = parse_row_metrics(&r, part.vocabulary).unwrap();
                verify_row_correspondence(&r, class, row_ordinal, true).unwrap();
                for (k, v) in &rm {
                    add_count(&mut b_sums, k, *v).unwrap();
                    add_count(&mut total, k, *v).unwrap();
                }
                target_positions.push(class.position);
                rows.push(r);
                row_ordinal += 1;
            }
            let mut b_agg = serde_json::to_value(&b_sums).unwrap();
            b_agg["loss_denominator_per_row"] = json!(256);
            batches.push(json!({
                "batch_index": b_idx,
                "target_positions": target_positions,
                "rows": rows,
                "aggregate": b_agg
            }));
        }
        let mut total_agg = serde_json::to_value(&total).unwrap();
        total_agg["loss_denominator_per_row"] = json!(256);
        let spec_json = json!({
            "vocabulary": part.vocabulary,
            "blocks": 4,
            "context": part.context,
            "kept_cells": 8.min(part.vocabulary)
        });
        let dev_source = json!({
            "tokens": {
                "bytes": 64 + 2 * part.payload_tokens,
                "sha256": part.tokens_sha256
            },
            "mask": {
                "bytes": part.payload_tokens,
                "sha256": part.mask_sha256
            }
        });
        let lineage = json!({
            "schema": "uor-r4.discrete-corpus-driver-lineage/1",
            "config": {
                "schema": "uor-r4.discrete-corpus-driver-config/1",
                "spec": spec_json,
                "read_spec": {"kind": "route"},
                "schedule": "interleaved_read_every_8_v1",
                "seed": 1,
                "batch_rows": batch_rows,
                "bos": {"id": part.bos_id, "token": "<|bos|>"},
                "dev": dev_source,
                "train": {"eligible_targets": 2_425_925}
            },
            "dev": {
                "bos": {"id": part.bos_id, "token": "<|bos|>"},
                "vocabulary": part.vocabulary,
                "documents": part.documents,
                "payload_tokens": part.payload_tokens,
                "mask_bytes": part.payload_tokens,
                "mask_selected_positions": part.eligible_rows,
                "eligible_targets": part.eligible_rows,
                "window_policy": WINDOW_POLICY,
                "source": dev_source
            },
            "train": {"eligible_targets": 2_425_925}
        });
        let lineage_sha = hash(&serde_json::to_vec(&lineage).unwrap());
        json!({
            "schema": DRIVER_SCHEMA,
            "scored_set_policy": WINDOW_POLICY,
            "evaluation_mutates_model": false,
            "updates_applied_by_evaluation": 0,
            "ordered_row_binding_sha256": part.ordered_row_binding_sha256,
            "model_bindings": {
                "schema": MODEL_SCHEMA,
                "spec": spec_json,
                "read_spec": {"kind": "route"},
                "seed": 1
            },
            "evaluation_lineage": lineage,
            "evaluation_lineage_sha256": lineage_sha,
            "batches": batches,
            "aggregate": total_agg
        })
    }

    #[test]
    fn s1_interleaved_schedule_24_step_fixture_and_cursor_round_trip() {
        for (mode, expected_param_count, expected_read_family) in [
            (ReadMode::Route, 24, ParameterFamily::Route),
            (
                ReadMode::Flock { kept: 2 },
                24,
                ParameterFamily::GateCombine,
            ),
            (
                ReadMode::Relation { kept: 2, sweeps: 1 },
                34,
                ParameterFamily::GateCombine,
            ),
        ] {
            let geom = ScheduleGeometry::new(10, 2, 3, mode, ScheduleKind::InterleavedReadEvery8V1)
                .unwrap();
            assert_eq!(geom.stages, 2);
            assert_eq!(geom.vector_parameters, 20);
            assert_eq!(geom.combine_parameters, 4);
            assert_eq!(geom.parameter_count, expected_param_count);
            assert_eq!(geom.schedule_cycle, 24);
            assert!(geom.slot(24).is_err());

            let mut read_slots = Vec::new();
            let mut vec_slots = Vec::new();
            for step in 0..24 {
                let slot = geom.slot(step).unwrap();
                if step % 8 == 7 {
                    read_slots.push((step, slot));
                } else {
                    vec_slots.push(slot.parameter_index);
                }
            }
            assert_eq!(
                read_slots,
                vec![
                    (
                        7,
                        ScheduledSlot {
                            family: expected_read_family,
                            parameter_index: 0
                        }
                    ),
                    (
                        15,
                        ScheduledSlot {
                            family: expected_read_family,
                            parameter_index: 1
                        }
                    ),
                    (
                        23,
                        ScheduledSlot {
                            family: expected_read_family,
                            parameter_index: 2
                        }
                    ),
                ]
            );
            let mut expected_vecs: Vec<u32> = (0..20).collect();
            expected_vecs.push(0);
            assert_eq!(vec_slots, expected_vecs);

            // Walk 0..=48 batches via advance_cursor and validate_cursor
            let mut cursor = CursorSpec {
                completed_batches: 0,
                accepted_updates: 0,
                corpus_pass: 0,
                next_target: 0,
                next_coordinate: 0,
            };
            geom.validate_cursor(cursor, 4, 3).unwrap();
            for batch in 0..48u64 {
                cursor = geom.advance_cursor(cursor, batch % 3 == 0, 4, 3).unwrap();
            }
            assert_eq!(cursor.completed_batches, 48);
            assert_eq!(cursor.accepted_updates, 16);
            assert_eq!(cursor.corpus_pass, 24);
            assert_eq!(cursor.next_target, 0);
            assert_eq!(cursor.next_coordinate, 0);
            assert!(geom
                .validate_cursor(
                    CursorSpec {
                        completed_batches: 24,
                        accepted_updates: 0,
                        corpus_pass: 12,
                        next_target: 0,
                        next_coordinate: 24,
                    },
                    4,
                    3,
                )
                .is_err());
        }
    }

    #[test]
    fn s2_interleaved_schedule_c192_counts_frequencies_and_wrap() {
        let route_geom = ScheduleGeometry::new(
            4096,
            4,
            192,
            ReadMode::Route,
            ScheduleKind::InterleavedReadEvery8V1,
        )
        .unwrap();
        let rel_geom = ScheduleGeometry::new(
            4096,
            4,
            192,
            ReadMode::Relation { kept: 8, sweeps: 1 },
            ScheduleKind::InterleavedReadEvery8V1,
        )
        .unwrap();
        assert_eq!(route_geom.stages, 5);
        assert_eq!(route_geom.parameter_count, 16_404);
        assert_eq!(route_geom.schedule_cycle, 16_404);
        assert_eq!(rel_geom.parameter_count, 16_429);
        assert_eq!(rel_geom.schedule_cycle, 16_404);

        let quarters = [
            (4_101u32, 3_589u32, 512u32),
            (8_202, 7_177, 1_025),
            (12_303, 10_766, 1_537),
            (16_404, 14_354, 2_050),
        ];
        for &(q, exp_vec, exp_read) in &quarters {
            let mut v = 0u32;
            let mut r = 0u32;
            for s in 0..q {
                match rel_geom.slot(s).unwrap().family {
                    ParameterFamily::VectorBlock => v += 1,
                    _ => r += 1,
                }
            }
            assert_eq!((v, r), (exp_vec, exp_read));
        }

        let mut route_freq = [0u32; 20];
        let mut gate_freq = [0u32; 45];
        let mut combine_total = 0u32;
        let mut relation_total = 0u32;
        for s in 0..16_404u32 {
            let r_slot = route_geom.slot(s).unwrap();
            if r_slot.family == ParameterFamily::Route {
                route_freq[r_slot.parameter_index as usize] += 1;
            }
            let g_slot = rel_geom.slot(s).unwrap();
            match g_slot.family {
                ParameterFamily::GateCombine => {
                    gate_freq[g_slot.parameter_index as usize] += 1;
                    combine_total += 1;
                }
                ParameterFamily::GateRelation => {
                    gate_freq[g_slot.parameter_index as usize] += 1;
                    relation_total += 1;
                }
                _ => {}
            }
        }
        assert!(route_freq[..10].iter().all(|&c| c == 103));
        assert!(route_freq[10..20].iter().all(|&c| c == 102));
        assert!(gate_freq[..25].iter().all(|&c| c == 46));
        assert!(gate_freq[25..45].iter().all(|&c| c == 45));
        assert_eq!(combine_total, 920);
        assert_eq!(relation_total, 1_130);
        rel_geom
            .validate_cursor(
                CursorSpec {
                    completed_batches: 16_404,
                    accepted_updates: 0,
                    corpus_pass: 0,
                    next_target: 1_049_856,
                    next_coordinate: 0,
                },
                2_425_925,
                64,
            )
            .unwrap();
    }

    #[test]
    fn s3_and_s4_schedule_refusals_and_split_family_counters() {
        assert!(ScheduleKind::parse(&json!("unknown_schedule")).is_err());
        assert!(ReadMode::parse(&json!({"kind":"route","kept":1})).is_err());
        assert!(ReadMode::parse(&json!({"kind":"relation","kept":0,"sweeps":1})).is_err());
        assert!(ReadMode::parse(&json!({"kind":"relation","kept":8,"sweeps":6})).is_err());

        let rel_interleaved = ScheduleGeometry::new(
            8,
            2,
            6,
            ReadMode::Relation { kept: 2, sweeps: 1 },
            ScheduleKind::InterleavedReadEvery8V1,
        )
        .unwrap();
        let rel_canonical = ScheduleGeometry::new(
            8,
            2,
            6,
            ReadMode::Relation { kept: 2, sweeps: 1 },
            ScheduleKind::CanonicalVectorThenReadV1,
        )
        .unwrap();
        let route_interleaved = ScheduleGeometry::new(
            8,
            2,
            6,
            ReadMode::Route,
            ScheduleKind::InterleavedReadEvery8V1,
        )
        .unwrap();

        let model = json!({
            "schema": MODEL_SCHEMA,
            "spec": {"vocabulary": 8, "blocks": 2, "context": 6, "kept_cells": 2},
            "read_spec": {"kind": "relation", "kept": 2, "sweeps": 1},
            "seed": 1,
            "epoch": 2,
            "table": vec![0i32; 8 * 2 * 8],
            "routes": vec![0u32; 4],
            "gates": vec![0i64; 14]
        });
        let model_sha = hash(&serde_json::to_vec(&model).unwrap());
        let ckpt = json!({
            "schema": CHECKPOINT_SCHEMA,
            "lineage": {
                "config": {
                    "spec": {"vocabulary": 8, "blocks": 2, "context": 6, "kept_cells": 2},
                    "read_spec": {"kind": "relation", "kept": 2, "sweeps": 1},
                    "schedule": "interleaved_read_every_8_v1",
                    "seed": 1,
                    "batch_rows": 4
                },
                "train_corpus": {"eligible_targets": 20}
            },
            "cursor": {
                "completed_batches": 20,
                "accepted_updates": 2,
                "corpus_pass": 4,
                "next_target": 0,
                "next_coordinate": 0
            },
            "model_sha256": model_sha,
            "model": model
        });
        let resumed = rel_interleaved
            .validate_checkpoint_resume(&ckpt, 1, 20, 4)
            .unwrap();
        assert_eq!(resumed.completed_batches, 20);
        // S3: refuse cross-schedule resume (Canonical vs Interleaved)
        assert!(rel_canonical
            .validate_checkpoint_resume(&ckpt, 1, 20, 4)
            .is_err());
        // S3: refuse cross-read_spec resume (Route vs Relation)
        assert!(route_interleaved
            .validate_checkpoint_resume(&ckpt, 1, 20, 4)
            .is_err());
        // S3: refuse wrong seed
        assert!(rel_interleaved
            .validate_checkpoint_resume(&ckpt, 2, 20, 4)
            .is_err());

        let rel_geom = ScheduleGeometry::new(
            4096,
            4,
            192,
            ReadMode::Relation { kept: 8, sweeps: 1 },
            ScheduleKind::InterleavedReadEvery8V1,
        )
        .unwrap();
        let sens = json!({
            "vector_block": {"attempted": 14354, "any_candidate_cost_differs_from_identity": 3000, "accepted_updates": 250},
            "gate_combine": {"attempted": 920, "any_candidate_cost_differs_from_identity": 100, "accepted_updates": 12},
            "gate_relation": {"attempted": 1130, "any_candidate_cost_differs_from_identity": 150, "accepted_updates": 18}
        });
        let verified = verify_split_family_counters(&sens, &rel_geom, 16_404, 280).unwrap();
        assert_eq!(verified["totals"]["attempted"], 16_404);
        assert_eq!(
            verified["totals"]["any_candidate_cost_differs_from_identity"],
            3250
        );
        assert_eq!(verified["totals"]["accepted_updates"], 280);

        let mut bad_sens = sens.clone();
        bad_sens["gate_relation"]["attempted"] = json!(1129);
        assert!(verify_split_family_counters(&bad_sens, &rel_geom, 16_404, 280).is_err());
        assert!(verify_split_family_counters(&sens, &rel_geom, 16_404, 279).is_err());
    }

    #[test]
    fn e1_and_e2_copy_successor_subpanel_partition_and_additive_aggregates() {
        // Doc 0: [BOS(0), x(1), A(2), B(3), y(4), A(2), B(3), A(2), C(5)]
        // Eligible targets (mask=1):
        // p=2 (target=2, query=1): no prior 1 -> non-recur (complement)
        // p=3 (target=3, query=2): no prior 2 in 0..2 -> non-recur (complement)
        // p=6 (target=3, query=2 at p-1=5): prior 2 at j=2 (d = 5-2 = 3 in 1..=8), tokens[3]==3 == target -> copy_successor!
        // p=8 (target=5, query=2 at p-1=7): most recent 2 at j=5 has tokens[6]=3 != 5 -> query_recurs but NOT copy_successor (complement)
        let tokens = [0u16, 1, 2, 3, 4, 2, 3, 2, 5];
        let mask = [0u8, 0, 1, 1, 0, 0, 1, 0, 1];
        let uort = build_uort(10, &tokens);
        let part = partition_corpus(&uort, &mask, 6, 0).unwrap();
        assert_eq!(part.eligible_rows, 4);
        assert_eq!(part.query_recur_rows, 2);
        assert_eq!(part.copy_successor_rows, 1);
        assert_eq!(part.distance_bucket_rows, [1, 0, 0, 0]);
        assert_eq!(part.distance_one_rows, 0);
        assert_eq!(part.any_occurrence_successor_rows, 1);
        assert_eq!(part.complement_rows, 3);

        let eval = build_eval_doc(
            &part,
            &[
                (16, 0, 7, 0, 1), // p=2 (comp): strict win, top1=1, top8=1, B3=1
                (32, 8, 8, 1, 1), // p=3 (comp): not admitted, top1=0, top8=0, B3=0
                (8, 0, 7, 0, 1),  // p=6 (succ): strict win, top1=1, top8=1, B3=1
                (24, 0, 7, 0, 1), // p=8 (comp): strict win, top1=1, top8=1, B3=1
            ],
            4,
        );
        let audit = audit_evaluation_subpanels(&part, &eval, 4).unwrap();
        assert_eq!(audit.full_panel["scored_rows"], 4);
        assert_eq!(audit.copy_successor["scored_rows"], 1);
        assert_eq!(audit.complement["scored_rows"], 3);
        assert_eq!(audit.copy_successor["top8_correct"], 1);
        assert_eq!(audit.complement["top8_correct"], 2);
        assert_eq!(audit.full_panel["top8_correct"], 3);
        assert_eq!(audit.copy_successor["target_cell_admitted_rows"], 1);
        assert_eq!(audit.complement["target_cell_admitted_rows"], 2);
        assert_eq!(audit.full_panel["target_cell_admitted_rows"], 3);

        let cmp = compare_evaluation_subpanels(&part, &eval, &eval, 4).unwrap();
        assert_eq!(
            cmp["aggregate_deltas_after_minus_before"]["copy_successor_subpanel"]["top8_correct"],
            0
        );

        // P1: Refuse wrong corpus tokens SHA256, mask SHA256, context, or bos.id in evaluation_lineage
        let mut bad_tok_sha = eval.clone();
        bad_tok_sha["evaluation_lineage"]["config"]["dev"]["tokens"]["sha256"] =
            json!("f".repeat(64));
        bad_tok_sha["evaluation_lineage"]["dev"]["source"]["tokens"]["sha256"] =
            json!("f".repeat(64));
        assert!(audit_evaluation_subpanels(&part, &bad_tok_sha, 4).is_err());

        let mut bad_mask_sha = eval.clone();
        bad_mask_sha["evaluation_lineage"]["config"]["dev"]["mask"]["sha256"] =
            json!("f".repeat(64));
        bad_mask_sha["evaluation_lineage"]["dev"]["source"]["mask"]["sha256"] =
            json!("f".repeat(64));
        assert!(audit_evaluation_subpanels(&part, &bad_mask_sha, 4).is_err());

        let mut bad_context = eval.clone();
        bad_context["evaluation_lineage"]["config"]["spec"]["context"] = json!(192);
        bad_context["model_bindings"]["spec"]["context"] = json!(192);
        assert!(audit_evaluation_subpanels(&part, &bad_context, 4).is_err());

        let mut bad_bos = eval.clone();
        bad_bos["evaluation_lineage"]["config"]["bos"]["id"] = json!(1);
        bad_bos["evaluation_lineage"]["dev"]["bos"]["id"] = json!(1);
        assert!(audit_evaluation_subpanels(&part, &bad_bos, 4).is_err());

        // P1: Swapping row ordinal 1 (p=3, complement, top8=0) and row ordinal 2 (p=6, copy_successor, top8=1)
        // within batch 0 preserves batch and full-panel aggregates, but MUST be refused by both
        // audit_evaluation_subpanels and compare_evaluation_subpanels!
        let mut swapped_eval = eval.clone();
        let row1 = swapped_eval["batches"][0]["rows"][1].clone();
        let row2 = swapped_eval["batches"][0]["rows"][2].clone();
        swapped_eval["batches"][0]["rows"][1] = row2;
        swapped_eval["batches"][0]["rows"][2] = row1;
        assert!(audit_evaluation_subpanels(&part, &swapped_eval, 4).is_err());
        assert!(compare_evaluation_subpanels(&part, &eval, &swapped_eval, 4).is_err());

        // P1: Even if row_ordinal and target_position are stripped from a row, audit_evaluation_subpanels refuses it
        let mut stripped_row_eval = eval.clone();
        stripped_row_eval["batches"][0]["rows"][0]
            .as_object_mut()
            .unwrap()
            .remove("target_position");
        assert!(audit_evaluation_subpanels(&part, &stripped_row_eval, 4).is_err());

        // P1: Violating top8 target membership consistency (top8_correct=1 when target_token is not in top8) is refused
        let mut bad_top8_eval = eval.clone();
        bad_top8_eval["batches"][0]["rows"][0]["top8"] = json!([0, 1, 3, 4, 5, 6, 7, 8]); // missing target_token 2
        assert!(audit_evaluation_subpanels(&part, &bad_top8_eval, 4).is_err());

        // Verify row/aggregate invariant refusals
        let mut bad_eval = eval.clone();
        bad_eval["aggregate"]["top8_correct"] = json!(99);
        assert!(audit_evaluation_subpanels(&part, &bad_eval, 4).is_err());

        let mut bad_row = eval["batches"][0]["rows"][0].clone();
        bad_row["token_rivals"] = json!(0);
        bad_row["token_hinge"] = json!(8);
        assert!(parse_row_metrics(&bad_row, 10).is_err());

        // Low 3: negative token_hinge and negative address_hinge are rejected
        let mut neg_tok_row = eval["batches"][0]["rows"][0].clone();
        neg_tok_row["token_hinge"] = json!(-1);
        neg_tok_row["total_loss_numerator"] = json!(-8);
        assert!(parse_row_metrics(&neg_tok_row, 10).is_err());

        let mut neg_addr_row = eval["batches"][0]["rows"][0].clone();
        neg_addr_row["address_hinge"] = json!(-1);
        neg_addr_row["total_loss_numerator"] = json!(16 * 8 - 32);
        assert!(parse_row_metrics(&neg_addr_row, 10).is_err());
    }

    #[test]
    fn e3_zero_trit_attribution_snapshot_invariance_and_b1_b2_b3_bars() {
        // Test C6/B2 (4 routes, 14 gates) AND C192/B4 (20 routes, 45 gates)
        let mut gates = vec![1i64; 4]; // 4 combine gates (B=2, S=2)
        gates.extend_from_slice(&[0, 1, -1, 0, 1, 0, 0, -1, 1, 0]); // 10 relation trits
        let model = json!({
            "schema": MODEL_SCHEMA,
            "spec": {"vocabulary": 8, "blocks": 2, "context": 6, "kept_cells": 2},
            "read_spec": {"kind": "relation", "kept": 2, "sweeps": 1},
            "seed": 1,
            "epoch": 5,
            "table": vec![0i32; 8 * 2 * 8],
            "routes": vec![0u32; 4],
            "gates": gates
        });
        // Standalone Snapshot (MODEL_SCHEMA) works directly
        let (zeroed_snap, snap_summary) = prepare_zero_trit_checkpoint(&model).unwrap();
        assert_eq!(snap_summary["nonzero_relation_trits_zeroed"], 5);
        assert_eq!(
            snap_summary["relation_trit_table_5x_stages"]["parent_child"],
            json!([0, 1])
        );
        assert_eq!(zeroed_snap["routes"], json!([0, 0, 0, 0]));

        // Empty routes array on Relation snapshot is refused (matches device.rs:632)
        let mut empty_routes_model = model.clone();
        empty_routes_model["routes"] = json!([]);
        assert!(prepare_zero_trit_checkpoint(&empty_routes_model).is_err());

        let model_sha = hash(&serde_json::to_vec(&model).unwrap());
        let ckpt = json!({
            "schema": CHECKPOINT_SCHEMA,
            "model_sha256": model_sha,
            "model": model
        });
        let (zeroed, summary) = prepare_zero_trit_checkpoint(&ckpt).unwrap();
        assert_eq!(summary["nonzero_relation_trits_zeroed"], 5);
        assert_eq!(summary["byte_identical_when_already_zero"], false);
        let zeroed_gates = zeroed["model"]["gates"].as_array().unwrap();
        assert!(zeroed_gates[..4].iter().all(|g| g == 1));
        assert!(zeroed_gates[4..14].iter().all(|g| g == 0));

        // Calling prepare_zero_trit_checkpoint a second time on already-zeroed checkpoint is byte-identical
        let (zeroed_again, summary2) = prepare_zero_trit_checkpoint(&zeroed).unwrap();
        assert_eq!(summary2["nonzero_relation_trits_zeroed"], 0);
        assert_eq!(summary2["byte_identical_when_already_zero"], true);
        assert_eq!(zeroed_again, zeroed);

        // C192/B4 (V=8, B=4, C=192 -> S=5, 20 routes, 20 combine gates + 25 relation trits = 45 gates)
        let mut c192_gates = vec![-1i64; 20];
        c192_gates.extend((0..25).map(|i| (i % 3) - 1));
        let c192_model = json!({
            "schema": MODEL_SCHEMA,
            "spec": {"vocabulary": 8, "blocks": 4, "context": 192, "kept_cells": 8},
            "read_spec": {"kind": "relation", "kept": 8, "sweeps": 1},
            "seed": 1,
            "epoch": 12,
            "table": vec![2i32; 8 * 4 * 8],
            "routes": vec![7u32; 20],
            "gates": c192_gates
        });
        let (c192_zeroed, c192_sum) = prepare_zero_trit_checkpoint(&c192_model).unwrap();
        assert_eq!(c192_sum["combine_gate_count"], 20);
        assert_eq!(c192_sum["relation_trit_count"], 25);
        assert_eq!(c192_sum["nonzero_relation_trits_zeroed"], 17);
        let c192_g = c192_zeroed["gates"].as_array().unwrap();
        assert!(c192_g[..20].iter().all(|g| g == -1));
        assert!(c192_g[20..45].iter().all(|g| g == 0));

        // B1, B2, B3 bar evaluation test + DiscriminatorSpreads::from_seed_pair
        let mut route_audit = SubpanelEvaluationAudit {
            full_panel: zero_slice(),
            copy_successor: zero_slice(),
            distance_buckets: [zero_slice(), zero_slice(), zero_slice(), zero_slice()],
            any_occurrence_successor: zero_slice(),
            complement: zero_slice(),
        };
        route_audit
            .copy_successor
            .insert("top8_correct".into(), 200);
        route_audit.copy_successor.insert("top1_correct".into(), 20);
        route_audit
            .copy_successor
            .insert("target_cell_admitted_rows".into(), 500);
        route_audit.full_panel.insert("top8_correct".into(), 1000);
        route_audit.full_panel.insert("top1_correct".into(), 95);
        route_audit
            .full_panel
            .insert("target_cell_admitted_rows".into(), 2000);

        let mut route_seed2 = route_audit.clone();
        route_seed2
            .copy_successor
            .insert("top8_correct".into(), 242); // |200 - 242| = 42 > 39
        let pair_spreads =
            DiscriminatorSpreads::from_seed_pair(&route_audit, &route_seed2).unwrap();
        assert_eq!(pair_spreads.succ_top8_spread, 42);
        assert_eq!(pair_spreads.full_top1_spread, 20);
        assert!(pair_spreads.two_seed_measured);

        let mut rel_audit = route_audit.clone();
        rel_audit.copy_successor.insert("top8_correct".into(), 245); // +45 > 42 and > 39
        rel_audit.full_panel.insert("top8_correct".into(), 1045);

        let mut zero_audit = route_audit.clone();
        zero_audit.copy_successor.insert("top8_correct".into(), 210); // 245 > 210

        let pass_verdict =
            evaluate_m2_discriminator_bars(&route_audit, &rel_audit, &zero_audit, 3, pair_spreads)
                .unwrap();
        assert_eq!(
            pass_verdict["b1_copy_successor_subpanel"]["spread_mode"],
            "two_seed_measured_with_39_floor_top8_and_0_floor_side_guards"
        );
        assert_eq!(pass_verdict["b1_copy_successor_subpanel"]["pass"], true);
        assert_eq!(pass_verdict["b2_full_panel_non_regression"]["pass"], true);
        assert_eq!(pass_verdict["b3_zero_trit_attribution"]["pass"], true);
        assert_eq!(
            pass_verdict["overall_outcome"],
            "KEEP_CANDIDATE_REQUIRES_TWO_SEEDS_AND_FRESH_TRANSFER"
        );

        let single_seed_pass = evaluate_m2_discriminator_bars(
            &route_audit,
            &rel_audit,
            &zero_audit,
            3,
            DiscriminatorSpreads::default(),
        )
        .unwrap();
        assert_eq!(
            single_seed_pass["b1_copy_successor_subpanel"]["spread_mode"],
            "floor_only_not_sheet_b1"
        );
        assert_eq!(
            single_seed_pass["overall_outcome"],
            "SINGLE_SEED_FLOOR_ONLY_NOT_SHEET_B1_REQUIRES_TWO_SEEDS_AND_FRESH_TRANSFER"
        );

        // If zero-trit attribution delta is 0 or gate_relation_accepted_updates == 0, B3 fails
        let fail_verdict = evaluate_m2_discriminator_bars(
            &route_audit,
            &rel_audit,
            &rel_audit,
            3,
            DiscriminatorSpreads::default(),
        )
        .unwrap();
        assert_eq!(fail_verdict["b3_zero_trit_attribution"]["pass"], false);
        assert_eq!(
            fail_verdict["overall_outcome"],
            "NEGATIVE_CLOSES_THIS_CONFIGURATION_ONLY_D22"
        );
    }

    #[test]
    fn filesystem_cli_round_trip_no_clobber_and_oversized_refusal() {
        let dir = std::env::temp_dir().join(format!(
            "relation-subpanel-audit-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let tokens = [0u16, 1, 2, 3, 1, 2];
        let mask = [0u8, 0, 1, 0, 0, 1];
        let tokens_path = dir.join("tokens.u16");
        let mask_path = dir.join("mask.u8");
        let eval_path = dir.join("eval.json");
        let out_path = dir.join("out.json");
        let uort_bytes = build_uort(4096, &tokens);
        fs::write(&tokens_path, &uort_bytes).unwrap();
        fs::write(&mask_path, mask).unwrap();
        let part = partition_corpus(&uort_bytes, &mask, 192, 0).unwrap();
        let eval = build_eval_doc(&part, &[(16, 0, 7, 0, 1), (8, 0, 7, 0, 1)], 64);
        fs::write(&eval_path, serde_json::to_vec(&eval).unwrap()).unwrap();

        run_subpanel_cli(&tokens_path, &mask_path, &eval_path, &out_path).unwrap();
        let saved = fs::read(&out_path).unwrap();
        // Refuses to clobber existing output file
        assert!(run_subpanel_cli(&tokens_path, &mask_path, &eval_path, &out_path).is_err());
        assert_eq!(fs::read(&out_path).unwrap(), saved);

        // Build complete discriminator trios for seed 1 and seed 2 with mandatory model_bindings,
        // evaluation_lineage, completed_batches == 16404, model_epoch == accepted_updates_in_model,
        // split-family sensitivity counters, and source_relation_model/checkpoint_sha256 linkage
        let spec_json = json!({"vocabulary": 4096, "blocks": 4, "context": 192, "kept_cells": 8});
        let make_trio = |seed: u64| -> (Value, Value, Value) {
            let mut r = eval.clone();
            r["completed_batches"] = json!(16404);
            r["cursor"] = json!({"completed_batches": 16404, "accepted_updates": 10, "corpus_pass": 0, "next_target": 1049856, "next_coordinate": 0});
            r["model_epoch"] = json!(10);
            r["accepted_updates_in_model"] = json!(10);
            r["model_snapshot_sha256"] = json!("a".repeat(64));
            r["model_bindings"] = json!({
                "schema": MODEL_SCHEMA,
                "spec": spec_json,
                "read_spec": {"kind": "route"},
                "seed": seed
            });
            r["evaluation_lineage"]["config"]["seed"] = json!(seed);
            r["evaluation_lineage"]["config"]["read_spec"] = json!({"kind": "route"});
            r["evaluation_lineage_sha256"] =
                json!(hash(&serde_json::to_vec(&r["evaluation_lineage"]).unwrap()));
            r["sensitivity"] = json!({
                "vector_block": {"attempted": 14354, "any_candidate_cost_differs_from_identity": 100, "accepted_updates": 8},
                "route": {"attempted": 2050, "any_candidate_cost_differs_from_identity": 20, "accepted_updates": 2}
            });

            let mut rel = eval.clone();
            rel["completed_batches"] = json!(16404);
            rel["cursor"] = json!({"completed_batches": 16404, "accepted_updates": 12, "corpus_pass": 0, "next_target": 1049856, "next_coordinate": 0});
            rel["model_epoch"] = json!(12);
            rel["accepted_updates_in_model"] = json!(12);
            rel["model_snapshot_sha256"] = json!("b".repeat(64));
            rel["driver_checkpoint_sha256"] = json!("d".repeat(64));
            rel["model_bindings"] = json!({
                "schema": MODEL_SCHEMA,
                "spec": spec_json,
                "read_spec": {"kind": "relation", "kept": 8, "sweeps": 1},
                "seed": seed
            });
            rel["evaluation_lineage"]["config"]["seed"] = json!(seed);
            rel["evaluation_lineage"]["config"]["read_spec"] =
                json!({"kind": "relation", "kept": 8, "sweeps": 1});
            rel["evaluation_lineage_sha256"] = json!(hash(
                &serde_json::to_vec(&rel["evaluation_lineage"]).unwrap()
            ));
            rel["sensitivity"] = json!({
                "vector_block": {"attempted": 14354, "any_candidate_cost_differs_from_identity": 100, "accepted_updates": 9},
                "gate_combine": {"attempted": 920, "any_candidate_cost_differs_from_identity": 10, "accepted_updates": 1},
                "gate_relation": {"attempted": 1130, "any_candidate_cost_differs_from_identity": 15, "accepted_updates": 2}
            });

            let mut zero = eval.clone();
            zero["completed_batches"] = json!(16404);
            zero["cursor"] = json!({"completed_batches": 16404, "accepted_updates": 12, "corpus_pass": 0, "next_target": 1049856, "next_coordinate": 0});
            zero["model_epoch"] = json!(12);
            zero["accepted_updates_in_model"] = json!(12);
            zero["model_snapshot_sha256"] = json!("c".repeat(64));
            zero["source_relation_model_sha256"] = json!("b".repeat(64));
            zero["source_relation_checkpoint_sha256"] = json!("d".repeat(64));
            zero["model_bindings"] = rel["model_bindings"].clone();
            zero["evaluation_lineage"] = rel["evaluation_lineage"].clone();
            zero["evaluation_lineage_sha256"] = rel["evaluation_lineage_sha256"].clone();
            (r, rel, zero)
        };

        let (r_s1, rel_s1, zero_s1) = make_trio(1);
        let (r_s2, rel_s2, zero_s2) = make_trio(2);

        // Verify mandatory discriminator refusals:
        // 1. Missing model_bindings is refused
        let mut no_bindings = r_s1.clone();
        no_bindings
            .as_object_mut()
            .unwrap()
            .remove("model_bindings");
        assert!(
            validate_discriminator_seed_trio(&part, &no_bindings, &rel_s1, &zero_s1, 64).is_err()
        );

        // 2. Wrong decision cursor (completed_batches != 16404) is refused
        let mut wrong_cursor = rel_s1.clone();
        wrong_cursor["completed_batches"] = json!(12303);
        wrong_cursor["cursor"]["completed_batches"] = json!(12303);
        assert!(
            validate_discriminator_seed_trio(&part, &r_s1, &wrong_cursor, &zero_s1, 64).is_err()
        );

        // 3. Mismatched source_relation_model_sha256 is refused
        let mut wrong_source_sha = zero_s1.clone();
        wrong_source_sha["source_relation_model_sha256"] = json!("e".repeat(64));
        assert!(
            validate_discriminator_seed_trio(&part, &r_s1, &rel_s1, &wrong_source_sha, 64).is_err()
        );

        // 4. Missing or mismatched source_relation_checkpoint_sha256 is refused
        let mut missing_ckpt_sha = zero_s1.clone();
        missing_ckpt_sha
            .as_object_mut()
            .unwrap()
            .remove("source_relation_checkpoint_sha256");
        assert!(
            validate_discriminator_seed_trio(&part, &r_s1, &rel_s1, &missing_ckpt_sha, 64).is_err()
        );

        // 5. Mismatched model_epoch between Relation and zero-trit is refused
        let mut wrong_epoch = zero_s1.clone();
        wrong_epoch["model_epoch"] = json!(11);
        wrong_epoch["accepted_updates_in_model"] = json!(11);
        wrong_epoch["cursor"]["accepted_updates"] = json!(11);
        assert!(validate_discriminator_seed_trio(&part, &r_s1, &rel_s1, &wrong_epoch, 64).is_err());

        // 6. P1: Route and Relation evaluation_lineage mismatch on train/schedule is refused
        let mut wrong_route_sched = r_s1.clone();
        wrong_route_sched["evaluation_lineage"]["config"]["schedule"] =
            json!("canonical_vector_then_read_v1");
        assert!(
            validate_discriminator_seed_trio(&part, &wrong_route_sched, &rel_s1, &zero_s1, 64)
                .is_err()
        );

        // 7. P2: Incomplete sensitivity or gate_relation.accepted_updates = 13 > model_epoch = 12 is refused
        let mut bad_rel_sens_incomplete = rel_s1.clone();
        bad_rel_sens_incomplete["sensitivity"] = json!({"gate_relation": {"accepted_updates": 2}});
        assert!(validate_discriminator_seed_trio(
            &part,
            &r_s1,
            &bad_rel_sens_incomplete,
            &zero_s1,
            64
        )
        .is_err());

        let mut bad_rel_sens_overflow = rel_s1.clone();
        bad_rel_sens_overflow["sensitivity"]["gate_relation"]["accepted_updates"] = json!(13);
        assert!(validate_discriminator_seed_trio(
            &part,
            &r_s1,
            &bad_rel_sens_overflow,
            &zero_s1,
            64
        )
        .is_err());

        let r_s1_path = dir.join("r-s1.json");
        let rel_s1_path = dir.join("rel-s1.json");
        let zero_s1_path = dir.join("zero-s1.json");
        let r_s2_path = dir.join("r-s2.json");
        let rel_s2_path = dir.join("rel-s2.json");
        let zero_s2_path = dir.join("zero-s2.json");
        for (p, doc) in [
            (&r_s1_path, &r_s1),
            (&rel_s1_path, &rel_s1),
            (&zero_s1_path, &zero_s1),
            (&r_s2_path, &r_s2),
            (&rel_s2_path, &rel_s2),
            (&zero_s2_path, &zero_s2),
        ] {
            fs::write(p, serde_json::to_vec(doc).unwrap()).unwrap();
        }

        let disc_out = dir.join("disc.json");
        run_discriminator_cli(
            &tokens_path,
            &mask_path,
            &r_s1_path,
            &rel_s1_path,
            &zero_s1_path,
            &disc_out,
        )
        .unwrap();
        assert!(disc_out.exists());

        // Two-seed discriminator refuses identical seeds (seed 1 paired with seed 1)
        let same_seed_out = dir.join("disc-same-seed.json");
        assert!(run_discriminator_two_seed_cli(
            &tokens_path,
            &mask_path,
            &r_s1_path,
            &rel_s1_path,
            &zero_s1_path,
            &r_s1_path,
            &rel_s1_path,
            &zero_s1_path,
            &same_seed_out,
        )
        .is_err());

        let disc_two_out = dir.join("disc-two-seed.json");
        run_discriminator_two_seed_cli(
            &tokens_path,
            &mask_path,
            &r_s1_path,
            &rel_s1_path,
            &zero_s1_path,
            &r_s2_path,
            &rel_s2_path,
            &zero_s2_path,
            &disc_two_out,
        )
        .unwrap();
        assert!(disc_two_out.exists());

        let mut gates = vec![1i64; 4];
        gates.extend_from_slice(&[0, 1, -1, 0, 1, 0, 0, -1, 1, 0]);
        let model = json!({
            "schema": MODEL_SCHEMA,
            "spec": {"vocabulary": 8, "blocks": 2, "context": 6, "kept_cells": 2},
            "read_spec": {"kind": "relation", "kept": 2, "sweeps": 1},
            "seed": 1,
            "epoch": 5,
            "table": vec![0i32; 8 * 2 * 8],
            "routes": vec![0u32; 4],
            "gates": gates
        });
        let model_sha = hash(&serde_json::to_vec(&model).unwrap());
        let ckpt = json!({
            "schema": CHECKPOINT_SCHEMA,
            "model_sha256": model_sha,
            "model": model
        });
        let ckpt_in = dir.join("ckpt-in.json");
        let ckpt_out = dir.join("ckpt-zero.json");
        fs::write(&ckpt_in, serde_json::to_vec(&ckpt).unwrap()).unwrap();
        run_zero_trits_cli(&ckpt_in, &ckpt_out).unwrap();
        assert!(ckpt_out.exists());

        let oversized = dir.join("oversized.json");
        fs::File::create(&oversized)
            .unwrap()
            .set_len(MAX_FILE_BYTES + 1)
            .unwrap();
        assert!(read_bounded_bytes(&oversized).is_err());
        fs::remove_dir_all(&dir).unwrap();
    }
}
